---
title: 'Claude Code settings inventory (Story 1.5)'
type: 'feature'
created: '2026-11-15'
status: 'draft'
route: 'dispatch'
baseline_commit: '88d3ad9f4e9a0f1dd64f3a49e7e12e688a762de8'
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/implementation-artifacts/epic-1-context.md']
---

<frozen-after-approval reason="human-owned intent: do not modify unless human renegotiates">

## Intent

**Problem:** A person cannot see which Claude Code settings sources exist on their host, or what else makes up their seat, before asking what is in effect (FR-43). Nothing reads a harness yet, and there is no store.

**Approach:** Add `usher inventory`. A new crate `usher-adapter-claude-code` lists every Claude Code settings source for a launch environment (home, config directory, repository) from the paths, layers and read rules that harness knowledge states, never from adapter code. A new crate `usher-store` opens the one SQLite store (AD-7) and records each source as derived state only: presence, path, size, modified time and key names. The command prints each source with its location, harness, the layer it would belong to and one status: in effect, shadowed, unread or unknown. Keys the knowledge does not recognize for the version are listed by name, never dropped. The seat around the harness (MCP registrations, packs, fleet-tool client config, the backend and auth route as presence only, the toolchain) is listed too.

## Boundaries & Constraints

**Always:** `#![forbid(unsafe_code)]`; read only the keys harness knowledge lists as readable for the installed version, and report any other key by name with its value unread (AD-8); never open `oauthAccount` in `~/.claude.json` or any key knowledge marks as credential; report credentials and the auth route by presence only; values inside MCP server definitions marked secret-bearing (`env`, `headers`) are shown by name only; the store is WAL with `synchronous=FULL` and a busy timeout, in an owner-only state directory (mode 0700) resolved from the OS user's account home, not `$HOME`; the store holds derived state only (AD-8, the spine's "Derived state only" row); every failure is a `UsherError`; every status the command prints stamps the harness version and the knowledge version it assumed; paths come from knowledge as tokens (`${HOME}`, `${CONFIG_DIR}`, `${REPO}`) and resolve per environment; `CLAUDE_CONFIG_DIR` relocates the user home, so a source is never found by `$HOME` alone; the Story 1.2 observation invariant passes for every run (hash of harness file metadata before and after).

**Never:** write, create, touch or chmod anything outside usher's own state directory; run the harness or any command on its never-call list; read file contents beyond the key names of known JSON settings files and the readable keys of AD-8; store a value of any key knowledge does not mark readable; compute which value is in effect per key or where it came from (Story 1.6); report a source as trusted or untrusted (Story 1.7); read instruction, skill or agent files (Story 1.8); discover launch directories beyond the one named (Story 1.9); add an HTTP client or any network call; persist the knowledge sequence floor (the store lands here, the persistence is a later story and stays in deferred-work); add a crate other than `rusqlite` (feature `bundled`, `=0.41.1`) and `serde_json` (`=1.0.154`), both from the spine Stack table.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| All sources | scratch home with managed, user, project and local settings, `~/.claude.json` and a `remote-settings.json` cache | each source listed with location, harness, layer and one status | N/A |
| Unrecognized keys | a settings file with a key knowledge does not list for the version | key listed by name under its source, value unread | N/A |
| Config directory moved | `CLAUDE_CONFIG_DIR` set to another directory | user layer is found there, not under `$HOME/.claude` | N/A |
| Managed first-wins | two managed sources present | the higher-ranked one is in effect, the other shadowed | N/A |
| Present but not read | a settings file in a place the knowledge says the version does not read | status unread, with the knowledge item id | N/A |
| Version unknown | no `claude` found, or a version outside every knowledge range | statuses read unknown, never guessed; the inventory still lists what exists | the knowledge-version stamp says not known |
| Absent source | a known source path with no file | listed as absent with its layer, or omitted per the display rule in Design Notes | N/A |
| Credential key | `oauthAccount` present in `~/.claude.json` | the key is not opened; the auth route reads present | N/A |
| MCP definition | a registration with `env` or `headers` | name and the key names of `env` and `headers` only | N/A |
| Store | first run, then a second run | first creates the store at mode 0700 with WAL and `synchronous=FULL`; second inserts a new inventory run and rewrites nothing | `UsherError` if the state directory is not owner-only or the store cannot be opened |
| No content stored | any run | the store holds no file content and no credential value, shown by a test that greps every table | N/A |
| Observation | any run against a scratch home | the Story 1.2 invariant holds for the real harness homes | the invariant failure names the changed path |
| Malformed settings file | invalid JSON | source listed with status unknown and the parse error text without file content | N/A |

