---
title: 'Claude Code settings inventory (Story 1.5)'
type: 'feature'
created: '2026-11-10'
status: 'draft'
route: 'dispatch'
baseline_commit: '88d3ad9f4e9a0f1dd64f3a49e7e12e688a762de8'
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/implementation-artifacts/epic-1-context.md']
---

<frozen-after-approval reason="human-owned intent: do not modify unless human renegotiates">

## Intent

**Problem:** A person cannot see which Claude Code settings sources exist on their host, so they cannot tell what could affect a launch before asking what is in effect (FR-43).

**Approach:** Add a Claude Code adapter that finds every settings source in a scratch or real home and under roots the person names, and `usher inventory` that lists each with location, harness, the layer it would belong to and one status. Only derived facts are read: presence, path, size, modified time and top-level key names. This slice stops before the SQLite store and the seat items (Open Questions 6 and 7).

## Boundaries & Constraints

**Always:** `#![forbid(unsafe_code)]`; failures use `UsherError`; observation changes nothing (AD-9): files are only stat-ed and, for `.json` sources, parsed in memory for key names, never written; no file content, value or credential is printed or kept (AD-8), and `oauthAccount` and every other value are never surfaced; tests use scratch homes only and no fixture carries a real key or token; each fact about a location, layer or precedence comes from harness knowledge, not adapter code, and carries the version it assumes (2.1.317).

**Never:** open a credential store; run any `claude` command; write to any home; follow symlinks below a source root; discover directories on its own (Story 1.9 owns that); resolve settings in effect or provenance (Story 1.6); add the store, MCP registrations, packs, fleet-tool config, backend and auth route, or toolchain listing; add a crate not ruled.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| All sources present | scratch home with managed, user, project, local settings, `~/.claude.json`, `remote-settings.json` | each listed once with location, harness, layer, status | N/A |
| Source absent | a known location with no file | not listed as present; listing says it was looked for | N/A |
| Unknown key | `.json` source with a key knowledge does not list for 2.1.317 | key named in the listing, never dropped, marked unrecognized | N/A |
| Unreadable or non-JSON file | permission denied or invalid JSON | source listed with status unknown and the reason | no abort |
| `CLAUDE_CONFIG_DIR` set | config dir relocated | user-scope files found under it | N/A |
| Managed directory | `managed-settings.json` plus `managed-settings.d/*.json` | each file its own source, in merge order | non-`.json` and hidden files ignored |
| Symlink root | a source path that is a symlink | followed once at the root, nothing below | N/A |
| Observation | any run | scratch tree digest identical before and after | the invariant test fails the run |

</frozen-after-approval>

## Open Questions

1. **Status with no mapped environment.** FR-43 defines in effect, shadowed and unread by mapped environments, and none exist until Story 1.9 (discovery) and Story 1.14 (labels). Options: every found source reads `unknown` with the reason "no environment is mapped" (honest; later stories refine) / `in effect` for user-scope sources of the OS user (a claim no test can check yet). Recommended: all `unknown`.
2. **Where project and local files are looked for.** Options: only under roots passed as `--root <dir>` (no discovery) / also the current directory. Recommended: `--root` only, so the command never scans on its own.
3. **Where the location facts live.** AD-6 puts them in harness knowledge, but a 1.3 item holds only text. Options: add an optional structured `locations` list to items of class `paths` (additive schema change, seed sequence 2) / keep path templates in the adapter, each pinned by a test to a knowledge item id. Recommended: the additive schema field.
4. **Known keys.** Unrecognized keys need a per-version list; the research map cites only some. Options: seed the keys the map names and mark all others unrecognized (over-reports until a research story fills the list) / list every key as recognized-unknown and skip the flag. Recommended: the first.
5. **JSON parser.** Key names need parsing. Options: `serde_json` 1.0.154 (spine Stack table) / `jsonc-parser` 0.34.0 (spine, tolerates comments). Recommended: `serde_json`; key names only, values dropped after parse.
6. **Split.** The story has three acceptance blocks. Options: this slice as Story 1.5, the store (rusqlite, spine Stack table) and the seat items as two later stories / one story. Recommended: split.
7. **`rusqlite`.** Needed only by the store; facts for the ruling go to the supervisor with this spec.

## Code Map

- `crates/usher-knowledge/knowledge/harness-knowledge.toml`, `src/schema.rs` -- seed items for Claude Code locations and layers (sequence 2); the optional `locations` field if Question 3 says so
- `crates/usher-adapter-claude-code/` -- new: source discovery, status, key names; reads locations from `KnowledgeFile`
- `crates/usher/src/main.rs`, `crates/usher/tests/cli.rs` -- the `inventory` mode and its output
- `crates/usher-testkit/src/scratch.rs`, `invariant.rs` -- `ScratchHome` (`HOME`, `CLAUDE_CONFIG_DIR`), the observation invariant to reuse
- Research map digests (Claude Code 2.1.317): managed `managed-settings.json`, `managed-settings.d/`, `managed-mcp.json` in `/Library/Application Support/ClaudeCode/` (macOS) and `/etc/claude-code/` (Linux); user `~/.claude/settings.json`; project `.claude/settings.json`; local `.claude/settings.local.json`; `~/.claude.json`; `~/.claude/remote-settings.json`; managed merge order
- Do not change: `crates/usher-core`, `deny.toml`, `.github/workflows/ci.yml`

## Tasks & Acceptance

**Execution:**
- [ ] `crates/usher-knowledge` -- Claude Code location and layer facts, with references -- per Question 3
- [ ] `crates/usher-adapter-claude-code/` -- discover, classify, read key names -- the inventory
- [ ] `crates/usher` -- `usher inventory [--root DIR]...` -- the command
- [ ] tests covering every matrix row, inside scratch homes

**Acceptance Criteria:**
- Given a scratch home with managed, user, project and local settings, `~/.claude.json` and a `remote-settings.json` cache, when I run `usher inventory`, then each source is listed with location, harness, layer and one status, and unrecognized keys are listed
- Given any run, when it ends, then no file in the scratch tree changed and no value or credential appears in the output

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: no diff
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` -- expected: no warnings
- `cargo test --locked --workspace --all-targets` -- expected: all pass
