# Extract: harness settings research and the aae-orc index

Source material for the usher PRD. Research = `_bmad-output/planning-artifacts/research/technical-harness-settings-observability-map-2026-11-02/research.md` (sections and [n] citations as in its appendix; "probe" = its `imports/` files). Index = `docs/research/orc-index.md`; orc paths are relative to the orchestrator checkout root. Extracted 2026-11-03.

## 1. Per harness

### Claude Code (2.1.317 installed and current)

- Layers, highest first: managed, `--settings`, `.claude/settings.local.json`, `.claude/settings.json`, `~/.claude/settings.json`. (verified; research section 1, [1])
- Lists merge across files except `fallbackModel`, `modelPicker`, `availableModels`, `modelSettings`. (verified; section 1, [1])
- Permission rules evaluate deny, then ask, then allow; first match wins regardless of scope. (verified; section 1, [3])
- Environment variables are not a layer: each variable resolves as a pair with its settings key (`ANTHROPIC_MODEL` beats `model` in any file); dedicated flags such as `--model` override per key; `CLAUDE_CONFIG_DIR` relocates the user home. (verified; section 1, [5])
- Per-key Scope column in the settings index; keys outside project scope are ignored in the shared project file; about a dozen security keys let a stricter lower-scope value beat managed. (verified; section 1, [1])
- Managed tier ranks remote over MDM (plist `com.anthropic.claudecode`, HKLM) over `managed-settings.json` plus `managed-settings.d/*.json` over HKCU; default first-source-wins skips the others without warning; `managedSourcesBehavior: "merge"` from v2.1.242; remote polls hourly, MDM every 30 minutes, files hot-reload. (unverified, medium: one docs page, not confirmed in changelog; section 1, [2])
- Layer limits: `autoMode` read only from user, managed and `--settings`, additive across scopes; `defaultMode` `auto` and `bypassPermissions` do not take effect from project or local files; OTEL variables and `CLAUDE_CONFIG_DIR` ignored in project and local settings. (verified; section 1, [4][1][5])
- Folder trust lives in `~/.claude.json` as `projects[<repo root>].hasTrustDialogAccepted`. (verified; section 1, [3][1])
- Resolved view: none with per-key provenance; `/status` "doesn't show which file supplied each key". (verified; section 1, [1])
- Partial views usher can obtain: `/permissions` (each rule with source file) [3]; `claude auto-mode config` (effective allow, soft_deny, hard_deny, environment lists as JSON, no per-entry source) [4][11]; `/status` (managed source, and skipped sources since v2.1.242) [2]; `claude doctor` (rejected entries; remote managed state as loaded, none, fetch failed or skipped, v2.1.248+) [6][39]; statusline input (effective model, effort, version, context window, no sources) [10]. (verified; section 1)
- Must reconstruct from files: the documented stack and per-key scope table, plus the server-managed cache `~/.claude/remote-settings.json` (present on kinu). (verified; section 1 and executive summary, [1][2][39][11]; probe-remote-settings-cache)
- Attribution of effective auto-mode entries needs a diff against files: on kinu effective lists held 27 allow and 45 environment entries against 16 and 21 shipped defaults. (verified; section 1, [11]; probe-resolved-views)
- Observable only by effect: feature flags fetched from Anthropic (off under `DISABLE_TELEMETRY` or a third-party provider); prompt cache TTL in force. (unverified, medium; section 1, [5])

### codex (installed 0.160.1, latest stable 0.166.0)

