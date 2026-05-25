# Plan: v1-RT-r3 — multi-source participation_consistency emitters + flag-bad-faith admin endpoint

## 1. Summary

v1-RT-r3 wires the **four PRD §5.3 reputation event sources** that feed
the v1 calculator (r2-shipped) and adds the **`flag-bad-faith` admin
endpoint** that produces the negative arm of the evidence-quality source.
Specifically:

- **Source 1** (weekly activity cron) emits `+1 participation_consistency`
  per (active user, community) per ISO week, idempotent via
  `dedupe_key = activity_cron:<community_id>:<person_id>:<iso_week>`.
- **Source 2** (weekly dormancy cron) emits `-2 participation_consistency`
  per (dormant user, community) per ISO week, idempotent via
  `dedupe_key = dormancy_cron:<community_id>:<person_id>:<iso_week>`.
- **Source 3** (post-decision vote-outcome) emits `+1
  participation_consistency` per majority-aligned juror, idempotent via
  `dedupe_key = vote_outcome:<case_id>:<juror_pseudonym>`. Minority
  jurors emit nothing (Brehon no-penalty-for-dissent per PRD §5.3).
- **Source 4a** (post-decision evidence-cited heuristic) emits `+1
  reporting_accuracy` for the reporter when `jury_vote.rationale.len() ≥
  participation.evidence_cited_rationale_threshold_chars` AND the
  reporter has ≥1 `case_evidence` row, idempotent via
  `dedupe_key = evidence_cited:<case_id>:<reporter_pseudonym>`.
- **Source 4b** (`POST /api/v4/governance/admin/emergency-remove/flag-bad-faith`)
  emits `-1 reporting_accuracy` for the reporter when an instance admin
  flags a case in `EmergencyRemove` status as bad-faith, idempotent via
  `dedupe_key = evidence_bad_faith:<case_id>`.

All five emitters write `reputation_event` rows unconditionally (the
`feature.reputation_v1_decay_enabled` flag gates the r2 calculator that
*consumes* the events, not the events themselves — per PRD §11 + ADR-008
append-only event log). r3 ships zero new migrations, zero new
`ENTRY_KIND_*` consts (r1 pre-landed all five), zero new
`governance_config` keys (r1 + v1-AD-a seeded everything r3 reads); it
is pure emitter wiring + one new admin endpoint.

Headline acceptance: the four §16a stories (one per source family) all
report `[done]` against `/brehon-verify`; the dedupe-key idempotency
guard is asserted under repeat-tick conditions for the two crons; the
no-penalty-for-dissent invariant is asserted for the vote-outcome emit;
`flag-bad-faith` returns 403 / 400 / 200 in the matching capability +
status pre-condition arms.

## 2. Source

- `.claude/PRPs/prds/v1-reputation-tuning.prd.md` §5.3 (multi-source
  events — Sources 1/2/3/4 enumerated) + §5.4 (gate strategies — NOT
  r3; read so the planner does not accidentally pull them in) + §6
  (acceptance criteria — only the participation/vote-outcome/evidence
  rows are r3) + §10 (security — admin auth + governance_log
  requirements) + §11 row 3 (this sub-phase scope) @ `e3f919d70`
- `.claude/PRPs/briefs/v1-RT-r3-planning-1.md` @ `e3f919d70`
- DQ `a3d0e9941441-018` (cron-batch governance_log uses
  `actor_pseudonym_id=None`, NOT a sentinel `system` row; resolved per
  user 2026-05-25)
- DQ `a3d0e9941441-019` (extend `admin_emergency_remove.rs` with second
  handler `flag_bad_faith_emergency_report`; do NOT create new file;
  resolved)
- DQ `a3d0e9941441-020` (vote-outcome + evidence-cited emit lives
  INSIDE the existing `case_decided` branch INSIDE the existing
  `run_transaction` at `submit_jury_vote.rs:139`, co-located with the
  existing `jury_reliability` emit at line 583; resolved)
- DQ `a3d0e9941441-021` (Source 4a heuristic queries
  `case_evidence.uploader_id`; canonical schema reference for §3
  Required reading; resolved)
- DQ `a3d0e9941441-022` (`jury_vote.rationale` is
  `Option<String>` per `jury_vote.rs:22+33`, populated by callers; ship
  Source 4a in r3; resolved)
- DQ `a3d0e9941441-023` (MIRROR `all_active_counts` at
  `scheduled_tasks.rs:163` as design reference for the comment-counting
  query shape; write a NEW dedicated helper for r3's per-(community,
  person) cardinality; resolved)
- DQ `a3d0e9941441-024` (enforce PRD literal URL `POST
  /api/v4/governance/admin/emergency-remove/flag-bad-faith`; resolved)
- `.claude/lessons/feedback_clippy_test_style.md` — Lemmy test style
  (no `unwrap`/`expect`; e2e tests return `LemmyResult<()>` with `?`)
- `.claude/lessons/feedback_async_pool_test_pattern.md` — canonical
  pool/conn/LemmyResult fixture for e2e (Task 4)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — error-shape
  uniformity (Case A — mirror the existing v1-SL-* / v1-JM-* fixtures
  module shape verbatim; Task 4 e2e edits)
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — read at
  Task 4 (this plan has 1 e2e edit; below the ≥2 edit threshold but
  the lesson still binds via the fixture-add discipline)
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md`
  — both crons wrap each per-community batch in `run_transaction`; the
  flag-bad-faith handler reuses the existing `run_transaction` shape
  from `admin_emergency_remove.rs:88-100`
- `.claude/lessons/feedback_features_full_workspace_only.md` +
  `.claude/lessons/feedback_features_full_p_crate_incompatible.md` —
  every cargo command uses `--workspace --features full`, NEVER `-p
  <crate> --features full`
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — FILES
  YAML (`creates:` + `modifies:` + `requires:`) on every §13 task
- `.claude/lessons/feedback_advisor_watchpoint_specificity.md` — every
  §4 / §18 watchpoint cites a specific file:line or table
- `.claude/lessons/feedback_plan_dod_dry_run_at_write.md` — every §15
  command dry-runnable by the advisor against current HEAD
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md` —
  Shape G suspended; cargo runs on laptop via
  `kind: "validate-pending-laptop"` + `validate-pending-laptop-e2e`
- `.claude/lessons/feedback_advisor_authoring_under_daemon_stress.md`
  — fallback authorisation if Junior daemon stress accumulates mid-phase
- `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` —
  pre-enumerate callsites for any struct-shape change (none in r3; no
  struct fields added or signatures changed on public API)
- `.claude/rules/governance-log-entry-kind-registry.md` "v1-RT-r1 entry
  kinds" — r1 pre-landed `_PARTICIPATION_CRON_TICK`,
  `_VOTE_OUTCOME_RECORDED`, `_EVIDENCE_QUALITY_RECORDED` consts; r3 is
  named as the call-site landing phase
- ADR-008 (append-only event log — r3 writes new `reputation_event`
  rows; never updates / deletes) + ADR-013 (admin-driven posture —
  `flag-bad-faith` is human-in-the-loop, no auto-detection) + ADR-015
  (pseudonymisation — every governance_log payload field naming a
  person uses `*_pseudonym`, never raw `PersonId`)
- `.claude/PRPs/plans/v1-RT-r2.plan.md` — prior phase plan for context
  (per-dimension calculator + bounds clamp ship-shape; MIRROR for §16a
  story discipline + per-task complexity ceiling)
- `.claude/PRPs/plans/v1-RT-r1.plan.md` — r1 plan for schema/config/
  entry-kind ground truth (the schema r3 depends on)
- `migrations/2026-05-10-000000-0000_add_reputation_event_v1_columns/up.sql`
  — confirms the partial unique index
  `reputation_event_dedupe_key_partial_idx ON reputation_event
  (dedupe_key) WHERE dedupe_key IS NOT NULL` is live on
  `governance-v0` HEAD; Diesel's `.on_conflict_do_nothing()` consumes it

## 3. Problem statement

The v1 reputation calculator (r2) ships a per-(dimension, direction)
chained-halving decay + bounds clamp behind
`feature.reputation_v1_decay_enabled`. Without r3's emitters, the
calculator has nothing new to consume — the v0 event sources
(endorsement / jury_vote / sponsor_liability / founder_seed) still feed
`reputation_event` rows, but the four v1 sources defined in PRD §5.3 are
absent:

- **No participation signal from comment activity** — operators have no
  mechanical way to reward sustained community participation; reputation
  drift is driven purely by jury work and endorsements. Per PRD §5.3
  source 1 + §6 acceptance row "weekly active cron emits +1 per
  community per active user".
- **No dormancy signal** — long-inactive users keep their accumulated
  participation_consistency forever; the PRD §5.3 source 2 negative
  decay arm is unwired. The r2 calculator's chained-halving handles
  *time-decay* of existing positive events but not the *active negative*
  signal a dormancy cron emits.
- **No vote-outcome signal** — jurors who align with the majority get
  `+1 jury_reliability` already (v0; `submit_jury_vote.rs:583`) but no
  `+1 participation_consistency` for *participating* in the case
  decision. PRD §5.3 source 3 fills the gap; Brehon's
  no-penalty-for-dissent invariant means minority jurors emit *nothing*
  (NOT a `0` event, NOT a `-1` event).
- **No evidence-quality signal** — case reporters who attach evidence
  AND whose argument lands in the jury's rationale get no
  reputational credit; case reporters whose reports an admin retro-flags
  as bad-faith get no reputational debit. PRD §5.3 source 4 wires both
  arms.
- **No admin tool for the bad-faith path** — ADR-013 mandates
  admin-driven posture (no auto-detection); v0 has no `flag-bad-faith`
  endpoint; the human-in-the-loop signal is mechanically impossible to
  produce.

Per PRD §11 row 3, all five emitters land in this sub-phase. The
schema, ENTRY_KIND consts, config keys, and `ReputationEventSourceType`
enum variants are pre-landed by r1 / v1-AD-a — r3 is pure call-site
wiring + the one new admin endpoint.

## 4. Solution statement

Three implementation surfaces, queued as three `[P]`-parallel cohort
tasks, followed by a barrier e2e task.

### Surface 1 — `participation_cron` module + scheduler block

A new helper module at
`crates/api/api/src/governance/participation_cron.rs` exposes two
`pub async fn run_*_batch(context: &LemmyContext) -> LemmyResult<...>`
entry points modelled on
`appeal_window_expiry.rs::run_appeal_window_expiry_batch` and
`sponsor_liability_grace.rs::run_grace_check_batch`. Each batch:

1. Reads its config knobs through a per-batch `ConfigCache`.
2. Issues a single discovery query against the `comment` table grouped
   by `(community_id, creator_id)` over the lookback window
   (`participation.lookback_days` for activity;
   `participation.dormancy_window_days` for dormancy). The activity
   path retains rows with `count >= participation.activity_threshold_comments`;
   the dormancy path retains rows for `(community_id, person_id)` pairs
   with a prior `participation_consistency` event in the community AND
   zero non-deleted/non-removed comments in the window.
3. Groups results by community.
4. Per community: runs a `conn.run_transaction(async |conn| { ... })`
   block that (a) for each qualifying person, INSERTs one
   `reputation_event` row with the matching `dedupe_key` +
   `source_event_type` + `ReputationDimension::ParticipationConsistency`
   + `.on_conflict_do_nothing()` (idempotency under repeat-tick), and
   (b) appends one `governance_log` row with
   `ENTRY_KIND_PARTICIPATION_CRON_TICK` + payload `{community_id,
   iso_week, qualifying_user_count, dedupe_key_prefix, delta}` +
   `actor_pseudonym = None` (per DQ a3d0e9941441-018; mirror
   `admin_assign_jury.rs:932` precedent — system-attributed cron-batch
   entry).
5. Per-community-tx isolation: a mid-community DB error rolls back THAT
   community only; earlier-committed communities stay. Mirror
   `sponsor_liability_grace::run_grace_check_batch` lines 156-178 —
   per-row tx; outer batch returns `Ok` regardless; per-row errors land
   in `tracing::warn`.

`crates/api/api/src/governance/mod.rs` adds `pub mod participation_cron;`.

`crates/routes/src/utils/scheduled_tasks.rs` adds a new
`scheduler.every(CTimeUnits::days(participation_interval_days)).run(...)`
block right after the federation-replay-cleanup block (after line 449,
before the `loop { scheduler.run_pending() ... }`), gated by a new
`BREHON_DISABLE_PARTICIPATION_JOB=1` env-var check for tests + a new
`PARTICIPATION_CRON_RUNNING` atomic + `ParticipationCronRunningGuard`
RAII struct (mirror `REPUTATION_SNAPSHOT_RUNNING` + `RunningGuard` at
lines 56-64). The closure body calls **both**
`lemmy_api::governance::participation_cron::run_activity_batch(&context)`
**then** `lemmy_api::governance::participation_cron::run_dormancy_batch(&context)`
sequentially under the single guard (per brief — "Weekly dormancy cron
in same `scheduled_tasks.rs` tick, registered *after* the activity cron
block"; the two batches share one tick + one guard, distinct
`dedupe_key` namespaces).

### Surface 2 — `submit_jury_vote.rs` Sources 3 + 4a emit

Inside the existing `process_vote` `case_decided` block (the
`if path_kind == SLDPathKind::Decided { ... }` branch starting at line
517), **co-locate two new emit sites**:

- **Source 3 (vote-outcome)**: extend the existing juror-loop at lines
  577-597. For each juror whose `juror_decision == winning_decision`,
  emit (inside the same loop iteration, immediately after the existing
  `emit_reputation_event(... JuryReliability ...)` call) a SECOND
  `emit_reputation_event` call with
  `ReputationDimension::ParticipationConsistency`, delta from
  `deltas.participation_juror_aligned` config (default `+1`),
  `source_event_type = VoteOutcome`,
  `dedupe_key = "vote_outcome:<case_id>:<juror_pseudonym>"`. The
  juror's `actor_pseudonym` is resolved via
  `actor_pseudonym_helper::get_or_create`. Minority jurors
  (`juror_decision != winning_decision`) emit **nothing** for
  `ParticipationConsistency` — the existing `juror_outlier_delta`
  jury_reliability emit at line 583 still fires for them (v0 behaviour
  preserved); the new `ParticipationConsistency` emit is gated on
  alignment. Append ONE `governance_log` row per juror with
  `ENTRY_KIND_VOTE_OUTCOME_RECORDED`, payload `{case_id,
  juror_pseudonym, dimension: "participation_consistency", delta,
  source_event_type: "vote_outcome"}`,
  `actor_pseudonym = Some(juror_pseudonym)` (the juror is the subject;
  per ADR-015 pseudonymisation, the governance_log entry attributes to
  them).

- **Source 4a (evidence-cited heuristic)**: append a NEW conditional
  block AFTER the existing reporter emit at lines 600-625 (still inside
  the `case_decided` branch). The heuristic:
  1. If `case_row.creator_id.is_none()` (no reporter), skip (mirror v0
     behaviour at line 600).
  2. Else, query `case_evidence` for an existence check:
     `SELECT EXISTS(SELECT 1 FROM case_evidence WHERE case_id = $1
     AND uploader_id = $reporter_id)`. If false, skip.
  3. Else, examine `winning_rationales` (the `Vec<String>` already
     assembled at line 411). If
     `winning_rationales.iter().any(|r| r.chars().count() >= rationale_threshold)`
     where `rationale_threshold = config::get_int("participation.evidence_cited_rationale_threshold_chars")`,
     emit `+1 reporting_accuracy` for the reporter via
     `emit_reputation_event` with `source_event_type =
     EvidenceQuality`, `dedupe_key =
     "evidence_cited:<case_id>:<reporter_pseudonym>"`, delta from
     `deltas.evidence_cited` config (default `+1`).
  4. Append ONE `governance_log` row with
     `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED`, payload `{case_id,
     reporter_pseudonym, dimension: "reporting_accuracy", delta,
     source_event_type: "evidence_quality", trigger:
     "rationale_cited"}`, `actor_pseudonym = Some(reporter_pseudonym)`.

