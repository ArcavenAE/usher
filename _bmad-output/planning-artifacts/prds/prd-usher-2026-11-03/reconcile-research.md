# Input reconciliation: harness settings research against the usher PRD

- **Input:** `research/technical-harness-settings-observability-map-2026-11-02/research.md` (cited below as R, with section and source number).
- **Checked against:** `prd.md` and `addendum.md` in this folder, both dated 2026-11-03.
- **Date:** 2026-11-03.

Dispositions: **PRD** (add to the PRD, usually as an FR consequence), **addendum** (mechanism detail for the architecture step), **set aside** (with the reason).

## Top gaps

1. FR-12 keys the codex `approval_policy = "untrusted"` retirement to a merge date, not a release, and leaves out that the harness now fails with a hard error (item C1).
2. Q4 gives the earliest re-check as 2026-12-01. The research staleness map says 14 of 27 claims were already past due on the day it was written, and FR-26 runs on dates only, with no re-check when a new release ships (items C2, D1).
3. Neither harness knowledge (glossary, FR-24) nor FR-12 records what a stale key does in each harness (warn, alias, hard error, silent skip), even though the research names silent skipping as the larger hazard (item D2).
4. FR-1 gives one layer and file per setting. Claude lists, and goose `extensions` and `providers`, merge across layers, so those keys need provenance per entry (item C3).
5. FR-2 covers silent non-application only for codex and goose. It leaves out the Claude cases the research documents: first-source-wins managed skipping, entries `claude doctor` rejects, env-variable and settings-key pairs, project-ignored `defaultMode` values, and cache values held back from effect (item D3).

## A. Research recommendations or hazards the PRD drops

**A1. Claude issue #70181 (an empty or 304 server response zeroed local managed deny rules).**
- Research: R §1 "Docs versus behavior" [9]. Closed by the stale bot 2026-10-13 with no linked fix.
- PRD: absent.
- Disposition: **addendum** §1 as a known hazard, plus a candidate finding (managed deny rules present in files but missing from effect). Keep it out of FR-12 until a probe (FR-28) reproduces it on the current version.

**A2. Docs lag source; cite source or changelog first.**
- Research: R "Cross-dimension insights", third bullet [24][26][29][28][12].
- PRD: FR-24 lists vendor docs, changelog, source and probe as equal references. No order and no confidence field.
- Disposition: **PRD**, FR-24 consequence: each item carries a confidence, and an item backed only by docs is marked lower confidence than one backed by source, changelog or probe.

**A3. Relocated homes: `CLAUDE_CONFIG_DIR`, `GOOSE_PATH_ROOT`, CODEX_HOME.**
- Research: R §1 [5], R §3 [27], R §2 [23].
- PRD: absent. FR-1 and FR-5 assume default paths.
- Disposition: **PRD**, FR-1 consequence: when a home-relocating variable is set in the launch environment, the map reads from the relocated home and names the variable as provenance. `CLAUDE_CONFIG_DIR` is ignored in project and local settings, so only the environment counts [5].

**A4. codex AGENTS.md size limit (`project_doc_max_bytes`, 32 KiB default).**
- Research: R §2 [23].
- PRD: FR-4 reports size but raises no finding for it.
- Disposition: **PRD**, FR-4 or FR-12 consequence: an AGENTS.md chain over the limit in effect is a finding. Size is metadata, so this needs no content read.

**A5. Telemetry keys documented but not read.**
- Research: R §3 [28][29]: `GOOSE_DISABLE_TELEMETRY` is in `CUSTOM_DISTROS.md` and not read by the source; the kill switch is `GOOSE_TELEMETRY_OFF`. R §2 [12]: codex analytics exporter on unless `analytics.enabled = false` (unverified beyond docs).
- PRD: the addendum hazards column names the goose kill switch. No FR uses it.
- Disposition: **PRD**, FR-2 consequence: a goose environment setting `GOOSE_DISABLE_TELEMETRY` is reported as declared and not read. The codex analytics default goes to **addendum** as an open item until verified.

**A6. Permission evaluation order: deny, then ask, then allow; first match wins whatever its scope.**
- Research: R §1 [3].
- PRD: FR-12 says "conflicts between layers" without the harness's own semantics.
- Disposition: **addendum**: conflict detection for Claude permissions follows the harness's evaluation order, not layer precedence. This keeps a deny in a lower layer from reading as overridden (it relates to SM-C2, false findings).

