# Product Brief: usher

- **Status:** DRAFT. Revised 2026-07-26 (evening) after a thirteen-voice
  design review and the market research landing the same day. The first
  draft came from the elicitation session of 2026-07-26 (bd
  `aae-orc-0mmn.1`); panel: Mary (analyst) with the pit (Bex, Wren, Penny,
  Sloane, Tessa, Scuttle, Marco, Gus). Feeds the PRD step.
- **What changed in this revision:** the Bet is rebuilt around the
  off-baseline contribution loop rather than a reconciliation loop; the
  customer is named; the enterprise-versus-individual framing is replaced
  by individuals-and-organizations; "drift" is retired in favor of
  management of change; v1 is read-only and communicates derived state
  rather than content. Rationale for each is in
  `_kos/findings/finding-002`.
- **Scope note:** this document names no organization and no individual.
  Deployment particulars for the first fleet live in that organization's
  own repositories and are referenced here by scope only. Figures drawn
  from the first fleet's own reporting are given as ratios and roles.
- **Settled inputs respected:** charter B1 (reporting off by default,
  user-enabled) is bedrock and was not reopened; this revision adds a
  testability requirement to it. Charter F6 (usher serves any fleet; the
  repo is not the instance) is developed below.

## Problem

Getting a person productive across today's AI tooling means seating them
into several harnesses (Claude Code, Claude Desktop, ChatGPT, Codex, Crush,
OpenCode) over several backends (enterprise plans, cloud platform
offerings, API keys, local inference), each with its own install, settings
precedence, and failure modes. No product manages install, configuration
change, and honest usage observation across harnesses. Per-vendor tooling
and per-concern tools each hold one piece; the union is whitespace
(prior-art research, Part 3; market research, competitive landscape).

Within a fleet, the tooling that does exist is expert-facing (scripts,
justfiles, skills) and per-machine. What should be on a machine and what is
actually on it are known to different systems, or to nobody.

The economics make this urgent rather than tidy. A handful of engineers can
consume a very large amount of tokens, and therefore money, in a short
time. Their return on that spend varies widely, and the variance turns on
individual expertise in a subject that changes constantly and that none of
them can afford the time to master. If nobody owns the problem, the
organization pays more than it should. And whoever does get assigned to it
finds that the tools to help, or even to see and understand what their
teammates are doing, simply do not exist.

What exists today is two instruments, and neither reaches the levers. The
first is public information and tips: a firehose of advice, changing faster
than anyone can track, unattributable to your own setup. The second is the
bill: authoritative, but late, aggregated, and several steps removed from
any control a person can actually touch. It can tell you that spending rose
last month. It cannot tell you which habit or setting did it, whose, or
what to change. Between an unreadable firehose and a blunt trailing
indicator there is no instrument that sits at the lever, in time, with
knowledge of your actual configuration. That absence is what usher is for.

The mechanism is well understood even where the measurements are private.
Prompt caching dominates the cost of agentic work: a cache write costs
somewhat more than ordinary input while a cache read costs a fraction of
it, so the break-even is a couple of reads per write and sustained reuse is
worth an order of magnitude. Whether a person achieves that reuse depends
on things entirely within their local control: whether the cache is allowed
to expire between turns, whether models get thrashed, whether an image
upload resets the context, how interactions are timed, how the harness is
configured. Two people with identical model access and identical
entitlements can therefore differ by a large multiple in what they pay for
comparable work, and the difference is invisible to both of them.

This brief was revised after checking that mechanism against one fleet's
internal usage reporting, and the result is worth stating precisely because
it is not the result that would have been most convenient. That reporting
is unusually mature: it reconciles to the authoritative billing export to
the cent, separates product from operations spend cleanly, and names cache
discipline explicitly as an ongoing behavior that shapes cost rather than a
contract that reduces it. And it still could not answer the question. Per
person measurement had been in place for days rather than months, so there
was no variance to read. The largest efficiency swing in the observation
window turned out to be a platform quota change that lifted for everyone at
once, which is a shared configuration event and not a difference in
individual habit. Flat-fee subscription seats are invisible to cloud
billing altogether, so the heaviest individual users can be the least
measured ones. And the signals that would settle it, which model was chosen
per session, how long conversations ran, whether reuse actually happened,
how sessions fell relative to cache lifetimes, are not collected anywhere,
because nothing collects them.

