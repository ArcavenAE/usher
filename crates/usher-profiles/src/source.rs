//! Profile sources: a directory or a file read into profiles with their
//! fragments, under the reader's limits.
//!
//! The walk lists every entry with `symlink_metadata` and refuses a symlink
//! anywhere in the profile before anything is read. Only `profile.toml` and
//! the fragment file the caller's table names for each covered harness are
//! opened; every other file is listed as not read and never opened. A source
//! with one invalid profile is invalid as a whole, and nothing is returned
//! from it.

use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

use usher_core::UsherError;

use crate::Manifest;
use crate::display::display_name;
use crate::fragment::{self, Node};

/// The most bytes one file is read at.
const FILE_CAP: u64 = 1024 * 1024;
/// The most bytes read from one profile in total.
const TOTAL_CAP: u64 = 4 * 1024 * 1024;
/// The most files one profile holds, read or not.
const FILES_CAP: usize = 64;
/// The most directory levels below the profile directory.
const DEPTH_CAP: usize = 8;
/// The most profiles one collection holds.
const PROFILES_CAP: usize = 64;

/// Where a profile comes from.
#[derive(Debug, Clone)]
pub enum Source {
    /// A profile directory, or a directory holding `profiles/<name>/`.
    Directory(PathBuf),
    /// A path to a `profile.toml`; its directory is the profile.
    File(PathBuf),
}

/// The format of a fragment file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FragmentFormat {
    Json,
    Toml,
    Yaml,
}

impl FragmentFormat {
    fn label(self) -> &'static str {
        match self {
            FragmentFormat::Json => "JSON",
            FragmentFormat::Toml => "TOML",
            FragmentFormat::Yaml => "YAML",
        }
    }
}

/// What the caller says about one harness: the native file name of its
/// fragment, the format of that file and where MCP servers sit in it.
#[derive(Debug, Clone)]
pub struct HarnessRule {
    id: String,
    file_name: String,
    format: FragmentFormat,
    mcp_path: Option<String>,
}

impl HarnessRule {
    /// A harness with no per-harness MCP override.
    pub fn new(id: &str, file_name: &str, format: FragmentFormat) -> HarnessRule {
        HarnessRule {
            id: id.to_string(),
            file_name: file_name.to_string(),
            format,
            mcp_path: None,
        }
    }

    /// The key path (printed form) of the table that holds the harness's MCP servers.
    pub fn with_mcp_path(mut self, path: &str) -> HarnessRule {
        self.mcp_path = Some(path.to_string());
        self
    }
}

/// The caller's table: harness rules and the credential-key names.
#[derive(Debug, Clone, Default)]
pub struct LoadRules {
    harnesses: Vec<HarnessRule>,
    credential_names: Vec<String>,
}

impl LoadRules {
    pub fn new(harnesses: Vec<HarnessRule>) -> LoadRules {
        LoadRules {
            harnesses,
            credential_names: Vec::new(),
        }
    }

    /// Names that mark a key path as holding a credential.
    pub fn with_credential_names(mut self, names: &[&str]) -> LoadRules {
        self.credential_names = names.iter().map(|n| (*n).to_string()).collect();
        self
    }
}

/// MCP servers of a fragment: each name with its string fields.
type McpServers = Vec<(String, Vec<(String, String)>)>;

/// One fragment, parsed.
#[derive(Debug, Clone)]
pub struct Fragment {
    leaves: Vec<String>,
    tree: Node,
    mcp: McpServers,
}

impl Fragment {
    /// The leaves in the printed key-path form. Arrays are leaves.
    pub fn leaves(&self) -> &[String] {
        &self.leaves
    }

    /// The string at `path` (printed form), if the entry is a string.
    pub fn string_at(&self, path: &str) -> Option<String> {
        let segments = fragment::parse_path(path)?;
        match self.tree.at(&segments)? {
            Node::Leaf(text) => text.clone(),
            Node::Object(_) => None,
        }
    }
}

/// A key path flagged as holding a credential. It never holds the value.
#[derive(Debug, Clone)]
pub struct CredentialFlag {
    harness: String,
    key_path: String,
}

