# finding-002: thirteen-voice design review, and what happened when the bet met the data

- **Date:** 2026-07-26
- **Probe:** stress-test the product brief before a second improvement pass.
  Four techniques (job-to-be-done, assumption reversal, one-feature-only,
  question-storming) argued by a cast of thirteen, then checked against the
  market research that landed the same day and against one fleet's internal
  usage reporting.
- **Outputs:** product brief revised (Bet rebuilt, customer named, instance
  model reframed), `docs/research/grok-build-collection-scope-20260726.md`,
  `docs/research/README.md` and its promotion rule, three frontier nodes,
  one bedrock amendment.
- **Scope:** no organization or individual is named here. Measurements from
  the fleet's reporting are deliberately absent; only mechanism is recorded.

## What the review changed

**The customer was unnamed and three parties were collapsed.** The brief
served an individual developer, an organization lead, and a fleet engineer
in one document and optimized for none. Resolved: the customer is the fleet
engineer, the AI advocate who deploys usher and whose standing rises when
fleet spend falls while output holds. The developer and the organization are
beneficiaries, not buyers. The market research had reached the same figure
independently from the adoption side, where a champion drives every
documented standardization event.

**Two words were each carrying two products.** "Drift" meant both deviation
from a personal last-known-good and deviation from a fleet baseline, which
need opposite architectures. Retired in favor of management of change: the
bar moves constantly, nobody knows where it should be, so there is no fixed
baseline to drift from. "Loop" meant both a reconciliation loop (desired
versus observed, top-down) and the off-baseline contribution loop
(individual deviation becoming team baseline, bottom-up). The market
research classifies the first as table stakes and the second as the moat.
The brief's Bet had named the wrong one.

**Enterprise versus individual was a false taxonomy hiding an assumption.**
The real axis is individuals and organizations; an enterprise is one kind of
organization. The framing had smuggled in the premise that organization
settings would be locked. An individual may belong to several organizations,
organizations have admissions gates that may be open doors by choice, and an
organization may legitimately require things of a member's usher. Usher's
obligation is transparency before acceptance, not refereeing the
employment relationship. Opened as
`question-org-admissions-and-locked-settings`.

**Consent is a partnership, not a gate.** An early proposal to let usher
ask exactly one class of question was rejected: usher may ask more, the
member may ask back, and consent is inspectable, revisable, and
withdrawable. The real constraint is attention economics rather than a
question budget. Spend attention only when the person can afford it, and
return more than you take.

## Where the review was wrong

Recording these because the review's errors were more instructive than its
hits.

**An invented confounder.** The reviewer argued that server-side feature
gates, being invisible on disk, would confound any recommendation: if one
person's better efficiency came from an entitlement rather than a setting,
usher would recommend the wrong thing and spend its credibility. This was
risk telescoping. Access tiers are managed in groups, so entitlements are
constant within a comparison cohort. The variance lives in local
configuration and habit. The residual requirement is much cheaper than the
alarm suggested: record which service or backend is in use, and compare
within cohorts.

**A cost objection with the inference backwards.** The build-cost voice
priced the per-harness observability catalog as expensive and
staleness-prone, and concluded it should be descoped. The pricing was right
and the conclusion was wrong. That work is currently done redundantly by
every member of every team, invisibly and unevenly. Its expense is the
argument for carrying it collectively, which is the product, not an argument
for cutting it.

**A near-unanimous vote to delete the product.** Four voices independently
moved to delete the dynamic-options index as remote execution wearing a
features hat. The security analysis was correct about the apply path and
wrong about the deletion, because it merged two capabilities: shipping
knowledge, and shipping action. Every attack in the case required the apply
path, which a read-only v1 removes by construction. Shipping knowledge is
arguably the product, since the brief's own thesis is that nobody has
settings for new things and the bar moves monthly. Resolution: keep the
options knowledge index (read-only, signed, human-confirmed at point of
use); rule out remote configuration application until it has a designed
trust model.

**A panel arguing table stakes.** The whole four-round review, and its
one-feature round in particular, argued about collectors: install, change
visibility, local checks, spend attribution. The market research committed
the same day had already classified all of those as table stakes and named
the contribution loop as the only uncontested differentiator. The loop came
up almost nowhere in four rounds. Cause below.

## The process failure worth keeping

Hours after the market research was written, a design session ran against
the charter and the brief without it, and thirteen independent reviewers
each read the same two documents and none read the research. The whole
four-round review therefore argued the layer that research had already
classified as table stakes.

**The first diagnosis was wrong and is recorded here because the correction
matters more than the original.** I attributed the miss to the research
being filed in the wrong place, in `_bmad-output/planning-artifacts/
research/` rather than in `docs/research/`, and "fixed" it by moving the
file and declaring `docs/research/` canonical. That is backwards.
`_bmad-output/planning-artifacts/research/` is precisely where the
`bmad-domain-research` workflow writes, per its own output contract.
The research was correctly filed. `docs/research/` is the ad-hoc location,
and the move put a workflow artifact somewhere its workflow will not look
for it. The files are staying where they now are, by decision, but the rule
that justified moving them does not stand.

The real cause has nothing to do with filing. I listed the directory
containing the research during orient and did not open the file. Then I
briefed thirteen agents with two documents and omitted the third. Both are
reading and briefing failures by the agent, not location failures by the
artifact. The load-bearing instance of the orchestrator's F19 and F25 is
the same either way (nothing actively surfaced the research at the moment
it was needed), but the remedy is a briefing discipline: enumerate what
exists, open what is enumerated, and pass the full set to every spawned
reviewer. Reorganizing directories does not address it and cost a
correctly-filed document its home.

