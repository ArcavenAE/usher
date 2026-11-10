//! Where a harness keeps its settings, as harness knowledge states it.
//!
//! An item of class `paths` may carry `locations`, each a path template, the
//! settings scope the file belongs to, and its shape. An item of class `keys`
//! may carry `keys`, the setting names the item cites. Adapters read both from
//! here instead of holding them in code (AD-6).

use toml_edit::{Item as TomlItem, TableLike};
use usher_core::UsherError;

use crate::schema::{fail, reject_unknown_keys, text_field};

/// The settings scope a source belongs to, from highest precedence down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Scope {
    /// Deployed by an organization: system files, MDM, or a cached server copy.
    Managed,
    /// A person's own file for every project.
    User,
    /// A file a project shares.
    Project,
    /// A person's own file for one project.
    Local,
    /// The harness's own state file, which also holds global settings.
    GlobalConfig,
}

impl Scope {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "managed" => Some(Scope::Managed),
            "user" => Some(Scope::User),
            "project" => Some(Scope::Project),
            "local" => Some(Scope::Local),
            "global-config" => Some(Scope::GlobalConfig),
            _ => None,
        }
    }

    /// The scope's name as it is written in knowledge and in reports.
    pub fn name(self) -> &'static str {
        match self {
            Scope::Managed => "managed",
            Scope::User => "user",
            Scope::Project => "project",
            Scope::Local => "local",
            Scope::GlobalConfig => "global-config",
        }
    }
}

/// What a location names on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Shape {
    /// One file.
    File,
    /// A directory whose visible `*.json` files are each a source, in name order.
    JsonDir,
}

impl Shape {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "file" => Some(Shape::File),
            "json-dir" => Some(Shape::JsonDir),
            _ => None,
        }
    }
}

/// The operating system a location applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Platform {
    /// Every system.
    Any,
    /// macOS only.
    Macos,
    /// Linux only.
    Linux,
}

impl Platform {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "any" => Some(Platform::Any),
            "macos" => Some(Platform::Macos),
            "linux" => Some(Platform::Linux),
            _ => None,
        }
    }
}

/// The placeholders a template may use; an adapter fills each one in.
pub const PLACEHOLDERS: [&str; 5] = ["home", "config_dir", "global_config_dir", "root", "system"];

/// One place a harness keeps a settings source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    template: String,
    scope: Scope,
    shape: Shape,
    platform: Platform,
}

impl Location {
    /// The path with `{home}`, `{config_dir}`, `{global_config_dir}`, `{root}`
    /// and `{system}` left for the adapter to fill in.
    pub fn template(&self) -> &str {
        &self.template
    }

    /// The scope the source belongs to.
    pub fn scope(&self) -> Scope {
        self.scope
    }

    /// Whether the path is one file or a directory of JSON files.
    pub fn shape(&self) -> Shape {
        self.shape
    }

    /// The system the location applies to.
    pub fn platform(&self) -> Platform {
        self.platform
    }
}

fn check_placeholders(template: &str, owner: &str) -> Result<(), UsherError> {
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            return Err(fail(
                format!("{owner}: template"),
                format!("`{template}` has a `{{` with no closing `}}`"),
            ));
        };
        let name = &after[..close];
        if !PLACEHOLDERS.contains(&name) {
            return Err(fail(
                format!("{owner}: template"),
                format!(
                    "`{{{name}}}` is not a placeholder; known ones are {}",
                    PLACEHOLDERS.join(", ")
                ),
            ));
        }
        rest = &after[close + 1..];
    }
    Ok(())
}

/// Reads an item's optional `locations` list.
pub(crate) fn parse_locations(
    table: &dyn TableLike,
    owner: &str,
) -> Result<Vec<Location>, UsherError> {
    let Some(entry) = table.get("locations") else {
        return Ok(Vec::new());
    };
    let list = entry.as_array().ok_or_else(|| {
        fail(
            format!("{owner}: locations"),
            "`locations` must be a list of tables",
        )
    })?;
    let mut locations = Vec::new();
    for (index, value) in list.iter().enumerate() {
        let here = format!("{owner} location #{}", index + 1);
        let location = value.as_inline_table().ok_or_else(|| {
            fail(
                format!("{here}"),
                "each location must be a table: { template, scope, shape, platform }",
            )
        })?;
        let location: &dyn TableLike = location;
        reject_unknown_keys(location, &["template", "scope", "shape", "platform"], &here)?;
        let template = text_field(location, "template", &here)?;
        check_placeholders(template, &here)?;
        let scope_name = text_field(location, "scope", &here)?;
        let scope = Scope::parse(scope_name).ok_or_else(|| {
            fail(
                format!("{here}: scope"),
                format!("`{scope_name}` is not managed, user, project, local or global-config"),
            )
        })?;
        let shape_name = text_field(location, "shape", &here)?;
        let shape = Shape::parse(shape_name).ok_or_else(|| {
            fail(
                format!("{here}: shape"),
                format!("`{shape_name}` is not file or json-dir"),
            )
        })?;
        let platform = match location.get("platform").and_then(TomlItem::as_str) {
            None => Platform::Any,
            Some(name) => Platform::parse(name).ok_or_else(|| {
                fail(
                    format!("{here}: platform"),
                    format!("`{name}` is not any, macos or linux"),
                )
            })?,
        };
        locations.push(Location {
            template: template.to_string(),
            scope,
            shape,
            platform,
        });
    }
    Ok(locations)
}

