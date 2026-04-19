# Sub-PRD: v1 Reputation Tuning

**Scope**: v1 governance-mechanics track — promotes the v0 hardcoded reputation-decay stub and single-source endorsement model into a configurable, multi-source, per-dimension decay system tuned per community via admin dashboard.
**Created**: 2026-04-19
**Status**: DRAFT
**Scheduled**: v1 (per [ADR-010](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) — "Is it production-grade governance?"). Strictly post-v0; depends on Phase 5/6 completion.
**Predecessor**: v0 reputation calculator at `crates/api/api/src/governance/reputation_snapshot.rs` (Phase 5a/5c) — the per-dimension halving stub that this PRD replaces.
**Naming note**: "v1" in this document refers to the v1 milestone in [ADR-010](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md); not to be confused with v2-security or v3-verifiability tracks.

---

## 1. Problem Statement

The v0 reputation system shipped as an intentional stub. The decay calculator in `crates/api/api/src/governance/reputation_snapshot.rs:320-349` halves positive organic events past a single instance-wide half-life (default 90d), never decays negative events (penalties persist), and applies one decay event per organic event regardless of how old it is — there is no chained halving, no per-dimension tuning, and no negative-decay path for restorative reintegration. The v0 `participation_consistency` dimension has exactly **one** event source (`create_endorsement` emits `+5` for the sponsee per [99 OQ-013](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)) — a user who never participates in the sponsorship graph stays pinned at zero forever, breaking the [01 §5.3](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) principle that participation reputation should reflect "regular, constructive participation in community." The v0 sponsor-gate has three strategies (`age` / `open` / `closed`) but cannot express the nuanced gates real pilot communities will need (vouch-or-age, reputation-based, allowlist). Reputation snapshots are per-(person, community) only; instance admins cannot see a user's instance-wide standing for cross-community jury eligibility filters per [99 OQ-001](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md). Five carry-forward issues from CodeRabbit's PR #10 review (#19, #20, #21, #22, #31) are open against the v0 reputation surface.

v1 lands the **production-grade reputation mechanic**: per-dimension decay rates tuned via admin dashboard, multi-source `participation_consistency` events fed by a weekly cron + vote/evidence outcomes, three additional sponsor-gate strategies, an instance-wide reputation roll-up endpoint, and the five carry-forward issue fixes.

**Critical framing (user, 2026-04-19):** "The admin dashboard will allow community groups to experiment with how they configure a lot of these settings, although defaults might have to be decided on." Reputation parameters are **dashboard knobs with sensible defaults**; communities tune to taste. The PRD reflects this — every decay rate, multi-source increment, and gate strategy is config-driven.

**Primary actor** ([02 §2](../../docs/brehon-law-inspired-network/02-domain-model.md)): **Community Admin** (operator persona). Tunes per-community reputation knobs from the dashboard and observes the downstream effects on jury eligibility, sponsor capability, and capability-flip cadence. Validation surface (whether the v1 mechanic actually feels right) lands here, not on individual jurors.

**Secondary actors**: Instance admin (rolls up reputation across communities for cross-community eligibility queries; enables/disables v1 decay via feature flag), trusted member (sees their participation_consistency rise from non-endorsement event sources), juror (vote-outcome event source produces realised reputation deltas after case close).

---

## 2. Evidence

**v0 reputation calculator** — `crates/api/api/src/governance/reputation_snapshot.rs:203-349` ships the single-half-life decay stub. Module-level docs at lines 1-46 are explicit that `compute_applied_delta` is a v0 simplification and that "v1 — switch to chained halving per half-life elapsed" is a documented TODO at line 341.

**v0 config surface** — 33 keys seeded by Phase 5a task 50 in `crates/api/api/src/governance/config.rs:319-352, 423-458`. The reputation-relevant keys (`thresholds.*`, `decay.*`, `deltas.*`, `liability.*`, `onboarding.*`, `founder.*`, `job.*`) all live in `governance_config_current` and are reachable via the typed `get_int` / `get_float` / `get_bool` / `get_text` helpers at lines 290-302. **Only `decay.positive_half_life_days` exists as a decay knob today** — there is no per-dimension or per-direction (positive/negative) decay surface.

**v0 reputation event sources** — Phase 5a/5b only emit reputation events from four code paths: `create_endorsement` (Phase 5a task 55, +5 to `endorsement_strength` for sponsor + +5 to `participation_consistency` for sponsee per [99 OQ-013](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)), `submit_jury_vote` (Phase 5b — juror align/outlier + reporter upheld/dismissed deltas per [05 §6](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md)), `apply_sponsor_liability` (Phase 5b task 56, sponsor-side penalties per [01 §5.2](../../docs/brehon-law-inspired-network/01-vision-and-principles.md)), and `seed_founders` CLI (Phase 5a task 59, founder-cliff seeds per [99 OQ-017](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)). **No periodic activity scan exists**. **No vote-outcome reputation feedback exists** beyond the alignment delta.

**v0 snapshot scope** — `reputation_snapshot.community_id` is `Option<CommunityId>`, so the schema already supports both per-community (`Some`) and instance-wide (`None`) rows. v0 only writes per-community rows from the calculator; instance-wide rows would only ever be written by Phase 5c's `load_or_compute_snapshot(... community_id = None ...)` path — which exists but is not invoked from any v0 handler. This is the seam v1 uses for the rollup.

**Issue carry-forward** — five PR #10 review findings deferred to v1: #19 (community-scope `count_active_sanctions` mismatch), #20 (`check_snapshot_staleness` overflow + spurious empty-table error), #21 (drop `Copy` on `ReputationBuckets` / `AdminReputationStatsResponse`), #22 (bound `limit` on `list_capability_changed_entries_since`), #31 (explicit `active_sanctions` constructor on `ReputationSummaryView::From<&ReputationSnapshot>`). All five have intact code paths today and intact test coverage; v1 is the ratification window.

