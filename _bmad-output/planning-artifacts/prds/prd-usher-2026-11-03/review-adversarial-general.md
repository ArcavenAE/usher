---
title: "Adversarial review: usher PRD 2026-11-03"
reviewed: prd.md and addendum.md (both updated 2026-11-05)
date: 2026-11-05
---

# Adversarial review: usher PRD

Scope: contradictions, untestable requirements, gaps that make architecture
or epics unsafe, §4 and §7 split inconsistencies, and guardrails a failure
case could slip past. Weighted toward §4.16 (FR-63, FR-64), its §5
guardrail, addendum §7, and coverage of every MVP change kind (installs,
updates, clones, generated files), plus FR-65, FR-66 and the §7 split.
Findings are ordered by severity. Out-of-order FR IDs are intentional and
not raised.

---

## F1. FR-64 cannot hold for changes usher delegates to installers

**Location:** FR-64 (all bullets), FR-11, FR-44, FR-47, §4.2 description
("drives existing installers"), §5 "Every change is journaled and
fail-safe", addendum §7.

**Problem:** FR-64 promises "a change touching only what the dry run
listed", with per-kind path lists, atomic temp-and-rename writes, backups
before every write, and "Any script usher runs stops on an unset variable or
a failed command." Every MVP change except a generated file is carried out
by someone else's program: brew, mise, sideshow, `tools/install.sh`, vendor
`curl | sh` installers, git. Usher does not choose which files those touch,
cannot make their writes atomic, cannot back up files it does not know will
be written, and cannot impose `set -u` on a third-party script. Most of them
offer no dry run that lists targets, so "more targets than the dry run
listed" is undetectable until after the fact. The only addendum §7 row for
an installer tests argument quoting. The PRD reads as if FR-64 covers the
MVP, while in practice it covers only the one MVP change kind usher writes
itself.

**Suggested fix:** Split FR-64 into two tiers. Tier A, usher's own writes:
the current bullets. Tier B, delegated changes: a named contract per
installer covering what usher can guarantee (arguments as a list, a resolved
and checked working directory and environment, a before and after snapshot of
the declared install roots, an exit-status check, a post-run diff reported as
"also touched"), optionally run under a write-restricting sandbox profile
where the installer tolerates one. State plainly that atomicity and the
target list do not apply to tier B. Add addendum §7 rows for tier B.

**Consequence if shipped:** The headline guarantee is false for installs,
updates and clones, the changes the MVP actually makes. An epic team either
builds an unbuildable interceptor or quietly ships tier B without the checks
while the §5 gate claims they exist.

---

## F2. §7.1 "No configuration writes" is contradicted by in-scope FRs

