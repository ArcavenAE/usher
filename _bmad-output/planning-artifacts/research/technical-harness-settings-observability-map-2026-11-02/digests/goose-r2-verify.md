# goose r2 verification digest (2026-11-02)

Budget used: 14 tool calls, 7 distinct sources (goose repo source files via raw.githubusercontent.com, GitHub REST API, goose-docs.ai, CUSTOM_DISTROS.md, Homebrew formulae API, Linux Foundation press release via search, one third-party write-up via search).

## Verdicts

G1 | verified (source), with one docs nuance
- claim: Config::default builds config_paths as system, then GOOSE_ADDITIONAL_CONFIG_FILES (split with env::split_paths), then user config.yaml in Paths::config_dir(). System path is /etc/goose/config.yaml on Unix; on Windows built from PROGRAMDATA, falling back to C:\ProgramData\goose\config.yaml. Doc comment: "Later entries take precedence over earlier ones." Most top-level keys replace wholesale; `extensions` and `providers` merge per entry (merge_nested_entries; later layer overwrites matching fields, others kept). The write target is the last entry in config_paths (user file by default); read-modify-write uses only that file, so lower-layer values are not persisted. Saves use temp file plus rename; symlinks followed at most one hop. Missing or unparseable files are skipped with a warning in `load`; `load_strict` errors instead. get_param checks an env var named by the uppercased key before the merged files; get_secret checks env, then keyring or secrets file.
  source: https://raw.githubusercontent.com/aaif-goose/goose/main/crates/goose/src/config/base.rs | aaif-goose/goose | main as of fetch | accessed 2026-11-02 | confidence high
- claim: docs state precedence env vars > config file > defaults. Docs describe no system-level or additional config files (docs lag source). Docs give Windows user path %APPDATA%\Block\goose\config\config.yaml.
  source: https://goose-docs.ai/docs/guides/config-files | goose docs | undated | accessed 2026-11-02 | confidence high
- claim: CUSTOM_DISTROS.md: "Config precedence: Environment variables -> config.yaml -> defaults"; init-config.yaml "Applied on first run if no config exists".
  source: https://raw.githubusercontent.com/aaif-goose/goose/main/CUSTOM_DISTROS.md | aaif-goose/goose | main | accessed 2026-11-02 | confidence high

G2 | verified, with precision
- claim: path_root() reads GOOSE_PATH_ROOT and uses it only if absolute. Under it: config -> <root>/config, data -> <root>/data, state -> <root>/state; plugins, agents and agents-home go under <root>/.agents (.agents/plugins, .agents/agents). No cache DirType seen in the fetched code.
  source: https://raw.githubusercontent.com/aaif-goose/goose/main/crates/goose/src/config/paths.rs | aaif-goose/goose | main | accessed 2026-11-02 | confidence high
- note: not documented on the config-files docs page or CUSTOM_DISTROS.md.

G3 | partly verified, partly unverified; one detail overturned
- claim: issue #6787 "Allow goose --config-file on cli" asks for a per-directory YAML config overriding the user config. Closed 2026-01-28 (23 minutes after creation), state_reason completed. Comments: DOsinga (COLLABORATOR) "can you not do this with recipes?"; elijahsgh (association NONE, apparently the filer) "This is solved by recipes." So "closed as solved by recipes" is the filer's own conclusion after a collaborator suggestion, not a maintainer ruling.
  source: https://api.github.com/repos/aaif-goose/goose/issues/6787 and /comments | GitHub | 2026-01-28 | accessed 2026-11-02 | confidence high
- claim: no project-level config.yaml exists: consistent with base.rs (config_paths contains only system, additional, user; no cwd lookup seen) but absence-of-feature, so confidence medium.
- claim: .goosehints / AGENTS.md / CONTEXT_FILE_NAMES: UNVERIFIED this run (not on the config-files docs page; budget did not allow fetching the hints source or docs).

G4 | partly verified; default disputed across doc versions
- claim: docs list GOOSE_MODE default "auto"; other values approve, chat, smart_approve. Docs list permission.yaml ("tool permission levels configured via goose configure") and permissions/tool_permissions.json (auto-managed).
  source: https://goose-docs.ai/docs/guides/config-files | goose docs | accessed 2026-11-02 | confidence medium-high
- dispute: a search summary of block.github.io and goose-docs.pages.dev doc versions reported one version listing smart_approve as default. Source tiebreak failed: crates/goose/src/config/goose_mode.rs returned 404 (file moved or named differently). Current default per live docs is auto; source not confirmed.
- claim: per-tool permissions apply in approve and smart_approve: only a third-party write-up ("Approve mode only respects explicit user permissions defined in permission.yaml"). Secondary; treat as unverified.
  source: https://instagit.com/block/goose/goose-tool-permissions-modes/ | instagit (unofficial) | undated | accessed 2026-11-02 | confidence low

