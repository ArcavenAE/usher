# Codex CLI r2 verification digest (2026-11-02)

Budget used: 14 tool calls, 7 sources fetched or searched. Official raw GitHub paths for the config loader 404'd (codex-rs/core/src/config_loader/ tree, codex-rs/app-server-protocol/src/protocol/v2.rs), so the precedence code was read from codex-rs/config/src/state.rs (official) plus a third-party docs.rs republish of codex-config (typeduck fork). The fork caveat applies to the numeric ranks.

## Per claim

X1 | verified in order, with two caveats (MDM base layer; legacy managed layers on top) | evidence:
- {claim: ConfigLayerStack layers are "listed from lowest precedence (base) to highest (top), so later entries in the Vec override earlier ones"; variants PackagedDefaults, Mdm, System, EnterpriseManaged, User, Project, SessionFlags, LegacyManagedConfigTomlFromFile, LegacyManagedConfigTomlFromMdm; verify_layer_ordering checks "project layers are ordered from root to cwd"; LoaderOverrides::ignore_project_config "Skip project-root discovery and all project configuration layers", source: https://raw.githubusercontent.com/openai/codex/main/codex-rs/config/src/state.rs, publisher: OpenAI (GitHub), pub_date: main as of 2026-11-02, accessed 2026-11-02, confidence: high}
- {claim: ConfigLayerSource::precedence() ranks Mdm 0, System 10, EnterpriseManaged 15, User 20 (21 when a profile is selected), Project 25, SessionFlags 30, LegacyManagedConfigTomlFromFile 40, LegacyManagedConfigTomlFromMdm 50; loader README lists highest to lowest: legacy MDM managed, legacy file managed, SessionFlags, Project, User profile, User, EnterpriseManaged, System, source: https://docs.rs/typeduck-codex-extension-items/0.5.0/src/codex_config/config_layer_source.rs.html and https://docs.rs/crate/typeduck-codex-extension-items/0.5.0/source/src/loader/README.md, publisher: third-party docs.rs republish of codex-config (fork, version vs upstream unknown), pub_date: unknown, accessed 2026-11-02, confidence: medium}
- Notes: the claimed order CLI > project > profile > user > cloud-managed > system > built-in matches ranks 30 > 25 > 21 > 20 > 15 > 10. Not in the claim: an Mdm layer at rank 0 (below system) and legacy managed_config layers at 40/50 above CLI flags. "Loaded only for trusted projects" was NOT confirmed in code: state.rs comments do not mention trust; layers carry a disabled_reason and the README says disabled layers "are still surfaced for UI, but are ignored". Trust gating is plausible as one disabled reason but unverified this run.

X2 | partially verified (managed_config.toml part); requirements part unverified; MDM config_toml_base64 placement disputed-unclear | evidence:
- {claim: LegacyManagedConfigTomlFromFile (40) and LegacyManagedConfigTomlFromMdm (50) rank above SessionFlags (30), so managed_config.toml overrides user config and -c overrides; README calls the MDM one "MDM-delivered managed_config.toml, while it is being phased out", source: docs.rs typeduck README and config_layer_source.rs (above), publisher: third-party republish, pub_date: unknown, accessed 2026-11-02, confidence: medium}
- {claim: a separate Mdm variant ranks 0, the lowest layer, i.e. a base config rather than an override, source: same, confidence: medium}. If config_toml_base64 feeds the Mdm (0) variant rather than LegacyManagedConfigTomlFromMdm (50), then "MDM config_toml_base64 overrides user config" is overturned for current code. I could not tie the MDM key name to a variant this run.
- {claim: requirement sources "are tracked separately and are not included here" (not part of the layer stack), source: codex-rs/config/src/state.rs (official), confidence: high}. What requirements constrain (approval policy, sandbox mode, MCP allowlist, features, hooks) and their sources (cloud, MDM requirements_toml_base64, /etc/codex/requirements.toml): not independently confirmed.

X3 | unverified | No independent source retrieved for the ignored-key list or the startup warning. state.rs has a startup_warnings field ("None means the loader did not check for stack-level warnings"), consistent with but not proof of the claim. Also found: ignore_user_and_project_exec_policy_rules ("Whether execpolicy should skip .rules files from user and project config-layer folders").

X4 | verified (untrusted removal); partially verified (profiles) | evidence:
- {claim: PR #39630 "Retire the untrusted approval policy", merged to main 2026-08-20; "Remove untrusted from the CLI, configuration schema, and MCP tool interface"; "Explicit approval_policy = "untrusted" settings now fail with an actionable error"; untrusted projects now require approval for every command unless an exec policy rule allows it, source: https://github.com/openai/codex/pull/39630, publisher: OpenAI (GitHub), pub_date: 2026-08-20, accessed 2026-11-02, confidence: high}
- {claim: 0.134.0 (published 26 May 2026) removed legacy profile v1: "config: remove legacy profile v1 resolution" (#24051), "fix: reject legacy profile selectors" (#24059), "cli: rename profile v2 flag to --profile" (#23883), "Added profile migration documentation links to relevant config errors", source: https://github.com/openai/codex/releases/tag/rust-v0.134.0, publisher: OpenAI (GitHub), pub_date: 2026-05-26, accessed 2026-11-02, confidence: high}. The release notes do not name [profiles.*] or the ~/.codex/<name>.config.toml file form; that the v2 profile is a separate file is unverified this run. The User rank 21 "with profile" supports a distinct profile layer above user config.

X5 | unverified | No source retrieved. 0.134.0 notes mention codex-analytics and "otel: drop legacy profile usage telemetry" (#24061), which shows analytics and otel are separate subsystems but says nothing about defaults.

X6 | unverified | Not checked (budget). README says disabled layers are surfaced for UI, consistent with a per-layer view.

X7 | verified | evidence:
- {claim: npm dist-tag latest for @openai/codex is "version":"0.166.0"; registry internal timestamp 1793303779665 ms, which decodes to about 2026-10-29 UTC, source: https://registry.npmjs.org/@openai/codex/latest, publisher: npm registry, pub_date: about 2026-10-29 (decoded), accessed 2026-11-02, confidence: high for version, medium for date}

X8 | partially resolved: skills yes (since about 0.89), AGENTS.md and rules unverified | evidence:
- {claim: issue #9696 (opened and closed 2026-01-22): maintainer etraut-openai: "Starting with version 0.88 of the CLI, we filtered local skills for non-trusted projects" and "The next version will once again enumerate local skills even for untrusted projects", independent of sandbox mode, with skill execution still subject to sandbox mode, source: https://github.com/openai/codex/issues/9696, publisher: OpenAI (GitHub), pub_date: 2026-01-22, accessed 2026-11-02, confidence: medium (statement of intent; no linked PR; no check against current code)}
- The conflict likely comes from this reversal: 0.88 filtered project skills in untrusted projects, the next release restored enumeration. Nothing retrieved covers AGENTS.md or project .rules in untrusted projects. Note that PR #39630 changes what "untrusted project" means for approvals (every command needs approval), which is separate from content loading.

## Lead findings

- codex doctor --json and codex mcp list: not investigated (budget). No evidence retrieved.
- Config internals moved from codex-core (core/src/config_loader) to a codex-config crate at codex-rs/config; future source checks should target codex-rs/config/src/state.rs and config_layer_source.rs there.
- Legacy managed_config.toml is being phased out (README wording), and it currently outranks CLI flags. Any usher reporting of "effective config" should read the layer stack, since a CLI flag is not guaranteed to win.
- LoaderOverrides::ignore_project_config and ignore_login_requirements exist (remote app servers own auth policy), so some hosts run Codex with no project layers at all.

## Looked for, could not find

- Official (openai/codex) copy of ConfigLayerSource::precedence(); raw paths tried 404'd. Ranks come from a third-party republish.
- Code or release note tying project-layer disabling to trust.
- Which ConfigLayerSource variant the MDM config_toml_base64 key feeds.
- Requirements.toml constraint list in code.
- Ignored project-config keys list and warning text.
- analytics.enabled default; OTEL default.
- /debug-config behavior.
- AGENTS.md and .rules behavior in untrusted projects on current versions.
- Official release date for 0.166.0 (only the npm timestamp decode).