- Layers, highest first: CLI flags and `-c`, project `.codex/config.toml` from repo root to cwd (closest wins), selected profile, user `~/.codex/config.toml`, cloud-managed defaults, system `/etc/codex/config.toml`, built-in defaults. (verified; section 2, [12][13][15])
- Numeric ranks, a lowest `Mdm` base layer, and two legacy `managed_config.toml` layers ranked above CLI flags come only from a third-party crate copy. (unverified, medium-low; section 2, [16])
- Project layers documented as loading only for trusted projects; code tying this to trust not found. (unverified; section 2, [12][15])
- Managed tier is two mechanisms: managed defaults (`managed_config.toml`, MDM `config_toml_base64`) set values; requirements (cloud, MDM `requirements_toml_base64`, `/etc/codex/requirements.toml`) constrain approval policy, sandbox mode, web search, MCP allowlist, features and hooks, falling back to a compliant value with a notice; tracked separately from the stack in source. (verified; section 2, [14][15])
- Resolved view with provenance: app-server JSON-RPC `config/read` with `cwd` and `includeLayers` returns `config`, `layers` (each with `disabledReason`) and a required `origins` map of key to layer {name, version}; without `cwd`, project layers are omitted. (verified, high; overturns round 1; section 2, [40][11]; probe-codex-app-server-schema)
- Layer source names in the 0.160.1 schema: packagedDefaults, mdm, system, enterpriseManaged, user, project, sessionFlags, legacyManagedConfigTomlFromFile, legacyManagedConfigTomlFromMdm. (verified; probe-codex-app-server-schema, [11])
- `configRequirements/read` returns requirements from `requirements.toml` and/or MDM. (verified; section 2, [40][11])
- Secondary views: `codex doctor --json` (redacted config-load section: scope, CODEX_HOME, feature-flag overrides, MCP servers, model provider; also installation section naming the package manager) and `codex features list` (each flag's stage and effective state). (verified; section 2, [11]; probe-resolved-views)
- `/debug-config` lists the layer stack per layer, including session-flag and MDM values since February 2026. (unverified this run; section 2, [37])
- Cloud requirements and defaults arrive at ChatGPT sign-in; requirements cached as a signed, identity-matched entry refreshed in the background; filename and lifetime unsourced; whether cloud defaults are cached is unknown. (unverified; section 2, [14])
- Must reconstruct from files: AGENTS.md chain (CODEX_HOME, then one file per directory root to cwd, `project_doc_max_bytes` 32 KiB) and skills dirs (`.agents/skills` to repo root, `~/.agents/skills`, `/etc/codex/skills`). (verified as docs; section 2, [23][12])
- Observable only by effect: cloud defaults (listed as not observable in Recommendation 2). (unverified; Recommendations, [14])

### goose (v1.57.0, not installed on kinu, no local probe)

- Layers, later wins: system `/etc/goose/config.yaml` (ProgramData on Windows), files in `GOOSE_ADDITIONAL_CONFIG_FILES`, user `~/.config/goose/config.yaml`; `extensions` and `providers` merge per entry, other top-level keys replaced; goose writes only the user file. (verified from source; section 3, [26])
- An environment variable named by the uppercased key is checked before any file. (verified; section 3, [26][25])
- Docs describe only env over user over defaults; the system layer (the only admin path found) is undocumented. (verified; section 3, [24])
- `GOOSE_PATH_ROOT`, when absolute, relocates config, data and state. (verified; section 3, [27])
- No project-level config file. (verified; section 3, [26][33])
- Secrets resolve env, then keyring, then `secrets.yaml`. (verified as docs; section 3, [24])
- Per-tool permissions in `permission.yaml` and `permissions/tool_permissions.json`. (verified as docs; section 3, [36])
- Resolved view: `goose info -v` shows active settings with current values; whether merged and from which layer is unknown; `goose session diagnostics` writes a JSON report including config files. (unverified, low on provenance; section 3, [35])
- Must reconstruct from files: the whole short stack, read from source not docs. (verified; executive summary, [26][24])
- Observable only by effect: silent (no remote config channel found; Contrary evidence).

## 2. Capability candidates for a settings map and drift check

- Report, per harness, the effective value of each observed key and the layer that supplied it, marked as harness-reported (codex `origins`) or computed by reconstruction (Claude Code, goose). (section 1-3; Recommendation 1)
- Report the project-scoped view relative to a given working directory, since codex omits project layers without `cwd` and Claude Code and codex both walk directories. (section 2, [11]; section 1, [1])
- Report layers that were present but disabled or skipped, with the reason where the harness gives one (codex `disabledReason`; Claude skipped managed sources, rejected entries). (section 1, [2][6]; section 2, [11])
- Report where a file's value differs from effective state: Claude skipped or held-back entries, codex ignored project keys, goose env overriding files. (executive summary "What drives it", [39][12][26])
- Report managed-tier state: which managed source is active, whether remote managed settings loaded, and codex requirements as constraints distinct from values. (section 1, [2][39]; section 2, [14][40])
- Report per observation: harness version, method used (harness query, file read, reconstruction), and an explicit list of what could not be observed. (Recommendation 2)
- Report drift as declared profile against resolved state where a resolved view exists (`config/read`; `claude auto-mode config`), and against files only where none exists. (Recommendation 4)
- Report when a profile names a retired or hard-failing value (codex `approval_policy = "untrusted"`). (Recommendation 6, [17])
- Report what a stale key does per harness: warn, alias, hard error, or silent skip. (Recommendation 3)
- Report folder trust state, since it gates which layers load (Claude `hasTrustDialogAccepted`; codex trusted projects). (section 1, [3]; section 2, [12])
- Report instruction and skill discovery per harness, including duplicate discovery where a harness reads both `.claude/skills/` and `.agents/skills/`. (orc `_kos/findings/finding-087-cross-harness-discovery-verification.md`)
- Report the context window in force against the configured autocompact window, since `CLAUDE_CODE_AUTO_COMPACT_WINDOW` can lower but never raise the model's window. (orc `_kos/findings/finding-055-bedrock-fable5-context-window-clamp.md`)

## 3. Install and update failure modes for install health

- Stale process after self-update. Symptom: "Login successful" then "Not logged in" in a loop, update banner "Restart to update". Check must observe: running process start time against the binary or symlink mtime and version. (orc `_kos/findings/finding-182-claude-self-update-wedges-running-seat.md`)
- Installed copy drifts behind its source. Symptom: the documented path runs an old copy missing merged fixes; neither copy names the other. Check must observe: content hash or version of each known copy against the source, and which copy the docs point at. (orc `_kos/findings/finding-aae-orc-gntz-installed-copy-drifts-from-merged-script.md`)
- Installed version reports itself current. Symptom: `install.sh` copies (cp, not symlink) and `resolve_version` reads the installed version file first, so a stale machine looks fine; a fix shipped without a version bump is invisible. Check must observe: installed version file against `tools/version.txt`, and content, not only the version string. (bd `aae-orc-9va5`; orc `tools/install.sh`)
- Update check that errors instead of answering. Symptom: `brew outdated <name>` prints "Formulae found in multiple taps" for a doubly-tapped formula; 6 of 7 affected tools were behind. Check must observe: whether the check ran, distinguishing error from "current", and the tap recorded in `INSTALL_RECEIPT.json`. (orc `_kos/findings/finding-221-brew-tap-ambiguity-hides-an-upgrade.md`)
- Stale bindings after a version transition. Symptom: 68 stale `bmad-*.md` files left in `~/.claude/commands/` pointing into the old pack. Check must observe: files a pack manager wrote for a prior version that still resolve. (orc `_kos/findings/finding-027-bmad-6.3.0-sideshow-install-end-to-end.md`)
- Status count undercount. Symptom: `sideshow status` reports synced 61 when 97 are synced, by prefix matching. Check must observe: actual installed artifacts, not a heuristic count. (orc `_kos/findings/finding-027-bmad-6.3.0-sideshow-install-end-to-end.md`)
- Old version left on disk. Symptom: prior pack directory stays after registry replacement, no pruning. Check must observe: unreferenced pack versions. (orc `_kos/findings/finding-027-bmad-6.3.0-sideshow-install-end-to-end.md`)
- Structural migration presented as a point release. Symptom: bmad 6.2.2 to 6.3.0 moved 1306 paths. Check must observe: layout change across versions, not version number alone. (orc `_kos/findings/finding-025-bmad-6.3.0-sideshow-install-probe.md`)
- Custom-built skill never resolves. Symptom: a bmb-built agent has no binding path from build output to a session. Check must observe: whether a built skill is discoverable by the harness. (orc `_kos/findings/finding-076-sideshow-custom-skill-binding-gap.md`)
- Plugin source gone. Symptom: with a directory marketplace, the source path is the load path; a dangling source shows "failed to load". Check must observe: that each enabled plugin's marketplace source path exists. (orc `_kos/findings/finding-091-claude-plugin-identity-directory-marketplace.md`)
- Skills-dir plugin active without consent. Symptom: a plugin in `$CONFIG/skills/<name>/` lists as loaded with no settings entry. Check must observe: loaded plugins with no recorded enablement. (orc `_kos/findings/finding-091-claude-plugin-identity-directory-marketplace.md`)
- Read-only install updated in name only. Symptom: `git checkout` on a read-only `contents/` exits 0 but leaves old files while `HEAD` moves; an `origin` pointing at a local checkout never sees GitHub. Check must observe: `git status --porcelain contents/` empty and the origin URL. (wardrobe `README.md`, "Updating an install")
- Client env not present. Symptom: bd needs `~/.beads/shared-server/client-env.sh` sourced for the server host, port, user and password. Check must observe: the file and the variables it sets. The file was absent on the host writing this extract while `bd show` still answered, so the variables may come from elsewhere there. (orc `.claude/skills/task-workflow/SKILL.md`, "bd server"; local check)
- Unsigned artifact. Symptom: signing deferred (`signing.status: DEFERRED`), no cosign or minisign installed. Check must observe: signature status of installed packs. (orc `_kos/findings/finding-025-bmad-6.3.0-sideshow-install-probe.md`)

## 4. Compatibility and versioning constraints

- Split the map into a slow layer (scopes, precedence, file paths, merge rules) and a fast layer (keys, defaults, value sets); re-verify the fast layer per harness release, keyed to the version install health detects. (Recommendation 3; Cross-dimension insights)
- Stamp every observation with harness version and method. (Recommendation 2)
- Claude Code: 2.1.317 installed and current; about ten releases in thirteen days, adding env vars and keys and changing meanings; precedence fixes landed 2.1.304 to 2.1.317. (section 1, [7][8][11])
- Claude Code version gates named: `managedSourcesBehavior` from 2.1.242, skipped sources in `/status` from 2.1.242, doctor remote state from 2.1.248, `$defaults` behavior change at 2.1.316. (section 1, [2][39][7])
- Claude plugin floor: directory marketplaces and v2 `installed_plugins.json` verified on 2.1.220; older unverified. (orc `_kos/findings/finding-091-claude-plugin-identity-directory-marketplace.md`)
- codex: installed 0.160.1, latest stable 0.166.0 (about 2026-10-29), six minors behind; legacy profiles removed in rust-v0.134.0 (2026-05-26); `approval_policy = "untrusted"` retired 2026-08-20 and now errors. (section 2, [17][18][19][20][11])
- goose: v1.57.0 published 2026-10-28; lead/worker mode and its successor removed in recent minors; weekly cadence rests on few data points; not installed locally. (section 3, [30][31])
- No vendor states a config stability or deprecation policy; most removed forms warn or are skipped rather than fail. (Contrary evidence, [42][43])
- Staleness windows: version, layers, resolved-view, effect-only 1 month; precedence and docs-gap 3 months; cadence 6 months; 14 of 27 claims past window on day of writing; earliest dated re-check 2026-12-01. (Staleness map)
- Map entries should cite source or changelog, with docs as a second source, because docs lag source in two of three harnesses. (Cross-dimension insights, [24][26][29][28][12])
- A community JSON schema for Claude Code settings exists and says it can lag releases. (Contrary evidence, [41])
- Harness market moves monthly; re-check any rating before building on it. (orc `_kos/findings/finding-aae-orc-2w6k-harness-set-beyond-claude-and-codex.md`)

## 5. Hazards for an observer

- `claude auto-mode` is "Inspect or reset auto mode configuration": the reset path writes. (probe-versions-help)
- `claude doctor` "reads settings files in the current directory without a trust prompt"; `/doctor` in a session "can also fix issues". (probe-versions-help)
- `claude purge`, `claude install`, `claude respawn` and `codex update` change state; an observer must not invoke them. (probe-versions-help)
- codex `doctor --json` reports an app-server daemon state dir and an update-loop pid file, so a background update loop exists; whether calling `config/read` starts the daemon or the update loop is silent. (probe-resolved-views; research silent)
- `codex app-server generate-json-schema --out <dir>` writes schema files only to the given dir and reads no config values. (probe-codex-app-server-schema)
- Claude self-update lands under a running process; a running session can diverge from the installed binary. (orc `_kos/findings/finding-182-claude-self-update-wedges-running-seat.md`)
- Claude settings files hot-reload, so a file read is a point-in-time sample of a moving target. (section 1, [2]; orc `_kos/findings/finding-024-integrated-security-model.md`)
- `claude plugin update` and `marketplace update` mutate per scope; observe with `list` or `details`, not update. (orc `_kos/findings/finding-091-claude-plugin-identity-directory-marketplace.md`)
- File differs from effect, Claude: bad entries skipped; some server-managed `env` categories, `modelPricing` and `managedMcpServers` held until server confirms; `model` and OTEL exporter changes apply at next launch; non-interactive runs do not write approval-requiring settings to the cache; `/logout` deletes the cache for API-key or token sessions. (section 1, [39])
- File differs from effect, Claude: first-source-wins managed tier skips other sources without warning. (section 1, [2])
- File differs from effect, Claude: `CLAUDE_CODE_AUTO_COMPACT_WINDOW=850000` was inert on a 200k Bedrock model; statusline percentage used the wrong denominator. (orc `_kos/findings/finding-055-bedrock-fable5-context-window-clamp.md`)
- File differs from effect, codex: project config ignores `model_provider(s)`, `notify`, `profile(s)`, `otel` and base-URL keys; requirements replace a non-compliant value with a compliant one. (section 2, [12][14])
- File differs from effect, goose: env var by key beats every file. (section 3, [26])
- Issue #70181 (empty or 304 server response zeroed local managed deny rules) closed stale with no fix. (section 1, [9])
- Device management can act on the same machine's settings (iTerm2 profiles lost to an MDM Software Updates reinstall). (index section 7)
- Silent skipping is the larger hazard: nothing tells the user a setting stopped applying. (Contrary evidence)

## 6. Disputed or unverified claims the PRD must not rely on

- Whether omitting `"$defaults"` discards Claude's built-in auto-mode rules on 2.1.317 (disputed: docs say yes [4], v2.1.316 changelog says no [7]). Test: `claude auto-mode config` under a scratch home with an `autoMode.allow` list lacking `"$defaults"`, diffed against `claude auto-mode defaults`. (Open questions)
- Whether codex MDM `config_toml_base64` feeds the high legacy layer (beats user and `-c`) or the low `Mdm` base layer (disputed [14][16]). Test: `config/read` `origins` on a machine with an MDM profile, or read current source. (Open questions)
- Whether codex loads project skills, rules and AGENTS.md in untrusted projects (disputed [12][21]). Test: one scratch repo, untrusted, inspected with `config/read`. (Open questions)
- `GOOSE_MODE` default: `auto` [25] or `smart_approve` [38] (disputed). Test: fresh goose install on a scratch home, `goose info -v`. (section 3; Open questions)
- Whether codex prints a startup warning for ignored project keys or is silent (unverified; the brief says "silently"). Test: one scratch-repo run with an ignored key in `.codex/config.toml`. (Recommendation 5)
- Claude managed-tier ordering, first-wins and poll intervals (medium, one docs page). Test: silent in the research; a changelog confirmation is the named gap. (section 1, [2])
- codex numeric layer ranks and the legacy layers above CLI flags (third-party crate only). Test: read the official `codex-rs` source for ranks. (section 2, [16])
- codex trust gating of project layers (documented, not found in code). Test: `config/read` with `cwd` in an untrusted repo. (section 2, [12][15])
- codex `/debug-config` output and the analytics exporter default (unverified beyond docs). Test: run `/debug-config` in a scratch session; read `analytics.enabled` effective value via `config/read`. (Open questions)
- Whether `goose info -v` shows merged values and sources; goose instruction-file discovery. Test: install goose on a test host (needs operator approval) and set one key at two layers. (Open questions)
- Whether Claude feature flags have a local cache (no evidence either way). Test: silent in the research. (Open questions)
- codex cloud requirements cache filename and lifetime; whether cloud defaults are cached (unsourced). Test: silent in the research. (section 2, [14])
- Per-harness stale-key behavior table (partly known). Test: silent; one retired key per harness under a scratch home would enumerate it. (Open questions)

## 7. Constraints (index section 8) and field notes (index section 7)

- Per-person data (I-140) never enters git. (index section 8)
- Client names stay out. (index section 8)
- Reporting stays off by default. (index section 8; matches `val-reporting-off-by-default`, index section 3)
- Field note: lost iTerm2 custom profiles traced to the device-management tool's Software Updates item reinstalling the app, not an app upgrade; profiles were absent from the device-guard inventory; usher and device management can both act on one machine's settings. (index section 7, 2026-11-02)
- Field note: credentials are per host; each host has its own Bedrock key; a read-only Datadog token is to be issued one per host and never copied between hosts. (index section 7)
