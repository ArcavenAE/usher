# usher Charter

Last updated: 2026-07-12 (seeded; prior-art probe complete — finding-001)

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

## Graveyard

*Empty.*
