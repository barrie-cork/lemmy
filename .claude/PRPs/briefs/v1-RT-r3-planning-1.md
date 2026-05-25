# Brief: planning — v1-RT-r3

## 1. Role + dispatch

`[role:planning] v1-RT-r3 — plan multi-source participation_consistency emitters — see .claude/PRPs/briefs/v1-RT-r3-planning-1.md`

## 2. Scope

Plan the implementation of **v1-RT-r3**: multi-source `participation_consistency` event emitters + the `flag-bad-faith` admin endpoint, behind the existing `feature.reputation_v1_decay_enabled` feature flag (calculator + bounds clamp already live from r2).

Per `.claude/PRPs/prds/v1-reputation-tuning.prd.md` §11 phase 3:

> "Add weekly activity cron + dormancy cron in `scheduled_tasks.rs`; add vote-outcome + evidence-quality emitters in `submit_jury_vote.rs`; add `flag-bad-faith` admin endpoint."

### 2.1 Pre-landed infrastructure (DO NOT re-seed)

Pre-flight verification (2026-05-25) confirms r1 already shipped the full schema + config + entry-kind surface r3 depends on. The planner MUST NOT propose seeds/migrations for any of the below; any plan that re-seeds is a scope violation.

- **Reputation event schema (r1 / Phase v1.r1):** `crates/db_schema/src/source/governance/reputation_event.rs:32-51` already carries `dedupe_key: Option<String>` + `source_event_type: ReputationEventSourceType`. The partial unique index on `dedupe_key WHERE dedupe_key IS NOT NULL` exists from r1's migration.
- **Config keys (r1 + v1-AD-a seed migrations):** all r3-relevant keys already in `crates/api/api/src/governance/config.rs`:
  - `deltas.participation_weekly_active` → const `DEFAULT_DELTAS_PARTICIPATION_WEEKLY_ACTIVE` at line 873 (default `1`)
  - `participation.dormancy_window_days` → line 874 (default `30`)
  - `deltas.participation_dormant` → line 875 (default `-2`)
  - `deltas.participation_juror_aligned` → line 1012 (default `1`)
  - `deltas.evidence_cited` → line 1019 (default `1`)
  - `deltas.evidence_bad_faith` → line 1020 (default `-1`)
  - `job.participation_interval_days` → line 1026 (default `7`)
  - Plus `participation.activity_threshold_comments`, `participation.lookback_days`, `participation.evidence_cited_rationale_threshold_chars` per PRD §8.
- **ENTRY_KIND constants (r1, pre-landed per registry):** at `crates/db_schema/src/source/governance/governance_log.rs:212-216`:
  - `ENTRY_KIND_PARTICIPATION_CRON_TICK = "participation_cron_tick"` (for both activity + dormancy crons per PRD §10)
  - `ENTRY_KIND_VOTE_OUTCOME_RECORDED = "vote_outcome_recorded"`
  - `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED = "evidence_quality_recorded"`
  - `ENTRY_KIND_ROLLUP_RECOMPUTED = "rollup_recomputed"` (r5-owned; do NOT emit in r3)
  - `ENTRY_KIND_DECAY_KNOB_CHANGED = "decay_knob_changed"` (r2-owned)
  - Re-exported via shim at `crates/api/api/src/governance/governance_log.rs`.
  - The registry at `.claude/rules/governance-log-entry-kind-registry.md` "v1-RT-r1 entry kinds" section already names r3 as the call-site landing phase for the three r3-owned kinds.
- **`feature.reputation_v1_decay_enabled` flag:** shipped by r1; gates r2 calculator. r3 emitters write to `reputation_event` regardless of flag (the events flow through r2's per-dimension calculator on the next 15-min snapshot tick; the calculator's flag check absorbs them transparently). Per PRD §11 ("r3 produces new `reputation_event` rows; r2's per-dimension calculator + bounds clamp absorb them transparently").
- **`submit_jury_vote.rs` transaction boundary:** the handler already wraps writes in `run_transaction` (line 139). Vote-outcome + evidence-quality emitters can fold into the same transaction; no new `feedback_multi_write_handlers_need_transactions.md` scope.

### 2.2 What r3 ships

**Five emitter paths + one admin endpoint:**

