# Rubric review: usher architecture spine, 2026-11-08

Reviewed: `ARCHITECTURE-SPINE.md` (draft, 2026-11-08) and its `.memlog.md`, against PRD
sections 4, 5, 7.1 and 9 and the PRD addendum (`prd-usher-2026-11-03`).

## Verdict

Not ready to hand to feature builders. The settings-write path (AD-2, AD-3, AD-5, AD-7) and
the observation rules (AD-8 to AD-10) are well fixed. Four gaps let independently built
features diverge: the in-harness agent can apply its own proposal through the CLI; delegated
runs (installers, git, sideshow) have no port or transaction rule, although about a third of
the MVP features depend on them; the spine has no testing strategy for adapters against
harness versions; and the operational envelope (usher's own upgrade, store migration, daemon
install and removal, collector schema versions) is mostly absent.

## Checklist summary

| Check | Result |
| --- | --- |
| Fixes the real divergence points, misses none | Partial. Misses delegated runs, path tokens, knowledge file schema, session-record parsing, environment identity, collector host credential |
| Every Rule is enforceable and prevents its divergence | Partial. AD-4 is bypassable; AD-3 crash recovery conflicts with AD-4; AD-5 conflicts with AD-9 and FR-64; AD-7 backups conflict with AD-8 |
| Nothing under Deferred lets two units diverge | No. Deferring distribution leaves FR-44 self-update (MVP) undefined |
| Covers PRD 7.1 capabilities | Partial. The capability map has no home for FR-9, FR-11, FR-12, FR-14, FR-15, FR-17, FR-18, FR-24 to FR-28, FR-30, FR-31, FR-43 to FR-48 |
| Every owned dimension decided, deferred or open | No. Testing strategy, usher upgrades, store migration, daemon lifecycle, CI and release pipeline are none of the three |

## Findings

### F1. An agent can apply its own proposal by running the CLI

- **Location:** AD-4.
- **Problem:** The rule says a proposal is applied "only from the CLI on a terminal", but an
  agent inside Claude Code or codex has a shell tool and can run `usher apply <id>` itself. The
  rule names a surface, not a check, so it does not prevent the divergence it states (an agent
  confirming its own change). The addendum §7 row "An in-LLM agent answering its own
  confirmation prompt: Refused" has no mechanism behind it.
- **Fix:** Make the confirmation a check the code can enforce and test. For example: `apply`
  requires an interactive controlling terminal (stdin and stdout are TTYs, and the prompt is
  read from `/dev/tty`), refuses when it detects it was launched under a harness process tree
  or with harness session variables set, and asks the person to type a short code shown only
  on that terminal. State which of these the MVP uses and name the addendum §7 test that
  proves it.
- **Consequence if unfixed:** The CLI and MCP teams each pick their own idea of "a prompt the
  agent cannot answer", and the PRD §5 confirmation rule holds only by convention.
- **Tier:** critical.

### F2. Delegated runs have no port and no transaction rule

- **Location:** Design Paradigm table, AD-3, Capability Map.
- **Problem:** FR-63 and FR-64 define two kinds of change: a file write and a run delegated
  to brew, mise, git, sideshow or a vendor installer. The spine covers only the first. There is
  no driven port for installers or package managers, no rule for how a delegated run is
  journaled (known paths before and after, observed difference, undo by pinned reinstall,
  clone moved aside only when clean), and no place for argument-list execution and pinned
  sources. FR-9, FR-11, FR-12, FR-44 to FR-48 and FR-50 depend on this. The map lists
  "Watching and notices (FR-10 to FR-13)" in `usher-daemon`, which puts FR-11 (install) in the
  one mode that may not change anything.
- **Fix:** Add an `Installer` (or `ToolManager`) port with one adapter per tool manager
  (`usher-installer-brew`, `-mise`, `-git`, `-sideshow`, vendor), and an AD that gives the
  delegated-run transaction: intent, snapshot of the installer's known paths, run with an
  argument list, snapshot after, outcome with exit status and difference, undo kind. Give FR-9
  to FR-12 and FR-44 to FR-48 their own rows in the map. Say whether the journal lock is held
  for the whole of a long install.
- **Consequence if unfixed:** Bootstrap, update, restore and repository help are built with
  three different ideas of what a run records and how it is undone, and FR-63 rollback cannot
  treat them as one journal.
- **Tier:** critical.

### F3. No testing strategy for adapters against harness versions

