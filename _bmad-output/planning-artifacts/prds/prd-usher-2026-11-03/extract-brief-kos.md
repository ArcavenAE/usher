# Extract: brief, addendum and kos sources for the usher PRD

Source abbreviations: brief = `_bmad-output/planning-artifacts/briefs/brief-usher-2026-11-02/brief.md`; add = `.../addendum.md`; idea = `_kos/ideas/fleet-settings-profile.md`; charter = `charter.md`; B1node = `_kos/nodes/bedrock/val-reporting-off-by-default.yaml`; F7node = `_kos/nodes/frontier/question-org-admissions-and-locked-settings.yaml`; F8node = `.../question-contribution-loop-sustainability.yaml`; F9node = `.../question-local-signal-validity.yaml`; F10node = `.../question-settings-discovery-across-directories.yaml`; f002 = `_kos/findings/finding-002-design-review-and-evidence-check.md`.

## 1. Vision and target users

- Vision: usher helps a person find their seat across AI harnesses, re-find it when the ground moves, and share what works with the team; it recommends, it does not take over (brief:13-17).
- Long-run vision: front of house for any team running AI harnesses; the team profile becomes a living baseline fed by members; the loop later extends to skills, prompts and workflows (brief:213-219).
- Lead problem is settings and permissions friction; cost (prompt caching) is the second payoff (brief:19-25; add:131-133).
- Customer: the fleet engineer, the AI advocate, who adopts usher for a team, owns the team profile, and gains standing when seats work and spend falls (brief:115-117; f002:18-22).
- Primary user: the developer, who finds and re-finds their seat and adopts team settings safely; may install usher or receive it via device management; value lands without knowing usher's internals (brief:118-121).
- Non-human seats (third subject): agent sessions launched by marvel get settings through marvel's per-session projection; usher reports on them, marvel writes for them (brief:122-124; idea:12).
- Beneficiary: the organization, which pays less for more and sees what is in use without seeing content (brief:125-126).
- Axis is individuals and organizations, not enterprise versus individual; a person may hold several affiliations, each with its own consent (brief:128-129; charter:118-121; f002:37-45).
- Evidence base: this fleet plus at least one unnamed outside team (brief:131-132; add:147-148).
- Non-users: silent as a named group. Nearest statements: identity gaps such as org membership are out of scope (idea:27, idea:69); usher does not referee the employment relationship (F7node:43-47).

## 2. Glossary candidates

- Drift: installed or resolved state falling behind what was declared, with no upside (brief:70-71; add:20-21). History: retired in July because there was no fixed baseline (f002:27-35), returned with a narrow meaning once a declared profile exists (add:139-141).
- Divergence: a person off the team baseline, which may be an improvement; the contribution loop's input (brief:71-72; add:17-21).
- Team profile (fleet profile): a declared, versioned, operator-authored file per fleet instance holding labeled prose entries, each with ruling and date (idea:10, idea:45; brief:82).
- Resolved view: the harness's own computed effective configuration; "a file saying X does not mean X is in effect" (add:171-173; brief:76-77).
- Claim marker: committable artifact at a scope root, scope names only (add:54-55).
- Private binding: local-only artifact paired with the claim marker (add:54-55).
- Scope: unit of attribution; scope resolution assigns each observation to an affiliation; attribution follows scope, not credential (add:53, add:60-62).
- Affiliation: one of a person's N organization memberships served by one daemon, isolated by a stated mechanism (add:53, add:58).
- Three units: attribution (scope), benefit (person), decision (organization) (add:61-62).
- Seat: silent as a formal definition; used as a working harness setup for a person (brief:13) and as a marvel-launched agent session (brief:122-124).
- Instance: the team's deployment of usher; the repo and its tools are not the instance (charter:95-102).
- Label: the text before the first colon of a profile entry, so a rerun replaces rather than duplicates (idea:45).
- Managed-only keys: the only settings that truly lock, e.g. `allowManagedPermissionRulesOnly`, `disableAutoMode` (add:64-67; idea:31).
- `$defaults`: marker whose absence from an auto-mode list discards the built-in rules, including the auto-mode bypass rule (idea:33).
- Two layers of staleness: slow (scopes, precedence, paths) and fast (keys, defaults per release) (add:174-176).
- Per-harness observability map: which settings are directory-scoped, user-wide or server-side per harness (brief:228-230; F9node:110-113).
- Cache efficiency score: fraction of turns served from cache versus paid at full write cost, per session (F9node:68-73).

