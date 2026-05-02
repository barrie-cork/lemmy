---
role: bm-task
verb: bm-merge
args: 107
phase: v1-JM-e
created: 2026-05-02
---

# Brief — `bm-merge 107`

## 1. Role + dispatch line

`[role:bm-task] bm-merge 107 — see .claude/PRPs/briefs/jm-e-bm-merge-1.md`

You are the **bm-task** subagent. Execute the `bm-merge` verb on PR #107 (`phase-v1-JM-e → governance-v0`). User has explicitly confirmed merge with **merge commit** (preserves 60+ task-by-task commits) per `.claude/rules/phase-branch.md` convention.

## 2. Scope

**Produce:**
- Merge PR #107 via `gh pr merge 107 --repo barrie-cork/lemmy --merge` (NOT `--squash`, NOT `--rebase`).
- Append `## bm: PR merged` entry to `.claude/runlog/v1-JM-e-runlog.md` per `bm-merge.md` Phase 6.
- Capture the merge commit SHA on `governance-v0` for the runlog.
- 5-section "PR merged" summary per `bm-merge.md` Phase 7.

**Do NOT:**
- Use `--squash` (loses task-by-task history; against phase-branch.md).
- Use `--rebase` (rewrites SHAs; breaks DQ entries that reference commits by SHA).
- Use `--admin` flag unless mergeStateStatus blocks the merge — solo-dev account is the admin, but use the explicit `--admin` only if `gh pr merge` refuses without it.
- Touch `crates/**`, `migrations/**`, `tests/**`, plan files, retro files (file-ownership boundary).
- Send Telegram pings without asking — but you're in `-p` mode, so **don't ping at all** this run.
- Open new PRs, edit any commit messages on `phase-v1-JM-e`, force-push.

**Commit only:**
- `.claude/runlog/v1-JM-e-runlog.md` on `governance-v0` (after the merge lands; lands AFTER the merge commit).

## 3. Required reading

1. `.claude/commands/bm/bm-merge.md` — operational script (Phase 1–7). Follow step-by-step.
2. `.claude/rules/branch-manager.md` — operating rules, hard refusals, autonomy bounds.
3. `.claude/rules/phase-branch.md` — phase-branch flow + merge convention.
4. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory.
5. `.claude/agents/bm-task.md` — your contract (no-AskUserQuestion, DQ-blocked stop pattern).

## 4. Constraints

**Pre-conditions verified at brief-write time:**
- Branch `phase-v1-JM-e` @ `9ac1e9143` (synced with origin/phase-v1-JM-e)
- Working tree clean
- 60+ commits ahead of governance-v0
- PR #107 state: `mergeable: MERGEABLE`, `mergeStateStatus: UNSTABLE` (from advisory red-flag scan; maintainer-ack already posted)
- Findings YAML state: 15 done · 5 rebut · 7 wont-fix · **0 open**
- 0 critical findings remain in `fix-in-pr` (mergability gate cleared)
- Retro authored, committed, pending advisor sign-off (deferred to after merge per JM-d/c convention — sign-off happens at `/brehon-phase-transition`)
- Two e2e regressions PASS (66/0/3 in 28:42 on c83aaec05; 66/0/3 in 28:40 on 537bd2f0f)

**Re-verify in Phase 1:** if any pre-condition has drifted (new findings landed since 13:50 UTC, branch diverged from origin, etc), file a `pending` DQ entry from `from: "bm"`, kind: `"blocker"`, with the specific drift, commit + push to `phase-v1-JM-e`, return `blocked-on-DQ-#<id>`.

**About `mergeStateStatus: UNSTABLE`:** the red-flag scan keeps failing because the regex scanner can't see the role-dispatch refactor in `accept_jury_assignment.rs`. The advisory comment posted on the PR (issuecomment-4363778416) is the maintainer-ack per the bot's instruction ("a maintainer must acknowledge each finding before merge"). Proceed with the merge; UNSTABLE here is the advisory-only state, not a hard CI fail.

**Exact merge invocation:**
```bash
gh pr merge 107 --repo barrie-cork/lemmy --merge
```

If `gh pr merge` refuses with "merge commit cannot be created" or "branch is required to be up-to-date" — STOP and surface a DQ entry. Do NOT add `--admin` without explicit advisor reconsent. Do NOT manually rebase on top of governance-v0.

**Linux discipline:** EliteDesk worker → laptop foreground branch-manager subagent (Tailscale ACL still blocks Junior daemon SSH). Use `bash`/`git`/`gh`. No `.bat` wrappers.

**--repo barrie-cork/lemmy:** mandatory on every `gh` invocation per `gh-pr-fork-target.md`.

**Post-merge cleanup:**
- Switch local checkout to `governance-v0` and `git pull --ff-only origin governance-v0` to get the merge commit.
- Capture the merge commit SHA via `git log --merges -1 --pretty=%H origin/governance-v0`.
- Append the bm-runlog entry on `governance-v0` (NOT phase-v1-JM-e; the phase branch is now historical).
- Do NOT delete the remote `phase-v1-JM-e` branch — leave it as the historical record. The user will delete it manually (or via `/brehon-phase-transition`) if desired.

**Commit message format for runlog:** `chore(bm): record PR-merge for phase-v1-JM-e (PR #107)`. Lands on `governance-v0`.

**No Telegram this run.** No `bm-ping` invocation. Advisor will queue separately if user wants channel notified post-merge.

## 5. Expected output

A 5-section summary per the bm-merge script Phase 7:

```
## /bm-merge complete

**PR #107:** Phase v1-JM-e — Appeal-vote tally + integration capstone + step-up / spoofing / admin-visibility hardening
**Merge type:** merge commit (NOT squash, NOT rebase)
**Merge commit SHA on governance-v0:** <new-merge-sha>
**Final phase tip merged:** 9ac1e9143
**Findings YAML at merge:** 15 done · 5 rebut · 7 wont-fix · 0 open

### What happens next (user-driven)

1. Advisor session runs `/brehon-phase-transition` to author the five-deliverable handoff per the brehon-phase-transition skill.
2. User decides whether to delete the remote `phase-v1-JM-e` branch (historical-record question; default keep).
3. Next sub-phase: pick from PRD §17 (likely `v1-sponsor-liability-d` or `v1-rep-tuning-r3`).

### Telegram ping

NOT sent this run. Advisor will queue `/bm-ping merge-ready` separately if the user wants the channel notified.
```

The advisor's polling loop reads this output and proceeds to the post-merge phase-transition gate.
