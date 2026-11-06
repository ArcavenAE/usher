---
title: 'technical research: harness settings observability map'
type: 'technical'
topic: 'harness settings observability map (Claude Code, codex, goose)'
decision: 'what usher v1 can observe per harness, from which layer and resolved view, and how fast that knowledge goes stale'
source: 'native run'
status: complete
claims_verified: 15
claims_unverified: 6
claims_disputed: 6
claims_overturned: 2
preset: 'standard'
validation: 'normal'
red_team: 'on'
created: '2026-11-02'
updated: '2026-11-02'
---

# technical research: harness settings observability map

**Decision this research serves:** what usher v1 can observe per harness (Claude Code, codex, goose), from which layer and through which resolved view, and how fast that knowledge goes stale. Feeds the usher PRD settings-map and drift-check requirements.

## Executive summary

**What to do.** Usher should use a different observation strategy per harness, not one file reader:

- **codex: ask the harness.** The app-server method `config/read`, called with `cwd` and `includeLayers`, returns the effective config, every layer with a `disabledReason`, and an `origins` map from each key to its layer; `configRequirements/read` returns the requirements from `requirements.toml` and/or MDM [40][11]. This is the only one of the three with a merged view that carries provenance.
- **Claude Code: reconstruct, then confirm with the harness's own partial views.** Model the documented stack and per-key scope table [1][2], read the server-managed cache at `~/.claude/remote-settings.json` [39], and cross-check against `claude auto-mode config` (effective auto-mode lists, no provenance), `/permissions` (rules with source file) and `claude doctor` (rejected entries, remote managed state) [4][3][39][6].
- **goose: reconstruct from a short stack.** System file, additional files, user file, then environment variables by key, with per-entry merging only for extensions and providers [26]. Read the source, not only the docs, which omit the system layer [24].

**What drives it.** (1) Provenance is available for codex and must be computed for the other two. (2) In every harness, what a file says can differ from what is in effect: Claude skips bad entries and holds back some server-managed values [39]; codex ignores some project keys [12]; goose reads environment variables before any file [26]. A drift check must compare against resolved state wherever a resolved view exists. (3) The key set changes on a scale of weeks (Claude Code shipped about ten releases in thirteen days [7][8]; codex retired a config value that now fails with an error [17]), while layer order changes more slowly [1].

**Biggest caveat.** Six claims are disputed, among them whether omitting `"$defaults"` still discards Claude's built-in auto-mode rules on 2.1.317 [4][7] and whether codex's MDM config outranks user config on current code [14][16]. Each is a cheap local test on a scratch home, and the map should not hard-code either until its test is run. goose was not probed locally because it is not installed here.

## 1. Claude Code (2.1.317)

**Layers and precedence.** Highest first: managed, `--settings`, `.claude/settings.local.json`, `.claude/settings.json`, `~/.claude/settings.json` [1]. Lists merge across files except four model keys (`fallbackModel`, `modelPicker`, `availableModels`, `modelSettings`) [1]. Permission rules are evaluated deny first, then ask, then allow; the first match wins whatever scope it came from [3]. Environment variables are not a layer: each variable and its settings key are resolved as a pair (for example `ANTHROPIC_MODEL` beats the `model` key in any file), and dedicated flags such as `--model` override per key [5]. `CLAUDE_CONFIG_DIR` relocates the user home [5]. The settings index carries a per-key Scope column (`User, local, or managed`, `User or managed`, `User`, `Managed`, `Global config`); keys outside the project scope are ignored in the shared project file, and about a dozen security keys let a stricter lower-scope value beat managed [1].

**Managed tier.** Sources rank remote (server-managed) over MDM (plist domain `com.anthropic.claudecode`, HKLM) over `managed-settings.json` plus `managed-settings.d/*.json` over HKCU. The default is first-source-wins, which skips the other sources without warning; `managedSourcesBehavior: "merge"` exists from v2.1.242 [2]. Remote settings poll hourly, MDM every 30 minutes, and settings files hot-reload [2]. _Confidence: medium (one docs page; not confirmed in the changelog within budget)._

