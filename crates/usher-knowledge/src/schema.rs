//! The harness-knowledge schema (AD-12) and its structural check.
//!
//! One TOML file, two layers. The slow layer holds layer order, file paths and
//! credential stores; the fast layer holds keys, defaults, hazards, the
//! never-call list and disputed claims. Every item names a harness, a version
//! range, a reference and the date it was last verified (FR-30). A file that
//! breaks any of that does not load.

use std::collections::HashSet;
use std::fmt;

use toml_edit::{DocumentMut, Item as TomlItem, TableLike};
use usher_core::UsherError;

use crate::support::{Capability, SupportReport};
use crate::version::{Version, VersionRange};

/// The knowledge file built into this release.
const EMBEDDED: &str = include_str!("../knowledge/harness-knowledge.toml");

/// The only schema version this build reads.
const SUPPORTED_SCHEMA: u32 = 4;

const STEP: &str = "load harness knowledge";
const NEXT: &str =
    "fix the entry in the knowledge file, then run `cargo test -p usher-knowledge` to check it";

fn fail(check: impl Into<String>, status: impl Into<String>) -> UsherError {
    UsherError::new(STEP, check, status, NEXT)
}

/// Which layer an item lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// Slow facts: layers and precedence, file paths, credential stores.
    Slow,
    /// Fast facts: keys, defaults, hazards, the never-call list, disputes.
    Fast,
}

impl Layer {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "slow" => Some(Layer::Slow),
            "fast" => Some(Layer::Fast),
            _ => None,
        }
    }
}

/// What kind of fact an item states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Layer order and precedence (slow).
    Layers,
    /// File and directory locations (slow).
    Paths,
    /// Settings sources, with the layer each belongs to and whether the version reads it (slow).
    Sources,
    /// Where credentials live (slow).
    CredentialStores,
    /// Settings keys and what they do (fast).
    Keys,
    /// Default values (fast).
    Defaults,
    /// Hazards for an observer (fast).
    Hazards,
    /// Commands usher must never call (fast).
    NeverCall,
    /// Claims whose sources disagree (fast).
    Disputed,
    /// The parts of the seat around a harness (slow).
    Seat,
    /// The settings keys a harness reads, with how each merges across layers (fast).
    SettingsKeys,
}

impl Class {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "layers" => Some(Class::Layers),
            "paths" => Some(Class::Paths),
            "sources" => Some(Class::Sources),
            "credential-stores" => Some(Class::CredentialStores),
            "keys" => Some(Class::Keys),
            "defaults" => Some(Class::Defaults),
            "hazards" => Some(Class::Hazards),
            "never-call" => Some(Class::NeverCall),
            "disputed" => Some(Class::Disputed),
            "seat" => Some(Class::Seat),
            "settings-keys" => Some(Class::SettingsKeys),
            _ => None,
        }
    }

    /// The layer this class belongs to.
    pub fn layer(self) -> Layer {
        match self {
            Class::Layers
            | Class::Paths
            | Class::Sources
            | Class::CredentialStores
            | Class::Seat => Layer::Slow,
            Class::Keys
            | Class::Defaults
            | Class::Hazards
            | Class::NeverCall
            | Class::Disputed
            | Class::SettingsKeys => Layer::Fast,
        }
    }
}

/// Where a fact comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    /// The vendor's documentation.
    VendorDocs,
    /// A release changelog.
    Changelog,
    /// The harness source code.
    Source,
    /// A probe usher ran.
    Probe,
}

impl ReferenceKind {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "vendor-docs" => Some(ReferenceKind::VendorDocs),
            "changelog" => Some(ReferenceKind::Changelog),
            "source" => Some(ReferenceKind::Source),
            "probe" => Some(ReferenceKind::Probe),
            _ => None,
        }
    }
}

/// The source an item cites: what kind it is and where to find it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    kind: ReferenceKind,
    locator: String,
}

impl Reference {
    /// The kind of source.
    pub fn kind(&self) -> ReferenceKind {
        self.kind
    }

    /// A URL, file path or probe name that leads to the source.
    pub fn locator(&self) -> &str {
        &self.locator
    }
}

/// A calendar date, `YYYY-MM-DD`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// The harness layer a settings source belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLayer {
    /// Managed by an organization or the vendor.
    Managed,
    /// The person's own settings.
    User,
    /// Shared settings in a repository.
    Project,
    /// Personal settings in a repository.
    Local,
    /// The harness's own state file.
    GlobalState,
    /// The cache of server-managed settings.
    ServerManagedCache,
}

impl SourceLayer {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "managed" => Some(SourceLayer::Managed),
            "user" => Some(SourceLayer::User),
            "project" => Some(SourceLayer::Project),
            "local" => Some(SourceLayer::Local),
            "global-state" => Some(SourceLayer::GlobalState),
            "server-managed-cache" => Some(SourceLayer::ServerManagedCache),
            _ => None,
        }
    }
}

/// Whether a harness version reads a source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadRule {
    /// The version reads it.
    Read,
    /// The version does not read it.
    NotRead,
    /// Read only when no higher-ranked source of the vendor's managed tier delivers a
    /// policy: under the default, "exists" means "delivers at least one policy key", so a
    /// file holding only control keys is skipped.
    FirstWins,
}

impl ReadRule {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "read" => Some(ReadRule::Read),
            "not-read" => Some(ReadRule::NotRead),
            "first-wins" => Some(ReadRule::FirstWins),
            _ => None,
        }
    }
}

/// The tokens a source path may contain; each resolves per launch environment.
const PATH_TOKENS: [&str; 6] = [
    "${HOME}",
    "${CONFIG_DIR}",
    "${GLOBAL_STATE_DIR}",
    "${REPO}",
    "${LAUNCH_DIR}",
    "${USER}",
];

/// The tokens a launch variable may bind.
const BINDABLE_TOKENS: [&str; 2] = ["${CONFIG_DIR}", "${GLOBAL_STATE_DIR}"];

/// The only tokens a binder's default may use.
const DEFAULT_TOKENS: [&str; 2] = ["${HOME}", "${USER}"];

/// Keys knowledge marks as credentials: no key path may start at or pass through one,
/// and one may be named only in the `keys` of a `keys-present` read, at top level.
const CREDENTIAL_KEYS: [&str; 1] = ["oauthAccount"];

/// What harness knowledge says about one settings source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFact {
    path: String,
    layer: SourceLayer,
    read: ReadRule,
    rank: u32,
    always_present: bool,
    platforms: Option<Vec<Platform>>,
}

/// An operating system a source is read on. WSL counts as Linux.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Platform {
    /// macOS.
    Macos,
    /// Linux, WSL included.
    Linux,
    /// Windows.
    Windows,
}

impl SourceFact {
    /// The platforms the source is read on; `None` means every platform.
    pub fn platforms(&self) -> Option<&[Platform]> {
        self.platforms.as_deref()
    }

