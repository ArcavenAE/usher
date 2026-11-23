# Prior intent not captured: consolidated themes (2026-11-04)

Merged from three sweeps, 121 rows with heavy overlap:
`reconcile-prior-usher-core.md` (38), `reconcile-prior-usher-research.md` (46),
`reconcile-prior-orc.md` (37). Row-level sources are in those files.

| # | Theme | Prior intent (key sources) | Current state | Proposed home |
|---|---|---|---|---|
| A | Recommendations | usher CLAUDE.md:20-22; charter F4; July brief 115-118; fleet-settings-profile idea:51 (from permission denials) | No FR; PRD §1 says usher reports facts and flags findings; brief keeps a bullet | PRD new feature; §1 |
| B | Central service, compliance templates, dynamic-options index | usher CLAUDE.md:18-24; charter F1; orc idea usher.md:32-38; repos.yaml:63-65; July 439-441 ("templates must not slip") | Only in §7.2 as v2; addendum §5 ties templates to v2; §2.2 "not a compliance product" | PRD §1 and a feature; §7 keeps the order |
| C | Contribution loop as the differentiator | July 104-118, 197-203; market research 704-750 | Only §7.2 v2+; brief keeps it | PRD §1 and a feature |
| D | Onboarding scope: attach pre-AI services, bootstrap with sideshow or eos, repo and git help for people not fluent in git | usher CLAUDE.md:10-11; charter F2; orc usher.md:14-16, 24-25; prior-art 11-14, 176-178 | Missing; eos appears nowhere; UJ-1 "pointed at a repo" has no FR | PRD §4.2 or new feature; §2 audience |
| E | Updating tools and usher itself | README 5-7 ("stay updated"); finding-001 (declarative stay-current); aae-orc-9va5 (aq/ax defers its updater to usher); prior-art 225-243 | FR-9 reports behind, FR-11 installs missing only; no self-update | PRD §4.2 |
| F | Restore and repair a seat; catch up; write safety | aae-orc-9fwcr modules 4, 5 | Comparison and watcher only; no catch-up of team changes; no dry run, diff, backup, rollback | PRD §4.6 and §5 |
| G | What a seat includes | aae-orc-9fwcr module 1 (MCP registrations, packs, bus and marvel client config, backend and auth route, toolchain); cache research 140-150, 202-218 (scheduled and headless launch environments, backend-conditional findings) | Environment is harness settings by launch directory | PRD §3 Environment, FR-1, FR-43 |
| H | Activity and usage reporting; wire shape | usher CLAUDE.md:12-16 (projects, tool calls, tokens, errors); F9 node 51-66; charter F3 (metrics-only or OTEL); marvel service-provider idea 76-78 (signals through usher's telemetry) | FR-26 cache reuse only; no OTEL, no wire-shape question | PRD §4.7; §9 |
| I | Surfaces: in-LLM (MCP) onboarding, usherd, CLI or app | prior-art 472-476; charter F1 | MCP only in Q9 and a token-budget line | PRD feature or §9 |
| J | Team instance enrollment and layering | charter F6 95-114; July 469-473; prior-art 505-508 (a personal layer usher always respects) | Missing | PRD §4.5 or §9 |
| K | Honest-security posture | July 379-407; prior-art 484-487 (person sees own data first; drill-down a recorded team decision; DO_NOT_TRACK; record whether an enable was member-chosen or organization-required); charter B1 (org may enable through its own MDM channel) | Shrunk; FR-37 reads narrower than B1 | PRD §5, FR-37, FR-39 |
| L | Expectations from other components | aae-orc-lp2pg (usher presents a fleet report from critic and the fleet watcher output); aae-orc-yyqp3 (seat a reviewer at any run variant); aae-orc-z6y7 (usher-managed key shim, brokering under ADR-009); tap design doc 58, 68 (homebrew formula) | Missing | Needs rulings; see below |
| M | Cleanups | charter F7 (stale against the Q7 ruling); brief addendum §C (medium-based boundaries); "spend outliers" against no-ranking; an older note placing token accounting in a sibling | Inconsistent | Brief addendum; charter (operator) |

## Theme L needs rulings

- Fleet report presenting critic and the fleet watcher output: by objective, presenting another
  component's judgment is not setting up or maintaining settings. Usher's part may be only
  the settings side of the report.
- Seating a reviewer at any run variant: fits "setup" if a variant is an environment.
- Key shim: brokering is permitted by ADR-009, custody is not; it conflicts with nothing,
  but it is a new function.
- Homebrew formula: a distribution channel, blocked while the repo is private (Q11).
