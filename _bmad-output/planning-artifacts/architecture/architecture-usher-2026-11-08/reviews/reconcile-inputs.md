---
title: 'Reconcile: inputs against the usher architecture spine'
spine: ../ARCHITECTURE-SPINE.md
created: '2026-11-08'
inputs:
  - prds/prd-usher-2026-11-03/prd.md
  - prds/prd-usher-2026-11-03/addendum.md
  - research/technical-harness-settings-observability-map-2026-11-02/research.md
---

# Reconcile: inputs against the spine

Scope: requirements, constraints and quiet qualities in the three inputs that the spine
neither decides, defers nor lists. Story-level product detail is left out. Contradictions
come first, because each one changes an existing AD rather than adding one.

Each item gives: source and location; what it says; why the spine needs it; suggested home.

## A. Contradictions between the spine and an input

### A1. Crash recovery acts without the person

- **Source:** PRD §5 "Changes on confirmation"; FR-64 "for both" (a run stopped part way:
  "the journal shows which steps finished, so the person can resume (FR-46) or roll back
  (FR-63)").
- **What it says:** finishing or undoing a half-done change is the person's choice, and a
  rollback is itself a change that needs confirmation.
- **Spine:** AD-3 says the next mode to take the lock "finishes or rolls back that change
  before doing anything else". Under AD-2 any mode may take the lock, so the daemon or MCP
  mode could complete or undo a harness write with nobody asked.
- **Why it matters:** this is the one place the spine lets usher change a harness file
  without confirmation.
- **Suggested home:** tighten AD-3. A crash marker blocks new changes and is surfaced; only
  a confirmed CLI action resumes or rolls it back. Daemon and MCP modes may detect a marker,
  never resolve one.

### A2. Backups evicted automatically versus kept until the person removes them

- **Source:** FR-63 last bullet ("kept until the person removes them; usher reports the
  space they use"); FR-67 (journal and backups kept unless the person asks to delete them).
- **Spine:** AD-7 evicts oldest backups to meet the 100 MB disk budget (Budgets row).
- **Why it matters:** the PRD gives the person sole say over deleting backups; the spine
  gives it to a budget. The "never one a recorded change could still roll back to" guard
  may mean nothing is ever evictable, since every journaled file write can be rolled back,
  which would make the budget unmeetable instead.
- **Suggested home:** tighten AD-7 and the Budgets row. Either the disk budget excludes
  backups and usher reports their size and asks, or eviction is a confirmed change. State
  which.

### A3. Observation through a harness call that writes state

- **Source:** PRD §5 "Observing does not change the harness" (zero state changes; a test
  before relying on any harness call); SM-5; addendum §1 (codex: whether `config/read`
  starts the app-server daemon or update loop is unknown); research §2 and executive
  summary (codex resolved view comes only from the app-server).
- **Spine:** AD-9 says codex app-server "writes state files into the codex home" and then
  allows it when a person asks.
- **Why it matters:** the PRD invariant has no "when a person asks" exception. If the
  app-server writes state, FR-1 for codex either rebuilds from files or needs an operator
  exception; if it does not, AD-9's hazard example is wrong. The codex adapter's main read
  path depends on the answer.
- **Suggested home:** tighten AD-9 to the PRD rule (a call is used for observation only
  after a scratch-home test shows zero writes), and add the codex app-server state-write
  test to Deferred as a probe the codex adapter waits on, with file rebuild as the fallback.

### A4. Agent-proof confirmation is not agent-proof

- **Source:** PRD §5 (confirm "through a prompt the agent cannot answer for them"); §7.1
  thin in-harness surface; addendum §7 row "An in-LLM agent answering its own confirmation
  prompt: refused".
- **Spine:** AD-4 sends apply to "the CLI on a terminal". An agent in Claude Code or codex
  runs shell commands, so it can run `usher apply <id>` itself.
- **Why it matters:** as written, AD-4 does not meet the confirmation rule or its test.
- **Suggested home:** tighten AD-4 with the mechanism: for example the confirm step reads
  from the controlling TTY and refuses when stdin is not a TTY or the parent process is a
  known harness, with a per-proposal code shown only on that TTY, or an OS-level dialog.
  Name the test.

### A5. Environment identity is narrower than the glossary

- **Source:** PRD §3 "Environment" (one harness, one launch context, one host, one person or
  seat; a launch context is a directory or a scheduled or headless launch with its own
  environment variables); FR-5 (headless launches as their own contexts).
- **Spine:** Consistency Conventions, Identifiers: "environments keyed by harness id plus
  absolute directory".
- **Why it matters:** a launchd job and an interactive shell in the same directory collide
  under that key, and reports from several hosts (FR-18, FR-41) cannot tell environments
  apart without host and person.
- **Suggested home:** tighten the Identifiers convention: harness id, host id, person or
  seat, launch context (directory plus a context discriminator).

### A6. Mixed-file backups copy credential bytes

- **Source:** FR-64 "Mixed files are written by key" ("The backup of such a file stays in
  the owner-only store"); FR-63 (a credential is recorded as present, never by value).
- **Spine:** AD-8 says credential keys are skipped "without reading their values"; AD-3 step
  2 backs up every file the change touches, which for `~/.claude.json` copies
  `oauthAccount` into the backup store and hashes it.
- **Why it matters:** the PRD accepts the backup in an owner-only store; the spine's AD-8
  wording forbids what AD-3 requires. Implementers will resolve it differently.
- **Suggested home:** tighten AD-8 to the PRD's form: credential values are never parsed,
  logged, journaled, reported or written; a whole-file backup of a mixed file is allowed only
  in the 0700 store and is never exported, reported or included in the JSON-lines journal
  export.

### A7. FR numbers mislabeled in the spine

- **Source:** PRD §4.9 (FR-36 is maintainer docs); §4.2 and §4.3 (FR-10 process table, FR-11
  install, FR-12 device management are not watcher features).
- **Spine:** AD-14 binds "FR-29, FR-36 to FR-39" and the capability map row "Reporting and
  the light collector (FR-29, FR-36 to FR-42)" treats FR-36 as reporting. AD-9 and the
  capability map place FR-10 to FR-13 in `usher-daemon`, but install (FR-11) is a CLI change
  and FR-10, FR-12 are on-demand reads.
- **Suggested home:** correct the binds. The capability map also omits FR-9, FR-14, FR-15,
  FR-17, FR-18, FR-24 to FR-28, FR-43 to FR-45, FR-47, FR-48, which are in `binds`.

### A8. "If the person names ~" versus no home-wide sweep yet

- **Source:** PRD §5 "Local observation from install" ("no home-wide sweep runs before open
  question 3 is settled"); FR-5 last bullet.
- **Spine:** AD-10 "Usher never searches `~` unless the person names it", which permits the
  sweep on request.
- **Suggested home:** either Deferred (Q3, home-wide sweep) or record AD-10 as the proposed
  answer to Q3 with an [ASSUMPTION] tag, so the operator rules on it.

### A9. Self-update is bound but its channel is deferred

- **Source:** FR-44 ("Usher updates itself from a signed feed and verifies the feed before
  it installs anything"; FR-63, "usher's own update keeps the previous binary").
- **Spine:** binds FR-44; Deferred puts distribution behind Q11; AD-12 signs only harness
  knowledge.
- **Suggested home:** either extend AD-12's key scheme to the self-update feed and keep the
  previous binary as the undo, or move the self-update clause of FR-44 to Deferred with Q11.

## B. PRD (prd.md)

### B1. No port for delegated runs, processes or device management

- **Source:** §4.16 description (two kinds of change: a write, or a run delegated to brew,
  mise, git, sideshow, a vendor installer); FR-9, FR-10, FR-11, FR-12, FR-44, FR-45, FR-46,
  FR-47, FR-48, FR-63 (delegated run records installer paths before and after), FR-64
  "for delegated runs".
- **What it says:** usher drives existing installers, reads the process table and device
  management profiles, clones repos, and journals each run with kind-specific undo.
- **Why the spine needs it:** the driven ports are harness adapters, store, profiles and
  knowledge only. AD-3 is written for file writes (back up the file, read back). Installers,
  git, sideshow, eos, `ps`/process inspection and MDM profile reading have no port, so they
  will be called from `usher-cli` directly, which breaks the hexagonal rule and skips AD-3.
- **Suggested home:** new AD: an `Executor` (or `Installer`) port with one adapter per tool,
  arguments as a list, pinned-source rule for piped installers, before and after path
  snapshots, and undo by kind (reinstall pinned version, move a clean clone aside, manual
  step). Plus a `HostInspector` port for processes and MDM profiles. Tighten AD-3 to say
  which steps differ for a delegated run.

### B2. Write-safety guards beyond the transaction

- **Source:** FR-64 "for writes usher makes itself"; addendum §7 rows 1 to 7, 8 to 13, 18.
- **What it says:** refuse empty, unexpanded, root, home, glob and `..` targets; a per-kind
  allowlist of paths; symlinks written through when the target is on the list, never
  replaced; stop if the dry run listed fewer targets; check free space; temp file in the same
  directory, parse in the file's format, fsync, rename; keep permissions, ownership,
  encoding, line endings and comments; stop if the target hash changed since the dry run;
  never delete (removal moves to the backup store).
- **Why the spine needs it:** AD-3 and AD-5 give order and editing method but not the target
  guard or the atomic write. Claude Code settings files hot-reload (research §1), so a
  non-atomic write is read half done. These must live in one place every adapter and the
  executor share.
- **Suggested home:** new AD "Target guard and atomic write" in `usher-core` (pure path
  checks) and `usher-store` or a shared writer crate (temp, fsync, rename), called from AD-3
  step 3. Add the dry-run hash to AD-3 step 1.

### B3. Journal durability and the stop rule

- **Source:** FR-63 first bullet ("written and flushed to disk before the change starts";
  "A journal that cannot be read or written stops every change"); FR-63 rollback bullet
  (dependent later entries undone first or the rollback stops; reverse order; hash check
  per target); FR-64 (stale lock released and journaled; second run waits for the person).
- **Why the spine needs it:** AD-7 chooses SQLite but not its durability settings (WAL,
  `synchronous=FULL`), and AD-2 relies on `flock`, which releases on process death silently,
  so the PRD's "released and journaled" needs an explicit owner record beside the lock. The
  dependency rule needs a field in the journal schema.
- **Suggested home:** tighten AD-7 (durability mode, journal unreadable means no change) and
  AD-2 (lock owner pid recorded; a found stale record is journaled; a second run waits on
  the person, not on a timeout).

### B4. Derived state and named fields only

- **Source:** §5 "Derived state, not content" and "Session records: named fields only"
  (marker-text test); FR-4; FR-43 (presence, path, size, mtime, key names; never content);
  FR-5 (environment variables by name; a value only where knowledge marks it a setting);
  FR-26 and FR-27 (in MVP).
- **Why the spine needs it:** AD-8 covers credentials only. Nothing binds that instruction
  files, prompts and session text are never parsed, or that environment values are dropped.
  FR-26 cache reuse is in MVP scope but no port reads session records and no home holds the
  model price table ("unpriced, not dropped").
- **Suggested home:** new AD "Derived state only", with the marker-text test named as a
  structural gate, and a session-record reader on `HarnessAdapter` (named fields only). Add
  the price table to harness knowledge or to Deferred.

### B5. Consent, enablement source and the two-way channel

- **Source:** FR-29 (DO_NOT_TRACK keeps reporting off and usher says so; published "we do not
  collect" list); FR-37 (reporting turned on by the person or by the organization's device
  management, recorded and told); FR-39 (per-collector disclosure before the first report;
  withdrawal stops at once; consent events recorded); §5 "Local observation from install"
  (first open tells the person it has been observing and shows what it looked at); §5
  "Two-way channel".
- **Why the spine needs it:** AD-14 gives the default only. How a managed preference turns
  reporting on, where consent lives, how withdrawal interrupts an in-flight send, and the
  record of what usher looked at before first open are cross-mode state the CLI, daemon and
  MCP modes must agree on.
- **Suggested home:** tighten AD-14 (DO_NOT_TRACK, managed enablement read from a named
  managed-preference domain, consent per collector in the store, withdrawal checked before
  each send) and add an observation log to AD-7's store contents.

### B6. Collector host identity, access and retention

- **Source:** FR-40 (per-host credential the collector can revoke and the host re-mint;
  ADR-009 audience test); FR-41 (person sees their own first; per-person drill-down only after
  a recorded team decision; aggregate by default; no ranking view); FR-42 (team-set retention,
  short default, 90 days assumed, deletion record kept); §4.10 feature NFR (per-person data).
- **Why the spine needs it:** AD-13 decides schema and storage only. Host authentication,
  who may query what, and retention deletion are the collector's security model and shape
  its tables.
- **Suggested home:** new AD for the collector: host keypair enrolled at consent, collector
  stores public keys and can revoke one; query roles (self, team aggregate, drill-down gated
  on a stored team decision); retention job with tombstone rows. The client side keeps the
  private key under AD-8's rules.

### B7. Path tokens left to architecture

- **Source:** FR-20 and its [ASSUMPTION] ("the token set and pattern syntax are the
  architecture step's"); §10 Assumptions Index; FR-15 (tokens resolved on each side before
  comparing).
- **Why the spine needs it:** the PRD hands this to architecture by name. It shapes the
  profile fragment format in AD-11, which adapters merge natively, so tokens inside a native
  fragment need a syntax that harness formats tolerate.
- **Suggested home:** tighten AD-11 with the token set (home, config dir, repo root, user
  name) and syntax, or list it under Deferred with an owner.

### B8. Profile shape gaps

- **Source:** §3 "Entry" (label, value, harness version range, note); FR-17 (entry with no
  range marked unchecked); FR-45 (a profile declares tools and toolchain, possibly via a mise
  file); FR-49 and §3 "Personal layer" (marks per entry, never written); Q8 (who signs
  profiles).
- **Why the spine needs it:** AD-11 fixes a native fragment per harness, which has no place
  for a per-entry note, version range or label, and `profile.toml` has no tool declaration.
  "A profile's version is its commit" leaves file and directory sources without a version,
  which FR-51 catch-up needs. Personal marks have no stated home. Profile signing is not
  deferred; only knowledge signing is.
- **Suggested home:** tighten AD-11 (sidecar metadata per entry, tool declarations, a
  content-hash version for non-git sources, personal marks in the local store keyed to
  entry) and add profile signing to Deferred under Q8.

### B9. Never write a managed layer, and support level per harness

- **Source:** FR-49 ("never writes a managed or device-management layer"); §5 ("usher is never
  the sole configuration authority"); FR-30 third bullet and §3 "Support level" (capability
  list per harness version, so a missing capability reads as unsupported); FR-3 (version
  stamp, method and blind spots on every map); FR-16 (a hazard check ships only after its
  test passes on the current version).
- **Why the spine needs it:** AD-6's trait has read, list writable keys and write, but no
  support level, no per-reading method or blind-spot list, and no rule that managed layers
  are never writable whatever knowledge says. A forged or mistaken knowledge item could mark
  a managed key writable; AD-12 checks the signature, not this.
- **Suggested home:** tighten AD-6: the trait returns a support level and each reading
  carries method, version stamp and blind spots; managed and MDM layers are unwritable in
  code, not in data. Hazard checks gate on a recorded probe result for the detected version.

### B10. Daemon lifecycle and its removal

- **Source:** FR-67 (after removal no usher process, login item, scheduled job or OTLP
  endpoint remains; a test checks); §5 "Local observation from install" (silent install by
  device management); §5 "Footprint" (usher reports its own use).
- **Why the spine needs it:** how the daemon is installed and started (launchd agent, login
  item) decides what removal must find, and whether an MDM silent install registers it.
  Self-reporting of CPU, memory and disk is a footprint requirement without a home.
- **Suggested home:** new convention or AD: one launchd agent with a fixed label, registered
  and removed only through AD-3; a `usher status` self-report of budget use.

### B11. Attention budget numbers

- **Source:** §5 "Attention budget" (prompts per session have a number set in architecture,
  Q25); FR-13 (a person sets which kinds interrupt).
- **Why the spine needs it:** the Budgets table sets token and notice budgets but not prompts
  per session, which Q25 asks architecture to set.
- **Suggested home:** add a row to Budgets.

### B12. Independence from siblings

- **Source:** §5 "Independence" (sideshow, marvel, wardrobe, stagekeeper, critic, beadle work
  without usher and usher without them); FR-47 (drives sideshow or eos, does not
  re-implement).
- **Why the spine needs it:** without a stated rule, a sideshow or eos adapter may become a
  hard dependency of bootstrap.
- **Suggested home:** convention: every sibling integration is an optional executor adapter;
  absence is a finding, never an error.

### B13. Per-person data stays out of git

- **Source:** §5 "Per-person data. Never enters git. Client names stay out."
- **Why the spine needs it:** profiles are captured to files that experts commit to team repos
  (AD-11), and local reports are files. Capture and report writers need the check.
- **Suggested home:** convention under Logging and a capture-time check in `usher-profiles`.

### B14. Credential scan

- **Source:** §5 "Credentials: presence, never value" (scan off by default, location and type
  only); addendum §5.
- **Why the spine needs it:** it is MVP behavior touching credential files, the area AD-8
  governs, and it is neither decided nor deferred.
- **Suggested home:** tighten AD-8 (scan off by default, pattern match without retaining
  matched text) or Deferred.

## C. PRD addendum (addendum.md)

### C1. Disputed claims missing from Deferred

- **Source:** §1.1 "Claims to test": Claude Code `$defaults` (the FR-16 hazard check waits on
  it); codex project skills in untrusted repos; codex startup warning for ignored project
  keys.
- **Spine:** Deferred lists only codex device-management precedence and `GOOSE_MODE`.
- **Suggested home:** add the three to Deferred with the adapter or check each one blocks.

### C2. Per-harness observer hazards not bound

- **Source:** §1 table, hazards column: `claude auto-mode` can reset settings; `/doctor` can
  apply fixes; `claude doctor` reads cwd settings without a trust prompt; `purge`, `install`,
  `respawn`, plugin `update` mutate; `codex update` mutates; files hot-reload.
- **Why the spine needs it:** AD-6 says adapters "declare hazards" but the spine names no
  never-call list and no place it lives. `claude doctor` reading untrusted cwd settings is a
  trust hazard distinct from a state write.
- **Suggested home:** tighten AD-6: a deny list of harness subcommands in harness knowledge,
  enforced by the executor port so no adapter can call one; `claude doctor` in an untrusted
  directory is never run.

### C3. Flush-on-reconnect and keypair auth

- **Source:** §5 prior-art rulings (accept a typed protocol with local cache and
  flush-on-reconnect; adapt keypair auth, prefer the organization's directory or device
  management). Bears on FR-37 to FR-40.
- **Why the spine needs it:** there is no outbound queue in the spine; whether reports queue
  while the collector is unreachable, and whether withdrawal purges the queue, is a store
  and consent decision.
- **Suggested home:** tighten AD-13 or AD-14: a local outbox in the store, purged on
  withdrawal; keypair host auth per B6.

### C4. Change-safety test catalog as a gate

- **Source:** §7 (one test per row, injected against a scratch home); PRD §5 (structural, may
  gate).
- **Why the spine needs it:** the spine names no test harness. A scratch-home fixture shared
  by adapters, the executor and the collector is a cross-crate concern, and these tests are
  the ratified gate.
- **Suggested home:** convention: a `usher-testkit` crate (scratch home, failure injection,
  marker-text fixture) and a statement that addendum §7 and SM-5 tests are required checks.

### C5. Desktop app readability is unprobed

- **Source:** §1 table row "Claude desktop app" (unknown until probed); PRD FR-7 [ASSUMPTION];
  Q12, Q17.
- **Spine:** AD-8 names `oauth:tokenCache` in the desktop `config.json`, and AD-10 reads
  desktop-app trusted folders and `git-worktrees.json`, as if the layout were known.
- **Suggested home:** Deferred: the FR-7 probe, with the claude-desktop adapter at inventory
  support level until it passes.

## D. Technical research (research.md)

### D1. Slow and fast layers of harness knowledge, keyed to detected version

- **Source:** Recommendation 3 (split scopes, precedence, paths and merge rules from keys,
  defaults and value sets; re-verify the fast layer per release; key it to the version
  install health detects); Cross-dimension insight 2.
- **Why the spine needs it:** AD-6 and AD-12 make knowledge one signed file with version
  ranges, but do not say how a detected version selects items or that a version change
  triggers re-verification marks (FR-32).
- **Suggested home:** tighten AD-12 or add an AD on the knowledge schema: item class (slow or
  fast), version range, reference, verified date, re-check date, stale-key behavior (FR-33).

### D2. Knowledge is stale on the day it ships

- **Source:** Staleness map (14 of 27 claims past their window on the day of writing; earliest
  re-check already due); PRD Q4.
- **Why the spine needs it:** a first release will carry items already past their re-check
  date. FR-32 marks such findings, but the spine has no rule on the initial knowledge file
  (embedded in the binary or fetched) or on where knowledge updates come from (AD-14 lists
  "a knowledge update" as a network call with no source).
- **Suggested home:** tighten AD-12: a knowledge file embedded at build, a named update source,
  and findings resting on overdue items marked.

### D3. Paths depend on the launch environment

- **Source:** §1 (`CLAUDE_CONFIG_DIR` relocates the user home; env vars beat their settings
  key; flags such as `--model` override per key); §2 (`CODEX_HOME`); §3 (`GOOSE_PATH_ROOT`
  relocates config, data and state; env var by uppercased key beats any file).
- **Why the spine needs it:** layer paths cannot come from knowledge alone; they resolve
  against each launch context's environment, and running sessions' flags are visible only
  through process arguments. This ties A5 (environment key) to the adapter trait.
- **Suggested home:** tighten AD-6: adapter reads take a launch context (directory, env,
  argv where known) and knowledge stores paths as templates over relocation variables.

### D4. Resolved state where one exists; rebuild only where none does

- **Source:** Recommendation 4; Executive summary "what drives it" (2); §1 (`remote-settings.json`
  may differ from what is in effect); §2 (`config/read` without `cwd` omits project layers).
- **Why the spine needs it:** AD-9 says the daemon "rebuilds values from files", which for codex
  is the weaker source, and the spine has no rule that a rebuilt value is checked against the
  harness's partial views (`/permissions`, `claude auto-mode config`) and labeled rebuilt.
- **Suggested home:** tighten AD-6: each reading records method; where a resolved view exists
  it wins and the rebuild is a cross-check; quirks such as `cwd` required for codex project
  layers live in knowledge.

### D5. goose writes only its user file; system and additional files are admin paths

- **Source:** §3 (goose writes only the user file; the system file is the only admin path and
  is undocumented; secrets resolve from env, keyring, then `secrets.yaml`).
- **Why the spine needs it:** placement (FR-24) and AD-8's credential list. `secrets.yaml` and
  the keyring are credential stores not named in AD-8.
- **Suggested home:** knowledge data for placement; add goose `secrets.yaml` and keyring to
  AD-8's named examples, with the codex auth file.