    /// Whether the layer always exists on a platform, so its absence is named.
    pub fn always_present(&self) -> bool {
        self.always_present
    }

    /// The path pattern, in the five path tokens.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The layer the source belongs to.
    pub fn layer(&self) -> SourceLayer {
        self.layer
    }

    /// Whether the version reads it.
    pub fn read(&self) -> ReadRule {
        self.read
    }

    /// Precedence within the vendor's managed tier, counting every managed delivery the
    /// vendor lists, listed here or not; 1 is highest. Sources of other layers each use
    /// their own rank.
    pub fn rank(&self) -> u32 {
        self.rank
    }
}

/// Which part of the seat around a harness an item describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SeatPart {
    /// MCP server registrations.
    Mcp,
    /// Installed packs.
    Packs,
    /// Client config for fleet tools.
    FleetClient,
    /// The backend and auth route.
    AuthRoute,
    /// The toolchain declaration.
    Toolchain,
}

/// How much of a seat part may be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SeatRead {
    /// A file or directory exists.
    Presence,
    /// The entry names of one directory, one level deep.
    Listing,
    /// Key names at the listed key paths of a JSON file.
    Names,
    /// The presence of the listed environment variables.
    Env,
    /// Whether a JSON file's top-level object has each listed key, nothing under it read.
    KeysPresent,
}

/// What harness knowledge says about one part of the seat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatFact {
    part: SeatPart,
    read: SeatRead,
    path: Option<String>,
    key_paths: Vec<String>,
    keys: Vec<String>,
    variables: Vec<String>,
    platforms: Option<Vec<Platform>>,
}

impl SeatPart {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "mcp" => Some(SeatPart::Mcp),
            "packs" => Some(SeatPart::Packs),
            "fleet-client" => Some(SeatPart::FleetClient),
            "auth-route" => Some(SeatPart::AuthRoute),
            "toolchain" => Some(SeatPart::Toolchain),
            _ => None,
        }
    }
}

impl SeatRead {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "presence" => Some(SeatRead::Presence),
            "listing" => Some(SeatRead::Listing),
            "names" => Some(SeatRead::Names),
            "env" => Some(SeatRead::Env),
            "keys-present" => Some(SeatRead::KeysPresent),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            SeatRead::Presence => "presence",
            SeatRead::Listing => "listing",
            SeatRead::Names => "names",
            SeatRead::Env => "env",
            SeatRead::KeysPresent => "keys-present",
        }
    }
}

impl SeatFact {
    /// The part of the seat.
    pub fn part(&self) -> SeatPart {
        self.part
    }

    /// How much may be read.
    pub fn read(&self) -> SeatRead {
        self.read
    }

    /// The path pattern; absent for an environment read.
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// The key paths whose names are read, `*` standing for one level.
    pub fn key_paths(&self) -> &[String] {
        &self.key_paths
    }

    /// The top-level keys whose membership is read.
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    /// The environment variables whose presence is read.
    pub fn variables(&self) -> &[String] {
        &self.variables
    }

    /// The platforms the item applies on; `None` means every platform.
    pub fn platforms(&self) -> Option<&[Platform]> {
        self.platforms.as_deref()
    }
}

/// How a settings key combines across layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MergeKind {
    /// The highest-ranked layer that sets the key wins.
    Scalar,
    /// Every layer's entries are kept, each tied to its layer.
    ListMerge,
    /// Objects combine key by key.
    ObjectMerge,
    /// The harness gives the key its own rule; usher does not decide it.
    OwnRule,
}

/// An environment variable that overrides a settings key when it is set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvOverride {
    variable: String,
    key: String,
}

impl EnvOverride {
    /// The variable whose presence overrides the key.
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// The settings key it overrides.
    pub fn key(&self) -> &str {
        &self.key
    }
}

/// What harness knowledge says about one settings key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsKeyFact {
    key: String,
    merge: MergeKind,
    sensitive: bool,
    overrides: Option<EnvOverride>,
}

impl SettingsKeyFact {
    /// The key path; `*` stands for one whole level.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// How the key combines across layers.
    pub fn merge(&self) -> MergeKind {
        self.merge
    }

    /// Whether the value is never read or stored, even though the key is listed.
    pub fn sensitive(&self) -> bool {
        self.sensitive
    }

    /// The environment variable that overrides the key, if knowledge names one.
    pub fn overrides(&self) -> Option<&EnvOverride> {
        self.overrides.as_ref()
    }
}

/// One fact about a harness, with its source and the date it was last verified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    id: String,
    harness: String,
    layer: Layer,
    class: Class,
    versions: VersionRange,
    summary: String,
    reference: Reference,
    last_verified: Date,
    source: Option<SourceFact>,
    keys: Vec<String>,
    version_from: Option<VersionFrom>,
    variable: Option<LaunchVariable>,
    seat: Option<SeatFact>,
    settings_key: Option<SettingsKeyFact>,
    stack: Vec<String>,
}

/// How a harness version is read from disk without running the harness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionFrom {
    launcher: String,
    rule: VersionRule,
    layout: String,
}

/// The rule that turns the launcher into a version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum VersionRule {
    /// Follow the launcher symlink and take the last component of its target.
    SymlinkTargetLastComponent,
}

/// A path token bound to a launch variable, with the default it takes when the variable
/// is unset or empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    variable: String,
    default: String,
}

impl Binding {
    /// The variable's name.
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// The path the token takes while the variable is unset or empty.
    pub fn default(&self) -> &str {
        &self.default
    }
}

/// A launch variable a harness reads, and the path token it moves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchVariable {
    name: String,
    binds: Option<String>,
    default: Option<String>,
}

impl LaunchVariable {
    /// The variable's name, as the harness reads it from its environment.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The path token the variable moves, when it moves one.
    pub fn binds(&self) -> Option<&str> {
        self.binds.as_deref()
    }

    /// The path the token takes while the variable is unset or empty.
    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }
}

impl VersionRule {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "symlink-target-last-component" => Some(VersionRule::SymlinkTargetLastComponent),
            _ => None,
        }
    }
}

impl VersionFrom {
    /// The launcher path, in path tokens.
    pub fn launcher(&self) -> &str {
        &self.launcher
    }

    /// The rule that reads the version.
    pub fn rule(&self) -> VersionRule {
        self.rule
    }

    /// The shape the link target must match for the last component to count.
    pub fn layout(&self) -> &str {
        &self.layout
    }
}

impl Item {
    /// The launch variable a `paths` item names, when it names one.
    pub fn variable(&self) -> Option<&LaunchVariable> {
        self.variable.as_ref()
    }

    /// How to read the harness version from disk, for a `paths` item that says so.
    pub fn version_from(&self) -> Option<&VersionFrom> {
        self.version_from.as_ref()
    }

