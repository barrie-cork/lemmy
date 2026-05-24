# impl-task Brief — v1-deps-r1 Task 1: diesel-async 0.9 migration (REV-2 — subagent batching)

**Role:** [role:impl-task]
**Phase:** v1-deps-r1
**Task:** 1 (wrapper + 42 callsites + 30 import collapses; single commit)
**Branch:** phase-v1-deps-r1
**Authored:** 2026-05-24 (rev-1)
**Revised:** 2026-05-24 19:50 UTC (rev-2 — subagent batching authorised after Task #454 hit Junior 150-turn cap)

---

## 0. REV-2 change summary (read this first)

Task #454 (first dispatch of this brief) **failed with `error_max_turns` at 150 turns** after editing all 33 files but before commit + push. The fix is to use **parallel subagents (Task tool) for the file-by-file mechanical sweep** so parent worker turn count stays under 30. This brief amendment overrides `.claude/agents/impl-task.md:316` ("Never invoke Agent(...) — subagents cannot nest") **for this task only**. The override is documented in §4.7 below; the agent definition is NOT being modified.

Evidence for the override: Task #454 log at `/srv/brehon-fork/.junior/logs/job-454-run-454.log` recorded 223 assistant turns, exit 1 via `subtype: error_max_turns` after 1242s. Daemon hard-cap is 150 turns (`/opt/junior-src/src/core/claude.ts` BREHON CARRY-PATCH). Single-context serial sweep of 33 Rust files at 4-6 turns/file structurally cannot fit. Bounded subagent fanout is the only mechanical fix that preserves single-commit semantics.

---

## 1. Role + dispatch line

[role:impl-task] v1-deps-r1 task 1 diesel-async 0.9 migration — see .claude/PRPs/briefs/v1-deps-r1-impl-1.md

---

## 2. Scope

Per plan §13 Task 1 (lines 816–944). Bump `diesel-async 0.8.0 -> 0.9.x` in workspace `Cargo.toml`; rewrite the `DbConn::run_transaction` wrapper at `crates/diesel_utils/src/connection.rs` per plan §10.1; mechanically rewrite all 42 `.run_transaction(...)` callsites (Shapes A/B/C per plan §10.2) and 30 `scoped_futures::ScopedFutureExt` imports per plan §10.3. **Single commit.**

After Task 0 (DQ `c60d3217f899-001` resolved option-a — Sha256 +1 drift accepted; PR #136 catalyst noted; proceed), baseline counts at HEAD are **42 / 30 / 38 / 0** for `(run_transaction / scoped_futures / scope_boxed / impl.*Digest)` — Sha256 drift was an unrelated Task 2 concern.

**Out of scope:**
- No `crates/server/tests/e2e.rs` edits (0 callsites — verified at plan-author time).
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

- `feedback_multi_write_handlers_need_transactions.md` — R12 byte-for-byte closure preservation rule
- `feedback_features_full_workspace_only.md` — every cargo command uses `--workspace --features full`
- `feedback_features_full_p_crate_incompatible.md` — companion
- `feedback_pq_sys_wrapper_env_propagation.md` — wrapper-prefix discipline
- `feedback_wrapper_script_flag_silence.md` — flags after `--` reach cargo, before reach the wrapper
- `feedback_pipes_mask_exit_codes.md` — write rg to file then `wc -l`

### 3.3 Mechanical-sweep lessons

- `feedback_fix_impl_enumerate_all_callsites.md` — rg-enumerate-first
- `feedback_fix_impl_pre_push_cargo_check.md` — Step 5 cargo check MANDATORY before push
- `feedback_verify_files_with_read.md` — Read before Edit

### 3.4 Subagent-batching lessons (NEW for rev-2)

- `.claude/rules/advisor-orchestrator.md` §6.3 — "Brief like a smart colleague who just walked into the room. Sub-agents see no parent conversation. Trust but verify: sub-agent reports describe what they *intended* to do, not necessarily what they did."
- `.claude/rules/advisor-orchestrator.md` §6.1 + §6.2 (referenced via refs/) — parallel dispatch + verify-after-completes patterns. The parent worker is in role of advisor here for the subagent fanout.

### 3.5 Handover from prior task (Task 0)

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
    worktree creation. Run `git submodule update --init --recursive`
    if this worktree was created without submodule init.

prior_dispatch_t1:
  task_id: 454
  outcome: failed_max_turns
  files_edited_before_failure: 33 (all .rs in FILES YAML)
  turns_consumed: 223 (capped at 150)
  pushed: false
  lesson_internalised: |
    Serial Read+Edit of 33 files in single context exceeds the
    150-turn daemon cap. This rev-2 brief uses bounded subagent
    fanout (§4.7) to keep parent turn count <30.
```

---

## 4. Constraints

### 4.1 Hard rules (refuse if violated)

- **No commits to other tasks' files.** Authoritative file list = plan §13 T1 FILES YAML (lines 822–861).
- **Single commit.** Wrapper + Cargo.toml + Cargo.lock + all 33 .rs files in ONE commit. Subject: `feat(deps): migrate to diesel-async 0.9 (task 1)`.
- **Wrapper-prefix every cargo command.** `cmd //c "scripts\\brehon\\cargo-*.bat ..."`. The worker is on Linux (EliteDesk) so use the `.sh` form: `bash scripts/brehon/cargo-check.sh ...`.
- **Step 1 counts must equal 42 / 30 / 38.** If ANY count drifts, file `kind: "blocker"` DQ via `bash scripts/brehon/dq-v3-append-fragment.sh --pending` and STOP.
- **R12 byte-for-byte closure preservation.** Closure body content unchanged; only wrapping shape changes. Any semantic delta beyond removing `.scope_boxed()` → `kind: "blocker"` DQ.
- **Step 5 pre-push cargo check is MANDATORY**. Non-zero exit → patch in same commit if in-scope, else `kind: "blocker"` DQ. NEVER `#[allow]`-spam.
- **Read before Edit** (Brehon hard rule).

### 4.2 DQ discipline

- Use `bash scripts/brehon/dq-v3-new-entry.sh` to generate composite id.
- Use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` to append.
- Commit + push immediately after raising any `kind: "blocker"` DQ.
- **NEVER write `answered_by: "advisor"`** (Hard refusal #1).
- **NEVER write `kind: "clarify"`** (advisor-only kind).

### 4.3 File-write discipline (worker is on Linux EliteDesk)

- Write rg output to files first, then `wc -l` the files.
- Use worktree-internal paths: `.claude/PRPs/debug/v1-deps-r1-task1-*.log`.
- Capture cargo logs via `> .log 2>&1` redirect; check `$?` immediately after.

### 4.4 Post-commit DQ: raise `kind: "validate-pending-laptop-e2e"`

Per plan §13 T1 Step 6 + DQ `a3d0e9941441-016`. After commit + push, raise ONE `kind: "validate-pending-laptop-e2e"` DQ with `commands[]`:

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-deps-r1-task1-validate-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full -- -D warnings > .claude/PRPs/debug/v1-deps-r1-task1-validate-clippy.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_api --features full --lib > .claude/PRPs/debug/v1-deps-r1-task1-validate-libtest-api.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_apub --features full --lib > .claude/PRPs/debug/v1-deps-r1-task1-validate-libtest-apub.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_db_schema --features full --lib > .claude/PRPs/debug/v1-deps-r1-task1-validate-libtest-dbschema.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-deps-r1-task1-validate-e2e.log 2>&1"'
```

`branch: phase-v1-deps-r1`, `phase_task: 1`. Advisor laptop session runs these locally (Shape G SUSPENDED until 2026-06-01).

### 4.5 HANDOVER trailer

Commit body MUST end with:

```yaml
HANDOVER:
  task: 1
  files_modified: 35  # 33 .rs + Cargo.toml + Cargo.lock
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
    Task 0 Probe 8d).
```

If `Cargo.lock` diff exceeds ±200 lines, raise `kind: "log"` DQ per plan §13 T1 GOTCHA.

### 4.6 Special-case verifications (per plan §13 T1 GOTCHA)

- **`diesel_async::AsyncFunc` re-export path:** verify at task-time. If `cargo check` reports "cannot find trait `AsyncFunc`", inspect `~/.cargo/registry/src/.../diesel-async-0.9.x/src/lib.rs`.
- **`reputation_snapshot.rs`** imports `scoped_futures` but has 0 `.run_transaction` callsites. If genuinely unused → drop import; if used indirectly → `kind: "blocker"` DQ (Shape D).
- **Shape A / B / C classification:** per plan §10.2. Shape C drops `.scope_boxed()` AND adds explicit `.await`.

### 4.7 SUBAGENT BATCHING AUTHORISATION (T1 carve-out — overrides impl-task.md:316 for THIS task only)

**Override:** the impl-task agent definition at `.claude/agents/impl-task.md:316` reads `Never invoke Agent(...) — subagents cannot nest`. **This brief explicitly overrides that prohibition for this task only.** The agent definition is not being modified — future impl-tasks continue to default to the existing refusal. This is a documented carve-out justified by the Task #454 turn-cap evidence in §0.

**You ARE authorised to use the `Task` tool (= `Agent`) for the mechanical file-by-file sweep in Step 4 of plan §13 T1.** Permission mode is `full` so the tool is technically available. The shape of this authorisation is bounded — see below. If your authorisation envelope feels uncertain, file a `kind: "blocker"` DQ rather than improvising.

**Bounded fanout shape (mandatory):**

1. **Parent worker does Steps 1, 2, 3 of plan §13 T1 sequentially in its own context** (count enumeration, `Cargo.toml` bump, wrapper rewrite at `connection.rs`). These are foundational; subagents cannot do them in parallel with each other because the wrapper change must land before any callsite can compile against the new signature. Budget: ~10 parent turns.

2. **Parent worker partitions the 33 .rs callsite files into 4 disjoint batches** of 8-9 files each, grouped by directory affinity to maximise per-subagent context locality. Suggested partition:
   - **Batch A** (8 files): `crates/api/api/src/community/` (4) + `crates/api/api/src/site/registration_applications/approve.rs` (1) + `crates/api/api_crud/src/user/create.rs` (1) + `crates/diesel_utils/src/connection.rs` ALREADY DONE BY PARENT — exclude + `crates/routes/src/utils/setup_local_site.rs` (1) + `crates/db_schema/src/source/governance/governance_log.rs` (1) = 8 (excluding the connection.rs done by parent)
   - **Batch B** (8 files): `crates/api/api/src/governance/` group 1 — files alphabetically `accept_jury_assignment` through `admin_emergency_remove` (5) + `crates/api/api_crud/src/governance/` (3 first by alphabetical)
   - **Batch C** (8 files): `crates/api/api/src/governance/` group 2 — `admin_rule_sets` through `submit_jury_vote` (7) + `crates/api/api_crud/src/governance/revoke_endorsement.rs` (1) = 8
   - **Batch D** (9 files): `crates/apub/` (3) + `crates/db_schema/src/impls/` (5) + one more if math works out
   - **THE EXACT PARTITION IS PARENT'S CHOICE** as long as: (a) every file in plan §13 T1 FILES YAML modifies[] except Cargo.toml/Cargo.lock/connection.rs gets exactly one batch assignment, (b) batches are disjoint, (c) batch count ≤4.

3. **Parent worker spawns the 4 subagents IN PARALLEL via a single message with multiple Task tool uses** (single-message parallel dispatch per advisor-orchestrator.md §6.1). Each subagent receives a self-contained prompt containing:
   - The exact file list for THIS batch
   - The Shape A / B / C recipe verbatim from plan §10.2 (copy the relevant excerpt INTO the prompt — subagent has no parent context)
   - The import-removal recipe from plan §10.3
   - The R12 byte-for-byte preservation rule
   - Instruction: "For each file in your batch: Read fully, identify each `.run_transaction(...)` block, classify Shape A/B/C, apply the corresponding Edit. For the `scoped_futures` import line: rewrite per §10.3. After every file: `rg "scope_boxed" <file>` must return 0. Report back: {batch_name, files_processed: [...], edits_made: <count>, any_failures: [...], any_Shape_D: [...]}"
   - **Hard refusal in subagent prompt:** "Do NOT commit. Do NOT push. Do NOT run cargo. Do NOT spawn nested subagents. Only Read + Edit + Grep."

4. **Parent worker waits for all 4 subagents to return** (parallel dispatch returns when all complete). Budget: ~5 parent turns for dispatch + ~5 for collecting results.

5. **Parent worker runs verify-after-subagent-completes (mandatory):**
   ```bash
   # Verify post-conditions across the whole workspace
   rg "scope_boxed" crates/ tests/ > .claude/PRPs/debug/v1-deps-r1-task1-verify-scope-boxed.txt; wc -l .claude/PRPs/debug/v1-deps-r1-task1-verify-scope-boxed.txt
   # EXPECT: 0
   rg "scoped_futures" crates/ > .claude/PRPs/debug/v1-deps-r1-task1-verify-scoped-futures.txt; wc -l .claude/PRPs/debug/v1-deps-r1-task1-verify-scoped-futures.txt
   # EXPECT: 0
   rg "\.run_transaction" crates/ tests/ > .claude/PRPs/debug/v1-deps-r1-task1-verify-run-transaction.txt; wc -l .claude/PRPs/debug/v1-deps-r1-task1-verify-run-transaction.txt
   # EXPECT: 42
   ```
   Any drift → `kind: "blocker"` DQ naming which subagent's batch was incomplete + STOP. **Trust but verify** — subagent reports describe intent; the rg counts are ground truth.

6. **Parent worker runs Step 5 cargo check** (mandatory):
   ```bash
   bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-deps-r1-task1-check.log 2>&1
   status=$?
   tail -20 .claude/PRPs/debug/v1-deps-r1-task1-check.log
   [ $status -eq 0 ] || { echo "CARGO CHECK FAILED"; exit 1; }
   ```

7. **Parent worker runs Step 6 commit + push** with the HANDOVER trailer per §4.5.

8. **Parent worker raises the `validate-pending-laptop-e2e` DQ per §4.4.**

**Parent worker total turn budget:** ~30 (10 sequential foundation + 5 dispatch + 5 collect + 5 verify + 3 cargo check + 2 commit/push). Well under the 150 cap.

**Subagent total turn budget per batch:** ~40-50 (8-9 files × 4-5 turns each for Read+Edit+verify). Each subagent has its own 150-turn cap, well within budget.

**Failure modes specific to subagent batching:**
- **Subagent omits a file:** caught by Step 5 rg counts (expected 42 → actual <42).
- **Subagent reports success but Edit failed silently:** caught by Step 5 rg "scope_boxed" check (expected 0 → actual >0). 
- **Two subagents touch overlapping files:** prevented by parent's disjoint partition + advisor-orchestrator.md §6.1 "file-disjoint" rule. If somehow it happens, git working tree shows merge-conflict-like state; parent worker files `kind: "blocker"` DQ.
- **A subagent itself hits its 150-turn cap:** subagent batch is 8-9 files at ~5 turns each = ~45 turns. 3x headroom. If a batch DOES hit cap (e.g. one file has an unexpected Shape D), parent's verify step (rg count drift) catches it; parent files `kind: "blocker"` DQ.

---

## 5. Validation gate (single-task)

This task's per-task validation gate IS the §15 DoD subset listed in §4.4. The advisor laptop runs them post-commit when it picks up the `kind: "validate-pending-laptop-e2e"` DQ. **Worker does NOT run e2e — only the §4.7 Step 6 in-task `cargo check --workspace --features full`.** E2E + per-crate lib-tests are advisor-laptop only.

---

## 6. Post-task

- Commit + push to `phase-v1-deps-r1` worker branch (daemon finalize-merges to `phase-v1-deps-r1`).
- Raise the §4.4 `validate-pending-laptop-e2e` DQ. Commit + push immediately.
- DO NOT proceed to T2 dispatch — advisor reviews validate-pending result first.
- Junior task complete when DQ is raised + pushed; advisor takes over for validation cycle.
