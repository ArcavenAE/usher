---
title: 'Bootstrap the usher workspace from the fleet baseline (Story 1.1)'
type: 'chore'
created: '2026-11-10'
status: 'done'
baseline_commit: 'e1a0e5aeac3a903626809e3e444ebe396f706b26'
route: 'dispatch'
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/implementation-artifacts/epic-1-context.md']
---

<frozen-after-approval reason="human-owned intent: do not modify unless human renegotiates">

## Intent

**Problem:** Usher has the fleet-bootstrap pack but no Rust tooling, no cargo workspace and no CI, so no later story can build, lint or test the same way on every host.

**Approach:** Apply the orc's `fleet-bootstrap-rust` pack with the toolchain pinned to 1.99.1, add a minimal three-crate workspace (`usher`, `usher-core`, `usher-testkit`) with no external crates, and add a pull-request CI that runs fmt, clippy, test and cargo-deny on macOS.

## Boundaries & Constraints

**Always:** edition 2024; `#![forbid(unsafe_code)]` in every crate; warnings are errors; no `unwrap()` outside tests; `UsherError` carries step, check, status and a next action. Pin Rust 1.99.1 in both `rust-toolchain.toml` and `mise.toml`. Pack files are copied as shipped except for the pin and the `@rust.md` index line.