**Location:** §7.1 first limit; FR-11 second consequence; FR-45 ("reads or
generates"); FR-46; FR-47; addendum §4 first bullet.

**Problem:** §7.1 says the MVP makes no configuration writes and "an install
(FR-11) configures nothing." FR-11 in the same scope says "Configuring a
harness during an install follows the confirmation rule (§5)", which only
makes sense if installs configure. FR-47 (in scope) attaches MCP servers,
which on Claude Code and codex means writing harness configuration
(`~/.claude.json`, `config.toml`), and installs packs through sideshow, which
writes into harness command and skill directories. FR-45 generates a mise
file. Vendor installers write PATH lines into shell profiles and create
harness config on first run. Each of these is a configuration write by any
plain reading, so the v1 boundary is not where §7.1 draws it.

**Suggested fix:** Define "configuration write" in §3 (writes to a harness
settings layer named in harness knowledge) and state which in-scope changes
fall outside it and why. Either move MCP registration and pack install out
of FR-47 for the MVP, or add them to §7.1 as named exceptions with their
FR-64 path lists. Delete or rewrite the FR-11 sentence so it agrees with
§7.1.

**Consequence if shipped:** Epic writers pick one reading and the other is
violated. Either FR-47 ships with MCP writes that bypass the v2 FR-49
conflict and personal-layer rules, or FR-47 ships unable to attach anything.

---

## F3. Backups hold the credentials and content the PRD says usher never keeps

**Location:** FR-63 first and second consequences; FR-64 "Nothing is lost";
§5 "Derived state, not content" and "Credentials: presence, never value";
FR-66 ("never journaled"); addendum §7 last row.

**Problem:** FR-63 requires "a content hash and a backup copy for files"
before each change, and FR-64 moves every removal into the backup store.
Files usher may change or remove carry credentials: `~/.claude.json`, codex
`auth.json`, `~/.config/gh/hosts.yml` after a forge login in FR-48, shell
profiles with exported API keys, a generated `.env`. The addendum's only
credential row says "The journal records presence only", which is true of
the journal text and false of the backup beside it. The same backups also
hold file content, against §5 "Derived state, not content". No rule says
where the store lives, its permissions, whether it is encrypted, or that it
is excluded from Time Machine, iCloud, dotfile sync and git.

**Suggested fix:** Add a backup-store requirement: location outside any repo
and sync root, owner-only permissions, excluded from backup tools, never in a
report. Add an addendum §7 row: "A target containing a credential is backed
up" with the expected result stated (encrypted at rest under a key in the
operator's keychain, or the change refused and the manual step named). Amend
§5 to name the backup store as the one sanctioned place content is kept.

**Consequence if shipped:** Usher becomes a plaintext secret cache that
outlives the original file ("kept until the person removes them"), the
custody problem ADR-009's 2026-10-19 amendment was written to prevent.

---

## F4. The §5 guardrail and FR-63 disagree on changes that cannot be undone

**Location:** §5 "Every change is journaled and fail-safe"; FR-63 fourth
consequence; FR-63 summary ("undo any one of them ... from the record
alone").

**Problem:** §5 says "No change runs without a journal entry that can undo
it." FR-63 allows "an entry whose undo cannot be automatic (for example an
installer with no uninstall)" if it names a manual step first. Those are two
different rules. Read strictly, §5 forbids most vendor installers and every
brew downgrade, since brew cannot reinstall an older formula version from a
version string. Read loosely, "names the manual step" lets any change
through with a free-text note, and the gate the guardrail claims ("may
gate") tests nothing.

**Suggested fix:** Pick one. Recommended: §5 reads "No change runs without a
journal entry that either undoes it automatically or names, before the
change, the manual step and what it cannot restore." Add a testable
consequence: the dry run marks such a change as one-way and needs its own
confirmation.

**Consequence if shipped:** The structural gate is either impossible to pass
for the MVP's main change kind, or passes on prose. Either outcome teaches
the team to wave the gate through.

---

## F5. Journal write ordering and durability are unspecified

**Location:** FR-63 first two consequences; FR-64 "Interruption is
recoverable"; addendum §7 rows "process killed between writing the temporary
file and the rename" and "disk full".

**Problem:** FR-63 lists the fields of an entry, including "the state after"
and "the exit status", which exist only once the change has finished.
Interruption recovery needs an intent record written and flushed before the
change starts, then a completion record after. The addendum row expects "the
journal shows the step unfinished", which requires that write-ahead order,
but no FR states it. Nor does anything say what happens when the journal
itself cannot be written (disk full, permissions), is corrupt, or was edited
by hand: "append-only" is a property of usher's code, not of a file on disk,
and "undo from the record alone" trusts that file without checking it.

**Suggested fix:** Add to FR-63: each change writes and syncs an intent
entry before its first side effect and a completion entry after; a failure
to write the intent entry stops the change; the journal carries per-entry
checksums or a hash chain and usher refuses rollback from an entry that
fails it. Add addendum §7 rows: journal unwritable; journal truncated
mid-entry; journal entry altered.

**Consequence if shipped:** A crash leaves changes with no trace, which
FR-64 says cannot happen, and a rollback can run from a damaged or edited
record against real files.

---

## F6. Rolling back a clone or an install can move away the person's work

**Location:** FR-48; FR-63 third consequence ("checks each target's current
hash first"); FR-64 "a removal moves the target to the backup store";
addendum §7.

**Problem:** The "before" state of a clone is an absent directory, so its
undo is "remove the directory". FR-63's conflict check is defined as a
content hash, which is undefined for a directory tree that, within minutes
of the clone, holds the person's commits, branches and untracked files.
Nothing says rollback compares a clone's HEAD, refs and status to the
journaled state. The same applies to a tool install directory a person has
since configured. Moving a multi-gigabyte repo into the backup store also
hits the free-space check, and across volumes a move becomes copy plus
delete, which FR-64 says never happens. No addendum §7 row covers a clone at
all: not a partial clone, not an existing non-empty target, not a failed
forge login.

**Suggested fix:** Define per-kind conflict checks in FR-63 (file: hash;
package: installed version; clone: HEAD, ref list, and an empty `git status`
and stash list; directory: tree manifest). A clone with any local work is
always a conflict. Add a scope list for clone targets in FR-64 (under the
roots of FR-5, never an existing non-empty directory). Add clone rows to
addendum §7.

**Consequence if shipped:** "Undo the bootstrap run" silently relocates a
newcomer's first week of work into a hidden backup store, the exact
not-fluent-in-git person FR-48 is for.

---

## F7. Update rollback cannot work "from the record alone", and usher's own update is unguarded

**Location:** FR-44 (including usher self-update from a signed feed); FR-63
"the version for a package" and "undo ... from the record alone"; §9 open
question 8 (signing unresolved).

**Problem:** A version string is not enough to roll back. brew does not keep
older bottles by default, vendor native installers replace the binary in
place, and Claude Code's own auto-updater will re-upgrade after a downgrade.
For usher's self-update, the code that reads the journal and performs the
rollback is the new, possibly broken, build, and the journal format may have
changed between versions. FR-44 also ships a signed feed in the MVP while
open question 8 (who signs, rotation, revocation) is still open.

**Suggested fix:** Require the journal to retain the prior artifact (or a
verified fetch reference to it) for any update it claims to undo, or mark
updates as one-way under F4's rule. Add: the previous usher build stays
runnable until the new one passes a self-check; the journal format is
versioned and readable by at least the prior release. Move open question 8
into an MVP prerequisite, or cut usher self-update from §7.1 until it is
answered. Add addendum §7 rows: update with no prior artifact; feed
signature invalid; harness auto-update after a rollback.

**Consequence if shipped:** The first bad usher release can break the tool
that was supposed to undo it, and harness "rollbacks" are reported done and
then reversed by the harness within the hour.

---

## F8. "Undo any one of them" ignores dependencies between changes

**Location:** FR-63 summary and third consequence ("Rolling back runs in
reverse order").

**Problem:** Reverse order is defined for a whole run. For "any one" change,
nothing stops undoing an early entry that later entries rely on: removing
mise after usher generated a mise file and installed tools through it,
removing a clone after tools were installed into it, rolling back a harness
install while a session from FR-10 is still running it. The hash check sees
nothing wrong because the early target itself is unchanged.

**Suggested fix:** Record dependencies between entries (a later entry names
the earlier ones it used). A single-entry rollback lists dependents and
offers to roll them back first or refuses. Before removing a binary, apply
the FR-44 running-session check.

**Consequence if shipped:** A one-entry rollback leaves a seat in a state no
run ever produced, with nothing in the journal explaining it.

---

## F9. Atomic rename breaks symlinked settings files, and the symlink rule refuses them

**Location:** FR-64 "Scope is bounded" (symlink that leaves the path list is
refused) and "Nothing is half written" (temp file and rename); addendum §7
symlink row; §6 "managing dotfiles" non-goal.

**Problem:** Many experts, the UJ-2 customer in particular, keep
`~/.claude/settings.json`, `~/.codex/config.toml` or shell profiles as
symlinks into a dotfile repo (chezmoi, stow, a git checkout). Under FR-64
such a target is either refused, because the link leaves the allowed paths,
or, if allowed, rename-into-place replaces the symlink with a regular file
and detaches it from the dotfile repo without notice. The same applies to
hard links. The PRD does not say which happens, and §6 makes dotfile
managers a non-goal without saying usher must not break them.

**Suggested fix:** Add to FR-64: a symlinked target is resolved, the write
goes to the link's destination by the same temp-and-rename in that
directory, the link is kept, and the dotfile manager owning it is named in
the dry run. Add addendum §7 rows: symlinked target inside allowed scope;
hard-linked target.

**Consequence if shipped:** For the expert cohort usher either refuses most
real writes or silently breaks their dotfile management, which they find
the next time they sync.

---

## F10. The definition of "change" pulls in usher's own state, or leaves gaps

**Location:** §3 "Change"; FR-63; FR-13 ("The watcher writes only usher's
own state"); FR-14 snapshots; FR-39 consent records; FR-40 host credential;
FR-46 resume state.

**Problem:** §3 defines a change as "anything usher does that alters a
host." Usher's snapshots, consent records, journal, lock files, harness
knowledge cache and FR-40 host credential all alter the host. Read
literally, each needs a journal entry with a backup, including the journal's
own writes, which never ends. Read loosely, the list in §3 (install, update,
clone, generated file, settings write, restore, rollback) is the closed set,
and FR-40's on-host credential, a `mise trust` call, a forge login's
credential-helper write and a shell profile PATH edit fall outside it with
no rule.

**Suggested fix:** Redefine: a change is any write outside usher's own state
directory; usher's own state is listed, kept under one root, and covered by
its own integrity rule. State the change kind (and FR-64 path list) for each
side effect an in-scope FR causes, including FR-40 credential storage and
login helpers run by FR-11 and FR-48.

**Consequence if shipped:** Architecture either builds a recursive journal
or treats real host writes as internal and skips the safety checks for them.

---

## F11. Generating a mise file writes into a team's repo and starts code execution

**Location:** FR-45 second consequence; §7.1 ("generated files (FR-11, FR-44
to FR-48)"); FR-64 scope lists; FR-65.

**Problem:** FR-45 says usher "reads or generates" a mise file. Where it is
written is unstated: a `mise.toml` in a repo is a team-owned, committed file,
so generating it changes the team's repo for everyone who pulls, a
team-level configuration write that skips the v2 FR-49 conflict and review
rules. A new mise file needs `mise trust`, and `mise install` runs plugin and
hook code from the file. FR-64 has no path list for generated files. The
read side matters too: FR-65 seats a reviewer at an agent-grown run variant
and installs its declared tools, so an untrusted repo's mise file decides
what gets downloaded and run on the reviewer's host.

**Suggested fix:** In FR-45, name the generation targets (the person's
global mise config, or a repo file only as a proposed diff the person
commits) and the conflict rule for an existing file. Treat `mise trust` as a
change with its own confirmation. In FR-65, require that a variant's
declared tools are shown and confirmed per tool, never trusted from the
variant's own file, and that the variant's trust state is never accepted on
the person's behalf (§6).

**Consequence if shipped:** A generated file lands in a shared repo without
review, and a reviewer seat becomes a path for an agent-grown repo to run
code on a human's machine.

---

## F12. Confirmation is required for settings changes only, and FR-45, FR-47 omit it

**Location:** §5 "Changes on confirmation" ("Usher shows a settings change
before making it"); FR-45; FR-47; FR-62 first consequence; FR-63 "the
person's confirmation".

**Problem:** The §5 guardrail is scoped to settings changes. FR-11, FR-44
and FR-48 each restate confirmation, but FR-45 (generate a mise file) and
FR-47 (attach services, install packs) have no confirmation consequence at
all. Separately, FR-63 records "the person's confirmation" without saying
what counts: in the v2 in-LLM surface, the agent itself can answer the tool
call, which is answering a prompt on someone's behalf (§6). The background
service in the MVP has no confirmation channel defined.

**Suggested fix:** Widen §5 to "Usher shows every change before making it."
Add a confirmation consequence to FR-45 and FR-47. Define confirmation in §3
as an act by the person on a channel the harness agent cannot answer, record
the channel in the journal, and state that the background service never
starts a change.

**Consequence if shipped:** A pack install or MCP registration runs without
a dry run, and later an agent can confirm its own changes and the journal
will record them as the person's.

---

## F13. FR-46's "visible win" requires launching a harness, which writes harness state

**Location:** FR-46 third consequence; SM-8; §5 "Observing does not change
the harness"; §6 "Answering prompts (folder trust, consent) on anyone's
behalf".

**Problem:** "At least one harness working in at least one repo" can only
be shown by starting the harness there. That launch writes session records,
`~/.claude.json`, and raises a folder-trust prompt. Either usher launches it
(a change not in any list, and not observation) or the person does, in which
case the check is a human statement and SM-8 cannot be counted. "Working" is
not defined.

**Suggested fix:** Define the win as a check usher can run without starting
a session (harness installed, auth route present, repo mapped, trust state
reported), plus the person's own launch, which usher then observes. Say that
usher never accepts the trust prompt.

**Consequence if shipped:** The bootstrap epic either launches harnesses and
breaks the zero-state-change invariant, or ships a consequence nobody can
test.

---

## F14. FR-66's in-memory key is visible to usher's own observers

**Location:** FR-66 first and second consequences; FR-5 ("with the
environment variables they actually receive"); FR-10 process table; §5
credential scan.

**Problem:** Harnesses take a brokered key through an environment variable
or a helper's stdout. On macOS a same-user process can read another
process's environment, and FR-5 explicitly maps launch contexts by the
environment variables they receive. So usher's own mapper, or a light
collector report of a launch context, can capture the key FR-66 says is
"never written to disk, logged, journaled or reported". "Never lands on disk"
is also outside usher's control, since a harness may cache an API key it was
handed.

**Suggested fix:** In FR-5 and FR-10, record environment variable names only,
never values, with a test that injects a marker key at launch and checks
every map, snapshot and report. Narrow FR-66's claim to what usher writes,
and add a harness knowledge item per harness on whether it persists a key it
was given.

**Consequence if shipped:** The v2 key broker leaks its key through the v1
mapper, which already ships.

---

## F15. FR-64's argument checks refuse legitimate arguments and leave "did not ask for" undefined

**Location:** FR-64 first bullet; addendum §7 rows on `~` and glob
characters.

**Problem:** The checks apply to "every path and argument". Legitimate
arguments contain these characters: git refs (`HEAD~1`), package extras
(`pkg[extra]`), version ranges with `*`, URLs with `~user`. "A glob character
the change did not ask for" has no defined source of truth for what was
asked, so the test cannot be written. The empty-variable row tests
`"$X/settings.json"`, a shell construct, while the next bullet says usher
never builds commands from strings, so the case is untestable as worded.

**Suggested fix:** Apply the unexpanded-variable, `~` and glob checks to
paths only. For arguments, require list passing and a per-installer
allowed-argument schema. Reword the empty-variable row as "a path component
resolved from an empty configured value".

**Consequence if shipped:** False refusals on ordinary installs, or a test
suite that passes because "did not ask for" was defined to whatever the code
does.

---

## F16. The one-change-at-a-time rule has no owner, scope or stale-lock rule

**Location:** FR-64 "One change run at a time per host"; addendum §7 row
"The second waits or refuses"; FR-63 ("stays on the host").

**Problem:** "Per host" conflicts with per-user state: brew is host-wide,
mise and harness config are per user, and a device-management install of
usher may run as root. Where the journal and lock live, and whose they are,
is unstated. A run killed mid-way leaves a lock; nothing says how it is
detected as stale. "Waits or refuses" accepts either outcome, so the test
cannot fail.

**Suggested fix:** Specify the lock scope (per user, plus a host lock for
host-wide installers), stale detection (holder pid and start time checked),
and one expected result. State the journal's owner and location for
multi-user hosts and for root-run installs.

**Consequence if shipped:** After the first crash, every later change
either blocks forever or steals a live lock, and two accounts on one Mac
write over each other's journal.

---

## F17. Addendum §7 has no rows for the change kinds the MVP actually makes

**Location:** addendum §7 table; §7.1 ("installs, updates, clones and
generated files").

**Problem:** Fifteen of sixteen rows exercise usher's own file writes, which
§7.1 defers to v2. The MVP change kinds get one row (argument quoting).
Missing: an installer that exits nonzero after partial install; network loss
mid-download; an installer that prompts for input; an installer with no
uninstall (FR-63 fourth bullet has no test); a clone into an existing
directory; a partial clone; an update whose signature fails; temp files left
behind after a crash (the kill row checks the target, not the leftover temp
file in a repo directory, where git or a hot-reloading harness may pick it
up).

**Suggested fix:** Add rows per MVP change kind, and tag each row with the
FR-64 tier (F1) it tests. Add a row that a crash leaves no temp file in the
target directory after the next run.

**Consequence if shipped:** The gate in §5 goes green on v2 behavior while
the v1 behavior it was written for is untested.

---

## F18. FR-55 in scope depends on templates, which are out of scope

**Location:** FR-55 description and first consequence; §7.1 second limit;
§7.2 "Templates (v2)"; §4.11.

**Problem:** FR-55 is in the MVP and draws recommendations from "the
templates that apply to them". §7.1 says "templates come with the team
instance" and §7.2 defers FR-56. FR-55's consequences have no variant for
the MVP's template-free state, and its cost-reduction bullet ("cache
settings that do not fit the time between turns", "MCP connections
configured and never used") needs activity data from FR-53, also v2.

**Suggested fix:** Give FR-55 an MVP form: sources are profiles and the
person's own records; cost-reduction recommendations use only FR-26 data.
Move the template source and the FR-53-dependent bullets under §7.2.

**Consequence if shipped:** The recommendations epic either pulls FR-53 and
FR-56 forward under FR-55's name or ships consequences that cannot be met.

---

## F19. Untestable budgets and attention rules

**Location:** FR-62 second consequence ("Every in-LLM surface has a token
budget"); §5 "Attention budget"; FR-13 "batched and quiet by default".

**Problem:** None states a number, a unit or a measure, so no test can fail.
"Quiet by default" does not say which notice kinds interrupt out of the box.

**Suggested fix:** State the budget per call and per session, or state that
architecture sets it and the PRD requires a published number with a test
that it is respected. List the default notice kinds that interrupt.

**Consequence if shipped:** Each surface sets its own budget, or none, and
the requirement is satisfied by a config key that is never checked.
