---
phase: chore/refactor-toctou
role: impl-task
task: refactor
brief_n: 1
authored: 2026-05-14
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_finding: 3.B.1 (rank 3, CRITICAL)
parent_phase_tip: <set by bm-cut — branch tip is governance-v0 HEAD at bm-cut time>
---

# [role:impl-task] chore/refactor-toctou — wrap create_report SELECT-then-write in run_transaction — see .claude/PRPs/briefs/refactor-toctou-impl.md

## §1 Role + dispatch

`[role:impl-task] chore/refactor-toctou — wrap SELECT-then-UPDATE/INSERT in create_report.rs in run_transaction per audit §3.B.1 (CRITICAL)`

## §2 Scope

### §2.1 Driving audit finding

Audit §3.B.1 (rank 3, severity **CRITICAL**, effort M, frequency 1):

> `create_report.rs:130-232` · Lens 1+2 · Axis-4 both · **[CRIT]** SELECT-then-UPDATE-or-INSERT branch with NO `run_transaction` wrapper — concurrent reports targeting the same entity can slip between read and write, causing duplicate case creation or lost weight accumulation; sibling files in the same directory (`create_endorsement.rs:136`, `revoke_endorsement.rs:143`) DO use `run_transaction` for analogous patterns — internal inconsistency · Lens 1: Lemmy CRUD convention requires atomicity for non-idempotent writes; Lens 2: TOCTOU race is a classic Rust safety lapse; the internal-inconsistency makes this a stronger finding than parity-only · wrap lines 130-232 in `conn.run_transaction(|conn| { async move { ... }.scope_boxed() })` capturing the entire SELECT-UPDATE-INSERT sequence · **M** · 1 file, single fix.

### §2.2 The canonical pattern (from `crates/api/api_crud/src/governance/create_endorsement.rs:130-145`)

```rust
let conn = &mut get_conn(pool).await?;

let data_for_tx = data;
let pseudonym_for_tx = sponsor_pseudonym.clone();

let outcome = conn
  .run_transaction(|conn| {
    async move {
      process_endorsement(conn, sponsor_id, pseudonym_for_tx, data_for_tx).await
    }
    .scope_boxed()
  })
  .await?;
```

The pattern is: (1) acquire conn, (2) move/clone data into closure scope, (3) call `run_transaction` with a `scope_boxed`'d async block, (4) named helper fn `process_<verb>` holds the actual SELECT + write logic.

### §2.3 The fix

**File:** `crates/api/api_crud/src/governance/create_report.rs`

**Lines to refactor:** approximately 130-232 (the SELECT existing-case block + the UPDATE-or-INSERT branch + governance_log append). Read the file in the worker session BEFORE editing to confirm exact line boundaries — they may have drifted.

**Refactor shape (mirror `create_endorsement.rs:130-145`):**

1. **Extract the body into a named helper:**
   ```rust
   /// Body of the `run_transaction` closure. Named helper so the outer
   /// future stays under the workspace `large_futures` lint threshold
   /// (mirror of `create_endorsement::process_endorsement`).
   async fn process_report(
     conn: &mut AsyncPgConnection,
     reporter_id: PersonId,
     pseudonym: String,
     data: CreateReport,
     // ...any other state captured from the outer scope
   ) -> LemmyResult<CreateReportResponse> {
     // existing lines 130-232 body, with `pool_ref` replaced by `&mut *conn`
     // for governance_log::append + actor_pseudonym_helper::get_or_create calls
   }
   ```

2. **Replace the inline body at the call site:**
   ```rust
   let conn = &mut get_conn(pool).await?;

   let data_for_tx = data;
   let pseudonym_for_tx = pseudonym.clone();

   let outcome = conn
     .run_transaction(|conn| {
       async move {
         process_report(conn, reporter_id, pseudonym_for_tx, data_for_tx).await
       }
       .scope_boxed()
     })
     .await?;
   ```