1. **Source 1 — Weekly activity cron** in `crates/routes/src/utils/scheduled_tasks.rs`. New `scheduler.every(CTimeUnits::days(N)).run(...)` block (cadence read from `job.participation_interval_days`, default 7), modeled after the existing reputation-snapshot block at line 201 with a matching `RunningGuard` pattern. For each community, for each user with ≥ `participation.activity_threshold_comments` non-deleted comments in the `participation.lookback_days` window, emit `+1 participation_consistency` reputation_event with `dedupe_key = format!("activity_cron:{community_id}:{person_id}:{iso_week}")` and `source_event_type = ParticipationCron`. ON CONFLICT (`dedupe_key`) DO NOTHING. Per-community-tx atomicity. Emit one `ENTRY_KIND_PARTICIPATION_CRON_TICK` governance_log entry per community per tick with `actor_pseudonym = system`.

2. **Source 2 — Weekly dormancy cron** in same `scheduled_tasks.rs` tick, registered *after* the activity cron block. For each `(person_id, community_id)` pair where the person has at least one prior `participation_consistency` event in that community AND has zero non-deleted comments in the `participation.dormancy_window_days` window, emit `−2 participation_consistency` (delta from `deltas.participation_dormant`) reputation_event with `dedupe_key = format!("dormancy_cron:{community_id}:{person_id}:{iso_week}")` and `source_event_type = DormancyCron`. Same dedupe + per-community-tx semantics. Same `ENTRY_KIND_PARTICIPATION_CRON_TICK` entry (one per community per tick, distinct dedupe namespace).

3. **Source 3 — Vote-outcome emitter (post-decision, embedded in `submit_jury_vote.rs`)** at `crates/api/api/src/governance/submit_jury_vote.rs`. Per PRD §5.3 source 3: when the case is decided (the `case_decided` branch already at line 282, inside the existing `run_transaction`), iterate the assembled panel; for each juror who voted with the majority, emit `+1 participation_consistency` event (delta from `deltas.participation_juror_aligned`, `source_event_type = VoteOutcome`, `dedupe_key = format!("vote_outcome:{case_id}:{juror_pseudonym}")`). Jurors who voted in the minority emit nothing (Brehon no-penalty-for-dissent per PRD §5.3 source 3). Emit one `ENTRY_KIND_VOTE_OUTCOME_RECORDED` governance_log entry per juror per decided case.

4. **Source 4a — Evidence-quality positive emitter (post-decision, same `submit_jury_vote.rs` `case_decided` branch)**. Heuristic: if the case has ≥1 `case_evidence` row authored by the reporter AND `jury_vote.rationale.length() ≥ participation.evidence_cited_rationale_threshold_chars` (default 256), emit `+1 reporting_accuracy` event for the reporter (delta from `deltas.evidence_cited`, `source_event_type = EvidenceQuality`, `dedupe_key = format!("evidence_cited:{case_id}:{reporter_pseudonym}")`). One `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED` entry. The planner MUST verify that `jury_vote.rationale` (or equivalent column) is a populated `TEXT` field in v0 schema; if it's `NULL`-by-default or empty-by-convention, Source 4a defers to v2 per PRD §5.3 N2 note — file a DQ.

