---
phase: v1-SL-e
role: bm-task
task: bm-merge
brief_n: 11
authored: 2026-05-13
---

# [role:bm-task] SL-e bm-merge — post pr-127-comment.md + merge PR #127 into governance-v0

## §1 Role + dispatch

`[role:bm-task] SL-e bm-merge — post pr-127-comment.md + merge PR #127 (user gates 3+5 cleared 2026-05-13)`

## §2 Scope

User has approved BOTH:
- **Gate 3 (CR triage):** all 15 findings resolved as 8 done + 7 rebut, 0 open.
- **Gate 5 (merge confirm):** post the digest comment + merge in one step.

**PR:** `#127` (phase-v1-SL-e → governance-v0)
**Phase branch tip:** `786dc1546` (advisor correction commit; recommendation: approve)
**Mergeable:** CLEAN
**Status checks:** Red-flag diff scan SUCCESS, CodeRabbit SUCCESS

### Step A — post the digest comment

The comment is drafted at `.claude/PRPs/reviews/pr-127-comment.md` on the **canonical brehon-fork checkout** (governance-v0; the file is gitignored at the BM artifacts pattern, so it lives only locally on the laptop checkout — NOT on phase-v1-SL-e). The bm-task worker WILL NOT see this file in its worktree.

**Therefore the comment body must be reconstructed in the bm-task** from the findings YAML state (15 findings on phase-v1-SL-e tip `786dc1546`):
- 8 done: cr-2, cr-3, cr-5, cr-10, copilot-1, copilot-2, copilot-3, copilot-4
- 7 rebut: cr-1, cr-4, cr-6, cr-7, cr-8, cr-9, cr-11

Alternatively (simpler): the worker can `git fetch origin governance-v0` then `git show origin/governance-v0:.claude/PRPs/reviews/pr-127-comment.md` IF the file is on governance-v0. **Check first**: `git ls-tree origin/governance-v0 -- .claude/PRPs/reviews/pr-127-comment.md`. If empty (gitignored), reconstruct the comment per the bullet list below.

**Reconstruction template** (use this verbatim if pr-127-comment.md is not retrievable):

```markdown
# PR #127 — review-response digest (final)

Thanks for the two-round review (5 CR + 4 Copilot initial; 6 CR follow-up). Final bucket triage of all 15 findings:

## Summary by bucket

| Bucket | Count | IDs |
|---|---|---|
| done | 8 | cr-2, cr-3, cr-5, cr-10, copilot-1, copilot-2, copilot-3, copilot-4 |
| rebut | 7 | cr-1, cr-4, cr-6, cr-7, cr-8, cr-9, cr-11 |

**Counters:** 0 open across all severities. Fix commits: 9871151f5, 2182f0542, 786dc1546.

## Rebuttals

- **cr-1**: DQ id immutability is forward-looking guidance, no defect in this PR. v2 rule already encodes the principle.
- **cr-4, cr-6, cr-7, cr-8, cr-9**: All target `.claude/runlog/v1-SL-e-runlog.md` — a gitignored runtime artifact (BM audit trail, append-only). MD022/arithmetic/label nits don't apply to a non-tracked human-readable log; format kept consistent with prior phase runlogs.
- **cr-11**: `bucket: rebut` (entry verb) vs `counters.<sev>.rebutted` (count noun) is schema-by-design per SCHEMA.md L58 + L94. Token asymmetry is structural, not drift.

All workflow checks green. Ready to merge.

---

🤖 Drafted as part of `bm-triage` + post-fix re-poll for PR #127.
```

Post via `gh pr comment 127 --repo barrie-cork/lemmy --body-file <tmpfile>` (write the markdown to a tmpfile first; do NOT use `--body` with HEREDOCs in a worker subprocess — HEREDOC handling varies and can corrupt the markdown).

### Step B — merge

After comment is posted, run `gh pr merge 127 --repo barrie-cork/lemmy --merge` (do NOT use `--squash` — task-per-commit history is load-bearing for retros per `.claude/rules/phase-branch.md` "Do not").

**Hard refusal**: if `gh pr merge` returns non-zero exit, surface verbatim and STOP — do NOT retry with a different merge strategy. Per `.claude/rules/auto-phase.md` "Hard refusals" gate 9.

### Step C — verify branch deletion

After merge succeeds, run `git ls-remote origin refs/heads/phase-v1-SL-e`. If the branch is still present (silent-skip case), run:

```bash
gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/phase-v1-SL-e
```

Single attempt; on second failure, surface to user.

### Step D — append runlog entry

Add to `.claude/runlog/v1-SL-e-runlog.md`:

```markdown
## bm: merge — <UTC ISO timestamp>

- **PR:** `#127`
- **Action:** merged phase-v1-SL-e → governance-v0
- **Merge commit:** `<merge_commit_sha>` (from `gh pr view 127 --json mergeCommit`)
- **Comment posted:** yes (pr-127-comment.md final digest)
- **Counters:** 0 open / 8 done / 7 rebut
- **Branch deleted:** yes
```

Commit subject: `chore(bm): merge PR #127 — v1-SL-e lane-closer landed in governance-v0`. Push to governance-v0.

## §3 Required reading

- `.claude/commands/bm/bm-merge.md` — bm-merge verb
- `.claude/rules/branch-manager.md` — autonomy bounds (`gh pr comment` + `gh pr merge` both confirmed-by-user this dispatch)
- `.claude/rules/phase-branch.md` — `--merge` strategy (not squash)
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/rules/auto-phase.md` "Hard refusals" — gate 9 (non-zero `gh pr merge` = stop, do not retry) + L16 (branch deletion post-condition)

## §4 Constraints

- **--repo barrie-cork/lemmy** on EVERY gh command (no exceptions)
- **Order:** comment FIRST, then merge — never the reverse. If comment post fails, surface and STOP before merging.
- **No `--squash` or `--rebase`** on `gh pr merge`. Use `--merge` (default merge commit).
- **No `--delete-branch` flag.** GitHub auto-deletes if repo settings allow; otherwise Step C handles it explicitly.
- **No retries.** Each gh subcommand single-attempt; on failure, surface verbatim error and STOP.
- **Touch only:** `.claude/runlog/v1-SL-e-runlog.md` (append) + post comment + merge action. NO `crates/`, NO `migrations/`, NO `.claude/PRPs/*`.
- **Runlog commit must land on governance-v0** (the canonical advisor-orchestration tracking branch), NOT on phase-v1-SL-e (already merged or being merged).
- After Step D succeeds, end task. Daemon finalize-merge will push the runlog commit.
