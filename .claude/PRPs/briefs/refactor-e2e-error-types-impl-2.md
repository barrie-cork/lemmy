---
phase: chore/refactor-e2e-error-types
role: impl-task
task: refactor-continuation
brief_n: 2
authored: 2026-05-15
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_findings: 3.E.1 (CRIT) + 3.E.3 (MAJ) + 3.E.4 (MAJ) — 3.E.2 DEFERRED (DQ #222)
continuation_of: "Junior #268 (delivered Passes 1+3+4; cherry-picked to chore tip f258824b5). This task ONLY adds the ~20 pre-existing e2e.rs lint fixes per DQ #221."
parent_phase_tip: "governance-v0 HEAD (ef393c7bd); worker cherry-picks chore code commit f258824b5 first"
note: "Dispatched with base_branch=governance-v0 (NOT chore/* — L3: daemon cannot resolve chore refs, proven PR-4/5/6). The worker's FIRST action is to cherry-pick the chore-branch code commit f258824b5 (Passes 1+3+4 + accepted create_report.rs fix) to establish the baseline, THEN fix the lints on top. The advisor cherry-picks the worker's resulting code commit back onto chore/refactor-e2e-error-types."
---

# [role:impl-task] chore/refactor-e2e-error-types — continuation: fix ~20 pre-existing e2e.rs clippy lints (DQ #221) — see .claude/PRPs/briefs/refactor-e2e-error-types-impl-2.md

## §1 Role + dispatch

`[role:impl-task] chore/refactor-e2e-error-types continuation — fix ~20 pre-existing e2e.rs clippy lints so Gate 2 passes (DQ #221 fix-all-e2e-lints; P2/3.E.2 deferred per DQ #222)`

## §2 Scope

### §2.0 What already happened (read this first)

Junior #268 ran the original PR-1 brief
(`.claude/PRPs/briefs/refactor-e2e-error-types-impl.md`) and **completed
Passes 1, 3, 4** but hit **two blockers** and exited `done`:

- **Pass 1 (audit 3.E.1, CRIT)** — error-type unification Case C → Case A
  across all Brehon-authored `e2e.rs` test fns + fixtures helpers to
  uniform `LemmyResult`. **DONE.**
- **Pass 3 (audit 3.E.3, MAJ)** — split `phase1_migrations_round_trip`
  into 3 independent test fns. **DONE** (net +2 async fns, 69→71).
- **Pass 4 (audit 3.E.4, MAJ)** — `PHASE_1_MIGRATION_COUNT` →
  named `MIGRATIONS_TO_REVERT_PHASE_1`. **DONE.**
- **Incidental** — `create_report.rs:168`
  `#[allow(clippy::too_many_arguments)]` → `#[expect(...)]` (1 line,
  zero behavior change). **DONE** — user-accepted scope exception
  (DQ #220). Keep it.
- **Pass 2 (audit 3.E.2, CRIT — fixtures dedup)** — **NOT done.**
  **DEFERRED** to a separate post-gate lane per the user's DQ #222
  decision (`defer-pass2-separate-task`). **THIS TASK MUST NOT TOUCH
  PASS 2.** The 4 fixtures modules (`v1_jm_b_fixtures`,
  `v1_jm_e_fixtures`, `v1_sl_b_fixtures`, `v1_sl_c_fixtures`) stay
  exactly as they are on the chore branch. Do NOT consolidate, do NOT
  extract `governance_test_helpers`, do NOT refactor those modules.

All of #268's done work is captured in **one squashed code commit on
`chore/refactor-e2e-error-types`: `f258824b5`** (e2e.rs + the 1-line
create_report.rs fix only). That commit's parent is `ef393c7bd` =
current `governance-v0` HEAD.

### §2.1 This task's job (the ONLY job)

Per the user's **DQ #221 decision = `fix-all-e2e-lints`**: on top of
#268's committed work, **fix all ~20 PRE-EXISTING clippy lints in
`crates/server/tests/e2e.rs`** so that **Gate 2
(`cargo-clippy.sh --workspace --features full --tests --no-deps --
-D warnings`) genuinely exits 0**. These lints pre-exist on
`governance-v0` HEAD — they were NOT introduced by #268's refactor;
the original brief's Gate 2 was a defect (specified a gate that was
never green on this file). The user authorised closing this
pre-existing debt in the same PR because PR-1 already rewrites this
exact file and Gate 2 must hold for the LAST refactor lane.

### §2.2 Verbatim DQ #221 + #222 contract (the authoritative spec)

> **DQ #221 answer (advisor, user-relayed 2026-05-15):**
> fix-all-e2e-lints. A continuation impl-task fixes ALL pre-existing
> clippy issues in `crates/server/tests/e2e.rs` so Gate 2 genuinely
> exits 0 — including the structural `tests_outside_test_module`
> (wrap in `#[cfg(test)] mod` where required), `as_conversions`
> (TryFrom/try_into, never `as`), `allow_attributes` (`#[allow]` →
> `#[expect]` with a reason ONLY where the lint is genuinely
> intended; otherwise remove the allow and fix the underlying lint),
> `ref_patterns`, `items_after_statements`, `map_err_ignore`
> (`|_|` → `|_e|`), `needless_ok_wrapping`, `indexing_slicing`
> (`.get()`/`.first()` + `?` or expect-with-message),
> `clippy::expect_used`, and the doc lints. NO blanket
> module/crate-level `#![allow]` masking (that is the
> `#[allow]`-spam anti-pattern the brief forbids — fix each lint
> properly). The `clippy::indexing_slicing` in
> `reputation_snapshot.rs:856` is OUT OF SCOPE (different crate); if
> it blocks the `--workspace` clippy run, scope the clippy
> invocation to the e2e test target (e.g. `--test e2e`) or raise a
> follow-up DQ noting it pre-exists and is unrelated — do NOT touch
> that file. ZERO behavior change in any test (no removed
> assertions, no changed seed data, no `#[ignore]`). After Gate 2
> is genuinely green, run Gate 3 (the load-bearing full e2e suite
> per brief §2.7).
>
> **DQ #222 answer (advisor, user-relayed 2026-05-15):**
> defer-pass2-separate-task. Pass 2 / audit 3.E.2 is DEFERRED to its
> own dedicated lane. PR-1 ships covering audit 3.E.1 + 3.E.3 +
> 3.E.4. The continuation impl-task does NOT attempt P2 — the 4
> fixtures modules stay as-is.

If anything below contradicts this blockquote, **the blockquote
wins** — stop and raise a `kind: "blocker"` DQ citing this section.

### §2.3 The ~20 pre-existing lints to fix (from #268 DQ #221 context)

The exact lint inventory #268 reported (line numbers are approximate
and PRE-#268-refactor — verify actual positions in the
cherry-picked file; the refactor moved lines):

