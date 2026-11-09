---
status: final
stepsCompleted: [1, 2, 3]
scope: full product (MVP and v2); release per epic follows specs/spec-usher/capability-scope.md
inputDocuments:
  - _bmad-output/planning-artifacts/prds/prd-usher-2026-11-03/prd.md
  - _bmad-output/planning-artifacts/prds/prd-usher-2026-11-03/addendum.md
  - _bmad-output/planning-artifacts/architecture/architecture-usher-2026-11-08/ARCHITECTURE-SPINE.md
  - _bmad-output/specs/spec-usher/SPEC.md
  - _bmad-output/specs/spec-usher/capability-scope.md
---

# usher - Epic Breakdown

## Overview

This document breaks usher's PRD, addendum, architecture spine and spec into epics and stories. Each FR's full consequences stay in the PRD; stories cite FR ids and the `AD-n` rules they must follow. Release (MVP, v2, later) is noted per epic and does not shape what a capability is.

## Requirements Inventory

### Functional Requirements

Release in brackets, from PRD §7.

FR-43: List every settings source per harness on the host, with location, layer and one status (in effect, shadowed, unread, unknown); unknown keys listed; derived state only; seat parts included (MCP registrations, packs, fleet client config, backend and auth route presence, toolchain). [MVP]
FR-1: Show each setting in effect per harness and launch directory with provenance (layer, file, harness-reported or rebuilt; per entry for merged lists), matching the harness's own resolved view where one exists. [MVP]
FR-2: Report values a file declares that the harness does not apply, including keys skipped silently (addendum §1.1 cases). [MVP]
FR-3: Stamp every map with the harness version, the method per reading, and what could not be observed. [MVP]
FR-4: List instruction, skill and agent files each harness discovers, as metadata only, including files discovered twice. [MVP]
FR-5: Map every launch directory: configured roots plus discovered proposals the person confirms; scheduled and headless launches as their own contexts with variable names (values only for setting variables). [MVP]
FR-6: Show folder trust state per directory and harness, matching the harness's own trust record. [MVP]
FR-7: Show the Claude desktop app and Claude general settings as environments; unreadable surfaces listed as not observed; shared settings shown once per surface. [MVP]
FR-8: Label environments and filter maps, comparisons and effects by label; labels stay local unless included in a report. [MVP]
FR-9: Show each tool's install state, source, version and whether it is behind; errored checks read as errors; addendum §3 failure modes caught. [MVP]
FR-10: Show running harness processes with start time, version and launch directory; flag sessions older than the binary on disk. [MVP]
FR-11: Install a missing harness or tool through its own installer, on confirmation, with the exact command shown; vendor login for credentials. [MVP]
FR-12: Mark apps and settings also governed by device management; report a reinstall that dropped custom profiles. [MVP]
FR-44: Update a harness, tool or usher itself on confirmation, showing from and to versions and naming running sessions first; usher updates itself through the installer that owns its install, or from a verified, team-named signed feed for a direct install. [MVP]
FR-45: Compare each host with a profile's declared tools, toolchain and versions; read or generate the declaration in the team's tool manager (for example mise). [MVP]
FR-46: Guided, resumable bootstrap from nothing: idempotent steps, every failure names step, error and next action, first session ends with one harness working in one repo. [MVP]
FR-47: Attach supporting services and method content (sideshow, eos) on confirmation, through each service's own setup; no bearer credential held. [MVP]
FR-48: Get a repository onto the host and open it in a harness for a person not fluent in git, each command named and confirmed. [MVP]
FR-65: Seat a reviewer at an agent-grown run variant: repo, declared tools and named profile, each on confirmation; the variant's tool declaration is untrusted. [v2]
FR-66: Launch a harness with a short-lived key the person's vault or cloud login issues for that launch, held in memory only; no stored long-lived key is read; plaintext keys found by the credential scan get a broker recommendation. [v2]
FR-67: Remove usher: list what it would leave, keep or roll back changes, keep journal and backups unless asked, and leave no process, login item, job or endpoint (tested). [MVP]
FR-13: Notify on harness self-update, sessions older than the installed binary, device-management reinstall and changed settings in effect; batched and quiet by default; watcher writes only usher state. [MVP]
FR-14: Keep dated, coarse snapshots of each environment map and report differences between them, to line up with the service operator's daily usage. [MVP]
FR-15: Compare two environments, or an environment and a profile, in either direction: present, missing, different, extra; order-blind lists; path tokens resolved; against settings in effect where possible. [MVP]
FR-16: Findings per environment: retired or invalid keys, layer conflicts, undocumented settings, weak locks, drift; hazard checks ship only as their tests pass; each finding names what the harness does with the key. [MVP]
FR-17: Mark each profile entry valid, out of range or unchecked for the target harness version before placement. [MVP]
FR-18: Answer which hosts or directories carry entry X and which run a stale tool Y, over local reports or the collector, listing hosts not seen. [MVP]
FR-19: Capture an environment, or parts of several, into a draft profile with source environment, layer and version per entry; no credentials or content. [MVP]
FR-20: Portable entries: capture proposes path tokens for home, config dir, repo root and user; a literal path naming the author's home is flagged. [MVP]
FR-21: Drop and annotate entries before sharing; drafts are marked unreviewed until saved. [MVP]
FR-22: Load a profile and see it compared with one's environments, per entry status and version validity; loading changes nothing. [MVP]
FR-23: Record the profile version an applied, verified entry came from; later changes show as drift. [MVP]
FR-24: Show exact placement for an entry: file, layer and snippet in the file's own format; never a layer the harness ignores; name a higher-layer override first. [MVP]
FR-25: Re-map after applying: in effect, overridden (by which layer) or not loaded (why). [MVP]
FR-49: Apply an entry, profile or recommendation on confirmation, with dry run, diff, journal and rollback; hand-set values shown as conflicts; never a managed layer or personal entry; mark and unmark personal entries. [MVP, local host; remote apply v2]
FR-50: Restore a seat after reinstall, new machine or time away: list differences and restore steps, apply confirmed ones; dropped device-management profiles restorable. [MVP]
FR-51: Catch up on followed profile sources since the versions last loaded, with author and note per change; adopt each or keep own value. [MVP]
FR-52: Produce an organization drop-in artifact (for example managed settings) to a file; never deploy it. [MVP]
FR-26: Cache reuse per session: cache read versus full-cost write, with model and environment; unpriced models reported; throttled calls counted apart. [MVP]
FR-27: Show observed effects beside a comparison or the settings history, stating sessions, dates and backend; never combine backends. [MVP]
FR-53: Activity effects per environment and project from named fields only: tool calls, tokens, errors, model changes, time between turns against cache lifetime, cache resets, instruction-file size. [v2]
FR-28: Save any map, comparison or effect as a local report in a published, versioned schema, holding only reported fields. [MVP]
FR-29: Reporting off by default, gating the code path; tests for no egress by default and reported-fields-only when on; published do-not-collect list; DO_NOT_TRACK honored. [MVP]
FR-54: Reporting levels from metrics to full OTEL, each with its own field list and consent; usher as local OTLP endpoint forwarding only allowed fields. [v2]
FR-30: Every harness knowledge item cites a reference and last-verified date (structural check); findings name their item; support level per harness version shown to the person. [MVP]
FR-31: Ship harness knowledge as a versioned, signed data file without a client release; client reports the version used. [MVP]
FR-32: Re-check dates per item class, plus due on each new harness release; findings resting on overdue or unverified items are marked. [MVP]
FR-33: Record per harness and version what the harness does with a stale key (warn, alias, fail, skip); at least one probe-verified entry per harness. [MVP]
FR-34: Release helpers collect what changed in a harness release and draft proposed knowledge updates under a scratch home. [MVP]
FR-35: Re-runnable probes for each claim needing a test, under a scratch home with a positive control, recording version and result in the knowledge item. [MVP]
FR-36: Maintainer docs per harness: sources, observation method, probes, last full re-check, update steps. [MVP]
FR-37: Self-hosted light collector; reporting turned on per host by the person or the organization's device management, recorded and told to the person; no usher-operated service. [MVP]
FR-38: Collector accepts only reported fields and rejects any other field, content or credential; the enabled-state egress test runs against it. [MVP]
FR-39: Consent before a host's first report: what, to whom, how to stop; withdrawal stops at once; events recorded; a person can see and delete their hosts' data. [MVP]
FR-40: Per-host credential the collector can revoke and the host can re-mint; passes the ADR-009 audience test; revocation is per host. [MVP]
FR-41: Cross-host queries for FR-18 questions, findings and effects; no view ranks people; a person sees their own first; per-person drill-down only after a recorded team decision. [MVP]
FR-42: Team-set retention, short by default (90 days); expired data deleted with a deletion record. [MVP]
FR-55: Recommendations from followed profiles, templates and own records, each with source, reason, assumed versions and backend, and blind spots; cost-bearing settings included; allow entries marked as loosening; pushed only for opted-in kinds. [MVP, templates v2]
FR-56: Team-declared templates evaluated per host and across hosts; they never block or change a setting. [v2]
FR-57: Turn a difference into a proposal for a team profile, body drafted from what usher knows, routed to a named reviewer with a review clock. [v2+]
FR-58: Reviewer accepts, holds or declines with a reason; accepted changes become a profile version offered through catch-up with author credit; overdue proposals surfaced. [v2+]
FR-59: Enroll with a team instance; team customization layers over defaults and never over the personal layer; enrollments and consents kept apart per organization. [v2]
FR-60: Signed, versioned catalog of profiles, templates and recommendations in an open, published format; client verifies before use. [v2]
FR-61: Options index extending what the client can check, install or recommend; signed, versioned, confirmed at point of use; unverifiable items ignored and reported. [v2]
FR-62: Reach usher from the CLI, from inside the harness (MCP), and from an app, with a background service running; in-harness agents follow the same confirmation rules; every in-LLM surface has a token budget. [MVP: CLI, service, thin MCP; v2: full in-LLM and app]
FR-63: Change journal and rollback: entry written and flushed before each change; full before and after state, undo fitted to the change kind; reverse-order rollback with conflict checks and dependency handling; append-only; owner-only backups kept until removed. [MVP]
FR-64: Safe execution: refuse empty, root, home, unexpanded or glob targets; bounded scope per write kind; no credential file writes; key-level writes in mixed files; back up before write; atomic temp-and-rename preserving format; concurrent-edit detection; argument lists for delegated runs; pinned installers; one change run per host under a lock; failure-injection test per addendum §7 row. [MVP]

### NonFunctional Requirements

NFR1: Changes only on confirmation, on every surface; an agent confirms as the person can (operator 2026-11-08); the journal records the confirming surface; usher is never the sole configuration authority.
NFR2: Every change is journaled first and passes the safe-execution checks for its kind; structural, tested, may gate.
NFR3: Observation changes no harness state: each harness call is shown by test not to write state, refresh caches, start update loops or self-update; reset, fix, update and purge calls are never made; SM-5 target zero, may gate.
NFR4: Local observation from install, limited to harness-knowledge locations until directories are confirmed; no home-wide sweep; first open tells the person what usher has looked at.
NFR5: Derived state, not content.
NFR6: Session records parsed by named fields only; a marker-text test proves no content field reaches any output.
NFR7: Credentials: presence, never value; the credential scan is off by default and reports location and type only; no bearer authority at a third party (ADR-009).
NFR8: Per-person data never enters git; client names stay out.
NFR9: Every check stamps the harness version it assumes.
NFR10: Every coded check is deterministic and tested; AI use follows progressive codification under about 10 USD per user per month.
NFR11: Builds are signed and notarized; outbound traffic is explicit; a behavioral profile is published.
NFR12: Independence: every sibling works without usher and usher without them.
NFR13: Boundaries by objective: setup and maintain settings; critic judges, stagekeeper recovers.
NFR14: Platforms: macOS first, Linux likely, Windows open (Q14).
NFR15: Attention budget: usher asks only for a confirmation, a consent, or a choice with no default; at most one unprompted notice a day, batched.
NFR16: Footprint, tested on a laptop on battery: daemon at most 10 MB resident idle, no timer wakeups and under 0.1% CPU averaged over an hour, at most 200 ms CPU per changed file, 256 watched directories by default with dropped paths named, 100 MB of state before asking to prune; usher reports its own use.
NFR17: Token budget: MCP tool descriptions at most 1,500 tokens in total; MCP responses at most 4,000 tokens by default, paged beyond.
NFR18: Two-way channel: a person can ask at any time what usher is doing, has looked at, and holds about them, and their consent state, and change it.