3. **Adjust callee signatures of `governance_log::append` + `actor_pseudonym_helper::get_or_create`** if they currently take `&mut DbPool<'_>`. Inside the run_transaction closure, you have `&mut AsyncPgConnection` (the `conn` param). Use whichever signature variant exists; if both take pool-only, you must extract the pseudonym before the tx OR refactor the helper signatures.

   Per audit §3.E.13: `governance_log::append` already accepts coercion from `&mut DbPool` to `&mut AsyncPgConnection` via `(&mut *conn).into()`. Use that pattern: `governance_log::append(&mut (&mut *conn).into(), ...)`.

   `actor_pseudonym_helper::get_or_create` — read its signature; if it requires pool, **move it OUTSIDE the tx** (acquire pseudonym BEFORE `run_transaction`), pass as parameter. This is what `create_endorsement.rs` does at line ~120.

### §2.4 Pre-flight worker checklist

The worker session reads these files BEFORE making any edits:

1. `crates/api/api_crud/src/governance/create_endorsement.rs` lines 100-160 — canonical pattern (the model).
2. `crates/api/api_crud/src/governance/revoke_endorsement.rs` lines 70-100 — second example of the pattern.
3. `crates/api/api_crud/src/governance/create_report.rs` lines 60-260 — the file being refactored; read the WHOLE handler fn `create_report` body so the refactor preserves the outer-scope variable bindings.
4. `governance_log::append` signature — find the canonical at `crates/db_schema/src/source/governance/governance_log.rs` `pub fn append` declaration.
5. `actor_pseudonym_helper::get_or_create` signature — find the helper.

If any pattern doesn't match the audit's claim (e.g. `create_report.rs` is already wrapped, or `process_report` helper already exists), STOP and file a DQ blocker. The audit is recent (2026-05-14) but baseline drift is possible.

### §2.5 Post-edit verification

```bash
# Confirm run_transaction is present in create_report.rs
grep -n "run_transaction" crates/api/api_crud/src/governance/create_report.rs
# Expected: ≥1 hit (the new wrapper)

# Confirm scope_boxed is present (the canonical pattern)
grep -n "scope_boxed" crates/api/api_crud/src/governance/create_report.rs
# Expected: 1 hit

# Confirm process_report helper exists
grep -n "^async fn process_report" crates/api/api_crud/src/governance/create_report.rs
# Expected: 1 hit

# Confirm sibling files still have their run_transaction (no accidental regression)
grep -c "run_transaction" crates/api/api_crud/src/governance/create_endorsement.rs crates/api/api_crud/src/governance/revoke_endorsement.rs
# Expected: ≥1 per file
```

### §2.6 Concurrency test consideration (out of scope, but worth flagging)

This refactor closes the TOCTOU race but does NOT add a concurrency test exercising the race. The audit didn't require one. A proper concurrency test would need 2 concurrent `create_report` calls against the same target with a sleep injection between SELECT and INSERT to force the race window — non-trivial; defer to future test-coverage work.

If during the refactor you find an obvious place to add a unit test for the new helper (`process_report`), do so — but only if XS effort. Otherwise leave as-is.

### §2.7 Validation gate (Shape G)

Push the worker branch. Triggers `cargo-validate-workspace.yml`.

The workspace check runs `cargo test --no-run` which catches compile errors but does NOT exercise the new transaction. For real validation, the existing `report_to_modlog_golden_path` e2e test in `crates/server/tests/e2e.rs` will exercise this code path on Phase 2 e2e (per `feedback_phase_2_e2e_gate_enforcement.md`, the user decides local vs dispatch at bm-pr time).

Raise `kind: "validate-pending"` DQ entry per Recipe 1.

## §3 Required reading

