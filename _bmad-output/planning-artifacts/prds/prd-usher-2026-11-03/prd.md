---
title: "PRD: usher"
status: final
created: 2026-11-03
updated: 2026-11-05
inputs:
  - _bmad-output/planning-artifacts/briefs/brief-usher-2026-11-02/brief.md
  - _bmad-output/planning-artifacts/briefs/brief-usher-2026-11-02/addendum.md
  - _bmad-output/planning-artifacts/research/technical-harness-settings-observability-map-2026-11-02/research.md
---

# PRD: usher

*Working title; the name is open question 10.*

## 0. Document Purpose

This PRD is for the operator, the architect who takes it next, and the seats
that will write epics from it. It builds on the 2026-11-02 brief and its
addendum, and on the harness settings research of the same date; it does
not repeat their evidence. Terms in §3 are used exactly as defined. Features
in §4 carry FRs with stable IDs; FR-43 to FR-67 were added after the first
finalization (FR-43 sits first in §4.1; the rest sit where their feature
belongs). Inferences not yet
confirmed are tagged `[ASSUMPTION]` inline and listed in §10. Mechanism
detail lives in `addendum.md` beside this file: how each harness is read,
with its test cases; the bootstrap sketch; the install failure catalog; the
boundaries by objective; and the change-safety test cases.

Calibration: built inside this fleet first, at internal-tool depth.
Launch-grade sections (license, public support, release process) wait for
the going-public decision (open question 11).

## 1. Vision

**Objective: setup & maintain settings.** Usher sets up a person's AI
harnesses and maintains their settings from then on. Think wakatime plus
enterprise configuration management: a quiet client on each host, and a
service a team runs, which together install, analyze and recommend.
Collecting and reporting serve those three.

Usher transfers the effort an expert has already spent on their AI tooling
to the next person, so the newcomer does not have to match it. A senior
engineer who has tuned their harnesses over months can show a newcomer what
is in effect, where each setting lives and which tools and versions are
running, without describing files, paths and JSON structure from memory.
The first form of team sharing is the same as standing over a colleague's
shoulder; usher makes it precise and repeatable.

Usher does this by keeping an inventory of settings, mapping environments,
and carrying settings between them. The inventory lists every settings
source on a host for every harness, whether or not anything reads it;
across a team it answers who runs which harness, at which version, with
which settings, on which hosts (FR-18, §4.10). Each person, host, directory,
repo and use case has an environment,
and usher describes every one with the same detail, so any two can be
compared in either direction. There is no single baseline. Usher reports
facts, flags findings, attaches observed effects such as cache reuse and
activity, and recommends. People decide what is better (SOUL §8).

Setup covers more than settings: installing and updating the harnesses and
the tools around them, attaching the supporting services a person needs,
bootstrapping method content through sideshow or eos, and getting a person
into a repository even when they are not fluent in git. Maintaining covers
restoring a seat after a reinstall, a new machine or time away, and
catching a person up on what their team changed.

What makes usher worth more than a script is the loop: a person's better
setting becomes a proposal, a named reviewer decides, and the change is
offered to the team with credit. The same loop extends past settings to
skills, prompts and workflows. A team's service carries the shared side:
a signed catalog of profiles and options, compliance templates that check
setups and produce recommendations, and an options index that extends
every client without a client release. Usher is reached from the command
line, from inside the AI harness itself, from an app, and runs quietly in
the background.

After install usher stays quiet. It watches for the ground moving under a
seat (a self-update, a reinstall, a setting that changed) and tells the
person, so they can re-find their seat. It acts on a person's settings only
with their confirmation.

Usher is local-first and open source, so a team's security reviewer can
check its posture in the code. Any use of AI, on the host or for analysis
and queries, follows progressive codification (§7.2). §7 sets the shipping
order, not this section.

## 2. Target User

### 2.1 Jobs To Be Done

- **Newcomer (primary user, the developer).** Get to a working seat fast and
  spend my attention on using AI, not on configuring the tools. Know which
  recommended settings are valid for the versions I run, and exactly where
  each one goes. Find out when something under my seat changed. Get into the
  repos I
need even if I am not fluent in git.
- **Expert (customer, the fleet engineer or AI advocate).** Hand my working
  setup to a colleague without dictating it. Find what I forgot, what is
  stale and what conflicts in my own environment. Show the team, with
  evidence, which setups work best for which use cases and which tools and
  versions are in use.
- **Organization (beneficiary).** See who runs what software on which
  projects without seeing content, and pay less for more. A person may
  belong to several organizations, each with its own consent.
- **Chat-product users (later cohort).** People who use AI only through
  chat products; sequenced (§7.2), and expected to be the largest group in
  time.
- **Non-human seats (third subject).** Agent sessions launched by marvel are
  environments too. Usher maps them; how the settings usher maintains reach
  a marvel-launched session is the usher-marvel seam (open question 2).

### 2.2 Non-Users

- Anyone seeking enforcement, the content of prompts or files, or a
  per-person ranking (§6). Usher checks setups against templates and
  recommends; it does not enforce.
- Anyone wanting usher to manage chat conversations. The Claude desktop app
  and Claude's general settings are environments (FR-7); conversations are
  not.

### 2.3 Key User Journeys

These are typical stories, not history. Finding usher on GitHub is
illustrative; this fleet is the first user.

- **UJ-1. John gets lost on the way to his seat (today, without usher).**
  John is new to AI on a team that already uses it. He is pointed at a repo
  and given a Claude login. He finds the Claude desktop app and soon loses
  his way in a README that lists string-based settings with no hint of where
  they go. Two days later he watches a colleague's screenshare, asks, and is
  pointed at the Claude Code install; he moves over by imitation. When he
  finally places the recommended settings, he cannot know that three are
  stale or invalid. His team runs dozens of differing settings without
  knowing it; one person edits the recommendations after install and nobody
  reads them. It takes John three weeks to get going, and two months pass
  before anyone asks why his usage and cache reuse are worse than the
  team's.
  **Edge case:** a setting that silently stopped applying looks the same as
  one that works.

- **UJ-2. An expert's environment becomes John's starting point.**
  John and a senior platform engineer try to talk through context,
  compaction, cache reuse, models, effort, `.claude/settings.json`,
  `CLAUDE.md` and per-directory settings. Both are frustrated by the
  precision needed to describe files, paths and JSON. They realize that if
  John is lost, every new hire will be, and wonder how many current members
  are out of step and how many hours and lessons the team loses each week.
  They find usher. The engineer installs it and captures his own tuned
  environment as a profile. Usher's first map of it shows seven settings he
  had forgotten (four never documented), two that are obsolete or no longer
  optimal for his versions, and a conflict between two repo-level settings
  he did not know about. He reviews the capture, drops what is personal,
  notes why each entry matters, and hands John the file. Usher shows John,
  for his own versions and directories, exactly where each entry goes and
  which already match.
  **Climax:** a week later John is using dozens of the engineer's settings
  with no friction.
  **Resolution:** the docs and repo settings are updated. The engineer plans
  a talk for next month: the team's AI numbers rose even from the passive
  repo changes, and he pitches team-wide adoption to share settings, compare
  which setups work best per use case, and see which tools and versions are
  in use.
  **Edge case:** an entry valid for the engineer's codex version is retired
  in John's; usher flags it before John applies it.

