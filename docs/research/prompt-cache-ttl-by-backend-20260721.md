# Prompt cache TTL by backend — what usher would have to detect and recommend

- **Date:** 2026-07-21
- **Status:** captured research, not yet a kos finding (no probe brief was
  filed; this arrived from live debugging in i-orc). Promote via the normal
  question → probe → finding loop if it earns a charter position.
- **Confidence:** high for the observed build, low for durability — see
  *Staleness risks* at the end. One value in this document is
  server-controlled and can change with no client release.
- **Method:** read the decision logic out of the installed Claude Code
  binary rather than the docs, after our own internal docs and a stale
  source copy disagreed with each other.

## Provenance

Everything below was extracted from the installed executable at
`~/.local/share/claude/versions/2.1.217`:

- `VERSION` 2.1.217, `BUILD_TIME` 2026-07-21T18:35:19Z,
  `GIT_SHA` 9963b018d22c3e6659c99e135e687870301b5c67

Do **not** rely on secondhand copies of the source for this
area. A snapshot predating the shipping build may contain
only the Bedrock-specific variable and lack `ENABLE_PROMPT_CACHING_1H` and
`FORCE_PROMPT_CACHING_5M` entirely, which reads as "our docs are wrong"
until you check the shipping binary and find all four present.

## Why this belongs in usher

This is a worked example of usher's function-2 output shape: a setting that
is (a) invisible in normal use, (b) worth real money, (c) *backend-dependent
in a way users get wrong*, and (d) mechanically detectable from config. It
also demonstrates the failure mode a naive compliance template would hit —
"subscription means 1 hour" is wrong in at least three ways, and one of the
three is not knowable from the client at all.

## 1. The decision, in precedence order

One function decides the TTL for every request. Precedence matters more than
any individual variable:

1. `FORCE_PROMPT_CACHING_5M` truthy → **5m**. Beats everything below.
2. `ENABLE_PROMPT_CACHING_1H` truthy → **1h**, on *any* backend. This is
   checked before any provider test. Or, only when the provider string is
   literally `bedrock`, `ENABLE_PROMPT_CACHING_1H_BEDROCK` truthy → 1h.
3. Not a claude.ai subscriber, **or** currently in overage → **5m**.
4. Subscriber, not in overage → 1h **only if the request's query source
   matches an allowlist**.

The allowlist default is `["repl_main_thread*", "sdk", "auto_mode",
"memdir_relevance"]`, sourced from a server gate named
`tengu_prompt_cache_1h_config` and latched once per session for cache
stability.

When the function returns false the client sends no TTL field at all and the
server default (5 minutes) applies. TTL is inactivity-based — every cache
hit resets the timer.

## 2. Provider identity, and why it decides which variable works

Provider comes from an ordered chain. The order is load-bearing:

| Env set | Provider string |
|---|---|
| gateway configured | `gateway` (short-circuits everything) |
| `CLAUDE_CODE_USE_BEDROCK` | `bedrock` |
| `CLAUDE_CODE_USE_FOUNDRY` | `foundry` |
| `CLAUDE_CODE_USE_ANTHROPIC_AWS` | `anthropicAws` |
| `CLAUDE_CODE_USE_ANTHROPIC_GOOGLE_CLOUD` | `anthropicGoogleCloud` |
| `CLAUDE_CODE_USE_MANTLE` | `mantle` |
| `CLAUDE_CODE_USE_VERTEX` | `vertex` |
| none of the above | `firstParty` |

Because `ENABLE_PROMPT_CACHING_1H_BEDROCK` requires the provider to be
exactly `bedrock`, it is **inert** on `anthropicAws`, and its behavior on
Mantle depends on how Mantle was configured (below).

## 3. The four backends asked about

| Backend | Provider | Default TTL | Path to 1h | Notes |
|---|---|---|---|---|
| **Claude Platform for AWS** | `anthropicAws` | 5m | `ENABLE_PROMPT_CACHING_1H=1` **only** | The `_BEDROCK` variable does nothing here. Easy misconfiguration to ship and never notice. |
| **Claude Enterprise** (claude.ai OAuth) | `firstParty` | **1h automatic**, but only for allowlisted sources and only outside overage | nothing to set | Setting the variable is counterproductive — see §6. |
| **Bedrock** | `bedrock` | 5m | either variable works | Cache support varies by model and region; zero cache-token counters is the symptom of an unsupported combination. |
| **Bedrock Mantle** | `bedrock` **or** `mantle` — depends on config | 5m | generic always; `_BEDROCK` only in the paired config | See below. |

