---
role: bm-task
verb: bm-pr
args: (no args; current branch phase-v1-JM-e)
phase: v1-JM-e
created: 2026-05-02
---

# Brief — `bm-pr` for phase-v1-JM-e

## 1. Role + dispatch line

`[role:bm-task] bm-pr phase-v1-JM-e — see .claude/PRPs/briefs/jm-e-bm-pr-1.md`

You are the **bm-task** subagent. Execute the `bm-pr` verb on the current branch `phase-v1-JM-e`. This will open a PR from `phase-v1-JM-e` into `governance-v0` on `barrie-cork/lemmy`.

## 2. Scope

**Produce:**
- New PR `phase-v1-JM-e → governance-v0` on `barrie-cork/lemmy` (NOT draft per `.claude/rules/phase-branch.md`).
- Title per Phase 2 of `bm-pr.md`: `Phase v1-JM-e — Appeal-vote tally + integration capstone + step-up / spoofing / admin-visibility hardening` (taken from the H1 of `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md`).
- Body assembled per Phase 3 source priority: completion-report path is **NOT** used (no `*-complete-report.md` exists — JM-e uses retro + verify reports instead). Plan reference + retro reference + verify reference + commit log.
- Append a `## bm: PR opened` entry to `.claude/runlog/v1-JM-e-runlog.md` per Phase 6.
- 7-section "PR opened" summary per Phase 7.

**Do NOT:**
- Open as draft (CR skips drafts).
- Open into `main` (hard refusal — trunk for v1 work is `governance-v0`).
- Touch `crates/**`, `migrations/**`, `tests/**` (file-ownership boundary).
- Send Telegram pings without asking — but you're in `-p` mode, so **don't ping at all** this run; the advisor will queue `bm-ping` separately if the user wants it.
- Author plan/PRD/retro content — those are advisor-owned.
- Edit any commit messages on `phase-v1-JM-e` — they are sealed.

**Commit only:**
- `.claude/runlog/v1-JM-e-runlog.md` on `phase-v1-JM-e` (Phase 6 append).

## 3. Required reading

1. `.claude/commands/bm/bm-pr.md` — operational script (Phase 1–7). Follow step-by-step.
2. `.claude/rules/branch-manager.md` — operating rules, hard refusals, autonomy bounds.
3. `.claude/rules/phase-branch.md` — phase-branch flow + PR-not-draft requirement.
4. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory.
5. `.claude/agents/bm-task.md` — your contract (no-AskUserQuestion, DQ-blocked stop pattern).
6. **Source files for body assembly:**
   - `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — H1 → PR title; §1 Summary → first-paragraph body source.
   - `.claude/PRPs/reports/v1-JM-e-retro.md` — retro path (cite under `## Retro reference`).
   - `.claude/PRPs/reports/v1-JM-e-verify.md` — verify path (cite under `## Verify reference`).
   - `git log governance-v0..phase-v1-JM-e --oneline` — commit-list source for `## Commits` section.
7. **Lesson scan:**
   - `feedback_pr_per_phase.md`
   - `feedback_branch_manager_pm_split.md`
   - `feedback_gh_pr_fork_repo_flag.md`

## 4. Constraints

- **Pre-conditions all green at brief-write time:** branch is `phase-v1-JM-e`; HEAD `d4b6dfa3b` matches `origin/phase-v1-JM-e`; working tree clean; 25+ commits ahead of `governance-v0`; no existing PR for this branch (verified via `gh pr list --head phase-v1-JM-e`). Re-verify in Phase 1; if any drift, file a `pending` DQ entry from `from: "bm"`, kind: `"blocker"`, with the specific drift, commit + push to `phase-v1-JM-e`, return `blocked-on-DQ-#<id>`.
- **Body assembly substitutions** (Phase 3 source priority — NO completion report):
  - `## Summary` source = first paragraph of plan §1 Summary (lines 34–36 of plan file). Use prose verbatim, single paragraph.
  - `## Plan reference` = `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md`
  - `## Completion report` = REPLACE with `## Retro reference` line citing `.claude/PRPs/reports/v1-JM-e-retro.md` (confidence 0.85). Append `## Verify reference` line citing `.claude/PRPs/reports/v1-JM-e-verify.md` (all 4 §16a stories ✓).
  - `## Commits` = full `git log governance-v0..phase-v1-JM-e --oneline` output, one bullet per line.
  - `## Closes` = empty for this PR (JM-e closes no GH issues directly; the JM-PRD capstone reference is in plan §1 prose, not in commit `closes #N` markers). Write `_(none — JM-e is the JM-PRD capstone; see plan §1 for cross-PRD context)_`.
- **Title is fixed** (per Phase 2 branch-name → title pattern):
  ```
  Phase v1-JM-e — Appeal-vote tally + integration capstone + step-up / spoofing / admin-visibility hardening
  ```
- **No-AskUserQuestion:** any "Yes asks first" branch in the script becomes a DQ-stop. Write a `pending` DQ from `bm`, commit + push it, return `blocked-on-DQ-#<id>`.
- **Linux discipline:** EliteDesk worker. Use `bash`/`git`/`gh`. No `.bat` wrappers.
- **--repo barrie-cork/lemmy:** mandatory on every `gh pr` invocation per `gh-pr-fork-target.md`.
- **Pipes mask exit codes** (`feedback_pipes_mask_exit_codes.md`): never pipe `gh` through `tail`/`grep` when needing exit-status. Capture to file or check `$?` directly.
- **Commit message format:** runlog commit uses `chore(bm): record PR-open for phase-v1-JM-e`. Lands on `phase-v1-JM-e`, never on `governance-v0`.
- **No Telegram this run:** the `Phase 7 / Telegram ping` line in the script suggests `bm-ping pr-ready`. Do NOT invoke it. Advisor queues `bm-ping` separately if the user wants the channel notified.

## 5. Expected output

A 7-section summary per the bm-pr script Phase 7:

```
## /bm-pr complete

**PR #<N>:** Phase v1-JM-e — Appeal-vote tally + integration capstone + step-up / spoofing / admin-visibility hardening
**URL:** https://github.com/barrie-cork/lemmy/pull/<N>
**Base ← Head:** governance-v0 ← phase-v1-JM-e
**Body source:** plan + retro + verify + commits
**Draft?** No (CR-eligible)

### What happens next (automatic)

1. CodeRabbit will review within ~5–10 min.
2. Run `/bm-poll-cr <N>` to ingest findings.
3. Run `/bm-prp-review <N>` to add Brehon ADR + cargo review.
4. Triage with `/bm-triage <N>` → confirm before posting digest.
5. Merge with `/bm-merge <N>` → confirm before merging.

### Telegram ping

NOT sent this run — advisor will queue `/bm-ping pr-ready` separately if the user wants the channel notified.
```

The advisor's polling loop reads this output and queues `bm-poll-cr` once CR posts findings (typically 5–10 min after PR open).
