---
id: SPEC-usher
companions:
  - capability-scope.md
  - ../../planning-artifacts/architecture/architecture-usher-2026-11-08/ARCHITECTURE-SPINE.md
  - ../../planning-artifacts/prds/prd-usher-2026-11-03/prd.md
  - ../../planning-artifacts/prds/prd-usher-2026-11-03/addendum.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Each capability cites its PRD FRs; the FR consequences in `prd.md` are the detailed tests, and the `AD-n` rules in the spine bind how. Terms are used as defined in PRD §3.

# usher

## Why

A pain to solve and a vision to realize. People new to AI on a team lose weeks finding their seat: they cannot tell where a setting goes, which recommended settings are stale for their versions, or when a setting has quietly stopped applying (PRD UJ-1: about three weeks to get going, two months before anyone notices worse cache reuse). Experts who tuned their harnesses over months cannot hand that work on without dictating files, paths and JSON from memory (UJ-2). Usher sets up a person's AI harnesses and maintains their settings from then on, so an expert's effort transfers to the next person precisely and repeatably, and people decide what is better from facts usher reports.

## Capabilities

Release for each capability, and the spine decisions that govern it, are in `capability-scope.md`.

- **CAP-1**
  - **intent:** A person can list every settings source on a host for each supported harness and, per launch context, see each setting in effect with its provenance, version stamp and blind spots (FR-43, FR-1 to FR-8).
  - **success:** For codex, values and layers match `config/read` for that directory; every declared-but-not-applied case in addendum §1.1 is reported as not in effect; every map carries a version stamp and the list of what it could not observe.
- **CAP-2**
  - **intent:** A person can see each harness and tool's install state, source, version, whether it is behind, and which running sessions predate the binary on disk, and install or update through the installer that owns it (FR-9 to FR-12, FR-44, FR-45).
  - **success:** Each failure mode in addendum §3 is caught; an errored check reads as an error, never as current; no install or update runs before the person sees the exact command and confirms.
- **CAP-3**
  - **intent:** A newcomer can go from nothing to a working seat in one guided run that stops and resumes, attaching supporting services, method content and the repositories they need (FR-46 to FR-48).
  - **success:** Rerunning a finished step changes nothing; every failure names the step, its error and a next action; the first session ends with one harness working in one repo, launched by the person.
- **CAP-4**
  - **intent:** A person can sit down at an agent-grown run variant with a working seat to review it (FR-65).
  - **success:** The variant's repo, declared tools and a named profile are set up, each on confirmation; the variant's own tool declaration runs only after it is shown and confirmed.
- **CAP-5**
  - **intent:** A person can launch a harness with a short-lived key their own vault or cloud login issues for that launch (FR-66).
  - **success:** The key reaches the harness in memory and is never written to disk, logged, journaled or reported; usher never reads a stored long-lived key, and the person sees which harnesses have a brokered route.
- **CAP-6**
  - **intent:** A person can remove usher and choose to keep or roll back the changes it made (FR-67).
  - **success:** After removal no usher process, login item, scheduled job or OTLP endpoint remains, checked by a test; the journal and backups are kept unless the person asks otherwise.
- **CAP-7**
  - **intent:** A person is told when the ground moves under a seat (a self-update, a stale running session, a device-management reinstall, a changed setting) and can see when each setting changed (FR-13, FR-14).
  - **success:** Each notice names the environment and the before and after values; dated snapshots line up with the service operator's daily usage; the watcher writes only usher's own state.
- **CAP-8**
  - **intent:** A person can compare any two environments, or an environment and a profile, in either direction, and see findings on each (FR-15 to FR-18).
  - **success:** Swapping sides swaps only missing and extra; a retired key is flagged from the release that retired it; an entry outside its version range is marked before placement; cross-host answers list the hosts they could not see.
- **CAP-9**
  - **intent:** An expert can capture an environment into a portable, reviewed, annotated profile, and a newcomer can load it and adopt entries (FR-19 to FR-23).
  - **success:** Capture holds no credential value or file content; a literal path naming the author's home is flagged; loading changes nothing; an adopted entry that later changes is reported as drift.