## 3. Glossary

- **Harness:** an AI tool whose setup and settings usher maintains. The
  supported set is in §7.1 and in harness knowledge.
- **Support level:** what usher can do for a harness, stated per harness
  and per harness version as the list of capabilities usher offers for it:
  inventory, install, map, compare, place and apply. A harness can be
  supported with a
  short list, for example map and compare before placement.
- **Harness knowledge:** usher's versioned data about each harness (layers,
  keys, valid values, defaults, version ranges, stale-key behavior,
  hazards), each item with its reference and the date it was last verified.
- **Settings inventory:** every settings source on a host for each
  harness, whether or not any launch directory reaches it, with its
  location, harness, layer and status (FR-43).
- **Environment:** what usher describes for one harness as seen from one
  launch context (a launch directory, or a scheduled or headless launch
  with its own environment variables) on one host for one person or seat.
  An environment has settings in effect, their provenance, versions and
  running processes, and the parts of the seat around the harness: MCP
  registrations, packs, client config for fleet tools, the backend and auth
  route in use (presence only), and the toolchain. A person has many
  environments.
- **Recommendation:** a proposed change to an environment, drawn from a
  profile, a template, or the person's own records, with its reason and
  what usher could not observe. A person accepts or declines it.
- **Template:** a team's declared check of a setup (desired settings,
  tools, versions or limits) that produces findings and recommendations.
  A template checks; it is not a baseline everyone must match.
- **Team instance:** the light collector or central service a team runs,
  and the team customization it holds.
- **Personal layer:** the entries a person marks as their own, in any file
  (a key, or one entry of a list). Usher never writes a marked entry, never
  recommends against it, and never treats it as a difference to resolve.
  Nothing is marked until the person marks it; a value they set by hand is
  still shown as a conflict before any write (FR-49).
- **Profile source:** a file, directory or git repository a person names as
  a place to read profiles from, such as a team repo. A team instance's
  catalog (FR-60) is also a profile source.
- **Change:** anything usher does that alters a host: an install, update,
  clone, generated file, settings write, restore or rollback.
- **Change journal:** usher's local, append-only record of every change,
  with what it needs to undo each one (FR-63). Usher's own state (the
  journal, backups, caches) is not itself a change.
- **Launch directory:** a directory a harness is started from.
- **Environment label:** a name a person gives an environment or a set of
  them, such as a use case ("reviews", "long refactors").
- **Layer:** one source in a harness's settings stack (managed, flag,
  project, user, session, system).
- **Setting in effect:** the value a harness applies after its layers
  resolve. Distinct from what any one file says. Permission rules are
  settings like any other.
- **Provenance:** the layer, file and version that supplied a setting in
  effect (for a list that merges across layers, each of its entries), and
  whether
  usher read it from the harness or rebuilt it from files.
- **Environment map:** usher's description of an environment.
- **Profile:** a reviewed, versioned file holding selected entries from one
  or more environments, with a note per entry. A profile is a shareable
  environment, not a standard.
- **Entry:** one setting in a profile, with its label, value, harness
  version range and note.
- **Path token:** a placeholder in an entry for a location that differs
  between people and hosts (home, config directory, repo root, user name),
  or a pattern that matches several locations. Resolved per environment.
- **Adoption record:** a local note that a person adopted a given entry
  from a given profile version, so drift against it can be computed.
- **Comparison:** the differences between any two environments or between
  an environment and a profile, in either direction.
- **Drift:** installed or resolved state falling behind its own declared
  source (an adopted profile entry, an installer's source, a merged
  script). Narrow, with no upside. All other differences are just
  differences.
- **Finding:** a fact usher flags about an environment: an invalid or
  retired key for the version, a conflict between layers, a setting that is
  set but undocumented, a setting a file declares but the harness ignores,
  a lock weaker than it looks, a stale tool or process, drift.
- **Hazard check:** a finding that rests on a harness claim, where the claim
  needs its own test before the check ships.
- **Observed effect:** a measured outcome attached to an environment, such
  as cache reuse per session. Effects are facts; judging them is human.
- **Seat:** a working harness setup for a person or an agent session.
- **Process table:** the running harness processes on a host, with start
  time, version and launch directory.
- **Watcher:** usher's local background process that notices
  changes under a seat.
- **Service operator:** the vendor running the model service a person uses
  (Anthropic, OpenAI, a cloud provider), whose console reports usage and
  cost.
- **Light collector:** a small service a team hosts itself that receives
  reports from hosts that opt in and answers queries across them (§4.10).
- **Reported fields:** the exact list of fields a host may send to a light
  collector, published with usher.
- **Reporting:** usher sending data off the host to a light collector. A
  person saving or sharing a local report by hand is not reporting.

## 4. Features

### 4.1 Settings inventory and environment map

**Description:** Usher first lists every settings source on the host for
each supported harness, whether or not anything reads it. On top of that
inventory, for each launch directory, it shows the settings in effect, the
provenance of each, the versions in use and what it could not observe. How
each harness is read is in addendum §1. Realizes UJ-1, UJ-2.

#### FR-43: Settings inventory

A person can list every settings source on a host for each supported
harness, whether or not any launch directory reaches it, with its location,
harness, the layer it would belong to, and its status.

**Consequences (testable):**
- Each source has one status: in effect for at least one mapped
  environment, shadowed (hidden by a higher-ranked source that wins: under
  first-wins, a managed source below one that delivers a policy key is not
  applied), unread (no mapped environment reads it), or unknown (usher
  cannot tell).
- Sources are found at the locations harness knowledge names for each
  harness and under the roots the person configures (FR-5).
- Keys usher does not recognize for the harness version are listed, never
  dropped.
- Each source is recorded as derived state: presence, path, size, modified
  time and key names; never file content or credential values (§5).
- A harness whose support level stops at inventory still shows its
  sources, marked as not resolved.
- The inventory includes the parts of the seat around each harness (§3
  Environment): MCP registrations, packs, client config for fleet tools,
  backend and auth route (presence only), and the toolchain.

#### FR-1: Settings in effect with provenance

A person can see, per harness and launch directory, each observed setting in
effect and its provenance. Realizes UJ-2.

**Consequences (testable):**
- Each setting names its layer and file, and is marked harness-reported or
  rebuilt.
- For a list that merges across layers, provenance is given per entry
  (cases per harness in addendum §1.1).
- Where the harness reports its own resolved view, usher's values and layers
  match it for that directory.
- Precedence among layers is the harness's, never usher's.
- Layers present but disabled or skipped are listed with the harness's
  reason where it gives one.

#### FR-2: Declared but not in effect

A person can see where a file declares a value the harness does not apply.
Realizes UJ-1 and its edge case.

**Consequences (testable):**
- Each declared-but-not-applied case listed for a harness in addendum §1.1
  is reported as declared and not in effect.
- A key the harness skips silently is reported as skipped, never as in
  effect.

#### FR-3: Version stamp and blind spots

Every map names the harness version it assumes, the method used per reading
and the list of things it could not observe.