**OQ readiness** — [99 OQ-001](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), [OQ-019](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), [OQ-020](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) all opened with v1-leans documented. [OQ-018](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (admin config write endpoint) is the upstream dependency; this PRD assumes it lands with `POST /api/v4/governance/admin/config { key, value }` per its current lean.

---

## 3. ADRs That Govern This

| ADR | Summary | How it constrains us |
|---|---|---|
| [ADR-005](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Reputation is multi-dimensional + event-sourced; never a single score | **Load-bearing.** v1 must extend the dimension-wise decay surface, **not collapse** dimensions into a composite. Per-dimension decay is exactly what ADR-005 §Consequences anticipated when it said "Decay, tuning, and policy changes happen at the snapshot level without touching the event log." Event log stays append-only. |
| [ADR-010](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Staged releases v0 → v1 → v2 → v3 | This PRD targets v1 ("Is it production-grade governance?"). Per [05 §7.1](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md), v1 explicitly ships **"Reputation decay tuning per dimension"** and **"Instance-wide reputation roll-up"** — so this PRD is fulfilling a chartered-in v1 deliverable, not adding scope. |
| [ADR-008](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Append-only signed governance log | Every cron emit, every dashboard knob change, every rollup recomputation MUST emit a `governance_log` entry. v1 introduces `ENTRY_KIND_PARTICIPATION_CRON_TICK`, `ENTRY_KIND_VOTE_OUTCOME_RECORDED`, `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED`, `ENTRY_KIND_ROLLUP_RECOMPUTED`, `ENTRY_KIND_DECAY_KNOB_CHANGED` (admin attribution per Watch 11). |
| [ADR-013](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Illegal-content `EmergencyRemove` | Evidence-quality event source (§5.4) emits `−1` to reporter's `reporting_accuracy` only when admin flags an `EmergencyRemove` original report as bad-faith. The flag is admin-driven, not auto. Bad-faith reports of CSAM should NOT be auto-penalised — preserves the legal-compliance-first posture. |
| [ADR-015](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Pseudonymised actor IDs | All v1 cron emits and rollup writes log via `actor_pseudonym`, never raw `person_id`. Existing pattern in `governance_log::append` is preserved. |
| [99 OQ-001](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Instance-wide vs per-community reputation | Resolved-by-this-PRD: instance-wide rollup as a derived snapshot in `reputation_snapshot WHERE community_id IS NULL` rows. Lean confirmed. |
| [99 OQ-019](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | `participation_consistency` event sources | Resolved-by-this-PRD: weekly cron emits `+1` per active user per community; `−2` per dormant user; vote-outcome and evidence-quality event sources additive. Aligned with the OQ's "v1 ships option (a)" lean. Option (b) event-driven per-comment is **rejected** in this PRD per OQ rationale (spam incentive). |
| [99 OQ-020](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Sponsor gate-strategy expansion | Resolved-by-this-PRD: three additional strategies (`age_or_surety`, `reputation`, `allowlist`) ship in v1 with per-community config. Composition strategies (e.g. `age_or_surety_or_allowlist`) explicitly **rejected** per OQ — exponential test cost. |
| [99 OQ-021](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Negative-reputation floor + reintegration | **Out of scope here.** v1 introduces negative-decay rates but does NOT change the v0 zero-floor clamp on sponsor-liability ([99 OQ-024](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)). Negative-band-with-reintegration is v1-or-v2 successor PRD work. |

**Contradiction check**: None found. ADR-005's "decay, tuning, and policy changes happen at the snapshot level without touching the event log" is the exact license this PRD operates under — events stay append-only, snapshots are the v1 evolution surface.

---

## 4. Open Questions Carried Forward

| OQ | Status | Impact on this sub-PRD |
|---|---|---|
| [OQ-005](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (UX flows) | 99-register; v1 scope | Soft gate. Admin dashboard surface for tuning decay knobs needs a basic config UI — the OQ-018 endpoint shape (per-key write) is sufficient backend. PRD does NOT ship the dashboard UI; it ships the API. UI is operator-side or v3 admin-dashboard-UX. |
| [OQ-018](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (admin config write endpoint) | 99-register; v1 scope | **Hard gate** (Blocking-Dependencies §). Dashboard knob writes go through `POST /api/v4/governance/admin/config`. This PRD assumes that endpoint lands with the per-key shape currently leaned in OQ-018. If endpoint shape changes to batch writes, §6 dashboard surface is unaffected (per-key calls compose into a UI batch); only audit-log granularity changes. |
| [OQ-021](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (negative-reputation floor + reintegration) | 99-register; deferred | **Out of scope.** v1 negative-decay rate (§4.3) is a slow-decay-towards-zero, not a negative-band-with-reintegration. The OQ-024 v0 zero-floor clamp on sponsor-liability is preserved unchanged. |
| OQ-V1-01 (cron mechanism) | Opened here; resolved | In-app scheduler via clokwerk (existing Lemmy pattern in `crates/routes/src/utils/scheduled_tasks.rs:177`). NOT pg_cron. Preserves the no-postgres-extension posture. |
| OQ-V1-02 (banned-from-community rollup behaviour) | Opened here; lean documented | Lean: exclude that community from the rollup denominator entirely. Banned users contribute zero rep + zero weight. Alternative (include with zero weight) under-counts the rollup. Resolution required before §7 implementation. |
| OQ-V1-03 (negative-decay rate semantics) | Opened here; lean documented | Lean: slow time-decay (default `−5%/180d` half-life, vs positive `−10%/90d`) plus optional per-event corrective-action multiplier. Restorative principle preserved without making penalties evaporate quickly. |

### Open Questions Opened Here

- **OQ-V1-01 — Cron mechanism: in-app vs pg_cron.** Resolved 2026-04-19: in-app via clokwerk, mirroring the existing snapshot tick at `crates/routes/src/utils/scheduled_tasks.rs:177`. Rationale: pg_cron is a Postgres extension we have not adopted (cf. ADR-style "no extra extensions in v0 stack"), and Lemmy's existing scheduling pattern is well-tested. Trade-off: scheduler dies if the server process dies, but so does everything else.
- **OQ-V1-02 — Banned-from-community rollup behaviour.** Open. Lean: when computing the instance-wide rollup for a person who is banned from community C, exclude C from the rollup entirely (denominator drops by 1, not by `(1, weight=0)`). Rationale: a ban means "this community has rejected your participation" and should not pull the user's instance-wide score down via a zero-vote inclusion. Counterargument: excluding could let a user game the rollup by attracting bans. PRD assumes exclude-from-denominator; revisit at v1-implementation review.
- **OQ-V1-03 — Negative-decay rate semantics.** Open. Lean: negative reputation decays slower than positive (default `−5%/180d` half-life vs positive `−10%/90d`), AND a per-corrective-action accelerator can apply an extra positive event that offsets the penalty faster. This avoids the v0 simplification (negatives never decay → permanent outcasts on a long-enough timeline) without inventing a separate "reintegration ceremony" mechanic (which is OQ-021's territory). PRD assumes this lean; revisit at v1-implementation review.
- **OQ-V1-04 — Rollup recomputation cadence vs synchronous.** Resolved 2026-04-19: weekly cron, materialised. Rationale: rollup recomputation across all communities is expensive (N × M dimension sums); a live-aggregation endpoint would either be slow or require a denormalised cache. Weekly cadence aligns with the participation cron, and instance-admins use the rollup for cross-community eligibility queries that do not need real-time freshness.
- **OQ-V1-05 — Dashboard write granularity for decay rates.** Open. Single-knob writes (`POST /admin/config { key, value }`) per OQ-018's lean produce N audit-log entries for an N-knob change. A batch-write shape (`POST /admin/config { changes: [...] }`) would be one audit entry per batch but is also vetoed by OQ-018's lean ("batch writes are v1+ if pilot feedback says they're needed"). PRD aligns with OQ-018's lean: per-knob single writes; the audit-log granularity is the cost.

---

## 5. Proposed Solution

Promote v0's hardcoded reputation-decay stub into a **per-dimension, per-direction, configurable decay system** with multi-source `participation_consistency` events fed by two weekly crons (activity-positive + dormancy-negative) plus vote-outcome + evidence-quality reputation feedback paths. Add three additional sponsor-gate strategies. Materialise instance-wide reputation rollup snapshots via a third weekly cron that runs after the participation cron. All knobs surface through `governance_config` and the admin dashboard via OQ-018's `POST /api/v4/governance/admin/config`. Address five carry-forward CodeRabbit issues in the same v1 release window so the v1 reputation API stabilises in one cut.

v1 ships behind a feature flag (`feature.reputation_v1_decay_enabled`, default `false` at v1 release) so v0-style decay is preserved by default; instance operators opt in via dashboard once they trust the new defaults.

### 5.1 Reputation Dimension Catalog (verified against v0 enums)

The four dimensions per `crates/db_schema_file/src/enums.rs:589-594`:

| Dimension | Purpose ([01 §5.3](../../docs/brehon-law-inspired-network/01-vision-and-principles.md)) | v0 positive sources | v0 negative sources | v1 positive sources (new) | v1 negative sources (new) |
|---|---|---|---|---|---|
| `ReportingAccuracy` | Trust in flagging | `submit_jury_vote` upheld (default `+10`) | `submit_jury_vote` dismissed (default `−5`) | Evidence-quality event: `+1` if reporter's evidence cited in jury rationale | Evidence-quality event: `−1` if `EmergencyRemove` flagged bad-faith by admin |
| `JuryReliability` | Trust in decisions | `submit_jury_vote` aligned with majority (default `+10`) | `submit_jury_vote` outlier (default `−5`) | (none) | (none) |
| `ParticipationConsistency` | Engagement quality | `create_endorsement` (default `+5` to sponsee per [99 OQ-013](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)) | (none in v0) | Weekly activity cron `+1` per user per community with ≥1 comment that week. Vote-outcome cron `+1` per juror voted with majority post-decision (already exists for jury_reliability; v1 mirrors to participation). | Weekly dormancy cron `−2` for users dormant >30d in a community where they previously participated |
| `EndorsementStrength` | Trust in vouching | `create_endorsement` (default `+5` to sponsor per [99 OQ-013](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)) | `apply_sponsor_liability` (defaults `−10` minor / `−50` moderate / `−200` severe per Phase 5b task 56), zero-floor clamped per [99 OQ-024](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | (none) | (none) |

Source citations for every entry in the v0 columns trace to verifiable code paths (`create_endorsement.rs`, `submit_jury_vote.rs`, `sponsor_liability.rs`); the v1 columns are introduced by this PRD.

### 5.2 Decay configuration (per-dimension, per-direction)

v0 ships a single `decay.positive_half_life_days` (default `90`). v1 expands this to a `(dimension, direction)` matrix — eight knobs total:

| Knob | Default | Range | Per-community? | Notes |
|---|---|---|---|---|
| `decay.reporting_accuracy.positive_half_life_days` | 90 | 1–3650 | Yes | Mirrors v0 default |
| `decay.reporting_accuracy.negative_half_life_days` | 180 | 1–3650 OR `unbounded` | Yes | OQ-V1-03 lean |
| `decay.jury_reliability.positive_half_life_days` | 90 | 1–3650 | Yes | |
| `decay.jury_reliability.negative_half_life_days` | 180 | 1–3650 OR `unbounded` | Yes | |
| `decay.participation_consistency.positive_half_life_days` | 60 | 1–3650 | Yes | Faster than other dimensions: participation reputation reflects current behaviour, not lifetime |
| `decay.participation_consistency.negative_half_life_days` | 60 | 1–3650 OR `unbounded` | Yes | Symmetric with positive |
| `decay.endorsement_strength.positive_half_life_days` | 90 | 1–3650 | Yes | |
| `decay.endorsement_strength.negative_half_life_days` | 180 | 1–3650 OR `unbounded` | Yes | Slower: sponsorship penalties should not evaporate; OQ-024 honour-price principle |

**Calculation method: chained-halving exponential decay.** v0's single-halving stub becomes `delta * (0.5 ^ floor(age_days / half_life_days))` for organic events (`expires_at IS NULL`). Founder-cliff events (`expires_at IS NOT Null`) keep their full delta until the cliff, unchanged from v0 (Watch 8 from `reputation_snapshot.rs:30`).

Justification for exponential over linear: the governance-research literature on slowly-decaying reputation (Friedman & Resnick 2001, Sabater & Sierra 2005) favours exponential decay because it asymptotes towards zero without ever reaching it — a high-reputation user who goes inactive for years drifts towards baseline rather than crashing through it, and the gradient of consequence is smooth enough that a user can recover from a lapse without a cliff. Linear decay produces a hard zero at `age = (delta / decay_rate)` which, combined with the OQ-024 zero-floor clamp, would give a user a single-event removal point — operationally jarring.

**Floor and ceiling per dimension.** v0 has no per-dimension cap; raw deltas can sum to arbitrary values. v1 adds soft bounds to prevent grossly-out-of-range integer arithmetic and preserve the [01 §1 principle 5](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) "no permanent elites or outcasts" intent:

| Knob | Default | Range | Per-community? |
|---|---|---|---|
| `bounds.reporting_accuracy.floor` | -100 | -10000 to 0 | Yes |
| `bounds.reporting_accuracy.ceiling` | 100 | 0 to 10000 | Yes |
| `bounds.jury_reliability.floor` | -100 | -10000 to 0 | Yes |
| `bounds.jury_reliability.ceiling` | 100 | 0 to 10000 | Yes |
| `bounds.participation_consistency.floor` | -100 | -10000 to 0 | Yes |
| `bounds.participation_consistency.ceiling` | 100 | 0 to 10000 | Yes |
| `bounds.endorsement_strength.floor` | 0 | -10000 to 0 | Yes (locked at 0 unless OQ-021 ships) |
| `bounds.endorsement_strength.ceiling` | 200 | 0 to 10000 | Yes (higher because sponsor-event accumulator can reach `+5 * 5 = +25` quickly) |

Bounds apply at snapshot-write time in `recompute_snapshot` after the dimension sums are tallied. They do **not** mutate `reputation_event` rows — the event log stays append-only per ADR-008.

**Cron cadence.** Reputation snapshot recompute stays at 15-minute ticks (existing v0 behaviour at `crates/routes/src/utils/scheduled_tasks.rs:177`). Participation activity-positive + dormancy-negative crons run **weekly** (default; configurable via `job.participation_interval_days`, default `7`, range `1`–`30`). Rollup cron runs **weekly, after** the participation cron in the same scheduler tick to ensure rollup sees the freshest activity events.

### 5.3 Multi-source `participation_consistency` events (resolves [99 OQ-019](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md))

**Source 1 — Weekly activity cron.** Once per week (default), for each community, emit `+1 participation_consistency` per user with ≥1 non-deleted comment authored in that community in the lookback window. Knobs:

- `deltas.participation_weekly_active` — default `1`, range `0`–`10` (policy knob: the reputation delta for an active user)
- `participation.activity_threshold_comments` — default `1`, range `0`–`100` (context knob: what "active" means for this cron)
- `participation.lookback_days` — default `7`, range `1`–`90` (context knob: the activity window)

Idempotency: each emission writes a `reputation_event` row with `dedupe_key = format!("participation_cron:{community_id}:{iso_week}")` (new column on `reputation_event`, partial unique index where `dedupe_key IS NOT NULL`). Re-running the cron in the same ISO week is a no-op via `ON CONFLICT (dedupe_key) DO NOTHING`.

Atomicity: each community's batch writes inside a single transaction. A mid-community crash rolls back that community only; earlier committed communities stay. Per-community batching matches the v0 snapshot-batch-chunk pattern at `reputation_snapshot.rs:465-495`.

**Source 2 — Weekly dormancy cron.** Same scheduler tick, after the activity cron. For each community, for each `(person_id, community_id)` pair where the person previously had at least one `participation_consistency` event in that community AND has had zero non-deleted comments in the dormancy window, emit `−2 participation_consistency`. Knobs:

- `deltas.participation_dormant` — default `-2`, range `-100`–`0` (policy knob: the reputation delta for a dormant user)
- `participation.dormancy_window_days` — default `30`, range `1`–`365` (context knob: how long the user must be silent to qualify as dormant)

Same dedupe-key pattern: `format!("dormancy_cron:{community_id}:{person_id}:{iso_week}")`. Same per-community-tx atomicity.

**Source 3 — Vote-outcome event (post-decision, embedded in `submit_jury_vote`).** Already exists for `jury_reliability`. v1 mirrors: when the case is decided and a juror voted with the majority, emit `+1 participation_consistency` (same case_id, same juror, separate `reputation_event` row from the existing `jury_reliability` delta). When a juror voted in the minority, **emit nothing** — Brehon principle of no-penalty-for-dissent ([99 OQ-009 lean](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)). Knobs:

- `deltas.participation_juror_aligned` — default `+1`, range `0`–`100`

**Source 4 — Evidence-quality event (post-decision).** When a case is decided and the jury's rationale (free-text) cites the reporter's evidence (heuristic detection: case has `case_evidence` rows uploaded by the reporter AND the rationale length ≥ minimum threshold), emit `+1 reporting_accuracy` for the reporter. When the case is `EmergencyRemove`-status AND an admin flags the original report as bad-faith via a new admin action, emit `−1 reporting_accuracy` for the reporter. Knobs:

- `deltas.evidence_cited` — default `+1`, range `0`–`100`
- `deltas.evidence_bad_faith` — default `-1`, range `-100`–`0`
- `participation.evidence_cited_rationale_threshold_chars` — default `256`, range `32`–`4096` (context knob: the minimum rationale length that qualifies a case as "evidence cited")

The evidence-cited heuristic is a v1 simplification; v2 may add explicit "cite this evidence" UI. The bad-faith flag is admin-only (no auto-detection per ADR-013 legal-compliance posture).

**v1-impl verification task (per N2 2026-04-19):** before implementing this source, verify that `jury_vote.rationale` (or equivalent column) exists as a free-text `TEXT` field in v0 schema and is populated by `submit_jury_vote`. If rationale is currently empty-by-default or nullable-unused, the heuristic has no signal and Source 4 cannot ship until v2 (explicit evidence-cite UI). The `256 chars` default is a placeholder — tune during v1-impl based on observed rationale lengths from pilot data.

### 5.4 Sponsor gate strategies (resolves [99 OQ-020](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md))

v0 ships three strategies in `config.onboarding.sponsor_gate_strategy` per `crates/api/api/src/governance/config.rs:346`:

- `'age'` (default) — account age ≥ `onboarding.sponsor_min_account_age_days` (default `30d`)
- `'open'` — bypass all checks (recruitment-drive mode)
- `'closed'` — reject all endorsement attempts (emergency lockdown)

v1 adds three:

- `'age_or_surety'` — passes if age gate OR caller has ≥1 active surety (as sponsored party). Lets newly-vouched members sponsor before the age threshold. Implementation: `OR EXISTS (surety WHERE sponsored_id = caller AND revoked_at IS NULL)`.
- `'reputation'` — passes if `reputation_snapshot.can_sponsor` is true for the caller (community-scoped if `community_id` provided, else instance-scoped). The `can_sponsor` boolean already populates per Phase 5a task 53 and is gated against `thresholds.endorsement_strength` (default `25`). v1 simply flips this from a computed-but-unread boolean ([99 OQ-014](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)) to a strategy-readable one.
- `'allowlist'` — passes only if caller is in `sponsor_allowlist` table (new). Fields: `id`, `community_id` (nullable for instance-wide allowlist), `person_id`, `added_by_admin_id`, `added_at`, `note`. Admin-managed via new endpoints `POST /api/v4/governance/admin/sponsor-allowlist/add` + `POST /api/v4/governance/admin/sponsor-allowlist/remove`.

Strategy is configured **per-community** (existing `sponsor_gate_strategy` key already supports per-community scope via `governance_config_current.scope`). The strategy is a single string — composition strategies (e.g. `'age_or_surety_or_allowlist'`) **explicitly rejected** per [99 OQ-020](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) "every new combination is exponential test cost."

**Migration path for community switching strategies mid-flight.** When a community changes its `sponsor_gate_strategy` config, **existing endorsements are grandfathered** — no retroactive validation. Only new `create_endorsement` calls are evaluated against the new strategy. The dashboard surfaces a "this affects future endorsements only" tooltip on the strategy-change form. Existing `surety` and `endorsement` rows stay in their current state regardless.

### 5.5 Instance-wide reputation rollup (resolves [99 OQ-001](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md))

**Schema.** Reuse `reputation_snapshot` with `community_id IS NULL` for instance-wide rollup rows. v0 schema already supports this via `community_id: Option<CommunityId>`; v0 just never writes `None` rows. No migration needed.

**Computation.** Materialised via a third weekly cron (`reputation_rollup_cron`) that runs after the participation cron. For each person with at least one per-community snapshot:

```
rollup.dimension = sum(per_community_snapshot.dimension * weight) / sum(weight)
```

Default weight is `1` per community (equal weights; weighted average). Per [99 OQ-001 lean](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), per-community weights are tunable via a new config row but default to equal weights for v1 simplicity.

Per OQ-V1-02 lean, communities the person is banned from (active sanction with `target_community_id` set, scope `Community`) are **excluded from the denominator entirely** — they do not contribute zero rep at zero weight, they do not contribute at all. A person banned from C1 and active in C2 has rollup = C2's snapshot.

Rollup rows are upserted via the same `upsert_snapshot` machinery as per-community snapshots; capability-flip detection fires on rollup-row changes too (per Watch 11). Instance-wide capability-flips emit `governance_log::ENTRY_KIND_CAPABILITY_CHANGED` entries with `snapshot_community_id: null`.

**Use case 1 — Instance-admin views user's standing.** New endpoint `GET /api/v4/governance/admin/reputation/rollup?person_id=N` (instance-admin only) returns the rollup row plus per-community contributing snapshots.

**Use case 2 — Cross-community jury eligibility filters.** A v1 jury-mechanics feature (separate PRD, jury-mechanics-v1) reads the rollup to expand the eligible pool when a single community lacks enough qualified jurors. This PRD ships only the data; jury-pool expansion is jury-mechanics-v1 PRD scope.

### 5.6 Issue carry-forward (CodeRabbit findings #19, #20, #21, #22, #31)

| Issue | File | Fix |
|---|---|---|
| **#19** community-scope `count_active_sanctions` | `crates/db_views/reputation/src/impls.rs:42-53` and `crates/api/api/src/governance/get_my_reputation.rs:41` | Change `count_active_sanctions(conn, person_id)` signature to `count_active_sanctions(conn, person_id, community_id: Option<CommunityId>)`. Filter sanctions by community when `Some`; instance-wide when `None`. Thread `data.community_id` from the handler. |
| **#20** `check_snapshot_staleness` overflow + spurious empty-table | `crates/api/api/src/governance/reputation_snapshot.rs:435` | Replace `2 * interval_s` with `interval_s.saturating_mul(2)`. In the `None` arm, query `reputation_event::table.count()` first; if zero, emit `tracing::info!` (not error) — fresh-deploy posture. Reserve `error!` for `Some` arm where the table has data but is stale. |
| **#21** drop `Copy` on `ReputationBuckets` + `AdminReputationStatsResponse` | `crates/api/api_common/src/governance.rs:292, 336` | Drop `Copy` from both derives. No call sites require `Copy` semantics today (bucket arrays are small but not idiomatically copy-passed). Keeps forward-compat for future non-`Copy` fields. |
| **#22** bound `limit` on `list_capability_changed_entries_since` | `crates/db_views/governance_modlog/src/impls.rs:196` | Add `let limit = limit.min(MAX_LIMIT);` at function top; use clamped value in `.limit(limit)`. Match the pattern at `list_cases_filtered`. |
| **#31** explicit `active_sanctions` constructor | `crates/db_views/reputation/src/lib.rs` (the `From<&ReputationSnapshot>` impl) | Drop the `From` impl in favour of an explicit named constructor `ReputationSummaryView::from_snapshot_needing_sanction_count(&snapshot, active_sanctions: i64)`. The verbose name signals "not a complete view yet"; signature requires `active_sanctions` so callers cannot forget. Update the two known call sites (`get_my_reputation.rs:43`, plus any reputation view tests) to pass the count explicitly. |

All five fixes ship in the same v1 release window so the v1 reputation API stabilises in one cut. Each fix has its own GitHub issue and deserves a dedicated v1 PR (or a single bundled "v1 reputation API hardening" PR — author's call).

---

## 6. Acceptance Criteria

- All eight per-dimension/per-direction decay knobs surface in `governance_config` and are read by `recompute_snapshot` instead of the v0 single-knob path.
- A snapshot-recompute integration test demonstrates chained halving: an organic positive event of `delta = +100` aged `2 × half_life` produces `applied_delta = 25` (vs v0's `50`).
- A snapshot-recompute integration test demonstrates negative-direction half-life: an organic negative event of `delta = -20` aged `1 × negative_half_life` produces `applied_delta = -10` (vs v0's `-20`, never decays).
- Weekly participation activity cron emits `+1 participation_consistency` per active user per community, idempotent across re-runs in the same ISO week (verified via `dedupe_key` constraint).
- Weekly dormancy cron emits `−2 participation_consistency` per dormant user, idempotent across re-runs.
- Vote-outcome event source emits `+1 participation_consistency` per aligned juror at case-decided time (in the same `submit_jury_vote` transaction).
- Evidence-quality event source emits `+1 reporting_accuracy` when the heuristic detects rationale-cited evidence; admin-flagged bad-faith `EmergencyRemove` original report emits `−1 reporting_accuracy`.
- Three new sponsor-gate strategies (`age_or_surety`, `reputation`, `allowlist`) each have integration-test coverage demonstrating the gate fires correctly under matching conditions.
- Per-community strategy switch demonstrates grandfathering (existing endorsements pre-switch remain valid; only new `create_endorsement` calls are evaluated against the new strategy).
- Instance-wide rollup snapshot exists in `reputation_snapshot WHERE community_id IS NULL` for every person with ≥1 per-community snapshot, populated by the weekly rollup cron.
- `GET /api/v4/governance/admin/reputation/rollup` returns the rollup plus per-community contributing snapshots; rejects non-admin callers.
- Issue #19: `get_my_reputation` returns sanction count scoped to the requested `community_id` (community-scoped when `Some`, instance-wide when `None`).
- Issue #20: staleness check no longer emits spurious `error!` on a fresh-deploy empty table; no overflow on near-`i64::MAX` `interval_s`.
- Issue #21: `ReputationBuckets` and `AdminReputationStatsResponse` no longer derive `Copy`; build still passes; no regressions.
- Issue #22: `list_capability_changed_entries_since` rejects (or clamps) limits exceeding `MAX_LIMIT`.
- Issue #31: `From<&ReputationSnapshot>` impl removed; named constructor required at every call site; cargo fails compile if a future caller forgets `active_sanctions`.
- Feature flag `feature.reputation_v1_decay_enabled` defaults `false` at v1 release; flipping `true` activates per-dimension decay; flipping `false` reverts to v0 single-half-life behaviour without restart.
- Every decay-knob dashboard write produces one `governance_log` entry with `entry_kind = decay_knob_changed`, `actor_pseudonym` of the calling admin, payload `{key, old_value, new_value, scope, community_id?}`.

---

## 7. Cross-Cutting Impact

- [x] **Hash-chain governance log touched?** YES — **seven** new `ENTRY_KIND_*` constants added to `crates/api/api/src/governance/governance_log.rs` (defined in `crates/db_schema/src/source/governance/governance_log.rs` post-Phase-6 per DQ-6.6, re-exported via the api shim): `ENTRY_KIND_PARTICIPATION_CRON_TICK`, `ENTRY_KIND_VOTE_OUTCOME_RECORDED`, `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED`, `ENTRY_KIND_ROLLUP_RECOMPUTED`, `ENTRY_KIND_DECAY_KNOB_CHANGED`, `ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED` (§10 allowlist add), `ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED` (§10 allowlist remove). Zero migration per `governance_log.entry_kind` being TEXT. The allowlist pair covers the admin-owned `sponsor_allowlist` table introduced in §7 schema additions (this PRD owns them because the table is declared here; v1-AD-b consumes them via the admin-config write path).
- [x] **`actor_pseudonym` table or redaction service touched?** YES (additive) — every cron emit logs via `actor_pseudonym`. Cron-batch entries use a synthetic `system` pseudonym (new addition; reserved value, never collides with a real person's pseudonym). Rollup writes log under the rolled-up person's pseudonym.
- [x] **`CaseStatus::EmergencyRemove` affected?** YES — evidence-quality bad-faith path requires admin to flag an `EmergencyRemove` original report. New endpoint `POST /api/v4/governance/admin/emergency-remove/flag-bad-faith` (instance-admin only) writes a single `reputation_event` row + governance_log entry. ADR-013 admin-driven posture preserved.
- [x] **AGPLv3 notice / source disclosure affected?** None. No new external dependencies.
- [x] **Schema changes** — three additions: (1) new `dedupe_key TEXT` nullable column on `reputation_event` with partial unique index; (2) new `source_event_type` enum column on `reputation_event` (`Endorsement | JuryVote | SponsorLiability | FounderSeed | ParticipationCron | DormancyCron | VoteOutcome | EvidenceQuality | ManualSeed`) — defaults to `Endorsement` for backfill, populated by emitters going forward; (3) new `sponsor_allowlist` table (id, community_id NULL, person_id, added_by_admin_id, added_at, note).
- [x] **Backfill** — existing `reputation_event` rows get `dedupe_key = NULL` (only new cron events use it) and `source_event_type` default-mapped from existing `reason`/source columns via a one-off migration at v1 release.
- [x] **Threat-model table extension** — [06 §7](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) does not currently cover cron-emitted reputation events. v1-implementation phase adds rows for: (a) cron-based reputation farming (e.g. user gaming dormancy cron by writing one trivial comment per week to dodge the −2), (b) admin abuse of `flag-bad-faith` endpoint, (c) allowlist-table tampering.
- [x] **Federation impact** — none in v1. Cross-instance reputation portability is v2/v3.

---

## 8. Defaults Matrix (every new knob)

| Knob | Namespace | Type | Default | Range | Per-community? | Citation |
|---|---|---|---|---|---|---|
| `decay.reporting_accuracy.positive_half_life_days` | `decay.*` | int | 90 | 1–3650 | Yes | §5.2; mirrors v0 default |
| `decay.reporting_accuracy.negative_half_life_days` | `decay.*` | int OR text(`unbounded`) | 180 | 1–3650 | Yes | §5.2; OQ-V1-03 lean |
| `decay.jury_reliability.positive_half_life_days` | `decay.*` | int | 90 | 1–3650 | Yes | §5.2 |
| `decay.jury_reliability.negative_half_life_days` | `decay.*` | int OR text(`unbounded`) | 180 | 1–3650 | Yes | §5.2 |
| `decay.participation_consistency.positive_half_life_days` | `decay.*` | int | 60 | 1–3650 | Yes | §5.2; faster — current behaviour focus |
| `decay.participation_consistency.negative_half_life_days` | `decay.*` | int OR text(`unbounded`) | 60 | 1–3650 | Yes | §5.2; symmetric |
| `decay.endorsement_strength.positive_half_life_days` | `decay.*` | int | 90 | 1–3650 | Yes | §5.2 |
| `decay.endorsement_strength.negative_half_life_days` | `decay.*` | int OR text(`unbounded`) | 180 | 1–3650 | Yes | §5.2; honour-price principle |
| `bounds.reporting_accuracy.floor` | `bounds.*` | int | -100 | -10000–0 | Yes | §5.2 |
| `bounds.reporting_accuracy.ceiling` | `bounds.*` | int | 100 | 0–10000 | Yes | §5.2 |
| `bounds.jury_reliability.floor` | `bounds.*` | int | -100 | -10000–0 | Yes | §5.2 |
| `bounds.jury_reliability.ceiling` | `bounds.*` | int | 100 | 0–10000 | Yes | §5.2 |
| `bounds.participation_consistency.floor` | `bounds.*` | int | -100 | -10000–0 | Yes | §5.2 |
| `bounds.participation_consistency.ceiling` | `bounds.*` | int | 100 | 0–10000 | Yes | §5.2 |
| `bounds.endorsement_strength.floor` | `bounds.*` | int | 0 | -10000–0 | Yes | §5.2; OQ-024 floor preserved |
| `bounds.endorsement_strength.ceiling` | `bounds.*` | int | 200 | 0–10000 | Yes | §5.2 |
| `deltas.participation_weekly_active` | `deltas.*` | int | 1 | 0–10 | Yes | §5.3 source 1; policy knob |
| `participation.activity_threshold_comments` | `participation.*` | int | 1 | 0–100 | Yes | §5.3 source 1; context knob |
| `participation.lookback_days` | `participation.*` | int | 7 | 1–90 | Yes | §5.3 source 1; context knob |
| `deltas.participation_dormant` | `deltas.*` | int | -2 | -100–0 | Yes | §5.3 source 2; policy knob |
| `participation.dormancy_window_days` | `participation.*` | int | 30 | 1–365 | Yes | §5.3 source 2; context knob |
| `job.participation_interval_days` | `job.*` | int | 7 | 1–30 | No (instance) | §5.2 cadence |
| `job.rollup_interval_days` | `job.*` | int | 7 | 1–30 | No (instance) | §5.5 cadence |
| `deltas.participation_juror_aligned` | `deltas.*` | int | 1 | 0–100 | Yes | §5.3 source 3 |
| `deltas.evidence_cited` | `deltas.*` | int | 1 | 0–100 | Yes | §5.3 source 4 |
| `deltas.evidence_bad_faith` | `deltas.*` | int | -1 | -100–0 | Yes | §5.3 source 4 |
| `participation.evidence_cited_rationale_threshold_chars` | `participation.*` | int | 256 | 32–4096 | Yes | §5.3 source 4; N2 placeholder — tune per pilot data |
| `feature.reputation_v1_decay_enabled` | `feature.*` | bool | false | true/false | No (instance) | §10 feature flag |
| `job.rollup_equal_weights` | `job.*` | bool | true | true/false | No (instance) | §5.5 weighted-avg default |

**Total new knobs: 28** (plus extension of existing `onboarding.sponsor_gate_strategy` enum range to add three new string values).

---

## 9. Backwards Compatibility

- v0 reputation events remain valid. New `dedupe_key` and `source_event_type` columns are nullable / default-able; backfill populates `source_event_type` from `reason` heuristics + source FK presence.
- v0 single-half-life decay (`decay.positive_half_life_days`) **stays in `governance_config` as the legacy knob**. v1 calculator reads `feature.reputation_v1_decay_enabled`: if `false`, falls through to v0 path; if `true`, reads the new per-dimension/per-direction knobs.
- Migration plan: ship v1 with `feature.reputation_v1_decay_enabled = false`. Operators flip via dashboard after evaluating defaults against their pilot data. Reverting is a single boolean flip; no data loss because both code paths read the same `reputation_event` rows.
- Cutover checkpoint: at the end of v1 (call it v1.x), the legacy `decay.positive_half_life_days` knob and the v0 code path are deleted. Operators who have not flipped by then get a one-version-ahead deprecation warning.

### 9.1 ADR-010 compliance posture — feature flag vs snapshot columns (NOT3)

Three of the five v1 sub-PRDs use **snapshot columns** to satisfy ADR-010's no-retroactive-invalidation-of-in-flight-juries invariant: `v1-jury-mechanics.prd.md` snapshots `quorum_snapshot` / `threshold_count_snapshot` / `panel_size_snapshot` / `severity_tier_snapshot` onto `moderation_case`; `v1-sponsor-liability.prd.md` snapshots `grace_expires_at` and severity onto the case at the Pending transition; `v1-admin-dashboard.prd.md` reads `applied_config_snapshot` for in-flight cases under `requires_re_jury` keys. Each snapshot column captures an **in-flight procedural invariant** — a rule under which a specific jury started voting or a specific sponsor was first notified — and is immutable for the life of the procedure. This is the strongest possible compliance: not just "don't retroactively change the rules," but "the old rules are physically preserved on the row in question."

This PRD uses a **feature flag** (`feature.reputation_v1_decay_enabled`) instead of snapshot columns. That is a **weaker** compliance posture, and the weakening is deliberate. The reasons:

- **Reputation snapshots are a rolling derived view, not an in-flight procedure.** A `reputation_snapshot` row for a given (person, community) is recomputed from the append-only `reputation_event` log on a 15-minute tick. There is no single "a decision is in flight" moment that must be pinned; the snapshot is always a recomputed summary of what the events say.
- **ADR-010's textual invariant is about juries, not reputation.** The invariant reads "a config edit must not retroactively invalidate in-flight juries" (paraphrased). Reputation-decay parameter changes do change historical event contributions retroactively — but "historical events" is the wrong frame; what changes is the *computed delta* for each event the next time the snapshot is recomputed. The events themselves, append-only per ADR-008, do not move.
- **Per-event-source snapshotting is v2-scope.** To give reputation the same compliance strength as jury-mechanics, every `reputation_event` row would need to store the `applied_delta` as-of-write-time (not just the raw `delta`), and every recompute would need to honour that frozen value. That requires either schema churn (add `applied_delta_snapshot`) or versioned calculator logic (add `calculator_version` column and dispatch). Both are v2-scope per ADR-010's staged release cadence — v1 is the "production-grade governance" milestone, not the "retroactively-complete reputation audit trail" milestone.
- **Consequence users should understand.** With the feature flag, if an operator flips `feature.reputation_v1_decay_enabled = true` after some v0-era events have been emitted under v0 decay semantics, the next recompute applies v1 semantics to all events including those v0-era ones. This changes the snapshot. It does NOT change the underlying events. A user who wants to see "what my reputation was under v0 decay" can still compute it by re-reading the events through v0 logic — the events did not move, the lens moved.
- **Escape hatch for pilot operators.** The feature flag is a single boolean; flipping it back is free. Operators who see pilot data under v1 decay they don't like can revert. Snapshot columns cannot be reverted (the snapshot is what it is).

If pilot feedback during v1 surfaces that the weaker compliance is a problem, v2 can upgrade to the snapshot strategy via a migration that adds `applied_delta_snapshot` and populates it from existing event-source-type-conditioned re-computation. That is a forward-compatible path and nothing about v1's feature-flag posture forecloses it.

---

## 10. Security

- Every cron emit signs into `governance_log` via the existing instance key. `system` pseudonym used for cron-batch entries; reserved value, never assigned to a real person.
- Admin override of reputation events (e.g. revert a cron emit) is **not in v1 scope**. Reserved for v2 with step-up auth per [99 ADR-010 v2 milestone](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md). v1 admins may delete `governance_config` rows (reverting a knob change) but cannot directly mutate `reputation_event` rows.
- Audit-log every dashboard knob change: one `governance_log` entry per write per OQ-018's per-key shape (one knob = one entry). The OQ-V1-05 trade-off ("N audit entries for N knob changes") is accepted as the v1 cost; batch writes are v1+.
- `flag-bad-faith` endpoint: instance-admin only; emits `governance_log::ENTRY_KIND_EVIDENCE_QUALITY_RECORDED` with the admin's pseudonym, the original report's case_id, and the `−1 reporting_accuracy` event.
- `sponsor_allowlist` table: writes via admin endpoints only; both add and remove emit `governance_log` entries (`ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED`, `ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED`).

---

## 11. Implementation Phases (for follow-up `/prp-plan` runs)

| # | Phase | Description | Status | Depends | PRP Plan |
|---|---|---|---|---|---|
| 1 | v1.r1 — schema & feature flag | Add `dedupe_key`, `source_event_type` columns; add `sponsor_allowlist` table; seed 28 new config rows + feature flag; backfill existing `reputation_event` rows | pending | OQ-018 endpoint | - |
| 2 | v1.r2 — per-dimension chained-halving decay | Replace `compute_applied_delta` with per-dimension/per-direction calculator behind feature flag; add bounds clamping to `recompute_snapshot` | pending | v1.r1 | - |
| 3 | v1.r3 — multi-source participation events | Add weekly activity cron + dormancy cron in `scheduled_tasks.rs`; add vote-outcome + evidence-quality emitters in `submit_jury_vote.rs`; add `flag-bad-faith` admin endpoint | pending | v1.r2 | - |
| 4 | v1.r4 — sponsor-gate strategies | Add three new strategies to `create_endorsement` gate; add `sponsor_allowlist` admin endpoints | pending | v1.r1 (allowlist table) | - |
| 5 | v1.r5 — instance-wide rollup | Add weekly rollup cron; add `GET /admin/reputation/rollup` endpoint | pending | v1.r2 (decay must be live for rollup) | - |
| 6 | v1.r6 — issue carry-forward bundle | Fix issues #19, #20, #21, #22, #31 in a single PR; ship v1 reputation API stabilised | pending | independent of r2–r5; can land in parallel | - |

---

## 12. Blocking Dependencies

**Hard gates (must be true at v1 schedule time):**

- **OQ-018 admin config write endpoint exists.** All knob changes go through `POST /api/v4/governance/admin/config { key, value }` per its current lean. If endpoint shape changes to batch writes, the dashboard surface composes per-key calls into UI batches; only audit-log granularity is affected.
- **Phase 5/6 reputation events are stable.** v1 builds on the v0 `reputation_event` table shape; no v0/v1 split in the event log.
- **`scheduled_tasks::setup` accepts new clokwerk registrations.** Existing pattern at `crates/routes/src/utils/scheduled_tasks.rs:177-233` is the template; v1 adds three more `scheduler.every(...).run(...)` blocks following the same `RunningGuard` + concurrency-flag pattern.
- **`governance_log::ENTRY_KIND_*` is TEXT-keyed.** Verified at `governance_log.rs:50-68`; no migration for new entry kinds.

**Soft gates (nice to have):**

- **Pilot-week retro data** for tuning the 28 default values. The defaults in §8 are best-guesses; one pilot week of v0 data would let v1 tune them empirically before shipping. Not blocking; defaults can be revisited at v1.x.
- **OQ-005 UX flows** for the dashboard. v1 ships the API; the dashboard UI can be a v1 stretch goal or v3 admin-dashboard-UX milestone work.

---

## 13. Cross-references

- admin-dashboard-v1 PRD (sibling) — config surface for all 28 knobs; consumes OQ-018's `POST /admin/config` endpoint.
- jury-mechanics-v1 PRD (sibling) — juror eligibility filter consumes `reputation_snapshot WHERE community_id IS NULL` (rollup rows) when a community lacks enough qualified jurors.
- sponsor-liability-v1 PRD (sibling) — `apply_sponsor_liability` interacts with negative-decay rate; OQ-025 grace window is sponsor-liability scope, not this PRD's.
- v1 carry-forward issues: [#19](https://github.com/barrie-cork/lemmy/issues/19), [#20](https://github.com/barrie-cork/lemmy/issues/20), [#21](https://github.com/barrie-cork/lemmy/issues/21), [#22](https://github.com/barrie-cork/lemmy/issues/22), [#31](https://github.com/barrie-cork/lemmy/issues/31).
- [99-decisions-and-open-questions.md](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) — OQ-001, OQ-019, OQ-020 resolved by this PRD; OQ-021 deferred; new OQ-V1-01 through OQ-V1-05 opened.

---

## 14. Sources

Codebase (fork-local):

- `crates/db_schema/src/source/governance/reputation_event.rs:1-43` — v0 event row shape, includes `expires_at` for founder cliffs
- `crates/db_schema/src/source/governance/reputation_snapshot.rs:1-48` — v0 snapshot row shape, `community_id: Option<CommunityId>` already supports rollup
- `crates/db_views/reputation/src/impls.rs:1-280` — v0 view queries, `count_active_sanctions` mis-scoped per issue #19
- `crates/api/api/src/governance/get_my_reputation.rs:27-49` — v0 handler, uses `community_id` for snapshot but instance-wide for sanction count
- `crates/api/api/src/governance/admin_reputation_stats.rs:1-246` — v0 admin observability surface, `ReputationBuckets` derive `Copy` per issue #21
- `crates/api/api/src/governance/reputation_snapshot.rs:1-899` — v0 calculator, `compute_applied_delta` line 320 is the decay stub; `check_snapshot_staleness` line 423 has issues #20
- `crates/api/api/src/governance/config.rs:319-458` — 33 v0 config knobs, `decay.positive_half_life_days` is the only decay knob today
- `crates/api/api/src/governance/governance_log.rs:50-68` — v0 entry-kind constants, TEXT-keyed
- `crates/db_views/governance_modlog/src/impls.rs:196` — `list_capability_changed_entries_since` unclamped per issue #22
- `crates/api/api_common/src/governance.rs:292, 336` — `ReputationBuckets` + `AdminReputationStatsResponse` `Copy` derives per issue #21
- `crates/db_schema_file/src/enums.rs:589-594` — `ReputationDimension` enum, four variants
- `crates/routes/src/utils/scheduled_tasks.rs:166-233` — v0 scheduler pattern; v1 adds three more cron blocks following the `RunningGuard` concurrency-guard template

External:

- Friedman & Resnick (2001), "The Social Cost of Cheap Pseudonyms" — exponential-decay reputation foundations
- Sabater & Sierra (2005), "Review on Computational Trust and Reputation Models" — half-life and chained-halving justification

Design-doc anchors:

- [01-vision-and-principles.md §5.3](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) — four reputation dimensions
- [02-domain-model.md §4](../../docs/brehon-law-inspired-network/02-domain-model.md) — reputation as event-sourced
- [04-data-model-and-api.md §3](../../docs/brehon-law-inspired-network/04-data-model-and-api.md) — `reputation_event`, `reputation_snapshot` tables
- [05-mvp-and-delivery-plan.md §7.1](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) — v1 reputation deliverables ("decay tuning per dimension" + "instance-wide reputation roll-up")
- [99-decisions-and-open-questions.md OQ-001, OQ-019, OQ-020, OQ-024](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) — all resolved or referenced by this PRD

---

## 15. Resolutions applied (2026-04-19)

Cross-PRD coherence-audit edits applied during v1-PRD edit pass (see `.claude/PRPs/v1-planning-queue.json`):

| ID | Severity | Scope | Status |
|---|---|---|---|
| **B3** | blocking | Namespace collapse: `cron.*` / `rollup.*` / `reputation.*` collapsed into `job.*` / `deltas.*` / `participation.*` / `feature.*` to match v0 flat-namespace precedent and eliminate dual cron/job namespace for the same concept. 9 keys renamed per the table below. `bounds.*` preserved as first-class (floor/ceiling clamps are semantically distinct from `thresholds.*` capability cutoffs). | done |
| **N2** | non-blocking | §5.3 source 4 evidence-cited heuristic now names an explicit threshold `participation.evidence_cited_rationale_threshold_chars = 256` (added to §8 defaults matrix). v1-impl verification task added: verify `jury_vote.rationale` free-text column exists and is populated before this source can ship. If rationale is empty-by-default in v0, Source 4 defers to v2 (explicit evidence-cite UI). | done |
| **NOT3** | noted | §9.1 added: explicit framing of feature-flag posture as weaker-but-deliberate compared to the snapshot-column posture used by jury-mechanics / sponsor-liability / admin-dashboard. Documents (a) why reputation is a rolling derived view rather than an in-flight procedure, (b) ADR-010's textual scope is juries not reputation, (c) per-event-source snapshotting (`applied_delta_snapshot` / calculator versioning) is forward-compatible v2-scope, (d) the consequence user should understand when flipping the flag, (e) the v2 migration path remains open. | done |

### B3 key-rename table (authoritative)

| Old (draft) | New (final) | Rationale |
|---|---|---|
| `cron.participation.weekly_increment` | `deltas.participation_weekly_active` | Policy knob — reputation delta, not a cron knob |
| `cron.participation.activity_threshold_comments` | `participation.activity_threshold_comments` | Context knob — what "active" means |
| `cron.participation.lookback_days` | `participation.lookback_days` | Context knob — activity window |
| `cron.participation.weekly_dormancy_decrement` | `deltas.participation_dormant` | Policy knob — reputation delta, not a cron knob |
| `cron.participation.dormancy_window_days` | `participation.dormancy_window_days` | Context knob — how long before "dormant" |
| `cron.participation_interval_days` | `job.participation_interval_days` | Cadence knob → job.* matches v0 `job.snapshot_interval_seconds` |
| `cron.rollup_interval_days` | `job.rollup_interval_days` | Cadence knob → job.* |
| `rollup.equal_weights` | `job.rollup_equal_weights` | Rollup job config → job.* (eliminates single-key namespace bloat) |
| `reputation.v1.decay_enabled` | `feature.reputation_v1_decay_enabled` | Feature flag → feature.* (prophylactic for future deferred-enforcement toggles) |

### Carry-forward impact

- **reputation-tuning-v1 §5.2 cadence sentence** now reads `job.participation_interval_days` (was `cron.participation_interval_days`).
- **reputation-tuning-v1 §5.3 source-1 and source-2 bullets** use the new policy/context split (4 knobs renamed in place).
- **reputation-tuning-v1 §8 defaults matrix** reflects the final 15-namespace v1 inventory.
- **reputation-tuning-v1 §6 feature-flag acceptance criterion** reads `feature.reputation_v1_decay_enabled`.
- **reputation-tuning-v1 §9 backwards-compat** paragraph cites the same feature-flag key.
- **admin-dashboard-v1 §3.1 namespace registry** updated separately to the 15-namespace / 138-key total (see admin-dashboard-v1 §11 Resolutions).

Total v1 knobs in this PRD: **28** (unchanged; namespaces redistributed).