G5 | resolved: GOOSE_DISABLE_TELEMETRY in CUSTOM_DISTROS.md appears stale; overturned as a working control
- claim: posthog.rs has no GOOSE_DISABLE_TELEMETRY. The env kill switch is GOOSE_TELEMETRY_OFF ("1" or "true", case-insensitive), which forces Some(false). Config key is TELEMETRY_ENABLED_KEY = "GOOSE_TELEMETRY_ENABLED". is_telemetry_enabled() = get_telemetry_choice().unwrap_or(false): opt-in, off unless the user chose it. Because get_param checks env first, GOOSE_TELEMETRY_ENABLED may also be settable via env (inferred from G1 semantics, not tested).
  source: https://raw.githubusercontent.com/aaif-goose/goose/main/crates/goose/src/posthog.rs | aaif-goose/goose | main | accessed 2026-11-02 | confidence high
- claim: CUSTOM_DISTROS.md says "Set GOOSE_DISABLE_TELEMETRY=1"; docs say GOOSE_TELEMETRY_ENABLED default false. Docs match source; CUSTOM_DISTROS.md names a variable the source does not read.
  sources: CUSTOM_DISTROS.md and goose-docs.ai config-files (above) | confidence high

G6 | verified merged, scope narrower than "entries dropped"
- claim: PR #9439 "fix: preserve unparseable extension entries during config refresh", merged true, merged_at 2026-06-11T05:24:47Z, base main, body "Fixes #8970". Scope: malformed extension YAML entries were silently removed when another extension was toggled in the UI; the fix keeps them.
  source: https://api.github.com/repos/aaif-goose/goose/pulls/9439 | GitHub | 2026-06-11 | accessed 2026-11-02 | confidence high

G7 | verified
- claim: Linux Foundation announced the Agentic AI Foundation (AAIF) on 2025-12-09 with founding projects Anthropic MCP, Block goose, OpenAI AGENTS.md; Block is a platinum member; Manik Surtani (Block Head of Open Source) quoted.
  source: https://www.linuxfoundation.org/press/linux-foundation-announces-the-formation-of-the-agentic-ai-foundation | Linux Foundation | 2025-12-09 | accessed 2026-11-02 (via search result summary, page not fetched directly) | confidence high
- corroboration: repo org aaif-goose and Homebrew source URL (G8) confirm the move off block/goose.

G8 | verified version; cadence only partly evidenced
- claim: v1.57.0 published 2026-10-28T19:26:10Z; v1.56.0 published 2026-10-23T15:49:34Z (5 days apart).
  source: https://api.github.com/repos/aaif-goose/goose/releases?per_page=6 | GitHub | accessed 2026-11-02 | confidence high
- second source: Homebrew formula block-goose-cli stable 1.57.0, homepage https://goose-docs.ai/, source https://github.com/aaif-goose/goose/archive/refs/tags/v1.57.0.tar.gz
  source: https://formulae.brew.sh/api/formula/block-goose-cli.json | Homebrew | accessed 2026-11-02 | confidence high
- "roughly weekly" rests on two data points only; unverified beyond that.

G9 | partly verified; sources-of-values unverified
- claim: docs: `goose info -v` shows active settings and their current values.
  source: https://goose-docs.ai/docs/guides/config-files | accessed 2026-11-02 | confidence medium
- whether it shows merged effective values (env plus layered files) and per-value source: not found. Unverified.

## Lead: separate desktop settings store (Goose2 or similar)

- Not found. v1.57.0 release notes mention only the existing desktop app (Linux window icons; agent starts once after a fork); no goose2 or settings-store change.
- Docs: desktop Settings page saves providers without restart; active sessions keep their provider. No separate store described. CUSTOM_DISTROS.md names only ~/.config/goose/config.yaml and keyring or ~/.config/goose/secrets.yaml.
- Inference (not verified): desktop and CLI share the same Config (base.rs) since the PR #9439 bug reproduced from desktop UI toggles against config.yaml.

## Looked for, could not find

- Source location of the GooseMode enum and its default (goose_mode.rs 404).
- Source or docs for .goosehints, AGENTS.md and CONTEXT_FILE_NAMES (not fetched; budget).
- Whether `goose info -v` reports value sources or only values.
- Release cadence beyond two consecutive releases.
- Any goose2 / new desktop app with its own settings store.
- The final ~4.4k chars of base.rs (test module) were not read.