**Setting classes with layer limits.** `autoMode` is read only from user, managed and `--settings`, never from project or local files; entries combine additively across scopes [4]. Whether omitting `"$defaults"` discards the built-in rules is **disputed**: the docs say it replaces the whole default list [4], while the v2.1.316 changelog says an `autoMode.allow` list without `"$defaults"` "no longer removes the built-in rule" [7]. `defaultMode` values `auto` and `bypassPermissions` do not take effect from project or local files [1]. OTEL variables and `CLAUDE_CONFIG_DIR` are ignored in project and local settings [5]. Folder trust lives in `~/.claude.json` as `projects[<repo root>].hasTrustDialogAccepted` [3][1].

**Resolved views.** No documented command shows which file supplied each key; `/status` "confirms which files Claude Code read; it doesn't show which file supplied each key" [1]. Partial views: `/permissions` lists each rule with its source file [3]; `claude auto-mode config` prints the effective allow, soft_deny, hard_deny and environment lists as JSON, with no per-entry source [4][11]; `/status` names the managed source and, since v2.1.242, skipped sources [2]; `claude doctor` lists rejected entries [6]. The statusline input exposes the effective model, effort level, version and context window, but no settings sources [10]. On this host the effective auto-mode lists held more allow and environment entries than the shipped defaults, and only a diff against the files could attribute those extra entries to a layer [11].

**Server-managed settings and what stays effect-only.** Server-managed settings are cached in a readable file, `~/.claude/remote-settings.json` [39], present on this host [11]. The cache can differ from what is in effect: some `env` categories (proxy and TLS, base URL, provider, credentials, `CLAUDE_CONFIG_DIR`, `HOME`), `modelPricing` and `managedMcpServers` are held until the server confirms; `model` and OTEL exporter changes apply at next launch; non-interactive runs do not write approval-requiring settings to it; `/logout` deletes it when the session used an API key or token credential [39]. `claude doctor` (v2.1.248+) reports the remote managed state as loaded, none, fetch failed or skipped [39]. Still effect-only: feature flags fetched from Anthropic, which `DISABLE_TELEMETRY` or a third-party provider turns off [5], and the prompt cache TTL in force. _Confidence: medium for the effect-only items._

**Docs versus behavior.** Issue #70181 (an empty or 304 server response zeroed local managed deny rules) was closed by the stale bot on 2026-10-13 with no linked fix [9]. Several precedence fixes landed between 2.1.304 and 2.1.317 [7].

**Cadence.** v2.1.317 is current, matching the installed version [8][11]; about ten releases shipped in thirteen days, adding environment variables and keys and changing the meaning of some [7][8].

## 2. Codex CLI (latest stable 0.166.0; installed 0.160.1)

**Layers and precedence.** Highest first: CLI flags and `-c` overrides, project `.codex/config.toml` files from repository root to cwd (closest wins), the selected profile, user `~/.codex/config.toml`, cloud-managed defaults, system `/etc/codex/config.toml`, built-in defaults [12][13]. The source names these layer variants and orders project layers root to cwd [15]; numeric ranks come only from a third-party copy of the crate, which also shows a lowest-rank `Mdm` base layer and two legacy `managed_config.toml` layers (file and MDM) ranked **above** CLI flags [16]. _Confidence: medium for the ranks._ Project layers are documented as loading only for trusted projects [12]; code tying this to trust was not found [15].

**Managed tier: defaults and requirements.** Two different mechanisms. Managed defaults (`managed_config.toml`, macOS MDM `config_toml_base64`) set values; requirements (cloud, MDM `requirements_toml_base64`, `/etc/codex/requirements.toml`) constrain the allowed values of approval policy, sandbox mode, web search, an MCP allowlist, features and hooks, falling back to a compliant value with a notice [14]. Requirements are tracked separately from the layer stack in source [15]. Whether `config_toml_base64` feeds the high legacy layer or the low `Mdm` base layer is **disputed**, which decides whether it beats user config on current versions [14][16]. The official docs back the high reading: MDM and `managed_config.toml` defaults override user config and CLI `--config` overrides, described as legacy ordering [14].