### Additional Requirements

From the architecture spine (AD-1 to AD-20, conventions, stack, seed):

- **Starter (Epic 1, Story 1):** no cargo starter template. The repo starts from the fleet's own bootstrap: `docs/new-repo-runbook.md` (orc) with packs `_packs/fleet-bootstrap` (already applied to usher: rules, editorconfig, dependency-review and release-verify workflows) and `_packs/fleet-bootstrap-rust` (not yet applied: `lefthook.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `deny.toml`, `.claude/rules/rust.md`). Then the cargo workspace per the spine's crate tree, with `usher-testkit` first (AD-20).
- Toolchain pin: the spine binds Rust 1.99.1; the pack's `rust-toolchain.toml` says `stable`. Pin `1.99.1` in `rust-toolchain.toml` (and `mise.toml`) so CI and hosts agree.
- One binary, four modes (`usher`, `usher mcp`, `usher daemon`, `usher collector`) over one SQLite store (AD-1); crates named `usher-<area>`, adapters `usher-adapter-<harness>`.
- Journal lock via `std::fs::File::lock` in the state directory; one change run per host (AD-2).
- Five-step change transaction with a crash marker resolved only from CLI or MCP (AD-3); every surface confirms the same way and the journal records which (AD-4).
- Harness native writer first (codex app-server `config/value/write`, `config/batchWrite`); otherwise `toml_edit`, `jsonc-parser` with `cst`, `yamlpatch`; byte check outside the edited value (AD-5).
- One adapter per harness implementing `HarnessAdapter`; harness facts in harness knowledge (AD-6).
- SQLite WAL, `synchronous=FULL`, insert-only journal, key-level backups for files holding credentials, no automatic backup eviction (AD-7).
- Only known keys read; credentials and trust records never read or written (AD-8); never-call list of harness commands (AD-9); codex reads against a scratch-home copy (app-server writes `installation_id` and SQLite files into the codex home).
- Daemon is event-driven on `notify` with `macos_fsevent`, no timers (AD-9); launchd per-user agent.
- Discovery proposes from harness records and named roots, depth-limited; never searches `~` unless named (AD-10).
- Profiles: a directory with `profile.toml` plus one native fragment per harness; MCP servers the only shared form; git source version is the commit (AD-11).
- Harness knowledge: one TOML schema, embedded first, slow and fast layers keyed to version ranges, signed updates verified with a compiled-in minisign ed25519 key and sequence numbers (AD-12).
- Collector: HTTPS with rustls, one report schema owned by `usher-core`, versions N and N-1 accepted, unknown fields rejected whole, host ed25519 key pair with the private key in the keychain, 90-day retention (AD-13).
- Reporting off by default, sent on events (after a change, at start, on network return) from an outbox, `DO_NOT_TRACK` honored (AD-14).
- Delegated runs go through the runner port with argument lists and before/after path capture (AD-15).
- Proposals hold operations and base hashes; the diff is computed at apply (AD-16).
- Environment key: host id, OS user, harness id, absolute directory, launch context (AD-17).
- One map update path in `usher-core`; usher knows its own writes so self-writes raise no notice (AD-18).
- Self-update and forward-only migrations through the same transaction (AD-19).
- Adapters tested against pinned harness versions in scratch homes via `usher-testkit` (AD-20).
- Conventions: ULIDs for journal entries, proposals and reports; RFC 3339 UTC; path tokens `${HOME}`, `${CONFIG_DIR}`, `${REPO}`, `${USER}` with globs; one `UsherError` carrying step, check, status and next action; local logs only, rotated in the disk budget; `tokio` in daemon, MCP and collector modes.
- Stack versions as pinned in the spine (tokio 1.54.1, rusqlite 0.41.1 `bundled`, toml_edit 0.25.19, jsonc-parser 0.34.0 `cst`, yamlpatch 1.32.0, serde_json 1.0.154, notify 8.2.2, rmcp 3.9.0, axum 0.8.9, axum-server 0.9.0, rustls 0.23.46, minisign-verify 0.3.0, ed25519-dalek 3.0.0, keyring 4.2.0, ulid 3.0.0).
- App bundle: a Tauri 2 menu-bar app over `usher-core` and the same store; its webview never writes, sending proposals only; it watches no files (AD-21).
- Launch keys through one broker port of issuing routes only; usher never reads a stored long-lived key, and Anthropic has no route until a key can be issued programmatically (AD-22).
- Self-update goes through the installer that owns usher's install, or a team-named signed feed for a direct install (AD-19). The public homebrew tap, a default feed, notarization and the installer are deferred while the repo is private (Q11); the sign and notarize plumbing comes later from the apple-codesign skill and the runbook's sidestep pointer.
- Untested claims stay in harness knowledge until probed: codex device-management precedence, goose `GOOSE_MODE` default.

### UX Design Requirements

None. No UX document; the surfaces are the CLI, the background service and a thin MCP surface.

### FR Coverage Map

FR-1 to FR-8, FR-43: Epic 1 - inventory, settings in effect, provenance, discovery, trust, desktop app, labels
FR-30, FR-31, FR-33, FR-35: Epic 1 - referenced, signed harness knowledge; stale-key behavior; re-runnable probes
FR-9, FR-10, FR-12: Epic 2 - install state, process table, device-management overlap
FR-32, FR-34, FR-36: Epic 3 - re-check dates, release helpers, maintainer docs
FR-15, FR-16, FR-18, FR-28: Epic 4 - comparison, findings, which-hosts queries over local reports, local reports
FR-13, FR-14: Epic 5 - change notices, dated settings history
FR-11, FR-44, FR-63, FR-64, FR-67: Epic 6 - journal, safe execution, install, update, self-update, removal
FR-17, FR-19 to FR-22, FR-45: Epic 7 - capture, portable entries, filter, consume, version validity, declared tools
FR-23 to FR-25, FR-49 to FR-52: Epic 8 - placement, verify, adoption, apply, restore, catch-up, drop-in (local host)
FR-46 to FR-48: Epic 9 - guided bootstrap, services and method content, repository help
FR-62 (MVP part): Epic 10 - thin in-harness MCP surface
FR-26, FR-27, FR-55: Epic 11 - cache reuse, effects beside settings, recommendations
FR-29, FR-37 to FR-42 (and FR-18 across hosts): Epic 12 - reporting off by default, light collector
FR-53, FR-54: Epic 13 - activity effects, reporting levels
FR-56, FR-59 to FR-61 (and FR-49 remote apply): Epic 14 - team instance
FR-62 (v2 part): Epic 15 - full in-harness surface and app
FR-65, FR-66: Epic 16 - reviewer seats, brokered keys
FR-57, FR-58: Epic 17 - contribution loop

## Epic List

### Epic 1: See my seat [MVP]
A person lists every settings source on their host and sees, per launch context, each setting in effect with its provenance, version stamp and blind spots, for Claude Code, the Claude desktop app, codex and goose, on harness knowledge that is referenced, signed and probe-verified. Story 1.1 bootstraps the repo; one story per harness, Claude Code first.
**FRs covered:** FR-43, FR-1 to FR-8, FR-30, FR-31, FR-33, FR-35

### Epic 2: Know what's installed and running [MVP]
A person sees each harness and tool's install state, source, version and whether it is behind (with the installer's command shown, not run), which sessions predate the binary on disk, and where device management overlaps.
**FRs covered:** FR-9, FR-10, FR-12

### Epic 3: Keep harness knowledge current [MVP]
A maintainer keeps harness knowledge fresh: re-check dates by class and by release, release helpers that draft updates, and a maintainer page per harness.
**FRs covered:** FR-32, FR-34, FR-36

### Epic 4: Compare environments and see findings [MVP]
A person compares any two environments, or an environment and a profile, in either direction, sees findings with hazard checks that ship only as their probes pass, saves local reports, and asks which hosts carry an entry.
**FRs covered:** FR-15, FR-16, FR-18, FR-28

### Epic 5: Hear when the ground moves [MVP]
The background service tells a person when a harness self-updates, a session runs an old binary, device management reinstalls an app, or a setting changes, and keeps a dated settings history, within the footprint budgets.
**FRs covered:** FR-13, FR-14

### Epic 6: Install, update and remove safely, with undo [MVP]
Every change goes through the journal and the safe-execution checks: a person installs and updates tools and usher itself on confirmation, rolls back any change, and removes usher. Includes a named story that makes the watcher recognize usher's own writes (AD-18), and covers self-update through the installer that owns usher's install and from a team-named feed, with only the public tap and default feed blocked on Q11.
**FRs covered:** FR-11, FR-44, FR-63, FR-64, FR-67

### Epic 7: Capture and load profiles [MVP]
An expert captures, filters and annotates a portable profile; a newcomer loads it, sees each entry's status and version validity, and the tools it declares. Loading changes nothing.
**FRs covered:** FR-17, FR-19 to FR-22, FR-45

### Epic 8: Apply, restore and catch up [MVP, local host]
A person sees exact placement, has usher apply entries with write safety, verifies them, restores a seat, catches up on followed sources, and produces an organization drop-in.
**FRs covered:** FR-23 to FR-25, FR-49 to FR-52

### Epic 9: Newcomer bootstrap from nothing [MVP]
A newcomer reaches a working seat in one guided, resumable run, with supporting services, method content and repository help.
**FRs covered:** FR-46 to FR-48

### Epic 10: Work from inside the harness [MVP]
From Claude Code or the desktop app, the thin MCP surface maps, compares, explains findings, and proposes and applies changes confirmed as on any surface, within its token budget.
**FRs covered:** FR-62 (MVP part)

### Epic 11: See effects and get recommendations [MVP]
A person sees cache reuse per session and effects beside their settings, and gets recommendations from followed profiles and their own records.
**FRs covered:** FR-26, FR-27, FR-55

### Epic 12: Team view through the light collector [MVP]
Reporting stays off until a person turns it on; it opens with the no-egress test, and the sender is not built before it. A team runs a self-hosted collector with consent, per-host keys, cross-host queries and retention.
**FRs covered:** FR-29, FR-37 to FR-42

### Epic 13: Richer activity and reporting levels [v2]
**FRs covered:** FR-53, FR-54

### Epic 14: Team instance [v2]
Enrollment, the signed catalog, options index and templates, and remote apply.
**FRs covered:** FR-56, FR-59 to FR-61, FR-49 (remote)

### Epic 15: Full in-harness surface and the app [v2]
**FRs covered:** FR-62 (v2 part)

### Epic 16: Reviewer seats and brokered keys [v2]
**FRs covered:** FR-65, FR-66

### Epic 17: Contribution loop [v2+]
**FRs covered:** FR-57, FR-58

## Epic 1: See my seat

A person lists every settings source on their host and sees, per launch context, each setting in effect with its provenance, version stamp and blind spots, for Claude Code, the Claude desktop app, codex and goose, on harness knowledge that is referenced, signed and probe-verified. Nothing in this epic writes outside usher's own state directory.

