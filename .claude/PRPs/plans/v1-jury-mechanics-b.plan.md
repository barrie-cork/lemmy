# Plan: v1-JM-b — admin_assign_jury cascade + diversity constraints + severity/status snapshot

## Table of contents

| § | Heading |
|---|---|
| 1 | Summary |
| 2 | Source |
| 3 | Problem statement |
| 4 | Solution statement |
| 5 | Metadata |
| 6 | Relationship to other v1-JM sub-phases |
| 7 | Preflight guardrails inherited from prior phases |
| 8 | Flow design |
| 9 | Mandatory reading |
| 10 | Patterns to mirror |
| 11 | Files to change |
| 12 | NOT building in v1-JM-b |
| 13 | Step-by-step tasks |
| 14 | Testing strategy |
| 15 | Validation commands (DoD) |
| 16 | Acceptance criteria |
| 17 | Completion checklist |
| 18 | Risks and mitigations |
| 19 | Notes |
| 20 | Sub-phase stubs (v1-JM-c/d/e TOC) |

---

## 1. Summary

v1-JM-b is the **selection-algorithm sub-phase** of v1 jury-mechanics. It consumes the schema + config + entry-kind substrate that v1-JM-a shipped, and it turns the v0 `admin_assign_jury` handler into a severity-aware, status-aware, diversity-constrained selector with an audited relaxation cascade. No appeals work, no `submit_jury_vote` edits, no new routes, no background jobs.

Concretely v1-JM-b delivers:

1. Two new config accessor helpers `config::get_int_cascade` / `config::get_float_cascade` per PRD §3.5 — a reader-side cascade `<status>.<severity> → <severity> → bare-key → const` that the existing `config::get_int` deliberately does not perform on dotted keys.
2. A new helper `compute_status_tier(case, conn)` that reads `reputation_event` (for `founder_seed`) + `person.membership_state` to classify the target person into `Founder / Regular / Probation` per PRD §4.2, matching the existing `is_founder` pattern inside `sponsor_liability.rs:219-228`.
3. `admin_assign_jury::process_assignment` rewrite — reads `severity_tier` (from the case, JM-a-backfilled), computes `status_tier` (lazily at assign time, not at case-open), cascades panel-size / quorum-fraction / threshold-fraction via (1), resolves integer `quorum_snapshot` / `threshold_count_snapshot` via `ceil(panel_size * fraction)`, snapshots all three onto `moderation_case`, emits a `severity_tier_frozen` governance_log entry.
4. `select_eligible_jurors` rewrite — a 3-phase algorithm (pool-build cooldown filter → sample-level sponsor-cluster re-roll → soft geographic scoring) with an R1→R2→R3 relaxation cascade, each relaxation writing both a `jury_constraint_violation_log` row AND a `jury_constraint_relaxed` governance_log entry per PRD §5.3.
5. A new helper `panel_has_sponsor_majority_cluster` in `jury_common.rs` — the panel-level generalisation of the existing v0 `shares_active_sponsor` one-hop check.
6. A new helper `recent_juror_cooldown_subquery` in `admin_assign_jury.rs` (or `jury_common.rs`) — SQL fragment excluding jurors whose `responded_at > now() - INTERVAL 'N day'`.
7. Per-juror JSONB `selected_under_constraints` write on `jury_assignment` insert — listing which constraints were applied at pick time (constraint names only, no person_ids per ADR-015 / v1-JM-a cr-9 discipline).
8. `admin_emergency_remove` edit — set `severity_tier = Severe` on case-open so the post-facto jury gets a Severe-tier panel per PRD §3.2 edge case.
9. Two test-surface extensions in `tests/e2e.rs`: a cascade round-trip test proving `get_int_cascade` resolves `jury.panel_size.founder.severe → jury.panel_size.<severity> → jury.panel_size → const`; and a constraint-relaxation assertion test proving that an under-sized pool triggers R1 → emits `jury_constraint_relaxed` + writes `jury_constraint_violation_log` row + still seats a panel.

No edits to `submit_jury_vote.rs`, `request_appeal.rs`, `accept_jury_assignment.rs`, `decline_jury_assignment.rs`. The panel `role` column stays at its `'Original'` DEFAULT (InsertForm omits it) — v1-JM-d's appeal-panel path is the only writer that will set it to `'Appeal'`.

This sub-phase's single-slice value: the admin-assign flow becomes **severity/status-proportional and diversity-aware** before any vote-tally or appeal code changes. A Severe case against a Founder now gets a 9-juror panel with sponsor-cluster + cooldown + geographic constraints applied; the v0 baseline Minor/Regular case still gets a 5-juror panel. v1-JM-c then builds on the quorum/threshold snapshots that JM-b just wrote.

---

## 2. Source

