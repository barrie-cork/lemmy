---
phase: v1-AD-e
role: bm-task
task: bm-merge
brief_n: 1
authored: 2026-05-17
---

# [role:bm-task] v1-AD-e bm-merge — merge PR #133 into governance-v0 (NO comment; gates 3+5 cleared)

## §1 Role + dispatch

`[role:bm-task] v1-AD-e bm-merge — merge PR #133 (user gate 5 confirmed 2026-05-17; gate 3 = no CR digest comment)`

## §2 Scope

This brief carries **ONLY Phases 5-9 of bm-merge.md** (the execute-side). The gate-side Phases 1-4 (read-only pre-checks + AskUserQuestion confirm) were run **ADVISOR-SIDE INLINE** per L15 (v1-SL-c-1 retro) and ALL PASSED:

- critical.open == 0 ✓ · major.open == 0 ✓ · recommendation: approve ✓
- all fix-in-pr rows (cr-1/cr-4/cr-5/cr-6) carry an `addressed_in` SHA ✓ · cr-2 rebut + cr-3 wont-fix (rationale present) ✓
- mergeStateStatus CLEAN ✓ · mergeable MERGEABLE ✓
- governance-v0 has NO branch protection → CodeRabbit is NOT a required check (advisory only) ✓
- DQ pending mentioning PR #133: 0 (pending total = 0) ✓
- Phase 2.5 (CR re-poll-since): user-authorized override — at gate 5 the user was shown the pending post-fix-impl-2 CodeRabbit re-review verbatim and chose "Confirm merge". Recorded as DQ #247 (`kind: log`, `answered_by: user`) for audit. ✓

**This dispatch does NOT post any PR comment.** Gate 3 was resolved "Approve triage, fix-in-PR (no comment)" — there is NO `pr-133-comment.md` and NONE is to be created. Skip the comment step entirely (unlike the sl-e canonical sibling which had a Step A comment).