That absence is the finding. A team with better AI cost reporting than most
organizations will ever build still cannot see which local behavior moves
its own largest cost lever. No figures from that reporting appear here or
anywhere in this repository; what transfers is the shape of the gap.
Public evidence of the consequences, including named budget overruns and
unactivated vendor controls, is collected in
[market-teams-over-individuals-20260726.md](research/market-teams-over-individuals-20260726.md).

The deeper problem underneath: nobody knows what the right settings are.
The tools change monthly, the bar moves, and expertise is currently
rediscovered privately by each person who bothers. Today that discovery
cost is paid redundantly by every member of every team, invisibly and
unevenly. That is the burden worth carrying collectively.

## The Bet

**Usher turns one person's hard-won configuration knowledge into the
team's baseline, and keeps doing it as the ground shifts.**

Concretely: usher installs and seats a person across their harnesses,
analyzes what their configuration and usage actually look like, and
recommends improvements drawn from what has demonstrably worked for
others on their team. When someone deviates from the baseline in a way
that works better, that deviation is captured, described, assessed, and
offered back to the team with credit. Collection and reporting exist to
serve that loop; they are not the product.

**Priority order, load-bearing:** install, analyze, and recommend are
primary. Collect and report are secondary and instrumental. Analysis and
recommendation happen locally wherever they can; nothing about the value
proposition requires shipping a person's configuration anywhere.

### The off-baseline contribution loop

The differentiator, specified in eight steps by the market research and
requirements-shaped for the PRD:

1. **Detect.** Distinguish *divergence* from *decay*. A deviation from
   baseline is a signal with two possible signs, and every clamp-style
   product in the market treats it as decay only. This distinction is why
   usher is not a compliance tool wearing a helpful hat.
2. **Declare.** The off-baseline person attaches a rationale: why this
   exists, what it improves, what it costs. Mostly auto-generated (the
   observed configuration difference is the body; the human adds intent).
   It has to be cheap enough to do in the flow of work, because the
   shadow-IT literature is unanimous that friction routes around itself.
3. **Describe.** Deviation plus rationale becomes a portable proposal: the
   difference against baseline, its scope, the rationale, and observed
   effects. Usher already holds the effect signals, which is what makes
   the proposal credible rather than anecdotal.
4. **Assess.** Proposals route to the team's review path, competency
   matched. This systematizes the champion figure instead of relying on
   heroics.
5. **Enrich and test.** Validation before ripple: does it apply cleanly
   elsewhere, does it pass the team's templates, what does a comparison of
   old and new baseline show. The assessment is automated; the judgment is
   not (ADR-007: automation computes, surfaces, and drafts; humans decide).
6. **Decide.** Accept into baseline, hold as a sanctioned option, or
   decline with a recorded reason. Baseline changes are human acts.
7. **Ripple.** Accepted deviations distribute through the signed catalog.
   Reconciliation *offers*; it never forces.
8. **Credit.** Contributor attribution rides the proposal. Recognition is
   what sustains participation.

Cross-cutting: consent and scope classification apply at every hop, and a
proposal crossing an organization boundary carries scope names, never
organization content. In-LLM surfaces carry token budgets. The loop
generalizes past configuration to skills, prompts, and workflows, where
distribution ecosystems already exist but graduation does not, anywhere.

### The loop does not emerge on its own

The inner-source literature is blunt about this and the brief should be
too. Participation inequality survives payment: paid contributors
contribute more often but the distribution stays skewed, and a
well-documented longitudinal case at a large engineering company found
that even with an internal contribution marketplace and paid staff, the
program reported too few contributions. Applying the observed power law to
a team of twelve to twenty-five people predicts roughly two to five natural
contributors, possibly as few as one to three. Proximity and shared tooling
do not produce a loop; deliberate design does.

