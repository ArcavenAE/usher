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

**Always:** `#![forbid(unsafe_code)]`; read only the keys harness knowledge lists as readable for the installed version, and report any other key by name with its value unread (AD-8); never open `oauthAccount` in `~/.claude.json` or any key knowledge marks as credential; report credentials and the auth route by presence only; values inside MCP server definitions marked secret-bearing (`env`, `headers`) are shown by name only; the store is WAL with `synchronous=FULL` and a busy timeout, in an owner-only state directory (mode 0700) resolved from the OS user's account home, not `$HOME`; the store holds derived state only (AD-8, the spine's "Derived state only" row); every failure is a `UsherError`; every status the command prints stamps the harness version and the knowledge version it assumed; paths come from knowledge as tokens (`${HOME}`, `${CONFIG_DIR}`, `${REPO}`, `${USER}`) and resolve per environment; `CLAUDE_CONFIG_DIR` relocates the user home, so a source is never found by `$HOME` alone; the Story 1.2 observation invariant passes for every run (hash of harness file metadata before and after).

**Never:** write, create, touch or chmod anything outside usher's own state directory, except creating missing standard parents of the state directory at 0700; run the harness or any command on its never-call list; read file contents beyond the key names of known JSON settings files and the readable keys of AD-8; store a value of any key knowledge does not mark readable; compute which value is in effect per key or where it came from (Story 1.6); report a source as trusted or untrusted (Story 1.7); read instruction, skill or agent files (Story 1.8); discover launch directories beyond the one named (Story 1.9); add an HTTP client or any network call; persist the knowledge sequence floor (the store lands here, the persistence is a later story and stays in deferred-work); add a crate other than `rusqlite` (feature `bundled`, `=0.41.1`) and `serde_json` (`=1.0.154`), both from the spine Stack table; put the verb in `crates/usher`, which stays mode selection only (spine, Structural Seed).

**Renegotiated 2026-11-15** (operator, relayed by director 01M7GG8BXE, verbatim): "allow Creating ~/.local/state is what Linux tools following the XDG directory spec normally do." The Never line above carries the exception; "standard parents" are the platform parents of usher's own state directory and nothing else.

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

## Rulings (architect read of 2026-11-15, sha256 5194d0ba...5406)

1. **Knowledge item shape: settled.** Spine AD-12 and AD-6 put paths, layers and read rules in harness knowledge. Use a slow-layer `sources` class whose items carry a path pattern in the four tokens, the layer, the precedence rank or exclusivity, and the read rule, each with its own reference and `last_verified`. Raise `schema_version` from 1 to 2, extend the structural check to the new fields, and reject a token outside `${HOME}`, `${CONFIG_DIR}`, `${REPO}`, `${USER}`. The Story 1.4 "Never" line bounded Story 1.4's own diff; it is not a standing freeze.
2. **Crates and dependencies: settled.** `usher-store`, `usher-adapter-claude-code`, `rusqlite` (bundled) and `serde_json` are in the spine's Structural Seed and Stack table. The PR's own `cargo deny check` is the evidence; a refusal is fixed by a reviewed `deny.toml` change in that PR, never an ignore.
3. **Where the verb lives: settled, against the first draft.** `usher inventory` goes in a new `usher-cli` driving adapter; `crates/usher` stays mode selection and hands `usher <command>` to it. The port traits (the source-listing part of `HarnessAdapter`, AD-6, and a store port) go in `usher-core`; the launch environment's key comes from AD-17's one `usher-core` function; `usher-adapter-claude-code` and `usher-store` implement the ports. Dependencies point inward, and no adapter depends on another adapter.
4. **Status vocabulary: partly settled by PRD FR-43.** in effect, unread and unknown are as proposed. The code takes a set of environments, not one, so Story 1.9 adds environments without a rewrite: in effect means in effect for at least one mapped environment, unread means no mapped environment reads it. Two points are open and go to the operator as Desk cards, and neither blocks A or B: how a managed source hidden by a higher first-wins source is reported (`shadowed`), and whether and how missing sources are shown.

## Rulings applied (PR B, architect ruling 01M7GFSGTF and answer 01M7GG12SP, 2026-11-15)

The frozen Boundaries above stand as written, except the Never line, which carries the exception in the Renegotiated note. These rulings settle what they leave to PR B:

- The store takes the state directory as an explicit path. The resolver from the OS user's account home, not `$HOME`, lands in PR C with a safe-API crate. The directory paths are the spine's AD-1: macOS `<account home>/Library/Application Support/usher`, Linux `<account home>/.local/state/usher`, database `usher.db`, no `XDG_*` or `HOME` override.
- The open creates the missing standard parents of the state directory at mode 0700, then the leaf at 0700, each in the mkdir call itself, never created and then chmodded; an existing parent is never changed, and nothing else outside the state directory is created. This follows the operator's ruling of 2026-11-15 on the frozen Never line (see the Renegotiated note in the block; architect answer 01M7GG45RB had offered a refusal fallback, dropped when the operator allowed creating them). "Standard parents" are exactly the components of the platform suffix of the state directory, `.local` and `.local/state` on Linux and `Library` and `Library/Application Support` on macOS, created only when the directory above the suffix exists; any other missing parent, or a suffix under a missing base, is refused with the path and the `mkdir` command and nothing is created (architect confirmation 01M7GGDFRH). The database file is 0600, and `-wal` and `-shm` come out owner-only.
- An existing state directory with group or other bits, or a symlink, is refused and never changed. The check for an owner that is not the current uid is deferred to PR C (deferred-work.md).
- Standard parents are the components of either platform suffix, .local/state/usher or Library/Application Support/usher, accepted on every platform. They are created at 0700 only when the directory above the suffix exists; the platform choice belongs to the resolver in Story 1.5 C (architect, 01M7GGG2C2; operator ruling 2026-11-15 on missing standard parents).
- `inventory_run` and `inventory_source` hold derived state only. Rows are inserted and never changed: the API has no update or delete, and triggers ABORT any UPDATE or DELETE on both tables.
- `user_version` is 1; a newer version is refused, and so is a non-empty file at version 0. There is no migration framework.

## One story, four chained PRs

The parts share the workspace `Cargo.toml` and `sprint-status.yaml`, so they are one chain (rule d), each a draft, each red-first, each based on the previous head.

| PR | Content | Acceptance carried | Based on |
|---|---|---|---|
| A | knowledge: the `sources` class, schema_version 2, the check, the Claude Code seed entries with references and dates | its own matrix rows only (bad token, missing reference) | main |
| B | `usher-store`: open from the account home, mode 0700, WAL, `synchronous=FULL`, busy timeout, inventory tables, insert-only, a test that greps every table for content | AC 2 | A |
| C | `usher-core` ports, `usher-adapter-claude-code`, `usher-cli` `inventory`, the observation invariant run | AC 1 | B |
| D | the seat around the harness: MCP registrations (env and headers by name), packs, fleet-tool client config, backend and auth route by presence, toolchain | AC 3 | C |

C waits on the two cards, since C is where statuses print. The story moves to done when D merges, and sprint status says so in D.

## Code Map

- PR A: `crates/usher-knowledge/` -- the `sources` class, `schema_version` 2, the check, the Claude Code seed entries with references and dates
- PR B: `crates/usher-store/` -- new: opens the store at a state directory it is given (macOS `<account home>/Library/Application Support/usher`, Linux `<account home>/.local/state/usher`, file `usher.db`, no `XDG_*` or `HOME` override; the spine's AD-1 says so), creates `inventory_run` and `inventory_source`, inserts runs; no update or delete of inventory rows, enforced by the API and by ABORT triggers on both tables
- PR C0: `crates/usher-knowledge/` -- `schema_version` 3: the `keys` field on class `keys` items, `always_present` on sources, `version_from` on `paths` items, and the seed items that use them
- PR C: `crates/usher-core/` -- the ports (source listing, store) and the environment key function (AD-17); `crates/usher-adapter-claude-code/` -- new: resolves paths from knowledge, reads known JSON for key names only, and reads the harness version from disk by the `version_from` rule in knowledge (read the launcher link once, match its target to the layout, and read the version as unknown, with the reason, when the launcher is not a link, the target does not match, or the first `claude` on PATH is not the knowledge launcher; the harness is never run); `crates/usher-cli/` -- new: the `inventory` verb; `crates/usher/src/main.rs` -- hands the command to `usher-cli`, no logic of its own
- PR D: the seat inventory, in the adapter and the store
- `crates/usher-testkit/` -- reuse `ScratchHome` and the invariant; add Claude Code fixtures under the scratch home as the PRs need them
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

One lookup feeds the environment key (AD-17). `environment_key` keeps exactly the variables its caller passes and filters none, so every caller takes the list from one `usher-knowledge` function, `KnowledgeFile::launch_variables(harness)`, read through `Knowledge::current()`. It returns the sorted, deduplicated `variable` names knowledge lists for the harness across every version range, so an environment's key does not move when the harness version is unknown or changes. The caller passes only the variables that are set and non-empty, so a knowledge update that names a new variable changes no key where that variable is unset. Which variable binds a path token, and its default, is a separate, version-scoped lookup, `KnowledgeFile::binding(harness, installed, token)`, and never feeds the key. No caller keeps its own list.

The "Credential key" row of the matrix is closed in two parts. PR C carries the half that the key is never opened: an `oauthAccount` value, and any key name nested under it, appears in neither the output nor the store. The half that the auth route reads present belongs to PR D with the rest of the seat around the harness (acceptance criterion 3), so the row is not closed until D lands.

The invariant test runs the command against a scratch home with the real harness homes guarded, as Story 1.2 built it. The `deferred-work.md` note that the watched set may miss `~/.claude.json` is settled here, because this is the first story that reads it.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: no diff
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` -- expected: no warnings
- `cargo test --locked --workspace --all-targets` -- expected: all pass
- `cargo deny check` (CI only) -- expected: green