**Consequences (testable):**
- No map is produced without a version stamp per harness.
- Each reading names its method: harness query, file read or rebuild.

#### FR-4: Instruction and agent files as metadata

A person can see which instruction, skill and agent files each harness
discovers for a directory, with presence, path, permissions and size, and
any file the harness discovers twice because it reads two locations.

**Consequences (testable):**
- No file content is read into a map or a report.

#### FR-5: Every launch directory

A person can map the directories they actually launch from, not only the
current directory and the user file, through directories they configure plus
directories usher discovers.

**Consequences (testable):**
- A person can name directories and roots to include or exclude.
- Usher proposes further directories it discovers (from the harness's own
  session records and repo roots under the configured roots); the person
  confirms before they are mapped.
- Scheduled and headless launches (for example a launchd job) are mapped as
  their own launch contexts, with the environment variables they actually
  receive, which differ from an interactive shell's. Variables are
  recorded by name; a value is recorded only for a variable harness
  knowledge marks as a setting, never for a credential (FR-66).
- The scope and consent of discovery beyond configured roots follow open
  question 3.

#### FR-6: Folder trust state

A person can see the trust state that gates which layers load, per
directory and harness.

**Consequences (testable):**
- The trust state matches the harness's own trust record for the repo root
  (addendum §1.1).
- A directory whose project layers do not load because it is untrusted is
  reported with that reason.

#### FR-7: Claude desktop app and Claude general settings

A person can see the Claude desktop app's settings and Claude's general
(account-level) settings as environments, beside Claude Code. Realizes UJ-1,
where John starts in the desktop app.

**Consequences (testable):**
- A surface usher cannot read is listed as not observed, never left out.
- A setting that exists in both the desktop app and Claude Code is shown
  once per surface, with a note of which value applies on which surface.

`[ASSUMPTION: which desktop-app and account settings are readable locally,
and which only through the app or the vendor's own interfaces, needs a probe
before usher relies on any of them.]`

#### FR-8: Environment labels

A person can label environments, for example by use case, and filter maps,
comparisons and effects by label.

**Consequences (testable):**
- A label is stored locally and travels in a report only if the person
  includes it.

### 4.2 Install, version and process health

**Description:** Usher reports which harnesses and fleet tools are
installed, from which source, at which version, whether each is behind its
source, and which harness processes are running and from what. It drives
existing installers (mise, brew, sideshow, `tools/install.sh`, vendor
installers) rather than re-hosting them. Realizes UJ-1 and the edge case of
UJ-2.

#### FR-9: Installed, source, version, behind

A person can see each tool's install state, install source, version and
whether it is behind its source.

**Consequences (testable):**
- A check that errored is reported as an error, never as current.
- A tool whose installed version file reads current while its content
  differs from source is reported as drift.
- Each failure mode in addendum §3 is caught.

#### FR-10: Process table

A person can see the running harness processes with start time, version and
launch directory, and which processes started before the binary on disk
last changed.

**Consequences (testable):**
- A session running a binary older than the installed one is flagged.

#### FR-11: Install by driving existing installers

A newcomer can install a missing harness or tool through the installer that
owns it, with usher naming the command and the source.

**Consequences (testable):**
- An install runs only on the person's action or confirmation; usher shows
  the exact command first.
- Configuring a harness during an install follows the confirmation rule
  (§5).
- Where an install needs a credential, the vendor's own login runs; usher
  never copies a credential between hosts.

#### FR-12: Device-management overlap

A person can see when device management also manages an app or setting usher
observes.

**Consequences (testable):**
- An app or setting governed by an installed device-management profile is
  marked as managed, with the profile's identifier.
- An app whose bundle was replaced, and whose custom profiles went missing
  at the same time, is reported as a reinstall that dropped them.

#### FR-44: Update on confirmation

A person can update an installed harness, tool or usher itself through the
installer that owns it, after seeing what will change.

**Consequences (testable):**
- An update runs only on the person's confirmation and shows the from and
  to versions first.
- Before updating a binary that running sessions use, usher names those
  sessions (FR-10) and lets the person choose when.
- Usher updates itself through the installer that owns its install (for
  example brew), like any other tool; a direct install updates from a
  signed feed the team names, verified before anything is installed
  (operator ruling, 2026-11-09).

#### FR-45: Declared tools and toolchain

A profile can declare the tools, toolchain and versions a seat needs, and
usher compares each host with that declaration.

**Consequences (testable):**
- A tool missing or outside its declared version range is a finding, with
  the install or update that would fix it (FR-11, FR-44).
- The declaration can be expressed through the tool manager a team already
  uses (for example a mise file), which usher reads, or generates on
  confirmation as a change (FR-63).

#### FR-46: Guided, resumable bootstrap

A newcomer can go from nothing to a working seat in one guided run that
can stop and resume.

**Consequences (testable):**
- Every step is idempotent; rerunning a finished step changes nothing.
- Every failure names the step, the command or check that failed with its
  exit status or error, and at least one next action (retry, skip, or a
  named manual step).
- The first session ends with a visible win: at least one harness working
  in at least one repo, launched by the person.

#### FR-47: Supporting services and method content

A person can attach the supporting services their seat needs (for example
issue trackers, MCP servers and accounts) and bootstrap method content
through sideshow or eos.

**Consequences (testable):**
- Each attachment runs on confirmation, as a change (FR-63, FR-64).
- Each attachment uses the service's own setup or login; usher holds no
  bearer credential for it (ADR-009).
- Pack and method content is installed by sideshow or eos, which usher
  drives and does not re-implement.

#### FR-48: Repository help

A person who is not fluent in git can get a repository they need onto their
host and open it in their harness.

**Consequences (testable):**
- Usher names each command before running it and runs it on confirmation.
- Access uses the forge's own login and the host's own credential; usher
  never copies a credential between hosts.

#### FR-65: Seat a reviewer at a run variant

A person can sit down at any agent-grown run variant (a repo a factory or
legion grew) with a working seat, to review it.

**Consequences (testable):**
- Usher sets up the variant as an environment: the repo on the host, the
  declared tools, and a named profile, each step on confirmation (FR-46,
  FR-48).
- A variant's own tool declaration (for example its mise file) is
  untrusted: it is shown, and runs only on the person's confirmation.
- Usher sets up the seat only; reviewing and judging the variant are the
  person's, or critic's (§6).

#### FR-66: Key broker at launch

A person can launch a harness with a short-lived key that a vault or cloud
login they own issues for that launch, so no key lands on disk.

**Consequences (testable):**
- Usher asks the person's vault for a short-lived, scoped key at launch
  and hands it to the harness in memory; it never holds the grant that
  issues the key, never reads a stored long-lived key, and never writes
  the person's store (ADR-009, brokering, not custody).
- A provider with no issuing route gets no brokered launch; Anthropic API
  keys cannot be issued programmatically today (operator ruling,
  2026-11-09), and usher shows the person which harnesses have a route.
- The key's value is never written to disk, logged, journaled or reported.
- A plaintext key found by the credential scan (§5) comes with a
  recommendation to move it behind the broker.

#### FR-67: Remove usher

A person can remove usher and choose what it leaves behind.

