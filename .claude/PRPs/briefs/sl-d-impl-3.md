---
phase: v1-SL-d
role: impl-task
task: 3
brief_n: 1
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d task 3 e2e test #1 — Decided→SponsorLiabilityPending transition — see .claude/PRPs/briefs/sl-d-impl-3.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d task 3 e2e test #1 SponsorLiabilityPending transition + mod v1_sl_d_fixtures shell`

## §2 Scope

In `crates/server/tests/e2e.rs`, append a NEW `mod v1_sl_d_fixtures` block AFTER the closing `}` of `mod v1_sl_c_fixtures` (near line 12,819).

**Anchor identification at task-start:**
```bash
grep -n '^mod v1_' crates/server/tests/e2e.rs | tail -1
# EXPECT: mod v1_sl_c_fixtures at line ~12819
```

**Content (ONE Edit at file end):**

1. Open `mod v1_sl_d_fixtures { use super::*; ... }` block.
2. Add helper `seed_target_with_sureties(conn, sponsor_count) -> LemmyResult<(PersonId, Vec<PersonId>)>` per plan §10.5 skeleton, body mirrors `mod v1_sl_b_fixtures` helpers.
3. Add first test fn:

```rust
#[tokio::test]
async fn submit_jury_vote_transitions_to_pending_for_liability_bearing_sponsored_case() -> LemmyResult<()>
```

**Test body:**
- **Setup:** bootstrap LemmyContext with Postgres testcontainer. Seed sponsee + 2 sponsors with active sureties. Seed `moderation_case` in `InReview` status with `target_person_id = sponsee`, `severity = CaseSeverity::Severe`. Seat a jury panel (mirror existing JM-c/SL-b test setup). Pre-seed N-1 jury votes for a liability-bearing sanction (e.g. `SanctionAction::InstanceSuspension`).
- **Drive:** invoke `submit_jury_vote` handler with the Nth (final) vote.
- **Assert:**
  - `response.case_decided == true`
  - `response.decision == Some(JuryDecision::InstanceSuspension)` (or matching variant)
  - `moderation_case.status == CaseStatus::SponsorLiabilityPending` (NOT Decided)
  - `moderation_case.grace_expires_at` is `Some(t)` where `|t - (now + Duration::hours(168))| < 5 seconds` (Severe → 168h)
  - 0 `reputation_event` rows for sponsors (compute is pure, writes deferred)
  - 1 `governance_log` row with `entry_kind == "case_decided"`
  - 1 `governance_log` row with `entry_kind == "sponsor_liability_pending"`
  - `sponsor_liability_pending` payload: `case_id` matches; `target_pseudonym` is a String; `target_pseudonym != format!("{}", sponsee.0)` (ADR-015 — Watchpoint #10); `severity == "severe"`; `grace_expires_at` present; `sponsors_pseudonyms` is array of length 2; each entry is a String
  - 1 `governance_log` row with `entry_kind == "sanction_created"`
  - `moderation_case.appeal_window_expires_at` is `Some(t)` (fires on BOTH paths)

Do NOT add Tests #2-#4 in this task. ONE Edit, ONE test fn.

### 2.1 §G4 CANONICAL CASE OVERRIDE — MANDATORY

**Case A (LemmyResult<()>) applies throughout `mod v1_sl_d_fixtures`.** All helper signatures: `-> LemmyResult<T>`. All test fn signatures: `-> LemmyResult<()>`. Bare `?` propagation. NO `Box<dyn Error>` outer. NO `.map_err` bridges. This is the canonical sibling shape from `mod v1_sl_b_fixtures` (e2e.rs:11001-11924). The 3-cycle SL-c-2 catch-fire was caused by non-uniformity; SL-d avoids by following Case A throughout.

## §3 Required reading

- `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` §10.4 (sponsor_liability_pending payload schema), §10.5 (fixture mod shape), §10.6 (governance_log row count assertion pattern), §13 Task 3 (IMPLEMENT steps + GOTCHAs)
- `crates/server/tests/e2e.rs:11001-11924` — `mod v1_sl_b_fixtures` canonical Case A sibling; read IN FULL at task-start; mirror `LemmyResult<T>` outer pattern verbatim
- `crates/server/tests/e2e.rs:907` — `governance_log_hash_chain_holds` test; governance_log query pattern
- `crates/api/api/src/governance/submit_jury_vote.rs` — Task 2 handler (Decided→SponsorLiabilityPending path, grace_expires_at computation)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Case A** is canonical; helpers `LemmyResult<T>`, test fn `LemmyResult<()>`, bare `?` — mandatory read, mandatory apply
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` + `DbPool::Conn` for e2e; mandatory read
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — e2e.rs is 12,819+ lines; ONLY anchor-Edit at file end; never bulk-edit; ONE test per task
- `.claude/lessons/feedback_clippy_test_style.md` — Lemmy clippy denies; no `unwrap`/`expect` in new code
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — if workspace-check clippy flags anything, recheck after fix

## §3a Handover from prior cohort

Task 2 final clean commit: fix-impl-4 (`ok_or_else` on Pending path `.expect()` calls).
DQ #193 result: `pass` — workspace-check clean on fix-impl-4.
Tasks 1+2 validated. Phase-v1-SL-d tip clean.

Key symbols available:
- `compute_sponsor_liability` in `sponsor_liability.rs`
- `grace_window_for_severity` in `sponsor_liability.rs`
- `CaseStatus::SponsorLiabilityPending` variant (shipped in SL-a)
- `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` const (shipped in SL-a)
- `mod v1_sl_c_fixtures` ends near line 12,819 — new mod appends after

## §4 Constraints

- **Files:** only `crates/server/tests/e2e.rs`. No other files.
- **ONE Edit at file end** — anchor after closing `}` of `mod v1_sl_c_fixtures`. No bulk edits. No re-reading and re-editing the same file multiple times.
- **Case A canonical shape** — `LemmyResult<()>` test fn, `LemmyResult<T>` helpers, bare `?`. Zero `Box<dyn Error>`. Zero `.map_err`. This is a HARD constraint — any deviation is a process miss.
- **No new ENTRY_KIND_*** — import existing consts only.
- **Governance_log assertions** — use diesel table queries, not raw SQL.
- **Shape G:** after committing, push the worker branch; write a `kind: "validate-pending"` DQ entry with `workflow_run_id` from the triggered `cargo-validate-workspace.yml` run.
- **DQ mid-task push:** if a DQ blocker is filed, commit + push immediately per decision-queue.md.
- **COMMIT MESSAGE:** `test(v1-SL-d): e2e test #1 — Decided → SponsorLiabilityPending transition + mod v1_sl_d_fixtures shell (task 3)`
