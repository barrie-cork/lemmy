---
phase: v1-SL-d
role: impl-task
task: 2
brief_n: 1
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d task 2 submit_jury_vote Decided→SponsorLiabilityPending transition — see .claude/PRPs/briefs/sl-d-impl-2.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d task 2 submit_jury_vote mutation + grace-window snapshot`

## §2 Scope

In `crates/api/api/src/governance/submit_jury_vote.rs`:

1. UPDATE imports (lines 42-48) — add `ENTRY_KIND_SPONSOR_LIABILITY_PENDING`.
2. REPLACE v0 step 8.5 block (lines 470-480) — invoke `compute_sponsor_liability` instead of `apply_sponsor_liability`; branch on `Vec::is_empty()` to set `path_kind` (Pending / Decided); collect `grace_expires_at` + `deltas_for_pending`. Define `enum SLDPathKind { Pending, Decided }` (derive PartialEq).
3. MODIFY case UPDATE (lines 491-498) — branch on path_kind: `status` = SponsorLiabilityPending vs Decided; `grace_expires_at` = Some(t) on Pending, else None.
4. ADD step 9b on Pending path — emit `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` governance_log entry per plan §10.4 payload schema. Position: AFTER case UPDATE, BEFORE step 10. Payload: `target_pseudonym` + `sponsors_pseudonyms` (per-sponsor via `actor_pseudonym_helper::get_or_create`) + `severity` (from `liability_severity_from_case_severity` mapping). NEVER raw `*_id.0` per ADR-015.
5. GUARD steps 10-12 (lines 500-607) on `path_kind == SLDPathKind::Decided`. On Pending these are deferred to SL-c scheduler.
6. GUARD step 12.5 (federation outbox lines 661-667) on `path_kind == SLDPathKind::Decided`.
7. PRESERVE step 9 (`appeal_window_expires_at` UPDATE + `case_decided` log) — fires on BOTH paths. NO CHANGE.
8. REMOVE the JM-c TODO comment block at lines 456-469.
9. ADR-013 sweep at task-end:
   ```bash
   rg -nE 'match.*case\.status\b|match.*CaseStatus|match.*case_row\.status\b|match.*case_row\.severity\b|match.*CaseSeverity' crates/api/api/src/governance/submit_jury_vote.rs
   ```
   For each match, read 14-20 lines of context; assert no `_ =>` arms in any new match site SL-d introduces.

Do NOT modify any other file. No migration, no new ENTRY_KIND_* consts, no route registration.

### 2.1 §G4 CANONICAL CASE OVERRIDE

No e2e test authorship in Task 2. The `LemmyResult<()>` / Case A mandate applies to Tasks 3-6 (e2e tests). Task 2 is a pure Rust mutation — no test fn return type decision required here.

## §3 Required reading

- `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` §10.3 (process_vote outer-transaction shape), §10.4 (sponsor_liability_pending payload schema), §13 Task 2 (IMPLEMENT steps + GOTCHAs)
- `crates/api/api/src/governance/submit_jury_vote.rs` (full file, ~700 lines) — MIRROR ref; read before writing
- `crates/api/api/src/governance/sponsor_liability.rs` — `compute_sponsor_liability` + `liability_severity_from_case_severity` + `grace_window_for_severity` signatures (from Task 1)
- `crates/api/api/src/governance/sponsor_liability_grace.rs:90-200` — canonical path_kind branch pattern to mirror
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — SL-d mutation lives inside existing `run_transaction` at line 140; NO new `run_transaction`
- `.claude/lessons/feedback_clippy_test_style.md` — Lemmy clippy denies; avoid `unwrap`/`expect` in new code
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — if workspace-check clippy flags anything, recheck after fix

## §3a Handover from prior cohort

Task 1 final clean commit: `3302ceb75` (fix-impl-2, `#[expect(dead_code)]` on grace_window_for_severity) + `fix-impl-3` removed spurious `#[expect]` from `liability_severity_from_case_severity`.
DQ #191 result: `pass` — workspace-check clean on fix-impl-3.
Phase-v1-SL-d tip after ci-watcher #193 merge: confirmed clean.

Key Task 1 outputs available for import:
- `compute_sponsor_liability(conn, target_person_id, case_id, community_id, action, cache) -> LemmyResult<Vec<SponsorDelta>>`
- `grace_window_for_severity(severity: CaseSeverity, cache, conn) -> LemmyResult<chrono::Duration>`
- `liability_severity_from_case_severity(severity: CaseSeverity) -> LiabilitySeverity` (private fn, not importable — use `grace_window_for_severity` for the mapping)
- `SponsorDelta` struct (pub(crate))

## §4 Constraints

- **Files:** only `crates/api/api/src/governance/submit_jury_vote.rs`. No other files.
- **No new ENTRY_KIND_*** — import existing `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` from the api shim. At task-end: `git diff governance-v0..HEAD -- crates/db_schema/src/source/governance/governance_log.rs crates/api/api/src/governance/governance_log.rs` — both diffs must be empty.
- **Single transaction:** mutation lives inside existing `run_transaction` at line 140. No nested `run_transaction`. Per `feedback_multi_write_handlers_need_transactions.md`.
- **Severity snapshot:** `grace_window_for_severity` takes `case_row.severity` (from `moderation_case.severity` column). Do NOT re-derive from config or sanction at fire-time.
- **JM-c TODO removal:** the TODO block at lines 456-469 must be removed (task 0 probe 8 confirmed it exists; task 2 discharges it).
- **ADR-015 pseudonym discipline:** no raw `PersonId.0` in governance_log payloads. Use `actor_pseudonym_helper::get_or_create`.
- **ADR-013:** no `_ =>` wildcard in any new match arms on CaseStatus / CaseSeverity.
- **Shape G:** after implementing and verifying, push the worker branch; write a `kind: "validate-pending"` DQ entry with `workflow_run_id` from the triggered `cargo-validate-workspace.yml` run.
- **DQ mid-task push:** if a DQ blocker is filed, commit + push immediately per decision-queue.md.
- **COMMIT MESSAGE:** `feat(v1-SL-d): submit_jury_vote Decided→SponsorLiabilityPending transition + grace-window snapshot + deferred writes (task 2)`
