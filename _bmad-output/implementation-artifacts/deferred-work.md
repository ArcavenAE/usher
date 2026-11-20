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
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: the store does not check that the state directory's owner is the current uid; a red for it is due in PR C.
  evidence: std has no safe getuid, and PR B adds one dependency crate, rusqlite, and no serde_json; the safe-API crate that PR C adds for the account-home resolver also gives the uid (architect answer 01M7GG12SP).
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: closed. The frozen Never line allows creating the missing standard parents of the state directory at 0700; the operator ruled it on 2026-11-15 (relayed by director 01M7GG8BXE) and PR B creates them.
  evidence: the Renegotiated note in the spec's frozen block; the store tests for missing and existing parents.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: the seat part `toolchain` has no seed item. The files that declare a toolchain (`mise.toml`, `rust-toolchain.toml`, `.tool-versions`) belong to other tools, not to Claude Code, and reading their contents is outside the settings-file rule. A later story can seed a presence-only item once each file is cited to its own tool's documentation.
  evidence: no Claude Code page names these files; the mise and rustup documentation are the owning references.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: the seat parts `packs` and `fleet-client` have no seed item, because no path is cited yet. Ask the sideshow repository for its install directory, the beads repository for its client environment file, and the marvel and director repositories for their client config, each cited to that repository's own document at a commit.
  evidence: the schema 4 `seat` class supports a `listing` read for a directory and a `presence` read for a file; only the citations are missing.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: MCP registrations in `~/.claude.json` (user and local scope) are not seeded. The vendor page says user-scoped servers "are stored in `~/.claude.json`" without a key path, and local-scope servers sit under that project's path, keyed by the project directory. A seat key path has one `*` level and cannot name the launch directory, so `projects.*` would read the servers of every project.
  evidence: the vendor's MCP page; a later change can add a launch-directory key path to the `names` read, or settle the user-scope key path with a probe.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: the key name `oauthAccount` rests on an observation on one host on 2026-11-20 (membership only), not on a vendor page, which says only that `~/.claude.json` holds account details. The seed item cites that page and says so in its summary.
  evidence: a later probe can settle the name, after which the item's reference kind becomes `probe`.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: which directory Claude Code loads `.mcp.json` from in a subdirectory launch is not stated. The vendor pages say only "project root" and "at the root". The seat inventory lists both the launch directory and the repository root, and says which of the two is loaded is not stated.
  evidence: the MCP and directory pages; a probe with a positive control can settle it.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: the local settings file has exceptions that knowledge cannot state yet: outside git, a repository root that is the home directory, Windows, and a root the user does not own, where the file stays beside `.claude/settings.json`. In a worktree the file is at the main checkout's root, not at the repository root of the worktree. Both belong with the launch-directory work.
  evidence: the settings page says "If you start Claude Code in a subdirectory of a git repository, it reads and writes that file at the repository root", and that Claude Code "reads the shared `.claude/settings.json` from the session's primary working directory, so to use a file committed at the repository root, start Claude Code there."
