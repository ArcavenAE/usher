---
title: 'Scratch homes and the observation invariant (Story 1.2)'
type: 'feature'
created: '2026-11-10'
status: 'done'
baseline_commit: 'b95c08a8da87832aa645710d1fbb59d8b8dac5d7'
route: 'dispatch'
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/implementation-artifacts/epic-1-context.md']
---

<frozen-after-approval reason="human-owned intent: do not modify unless human renegotiates">

## Intent

**Problem:** "Observation changes nothing" (SM-5, AD-9, AD-20) is a claim until a test proves it, and every adapter story from 1.5 on needs the same way to run a harness without touching the real home.

**Approach:** Give `usher-testkit` a scratch home (a short-path temp dir that carries `HOME`, `TMPDIR` and each harness home variable for the processes a test spawns), a metadata digest guard over the real harness homes, and a pinned-binary probe that records the harness version or names why it is missing. No external crates.

## Boundaries & Constraints

**Always:** std only; `#![forbid(unsafe_code)]`, so the scratch environment is applied to child `Command`s and never through `std::env::set_var`. The scratch root is a fresh directory under `/tmp` with a short name and is removed on drop. The digest covers name, file type, size, mtime, ctime and inode (symlinks are not followed). Tests that write files run only under scratch homes or injected fake homes; the real home is only ever read for metadata. The harness variables are `CLAUDE_CONFIG_DIR`, `CODEX_HOME` and `GOOSE_PATH_ROOT` (goose's relocation variable per the research map; absolute paths only).

**Decisions (director, 2026-11-10):** a missing harness binary is reported as `Probe::Missing { reason }`; the test prints `SKIPPED: <reason>` to stderr and returns, the testkit records every skip, and prints the skip list (test and reason) so a skip never reads as a pass in aggregate. No new environment switch. The digest is compared only within one process run and never written to disk; if that changes, a stable hash replaces `DefaultHasher`. The PR body states that cargo shows skipped tests as ok with the reason on stderr.

**Never:** add a crate; change `usher`, `usher-core` or CI; add a harness adapter; walk or hash the real home inside this story's own tests (a live harness session there would make them flaky; they use injected homes); choose or add a license field.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Scratch env | `ScratchHome::new()` | `HOME`, `TMPDIR`, `CLAUDE_CONFIG_DIR`, `CODEX_HOME`, `GOOSE_PATH_ROOT` all resolve inside one fresh dir whose path is under 60 chars | creation failure returns an `io::Error` |
| Cleanup | scratch dropped | the directory no longer exists | removal failure is ignored on drop |
| Child sees scratch | `home.command("sh")` printing `$HOME` | prints the scratch home, not the real one | N/A |
| Unchanged | guard snapshot, no writes, verify | `Ok` | N/A |
| Changed (positive control) | guard over a fake "real" home, then a file is written there | `verify` returns `Err` naming the home | the test asserts the `Err` |
| Absent home | a harness home that does not exist | snapshots as absent and stays equal until it appears | creating it later is a change |
| Binary found | pinned binary on the path | result records the binary path and its `--version` output | non-UTF-8 output is kept lossily |
| Binary missing | name not on the path | result is `Missing` with a named reason; the skip is recorded and the skip list printed | N/A |

</frozen-after-approval>

## Code Map

- `crates/usher-testkit/src/lib.rs` -- currently a placeholder (`CRATE_NAME`); declares the new modules
- `crates/usher-testkit/Cargo.toml` -- no dependencies; unchanged
- `_bmad-output/planning-artifacts/architecture/architecture-usher-2026-11-08/ARCHITECTURE-SPINE.md` -- AD-9 and AD-20 (rules the kit serves)
- Do not change: `crates/usher`, `crates/usher-core`, `.github/workflows/ci.yml`, `deny.toml`, pack files.

## Tasks & Acceptance

**Execution:**
- [x] `crates/usher-testkit/src/scratch.rs` -- `ScratchHome` (root, home, tmp, the three harness dirs), `env()` pairs, `command(program)` with those variables set, removal on drop -- scratch homes
- [x] `crates/usher-testkit/src/invariant.rs` -- metadata digest of a tree, `RealHomes` (ambient resolution from the real environment and injected paths), `RealHomeGuard::snapshot`/`verify`/`assert_unchanged`, `observe(|scratch| ..)` wrapper -- the observation invariant
- [x] `crates/usher-testkit/src/harness.rs` -- `probe(name, search_path)` returning `Found { path, version }` or `Missing { reason }`, and the skip helper per the Decisions block -- pinned binaries
- [x] `crates/usher-testkit/src/lib.rs` -- declare and re-export the modules; keep `CRATE_NAME`
- [x] tests in each module covering every matrix row, including the positive control

**Acceptance Criteria:**
- Given usher-testkit, when a test asks for a scratch home, then the five variables point inside one fresh short-path temp dir that is removed afterwards
- Given a test run inside a scratch home, when it finishes, then the guard compares the digest of each real harness home before and after and fails on a difference, and a positive-control test proves the check can fire
- Given a pinned harness binary, when the testkit runs it, then its version is recorded with the result, and a missing binary is reported as skipped with a named reason

## Implementation Notes

- Std only. The scratch environment reaches spawned `Command`s only; no `set_var`.
- Skip list: the std test harness has no end-of-run hook, so `skip()` prints the cumulative list on each skip. Director ruled option (a) on 2026-11-10 (valid until 2026-11-12). The list is per test binary, and cargo captures stderr of passing tests, so the SKIPPED lines show only under `-- --nocapture` or `--show-output`.
- The digest root is followed when it is a symlink (a patch from review); entries below it are not followed.
- Tests: 34 pass in the workspace (25 in the testkit, 9 existing; of the 25, 20 written first, 5 added by review: symlinks below the root, a symlinked root, an empty home list refused by `observe`, a dangling symlink at the watched path, and creating or repointing a link below the root).
- A symlink at a watched path, dangling or not, counts as present, and its link metadata including the `readlink` target text goes into the digest (director ruling after review, as `find -H` would list it).

## Spec Change Log

## Review Triage Log

| Finding | Verdict | Route | Evidence |
|---|---|---|---|
| Symlinked harness home root hashed as a bare link (edge, verification-gap) | medium | patch | `digest_tree` used `symlink_metadata` on the root; now `fs::metadata` on the root only, with a test |
| No test for symlinks not followed (verification-gap) | medium | patch | test added with a link out and a self loop |
| `observe` passes with an empty home list (edge) | medium | patch | assert added, test added |
| Entry vanishes between listing and stat (edge) | low | patch | NotFound skipped; one match arm |
| Skip lines hidden by libtest capture (blind, edge) | medium | defer to PR body | true; ruling text says cargo shows ok with reason on stderr; stated accurately in the PR body and raised to the supervisor |
| Skip list per binary, demo entry in list (blind) | low | reject | per-binary is the ruled option; the demo entry exists only in the testkit's own unit test binary |
| Missing conflates broken with absent (blind, edge) | medium | defer | the matrix row makes a failing `--version` Missing; a distinct variant is a spec change for a later story |
| No timeout on `--version` (blind, edge) | medium | defer | needs a spawn and deadline; later adapter story |
| XDG_* and other variables inherited by `command()` (blind, edge) | medium | defer | spec fixes the five variables; revisit in the first adapter story |
| `~/.claude.json` and goose data/state not watched (blind) | maybe-false | defer | would need the research map; settles the SM-5 coverage claim |
| No exclusion list for volatile paths (blind) | medium | defer | needed once stories 1.5 and later guard real homes |
| Non-Unix build fails (blind, edge) | low | reject | doc says Unix only for now; no Windows target exists |
| Empty or relative HOME and variable values (edge) | low | reject | fix adds branches; unlikely in practice |
| Empty `--version` output returns Found (edge) | low | reject | adds a branch for an unlikely case |
| Same-size rewrite test may flake on coarse timestamps (blind, edge) | low | reject | ns mtime and ctime on APFS; ctime changes on write |
| ETXTBSY on exec of fresh script (edge, verification-gap) | maybe-false | reject | Linux only; macOS target; not reproduced |
| Scratch root mode 0755 and crash leaks (blind, edge) | low | reject | short-lived test scratch; fix adds code |
| Spec loose ends: Open Question wording, tasks, notes (blind) | low | patch | wording fixed, tasks ticked, notes filled |
| Unrelated 1-1 done in sprint-status (blind) | false | reject | supervisor approved; stated in the PR body |

## Design Notes

The guard is an equality check, so `std::hash::DefaultHasher` is enough; its algorithm is not promised stable across Rust releases, so a digest is never stored. Real homes resolve as `CLAUDE_CONFIG_DIR` or `~/.claude`, `CODEX_HOME` or `~/.codex`, and `GOOSE_PATH_ROOT` or `~/.config/goose`, read-only. Library code that later stories write must take its launch context explicitly (AD-17) rather than read `HOME` in process, because the scratch variables only reach spawned processes.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: no diff
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` -- expected: no warnings
- `cargo test --locked --workspace --all-targets` -- expected: all pass, including the positive control