- **Location:** missing section (the spine has no Testing heading).
- **Problem:** The harnesses ship every few days, and the PRD makes several things structural
  tests that may gate: zero harness state changed by observation (SM-5), no egress by default
  (FR-29), the session-record marker test (§5), the addendum §7 failure injections, and
  FR-35 probes with a positive control. The spine does not say how adapters are tested
  against real harness versions: recorded fixtures or live binaries, which versions CI pins,
  how a scratch home is built, how the "observation changes nothing" test measures change
  (for example a hash of the harness home before and after), or where probes live and how
  their results reach the knowledge file.
- **Fix:** Add a testing AD: per adapter, a fixture set keyed by harness version (captured by
  the FR-34 helpers) for unit tests, plus a live suite that installs pinned harness versions
  into a scratch home in CI and runs the observation-invariant and egress tests; one shared
  test harness crate (`usher-testkit`) that builds scratch homes, injects addendum §7 failures,
  and hashes harness homes; and the rule that a hazard check ships only when its live test
  passes on the current version (FR-16).
- **Consequence if unfixed:** Each adapter author invents a test approach, the SM-5 and FR-29
  invariants are tested differently per harness or not at all, and a harness release breaks
  adapters silently.
- **Tier:** critical.

### F4. Usher's own upgrade path is undefined