**Consequences (testable):**
- Before removing anything, usher lists its background service, its
  binaries, its own state (journal, backups, snapshots, reports) and every
  change from the journal still in effect.
- The person chooses to keep the changes usher made, or to roll back some
  or all of them first (FR-63).
- Tools usher installed through their own installers stay unless the
  person picks each one for removal, which runs through that installer.
- The journal and backups are kept, and usher says where, unless the
  person asks for them to be deleted.
- After removal no usher process, login item, scheduled job or OTLP
  endpoint remains; a test checks this.

### 4.3 Watcher

**Description:** After install usher keeps quiet and watches. When the
ground moves under a seat, it tells the person what changed and where, so
they can re-find their seat. It changes nothing. Realizes the UJ-1 edge case
and the brief's "re-find it when the ground moves".

#### FR-13: Change notices

A person is told when a harness self-updates, when a running session is
older than the installed binary, when device management reinstalls an app
usher observes, and when a setting in effect changes.

**Consequences (testable):**
- Each notice names the environment, what changed, and the before and after
  versions or values.
- The watcher writes only usher's own state and never calls a harness
  command that can write state (§5).
- Notices are batched and quiet by default; a person sets which kinds
  interrupt them.

#### FR-14: Dated settings history

A person can see when each setting in effect changed, per environment, so
the change can be lined up with the daily usage and cost the service
operator reports. Realizes SM-2.

**Consequences (testable):**
- The watcher keeps local, dated snapshots of each environment map and
  reports the differences between consecutive snapshots.
- Snapshots are coarse (at most one a day per environment, unless the person
  asks for more or a change notice fires) and hold derived state only.

### 4.4 Comparison and findings

**Description:** Any two environments, or an environment and a profile, can
be compared in either direction. Usher shows the differences and the
findings on each side, and leaves the judgment to people. Realizes UJ-2.

#### FR-15: Compare in either direction

A person can compare two environments, or an environment and a profile, and
see each entry as present, missing, different or extra, with provenance on
both sides.

**Consequences (testable):**
- Comparison runs against settings in effect wherever a resolved view
  exists, and against files only where none does; the report says which.
- Swapping the two sides swaps "missing" and "extra" and changes nothing
  else.
- List entries whose order the harness ignores compare as sets; order is
  compared only where harness knowledge says the harness uses it.
- Path tokens are resolved on each side before comparing.

#### FR-16: Findings per environment

A person can see findings on any environment: invalid or retired keys for
the version in use, conflicts between layers or between repo and directory
settings, settings that are set but appear in no profile or doc the person
names, locks weaker than they look, and drift.

**Consequences (testable):**
- An entry naming a key the harness has retired is flagged for every
  version from the first release that retired it, with what the harness now
  does with it; the harness knowledge item records that version.
- An organization deny that a lower layer's allow entry can override is
  flagged; only managed-only keys are reported as locks.
- A hazard check ships only after its test passes on the current harness
  version.
- Each case in addendum §1.1 is detected.
- Each finding names what the harness does with the key in question (warns,
  aliases, fails, or skips silently), from harness knowledge.

#### FR-17: Version-aware entries

A person can see, for each profile entry, whether it is valid for the
harness version in the target environment. Realizes the UJ-2 edge case.

**Consequences (testable):**
- An entry outside its stated version range is marked before placement.
- An entry with no version range is marked as unchecked.

#### FR-18: Which environments carry entry X

The expert can answer "which hosts or directories carry entry X" and "which
hosts run a stale copy of tool Y".

**Consequences (testable):**
- The answer works over a set of local reports given to usher, and through
  the light collector (FR-41).
- Each answer lists the hosts it could not see.

### 4.5 Profile capture and sharing

**Description:** An expert captures an environment into a profile, filters
out what is personal, annotates why each entry matters, and hands the file
on. A newcomer consumes it. The same file format serves sharing by hand and
the central service. Realizes UJ-2.

#### FR-19: Capture

An expert can capture an environment, or chosen parts of several, into a
draft profile.

**Consequences (testable):**
- Capture never includes credential values or file content.
- Each captured entry records its source environment, layer and harness
  version.

#### FR-20: Portable entries

Entries carry path tokens and patterns, so one profile works across
different people's filesystems and locations.

**Consequences (testable):**
- Capture proposes a path token wherever a value contains the capturer's
  home, config directory, repo root or user name; the expert accepts or
  edits each.
- A profile carrying a literal path that names its author's home is
  flagged at review.

`[ASSUMPTION: the token set and pattern syntax are the architecture step's;
the PRD requires only that they exist and resolve per environment.]`

#### FR-21: Filter and annotate

An expert can drop entries and add a note per entry before the profile is
shared.

**Consequences (testable):**
- Nothing is shared that the expert has not reviewed; a draft is marked as
  unreviewed until saved.

#### FR-22: Consume

A newcomer can load a profile and see it compared with their own
environments, with findings for their versions.

**Consequences (testable):**
- Loading a profile changes nothing; entries are applied only when the
  person chooses (FR-49).
- Each entry shows whether it already matches, differs, is missing, or is
  invalid for the newcomer's version.

#### FR-23: Adoption record

When a person applies an entry and verifies it (FR-25), usher records which
profile version it came from, so later drift against it can be shown.

**Consequences (testable):**
- An adopted entry whose setting in effect later changes is reported as
  drift against the adoption record.

### 4.6 Placement, apply and restore

**Description:** For each entry a person chooses to adopt, usher shows
exactly where it goes: the file, the layer, the key path and the structure
for that harness and version, and whether a higher layer will override it.
The person applies it, or usher applies it on their confirmation (§5).
Realizes UJ-1, UJ-2.

#### FR-24: Exact placement

A person can see, for an entry and a target directory, the file path, the
layer, and the snippet in that file's own format.

**Consequences (testable):**
- An entry is never placed in a layer the harness ignores for that key
  (cases in addendum §1.1).
- When a higher layer will override the placed value, usher says so before
  the person applies it.

#### FR-25: Verify after applying

A person can re-map after applying an entry and see whether it is now in
effect.

**Consequences (testable):**
- The re-map reports the entry as in effect, overridden (with the
  overriding layer), or not loaded (with the reason).

#### FR-49: Apply with write safety

A person can have usher write an entry, a profile or a recommendation into
their settings, after confirming.

**Consequences (testable):**
- Usher shows a dry run and a diff before writing, journals the write and
  can roll it back (FR-63), and writes under the safe-execution rules
  (FR-64).
- Nothing is overwritten silently; a value the person set by hand is shown
  as a conflict.
- Usher never writes a managed or device-management layer, and never
  writes an entry the person marked as personal (§3).
- A person can mark or unmark any entry as personal, from a map or a
  comparison.

#### FR-50: Restore a seat

A person can bring a host back to their declared seat after a reinstall, a
new machine or time away.

**Consequences (testable):**
- Usher lists every difference from the declared seat and the step that
  would restore each one, then applies the steps the person confirms
  (FR-44, FR-49).
- A reinstall that dropped custom profiles (FR-12) is restorable.

#### FR-51: Catch up

A person can see what changed in the profiles from the profile sources
they follow (§3) since the version they last loaded, with the reason
recorded for each change.

