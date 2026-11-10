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
