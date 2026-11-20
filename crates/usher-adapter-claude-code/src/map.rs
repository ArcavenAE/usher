//! The settings map reader (Story 1.6, FR-14): which value is in effect for each setting
//! knowledge lists, from which layer, and why.
//!
//! A pure function over the source listing and the files it names. The order of layers,
//! the keys, their merge kinds, their scopes and their overrides all come from knowledge.
//! A key knowledge does not list is walked to its leaf paths and printed by name only, and
//! no value of a sensitive or unlisted key is kept. The harness is never run.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::Value;
use usher_core::{
    LaunchContext, NotObserved, ReadingMethod, ReadingStatus, SettingReading, SettingsMap,
    SkippedLayer, SourceListing, SourceStatus,
};
use usher_knowledge::{Item, KnowledgeFile, MergeKind, Scopes, SettingsKeyFact, Version};

use crate::{HARNESS, layer_name};

const NOT_GIVEN: &str = "reason: not given (usher does not call /status)";
const DEPTH_LIMIT: usize = 8;
const UNLISTED: &str = "key not marked readable";
const SENSITIVE_CHILD: &str = "key is sensitive";
const CLASSIFIER_PREFIX: &str = "autoMode.";

/// One listed key: the fact and the item that states it.
struct Listed<'a> {
    item: &'a Item,
    fact: &'a SettingsKeyFact,
    segments: Vec<&'a str>,
}

/// One source file read for the map.
struct Layer<'a> {
    name: &'a str,
    rank: usize,
    path: &'a Path,
    item_id: &'a str,
    /// How the source takes part: it decides values, it is skipped, or its version is not known.
    role: Role<'a>,
    /// The parsed top-level object, when the file could be read.
    object: Option<serde_json::Map<String, Value>>,
}

enum Role<'a> {
    Decides,
    Skipped,
    Unknown(&'a str),
}

/// One occurrence of a listed key in one layer.
struct Occurrence<'a> {
    layer: &'a Layer<'a>,
    value: &'a Value,
}

/// A row with the key's segments, for sorting.
struct Row {
    segments: Vec<String>,
    rank: usize,
    reading: SettingReading,
}

pub(crate) fn map_settings(
    knowledge: &KnowledgeFile,
    listing: &SourceListing,
    context: &LaunchContext,
) -> SettingsMap {
    let installed = context
        .harness_version
        .as_deref()
        .and_then(|text| Version::parse(text).ok());
    let stack = layer_stack(knowledge, installed.as_ref());
    let listed: Vec<Listed<'_>> = match &installed {
        Some(version) => knowledge
            .items_for(HARNESS, version)
            .filter_map(|item| {
                item.settings_key().map(|fact| Listed {
                    item,
                    fact,
                    segments: fact.key().split('.').collect(),
                })
            })
            .collect(),
        None => Vec::new(),
    };

    let mut skipped_layers: Vec<SkippedLayer> = Vec::new();
    let mut missing: BTreeSet<String> = BTreeSet::new();
    let mut layers: Vec<Layer<'_>> = Vec::new();
    for source in &listing.sources {
        let Some(rank) = stack.iter().position(|name| *name == source.layer) else {
            missing.insert(source.layer.clone());
            continue;
        };
        let role = match &source.status {
            SourceStatus::InEffect => Role::Decides,
            SourceStatus::Unread { reason } => {
                skipped_layers.push(SkippedLayer {
                    layer: source.layer.clone(),
                    reason: format!("{reason}; {NOT_GIVEN}"),
                });
                Role::Skipped
            }
            SourceStatus::Shadowed { by } => {
                skipped_layers.push(SkippedLayer {
                    layer: source.layer.clone(),
                    reason: format!("shadowed by {}; {NOT_GIVEN}", by.display()),
                });
                Role::Skipped
            }
            SourceStatus::Unknown { reason } => Role::Unknown(reason),
            _ => Role::Unknown("status not recognized"),
        };
        layers.push(Layer {
            name: &source.layer,
            rank,
            path: &source.path,
            item_id: &source.item_id,
            role,
            object: read_object(&source.path),
        });
    }
    layers.sort_by_key(|layer| layer.rank);

    let mut rows: Vec<Row> = Vec::new();
    let mut occurrences: Vec<Vec<Occurrence<'_>>> = listed.iter().map(|_| Vec::new()).collect();
    for layer in &layers {
        let Some(object) = &layer.object else {
            continue;
        };
        if let Role::Unknown(reason) = &layer.role {
            for name in object.keys() {
                rows.push(row(
                    vec![name.clone()],
                    layer,
                    ReadingStatus::Unknown,
                    None,
                    None,
                    Some((*reason).to_string()),
                ));
            }
            continue;
        }
        let mut path: Vec<String> = Vec::new();
        walk(
            object,
            &mut path,
            layer,
            &listed,
            &mut occurrences,
            &mut rows,
            UNLISTED,
        );
    }

    let broken: Vec<&Layer<'_>> = layers
        .iter()
        .filter(|l| match l.role {
            Role::Decides => l.object.is_none(),
            Role::Unknown(_) => true,
            Role::Skipped => false,
        })
        .collect();
    for (key, found) in listed.iter().zip(&occurrences) {
        decide(key, found, &broken, context, &mut rows);
    }

    rows.sort_by(|a, b| {
        (&a.segments, a.rank, a.reading.entry_index).cmp(&(
            &b.segments,
            b.rank,
            b.reading.entry_index,
        ))
    });

    let mut not_observed = vec![
        NotObserved::CommandLine,
        NotObserved::EnvValues,
        NotObserved::ServerManagedValues,
    ];
    if listed.iter().any(|key| key.fact.key() == "model") {
        not_observed.push(NotObserved::OrganizationDefaultModel);
    }
    let classifier_set = rows.iter().any(|r| {
        r.reading.status == ReadingStatus::InEffect
            && listed.iter().any(|key| {
                key.fact.merge() == MergeKind::ListMerge
                    && key.fact.key().starts_with(CLASSIFIER_PREFIX)
                    && key.fact.key() == r.reading.key
            })
    });
    if classifier_set {
        not_observed.push(NotObserved::BuiltInClassifierRules);
    }
    not_observed.extend(missing.into_iter().map(NotObserved::LayerMissingFromStack));

    SettingsMap {
        harness: listing.harness.clone(),
        harness_version: listing.harness_version.clone(),
        readings: rows.into_iter().map(|r| r.reading).collect(),
        skipped_layers,
        not_observed,
    }
}