**Consequences (testable):**
- Usher records, per profile source, the profile versions the person last
  loaded; "since they last synced" means since those versions.
- A profile from a source is untrusted until the person loads it, and
  loading changes nothing (FR-22).
- Each change names its profile version, author and note.
- The person can adopt each change, or keep their own value.

#### FR-52: Organization drop-in

Where an organization wants a setting required, usher can produce the
artifact the organization's own device management deploys (for example a
managed-settings drop-in).

**Consequences (testable):**
- Usher writes the artifact to a file for the organization; it never
  deploys it.

### 4.7 Observed effects

**Description:** Usher attaches measured outcomes to an environment, so a
settings difference and its effect can be seen together. Effects are
computed locally from fields in the harness's own session records (§5).
Realizes the UJ-1 problem and the UJ-2 resolution.

#### FR-26: Cache reuse per session

A person can see, per session, the share of input tokens read from cache
versus written to cache at full cost, with the model and environment in
effect.

**Consequences (testable):**
- A session whose model is missing from the price table is reported as
  unpriced, not dropped.
- Throttled or failed calls are counted apart, so they do not read as
  efficiency.

#### FR-27: Effects beside settings

A person can see observed effects next to a comparison of two environments
or next to the dated settings history.

**Consequences (testable):**
- Each effect states the sessions and dates it covers and the backend
  (service operator and model) it was measured on.
- Effects from different backends are never combined in one figure.

#### FR-53: Activity effects

A person can see activity per environment and project: tool calls, token
counts, errors experienced, model changes within a session, time between
turns against the cache lifetime, cache resets, and instruction-file size
over time.

**Consequences (testable):**
- Each effect is read from named fields only (§5) and states its source
  and backend.
- A signal the harness does not expose is listed as not observed.

### 4.8 Reports and reporting

**Description:** Every map, comparison and effect is available as a local
report a person can read and choose to share by hand. Nothing leaves the
host on its own unless reporting to a light collector is turned on.

#### FR-28: Local reports

A person can save any map, comparison or effect as a file in a published,
versioned format.

**Consequences (testable):**
- A saved report validates against the published schema for its version.
- A report holds only reported fields.

#### FR-29: Reporting off by default

Reporting is off by default; the off switch gates the code path, not just
the send.

**Consequences (testable):**
- A test proves no reporting egress in the default state, and no traffic
  beyond the endpoints the published behavioral profile lists for the
  version check (FR-9), which carries nothing about the person and has its
  own off switch (operator ruling, 2026-11-09).
- A test proves the enabled state sends only reported fields.
- A published "we do not collect" list is part of the contract.
- DO_NOT_TRACK set in the environment keeps reporting off on that host,
  and usher says so.

#### FR-54: Reporting levels and OTEL

When reporting is on, a person or team chooses its level, from metrics
only to full OTEL, and usher can take in the harness's own OTEL output.

**Consequences (testable):**
- Each level has its own published field list; a higher level never
  turns on without the same consent as the first (FR-39).
- Usher can act as the local OTLP endpoint for a harness that exports
  OTEL, and forwards only the fields the chosen level allows.

### 4.9 Harness knowledge upkeep

**Description:** The harnesses ship every few days, so what usher knows
about them goes stale on a clock and on every release. Harness knowledge
lives as versioned data with a reference for every item, and the repo
carries docs and helpers that let a maintainer update it quickly. Helpers
propose; a person accepts (SOUL §8).

#### FR-30: Referenced harness knowledge

Every item of harness knowledge cites its reference (vendor docs, changelog,
source code, or a probe usher ran) and the date it was last verified.

**Consequences (testable):**
- An item without a reference or a verification date fails the knowledge
  file's structural check.
- A finding shown to a person names the knowledge item and its reference.
- Harness knowledge records each harness version's support level (§3), and
  usher shows a person which capabilities it offers for each harness they
  use, so a missing capability reads as unsupported, never as a clean
  result.

#### FR-31: Updated without a client release

Harness knowledge can be updated and shipped without a new usher build.

**Consequences (testable):**
- The client reads harness knowledge from a versioned, signed data file and
  reports which version it used.

#### FR-32: Re-check by date and by release

Each item carries a re-check date set by its class (fast-changing keys and
defaults sooner, layers and precedence later) and is also due whenever a new
release of its harness is detected.

**Consequences (testable):**
- Usher marks findings that rest on an item past its date, or on an item
  not yet verified for the harness version in use.

#### FR-33: Stale-key behavior

Harness knowledge records, per harness and version, what the harness does
with a stale or unknown key: warn, alias, fail with an error, or skip
silently.

**Consequences (testable):**
- Each supported harness has at least one entry verified by probe (FR-35).

#### FR-34: Release helpers

A maintainer can run repo helpers that collect what changed in a new
harness release (changelog, generated schema, the harness's own help and
config output; sources per harness in addendum §1) and draft proposed
knowledge updates for review.

**Consequences (testable):**
- Helpers run under a scratch home and never touch the maintainer's real
  harness state.
- A helper's output is a proposed change for review, never an applied one.

#### FR-35: Re-runnable probes

Each claim that needs a test (the disputed claims in addendum §1, and each
new one) has a probe that a maintainer can re-run against a new release.

**Consequences (testable):**
- Each probe runs under a scratch home with a positive control, and records
  the harness version and the result in the knowledge item it verifies.

#### FR-36: Maintainer docs

The repo documents, per harness, where its truth lives (source, docs,
changelog), how usher observes it, and the steps to verify and ship an
update.

**Consequences (testable):**
- Each supported harness has a maintainer page naming its sources, its
  probes and
  its last full re-check.

### 4.10 Light collector

**Description:** A small service a team hosts itself. Hosts that opt in send
the reported fields; the expert queries it across hosts. Realizes UJ-2's
resolution (the team-wide view) and SM-2 at team level.

#### FR-37: Self-hosted, opt-in per host

A team can run the light collector on infrastructure it controls, and a
person can turn on reporting to it for their host.

**Consequences (testable):**
- The client's default stays off (FR-29). Reporting is turned on by the
  person, or by the organization through its own device-management
  channel; usher records which, and tells the person (FR-39).
- The light collector runs without any usher-operated service.

#### FR-38: Reported fields only

The light collector accepts only reported fields: settings inventories,
environment maps,
findings, versions, process table summary, dated settings history, cache
reuse per session and, where usher measures them, activity effects
(FR-53) at the chosen level.

**Consequences (testable):**
- The light collector rejects any field not in the reported fields.
- No file content, prompt content or credential value is accepted.
- The enabled-state egress test (FR-29) runs against the light collector.

#### FR-39: Consent before reporting

Before a host's first report to a light collector, the person sees what will
be sent, to whom, and how to stop; they can withdraw at any time, and
withdrawal stops sending at once.

**Consequences (testable):**
- No report leaves a host before the person confirms the disclosure for
  that light collector.
- Consent and withdrawal events are recorded on the host.
- A person can ask the light collector for, and delete, the data held about
  their hosts.

#### FR-40: Host identity

Each host authenticates to the light collector with its own credential,
which the light collector can revoke and the host can re-mint; no credential
is copied between hosts.

**Consequences (testable):**
- The credential passes the ADR-009 audience test: it means nothing outside
  that light collector.