**Mantle's split personality.** Because `CLAUDE_CODE_USE_BEDROCK` is tested
first in the chain, a paired `USE_BEDROCK=1 + USE_MANTLE=1` install reports
as `bedrock` and honors `ENABLE_PROMPT_CACHING_1H_BEDROCK`. A solo
`USE_MANTLE=1` install reports as `mantle` and silently ignores it. Both
configurations exist in the field (our own doctor has a code comment
handling the paired case, and Mantle uses `anthropic.*` model IDs against
Bedrock's `us.anthropic.*`). Practical rule: **prefer the generic variable
everywhere.** The Bedrock-specific one buys nothing except the ability to
scope an opt-in to Bedrock inside a settings file shared across backends.

## 4. Claude Code vs. its other query sources

Not all traffic from one Claude Code process is equal. Sources are
classified three ways:

- **main** — `repl_main_thread*` and `sdk`
- **subagent** — `agent:*` and `hook_agent`
- **auxiliary** — everything else

The build carries roughly thirty auxiliary source literals: `compact`,
`agent_classifier`, `agent_summary`, `agent_namer`, `insights`,
`extract_memories`, `generate_session_title`, `prompt_suggestion`,
`side_question`, `web_search_tool`, `web_fetch_apply`, `hook_prompt`,
`bash_extract_prefix`, `permission_explainer`, `model_validation`,
`plugin_eval_judge`, and others.

**The consequence usher should encode:** on a subscription seat, only main,
plus `auto_mode` and `memdir_relevance`, get the 1-hour TTL by default.
Subagent fan-out and every auxiliary call run at 5 minutes. So "this seat is
on Enterprise, therefore it caches for an hour" is false for a meaningful
share of its traffic, and the share grows with how heavily the user runs
subagents and workflows.

## 5. Headless: `claude -p`, streaming, launchd

`claude -p` reports query source **`sdk`**, which is in the default
allowlist and classified as **main**. So headless is treated like the
interactive REPL, not like a subagent. There is no separate print or
headless source literal in the build. Streaming output format
(`--output-format stream-json`) is orthogonal and does not affect caching.

That makes periodic headless jobs the strongest case for the 1-hour TTL
anywhere, because the cache is server-side and keyed on the prompt prefix —
a fresh process per run can still hit an entry written by the previous run,
provided the prefix is byte-identical and unexpired. Therefore **match the
TTL to the schedule interval**:

| Interval | Right TTL | Why |
|---|---|---|
| < 5 min | 5m (default) | Already hits. 1h adds the write premium for nothing. |
| 5–60 min | **1h** | The only band where the default goes cold every single run. Largest available saving. |
| > 60 min | 5m | Cold either way. 1h pays double on every write and never collects a read. |

**The launchd trap, which is the most likely real-world defect:** launchd
jobs do not inherit the interactive shell environment. Anything exported
from `.zshrc` or `.zprofile` is invisible to the scheduled job, so
`ENABLE_PROMPT_CACHING_1H` set there silently does nothing in the runs that
actually matter while looking correct in a terminal. It must live in the
plist's `EnvironmentVariables` dictionary, or in the `env` block of
`~/.claude/settings.json`, which Claude Code reads itself. The same trap
applies to backend selection: a launchd job missing `CLAUDE_CODE_USE_BEDROCK`
quietly runs against a different provider — and therefore a different
default TTL and a different bill — than the interactive sessions on the same
machine.

Cross-run hits also require prefix stability, which is easy to lose
headlessly: model and effort are part of the cache key, so pin them instead
of relying on aliases (alias re-resolution on Bedrock can hand you a
behavior change and a full cache miss on the same day); a varying
`--append-system-prompt`, inconsistently-connecting MCP servers, and dynamic
`CLAUDE.md` content each break it too.

One feedback loop worth instrumenting: on subscription, scheduled `-p` jobs
draw from the same 5-hour and 7-day budget as interactive work. Enough
scheduled consumption pushes the seat into overage, and overage flips the
automatic 1h path back to 5 minutes — making every subsequent run more
expensive precisely when the seat is already over. `-p` returns
`total_cost_usd` per invocation, which is the cheap thing to log.

## 6. Economics, and when 1h is the wrong answer

1-hour cache writes bill at 2x base input against 1.25x for 5-minute, so the
break-even is roughly three reads per write. Below that, the opt-in loses
money.

Three non-obvious downsides, all consequences of the precedence order in §1:

1. **It bypasses the allowlist.** On a subscription seat,
   `ENABLE_PROMPT_CACHING_1H=1` returns true at step 2, so 1h extends to
   every subagent and auxiliary call — contexts typically written once and
   never re-read. That is close to pure 2x loss, and it scales with fan-out.
2. **It bypasses the overage guard.** The automatic path deliberately drops
   to 5m during overage; the variable short-circuits that check, so the seat
   keeps paying the premium exactly when it is over budget.
3. **A longer TTL does not protect against invalidation.** It extends idle
   survival only. A mid-session model or effort switch still re-reads
   everything uncached.

So on subscription and Enterprise the correct configuration is *nothing
set*. The variable is for pay-per-token backends, and even there only when
the read:write ratio justifies it.

## 7. Candidate usher checks

Falling straight out of the above, each detectable from config plus a
provider probe:

- **Inert variable:** `ENABLE_PROMPT_CACHING_1H_BEDROCK` set while the
  resolved provider is `anthropicAws`, or Mantle configured solo. Reports as
  enabled, does nothing.
- **Dead config:** `ENABLE_PROMPT_CACHING_1H` and `FORCE_PROMPT_CACHING_5M`
  both set. The 5m force wins; the 1h opt-in is decorative.
- **Counterproductive opt-in:** `ENABLE_PROMPT_CACHING_1H` set on a
  subscription/Enterprise seat. Disables two cost controls and buys nothing
  on the main path.
- **Scheduler/env mismatch:** a caching or backend variable present in shell
  rc files but absent from both the launchd plist and `settings.json`, on a
  machine that has Claude Code launchd jobs. This is the highest-value check
  in the list because the symptom is invisible.
- **Interval/TTL mismatch:** launchd `StartInterval` in the 5–60 minute band
  with a 5m effective TTL (recommend 1h), or above 60 minutes with 1h forced
  (recommend removing it).
- **Prefix instability:** unpinned model alias, or varying
  `--append-system-prompt`, on a scheduled job.
- **Per-model cache kill switches:** `DISABLE_PROMPT_CACHING` and the
  per-model variants — Haiku, Sonnet, Opus, Fable, **and Mythos**. Any set
  is a ~10x cost and rate-limit event and is almost never intended outside
  active debugging.

Note the recommendation direction differs by backend for the *same* detected
setting, which is a good early test of whether the compliance-template
design can express backend-conditional rules rather than flat assertions.

## 8. Staleness risks

- **The allowlist is server-controlled.** `tengu_prompt_cache_1h_config` is
  a gate with a client-side default. Anthropic can widen or narrow it with
  no client release, so any usher rule asserting which sources get 1h is
  asserting something it cannot verify locally. Prefer reporting observed
  cache-token counters over predicting TTL from config alone.
- **This is minified-bundle observation of one pinned build** (2.1.217).
  Symbol names and structure will churn. What is worth re-verifying on
  upgrade is the precedence order and the allowlist default, not the
  variable names, which appear stable.
- **Two of our own artifacts are already wrong** in ways this corrects: the
  peu course doc describes `ENABLE_PROMPT_CACHING_1H` as a pay-per-token
  opt-in when it is unconditional and provider-independent, and neither that
  doc nor the peu doctor mentions the subscription allowlist at all. Any
  usher content sourced from them inherits both errors.

## Sources

- Installed binary 2.1.217 (see *Provenance*) — decision function, provider
  chain, source classifier, query-source literals, gate default.
- `i-orc/peu/14_Claude_Code_Deep_Dive/claude-caching-and-limits.md` — the
  2x/1.25x write-premium figures and the Bedrock model/region caveat.
- `i-orc/peu/doctor.just` — existing checks, and the Mantle pairing comment
  confirming both configurations are in use.