## 3. Feature and capability candidates

### Settings map
- For each supported harness and each directory the person launches from, show where settings live across managed, user, project and live-session layers, and which value is in effect (brief:74-77).
- Prefer the harness's own resolved view where one exists (brief:77-78; add:171-173).
- Codex: report effective config, every layer with a disabled reason, and the layer behind each key; also admin constraints (add:156-160).
- Claude Code: rebuild the documented stack from files, read the server-managed cache (which may differ from effective state), confirm against the harness's own views (add:161-165).
- Goose: rebuild the stack from source (system file, additional files, user file, env ahead of files), including the system layer the docs omit (add:166-170; brief:35-36).
- Each reading names the harness version it assumes (brief:80-81; add:125-127).
- Distinguish point-in-time file reads from live-session state, since a session snapshots env at launch (F10node:25-28; charter:160-162).
- Discover settings across every launch directory, not only the user file and current directory (F10node:18-23, F10node:30-35).
- Report derived state (which pins diverge, which directories carry overrides), not content (F10node:33-35).

### Drift check against a team profile
- Profile shape: one versioned file per fleet instance, operator-authored, in an operator-controlled repo; labeled prose entries with ruling and date (idea:45).
- Profile sections: `claude.autoMode.environment`, `claude.autoMode.allow`, `claude.permissions.ask/deny`, `codex.approval_policy` and `sandbox_mode` per seat class, `trust` intent per path class (idea:45).
- Compare each host's resolved state, not files alone, with the profile (brief:82-84; idea:50).
- Entry statuses per brief: present, missing, stale (brief:84-85). Per idea: missing, extra, stale label, `$defaults` absent (idea:50); first slice prints present, missing, stale plus `$defaults` presence (idea:76).
- Structural hazards checked per harness version, e.g. an allow list that drops built-in defaults, still to be confirmed on current Claude Code (brief:85-86; add:177-179).
- Structural check refuses any rendered list omitting `"$defaults"`; structural gate permitted under SOUL section 8 (idea:66).
- A profile must not name codex `approval_policy = "untrusted"` (retired) (add:182-184).
- Answer "which hosts carry entry X" (brief:140-141; idea:78).
- Smallest first slice: drift check for Claude auto mode only, read-only, one host, profile seeded from five existing entries, no writes, no network, no marvel dependency (idea:76).

### Install and update health
- Report per tool: installed or not, source, version, and whether behind its source (brief:87-88).
- Drive existing installers (mise, brew, sideshow, `tools/install.sh`, vendor installers); do not re-host (brief:89-90).
- Scope covers the harnesses and the ArcavenAE tool set (brief:160-161).
- Detect failure modes named in the problem: self-update wedging a running seat, installed copy drifting from merged source, doubly tapped formula silencing staleness checks, a version check reading only the installed file (brief:49-55).
- Detect MDM reinstalls that drop custom profiles (brief:55-56, brief:177-179).
- Answer "which hosts run a stale copy of tool Y" (brief:141).
- Version detected here keys the fast-layer re-verification of the settings model (add:175-176).

### Recommendations
- Proposals drawn from the team profile and from what the harness recorded, e.g. permission denials (brief:91-92; idea:51).
- Each recommendation declares what it could not observe (brief:92-93; F9node:103-105).
- Recommendations are proposals for the operator, never applied (idea:51).
- Cloud-side template compliance and recommendations on spend outliers, cost reduction, settings (charter:79-82).
- Options knowledge index kept: read-only, signed, human-confirmed at point of use (f002:84-87).