- Revoking one host's credential does not affect another host.

#### FR-41: Query across hosts

The expert can ask the light collector the FR-18 questions across hosts,
and see findings across hosts and observed effects beside settings
differences.

**Consequences (testable):**
- Query results compare environments; no view ranks people.
- A person sees their own findings and recommendations before anyone else
  does.
- Per-person drill-down is available only after the team records that
  decision; rollups are aggregate by default.

#### FR-42: Retention

The team sets how long the light collector keeps data; the default is short.
`[ASSUMPTION: default retention of 90 days, enough for the F9
four-to-six-week window plus margin.]`

**Consequences (testable):**
- Data older than the retention period is deleted, and a deletion record is
  kept without the data.

**Feature-specific NFRs:**
- The light collector holds per-person data, under §5's per-person rule.
- Storage choice is the architecture step's call, smallest that works
  (open question 6).

### 4.11 Recommendations

**Description:** Usher proposes changes worth acting on and brings them to
the person, rather than waiting to be asked. Each recommendation says why
and what usher could not observe. Realizes UJ-1 and UJ-2.

#### FR-55: Local recommendations

A person can see recommendations drawn from the profiles in the sources
they follow (§3), the templates that apply to them, and their own records
(for example permission denials a harness recorded).

**Consequences (testable):**
- Each recommendation states its source, its reason, the versions and
  backend it assumes, and what usher could not observe.
- Recommendations that differ by backend for the same setting say so.
- Cost-reduction recommendations include cost-bearing settings (for
  example cache settings that do not fit the time between turns, MCP
  connections configured and never used, a spend cap not set).
- An allow entry offered as a recommendation is marked as a loosening.
- A recommendation is pushed only for the kinds the person set to notify
  them (FR-13); the rest wait in the list.

#### FR-56: Template checks

A team can declare templates, and usher evaluates them on each host and
across hosts.

**Consequences (testable):**
- A template produces findings and recommendations per environment; it
  never blocks the person or changes a setting by itself.
- Templates evaluate locally on each host; the team instance evaluates
  them across hosts.

### 4.12 Contribution loop

**Description:** A person's better setting becomes a proposal the team can
adopt, with credit. Declaring costs the person almost nothing; judging and
deciding stay human acts (SOUL §8). Realizes UJ-2's resolution.

#### FR-57: Propose a difference

A person can turn a difference in their environment into a proposal for a
team profile, with the body drafted from the observed difference.

**Consequences (testable):**
- The person adds only intent; scope, versions and observed effects are
  filled in from what usher already knows.
- The proposal is routed to a named reviewer with a review clock.

#### FR-58: Decide, ripple, credit

A reviewer can accept, hold as an option, or decline a proposal with a
reason; an accepted change is offered to the team, never forced, and
carries its author's credit.

**Consequences (testable):**
- An accepted change becomes a new profile version and reaches others
  through catch-up (FR-51).
- A proposal past its review clock is surfaced to the team.

### 4.13 Team instance and enrollment

#### FR-59: Enroll with a team

A person's usher can find and join its team's instance, and the team's
customization layers over usher's shipped defaults.

**Consequences (testable):**
- Enrollment shows the person what the team will receive before any report
  (FR-39).
- Team customization never overrides the personal layer (§3).
- A person in several organizations keeps each enrollment and consent
  apart (Q18).

### 4.14 Central service

**Description:** The service a team runs for the shared side: it grows out
of the light collector and is self-hosted first (SOUL §6).

#### FR-60: Signed catalog and distribution

The team instance distributes profiles, templates and recommendations
through a signed, versioned catalog in an open format.

**Consequences (testable):**
- A client verifies the catalog's signature before using any item.
- The catalog format is published, so another client could read it.

#### FR-61: Options index

The team instance can serve an index of options that extends what the
client can check, install or recommend, without a client release.

**Consequences (testable):**
- Each index item is signed, versioned and confirmed by a person at the
  point of use.
- An item the client cannot verify is ignored and reported.

### 4.15 Surfaces

#### FR-62: Where a person reaches usher

A person can use usher from a command line, from inside their AI harness
(an in-LLM surface, such as MCP), and from an app, while a background
service keeps the inventory and the watcher running.

**Consequences (testable):**
- From inside the harness, an agent can drive onboarding conversationally
  (install, clone, configure) under the same confirmation rules as every
  other surface.
- Every in-LLM surface has a token budget.

### 4.16 Change journal and safe execution

**Description:** Every change usher makes (§3) is recorded before it runs,
with enough detail to undo it, and is carried out so that a common failure
cannot widen it, lose data or leave a file half written. A change is one of
two kinds: a write usher makes itself, or a run usher delegates to the tool
that owns it (brew, mise, git, sideshow, a vendor installer). Both hold on
every surface.

#### FR-63: Change journal and rollback

A person can see every change usher has made on their host and undo any one
of them, or a whole run, from the journal.

**Consequences (testable):**
- The entry is written and flushed to disk before the change starts, and
  completed after it. A journal that cannot be read or written stops every
  change.
- Each entry records the time, the surface, the person's confirmation, the
  command or write, every target as a resolved absolute path, the state
  before (a content hash and a backup for a file, the version for a
  package), the state after, the exit status, and the undo. For a delegated
  run it also records the installer's known paths before and after, and the
  difference observed.
- The undo fits the kind of change. A file write restores its backup. A
  clone is moved aside only when nothing in it is new since usher made it
  (no commits, modified or untracked files); otherwise it is a conflict. An
  update reinstalls the earlier version through its installer where the
  installer can pin one; usher's own update keeps the previous binary.
  Where no undo applies, the entry names the manual step, and the person
  sees it before confirming.
- A rollback runs in reverse order and checks each target's current hash
  first; a target changed since usher wrote it is shown as a conflict and
  left alone unless the person chooses. Later entries that depend on the
  one being undone are listed and undone first, or the rollback stops.
- The journal is append-only and stays on the host; a rollback is a new
  entry, never an edit. A credential is recorded as present, never by value.
- Backups live in a store on the host readable only by the person, are
  never reported or put in a shared report, and are kept until the person
  removes them; usher reports the space they use.

#### FR-64: Safe execution

A person can rely on a change touching only what its dry run listed.

**Consequences (testable), for writes usher makes itself:**
- **Empty or unexpanded values stop the change.** Every target path is
  resolved and checked before use. A target that is empty, is a filesystem
  root, the home directory or a parent of it, or contains an unexpanded
  variable, a leading `~` or a glob character, is refused.
