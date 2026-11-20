//! The Claude Code adapter (Story 1.5, FR-43): finds the settings sources of Claude
//! Code on a host and lists them through the source-listing port.
//!
//! Locations, layers, read rules and the version link come from harness knowledge,
//! not from this crate (AD-6). The harness is never run.

#![forbid(unsafe_code)]

mod paths;
mod seat;
pub mod version;

pub use version::{VersionReading, read_version};

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use std::{fs, io};

use usher_core::{
    AbsentLayer, CheckedPath, KeyNames, LaunchContext, ListedSource, NotCheckedLayer, SourceLister,
    SourceListing, SourceStatus, SystemRoot, UsherError,
};
use usher_knowledge::{Item, KnowledgeFile, Platform, ReadRule, SourceFact, SourceLayer, Version};

use crate::paths::resolve;

/// The Claude Code adapter: lists the settings sources knowledge names for the
/// harness, under the host's system root.
#[derive(Debug, Clone)]
pub struct ClaudeCodeAdapter {
    knowledge: KnowledgeFile,
    system_root: SystemRoot,
    platform: Platform,
}

impl ClaudeCodeAdapter {
    /// An adapter over `knowledge`. Production passes [`SystemRoot::host`] and the
    /// build's own platform; only tests pass others.
    pub fn new(knowledge: KnowledgeFile, system_root: SystemRoot, platform: Platform) -> Self {
        ClaudeCodeAdapter {
            knowledge,
            system_root,
            platform,
        }
    }
}

const HARNESS: &str = "claude-code";
const CONTROL_KEYS_ITEM: &str = "claude-code.managed-control-keys";
const CONFIG_DIR_TOKEN: &str = "${CONFIG_DIR}";

/// One source item the listing considers, with its place on this host.
struct Candidate<'a> {
    item: &'a Item,
    fact: &'a SourceFact,
    path: PathBuf,
    platform_ok: bool,
}

/// What reading a source's top-level key names gave.
enum Parsed {
    Names(Vec<String>),
    Failed(String),
}

impl Parsed {
    fn keys(&self, classification: &str) -> KeyNames {
        match self {
            Parsed::Names(names) => KeyNames::Names {
                names: names.clone(),
                classification: classification.to_string(),
            },
            Parsed::Failed(reason) => KeyNames::NotRead {
                reason: reason.clone(),
            },
        }
    }
}

impl ClaudeCodeAdapter {
    /// The name of the variable that moves the config directory, for a reason text.
    fn config_dir_variable(&self) -> String {
        self.knowledge
            .items()
            .iter()
            .filter(|item| item.harness() == HARNESS)
            .filter_map(Item::variable)
            .find(|variable| variable.binds() == Some(CONFIG_DIR_TOKEN))
            .map_or_else(
                || "the config directory variable".to_string(),
                |variable| variable.name().to_string(),
            )
    }

    /// The managed keys that carry no policy, for the version.
    fn control_keys(&self, version: Option<&Version>) -> HashSet<String> {
        self.knowledge
            .items()
            .iter()
            .filter(|item| item.id() == CONTROL_KEYS_ITEM)
            .filter(|item| version.is_none_or(|v| item.versions().contains(v)))
            .flat_map(|item| item.keys().iter().cloned())
            .collect()
    }
}

