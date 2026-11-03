# Red team M2: "off-disk config inputs must be labeled not observable"

Verdict: WEAKENED (overturned for Claude Code server-managed settings; weakened for Codex cloud requirements; goose leg unchallenged on remote config, partially weakened on model access).

The conclusion's premise is that these inputs "never land on the local disk in readable form". For Claude Code server-managed settings that is contradicted by the vendor's own docs, and for Codex cloud requirements a source-repo commit describes a local cache. The correct label is closer to "observable with caveats (cached copy may be stale, withheld, or tampered; authoritative value is server-side)" plus "harness self-report available" than a flat "not observable".

## Contrary evidence

1. Claim: Claude Code caches server-managed settings to a local file, `~/.claude/remote-settings.json`; "A startup that runs on the cache at `~/.claude/remote-settings.json` treats invalid entries the way the fetch that wrote the cache did". Cached settings apply at startup and persist through network failures. The security table lists "User edits the cached settings file" (so it is a user-editable, readable file).
   - Source: https://code.claude.com/docs/en/server-managed-settings
   - Publisher: Anthropic (Claude Code docs)
   - pub_date: not stated on page (current as of fetch; references versions up to v2.1.314)
   - accessed: 2026-11-02
   - confidence: high (primary vendor doc, verbatim path)
   - Caveats that keep part of the conclusion alive: the cache can diverge from effective settings. Cached `modelPricing` and `managedMcpServers` do not apply until the server confirms; categories of `env` (proxy/TLS, ANTHROPIC_BASE_URL, provider selection, credentials, CLAUDE_CONFIG_DIR, HOME etc.) are withheld until confirmation; `model` and OTEL exporter changes apply only at next launch; non-interactive runs (`claude -p`, Agent SDK, VS Code chat panel, desktop Code tab) apply settings needing approval "for that run only" and do NOT write them to the local cache; `/logout` deletes the cached copy; an all-invalid payload leaves the cache unchanged. So the file is readable but is not a faithful proxy for effective settings in every case.

2. Claim: Claude Code surfaces the server-delivered state through commands. `claude doctor` has a `Managed settings (remote)` line (v2.1.248+) reporting loaded / none configured / fetch failed (with whether a cached policy applies) / skipped with reason. `/status` shows "which managed source is active" and the line after a failed fetch or some skipped causes. `/permissions` shows effective permission rules. `claude --debug-file <path>` logs "Remote settings".
   - Source: https://code.claude.com/docs/en/server-managed-settings
   - Publisher: Anthropic
   - pub_date: not stated
   - accessed: 2026-11-02
   - confidence: high
   - Note: these are harness self-reports, not file reads; an observer that can invoke the CLI (or parse a debug log) gets more than "not observable".

3. Claim: Codex caches cloud requirements locally. A codex-rs commit (cloud-requirements crate) states requirements were loaded from the web on every startup and the change stores them in a local file with a 1 hour TTL, signed with a key compiled into the CLI; the cache is ignored and re-fetched if the signature is invalid, identity mismatches, TTL expired, or requirements.toml cannot be parsed from the payload. Exact filename not retrieved.
   - Source: https://git.halfakop.ru/mirrors/codex/src/commit/b364faf4ecc69401ca4c796d398d6646a6da7621/codex-rs/cloud-requirements (mirror of openai/codex; also forge.lthn.ai mirror at commit 02e9006547eaf2910fe769df955f8aaa166d2b62). Content seen only via search-result summary; direct fetch failed (ECONNREFUSED); github.com raw path 404'd.
   - Publisher: third-party mirror of OpenAI's openai/codex repo
   - pub_date: unknown
   - accessed: 2026-11-02
   - confidence: medium-low (secondhand summary, mirror, filename and current-main status unverified). Signed payload containing requirements.toml suggests readable TOML-in-file, but not confirmed.

4. Claim: Codex exposes effective requirements programmatically: "App-server clients that read effective requirements through `configRequirements/read` receive the same restriction". The doc also describes cloud-managed `config.toml` defaults and requirements.toml (TOML), and an admin "Edit as TOML" view.
   - Source: https://learn.chatgpt.com/docs/enterprise/managed-configuration (redirect target of developers.openai.com/codex/enterprise/managed-configuration)
   - Publisher: OpenAI
   - pub_date: not stated
   - accessed: 2026-11-02
   - confidence: medium (verbatim quote, but only first 100k of 128k chars read; Locations section not seen)
   - Implication: a local observer that can talk to the Codex app-server can read effective requirements, so "not observable" is too strong for requirements; still unverified for cloud-managed config defaults.

5. Claim: goose provider model access is at least partially queryable locally: `goose configure` performs a model fetch from the provider ("Model fetch complete") and shows a picker of available models for some providers; CLI-pass-through providers (Claude Code, Gemini CLI) report only "default".
   - Source: https://block.github.io/goose/docs/getting-started/providers (and https://block.github.io/goose/docs/guides/cli-providers)
   - Publisher: Block (goose docs)
   - pub_date: not stated
   - accessed: 2026-11-02
   - confidence: medium (from search summary, page not fetched directly)
   - Implication: model listing is queryable by an observer willing to make a provider API call, though that is a network query, not a file read, and the list says what models exist, not entitlement/rate limits/org policy. The conclusion's goose leg survives for a strictly file-reading observer.

## Searched for and not found

- Claude Code: a readable local cache file for server-fetched feature flags (as distinct from server-managed settings). Not searched to completion within budget; no source retrieved this run either way. That sub-claim is neither confirmed nor refuted here.
- Codex: exact cache filename/path for cloud requirements; whether cloud-managed config defaults (delivered at ChatGPT workspace sign-in) are cached locally; a `/debug-config` or `/status` command that prints effective requirements. Doc pages read did not mention them; the Locations section and last 28k chars of the managed-configuration page were not read.
- goose: any remote/server-delivered configuration mechanism (allowlist, remote config, org policy). Search returned nothing, which supports the conclusion that goose has no remote config channel, but absence in one search is weak evidence.

## Suggested restatement

"For Claude Code, server-managed settings are cached in readable JSON at `~/.claude/remote-settings.json` and self-reported via `claude doctor` / `/status`; a file reader can observe the last cached policy but must label it 'cached, may differ from effective' (withheld env keys, launch-only keys, non-interactive runs that do not cache, logout deletion). For Codex, cloud requirements appear to be cached locally with a 1h TTL (unverified filename) and are readable via app-server `configRequirements/read`; cloud-managed defaults remain unverified. For goose, no remote config channel was found; provider model access is network-queryable, not file-observable."
