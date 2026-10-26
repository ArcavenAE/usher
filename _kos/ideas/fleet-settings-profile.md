# A declared fleet settings profile: usher renders, checks and reports harness settings

- **Status:** idea, written as a research note. Nothing here is ruled.
- **Subject:** usher. By the subject test this lives in usher's graph: without usher there is no settings distributor to discuss. marvel, the role library and the harnesses are objects it renders into.
- **Asked by:** the operator, via director (bus 01M5WK36JG), 2026-10-26: "kos idea usher we have a LOT of friction over permissions, automode what if usher was able to better distribute and manage these settings more effectively? What would that look like and how does that fit? send this to research further".
- **Extends:** F2 (onboarding includes "correct initial settings"), F7 (`question-org-admissions-and-locked-settings`, what an organization may require) and F10 (`question-settings-discovery-across-directories`, the read side). The orc's usher idea leaves one tension open: "configuration *reporting* (usher) vs configuration *management* (marvel control plane). Usher observes and recommends; who enforces?" This note proposes an answer for one class of settings.

## Short answer

Yes, as a narrow tool. Usher can own a declared, versioned **fleet profile**: trusted orgs and domains, auto mode allow entries, codex approval and sandbox choices, and folder-trust intent. It renders the profile into each host's layers on the operator's command, and it reports drift between the profile and what each harness actually resolves. It proposes and checks. It never adds an entry the operator did not write, and it never answers a prompt.

One fact from the docs changes the shape of the problem. Auto mode reads `autoMode` from user settings, managed settings, **and the `--settings` flag**. marvel already gives every Claude seat a per-session `--settings` file. So the per-host edit in aae-orc#705 is not the only route for seats: a profile rendered into marvel's policy projection would reach them with no hand edit on any host. Interactive human sessions still need the user or managed layer.

## The friction, by layer

All observed 2026-10-26 unless marked.

