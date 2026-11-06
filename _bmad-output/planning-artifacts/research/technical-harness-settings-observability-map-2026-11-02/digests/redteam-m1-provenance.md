# Red team M1: merged-config provenance claim

Conclusion attacked: none of Claude Code v2.1.317, Codex CLI v0.166 or goose v1.57 exposes one command or API that prints the merged effective config with the source layer of every key, so a reporting tool must model layer precedence per harness and version.

## Verdict: WEAKENED (Codex half). Claude and goose halves survive.

The Codex leg is wrong as written. The Codex app-server JSON-RPC protocol has `config/read`, documented officially as returning "the effective configuration on disk after resolving configuration layering", and it takes an `includeLayers` parameter. A Codex PR (seen only via a search snippet) says `config/read` takes an optional `cwd` and can return all layers, including project config files. So Codex has a machine-readable API that returns the merged result plus the layer stack. The conclusion lists only `/debug-config` and `codex doctor --json` and misses this.

What I could NOT confirm is a per-key origin map (key -> layer). If one exists, the Codex leg is overturned. If the response carries only the merged result plus the raw layers, a tool still diffs layers to get per-key provenance, but it no longer has to model Codex precedence by hand, because the merge is done server-side. Either way, "must reconstruct by modelling each harness's layer precedence" does not hold for Codex.

## Contrary evidence

1. Claim: Codex app-server `config/read` returns the effective config after layering; the request takes `includeLayers`.
   - Source: https://learn.chatgpt.com/docs/app-server (redirected from https://developers.openai.com/codex/app-server)
   - Publisher: OpenAI
   - pub_date: not shown on page
   - accessed: 2026-11-02
   - Confidence: high that the method and parameter exist. This is the official docs; I read the text and the example `{ "method": "config/read", "id": 60, "params": { "includeLayers": false } }`. I did not check which version added them, or whether v0.166 has them.

2. Claim: a Codex PR added layered config.toml merging to the app server. `config/read` takes an optional `cwd`, returns the effective config for that directory, and can include all layers, project files among them. Without `cwd`, only global config is used.
   - Source: search-engine summary of an openai/codex PR. The PR URL was not returned and I did not open the PR.
   - Publisher: OpenAI (openai/codex repo), indirect
   - pub_date: 2026 (per the snippet)
   - accessed: 2026-11-02
   - Confidence: medium. The cwd caveat matters: a reporting tool must pass the project cwd, or project layers are left out.

3. Claim: Codex `configRequirements/read` returns effective admin requirements loaded from requirements.toml and/or MDM (`result.requirements` is null when none are set).
   - Source: https://learn.chatgpt.com/docs/app-server
   - Publisher: OpenAI
   - pub_date: not shown
   - accessed: 2026-11-02
   - Confidence: high. This covers the managed/MDM layer through an API, which the conclusion does not mention.

4. Claim: a third-party VS Code extension, "Claude Code Settings Helper" (jpegg-dev), shows the effective value of each Claude Code setting, every level where it is defined, and the override chain.
   - Source: https://marketplace.visualstudio.com/items?itemName=jpegg-dev.claude-code-settings-helper
   - Publisher: jpegg-dev (Visual Studio Marketplace)
   - pub_date: not listed
   - accessed: 2026-11-02
   - Confidence: high that it exists; low that it is reliable. It has 103 installs and 0 reviews. It models only Enterprise, Local, Project, User and Legacy ~/.claude.json, with no CLI flags (--settings), no env vars, and no documented array-merge logic. That supports the conclusion: this is reconstruction, and incomplete reconstruction.

## Searched for and not found

- Any Claude Code command, flag, debug env var or Agent SDK call (`resolveSettings` or similar) that prints merged settings with per-key source. Search found only `/status` "Setting sources" (layer list, not per key), from third-party posts (dev.to rulestack; thepromptshelf.dev). No official source was retrieved either way, so the Claude leg rests on absence of evidence within this budget.
- Any goose command that prints per-key provenance. Official docs (https://block.github.io/goose/docs/guides/config-files) point to `goose info -v` for "all active settings and their current values", with no source attribution, and give the precedence as env > config file > defaults. That precedence is simple enough that reconstruction is easy, so the cost argument behind the conclusion is weaker for goose.
- The exact `config/read` response schema (whether it has a per-key `origins` map). I could not fetch the raw protocol source (codex-rs/app-server-protocol/src/protocol/v2.rs returned 404), and the docs do not describe the response fields. Next step: run `codex app-server generate-json-schema` on v0.166 and look up ConfigReadResponse.
- Third-party multi-harness provenance tools. None found.
- Evidence that reconstruction is infeasible (undocumented layers). Indirect only: sources disagree on Claude's layer count ("five" vs "four" files), and the Codex cwd caveat shows a hidden input to the merge.

## Recommended rewording

"Claude Code and goose have no single surface for merged config with per-key source. Codex does: app-server `config/read` (with `cwd` and `includeLayers`) returns the effective config and the layer stack, and `configRequirements/read` returns managed requirements. Whether it carries per-key origins is unverified for v0.166."