Both Source 3 and Source 4a writes live INSIDE the existing
`run_transaction` at line 139 (per DQ a3d0e9941441-020). The
transaction's existing rollback semantics cover any mid-case failure
atomically.

The `emit_reputation_event` helper at line 968 already accepts every
field r3 needs via its `ReputationEventInsertForm` body — except that
it **hard-codes** `source_event_type: Some(JuryVote)` and `dedupe_key:
None`. Task 2 must extend the helper signature to accept both as
parameters (with the existing JuryVote callers continuing to pass
`Some(JuryVote)` + `None` to preserve byte-identical v0 behaviour at
the existing call sites at lines 583-595 + 615-624). This is the
**only public-symbol shape change in r3**; it is `async fn`
(non-public, file-private), so the change is purely intra-file.

### Surface 3 — `flag-bad-faith` admin endpoint

`crates/api/api_common/src/governance.rs` adds two new public types
(mirror `AdminCloseCase` + `AdminCloseCaseResponse` at lines 191-203
verbatim):

```rust
pub struct FlagBadFaithEmergencyReport {
  pub case_id: ModerationCaseId,
}

pub struct FlagBadFaithEmergencyReportResponse {
  pub case_id: ModerationCaseId,
  pub flagged: bool,
}
```

`crates/api/api/src/governance/admin_emergency_remove.rs` extends the
file with a second public handler `flag_bad_faith_emergency_report`,
mirroring `emergency_remove_open_case`'s outer
`is_admin(&local_user_view)?` capability-check + `run_transaction`
shape. Steps:

1. `is_admin(&local_user_view)?` — 403 on non-admin.
2. Load `ModerationCase` by `case_id`; 404 on missing.
3. Assert `case_row.status == CaseStatus::EmergencyRemove`; 400 on
   any other status (per ADR-013: the bad-faith signal only applies
   to admin-removed cases — non-EmergencyRemove cases have their own
   jury-driven accountability arc).
4. `case_row.creator_id.ok_or(LemmyErrorType::NotFound)?` — the
   reporter PersonId; cases without a reporter (defense-in-depth)
   cannot be flagged.
5. Resolve `reporter_pseudonym` via
   `actor_pseudonym_helper::get_or_create`.
6. Resolve `admin_pseudonym` via the same helper (for the
   governance_log attribution).
7. Emit `-1 reporting_accuracy` for the reporter via the local
   `emit_reputation_event_local` private helper (inlined in this file
   following the same shape as `submit_jury_vote.rs:968` — file-private
   helper, NOT a module-level cross-file refactor) with `delta =
   deltas.evidence_bad_faith` (default `-1`), `source_event_type =
   EvidenceQuality`, `dedupe_key =
   "evidence_bad_faith:<case_id>"`, `.on_conflict_do_nothing()`. The
   dedupe key contains no `reporter_pseudonym` segment (one bad-faith
   flag per case per ADR-013).
8. Append ONE `governance_log` row with
   `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED`, payload `{case_id,
   reporter_pseudonym, dimension: "reporting_accuracy", delta,
   source_event_type: "evidence_quality", trigger:
   "admin_flagged_bad_faith"}`, `actor_pseudonym =
   Some(admin_pseudonym)` (the ADMIN is the actor; per ADR-015 +
   brief §4 "real admin attribution" — NOT a cron-batch entry).
9. Return `Json(FlagBadFaithEmergencyReportResponse { case_id,
   flagged: true })`.

All nine steps execute inside a single `run_transaction` mirroring
`emergency_remove_open_case`'s outer wrap (lines 88-100). The handler
is `pub async fn`, registered in routes.

`crates/api/routes/src/lib.rs` adds:

- Import: `governance::admin_emergency_remove::flag_bad_faith_emergency_report`
  (alongside the existing governance imports at lines 31-49).
- Route registration: under the existing `/admin` scope (line 493),
  add a new `scope("/emergency-remove")` sub-scope containing
  `.route("/flag-bad-faith", post().to(flag_bad_faith_emergency_report))`.
  The PRD literal URL resolves to
  `POST /api/v4/governance/admin/emergency-remove/flag-bad-faith` (per
  DQ a3d0e9941441-024).

### Surface 4 — e2e integration tests

`crates/server/tests/e2e.rs` adds ONE new fixtures module
`v1_rt_r3_fixtures` (mirror the existing `v1_*_fixtures` modules
already present in the file) containing 5 stories' worth of tests +
shared seeding helpers. Per `feedback_lemmy_error_no_std_error.md`
Case A — outer test fns return `LemmyResult<()>`, helper fns return
`LemmyResult<T>` consistently; no `.map_err` bridges; mirror the
canonical sibling module's error-shape verbatim.

Stories ↔ tests:
- **Story 1 — activity cron emit + idempotency**: 2 tests
  (`participation_activity_cron_emits_plus_one_per_active_user`,
  `participation_activity_cron_idempotent_across_same_iso_week`).
- **Story 2 — dormancy cron emit + idempotency**: 2 tests
  (`participation_dormancy_cron_emits_minus_two_per_dormant_user`,
  `participation_dormancy_cron_idempotent_across_same_iso_week`).
- **Story 3 — vote-outcome emit + no-penalty-for-dissent**: 2 tests
  (`vote_outcome_emits_plus_one_for_majority_aligned_jurors`,
  `vote_outcome_emits_nothing_for_minority_jurors`).
- **Story 4 — `flag-bad-faith` 403/400/200 arms**: 3 tests
  (`flag_bad_faith_returns_403_for_non_admin`,
  `flag_bad_faith_returns_400_for_non_emergency_remove_status`,
  `flag_bad_faith_admin_on_emergency_remove_case_emits_minus_one`).
- **Story 5 — evidence-cited heuristic emit**: 1 test
  (`evidence_cited_heuristic_emits_plus_one_when_rationale_above_threshold`).

Total: 10 new e2e tests in 1 fixtures module.

The crons run BREHON_DISABLE_PARTICIPATION_JOB-gated; tests call
`participation_cron::run_activity_batch(&context).await?` and
`participation_cron::run_dormancy_batch(&context).await?` directly
(per the same pattern as `BREHON_DISABLE_SNAPSHOT_JOB` + direct
`run_snapshot_batch` invocation in existing e2e tests).

## 5. Metadata

