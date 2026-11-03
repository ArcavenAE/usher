---
title: "Product Brief: usher"
status: draft
created: 2026-11-02
updated: 2026-11-02
supersedes: docs/product-brief-20260726.md
---

# Product Brief: usher

## Executive Summary

Usher helps a person find their seat across AI harnesses, re-find it when
the ground moves, and share what works with their team. It shows where a
person's settings live, which are in effect, how they compare with what the
team declared, and whether the tools underneath are installed and current.
It recommends; it does not take over.

The pressing problem is settings and permissions friction. Every harness
resolves configuration from its own stack of layers (managed, flag,
project, user, live session), across every directory a person launches
from, and each harness ships every few days. Nobody on a team can say which settings
are in use, where, or whether the agreed baseline reached every machine.
Cost is the second payoff: the same visibility shows the local habits and
settings behind a team's largest spend lever, prompt caching.

V1 is read-only and local-first, with reporting off by default (charter
B1). It covers Claude Code, codex and goose.

## The Problem

**Settings are scattered and resolved differently by every harness.**
Claude Code reads managed, `--settings`, project-local, project and user
layers, with list merges and managed-only locks. Codex ignores provider,
auth, notify and otel keys in project config. Goose reads a system file,
`/etc/goose/config.yaml`, that its docs do not mention. Instruction and
skill discovery differs per harness (orc finding-087). A per-repo tool sees
its own directory and the user file, so stale pins and policy drift survive
indefinitely in side directories, and a long-lived session holds the
environment it launched with (charter F10).

**Permissions friction is the daily cost.** Sessions stall on auto mode,
approval policies and folder trust. Fixes are made by hand per host, and
nobody can say which hosts carry them (`_kos/ideas/fleet-settings-profile.md`).
Organization "locks" are weaker than they look: auto-mode allow entries
combine additively and can override an organization soft deny; only
managed-only keys lock.

**Install and update fail quietly.** A self-update wedges a running seat
that then reports "not logged in" with valid credentials (orc finding-182).
An installed copy drifts from its merged source while docs point at the
stale copy (finding-aae-orc-gntz). A doubly tapped formula makes the
routine staleness check stop answering (finding-221). A version check that
reads only the installed file reports a stale machine as current
(aae-orc-9va5). Device management can reinstall an app underneath a user
and drop their custom profiles, and nothing notices.

**Team knowledge does not travel.** The person who works out the right
settings keeps them on their own machine. Everyone else rediscovers them
privately, unevenly, and late.

**Cost is the downstream symptom.** Cache reuse depends on settings and
habits within a person's local control. Billing sits too far from those
levers to say which one moved, and a team with mature cost reporting still
cannot see it (`_kos/findings/finding-002`).

## The Solution

Usher is a local observer with a small, honest surface. Two terms stay
apart: **drift** is installed or resolved state falling behind what was
declared, with no upside; **divergence** is a person off the team
baseline, which may be an improvement and is the contribution loop's input.

- **Settings map.** For each supported harness, in every directory the
  person launches from: where settings live across managed, user, project
  and live-session layers, and which value is in effect. Usher reads the
  harness's own resolved view where one exists, because the harness's
  precedence is the truth. Codex reports its effective config and the
  layer behind each key; for Claude Code and goose, usher rebuilds the
  stack from files and checks it against the harness's own views. Each
  reading names the harness version it assumes (addendum, section G).
- **Drift check.** A team declares a versioned settings profile. Usher
  compares each host's resolved state with it, not the files alone, and
  reports, per entry, present, missing or stale, plus structural hazards
  checked per harness version (an allow list that drops the built-in
  defaults, for one, still to be confirmed on current Claude Code).
- **Install and update health.** Whether each tool is installed, from which
  source, at which version, and whether that version is behind its source.
  Usher drives the existing installers (mise, brew, sideshow,
  `tools/install.sh`, vendor installers); it does not re-host them.
- **Recommendations.** Proposals drawn from the team profile and from what
  the person's own harness recorded (permission denials, for example). Each
  declares what it could not observe.
- **Divergence, offered back.** When a person's settings differ from the
  baseline in a way that works better, usher drafts the difference as a
  proposal for the team, with credit (addendum, section A).

## What Makes This Different

Spend meters, config readers and change detectors exist or can be built in
an afternoon, and vendor admin consoles deepen monthly. Two things exist
nowhere: one view of settings across several harnesses and directories,
and a path that turns one person's better setting into the team's
baseline. The value is the cross-harness map plus the loop plus the
posture, not the collectors. Treating divergence as a possible improvement,
not as decay to clamp, keeps usher out of the compliance category.

Usher is open source. Its posture claims (read-only, no egress when
reporting is off, credential presence but never value) can be checked in
the code by a team's security reviewer, which a closed observer on every
developer machine cannot offer.

## Who This Serves

- **Customer: the fleet engineer, the AI advocate.** Adopts usher for a
  team, owns the team profile, and gains standing when the team's seats
  work and spend falls.
- **Primary user: the developer.** Finds their seat, re-finds it after an
  update or a new machine, and adopts team settings safely. May install
  usher or receive it through device management; either way the value
  lands without understanding usher's internals.
- **Third subject: non-human seats.** Agent sessions launched by marvel
  receive settings through marvel's per-session projection. Usher reports
  on them; marvel writes for them.
