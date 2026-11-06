---
title: "Reconcile: prior usher docs against the current brief and PRD"
created: 2026-11-04
baseline:
  - _bmad-output/planning-artifacts/briefs/brief-usher-2026-11-02/brief.md (+ addendum.md)
  - _bmad-output/planning-artifacts/prds/prd-usher-2026-11-03/prd.md (+ addendum.md)
sources:
  - CLAUDE.md, README.md, charter.md
  - docs/product-brief-20260726.md (the July brief)
  - _kos/ (kos.yaml, findings 001 and 002, ideas/fleet-settings-profile.md, probes, nodes)
status: for operator decision; nothing here is applied
---

# Prior usher intent not carried by the current brief and PRD

Scope: stated purpose, function, audience, form, integration, posture or
success idea in the prior docs that the current brief and PRD omit, shrink or
contradict, plus places where the current docs narrowed the product
description to fit a v1 milestone. Mechanism already carried by the
addendums, and items ruled out with a recorded reason, are not counted as gaps;
the ruled items are listed at the end.

Line numbers refer to the files as of 2026-11-04. "Brief" is the 2026-11-02
brief; "PRD" is the 2026-11-03 PRD; "July" is `docs/product-brief-20260726.md`.

## Gaps

| # | Source (file:line) | Prior text (short verbatim quote) | Kind | Current status, and where the current docs touch it | Suggested home |
|---|---|---|---|---|---|
| 1 | CLAUDE.md:18-24; charter.md:54-55 (F1); README.md:7-9 | "Both functions report to/through a **cloud service that lives in this repo too**" | form | **Phase-narrowed.** The PRD describes only the light collector (§4.10). The central service appears only in §7.2 as v2 ("grows the light collector to distribute profiles, run cloud-side templates and make recommendations"), so the product description in §1 to §6 has no cloud side beyond receive-and-query. Brief: absent apart from Q9 ("cloud"). | PRD §1 and a §4 feature for the central service (phasing stays in §7.2); brief Solution |
| 2 | CLAUDE.md:20-22; charter.md:79-83 (F4); finding-001:34-36 | "runs template-based compliance for different setups to produce recommendations (spend outliers, cost-reduction options, recommended settings)" | feature | **Missing / phase-narrowed.** The PRD has no recommendation FR at all; §1 says usher "reports facts, flags findings" and people decide. Cloud templates and recommendations appear only in §7.2 (v2). The brief keeps a "Recommendations" bullet (brief:92-94) that the PRD did not carry forward. "Spend outliers" also sits in tension with FR-41 ("no view ranks people") and SM-C3; the operator may want to say which reading holds. | PRD §4 (a recommendations feature, with FR-3 style "what it could not observe"); PRD §6 note on outliers vs ranking |
| 3 | July:439-441; finding-001:17-19; brief addendum:115-117 | "compliance templates are the fidelity channel and must not slip to a late phase" | posture | **Contradicted.** PRD addendum §5 (140-142) carries the obligation "for the central service", which §7.2 puts in v2, so the obligation has slipped to a late phase. PRD §2.2 also says "Usher is not a compliance product", while CLAUDE.md and README frame usher as "enterprise configuration management" with compliance templates. | PRD §5 or §7.1 (say whether local templates or checks carry the fidelity channel in v1); reconcile §2.2 wording |
| 4 | CLAUDE.md:22-24; July:417-419; finding-002:77-87 | "an index of dynamic options that extends the client's built-in capabilities, so usher can be extended without a client update" | feature | **Shrunk.** FR-31 ships harness knowledge (data about harnesses) without a release. The prior idea is wider: new options and capabilities offered to the client from the index. finding-002 kept the "options knowledge index (read-only, signed, human-confirmed at point of use)" as "arguably the product". | PRD §4.9 (widen FR-31 or add an FR) |
| 5 | CLAUDE.md:10; README.md:6; charter.md:60 (F2) | "attach pre-AI services" / "attach supporting services" / "pre-AI service attachment" | integration | **Missing.** Neither document mentions attaching pre-AI or supporting services (for example MCP servers, issue trackers, repos, accounts) during setup. | Brief Solution; PRD §4.2 (install) or a new §4 feature |
| 6 | CLAUDE.md:10; charter.md:61 (F2) | "bootstrap with sideshow or eos" | integration | **Shrunk / missing.** sideshow appears only as one installer usher drives (brief:90, PRD §4.2). Bootstrapping a person's packs or method content through sideshow is not stated as a function; eos is not mentioned anywhere. | Brief Solution; PRD §4.2 and the sibling boundary list in §5 |
| 7 | CLAUDE.md:10-11; charter.md:61 (F2) | "help with repositories for users who aren't fluent in git" / "git hand-holding for repository access" | feature, audience | **Missing.** No repo or git help appears. The PRD's newcomer (UJ-1) is "pointed at a repo" but gets no help with it. The audience of people not fluent in git, and finding-001:27-28's "cross-AI-tool install/config manager for non-experts", are not named in §2. | PRD §2.1 (audience) and a §4 feature, or an explicit non-goal if ruled out |
| 8 | README.md:5-7; CLAUDE.md:9-10 | "install and configure AI tools ... start with correct settings, stay updated" | feature | **Shrunk.** FR-11 installs a *missing* harness or tool; FR-9 only reports "behind". Updating an installed tool on the person's confirmation is not stated as a function. finding-001:22-23 names "hybrid imperative-day-0 + declarative-stay-current". | PRD §4.2 (an update FR beside FR-11) |
| 9 | finding-001:21-24, 43-46 | "curated-catalog + thin-client is the winning shape"; "install side knows desired state, observation side knows actual state" | feature | **Shrunk.** Profiles carry settings entries; there is no team-declared tool set or tool version (a desired state for what should be installed). FR-18 answers "stale copy of tool Y" only against each tool's own source. | PRD §4.2 or §4.5 (allow profile entries for tools and versions) |
| 10 | CLAUDE.md:12-16; charter.md:66-68 (F3) | "monitor harness configuration and activity ... projects, tool calls, token counts where possible, options set, errors experienced" | feature | **Shrunk.** Activity reporting is reduced to cache reuse per session (FR-26). Errors experienced, tool calls and token counts per project are not observed effects or reported fields (FR-38), although §5 already allows parsing tool outcomes and error markers from session records. | PRD §4.7 (observed effects) and FR-38 reported fields |
| 11 | CLAUDE.md:14-15 | "know-your-users enterprise services: who is running what software on what projects" | feature, audience | **Shrunk.** PRD §1 and FR-18 cover who runs which harness and version on which hosts; "on what projects" is not carried. The enterprise know-your-users framing is absent. | PRD §1 and FR-18 / FR-41 |
| 12 | CLAUDE.md:13; charter.md:65-76 (F3); finding-001:31-34; July:489-491 | "optionally send telemetry (metrics-only or OTEL)"; "Claude Code exports first-party OTEL and orgs can force it via managed settings" | form, integration | **Missing.** The wire shape (metrics-only vs OTEL) is neither decided nor listed as an open question, and OTLP ingestion of first-party harness telemetry is not mentioned. July Q8 gave a direction (metrics-first; trace-shaped telemetry and token accounting likely a sibling tool), not a ruling. | PRD §9 (open question), or §6 if the sibling direction is ruled |
| 13 | CLAUDE.md:34-36; charter.md:52-55 (F1) | "a (mac/windows) service `usherd`, an MCP or other in-LLM interface, a CLI or app UI, possibly a spider" | form | **Shrunk.** PRD Q9 lists usherd and the watcher, MCP surface, CLI and light collector. The app UI and the spider are dropped without a recorded reason. The product description (§1, §4) names no client forms; §5 refers to "every in-LLM surface" without describing one. | PRD Q9 (restore app UI and spider), and a short forms line in §1 or §4 |
| 14 | CLAUDE.md:26; README.md:11 | "Think wakatime + enterprise configuration management" | intent | **Missing.** The positioning line is gone from both documents. The wakatime half implies per-project activity reporting, which links to gaps 10 and 11. | Brief Executive Summary or Vision |
| 15 | charter.md:95-114 (F6); July:469-473 | "a **bare-org deployment approach** that (a) associates a usher installation with its service instance, and (b) collects the instructions/customizations belonging to that instance/team" | form, feature | **Missing.** Neither document says how a person's usher finds its team's instance, or how team customization layers over shipped defaults. The light collector is "hosted by a team", but enrollment and the team-instance artifact are not described. Q1 (profile location) covers only part of it. | PRD §4.10 or a new feature; PRD §9 if left open |
| 16 | July:104-118; July:197-203; finding-002:27-35 | "Usher turns one person's hard-won configuration knowledge into the team's baseline, and keeps doing it as the ground shifts"; "The moat is the loop plus the posture" | intent | **Phase-narrowed (PRD).** The brief keeps the loop as one of two things that "exist nowhere" (brief:101-107). The PRD carries the contribution loop only in §7.2 (v2+) and has no differentiation section, so its product description omits the stated differentiator. The many-environments ruling removed the single baseline, not the loop. | PRD §1 (state the loop as product intent; phasing stays in §7.2) |
| 17 | July:115-118 | "install, analyze, and recommend are primary. Collect and report are secondary and instrumental." | intent | **Shrunk.** The PRD has install, map and compare; "recommend" is absent (see gap 2). The priority order itself is not stated. | PRD §1 |
| 18 | July:347-351 | "The recurring reason to return is a recommendation that is worth acting on, pushed rather than waited for ... Usher must create the occasion" | success, posture | **Missing / contradicted in tone.** The PRD watcher sends change notices, "batched and quiet by default" (FR-13). Nothing pushes a recommendation worth acting on. | PRD §4.3 or the recommendations feature from gap 2; §8 success idea |
| 19 | July:379-381; charter.md:76-77 (F3) | "The observed person sees their own data and recommendations first. Rollups are aggregate; per-user drill-down is an explicit governance decision." | posture | **Shrunk.** FR-39 lets a person request and delete their data; FR-41 bars ranking people. The "own data first" view (honest-security personal page) and "rollups aggregate, drill-down a governance decision" are not stated. The light collector answers per-host questions (FR-18) with no governance step. | PRD §4.10 (FR-41) and §5 |
| 20 | July:382-384; charter.md:24 | "Loud first-run disclosure, DO_NOT_TRACK honored in personal contexts" | posture | **Shrunk.** §5 has a first-open disclosure of local observation and FR-39 a disclosure before reporting. DO_NOT_TRACK is not mentioned. | PRD FR-29 or FR-39 |
| 21 | July:273-275; finding-002:47-52; val-reporting-off-by-default:52-60 | "The member can ask usher what it is doing, check their consent state, change it, or withdraw it ... a partnership with a two-way channel"; "usher returns more than it takes" | posture | **Shrunk.** PRD §5 "Attention budget. Usher asks only when it has to" keeps the budget but drops the two-way channel (the person asking usher what it is doing, at any time) and "returns more than it takes". | PRD §5 |
| 22 | July:404-407; val-reporting-off-by-default:15-17, 55-57; F7 node:35-36 | "whether an enable was member-chosen or organization-required"; "orgs that want force-enable act through their own visible channels (MDM, managed settings)" | posture | **Shrunk / possible contradiction.** FR-39 records consent events but not who made the enable. FR-37 says "turning reporting on is the person's act on that host", which reads narrower than B1, where an organization may enable through its own MDM channel and usher records that it did. | PRD FR-37 and FR-39 |
| 23 | _kos/nodes/frontier/question-local-signal-validity.yaml:51-66 | "Which model was selected per session, and whether it changed mid-session ... Inter-turn timing relative to cache lifetime ... Context reset events ... Cache-invalidating inputs, such as image uploads ... project-context file size over time" | feature | **Shrunk.** FR-26 gives cache reuse share per session. Model thrashing, inter-turn timing against the cache lifetime (the node calls it "the single most actionable local signal"), image-upload resets and context-file size over time are not observed effects. Brief addendum E names only the last. | PRD §4.7 (named observed effects, each diagnostic) |
| 24 | July:26-29; July:337-342 | "several harnesses (Claude Code, Claude Desktop, ChatGPT, Codex, Crush, OpenCode) over several backends (enterprise plans, cloud platform offerings, API keys, local inference)"; "reversible backend profiles and local inference" | feature, audience | **Missing.** Backend setup and switching (API key vs plan vs cloud platform, local inference) is not a usher function in either document; backends appear only as a label on observed effects (FR-27). | PRD §4.2 or a new feature; §7.2 if sequenced |
| 25 | July:343-345 | "Chat products are sequenced later ... Their cohort is expected to be the largest in time" | audience | **Phase-narrowed.** PRD §7.2 says "Chat products. Sequenced, not dropped", but §2 target users never name the chat-product cohort, so the eventual largest audience is missing from the product description. | PRD §2.1 |
| 26 | July:207-213 | "Existing scripts, justfiles, and skills are content in their current costume: their logic migrates into usher's catalog ... harvested from tooling that already passes CI" | feature | **Missing.** Neither document says that existing fleet tooling is harvested into usher's catalog or knowledge. (The retirement criterion beside it is in the ruled-out list.) | PRD §4.9 (harness knowledge sources) or brief Solution |
| 27 | July:152-156; brief:224-226 | "The loop generalizes past configuration to skills, prompts, and workflows, where distribution ecosystems already exist but graduation does not" | intent | **Missing in PRD.** The brief Vision keeps it; the PRD §1 vision does not. | PRD §1 |
| 28 | July:228-230; contribution-loop node:64-67 | "Cross-organization sharing of findings, done in a way that respects user sovereignty, would be upside rather than premise"; "should be architecturally anticipated" | intent | **Missing.** Not in either document. | PRD §7.2 (later) or brief Vision |
| 29 | fleet-settings-profile.md:49, 59 | "an organization that wants it locked: a `managed-settings.d/` drop-in, written by that organization's own MDM, never by usher"; "Usher should be able to emit a drop-in for an organization to deploy" | feature, integration | **Missing.** The PRD treats device management only as something to detect (FR-12) and as the enforcement owner (§6). Producing an artifact an organization's MDM can deploy is not mentioned. | PRD §4.6 or §7.2 |
| 30 | fleet-settings-profile.md:51; brief:92-94 | "From the denials a host recorded (`/permissions` Recently denied, or a `PermissionDenied` hook), usher drafts candidate entries and marks each as a proposal" | feature | **Missing in PRD.** The brief carries it ("Recommendations ... permission denials, for example"); the PRD has no FR for proposals drawn from a host's own records. | PRD §4 (with gap 2) |
| 31 | fleet-settings-profile.md:65 | "an allow entry is a loosening, and the profile records it as one" | posture | **Missing.** Profile entries carry a label, value, range and note (PRD §3 "Entry"), with no marking of entries that loosen a control. | PRD FR-21 or the Entry glossary item |
| 32 | July:416-419; finding-002:85-87 | "rule out remote configuration application until it has a designed trust model" | posture | **Shrunk.** PRD §7.2 lists "remote apply" for v2 "on the person's confirmation" with no trust-model precondition. Q8 (signing) touches part of it. | PRD §7.2 |
| 33 | July:302-312 | "Some things that shape outcomes, including model access and feature availability, are server-side and never readable on disk at all; those are observable only by their effects ... what confounds each observable signal" | feature | **Shrunk.** Harness knowledge (PRD §3) records layers, keys and defaults; the setting class (directory, user-wide, server-side) and the confounds per signal are not named items. FR-3 blind spots covers part. | PRD §3 "Harness knowledge" and FR-30 |
| 34 | July:364-373 | "Validate on one managed machine before any breadth." | success | **Missing.** §5 keeps signed builds and a behavioral profile; the step of validating on one managed machine before breadth is gone. | PRD §7.1 or §8 |
| 35 | July:424 | Non-goal: "One configuration file understood identically by every harness." | posture | **Missing.** Not in PRD §6 or brief Scope. | PRD §6 |
| 36 | July:495-496 | "Whether usher is a product, a protocol, or a service. Deliberately deferred" | intent | **Missing.** Not carried as an open question. | PRD §9 |
| 37 | charter.md:69-70 (F3) | "a helper and a tattletale of the benign, runner/messenger kind; it must not become a creepy peephole" | posture | **Shrunk.** The substance is in §5 (derived state, named fields only); the posture statement itself is gone. Low priority. | Brief "What Makes This Different" or PRD §5 |
| 38 | fleet-settings-profile.md:69 | "Identity gaps such as org membership are reported, not fixed." | feature | **Shrunk.** PRD §6 rules out fixing identity gaps but does not say usher reports them. | PRD §6 note or FR-16 findings list |

