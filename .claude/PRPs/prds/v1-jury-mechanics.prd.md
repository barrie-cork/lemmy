# Sub-PRD: V1 Jury Mechanics — Configurable, Severity-Aware, Diversity-Constrained

**Scope**: V1 release per [ADR-010](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md). Expands the v0 simplifications (5-juror panels, quorum 3, simple majority, no diversity, no severity thresholds) into a configurable, principled jury system. Adds appeal-jury, status-aware sizing, diversity constraints, and the severity tier mapping.
**Created**: 2026-04-19
**Status**: DRAFT
**Scheduled**: V1 release per ADR-010 §7.1 (Governance mechanics block). Sequenced after v0 ships (`docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md` §7.1).
**Predecessor**: v0 implementation in `crates/api/api/src/governance/{submit_jury_vote,admin_assign_jury,accept_jury_assignment,decline_jury_assignment}.rs`. v0 simplifications enumerated at [05 §3](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md). v1 targets at [01 §5.6](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) and [05 §7.1](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md).

---

## Problem Statement

v0 ships a 5-juror, quorum-3, simple-majority jury that decides every case identically regardless of how serious the alleged offence is. That was deliberate (ADR-007) — the v0 question was *does the mechanic work?*, not *is it production-grade governance?*. v0 also has no diversity constraints, no juror cooldown, no appeal jury, and no notion of "founder" cases needing larger panels. Every case gets a 5-person random-weighted panel from the eligible pool.

For v1 (production-grade governance), four real failure modes show up:

1. **Severity blindness.** A `Label` sanction and an `InstanceSuspension` are decided by the same 3-of-5 simple majority. There is no proportionality to the consequence — a brigading minority of three jurors can land an instance suspension on the same threshold as a content label.
2. **Cluster capture.** v0 has a one-hop sponsor-cluster conflict check on individual jurors but no panel-level diversity constraint — five jurors who all share the same sponsor are a perfectly valid panel.
3. **Appeals are toothless.** v0's appeal flips the case to `Appealed` and waits for `admin_close_case`. There is no second jury, no larger panel, no original-juror exclusion, and no bounded window. Issues #11, #14, #15 carry these forward.
4. **Hardcoded constants.** Panel size, quorum, deltas, cooldowns are partly config-driven (`jury.panel_size`, `jury.quorum`, `jury.max_concurrent_assignments`, etc.) but the *shape* of the jury (severity buckets, status awareness, diversity rules) is not configurable at all.

V1 transforms these from "hardcoded mechanic" into "configurable mechanic with sensible Brehon-derived defaults" — communities tune within a safe envelope per the user framing 2026-04-19. The admin dashboard surfaces every knob; defaults inherit Brehon-honour-price intent (severity proportionality, kin-group diversity, status-aware procedure).

**Primary actor** ([02 §2](../../docs/brehon-law-inspired-network/02-domain-model.md)): **Juror Eligible / Juror**, plus the **Defendant** (whose case severity determines panel shape) and the **Instance Admin** (whose dashboard knobs decide community defaults).

---

## Evidence

- **v0 simplifications enumerated** at [05 §3](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) explicitly list every v0 deferral with the v1 target alongside it. The four v1 jury items appear verbatim at [05 §7.1](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md): jury size 5→7, severity-based voting thresholds, jury diversity constraints in selection, appeals with larger different jury (original jurors excluded).
- **Brehon honour-price origin** of severity tiers at [01 §3](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) principle 1: "Honor price — every person had a 'status value' they could lose" → "high-trust users suffer bigger consequences when wrong." [01 §5.6](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) tabulates the v1 voting thresholds: Minor=simple majority, Moderate=60%, Severe=75% supermajority. [01 §5.7](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) tabulates appeals: one guaranteed appeal, larger jury, previous jurors excluded.
- **OQ-026 status-aware extension pattern** at [99 OQ-026](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) explicitly anticipates the v1 use case: `jury.panel_size.founder vs jury.panel_size.regular`. Current lean: option (a) keyed-by-status rows with dotted namespace + reader-side cascade; v0 schema unchanged.
- **OQ-004 resolved 2026-04-17** at [99 OQ-004](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md): concurrent cap = 3 instance-wide via `config.jury.max_concurrent_assignments`. v1 makes this per-community-overrideable and adds an optional per-juror cap.
- **OQ-009 juror anonymity** at [99 OQ-009](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md): current lean is "anonymous to target/public always; revealed to each other after accepting; aggregate count public." v1 must address this concretely.
- **GitHub issues** carry forward the four appeal/jury items from the Phase 5c carry-forward triage:
  - #11 — original-reporter appeals (`request_appeal` rejects anyone except the sanction target).
  - #14 — re-jury path for `Appealed` cases (v0 has no re-jury; case sits until admin closes).
  - #15 — formalise appeal window with bounded duration (v0 defines window as `case.closed_at IS NULL`).
  - #19 — community-scope `count_active_sanctions` in `get_my_reputation` (intersects with reputation-tuning v1, not jury-mechanics v1; cross-referenced).
  - #29 — config-flip determinism when eligible pool > panel size (intersects with diversity selection algorithm).
- **v0 implementation truth** verified in `crates/api/api/src/governance/`:
  - `admin_assign_jury.rs:120` reads `jury.panel_size` from `governance_config` (already config-driven per Phase 5b task 57). The reputation gate at `:215-274` (`select_eligible_jurors`) reads `jury.max_concurrent_assignments` and `jury.fallback_on_small_pool`. v1 extends this surface, does not rewrite it.
  - `submit_jury_vote.rs:85,87` hardcodes `QUORUM = 3` and `APPEAL_WINDOW_DAYS = 7` as Rust consts. v1 promotes both to config + per-case-snapshot semantics per ADR-010 (no retroactive invalidation).
  - `request_appeal.rs:108` checks `case.closed_at.is_some()` as the appeal-window guard. v1 introduces explicit `appeal_window_expires_at`.
  - `accept_jury_assignment.rs:120` uses `shares_active_sponsor` from `jury_common.rs` (one-hop only). v1 adds panel-level diversity (no-majority-from-same-sponsor-cluster).
  - `sponsor_liability.rs:112-124` already has a `severity_for_action` mapping (Minor/Moderate/Severe) for sponsor-liability-delta lookup. **v1 reuses this mapping** for jury voting thresholds (single source of truth).

---

## ADRs That Govern This

