---
title: "Product Brief Addendum: usher"
status: draft
created: 2026-11-02
updated: 2026-11-02
---

# Addendum: usher brief, 2026-11-02

This addendum holds depth that belongs to the PRD and architecture steps,
or that earned a place but does not fit the brief. Orc paths are relative
to the aae-orc root. The orc material is indexed in
`docs/research/orc-index.md` (section D).

## A. The contribution loop

Carried from the July brief (lines 120-195), unchanged in substance except
that its input is now named **divergence**.

1. **Detect** divergence, which may be an improvement, as distinct from
   drift, which is state falling behind what was declared, with no upside.
2. **Declare.** The person adds intent; the body is generated from the
   observed difference. Declaring must stay nearly free, because authoring
   cost is the documented failure of internal contribution marketplaces.
3. **Describe.** Difference, scope, rationale and observed effects become a
   portable proposal.
4. **Assess.** Routed to a named reviewer with a review clock. Two weeks
   is the point past which an unreviewed contribution breaks the loop for
   that person.
5. **Enrich and test.** Automated checks; human judgment (ADR-007).
6. **Decide.** Accept, hold as a sanctioned option, or decline with a
   reason. Baseline changes are human acts.
7. **Ripple.** The change spreads through the signed catalog or the team
   profile, offered, never forced. In v1 this is a proposal only; render
   is v2.
8. **Credit.** Attribution rides the proposal.

Literature notes (from `docs/research/market-teams-over-individuals-20260726.md`):
participation inequality survives payment; a team of twelve to twenty-five
predicts roughly two to five natural contributors; the dominant failure is
return-on-investment misalignment.

F8 sub-questions the PRD should carry
(`_kos/nodes/frontier/question-contribution-loop-sustainability.yaml`):
misalignment may return at review and adoption; one busy reviewer against
the two-week clock; credit with no promotion system to convert it into;
whether "three contributors, twenty consumers" counts as success.

## B. Instance model (F6, F7)

Carried from the July brief (lines 252-312):

- One daemon, N affiliations; scope resolution assigns each observation.
- Two artifacts split by visibility: a claim marker at a scope root
  (committable, scope names only) and a private binding (local only).
  bmad-extras' per-person `~/.config/aae/scopes.toml` (aae-orc-4b0c) is a
  candidate instance of the same pattern.
- Affiliations isolated with a stated mechanism, not only an intent.
- Nested claims need resolution rules.
- Attribution follows scope, not credential.
- Three units: attribution (scope), benefit (person), decision
  (organization).

Correction from the 2026-10-26 idea: organization "locks" are enforceable
only for managed-only keys (`allowManagedPermissionRulesOnly`,
`disableAutoMode`). Auto-mode allow entries are additive and can override
an organization soft deny. F7's "what an organization may require" must be
answered per setting class.

## C. Sibling boundaries

| Component | Overlap | Source |
|---|---|---|
| sideshow | install and five-layer doctor; signed packs overlap the ripple substrate | orc charter F24; ADR-008; aae-orc-9fwcr, aae-orc-70os |
| marvel | seat settings via per-session `--settings`; measurement fabric | fleet-settings-profile idea; marvel/docs/design/service-provider/03-probes-and-roadmap.md:42 |
| wardrobe | `constraints.permissions` and `policy.sh` render | aae-orc-079r8 (closed 2026-10-28) |
| stagekeeper | read-only session watcher | stagekeeper/charter.md B1; aae-orc-lp2pg |
| critic | fleet aggregation | aae-orc-lp2pg |
| beadle | name and persona overlap | repos.yaml:70; usher charter |
| aq/ax | install and update posture belongs to usher, not a self-updater | aae-orc-9va5 |

Usher is absent from the independence matrix (CLAUDE.md, ADR-005), and
`marquee/data/ecosystem.yaml:80` draws an unratified "marvel to usher:
Onboards" edge. Both need a decision.

## D. Evidence pointers (orc)