### Divergence offered back
- When a person's settings differ from baseline in a way that works better, draft the difference as a team proposal with credit (brief:94-96).
- Body generated from the observed difference; the person adds only intent (add:22-24; f002:216-221).

### Other
- Per-session cache efficiency score as first cost-side output, in v1 scope (brief:162; F9node:68-73).
- Local cost signals: model per session and mid-session change, inter-turn timing versus cache lifetime, context resets, cache-invalidating inputs, context-file size over time (F9node:49-66; add:119-123).
- Detect throttled or failed calls so unbilled throttling is not read as efficiency (F9node:88-93).
- Credential scan: report location and type of credential-shaped material, off by default (brief:182-186).
- Report a seat held at a prompt, without answering it (idea:67).
- Consent inspection: member can ask what usher is doing, check, change or withdraw consent; consent events recorded (B1node:52-60).
- Personal page where the user sees their own data (charter:77).
- Onboarding: install and configure harnesses, pre-AI services, correct initial settings, sideshow or eos bootstrap, git hand-holding (charter:58-63).
- Emit a `managed-settings.d/` drop-in for an organization to deploy, never deploy it (idea:49, idea:59).

## 4. Cross-cutting constraints and NFR candidates

- Read-only v1: usher writes nothing to harness configuration; proposals only (brief:167-170).
- Never the sole configuration authority; seat settings are marvel's, locks are the organization's MDM (brief:168-170).
- Observing must not change the harness: each resolved-view call shown not to write state, refresh a cache, or trigger self-update before usher relies on it (brief:171-176).
- MDM coexistence: detect and report overlap rather than fight it (brief:177-179).
- Derived state, not content: report that a value or file exists, never ship files (brief:180-181).
- Credentials presence-only: never retain or transmit values; scan off by default; holds no bearer authority at any third party (brief:182-186).
- One credential per host: install flow provisions or prompts per host, never copies between hosts; usher holds none (brief:187-189).
- Reporting off by default (B1); the off switch gates the code path, not just the send (charter:9-31; B1node:26-29).
- Egress tests: one proving no egress in default state, one proving enabled state sends only the published inventory (B1node:29-33; brief:190-191).
- Published "we do NOT collect" list is a contract (B1node:33-35).
- Collector self-hosted first (brief:191-192).
- Signed and notarized builds, explicit outbound traffic, published behavioral profile (brief:193-194; B1node:37-42).
- Open source so posture claims can be checked in the code (brief:108-111).
- Per-person data never enters git; client names stay out (add:101-102).
- Version stamping: every check stamps its assumptions; fast layer re-verified per release (add:125-127, add:174-176; brief:80-81).
- Precedence model must track vendor implementations per harness and version (F10node:47-50).
- Price map completeness is a correctness requirement (F9node:82-87).
- Compare within backend cohorts; record backend in use (f002:63-67; add:122).
- Coarse records (daily milestones on key settings) as a privacy and cost feature (F9node:106-109).
- Attention is the scarce resource; ask only when affordable, return more than taken (B1node:58-60; f002:47-52).
- Every in-LLM surface carries a token budget from day one (add:116-117).
- Independence: each sibling works without usher and vice versa; the seam is a file format, not a call (brief:205-209; idea:70).
- Platforms: macOS first, Linux likely, Windows unknown (brief:163). Charter names usherd as a mac/windows service (charter:52).
- Method: probes against an organization's real data run in its own repos; only mechanism returns (F9node:115-120; f002:188-193).

## 5. Explicit non-goals and out-of-scope

