# Red team M3: config staleness claim

Conclusion attacked: per-harness config knowledge goes stale within weeks; removed config forms fail at startup; a settings map must be version-stamped and re-verified per release; a static map is unsafe.

## Verdict: survives, narrowed (weakened on two sub-claims)

The core recommendation (version-stamp each entry, re-verify per release) survives. No vendor states a config-stability commitment, a semver policy for config, or a deprecation window. Two sub-claims are weakened:

1. "Config knowledge goes stale" overstates it for the layer/precedence model. Claude Code documents a fixed five-level precedence stack and a documented degrade-gracefully path for bad entries. What churns is the key set and per-key semantics, not the layering. A map can split into a slow-moving layer (scopes, precedence, file locations) and a fast-moving layer (keys, values), and only the second needs per-release re-verification.
2. "Removed config forms now fail at startup" is not the general behavior. Claude Code skips bad entries with a warning; Codex keeps several deprecated aliases as warnings; goose appears to silently ignore removed lead/worker keys. Hard failure is the exception (Codex `approval_policy = "untrusted"`), not the rule. Note this cuts both ways: silent ignore makes a stale static map MORE dangerous, because nothing tells the user the key stopped working.

## Contrary evidence

1. Claim: Claude Code publishes a JSON schema for settings.json on SchemaStore and the official docs point to it; but the docs themselves say it lags releases ("The schema can lag behind the newest CLI releases, so a validation warning on a recently documented key doesn't mean your configuration is invalid"). So a schema exists, but it is not a stability anchor and is not versioned per release.
   - Source: https://code.claude.com/docs/en/settings (schema: https://json.schemastore.org/claude-code-settings.json)
   - Publisher: Anthropic
   - pub_date: not stated on page (live doc, references versions up to v2.1.314)
   - accessed: 2026-11-02
   - confidence: high
   - Effect: slightly weakens "no machine-readable surface exists"; does not weaken staleness (the vendor concedes lag).

2. Claim: Claude Code layer precedence is documented as a stable five-level stack (managed > command line --settings > project local > shared project > user), with list-merge semantics and a named exception table.
   - Source: https://code.claude.com/docs/en/settings
   - Publisher: Anthropic
   - pub_date: not stated
   - accessed: 2026-11-02
   - confidence: high for the current shape; medium for "stable", because the same page carries version-gated changes inside the layering (local file location moved at v2.1.211; /cd re-read needs v2.1.246; bypassPermissions from project files stopped at v2.1.257; modelPicker needs v2.1.242; blockWritesOutsideWorkingDirectories needs v2.1.314). These version notes are themselves evidence FOR the conclusion: even precedence-adjacent semantics move.

3. Claim: Claude Code does not fail at startup on bad or unknown entries. Invalid individual entries produce a "Settings Warning" and are skipped while the rest of the file applies; whole-file schema errors produce a dialog offering "continue without the broken settings"; `-p` runs skip broken files or values and continue; `claude doctor` lists rejected entries.
   - Source: https://code.claude.com/docs/en/settings (section "Fix a broken settings file")
   - Publisher: Anthropic
   - pub_date: not stated
   - accessed: 2026-11-02
   - confidence: high
   - Effect: weakens "removed forms fail at startup" for Claude Code; but silent skip in `-p` mode means stale keys can go unnoticed in automation.

4. Claim: Codex keeps deprecated config forms as warned aliases rather than hard failures: `approval_policy = "on-failure"` is deprecated (still accepted), `features.codex_hooks` is a "deprecated alias", `agents.max_threads` is a "Legacy alias", `personality` is a "Deprecated compatibility setting". Feature flags carry stability labels ("stable; on by default", "experimental; off by default").
   - Source: https://learn.chatgpt.com/docs/config-file/config-reference (redirected from https://developers.openai.com/codex/config-reference)
   - Publisher: OpenAI
   - pub_date: not stated
   - accessed: 2026-11-02 (first 100000 of 166881 characters read)
   - confidence: high
   - Effect: shows Codex does practice alias-with-warning deprecation for some keys, weakening the "removed forms fail" generalization. The same page confirms `approval_policy = "untrusted"` is unsupported ("Codex and ChatGPT Work no longer support approval_policy = \"untrusted\""), so that half of the conclusion stands. It also confirms profiles now live as `$CODEX_HOME/profile-name.config.toml`, consistent with a legacy-profiles change.

5. Claim: the Codex deprecated-key path is a warning, not an error, observed in 0.129.0 to 0.131.0-alpha ("[features].codex_hooks is deprecated. Use [features].hooks instead."); maintainer called it "working as intended".
   - Source: https://github.com/openai/codex/issues/22148
   - Publisher: OpenAI (GitHub issue, user-filed, maintainer etraut-openai replied)
   - pub_date: 2026-05-11
   - accessed: 2026-11-02
   - confidence: high
   - Effect: supports point 4. Also shows the warning can misfire from a bundled override in the Desktop app, a reminder that "what config is in effect" includes layers the user does not write.

6. Claim: goose removed the lead/worker provider (PR #7989, merged 2026-03-18) without a migration or deprecation notice; per a later PR (#12061, as quoted on the #7989 page) the keys are "silently ignored", i.e. not a startup failure.
   - Source: https://github.com/block/goose/pull/7989 (now under aaif-goose/goose)
   - Publisher: Block / AAIF goose maintainers
   - pub_date: 2026-03-18 (merge)
   - accessed: 2026-11-02
   - confidence: medium (the "silently ignored" wording is second-hand via the page; #12061 itself was not retrieved)
   - Effect: weakens "fail at startup" for goose, strengthens "static map unsafe" (silent no-op is the worst case for a map consumer).

## Searched for and did not find

- Any vendor commitment to config stability, a semver or deprecation policy covering config keys, or a minimum deprecation window, for any of the three harnesses.
- A versioned (per-release) JSON schema for Claude Code settings; the SchemaStore entry exists but the vendor says it lags.
- A published JSON schema URL for Codex config.toml on the config reference (not on the portion read).
- Migration tooling for config (no `codex migrate` from an OpenAI source; a third-party blog claims one, uncorroborated and not cited here).
- A goose migration note or warning for removed lead/worker or planning-mode keys.
- goose PR #12061 itself (search did not surface it).
- Codex's general behavior on unknown keys (the reference page portion read does not say).

## Net for the map design

Keep version stamps. Split the map into a slow layer (scopes, precedence, file paths, merge rules; re-verify on major changes or quarterly) and a fast layer (keys, values, defaults; re-verify per release). Record, per harness, what a stale key does (Claude Code: warn and skip; Codex: warn for aliases, error for retired values; goose: apparently silent), since that decides how loudly the map must flag drift.