The dominant documented failure mode is return-on-investment misalignment,
expressed by one engineer in that case study as: what benefit do I get from
building something others reuse, when I am the one spending the money.
Adjacent failure modes are time pressure, a steep curve to understand what
already exists, owner resistance to maintenance burden, poor
discoverability, and middle management treating contribution as off-task.
Named enabling conditions are explicit multi-level recognition programs,
lightweight governance rather than heavy process, and a review commitment
fast enough to keep faith with contributors, with two weeks named as the
point past which an unreviewed contribution breaks the loop for that
person.

Three design consequences follow, and they promote parts of the loop from
nice-to-have to load-bearing:

- **Step 2 must stay nearly free, and it is usher's structural advantage.**
  The reason the documented marketplaces failed was authoring cost. Usher
  generates the proposal body from the observed configuration difference
  and asks the human only for intent. If that holds, usher attacks the
  dominant failure mode directly rather than hoping the team is unusually
  generous.
- **Step 4 needs a named reviewer and a review clock**, not a queue. The
  two-week figure is a design constraint on the assess stage.
- **Step 8 is not decoration.** Credit is consistently associated with
  sustained participation across the literature, even though no study
  isolates its effect size. Attribution rides the proposal.

### Why this is the bet and not the collectors

Spend metering, configuration reading, and change detection all have
competitors or credible do-it-yourself answers, and the whitespace around
them is narrowing monthly as vendor admin consoles deepen. Detect,
declare, describe, assess, ripple has no product and no published roadmap
anywhere. The moat is the loop plus the posture, not the collectors.

Two positioning commitments carried forward from the first draft:

1. **Complement now, replace over time.** Usher complements a fleet's
   existing tooling at birth and is designed to make it unnecessary.
   Existing scripts, justfiles, and skills are content in their current
   costume: their logic migrates into usher's catalog rather than being
   absorbed as code. The catalog for the first deployment is harvested
   from tooling that already passes CI, subject to the authorship rule in
   Posture below.
2. **Retirement is the success criterion.** Usher has succeeded when none
   of the parallel surfaces are needed. Retirement is observed, never
   enforced: old surfaces evaporate because nobody reaches for them, not
   because somebody turns them off. This is a diagnostic measure, not a
   gate (SOUL §8). It also functions as a test on usher itself: if the
   fleet keeps maintaining everything it maintained before, usher has not
   earned its focus.

### Size of the bet

The win being underwritten is inside a single team of a dozen to two dozen
people. At the spend levels a small team of heavy agentic users now reaches,
closing even part of the efficiency spread pays for the tool many times
over, so the internal case stands on its own without appeal to a market.
Cross-organization sharing of findings, done in a way that respects user
sovereignty, would be upside rather than premise. Whether a team that size
sustains a contribution loop internally is an open question with real
literature behind it, and it is named as such below rather than assumed.

## Customer and Beneficiaries

**The customer is the fleet engineer, the AI advocate.** They deploy usher,
they carry it, and their professional standing rises when fleet-wide AI
spend falls while output holds or improves. The market research reached the
same figure independently from the adoption side, where the champion is the
engine of every documented standardization event.

**Beneficiaries are the individual developer and the organization.** The
developer gets better results for less money and less lost time. The
organization pays less for more. Neither is the buyer, and neither has to
care how usher works. A developer may install usher themselves or find it
arrived through device management; either way the value has to land without
requiring them to understand usher's internals.

This distinction is the fix for a real defect in the first draft, which
collapsed three hiring parties into one document and optimized for none of
them.

## The Instance Model (F6, revised)

Usher serves **individuals and organizations**. The earlier
enterprise-versus-individual framing was wrong and was hiding an
assumption; an enterprise is simply one kind of organization, and treating
it as a separate product category smuggled in the premise that
organization settings would be locked.

- An individual may belong to **several organizations at once**, and an
  organization may be an employer, a community, a client engagement, or a
  project.
- Organizations have an **admissions gate**, which may be an open door by
  choice. Membership is a real boundary with real semantics, not a label.