- Configuration writes, remote apply, enforcement through the daemon (brief:198).
- Answering prompts (folder trust, consent) on anyone's behalf (brief:199; idea:67).
- Dashboards as the product; a clamp; proxy or gateway spend enforcement (brief:200).
- Re-implementing mise, brew, sideshow or vendor installers; managing dotfiles (brief:201-202).
- Chat products in v1, sequenced not deprioritized (brief:203).
- Remote configuration application until it has a designed trust model (f002:86-87).
- Loosening a control the operator did not choose; adding entries not operator-authored (idea:65).
- Removing built-in rules (idea:66).
- Judging (e.g. whether an org is trusted) (idea:68).
- Bypass channel: a post-render denial is reported, never answered with a wider entry (idea:71).
- Fixing identity gaps such as org membership (idea:27, idea:69).
- Refereeing the employment relationship (F7node:43-47; charter:13-18).
- Rejected prior art: session tiles as primary UI; workflow-stage telemetry as-is (add:112-113).
- Treating divergence as decay to clamp, i.e. the compliance category (brief:105-106).

## 6. Later-phase (v2+) material

- Contribution loop full cycle: detect, declare, describe, assess (named reviewer, two-week clock), enrich and test, decide, ripple, credit (add:20-36). V1 ripple is proposal only (add:34-35).
- Render into user layer: v2 (brief:167-168; add:35). Idea's slice two is user-layer render by label-keyed merge with backup and temp-file rename (idea:46-48, idea:78).
- Marvel policy fragment: idea's slice three, needs marvel's agreement on the seam (idea:47, idea:78).
- Cache score: listed in v1 scope (brief:162) but its value as a cost signal needs F9's four to six week window (brief:149-150; F9node:97-99).
- Aggregate-and-return, minimum substrate, self-hosted first (brief:233); cloud collector and templates (charter:53-55, charter:79-82).
- Cross-organization sharing treated as upside, not premise (F8node:64-67).
- Loop extended to skills, prompts, workflows (brief:216-218).
- Chat products after v1 (brief:203).

## 7. Success criteria and measurable signals

- A team outside this fleet installs usher and reaches a working seat without our help (brief:136-137).
- A developer on a new or updated machine reaches a working seat on all three harnesses without asking anyone (brief:138-139).
- Fleet engineer answers "which hosts carry entry X" and "which hosts run a stale copy of tool Y" from usher (brief:140-142).
- At least one team setting per quarter arrives through the divergence path; diagnostic, not a gate (brief:143-144).
- Parallel surfaces retired because nobody reaches for them; observed, never enforced (brief:145-147).
- Second payoff: tie a cost change to a settings or habit change within days, not a billing cycle (brief:148-150).
- Two-week review clock as the point an unreviewed contribution breaks the loop (add:27-29; F8node:33-36).
- Predicted natural contributors: two to five (possibly one to three) in a team of twelve to twenty-five (add:40-41; F8node:26-27).
- Four to six week minimum evidence window for local signals (add:122; F9node:97-99).
- Baseline cost of the current method: a new script, review and three host runs per change, no report of which hosts ran it (idea:61).

## 8. Open questions