    /// The setting names a `keys` item lists, empty for every other class.
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    /// The settings-key fact, for an item of class `settings-keys`.
    pub fn settings_key(&self) -> Option<&SettingsKeyFact> {
        self.settings_key.as_ref()
    }

    /// The cross-layer order, highest first, for an item of class `layers`.
    pub fn stack(&self) -> &[String] {
        &self.stack
    }

    /// The seat fact, for an item of class `seat`.
    pub fn seat(&self) -> Option<&SeatFact> {
        self.seat.as_ref()
    }

    /// The settings-source fact, for an item of class `sources`.
    pub fn source(&self) -> Option<&SourceFact> {
        self.source.as_ref()
    }

    /// The item's unique id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The harness the fact is about.
    pub fn harness(&self) -> &str {
        &self.harness
    }

    /// The layer the item lives in.
    pub fn layer(&self) -> Layer {
        self.layer
    }

    /// The kind of fact.
    pub fn class(&self) -> Class {
        self.class
    }

    /// The harness versions the fact applies to.
    pub fn versions(&self) -> &VersionRange {
        &self.versions
    }

    /// The fact, in one sentence.
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// The source the fact cites.
    pub fn reference(&self) -> &Reference {
        &self.reference
    }

    /// The date the fact was last verified against its source.
    pub fn last_verified(&self) -> Date {
        self.last_verified
    }
}

/// The capabilities usher offers for a harness within a version range.
///
/// This is usher's own statement, not a fact about the harness, so it carries
/// no reference. When two entries for one harness both match a version, the
/// first in file order wins.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SupportEntry {
    harness: String,
    versions: VersionRange,
    capabilities: Vec<Capability>,
}

/// The version of a knowledge file, for any report that relies on it (FR-31).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnowledgeVersion {
    schema_version: u32,
    sequence: u64,
}

impl KnowledgeVersion {
    /// The schema the file is written in.
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// The file's sequence number; a later file has a higher one.
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

impl fmt::Display for KnowledgeVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "knowledge v{} (schema {})",
            self.sequence, self.schema_version
        )
    }
}

/// A parsed knowledge file that passed the structural check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeFile {
    version: KnowledgeVersion,
    items: Vec<Item>,
    support: Vec<SupportEntry>,
}

fn text_field<'a>(table: &'a dyn TableLike, key: &str, owner: &str) -> Result<&'a str, UsherError> {
    table
        .get(key)
        .and_then(TomlItem::as_str)
        .filter(|s| !s.trim().is_empty() && *s == s.trim())
        .ok_or_else(|| {
            fail(
                format!("{owner}: {key}"),
                format!("`{key}` is missing, empty, padded with spaces, or not a string"),
            )
        })
}

/// Fails on any key outside `allowed`, so a misspelled table or field is an
/// error and not an empty or half-read file.
fn reject_unknown_keys(
    table: &dyn TableLike,
    allowed: &[&str],
    owner: &str,
) -> Result<(), UsherError> {
    match table
        .iter()
        .map(|(key, _)| key)
        .find(|key| !allowed.contains(key))
    {
        None => Ok(()),
        Some(key) => Err(fail(
            format!("{owner}: {key}"),
            format!(
                "`{key}` is not a known key; known keys are {}",
                allowed.join(", ")
            ),
        )),
    }
}

fn number_field(table: &dyn TableLike, key: &str, owner: &str) -> Result<u64, UsherError> {
    table
        .get(key)
        .and_then(TomlItem::as_integer)
        .and_then(|n| u64::try_from(n).ok())
        .filter(|n| *n >= 1)
        .ok_or_else(|| {
            fail(
                format!("{owner}: {key}"),
                format!("`{key}` is missing or not a whole number of at least 1"),
            )
        })
}

fn tables<'a>(doc: &'a DocumentMut, key: &str) -> Result<Vec<&'a dyn TableLike>, UsherError> {
    match doc.get(key) {
        None => Ok(Vec::new()),
        Some(item) => item
            .as_array_of_tables()
            .map(|array| array.iter().map(|t| t as &dyn TableLike).collect())
            .ok_or_else(|| {
                fail(
                    format!("[[{key}]]"),
                    format!("`{key}` must be written as [[{key}]] tables"),
                )
            }),
    }
}

fn parse_reference(table: &dyn TableLike, id: &str) -> Result<Reference, UsherError> {
    let owner = format!("item `{id}`");
    let reference = table
        .get("reference")
        .and_then(TomlItem::as_table_like)
        .ok_or_else(|| {
            fail(
                format!("{owner}: reference"),
                "the item has no reference (kind and locator)",
            )
        })?;
    reject_unknown_keys(
        reference,
        &["kind", "locator"],
        &format!("{owner} reference"),
    )?;
    let kind_name = text_field(reference, "kind", &format!("{owner} reference"))?;
    let kind = ReferenceKind::parse(kind_name).ok_or_else(|| {
        fail(
            format!("{owner}: reference kind"),
            format!("`{kind_name}` is not one of vendor-docs, changelog, source, probe"),
        )
    })?;
    let locator = text_field(reference, "locator", &format!("{owner} reference"))?;
    Ok(Reference {
        kind,
        locator: locator.to_string(),
    })
}

fn parse_date(table: &dyn TableLike, id: &str) -> Result<Date, UsherError> {
    let owner = format!("item `{id}`: last_verified");
    let datetime = table
        .get("last_verified")
        .and_then(TomlItem::as_value)
        .and_then(|v| v.as_datetime())
        .ok_or_else(|| {
            fail(
                owner.clone(),
                "the item has no last_verified date (write it as 2026-11-02, without quotes)",
            )
        })?;
    match (datetime.date, datetime.time, datetime.offset) {
        (Some(date), None, None) => Ok(Date {
            year: date.year,
            month: date.month,
            day: date.day,
        }),
        _ => Err(fail(
            owner,
            "last_verified must be a plain date with no time part",
        )),
    }
}

fn parse_range(table: &dyn TableLike, owner: &str) -> Result<VersionRange, UsherError> {
    let text = text_field(table, "versions", owner)?;
    VersionRange::parse(text).map_err(|why| fail(format!("{owner}: versions"), why))
}

/// The placeholder a `version_from` layout holds once, as its last component. It is a
/// capture marker and not a path token.
const VERSION_PLACEHOLDER: &str = "${VERSION}";