- **Beneficiary: the organization**, which pays less for more and can see
  what is in use without seeing content.

Usher serves individuals and organizations; a person may belong to several
organizations, each with its own consent (charter F6, F7; addendum, section B).

The evidence so far comes from this fleet and from at least one other team
with the same problem, not named here.

## Success Criteria

- A team outside this fleet installs usher and reaches a working seat on
  its own harnesses without our help.
- A developer on a new or updated machine reaches a working seat on all
  three harnesses without asking anyone.
- The fleet engineer can answer "which hosts carry entry X of the profile"
  and "which hosts run a stale copy of tool Y" from usher, not from a round
  of messages.
- At least one team setting per quarter arrives through the divergence path
  rather than by hand. (Diagnostic, not a gate; SOUL section 8.)
- Parallel surfaces (per-host scripts, hand-edited settings, doctor
  checks) are retired because nobody reaches for them. Retirement is
  observed, never enforced.
- Second payoff: the team can tie a cost change to a settings or habit
  change within days, not a billing cycle. `[ASSUMPTION: this needs F9's
  four to six week evidence window before it can be judged.]`

## Scope

**In for v1:**

- Claude Code, codex and goose (goose ranked first to add in orc
  finding-aae-orc-2w6k). Other harnesses enter through the per-harness
  observability map.
- Settings map and drift check, read-only.
- Install and update health for the harnesses and the ArcavenAE tool set,
  by driving existing installers.
- Per-session cache efficiency score as the first cost-side output.
- macOS first. Linux likely. Windows unknown (aae-orc-1vgfk).

**Posture, from the first commit:**

- **Usher writes nothing to harness configuration in v1.** Proposals only;
  rendering into the user layer is v2. Seat settings are marvel's to write;
  locked settings belong to the organization's device management. Usher is
  never the sole configuration authority.
- **Asking a harness must not change it.** Before usher relies on a
  harness's resolved view (`codex app-server`, `claude auto-mode config`,
  `claude doctor`), each call is shown not to write state, refresh a
  cache, or trigger a self-update. A self-update can wedge a running seat
  (orc finding-182), and the observer must not cause the failure it
  reports.
- **Coexist with device management.** When MDM also manages an app or
  setting, usher detects and reports the overlap (a reinstall that drops
  custom profiles, for one) rather than fighting it.
- **Derived state, not content.** Usher reports that a setting has a value
  or a file exists; it does not ship the files.
- **Credentials: presence, never value.** Usher may scan for
  credential-shaped material on disk and report its location and type. It
  never retains or transmits the value, and the scan is off by default.
  This reconciles plaintext-key detection (aae-orc-z6y7 W2) with ADR-009:
  usher holds no bearer authority at any third party.
- **One credential per host.** The install flow provisions or prompts for
  a credential per host and never copies one between hosts. Usher holds
  none.
- **Reporting off by default (B1)**, with tests that prove no egress when
  off and only the published inventory when on. Any collector is
  self-hosted first (SOUL section 6).
- **Endpoint security is a design constraint:** signed and notarized
  builds, explicit outbound traffic, and a published behavioral profile.

**Out:**

- Configuration writes, remote apply, and enforcement through the daemon.
- Answering prompts (folder trust, consent) on anyone's behalf.
- Dashboards as the product; a clamp; proxy or gateway spend enforcement.
- Re-implementing mise, brew, sideshow or vendor installers; managing
  dotfiles.
- Chat products in v1 (sequenced, not deprioritized).

**Boundaries with sibling components** (each must work without usher, and
usher without them; the seam is a file format, not a call): sideshow
(pack install and doctor), marvel (seat settings), wardrobe (role
permission rendering), stagekeeper (session watching), beadle (name and
persona overlap), critic (fleet aggregation). Addendum, section C.

## Vision

Usher becomes the front of house for any team running AI harnesses: the
place a person goes to get seated, and the quiet process that keeps the
house in order afterwards. The team profile becomes a living baseline that
improves because members feed it. The same loop extends past settings to
skills, prompts and workflows, where distribution exists but graduation
does not. Usher is open source, so a team can read what it observes and
run every part of it themselves.

## Open Questions for the PRD

1. Where the team profile lives (usher repo, orc, per-fleet instance repo;
   F6), and its format.
2. The usher-marvel seam: a profile fragment marvel merges, or role files
   marvel already reads (finding-180: two `--settings` are last-wins).
3. Discovery method and consent for a home-wide directory sweep (F10).
4. The per-harness observability map: a first cut exists for the three v1
   harnesses (addendum, section G). Its disputed claims need one test each
   before the PRD relies on them, and it needs re-checking per release.
5. Does a team of a dozen to two dozen sustain the contribution loop when
   authoring is nearly free (F8)?
6. Minimum substrate for aggregate-and-return (F4), self-hosted first.
7. Admissions, org-required settings, and what a member sees before
   accepting (F7). Install may be silent; observation may not begin before
   consent `[ASSUMPTION: proposed answer, not ruled]`.
8. Catalog and profile signing: who signs, rotation, revocation, and the
   ADR-009 test for usher-minted keys.
9. Component decomposition (F1): usherd, MCP surface, CLI, cloud.
10. Name and backronym (F5), and the overlap with beadle.
11. Going public. The repo is private today. A release needs a time, a
    license, and a sweep of the repo for private repository names, client
    names and per-person data (orc index, section 8).
