---
role: bm-task
verb: bm-merge
phase: v1-quality-r2a
pr_number: 161
created: 2026-05-29
related_dq: null
---

# Brief: v1-quality-r2a bm-merge — merge PR #161 into governance-v0

## 1. Role + dispatch line

`[role:bm-task] v1-quality-r2a bm-merge — see .claude/PRPs/briefs/v1-quality-r2a-bm-merge-1.md`

You are the **bm-task** subagent (Haiku 4.5). Execute branch-manager verb `bm-merge` per `.claude/commands/bm/bm-merge.md`.

## 2. Scope

Merge PR #161 (`phase-v1-quality-r2 → governance-v0`) on `barrie-cork/lemmy`.

- **Phase:** `v1-quality-r2a` (branch `phase-v1-quality-r2`)
- **PR number:** #161 on `barrie-cork/lemmy`
- **Phase tip:** `be0807571477d77778eccf4ca365acf443954f6d` on `phase-v1-quality-r2`
- **Base:** `governance-v0`

**Produce:**

- PR #161 merged via `gh pr merge 161 --repo barrie-cork/lemmy --merge --delete-branch` (no squash — task-per-commit history is load-bearing for retros).
- Runlog entry in `.claude/runlog/v1-quality-r2a-runlog.md` noting merge SHA + timestamp.
- PR number + merge SHA recorded in task output.

**Do NOT:**

- Touch any file in `crates/`, `migrations/`, `tests/`, `crates/server/tests/`, `docs/brehon-law-inspired-network/`, `.claude/PRPs/plans/**`, `.claude/PRPs/prds/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`.
- Force-push or delete branches outside the `gh pr merge --delete-branch` flag (which is part of the verb).
- Squash the PR.
- Send Telegram pings (separate confirm-gated verb).

## 3. Pre-merge gate (verify before merging)

All three gates have been **pre-cleared by the advisor 2026-05-29 09:50 UTC**. Re-verify before merging:

1. **No open critical / fix-in-pr findings:** scan `.claude/PRPs/reviews/pr-161-findings.yaml` — assert `counters.critical.open` = 0 and no `severity: critical` findings in `fix-in-pr` bucket. Advisor-cleared state: done=7, carry-forward=2 (cr-1, cr-2 → r2b), fix-in-pr=0, rebut=0, wont-fix=0.
2. **PR open + mergeable:** `gh pr view 161 --repo barrie-cork/lemmy --json state,mergeable,mergeStateStatus`. Advisor-cleared state: state=OPEN, mergeable=MERGEABLE, mergeStateStatus=CLEAN, head=`be0807571`.
3. **No pending DQ entries:** read `.claude/decision-queue.json` on `phase-v1-quality-r2` — assert `pending` array is empty. Advisor-cleared state: pending=0, resolved=214.

If any gate fails (state has drifted since 09:50 UTC), write a `kind: "blocker"` DQ entry per Hard refusal #14 and stop. Do NOT proceed to merge if a gate fails.

## 4. Merge command

```bash
gh pr merge 161 --repo barrie-cork/lemmy --merge --delete-branch
```

Use `--merge` (NOT `--squash`, NOT `--rebase`). The `--delete-branch` flag cleans up the remote `phase-v1-quality-r2` branch after merge — that's part of the verb's intended behaviour (Hard refusal #6 carve-out).

## 5. Required reading

- `.claude/commands/bm/bm-merge.md` — the bm-merge verb script
- `.claude/rules/branch-manager.md` — file-ownership boundaries, autonomy bounds, merge gate
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/rules/phase-branch.md` — no squash at merge
- `.claude/PRPs/reviews/pr-161-findings.yaml` — gate-1 verification source

## 6. Constraints

1. **NEVER touch `crates/`, `migrations/`, `tests/`, `crates/server/tests/`, `docs/brehon-law-inspired-network/`, `.claude/PRPs/plans/**`, `.claude/PRPs/prds/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`** (per `.claude/rules/branch-manager.md` §"File ownership boundaries").
2. **NEVER post a PR comment** (`gh pr comment`).
3. **NEVER submit a PR review** (`gh pr review --approve|--request-changes`).
4. **NEVER force-push.**
5. **NEVER delete a branch outside the `gh pr merge --delete-branch` flag.**
6. **NEVER send a Telegram ping** without separate confirmation.
7. **NEVER write `answered_by: "advisor"` or `approved_by`** in any DQ entry (Hard refusals #1 + #8 in `decision-queue.md`).
8. **NEVER use the abolished `next_id = max+1` recipe** — use `bash scripts/brehon/dq-v3-new-entry.sh` + `bash scripts/brehon/dq-v3-append-fragment.sh` (Hard refusal #9 in `decision-queue.md`).
9. **`--repo barrie-cork/lemmy` MANDATORY** on every `gh` command.
10. **Base branch MUST be `governance-v0`** (the PR's existing base — do NOT change it).
11. **PR MUST NOT be squashed** — `--merge`, not `--squash`, not `--rebase`.
12. **Refuse silently and raise a `kind: "blocker"` DQ** if any gate in §3 fails (drift since advisor pre-clearance).

**Commit subject discipline:** the merge happens server-side via `gh pr merge` — no client-side commit. The runlog entry uses subject `chore(runlog): v1-quality-r2a bm-merge — PR #161 merged at <sha>` per `phase-branch.md` direct-on-governance-v0 policy (runlog is meta-content, not crates/).

**Push discipline:** no client-side push (server-side merge). Runlog commit pushed to its worker branch; daemon finalize-merges into `governance-v0` (Lane Q-r2a's phase branch will be deleted by `--delete-branch`, so the runlog must land on trunk via worker-branch finalize-merge).

## 7. Success signals

- PR #161 state=MERGED on `barrie-cork/lemmy`.
- Merge SHA captured in task output.
- `phase-v1-quality-r2` deleted on origin (per `--delete-branch`).
- `.claude/runlog/v1-quality-r2a-runlog.md` appended with merge SHA + timestamp.
- Runlog commit pushed to worker branch.
- HANDOVER trailer on runlog commit.

## 8. Out of scope

- Authoring or cleaning up local `phase-v1-quality-r2` worktrees (advisor handles post-merge cleanup separately).
- Cutting a new lane for r2b or RT-r3-followup (separate sessions).
- Closing GitHub issues from PR body (#157 closed automatically via PR body's "Closes #157"; #158 stays open by design per `dd6012873857-001` trigger condition).
- Bm-merge-forward (`merge governance-v0 → phase-v1-quality-r2`) — not needed; advisor verified empty `origin/governance-v0 ^phase-v1-quality-r2` pre-clearance.

## 9. Done

PR #161 merged into `governance-v0` via `--merge --delete-branch`. Runlog entry committed and pushed to worker branch. Daemon finalize-merges runlog into `governance-v0`. Advisor session resumes at gate-6 (retro sign-off) for Lane Q-r2a, then `/brehon-phase-transition`.
