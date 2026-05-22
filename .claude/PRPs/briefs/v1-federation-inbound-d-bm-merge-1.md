---
role: bm-task
phase: v1-federation-inbound-d
brief_n: bm-merge-1
kind: bm-merge
---

# [role:bm-task] v1-federation-inbound-d bm-merge-1 — merge PR #146 (execute-side only)

## 1. Role + dispatch

`[role:bm-task]` Execute-side merge of PR #146 (`phase-v1-federation-inbound-d` →
`governance-v0`). Gate-side checks (pre-merge readiness) were run inline by the advisor
session and confirmed clear: Stories 1+2 ✓, no open critical findings, user gate 5
approved. This brief covers Phases 5-9 only — the actual merge + post-merge bookkeeping.

## 2. Scope

Merge PR #146 using the exact L14-revised git sequence below. No squash, no rebase.
Write the runlog COMPLETE entry **POST-merge** (per L14 fix, 2026-05-18 revision).

**Target PR:** `#146` (`phase-v1-federation-inbound-d` → `governance-v0`)
**Repo:** `barrie-cork/lemmy`
**Head branch to delete:** `phase-v1-federation-inbound-d`

### Mandatory git sequence (numbered — follow exactly)

1. `git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0`
2. Re-verify PR #146 is MERGEABLE:
   `gh pr view 146 --repo barrie-cork/lemmy --json mergeable,state`
   If `state != "OPEN"` or `mergeable == "CONFLICTING"` → **STOP + surface. Do NOT proceed.**
3. `gh pr merge 146 --repo barrie-cork/lemmy --merge --delete-branch`
4. If exit non-zero → **HARD REFUSAL**: capture verbatim error, STOP, surface. NEVER retry,
   NEVER `git push -f`, NEVER hand-resolve.
5. **POST-MERGE ONLY** (step 3 succeeded):
   - `git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0`
   - Get real merge SHA: `gh pr view 146 --repo barrie-cork/lemmy --json mergeCommit -q .mergeCommit.oid`
   - Edit `.claude/runlog/bm-runlog.md` — append entry:
     ```
     ## bm: merge COMPLETE — PR #146 phase-v1-federation-inbound-d → governance-v0
     sha: <real-merge-sha>
     merged_at: <ISO8601>
     ```
   - `git add .claude/runlog/bm-runlog.md`
   - `git commit -m "chore(bm): merge PR #146 complete — runlog COMPLETE entry"`
   - `git push origin governance-v0`
6. **L16 branch-delete verification:**
   `git ls-remote origin refs/heads/phase-v1-federation-inbound-d`
   If still present → run ONCE:
   `gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/phase-v1-federation-inbound-d`
   Re-check. If still present after single attempt → note "manual branch-delete follow-up"
   in output and STOP (do NOT loop).

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — L14 fix, hard-refusal contract
- `.claude/rules/branch-manager.md` — file ownership + autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`

## 4. Constraints

1. **NEVER `git push -f`, `--force`, or `--force-with-lease` on ANY branch.**
2. **NEVER retry `gh pr merge`.** One attempt. If it fails → STOP + surface verbatim error.
3. **NEVER hand-resolve a merge conflict.** Do NOT stage a local merge, edit conflict
   markers, or improvise an alternate merge strategy. Merge fails → STOP.
4. **NEVER commit to `crates/`, `migrations/`, `tests/`, or `docs/brehon-law-inspired-network/`.**
5. Runlog COMPLETE entry goes AFTER merge succeeds (POST-merge), not before.
6. Use `--merge` (not `--squash`, not `--rebase`) — task-per-commit history is load-bearing.
7. Pass `--repo barrie-cork/lemmy` on every `gh pr` command.