## Ruled out, for the record

These prior items were changed or dropped by a recorded decision, so they are
not gaps. Listed so the operator can see each was a choice.

- **Observation may not begin before consent** (charter.md:127-128 F7;
  July:472-473; F7 node:32-34, "Proposed answer: no"). Ruled the other way,
  operator 2026-11-04: local observation starts at install, including silent
  MDM installs, and nothing leaves the host (PRD §5; PRD memlog line 45).
  Note: charter F7 still states the old corollary.
- **Secrets are never read** (July:329-331). Amended to a presence-only scan,
  off by default, location and type only (brief memlog line 10; brief
  addendum F).
- **"Drift" retired** (July:11-12; finding-002:27-35). Returned with a narrow
  meaning (brief memlog line 12; brief addendum F; PRD §3).
- **Divergence from a single team baseline** (July Bet; brief Solution).
  Replaced in the PRD by the many-environments model with comparison in any
  direction (PRD memlog lines 16-17). Brief follow-up is PRD Q13.
- **V1 read-only as product posture** (July:316-323). Operator 2026-11-04:
  read-only is a phased MVP goal, not product intent; configuration writes
  are within the objective and deferred to v2 (PRD memlog lines 41-43; PRD
  §7.2).
- **Install and analyze before collect** (July:316). Operator added the light
  collector to v1 (PRD memlog line 31).
- **Retirement of parallel surfaces as a success criterion** (July:214-220;
  brief:147-149) and, it looks like, **one team setting per quarter through
  the divergence path** (brief:145-146). PRD gate 5: "restore only the first
  dropped success criterion ... the other two stay dropped" (PRD memlog line
  37). The memlog does not name the two; these are the two brief criteria
  with no PRD counterpart. The brief still carries both.
- **Enterprise versus individual taxonomy** (charter.md:118-121). Retired in
  favor of individuals and organizations; carried in the brief (brief:130-131).
- **Several affiliations per person** (July:279-301; charter F6). Deferred to
  later, one affiliation per light collector in v1 (PRD memlog line 35); the
  instance model stays design direction in PRD addendum §5.
- **Configuration writes, render `--apply`, marvel policy fragment**
  (fleet-settings-profile.md:46-49, 76-78). Sequenced to v2 and v2+ (PRD
  §7.2; PRD addendum §4).
- **Agent and instruction file content** (implied by July "analyze"
  breadth). Operator: metadata only, no content and no optimizing (PRD memlog
  line 15; PRD §6).