- **CAP-10**
  - **intent:** A person can see exactly where an entry goes for their harness and version, have usher apply it with write safety, verify it, restore a seat, catch up on followed profiles, and produce an organization drop-in (FR-24, FR-25, FR-49 to FR-52).
  - **success:** No entry is placed in a layer the harness ignores for it; a higher-layer override is named before applying; a re-map reports in effect, overridden or not loaded; a hand-set value is shown as a conflict, never overwritten; the drop-in is written to a file, never deployed.
- **CAP-11**
  - **intent:** A person can see observed effects (cache reuse per session, and activity) beside their settings and comparisons (FR-26, FR-27, FR-53).
  - **success:** An unpriced model is reported, not dropped; throttled calls are counted apart; effects from different backends are never combined in one figure.
- **CAP-12**
  - **intent:** A person can save any map, comparison or effect as a local report, and reporting off the host stays off until they turn it on, at a level they choose (FR-28, FR-29, FR-54).
  - **success:** A test proves no reporting egress in the default state (only the listed version-check endpoints are contacted) and only reported fields in the enabled state; saved reports validate against the published schema.
- **CAP-13**
  - **intent:** A maintainer can keep harness knowledge current, with a reference and verification date per item, re-check dates by class and by release, re-runnable probes and release helpers, shipped without a client release (FR-30 to FR-36).
  - **success:** An item without reference or date fails the structural check; each supported harness has a probe-verified stale-key entry; helpers and probes run under a scratch home and only propose.
- **CAP-14**
  - **intent:** A team can run a self-hosted light collector that receives reported fields from hosts that opt in and answers queries across them (FR-37 to FR-42).
  - **success:** Any field outside the reported fields is rejected; no report leaves a host before consent for that collector; one host's credential is revoked without affecting another; data past retention is deleted with a record kept.
- **CAP-15**
  - **intent:** A person receives recommendations drawn from followed profiles, applicable templates and their own records, and a team can declare template checks (FR-55, FR-56).
  - **success:** Each recommendation states its source, reason, assumed versions and backend, and what usher could not observe; an allow entry is marked as a loosening; a template never blocks or changes a setting by itself.
- **CAP-16**
  - **intent:** A person can turn a difference in their environment into a proposal a named reviewer decides, and an accepted change reaches the team with credit (FR-57, FR-58).
  - **success:** The proposal body is drafted from the observed difference; an accepted change becomes a new profile version offered through catch-up, never forced; a proposal past its review clock is surfaced.
- **CAP-17**
  - **intent:** A person's usher can enroll with its team's instance, which distributes a signed catalog of profiles, templates and recommendations and an options index (FR-59 to FR-61).
  - **success:** The client verifies signatures before using any item and ignores and reports an item it cannot verify; team customization never overrides the personal layer.
- **CAP-18**
  - **intent:** A person can use usher from the command line, from inside their harness over MCP, and from an app, while a background service keeps the inventory and watcher running (FR-62).
  - **success:** An in-harness agent applies only under the same confirmation as every other surface, and the journal records the confirming surface; every in-harness surface stays within its token budget.
- **CAP-19**
  - **intent:** A person can see every change usher made on their host and undo any one, or a run, and can rely on a change touching only what its dry run listed (FR-63, FR-64).
  - **success:** Every failure case in addendum §7 has a test that injects it and passes; the journal entry is written before each change starts.

## Constraints