**Setting classes with layer limits.** Project config ignores `model_provider`, `model_providers`, `notify`, `profile`, `profiles`, `otel` and several base-URL keys [12] (a startup warning was reported in round 1 but not confirmed by the citation check); issue #22222 reports this broke repo-local provider routing and is open [22]. _Unverified beyond the docs._ AGENTS.md is read from CODEX_HOME globally and then one file per directory from root to cwd, joined so closer files come later; `project_doc_max_bytes` defaults to 32 KiB [23]. Skills come from `.agents/skills` up to the repository root, then `~/.agents/skills`, then `/etc/codex/skills` [12]. Whether project skills and rules load in untrusted projects is **disputed**: the docs say they are skipped [12]; a maintainer said in January 2026 that local skills would again enumerate in untrusted projects [21]. OTEL export is off by default, but a built-in analytics exporter is on unless `analytics.enabled = false` [12]. _Unverified beyond the docs._

**Resolved views.** `/debug-config` lists the layer stack per layer and, since February 2026, values from session-flag and MDM layers [37]. _Unverified this run._ **The app server exposes merged config with provenance.** The app-server JSON-RPC method `config/read` returns the effective configuration after layering [40]; its schema on the installed 0.160.1 takes `cwd` (to include the project layers between `cwd` and the repository root) and `includeLayers`, and returns `config`, `layers` (each with a `disabledReason`) and a required `origins` map from each key to the layer that set it [11]. `configRequirements/read` returns the requirements read from `requirements.toml` and/or MDM [40][11]. Without `cwd`, project layers are not included [11]. Secondary views: `codex doctor --json` reports a redacted config-load section (configuration scope, CODEX_HOME, feature-flag overrides, MCP servers, model provider) and `codex features list` gives each flag's stage and effective state [11]. _This overturns the round-1 finding that no merged view exists._

**Cloud-delivered layers.** Cloud requirements and cloud defaults arrive at ChatGPT sign-in and take effect when an admin saves them [14]. Requirements are readable through `configRequirements/read` [40], and the official docs describe the client keeping a signed, identity-matched cache entry and refreshing it in the background [14]; its filename and lifetime are unsourced. Whether cloud defaults are cached locally is unknown.

**Breaking changes and cadence.** `approval_policy = "untrusted"` was retired by PR #39630 (merged 2026-08-20); an explicit setting now fails with an actionable error [17]. Legacy profiles were removed in rust-v0.134.0 (2026-05-26) [18]. Latest stable is 0.166.0, released about 2026-10-29 [19][20]; the installed 0.160.1 is six minors behind [11]. The docs moved hosts (developers.openai.com now redirects to learn.chatgpt.com) and some paths 404 [12].

## 3. goose (v1.57.0)

**Governance.** goose is a founding project of the Agentic AI Foundation, announced by the Linux Foundation on 2025-12-09; the repository is now `aaif-goose/goose` and the docs are at goose-docs.ai [32][30]. Older pages on block.github.io are still served and some state superseded defaults [38].

**Layers and precedence.** The source stacks a system file (`/etc/goose/config.yaml`, or ProgramData on Windows), then files named in `GOOSE_ADDITIONAL_CONFIG_FILES`, then the user `~/.config/goose/config.yaml`; later wins, `extensions` and `providers` merge per entry, other top-level keys are replaced, and goose writes only the user file [26]. An environment variable named by the uppercased key is checked before any file [26][25]. The docs describe only environment over user config over defaults, so the system layer, the only admin path found, is undocumented [24]. `GOOSE_PATH_ROOT`, when absolute, relocates config, data and state [27]. There is no project-level config file; issue #6787 asking for one was closed by its filer, not by a maintainer ruling [26][33]. Secrets resolve from environment, then keyring, then a `secrets.yaml` fallback [24].