1. `clippy::tests_outside_test_module` — fires for the Brehon test
   fns / fixtures not inside a `#[cfg(test)] mod`. **Structural** —
   this is the biggest one. Wrap the affected fns/modules in
   `#[cfg(test)] mod` blocks **without changing test discovery or
   behavior**. If wrapping would change how the integration test
   harness discovers `#[tokio::test]` fns (these are integration
   tests in `tests/e2e.rs`, top-level — `tests_outside_test_module`
   may need a targeted `#[expect(clippy::tests_outside_test_module,
   reason = "integration test file; tests are top-level by Cargo
   convention")]` rather than a structural wrap, because Cargo
   integration tests live at crate root, not in a `mod tests`).
   **Use judgment: prefer the structural fix; if it changes
   discovery/behavior, the `#[expect]` with an explicit reason is
   the correct fix, NOT `#[allow]`-spam.** This single lint may be
   the dominant edit surface.
2. `clippy::allow_attributes` (~old line 1586, `#[allow(dead_code)]`
   and any others) — `#[allow]` → `#[expect]` with `reason = "..."`.
3. `clippy::ref_patterns` (~old 6759).
4. `clippy::as_conversions` (~old 2815, 12679) — replace `as` with
   `TryFrom`/`try_into()` + proper error handling, OR
   `u32::try_from(x).expect("...")` where the cast is provably safe
   in test context.
5. doc lints (~old 243-245) — `doc_list_item_without_indentation` /
   similar; reflow the doc comment.
6. `clippy::items_after_statements` (~old 447) — hoist the item
   above the statements, or scope it.
7. `clippy::map_err_ignore` (~old 539) — `|_|` → `|_e|`.
8. `clippy::needless_ok_wrapping` (~old 839) — return the value
   directly instead of `Ok(...)` where the signature allows.
9. `clippy::indexing_slicing` (~old 1008) — `.get(i)` + `?`/expect,
   or `.first()`.
10. `clippy::expect_used` (~old 1801) — these are tests, so
    `expect_used` is usually fine; this likely needs an
    `#[expect(clippy::expect_used, reason = "test assertion")]` at
    the narrowest scope (fn-level, not module/crate). NEVER a
    blanket allow.

This is #268's reported set; **the authoritative source is Gate 2's
actual output** — run Gate 2, read EVERY error, fix EVERY one. Do
not assume the list above is exhaustive or the line numbers current.