- [../prds/v1-jury-mechanics.prd.md](../prds/v1-jury-mechanics.prd.md) §3 (severity tiers), §4 (status awareness), §5 (diversity constraints), §9.2 (`admin_assign_jury` changes), §9.6 (`admin_emergency_remove` severity default), §10 (defaults matrix, JM-a-seeded)
- [../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) ADR-007, ADR-010 (hard rule: no retroactive invalidation — snapshots immutable), ADR-013 (EmergencyRemove tier), ADR-015 (pseudonymised audit), OQ-026 (dotted cascade pattern)
- [../../../docs/brehon-law-inspired-network/01-vision-and-principles.md §5.6](../../../docs/brehon-law-inspired-network/01-vision-and-principles.md) — canonical default matrix (Minor 5/simple, Moderate 5/60%, Severe 7/75%) that PRD §10 materialises
- [../../../docs/brehon-law-inspired-network/04-data-model-and-api.md §6](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) — handler responsibility boundaries for governance endpoints
- [v1-JM-a plan §12 "NOT building"](./phase-v1-JM-a.plan.md) — explicitly defers admin_assign_jury edits, cascade helpers, select_eligible_jurors rewrites, admin_emergency_remove severity tier to JM-b; this plan picks those up
- Precedent: [v1-admin-dashboard-a.plan.md](./v1-admin-dashboard-a.plan.md) `EXPECTED_SEED_COUNT_V1_*` pattern — JM-b does NOT add a new `EXPECTED_SEED_COUNT_V1_JM_B` because it seeds zero new config keys; all 27 `jury.*`/`appeal.*` keys are already in the JM-a seed migration
- [GH issue #29](https://github.com/barrie-cork/lemmy/issues/29) — config-flip determinism note; JM-b audit-log shape (written to `jury_constraint_violation_log` + `governance_log`) makes test assertions checkable on stable derived properties without making `ORDER BY random()` deterministic

---

## 3. Problem statement

Post-v1-JM-a-merge, the fork's moderation substrate has:

- Three new Postgres enums (`severity_tier`, `case_status_tier`, `jury_assignment_role`) and the matching Rust enums + `DbValueStyle = "verbatim"` derives.
- `moderation_case` with six new columns (`severity_tier NOT NULL DEFAULT 'Minor'`, `status_tier NOT NULL DEFAULT 'Regular'`, `panel_size_snapshot INT4 NULL`, `quorum_snapshot INT4 NULL`, `threshold_count_snapshot INT4 NULL`, `appeal_window_expires_at TIMESTAMPTZ NULL`).
- `jury_assignment` with two new columns (`selected_under_constraints JSONB NULL`, `role jury_assignment_role NOT NULL DEFAULT 'Original'`).
- New table `jury_constraint_violation_log` with `reason_code` (a 4-variant PascalCase Postgres enum `JuryConstraintRelaxationReason`: `SmallPool | ClusterPressure | ClusterPressureExhausted | AdminOverride`) + optional `relaxation_metadata JSONB`.
- 27 new seeded config keys spanning `jury.panel_size.<status>.<severity>`, `jury.quorum_fraction.<severity>`, `jury.threshold_fraction.<severity>`, `jury.constraints.*`, `jury.max_concurrent_assignments_per_juror_total`, `appeal.*` — all tied into `SEEDED_KEYS_WITH_CONSTS` + `CONFIG_KEY_METADATA` + `const_default_{int,float,bool}` arms.
- Six new `ENTRY_KIND_*` consts (`JURY_CONSTRAINT_RELAXED`, `APPEAL_PANEL_ASSEMBLED`, `APPEAL_DECIDED`, `APPEAL_REJECTED`, `APPEAL_WINDOW_EXPIRED`, `SEVERITY_TIER_FROZEN`) defined in `crates/db_schema/src/source/governance/governance_log.rs` and re-exported via the `crates/api/api/src/governance/governance_log.rs` shim.

**None of that substrate is read or written by any handler yet.** `admin_assign_jury.rs:120` still reads bare `jury.panel_size` (returning 5 via the v0 const fallback); `select_eligible_jurors` has no concept of constraints, constraint relaxation, or severity-aware panel sizing; `admin_emergency_remove.rs:153` still sets `CaseSeverity::default()` (`Medium`) and leaves `severity_tier` at the column default `'Minor'` even though the ADR-013 emergency path is semantically Severe. The 9-cell panel-size matrix, the three-phase diversity algorithm, and the R1/R2/R3 relaxation cascade exist as *rows in a config table* and *columns in a case table* — not as code.

v1-JM-b is the first sub-phase where those substrates translate into runtime behaviour. It is intentionally scoped to the **pick-time** path: the admin-assign handler. Vote-tally (`submit_jury_vote`) reads the quorum/threshold snapshots JM-b writes, but JM-b does not touch the vote path — that's v1-JM-c. Appeal panel assembly reuses `select_eligible_jurors` with `exclude_person_ids = original_jurors` + `role = Appeal` — but that's v1-JM-d. JM-b is **just the pick path, end-to-end**.

The scope boundary is load-bearing: if JM-b also edits `submit_jury_vote` or `request_appeal`, the ralph loop exceeds the ~10-12 task phase-splitting threshold that both Phase 5 retros and the v1-AD retro identified as the compound-context zone where impl quality degrades.

---

## 4. Solution statement

Keep the v0 `admin_assign_jury` handler **structurally recognisable** — the outer shape (transaction, status guard, eligible-juror selection, bulk-insert, status flip, per-juror log entries, panel_assembled marker) stays intact. Three surgical extensions:

### 4.1 Architecturally load-bearing decisions locked in this sub-phase

- **`get_int_cascade` / `get_float_cascade` are new PUBLIC functions in `config.rs`, NOT replacements for `get_int`/`get_float`.** Every existing v0/AD call site that reads a bare single-segment key (`jury.panel_size`, `thresholds.jury_reliability`) keeps calling `get_int`. The cascade functions are for dotted-namespace keys where the caller wants the `<status>.<severity> → <severity> → bare-key → const` walk. Renaming existing callers is out of scope for JM-b. Confirmed from Explore agent #2 §1: the existing `get_int` cascades over *scope* (`Community → Instance`) but does NOT cascade over *key*, and v1-JM-a §9.2 explicitly said "Do NOT call `get_int("jury.panel_size.<status>.<severity>")` directly — that skips the cascade and returns Err on partial seeding."
- **`status_tier` is computed LAZILY at admin_assign_jury time, not eagerly at case-open.** Per PRD §3.2 the severity tier is reporter-suggested + admin-overridable at case-open time, BUT the status tier (Founder/Regular/Probation) depends on the target person's current reputation state, which can change between report and assign. Computing at admin_assign_jury time is also natural because it's the same transaction where `status_tier` gets snapshotted onto the case row — single write point, no case-open hook required. This matches the PRD §3.2 "frozen on transition to JurySelection" marker. The JM-a backfill's `NOT NULL DEFAULT 'Regular'` covers any case that was created between JM-a merge and JM-b merge (a narrow window but non-zero).
- **Severity tier stays at its JM-a-backfilled default `'Minor'` for reports created post-JM-a-pre-JM-b.** No case-open hook is added in JM-b. The full reporter-suggested + admin-override severity logic from PRD §3.2 / §12 ships separately — it's a `create_report` / DTO concern, not an admin-assign concern, and bundling it here violates the 10-12-task rule. `admin_emergency_remove` is the ONE exception: it explicitly sets `severity_tier = Severe` at case-open because ADR-013's invocation is already the severe action. General post-JM-b case-open severity inference is tracked as an OQ in §7.
- **JM-b re-uses the v1-JM-a-seeded 27 config keys.** No new keys are seeded. `SEEDED_KEYS_WITH_CONSTS.len()` does not change. `EXPECTED_SEED_COUNT_V1_JM` stays at 27. No migration is added.
- **Panel-level sponsor-cluster query is a single extended SQL block, not five separate `shares_active_sponsor` calls.** Calling `shares_active_sponsor(a, b)` for every panel pair is O(panel_size²) round trips; for a 9-juror Severe-Founder panel that's 36 round trips per sample, 180 per R2 re-roll stack — unacceptable. The helper `panel_has_sponsor_majority_cluster(conn, person_ids)` runs **one** `sql_query` that joins `surety` on itself, groups by `sponsor_id`, counts distinct `sponsored_id` IN the panel, and returns the max group size. If `max > panel_size / 2` → violation.
- **Geographic diversity is a SOFT scoring input, not a SQL filter.** PRD §5.3 Phase 3 is explicit: "bias sampling towards higher-scoring panels in Phase 2's re-roll loop (soft — a panel with low diversity still returns; it just lost the coin toss against a higher-diversity alternative during re-roll)." Implement as: inside the Phase-2 re-roll loop, compute a diversity score for each candidate panel as `unique_declared_community_count / panel_size`; when two samples both pass the sponsor-cluster constraint, pick the higher-scoring one. When `geographic_diversity_preferred = false`, skip the scoring step entirely (return the first sponsor-cluster-passing sample).
- **Relaxation audit writes BOTH a `jury_constraint_violation_log` row AND a `governance_log` entry of kind `jury_constraint_relaxed`.** Per PRD §8.3 + §12.2. The governance_log entry is the tamper-evident record; the violation_log table is the queryable index for the dashboard. Both writes land inside the same `run_transaction` as the panel seating — no intermediate state in which a relaxation is recorded in one but not the other.
- **`panel_assembled` payload extended with the panel-level constraint summary.** v0 `panel_assembled` payload is `{case_id, juror_count}`. JM-b extends to `{case_id, juror_count, severity_tier, status_tier, constraints_applied: {...}, relaxations: [...]}` — the `constraints_applied` field matches the JSONB written to each `jury_assignment.selected_under_constraints` row; the `relaxations` field lists any R1/R2/R3 cascade steps that fired for this pick. This stays within the v1-JM-a-blessed `panel_assembled` entry kind (no new kind needed for the panel summary).
- **`admin_emergency_remove` sets `severity_tier = Severe`.** ADR-013's emergency invocation is already a Severe-level action per PRD §3.2 edge case + PRD §9.6. This is a single additional field in the `ModerationCaseInsertForm` literal at `admin_emergency_remove.rs:143-157`. The resulting post-facto jury is then sized via the same cascade as any other Severe case (`jury.panel_size.regular.severe = 7` by default, since emergency-remove cases have `target_person_id = None` so `status_tier` stays Regular).
- **No cross-community juror cap reads.** The key `jury.max_concurrent_assignments_per_juror_total` was seeded by JM-a but no caller reads it yet. Adding the "NOT EXISTS (SELECT ... FROM jury_assignment WHERE person_id = p.id AND status IN (Accepted, Selected))" clause to the strict query is a natural JM-b scope addition, BUT v1-JM-a's integer default is seeded so the read path is trivial once the cross-community clause is wired. **Include this in JM-b**: it's one extra SQL subquery + one extra `config::get_int` read.

### 4.2 Rejected alternatives

- **Bundle `create_report` severity-inference into JM-b.** Rejected: it's a separate reporter-UX surface (which `reason_code` → tier mapping?) with its own OQ surface, and PRD §12.1 puts the reporter-tagged severity behind a per-community policy knob that would itself need a new seeded key. Keep JM-b focused on the pick-time path; case-open severity-inference is a JM-b/c/d OQ (see §7).
- **Write a single multi-column insert helper `insert_panel(eligible, constraints)`.** Rejected: the v0 shape has `insert_into(jury_assignment::table).values(&forms).execute(conn)` as the bulk insert, and JM-b's only change is that every form in the `Vec<JuryAssignmentInsertForm>` now carries the `selected_under_constraints` JSONB. InsertForm already derives `Default` so the `role` field is omitted (DEFAULT 'Original' applies). Abstracting this into a helper is premature.
- **Compute `status_tier` per-juror (each juror gets their own status tier).** Rejected: `status_tier` is a property of the **case** (specifically the target person whose content is on trial), not of each juror. PRD §4.1-§4.2 explicitly defines it that way. Every juror on a given case shares the same status_tier snapshot.
- **Use the existing `get_int` with an explicit `DEFAULT_JURY_PANEL_SIZE_REGULAR_MINOR` fallback at every call site.** Rejected: duplicates the cascade logic at every call site, every update to the matrix churns every caller. A dedicated `get_int_cascade` helper centralises the cascade logic.
- **Make the relaxation cascade synchronous across multiple Postgres round trips without a single transaction.** Rejected: the v0 `admin_assign_jury` already runs every write inside `conn.run_transaction(...)` (lines 73-82). R1/R2/R3 relaxation writes join the same transaction — if panel assembly ultimately fails (R3 exhausted + `fallback_on_small_pool = false`), the rollback reverts both the violation_log rows and the governance_log entries. This is the correct atomicity for audit integrity.

---

## 5. Metadata

| Field | Value |
|---|---|
| Type | `HANDLER + CROSS_CUTTING` (admin_assign_jury + select_eligible_jurors + admin_emergency_remove + new helpers) |
| Complexity | MEDIUM–HIGH (new selection algorithm; 3-phase logic with relaxation cascade; new cascade helper) |
| Crates Affected | `lemmy_api` (config.rs, admin_assign_jury.rs, admin_emergency_remove.rs, jury_common.rs) |
| v1 Step | v1-JM sub-phase B (per v1-JM-a plan §6 split) |
| Dependencies | v1-JM-a merged to `governance-v0` (schema + 27 seeded jury/appeal keys + 6 new entry kinds) |
| Estimated Tasks | 11 (including Task 0 pre-flight and Task 10 retro + PR) |
| Sub-phase target branch | `phase-v1-JM-b` branched from `governance-v0` post-JM-a-merge |
| PR target | `governance-v0` per `.claude/rules/phase-branch.md` |
| Closes | None directly (issue #29 note only — see §2) |
| Opens | 1 new OQ (see §7) about post-JM-b case-open severity inference |

---

## 6. Relationship to other v1-JM sub-phases

| Sub-phase | Depends on | Scope |
|---|---|---|
| v1-JM-a (shipped) | v1-AD-a merged | Schema + enums + 27 seeded keys + 6 entry-kind consts + backfill |
| **v1-JM-b (this plan)** | v1-JM-a merged | admin_assign_jury cascade + diversity + severity_tier_frozen + admin_emergency_remove severity=Severe |
| v1-JM-c | v1-JM-b merged | submit_jury_vote 9-step handler: snapshot reads, threshold check, deadlock→AdminReview, appeal_window_expires_at write, sponsor-liability branch compute-phase integration |
| v1-JM-d | v1-JM-c merged | request_appeal (bounded window, reporter-rights, auto re-jury) + admin_trigger_appeal_rejury + appeal-window-expired background job |
| v1-JM-e | v1-JM-d merged | Capstone test: `v0_case_completes_under_v0_rules_after_v1_config_flip` per PRD §11 + cross-sub-phase integration assertions |

v1-JM-b **must merge** before JM-c starts. Concurrent work: v1-rep-tuning-r2+ is independent (different crates and queries), v1-SL-a is independent (sponsor-liability schema + scheduler module, no admin_assign_jury overlap). No DQ entry from v1-JM-b blocks any other v1 PRD.

v1-JM-b must **NOT land** while a v1-AD PR is open — both phases' PRs touch `config.rs` in overlapping regions (const declarations + `const_default_*` arms + `SEEDED_KEYS_WITH_CONSTS` in v1-AD; cascade helpers next to them in JM-b). Enforce with a pre-phase git-remote check in Task 0.

---

## 7. Preflight guardrails inherited from prior phases

Per `.claude/rules/pre-phase-harness-audit.md`, the Task-0 harness audit is mandatory. Four DQs from the v1-AD-d retrospective are already incorporated in `/prp-core:prp-implement`:

- **DQ #42 (task-resume safety)** — resume-detection matches commit subjects against plan §13 COMMIT MESSAGE lines
- **DQ #43 (HTTP status code audit)** — not directly applicable; JM-b adds no new HTTP routes (but existing `admin_assign_jury` responses stay within 200/404 per v0)
- **DQ #44 (Docker daemon preflight)** — mandatory probe in Task 0
- **DQ #46 (v1/limitation GH issue capture)** — JM-b has one identifiable candidate: the "general case-open severity-inference" deferral (see §7.1 below + Task 10 retro)

### 7.1 New OQ to open (before Task 1)

One OQ is opened by this plan per v1-JM-a-precedent (Task 7 of that plan). The OQ is a blocking-for-future-sub-phase, not for JM-b execution. Create it in a single commit in Task 1.

| OQ | Question | Lean | Blocks |
|---|---|---|---|
| OQ-V1-JM-07 (new) | Post-JM-b general case-open severity inference — does `create_report` DTO add a `severity_tier_suggested` field, or does the `reason_code → severity_tier` mapping live in a per-community `governance_config` text/json key, or is it hardcoded? | **Lean**: hardcoded `reason_code → severity_tier` table in a new `severity_inference.rs` module, callable from `create_report` at case-open. Per-community overrides via a new `report.reason_code_severity_map` text key are v1.5. Hardcode keeps JM-b-following PRs small. | v1.5 general-severity-inference sub-phase (not JM-c/d/e — they operate on already-snapshotted severity) |

OQ-V1-JM-07 does not block JM-b because JM-b only **reads** `moderation_case.severity_tier`, and the column is NOT NULL with `DEFAULT 'Minor'` — so every non-emergency-remove case pre-JM-b-merge and post-JM-b-merge starts as Minor. Once the general severity-inference ships (v1.5), existing Minor cases stay Minor (ADR-010) and new cases pick up the inference. JM-b's correctness is unaffected.

---

## 8. Flow design

### 8.1 Before state (post v1-JM-a merge on `governance-v0`)

```text
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              BEFORE STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║   admin_assign_jury.rs (v0 post-5b task 57, JM-a unmodified):                 ║
║     process_assignment reads bare "jury.panel_size" via config::get_int       ║
║     process_assignment reads bare "jury.max_concurrent_assignments"           ║
║     process_assignment reads bare "jury.fallback_on_small_pool"               ║
║     select_eligible_jurors: reputation gate + concurrent-cap + small-pool      ║
║       fallback, no diversity constraints, no cooldown                         ║
║     JuryAssignmentInsertForm { case_id, person_id, status } — 3 fields        ║
║     No severity_tier read. No status_tier compute. No snapshot writes.        ║
║     No severity_tier_frozen log entry. No jury_constraint_relaxed log.        ║
║                                                                               ║
║   admin_emergency_remove.rs:153:                                              ║
║     severity: CaseSeverity::default() (Medium)                                ║
║     severity_tier: backfill default 'Minor' (JM-a)                            ║
║                                                                               ║
║   config.rs (post-AD-a + post-JM-a):                                          ║
║     get_int / get_float / get_bool / get_text (scope cascade)                 ║
║     27 jury.*/appeal.* keys seeded                                            ║
║     No get_int_cascade / get_float_cascade (deferred by JM-a §12)             ║
║                                                                               ║
║   jury_common.rs:                                                             ║
║     shares_active_sponsor(conn, a, b) -> one-hop pairwise check (2-person)    ║
║     No panel-level cluster check                                              ║
║                                                                               ║
║   moderation_case snapshot columns:                                           ║
║     panel_size_snapshot NULL for every NEW case (backfill only populated      ║
║       pre-v1 cases via JM-a up.sql)                                           ║
║     appeal_window_expires_at NULL for every case in Open/ThresholdMet         ║
║                                                                               ║
║   jury_assignment.selected_under_constraints NULL for every row               ║
║   jury_constraint_violation_log table exists but has zero rows ever written   ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### 8.2 After state (end of v1-JM-b)

```text
╔═══════════════════════════════════════════════════════════════════════════════╗
║                               AFTER STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║   admin_assign_jury.rs (v1-JM-b):                                             ║
║     process_assignment reads case.severity_tier + computes status_tier        ║
║     process_assignment uses get_int_cascade for panel_size                    ║
║     process_assignment uses get_float_cascade for quorum_fraction +           ║
║       threshold_fraction, resolves ceil() to integer counts                   ║
║     process_assignment snapshots panel_size_snapshot + quorum_snapshot +      ║
║       threshold_count_snapshot + (if needed) status_tier onto moderation_case ║
║     select_eligible_jurors runs 3-phase algorithm (cooldown filter +          ║
║       sponsor-cluster re-roll + geographic soft score) per PRD §5.3           ║
║     R1/R2/R3 relaxation cascade emits jury_constraint_relaxed log + writes    ║
║       jury_constraint_violation_log row                                       ║
║     Per-juror JuryAssignmentInsertForm carries selected_under_constraints     ║
║       JSONB (constraint names only per ADR-015)                               ║
║     severity_tier_frozen governance_log entry emitted at snapshot time        ║
║     Extended panel_assembled payload (adds severity_tier, status_tier,        ║
║       constraints_applied, relaxations[])                                     ║
║                                                                               ║
║   admin_emergency_remove.rs:                                                  ║
║     InsertForm sets severity_tier = SeverityTier::Severe                      ║
║                                                                               ║
║   config.rs:                                                                  ║
║     +get_int_cascade(cache, pool, scope, namespace, segments[]) -> i64        ║
║     +get_float_cascade(...) -> f64                                            ║
║     27 key count unchanged (no new keys seeded)                               ║
║                                                                               ║
║   jury_common.rs:                                                             ║
║     +panel_has_sponsor_majority_cluster(conn, person_ids) -> bool             ║
║     +recent_juror_cooldown_subquery(days) helper (SQL string or sub-filter)  ║
║                                                                               ║
║   moderation_case snapshot columns:                                           ║
║     panel_size_snapshot populated on every admin_assign_jury call             ║
║     quorum_snapshot + threshold_count_snapshot populated                      ║
║                                                                               ║
║   jury_assignment.selected_under_constraints JSONB populated on               ║
║     every new panel seating                                                   ║
║                                                                               ║
║   jury_constraint_violation_log table written to whenever the                 ║
║     relaxation cascade fires R1/R2/R3 for small-pool / cluster-pressure       ║
║                                                                               ║
║   Extended e2e test coverage:                                                 ║
║     admin_assign_jury_severity_tier_regular_minor_panel_5_jurors              ║
║     admin_assign_jury_severity_tier_regular_severe_panel_7_jurors             ║
║     admin_assign_jury_severity_tier_founder_severe_panel_9_jurors             ║
║     admin_assign_jury_small_pool_triggers_R1_relaxation                       ║
║     admin_assign_jury_writes_selected_under_constraints_jsonb                 ║
║     admin_assign_jury_emits_severity_tier_frozen_governance_log               ║
║     admin_emergency_remove_case_has_severity_tier_severe                      ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### 8.3 Endpoint changes (no new HTTP surface)

| Endpoint | v0 / pre-JM-b | Post-JM-b |
|---|---|---|
| `POST /api/v4/governance/admin/assign-jury` | reads bare `jury.panel_size = 5`, no diversity constraints, no severity awareness, panel_assembled payload `{case_id, juror_count}` | reads severity/status tiers from case + cascade, applies 3-phase diversity algorithm with R1/R2/R3 relaxation, writes panel_size/quorum/threshold_count snapshots, emits severity_tier_frozen + extended panel_assembled payload, writes selected_under_constraints JSONB per assignment row |
| `POST /api/v4/governance/admin/emergency-remove` | opens case with severity=Medium, severity_tier=Minor (backfill default) | opens case with severity=Medium + severity_tier=Severe so post-facto jury sizes Severe |
| `POST /api/v4/governance/submit-jury-vote` | reads hardcoded `QUORUM = 3` + `APPEAL_WINDOW_DAYS = 7` | **unchanged** (snapshot reads land in v1-JM-c) |
| `POST /api/v4/governance/request-appeal` | checks `case.closed_at.is_some()` | **unchanged** (v1-JM-d) |

No new routes. No DTO changes. `AdminAssignJuryResponse { case_id, assigned_person_ids }` is unchanged (no new fields).

---

## 9. Mandatory reading

The implementation agent MUST read these files before starting, in this order:

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `.claude/PRPs/prds/v1-jury-mechanics.prd.md` | §3 (severity), §4 (status), §5 (diversity + relaxation cascade), §9.2, §9.6, §10 (defaults matrix), §15 Resolutions | PRD is authoritative; §15 B2/B6/N3/NOT1 lock critical decisions |
| P0 | `.claude/PRPs/plans/phase-v1-JM-a.plan.md` | §4.1 (lock-in decisions), §10.2 (enum pattern), §10.7 (Queryable/InsertForm pattern), §12 (NOT building — defines JM-b scope by subtraction) | Predecessor plan defines scope boundary JM-b must respect |
| P0 | `crates/api/api/src/governance/admin_assign_jury.rs` | 1-371 | Full v0 file — JM-b rewrites `process_assignment` and `select_eligible_jurors` in place; Task 4/5 mirror the existing transaction shape |
| P0 | `crates/api/api/src/governance/config.rs` | 58-71 (Scope), 100-131 (get_int), 133-230 (other accessors), 269-320 (fetch_value cascade over scope), 354-414 (const_default_* arms), 922-1145 (SEEDED_KEYS_WITH_CONSTS incl. 27 JM-a entries), 1195+ (CONFIG_KEY_METADATA) | Existing cascade pattern to mirror for `get_int_cascade`; seed list for cross-checking key names |
| P0 | `crates/api/api/src/governance/jury_common.rs` | 1-70 | `shares_active_sponsor` — the pattern to extend to panel-level cluster check |
| P0 | `crates/api/api/src/governance/sponsor_liability.rs` | 112-124 (`severity_for_action`), 159-165 (sponsor-ids query), 217-228 (`is_founder` inline) | Source of truth for severity mapping (v1 reuses); pattern for querying `reputation_event` for founder_seed |
| P0 | `crates/db_schema/src/source/governance/moderation_case.rs` | full (post-JM-a) | All snapshot column fields to snapshot-into |
| P0 | `crates/db_schema/src/source/governance/jury_assignment.rs` | full (post-JM-a) | `selected_under_constraints` + `role` field |
| P0 | `crates/db_schema/src/source/governance/jury_constraint_violation_log.rs` | full (post-JM-a) | `JuryConstraintViolationLogInsertForm` shape — fields `case_id, constraint_name, reason_code, relaxation_metadata, pool_size_at_relax, panel_size_target` |
| P0 | `crates/db_schema/src/source/governance/governance_log.rs` | 112-180 (ENTRY_KIND_* consts incl. JM-a additions), 160-200 (`append` signature + body) | Emission call-site for severity_tier_frozen + jury_constraint_relaxed |
| P0 | `crates/db_schema_file/src/enums.rs` | `JuryConstraintRelaxationReason` definition (4 variants: SmallPool, ClusterPressure, ClusterPressureExhausted, AdminOverride) | Bounded vocabulary for `reason_code` JSONB + InsertForm field |
| P0 | `crates/api/api/src/governance/admin_emergency_remove.rs` | 140-175 | `ModerationCaseInsertForm` literal to extend with `severity_tier = Severe` |
| P1 | `crates/api/api/src/governance/accept_jury_assignment.rs` | 118-135 (`shares_active_sponsor` call), 155-170 (`governance_log::append` pattern with `json!` + pseudonym) | Call-site idiom for constraint relaxation log writes |
| P1 | `crates/api/api/src/governance/decline_jury_assignment.rs` | 110-150 (replacement selection — also calls `select_eligible_jurors`) | Second caller of `select_eligible_jurors`; JM-b must NOT break it |
| P1 | `crates/server/tests/e2e.rs` | ~900-990 (existing admin_assign_jury test fixture), 1293-1420 (config_parity_round_trip) | Extension point for new cascade and constraint-relaxation tests |
| P1 | `.claude/rules/cargo-output-capture.md`, `.claude/rules/no-cargo-output-paste.md`, `.claude/rules/phase-branch.md`, `.claude/rules/view-crate-selectable-template.md` (load-bearing reasoning only), `.claude/rules/pre-phase-harness-audit.md`, `.claude/rules/pm-plugin-hooks-stable.md` | all | Mandatory `-p` mode rules + phase branch + pre-phase audit |

### External documentation

| Source | Version | Section | Why |
|---|---|---|---|
| [diesel 2.x docs](https://docs.rs/diesel/2.2) | match `Cargo.toml` | `sql_query`, `bind`, `BigInt`, `Array<Integer>`, `Bool` | Existing `run_strict_eligibility_query` pattern to extend for panel-level cluster check |
| [serde_json 1](https://docs.rs/serde_json/1) | match `Cargo.toml` | `json!` macro, `Value` | Build the `selected_under_constraints` JSONB |

No new dependencies added by JM-b.

---

## 10. Patterns to mirror

Every snippet below is copied verbatim from the current workspace (or from the JM-a branch for post-merge shape) — not invented. Implementer should copy-paste and edit field names only.

### 10.1 `get_int` signature + body (pattern for `get_int_cascade`)

```rust
// SOURCE: crates/api/api/src/governance/config.rs:100-131
// COPY THIS PATTERN and extend with key-cascade walk:
pub async fn get_int(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<i64> {
  let scope_repr = scope.as_str();
  if let Some(CachedValue::Int(v)) = cache
    .entries
    .get(&(scope_repr.as_ref().to_string(), key.to_string()))
  {
    return Ok(*v);
  }
  let v = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Int(v)) => v,
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as int but stored as {other:?}"
      ))
      .into());
    }
    None => const_default_int(key).ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` missing from DB and has no Rust const default"
      ))
    })?,
  };
  cache
    .entries
    .insert((scope_repr.into_owned(), key.to_string()), CachedValue::Int(v));
  Ok(v)
}
```

**`get_int_cascade` reuses `fetch_value` four times** (once per cascade level) before falling to `const_default_int`. Each level is cache-keyed independently — cache lookups are cheap — so the cascade walk is at most 4 DB reads on first call, 0 on subsequent calls within the same `ConfigCache`.

### 10.2 Cascade walk pattern (new)

```rust
// SOURCE: NEW for v1-JM-b in crates/api/api/src/governance/config.rs
//
// Cascade: jury.panel_size.<status>.<severity>
//       -> jury.panel_size.<severity>
//       -> jury.panel_size
//       -> const_default_int("jury.panel_size")
//
// `namespace` = "jury.panel_size" (bare key)
// `segments` = &["founder", "severe"] (status, severity)
//
// The cascade tries most-specific first, falls through to bare-key, then const.
// Each level is a normal fetch_value call so the existing scope-cascade
// (community -> instance) applies at every level independently.

