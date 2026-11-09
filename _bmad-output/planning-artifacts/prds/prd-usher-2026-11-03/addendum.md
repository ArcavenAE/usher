---
title: "PRD Addendum: usher"
status: final
created: 2026-11-03
updated: 2026-11-05
---

# Addendum: usher PRD, 2026-11-03

Mechanism detail for the architecture step. Sources: the harness settings
research (2026-11-02) and the orc index, `docs/research/orc-index.md`.

| Section | Holds | Cited by |
|---|---|---|
| §1 | How each harness is read, and the hazards for the observer | FR-1, FR-7, FR-34, PRD §5 |
| §1.1 | Test cases and claims to test, per harness | FR-1, FR-2, FR-6, FR-16, FR-24, FR-35, Q4 |
| §2 | Bootstrap sketch | PRD §4.5 |
| §3 | Install failure modes the health check must catch | FR-9, FR-10 |
| §4 | Source conflicts resolved in the PRD (a log) | none |
| §5 | Carried from the brief, for architecture | PRD §5, FR-37 to FR-40 |
| §6 | Boundaries by objective | PRD §5, Q16 |
| §7 | Change-safety failure cases | FR-49, FR-63, FR-64, FR-67, PRD §5 |

## 1. Observation method per harness

| Harness | Resolved view | Rebuilt from files | Hazards for the observer |
|---|---|---|---|
| Claude Code (2.1.317) | None with per-key provenance. Partial views: `/permissions` (rule plus source file); `claude auto-mode config` (effective lists, no source); `/status` (managed source, skipped sources since 2.1.242); `claude doctor` (rejected entries, remote managed state since 2.1.248); statusline input | Managed, `--settings`, `.claude/settings.local.json`, `.claude/settings.json`, `~/.claude/settings.json`; per-key scope table; `~/.claude/remote-settings.json` cache (may differ from the values in effect); trust in `~/.claude.json` | `claude auto-mode` can reset settings; `/doctor` can apply fixes; `claude doctor` reads cwd settings without a trust prompt; `purge`, `install`, `respawn`, plugin `update` mutate; files hot-reload |
| codex (0.160.1) | App-server `config/read` with `cwd` and `includeLayers`: config, layers with `disabledReason`, `origins` per key; `configRequirements/read` | AGENTS.md chain and skills dirs | Whether `config/read` starts the app-server daemon or update loop is unknown; `codex update` mutates |
| goose (1.57.0, not installed here) | `goose info -v` (provenance unknown) | `/etc/goose/config.yaml`, `GOOSE_ADDITIONAL_CONFIG_FILES`, user file; env by key beats files (read from source code, not docs) | No local probe yet; telemetry kill switch is `GOOSE_TELEMETRY_OFF` |
| Claude desktop app and Claude general settings | Unknown; needs the FR-7 probe | Unknown; likely an app config file for MCP servers plus app preferences, account settings server-side `[unverified]` | Unknown until probed |

Codex reports its own resolved view with a layer per key. For Claude Code
and goose, usher rebuilds the stack from files and checks it against the
harness's partial views. Release sources for FR-34: the changelog, generated
schema (codex `generate-json-schema`), and each harness's own help and
config output under a scratch home.

### 1.1 Test cases per harness

Each row is a testable case the PRD's FRs point at. Rows are keyed to the
harness version they were written for and are re-checked per FR-32.

| FR | Harness | Case |
|---|---|---|
| FR-1 | Claude Code | Permission lists and `autoMode` merge across layers; provenance per entry |
| FR-1 | goose | `extensions` and `providers` merge across layers; provenance per entry |
| FR-1 | codex | Values and layers match `config/read` with `cwd` and `includeLayers` for that directory |
| FR-2 | codex | A project file setting `notify` is declared and ignored |
| FR-2 | goose | A key set in the user file and by environment variable: the environment value is in effect |
| FR-2 | Claude Code | A managed source skipped under first-source-wins |
| FR-2 | Claude Code | An entry `claude doctor` rejects |
| FR-2 | Claude Code | An environment variable overriding its settings key |
| FR-2 | Claude Code | Any of these placed in a project or local file: `defaultMode` set to `auto` or `bypassPermissions`, `autoMode`, OTEL settings |
| FR-2 | Claude Code | A server-managed value held back until the server confirms |
| FR-6 | Claude Code | Trust state matches `projects[<repo root>].hasTrustDialogAccepted` in `~/.claude.json` |
| FR-16 | codex | `approval_policy = "untrusted"` is flagged from the first release carrying its removal (PR 39630, merged 2026-08-20); the harness now fails with a hard error on it |
| FR-16 | Claude Code | An organization soft deny that an auto-mode allow entry overrides is flagged; only managed-only keys are locks |
| FR-24 | Claude Code | An `autoMode` entry is never placed in a project file, since that layer ignores it |

