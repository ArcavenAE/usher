---
title: "Reconcile: prior aae-orc material against the usher brief and PRD"
created: 2026-11-04
baseline:
  - briefs/brief-usher-2026-11-02/brief.md and addendum.md
  - prds/prd-usher-2026-11-03/prd.md and addendum.md
---

# Reconcile: prior aae-orc material against the current usher brief and PRD

Scope: aae-orc material outside the usher repo that states usher's intent,
functions, users, deployment forms or what other components expect of it.
Each row is a candidate for the operator to accept, reject or defer. Orc
paths are relative to the aae-orc root; bd ids are tickets read with
`aq show` on 2026-11-04. Baseline abbreviations: B = brief, BA = brief
addendum, P = PRD, PA = PRD addendum.

Status key: **missing** (nothing in the baseline), **shrunk** (present but
narrower), **contradicted**, **phase-narrowed** (the product description
lost it because v1 defers it, against the operator's ruling that
development order must not shape the product description),
**medium-boundary** (a boundary drawn by what a component touches rather
than by its objective).

Sources swept: `_kos/ideas/usher.md`, `repos.yaml`, `CLAUDE.md`,
`vision.md`, `org-diagram-repo/data/ecosystem.yaml`, the listed findings, ideas,
research, design and runbook files, `session-log.md` (usher entries),
director, marvel, sideshow, the fleet watcher, critic and wardrobe markdown, and
bd (`bd sql` reachable; 15 issues matched). `charter.md` has no usher
mention. sideshow, the fleet watcher, critic and wardrobe markdown has no usher
mention. One marvel hit (`marvel/_kos/probes/probe-fleet-throughput-prior-art.md:434`)
is a false positive: "Packet Pushers" in a reference link.

## Gaps

