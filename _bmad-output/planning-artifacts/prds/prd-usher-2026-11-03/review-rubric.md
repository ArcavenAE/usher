# PRD Quality Review: usher (prd-usher-2026-11-03, updated 2026-11-05)

Reviewed against `bmad-prd/assets/prd-validation-checklist.md`, covering `prd.md` and `addendum.md` as they stand after FR-43 to FR-66 were appended. Calibration: internal tool built inside one fleet first (§0), chain-top (feeds architecture and epics). Operator rulings cited in the PRD (objective, boundaries, the §7.1/§7.2 split, Q21 to Q23) are taken as given; findings are about contradictions, testability and gaps.

Operator-rule checks, run by grep over both files: no em dashes; none of the banned words; `v1`, `v2` and `MVP` appear in `prd.md` only inside §7 (lines 1236 to 1275). The addendum uses `v1`/`v2` in §2 and §4 (see Downstream usability). FR-1 to FR-66 are all present, each heading unique.

## Overall verdict

The thesis and the environment model hold, and the new §4.16 is the most testable part of the document: FR-63 and FR-64 name concrete failure classes and addendum §7 gives one injected test per row. What is at risk is that the new change-safety promises are written for writes usher performs itself, while the MVP's only changes are installs, updates and clones driven by third-party installers, and the journal's backup copies collide with the credential and content rules. Two scope leaks (FR-38 citing deferred FR-53/FR-54; addendum §2 deferring recommendations that §7.1 ships) and an SM-2 target resting on open Q24 should be settled before epics.

## Decision-readiness: adequate

Decisions are stated as rulings with dates and sources: the objective (§1, addendum §6), boundaries by objective (§5), the FR-44 to FR-66 split "ruled by the operator on 2026-11-05" (§7.1), and Q21 to Q23 each record a ruling. §7.2 says what was given up ("Usher's objective includes them; v1 defers them"), which is an honest trade-off line. No `[NOTE FOR PM]` callouts; with the operator ruling directly, that is acceptable.

The weak spot is SM-2, whose target depends on an input the PRD has not decided how to obtain.

### Findings
- **medium** SM-2's target rests on open Q24 (§8 SM-2, §9 Q24, FR-14). SM-2 says the four-day target is "reachable because FR-14 dates each settings change against the service operator's daily usage and cost", but FR-14 only says the change "can be lined up with" that data, and Q24 leaves ingesting the operator's usage and cost APIs open on ADR-009 custody grounds. If usher cannot hold an admin key, the person lines the two up by hand, and the target may not follow. *Fix:* state in SM-2 or FR-14 which path the MVP assumes (person exports the console's daily figures, or a brokered key per FR-66, which is deferred), or tag the target `[ASSUMPTION]` pending Q24.
- **low** Q21, Q22 and Q23 are ruled, not open (§9). Each reads "Ruled 2026-11-05". Keeping them in Open Questions inflates the open count and makes a reader check each one. *Fix:* move them to a short "Resolved" list, or to addendum §4's resolution log.

## Substance over theater: adequate

Personas are few and each drives FRs (Newcomer: FR-46, FR-22, FR-24; Expert: FR-19 to FR-21, FR-41). UJ-1 and UJ-2 carry the numbers SM-1, SM-3 and SM-4 use. Most §5 NFRs are product-specific and testable (the observation invariant with its zero count, the named-fields test with marker text, the egress tests).

### Findings
- **low** Two §5 lines are slogans with no bound (§5 "Attention budget", "Two-way channel"). "Usher asks only when it has to" and "Usher returns more than it takes" cannot be checked, and the token budget for in-LLM surfaces has no figure here or in FR-62 ("Every in-LLM surface has a token budget"). *Fix:* give the budget a number or a placeholder `[ASSUMPTION]`; cut "returns more than it takes" or turn it into a check (for example, every prompt usher raises names what the person gets for answering).

## Strategic coherence: adequate

The arc is clear: one environment model serves inventory, map, comparison, profiles, placement, recommendations and the collector, and the counter-metrics (SM-C1 to SM-C3) are real. Two places strain it.