pub async fn get_int_cascade(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  namespace: &str,
  segments: &[&str],
) -> LemmyResult<i64> {
  // Walk from most-specific to least-specific.
  // Example segments = ["founder", "severe"]:
  //   try "jury.panel_size.founder.severe"
  //   then "jury.panel_size.severe"          (strip first segment)
  //   then "jury.panel_size"                 (bare)
  let mut candidates: Vec<String> = Vec::with_capacity(segments.len() + 1);
  if !segments.is_empty() {
    // Full dotted key first
    candidates.push(format!("{}.{}", namespace, segments.join(".")));
    // Per-severity fallback (strip status segment) — PRD §3.5 reserves
    // this form but the JM-a seed does NOT populate it; it exists for
    // future per-severity admin tuning without per-status differentiation.
    for i in 1..segments.len() {
      candidates.push(format!("{}.{}", namespace, segments[i..].join(".")));
    }
  }
  // Bare namespace key as final DB-level fallback
  candidates.push(namespace.to_string());

  for candidate in &candidates {
    let scope_repr = scope.as_str();
    if let Some(CachedValue::Int(v)) = cache
      .entries
      .get(&(scope_repr.as_ref().to_string(), candidate.clone()))
    {
      return Ok(*v);
    }
    match fetch_value(pool, scope, candidate).await? {
      Some(CachedValue::Int(v)) => {
        cache.entries.insert(
          (scope_repr.into_owned(), candidate.clone()),
          CachedValue::Int(v),
        );
        return Ok(v);
      }
      Some(other) => {
        return Err(LemmyErrorType::Unknown(format!(
          "governance_config key `{candidate}` requested as int but stored as {other:?}"
        ))
        .into());
      }
      None => continue,
    }
  }

  // Const fallback — always use the bare namespace, consistent with bare-key
  // cascade tail. Matches v0 behaviour for jury.panel_size (returns 5).
  const_default_int(namespace).ok_or_else(|| {
    LemmyErrorType::Unknown(format!(
      "cascade walked {:?} — no DB row found and no Rust const default for namespace `{namespace}`",
      candidates
    ))
  })
}
```

`get_float_cascade` follows the same shape with `const_default_float` + `CachedValue::Float`. Do NOT add `get_bool_cascade` / `get_text_cascade` in JM-b — the PRD does not use them and YAGNI applies.

### 10.3 Rust-side status-tier compute (new, module-private to admin_assign_jury.rs)

```rust
// SOURCE: NEW helper in crates/api/api/src/governance/admin_assign_jury.rs
// MIRROR: sponsor_liability.rs:217-228 (is_founder inline query)
//
// Returns the status tier for the case's target. Cases with no
// target_person_id (post/comment/community-targeted) default to Regular
// per PRD §4.2.

async fn compute_status_tier(
  conn: &mut diesel_async::AsyncPgConnection,
  case: &ModerationCase,
) -> LemmyResult<CaseStatusTier> {
  use diesel::dsl::{exists, now, select};
  use diesel::prelude::*;
  use lemmy_db_schema_file::enums::{MembershipState, ReputationDimension};
  use lemmy_db_schema_file::schema::{person, reputation_event};

  let Some(target_id) = case.target_person_id else {
    return Ok(CaseStatusTier::Regular);
  };

  // Founder check — matches is_founder at sponsor_liability.rs:219-228
  let is_founder: bool = select(exists(
    reputation_event::table
      .filter(reputation_event::person_id.eq(target_id))
      .filter(reputation_event::dimension.eq(ReputationDimension::EndorsementStrength))
      .filter(reputation_event::reason.eq("founder_seed"))
      .filter(reputation_event::expires_at.is_not_null())
      .filter(reputation_event::expires_at.gt(now)),
  ))
  .get_result::<bool>(conn)
  .await?;
  if is_founder {
    return Ok(CaseStatusTier::Founder);
  }

  // Probation check — provisional membership state
  let is_provisional: Option<MembershipState> = person::table
    .filter(person::id.eq(target_id))
    .select(person::membership_state)
    .first::<MembershipState>(conn)
    .await
    .optional()?;
  if matches!(is_provisional, Some(MembershipState::Provisional)) {
    return Ok(CaseStatusTier::Probation);
  }

  Ok(CaseStatusTier::Regular)
}
```

**GOTCHA:** `matches!(MembershipState::Provisional)` — the `MembershipState` enum uses `DbValueStyle = "snake_case"` (the documented deferred-enforcement exception per JM-a plan §4.1). Double-check the variant name post-JM-a-merge — if it changed (e.g. `provisional_member` vs `Provisional`), update accordingly. The Phase 5a enum definition in `enums.rs:624-648` is authoritative.

### 10.4 Severity-tier → panel-size resolution (new, in `process_assignment`)

```rust
// SOURCE: NEW logic in crates/api/api/src/governance/admin_assign_jury.rs
// step 3 replacement (around line 118-121)
//
// Previous: bare jury.panel_size read
// New: cascade on (status, severity) with float -> int resolution

use lemmy_db_schema_file::enums::{CaseStatusTier, SeverityTier};

let severity = case.severity_tier;  // NOT NULL — JM-a backfill / DEFAULT 'Minor'
let status = if case.status_tier == CaseStatusTier::Regular {
  // JM-a DEFAULT; compute lazily — may actually be Founder/Probation
  compute_status_tier(conn, &case).await?
} else {
  // Backfilled or previously-set; trust the snapshot
  case.status_tier
};

// Cascade: jury.panel_size.<status>.<severity> -> .<severity> -> bare -> const
let status_str = status_tier_slug(status);        // "founder" | "regular" | "probation"
let severity_str = severity_tier_slug(severity);  // "minor" | "moderate" | "severe"

let panel_size = config::get_int_cascade(
  &mut cache,
  &mut (&mut *conn).into(),
  Scope::Instance,  // Per-community scope override handled by fetch_value
  "jury.panel_size",
  &[status_str, severity_str],
)
.await?;