5. **Source 4b — Evidence-quality bad-faith path + `flag-bad-faith` admin endpoint** — handler EXTENDS `crates/api/api/src/governance/admin_emergency_remove.rs` (per clarify DQ a3d0e9941441-019; do NOT create a new file). Add a second handler `flag_bad_faith_emergency_report` alongside the existing `emergency_remove_open_case` (330-line file; mirror its capability-check + case-load shape). Endpoint URL: `POST /api/v4/governance/admin/emergency-remove/flag-bad-faith { case_id }` — verbatim PRD literal (per clarify DQ a3d0e9941441-024). Instance-admin-only capability check; load the case; assert `case.status = EmergencyRemove`; emit `−1 reporting_accuracy` for the case's reporter (delta from `deltas.evidence_bad_faith`, `source_event_type = EvidenceQuality`, `dedupe_key = format!("evidence_bad_faith:{case_id}")`). One `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED` entry with `actor_pseudonym_id = <admin's id>` (real admin attribution; this is NOT a cron-batch system-attributed entry), payload distinguishing the bad-faith trigger from the cited-rationale trigger.

**Out of scope for r3:**

- No new DB migrations (r1 + v1-AD-* own the schema).
- No new `governance_config` keys (r1 + v1-AD-a seeded everything r3 needs).
- No new `ENTRY_KIND_*` constants (r1 pre-landed all five r3-relevant kinds).
- No `reputation_snapshot` calculator changes (r2 ships the per-dimension calculator + bounds clamp; r3 events flow through it transparently).
- No rollup cron (`ENTRY_KIND_ROLLUP_RECOMPUTED` is r5-owned).
- No `decay.*` knob admin handler (`ENTRY_KIND_DECAY_KNOB_CHANGED` is r2-owned).
- No `sponsor_allowlist` table or admin handlers (r4-owned).
- No 5 carry-forward CR fixes (#19, #20, #21, #22, #31 — r6-owned per PRD §11).

## 3. Required reading

- `.claude/PRPs/prds/v1-reputation-tuning.prd.md` §5.3 (multi-source participation events — all four sources defined) + §5.4 (gate strategies — NOT r3, but read so the planner doesn't accidentally pull them in) + §6 (acceptance criteria — only the participation/vote-outcome/evidence rows are r3) + §10 (security — admin auth + governance_log requirements) + §11 (phase row 3).
- `crates/routes/src/utils/scheduled_tasks.rs` lines 50-130 (RunningGuard + structural cron patterns) + lines 190-260 (the canonical `RunningGuard` + 15-min reputation-snapshot tick; MIRROR ref for the two new cron blocks).
- `crates/api/api/src/governance/submit_jury_vote.rs` (full file — understand `process_vote`, the `case_decided` branch at line 282, the existing `run_transaction` at line 139, the existing `emit_reputation_event` helper at line 968).
- `crates/db_schema/src/source/governance/reputation_event.rs` (full file — confirm `dedupe_key` + `source_event_type` columns + the `ReputationEventSourceType` enum variants `ParticipationCron`, `DormancyCron`, `VoteOutcome`, `EvidenceQuality`).
- `crates/db_schema/src/source/governance/case_evidence.rs` (full file — Source 4a heuristic queries this; planner must identify the submitter_id/person_id FK column tying evidence to its reporter; added per clarify DQ a3d0e9941441-021).
- `crates/db_schema/src/source/governance/actor_pseudonym.rs` + `migrations/2026-04-15-100400-0000_add_actor_pseudonym/up.sql` (schema reference for the `actor_pseudonym_id=None` cron-batch attribution decision per clarify DQ a3d0e9941441-018).
- `crates/api/api/src/governance/admin_assign_jury.rs:932` (v0 precedent — `actor_pseudonym = None` for system-attributed governance_log entries; MIRROR for r3 cron-batch entries).
- `crates/db_schema/src/source/governance/governance_log.rs` lines 200-220 (the r1-shipped ENTRY_KIND constants for r3 use; do NOT add new ones).
- `crates/api/api/src/governance/governance_log.rs` (the shim — verify `pub use` re-exports for the five r3-relevant kinds).
- `crates/api/api/src/governance/admin_emergency_remove.rs` (full file — understand the existing emergency-remove handler shape; the `flag-bad-faith` endpoint mirrors its capability-check pattern).
- `.claude/PRPs/plans/v1-RT-r2.plan.md` — prior phase plan for context (calculator + bounds clamp shape, MIRROR refs, e2e test patterns, §16a story discipline).
- `.claude/PRPs/plans/v1-RT-r1.plan.md` — r1 plan for schema/config/entry-kind ground truth (the schema r3 depends on).
- `.claude/rules/governance-log-entry-kind-registry.md` — read the "v1-RT-r1 entry kinds" section: it names r3 as the call-site landing phase for `_PARTICIPATION_CRON_TICK`, `_VOTE_OUTCOME_RECORDED`, `_EVIDENCE_QUALITY_RECORDED`. Plan must wire all three call sites.
- `.claude/rules/advisor-orchestrator.md` §2.4 — mandatory file-class lesson injection table.
- `.claude/lessons/feedback_clippy_test_style.md` — Lemmy test style (no `unwrap`/`expect`; `LemmyResult<()>` with `?`).
- `.claude/lessons/feedback_async_pool_test_pattern.md` + `.claude/lessons/feedback_lemmy_error_no_std_error.md` — mandatory for any e2e.rs edit.
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — read if the plan totals ≥2 e2e.rs edits.
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — confirm the cron handlers use `run_transaction` for per-community-tx atomicity.
- `.claude/lessons/feedback_advisor_watchpoint_specificity.md` — every plan §4 watchpoint must cite specific file:line.
- `.claude/lessons/feedback_plan_dod_dry_run_at_write.md` — DoD commands must be executable as written.
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — FILES YAML (`creates:` + `modifies:` + `requires:`) required on every §13 task for cohort-dispatch + brehon-verify.
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md` — Shape G suspended; cargo runs on laptop via `kind: "validate-pending-laptop"` + `validate-pending-laptop-e2e` DQs.
- `.claude/lessons/feedback_advisor_authoring_under_daemon_stress.md` — if Junior dispatch stress accumulates, advisor-authoring fallback is authorised after the criterion fires.