/// A path starts with a token or `/`, and holds no token outside the five and `extra`.
fn check_path(path: &str, owner: &str, what: &str, extra: &[&str]) -> Result<(), UsherError> {
    let mut rest = path;
    while let Some(start) = rest.find("${") {
        let from = &rest[start..];
        let Some(end) = from.find('}') else {
            return Err(fail(
                format!("{owner}: {what}"),
                format!("`{from}` has a token that is never closed"),
            ));
        };
        let token = &from[..=end];
        if !PATH_TOKENS.contains(&token) && !extra.contains(&token) {
            return Err(fail(
                format!("{owner}: {what}"),
                format!(
                    "`{token}` is not a path token; known tokens are {}",
                    PATH_TOKENS.join(", ")
                ),
            ));
        }
        rest = &from[end + 1..];
    }
    let rooted = path.starts_with('/') || PATH_TOKENS.iter().any(|token| path.starts_with(token));
    if !rooted {
        return Err(fail(
            format!("{owner}: {what}"),
            format!("`{path}` must be absolute: start with a path token or `/`"),
        ));
    }
    Ok(())
}

/// The layout holds `${VERSION}` exactly once, as the whole last component.
fn check_layout(layout: &str, owner: &str) -> Result<(), UsherError> {
    check_path(layout, owner, "layout", &[VERSION_PLACEHOLDER])?;
    let last_is_placeholder = layout.rsplit('/').next() == Some(VERSION_PLACEHOLDER);
    if layout.matches(VERSION_PLACEHOLDER).count() != 1 || !last_is_placeholder {
        return Err(fail(
            format!("{owner}: layout"),
            format!(
                "`{layout}` must hold {VERSION_PLACEHOLDER} exactly once, as its whole last path component"
            ),
        ));
    }
    Ok(())
}

fn parse_version_from(table: &dyn TableLike, owner: &str) -> Result<VersionFrom, UsherError> {
    let from = table
        .get("version_from")
        .and_then(TomlItem::as_table_like)
        .ok_or_else(|| {
            fail(
                format!("{owner}: version_from"),
                "`version_from` must be a table (launcher, rule, layout)",
            )
        })?;
    let owner = format!("{owner} version_from");
    reject_unknown_keys(from, &["launcher", "rule", "layout"], &owner)?;
    let launcher = text_field(from, "launcher", &owner)?;
    check_path(launcher, &owner, "launcher", &[])?;
    let rule_name = text_field(from, "rule", &owner)?;
    let rule = VersionRule::parse(rule_name).ok_or_else(|| {
        fail(
            format!("{owner}: rule"),
            format!("`{rule_name}` is not a version rule; the one known is symlink-target-last-component"),
        )
    })?;
    let layout = text_field(from, "layout", &owner)?;
    check_layout(layout, &owner)?;
    Ok(VersionFrom {
        launcher: launcher.to_string(),
        rule,
        layout: layout.to_string(),
    })
}

/// The setting names of a `keys` item: a non-empty list of distinct, non-empty strings.
fn parse_keys(table: &dyn TableLike, owner: &str) -> Result<Vec<String>, UsherError> {
    let list = table
        .get("keys")
        .and_then(TomlItem::as_array)
        .ok_or_else(|| {
            fail(
                format!("{owner}: keys"),
                "an item of class `keys` needs a `keys` list of setting names",
            )
        })?;
    if list.is_empty() {
        return Err(fail(
            format!("{owner}: keys"),
            "`keys` is empty; list at least one setting name",
        ));
    }
    let mut names: Vec<String> = Vec::new();
    for entry in list {
        let name = entry
            .as_str()
            .ok_or_else(|| fail(format!("{owner}: keys"), "every key name must be a string"))?;
        if name.trim().is_empty() || name != name.trim() {
            return Err(fail(
                format!("{owner}: keys"),
                "a key name is empty or padded with spaces",
            ));
        }
        if names.iter().any(|seen| seen == name) {
            return Err(fail(
                format!("{owner}: keys"),
                format!("`{name}` is listed twice"),
            ));
        }
        names.push(name.to_string());
    }
    Ok(names)
}

fn parse_source(table: &dyn TableLike, owner: &str) -> Result<SourceFact, UsherError> {
    let source = table
        .get("source")
        .and_then(TomlItem::as_table_like)
        .ok_or_else(|| {
            fail(
                format!("{owner}: source"),
                "an item of class `sources` needs a `source` table (path, layer, read, rank)",
            )
        })?;
    let owner = format!("{owner} source");
    reject_unknown_keys(
        source,
        &[
            "path",
            "layer",
            "read",
            "rank",
            "always_present",
            "platforms",
        ],
        &owner,
    )?;
    let path = text_field(source, "path", &owner)?;
    check_path(path, &owner, "source path", &[])?;
    let layer_name = text_field(source, "layer", &owner)?;
    let layer = SourceLayer::parse(layer_name).ok_or_else(|| {
        fail(
            format!("{owner}: layer"),
            format!(
                "`{layer_name}` is not one of managed, user, project, local, global-state, server-managed-cache"
            ),
        )
    })?;
    let read_name = text_field(source, "read", &owner)?;
    let read = ReadRule::parse(read_name).ok_or_else(|| {
        fail(
            format!("{owner}: read"),
            format!("`{read_name}` is not one of read, not-read, first-wins"),
        )
    })?;
    let rank = u32::try_from(number_field(source, "rank", &owner)?)
        .map_err(|_| fail(format!("{owner}: rank"), "`rank` is too large"))?;
    let always_present = source
        .get("always_present")
        .and_then(TomlItem::as_bool)
        .ok_or_else(|| {
            fail(
                format!("{owner}: always_present"),
                "`always_present` is missing or not a boolean (true or false)",
            )
        })?;
    let platforms = parse_platforms(source, &owner)?;
    Ok(SourceFact {
        path: path.to_string(),
        layer,
        read,
        rank,
        always_present,
        platforms,
    })
}

impl Platform {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "macos" => Some(Platform::Macos),
            "linux" => Some(Platform::Linux),
            "windows" => Some(Platform::Windows),
            _ => None,
        }
    }
}

fn parse_platforms(
    source: &dyn TableLike,
    owner: &str,
) -> Result<Option<Vec<Platform>>, UsherError> {
    let Some(entry) = source.get("platforms") else {
        return Ok(None);
    };
    let list = entry.as_array().ok_or_else(|| {
        fail(
            format!("{owner}: platforms"),
            "`platforms` must be a list of macos, linux and windows",
        )
    })?;
    if list.is_empty() {
        return Err(fail(
            format!("{owner}: platforms"),
            "`platforms` is empty; leave the field out to mean every platform",
        ));
    }
    let mut platforms = Vec::new();
    for value in list {
        let name = value.as_str().ok_or_else(|| {
            fail(
                format!("{owner}: platforms"),
                "every entry of `platforms` must be a string",
            )
        })?;
        let platform = Platform::parse(name).ok_or_else(|| {
            fail(
                format!("{owner}: platforms"),
                format!("`{name}` is not one of macos, linux, windows"),
            )
        })?;
        if platforms.contains(&platform) {
            return Err(fail(
                format!("{owner}: platforms"),
                format!("`{name}` is listed twice in `platforms`"),
            ));
        }
        platforms.push(platform);
    }
    Ok(Some(platforms))
}

