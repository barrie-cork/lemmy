# Sub-PRD: v1 Sponsor Liability — Athgabál Grace Window + `endorsement/revoke`

**Scope:** v1 of the Brehon governance fork (per ADR-010). Resolves OQ-025; ships the v0-deferred `endorsement/revoke` endpoint per [05 §7.1](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md). Additive to numbered design docs; does NOT supersede any ADR.
**Created:** 2026-04-19
**Status:** DRAFT
**Scheduled:** v1 cycle, gated on **admin-dashboard-v1 PRD** (provides the config-write surface) and **jury-mechanics-v1 PRD** (severity tiers feeding grace-window duration mapping). Issue #24 (partial index on `surety`) folded into this PRD's migration plan.
**Naming note:** "v1" here is the ADR-010 v1 milestone ("Is it production-grade governance?") — distinct from the V2 messaging capability track.

---

## §1. Vision and Goals

### 1.1 Brehon mapping — `athgabál` as carrot for early intervention

Brehon distraint (`athgabál`, Higgins 2010 p.7) required the wronged party to issue **formal notice** to the wrongdoer's surety, then wait through a **grace period "depending on the nature of the wrong committed"** before seizure could take place. The grace window served two purposes simultaneously:

1. **Procedural fairness** — no surprise seizure; the surety knew enforcement was coming and could weigh response options.
2. **Carrot for de-escalation** — if the surety revoked their guarantee OR the wrongdoer made restitution within the window, seizure was averted entirely. The grace window is **how Brehon law turned enforcement into invitation**.