/// The layer order from knowledge, highest first. Without a stack no layer is a settings
/// layer and the map has no rows.
fn layer_stack<'a>(knowledge: &'a KnowledgeFile, installed: Option<&Version>) -> Vec<&'a str> {
    knowledge
        .items()
        .iter()
        .filter(|item| item.harness() == HARNESS && !item.stack().is_empty())
        .find(|item| installed.is_none_or(|v| item.versions().contains(v)))
        .map(|item| item.stack().iter().map(String::as_str).collect())
        .unwrap_or_default()
}

fn read_object(path: &Path) -> Option<serde_json::Map<String, Value>> {
    let bytes = fs::read(path).ok()?;
    match serde_json::from_slice::<Value>(&bytes).ok()? {
        Value::Object(map) => Some(map),
        _ => None,
    }
}

fn row(
    segments: Vec<String>,
    layer: &Layer<'_>,
    status: ReadingStatus,
    entry_index: Option<u32>,
    value: Option<Value>,
    note: Option<String>,
) -> Row {
    Row {
        reading: SettingReading {
            key: print_path(&segments),
            layer: layer.name.to_string(),
            source_path: layer.path.to_path_buf(),
            method: ReadingMethod::Rebuilt,
            status,
            entry_index,
            value,
            note,
        },
        segments,
        rank: layer.rank,
    }
}

