# Applying a profile to Claude Code's user settings: what the spine does not yet say

- **Status:** design for review, 2026-11-21. Nothing here is built. A build waits until this design and its plan are reviewed and the operator says go.
- **Author:** the arcaven-ent team's architect seat, from a six-seat design review.
- **Checked at:** usher `origin/main` 4147ad6; Claude Code 2.1.329 (binary strings, read only); Claude Code docs read 2026-11-21: [settings](https://code.claude.com/docs/en/settings#settings-precedence), [settings reference](https://code.claude.com/docs/en/settings-reference), [managed settings](https://code.claude.com/docs/en/managed-settings).
- **Builds on:** the architecture spine (`_bmad-output/planning-artifacts/architecture/architecture-usher-2026-11-08/ARCHITECTURE-SPINE.md`, cited as AD-n), the stories in `_bmad-output/planning-artifacts/epics.md` (cited as story n.m), and the PRD addendum's change-safety table (`_bmad-output/planning-artifacts/prds/prd-usher-2026-11-03/addendum.md` section 7). This design adds deltas to them. It adds no second write path, lock, journal or backup store.
- **Extends:** `_kos/ideas/fleet-settings-profile.md` (the fleet profile idea), charter F2 (correct initial settings), F7 (organization-locked settings) and F10 (settings discovery).
- **Prior art, design only:** cc-switch (farion1231/cc-switch, MIT), read at commit `2c084ec752a985fa77470facc14cad76ee67e8e1`. No code is copied (section 7). Cites written `cc-switch <path>` are under that repository's `src-tauri/src/`.

## 1. Why

An operator who keeps Claude Code auto mode and permission entries the same on several machines edits `~/.claude/settings.json` by hand on each one, and the idea file records that friction in the operator's own words. Applying a profile (story 8.3) to that file removes the hand edit.

The spine already says how usher writes any harness file. This file is a hard case for it, for four reasons the spine and stories do not yet name:

- **Claude Code writes it itself.** `/model` and most `/config` options save to it. Strings in 2.1.329 also show writes from an auto-mode prompt and from the Bedrock and Vertex setup flows.
- **Other tools rewrite it.** cc-switch rewrites it to switch providers.
- **It can hold credentials.** After cc-switch has run, its `env` table can hold an OAuth token, an AWS secret or a Bedrock bearer token: cc-switch manages every `AWS_*` name and `CLAUDE_CODE_OAUTH_TOKEN` with its refresh token (cc-switch `live/floor.rs:30`, `:38-41`). Knowledge already marks `env` as sensitive, but nothing connects that mark to story 6.4's key-level backup, and nothing marks `apiKeyHelper`.
- **An auto-mode array can silently drop the classifier's built-in rules** (D3).

## 2. What the spine already settles

These parts of the earlier draft are dropped, because the spine already decides them and the draft disagreed with it.

| concern | the spine's rule | what the earlier draft had, now dropped |
|---|---|---|
| one writer at a time | AD-2, story 6.1: the journal lock in the state directory | its own per-target lock |
| the write | AD-3, story 6.3: intent, backup, write, read back, outcome | an eight-step protocol of its own |
| a target changed since the proposal | AD-3, AD-16, story 6.3: write nothing and ask for a fresh proposal; a hash that is neither before nor after is a conflict | replan up to three times |
| format, mode, symlinks, temp file and rename | AD-5, story 6.3, including the byte check on permissions | the same rules, restated |
| what is kept to undo a change | AD-7, story 6.4: key-level backup of a mixed file, kept until the person removes it | a 0600 undo record, last five kept |
| a crash part way | AD-3, story 6.5: the crash marker, resolved by the person | a temp file left for the next run |
| usher's own writes | AD-18, story 6.13 | none |
| a value set by hand that the entry would replace | story 8.3: shown as a conflict | none |
| a higher layer that overrides | story 8.1 names it before apply; story 8.4 reports it after | a skip rule (kept as D5, narrowed) |
| observation | AD-9 | a restatement |
| credentials | AD-8, story 6.4 | the same bar, restated |

## 3. Deltas

Each delta names the spine rule or story it lands in. Every Claude Code fact a delta uses is a harness-knowledge item, not an adapter or core constant (AD-6).

**D1. Key-level backup on a file that holds a sensitive key (story 6.4, AD-7, AD-8).**

- **The trigger.** Story 6.4's key-level backup applies to any file that holds a key knowledge marks `sensitive` or credential, not only to files named as mixed. The user settings file qualifies through `env`, which knowledge already marks `sensitive = true` (item `claude-code.key-env`). Marking the whole table covers every name, including the Bedrock, AWS and Foundry bearer names a name list would miss.
- **Not a credential file.** The file is never given story 6.4's first mark, "a file harness knowledge marks as holding credentials", because that mark refuses every write.
- **Outside the mark's version range, fail closed.** The `env` mark covers Claude Code `>=2.1.317, <3.0.0`. For a version outside that range, the file is treated as holding a sensitive key it cannot see, so the apply refuses the write; it is never treated as unmarked.
- **The path bar stays in D2.** `sensitive` stops values being read or stored, but unlike a credential key it does not make AD-8 refuse a key path through `env`. D2's refusal of every `env.*` path does that job, so it is not optional.
- **`apiKeyHelper`.** Knowledge gains one item marking it sensitive, because it names a command that mints credentials. Knowledge has no item for it at 4147ad6.
- **The backup unit for an owned entry is the entry.** A key-level backup of `permissions.allow` would hold the whole array, including other writers' entries, and a rollback that writes it back would erase entries added since. For an entry usher adds, the backup records "absent" and the entry's value. Rollback removes that entry by value, as a new AD-3 transaction, and leaves every other entry alone.
- **AD-8's reading.** The format-preserving editor (AD-5) parses every byte of the file to patch it. This design reads AD-8's "its value is not read" as "never extracted": a sensitive value never leaves the editor's memory, so it never reaches a backup, the journal, a diff, a log or an error. Diffs and errors on this file print owned key paths and line numbers only. The reading is flagged for usher's owners to confirm.
- **The one exception: D4's read.** One `env` name, `CLAUDE_CODE_AUTO_MODE_SERVER`, is read by value, for D4 only. Its value is compared to `1` and then dropped. It is never stored, logged or shown, and it never reaches a backup, the journal, a diff or an error; only the result, on or off, is reported. No other `env` value is read. The exception is flagged for usher's owners with the reading above (section 6).

**D2. What a profile may own here, and how usher knows it owns it (stories 7.1, 6.13, 8.4; AD-6, AD-11, AD-18).**

The Claude Code fragment is native JSON, as AD-11 requires. A profile may own entries in `permissions.allow`, `ask` and `deny`, and in `autoMode` arrays, and nothing else. Knowledge marks which keys a profile may own entries in, and the profile loader reads that mark, so the bar is a knowledge fact (AD-6). Loading refuses, before any file is read, a fragment that sets any other key. The keys it refuses include these:

| path | why never | who writes it |
|---|---|---|
| `env.*`, any name | can carry a credential or a backend selector | cc-switch (`floor.rs:18-41`), Claude Code's setup flows (2.1.329) |
| `apiKeyHelper` | mints credentials | cc-switch (`floor.rs:61`) |
| `model`, `advisorModel` | owned by `/model` and by provider switchers | Claude Code (2.1.329), cc-switch (`floor.rs:68`, `:75`) |
| `permissions.defaultMode`, `permissions.blockReadsOutsideWorkingDirectories` | single values Claude Code writes | Claude Code (2.1.329) |
| `hooks`, `statusLine` | run shell commands; out unless ruling R-C rules them in | none seen |

**Ownership comes from the journal.** An entry is usher's when an AD-3 outcome row records usher adding that value to that key and no later outcome records its removal. Matching on the value alone is not enough, because the same value added by hand would be claimed too. The journal row exists for every write. Story 8.4's adoption record does not: 8.4 writes it only for an entry verified in effect, so an entry left overridden (D5) or inert (D4) would never be usher's. The adoption record keeps provenance and drift only. An entry with no journal row is never removed, and an entry another writer moved or reordered is still found by its value.

**D3. The `$defaults` rule (AD-5, story 6.3).** Every `autoMode` array must hold the literal `"$defaults"`, or its entries replace the classifier's built-in rules (settings reference, `autoMode`; the same instruction is in 2.1.329's strings). The rule is a field on knowledge's `autoMode` items.