impl SourceLister for ClaudeCodeAdapter {
    fn list_sources(&self, context: &LaunchContext) -> Result<SourceListing, UsherError> {
        let installed = context
            .harness_version
            .as_deref()
            .and_then(|text| Version::parse(text).ok());
        let covered: Vec<&Item> = match &installed {
            Some(version) => self
                .knowledge
                .items_for(HARNESS, version)
                .filter(|item| item.source().is_some())
                .collect(),
            None => Vec::new(),
        };
        // Without a version the knowledge cannot say which items apply, so every source
        // item of the harness is looked at and each one reads unknown.
        let known = !covered.is_empty();
        let items: Vec<&Item> = if known {
            covered
        } else {
            self.knowledge
                .items()
                .iter()
                .filter(|item| item.harness() == HARNESS && item.source().is_some())
                .collect()
        };
        let unknown_reason = match (&installed, known) {
            (Some(version), false) => format!("knowledge does not cover claude-code {version}"),
            _ => "the harness version is not known".to_string(),
        };

        let variable = self.config_dir_variable();
        let mut absent: Vec<AbsentLayer> = Vec::new();
        let mut not_checked: Vec<NotCheckedLayer> = Vec::new();
        let mut candidates: Vec<Candidate> = Vec::new();
        for item in items {
            let Some(fact) = item.source() else { continue };
            let platform_ok = fact
                .platforms()
                .is_none_or(|list| list.contains(&self.platform));
            if fact.path().contains("${REPO}") && context.repo.is_none() {
                continue;
            }
            match resolve(fact.path(), context, &self.system_root, &variable) {
                // One file is one source: a second item of the same layer that places
                // at the same path adds no row.
                Ok(path)
                    if candidates
                        .iter()
                        .any(|c| c.fact.layer() == fact.layer() && c.path == path) => {}
                Ok(path) => candidates.push(Candidate {
                    item,
                    fact,
                    path,
                    platform_ok,
                }),
                Err(reason) => {
                    // A layer that is always present, and the global state file that the
                    // seat's account item also reads, are named when they cannot be placed.
                    let named = fact.always_present() || fact.layer() == SourceLayer::GlobalState;
                    if named && platform_ok {
                        not_checked.push(NotCheckedLayer {
                            layer: layer_name(fact.layer()).to_string(),
                            reason,
                        });
                    }
                }
            }
        }

        let mut present: Vec<(&Candidate, fs::Metadata)> = Vec::new();
        // First-wins candidates that could not be checked: each might deliver, so each
        // counts as an unreadable file at its rank.
        let mut unchecked: Vec<(u32, PathBuf)> = Vec::new();
        for candidate in &candidates {
            match fs::metadata(&candidate.path) {
                Ok(meta) => present.push((candidate, meta)),
                // Any other error is not evidence of absence: the path may exist behind a
                // directory this process cannot look into. It is named as not checked,
                // with the error kind and never any content.
                Err(error) if error.kind() != io::ErrorKind::NotFound => {
                    if candidate.platform_ok {
                        if candidate.fact.read() == ReadRule::FirstWins {
                            unchecked.push((candidate.fact.rank(), candidate.path.clone()));
                        }
                        not_checked.push(NotCheckedLayer {
                            layer: layer_name(candidate.fact.layer()).to_string(),
                            reason: format!(
                                "{} could not be checked: {:?}",
                                candidate.path.display(),
                                error.kind()
                            ),
                        });
                    }
                }
                Err(_) => {
                    // Project and local settings belong to a repository, so their absence
                    // is named only when the directory is in one.
                    let needs_repo = matches!(
                        candidate.fact.layer(),
                        SourceLayer::Project | SourceLayer::Local
                    );
                    if candidate.fact.always_present()
                        && candidate.platform_ok
                        && !(needs_repo && context.repo.is_none())
                    {
                        let layer = layer_name(candidate.fact.layer());
                        match absent.iter_mut().find(|a| a.layer == layer) {
                            Some(entry) => entry.checked.push(CheckedPath {
                                path: candidate.path.clone(),
                                item_id: candidate.item.id().to_string(),
                            }),
                            None => absent.push(AbsentLayer {
                                layer: layer.to_string(),
                                checked: vec![CheckedPath {
                                    path: candidate.path.clone(),
                                    item_id: candidate.item.id().to_string(),
                                }],
                            }),
                        }
                    }
                }
            }
        }

        let version_text = installed.as_ref().map(Version::to_string);
        let classification = format!(
            "not classified: {} lists no keys for claude-code {}",
            self.knowledge.version(),
            version_text.as_deref().unwrap_or("(version unknown)")
        );
        let control = self.control_keys(installed.as_ref().filter(|_| known));

        // Layers by precedence, then rank within a layer: a smaller number ranks higher.
        present.sort_by_key(|(candidate, _)| {
            (layer_order(candidate.fact.layer()), candidate.fact.rank())
        });

        let parsed: Vec<Option<Parsed>> = present
            .iter()
            .map(|(candidate, meta)| {
                let reads =
                    known && candidate.platform_ok && candidate.fact.read() != ReadRule::NotRead;
                reads.then(|| parse_keys(&candidate.path, meta))
            })
            .collect();

        let statuses = if known {
            self.statuses(&present, &parsed, &control, &unchecked)
        } else {
            vec![
                SourceStatus::Unknown {
                    reason: unknown_reason,
                };
                present.len()
            ]
        };

        let sources = present
            .iter()
            .zip(&parsed)
            .zip(statuses)
            .map(|(((candidate, meta), parsed), status)| {
                let keys = match parsed {
                    Some(parsed) => parsed.keys(&classification),
                    None => KeyNames::NotRead {
                        reason: "no key names were read".to_string(),
                    },
                };
                ListedSource {
                    path: candidate.path.clone(),
                    harness: HARNESS.to_string(),
                    layer: layer_name(candidate.fact.layer()).to_string(),
                    item_id: candidate.item.id().to_string(),
                    status,
                    size: Some(meta.len()),
                    modified: meta
                        .modified()
                        .ok()
                        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                        .map(|elapsed| elapsed.as_secs()),
                    keys,
                }
            })
            .collect();

        Ok(SourceListing {
            harness: HARNESS.to_string(),
            harness_version: version_text,
            knowledge_version: self.knowledge.version().to_string(),
            sources,
            absent_layers: absent,
            not_checked,
            seat: seat::read_seat(
                &self.knowledge,
                context,
                &self.system_root,
                self.platform,
                installed.as_ref(),
                &variable,
            ),
        })
    }
}

