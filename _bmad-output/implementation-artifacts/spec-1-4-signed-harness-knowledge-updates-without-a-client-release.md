---
title: 'Signed harness-knowledge updates without a client release (Story 1.4)'
type: 'feature'
created: '2026-11-10'
status: 'draft'
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

**Always:** `#![forbid(unsafe_code)]`; one new external crate, `minisign-verify = "=0.3.0"` (spine Stack table, line 379), used in `usher-knowledge` and as its build-dependency, no other new crate without a ruling; failures use `UsherError` (step, check, status, next action). The signature is checked over the exact received bytes before any parse. The trusted list is read by `build.rs` from `USHER_TRUSTED_KEYS` (`current:<base64>,next:<base64>`, one `current`, at most one `next`), each key validated with `minisign-verify`, rerun-if-env-changed, written as a const into `OUT_DIR`. A release build (`PROFILE=release`) with an empty list, a malformed key, or the test-key feature on fails the build. No key, public or private, is stored in the repo; tests generate a throwaway pair per run in `usher-testkit` and inject the public half through a test-only constructor. A key not in the list is revoked.

**Never:** fetch or write anything over the network or to disk (Story 1.15 owns the egress port); add an HTTP client; change `usher`, `usher-core`, CI or `deny.toml`; change Story 1.3 item shape; decide key holders or rotation (PRD Q8); touch the embedded seed file.

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
| Malformed signature | signature text missing or not minisign | rejected | same |
| Empty list, dev build | no `USHER_TRUSTED_KEYS` | builds; every update rejected as untrusted | `UsherError`, next action names the variable |
| Empty list or malformed key, release build | env empty or key not valid base64 minisign | build fails | `build.rs` message naming the entry |

</frozen-after-approval>

## Open Questions

1. **Does 1.4 fetch, or only load?** epics 1.4 says "when usher loads it"; AD-12 says updates are fetched only when asked (AD-14); Story 1.15 says "Story 1.4's knowledge fetch is the first caller" of the egress port. Options: load from supplied bytes only (no network code, no HTTP crate; 1.15 adds the fetch and its egress registration) / fetch here (needs an HTTP client crate and the egress port, which does not exist). Recommended: load only.
2. **Test signer.** `usher-testkit` must sign for tests, and `minisign-verify` only verifies. Options: A) `ed25519-dalek = "=3.0.0"` (BSD-3-Clause, in deny.toml's allow list; spine line 380) in `usher-testkit` only, seed from std (a test key needs uniqueness, not secrecy), writing legacy `Ed` signatures, so the verifier accepts legacy mode / B) dalek plus a BLAKE2b crate to write the default prehashed `ED` signatures, so the verifier rejects legacy mode (two new crates, closest to a real release). Recommended: A. Transitive crate count not yet measured.

## Code Map

- `crates/usher-knowledge/src/schema.rs` -- `KnowledgeFile::parse` (structural check), `version()` -> `KnowledgeVersion` (`sequence()`, `schema_version()`), `embedded()`; reuse, do not change item shape
- `crates/usher-knowledge/src/lib.rs` -- re-exports; add `mod update` and `mod trust`
- `crates/usher-knowledge/src/update.rs` -- new: `Knowledge::update(&mut self, file, signature, &TrustedKeys)`; leaves `self` unchanged on any error
- `crates/usher-knowledge/src/trust.rs` -- new: `TrustedKeys`, the generated const, test-only constructor behind `cfg(any(test, feature = "test-keys"))`
- `crates/usher-knowledge/build.rs` and `build/keys.rs` -- new: list reader and validator; `keys.rs` is `#[path]`-included by build.rs and by a test, so a malformed key's rejection is tested
- `crates/usher-testkit/src/signer.rs` -- new: throwaway key pair and minisign-format signature (per Open Question 2)
- `Cargo.toml`, `crates/usher-knowledge/Cargo.toml`, `crates/usher-testkit/Cargo.toml`, `Cargo.lock` -- the dependency lines
- `_bmad-output/planning-artifacts/architecture/architecture-usher-2026-11-08/ARCHITECTURE-SPINE.md` -- AD-12 (lines 196-208); record the build.rs ruling beside it
- minisign-verify 0.3.0 `PublicKey::from_base64`, `Signature::decode`, `PublicKey::verify(bin, sig, allow_legacy)` (checks key id, returns `UnexpectedKeyId`)
- Do not change: `crates/usher`, `crates/usher-core`, `.github/workflows/ci.yml`, `deny.toml`, the seed file

## Tasks & Acceptance

**Execution:**
- [ ] `crates/usher-knowledge/build.rs`, `build/keys.rs` -- read, validate, emit the trusted list; fail release on empty, malformed or test-keys -- the compile-time rule
- [ ] `crates/usher-knowledge/src/trust.rs`, `src/update.rs`, `src/lib.rs` -- verify, then sequence, then structural check; atomic swap -- the update path
- [ ] `crates/usher-testkit/src/signer.rs` -- throwaway signer -- tests need signed fixtures
- [ ] tests in each module covering every matrix row, including that a malformed key fails the validator the build script runs
- [ ] `ARCHITECTURE-SPINE.md` -- ruling note beside AD-12

**Acceptance Criteria:**
- Given an update signed by a key in the build-time list, when usher loads it, then the signature and a higher sequence are both checked before any item is used, and the new version is reported
- Given the repo, when tests run, then no signing key, public or private, is stored in it, and a release build with an empty or malformed list fails
- Given a bad signature or a sequence not higher, when usher loads the update, then it is rejected, the previous knowledge stays in use, and the error says why

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Design Notes

Order is fixed: signature over raw bytes, then `parse` (which includes the structural check), then sequence. An unsigned file never reaches the parser, and a forged file cannot probe it. `KnowledgeFile` is not mutated on error. The build script's validator is a shared module so a malformed key is tested without nesting a cargo build in a test; the test-keys feature is refused in release by the same script.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: no diff
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` -- expected: no warnings
- `cargo test --locked --workspace --all-targets` -- expected: all pass
- `USHER_TRUSTED_KEYS= cargo build --locked --release -p usher-knowledge` -- expected: fails naming the empty list
- `cargo deny check` (CI only) -- expected: green