The orc index, `docs/research/orc-index.md` (landed in #9), replaces the
list of pointers that was here. How its sections map to the brief:

- Sections 1 and 5 (install and update failure modes; the install path
  usher would drive): the Problem and install and update health.
- Sections 2 and 4 (settings discovery; settings evidence): the settings
  map and the drift check.
- Section 3 (spend and telemetry): the cost-side output. Use the token
  audit's method, not its figures.
- Section 6: open bd work touching usher, none linked to aae-orc-0mmn.
- Section 7 (field notes): the device-management reinstall that dropped
  custom profiles, and per-host credentials. Both are in the brief's
  posture.
- Section 8 (constraints): per-person data never enters git, client names
  stay out, reporting stays off by default.

## E. Carried from July

Prior-art rulings (July brief lines 426-441) stand:

- **Accept:** typed protocol with local cache and flush-on-reconnect,
  system-generated push channels, and "it advises; the rider decides".
- **Adapt:** keypair auth (prefer the organization's directory or device
  management), broadcast routing and zero-interruption visibility.
- **Reject:** session tiles as primary UI, and workflow-stage telemetry
  as-is.

The two predecessor seam obligations remain: compliance templates are the
fidelity channel and must not slip, and every in-LLM surface carries a
token budget from day one.

F9 detail for the cost-side output (`_kos/nodes/frontier/question-local-signal-validity.yaml`):
a price table missing a new model silently drops its tokens; unbilled
throttled calls look like efficiency gains; four to six weeks is the
minimum evidence window; compare only within backend cohorts
(finding-002); context-file size over time is a further signal.

Cache-TTL research (`docs/research/prompt-cache-ttl-by-backend-20260721.md`)
was read from one Claude Code version, and one value is server-controlled.
Every check must version-stamp its assumptions.

## F. What changed since July, and why

- **Lead problem.** Cost led in July. Settings and permissions friction
  leads now, after the 2026-10-26 operator ask and aae-orc-9fwcr's "find
  your seat" framing. The cost mechanism is kept as the second payoff.
- **First outputs.** The per-session cache score was the July candidate.
  The settings map with drift check, and install and update health, now
  come first; the cache score is the first cost-side output.
- **Secrets.** "Never read" became "presence, never value" so plaintext-key
  detection can live in usher without custody.
- **Vocabulary.** "Drift" was retired in July because there was no fixed
  baseline. A declared profile is a fixed baseline, so drift returns with a
  narrow meaning, and divergence carries the upside.
- **Harnesses.** Goose added on the strength of finding-aae-orc-2w6k.
- **Customer split.** The fleet engineer stays the customer; the developer
  is now named the primary user, and non-human seats are a third subject.
- **Concept type.** Open source, decided 2026-11-03. The repo is private
  today; going public is open question 11.
- **Evidence base.** At least one team outside this fleet has the same
  problem. The operator does not disclose which.

## G. From the harness settings research (2026-11-02)

Source: `_bmad-output/planning-artifacts/research/technical-harness-settings-observability-map-2026-11-02/research.md`
(43 sources; of 27 tracked claims, 15 verified, 6 unverified, 6 disputed,
2 overturned).

- **Codex: ask the harness.** The app-server `config/read` call, given
  `cwd` and `includeLayers`, returns the effective config, every layer with
  a `disabledReason`, and an `origins` map from each key to the layer that
  set it. `configRequirements/read` returns admin constraints. Confirmed
  against installed 0.160.1 by generating the protocol schema.
- **Claude Code: rebuild, then confirm.** No command names the file behind
  each key. Usher models the documented stack, reads the server-managed
  cache (`~/.claude/remote-settings.json`, which may differ from effective
  state), and confirms against `claude auto-mode config`, `/permissions`
  and `claude doctor`.
- **Goose: rebuild from source, not docs.** The stack is
  `/etc/goose/config.yaml`, then `GOOSE_ADDITIONAL_CONFIG_FILES`, then the
  user file, with environment variables ahead of files. The docs omit the
  system layer, and the distro guide names a telemetry variable the code
  never reads; the kill switch is `GOOSE_TELEMETRY_OFF`.
- **Resolved over declared.** A file saying X does not mean X is in effect,
  so the drift check compares the profile with the resolved view wherever
  one exists.
- **Two layers of staleness.** Scopes, precedence and paths change slowly;
  keys and defaults change per release. The fast layer is re-verified per
  release and keyed to the version install health detects.
- **Disputed, one cheap test each before the PRD relies on them:** whether
  omitting `"$defaults"` still drops Claude Code's built-in auto-mode rules
  on 2.1.317 (this decides the settings-profile idea's structural gate);
  where codex ranks its device-management config; whether codex loads
  project skills in untrusted repos; goose's `GOOSE_MODE` default.
- **Answered:** codex retired `approval_policy = "untrusted"` (PR 39630,
  2026-08-20), so a profile must not name it. This closes open question 3
  of `_kos/ideas/fleet-settings-profile.md`.
- **Unconfirmed:** whether codex warns at startup when it ignores project
  keys. The brief no longer says it does so silently.