**A7. Claude `/status` and `/permissions` are interactive.**
- Research: R §1 "Resolved views" lists them as partial views without saying they can be called from outside a session.
- PRD: §4.1 and the addendum treat them as cross-checks.
- Disposition: **addendum** `[ASSUMPTION]`: whether usher can reach these views from outside an interactive session is unverified. Until it is, the cross-check rests on `claude auto-mode config` and `claude doctor`.

**A8. codex requirements constrain values and fall back with a notice.**
- Research: R §2 "Managed tier" [14][15]. Requirements are tracked apart from the layer stack and can force a compliant value.
- PRD: the addendum names `configRequirements/read`. No FR reports a requirement or a value it clamped.
- Disposition: **PRD**: FR-1 consequence, requirements are listed apart from layers. FR-2 consequence, a value a requirement replaced is reported as declared and not in effect, with the requirement as the reason.

## B. PRD relies on claims the research marks disputed, unverified or overturned

**B1. codex project file ignores `notify` (FR-2 consequence).**
- Research: R §2 "Setting classes" [12]: "Unverified beyond the docs." Whether a startup warning prints is unconfirmed (R Rec 5).
- PRD: a testable consequence built on the docs list.
- Disposition: **PRD**, keep it, but make `config/read` origins the oracle ("`notify` from a project file is absent from `origins`"). Add the key list to FR-28 probes so a release that changes it is caught.

**B2. Folder trust gates which layers load (FR-6).**
- Research: R §2 [12][15]: codex project layers are documented as loading only for trusted projects, and code tying this to trust was not found. Untrusted project skills are disputed [12][21]. R §3: goose has no directory trust.
- PRD: FR-6 states the gating as fact for every harness and has no consequences.
- Disposition: **PRD**, FR-6 consequences: a harness without a trust mechanism (goose) reports "no trust mechanism", never "untrusted". For codex, trust effects are read from `config/read` (`disabledReason`), not inferred, until the disputed test runs.

**B3. Placement says whether a higher layer will override (FR-18).**
- Research: R §2 [14][16]: the codex rank of `config_toml_base64` (MDM defaults) is disputed, and it decides whether MDM beats user config.
- PRD: FR-18 promises a pre-apply override warning for every harness.
- Disposition: **PRD**, FR-18 consequence: for codex, the override prediction rests on `config/read` origins for the target directory, not a hard-coded rank. Where the rank is disputed, the warning says so and FR-19 confirms after the person applies.

**B4. Count of disputed claims.**
- Research: frontmatter `claims_disputed: 6`. The prose names four ($defaults, codex MDM rank, codex untrusted project skills, goose `GOOSE_MODE` default).
- PRD: Q4 and addendum §1 say four.
- Disposition: **set aside** in the PRD. The PRD matches the research prose; the gap is in the research. Ask the research owner for the other two from the claims ledger, then update Q4 if they bear on an FR.

**B5. `$defaults` hazard check.**
- Research: R §1 [4][7], Rec 4.
- PRD: FR-12 already gates it on its test. No gap; listed only to confirm the check was done.

## C. Harness facts the PRD states incorrectly against the research

**C1. codex `approval_policy = "untrusted"` "retired for versions after 2026-08-20" (FR-12).**
- Research: R §2 "Breaking changes" [17]: PR #39630 merged 2026-08-20, and an explicit setting now **fails with an actionable error**. The release that first carried it is not given.
- PRD: a date stands in for a version range, and "retired" does not say the harness refuses to start with it.
- Disposition: **PRD**: key the consequence to the first codex release containing PR #39630 (look it up, FR-24 reference), and give the finding severity "hard error in the target version". R Rec 6 (a profile must not name it) is met by FR-12 and FR-13 once versioned.

**C2. "Earliest dated re-check 2026-12-01" (Q4).**
- Research: R "Staleness map": the earliest re-check is "already due" (goose G1 to G4 and G6 to G9, codex X3 to X8). 14 of 27 claims were past their window on the day of writing. 2026-12-01 is the second row.
- PRD: Q4 gives 2026-12-01.
- Disposition: **PRD**, fix Q4: the earliest re-check is already due, for the goose and codex claims named in the staleness map. Name a re-check owner by role (expiry-is-sell-by rule).

