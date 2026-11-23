//! The `profile.toml` manifest.

use toml_edit::{DocumentMut, Item, Table, Value};
use usher_core::{UsherError, VersionRange};

use crate::display::{display_name, is_escaped};

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
    ///
    /// Values are stored as written; path tokens stay unresolved.
    pub fn parse(text: &str, origin: &str) -> Result<Manifest, UsherError> {
        let refuse = Refuse {
            step: format!("read the profile at {}", display_name(origin)),
        };
        let doc: DocumentMut = text.parse().map_err(|err: toml_edit::TomlError| {
            refuse.error(
                "profile.toml: valid TOML".to_string(),
                format!("not valid TOML: {}", display_name(err.message())),
                "fix the TOML syntax in profile.toml",
            )
        })?;
        let root = doc.as_table();

        for (key, _) in root.iter() {
            if !TOP_KEYS.contains(&key) {
                return Err(refuse.unknown(&display_name(key), TOP_KEYS));
            }
        }

        let name = required_text(&refuse, root, "name")?;
        if !valid_name(name) {
            return Err(refuse.error(
                "profile.toml: name matches [a-z][a-z0-9-]{0,63}".to_string(),
                format!("name `{}` does not match", display_name(name)),
                "use lowercase letters, digits and hyphens, starting with a letter, at most 64 long",
            ));
        }

        let version = required_text(&refuse, root, "version")?;
        if version.is_empty() || version.chars().count() > 64 || version.chars().any(is_escaped) {
            return Err(refuse.error(
                "profile.toml: version is one line of 1 to 64 characters".to_string(),
                format!("version `{}` does not fit", display_name(version)),
                "write the version as one short line of text",
            ));
        }

        let harnesses = parse_harnesses(&refuse, root)?;
        let tools = parse_tools(&refuse, root)?;
        let mcp_servers = parse_mcp_servers(&refuse, root)?;
        let notes = parse_notes(&refuse, root, &harnesses)?;

        Ok(Manifest {
            name: name.to_string(),
            version: version.to_string(),
            harnesses,
            tools,
            mcp_servers,
            notes,
        })
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

const TOP_KEYS: &[&str] = &[
    "name",
    "version",
    "harnesses",
    "tools",
    "mcp_servers",
    "notes",
];

/// Builds the four-field refusals of one parse.
struct Refuse {
    step: String,
}

impl Refuse {
    fn error(&self, check: String, status: String, next: &str) -> UsherError {
        UsherError::new(self.step.clone(), check, status, next)
    }

    /// An unknown key. `shown` is already safe to print.
    fn unknown(&self, shown: &str, allowed: &[&str]) -> UsherError {
        self.error(
            format!("profile.toml: {shown} is a known key"),
            format!("unknown key `{shown}`"),
            &format!("remove it or use one of: {}", allowed.join(", ")),
        )
    }
}

/// A name or harness id: `[a-z][a-z0-9-]{0,63}`.
fn valid_name(text: &str) -> bool {
    let mut chars = text.chars();
    text.len() <= 64
        && chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// An MCP server name: 1 to 64 of letters, digits, `.`, `_`, `-`, not starting with `.`.
fn valid_mcp_name(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 64
        && !text.starts_with('.')
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// A tool name: 1 to 128 printable ASCII characters, no space and no quote.
fn valid_tool_name(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 128
        && text
            .chars()
            .all(|c| c.is_ascii_graphic() && c != '\'' && c != '"')
}

fn required_text<'a>(
    refuse: &Refuse,
    table: &'a Table,
    field: &str,
) -> Result<&'a str, UsherError> {
    match table.get(field) {
        None => Err(refuse.error(
            format!("profile.toml: {field} is present"),
            format!("`{field}` is missing"),
            &format!("add `{field} = \"...\"` to profile.toml"),
        )),
        Some(item) => item.as_str().ok_or_else(|| {
            refuse.error(
                format!("profile.toml: {field} is text"),
                format!("`{field}` is not a string"),
                &format!("write `{field}` as a quoted string"),
            )
        }),
    }
}

fn parse_harnesses(refuse: &Refuse, root: &Table) -> Result<Vec<HarnessCover>, UsherError> {
    let none = || {
        refuse.error(
            "profile.toml: harnesses holds at least one harness".to_string(),
            "the profile covers no harness".to_string(),
            "add a [harnesses.<id>] table with a versions range",
        )
    };
    let Some(item) = root.get("harnesses") else {
        return Err(none());
    };
    let Some(table) = item.as_table_like() else {
        return Err(refuse.error(
            "profile.toml: harnesses is a table".to_string(),
            "`harnesses` is not a table".to_string(),
            "write one [harnesses.<id>] table per harness",
        ));
    };
    let mut covers = Vec::new();
    for (id, value) in table.iter() {
        let shown = display_name(id);
        if !valid_name(id) {
            return Err(refuse.error(
                format!(
                    "profile.toml: harnesses.{shown} has an id matching [a-z][a-z0-9-]{{0,63}}"
                ),
                format!("harness id `{shown}` does not match"),
                "use lowercase letters, digits and hyphens, starting with a letter",
            ));
        }
        let Some(fields) = value.as_table_like() else {
            return Err(refuse.error(
                format!("profile.toml: harnesses.{shown} is a table"),
                format!("`harnesses.{shown}` is not a table"),
                "give it a versions range",
            ));
        };
        for (key, _) in fields.iter() {
            if key != "versions" {
                let key = display_name(key);
                return Err(refuse.error(
                    format!("profile.toml: harnesses.{shown}.{key} is a known key"),
                    format!("unknown key `{key}` under harnesses.{shown}"),
                    "remove it; a harness table holds only `versions`",
                ));
            }
        }
        let field = format!("harnesses.{shown}.versions");
        let versions = match fields.get("versions").and_then(Item::as_str) {
            Some(text) => VersionRange::parse(text).map_err(|why| {
                refuse.error(
                    format!("profile.toml: {field} is a satisfiable version range"),
                    display_name(&why),
                    "write a range such as \">=2.1.317, <3.0.0\"",
                )
            })?,
            None => {
                return Err(refuse.error(
                    format!("profile.toml: {field} is a version range"),
                    format!("`{field}` is missing or not a string"),
                    "write a range such as \">=2.1.317, <3.0.0\"",
                ));
            }
        };
        covers.push(HarnessCover {
            id: id.to_string(),
            versions,
        });
    }
    if covers.is_empty() {
        return Err(none());
    }
    Ok(covers)
}

fn parse_tools(refuse: &Refuse, root: &Table) -> Result<Vec<Tool>, UsherError> {
    let Some(item) = root.get("tools") else {
        return Ok(Vec::new());
    };
    let Some(table) = item.as_table_like() else {
        return Err(refuse.error(
            "profile.toml: tools is a table".to_string(),
            "`tools` is not a table".to_string(),
            "write `[tools]` with name = range pairs",
        ));
    };
    let mut tools = Vec::new();
    for (name, value) in table.iter() {
        let shown = display_name(name);
        if !valid_tool_name(name) {
            return Err(refuse.error(
                format!("profile.toml: tools.{shown} has a name of 1 to 128 printable characters without space or quote"),
                format!("tool name `{shown}` does not fit"),
                "rename the tool",
            ));
        }
        let field = format!("tools.{shown}");
        let Some(text) = value.as_str() else {
            return Err(refuse.error(
                format!("profile.toml: {field} is a version range"),
                format!("`{field}` is not a string"),
                "write the tool as name = \">=1.0.0\"",
            ));
        };
        let versions = VersionRange::parse(text).map_err(|why| {
            refuse.error(
                format!("profile.toml: {field} is a satisfiable version range"),
                display_name(&why),
                "write a range such as \">=22.0.0, <23.0.0\"",
            )
        })?;
        tools.push(Tool {
            name: name.to_string(),
            versions,
        });
    }
    Ok(tools)
}

fn to_table(item: &Item) -> Option<Table> {
    match item {
        Item::Table(table) => Some(table.clone()),
        Item::Value(Value::InlineTable(inline)) => Some(inline.clone().into_table()),
        _ => None,
    }
}

fn parse_mcp_servers(refuse: &Refuse, root: &Table) -> Result<Vec<McpServer>, UsherError> {
    let Some(item) = root.get("mcp_servers") else {
        return Ok(Vec::new());
    };
    let Some(table) = item.as_table_like() else {
        return Err(refuse.error(
            "profile.toml: mcp_servers is a table".to_string(),
            "`mcp_servers` is not a table".to_string(),
            "write one [mcp_servers.<name>] table per server",
        ));
    };
    let mut servers = Vec::new();
    for (name, value) in table.iter() {
        let shown = display_name(name);
        if !valid_mcp_name(name) {
            return Err(refuse.error(
                format!("profile.toml: mcp_servers.{shown} has a name of 1 to 64 of letters, digits, dot, underscore and hyphen, not starting with a dot"),
                format!("server name `{shown}` does not fit"),
                "rename the server",
            ));
        }
        let Some(fields) = to_table(value) else {
            return Err(refuse.error(
                format!("profile.toml: mcp_servers.{shown} is a table"),
                format!("`mcp_servers.{shown}` is not a table"),
                &format!("write it as [mcp_servers.{shown}]"),
            ));
        };
        servers.push(McpServer {
            name: name.to_string(),
            table: fields,
        });
    }
    Ok(servers)
}

fn parse_notes(
    refuse: &Refuse,
    root: &Table,
    harnesses: &[HarnessCover],
) -> Result<Vec<Note>, UsherError> {
    let Some(item) = root.get("notes") else {
        return Ok(Vec::new());
    };
    let Some(table) = item.as_table_like() else {
        return Err(refuse.error(
            "profile.toml: notes is a table".to_string(),
            "`notes` is not a table".to_string(),
            "write `[notes]` with key = \"text\" pairs",
        ));
    };
    let mut notes = Vec::new();
    for (key, value) in table.iter() {
        let shown = display_name(key);
        let field = format!("notes.{shown}");
        check_note_key(refuse, key, &shown, harnesses)?;
        let text = match value.as_str() {
            Some(text) if !text.is_empty() => text,
            _ => {
                return Err(refuse.error(
                    format!("profile.toml: {field} has text"),
                    format!("the note `{shown}` is empty or not a string"),
                    "write the note as a non-empty string",
                ));
            }
        };
        notes.push(Note {
            key: key.to_string(),
            text: text.to_string(),
        });
    }
    Ok(notes)
}

/// A note key is `mcp:<server name>` or `<covered harness id>:<key path>`.
fn check_note_key(
    refuse: &Refuse,
    key: &str,
    shown: &str,
    harnesses: &[HarnessCover],
) -> Result<(), UsherError> {
    let field = format!("notes.{shown}");
    let bad = |status: String, next: &str| {
        refuse.error(
            format!("profile.toml: {field} is a valid note key"),
            status,
            next,
        )
    };
    let Some((prefix, rest)) = key.split_once(':') else {
        return Err(bad(
            format!("the note key `{shown}` has no `:`"),
            "write the key as <harness-id>:<key path> or mcp:<server name>",
        ));
    };
    if prefix == "mcp" {
        if valid_mcp_name(rest) {
            return Ok(());
        }
        return Err(bad(
            format!("the note key `{shown}` names no valid MCP server"),
            "write mcp:<server name> with a name of letters, digits, dot, underscore and hyphen",
        ));
    }
    if !harnesses.iter().any(|h| h.id == prefix) {
        return Err(bad(
            format!(
                "the note key `{shown}` names harness `{}`, which [harnesses] does not hold",
                display_name(prefix)
            ),
            "cover the harness under [harnesses] or fix the key",
        ));
    }
    if rest.is_empty() {
        return Err(bad(
            format!("the note key `{shown}` has no key path"),
            "write a key path after the colon",
        ));
    }
    for segment in split_path(rest) {
        if let Err(why) = check_segment(segment) {
            return Err(bad(
                format!("segment `{}` of `{shown}` {why}", display_name(segment)),
                "write segments unquoted unless they hold a dot, whitespace or a double quote; quote those as a JSON string",
            ));
        }
    }
    Ok(())
}

/// Splits a key path at the dots outside quoted segments. A quoted segment
/// starts with `"` and ends at the next unescaped `"`; one that never closes
/// runs to the end of the path.
fn split_path(path: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut in_quote = false;
    let mut escaped = false;
    for (index, c) in path.char_indices() {
        if in_quote {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_quote = false;
            }
        } else if c == '"' && index == start {
            in_quote = true;
        } else if c == '.' {
            segments.push(&path[start..index]);
            start = index + 1;
        }
    }
    segments.push(&path[start..]);
    segments
}

/// One segment is unquoted, or a quoted JSON string that has to be quoted.
fn check_segment(segment: &str) -> Result<(), &'static str> {
    if segment.is_empty() {
        return Err("is empty");
    }
    let needs_quote = |text: &str| {
        text.chars()
            .any(|c| c == '.' || c == '"' || c.is_whitespace())
    };
    if segment.starts_with('"') {
        let decoded: String =
            serde_json::from_str(segment).map_err(|_| "is not a closed JSON string")?;
        if decoded.is_empty() {
            return Err("is empty");
        }
        if decoded.chars().any(is_escaped) {
            return Err("holds a control or direction character");
        }
        if !needs_quote(&decoded) {
            return Err("is quoted but holds no dot, whitespace or double quote");
        }
        return Ok(());
    }
    if segment.chars().any(is_escaped) {
        return Err("holds a control or direction character");
    }
    if needs_quote(segment) {
        return Err("holds a dot, whitespace or double quote and must be quoted");
    }
    Ok(())
}
