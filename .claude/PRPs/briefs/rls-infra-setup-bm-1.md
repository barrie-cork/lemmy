[role:bm-task] rls-infra-setup PR — branch chore/rls-infra-setup → governance-v0

## Scope

The advisor session has prepared the brehon-fork RLS infrastructure on the EliteDesk Junior daemon working directory. The chore branch chore/rls-infra-setup contains:

- 7 shared skills (code-audit, code-refactor, daemon-resume, evaluate-run, post-task-retro, resource-cleanup, weekly-review) deployed via scripts/sync-shared-skills.sh from the homeserver repo's skills/shared/
- 10 shared rules (circuit-breaker, cross-repo-coordination, escalation, evaluation-calibration, integrator, memory-injection, no-destructive-defaults, pmd-search-strategy, post-task-retro, session-awareness)
- 5 shared hooks (edit-readback-reminder, observation-capture, retro-check, validate-memory-search, worktree-guard) registered in .claude/settings.json
- .claude/.sync-manifest checksum file
- .gitignore entry for .mcp.json (contains Ref API key + host-specific paths)
- .mcp.json.example template (for fresh setup of this repo on a new machine)

The branch has 1 commit prepared by the advisor (the staged 26 files). BM verifies, opens PR.

## Required reading

- .claude/rules/branch-manager.md — your operating contract
- .claude/commands/bm/bm-pr.md — the verb script you're executing
- .claude/rules/phase-branch.md — PR target rule (always governance-v0, never main)
- .claude/rules/gh-pr-fork-target.md — gh --repo flag is mandatory on this fork

## Constraints

- Open PR via: gh pr create --repo barrie-cork/lemmy --base governance-v0 --head chore/rls-infra-setup
- PR title under 70 chars: 'chore(rls): bootstrap shared skills/rules/hooks + MCP config'
- PR body: 3-bullet Summary section + Test plan section per gh-pr-fork-target.md
- Do not merge — CodeRabbit auto-reviews PRs into governance-v0
- Do not write LESSON: trailers on this PR's commits (per feedback_junior_pmd_write_convention.md, the bar is 'future me would have wanted to know this' — bootstrapping is one-time)
- Append a runlog entry at .claude/runlog/rls-infra-runlog.md per bm-pr Phase 4
