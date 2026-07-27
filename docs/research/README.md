# usher research index

`docs/research/` is the canonical home for durable research artifacts in
this repo. Working output from tool-driven workflows may land in
`_bmad-output/` first; when it becomes a durable reference, it moves here
and gets a row in the table below.

Why the rule exists: the teams-over-individuals market research was
committed on 2026-07-26 into `_bmad-output/planning-artifacts/research/`,
away from its siblings. Later the same day a design session ran against
the charter and the product brief without it, and a thirteen-voice review
panel argued about the layer that research had already classified as table
stakes. Every artifact existed; nothing pointed at them. Filed as an
instance of the orchestrator's F19 (active knowledge surfacing) and F25
(stage rig) in `_kos/findings/finding-002`.

| Document | Date | What it establishes |
|---|---|---|
| [prior-art-landscape-20260712.md](prior-art-landscape-20260712.md) | 2026-07-12 | Architecture convergence, posture constraints, ranked failure modes. Grounds charter B1. |
| [prompt-cache-ttl-by-backend-20260721.md](prompt-cache-ttl-by-backend-20260721.md) | 2026-07-21 | Prompt-cache TTL behavior per backend. Input to the analysis side. |
| [market-teams-over-individuals-20260726.md](market-teams-over-individuals-20260726.md) | 2026-07-26 | Teams-over-individuals thesis supported; the off-baseline contribution loop specified in eight steps; collectors are table stakes, the loop is the moat. |
| [grok-build-collection-scope-20260726.md](grok-build-collection-scope-20260726.md) | 2026-07-26 | A 2026 same-category consent failure. Gives B1 a testability requirement and the design rule of state-not-content. |

Reading order for someone new: prior-art (posture), market (the bet),
grok-build (the line you must not cross), cache-TTL (a worked example of
the kind of knowledge usher exists to hold).