- When usher creates an `autoMode` array, it writes `"$defaults"` first.
- It never removes `"$defaults"`.
- When an existing array lacks it, the dry run says so, and the operations on that array are shown unselected; the person may still select them.

**D4. Auto mode turned off (story 8.4, AD-8, AD-17).** cc-switch sets `CLAUDE_CODE_AUTO_MODE_SERVER` on gateways (`floor.rs:104`). 2.1.329's strings say that turning it off leaves "no check of its own to use instead", and that to use auto mode you "set CLAUDE_CODE_AUTO_MODE_SERVER to 1 or remove it, in your shell or in the settings file that sets it".

- Knowledge reads `env` names only, so knowledge gains one item that makes this one name readable by value, from the settings `env` table and from the launch context (AD-17), with its reason and `last_verified` at 2.1.329. A value other than `1` reads as off. This is D1's one exception, with D1's bounds: one name, compared to `1`, never stored, logged or shown.
- The new item is a child of `claude-code.key-env` for the single key path `env.CLAUDE_CODE_AUTO_MODE_SERVER`. For that path only, its readable mark takes the place of the parent's `sensitive` mark; every other child of `env` stays `sensitive`. The `claude-code.key-env` summary changes with it, to name the one exception (U1).
- usher never writes it (D2).
- When it reads as off, usher reports every adopted `autoMode` entry as not in effect, with that reason: story 8.4's "not loaded (with the reason)".

