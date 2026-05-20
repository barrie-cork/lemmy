# Brief — v1-federation-inbound-b bm-merge (execute merge PR #139, L14 REVISED post-merge runlog)

## 1. Role + dispatch line

`[role:bm-task] bm-merge v1-federation-inbound-b — execute PR #139 merge + post-merge runlog COMPLETE`

Dispatcher → `branch-manager` subagent. Execute **ONLY the merge + post-merge runlog + L16 branch-delete verify** (a reduced Phase 5-9). Phases 1-4 (gate-side: findings counters, mergeStateStatus, CR re-review SUCCESS, user gate-5b confirm) already ran inline advisor-side per L15. **You MUST NOT write a runlog entry to governance-v0 BEFORE the merge — that exact step caused the L14 self-conflict that BLOCKED Junior #322 on v1-ship-1-r2.** The L14 ordering is now **POST-merge** per `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` (REVISED 2026-05-18).

## 2. Scope

Merge PR #139 (`phase-v1-federation-inbound-b` → `governance-v0`) via `gh pr merge 139 --repo barrie-cork/lemmy --merge --delete-branch`, then POST-merge runlog write on `bm-runlog.md` (governance-v0) with the real merge SHA, then L16 branch-delete verify.

**PR:** `#139` (phase-v1-federation-inbound-b → governance-v0)
**Phase branch tip:** `512e42d1c` (fix-impl-8 commit + daemon finalize-merge)
**Mergeable:** CLEAN / MERGEABLE
**Status checks:** CodeRabbit = SUCCESS at 13:12:54 UTC (re-reviewed fix-impl-8 tip, zero new actionable findings)

**Advisor-side gate evidence (already verified — context only, do NOT re-check):**