- Every change is shown first and made only on confirmation. An agent working for the person confirms as the person can (operator ruling, 2026-11-08); the journal records the confirming surface; the background service never applies (AD-4).
- Observing changes no harness state; harness calls that can reset, fix, update or purge are never made. Tested invariant, may gate (PRD §5, SM-5, AD-9).
- Nothing leaves the host unless the person opts in per collector after seeing what is sent; `DO_NOT_TRACK` keeps reporting off; the off switch gates the code path (FR-29, FR-39, AD-14).
- Derived state, never content: session records are parsed by named fields only; credential values and folder-trust records are never read or written; a credential is recorded as present (PRD §5, AD-8).
- Usher never writes a managed or device-management layer, an entry the person marked personal, or a file holding credentials; it is never the sole configuration authority (FR-49, FR-64).
- Precedence is the harness's. Facts about a harness live in signed, referenced harness knowledge that updates without a client release (FR-1, FR-30, FR-31, AD-6, AD-12).
- Local observation runs from install, limited to harness-knowledge locations until the person confirms directories; no home-wide sweep (PRD §5, AD-10).
- The footprint, attention and token budgets ratified for PRD Q25 (spine, Budgets) are tests on a laptop on battery; the background service is idle between OS file-change notices.
- Boundary by objective: usher's is setup and maintain settings; critic measures and judges, stagekeeper detects and recovers (PRD §5, addendum §6).
- Every sibling works without usher and usher without them. Usher holds no bearer authority at a third party (ADR-009). Per-person data never enters git; client names stay out.
- Coded checks are deterministic and tested; any AI use follows progressive codification under about 10 USD per user per month (PRD §7.2).
- macOS first; Linux likely; Windows open (Q14). Builds are signed and notarized.

## Non-goals

- Enforcing organization policy or clamping a difference (device management's objective).
- Answering prompts such as folder trust or consent on anyone's behalf.
- Judging which environment is better, or ranking people or configurations (critic's objective); presenting critic's or stagekeeper's results.
- Recovering a running session from a fault (stagekeeper's objective).
- Reading or optimizing the content of prompts, sessions, or instruction, skill or agent files.
- Dashboards as the product; a spend clamp; a proxy or gateway.
- Re-implementing mise, brew, sideshow or vendor installers; managing dotfiles.
- Fixing identity gaps such as organization membership; refereeing the employment relationship.
- One configuration file understood the same way by every harness.
- Managing chat conversations.

## Success signal

- A newcomer reaches a working seat on every supported harness the team uses in two days instead of about three weeks (SM-1), with zero invalid or stale entries adopted (SM-4).
- A settings change is seen against its effect within four days instead of about two months (SM-2), while observation changes harness state zero times, held as a tested invariant (SM-5).

## Assumptions

- The output folder is `_bmad-output`, since `_bmad/config.toml` is absent; the slug `usher` is inherited from the PRD folder.
- The PRD's three assumptions are settled upstream: the FR-7 probe ran on 2026-11-08 (desktop-app files mapped, account settings not stored locally); path tokens are fixed in the spine's conventions; 90-day collector retention is ratified in AD-13.
- SM-2's four-day target assumes the dated settings history alone (FR-14) is enough, without ingesting the service operator's usage APIs (Q24).

## Open Questions

- Q2: does marvel read usher profile fragments or its own role files? (AD-11 keeps either open.)
- Q4 remainder: codex device-management precedence and the goose `GOOSE_MODE` default, untested until a managed codex host and a goose host are available.
- Q5: does a team of a dozen to two dozen sustain sharing when capture is nearly free?
- Q7: admissions and organization-required settings beyond the consent in FR-39.
- Q8: who signs harness knowledge and profiles, and how keys rotate and are revoked.
- Q10: the name, and the overlap with beadle.
- Q11: going public (timing, license, disclosure sweep, README, the public homebrew tap and a default update feed). Self-update through the owning installer does not wait on it.
- Q13: replace "divergence from the baseline" with the comparison model in the brief.
- Q14: Windows.
- Q15: usher's place in the independence matrix, and marquee's "marvel to usher: Onboards" edge.
- Q16: the objective test applied to FR-10, FR-13 and FR-26.
- Q17: the support level of each supported harness.
- Q19: the reporting wire shape beyond the MVP report schema (metrics, OTEL, or levels).
- Q20: is usher a product, a protocol, or a service?
- Q24: may usher ingest the service operator's usage and cost APIs, given an admin key is bearer authority at a third party?
