# Brief — v1-rls-r1 bm-merge (execute merge PR #140, L14 REVISED post-merge runlog)

## 1. Role + dispatch line

`[role:bm-task] bm-merge v1-rls-r1 — execute PR #140 merge + post-merge runlog COMPLETE`

Dispatcher → `branch-manager` subagent. Execute **ONLY the merge + post-merge runlog + L16 branch-delete verify** (a reduced Phase 5-9). Phases 1-4 (gate-side: findings counters, mergeStateStatus, CR re-review SUCCESS, user gate-5 confirm) already ran inline advisor-side per L15. **You MUST NOT write a runlog entry to governance-v0 BEFORE the merge** — that exact step caused the L14 self-conflict that BLOCKED Junior #322 on v1-ship-1-r2. The L14 ordering is **POST-merge** per `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` (REVISED 2026-05-18).

## 2. Scope

Merge PR #140 (`phase-v1-rls-r1` → `governance-v0`) via `gh pr merge 140 --repo barrie-cork/lemmy --merge --delete-branch`, then POST-merge runlog write on `bm-runlog.md` (governance-v0) with the real merge SHA, then L16 branch-delete verify.

**PR:** `#140` (phase-v1-rls-r1 → governance-v0)
**Phase branch tip:** `be93c4460` (fix-impl-2 cr-9 commit + DQ #323 mutation)
**Mergeable:** CLEAN / MERGEABLE
**Status checks:** CodeRabbit = SUCCESS at 2026-05-21T10:09:42Z (re-reviewed fix-impl-2 tip, zero new actionable findings)

**Advisor-side gate evidence (already verified — context only, do NOT re-check):**

- PR #140 `mergeable: MERGEABLE`, `mergeStateStatus: CLEAN`, base `governance-v0`, head `be93c4460`.
- CR statusCheckRollup: CodeRabbit `state: SUCCESS` (the re-review on tip be93c4460 was silent — 0 new actionable comments).
- Findings YAML `.claude/PRPs/reviews/pr-140-findings.yaml` (gitignored runtime artifact): critical.open 0, major.open 0 (cp-1 + cr-2 + cr-9 all addressed across fix-impl-1 commit cc1590904 + fix-impl-2 commit be93c4460), medium.open 0 (cp-2/cp-3/cp-5 doc-consistency addressed in fix-impl-1), low.open 0, nit.open 0. Three findings carry-resolution-as-PR-comment: cr-1 (rebut, plan-approval audit trail), cr-5 + cr-6 (wont-fix, briefs are historical artifacts).
- DQ `.claude/decision-queue.json`: pending []. All v1-rls-r1 validate-pending entries (DQ #307-#323 minus the kind:log retros from Tasks 6/8 daemon PMD-fallback) result:pass and resolved.
- carry-forward: none (zero findings deferred to a future sub-phase).
- /brehon-verify: not re-run; the dogfood report at `.claude/PRPs/reports/v1-rls-r1-dogfood-2026-05-21.md` is the integration-test equivalent for this meta-work-only phase; 4 dogfood probes pass post-fix-impl-2.
- User Gate 5 (merge confirm): CONFIRMED 2026-05-21 via AskUserQuestion "Approve merge — dispatch bm-merge Junior".

Boundaries:

- Do NOT touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `Cargo.{toml,lock}`, `rust-toolchain.toml`.
- Do NOT use `--squash`/`--rebase` — `--merge` ONLY (task-per-commit history load-bearing for retros).
- Do NOT merge into `main` — base is `governance-v0`.
- **Do NOT write/commit a runlog entry to governance-v0 (or anywhere) BEFORE the merge.** Post-merge ONLY (see §4 step 5).
- Do NOT re-run Phase 1-4 gate checks or re-ask the user (L15: ran advisor-side; confirm given 2026-05-21).
- Do NOT retry `gh pr merge` if it fails (hard refusal — see §4).
- Do NOT send a Telegram ping.
- Do NOT `git push -f` anything, ever (v1-ship-1-r2 Junior #322 attempted this on protected trunk — hard refusal).

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — Phase 5 (execute), Phase 6 (post-merge trunk verify), Phase 9 (output). **SKIP the Phase 8 runlog-BEFORE-merge ordering entirely** — that is the L14 OLD-ORDER step that caused the v1-ship-1-r2 self-conflict; the post-merge follow-up in §4 step 5 is the L14 REVISED replacement.
- `.claude/rules/branch-manager.md` — file-ownership + autonomy + "advisor-side gate-only" (L15).
- `.claude/rules/phase-branch.md` — `--merge` not `--squash`; base `governance-v0`.
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` MANDATORY on EVERY gh command.
- `.claude/rules/no-destructive-defaults.md` — NEVER `--force`/`-f`/`--hard`/`--no-verify` (Junior #322 breached this).
- `.claude/rules/auto-phase.md` "Hard refusals" #9 — `gh pr merge` non-zero = STOP, no retry, no alternate strategy. L14 REVISED (post-merge runlog ordering). L16 (post-condition branch-delete verify).
- `.claude/lessons/feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` — the canonical rationale for the post-merge ordering used here.

## 4. Constraints

- **Reduced git sequence (explicit numbered order — L14 REVISED post-merge):**

  ```text
  1. git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0   (get on trunk; verify clean working tree)
  2. Re-verify PR #140 is still MERGEABLE: gh pr view 140 --repo barrie-cork/lemmy --json mergeable,mergeStateStatus,state  → mergeable MUST be "MERGEABLE" and state "OPEN". If mergeStateStatus is UNSTABLE because a check is PENDING (CodeRabbit re-review), that is NON-blocking per the prior treat-as-clean user decision — proceed. If mergeable is "CONFLICTING" or state is not "OPEN" → STOP and surface (do NOT improvise).
  3. gh pr merge 140 --repo barrie-cork/lemmy --merge --delete-branch
  4. Wait for it to return. If exit non-zero → HARD REFUSAL (see below). If exit 0 → continue.
  5. POST-MERGE ONLY (now that the merge succeeded): git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0. Capture the real merge SHA: REAL_SHA=$(gh pr view 140 --repo barrie-cork/lemmy --json mergeCommit -q .mergeCommit.oid). Edit .claude/runlog/bm-runlog.md — append a NEW short block at the END of the file:

     ## bm: merge COMPLETE — <UTC ISO timestamp>

     - **PR:** #140
     - **merge sha:** <REAL_SHA from the gh pr view above>
     - **trunk position:** <git rev-parse --short governance-v0> (<short subject>)
     - **remote branch deleted?** <yes if step 6 confirms, else no — see step 6>
     - **CR triage summary:** 11 fix-in-pr addressed across fix-impl-1 (commit cc1590904: pmd-canonical-guard.sh abspath+show-toplevel + 3 doc-consistency batch + dogfood alignment + retro count + markdown lint) + fix-impl-2 (commit be93c4460: cr-9 anchored-relative-path helper). 3 carry-resolution-as-PR-comment: cr-1 rebut (DQ #297 schema ratified at plan-approval gate), cr-5 + cr-6 wont-fix (briefs are historical authoring records). 0 carry-forward.
     - **DQs:** all v1-rls-r1 validate-pending entries (DQ #307-#323) result:pass; pending: 0.

     Then: git add .claude/runlog/bm-runlog.md && git commit -m "chore(bm): merge PR #140 complete — runlog COMPLETE entry" && git push origin governance-v0.
     (This runlog write is POST-merge, so it cannot conflict with the phase branch — the phase branch no longer exists after --delete-branch. bm-runlog.md has `merge=union` per .gitattributes; even if a concurrent governance-v0 commit lands, union merge resolves cleanly.)
  6. L16 post-condition: git ls-remote origin refs/heads/phase-v1-rls-r1 — if STILL PRESENT (gh --delete-branch silently skipped), run ONCE: gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/phase-v1-rls-r1. Re-check git ls-remote; if still present after the single DELETE attempt, note "branch-delete needs manual follow-up" in the Phase 9 output and STOP (do NOT loop). Update the step-5 runlog "remote branch deleted?" line accordingly if you can before the commit, else add a one-line correction commit `chore(bm): merge PR #140 — confirm branch deleted`.
  ```

- **`gh pr merge` HARD REFUSAL (per `.claude/rules/auto-phase.md` #9 + Junior #322 incident):** if `gh pr merge` exits non-zero, capture the VERBATIM stdout+stderr, write it into the Phase 9 output, and STOP IMMEDIATELY. Do NOT retry (Junior #322 retried 3× — forbidden). Do NOT `git reset`/`git push -f` (Junior #322 attempted this — forbidden, destructive). Do NOT hand-resolve a local merge and stage it (Junior #322 did this — forbidden). Do NOT improvise ANY alternate merge strategy (no `--admin`, no manual `git merge` + push). The advisor will diagnose. A single clean STOP with the verbatim error is the ONLY acceptable failure behavior.

- **Commit subjects:** post-merge runlog commit = `chore(bm): merge PR #140 complete — runlog COMPLETE entry`. Optional branch-delete confirm = `chore(bm): merge PR #140 — confirm branch deleted`. Both `chore(bm):` (BM attribution, NOT `chore(advisor)`).

- **`--merge` ONLY:** NEVER `--squash`/`--rebase`. `--delete-branch` removes remote `phase-v1-rls-r1`; do NOT `git branch -D` the local branch (user's call).

- **NO force, NO -f, NO --hard, NO --no-verify, NO --admin, NO scope creep, NO retry.** Junior #322 breached the no-force and no-retry rules; this dispatch must not.

- **Phase 9 output:** return the real merge SHA + trunk position + L16 branch-delete result + a "Next: advisor confirms phase-transition + lane worktree teardown" line. You do NOT run /brehon-phase-transition.

Brief commit body — mandatory file-class lessons: N/A (bm-task, merge-execute only). §2.3 hybrid search fired: PMD #115 L14 REVISED (post-merge runlog ordering to avoid bm-pr/bm-merge self-conflict), #335 (BM autonomy/file-ownership), L16 (post-merge branch-delete verify), no-destructive-defaults (Junior #322 breached). Authored on lane-dedicated brehon-fork-rls-r1 checkout `phase-v1-rls-r1`. User Gate 5 confirmed 2026-05-21. Mirrors canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-b-bm-merge-1.md` (the L14-REVISED-post-merge dispatch shape that successfully merged PR #139 → governance-v0 in commit 413ef5899).
