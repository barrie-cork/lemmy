# Brief: impl-task 3 — v1-RT-r3 flag-bad-faith admin endpoint (DTOs + handler + route wire)

## 1. Role + dispatch

`[role:impl-task] v1-RT-r3 task 3 — flag-bad-faith admin endpoint — see .claude/PRPs/briefs/v1-RT-r3-impl-3.md`

## 2. Scope

Implement §13 Task 3 of `.claude/PRPs/plans/v1-RT-r3.plan.md` verbatim.

**FILES (per plan §13 Task 3 FILES yaml):**

- creates: []
- modifies: `crates/api/api_common/src/governance.rs` (new request/response DTOs)
- modifies: `crates/api/api/src/governance/admin_emergency_remove.rs` (new pub handler + inner fn + private emit helper)
- modifies: `crates/api/routes/src/lib.rs` (import + scope("/emergency-remove") wire)
- requires: [] (Task 0 already done; no inter-task dependencies)

**IMPLEMENT:** Follow plan §13 Task 3 IMPLEMENT blocks (3 files) verbatim:
- **File 1 (`api_common/src/governance.rs`):** Append two new DTOs per §10.7 verbatim, alongside `AdminCloseCase` / `AdminCloseCaseResponse` at lines 191-203.
- **File 2 (`admin_emergency_remove.rs`):** Append `flag_bad_faith_emergency_report` (pub handler outer; `is_admin` + `run_transaction` wrap) + `process_flag_bad_faith` (inner; status assertion + emits) + `emit_reputation_event_local` (file-private helper mirroring `submit_jury_vote.rs:968-994` shape verbatim including `.on_conflict_do_nothing()` per §10.2). Add imports per plan §13 Task 3 IMPLEMENT file 2.
- **File 3 (`routes/src/lib.rs`):** Append `admin_emergency_remove::flag_bad_faith_emergency_report` to existing `governance::{...}` import (lines 31-49, alphabetical between `admin_dashboard_html::{...}` and `admin_reputation_stats`). Add `.service(scope("/emergency-remove").route("/flag-bad-faith", post().to(flag_bad_faith_emergency_report))),` inside existing `scope("/admin")` at line 493.