// Fractions cascade only on <severity> (PRD §4.4: "quorum and threshold
// inherit severity, not status")
let quorum_fraction = config::get_float_cascade(
  &mut cache,
  &mut (&mut *conn).into(),
  Scope::Instance,
  "jury.quorum_fraction",
  &[severity_str],
)
.await?;
let threshold_fraction = config::get_float_cascade(
  &mut cache,
  &mut (&mut *conn).into(),
  Scope::Instance,
  "jury.threshold_fraction",
  &[severity_str],
)
.await?;

// Resolve to integer counts via ceil (PRD §3.4)
let quorum = ((panel_size as f64) * quorum_fraction).ceil() as i32;
let threshold_count = ((panel_size as f64) * threshold_fraction).ceil() as i32;
let panel_size_i32 = panel_size as i32;
```

**GOTCHA:** `as i32` cast from `i64` is a narrowing conversion. The PRD bounds panel_size to `[3, 11]` which fits; assume the admin-dashboard write-path enforces the bound. Do NOT add `.try_into()` here unless the workspace-level `cast_possible_truncation` lint flags it; if it does, wrap in `i32::try_from(panel_size).map_err(|_| LemmyErrorType::Unknown(...))`.

**GOTCHA:** `ceil()` on `0.5001 * 5.0 = 2.5005` → `ceil = 3`. On `0.75 * 7.0 = 5.25` → `ceil = 6`. On `0.71 * 7.0 = 4.97` → `ceil = 5`. Hand-verify the defaults table (PRD §10) matches: Minor 5/3/3, Moderate 5/3/3 (60% of 5 = 3.0, ceil = 3), Severe 7/5/6. Test assertions in §14 cover this.

### 10.5 Snapshot write (new, in `process_assignment` between current steps 6 and 7)

```rust
// SOURCE: NEW — matches the existing moderation_case::status update at
// admin_assign_jury.rs:154-157 idiom
use crate::schema::moderation_case::dsl as mc;

update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
  .set((
    mc::panel_size_snapshot.eq(Some(panel_size_i32)),
    mc::quorum_snapshot.eq(Some(quorum)),
    mc::threshold_count_snapshot.eq(Some(threshold_count)),
    mc::status_tier.eq(status),
    mc::status.eq(CaseStatus::JurySelection),
  ))
  .execute(conn)
  .await?;
```

Fold the v0 status flip (step 6 at lines 154-157) into this same `UPDATE` — one round trip instead of two. The `mc::status.eq(CaseStatus::JurySelection)` line is the v0 behaviour preserved.

### 10.6 severity_tier_frozen governance_log emission (new, between snapshot write and per-juror loop)

```rust
// SOURCE: NEW — matches the panel_assembled emission at admin_assign_jury.rs:178-187
use crate::governance::governance_log::ENTRY_KIND_SEVERITY_TIER_FROZEN;

governance_log::append(
  &mut conn.into(),
  ENTRY_KIND_SEVERITY_TIER_FROZEN,
  json!({
    "case_id": data.case_id.0,
    "severity_tier": severity_tier_slug(severity),
    "status_tier": status_tier_slug(status),
    "panel_size_snapshot": panel_size_i32,
    "quorum_snapshot": quorum,
    "threshold_count_snapshot": threshold_count,
  }),
  Some(admin_pseudonym.clone()),
)
.await?;
```

Emit BEFORE the per-juror `jury_assigned` emissions so the audit timeline reads `severity_tier_frozen → jury_assigned × N → panel_assembled`.

### 10.7 Panel-level sponsor-cluster helper (new, in `jury_common.rs`)

```rust
// SOURCE: NEW in crates/api/api/src/governance/jury_common.rs
// MIRROR: shares_active_sponsor at jury_common.rs:23-45 (raw SQL pattern)
//
// Returns true if >50% of the panel shares a single sponsor (active,
// non-revoked). PRD §5.1 "Hard (re-roll if violated)" constraint.

use diesel::sql_types::{Array, BigInt, Integer};

#[derive(QueryableByName)]
struct ClusterCountRow {
  #[diesel(sql_type = BigInt)]
  max_shared: i64,
}

pub(crate) async fn panel_has_sponsor_majority_cluster(
  conn: &mut AsyncPgConnection,
  person_ids: &[PersonId],
) -> LemmyResult<bool> {
  if person_ids.len() < 2 {
    return Ok(false);
  }
  let ids_bind: Vec<i32> = person_ids.iter().map(|p| p.0).collect();

  // For each sponsor active for any panel member, count how many distinct
  // panel members share that sponsor. Max across sponsors is the cluster size.
  let row: ClusterCountRow = diesel::sql_query(
    "SELECT COALESCE(MAX(c), 0) AS max_shared FROM ( \
       SELECT s.sponsor_id, COUNT(DISTINCT s.sponsored_id) AS c \
       FROM surety s \
       WHERE s.sponsored_id = ANY($1) \
         AND s.revoked_at IS NULL \
       GROUP BY s.sponsor_id \
     ) t",
  )
  .bind::<Array<Integer>, _>(ids_bind)
  .get_result(conn)
  .await
  .map_err(|_e| {
    LemmyErrorType::Unknown("panel_has_sponsor_majority_cluster query failed".to_string())
  })?;

  // >50% means strictly more than half. For panel_size = 7, majority = 4.
  // For panel_size = 5, majority = 3.
  let majority = (person_ids.len() / 2) + 1;
  Ok(row.max_shared as usize >= majority)
}
```

**GOTCHA:** `majority = (len / 2) + 1` for odd panel sizes gives strict >50%. For even panel sizes (which PRD §3.4 bounds out anyway — "must be odd") this still returns "more than half". Odd-panel-size is enforced by the admin-dashboard write-path; JM-b does not re-validate.

### 10.8 `select_eligible_jurors` rewrite — 3-phase algorithm (new shape)

```rust
// SOURCE: NEW shape for crates/api/api/src/governance/admin_assign_jury.rs:215-274
// MIRROR: existing scaffolding — keep ConfigCache, excluded IDs, run_strict_eligibility_query,
//         legacy_select_eligible_jurors fallback — but wrap with phases and relaxation cascade.
//
// Returns (eligible_panel, constraint_record) so the caller can write the
// per-assignment selected_under_constraints JSONB and panel_assembled payload.

#[derive(Debug, Clone, Default)]
pub(crate) struct ConstraintRecord {
  pub no_majority_from_same_sponsor_cluster: &'static str,  // "applied" | "relaxed" | "disabled"
  pub geographic_diversity_preferred: &'static str,         // "applied_soft" | "disabled" | "relaxed"
  pub no_recent_juror_repeat: &'static str,                 // "applied" | "relaxed_small_pool" | "disabled"
  pub no_same_endorsement_chain: &'static str,              // always "disabled" in v1-JM-b
  pub relaxations_fired: Vec<&'static str>,  // ["R1", "R2", "R3"] for summary
}

pub(crate) async fn select_eligible_jurors(
  conn: &mut diesel_async::AsyncPgConnection,
  case: &ModerationCase,
  panel_size: i64,                   // JM-b change: caller computes via cascade
  exclude_person_ids: Option<&[PersonId]>,
  cache: &mut ConfigCache,
) -> LemmyResult<(Vec<PersonId>, ConstraintRecord)> {
  // Read all constraint toggles (all have const defaults from JM-a seed)
  let cooldown_enabled = config::get_bool(cache, &mut (&mut *conn).into(), Scope::Instance,
    "jury.constraints.no_recent_juror_repeat").await?;
  let cooldown_days = config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance,
    "jury.constraints.juror_cooldown_days").await?;
  let cluster_constraint_enabled = config::get_bool(cache, &mut (&mut *conn).into(), Scope::Instance,
    "jury.constraints.no_majority_from_same_sponsor_cluster").await?;
  let geo_pref_enabled = config::get_bool(cache, &mut (&mut *conn).into(), Scope::Instance,
    "jury.constraints.geographic_diversity_preferred").await?;
  let max_retries = config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance,
    "jury.constraints.max_retries_before_relax").await?;
  let max_concurrent = config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance,
    "jury.max_concurrent_assignments").await?;
  let max_concurrent_per_juror_total = config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance,
    "jury.max_concurrent_assignments_per_juror_total").await?;
  let fallback_allowed = config::get_bool(cache, &mut (&mut *conn).into(), Scope::Instance,
    "jury.fallback_on_small_pool").await?;

  let mut record = ConstraintRecord {
    no_majority_from_same_sponsor_cluster: if cluster_constraint_enabled { "applied" } else { "disabled" },
    geographic_diversity_preferred: if geo_pref_enabled { "applied_soft" } else { "disabled" },
    no_recent_juror_repeat: if cooldown_enabled { "applied" } else { "disabled" },
    no_same_endorsement_chain: "disabled",
    relaxations_fired: Vec::new(),
  };

  // Excluded IDs set (same shape as v0)
  let mut excluded: Vec<PersonId> = Vec::new();
  if let Some(t) = case.target_person_id { excluded.push(t); }
  if let Some(r) = case.creator_id { excluded.push(r); }
  if let Some(xs) = exclude_person_ids { excluded.extend_from_slice(xs); }

  // ============================================================
  // PHASE 1 — pool build (filter)
  // ============================================================
  let mut current_cooldown_days = if cooldown_enabled { Some(cooldown_days) } else { None };
  let mut pool: Vec<PersonId> = run_extended_eligibility_query(
    conn, case.community_id, &excluded, max_concurrent,
    max_concurrent_per_juror_total, current_cooldown_days,
    // Build a big-enough pool (10x panel_size or 50, whichever smaller)
    panel_size.saturating_mul(10).min(200),
  ).await?;

  if (pool.len() as i64) < panel_size && current_cooldown_days.is_some() {
    // R1 — drop cooldown and re-run
    tracing::info!(
      case_id = case.id.0, pool_size = pool.len(), target = panel_size,
      "jury selection R1: dropping no_recent_juror_repeat (small_pool)"
    );
    write_constraint_relaxation(
      conn, case, "no_recent_juror_repeat", JuryConstraintRelaxationReason::SmallPool,
      json!({"phase": "pool_build", "dropped_constraint_name": "no_recent_juror_repeat"}),
      pool.len() as i32, panel_size as i32,
    ).await?;
    record.no_recent_juror_repeat = "relaxed_small_pool";
    record.relaxations_fired.push("R1");
    current_cooldown_days = None;
    pool = run_extended_eligibility_query(
      conn, case.community_id, &excluded, max_concurrent,
      max_concurrent_per_juror_total, None,
      panel_size.saturating_mul(10).min(200),
    ).await?;
  }

  // If still < panel_size after R1, fall through to legacy if allowed
  if (pool.len() as i64) < panel_size {
    if !fallback_allowed {
      return Ok((pool, record));
    }
    tracing::warn!(
      case_id = case.id.0, community_id = ?case.community_id.map(|c| c.0),
      pool_size = pool.len(), panel_size,
      "jury pool below panel_size post-R1 — legacy Phase 4 fallback per jury.fallback_on_small_pool=true",
    );
    let legacy = legacy_select_eligible_jurors(conn, &excluded, panel_size).await?;
    return Ok((legacy, record));
  }

  // ============================================================
  // PHASE 2 — sample with re-roll on sponsor cluster
  // ============================================================
  let mut chosen: Option<Vec<PersonId>> = None;
  let mut best_score: f64 = -1.0;
  let mut current_geo_enabled = geo_pref_enabled;

  for attempt in 0..max_retries {
    // Weighted random — for v1-JM-b, same as v0 `ORDER BY random()`; pool
    // is already pre-randomised by the SQL so take the first `panel_size`.
    let sample = sample_panel(&pool, panel_size as usize);

    // Sponsor-cluster check
    let violates = if cluster_constraint_enabled && record.no_majority_from_same_sponsor_cluster != "relaxed" {
      panel_has_sponsor_majority_cluster(conn, &sample).await?
    } else {
      false
    };
    if violates {
      // Re-roll with a fresh pool-shuffle on next iteration
      // (by randomising the pool slice; sample_panel can accept a seed)
      continue;
    }

    // PHASE 3 — soft geographic score (only if enabled and not the first passing sample)
    if current_geo_enabled && geo_pref_enabled {
      let score = geographic_diversity_score(conn, &sample).await.unwrap_or(0.0);
      if score > best_score {
        best_score = score;
        chosen = Some(sample);
      } else if chosen.is_none() {
        chosen = Some(sample);
      }
    } else {
      // First passing sample wins
      chosen = Some(sample);
      break;
    }
  }

  if chosen.is_some() {
    return Ok((chosen.unwrap(), record));
  }

  // ============================================================
  // R2 — drop geographic scoring bias; re-try N_RETRIES without it
  // ============================================================
  if cluster_constraint_enabled {
    tracing::info!(
      case_id = case.id.0,
      "jury selection R2: dropping geographic_diversity_preferred bias (cluster_pressure)"
    );
    write_constraint_relaxation(
      conn, case, "geographic_diversity_preferred", JuryConstraintRelaxationReason::ClusterPressure,
      json!({"phase": "panel_sample", "dropped_constraint_name": "geographic_diversity_preferred"}),
      pool.len() as i32, panel_size as i32,
    ).await?;
    record.geographic_diversity_preferred = "relaxed";
    record.relaxations_fired.push("R2");
    current_geo_enabled = false;

    for _attempt in 0..max_retries {
      let sample = sample_panel(&pool, panel_size as usize);
      let violates = panel_has_sponsor_majority_cluster(conn, &sample).await?;
      if !violates {
        return Ok((sample, record));
      }
    }
  }

  // ============================================================
  // R3 — last resort; drop sponsor-cluster entirely
  // ============================================================
  tracing::warn!(
    case_id = case.id.0,
    "jury selection R3: dropping no_majority_from_same_sponsor_cluster (cluster_pressure_exhausted)"
  );
  write_constraint_relaxation(
    conn, case, "no_majority_from_same_sponsor_cluster",
    JuryConstraintRelaxationReason::ClusterPressureExhausted,
    json!({"phase": "panel_sample", "dropped_constraint_name": "no_majority_from_same_sponsor_cluster"}),
    pool.len() as i32, panel_size as i32,
  ).await?;
  record.no_majority_from_same_sponsor_cluster = "relaxed";
  record.relaxations_fired.push("R3");
  let last_sample = sample_panel(&pool, panel_size as usize);
  Ok((last_sample, record))
}
```

**GOTCHA:** `sample_panel` needs to be a standalone helper that takes `&[PersonId]` and returns a `Vec<PersonId>` of first-N from a fresh `rand::shuffle`. Use `rand::seq::SliceRandom::choose_multiple` (already a transitive Cargo dep — verify in `Cargo.toml` before Task 1) OR keep `ORDER BY random()` at the SQL level and fetch a fresh pool inside the loop. **Cleaner option**: the SQL already randomises; instead of `sample_panel`, re-run the pool query with `LIMIT panel_size` inside the loop. Each iteration gets a fresh random sample from the pool. This avoids adding `rand` as a direct dep.

**GOTCHA:** `geographic_diversity_score` — see §10.9 below. If no `actor` field for timezone/community exists yet, return `0.0` for every panel (making geo scoring a no-op but leaving the plumbing in place). The PRD §OQ-V1-JM-02 explicitly leans "declared community for cross-community jurors (where multi-community)"; if the workspace doesn't have that plumbing, v1-JM-b ships the stub with `community_id` from `jury_assignment` / `moderation_case` and flags the full heuristic as a v1.5 concern.

**GOTCHA:** the loop structure must write at most ONE `jury_constraint_violation_log` row per constraint per case. Re-entering the inner loop on re-roll must NOT re-emit. The `record.relaxations_fired` check gates this.

### 10.9 Geographic diversity score stub (new, in jury_common.rs or admin_assign_jury.rs)

```rust
// SOURCE: NEW — stub for v1-JM-b. Full heuristic (timezone + declared community)
// is OQ-V1-JM-02 territory and deferred to v1.5.
//
// v1-JM-b contract: return the fraction of distinct community_ids the panel
// members appear in (via their own recent jury_assignment history) divided
// by panel_size. `0.0` = all same community; `1.0` = every juror a different
// community. When no community plumbing is available, returns 0.0 uniformly
// and the soft-bias is a no-op.