- An organization **may require things** of a member's usher. Some settings
  may be organization-locked. That is legitimate and expected.
- Usher's obligation is **transparency, not refereeing**. Before accepting
  an affiliation, the member sees what that organization asks of their
  usher: which features are locked, what telemetry is enabled, what is
  shared and with whom. They accept or decline knowing. If a member has no
  practical choice, that is between them and their employer; usher's job is
  to make sure it is never a surprise (charter B1).
- **Consent is revisable and inspectable.** The member can ask usher what
  it is doing, check their consent state, change it, or withdraw it. This
  is a partnership with a two-way channel, not a one-time gate.

Design consequences, to be designed in the PRD:

- **One daemon, N affiliations.** Scope resolution decides which
  affiliation an observation or action belongs to, the way harnesses
  resolve their own configuration chains.
- **Two artifacts, split by visibility.** A *claim marker* at a scope root
  is discoverable and safe to commit; it names the affiliation in
  scope-name-only terms. A *private binding* is local-only and never
  distributed; it holds where things live, what is enabled, and what routes
  where. This is the fourth appearance of the same house pattern.
- **Affiliations are isolated.** Two affiliations on one machine never see
  each other's data. Classification happens at write time; consent is
  granted per affiliation, not per machine. Isolation needs a stated
  mechanism, not just a stated intent, since a process running as the user
  can reach everything the user owns.
- **Nested claims exist**, and a disagreement between two claim markers
  over one scope is a split-brain needing stated resolution rules.
- **Attribution follows scope, not credential.** The same key may serve two
  scopes in an afternoon, so directory-scope attribution is the only honest
  way to divide usage. This is a requirement, not a moat: it is worth
  building because it is true, and it stops being a differentiator the day
  someone else ships it.
- **Three units, named separately** so the PRD stops conflating them: the
  *attribution* unit is the scope or project, the *benefit* unit is the
  person, and the *decision* unit is the organization.
- **Partition depth is bounded by harness reality.** Correcting the first
  draft: the ratio of user-wide to directory-scoped settings differs per
  harness rather than per setting, and the dominant terminal harness is
  directory-first, with a project-root settings file as the normal
  operating surface rather than the edge case. Chat products are
  user-scoped or cloud-scoped. Some things that shape outcomes, including
  model access and feature availability, are server-side and never
  readable on disk at all; those are observable only by their effects.
  Usher needs a per-harness observability map recording, for each setting
  class, whether it is directory-scoped, user-wide, or server-side, and
  what confounds each observable signal.

## V1 Cut

**Read-only, local-first, install and analyze before collect.**

- **Usher writes nothing to a machine's harness configuration in v1.** No
  remediation, no apply button, no remote configuration application.
  Proposals only. This is what makes usher safe to introduce into a managed
  fleet, and it defers a large class of design problems (interrupted
  profile swaps, restore points, write-audit independence) until there is
  reason to take them on.
- **Communicate derived state, not content.** Usher reports that the cache
  TTL is five minutes, that a given model is configured, that three hooks
  and two servers are registered. It does not ship the files that say so.
  See `docs/research/grok-build-collection-scope-20260726.md` for what the
  alternative costs.
- **Secrets are never read.** Record that authentication is configured and
  by what method, never the material. Masking at output time is not
  sufficient, because ingestion is the exposure.
- **Change records are coarse by design.** Daily milestones on key settings
  are enough to say that a model or a cache setting changed within a few
  days of a date that lines up with a billing change. This is a statistical
  instrument working in broad strokes and averages, not an event log, and
  the coarseness is a privacy and cost feature rather than a limitation.
- **Harness and backend coverage: what the first fleet's continuity tooling
  already covers.** Terminal-side harnesses and the backend matrix,
  including reversible backend profiles and local inference with
  platform-specific hardware support on Apple silicon. Breadth is
  affordable because usher drives that tooling rather than re-hosting what
  a package manager or an organization's own tooling already owns.
