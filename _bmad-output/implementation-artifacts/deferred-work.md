# Deferred work

- source_spec: `_bmad-output/implementation-artifacts/spec-1-2-scratch-homes-and-the-observation-invariant.md`
  summary: `Probe::Missing` covers both an absent binary and one that fails `--version`; a broken pinned binary skips instead of failing.
  evidence: the matrix row requires Missing for a failing `--version`; a distinct variant is a spec change.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-2-scratch-homes-and-the-observation-invariant.md`
  summary: `--version` runs with no timeout, and `command()` inherits XDG_* and credential variables.
  evidence: `harness::probe` calls `.output()`; `ScratchHome::command` sets five variables only. Unverified severity: medium. Settled by the first adapter story that runs a real harness.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-2-scratch-homes-and-the-observation-invariant.md`
  summary: the watched set may miss goose data and state directories, and there is no exclusion list for volatile paths. `~/.claude.json` and the Claude Code managed settings directories are watched since Story 1.5 PR C.
  evidence: unverified against the research map; matters when stories 1.6 and later guard live real homes.
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
  summary: closed in PR C. The store open refuses a state directory owned by another uid, and the state directory comes from the account database, not `$HOME`.
  evidence: `nix` (feature `user`) in `usher-store` gives the uid and the account home; the resolver tests cover both.
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
- source_spec: `_bmad-output/implementation-artifacts/spec-1-2-scratch-homes-and-the-observation-invariant.md`
  summary: knowledge does not refuse two managed sources of one harness with the same rank over overlapping versions and platforms; the adapter reads two present ones as unknown.
  evidence: PR C reads a tie as "two managed sources share rank <n>" and never guesses; a structural check in `usher-knowledge` would catch it before a file is signed.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: the adapter names the item `claude-code.managed-control-keys` by id to find the keys that carry no policy; a knowledge field on the item would say so without the id.
  evidence: schema 3 has class `keys` and nothing marks which list is the control list; the full Claude Code key list is a later knowledge change.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: key names are listed sorted by name, not in file order, and values are parsed in memory and dropped.
  evidence: the workspace has no streaming JSON reader; reading file order or skipping a nested value without parsing it needs `serde` or the `raw_value` feature of serde_json.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: the host id is the output of `hostname`, run once per inventory.
  evidence: std has no hostname call and the dependency list for PR C is `nix` for the account-home resolver only; a later change can read it from the account database tooling.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: knowledge accepts `${CONFIG_DIR}` in a `version_from` launcher or layout; the binder rule checks source paths only. A knowledge refusal at parse time is the next step.
  evidence: nothing in the seeded knowledge does it. The version is read before it is known, so a `${CONFIG_DIR}` in a launcher resolves under the config directory as known before the version, which can give a guessed Known.
- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-claude-code-settings-inventory.md`
  summary: with `CLAUDE_CONFIG_DIR` set, Claude Code keeps its global state file under that directory, not at `${HOME}/.claude.json`. The seed names `${HOME}/.claude.json`, so on such a host the inventory reads a file the harness does not use there, which may be stale or absent. Hosts that do not set the variable are unaffected. The window runs from this change's merge to the follow-up that moves the path; nothing is released in it.
  evidence: the vendor's directory page says "If you set CLAUDE_CONFIG_DIR, every ~/.claude path on this page lives under that directory instead. The same applies to .claude.json, which otherwise sits at ~/.claude.json". The follow-up adds a global-state path token bound by `CLAUDE_CONFIG_DIR` with a default of `${HOME}`, and moves the global-state source to it.
- source_spec: `_bmad-output/planning-artifacts/epics.md`
  summary: `permissions.defaultMode` is seeded with its own merge rule, so the map shows every layer's value and says what applies is not decided. A narrower form would stay plain scalar unless a project or local layer sets `auto` or `bypassPermissions`, and only then read "not decided (<value> in <layer> does not take effect there; what applies instead is not stated)", which gives the common case its value back.
  evidence: the settings reference says `auto` and `bypassPermissions` "don't take effect from project or local settings" and does not say what applies instead; the narrower form needs a ruling of its own, or a probe that states the fallback.
- source_spec: `_bmad-output/planning-artifacts/epics.md`
  summary: the settings reference says that for conversations the VS Code extension starts, Claude Code reads only user, managed and `--settings` values. That launch context is not mapped yet, so the map's layers for it are not stated.
  evidence: the settings reference entry for `permissions.defaultMode`; story 1.9 maps the launch directories and is where the extension's launch context belongs.
- source_spec: `_bmad-output/planning-artifacts/epics.md`
  summary: the map reader names two not-observed items from names written in the reader: the organization default model, keyed to the listed key `model`, and the built-in classifier rules, keyed to the `autoMode.` key prefix. A knowledge field on the settings key would move both names out of code.
  evidence: the reader reads no such field today, and the knowledge schema has none; a later change adds the field to schema 6 and moves the two checks to it, and the red that pins each item stays as it is.
- source_spec: `_bmad-output/planning-artifacts/epics.md`
  summary: the harness can raise a settings error for a whole file when one value fails its schema; the reader does not run the schema, so the other keys in that file keep their rows and the file reads as readable.
  evidence: the vendor's settings page says a value the schema rejects can raise a settings error for the file; a later story that models it needs a probe that shows which values are rejected, and a status for a file the harness refuses.
