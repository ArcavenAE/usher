//! Story 1.5, PR C: what the Claude Code adapter lists for one launch context. Runs
//! against the public API, a scratch home and a scratch system root.

use std::fs;
use std::path::{Path, PathBuf};

use usher_adapter_claude_code::ClaudeCodeAdapter;
use usher_core::{
    KeyNames, LaunchContext, SourceLister, SourceListing, SourceStatus, SystemRoot, environment_key,
};
use usher_knowledge::{KnowledgeFile, Platform};
use usher_testkit::ScratchHome;

const VERSION: &str = "2.1.317";
const LINUX_MANAGED: &str = "etc/claude-code/managed-settings.json";

struct Rig {
    scratch: ScratchHome,
    system: PathBuf,
    repo: PathBuf,
}

impl Rig {
    fn new() -> Self {
        let scratch = ScratchHome::new().expect("scratch");
        let system = scratch.root().join("system");
        let repo = scratch.home().join("work/repo");
        fs::create_dir_all(&system).expect("system root");
        fs::create_dir_all(&repo).expect("repo");
        Rig {
            scratch,
            system,
            repo,
        }
    }

    fn context(&self, version: Option<&str>, with_repo: bool) -> LaunchContext {
        let home = self.scratch.home().to_path_buf();
        let key = environment_key("host", "alex", "claude-code", &self.repo, &[])
            .expect("an absolute directory");
        LaunchContext {
            key,
            config_dir: home.join(".claude"),
            global_state_dir: home.clone(),
            home,
            repo: with_repo.then(|| self.repo.clone()),
            user: "alex".to_string(),
            harness_version: version.map(str::to_string),
        }
    }

    fn list(&self, context: &LaunchContext) -> SourceListing {
        let knowledge = KnowledgeFile::embedded().expect("the embedded file parses");
        ClaudeCodeAdapter::new(
            knowledge,
            SystemRoot::at(self.system.clone()),
            Platform::Linux,
        )
        .list_sources(context)
        .expect("a listing")
    }

    fn write(&self, path: &Path, text: &str) -> PathBuf {
        fs::create_dir_all(path.parent().expect("a parent")).expect("dirs");
        fs::write(path, text).expect("file");
        path.to_path_buf()
    }

    fn user(&self, text: &str) -> PathBuf {
        self.write(&self.scratch.home().join(".claude/settings.json"), text)
    }

    fn managed(&self, which: &str, text: &str) -> PathBuf {
        self.write(&self.system.join(which), text)
    }
}

fn layers(listing: &SourceListing) -> Vec<&str> {
    listing.sources.iter().map(|s| s.layer.as_str()).collect()
}

