---
title: 'Referenced harness knowledge, embedded in the binary (Story 1.3)'
type: 'feature'
created: '2026-11-10'
status: 'in-review'
route: 'dispatch'
baseline_commit: '5c4fbcd4066f39104de21b497ece95e940767ee4'
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/implementation-artifacts/epic-1-context.md']
---

<frozen-after-approval reason="human-owned intent: do not modify unless human renegotiates">

## Intent

**Problem:** Every fact usher states about a harness must name its source and when it was last verified (FR-30), and a missing capability must read as unsupported, but no crate holds harness knowledge yet.

**Approach:** Add `usher-knowledge`: the AD-12 TOML schema in two layers (slow: layers, paths, credential stores; fast: keys, defaults, hazards), items keyed to harness version ranges, a structural check that rejects an item missing a reference or a last-verified date, an embedded seed file, and a support-level query that lists capabilities per harness version. The file's version is reportable (FR-31). No signing or updates (Story 1.4).

## Boundaries & Constraints

**Always:** `#![forbid(unsafe_code)]`; one new external crate, `toml_edit = "=0.25.19"` (default features, no serde), only in `usher-knowledge`, walked by hand (supervisor and the spine's Stack table); failures use `UsherError` from `usher-core`. An item is valid only with a reference (kind one of `vendor-docs`, `changelog`, `source`, `probe`, plus a locator) and a TOML date `last_verified`. Capabilities are `inventory`, `install`, `map`, `compare`, `place`, `apply`; an unknown capability, layer, class or reference kind fails the check. Support entries are usher's own statement, not a harness fact, so they carry no reference. A harness or version with no matching support entry reports every capability unsupported. The seed file states only facts that research map `technical-harness-settings-observability-map-2026-11-02` cites, with its URLs and the map's verification date; it offers no capability yet, because no adapter exists.

**Never:** add signing, update fetching, sequence comparison or the trusted-key list (Story 1.4); add a harness adapter; change `usher`, `usher-core`, `usher-testkit`, CI or `deny.toml`; add a crate other than `toml_edit`; choose or add a license field.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Valid file | items with reference and date, ranges, both layers | parses; items listed per harness and version | N/A |
| Missing reference | an item with no `reference` | check fails naming the item id | `UsherError` with step, check, status, next action |
| Missing date | an item with no `last_verified` | check fails naming the item id | same |
| Bad date or reference | date with a time part, unknown kind, empty locator | check fails naming the item id | same |
| Unknown layer, class or capability | any value outside the sets | check fails naming it | same |
| Duplicate item id | two items share an id | check fails | same |
| Version range | `>=1.2.0, <2.0.0` against 1.2.0, 1.9.9, 2.0.0, 1.1.9 | in, in, out, out | malformed range fails the check |
| Support level | harness and installed version in a support range | lists offered capabilities; every other capability reads unsupported | N/A |
| Unknown harness or version | no matching support entry | all six capabilities unsupported, marked not known | N/A |
| Knowledge version | any loaded file | reports `knowledge v<sequence>` and schema version, for later reports | missing or non-integer sequence fails the check |
| Embedded file | the seed file built into the binary | passes the same check, in tests that run in CI | N/A |

</frozen-after-approval>

## Code Map

- `crates/usher-knowledge/` -- new crate: `Cargo.toml` (workspace package fields, `[lints] workspace = true`, deps `usher-core`, `toml_edit`), `src/lib.rs`, `src/schema.rs` (types, parse and check), `src/version.rs` (semver triple and range), `src/support.rs` (capabilities and support report), `knowledge/harness-knowledge.toml` (seed, embedded with `include_str!`)
- `Cargo.toml` -- add the member and `toml_edit` under `[workspace.dependencies]`; `Cargo.lock` committed
- `crates/usher-core/src/lib.rs` -- `UsherError::new(step, check, status, next_action)`; reuse, do not change
- Architecture spine AD-12 and the Stack table (toml_edit 0.25.19); PRD FR-30 and FR-31; research map for seed facts (Claude Code `CLAUDE_CONFIG_DIR` relocates the user home; codex `CODEX_HOME`; goose `GOOSE_PATH_ROOT` absolute only)
- Do not change: `crates/usher`, `crates/usher-core`, `crates/usher-testkit`, `.github/workflows/ci.yml`, `deny.toml`, pack files

## Tasks & Acceptance

**Execution:**
- [x] `crates/usher-knowledge/src/version.rs` -- `Version` (major.minor.patch) and `VersionRange` (comma-joined `>=`, `>`, `<=`, `<`, `=` terms) with `contains` -- version ranges
- [x] `crates/usher-knowledge/src/schema.rs` -- `KnowledgeFile::parse(&str)`, `Item`, `Reference`, `Layer`, `Class`, the structural check, `version()` stamp, `embedded()` -- the schema and its check
- [x] `crates/usher-knowledge/src/support.rs` -- `Capability`, `SupportReport` from `KnowledgeFile::support(harness, installed)` -- support levels
- [x] `crates/usher-knowledge/knowledge/harness-knowledge.toml` -- seed items for claude-code, codex and goose launch-context variables, support entries offering nothing yet
- [x] `Cargo.toml`, `crates/usher-knowledge/Cargo.toml`, `Cargo.lock` -- the member and the one dependency
- [x] tests in each module covering every matrix row; `cargo deny check` stays green

**Acceptance Criteria:**
- Given the knowledge schema, when a file is parsed, then an item missing a reference or a last-verified date fails the structural check, and the embedded file passes it in CI
- Given items in a slow and a fast layer, when they are read, then each is keyed to a harness version range
- Given a harness and its installed version, when usher reports its support level, then it lists the capabilities offered for that version and every other capability reads unsupported, never as a clean result
- Given a report that relies on knowledge, when it is built, then it can name the knowledge file version used

## Implementation Notes

- Std plus `toml_edit =0.25.19` only. With default features it pulls 15 transitive crates into `Cargo.lock` (indexmap, hashbrown, equivalent, winnow, memchr, toml_datetime, toml_parser, toml_writer, serde_core, serde_derive, syn, quote, proc-macro2, unicode-ident); all are MIT or Apache-2.0 per `cargo metadata`, and `cargo deny` runs in CI (not installed locally).
- The seed file keys each fact to the version the research map was run against (claude-code 2.1.317, codex 0.160.1, goose 1.57.0) up to the next major line, not to all versions.
- When two support entries for one harness both match a version, the first in file order wins.
- A class must belong to the item's layer (slow: layers, paths, credential-stores; fast: keys, defaults, hazards, never-call, disputed); a mismatch fails the check.

## Spec Change Log

## Review Triage Log

## Design Notes

The version stamp is `sequence` (the integer Story 1.4 compares) plus `schema_version`; Story 1.4 adds signature and sequence checks on top without changing item shape. Ranges are strict three-part versions; a harness whose versions do not fit that shape is a later story's problem, and a malformed range fails loudly rather than matching nothing. Dates are TOML dates, so a stray time part is caught by the parser's own type. The deny.toml license allow list already covers `toml_edit` and its dependencies (MIT or Apache-2.0).

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: no diff
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` -- expected: no warnings
- `cargo test --locked --workspace --all-targets` -- expected: all pass, including the embedded-file check
- `cargo deny check` (CI only; not installed locally) -- expected: green