### Story 1.1: Bootstrap the usher workspace from the fleet baseline

As a usher maintainer,
I want the repo set up from the fleet's new-repo runbook with the Rust pack and a minimal cargo workspace,
So that every later story builds, lints and tests the same way on every host and in CI.

**Acceptance Criteria:**

**Given** the orc's `docs/new-repo-runbook.md` and `_packs/fleet-bootstrap-rust`
**When** the pack is applied to usher
**Then** `lefthook.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `deny.toml` and `.claude/rules/rust.md` are present, and `.claude/rules/_index.md` includes `@rust.md`
**And** `rust-toolchain.toml` and `mise.toml` both pin Rust 1.99.1, the spine's version, not `stable`

**Given** the spine's crate tree
**When** the workspace is created
**Then** it holds only the crates this story needs: `usher` (mode selection), `usher-core` (with `UsherError` carrying step, check, status and next action) and `usher-testkit`; later crates are added by the stories that need them
**And** `usher --version` prints the version, and `usher` with an unknown mode exits nonzero with a next action

**Given** a pull request
**When** CI runs
**Then** it runs `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` and `cargo deny check` on macOS with the pinned toolchain, using the step-security action pins the runbook names

### Story 1.2: Scratch homes and the observation invariant

As a usher maintainer,
I want every adapter test to run against a scratch home and prove the real home was untouched,
So that "observation changes nothing" (SM-5, AD-9, AD-20) is a test from the first adapter on.

**Acceptance Criteria:**

**Given** `usher-testkit`
**When** a test asks for a scratch home
**Then** `HOME`, `TMPDIR` and each harness home variable (`CLAUDE_CONFIG_DIR`, `CODEX_HOME`, goose's config dir) point inside a fresh temporary directory with a short path, removed after the test

**Given** a test that runs inside a scratch home
**When** it finishes
**Then** the testkit compares a metadata hash (name, type, size, mtime, ctime, inode) of each real harness home before and after, and fails the test if they differ
**And** a positive-control test that deliberately writes to a fake "real" home fails, proving the check can fire

**Given** a harness binary pinned for tests
**When** the testkit runs it
**Then** the binary's version is recorded with the result, and a missing binary skips the test with a named reason rather than passing it

### Story 1.3: Referenced harness knowledge, embedded in the binary

As a person using usher,
I want every fact usher states about a harness to name its reference and the date it was last verified,
So that I can check a finding against its source and see when a fact may be stale (FR-30).

**Acceptance Criteria:**

**Given** `usher-knowledge` and its TOML schema (AD-12)
**When** the embedded knowledge file is built
**Then** each item carries a reference (vendor docs, changelog, source or a usher probe) and a last-verified date, and an item missing either fails the structural check in CI
**And** items are keyed to harness version ranges, in a slow layer (layers, precedence) and a fast layer (keys, defaults)

**Given** a harness and its installed version
**When** usher reports its support level
**Then** it lists the capabilities offered for that version (inventory, install, map, compare, place, apply), so a missing capability reads as unsupported, never as a clean result

**Given** any later report or finding
**When** it relies on knowledge
**Then** it names the knowledge file version used (FR-31)

### Story 1.4: Signed harness-knowledge updates without a client release

As a person using usher,
I want usher to take newer harness knowledge only when it is signed by the usher key,
So that a release that changes a harness reaches me without a new usher build, and nobody can feed me forged facts (FR-31).

**Acceptance Criteria:**

**Given** an update file signed with the minisign ed25519 key compiled into the binary
**When** usher loads it
**Then** the signature and a sequence number higher than the current one are both checked before any item is used, and the new version is reported

**Given** an update with a bad signature, or a sequence number not higher than the current one
**When** usher loads it
**Then** it is rejected, the embedded or last good knowledge stays in use, and the person is told why

### Story 1.5: Claude Code settings inventory

As a person using Claude Code,
I want to list every Claude Code settings source on my host, whether or not anything reads it,
So that I can see what exists before asking what is in effect (FR-43).

**Acceptance Criteria:**

**Given** a scratch home with managed, user, project and local settings files, `~/.claude.json` and a `remote-settings.json` cache
**When** I run `usher inventory`
**Then** each source is listed with location, harness, the layer it would belong to and one status: in effect, shadowed, unread or unknown
**And** keys usher does not recognize for the version are listed, never dropped

**Given** any source
**When** it is recorded in usher's store (the SQLite tables this story creates, WAL, `synchronous=FULL`)
**Then** only derived state is kept: presence, path, size, modified time and key names; no file content and no credential value (AD-8)

**Given** the seat around the harness
**When** the inventory runs
**Then** MCP registrations, packs, fleet-tool client config, the backend and auth route (presence only) and the toolchain are listed too
**And** the Story 1.2 invariant test passes for the run

### Story 1.6: Claude Code settings in effect with provenance

As a person using Claude Code,
I want to see each setting in effect for a launch directory and where it came from,
So that I know what actually applies, not just what one file says (FR-1, FR-3).

**Acceptance Criteria:**

**Given** a directory with layered settings in a scratch home
**When** I run `usher map <dir>`
**Then** each setting names its layer and file and is marked harness-reported or rebuilt, and precedence is Claude Code's, per harness knowledge
**And** permission lists and `autoMode` that merge across layers show provenance per entry (addendum §1.1)

**Given** any map
**When** it is produced
**Then** it carries the harness version, the method per reading (harness query, file read or rebuild), and the list of what usher could not observe
**And** a layer present but skipped is listed with the harness's reason, where `/status` gives one
**And** no command on the never-call list (`claude auto-mode`, `/doctor` fixes, `purge`, `install`, `respawn`, plugin `update`) is run

### Story 1.7: Declared but not in effect, and folder trust, for Claude Code

As a person using Claude Code,
I want to see where a file declares a value Claude Code does not apply, and whether a directory is trusted,
So that a setting that silently stopped applying no longer looks like one that works (FR-2, FR-6, UJ-1 edge case).

**Acceptance Criteria:**

**Given** each Claude Code case in addendum §1.1 for FR-2 (a skipped managed source, an entry `claude doctor` rejects, an environment variable overriding its key, `defaultMode` auto or bypass, `autoMode` or OTEL settings in a project or local file, a server-managed value held back)
**When** the directory is mapped
**Then** each is reported as declared and not in effect, never as in effect

**Given** a repo root
**When** trust is mapped
**Then** the state matches `projects[<repo root>].hasTrustDialogAccepted` in `~/.claude.json`, read by key only, and an untrusted directory whose project layers do not load says so

### Story 1.8: Instruction, skill and agent files as metadata

As a person using any supported harness,
I want to see which instruction, skill and agent files each harness discovers for a directory,
So that I know what loads without usher reading what is in them (FR-4).

**Acceptance Criteria:**

**Given** a directory with `CLAUDE.md`, `AGENTS.md`, skills and agent files at the locations harness knowledge names
**When** I map it
**Then** each file is listed with presence, path, permissions and size, and a file discovered twice through two locations is flagged
**And** a test with marker text in every file confirms no output contains it

### Story 1.9: Every launch directory, with consent for discovery

As a person,
I want usher to map the directories I actually launch from, including scheduled and headless launches,
So that my map covers more than the current directory and the user file (FR-5, AD-10, AD-17).

**Acceptance Criteria:**

**Given** a fresh install
**When** usher observes before I confirm any directory
**Then** it reads only harness-knowledge locations, and directories found in harness records (Claude Code `projects` keys, codex trusted projects, desktop-app trusted folders) are kept as proposals, none mapped

**Given** roots I name
**When** I ask usher to look
**Then** it runs one depth-limited search for `.git`, `.claude/`, `.codex/` and `AGENTS.md`, proposes what it finds, records the named roots and my consent, and never searches `~` unless I name it
**And** I can include or exclude any directory or root

**Given** a launchd job that starts a harness
**When** it is mapped
**Then** it is its own launch context keyed by host, user, harness, directory and launch context, with the environment variables it receives recorded by name; a value is recorded only for a variable harness knowledge marks as a setting, never for a credential

### Story 1.10: codex map through its own resolved view

As a person using codex,
I want my codex settings inventoried and mapped from codex's own resolved view,
So that codex's values and layers in usher match what codex itself reports (FR-43, FR-1, FR-2, FR-6).

**Acceptance Criteria:**

**Given** a codex home
**When** usher reads it
**Then** it copies the codex home into usher's state directory and runs `codex app-server` stdio `config/read` with `cwd` and `includeLayers` against the copy, because the app-server writes `installation_id` and sqlite files into its home; the copy is usher's own state, not a change, and is removed after the read
**And** the real codex home's metadata hash is unchanged (Story 1.2 check)

**Given** a directory
**When** it is mapped
**Then** values, layers, `disabledReason` and per-key origins match `config/read` for that directory
**And** a project file setting `notify` is reported as declared and ignored
**And** in an untrusted repo the project config layer is reported disabled while project skills under `.agents/skills` and `.codex/skills` are reported as loaded (probe of 2026-11-08, codex 0.160.1)

### Story 1.11: goose map rebuilt from files

As a person using goose,
I want my goose settings mapped with environment variables taking precedence over files,
So that goose's map is right even though goose has no per-key resolved view (FR-43, FR-1, FR-2).

**Acceptance Criteria:**

**Given** `/etc/goose/config.yaml`, files named in `GOOSE_ADDITIONAL_CONFIG_FILES` and the user file
**When** usher maps goose
**Then** it rebuilds the stack in goose's order, with an environment variable for a key beating every file, and marks each reading as rebuilt
**And** `extensions` and `providers` that merge across layers show provenance per entry
**And** a key set in the user file and by environment variable reports the environment value in effect and the file value as declared, not in effect

**Given** the `GOOSE_MODE` default, still untested
**When** a map depends on it
**Then** it is listed as not observed, with the knowledge item marked unverified

### Story 1.12: Claude desktop app and Claude general settings

As a person who starts in the Claude desktop app,
I want its settings shown as an environment beside Claude Code,
So that what I set in the app and what Claude Code applies are visible side by side (FR-7, UJ-1).

**Acceptance Criteria:**

**Given** `claude_desktop_config.json`, `config.json`, `developer_settings.json`, extension files and the managed preference `com.anthropic.claudefordesktop`
**When** usher maps the desktop app
**Then** MCP servers and preferences are read by the keys harness knowledge lists; `config.json` is treated as holding credentials, so `oauth:tokenCache` is never read; the managed `disableAutoUpdates` is shown as managed
**And** Claude's account-level settings, not stored locally, are listed as not observed, never left out

**Given** a setting that exists in both the desktop app and Claude Code
**When** both are mapped
**Then** it is shown once per surface, with which value applies where

### Story 1.13: Stale-key behavior and re-runnable probes

As a usher maintainer,
I want each harness claim that needs a test to have a probe I can re-run against a new release,
So that usher's findings rest on behavior we measured, not on what the docs say (FR-33, FR-35).

**Acceptance Criteria:**

**Given** a claim in harness knowledge marked as needing a test
**When** its probe runs under a Story 1.2 scratch home
**Then** it includes a positive control, and records the harness version and the result in the knowledge item it verifies

**Given** the probes run on 2026-11-08
**When** this story lands
**Then** the three confirmed claims are probes in the repo: Claude Code omitting `$defaults` drops 16 built-in allow rules to the one set; codex loads project skills in an untrusted repo; codex logs an ignored project key at error level
**And** each supported harness has at least one stale-key behavior entry (warn, alias, fail or skip silently) verified by probe; codex `approval_policy = "untrusted"` is recorded as a hard error from the release that removed it

### Story 1.14: Environment labels

As a person,
I want to label environments by use case and filter by label,
So that I can compare, for example, my "reviews" setup with my "long refactors" setup (FR-8).

**Acceptance Criteria:**

**Given** a mapped environment
**When** I add a label
**Then** the label is stored locally and filters maps, and later comparisons and effects
**And** a label appears in a saved report only if I include it

## Epic 2: Know what's installed and running

A person sees each harness and fleet tool's install state, source, version and whether it is behind its source, which running sessions predate the binary on disk, and where device management also governs what usher observes. Usher shows the owning installer's command for any fix and runs nothing; running it arrives in Epic 6.

### Story 2.1: Installed tools, their source, version and whether they are behind

As a newcomer,
I want to see which harnesses and fleet tools are installed, from where, at which version, and whether each is behind,
So that I know what I have before I try to use it (FR-9).

**Acceptance Criteria:**

**Given** harnesses and tools installed through brew, mise, sideshow, `tools/install.sh` and vendor installers
**When** I run `usher tools`
**Then** each is listed with install state, install source, installed version, the source's current version and behind, current or unknown
**And** a tool that is missing or behind shows the owning installer's exact command and source, which usher does not run

**Given** the installer queries this story makes
**When** they run
**Then** each runs with that installer's no-update switches set (for example `HOMEBREW_NO_AUTO_UPDATE=1`), and commands on the installer never-call list (any that update, upgrade, clean up or re-link) are never run
**And** the Story 1.2 invariant test covers each installer's own state as well as harness homes, including `$(brew --repository)` and the mise install directory

**Given** the behind-check, which asks each source for its current version
**When** usher runs with defaults
**Then** the check is on, sends nothing about the person or host beyond the request itself, is listed in the published behavioral profile with each endpoint it contacts, and has its own off switch
**And** with the check off, or the network unreachable, a tool reads "unknown", never "current"

**Given** a version check that errors (for example an installer that fails to answer)
**When** the list is produced
**Then** that tool is reported as an error with the failing command and its exit status, never as current

### Story 2.2: Installer-level drift and failure modes

As a fleet engineer,
I want usher to catch the installer failures this fleet has already hit,
So that a tool that only looks current is reported as it is (FR-9, addendum §3).

**Acceptance Criteria:**

**Given** a tool whose installed version file reads current while its content differs from its source
**When** usher checks it
**Then** it is reported as drift

**Given** a fixture for each installer-level failure mode in addendum §3 (an installed copy behind its merged source, a version file that reads current when it is not, a doubly tapped formula that errors instead of answering, a read-only install updated in name only)
**When** the health check runs against it
**Then** the failure is reported with its cause, and each mode has its own test

### Story 2.3: Pack-level failure modes

As a fleet engineer,
I want usher to catch the pack and plugin failures this fleet has already hit,
So that method content that looks installed but does not resolve is reported (FR-9, addendum §3).

**Acceptance Criteria:**

**Given** a fixture shaped like real sideshow state for each pack-level failure mode in addendum §3 (stale command files after a pack version change, undercounted status, unpruned old versions, a layout migration shipped as a point release, built skills that never resolve, plugins with a missing source or no recorded enablement, a missing client env file, unsigned packs)
**When** the health check runs against it
**Then** the failure is reported with its cause, and each mode has its own test
**And** usher reads pack state only; fixing it is sideshow's, which usher drives in Epic 6

### Story 2.4: Running harness processes and stale sessions

As a person,
I want to see which harness sessions are running, from where and on which version,
So that I know when a session is running an older binary than the one now installed (FR-10).

**Acceptance Criteria:**

**Given** running Claude Code, codex and goose processes
**When** I run `usher ps`
**Then** each is listed with start time, version and launch directory, matched to its environment, read through `proc_pidinfo` rather than `ps` (which cannot run under `sandbox-exec`)

**Given** a process usher sees for the first time
**When** it is recorded
**Then** its version is the binary's version at that sighting, stored with the process; a process that started before usher first saw it shows "version at start: unknown" where the binary changed in between

**Given** a session that started before its binary on disk last changed, including a harness that self-updated under a running seat (addendum §3)
**When** the table is produced
**Then** that session is flagged as running an older binary, with both versions where known
**And** usher never signals, restarts or attaches to a process (restarting is stagekeeper's objective)

### Story 2.5: Device-management overlap

As a person on a managed laptop,
I want to see when device management also governs an app or setting usher observes,
So that I know which settings my organization locks and when a reinstall dropped them (FR-12).

**Acceptance Criteria:**

**Given** a managed preference or configuration profile (for example `com.anthropic.claudefordesktop` in `/Library/Managed Preferences`)
**When** usher maps the app or setting it governs
**Then** it is marked managed, with the profile's identifier

**Given** an app whose bundle was replaced while its custom profiles went missing at the same time
**When** usher compares with the inventory it last stored (Story 1.5)
**Then** it reports a reinstall that dropped them, naming the profiles; noticing this as it happens arrives in Epic 5

## Epic 3: Keep harness knowledge current

A maintainer keeps harness knowledge fresh as the harnesses ship every few days: items come due by class and by release, helpers draft the update from what a release changed, and each harness has a page saying where its truth lives. Helpers propose; a person accepts (SOUL §8).

### Story 3.1: Re-check dates by class and by release

As a person using usher,
I want each finding that rests on an overdue or unverified fact to say so,
So that I can tell a fresh check from one that may have gone stale (FR-32).

**Acceptance Criteria:**

**Given** knowledge items of different classes
**When** they are loaded
**Then** each carries a re-check date set by its class, with fast-changing keys and defaults due sooner than layers and precedence

**Given** a harness release newer than the version an item was verified on
**When** usher detects the new version (Story 2.1)
**Then** every item for that harness is due, whatever its date

**Given** a finding resting on an item past its date, or not yet verified for the version in use
**When** it is shown
**Then** it is marked as resting on an overdue or unverified item, naming the item

### Story 3.2: Release helpers that draft knowledge updates

As a usher maintainer,
I want a helper that collects what changed in a harness release and drafts the knowledge update,
So that keeping up with releases takes minutes, not an afternoon of reading (FR-34).

**Acceptance Criteria:**

**Given** a new release of a supported harness
**When** I run the helper for it
**Then** it collects the changelog, any generated schema (codex `generate-json-schema`) and the harness's own help and config output, each run under a Story 1.2 scratch home
**And** it writes a proposed change to the knowledge file, with a reference per item, for review; it never applies the change
**And** the invariant test shows the maintainer's real harness homes untouched

### Story 3.3: A maintainer page per harness

As a usher maintainer,
I want each supported harness to have a page naming its sources, how usher observes it, its probes and its last full re-check,
So that whoever picks up a release knows where to look and what to re-run (FR-36).

**Acceptance Criteria:**

**Given** each supported harness (Claude Code, the desktop app, codex, goose)
**When** the docs are built
**Then** it has a page naming where its truth lives (source, docs, changelog), the observation method, its probes, its last full re-check, and the steps to verify and ship an update
**And** a structural check fails CI when a supported harness has no page or a page names a probe that does not exist

## Epic 4: Compare environments and see findings

A person compares any two environments, or an environment and a profile, in either direction, sees findings on each, saves local reports, and asks which hosts carry an entry. Usher shows differences and facts; people judge which is better.

### Story 4.1: Compare two environments in either direction

As an expert,
I want to compare any two of my environments, or mine with a colleague's from a report,
So that I see every difference without deciding in advance which side is right (FR-15).

**Acceptance Criteria:**

**Given** two environments
**When** I run `usher compare <a> <b>`
**Then** each entry is present, missing, different or extra, with provenance on both sides
**And** swapping the sides swaps missing and extra and changes nothing else

**Given** a harness with a resolved view
**When** the comparison runs
**Then** it compares settings in effect, and compares files only where no resolved view exists; the output says which

**Given** list entries whose order the harness ignores
**When** they are compared
**Then** they compare as sets; order is compared only where harness knowledge says the harness uses it
**And** path tokens (`${HOME}`, `${CONFIG_DIR}`, `${REPO}`, `${USER}`) are resolved on each side before comparing

### Story 4.2: Findings on keys, layers and undocumented settings

As a person,
I want findings on my environment for invalid or retired keys, conflicts and settings nobody documented,
So that what I forgot and what went stale shows up without my looking (FR-16, SM-3).

**Acceptance Criteria:**

**Given** an entry naming a key the harness has retired
**When** findings run
**Then** it is flagged for every version from the first release that retired it, with what the harness now does with it, from the knowledge item that records that version; codex `approval_policy = "untrusted"` is flagged as a hard error

**Given** conflicts between layers, or between repo and directory settings
**When** findings run
**Then** each conflict is a finding naming both sources

**Given** profiles or docs the person names
**When** findings run
**Then** a setting set in the environment and in none of them is a finding

**Given** any finding
**When** it is shown
**Then** it names what the harness does with the key (warns, aliases, fails or skips silently), the knowledge item and its reference

### Story 4.3: Locks weaker than they look, and hazard checks gated on probes

As a person on a managed host,
I want to know when an organization deny can be overridden from a lower layer,
So that I do not mistake a soft rule for a lock (FR-16, addendum §1.1).

**Acceptance Criteria:**

**Given** an organization soft deny that a lower layer's allow entry, such as an auto-mode allow, can override
**When** findings run
**Then** it is flagged, and only managed-only keys are reported as locks

**Given** a hazard check that rests on a harness claim
**When** usher runs on a harness version
**Then** the check runs only if its probe (Story 1.13) has passed on that version; otherwise the person sees that the check is withheld and why
**And** the `$defaults` check ships with this story: an `autoMode` list that omits `$defaults` is flagged as dropping the built-in rules (probe confirmed on Claude Code 2.1.319)

### Story 4.4: Local reports in a published schema

As a person,
I want to save any map or comparison as a file in a published, versioned format,
So that I can share it by hand and anyone can check what it holds (FR-28).

**Acceptance Criteria:**

**Given** a map or comparison
**When** I run `usher report save`
**Then** the file validates against the published schema for its version, owned by `usher-core` (the same schema the collector accepts in Epic 12, AD-13), with a ULID and RFC 3339 UTC timestamps
**And** it holds only reported fields; labels appear only if I include them

**Given** this story
**When** it lands
**Then** no code path that sends a report off the host exists; saving is local only

### Story 4.5: Which environments carry entry X

As an expert,
I want to ask which hosts or directories carry a given entry, and which run a stale copy of a tool,
So that I can answer the team's questions from the reports I have (FR-18).

**Acceptance Criteria:**

**Given** a set of local reports from several hosts
**When** I ask which carry entry X, or which run a stale copy of tool Y
**Then** usher answers over those reports, and lists the hosts it could not see
**And** the same query through the light collector arrives in Epic 12

## Epic 5: Hear when the ground moves

After install usher stays quiet and watches. When the ground moves under a seat it tells the person what changed and where, keeps a dated history of each environment, and stays within its footprint budgets. It changes nothing. The person starts the background service here; installing it as a login item is a change, so it arrives with the journal in Epic 6.

### Story 5.1: The background service, idle between events

As a person,
I want usher's background service to cost my laptop almost nothing,
So that I can leave it running and forget it (NFR16, AD-9).

**Acceptance Criteria:**

**Given** `usher daemon`, started by the person
**When** it runs
**Then** it wakes only on OS file-change notices (`notify` with `macos_fsevent`) for the locations it maps, with no timers
**And** it updates the map through the one `usher-core` update path (AD-18)

**Given** a laptop on battery
**When** the footprint tests run
**Then** resident memory idle for an hour is 10 MB or less, idle CPU is under 0.1% averaged over the hour with no timer wakeups, and a changed file is recorded within 200 ms of CPU

**Given** more than 256 directories to watch
**When** the service starts
**Then** it watches 256 by default and names the paths it dropped, rather than going quiet
**And** `usher status` reports the service's own memory, CPU and watched-path count

### Story 5.2: Change notices

As a person,
I want to be told when a harness self-updates, a session runs an old binary, device management reinstalls an app, or a setting in effect changes,
So that I can re-find my seat when the ground moves (FR-13).

**Acceptance Criteria:**

**Given** each of those four events
**When** it happens
**Then** a notice names the environment, what changed, and the before and after versions or values (values only where AD-8 allows)

**Given** default settings
**When** notices accumulate
**Then** they are batched, with at most one unprompted notice a day, and I can set which kinds interrupt me

**Given** the service while watching
**When** any notice fires
**Then** it has written only usher's own state, has called no command on the harness or installer never-call lists, and the invariant test passes

### Story 5.3: Dated settings history

As a person,
I want to see when each setting in effect changed,
So that I can line the change up with the daily usage and cost my provider reports (FR-14, SM-2).

**Acceptance Criteria:**

**Given** a mapped environment
**When** time passes
**Then** usher keeps dated snapshots of its map, at most one a day per environment unless I ask for more or a change notice fires, holding derived state only

**Given** two consecutive snapshots
**When** I ask for history
**Then** usher shows the differences between them with their dates

**Given** usher's state on disk
**When** it reaches 100 MB, backups included
**Then** usher asks me to prune, and never evicts snapshots on its own

### Story 5.4: What usher is doing and has looked at

As a person,
I want to ask usher at any time what it is doing, what it has looked at, and what my consent state is,
So that nothing usher does about me is hidden from me (NFR4, NFR18).

**Acceptance Criteria:**

**Given** usher has been observing since install, including a silent install by device management
**When** I open usher for the first time
**Then** it tells me it has been observing since install and shows what it looked at

**Given** any time after
**When** I run `usher status`
**Then** it shows what the service is watching, what it has read, the proposals waiting for my confirmation, and my consent state, and lets me change each

## Epic 6: Install, update and remove safely, with undo

Every change usher makes goes through the journal and the safe-execution checks, so a person can see it before confirming, trust it to touch only what its dry run listed, and undo it. With that in place a person installs and updates tools and usher itself, rolls back any change, installs the background service as a login item, and removes usher. In this epic changes are confirmed from the CLI; the journal records the surface either way (AD-4). Each story carries the addendum §7 rows it makes true, as failure-injection tests against a scratch home.

### Story 6.1: The change journal and the one-writer lock

As a person,
I want every change usher makes recorded before it starts, and only one change run at a time,
So that nothing usher does to my host goes unrecorded or collides with another run (FR-63, FR-64, AD-2, AD-7).

**Acceptance Criteria:**

**Given** `usher-store`
**When** a change is about to run
**Then** its journal entry (ULID, time, surface, my confirmation, the command or write, every target as a resolved absolute path, the state before, the planned undo or manual step) is written and flushed to disk before the change starts, and completed after it with the state after and the exit status
**And** the journal is insert-only; a rollback is a new entry, never an edit; a credential is recorded as present, never by value

**Given** a journal that cannot be read or written
**When** any change is requested
**Then** no change runs, and usher says why

**Given** a change run in progress
**When** a second run starts on the same host
**Then** it waits for the person or refuses, and never runs alongside the first, under a `std::fs::File::lock` held by the running usher process
**And** a lock whose process is gone is released, and the release is journaled

### Story 6.2: Dry run and target checks for usher's own writes

As a person,
I want to see every target a change will touch, and have usher refuse targets that could widen it,
So that a bad variable or path can never turn a settings write into damage (FR-64).

**Acceptance Criteria:**

**Given** a planned write
**When** usher prepares it
**Then** it shows a dry run listing every target and a diff, and runs only on my confirmation

**Given** a target built from an empty variable, holding an unexpanded `$VAR`, `${VAR}`, `%VAR%` or leading `~`, a glob character the change did not ask to expand, or resolving to `/`, the home directory or a parent of it
**When** usher checks it
**Then** the change is refused before any write, and nothing is journaled as done

**Given** a write kind and its list of allowed paths (for a settings write, the harness's known settings files from harness knowledge)
**When** a target reached through `..` or a symlink leaves the list
**Then** it is refused; a symlink whose target is on the list is written through and kept as a link, never replaced

**Given** a change that would touch more targets than its dry run listed
**When** it reaches the first extra target
**Then** it stops there

### Story 6.3: The five-step write: backup, atomic replace, format kept

As a person,
I want every file usher writes to be backed up first and replaced whole or not at all,
So that a crash, a full disk or a bad edit can never leave a settings file half written or lost (FR-64, AD-3, AD-5).

**Acceptance Criteria:**

**Given** a confirmed write
**When** it runs under the lock
**Then** usher records the intent with each target's hash, checks free space, backs the file up and checks the backup reads back, writes a temporary file in the same directory, parses it in the file's own format, flushes it, and renames it into place, so a harness that reloads the file sees the old version or the new one
**And** a target whose hash changed since the dry run stops the change and shows the conflict

**Given** the harness's own writer exists for the key (codex app-server `config/value/write`, `config/batchWrite`)
**When** usher writes it
**Then** it uses that writer; otherwise it edits with `toml_edit`, `jsonc-parser` (`cst`) or `yamlpatch`, and a byte check shows everything outside the edited value unchanged, including comments, formatting, permissions, ownership, encoding and line endings

**Given** a removal
**When** it runs
**Then** the target moves to the backup store; usher never deletes

**Given** the failure cases for this story (disk full, read-only or locked target, content that does not parse, a JSONC or TOML file with comments outside the changed key, a target edited by hand between dry run and write)
**When** each is injected
**Then** the change stops or keeps the file intact as addendum §7 requires

### Story 6.4: Credential files, mixed files and trust records

As a person,
I want usher never to touch my credentials or trust records, even in a file that also holds settings,
So that letting usher write a setting never risks my login or a trust decision (FR-64, AD-7, AD-8).

**Acceptance Criteria:**

**Given** a file harness knowledge marks as holding credentials (for example a harness auth file)
**When** a write is aimed at it
**Then** it is refused and no backup is made

**Given** a mixed file holding settings beside account data and trust records (for example `~/.claude.json`)
**When** usher writes one key harness knowledge lists as writable for it
**Then** only that key changes, every other key is byte for byte the same, and the backup is a key-level backup in the owner-only store

**Given** a write that would change a folder-trust record
**When** usher checks it
**Then** it is refused

**Given** the backup store
**When** anything is written to it
**Then** it is readable only by the person, never reported, kept until the person removes it, and its size is shown in `usher status`

### Story 6.5: A run stopped part way can be resumed or rolled back

As a person,
I want a change that was interrupted to leave every file old or new and tell me where it stopped,
So that a crash or a closed laptop never leaves my seat in an unknown state (FR-64, AD-3).

**Acceptance Criteria:**

**Given** the process killed between writing the temporary file and the rename
**When** usher next starts
**Then** the target is the old version, the journal shows the step unfinished, and a crash marker is raised

**Given** a crash marker
**When** I next use usher from the CLI
**Then** it shows which steps finished and offers to resume or roll back; the background service never resolves it

### Story 6.6: Roll back any change, or a whole run

As a person,
I want to undo any change usher made, or a whole run, from the journal,
So that trying a setting never commits me to keeping it (FR-63).

**Acceptance Criteria:**

**Given** journal entries
**When** I run `usher rollback <entry or run>`
**Then** undo runs in reverse order, each fitted to its kind: a file write restores its backup; other kinds follow their own stories

**Given** a target changed since usher wrote it
**When** it is rolled back
**Then** it is shown as a conflict and left alone unless I choose

**Given** later entries that depend on the one being undone
**When** I roll it back
**Then** they are listed and undone first, or the rollback stops

### Story 6.7: Delegated runs through the runner port

As a person,
I want commands usher hands to brew, mise, git, sideshow or a vendor installer run exactly as shown and recorded,
So that a delegated run is as safe and undoable as a write usher makes itself (FR-64, AD-15).

**Acceptance Criteria:**

**Given** `usher-runner`
**When** a delegated run is prepared
**Then** the exact command and its source are shown before I confirm, and arguments are passed as a list, never a string built from values; a value with spaces, quotes or `;` is one argument
**And** any path usher puts in an argument passes the Story 6.2 target checks

**Given** an installer fetched and piped into a shell
**When** its source is not pinned and verified
**Then** it is refused

**Given** a run
**When** it finishes or exits nonzero part way
**Then** the journal holds the installer's known paths before and after, the difference observed, and the exit status, and a failure is reported with the paths that changed

### Story 6.8: Install a harness or tool on confirmation

As a newcomer,
I want to install a missing harness or tool through the installer that owns it,
So that I get it the way the team supports, with usher showing me exactly what it will run (FR-11).

**Acceptance Criteria:**

**Given** a tool Story 2.1 shows as missing
**When** I ask usher to install it
**Then** it shows the installer's exact command and source, and runs it through the runner port only on my confirmation

**Given** an install that needs a credential
**When** it reaches that point
**Then** the vendor's own login runs; usher never copies or holds a credential

**Given** an installer that also configures the harness
**When** it would write settings
**Then** that write is shown and confirmed like any other change

### Story 6.9: Update a harness or tool on confirmation

As a person,
I want to update a harness or tool after seeing what will change and which sessions use it,
So that an update never pulls the ground out from under a running seat by surprise (FR-44).

**Acceptance Criteria:**

**Given** a tool Story 2.1 shows as behind
**When** I ask usher to update it
**Then** it shows the from and to versions and the running sessions that use the binary (Story 2.4), and lets me choose when

**Given** an installer that can pin the earlier version
**When** the update is journaled
**Then** its undo reinstalls the earlier version through that installer; where none can, the entry names the manual step, and I see it before confirming

### Story 6.10: Usher updates itself from a local signed feed

As a person,
I want usher to update itself only from a feed it has verified, keeping the previous binary,
So that a usher update is as reversible as any other change (FR-44, AD-19).

**Acceptance Criteria:**

**Given** a signed feed served from a local directory in a scratch home
**When** usher checks it
**Then** the feed's signature is verified before anything is downloaded or installed; an unverifiable feed is refused, and a release whose version is not higher than the installed one is refused

**Given** a verified update
**When** I confirm it
**Then** it runs as a journaled transaction: before any migration the store is backed up beside the previous binary, so the undo restores both; migrations are forward-only and run under the lock

### Story 6.11: Self-update through the owning installer, or from a team-named feed

As a person,
I want usher to update itself the way it was installed,
So that I get fixes without reinstalling by hand, and usher never needs a second update path for an install brew already owns (FR-44, AD-19).

**Acceptance Criteria:**

**Given** usher installed through brew
**When** I ask usher to update itself
**Then** it updates through brew exactly as Story 6.9 updates any tool: from and to versions shown, running usher modes named, confirmed and journaled; where brew cannot reinstall the earlier version, I see "no undo" before confirming
**And** where the repo is private, brew uses the forge's own login (for example the person's `gh` login); usher never holds or copies that token

**Given** usher installed directly (a downloaded binary, a collector host, a CI runner)
**When** the team names a feed (a URL or a directory, for example served by its collector)
**Then** usher updates from it through Story 6.10's verification and rollback

**Given** no team-named feed and no owning installer
**When** usher checks for updates
**Then** it says it has no update source; it never falls back to a built-in URL

### Story 6.12: Install the background service as a login item

As a person,
I want usher's background service to start when I log in,
So that watching continues without my starting it each time (Epic 5).

**Acceptance Criteria:**

**Given** the per-user launchd agent for `usher daemon`
**When** I confirm installing it
**Then** its plist in `~/Library/LaunchAgents` is written and loaded as a journaled change, with unload and removal as the undo
**And** the plist runs usher from its stable install path (for example `/opt/homebrew/bin/usher`), never a versioned one that an installer's cleanup can remove

### Story 6.13: Usher recognizes its own writes

As a person,
I want the background service to stay quiet about changes usher itself just made,
So that I am told about the ground moving, not about my own confirmed changes (AD-18).

**Acceptance Criteria:**

**Given** a change usher journaled
**When** the background service sees the file change it caused
**Then** it updates the map through the one update path, matching the change to its journal entry by target and hash, and raises no change notice
**And** a change to the same file by anything else still raises a notice

### Story 6.14: Remove usher

As a person,
I want to remove usher and choose what it leaves behind,
So that trying usher never leaves residue I did not choose (FR-67).

**Acceptance Criteria:**

**Given** `usher remove`
**When** it starts
**Then** before removing anything it lists the background service, the binaries, usher's own state (journal, backups, snapshots, reports) and every journaled change still in effect

**Given** that list
**When** I choose to keep usher's changes, or roll back some or all of them first
**Then** rollback runs in reverse order with conflicts shown, as in Story 6.6
**And** tools usher installed stay unless I pick each one for removal through its own installer

**Given** removal completes
**When** a test inspects the host
**Then** no usher process, login item, scheduled job or OTLP endpoint remains, and the journal and backups are kept, with usher saying where, unless I asked for them to be deleted

### Story 6.15: Every change-safety failure case has a test

As a usher maintainer,
I want each row of addendum §7 traced to the test that injects it,
So that a safety rule cannot quietly lose its test (FR-63, FR-64).

**Acceptance Criteria:**

**Given** addendum §7
**When** the trace check runs in CI
**Then** each row names its test; rows made true by this epic or an earlier one pass (observation before consent is Story 1.9), and rows that belong to later epics (a write to a personal entry, Epic 8; the undo of a clone, Epic 9; an in-harness agent confirming, Epic 10) are listed as pending with the epic that closes them
**And** a row with no test and no pending epic fails the check

### Story 6.16: The public tap and a default feed

As a person outside this fleet,
I want to install and update usher from a public homebrew tap,
So that I can start without access to a private repo (FR-44, SM-7).

**Acceptance Criteria:**

**Given** the going-public decision (PRD Q11)
**When** it is made
**Then** a formula in the public tap installs signed, notarized usher, and Story 6.11's brew path updates through it
**And** until then this story stays blocked; usher ships with no default feed rather than a placeholder

## Epic 7: Capture and load profiles

An expert captures an environment into a portable profile, filters out what is personal, notes why each entry matters, and hands it on; a newcomer loads it and sees, for their own versions, what matches, what differs and what is invalid. Loading changes nothing. Profiles are native fragments with one shared MCP form (AD-11).

### Story 7.1: Profile format and profile sources

As an expert,
I want profiles to be plain directories in each harness's own format, read from files, directories or git repos I name,
So that a profile is something my team can review in a pull request and never lags a harness release (AD-11).

**Acceptance Criteria:**

**Given** a profile directory
**When** usher reads it
**Then** it finds `profile.toml` (name, version, harnesses covered with a version range each, tools it needs, an optional shared `mcp_servers` table, an optional note per entry) and one fragment per harness in that harness's own format and keys
**And** a fragment entry for an MCP server overrides the shared entry for that harness

**Given** profile sources I name (a file, a directory, or a git repo laid out as `profiles/<name>/`)
**When** usher reads them
**Then** a git source's profile version is its commit, and a file or directory source uses the `version` field; a git source is fetched into usher's own state, which is not a change

**Given** two profiles that set the same key
**When** both are loaded
**Then** the clash is shown as a conflict, never merged silently

### Story 7.2: Capture an environment into a draft profile

As an expert,
I want to capture an environment, or chosen parts of several, into a draft profile,
So that I hand on my setup without describing files and JSON from memory (FR-19, FR-20, UJ-2).

**Acceptance Criteria:**

**Given** one or more mapped environments
**When** I run `usher capture`
**Then** a draft profile is created in usher's state, each entry recording its source environment, layer and harness version
**And** it holds no credential value and no file content

**Given** a value containing my home, the harness config directory, a repo root or my user name
**When** it is captured
**Then** usher proposes the matching path token (`${HOME}`, `${CONFIG_DIR}`, `${REPO}`, `${USER}`) or a glob pattern, and I accept or edit each

### Story 7.3: Filter, annotate and save a profile

As an expert,
I want to drop entries, add a note per entry and review the draft before anyone sees it,
So that nothing personal or unexplained leaves my host (FR-21, FR-20).

**Acceptance Criteria:**

**Given** a draft profile
**When** I review it
**Then** I can drop entries and add a note to each, and the draft is marked unreviewed until I save it
**And** an entry carrying a literal path that names my home is flagged

**Given** a reviewed profile
**When** I save it to a path I choose (for example a team repo)
**Then** writing it is a journaled change confirmed like any other (Epic 6); nothing is shared that I have not reviewed

### Story 7.4: Load a profile and see it against my environments

As a newcomer,
I want to load a profile and see each entry against my own environments and versions,
So that I know what already matches and what is wrong for me before I adopt anything (FR-22, FR-17, UJ-2 edge case).

**Acceptance Criteria:**

**Given** a profile from a source I follow
**When** I load it
**Then** nothing on my host changes, and each entry shows matches, differs, missing, or invalid for my version, with findings for my versions

**Given** an entry outside its stated version range for the target environment
**When** it is shown
**Then** it is marked before placement; an entry with no version range is marked unchecked
**And** the comparison is the Story 4.1 comparison, with path tokens resolved for my environment

### Story 7.5: Tools and toolchain a profile declares

As a newcomer,
I want a profile to declare the tools and versions my seat needs, and to see where my host falls short,
So that "install these five things at these versions" is a checklist usher keeps, not a README (FR-45).

**Acceptance Criteria:**

**Given** a profile's declared tools, toolchain and version ranges
**When** usher compares my host with it
**Then** a tool missing or outside its range is a finding, with the install or update that would fix it (Stories 6.8, 6.9)

**Given** a team that declares tools through its tool manager (for example a `mise.toml`)
**When** the profile points at it
**Then** usher reads it; generating one for my host runs only on confirmation, as a journaled change

## Epic 8: Apply, restore and catch up

A person sees exactly where an entry goes for their harness and version, has usher apply it with write safety, checks that it took effect, brings a seat back after a reinstall or new machine, catches up on followed profiles, and produces the artifact an organization's device management deploys. Every write goes through Epic 6. This epic acts on the host usher runs on; remote apply is v2 (Epic 14).

### Story 8.1: Exact placement for an entry

As a person,
I want to see, for an entry and a directory, the exact file, layer and snippet in that file's format,
So that I place a setting right the first time (FR-24, UJ-1).

**Acceptance Criteria:**

**Given** a profile entry and a target directory
**When** I ask where it goes
**Then** usher shows the file path, the layer and the snippet in that file's own format, for my harness version

**Given** a key a layer ignores (for example an `autoMode` entry, which Claude Code ignores in a project file)
**When** placement is computed
**Then** that layer is never offered

**Given** a higher layer that will override the placed value
**When** placement is shown
**Then** usher names the overriding layer before I apply

### Story 8.2: Proposals hold operations, not diffs

As a person,
I want a proposed change to be checked against my files as they are when I apply it, not as they were when it was proposed,
So that a stale or agent-written diff can never overwrite something that changed since (AD-16).

**Acceptance Criteria:**

**Given** a proposal
**When** it is stored
**Then** it holds structured operations (adapter, file, key, new value) and each file's base hash, not a text diff

**Given** a proposal whose base hash no longer matches a file
**When** I go to apply it
**Then** usher recomputes the diff, shows what changed, and asks again

### Story 8.3: Apply with write safety, and personal entries

As a person,
I want usher to write an entry, a profile or a recommendation into my settings after I confirm,
So that adopting a setting is one confirmed step that I can undo (FR-49).

**Acceptance Criteria:**

**Given** a proposal
**When** I confirm it
**Then** usher writes it through the Epic 6 transaction, with dry run, diff, journal and rollback

**Given** a value I set by hand that the entry would replace
**When** the dry run is shown
**Then** it is shown as a conflict; nothing is overwritten silently

**Given** a managed or device-management layer
**When** a write is aimed at it
**Then** it is refused; usher is never the sole configuration authority

**Given** a map or comparison
**When** I mark or unmark an entry as personal
**Then** usher never writes, recommends against, or treats as a difference to resolve a marked entry, and a write aimed at one is refused and shown as personal (addendum §7)

### Story 8.4: Verify after applying, and the adoption record

As a person,
I want to see whether an applied entry is now in effect, and have usher remember where it came from,
So that I know it worked, and later learn if it drifted (FR-25, FR-23).

**Acceptance Criteria:**

**Given** an applied entry
**When** usher re-maps the directory
**Then** it reports the entry as in effect, overridden (naming the layer), or not loaded (with the reason)

**Given** an entry verified in effect
**When** it is recorded
**Then** an adoption record notes the profile and version it came from

**Given** an adopted entry whose setting in effect later changes
**When** usher maps the environment
**Then** it is reported as drift against the adoption record

### Story 8.5: Restore a seat

As a person back from a reinstall, a new machine or time away,
I want usher to list every difference from my declared seat and restore the ones I confirm,
So that getting back to work takes one guided pass (FR-50).

**Acceptance Criteria:**

**Given** a declared seat (the profiles I adopted and the tools they declare)
**When** I run `usher restore`
**Then** usher lists each difference and the step that would restore it (an install, update or settings write), and applies only the steps I confirm

**Given** a device-management reinstall that dropped custom profiles (Story 2.5)
**When** I restore
**Then** those profiles are among the restorable steps

### Story 8.6: Catch up on profiles I follow

As a person,
I want to see what changed in the profiles I follow since I last loaded them, and why,
So that I keep up with my team without rereading everything (FR-51).

**Acceptance Criteria:**

**Given** each profile source I follow
**When** usher records a load
**Then** it keeps the profile versions I last loaded, per source

**Given** newer versions in a source
**When** I run `usher catch-up`
**Then** each change names its profile version, author and note; a profile stays untrusted until I load it, and loading changes nothing
**And** I can adopt each change through Story 8.3, or keep my own value

### Story 8.7: An organization drop-in

As an organization's admin,
I want usher to produce the artifact our device management deploys when we require a setting,
So that required settings go through our own channel, not through usher (FR-52).

**Acceptance Criteria:**

**Given** settings the organization wants required (for example a managed-settings drop-in for Claude Code)
**When** I run `usher drop-in`
**Then** usher writes the artifact to a file I name, in the format the harness's managed layer reads
**And** usher never deploys it

## Epic 9: Newcomer bootstrap from nothing

A newcomer goes from an empty host to a working seat in one guided run that can stop and resume, attaching the supporting services, method content and repositories they need. Every step is a change through Epic 6, built on the install and apply stories before it.

### Story 9.1: A guided, resumable setup run

As a newcomer,
I want one guided run from nothing to a working seat that I can stop and pick up again,
So that my first day ends with AI working in a repo, not with a README (FR-46, SM-1, SM-8).

**Acceptance Criteria:**

**Given** a host with no harness installed
**When** I run `usher setup` (from the command line, since no harness is there yet)
**Then** it walks the steps (tools from a profile, harness install, settings, repo) and runs each on confirmation through the stories that own it

**Given** any step
**When** it is rerun after it finished
**Then** it changes nothing

**Given** a step that fails
**When** usher reports it
**Then** it names the step, the command or check that failed with its exit status or error, and at least one next action (retry, skip, or a named manual step), through `UsherError`

**Given** the run is stopped part way
**When** I start it again
**Then** it resumes from the journal at the first unfinished step

**Given** the first session
**When** it ends
**Then** at least one harness works in at least one repo, launched by me

### Story 9.2: Attach supporting services

As a newcomer,
I want to attach the MCP servers, issue trackers and accounts my seat needs,
So that my harness can reach the team's services from day one (FR-47).

**Acceptance Criteria:**

**Given** a service a profile names
**When** I attach it
**Then** usher runs the service's own setup or login on confirmation, as a journaled change, and holds no bearer credential for it (ADR-009)

### Story 9.3: Bootstrap method content through sideshow or eos

As a newcomer,
I want my team's method content installed the way my team installs it,
So that my seat has the same skills and packs as everyone else's (FR-47).

**Acceptance Criteria:**

**Given** packs or method content a profile names
**When** I confirm
**Then** usher drives sideshow or eos through the runner port to install them, as a journaled change, and does not re-implement either

### Story 9.4: Get a repository I need, without git fluency

As a newcomer not fluent in git,
I want usher to get a repository onto my host and open it in my harness,
So that I can start work without learning git first (FR-48).

**Acceptance Criteria:**

**Given** a repository I need
**When** I ask usher for it
**Then** usher names each git command before running it and runs it on confirmation through the runner port, using the forge's own login and the host's own credential; it never copies a credential between hosts

**Given** a clone usher made
**When** I undo it
**Then** it is moved aside only when nothing in it is new since usher made it (no commits, modified or untracked files); otherwise it is shown as a conflict and nothing moves (addendum §7)

## Epic 10: Work from inside the harness

From Claude Code, and from the Claude desktop app where it takes MCP servers, a thin MCP surface maps, compares, explains findings, and proposes and applies changes, each confirmed as on any surface (AD-4). It stays within its token budgets.

### Story 10.1: The MCP server for maps, comparisons and findings

As a newcomer who starts inside a harness,
I want to ask my harness what my settings are and why a finding matters,
So that I can learn my seat without leaving the conversation (FR-62).

**Acceptance Criteria:**

**Given** `usher mcp` over stdio (`rmcp`, `server` and `transport-io`)
**When** an MCP client connects
**Then** it offers tools to map, compare and explain findings, backed by the same `usher-core` functions as the CLI

**Given** the tool descriptions
**When** they are measured
**Then** they total 1,500 tokens or less, and a test fails when they grow past it

**Given** a response
**When** it would exceed 4,000 tokens
**Then** it is paged, with a way to ask for the next page
**And** no response carries a value AD-8 keeps out

### Story 10.2: Register usher's MCP server in Claude Code and the desktop app

As a newcomer,
I want usher added to my harness's MCP servers when I confirm,
So that I can reach usher from inside the harness without editing JSON (FR-62).

**Acceptance Criteria:**

**Given** Claude Code, or the desktop app where it takes MCP servers
**When** I confirm registering usher
**Then** the registration is written through Story 8.3's apply path as a journaled change, with removal as its undo

### Story 10.3: Propose and apply from inside the harness

As a person working with an agent,
I want the agent to propose and apply a change the same way I would from the command line,
So that an agent working for me can set up my seat without a weaker path (FR-62, AD-4, operator ruling 2026-11-08).

**Acceptance Criteria:**

**Given** an MCP tool call that proposes a change
**When** the agent confirms it on my behalf
**Then** it is applied through the same Story 8.2 and 8.3 path as from the CLI, and the journal records the confirming surface and the MCP client's name (addendum §7)

**Given** a crash marker (Story 6.5)
**When** I am working from inside the harness
**Then** it can be resolved from MCP as from the CLI; the background service still never resolves it or applies anything

## Epic 11: See effects and get recommendations

A person sees measured outcomes beside their settings, such as cache reuse per session, and gets recommendations drawn from the profiles they follow and their own records. Effects are facts; judging them stays human.

### Story 11.1: Cache reuse per session

As a person,
I want to see, per session, how much input came from cache versus was written at full cost,
So that I can tell when a setting is costing me cache reuse (FR-26, UJ-1).

**Acceptance Criteria:**

**Given** a harness's session records
**When** usher reads them
**Then** it parses only named fields (timestamps, working directory, model, token and cache counts, tool and permission outcomes, error and throttle markers), and a test with marker text in every content field shows no output contains it

**Given** a session
**When** cache reuse is computed
**Then** it shows the share read from cache and the share written at full cost, with the model and environment in effect
**And** a model missing from the price table is reported unpriced, not dropped, and throttled or failed calls are counted apart

### Story 11.2: Effects beside settings

As an expert,
I want to see effects next to a comparison of two environments or next to the dated settings history,
So that a settings difference and its outcome can be read together (FR-27, SM-2).

**Acceptance Criteria:**

**Given** a comparison or a settings history
**When** effects are shown beside it
**Then** each effect states the sessions and dates it covers and the backend (service operator and model)
**And** effects from different backends are never combined in one figure; no view ranks people

### Story 11.3: Recommendations from profiles and my own records

As a person,
I want usher to bring me changes worth making, each with its reason and what usher could not see,
So that I improve my seat without hunting for what to change (FR-55).

**Acceptance Criteria:**

**Given** the profiles I follow and my own records (for example permission denials a harness recorded)
**When** recommendations are produced
**Then** each states its source, reason, the versions and backend it assumes, and what usher could not observe; recommendations that differ by backend say so

**Given** cost-bearing settings (cache settings that do not fit my time between turns, MCP connections configured and never used, no spend cap set)
**When** they apply to me
**Then** they appear as cost-reduction recommendations

**Given** an allow entry offered as a recommendation
**When** it is shown
**Then** it is marked as a loosening

**Given** my notice settings (Story 5.2)
**When** a recommendation is new
**Then** it is pushed only for the kinds I set to notify me; the rest wait in the list
**And** accepting one goes through Story 8.3

### Story 11.4: Credential scan, off by default

As a person,
I want to find plaintext keys on my host if I ask, without usher reading their values,
So that I can move them somewhere safer (NFR7, addendum §5).

**Acceptance Criteria:**

**Given** default settings
**When** usher runs
**Then** the credential scan is off

**Given** I turn it on
**When** it runs
**Then** it reports the location and type of credential-shaped material only, never a value, and never writes it into the journal, a report or a log

## Epic 12: Team view through the light collector

The only epic where data leaves the host. Reporting stays off until a person turns it on for a named collector after seeing what will be sent. A team runs the collector itself; usher operates no service. The epic opens with the test that proves the default sends nothing, before any sending code exists (AD-13, AD-14).

### Story 12.1: Prove the default sends no report

As a person,
I want a test, not a promise, that usher sends no report off my host unless I turn reporting on,
So that installing usher never means being tracked (FR-29, SOUL §1).

**Acceptance Criteria:**

**Given** the usher binary in its default state, before any sending code exists
**When** the no-egress test runs every mode under a `sandbox-exec` profile that denies network except the endpoints the behavioral profile lists for the version check (Story 2.1), with a refused-connection control showing the profile's own refusal
**Then** no connection other than those listed is attempted, and no request carries a report field

**Given** the build
**When** reporting is off
**Then** the off switch gates the code path: the sender is not reachable, not merely skipped at send time

**Given** `DO_NOT_TRACK` set in the environment
**When** usher runs
**Then** reporting stays off on that host, whatever else is configured, and usher says so in `usher status`

**Given** the published docs
**When** this story lands
**Then** a "we do not collect" list is part of the contract, beside the reported fields

### Story 12.2: The light collector receives one versioned schema

As a team,
I want to run a small collector on infrastructure we control that accepts only the published report fields,
So that we see our fleet without anyone outside the team in the path (FR-37, FR-38).

**Acceptance Criteria:**

**Given** `usher collector` with a TLS certificate the team provides
**When** it starts
**Then** it serves HTTPS with rustls, keeps its own SQLite store, and runs without any usher-operated service

**Given** a report in the current schema version or the one before
**When** it arrives
**Then** it is accepted; a report with any field not in the reported fields is rejected whole, and no file content, prompt content or credential value is ever stored

**Given** one collector
**When** it is set up
**Then** it serves one affiliation (one organization or team)

### Story 12.3: Host keys the collector can revoke

As a team,
I want each host to authenticate with its own key, which we can revoke and the host can re-mint,
So that one lost laptop never exposes anyone else's reporting (FR-40, ADR-009).

**Acceptance Criteria:**

**Given** a host enrolling with a collector
**When** it is enrolled
**Then** it generates an ed25519 key pair, keeps the private key in the OS keychain (`keyring`), and registers the public key; the key means nothing outside that collector (the ADR-009 audience test)

**Given** one host's key revoked at the collector
**When** that host and another report
**Then** the revoked host is refused and the other is unaffected
**And** the revoked host can re-mint a key and re-enroll; no credential is ever copied between hosts

### Story 12.4: Consent before the first report, and withdrawal at any time

As a person,
I want to see exactly what will be sent, to whom and how to stop, before my host reports anything,
So that turning reporting on is my informed choice (FR-39, FR-37).

**Acceptance Criteria:**

**Given** a collector I have not reported to
**When** reporting would start
**Then** usher shows the fields that will be sent, the collector, and how to stop, and nothing leaves the host before I confirm for that collector

**Given** an organization that turns reporting on through its own device-management channel
**When** usher sees that setting
**Then** it records that the organization turned it on, and tells me before the first report, with the same disclosure

**Given** I withdraw
**When** I run `usher reporting off`
**Then** sending stops at once, and consent and withdrawal events are recorded on the host

### Story 12.5: Send on events from an outbox

As a person who turned reporting on,
I want reports queued locally and sent after a change, at start and when the network returns,
So that reporting adds no timers and loses nothing while I am offline (AD-14, FR-29, FR-38).

**Acceptance Criteria:**

**Given** reporting on for a collector
**When** a change is journaled, usher starts, or the network returns
**Then** pending reports are sent from the outbox in usher's state, with no polling timer

**Given** the enabled state
**When** the egress test runs against a real collector
**Then** every request carries only reported fields for the schema version in use

### Story 12.6: Ask the collector across hosts

As an expert,
I want to ask the collector which hosts carry an entry, which run a stale tool, and how findings and effects compare across hosts,
So that I can show the team, with evidence, what is in use and what works (FR-41, FR-18, UJ-2).

**Acceptance Criteria:**

**Given** reports from several hosts
**When** I ask the Story 4.5 questions through the collector
**Then** each answer lists the hosts it could not see

**Given** any query
**When** results are produced
**Then** they compare environments and never rank people; rollups are aggregate by default, and per-person drill-down is available only after the team records that decision
**And** a person sees their own findings and recommendations before anyone else does

### Story 12.7: See and delete my own data at the collector

As a person,
I want to see what the collector holds about my hosts and delete it,
So that anything held about me stays mine to inspect and remove (FR-39, NFR18).

**Acceptance Criteria:**

**Given** a collector my hosts report to
**When** I ask for my data
**Then** the collector returns everything it holds about my hosts, authenticated by my host keys

**Given** I ask for deletion
**When** the collector acts on it
**Then** the data is deleted and a deletion record is kept without the data

### Story 12.8: Retention

As a team,
I want to set how long the collector keeps data, with a short default,
So that old per-person data does not pile up (FR-42).

**Acceptance Criteria:**

**Given** a collector with no retention set
**When** it runs
**Then** it keeps data for 90 days

**Given** data older than the retention period
**When** the collector prunes, on its next write rather than on a timer
**Then** that data is deleted and a deletion record is kept without the data

## Epic 13: Richer activity and reporting levels [v2]

A person sees more of how each environment behaves, and a team that reports can choose how much, from metrics only to full OTEL, with usher as the local endpoint that forwards only what the chosen level allows.

### Story 13.1: Activity effects

As an expert,
I want activity per environment and project (tool calls, tokens, errors, model changes in a session, time between turns against the cache lifetime, cache resets, instruction-file size over time),
So that I can see how a setup behaves, not only what it is set to (FR-53).

**Acceptance Criteria:**

**Given** a harness's session records
**When** activity is computed
**Then** each effect is read from named fields only, under the Story 11.1 marker-text test, and states its source and backend

**Given** a signal the harness does not expose
**When** activity is shown
**Then** it is listed as not observed

### Story 13.2: Reporting levels

As a team,
I want to choose a reporting level, each with its own published field list,
So that we report as much as we need and no more (FR-54).

**Acceptance Criteria:**

**Given** the reporting levels, from metrics only to full OTEL
**When** they are published
**Then** each has its own field list in the report schema, and the collector rejects fields above the host's level

**Given** a host reporting at one level
**When** a higher level is requested
**Then** it turns on only after the same disclosure and consent as the first report (Story 12.4)

### Story 13.3: Usher as the local OTLP endpoint

As a person whose harness exports OTEL,
I want usher to receive it locally and forward only what my reporting level allows,
So that harness telemetry reaches my team's collector only through the same field list (FR-54).

**Acceptance Criteria:**

**Given** a harness that exports OTEL
**When** I point it at usher
**Then** the setting is written in a layer the harness reads it from, never a project or local file (Story 1.7), through the Story 8.3 apply path

**Given** the endpoint
**When** it listens
**Then** it binds to loopback only and accepts data only with the per-launch token usher hands the harness with the endpoint setting; anything without it is dropped and counted in `usher status`

**Given** OTLP data arriving at usher
**When** it is received
**Then** content fields (for example prompt text, when a harness logs user prompts) are dropped on arrival at every level, never stored, logged or forwarded, and the Story 11.1 marker-text test runs against the endpoint

**Given** the remaining fields
**When** they are forwarded
**Then** only fields the chosen level allows leave the host, and nothing is forwarded while reporting is off
**And** removing usher leaves no OTLP endpoint (Story 6.14)

## Epic 14: Team instance [v2]

The light collector grows into the service a team runs for the shared side: a person's usher enrolls with it, it distributes a signed catalog of profiles, templates and recommendations and an index of options, and it evaluates templates across hosts. It stays self-hosted first (SOUL §6).

### Story 14.1: Enroll with a team instance

As a person,
I want my usher to find and join my team's instance, with my own settings left alone,
So that the team's customization reaches me without taking over my seat (FR-59).

**Acceptance Criteria:**

**Given** a team instance
**When** I enroll
**Then** usher shows what the team will receive before any report (Story 12.4), and the team's customization layers over usher's shipped defaults

**Given** team customization and my personal layer
**When** they disagree
**Then** the personal layer wins; team customization never overrides it

**Given** a person enrolled with several organizations
**When** usher keeps their enrollments
**Then** each enrollment and consent is kept apart

### Story 14.2: The signed catalog

As a team,
I want to distribute profiles, templates and recommendations through a signed, versioned catalog in an open format,
So that every client can trust what it receives and another client could read it too (FR-60).

**Acceptance Criteria:**

**Given** a catalog published by the team instance, signed under the key governance PRD Q8 settles
**When** a client fetches it
**Then** the client verifies the signature before using any item, and an item that fails verification is not used

**Given** the catalog format
**When** it is published
**Then** it is documented so another client could read it

### Story 14.3: The options index

As a team,
I want to serve an index of options that extends what usher can check, install or recommend,
So that we can extend usher without waiting for a client release (FR-61).

**Acceptance Criteria:**

**Given** an index item
**When** a client receives it
**Then** it is signed, versioned, and used only after a person confirms it at the point of use

**Given** an item the client cannot verify
**When** the index is loaded
**Then** it is ignored and reported

### Story 14.4: Template checks

As a team,
I want to declare templates that check a setup and produce findings and recommendations,
So that we can say what we recommend for a use case without enforcing it (FR-56).

**Acceptance Criteria:**

**Given** a template from the catalog
**When** a host evaluates it
**Then** it produces findings and recommendations per environment, and never blocks the person or changes a setting by itself

**Given** reports from several hosts
**When** the team instance evaluates templates
**Then** it does so across hosts, comparing environments and ranking no one

### Story 14.5: Remote apply as a proposal my other host receives

As a person with more than one host,
I want to send a change I confirmed on one host to another host of mine, and confirm it there,
So that I do not repeat the same setup on each machine, without any host changing without its person's confirmation (FR-49, PRD §5, §7.2).

**Acceptance Criteria:**

**Given** a proposal on host A (Story 8.2)
**When** I send it to my host B
**Then** it travels as a signed proposal, signed with host A's key (Story 12.3), through the team instance or a file, and host B treats "proposals from my own enrolled hosts" as a profile source

**Given** the proposal arrives on host B
**When** I next use usher there
**Then** it is shown like any catch-up change (Story 8.6), recomputed against host B's files, and applied only when I confirm on host B, through host B's journal and safe-execution checks; the journal records the originating host

**Given** a proposal whose signature does not verify against one of my enrolled hosts
**When** host B receives it
**Then** it is rejected and reported
**And** nothing applies unattended; applying to a fleet without each person's confirmation is out of scope

## Epic 15: Full in-harness surface and the app [v2]

From inside the harness an agent drives installs, clones and setup conversationally, and an app gives the same views and actions to people who prefer it. Both confirm as every surface does.

### Story 15.1: An agent drives setup from inside the harness

As a newcomer talking to my harness,
I want the agent to install tools, clone repos and run setup steps for me,
So that onboarding is a conversation (FR-62).

**Acceptance Criteria:**

**Given** the MCP surface
**When** install, update, clone and setup are added
**Then** they are operation kinds of one `propose` tool and one `apply` tool, not a tool per verb, and all MCP tool descriptions together stay within 1,500 tokens

**Given** an agent that applies one
**When** it runs
**Then** it goes through the same stories as the CLI (6.8, 6.9, 9.1, 9.4), confirmed as in Story 10.3, with the surface and client name journaled

**Given** a run that takes longer than an MCP call should block
**When** `apply` starts it
**Then** it returns a run id at once, and the agent asks for progress by that id, paged within the 4,000-token response budget

### Story 15.2: The app

As a person who prefers an app to a terminal,
I want an app that shows my maps, comparisons and findings and lets me apply changes,
So that usher is usable without the command line (FR-62).

**Acceptance Criteria:**

**Given** `usher-app`, a Tauri 2 menu-bar app shipped as its own optional bundle (AD-21), with Tauri's current version confirmed when this story starts
**When** it shows anything
**Then** it reads the same store through `usher-core` and watches no files itself; the background service keeps the store current
**And** it ships at the same version as `usher`, never migrates the store, refuses a store whose schema version differs from its own, and updates only through Story 6.11, never through Tauri's updater

**Given** a change I make in the app
**When** the webview requests it
**Then** the webview's Tauri capability file grants no fs, shell, http or opener permission, only the commands `propose`, `show_diff` and `confirm(proposal_id, diff_hash)`; `usher-core` computes the diff and applies through Story 8.3 only when my confirmation carries that diff's hash, with the journal recording `app` as the surface

**Given** the UI bundle
**When** the app is built
**Then** it is pinned and built from the repo, a content security policy forbids remote origins, a structural test checks the capability file and the policy, and it runs on macOS first; Windows follows PRD Q14
**And** `usher remove` (Story 6.14) also removes the app bundle and its login item

## Epic 16: Reviewer seats and brokered keys [v2]

A person sits down at an agent-grown run variant with a working seat to review it, and launches a harness with a short-lived key their own vault or cloud login issues for that launch.

### Story 16.1: Seat a reviewer at a run variant

As a reviewer,
I want a working seat at a repo a factory or legion grew,
So that I can review the variant instead of setting it up (FR-65).

**Acceptance Criteria:**

**Given** a run variant
**When** I ask usher to seat me
**Then** it sets up the repo on my host, the declared tools and a named profile, each step on confirmation (Stories 9.1, 9.4)

**Given** the variant's own tool declaration (for example its `mise.toml`)
**When** usher reads it
**Then** it is treated as untrusted: shown, and run only on my confirmation

**Given** the variant's harness settings, written by an agent
**When** usher seats me
**Then** before the first launch it shows them, hooks first (for example a `SessionStart` hook that runs code at launch), and where the harness allows, launches with the variant's project layer off by default
**And** usher never answers the harness's trust prompt; I trust the folder myself, knowing what is in it
**And** usher sets up the seat only; reviewing and judging stay mine, or critic's

### Story 16.2: Launch with a key my vault or cloud login issues

As a person,
I want to launch a harness with a short-lived key my vault or cloud login issues for that launch,
So that no long-lived key sits on my disk or in usher (FR-66, ADR-009).

**Acceptance Criteria:**

**Given** `usher-broker` and its routes (AD-22)
**When** I launch a harness through usher
**Then** an issuing route (AWS STS or GCP for a cloud route; Vault's OpenAI plugin, labelled experimental) gets a short-lived, scoped credential for that launch, and usher hands it to the harness in memory
**And** usher never reads a stored long-lived key from 1Password, the keychain or any other store; a provider with no issuing route (Anthropic API keys today) gets no brokered launch
**And** an issuing route gets its credential through the owning tool's own login and CLI (`aws configure export-credentials`, `gcloud`, the `vault` token helper); usher never reads, caches or stores the base credential
**And** usher never creates or modifies an item in my store
**And** only a launch I start from the CLI or the app calls the broker; MCP and the background service never do

**Given** any route
**When** `usher status` shows it
**Then** it says, per harness, whether a brokered route exists

**Given** Anthropic workload identity federation, unverified
**When** this story lands
**Then** it is a harness-knowledge item marked unverified, with a probe; the Anthropic route moves to issuing only after the probe passes

**Given** the key
**When** the launch runs
**Then** its value is never written to disk, logged, journaled or reported, which a test checks with a marker key
**And** the docs state that the key reaches the harness through its process environment, which the same OS user can read; usher promises no more than that

### Story 16.3: Move plaintext keys behind the broker

As a person,
I want a plaintext key the credential scan finds to come with a recommendation to move it behind the broker,
So that I know a safer path exists (FR-66).

**Acceptance Criteria:**

**Given** a plaintext key found by the Story 11.4 scan
**When** findings are shown
**Then** it names the key's location and type, never its value, and where an issuing route exists for that provider, recommends launching through the broker instead; where none exists (Anthropic today), it says so

## Epic 17: Contribution loop [v2+]

A person's better setting becomes a proposal the team can adopt, with credit. Declaring costs the person almost nothing; judging and deciding stay human acts (SOUL §8).

### Story 17.1: Propose a difference

As a person,
I want to turn a difference in my environment into a proposal for a team profile,
So that a better setting I found can help the team (FR-57).

**Acceptance Criteria:**

**Given** a difference between my environment and a team profile
**When** I propose it
**Then** usher drafts the body from what it already knows (scope, versions, observed effects), and I add only the intent
**And** effects are drafted as facts about environments, with no person attached, and I see the whole body and can cut anything before it is filed

**Given** a proposal
**When** it is filed
**Then** it is routed to a named reviewer with a review clock

### Story 17.2: Decide, ripple and credit

As a reviewer,
I want to accept, hold as an option, or decline a proposal with a reason,
So that the team's profiles improve through a decision someone owns (FR-58).

**Acceptance Criteria:**

**Given** a proposal
**When** I accept it
**Then** it becomes a new profile version, and reaches others through catch-up (Story 8.6), offered and never forced
**And** credit is the git author of the commit the proposer makes themselves; usher writes no person's name into a profile

**Given** a proposal past its review clock
**When** the clock passes
**Then** it is surfaced to the team; nothing is decided by the clock
