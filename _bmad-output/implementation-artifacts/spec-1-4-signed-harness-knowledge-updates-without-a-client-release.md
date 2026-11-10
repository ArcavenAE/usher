---
title: 'Signed harness-knowledge updates without a client release (Story 1.4)'
type: 'feature'
created: '2026-11-10'
status: 'in-review'
route: 'dispatch'
baseline_commit: '53f1b3b3d056656e7b478e9d86a83c157fb9e86f'
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/implementation-artifacts/epic-1-context.md']
---

<frozen-after-approval reason="human-owned intent: do not modify unless human renegotiates">

## Intent

**Problem:** A release that changes a harness should reach people without a new usher build, but knowledge is only the file built into the binary, and an unsigned update could feed usher forged or replayed facts (FR-31).

**Approach:** In `usher-knowledge`, accept an update file only when its minisign (ed25519) signature verifies against a key in a build-time trusted list (`current` and `next`), its sequence number is higher than the one in force, and it passes the Story 1.3 structural check, in that order. A rejected update leaves the embedded or last good knowledge in use and says why. A `build.rs` supplies the list (director ruling, Story 1.4: option B).

## Boundaries & Constraints

**Always:** `#![forbid(unsafe_code)]`; `minisign-verify = "=0.3.0"` (spine Stack table, line 379), used in `usher-knowledge` and as its build-dependency; `ed25519-dalek = "=3.0.0"` and `blake2 = "=0.11.0"` as normal dependencies of `usher-testkit` only (a library module cannot use dev-dependencies), `usher-testkit` only ever a dev-dependency of shipping crates, so the test signer writes real prehashed minisign signatures and neither crate is in a shipping build; no other new crate without a ruling (director ruling, Story 1.4); production verifies prehashed `ED` signatures only and never sets `allow_legacy`, so a legacy `Ed` signature is rejected; failures use `UsherError` (step, check, status, next action). The signature is checked over the exact received bytes before any parse. The trusted list is read by `build.rs` from `USHER_TRUSTED_KEYS` (`current:<base64>,next:<base64>`, one `current`, at most one `next`), each key validated with `minisign-verify`, rerun-if-env-changed, written as a const into `OUT_DIR`. A release build (`PROFILE=release`) with an empty list, a malformed key, or the test-key feature on fails the build. No key, public or private, is stored in the repo; tests generate a throwaway pair per run in `usher-testkit` and inject the public half through a test-only constructor. A key not in the list is revoked.

**Never:** fetch or write anything over the network or to disk: updates load from supplied bytes only, and Story 1.15 adds the fetch through its egress port (director ruling, Story 1.4); add an HTTP client; change `usher`, `usher-core`, `deny.toml` or CI beyond the one step below; change Story 1.3 item shape; decide key holders or rotation (PRD Q8); touch the embedded seed file.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Valid update | signed by `current`, sequence above in force, passes structural check | new knowledge in use; `knowledge v<seq>` reported | N/A |
| Signed by `next` | same, signed by the `next` key | accepted | N/A |
| Bad signature | one byte of the file changed after signing | rejected, previous knowledge stays | `UsherError` naming the signature check |
| Unknown key | signed by a key not in the list | rejected, previous knowledge stays | same, names the key id |
| Not higher | sequence equal to or below the one in force | rejected, previous knowledge stays | same, names both numbers |
| Signed but invalid | valid signature, item missing a reference | rejected, previous knowledge stays | the Story 1.3 check error |
| Unsigned garbage | bytes that are not TOML, bad signature | signature failure reported, not a parse error | same as bad signature |
| Legacy signature | valid legacy `Ed` signature by a trusted key | rejected, previous knowledge stays | `UsherError` naming the signature check |
| Malformed signature | signature text missing or not minisign | rejected | same |
| Empty list, dev build | no `USHER_TRUSTED_KEYS` | builds; every update rejected as untrusted | `UsherError`, next action names the variable |
| Empty list or malformed key, release build | env empty or key not valid base64 minisign | build fails | `build.rs` message naming the entry |

</frozen-after-approval>

## Code Map

- `crates/usher-knowledge/src/schema.rs` -- `KnowledgeFile::parse` (structural check), `version()` -> `KnowledgeVersion` (`sequence()`, `schema_version()`), `embedded()`; reuse, do not change item shape
- `crates/usher-knowledge/src/lib.rs` -- re-exports; add `mod update` and `mod trust`
- `crates/usher-knowledge/src/update.rs` -- new: `Knowledge::update(&mut self, file, signature, &TrustedKeys)`; leaves `self` unchanged on any error
- `crates/usher-knowledge/src/trust.rs` -- new: `TrustedKeys`, the generated const, test-only constructor behind `cfg(any(test, feature = "test-keys"))`
- `crates/usher-knowledge/build.rs` and `build/keys.rs` -- new: list reader and validator; `keys.rs` is `#[path]`-included by build.rs and by a test, so a malformed key's rejection is tested
- `crates/usher-testkit/src/signer.rs` -- new: throwaway key pair and real prehashed (`ED`) minisign signature (BLAKE2b-512 of the message, ed25519 over the hash, then the global signature)
- `Cargo.toml`, `crates/usher-knowledge/Cargo.toml`, `crates/usher-testkit/Cargo.toml`, `Cargo.lock` -- the dependency lines
- `_bmad-output/planning-artifacts/architecture/architecture-usher-2026-11-08/ARCHITECTURE-SPINE.md` -- AD-12 (lines 196-208); record the build.rs ruling beside it
- minisign-verify 0.3.0 `PublicKey::from_base64`, `Signature::decode`, `PublicKey::verify(bin, sig, allow_legacy)` (checks key id, returns `UnexpectedKeyId`)
- `.github/workflows/ci.yml` -- one new step: fail if `cargo tree -e normal -p usher` shows `ed25519-dalek` or `blake2` (director ruling, Story 1.4; no other CI change)
- Do not change: `crates/usher`, `crates/usher-core`, `deny.toml`, the seed file

