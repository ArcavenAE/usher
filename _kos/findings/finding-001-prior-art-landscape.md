# finding-001 — Prior-art landscape: both usher functions have proven shapes; the union is whitespace

- **Probe:** `_kos/probes/brief-prior-art-landscape.md`
- **Date:** 2026-07-12
- **Confidence:** frontier
- **Full detail:** `docs/research/prior-art-landscape-20260712.md`
  (the prop; this finding is the summary)

## Result

1. **Internal lineage exists and is adjudicated, not inherited.**
   Pennyfarthing's BikeShop/BikeRack/WheelHub + RaceCoach designed
   most of function 2 in Feb 2026 for a different product under
   different assumptions. Each candidate borrowing is tabled with a
   portability note (accept/adapt/reject at bmad design); two are
   flagged non-portable (dashboards-as-product, BMAD workflow-stage
   telemetry). Two pennyfarthing seam findings become usher design
   constraints: build the fidelity/compliance channel (not just
   execution telemetry), and budget context on any in-LLM surface.

2. **Function 1 (onboarding/install):** curated-catalog + thin-client
   is the winning shape (Munki/winget/Kandji); hybrid
   imperative-day-0 + declarative-stay-current; sign the feed
   (Sparkle/TUF); interop with Claude Code managed settings rather
   than parallel policy; Boxen's death defines the anti-pattern
   (sole-authority conformity + non-actionable failure). **No
   productized cross-AI-tool install/config manager for non-experts
   exists** — open lane.

3. **Function 2 (observation/assessment):** local observation
   (daemon + per-harness collectors: JSONL logs, config files, OTLP
   ingestion) is the only cross-vendor architecture; Claude Code
   exports first-party OTEL and orgs can force it via managed
   settings; policy-as-template has three shipped precedents
   (Fleet SQL policies, InSpec controls, Kolide checks);
   Kolide's Honest Security is the market-validated
   helper-not-surveillance posture; Homebrew analytics is the
   consent cautionary tale. **No product combines cross-harness
   observation + AI-config compliance templates + user-facing
   honest-security recommendations** — the union is usher's
   whitespace.

4. **The two functions close a loop:** install side knows desired
   state, observation side knows actual state; reconciliation
   between them (chezmoi/Munki applied to AI-tool config) is a
   differentiator neither function has alone.

## Affects

- charter F1–F5: each frontier question now carries evidence-backed
  direction (see research doc Part 3 "Sharpened open questions").
- bd `aae-orc-0mmn` (bmad elicitation) — this research is its input.