fn parse_variable(table: &dyn TableLike, owner: &str) -> Result<LaunchVariable, UsherError> {
    let variable = table
        .get("variable")
        .and_then(TomlItem::as_table_like)
        .ok_or_else(|| {
            fail(
                format!("{owner}: variable"),
                "`variable` must be a table (name, and optionally binds and default)",
            )
        })?;
    let owner = format!("{owner} variable");
    reject_unknown_keys(variable, &["name", "binds", "default"], &owner)?;
    let name = text_field(variable, "name", &owner)?;
    let mut chars = name.chars();
    let first_ok = chars.next().is_some_and(|c| c.is_ascii_uppercase());
    if !first_ok || !chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
        return Err(fail(
            format!("{owner}: name"),
            format!(
                "`{name}` is not a variable name: upper-case letters, digits and underscores, starting with a letter"
            ),
        ));
    }
    let binds = match variable.get("binds") {
        None => None,
        Some(_) => {
            let token = text_field(variable, "binds", &owner)?;
            if !BINDABLE_TOKENS.contains(&token) {
                return Err(fail(
                    format!("{owner}: binds"),
                    format!(
                        "`{token}` cannot be bound; only {} can",
                        BINDABLE_TOKENS.join(" and ")
                    ),
                ));
            }
            Some(token.to_string())
        }
    };
    let default = match variable.get("default") {
        None => None,
        Some(_) => Some(text_field(variable, "default", &owner)?.to_string()),
    };
    match (&binds, &default) {
        (Some(_), None) => {
            return Err(fail(
                format!("{owner}: default"),
                "a variable that binds a token needs a `default`, the path the token takes while the variable is unset",
            ));
        }
        (None, Some(_)) => {
            return Err(fail(
                format!("{owner}: default"),
                "`default` is only for a variable that binds a token",
            ));
        }
        _ => {}
    }
    if let Some(default) = &default {
        check_path(default, &owner, "default", &[])?;
        for token in PATH_TOKENS {
            if default.contains(token) && !DEFAULT_TOKENS.contains(&token) {
                return Err(fail(
                    format!("{owner}: default"),
                    format!(
                        "`{default}` uses {token}; a default may use only {}",
                        DEFAULT_TOKENS.join(" and ")
                    ),
                ));
            }
        }
    }
    Ok(LaunchVariable {
        name: name.to_string(),
        binds,
        default,
    })
}

/// A key path is names joined by dots; `*` stands for exactly one whole level. No path
/// may start at or pass through a credential key, and a leading `*` could reach one.
fn check_key_path(path: &str, owner: &str) -> Result<(), UsherError> {
    let bad = |why: String| fail(format!("{owner}: key_paths"), format!("`{path}` {why}"));
    if path.is_empty() {
        return Err(fail(
            format!("{owner}: key_paths"),
            "a key path is empty".to_string(),
        ));
    }
    for (index, segment) in path.split('.').enumerate() {
        let plain = !segment.is_empty()
            && segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if segment != "*" && !plain {
            return Err(bad(
                "is not names joined by dots; `*` must be a whole level on its own".to_string(),
            ));
        }
        if CREDENTIAL_KEYS.contains(&segment) {
            return Err(bad(format!(
                "starts at or passes through the credential key `{segment}`"
            )));
        }
        if index == 0 && segment == "*" {
            return Err(bad(
                "starts with `*`, which could reach a credential key".to_string()
            ));
        }
    }
    Ok(())
}

fn string_list(
    seat: &dyn TableLike,
    key: &str,
    owner: &str,
) -> Result<Option<Vec<String>>, UsherError> {
    let Some(entry) = seat.get(key) else {
        return Ok(None);
    };
    let list = entry.as_array().ok_or_else(|| {
        fail(
            format!("{owner}: {key}"),
            format!("`{key}` must be a list of strings"),
        )
    })?;
    let mut values = Vec::new();
    for value in list {
        let text = value.as_str().ok_or_else(|| {
            fail(
                format!("{owner}: {key}"),
                format!("every entry of `{key}` must be a string"),
            )
        })?;
        values.push(text.to_string());
    }
    Ok(Some(values))
}

fn parse_seat(table: &dyn TableLike, owner: &str) -> Result<SeatFact, UsherError> {
    let seat = table
        .get("seat")
        .and_then(TomlItem::as_table_like)
        .ok_or_else(|| {
            fail(
                format!("{owner}: seat"),
                "an item of class `seat` needs a `seat` table (part, path, read)",
            )
        })?;
    let owner = format!("{owner} seat");
    reject_unknown_keys(
        seat,
        &[
            "part",
            "path",
            "read",
            "key_paths",
            "keys",
            "variables",
            "platforms",
        ],
        &owner,
    )?;
    let part_name = text_field(seat, "part", &owner)?;
    let part = SeatPart::parse(part_name).ok_or_else(|| {
        fail(
            format!("{owner}: part"),
            format!("`{part_name}` is not one of mcp, packs, fleet-client, auth-route, toolchain"),
        )
    })?;
    let read_name = text_field(seat, "read", &owner)?;
    let read = SeatRead::parse(read_name).ok_or_else(|| {
        fail(
            format!("{owner}: read"),
            format!("`{read_name}` is not one of presence, listing, names, env, keys-present"),
        )
    })?;
    let path = match (read, seat.get("path")) {
        (SeatRead::Env, Some(_)) => {
            return Err(fail(
                format!("{owner}: path"),
                "a read of `env` has no `path`; it reads environment variables",
            ));
        }
        (SeatRead::Env, None) => None,
        (_, None) => {
            return Err(fail(
                format!("{owner}: path"),
                format!("a read of `{}` needs a `path`", read.name()),
            ));
        }
        (_, Some(_)) => {
            let path = text_field(seat, "path", &owner)?;
            check_path(path, &owner, "path", &[])?;
            Some(path.to_string())
        }
    };
    // Each list belongs to one read rule and to no other.
    for (key, belongs) in [
        ("key_paths", SeatRead::Names),
        ("keys", SeatRead::KeysPresent),
        ("variables", SeatRead::Env),
    ] {
        if seat.contains_key(key) && read != belongs {
            return Err(fail(
                format!("{owner}: {key}"),
                format!(
                    "`{key}` belongs to read `{}`, not `{}`",
                    belongs.name(),
                    read.name()
                ),
            ));
        }
    }
    let list = |key: &str, belongs: SeatRead| -> Result<Vec<String>, UsherError> {
        if read != belongs {
            return Ok(Vec::new());
        }
        let values = string_list(seat, key, &owner)?.unwrap_or_default();
        if values.is_empty() {
            return Err(fail(
                format!("{owner}: {key}"),
                format!(
                    "a read of `{}` needs a non-empty `{key}` list",
                    belongs.name()
                ),
            ));
        }
        Ok(values)
    };
    let key_paths = list("key_paths", SeatRead::Names)?;
    for path in &key_paths {
        check_key_path(path, &owner)?;
    }
    let keys = list("keys", SeatRead::KeysPresent)?;
    for (index, key) in keys.iter().enumerate() {
        let plain = !key.is_empty()
            && key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if !plain {
            return Err(fail(
                format!("{owner}: keys"),
                format!("`{key}` is not a plain top-level key name: no dot, no `*`, not empty"),
            ));
        }
        if keys[..index].contains(key) {
            return Err(fail(
                format!("{owner}: keys"),
                format!("`{key}` is listed twice"),
            ));
        }
    }
    let variables = list("variables", SeatRead::Env)?;
    for name in &variables {
        let mut chars = name.chars();
        let ok = chars.next().is_some_and(|c| c.is_ascii_uppercase())
            && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        if !ok {
            return Err(fail(
                format!("{owner}: variables"),
                format!(
                    "`{name}` is not a variable name: upper-case letters, digits and underscores, starting with a letter"
                ),
            ));
        }
    }
    let platforms = parse_platforms(seat, &owner)?;
    Ok(SeatFact {
        part,
        read,
        path,
        key_paths,
        keys,
        variables,
        platforms,
    })
}

