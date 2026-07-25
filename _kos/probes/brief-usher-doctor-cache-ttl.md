# Exploration Brief — usher doctor: read-only harness inspection, first check = prompt-cache TTL

- **Date filed:** 2026-07-25 (party-mode session; "primary research"
  per user)
- **Status:** filed, not started
- **Confidence of any code produced:** placeholder (probe instrument)

## Hypothesis

A read-only `usher doctor` — inspect local harness config, report
what's set and what it costs, phone nothing home — is buildable now,
delivers value on our own fleet immediately, and teaches usher's
function-2 shape (observation → assessment → recommendation) with
zero consent surface: B1 is satisfied by construction because nothing
is transmitted.

## Method

1. Turn `docs/research/prompt-cache-ttl-by-backend-20260721.md` into
   the first executable check: read the operator's Claude Code
   config/env (FORCE_PROMPT_CACHING_5M, ENABLE_PROMPT_CACHING_1H*,
   provider identity, subscriber state where detectable), walk the
   documented precedence, report the effective TTL and what it means
   in dollars — including the "not knowable from the client" branch
   (the server-side allowlist), reported honestly as unknowable.
2. Ship as the smallest honest artifact — a script or tiny CLI in
   this repo; language whatever is fastest (language choice is NOT
   what this probe tests).
3. Run against the operator's own machines (kinu first).

## Timebox

One session.

## Success signal

- Runs locally, read-only, zero network.
- Correctly reports the TTL decision for at least two config states
  (e.g. subscriber default vs an env override), matching the research
  doc's decision table.
- Produces at least one actionable finding on our own fleet.
- Failure is informative too: if the check can't be made mechanical,
  that bounds what usher's compliance templates can promise (feeds
  F4).

## Feeds

- charter F1 (component decomposition — does a spider/CLI emerge
  before usherd?), F3 (signal inventory grows its first concrete
  entry), F4 (template shape), F6 (instance model: a doctor that
  works bare, before any team instance exists, is the floor).
- The elicitation (`aae-orc-0mmn.1`) — primary research running in
  parallel with the brief, per the session-046-era pattern of code
  probes feeding design.

## Staleness risk

Inherited from the research doc: one value is server-controlled and
the decision logic was read from build 2.1.217. The check must
version-stamp its assumptions and degrade to "verify against your
build" rather than assert stale precedence.
