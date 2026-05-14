---
phase: chore/refactor-diesel-errors
role: impl-task
task: refactor
brief_n: 1
authored: 2026-05-14
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_finding: 3.A.5 (rank 13)
parent_phase_tip: <set by bm-cut — branch tip is governance-v0 HEAD at bm-cut time>
---

# [role:impl-task] chore/refactor-diesel-errors — propagate Diesel errors in admin_audit_stream — see .claude/PRPs/briefs/refactor-diesel-errors-impl.md

## §1 Role + dispatch

`[role:impl-task] chore/refactor-diesel-errors — replace .optional().ok().flatten() in admin_audit_stream.rs with proper error propagation per audit §3.A.5`

## §2 Scope

### §2.1 Driving audit finding

Audit §3.A.5 (rank 13, severity MAJ, effort S, frequency 2):

> `admin_audit_stream.rs:227,234` · Lens 2 · Axis-4 quality-fail · **[MAJ]** `.await.optional().ok().flatten()` chain swallows database errors — the `.ok()` between `.optional()` (returns `Result<Option<T>>`) and `.flatten()` reduces a query failure to `None` indistinguishable from a legitimate empty result · Lens 2: `.ok()` swallows errors; upstream `like.rs` uses `?` propagation or explicit `.map_err()` with context · replace with `await.optional()?` or `.map_err(|e| LemmyErrorType::Unknown(format!("governance_log query: {e}")))` · **S** · 2 sites in 1 file.

### §2.2 Verified shape (pre-edit)

The chain at `crates/api/api/src/governance/admin_audit_stream.rs` around lines 228-234 (verified 2026-05-14):

```rust
let row: Option<GovernanceLog> = governance_log_schema::table
    .filter(governance_log_schema::id.eq(GovernanceLogId(entry_id)))
    .select(GovernanceLog::as_select())
    .first(&mut conn)
    .await
    .optional()
    .ok()       // <-- swallows DB errors silently
    .flatten();
let Some(row) = row else { continue };
```

The audit's claim is correct: `.optional()` returns `Result<Option<GovernanceLog>>`. Chaining `.ok().flatten()` reduces a query failure to `None`, which is then indistinguishable from "no row matched the filter". An observability-critical handler is silently swallowing pool-exhaustion / syntax / connection errors.

### §2.3 The fix

**Audit recommendation:** "replace with `await.optional()?` or `.map_err(|e| LemmyErrorType::Unknown(format!("governance_log query: {e}")))`".

**Picked option: `.optional()?`** — most idiomatic. `?` propagates `DieselError` via the function's `LemmyResult` outer; loses no context.

BUT — `admin_audit_stream.rs` is a streaming handler; check the enclosing function signature before applying. If the enclosing scope is a `tokio::spawn`-ed async block or a `Stream::poll_next` body that DOES NOT return `Result`, `?` won't compile and we need the `.map_err(...).ok()?` variant or restructure.

**Worker pre-flight:** `Read crates/api/api/src/governance/admin_audit_stream.rs` from line 200 to line 250 to verify the enclosing function's error type before editing. Pick the variant that compiles:

- **If enclosing returns `LemmyResult<T>`:** replace `.optional().ok().flatten()` with `.optional()?` (clean propagation; lets `?` desugar to early-return).
- **If enclosing is a stream poll body or `tokio::spawn` closure** (no `Result` outer): replace `.optional().ok().flatten()` with `.optional().map_err(|e| { tracing::warn!("governance_log query failed: {e}"); e }).ok().flatten()` — at least logs the error before erasing it. Still imperfect; flag for follow-up if the function shape can be restructured.

**Apply the same fix at both sites** (lines 227 + 234 in the audit's reading; line numbers may have drifted slightly — use `grep -n "\.optional()\.ok()\.flatten()" crates/api/api/src/governance/admin_audit_stream.rs` to find current sites).

### §2.4 Post-edit verification

```bash
# No remaining .optional().ok().flatten() chains in the file
grep -c "\.optional()\.ok()\.flatten()" crates/api/api/src/governance/admin_audit_stream.rs
# Expected: 0

# Verify the new pattern is present
grep -nE "\.optional\(\)\?|\.optional\(\).map_err" crates/api/api/src/governance/admin_audit_stream.rs
# Expected: 2 hits (one per fixed site)
```

### §2.5 Validation gate (Shape G)

Push the worker branch. The push triggers `cargo-validate-workspace.yml` (path filter `crates/**` matches).

Capture `workflow_run_id`. Raise `kind: "validate-pending"` DQ entry per Recipe 1.

## §3 Required reading

- `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` §3.A.5 — driving finding; §3.A.7 (5 sister sites of similar pattern, deferred to fix-during-relevant-sub-phase)
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` — PR-5 of 6 context
- `crates/api/api/src/governance/admin_audit_stream.rs` lines 200-250 — read BEFORE editing to confirm enclosing fn shape
- Upstream Lemmy `crates/api/api/src/post/like.rs` (read via `git show upstream/main:crates/api/api/src/post/like.rs`) — canonical error propagation pattern to mirror
- `.claude/agents/impl-task.md` — subagent contract
- `.claude/rules/decision-queue.md` Recipe 1
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — error-type case enumeration (Case A is the target shape)

## §4 Constraints

- **Files:** ONLY `crates/api/api/src/governance/admin_audit_stream.rs`. NO other files.
- **Edits:** 2 expression chains rewritten (the two `.optional().ok().flatten()` sites). No structural function changes. No new helpers. No tests added (audit didn't flag a coverage gap).
- **If the enclosing function is not `LemmyResult`-returning:** the brief explicitly authorizes the `.map_err(...).ok().flatten()` variant with `tracing::warn!` logging. DO NOT restructure the function to add `Result`; that's out of scope.
- **Branch:** `chore/refactor-diesel-errors`.
- **Pre-push cargo-check (mandatory per `feedback_fix_impl_pre_push_cargo_check.md`):**

  ```bash
  bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/refactor-diesel-errors-precheck.log 2>&1
  status=$?
  tail -20 .claude/PRPs/debug/refactor-diesel-errors-precheck.log
  [ $status -eq 0 ] || exit $status
  ```

  Per `.claude/rules/cargo-output-capture.md` (capture-then-tail), per `.claude/rules/no-cargo-output-paste.md` (don't paste full log into commit messages or DQ entries). If non-zero: STOP, file DQ blocker.

- **Shape G:** push triggers workspace check. Raise 1 validate-pending DQ entry.
- **DQ discipline:** atomic raise + ensure_ascii=False + next_id-spans-archives.
- **COMMIT MESSAGE:** `chore(refactor): propagate Diesel errors in admin_audit_stream (audit 3.A.5)`

## §5 Out of scope

- The 5 sister sites in audit §3.A.7 (`jury_common.rs:49,86`, `sponsor_liability_grace.rs`, `admin_rule_sets.rs`, `admin_audit_stream.rs`) — those are tier `fix-during-relevant-sub-phase` (rank 5 score 40 but tier downgrades to "handle when touching"). DO NOT pre-emptively fix.
- Adding `tracing::warn!` logging to anywhere except the fallback variant if used.
- Adding new tests for the error path.
- Renaming variables or restructuring the streaming logic.

## §6 HANDOVER trailer

Parallel-lane refactor; no cohort handover. Trailer optional. If added: note which variant (`?` vs `.map_err`) was picked, citing the enclosing function shape.