/// Reads an item's optional `keys` list of setting names.
pub(crate) fn parse_keys(table: &dyn TableLike, owner: &str) -> Result<Vec<String>, UsherError> {
    let Some(entry) = table.get("keys") else {
        return Ok(Vec::new());
    };
    let list = entry
        .as_array()
        .ok_or_else(|| fail(format!("{owner}: keys"), "`keys` must be a list of strings"))?;
    let mut keys: Vec<String> = Vec::new();
    for value in list.iter() {
        let name = value
            .as_str()
            .filter(|s| !s.is_empty() && *s == s.trim())
            .ok_or_else(|| {
                fail(
                    format!("{owner}: keys"),
                    "each key must be a non-empty string without padding",
                )
            })?;
        if keys.iter().any(|k| k == name) {
            return Err(fail(
                format!("{owner}: keys"),
                format!("`{name}` is listed twice"),
            ));
        }
        keys.push(name.to_string());
    }
    Ok(keys)
}

#[cfg(test)]
mod tests {
    use crate::KnowledgeFile;

    fn file(item_extra: &str, class: &str, layer: &str) -> String {
        format!(
            "schema_version = 1\nsequence = 1\n\n[[item]]\nid = \"x.item\"\nharness = \"x\"\nlayer = \"{layer}\"\nclass = \"{class}\"\nversions = \">=1.0.0\"\nsummary = \"s\"\nreference = {{ kind = \"vendor-docs\", locator = \"https://example.com\" }}\nlast_verified = 2026-11-10\n{item_extra}"
        )
    }

    #[test]
    fn locations_parse_with_scope_shape_and_platform() {
        let text = file(
            "locations = [ { template = \"{system}/etc/x/a.json\", scope = \"managed\", shape = \"file\", platform = \"linux\" }, { template = \"{root}/.x/b\", scope = \"project\", shape = \"json-dir\" } ]\n",
            "paths",
            "slow",
        );
        let parsed = KnowledgeFile::parse(&text).expect("valid");
        let locations = parsed.items()[0].locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].scope().name(), "managed");
        assert_eq!(locations[1].platform(), super::Platform::Any);
        assert_eq!(locations[1].shape(), super::Shape::JsonDir);
    }

    #[test]
    fn a_bad_location_fails_naming_the_item() {
        for (extra, needle) in [
            (
                "locations = [ { template = \"{nope}/a\", scope = \"user\", shape = \"file\" } ]\n",
                "not a placeholder",
            ),
            (
                "locations = [ { template = \"{home\", scope = \"user\", shape = \"file\" } ]\n",
                "no closing",
            ),
            (
                "locations = [ { template = \"a\", scope = \"world\", shape = \"file\" } ]\n",
                "is not managed",
            ),
            (
                "locations = [ { template = \"a\", scope = \"user\", shape = \"blob\" } ]\n",
                "not file or json-dir",
            ),
            (
                "locations = [ { template = \"a\", scope = \"user\", shape = \"file\", platform = \"bsd\" } ]\n",
                "not any, macos or linux",
            ),
            (
                "locations = [ { template = \"a\", scope = \"user\", shape = \"file\", extra = 1 } ]\n",
                "not a known key",
            ),
            ("locations = \"a\"\n", "list of tables"),
        ] {
            let err = KnowledgeFile::parse(&file(extra, "paths", "slow")).expect_err(extra);
            assert!(
                err.check().contains("x.item")
                    && format!("{} {}", err.check(), err.status()).contains(needle),
                "{extra}: {err}"
            );
        }
    }

    #[test]
    fn locations_belong_to_paths_items_and_keys_to_keys_items() {
        let loc = "locations = [ { template = \"a\", scope = \"user\", shape = \"file\" } ]\n";
        let err = KnowledgeFile::parse(&file(loc, "keys", "fast")).expect_err("wrong class");
        assert!(err.status().contains("paths"), "{err}");
        let err = KnowledgeFile::parse(&file("keys = [\"a\"]\n", "paths", "slow"))
            .expect_err("wrong class");
        assert!(err.status().contains("keys"), "{err}");
    }

    #[test]
    fn keys_parse_and_reject_duplicates_and_blanks() {
        let parsed = KnowledgeFile::parse(&file("keys = [\"model\", \"env\"]\n", "keys", "fast"))
            .expect("valid");
        assert_eq!(parsed.items()[0].keys(), ["model", "env"]);
        for bad in [
            "keys = [\"a\", \"a\"]\n",
            "keys = [\"\"]\n",
            "keys = [1]\n",
            "keys = \"a\"\n",
        ] {
            assert!(
                KnowledgeFile::parse(&file(bad, "keys", "fast")).is_err(),
                "{bad}"
            );
        }
    }
}
