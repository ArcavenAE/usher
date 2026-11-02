# usher research index

This directory holds research documents for usher. It is an index, not a
canonical destination.

**Where research is supposed to go.** BMAD workflows write their own
research to `_bmad-output/planning-artifacts/research/`, which is the
canonical location defined by the `bmad-domain-research` output contract.
That directory is tracked, and it is also where the process writes briefs,
PRDs, architecture documents, epics, and stories. Nothing in this file
overrides that.

**Why some research is here anyway.** Hand-written research (the prior-art
landscape, the cache-TTL capture) was written directly into `docs/` before
any workflow ran. On 2026-07-26 the market research was additionally moved
here from `_bmad-output/planning-artifacts/research/` on a wrong diagnosis:
a design review had failed to read it, I attributed that to its location,
and it was not a location problem. The files stay here by decision rather
than by rule. New workflow output should land wherever its workflow writes
it, not be relocated to match this table.

**What actually went wrong, so it doesn't recur.** The research existed and
was correctly filed; the reviewers were briefed without it. The discipline
is to enumerate what exists, open what is enumerated, and pass the full set
to every spawned reviewer. Recorded as an instance of the orchestrator's F19
(active knowledge surfacing) and F25 (stage rig) in
`_kos/findings/finding-002`, with the mis-diagnosis kept alongside the
correction.

| Document | Date | What it establishes |
|---|---|---|
| [prior-art-landscape-20260712.md](prior-art-landscape-20260712.md) | 2026-07-12 | Architecture convergence, posture constraints, ranked failure modes. Grounds charter B1. |
| [prompt-cache-ttl-by-backend-20260721.md](prompt-cache-ttl-by-backend-20260721.md) | 2026-07-21 | Prompt-cache TTL behavior per backend. Input to the analysis side. |
| [market-teams-over-individuals-20260726.md](market-teams-over-individuals-20260726.md) | 2026-07-26 | Teams-over-individuals thesis supported; the off-baseline contribution loop specified in eight steps; collectors are table stakes, the loop is the moat. |
| [grok-build-collection-scope-20260726.md](grok-build-collection-scope-20260726.md) | 2026-07-26 | A 2026 same-category consent failure. Gives B1 a testability requirement and the design rule of state-not-content. |
| [orc-index.md](orc-index.md) | 2026-11-02 | Pointers to aae-orc and bd material that bears on usher but does not name it: install failure modes, settings discovery, spend and telemetry, compliance evidence, the current install path. |

Research written by a BMAD workflow will not be listed here unless someone
adds a row by hand. Check `_bmad-output/planning-artifacts/research/` too.

Reading order for someone new: prior-art (posture), market (the bet),
grok-build (the line you must not cross), cache-TTL (a worked example of
the kind of knowledge usher exists to hold).
