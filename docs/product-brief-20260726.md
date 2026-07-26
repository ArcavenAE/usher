# Product Brief: usher

- **Status:** DRAFT from the elicitation session of 2026-07-26 (bd
  `aae-orc-0mmn.1`); feeds the PRD step. Panel: Mary (analyst) with
  the pit (Bex, Wren, Penny, Sloane, Tessa, Scuttle, Marco, Gus).
- **Scope note:** this document names no organization. Deployment
  particulars for the first fleet live in that organization's own
  repositories and are referenced here by scope only.
- **Settled inputs respected:** charter B1 (reporting off by default,
  user-enabled) is bedrock and was not reopened. Charter F6 (usher
  serves any fleet; the repo is not the instance) was the scoping
  input and is developed further below.

## Problem

Getting a person productive across today's AI tooling means seating
them into several harnesses (Claude Code, Claude Desktop, ChatGPT,
Codex, Crush, OpenCode) over several backends (enterprise plans,
cloud platform offerings, API keys, local inference), each with its
own install, settings precedence, and failure modes. No product
manages install, configuration drift, and honest usage observation
across harnesses. Per-vendor tooling and per-concern tools each hold
one piece; the union is whitespace (research doc, Part 3).

Within a fleet, the tooling that does exist is expert-facing
(scripts, justfiles, skills) and per-machine. What should be on a
machine and what is actually on it are known to different systems,
or to nobody.

## The Bet

Usher is the **instance layer and the loop**: it enrolls a person's
machines into a team instance, drives existing install and doctor
tooling to seat them across every covered harness, and aggregates
what observation finds into per-person findings and recommendations.
The install side knows the desired state; the observation side knows
the observed state; usher owns the reconciliation between them.

Two positioning sentences ratified in elicitation:

1. **Complement now, replace over time.** Usher complements a
   fleet's existing tooling at birth and is designed to make it
   unnecessary. Existing scripts, justfiles, and skills are content
   in their current costume: their logic migrates into usher's
   catalog (the dynamic-options index) rather than being absorbed as
   code. The catalog for the first deployment is harvested from
   tooling that already passes CI, not authored from scratch.
2. **Retirement is the success criterion.** Usher has succeeded when
   none of the parallel surfaces are needed (they may be held as
   options during the transition). Retirement is observed, never
   enforced: the old surfaces evaporate because nobody reaches for
   them, not because somebody turns them off. This is a diagnostic
   measure, not a gate (SOUL.md section 8).

## The Instance Model (F6, developed)

One machine may host **multiple usher instances**, partitioned by
directory scope. The scope layers mirror the layering coding agents
already honor: system (rarely used), user home, project or
orchestrator root, repository, and nested contribution directories
inside a project. A person can carry user-level settings belonging
to one organization, project-level overrides belonging to another,
and per-contribution overrides inside that project for third-party
upstream work.

Design consequences (named here, designed in the PRD):

- **One daemon, N instance bindings.** Scope resolution decides
  which instance an observation or action belongs to, the same way
  harnesses resolve their own config chains.
- **Two artifacts, split by visibility.** A *claim marker* at a
  scope root is discoverable and safe to commit: it says "this scope
  answers to the instance named X" in scope-name-only terms. A
  *private binding* is local-only and never distributed: it holds
  where each instance lives, what is enabled, and what routes where.
  This is the fourth appearance of the same house pattern (sideshow
  user-install/orc-declared, scope-classified learning, team
  customization, now instance claims).
- **Instances are isolated.** Two instances on one machine never see
  each other's data. Classification happens at write time; consent
  (B1) is granted per instance, not per machine.
- **Nested claims exist** (a contribution directory inside a claimed
  project root is a third party's context inside a second party's
  claim). Precedence and conflict semantics are PRD work; a
  disagreement between two claim markers over one scope is a
  split-brain and needs stated resolution rules.
- **Attribution follows scope, not credential.** The same API key
  may serve two scopes in one afternoon; directory-scope attribution
  is the only honest way to split spend. No surveyed product does
  this; it is a differentiator.
- **The swappability boundary limits partition depth.** Most harness
  settings are user-wide (login, keychain, primary settings file);
  only some vary by launch directory. User-wide settings are
  therefore singleton resources shared across instances: at that
  layer multi-instance is time-sliced (one active profile, swapped
  reversibly, the pattern the first fleet's backend-profile tooling
  already proves), while directory-layer settings are genuinely
  parallel. Conflicts over singleton settings surface as advisories
  to the user, never silent rewrites (posture constraint 3 applied
  to usher itself). Usher must carry a per-harness inventory of
  which settings live on which side of this boundary.

## Deployment Ladder = Consent Ladder

Three deployments, in order, deliberately exercising the three
consent postures from the research before any stranger installs:

1. **An enterprise fleet** (managed context; org acts through its
   own visible channels for any force-enable, per B1).
