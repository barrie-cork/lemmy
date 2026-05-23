# [role:bm-task] bm-merge PR #147 — v1-ship-2

## 1. Role + dispatch

`[role:bm-task]` — execute merge of PR #147 (phase-v1-ship-2 → governance-v0).

Advisor has completed the gate-side checks (Phase 1–4 of bm-merge.md) inline.
This brief covers the execute-side only (Phases 5–9).

## 2. Scope

Execute merge of PR #147 and complete post-merge bookkeeping.

**Produce:**
- Merged PR #147 on governance-v0
- Post-merge runlog COMPLETE entry in `.claude/runlog/bm-runlog.md`
- Updated `.claude/PRPs/reviews/pr-147-findings.yaml` (merged_at, merge_commit)
- Remote branch `phase-v1-ship-2` deleted

**Do NOT author:**
- Any implementation code
- Any plan edits
- Any new DQ entries (no blockers anticipated; file `kind: "blocker"` only if merge fails)

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — full verb spec; follow Phases 5–9 exactly
- `.claude/rules/branch-manager.md` — file ownership + hard refusals
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory

## 4. Constraints (HARD)

### Git sequence (L14 fix — POST-merge runlog, from bm-merge.md)

Execute in this exact order:

1. `git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0`
2. Re-verify: `gh pr view 147 --repo barrie-cork/lemmy --json mergeable,state`
   - If not `OPEN` + `MERGEABLE` → STOP + surface. Do NOT improvise.
3. `gh pr merge 147 --repo barrie-cork/lemmy --merge --delete-branch`
4. Wait for return. If exit non-zero → HARD STOP. Capture verbatim error. Surface. NEVER retry.
5. POST-MERGE ONLY (merge succeeded):
   - `git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0`
   - Get real merge sha: `gh pr view 147 --repo barrie-cork/lemmy --json mergeCommit -q .mergeCommit.oid`
   - Edit `.claude/runlog/bm-runlog.md` — append COMPLETE entry (see Phase 8 template in bm-merge.md)
   - `git add .claude/runlog/bm-runlog.md`
   - `git commit -m "chore(bm): merge PR #147 complete — v1-ship-2"`
   - `git push origin governance-v0`
6. L16: `git ls-remote origin refs/heads/phase-v1-ship-2`
   - If still present → run ONCE: `gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/phase-v1-ship-2`
   - If still present after single attempt → note in output, STOP (do NOT loop)

### Hard refusals (from bm-merge.md)

1. NEVER `git push -f`, `git push --force`, or `git push --force-with-lease` on ANY branch
2. NEVER retry `gh pr merge` — one attempt only
3. NEVER hand-resolve a merge conflict
4. NEVER self-report `result:success` if `gh pr merge` exited non-zero
5. Phase 5.5 post-condition check is MANDATORY before Phase 6:
   - `gh pr view 147 --repo barrie-cork/lemmy --json state,mergedAt,mergeCommit`
   - Require: `state=="MERGED"`, `mergedAt` non-null, `mergeCommit.oid` 40-hex chars
   - Any condition fails → catch-fire, STOP, surface

### PR details

- **PR:** #147
- **Title:** v1-ship-2 — 4 governance e2e tests (appeal, modlog, reputation, endorsement)
- **Head:** phase-v1-ship-2 (tip: 65cef4f97)
- **Base:** governance-v0
- **Findings YAML:** `.claude/PRPs/reviews/pr-147-findings.yaml` (gitignored, local only)
  - Update with `merged_at` and `merge_commit` after Phase 5.5 confirms success