impl ClaudeCodeAdapter {
    /// The status of each present source, in the order given.
    fn statuses(
        &self,
        present: &[(&Candidate, fs::Metadata)],
        parsed: &[Option<Parsed>],
        control: &HashSet<String>,
        unchecked: &[(u32, PathBuf)],
    ) -> Vec<SourceStatus> {
        let mut statuses: Vec<Option<SourceStatus>> = vec![None; present.len()];
        // Sources that do not take part in first-wins are decided on their own.
        let mut first_wins: Vec<usize> = Vec::new();
        for (index, (candidate, _)) in present.iter().enumerate() {
            let fact = candidate.fact;
            statuses[index] = if !candidate.platform_ok {
                Some(SourceStatus::Unread {
                    reason: format!(
                        "knowledge item {}; read on {} only",
                        candidate.item.id(),
                        platform_names(fact.platforms().unwrap_or(&[]))
                    ),
                })
            } else {
                match fact.read() {
                    ReadRule::NotRead => Some(SourceStatus::Unread {
                        reason: format!("knowledge item {}", candidate.item.id()),
                    }),
                    ReadRule::Read => Some(match &parsed[index] {
                        Some(Parsed::Failed(reason)) => SourceStatus::Unknown {
                            reason: reason.clone(),
                        },
                        _ => SourceStatus::InEffect,
                    }),
                    ReadRule::FirstWins => {
                        first_wins.push(index);
                        None
                    }
                }
            };
        }

        // `present` is sorted by layer then rank, so `first_wins` is in rank order. The
        // decision is made rank by rank and never guessed past a point it cannot be made.
        let mut decision = Decision::Pending;
        let mut ranks: Vec<u32> = first_wins
            .iter()
            .map(|&member| present[member].0.fact.rank())
            .chain(unchecked.iter().map(|(rank, _)| *rank))
            .collect();
        ranks.sort_unstable();
        ranks.dedup();
        for rank in ranks {
            let group: Vec<usize> = first_wins
                .iter()
                .copied()
                .filter(|member| present[*member].0.fact.rank() == rank)
                .collect();
            let hidden: Vec<&Path> = unchecked
                .iter()
                .filter(|(at, _)| *at == rank)
                .map(|(_, path)| path.as_path())
                .collect();
            match &decision {
                Decision::Winner(by) => {
                    for other in group {
                        statuses[other] = Some(SourceStatus::Shadowed { by: by.clone() });
                    }
                }
                Decision::Undecided(cause) => {
                    for other in group {
                        statuses[other] = Some(SourceStatus::Unknown {
                            reason: format!("first-wins not decided: {cause}"),
                        });
                    }
                }
                Decision::Pending => {
                    decision = decide_rank(
                        rank,
                        &group,
                        &hidden,
                        present,
                        parsed,
                        control,
                        &mut statuses,
                    );
                }
            }
        }
        statuses
            .into_iter()
            .map(|status| {
                status.unwrap_or(SourceStatus::Unknown {
                    reason: "no status could be placed".to_string(),
                })
            })
            .collect()
    }
}

/// Where the first-wins decision stands as the ranks are walked from the top.
enum Decision {
    /// No source ranked so far delivers a policy key.
    Pending,
    /// A source delivers, and everything below it is shadowed by it.
    Winner(PathBuf),
    /// The decision cannot be made, and why; everything below reads that.
    Undecided(String),
}