| ADR | Summary | How it constrains us |
|---|---|---|
| [ADR-005](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Reputation is multi-dimensional, event-sourced | Jury-eligibility weighting reads `jury_reliability` per existing `reputation_snapshot.jury_eligible` flag. v1 jury-selection extension adds NO new reputation dimensions. |
| [ADR-007](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | MVP uses 5-juror, quorum-3, simple-majority | v1 supersedes ADR-007's *defaults*; the *machinery* (config-driven panel size) is preserved. v0 cases in flight when v1 ships still complete under ADR-007 rules — **see §11 backwards-compat**. |
| [ADR-008](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Append-only signed governance log | Every config-relaxation (constraint relaxed because pool too small), severity reassignment (case-open time only), appeal-window decision, and appeal-rejury action emits a `governance_log` entry. |
| [ADR-010](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Staged releases v0 → v1 → v2 → v3 | **Hard rule**: config changes MUST NOT retroactively invalidate in-flight juries. v1 panel size / quorum / threshold are snapshotted onto the case at jury-pick time, NOT re-read from config at vote-time. See §8 schema and §9 handler changes. |
| [ADR-013](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Illegal-content emergency-remove | The post-facto jury for `EmergencyRemove` cases sizes and tiers per the same v1 machinery; severity = `Severe` by default (admin override is a mid-case status change, disallowed by ADR-010). |
| [ADR-015](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Pseudonymised actor IDs | All `governance_log` entries from this PRD use `pseudonym`, never `person_id` / username — same discipline as `submit_jury_vote.rs` and `sponsor_liability.rs` today. |

**Contradiction check**: None found. The v1 jury parameters are explicitly the upgrade path foreshadowed by ADR-007 → ADR-010 §7.1. Backwards-compat with in-flight v0 cases is enforced at the schema level (§8 panel-size snapshot column) per ADR-010.

---

## Open Questions Carried Forward & Newly Opened

**Resolved by this PRD (proposed):**

| OQ | Resolution proposed here | Migration path if accepted |
|---|---|---|
| **OQ-009** (juror anonymity in decision phase) | v1 commits: anonymous to target + public always; revealed to each other after accepting (matches the current lean). Concrete rendering: jury-room participant identifier = `Juror-<actor_pseudonym 4-char suffix>`. Aggregate count public per [02 §9](../../docs/brehon-law-inspired-network/02-domain-model.md). | Add to [02 §9 transparency boundary](../../docs/brehon-law-inspired-network/02-domain-model.md) at v1 ship; no schema change. |
| **OQ-026** (status-aware rule extension) | Accept Option (a) — keyed-by-status dotted namespace `jury.panel_size.<status>.<severity>` in `governance_config`. v1 schema unchanged for `governance_config`; new keys only. | New rows in the Phase 5a config-seed migration; new `pub const` fallbacks in `crates/api/api/src/governance/config.rs`. |

**Newly opened (this PRD):**

| OQ | Question | v1 / v1.5 / parked |
|---|---|---|
| **OQ-V1-JM-01** — Severity tier inference | Who/what determines a case's severity tier — the reporter (capability-tagged), the alleged sanction shape, an admin overrider, or a content classifier? | v1 lean: reporter-suggested + admin-overridable at case-open time. Frozen on transition to `JurySelection`. See §3.2. |
| **OQ-V1-JM-02** — Geographic diversity heuristic | Which signal proxies "geographic diversity" — `actor.timezone`, IP-region inference, declared community, or none? | v1 lean: declared community for cross-community jurors (where multi-community); IP-region inference rejected as PII surface; timezone deferred to v1.5 pending UX research. Constraint default = ON-effort (best-effort, not hard fail). |
| **OQ-V1-JM-03** — Founder-status threshold for case-status determination | Does "founder case" mean the *defendant* is a founder, the *case-class* is founder-class, or both? | v1 lean: defendant status — a case with `target_person_id` of a founder (per `reputation_event.reason='founder_seed'` unexpired) gets `case_status_tier = Founder`. Case-class is a v1.5 question pending pilot data. |
| **OQ-V1-JM-04** — Cross-instance juror eligibility | Can a juror on instance A serve on a case on instance B? | **Parked, v2 territory** per ADR-014 (federation interop limits). Flag don't decide. v1 commits: jurors are per-community; per-community is bounded to a single instance for v1. |
| **OQ-V1-JM-05** — Re-jury auto vs admin trigger | When a case enters `Appealed`, does the appeal panel get auto-selected, or wait for admin trigger? | v1 lean: admin-triggered with default `appeal.auto_select_on_appeal_acceptance = true`. Communities can flip the bool to require manual admin assembly. See §6. |
| **OQ-V1-JM-06** — Original-reporter appeal-rights scope | Issue #11 says original-reporter appeals; what is the scope? Reporter who appeals a `NoAction` decision? Reporter who appeals a too-light sanction? Both? | v1 lean: reporter-of-record can appeal **only** if decision was `NoAction` or `Label` (the lightest two outcomes), and only within the same appeal window as the defendant. Stricter scopes are v2. |

---

## §1 — Vision and goals

**Brehon honour-price → severity tier mapping** (cite [01 §3 principle 1](../../docs/brehon-law-inspired-network/01-vision-and-principles.md)):

> Every person had a "status value" they could lose. High-trust users suffer bigger consequences when wrong.

Translated to v1 jury mechanics:

- **Severity tiers** mirror the magnitude of the consequence the jury can impose. A `Label` is an annotation; an `InstanceSuspension` excludes the user from the platform. The procedural threshold for the latter must be higher.
- **Status awareness** mirrors that the same offence by a high-trust user (founder, long-tenured Trusted Member) carries a larger reputation hit per [01 §5.2 honour-price floor footnote](../../docs/brehon-law-inspired-network/01-vision-and-principles.md). Cases against founders are higher-stakes; v1 sizes their panels accordingly.
- **Diversity constraints** mirror the kin-group (`fine`) principle from [01 §4 principle 3](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) — moderation starts local but must not concentrate in a single kin/sponsor cluster.

**Configurable but principled** — defaults in this PRD are **starting points**, not gospel. The admin dashboard (cross-referenced) lets community groups experiment within a safe envelope: panel-size range 3–11, quorum 50%–100% of panel, threshold 50%–100% of votes. Defaults inherit Brehon-derived intent; communities tune within these bounds.

**No retroactive invalidation** — ADR-010 hard rule. Once a jury is picked, that case's panel-size/quorum/threshold are *snapshotted* onto the case row. Subsequent config changes cannot retroactively change the rules under which a vote will be counted. This is critical for procedural legitimacy: a jury that began deliberating under a 60% threshold cannot have its threshold raised to 75% by a mid-case admin config change.

---

## §2 — Scope (in / out)

**IN scope for v1:**

- Severity-tier enum (`Minor | Moderate | Severe`) and a `severity_for_action(SanctionAction) → SeverityTier` mapping (extending the v0 `severity_for_action` already in `sponsor_liability.rs:112-124`).
- Per-tier panel size, quorum, and voting threshold — all configurable, all defaulted from this PRD.
- Status-aware panel sizing (`Founder | Regular | Probation`) using OQ-026's dotted namespace.
- Jury diversity constraints in the `select_eligible_jurors` selection algorithm:
  - `no_majority_from_same_sponsor_cluster` (ON by default).
  - `geographic_diversity_preferred` (ON-effort by default; soft constraint).
  - `no_recent_juror_repeat` (ON by default; default cooldown 7 days, configurable).
- Appeal jury — different + larger + original-jurors-excluded + higher-threshold-tier.
- Original-reporter appeal-rights for `JuryDecision::NoAction` / `JuryDecision::AdvisoryLabel` decisions (issue #11, scoped per OQ-V1-JM-06). **Terminology note:** `AdvisoryLabel` is the `JuryDecision` enum variant (see `crates/db_schema_file/src/enums.rs:502`); `Label` is the `SanctionAction` it maps to (see `crates/api/api/src/governance/submit_jury_vote.rs:441`). Throughout this PRD, `AdvisoryLabel` refers to the decision; `Label` refers to the downstream sanction action.
- Bounded appeal window (issue #15) with explicit `appeal_window_expires_at` column.
- Re-jury path for `Appealed` cases (issue #14) — admin-triggered with default auto-select-on-appeal-acceptance.
- Per-community concurrent-cap override + optional per-juror cap.
- Constraint-relaxation audit log (which constraint was relaxed and why, when pool was too small).

**OUT of scope for v1 (deferred):**

- **Instance-wide juries** — always per-community. No "instance jury pool" sampled across communities. v0 reputation snapshot already supports both per-community (`community_id IS NOT NULL`) and instance-wide (`community_id IS NULL`); v1 keeps jury selection at the community level. Cross-community pool is v2.
- **Cross-instance jury** (federated jury). v2 territory per ADR-014; OQ-V1-JM-04 flags don't decide.
- **Juror identity reveal post-decision.** v0/v1 keeps jurors pseudonymous to the public always per ADR-015. v2/governance-research can revisit if pilot communities request it.
- **Composable diversity constraints.** `no_majority_from_same_sponsor_cluster AND no_majority_from_same_endorsement_chain AND geographic_diversity_preferred AND ...` is a v1.5 composition surface. v1 ships each constraint as an independent boolean toggle; v1.5 may compose.
- **Severity tier mid-case change.** Disallowed by ADR-010 (would effectively invalidate an in-flight jury). Severity is set at case-open time and frozen on transition to `JurySelection`.
- **`no_same_endorsement_chain`** constraint. Default OFF in v1; needs deeper graph analysis (multi-hop endorsement traversal). v1.5.
- **Severity-aware jury notification UX** (separate notification-routing PRD; OQ-005 territory).

---

## §3 — Severity tiers and thresholds

### §3.1 Tier enum and mapping

Three tiers, defined as a new Diesel-backed enum in `crates/db_schema_file/src/enums.rs`:

```rust
pub enum SeverityTier {
    Minor,
    Moderate,
    Severe,
}
```

The mapping `severity_for(sanction: &SanctionAction) -> SeverityTier` extends the existing function in `crates/api/api/src/governance/sponsor_liability.rs:112-124`. The two callers (sponsor-liability deltas and v1 jury thresholds) share the same source-of-truth function.

| `SanctionAction` | Severity tier |
|---|---|
| `Label` | Minor |
| `VisibilityReduction` | Minor |
| `Restoration` | Minor (per OQ-003 amendment 2026-04-17 — restorative actions reserved as Minor) |
| `TemporaryRestriction` | Moderate |
| `ContentRemoval` | Moderate |
| `CommunityExclusion` | Severe |
| `InstanceSuspension` | Severe |
| `FederationQuarantineRecommendation` | Severe |

This is the v0 mapping; v1 does not change it. v1 *adds* the per-tier panel/quorum/threshold semantics on top of the existing severity bucket.

### §3.2 Severity assignment timing — the "case severity is known before sanction shape" problem

**Tension**: The jury is *picked* at case-open time, but the *sanction action* (which determines severity per §3.1) is only chosen by the jury at vote-tally time. So the panel must be sized *before* the severity is fully knowable.

**v1 resolution**: Severity is a property of the **case**, set at case-open time, independent of which sanction the jury eventually picks. Two parallel resolutions, with the PRD picking the first:

1. **(Chosen)** Severity is reporter-suggested + admin-overridable at case-open time. The reporter's `reason_code` carries an implicit severity (e.g. `harassment_severe` → Severe; `off_topic` → Minor). An admin or capability-tagged moderator can override. Once the case transitions to `JurySelection` (admin_assign_jury fires), the severity tier is **frozen** and snapshotted onto `moderation_case.severity_tier_snapshot` (see §8). This matches the OQ-V1-JM-01 lean.
2. **(Rejected)** Panel sized for max-severity-possible. Always pick a Severe-tier panel; if jury votes for a Minor sanction, the threshold drops to Minor. Rejected because it inflates panel sizes (and juror cooldown depletion) for cases where the actual outcome turns out to be a `Label`.

**Edge case — emergency-remove**: `admin_emergency_remove` (existing `crates/api/api/src/governance/admin_emergency_remove.rs:154`) opens a case with `severity = CaseSeverity::default()` (Medium) today. v1 sets `severity_tier_snapshot = Severe` for emergency-remove cases by default, with the rationale that the admin's invocation already constitutes a Severe-level action. The post-facto jury reviews under Severe-tier rules.

### §3.3 Per-tier defaults

Defaults are **starting points** — every value is configurable per-community via the dashboard.

| Tier | Default panel size (regular) | Default quorum | Default threshold |
|---|---|---|---|
| Minor | 5 | 3 (60% of panel) | Simple majority (>50%) |
| Moderate | 5 | 3 (60% of panel) | 60% supermajority |
| Severe | 7 | 5 (~71% of panel) | 75% supermajority |

These defaults map directly onto [01 §5.6](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) which is the canonical v1 target table.

### §3.4 Configurability surface

Every tier × parameter combination is a config key in `governance_config`. The dotted namespace per OQ-026 / Phase 5a precedent:

```text
jury.panel_size.regular.minor       (int, default 5)
jury.panel_size.regular.moderate    (int, default 5)
jury.panel_size.regular.severe      (int, default 7)
jury.panel_size.founder.minor       (int, default 5)
jury.panel_size.founder.moderate    (int, default 7)
jury.panel_size.founder.severe      (int, default 9)
jury.panel_size.probation.minor     (int, default 3)
jury.panel_size.probation.moderate  (int, default 5)
jury.panel_size.probation.severe    (int, default 5)

jury.quorum_fraction.minor          (float, default 0.6)
jury.quorum_fraction.moderate       (float, default 0.6)
jury.quorum_fraction.severe         (float, default 0.71)

jury.threshold_fraction.minor       (float, default 0.5001)   // simple majority — strict >50%
jury.threshold_fraction.moderate    (float, default 0.6)
jury.threshold_fraction.severe      (float, default 0.75)
```

**Bounds enforcement** (admin-dashboard PRD must enforce these on write):

- `panel_size`: integer in `[3, 11]`, must be odd (avoids deadlocks).
- `quorum_fraction`: float in `[0.5, 1.0]`, computed `quorum = ceil(panel_size * quorum_fraction)`.
- `threshold_fraction`: float in `[0.5001, 1.0]`. `0.5001` is the canonical "simple majority" representation.

Quorum and threshold are stored as **fractions** in config but **resolved to integer counts** at jury-pick time by `ceil(panel_size * fraction)`, then snapshotted onto the case row (see §8).

### §3.5 Cascade fallback (per B2 2026-04-19)

The 9-cell matrix coexists with v0's bare `jury.panel_size` key — the reader cascade operates in this order:

1. `jury.panel_size.<status>.<severity>` — most specific (this PRD's new keys)
2. `jury.panel_size.<severity>` — per-severity fallback (reserved; not seeded in v1)
3. `jury.panel_size` — v0-compatible bare key (admin-dashboard-v1 §5.1 default = 7)
4. Rust const `DEFAULT_JURY_PANEL_SIZE` — hardcoded final fallback

This preserves v0 behaviour for existing call sites (e.g. `admin_assign_jury.rs:120`) during v1 rollout — communities can set a single number for everything (write `jury.panel_size = 9`) or tune per status × severity. Provenance on reads returns the full matched-key path so dashboard UX can show which cell in the cascade resolved the value (extends admin-dashboard §3.3's `effective_from` field).

---

## §4 — Jury sizing and status awareness

### §4.1 Status enum

Define `CaseStatusTier` (named to disambiguate from `CaseStatus` enum already at `crates/db_schema_file/src/enums.rs`):

```rust
pub enum CaseStatusTier {
    Founder,
    Regular,
    Probation,
}
```

### §4.2 Status determination

Per OQ-V1-JM-03 lean:

- **Founder**: `target_person_id` has at least one unexpired `reputation_event WHERE reason = 'founder_seed' AND expires_at > now()` (matches the existing `is_founder` check in `sponsor_liability.rs:217-228`).
- **Probation**: `target_person_id` has `person.membership_state = 'provisional'` (per the Phase 5a `MembershipState` enum already shipped).
- **Regular**: neither of the above.

For cases with no `target_person_id` (post/comment/community-targeted), default to **Regular**.

### §4.3 Status × severity sizing

Resolution order for `panel_size`:

1. Read `jury.panel_size.<status>.<severity>` from `governance_config` (community scope first, instance scope second per OQ-026 cascade).
2. Fall back to Rust const default (declared in `crates/api/api/src/governance/config.rs`).

**Probation cases use smaller panels by default** to enable faster turnaround for low-stakes provisional-member cases. **Founder cases use larger panels by default** because the consequences of getting it wrong (sanctioning a founder) are higher per [01 §5.2 honour-price footnote](../../docs/brehon-law-inspired-network/01-vision-and-principles.md).

### §4.4 Quorum and threshold inherit severity, not status

To bound the configuration combinatorics, **quorum and threshold are functions of severity only, not status**. A founder Severe case uses a 9-juror panel with quorum=`ceil(9*0.71)=7` and threshold=`ceil(7*0.75)=6`. A regular Severe case uses a 7-juror panel with quorum=5 and threshold=4.

---

## §5 — Diversity constraints

### §5.1 Constraint catalog

Each constraint is a per-community boolean toggle in `governance_config`. Selection algorithm (§5.3) consults each enabled constraint.

| Constraint key | Default | Hard / Soft | Rationale |
|---|---|---|---|
| `jury.constraints.no_majority_from_same_sponsor_cluster` | ON | Hard (re-roll if violated) | Trace via `surety` table; >50% of panel sharing a sponsor would replicate v0's per-juror cluster check at the panel level. Maps to [01 §4 principle 3](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) kin-group diversity. |
| `jury.constraints.geographic_diversity_preferred` | ON-effort | Soft (best-effort, not hard fail) | Uses `community_id` declared affiliation; multi-community jurors preferred. Per OQ-V1-JM-02 lean — IP region rejected, timezone deferred. |
| `jury.constraints.no_recent_juror_repeat` | ON | Hard | Prevents a small set of jurors from dominating community moderation. Default cooldown 7 days, configurable as `jury.constraints.juror_cooldown_days` (int, default 7). |
| `jury.constraints.no_same_endorsement_chain` | OFF | Hard (when ON) | v1.5 — needs multi-hop graph analysis. Schema is in place (`endorsement` table) but query cost untested. |

### §5.2 Cooldown semantics

`no_recent_juror_repeat` excludes jurors who have a `jury_assignment` row with `responded_at > now() - cooldown_days * INTERVAL '1 day'` AND `status IN (Accepted, Submitted)` — i.e. they actively served (not declined) recently. The cooldown is per-community by default.

### §5.3 Selection algorithm

Extends `select_eligible_jurors` in `crates/api/api/src/governance/admin_assign_jury.rs:215-274`. The current strict-eligibility query at `:300-338` gets a two-phase wrapper. **Each constraint applies at a specific phase** (pool-build filter vs. panel-sample check vs. soft score) — the phase determines the relaxation semantics, so the distinction is load-bearing.

```text
PHASE 1 — pool build (filter)
  build base eligible pool (existing v0 reputation gate + concurrent cap)
    ↓
  apply `no_recent_juror_repeat` as SQL WHERE clause
    (EXCLUDES recent jurors from the pool; the pool's size is the post-filter size)
    ↓
  IF pool_size < panel_size:
    → relaxation step R1 (see cascade below); re-run pool build.

PHASE 2 — panel sample (re-roll)
  weighted-random sample `panel_size` candidates from the pool
    ↓
  check `no_majority_from_same_sponsor_cluster` on the SAMPLE:
    ├── violated → re-roll (up to N_RETRIES, default 5)
    └── satisfied → continue
    ↓
  IF still violated after N_RETRIES:
    → relaxation step R2 (see cascade below).

PHASE 3 — panel score (soft)
  score the panel on `geographic_diversity_preferred` (weighted sum of unique countries)
    ↓
  bias sampling towards higher-scoring panels in Phase 2's re-roll loop
    (soft — a panel with low diversity still returns; it just lost the coin toss against
     a higher-diversity alternative during re-roll)
```

**Relaxation cascade (clearer intent than v0's "drop in priority order"):**

| Step | Trigger | Action | Audit |
|---|---|---|---|
| **R1** | Phase 1: pool is too small post-cooldown to fill `panel_size`. | Drop `no_recent_juror_repeat` (expand the pool by lifting the cooldown WHERE clause) and re-run pool build. This is the cheapest relaxation — it expands the pool without compromising sponsor-cluster independence. | Emit `jury_constraint_relaxed` with `(case_id, constraint_dropped='no_recent_juror_repeat', reason='small_pool', phase='pool_build')`. |
| **R2** | Phase 2: after N_RETRIES re-rolls the sample still violates `no_majority_from_same_sponsor_cluster`. | Drop `geographic_diversity_preferred` as a soft scoring input (so re-rolls become purely random rather than diversity-biased, which often helps break cluster ties) and re-try N_RETRIES re-rolls. | Emit `jury_constraint_relaxed` with `(case_id, constraint_dropped='geographic_diversity_preferred', reason='cluster_pressure', phase='panel_sample')`. |
| **R3** | Phase 2 still fails after R2. | Drop `no_majority_from_same_sponsor_cluster` entirely and return the best-effort sample. This is the **last resort** because sponsor-cluster is the most Brehon-essential constraint. | Emit `jury_constraint_relaxed` with `(case_id, constraint_dropped='no_majority_from_same_sponsor_cluster', reason='cluster_pressure_exhausted', phase='panel_sample')` **AND** `tracing::warn!("jury selection dropped sponsor-cluster constraint on case {case_id}")`. |

**Rationale for the order:** cooldown at pool-build is a per-user exclusion (lifting it just adds users); geographic diversity is already declared soft so removing its bias is a natural intermediate step; sponsor-cluster is the only constraint that prevents structural capture of the jury by a single sponsor network, so it sits at the bottom of the relaxation stack and its removal is both logged and `warn!`-surfaced.

Each relaxation writes both a `governance_log` entry of kind `jury_constraint_relaxed` AND a `jury_constraint_violation_log` row (separate audit table per §9.2). Both are audit-visible to community admin via the modlog query; transparency replaces strict enforcement when the pool cannot support the constraint stack.

If after R3 the pool is still smaller than `panel_size` and `jury.fallback_on_small_pool = true` (existing config), fall through to the v0 unfiltered legacy path at `admin_assign_jury.rs:349-370` (already implemented). If `fallback_on_small_pool = false`, return `NotFound` so the admin learns the pool is too small.

### §5.4 Determinism note (cross-ref issue #29)

Selection randomness is `ORDER BY random()` (existing `:328`). Issue #29 flags non-determinism when `eligible_pool_size > panel_size` makes config-flip assertions flaky. v1 does **not** make selection deterministic (random selection is the principle), but the v1 audit-log shape (which jurors were excluded, which constraints were relaxed) makes test assertions checkable on stable derived properties.

---

## §6 — Appeals (v1 first-class)

### §6.1 Appeal panel sizing

Default: appeal panel is **1.5× original panel size, rounded up**, with a minimum of `original_panel_size + 2`. So a 5-panel original gets a 7-juror appeal panel; a 7-panel original gets an 11-juror appeal panel (capped at the bounds in §3.4 = 11).

Configurable as `appeal.panel_size_multiplier` (float, default 1.5) and `appeal.panel_size_floor_increment` (int, default 2).

### §6.2 Original jurors excluded — hard rule, no override

[01 §5.7](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) explicitly mandates: "Previous jurors are excluded from the appeal panel." This is **not configurable**. The `select_eligible_jurors` call for the appeal panel adds every original juror (every `jury_assignment` row for the case) to the `exclude_person_ids` list passed in. This is a hard rule in code, not in config.

### §6.3 Appeal threshold — bump up one tier

The appeal panel uses a **one-tier-higher threshold** than the original. So a Minor original (simple majority) appeals at 60%; a Moderate original appeals at 75%; a Severe original appeals at 75% (already at the top — no further bump). This is the "appeal raises the bar" principle.

Configurable as `appeal.threshold_tier_bump` (int, default 1; range 0–2). Setting to 0 disables the bump.

### §6.4 Original-reporter appeal-rights (issue #11)

Per OQ-V1-JM-06 lean:

- **Defendant** can always appeal (current v0 behaviour preserved).
- **Original reporter** (the case `creator_id`) can appeal **only if** the decision was `JuryDecision::NoAction` or sanction `Label`. Rationale: a reporter has a legitimate grievance only when the system effectively dismissed their report.

`request_appeal` extension in `crates/api/api_crud/src/governance/request_appeal.rs:108`:

- Add `request_appeal.requester_role` field on the `Appeal` row (enum `Defendant | OriginalReporter`).
- Eligibility check: defendant always; reporter only if `winning_decision IN (NoAction, AdvisoryLabel)`.
- Both follow the same appeal window (§6.5).

### §6.5 Appeal window (issue #15)

v0 defines the window as `case.closed_at IS NULL` — implicit and unbounded. v1 introduces an **explicit, bounded window**:

- New column `moderation_case.appeal_window_expires_at TIMESTAMPTZ` (NULL until decided).
- On case decision (`submit_jury_vote.rs:272-280`), set `appeal_window_expires_at = decided_at + appeal.window_days * INTERVAL '1 day'`. Default 7 days (matches v0 `APPEAL_WINDOW_DAYS = 7`).
- `request_appeal` checks `appeal_window_expires_at > now()` instead of `closed_at IS NULL`.
- Configurable as `appeal.window_days` (int, default 7).

**Interaction with sponsor-liability grace window (OQ-025)**: OQ-025 proposes a sponsor-liability grace window between `Decided` and the actual reputation_event write. v1 jury-mechanics reserves the appeal window column shape; sponsor-liability-v1 PRD (cross-referenced) decides how the two windows compose. Conservative interpretation: sponsor-liability grace runs in parallel to the appeal window with `min(appeal.window_days, sponsor_grace_days)` as the effective sponsor-grace duration, so a successful appeal can void the liability before grace expires.

### §6.6 Re-jury path on appeal (issue #14)

Currently `request_appeal` flips `Decided → Appealed` and the case sits until `admin_close_case` runs. v1 introduces auto re-jury:

- New config key `appeal.auto_select_on_appeal_acceptance` (bool, default `true`).
- When `true` and an appeal is filed, the same `request_appeal` transaction also calls `select_eligible_jurors` with the original-jurors-excluded list and seats the appeal panel as `JuryAssignmentStatus::Selected` (mirroring `admin_assign_jury`'s seating shape).
- When `false`, the case sits in `Appealed` status until an admin runs the new `admin_trigger_appeal_rejury` handler (see §9.4).

### §6.7 Appeal status state machine

Existing `AppealStatus` enum already has `Requested | Accepted | Rejected | Decided`. v1 adds clarity:

- `Requested` — appeal just filed, panel not yet seated.
- `Accepted` — admin (or auto) accepted the appeal; panel is seated.
- `Decided` — appeal panel returned a verdict.
- `Rejected` — admin denied the appeal request (rare; logged with reason).

The case `status` flow becomes `Decided → Appealed → Decided (again, with appeal verdict) → Closed` for accepted appeals; `Decided → Appealed → Closed` (no re-decision) for rejected appeals.

---

## §7 — Concurrent jury cap

### §7.1 v0 baseline preserved

OQ-004 resolved at instance-wide cap = 3 via `config.jury.max_concurrent_assignments`. This stays as the **instance default**.

### §7.2 v1 extensions

- **Per-community override**: `jury.max_concurrent_assignments` is already config-cascade-capable (community → instance → const). v1 adds no new key; communities can set the per-community row via the admin dashboard (admin-dashboard-v1 PRD §3 surface).
- **Per-juror cap (optional)**: New key `jury.max_concurrent_assignments_per_juror_total` (int, default 2). This is the cross-community cap — a juror cannot be on more than N active assignments **total** across all communities. Default 2 because a juror serving on 3 simultaneous panels in different communities is plausibly overcommitted.
  - When NULL/unset, no cross-community cap is applied (preserves v0 behaviour).
  - When set, the eligibility query in `select_eligible_jurors` adds an additional NOT IN subquery counting all-communities active assignments for the candidate juror.
- **Instance-admin can set instance-wide cap separately from per-community cap**. The cascade resolution in `config::get_int` already does this — no extra machinery.

### §7.3 Cross-community jury load balancing

Communities with high case volume can raise their per-community cap (e.g. to 5) while the instance-wide cap stays at 3. The per-juror cross-community cap (`per_juror_total = 2`) prevents a single juror from absorbing the higher load — they'd get distributed across the eligible pool.

---

## §8 — Database & migration changes

### §8.1 New columns on `moderation_case`

Per ADR-010 (no retroactive invalidation), the panel-size / quorum / threshold / severity must be **snapshotted** at jury-pick time, not re-read from config at vote-time.

```sql
ALTER TABLE moderation_case
  ADD COLUMN severity_tier severity_tier NOT NULL DEFAULT 'Minor',
  ADD COLUMN status_tier case_status_tier NOT NULL DEFAULT 'Regular',
  ADD COLUMN panel_size_snapshot INTEGER,             -- snapshot at admin_assign_jury time
  ADD COLUMN quorum_snapshot INTEGER,                 -- snapshot at admin_assign_jury time
  ADD COLUMN threshold_count_snapshot INTEGER,        -- snapshot at admin_assign_jury time
  ADD COLUMN appeal_window_expires_at TIMESTAMPTZ;    -- set at submit_jury_vote time
```

Where `severity_tier` and `case_status_tier` are new Postgres enums, mirrored as `diesel-derive-enum` types in `crates/db_schema_file/src/enums.rs`.

`panel_size_snapshot`, `quorum_snapshot`, `threshold_count_snapshot` are NULL until `admin_assign_jury` seats the panel; from that moment they are immutable.

`appeal_window_expires_at` is NULL until the case is decided; from that moment the appeal window is bounded.

### §8.2 New columns on `jury_assignment`

```sql
ALTER TABLE jury_assignment
  ADD COLUMN selected_under_constraints JSONB,        -- which constraints were applied at pick
  ADD COLUMN role jury_assignment_role NOT NULL DEFAULT 'Original'; -- enum: Original | Appeal
```

`selected_under_constraints` example payload (Watch 10 PII discipline — no person_ids, only constraint names):

```json
{
  "no_majority_from_same_sponsor_cluster": "applied",
  "geographic_diversity_preferred": "applied_soft",
  "no_recent_juror_repeat": "relaxed_small_pool",
  "no_same_endorsement_chain": "disabled"
}
```

`role` distinguishes original-jury rows from appeal-jury rows on the same case (since both reference the same `case_id`). Used by the appeal-panel exclusion logic in §6.2.

### §8.3 New table — `jury_constraint_violation_log`

```sql
CREATE TABLE jury_constraint_violation_log (
  id SERIAL PRIMARY KEY,
  case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
  constraint_name TEXT NOT NULL,
  relaxation_reason TEXT NOT NULL,                 -- 'small_pool', 'admin_override', etc.
  pool_size_at_relax INTEGER NOT NULL,
  panel_size_target INTEGER NOT NULL,
  relaxed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_jcvl_case_id ON jury_constraint_violation_log (case_id);
```

Persisted alongside the `governance_log` `jury_constraint_relaxed` entry — the `governance_log` is the tamper-evident record; this table is the queryable index for the admin dashboard's "show constraint relaxations" view.

### §8.4 Backfill migration for v0 → v1

In-flight v0 cases (status `Open | ThresholdMet | JurySelection | InReview | Decided | Appealed`) at the moment of v1 ship MUST complete under v0 rules per ADR-010. The backfill migration:

```sql
UPDATE moderation_case
SET severity_tier = 'Minor',
    status_tier = 'Regular',
    panel_size_snapshot = 5,
    quorum_snapshot = 3,
    threshold_count_snapshot = 3,    -- 3-of-5 simple majority
    appeal_window_expires_at = COALESCE(closed_at, decided_at + INTERVAL '7 days')
WHERE severity_tier IS NULL OR panel_size_snapshot IS NULL;
```

After the backfill, every existing case has a v0-equivalent snapshot. The handlers (§9) will read these snapshots, not the new config, so v0 cases continue to behave as if v1 hadn't shipped. New cases post-v1-ship use the new config-driven snapshot at admin_assign_jury time.

### §8.5 New `governance_log` entry kinds

Add to `crates/api/api/src/governance/governance_log.rs:50-68`:

```rust
pub const ENTRY_KIND_JURY_CONSTRAINT_RELAXED: &str = "jury_constraint_relaxed";
pub const ENTRY_KIND_APPEAL_PANEL_ASSEMBLED: &str = "appeal_panel_assembled";
pub const ENTRY_KIND_APPEAL_DECIDED: &str = "appeal_decided";
pub const ENTRY_KIND_APPEAL_REJECTED: &str = "appeal_rejected";
pub const ENTRY_KIND_APPEAL_WINDOW_EXPIRED: &str = "appeal_window_expired";
pub const ENTRY_KIND_SEVERITY_TIER_FROZEN: &str = "severity_tier_frozen";
```

These are TEXT additions only — no schema migration needed because `governance_log.entry_kind` is `String` per `crates/db_schema/src/source/governance/governance_log.rs:23-44` (V2 messaging PRD verified this property; v1 jury-mechanics inherits it).

---

## §9 — Handler changes

### §9.1 `submit_jury_vote` — combined 9-step handler (jury-mechanics + sponsor-liability integrated)

`crates/api/api/src/governance/submit_jury_vote.rs`:

**Why this section owns the combined shape.** v1 jury-mechanics changes the decision point itself (quorum snapshot, threshold vs simple-majority, deadlock-to-AdminReview), and v1 sponsor-liability adds a post-decision lifecycle branch (`SponsorLiabilityPending` status, grace-window compute, deferred public_log + juror rep events) on top. Fragmenting the integrated pseudocode across two PRDs left five unanswered cross-cutting questions (see `.claude/PRPs/v1-planning-queue.json` B6 `context`). This section owns the combined 9-step shape; the sponsor-liability-v1 PRD §9.3 reduces to a pointer. The compute/fire split (§9.1 of sponsor-liability-v1) and the grace-window scheduler helper module (§9.4 of sponsor-liability-v1) remain sponsor-liability's concerns — this section just declares **when** the compute fires and **what status transitions follow**.

**9-step pseudocode** (user-provided 2026-04-19, B6 resolution):

```text
submit_jury_vote(vote, context):
  1. Load case row (FOR UPDATE per v0 concurrency guard).
  2. Validate juror is assigned + hasn't already voted + case is in InReview.
  3. Insert jury_vote row.
  4. Tally: read all jury_vote rows for this case.
  5. Threshold check (v1 new):
     - quorum      = case.quorum_snapshot             (set at admin_assign_jury time per §9.2)
     - threshold   = case.threshold_count_snapshot    (set at admin_assign_jury time per §9.2)
     - if any decision has >= threshold votes:
         winning_decision = that decision; continue to step 6.
     - elif all jurors have voted AND no decision reached threshold (deadlock):
         flip case.status to CaseStatus::AdminReview;
         emit governance_log entry jury_deadlock;
         return SubmitJuryVoteResponse { vote_recorded: true, case_decided: false, decision: None }.
     - else (partial tally, no threshold met yet):
         return SubmitJuryVoteResponse { vote_recorded: true, case_decided: false, decision: None };
         case stays InReview.
  6. Winning decision reached. Insert sanction rows per winning_decision
     (one or more sanction rows via the existing sanction-insert helper).
  7. Sponsor-liability branch (per sponsor-liability-v1 §9.3 semantics):
     - if sanction has liability implications (winning_decision != NoAction AND
       severity-tier maps to a sponsor-liability grade) AND target has active sponsors:
         deltas = sponsor_liability::compute_sponsor_liability(conn, target, case_id, …);
         (compute/fire split per sponsor-liability-v1 §9.1 — compute only, no event rows yet)
         grace_expires_at = now() + grace_window_for_severity(severity);
         update moderation_case
           SET status = CaseStatus::SponsorLiabilityPending,
               grace_expires_at = grace_expires_at,
               decided_at = now();
         emit governance_log entry sponsor_liability_pending
           { case_id, grace_expires_at, sponsor_count: deltas.len(), severity };
         for delta in deltas: notify_sponsor_of_pending_liability(conn, delta, case_id, grace_expires_at);
         // DEFERRED to scheduler fire/escape time (per sponsor-liability-v1 §11.4):
         //   - public_case_log entry
         //   - juror reputation_events (aligned/outlier)
         //   - reporter reputation_event (upheld/dismissed)
         //   - case_decided governance_log entry
         // jump to step 9 for appeal_window_expires_at bound.
         return SubmitJuryVoteResponse { vote_recorded: true, case_decided: true, decision: Some(winning_decision) }.
  8. No-sponsor / NoAction path (v0 lifecycle preserved):
     - update moderation_case
         SET status = CaseStatus::Decided,
             decided_at = now(),
             closed_at = now();
     - write public_case_log entry (immediate, not deferred);
     - write juror reputation_events (aligned/outlier) and reporter reputation_event;
     - emit case_decided governance_log entry;
     - jump to step 9.
  9. Appeal window bound (v1 new — per appeal.window_days):
     - window_days = config::get_int(..., "appeal.window_days").await?;   // read at decision time
     - appeal_window_expires_at = decided_at + Duration::days(window_days);
     - update moderation_case SET appeal_window_expires_at = appeal_window_expires_at;
     - (This step fires on BOTH the SponsorLiabilityPending path AND the no-sponsor Decided
        path. It composes with sponsor-liability §6.5 min(appeal_window_expires_at, grace_expires_at)
        semantics: the sponsor can be pulled back by revocation during the grace window OR an
        admin/defendant can lodge an appeal during the appeal window — whichever comes first in
        the timeline.)
     - return SubmitJuryVoteResponse as per the branch taken above.
```

**Golden-path-test impact (test-strategy sub-note):** the v0 `report_to_modlog_golden_path` integration test uses an unsponsored target → step 8 no-sponsor path → public_case_log fires immediately → existing test assertions preserved. The v1 sponsored-target test is a **v1-impl follow-up** (new test, not a modification of the golden path): it asserts that for a sponsored target, (a) the case transitions to `SponsorLiabilityPending` with `grace_expires_at` set, (b) `public_case_log` does NOT fire at vote-tally time, (c) `case_decided` governance_log entry does NOT fire at vote-tally time, and (d) both fire at the scheduler's fire-or-escape transition.

**Cross-references within v1:**
- **SponsorLiabilityPending branch semantics** are specified in detail by `v1-sponsor-liability.prd.md` §9.3 (which now points here for the combined handler shape) and §6 (grace-window scheduler `run_grace_check_batch`).
- **`compute_sponsor_liability` vs `fire_sponsor_liability`** (the compute/fire split) is specified by `v1-sponsor-liability.prd.md` §9.1; this §9.1 only declares when the compute phase fires (step 7).
- **`grace_window_for_severity` helper module** is specified by `v1-sponsor-liability.prd.md` §9.4.
- **`appeal.window_days` knob** is owned by this PRD (§10); read at decision time (step 9), NOT snapshotted, per ADR-010's no-retroactive-invalidation-of-procedural-rules-of-the-*past*-vote reading. Changing `appeal.window_days` mid-flight affects windows for cases decided after the change, not cases already decided.
- **`quorum_snapshot` / `threshold_count_snapshot`** are set by `admin_assign_jury` at the moment of jury assembly (per §9.2); they are fixed for the life of the case per ADR-010. Admin retuning of `jury.panel_size` or `jury.*_fraction` mid-flight does NOT affect cases already in `InReview`.

**Status transitions summary** (v0 → v1 reshape):
- `InReview` → `Decided` (no-sponsor branch, step 8) — unchanged from v0.
- `InReview` → `SponsorLiabilityPending` (sponsor branch, step 7) — new in v1.
- `InReview` → `AdminReview` (deadlock, step 5) — new in v1 (reactivates existing dead `CaseStatus::AdminReview` enum variant per V2 messaging PRD §Q3 finding).
- (Scheduler then transitions `SponsorLiabilityPending` → `SponsorLiabilityFired` | `SponsorLiabilityEscaped` per sponsor-liability-v1 §6.1 / §6.2; those are out of this handler's scope but in-scope for the combined lifecycle shape.)

### §9.2 `admin_assign_jury` — size, snapshot, and apply diversity

`crates/api/api/src/governance/admin_assign_jury.rs`:

- **Read** `case.severity_tier` and `case.status_tier` at line 119 (replacing the singular `jury.panel_size` read).
- **Compute** `panel_size` via a new cascade helper `config::get_int_cascade(&["jury.panel_size", &status_str, &severity_str])` (per §3.5) that walks the cascade `jury.panel_size.<status>.<severity>` → `jury.panel_size.<severity>` → `jury.panel_size` → Rust const. Same pattern for `quorum_fraction` and `threshold_fraction`. Resolve `quorum = ceil(panel_size * quorum_fraction)`, `threshold_count = ceil(panel_size * threshold_fraction)`. Do NOT call `get_int("jury.panel_size.<status>.<severity>")` directly — that skips the cascade and returns Err on partial seeding.
- **Snapshot** these onto the case: `update(moderation_case::table)... .set((moderation_case::panel_size_snapshot.eq(panel_size_i32), ...))`.
- **Pass** the active diversity-constraint set into `select_eligible_jurors` as a new parameter.
- **Emit** `governance_log` entry `severity_tier_frozen` at the moment of snapshot (audit trail).

`select_eligible_jurors` (same file, `:215-274`) gains:

- A new `constraints: ConstraintSet` parameter.
- The constraint application logic per §5.3 (cooldown filter, sponsor-cluster panel-level check with re-roll, soft geographic diversity scoring).
- The relaxation cascade per §5.3, with each relaxation writing both a `governance_log` entry (`jury_constraint_relaxed`) and a `jury_constraint_violation_log` row.

### §9.3 `request_appeal` — bounded window, reporter-rights, auto re-jury

`crates/api/api_crud/src/governance/request_appeal.rs`:

- **Replace** `if case.closed_at.is_some()` (line 105) with `if case.appeal_window_expires_at.is_none() || case.appeal_window_expires_at.unwrap() < now()`.
- **Extend** caller eligibility (line 110): defendant always; reporter only if `case_decision IN (NoAction, AdvisoryLabel)`. Reporter eligibility requires loading the winning decision from the latest `jury_vote` rows OR adding a `winning_decision` column to `moderation_case` for queryability. **Recommendation**: add `moderation_case.winning_decision JuryDecision NULL` set by `submit_jury_vote` at line 272 (cross-references issue #19 / GovernanceModlogView Phase 2b drift resolutions).
- **On appeal accepted** and `appeal.auto_select_on_appeal_acceptance = true`: in the same transaction, call a new helper `select_appeal_panel(case, original_jurors)` that re-runs `select_eligible_jurors` with the original jurors added to `exclude_person_ids` and the **next-tier** threshold/panel size per §6. Snapshot the appeal panel's parameters onto a new column or row (TBD — see §8.2 `jury_assignment.role = 'Appeal'`).
- **Emit** `governance_log` entry `appeal_panel_assembled` with the new panel pseudonyms.

### §9.4 New handler — `admin_trigger_appeal_rejury`

For the `appeal.auto_select_on_appeal_acceptance = false` configuration path. New file `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs`:

- Admin-only (`is_admin` check).
- Input: `AdminTriggerAppealRejury { case_id }`.
- Loads the case; verifies status is `Appealed` and no Appeal-role jury_assignment rows exist yet.
- Calls the same `select_appeal_panel` helper as §9.3.
- Emits `appeal_panel_assembled` log entry attributed to the admin.

Route: `POST /api/v4/governance/admin/trigger-appeal-rejury`.

### §9.5 New background job — appeal-window expiry

A periodic job (mirrors the existing `expired sanction cleanup` job pattern in `crates/server/src/governance.rs`) that:

- Finds cases where `status = Decided AND appeal_window_expires_at < now()`.
- Flips them to `Closed`.
- Emits `appeal_window_expired` log entry.

### §9.6 `admin_emergency_remove` — Severe by default

`crates/api/api/src/governance/admin_emergency_remove.rs:153`:

- **Set** `severity_tier = SeverityTier::Severe` (matches §3.2 edge case).
- The post-facto jury then gets sized per `jury.panel_size.regular.severe` (default 7) automatically via `admin_assign_jury` reuse pattern.

---

## §10 — Default values matrix

Every knob introduced or modified by this PRD. Source-doc citations point to where the default is justified.

| Knob | Namespace key | Type | Default | Range / Bounds | Per-community? | Requires re-jury? | Source citation |
|---|---|---|---|---|---|---|---|
| Panel size — Regular Minor | `jury.panel_size.regular.minor` | int | 5 | [3, 11] odd | yes | no (snapshotted) | [05 §3](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) v0 baseline preserved |
| Panel size — Regular Moderate | `jury.panel_size.regular.moderate` | int | 5 | [3, 11] odd | yes | no | [05 §3](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md), v0 baseline |
| Panel size — Regular Severe | `jury.panel_size.regular.severe` | int | 7 | [3, 11] odd | yes | no | [01 §5.6](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) v1 target |
| Panel size — Founder Minor | `jury.panel_size.founder.minor` | int | 5 | [3, 11] odd | yes | no | OQ-026 status-aware extension |
| Panel size — Founder Moderate | `jury.panel_size.founder.moderate` | int | 7 | [3, 11] odd | yes | no | OQ-026; founder cases higher-stakes |
| Panel size — Founder Severe | `jury.panel_size.founder.severe` | int | 9 | [3, 11] odd | yes | no | OQ-026; founder × Severe = max-care |
| Panel size — Probation Minor | `jury.panel_size.probation.minor` | int | 3 | [3, 11] odd | yes | no | OQ-026; probation = faster turnaround |
| Panel size — Probation Moderate | `jury.panel_size.probation.moderate` | int | 5 | [3, 11] odd | yes | no | OQ-026 |
| Panel size — Probation Severe | `jury.panel_size.probation.severe` | int | 5 | [3, 11] odd | yes | no | OQ-026 |
| Quorum fraction — Minor | `jury.quorum_fraction.minor` | float | 0.6 | [0.5, 1.0] | yes | no | derived: 3-of-5 v0 baseline = 60% |
| Quorum fraction — Moderate | `jury.quorum_fraction.moderate` | float | 0.6 | [0.5, 1.0] | yes | no | mirrors Minor for v1 |
| Quorum fraction — Severe | `jury.quorum_fraction.severe` | float | 0.71 | [0.5, 1.0] | yes | no | derived: 5-of-7 = 71% |
| Threshold fraction — Minor | `jury.threshold_fraction.minor` | float | 0.5001 | [0.5001, 1.0] | yes | no | [01 §5.6](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) simple majority |
| Threshold fraction — Moderate | `jury.threshold_fraction.moderate` | float | 0.6 | [0.5001, 1.0] | yes | no | [01 §5.6](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) 60% |
| Threshold fraction — Severe | `jury.threshold_fraction.severe` | float | 0.75 | [0.5001, 1.0] | yes | no | [01 §5.6](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) 75% supermajority |
| Diversity — sponsor cluster | `jury.constraints.no_majority_from_same_sponsor_cluster` | bool | true | true / false | yes | no | [01 §4 principle 3](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) kin-group |
| Diversity — geographic | `jury.constraints.geographic_diversity_preferred` | bool | true | true / false | yes | no | OQ-V1-JM-02 lean |
| Diversity — recent juror repeat | `jury.constraints.no_recent_juror_repeat` | bool | true | true / false | yes | no | OQ-V1-JM-02 anti-capture |
| Juror cooldown days | `jury.constraints.juror_cooldown_days` | int | 7 | [0, 90] | yes | no | tunable per community size |
| Diversity — endorsement chain | `jury.constraints.no_same_endorsement_chain` | bool | false | true / false | yes | no | v1.5 deferred (graph cost) |
| Selection retries | `jury.constraints.max_retries_before_relax` | int | 5 | [1, 20] | yes | no | Operational tuning |
| Per-juror cross-community cap | `jury.max_concurrent_assignments_per_juror_total` | int | 2 | [1, 10] | no (instance only) | no | §7.2 derived from cap=3 baseline |
| Appeal panel multiplier | `appeal.panel_size_multiplier` | float | 1.5 | [1.0, 3.0] | yes | no | [01 §5.7](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) "larger jury" |
| Appeal panel floor increment | `appeal.panel_size_floor_increment` | int | 2 | [0, 6] | yes | no | §6.1 minimum increment |
| Appeal threshold tier bump | `appeal.threshold_tier_bump` | int | 1 | [0, 2] | yes | no | §6.3 "appeal raises bar" |
| Appeal window days | `appeal.window_days` | int | 7 | [1, 90] | yes | no | matches v0 hardcoded `APPEAL_WINDOW_DAYS = 7` |
| Auto re-jury on appeal | `appeal.auto_select_on_appeal_acceptance` | bool | true | true / false | yes | no | §6.6 default behaviour |

**Knobs preserved from v0 (no change in v1, listed for completeness)**:

| Knob | Namespace key | v0 default | v1 status |
|---|---|---|---|
| Concurrent cap | `jury.max_concurrent_assignments` | 3 | unchanged; per-community override now operationally relevant |
| Fallback on small pool | `jury.fallback_on_small_pool` | true | unchanged; still the final fallback after constraint-relaxation cascade |
| Reputation gate min | `thresholds.jury_reliability` | 50 | unchanged in this PRD; reputation-tuning-v1 PRD owns this |

**Total new knobs introduced by this PRD: 25.**

---

## §11 — Backwards compatibility with v0 cases

Per ADR-010 hard rule: config changes MUST NOT retroactively invalidate in-flight juries.

**Mechanism**: the snapshot columns added in §8.1 (`panel_size_snapshot`, `quorum_snapshot`, `threshold_count_snapshot`, `severity_tier`, `status_tier`). The migration at §8.4 backfills these for every pre-v1 case.

**Behaviour after v1 ships**:

- A v0 case in `JurySelection` with 5 already-seated jurors continues with `quorum_snapshot=3`, `threshold_count_snapshot=3`, `severity_tier=Minor`. `submit_jury_vote` reads the snapshot, not the new config. The case completes under v0 rules.
- A v0 case in `Decided` waiting for appeal gets `appeal_window_expires_at = decided_at + 7 days` from the backfill. If the appeal window had already expired by v1-ship date, the case is also flipped to `Closed` by the backfill (additional `WHERE decided_at + INTERVAL '7 days' < now()` clause).
- New cases opened post-v1-ship use the new config and the new snapshot machinery from `admin_assign_jury` onward.

**Test invariant**: A separate integration test in `tests/e2e.rs` named `v0_case_completes_under_v0_rules_after_v1_config_flip` opens a case under v0 defaults, mid-flight flips the v1 config to wildly different values (e.g. `panel_size=11, threshold=0.95`), and asserts the case still concludes with the v0 3-of-5 simple-majority rule per the snapshot.

---

## §12 — Security

### §12.1 Severity tier assignment requires capability + audit

- Severity tier set at case-open time can be:
  - **Reporter-suggested** (default behaviour) — derived from `reason_code` + community policy mapping; no capability check.
  - **Capability-tagged moderator override** — the `trusted_reporter` reputation flag (existing `reputation_snapshot.trusted_reporter`) is repurposed as the severity-override capability. A trusted reporter can override the suggested severity; their override is logged.
  - **Admin override** — instance admin can override at any time before `JurySelection`. Logged.
- Every severity change emits a `governance_log` entry tagged `severity_tier_assigned` (initial) or `severity_tier_overridden` (subsequent), with the reason_code, prior tier, new tier, actor pseudonym.
- Severity tier is **frozen** on transition to `JurySelection`. The `severity_tier_frozen` log entry is the canonical "after this point, no change is procedurally legitimate" marker.

### §12.2 Constraint relaxation requires audit-log entry visible to community admin

Each relaxation in §5.3 emits both:

- A tamper-evident `governance_log` entry of kind `jury_constraint_relaxed`.
- A queryable `jury_constraint_violation_log` row.

The community admin's dashboard (cross-referenced) surfaces these in a "Why was a constraint relaxed?" section. **Operators cannot silently relax constraints**; transparency replaces strict enforcement.

### §12.3 Step-up auth for severity-tier changes mid-case

ADR-010 disallows mid-case severity changes (would re-jury). v1 enforces:

- After `JurySelection`, severity-tier updates are **rejected at the API level** with `ErrorMethodNotAllowed`. The only path to "change severity mid-case" is to close the case and open a new one (which is a separate audit event).
- The `severity_tier_frozen` log entry serves as the immutability marker; any later attempt to update `moderation_case.severity_tier` triggers a database-level CHECK constraint failure (recommended) or handler-level rejection.

### §12.4 Appeal-rights spoofing protection

`request_appeal`'s reporter-eligibility check (§9.3) trusts `case.creator_id`. If `creator_id` is NULL (orphaned case via deleted reporter), reporter-appeal is impossible by construction. No spoofing surface.

---

## §13 — Open questions for v1 design phase

(Already enumerated in §Open Questions Carried Forward; restated here for the standard PRD shape.)

- **OQ-V1-JM-01**: Severity-tier inference — reporter-tags (lean) vs reported-content-classifier vs moderator-set. v1 commits to reporter-suggested + admin-override at case-open time.
- **OQ-V1-JM-02**: Geographic diversity heuristic — declared community (lean), timezone (deferred v1.5), IP-region (rejected as PII).
- **OQ-V1-JM-03**: Founder-status threshold — defendant status (lean) vs case-class. v1 commits to defendant status; case-class is v1.5.
- **OQ-V1-JM-04**: Cross-instance juror eligibility — parked, v2 territory.
- **OQ-V1-JM-05**: Re-jury auto vs admin trigger — default `auto_select_on_appeal_acceptance = true`.
- **OQ-V1-JM-06**: Original-reporter appeal scope — only `NoAction` / `AdvisoryLabel` decisions; same window as defendant.

---

## §14 — Cross-references

- **admin-dashboard-v1 PRD** (§3 config surface) — every config knob in §10 is exposed via the admin dashboard. The dashboard enforces bounds (panel_size odd, fractions in [0.5, 1.0]). Dashboard PRD must reserve the dotted namespace prefix `jury.*` and `appeal.*` for this PRD.
- **reputation-tuning-v1 PRD** — owns the eligibility weighting (`thresholds.jury_reliability`, decay tuning). Jury-mechanics-v1 reads `reputation_snapshot.jury_eligible` but does not change how it's computed. Issue #19 (`count_active_sanctions` community-scope) belongs in reputation-tuning-v1.
- **sponsor-liability-v1 PRD** — OQ-025 grace window. The `appeal.window_days` from this PRD and the `sponsor_grace_*_hours` from sponsor-liability-v1 must compose: a successful appeal voids sponsor-liability if it concludes before `min(appeal.window_days, sponsor_grace_days)` elapses.
- **v2 messaging PRD** (`.claude/PRPs/prds/v2-messaging-rtc.prd.md`) — V2b auto-provisions a Matrix room on `CaseStatus::JurySelection`. Larger v1 panels mean bigger jury rooms; V2b's pseudonym rendering scales with `panel_size_snapshot`. No conflict.
- **v0 implementation files** — every handler change in §9 lists its source file. None require new crates; all are extensions to existing v0 governance handlers in `crates/api/api{,_crud}/src/governance/`.
- **GitHub issues carried forward**:
  - #11 — original-reporter appeals → §6.4
  - #14 — re-jury path for Appealed cases → §6.6, §9.4
  - #15 — formalise appeal window with bounded duration → §6.5, §8.1, §9.3
  - #19 — community-scope `count_active_sanctions` → reputation-tuning-v1 PRD (cross-ref only)
  - #29 — config-flip determinism → §5.4 (audit-log shape supports stable assertions; v1 does not make selection deterministic)

---

### Critical Files for Implementation

- `crates/api/api/src/governance/admin_assign_jury.rs` — sizing, snapshot, diversity selection (the largest single change surface)
- `crates/api/api/src/governance/submit_jury_vote.rs` — read snapshots not config, threshold-based winner picking
- `crates/api/api_crud/src/governance/request_appeal.rs` — bounded window, reporter rights, auto re-jury seeding
- `crates/api/api/src/governance/config.rs` — new const defaults + new keys in `SEEDED_KEYS_WITH_CONSTS` + new `get_int_cascade` / `get_float_cascade` helpers per §3.5
- `crates/db_schema_file/src/enums.rs` + a new migration `migrations/<ts>_jury_mechanics_v1/up.sql` — `SeverityTier`, `CaseStatusTier`, `JuryAssignmentRole` enums + the snapshot/audit columns and table

---

## §15 — Resolutions applied (2026-04-19)

v1-planning-session cross-PRD coherence audit applied the following resolutions to this PRD. Traceable via `.claude/PRPs/v1-planning-queue.json`.

| ID | Resolution | Effect in this PRD |
|---|---|---|
| **B1** | jury-mechanics owns `appeal.*` namespace fully | No change — already the owner. admin-dashboard §5.2 delegates and §10.2 #15 cross-references here. `appeal.original_jurors_excluded` remains a hard code rule (§6.2), not a config key. |
| **B2** | Dotted `jury.panel_size.<status>.<severity>` matrix WITH single-key fallback per OQ-026 cascade | §3.5 Cascade fallback subsection added; §9.2 directive to use new `get_int_cascade` helper. Bare `jury.panel_size` retained as v1 root default (value=7) in admin-dashboard §5.1 — not duplicated here. |
| **B6** | jury-mechanics owns combined post-v1 `submit_jury_vote` handler shape | **Done.** §9.1 expanded to 9-step combined-handler pseudocode (load-case/validate/insert-vote/tally → threshold-check with deadlock-to-AdminReview → winning-decision → sponsor-liability branch with `SponsorLiabilityPending` + grace-window compute + deferred public_log/juror-rep → no-sponsor Decided branch preserved from v0 → appeal-window bound step 9 firing on both paths). Cross-references to sponsor-liability-v1 §9.1/§9.3/§9.4 added. Golden-path test impact documented (step 8 no-sponsor assertions preserved; new sponsored-target test is v1-impl follow-up). sponsor-liability-v1 §9.3 reduces to pointer (see its §18 Resolutions). |
| **N3** | jury-mechanics §5.3 constraint-relaxation algorithm clarified | **Done.** §5.3 rewritten into a three-phase flowchart (Phase 1 pool-build filter for cooldown, Phase 2 panel-sample re-roll for sponsor-cluster, Phase 3 panel-score soft bias for geographic). Relaxation cascade renamed to R1/R2/R3 with explicit triggers and audit-log entries; `warn!` added on R3 (sponsor-cluster drop) because it's the last resort. Makes the phase structure (filter vs check vs score) explicit for the §9.3 impl agent and clarifies why cooldown relaxes first (expands the pool), sponsor-cluster relaxes last (Brehon-essential structural-capture guard). |
| **NOT1** | OQ prefixed-namespace convention (collision resolution) | **Done.** jury-mechanics' six `OQ-027..032` renumbered to `OQ-V1-JM-01..06` via mechanical replace-all. Affects §4 Open Questions table, §3.2 severity inference reference, §5 sponsor-cluster and geographic-diversity lean references, §6 appeal rerun config reference, §9 original-reporter eligibility reference, §14 OQs Opened Here summary. No substantive text changed; only the prefix shifts to match the convention already used by reputation-tuning / sponsor-liability / federation-inbound. Resolves the admin-dashboard/jury-mechanics OQ-027 collision identified during the audit. admin-dashboard OQ-027..031 renamed separately to OQ-V1-AD-01..05. |

---

## §16 — Decisions Log (phase split)

| Decision | Choice | Alternatives | Rationale |
|---|---|---|---|
| Monolithic PRD vs sub-phase split | Split into 5 sub-phases (v1-JM-a..e) | Single `/prp-ralph` run over ~28-33 tasks | Phase-splitting rule in `project_brehon_governance_platform.md` caps any single ralph loop at ~10-12 tasks. Phase 5 retro confirmed the ~200k used-context threshold as load-bearing; JM's handler + constraint-algorithm reasoning density is comparable to Phase 5b's sponsor-liability math. Split at natural crate/subsystem boundaries per the phase-splitting rule. |
| Split boundary — schema vs handler | Phase a = schema + enums + backfill + log-kind consts only; no handler edits | Schema + first handler in one phase (mirrors Phase 4 which bundled schema with first handlers) | Phase 4 was grandfathered under a pre-PR workflow. Every v1 sub-phase ships as a separate PR → CodeRabbit review → merge; isolating schema in its own PR lets review focus on migration safety + backfill correctness without handler churn distracting reviewers. Mirrors v1-AD-a and v1-SL-a cadence. |
| `submit_jury_vote` isolation | Own sub-phase (v1-JM-c) | Bundle with admin_assign_jury in one handler-phase | `submit_jury_vote` is the combined 9-step handler per §9.1 B6 resolution — HIGH-risk critical-path code with a sponsor-liability hook point at step 7. Phase 5 retro: "concurrency tests should land in the phase that introduces the concurrency primitive" — step-5 threshold check + deadlock path needs its own review surface. Also: sponsor-liability-v1 v1-SL-d depends on v1-JM-c's handler shape stabilising before SL grafts its compute branch at step 7. |
| Constraint-relaxation isolation | Own sub-phase slot (v1-JM-b) | Bundle admin_assign_jury + submit_jury_vote in one "handlers" phase | Constraint-relaxation algorithm (§5.3 three-phase flowchart with R1/R2/R3 cascade) is the densest reasoning in JM per the N3 resolution. Phase 5a retro: "isolate cross-cutting integer math + Phase-N-1 file mutations into a small focused sub-phase when present." v1-JM-b is that sub-phase for JM — small, pure, high-reasoning. |
| Appeals as own sub-phase | Own sub-phase (v1-JM-d) after submit_jury_vote | Merge appeals into v1-JM-c | Appeals read `appeal_window_expires_at` which is written by v1-JM-c's submit_jury_vote step 9. v1-JM-d depends on v1-JM-c but is a coherent feature unit (request_appeal + admin_trigger_appeal_rejury + background job) — splitting keeps v1-JM-c small and defers the §6 sub-sections (reporter-rights, auto-rejury, window expiry) to their own review surface. |
| Capstone sub-phase | Own sub-phase (v1-JM-e) for security + e2e integration | Bundle e2e into each sub-phase's DoD | Each sub-phase ships with its own per-feature tests, but the canonical regression test `v0_case_completes_under_v0_rules_after_v1_config_flip` (§11) exercises every JM touchpoint — schema snapshots, admin_assign_jury, submit_jury_vote, request_appeal. Cannot write it until d merges. Phase 4b model: dedicated capstone sub-phase for integration validation. Also carries §12 security hardening (step-up auth stub + spoofing protection) as low-risk additive work. |
| Sequencing across PRDs — JM before SL | JM-a/b/c must merge before SL-d starts impl | Parallel tracks with merge conflict resolution after | SL-d mutates `submit_jury_vote` step 7 (compute/fire split); that's the exact line v1-JM-c stabilises. Concurrent impl would produce a three-way merge on the handler file every time either side commits. JM-a/b/c serially first, then SL-d safe to run parallel with JM-d (different files). |
| Sequencing vs reputation-tuning | JM-a independent of RT.r1; can run in parallel | Serial with RT | Different tables (JM touches moderation_case + jury_assignment + new jury_constraint_violation_log; RT.r1 touches reputation_event + new sponsor_allowlist). Different handlers (JM touches admin_assign_jury/submit_jury_vote/request_appeal; RT.r3 touches submit_jury_vote step-7 emitter which is stubbed by JM-c with a TODO). Safe to parallelise. |

---

## §17 — Implementation Phases (for follow-up `/prp-plan` runs)

| # | Phase | Description | Task est. | Risk | Status | Depends |
|---|---|---|---|---|---|---|
| 1 | **v1-JM-a** — Schema + enums + snapshot columns + backfill | 3 new Postgres enums (`severity_tier`, `case_status_tier`, `jury_assignment_role`) in own migrations (must land before table ALTERs reference them); 6 new columns on `moderation_case` (`severity_tier`, `status_tier`, `panel_size_snapshot`, `quorum_snapshot`, `threshold_count_snapshot`, `appeal_window_expires_at`); 2 new columns on `jury_assignment` (`selected_under_constraints` JSONB, `role`); new table `jury_constraint_violation_log`; v0→v1 backfill migration (§8.4 — minor-default + 3-of-5 + 7-day appeal window); `EXPECTED_SEED_COUNT_V1_JM` parametric const (follows v1-AD-a precedent); JM's owned `jury.*` + `appeal.*` config keys seeded; 6 new `ENTRY_KIND_*` consts per §8.5 (dual-file edit per v1-AD-a §10.8 pattern: define in `db_schema`, re-export in `api` shim); append jury-mechanics-v1 section to `.claude/rules/governance-log-entry-kind-registry.md` | ~8-9 | LOW | pending | v1-AD-a merged |
| 2 | **v1-JM-b** — `admin_assign_jury` cascade + diversity constraints | `get_int_cascade` / `get_float_cascade` helpers in `config.rs` for the dotted-namespace fallback chain per §3.5 (B2 resolution); `admin_assign_jury` reads cascade for panel_size / quorum / threshold, writes snapshots on `moderation_case`, emits `severity_tier_frozen` log entry; three-phase constraint-relaxation algorithm per §5.3 N3 resolution (Phase 1 pool-build filter → Phase 2 panel-sample re-roll → Phase 3 panel-score soft bias); R1/R2/R3 relaxation cascade with `warn!` on R3 (sponsor-cluster drop); `jury_constraint_violation_log` write on every relaxation + matching `jury_constraint_relaxed` governance_log entry; `selected_under_constraints` JSONB payload writer (Watch 10 PII discipline — constraint names only, no `person_id`); unit tests per relaxation path + e2e constraint-relaxation audit trail | ~6-7 | MED | pending | Phase 1 |
| 3 | **v1-JM-c** — `submit_jury_vote` 9-step handler (JM portion) | §9.1 pseudocode steps 1-6 + step 9 (JM-owned scope); step 1-2 = load case `FOR UPDATE` + validate juror; step 3-4 = insert jury_vote + tally; step 5 = threshold check reading `case.quorum_snapshot` + `case.threshold_count_snapshot` (NOT re-read from config — Watch 4 snapshot-at-decision-time); deadlock path flips to `CaseStatus::AdminReview` + emits `jury_deadlock`; step 6 = sanction row insert + `case_decided` log; step 7 = `TODO(sponsor-liability-v1)` stub at insertion point (SL-d grafts compute branch here); step 9 = `appeal_window_expires_at = now + appeal.window_days` (LIVE config read, NOT snapshotted, per §9.1); e2e `v0_case_completes_under_v0_rules_after_v1_config_flip` (§11 canonical regression); e2e deadlock-to-AdminReview; e2e concurrency test for two concurrent votes racing to meet threshold | ~5-6 | HIGH | pending | Phase 1 |
| 4 | **v1-JM-d** — Appeals (bounded window + reporter-rights + auto-rejury) | §6 + §9.3-9.5; `request_appeal` bounded-window check (replaces `case.closed_at.is_some()` with `appeal_window_expires_at` comparison per §6.5); reporter-eligibility check — only if `case.winning_decision IN (NoAction, AdvisoryLabel)` per §6.4 OQ-V1-JM-06; auto-rejury on appeal acceptance when `appeal.auto_select_on_appeal_acceptance = true` (per OQ-V1-JM-05 lean); `select_appeal_panel` helper with `role = Appeal` + original-jurors-excluded hard rule (§6.2 B1 — hard code rule, NOT a config key); appeal threshold-tier bump (§6.3); new handler `admin_trigger_appeal_rejury` (§9.4); appeal-window expiry background job in `scheduled_tasks.rs` hourly tick, flips expired appeals, emits `appeal_window_expired` (§9.5); e2e appeal happy path + reporter appeal on NoAction + appeal panel excludes original jurors | ~6-7 | MED | pending | Phase 3 |
| 5 | **v1-JM-e** — Security hardening + integration test capstone | §12.1-12.4; severity-tier change capability check (`is_admin` OR capability-tagged moderator) + audit log entry; step-up auth stub for mid-case severity changes — v1 ships `step_up_token: Option<String>` DTO slot (v1 behaviour = ignore, v2 activates per ADR-010); appeal-rights spoofing protection — verify `case.creator_id IS NOT NULL` path in request_appeal (§12.4); constraint-relaxation admin-visibility check (§12.2) — dashboard query for community-admin role; e2e capstone chained test — open case → assign jury with constraint relaxation → jury votes → appeal requested → rejury triggered → appeal decided (full v1 golden path); mid-flight config-churn regression test | ~4-5 | LOW | pending | Phases 1-4 |

**Total: ~28-33 tasks across 5 PRs.** Each phase ships as its own PR into `governance-v0`; CodeRabbit auto-reviews; `--merge` (not `--squash`) to preserve task-per-commit history per `feedback_pr_per_phase.md`. Each phase writes its own retro at `.claude/PRPs/reports/phase-v1-JM-<x>-retro.md` BEFORE PR merge per `feedback_retro_not_report.md`.

### §17.1 — Cross-PRD sequencing

- **v1-JM-a** can run in parallel with **v1-rep-tuning-r1** (reputation-tuning schema phase) — different tables, different migrations, zero file overlap.
- **v1-JM-c must merge before v1-SL-d starts impl** — v1-SL-d grafts the sponsor-liability compute branch onto `submit_jury_vote` step 7, the exact line v1-JM-c stabilises. Concurrent impl would produce three-way merge conflicts on every commit.
- **v1-JM-c must merge before v1-rep-tuning-r3 starts impl** — v1-rep-tuning-r3 adds vote-outcome + evidence-quality emitters at `submit_jury_vote` step 7 (same TODO insertion point SL-d grafts onto). Serial with v1-JM-c; parallel with v1-JM-d/e.
- **v1-JM-d/e can run in parallel with v1-SL-d and v1-rep-tuning-r3/r4/r5** — different handlers, different files once v1-JM-c has stabilised the submit_jury_vote shape.

### §17.2 — Feature-flag posture

Unlike v1-SL-a (which could ship harmlessly behind `feature.sponsor_liability_grace_window_enabled` because new CaseStatus variants had no writer until SL-d), **v1-JM-a cannot be dark-launched via a feature flag**. The schema migration changes behaviour the moment it lands:

- Backfill migration writes `severity_tier = 'Minor'` + `panel_size_snapshot = 5` on every existing case. Any v1-JM-c-merged submit_jury_vote build reads these snapshots immediately. There is no "dormant schema" state.
- The `_snapshot` columns are NULL until v1-JM-b runs `admin_assign_jury` — but the backfill populates them for pre-v1 cases, so reads see consistent data from day one.

Consequence: v1-JM-a's PR gate is stricter than v1-SL-a's. CodeRabbit review + e2e `phase1_migrations_round_trip.limit(N)` extended to cover the new migrations + smoke test that backfill on a seeded DB produces the expected Minor/Regular/5/3/3 snapshot values.

### §17.3 — Worktree and branch naming (per `.claude/rules/phase-branch.md`)

| Phase | Branch | Worktree path | Advisor-provisioned |
|---|---|---|---|
| v1-JM-a | `phase-v1-JM-a` | `../brehon-fork-phase-v1-JM-a` | before Phase-a `/prp-plan` runs |
| v1-JM-b | `phase-v1-JM-b` | `../brehon-fork-phase-v1-JM-b` | after Phase-a PR merges |
| v1-JM-c | `phase-v1-JM-c` | `../brehon-fork-phase-v1-JM-c` | after Phase-b PR merges |
| v1-JM-d | `phase-v1-JM-d` | `../brehon-fork-phase-v1-JM-d` | after Phase-c PR merges |
| v1-JM-e | `phase-v1-JM-e` | `../brehon-fork-phase-v1-JM-e` | after Phase-d PR merges |

Each worktree cut from then-current `governance-v0` HEAD (parametric, not a hard-coded SHA). Advisor verifies merge-base parity at worktree-cut time. Submodules initialised per-worktree (`crates/email/translations` doesn't auto-propagate); `.env` copied from primary worktree (gitignored).