## 4. Constraints

- **No new migrations** — r3 is emitter wiring only; any proposed migration is a scope violation. Catch-fire if a §13 task proposes one.
- **No new `ENTRY_KIND_*` consts** — r1 pre-landed all five r3-relevant kinds. Catch-fire if a §13 task proposes adding to `governance_log.rs`.
- **No new `governance_config` keys** — r1 + v1-AD-* seeded all r3-relevant keys. Catch-fire if a §13 task proposes seeding.
- **`feature.reputation_v1_decay_enabled` does NOT gate r3 emitters.** r3 writes `reputation_event` rows unconditionally; r2's calculator + bounds clamp absorb them on the next 15-min snapshot tick. The flag gates the calculator, not the event log. Per PRD §11 + ADR-008 (append-only event log).
- **Per-community-tx atomicity for crons.** Both crons MUST wrap each community's batch in `conn.run_transaction(...)` — a mid-community crash rolls back THAT community only; earlier committed communities stay. Mirror the pattern at `submit_jury_vote.rs:139`.
- **Dedupe-key idempotency MUST be tested.** Plan §16a MUST include a story: "re-running the activity cron in the same ISO week produces zero new rows (constraint violation = idempotent success)". Same for dormancy.
- **No-penalty-for-dissent in vote-outcome emitter.** Jurors who vote in the minority emit nothing — NOT a `0` event, NOT a `−1` event. Test must assert minority jurors have no `VoteOutcome` event after case decision.
- **`flag-bad-faith` endpoint is instance-admin-only.** Capability check is `is_admin(person)` AND the case status must be `EmergencyRemove`. Per ADR-013 admin-driven posture: no auto-detection. Returns 403 for non-admins, 400 if case is not in EmergencyRemove status.
- **`actor_pseudonym_id = None` for cron-batch entries (resolved per clarify DQ a3d0e9941441-018).** Cron-emitted governance_log entries write `actor_pseudonym_id = None` (the column is `Option<...>` on `governance_log`). Honors PRD §5.3 "synthetic system pseudonym" framing semantically (no real-person attribution) but realises it via `Option::None` rather than seeding a sentinel row. MIRROR the v0 precedent at `admin_assign_jury.rs:932` doc-comment. **NOTE:** the `flag-bad-faith` admin endpoint (Source 4b) is NOT a cron-batch entry — it uses real admin attribution (`actor_pseudonym_id = <admin's pseudonym row id>`).
- **DQ discipline:** write `kind: "validate-pending-laptop"` DQ entry after impl-task pushes its worker branch (Shape G suspended per DQ #229; cargo runs on laptop). E2e is `kind: "validate-pending-laptop-e2e"` separately.
- **e2e-on-laptop-only guard (MANDATORY in every impl-task brief whose DoD includes e2e)** — verbatim text per `.claude/PRPs/handovers/v1-RT-r3-bootstrap.md` §3 carry-forward Action 1:

  > e2e runs on **laptop only** — after cargo-check/clippy/unit-tests pass, write `kind: "validate-pending-laptop-e2e"` DQ entry (commands array = all 4 validate commands, branch, phase_task) and **stop**. Do NOT run e2e on the EliteDesk worker; the laptop advisor session runs it and mutates the DQ entry.

- **Mandatory lesson injection per `.claude/rules/advisor-orchestrator.md` §2.4:**
  - Any edit to `crates/server/tests/e2e.rs` → inject `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md`. If ≥2 e2e edits across the plan, also `feedback_junior_worker_e2e_edit_hang.md`.
  - Cron handlers do 2+ DB writes inside their per-community transaction → inject `feedback_multi_write_handlers_need_transactions.md`.
  - `#[cfg(feature = "full")]` gates → `feedback_features_full_workspace_only.md` + `feedback_features_full_p_crate_incompatible.md`.
- **MIRROR ref discipline:** cite specific existing sibling code/test for every new pattern. For the crons: MIRROR the existing reputation-snapshot tick at `scheduled_tasks.rs:201` for `RunningGuard` + concurrency pattern. For the vote-outcome emitter: MIRROR the existing `emit_reputation_event` call at `submit_jury_vote.rs:583`. For the e2e tests: MIRROR `v1_*_fixtures` modules already present in the file.
- **FILES YAML on every §13 task** — `creates:` + `modifies:` + `requires:` arrays per `feedback_explicit_file_arrays_on_tasks.md`. The `requires:` array is load-bearing for cohort-dispatch dependency check (per `.claude/rules/advisor-orchestrator.md` §4.1 step 4a). Crons in `scheduled_tasks.rs` are likely a single task (cohesive file); vote-outcome + evidence-cited emitters are in the same `submit_jury_vote.rs` `case_decided` branch (likely a single task too); the `flag-bad-faith` endpoint is a distinct file (separate task, may be `[P]` parallel with the crons but NOT with the submit_jury_vote work since both crons and submit_jury_vote share zero files).
- **Commit attribution:** impl-task commits use `feat(rep-tuning): <description> (task N)` subject (matching r2 convention).
- **Watchpoints in plan §4** cite specific file:line:
  - `scheduled_tasks.rs:201` (existing reputation-snapshot tick — shape to copy for `RunningGuard`).
  - `scheduled_tasks.rs:163` (existing `all_active_counts` call — MIRROR for the activity-cron comment-counting query shape, per clarify DQ a3d0e9941441-023).
  - `submit_jury_vote.rs:282` (the `case_decided` branch where vote-outcome + evidence-cited emitters land — emit INSIDE the existing `run_transaction` at line 139, per clarify DQ a3d0e9941441-020).
  - `submit_jury_vote.rs:583` (existing `emit_reputation_event` call — MIRROR for new calls; co-locate vote-outcome emit immediately after the existing jury_reliability emit in the same iteration).
  - `submit_jury_vote.rs:968` (existing `emit_reputation_event` helper — confirm signature accepts `source_event_type` + `dedupe_key`; already verified at reputation_event.rs:34+51).
  - `reputation_event.rs:32-51` (dedupe_key + source_event_type schema columns).
  - `governance_log.rs:212-216` (r1-pre-landed ENTRY_KIND consts — do NOT add to this file).
  - `admin_emergency_remove.rs` (330-line file — second handler `flag_bad_faith_emergency_report` lands here, MIRROR `emergency_remove_open_case` shape, per clarify DQ a3d0e9941441-019).
  - `jury_vote.rs:22+33` (`rationale: Option<String>` column — Source 4a heuristic precondition, verified populated per clarify DQ a3d0e9941441-022).

- **Plan §16a stories** — minimum 4 stories:
  1. Activity cron emits `+1` per active user per community; idempotent across same-ISO-week re-runs (`dedupe_key` ON CONFLICT DO NOTHING).
  2. Dormancy cron emits `−2` per dormant user per community; idempotent.
  3. Vote-outcome emitter: majority-aligned jurors get `+1 participation_consistency`; minority jurors get nothing (asserted).
  4. `flag-bad-faith` admin endpoint emits `−1 reporting_accuracy` only when called by instance-admin AND case is in `EmergencyRemove` status; 403 for non-admin, 400 for non-EmergencyRemove case.

  Optional 5th: evidence-cited heuristic emits `+1 reporting_accuracy` when rationale length ≥ threshold AND reporter has ≥1 case_evidence row. (Defer to v2 if `jury_vote.rationale` is empty-by-default in v0 — file DQ.)

- **Forbidden-window awareness:** Shape G suspended; cargo runs on laptop. The forbidden windows in `.claude/rules/advisor-orchestrator.md` §5.1 apply to local cargo dispatch. Advisor enforces.
- **Stop-and-ask tripwires for the planner** (carried from bootstrap §"Stop-and-ask tripwires"):
  - Stop if a §13 task proposes a new migration → catch-fire (scope violation; r1 owns the schema).
  - Stop if a §13 task proposes adding to `governance_log.rs` ENTRY_KIND list → catch-fire (r1 pre-landed all five).
  - Stop if a §13 task proposes seeding a `system` person + actor_pseudonym row → catch-fire (resolved per clarify DQ a3d0e9941441-018: use `actor_pseudonym_id=None` for cron entries; no sentinel row).
  - Stop if the planner proposes more than 4 §13 tasks — r3's file footprint is wider than r2's but 5+ tasks signals over-splitting; check if tasks can be batched by emitter location (one task per file makes sense: scheduled_tasks.rs, submit_jury_vote.rs, admin_emergency_remove.rs, e2e.rs).
  - Stop if `jury_vote.rationale` is `None`-by-default in v0 e2e fixtures (verified at schema level per clarify DQ a3d0e9941441-022; population is fixture-dependent) — Source 4a heuristic has no signal; file a DQ and defer Source 4a to v2 (keep Source 4b `flag-bad-faith` in r3 since it doesn't depend on rationale).