fn parse_item(table: &dyn TableLike, index: usize) -> Result<Item, UsherError> {
    let id = text_field(table, "id", &format!("item #{index}"))?;
    let owner = format!("item `{id}`");
    reject_unknown_keys(
        table,
        &[
            "id",
            "harness",
            "layer",
            "class",
            "versions",
            "summary",
            "reference",
            "last_verified",
            "source",
            "keys",
            "version_from",
            "variable",
            "seat",
        ],
        &owner,
    )?;
    let harness = text_field(table, "harness", &owner)?;
    let layer_name = text_field(table, "layer", &owner)?;
    let layer = Layer::parse(layer_name).ok_or_else(|| {
        fail(
            format!("{owner}: layer"),
            format!("`{layer_name}` is not slow or fast"),
        )
    })?;
    let class_name = text_field(table, "class", &owner)?;
    let class = Class::parse(class_name).ok_or_else(|| {
        fail(
            format!("{owner}: class"),
            format!("`{class_name}` is not a known class"),
        )
    })?;
    if class.layer() != layer {
        return Err(fail(
            format!("{owner}: class"),
            format!("class `{class_name}` belongs to the other layer than `{layer_name}`"),
        ));
    }
    let source = if class == Class::Sources {
        Some(parse_source(table, &owner)?)
    } else if table.contains_key("source") {
        return Err(fail(
            format!("{owner}: source"),
            format!("only items of class `sources` carry a `source` table, not `{class_name}`"),
        ));
    } else {
        None
    };
    let keys = if class == Class::Keys {
        parse_keys(table, &owner)?
    } else if table.contains_key("keys") {
        return Err(fail(
            format!("{owner}: keys"),
            format!("only items of class `keys` carry a `keys` list, not `{class_name}`"),
        ));
    } else {
        Vec::new()
    };
    let version_from = if class == Class::Paths && table.contains_key("version_from") {
        Some(parse_version_from(table, &owner)?)
    } else if table.contains_key("version_from") {
        return Err(fail(
            format!("{owner}: version_from"),
            format!("only items of class `paths` carry `version_from`, not `{class_name}`"),
        ));
    } else {
        None
    };
    let variable = if class == Class::Paths && table.contains_key("variable") {
        Some(parse_variable(table, &owner)?)
    } else if table.contains_key("variable") {
        return Err(fail(
            format!("{owner}: variable"),
            format!("only items of class `paths` carry `variable`, not `{class_name}`"),
        ));
    } else {
        None
    };
    let seat = if class == Class::Seat {
        Some(parse_seat(table, &owner)?)
    } else if table.contains_key("seat") {
        return Err(fail(
            format!("{owner}: seat"),
            format!("only items of class `seat` carry a `seat` table, not `{class_name}`"),
        ));
    } else {
        None
    };
    Ok(Item {
        id: id.to_string(),
        harness: harness.to_string(),
        layer,
        class,
        versions: parse_range(table, &owner)?,
        summary: text_field(table, "summary", &owner)?.to_string(),
        reference: parse_reference(table, id)?,
        last_verified: parse_date(table, id)?,
        source,
        keys,
        version_from,
        variable,
        seat,
        settings_key: None,
        stack: Vec::new(),
    })
}