**D5. Under a managed layer (stories 8.1, 8.3, 8.4).** Story 8.3 refuses a write aimed at the managed layer, and story 8.1 names an overriding layer before the person applies. This delta covers one case they leave open: a list entry that a managed lock makes useless.

- **Precedence.** Among managed sources, Claude Code by default uses the highest-ranked one that delivers a policy key, except `env`, which merges per variable ([managed settings](https://code.claude.com/docs/en/managed-settings), "How Claude Code combines managed sources"). Knowledge records first-wins per source; the `env` exception is not recorded, and nothing here needs it, since usher never reads a managed `env` value.
- **Lists merge,** so usher's entries still count, unless managed sets `allowManagedPermissionRulesOnly`. Then the permission operations are shown labelled `inert: managed-only` and unselected by default. The person still decides, as story 8.1 leaves the apply to them.
- **The remote layer:** usher never fetches it, because fetching needs the person's claude.ai credential (AD-8). Knowledge lists its local cache (item `claude-code.source-server-managed-cache`). If the cache exists, usher reads its key paths only and the dry run shows `managed: from cache, <mtime>`, since the settings reference describes startup waiting until the settings are "freshly fetched", so the cache can lag. If it does not exist, the dry run says `managed: unknown` rather than claim a key is free.
- **Managed sources:** read for key paths only, never `env` values, never written.

A codex target needs its own rule for codex's admin layer, `requirements.toml`, before it ships (ruling R-B).

**D6. Two typed outcomes, and no retry (AD-3, AD-18, story 6.15).** AD-3 and story 6.3 already stop a write when the target changed since the proposal. Two things are new.

- **A last check before the rename.** The adapter checks the target's hash again just before AD-5's rename. On a change it writes nothing, records the outcome `aborted-target-changed` with the hash it read, and asks for a fresh proposal. Recording the outcome matters: AD-3's intent and backup rows are already committed, and an intent with no outcome is a crash marker that makes every mode refuse changes.
- **A read-back mismatch** after the rename is recorded as `readback-mismatch` with the hash read, reported, and never retried. Two writers that each reassert their value would overwrite each other without end, so there is no watch mode and no re-apply loop. A `readback-mismatch` outcome is left out of AD-18's own-write match: its hash includes another writer's change, which must still reach the map.
- **The window that remains.** A write by another program between the last check and the rename is lost: the rename replaces it, and the read-back passes because usher's entries hold. cc-switch documents the same window for writers outside its own lock (cc-switch `live/engine.rs:8-10`). U3 adds it to addendum section 7 as a known, accepted row naming `hash_change_before_rename_writes_nothing`, so story 6.15's trace check passes. Both typed outcomes appear in the journal export, so how often each happens can be counted.

**D7. Another writer's presence (story 8.4's report).**

- cc-switch is detected by a `stat` of `~/.cc-switch/` only. usher never opens a file inside it, since its database stores provider keys.
- Present: the report says the tool is present and that keys outside the profile are not usher's.
- Not present: the report says "not found at its default path", never "absent", because cc-switch's data directory can be moved by a stored override (cc-switch `config.rs:314-316`).
- usher never installs, starts or configures it. Whether the report names the tool at all waits on ruling R-E.
- marvel's per-seat `--settings` layer sits above the user layer for marvel seats, and seats still load the user layer, so a value another tool writes into user `env` can reach a seat. That is a marvel property, tracked as ArcavenAE/marvel#1555, outside this design.

## 4. What it never does

- Write a key D2 bars, or any key that harness knowledge does not list as writable (AD-6, AD-8).
- Let a sensitive value out of the editor's memory, or read any `env` value except D4's one exception, which is compared to `1` and never stored, logged or shown (D1, D4).
- Roll back an entry another writer added (D1, D2).
- Fetch the remote managed layer, or write any managed source (D5, story 8.3).
- Retry after a read-back mismatch, or leave an aborted write as a crash marker (D6).
- Open a file inside another writer's data directory (D7).

## 5. Red tests

Every fixture is synthetic: scratch homes and made-up canary tokens. A test that asserts something did not happen carries a positive control in the same run, so it cannot pass against an adapter that writes nothing. The failpoint is a `failpoints` Cargo feature, enabled from the dev-dependencies of the crate whose tests use it, following the existing `test-keys` feature (`crates/usher-knowledge/Cargo.toml:23`); test-only `cfg(test)` code does not reach tests under `tests/`.

- **`crates/usher-knowledge/tests/settings_keys.rs`, extended (U1).** These fail at 4147ad6 because the items do not exist:
  - `api_key_helper_is_sensitive`;
  - `auto_mode_server_is_readable_by_value_with_reason`: also asserts that every other child of `env` stays `sensitive` and that the `claude-code.key-env` summary names the exception;
  - `profile_ownable_keys_are_marked`;
  - `auto_mode_items_carry_the_defaults_rule`.
- **`crates/usher-profiles/tests/profile_fragment.rs` (U2).** The spine places profiles in `usher-profiles` (spine, capability map), which does not exist at 4147ad6. These fail because no fragment loader exists:
  - `fragment_refuses_every_barred_path`: every row of D2's table;
  - `fragment_with_only_list_entries_loads`: the positive control.
- **`crates/usher-adapter-claude-code/tests/apply.rs` (U3).** These fail because no Claude Code write path exists:
  - `file_with_sensitive_key_gets_key_level_backup_and_is_not_a_credential_file`;
  - `user_settings_outside_mark_range_fails_closed`: a Claude Code version outside `>=2.1.317, <3.0.0`, with a positive control at an in-range version;
  - `canary_in_env_never_leaves_the_file`: stdout, stderr, journal, backup store, diff, and a forced parse failure. The canary sits in a second `env` name, with `CLAUDE_CODE_AUTO_MODE_SERVER` also present, so the exception cannot widen to the whole table;
  - `backup_of_owned_entry_holds_only_that_entry`: a canary in another writer's `allow` entry is absent from the backup;
  - `rollback_removes_only_usher_entry`;
  - `adopted_entry_found_after_reorder`;
  - `entry_without_journal_row_is_never_removed`: includes a hand-added entry with the same value as one usher once removed;
  - `entry_written_while_overridden_is_still_removable`;
  - `new_auto_mode_array_starts_with_defaults`;
  - `auto_mode_array_without_defaults_shows_operations_unselected`;
  - `hash_change_before_rename_writes_nothing`: uses the failpoint, and asserts an `aborted-target-changed` outcome and no crash marker;
  - `read_back_mismatch_is_recorded_and_not_retried`;
  - `read_back_mismatch_is_not_an_own_write`.
- **`crates/usher-adapter-claude-code/tests/managed.rs`, extended (U4).** Each test also proposes an unmanaged entry in the same run. These fail because no apply path reads managed locks:
  - `managed_only_permissions_shown_inert_and_unselected`;
  - `no_managed_env_value_is_read`;
  - `managed_from_cache_shows_mtime`;
  - `managed_unknown_without_remote_cache`.
- **`crates/usher-cli/tests/apply_report.rs` (U5).** These fail because no apply report exists:
  - `cc_switch_presence_by_stat_only`: the scratch `.cc-switch/` holds a canary file and the test runs under a read-deny profile for that directory, with a positive control that fails the test, rather than skipping it, when the process can read the canary anyway, as under a root runner;
  - `not_found_at_default_path_wording`;
  - `auto_mode_off_reports_entries_not_in_effect`.

The read-deny profile is `sandbox-exec` on macOS. Its Linux equivalent is unverified, and U5 names it before it lands.

## 6. Rulings this design needs

- **R-A, the go:** whether to build this, after review.
- **R-B, codex:** whether a first slice also applies to `~/.codex/config.toml` for human sessions. The party split 3 to 3. For: marvel's codex seats use a private `CODEX_HOME`, so it reaches human sessions only. Against: the named codex pain is seats stopping at approval prompts, and `requirements.toml` has no managed rule yet. The plan holds the codex ticket until this is ruled.
- **R-C, `hooks` and `statusLine`:** whether a profile may ever own them. The party voted 6 to 0 to keep them out unless ruled in.
- **R-E, vendor terms:** the rule on subscription use outside a vendor's official client. It may decide whether D7's report names cc-switch at all, so U5's wording waits on it.
- **For usher's owners, not a ruling of this plan:** D1's reading of AD-8 ("never extracted"), and D4's one-name exception to `env`'s `sensitive` mark (D1, D4).

## 7. Prior art and license basis

The deltas take four ideas from cc-switch: patch named keys and preserve the rest; stop on a parse failure rather than write an empty document; reread and compare just before the rename; and the documented window for outside writers (cc-switch `live/engine.rs:3-10`). usher departs from it in four places:

- no full-file backup, and a backup unit of one entry (D1);
- an allow-list of list entries (D2);
- on a change before the rename, it writes nothing and asks for a fresh proposal, where cc-switch replans up to three times (cc-switch `mode/operation.rs:33`) (D6);
- it reads the file back after the rename, which cc-switch does not, and on a mismatch it reports and never retries (D6).

No code is copied. cargo-deny checks dependencies, not source copied into usher's crates (`deny.toml:26-29`), so a copied notice would rest on review alone; git sources are denied (`deny.toml:36-38`); and cc-switch's engine depends on other modules of its app (cc-switch `live/engine.rs:12-24`).

## 8. Plan

Flat tickets with dependency edges and no umbrella. "Story n.m" edges are usher's own backlog stories, which this plan does not replace.

| ticket | does | depends on | red tests it turns green |
|---|---|---|---|
| U1 | knowledge items for D1, D2, D3 and D4, including the D4 child item that overrides `env`'s `sensitive` mark for one path, and the `claude-code.key-env` summary that names it | R-A | `settings_keys.rs`: 4 more |
| U2 | fragment refusal for D2 | R-A, U1; story 7.1 | `profile_fragment.rs`: 2 |
| U3 | Claude Code apply deltas: D1, D2's journal ownership, D3, D6, and the addendum row | U1, U2; stories 6.3, 6.4, 6.13, 8.3, 8.4 | `apply.rs`: 13 |
| U4 | D5, the managed rule | U3; story 8.1 | `managed.rs`: 4 more |
| U5 | D4 and D7, the reports | U3; R-E for D7's wording | `apply_report.rs`: 3 |
| U6 | a codex user target, with a `requirements.toml` rule | R-B, U3, U4 | named when R-B is ruled; **waiting** |

Plan check:

- Every ticket but U6 names its tests, and each test sits in the crate the ticket changes: knowledge in `usher-knowledge`, fragment loading in `usher-profiles`, the write path in `usher-adapter-claude-code`, the report in `usher-cli`.
- U2 cannot start before story 7.1 creates `usher-profiles`. U3 cannot start before stories 6.3, 6.4 and 8.3 land, since it changes their write path rather than building one, and its failpoint test needs story 6.3's transaction.
- D6's window becomes a row in addendum section 7 naming its test, so story 6.15's trace check passes. U3 carries that row.
- U6 names no tests, because its scope waits on R-B. It is marked waiting, not hidden.
