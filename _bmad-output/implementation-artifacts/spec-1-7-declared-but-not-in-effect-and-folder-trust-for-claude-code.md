---
title: 'Declared but not in effect, and folder trust, for Claude Code (Story 1.7)'
type: 'feature'
created: '2026-11-23'
status: 'draft'
route: 'dispatch'
baseline_commit: '58fb3ddda3286fecbb2794806e854f68745a1407'
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/implementation-artifacts/epic-1-context.md']
---

<frozen-after-approval reason="human-owned intent: do not modify unless human renegotiates">

## Intent

**Problem:** A Claude Code setting that a file declares but the harness does not apply can still look like one that works. The settings map from Story 1.6 already reports four of the addendum section 1.1 FR-2 cases that way, but two cases have no reading yet (`defaultMode` set to `auto` or `bypassPermissions` in a project or local file, and an entry `claude doctor` rejects), OTEL settings have no knowledge item, and nothing reports whether a directory is trusted, which decides whether its project layers load at all (FR-2, FR-6, UJ-1 edge case).

**Approach:** Close the FR-2 cases for Claude Code so that each is reported as declared and not in effect, never as in effect, with the reason it rests on. Then, if the operator rules that usher may read a trust record (open question Q-d), add a trust reading per repo root from the harness's own record, read by key only and never written, and report a directory whose project layers do not load because it is untrusted with that reason. How each case is represented is open (Q-a to Q-f below) and is not decided by this spec.

## Boundaries & Constraints

**Always:** every Story 1.5 and 1.6 boundary holds unchanged: `#![forbid(unsafe_code)]`; read only the keys harness knowledge lists as readable for the installed version, and report any other key by name with its value unread (AD-8); harness facts (which layer ignores which key or value, which key holds trust) come from harness knowledge, never adapter code; every reading stamps the harness version and the knowledge version it assumed; paths come from knowledge tokens, so the global state file follows `CLAUDE_CONFIG_DIR` through its binder and is never found by `$HOME` alone; every failure is a `UsherError`; the Story 1.2 observation invariant passes for every run.

**Never:** report any FR-2 case as in effect; run the harness or `claude doctor` (an entry doctor rejects is decided from knowledge or not at all); write, create or change a trust record; read any key of `~/.claude.json` other than those knowledge lists, and never `oauthAccount`; read a trust record before Q-d is ruled; read instruction, skill or agent files (Story 1.8); discover launch directories beyond the one named (Story 1.9); add a crate; bump the knowledge schema or the store version except where an open question below is ruled to need it.

## I/O & Edge-Case Matrix

The first four rows are green on main and are pinned, not new. The rest wait on the open question named in the last column.

| Scenario | Input / State | Expected Output / Behavior | Status |
|----------|--------------|---------------------------|--------|
| Skipped managed source | two managed sources, first-source-wins | the lower one is a skipped layer with the knowledge reason; none of its keys reads in effect | pinned on main |
| Environment override | a key's override variable present | the key reads overridden by environment; the note holds the variable name only | pinned on main |
| `autoMode` in a project file | `autoMode.allow` in the project and user files | project row unread with the knowledge item id; the value comes from the user file | pinned on main |
| `autoMode` in a local file | `autoMode.allow` in the local and user files | local row unread with the knowledge item id | new pin, the only one for these cases |
| Server-managed values | a `remote-settings.json` cache present | no rows from the cache; the not-observed set names server-managed values | pinned on main |
| `defaultMode` auto or bypass | `permissions.defaultMode` set to `auto` or `bypassPermissions` in a project or local file | declared and not in effect, with the knowledge reason; reads not decided today | open, Q-a |
| Entry doctor rejects | a settings entry the vendor's validator rejects | declared and not in effect, with the rule it breaks | open, Q-b |
| OTEL settings in a project or local file | an OTEL setting in a project or local file | declared and not in effect | open, Q-f |
| Trusted repo root | `projects[<root>].hasTrustDialogAccepted` true in the global state file | the root reads trusted | open, Q-c and Q-d |
| Untrusted repo root | the flag false | the root reads not trusted, and its project layers are reported as not loading for that reason | open, Q-c and Q-d |
| No trust record | no `projects[<root>]` entry, or no flag | a state that says no record exists, never trusted | open, Q-c and Q-d |
| Global state file unreadable | invalid JSON, or the file absent | trust is not decided, with the reason; the reason never quotes the file's bytes | open, Q-c and Q-d |
| Observation | any run against a scratch home | the Story 1.2 invariant holds, the global state file included | every part |

</frozen-after-approval>

## Open questions (operator rulings pending)

Every question here is open. A proposal is the builder's, recorded beside the question as context, and is not an answer. Red Gate does not start on a part until the questions that part depends on are ruled.

**Q-a. How "declared and not in effect" is represented for a value a layer ignores.** On main, `permissions.defaultMode` is one knowledge item with merge `own-rule` and scopes `any`, so every layer's value reads `NotDecided`. The harness ignores only the values `auto` and `bypassPermissions`, and only in the project and local layers, so the fact depends on the value, which no knowledge field states today (schema 5).
- Option 1: reuse `Unread` with a note naming the knowledge item, as `autoMode` in a project file does now. No store change.
- Option 2: a new reading status. The store's check constraint lists the statuses, so this raises the store `user_version` from 3 to 4.
- Either way, knowledge needs a field for values a layer ignores, which raises the knowledge schema from 5 to 6.
- Proposal (builder): a schema 6 field for values a layer ignores, seeded for `permissions.defaultMode` only, landed as A0 (knowledge) and then A (reader).

**Q-b. An entry `claude doctor` rejects.** Usher never runs the harness or its doctor, so the case needs a validity rule per key in knowledge, which is another schema 6 field, or it is held out of 1.7.
- Proposal (builder): a per-key validity rule in a later part, deferred.

