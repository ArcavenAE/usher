//! Story 1.5, PR D: the seed moves the global state file to the token that follows
//! CLAUDE_CONFIG_DIR, and the account-key item reads the same file by the same string.

use usher_knowledge::{KnowledgeFile, Version};

const GLOBAL_STATE: &str = "claude-code.source-global-state";
const ACCOUNT_KEY: &str = "claude-code.seat-auth-account";
const CACHE: &str = "claude-code.source-server-managed-cache";
const TOKEN: &str = "${GLOBAL_STATE_DIR}";
const VENDOR: &str = "every ~/.claude path on this page lives under that directory";

fn item<'a>(file: &'a KnowledgeFile, id: &str) -> &'a usher_knowledge::Item {
    file.items()
        .iter()
        .find(|item| item.id() == id)
        .unwrap_or_else(|| panic!("{id} is seeded"))
}

#[test]
fn the_global_state_source_and_the_account_key_item_share_one_path_string() {
    let file = KnowledgeFile::embedded().expect("embedded");
    let source = item(&file, GLOBAL_STATE).source().expect("a source");
    let seat = item(&file, ACCOUNT_KEY).seat().expect("a seat item");
    assert_eq!(source.path(), "${GLOBAL_STATE_DIR}/.claude.json");
    assert_eq!(seat.path(), Some("${GLOBAL_STATE_DIR}/.claude.json"));
}

#[test]
fn the_variable_binds_the_global_state_token_with_the_home_directory_as_its_default() {
    let file = KnowledgeFile::embedded().expect("embedded");
    let installed = Version::parse("2.1.317").expect("a version");
    let binding = file
        .binding("claude-code", &installed, TOKEN)
        .expect("a binder for the global state token");
    assert_eq!(binding.variable(), "CLAUDE_CONFIG_DIR");
    assert_eq!(binding.default(), "${HOME}");
    let config = file
        .binding("claude-code", &installed, "${CONFIG_DIR}")
        .expect("the config directory binder stays");
    assert_eq!(config.default(), "${HOME}/.claude");
}

#[test]
fn the_variable_is_listed_once_for_the_environment_key() {
    let file = KnowledgeFile::embedded().expect("embedded");
    let names = file.launch_variables("claude-code");
    assert_eq!(
        names.iter().filter(|n| **n == "CLAUDE_CONFIG_DIR").count(),
        1,
        "{names:?}"
    );
}

#[test]
fn the_global_state_and_cache_summaries_cite_the_vendor_sentence_and_drop_not_stated() {
    let file = KnowledgeFile::embedded().expect("embedded");
    for id in [GLOBAL_STATE, CACHE] {
        let summary = item(&file, id).summary();
        assert!(summary.contains(VENDOR), "{id}: {summary}");
        assert!(!summary.contains("not stated"), "{id}: {summary}");
    }
    assert!(
        item(&file, GLOBAL_STATE)
            .summary()
            .contains("The same applies to .claude.json, which otherwise sits at ~/.claude.json")
    );
}
