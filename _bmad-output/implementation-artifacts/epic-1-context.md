# Epic 1 Context: See my seat

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

A person lists every settings source on their host and sees, per launch context, each setting in effect with its provenance, version stamp and blind spots, for Claude Code, the Claude desktop app, codex and goose, on harness knowledge that is referenced, signed and probe-verified. Nothing in this epic writes outside usher's own state directory. This is the MVP epic and the first one built.

## Stories

- Story 1.1: Bootstrap the usher workspace from the fleet baseline
- Story 1.2: Scratch homes and the observation invariant
- Story 1.3: Referenced harness knowledge, embedded in the binary
- Story 1.4: Signed harness-knowledge updates without a client release
- Story 1.5: Claude Code settings inventory
- Story 1.6: Claude Code settings in effect with provenance
- Story 1.7: Declared but not in effect, and folder trust, for Claude Code
- Story 1.8: Instruction, skill and agent files as metadata
- Story 1.9: Every launch directory, with consent for discovery
- Story 1.10: codex map through its own resolved view
- Story 1.11: goose map rebuilt from files
- Story 1.12: Claude desktop app and Claude general settings
- Story 1.13: Stale-key behavior and re-runnable probes
- Story 1.14: Environment labels
- Story 1.15: One egress port and a generated behavioral profile

## Requirements & Constraints

- Observation changes nothing: only known keys are read, credentials and trust records are never read or written, and credentials are reported by presence, never value.
- Every check stamps the harness version it assumes; every coded check is deterministic and tested.
- Per-person data never enters git; client names stay out of the repo.
- macOS first; Linux likely; Windows open.
- Every failure surfaces through one error type that names the step, the check, its status and at least one next action.

## Technical Decisions

- One binary with four modes (`usher`, `usher mcp`, `usher daemon`, `usher collector`) over one SQLite store; crates are named `usher-<area>`, adapters `usher-adapter-<harness>`. Later crates are added by the stories that need them.
- The toolchain is pinned to Rust 1.99.1 (the spine's version), not `stable`, in `rust-toolchain.toml` and `mise.toml`. Edition 2024; warnings are errors; `cargo deny check` gates supply chain.
- Adapters are tested against pinned harness versions inside scratch homes provided by `usher-testkit`; observation is checked by hashing harness file metadata before and after a run. These structural tests may gate merges; no health metric does.
- Harness facts live in harness knowledge (one TOML schema, embedded first, signed updates later), never in adapter code.

## Cross-Story Dependencies

- Story 1.1 gives every later story the workspace, the CI gates and `usher-testkit`; Story 1.2 builds the scratch-home kit and the observation invariant on that crate.
- Stories 1.5 to 1.12 (one per harness, Claude Code first) depend on harness knowledge (1.3) and the scratch-home invariant (1.2).