### §2.4 Out of scope (HARD)

- **Pass 2 / audit 3.E.2 (fixtures dedup)** — DEFERRED (DQ #222).
  Do not touch the 4 fixtures modules' structure.
- **`reputation_snapshot.rs:856`** `indexing_slicing` — different
  crate; if it blocks `--workspace --tests` clippy, scope Gate 2 to
  `--test e2e` (see §4) or raise a follow-up DQ. Do NOT edit that
  file.
- **Any file other than `crates/server/tests/e2e.rs`** — the
  `create_report.rs` 1-line fix is ALREADY in `f258824b5`; do not
  re-touch it, do not touch anything else.
- No new test cases. No removed assertions. No changed seed data.
  No `#[ignore]`. No blanket `#![allow]`.

## §3 Required reading (read BEFORE any edit)

- This brief in full (esp §2.2 verbatim DQ contract + §2.4 out of scope).
- `.claude/PRPs/briefs/refactor-e2e-error-types-impl.md` — the
  original PR-1 brief (context on Passes 1/3/4 already done; §4 gate
  command exact form).
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` —
  **MANDATORY**: anchor-Edit discipline on the 8945-line e2e.rs
  (surgical `old_string`, NO `replace_all`, NO full-file Read,
  ~10-edit batches + local commit + cargo check between batches).
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A
  canonical (Pass 1 already applied this; do NOT regress it while
  fixing lints).
- `.claude/lessons/feedback_clippy_test_style.md` — Lemmy workspace
  clippy conventions (`LemmyResult<()>` + `?`, no unwrap/expect/allow
  spam, the workspace lint set).
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — re-run
  clippy after each fix batch; one fix can surface/mask another.
- `.claude/lessons/feedback_clippy_map_err_ignore_pattern_rename.md`
  (if present) — `|_|` → `|_e|` recipe.
- `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` —
  local cargo-check before push is mandatory.
- `.claude/rules/decision-queue.md` — DQ schema + mid-task push
  discipline (commit + push the DQ entry immediately so the advisor
  sees it).

## §4 Constraints

- **Branch / base:** dispatched `base_branch=governance-v0`. **FIRST
  ACTION (run these EXACT commands before anything else):**

  ```bash
  git fetch origin chore/refactor-e2e-error-types
  git cherry-pick f258824b5
  # f258824b5 = the chore-branch code commit: Passes 1+3+4 + the
  # accepted 1-line create_report.rs fix. Its parent is governance-v0
  # HEAD (ef393c7bd) so the pick applies cleanly with NO conflicts.
  # If git reports the commit object is missing, run:
  #   git fetch origin 'refs/heads/chore/refactor-e2e-error-types:refs/remotes/origin/chore/refactor-e2e-error-types'
  # then retry the cherry-pick by SHA.
  ```

  Then verify the baseline BEFORE touching any lint:

  ```bash
  git diff --stat $(git merge-base HEAD origin/governance-v0)...HEAD
  # MUST show ONLY:
  #   crates/api/api_crud/src/governance/create_report.rs   |  2 +-
  #   crates/server/tests/e2e.rs                             | ... 
  grep -c '^async fn ' crates/server/tests/e2e.rs   # MUST be 71
  ```

  If the diff shows any other file or the count is not 71, **STOP
  and raise a `kind: "blocker"` DQ** — the cherry-pick is wrong; do
  not proceed. THEN do the lint fixes on top.
- **Files:** ONLY `crates/server/tests/e2e.rs` for the lint work.
  The `create_report.rs` change is inherited from the cherry-pick —
  do NOT modify it further. NO other files.
- **No behavior change:** every existing test must still pass with
  the same assertions. Lint fixes are mechanical/structural ONLY. If
  a lint fix would change test behavior or you cannot fix a lint
  cleanly without `#[allow]`-spam, **STOP and file a `kind:
  "blocker"` DQ** (commit + push it immediately per
  `decision-queue.md` mid-task visibility) — do NOT `#[ignore]`, do
  NOT delete the assertion, do NOT blanket-allow.
- **Final test count must still equal pre-refactor + 2** (= 71
  async fns; #268 already achieved this — do not change it). Verify
  `grep -c "^async fn " crates/server/tests/e2e.rs` == 71.
- **No `#[ignore]`. No blanket `#![allow]` at module/crate level.**
  Narrow `#[expect(clippy::X, reason = "...")]` at the tightest
  possible scope is acceptable ONLY where the lint is genuinely
  intended (e.g. `expect_used` / `tests_outside_test_module` in an
  integration-test file) — never as a shortcut to skip a fixable
  lint.
- **Anchor-Edit discipline (mandatory per
  `feedback_junior_worker_e2e_edit_hang.md`):** every Edit on this
  8945-line file uses surgical `old_string` (3-5 line context).
  **NO `replace_all: true`. NO full-file Read.** Batch ~10 edits,
  then local `cargo check` + local commit, repeat. (#268 proved the
  anchor-Edit discipline works on this file — keep it.)
- **Pre-push gates (mandatory; same as original brief §4):**

  ```bash
  # Gate 1: cargo check
  bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/refactor-e2e-c2-precheck.log 2>&1
  status=$?; tail -20 .claude/PRPs/debug/refactor-e2e-c2-precheck.log
  [ $status -eq 0 ] || exit $status

  # Gate 2: clippy --tests  (THE gate this task exists to make green)
  bash scripts/brehon/cargo-clippy.sh --workspace --features full --tests --no-deps -- -D warnings > .claude/PRPs/debug/refactor-e2e-c2-clippy.log 2>&1
  status=$?; tail -40 .claude/PRPs/debug/refactor-e2e-c2-clippy.log
  # If Gate 2 fails ONLY on reputation_snapshot.rs:856 (different crate,
  # out of scope per DQ #221) and e2e.rs is clean, re-run scoped to the
  # e2e target and use that as the gate, and raise a kind:"log" DQ noting
  # the reputation_snapshot.rs pre-existing lint is unrelated + out of scope:
  #   bash scripts/brehon/cargo-clippy.sh --workspace --features full --test e2e --no-deps -- -D warnings
  [ $status -eq 0 ] || exit $status

  # Gate 3: e2e tests (FULL run, ~26 min) — load-bearing signal (brief §2.7)
  bash scripts/brehon/cargo-test.sh --workspace --features full --test e2e > .claude/PRPs/debug/refactor-e2e-c2-tests.log 2>&1
  status=$?; tail -40 .claude/PRPs/debug/refactor-e2e-c2-tests.log
  [ $status -eq 0 ] || exit $status
  ```

- **Commit:** squash to ONE final commit. Subject:
  `chore(test): fix pre-existing e2e.rs clippy lints for Gate 2 (DQ #221; PR-1 continuation)`.
  Body: list lint classes fixed + counts; state Gate 1/2/3 exit
  codes; HANDOVER trailer (files: e2e.rs; P2/3.E.2 deferred per
  DQ #222; create_report.rs inherited from f258824b5 unchanged).
- **DQ discipline:** if you must block, `kind: "blocker"`, `from:
  "impl"`, `answered_by: null`, commit + push immediately. Do NOT
  self-resolve a blocker. Do NOT write `answered_by: "advisor"`.

## §5 Acceptance criteria

1. `git cherry-pick f258824b5` applied; diff vs base = ONLY
   `e2e.rs` + `create_report.rs` BEFORE lint work begins.
2. Gate 1 (`cargo check --workspace --features full`) exits 0.
3. Gate 2 (`clippy --workspace --features full --tests --no-deps --
   -D warnings`) exits 0 — OR exits 0 scoped to `--test e2e` with a
   `kind: "log"` DQ documenting that the only remaining failure is
   the out-of-scope `reputation_snapshot.rs:856` pre-existing lint.
4. Gate 3 (full e2e suite, ~26 min) exits 0 — every test green.
5. `grep -c "^async fn " crates/server/tests/e2e.rs` == 71.
6. ZERO behavior change: no removed assertions, no changed seed
   data, no `#[ignore]`, no blanket `#![allow]`.
7. Pass 2 / 3.E.2 untouched (4 fixtures modules structurally
   unchanged).
8. One squashed commit with the §4 subject + HANDOVER trailer,
   pushed to the worker branch.

## §6 Concurrency / handover note

This is the **continuation of the LAST refactor-tier lane (PR-1)**.
On this task's clean completion the advisor: cherry-picks the
resulting code commit onto `chore/refactor-e2e-error-types`, runs
§5.2 advisor-laptop validation (the load-bearing full e2e suite),
opens PR-1 via bm-pr-inline, runs CR + user gates 3/5, merges →
**strict-gate 5/5** → STOPS the autonomous loop → authors the
4-role refactor-tier FINAL retro (which MUST call out the deferred
audit 3.E.2 / Pass 2 as an explicit carry-forward, tracked as
advisor task #8) → waits for user retro sign-off. Do NOT proceed to
v1 PRD planning. Audit 3.E.2 (fixtures dedup) is a separate
post-gate lane, NOT part of this task.
