# Brief: v1-deps-r1 BM-MERGE

## 1. Role + dispatch

`[role:bm-task] v1-deps-r1-bm-merge — see .claude/PRPs/briefs/v1-deps-r1-bm-merge-1.md`

## 2. Scope

Merge PR #153 (`phase-v1-deps-r1 → governance-v0`) on `barrie-cork/lemmy`.

**Produce:**
- PR #153 merged (squash disabled — task-per-commit history is load-bearing for retros)
- Runlog entry in `.claude/runlog/v1-deps-r1-runlog.md`
- PR number + merge SHA recorded in task output

**Do NOT:**
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `docs/`
- Force-push or delete branches without confirmation
- Squash the PR (per `phase-branch.md`: "Do not squash the PR at merge")

## 3. Pre-merge gate (verify before merging)

All gates have been cleared by the advisor. Verify the following before issuing the merge command:

1. **No open critical findings:** scan `.claude/PRPs/reviews/pr-153-findings.yaml` — assert `fix-in-pr` count = 0 and no `severity: critical` findings in `fix-in-pr` bucket.
2. **PR is open and mergeable:** `gh pr view 153 --repo barrie-cork/lemmy --json state,mergeable,mergeStateStatus`
3. **No pending DQ entries:** read `.claude/decision-queue.json` — assert `pending` array is empty.

If any gate fails, write a `kind: "blocker"` DQ entry and stop.

## 4. Merge command

```bash
gh pr merge 153 --repo barrie-cork/lemmy --merge --delete-branch
```

Use `--merge` (not `--squash`, not `--rebase`). The `--delete-branch` flag cleans up the remote phase branch after merge.

## 5. Required reading

- `.claude/rules/branch-manager.md` — file-ownership boundaries, autonomy bounds, merge gate
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/rules/phase-branch.md` — no squash at merge

## 6. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- PR number: 153
- No squash merge
- Record merge SHA in `.claude/runlog/v1-deps-r1-runlog.md`
- Ask before sending any Telegram ping