v0 of the fork ships sponsor-liability immediately on case-decision (Phase 5b task 56's `apply_sponsor_liability` runs in the same `run_transaction` as the sanction insert). This is the procedurally simplest implementation but **diverges from the Brehon notice requirement**: the sponsor receives no formal notice, has no opportunity to escape liability, and learns of the reputation hit only after the modlog publishes. v1 closes this gap.

### 1.2 v1 goals

1. **Procedural fairness** — every sponsor-liability event is preceded by a formal notice to all active sponsors, with a severity-proportional grace window (Minor 24h / Moderate 72h / Severe 168h defaults; per-community configurable).
2. **Carrot for early intervention** — sponsor revocation OR defendant restoration during the grace window severs the liability chain. The reputation event is never written; the case transitions to a terminal `SponsorLiabilityEscaped` state.
3. **Audit transparency** — every grace-window state change emits a `governance_log` entry. Auditors can reconstruct the full timeline (notice issued → revocation OR restoration OR expiry → fired-or-escaped) from the log alone.
4. **Operator tunability** — grace-window durations and severity-tier mappings are dashboard knobs with sensible Brehon-derived defaults. Per-community overrides allowed; instance-wide defaults seeded.
5. **Backwards compatibility** — v0 cases that have already fired liability are unaffected; cases mid-flight at v1 deploy time get the most lenient retroactive grace per ADR-010's won't-disadvantage rule.

### 1.3 Non-goals

- Cross-instance sponsor-liability (deferred to v2 federation work — see ADR-014).
- Automatic restoration mediation (research territory; v2+ at earliest).
- Negative-band reputation reintegration ceremonies (OQ-021 — v1+ separate PRD).
- Variant refinement of `Restoration` into `Apology | ContentCorrection | CommunityService` (OQ-003 amendment) — flagged here in §7 but is its own decision.

---

## §2. Scope

### IN scope (v1)

- **`CaseStatus::SponsorLiabilityPending`** — new variant inserted between `Decided` and the eventual liability fire.
- **`CaseStatus::SponsorLiabilityFired`** — terminal variant; reputation event was written.
- **`CaseStatus::SponsorLiabilityEscaped`** — terminal variant; sponsor revoked or defendant restored within window.
- **Severity-proportional grace windows** — Minor 24h / Moderate 72h / Severe 168h defaults; per-community configurable via `governance_config` keys with `community:<id>` scope.
- **`POST /api/v4/governance/endorsement/revoke`** HTTP endpoint — the v0-deferred sprint-2 endpoint per [05 §2](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) line 44, now landing.
- **Scheduled task `sponsor_liability_grace_check`** — runs every 5 min via clokwerk; transitions `SponsorLiabilityPending` cases past `grace_expires_at` to `SponsorLiabilityFired` or `SponsorLiabilityEscaped`.
- **Audit-log of every grace-window state change** — five new `ENTRY_KIND_*` constants; payloads pseudonymised per ADR-015.
- **Issue #24 partial index** on `surety(sponsored_id, sponsor_id) WHERE revoked_at IS NULL` — folded into this PRD's migration to avoid a second migration churn.
- **Migration backfill** for v0 cases mid-flight at upgrade time (most-lenient default per ADR-010).

### OUT of scope (deferred)

- **Cross-instance sponsor-liability** — v2 federation work. Inbound `sponsor_liability_pending` notices are stored as advisory evidence per ADR-006; no auto-application across federation boundary.
- **Automatic restoration mediation** — research territory; the v1 design treats restoration as defendant-initiated + admin-attested.
- **Step-up auth for admin revocation** — slot reserved (v2 per ADR-010, alongside Keycloak/MFA).
- **Notification UX surface** — Lemmy notification + dashboard listing only in v1; richer email / push deferred per OQ-005.
- **`Restoration` variant refinement** — flagged in §7 as cross-PRD decision; PRD does not commit.

---

## §3. CaseStatus Extensions

### 3.1 New variants

The `CaseStatus` enum at `crates/db_schema_file/src/enums.rs:393-408` currently has 9 variants (Open, ThresholdMet, JurySelection, InReview, Decided, Appealed, Closed, EmergencyRemove, AdminReview). v1 adds **three** new variants:

```rust
pub enum CaseStatus {
  // ... existing 9 variants ...

  /// Per OQ-025 / v1 sponsor-liability athgabál window.
  /// Case has been Decided AND a sanction with sponsor-liability implications
  /// was created; the case is in its grace window. `moderation_case.grace_expires_at`
  /// holds the computed deadline. Sponsor revocation OR defendant restoration
  /// during this window transitions to `SponsorLiabilityEscaped`. Window expiry
  /// triggers `SponsorLiabilityFired`.
  SponsorLiabilityPending,

  /// Terminal — the grace window expired without escape. The
  /// `apply_sponsor_liability` helper (v0 Phase 5b code, now gated) ran and the
  /// `reputation_event` rows for sponsors were written.
  SponsorLiabilityFired,

  /// Terminal — sponsor revoked or defendant restored within the grace window.
  /// `moderation_case.liability_escape_reason` records the escape mechanism.
  /// No `reputation_event` rows for sponsors were written.
  SponsorLiabilityEscaped,
}
```

### 3.2 New lifecycle

```
... existing Phase 4 lifecycle through Decided ...
    │
    ▼
[Decided + sanction with liability implications]
    │
    ▼  (immediately)
[Set grace_expires_at = decided_at + grace_window_for_severity]
    │
    ▼
SponsorLiabilityPending
    │
    ├── Sponsor revokes endorsement during window ───┐
    │                                                 │
    ├── Defendant marks restoration complete +       │
    │   admin attests during window ─────────────────┤
    │                                                 │
    │                                                 ▼
    │                                          SponsorLiabilityEscaped (terminal)
    │
    └── Window expires
          │
          ▼
     [Scheduled grace-check task evaluates escape conditions]
          │
          ├── Escape conditions met ──► SponsorLiabilityEscaped (terminal)
          │
          └── No escape ──► [apply_sponsor_liability fires]
                            │
                            ▼
                     SponsorLiabilityFired (terminal)
```

The existing `Decided → Appealed | Closed` paths still apply for cases without sponsor-liability implications (e.g. `JuryDecision::NoAction` cases, or Person-target cases where the target has no active sureties). For those, the v0 lifecycle is preserved unchanged.

### 3.3 ADR-013 compliance

Every existing `match CaseStatus { ... }` site MUST be updated to handle the three new variants explicitly — no `_ =>` arms (per workspace clippy `feedback_clippy_test_style.md` denial). Sites enumerated by grep at PRD-write time:

| Site | Current handling | v1 required handling |
|---|---|---|
| `crates/api/api_crud/src/governance/request_appeal.rs:90-102` | `Decided => {}` allows; all others reject | Add `SponsorLiabilityPending => {}` (allow appeal — v1 design choice: appeals during grace window are valid; appeals reset the case to `Appealed` and cancel the grace timer); `SponsorLiabilityFired \| SponsorLiabilityEscaped => Err(NotFound)` (case is terminal — appeal window already expired by the time we reached either) |
| `crates/api/api/src/governance/admin_close_case.rs:65-75` | All except `Closed` allowed | Add three new variants to allowed-set (admins may force-close terminal liability states for ops purposes, e.g. if scheduler is broken) |
| `crates/api/api/src/governance/get_case.rs` (any matches) | TBD per implementation audit | Add three variants where matched |
| `crates/db_views/governance_case/src/impls.rs` | TBD per implementation audit | Add three variants where matched (display label rendering in views) |
| `crates/db_views/governance_modlog/src/impls.rs` | TBD per implementation audit | Add three variants where matched |
| `crates/server/tests/e2e.rs` test-side matches | TBD per implementation audit | Add three variants |

The migration adding the three Postgres enum values uses the **`-- no-transaction`** Diesel directive (per Phase 5b task 56's `Restoration` variant precedent and the carry-forward documented in `phase-5a-config-and-reputation-infrastructure.plan.md §17.2`).

### 3.4 Migration enum additions

```sql
-- migrations/{ts}_add_sponsor_liability_grace_window/up.sql
-- no-transaction
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityPending';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityFired';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityEscaped';
```

Down migration is a no-op doc-comment per OQ-003 / Phase 5b precedent (Postgres enum-value drop unsupported without full type rebuild).

---

## §4. Grace Window Durations

### 4.1 Severity-proportional defaults (Brehon-derived)

| Severity | Default grace window | Brehon-derived rationale |
|---|---|---|
| Minor (Label, VisibilityReduction, Restoration) | **24 hours (1d)** | Restorative-band sanctions imply softest correction; one day suffices for sponsor self-awareness via dashboard or notification |
| Moderate (TemporaryRestriction, ContentRemoval) | **72 hours (3d)** | Mid-band sanctions deserve the working-week window; matches typical activist-coordination response cadence |
| Severe (CommunityExclusion, InstanceSuspension, FederationQuarantineRecommendation) | **168 hours (7d)** | Highest stakes — give sponsors the full week to deliberate, consult sponsee, and choose revocation vs. defending |

Severity is determined by the same `severity_for_action` function in `crates/api/api/src/governance/sponsor_liability.rs:112-124` that v0 ships. Snapshot semantics per ADR-010 won't-disadvantage rule: severity is recorded **at jury-decision time** (Phase 4b Decided transition), not re-evaluated against config at fire time.

### 4.2 Configuration knobs (per-community + instance default)

New `governance_config` rows seeded by the v1 migration:

| Key | Type | Instance default | Range | Per-community? |
|---|---|---|---|---|
| `liability.grace_window_minor_hours` | int | 24 | 1 – 720 (30d) | Yes |
| `liability.grace_window_moderate_hours` | int | 72 | 1 – 720 | Yes |
| `liability.grace_window_severe_hours` | int | 168 | 1 – 720 | Yes |
| `liability.grace_window_minimum_hours` | int | 1 | hard floor; community cannot go below | No (instance only) |
| `liability.grace_window_maximum_hours` | int | 720 | hard ceiling; community cannot exceed | No (instance only) |
| `liability.grace_window_alert_threshold_hours` | int | 24 | audit alert if community sets any per-tier grace below this value | No (instance only) |

Read cascade follows v0 `config.rs` pattern: `community:<id>` row → `instance` row → Rust `const DEFAULT_*` fallback. The dashboard-write path enforces the min/max range; direct `psql` writes also bounded by a Postgres `CHECK` added in the v1 migration (defence-in-depth).

### 4.3 Severity-source semantics (snapshot, not config-derived)

Per ADR-010 the severity that determines grace-window duration is **the severity recorded on the `moderation_case` row at Decided-transition time** (a snapshot of the case's `severity` field), NOT re-evaluated from config at fire-time. This prevents an admin who lowers `Severe → Minor` mid-grace from accelerating fire on cases that were severe at decision-time.

The grace-window duration itself, however, IS read from current config at the `Decided → SponsorLiabilityPending` transition (so admin retuning of grace windows DOES affect cases not yet in pending). After the transition, `grace_expires_at` is locked on the row.

---

## §5. `POST /api/v4/governance/endorsement/revoke` Endpoint

### 5.1 Request DTO

```rust
// crates/api/api_common/src/governance.rs
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RevokeEndorsement {
  pub endorsement_id: EndorsementId,
  pub reason: String,  // required; passed through redaction layer per ADR-015
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RevokeEndorsementResponse {
  pub endorsement_id: EndorsementId,
  pub revoked_at: DateTime<Utc>,
  /// Set true when the revocation severed an active grace-window chain
  /// (transitioned a `SponsorLiabilityPending` case to `SponsorLiabilityEscaped`).
  pub liability_chain_severed_for_cases: Vec<ModerationCaseId>,
}
```

### 5.2 Capability check

- **Self-revocation:** any user MAY revoke their own outbound endorsements (where `endorsement.from_person_id == caller_id`). No reputation gate, no admin role required.
- **Admin revocation:** instance admins MAY revoke any endorsement; requires `is_admin(&local_user_view)` check. Step-up auth is reserved for v2 per ADR-010 (slot in `RevokeEndorsement` DTO would be a `step_up_token: Option<String>` field added in v2 without breaking v1 callers).

### 5.3 Effect

Inside a single `run_transaction` closure (per `feedback_multi_write_handlers_need_transactions.md`):

1. Load endorsement; verify caller is sponsor OR admin; verify not already revoked.
2. Set `endorsement.revoked_at = now()`.
3. Find the matching `surety` row (`from_person_id == sponsor_id, to_person_id == sponsored_id, community_id`); set `surety.revoked_at = now()` if it exists. (Endorsement and Surety are sibling rows created together in `create_endorsement`; revocation severs both.)
4. **Grace-window evaluation:** query for any `moderation_case` rows with `status = 'SponsorLiabilityPending'` AND `target_person_id = sponsored_id` AND `grace_expires_at > now()`. For each such case:
   - Re-query active sponsors (those with `revoked_at IS NULL` after step 3's update).
   - Apply community-configured escape rule (§13's "multiple sponsors" config — default: any-sponsor revocation escapes).
   - If escape condition met: transition case to `SponsorLiabilityEscaped`, set `liability_escape_reason = json!({"reason": "sponsor_revoked", "endorsement_id": ..., "revoked_by_pseudonym": ...})`, emit `sponsor_liability_escaped` log entry.
   - Append revoked case ID to response's `liability_chain_severed_for_cases`.
5. Emit `endorsement_revoked` governance-log entry (always — even if no grace-window severance).
6. Recompute snapshots for sponsor + sponsee (sibling-write pattern from `create_endorsement.rs:307-309`).

### 5.4 Idempotency

Re-revocation is a no-op: if `endorsement.revoked_at IS NOT NULL` on read, the handler returns the existing `revoked_at` in the response with `liability_chain_severed_for_cases: vec![]` and emits no log entry. Per `feedback_multi_write_handlers_need_transactions.md` — read inside the same transaction to avoid TOCTOU.

### 5.5 Backfill

Prior endorsements (v0 rows) with `revoked_at = NULL` continue working unchanged. The endorsement ID space is preserved; no schema column rename. v0 callers of `POST /api/v4/governance/endorsement` are unaffected.

### 5.6 Route registration

Add to `crates/api/routes/src/lib.rs:518` (after `route("/endorsement", post().to(create_endorsement))`):

```rust
.route("/endorsement/revoke", post().to(revoke_endorsement))
```

Handler lives at `crates/api/api_crud/src/governance/revoke_endorsement.rs` (CRUD-shaped per `crates/api/api_crud/src/governance/mod.rs` siblings: `create_endorsement.rs`, `request_appeal.rs`, `create_report.rs`).

---

## §6. Grace-Window Scheduler

### 6.1 New scheduled task

Add to `crates/routes/src/utils/scheduled_tasks.rs::setup` (sibling of the 15-minute `reputation_snapshot` job at lines 166-233):

```rust
// Brehon governance v1: sponsor-liability grace-check tick.
// Every 5 minutes, find SponsorLiabilityPending cases past their
// grace_expires_at and transition them to Fired or Escaped.
// Disabled in tests via BREHON_DISABLE_GRACE_CHECK_JOB=1 (mirrors the
// snapshot-job override pattern from S4 design review).
let context_grace = context.reset_request_count();
scheduler.every(CTimeUnits::minutes(5)).run(move || {
  let context = context_grace.reset_request_count();
  async move {
    if std::env::var("BREHON_DISABLE_GRACE_CHECK_JOB").as_deref() == Ok("1") {
      return;
    }
    if SPONSOR_LIABILITY_GRACE_RUNNING
      .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
      .is_err()
    {
      warn!("sponsor_liability_grace_check: previous batch still running, skipping this tick");
      return;
    }
    let _guard = GraceCheckRunningGuard;
    lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch(&context)
      .await
      .inspect_err(|e| warn!("Failed to run grace-check batch: {e}"))
      .ok();
  }
});
```

`SPONSOR_LIABILITY_GRACE_RUNNING: AtomicBool` + `GraceCheckRunningGuard: Drop` follow the `REPUTATION_SNAPSHOT_RUNNING` / `RunningGuard` pattern from `scheduled_tasks.rs:65-79`.

### 6.2 `run_grace_check_batch` semantics

In `crates/api/api/src/governance/sponsor_liability_grace.rs` (new module):

```sql
-- Conceptual query (Diesel-built in Rust)
SELECT id, target_person_id, community_id, severity, decided_at, grace_expires_at
FROM moderation_case
WHERE status = 'SponsorLiabilityPending'
  AND grace_expires_at <= now()
ORDER BY grace_expires_at ASC
LIMIT $batch_size;  -- config.job.grace_check_batch_size, default 100
```

For each row, inside its **own** `run_transaction` (per-case isolation; one bad case doesn't block the rest):

1. Re-load case row with `FOR UPDATE` (per `feedback_multi_write_handlers_need_transactions.md` and Phase 5a Watch 9 pattern).
2. Re-check status (defence against scheduler-vs-handler race): if status is no longer `SponsorLiabilityPending`, skip silently.
3. Evaluate escape conditions:
   - Any active sponsor revoked since `decided_at`? → Escape.
   - Any restoration completed for `target_person_id` between `decided_at` and now (community config dependent — see §7)? → Escape.
4. If escape: transition to `SponsorLiabilityEscaped`, emit `sponsor_liability_escaped` log, set `liability_escape_reason` with the cause.
5. If fire: invoke the existing `apply_sponsor_liability` helper (Phase 5b code) — now refactored per §9.1 into a "fire" function gated on this scheduler. After fire, transition to `SponsorLiabilityFired`, emit `sponsor_liability_fired` log entry.

### 6.3 Failure mode + observability

- Cron failure logged via `warn!` (mirrors `reputation_snapshot` pattern at line 198).
- Cases stuck in `SponsorLiabilityPending` past `2× max grace window` (i.e. >60 days) emit a `tracing::error!` from a sibling staleness check (mirrors `check_snapshot_staleness` at lines 209-228) — observable from admin dashboard.
- Per-case failure inside the per-case transaction is logged but doesn't block the batch — next tick retries.

### 6.4 Configuration

| Key | Type | Default | Scope |
|---|---|---|---|
| `job.grace_check_interval_minutes` | int | 5 | instance |
| `job.grace_check_batch_size` | int | 100 | instance |
| `job.grace_check_staleness_alert_multiplier` | float | 2.0 | instance |

---

## §7. Restoration Interaction (OQ-003)

### 7.1 v0 reservation

Phase 5b task 56 reserved `SanctionAction::Restoration` as a **unit variant** (per GOTCHA-56b fallback — see `crates/db_schema_file/src/enums.rs:540-557`). The variant is enum-exhaustiveness only in v0; no v0 handler emits it.

### 7.2 v1 refinement (separate decision; flagged here)

OQ-003 contemplates refining `Restoration` into `Apology | ContentCorrection | CommunityService`. **This PRD does NOT commit to that refinement** — it lives in its own jury-mechanics-v1 PRD (which also handles severity-tier widening). This PRD's responsibility is to define how restoration interacts with the grace window IF it lands.

### 7.3 Restoration completion as escape mechanism

When the defendant marks their own restoration as complete AND an admin attests, the case transitions to `SponsorLiabilityEscaped` (sponsor never absorbs reputation hit). Two sub-questions:

1. **Who initiates?** Defendant via a v1 endpoint `POST /api/v4/governance/restoration/complete` (out of scope for this PRD; lives in restorative-mechanics-v1 PRD). Admin attests via a sibling endpoint OR by approving the defendant's claim from the dashboard.
2. **Does restoration ALWAYS escape liability, or just reduce severity?** Configurable per-community:

| Key | Type | Default | Scope |
|---|---|---|---|
| `liability.restoration_escapes_liability` | bool | `true` | community + instance |
| `liability.restoration_severity_reduction_steps` | int | 0 (means "full escape"; non-zero shifts severity down N tiers) | community + instance |

Default `true` keeps the carrot; communities preferring "restoration mitigates but doesn't fully escape" set `restoration_escapes_liability = false` and `restoration_severity_reduction_steps = 1` (Severe→Moderate→Minor cascade).

### 7.4 Cross-PRD coordination

This PRD defines the **escape semantics** at the grace-window level. The jury-mechanics-v1 PRD (which lives in OQ-003 and the wider v1 sanction-variant work) defines the **restoration completion mechanism** itself. Both PRDs reference this section as the contract.

---

## §8. Database & Migration Changes

### 8.1 New columns on `moderation_case`

```sql
ALTER TABLE moderation_case ADD COLUMN grace_expires_at TIMESTAMPTZ;
COMMENT ON COLUMN moderation_case.grace_expires_at IS
    'Per OQ-025 v1 sponsor-liability grace window. Set when transitioning Decided → SponsorLiabilityPending; locked thereafter. NULL for cases not in the grace lifecycle (NoAction outcomes, no-sponsor target, v0 backfill).';

ALTER TABLE moderation_case ADD COLUMN liability_escape_reason JSONB;
COMMENT ON COLUMN moderation_case.liability_escape_reason IS
    'Per OQ-025: structured reason recording the mechanism that caused SponsorLiabilityEscaped. Schema: {"reason": "sponsor_revoked"|"restoration_completed"|"admin_override", "actor_pseudonym": "...", "endorsement_id"|"restoration_id": ...}. NULL for non-escaped cases.';

CREATE INDEX moderation_case_grace_expires_idx
    ON moderation_case (grace_expires_at)
    WHERE status = 'SponsorLiabilityPending';
```

The partial index above is the scheduler's primary access path — only ~handful of pending cases at any time across an instance, so the partial-index covers the scheduler query in O(rows-pending).

### 8.2 Issue #24 partial index

Per GitHub issue [#24](https://github.com/barrie-cork/lemmy/issues/24) (CodeRabbit finding on PR #10) — fold into this PRD's migration to avoid a second migration churn:

```sql
CREATE INDEX surety_sponsored_id_active
    ON surety (sponsored_id, sponsor_id)
    WHERE revoked_at IS NULL;
COMMENT ON INDEX surety_sponsored_id_active IS
    'Per Issue #24 (CodeRabbit, PR #10). Speeds up jury_common::select_eligible_jurors EXISTS subquery and apply_sponsor_liability sponsor enumeration. Partial WHERE matches the existing "active sureties" filter pattern in both call sites.';
```

The matching `DROP INDEX IF EXISTS surety_sponsored_id_active;` lands in `down.sql`.

### 8.3 New `governance_config` seed rows

The v1 migration seeds the §4.2, §6.4, §7.3, §12.1, §13.1 keys via `INSERT INTO governance_config ... ON CONFLICT DO NOTHING` (matches the Phase 5a task 50 idempotent-seed pattern). Full enumeration in §10 defaults matrix.

### 8.4 Migration backfill (ADR-010 won't-disadvantage rule)

Per ADR-010: existing v0 cases with already-fired liability are unchanged (their `reputation_event` rows already exist; their `status = Decided`; no migration touches them). For cases mid-flight at v1 deploy time:

```sql
-- For cases that have decided_at recent enough that liability was supposed to fire
-- under v0 semantics but the v1 deploy interrupts: retroactive grace window of
-- 24h (Minor default — most lenient per ADR-010 won't-disadvantage rule).
-- These cases acquire SponsorLiabilityPending status; the scheduler picks them up.
-- IMPORTANT: must also exclude cases where v0 immediate-fire already wrote
-- reputation_event rows for sponsors (otherwise we double-fire on next scheduler tick).
UPDATE moderation_case
SET status = 'SponsorLiabilityPending',
    grace_expires_at = decided_at + INTERVAL '24 hours'
WHERE status = 'Decided'
  AND decided_at IS NOT NULL
  AND decided_at > now() - INTERVAL '24 hours'
  AND target_person_id IS NOT NULL
  AND id IN (
    SELECT mc.id
    FROM moderation_case mc
    WHERE EXISTS (
      SELECT 1 FROM surety s
      WHERE s.sponsored_id = mc.target_person_id
        AND s.revoked_at IS NULL
    )
    AND EXISTS (
      SELECT 1 FROM sanction sa
      WHERE sa.case_id = mc.id
    )
    AND NOT EXISTS (
      SELECT 1 FROM reputation_event re
      WHERE re.source_case_id = mc.id
        AND re.reason = 'sponsor_liability_applied'
    )
  );
```

The 24-hour minor-default grace gives sponsors of mid-flight cases a chance to escape liability — most lenient possible interpretation of the v0→v1 transition.

### 8.5 Migration ordering

Single migration file: `migrations/{ts}_add_sponsor_liability_grace_window/`. Combines:

1. `-- no-transaction` directive (required for `ALTER TYPE`).
2. Three `ALTER TYPE case_status ADD VALUE IF NOT EXISTS ...` statements.
3. Two `ALTER TABLE moderation_case ADD COLUMN ...` statements.
4. Index creation: `moderation_case_grace_expires_idx` + `surety_sponsored_id_active` (Issue #24).
5. Seed `INSERT INTO governance_config ... ON CONFLICT DO NOTHING` for all new keys.
6. Backfill `UPDATE moderation_case ...` for mid-flight cases.

The migration runs idempotently; reruns after manual admin edits preserve the edits via the unique-on-`(scope, key, valid_from)` constraint.

---

## §9. Handler Changes

### 9.1 `apply_sponsor_liability` split

Phase 5b's `crates/api/api/src/governance/sponsor_liability.rs` currently does both **compute** AND **fire** in one function call (called from `submit_jury_vote::process_vote` step 8.5). v1 splits it:

- **`compute_sponsor_liability(...) -> Vec<SponsorDelta>`** — pure read; computes every per-sponsor delta, founder multiplier, clamped final delta. Does NOT write `reputation_event` or `governance_log`. Idempotent.
- **`fire_sponsor_liability(deltas: Vec<SponsorDelta>, ...) -> LemmyResult<usize>`** — writes `reputation_event` rows + `sponsor_liability_applied` + `sponsor_liability_clamped` log entries. Called from the **scheduler** (§6) at grace-window expiry, not from `submit_jury_vote`.

Existing `apply_sponsor_liability` becomes a thin wrapper retained for **internal call sites that DO want immediate fire** (none in v1, but preserved for v2 admin-override paths).

### 9.2 New handler: `revoke_endorsement`

File: `crates/api/api_crud/src/governance/revoke_endorsement.rs` (new).
Module wiring: add `pub mod revoke_endorsement;` to `crates/api/api_crud/src/governance/mod.rs`.
Route wiring: per §5.6.

Implementation skeleton mirrors `create_endorsement.rs:118-145` (outer handler) + `process_endorsement(...)` (transaction body):

```rust
pub async fn revoke_endorsement(
  Json(data): Json<RevokeEndorsement>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<RevokeEndorsementResponse>> {
  check_local_user_valid(&local_user_view)?;

  let caller_id = local_user_view.person.id;
  let caller_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), caller_id).await?;
  let is_admin_caller = is_admin(&local_user_view).is_ok();

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let outcome = conn
    .run_transaction(|conn| {
      async move {
        process_revocation(conn, caller_id, caller_pseudonym, is_admin_caller, data).await
      }
      .scope_boxed()
    })
    .await?;

  Ok(Json(outcome))
}
```

`process_revocation` body follows §5.3.

### 9.3 `submit_jury_vote` mutation — pointer to jury-mechanics-v1 §9.1

**Per B6 resolution 2026-04-19, the combined post-v1 `submit_jury_vote` handler shape is owned by `v1-jury-mechanics.prd.md` §9.1** (9-step integrated pseudocode: load/validate/insert-vote → tally → threshold-check with deadlock-to-`AdminReview` → winning-decision sanction-insert → **sponsor-liability branch (this PRD contributes the semantics)** → no-sponsor `Decided` branch preserved from v0 → appeal-window bound). See that section for the full flow; the rest of this paragraph documents what **this** PRD contributes into the integrated handler:

1. **The `SponsorLiabilityPending` status transition.** When the sanction has liability implications AND the target has active sponsors, the handler transitions the case to `CaseStatus::SponsorLiabilityPending` (rather than v0's immediate `Decided`). The compute phase (not the fire phase) runs inline; the fire phase runs from the scheduler per §6.
2. **Grace-window computation.** `grace_expires_at = now() + grace_window_for_severity(severity)`, snapshotted onto `moderation_case.grace_expires_at` at the Pending transition. The severity used is the snapshot severity (§4.3), not the live config-read severity.
3. **Deferred write set.** On the `SponsorLiabilityPending` branch, `public_case_log` entries and juror `reputation_event` rows are **NOT written at vote-tally time**. They fire from the scheduler when the case transitions to `SponsorLiabilityFired` or `SponsorLiabilityEscaped` (see §6.2 `run_grace_check_batch`). This is deliberate — auditors see "case decided X AND liability outcome was Y" as one coherent timeline event per §11.4. The no-sponsor path (step 8 of the integrated handler) preserves v0 immediate-write semantics.

The **compute/fire split** itself (`compute_sponsor_liability` vs `fire_sponsor_liability`) is specified in §9.1 of this PRD. The **grace-window helper module** (`grace_window_for_severity`, `evaluate_escape_conditions`, `fire_or_escape_case`) is specified in §9.4 of this PRD. The **`submit_jury_vote` call site** where the compute phase fires is specified in `v1-jury-mechanics.prd.md` §9.1 step 7.

### 9.4 Grace-window helper module

New file: `crates/api/api/src/governance/sponsor_liability_grace.rs`. Contains:

- `pub async fn run_grace_check_batch(context: &LemmyContext) -> LemmyResult<()>` — scheduler entry.
- `pub async fn evaluate_escape_conditions(conn, case_id, target_person_id, community_id) -> LemmyResult<EscapeStatus>` — pure read; returns `Escape{reason}` or `Fire`.
- `pub async fn fire_or_escape_case(conn, case_row, ...) -> LemmyResult<()>` — per-case transaction body.

Module wired via `crates/api/api/src/governance/mod.rs` `pub mod sponsor_liability_grace;`.

### 9.5 Scheduler wiring

Per §6.1 — additions to `crates/routes/src/utils/scheduled_tasks.rs`.

---

## §10. Defaults Matrix

| Knob | Namespace | Default | Range | Per-community? | Citation |
|---|---|---|---|---|---|
| Minor grace window | `liability.grace_window_minor_hours` | 24 (1d) | 1–720 | Yes | §4.1 Brehon-derived; OQ-025 |
| Moderate grace window | `liability.grace_window_moderate_hours` | 72 (3d) | 1–720 | Yes | §4.1 |
| Severe grace window | `liability.grace_window_severe_hours` | 168 (7d) | 1–720 | Yes | §4.1; mirrors typical "deliberation week" |
| Hard floor | `liability.grace_window_minimum_hours` | 1 | (instance) | No | §4.2 audit-floor |
| Hard ceiling | `liability.grace_window_maximum_hours` | 720 (30d) | (instance) | No | §4.2; matches admin-config-write maximum-config-rolloff |
| Audit alert below | `liability.grace_window_alert_threshold_hours` | 24 | (instance) | No | §4.2; trips dashboard warning if community sets <24h |
| Restoration auto-escapes | `liability.restoration_escapes_liability` | true | bool | Yes | §7.3 |
| Restoration severity-reduction steps | `liability.restoration_severity_reduction_steps` | 0 | 0–3 | Yes | §7.3 |
| Multi-sponsor escape rule | `liability.multi_sponsor_escape_rule` | `"any_revocation"` | enum: `any_revocation` \| `all_revocation` \| `majority_revocation` | Yes | §13.1 |
| Revocation rate-limit (per user per day) | `liability.revoke_rate_limit_per_day` | 5 | 1–100 | No (instance) | §12.1 |
| Grace-check tick interval | `job.grace_check_interval_minutes` | 5 | 1–60 | No (instance) | §6.4 |
| Grace-check batch size | `job.grace_check_batch_size` | 100 | 1–10000 | No (instance) | §6.4 |
| Staleness alert multiplier | `job.grace_check_staleness_alert_multiplier` | 2.0 | 1.0–10.0 | No (instance) | §6.4 |

**Total new config keys:** 12. All seeded in the v1 migration via the same `ON CONFLICT DO NOTHING` idempotent-seed pattern from Phase 5a.

---

## §11. Backwards Compatibility (incl. documented behavioural changes)

### 11.1 Already-fired v0 cases

Cases with `status = 'Decided'` AND already-existing `reputation_event` rows for sponsors (the v0 immediate-fire pattern) are **unchanged**. The migration backfill query (§8.4) excludes them via the `AND NOT EXISTS reputation_event` clause.

### 11.2 Mid-flight cases at v1 deploy

Cases `Decided` within the last 24 hours of v1 deploy time get retroactive grace per ADR-010 won't-disadvantage rule:

- `status` flipped to `SponsorLiabilityPending`.
- `grace_expires_at = decided_at + 24h` (Minor default — most lenient).
- The scheduler picks them up at `grace_expires_at` and fires-or-escapes.

### 11.3 v0 endpoint contracts preserved

- `POST /api/v4/governance/endorsement` — unchanged (sets `revoked_at = NULL` on insert; v1 readers handle NULL identically to v0).
- `POST /api/v4/governance/jury/vote` — DTO unchanged; response shape unchanged. Internal lifecycle differs (case may flip to `SponsorLiabilityPending` instead of `Decided`) but the response field `case_decided: true` still fires when quorum is reached.
- `GET /api/v4/governance/case` — exposes `status: CaseStatus` directly; clients should handle the three new variants. Deprecation pattern: clients that match exhaustively on `CaseStatus` will fail-loud with v1 enum additions; clients that switch on a few variants and ignore the rest are unaffected.

### 11.4 v0 modlog readers — documented behavioural change

`GET /api/v4/governance/modlog` returns rows from `public_case_log` — which for sponsor-liability-bearing cases is ONLY written **at scheduler fire/escape time**, not at vote-tally time (see `v1-jury-mechanics.prd.md` §9.1 step 7's deferred-write set). v0 modlog readers polling the modlog for a case in `SponsorLiabilityPending` state see the case **absent** from the modlog until the grace window expires and the scheduler transitions it to `SponsorLiabilityFired` or `SponsorLiabilityEscaped`. Until then (minutes-to-days delay, bounded by the grace window — default Minor 24h / Moderate 72h / Severe 168h per §4.1), the `modlog` endpoint reports "no entry" for that case.

**This is a documented behavioural change from v0**, not a bug. Framed as correct semantics — the case isn't "publicly decided" until liability is resolved. But v0 clients that poll `GET /modlog` and expect decided cases to appear immediately will observe a delay gap on sponsor-liability cases. No API versioning is introduced for this change (v4 modlog endpoint unchanged); the delay is a property of when `public_case_log` emits.

**Recommended v1 client pattern:**
- Poll `GET /case/<id>` for the authoritative current state. The `status` field reflects the truth immediately at vote-tally time: `InReview` → (`Decided` | `SponsorLiabilityPending` | `AdminReview`) is visible in the case endpoint within the same transaction as the jury vote.
- Poll `GET /modlog` for the **public** event stream; accept that sponsor-liability-bearing cases will appear minutes-to-days after the jury decides, once liability is resolved.
- For admin dashboards, prefer the admin case-list endpoint (`GET /admin/cases?status=SponsorLiabilityPending`) which surfaces mid-grace cases explicitly.

ADR-010's won't-disadvantage rule is honoured: v0-decided cases (those already in `Decided` status at v1 deploy) have their existing `public_case_log` entries preserved, and the migration backfill (§8.4) only touches cases where both the `Decided` status AND the absence of `reputation_event` rows for sponsors indicate the case is genuinely mid-flight. Already-published modlog entries are not rewritten.

### 11.5 Federation interop

ADR-014: governance signals are fork-only AP types. v1 MUST NOT silently introduce a new `SponsorLiabilityPendingObject` AP type — federation handling of grace windows is **out of scope** per §2 OUT (cross-instance sponsor-liability deferred to v2). Federated content actions (sanction-mediated `removed = true` flips on posts) still fire from the scheduler at fire-time, identical to v0 fire-time semantics.

---

## §12. Security

### 12.1 Endorsement revocation rate-limit

| Key | Type | Default | Notes |
|---|---|---|---|
| `liability.revoke_rate_limit_per_day` | int | 5 | Per-user per-rolling-24h cap on revocation calls |

Implemented as `count(*) FROM endorsement WHERE from_person_id = caller_id AND revoked_at > now() - INTERVAL '24 hours'` check at handler entry, before any writes. Rate-limit response is `LemmyErrorType::RateLimitError` (consistent with `crates/api/api_crud/src/governance/create_endorsement.rs:198-214` cooldown pattern).

Admin revocation **bypasses the rate-limit** (explicit log entry records the bypass).

### 12.2 Admin revocation requires audit-log + reason

The `RevokeEndorsement.reason` field is **required** (length > 0 after trim — mirror of `admin_close_case.rs:30-32` pattern). Scrubbed via `governance_log`'s `scrub_json` on append (per ADR-015).

### 12.3 Step-up auth — v2 reservation

Per ADR-010 v2 brings step-up auth (Keycloak/MFA). v1 reserves a slot in the DTO (a future `step_up_token: Option<String>` field) but does not enforce. Documented in code-comments and the v2-security-hardening PRD.

### 12.4 Threat-model rows for v1

The threat-model in [06 §7](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) gains rows for:

| Threat | Mitigation |
|---|---|
| Compromised sponsor account mass-revokes during grace windows to escape liability for malicious sponsees | Revocation rate-limit (§12.1) bounds blast radius; admin can re-fire post-hoc via `admin_force_liability_fire` (v2 reservation) |
| Hostile community admin sets `grace_window.severe = 1h` to squeeze sponsors of dissident defendants | Hard floor `minimum_hours = 1` (instance, non-overridable); audit alert when community sets <`alert_threshold_hours` |
| Scheduler crash/silent failure leaves cases stuck in `SponsorLiabilityPending` indefinitely | Staleness check (§6.3) emits `tracing::error!`; admin dashboard surfaces stuck cases |
| Defendant + sponsor collude on fake restoration to escape liability | Admin-attestation requirement on restoration completion (§7.3); audit-log captures attestor pseudonym |
| Race between scheduler fire and sponsor revocation | Per-case `FOR UPDATE` (§6.2 step 1); revocation handler re-queries case status inside its own tx (§5.3 step 4) |

---

## §13. Open Questions for v1 Design Phase

### OQ-V1-SL-01 — Multi-sponsor escape rule

**Question:** If a user has 3 sponsors and 1 revokes during the grace window, does liability escape for ALL sponsors (including the 2 who didn't revoke), or only for the revoking sponsor?

**Proposal:** Configurable per-community via `liability.multi_sponsor_escape_rule`:

| Value | Behaviour |
|---|---|
| `any_revocation` (default) | One sponsor revoking severs the chain entirely; ALL sponsors escape liability. Reflects Brehon "any surety can refuse" reading. |
| `all_revocation` | Only escapes if EVERY active sponsor revokes during the window. Stricter; prevents one cooperative sponsor from absolving the others. |
| `majority_revocation` | Escapes when >50% of active sponsors revoke. Mid-ground. |

Default `any_revocation` favours the carrot-for-de-escalation reading. Communities preferring stricter accountability set `all_revocation`.

### OQ-V1-SL-02 — Sponsor-of-sponsor liability chain depth

**Question:** v0 sponsor-liability is 1-deep (sponsor → sponsee). Should v1 cascade liability through multi-hop chains (sponsor of a sponsor of a sanctioned user)?

**Proposal:** **Keep 1-deep for v1.** Multi-hop is an OQ-021-adjacent design surface that needs pilot data before committing. Document as v2 candidate.

### OQ-V1-SL-03 — Sponsor notification cadence

**Question:** How does v1 notify sponsors that they're in a grace window?

**Proposal:** Per OQ-005 (juror notification UX, v1 scope) the v1 notification surface already grows. Reuse it:

- **Lemmy notification row** at `Decided → SponsorLiabilityPending` transition (per `crates/api/api_utils/src/notify.rs` plugin-hook surface).
- **Admin dashboard listing** of sponsor's active grace-window cases (read-side view crate).
- **Email** opt-in via `notification_settings` (already exists in upstream Lemmy).

Reminder cadence: at 50% of grace-window elapsed, plus 24h before expiry. No SMS / push in v1 (v3 polish per ADR-010).

### OQ-V1-SL-04 — Restoration mechanism interaction with grace window

**Question:** Should the restoration completion endpoint be in this PRD, the jury-mechanics-v1 PRD, or a third "restorative-mechanics-v1" PRD?

**Current lean:** Restorative-mechanics-v1 PRD owns the endpoint; this PRD owns only the **escape semantics** (§7). Cross-reference both ways.

### OQ-V1-SL-05 — `liability_escape_reason` schema versioning

**Question:** The `liability_escape_reason` JSONB column is unversioned in this PRD. Should we add `{"version": 1, "reason": ..., ...}` from day one for forward-compat?

**Proposal:** **Yes** — add `version: 1` to the JSON shape at v1 land. Future schema changes (e.g. adding `restoration_id` for restoration escapes vs `endorsement_id` for revocation escapes) bump version. Costs nothing in v1; saves a migration in v2.

---

## §14. Cross-References

- **admin-dashboard-v1 PRD** (sibling) — provides the config-write surface for §4.2 community-scoped grace-window keys, §10 defaults matrix, and §13 multi-sponsor escape-rule selection. Soft gate: admins can edit via direct `psql` until dashboard ships, mirroring the v0→v1 admin-config-write maturity curve (OQ-018).
- **jury-mechanics-v1 PRD** (sibling) — provides the v1 severity tiers (Low/Medium/High/Critical → Minor/Moderate/Severe sponsor-liability buckets) that feed §4.1 grace-window duration mapping. The PRD also handles the OQ-003 `Restoration` variant refinement that this PRD's §7.3 references.
- **reputation-tuning-v1 PRD** (sibling) — provides per-dimension decay tuning that interacts with how sponsors recover from sponsor-liability events. v1 scope per ADR-010.
- **OQ-018 admin config write endpoint** (99-register) — provides the HTTP endpoint that the dashboard calls to mutate this PRD's config keys.
- **OQ-021 (negative-band reintegration ceremony)** — separate v1+ design surface; this PRD's `SponsorLiabilityEscaped` status interacts with reintegration paths.

### v1 carry-forward issues

- **GitHub Issue [#24](https://github.com/barrie-cork/lemmy/issues/24)** — Partial index on `surety(sponsored_id)` — folded into this PRD's migration (§8.2).

---

## §15. Implementation Phases (for follow-up `/prp-plan` runs)

| # | Phase | Description | Status | Depends |
|---|---|---|---|---|
| 1 | v1-SL-a — Schema + enum + migration | Three new `CaseStatus` variants; two new `moderation_case` columns; Issue #24 partial index; 12 new `governance_config` seed rows; backfill query | pending | v0 merge complete |
| 2 | v1-SL-b — `revoke_endorsement` handler + DTO + route | New handler crate; route registration; integration tests for self-revoke + admin-revoke + idempotency + rate-limit | pending | Phase 1 |
| 3 | v1-SL-c — Scheduler + grace-check helper | New `sponsor_liability_grace.rs` module; clokwerk wiring at 5-min tick; staleness check; per-case `FOR UPDATE` semantics | pending | Phase 1 |
| 4 | v1-SL-d — `submit_jury_vote` mutation + `apply_sponsor_liability` split | Compute/fire split; `Decided → SponsorLiabilityPending` transition; sponsor notifications | pending | Phases 1, 3 |
| 5 | v1-SL-e — e2e test suite | Three+ branches: revocation-during-window-escapes; restoration-during-window-escapes; window-expiry-fires; backfill-of-mid-flight | pending | Phases 1–4 |

Each phase is independently shippable behind a feature flag if needed (e.g. `feature.sponsor_liability_grace_window_enabled` per the B3 `feature.*` namespace convention); v1-SL-a alone is harmless (the new variants exist but no handler writes them).

---

## §16. Decisions Log

| Decision | Choice | Alternatives | Rationale |
|---|---|---|---|
| Three new `CaseStatus` variants vs reuse of `Decided` | Three variants (Pending, Fired, Escaped) | Single `Decided` with sub-status column | Three variants compose with ADR-013 exhaustive-match invariant; sub-status would split the truth across two columns and require custom checks at every match site |
| Grace-window severity source | Snapshot at Decided-transition time | Re-evaluate from config at fire-time | Per ADR-010 won't-disadvantage rule; admin retuning of severity mid-grace would otherwise re-litigate the jury's decision |
| Default grace windows | Minor 24h / Moderate 72h / Severe 168h | All-72h flat; admin sets all | Brehon athgabál required severity-proportional windows; "depending on the nature of the wrong" is the original principle |
| Multi-sponsor escape rule | Configurable; default any-revocation | Always-all; always-any | OQ-V1-SL-01 — communities should choose; default favours carrot-for-de-escalation |
| Restoration escape | Configurable per-community | Always-escape; severity-reduction-only | OQ-V1-SL-04; restorative-band community policy varies |
| Issue #24 partial index | Fold into v1-SL migration | Separate migration | Avoids second migration churn; index needs land in same release as grace-window scheduler that exercises the same query path |
| Step-up auth for admin revocation | Reserve slot for v2; no enforcement in v1 | Implement now | Per ADR-010 v2 brings step-up auth alongside Keycloak; v1 doesn't have the auth substrate |
| Scheduler tick interval | 5 minutes | 1 min; 15 min | 5 min balances responsiveness (sponsors don't wait long after grace expires) with DB load (one query every 5 min is negligible) |
| Mid-flight backfill window | 24h pre-deploy | 7 days; 0 (don't backfill) | Most lenient default per ADR-010; 24h covers same-day rollouts; longer windows risk re-firing already-fired cases without a careful EXISTS guard |
| `liability_escape_reason` schema versioning | Version 1 from day one | Unversioned | OQ-V1-SL-05 — costs nothing now, saves migration later |
| Public log + juror reputation timing | Move to scheduler fire-time | Keep at jury-vote-submission time | Auditors see coherent timeline (decision + outcome) in one logical "event"; matches Brehon "decision is incomplete until enforcement resolves" model |

---

## §17. Cross-Cutting Impact

- [x] **Hash-chain governance log touched?** YES — five new `ENTRY_KIND_*` constants: `SPONSOR_LIABILITY_PENDING`, `SPONSOR_LIABILITY_FIRED`, `SPONSOR_LIABILITY_ESCAPED`, `ENDORSEMENT_REVOKED`, `RESTORATION_COMPLETED` (the last reserved here even though emitted by the restorative-mechanics-v1 PRD's endpoint, for `governance_log.rs` const-discipline). Zero migration — `entry_kind` is TEXT per PR #10 / V2 messaging research §Q4.
- [x] **`actor_pseudonym` table or redaction service touched?** YES (passively) — every grace-window log entry uses `actor_pseudonym_helper::get_or_create` for sponsor / defendant / admin attribution. No change to the helper or redaction logic.
- [x] **`CaseStatus::EmergencyRemove` affected?** No — emergency-remove cases bypass the grace window entirely (admin override is post-facto reviewed by jury but never produces sponsor-liability per Phase 5b GOTCHA-56h target-person inference).
- [x] **AGPLv3 notice / source disclosure affected?** No — additive code only.
- [x] **ADR-013 enum-exhaustiveness invariant?** YES — three new `CaseStatus` variants force every existing match site to update. v1-SL-a phase plan must enumerate via grep (mirror of Phase 5b `SanctionAction::Restoration` audit at plan §10).
- [x] **ADR-010 won't-disadvantage rule?** YES — §11.2 backfill applies most-lenient default (Minor 24h) to mid-flight cases.
- [x] **ADR-015 pseudonymisation?** YES — every new log payload routed through `governance_log::append`'s `scrub_json` layer; no raw `person_id` in any payload (Watch 10 grep applies).

---

## §18. Resolutions applied (2026-04-19)

Cross-PRD coherence-audit edits applied during v1-PRD edit pass (see `.claude/PRPs/v1-planning-queue.json`):

| ID | Severity | Scope | Status |
|---|---|---|---|
| **B4** | blocking | Flatten `sponsor.liability.*` → `liability.*` to match v0 flat-namespace precedent. 10 keys renamed (6 grace-window + 2 restoration + 1 multi-sponsor + 1 rate-limit). Eliminates a parallel `sponsor.*` top-level namespace for the same concept as `liability.*` (v0 flat). | done |
| **B6** | blocking | §9.3 reduced from full Rust pseudocode to pointer to `v1-jury-mechanics.prd.md` §9.1 (combined 9-step `submit_jury_vote` handler). This PRD retains ownership of three semantic contributions: the `SponsorLiabilityPending` status transition, the grace-window compute formula, and the deferred `public_case_log` + juror `reputation_event` write set. §9.1 compute/fire split and §9.4 scheduler helper module remain this PRD's concern. | done |
| **N4** | non-blocking | §11.4 rewritten as explicit documented behavioural change: sponsor-liability-bearing cases appear in modlog minutes-to-days after jury decides (bounded by grace window), not immediately. Client pattern recommended: poll `GET /case/<id>` for authoritative status and `GET /modlog` for the public event stream; admin dashboards should prefer the admin case-list endpoint. §11 heading renamed to "Backwards Compatibility (incl. documented behavioural changes)." | done |

### B4 key-rename table (authoritative)

| Old (draft) | New (final) |
|---|---|
| `sponsor.liability.grace_window.minor_hours` | `liability.grace_window_minor_hours` |
| `sponsor.liability.grace_window.moderate_hours` | `liability.grace_window_moderate_hours` |
| `sponsor.liability.grace_window.severe_hours` | `liability.grace_window_severe_hours` |
| `sponsor.liability.grace_window.minimum_hours` | `liability.grace_window_minimum_hours` |
| `sponsor.liability.grace_window.maximum_hours` | `liability.grace_window_maximum_hours` |
| `sponsor.liability.grace_window.alert_threshold_hours` | `liability.grace_window_alert_threshold_hours` |
| `sponsor.liability.restoration_escapes_liability` | `liability.restoration_escapes_liability` |
| `sponsor.liability.restoration_severity_reduction_steps` | `liability.restoration_severity_reduction_steps` |
| `sponsor.liability.multi_sponsor_escape_rule` | `liability.multi_sponsor_escape_rule` |
| `sponsor.liability.revoke_rate_limit_per_day` | `liability.revoke_rate_limit_per_day` |

### Carry-forward impact

- **§4.2 configuration-knobs table** reflects new flat keys (6 rows).
- **§7.3 restoration-escape knobs** reflect new flat keys (2 rows).
- **§10 Defaults Matrix** — all 10 sponsor-liability-owned rows carry new key names; `job.grace_check_*` rows unchanged (already under `job.*`).
- **§12.1 revocation rate-limit table** reflects new flat key name.
- **§13 OQ-V1-SL-01 multi-sponsor escape-rule** references new flat key.
- **§15 implementation-phase feature-flag illustration** updated to use `feature.*` namespace (from B3).
- **admin-dashboard-v1 §3.1 + §5.2** updated separately to reflect these 10 owned keys (see admin-dashboard-v1 §11 Resolutions B4 row).

Total new config keys in this PRD: **12** (10 sponsor-liability + 2 scheduler-job; unchanged; namespace flattened).

---

*Generated: 2026-04-19*
*Status: DRAFT — v1 scope. Review before running `/prp-plan` (at v1 schedule time, after admin-dashboard-v1 PRD ships).*
