---
title: "Input reconciliation: brief and brief addendum against PRD"
created: 2026-11-03
inputs:
  - briefs/brief-usher-2026-11-02/brief.md (cited as B:line)
  - briefs/brief-usher-2026-11-02/addendum.md (cited as BA:line)
  - briefs/brief-usher-2026-11-02/.memlog.md
target:
  - prds/prd-usher-2026-11-03/prd.md (cited as P:line)
  - prds/prd-usher-2026-11-03/addendum.md (cited as PA:line)
  - prds/prd-usher-2026-11-03/.memlog.md (cited as PM:line)
---

# Reconciliation: brief to PRD

Scope: what the brief and its addendum carry that the PRD dropped or
contradicts without a ruling in the PRD memlog. Excluded as ruled: the
many-environments comparison model replacing divergence from a baseline
(PM:16-17), the Claude desktop app and general settings in v1 (PM:23), the
light collector in v1 (PM:31), the retargeted metrics (PM:18-20, PM:25-26),
read-only placement guidance (PM:12), fleet-first calibration (PM:7, PM:13),
metadata-only instruction files (PM:15).

Dispositions: **PRD** (add to prd.md), **PA** (add to the PRD addendum), or
**set aside** (with a reason).

## Ranked summary

| # | Gap | Severity | Disposition |
|---|---|---|---|
| 1 | Permissions friction, the lead problem, has no feature | High | PRD |
| 2 | Re-finding a seat, and "quiet, invisible after install", lost; PRD is pull-only | High | PRD |
| 3 | Instance model (N affiliations, scope) dropped while the v1 collector makes it live | High | PRD + PA |
| 4 | Sibling overlap table dropped; collector meets critic, process table meets the fleet watcher | Medium | PA + PRD Q |
| 5 | Recommendations from the person's own harness records dropped | Medium | PRD or ruling |
| 6 | Three brief success criteria dropped without a ruling | Medium | PRD (SM) or ruling |
| 7 | Prior-art rulings and the template seam obligation dropped | Medium | PA |
| 8 | Non-human seats named as a subject but no FR observes them | Medium | PRD |
| 9 | Credential presence scan is a guardrail with no FR and no scope line | Low | PRD |
| 10 | Smaller drops (see section B) | Low | mixed |

## A. Main gaps

### 1. Permissions friction has no feature

**Input.** The brief names "settings and permissions friction" as the
pressing problem (B:19) and "Permissions friction is the daily cost": sessions
stall on auto mode, approval policies and folder trust; fixes are made by hand
per host (B:42-44). Organization locks are weaker than they look: auto-mode
allow entries combine additively and can override an organization soft deny;
only managed-only keys lock (B:45-47; BA:64-68, "F7's 'what an organization
may require' must be answered per setting class"). Memlog B:7 is the operator
ruling that set this as the lead problem.

**PRD.** The word "permission" appears only as file permissions (P:231) and
the addendum table (PA:17). FR-6 covers folder trust state (P:251-254) and
FR-18 one `autoMode` placement rule (P:406-407). Nothing surfaces where a
person's sessions stall (approval prompts, denials, auto-mode classifier
blocks), and no finding reports that an organization rule the person believes
is a lock is in fact overridable. The Vision and JTBD (P:31-73) frame the
product around onboarding and settings, not permissions.

**Disposition: PRD.** Add a feature or FR group under 4.3, for example:
- FR: permission rules in effect per harness and directory, with provenance,
  including the effective auto-mode lists (`claude auto-mode config`) and
  codex `configRequirements/read`.
- FR: a finding class "lock strength": for each organization rule, whether it
  is enforced (managed-only key) or soft (overridable by additive allow
  entries), per BA:64-68.
- FR or consequence: where the harness records stalls or denials locally,
  count them per environment as an observed effect (ties to gap 5).
Name permissions in the Vision paragraph so the lead problem survives into
epics.

### 2. Re-finding a seat, and the quiet process, are lost