| # | Source (file:line) | Prior text (short verbatim quote) | Kind | Current status, with where current docs touch it | Suggested home |
|---|---|---|---|---|---|
| 1 | `_kos/ideas/usher.md:14`, `:25`; `repos.yaml:61` | "attach pre-AI services" | function | **missing.** No baseline doc names connecting a newcomer's seat to the pre-AI services the team already runs (issue tracker, repos, chat, identity). | P §4.2 as a feature, or P §9 as an open question on what "pre-AI services" covers |
| 2 | `_kos/ideas/usher.md:16`, `:25`; `repos.yaml:62` | "get repositories when they aren't fluent in git" / "git hand-holding" | function, audience | **missing.** UJ-1 has John "pointed at a repo", but no feature helps a person who is not fluent in git get and keep the repos their seat needs. The audience (non-git-fluent people) is absent from P §2. | P §2.1 (newcomer JTBD) and a P §4.2 feature; decide whether it is setup (usher) by the objective rule |
| 3 | `_kos/ideas/usher.md:24-25`; `repos.yaml:62` | "sideshow/eos bootstrap" | integration | **shrunk.** B and P name sideshow only as an installer usher drives; eos as a bootstrap target is gone, and "bootstrap a seat with packs" is not a stated function. | P §4.2 FR-11 consequences, BA §C row |
| 4 | bd aae-orc-9fwcr module 1; `_kos/ideas/usher.md:14`; `repos.yaml:60` | "the ArcavenAE tools (marvel, director shim, sideshow, kos, aq/ax, [one private tool omitted], tmux-cmc where relevant) and the toolchain they need (go, rust, node, tmux, nats, dolt)" | function | **shrunk.** P FR-9 says "harnesses and fleet tools" with no list; the toolchain layer is absent; "usher may generate or check a .mise.toml" (same ticket) is absent. | P §4.2 FR-9 and harness knowledge (the tool list as data) |
| 5 | bd aae-orc-9fwcr module 1 | "usher owns the agentic-engineering seat: harness settings, MCP registrations, skills and packs (via sideshow), bus and marvel client config, identity and keys the seat needs" | scope | **shrunk.** P covers harness settings and instruction, skill and agent files as metadata (FR-4). MCP registrations, pack state, bus and marvel client config, and the seat's identity and keys (as presence) are not named as parts of an environment. | P §3 (Environment) and §4.1 |
| 6 | bd aae-orc-9fwcr module 3 | "the team's declared seat (a versioned bundle in a repo: harness set, packs, MCP allowlist, settings tweaks, bus and cluster names)" | feature | **shrunk.** P's profile holds settings entries only (§3 Profile, Entry). A declared seat also names the harness set, packs, an MCP allowlist and cluster names. | P §3 and §4.5; P §9 Q1 (format) |
| 7 | bd aae-orc-9fwcr module 3 | "bring a new machine or a new person to it. Re-find your seat: after a machine change, an OS reinstall, or six weeks away, diff current against declared and repair" | feature | **phase-narrowed.** P has comparison and the watcher; "repair" to the declared seat appears only as v2 writes (P §7.2). The product description (P §1, §4) never says usher brings a host back to its seat. | P §1 and a §4 feature, phased in §7 |
| 8 | bd aae-orc-9fwcr module 2 | "drive mise, brew, sideshow, and each harness's own installer to reach a declared seat; idempotent; prints what it would do first" | feature | **shrunk.** P FR-11 installs one missing tool on confirmation. Converging a host to a declared seat, and idempotence, are absent. | P §4.2 FR-11 |
| 9 | bd aae-orc-9fwcr module 4 | "Catch up: show what changed in the team seat since your last sync (settings, packs, skills, MCPs, marvel manifests you consume), with the reason if the change carries one" | feature | **missing.** P FR-14 dates changes to a person's own settings; nothing shows what changed in a shared profile since the person last adopted from it. | P §4.5 (a profile changelog against the adoption record, FR-23) |
| 10 | bd aae-orc-9fwcr module 5 | "with a dry run, a diff, a backup, and a rollback; never overwrite a user file without saying so" | posture | **phase-narrowed.** P §5 states changes on confirmation, but the write-safety rules (dry run, backup, rollback, no silent overwrite) were not written because v1 makes no writes. | P §5 guardrails, stated now for when writes ship |
| 11 | bd aae-orc-9fwcr module 5 | "adopt team tweaks into your local harness configs and your agent teams" | integration | **shrunk.** Adoption into marvel agent teams sits behind P Q2 (usher-marvel seam) and "Marvel policy fragment (v2+)"; the product description does not say adopted settings reach agent teams. | P §2.1 non-human seats; Q2 |
| 12 | bd aae-orc-9fwcr title; `docs/research/2026-10-08-harness-survey/stream-a-prior-work.md:115-118` | "the harness ecosystem (claude code, codex, crush, opencode, pi, toolchain, settings)" | harness set | **phase-narrowed.** P names only the v1 set (Claude Code, desktop app, codex, goose); crush, opencode and pi appear only in PA §4's log. The product intent (the harness list is data, any harness) is implied by "harness knowledge" but not stated as intent in P §1. | P §1 (any harness), §7.1 for the v1 set |
| 13 | `stream-a-prior-work.md:20-23` | "repo-supplied `.crushrc` executed as bash at config load" | posture (observer hazard) | **missing.** PA §1 lists observer hazards for the v1 harnesses only. A harness whose config load executes repo code means usher must never ask that harness to load config. | PA §1 hazards; harness knowledge per harness |
| 14 | `stream-a-prior-work.md:128-134` | "the five answers any fifth harness needs ... (adapter, settings projection, MCP wiring, env to MCP children, seat isolation lever, first-run gate)" | function | **missing.** The fleet already has a per-harness onboarding checklist. The first-run gate (what a new seat must pass before it works) is not in harness knowledge or the setup path; P §6 rules out answering prompts, not knowing about them. | P §3 Harness knowledge; FR-11 |
| 15 | bd aae-orc-9fwcr module 1 | "Reuse the training course repository's doctor logic for backend detection rather than rewriting it" | function | **missing.** P records the backend only for observed effects (FR-27). Which backend and auth route a seat uses (subscription, API key, a cloud provider) is not part of an environment map. | P §3 Environment, FR-1 |
| 16 | `_kos/ideas/usher.md:24`, `:13`; bd aae-orc-9va5 | "updates" / "usher ... is the project that owns install and update posture" | function | **shrunk.** P FR-9 reports a tool as behind its source; FR-11 installs a missing tool. No FR updates an installed tool. aq/ax defers its own updater to usher on this basis. | P §4.2 (an update FR beside FR-11) |
| 17 | bd aae-orc-1vgfk item 4; PA §3 "Unsigned packs" | "Signature and integrity verification on the client: cosign and Rekor verification" | posture | **shrunk.** P Q8 covers signing of profiles and the harness knowledge file. Verifying what usher installs (signature, attestation) before it lands is not stated, though PA §3 lists unsigned packs as a failure mode. | P §5 or FR-11 consequences |
| 18 | `_kos/ideas/usher.md:17-19`, `:28-30` | "which harnesses, on what projects, making what tool calls, with what configuration, what token spend, what errors" / "errors experienced" | function | **shrunk.** P keeps settings, versions, processes and cache reuse. Activity by project, tool-call outcomes and errors experienced appear only as permitted session-record fields (P §5), with no feature that reports them. Needs the objective test: errors that bear on a seat's setup look like usher's; fault detection is the fleet watcher's. | P §4.7 or §4.3, after the objective test (Q16) |
| 19 | `_kos/ideas/usher.md:47-48`; `repos.yaml:67-68` | "wakatime + enterprise configuration management" | positioning | **shrunk.** The wakatime half (passive per-project usage reporting) has no feature; P's "usage" is cache reuse per session. The configuration-management half is reframed (see ruled out). | P §1 positioning; §4.7 |
| 20 | `_kos/ideas/usher.md:28`; bd aae-orc-0mmn notes (F3) | "optionally send telemetry (metrics-only or OTEL)" | deployment form | **shrunk.** P's only egress is the light collector with reported fields; OTEL as a transport, and usher helping a person point a harness's own OTEL export at a self-hosted collector (SOUL §6), are absent. | P §4.8 or §4.10; Q9 |
| 21 | `_kos/ideas/usher.md:32-38`; `repos.yaml:63-65`; `CLAUDE.md:149-152` | "Both feed a cloud service that lives in the same repo" / "template-based compliance for different setups to produce recommendations" | deployment form | **phase-narrowed.** The central service (templates, recommendations, distribution) is a co-equal half of the product in every orc source, but P §1 to §6 describe only the light collector; the central service lives only in P §7.2 and PA §2. | P §1 and a §4 section, phased in §7 |
| 22 | B "The Solution" (Recommendations); origin `_kos/ideas/usher.md:36-38` | "what could reduce spending, what settings they should consider" | feature | **phase-narrowed.** B carries local recommendations ("from what the person's own harness recorded (permission denials, for example)"). P has no recommendation FR; recommendations appear only as a v2 central-service job. Local recommendations need no service. | P §4.4 (a recommendation FR) |
| 23 | `_kos/ideas/usher.md:39-40`; `repos.yaml:65-66` | "an index of dynamic options that extends the client's built-in capabilities, so usher can expand without a client update" | feature | **shrunk.** P FR-31 updates harness knowledge without a release. Extending capabilities (new checks, new harness support, new install recipes) by data is narrower in P than in the origin. | P §4.9 FR-31 |
| 24 | `_kos/ideas/usher.md:42-45`; `repos.yaml:66-67` | "an MCP or other interface within the LLM, a CLI or app UI, possibly a spider" | deployment form | **shrunk.** P Q9 lists usherd, MCP surface, CLI, light collector; the app UI and spider are gone, and no FR describes a person asking their own harness about their seat through the in-LLM surface (only its token budget, P §5). | P Q9; a §4 feature for the in-LLM surface |
| 25 | `session-log.md:1541-1543` | "no cross-AI-tool installer for non-experts" | differentiation | **shrunk.** One of the two whitespace claims from the prior-art probe. B "What Makes This Different" keeps the cross-harness map, the loop and the posture; installing for non-experts is not named as a differentiator. | B "What Makes This Different" |
| 26 | `session-log.md:1544-1545` | "Boxen death + Homebrew-analytics backlash as ranked failure modes" | risk | **missing.** BA §E carries the prior-art rulings but not the two ranked failure modes (an install tool that dies of upkeep; analytics backlash). Both bear directly on SM-6 and FR-29. | P §8 counter-metrics or BA §E |
| 27 | `session-log.md:2354-2356` | "usher may ask more, the user may ask back, and the real constraint is attention, so usher should give more than it takes" | posture | **shrunk.** P §5 "Attention budget. Usher asks only when it has to" keeps the restraint and drops the ruling's other half: the person may ask back, and usher should give more than it takes. | P §5 attention budget |
| 28 | bd aae-orc-0mmn notes; `session-log.md:2364-2365` | "probes against an organization's real usage data run inside that organization's own repositories; only mechanism returns to this repo" / "mechanisms travel, measurements stay where they were taken" | posture (method) | **missing.** P §5 keeps per-person data out of git; the rule on where measurement work runs is not carried. | P §5 or PA §5 |
| 29 | bd aae-orc-z6y7 workstream 1 | "the shim shape (shell wrapper / launchd job / usher-managed) that gets the secret into the harness without landing on disk" | function | **missing**, possibly **contradicted**. B and P say usher holds no credential and reports presence only. A usher-managed shim that brokers a vault-minted value into a harness at launch is brokering, which ADR-009 permits; the baseline neither includes nor rules it out. | P §5 credentials; P §9 as a new question |
| 30 | bd aae-orc-z6y7 workstream 2 | "define exactly what leaves the machine (a boolean-plus-shape finding, not the secret)" | function | **shrunk.** PA §5 keeps the scan off by default with location and type. P FR-38's reported fields do not list credential findings, so whether a shape finding may be reported is unstated. The ticket's scan locations (shell rc files, .env files, other harness configs) are not carried. | P FR-38; PA §5 |
| 31 | bd aae-orc-1vgfk items 3, 7 | "corporate Claude Enterprise users on managed Windows laptops are the likely case" / "WSL2 only ... versus native" | audience, deployment form | **shrunk.** P Q14 keeps Windows open. The likely audience (managed corporate Windows laptops), the WSL2-first option and the client-only split (the service side stays macOS and Linux) are not carried. | P Q14 |
| 32 | `docs/design/2026-10-31-i78-native-signed-tap.md:58`, `:68` | "the agent sandbox and usher are private repos, so the tap job's token cannot read their releases" | deployment form | **missing.** Usher is planned as a formula in the fleet's homebrew tap. P §5 says signed builds; the distribution channel and its private-repo blocker are not stated. `usher/CLAUDE.md` defers gitflow until a distribution channel exists. | P §5 signed builds or Q11 |
| 33 | `org-diagram-repo/data/ecosystem.yaml:58` | "Onboards people / and machines" | audience | **shrunk.** "Machines" as onboarded subjects (a host provisioned to a seat) is close to B's new-machine success criterion but is not named in P §2.1. The "marvel to usher: Onboards" edge is P Q15. | P §2.1 |
| 34 | B "Boundaries with sibling components"; BA §C | "the fleet watcher (session watching), ... critic (fleet aggregation)" | boundary | **medium-boundary.** The brief's parenthetical labels describe media, not objectives; usher also watches (FR-13) and aggregates (FR-41). P §5 and PA §6 were corrected to objectives; B and BA §C still carry the medium labels. | B Scope and BA §C, reworded by objective |
| 35 | bd aae-orc-0mmn notes (F3) | "trace-shaped telemetry and token accounting likely belong to a sibling tool per composition" | boundary | **medium-boundary** and **contradicted.** P FR-26 puts per-session token and cache accounting in usher. The older line assigns token accounting by medium; the objective rule should settle it. | PA §6 table; Q16 |
| 36 | `_kos/ideas/usher.md:80-82` | "configuration reporting (usher) vs configuration management (marvel control plane). Usher observes and recommends; who enforces?" | boundary | **medium-boundary**, partly superseded. The objective ruling makes maintaining settings usher's; B's memlog still records "seat writes are marvel's", a boundary drawn on the write. P Q2 holds the seam. | P Q2, stated by objective |
| 37 | bd aae-orc-lp2pg | "usher (fleet reporting and catch-up, the front-of-house)" / "usher presents the fleet report to the operator" | integration | **missing.** P's cross-host queries serve the expert on settings. An operator-facing fleet report that presents results critic and the fleet watcher produce is not addressed. The ticket's split is partly by medium; the objective test may move it. | P Q16 and §9 |

