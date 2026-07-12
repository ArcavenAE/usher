# Prior-Art Landscape — Usher's Two Functions

Date: 2026-07-12
Probe: `_kos/probes/brief-prior-art-landscape.md`
Method: internal-artifact mining + two parallel web-research passes
(one per function). Web sections synthesized from agent research runs;
URLs spot-checkable.

Usher's two functions, restated:

1. **Front-of-house onboarding** — self-service install/configure/
   update of AI tooling (Claude Code, marvel, codex, …), pre-AI
   service attachment, correct settings, sideshow/eos bootstrap, git
   hand-holding.
2. **Harness observation** — inventory/versions/configuration +
   usage/token telemetry from every harness on a machine, reported
   centrally; template-based compliance producing spend and settings
   recommendations.

---

## Part 0 — Internal prior art (pennyfarthing lineage)

**Borrowing posture: intentional, not inherited.** BikeShop/BikeShow/
RaceCoach were designed for a different project with different
assumptions — pennyfarthing was a team-visibility dashboard product
for developers voluntarily running a BMAD-workflow console, single
harness (Claude Code), TypeScript/Electron ecosystem, telemetry as
the product. Usher's context differs on most axes: onboarding +
enterprise know-your-users, multi-harness, invisible-after-install,
compliance/recommendations as the product, stack undecided. Per orc
charter F16 (assumption provenance), the pennyfarthing findings are
true *for that implementation*; each item below is a **candidate
borrowing that requires a deliberate accept/adapt/reject decision at
bmad design time**, not a default.

The artifacts (designed with Keith Avery; bikeshop-prd.md authored
2026-02-11, revised by M. Pursifull):

### The pipeline: Claude CLI → WheelHub → BikeRack → BikeShop

- **WheelHub** — client-side Express/WebSocket server with an OTLP
  receiver; Claude Code's native OTEL flows into it. An
  implementation detail of BikeRack (a terminology correction that
  itself became a drift finding — the original PRD drew the
  component boundary wrong).
- **BikeRack** — standalone panel viewer for CLI-first developers:
  WheelHub without the conversation UI. 16 WS channels, 13 dashboard
  panels (sprint, git, diffs, workflow, audit, acceptance criteria,
  …), file watchers on `.session/`, `sprint/`, git state. Claude
  stays in the terminal; the dashboard is "a companion, not a
  replacement."