- **Phase:** `v1-RT-r3`
- **Branch:** `phase-v1-RT-r3` (cut by `bm-cut` before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 6 (Task 0 pre-flight + Tasks 1-4 impl + Task 5 retro)
- **Estimated cargo budget:** `~6 GB peak` per task (single `cargo
  check --workspace --features full`; no migrations; one e2e edit)
- **Forbidden-window applicability:** standard (per
  `.claude/rules/advisor-orchestrator.md` §5.1). Shape G **SUSPENDED**
  per DQ #229 → `validate-pending-laptop` + `validate-pending-laptop-e2e`
  pathways active; cargo runs on laptop. Forbidden-window check applies.
- **Complexity score:** `7/10`

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Sonnet target ⇒ split-DQ
fires at `score > 8`; r3 scores `7/10` — no split required.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 4 impl tasks (Tasks 1-4); Task 0 + retro excluded |
| Migrations touched | +2 each | 0 | r3 is emitter-wiring + 1 admin handler only; brief §4 explicit no-migration constraint |
| Crates touched | +1 each | 4 | `crates/api/api`, `crates/api/api_common`, `crates/api/routes`, `crates/routes` (Task 1's `scheduled_tasks.rs` lives in `lemmy_routes` per `crates/routes/`); `crates/server` for e2e doesn't add to the count because the test target only |
| `crates/server/tests/e2e.rs` edits | +3 each | 1 | Task 4 only — one e2e edit, fixtures module add |
| New ADR-affecting decisions | +2 each | 0 | r3 inherits PRD's ADR posture; no new OQ deltas |
| Cargo budget peak above 6 GB | +1 per GB | 0 | ~6 GB peak; Shape-G suspended (laptop path) |
| **Total** | — | **7** | Threshold for split-DQ: `>8` (Sonnet target); no split |

### 5.2 Per-task complexity ceiling (Sonnet target)

Sonnet ceiling: `≤ 4` files per task / `≤ 2` distinct crates per task;
`crates/server/tests/e2e.rs` does NOT appear in `modifies:` alongside
non-test logic.

- **Task 1** (crons): 3 files (`scheduled_tasks.rs` +
  `participation_cron.rs` + `governance/mod.rs`) in 2 crates
  (`crates/routes`, `crates/api/api`). ✓ Under ceiling.
- **Task 2** (vote-outcome + evidence-cited emit): 1 file
  (`submit_jury_vote.rs`) in 1 crate. ✓ Under ceiling.
- **Task 3** (`flag-bad-faith` endpoint): 3 files
  (`api_common/src/governance.rs` + `admin_emergency_remove.rs` +
  `api/routes/src/lib.rs`) in 3 crates (`crates/api/api_common`,
  `crates/api/api`, `crates/api/routes`). **Soft ceiling violation on
  crates: 3 > 2.** Rationale for accepting + not splitting: the
  endpoint is mechanically one feature (one request type + one handler
  + one route line); each per-crate edit is small (~30 lines of
  request type, ~80 lines of handler body, 4 lines of route + 1
  import); splitting into 3 sub-tasks (à la v1-AD-c Tasks 2/3/7) adds
  3 commits + 3 ci-watcher cycles + 2 cohort barriers for a unit of
  work that is one shape. The brief's stop-and-ask tripwire #4
  (>4 §13 tasks signals over-splitting) takes precedence; bundling
  the three small edits keeps the impl-task within the canonical
  "≤4 files" file-count ceiling and within the spirit of the per-task
  budget. Documented as a deliberate acceptance, not a planner miss.
- **Task 4** (e2e): 1 file (`crates/server/tests/e2e.rs`) in 1 crate.
  ✓ Under ceiling. e2e is the SOLE `modifies:` entry — not bundled
  with non-test logic per the lesson.

## 6. Relationship to other v1-RT-* sub-phases

- **Depends on:** v1-RT-r1 (schema foundation: `dedupe_key` +
  `source_event_type` columns + partial unique index; r3-relevant
  ENTRY_KIND consts; `ReputationEventSourceType` variants
  `ParticipationCron` / `DormancyCron` / `VoteOutcome` /
  `EvidenceQuality`) AND v1-RT-r2 (per-dimension calculator + bounds
  clamp — the consumer of r3's new events). Both shipped to
  `governance-v0`; r3 builds on the merged base.
- **Coexists with:** v1-AD-* (admin config seeded all 10 r3-relevant
  config keys: `deltas.participation_weekly_active`,
  `participation.dormancy_window_days`, `deltas.participation_dormant`,
  `deltas.participation_juror_aligned`,
  `participation.activity_threshold_comments`,
  `participation.lookback_days`, `deltas.evidence_cited`,
  `deltas.evidence_bad_faith`,
  `participation.evidence_cited_rationale_threshold_chars`,
  `job.participation_interval_days`).
- **Followed by:** v1-RT-r4 (sponsor-gate strategies — independent of
  r3), v1-RT-r5 (rollup cron — `ENTRY_KIND_ROLLUP_RECOMPUTED` ships
  there), v1-RT-r6 (CR carry-forward bundle — independent).
- v1 planning-queue id: `RT-r3` (per PRD §11).

## 7. Preflight guardrails inherited from prior phases

- **R1 (clippy — i32↔i64):** every `i32 ↔ i64` comparison uses
  `i64::from(...)`, never `as` cast (per `feedback_clippy_test_style.md`).
  Threshold reads from `config::get_int` return `i64`; cast back to
  `i32` only via `i32::try_from(...).map_err(...)?` with a
  `LemmyErrorType::Unknown` wrapper (mirror
  `submit_jury_vote.rs:561-570` shape).
- **R2 (config read):** every governance_config read flows through
  `config::get_int` / `config::get_bool` with a per-batch
  `ConfigCache` already in scope. Never call `fetch_value` directly.
- **R3 (Watch 8 — penalty + cliff guards):** the `expires_at.is_some()`
  founder cliff guard and the existing penalty handling in
  `submit_jury_vote.rs` stay invariant; r3's new emits do NOT bypass
  them. r3's negative deltas (`-2` dormancy, `-1` evidence_bad_faith)
  flow through the SAME `compute_applied_delta` path r2 ships — the
  penalty-guard short-circuits the half-life resolution, preserving
  v0+v1 "penalties never decay" semantics.
- **R4 (test style):** e2e tests use `?` propagation and
  `LemmyResult<()>` return per `feedback_lemmy_error_no_std_error.md`
  Case A; helper fns return `LemmyResult<T>`. No `.unwrap()`, no
  `.expect()`, no `dbg!`. Test fixtures use `assert_eq!` /
  `assert!` only.
- **R5 (Task 0 audit):** enumerate ALL probes explicitly (per
  `.claude/rules/pre-phase-harness-audit.md`).
- **R6 (clippy invocations):** all clippy commands use
  `--workspace --features full --no-deps -- -D warnings` uniformly.
- **R7 (test-target compile gate):** any task that edits a struct or
  re-export runs `cargo test --no-run -p lemmy_server --test e2e`.
  Task 2 changes the `emit_reputation_event` private fn signature
  (adds two parameters); this is intra-file (private, no re-export)
  so R7 is **not** invoked. Task 3 adds new pub types in
  `api_common::governance`; R7 IS invoked for Task 3.
- **R8 (features full):** all cargo commands use `--features full
  --workspace`, NEVER `-p <crate> --features full` (per
  `feedback_features_full_p_crate_incompatible.md`).
- **R9 (per-community-tx atomicity for crons):** both crons MUST
  wrap each community's batch in `conn.run_transaction(async |conn| { ... })`
  per `feedback_multi_write_handlers_need_transactions.md` — a
  mid-community crash rolls back THAT community only; earlier-
  committed communities stay (mirror
  `sponsor_liability_grace::run_grace_check_batch`).
- **R10 (dedupe-key idempotency):** the
  `reputation_event_dedupe_key_partial_idx` partial unique index (live
  from r1; verified in §2) means duplicate-`dedupe_key` INSERTs raise
  `UniqueViolation` at the SQL layer. Task 1 + Task 3 use Diesel's
  `.on_conflict_do_nothing()` to consume the violation silently; this
  is the entire idempotency mechanism (NOT an application-level
  `SELECT EXISTS` pre-check, which races concurrent cron ticks
  hypothetically — though the `PARTICIPATION_CRON_RUNNING` guard
  serialises ticks on a single node).
- **R11 (ADR-015 pseudonym discipline):** every governance_log
  payload field naming a person uses `*_pseudonym` (string resolved
  via `actor_pseudonym_helper::get_or_create`), NEVER raw `PersonId`.
  Cron-batch entries use `actor_pseudonym = None` (system-attributed
  per DQ a3d0e9941441-018); the `flag-bad-faith` entry uses
  `actor_pseudonym = Some(admin_pseudonym)` (real admin attribution
  per brief §4).

## 8. Flow design

### Before (governance-v0 HEAD post-r2)

```
[15-min reputation snapshot tick]
  → recompute_snapshot (per-dimension chained-halving + bounds clamp via r2)
     → consumes reputation_event rows from v0 sources:
        - endorsement_created (api_crud/create_endorsement)
        - jury_vote outcomes (api/submit_jury_vote — JuryReliability + ReportingAccuracy only)
        - sponsor_liability (api/sponsor_liability_grace fired branch)
        - founder_seed (api/reputation_snapshot::seed)

[no participation cron]
[no dormancy cron]
[vote-outcome emit covers JuryReliability only — ParticipationConsistency unwired]
[no evidence-quality emit — neither positive heuristic nor admin bad-faith path]
[no flag-bad-faith admin endpoint]
```

### After (r3)

```
[15-min reputation snapshot tick]      ← unchanged (r2 calculator consumes the new rows transparently)
  → recompute_snapshot

[NEW: weekly participation cron tick (cadence: job.participation_interval_days, default 7d)]
  → guarded by PARTICIPATION_CRON_RUNNING + ParticipationCronRunningGuard
  → guarded by BREHON_DISABLE_PARTICIPATION_JOB=1 env-var for tests
  → participation_cron::run_activity_batch(&context):
      → SELECT community_id, creator_id, COUNT(*) FROM comment
        WHERE published_at >= now() - lookback_days AND NOT deleted AND NOT removed
        GROUP BY community_id, creator_id HAVING COUNT(*) >= activity_threshold
      → group by community
      → for each community, run_transaction:
          → for each qualifying person:
              INSERT reputation_event (... +1 ParticipationConsistency, dedupe_key, source_event_type=ParticipationCron)
                ON CONFLICT DO NOTHING
          → governance_log::append(ENTRY_KIND_PARTICIPATION_CRON_TICK, {community_id, iso_week, qualifying_user_count, delta=+1}, actor_pseudonym=None)
  → participation_cron::run_dormancy_batch(&context):
      → SELECT community_id, person_id FROM (
          existing prior_participation_consistency events
        ) LEFT JOIN (
          recent non-deleted/non-removed comments in window
        ) WHERE recent_comments IS NULL
      → group by community
      → for each community, run_transaction:
          → for each dormant person:
              INSERT reputation_event (... -2 ParticipationConsistency, dedupe_key, source_event_type=DormancyCron)
                ON CONFLICT DO NOTHING
          → governance_log::append(ENTRY_KIND_PARTICIPATION_CRON_TICK, {community_id, iso_week, dormant_user_count, delta=-2}, actor_pseudonym=None)

[ON submit_jury_vote → case_decided branch]            ← extends existing run_transaction at line 139
  → existing: emit reputation_event JuryReliability per juror (+1 aligned, -1 outlier)
  → existing: emit reputation_event ReportingAccuracy for reporter (+1 upheld, -1 dismissed)
  → NEW: emit reputation_event ParticipationConsistency for each majority-aligned juror (+1, source=VoteOutcome, dedupe_key=vote_outcome:case:juror)
  → NEW: governance_log::append(ENTRY_KIND_VOTE_OUTCOME_RECORDED, ...) per aligned juror
  → NEW: evidence-cited heuristic — if reporter_id exists AND case_evidence(uploader=reporter) exists AND any winning_rationale.chars() >= threshold:
       emit reputation_event ReportingAccuracy (+1, source=EvidenceQuality, dedupe_key=evidence_cited:case:reporter)
       governance_log::append(ENTRY_KIND_EVIDENCE_QUALITY_RECORDED, ..., trigger="rationale_cited")

[NEW: POST /api/v4/governance/admin/emergency-remove/flag-bad-faith]
  → is_admin check → 403 on non-admin
  → load ModerationCase by case_id → 404 on missing
  → assert status == EmergencyRemove → 400 otherwise
  → run_transaction:
      → emit reputation_event ReportingAccuracy (-1, source=EvidenceQuality, dedupe_key=evidence_bad_faith:case)
        ON CONFLICT DO NOTHING (one bad-faith flag per case)
      → governance_log::append(ENTRY_KIND_EVIDENCE_QUALITY_RECORDED, ..., trigger="admin_flagged_bad_faith", actor_pseudonym=Some(admin))
  → Json { case_id, flagged: true }
```

**Box → task map:**

- `participation_cron.rs` (new module) + `governance/mod.rs` (`pub mod`) +
  `scheduled_tasks.rs` (new tick block) → Task 1.
- `submit_jury_vote.rs` Source 3 (vote-outcome) + Source 4a
  (evidence-cited) emits → Task 2.
- `api_common/governance.rs` (new DTOs) + `admin_emergency_remove.rs`
  (`flag_bad_faith_emergency_report` handler) + `api/routes/lib.rs`
  (route wire + import) → Task 3.
- `e2e.rs` (`v1_rt_r3_fixtures` module + 10 tests) → Task 4.

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit on each
task:

**Schema / type definitions (every task):**

- `crates/db_schema_file/src/enums.rs:609-642` —
  `ReputationDimension` (4 variants — Task 1+2+3 use
  `ParticipationConsistency` + `ReportingAccuracy`);
  `ReputationEventSourceType` (verify `ParticipationCron`,
  `DormancyCron`, `VoteOutcome`, `EvidenceQuality` variants present).
- `crates/db_schema/src/source/governance/reputation_event.rs:20-52` —
  `ReputationEvent` (note `delta: i32`, `dimension`, `dedupe_key:
  Option<String>`, `source_event_type: ReputationEventSourceType`) +
  `ReputationEventInsertForm` (matching fields, plus the
  `source_event_type: Option<ReputationEventSourceType>` arity quirk
  on the insert form — DEFAULT covers omissions per r1).
- `crates/db_schema_file/src/schema.rs:201-227` — `comment` table
  (Task 1 query needs `creator_id`, `community_id`, `published_at`,
  `deleted`, `removed`).
- `crates/db_schema/src/source/governance/case_evidence.rs:1-39` —
  `CaseEvidence` (Task 2 Source 4a queries `uploader_id`; note the
  field is `uploader_id` not `submitter_id`).
- `crates/db_schema/src/source/governance/jury_vote.rs:9-34` —
  `JuryVote` (note `rationale: Option<String>` at line 22; Task 2
  Source 4a checks `winning_rationales` which is already assembled at
  `submit_jury_vote.rs:411-416`).
- `crates/db_schema/src/source/governance/moderation_case.rs` (entire
  file — Task 3 needs `status`, `creator_id`, `community_id`).
- `crates/db_schema/src/source/governance/governance_log.rs:200-220`
  — the r1-shipped ENTRY_KIND consts (`_PARTICIPATION_CRON_TICK`,
  `_VOTE_OUTCOME_RECORDED`, `_EVIDENCE_QUALITY_RECORDED`). Do NOT add
  to this file.
- `crates/api/api/src/governance/governance_log.rs` — shim re-export
  (verify the three r3 consts already re-exported per registry
  `governance-log-entry-kind-registry.md` v1-RT-r1 section).
- `crates/api/api/src/governance/config.rs:873-1026` — DEFAULT_*
  consts r3 reads (verify 10 keys exist: see brief §2.1 enumeration).
- `crates/api/api/src/governance/config.rs:1093-1183` — the key match
  arms r3 reads (verify each key has a `match` branch).

**Existing patterns (per-task MIRROR refs — see §10):**

- `crates/routes/src/utils/scheduled_tasks.rs:50-104` (4 RAII guards +
  atomic-bool patterns) + `:201-255` (15-min snapshot tick — canonical
  cron block with RunningGuard + staleness check) + `:262-285`
  (appeal-window-expiry block) + `:286-375` (sponsor-liability-grace
  block — closest sibling for r3's cron because it does the per-row
  run_transaction pattern Task 1 needs).
- `crates/api/api/src/governance/appeal_window_expiry.rs` (entire file
  — canonical Lemmy-API cron-batch helper module shape; Task 1's
  `participation_cron.rs` mirrors this).
- `crates/api/api/src/governance/sponsor_liability_grace.rs:115-185`
  (`run_grace_check_batch` — per-row `run_transaction` pattern with
  `tracing::warn` on per-row failure; outer batch returns `Ok` regardless).
- `crates/api/api/src/governance/submit_jury_vote.rs:115-145` (handler
  outer; `run_transaction` wrap) + `:157-285` (`process_vote` outer
  steps 1-6) + `:517-626` (`case_decided` branch where Task 2's new
  emits land) + `:577-597` (juror loop for Source 3 co-location) +
  `:600-625` (reporter emit for Source 4a co-location) + `:968-994`
  (`emit_reputation_event` helper — Task 2 extends its signature).
- `crates/api/api/src/governance/admin_emergency_remove.rs` (entire
  file — Task 3 mirrors the outer handler + `run_transaction` shape;
  the new `flag_bad_faith_emergency_report` handler lives right
  alongside `emergency_remove_open_case`).
- `crates/api/api/src/governance/admin_assign_jury.rs:925-980`
  (`write_constraint_relaxation` — actor_pseudonym=None precedent
  for cron-batch / system-attributed governance_log entries; brief
  §4 cites `:932` doc-comment).
- `crates/api/api/src/governance/admin_config.rs:1099` (`is_admin`
  capability-check pattern) + `:1142` (second admin handler shape
  — siblings extending the same file with multiple pub handlers).
- `crates/api/api_common/src/governance.rs:191-203` (`AdminCloseCase`
  + `AdminCloseCaseResponse` — canonical shape for Task 3's new DTOs).
- `crates/api/routes/src/lib.rs:474-520` (existing `/governance` scope
  structure — Task 3 nests `scope("/emergency-remove")` under
  `scope("/admin")`).

**Adjacent test fixtures (Task 4 MIRROR refs):**

- `crates/server/tests/e2e.rs` — every `v1_*_fixtures` module
  (`v1_jm_*`, `v1_sl_*`, `v1_ad_*`, `v1_rt_r1`, `v1_rt_r2`). Pick the
  module closest in shape to r3's e2e needs (Lemmy-context + pool
  + governance writes); MIRROR the chosen sibling's error-shape
  verbatim (Case A per `feedback_lemmy_error_no_std_error.md`).
- `crates/server/tests/e2e.rs:9267-9400` (the `emergency_remove_open_case`
  test region — Task 4 reuses these helpers for the
  `flag-bad-faith` 200-arm test seed).

**Lessons (per `.claude/rules/advisor-orchestrator.md` §2.4 mandatory
file-class lesson injection):**

- `.claude/lessons/feedback_clippy_test_style.md` — Task 4 e2e tests
  use `assert_eq!` / `assert!`; no `unwrap`/`expect`; `?` propagation
  on `LemmyResult<()>`.
- `.claude/lessons/feedback_async_pool_test_pattern.md` — Task 4
  canonical pool/conn/LemmyResult fixture.
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Task 4 e2e
  Case A: mirror sibling fixtures module's error-shape verbatim
  (outer `LemmyResult<()>` + helper `LemmyResult<T>`).
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — Task 4
  authors ONE e2e edit (single fixtures module add); below the ≥2
  edit threshold but the discipline still binds (add the fixtures
  module as a single contiguous block, NOT scattered edits).
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md`
  — Tasks 1 + 2 + 3 all do multi-write inside `run_transaction`
  blocks; the lesson's discipline binds for every emit-site cluster.
- `.claude/lessons/feedback_features_full_workspace_only.md` +
  `.claude/lessons/feedback_features_full_p_crate_incompatible.md` —
  Tasks 1+2+3+4 VALIDATE commands use `--workspace --features full`,
  NEVER `-p <crate> --features full`.
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — every
  §13 task carries `creates:` + `modifies:` + `requires:` YAML.
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md` —
  Tasks 1+2+3 push then write `kind: "validate-pending-laptop"` DQ;
  Task 4 pushes then writes `kind: "validate-pending-laptop-e2e"` DQ.

## 10. Patterns to mirror

Per `feedback_advisor_watchpoint_specificity.md`: every pattern below
cites a specific file:line.

### 10.1 `participation_cron::run_activity_batch` — per-community-tx cron

**Mirror:** `crates/api/api/src/governance/sponsor_liability_grace.rs:115-185`
(`run_grace_check_batch` outer + per-row `run_transaction`).

```rust
//! Multi-source participation cron — Sources 1 + 2 per PRD section 5.3.
//!
//! Runs every `job.participation_interval_days` (default 7): emits
//! `+1 participation_consistency` per active user per community
//! (Source 1) and `-2 participation_consistency` per dormant user
//! per community (Source 2). Per-community `run_transaction`
//! isolation per PRD section 5.3 + `feedback_multi_write_handlers_need_transactions.md`.
//!
//! ## Idempotency
//!
//! Both batches write `reputation_event` rows with `dedupe_key` set
//! to a per-(community, person, ISO week) string. The
//! `reputation_event_dedupe_key_partial_idx` partial unique index
//! (r1-shipped) catches repeat-tick inserts; Diesel's
//! `.on_conflict_do_nothing()` consumes the violation silently. A
//! second tick within the same ISO week is a no-op (zero new rows).
//!
//! ## Actor attribution (ADR-015 + DQ a3d0e9941441-018)
//!
//! `governance_log` entries from this module use `actor_pseudonym =
//! None` (system-attributed). Mirror v0 precedent at
//! `admin_assign_jury.rs:932` doc-comment.

use chrono::{Datelike, Duration, Utc};
use diesel::{ExpressionMethods, QueryDsl, dsl::count_star, insert_into};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::governance::reputation_event::ReputationEventInsertForm;
use lemmy_db_schema_file::{
  PersonId,
  enums::{ReputationDimension, ReputationEventSourceType},
  schema::{comment, reputation_event},
};
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::LemmyResult;
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::governance::{
  config::{self, ConfigCache, Scope},
  governance_log::{self, ENTRY_KIND_PARTICIPATION_CRON_TICK},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ParticipationBatchOutcome {
  pub communities_processed: usize,
  pub events_emitted: usize,
  pub events_deduped: usize,
}

/// Source 1 — weekly activity cron. SELECTs per-(community, person)
/// comment counts in the lookback window; for each qualifying person,
/// emits `+1 participation_consistency` with dedupe-key idempotency.
pub async fn run_activity_batch(
  context: &LemmyContext,
) -> LemmyResult<ParticipationBatchOutcome> {
  // ... per the 5-step pattern in section 4 + cargo-output-capture rule.
}

/// Source 2 — weekly dormancy cron. SELECTs (community, person) pairs
/// with a prior participation_consistency event but zero recent
/// comments; emits `-2 participation_consistency` with dedupe-key
/// idempotency.
pub async fn run_dormancy_batch(
  context: &LemmyContext,
) -> LemmyResult<ParticipationBatchOutcome> {
  // ... mirror run_activity_batch shape; query joins reputation_event
  // (prior +1 participation events) LEFT ANTI JOIN comment (no recent
  // activity).
}
```

The exact body — discovery query SQL, per-community tx body, dedupe-key
format — is left to the impl-task subagent to derive from the MIRROR
refs + the brief's recipe (sections 2.1 + 4). Plan-time specifying body
shape verbatim risks Sonnet trying to LITERAL the plan against a slightly
different runtime SQL surface (`diesel::dsl::sum` vs `count_star` vs
`group_by` shape) — the MIRROR refs are the contract.

### 10.2 `submit_jury_vote::emit_reputation_event` extended signature

**Mirror:** `crates/api/api/src/governance/submit_jury_vote.rs:968-994`
(current signature).

```rust
/// One-shot insert helper for `reputation_event`. Pulled out to keep
/// the handler body readable and to ensure every reputation write
/// goes through the same shape (no inline insert forms).
///
/// `source_event_type` + `dedupe_key` are explicit per r3 per PRD
/// section 5.3 + 7. The v0 callers (juror reliability + reporter
/// accuracy at lines 583, 615) pass
/// `ReputationEventSourceType::JuryVote` + `None` to preserve
/// byte-identical v0 behaviour at those sites. The r3 callers
/// (vote-outcome juror loop + evidence-cited reporter) pass
/// `VoteOutcome` / `EvidenceQuality` + a populated `dedupe_key`.
async fn emit_reputation_event(
  conn: &mut diesel_async::AsyncPgConnection,
  person_id: PersonId,
  community_id: Option<CommunityId>,
  dimension: ReputationDimension,
  delta: i32,
  source_case_id: ModerationCaseId,
  reason: &str,
  source_event_type: ReputationEventSourceType,
  dedupe_key: Option<String>,
) -> LemmyResult<()> {
  let form = ReputationEventInsertForm {
    person_id,
    community_id,
    dimension,
    delta,
    source_case_id: Some(source_case_id),
    source_report_id: None,
    reason: reason.to_string(),
    expires_at: None,
    dedupe_key,
    source_event_type: Some(source_event_type),
  };
  insert_into(reputation_event::table)
    .values(&form)
    .on_conflict_do_nothing()
    .execute(conn)
    .await?;
  Ok(())
}
```

The `.on_conflict_do_nothing()` addition is intentional: v0 emit sites
never set `dedupe_key`, so the partial unique index never matches; the
on-conflict clause is inert for them. r3 emit sites set `dedupe_key`;
the on-conflict clause silently de-dupes repeat emits (e.g. a juror's
vote-outcome emit re-firing due to a retry).

### 10.3 Source 3 vote-outcome emit (in submit_jury_vote.rs case_decided branch)

**Mirror:** `crates/api/api/src/governance/submit_jury_vote.rs:577-597`
(the existing juror loop where Source 3 co-locates).

The impl-task subagent extends the existing loop body. For each juror
whose `juror_decision == winning_decision`:

1. Resolve `juror_pseudonym` via
   `actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), other_juror_id).await?`
   (mirror the resolution shape at
   `admin_emergency_remove.rs:276`).
2. Read `deltas.participation_juror_aligned` via
   `config::get_int(&mut cache, &mut (&mut *conn).into(),
   Scope::Instance, "deltas.participation_juror_aligned").await?`
   (cached after the first iteration).
3. `i32::try_from(...)` per R1.
4. Call `emit_reputation_event(conn, other_juror_id,
   case_row.community_id, ReputationDimension::ParticipationConsistency,
   participation_delta, data.case_id, "vote_outcome_aligned",
   ReputationEventSourceType::VoteOutcome,
   Some(format!("vote_outcome:{}:{}", data.case_id.0, juror_pseudonym)))?`.
5. Call `governance_log::append(&mut conn.into(),
   ENTRY_KIND_VOTE_OUTCOME_RECORDED, json!({ "case_id": data.case_id.0,
   "juror_pseudonym": juror_pseudonym, "dimension":
   "participation_consistency", "delta": participation_delta,
   "source_event_type": "vote_outcome" }), Some(juror_pseudonym))?`.

Minority jurors (`juror_decision != winning_decision`): skip the
new Source 3 emit (and the gov-log row). The existing v0
jury_reliability `-1 outlier_vote` emit at line 591-595 still fires
for them.

### 10.4 Source 4a evidence-cited heuristic (in submit_jury_vote.rs case_decided branch)

**Mirror:** `crates/api/api/src/governance/submit_jury_vote.rs:600-625`
(the existing reporter emit — Source 4a follows immediately AFTER).

```rust
// 12.5 (r3) — Source 4a evidence-cited heuristic per PRD section 5.3.
if let Some(reporter_id) = case_row.creator_id {
  let reporter_has_evidence: bool = diesel::select(diesel::dsl::exists(
    case_evidence::table
      .filter(case_evidence::case_id.eq(data.case_id))
      .filter(case_evidence::uploader_id.eq(reporter_id)),
  ))
  .get_result(conn)
  .await?;
  if reporter_has_evidence {
    let threshold_i64 = config::get_int(
      &mut cache,
      &mut (&mut *conn).into(),
      Scope::Instance,
      "participation.evidence_cited_rationale_threshold_chars",
    )
    .await?;
    let threshold_chars = usize::try_from(threshold_i64.max(0)).unwrap_or(0);
    let cited = winning_rationales
      .iter()
      .any(|r| r.chars().count() >= threshold_chars);
    if cited {
      let evidence_delta_i64 = config::get_int(
        &mut cache,
        &mut (&mut *conn).into(),
        Scope::Instance,
        "deltas.evidence_cited",
      )
      .await?;
      let evidence_delta = i32::try_from(evidence_delta_i64).map_err(|_e| {
        LemmyErrorType::Unknown(format!(
          "deltas.evidence_cited ({evidence_delta_i64}) overflows i32"
        ))
      })?;
      let reporter_pseudonym = actor_pseudonym_helper::get_or_create(
        &mut (&mut *conn).into(),
        reporter_id,
      )
      .await?;
      emit_reputation_event(
        conn,
        reporter_id,
        case_row.community_id,
        ReputationDimension::ReportingAccuracy,
        evidence_delta,
        data.case_id,
        "evidence_cited",
        ReputationEventSourceType::EvidenceQuality,
        Some(format!("evidence_cited:{}:{}", data.case_id.0, reporter_pseudonym)),
      )
      .await?;
      governance_log::append(
        &mut conn.into(),
        ENTRY_KIND_EVIDENCE_QUALITY_RECORDED,
        json!({
          "case_id": data.case_id.0,
          "reporter_pseudonym": reporter_pseudonym,
          "dimension": "reporting_accuracy",
          "delta": evidence_delta,
          "source_event_type": "evidence_quality",
          "trigger": "rationale_cited",
        }),
        Some(reporter_pseudonym),
      )
      .await?;
    }
  }
}
```

### 10.5 `flag_bad_faith_emergency_report` handler shape

**Mirror:** `crates/api/api/src/governance/admin_emergency_remove.rs:75-101`
(`emergency_remove_open_case` outer wrap) + `:103-330`
(`process_emergency_remove` inner) + `crates/api/api/src/governance/admin_config.rs:1086-1099`
(`admin_get_config` capability-check + LocalUserView signature).

```rust
/// Admin-flagged bad-faith report against an EmergencyRemove-status
/// case. Emits `-1 reporting_accuracy` for the case reporter via a
/// dedupe-keyed `reputation_event` row plus an
/// `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED` governance_log entry
/// attributed to the admin. PRD section 5.3 source 4b + ADR-013.
pub async fn flag_bad_faith_emergency_report(
  Json(data): Json<FlagBadFaithEmergencyReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<FlagBadFaithEmergencyReportResponse>> {
  is_admin(&local_user_view)?;

  let admin_id = local_user_view.person.id;
  let admin_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?;
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let case_id = data.case_id;
  let admin_pseudonym_for_tx = admin_pseudonym;

  let flagged = conn
    .run_transaction(async |conn| {
      process_flag_bad_faith(conn, admin_pseudonym_for_tx, case_id).await
    })
    .await?;

  Ok(Json(FlagBadFaithEmergencyReportResponse {
    case_id,
    flagged,
  }))
}

async fn process_flag_bad_faith(
  conn: &mut AsyncPgConnection,
  admin_pseudonym: String,
  case_id: ModerationCaseId,
) -> LemmyResult<bool> {
  // 1. Load case + assert status.
  let case_row: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await
    .map_err(|_e| LemmyErrorType::NotFound)?;
  if case_row.status != CaseStatus::EmergencyRemove {
    return Err(
      LemmyErrorType::Unknown(format!(
        "case {} is in status {:?}; flag-bad-faith requires EmergencyRemove",
        case_id.0, case_row.status
      ))
      .into(),
    );
  }
  let reporter_id = case_row.creator_id.ok_or_else(|| {
    LemmyErrorType::Unknown(format!(
      "case {} has no reporter (creator_id); flag-bad-faith requires a reporter",
      case_id.0
    ))
  })?;

  // 2. Emit reputation_event + governance_log.
  let reporter_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), reporter_id).await?;
  let mut cache = ConfigCache::new();
  let evidence_delta_i64 = config::get_int(
    &mut cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "deltas.evidence_bad_faith",
  )
  .await?;
  let evidence_delta = i32::try_from(evidence_delta_i64).map_err(|_e| {
    LemmyErrorType::Unknown(format!(
      "deltas.evidence_bad_faith ({evidence_delta_i64}) overflows i32"
    ))
  })?;
  // file-private emit_reputation_event_local helper (mirrors
  // submit_jury_vote.rs:968 verbatim but lives in admin_emergency_remove.rs).
  emit_reputation_event_local(
    conn,
    reporter_id,
    case_row.community_id,
    ReputationDimension::ReportingAccuracy,
    evidence_delta,
    case_id,
    "evidence_bad_faith",
    ReputationEventSourceType::EvidenceQuality,
    Some(format!("evidence_bad_faith:{}", case_id.0)),
  )
  .await?;
  governance_log::append(
    &mut conn.into(),
    ENTRY_KIND_EVIDENCE_QUALITY_RECORDED,
    json!({
      "case_id": case_id.0,
      "reporter_pseudonym": reporter_pseudonym,
      "dimension": "reporting_accuracy",
      "delta": evidence_delta,
      "source_event_type": "evidence_quality",
      "trigger": "admin_flagged_bad_faith",
    }),
    Some(admin_pseudonym),
  )
  .await?;
  Ok(true)
}
```

The `emit_reputation_event_local` is a file-private helper (NOT
shared with `submit_jury_vote.rs::emit_reputation_event`) — keeping the
two functions independent avoids a cross-file refactor that would
expand Task 3's scope. Both helpers have the same shape; future
consolidation (post-r3) may pull them into
`crates/api/api/src/governance/reputation_event_emit.rs` if a third
emitter site needs the shape.

### 10.6 `routes/lib.rs` scope nesting for `/emergency-remove`

**Mirror:** `crates/api/routes/src/lib.rs:474-520` (existing
`/governance` scope structure; the nested `scope("/config")` at line
503-508 is the pattern for adding a new sub-scope under `/admin`).

```rust
// Inside scope("/admin") at line 493, alongside existing nested scopes:
.service(
  scope("/emergency-remove")
    .route("/flag-bad-faith", post().to(flag_bad_faith_emergency_report)),
),
```

The handler import joins the existing `governance::{...}` block at
lines 31-49:

```rust
governance::{
  // ... existing imports ...
  admin_emergency_remove::flag_bad_faith_emergency_report,
  // ... rest ...
},
```

### 10.7 `api_common::governance` DTOs

**Mirror:** `crates/api/api_common/src/governance.rs:182-203`
(`AdminCloseCase` + `AdminCloseCaseResponse` — same single-`case_id`
shape).

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Request payload for `POST /api/v4/governance/admin/emergency-remove/flag-bad-faith`.
/// Admin-only per ADR-013 + PRD section 5.3 source 4b. Flags the
/// reporter of an `EmergencyRemove`-status case as bad-faith; emits
/// `-1 reporting_accuracy`.
pub struct FlagBadFaithEmergencyReport {
  pub case_id: ModerationCaseId,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from `flag_bad_faith_emergency_report`. `flagged: true`
/// confirms the reputation_event row was written (or de-duped to a
/// prior identical row via the dedupe_key partial unique index).
pub struct FlagBadFaithEmergencyReportResponse {
  pub case_id: ModerationCaseId,
  pub flagged: bool,
}
```

### 10.8 `scheduled_tasks.rs` new tick block

**Mirror:** `crates/routes/src/utils/scheduled_tasks.rs:201-255`
(15-min snapshot block — canonical RunningGuard pattern).

The block lands AFTER the federation-replay-cleanup tick (after
line 449), BEFORE the `loop { scheduler.run_pending().await; ... }`
at line 452.

Pattern (verbatim shape; impl-task subagent fills in the import / scope
details):

```rust
// v1-RT-r3 participation cron (Sources 1 + 2). Interval is read from
// `job.participation_interval_days` at scheduler setup (default 7).
// Both run_activity_batch and run_dormancy_batch dispatch under one
// tick + one RunningGuard (per brief — "in same scheduled_tasks.rs
// tick, registered after the activity cron block"). Disabled in tests
// via BREHON_DISABLE_PARTICIPATION_JOB=1 (mirrors
// BREHON_DISABLE_SNAPSHOT_JOB at line 209).
let context_participation = context.reset_request_count();
let participation_pool = &mut context.pool();
let participation_interval_days_i64: i64 = lemmy_api::governance::config::get_int(
  &mut lemmy_api::governance::config::ConfigCache::new(),
  participation_pool,
  lemmy_api::governance::config::Scope::Instance,
  "job.participation_interval_days",
)
.await
.unwrap_or(7);
let participation_interval_days: u32 =
  u32::try_from(participation_interval_days_i64).unwrap_or(7);
scheduler
  .every(CTimeUnits::days(participation_interval_days))
  .run(move || {
    let context = context_participation.reset_request_count();
    async move {
      if std::env::var("BREHON_DISABLE_PARTICIPATION_JOB").as_deref() == Ok("1") {
        return;
      }
      if PARTICIPATION_CRON_RUNNING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
      {
        warn!("participation_cron: previous batch still running, skipping this tick");
        return;
      }
      let _guard = ParticipationCronRunningGuard;
      lemmy_api::governance::participation_cron::run_activity_batch(&context)
        .await
        .inspect_err(|e| warn!("Failed to run participation activity batch: {e}"))
        .ok();
      lemmy_api::governance::participation_cron::run_dormancy_batch(&context)
        .await
        .inspect_err(|e| warn!("Failed to run participation dormancy batch: {e}"))
        .ok();
    }
  });
```

The matching `static PARTICIPATION_CRON_RUNNING: AtomicBool =
AtomicBool::new(false);` + `struct ParticipationCronRunningGuard;` +
`impl Drop` lands at module scope (alongside the four existing
RunningGuard pairs at lines 56-104). Mirror the `RunningGuard` shape
verbatim.

## 11. Files to change

### `crates/api/api` (1 crate, 4 files across tasks)

- `crates/api/api/src/governance/participation_cron.rs` — **NEW** —
  Sources 1+2 cron helper module; `run_activity_batch` +
  `run_dormancy_batch` pub async fns + `ParticipationBatchOutcome`
  struct (Task 1).
- `crates/api/api/src/governance/mod.rs` — add `pub mod
  participation_cron;` declaration in alphabetical order (between
  `pub mod list_my_jury_queue;` and `pub mod redaction;`) (Task 1).
- `crates/api/api/src/governance/submit_jury_vote.rs` — extend
  `emit_reputation_event` signature (add `source_event_type` +
  `dedupe_key` params); update the 2 existing JuryVote call sites to
  pass `ReputationEventSourceType::JuryVote` + `None`; insert Source 3
  vote-outcome emit inside the existing juror loop at lines 577-597;
  insert Source 4a evidence-cited heuristic block after lines 600-625
  (Task 2).
- `crates/api/api/src/governance/admin_emergency_remove.rs` — append
  the new pub handler `flag_bad_faith_emergency_report` + inner
  `process_flag_bad_faith` + file-private `emit_reputation_event_local`
  helper after the existing `process_emergency_remove` at line 330
  (Task 3).

### `crates/api/api_common` (1 crate, 1 file)

- `crates/api/api_common/src/governance.rs` — add the two new pub
  request/response types (`FlagBadFaithEmergencyReport` +
  `FlagBadFaithEmergencyReportResponse`) (Task 3).

### `crates/api/routes` (1 crate, 1 file)

- `crates/api/routes/src/lib.rs` — add the
  `admin_emergency_remove::flag_bad_faith_emergency_report` import in
  the `governance::{...}` block; nest `scope("/emergency-remove")`
  under the existing `scope("/admin")` (Task 3).

### `crates/routes` (1 crate, 1 file)

- `crates/routes/src/utils/scheduled_tasks.rs` — add the
  `PARTICIPATION_CRON_RUNNING` AtomicBool + `ParticipationCronRunningGuard`
  RAII struct (at module scope alongside lines 56-104); add the new
  `scheduler.every(CTimeUnits::days(participation_interval_days)).run(...)`
  block after the federation-replay-cleanup block at line 449 (Task 1).

### `crates/server` (1 crate, 1 file)

- `crates/server/tests/e2e.rs` — add ONE new `v1_rt_r3_fixtures`
  module containing 10 new e2e tests + shared seeding helpers (Task 4).

### Struct-field add: none

r3 does not add fields to any public struct. The only public-type
additions are 2 new request/response DTOs in `api_common`; no new
fields on existing structs; no constructor-site enumeration required
per `feedback_planner_enumerate_struct_callsites_for_addfield.md`.

The internal `emit_reputation_event` signature in `submit_jury_vote.rs`
gains 2 parameters but is `async fn` (private, not `pub`). Only 2
existing callers (both in the same file at lines 583 + 615); both
updated in Task 2 alongside the new callers. No cross-file or
cross-crate impact (verified via `rg "emit_reputation_event"
crates/`).

## 12. NOT building in v1-RT-r3

- **New migrations** — r3 is emitter wiring + 1 admin handler only.
  Schema, partial unique index, ENUM variants all pre-landed by r1
  (`migrations/2026-05-10-000000-0000_add_reputation_event_v1_columns/up.sql`).
  Any §13 task proposing a migration is a scope violation per brief §4;
  advisor catch-fire tripwire per stop-and-ask #1.
- **New `governance_config` keys** — all 10 r3-relevant keys pre-landed
  by r1 + v1-AD-a (see §6 enumeration). Any §13 task proposing a seed
  migration is a scope violation per brief §4; advisor catch-fire
  tripwire per stop-and-ask #2.
- **New `ENTRY_KIND_*` consts** — r1 pre-landed all 7 r3-relevant
  consts (only 3 used by r3); the registry at
  `governance-log-entry-kind-registry.md` v1-RT-r1 section names r3
  as the call-site landing phase. Any §13 task proposing a new const
  in `crates/db_schema/src/source/governance/governance_log.rs` is a
  scope violation per brief §4; advisor catch-fire tripwire per
  stop-and-ask #3.
- **New `ReputationEventSourceType` variants** — r1 pre-landed all 9
  variants including the 4 r3 needs (`ParticipationCron`,
  `DormancyCron`, `VoteOutcome`, `EvidenceQuality`).
- **`feature.reputation_v1_decay_enabled` gate on emit sites** —
  r3 emits unconditionally; the flag gates the r2 calculator that
  *consumes* events, not the events themselves (PRD §11 + ADR-008).
  Gating emits would prevent calibration / backfill operators need.
- **Rollup cron** — `ENTRY_KIND_ROLLUP_RECOMPUTED` is v1-RT-r5-owned.
- **`decay.*` knob admin handler / `decay_knob_changed` emit** —
  v1-RT-r2-owned (and itself deferred to v1-AD-b admin-config write).
- **`sponsor_allowlist` table / admin handlers** — v1-RT-r4-owned.
- **5 carry-forward CR fixes** (#19, #20, #21, #22, #31) —
  v1-RT-r6-owned per PRD §11.
- **Shared `emit_reputation_event` helper across files** — the file-
  private helpers in `submit_jury_vote.rs` and `admin_emergency_remove.rs`
  stay distinct in r3. Cross-file unification is a post-r3 refactor
  (likely in r6 carry-forward or v1-RT-r4 if a third emit site needs
  the same shape).
- **`appeal_window_expiry`-style staleness check** for the
  participation cron — r3's cron is informational; no per-tick
  staleness telemetry. v0 lessons (PRD §6.3 grace-check staleness
  observability) do not transfer because r3 has no SLA on per-row
  emit latency.
- **Cron emit sites outside `submit_jury_vote.rs::case_decided`** —
  the no-quorum / deadlock / appeal-decided branches do NOT emit
  Sources 3/4a; those branches don't produce a single `winning_decision`
  with a populated `winning_rationales` (Source 4a precondition), and
  no juror-aligned/minority split exists outside the regular Decided
  path (Source 3 precondition).

---

## 13. Step-by-step tasks

> **Cohort dispatch:** Task 0 is a barrier (Junior task, non-`[P]`).
> Tasks 1 + 2 + 3 share zero IMPLEMENT files (Task 1 in
> `crates/routes` + `crates/api/api/.../participation_cron.rs` +
> `governance/mod.rs`; Task 2 in `crates/api/api/.../submit_jury_vote.rs`;
> Task 3 in `crates/api/api_common`, `crates/api/api/.../admin_emergency_remove.rs`,
> `crates/api/routes/src/lib.rs`) → **`[P]` cohort of 3** dispatched
> simultaneously. Task 4 is a barrier (`requires: 1, 2, 3` — e2e
> validation needs all three impl tasks merged on the phase branch).
> Task 5 (retro) is a final barrier.
>
> **Shape G:** SUSPENDED per DQ #229. r3 is a pre-Shape-G plan:
> cargo runs on laptop via the `validate-pending-laptop` handler.
> impl-task subagents on the EliteDesk write the
> `kind: "validate-pending-laptop"` DQ entry after push (Tasks 1+2+3);
> Task 4 writes `kind: "validate-pending-laptop-e2e"` for the e2e command.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for v1-RT-r3; confirm branch is
`phase-v1-RT-r3`; confirm prior phase's r2 + r1 deliverables are intact
on the base; confirm pre-existing clippy baseline is clean; confirm
docker daemon up.

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5:
enumerate ALL probes explicitly):**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — wrapper sanity (cargo-check honors -p)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/audit-cargo-check-p.log 2>&1"
tail -20 .claude/audit-cargo-check-p.log
# EXPECT: only lemmy_utils compiles

# Probe 2 — feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/audit-cargo-check-features.log 2>&1"
echo "exit: $?"
tail -20 .claude/audit-cargo-check-features.log
# EXPECT: workspace compiles with --features full; exit 0

# Probe 3 — cargo-test wrapper honors target selection
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --no-run --features full > .claude/audit-cargo-test.log 2>&1"
tail -20 .claude/audit-cargo-test.log
# EXPECT: e2e test target compiles workspace-wide

# Probe 4 — wrappers fail loud on cargo errors (exit-code propagation)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --no-run --features nonexistent_xyz > .claude/audit-cargo-test-negative.log 2>&1"
echo "cargo-test.bat exit on bogus feature: $?"
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"
# EXPECT: BOTH non-zero (typically 101)

# Probe 5 — clippy baseline against post-r2 HEAD
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/audit-clippy-baseline.log 2>&1"
echo "exit: $?"
tail -40 .claude/audit-clippy-baseline.log
# EXPECT: exit 0

# Probe 6 — current branch is phase-v1-RT-r3
git branch --show-current
# EXPECT: phase-v1-RT-r3

# Probe 7 — r1 + r2 deliverables landed on the base
git log governance-v0 --oneline | head -25
# EXPECT: r1 (PR #126) + r2 (PR ref-TBD) merge commits visible

# Probe 8 — r1 schema columns (dedupe_key + source_event_type) present
rg "dedupe_key" crates/db_schema/src/source/governance/reputation_event.rs | wc -l
# EXPECT: at least 3 (struct field decl + form field decl + doc-comment)
rg "source_event_type" crates/db_schema/src/source/governance/reputation_event.rs | wc -l
# EXPECT: at least 3
rg "ParticipationCron|DormancyCron|VoteOutcome|EvidenceQuality" crates/db_schema_file/src/enums.rs | wc -l
# EXPECT: 4

# Probe 9 — r1 ENTRY_KIND consts pre-landed
rg "ENTRY_KIND_PARTICIPATION_CRON_TICK|ENTRY_KIND_VOTE_OUTCOME_RECORDED|ENTRY_KIND_EVIDENCE_QUALITY_RECORDED" crates/db_schema/src/source/governance/governance_log.rs | wc -l
# EXPECT: 3
rg "ENTRY_KIND_PARTICIPATION_CRON_TICK|ENTRY_KIND_VOTE_OUTCOME_RECORDED|ENTRY_KIND_EVIDENCE_QUALITY_RECORDED" crates/api/api/src/governance/governance_log.rs | wc -l
# EXPECT: 3 (shim re-export)

# Probe 10 — r1+v1-AD seeded all 10 r3-relevant config keys
rg 'deltas\.participation_weekly_active|participation\.dormancy_window_days|deltas\.participation_dormant|deltas\.participation_juror_aligned|participation\.activity_threshold_comments|participation\.lookback_days|deltas\.evidence_cited|deltas\.evidence_bad_faith|participation\.evidence_cited_rationale_threshold_chars|job\.participation_interval_days' crates/api/api/src/governance/config.rs | wc -l
# EXPECT: at least 30 (each key appears in DEFAULT_* const, match arm, key-list array, seed-fragment — at least 3 occurrences per key × 10 keys)

# Probe 11 — partial unique index on dedupe_key present in migration
rg "reputation_event_dedupe_key_partial_idx" migrations/2026-05-10-000000-0000_add_reputation_event_v1_columns/up.sql | wc -l
# EXPECT: at least 1 (CREATE UNIQUE INDEX line)

# Probe 12 — submit_jury_vote.rs emit_reputation_event helper present
rg "^async fn emit_reputation_event" crates/api/api/src/governance/submit_jury_vote.rs | wc -l
# EXPECT: 1
rg "Some\(ReputationEventSourceType::JuryVote\)" crates/api/api/src/governance/submit_jury_vote.rs | wc -l
# EXPECT: at least 1 (the hard-coded source in the v0 helper body — Task 2 changes this)

# Probe 13 — concurrent-PR check (no other PR touches r3's IMPLEMENT files)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("participation_cron|submit_jury_vote|admin_emergency_remove|scheduled_tasks|api_common/src/governance|api/routes/src/lib")) | {number, title, headRefName}'
# EXPECT: empty output; if any other lane is touching r3 files, STOP

# Probe 14 — registry invariant: ENTRY_KIND const count == unique string-literal count
rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
# EXPECT: numeric count A
rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs \
  | awk -F: '/ENTRY_KIND_/ {print}' \
  | grep -oE '"[a-z_]+"' | sort -u | wc -l
# EXPECT: same count B == A
rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs \
  | awk -F: '/ENTRY_KIND_/ {print}' \
  | grep -oE '"[a-z_]+"' | sort | uniq -d
# EXPECT: empty
```

**EXPECT block:**
- Probes 0..3, 5..14 exit 0 (or as documented per probe)
- Probe 4 exits NON-ZERO (negative test confirms exit-code propagation)
- Probe 6 returns `phase-v1-RT-r3`
- Probe 8-12 confirm r1 + r2 deliverables intact (mechanical checks)
- Probe 13 returns empty (no concurrent PR overlap)
- Probe 14: count A == count B; uniq -d empty (registry invariant
  holds; r3 is consuming pre-landed consts, not adding new ones)

**No commit at Task 0** — verification only. Any probe failure files
a `kind: "blocker"` DQ pending and stops.

### Task 1 [P]: participation cron module + scheduler tick

**ACTION:** Create the new `lemmy_api::governance::participation_cron`
module (Sources 1 + 2 cron helpers) and wire the new scheduler block in
`crates/routes/src/utils/scheduled_tasks.rs` that calls both batches
under one tick + one RunningGuard.

**FILES:**

```yaml
creates:
  - crates/api/api/src/governance/participation_cron.rs   # Sources 1 + 2 cron module
modifies:
  - crates/api/api/src/governance/mod.rs                  # add pub mod participation_cron
  - crates/routes/src/utils/scheduled_tasks.rs            # new RunningGuard + scheduler tick block
requires: []
```

**IMPLEMENT (file 1 of 3):** in
`crates/api/api/src/governance/participation_cron.rs`, create the
module per §10.1 shape. Both `run_activity_batch` and
`run_dormancy_batch`:
- Read config knobs through a per-batch `ConfigCache` (R2).
- Issue a single discovery query against the `comment` table (mirror
  the Diesel patterns in `scheduled_tasks.rs:613-639` for raw filter
  shapes; mirror `sponsor_liability_grace.rs:135-142` for the `let
  candidates: Vec<...> = ... .load(conn).await?` shape).
- Group results by community via `HashMap<CommunityId, Vec<(PersonId, ...)>>`.
- For each community, `conn.run_transaction(async |conn| { ... })` per R9:
  - Per qualifying person, `insert_into(reputation_event::table)
    .values(&form).on_conflict_do_nothing().execute(conn).await?` (R10).
  - One `governance_log::append(&mut (&mut *conn).into(),
    ENTRY_KIND_PARTICIPATION_CRON_TICK, json!({...}), None)` per
    community per batch.
- Per-batch outer error policy: catch per-community errors with
  `tracing::warn`; outer fn returns `Ok(...)` regardless (mirror
  `sponsor_liability_grace.rs:165-176`).
- `ParticipationBatchOutcome` returned for telemetry.

ISO week derivation: use `chrono::Datelike::iso_week` on `Utc::now()`
and format as `format!("{}-W{:02}", iso.year(), iso.week())` to get
`"2026-W21"` shape. This becomes the `dedupe_key` suffix.

Discovery query shapes (planner-level — impl-task subagent finalises
the exact Diesel form):

- **Activity**: `comment` table filtered on `published_at >= (now -
  lookback_days)` AND `NOT deleted` AND `NOT removed`, grouped by
  `(community_id, creator_id)`, HAVING `count(*) >= activity_threshold`.
  This is a Diesel `group_by` + `having` query; alternatively a raw
  `sql_query` if Diesel's group_by ergonomics are awkward for the
  HAVING clause (mirror `scheduled_tasks.rs:510-526` for the raw
  `sql_query` shape if needed).
- **Dormancy**: `(community_id, person_id)` pairs with `EXISTS (SELECT
  1 FROM reputation_event WHERE person_id=$p AND community_id=$c AND
  dimension='ParticipationConsistency')` (prior participation event)
  AND `NOT EXISTS (SELECT 1 FROM comment WHERE creator_id=$p AND
  community_id=$c AND published_at >= now - dormancy_window_days AND
  NOT deleted AND NOT removed)`. Single SQL query; LEFT ANTI JOIN
  pattern via `NOT EXISTS` subquery.

**IMPLEMENT (file 2 of 3):** in
`crates/api/api/src/governance/mod.rs`, add `pub mod participation_cron;`
in alphabetical order. Verbatim insertion between line 38
(`pub mod list_my_jury_queue;`) and line 39 (`pub mod redaction;`).

**IMPLEMENT (file 3 of 3):** in
`crates/routes/src/utils/scheduled_tasks.rs`:
- At module scope alongside lines 56-104, add
  `static PARTICIPATION_CRON_RUNNING: AtomicBool = AtomicBool::new(false);`
  + `struct ParticipationCronRunningGuard;` + `impl Drop for
  ParticipationCronRunningGuard { fn drop(&mut self) {
  PARTICIPATION_CRON_RUNNING.store(false, Ordering::Release); } }`.
- After the federation-replay-cleanup `.run()` block at line 449,
  insert the new tick block per §10.8 verbatim (with the impl-task
  subagent filling in the import / scope details).

**MIRROR:** `appeal_window_expiry.rs` (cron helper module shape);
`sponsor_liability_grace.rs:115-185` (per-row run_transaction
pattern); `scheduled_tasks.rs:201-255` (RunningGuard + scheduler
tick); `scheduled_tasks.rs:163` (existing `all_active_counts` as
query design reference per DQ a3d0e9941441-023).

**GOTCHA:**
- `comment` schema field names: `creator_id`, `community_id`,
  `published_at`, `deleted`, `removed` — verified at schema.rs:201-227.
  Filter syntax: `comment::deleted.eq(false)` AND
  `comment::removed.eq(false)`.
- `Datelike::iso_week()` returns `chrono::IsoWeek`; format as
  `format!("{}-W{:02}", iso.year(), iso.week())` to get
  `"2026-W21"` shape. Same format BOTH batches use, so the same ISO
  week always matches the same dedupe-key segment regardless of
  activity-vs-dormancy emit.
- Diesel's `.on_conflict_do_nothing()` (untargeted form) works for
  partial unique indexes — DO NOT use the targeted
  `.on_conflict(reputation_event::dedupe_key).do_nothing()` form,
  which requires a non-partial column constraint.
- The env-var check comes FIRST in the closure body (BEFORE the
  guard acquisition) — mirror the order at
  `scheduled_tasks.rs:320-325` (sponsor-liability-grace block) — to
  avoid leaking guard slots when tests disable the job.
- `lemmy_api::governance::participation_cron::run_activity_batch`
  is called from `crates/routes` — the `lemmy_routes` crate already
  depends on `lemmy_api` (verified via existing
  `lemmy_api::governance::reputation_snapshot::run_snapshot_batch`
  call at `scheduled_tasks.rs:220`); no new Cargo.toml dep needed.

**VALIDATE (story-checkpoint feeds §16a Stories 1 + 2):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r3-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-task1-check.log
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r3-task1-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-RT-r3-task1-clippy.log
# EXPECT: exit 0
```

**Post-validate:** impl-task writes `kind: "validate-pending-laptop"`
DQ entry per `advisor-orchestrator.md` §5.2. Required fields:
`commands` (the 2 VALIDATE commands above), `branch:
"phase-v1-RT-r3"`, `phase_task: 1`. Pushes worker branch. Advisor
laptop session mutates the entry.

### Task 2 [P]: vote-outcome + evidence-cited emit in submit_jury_vote

**ACTION:** Extend `submit_jury_vote.rs::emit_reputation_event` to
accept `source_event_type` + `dedupe_key`; update the 2 existing
JuryVote call sites; insert Source 3 vote-outcome emit inside the
existing juror loop in the `case_decided` branch; insert Source 4a
evidence-cited heuristic immediately after the existing reporter emit.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/submit_jury_vote.rs   # emit helper signature + Sources 3 + 4a emit
requires: []
```

**IMPLEMENT (file 1 of 1):**

1. Extend `emit_reputation_event` (lines 968-994) per §10.2: add
   `source_event_type: ReputationEventSourceType` and `dedupe_key:
   Option<String>` parameters; populate matching fields in the
   `ReputationEventInsertForm`; add `.on_conflict_do_nothing()` to the
   insert chain.
2. Update the 2 existing JuryVote call sites in this file:
   - Line 583 (juror reliability emit inside the juror loop):
     append `ReputationEventSourceType::JuryVote, None` as the last
     two arguments.
   - Line 615 (reporter accuracy emit): append
     `ReputationEventSourceType::JuryVote, None` as the last two args.
3. **Source 3 vote-outcome insert** per §10.3: inside the existing
   `for (other_juror_id, juror_decision) in juror_decisions { ... }`
   loop at lines 577-597, after the existing JuryReliability emit at
   line 583, add (only when `juror_decision == winning_decision`):
   - Resolve `juror_pseudonym` via `actor_pseudonym_helper::get_or_create`.
   - Read `deltas.participation_juror_aligned` via `config::get_int`.
   - `i32::try_from(...)` per R1.
   - Call `emit_reputation_event(... ParticipationConsistency, +1
     delta, "vote_outcome_aligned", VoteOutcome,
     Some(format!("vote_outcome:{}:{}", data.case_id.0,
     juror_pseudonym)))?`.
   - Append `governance_log` row with
     `ENTRY_KIND_VOTE_OUTCOME_RECORDED` per §10.3.
4. **Source 4a evidence-cited insert** per §10.4: after the existing
   reporter emit at line 625 (still inside the `if path_kind ==
   SLDPathKind::Decided` block), add the heuristic block per §10.4
   verbatim. Reads `case_evidence` for the existence check; checks
   `winning_rationales` (already assembled at line 411) against the
   character-length threshold; emits `+1 reporting_accuracy` if both
   pre-conditions met.
5. Add `ENTRY_KIND_VOTE_OUTCOME_RECORDED` +
   `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED` + `case_evidence` schema +
   `case_evidence::table` import to the `use` block at lines 42-86.

**MIRROR:**
- `crates/api/api/src/governance/submit_jury_vote.rs:968-994` (current
  `emit_reputation_event` shape for the extended signature).
- `crates/api/api/src/governance/submit_jury_vote.rs:577-597` (juror
  loop where Source 3 lands).
- `crates/api/api/src/governance/submit_jury_vote.rs:600-625` (reporter
  emit AFTER which Source 4a lands).
- `crates/api/api/src/governance/admin_emergency_remove.rs:276`
  (`actor_pseudonym_helper::get_or_create(&mut conn.into(), ...)` shape).

**GOTCHA:**
- The existing juror loop iterates `juror_decisions: Vec<(PersonId,
  JuryDecision)>` (at lines 572-576) loaded fresh from `jury_vote`
  WITHIN the case_decided branch. Source 3's new emit lands INSIDE
  this loop's body, so the iteration count stays at most 5 (panel
  size) — no scaling concern. Per-iteration cost: 1
  `actor_pseudonym::get_or_create` + 1 INSERT + 1 governance_log
  append = ~3 round-trips per juror = ~15 round-trips per case at v0.
  Acceptable.
- The `cache: &mut ConfigCache` borrow inside `process_vote` is
  mutably borrowed through `Source 3`'s emit body — the same cache
  variable that's used at the existing reads (lines 547, 554). Pass
  by `&mut` reborrow; do NOT clone. Mirror the existing pattern.
- `winning_rationales: Vec<String>` (line 411) is already in scope
  when Source 4a's block runs. Length check uses `r.chars().count()`
  (UTF-8-correct character count), NOT `r.len()` (byte count). The
  PRD threshold (`evidence_cited_rationale_threshold_chars`) is
  measured in characters per its name.
- `case_evidence::uploader_id` is the field name (NOT `submitter_id`
  as the brief's clarify DQ -021 answer initially called it; the
  schema at `case_evidence.rs:20` is authoritative). The plan-time
  clarify answer used "submitter_id (or equivalent person_id FK)";
  `uploader_id` is the equivalent.
- The `governance_log::append` calls inside the new emits use `&mut
  (&mut *conn).into()` syntax for the conn borrow (mirror
  `submit_jury_vote.rs:687-699` shape).
- Both Source 3 and Source 4a emits are inside the existing
  `run_transaction` at line 139; no new transaction wrap needed.
- The two new ENTRY_KIND const imports go into the existing
  `governance_log::{ENTRY_KIND_APPEAL_DECIDED, ...}` import block at
  lines 45-47.

**VALIDATE (story-checkpoint feeds §16a Stories 3 + 5):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r3-task2-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-task2-check.log
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r3-task2-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-RT-r3-task2-clippy.log
# EXPECT: exit 0
```

**Post-validate:** writes `kind: "validate-pending-laptop"` DQ entry.
Required fields: `commands` (the 2 VALIDATE commands above), `branch:
"phase-v1-RT-r3"`, `phase_task: 2`. Pushes worker branch.

### Task 3 [P]: flag-bad-faith admin endpoint (DTOs + handler + route wire)

**ACTION:** Add `FlagBadFaithEmergencyReport` +
`FlagBadFaithEmergencyReportResponse` DTOs in `api_common`; extend
`admin_emergency_remove.rs` with `flag_bad_faith_emergency_report`
handler + inner `process_flag_bad_faith` + file-private
`emit_reputation_event_local`; wire the route in
`crates/api/routes/src/lib.rs` under `/admin/emergency-remove/flag-bad-faith`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api_common/src/governance.rs               # new request/response DTOs
  - crates/api/api/src/governance/admin_emergency_remove.rs   # new pub handler + inner fn + private emit helper
  - crates/api/routes/src/lib.rs                          # import + scope("/emergency-remove") wire
requires: []
```

**IMPLEMENT (file 1 of 3):** in
`crates/api/api_common/src/governance.rs`, append the two new DTOs per
§10.7 verbatim. Place them in the file alongside `AdminCloseCase` /
`AdminCloseCaseResponse` (lines 191-203 of the existing file is the
nearest sibling).

**IMPLEMENT (file 2 of 3):** in
`crates/api/api/src/governance/admin_emergency_remove.rs`, append the
new handler + inner fn + file-private emit helper per §10.5 verbatim:
- `pub async fn flag_bad_faith_emergency_report` (outer; `is_admin`
  + `run_transaction` wrap).
- `async fn process_flag_bad_faith` (inner; status assertion + emits).
- `async fn emit_reputation_event_local` (file-private helper;
  mirrors `submit_jury_vote.rs:968-994` shape verbatim including the
  `.on_conflict_do_nothing()` from §10.2).

Add the new imports to the existing `use` block at lines 17-40:
- `governance::governance_log::ENTRY_KIND_EVIDENCE_QUALITY_RECORDED`
- `lemmy_api_utils::utils::is_admin`
- `lemmy_db_views_local_user::LocalUserView`
- `lemmy_api_common::governance::{FlagBadFaithEmergencyReport,
  FlagBadFaithEmergencyReportResponse}`
- `actix_web::web::Json` + `activitypub_federation::config::Data`
- `lemmy_db_schema::source::governance::reputation_event::ReputationEventInsertForm`
- `lemmy_db_schema_file::{enums::{ReputationDimension,
  ReputationEventSourceType}, schema::reputation_event}`

**IMPLEMENT (file 3 of 3):** in `crates/api/routes/src/lib.rs`:
- Append `admin_emergency_remove::flag_bad_faith_emergency_report` to
  the existing `governance::{...}` import at lines 31-49 (alphabetical
  order — between `admin_dashboard_html::{...}` and
  `admin_reputation_stats`).
- Inside the existing `scope("/admin")` at line 493, alongside the
  existing `.service(scope("/config")...)`, `.service(scope("/rule-sets")...)`,
  `.service(scope("/audit")...)`, add a new
  `.service(scope("/emergency-remove").route("/flag-bad-faith",
  post().to(flag_bad_faith_emergency_report))),`.

**MIRROR:**
- `crates/api/api/src/governance/admin_emergency_remove.rs:75-101`
  (`emergency_remove_open_case` outer wrap).
- `crates/api/api/src/governance/admin_config.rs:1094-1142` (`is_admin`
  + `LocalUserView` capability-check on admin handler).
- `crates/api/api_common/src/governance.rs:191-203` (`AdminCloseCase`
  + `AdminCloseCaseResponse` shape).
- `crates/api/routes/src/lib.rs:503-518` (existing nested scopes
  under `/admin`).

**GOTCHA:**
- The handler's status-check returns `LemmyErrorType::Unknown(...)`
  with a specific message; the HTTP layer maps `LemmyErrorType::Unknown`
  to HTTP 500 by default. For the 400-arm assertion per brief §4
  ("400 for non-EmergencyRemove case"), the impl-task subagent should
  use a more specific `LemmyErrorType` variant if one exists that
  maps to 400. Search via
  `rg "pub enum LemmyErrorType" crates/utils/src/error.rs` for the
  available 400-mapping variants. If none cleanly map to 400, fall
  back to `LemmyErrorType::Unknown` with a descriptive message; the
  e2e test asserts on the response status_code which the brief's
  "400 for non-EmergencyRemove" calls for — if the actual mapping is
  500 with a structured payload, the e2e test asserts on that
  instead. Document the mapping decision in the Task 3 commit body.
- `is_admin` returns `LemmyResult<()>` which propagates as 403 via
  the HTTP layer's default `LemmyErrorType::NotAnAdmin` mapping (or
  equivalent — verify against existing admin handlers via `rg
  "is_admin\(" crates/api/api/src/governance/`).
- The dedupe_key for source 4b contains NO `reporter_pseudonym`
  segment (one bad-faith flag per case per ADR-013) — `dedupe_key =
  format!("evidence_bad_faith:{}", case_id.0)`. Repeat-admin flags
  on the same case are no-ops.
- The `actor_pseudonym` on the governance_log entry is the
  `admin_pseudonym` (NOT the reporter). The reporter's pseudonym
  appears only in the payload JSON. Per ADR-015 + brief §4 "real
  admin attribution".
- `ModerationCase::status` comparison: `case_row.status !=
  CaseStatus::EmergencyRemove` — mirror existing comparisons at
  `submit_jury_vote.rs:266-285`.
- The two new DTOs need the `#[cfg_attr(feature = "ts-rs", ...)]`
  attribute lines per the AdminCloseCase sibling at line 197.
  Without them, the `ts-rs` cargo feature's typescript-generation
  step will miss the new types and break a downstream CR finding.

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

**Post-validate:** writes `kind: "validate-pending-laptop"` DQ entry.
Required fields: `commands` (the 3 VALIDATE commands above), `branch:
"phase-v1-RT-r3"`, `phase_task: 3`. Pushes worker branch.

### Task 4: e2e tests (10 tests across 5 stories)

**ACTION:** Add `v1_rt_r3_fixtures` module + 10 new e2e tests to
`crates/server/tests/e2e.rs`. Tests assert the 5 stories' behaviours
end-to-end (HTTP + DB + governance_log) against the just-merged Task
1-3 implementations.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # v1_rt_r3_fixtures module + 10 new tests
requires:
  - task: 1
    reason: "tests call participation_cron::run_activity_batch + run_dormancy_batch directly (BREHON_DISABLE_PARTICIPATION_JOB-gated to prevent scheduler races); the module must exist on the phase branch before this validates."
  - task: 2
    reason: "tests assert vote-outcome + evidence-cited reputation_event rows after submit_jury_vote case_decided; the new emit logic must be present on the phase branch before this validates."
  - task: 3
    reason: "tests POST to /api/v4/governance/admin/emergency-remove/flag-bad-faith and assert 403/400/200 arms; the route + handler must be present on the phase branch before this validates."
```

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`, add ONE
new `v1_rt_r3_fixtures` module right after the most recent prior
fixtures module (`v1_rt_r2_fixtures` if it exists, else
`v1_sl_fixtures` or whichever is the latest `v1_*_fixtures`). Verify
the canonical sibling shape (Case A per
`feedback_lemmy_error_no_std_error.md`) by reading the chosen sibling
at its declaration line BEFORE writing the new module.

The fixtures module contains:

- 5 shared seeding helpers (`LemmyResult<T>` outer):
  - `seed_active_users_in_community(pool, community_id,
    person_count, comments_per_user) -> LemmyResult<Vec<PersonId>>`
  - `seed_dormant_users_in_community(pool, community_id,
    person_count) -> LemmyResult<Vec<PersonId>>` (seeds prior
    participation_consistency events but no recent comments)
  - `seed_decided_case_with_panel(context, panel_size,
    aligned_count) -> LemmyResult<(ModerationCaseId, Vec<PersonId>)>`
  - `seed_emergency_remove_case_with_reporter(context,
    reporter_id) -> LemmyResult<ModerationCaseId>`
  - `count_reputation_events_for(pool, person_id, dimension,
    source_event_type) -> LemmyResult<i64>`
- 10 test fns (`LemmyResult<()>` outer) per the story-to-test
  mapping in §4 Surface 4.

Test invocation pattern:
```rust
#[tokio::test(flavor = "multi_thread")]
async fn participation_activity_cron_emits_plus_one_per_active_user() -> LemmyResult<()> {
  // Set BREHON_DISABLE_PARTICIPATION_JOB=1 BEFORE build_test_context so
  // the scheduler tick never races the test's direct call.
  let (context, _guard) = build_test_context().await?;
  // Seed 3 communities, 5 active users each
  participation_cron::run_activity_batch(&context).await?;
  // Assert reputation_event count == 15 (3 communities * 5 users)
  // Assert each event has dedupe_key matching activity_cron:<community>:<person>:<iso_week>
  // Assert governance_log has 3 ENTRY_KIND_PARTICIPATION_CRON_TICK rows
  Ok(())
}
```

**MIRROR:**
- Pick the closest `v1_*_fixtures` sibling at e2e.rs (the planner
  cannot enumerate without reading e2e.rs; the impl-task subagent
  identifies the literal MIRROR file:line on its first read of e2e.rs
  per Case A discipline). Likely candidates: `v1_rt_r2_fixtures`,
  `v1_sl_d_fixtures`, `v1_jm_e_fixtures` — the impl-task picks one
  whose error-shape exactly matches (Case A: outer `LemmyResult<()>`,
  helper `LemmyResult<T>`, no `.map_err` bridges).
- `crates/server/tests/e2e.rs:9267-9400` (the existing
  `emergency_remove_open_case` test region — Task 4 reuses helpers
  for the `flag-bad-faith` 200-arm test seed).

**GOTCHA:**
- The fixtures module add is a SINGLE contiguous block (per
  `feedback_junior_worker_e2e_edit_hang.md` discipline; 1 edit, not
  scattered).
- The `BREHON_DISABLE_PARTICIPATION_JOB=1` env var must be set
  BEFORE `build_test_context().await?` so the scheduler tick never
  fires during the test (test directly calls
  `participation_cron::run_activity_batch`). Same pattern as
  `BREHON_DISABLE_SNAPSHOT_JOB` in existing e2e tests.
- The `iso_week` derivation uses `Utc::now()` at the moment of the
  cron run; the test's idempotency assertion relies on both
  invocations of `run_activity_batch` landing in the same wallclock
  ISO week. The test's wall-clock should be stable within
  milliseconds (no `tokio::time::sleep` between runs). If a future
  test crosses an ISO-week boundary mid-test, the assertion would
  flake; gate the test on `chrono::Utc::now().iso_week()` returning
  the same value at start + end of the test.
- The `flag-bad-faith` 400-arm test (Story 4 second test) seeds a
  case in `CaseStatus::Decided` (NOT `EmergencyRemove`) and asserts
  the HTTP response status code matches the Task 3 mapping decision.
- The flag-bad-faith 403-arm test seeds a non-admin user and asserts
  403; mirror existing `is_admin` denial tests in the file (search
  for `403|NotAnAdmin` in e2e.rs).

**VALIDATE (story-checkpoint feeds §16a Stories 1-5):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r3-task4-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-task4-check.log
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r3-task4-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-RT-r3-task4-clippy.log
# EXPECT: exit 0
```

```bash
# Compile-only check on e2e target (R7 — Task 4 adds new test fns)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-RT-r3-task4-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-task4-test-no-run.log
# EXPECT: exit 0
```

**Post-validate (e2e on laptop only, per brief §4):** Task 4's
e2e command is dispatched via a separate
`kind: "validate-pending-laptop-e2e"` DQ entry — impl-task pushes the
worker branch, writes the DQ blob with the 4 cargo commands array
(the 3 above + the e2e run below), and STOPS. The laptop advisor
session runs the e2e command and mutates the DQ entry. Do NOT run
e2e on the EliteDesk worker.

```bash
# Laptop-only e2e (mutated by advisor-laptop session, NOT EliteDesk worker)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-RT-r3-task4-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-RT-r3-task4-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-RT-r3-task4-e2e.log"
tail -50 .claude/PRPs/debug/v1-RT-r3-task4-e2e.log
# EXPECT: E2E_EXIT_0 marker present; all 10 new tests pass + all pre-existing tests still pass.
```

DQ blob shape per `decision-queue.md` "validate-pending-laptop-e2e":
- `commands`: the 4 commands above as a YAML array
- `branch`: `"phase-v1-RT-r3"`
- `phase_task`: 4

### Task 5: Retro

**Goal:** author retro at `.claude/PRPs/reports/v1-RT-r3-retro.md` per
`feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`.
One H2 per role (Advisor / Planning / Impl / BM) with signals +
lessons. Promote any new lessons to `.claude/lessons/feedback_*.md` in
the same retro commit (per `feedback_one_system_memory_in_repo.md`).
Per-task complexity score per
`feedback_retro_task_complexity_score.md`
(`<files>/<commits>/<runtime-min>/<max-log-silence-min>` per task,
aggregated in §5).

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-RT-r3-retro.md
modifies: []
requires:
  - task: 4
    reason: "Retro reads Task-1+2+3+4 impl commits + validate-pending-laptop + validate-pending-laptop-e2e DQ entries to populate per-role signals."
```

**No new code edits in retro task.** Retro is meta-work only.

---

## 14. Testing strategy

- **Unit (compile-time, per task):**
  `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full"`. Exit 0.
- **Lint (per task, uniform R6):**
  `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"`. Exit 0.
- **Test-target compile (R7 — Tasks 3 + 4):**
  `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run"`. Exit 0.
- **e2e execution (Task 4 only):**
  `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"`. All 10 new tests pass; pre-existing tests still pass.
- **No migration round-trip** — r3 has no migrations.
- **No unit-test mod additions** — r3's emit logic is end-to-end-only
  (cron behaviours need DB state; case-decision behaviours need a full
  case lifecycle). Unit tests of `participation_cron::run_activity_batch`
  in isolation would re-implement the DB layer; skip. e2e covers the
  surface adequately.

## 15. Validation commands (DoD)

> **Planner-side dry-run gate (per `feedback_plan_dod_dry_run_at_write.md`):**
> every command below MUST be dry-run by the advisor against current
> HEAD before plan approval. The advisor laptop session runs §3.4
> DoD smoke as gate 1's pre-condition.

### 15.1 Static analysis (per task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r3-<task>-check.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.2 Lint (per task — uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r3-<task>-clippy.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.3 Test-target compile (R7 — Tasks 3 + 4)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-RT-r3-<task>-test-no-run.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.4 e2e execution (Task 4 — laptop only)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-RT-r3-task4-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-RT-r3-task4-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-RT-r3-task4-e2e.log"
tail -50 .claude/PRPs/debug/v1-RT-r3-task4-e2e.log
# EXPECT: E2E_EXIT_0 marker present; 10 new tests + all pre-existing tests pass.
```

### 15.5 Cross-cutting verification

- [ ] R1: every `i32 ↔ i64` comparison uses `i64::from(...)` or
  `i32::try_from(...).map_err(...)?`, never `as` cast.
- [ ] R2: every governance_config read flows through `ConfigCache`
  via `config::get_int` / `config::get_bool`. No direct `fetch_value`.
- [ ] R3 (Watch 8): the `expires_at.is_some()` cliff guard +
  `original <= 0` penalty guard in r2's `compute_applied_delta` remain
  unchanged; r3 emits flow through r2 calculator transparently.
- [ ] R4: e2e tests use `?` propagation and `LemmyResult<()>` outer.
  No `.unwrap()`, no `.expect()`, no `dbg!`.
- [ ] R5: Task 0 enumerated all 14 probes explicitly.
- [ ] R6: all clippy invocations use
  `--workspace --features full --no-deps -- -D warnings` uniformly.
- [ ] R7: Tasks 3 + 4 ran `cargo test --no-run` (struct/re-export
  add via DTOs in api_common + e2e fn adds).
- [ ] R8: no `-p <crate> --features full` invocation in any §13 task
  body or §15 DoD command (only `--workspace --features full`).
- [ ] R9: both crons wrap per-community batches in `run_transaction`;
  `flag-bad-faith` handler wraps its emit + log in `run_transaction`.
- [ ] R10: every Source 1/2/3/4a/4b INSERT uses
  `.on_conflict_do_nothing()` to consume the
  `reputation_event_dedupe_key_partial_idx` partial unique violation.
- [ ] R11 (ADR-015): every governance_log payload field naming a
  person uses `*_pseudonym` (NEVER raw `PersonId`); cron entries use
  `actor_pseudonym = None`; `flag-bad-faith` uses `Some(admin_pseudonym)`.
- [ ] No new migration files in `migrations/`.
- [ ] No new ENTRY_KIND consts added in
  `crates/db_schema/src/source/governance/governance_log.rs`.
- [ ] No new `governance_config` keys in
  `crates/api/api/src/governance/config.rs` (only existing key reads).
- [ ] No edits to `feature.reputation_v1_decay_enabled` flag-gating
  semantics (emit unconditionally per brief §4 + PRD §11).
- [ ] No `crates/server/tests/e2e.rs` edits in Tasks 1-3 (only Task 4
  touches e2e per per-task complexity ceiling §5.2).
- [ ] Watchpoint W1 (§18 risk row 1): `emit_reputation_event`
  signature change is intra-file (no cross-crate cascade); 2 existing
  callers updated in same commit as the signature change.
- [ ] Watchpoint W2: minority jurors emit NO new
  ParticipationConsistency event after Source 3 wiring (test
  `vote_outcome_emits_nothing_for_minority_jurors` asserts this).
- [ ] Watchpoint W3: dedupe-key idempotency means repeat
  `run_activity_batch` within the same ISO week produces ZERO new
  reputation_event rows (Story 1's idempotency test asserts this).
- [ ] Watchpoint W4: `flag-bad-faith` non-admin returns 403; non-
  EmergencyRemove returns 400-class (Task 3 GOTCHA documents the
  exact mapping); admin + EmergencyRemove returns 200 with the
  reputation_event row written.
- [ ] PRD §6 acceptance: weekly active cron emits +1 per community
  per active user → Story 1 + Story 2.
- [ ] PRD §6 acceptance: vote-outcome emits +1 participation per
  majority-aligned juror, nothing for minority → Story 3.
- [ ] PRD §6 acceptance: evidence-cited heuristic emits +1
  reporting_accuracy when rationale ≥ threshold + reporter has ≥1
  case_evidence → Story 5 (optional).
- [ ] PRD §6 acceptance: admin flag-bad-faith emits -1
  reporting_accuracy for the reporter → Story 4 200-arm.

### 15.6 DoD per workflow (Shape G plans — not applicable)

Shape G is **SUSPENDED** per DQ #229. v1-RT-r3 runs under the
pre-Shape-G `validate-pending-laptop` + `validate-pending-laptop-e2e`
pathways. Each task's VALIDATE block lists the explicit cargo commands
run by the advisor laptop session per `advisor-orchestrator.md` §5.2.

---

## 16. Acceptance criteria

- [ ] All 6 tasks (Task 0 + Tasks 1-4 impl + Task 5 retro) completed
  in dependency order. Task 0 is verification-only (no commit). Tasks
  1-4 each produce one commit with subject `feat(rep-tuning): <description> (task N)`
  per brief §4 attribution convention. Task 5 produces one commit with
  subject `docs(retro): v1-RT-r3 retro`.
- [ ] §15.1 (cargo check `--workspace --features full`) exit 0 after
  every task.
- [ ] §15.2 (cargo clippy `--workspace --features full --no-deps -- -D warnings`)
  exit 0 after every task.
- [ ] §15.3 (cargo test `--no-run --test e2e`) exit 0 after Tasks 3 + 4.
- [ ] §15.4 (cargo test e2e workspace) exit 0 after Task 4 — all 10 new
  tests pass; no pre-existing-test regression.
- [ ] §15.5 (cross-cutting verification) — all 23 boxes ticked.
- [ ] §16a stories — all 5 stories `[done]` per `/brehon-verify`.
- [ ] No edits to files outside §11 list.
- [ ] Retro committed per §13 Task 5.
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo
  barrie-cork/lemmy` flag (per `gh-pr-fork-target.md`).

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: Activity cron emits +1 per active user per community (idempotent)

- **Composing tasks:** Task 1, Task 4.
- **Checkpoint command:**
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e -- v1_rt_r3_fixtures::participation_activity > .claude/PRPs/debug/v1-RT-r3-story1-checkpoint.log 2>&1"
  echo "exit: $?"
  tail -30 .claude/PRPs/debug/v1-RT-r3-story1-checkpoint.log
  ```
- **Expected output:** 2 tests passed (`participation_activity_cron_emits_plus_one_per_active_user`,
  `participation_activity_cron_idempotent_across_same_iso_week`).
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `crates/api/api/src/governance/participation_cron.rs` contains
    `pub async fn run_activity_batch(` declaration.
  - `crates/api/api/src/governance/mod.rs` contains `pub mod
    participation_cron;`.
  - `crates/routes/src/utils/scheduled_tasks.rs` contains
    `PARTICIPATION_CRON_RUNNING` static AtomicBool.
  - `crates/routes/src/utils/scheduled_tasks.rs` contains a new
    `scheduler.every(CTimeUnits::days(participation_interval_days)).run(...)`
    block that calls `participation_cron::run_activity_batch`.

### Story 2: Dormancy cron emits -2 per dormant user per community (idempotent)

- **Composing tasks:** Task 1, Task 4.
- **Checkpoint command:**
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e -- v1_rt_r3_fixtures::participation_dormancy > .claude/PRPs/debug/v1-RT-r3-story2-checkpoint.log 2>&1"
  echo "exit: $?"
  tail -30 .claude/PRPs/debug/v1-RT-r3-story2-checkpoint.log
  ```
- **Expected output:** 2 tests passed (`participation_dormancy_cron_emits_minus_two_per_dormant_user`,
  `participation_dormancy_cron_idempotent_across_same_iso_week`).
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/participation_cron.rs` contains
    `pub async fn run_dormancy_batch(` declaration.
  - The scheduler tick block in `scheduled_tasks.rs` calls BOTH
    `run_activity_batch` AND `run_dormancy_batch` sequentially under
    one guard.

### Story 3: Vote-outcome emits +1 for majority jurors, nothing for minority

- **Composing tasks:** Task 2, Task 4.
- **Checkpoint command:**
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e -- v1_rt_r3_fixtures::vote_outcome > .claude/PRPs/debug/v1-RT-r3-story3-checkpoint.log 2>&1"
  echo "exit: $?"
  tail -30 .claude/PRPs/debug/v1-RT-r3-story3-checkpoint.log
  ```
- **Expected output:** 2 tests passed (`vote_outcome_emits_plus_one_for_majority_aligned_jurors`,
  `vote_outcome_emits_nothing_for_minority_jurors`).
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/submit_jury_vote.rs` contains
    `ReputationDimension::ParticipationConsistency` reference inside
    the `case_decided` branch.
  - `crates/api/api/src/governance/submit_jury_vote.rs` references
    `ENTRY_KIND_VOTE_OUTCOME_RECORDED`.
  - `crates/api/api/src/governance/submit_jury_vote.rs::emit_reputation_event`
    signature includes `source_event_type` and `dedupe_key` parameters.

### Story 4: flag-bad-faith returns 403/400/200 in matching pre-condition arms

- **Composing tasks:** Task 3, Task 4.
- **Checkpoint command:**
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e -- v1_rt_r3_fixtures::flag_bad_faith > .claude/PRPs/debug/v1-RT-r3-story4-checkpoint.log 2>&1"
  echo "exit: $?"
  tail -30 .claude/PRPs/debug/v1-RT-r3-story4-checkpoint.log
  ```
- **Expected output:** 3 tests passed (`flag_bad_faith_returns_403_for_non_admin`,
  `flag_bad_faith_returns_400_for_non_emergency_remove_status`,
  `flag_bad_faith_admin_on_emergency_remove_case_emits_minus_one`).
- **Brief-Scope outputs to verify:**
  - `crates/api/api_common/src/governance.rs` contains
    `pub struct FlagBadFaithEmergencyReport` declaration.
  - `crates/api/api/src/governance/admin_emergency_remove.rs` contains
    `pub async fn flag_bad_faith_emergency_report(` declaration.
  - `crates/api/routes/src/lib.rs` contains
    `scope("/emergency-remove")` under the existing `/admin` scope
    with `.route("/flag-bad-faith", post().to(flag_bad_faith_emergency_report))`.

### Story 5: Evidence-cited heuristic emits +1 reporting_accuracy when rationale ≥ threshold AND reporter has ≥1 case_evidence row (OPTIONAL — defers to v2 if rationale empty in v0 fixtures)

- **Composing tasks:** Task 2, Task 4.
- **Checkpoint command:**
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e -- v1_rt_r3_fixtures::evidence_cited > .claude/PRPs/debug/v1-RT-r3-story5-checkpoint.log 2>&1"
  echo "exit: $?"
  tail -30 .claude/PRPs/debug/v1-RT-r3-story5-checkpoint.log
  ```
- **Expected output:** 1 test passed (`evidence_cited_heuristic_emits_plus_one_when_rationale_above_threshold`).
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/submit_jury_vote.rs` contains
    `participation.evidence_cited_rationale_threshold_chars` config
    read.
  - `crates/api/api/src/governance/submit_jury_vote.rs` contains the
    `case_evidence::uploader_id.eq(reporter_id)` existence check.
  - `crates/api/api/src/governance/submit_jury_vote.rs` references
    `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED` with `trigger:
    "rationale_cited"` payload.

**Story 5 fallback** (per brief §4 stop-and-ask #5): if Task 4's
pre-flight discovers that v0 e2e fixtures do NOT populate
`jury_vote.rationale` AT ALL (rare per clarify a3d0e9941441-022), file
a `kind: "blocker"` DQ and defer Source 4a to v2. Source 4b
(`flag-bad-faith`) ships regardless — it does not depend on rationale.

> **Verification mapping:** the advisor's `/brehon-verify` step
> iterates this section, runs each Story's Checkpoint against the
> worktree branch, and confirms each Brief-Scope output exists +
> matches its structural pattern. Phantoms (task complete but output
> absent or empty) trigger the catch-fire procedure in
> advisor-orchestrator.md §5.6.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all 14 probes confirmed).
- [ ] Task 1 committed: `feat(rep-tuning): participation cron module + scheduler tick (task 1)`.
- [ ] Task 2 committed: `feat(rep-tuning): vote-outcome + evidence-cited emit in submit_jury_vote (task 2)`.
- [ ] Task 3 committed: `feat(rep-tuning): flag-bad-faith admin endpoint (task 3)`.
- [ ] Task 4 committed: `feat(rep-tuning): r3 e2e tests across all 5 stories (task 4)`.
- [ ] §15 validation green at every gate.
- [ ] §16a 5 stories all `[done]`.
- [ ] Task 5 retro committed: `docs(retro): v1-RT-r3 — multi-source participation emitters`.
- [ ] PR opened by BM session against `governance-v0` via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-RT-r3`.
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-RT-r3-verify.md` shows all 5 stories ok.
- [ ] Post-merge phase branch retained for retro reads.

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| W1 — `emit_reputation_event` signature change (Task 2) breaks an external caller | VERY LOW | LOW | The helper is `async fn` (private, non-`pub`) in `submit_jury_vote.rs`; `rg "emit_reputation_event" crates/` confirms ZERO callers outside this file. Both intra-file v0 callers (lines 583, 615) updated in the same commit as the signature change. R7 cargo test --no-run runs after the change to catch any unexpected cross-file dependency. |
| W2 — Dedupe-key partial unique index conflict NOT consumed silently | LOW | MED | `.on_conflict_do_nothing()` (untargeted form) consumes ANY uniqueness violation including partial-index ones (confirmed per Diesel docs + r1 schema). If the violation surfaces as an Err instead of being silently consumed, Story 1's idempotency test will fail loudly (re-tick produces > 0 rows). |
| W3 — ISO-week derivation cross-boundary flakes idempotency test | LOW | LOW | Test captures `Utc::now().iso_week()` at start and asserts the same value at end. ISO-week boundary is once-per-week (Mon 00:00 UTC) so the cross-boundary risk is small. Acceptable. |
| W4 — `case_evidence.uploader_id` field name drift (clarify said "submitter_id (or equivalent)") | VERY LOW | LOW | Plan §11 + Task 2 GOTCHA + this risk row all cite `uploader_id` per the actual schema at `case_evidence.rs:20`. Impl-task subagent reads §11 + schema first per Mandatory Reading; field name drift would surface at cargo check time. |
| W5 — `jury_vote.rationale` is `Option<String>` per `jury_vote.rs:22` but v0 fixtures populate it inconsistently | LOW | LOW | Source 4a heuristic explicitly handles `Option` shape (uses `winning_rationales` which is already filtered to non-None strings at line 411-416). If a particular case has all jurors voting with `rationale: None`, `winning_rationales` is empty and Source 4a's `any(|r| r.chars().count() >= threshold)` is false — no emit, no panic. Story 5's test explicitly seeds a populated rationale to assert the positive path. |
| W6 — Scheduler tick fires during e2e tests despite BREHON_DISABLE env var | LOW | MED | The env-var check is the FIRST action in the closure body (mirror BREHON_DISABLE_SNAPSHOT_JOB at line 209). Tests set the var BEFORE `build_test_context()`. If the var is unset due to test ordering, the scheduler will fire `run_activity_batch` concurrently with the test's direct call — the per-community `run_transaction` + ON CONFLICT DO NOTHING means the worst case is "test asserts 15 events but finds the dedupe-key-correct count regardless". |
| W7 — `participation_cron` query plan O(N) on `comment` table without index | LOW | MED | The `comment` table already has indexes on `creator_id`, `community_id`, `published_at` (verified via existing v0 query shapes in `scheduled_tasks.rs::active_counts`). Group-by on (community_id, creator_id) over a window-filtered subset should use existing indexes. If a future operator's `comment` table grows to 10M+ rows AND the query plan goes sequential-scan, the cron tick exceeds the 15-min window and skips next tick via the RunningGuard — degraded but not broken. Operator-side mitigation is index tuning; documenting in retro is sufficient. |
| W8 — `flag-bad-faith` non-admin 403 vs non-EmergencyRemove 400 mapping ambiguity | LOW | LOW | Task 3 GOTCHA documents the impl-task decision: use the most specific `LemmyErrorType` variant available; if none cleanly map to 400, fall back to `Unknown` and the e2e test asserts on whatever status the mapping yields. CR review will surface the mapping if it's surprising. |
| W9 — Cycle-count meta-rule trips on `(error_class, file_basename)` for `submit_jury_vote.rs` | VERY LOW | MED | Task 2 is the only task editing `submit_jury_vote.rs`. The signature change is isolated (5 sites: 1 helper signature + 2 internal callers + 2 new callers). Cargo check + clippy gate the change before push. If a fix-impl cycle fires on `(E0277, submit_jury_vote.rs)`, the §G4 classifier reads the log slice and applies the allowlist recipe; cycle-count >=3 triggers hard refusal per `advisor-orchestrator.md` §5.3. |
| W10 — Concurrent v1-RT-r4 or v1-RT-r5 lane edits a shared file | VERY LOW | MED | Task 0 Probe 13 (gh pr list with file filter) confirms no overlap before phase start. r4 owns `sponsor_allowlist`; r5 owns rollup; neither touches r3's IMPLEMENT files. Per `.claude/rules/multi-lane-worktree.md` discipline, each lane runs in its own worktree. |
| W11 — Cron tick interval read from `job.participation_interval_days` returns NULL or invalid | VERY LOW | LOW | The `unwrap_or(7)` fallback handles missing or invalid values gracefully (mirror existing pattern at `scheduled_tasks.rs:309` for grace_check_interval_minutes). The 7-day default is itself the PRD-specified value. Operator misconfiguration degrades to default; never panics. |
| W12 — Task 3's soft per-task ceiling violation (3 crates) triggers planner-side rework | LOW | LOW | §5.2 documents the acceptance + rationale (small per-crate edits; bundling avoids 3 cohort barriers for one feature unit). If advisor or user flags this at gate 1, splitting Task 3 into Task 3a (api_common DTOs) + Task 3b (handler) + Task 3c (route wire) is mechanical — push the plan back, no impl rework. |
| W13 — Source 4a winning_rationales loaded at line 411 only on the Decided path; cases reaching the Pending path skip Source 4a | LOW (by design) | LOW | Source 4a is co-located with the existing reporter emit (lines 600-625) which is itself INSIDE `if path_kind == SLDPathKind::Decided`. So Source 4a fires only on the Decided path — matching v0 reporter-emit semantics. Cases that reach SponsorLiabilityPending have a future grace-window resolution that doesn't currently emit Source 4a; that's PRD-correct per §5.3 (the heuristic measures "what was cited in the decision rationale", which only exists on Decided cases). Documented as expected behaviour, not a gap. |

---

## 19. Notes

- **Source 4a vs Source 4b emit-site asymmetry**: Source 4a (positive,
  rationale-cited) lives INSIDE `submit_jury_vote::process_vote` and
  fires automatically when the heuristic matches at case-decision time.
  Source 4b (negative, admin-flagged bad-faith) is a separate admin
  endpoint requiring explicit human action per ADR-013 (admin-driven
  posture). The two arms write to the same governance_log entry kind
  (`ENTRY_KIND_EVIDENCE_QUALITY_RECORDED`) but distinguish via the
  `trigger` payload field (`"rationale_cited"` vs
  `"admin_flagged_bad_faith"`). The reputation_event rows distinguish
  via the `dedupe_key` prefix (`evidence_cited:` vs
  `evidence_bad_faith:`).
- **`PARTICIPATION_CRON_RUNNING` guard scope**: one guard covers
  BOTH `run_activity_batch` and `run_dormancy_batch` running
  sequentially per tick (per brief §2.1 "in same scheduled_tasks.rs
  tick, registered after the activity cron block"). If activity hangs
  for 7 days, the next tick skips both — acceptable since dormancy
  depends on activity being run regularly (prior participation events
  are the dormancy precondition; without activity emits, dormancy has
  no anchor).
- **Cohort-3 cohesion**: Task 3 bundles 3 files across 3 crates for
  one feature unit (the admin endpoint). The brief's "stop if > 4 §13
  tasks" constraint takes precedence over the Sonnet per-task ceiling
  (≤2 crates implicit); the soft violation is documented in §5.2 with
  rationale. v1-AD-c split-by-crate is the prior convention; r3
  deliberately bundles for compactness given the small per-crate edit
  size.
- **Test fixtures discipline**: the e2e module add is a SINGLE
  contiguous block per `feedback_junior_worker_e2e_edit_hang.md`. The
  impl-task subagent should write the entire `v1_rt_r3_fixtures` mod
  + all 10 tests in one Edit call (or one Write call if appending at
  end-of-file is cleaner). Scattered edits to e2e.rs are the
  known-bad pattern that triggers the worker-hang risk.
- **No emit changes on the appeal-vote path**: the
  `process_appeal_vote` branch at `submit_jury_vote.rs:727` is
  out-of-scope for r3 per the PRD §5.3 "vote-outcome" definition (the
  appeal panel's vote IS itself a vote outcome, but the brief and PRD
  scope Source 3 to the original `process_vote::case_decided` branch
  only). Adding appeal-vote-outcome emits would require a separate
  ADR amendment + a v1-RT-r? sub-phase.
- **DQ entries pre-resolved**: a3d0e9941441-018 through -024 all
  resolved at commit `e3f919d70`. No clarify-pending DQs remain on
  this brief.
- **r3's emit volume estimate**: at v0 community sizes (~100 active
  users per community, ~10 communities = 1000 active-user-community
  pairs per week), Source 1 emits ~1000 rows/week + ~10 governance_log
  entries/week. Source 2 emits ~50 rows/week (dormant tail). Source 3
  emits ~5 rows per Decided case (panel size). Source 4a emits ≤1
  per case. Source 4b emits ≤1 per admin flag (rare, admin-driven).
  Total: ~1100 reputation_event rows/week + ~20 governance_log
  rows/week — well within the 15-min snapshot tick's budget.
- **Forward compat with v2 rollup cron**: r5's rollup cron will
  consume the per-community `participation_consistency` snapshot
  rows that r3's emits feed via r2's calculator. r3's rows include
  the `dedupe_key` for backfill / replay scenarios; r5's rollup
  query doesn't care about dedupe_key (it reads the final calculated
  values).

---

## 20. Confidence score

- **Plan correctness:** 8/10 — the multi-source emit shape is
  well-specified by PRD §5.3; the dedupe-key idempotency mechanism is
  the central design choice and the partial unique index pre-landed
  by r1 makes it mechanically correct. One point reserved for the
  Diesel `group_by` + `HAVING` shape for the activity cron — if
  Diesel's ergonomics on grouped subqueries are awkward, the
  impl-task subagent falls back to raw `sql_query` (mirror
  `scheduled_tasks.rs:510-526`); both shapes are valid but the
  fallback adds non-trivial review surface for CR. One point
  reserved for the Task 3 soft per-task ceiling violation — a CR or
  advisor-gate-1 push-back could request a split, costing a plan
  re-author cycle.
- **Cargo budget:** 9/10 — `cargo check --workspace --features full`
  is the dominant cost (~6 GB peak), within the 10 GB EliteDesk cap.
  r3 has no migrations and 1 e2e edit; ~6 GB total peak per task
  (serial-within-cohort + cohort-of-3 cleanly serial vs Task 4).
  Laptop e2e adds ~26 min wall-clock per cycle but zero billed cost.
- **Test coverage:** 8/10 — 10 e2e tests cover 5 stories with
  positive + negative arms (idempotency, no-penalty-for-dissent,
  403/400/200 status arms, evidence-cited positive). One point
  reserved for the absent unit-test mod additions — the cron
  query-shape correctness is asserted at the e2e level only; a
  query-shape regression that returns the right ROWS but the wrong
  COLUMNS could pass e2e if the assertions miss. CR review will
  surface this. One point reserved for the Source 5 (evidence-cited)
  fallback path — if v0 fixtures don't populate rationale,
  Story 5 defers to v2 and r3 ships with 4 stories `[done]` + 1
  story `[deferred]`; the partial coverage is an acceptance edge
  case.