| friction | lives in | why it is friction |
|---|---|---|
| auto mode trusted orgs and allow entries must be edited per host (aae-orc#705) | Claude Code user settings, `~/.claude/settings.json` | the classifier does not read project settings, so the repo cannot carry them |
| [External System Writes] on reviews in our own orgs | auto mode `environment` (missing trust slot) | by default the classifier "trusts only the working directory and the current repo's configured remotes" |
| [Create Public Surface] on a PR to a private org, cleared once the org was declared trusted | the same | the same |
| [Self-Modification] refused once, passed later on the operator's explicit request | the classifier's soft-deny tier | explicit user intent clears a soft deny; a relayed grant does not (finding-192) |
| [Auto-Mode Bypass] blocks one session answering another's prompt | a built-in soft-deny rule | by design; this is a control, not friction to remove |
| codex seats stop at approval prompts (`gh auth status`) | codex `approval_policy`, set per seat through `runtime.Args` | marvel deliberately injects no approval policy |
| folder trust holds new seats at a dialog | `~/.claude.json` `projects[<path>].hasTrustDialogAccepted` | not a settings.json key; marvel's design to write it (seat-bootstrap.md) has no code on main yet |
| arcavenai is not a member of every org | GitHub org membership | identity, not settings; out of scope here |

## The layers that exist

**Claude Code**, highest precedence first: managed (remote or server-managed, then MDM or OS policy, then `managed-settings.json`, then HKCU on Windows), then the `--settings` flag, then project local `.claude/settings.local.json`, then shared project `.claude/settings.json`, then user `~/.claude/settings.json`. "Nothing in your own settings files or `--settings` overrides a managed key." Permission rule lists merge across layers, and "a user-level deny blocks a project-level allow." Managed-only locks include `allowManagedPermissionRulesOnly` and `disableAutoMode`.

**Auto mode**, quoted: "The classifier doesn't read `autoMode` from project settings in `.claude/settings.json` or `.claude/settings.local.json`. Both files live in the repo directory, so a checked-in repo or a build step could otherwise inject its own allow rules." And: "Entries from each scope are combined ... a developer-added `allow` entry can override an organization `soft_deny` entry: the combination is additive, not a hard policy boundary." Setting a list without `"$defaults"` discards the built-in rules for that section, including the auto-mode bypass rule.

**Folder trust:** shown in interactive sessions only; `-p` and SDK runs never show it. Pre-accepting is done by setting `projects["<path>"].hasTrustDialogAccepted` in `~/.claude.json`. No settings key does it.

**Codex**, highest first: CLI flags and `--config`, project `.codex/config.toml` (trusted projects only), a `--profile` file, user `~/.codex/config.toml`, cloud-managed defaults, `/etc/codex/config.toml`, built-ins. Admin enforcement is `requirements.toml` (`/etc/codex/requirements.toml`, or MDM domain `com.openai.codex`), for example `allowed_approval_policies` and `allowed_sandbox_modes`, "admin-enforced constraints that users can't override".

**marvel seeding** (marvel main 8419e8a): each Claude seat gets a per-session policy file passed as `--settings` (`internal/runtime/claude.go:113`), built from the role policy's `settings_file` plus inline keys (`internal/session/projection.go:205-223`), and `--setting-sources` defaulting to `user,project,local`. marvel "does not interpret settings files". Codex seats get a private `CODEX_HOME` with a fresh `config.toml` that sets per-path `trust_level`; "Sandbox and approval policy are the operator's to set through runtime.Args; the adapter injects neither" (`internal/runtime/codex.go:20-23`). Nothing on main declares a fleet-wide settings profile or compares declared settings with what a harness resolves; `docs/design/drift-view.md` compares a session's stored runtime with its role, not with harness files.

Sources: code.claude.com/docs/en/settings, /managed-settings, /permissions, /permission-modes, /auto-mode-config; learn.chatgpt.com/docs/config-file/config-basic, /enterprise/managed-configuration (developers.openai.com/codex redirects there). Fetched 2026-10-26.

## What it would look like

1. **The profile.** One versioned file per fleet instance (F6: the instance is the team's, not the repo's), operator-authored, in a repo the operator controls. It holds entries as prose with a label (the shape #705 already uses: a label before the first colon, so a rerun replaces rather than duplicates), each with its ruling and date. Sections: `claude.autoMode.environment`, `claude.autoMode.allow`, `claude.permissions.ask/deny`, `codex.approval_policy` and `sandbox_mode` per seat class, `trust` intent per path class.
2. **Render.** `usher render --check` prints what would change in each target layer; `usher render --apply` writes it, with a backup and a temp-file rename. Targets, in order of preference:
   - seats: a fragment marvel's policy projection merges (marvel owns the write; see the boundary);
   - interactive sessions: the user layer, by label-keyed merge;
   - an organization that wants it locked: a `managed-settings.d/` drop-in, written by that organization's own MDM, never by usher.
3. **Drift.** `usher drift` compares the profile with what each harness reports it resolved: `claude auto-mode config` for Claude, codex's effective config for codex. It reads the effective view, not the files, because the harness's precedence is the truth (F10 sub-question c). Output is derived state per host: missing, extra, stale label, and `$defaults` absent.
4. **Recommend.** From the denials a host recorded (`/permissions` Recently denied, or a `PermissionDenied` hook), usher drafts candidate entries and marks each as a proposal for the operator. It never applies one.

## How it fits usher

Front of house means getting a person, or a seat, to a working seat without a fight with the harness. Settings are the largest part of "correct initial settings" (F2), and drift reporting is F10's sweep turned toward a declared baseline. The profile is the team customization F6 asks about, applied to one class of settings. F7's disclosure question applies directly: a member must see which entries an organization's profile adds before usher renders them.

Compared with the alternatives:

- **Managed settings** already distribute and lock, and a remote or MDM tier reaches every host. They are the right tool for an organization that wants a lock. They are not a fit for this fleet today: there is no MDM on these hosts (no `managed-settings.json` on corporate), and a lock is a stronger act than the operator has asked for. Usher should be able to emit a drop-in for an organization to deploy, not deploy one.
- **marvel seeding** already reaches every seat with no host edit, through `--settings`. It is the right channel for seats. What it lacks is the declared source and the drift check: today the entries would be hand-written into each role's policy file. Usher supplies the profile and the check; marvel keeps the write.
- **A tracked script per change** (#705) works and is the measured baseline. Its cost is a new script, review and three host runs per change, and no report of which hosts ran it.

## What it must not become

- **A way to loosen a control the operator did not choose.** Every rendered entry traces to an operator-authored line with a ruling. Usher adds nothing from a recommendation without the operator's edit to the profile. Because allow entries combine additively and can override an organization's soft deny, an allow entry is a loosening, and the profile records it as one.
- **A remover of built-in rules.** A structural check refuses any rendered list that omits `"$defaults"`, since that discards the built-in soft-deny list, including the auto-mode bypass rule. This is a structural gate, which SOUL section 8 allows; health numbers stay diagnostic.
- **A prompt answerer.** Folder trust and consent prompts stay where their owner answers them. Usher may report a seat held at a prompt. It does not write `hasTrustDialogAccepted` for another session; whether marvel does for its own seats is marvel's design (seat-bootstrap.md) and the operator's ruling.
- **A judge.** Usher computes, checks and proposes (SOUL section 8, ADR-007). Whether an org is trusted is the operator's call.
- **A credential holder.** The profile holds no tokens, keys or logins (SOUL section 3). Identity gaps such as org membership are reported, not fixed.
- **A conscript.** Usher must work with no marvel present (human sessions, user layer only), and marvel must work with no usher present (policy files written by hand, as today). The seam between them is a file format, not a call.
- **A bypass channel.** A denial that fires after a render is reported to the operator, never answered by rendering a wider entry (no-control-bypass.md).
- **Silent.** Rendering is off by default and runs on the operator's command, per host (B1's shape applied to writes).

## Smallest first slice

`usher drift --check` for Claude auto mode only, read-only, on one host. It reads a profile file (the five entries in #705 are the first profile), runs `claude auto-mode config`, and prints per label: present, missing, stale, plus whether `$defaults` is present in each list. No writes, no network, no marvel dependency.

Cost: one small CLI subcommand and a profile format, about a day of builder time plus review, and a run on each of the three hosts to see what has drifted since #705. It answers the open question #705 leaves, which hosts actually carry the entries, before anything writes. Render to the user layer is slice two, and the marvel policy fragment is slice three, which needs marvel's agreement on the seam.

## Open questions

1. Does marvel want a profile fragment its projection merges, or should usher write role policy files that marvel already reads? The first keeps one writer per file (finding-180: two `--settings` on one launch are last-wins, wholesale).
2. Does an entry delivered through `--settings` show the same `claude auto-mode config` result as one in the user file, on the installed version? Measure before slice three.
3. The codex docs contradict each other on `untrusted` as an `approval_policy` value. Measure on the installed codex before a profile names it.
4. Should the profile live in usher's repo, in the orc, or in a per-fleet instance repo (F6)?
