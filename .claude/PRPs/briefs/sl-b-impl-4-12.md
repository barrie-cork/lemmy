---
role: advisor-direct
plan_tasks: 4-12
phase: v1-SL-b
created: 2026-05-04
related_plan: .claude/PRPs/plans/v1-sponsor-liability-b.plan.md §13 Tasks 4-12
---

# Brief — v1-SL-b Tasks 4-12 — e2e tests #1-9 (advisor-direct, no Junior)

## 1. Role + dispatch

**`advisor-direct`** — authored by the Brehon advisor session, NOT
queued as a Junior task. Per PMD #117 (homeserver `pattern_*`),
Junior workers reliably hang on Edit calls into
`crates/server/tests/e2e.rs` (10976 lines pre-edit). The lesson is
not mirrored in `brehon-fork/.claude/lessons/` so impl-task
subagents wouldn't see the warning. Authoring this in the persistent
advisor session bypasses the hang risk entirely.

## 2. Scope (combined Tasks 4-12)

**Produced (single commit):**

1. New `mod v1_sl_b_fixtures` appended to
   `crates/server/tests/e2e.rs` (after line 10976) containing:
   - 7 helper fns: `seed_endorsement_active`, `seed_pending_case`,
     `read_endorsement_revoked_at`, `read_surety_revoked_at`,
     `read_case_status_and_escape`, `count_log_entries`,
     `read_log_payload`, `read_snapshot_calculated_at`.
   - 9 `#[tokio::test]` async fns — one per plan §13 Task 4-12,
     covering: self-revoke success, admin-revoke under threshold,
     re-revoke idempotency, capability rejection (NotFound),
     reason validation (empty / whitespace-only), rate-limit +
     admin bypass, single-sponsor severance, multi-sponsor
     any_revocation default rule, no-pending-case non-severance.

2. Push to `junior/advisor-sl-b-tests-4-12` to fire
   `cargo-validate-workspace.yml` (per
   `feedback_advisor_phase_branch_push_skips_workspace_check`).

3. Validate-pending DQ entry (kind: "validate-pending", from:
   "advisor", workflow_run_id from gh run list).

4. Queue ci-watcher Junior task to poll + mutate DQ.

**Not in this commit:** plan §13 Task 13 (Retro) — happens
post-Phase-2-e2e-pass per advisor-orchestrator stage-shape.

## 3. Reading consulted

- `crates/api/api_crud/src/governance/revoke_endorsement.rs` —
  function signature, rate-limit logic, governance_log payload
  emission, error types (TooManyRequests not RateLimitError).
- `crates/api/api_common/src/governance.rs` — RevokeEndorsement
  + RevokeEndorsementResponse DTO shape.
- `crates/db_schema/src/source/governance/{endorsement, surety,
  moderation_case, reputation_snapshot}.rs` — InsertForm shapes,
  field availability (no `revoked_at` in EndorsementInsertForm),
  reputation_snapshot's freshness signal (calculated_at, no
  stale_at column).
- `crates/db_schema_file/src/enums.rs` — CaseStatus variants
  (SponsorLiabilityPending → SponsorLiabilityEscaped).
- `crates/db_schema_file/src/schema.rs` lines 471-481 —
  governance_log table columns.
- `crates/server/tests/e2e.rs:9786+` mod v1_jm_e_fixtures —
  fixture-mod shape precedent.
- `crates/server/tests/e2e.rs:813+` governance_fixtures::seed_user
  / seed_community / bootstrap helpers.
- `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` §10.3-10.7,
  §13 Tasks 4-12, §15 DoD.

## 4. Plan-§13 deviations (deliberate)

| Plan task | Plan says | Handler emits | Test asserts |
|---|---|---|---|
| 9 (rate-limit) | `LemmyErrorType::RateLimitError` | `LemmyErrorType::TooManyRequests` | `TooManyRequests` (matches handler) |
| 4 (snapshots) | "non-stale (recompute fired)" | `calculated_at` updated | `calculated_at >= test_start` |

Both deviations are corrections to the plan's PRD-side language;
the handler is canonical. A retro note will surface this for plan
template hygiene.

## 5. Validation gates

**Laptop pre-validation** (before push, per PMD #200):

```
scripts\brehon\cargo-check.bat --workspace --features full
scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings
scripts\brehon\cargo-test.bat -p lemmy_server --test e2e --no-run --features full
```

All three must exit 0 before pushing to junior/advisor-*.

**Push gate** (Shape G workspace-check): `cargo-validate-workspace`
fires automatically on push to `junior/*`.

**Validate-pending DQ** raised post-push, ci-watcher mutates on
workflow conclusion.

**Phase 2 e2e** — user gate after finalize-merge into phase-v1-SL-b
per `feedback_e2e_local_or_dispatch_user_choice.md`. Local
(`run_in_background` ~26 min, zero billed) vs dispatch
(`gh workflow run cargo-test-e2e.yml`, ~26 min billed).

## 6. Why one commit, not nine

- Eliminates 9× hang risk (PMD #117).
- Eliminates 9× daemon-finalize-variant risk (3 variants observed
  this phase: full-skip, merge-no-push, run-end-never-fired).
- Compresses ci-watcher round-trips 9 → 1.
- Single laptop pre-validation surfaces all clippy errors in one
  pass; with `-D warnings` the workflow halts at first error
  anyway, so per-task isolation buys nothing.
- Plan task granularity preserved in the commit body's enumerated
  list (one line per plan task → test name → assertion summary).