**C3. One layer and file per setting (FR-1).**
- Research: R §1 [1]: Claude lists merge across files, except four model keys (`fallbackModel`, `modelPicker`, `availableModels`, `modelSettings`). `autoMode` entries combine additively across scopes [4]. R §3 [26]: goose `extensions` and `providers` merge per entry.
- PRD: "Each setting names its layer and file."
- Disposition: **PRD**, FR-1 consequence: a merged list or map reports provenance per entry, and the four Claude model keys are reported as replaced, not merged. The research notes that on this host only a diff against the files could attribute the extra auto-mode entries [11], so per-entry provenance for Claude is rebuilt, and FR-1 marks it that way.

**C4. FR-26 puts "layers and precedence" in the slow class.**
- Research: R Rec 3 puts scopes, precedence, file paths and merge rules in the slow layer. The R "Staleness map" windows, though, put `layers` in the one-month class with version, resolved view and effect-only, and precedence and docs-gap at three months.
- PRD: FR-26 follows Rec 3 in prose and names no windows.
- Disposition: **PRD**, FR-26: adopt the research's classes and windows as the starting values (version, layers, resolved-view, effect-only 1 month; precedence, docs-gap 3 months; cadence 6 months), and flag the Rec 3 versus staleness-map difference on "layers" in **addendum** for the architect to settle.

**C5. Glossary "Layer" list.**
- Research: Claude environment variables "are not a layer"; each variable pairs with its key, and dedicated flags override per key [5]. codex has profile, cloud-managed defaults and built-in defaults as layers, and requirements beside the stack [12][15]. goose has additional files [26].
- PRD: "managed, flag, project, user, session, system."
- Disposition: **PRD**, glossary: say that the layer set is per harness and lives in harness knowledge. Name two kinds that are not layers: per-key environment overrides (Claude, goose) and requirements (codex constraints).

## D. Research findings that should shape an FR consequence but do not

**D1. Re-check on release, not only on a date (two-layer staleness model).**
- Research: R Rec 3 and the staleness-map closing note: "Every fast-layer claim should be re-run on each harness release." R "Cross-dimension insights", second bullet: the detected version selects the map version, and a version change triggers re-verification.
- PRD: §5 "Version stamping" states the principle. FR-26 is date-only. No FR says what happens when the detected version is newer than any version the knowledge was verified against.
- Disposition: **PRD**: FR-26 consequence, an item's re-check falls due on its date or when install health detects a harness release newer than its verified range, whichever comes first. FR-3 consequence, a harness version outside the knowledge's verified range is stated on the map, and findings that rest on fast-layer items are marked unverified for that version.

**D2. Stale-key behavior per harness.**
- Research: R Rec 3 ("Record per harness what a stale key does: warn, alias, hard error, or silent skip"). R "Contrary evidence", third bullet: codex keeps deprecated forms with a warning [42] and fails hard on `untrusted` [17]; goose removed lead/worker with no migration note [43]; Claude skips bad entries [39]. "Silent skipping is the larger hazard." Open question: the table is "partly known, not enumerated."
- PRD: glossary "Harness knowledge" lists "hazards" but no stale-key behavior. UJ-1 names silent non-application as the edge case, and no FR picks it up.
- Disposition: **PRD**: FR-24 consequence, each retired or deprecated key item records its behavior (warn, alias, hard error, silent skip) with a reference. FR-12 consequence, a finding on such a key states that behavior, and a silent-skip key is reported even where the harness gives no signal. Add the stale-key table to FR-28 as a probe set and to Q4.

**D3. Silent non-application on Claude Code.**
- Research: R §1 [1][2][5][39]: managed sources are first-source-wins by default and skip the others without warning (`managedSourcesBehavior: "merge"` from v2.1.242). `claude doctor` lists rejected entries. `ANTHROPIC_MODEL` beats `model` in any file, and `--model` beats both. `defaultMode` values `auto` and `bypassPermissions` do not take effect from project or local files. OTEL variables are ignored in project and local settings. About a dozen security keys let a stricter lower-scope value beat managed. Keys outside project scope are ignored in the shared project file.
- PRD: FR-2 lists the classes but gives consequences only for codex and goose. FR-18 covers only `autoMode`.
- Disposition: **PRD**: FR-2 consequences for Claude: a skipped managed source is reported under first-source-wins; entries `claude doctor` rejects are reported as declared and not in effect; a key overridden by its paired environment variable reports the variable. FR-18 consequences: `defaultMode: auto` or `bypassPermissions`, OTEL variables and any key outside project scope are never placed in a project or local file. Put the per-key scope table in **addendum** as harness knowledge.

