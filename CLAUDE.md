# usher — front-of-house for the AI tooling fleet

## What This Is

Usher helps humans get seated and keeps the house running. Two main
functions:

1. **Onboarding / install** — get people started with AI tools (Claude
   Code, marvel, codex, …): install, configure with correct settings,
   attach pre-AI services, update, bootstrap with sideshow or eos, and
   help with repositories for users who aren't fluent in git.
2. **Harness observation** — monitor harness configuration and activity;
   optionally send telemetry (metrics-only or OTEL): configuration
   reporting, versions, know-your-users enterprise services — who is
   running what software on what projects, token counts where possible,
   options set, errors experienced.

Both functions report to/through a **cloud service that lives in this
repo too** (service side, not client side). The cloud side collects
reports and runs template-based compliance for different setups to
produce recommendations (spend outliers, cost-reduction options,
recommended settings). The cloud side may also serve an index of
dynamic options that extends the client's built-in capabilities, so
usher can be extended without a client update.

Think wakatime + enterprise configuration management, starting with
configuration reporting and usage reporting. After install, usher
should be pretty much invisible — just doing its thing.

## Status

Design phase. No code yet. Elicitation and design via bmad are the
next step; language and component boundaries are undecided. Candidate
client-side parts: a (mac/windows) service `usherd`, an MCP or other
in-LLM interface, a CLI or app UI, possibly a spider if `usherd` isn't
doing the scanning.

## Architecture

```
usher/
├── CLAUDE.md          (you are here)
├── charter.md         (bedrock / frontier / graveyard)
├── _kos/              (repo-scoped knowledge graph)
└── .claude/rules/     (fleet-bootstrap rule bundle)
```

@.claude/rules/_index.md

## Conventions

- **Git workflow:** trunk-based, `main` only. Gitflow adoption (per orc
  charter B11) waits for a distribution channel.
- **Visibility:** private for now.
- **Auth:** delegate to owning tools; never store credentials (SOUL.md §3).
- **Task tracking:** bd via `.beads -> ../.beads` symlink (orc shared).

## How to Work Here (kos Process)

Repo-scoped questions live in this repo's `_kos/`. Cross-repo questions
live in the orchestrator's `_kos/`. Follow the orc session protocol:
orient (charter.md) → ideate (_kos/ideas/) → question → probe →
finding → harvest. Commits use kos action vocabulary (probe:, finding:,
harvest:) alongside conventional commits.