**Input.** Usher helps a person "find their seat ... re-find it when the
ground moves" (B:13-14). The developer "re-finds it after an update or a new
machine" (B:119). Success criterion: "A developer on a new or updated machine
reaches a working seat on all three harnesses without asking anyone"
(B:138-139). Vision: "the quiet process that keeps the house in order
afterwards" (B:214-215). Prior art: "zero-interruption visibility" (BA:111).
Problem: MDM reinstalls an app "and nothing notices" (B:55-56); a self-update
wedges a running seat (B:49-51). Repo CLAUDE.md: "After install, usher should
be pretty much invisible." Exec summary: "It recommends; it does not take
over" (B:17); prior art "it advises; the rider decides" (BA:109).

**PRD.** Every FR is "a person can see" on demand. SM-1 measures only a
newcomer's first login (P:691-693). There is no FR for usher noticing a change
(a harness self-updated, MDM reinstalled an app and dropped profiles, a
setting stopped applying) and telling the person. FR-20a implies periodic
snapshots (P:443-446) but does not say usher runs unattended or what it does
when a snapshot differs. "Attention budget: usher asks only when it has to"
(P:637-638) is the only trace of the attention stance; "recommends, does not
take over" survives only as read-only posture, not as a voice.

**Disposition: PRD.**
- Add an FR (4.2 or a new 4.10 "Watch"): usher observes on a schedule without
  the person's attention and raises a notice only when a seat that worked
  stops working or a declared source moves (self-update, MDM reinstall,
  setting no longer in effect, tool behind source). Consequence: no notice
  for a difference that does not change a working seat.
- Extend SM-1, or add SM-1b: time to a working seat after a new machine or a
  harness update, target "without asking anyone" (B:138-139).
- Add to §5 a tone line: usher proposes and explains; it never nags,
  blocks, or presents a difference as a fault. This is the qualitative
  carry of B:17 and BA:109.

### 3. The instance model is dropped while the collector makes it live

**Input.** "A person may belong to several organizations, each with its own
consent" (B:128-129). BA:49-62: one daemon, N affiliations; scope resolution
assigns each observation; a committable claim marker at a scope root and a
private local binding; affiliations isolated by a stated mechanism; nested
claims need rules; attribution follows scope, not credential; three units
(attribution, benefit, decision).

**PRD.** None of this appears. Q1 and Q7 name F6 and F7 (P:724, P:734-736)
but no FR or addendum section carries the model. With the light collector in
v1 (FR-30 to FR-35), the question is no longer v2: a host whose person
belongs to two teams must route each observation to the right collector, and
FR-32 consent is per host (P:504-514), not per affiliation.

**Disposition: PRD + PA.** Copy BA:53-62 into the PRD addendum as a section.
In the PRD, change FR-30 and FR-32 so opt-in and consent are per host per
affiliation, and add a consequence: an observation under scope A is never
sent to scope B's collector. If the operator wants v1 to be single
affiliation, record that as a ruling and an assumption instead.

### 4. Sibling overlaps lost

**Input.** B:205-209 and the table at BA:72-80: sideshow (install and
five-layer doctor; signed packs overlap the ripple substrate), marvel (seat
settings, measurement fabric), wardrobe (`constraints.permissions` and
`policy.sh` render), the fleet watcher (read-only session watcher), critic (fleet
aggregation), beadle (name), aq/ax (install and update posture belongs to
usher, not a self-updater).

**PRD.** §5 keeps the independence sentence and the sibling names (P:632-634)
but drops what each overlap is. Two overlaps became sharper after rulings:
the v1 light collector's cross-host query (FR-34) is what critic is chartered
for ("fleet aggregation"), and the process table (FR-8) is close to
the fleet watcher's session watcher. Install health (FR-7) overlaps sideshow's
doctor, and wardrobe renders permissions, which matters once gap 1 lands.

**Disposition: PA + PRD.** Copy the BA:72-80 table into the PRD addendum. Add
an open question: is the light collector usher's or critic's, and is the
process table usher's or the fleet watcher's? Add to FR-7 that usher reads
sideshow's doctor output rather than re-checking packs, per "drives, does not
re-host" (B:89-90).

