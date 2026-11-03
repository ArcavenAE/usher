# aae-orc material that bears on usher

This is an index of pointers. It copies nothing. Each entry sits in the
orchestrator repo (aae-orc) or its tracker (bd) and bears on usher's two
functions, onboarding and install, and harness observation, without naming
usher. A bmad agent working here should open the entries in the section that
matches its task. Paths are relative to the aae-orc root unless a repo is named.

## 1. Install and update failure modes

- `_kos/findings/finding-182-claude-self-update-wedges-running-seat.md`: a claude self-update wedges a running seat; a fresh spawn escapes it.
- `_kos/findings/finding-aae-orc-gntz-installed-copy-drifts-from-merged-script.md`: an installed copy of a script drifts from the merged one, and the docs point at the stale copy.
- `_kos/findings/finding-221-brew-tap-ambiguity-hides-an-upgrade.md`: a doubly-tapped brew formula makes `brew outdated` a check that never ran.
- bd `aae-orc-9va5`: nothing detects an installed copy drifting behind its source.

## 2. Settings discovery across harnesses

- `_kos/findings/finding-087-cross-harness-discovery-verification.md`: instruction and skill discovery across harnesses, checked against vendor docs.
- `_kos/findings/finding-aae-orc-2w6k-harness-set-beyond-claude-and-codex.md`: the harness set beyond Claude Code and codex, ranked, with proposed criteria.

Relates to the usher frontier node `question-settings-discovery-across-directories`.

## 3. Spend and telemetry

- `docs/reports/2026-10-token-audit/`: aggregate tables only (`report.md`, `by-model.csv`, `by-token-class.csv`, `by-activity.csv`, `by-day.csv`).
- `docs/design/2026-11-01-i100-otel-execution-plan.md`: telemetry off by default with an owner-held collector; matches the usher value `val-reporting-off-by-default`.
- `_kos/findings/finding-145-cost-metered-by-proxy-in-the-conductor.md`: spend metered by proxy disarmed a budget rail.
- `_kos/findings/finding-123-harness-invocation-agent-identity-survey.md`: agent attribution is an unused field, not a missing capability.

## 4. Settings evidence for compliance templates

- `_kos/findings/finding-055-bedrock-fable5-context-window-clamp.md`: the Bedrock context clamp.
- `_kos/findings/finding-056-fable5-1m-autocompact-window-study.md`: the autocompact window for the 1M model.
- `_kos/findings/finding-057-multi-model-autocompact-window-study.md`: the autocompact window across the model fleet.
- `_kos/findings/finding-021-claude-permission-protocol.md`: the permission protocol.
- `_kos/findings/finding-024-integrated-security-model.md`: the integrated security model.
- `_kos/findings/finding-091-claude-plugin-identity-directory-marketplace.md`: plugin identity for delivered packs.

## 5. The install path usher would drive

- `tools/install.sh`: puts `aq`, `ax` and `ae` in `~/.local/bin`.
- `~/.beads/shared-server/client-env.sh`: the bd client env, described in the task-workflow skill (`.claude/skills/task-workflow/SKILL.md`, section "bd server").
- wardrobe `README.md`, section "Updating an install" (in the wardrobe repo).
- `_kos/findings/finding-025-bmad-6.3.0-sideshow-install-probe.md`, `finding-027-bmad-6.3.0-sideshow-install-end-to-end.md` and `finding-076-sideshow-custom-skill-binding-gap.md`: sideshow install behavior.

## 6. Open bd work not linked to aae-orc-0mmn

- `aae-orc-9fwcr`: make usher modular across the ArcavenAE tool set and the harness ecosystem.
- `aae-orc-z6y7`: long-term AI API key handling, including plaintext-on-disk detection via usher.
- `aae-orc-1vgfk`: client-side Windows support for sideshow and usher.
- `aae-orc-lp2pg`: fleet-wide context sweeps per harness runtime, and where they live.
- `aae-orc-k0l75`: mining the accumulated sim files for lessons.

## 7. Field notes not recorded elsewhere

- 2026-11-02: lost iTerm2 custom profiles were traced to the device-management tool's own Software Updates item reinstalling the app, not to an app upgrade. The profiles were also absent from the device-guard inventory. Usher and device management can both act on one machine's settings.
- Credentials are per host. Each host has its own Bedrock key. A read-only Datadog token is to be issued one per host and never copied between hosts.

## 8. Constraints for anything built from this index

- Per-person data (I-140) never enters git.
- Client names stay out.
- Reporting stays off by default.