async fn geographic_diversity_score(
  conn: &mut diesel_async::AsyncPgConnection,
  sample: &[PersonId],
) -> LemmyResult<f64> {
  if sample.is_empty() { return Ok(0.0); }
  // COUNT(DISTINCT community_id) across the sample's recent jury_assignment
  // + moderation_case history.
  use diesel::sql_types::{Array, BigInt, Integer};

  #[derive(QueryableByName)]
  struct CommunityCountRow {
    #[diesel(sql_type = BigInt)]
    distinct_count: i64,
  }
  let ids_bind: Vec<i32> = sample.iter().map(|p| p.0).collect();
  let row: CommunityCountRow = diesel::sql_query(
    "SELECT COUNT(DISTINCT mc.community_id) AS distinct_count \
     FROM jury_assignment ja \
     INNER JOIN moderation_case mc ON mc.id = ja.case_id \
     WHERE ja.person_id = ANY($1) \
       AND mc.community_id IS NOT NULL",
  )
  .bind::<Array<Integer>, _>(ids_bind)
  .get_result(conn)
  .await?;
  Ok((row.distinct_count as f64) / (sample.len() as f64))
}
```

**GOTCHA:** Jurors with no prior assignments contribute 0 to `distinct_count`. For a bootstrapping instance this makes `score = 0.0` for every panel — which is fine because the soft bias is a no-op when all scores are equal.

### 10.10 Constraint-relaxation audit write helper (new, module-private)

```rust
// SOURCE: NEW helper in crates/api/api/src/governance/admin_assign_jury.rs
// or pull into crates/api/api/src/governance/jury_common.rs if size justifies.
// MIRROR: governance_log::append call at admin_assign_jury.rs:178-187

use lemmy_db_schema::source::governance::jury_constraint_violation_log::{
  JuryConstraintViolationLog, JuryConstraintViolationLogInsertForm,
};
use lemmy_db_schema_file::enums::JuryConstraintRelaxationReason;
use crate::governance::governance_log::ENTRY_KIND_JURY_CONSTRAINT_RELAXED;

async fn write_constraint_relaxation(
  conn: &mut diesel_async::AsyncPgConnection,
  case: &ModerationCase,
  constraint_name: &str,
  reason_code: JuryConstraintRelaxationReason,
  metadata: serde_json::Value,
  pool_size_at_relax: i32,
  panel_size_target: i32,
) -> LemmyResult<()> {
  // 1. Queryable index row
  let form = JuryConstraintViolationLogInsertForm {
    case_id: case.id,
    constraint_name: constraint_name.to_string(),
    reason_code,
    relaxation_metadata: Some(metadata.clone()),
    pool_size_at_relax,
    panel_size_target,
  };
  diesel::insert_into(lemmy_db_schema::schema::jury_constraint_violation_log::table)
    .values(&form)
    .execute(conn)
    .await?;

  // 2. Tamper-evident governance_log entry
  //    Payload includes reason_code as snake_case string (per enum serde rename)
  governance_log::append(
    &mut conn.into(),
    ENTRY_KIND_JURY_CONSTRAINT_RELAXED,
    json!({
      "case_id": case.id.0,
      "constraint_name": constraint_name,
      "reason_code": serde_json::to_value(&reason_code).unwrap_or(serde_json::Value::Null),
      "pool_size_at_relax": pool_size_at_relax,
      "panel_size_target": panel_size_target,
      "metadata": metadata,
    }),
    None,  // System-level relaxation; no admin_pseudonym attribution
  )
  .await?;

  Ok(())
}
```

**GOTCHA:** `serde_json::to_value(&reason_code).unwrap_or(Null)` — `JuryConstraintRelaxationReason` derives Serialize per JM-a enum definition (snake_case per `#[serde(rename_all = "snake_case")]`). The fallback `unwrap_or(Null)` is a belt-and-braces guard; the serialization should never fail on an enum. Under no circumstances write the variant Debug representation — that would be "`SmallPool`" (PascalCase) instead of "`small_pool`" and diverge from PRD §5.3's narrative vocabulary.

**GOTCHA:** `None` as actor_pseudonym is correct for system-level relaxation — the relaxation is not attributed to any individual. Compare to `admin_emergency_remove` where the admin's pseudonym IS attributed. The distinction matters for the modlog UX.

### 10.11 `selected_under_constraints` JSONB per-assignment write

```rust
// SOURCE: NEW in the per-juror InsertForm construction loop
// (replacement for current admin_assign_jury.rs:140-151 block)

let constraints_applied_json = json!({
  "no_majority_from_same_sponsor_cluster": record.no_majority_from_same_sponsor_cluster,
  "geographic_diversity_preferred": record.geographic_diversity_preferred,
  "no_recent_juror_repeat": record.no_recent_juror_repeat,
  "no_same_endorsement_chain": record.no_same_endorsement_chain,
});

let forms: Vec<JuryAssignmentInsertForm> = eligible
  .iter()
  .map(|person_id| JuryAssignmentInsertForm {
    case_id: data.case_id,
    person_id: *person_id,
    status: JuryAssignmentStatus::Selected,
    selected_under_constraints: Some(constraints_applied_json.clone()),
    // role: DEFAULT 'Original' via Postgres — JM-a InsertForm has no role field
  })
  .collect();
```

**GOTCHA:** `JuryAssignmentInsertForm` from JM-a DOES have a `selected_under_constraints: Option<Value>` field. Verify post-JM-a-merge by reading `jury_assignment.rs:30-47`. If it doesn't, that's a JM-a drift that must be fixed via DQ entry before Task 4.

**GOTCHA:** DO NOT add `role: JuryAssignmentRole::Original` to the InsertForm. The JM-a plan §4.1 deliberately omits `role` from the InsertForm so the Postgres DEFAULT fires; the v0/v1-JM-b writer is implicit-Original. Adding an explicit Original would work (the DEFAULT match is redundant) BUT the v1-JM-d appeal path will add `role` as an explicit field for the `Appeal` writer, and keeping JM-b's writer default-driven is the cleanest forward-compatibility story.

### 10.12 Extended `panel_assembled` payload

```rust
// SOURCE: REPLACEMENT for admin_assign_jury.rs:178-187
governance_log::append(
  &mut conn.into(),
  ENTRY_KIND_PANEL_ASSEMBLED,
  json!({
    "case_id": data.case_id.0,
    "juror_count": panel_size_i32,
    "severity_tier": severity_tier_slug(severity),
    "status_tier": status_tier_slug(status),
    "constraints_applied": {
      "no_majority_from_same_sponsor_cluster": record.no_majority_from_same_sponsor_cluster,
      "geographic_diversity_preferred": record.geographic_diversity_preferred,
      "no_recent_juror_repeat": record.no_recent_juror_repeat,
      "no_same_endorsement_chain": record.no_same_endorsement_chain,
    },
    "relaxations": record.relaxations_fired,
  }),
  Some(admin_pseudonym.clone()),
)
.await?;
```

### 10.13 `admin_emergency_remove` severity_tier injection

```rust
// SOURCE: REPLACEMENT for crates/api/api/src/governance/admin_emergency_remove.rs:143-157
// Add one line to the ModerationCaseInsertForm literal:

use lemmy_db_schema_file::enums::SeverityTier;

let form = ModerationCaseInsertForm {
  community_id,
  creator_id: Some(admin_id),
  target_type: target.case_target_type(),
  target_post_id,
  target_comment_id,
  target_person_id: None,
  target_community_id,
  target_remote_url: None,
  reason_code: "emergency_remove".to_string(),
  severity: CaseSeverity::default(),   // Medium — unchanged v0 field
  severity_tier: SeverityTier::Severe, // NEW — PRD §9.6 / ADR-013
  status: CaseStatus::EmergencyRemove,
  threshold_score: 0,
  ..Default::default()
};
```

**GOTCHA:** `ModerationCaseInsertForm` derives `Default`. Adding a non-defaulted field when the form already carries `severity_tier: SeverityTier::default()` (Minor) would silently override to Minor if the explicit assignment is omitted. Verify from the JM-a model that `severity_tier` has a `Default` impl returning `Minor` — it does, per JM-a enum `#[default] Minor`.

---

## 11. Files to change

| File | Action | Justification |
|---|---|---|
| `crates/api/api/src/governance/config.rs` | UPDATE | Add `get_int_cascade` + `get_float_cascade` functions; no existing function signatures change |
| `crates/api/api/src/governance/admin_assign_jury.rs` | UPDATE | Rewrite `process_assignment` + `select_eligible_jurors` shape; add `compute_status_tier`, `status_tier_slug`, `severity_tier_slug`, `write_constraint_relaxation`, `geographic_diversity_score` helpers; extend `run_strict_eligibility_query` into `run_extended_eligibility_query` (new name) with cooldown + per-juror-total params |
| `crates/api/api/src/governance/jury_common.rs` | UPDATE | Add `panel_has_sponsor_majority_cluster` public-crate helper next to existing `shares_active_sponsor` |
| `crates/api/api/src/governance/admin_emergency_remove.rs` | UPDATE | Add `severity_tier: SeverityTier::Severe` to the `ModerationCaseInsertForm` literal |
| `crates/server/tests/e2e.rs` | UPDATE | Add 7 new tests per §14 (severity/status combinations × 3, R1 relaxation, selected_under_constraints JSONB assertion, severity_tier_frozen assertion, admin_emergency_remove severity_tier=Severe assertion) |
| `.claude/decision-queue.json` | UPDATE | Open OQ-V1-JM-07 per §7.1 (Task 1) |
| `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` | UPDATE | Open OQ-V1-JM-07 per §7.1 (Task 1) — stub only, no resolution |
| `.claude/PRPs/reports/v1-JM-b-retro.md` | CREATE | Phase retrospective at Task 10 before PR open |

**No migration files added.** All schema changes landed in JM-a. The three new helpers live in existing files.

**No Cargo.toml changes.** All crate dependencies already pulled in transitively.

---

## 12. NOT building in v1-JM-b

Explicitly out of scope. If one of these creeps in, STOP and file a decision-queue entry.

- **No `submit_jury_vote.rs` edits.** The 9-step combined handler (quorum/threshold snapshot reads, deadlock→AdminReview, sponsor-liability branch, appeal_window_expires_at write) is v1-JM-c. JM-b does not read snapshot columns from submit_jury_vote; it only writes them.
- **No `request_appeal.rs` edits.** Bounded-window check, reporter-rights extension, auto re-jury seeding are all v1-JM-d.
- **No new HTTP routes.** `admin_trigger_appeal_rejury` is v1-JM-d; `crates/api/routes/src/governance.rs` is untouched.
- **No `accept_jury_assignment.rs` / `decline_jury_assignment.rs` edits.** The one-hop `shares_active_sponsor` check stays as-is; it's a per-juror check, distinct from the new panel-level check. Decline-replacement continues to call `select_eligible_jurors` via the new signature; verify post-rewrite that the single-person replacement still works under the 3-phase algorithm.
- **No new DTO fields on `AdminAssignJury` / `AdminAssignJuryResponse`.** The per-case severity/status tiers are read off the DB, not passed in the request. No new response fields.
- **No case-open severity inference.** Per §7.1, general `reason_code → severity_tier` logic is a post-JM-b OQ. `admin_emergency_remove` is the ONE exception (explicit Severe per ADR-013).
- **No per-community overrides for the cascade**. The JM-a `Scope` enum supports community scope and the existing `fetch_value` handles it, so the cascade picks it up automatically. JM-b adds no new per-community write path.
- **No reputation-dimension changes.** `jury_eligible` and `jury_reliability` reads stay as in Phase 5b task 57. Reputation-tuning sub-PRD owns those.
- **No federation work.** Governance signals stay within the local hash chain; `jury_constraint_relaxed` is a local-only entry-kind (not federated). ADR-014 content-level federation unaffected.
- **No new AGPL / release artefacts.** Not a release sub-phase.
- **No changes to the seven PM plugin hooks** per `.claude/rules/pm-plugin-hooks-stable.md`. Task 0 grep check is mandatory.
- **No new migration files.** 27 JM-a seeded keys are sufficient; parity tests remain at `EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD + EXPECTED_SEED_COUNT_V1_JM = 34 + 27 + 27 = 88`.
- **No `Cargo.toml` changes.** `serde_json`, `diesel`, `chrono` already in `lemmy_api`'s deps. `rand::seq::SliceRandom` not added — re-use SQL `ORDER BY random()` per §10.8 GOTCHA.
- **No admin-dashboard PR concurrent with JM-b.** Verify in Task 0 via `gh pr list --base governance-v0 --state open` that no `config.rs`-touching PR is in flight.