### 5. Recommendations from the person's own harness records dropped

**Input.** v1 Solution includes "Recommendations. Proposals drawn from the
team profile and from what the person's own harness recorded (permission
denials, for example). Each declares what it could not observe" (B:91-93).

**PRD.** Recommendations appear only as out of scope for the collector and as
v2 central-service work (P:480, P:542, P:676). The many-environments ruling
(PM:16-17) explains dropping baseline-driven recommendations ("people
judge"), but not the second source: proposals drawn from the person's own
local records, which need no baseline. No ruling covers that.

**Disposition: PRD, or a logged ruling.** Either add an FR under 4.5: from
local harness records (denials, retired keys in use), usher proposes an entry
and its placement, stating what it could not observe; or record an operator
ruling that v1 has no recommendations of any kind and FR-18 placement is the
whole of "it recommends".

### 6. Brief success criteria dropped without a ruling

**Input.** B:136-147:
- (a) "A team outside this fleet installs usher and reaches a working seat on
  its own harnesses without our help" (B:136-137; added by validation,
  memlog B:24).
- (b) "At least one team setting per quarter arrives through the divergence
  path rather than by hand" (B:143-144).
- (c) "Parallel surfaces (per-host scripts, hand-edited settings, doctor
  checks) are retired because nobody reaches for them. Retirement is
  observed, never enforced" (B:145-147).
- (d) the new or updated machine criterion (B:138-139), covered under gap 2.

**PRD.** §8 has none of (a) to (c). (a) plausibly falls under fleet-first
calibration (PM:7), but no ruling says the criterion is deferred. (b) loses
its vocabulary to the comparison model but its intent (a team setting that
came from a person's environment, not by hand) maps directly onto UJ-2's
capture path. (c) is the measurable form of "invisible after install" and is
not covered by any SM.

**Disposition.** (a) set aside to the going-public decision (Q11), with one
line in §8 saying so. (b) PRD: SM, diagnostic, "team settings per quarter
that arrived through a captured profile rather than by hand". (c) PRD: SM,
diagnostic, observed not enforced, as the brief wrote it.

### 7. Prior-art rulings and the template seam obligation dropped

**Input.** BA:106-117. Accept: typed protocol with local cache and
flush-on-reconnect; system-generated push channels; "it advises; the rider
decides". Adapt: keypair auth preferring the organization's directory or
device management; broadcast routing; zero-interruption visibility. Reject:
session tiles as primary UI; workflow-stage telemetry as-is. Seam
obligations: "compliance templates are the fidelity channel and must not
slip", and every in-LLM surface carries a token budget.

**PRD.** Only the token budget survives (P:637-638). With the collector in
v1, local cache and flush-on-reconnect bears on FR-30 to FR-32 (what happens
off-network), and "prefer the organization's directory or MDM" bears on FR-33
host identity. Templates are pushed to v2 (P:676) with no statement that the
v1 inventory format keeps the template seam open, which is what "must not
slip" asks.

**Disposition: PA.** Copy BA:106-117 into the PRD addendum. In the PRD, add a
consequence to FR-31 that the inventory format is the input templates will
run on in v2, and a consequence to FR-30 that a host buffers locally and
flushes on reconnect. Add to FR-33 a preference for organization identity or
MDM-provisioned keys where present.

### 8. Non-human seats have no FR

**Input.** "Agent sessions launched by marvel receive settings through
marvel's per-session projection. Usher reports on them" (B:122-124). Open
question 2 cites finding-180: two `--settings` are last-wins (B:225-226).
Brief Problem: "a long-lived session holds the environment it launched with"
(B:39-40).

**PRD.** JTBD names the subject (P:72-73). FR-8 reports process start time,
version and directory (P:293-301) but not launch flags, and the flag layer is
in the glossary (P:149) with no FR that reads it. A marvel seat's settings
arrive by `--settings`, so without argv the seat's environment cannot be
mapped. FR-8 flags an old binary but not a session running settings that have
since changed on disk.

**Disposition: PRD.** Add to FR-8: the flag layer is read from each running
process's launch arguments (paths, not content), and a session whose
launch-time settings differ from the current resolved state is flagged, with
the hot-reload exception noted per harness (PA:17).

### 9. Credential presence scan has no FR

**Input.** "Usher may scan for credential-shaped material on disk and report
its location and type ... the scan is off by default" (B:182-186; memlog
B:10, an operator ruling reconciling aae-orc-z6y7 W2).

