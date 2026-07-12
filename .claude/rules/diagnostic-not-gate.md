# Diagnostic, Not Gate — Vital Signs Stay Out of Required Checks

Behavior-trigger rule. Same shape as the force-push abort signal,
`tooling-friction.md`, and `agent-tools.md`. Names the *typing pattern*,
not the concept. Enforcement locus for ADR-007 clause 3 / SOUL.md §8.

## The trigger

If you find yourself about to type any of these, **STOP**:

- Adding a `kos status`, `kos census`, `kos reflect`, or any kos
  vital-sign command to a CI workflow's required checks, branch
  protection, or a blocking lefthook/pre-commit hook
- Adding `bd stats`, `bd stale`, staleness thresholds, throughput
  counts, or briefless-probe ratios as a pass/fail condition anywhere
- Wiring any "how many / how fast / how stale / how often" number so
  that a nonzero or over-threshold value blocks a commit, merge, or
  release
- Writing a script that exits nonzero on a health metric (as opposed
  to a structural check) and calling it from CI

That is a gate being created by accretion. Before proceeding, ask:
**is there a ratified gate definition for this metric?** — a written
decision (ADR, charter bedrock, or explicit user ratification in the
session) that scopes the gate, names the metric, and states why gating
is correct here. If yes, cite it next to the check. If no, stop and
propose it to the user instead of wiring it.

## What MAY gate without ceremony

Structural validity is not a vital sign (ADR-007, "Structural validity
vs vital signs"):

- `kos validate` — parse errors, illegal edge types, misfiled
  confidence directories (kos#54 CI gate is legitimate)
- Schema checks, linters, compilers, test suites
- `bd lint` shape checks on issue structure

The test: does the check ask "is this artifact well-formed?"
(structural — may gate) or "is this project healthy/fast/fresh/used
enough?" (vital sign — diagnostic; gates only via the exception path).

## Why this rule exists

Goodhart/Austin (kos literature survey, 2026-07-04 research drop): a
measure that becomes a target stops measuring. The kos vital-sign
instrumentation (census, status, staleness) exists to inform operator
judgment. The first time one of those numbers blocks a merge, every
incentive points at gaming the number instead of reading it — and the
instrument is dead. The keystroke this rule intercepts is the cheapest
place to stop that: the moment the metric is typed into a required
check, not the quarter after the team starts gaming it.

## Cross-references

- ADR-007 (decisions/adr-007-bainbridge-automation-boundary.md)
- SOUL.md §8 — Automation Boundary
- Behavior-trigger pattern: `~/.claude/CLAUDE.md` (force-push abort),
  `tooling-friction.md`, `agent-tools.md`
- kos roadmap v3.1 standing rule 4; Grudin/Goodhart/Bainbridge survey
  sections (kos/docs/)