## Ruled out, for the record

Each item below appears in orc material and was excluded with a recorded
reason in the baseline or its memlogs.

- **Enforcing organization policy; enterprise configuration management as
  enforcement** (`_kos/ideas/usher.md:47-48`). B "Out of the product"; P §6:
  device management's objective. Managed settings belong to that layer
  (PRD memlog, "maintain over manage").
- **"Which users are spending too much"** (`_kos/ideas/usher.md:37`). P §2.2,
  §6 and FR-41: no per-person ranking; SM-C3. Ranking is critic's
  objective.
- **Employer force-enabling reporting** (`_kos/ideas/usher.md:69-75`). Charter
  B1; P §6 "refereeing the employment relationship".
- **Re-implementing mise, brew or vendor installers; managing dotfiles**
  (bd aae-orc-9fwcr). B and P §6, in agreement with the ticket.
- **Reading prompt or file content.** P §6 and §5 (named session fields
  only).
- **Reconciliation as the central bet** (`session-log.md:2340-2344`).
  Replaced by the contribution loop as the position in July; the drift
  check stays as a function.
- **"Drift" as the general term.** Retired in July, returned with a narrow
  meaning (BA §F); P §3 narrows it further.
- **"Never read" secrets.** Changed to presence, never value (BA §F).
- **Session tiles as primary UI; workflow-stage telemetry as-is.** BA §E.
- **Clamp, proxy or gateway; dashboards as the product.** B; P §6.
- **Recovering a running session.** P §6: the fleet watcher's objective.
- **Answering folder-trust or consent prompts on anyone's behalf.** B; P §6.
- **Chat conversations.** P §2.2; sequenced, not dropped (P §7.2).
- **Two July success criteria.** Dropped at PRD gate 5 (PRD memlog); only
  "a team outside the fleet reaches a working seat" was restored.