**Claims to test before the PRD relies on them** (one probe each, FR-35;
open question 4):

| Harness | Claim | Status |
|---|---|---|
| Claude Code 2.1.317 | Omitting `$defaults` drops the built-in auto-mode rules; the FR-16 `$defaults` hazard check waits on this | Confirmed on 2.1.319, 2026-11-08: 16 built-in allow rules drop to the one set; deny lists unchanged |
| codex | Where device-management config ranks in precedence | Disputed; untested, no managed codex layer on the probe host |
| codex | Whether project skills load in untrusted repos | Confirmed on 0.160.1, 2026-11-08: `.agents/skills` and `.codex/skills` load; the project config layer is disabled |
| goose | The `GOOSE_MODE` default | Disputed |
| codex | A startup warning for ignored project keys | Confirmed on 0.160.1, 2026-11-08: app-server logs the ignored key (`notify`) at error level |

## 2. Bootstrap sketch

Capture, filter, annotate, consume. Repo and project settings are already a
sharing channel, which is why UJ-2's "passive" repo changes moved team
numbers. Usher makes that channel visible rather than replacing it.

- **Self-serve sharing** (v1). The expert captures an environment into a
  draft profile, removes what is personal, adds a note per entry, and shares
  the file by any channel (repo, message). The newcomer loads it; usher
  compares, places and applies the entries on confirmation. Reports are
  local files, shared by hand or sent to the light collector.
- **Light collector** (v1, operator 2026-11-03). Self-hosted, opt-in per
  host, receives the reported fields and answers cross-host queries.
  Adding it pulls a minimal consent flow (F7), host credentials (ADR-009
  test) and a
  storage choice (F4) into the first release.
- **Central distribution** (v2). The collector grows to distribute files in
  the same format, run templates and recommend from them.

## 3. Install failure modes the health check must catch

From the orc index and findings:

| Failure mode | Source |
|---|---|
| A self-update under a running seat | finding-182 |
| An installed copy behind its merged source | aae-orc-gntz |
| An installed version file that reads current when it is not | aae-orc-9va5 |
| A doubly tapped formula that errors instead of answering | finding-221 |
| Stale command files after a pack version change; undercounted status; unpruned old versions; layout migrations shipped as point releases | findings 025, 027 |
| Built skills that never resolve | finding-076 |
| Plugins with a missing source or no recorded enablement | finding-091 |
| A read-only install updated in name only | wardrobe README |
| A missing client env file | orc index |
| Unsigned packs | orc index |

## 4. Source conflicts resolved in the PRD

A log of how the draft reconciled its sources, kept so the architect can see
where they disagreed.

- Render `--apply` in the settings-profile idea
  (`_kos/ideas/fleet-settings-profile.md`) is FR-49, apply with write
  safety.
- Entry statuses differ between brief and idea: unified as present,
  missing, different, extra, plus findings.
- Cache score in v1 against the four-to-six-week window: per-session
  score is a fact in v1; team cost claims stay diagnostic with a window.
- F10 lists Crush, OpenCode and Gemini but not goose: the supported set is
  Claude Code, codex, goose; others enter through the observability map.
- Per-host credential provisioning against the first release's scope:
  usher runs the vendor login and never copies or holds a credential.