**Q-c. The trust reading.** Whether there is a trust reading per repo root, and its states. The record is `projects[<root>].hasTrustDialogAccepted` in the global state file, which knowledge places at `${GLOBAL_STATE_DIR}/.claude.json`. Knowledge has no item naming that key today, and AD-8 reads only keys knowledge lists, so a knowledge item for it is part of any answer.
- Proposal (builder): `TrustReading { root, state: Trusted | NotTrusted | NotRecorded | NotChecked, note }`.

**Q-d. Whether usher may read a trust record at all.** Two texts disagree:
- `epic-1-context.md:29`: "credentials and trust records are never read or written". The AD-8 heading (`ARCHITECTURE-SPINE.md:146`) says the same.
- `ARCHITECTURE-SPINE.md:157`, in the body of AD-8: trust records "are read for discovery (AD-10) and never written".
- PRD FR-6 and the Story 1.7 criteria ask for the trust state "read by key only", which is a read of the trust value, not only of the `projects` keys AD-10 uses for discovery.
- No proposal. This is the operator's call. Part B exists only if the answer allows the read.

**Q-e. The split.** Proposal (builder): A for the FR-2 cases, B for trust, C for doctor-rejects. If Q-a is ruled to need a schema 6 field, A0 (knowledge) comes before A.

**Q-f. OTEL settings in a project or local file.** `harness-knowledge.toml` has no OTEL item. Its 18 settings keys are a list the operator ruled, so adding OTEL keys changes a ruled list.
- Proposal (builder): hold it out of 1.7 and record it in `deferred-work.md`.

## Parts (provisional until Q-e is ruled)

| Part | Content | Waits on | Based on |
|---|---|---|---|
| A0 | knowledge: the field Q-a needs, schema 6, its check, the `permissions.defaultMode` seed | Q-a | main |
| A | the FR-2 cases in the Claude Code map; the local-layer `autoMode` pin | Q-a, Q-f | A0, or main if no A0 |
| B | the trust reading, its knowledge item, and the untrusted-layers reason | Q-c, Q-d | A |
| C | an entry doctor rejects | Q-b | B or A |

## Code Map

- `crates/usher-knowledge/` -- A0: the field Q-a needs and its structural check; B: the trust key item
- `crates/usher-core/src/map.rs` -- the reading types (`ReadingStatus`, `SkippedLayer`, `NotObserved`); a new status or a trust type only if Q-a or Q-c is ruled to need it
- `crates/usher-adapter-claude-code/src/map.rs` -- the readings for each FR-2 case; B: the trust reading from the global state file
- `crates/usher-store/` -- only if a new status is ruled (Q-a option 2): the check constraint and `user_version` 4
- `crates/usher-adapter-claude-code/tests/map.rs` -- the pins below, and the reds for each part
- `_bmad-output/implementation-artifacts/sprint-status.yaml` -- 1-7 moves with the story
- `_bmad-output/implementation-artifacts/deferred-work.md` -- whatever Q-b and Q-f hold out

## Tasks & Acceptance

**Already pinned on main** (named here so no part rebuilds them), in `crates/usher-adapter-claude-code/tests/map.rs`:
- `a_key_found_in_a_layer_outside_its_scopes_is_unread_and_never_feeds_the_value` (`autoMode` in a project file)
- `an_override_variable_that_is_present_overrides_the_key_and_the_note_holds_only_its_name` and `the_override_does_nothing_while_its_variable_is_not_set`
- `a_skipped_layer_carries_the_knowledge_reason_and_says_the_harness_gave_none` and `a_managed_source_shadowed_by_rank_is_a_skipped_layer`
- `the_server_managed_cache_and_the_global_state_file_yield_no_rows` and `the_not_observed_set_names_what_applies_to_this_environment`

**New pin:** `autoMode` set in the local layer reads unread with the knowledge item id, and the value comes from the user layer. It is the only new pin for the green cases.

**Execution:**
- [ ] reds first for each part, shown failing on assertions, reported before any implementation
- [ ] A0, if Q-a needs it: the knowledge field, schema 6, the check, the seed
- [ ] A: the FR-2 readings and the local-layer pin
- [ ] B, if Q-d allows it: the trust knowledge item, the trust reading, the untrusted-layers reason
- [ ] C, if Q-b keeps it in 1.7
- [ ] sprint status: 1-7 moves with the story

**Acceptance Criteria:**
- Given each Claude Code case in addendum section 1.1 for FR-2 that the rulings keep in 1.7, when the directory is mapped, then each is reported as declared and not in effect, with its reason, and none reads in effect
- Given a repo root, if Q-d allows the read, when trust is mapped, then the state matches `projects[<repo root>].hasTrustDialogAccepted` in the global state file, read by key only and never written, and an untrusted directory whose project layers do not load says so

## Design Notes

"Declared and not in effect" is already a family on main, not one status: a skipped layer, `OverriddenByEnvironment`, `Unread` with a knowledge note, and `NotObserved::ServerManagedValues`. The FR-2 test for each case is therefore that the reading is never `InEffect` and carries the reason. `NotDecided`, which `defaultMode` reads today, is not in effect either, but it does not say the value is ignored, so it does not meet FR-2. Q-a decides how the ignored-value case joins the family.

The global state file is already a listed source (read `read`, rank 1), and `the_server_managed_cache_and_the_global_state_file_yield_no_rows` pins that it yields no settings rows. A trust reading in B is a separate reading from that file, not a settings row.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: no diff
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` -- expected: no warnings
- `cargo test --locked --workspace --all-targets` -- expected: all pass
- `cargo deny check` (CI only) -- expected: green