/// Segments joined with `.`; a segment with a dot, whitespace or a quote prints as a JSON
/// string so it stays one segment.
fn print_path(segments: &[String]) -> String {
    segments
        .iter()
        .map(|segment| {
            if segment.contains(['.', '"']) || segment.chars().any(char::is_whitespace) {
                Value::String(segment.clone()).to_string()
            } else {
                segment.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// Walks one object. A listed key is collected for deciding, the parents of listed keys are
/// descended, and everything else is walked to leaf paths and printed by name only.
fn walk<'a>(
    object: &'a serde_json::Map<String, Value>,
    path: &mut Vec<String>,
    layer: &'a Layer<'a>,
    listed: &[Listed<'_>],
    occurrences: &mut [Vec<Occurrence<'a>>],
    rows: &mut Vec<Row>,
    leaf_note: &str,
) {
    for (name, value) in object {
        path.push(name.clone());
        let found = listed.iter().position(|key| {
            key.segments
                .iter()
                .copied()
                .eq(path.iter().map(String::as_str))
        });
        if KnowledgeFile::is_credential_key(name) {
            rows.push(row(
                path.clone(),
                layer,
                ReadingStatus::ValueNotRead,
                None,
                None,
                Some("credential key; not opened".to_string()),
            ));
        } else if let Some(index) = found {
            occurrences[index].push(Occurrence { layer, value });
        } else if value.is_object()
            && listed.iter().any(|key| {
                key.segments.len() > path.len()
                    && key.segments.iter().zip(path.iter()).all(|(a, b)| *a == b)
            })
        {
            if let Value::Object(child) = value {
                walk(child, path, layer, listed, occurrences, rows, leaf_note);
            }
        } else {
            leaf(value, path, layer, rows, leaf_note);
        }
        path.pop();
    }
}

/// An unlisted value: an object is walked to its leaves, down to the depth limit.
fn leaf(value: &Value, path: &mut Vec<String>, layer: &Layer<'_>, rows: &mut Vec<Row>, note: &str) {
    match value {
        Value::Object(child) if !child.is_empty() => {
            if path.len() >= DEPTH_LIMIT {
                rows.push(row(
                    path.clone(),
                    layer,
                    ReadingStatus::ValueNotRead,
                    None,
                    None,
                    Some(format!(
                        "deeper keys not listed (depth limit {DEPTH_LIMIT})"
                    )),
                ));
                return;
            }
            for (name, inner) in child {
                path.push(name.clone());
                if KnowledgeFile::is_credential_key(name) {
                    rows.push(row(
                        path.clone(),
                        layer,
                        ReadingStatus::ValueNotRead,
                        None,
                        None,
                        Some("credential key; not opened".to_string()),
                    ));
                } else {
                    leaf(inner, path, layer, rows, note);
                }
                path.pop();
            }
        }
        _ => rows.push(row(
            path.clone(),
            layer,
            ReadingStatus::ValueNotRead,
            None,
            None,
            Some(note.to_string()),
        )),
    }
}

fn in_scope(fact: &SettingsKeyFact, layer: &str) -> bool {
    match fact.scopes() {
        Scopes::Layers(allowed) => allowed.iter().any(|l| layer_name(*l) == layer),
        _ => true,
    }
}

/// Decides one listed key over the layers that set it.
fn decide(
    key: &Listed<'_>,
    found: &[Occurrence<'_>],
    broken: &[&Layer<'_>],
    context: &LaunchContext,
    rows: &mut Vec<Row>,
) {
    let fact = key.fact;
    let segments: Vec<String> = key.segments.iter().map(|s| (*s).to_string()).collect();
    let shown = |value: &Value| (!fact.sensitive()).then(|| value.clone());

    // Sensitive objects list their children by name only.
    if fact.sensitive() && fact.merge() == MergeKind::ObjectMerge {
        for occurrence in found {
            let mut path = segments.clone();
            if let Value::Object(object) = occurrence.value {
                for (name, inner) in object {
                    path.push(name.clone());
                    leaf(inner, &mut path, occurrence.layer, rows, SENSITIVE_CHILD);
                    path.pop();
                }
            } else {
                leaf(
                    occurrence.value,
                    &mut path,
                    occurrence.layer,
                    rows,
                    SENSITIVE_CHILD,
                );
            }
        }
        return;
    }

    let mut deciding: Vec<&Occurrence<'_>> = Vec::new();
    for occurrence in found {
        let layer = occurrence.layer;
        if !matches!(layer.role, Role::Decides) {
            let note = format!(
                "not read from {}; knowledge item {}",
                layer.name, layer.item_id
            );
            unread(&segments, occurrence, fact, note, rows);
        } else if !in_scope(fact, layer.name) {
            let note = format!(
                "not read from {} for this key; knowledge item {}",
                layer.name,
                key.item.id()
            );
            unread(&segments, occurrence, fact, note, rows);
        } else {
            deciding.push(occurrence);
        }
    }
    let Some(best) = deciding.first().map(|o| o.layer.rank) else {
        return;
    };

    if let Some(overrides) = fact.overrides()
        && context.set_variables.contains(overrides.variable())
    {
        for occurrence in &deciding {
            rows.push(row(
                segments.clone(),
                occurrence.layer,
                ReadingStatus::OverriddenByEnvironment,
                None,
                None,
                Some(format!("{} present", overrides.variable())),
            ));
        }
        return;
    }

    if fact.merge() == MergeKind::Scalar
        && let Some(blocker) = broken
            .iter()
            .find(|b| b.rank < best && in_scope(fact, b.name))
    {
        for occurrence in &deciding {
            rows.push(row(
                segments.clone(),
                occurrence.layer,
                ReadingStatus::NotDecided,
                None,
                shown(occurrence.value),
                Some(format!("{} file could not be read", blocker.name)),
            ));
        }
        return;
    }

    match fact.merge() {
        MergeKind::Scalar => {
            for (index, occurrence) in deciding.iter().enumerate() {
                let (status, value, note) = if index == 0 {
                    (
                        ReadingStatus::InEffect,
                        shown(occurrence.value),
                        fact.sensitive()
                            .then(|| "value not read (sensitive)".to_string()),
                    )
                } else {
                    let winner = deciding[0];
                    (
                        ReadingStatus::Shadowed,
                        shown(occurrence.value),
                        Some(format!(
                            "shadowed by {} ({})",
                            winner.layer.name,
                            winner.layer.path.display()
                        )),
                    )
                };
                rows.push(row(
                    segments.clone(),
                    occurrence.layer,
                    status,
                    None,
                    value,
                    note,
                ));
            }
        }
        MergeKind::ListMerge => {
            let mut seen: Vec<(&Value, &str)> = Vec::new();
            for occurrence in &deciding {
                let Value::Array(entries) = occurrence.value else {
                    rows.push(row(
                        segments.clone(),
                        occurrence.layer,
                        ReadingStatus::NotDecided,
                        None,
                        None,
                        Some("value is not an array".to_string()),
                    ));
                    continue;
                };
                for (index, entry) in entries.iter().enumerate() {
                    let position = u32::try_from(index).unwrap_or(u32::MAX);
                    if fact.sensitive() {
                        rows.push(row(
                            segments.clone(),
                            occurrence.layer,
                            ReadingStatus::InEffect,
                            Some(position),
                            None,
                            Some("value not read (sensitive)".to_string()),
                        ));
                        continue;
                    }
                    let also = seen
                        .iter()
                        .find(|(value, layer)| *value == entry && *layer != occurrence.layer.name)
                        .map(|(_, layer)| format!("also set in {layer}"));
                    if also.is_none() {
                        seen.push((entry, occurrence.layer.name));
                    }
                    rows.push(row(
                        segments.clone(),
                        occurrence.layer,
                        ReadingStatus::InEffect,
                        Some(position),
                        Some(entry.clone()),
                        also,
                    ));
                }
            }
        }
        MergeKind::OwnRule => {
            for occurrence in &deciding {
                rows.push(row(
                    segments.clone(),
                    occurrence.layer,
                    ReadingStatus::NotDecided,
                    None,
                    shown(occurrence.value),
                    Some("own merge rule".to_string()),
                ));
            }
        }
        _ => {
            for occurrence in &deciding {
                rows.push(row(
                    segments.clone(),
                    occurrence.layer,
                    ReadingStatus::NotDecided,
                    None,
                    None,
                    Some("object merge not decided".to_string()),
                ));
            }
        }
    }
}

/// A reading of a key in a layer that takes no part in deciding it. A list key keeps one
/// row per entry; a value shows only for a readable key that is not sensitive.
fn unread(
    segments: &[String],
    occurrence: &Occurrence<'_>,
    fact: &SettingsKeyFact,
    note: String,
    rows: &mut Vec<Row>,
) {
    let shown = |value: &Value| (!fact.sensitive()).then(|| value.clone());
    match occurrence.value {
        Value::Array(entries) if fact.merge() == MergeKind::ListMerge => {
            for (index, entry) in entries.iter().enumerate() {
                rows.push(row(
                    segments.to_vec(),
                    occurrence.layer,
                    ReadingStatus::Unread,
                    Some(u32::try_from(index).unwrap_or(u32::MAX)),
                    shown(entry),
                    Some(note.clone()),
                ));
            }
        }
        value => rows.push(row(
            segments.to_vec(),
            occurrence.layer,
            ReadingStatus::Unread,
            None,
            shown(value),
            Some(note),
        )),
    }
}