- **Chat products are sequenced later**, not deprioritized. Their cohort is
  expected to be the largest in time, but the first users are fluent
  operators. This is a sequencing fact, not a persona decision.
- **Windows and Linux local inference: unknown**, not "later."
- **The recurring reason to return** is a recommendation that is worth
  acting on, pushed rather than waited for. The first fleet's habit of
  checking state is a foothold, not a mechanism, and it does not exist at
  all for a user who never wrote the scripts. Usher must create the
  occasion, not assume it.
- **Candidate first analyze output, for the PRD to confirm or replace: a
  per-session cache efficiency score.** What fraction of a session's turns
  were served from cache rather than paid at full write cost. It needs two
  local signals, inter-turn timing against the backend's cache lifetime and
  whether a turn continued or restarted context, and it is not derivable
  from any billing export at any aggregation. It sits precisely where a
  local agent sits and nowhere else, it speaks directly to the largest cost
  lever, and it turns the piece of expertise nobody has time to acquire
  (knowing the cache lifetime and working within it) into a number a person
  can act on. See `_kos/nodes/frontier/question-local-signal-validity.yaml`
  for the full signal taxonomy and for two reasons billing-side numbers
  mislead.
- **Endpoint-security compatibility is a design constraint from the first
  commit, not a late validation.** A background agent that enumerates
  configuration directories, touches credential-adjacent paths, persists,
  and sends data on a schedule resembles software that endpoint protection
  is trained to quarantine, and the read-only posture reduces but does not
  eliminate that resemblance. Never reading secrets, keeping outbound
  traffic explicit and user-visible, signing and notarizing the binary, and
  publishing the behavioral profile so a security team can allowlist usher
  deliberately are all cheaper than being quarantined once. Validate on one
  managed machine before any breadth.

## Posture Constraints

From charter B1, the prior-art research, and the Grok Build case:

1. **Honest security.** The observed person sees their own data and
   recommendations first. Rollups are aggregate; per-user drill-down is an
   explicit governance decision.
2. **Consent stack, and it must be testable.** Loud first-run disclosure,
   DO_NOT_TRACK honored in personal contexts, a published "we do NOT
   collect" list, content capture off by default. Reporting off by default
   per B1. New requirement: the default-off state is asserted by a test
   that proves no egress occurs, and the enabled state is asserted by a
   test that proves only the published inventory is sent. A control that
   does not gate the code path is worse than no control, because it
   converts a privacy claim into a discoverable lie.
3. **Auditable on purpose.** Usher should be trivially inspectable on the
   wire, and should publish its behavioral profile. Being easy to check is
   a stronger position than asking to be trusted, and the same document
   serves the endpoint-security conversation.
4. **Never sole configuration authority.** Out-of-band edits and
   user-installed tools are signals to reconcile around, not violations.
5. **Advise, do not judge.** Recommendations are proposals; enforcement,
   where an organization wants it, routes through the vendor's own policy
   layer. Recommendations must also declare what they could not observe,
   because a recommendation built on an unseen variable is misleading in a
   way the user cannot detect.
6. **Attention is the scarce resource.** Usher may ask questions and the
   member may ask questions back, but attention gets spent only when the
   person can afford it, and usher returns more than it takes.
7. **Consent events are recorded.** What disclosure was shown, what was
   accepted, when, and whether an enable was member-chosen or
   organization-required. This is what makes B1 defensible under hostile
   examination.

## Non-Goals

- Dashboards as the product. The predecessor's bet was
  telemetry-as-dashboard; usher's product is analysis and recommendation.
- A clamp. Detecting divergence and treating it as decay is the category
  usher is deliberately not in.
- Enforcement through the daemon.
- Configuration writes, remote configuration application, and any
  extend-the-client-without-a-release capability in v1. The knowledge
  index that keeps recommendations current is in scope; the channel that
  applies changes to machines is not, until it has a designed trust model.
- Re-hosting or re-packaging what package managers, vendor installers, or
  an organization's own tooling already own.
- Chat-product onboarding in v1 (sequenced).
- Proxy or gateway spend enforcement (different trust posture).
- One configuration file understood identically by every harness.