**Setting classes.** `GOOSE_MODE` takes `auto`, `approve`, `smart_approve` or `chat`; the default is **disputed** (live docs: `auto` [25]; older docs: `smart_approve` [38]; the source check failed). `/mode` changes the mode for the session only [35]. Per-tool permissions live in `permission.yaml` and `permissions/tool_permissions.json` [36]. Telemetry is opt-in through `GOOSE_TELEMETRY_ENABLED`, with `GOOSE_TELEMETRY_OFF` as the environment kill switch; the `GOOSE_DISABLE_TELEMETRY` named in `CUSTOM_DISTROS.md` is **not read** by the source [28][29][25]. Project instruction files (`.goosehints`, AGENTS.md, `CONTEXT_FILE_NAMES`) were reported in round 1 but not verified. No hooks mechanism and no directory trust were found.

**Resolved views.** `goose info -v` shows active settings with current values [35]; whether those are merged values, and from which layer, is unknown. `goose session diagnostics` writes a JSON report including config files [35]. _Low confidence on provenance._

**Docs versus behavior.** PR #9439 (merged 2026-06-11) fixed malformed extension entries being deleted when another extension was toggled; its scope is narrower than general config loss [34].

**Cadence.** v1.57.0 published 2026-10-28, confirmed by GitHub and the Homebrew formula [30][31]. Lead/worker mode and its planning-mode successor were removed in recent minors [30]. Weekly cadence rests on few data points. goose is not installed on this host, so no local probe was run.

## Cross-dimension insights

- **The harness with the most complex managed tier exposes the best view of it.** codex has two managed mechanisms (defaults and requirements) plus a cloud channel [14], and it is also the one that reports merged config with per-key origins [40][11]. goose has the simplest stack and the weakest view, but needs it least [26]. Claude Code sits between: a four-source managed tier with first-wins skipping [2] and only partial self-reports [1].
- **Install health and the settings map are one problem.** The behavior usher must model depends on the installed version, and the installed versions lag: codex here is 0.160.1 against 0.166.0 stable [11][19]. The version usher detects for install health should select which version of the map it applies, and a version change should trigger re-verification of the fast-moving part of the map.
- **Docs lag source in two of three harnesses.** goose's docs omit its system layer [24][26] and its distro guide names a telemetry variable the code does not read [29][28]; codex's docs moved hosts with some paths broken [12]. Map entries should cite source or changelog, and docs only as a second source.

## Contrary evidence

The red-team pass attacked three conclusions.