- **BikeShop** (PRD, 46 requirements, 8 user journeys, 8 design
  reviews) — the team aggregation server. Phase 1 "Team Awareness":
  Docker-deployed service ingesting enriched telemetry from many
  BikeRacks over a typed WS protocol; **ShowRoom** dashboard with
  real-time session tiles (agent, story, workflow stage, duration,
  context %); **BikeShopPass** OpenSSH-keypair auth with
  per-identity revocation; **automated push delivery** (three
  system-generated channels: framework changelog on version change,
  sprint catch-up on connect, live sprint events; no operator
  authoring UI); **connection resilience** (local cache + flush on
  reconnect, zero workflow disruption). Phase 2: analytics,
  BikeShow multi-session composed views, topic-based routing,
  onboarding bundles. Phase 3: REST API, cross-shop casting,
  failover, pattern recommendations ("teams using workflow X on
  this story type complete 30% faster"), leaderboards.
- **RaceCoach** (PRD stub — the "bike coach") — a coaching layer
  over the telemetry stream: three-tier rule sources
  (human-curated rules → behavioral linter → AI-driven), five
  feedback categories (context-window efficiency, workflow step
  ordering, persona suggestions, agent action triggers, document
  timing). Operating principle: **"It advises; the rider
  decides."** Open questions it recorded that transfer verbatim to
  usher: who authors rules, where feedback surfaces, how AI-tier
  advice avoids being annoying or wrong, privacy of team-visible
  coaching.

### What the kos probe found wrong with that design (seam findings)

The mvg-bootstrap probe nodes (kos `_kos/probes/mvg-bootstrap-nodes/`)
recorded two gaps that usher must not reproduce:

1. **po-seam-drift-vs-telemetry — "BikeShop monitors everything
   except spec drift."** Elaborate execution visibility (who, what,
   how fast, stuck-or-not) with no channel for "the implementation
   diverged from the spec." Execution telemetry and fidelity
   assessment are orthogonal investments; BikeShop bought only the
   first. Usher's compliance-template function is precisely the
   second — configuration fidelity ("your setup diverged from the
   recommended template") — so usher is in part the answer to this
   seam, and should say so in its design.
2. **po-seam-context-vs-bikeshop — context-as-free vs
   context-as-constraint.** BikeShop's push delivery treats agent
   context as free (push sprint state, changelogs, notifications on
   connect) while the same project's research argued context is the
   binding constraint. Usher's dynamic-options index and any
   in-LLM (MCP) surface must carry a context budget from day one.

### Candidate borrowings (each needs an accept/adapt/reject at design)

| BikeShop/RaceCoach decision | Possible usher translation | Portability note |
|---|---|---|
| Typed protocol + local cache + flush-on-reconnect | usherd → cloud transport; offline-tolerant | Likely portable — general distributed-client pattern, not pf-specific |
| Per-identity keypair auth, revocation without team disruption | enrollment/identity model for usherd | Pattern portable; mechanism (OpenSSH keys) is a pf-era choice — enterprise context may want existing IdP/MDM identity instead |
| Push channels are system-generated, no authoring UI in MVP | recommendation delivery templated, not freeform | Portable as an MVP-scoping discipline |
| Broadcast-all + naive local filter first; topic routing later | fleet messaging simplification | Pf-scale assumption (teams of 5–10); usher's fleet size unknown — re-derive |
| "Zero-interruption visibility" | manager-side value prop | Framing portable, but pf assumed voluntary adoption by dashboard-curious devs; usher's enterprise KYU context changes the consent story entirely |
| "It advises; the rider decides" (RaceCoach) | recommendation posture; aligns with SOUL.md §8 | Portable — and independently required by aae-orc values, so this one is a platform constraint, not a borrowing |
| Session tiles / ShowRoom dashboard as primary UI | — | Probably NOT portable: telemetry-as-dashboard was pennyfarthing's product; usher's product is compliance + recommendations, dashboards secondary |
| BMAD workflow-stage telemetry (story, sprint, workflow phase) | — | NOT portable as-is: assumes every observed session runs a BMAD-shaped workflow; usher observes arbitrary harnesses |

---

## Part 1 — Self-service installer / onboarding landscape

*(web research pass, 2026-07-12)*

### 1a. Dev-machine bootstrap tools

| Project | Shape | Feature decisions | Status |
|---|---|---|---|
| [thoughtbot/laptop](https://github.com/thoughtbot/laptop) | Imperative idempotent bash, individual dev | Curated baseline + user-owned `~/.laptop.local` extension point; no catalog, no phone-home; update = re-run | Alive, low-churn |
| [Strap](https://github.com/MikeMcQuaid/strap) (McQuaid) | Bash + tiny web app minting personalized script after GitHub OAuth | Delegates package state to Homebrew Bundle `Brewfile`; credential-assisted setup; what GitHub replaced Boxen with | Alive |
| [Boxen](https://github.com/boxen/boxen) | Puppet-based declarative fleet manager for dev Macs | Centralized conformity; 330 module repos | **Dead (2016) — most instructive corpse in the set** |
| [omakub](https://omakub.org/) (DHH) | One-command opinionated Ubuntu setup | Taste-as-a-feature; no fleet mgmt, no drift correction | Alive, trendy |
| [chezmoi](https://www.chezmoi.io/) | Declarative Git-backed dotfile manager | Templates + secrets; `chezmoi apply` reconciles = real drift correction for dotfiles | Alive, dominant |
| Nix + devbox/devenv | Hermetic declarative environments | Reproducibility at the cost of conceptual load; wrappers tame the language | Alive, growing |
| mise / asdf / proto | Polyglot runtime version managers | Per-dir TOML activation; mise emerging default; scope = tool versions only | Alive |

**Boxen's retirement is the load-bearing lesson.** Three assumptions
broke: it assumed it was the *sole* config method (users installed via
Homebrew independently, undermining its authority); it assumed users
would *understand* it (output was "confusingly non-actionable"); it
assumed *deterministic* operations (flaky networks made "rerun boxen"
the standard fix). Enforced conformity was resented and routed around.
The replacement (Strap + Homebrew Bundle + dotfiles) won by
**decomposing into small tools with explicit user-owned extension
points** — centralized declarative fleet governance lost to imperative
simplicity + user autonomy.

### 1b. Org self-service portals

- **[Backstage](https://backstage.io/)** — software catalog
  (`catalog-info.yaml` entities) + Software Templates (parameterized
  scaffolding wizard). Plugin architecture, self-hosted, **~2+ FTE
  standing maintenance cost**. Managed alternatives (Port, Cortex,
  Roadie, OpsLevel) trade extensibility for time-to-value; Cortex's
  **scorecards** (standards/maturity dashboards) rhyme with usher's
  compliance templates.
- **Jamf Self Service / Kandji Auto Apps / Intune Company Portal** —
  MDM "internal App Store": server-defined catalog of approved apps +
  patch policies users trigger on demand. Kandji packages, hosts, and
  auto-patches 200+ titles with per-app policy
  (auto-install / available / update-only-if-present).
  Curated-catalog-as-a-product.
- **[Munki](https://github.com/munki/munki) + AutoPkg** — OSS managed
  software center: client reads a **server-hosted manifest**,
  reconciles installed vs desired, self-heals; AutoPkg automates
  repackaging upstream software into the catalog. **The reference
  design for "thin client + server catalog + desired-state
  reconciliation"** — the closest structural analog to usher's
  dynamic-options index + usherd.

The split: IDPs (Backstage-class) optimize day-0 scaffolding +
standards visibility; MDM self-service optimizes curated install +
patch + policy. **Usher spans both.**

### 1c. Dev environment provisioners

- **devcontainer.json** — declarative container manifest, de-facto
  interchange format, portable across Codespaces/DevPod/etc. **The
  spec outlived vendors** — Gitpod Classic shut down Oct 2025
  (pivoted to Ona, AI-agent platform) while the portable-manifest +
  self-hosted models (Coder, DevPod) survived.
- **Shopify `dev`** — internal onboarding tool; core value =
  first-day readiness, including copying GitHub credentials so
  non-experts can clone. The git hand-holding precedent.
- **Coder** — self-hosted control plane, Terraform-defined workspace
  templates owned by the platform team; RBAC/SSO/audit/quotas.

### 1d. AI-tool-specific install/config managers

- **[Claude Code managed settings](https://code.claude.com/docs/en/admin-setup)**
  — first-party enterprise policy: hierarchical settings with an
  un-overridable enterprise layer; delivery via admin console, macOS
  MDM (`com.anthropic.claudecode`), Windows GPO/Intune
  (`HKLM\SOFTWARE\Policies\ClaudeCode`), or file drop
  (`managed-settings.json` via Ansible/Chef/shell); status tab shows
  each setting's source. **The precedence + delivery model usher must
  interoperate with, not reinvent.**
- **Community fleet/config tools** — multi-agent fleet CLIs
  (fleet/clawhub), dotfiles-style AI-config repos (my-ai-tools),
  and hand-curated awesome-lists of agents/skills/MCP configs. The
  catalog layer exists today **only as awesome-lists** — no
  productized cross-tool index/installer.
- **ollama / LM Studio** — background service + server-side model
  registry (`ollama pull`); the daemon+catalog shape usher's
  usherd + index mirrors.

**Finding:** the AI-tool space has a first-party policy story (per
vendor) and a grassroots catalog culture, but **no productized
cross-tool install/update/config-bootstrap manager for non-experts.
That is usher's open lane.**

### 1e. Self-update frameworks & dynamic-capability patterns

- **Sparkle** — RSS appcast + EdDSA-signed feed
  (`SURequireSignedFeed`): a compromised update server can't redirect
  users; server-side appcast = new versions without client changes.
- **TUF** — role-based signing with M-of-N thresholds, versioned +
  expiring metadata; defends key compromise, rollback, mix-and-match.
  The gold standard when the *index itself* is a trust boundary.
  Sigstore uses TUF for its own roots — pairs with the
  cosign/Sigstore posture sideshow already has.
- **winget / Chocolatey / Homebrew / VS Code Marketplace** — curated
  catalog + thin client: the dominant pattern for extending a
  client's options without shipping a new client.

### 1f. Patterns and lessons (function 1)

1. **Curated catalog + thin client wins** for
   extension-without-client-update. Prefer a signed, versioned
   manifest the client reconciles against.
2. **Hybrid imperative/declarative:** imperative guided bootstrap for
   day-0 (Strap/laptop — don't make day-0 require understanding a
   declarative model); declarative reconciliation for staying current
   (chezmoi/Munki). Build the stay-current loop from the start —
   bootstrap-tool rot (great day-0, silent divergence after) is the
   #1 failure mode of the class.
3. **Portable manifest as open contract** — the index/pack schema
   should be an interchange format, not a client-internal one
   (devcontainer.json outlived Gitpod).
4. **Interop with vendor policy layers** — deliver INTO Claude Code's
   managed-settings/MDM precedence chain, don't build a parallel
   policy store.
5. **Sign the feed, not just the payload** — if the cloud index can
   flip client behavior, the index is a trust boundary: TUF-style
   threshold-signed expiring metadata, above a single cosign
   signature.
6. **Boxen's grave rules:** never assume sole config authority (treat
   out-of-band installs as legitimate); every failure actionable and
   resumable; explicit user-owned extension points.
7. **Catalog maintenance is a standing cost** (Backstage 2+ FTE;
   AutoPkg recipe upkeep) — automate upstream repackaging and lean on
   existing package managers rather than re-hosting everything.
8. **Client works degraded when the cloud is unreachable** (Gitpod
   churn lesson) — which is also SOUL.md component independence.

---

## Part 2 — Telemetry / inventory / configuration assessment landscape

*(web research pass, 2026-07-12)*

### 2a. Developer activity telemetry

- **WakaTime** — SaaS coding-time tracker. Editor/IDE plugins send
  heartbeats; a background CLI (`wakatime-cli`) buffers locally and
  batches to the server. Collects project, language, editor, OS,
  host, branch, file, durations. Privacy: can obfuscate/exclude file
  and project names. Team dashboards, leaderboards, goals. The
  documented ingest API spawned a self-hosted ecosystem.
  https://wakatime.com
- **wakapi / hakatime** — self-hosted WakaTime-compatible backends
  (Go / Haskell). Reuse WakaTime's editor plugins and heartbeat API
  unchanged; only the ingest server is yours. Lesson: **a documented
  ingest protocol lets a client population outlive any one backend.**
  https://github.com/muety/wakapi
- **ActivityWatch** — FOSS, fully local. The most usher-relevant
  client architecture in this group: local server (`aw-server`) +
  pluggable **watchers** (`aw-watcher-window`, `aw-watcher-afk`,
  community watchers for editors/browsers) reporting over localhost
  REST. Local SQLite, never phones home, sync opt-in. A near-exact
  template for a usher client hosting per-harness collectors.
  https://activitywatch.net
- **RescueTime** — SaaS automatic tracker (desktop daemon). Logs
  active app, window title, trimmed URLs; explicitly does NOT
  capture keystrokes, form input, screenshots, page content — and
  **publishes the non-collection list prominently**. Good model for
  "state what you don't collect" as a trust lever.
  https://www.rescuetime.com/privacy
- **Software.com Code Time** — free editor-plugin telemetry feeding
  SaaS flow/focus dashboards and team rollups.
  https://www.software.com/code-time

### 2b. Endpoint inventory + config compliance

- **osquery + Fleet (fleetdm)** — the reference architecture for
  usher's inventory function. osquery agent exposes the OS as SQL
  tables; Fleet distributes scheduled/live queries, defines
  **policies** (SQL query → pass/fail), ships a **policy-template
  library** (CIS benchmarks), labels for host segmentation, GitOps
  config. OSS core + paid tiers, self-hostable. Policy-as-SQL maps
  cleanly to usher's template-based compliance. https://fleetdm.com
- **Kolide (now 1Password)** — the single most philosophically
  relevant reference. osquery-based, wrapped in **"Honest
  Security"**: when a device fails a check, Kolide notifies the END
  USER directly (Slack) with a plain-language explanation and
  self-remediation steps, instead of silent enforcement. Tenets:
  informed consent, transparency, educated users make rational
  decisions, expect personal use. This is usher's
  helper-not-surveillance posture, already validated in market.
  https://www.kolide.com/blog/what-is-honest-security-and-why-is-it-important
- **Jamf / Intune / Kandji** — MDM compliance suites: continuous
  inventory, baselines/blueprints, remediation — but
  enforcement-first, IT-owned. The incumbent posture usher
  differentiates *against*.
- **Chef InSpec** — compliance-as-code: declarative `control` blocks
  assert desired state, emit pass/fail; CIS/STIG profile packs;
  remediation separate. The declarative-template model for usher's
  config-assessment rules; contrast with osquery's SQL-inventory
  model. https://github.com/inspec/inspec

### 2c. AI-harness / LLM usage + spend observability (most usher-adjacent)

- **Claude Code native OpenTelemetry** — the keystone precedent.
  Opt-in via `CLAUDE_CODE_ENABLE_TELEMETRY=1`, OTLP export. Metrics:
  session count, lines of code, PRs, commits, **cost.usage (USD)**,
  **token.usage**, tool-decision, active time. Events: prompts,
  tool results/decisions, api_request/error/refusal, MCP server
  connection, permission_mode_changed, skill/plugin activity, hook
  execution, compaction. Resource attributes bake in session.id,
  user.id/email, organization.id, app.version. Crucially, **an org
  can FORCE telemetry via MDM-distributed managed-settings files**
  (user cannot override), and `OTEL_LOG_USER_PROMPTS` /
  `OTEL_LOG_TOOL_DETAILS` gate content capture (off by default).
  First-party config-assessment + spend signal for Claude Code with
  zero scraping. https://code.claude.com/docs/en/monitoring-usage
- **Anthropic admin/usage-cost + Claude Code Analytics API** —
  server-side pull: per-user daily sessions, LoC, commits, PRs, tool
  accept/reject rates, per-model cost; Enterprise console rollups.
  Architectural limit: **vendor admin APIs only see that vendor** —
  Anthropic's API can't see codex/Copilot/Gemini.
  https://platform.claude.com/docs/en/manage-claude/usage-cost-api
- **ccusage / claude-usage (community CLIs)** — the log-scraping
  precedent: Claude Code writes **local JSONL usage logs regardless
  of plan**; these tools parse them locally. Codex CLI writes local
  session logs too — local observation generalizes cross-harness.
  https://github.com/ryoppippi/ccusage
- **LiteLLM proxy** — inline gateway: routes 100+ models, tracks
  spend per key/user/team, **enforces budgets and can block**.
  Sees only traffic routed through it; a control point, not passive
  observation. https://docs.litellm.ai/docs/proxy/cost_tracking
- **Helicone / Langfuse / OpenMeter / Portkey** — request-level LLM
  observability via SDK/proxy: traces + cost/latency/tokens,
  dashboards, alerts. Record-and-explain, not enforce; they observe
  API calls, not local harness config. https://langfuse.com
- **DX (getdx) / Jellyfish / LinearB / Faros AI** — enterprise
  engineering-intelligence measuring AI adoption + ROI; DX has a
  Claude Code connector pulling the Analytics API. Vendor API-pull +
  Git/Jira metadata, not local observation.
  https://docs.getdx.com/connectors/claude-code
- **Nudge Security** — shadow-AI discovery via OAuth/email graph
  (not endpoint): finds AI apps, MCP integrations, shadow agents,
  AI spend. Governance posture: guardrails + user nudges over
  blocking — same helper stance as Kolide. Complementary discovery
  angle to usher's endpoint view. https://www.nudgesecurity.com

### 2d. CLI telemetry precedents + consent norms

- **DO_NOT_TRACK** (`donottrack.sh`) — de-facto standard env var for
  CLI telemetry opt-out (GitHub CLI, Netlify, netdata). Honoring it
  is table-stakes for credibility.
- **Homebrew analytics** — the cautionary tale: opt-out anonymous
  analytics on by default → sustained backlash (esp. enterprise/gov
  objecting to ANY egress without consent); eventually
  `HOMEBREW_NO_ANALYTICS` + DO_NOT_TRACK + louder disclosure.
  **Opt-out-in-silence burns trust even with anonymous data.**
- **.NET CLI / Next.js** — opt-out but with loud first-run
  disclosure + one-line opt-out command; this pattern survived
  scrutiny. **VS Code** — graduated levels (off/crash/error/all).
  **Scarf** honors DO_NOT_TRACK.

### 2e. Patterns and lessons (function 2)

**Architecture ranking for observing local AI harnesses:**

1. **Local observation (daemon + per-harness collectors) — the
   right primary.** Read on-disk logs (Claude Code and codex both
   write local JSONL), tail config files, consume OTEL where offered.
   The ONLY architecture that sees every harness plus versions,
   config, MCP servers, errors. Proven shapes: ActivityWatch's
   local-hub-plus-watchers × osquery's inventory agent.
2. **OTEL collector, org-forced** — usherd as OTLP endpoint, org
   pushes managed settings via MDM. Richest first-party signal for
   Claude Code; per-vendor only.
3. **Proxy/gateway (LiteLLM-style)** — spend enforcement add-on,
   requires rerouting; misses local config/inventory. Optional layer.
4. **Vendor admin API pull** — cleanest per-vendor cost truth, but
   siloed. Reconciliation/enrichment source, not the core sensor.

**Invisible after install:** low-footprint signed daemon
(launchd/systemd/MDM), local buffering + batched upload, degrade
silently offline, observe passive signals (logs/OTEL/config files)
rather than injecting into harness processes so harness upgrades
don't break collection.

**Consent that avoids backlash:** Kolide-style honest security (the
observed person sees their own data + gets plain-language
self-remediation) + CLI hygiene (DO_NOT_TRACK, loud first-run
disclosure, published "what we do NOT collect" list, content capture
off by default — mirror `OTEL_LOG_USER_PROMPTS=0`).

**Shipped shapes of "template-based compliance + recommendations":**
Fleet policies (SQL → pass/fail + CIS template library), InSpec
controls (declarative assertions + regulatory packs), Kolide checks
(fail → user-facing remediation steps). Usher's version: templates
assert desired harness config ("MCP server X allowed," "telemetry
enabled," "model tier within budget," "version ≥ N"), evaluate
against observed state, emit per-user findings + recommendations.
Spend recommendations follow the LiteLLM/DX pattern: attribute cost
per user, flag outliers, suggest cheaper model/settings.

**Table-stakes:** background low-footprint agent; local buffer +
batched central reporting; mac/win/linux; DO_NOT_TRACK + first-run
disclosure; metadata-not-content default; self-hosted option (the
wakapi/ActivityWatch/Fleet expectation for anything
enterprise-security-adjacent); OTLP ingestion for Claude Code.

**Differentiating:** cross-harness local observation (no vendor API
or single-vendor dashboard does this); config/version assessment
against templates (Fleet/InSpec-for-AI-harnesses doesn't exist as a
product yet); Kolide-style user-visible findings; unified
cross-vendor spend-recommendation engine.

**The whitespace:** no shipped product combines (1) cross-harness
local observation, (2) AI-harness config compliance templates, and
(3) honest-security user-facing recommendations. Each exists
separately (ccusage/OTEL; Fleet/InSpec; Kolide). **Usher's
whitespace is the union.**

---

## Part 3 — Synthesis for usher

### The two whitespace claims (one per function)

1. **No productized cross-tool install/update/config-bootstrap
   manager for AI tooling aimed at non-experts.** Per-vendor policy
   (Claude Code managed settings) and awesome-list catalogs exist;
   the product doesn't.
2. **No shipped product combines cross-harness local observation +
   AI-harness config compliance templates + honest-security
   user-facing recommendations.** ccusage/OTEL, Fleet/InSpec, and
   Kolide each hold one third.

Both whitespace claims share one root: everyone else is either
per-vendor (Anthropic's own tooling) or per-concern (install XOR
observe XOR advise). Usher's bet is the union across vendors and
across the two functions — the install side knows what SHOULD be on
the machine; the observation side knows what IS. That closed loop
(desired state ↔ observed state) is exactly Munki/chezmoi
reconciliation and exactly Fleet policy evaluation, applied to a
domain neither serves.

### Architecture convergence (both research passes agree)

- **Client:** one background daemon (usherd — signed
  launchd/Windows-service, MDM-deliverable) hosting per-harness
  collectors (ActivityWatch watcher pattern) and a reconciliation
  loop against a server manifest (Munki pattern). Passive
  observation: on-disk JSONL logs, config files, OTLP ingestion for
  harnesses that export (Claude Code natively; usherd can BE the
  org's OTLP endpoint, forced via managed settings). No process
  injection — harness upgrades must not break collection.
- **Cloud:** signed, versioned catalog/manifest (dynamic-options
  index) + telemetry collector + template-based compliance engine
  (Fleet policy / InSpec control shape) + recommendation delivery.
  The index is a trust boundary → TUF-class feed signing, not just
  payload cosign.
- **In-LLM (MCP) surface:** genuinely novel vs all surveyed prior
  art — onboarding as an in-agent capability (agent drives
  clone/install/config conversationally for git-shy users). Carries
  the pennyfarthing context-budget warning: this surface must be
  token-frugal by design.
- **Offline/degraded:** local buffer + batched upload + flush on
  reconnect (BikeShop DR-4 pattern, independently confirmed by
  wakapi/ActivityWatch); client fully functional without the cloud
  (SOUL.md independence + Gitpod churn lesson).

### Posture constraints (violate any of these and the product reads as spyware or Boxen)

1. **Honest-security default:** the observed person sees their own
   data and gets the recommendations FIRST (Kolide). Manager rollups
   are aggregate; per-user drill-down is a governance decision made
   explicit, not a default.
2. **Consent stack:** loud first-run disclosure (.NET/Next.js
   pattern), DO_NOT_TRACK honored in any personal-mode context,
   published "what we do NOT collect" list (RescueTime), content
   capture off by default (`OTEL_LOG_USER_PROMPTS=0` mirror).
   Enterprise force-enable exists (managed settings precedent) but
   is the org's explicit act, not usher's default.
3. **Never sole config authority:** treat user-installed tools and
   out-of-band config edits as legitimate signals to reconcile
   around, not violations to stamp out (Boxen).
4. **Advise, don't judge:** recommendations are proposals
   (RaceCoach "it advises; the rider decides" = SOUL.md §8 =
   Kolide self-remediation). Enforcement, where an org wants it,
   routes through the vendor's own policy layer (managed settings),
   not through usherd.

### Feature tiers for bmad elicitation

**Table-stakes (from shipped precedent):** idempotent resumable
guided bootstrap; git-clone/credential assistance; signed
catalog + signed payloads; self-update via signed feed; low-footprint
daemon, buffered batched reporting, mac/win(/linux); OTLP ingestion;
self-hosted server option (enterprise-security-adjacent tools are
expected to offer it — wakapi/Fleet norm); interop with Claude Code
managed-settings precedence.

**Differentiators (the whitespace):** cross-harness inventory +
spend in one view; config-drift detection/reconciliation for AI-tool
state (chezmoi-for-settings.json — nobody does this); compliance
templates for AI harness config with per-user honest-security
findings; unified spend recommendations (outliers, cheaper
model/settings); MCP onboarding surface; desired↔observed closed
loop between the two functions.

**Explicitly deferred/optional:** proxy/gateway spend enforcement
(LiteLLM-class — a control point, different trust posture);
dashboards as product (that was pennyfarthing's bet, not usher's);
vendor admin API pull (enrichment/reconciliation source only).

### Sharpened open questions (feed charter F1–F5)

- F1 (decomposition): evidence favors usherd + cloud + MCP surface;
  the "spider" is probably a usherd collector mode, not a separate
  part. CLI/app UI thinnest possible — most UX lives in first-run +
  the honest-security personal page.
- F2 (onboarding boundary): usher DRIVES sideshow/brew/vendor
  installers (Strap→Homebrew delegation pattern); it must not
  re-host or re-package what a package manager already owns
  (catalog-burden lesson). AutoPkg-style automation applies only to
  the thin usher-index layer.
- F3 (telemetry shape): metrics-first (OTEL where offered, log/config
  scraping otherwise); events/content gated off by default. The
  personal-vs-enterprise consent split is the design center, per
  posture constraints above.
- F4 (compliance/recommendations): policy-as-template with a shipped
  library (Fleet/InSpec/CIS shape), findings rendered per-user first
  (Kolide shape), spend attribution per user/team (LiteLLM/DX shape).
- F5 (name/persona): "usher" survives the research — front-of-house
  role covers both seating (onboarding) and quietly keeping order
  (observation). Beadle boundary: beadle tends issues; usher tends
  humans and harnesses.

### Failure modes this design must dodge (ranked)

1. Boxen death (conformity + opaque failure) — highest risk on the
   install side.
2. Homebrew-analytics backlash (silent opt-out telemetry) — highest
   risk on the observation side.
3. Bootstrap rot (day-0 tool, no stay-current loop).
4. Catalog burden (index becomes a 2-FTE distro).
5. BikeShop seam #1 (execution telemetry without fidelity
   assessment) — usher's compliance templates ARE the fidelity
   channel; don't let them slip to Phase 3.
6. BikeShop seam #2 / context-as-free (MCP surface + push delivery
   without a token budget).