</frozen-after-approval>

## Rulings needed before the build

These are decisions the spec relies on and does not make:

1. **Knowledge item shape (Story 1.3).** Sources, their layers, path tokens and the read rule per version are new harness facts. They need either a new item class (`sources`) in the knowledge schema or a new table. This changes the Story 1.3 shape and the embedded seed file, which the Story 1.4 boundary froze.
2. **The new crates.** `usher-adapter-claude-code` and `usher-store` follow the Epic 1 naming rule; `rusqlite` and `serde_json` are in the spine's Stack table. `cargo deny` may need license or advisory entries for `rusqlite` with `bundled`.
3. **What `usher` becomes.** `crates/usher` is mode selection only today. `usher inventory` is its first real command.
4. **Status vocabulary.** The four statuses are applied to a source, not to a key (per-key is Story 1.6). The mapping of read rules to statuses in Design Notes is a proposal.

## Code Map

- `crates/usher-knowledge/` -- new item class for sources; embedded seed gains the Claude Code sources with references and dates (needs ruling 1)
- `crates/usher-adapter-claude-code/` -- new: resolves paths from knowledge, reads known JSON for key names only, returns sources and the seat inventory
- `crates/usher-store/` -- new: opens the store, creates the inventory tables, inserts runs; no update or delete of inventory rows
- `crates/usher/src/main.rs` -- the `inventory` mode
- `crates/usher-testkit/` -- reuse `ScratchHome` and the invariant; add Claude Code fixtures under the scratch home
- `_bmad-output/implementation-artifacts/sprint-status.yaml` -- 1-4 to `done` (merged as #26), 1-5 to `ready-for-dev`

## Tasks & Acceptance

**Execution:**
- [ ] reds first: tests for every matrix row, shown failing on assertions, reported before any implementation
- [ ] knowledge: the sources item class, the seed entries, the check
- [ ] store crate: open, pragmas, owner-only directory, tables, insert-only inventory
- [ ] adapter crate: resolve, read key names, classify, seat inventory
- [ ] `usher inventory`: the command and its text output
- [ ] sprint status: 1-4 done citing #26; 1-5 moves with the story

**Acceptance Criteria:**
- Given a scratch home with managed, user, project and local settings files, `~/.claude.json` and a `remote-settings.json` cache, when I run `usher inventory`, then each source is listed with location, harness, layer and one status, and keys usher does not recognize for the version are listed, never dropped
- Given any source, when it is recorded in usher's store, then only derived state is kept: presence, path, size, modified time and key names; no file content and no credential value
- Given the seat around the harness, when the inventory runs, then MCP registrations, packs, fleet-tool client config, the backend and auth route (presence only) and the toolchain are listed too, and the Story 1.2 invariant test passes for the run

## Design Notes

Proposed status rule (ruling 4): in effect, a present source of a layer the knowledge says the version reads; shadowed, a present source that an exclusive higher source hides (managed sources are first-wins); unread, a present source in a place the knowledge says the version does not read; unknown, any source whose version or format the knowledge cannot place. Absent sources are shown only for the layers knowledge says always exist on the platform, so the listing answers "what exists" without padding.

The invariant test runs the command against a scratch home with the real harness homes guarded, as Story 1.2 built it. The `deferred-work.md` note that the watched set may miss `~/.claude.json` is settled here, because this is the first story that reads it.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: no diff
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` -- expected: no warnings
- `cargo test --locked --workspace --all-targets` -- expected: all pass
- `cargo deny check` (CI only) -- expected: green