- **"No harness exposes merged config with provenance": overturned for codex**, by the app-server `config/read` method [40], confirmed in the locally generated schema [11]. It survives for Claude Code, where `/status` explicitly does not show which file supplied each key [1], and for goose [35]. A third-party VS Code extension reconstructs the Claude override chain without covering flags or environment variables, which supports reconstruction as the method rather than offering a replacement for it.
- **"Each harness has a layer that never lands on disk": weakened.** Claude Code caches server-managed settings in a readable file and self-reports their state [39]; the narrower claim that survives is that the cache can differ from what is in effect, and that server-fetched feature flags stay unobservable. codex requirements are API-readable [40] and cached locally as a signed entry [14]. No remote config channel was found for goose.
- **"Config knowledge goes stale within weeks": survives, refined.** No vendor states a config stability or deprecation policy. Layer order is documented and slow-moving [1]; keys and semantics are what churn. Most removed forms warn or are skipped rather than fail: codex keeps several deprecated forms with a warning (openai/codex#22148, [42]), and goose's lead/worker removal shipped without a migration note [43]. Silent skipping is the larger hazard, because nothing tells the user a setting stopped applying. A community JSON schema for Claude Code settings exists but its own docs say it can lag releases [41].

## Recommendations

1. **PRD, settings map:** specify the per-harness observation strategy above (query codex; reconstruct and confirm Claude Code; reconstruct goose). _Confidence: high for codex (official docs plus local schema), medium for Claude Code and goose._
2. **PRD, every observation:** carry the harness version, the method used (harness query, file read, reconstruction), and an explicit list of what could not be observed (Claude feature flags, prompt cache TTL; codex cloud defaults). This implements the brief's rule that recommendations declare what they could not observe.
3. **Architecture:** split the map into a slow layer (scopes, precedence, file paths, merge rules) and a fast layer (keys, defaults, value sets), with the fast layer re-verified per harness release and keyed to the version install health detects. Record per harness what a stale key does: warn, alias, hard error, or silent skip.
4. **PRD, drift check:** compare the declared profile with resolved state, not with files, where a resolved view exists (`config/read` for codex; `claude auto-mode config` for Claude auto mode). The settings-profile idea's `"$defaults"` structural check must be version-gated, since its premise is disputed on 2.1.316 and later [4][7]. _Confidence: medium._
5. **Brief, check before correcting:** the brief says codex "silently ignores" provider, auth, notify and otel keys in project config. The ignored-key list matches the docs [12], but whether a startup warning is printed was not confirmed in the part of the page the citation check read. Confirm with one scratch-repo run before changing "silently". _Confidence: low._
6. **Settings-profile idea, open question 3 is answered:** `approval_policy = "untrusted"` was retired and an explicit setting now fails with an actionable error [17]; a profile must not name it.

## Open questions

- **`"$defaults"` on 2.1.317:** run `claude auto-mode config` under a scratch home with an `autoMode.allow` list that omits `"$defaults"`, and compare with `claude auto-mode defaults`.
- **codex MDM rank:** which layer `config_toml_base64` feeds on current code; answerable from `config/read` origins on a machine with an MDM profile, or from source.
- **codex untrusted projects:** whether project skills, rules and AGENTS.md load; one scratch repo test with `config/read`.
- **codex analytics default** and **`/debug-config` output** (unverified beyond docs).
- **goose:** the `GOOSE_MODE` default, whether `goose info -v` shows merged values and their sources, and instruction-file discovery. Needs goose installed on a test host; installing it here needs your approval.
- **Claude Code feature flags:** whether any local cache exists; no evidence was found either way.
- **Per-harness stale-key behavior table** (warn, alias, error, silent): partly known, not enumerated.

## Source appendix

| # | Supports | Publisher | Pub date | Accessed | Confidence |
|---|---|---|---|---|---|
| [1] | Claude settings precedence, merge, scope column, /status limits | [Anthropic: Claude Code settings](https://code.claude.com/docs/en/settings) | undated | 2026-11-02 | high |
| [2] | Claude managed tier sources and first-wins | [Anthropic: managed settings](https://code.claude.com/docs/en/managed-settings) | undated | 2026-11-02 | medium |
| [3] | Permission evaluation order, /permissions, folder trust | [Anthropic: permissions](https://code.claude.com/docs/en/permissions) | undated | 2026-11-02 | high |
| [4] | autoMode scopes, additivity, $defaults (docs side) | [Anthropic: auto mode config](https://code.claude.com/docs/en/auto-mode-config) | undated | 2026-11-02 | high |
| [5] | Env var pairing, CLAUDE_CONFIG_DIR, telemetry and flags | [Anthropic: environment variables](https://code.claude.com/docs/en/env-vars) | undated | 2026-11-02 | high |
| [6] | claude doctor rejected entries | [Anthropic: debug your config](https://code.claude.com/docs/en/debug-your-config) | undated | 2026-11-02 | medium |
| [7] | v2.1.316 $defaults change; release cadence; precedence fixes | [Anthropic: CHANGELOG.md](https://raw.githubusercontent.com/anthropics/claude-code/main/CHANGELOG.md) | 2026-11 | 2026-11-02 | high |
| [8] | Current version 2.1.317 and cadence | [GitHub: claude-code releases](https://github.com/anthropics/claude-code/releases) | 2026-11 | 2026-11-02 | high |
| [9] | Issue 70181 closed stale, no fix | [GitHub: anthropics/claude-code#70181](https://github.com/anthropics/claude-code/issues/70181) | 2026-10 | 2026-11-02 | high |
| [10] | Statusline input fields | [Anthropic: statusline](https://code.claude.com/docs/en/statusline) | undated | 2026-11-02 | high |
| [11] | Local probes: versions, auto-mode config structure, codex doctor/features, app-server schema, remote-settings cache presence | [this run: imports/](imports/) | 2026-11-03 | 2026-11-03 | high |
| [12] | codex precedence, ignored project keys, skills, telemetry defaults | [OpenAI: codex config reference](https://learn.chatgpt.com/docs/config-file/config-reference) | undated | 2026-11-02 | medium |
| [13] | codex layered config | [OpenAI: codex advanced config](https://learn.chatgpt.com/docs/config-file/config-advanced) | undated | 2026-11-02 | medium |
| [14] | codex managed defaults, requirements, cloud delivery | [OpenAI: codex managed configuration](https://learn.chatgpt.com/docs/enterprise/managed-configuration.md) | undated | 2026-11-02 | medium |
| [15] | codex layer variants, project order, requirements tracked separately | [OpenAI: codex-rs config state.rs](https://raw.githubusercontent.com/openai/codex/main/codex-rs/config/src/state.rs) | 2026-11 | 2026-11-02 | high |
| [16] | codex numeric layer ranks, legacy managed layers | [docs.rs: typeduck-codex fork](https://docs.rs/typeduck-codex-extension-items/0.5.0/src/codex_config/config_layer_source.rs.html) | unknown | 2026-11-02 | low |
| [17] | approval_policy untrusted retired | [GitHub: openai/codex#39630](https://github.com/openai/codex/pull/39630) | 2026-08-20 | 2026-11-02 | high |
| [18] | Legacy profiles removed | [GitHub: codex rust-v0.134.0](https://github.com/openai/codex/releases/tag/rust-v0.134.0) | 2026-05-26 | 2026-11-02 | high |
| [19] | codex latest stable 0.166.0 | [npm: @openai/codex](https://registry.npmjs.org/@openai/codex/latest) | 2026-10-29 | 2026-11-02 | high |
| [20] | codex release cadence | [GitHub: codex releases](https://github.com/openai/codex/releases) | 2026-10 | 2026-11-02 | high |
| [21] | Untrusted project skills (maintainer statement) | [GitHub: openai/codex#9696](https://github.com/openai/codex/issues/9696) | 2026-01-22 | 2026-11-02 | medium |
| [22] | Project provider routing broken by ignore list | [GitHub: openai/codex#22222](https://github.com/openai/codex/issues/22222) | 2026-05 | 2026-11-02 | medium |
| [23] | AGENTS.md discovery and size limit | [OpenAI: AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md) | undated | 2026-11-02 | medium |
| [24] | goose documented precedence, secrets | [goose: config files](https://goose-docs.ai/docs/guides/config-files) | undated | 2026-11-02 | medium |
| [25] | goose env vars, GOOSE_MODE values, telemetry key | [goose: environment variables](https://goose-docs.ai/docs/guides/environment-variables) | undated | 2026-11-02 | medium |
| [26] | goose layer stack in source | [aaif-goose: config/base.rs](https://raw.githubusercontent.com/aaif-goose/goose/main/crates/goose/src/config/base.rs) | 2026-10 | 2026-11-02 | high |
| [27] | GOOSE_PATH_ROOT | [aaif-goose: config/paths.rs](https://raw.githubusercontent.com/aaif-goose/goose/main/crates/goose/src/config/paths.rs) | 2026-10 | 2026-11-02 | high |
| [28] | Telemetry kill switch in source | [aaif-goose: posthog.rs](https://raw.githubusercontent.com/aaif-goose/goose/main/crates/goose/src/posthog.rs) | 2026-10 | 2026-11-02 | high |
| [29] | Distro guide names GOOSE_DISABLE_TELEMETRY | [aaif-goose: CUSTOM_DISTROS.md](https://raw.githubusercontent.com/aaif-goose/goose/main/CUSTOM_DISTROS.md) | 2026-10 | 2026-11-02 | high |
| [30] | goose v1.57.0, removals, repo home | [GitHub: goose releases](https://github.com/aaif-goose/goose/releases) | 2026-10-28 | 2026-11-02 | high |
| [31] | goose 1.57.0 second source | [Homebrew: block-goose-cli](https://formulae.brew.sh/api/formula/block-goose-cli.json) | 2026-10 | 2026-11-02 | high |
| [32] | Agentic AI Foundation, goose founding project | [Linux Foundation press release](https://www.linuxfoundation.org/press/linux-foundation-announces-the-formation-of-the-agentic-ai-foundation) | 2025-12-09 | 2026-11-02 | medium |
| [33] | Project config request closed by filer | [GitHub: goose#6787](https://api.github.com/repos/aaif-goose/goose/issues/6787) | 2026 | 2026-11-02 | high |
| [34] | PR 9439 scope and merge | [GitHub: goose PR 9439](https://api.github.com/repos/aaif-goose/goose/pulls/9439) | 2026-06-11 | 2026-11-02 | high |
| [35] | goose info -v, /mode, session diagnostics | [goose: CLI commands](https://goose-docs.ai/docs/guides/goose-cli-commands) | undated | 2026-11-02 | medium |
| [36] | goose per-tool permission files | [goose: permissions](https://goose-docs.ai/docs/guides/managing-tools/goose-permissions/) | undated | 2026-11-02 | medium |
| [37] | /debug-config per-layer view | [GitHub: openai/codex#11412](https://github.com/openai/codex/pull/11412) | 2026-02-11 | 2026-11-02 | medium |
| [38] | Older goose docs with superseded defaults | [Block: goose env vars (old site)](https://block.github.io/goose/docs/guides/environment-variables) | undated | 2026-11-02 | low |
| [39] | Claude server-managed cache, doctor remote state | [Anthropic: server-managed settings](https://code.claude.com/docs/en/server-managed-settings) | undated | 2026-11-02 | high |
| [40] | codex app-server config/read and configRequirements/read | [OpenAI: codex app server](https://learn.chatgpt.com/docs/app-server) | undated | 2026-11-02 | high |
| [41] | Community Claude Code settings schema, may lag | [SchemaStore: claude-code-settings](https://json.schemastore.org/claude-code-settings.json) | undated | 2026-11-02 | medium |
| [42] | codex deprecated forms accepted with warning | [GitHub: openai/codex#22148](https://github.com/openai/codex/issues/22148) | 2026-05-11 | 2026-11-02 | medium |
| [43] | goose lead/worker removal without migration note | [GitHub: aaif-goose/goose PR 7989](https://github.com/aaif-goose/goose/pull/7989) | 2026-03-18 | 2026-11-02 | medium |

## Staleness map

Computed with `recon_kit.py staleness` from the claims ledger, using month-granularity publication dates and these windows: version, layers, resolved-view and effect-only 1 month; precedence and docs-gap 3 months; cadence 6 months. Fourteen of 27 claims are already past their window on the day of writing, which is the expected result for a fast layer on one-month windows.

| Re-check by | Class | Claims |
|---|---|---|
| already due | layers, version, resolved-view, docs-gap, cadence | goose G1-G4, G6-G9; codex X3-X8 |
| 2026-12-01 | layers, resolved-view, version, effect-only | Claude C3-C5, C7; codex X2, M1; off-disk M2 |
| 2027-01-01 | docs-gap | Claude C6; goose G5 |
| 2027-02-01 | precedence | Claude C1, C2; codex X1 |
| 2027-05-01 | cadence | stability policy M3 |

Earliest re-check: already due (codex `/debug-config`, from February 2026). Every fast-layer claim should be re-run on each harness release; Refresh handles this.