- PR #139 `mergeable: MERGEABLE`, `mergeStateStatus: CLEAN`, base `governance-v0`, head `512e42d1c6`.
- CR statusCheckRollup: CodeRabbit `state: SUCCESS` (the re-review on tip 512e42d1c).
- Findings YAML `.claude/PRPs/reviews/pr-139-findings.yaml`: critical.open 0, major.open 2 (cr-1 + cr-2, **addressed_in fix-impl-8 commit 9e78c886a**), low.open 2 (cr-3 + copilot-4 done-dup, **addressed_in fix-impl-8 commit 9e78c886a**), nit.wont_fix 13 (markdownlint on archived advisor artifacts, gate-3 ratified). Recommendation: pending → approve (advisor flips on post-merge if YAML is re-touched; YAML is gitignored so not required).
- DQ `.claude/decision-queue.json`: pending [] (fed-in-b's 9 §13-task validate-pending + DQ #290 Phase-2 e2e all result:pass and resolved).
- carry-forward (3 Copilot items: copilot-1/2/3 DoS-hardening family) — file as separate GH issue after merge, NOT a merge blocker.
- /brehon-verify: not re-run for fix-impl-8 (the 3 hunks are quality-bar improvements without semantic change; round-2 Phase-2 e2e on 4efce35c8 cleared the impl phase as verified at gate-5 approve).
- User Gate 5b (merge confirm): CONFIRMED 2026-05-20.

Boundaries:

- Do NOT touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `Cargo.{toml,lock}`, `rust-toolchain.toml`.
- Do NOT use `--squash`/`--rebase` — `--merge` ONLY (task-per-commit history load-bearing for retros).
- Do NOT merge into `main` — base is `governance-v0`.
- **Do NOT write/commit a runlog entry to governance-v0 (or anywhere) BEFORE the merge.** Post-merge ONLY (see §4 step 5).
- Do NOT re-run Phase 1-4 gate checks or re-ask the user (L15: ran advisor-side; confirm given 2026-05-20).
- Do NOT retry `gh pr merge` if it fails (hard refusal — see §4).
- Do NOT send a Telegram ping.
- Do NOT `git push -f` anything, ever (v1-ship-1-r2 Junior #322 attempted this on protected trunk — hard refusal).

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — Phase 5 (execute), Phase 6 (post-merge trunk verify), Phase 9 (output). **SKIP the Phase 8 runlog-BEFORE-merge ordering entirely — that is the L14 OLD-ORDER step that caused the v1-ship-1-r2 self-conflict; the post-merge follow-up in §4 step 5 is the L14 REVISED replacement.**
- `.claude/rules/branch-manager.md` — file-ownership + autonomy + "advisor-side gate-only" (L15).
- `.claude/rules/phase-branch.md` — `--merge` not `--squash`; base `governance-v0`.
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` MANDATORY on EVERY gh command.
- `.claude/rules/no-destructive-defaults.md` — NEVER `--force`/`-f`/`--hard`/`--no-verify` (Junior #322 breached this).
- `.claude/rules/auto-phase.md` "Hard refusals" #9 — `gh pr merge` non-zero = STOP, no retry, no alternate strategy. #L14 REVISED (post-merge runlog ordering). #L16 (post-condition branch-delete verify).
- `.claude/lessons/feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` — the canonical rationale for the post-merge ordering used here.

## 4. Constraints

- **Reduced git sequence (explicit numbered order — L14 REVISED post-merge):**

  ```
  1. git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0   (get on trunk; verify clean working tree)
  2. Re-verify PR #139 is still MERGEABLE: gh pr view 139 --repo barrie-cork/lemmy --json mergeable,mergeStateStatus,state  → mergeable MUST be "MERGEABLE" and state "OPEN". If mergeStateStatus is UNSTABLE because a check is PENDING (CodeRabbit re-review), that is NON-blocking per the prior treat-as-clean user decision — proceed. If mergeable is "CONFLICTING" or state is not "OPEN" → STOP and surface (do NOT improvise).
  3. gh pr merge 139 --repo barrie-cork/lemmy --merge --delete-branch
  4. Wait for it to return. If exit non-zero → HARD REFUSAL (see below). If exit 0 → continue.
  5. POST-MERGE ONLY (now that the merge succeeded): git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0. Capture the real merge SHA: REAL_SHA=$(gh pr view 139 --repo barrie-cork/lemmy --json mergeCommit -q .mergeCommit.oid). Edit .claude/runlog/bm-runlog.md — append a NEW short block at the END of the file:

     ## bm: merge COMPLETE — <UTC ISO timestamp>

     - **PR:** #139
     - **merge sha:** <REAL_SHA from the gh pr view above>
     - **trunk position:** <git rev-parse --short governance-v0> (<short subject>)
     - **remote branch deleted?** <yes if step 6 confirms, else no — see step 6>
     - **CR triage summary:** 3 fix-in-pr addressed in fix-impl-8 (commit 9e78c886a: cr-1 scheduled_tasks.rs replay_window_days clamp + cr-2 e2e.rs seed-loop bare-? + cr-3 e2e.rs assert_eq label count); 3 Copilot carry-forward (rate-map DoS-hardening family — file separate GH issue); 13 nit/wont-fix (markdownlint on archived advisor briefs/runlog, gate-3 ratified).
     - **DQs:** all fed-in-b validate-pending entries (9 §13 tasks + DQ #290 Phase-2 e2e) result:pass; pending: 0.

     Then: git add .claude/runlog/bm-runlog.md && git commit -m "chore(bm): merge PR #139 complete — runlog COMPLETE entry" && git push origin governance-v0.
     (This runlog write is POST-merge, so it cannot conflict with the phase branch — the phase branch no longer exists after --delete-branch. bm-runlog.md has `merge=union` per .gitattributes; even if a concurrent governance-v0 commit lands, union merge resolves cleanly.)
  6. L16 post-condition: git ls-remote origin refs/heads/phase-v1-federation-inbound-b — if STILL PRESENT (gh --delete-branch silently skipped), run ONCE: gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/phase-v1-federation-inbound-b. Re-check git ls-remote; if still present after the single DELETE attempt, note "branch-delete needs manual follow-up" in the Phase 9 output and STOP (do NOT loop). Update the step-5 runlog "remote branch deleted?" line accordingly if you can before the commit, else add a one-line correction commit `chore(bm): merge PR #139 — confirm branch deleted`.
  ```

- **`gh pr merge` HARD REFUSAL (per `.claude/rules/auto-phase.md` #9 + Junior #322 incident):** if `gh pr merge` exits non-zero, capture the VERBATIM stdout+stderr, write it into the Phase 9 output, and STOP IMMEDIATELY. Do NOT retry (Junior #322 retried 3× — forbidden). Do NOT `git reset`/`git push -f` (Junior #322 attempted this — forbidden, destructive). Do NOT hand-resolve a local merge and stage it (Junior #322 did this — forbidden). Do NOT improvise ANY alternate merge strategy (no `--admin`, no manual `git merge` + push). The advisor will diagnose. A single clean STOP with the verbatim error is the ONLY acceptable failure behavior.

- **Commit subjects:** post-merge runlog commit = `chore(bm): merge PR #139 complete — runlog COMPLETE entry`. Optional branch-delete confirm = `chore(bm): merge PR #139 — confirm branch deleted`. Both `chore(bm):` (BM attribution, NOT `chore(advisor)`).

- **`--merge` ONLY:** NEVER `--squash`/`--rebase`. `--delete-branch` removes remote `phase-v1-federation-inbound-b`; do NOT `git branch -D` the local branch (user's call).

- **NO force, NO -f, NO --hard, NO --no-verify, NO --admin, NO scope creep, NO retry.** Junior #322 breached the no-force and no-retry rules; this dispatch must not.

- **Phase 9 output:** return the real merge SHA + trunk position + L16 branch-delete result + a "Next: advisor runs Task 7 retro → user gate 6 → /brehon-phase-transition" line. You do NOT run the retro.

Brief commit body — mandatory file-class lessons: N/A (bm-task, merge-execute only). §2.3 hybrid search fired: PMD #115 L14 REVISED (post-merge runlog ordering to avoid bm-pr/bm-merge self-conflict), #335 (BM autonomy/file-ownership), L16 (post-merge branch-delete verify), no-destructive-defaults (Junior #322 breached). Authored on canonical brehon-fork checkout `governance-v0`. User Gate 5b confirmed 2026-05-20. Mirrors canonical sibling `.claude/PRPs/briefs/v1-ship-1-r2-bm-merge-2.md` (the L14-REVISED-post-merge dispatch shape that successfully merged PR #137 → governance-v0 in commit a278126315).