/// Cross-item checks on launch variables: no two items of a harness bind the same token
/// over versions that overlap (one variable may bind two tokens), and every source or
/// seat path that uses a bindable token has a binder of it over the same versions.
fn check_variables(items: &[Item]) -> Result<(), UsherError> {
    let binders: Vec<(&Item, &LaunchVariable)> = items
        .iter()
        .filter_map(|item| item.variable.as_ref().map(|variable| (item, variable)))
        .collect();
    for (index, (first, one)) in binders.iter().enumerate() {
        for (second, two) in &binders[index + 1..] {
            if first.harness != second.harness || !first.versions.overlaps(&second.versions) {
                continue;
            }
            if let (Some(token), true) = (&one.binds, one.binds == two.binds) {
                return Err(fail(
                    format!("item `{}`", second.id),
                    format!(
                        "items `{}` and `{}` both bind the token `{token}` over versions that overlap",
                        first.id, second.id
                    ),
                ));
            }
        }
    }
    for item in items {
        let path = match (&item.source, &item.seat) {
            (Some(source), _) => Some((source.path.as_str(), "source")),
            (None, Some(seat)) => seat.path.as_deref().map(|path| (path, "seat")),
            _ => None,
        };
        let Some((path, what)) = path else { continue };
        for token in BINDABLE_TOKENS {
            if !path.contains(token) {
                continue;
            }
            let bound = binders.iter().any(|(binder, variable)| {
                binder.harness == item.harness
                    && variable.binds.as_deref() == Some(token)
                    && binder.versions.same_versions(&item.versions)
            });
            if !bound {
                return Err(fail(
                    format!("item `{}`", item.id),
                    format!(
                        "the {what} path uses {token}, and no `paths` item of harness `{}` binds it over the same versions as this item",
                        item.harness
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// A file a `keys-present` item reads must be one a `sources` item or a `names` seat item
/// of the same harness already names, over versions that overlap. Only those make it a
/// known JSON file; a `keys-present` read reads membership and confers no such standing.
fn check_anchors(items: &[Item]) -> Result<(), UsherError> {
    for item in items {
        let Some(seat) = &item.seat else { continue };
        if seat.read != SeatRead::KeysPresent {
            continue;
        }
        let Some(path) = seat.path.as_deref() else {
            continue;
        };
        let anchored = items.iter().any(|other| {
            other.id != item.id
                && other.harness == item.harness
                && other.versions.overlaps(&item.versions)
                && (other
                    .source
                    .as_ref()
                    .is_some_and(|source| source.path == path)
                    || other.seat.as_ref().is_some_and(|fact| {
                        fact.read == SeatRead::Names && fact.path.as_deref() == Some(path)
                    }))
        });
        if !anchored {
            return Err(fail(
                format!("item `{}`: path", item.id),
                format!(
                    "no sources or names item names `{path}`; name that file with a `sources` item or a `names` seat item of the same harness over overlapping versions"
                ),
            ));
        }
    }
    Ok(())
}

fn parse_support(table: &dyn TableLike, index: usize) -> Result<SupportEntry, UsherError> {
    let owner = format!("support entry #{index}");
    reject_unknown_keys(table, &["harness", "versions", "capabilities"], &owner)?;
    let harness = text_field(table, "harness", &owner)?;
    let versions = parse_range(table, &owner)?;
    let list = table
        .get("capabilities")
        .and_then(TomlItem::as_array)
        .ok_or_else(|| {
            fail(
                format!("{owner}: capabilities"),
                "`capabilities` must be a list, empty when nothing is offered",
            )
        })?;
    let mut capabilities = Vec::new();
    for entry in list {
        let name = entry.as_str().unwrap_or("(not a string)");
        let capability = Capability::parse(name).ok_or_else(|| {
            fail(
                format!("{owner}: capabilities"),
                format!("`{name}` is not one of inventory, install, map, compare, place, apply"),
            )
        })?;
        capabilities.push(capability);
    }
    Ok(SupportEntry {
        harness: harness.to_string(),
        versions,
        capabilities,
    })
}

impl KnowledgeFile {
    /// Parses a knowledge file and runs the structural check.
    ///
    /// Returns an error naming the first item that breaks the schema: a
    /// missing reference or date, an unknown layer, class, kind or capability,
    /// a malformed version range, or a duplicate id.
    pub fn parse(text: &str) -> Result<Self, UsherError> {
        let doc = text
            .parse::<DocumentMut>()
            .map_err(|why| fail("parse the TOML", why.to_string()))?;
        let header = doc.as_table() as &dyn TableLike;
        let schema_version = number_field(header, "schema_version", "file header")?;
        let schema_version = u32::try_from(schema_version).map_err(|_| {
            fail(
                "file header: schema_version",
                "`schema_version` is too large",
            )
        })?;
        if schema_version != SUPPORTED_SCHEMA {
            return Err(fail(
                "file header: schema_version",
                format!(
                    "schema {schema_version} is not supported; this build reads schema {SUPPORTED_SCHEMA}"
                ),
            ));
        }
        reject_unknown_keys(
            header,
            &["schema_version", "sequence", "item", "support"],
            "file header",
        )?;
        let sequence = number_field(header, "sequence", "file header")?;

        let mut items = Vec::new();
        let mut seen = HashSet::new();
        for (index, table) in tables(&doc, "item")?.into_iter().enumerate() {
            let item = parse_item(table, index + 1)?;
            if !seen.insert(item.id.clone()) {
                return Err(fail(
                    format!("item `{}`", item.id),
                    "two items share this id",
                ));
            }
            items.push(item);
        }
        check_variables(&items)?;
        check_anchors(&items)?;
        let mut support = Vec::new();
        for (index, table) in tables(&doc, "support")?.into_iter().enumerate() {
            support.push(parse_support(table, index + 1)?);
        }
        Ok(Self {
            version: KnowledgeVersion {
                schema_version,
                sequence,
            },
            items,
            support,
        })
    }

    /// The knowledge file built into this release.
    pub fn embedded() -> Result<Self, UsherError> {
        Self::parse(EMBEDDED)
    }

    /// The version of this file, for any report that relies on it.
    pub fn version(&self) -> KnowledgeVersion {
        self.version
    }

    /// Every item, in file order.
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// The launch variable names harness knowledge lists for `harness` across every
    /// version range, sorted and without repeats, for the variables that bind a path
    /// token. A variable that moves no path is never part of a key. Every caller keys an
    /// environment from this one lookup (AD-17), so the key does not move with the
    /// harness version.
    pub fn launch_variables(&self, harness: &str) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .items
            .iter()
            .filter(|item| item.harness == harness)
            .filter_map(|item| item.variable.as_ref())
            .filter(|variable| variable.binds.is_some())
            .map(|variable| variable.name.as_str())
            .collect();
        names.sort_unstable();
        names.dedup();
        names
    }

    /// The variable that binds `token` for `harness` at `installed`, with the default the
    /// token takes while it is unset or empty. Version-scoped, and never part of an
    /// environment key.
    pub fn binding(&self, harness: &str, installed: &Version, token: &str) -> Option<Binding> {
        self.items_for(harness, installed).find_map(|item| {
            let variable = item.variable.as_ref()?;
            if variable.binds.as_deref() != Some(token) {
                return None;
            }
            Some(Binding {
                variable: variable.name.clone(),
                default: variable.default.clone()?,
            })
        })
    }

    /// The items about `harness` that apply to `installed`.
    pub fn items_for<'a>(
        &'a self,
        harness: &'a str,
        installed: &'a Version,
    ) -> impl Iterator<Item = &'a Item> + 'a {
        self.items
            .iter()
            .filter(move |item| item.harness == harness && item.versions.contains(installed))
    }

    /// The support level for `harness` at `installed`: what usher offers, and
    /// everything else reading as unsupported.
    pub fn support(&self, harness: &str, installed: &Version) -> SupportReport {
        let entry = self
            .support
            .iter()
            .find(|entry| entry.harness == harness && entry.versions.contains(installed));
        SupportReport {
            harness: harness.to_string(),
            installed: *installed,
            known: entry.is_some(),
            offered: entry.map(|e| e.capabilities.clone()).unwrap_or_default(),
            knowledge: self.version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
schema_version = 4
sequence = 7

[[item]]
id = "demo.layer-order"
harness = "demo"
layer = "slow"
class = "layers"
versions = ">=1.2.0, <2.0.0"
summary = "A fact."
reference = { kind = "vendor-docs", locator = "https://example.test/docs" }
last_verified = 2026-11-02

[[item]]
id = "demo.default-model"
harness = "demo"
layer = "fast"
class = "defaults"
versions = ">=2.0.0"
summary = "Another fact."
reference = { kind = "probe", locator = "probe-demo-1" }
last_verified = 2026-11-03

[[support]]
harness = "demo"
versions = ">=1.2.0, <2.0.0"
capabilities = ["inventory", "map"]
"#;

    fn v(text: &str) -> Version {
        Version::parse(text).expect("a valid version")
    }

    fn rejects(text: &str, needle: &str) {
        let err = KnowledgeFile::parse(text).expect_err("the check must fail");
        let shown = err.to_string();
        assert!(shown.contains(needle), "`{needle}` not in `{shown}`");
        assert!(!err.step().is_empty());
        assert!(!err.check().is_empty());
        assert!(!err.status().is_empty());
        assert!(!err.next_action().is_empty());
    }

    #[test]
    fn a_valid_file_parses_with_both_layers_and_ranges() {
        let file = KnowledgeFile::parse(VALID).expect("valid");
        assert_eq!(file.items().len(), 2);
        assert_eq!(file.items()[0].layer(), Layer::Slow);
        assert_eq!(file.items()[1].layer(), Layer::Fast);
        let one_five = v("1.5.0");
        let for_one_five: Vec<&str> = file.items_for("demo", &one_five).map(Item::id).collect();
        assert_eq!(for_one_five, ["demo.layer-order"]);
        let two = v("2.0.0");
        let for_two: Vec<&str> = file.items_for("demo", &two).map(Item::id).collect();
        assert_eq!(for_two, ["demo.default-model"]);
        assert_eq!(file.items_for("other", &one_five).count(), 0);
        let item = &file.items()[0];
        assert_eq!(item.reference().kind(), ReferenceKind::VendorDocs);
        assert_eq!(item.reference().locator(), "https://example.test/docs");
        assert_eq!(item.last_verified().to_string(), "2026-11-02");
    }

    #[test]
    fn an_item_without_a_reference_fails_naming_the_item() {
        let text = VALID.replace(
            "reference = { kind = \"vendor-docs\", locator = \"https://example.test/docs\" }\n",
            "",
        );
        rejects(&text, "demo.layer-order");
        rejects(&text, "no reference");
    }

    #[test]
    fn an_item_without_a_last_verified_date_fails_naming_the_item() {
        let text = VALID.replace("last_verified = 2026-11-03\n", "");
        rejects(&text, "demo.default-model");
        rejects(&text, "last_verified");
    }

    #[test]
    fn a_bad_date_or_reference_fails() {
        rejects(
            &VALID.replace("2026-11-02", "2026-11-02T10:00:00"),
            "no time part",
        );
        rejects(
            &VALID.replace("2026-11-02", "\"2026-11-02\""),
            "last_verified",
        );
        rejects(&VALID.replace("vendor-docs", "rumor"), "rumor");
        rejects(&VALID.replace("https://example.test/docs", ""), "locator");
    }

    #[test]
    fn unknown_layer_class_or_capability_fails() {
        rejects(
            &VALID.replace("layer = \"slow\"", "layer = \"medium\""),
            "medium",
        );
        rejects(
            &VALID.replace("class = \"layers\"", "class = \"vibes\""),
            "vibes",
        );
        rejects(&VALID.replace("\"map\"", "\"teleport\""), "teleport");
    }

    #[test]
    fn a_class_in_the_other_layer_fails() {
        rejects(
            &VALID.replace("layer = \"slow\"", "layer = \"fast\""),
            "other layer",
        );
    }

    #[test]
    fn duplicate_item_ids_fail() {
        rejects(
            &VALID.replace("demo.default-model", "demo.layer-order"),
            "share this id",
        );
    }

    #[test]
    fn a_malformed_range_fails() {
        rejects(&VALID.replace(">=2.0.0", "2.0"), "demo.default-model");
    }

    #[test]
    fn a_missing_or_bad_sequence_or_schema_version_fails() {
        rejects(&VALID.replace("sequence = 7\n", ""), "sequence");
        rejects(
            &VALID.replace("sequence = 7", "sequence = \"seven\""),
            "sequence",
        );
        rejects(&VALID.replace("schema_version = 4\n", ""), "schema_version");
    }

    #[test]
    fn invalid_toml_fails_with_the_four_error_fields() {
        rejects("this is = = not toml", "parse the TOML");
    }

    #[test]
    fn items_must_be_tables() {
        rejects("schema_version = 4\nsequence = 1\nitem = 3\n", "[[item]]");
    }

    #[test]
    fn the_file_reports_its_version() {
        let file = KnowledgeFile::parse(VALID).expect("valid");
        assert_eq!(file.version().sequence(), 7);
        assert_eq!(file.version().schema_version(), 4);
        assert_eq!(file.version().to_string(), "knowledge v7 (schema 4)");
    }

    #[test]
    fn a_support_level_lists_offered_capabilities_and_marks_the_rest_unsupported() {
        let file = KnowledgeFile::parse(VALID).expect("valid");
        let report = file.support("demo", &v("1.5.0"));
        assert!(report.known());
        assert_eq!(report.offered(), [Capability::Inventory, Capability::Map]);
        assert_eq!(
            report.unsupported(),
            [
                Capability::Install,
                Capability::Compare,
                Capability::Place,
                Capability::Apply
            ]
        );
        let lines = report.lines();
        assert!(lines.contains(&"demo 1.5.0: map offered".to_string()));
        assert!(lines.contains(&"demo 1.5.0: apply unsupported".to_string()));
        assert_eq!(
            lines.last().map(String::as_str),
            Some("knowledge v7 (schema 4)")
        );
    }

    #[test]
    fn an_unknown_harness_or_version_offers_nothing_and_says_so() {
        let file = KnowledgeFile::parse(VALID).expect("valid");
        for report in [
            file.support("nope", &v("1.5.0")),
            file.support("demo", &v("3.0.0")),
        ] {
            assert!(!report.known());
            assert!(report.offered().is_empty());
            assert_eq!(report.unsupported(), Capability::ALL);
            assert!(report.lines()[0].contains("no support entry"));
            assert_eq!(report.knowledge_version().sequence(), 7);
        }
    }

    #[test]
    fn the_embedded_seed_file_passes_the_structural_check() {
        let file = KnowledgeFile::embedded().expect("the embedded file must pass the check");
        assert!(!file.items().is_empty());
        for item in file.items() {
            assert!(!item.reference().locator().is_empty(), "{}", item.id());
            assert!(
                item.last_verified()
                    >= Date {
                        year: 2026,
                        month: 1,
                        day: 1
                    }
            );
        }
    }

    #[test]
    fn the_embedded_seed_offers_no_capability_and_says_so() {
        let file = KnowledgeFile::embedded().expect("embedded");
        for (harness, version) in [
            ("claude-code", "2.1.317"),
            ("codex", "0.160.1"),
            ("goose", "1.57.0"),
        ] {
            let report = file.support(harness, &v(version));
            assert!(report.known(), "{harness} has a support entry");
            assert!(report.offered().is_empty(), "{harness} offers nothing yet");
            assert_eq!(report.unsupported(), Capability::ALL);
        }
    }
}
