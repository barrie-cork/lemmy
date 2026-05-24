# impl-task Brief — v1-deps-r1 Task 1: diesel-async 0.9 migration

**Role:** [role:impl-task]
**Phase:** v1-deps-r1
**Task:** 1 (wrapper + 42 callsites + 30 import collapses; single commit)
**Branch:** phase-v1-deps-r1
**Authored:** 2026-05-24

---

## 1. Role + dispatch line

[role:impl-task] v1-deps-r1 task 1 diesel-async 0.9 migration — see .claude/PRPs/briefs/v1-deps-r1-impl-1.md

---

## 2. Scope

Per plan §13 Task 1 (lines 816–944). Bump `diesel-async 0.8.0 -> 0.9.x` in workspace `Cargo.toml`; rewrite the `DbConn::run_transaction` wrapper at `crates/diesel_utils/src/connection.rs` per plan §10.1; mechanically rewrite all 42 `.run_transaction(...)` callsites (Shapes A/B/C per plan §10.2) and 30 `scoped_futures::ScopedFutureExt` imports per plan §10.3. **Single commit.**

After Task 0 (DQ `c60d3217f899-001` resolved option-a — Sha256 +1 drift accepted; PR #136 catalyst noted; proceed), baseline counts at this commit are still **42 / 30 / 38 / 0** for `(run_transaction / scoped_futures / scope_boxed / impl.*Digest)` — Sha256 drift was an unrelated Task 2 concern.

**Out of scope:**
- No `crates/server/tests/e2e.rs` edits (0 callsites in e2e.rs — verified at plan-author time; the e2e edit-hang lesson is non-applicable here).
- No sha2 / SemVer-compat work — those are Tasks 2 / 3.
- No `chore(lint):` task before T1 — Task 0 Probe 6 confirmed workspace clippy baseline is exit 0 clean.
- No `futures_util::FutureExt::boxed` change at `connection.rs:220` — that's `BoxFuture` not `ScopedBoxFuture` (plan §13 T1 GOTCHA, verified).

---

## 3. Required reading

### 3.1 Plan sections (cite by line range)

- `.claude/PRPs/plans/v1-deps-r1.plan.md` §10.1 (wrapper signature, lines 382–421)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §10.2 (per-callsite rewrites, Shapes A/B/C, lines 422–512)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §10.3 (import collapse, lines 513–539)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §11 (alphabetical file order, lines 580–710)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §13 Task 1 (full IMPLEMENT discipline, lines 816–944) — authoritative
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §15 (DoD commands, wrapper-prefixed)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §16a Story 1 (story checkpoint)

### 3.2 Mandatory lessons (per advisor-orchestrator.md §2.4 file-class injection)

The 34-file modify list per plan §13 T1 FILES YAML triggers these classes:

- `feedback_multi_write_handlers_need_transactions.md` — handlers doing 2+ DB writes (T1 touches every such file; the wrapper IS the transaction boundary). **R12 byte-for-byte preservation rule:** closure body content preserved; only wrapping shape changes.
- `feedback_features_full_workspace_only.md` — every cargo command in Step 5 + DoD uses `--workspace --features full`, never `-p <crate> --features full`.
- `feedback_features_full_p_crate_incompatible.md` — companion to above.
- `feedback_pq_sys_wrapper_env_propagation.md` — wrapper-prefix discipline; libpq.dll vcpkg PATH.
- `feedback_wrapper_script_flag_silence.md` — flags after `--` reach cargo, before reach the wrapper.
- `feedback_pipes_mask_exit_codes.md` — Step 5 writes rg to file then `wc -l`; never pipe wc to anything that masks exit.

### 3.3 Mechanical-sweep lessons (R11 + R12 + edit discipline)

- `feedback_fix_impl_enumerate_all_callsites.md` — rg-enumerate-first; brief and plan name 42 explicitly; if Step 1 returns ≠42, STOP and file `kind: "blocker"` DQ.
- `feedback_fix_impl_pre_push_cargo_check.md` — Step 5 cargo check is MANDATORY before push. Non-zero exit → patch in same commit (if in-scope) OR `kind: "blocker"` DQ (if out-of-scope). NEVER `#[allow]`-spam.
- `feedback_verify_files_with_read.md` — Junior MUST Read each file fully before Edit (per Brehon hard rule + plan §13 T1 Step 4.1).

### 3.4 Handover from prior cohort (Task 0)

Task 0 was non-`[P]` (serial, no cohort). Prior-task handover summary:

```yaml
prior_task:
  task: 0
  commit: de57d5859
  type: pre-flight harness audit (no impl)
  outcome: 11/12 probes PASS, 1 advisory (Sha256 +1 unrelated to T1)
  blocker_DQ:
    id: c60d3217f899-001
    resolved_by: advisor (option-a, commit 4b1734c28)
    impact_on_T1: none — Sha256 is T2 concern; T1 baselines (42/30/38) confirmed unchanged
  baseline_at_HEAD:
    run_transaction_callsites: 42
    scoped_futures_imports: 30
    scope_boxed_total: 38
    clippy_workspace: exit 0 (clean)
    e2e_no_run: exit 0
    docker: up
    wrappers: cargo-check.bat / cargo-clippy.bat / cargo-test.bat all honor -p + --features + exit codes
  notes: |
    Submodule `crates/email/translations` was uninitialized at worker
    worktree creation (CLAUDE.md "Lane worktree bootstrap" — `git worktree add`
    doesn't init submodules). Worker fixed inline via
    `git submodule update --init --recursive`. Task 1 worker should run
    the same command at brief intake if its worktree was created without
    submodule init.
```

---

## 4. Constraints

### 4.1 Hard rules (refuse if violated)

- **No commits to other tasks' files.** This is single-commit T1; the FILES YAML in plan §13 T1 (lines 822–861) is the authoritative file list.
- **Single commit.** Wrapper + all 42 callsites + all 30 imports in ONE commit. Subject: `feat(deps): migrate to diesel-async 0.9 (task 1)`.
- **No `[P]` cohort.** T1 → T2 → T3 strictly serial. Do NOT queue T2 from inside T1.
- **Wrapper-prefix every cargo command.** `cmd //c "scripts\\brehon\\cargo-*.bat ..."`. Bare `cargo` will fail on libpq.dll (Windows discipline).
- **Step 1 counts must equal 42 / 30 / 38.** If ANY count drifts, file `kind: "blocker"` DQ via `scripts/brehon/dq-v3-append-fragment.sh --pending` and STOP. Do NOT proceed.
- **R12 byte-for-byte closure preservation.** Closure body content unchanged; only wrapping shape changes. Any semantic delta in `new_string` beyond removing `.scope_boxed()` → `kind: "blocker"` DQ.
- **Step 5 pre-push cargo check is MANDATORY** (per `feedback_fix_impl_pre_push_cargo_check.md`). Non-zero exit → patch in same commit if in-scope, else `kind: "blocker"` DQ. NEVER `#[allow]`-spam.
- **Read before Edit** (Brehon hard rule) — every one of the 34 files gets a Read first.

### 4.2 DQ discipline (mid-task push per `decision-queue.md`)

- Use `bash scripts/brehon/dq-v3-new-entry.sh` to generate composite id.
- Use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` to append.
- Commit + push immediately after raising any `kind: "blocker"` DQ (mid-task visibility, per `decision-queue.md` §"Mid-task visibility").
- **NEVER write `answered_by: "advisor"`** from this Junior session (Hard refusal #1).
- **NEVER write `kind: "clarify"`** (advisor-only kind; T1 ambiguity → `kind: "blocker"` from `from: "impl"`).

### 4.3 Windows file-write discipline

- Write rg output to files first, then `wc -l` the files (per `feedback_pipes_mask_exit_codes.md`).
- Use worktree-internal paths: `.claude/PRPs/debug/v1-deps-r1-task1-*.log` (NOT `/tmp/` — Windows `/tmp` unreliable; `feedback_windows_tmp_path_unreliable.md`).
- Capture cargo logs via the `> .log 2>&1` redirect; `$status=$?` immediately after; check `[ $status -eq 0 ]` before proceeding.

### 4.4 Post-commit DQ: raise `kind: "validate-pending-laptop-e2e"`

Per plan §13 T1 Step 6 + DQ `a3d0e9941441-016`. After commit + push (Step 6), raise ONE `kind: "validate-pending-laptop-e2e"` DQ with `commands[]` array:

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-deps-r1-task1-validate-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full -- -D warnings > .claude/PRPs/debug/v1-deps-r1-task1-validate-clippy.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_api --features full --lib > .claude/PRPs/debug/v1-deps-r1-task1-validate-libtest-api.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_apub --features full --lib > .claude/PRPs/debug/v1-deps-r1-task1-validate-libtest-apub.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_db_schema --features full --lib > .claude/PRPs/debug/v1-deps-r1-task1-validate-libtest-dbschema.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-deps-r1-task1-validate-e2e.log 2>&1"'
```

`branch: phase-v1-deps-r1`, `phase_task: 1`. Advisor laptop session runs these locally on its next poll (Shape G SUSPENDED until 2026-06-01 per `project_shape_g_suspended_2026_05_16`).

### 4.5 HANDOVER trailer (Task 2 dependency)

Commit body MUST end with a `HANDOVER:` YAML trailer summarising for Task 2:

```yaml
HANDOVER:
  task: 1
  files_modified: 34
  callsite_counts:
    pre_run_transaction: 42
    post_run_transaction: 42  # preserved
    pre_scoped_futures: 30
    post_scoped_futures: 0
    pre_scope_boxed: 38
    post_scope_boxed: 0
  cargo_check_status: 0
  cargo_lock_delta_lines: <actual diff line count>
  shape_distribution:
    shape_a: <count>
    shape_b: <count>
    shape_c: <count>
  notes: |
    diesel-async 0.9 wrapper + callsites complete. T2 (sha2 0.11) sees
    a workspace where the transaction wrapper signature is the new
    AsyncFnOnce + AsyncFunc form. Sha256 callsites unaffected (9 per
    Task 0 Probe 8d; up from plan's 8 due to admin_rule_sets.rs).
```

If `Cargo.lock` diff exceeds ±200 lines, raise `kind: "log"` DQ (resolved-immediately) per plan §13 T1 GOTCHA — flag unexpected transitive resolution drift for advisor verification before T2.

### 4.6 Special-case verifications (per plan §13 T1 GOTCHA)

- **`diesel_async::AsyncFunc` re-export path:** verify exact public path at task-time (could be `diesel_async::AsyncFunc` or `diesel_async::prelude::AsyncFunc`). If `cargo check` reports "cannot find trait `AsyncFunc`", inspect `~/.cargo/registry/src/.../diesel-async-0.9.x/src/lib.rs` or `crates.io/crates/diesel-async/0.9.0` source.
- **`reputation_snapshot.rs`** imports `scoped_futures` but has 0 `.run_transaction` callsites. Verify whether the import is genuinely unused vs used indirectly via `.transaction(...)` on raw `AsyncPgConnection`. If unused → drop import. If used indirectly → STOP and file `kind: "blocker"` DQ (Shape D the plan didn't anticipate).
- **Shape A / B / C classification:** per plan §10.2. Shape C drops `.scope_boxed()` AND adds an explicit `.await` because `async |conn| { expr }` requires the body to be an expression evaluating to `LemmyResult<R>`, not a future. Mis-classifying breaks compilation (Shape A→C drops the `Ok(...)`; Shape C→A wraps an unnecessary `{}` block — compiles but stylistically wrong).

---

## 5. Validation gate (single-task)

This task's per-task validation gate IS the §15 DoD subset listed in §4.4 above. The advisor laptop runs them post-commit when it picks up the `kind: "validate-pending-laptop-e2e"` DQ. **Junior does NOT run the e2e — only the Step 5 in-task `cargo check --workspace --features full`.** The e2e + per-crate lib-tests are advisor-laptop only.

---

## 6. Post-task

- Commit + push to `phase-v1-deps-r1` worker branch (daemon finalize-merges to `phase-v1-deps-r1`).
- Raise the §4.4 `validate-pending-laptop-e2e` DQ. Commit + push immediately.
- DO NOT proceed to T2 dispatch — advisor reviews validate-pending result first.
- Junior task complete when DQ is raised + pushed; advisor takes over for validation cycle.
