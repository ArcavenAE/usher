# Adversarial review: usher architecture spine (2026-11-08)

Reviewed: `ARCHITECTURE-SPINE.md` (draft, 2026-11-08), against the PRD sections it binds
(FR-13, FR-22, FR-23, FR-39 to FR-42, FR-49 to FR-52, FR-62 to FR-64, FR-67, section 5) and
the `.memlog.md` decisions.

Method: for each finding, two units one level below the spine that each obey every AD as
written and still build incompatibly. Each finding gives the pair, the divergence, proposed
AD text, and a tier.

Tiers: critical (defeats a stated guardrail or loses a person's data or credentials),
high (two compliant builds will not interoperate, or a PRD consequence becomes untestable),
medium (a likely rework or an ambiguity builders will resolve differently), low (cleanup).

## Summary

| # | Pair | Tier |
| --- | --- | --- |
| 1 | Crash recovery in `usher-daemon` or `usher-mcp` vs AD-4 and FR-64 | critical |
| 2 | `usher-mcp` vs `usher-cli apply`: the agent runs the CLI through its own shell | critical |
| 3 | Rollback (AD-7 backups) vs AD-8 on mixed files | critical |
| 4 | `usher-mcp` proposal record vs `usher-cli apply` diff display | critical |
| 5 | `usher-daemon` watcher vs `usher-cli` apply: usher sees its own writes | high |
| 6 | Map rows: daemon (files only) vs CLI or MCP (native reads) | high |
| 7 | `usher-mcp` vs AD-9: an agent request triggers a hazard read | high |
| 8 | `usher-adapter-codex` (native writer) vs `usher-adapter-claude-code` (span editor) | high |
| 9 | `usher-adapter-claude-code` vs `usher-adapter-claude-desktop` (and codex vs goose): two adapters, one file | high |
| 10 | Environment key: discovery (AD-10) vs daemon (`notify` paths) | high |
| 11 | Adapter credential denylist vs a stale knowledge file | high |
| 12 | Secrets in ordinary values: map vs MCP responses, logs and reports | high |
| 13 | Reporting sender: daemon (no timers) vs CLI; client vs collector schema version | high |
| 14 | Delegated runs (bootstrap, install, clone) vs the five-step transaction and the lock | high |
| 15 | Knowledge in a long-lived daemon vs a fresh CLI; signed-file replay | high |
| 16 | `usher-store` durability settings vs AD-3 crash marker | medium |
| 17 | Lock acquisition point vs the confirmation prompt | medium |
| 18 | Profile merge: shared `mcp_servers` vs a fragment; two profiles on one key; non-git versions | medium |
| 19 | State directory resolution: `usher mcp` under a harness's HOME or sandbox vs CLI | medium |
| 20 | Backup eviction (AD-7, 100 MB budget) vs FR-63 keep-until-removed | medium |
| 21 | Journal order: ULID time vs insert order | low |
| 22 | Per-collector consent, queue and host credential storage | low |

## Findings

### 1. Crash recovery runs in whichever mode takes the lock next (critical)

**Units:** `usher-daemon` (or `usher-mcp`) and `usher-cli`.

**Divergence while compliant:** AD-3 says "the next mode to take the lock finishes or rolls
back that change before doing anything else". AD-2 lets any mode take the lock. If the daemon
needs the lock to record anything in the journal, or `usher mcp` needs it to journal a
discovery consent (AD-10 says consent is journaled), that mode is the one that must finish or
roll back a half-applied change. Finishing means writing harness configuration from the
daemon (contradicts FR-13 "the watcher ... changes nothing") or from MCP (contradicts AD-4,
the agent applies). Rolling back is also a harness write nobody confirmed. Separately, AD-3
lets the mode choose between finish and roll back, while FR-64 says the journal shows which
steps finished "so the person can resume (FR-46) or roll back (FR-63)": the PRD gives that
choice to the person. One builder writes automatic roll-forward; another writes automatic
rollback; both comply.

**Proposed AD-3 replacement for its last sentence:**

> An intent with no outcome is a crash marker. Only the CLI on a terminal resolves one, and
> only after showing the person the marker, the state of each target (pre-hash, post-hash or
> neither) and the two options; the person chooses resume or roll back, and the choice is
> journaled. Any other mode that finds a crash marker under the lock records nothing that
> changes a harness file, refuses every change, keeps serving reads, and reports the marker
> with the command that resolves it. A target whose current hash is neither the recorded
> pre-hash nor the recorded post-hash is a conflict and is left alone.

### 2. The agent can drive the CLI, so "CLI on a terminal" is not a surface the agent cannot answer (critical)

**Units:** `usher-mcp` and `usher-cli apply`.

**Divergence while compliant:** AD-4 forbids MCP from applying and sends application to "the
CLI on a terminal". Every harness usher targets gives the agent a shell tool. The agent
creates a proposal over MCP, then runs `usher apply <id>` in its own shell and answers `y` on
stdin. Both units obey AD-4 to the letter; section 5 ("a prompt the agent cannot answer for
them") is defeated. A builder who adds an `isatty` check is still compliant and still beatable
with `script(1)` or a pty.

**Proposed AD-4 addition:**

> Apply, rollback, crash resolution and any change that writes a harness file confirm through
> a prompt the agent cannot answer: on macOS an OS authentication prompt (LocalAuthentication,
> Touch ID or password) whose text names the proposal id and the target files. A stdin or tty
> answer is never sufficient on its own. `--yes` and any non-interactive confirm flag do not
> exist for these verbs. The confirmation method is recorded in the journal entry (FR-63
> "the person's confirmation").

If the OS prompt is rejected as too heavy, the spine must instead name the threat it accepts
("an agent with a shell can apply") as a [DECISION] with the operator's ruling, since the PRD
states the stronger rule.

### 3. Rolling back a mixed file restores old credentials and trust records (critical)

**Units:** `usher-store` rollback (AD-7 backups, FR-63 "a file write restores its backup") and
the AD-8 rule for mixed files.

**Divergence while compliant:** a span edit to `~/.claude.json` changes one key. AD-7 backs up
the whole file. Later, rollback restores that backup as bytes. Meanwhile Claude Code has
refreshed `oauthAccount` and the person has trusted a new folder. The restore writes the old
token and the old trust state: usher has written a credential and a trust record, which AD-8
forbids, while every step obeyed AD-3 and AD-7. The current-hash check in FR-63 catches this
only as "conflict", and on a file the harness rewrites on every session, every rollback of a
mixed file becomes a conflict, so rollback never works there.

**Proposed AD-7 addition:**

> A rollback is a change and runs as an AD-3 transaction through the same write path as the
> change it undoes (AD-5). For a file harness knowledge marks as mixed (credential or trust
> keys beside writable keys), rollback is a key-level inverse: it restores the recorded
> before-value of each key the original change wrote and leaves every other key as it is now.
> Whole-file restore from backup is allowed only for files with no credential or trust keys.
> A key whose current value differs from the value the original change wrote is a conflict.

### 4. The diff the person sees may not be what gets written (critical)

**Units:** `usher-mcp` (creates a proposal) and `usher-cli apply` (shows "the diff" and writes).

**Divergence while compliant:** AD-4 says the person "sees the diff" before applying by id. It
does not say who computes that diff or when. Builder A stores the diff text the MCP call
produced and the CLI prints it; Builder B recomputes from structured operations at apply time.
In A, the agent authored or influenced the text the person approves (including any prose
explanation), and the files may have changed since. Both comply.

**Proposed AD-4 addition:**

> A proposal stores structured operations only (harness, environment key, file, key path,
> value, layer) and the base hash of each target file at creation. It never stores display
> text from the creating surface. At apply, the CLI recomputes the dry run and diff from those
> operations against the current files, refuses entries marked personal (FR-49) and managed
> layers, and stops if any base hash changed (FR-64). Any explanation an agent attached is
> shown labeled as agent-written, after the recomputed diff.

### 5. The daemon reports usher's own writes as outside changes (high)

**Units:** `usher-daemon` and `usher-cli apply`.

**Divergence while compliant:** AD-3 step 3 writes by temp file and rename. FSEvents delivers
create and rename events for it. The daemon (AD-9) rebuilds values and, per FR-13, issues "a
setting in effect changed" notice for a change the person just made. Nothing in AD-2 says the
daemon must look at the lock or the journal; the lock governs writes to harness config and the
journal, and the daemon writes neither. One builder suppresses by timing, another by journal
lookup, another not at all.

**Proposed AD-9 addition:**

> The daemon attributes every observed change before issuing a notice: a change whose
> resulting file hash matches the post-hash of a journal entry (complete, or in progress
> under a held lock) is recorded as usher's own and never raises a notice. While the journal
> lock is held, the daemon queues events for the locked targets and processes them after
> release.

### 6. Three writers of the map with different read powers (high)

**Units:** `usher-daemon` (files only, AD-9) and `usher-cli map` or `usher-mcp map` (may run
native reads such as codex app-server `config/read` when a person asks).

**Divergence while compliant:** AD-7 puts "map" in the store but names no owner. For codex, the
CLI's native read reports project, user and system layers with app-server's own precedence;
the daemon rebuilds from files using precedence from harness knowledge. Each overwrites the
other's rows, values flap, and FR-13 notices fire on the flapping. Both comply.

**Proposed new AD (map ownership):**

> The map is derived, never edited. Each observation row carries environment key, key path,
> value or value hash, layer, source (`file` or `native`), knowledge hash and observed time,
> and is inserted, not updated. The value in effect is computed in `usher-core` from the
> newest observation per (environment, key, layer), preferring `native` over `file` when both
> exist for the same file state. Notices compare effective values, never raw rows from
> different sources.

### 7. An agent request can trigger a state-changing read (high)

**Units:** `usher-mcp` and the codex adapter hazard declaration (AD-6, AD-9).

**Divergence while compliant:** AD-9 says a hazard read "runs only when a person asks, never
from the daemon". AD-4 lets MCP map and compare. An agent asks MCP to map a codex environment;
one builder treats the agent's tool call as "a person asks" and runs app-server, which writes
`installation_id`, logs and sqlite files into the codex home (memlog Q4 probe). That is a
change to harness state with no journal entry and no confirmation. Another builder refuses.
Both read AD-9 as satisfied.

**Proposed AD-9 replacement for its fourth sentence:**

> An adapter read declared as a hazard is a change: it runs only from the CLI with the
> person's confirmation, as an AD-3 transaction whose declared targets include the state the
> read writes, or against a scratch harness home usher creates and removes. MCP mode and the
> daemon never run one; MCP answers from the map and says which values need a hazard read.

### 8. Native-writer and span-editor adapters journal, back up and roll back differently (high)

**Units:** `usher-adapter-codex` (app-server `config/value/write`) and
`usher-adapter-claude-code` (span editor).

**Divergence while compliant:** AD-3 step 2 says back up "every file the change touches". With
a native writer, usher does not choose the files; the app-server process also writes state
files (finding 7) and may rewrite the whole `config.toml`. FR-64 says a write that touches more
targets than the dry run listed stops, and that writes are temp file plus rename. The codex
build cannot guarantee either and still claims AD-5 compliance; the claude-code build can. For
rollback, one codex build restores the backup bytes (bypassing the native writer, against
AD-5), another calls the native writer with the old value. The journal rows from the two
adapters will not share a shape.

**Proposed AD-5 addition:**

> Every adapter write, native or file edit, first returns a write plan: the exact target files,
> each with its pre-hash and the keys it will change. Core backs up exactly those files and
> records the plan. After the write, core hashes every file in the plan and every file in the
> adapter's declared side-effect set; any change outside the plan's keys or files is recorded
> and reported as a conflict. A native writer whose side effects cannot be declared is not
> used for writes. Rollback calls the same adapter write path with the before-values (see
> finding 3).

### 9. Two adapters claim one file (high)

**Units:** `usher-adapter-claude-code` and `usher-adapter-claude-desktop`, and likewise
`usher-adapter-codex` and `usher-adapter-goose` for `AGENTS.md`.

**Divergence while compliant:** AD-6 is one adapter per harness, not one owner per file.
`~/.claude/settings.json` is read by Claude Code and by the desktop app's embedded Claude Code;
`AGENTS.md` is read by codex and goose. A profile with a fragment for each harness causes two
adapters to plan writes to one file in one transaction or two, each with its own backup and
its own read-back. Map rows for the same bytes appear under two harness ids. Both comply.

**Proposed AD-6 addition:**

> Harness knowledge assigns every known file path exactly one owning adapter. Only the owner
> plans writes to that file. Another harness that reads the file declares it as a shared
> layer, and core routes that harness's entries for the file to the owner's write plan, so one
> transaction writes the file once. Adapters still never call each other; core does the
> routing.

### 10. One environment, several keys (high)

**Units:** discovery from harness records (AD-10) and `usher-daemon` paths from `notify`.

**Divergence while compliant:** the spine keys environments by "harness id plus absolute
directory". `~/.claude.json` `projects` keys are stored as Claude wrote them; FSEvents reports
real paths (`/private/var/...`, firmlink and case differences on case-insensitive APFS); a git
worktree has its own path but shares config. Discovery and the daemon each produce an
absolute path for the same directory, and they differ. Duplicate environments appear, and
notices land on unmapped keys. User-level layers have no directory at all, so builders invent
different placeholder keys.

**Proposed addition to Consistency Conventions, Identifiers:**

> An environment key is (harness id, canonical directory), where the canonical directory is
> produced by one `usher-core` function: resolve symlinks and firmlinks with `realpath`, keep
> case as the file system returns it, normalize to Unicode NFC, no trailing slash. Paths read
> from harness records are canonicalized before use, and the original spelling is stored as an
> alias. A harness's user-level scope uses the reserved directory value `@user`; a managed
> scope uses `@managed`.

### 11. A credential denylist leaks keys a newer harness adds (high)

**Units:** `usher-adapter-claude-code` reading `~/.claude.json` and `usher-knowledge` when the
knowledge file lags a harness release.

**Divergence while compliant:** AD-8 names two keys and "every key harness knowledge marks as
credential". A harness release adds a new token key; the knowledge file has not caught up
(AD-12 keeps the last verified file). The adapter reads the value into the map, log and maybe
a report. It obeyed AD-8. Also, "skip ... without reading their values" is not something a JSON
or TOML parser can do: it lexes every byte. Builders will differ on whether a parsed but
discarded value counts as read.

**Proposed AD-8 replacement for its first sentence:**

> In a mixed file, adapters decode only the keys harness knowledge lists as readable for that
> file and version; every other key is recorded by name and type only, never by value. "Never
> read" means the value is never retained in memory past parsing, never stored, logged,
> journaled, reported or returned by any surface. Whole-file backups of mixed files are opaque
> bytes in the owner-only store and are never parsed except by the rollback path in finding 3.

### 12. Secrets sit in ordinary setting values and reach the agent (high)

**Units:** the map (core, adapters) and `usher-mcp` responses, plus logging and the report
schema.

**Divergence while compliant:** MCP server definitions carry tokens in `env`, `headers` and
`args` (`mcpServers.<name>.env.GITHUB_TOKEN`). These are not credential keys in AD-8's sense,
so the map holds them, MCP returns them to the agent (and so to the model provider), and a
report could carry them if the schema has a "settings in effect" field. Every unit complies.

**Proposed new AD (value redaction):**

> Harness knowledge marks secret-bearing paths by pattern (MCP server `env`, `headers`, URL
> userinfo and query, any key matching the credential name list). Values at those paths are
> stored as a hash and a presence flag only. No surface returns, logs or reports them; a diff
> shows `<set>` or `<changed>`. A profile fragment that supplies such a value is an error;
> profiles reference secrets by name for the person to supply at apply.

### 13. Nobody can send reports on a schedule, and versions do not meet (high)

**Units:** `usher-daemon` (AD-9: no timers) and `usher-cli`; client and `usher-collector`
across releases.

**Divergence while compliant:** the spine never says which mode sends reports. The daemon may
not run timers, so a daemon sender reports only when a file changes; a CLI sender reports only
when the person runs a command. Retrying after a collector outage needs a timer. Two builders
pick different senders and both comply. Separately, AD-13 rejects any field outside "that
schema version" but says nothing about which versions a collector accepts. A newer client
sending v3 to a v2 collector is rejected whole; a builder who "helpfully" drops fields to
downgrade has changed what leaves the host without consent.

**Proposed AD-13 and AD-9 additions:**

> AD-9: The daemon may hold one timer, for report sending, and only while reporting is on;
> its interval comes from the consented collector settings and it is counted in the idle-CPU
> budget. With reporting off, the daemon has no timers.
>
> AD-13: Each schema version is frozen once released. The collector advertises the versions
> it accepts; the client sends a report in the highest version both support and never in a
> version the person did not consent to. If none is shared, the client holds the report and
> tells the person. Unsent reports are queued in the store with their consent reference and
> are dropped at withdrawal (FR-39).

### 14. Delegated runs do not fit the five steps, and the lock scope is unclear (high)

**Units:** bootstrap and install (`usher-cli`, FR-11, FR-44, FR-46, delegated to brew, mise,
git) and apply.

**Divergence while compliant:** AD-2 and AD-3 speak of "harness configuration" and backing up
"every file the change touches". FR-63 and FR-64 cover delegated runs too, with different
records (installer known paths before and after). One builder runs `brew install` outside the
lock since it is not harness configuration; another holds the lock for ten minutes. If the
daemon or MCP journal anything (finding 1), the long hold blocks them. The spine also uses
"journal" for both changes and consent records (AD-10), so builders disagree on whether a
consent record is a five-step transaction.

**Proposed AD-3 addition:**

> The journal holds two entry kinds. A change (file write, hazard read, delegated run,
> rollback, removal step, login item install) runs under the lock as a transaction: intent;
> snapshot (file backups for writes; installer known paths and versions for delegated runs);
> act; observe; outcome. A record (consent, named roots, notice, adoption, observation) is a
> single insert that needs no lock and never touches a harness file. The daemon and MCP write
> records only.

### 15. A long-lived daemon runs on stale harness knowledge; old signed files replay (high)

**Units:** `usher-knowledge` loaded once in `usher-daemon` and loaded fresh in `usher-cli`.

**Divergence while compliant:** AD-12 says an unverified file is ignored and the last verified
one stays. It does not say the daemon reloads after an update, so the daemon watches old paths
while the CLI writes by new writable-key lists. It also has no ordering: an attacker who can
place files in the state directory, or a mirror, can serve an older validly signed file that
still marks a since-corrected dangerous key writable, and AD-12 accepts it.

**Proposed AD-12 addition:**

> Each knowledge file carries a monotonic sequence number and a minimum usher version inside
> the signed payload. A file is accepted only if its sequence is greater than the active one.
> The active file's hash is stored, every mode reads that one, the daemon reloads when it
> changes, and every journal entry and observation records the knowledge hash it used. The
> binary embeds a baseline knowledge file used when none has been accepted.

### 16. Store durability can lose the intent row after the harness write lands (medium)

**Units:** `usher-store` SQLite settings and AD-3 step 1.

**Divergence while compliant:** FR-63 requires the intent to be "written and flushed to disk
before the change starts". A builder choosing WAL with `synchronous=NORMAL` (a common default
for speed) can lose the last commit on power loss: the harness file is changed and no crash
marker exists. Another chooses `FULL`. Both comply with AD-7.

**Proposed AD-7 addition:**

> The store runs with `synchronous=FULL`. Each AD-3 step commits its row before the next step
> begins. A backup file is fsynced, and its directory fsynced, before the intent's backup
> reference is committed.

### 17. When is the lock taken relative to the confirmation prompt (medium)

**Units:** `usher-cli apply` and every other lock taker.

**Divergence while compliant:** AD-2 says the lock is held "for the whole change transaction".
Builder A takes it before showing the diff and holds it while the person reads (blocking other
modes for minutes or forever if the terminal is left open). Builder B takes it after
confirmation. FR-64 "a second run waits for the person" fits either.

**Proposed AD-2 addition:**

> The lock is taken after the person confirms and is never held across a prompt. Under the
> lock, the CLI re-checks every base hash from the confirmed dry run; any change aborts the
> transaction before step 1 and re-shows the diff. A mode that cannot take the lock waits
> with a message naming the holder's process id and mode, which the lock file records.

### 18. Profile merge has no conflict rule (medium)

**Units:** two adapters merging one profile (AD-11), and two profiles applied to one key.

**Divergence while compliant:** a profile defines MCP server `foo` in shared `mcp_servers` and
also in the claude-code fragment with different arguments. The codex adapter lets the shared
form win, the claude-code adapter lets the fragment win; both "merge by that harness's
adapter". Two followed profiles both set one key, and FR-23 adoption records point at both.
Also, AD-11 says "a profile's version is its commit", which leaves file and directory sources
(also allowed) with no version for FR-51.

**Proposed AD-11 addition:**

> The same MCP server name in the shared table and in a fragment is a load error naming both.
> When two loaded profiles set one key for one environment, comparison shows both and the
> person picks one at apply; the adoption record names the chosen profile and version. A
> profile version is its git commit for a git source and the content hash of the profile
> directory otherwise.

### 19. `usher mcp` may open a different store (medium)

**Units:** `usher-mcp` launched by a harness and `usher-cli` in the person's shell.

**Divergence while compliant:** harnesses launch MCP servers with their own environment;
codex runs tools in a sandbox that can deny writes outside the workspace, and test or managed
setups change `HOME`. A builder resolving the state directory from `$HOME` opens a second
store or fails writes; AD-1 "one store" is broken without any AD being violated, since none
says how the directory is found.

**Proposed AD-7 addition:**

> The state directory is resolved from the OS user record (`getpwuid` home on macOS), not
> `$HOME`, unless `USHER_STATE_DIR` is set, and the resolved path is printed by `usher doctor`
> and recorded in each journal entry. A mode that cannot open the store read-write runs
> read-only and says so in every response.

### 20. Eviction cannot both meet 100 MB and keep every rollback (medium)

**Units:** `usher-store` eviction (AD-7, Budgets) and FR-63 rollback.

**Divergence while compliant:** AD-7 forbids evicting a backup "a recorded change could still
roll back to". FR-63 lets any entry be rolled back at any time and keeps backups "until the
person removes them". So nothing is ever evictable, and the 100 MB budget is unenforceable.
One builder silently exceeds the budget, another reads "could still roll back" narrowly and
evicts old backups, losing rollback.

**Proposed AD-7 replacement for its eviction sentence:**

> Usher never evicts a backup on its own. When state passes the disk budget, usher tells the
> person, names the oldest entries and their size, and removes backups only for the entries
> the person picks; those entries are then recorded as no longer reversible.

### 21. Journal order (low)

**Units:** the rollback planner (FR-63 "reverse order", dependency check) and the JSON lines
export.

**Divergence while compliant:** ULIDs embed wall-clock time, which can step backwards. One
builder orders by ULID, another by SQLite rowid; they disagree after a clock change.

**Proposed Identifiers addition:**

> Journal order is a sequence number assigned under the lock. ULIDs identify entries and never
> define order.

### 22. Consent, queue and host credential per collector (low)

**Units:** `usher-cli` enrollment (FR-39, FR-40) and whichever mode sends reports.

**Divergence while compliant:** AD-13 says one collector serves one affiliation, so a person
in two affiliations follows two collectors. The spine does not say where the host credential
lives or how consent is keyed, so enrollment may store it in the keychain while the sender
looks in SQLite.

**Proposed AD-14 addition:**

> Consent, queued reports and the host credential are keyed by collector URL plus the pinned
> certificate public key. The host credential is stored in the OS keychain, never in SQLite,
> logs or the journal, and is read only by the sending path.

## Gaps outside the pair format

- AD-1 says removal works with no daemon, but no AD covers the login item or launchd plist as
  a journaled change; FR-67 requires that nothing remains after removal. Finding 14's change
  kinds would cover it if "login item install" stays in that list.
- AD-10 discovery reads `~/.claude.json` `projects` keys; with finding 11 in place this becomes
  an allowlisted read, which matches the memlog's own note ("projects keys only").