**PRD.** Present only as a guardrail (P:619-621). No FR, and §7.1 does not
say whether the scan is in v1. FR-15 and FR-31 exclude credential values but
nothing implements detection.

**Disposition: PRD.** Add an FR (4.2 or 4.7): opt-in scan reporting location
and type, with consequences that no value is retained or sent and the scan is
off by default. Or list it in §7.2 as deferred, so the ruling is not lost.

## B. Smaller drops

| Input | PRD | Disposition |
|---|---|---|
| Developer "may install usher or receive it through device management; either way the value lands without understanding usher's internals" (B:119-121) | No MDM-delivered install path; no "without internals" stance | PRD: one line in JTBD and a consequence on FR-9 or §5 |
| Differentiation: "one view of settings across several harnesses and directories, and a path that turns one person's better setting into the team's" (B:101-104); "keeps usher out of the compliance category" (B:105-106) | Non-compliance kept (P:77-78); the cross-harness claim and the "collectors are an afternoon" framing are gone | PA: two lines, so the architect does not over-invest in collectors |
| Vision: the loop "extends past settings to skills, prompts and workflows" (B:216-218) | Absent | PA under later phases; set aside from v1 |
| Contribution loop depth: nearly-free declaring, two-week review clock, credit, F8 sub-questions (BA:15-47) | Loop is v2+ (P:677-678); Q5 keeps only the headline | PA: carry the F8 sub-questions (BA:43-47) beside Q5. PRD: add author credit to FR-15 consequences, since capture already records source environment |
| Q6 "aggregate-and-return" (B:233) | PRD Q6 retargets to the v2 central service (P:733) while FR-35's NFR cites Q6 for v1 collector storage (P:548-549) | PRD: split Q6 into v1 collector storage and v2 substrate; state whether results return to members or only to the expert |
| Q8 "Catalog and profile signing" (B:237-238) | "Profile signing" only (P:737); FR-25 adds a signed harness knowledge file with no signer named | PRD: Q8 covers knowledge-file signing too |
| F9: context-file size over time is a further signal (BA:123) | FR-4 reports size; FR-20a snapshots maps; no link | PRD: one consequence on FR-20a that instruction-file size is in the snapshot |
| Token audit: "use its method, not its figures" (BA:95-96) | Absent | PA, one line under observed effects |
| "Chat products in v1" out (B:203) | Desktop app now in v1 (ruled); §7.2 still lists "Chat products. Sequenced" (P:682) | PRD: reword to "chat conversations and content" to match P:83-85, so the two lines do not read as a contradiction |
| Codex `configRequirements/read` returns admin constraints (BA:159) | In PA:18 only; no FR uses it | PRD: cite it under gap 1's lock-strength FR |
| Brief addendum §G "Two layers of staleness" keyed to install health versions (BA:174-176) | Carried in FR-26 and §5 | No action |

## C. Checked and covered

Read-only posture, observing does not change the harness (strengthened as
SM-5), derived state, per-host credentials, reporting off with egress tests,
self-hosted collector, signed builds, drift's narrow meaning, version
stamping, cache score with unpriced and throttled handling, the disputed
claims and their tests, the approval_policy retirement, MDM overlap
reporting, which hosts carry entry X, open questions 1 to 5, 7, 9 to 11,
independence matrix (Q15), Windows (Q14).

Noted in passing, outside this reconciliation's scope: §4.9 precedes §4.8 in
prd.md (P:476, P:551).
