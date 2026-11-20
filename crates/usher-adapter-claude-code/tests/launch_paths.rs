//! Story 1.5, PR D: the starting-directory local settings file, and the global state
//! file that follows the directory CLAUDE_CONFIG_DIR names. Public API, a scratch home
//! and a scratch system root.

use std::fs;
use std::path::{Path, PathBuf};

use usher_adapter_claude_code::ClaudeCodeAdapter;
use usher_core::{LaunchContext, SourceLister, SourceListing, SystemRoot, environment_key};
use usher_knowledge::{KnowledgeFile, Platform};
use usher_testkit::ScratchHome;

const VERSION: &str = "2.1.317";
const STARTING_DIR_ITEM: &str = "claude-code.source-local-settings-starting-dir";
const ROOT_LOCAL_ITEM: &str = "claude-code.source-local-settings";
const GLOBAL_STATE_ITEM: &str = "claude-code.source-global-state";

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
        fs::create_dir_all(repo.join("pkg/inner")).expect("repo");
        Rig {
            scratch,
            system,
            repo,
        }
    }

    /// A launch from `launch_dir`, with the global state directory the caller names.
    fn context(&self, launch_dir: &Path, global_state_dir: PathBuf) -> LaunchContext {
        let home = self.scratch.home().to_path_buf();
        let key = environment_key("host", "alex", "claude-code", launch_dir, &[])
            .expect("an absolute directory");
        LaunchContext {
            key,
            config_dir: home.join(".claude"),
            global_state_dir,
            set_variables: Default::default(),
            home,
            repo: Some(self.repo.clone()),
            user: "alex".to_string(),
            harness_version: Some(VERSION.to_string()),
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
}

fn item_ids(listing: &SourceListing) -> Vec<&str> {
    listing.sources.iter().map(|s| s.item_id.as_str()).collect()
}

#[test]
fn a_subdirectory_launch_with_both_local_files_lists_both_as_local_sources() {
    let rig = Rig::new();
    let launch = rig.repo.join("pkg");
    let root = rig.write(&rig.repo.join(".claude/settings.local.json"), r#"{"a":1}"#);
    let start = rig.write(&launch.join(".claude/settings.local.json"), r#"{"b":2}"#);
    let home = rig.scratch.home().to_path_buf();
    let listing = rig.list(&rig.context(&launch, home));
    let locals: Vec<_> = listing
        .sources
        .iter()
        .filter(|s| s.layer == "local")
        .collect();
    assert_eq!(locals.len(), 2, "{:?}", listing.sources);
    let paths: Vec<&Path> = locals.iter().map(|s| s.path.as_path()).collect();
    assert!(paths.contains(&root.as_path()) && paths.contains(&start.as_path()));
    let ids = item_ids(&listing);
    assert!(ids.contains(&ROOT_LOCAL_ITEM) && ids.contains(&STARTING_DIR_ITEM));
}

#[test]
fn a_launch_from_the_repository_root_lists_the_local_file_once() {
    let rig = Rig::new();
    let root = rig.write(&rig.repo.join(".claude/settings.local.json"), "{}");
    let home = rig.scratch.home().to_path_buf();
    let listing = rig.list(&rig.context(&rig.repo, home));
    let locals: Vec<_> = listing
        .sources
        .iter()
        .filter(|s| s.layer == "local")
        .collect();
    assert_eq!(locals.len(), 1, "{:?}", listing.sources);
    assert_eq!(locals[0].path, root);
    for absent in &listing.absent_layers {
        for checked in &absent.checked {
            assert_ne!(
                checked.item_id, STARTING_DIR_ITEM,
                "the starting-directory item is not a second row when the paths are equal"
            );
        }
    }
}

#[test]
fn a_starting_directory_file_alone_is_listed_and_an_absent_one_is_not_reported_as_missing() {
    let rig = Rig::new();
    let launch = rig.repo.join("pkg");
    let start = rig.write(&launch.join(".claude/settings.local.json"), "{}");
    let home = rig.scratch.home().to_path_buf();
    let listing = rig.list(&rig.context(&launch, home.clone()));
    assert!(
        listing.sources.iter().any(|s| s.path == start),
        "{listing:?}"
    );
    fs::remove_file(&start).expect("remove");
    let listing = rig.list(&rig.context(&launch, home));
    let reports_starting_dir = listing
        .absent_layers
        .iter()
        .flat_map(|layer| layer.checked.iter())
        .any(|checked| checked.item_id == STARTING_DIR_ITEM);
    assert!(
        !reports_starting_dir,
        "an item that is not always present adds no no-file row: {listing:?}"
    );
}

#[test]
fn the_starting_directory_item_quotes_the_vendor_sentence() {
    let knowledge = KnowledgeFile::embedded().expect("embedded");
    let item = knowledge
        .items()
        .iter()
        .find(|item| item.id() == STARTING_DIR_ITEM)
        .expect("the starting-directory item is seeded");
    assert!(
        item.summary().contains(
            "Before v2.1.211, Claude Code kept the file in the starting directory. It still reads a file an earlier version left there alongside the root file; where both set the same key, the root's value applies"
        ),
        "{}",
        item.summary()
    );
    let source = item.source().expect("a source");
    assert_eq!(source.path(), "${LAUNCH_DIR}/.claude/settings.local.json");
    assert!(!source.always_present());
}

#[test]
fn the_global_state_file_is_read_from_the_directory_the_context_names() {
    let rig = Rig::new();
    let moved = rig.scratch.root().join("state");
    let at_home = rig.write(&rig.scratch.home().join(".claude.json"), "{}");
    let in_moved = rig.write(&moved.join(".claude.json"), "{}");
    let listing = rig.list(&rig.context(&rig.repo, moved));
    let global: Vec<_> = listing
        .sources
        .iter()
        .filter(|s| s.item_id == GLOBAL_STATE_ITEM)
        .collect();
    assert_eq!(global.len(), 1, "{:?}", listing.sources);
    assert_eq!(global[0].path, in_moved);
    assert!(
        listing.sources.iter().all(|s| s.path != at_home),
        "the file under the home directory is not read: {listing:?}"
    );
}

#[test]
fn a_relative_global_state_directory_is_not_checked_and_says_why() {
    let rig = Rig::new();
    rig.write(&rig.scratch.home().join(".claude.json"), "{}");
    let listing = rig.list(&rig.context(&rig.repo, PathBuf::from("relative/state")));
    let not_checked: Vec<_> = listing
        .not_checked
        .iter()
        .filter(|layer| layer.layer == "global-state")
        .collect();
    assert_eq!(not_checked.len(), 1, "{:?}", listing.not_checked);
    assert_eq!(not_checked[0].reason, "CLAUDE_CONFIG_DIR is not absolute");
    assert!(
        listing
            .sources
            .iter()
            .all(|s| s.item_id != GLOBAL_STATE_ITEM),
        "a file that was not checked is not listed: {listing:?}"
    );
}