- `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` §3.B.1 — driving finding; §5.1 internal-inconsistency framing
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` — PR-2 of 6 context
- `crates/api/api_crud/src/governance/create_endorsement.rs` lines 100-200 — canonical pattern (Mirror)
- `crates/api/api_crud/src/governance/revoke_endorsement.rs` lines 70-150 — second canonical example
- `.claude/agents/impl-task.md` — subagent contract
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — the exact lesson this refactor operationalizes
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — LemmyResult discipline (this fn returns LemmyResult already)
- `.claude/rules/decision-queue.md` Recipe 1
- `.claude/rules/governance-log-entry-kind-registry.md` — `governance_log::append` discipline (the call inside the new tx)
- Upstream Lemmy `crates/api/api_crud/src/community/create.rs` (read via `git show upstream/main:crates/api/api_crud/src/community/create.rs`) — Lemmy's canonical pre-tx-validate pattern for reference

## §4 Constraints

- **Files:** ONLY `crates/api/api_crud/src/governance/create_report.rs`. NO other files.
- **Edits:** structural refactor (extract `process_report` helper + wrap call site in `run_transaction`). No behavior change beyond closing the race. No new fields. No new variants. No new validation logic. **The refactor MUST preserve every existing observable behavior** — same outputs for same inputs; same errors raised at same conditions; same governance_log entries written with same payloads.
- **Helper signature:** `process_report` takes `&mut AsyncPgConnection` as first param (NOT `&mut DbPool`). This is required for the tx-body discipline (we're INSIDE the tx; conn is what we have).
- **Pseudonym handling:** `actor_pseudonym_helper::get_or_create` MUST run BEFORE `run_transaction` per `create_endorsement.rs` pattern. Pass the resulting String into `process_report` by move.
- **`governance_log::append`** inside the tx body uses the `&mut (&mut *conn).into()` coercion pattern per audit §3.E.13 + the registry rule. Verify the signature accepts this form.
- **Pre-tx validation** (reason_code length + emptiness check at lines 75-81): this is the §3.B.6 POSITIVE exemplar — KEEP IT WHERE IT IS, pre-tx. Do NOT move it inside the tx.
- **Branch:** `chore/refactor-toctou`.
- **Pre-push cargo-check (mandatory per `feedback_fix_impl_pre_push_cargo_check.md`):**

  ```bash
  bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/refactor-toctou-precheck.log 2>&1
  status=$?
  tail -20 .claude/PRPs/debug/refactor-toctou-precheck.log
  [ $status -eq 0 ] || exit $status
  ```

  Per `.claude/rules/cargo-output-capture.md`. If non-zero: STOP, surface failure mode. Most likely compile errors: `process_report` signature mismatch with call site; `governance_log::append` coercion failure; pseudonym scope issue.

- **Test target compile (mandatory for struct-shape changes per `feedback_test_target_compile_validation.md`):**

  ```bash
  bash scripts/brehon/cargo-test.sh --workspace --features full --no-run > .claude/PRPs/debug/refactor-toctou-testcompile.log 2>&1
  status=$?
  tail -20 .claude/PRPs/debug/refactor-toctou-testcompile.log
  [ $status -eq 0 ] || exit $status
  ```

  This catches the case where `create_report` is called from `e2e.rs` and the signature change ripples into test code.

- **Shape G:** push triggers workspace check. Raise 1 validate-pending DQ entry.
- **DQ atomic raise + ensure_ascii=False + next_id-spans-archives** per standard discipline.
- **COMMIT MESSAGE:** `fix(api_crud): wrap create_report SELECT-then-write in run_transaction (audit 3.B.1 CRIT)`

## §5 Out of scope

- The `request_appeal.rs` validate-inside-tx finding (audit §3.B.2 / rank-untiered MED) — separate concern; defer.
- Adding concurrency tests for the TOCTOU race — non-trivial; defer to future test-coverage work.
- Refactoring other handlers in `crates/api/api_crud/src/governance/` for any pattern.
- Touching `create_endorsement.rs` or `revoke_endorsement.rs` (they are the canonical mirror; do not modify).
- Renaming or restructuring `governance_log::append` signature.

## §6 HANDOVER trailer

Critical-severity finding; the only one in PR-2's scope. Trailer recommended:

```yaml
HANDOVER:
  filesCreated: []
  filesModified:
    - crates/api/api_crud/src/governance/create_report.rs
  keyDecisions:
    - Wrapped SELECT-then-write block in run_transaction per audit §3.B.1; extracted process_report helper mirroring create_endorsement::process_endorsement pattern.
    - Pseudonym fetch stays pre-tx; governance_log::append uses (&mut *conn).into() coercion inside tx body.
  notes: CRITICAL TOCTOU race closed. Pre-tx validation (reason_code checks) intentionally left at pre-tx position per audit §3.B.6 positive exemplar. No concurrency test added (out of scope per brief §5).
```