### Findings
- **medium** The stated differentiator is outside the MVP and no metric tests it (§1 "What makes usher worth more than a script is the loop"; §7.2 "Contribution loop (v2+): FR-57, FR-58"; §8 has no SM citing FR-57 or FR-58). The MVP validates the map-and-place thesis, which is fine, but the document should say so rather than leave the reader to notice the core claim ships last. *Fix:* one line in §7.1 naming what the MVP proves (the map thesis, via SM-1 to SM-4) and that the loop thesis waits, plus an SM placeholder for the loop (for example, share of accepted proposals adopted through FR-51).
- **medium** FR-53 and FR-66 have not been run through the objective test (§5 "Boundaries by objective", addendum §6, Q16). Addendum §6 covers FR-10, FR-13, FR-15 and FR-26, but FR-53's "errors experienced", "cache resets" and "time between turns" sit close to the fleet watcher's fault detection, and FR-66 puts usher in the launch path, which marvel also owns for agent sessions (§2.1 Non-human seats, Q2). *Fix:* add FR-53 and FR-66 rows to the addendum §6 table, and say in FR-66 whether a marvel-launched session gets its key from usher or from marvel.

## Done-ness clarity: adequate

Most FRs carry direct tests. FR-43's four statuses, FR-15's swap rule, FR-29's egress tests, FR-63's reverse-order rollback with hash check, and FR-64's seven bolded rules backed by addendum §7 are each ready for story writing. The two high findings are both in the new §4.16, where the promises outrun what usher controls or contradict another rule.

### Findings
- **high** FR-64 promises what usher cannot enforce for changes run by a third-party installer, and those are the MVP's only changes (FR-64 lead "A person can rely on a change touching only what the dry run listed"; "Scope is bounded. Each kind of change has a list of the paths it may touch"; "Any script usher runs stops on an unset variable or a failed command"; against FR-11 "drives existing installers", §4.2 "rather than re-hosting them", and §7.1 "In the MVP those changes are installs, updates, clones and generated files"). A vendor installer or `brew upgrade` writes where it writes, and usher cannot make a vendor script stop on an unset variable. As written, the tests in addendum §7 cannot pass for those changes, or pass only by testing the wrong layer. *Fix:* split FR-64 by who executes: rules that bind writes usher performs (temp-file rename, backup, parse, scope list) and rules that bind delegated runs (arguments as a list, resolved and checked inputs, journaled before/after version, outcome check after the run). State that a delegated installer's own footprint is reported, not bounded.
- **high** The journal's backup copies contradict the credential and content rules (FR-63 "the state before (a content hash and a backup copy for files ...)" and "It records presence only for credentials, never a value"; FR-64 "Usher backs a file up"; addendum §7 last row "A credential present in a target: The journal records presence only"; §5 "Credentials: presence, never value"). Harness settings files can hold credential values (an `env` block with an API key, a token in a config file), so a byte-for-byte backup stores the value the journal claims to omit. *Fix:* state the rule for backups explicitly: either backups are a local content store exempt from the presence-only rule, with their own permissions and a credential-scan marker, or a file found to hold credential material is backed up with the value redacted and its undo marked as manual. Add the case to addendum §7.
- **medium** FR-66's title overclaims (FR-66 "so the key never lands on disk"). The consequences bind usher only ("The key's value is never written to disk, logged, journaled or reported"); whether the harness itself writes a key it was handed is outside usher's control. *Fix:* reword the lead to "usher never writes the key to disk", and add a consequence that harness knowledge records whether each harness persists a key it receives, with the hazard check that tests it.
- **low** A few parameters have no value or default (FR-57 "a review clock", FR-13 "Notices are batched and quiet by default", FR-62 token budget). *Fix:* give each a default or an `[ASSUMPTION]` tag so stories do not invent one.

## Scope honesty: adequate

§6 Non-Goals do real work, the boundary items cite operator rulings, and §7.2 names each deferred FR. Inline assumptions round-trip to §10. Two leaks remain from the late split.