---

## 13. Step-by-step tasks

Execute in order. **One commit per task** on branch `phase-v1-JM-b`. Each task has a MIRROR reference, exact file paths, and a validation command. Task 0 is a pre-flight gate with no commits; Tasks 1–9 each produce one commit; Task 10 is the retro write-up (commit) before PR open.

### Task 0: PRE-FLIGHT — verify branch + wrapper sanity + concurrent-PR check + harness audit

- **ACTION**: Confirm branch + run `.claude/rules/pre-phase-harness-audit.md` probes + verify no open `config.rs`-touching PR against `governance-v0`.
- **COMMIT MESSAGE**: No commit from Task 0.
- **GOTCHA**: If the current branch is `governance-v0`, STOP and write a `.claude/decision-queue.json` entry — advisor cuts the phase branch, not the impl agent (per `.claude/rules/phase-branch.md:7–15`).
- **GOTCHA**: All four pre-phase-harness probes (1/2/3 positive + 4 negative) must match the expected exit codes. Failure = wrapper bug; fix wrapper in a pre-Task 1 commit.
- **GOTCHA**: If `gh pr list --repo barrie-cork/lemmy --base governance-v0 --state open` returns a PR labelled or scoped to `v1-AD-*` or touches `crates/api/api/src/governance/config.rs`, pause and ask the user — concurrent config.rs edits have high conflict probability.
- **VALIDATE**:

  ```bash
  git branch --show-current   # expect: phase-v1-JM-b
  git merge-base phase-v1-JM-b governance-v0   # record SHA
  git log -1 --format=%H governance-v0          # should match merge-base
  docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER DOWN — start Docker Desktop"; exit 1; }
  gh pr list --repo barrie-cork/lemmy --base governance-v0 --state open --json number,title,files --limit 20 > .claude/PRPs/debug/v1-JM-b-open-prs.json
  jq -r '.[] | select(.files[].path | test("config.rs")) | .number' .claude/PRPs/debug/v1-JM-b-open-prs.json  # should be empty
  for h in local_private_message_before_create local_private_message_after_create \
           local_private_message_before_update local_private_message_after_update \
           federated_private_message_before_receive federated_private_message_after_receive; do
    rg -q "\"$h\"" crates/ || { echo "missing PM hook literal: $h"; exit 1; }
  done
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/PRPs/debug/v1-JM-b-audit-probe1.log 2>&1"
  echo "probe1 exit: $?"   # expect 0
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features nonexistent_xyz > .claude/PRPs/debug/v1-JM-b-audit-probe4.log 2>&1"
  echo "probe4 exit: $?"   # expect non-zero (wrapper propagates cargo failure)
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-JM-b-audit-probe3.log 2>&1"
  echo "probe3 exit: $?"   # expect 0
  ```
- **EXPECT**: Current branch `phase-v1-JM-b`; `git merge-base` output equals the JM-a-merge SHA on `governance-v0`; no concurrent config.rs-touching open PR; Docker up; 6 PM hook literals present; probes 1+3 exit 0; probe 4 exits non-zero.

### Task 1: OPEN OQ-V1-JM-07 per §7.1

- **ACTION**: Append OQ-V1-JM-07 to `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` (stub only — no resolution). Also add a minimal `.claude/decision-queue.json` entry with `answered_by: null` and `pending` status.
- **IMPLEMENT**: One paragraph in 99-decisions under the "Open questions" table referring to "post-JM-b general case-open severity inference"; cite §7.1 of this plan.
- **MIRROR**: `v1-AD-a` plan Task 7 precedent (three OQs added as stubs; resolution happens before the sub-phase that depends on them).
- **GOTCHA**: Do NOT resolve the OQ in this commit. Pattern is strictly additive; resolution is a separate later commit.
- **VALIDATE**:

  ```bash
  grep -q "OQ-V1-JM-07" docs/brehon-law-inspired-network/99-decisions-and-open-questions.md && echo OK
  jq '.pending | map(select(.id | test("V1-JM-07"))) | length' .claude/decision-queue.json  # expect 1
  ```
- **COMMIT MESSAGE**: `docs(v1-JM-b): open OQ-V1-JM-07 — post-JM-b case-open severity inference (task 1)`

### Task 2: ADD `get_int_cascade` + `get_float_cascade` to `config.rs`

- **ACTION**: Add the two cascade helpers per §10.1 + §10.2 next to the existing `get_int` / `get_float` functions.
- **IMPLEMENT**: Both functions `pub`. Walk `<status>.<severity> → <severity> → <bare>` via repeated `fetch_value` calls; final fallback is `const_default_int` / `const_default_float` on the bare namespace.
- **MIRROR**: Existing `get_int` body (`config.rs:100-131`).
- **GOTCHA**: The cascade walks KEYS, not scopes. Scope cascade (community → instance) happens inside `fetch_value` and applies at every level of the key cascade.
- **GOTCHA**: Return `Result<i64, LemmyErrorType>` (matches the `LemmyResult<i64>` idiom — `LemmyResult` is an alias).
- **GOTCHA**: Cache the resolved value under the SPECIFIC candidate that hit — do NOT normalise keys or the cache loses O(1) lookup on repeat reads.
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/PRPs/debug/v1-JM-b-task2-check.log 2>&1"
  echo "exit: $?"   # expect 0
  tail -20 .claude/PRPs/debug/v1-JM-b-task2-check.log
  ```
- **COMMIT MESSAGE**: `feat(v1-JM-b): add get_int_cascade + get_float_cascade helpers in config.rs (task 2)`

### Task 3: ADD `panel_has_sponsor_majority_cluster` in `jury_common.rs`

- **ACTION**: Add one public-crate function per §10.7, immediately after the existing `shares_active_sponsor` function in the file.
- **IMPLEMENT**: One `sql_query` with a grouped self-join on `surety`; majority = `(len / 2) + 1`.
- **MIRROR**: `shares_active_sponsor` at `jury_common.rs:23-45` (raw SQL + `QueryableByName` + `bind::<Array<Integer>, _>`).
- **GOTCHA**: Return `Ok(false)` for `person_ids.len() < 2` — two-person panels cannot have a majority cluster by definition, and the SQL would compute `(1/2)+1 = 1` which `COUNT(DISTINCT sponsored_id) >= 1` is always true → false positive.
- **GOTCHA**: `Array<Integer>` NOT `Array<Int4>` — diesel sql_types naming (matches `run_strict_eligibility_query` at `admin_assign_jury.rs:300-338`).
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/PRPs/debug/v1-JM-b-task3-check.log 2>&1"
  echo "exit: $?"
  tail -20 .claude/PRPs/debug/v1-JM-b-task3-check.log
  ```
- **COMMIT MESSAGE**: `feat(v1-JM-b): add panel_has_sponsor_majority_cluster helper in jury_common.rs (task 3)`

### Task 4: REWRITE `select_eligible_jurors` + `run_extended_eligibility_query` in `admin_assign_jury.rs`