**Decisions (director, 2026-11-10):** CI uses upstream `dtolnay/rust-toolchain` and `Swatinem/rust-cache` by SHA plus `step-security/harden-runner`, because usher is private and the wrapped actions fail closed (deviates from the AC's "step-security action pins"). `cargo deny check` runs in a macOS job that installs a pinned cargo-deny through `taiki-e/install-action` pinned by SHA, because the fleet's cargo-deny action is docker and Linux only (new action accepted). fmt runs as `cargo fmt --all -- --check` on 1.99.1, with the pack's `rustfmt.toml` and `lefthook.yml` left as shipped. The PR body names each deviation.

**Never:** add an external crate or dependency; add crates beyond the three named; implement any mode other than version and unknown-mode handling; change any repo or org setting; touch the installed bmad pack.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Version | `usher --version` | prints `usher <version>`, exit 0 | N/A |
| Unknown mode | `usher frobnicate` | exit nonzero; stderr names the step, the check, the status and a next action | `UsherError` rendered |
| No mode | `usher` | exit nonzero with the same next-action shape | `UsherError` rendered |

</frozen-after-approval>

## Code Map

- `_packs/fleet-bootstrap-rust/distribute/` in the orc checkout -- source of `lefthook.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `deny.toml`, `rules/rust.md` (read-only inputs)
- `.claude/rules/_index.md` -- add `@rust.md`
- `.gitignore` -- add `/target/` (nothing ignores build output yet)
- `bloomctl/.github/workflows/ci.yml` (sibling) -- action SHAs to reuse: harden-runner, checkout, dtolnay/rust-toolchain, Swatinem/rust-cache
- `docs/` and `_bmad-output/planning-artifacts/architecture/architecture-usher-2026-11-08/ARCHITECTURE-SPINE.md` -- crate tree (`crates/usher`, `crates/usher-core`, `crates/usher-testkit`), AD-20, stack pin 1.99.1
- Do not change: `_kos/`, `charter.md`, planning artifacts, `.github/workflows/dependency-review.yml`, `release-verify.yml`, `release.yml`.

## Tasks & Acceptance

**Execution:**
- [x] `rust-toolchain.toml`, `rustfmt.toml`, `deny.toml`, `lefthook.yml`, `.claude/rules/rust.md` -- copy from the pack; set channel `1.99.1` -- pack presence and the pin
- [x] `mise.toml` -- `rust = "1.99.1"` -- hosts and CI agree
- [x] `.claude/rules/_index.md` -- add `@rust.md`; `.gitignore` -- add `/target/`
- [x] `Cargo.toml` -- workspace of three crates, resolver 3, edition 2024, `rust-version = "1.99.1"`, workspace lints (`unsafe_code` forbid, clippy defaults)
- [x] `crates/usher-core/src/lib.rs` -- `UsherError` (step, check, status, next action), std-only `Display` and `Error`, unit tests
- [x] `crates/usher-testkit/src/lib.rs` -- empty-by-design library with a crate doc naming AD-20 and one smoke test; scratch homes arrive in Story 1.2
- [x] `crates/usher/src/main.rs` -- `--version`; unknown or missing mode exits nonzero via `UsherError`; integration tests against the built binary
- [x] `.github/workflows/ci.yml` -- PR and main CI per the resolved open questions
- [x] `Cargo.lock` -- generated and committed

**Acceptance Criteria:**
- Given the orc pack, when applied, then the five pack files exist, `_index.md` includes `@rust.md`, and `rust-toolchain.toml` and `mise.toml` both read 1.99.1
- Given the workspace, then it holds only `usher`, `usher-core`, `usher-testkit`, and `usher --version` prints the version while an unknown mode exits nonzero with a next action
- Given a pull request, when CI runs, then fmt, clippy `-D warnings`, test and `cargo deny check` run on macOS with the pinned toolchain

## Implementation Notes

- Applied the pack files as shipped; only `rust-toolchain.toml` (channel 1.99.1), `.claude/rules/_index.md` (`@rust.md`) and `.gitignore` (`/target/`) differ from the sources.
- Workspace `[workspace.lints]` forbids unsafe code; each crate also carries `#![forbid(unsafe_code)]` as `rust.md` writes it. Imports use the merged `use std::{..}` form so stable and nightly rustfmt agree.
- No external crates. `UsherError` is std-only (no `thiserror`); a later story that needs it asks first.
- `Cargo.lock` is committed and CI passes `--locked`.
- Running rustc in the worktree made rustup auto-install the 1.99.1 toolchain under `~/.rustup` (a side effect of `rust-toolchain.toml`).
- `cargo deny` was not run locally (not installed); the first CI run is the check. If `licenses` or `bans` fail on the unlicensed path crates, the likely fix is `[licenses.private] ignore = true` or `allow-wildcard-paths = true` in `deny.toml`.

## Spec Change Log

## Review Triage Log

| Finding | Verdict | Route | Evidence |
|---|---|---|---|
| fmt: CI stable ignores nightly rustfmt options (3 reports) | low | defer | Director ruled Q3 (a); pack files stay as shipped. Import layout is merged-form so both agree |
| main.rs imports would be merged by nightly, hook vs CI disagree | low | patch | Merged to `use std::{env, process::ExitCode}`; stable check still passes |
| deny may fail on unlicensed private crates (3 reports) | maybe-false | defer | Needs a `cargo deny` run; the first CI run settles it |
| deny `wildcards = deny` vs path dependency (2 reports) | maybe-false | defer | Same: the first CI run settles it |
| deny.toml header names the Embark action (3 reports) | low | defer | Pack file left as shipped per Q3; PR body notes it |
| non-UTF-8 argument panics `env::args()` | medium | patch | Confirmed by reading `env::args`; now `args_os` with lossy conversion, and a 0xff argument exits 1 with the report |
| `--version extra` accepted, exit 0 | low | defer | Not in the matrix; a guard adds complexity for an unlikely case |
| println/eprintln panic on broken pipe | low | defer | Unlikely in use; the fix adds code |
| mode text with control characters can forge report lines | low | patch | One-token change: `escape_debug()` on the mode |
| `--help`/`-h` reported as unknown mode | low | defer | Other modes are out of intent for 1.1 |
| `UsherError::new` accepts empty fields | low | defer | A validating constructor adds public surface; later story |
| toolchain version duplicated in four places | low | defer | A comment keeps them in step; a check is a later chore |
| cancel-in-progress also cancels push-to-main runs | low | defer | Same pattern as the sibling repos |
| `#![forbid(unsafe_code)]` attribute missing | low | patch | Added to all three crates |
| `unwrap_used` is warn only, no `allow-unwrap-in-tests` | low | defer | Tests use `expect`; a config file is a later chore |
| CI lacks `--locked` | low | patch | Added to clippy and test |
| harden-runner may not support macOS | false | none | bloomctl's CI runs the same action on `macos-latest` |
| `macos-latest` image not pinned | low | defer | Runner image pinning is a fleet-wide choice |
| spec Code Map has an absolute local path | low | patch | Replaced with an orc-relative path |
| sprint-status should read `review`, not `in-progress` | medium | patch | Set to `review` at completion |
| Notes, Triage and Change Log empty | low | patch | Notes and this log are written; the Change Log stays empty (no loopback) |
| `-V` alias and trailing-arg layout not tested | low | defer | `-V` shares the `--version` code path; unlikely to matter |
| `usher: ` prefix on first line of multi-line report | low | defer | Cosmetic; the layout is stable |
| deny job also runs `sources`, beyond the AC | false | none | deny.toml ships a `[sources]` section, so the check is meaningful |
| verification-gap layer: no gaps in behavioral code | n/a | none | Layer reported none for the CLI and `UsherError` |

## Design Notes

`UsherError` is a plain struct with private fields and getters; `status` is a short string for now (an exit status or error text, per the spine's "its exit status or error"). Callers build it through `UsherError::new`. thiserror is not used because the story adds no external crate; a later story that needs it asks first.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: no diff
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` -- expected: no warnings
- `cargo test --workspace --all-targets` -- expected: all pass
- `cargo run -q -p usher -- --version` -- expected: prints the version, exit 0
- `cargo run -q -p usher -- bogus; echo $?` -- expected: nonzero with a next action on stderr