## Prior-Art Borrowings: Accept / Adapt / Reject

| Borrowing | Ruling | Note |
|---|---|---|
| Typed protocol, local cache, flush-on-reconnect | **Accept** | Offline tolerance is table stakes |
| Per-identity keypair auth, revocation | **Adapt** | Identity should ride the organization's existing directory or device management where present; keypair is the fallback |
| System-generated push channels, no authoring UI | **Accept** | Scoping discipline |
| Broadcast-all plus naive local filter | **Adapt** | Fleet size and the affiliation model change routing entirely |
| Zero-interruption visibility | **Adapt** | The framing survives; consent is per affiliation and honest-security first |
| "It advises; the rider decides" | **Accept** | Independently required by SOUL §8 |
| Session tiles, dashboard as primary UI | **Reject** | See non-goals |
| Workflow-stage telemetry | **Reject as-is** | Assumes every session runs one workflow shape |

The two predecessor seam findings remain design obligations: usher's
compliance templates are the fidelity channel and must not slip to a late
phase, and every in-LLM surface carries a token budget from day one.

## Open Questions (feed the PRD)

1. **Does a dozen-to-two-dozen-person team sustain a contribution loop
   internally?** Partly answered, and the answer is "not by itself." The
   literature establishes that participation inequality persists among paid
   colleagues and that a small team should expect only a handful of natural
   contributors, so the loop needs deliberate incentive and governance
   design rather than emergence (see "The loop does not emerge on its own"
   above). What no research establishes is a minimum viable size: no
   empirical threshold exists for when an internal contribution loop
   becomes self-sustaining at a given headcount, and the honest position is
   that this is unresolved rather than answered either way. The remaining
   question for usher specifically is whether near-zero authoring cost
   changes the arithmetic, since every documented failure was measured
   against loops where contributing meant writing something.
2. **What is the minimum substrate for aggregate-and-return?** The loop
   requires proposals to route somewhere and ripple back. Candidates run
   from a hosted service through a per-organization deployment to a git
   repository of signed packs distributed by existing tooling. Composition
   argues for the lightest thing that works. Also: what does a member with
   no organization get, given that the analysis is comparative by nature?
3. **Admissions and organization-locked settings.** What may an
   organization require, how is that surfaced before acceptance, and what
   does a member see about what is shared? New question, opened by the
   individuals-and-organizations reframe.
4. Claim-marker format and nested-claim precedence.
5. Enrollment mechanics: how a member's usher finds its organization from
   the claim marker plus private binding, and what happens on a
   device-management install where the software arrives before the person
   has seen the disclosure. Install may be silent; observation may not
   begin before consent.
6. **The per-harness observability map.** Per harness and setting class:
   directory-scoped, user-wide, or server-side, and what confounds each
   observable signal. Replaces the earlier swappability-matrix framing,
   whose write-safety half is deferred with writes. Needs to be a named
   research deliverable, since it is empirical, it goes stale, and every
   estimate downstream of it is fiction until it exists.
7. **Which locally observable signals actually predict cost efficiency.**
   Billing-side reporting can show that a spread exists; it cannot show
   which local signal moves it, because it sits too far from the levers.
   The first probe worth running is whether the named candidates (cache
   expiry, model thrashing, image uploads, interaction timing) explain the
   variance, and that probe requires local instrumentation that does not
   exist yet. Any such probe runs against an organization's own data inside
   that organization's own repositories; results reach this repository only
   as mechanism, never as measurements.
8. Telemetry wire shape (F3). Metrics-first is the direction. Trace-shaped
   telemetry and token accounting likely belong to a sibling tool rather
   than to usher, per composition, optionally deployed alongside.
9. Catalog authorship and signing: who signs, key rotation, revocation,
   and whether a passing-CI provenance is sufficient admission or whether
   entries need a named human author.
10. Whether usher is a product, a protocol, or a service. Deliberately
    deferred; expected to become clear in use.
11. Name and backronym (F5). "Usher" survived; the backronym is undecided.