## Expectations other components have of usher

| Component | Expectation | Source | Carried? |
|---|---|---|---|
| aq/ax | Usher owns install and update posture; ax must not grow its own updater | bd aae-orc-9va5 | B cites it; P has no update FR (gap 16) |
| marvel | Marvel "Onboards" usher (edge direction and meaning unratified) | `org-diagram-repo/data/ecosystem.yaml:80` | P Q15 |
| marvel | Adapter signals should flow through "OTEL, usher's telemetry, critic downstream" | `_kos/ideas/marvel-service-provider-architecture.md:76-78`; `marvel/docs/design/service-provider/03-probes-and-roadmap.md:39-42` | missing (gap 20) |
| marvel | Settings management for agent seats is a marvel projection primitive | `_kos/ideas/marvel-service-provider-architecture.md:61-64` | P Q2 |
| the fleet watcher, critic, director | A split where the fleet watcher detects, critic aggregates and usher presents the fleet report | bd aae-orc-lp2pg | missing (gap 37) |
| legion review loop | "usher tells a reviewer how to sit down at any variant" (agent-grown run repos) | bd aae-orc-yyqp3 item 5 | missing: seating a person at an arbitrary repo is not a stated function |
| key-handling work | Plaintext key detection lives in usher; a usher-managed shim is one option for key delivery | bd aae-orc-z6y7 workstreams 1, 2 | detection carried; shim missing (gap 29) |
| sideshow | Usher drives sideshow for bootstrap rather than replacing it; packs reach the seat via sideshow | `_kos/ideas/usher.md:77-79`; bd aae-orc-9fwcr | install carried; pack bootstrap shrunk (gap 3) |
| sideshow | Per-installation state follows the same "the repo is not the instance" pattern as usher F6 | bd aae-orc-70os | BA §B |
| Windows assessment | A Windows-capable client: inventory, install, find-your-seat, report client | bd aae-orc-1vgfk | P Q14 (gap 31) |
| homebrew-tap | Usher publishes a formula through the fleet tap | `docs/design/2026-10-31-i78-native-signed-tap.md:58` | missing (gap 32) |
| beadle | "beadle tends issues, usher tends humans and harnesses" | `repos.yaml:69-70`; `_kos/ideas/usher.md:65-68` | P Q10 names the overlap but not this proposed line |
| operator ask via director | "distribute and manage permission and auto-mode settings" | `director/sim/notes/relay-log.md:9678` | carried (B lead problem; distribution in P §7.2) |
| wardrobe, critic, the fleet watcher repos | No markdown in those repos mentions usher | grep, 2026-11-04 | n/a |