- **Scope is bounded.** Each kind of write has a list of the paths it may
  touch (for a settings write, the harness's known settings files). A
  target reached through `..` that leaves the list is refused. A symlink
  is written through to its target when the target is on the list, and is
  never replaced; one that leads off the list is refused. A write that
  would touch more targets than its dry run listed stops.
- **Credentials are not written.** A file harness knowledge marks as
  holding credentials (for example a harness's auth file) belongs to the
  vendor's login; usher never writes it, so never backs it up.
- **Mixed files are written by key.** Some files hold settings beside
  account data or trust records (for example Claude Code's user config,
  which holds MCP registrations, folder trust and account details). Usher
  writes only the keys harness knowledge lists as writable for that file,
  never a trust record (§6), and leaves every other key byte for byte as it
  was. The backup of such a file stays in the owner-only store (FR-63).
- **Nothing is lost.** Usher backs a file up and checks the backup reads
  back before writing it, and never deletes: a removal moves the target to
  the backup store. It checks free space first.
- **Nothing is half written.** Writes go to a temporary file in the same
  directory, are parsed in the file's own format, flushed to disk and
  renamed into place, so a harness that reloads the file sees the old
  version or the new one. Permissions, ownership, encoding, line endings,
  and comments and formatting outside the changed keys are kept.
- **Concurrent edits are caught.** A target whose hash changed between the
  dry run and the write stops the change.

**Consequences (testable), for delegated runs:**
- Commands run with arguments passed as a list, never as a string built
  from values; a path usher puts in an argument passes the same target
  checks. Any script usher writes and runs stops on an unset variable or a
  failed command.
- The exact command and its source are shown before the person confirms.
  An installer fetched and piped into a shell runs only from a pinned,
  verified source.
- A failed run is reported with its exit status and the paths that changed.

**Consequences (testable), for both:**
- One change run at a time per host, under a lock held by the usher process
  running it. A lock whose process is gone is released and journaled; a
  second run waits for the person and never runs alongside the first.
- A run stopped part way leaves each file usher writes old or new, and the
  journal shows which steps finished, so the person can resume (FR-46) or
  roll back (FR-63).
- Each of these has a test that injects the failure (addendum §7).

## 5. Cross-Cutting NFRs and Guardrails

- **Changes on confirmation.** Usher shows every change before making it
  and makes it only on confirmation. An agent working for the person
  confirms and applies as the person can (operator ruling, 2026-11-08);
  the journal records which surface confirmed. Locked settings belong to
  device management; usher is never the sole configuration authority.
- **Every change is journaled and fail-safe.** No change runs without a
  journal entry, written first, that names its undo or its manual step,
  which the person sees before confirming (FR-63), and without the
  safe-execution checks for its kind (FR-64). Like the observation
  invariant, this is structural with tests, so it may gate (SOUL §8).
- **Observing does not change the harness.** Before usher relies on a
  harness call (addendum §1), a test shows that the call does not write
  state, refresh a cache, start an update loop or
  trigger a self-update. Calls that can reset, fix, update or purge are
  never made. The count of harness state changes caused by observation is
  zero; this is a structural invariant with a test, so it may gate (SOUL
  §8).
- **Local observation from install** (operator 2026-11-04). Usher maps and
  watches from install onward, including a silent install by device
  management, and nothing leaves the host. Until the person confirms
  directories (FR-5), usher observes only the locations harness knowledge
  names for each harness (settings files, install locations, the process
  table); directories it discovers are kept as proposals and not mapped,
  and no home-wide sweep runs before open question 3 is settled. The
  first time the person opens usher, it tells them it has been observing
  since install and shows what it looked at. Sending anything off the host
  keeps its own opt-in (FR-39, charter B1).
- **Derived state, not content.** Usher reports that a value or file exists.
- **Session records: named fields only.** Where usher reads a harness's
  session records (FR-5, FR-26), it parses only named fields: timestamps,
  working directory, model, token and cache counts, tool and permission
  outcomes, error and throttle markers. Prompt and response text is never
  parsed, stored or sent. A test feeds a session record with marker text in
  every content field and checks that no output contains it.
- **Credentials: presence, never value.** A credential scan is off by
  default and reports location and type only (addendum §5). Usher holds no
  bearer authority at any third party (ADR-009).
- **Per-person data.** Never enters git. Client names stay out.
- **Version stamping.** Every check stamps the harness version it assumes
  (FR-3, FR-32).
- **Deterministic checks.** Every coded check is deterministic and tested;
  AI follows progressive codification (§7.2).
- **Signed builds.** Builds are signed and notarized, outbound traffic is
  explicit, and a behavioral profile is published.
- **Independence.** Each sibling (sideshow, marvel, wardrobe, the fleet watcher,
  critic, beadle) works without usher and usher without them.
- **Boundaries by objective, not medium** (operator 2026-11-03). A
  requirement belongs to usher when its purpose is setting up a person's
  harnesses or maintaining their settings. Critic measures, compares and
  judges; the fleet watcher detects faults and recovers from them (addendum §6).
- **Platforms.** macOS first. Linux likely. Windows unknown (open question
  14).
- **Attention budget.** Usher asks only for a confirmation, a consent, or a
  choice it has no default for. Prompts per session and the token budget of
  every in-LLM surface have numbers set in architecture (Q25).
- **Footprint.** The background service is idle between events: it uses the
  operating system's file-change notices, not polling. CPU, memory, disk
  and watched-path counts each have a budget (Q25), tested on a laptop on
  battery, and usher reports its own use. Past the watched-path budget,
  usher names the paths it stopped watching rather than going quiet.
- **Two-way channel.** A person can ask usher at any time what it is doing,
  what it has looked at, and what their consent state is, and change it.
  Anything usher holds about a person, that person can see (FR-39, FR-41).

## 6. Non-Goals

- Enforcing organization policy or clamping a difference; that is device
  management's objective.
- Answering prompts (folder trust, consent) on anyone's behalf.
- Judging which environment is better; ranking people or configurations
  (critic's objective).
- Presenting critic's or the fleet watcher's results. In a fleet report, usher
  supplies the setup and settings side (operator 2026-11-05,
  aae-orc-lp2pg).
- Recovering a running session from a fault (the fleet watcher's objective).
  Usher reports an install-side cause, such as a self-update under a
  running seat, so the person can re-find their seat.
- Reading or optimizing the content of prompts, sessions, or instruction,
  skill or agent files; usher reads those files as metadata only.
- Dashboards as the product; a spend clamp; a proxy or gateway.
- Re-implementing mise, brew, sideshow or vendor installers; managing
  dotfiles.
- Fixing identity gaps such as organization membership (usher reports
  them); refereeing the employment relationship.
- One configuration file understood the same way by every harness.

## 7. MVP Scope

### 7.1 In Scope

All of §4 except the features in §7.2 (FR-1 to FR-52, FR-55, FR-62 to
FR-64, FR-67) for Claude Code, the Claude desktop app with Claude's general
settings, codex and goose. Other harnesses enter through harness
knowledge. Limits for the MVP:

- Apply, restore, catch-up and the organization drop-in (FR-49 to FR-52)
  act on the host usher runs on; remote apply follows (§7.2).
- Profile sources are files, directories and git repositories the person
  names; the signed catalog comes with the central service.
- Recommendations (FR-55) draw on profiles and the person's own records;
  templates come with the team instance.
- Surfaces (FR-62): the command line, the background service, and a thin
  in-harness surface for the newcomer, who starts inside a harness, not a
  terminal. From Claude Code, and from the Claude desktop app where it
  takes MCP servers, the thin surface maps, compares, explains findings and
  proposes and applies changes, each confirmed as on any surface (§5).
  Bootstrap from nothing (FR-46) starts at the command line, since no
  harness is there yet. The app follows.
- The change journal and safe execution (FR-63, FR-64) cover every change
  the MVP makes.
- Hazard checks ship only as their tests pass.
- The light collector only receives reports and answers queries, and each
  light collector serves one affiliation (one organization or team).
- Operator rulings, 2026-11-05: the split of FR-44 to FR-66 between this
  section and §7.2, and configuration writes in the MVP.
- Operator ruling, 2026-11-07: the thin in-harness surface in the MVP.

### 7.2 Out of Scope for MVP

- **Remote apply** (v2): applying a change to another host, once a trust
  model is designed.
- **Activity effects and reporting levels** (v2): FR-53, FR-54.
- **Templates** (v2): FR-56, with the team instance.
- **Full in-LLM surface and the app** (v2): the rest of FR-62, including
  an agent driving installs and clones conversationally.
- **Central service** (v2): FR-60, FR-61, and enrollment (FR-59).
- **Reviewer seating and the key broker** (v2): FR-65, FR-66.
- **Several affiliations per person** (later): associating some
  directories and resources with one organization and others with another.
- **Contribution loop** (v2+): FR-57, FR-58 (brief addendum §A).
- **Marvel policy fragment** (v2+): needs marvel's agreement on the seam.
- **AI at the endpoint and for analysis and query** (later), under
  progressive codification: each case AI handles is logged as a candidate
  for a deterministic check, and once a pattern is understood it becomes a
  coded check with a test, leaving AI for edge and emerging cases. AI stays
  under about 10 USD per user per month in LLM services. No metric tracks
  the AI share.
- **Chat products.** Sequenced, not dropped.

## 8. Success Metrics

All metrics are diagnostic, not gates, except SM-5, which is a structural
test (SOUL §8). The hours and lessons a team loses each week are the
motivation (UJ-2), not a counted metric.

**Primary**
- **SM-1: Newcomer time to a working seat.** From first login to working on
  all supported harnesses the team uses. Today about three weeks (UJ-1); target
  two days. Validates FR-9, FR-11, FR-22, FR-24.
- **SM-2: Lag from a settings cause to a noticed effect.** From a setting
  change, or a newcomer's differing setting, to someone seeing its effect.
  Today about two months (UJ-1); target four days, reachable because FR-14
  dates each settings change against the service operator's daily usage and
  cost. Validates FR-14, FR-16, FR-26, FR-27.

**Secondary**
- **SM-3: Findings on an expert's first map.** Forgotten, undocumented,
  obsolete and conflicting settings. UJ-2 has 10: seven forgotten (four of
  them undocumented), two obsolete, one conflict. Diagnostic, no target.
  Validates FR-1, FR-2, FR-16, FR-43.
- **SM-4: Invalid or stale entries caught before adoption.** Today three
  slipped through (UJ-1); target zero. Validates FR-16, FR-17.
- **SM-5: Harness state changed by observation.** Target zero, held as a
  tested invariant. Validates §5.
- **SM-6: Maintainer upkeep.** Not measured, and no target. It can be
  measured only where a vendor provides its own support for it (Anthropic
  for Claude; OpenAI for codex if and when it does), and is reported there.
- **SM-7: A team outside this fleet reaches a working seat without our
  help.** Observed, not targeted. Validates FR-9, FR-11, FR-22, FR-36.
- **SM-8: First visible win in the first session.** Share of newcomers with
  one harness working in one repo by the end of their first session.
  Diagnostic, no target. Validates FR-46, FR-48.

**Counter-metrics (do not optimize)**
- **SM-C1: Expert setup time.** The expert's effort to capture and review a
  profile must not grow faster than SM-1 shrinks. Counterbalances SM-1.
- **SM-C2: False findings.** Findings that send people chasing valid
  settings. Counterbalances SM-4.
- **SM-C3: Per-person exposure.** No metric singles out a person on thin
  data; effects compare environments. Counterbalances SM-2.

## 9. Open Questions

1. Profile format (F6). Where profiles live is answered for the MVP by
   profile sources (§3); the file format, and how a repo source is laid
   out, are still open, and block FR-19 to FR-23 and FR-51.
2. The usher-marvel seam: a profile fragment, or role files marvel already
   reads (two `--settings` are last-wins).
3. Discovery and consent for a home-wide directory sweep (F10).
4. Harness knowledge: four disputed claims need one test each (addendum
   §1). Several research claims were past their re-check window when
   written, so some re-checks are already due.
5. Does a team of a dozen to two dozen sustain sharing when capture is
   nearly free (F8)?
6. Storage for the light collector (F4), smallest that works.
7. Admissions and organization-required settings (F7), beyond the consent
   to report in FR-39.
8. Signing: who signs profiles and the harness knowledge file, rotation,
   revocation, the ADR-009 test.
9. Component decomposition (F1): usherd and the watcher, MCP surface, CLI,
   app UI, a spider if usherd does not scan, light collector and central
   service.
10. Name and backronym (F5), and the overlap with beadle.
11. Going public: time, license, and a sweep for private repo names, client
    names and per-person data; a front door (README) that carries the
    problem; contributor guidelines; the public homebrew tap formula and
    a default update feed, blocked while the repo is private. Self-update
    through the owning installer and a team-named feed does not wait on
    this (2026-11-09).
12. Which Claude desktop app and account settings usher can read (FR-7
    probe).
13. Brief follow-up: replace "divergence from the baseline" with the
    comparison model in the brief and its addendum (Mary, after Finalize).
14. Windows: the charter names usherd as a mac and windows service; the
    brief says unknown (aae-orc-1vgfk).
15. Usher's absence from the independence matrix, and org-diagram-repo's unratified
    "marvel to usher: Onboards" edge.
16. The objective test (§5) applied to FR-10, FR-13 and FR-26, which share
    a medium with the fleet watcher and critic (addendum §6).
17. The support level of each supported harness, set once the FR-7 probe
    and the disputed claims (Q4) are tested.
18. Associating directories and resources with an organization, for
    several affiliations per person (later).
19. Reporting wire shape (charter F3): metrics-only, OTEL, or levels of
    both (FR-54).
20. Whether usher is a product, a protocol, or a service (July brief).
21. Fleet report (aae-orc-lp2pg). Ruled 2026-11-05: usher supplies the
    setup and settings side only (§6).
22. Seating a reviewer at a run variant (aae-orc-yyqp3). Ruled
    2026-11-05: in scope as setup (FR-65).
23. Key broker at launch (aae-orc-z6y7). Ruled 2026-11-05: in scope as
    brokering under ADR-009 (FR-66).
24. Ingesting the service operator's usage and cost APIs: helps SM-2, but
    an admin key is bearer authority at a third party (ADR-009 custody).
    SM-2's target depends on the answer.
25. Numbers for the attention, token and footprint budgets (§5, FR-62),
    set in architecture.
26. Does ADR-009's 2026-10-19 amendment apply to usher? Ruled 2026-11-09:
    no, it is marvel-specific; FR-66 offers issuing routes only.

## 10. Assumptions Index

- §4.1 FR-7: which desktop-app and account settings are readable locally
  needs a probe.
- §4.5 FR-20: token set and pattern syntax left to architecture.

- §4.10 FR-42: default light collector retention of 90 days.