/// Decides one rank of first-wins sources while nothing above it has decided. Only a
/// source that delivers can win or tie; one that cannot be read counts as one that might.
fn decide_rank(
    rank: u32,
    group: &[usize],
    hidden: &[&Path],
    present: &[(&Candidate, fs::Metadata)],
    parsed: &[Option<Parsed>],
    control: &HashSet<String>,
    statuses: &mut [Option<SourceStatus>],
) -> Decision {
    let mut delivering: Vec<usize> = Vec::new();
    let mut unreadable: Vec<usize> = Vec::new();
    for &member in group {
        match &parsed[member] {
            Some(Parsed::Names(names)) if names.is_empty() => {
                statuses[member] = Some(SourceStatus::Unread {
                    reason: "delivers no policy key: empty".to_string(),
                });
            }
            Some(Parsed::Names(names)) if names.iter().all(|name| control.contains(name)) => {
                statuses[member] = Some(SourceStatus::Unread {
                    reason: "delivers no policy key: only control keys".to_string(),
                });
            }
            Some(Parsed::Names(_)) => delivering.push(member),
            Some(Parsed::Failed(_)) => unreadable.push(member),
            None => {
                statuses[member] = Some(SourceStatus::Unknown {
                    reason: "the file was not read".to_string(),
                });
            }
        }
    }
    let path = |member: usize| present[member].0.path.clone();
    for &member in &unreadable {
        if let Some(Parsed::Failed(note)) = &parsed[member] {
            statuses[member] = Some(SourceStatus::Unknown {
                reason: note.clone(),
            });
        }
    }
    let cause = match (unreadable.first(), hidden.first()) {
        (Some(&bad), _) => Some(format!("{} could not be read", path(bad).display())),
        (None, Some(path)) => Some(format!("{} could not be checked", path.display())),
        (None, None) => None,
    };
    match (delivering.as_slice(), cause) {
        ([], None) => Decision::Pending,
        ([only], None) => {
            statuses[*only] = Some(SourceStatus::InEffect);
            Decision::Winner(path(*only))
        }
        ([], Some(cause)) => Decision::Undecided(cause),
        (_, Some(cause)) => {
            for &member in &delivering {
                statuses[member] = Some(SourceStatus::Unknown {
                    reason: format!("first-wins not decided: {cause}"),
                });
            }
            Decision::Undecided(cause)
        }
        (_, None) => {
            let reason = format!("two managed sources share rank {rank}");
            for &member in &delivering {
                statuses[member] = Some(SourceStatus::Unknown {
                    reason: reason.clone(),
                });
            }
            Decision::Undecided(reason)
        }
    }
}

fn layer_name(layer: SourceLayer) -> &'static str {
    match layer {
        SourceLayer::Managed => "managed",
        SourceLayer::User => "user",
        SourceLayer::Project => "project",
        SourceLayer::Local => "local",
        SourceLayer::GlobalState => "global-state",
        SourceLayer::ServerManagedCache => "server-managed-cache",
    }
}

fn layer_order(layer: SourceLayer) -> u8 {
    match layer {
        SourceLayer::Managed => 0,
        SourceLayer::User => 1,
        SourceLayer::Project => 2,
        SourceLayer::Local => 3,
        SourceLayer::GlobalState => 4,
        SourceLayer::ServerManagedCache => 5,
    }
}

fn platform_names(platforms: &[Platform]) -> String {
    platforms
        .iter()
        .map(|platform| match platform {
            Platform::Macos => "macos",
            Platform::Linux => "linux",
            Platform::Windows => "windows",
            _ => "another platform",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Reads the top-level key names of a JSON settings file, sorted by name. The values are
/// parsed in memory and dropped, and never reach a listing, a store row or a message.
fn parse_keys(path: &Path, meta: &fs::Metadata) -> Parsed {
    if !meta.is_file() {
        return Parsed::Failed("not a regular file".to_string());
    }
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(why) => return Parsed::Failed(format!("cannot be read: {why}")),
    };
    match serde_json::from_reader::<_, serde_json::Map<String, serde_json::Value>>(
        io::BufReader::new(file),
    ) {
        Ok(object) => Parsed::Names(object.keys().cloned().collect()),
        Err(why) => Parsed::Failed(json_note(&why)),
    }
}

/// The parse failure as position and category, never any file content.
fn json_note(why: &serde_json::Error) -> String {
    let category = match why.classify() {
        serde_json::error::Category::Io => "read error",
        serde_json::error::Category::Syntax => "syntax",
        serde_json::error::Category::Data => "not a JSON object",
        serde_json::error::Category::Eof => "unexpected end of file",
    };
    format!(
        "not valid JSON at line {}, column {}: {category}",
        why.line(),
        why.column()
    )
}