- **ACTION**: Rename `run_strict_eligibility_query` to `run_extended_eligibility_query` and extend with (a) an optional `cooldown_days` param that adds a NOT EXISTS subquery against `jury_assignment`, and (b) an optional `max_concurrent_per_juror_total` param that adds a NOT IN subquery counting all-communities active assignments. Rewrite `select_eligible_jurors` to use the 3-phase algorithm per §10.8 with the R1/R2/R3 relaxation cascade per §5.3. Add the `ConstraintRecord` struct and the `write_constraint_relaxation`, `geographic_diversity_score`, `sample_panel` helpers (sample_panel may re-use `run_extended_eligibility_query` with a fresh LIMIT-random call per attempt).
- **IMPLEMENT**: Per §10.7 + §10.8 + §10.9 + §10.10. Keep `legacy_select_eligible_jurors` untouched.
- **SIGNATURE CHANGE**: `pub(crate) async fn select_eligible_jurors(conn, case, panel_size, exclude_person_ids, cache) -> LemmyResult<(Vec<PersonId>, ConstraintRecord)>` — adds a `panel_size` parameter (the caller now computes it via cascade, not the callee) and returns the `ConstraintRecord` alongside the eligible vec.
- **MIRROR**: Existing `select_eligible_jurors` shape at `admin_assign_jury.rs:215-274` (transaction-aware `ConfigCache` use, excluded-ids build, small-pool fallback) + `run_strict_eligibility_query` at `:300-338` (raw SQL + sql_query + bind patterns).
- **GOTCHA**: `decline_jury_assignment.rs:147` calls `select_eligible_jurors`. Update THAT call site in the same commit to pass the new `panel_size` argument (read from the case's `panel_size_snapshot.unwrap_or(DEFAULT_JURY_PANEL_SIZE)` — because decline replacement operates on an already-seated panel).
- **GOTCHA**: `admin_emergency_remove.rs:170` also calls `select_eligible_jurors`. Update THAT call site to compute panel_size via cascade (same as `process_assignment`) — emergency-remove cases are Severe + Regular so they should get a 7-juror panel. Or, simpler, have emergency-remove pre-compute panel_size via cascade in the same way; a tiny amount of code duplication is acceptable for now and may factor later.
- **GOTCHA**: Keep the `legacy_select_eligible_jurors` fallback — but update it to accept a `panel_size: i64` parameter (it already does) and call it at the SAME two fallback points as v0 (pool < panel_size post-R1 AND fallback_allowed = true). The new fallback path should continue to return a `ConstraintRecord` with `relaxations_fired = ["legacy_fallback"]` or similar signal.
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-b-task4-check.log 2>&1"
  echo "exit: $?"   # expect 0
  tail -40 .claude/PRPs/debug/v1-JM-b-task4-check.log
  cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-b-task4-clippy.log 2>&1"
  echo "clippy exit: $?"   # expect 0
  tail -40 .claude/PRPs/debug/v1-JM-b-task4-clippy.log
  ```
- **COMMIT MESSAGE**: `feat(v1-JM-b): 3-phase diversity-aware select_eligible_jurors + R1/R2/R3 relaxation cascade (task 4)`

### Task 5: ADD `compute_status_tier` + severity/status slug helpers + REWRITE `process_assignment`

- **ACTION**: Add `compute_status_tier`, `status_tier_slug`, `severity_tier_slug` module-private helpers per §10.3. Rewrite `process_assignment` per §10.4 + §10.5 + §10.6 + §10.11 + §10.12.
- **IMPLEMENT**:
  1. Read case.severity_tier (trust backfill).
  2. Compute status_tier via `compute_status_tier` (lazy).
  3. Cascade read panel_size / quorum_fraction / threshold_fraction.
  4. Resolve to integer counts.
  5. Call `select_eligible_jurors(conn, &case, panel_size, None, &mut cache)` and unpack `(eligible, record)`.
  6. Snapshot onto moderation_case (including status_tier if compute changed it).
  7. Emit `severity_tier_frozen` governance_log entry.
  8. Insert JuryAssignment rows with `selected_under_constraints = Some(record.as_json())`.
  9. Keep the per-juror `jury_assigned` log entries (v0 behaviour).
  10. Emit extended `panel_assembled` payload with constraints_applied + relaxations.
  11. Return `AdminAssignJuryResponse { case_id, assigned_person_ids: eligible }` unchanged.
- **MIRROR**: Existing `process_assignment` at `admin_assign_jury.rs:91-193` (transaction shape, status guard, juror-pseudonym fetch, panel_assembled emission).
- **GOTCHA**: Use `SeverityTier::Severe` / `CaseStatusTier::Founder` etc. as **strongly-typed** constants, not strings. Slug helpers convert to snake_case strings only at the serde boundary (governance_log JSON payload) or at the cascade key-construction boundary.
- **GOTCHA**: `compute_status_tier` is async and borrows `conn`. Call it BEFORE any other `&mut *conn` borrow in the same scope. Hoist the call to the start of `process_assignment` after the case load.
- **GOTCHA**: The governance_log `severity_tier_frozen` emission attributes the `admin_pseudonym` (the admin who ran the assign). NOT `None` — the admin is the actor.
- **GOTCHA**: The extended `panel_assembled` payload must still pass the governance_log's `scrub_json` (it does — constraint names and slugs are not PII).
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-b-task5-check.log 2>&1"
  echo "exit: $?"
  tail -40 .claude/PRPs/debug/v1-JM-b-task5-check.log
  cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-b-task5-clippy.log 2>&1"
  echo "clippy exit: $?"
  tail -20 .claude/PRPs/debug/v1-JM-b-task5-clippy.log
  ```
- **COMMIT MESSAGE**: `feat(v1-JM-b): severity/status-aware process_assignment + snapshot writes + severity_tier_frozen emission (task 5)`

### Task 6: EDIT `admin_emergency_remove.rs` — severity_tier = Severe

- **ACTION**: Per §10.13. One-line change to the `ModerationCaseInsertForm` literal at `admin_emergency_remove.rs:143-157`.
- **IMPLEMENT**: Add `severity_tier: SeverityTier::Severe,` between `severity:` and `status:`.
- **MIRROR**: the existing literal.
- **GOTCHA**: Add `use lemmy_db_schema_file::enums::SeverityTier;` at the top of the file if not already present.
- **GOTCHA**: ADR-013 emergency-remove is the ONLY v1-JM-b case-open severity write. All other case-open paths (create_report, etc.) remain Minor per JM-a backfill default — the general severity-inference is OQ-V1-JM-07.
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-b-task6-check.log 2>&1"
  echo "exit: $?"
  tail -20 .claude/PRPs/debug/v1-JM-b-task6-check.log
  ```
- **COMMIT MESSAGE**: `feat(v1-JM-b): admin_emergency_remove sets severity_tier = Severe per ADR-013 (task 6)`

### Task 7: EXTEND `tests/e2e.rs` with cascade round-trip + 3 severity/status matrix tests

- **ACTION**: Add tests to the existing admin_assign_jury test module in `crates/server/tests/e2e.rs`:
  1. `admin_assign_jury_severity_tier_regular_minor_panel_5_jurors` — opens a regular (non-founder, non-provisional) target + Minor case → assert 5 jurors assigned, `moderation_case.panel_size_snapshot = 5`, `quorum_snapshot = 3`, `threshold_count_snapshot = 3`.
  2. `admin_assign_jury_severity_tier_regular_severe_panel_7_jurors` — regular target + Severe case → assert 7 jurors, `panel_size_snapshot = 7`, `quorum_snapshot = 5`, `threshold_count_snapshot = 6`.
  3. `admin_assign_jury_severity_tier_founder_severe_panel_9_jurors` — founder target (insert a `reputation_event` with `reason = 'founder_seed'`, unexpired) + Severe case → assert 9 jurors, `panel_size_snapshot = 9`, `quorum_snapshot = 7` (`ceil(9 * 0.71) = 7`), `threshold_count_snapshot = 7` (`ceil(9 * 0.75) = 7`).
  4. `admin_assign_jury_writes_selected_under_constraints_jsonb` — after a Minor/Regular assign, query `jury_assignment` for the case and assert every row has a non-null `selected_under_constraints` matching `{"no_majority_from_same_sponsor_cluster": "applied", "geographic_diversity_preferred": "applied_soft", "no_recent_juror_repeat": "applied", "no_same_endorsement_chain": "disabled"}`.
  5. `admin_assign_jury_emits_severity_tier_frozen_governance_log` — assert at least one `governance_log.entry_kind = 'severity_tier_frozen'` row exists post-assign with payload including `severity_tier = "minor"` and `status_tier = "regular"`.
  6. `config_get_int_cascade_resolves_founder_severe_to_bare_then_const` — seed no DB rows for `jury.panel_size.founder.severe` but seed `jury.panel_size.severe = 7` → assert cascade returns 7 from the per-severity key; then with no per-severity key, assert it falls to bare `jury.panel_size = 5` (JM-a seeded); then delete bare key entirely, assert it falls to const `DEFAULT_JURY_PANEL_SIZE`.
- **IMPLEMENT**: Mirror the existing fixture at `tests/e2e.rs:~900-990` (8 juror seeds, reporter, target, admin, community). For test 3, add a helper `seed_founder_target(conn, target_id)` that inserts the `reputation_event` row.
- **MIRROR**: Existing admin_assign_jury test fixture + `config_parity_round_trip` patterns at `e2e.rs:1357-1414`.
- **GOTCHA**: Seeding 9 jurors for the founder-severe test requires expanding the 8-juror v0 fixture. Pre-seed juror_i through juror_p if needed for the small-pool-relaxation tests in Task 8.
- **GOTCHA**: The `parity::check()` style tests for the cascade helper go in the e2e harness because they need a live DB; unit tests for the cascade walk logic (without DB) can live in `config.rs::tests` — but that's bonus. The minimum is the e2e test.
- **GOTCHA**: Use `LemmyResult<()>` with `?` per `.claude/memory/feedback_clippy_test_style.md` — no `unwrap` / `expect` / `allow_attributes`.
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_assign_jury > .claude/PRPs/debug/v1-JM-b-task7-tests.log 2>&1"
  echo "exit: $?"
  tail -40 .claude/PRPs/debug/v1-JM-b-task7-tests.log
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server config_get_int_cascade >> .claude/PRPs/debug/v1-JM-b-task7-tests.log 2>&1"
  echo "exit (cascade): $?"
  tail -20 .claude/PRPs/debug/v1-JM-b-task7-tests.log
  ```
- **COMMIT MESSAGE**: `test(v1-JM-b): 6 new e2e tests — 3 severity/status matrix + selected_under_constraints + severity_tier_frozen + cascade (task 7)`

### Task 8: EXTEND `tests/e2e.rs` with R1 relaxation test + admin_emergency_remove severity_tier assertion

- **ACTION**: Two more e2e tests:
  1. `admin_assign_jury_small_pool_triggers_R1_relaxation` — seed only 5 eligible jurors (not 10); set `juror_a` through `juror_c` as "recently served" (insert a `jury_assignment` row with `responded_at = now() - 1 day`, `status = Submitted`); attempt to assign a 5-juror panel → assert that the pool must lift cooldown (R1) to satisfy, assert a `jury_constraint_violation_log` row exists with `constraint_name = 'no_recent_juror_repeat'` + `reason_code = 'SmallPool'`, assert a `governance_log.entry_kind = 'jury_constraint_relaxed'` row exists, and assert the panel IS still seated (5 jurors).
  2. `admin_emergency_remove_case_has_severity_tier_severe` — call the emergency-remove handler; query the case row; assert `case.severity_tier == SeverityTier::Severe`.
- **IMPLEMENT**: Mirror the v0 admin_emergency_remove e2e test if one exists; else write a fresh fixture. For the cooldown-relaxation test, the fixture manipulates `jury_assignment.responded_at` directly.
- **MIRROR**: existing tests in `e2e.rs`.
- **GOTCHA**: The R1 test requires `jury.constraints.juror_cooldown_days` = default 7; verify the seed migration has this value. Setting `responded_at = now() - INTERVAL '1 day'` falls inside the 7-day window.
- **GOTCHA**: For the emergency-remove test, the handler is async and requires Docker for Postgres; do not run it under a skipped-DB fixture.
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_assign_jury_small_pool_triggers_R1_relaxation admin_emergency_remove_case_has_severity_tier_severe > .claude/PRPs/debug/v1-JM-b-task8-tests.log 2>&1"
  echo "exit: $?"
  tail -40 .claude/PRPs/debug/v1-JM-b-task8-tests.log
  ```
- **COMMIT MESSAGE**: `test(v1-JM-b): R1 relaxation assertion + admin_emergency_remove severity_tier assertion (task 8)`

### Task 9: FULL-WORKSPACE VALIDATION — `cargo check --workspace --features full` + `cargo clippy --workspace --features full -- -D warnings` + `cargo test --test e2e -p lemmy_server`

- **ACTION**: Run the full validation stack and confirm all pass. Capture logs.
- **IMPLEMENT**: sequence the three commands, stop on first failure.
- **MIRROR**: v1-JM-a Task 10 precedent.
- **GOTCHA**: Clippy's `--features full` + `--no-deps` per `.claude/rules/pre-phase-harness-audit.md` DoD footguns — or narrow to `-p lemmy_api` if upstream lints are noisy. The workspace-level command with `--no-deps` is the canonical form.
- **GOTCHA**: If any test run is flaky, re-run ONCE. Consistent failure is a real bug; transient failure likely = docker startup race (acceptable to retry once per `.claude/rules/pre-phase-harness-audit.md` retry guidance).
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-b-task9-check.log 2>&1"
  status=$?
  tail -40 .claude/PRPs/debug/v1-JM-b-task9-check.log
  echo "cargo check exit: $status"
  [ $status -eq 0 ] || exit $status

  cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-b-task9-clippy.log 2>&1"
  status=$?
  tail -40 .claude/PRPs/debug/v1-JM-b-task9-clippy.log
  echo "cargo clippy exit: $status"
  [ $status -eq 0 ] || exit $status

  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/PRPs/debug/v1-JM-b-task9-e2e.log 2>&1"
  status=$?
  tail -60 .claude/PRPs/debug/v1-JM-b-task9-e2e.log
  echo "cargo test e2e exit: $status"
  [ $status -eq 0 ] || exit $status
  ```
- **COMMIT MESSAGE**: `chore(v1-JM-b): full-workspace validation pass — check + clippy + e2e (task 9)`

  Note: this commit amends only the validation log files in `.claude/PRPs/debug/`; no production source changes.

### Task 10: WRITE `v1-JM-b-retro.md` then `gh pr create`

- **ACTION**: Write phase retrospective covering Tasks 1–9, capturing surprises + carry-forward items. Then open PR against `governance-v0`.
- **IMPLEMENT**: Mirror `.claude/PRPs/reports/v1-JM-a-retro.md` structure: (1) what shipped, (2) what surprised, (3) what we carry forward, (4) metrics (LoC, test count, relaxation-cascade firing count in test harness).
- **MIRROR**: existing retro files under `.claude/PRPs/reports/`.
- **GOTCHA**: Per `.claude/rules/gh-pr-fork-target.md`, always pass `--repo barrie-cork/lemmy` to `gh pr create`. Default target is `governance-v0`.
- **GOTCHA**: PR must NOT be a draft — CodeRabbit skips drafts per `.claude/rules/phase-branch.md`.
- **GOTCHA**: PR title ≤ 70 chars. Suggested: `Phase v1-JM-b — severity/status-aware admin_assign_jury + diversity constraints`.
- **VALIDATE**:

  ```bash
  git log --oneline phase-v1-JM-b ^governance-v0   # expect ~9 commits (task 1..9)
  gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-JM-b \
    --title "Phase v1-JM-b — severity/status-aware admin_assign_jury + diversity constraints" \
    --body "$(cat <<'EOF'
## Summary
- Severity/status-tier-aware panel sizing via `get_int_cascade` / `get_float_cascade`
- 3-phase diversity algorithm with R1/R2/R3 relaxation cascade
- `selected_under_constraints` JSONB + `severity_tier_frozen` governance_log
- `admin_emergency_remove` severity_tier = Severe per ADR-013

## Completion report
.claude/PRPs/reports/v1-JM-b-retro.md

## Plan reference
.claude/PRPs/plans/v1-jury-mechanics-b.plan.md

## OQ opened
OQ-V1-JM-07 (post-JM-b case-open severity inference; blocks v1.5 general-severity-inference sub-phase only)
EOF
)"
  ```
- **COMMIT MESSAGE**: `docs(v1-JM-b): phase retrospective before PR open (task 10)`

---

## 14. Testing strategy

Per `.claude/PRPs/plans/completed/phase-0-test-harness.plan.md` + IMPLEMENTATION-PLAN-v0.md §5: **integration-only** in v0/v1. Every JM-b test lands in `crates/server/tests/e2e.rs`. Unit tests inside `config.rs::tests` / `admin_assign_jury::tests` are **bonus** but not mandatory.

### 14.1 Test inventory

| Test | Task | Validates |
|---|---|---|
| `admin_assign_jury_severity_tier_regular_minor_panel_5_jurors` | 7 | Minor/Regular → panel_size 5 + quorum 3 + threshold_count 3 snapshotted |
| `admin_assign_jury_severity_tier_regular_severe_panel_7_jurors` | 7 | Severe/Regular → panel_size 7 + quorum 5 (ceil 7*0.71) + threshold_count 6 (ceil 7*0.75) snapshotted |
| `admin_assign_jury_severity_tier_founder_severe_panel_9_jurors` | 7 | Founder/Severe → panel_size 9 + quorum 7 (ceil 9*0.71) + threshold_count 7 (ceil 9*0.75) snapshotted. REQUIRES founder seed fixture. |
| `admin_assign_jury_writes_selected_under_constraints_jsonb` | 7 | Every `jury_assignment` row has a non-null JSONB payload with 4 constraint keys |
| `admin_assign_jury_emits_severity_tier_frozen_governance_log` | 7 | `governance_log` contains at least one `severity_tier_frozen` row |
| `config_get_int_cascade_resolves_founder_severe_to_bare_then_const` | 7 | Cascade walks `<status>.<severity> → <severity> → <bare> → const` as expected |
| `admin_assign_jury_small_pool_triggers_R1_relaxation` | 8 | Under-sized pool triggers cooldown lift + emits `jury_constraint_relaxed` + writes `jury_constraint_violation_log` + still seats panel |
| `admin_emergency_remove_case_has_severity_tier_severe` | 8 | Emergency-remove sets severity_tier = Severe |

### 14.2 Edge cases (covered in existing + new tests)

- [ ] v0 golden-path test unchanged — Minor/Regular case still gets 5 jurors. (Regression check: existing `assign_resp.assigned_person_ids.len() == 5` at `e2e.rs:984` still passes.)
- [ ] `admin_emergency_remove` post-facto jury still works (existing test unchanged; now gets a Severe-tier panel automatically).
- [ ] `decline_jury_assignment` replacement still works with the new `select_eligible_jurors(conn, case, panel_size, Some(&exclude_ids), cache)` signature.
- [ ] `jury_assignment.role` stays `'Original'` for every JM-b writer — confirmed by "`SELECT DISTINCT role FROM jury_assignment WHERE case_id IN (...)` → only `Original`" assertion in existing test.
- [ ] Hash-chain integrity holds post-panel_assembled + severity_tier_frozen emissions (covered by existing governance_log_hash_chain_holds test at `e2e.rs`).
- [ ] No person_id / username leaks in `jury_constraint_violation_log.relaxation_metadata` — assert that the JSONB payload does NOT contain any `person_id` keys (only `dropped_constraint_name`, `phase`).
- [ ] `parity::seeded_keys_count_matches_const_count` test still passes (JM-b adds zero seeded keys; EXPECTED_SEED_COUNT + V1_AD + V1_JM = 34 + 27 + 27 = 88 unchanged).

---

## 15. Validation commands (DoD)

Every validation command below must exit 0. Use the wrapper scripts per `.claude/rules/cargo-output-capture.md`.

### 15.1 STATIC_ANALYSIS

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-b-dod-check.log 2>&1"
status=$?
tail -40 .claude/PRPs/debug/v1-JM-b-dod-check.log
echo "check exit: $status"
[ $status -eq 0 ] || exit $status

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-b-dod-clippy.log 2>&1"
status=$?
tail -40 .claude/PRPs/debug/v1-JM-b-dod-clippy.log
echo "clippy exit: $status"
[ $status -eq 0 ] || exit $status
```

### 15.2 INTEGRATION_TESTS

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/PRPs/debug/v1-JM-b-dod-e2e.log 2>&1"
status=$?
tail -60 .claude/PRPs/debug/v1-JM-b-dod-e2e.log
echo "e2e exit: $status"
[ $status -eq 0 ] || exit $status
```

### 15.3 FULL_BUILD

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full --tests > .claude/PRPs/debug/v1-JM-b-dod-build.log 2>&1"
status=$?
tail -30 .claude/PRPs/debug/v1-JM-b-dod-build.log
echo "build exit: $status"
[ $status -eq 0 ] || exit $status
```

### 15.4 MIGRATION_VALIDATION

Not applicable — JM-b adds no migrations.

### 15.5 CROSS_CUTTING_VERIFICATION

- [ ] `governance_log::append` call-site count increased by ≥ 1 (severity_tier_frozen) + N relaxations.
- [ ] `actor_pseudonym_helper::get_or_create` still called once per juror in `process_assignment` (v0 behaviour preserved).
- [ ] No raw `person_id` or `username` string pushed into any governance_log payload, `selected_under_constraints` JSONB, or `jury_constraint_violation_log.relaxation_metadata`. Verify with grep `grep -rn "person_id" crates/api/api/src/governance/admin_assign_jury.rs` — all references should be to bound variables, NOT string literals, NOT JSON keys in payload.
- [ ] Hash-chain integrity test (`governance_log_hash_chain_holds` in e2e.rs) passes.
- [ ] Seven PM plugin hooks still present per `.claude/rules/pm-plugin-hooks-stable.md` (Task 0 grep check re-run in Task 9).

### 15.6 MANUAL_VALIDATION (optional — local instance)

With a docker-compose governance instance running:

```bash
# 1. Open a regular/minor case via create_report
curl -X POST http://localhost:8536/api/v4/governance/report \
  -H "Authorization: Bearer $JWT" \
  -d '{"target_post_id": 1, "reason_code": "off_topic"}'

# 2. Run admin_assign_jury
curl -X POST http://localhost:8536/api/v4/governance/admin/assign-jury \
  -H "Authorization: Bearer $ADMIN_JWT" \
  -d '{"case_id": 1}'

# 3. Read back the case + assignments
psql -h localhost -U lemmy -d lemmy -c "
  SELECT id, severity_tier, status_tier, panel_size_snapshot, quorum_snapshot, threshold_count_snapshot
  FROM moderation_case WHERE id = 1"
# Expect: Minor, Regular, 5, 3, 3

psql -h localhost -U lemmy -d lemmy -c "
  SELECT person_id, status, role, selected_under_constraints
  FROM jury_assignment WHERE case_id = 1"
# Expect: 5 rows, role = 'Original', JSONB populated

psql -h localhost -U lemmy -d lemmy -c "
  SELECT entry_kind, payload FROM governance_log
  WHERE entry_kind = 'severity_tier_frozen' ORDER BY id DESC LIMIT 1"
# Expect: 1 row with the 5-field payload
```

---

## 16. Acceptance criteria

- [ ] All 10 tasks committed in order on `phase-v1-JM-b`
- [ ] `cargo check --workspace --features full` exit 0
- [ ] `cargo clippy --workspace --features full --no-deps -- -D warnings` exit 0
- [ ] `cargo test --test e2e -p lemmy_server` exit 0 (includes all 8 new tests)
- [ ] No new migration files
- [ ] No new config keys seeded (count unchanged at 88)
- [ ] `admin_assign_jury` writes all three snapshot columns on every new assign
- [ ] `severity_tier_frozen` governance_log entry emitted on every new assign
- [ ] `jury_constraint_relaxed` governance_log entry emitted when R1/R2/R3 fires (verified by the R1 test)
- [ ] `jury_constraint_violation_log` row written when R1/R2/R3 fires
- [ ] `selected_under_constraints` JSONB populated on every new `jury_assignment` row written by admin_assign_jury
- [ ] `admin_emergency_remove` sets `severity_tier = Severe`
- [ ] `get_int_cascade` / `get_float_cascade` public + tested
- [ ] `panel_has_sponsor_majority_cluster` public-crate + covered by the cluster test (when added in a follow-on sub-phase; JM-b's R2/R3 cascade tests are via the algorithm-level test, not direct unit test)
- [ ] OQ-V1-JM-07 opened in 99-decisions + decision-queue.json
- [ ] `v1-JM-b-retro.md` written
- [ ] PR opened as non-draft, targets `governance-v0`, title ≤ 70 chars, uses `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review landed (at least one pass) — blocking Critical findings fixed per `.claude/rules/phase-branch.md` / `feedback_coderabbit_block_merge_critical.md`
- [ ] No contradictions with the 15 ADRs in 99-decisions
- [ ] Hash-chain integrity test still passes
- [ ] Seven PM hooks still present (regression check)

---

## 17. Completion checklist

- [ ] Task 0 pre-flight probes exit codes correct
- [ ] Task 1 OQ-V1-JM-07 stub committed
- [ ] Task 2 cascade helpers committed
- [ ] Task 3 panel_has_sponsor_majority_cluster committed
- [ ] Task 4 select_eligible_jurors rewrite committed (callers in `decline_jury_assignment`, `admin_emergency_remove` updated)
- [ ] Task 5 process_assignment rewrite + compute_status_tier + severity_tier_frozen emission committed
- [ ] Task 6 admin_emergency_remove severity_tier = Severe committed
- [ ] Task 7 e2e tests 1–6 committed and passing
- [ ] Task 8 e2e tests 7–8 committed and passing
- [ ] Task 9 full-workspace validation pass committed
- [ ] Task 10 retrospective + PR opened

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `select_eligible_jurors` signature change breaks `decline_jury_assignment` / `admin_emergency_remove` callers (Task 4 introduces panel_size param) | MED | MED | Task 4 explicitly updates both callers in the same commit; Task 9 full-workspace check catches miss. |
| Cascade helper returns wrong value on partial seed (e.g. `jury.panel_size.founder.minor` missing but `jury.panel_size.minor` seeded) | MED | MED | Cascade `<status>.<severity> → <severity> → <bare> → const` is explicitly ordered; Task 7 test `config_get_int_cascade_resolves_founder_severe_to_bare_then_const` covers the walk. |
| `compute_status_tier` misclassifies a provisional member as Regular (due to `MembershipState` variant name mismatch post-JM-a merge) | MED | MED | Task 5 GOTCHA: read `enums.rs:624-648` post-JM-a-merge and confirm variant name before writing `matches!(MembershipState::Provisional)`. |
| Panel-level cluster query misses cross-community sponsors (false negative) | LOW | HIGH | `surety` table is flat (no community scope on cluster check per Phase 5 design); `sql_query` ANY($1) covers all panel members' sponsors instance-wide. |
| R1/R2/R3 cascade writes orphan `jury_constraint_violation_log` rows if panel assembly ultimately fails | LOW | MED | Every write is inside `run_transaction`; R3 exhausted + `fallback_allowed = false` path returns an undersized pool WITHOUT flipping case to JurySelection — the violation_log rows still persist as a legitimate audit of the attempt. This is intentional per PRD §12.2. |
| Geographic diversity stub (§10.9) returns 0.0 for every panel on a bootstrapping instance → soft bias becomes no-op | HIGH | LOW | Acceptable for v1-JM-b per PRD §OQ-V1-JM-02 lean (full heuristic deferred to v1.5). Document in retro. |
| Upstream Lemmy 1.0-beta weekly rebase touches `admin_assign_jury.rs` or `jury_common.rs` | LOW | MED | JM-b is governance-only code; conflicts with upstream near-zero. Weekly rebase guard in CLAUDE.md covers. |
| CodeRabbit flags Critical on R3 warn-level logging (information leak concern) | LOW | LOW | `warn!` message doesn't include any PII (only case_id + constraint name); CR pattern per `feedback_coderabbit_block_merge_critical.md` handles fix-in-same-PR. |
| Test assertion on `ceil(7 * 0.71) = 5` fails due to floating-point representation (e.g. `7 * 0.71 = 4.9700000000000006` vs `4.97`) → `ceil` still 5 but edge cases | LOW | LOW | Explicit test values pinned; PRD §10 has the matching canonical values. |
| Concurrent `admin_assign_jury` calls on the same case (two admins) | LOW | MED | v0 protection: `run_transaction` + row-level `ModerationCase::as_select().first()` doesn't hold a FOR UPDATE lock. JM-b does not regress this. Document as a known edge case (case-status guard catches the 2nd caller). |

---

## 19. Notes

- PRD §9.1 (the 9-step `submit_jury_vote` combined handler) is **JM-c scope**. JM-b writes `quorum_snapshot` + `threshold_count_snapshot` onto `moderation_case`; JM-c reads those snapshots at vote-tally. JM-c also writes `appeal_window_expires_at` at case-decision time. The snapshot-column plumbing is JM-a's; JM-b is the first writer of the admin_assign_jury-time snapshots; JM-c is the first reader at vote-tally time.
- PRD §5.3 Phase 3 "soft geographic score" is intentionally a **no-op stub** in v1-JM-b on a bootstrapping instance (every juror has 0 `community_id` distinct count). Full heuristic per OQ-V1-JM-02 lean is v1.5. This keeps JM-b within scope.
- PRD §OQ-V1-JM-01 (severity-tier inference at case-open) is OQ-V1-JM-07 in this plan's new-OQ framing — explicitly deferred.
- The extended `panel_assembled` payload is a superset of the v0 payload (adds fields, doesn't rename). Downstream consumers (modlog view at JM-d / federation-outbound at FI) that read `{case_id, juror_count}` continue to work; new fields are read-optional.
- The `ConstraintRecord::as_json()` helper referenced in Task 5 can be a simple `impl ConstraintRecord { fn to_json(&self) -> serde_json::Value { json!({...}) } }` inline; no `serde::Serialize` derive needed because we're only ever going one-way (Rust → JSON).
- `admin_emergency_remove`'s post-facto jury pick (via `admin_assign_jury::select_eligible_jurors`) automatically picks up the Severe-tier sizing once `severity_tier = Severe` is set on the case. The post-facto jury's `process_assignment`-equivalent code path in `admin_emergency_remove.rs:169-190` may need a similar panel-size cascade read for symmetry — include in Task 6 scope or flag as a follow-up.

---

## 20. Sub-phase stubs (v1-JM-c/d/e TOC only)

Per the v1-AD / v1-JM-a precedent: each sub-phase is its own plan file, owns its own PR → `governance-v0`, earns its own CodeRabbit review, and only gets written after the preceding sub-phase merges.

### v1-JM-c — `submit_jury_vote` 9-step handler rewrite

- Read `quorum_snapshot` + `threshold_count_snapshot` from `moderation_case` (v1-JM-b writes these; v1-JM-c is the first reader at vote-tally time)
- Replace hardcoded `QUORUM = 3` + `APPEAL_WINDOW_DAYS = 7` consts with snapshot reads + config read
- Deadlock path: `all_jurors_voted AND no_decision_met_threshold` → flip `CaseStatus::AdminReview` + emit `jury_deadlock` governance_log
- Sponsor-liability branch integration (compute/fire split per sponsor-liability-v1 §9.1 + §9.3): on sanctioned decision with active sponsor(s), flip to `SponsorLiabilityPending` + set `grace_expires_at` + defer `public_case_log` + juror reputation events to scheduler
- No-sponsor / NoAction path: preserve v0 `public_case_log` + `reputation_event` writes at vote-tally
- Write `appeal_window_expires_at = decided_at + appeal.window_days` on BOTH paths (step 9)
- ~8 tasks, blocks JM-d + sponsor-liability-v1-d

### v1-JM-d — Appeals (bounded window + reporter-rights + auto re-jury + background job)

- `request_appeal` rewrite: check `appeal_window_expires_at > now()` instead of `closed_at IS NULL`
- Reporter-rights extension: allow reporter to appeal iff `winning_decision IN (NoAction, AdvisoryLabel)`
- Auto re-jury: when `appeal.auto_select_on_appeal_acceptance = true`, same-transaction call to new `select_appeal_panel` helper (re-uses `select_eligible_jurors` with `exclude_person_ids = original_jurors`, `role = Appeal`, next-tier threshold)
- New handler `admin_trigger_appeal_rejury` for the `false` config path
- New background job in `crates/server/src/governance.rs` that finds cases with `appeal_window_expires_at < now() AND status = Decided` and flips them to `Closed` + emits `appeal_window_expired`
- ~7 tasks, blocks JM-e

### v1-JM-e — Capstone test + cross-sub-phase integration assertions

- `v0_case_completes_under_v0_rules_after_v1_config_flip` integration test per PRD §11
- Cross-sub-phase integration tests: full case lifecycle from report → assign (JM-b) → vote (JM-c) → appeal (JM-d) → re-jury (JM-d) → decision → close
- Audit log invariant test: every case's `governance_log` sequence matches the expected state-transition shape
- ~3-4 tasks, closes the JM PRD.