- Brief Q1: where the team profile lives and its format (F6) (brief:223-224; idea:85).
- Brief Q2: usher-marvel seam, profile fragment or role files; two `--settings` are last-wins (brief:225-226; idea:82).
- Brief Q3: discovery method and consent for a home-wide sweep (F10, question-settings-discovery-across-directories sub-q a, b) (brief:227; F10node:39-46).
- Brief Q4: per-harness observability map; disputed claims need one test each; re-check per release (brief:228-230; add:177-181).
- Brief Q5: does a team of a dozen to two dozen sustain the loop (F8, question-contribution-loop-sustainability) (brief:231-232).
- Brief Q6: minimum substrate for aggregate-and-return (F4) (brief:233).
- Brief Q7: admissions, org-required settings, pre-acceptance disclosure (F7, question-org-admissions-and-locked-settings) (brief:234-236).
- Brief Q8: catalog and profile signing, rotation, revocation, ADR-009 test for usher-minted keys (brief:237-238).
- Brief Q9: component decomposition (F1) (brief:239; charter:50-56).
- Brief Q10: name, backronym, beadle overlap (F5) (brief:240; charter:85-93).
- Brief Q11: going public: time, license, sweep for private names, client names, per-person data (brief:241-243).
- question-contribution-loop-sustainability a to e: ROI misalignment at review and adoption; minimum governance and one busy reviewer; credit without promotion system; is three contributors to twenty consumers success; cross-org friction (F8node:52-67).
- question-local-signal-validity e to i: do signals predict cost per unit of work; which are observable as derived state; confounds; record coarseness; does the observability map have to exist first (F9node:97-113).
- question-org-admissions-and-locked-settings a to f: lockable setting classes; disclosure contents; inspect, change, withdraw consent; incompatible demands on user-wide singletons; observation before consent on MDM install; forensic record of org-required enable (F7node:21-36).
- question-settings-discovery-across-directories a to d: discovery method (filesystem versus session logs); scope and consent; precedence modeling home; relation to F4 and F9 (F10node:39-53).
- Charter F3 remainder: wire shape (metrics-only versus OTEL), signal inventory and its complement, first-run disclosure UX, user's own-data view (charter:73-77).
- Charter F2: boundary with sideshow and fleet-bootstrap packs (charter:62-63).
- Charter F6: minimal bare-org artifact, how an individual's usher finds its instance, how team customization layers over defaults (charter:104-114).
- Idea Q2: does a `--settings` entry show the same `claude auto-mode config` result as a user-file entry (idea:83).
- Usher absence from the independence matrix and the unratified "marvel to usher: Onboards" edge need a decision (add:82-84).
- Disputed harness claims: `$defaults` omission on Claude Code 2.1.317; codex MDM config rank; codex project skills in untrusted repos; goose `GOOSE_MODE` default (add:177-181).
- Unconfirmed: whether codex warns at startup when ignoring project keys (add:185-186).

## 9. Assumptions and contested points

- [ASSUMPTION] Tying cost change to settings change within days needs F9's four to six week window before it can be judged (brief:149-150).
- [ASSUMPTION] Install may be silent; observation may not begin before consent; proposed, not ruled (brief:235-236; F7node:32-34; charter:127-128).
- Untested core claim: cost efficiency varies between people with identical access and is driven by local configuration and habit (F9node:10-13).
- Withdrawn claim: multiple-fold per-person spreads in one fleet's reporting; data too new, a shared quota change dominated, flat-fee seats invisible, caching figure disclaimed (f002:153-168; F9node:23-32).
- Caution: billing-side efficiency gains can be artifacts of throttling (F9node:88-93).
- Caution: a price table missing a model silently drops its tokens (F9node:82-87; add:120).
- Caution: use the token audit's method, not its figures (add:95-96).
- Caution: cache-TTL research read from one Claude Code version; one value is server-controlled (add:125-126).
- Contested and resolved: server-side entitlements as confounder was risk telescoping; compare within cohorts (f002:59-67).
- Contested and resolved: observability catalog cost argues for carrying it collectively, not descoping (f002:69-75).
- Contested and resolved: dynamic-options index kept as knowledge; apply path removed (f002:77-87).
- Contested: the loop is the moat, collectors are table stakes; the review argued collectors (f002:89-94; brief:100-106).
- No empirical minimum team size for a sustainable loop exists either way (F8node:38-41; f002:204-206).
- Near-zero authoring cost may change small-team arithmetic; unproven (F8node:43-48).
- Organization locks are weaker than they look: auto-mode allows are additive and can override an org soft deny (brief:45-47; add:64-68; idea:33).
- Research base: of 27 tracked claims, 15 verified, 6 unverified, 6 disputed, 2 overturned (add:152-154).
- The `$defaults` structural hazard is still to be confirmed on current Claude Code (brief:85-86), yet the idea treats it as a refusing structural gate (idea:66).
- Server-managed cache `~/.claude/remote-settings.json` may differ from effective state (add:163-164).
