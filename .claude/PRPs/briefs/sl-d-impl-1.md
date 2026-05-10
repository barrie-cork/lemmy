---
phase: v1-SL-d
role: impl-task
task: 1
brief_n: 1
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d task 1 compute/fire split + grace_window_for_severity — see .claude/PRPs/briefs/sl-d-impl-1.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d task 1 apply_sponsor_liability split + grace helper`

## §2 Scope

In `crates/api/api/src/governance/sponsor_liability.rs` (currently 358 lines):

1. ADD `pub(crate) struct SponsorDelta { ... }` after the `LiabilitySeverity` decl. Fields per plan §10.1. Derives: `Debug, Clone, PartialEq` (NOT Eq — f64 doesn't impl Eq).
2. ADD `fn liability_severity_from_case_severity(severity: CaseSeverity) -> LiabilitySeverity` — exhaustive match, no `_ =>` arm, per ADR-013.
3. SPLIT `apply_sponsor_liability`'s body (lines 142+) into three functions:
   - `pub(crate) async fn compute_sponsor_liability(conn, target_person_id, case_id, community_id, action, cache) -> LemmyResult<Vec<SponsorDelta>>` — pure read, ZERO DB writes.
   - `pub(crate) async fn fire_sponsor_liability(conn, target_person_id, case_id, community_id, deltas: &[SponsorDelta], cache) -> LemmyResult<usize>` — performs the writes.
   - `pub(crate) async fn apply_sponsor_liability(conn, target_person_id, case_id, community_id, action, cache) -> LemmyResult<usize>` — thin wrapper: `compute + fire`, signature BYTE-IDENTICAL to v0.
4. ADD `pub(crate) async fn grace_window_for_severity(severity: CaseSeverity, cache: &mut ConfigCache, conn: &mut AsyncPgConnection) -> LemmyResult<chrono::Duration>` — reads `liability.grace_window_<bucket>_hours` at `Scope::Instance`; returns `chrono::Duration::hours(value)`.

Do NOT modify any other file. No commit to `crates/server/tests/e2e.rs`, no migration, no route registration.

### 2.1 §G4 CANONICAL CASE OVERRIDE

No e2e test authorship in Task 1. The `LemmyResult<()>` / Case A mandate applies to Tasks 3-6 (e2e tests). Task 1 is a pure Rust refactor — no test fn return type decision required here.

## §3 Required reading

- `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` §4.1 (SponsorDelta struct + split-line), §10.1 (MIRROR: v0 body structure), §10.2 (pure-fn/write-fn split from sponsor_liability_grace.rs), §13 Task 1 (IMPLEMENT steps + GOTCHAs)
- `crates/api/api/src/governance/sponsor_liability.rs` (full file, 358 lines) — the MIRROR ref; read before writing
- `crates/api/api/src/governance/sponsor_liability_grace.rs:90-540` — canonical compute/fire split pattern to mirror
- `crates/api/api/src/governance/config.rs:925-945` — grace-window default consts
- `crates/db_schema_file/src/enums.rs` — `CaseSeverity` variants (confirm exhaustive match set)
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — `fire_sponsor_liability` performs 2+ writes; must stay inside existing transaction context (wrapper passes through; no new transaction)
- `.claude/lessons/feedback_clippy_test_style.md` — Lemmy clippy denies; avoid `unwrap`/`expect` in new code
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — if first workspace-check clippy flags anything, recheck after fix

## §3a Handover from prior cohort

(none — Task 1 is the first impl task; Task 0 was verification-only with no commit)

## §4 Constraints

- **Files:** only `crates/api/api/src/governance/sponsor_liability.rs`. No other files.
- **Wrapper signature byte-identical:** `pub(crate) async fn apply_sponsor_liability(conn: &mut AsyncPgConnection, target_person_id: PersonId, case_id: ModerationCaseId, community_id: Option<CommunityId>, action: SanctionAction, cache: &mut ConfigCache) -> LemmyResult<usize>` — same as v0 line 142. Any change to argument order or types = STOP.
- **Compute purity:** zero DB writes inside `compute_sponsor_liability`'s body. At task-end, run: `rg -nE 'insert_into|update\(|append\(|persist' crates/api/api/src/governance/sponsor_liability.rs` — zero matches inside compute body.
- **Wrapper signature grep:** at task-end, run `git diff governance-v0..HEAD -- crates/api/api/src/governance/sponsor_liability.rs > /tmp/sl-d-task1-diff.log && rg 'pub\(crate\) async fn apply_sponsor_liability' /tmp/sl-d-task1-diff.log` — the wrapper fn signature line must be a context line (no leading `+` or `-`).
- **ADR-013:** `liability_severity_from_case_severity` must be exhaustive — no `_ =>` arm.
- **Instance scope:** `grace_window_for_severity` reads `Scope::Instance` only (DQ #178 LOCKED).
- **DQ mid-task push:** if a DQ blocker is filed, commit + push immediately per decision-queue.md.
- **Shape G:** after implementing and verifying, push the worker branch; write a `kind: "validate-pending"` DQ entry with `workflow_run_id` from the triggered `cargo-validate-workspace.yml` run.
- **COMMIT MESSAGE:** `refactor(v1-SL-d): split apply_sponsor_liability into compute + fire + wrapper, add grace_window_for_severity (task 1)`
