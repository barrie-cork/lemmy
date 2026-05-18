# Brief — v1-ship-1-r2 bm-merge (execute-side only: merge PR #137 into governance-v0)

## 1. Role + dispatch line

`[role:bm-task] bm-merge v1-ship-1-r2 — execute merge PR #137 → governance-v0 (Phases 5-9 only)`

Dispatcher → `branch-manager` subagent. Execute **ONLY Phases 5-9** of
`.claude/commands/bm/bm-merge.md` (the execute-side). Phases 1-4
(gate-side: pre-merge read-only checks, pre-merge summary,
AskUserQuestion confirm) **ALREADY RAN INLINE in the advisor session**
per the L15 split (`.claude/commands/bm/bm-merge.md` "Two execution
modes" + `.claude/rules/auto-phase.md` invariant 6). **Do NOT re-run
Phases 1-4. Do NOT re-ask the user — user gate 5 is already CONFIRMED.**

## 2. Scope

Merge PR #137 (`phase-v1-ship-1` → `governance-v0`) via
`gh pr merge 137 --repo barrie-cork/lemmy --merge --delete-branch`,
then post-merge bookkeeping. This is the final irrevocable step of
v1-ship-1-r2 (the AGPL §13 source-disclosure surface deliverable).

**Advisor-side gate evidence (already verified inline — context only,
do NOT re-check):**
- PR #137 `mergeable: MERGEABLE`, `mergeStateStatus: CLEAN`, `state: OPEN`, base `governance-v0`, head `40ce10f84` (will be slightly ahead if advisor pushed more DQ commits — re-read head at Phase 5).
- Findings YAML `.claude/PRPs/reviews/pr-137-findings.yaml`: `critical.open == 0`, `major.open == 0`, `recommendation: approve`, zero `fix-in-pr` rows (1 finding cr-1 = nit/wont-fix, Brehon description-template convention).
- DQ `.claude/decision-queue.json`: `pending: []` (empty — #248 superseded by Task 6 at the L15 gate; #264 Phase-2 e2e PASS in resolved).
- /brehon-verify `.claude/PRPs/reports/v1-ship-1-r2-verify.md`: 3 stories 3✓ 0✗-phantom 0✗-regression.
- Phase-2 full e2e GREEN on the MERGED tip b941f5651: `test result: ok. 95 passed; 0 failed; 5 ignored` (5 ignored = pre-existing GH #42/#43/#45 baseline, NOT regressions; +5 vs pre-merge = v1-AD-e admin-HTML tests — coexistence proven).
- User Gate 5 (merge confirm): user replied "Confirm merge now" 2026-05-18.
- CodeRabbit: zero substantive findings; re-review of merge commit RESOLVED (mergeStateStatus flipped UNSTABLE→CLEAN).

Boundaries:
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `Cargo.{toml,lock}`, `rust-toolchain.toml` (BM file-ownership hard refusal).
- Do NOT use `--squash` — `--merge` ONLY (task-per-commit history is load-bearing for retros, per `phase-branch.md`).
- Do NOT merge into `main` — base is `governance-v0`.
- Do NOT re-run the Phase 1-4 gate checks or re-ask the user (L15: gate ran advisor-side; confirm already given).
- Do NOT send a Telegram ping (that is a separate `/bm-ping` ASK-first step).
- Commit ONLY `.claude/runlog/bm-runlog.md` (the Phase 8 append). The findings YAML stays gitignored (Phase 7 archival edit is local-only, NOT committed).

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — Phases **5-9 ONLY** (skip 1-4; they ran advisor-side per the L15 split documented in that file's "Two execution modes").
- `.claude/rules/branch-manager.md` — file-ownership boundaries + autonomy table + "advisor-side gate-only" row (the L15 fix).
- `.claude/rules/phase-branch.md` — `--merge` not `--squash`; base = `governance-v0`.
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` MANDATORY on every `gh pr` call.

## 4. Constraints

- **L14 git-sequence (LOAD-BEARING — explicit numbered order, do NOT improvise; per PMD #115 + `.claude/commands/bm/bm-merge.md` "L14 fix" + `.claude/rules/auto-phase.md` invariant 7):**

  ```
  1. git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0   (get on trunk FIRST, before the runlog Edit — the runlog lives on governance-v0)
  2. Edit .claude/runlog/bm-runlog.md — append the "## bm: merge — <ISO ts>" entry per Phase 8 template (PR #137, base←head governance-v0←phase-v1-ship-1, merge sha = TBD-fill-after-Phase-5, remote branch deleted? yes, trunk position, findings YAML archived path).
  3. git add .claude/runlog/bm-runlog.md
  4. git commit -m "chore(bm): merge PR #137 — runlog entry"
  5. git push origin governance-v0
  6. THEN (and only then): gh pr merge 137 --repo barrie-cork/lemmy --merge --delete-branch
  7. After merge returns 0: git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0 && git log -1 --oneline   (verify the merge commit is on trunk)
  8. Go back and fill the real merge sha into the runlog entry from step 2 IF it was left as TBD — append a one-line "merge sha: <sha>" correction commit `chore(bm): merge PR #137 — record merge sha` if needed (do NOT amend; new commit).
  9. L16 post-condition: git ls-remote origin refs/heads/phase-v1-ship-1 — if the ref STILL EXISTS (gh --delete-branch silently skipped), run ONCE: gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/phase-v1-ship-1. Re-check git ls-remote; if still present after the single DELETE attempt, STOP and surface to user (do NOT loop).
  ```

  **Rationale:** the runlog commit lands BEFORE the merge so the audit trail is durable even if the merge errors mid-flight. The BM Junior has historically checked-out-before-committing and lost the runlog Edit (c-1 session 2026-05-06). Do NOT `git checkout` or `git pull` between the runlog Edit (step 2) and its commit+push (steps 3-5) — checkout discards uncommitted worktree Edits. Step 1 deliberately gets onto trunk FIRST so steps 2-5 happen on the branch that owns the runlog.

- **Commit subject:** the runlog commit MUST be exactly `chore(bm): merge PR #137 — runlog entry` (BM attribution; NOT `chore(advisor)`). Any sha-correction follow-up = `chore(bm): merge PR #137 — record merge sha`.

- **`gh pr merge` hard refusal (per `.claude/rules/auto-phase.md` hard refusal #9):** if `gh pr merge` exits non-zero, surface the verbatim error and STOP. Do NOT retry. Do NOT improvise an alternate merge strategy (no manual `git merge` + push, no `--admin`, no second attempt). The advisor will diagnose.

- **`--merge` ONLY (per `phase-branch.md`):** NEVER `--squash`, NEVER `--rebase`. The 113-commit task-per-commit history is load-bearing for the v1-ship-1-r2 retro. `--delete-branch` removes the remote `phase-v1-ship-1`; local branch survives (do NOT `git branch -D` it — that's the user's call).

- **Phase 7 findings YAML (Phase 7 of the script):** add `merged_at: <ISO>`, `merge_commit: <sha>`, `final_recommendation: approve` to `.claude/PRPs/reviews/pr-137-findings.yaml`. This file is GITIGNORED — edit it but do NOT `git add`/commit it (it's archival in the local sense only, per the script Phase 7 note).

- **Phase 9 output:** return the merge SHA + trunk position + the L16 branch-delete verification result + a "Next suggested" line (the advisor will then run Task 7 retro → user gate 6 → /brehon-phase-transition; you do NOT run those).

- **NO force, NO --no-verify, NO --admin, NO scope creep.** If the merge state is anything other than the verified CLEAN/MERGEABLE (e.g. a race flipped it to BEHIND/DIRTY between the advisor gate and your Phase 5), STOP and surface — do NOT improvise (the advisor will re-gate).

Brief commit body — mandatory file-class lessons: N/A (bm-task, no impl files; merge-execute only). §2.3 hybrid search fired: PMD #115 (L14 git-sequence runlog-before-merge), #335 (BM autonomy/file-ownership), L15/L16 from v1-SL-c-1 retro (gate-side inline advisor / post-merge branch-delete verify). L14 + L16 constraints injected into §4 as the load-bearing rules. Authored on phase-v1-ship-1 lane worktree. User Gate 5 confirmed 2026-05-18; this brief carries Phases 5-9 ONLY per the L15 split.