### Findings
- **medium** FR-38, in the MVP, lists fields from deferred FRs (FR-38 "cache reuse per session and activity effects (FR-53) at the chosen level"; §7.2 "Activity effects and reporting levels (v2): FR-53, FR-54"). An architect building the MVP collector cannot tell whether to accept activity fields or which level is "chosen". *Fix:* mark the FR-53 field and the level clause in FR-38 as arriving with FR-53/FR-54, or move them into those FRs.
- **low** The §7.1 scope sentence is easy to misread, and two deferrals have no reason (§7.1 "All of §4 except the features in §7.2 (FR-1 to FR-48, FR-55, FR-62 to FR-64)"). The parenthetical names the included set, but sits right after "§7.2" and reads as the excluded set. FR-62 is in both lists without "in part" in §7.1. FR-65 is built from FR-46 and FR-48, both in scope, and is deferred without a stated reason. *Fix:* rewrite as "In scope: FR-1 to FR-48, FR-55, FR-62 (command line and background service only), FR-63, FR-64"; add one clause for why FR-65 waits.
- **low** Phasing framing near-misses outside §7. The operator rule names `v1`, `v2` and `MVP`, and none appear outside §7. These carry development order in other words: §2.1 "Chat-product users (later cohort)" and "sequenced (§7.2)"; §4.16 description "from the first change usher ships"; §5 "carries a token budget from the start"; §9 Q18 "(later)". *Fix:* operator's call whether the rule reaches these; if it does, reword to state the property without the timing (for example §4.16: "This holds for every surface and every kind of change.").

## Downstream usability: adequate

IDs are unique and the stable, non-sequential ordering is explained in §0. Cross-references to addendum sections resolve. The glossary covers most nouns and the new "Change" and "Change journal" entries are used consistently in §4.16 and §5. Gaps are in terms added with the late FRs and in the addendum lagging the split.

### Findings
- **medium** Addendum §2 contradicts §7.1 on recommendations (addendum §2 "Central distribution (v2). The collector grows to distribute files in the same format, run templates and make recommendations"; PRD §7.1 puts FR-55 Local recommendations in scope). Templates in v2 agrees; recommendations does not. The addendum also uses `v1`/`v2` in §2 and §4. *Fix:* change addendum §2 to "run templates and distribute recommendations across hosts", and replace `v1`/`v2` there with references to §7.1/§7.2 if the phasing rule is meant to reach the addendum.
- **low** Terms used by the new FRs are missing from §3: "run variant" (and "factory", "legion") in FR-65, "vault" in FR-66, "declared seat" in FR-50, "backup store" in FR-64, "surface" in FR-62 and FR-63. *Fix:* add one-line glossary entries, or define each inline at first use.

## Shape fit: adequate

The capability-spec shape fits an internal tool with two UJs carrying named protagonists (John, the senior platform engineer). The document has grown to 66 FRs and roughly 1,380 lines, which is heavy for the "internal-tool depth" §0 claims, but as chain-top input the FR density earns its place. §4.13 and §4.15 lack the Description paragraph every other feature has; minor.

## Mechanical notes

- IDs: FR-1 to FR-66 contiguous and unique; SM-1 to SM-8 and SM-C1 to SM-C3 resolve; UJ-1 and UJ-2 each have a protagonist.
- Assumptions Index: three inline `[ASSUMPTION]` tags (FR-7, FR-20, FR-42), three index entries; round-trip holds. A stray blank line sits between the FR-20 and FR-42 entries in §10.
- Addendum index "Cited by" column: §3 lists FR-10, which does not cite it; §7 lists FR-63, which does not cite it (FR-64 does). Harmless, but the table should match.
- FR-66's last consequence recommends moving a plaintext key "behind the broker" from the credential scan, which ships before FR-66 does; the recommendation would point at a deferred feature.
- FR-63 says backups "are kept until the person removes them" but not what the journal shows for an entry whose backup was removed (undo no longer possible). One line would close it.
- Q13 names a person ("Mary, after Finalize"); fine for an internal draft, worth a sweep before going public (Q11).
