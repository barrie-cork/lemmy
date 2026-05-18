# Brief — v1-ship-1-r2 bm-merge RE-DISPATCH (execute merge PR #137, NO L14 runlog-on-trunk step)

## 1. Role + dispatch line

`[role:bm-task] bm-merge v1-ship-1-r2 RETRY — execute PR #137 merge, NO runlog-on-trunk (L14 self-conflict fixed)`

Dispatcher → `branch-manager` subagent. This is a **RE-DISPATCH** after the
first bm-merge attempt (Junior #322) was BLOCKED by an L14-self-inflicted
`bm-runlog.md` conflict. Execute **ONLY the merge + post-merge verify**
(a reduced Phase 5-9). Phases 1-4 (gate-side) already ran inline
advisor-side; user gate 5 CONFIRMED. **The runlog entry is ALREADY on
phase-v1-ship-1** (advisor resolved the conflict in merge commit
`33edf7848` — the `## bm: merge ATTEMPTED — BLOCKED` block). **You MUST
NOT write a new runlog entry to governance-v0 before the merge — that
exact step caused the self-conflict that blocked Junior #322.**

## 2. Scope

Merge PR #137 (`phase-v1-ship-1` → `governance-v0`) via
`gh pr merge 137 --repo barrie-cork/lemmy --merge --delete-branch`,
then post-merge trunk verify + L16 branch-delete verify. **NO
runlog-commit-before-merge step** (the runlog history is already merged
into the phase branch via the conflict-resolution merge `33edf7848`;
re-committing it to governance-v0 first is what self-conflicts).

**Why the first attempt (Junior #322) failed (context — do NOT repeat):**
The original bm-merge brief carried the L14 sequence "Edit runlog → commit
to governance-v0 → push → THEN gh pr merge". But bm-pr had ALREADY written
a `## bm: PR opened #137` runlog entry on phase-v1-ship-1 (commit
`d0e52fdb4`). When Junior #322 committed its `## bm: merge` entry to
governance-v0 (`32548e55f`) and tried to merge, `bm-runlog.md` conflicted
between the two branches → PR went DIRTY/CONFLICTING → merge impossible.
The advisor has since resolved that conflict on phase-v1-ship-1 (merge
`33edf7848`, additive union + corrected the false `32548e55f` entry to a
truthful ATTEMPTED-BLOCKED record). PR #137 is now MERGEABLE again.

**Advisor-side gate evidence (already verified — context only, do NOT
re-check):**
- PR #137 `mergeable: MERGEABLE`, base `governance-v0`, head `33edf7848` (the runlog-conflict-resolution merge).
- Findings YAML `.claude/PRPs/reviews/pr-137-findings.yaml`: critical.open 0, major.open 0, recommendation approve, 0 fix-in-pr (1 nit/wont-fix).
- DQ `.claude/decision-queue.json`: pending [] (will be re-confirmed advisor-side; the bm-merge brief's earlier #248/#264 state stands; a kind:log on the L14 flaw may be added by advisor — not your concern).
- /brehon-verify: 3 stories 3✓ 0✗.
- Phase-2 full e2e GREEN on the pre-runlog-fix merged tip b941f5651: 95 passed / 0 failed / 5 baseline-ignored. The runlog-conflict-resolution merge `33edf7848` changed ONLY `.claude/runlog/bm-runlog.md` (zero code delta vs b941f5651 — verify with `git diff --stat b941f5651 33edf7848 -- crates/ migrations/ Cargo.lock` == empty if you wish, but this is advisor-verified).
- User Gate 5 (merge confirm): CONFIRMED 2026-05-18.

Boundaries:
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `Cargo.{toml,lock}`, `rust-toolchain.toml`.
- Do NOT use `--squash`/`--rebase` — `--merge` ONLY (113+commit task-per-commit history load-bearing).
- Do NOT merge into `main` — base is `governance-v0`.
- **Do NOT write/commit a runlog entry to governance-v0 (or anywhere) BEFORE the merge.** The runlog "bm: merge ATTEMPTED — BLOCKED" entry is already merged via `33edf7848`. After the SUCCESSFUL merge you MAY append a short "## bm: merge COMPLETE — PR #137 sha <sha>" follow-up on governance-v0 (see §4 step 5) — but ONLY post-merge, never pre-merge.
- Do NOT re-run Phase 1-4 gate checks or re-ask the user (L15: ran advisor-side; confirm given).
- Do NOT retry `gh pr merge` if it fails (hard refusal — see §4).
- Do NOT send a Telegram ping.
- Do NOT `git push -f` anything, ever (Junior #322 attempted this on protected trunk — it is a hard refusal).

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — Phase 5 (execute), Phase 6 (post-merge trunk verify), Phase 9 (output). **SKIP Phase 7 findings-YAML archival edit is OPTIONAL (gitignored, local-only — do it if trivial, skip if not). SKIP the Phase 8 runlog-BEFORE-merge ordering entirely — that is the L14 step that caused the failure; the post-merge follow-up in §4 step 5 replaces it.**
- `.claude/rules/branch-manager.md` — file-ownership + autonomy + "advisor-side gate-only" (L15).
- `.claude/rules/phase-branch.md` — `--merge` not `--squash`; base `governance-v0`.
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` MANDATORY.
- `.claude/rules/no-destructive-defaults.md` — NEVER `--force`/`-f`/`--hard` (Junior #322 breached this).

## 4. Constraints

- **Reduced git sequence (explicit numbered order — the L14 runlog-on-trunk step is DELIBERATELY REMOVED):**

  ```
  1. git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0   (get on trunk; verify clean)
  2. Re-verify PR #137 is still MERGEABLE: gh pr view 137 --repo barrie-cork/lemmy --json mergeable,mergeStateStatus,state  → mergeable MUST be "MERGEABLE" and state "OPEN". If mergeStateStatus is UNSTABLE because a check is PENDING (CodeRabbit re-review), that is NON-blocking per the prior treat-as-clean user decision — proceed. If mergeable is "CONFLICTING" or state is not "OPEN" → STOP and surface (do NOT improvise).
  3. gh pr merge 137 --repo barrie-cork/lemmy --merge --delete-branch
  4. Wait for it to return. If exit non-zero → HARD REFUSAL (see below). If exit 0 → continue.
  5. POST-MERGE ONLY (now that the merge succeeded): git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0. Edit .claude/runlog/bm-runlog.md — append a NEW short block:
     "## bm: merge COMPLETE — <ISO ts>\n\n- **PR:** #137\n- **merge sha:** <the real sha from gh pr view 137 --json mergeCommit -q .mergeCommit.oid>\n- **trunk position:** <git rev-parse --short governance-v0> (<short subject>)\n- **remote branch deleted?** <yes if step 6 confirms, else no — see step 6>\n- **note:** completes the 'bm: merge ATTEMPTED — BLOCKED' entry above (Junior #322 L14 self-conflict, advisor-resolved in merge 33edf7848)."
     Then: git add .claude/runlog/bm-runlog.md && git commit -m "chore(bm): merge PR #137 complete — runlog COMPLETE entry" && git push origin governance-v0.
     (This runlog write is POST-merge, so it cannot conflict with the phase branch — the phase branch no longer exists after --delete-branch.)
  6. L16 post-condition: git ls-remote origin refs/heads/phase-v1-ship-1 — if STILL PRESENT (gh --delete-branch silently skipped), run ONCE: gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/phase-v1-ship-1. Re-check git ls-remote; if still present after the single DELETE attempt, note "branch-delete needs manual follow-up" in the Phase 9 output and STOP (do NOT loop). Update the step-5 runlog "remote branch deleted?" line accordingly if you can before the commit, else add a one-line correction commit `chore(bm): merge PR #137 — confirm branch deleted`.
  ```

- **`gh pr merge` HARD REFUSAL (per `.claude/rules/auto-phase.md` #9 + Junior #322 incident):** if `gh pr merge` exits non-zero, capture the VERBATIM stdout+stderr, write it into the Phase 9 output, and STOP IMMEDIATELY. Do NOT retry (Junior #322 retried 3× — forbidden). Do NOT `git reset`/`git push -f` (Junior #322 attempted this — forbidden, destructive). Do NOT hand-resolve a local merge and stage it (Junior #322 did this — forbidden). Do NOT improvise ANY alternate merge strategy (no `--admin`, no manual `git merge` + push). The advisor will diagnose. A single clean STOP with the verbatim error is the ONLY acceptable failure behavior.

- **Commit subjects:** post-merge runlog commit = `chore(bm): merge PR #137 complete — runlog COMPLETE entry`. Optional branch-delete confirm = `chore(bm): merge PR #137 — confirm branch deleted`. Both `chore(bm):` (BM attribution, NOT `chore(advisor)`).

- **`--merge` ONLY:** NEVER `--squash`/`--rebase`. `--delete-branch` removes remote `phase-v1-ship-1`; do NOT `git branch -D` the local branch (user's call).

- **NO force, NO -f, NO --hard, NO --no-verify, NO --admin, NO scope creep, NO retry.** Junior #322 breached the no-force and no-retry rules; this re-dispatch must not. If anything is not the clean MERGEABLE state, STOP + surface.

- **Phase 9 output:** return the real merge SHA + trunk position + L16 branch-delete result + a "Next: advisor runs Task 7 retro → user gate 6 → /brehon-phase-transition" line. You do NOT run the retro.

Brief commit body — mandatory file-class lessons: N/A (bm-task, merge-execute only). §2.3 hybrid search fired: PMD #115 (L14 — DELIBERATELY MODIFIED here: runlog-on-trunk-before-merge REMOVED because it self-conflicts when bm-pr also wrote a phase-branch runlog entry; this is the recovery brief for that exact incident), #335 (BM autonomy/file-ownership), L16 (post-merge branch-delete verify), no-destructive-defaults (Junior #322 breached). Re-dispatch after Junior #322 BLOCKED by L14 self-conflict (advisor-resolved in merge 33edf7848). Authored on phase-v1-ship-1 lane worktree. User Gate 5 confirmed 2026-05-18.
