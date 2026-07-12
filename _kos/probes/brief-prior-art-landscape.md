# Exploration Brief — Prior-Art Landscape for Usher's Two Functions

- **Question:** What does the landscape of prior art look like for
  (1) self-service install/onboarding of developer tooling and
  (2) usage telemetry + inventory + configuration assessment with
  central reporting — and what features did those projects select?
- **Hypothesis:** Both functions have mature non-AI prior art
  (bootstrap tools, MDM self-service, wakatime-class telemetry,
  osquery-class inventory) plus a fast-moving AI-specific segment
  (Claude Code OTEL, spend observability). Feature selection patterns
  from both should constrain usher's design before bmad elicitation.
- **Method:** (a) Mine internal prior art — pennyfarthing's
  WheelHub/BikeRack/BikeShop PRDs and RaceCoach stub, the kos
  mvg-bootstrap probe nodes with their seam findings; (b) two
  parallel web-research passes, one per function; (c) synthesize
  into docs/research/prior-art-landscape-20260712.md.
- **Timebox:** one session (2026-07-12).
- **Success signal:** a research doc that names the recurring
  architectural choices, table-stakes vs differentiating features,
  and failure modes — enough to seed the bmad elicitation
  (aae-orc-0mmn) with real constraints instead of blank-page design.
- **Status:** complete — see `docs/research/prior-art-landscape-20260712.md`
  and finding-001.