impl CredentialFlag {
    pub fn harness(&self) -> &str {
        &self.harness
    }

    pub fn key_path(&self) -> &str {
        &self.key_path
    }
}

/// Where an MCP entry came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpOrigin {
    Shared,
    Fragment,
}

/// The MCP entry that applies to one harness.
#[derive(Debug, Clone)]
pub struct McpEntry {
    origin: McpOrigin,
    fields: Vec<(String, String)>,
}

impl McpEntry {
    pub fn origin(&self) -> McpOrigin {
        self.origin
    }

    pub fn field_str(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// A loaded profile.
#[derive(Debug, Clone)]
pub struct Profile {
    manifest: Manifest,
    path: PathBuf,
    not_read: Vec<String>,
    flags: Vec<CredentialFlag>,
    fragments: Vec<(String, Fragment)>,
}

impl Profile {
    /// Loads the profile or profiles `source` names. One invalid profile
    /// makes the whole source invalid.
    pub fn load(source: &Source, rules: &LoadRules) -> Result<Vec<Profile>, UsherError> {
        match source {
            Source::Directory(root) => load_directory(root, rules),
            Source::File(path) => load_file(path, rules),
        }
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// The profile directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The fragment of one covered harness.
    pub fn fragment(&self, harness: &str) -> Option<&Fragment> {
        self.fragments
            .iter()
            .find(|(id, _)| id == harness)
            .map(|(_, fragment)| fragment)
    }

    /// Files found under the profile and never opened, as `/`-separated
    /// paths relative to the profile directory.
    pub fn not_read(&self) -> &[String] {
        &self.not_read
    }

    pub fn credential_flags(&self) -> &[CredentialFlag] {
        &self.flags
    }

    /// The MCP entry of `name` for `harness`: the fragment's entry as a whole
    /// where it has one, else the shared entry.
    pub fn mcp_entry(&self, harness: &str, name: &str) -> Option<McpEntry> {
        if !self.manifest.harnesses().iter().any(|h| h.id() == harness) {
            return None;
        }
        if let Some(fragment) = self.fragment(harness)
            && let Some((_, fields)) = fragment.mcp.iter().find(|(n, _)| n == name)
        {
            return Some(McpEntry {
                origin: McpOrigin::Fragment,
                fields: fields.clone(),
            });
        }
        let server = self
            .manifest
            .mcp_servers()
            .iter()
            .find(|s| s.name() == name)?;
        let fields = server
            .field_names()
            .into_iter()
            .filter_map(|key| {
                server
                    .field_str(key)
                    .map(|v| (key.to_string(), v.to_string()))
            })
            .collect();
        Some(McpEntry {
            origin: McpOrigin::Shared,
            fields,
        })
    }
}

// Errors: four fields, the step naming the profile path.

struct Ctx {
    step: String,
}

impl Ctx {
    fn new(path: &Path) -> Ctx {
        Ctx {
            step: format!("read the profile source at {}", shown(path)),
        }
    }

    fn refuse(&self, check: String, status: String, next: &str) -> UsherError {
        UsherError::new(self.step.clone(), check, status, next)
    }

    fn io(&self, what: &str, path: &Path, err: &std::io::Error) -> UsherError {
        self.refuse(
            format!("{what}: {} can be read", shown(path)),
            display_name(&err.to_string()),
            "check that the path exists and can be read",
        )
    }
}

fn shown(path: &Path) -> String {
    display_name(&path.to_string_lossy())
}

// The sources.

fn load_file(path: &Path, rules: &LoadRules) -> Result<Vec<Profile>, UsherError> {
    let ctx = Ctx::new(path);
    if path.file_name().and_then(|n| n.to_str()) != Some("profile.toml") {
        return Err(ctx.refuse(
            "a file source is a path to profile.toml".to_string(),
            format!("`{}` is not named profile.toml", shown(path)),
            "name the profile.toml of the profile, or name its directory",
        ));
    }
    let dir = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    check_directory(&ctx, dir)?;
    Ok(vec![load_profile(dir, rules)?])
}

fn load_directory(root: &Path, rules: &LoadRules) -> Result<Vec<Profile>, UsherError> {
    let ctx = Ctx::new(root);
    check_directory(&ctx, root)?;
    if fs::symlink_metadata(root.join("profile.toml")).is_ok() {
        return Ok(vec![load_profile(root, rules)?]);
    }
    let profiles_dir = root.join("profiles");
    let has_profiles = match fs::symlink_metadata(&profiles_dir) {
        Ok(meta) if meta.file_type().is_symlink() => {
            return Err(symlink_refusal(&ctx, &profiles_dir));
        }
        Ok(meta) => meta.is_dir(),
        Err(_) => false,
    };
    if !has_profiles {
        return Err(ctx.refuse(
            "the directory holds profile.toml or profiles/<name>/".to_string(),
            format!("{} holds no profile.toml and no profiles/ directory", shown(root)),
            "name a profile directory, a profile.toml, or a directory with profiles/<name>/profile.toml",
        ));
    }
    let mut names = Vec::new();
    let entries =
        fs::read_dir(&profiles_dir).map_err(|err| ctx.io("profiles", &profiles_dir, &err))?;
    for entry in entries {
        let entry = entry.map_err(|err| ctx.io("profiles", &profiles_dir, &err))?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path).map_err(|err| ctx.io("profiles", &path, &err))?;
        if meta.file_type().is_symlink() {
            return Err(symlink_refusal(&ctx, &path));
        }
        if meta.is_dir() {
            names.push(entry.file_name());
        }
    }
    names.sort();
    if names.len() > PROFILES_CAP {
        return Err(ctx.refuse(
            format!("a collection holds at most {PROFILES_CAP} profiles"),
            format!(
                "{} holds {} profiles, more than {PROFILES_CAP} profiles",
                shown(&profiles_dir),
                names.len()
            ),
            "split the collection",
        ));
    }
    let mut loaded = Vec::new();
    for name in names {
        let dir = profiles_dir.join(&name);
        let profile = load_profile(&dir, rules)?;
        let dir_name = name.to_string_lossy();
        if profile.manifest.name() != dir_name {
            return Err(Ctx::new(&dir).refuse(
                "the manifest name equals its profiles/ directory name".to_string(),
                format!(
                    "profile.toml names `{}` but its directory is `{}`",
                    display_name(profile.manifest.name()),
                    display_name(&dir_name)
                ),
                "rename the directory or the profile so the two match",
            ));
        }
        loaded.push(profile);
    }
    Ok(loaded)
}

fn symlink_refusal(ctx: &Ctx, path: &Path) -> UsherError {
    ctx.refuse(
        "a profile holds no symlink".to_string(),
        format!("{} is a symlink", shown(path)),
        "replace the symlink with the file or directory itself",
    )
}

/// The directory named is a real directory, not a symlink.
fn check_directory(ctx: &Ctx, dir: &Path) -> Result<(), UsherError> {
    let meta = fs::symlink_metadata(dir).map_err(|err| ctx.io("source", dir, &err))?;
    if meta.file_type().is_symlink() {
        return Err(symlink_refusal(ctx, dir));
    }
    if !meta.is_dir() {
        return Err(ctx.refuse(
            "the source is a directory".to_string(),
            format!("{} is not a directory", shown(dir)),
            "name a profile directory",
        ));
    }
    Ok(())
}

// One profile.

fn load_profile(dir: &Path, rules: &LoadRules) -> Result<Profile, UsherError> {
    let ctx = Ctx::new(dir);
    let files = walk(&ctx, dir)?;
    let Some(manifest_file) = files.iter().find(|f| f.rel == "profile.toml") else {
        return Err(ctx.refuse(
            "the profile holds profile.toml".to_string(),
            "profile.toml is missing".to_string(),
            "add a profile.toml",
        ));
    };
    let mut total = 0u64;
    let manifest_path = dir.join("profile.toml");
    let text = read_text(
        &ctx,
        &manifest_path,
        "profile.toml",
        &manifest_file.meta,
        &mut total,
    )?;
    let manifest = Manifest::parse(&text, &shown(&manifest_path))?;

    let mut used = vec!["profile.toml".to_string()];
    let mut fragments: Vec<(String, Fragment)> = Vec::new();
    let mut flags = Vec::new();
    for cover in manifest.harnesses() {
        let id = cover.id();
        let Some(rule) = rules.harnesses.iter().find(|r| r.id == id) else {
            return Err(ctx.refuse(
                format!(
                    "the caller's table has a rule for harness `{}`",
                    display_name(id)
                ),
                format!("no rule for harness `{}`", display_name(id)),
                "cover only harnesses the caller knows, or add a rule for it",
            ));
        };
        let file_name = Path::new(&rule.file_name);
        let single = {
            let mut parts = file_name.components();
            matches!(parts.next(), Some(Component::Normal(_))) && parts.next().is_none()
        };
        if !single {
            return Err(ctx.refuse(
                "a fragment file name is one name inside the harness directory".to_string(),
                format!(
                    "`{}` leaves the harness directory",
                    display_name(&rule.file_name)
                ),
                "use a plain file name in the caller's table",
            ));
        }
        let rel = format!("{id}/{}", rule.file_name);
        let Some(walked) = files.iter().find(|f| f.rel == rel) else {
            return Err(ctx.refuse(
                format!(
                    "harness `{}` has its fragment file {}",
                    display_name(id),
                    display_name(&rule.file_name)
                ),
                format!("{} is missing", display_name(&rel)),
                "add the fragment file or remove the harness from profile.toml",
            ));
        };
        let text = read_text(
            &ctx,
            &dir.join(id).join(file_name),
            &rel,
            &walked.meta,
            &mut total,
        )?;
        let tree = fragment::parse(rule.format, &text).map_err(|why| {
            ctx.refuse(
                format!("{}: valid {}", display_name(&rel), rule.format.label()),
                fragment::shown(&why),
                "fix the file",
            )
        })?;
        let mcp = mcp_servers(&ctx, rule, &tree, &rel)?;
        let paths = tree.leaf_paths();
        let leaves = paths.iter().map(|p| fragment::print_path(p)).collect();
        for path in &paths {
            let hit = path.iter().any(|segment| {
                rules
                    .credential_names
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(segment))
            });
            if hit {
                flags.push(CredentialFlag {
                    harness: id.to_string(),
                    key_path: fragment::print_path(path),
                });
            }
        }
        used.push(rel);
        fragments.push((id.to_string(), Fragment { leaves, tree, mcp }));
    }

    check_notes(&ctx, &manifest, &fragments)?;

    let not_read = files
        .into_iter()
        .map(|f| f.rel)
        .filter(|f| !used.contains(f))
        .collect();
    Ok(Profile {
        manifest,
        path: dir.to_path_buf(),
        not_read,
        flags,
        fragments,
    })
}

/// The servers under the harness's MCP key path, each with its string fields.
fn mcp_servers(
    ctx: &Ctx,
    rule: &HarnessRule,
    tree: &Node,
    rel: &str,
) -> Result<McpServers, UsherError> {
    let Some(path) = &rule.mcp_path else {
        return Ok(Vec::new());
    };
    let Some(segments) = fragment::parse_path(path) else {
        return Err(ctx.refuse(
            "the MCP key path in the caller's table is a key path".to_string(),
            format!("`{}` is not a key path", display_name(path)),
            "fix the caller's table",
        ));
    };
    match tree.at(&segments) {
        None => Ok(Vec::new()),
        Some(Node::Object(servers)) => Ok(servers
            .iter()
            .map(|(name, node)| {
                let fields = match node {
                    Node::Object(fields) => fields
                        .iter()
                        .filter_map(|(key, value)| match value {
                            Node::Leaf(Some(text)) => Some((key.clone(), text.clone())),
                            _ => None,
                        })
                        .collect(),
                    Node::Leaf(_) => Vec::new(),
                };
                (name.clone(), fields)
            })
            .collect()),
        Some(Node::Leaf(_)) => Err(ctx.refuse(
            format!(
                "{}: {} is a table of servers",
                display_name(rel),
                display_name(path)
            ),
            format!("{} is not an object", display_name(path)),
            "write the MCP servers as an object keyed by server name",
        )),
    }
}

/// A note names a leaf or an object of its harness's fragment, or an MCP
/// server in the shared table or in some fragment.
fn check_notes(
    ctx: &Ctx,
    manifest: &Manifest,
    fragments: &[(String, Fragment)],
) -> Result<(), UsherError> {
    for note in manifest.notes() {
        let key = note.key();
        let found = match key.split_once(':') {
            Some(("mcp", name)) => {
                manifest.mcp_servers().iter().any(|s| s.name() == name)
                    || fragments
                        .iter()
                        .any(|(_, f)| f.mcp.iter().any(|(n, _)| n == name))
            }
            Some((harness, path)) => {
                let segments = fragment::parse_path(path);
                match (fragments.iter().find(|(id, _)| id == harness), segments) {
                    (Some((_, fragment)), Some(segments)) => fragment.tree.at(&segments).is_some(),
                    _ => false,
                }
            }
            None => false,
        };
        if !found {
            return Err(ctx.refuse(
                format!(
                    "the note `{}` names an entry that exists",
                    display_name(key)
                ),
                format!("the note `{}` names no entry", display_name(key)),
                "name an entry of the fragment or a shared MCP server, or remove the note",
            ));
        }
    }
    Ok(())
}

/// One file the walk found: its path relative to the profile and the
/// metadata the walk took of it.
struct Walked {
    rel: String,
    meta: fs::Metadata,
}

/// Lists every file under `dir` as a `/`-separated path relative to it,
/// sorted, refusing a symlink, a depth past the cap and a file count past the
/// cap. Nothing is opened.
fn walk(ctx: &Ctx, dir: &Path) -> Result<Vec<Walked>, UsherError> {
    let mut files: Vec<Walked> = Vec::new();
    let mut pending: Vec<(PathBuf, String, usize)> = vec![(dir.to_path_buf(), String::new(), 0)];
    while let Some((path, rel, depth)) = pending.pop() {
        let mut names = Vec::new();
        for entry in fs::read_dir(&path).map_err(|err| ctx.io("directory", &path, &err))? {
            let entry = entry.map_err(|err| ctx.io("directory", &path, &err))?;
            names.push(entry.file_name());
        }
        names.sort();
        for name in names {
            let text = name.to_string_lossy();
            let child_rel = if rel.is_empty() {
                text.to_string()
            } else {
                format!("{rel}/{text}")
            };
            if depth > DEPTH_CAP {
                return Err(ctx.refuse(
                    format!("a profile nests at most {DEPTH_CAP} directory levels"),
                    format!(
                        "{} is deeper than {DEPTH_CAP} levels",
                        display_name(&child_rel)
                    ),
                    "flatten the profile directory",
                ));
            }
            let child = path.join(&name);
            let meta = fs::symlink_metadata(&child).map_err(|err| ctx.io("entry", &child, &err))?;
            if meta.file_type().is_symlink() {
                return Err(ctx.refuse(
                    "a profile holds no symlink".to_string(),
                    format!("{} is a symlink", display_name(&child_rel)),
                    "replace the symlink with the file or directory itself",
                ));
            }
            if meta.is_dir() {
                pending.push((child, child_rel, depth + 1));
            } else {
                if files.len() >= FILES_CAP {
                    return Err(ctx.refuse(
                        format!("a profile holds at most {FILES_CAP} files"),
                        format!("the profile holds more than {FILES_CAP} files"),
                        "remove files the profile does not need",
                    ));
                }
                files.push(Walked {
                    rel: child_rel,
                    meta,
                });
            }
        }
    }
    files.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(files)
}

/// Reads one file as text under the per-file and total caps. The size is
/// checked from metadata before the open and again on the bytes read.
fn read_text(
    ctx: &Ctx,
    path: &Path,
    rel: &str,
    _walked: &fs::Metadata,
    total: &mut u64,
) -> Result<String, UsherError> {
    let over_file = || {
        ctx.refuse(
            "a file in a profile is at most 1 MiB".to_string(),
            format!("{} is over 1 MiB", display_name(rel)),
            "shrink the file",
        )
    };
    let meta = fs::symlink_metadata(path).map_err(|err| ctx.io("file", path, &err))?;
    if !meta.is_file() {
        return Err(ctx.refuse(
            "a profile file is a regular file".to_string(),
            format!("{} is not a regular file", display_name(rel)),
            "replace it with a regular file",
        ));
    }
    if meta.len() > FILE_CAP {
        return Err(over_file());
    }
    let file = File::open(path).map_err(|err| ctx.io("file", path, &err))?;
    let opened = file.metadata().map_err(|err| ctx.io("file", path, &err))?;
    if !same_file(&meta, &opened) {
        return Err(symlink_refusal(ctx, path));
    }
    let mut bytes = Vec::new();
    file.take(FILE_CAP + 1)
        .read_to_end(&mut bytes)
        .map_err(|err| ctx.io("file", path, &err))?;
    if bytes.len() as u64 > FILE_CAP {
        return Err(over_file());
    }
    *total += bytes.len() as u64;
    if *total > TOTAL_CAP {
        return Err(ctx.refuse(
            "a profile reads at most 4 MiB in total".to_string(),
            format!("the files read pass 4 MiB at {}", display_name(rel)),
            "shrink the fragments",
        ));
    }
    String::from_utf8(bytes).map_err(|_| {
        ctx.refuse(
            format!("{} is UTF-8 text", display_name(rel)),
            format!("{} is not valid UTF-8", display_name(rel)),
            "save the file as UTF-8",
        )
    })
}

/// Whether the file that was opened is the one the walk checked. The walk
/// refuses a symlink with `symlink_metadata` and the open follows one, so a
/// file or a parent directory swapped for a symlink between the two would be
/// read. Comparing the device and inode of the opened file with the checked
/// one closes that race with std alone.
fn same_file(checked: &fs::Metadata, opened: &fs::Metadata) -> bool {
    checked.dev() == opened.dev() && checked.ino() == opened.ino()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{Ctx, read_text, same_file};

    #[test]
    fn two_different_files_are_not_the_same_file() {
        let dir = std::env::temp_dir().join(format!("usher-same-file-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("create scratch");
        let a = dir.join("a");
        let b = dir.join("b");
        fs::write(&a, "x").expect("write a");
        fs::write(&b, "x").expect("write b");
        let meta_a = fs::symlink_metadata(&a).expect("metadata a");
        let meta_b = fs::symlink_metadata(&b).expect("metadata b");
        let same = same_file(&meta_a, &meta_a);
        let different = same_file(&meta_a, &meta_b);
        let _ = fs::remove_dir_all(&dir);
        assert!(same, "a file is the same file as itself");
        assert!(!different, "two files with the same size are not one file");
    }

    #[test]
    fn a_file_is_read_only_if_it_is_the_one_the_walk_listed() {
        let dir = std::env::temp_dir().join(format!("usher-walked-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("create scratch");
        let a = dir.join("a");
        let b = dir.join("b");
        fs::write(&a, "x").expect("write a");
        fs::write(&b, "x").expect("write b");
        let meta_a = fs::symlink_metadata(&a).expect("metadata a");
        let meta_b = fs::symlink_metadata(&b).expect("metadata b");
        let ctx = Ctx::new(&dir);
        let mut total = 0;
        let own = read_text(&ctx, &a, "a", &meta_a, &mut total);
        let other = read_text(&ctx, &a, "a", &meta_b, &mut total);
        let _ = fs::remove_dir_all(&dir);
        assert!(own.is_ok(), "the file the walk listed is read");
        assert!(
            other.is_err(),
            "a file whose identity differs from the walk's is refused"
        );
    }
}
