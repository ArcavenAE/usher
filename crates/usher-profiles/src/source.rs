//! Profile sources: a directory or a file read into profiles with their
//! fragments (stub: every load is refused until the change that implements it).

use std::path::{Path, PathBuf};

use usher_core::UsherError;

use crate::Manifest;

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

// The stub reads none of these fields; the change that implements the reader does.
#[allow(dead_code)]
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

// Stub only: see above.
#[allow(dead_code)]
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

/// One fragment, parsed.
#[derive(Debug, Clone)]
pub struct Fragment {
    leaves: Vec<String>,
}

impl Fragment {
    /// The leaves in the printed key-path form. Arrays are leaves.
    pub fn leaves(&self) -> &[String] {
        &self.leaves
    }

    /// The string at `path` (printed form), if the entry is a string.
    pub fn string_at(&self, _path: &str) -> Option<String> {
        None
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
}

impl Profile {
    /// Loads the profile or profiles `source` names. One invalid profile
    /// makes the whole source invalid.
    pub fn load(_source: &Source, _rules: &LoadRules) -> Result<Vec<Profile>, UsherError> {
        Err(UsherError::new(
            "load the profile source".to_string(),
            "the source is read".to_string(),
            "reading sources is not implemented".to_string(),
            "implement the reader",
        ))
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// The profile directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The fragment of one covered harness.
    pub fn fragment(&self, _harness: &str) -> Option<&Fragment> {
        None
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
    pub fn mcp_entry(&self, _harness: &str, _name: &str) -> Option<McpEntry> {
        None
    }
}