#[test]
fn a_present_user_file_is_listed_in_effect_with_its_key_names_and_no_value() {
    let rig = Rig::new();
    let path = rig.user(r#"{"model":"super-secret-model","permissions":{}}"#);
    let listing = rig.list(&rig.context(Some(VERSION), true));
    assert_eq!(listing.harness, "claude-code");
    assert_eq!(listing.harness_version.as_deref(), Some(VERSION));
    let knowledge = KnowledgeFile::embedded().expect("embedded");
    assert_eq!(listing.knowledge_version, knowledge.version().to_string());
    assert_eq!(listing.sources.len(), 1, "{:?}", listing.sources);
    let source = &listing.sources[0];
    assert_eq!(source.path, path);
    assert_eq!(source.layer, "user");
    assert_eq!(source.item_id, "claude-code.source-user-settings");
    assert_eq!(source.status, SourceStatus::InEffect);
    assert!(source.size.is_some() && source.modified.is_some());
    match &source.keys {
        KeyNames::Names {
            names,
            classification,
        } => {
            assert_eq!(names, &["model", "permissions"]);
            assert!(
                classification.starts_with("not classified: knowledge"),
                "{classification}"
            );
            assert!(classification.contains("lists no keys"), "{classification}");
            assert!(
                classification.contains(&knowledge.version().to_string()),
                "{classification}"
            );
            assert!(classification.contains(&format!("claude-code {VERSION}")));
        }
        other => panic!("expected names, got {other:?}"),
    }
    assert!(
        !format!("{listing:?}").contains("super-secret-model"),
        "no value appears in a listing"
    );
}

/// Names come back sorted by name, whatever order the file wrote them in.
#[test]
fn key_names_are_sorted_by_name_not_by_file_order() {
    let rig = Rig::new();
    rig.user(r#"{"permissions":{},"model":"m"}"#);
    let listing = rig.list(&rig.context(Some(VERSION), true));
    match &listing.sources[0].keys {
        KeyNames::Names { names, .. } => assert_eq!(names, &["model", "permissions"]),
        other => panic!("expected names, got {other:?}"),
    }
}

#[test]
fn an_invalid_file_is_listed_with_a_reason_and_nothing_aborts() {
    let rig = Rig::new();
    rig.user("{ not json");
    let listing = rig.list(&rig.context(Some(VERSION), true));
    assert_eq!(listing.sources.len(), 1, "{:?}", listing.sources);
    match &listing.sources[0].keys {
        KeyNames::NotRead { reason } => assert!(!reason.is_empty()),
        other => panic!("expected not-read, got {other:?}"),
    }
}

#[test]
fn present_sources_are_listed_highest_layer_first() {
    let rig = Rig::new();
    rig.user("{}");
    rig.managed(LINUX_MANAGED, r#"{"permissions":{}}"#);
    rig.write(&rig.repo.join(".claude/settings.json"), "{}");
    rig.write(&rig.repo.join(".claude/settings.local.json"), "{}");
    let listing = rig.list(&rig.context(Some(VERSION), true));
    assert_eq!(layers(&listing), ["managed", "user", "project", "local"]);
}

#[test]
fn absent_always_present_layers_are_named_with_the_paths_checked() {
    let rig = Rig::new();
    let listing = rig.list(&rig.context(Some(VERSION), true));
    assert!(listing.sources.is_empty());
    let absent: Vec<&str> = listing
        .absent_layers
        .iter()
        .map(|a| a.layer.as_str())
        .collect();
    assert_eq!(absent, ["user", "project", "local"]);
    let user_paths: Vec<&Path> = listing.absent_layers[0]
        .checked
        .iter()
        .map(|c| c.path.as_path())
        .collect();
    assert_eq!(
        user_paths,
        [rig.scratch.home().join(".claude/settings.json")]
    );
    assert!(listing.not_checked.is_empty());
}

/// Each path checked is paired with the knowledge item that states it, so the store can
/// record an absent source under the item it belongs to.
#[test]
fn each_absent_path_comes_with_the_knowledge_item_that_states_it() {
    let rig = Rig::new();
    let listing = rig.list(&rig.context(Some(VERSION), true));
    let pairs: Vec<(String, PathBuf, &str)> = listing
        .absent_layers
        .iter()
        .flat_map(|layer| {
            layer
                .checked
                .iter()
                .map(|c| (layer.layer.clone(), c.path.clone(), c.item_id.as_str()))
        })
        .collect();
    assert_eq!(
        pairs,
        [
            (
                "user".to_string(),
                rig.scratch.home().join(".claude/settings.json"),
                "claude-code.source-user-settings"
            ),
            (
                "project".to_string(),
                rig.repo.join(".claude/settings.json"),
                "claude-code.source-project-settings"
            ),
            (
                "local".to_string(),
                rig.repo.join(".claude/settings.local.json"),
                "claude-code.source-local-settings"
            ),
        ]
    );
}

#[test]
fn managed_cache_and_global_state_are_never_named_when_absent() {
    let rig = Rig::new();
    let listing = rig.list(&rig.context(Some(VERSION), true));
    assert!(
        listing.absent_layers.iter().any(|a| a.layer == "user"),
        "the always-present layers are named, so the rest is a choice and not an empty list"
    );
    for layer in ["managed", "server-managed-cache", "global-state"] {
        assert!(
            listing.absent_layers.iter().all(|a| a.layer != layer),
            "{layer} is not always present"
        );
    }
}

#[test]
fn project_and_local_are_named_only_when_the_repository_resolves() {
    let rig = Rig::new();
    let listing = rig.list(&rig.context(Some(VERSION), false));
    let absent: Vec<&str> = listing
        .absent_layers
        .iter()
        .map(|a| a.layer.as_str())
        .collect();
    assert_eq!(absent, ["user"]);
    assert!(listing.not_checked.is_empty());
}

#[test]
fn a_relative_config_directory_is_not_checked_and_says_why() {
    let rig = Rig::new();
    let mut context = rig.context(Some(VERSION), true);
    context.config_dir = PathBuf::from("relative/claude");
    let listing = rig.list(&context);
    assert_eq!(listing.not_checked.len(), 1);
    assert_eq!(listing.not_checked[0].layer, "user");
    assert!(
        listing.not_checked[0]
            .reason
            .contains("CLAUDE_CONFIG_DIR is not absolute"),
        "{}",
        listing.not_checked[0].reason
    );
    assert!(listing.absent_layers.iter().all(|a| a.layer != "user"));
}

#[test]
fn an_unknown_version_reads_every_status_unknown_and_says_so() {
    let rig = Rig::new();
    rig.user("{}");
    rig.managed(LINUX_MANAGED, r#"{"permissions":{}}"#);
    let listing = rig.list(&rig.context(None, true));
    assert_eq!(listing.harness_version, None);
    assert_eq!(listing.sources.len(), 2, "{:?}", listing.sources);
    for source in &listing.sources {
        match &source.status {
            SourceStatus::Unknown { reason } => {
                assert!(reason.contains("version"), "{}: {reason}", source.layer)
            }
            other => panic!("{}: expected unknown, got {other:?}", source.layer),
        }
    }
}
