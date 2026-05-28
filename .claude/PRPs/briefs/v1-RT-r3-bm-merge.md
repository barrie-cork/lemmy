# v1-RT-r3 — bm-merge execute-side brief

## 1. Role + dispatch line

`[role:bm-task]` bm-merge — execute the merge of PR #155 (v1-RT-r3 → governance-v0).

Dispatch:

```
[role:bm-task] v1-RT-r3 bm-merge — see .claude/PRPs/briefs/v1-RT-r3-bm-merge.md
```

## 2. Scope

Execute Phases 5-9 of `.claude/commands/bm/bm-merge.md`:

- Phase 5: `gh pr merge 155 --repo barrie-cork/lemmy --merge --delete-branch` (NOT `--squash`).
- Phase 5.5: advisor post-condition verification — verify `state == "MERGED"`, `mergedAt` non-null, `mergeCommit.oid` is a real sha BEFORE accepting `result:success`. If ANY condition fails → hard catch-fire, do NOT improvise.
- Phase 6: fast-forward local `governance-v0` to the new tip after merge.
- Phase 7: archive findings YAML with `merged_at` + `merge_commit` + `final_recommendation: approve`.
- Phase 8: append runlog COMPLETE entry on `governance-v0` POST-merge with the REAL merge sha.
- Phase 9: return success summary with merge SHA + trunk position.

Gate-side (Phases 1-4) already ran inline in the advisor session — all gates pass; user authorised merge via AskUserQuestion (confirm). Do NOT re-run gate checks; proceed directly to Phase 5.

**Phase tip pre-merge:** `ada35e1b2`. **Base:** `governance-v0`. **Head:** `phase-v1-RT-r3`.

## 3. Required reading

- `.claude/rules/branch-manager.md` — autonomy bounds + file ownership
- `.claude/rules/phase-branch.md` — `--merge` not `--squash`; only into `governance-v0`
- `.claude/commands/bm/bm-merge.md` Phases 5-9 verbatim
- `.claude/rules/decision-queue.md` — attribution
- `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` — POST-merge runlog ordering
- `feedback_bm_false_success_advisor_post_condition_catch.md` — hard refusal #5

## 4. Constraints

### 4.1 Explicit numbered git sequence (L14 fix, POST-merge runlog)

1. `git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0`
2. Re-verify PR #155 is MERGEABLE: `gh pr view 155 --repo barrie-cork/lemmy --json mergeable,state`. If CONFLICTING / not OPEN → STOP + surface (do NOT improvise).
3. `gh pr merge 155 --repo barrie-cork/lemmy --merge --delete-branch`
4. Wait for return. If exit non-zero → **HARD REFUSAL**: capture verbatim error, STOP, surface. NEVER retry, NEVER `git push -f`, NEVER hand-resolve.
5. POST-MERGE ONLY (merge succeeded):
   - `git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0`
   - Get real merge sha: `gh pr view 155 --repo barrie-cork/lemmy --json mergeCommit -q .mergeCommit.oid`
   - Edit `.claude/runlog/bm-runlog.md` — append the `## bm: merge COMPLETE — <ISO timestamp>` entry per bm-merge.md Phase 8 template with REAL merge sha + PR #155 + base/head + remote-branch-deleted-status + trunk position
   - Also update `.claude/PRPs/reviews/pr-155-findings.yaml` — add top-level `merged_at: <ISO>`, `merge_commit: <real-sha>`, `final_recommendation: approve`
   - `git add .claude/runlog/bm-runlog.md .claude/PRPs/reviews/pr-155-findings.yaml`
   - `git commit -m "chore(bm): merge PR #155 complete — runlog COMPLETE + findings archive"`
   - `git push origin governance-v0`
6. L16 remote-branch deletion follow-up: `git ls-remote origin refs/heads/phase-v1-RT-r3`. If still present → run ONCE: `gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/phase-v1-RT-r3`. Re-check; if still present after the single attempt, note "manual branch-delete follow-up" in output and STOP (do NOT loop).

### 4.2 Hard refusals (5 categorical, per bm-merge.md spec)

1. **NEVER `git push -f`, `git push --force`, or `git push --force-with-lease`** on ANY branch. Not on `phase-*`, not on `governance-v0`, not on anything. If a push fails, capture the verbatim error, STOP, surface.
2. **NEVER retry `gh pr merge`.** A failed merge is a clean stop. Capture the verbatim error message, STOP, surface. Do NOT issue the command a second time under any circumstance.
3. **NEVER hand-resolve a merge conflict.** Do NOT stage a local merge. Do NOT edit conflict markers. Do NOT improvise an alternate merge strategy. The merge fails → STOP.
4. **On any merge failure, ONE clean stop is the ONLY acceptable failure behavior.** No creative recovery. No fallback path. No "try `--squash` instead". The advisor decides next steps; the BM Junior surfaces and waits.
5. **NEVER self-report `result:success` when the primary objective failed.** If `gh pr merge` exited non-zero, the merge did not happen; the task did not succeed. Self-attribution must be honest; the advisor's post-condition check (Phase 5.5) is independent verification, not a license for the subagent to optimistically self-report.

### 4.3 File ownership

- **OWN:** `.claude/runlog/bm-runlog.md`, `.claude/PRPs/reviews/pr-155-findings.yaml`
- **NEVER touch:** `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `Cargo.{toml,lock}`, `rust-toolchain.toml`

### 4.4 Attribution

Commit subjects MUST match `^(chore|docs)\((advisor|decision-queue|bm)\)` per `.claude/rules/decision-queue.md` Attribution. NEVER write `answered_by: "advisor"` or `answered_by: "user"` from this BM session.

### 4.5 PR target

ALL `gh pr` / `gh issue` commands MUST include `--repo barrie-cork/lemmy` per `.claude/rules/gh-pr-fork-target.md`.

## 5. Validation

Phase 5.5 advisor post-condition verification is the validation gate:

- `gh pr view 155 --repo barrie-cork/lemmy --json state,mergedAt,mergeCommit` after Phase 5
- ALL three must hold: `state == "MERGED"`, `mergedAt` non-null, `mergeCommit.oid` is a real 40-char sha
- ANY condition fails → STOP, surface verbatim `gh pr view` output

No cargo gates for this brief (no code edits).

## 6. Return shape

Phase 9 output per bm-merge.md template, plus:
- Real merge SHA (verified)
- New trunk tip (post `git pull --ff-only`)
- "Next suggested": author retro per `feedback_retro_not_report` → `/brehon-phase-transition`
