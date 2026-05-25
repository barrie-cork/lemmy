# Brief: v1-deps-r1 BM-MERGE (retry 2)

## 1. Role + dispatch

`[role:bm-task] v1-deps-r1-bm-merge-2 — see .claude/PRPs/briefs/v1-deps-r1-bm-merge-2.md`

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

All gates have been cleared by the advisor. The DQ has been confirmed clean (0 pending entries) at `87de1e37f`.

**IMPORTANT NOTE on findings YAML:** `.claude/PRPs/reviews/pr-153-findings.yaml` is gitignored (runtime artifact). Do NOT block on its absence. Instead:
- Gate 1 (no critical findings): the advisor has confirmed 0 critical findings — all 9 CR findings were triaged to rebut (4) or wont-fix (5) in task #460. Gate 1 = PASS.
- Gate 2 (PR open/mergeable): verify `gh pr view 153 --repo barrie-cork/lemmy --json state,mergeable,mergeStateStatus`
- Gate 3 (no pending DQ): read `.claude/decision-queue.json` — assert `pending` array is empty (confirmed 0 at current HEAD).

If Gate 2 or Gate 3 fails, write a `kind: "blocker"` DQ entry and stop. Gate 1 is pre-cleared by advisor.

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