**PR:** `#133` (phase-v1-AD-e → governance-v0)
**Phase branch tip:** `c9da8f79c` (advisor DQ #247 log commit; recommendation: approve)
**Mergeable:** CLEAN
**Status checks:** CodeRabbit (advisory, not required — no branch protection)

### Step A — append runlog entry FIRST (L14 — before merge)

Per the L14 fix in `.claude/commands/bm/bm-merge.md`, the runlog commit MUST land BEFORE `gh pr merge` so the audit trail is durable even if merge errors mid-flight.

Append to `.claude/runlog/v1-AD-e-runlog.md`:

```markdown
## bm: merge — <UTC ISO timestamp>

- **PR:** `#133`
- **Action:** merged phase-v1-AD-e → governance-v0
- **Merge commit:** `<merge_commit_sha>` (from `gh pr view 133 --repo barrie-cork/lemmy --json mergeCommit --jq .mergeCommit.oid` AFTER merge — see note below)
- **Comment posted:** no (gate 3 = no CR digest comment)
- **Counters:** 0 open / cr-1+cr-4+cr-5+cr-6 done / cr-2 rebut / cr-3 wont-fix
- **CR fix cycles:** fix-impl-1 5d742a323 (cr-1/4/5-partial/6, recovered #297) + fix-impl-2 5805ab27f (cr-5 ADR-015 completion, recovered #299 — 5th #292 stale-base rescue)
- **DQ #245 + #246:** both validate-pending-laptop result=pass (full e2e 94/0)
- **Branch deleted:** yes
```

**Merge-commit-sha note:** the runlog commit lands BEFORE merge, so `<merge_commit_sha>` is not yet known at Step-A commit time. Write the placeholder literal `<pending — see Phase 9 output>` for the merge commit line in the Step-A commit. After Phase 9, the advisor's L14 belt-and-braces tick will reconcile it if needed — do NOT block the merge to backfill it.

### Step B — the explicit numbered git sequence (L14, MANDATORY ORDER)

Run EXACTLY in this order. Do NOT checkout-before-commit (the c-1 2026-05-06 failure mode that lost the runlog Edit):

```
1. (Step A done) Edit .claude/runlog/v1-AD-e-runlog.md — append the bm: merge entry above.
2. git add .claude/runlog/v1-AD-e-runlog.md
3. git commit -m "chore(bm): merge PR #133 — v1-AD-e admin-dashboard-HTML lane landed in governance-v0"
4. git push origin governance-v0
5. THEN: gh pr merge 133 --repo barrie-cork/lemmy --merge
6. After merge: git ls-remote origin refs/heads/phase-v1-AD-e — if branch STILL present, run:
   gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/phase-v1-AD-e
```

**Runlog commit branch:** `governance-v0` (the canonical advisor-orchestration tracking branch), NOT phase-v1-AD-e (being merged + deleted). To commit on governance-v0 from the bm worktree: `git fetch origin governance-v0 && git checkout governance-v0 && git pull --ff-only origin governance-v0`, make the runlog edit there, commit, push — THEN run the merge. (The runlog Edit content is reconstructable from this brief; if v1-AD-e-runlog.md on governance-v0 lags the phase-branch copy, that is expected — append the bm: merge entry to whatever the governance-v0 copy's tail is; do NOT attempt to reconcile the full phase-branch runlog into governance-v0.)

### Step C — verify branch deletion (L16 post-condition)

After merge succeeds, `git ls-remote origin refs/heads/phase-v1-AD-e`. Empty = deleted (good). Still present = run the `gh api -X DELETE` from Step B.6. Single attempt; on second failure, surface to user.

### Step D — output

Return: merge SHA, trunk position (`git log -1 --oneline governance-v0` after `git pull --ff-only origin governance-v0`), branch-deleted yes/no, runlog-commit SHA.

## §3 Required reading

- `.claude/commands/bm/bm-merge.md` — bm-merge verb (Phases 5-9 only; Phases 1-4 already done advisor-side)
- `.claude/rules/branch-manager.md` — autonomy bounds (`gh pr merge` user-confirmed this dispatch; NO comment this dispatch)
- `.claude/rules/phase-branch.md` — `--merge` strategy (NEVER `--squash`/`--rebase`; task-per-commit history load-bearing for retros)
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory on EVERY gh command
- `.claude/rules/auto-phase.md` "Hard refusals" — gate 9 (non-zero `gh pr merge` = STOP, do NOT retry an alternate strategy) + L16 (branch-deletion post-condition)

## §4 Constraints

- **--repo barrie-cork/lemmy** on EVERY gh command (no exceptions).
- **Order is MANDATORY:** runlog commit + push to governance-v0 FIRST, THEN `gh pr merge`. Never the reverse. (L14 — without explicit ordering the BM Junior has checkout-before-committed and lost the runlog Edit.)
- **NO PR comment.** Gate 3 = "no CR digest comment". Do NOT create or post `pr-133-comment.md` or any `gh pr comment`. (This is the key divergence from the sl-e canonical sibling.)
- **No `--squash` or `--rebase`** on `gh pr merge`. Use `--merge` (default merge commit).
- **No `--delete-branch` flag.** GitHub auto-deletes if repo settings allow; otherwise Step C handles it explicitly via `gh api -X DELETE`.
- **No retries.** Each gh subcommand single-attempt; on failure surface verbatim error and STOP (gate-9 hard refusal).
- **Touch ONLY:** `.claude/runlog/v1-AD-e-runlog.md` (append, on governance-v0) + the merge action + the post-merge branch-delete. ZERO `crates/`, ZERO `migrations/`, ZERO `tests/`, ZERO `.claude/PRPs/*`, ZERO `docs/brehon-law-inspired-network/`. (File-ownership HARD per branch-manager.md — any write outside this set is a catch-fire breach.)
- **Runlog commit MUST land on governance-v0** (canonical advisor-orchestration tracking branch), NOT phase-v1-AD-e.
- After Step D succeeds, END task. Daemon finalize-merge pushes the governance-v0 runlog commit.