- **Location:** Deferred ("Distribution ... blocked while the repo is private"), AD-1, AD-7.
- **Problem:** FR-44 is in the MVP: usher updates itself from a signed feed, verifies it
  first, and keeps the previous binary for undo. Deferring all of distribution leaves the feed,
  its signature and the rollback undecided. Separately, one binary in four modes shares one
  SQLite store, but nothing says how the store schema migrates across usher versions, what
  happens when a daemon still running the old binary meets a store the new CLI migrated, or
  whether a downgrade (FR-63 undo of usher's update) can read a newer store.
- **Fix:** Split the deferral: distribution channels (tap, installer, notarization) can wait on
  Q11, but decide now the self-update feed format and its signature (the AD-12 minisign key
  could serve), where the previous binary is kept, a store schema version with forward-only
  migrations run under the journal lock, and the rule that a mode refuses a store newer than
  it knows and that the CLI restarts the daemon after an update.
- **Consequence if unfixed:** The first self-update or schema change corrupts or locks out the
  store for one mode, and FR-44 undo is impossible to build consistently.
- **Tier:** high.

### F5. Daemon install, start and removal are not decided

- **Location:** AD-1, Structural Seed deployment paragraph, Capability Map (FR-67).
- **Problem:** The PRD requires observation from install, including a silent install by device
  management (§5), and removal that leaves no process, login item or scheduled job (FR-67).
  The spine never says how the daemon is installed and started on macOS (LaunchAgent per user,
  LaunchDaemon, login item), who registers it, or how it is found again for removal.
- **Fix:** Decide the macOS service form (a per-user LaunchAgent plist with a fixed label is the
  usual choice), that its registration and removal are journaled changes under AD-3, and how a
  device-management install registers it for each user.
- **Consequence if unfixed:** Install, bootstrap and removal features register the daemon in
  different ways, and the FR-67 "nothing remains" test has no fixed list to check.
- **Tier:** high.

### F6. Crash recovery lets a non-CLI mode finish a change

- **Location:** AD-3 last sentence, AD-4, AD-9.
- **Problem:** "The next mode to take the lock finishes or rolls back that change." The next
  mode may be MCP (which AD-4 forbids from applying) or the daemon (which AD-9 says changes
  nothing). Finishing an interrupted change is an apply without a fresh confirmation.
- **Fix:** Only the CLI resolves a crash marker, and only after showing the person the
  unfinished step and offering resume or roll back. MCP and daemon modes that find a marker
  report it and take no lock for changes.
- **Consequence if unfixed:** A daemon restart applies half of a change the person may have
  abandoned, breaking the PRD §5 confirmation rule.
- **Tier:** high.

### F7. Writing through codex app-server writes outside the dry run

- **Location:** AD-5, AD-9, memlog constraint on app-server side effects.
- **Problem:** AD-5 prefers the codex app-server writer. The memlog records that one stdio
  app-server run created `installation_id`, state, logs and sqlite files in the codex home.
  FR-64 says a write that touches more targets than its dry run listed stops, and AD-3 backs up
  only the files the change touches. A native write therefore breaks FR-64 and leaves side
  files with no backup or undo.
- **Fix:** Either run the app-server writer against a known home and list its side files in
  the dry run and backups, or prefer the file-edit path for codex until a probe shows a writer
  call with no side writes. Record the probe in harness knowledge and make the choice per
  harness version.
- **Consequence if unfixed:** The codex adapter passes its own tests while failing the FR-64
  scope test, and rollback misses files.
- **Tier:** high.

### F8. Backups of mixed files hold credentials that AD-8 says are never read

- **Location:** AD-3 step 2, AD-7, AD-8.
- **Problem:** AD-3 backs up every file a change touches. A write to `~/.claude.json` or the
  desktop `config.json` (both writable mixed files) copies `oauthAccount` or
  `oauth:tokenCache` into usher's backup store at rest. AD-8 says these values are never read,
  and SOUL §3 (ADR-009, amended 2026-10-19) limits third-party bearer material at rest. Also,
  "skip credential keys without reading their values" cannot be met literally when a JSON
  parser reads the file; the rule needs to say what "read" means.
- **Fix:** Decide one of: for mixed files, store a key-level undo record (the prior value of
  the changed keys only) instead of a whole-file backup; or keep whole-file backups and record
  an ADR-009 analysis that names the store as acceptable custody. Reword AD-8 to "credential
  values are never held in domain types, stored, logged, journaled or reported", which a test
  can check.
- **Consequence if unfixed:** The store and the adapters disagree on what a backup holds, and
  usher keeps bearer tokens at rest without a ruling.
- **Tier:** high.

### F9. Path tokens and pattern syntax are not decided

- **Location:** AD-11, Consistency Conventions.
- **Problem:** The PRD (FR-20 assumption) hands the token set and pattern syntax to this step.
  Capture (FR-20), comparison (FR-15 resolves tokens on each side), placement (FR-24) and
  apply all depend on one definition. The spine is silent.
- **Fix:** Add a convention: the token set (for example `{home}`, `{harness_config}`,
  `{repo_root}`, `{user}`), the syntax, resolution order, and that resolution lives once in
  `usher-core`, with adapters supplying values per environment.
- **Consequence if unfixed:** Profiles captured on one host do not compare or place correctly
  on another, because capture and comparison tokenize differently.
- **Tier:** high.

### F10. The harness-knowledge file has no schema

- **Location:** AD-6, AD-12.
- **Problem:** AD-6 moves paths, precedence, writable keys, credential keys, version ranges,
  stale-key behavior and support levels into the knowledge file, and AD-12 says how it is
  verified, but not its format or item shape. FR-30 to FR-33 require each item to carry a
  reference, a verification date, a re-check class and per-version behavior, and FR-34
  helpers and FR-35 probes must write the same shape the adapters read. Nor is it said where
  the client gets an update or what baseline ships with the binary.
- **Fix:** Fix the file format (TOML or JSON), a schema with a version, the required fields per
  item, the structural check that fails an item without a reference or date (FR-30), the
  update source, and that a verified baseline is embedded in each release.
- **Consequence if unfixed:** Four adapter crates, the knowledge crate and the release helpers
  each assume a different item shape.
- **Tier:** high.

### F11. Collector and client versions are not negotiated, and host credentials are not decided

- **Location:** AD-13.
- **Problem:** "A report carrying any field outside that schema version is rejected whole" is
  enforceable, but it does not say whether the collector accepts several schema versions.
  Hosts update at different times, so a collector that knows only its own version rejects
  either old or new clients. FR-40 (per-host credential the collector can revoke and the host
  can re-mint) is bound but not decided; the addendum §5 points at keypair auth.
- **Fix:** State that the collector accepts every schema version from N back to a declared
  floor, each validated against its own field list, and that a client sends the highest
  version the collector advertises. Decide host identity: a per-host keypair generated on the
  host, registered at enrollment, with revocation by key id at the collector.
- **Consequence if unfixed:** Client and collector are built against different version and auth
  assumptions, and a team loses reports during every rollout.
- **Tier:** high.

### F12. Session-record parsing has no home and no rule

- **Location:** Design Paradigm table, Capability Map.
- **Problem:** FR-26 and FR-27 (MVP) read harness session records, and FR-5 reads them for
  discovery. PRD §5 requires named fields only, with a marker-text test. The spine gives this
  no port, crate or rule, and does not say where the price table for FR-26 lives.
- **Fix:** Add session-record reading to the `HarnessAdapter` trait (or a separate
  `SessionRecords` port) returning only a typed struct of the named fields, so prompt text
  cannot reach the core by type; put the price table in harness knowledge; bind FR-26 and
  FR-27 in the map.
- **Consequence if unfixed:** Adapters return raw records and the core filters them, so the
  content rule depends on each caller, and the marker test is written per adapter.
- **Tier:** high.

### F13. Environment identity is too narrow

- **Location:** Consistency Conventions, Identifiers row.
- **Problem:** "Environments keyed by harness id plus absolute directory" cannot represent a
  scheduled or headless launch context in the same directory with a different environment
  (FR-5), or the desktop app and account settings, which have no directory (FR-7).
- **Fix:** Key an environment by harness id, a launch-context id (interactive, or a named job),
  and an optional directory; define the key once in `usher-core`.
- **Consequence if unfixed:** Adapters invent their own keys for these cases, and comparison
  and history (FR-14) join on keys that do not match.
- **Tier:** medium.

### F14. SQLite concurrency between modes is not stated

- **Location:** AD-2, AD-7.
- **Problem:** The daemon writes snapshots and records while the CLI holds the journal lock for
  a change, and MCP writes proposals. AD-2 covers harness config and the journal but not other
  store writes. The SQLite journal mode and busy handling are not set, and FR-64 asks that a
  stale lock be released and journaled, which `flock` does not record by itself.
- **Fix:** Set WAL mode with a busy timeout for all modes; state that non-journal store writes
  do not take the journal lock; write the holder's pid and start time into the lock file so a
  stale holder can be journaled when found.
- **Consequence if unfixed:** Intermittent "database is locked" failures in the daemon during
  long changes, handled differently per mode.
- **Tier:** medium.

### F15. Several bindings are wrong or missing

- **Location:** AD-14 Binds, Capability Map, frontmatter `binds`.
- **Problem:** AD-14 and the map bind FR-36 (maintainer docs) to reporting. AD-13 binds FR-40
  without a rule for it (see F11). The map has no row for FR-14, FR-15, FR-17, FR-18, FR-24,
  FR-25, FR-28, FR-30, FR-31 or FR-43, though `binds` claims FR-1 to FR-52.
- **Fix:** Correct the FR-36 binding (it belongs with FR-34 and FR-35 as repo content), and add
  map rows for each listed FR with its crate and governing AD.
- **Consequence if unfixed:** Builders read the map as the list of what the spine covers and
  miss features, or attach them to the wrong rules.
- **Tier:** medium.

### F16. Release pipeline and CI are not decided

- **Location:** missing; Deferred touches Windows cross-build only.
- **Problem:** PRD §5 requires signed and notarized builds and a published behavioral profile.
  AD-12 assumes a release pipeline that signs knowledge updates and carries current and next
  keys. Neither the CI system, the macOS signing and notarization step, nor where the signing
  key for knowledge lives is placed as decided, deferred or open.
- **Fix:** Name the CI (GitHub Actions, as the fleet uses), the release job that builds,
  signs and notarizes, and the separate job that signs knowledge files; list who holds the key
  as open under Q8.
- **Consequence if unfixed:** AD-12's rotation rule has nothing to run on, and the first
  release is hand built.
- **Tier:** medium.

### F17. The daemon may report usher's own writes as changes

- **Location:** AD-9, FR-13.
- **Problem:** When the CLI applies a change, the daemon sees the file change and may raise a
  "setting in effect changed" notice, using up the one-a-day notice budget on the person's
  own action.
- **Fix:** The daemon checks the journal for an outcome covering the path and hash before
  raising a notice, and records the change as usher-made.
- **Consequence if unfixed:** Noisy notices and a dated history (FR-14) that cannot tell
  usher's changes from outside ones.
- **Tier:** low.

### F18. Open questions are scattered, and some are missing

- **Location:** Deferred, Budgets.
- **Problem:** The spine has no Open Questions list. PRD Q16 (objective test for FR-10, FR-13,
  FR-26), Q17 (support levels per harness), Q24 (usage and cost APIs) and the unratified Q25
  budgets are not tracked in one place. DO_NOT_TRACK (FR-29) is absent from AD-14, and
  usher's account of its own activity (§5 two-way channel and "usher reports its own use") is
  not placed.
- **Fix:** Add an Open Questions section listing each with its owner and what it blocks; add
  DO_NOT_TRACK to AD-14; place the self-report under Consistency Conventions or the CLI.
- **Consequence if unfixed:** Small, but unowned questions get answered differently by whoever
  reaches them first.
- **Tier:** low.

## Operational envelope

| Dimension | State in spine | Needed |
| --- | --- | --- |
| Deployment, client | macOS host named; daemon install form absent | F5 |
| Deployment, collector | Team infra and team TLS named | Collector backup and retention job ownership (FR-42) |
| Environments (dev, test, release) | Absent | Part of F3 and F16 |
| Infrastructure, CI and signing | Absent | F16 |
| Operations (logs, self-report, collector upkeep) | Local logs decided | F18 for self-report |
| Upgrades of usher itself | Deferred with distribution | F4 |
| Store and collector schema migration | Absent | F4, F11 |
| Testing of adapters against harness versions | Absent | F3 |
