//! The `profile.toml` manifest. Stub: nothing parses yet.

use usher_core::{UsherError, VersionRange};

/// A parsed `profile.toml`.
#[derive(Debug, Clone)]
pub struct Manifest {
    name: String,
    version: String,
    harnesses: Vec<HarnessCover>,
    tools: Vec<Tool>,
    mcp_servers: Vec<McpServer>,
    notes: Vec<Note>,
}

/// A harness the profile covers, with the versions its fragment is for.
#[derive(Debug, Clone)]
pub struct HarnessCover {
    id: String,
    versions: VersionRange,
}

/// A tool the profile declares, with the versions it needs.
#[derive(Debug, Clone)]
pub struct Tool {
    name: String,
    versions: VersionRange,
}

/// A shared MCP server entry, kept as written.
#[derive(Debug, Clone)]
pub struct McpServer {
    name: String,
    table: toml_edit::Table,
}

/// A note on one entry.
#[derive(Debug, Clone)]
pub struct Note {
    key: String,
    text: String,
}

impl Manifest {
    /// Parses `text` as a `profile.toml`. `origin` is the profile's path as the
    /// person knows it, and every error names it.
    pub fn parse(_text: &str, origin: &str) -> Result<Manifest, UsherError> {
        Err(UsherError::new(
            format!("reading profile {origin}"),
            "profile.toml",
            "not parsed yet",
            "wait for the parser",
        ))
    }

    /// The profile's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The profile's version text.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The harnesses covered.
    pub fn harnesses(&self) -> &[HarnessCover] {
        &self.harnesses
    }

    /// The tools declared.
    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    /// The shared MCP servers.
    pub fn mcp_servers(&self) -> &[McpServer] {
        &self.mcp_servers
    }

    /// The notes.
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }
}

impl HarnessCover {
    /// The harness id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The versions the fragment is for.
    pub fn versions(&self) -> &VersionRange {
        &self.versions
    }
}

impl Tool {
    /// The tool name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The versions the tool must be in.
    pub fn versions(&self) -> &VersionRange {
        &self.versions
    }
}

impl McpServer {
    /// The server name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// A string field as written, or `None` when absent or not a string.
    pub fn field_str(&self, key: &str) -> Option<&str> {
        self.table.get(key).and_then(|item| item.as_str())
    }

    /// The names of every field, in the order written.
    pub fn field_names(&self) -> Vec<&str> {
        self.table.iter().map(|(key, _)| key).collect()
    }
}

impl Note {
    /// The entry the note is on, as `<harness-id>:<key path>` or `mcp:<name>`.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The note text.
    pub fn text(&self) -> &str {
        &self.text
    }
}
