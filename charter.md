# usher Charter

Last updated: 2026-07-12 (seeded — no probes run yet)

## Bedrock

*Nothing committed yet. Seeded 2026-07-12; design/elicitation via bmad
is the next step.*

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

### F3: Telemetry shape

Metrics-only vs OTEL. What is reported: harness configuration, versions,
projects, tool calls, token counts (if possible), options set, errors.
Privacy/consent model — usher is a helper and a tattletale of the
benign, runner/messenger kind; it must not become a creepy peephole.
How does this square with SOUL.md user sovereignty (no phone-home) —
enterprise opt-in context is different from BYOA personal use, and the
boundary needs explicit design.

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