**D4. Claude server-managed cache can differ from effect.**
- Research: R §1 "Server-managed settings" [39]. Some `env` categories, `modelPricing` and `managedMcpServers` are held until the server confirms. `model` and OTEL exporter changes apply at next launch. Non-interactive runs do not write approval-requiring settings to the cache. `/logout` deletes the cache under API-key or token credentials.
- PRD: addendum §1 says "may differ from effect". No FR consequence.
- Disposition: **PRD**, FR-1 consequence: a value read from `~/.claude/remote-settings.json` in a held-back category is marked "cached, not confirmed in effect". A missing cache is reported as missing, not as "no managed settings", and `claude doctor`'s remote state is the tie-breaker. The category list goes to **addendum**.

**D5. Blind spots named by the research.**
- Research: R Rec 2: Claude feature flags fetched from Anthropic, the prompt cache TTL in force, and codex cloud defaults (whether they are cached locally is unknown, R §2). R §3: goose provenance of `goose info -v` is unknown.
- PRD: FR-3 requires a blind-spot list but no consequence names its known members.
- Disposition: **PRD**, FR-3 consequence: a Claude map lists feature flags and prompt cache TTL as not observed, and a codex map lists cloud defaults as not observed, until harness knowledge says otherwise. This also fixes FR-20's cache-reuse reading, which runs without the TTL in force.

**D6. Staleness window of the observed state itself.**
- Research: R §1 [2]: remote settings poll hourly, MDM every 30 minutes, and settings files hot-reload.
- PRD: FR-20a takes daily snapshots. No FR says a map is a point-in-time reading.
- Disposition: **PRD**, FR-3 consequence: each map records when it was taken. Managed values may lag the source by up to the harness's poll interval, which the map states. **Addendum**: FR-20a dates a change by snapshot, so a change shows to within the snapshot interval plus the poll lag. SM-2's four-day target holds against that.

**D7. codex `config/read` verified only on 0.160.1.**
- Research: R §2 "Resolved views" [11][40]: the schema was read locally on 0.160.1, six minors behind stable 0.166.0 [19]. Without `cwd`, project layers are left out.
- PRD: FR-1 makes `config/read` the codex oracle without a version range.
- Disposition: **PRD**, FR-1 consequence: the codex resolved view is used only for versions whose generated schema (FR-27 `generate-json-schema`) matches the harness knowledge, and otherwise the map falls back and says so. Whether `config/read` starts a daemon or update loop is already in §5 and addendum §1, so it is not repeated here.

**D8. codex deprecated forms and removed profiles.**
- Research: R §2 [18]: legacy profiles were removed in rust-v0.134.0. R "Contrary evidence" [42]: several deprecated forms are kept with a warning.
- PRD: FR-12 names only `untrusted`.
- Disposition: **addendum**: seed list of retired and deprecated codex forms for the first harness knowledge release. This is input to D2.

**D9. goose has no project-level config.**
- Research: R §3 [26][33].
- PRD: FR-5 maps every launch directory without saying how a harness with no directory layer is handled.
- Disposition: **PRD**, FR-5 consequence: for goose, environments differ across directories only by environment and instruction files, and the map says that rather than repeating an identical stack per directory.

## E. Items checked and set aside

- **R Rec 5** (brief says codex "silently ignores" project keys): this is a brief edit, not a PRD one. The PRD wording ("declared and ignored") is already neutral. Set aside here, and carry it with Q13's brief follow-up.
- **R §3 governance** (goose moved to the Agentic AI Foundation, new repo and docs hosts [32][30]): affects where FR-29 maintainer docs point. Set aside for FR-29 authoring; no requirement change.
- **R §3 PR #9439** (malformed goose extension entries deleted on toggle, fixed 2026-06-11): narrower than general config loss and fixed. Set aside; it bears on a write path usher does not use.
- **Addendum §1 observer hazards** for Claude (`claude auto-mode` can reset, `/doctor` can fix, `claude doctor` reads cwd settings without a trust prompt): not in this research, so they cannot be checked here. Set aside for the input that supplied them. §5's guardrail covers them either way.
- **Claude desktop app and general settings (FR-6a):** the research does not cover them. FR-6a's assumption and Q12 stand.