- Windows: charter and brief differ; open question 14.
- Permissions friction (the brief's lead problem): operator ruled permissions
  are settings like any other; covered by FR-1, FR-2, FR-16 (locks weaker
  than they look), plus portable entries (FR-20) and order-blind comparison
  (FR-15).

## 5. Carried from the brief, for architecture

- **Prior-art rulings** (brief addendum §E): accept a typed protocol with
  local cache and flush-on-reconnect, system-generated push channels, and
  "it advises; the rider decides"; adapt keypair auth (prefer the
  organization's directory or device management) and zero-interruption
  visibility; reject session tiles as primary UI and workflow-stage
  telemetry as-is. Flush-on-reconnect and keypair auth bear on FR-37 to
  FR-40.
- **Seam obligations:** compliance templates are the fidelity channel for
  the central service and must not slip; every in-LLM surface carries a
  token budget.
- **Credential scan** (brief posture): may report location and type of
  credential-shaped material, never the value; off by default; reconciles
  plaintext-key detection (aae-orc-z6y7 W2) with ADR-009.
- **Instance model** (brief addendum §B): claim markers, private bindings,
  scope resolution and attribution by scope stay the design direction for
  several affiliations per person; the first release has one affiliation
  per light collector.

## 6. Boundaries by objective

Usher's objective, ruled 2026-11-04: "setup & maintain settings".
Configuration writes on the person's confirmation are within it.

The boundary rule (operator 2026-11-03): "The key to understanding the
differences and relationship is objectives, not medium. usher helps you
install & run it, critic measures & competes/judges it, and stagekeeper
detects & recovers from faults." The later ruling above refines usher's
part. All three components read files, watch processes and run control
loops, so a boundary drawn on what a component touches is wrong.

| Requirement | Shared medium | Why it is usher's | Where it would stop being usher's |
|---|---|---|---|
| FR-10 process table | live processes | shows which sessions run an old version, so the person can re-find their seat | deciding a session is wedged and restarting it (stagekeeper) |
| FR-13 watcher | file changes | tells the person their settings moved | deriving fault conditions from session activity (stagekeeper) |
| FR-26 observed effects | session records | a fact about the person's own seat, never combined across backends | scoring or ranking configurations against each other (critic) |
| FR-15 comparison | settings sets | shows differences so a person can adopt a setting | declaring a winner (critic); usher leaves the judgment to people |

## 7. Change-safety failure cases

One test per row, each injecting the failure against a scratch home and
checking that the change stops or recovers as FR-63 and FR-64 require.

| Failure injected | Expected result |
|---|---|
| A target path built from an empty variable (`"$X/settings.json"` with `X` unset) | Refused before any write; nothing journaled as done |
| A target holding an unexpanded `$VAR`, `${VAR}`, `%VAR%` or `~` | Refused |
| A target that resolves to `/`, the home directory or a parent of it | Refused |
| A glob character in a value the change did not ask to expand | Refused |
| A path with `..`, or a symlink, that leaves the change's allowed paths | Refused |
| A value with spaces, quotes or `;` passed to an installer | Passed as one argument; no command runs from it |
| A change that would touch more targets than its dry run listed | Stops at the first extra target |
| The process killed between writing the temporary file and the rename | The target is the old version; the journal shows the step unfinished |
| The disk full, or too little space for the backup | Stops before writing |
| A target edited by hand between the dry run and the write | Stops and shows the conflict |
| A read-only or locked target | Stops; nothing partial left behind |
| New content that does not parse in the file's format | Not renamed into place |
| A JSONC or TOML file with comments outside the changed key | Comments and formatting kept |
| A rollback of a target changed since usher wrote it | Shown as a conflict, left alone |
| Two change runs started at once on one host | The second waits or refuses |
| A credential present in a target | The journal records presence only |
| A write aimed at a file harness knowledge marks as holding credentials | Refused; no backup made |
| A symlinked settings file whose target is on the allowed list | Written through the link; the link kept |
| The journal unwritable or unreadable | No change runs |
| A delegated installer exits nonzero part way | Exit status and changed paths journaled and reported |
| An installer piped into a shell from an unpinned source | Refused |
| Undo of a clone that has new commits or untracked files | Shown as a conflict; nothing moved |
| Undo of an update whose installer cannot pin the earlier version | The named manual step shown; nothing run |
| Undo of an entry a later entry depends on | The later entry listed; undone first or the rollback stops |
| A lock left by an usher process that is gone | Released and journaled |
| An in-LLM agent confirming a change | Applied as from any surface; the journal records the surface that confirmed |
| A write to one writable key in a file that also holds account data and trust records | Only that key changes; every other key is byte for byte the same |
| A write that would change a folder-trust record | Refused |
| A write to an entry the person marked as personal | Refused; shown as personal |
| Removal of usher, keeping its changes | No usher process, login item, scheduled job or endpoint remains; journal and backups kept |
| Removal of usher with rollback chosen | Changes rolled back in reverse order first; conflicts shown, as in any rollback |
| Observation before the person confirms any directory | Only harness-knowledge locations read; discovered directories listed as proposals, none mapped |
