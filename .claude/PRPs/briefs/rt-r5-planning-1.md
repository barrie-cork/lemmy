# RT-r5 planning brief

**Written**: 2026-05-31 by advisor session (lane worktree `C:\Users\barri\Developer\brehon-fork-rt-r5` on `phase-v1-RT-r5`, Mode A) via `/auto-roadmap` plan-gap handler.

**Subagent target**: `planning` (Opus 4.8, color purple — see `.claude/agents/planning.md`).

**Worktree**: Junior cuts `junior/rt-r5-planning-1` from `phase-v1-RT-r5` per concurrency-1 default. The plan file commits and pushes to `phase-v1-RT-r5` at finalize.

**Authority anchor**: `v1-reputation-tuning.prd.md` §11 row 5 (`v1.r5 — instance-wide rollup`) + §5.5 (instance-wide reputation rollup — read verbatim) + §7 (Cross-Cutting Impact — `ENTRY_KIND_ROLLUP_RECOMPUTED`) + §10 (Security — cron signs into governance_log via instance key; `system` pseudonym for cron-batch entries). PRD §11 row 5 dependency: **v1.r2 (decay must be live for rollup)** — verified: RT-r2 shipped (PR #150, merge `3b36b4e61`); the v1 decay calculator is live behind `feature.reputation_v1_decay_enabled`. Dependency satisfied.

**Lane mode**: A (dedicated lane worktree on the laptop). Briefs author directly on the phase branch.

---

## 0. Dependency-reality check (advisor-verified 2026-05-31, before brief author)

Per `feedback_runbook_audit_drift_post_event_check.md` + `feedback_handover_assumptions_need_empirical_verification.md` — every claim below was grep-verified against the `phase-v1-RT-r5` tree:

| Dependency | State on `phase-v1-RT-r5` | Evidence |
|---|---|---|
| `ENTRY_KIND_ROLLUP_RECOMPUTED` const | ✅ EXISTS (pre-landed RT-r1) | `crates/db_schema/src/source/governance/governance_log.rs:215` (`= "rollup_recomputed"`); re-exported in api shim `crates/api/api/src/governance/governance_log.rs:59` |
| Registry row naming RT-r5 as fire site | ✅ PRESENT | `.claude/rules/governance-log-entry-kind-registry.md:188` — `v1-RT-r5 scheduled_tasks.rs::reputation_rollup_cron (pending)` |
| `reputation_snapshot` supports `community_id IS NULL` | ✅ EXISTS | `crates/db_schema/src/source/governance/reputation_snapshot.rs` has `community_id: Option<CommunityId>`; v0 only writes `Some` rows but schema supports `None` per PRD §5.5 |
| `recompute_snapshot` function | ✅ EXISTS | `crates/api/api/src/governance/reputation_snapshot.rs:222` — `pub async fn recompute_snapshot(conn, person_id, community_id, cache)` |
| `upsert_snapshot` function | ✅ EXISTS | `reputation_snapshot.rs:770` — handles INSERT or UPDATE for a `(person_id, community_id)` pair |
| Config knob `job.rollup_interval_days` | ✅ SEEDED | `config.rs:1027` default `7`, metadata at `:3895` |
| Config knob `job.rollup_equal_weights` | ✅ SEEDED | `config.rs:1030` default `true`, metadata at `:3910` |
| Participation cron (RT-r3) shipped | ✅ SHIPPED | PR #155 merged. `scheduled_tasks.rs` has `PARTICIPATION_CRON_RUNNING` AtomicBool + `ParticipationCronRunningGuard` at lines 105-114 |
| Cron concurrency-guard pattern | ✅ 5 existing guards | `RunningGuard`, `AppealWindowExpiryRunningGuard`, `GraceCheckRunningGuard`, `FedReplayCleanupRunningGuard`, `ParticipationCronRunningGuard` — RT-r5 adds a 6th |
| v1 decay calculator (RT-r2) shipped | ✅ SHIPPED | PR #150 merged. `feature.reputation_v1_decay_enabled` flag at `config.rs:1032` |
| `GET /admin/reputation/rollup` endpoint | ❌ DOES NOT EXIST | this is what RT-r5 BUILDS |
| `reputation_rollup_cron` function | ❌ DOES NOT EXIST | this is what RT-r5 BUILDS |

**Load-bearing consequence**: RT-r5 needs **NO new migration** for the rollup rows (the `reputation_snapshot` table already supports `community_id IS NULL`) or the entry-kind const (shipped in RT-r1). The const is **declared-but-not-yet-emitted**; RT-r5 is its first emission site. This flips it from registry `(pending)` to live.

---

## 1. Role + dispatch line

`[role:planning] v1-RT-r5 plan — reputation_rollup_cron + GET /admin/reputation/rollup endpoint`

The actual create-task description is a single line under 100 chars per `.claude/rules/advisor-orchestrator.md` Junior task description template:

```
[role:planning] v1-RT-r5 plan — see .claude/PRPs/briefs/rt-r5-planning-1.md
```

---

## 2. Scope — what to produce

Produce **one plan file** at `.claude/PRPs/plans/v1-RT-r5.plan.md` for sub-phase **v1-RT-r5**. The plan covers PRD §11 row 5 in full: weekly instance-wide reputation rollup cron + admin rollup query endpoint + governance_log emit + e2e coverage.

### 2.1 Concrete deliverables (per PRD §5.5 + §7 + §10)

a. **Weekly rollup cron in `crates/routes/src/utils/scheduled_tasks.rs`** (per PRD §5.5). A new `reputation_rollup_cron` function registered with clokwerk, mirroring the existing participation-cron pattern:
   - **Concurrency guard**: new `ROLLUP_CRON_RUNNING: AtomicBool` + `RollupCronRunningGuard` struct with `Drop` impl, mirroring `PARTICIPATION_CRON_RUNNING` / `ParticipationCronRunningGuard` at lines 105-114.
   - **Cadence**: weekly, configurable via `job.rollup_interval_days` (default `7`, already seeded). Runs **after** the participation cron in the same scheduler tick per PRD §5.2.
   - **Logic** (per PRD §5.5): for each person with at least one per-community `reputation_snapshot` row, compute instance-wide rollup:
     ```
     rollup.dimension = sum(per_community_snapshot.dimension * weight) / sum(weight)
     ```
     Default weight = 1 per community (`job.rollup_equal_weights = true`). Per OQ-V1-02 lean, communities the person is banned from (active sanction with `target_community_id` set, scope `Community`) are **excluded from the denominator entirely**.
   - **Write**: upsert via `recompute_snapshot(conn, person_id, None, cache)` — the existing function handles `community_id = None` rows via the same `upsert_snapshot` path. **OR** if the rollup logic differs enough from event-sourced recompute (it's a weighted average of snapshots, not an event sum), the planner may introduce a dedicated `upsert_rollup_snapshot` helper that writes the pre-computed dimension values directly. Planner reads `recompute_snapshot` at `:222-375` to decide which path fits.
   - **Governance log**: each person's rollup emit calls `governance_log::append` with `ENTRY_KIND_ROLLUP_RECOMPUTED`, payload per registry: `{person_id, contributing_community_count, rollup_dimensions: {<dim>: <int>, ...}, recomputed_at}`. Uses `system` pseudonym (not admin — cron-driven per PRD §10).
   - **Capability-flip detection** fires on rollup-row changes too (PRD §5.5). Instance-wide capability-flips emit `ENTRY_KIND_CAPABILITY_CHANGED` with `snapshot_community_id: null`.

b. **Admin rollup query endpoint: `GET /api/v4/governance/admin/reputation/rollup`** (per PRD §5.5 use case 1). New handler in a new file `crates/api/api/src/governance/admin_reputation_rollup.rs`:
   - Instance-admin only (same admin-gate as `admin_reputation_stats.rs`).
   - Input: `person_id: PersonId` (query parameter or JSON body — planner checks the sibling pattern at `admin_reputation_stats.rs`).
   - Returns: the rollup row (`reputation_snapshot WHERE person_id = $1 AND community_id IS NULL`) plus the per-community contributing snapshots (the set of `reputation_snapshot WHERE person_id = $1 AND community_id IS NOT NULL`).
   - DTO: `AdminReputationRollup` request + `AdminReputationRollupResponse` response in `crates/api/api_common/src/governance.rs`.
   - Route registration in `crates/api/routes/src/lib.rs` admin scope.

c. **Registry-row update** in `.claude/rules/governance-log-entry-kind-registry.md:188`: drop `(pending)` marker; replace the TBD handler name with the real `reputation_rollup_cron` function path.

d. **e2e coverage in `crates/server/tests/e2e.rs`** — at minimum:
   - (i) rollup cron produces an instance-wide snapshot row after per-community data exists;
   - (ii) banned-community exclusion (per OQ-V1-02 lean);
   - (iii) `GET /admin/reputation/rollup` returns the rollup + contributing snapshots; rejects non-admin callers;
   - (iv) governance_log contains `ENTRY_KIND_ROLLUP_RECOMPUTED` entry after cron.

### 2.2 Scope boundary — what is NOT in RT-r5

Per PRD §11 phase table:

- **RT-r1:** Schema + consts + foundation tables. SHIPPED. RT-r5 does NOT add schema or consts.
- **RT-r2:** Per-dimension chained-halving decay calculator + `DECAY_KNOB_CHANGED` emit. SHIPPED. RT-r5 does NOT touch `compute_applied_delta`.
- **RT-r3:** Multi-source participation events (crons + vote-outcome + evidence-quality emitters). SHIPPED. RT-r5 does NOT touch the participation cron.
- **RT-r4:** Sponsor-gate strategies + allowlist admin endpoints. SHIPPED. RT-r5 does NOT touch `create_endorsement.rs` or `admin_sponsor_allowlist.rs`.
- **RT-r6:** Carry-forward CodeRabbit fixes (#19-#22, #31). NOT in RT-r5.

**Hard out-of-scope:**
- **Per-community weighted rollup** — `job.rollup_equal_weights` defaults `true` (equal weights). Weighted-average mode is v1+ stretch; this sub-phase implements equal weights only.
- **Cross-instance reputation portability** — v2/v3 per PRD §7 federation impact.
- **Admin override of reputation events** — v2 per PRD §10.
- **New migration** — the `reputation_snapshot` table already supports `community_id IS NULL` rows. The rollup cron writes to the existing table shape.

### 2.3 Plan file deliverable shape

Plan file at `.claude/PRPs/plans/v1-RT-r5.plan.md` follows `.claude/PRPs/templates/plan.template.md` verbatim. Required sections per §2.4 of the RT-r4 brief (identical requirements apply).

---

## 3. Required reading (in order, before drafting any plan section)

### 3.1 PRD anchors (load first)

1. `.claude/PRPs/prds/v1-reputation-tuning.prd.md` — full PRD. Most load-bearing for r5: **§5.5** (instance-wide reputation rollup — read verbatim), §7 (the `ENTRY_KIND_ROLLUP_RECOMPUTED` + payload), §10 (Security — cron signs via instance key; `system` pseudonym), §11 row 5 (phase definition + dependency = v1.r2 decay live).

2. `.claude/rules/governance-log-entry-kind-registry.md:188` + the pre-landed-const exemption at `:237` — the exact payload shape the rollup emit must produce.

### 3.2 ADR + open-question anchors

3. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` **OQ-001** (instance-wide vs per-community reputation — resolved by this sub-phase; rollup as derived snapshot in `reputation_snapshot WHERE community_id IS NULL`).

4. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` **OQ-V1-02** (banned-from-community rollup behaviour — lean: exclude from denominator entirely).

5. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` **ADR-008** (governance_log append-only — every rollup recomputation emits a governance_log entry).

6. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` **ADR-015** (pseudonymised actor IDs — cron uses `system` pseudonym, rollup writes log under the rolled-up person's pseudonym).

### 3.3 Mandatory file-class lessons (per advisor-orchestrator.md §2.4 file-class table)

RT-r5 IMPLEMENT file list (per §2.1 deliverables):
- `crates/routes/src/utils/scheduled_tasks.rs` (modify — rollup cron block)
- `crates/api/api/src/governance/admin_reputation_rollup.rs` (CREATE — new handler)
- `crates/api/api/src/governance/mod.rs` (modify — module declaration)
- `crates/api/api_common/src/governance.rs` (modify — 2 new DTOs)
- `crates/api/routes/src/lib.rs` (modify — use-import + route registration)
- `crates/api/api/src/governance/reputation_snapshot.rs` (modify — possibly, if rollup logic needs a helper; planner decides after reading)
- `.claude/rules/governance-log-entry-kind-registry.md` (modify — drop `(pending)`)
- `crates/server/tests/e2e.rs` (modify — rollup cron + endpoint tests)

Walk against the §2.4 file-class table → these rows fire:

| Pattern matched | Mandatory lesson(s) for §3 Required reading |
|---|---|
| `crates/server/tests/e2e.rs` (any edit) | `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`. Mirror the sibling fixtures module error-shape (Case A or B) verbatim. |
| `crates/server/tests/e2e.rs` (≥2 edits in this task or cohort) | `feedback_fix_impl_pre_locate_e2e_anchors.md` (pre-locate verbatim anchors) |
| Any `#[cfg(feature = "full")]` gate | `feedback_features_full_workspace_only.md`, `feedback_features_full_p_crate_incompatible.md` |

**Note**: the rollup cron handler does NOT do 2+ DB writes in a single handler request context (it's a cron batch, not a request handler), so `feedback_multi_write_handlers_need_transactions.md` does NOT fire for the cron. It MAY fire for the admin rollup endpoint if it does reads + log — planner checks.

### 3.4 Plan-template + canonical-sibling reads

7. `.claude/PRPs/templates/plan.template.md` — section structure verbatim.

8. **Canonical sibling plan** — RT-r3 added the participation cron (also a `scheduled_tasks.rs` cron block with `AtomicBool` + `RunningGuard` + governance_log emit). Its plan `.claude/PRPs/plans/v1-RT-r3.plan.md` is the closest structural sibling for "add a cron block to scheduled_tasks.rs." Read its §13 + §15 + §16a.

9. **Canonical sibling cron code** — the participation-cron block in `crates/routes/src/utils/scheduled_tasks.rs` (the `PARTICIPATION_CRON_RUNNING` guard + the cron body). This is the verbatim pattern the rollup cron mirrors.

10. **Canonical sibling for the admin endpoint** — `crates/api/api/src/governance/admin_reputation_stats.rs` (admin-gated read endpoint returning reputation data — the closest structural sibling for a read-only admin reputation query).

### 3.5 PMD-promoted patterns (load on demand)

- `pattern_cargo_feature_flag_propagation.md` — `--features full` discipline; `--workspace` over `-p`.
- `pattern_spec_schema_co_commit.md` — DTO + handler + route + e2e land in coherent task grouping.
- `pattern_context_is_finite.md` — plan body likely 500-700 lines; cite PRD by anchor.

---

## 4. Constraints (hard rules — violating any is a process breach)

### 4.1 Plan-content discipline

a. **PRD-aligned narrow scope only.** RT-r5 = rollup cron + admin rollup endpoint + `ROLLUP_RECOMPUTED` emit + e2e. No temptation to bundle RT-r6's carry-forward fixes or retroactively touch RT-r2/r3/r4 code.

b. **No new migration.** The `reputation_snapshot` table already supports `community_id IS NULL` rows (§0 verified). Plan §13 must NOT include a `migrations/**` task. If the planner believes a migration IS needed (e.g. a missing index on `community_id IS NULL`), file a `kind: "blocker"` planner DQ.

c. **PRD §5.5 + OQ-001 + OQ-V1-02 are the contract.** Instance-wide rollup, equal weights, banned-community exclusion, weekly cadence, materialised. Drift (e.g. planning real-time aggregation instead of materialised cron) is a process miss.

d. **Pseudonym discipline (ADR-015).** Cron-batch entries use the `system` pseudonym. Rollup writes log under the rolled-up person's pseudonym. Plan §13 must name the pseudonym source in the cron task.

e. **Cron ordering (PRD §5.2 + §5.5).** Rollup cron runs **after** the participation cron in the scheduler setup. The plan must specify the registration order in `scheduled_tasks.rs`.

### 4.2 Decision-queue discipline

Same as RT-r4 brief §4.2 (planner writes `kind: "blocker"` from `from: "planner"`, may pre-seed, never writes `answered_by: "advisor"` or `"user"`, mid-task push via v3 helper).

### 4.3 File ownership

Planner writes ONLY:
- `.claude/PRPs/plans/v1-RT-r5.plan.md`
- `.claude/decision-queue.json` (DQ entries via v3 helper)

Planner does NOT write any `crates/**`, `docs/**`, `.claude/lessons/**`, or `.claude/rules/**` file.

### 4.4 Attribution integrity reminder

Every commit subject on the planning worker branch matches the planner-attribution pattern: `chore(plan): rt-r5 — <slug>` or `feat(plan): rt-r5 — <slug>`.

### 4.5 Forbidden execution windows

Non-binding for the planning dispatch (planner is Junior task, no cargo).

---

## 5. Pre-commit dogfood (per advisor-orchestrator.md §3.7)

This brief was walked through against (advisor session, 2026-05-31):
- `.claude/PRPs/prds/v1-reputation-tuning.prd.md` §5.5 (parsed — rollup cron + materialised weekly + equal weights + banned-community exclusion + capability-flip on rollup changes), §7 (parsed — `ENTRY_KIND_ROLLUP_RECOMPUTED` payload), §10 (parsed — cron signs via instance key, `system` pseudonym), §11 row 5 (phase definition + v1.r2 dependency).
- `crates/routes/src/utils/scheduled_tasks.rs` (read — 5 existing concurrency guards, cron registration pattern via clokwerk).
- `crates/api/api/src/governance/reputation_snapshot.rs` (read — `recompute_snapshot` at `:222`, `upsert_snapshot` at `:770`, `load_or_compute_snapshot` at `:624`).
- `crates/api/api/src/governance/admin_reputation_stats.rs` (read — admin-gated reputation read endpoint, closest sibling for the rollup query endpoint).
- `crates/api/api_common/src/governance.rs` (read — DTO patterns for admin reputation endpoints).
- `crates/api/api/src/governance/config.rs` (read — `job.rollup_interval_days` at `:1027`, `job.rollup_equal_weights` at `:1030`, metadata at `:3895-3920`).
- `crates/db_schema/src/source/governance/governance_log.rs:208,215` (read — `ROLLUP_RECOMPUTED` const declared-but-not-emitted).
- `.claude/rules/governance-log-entry-kind-registry.md:188` (read — payload shape + `(pending)` marker).
- `.claude/PRPs/briefs/rt-r4-planning-1.md` (canonical sibling brief — same §0-§6 structure mirrored).

What worked:
- §0 dependency-reality check confirmed: no migration needed (snapshot table already supports NULL community_id), both config knobs already seeded, the `recompute_snapshot` function exists and handles `community_id = None`. RT-r5 is pure handler + cron + endpoint + emit.
- The participation-cron pattern (RT-r3) is the closest cron sibling — same `AtomicBool` + guard + clokwerk registration. The rollup cron mirrors it structurally.
- `admin_reputation_stats.rs` is the closest admin-read-endpoint sibling for the rollup query.

What the planner must resolve:
- **Rollup computation path**: should the cron call `recompute_snapshot(conn, person_id, None, cache)` directly (leveraging event-sourced recompute for instance-wide), or does it need a dedicated helper that aggregates per-community snapshots into a weighted average? The PRD says "weighted average of per-community snapshots" (§5.5), which is a different computation from the event-sourced `recompute_snapshot` (which sums events). Planner reads `recompute_snapshot` body and decides.
- **Banned-community exclusion SQL**: how to detect "person is banned from community C" — likely an active sanction with `target_community_id = C`. Planner greps for the existing ban-detection pattern.
- **e2e test fixture strategy**: seeding per-community snapshots + triggering the rollup cron from a test. Planner reads the participation-cron e2e tests (RT-r3) as the canonical pattern.

---

## 6. Acceptance for this brief

Brief is queueable when:
- DQ pending count = 0 OR all pending entries are non-blocking for RT-r5 planning.
- Forbidden-window check at dispatch time per advisor-orchestrator.md §5.1 (non-binding for planning dispatch).

Brief is committed to `phase-v1-RT-r5` (Mode A — directly on the phase branch the planning Junior forks from) with subject:
```
chore(advisor): auto-roadmap auto-author planning brief for v1-RT-r5
```
