# polish-1 critical-bugs — plan

**Base:** `governance-v0` @ `08065e1a1` (PR #46 merge)
**Branch:** `polish/critical-bugs` (to cut)
**Target:** ship 4 production-code fixes in one PR. Est. ~3–4 hours including validation.
**Sequence (per T1 + T2 + owner comments on each issue):** #35 + #48(a) → #48(b) → #34 → #33 → validate.

---

## Scope (post-triage)

### #48 (a) + #35 — `governance_log::append` atomicity

**Location:** `crates/db_schema/src/source/governance/governance_log.rs:165-210`

**Current shape:** get_conn → INSERT (line 181-185) → compute signature → UPDATE (line 195-198). If UPDATE fails, orphan row with `signature = NULL`. The NOTIFY trigger already fires `AFTER UPDATE OF signature` (fixed in migration `2026-04-20-000100`), so #35 is implicitly resolved once the two writes are atomic — subscribers will still observe only signed rows.

**Fix:** wrap INSERT+UPDATE in `conn.run_transaction(...)`. Safe: callers (e.g. `federation_outbox.rs:187-193`, `admin_assign_jury.rs:165-174`) invoke `append` from inside their own outer `run_transaction` already; nested `run_transaction` on a diesel-async conn becomes a SAVEPOINT, not a new tx (documented in `sponsor_liability.rs:34-35` — but we're NOT nesting from the caller; `append`'s own tx would be the first tx when called from the bare-pool context).

**Caller audit:** confirm nobody passes `append` a conn that's already in a `run_transaction` closure and expects the outer tx to cover it. Per T2: all callers run `append` inside their outer tx via `&mut conn.into()` reborrow. If `append` wraps INSERT+UPDATE in its own tx while already inside one, diesel-async will SAVEPOINT — safe. But this is a real behavioural change; need e2e-run to confirm.

**Acceptance:** `governance_log_hash_chain_holds` still passes.

### #48 (b) — causal ordering in `submit_jury_vote`

**Location:** `crates/api/api/src/governance/submit_jury_vote.rs:466-488`

**Fix:** move the `governance_log::append("case_decided", ...)` block (lines 478-488) ABOVE the `if matches!(map_decision_to_sanction(...), Some((SanctionScope::FederatedRecommendation, _)))` federation-send block (lines 466-476). Both already share the outer `run_transaction`; moving doesn't change atomicity.

**Acceptance:** `governance_log_hash_chain_holds` still passes. Manually inspect e2e chain dump for `sanction_notice_round_trip` — `case_decided` entry_kind must precede `federation_sanction_sent` by hash-link order.

### #48 (c) — hidden `get_or_create` write (DEFER)

**Location:** `crates/api/api/src/governance/federation_outbox.rs:159-160`

Per T2: no `get_or_require` variant exists; current `get_or_create` is idempotent under unique constraint. Cost of introducing read-only variant is low but not strictly required. **Defer to polish-N or post-tag.** Add one-line comment at call site clarifying invariant instead.

### #34 — `request_appeal` guard inversion

**Location:** `crates/api/api_crud/src/governance/request_appeal.rs:104-107`

**Current shape:** `if case.closed_at.is_some() { return NotFound }`. Comment says "closed_at must still be null" — but `submit_jury_vote.rs:327-332` always stamps `closed_at = decided_at + 7 days` when flipping to `Decided`. So every Decided case is unreachable by this guard.

**Fix:** compare against `now()`:
```rust
let now_ts = diesel::dsl::now;  // already in use — or use chrono::Utc::now()
let within_window = case.closed_at.map(|c| c > chrono::Utc::now()).unwrap_or(false);
if !within_window {
  return Err(LemmyErrorType::NotFound.into());
}
```
Semantics: appeal allowed iff `closed_at > now()`. NULL `closed_at` on a `Decided` case is a data error — reject.

**Acceptance:** add e2e coverage: decide a case → immediately call request_appeal → expect 200 (or whatever the success shape is). Mirror `report_to_modlog_golden_path`.

### #33 — declining juror self-replacement guard

**Location:** `crates/api/api/src/governance/decline_jury_assignment.rs:106-143`

Per T2 reading: flow is (i) flip declining juror's assignment row to `Declined` (line 106-112); (ii) load `current_assignees` filtering `status != Declined AND status != Expired` (line 132-138); (iii) call `select_eligible_jurors(conn, &case, Some(&current_assignees), ...)` which excludes `current_assignees` and case-principals from the pool.

T2 says: declining juror is naturally excluded because their row was just flipped to `Declined`, so they're not in `current_assignees`.

T1 says: PR #10 CodeRabbit flagged this at decline_jury_assignment.rs:132-138 as a real 5-LOC bug; fix is explicit self-exclude + e2e extension.

**Who's right?** The caller_id is the declining juror; `select_eligible_jurors` excludes `case.target_person_id`, `case.creator_id`, and caller-provided `exclude_person_ids`. If `current_assignees` correctly excludes the declining juror, they're excluded. BUT: `select_eligible_jurors` may filter on raw person_id regardless of case-history; if the pool query is `SELECT p.id FROM person p WHERE p.id != ALL($2) AND <other filters>`, and `$2` = `current_assignees` which excludes the just-declined, then the declining juror IS a candidate for *their own replacement*.

**The bug:** the declining juror is not in `current_assignees` (their status is `Declined`, not `Pending`/`Accepted`), so `select_eligible_jurors` can legitimately pick them back.

**Fix:** push the declining juror's `caller_id` onto the exclude list before calling `select_eligible_jurors`:
```rust
let mut exclude_list = current_assignees.clone();
exclude_list.push(caller_id);
let replacements = select_eligible_jurors(conn, &case, Some(&exclude_list), &mut cache).await?;
```

**Acceptance:** extend `ineligible_user_cannot_be_picked_for_jury` or add new e2e: assign juror A, A declines, trigger replacement selection, assert A is not in the new panel.

---

## PR structure

Commit list (one per issue, in sequence):

1. `fix(governance-log): wrap append INSERT+UPDATE in run_transaction (GH #48 #35)` — atomicity + NOTIFY timing (shared fix)
2. `fix(governance): log case_decided before federation_sanction_sent (GH #48)` — causal ordering
3. `fix(governance): appeal window compares closed_at to now (GH #34)` — guard inversion
4. `fix(governance): exclude declining juror from replacement pool (GH #33)` — self-exclude
5. `test(governance): e2e coverage for #34 appeal + #33 self-exclude` — regression tests

#48(c) deferred — commit note explains.

---

## Validation sequence

After all 4 fixes + tests:

1. `cargo check --workspace --features full`
2. `cargo clippy --workspace --no-deps --features full -- -D warnings`
3. `cargo test --test e2e --no-run -p lemmy_server`
4. `cargo test --test e2e -p lemmy_server`

Expect 14 → 16 passed (plus 3 ignored), 0 failed.

---

## Execution plan

- Impl1 (me): work all 5 commits in sequence on `polish/critical-bugs` in the advisor worktree
- Single-impl — Phase 6 Bucket C coordination overhead isn't needed for a 4-file, 5-commit PR
- DQ for anything surprising during impl

---

## Next steps after polish-1 merges

- polish-3 docs sweep (~40 lines across 5 files; immediate parallel work candidate)
- polish-2 Bucket C residual (#2p-7 TOCTOU + #2p-2/#2p-3 defensive)
- polish-N individual items (#36, #47, #37, #38, etc.)
- polish-tag v0.0.0
