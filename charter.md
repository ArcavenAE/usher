# usher Charter

Last updated: 2026-07-27 (design review of the product brief: bet moved to the
off-baseline contribution loop; F7-F9 opened; B1 amended with a testability
requirement. See `_kos/findings/finding-002` and `docs/product-brief-20260726.md`.)

## Bedrock

### B1: Reporting is off by default — enabling it is the user's act

Telemetry/central reporting ships disabled. Usher's contract is a
developer-user compact: the tool exists to help the user, so the user
— and only the user — turns reporting on. If an employer requires an
employee to enable it, that arrangement is between employer and
employee; it is not usher's contract to implement. Usher ships no
hidden or default-on reporting path of its own (orgs that want
force-enable act through their own visible channels — MDM, managed
settings — and that act is theirs, not usher's).

Rationale: SOUL.md §1 user sovereignty (no phone-home). Software in
this category requires trust, or users will not accept the benefits
for fear of hidden traps. Independently corroborated by finding-001:
the consent patterns that survived community scrutiny (Kolide honest
security, loud disclosure, DO_NOT_TRACK) versus the one that burned
trust (Homebrew's silent opt-out analytics).

Decided: 2026-07-12, user ratification.

Amended 2026-07-26: the off switch must gate the code path, not just the
send. A control that does not gate the path is worse than no control,
because it converts a privacy claim into a discoverable lie. Consent is
revisable and withdrawable in both directions, consent events are
recorded, and attention is treated as the scarce resource usher spends.
Detail: `_kos/nodes/bedrock/val-reporting-off-by-default.yaml`; evidence:
`docs/research/grok-build-collection-scope-20260726.md`.

*Everything else: nothing committed yet. Seeded 2026-07-12;
design/elicitation via bmad is the next step, now with evidence:
finding-001 / 
`docs/research/prior-art-landscape-20260712.md` gives each frontier
question below an evidence-backed starting direction (research doc
Part 3) — architecture convergence (usherd daemon + signed-catalog
cloud + MCP surface), posture constraints (honest security, consent
stack, never-sole-authority, advise-don't-judge), and ranked failure
modes (Boxen conformity death, Homebrew-analytics backlash, bootstrap
rot, catalog burden, the two pennyfarthing seams).*

## Frontier

### F1: What is usher's component decomposition?

Candidate client parts: `usherd` (mac/windows background service), an
MCP or other in-LLM interface, a CLI or app UI, possibly a spider if
`usherd` doesn't scan. Plus the cloud service (collector + compliance
templates + recommendations + dynamic-options index) in this same repo.
Which of these exist, in what language(s), and in what order?

### F2: Onboarding scope — what does "get started" cover?

Install + configure Claude/marvel/codex, pre-AI service attachment,
correct initial settings, updates, bootstrap via sideshow or eos, git
hand-holding for repository access. Where is the boundary with sideshow
(pack distribution) and the fleet-bootstrap packs?

### F3: Telemetry shape [consent model RESOLVED → B1]

Metrics-only vs OTEL. What is reported: harness configuration, versions,
projects, tool calls, token counts (if possible), options set, errors.
Usher is a helper and a tattletale of the benign, runner/messenger
kind; it must not become a creepy peephole.

The consent model is decided (B1): off by default, user-enabled,
employer-forced enablement is employer-employee business outside
usher's compact. Remaining open: the wire shape (metrics-only vs
OTEL), the collected-signal inventory and its "we do NOT collect"
complement, first-run disclosure UX, and how the user sees their own
data (honest-security personal page per finding-001).

### F4: Cloud-side compliance and recommendations

Template-based compliance per setup type; recommendations on spend
outliers, cost reduction, settings. What are the templates, and where
does the recommendation engine live?

### F5: Name and persona

"Usher" fits the theatre family and the beadle-style persona feel
(front-of-house helper, quietly present). Note: beadle's repos.yaml
notes already describe beadle as "the minor official who ushers" —
the overlap needs resolving (beadle tends issues; usher tends humans
and harnesses). Backronym candidates (undecided):
- **U**sage, **S**ettings & **H**arness **E**nvironment **R**eporter
- **U**ser **S**etup, **H**arness **E**nrollment & **R**eporting

### F6: The instance model — usher serves any fleet; the repo is not the instance

Direction ratified 2026-07-25 (party-mode session; design open).
Usher works for **any fleet**: it is deployed by an org or team,
customized by that team for their fleet, and it serves the individual
person by contributing to — and being supported by — that team's
collective knowledge. The repo and the tools it deploys are NOT its
instance.

Consequence: there must be a **bare-org deployment approach** that
(a) associates a usher installation with its service instance, and
(b) collects the instructions/customizations belonging to that
instance/team. Open questions: what is the minimal bare-org artifact
(a repo template? an org-level config? a signed manifest)? How does
an individual's usher find its team instance? How does team
customization layer over shipped defaults (echoes sideshow's
user-install + orc-declared split, and the BMAD add-on repo's scope-classified
learning — same shape, third instance)? How does this reframe F2's
onboarding boundary and F4's cloud side (the "cloud service" may be
per-team, not central)?

Feeds the elicitation (`aae-orc-0mmn.1`) as its scoping input.

Revised 2026-07-26: "enterprise versus individual" was a taxonomy error
and is retired. The axis is individuals and organizations. A person may
hold more than one affiliation; a community and an employer are the same
shape with different admissions rules. That reframe is what exposed F7.

### F7: Admissions and organization-locked settings

What may an organization require of a member, how is the requirement
surfaced before acceptance, and what does a member see about what is
shared? Carries a device-management corollary: install may be silent,
observation may not begin before consent.

Node: `_kos/nodes/frontier/question-org-admissions-and-locked-settings.yaml`

### F8: Does a small team sustain a contribution loop?

The brief's bet is the off-baseline contribution loop. Participation-
inequality literature says such loops do not emerge on their own at a
dozen to two dozen people, and no empirical minimum size exists either
way. Open: whether near-zero authoring cost changes the arithmetic, since
every documented failure measured loops where contributing meant writing
something.

Node: `_kos/nodes/frontier/question-contribution-loop-sustainability.yaml`

### F9: Which locally observable signals predict cost efficiency?

Billing-side reporting can show that a spread exists; it cannot show which
local behavior moves it, because it sits too far from the levers. Candidate
local signals: cache expiry, model thrashing, image uploads, interaction
timing. Method constraint: any probe runs against an organization's own
data inside that organization's own repositories, and only mechanism
returns here.

Node: `_kos/nodes/frontier/question-local-signal-validity.yaml`

### F10: Settings discovery across directories

Harness settings live in more places than any per-repo tool can see:
managed, user, per-project in every directory a person launches from, and
the live environment, each with harness-specific precedence. A per-repo
doctor reads the user file plus its own directory, so stale model pins and
policy drift survive indefinitely in side directories; and a long-lived
session snapshots its env at launch, so point-in-time file reads and live
session state are different observables. Usher, as the machine-level
configuration observer, is the natural home for the sweep — under the B1
consent model, reporting derived state (what diverges, where) rather than
content.

Node: `_kos/nodes/frontier/question-settings-discovery-across-directories.yaml`

## Graveyard

*Empty.*