2. **A small organization** (team-scale; the org runs its own
   instance).
3. **A non-affiliated individual** (personal mode; DO_NOT_TRACK
   honored, honest-security personal view is the whole product).

The first deployment already spans the full cross-harness and
multi-backend matrix, so cross-harness support is v1 scope, not a
later tier.

## V1 Cut

**Install-first, observation fast-following.** Bias for action:
ship when there is something to install.

- **Harness and backend coverage: exactly what the first fleet's
  continuity tooling already covers.** Terminal-side harnesses and
  the backend matrix, including reversible backend profiles and
  local inference with platform-specific hardware support on Apple
  silicon. Breadth is affordable because usher drives that tooling
  (delegation pattern: usher drives installers and doctors; it does
  not re-host what a package manager or an org's tooling already
  owns).
- **Chat products (Claude Desktop, ChatGPT) are sequenced later**,
  not deprioritized: their cohort is expected to be the largest in
  time, but the first users are fluent operators. This is a
  sequencing fact, not a persona decision.
- **Windows and Linux local inference: unknown**, not "later" (the
  first fleet has not identified support; different word, different
  commitment).
- **The week-two moment** (the recurring reason a user returns):
  the drift check the fleet already runs habitually, aggregated
  across machines and harnesses, with a spend or settings
  recommendation attached. The habit exists; usher upgrades it.

## Posture Constraints (inherited)

From charter B1 and research Part 3; violating any reads as spyware:

1. Honest security: the observed person sees their own data and
   recommendations first; rollups are aggregate; per-user drill-down
   is an explicit governance decision.
2. Consent stack: loud first-run disclosure, DO_NOT_TRACK honored in
   personal contexts, a published "we do NOT collect" list, content
   capture off by default. Reporting off by default (B1); enabling
   it is the user's act; org force-enable routes through the org's
   own visible channels.
3. Never sole config authority: out-of-band edits and user-installed
   tools are signals to reconcile around, not violations.
4. Advise, don't judge: recommendations are proposals; enforcement,
   where an org wants it, routes through the vendor's own policy
   layer.

## Non-Goals

- Dashboards as the product (telemetry-as-dashboard was the
  predecessor's bet; usher's product is compliance and
  recommendations).
- Enforcement through usherd (see posture constraint 4).
- Re-hosting or re-packaging what package managers, vendor
  installers, or an org's own tooling already own.
- Chat-product onboarding in v1 (sequenced, see V1 Cut).
- Proxy/gateway spend enforcement (different trust posture;
  explicitly deferred).
- One config file understood identically by every harness (the
  cross-harness contract work underway elsewhere holds that line
  too; usher reads per-harness surfaces).

## Prior-Art Borrowings: Accept / Adapt / Reject

Disposition of the predecessor-lineage candidates (research Part 0):

| Borrowing | Ruling | Note |
|---|---|---|
| Typed protocol + local cache + flush-on-reconnect | **Accept** | General distributed-client pattern; offline tolerance is table stakes |
| Per-identity keypair auth, revocation | **Adapt** | Identity should ride the org's existing IdP/MDM where present; keypair is the fallback, not the design center |
| System-generated push channels, no authoring UI | **Accept** | MVP scoping discipline |
| Broadcast-all + naive local filter | **Adapt (re-derive)** | Fleet size unknown and the multi-instance model changes routing entirely |
| Zero-interruption visibility | **Adapt** | Framing survives; the consent story is per-instance and honest-security-first, not dashboard-curious voluntary adoption |
| "It advises; the rider decides" | **Accept** | Platform constraint independently required by SOUL.md section 8 |
| Session tiles / dashboard as primary UI | **Reject** | See non-goals |
| Workflow-stage telemetry | **Reject as-is** | Assumes every session runs one workflow shape; usher observes arbitrary harnesses |

The two predecessor seam findings remain design obligations: usher's
compliance templates ARE the fidelity channel (do not let them slip
to a late phase), and every in-LLM surface carries a token budget
from day one.

## Open Questions (feed the PRD)

1. Claim-marker format and nested-claim precedence (instance model).
2. Enrollment mechanics: how an individual's usher finds its team
   instance from the claim marker plus private binding.
3. Catalog harvest: the mechanics of turning passing org tooling
   into signed catalog content, and who signs.
4. Telemetry wire shape (F3): metrics-first is the direction; OTLP
   ingestion where harnesses export natively.
5. The usher-side surface for reading per-harness config chains
   (collector spec; the cross-harness contract work gives it stable
   on-disk targets).
6. The swappability matrix: per harness, which settings vary by
   launch directory and which are user-wide singletons; and the
   profile-swap semantics for the singleton side (what is saved,
   what is restored, what happens on crash mid-swap).
7. Name/backronym (F5): "usher" survived the research and this
   elicitation; backronym undecided, no candidate ruled out.
