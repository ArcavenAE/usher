---
title: 'Claude Code settings inventory (Story 1.5)'
type: 'feature'
created: '2026-11-10'
status: 'in-review'
route: 'dispatch'
baseline_commit: '88d3ad9f4e9a0f1dd64f3a49e7e12e688a762de8'
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/implementation-artifacts/epic-1-context.md']
---

<frozen-after-approval reason="human-owned intent: do not modify unless human renegotiates">

## Intent

**Problem:** A person cannot see which Claude Code settings sources exist on their host, so they cannot tell what could affect a launch before asking what is in effect (FR-43).

**Approach:** Add a Claude Code adapter that finds every settings source in a scratch or real home and under roots the person names, and `usher inventory` that lists each with location, harness, the layer it would belong to and one status. Only derived facts are read: presence, path, size, modified time and top-level key names. This slice stops before the SQLite store and the seat items, which come as their own later stories (director ruling, 2026-11-10).

## Boundaries & Constraints

**Always:** `#![forbid(unsafe_code)]`; failures use `UsherError`; observation changes nothing (AD-9): files are only stat-ed and, for `.json` sources, parsed in memory for key names, never written; no file content, value or credential is printed or kept (AD-8), and `oauthAccount` and every other value are never surfaced; tests use scratch homes only and no fixture carries a real key or token; each fact about a location, layer or precedence comes from harness knowledge, not adapter code (locations and recognized keys are optional fields on 1.3 items, seed sequence 2), and carries the version it assumes (2.1.317); `serde_json =1.0.154` is the one new crate, used only to read top-level key names (settings files are strict JSON, see Design Notes); every found source has status `unknown` with the reason "no environment is mapped" until environments exist; project and local files are looked for only under `--root` directories; the recognized-key list is partial, and the output says so; a source that cannot be read or parsed is listed with a named reason, never a crash and never an empty inventory.

**Never:** open a credential store; run any `claude` command; write to any home; follow symlinks below a source root; discover directories on its own (Story 1.9 owns that); resolve settings in effect or provenance (Story 1.6); add the store (and so `rusqlite`), MCP registrations, packs, fleet-tool config, backend and auth route, or toolchain listing; add a crate not ruled.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| All sources present | scratch home with managed, user, project, local settings, `~/.claude.json`, `remote-settings.json` | each listed once with location, harness, layer, status | N/A |
| Source absent | a known location with no file | not listed as present; listing says it was looked for | N/A |
| Unknown key | `.json` source with a key knowledge does not list for 2.1.317 | key named in the listing, never dropped, marked unrecognized | N/A |
| Unreadable or non-JSON file | permission denied, invalid JSON, a comment in a settings file, or `~/.claude.json` that does not parse | source listed with status unknown and a named reason | no abort |
| `CLAUDE_CONFIG_DIR` set | config dir relocated | user-scope files found under it | N/A |
| Managed directory | `managed-settings.json` plus `managed-settings.d/*.json` | each file its own source, in merge order | non-`.json` and hidden files ignored |
| Symlink root | a source path that is a symlink | followed once at the root, nothing below | N/A |
| Observation | any run | scratch tree digest identical before and after | the invariant test fails the run |

</frozen-after-approval>

## Code Map

- `crates/usher-knowledge/knowledge/harness-knowledge.toml`, `src/schema.rs` -- seed items for Claude Code locations and layers (sequence 2); the optional `locations` field if Question 3 says so
- `crates/usher-adapter-claude-code/` -- new: source discovery, status, key names; reads locations from `KnowledgeFile`
- `crates/usher/src/main.rs`, `crates/usher/tests/cli.rs` -- the `inventory` mode and its output
- `crates/usher-testkit/src/scratch.rs`, `invariant.rs` -- `ScratchHome` (`HOME`, `CLAUDE_CONFIG_DIR`), the observation invariant to reuse
- Research map digests (Claude Code 2.1.317): managed `managed-settings.json`, `managed-settings.d/`, `managed-mcp.json` in `/Library/Application Support/ClaudeCode/` (macOS) and `/etc/claude-code/` (Linux); user `~/.claude/settings.json`; project `.claude/settings.json`; local `.claude/settings.local.json`; `~/.claude.json`; `~/.claude/remote-settings.json`; managed merge order
- Do not change: `crates/usher-core`, `deny.toml`, `.github/workflows/ci.yml`

## Tasks & Acceptance

**Execution:**
- [x] `crates/usher-knowledge` -- optional `locations` and `keys` fields, the Claude Code items, seed sequence 2 -- location and layer facts with references
- [x] `crates/usher-adapter-claude-code/` -- discover, classify, read key names -- the inventory
- [x] `crates/usher` -- `usher inventory [--root DIR]...` -- the command
- [x] tests covering every matrix row, inside scratch homes

**Acceptance Criteria:**
- Given a scratch home with managed, user, project and local settings, `~/.claude.json` and a `remote-settings.json` cache, when I run `usher inventory`, then each source is listed with location, harness, layer and one status, and unrecognized keys are listed
- Given any run, when it ends, then no file in the scratch tree changed and no value or credential appears in the output

## Implementation Notes

- Locations and recognized keys are optional fields (`locations`, `keys`) on 1.3 items of class `paths` and `keys`; the seed is sequence 2. Claude Code now offers `inventory` in its support entry (older versions still offer nothing).
- The assumed version is a constant (2.1.317, the version the map checked) because inventory never runs `claude`; the output says it is not detected.
- `USHER_SYSTEM_ROOT` is a test hook that moves `/etc/claude-code` and the macOS system directory under a scratch root; unset, the real system root is used.
- Sources print highest precedence first (managed, local, project, user), then the harness's own state file; within managed, the cached server copy precedes the files.
- `~/.claude.json` is parsed in memory to reach its key names and the values are dropped; its names are listed but not classified, because the seed's key list describes settings files, not that state file.
- Added to `Cargo.lock`: `serde_json` with `serde`, `itoa` and `zmij`.

## Spec Change Log

## Review Triage Log

## Design Notes

JSONC evidence (director ruling, 2026-11-10: the documented contract is enough, no CLI run). Settings files are strict JSON. Source: https://code.claude.com/docs/en/settings, section "Edit a settings file": "Settings files are strict JSON: a `//` comment or a trailing comma is a syntax error, and Claude Code reports the file as a Settings Error at the next start." So `serde_json` suffices and a settings file with a comment is listed as not valid JSON. For `~/.claude.json` the docs say nothing about comments; it is a file Claude Code writes, and one that does not parse is a "Configuration error" there, so usher reports a parse failure as a named reason.

`CLAUDE_CONFIG_DIR` relocation. Source: https://code.claude.com/docs/en/claude-directory: "If you set `CLAUDE_CONFIG_DIR`, every `~/.claude` path on this page lives under that directory instead. The same applies to `.claude.json`, which otherwise sits at `~/.claude.json`." The remote-settings cache is listed there under `~/.claude/`, so it moves with the config home. The research map was silent on both.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: no diff
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` -- expected: no warnings
- `cargo test --locked --workspace --all-targets` -- expected: all pass
