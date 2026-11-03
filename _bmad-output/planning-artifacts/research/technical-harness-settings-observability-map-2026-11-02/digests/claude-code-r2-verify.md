# Claude Code r2 verify digest (2026-11-02, CLI 2.1.317)

Method note: independence here means a different docs page or the CHANGELOG/GitHub, not the page the claim was first drawn from. All docs pages are on code.claude.com (same publisher, Anthropic); only the CHANGELOG and the GitHub issue are a different surface. Docs pages carry no publication date; pub_date is "undated (live page)". Budget: 12 tool calls used; 8 pages retrieved (over the 6-source budget by 2, because the permission-modes page truncated and the auto-mode page was needed for C2).

## Claims

C1 | verified | evidence:
- {claim: precedence list "1. Managed settings 2. Command line (--settings) 3. Project local (.claude/settings.local.json) 4. Shared project (.claude/settings.json) 5. User (~/.claude/settings.json)", source https://code.claude.com/docs/en/settings, publisher Anthropic, pub_date undated (live page), accessed 2026-11-02, confidence high}
- {claim: "When you set the same list key, such as permissions.allow, in more than one file, Claude Code combines the lists instead of picking one." Exceptions: fallbackModel, modelPicker, availableModels (managed list applied as-is), modelSettings. So "arrays combine" holds with four named exceptions, source https://code.claude.com/docs/en/settings, publisher Anthropic, pub_date undated, accessed 2026-11-02, confidence high}
- {claim: independent page: "Rules are evaluated in order: deny, then ask, then allow. The first match in that order determines the outcome, and rule specificity doesn't change the order." and "a user-level deny blocks a project-level allow, because deny rules from any scope are evaluated before allow rules." Also "no other level, including command line arguments, can override a managed permission rule.", source https://code.claude.com/docs/en/permissions, publisher Anthropic, pub_date undated, accessed 2026-11-02, confidence high}
- Qualifier: PreToolUse hooks cannot bypass deny/ask; a blocking hook (exit 2) precedes allow rules (same permissions page). Project permissions.allow waits for workspace trust; deny/ask apply immediately.

C2 | disputed (third clause) / verified (first two) | evidence:
- {claim: "The classifier doesn't read autoMode from project settings in .claude/settings.json or .claude/settings.local.json." Scopes read: ~/.claude/settings.json, managed settings, --settings / Agent SDK inline JSON, source https://code.claude.com/docs/en/auto-mode-config, publisher Anthropic, pub_date undated, accessed 2026-11-02, confidence high}
- {claim: "Entries from each scope are combined. A developer can extend environment, allow, soft_deny, and hard_deny with personal entries but can't remove entries that managed settings provide... the combination is additive, not a hard policy boundary.", source same page, confidence high}
- {claim (docs side): "Setting any of environment, allow, soft_deny, or hard_deny without "$defaults" replaces the entire default list for that section.", source same page, confidence high for current docs text}
- {claim (changelog side): v2.1.316: "Changed auto mode so that an autoMode.allow list without "$defaults" no longer removes the built-in rule" and "Fixed a managed autoMode.allow list without "$defaults" getting the built-in exceptions back", source https://raw.githubusercontent.com/anthropics/claude-code/main/CHANGELOG.md, publisher Anthropic (GitHub), pub_date v2.1.316 entry (date not shown in excerpt), accessed 2026-11-02, confidence medium (wording "the built-in rule", singular, may mean one specific rule rather than the whole list)}
- Net: for allow, behavior changed in 2.1.316 and the docs caution may lag; for environment, soft_deny, hard_deny the docs claim stands unchallenged. Treat "omitting $defaults discards built-ins" as true for soft_deny/hard_deny/environment, uncertain for allow.

C3 | verified | evidence:
- {claim: ranking "remote settings at the top through MDM, managed settings files, and the HKCU registry"; list items "2. MDM or OS-level policies: the macOS plist or the HKLM registry key 3. Managed settings files, managed-settings.d/*.json and managed-settings.json merged together 4. The HKCU registry", source https://code.claude.com/docs/en/managed-settings, publisher Anthropic, pub_date undated, accessed 2026-11-02, confidence high}
- {claim: plist domain com.anthropic.claudecode; HKLM\SOFTWARE\Policies\ClaudeCode value Settings; HKCU same path, source same page, confidence high}
- {claim: ""first-wins", the default: Claude Code uses the highest-ranked source that delivers at least one policy key and ignores the rest"; ""merge"... Requires Claude Code v2.1.242 or later.", source same page, confidence high}
- Qualifiers: some keys are read from every admin source even under first-wins (env per variable since v2.1.223; deniedMcpServers, disableClaudeAiConnectors, maxEffortLevel lowest cap, useAutoModeDuringPlan/syncClaudeAi* false, enableArtifact false). HKCU is not an "admin source" and is excluded from merge. managedSourcesBehavior is read only from the highest-ranked source carrying it or a policy key.
- Independence caveat: managedSourcesBehavior did not appear in the first 100k chars of the CHANGELOG (1.14M chars total; unread remainder). The settings page independently points to "precedence within the managed tier" on managed-settings but does not restate the order. So independence for C3 is weak (one docs page).

C4 | verified (absence, with qualifiers) | evidence:
- {claim: "The line confirms which files Claude Code read; it doesn't show which file supplied each key." Pointers instead: claude doctor lists rejected entries; startup header names the file that set a project/managed model, source https://code.claude.com/docs/en/settings (Confirm what loaded), publisher Anthropic, pub_date undated, accessed 2026-11-02, confidence high}
- {claim: "/permissions ... lists all permission rules and the settings.json file each rule comes from.", source https://code.claude.com/docs/en/permissions, confidence high}
- {claim: "claude auto-mode config: Print what the classifier actually uses as JSON, with your settings applied where set and defaults otherwise" (four lists: allow, soft_deny, hard_deny, environment); it does not annotate per-entry source, source https://code.claude.com/docs/en/auto-mode-config, confidence high}
- {claim: /status "Setting sources" line names the managed source in parentheses, e.g. (file), (plist), (HKLM), (remote + file, merged), (HKCU), plus a "Skipped sources" line since v2.1.242, source https://code.claude.com/docs/en/managed-settings, confidence high}
- Lead: the Agent SDK resolveSettings() helper exists (mentioned on the settings page) and may return merged settings; not checked whether it reports per-key source.

C5 | verified | evidence:
- {claim: "set projects["<path>"].hasTrustDialogAccepted to true in ~/.claude.json, where <path> is the repository root, or the folder itself outside a repository.", source https://code.claude.com/docs/en/permissions (Project allow rules and workspace trust), publisher Anthropic, pub_date undated, accessed 2026-11-02, confidence high}
- {claim: ~/.claude.json "holds ... per-project state such as trust decisions", source https://code.claude.com/docs/en/settings, confidence high}

C6 | verified (with nuance) | evidence:
- {claim: #70181 "[BUG] Empty server-managed settings (304 cached) zero out local managed-settings.json: deny/allow rules never enforced"; Closed, reason "Not planned", by github-actions bot on 2026-10-13 with "Closing for now - inactive for too long."; labels area:permissions, area:security, bug, platform:linux, stale; Development: "No branches or pull requests"; no maintainer response; bot had earlier flagged possible duplicates #53662, #56671, #44640, source https://github.com/anthropics/claude-code/issues/70181, publisher GitHub/Anthropic, pub_date closed 2026-10-13, accessed 2026-11-02, confidence high}
- Possibly related changelog: v2.1.316 "Changed server-managed settings for Team and Enterprise logins: settings fetched from Anthropic's API now apply"; v2.1.304 "Fixed OpenTelemetry settings delivered through server-managed settings being ignored for the whole session". Neither names an empty/304 response; no link to #70181 established. Confidence that either fixes #70181: low.

## Lead findings

L1 | verified | per-key scope column:
- {claim: the settings index has a Scope column with values including "User, local, or managed", "User or managed", "User", "Managed", and "Global config"; those keys never apply from the shared .claude/settings.json (a few can still be switched off from it); Global config keys apply only from ~/.claude.json, source https://code.claude.com/docs/en/settings, publisher Anthropic, pub_date undated, accessed 2026-11-02, confidence high}
- {claim: examples of scope-restricted keys: permissions.defaultMode values auto and bypassPermissions do not take effect from project or local settings (bypassPermissions did before v2.1.257); modelPicker ignored in project and local; telemetry export variables in env ignored from project/local except some off values; --settings "can't set Managed or Global config keys", source same page, confidence high}
- {claim: managed-only key list exists at managed-settings#managed-only-settings; delivery controls policyHelper, wslInheritsWindowsSettings, managedSourcesBehavior; allowManagedPermissionRulesOnly makes managed the only source of permission rules, source https://code.claude.com/docs/en/managed-settings and /permissions, confidence high}
- {claim: security exceptions to managed precedence (stricter lower value honored): disableClaudeAiConnectors, enableArtifact, isolatePeerMachines, permissions.blockReads/WritesOutsideWorkingDirectories, autoMode.classifyAllShell (user or --settings), remoteControlAtStartup, crossSessionInbound, useAutoModeDuringPlan, syncClaudeAiSkills, syncClaudeAiPlugins, maxEffortLevel, source https://code.claude.com/docs/en/settings, confidence high}
- Not retrieved: the settings-reference page itself, so the full per-key list was not enumerated.

L2 | verified | status line input fields exposing effective state:
- {claim: status line stdin JSON includes model.id, model.display_name, cwd, workspace.current_dir, workspace.project_dir, workspace.added_dirs, workspace.git_worktree, workspace.repo.{host,owner,name}, cost.total_cost_usd, context_window.context_window_size, fast_mode, effort.level (session level; shows the level actually run), thinking.enabled, version, output_style.name, vim.mode, agent.name. No field exposes settings sources or the merged settings object, source https://code.claude.com/docs/en/statusline, publisher Anthropic, pub_date undated, accessed 2026-11-02, confidence high (field list may be incomplete; only fields matching my filter were extracted)}
- {claim: subagentStatusLine input includes base hook fields plus per-task model, effort, contextWindowSize, agentType, cwd, source same page, confidence high}
- Permission mode field: not seen in the status line field list extracted; unverified whether one exists.
- Hook input payload fields (hooks page common input fields): not retrieved this run.

## Looked for, could not find

- managedSourcesBehavior, managed-settings.d, hasTrustDialogAccepted, or "auto-mode config" in the first 100k of 1.14M chars of CHANGELOG.md; the remainder was not read, so no changelog independence for C3/C5.
- A changelog entry or PR linked to #70181.
- A command that dumps merged settings with per-key source (consistent with C4; the settings page states /status does not show which file supplied each key).
- Publication dates for any code.claude.com docs page.
- A permission_mode field in status line input; hook common input fields (not fetched).
- Whether docs auto-mode caution about omitting "$defaults" in allow has been updated for v2.1.316 (current page text still says it replaces the list).