## Tasks & Acceptance

**Execution:**
- [x] `crates/usher-knowledge/build.rs`, `build/keys.rs` -- read, validate, emit the trusted list; fail release on empty, malformed or test-keys -- the compile-time rule
- [x] `crates/usher-knowledge/src/trust.rs`, `src/update.rs`, `src/lib.rs` -- verify, then sequence, then structural check; atomic swap -- the update path
- [x] `crates/usher-testkit/src/signer.rs` -- throwaway signer -- tests need signed fixtures
- [x] tests in each module covering every matrix row, including that a malformed key fails the validator the build script runs
- [x] `.github/workflows/ci.yml` -- the one step above, shown red locally when a crate is made a normal dependency of a shipping crate, green at head
- [x] `ARCHITECTURE-SPINE.md` -- ruling note beside AD-12, and the three rulings (build.rs, load-only, prehashed-only signer) with `blake2` 0.11.0 added to the Stack table

**Acceptance Criteria:**
- Given an update signed by a key in the build-time list, when usher loads it, then the signature and a higher sequence are both checked before any item is used, and the new version is reported
- Given the repo, when tests run, then no signing key, public or private, is stored in it, and a release build with an empty or malformed list fails
- Given a bad signature or a sequence not higher, when usher loads the update, then it is rejected, the previous knowledge stays in use, and the error says why

## Implementation Notes

- New: `usher-knowledge` `build.rs`, `build/keys.rs` (list parser, release rule, generated const), `src/trust.rs`, `src/update.rs`; `usher-testkit` `src/signer.rs`. Edited: the three manifests, `Cargo.lock`, `lib.rs` of both crates, the spine (AD-12 rulings, Stack table), one CI step.
- Added crates (lock file, names only): `minisign-verify` (no dependencies) for production; `ed25519-dalek` and `blake2` with their dependencies for the test signer, 24 lock entries in all. `cargo tree -e normal -p usher` shows neither signing crate.
- Key ids print the way minisign prints them (eight bytes, reversed); checked against the example in the minisign documentation.
- The release rule lives in `keys::check_release` so a unit test covers it; the release build was also run by hand for: empty list, `test-keys` on, valid key (fail, fail, pass).
- Known limit: the sequence in force lives in memory. After a restart, a replay of a signed file above the embedded sequence is accepted until a later story persists the highest accepted sequence (deferred-work.md).

## Spec Change Log

## Review Triage Log

Three layers (blind, edge-case, verification-gap), findings grouped by root cause.

| Finding | Verdict | Route / evidence |
|---|---|---|
| Frozen Approach says signature, sequence, structural; code and Design Notes parse (structural) first, then sequence | low | intent_gap, sent to the supervisor: the sequence is inside the file, so parsing must come first; asks to reword the frozen sentence |
| Release refusals in build.rs untested | medium | patch: rule moved to `keys::check_release`, unit-tested; release builds also run by hand |
| `build_time` with a non-empty list never run; role mapped any other string to `next` | medium | patch: `TrustedKeys::from_pairs` (strict roles), tested and used in an update test |
| "names the key id" test compared the function to itself; id printed in file order, minisign prints it reversed | medium | patch: reversed, test pins to the signer's own id; verified against the minisign documentation example |
| CI guard host-only, `-e normal` only, silent on an empty tree | low | patch: `-e normal,build --target all --all-features`, and it fails if `usher` is not in the tree |
| CI guard may collide with a later shipping use of ed25519-dalek (host keys) | maybe-false | defer: only a story that adds host keys can settle it |
| base64 decoder accepted padding in a non-final chunk | low | patch: padding only in the last chunk, tested |
| non-UTF-8 `USHER_TRUSTED_KEYS` read as empty | low | patch: build stops |
| one next-action text for every failure | low | patch: signing failures and file failures read differently |
| rollback protection lasts only for the process | medium | defer: persistence needs the store (a later story); recorded as a known limit |
| nothing outside `usher-knowledge` calls `Knowledge::update` | low | defer: wiring belongs to the story that reports knowledge |
| no size cap on update bytes | low | rejected: the caller supplies the bytes, no network input in this story, and a cap adds a guard |
| `PROFILE` is the only release signal | false | the director ruling names `PROFILE release`; cargo reports the root profile for custom profiles |
| `Cargo.lock` missing from the diff | false | the diff file excluded it; it is modified and committed |
| hand-rolled base64 on both sides | low | rejected: removing it adds a crate or loses the key id in errors |
| `cargo deny` not run locally, bookkeeping blank, 1-3 still `in-progress` in sprint status | low | patch: logs filled here; 1-3 set to done; deny runs in CI |

## Design Notes

Order is fixed: signature over raw bytes, then `parse` (which includes the structural check), then sequence. An unsigned file never reaches the parser, and a forged file cannot probe it. `KnowledgeFile` is not mutated on error. The build script's validator is a shared module so a malformed key is tested without nesting a cargo build in a test; the test-keys feature is refused in release by the same script.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: no diff
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` -- expected: no warnings
- `cargo test --locked --workspace --all-targets` -- expected: all pass
- `USHER_TRUSTED_KEYS= cargo build --locked --release -p usher-knowledge` -- expected: fails naming the empty list
- `cargo deny check` (CI only) -- expected: green