**VALIDATE (story-checkpoint feeds §16a Story 4):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r3-task3-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-task3-check.log
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r3-task3-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-RT-r3-task3-clippy.log
# EXPECT: exit 0
```

```bash
# R7 test-target compile (Task 3 adds pub types in api_common)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-RT-r3-task3-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-task3-test-no-run.log
# EXPECT: exit 0
```

**Note:** Shape G is SUSPENDED per DQ #229 — cargo runs on the laptop. Junior subagent must NOT execute VALIDATE locally.

**Post-validate:** Write `kind: "validate-pending-laptop"` DQ entry per `advisor-orchestrator.md` §5.2.
Required fields:
- `commands`: the 3 VALIDATE bash blocks above (verbatim including `--workspace --features full`)
- `branch`: `"phase-v1-RT-r3"`
- `phase_task`: `3`

Generate id via `bash scripts/brehon/dq-v3-new-entry.sh`. Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Commit + push DQ to worker branch.

## 3. Required reading

- `.claude/PRPs/plans/v1-RT-r3.plan.md` §13 Task 3 (authoritative IMPLEMENT + MIRROR + GOTCHA + VALIDATE)
- `.claude/PRPs/plans/v1-RT-r3.plan.md` §10.5 (handler + inner fn + emit_reputation_event_local verbatim)
- `.claude/PRPs/plans/v1-RT-r3.plan.md` §10.7 (DTO definitions verbatim)
- `crates/api/api/src/governance/admin_emergency_remove.rs:75-101` (`emergency_remove_open_case` outer wrap)
- `crates/api/api/src/governance/admin_config.rs:1094-1142` (`is_admin` + `LocalUserView` capability-check pattern)
- `crates/api/api_common/src/governance.rs:191-203` (`AdminCloseCase` + `AdminCloseCaseResponse` shape — sibling DTOs with `#[cfg_attr(feature = "ts-rs", ...)]`)
- `crates/api/routes/src/lib.rs:503-518` (existing nested scopes under `/admin`)
- `crates/api/api/src/governance/submit_jury_vote.rs:968-994` (`emit_reputation_event` to mirror file-private helper verbatim)
- `crates/api/api/src/governance/submit_jury_vote.rs:266-285` (`ModerationCase::status` comparison pattern)
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — `flag_bad_faith_emergency_report` reuses existing `run_transaction` shape from `admin_emergency_remove.rs:88-100`
- `.claude/lessons/feedback_features_full_workspace_only.md` + `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — cargo always uses `--workspace --features full`
- `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — Lemmy workspace test-style
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md` — Shape G suspended; cargo runs on laptop
- `.claude/rules/decision-queue.md` (DQ schema-v3; impl-task writes `kind: "validate-pending-laptop"`; ALWAYS use `dq-v3-new-entry.sh`)
- ADR-013 (admin-driven posture) + ADR-015 (pseudonymisation)

## 4. Constraints

- **No edits outside the FILES yaml block.** Touching any other file is a process breach.
- **One commit** — `feat(governance): flag-bad-faith admin endpoint (task 3)`.
- **DTO `#[cfg_attr(feature = "ts-rs", ...)]` lines mandatory** — without them, the `ts-rs` feature's typescript-generation step misses the new types and breaks a downstream CR finding (per plan §13 Task 3 GOTCHA).
- **`dedupe_key` for source 4b contains NO `reporter_pseudonym` segment** — `dedupe_key = format!("evidence_bad_faith:{}", case_id.0)` (one bad-faith flag per case per ADR-013).
- **`actor_pseudonym` on governance_log entry is `admin_pseudonym`, NOT reporter** — reporter pseudonym appears only in payload JSON (per ADR-015 + brief §4 "real admin attribution").
- **`ModerationCase::status != CaseStatus::EmergencyRemove`** — mirror existing comparisons at `submit_jury_vote.rs:266-285`.
- **HTTP status mapping decision:** if no clean `LemmyErrorType` variant maps to 400 for "non-EmergencyRemove case", fall back to `LemmyErrorType::Unknown` with a descriptive message. **Document the mapping decision in the Task 3 commit body** (e2e test in Task 4 asserts on the actual mapping; document so Task 4's assertion can match).
- **URL must be exactly `POST /api/v4/governance/admin/emergency-remove/flag-bad-faith`** per DQ a3d0e9941441-024 (PRD literal).
- **No new file under `crates/api/api/src/governance/`** — extend `admin_emergency_remove.rs` per DQ a3d0e9941441-019.
- **No clippy `#[allow]`** — use `#[expect(...)]` only after verifying lint is intentional.
- **Mid-task DQ push** — raise `kind: "blocker"` immediately if you find:
  - Unexpected sibling DTO shape (no `#[cfg_attr(feature = "ts-rs", ...)]` on `AdminCloseCase` would be surprising)
  - Existing `scope("/admin")` doesn't have `.service(scope(...))` children at line 503-518
  - `is_admin` signature drift from `admin_config.rs:1094-1142`
- **LESSON-trailer convention** — end commit body with `LESSON:` if a durable pattern emerges (e.g. HTTP status mapping discovery worth recording).

## 5. Forbidden-window check (advisor pre-queue)

Per `advisor-orchestrator.md` §5.1: Shape G suspended → forbidden-window check is binding for laptop validate-pending-laptop runs.

Current UTC at queue: Mon 19:08 UTC (primary window 16:00–02:30 — OK).

## 6. Context

- Phase: v1-RT-r3
- Plan: `.claude/PRPs/plans/v1-RT-r3.plan.md` (on trunk + phase branch)
- Phase branch: `phase-v1-RT-r3` @ `0401709e0`
- Base branch for this task: `phase-v1-RT-r3`
- Cohort 2 — Tasks 1 + 2 + 3 dispatched in parallel (`[P]` per plan §13); YAML overlap check confirmed disjoint file sets.
- Prior cohort handover (Task 0): all 19 probes pass; api_common/src/governance.rs + admin_emergency_remove.rs + routes/src/lib.rs all unchanged on phase branch.
