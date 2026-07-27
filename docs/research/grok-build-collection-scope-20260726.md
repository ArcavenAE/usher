# Grok Build collection-scope failure: what it costs to get consent wrong

- **Date:** 2026-07-26
- **Status:** research note, gathered in one pass; the primary wire analysis
  is secondhand here and is worth reading directly before the note is
  treated as settled.
- **Why this exists:** charter B1 (reporting off by default, enabling it is
  the user's act) rested on a 2010s-era evidence base: Kolide's honest
  security as the pattern that survived scrutiny, Homebrew's silent opt-out
  analytics as the one that burned trust (see
  [prior-art-landscape-20260712.md](prior-art-landscape-20260712.md),
  finding-001). This is the same failure in usher's own product category,
  in the same month usher was being designed, at roughly a hundred times
  the scale. It converts several posture statements from aspiration into
  testable requirements.

## What happened

A wire-level analysis of xAI's Grok Build CLI, published 2026-07-14, found
the tool transmitted file contents including unredacted `.env` secrets to
xAI, and uploaded whole repositories including full git history to a cloud
storage bucket, regardless of which files the agent actually read.

The methodology is the part worth copying. The researcher instructed the
CLI to reply only with "OK" and explicitly ordered it not to open any
files. It uploaded the repository anyway. A canary credential planted in a
`.env` file appeared verbatim in captured traffic. Git history carried
secrets that had been deleted months earlier. The result reproduced on a
second repository.

The scale mismatch is the finding's sharpest number: on a 12 GB test
repository, the model-turn channel carried roughly 192 KB of task-relevant
traffic while the storage-upload channel moved about 5.1 GB across 73
chunks, every request returning 200. Collection had no relationship to the
task.

Two aggravating facts. The behavior persisted with the "improve the model"
setting disabled, so the user-facing control did not gate the code path.
And the marketing claim was explicitly "local-first," with nothing from the
codebase transmitted.

Response: the analysis reached the Hacker News front page on 2026-07-14 and
drew a public promise to delete previously uploaded data. No formal security
advisory was issued. The CLI was open-sourced on 2026-07-15, alongside a
claim that Zero Data Retention had always been respected. Setting
`disable_codebase_upload = true` was confirmed to stop the repository
transmission.

Analysts framed the core failure as scope exceeding what documentation led
users to expect, rather than collection as such. That framing matters for
usher, because usher intends to collect something.

## Six rules this imposes on usher

1. **Communicate derived state, not content.** The 192 KB versus 5.1 GB
   ratio is the whole lesson. Reporting "cache TTL is 5m, model is X, three
   hooks configured, two MCP servers registered" is a categorically
   different act from shipping the files that say so. State is small,
   inspectable, and bounded; content is none of those. This also happens to
   be the cheapest answer to the spend-attribution-without-a-peephole
   problem, and it lowers the endpoint-security profile discussed under
   rule 4.

2. **Secret handling must be structural, not presentational.** Masking at
   display time does not help if secrets were ingested. Grok Build's git
   history leaked credentials deleted months earlier, which no output filter
   would have caught. The requirement is to never read the secret in the
   first place: record *that* auth is configured and by what method, never
   the material. Corollary from the same reporting: agentic shell access
   bypasses `.gitignore` and equivalents entirely, so ignore-file protection
   is not protection in any agent context.

3. **An off-by-default switch is only real if it gates the code path, and
   the gate must be testable.** This is the specific upgrade to B1. B1
   currently states a posture; it should state a verifiable property, with a
   test that asserts no egress occurs in the default state, and a second
   test that asserts the enabled state sends only what the published
   inventory names. Homebrew's failure was a default nobody noticed; this
   failure was a control that did not work. B1 must be immune to both.

4. **Invert the verifiability lesson: be auditable on purpose.** Grok Build
   was caught because it did not pin certificates against a local proxy, so
   any developer could reproduce the finding, and that reproducibility is
   why it spread. For usher the same property is an asset rather than an
   exposure. If usher is trivially wire-auditable and publishes its
   behavioral profile, then honest security becomes provable instead of
   promised, and the same document lets an endpoint-security team allowlist
   usher deliberately rather than discovering it as an anomaly. "Check us"
   is a stronger claim than "trust us," and it is cheaper to maintain than
   a reputation.

5. **The "we do NOT collect" list is load-bearing and must be testable.**
   The brief already names it as part of the consent stack. This case
   promotes it from a trust signal to a contract: analysts judged the
   product against its documentation, not against a privacy ideal. A
   published negative inventory that CI verifies is the artifact that makes
   the judgment survivable.

6. **Consent events belong in the record.** Grok Build's defenders and
   critics could not agree on what any given account had actually been
   configured to do. Usher should be able to answer that question about
   itself: what disclosure was shown, what was accepted, when, and whether
   an enable was user-chosen or org-required. This is the forensic artifact
   that makes B1 defensible under hostile examination, and it is cheap
   because it is a handful of rows, not a telemetry stream.

## What this does not establish

The wire analysis showed undisclosed transmission and storage. It did not
show that the vendor trained on the code, that employees read it, or that
every account received the same configuration. Usher should not cite this
case as proof of malice, and the note should not be used to argue that
collection is inherently wrong. The claim it supports is narrower and more
useful: in this product category, in 2026, the distance between a stated
privacy posture and a shipped code path was large enough to be found by one
researcher with a proxy, and the market reaction was immediate. Design for
being checked.

## Sources

- The Register, 2026-07-14: https://www.theregister.com/ai-and-ml/2026/07/14/musk_promises_purge_after_grok_build_caught_sending_entire_repos_to_the_cloud/5271123
- Developers Digest, wire-level analysis writeup: https://www.developersdigest.tech/blog/grok-cli-wire-level-analysis
- The Agent Report, 2026-07: https://the-agent-report.com/2026/07/grok-build-cli-repo-upload-privacy-july-2026/
- Tech Times, 2026-07-14: https://www.techtimes.com/articles/320420/20260714/grok-build-shipped-entire-codebases-xai-cloud-privacy-toggle-did-nothing.htm
