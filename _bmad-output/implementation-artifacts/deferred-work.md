# Deferred work

- source_spec: `_bmad-output/implementation-artifacts/spec-1-2-scratch-homes-and-the-observation-invariant.md`
  summary: `Probe::Missing` covers both an absent binary and one that fails `--version`; a broken pinned binary skips instead of failing.
  evidence: the matrix row requires Missing for a failing `--version`; a distinct variant is a spec change.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-2-scratch-homes-and-the-observation-invariant.md`
  summary: `--version` runs with no timeout, and `command()` inherits XDG_* and credential variables.
  evidence: `harness::probe` calls `.output()`; `ScratchHome::command` sets five variables only. Unverified severity: medium. Settled by the first adapter story that runs a real harness.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-2-scratch-homes-and-the-observation-invariant.md`
  summary: the watched set may miss `~/.claude.json` and goose data and state directories, and there is no exclusion list for volatile paths.
  evidence: unverified against the research map; matters when stories 1.5 and later guard live real homes.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-4-signed-harness-knowledge-updates-without-a-client-release.md`
  summary: the highest accepted knowledge sequence is not persisted, so a restart resets rollback protection to the embedded sequence.
  evidence: `Knowledge::update` compares against the in-memory file; the store that can hold the floor arrives with Story 1.5.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-4-signed-harness-knowledge-updates-without-a-client-release.md`
  summary: nothing outside `usher-knowledge` calls `Knowledge::update` or reports the updated knowledge version.
  evidence: `SupportReport` and the usher binary still read the embedded file; the caller is a later story.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-4-signed-harness-knowledge-updates-without-a-client-release.md`
  summary: the CI step that keeps ed25519-dalek and blake2 out of `usher` will need a scoped exception when host keys (AD-13) put ed25519-dalek in a shipping crate.
  evidence: the spine Stack table lists ed25519-dalek for host keys; maybe-false until that story exists.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: a later prune of inventory rows needs a migration that relaxes the ABORT triggers on `inventory_run` and `inventory_source`.
  evidence: PR B makes both tables insert-only by trigger (architect ruling 01M7GFSGTF); no story prunes them yet, and the disk budget (AD-7) is the first reason one would.