Second-order lesson, recorded because it was the more expensive error: from
the same wrong diagnosis I went on to gitignore `_bmad-output/` entirely.
`bmm/config.yaml` resolves `planning_artifacts` to
`{project-root}/_bmad-output/planning-artifacts`, so that directory is the
configured destination for briefs, PRDs, architecture documents, epics,
stories, and research. The change would have silently untracked every one
of them from that point forward. Reverted in `9fb7cd6`. The generalization
from "one file looks misplaced" to "untrack the whole tree" was never in
scope and was never asked for.

The mechanism of that error is worth naming, because it will recur in
another costume. The orchestrator does gitignore `_bmad-output/`, and I
took that as license. But the orchestrator's ignore is a deliberate guard
added the day before in session-065, for a reason that applies only there:
the orc root is where a session-close `git add -A` habit meets untracked
sensitive material, and the orc is a coordinator that holds no planning
artifacts of its own. In a subrepo the same directory holds the
deliverables. So the rule is not "ignore `_bmad-output/`" or "never ignore
it"; it is that a guard's justification does not travel with its pattern.
Copying a fleet-level ignore into a repo where the directory is the product
inverts its purpose. I also, before learning the guard was deliberate,
speculated in conversation that the orc's ignore might be the same mistake
at larger scale. It was not, and the speculation was unfounded.

## The evidence check, and a claim that did not survive it

The brief was revised to claim that one fleet's reporting showed
multiple-fold per-person spreads in cache amortization and effective cost
per token among people with identical access. That claim was withdrawn
before it shipped, and the reasons matter more than the claim did:

- Per-person partition data had been in place for days, not months. There
  was no variance to read, only short samples at different workloads.
- The largest efficiency swing in the observation window was a platform
  quota change that lifted for everyone simultaneously. That is a shared
  configuration event, and individual-habit variance would look nothing
  like it.
- Flat-fee subscription seats are invisible to cloud billing entirely, so
  the heaviest individual users can be the least measured.
- The fleet-level caching-savings figure nearly quoted is disclaimed in its
  own source as an order-of-magnitude list-price indicator, inflated in the
  latest run by a mid-run pricing-table update.

**What replaced it is stronger.** A team whose AI cost reporting reconciles
to the authoritative export to the cent, separates product from operations
spend cleanly, and explicitly names cache discipline as an ongoing behavior
rather than a contract, still cannot see which local behavior moves its own
largest cost lever. The deciding signals, which model was chosen per
session, how long conversations ran, whether reuse actually happened, how
sessions fell relative to cache lifetimes, are collected nowhere because
nothing collects them. The absence is the finding, and it confirms the
two-instrument gap from the inside: public advice changes faster than anyone
can track, and the bill is authoritative but late, aggregated, and several
steps from any lever a person can touch.

Consequence for the roadmap: those four signals are usher's collector
specification, derived empirically rather than guessed. And since per-person
partitioning now exists in that fleet, a four-to-six week read is the
minimum viable window to test whether local signals predict efficiency.
Opened as `question-local-signal-validity`.

**Method note, general.** The privacy scrub and the correctness scrub
removed the same sentences. Measurements borrowed from a private corpus were
both the thing that could not be published and the thing that could not be
trusted, because a number lifted from someone else's instrument arrives
without its caveats. The rule adopted: mechanisms and ratios-of-mechanism
travel between repositories; measurements stay where they were taken.

## What the contribution literature says about the bet

The loop is the differentiator, so its sustainability is load-bearing.

Participation inequality survives payment. Paid contributors contribute more
often but the distribution stays skewed, and a documented longitudinal case
at a large engineering company found that paid staff with an internal
contribution marketplace still produced too few contributions. Applying the
observed power law to a team of twelve to twenty-five predicts roughly two
to five natural contributors, possibly one to three. No empirical
minimum-viable-size threshold exists anywhere in the literature; that is
genuinely unresolved rather than answered.

The dominant failure mode is return-on-investment misalignment: the
contributor bears the cost of making something reusable while others
capture the benefit. Then time pressure, learning curve, owner resistance,
discoverability, and management treating contribution as off-task. Named
enablers are multi-level recognition programs, lightweight governance, and a
review clock, with two weeks named as the point past which an unreviewed
contribution breaks the loop for that contributor.

**Usher's structural advantage, and it is specific.** Every documented
failure was measured against loops where contributing meant authoring
something. Usher generates the proposal body from the observed configuration
difference and asks the human only for intent, so authoring cost approaches
zero. That attacks the dominant failure mode directly instead of hoping for
unusual generosity. Whether near-zero authoring cost changes the small-team
arithmetic is the open question, and it is a better question than the one
the review started with.

Design consequences now in the brief: the declare step must stay nearly
free, the assess step needs a named reviewer and a two-week clock rather
than a queue, and credit is load-bearing rather than decorative.

## Node changes

- `question-org-admissions-and-locked-settings`: new frontier.
- `question-contribution-loop-sustainability`: new frontier.
- `question-local-signal-validity`: new frontier.
- `val-reporting-off-by-default`: amended. B1 gains a testability
  requirement. A control that does not gate the code path is worse than no
  control, because it converts a privacy claim into a discoverable lie.
  Grounded in the Grok Build case rather than in principle.
