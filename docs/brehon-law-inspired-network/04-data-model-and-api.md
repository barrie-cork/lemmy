# 04 — Data Model & API

**Audience:** Backend engineers (daily-driver reference)
**Status:** LIVING — current as of `governance-v0` @ `644ce42d6` (2026-05-29); reflects the v0 base + all merged v1 schema through the federation-inbound lane (`2026-05-17` migration, shipped 2026-05-18). Derived from live code (`migrations/` + `crates/`), not design intent. Remaining v1 lanes (RT-r4/r5, quality-r2, federation-inbound-c onward) update this doc as they ship.
**Source of truth:** the `migrations/` DDL + `crates/db_schema/src/source/governance/*.rs` Diesel models + `crates/db_schema_file/src/enums.rs` enum defs + the route registration in `crates/api/routes/src/lib.rs` are authoritative; this doc is their readable synthesis. **On any discrepancy, CODE WINS — fix the doc.**

This is the implementation reference. It holds migrations, tables, enums, Diesel structs, view structs, request/response DTOs, the REST route table, handler responsibilities, the jury-vote aggregation lifecycle, and the federation surface. Concepts and lifecycles live in [02-domain-model.md](02-domain-model.md). Architecture and crate layout live in [03-architecture.md](03-architecture.md). ADRs and open questions live in [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md).

> **How to read this doc.** Sections are grouped v0-core-first, then v1 extensions, in dependency order. Where a column/table/enum was added by a specific v1 sub-phase, the sub-phase tag (e.g. `v1-JM-a`, `v1-SL-a`, `v1-RT-r1`, `v1-federation-inbound-a`) is noted inline so you can trace the migration that introduced it. Reputation, jury-mechanics, sponsor-liability, and federation-inbound knobs are NOT hardcoded — they live in `governance_config` (§8) and are read at runtime; the one deliberate live-read exception (`appeal.window_days`) is flagged in §9.

---

## 1. Migrations (dependency order)

**31 governance migrations** (the `2026-04-15` enum migration through `2026-05-17` federation-inbound), plus governance-relevant constraint/seed migrations. All reuse Lemmy's integer ID style and add FKs to `person`, `community`, `post`, `comment`, `instance`, and existing moderation targets. Every migration has `up.sql` + `down.sql`. Migrations that use `ALTER TYPE ... ADD VALUE` carry a `-- no-transaction` header (Postgres refuses to use a new enum value inside the creating transaction).

### v0 core (Phase 1–4)

| # | Migration dir | What it creates |
|---|---|---|
| 1 | `2026-04-15-100000_add_governance_enums` | 11 enum types: `case_status`, `case_target_type`, `case_severity`, `evidence_visibility`, `jury_assignment_status`, `jury_decision`, `sanction_scope`, `sanction_action`, `appeal_status`, `reputation_dimension`, `attestation_type`. (Pure `CREATE TYPE`; no tables.) |
| 2 | `2026-04-15-100100_add_governance_core` | Tables `moderation_case`, `case_evidence`, `sanction`, `appeal`, `public_case_log`. Indexes `idx_moderation_case_status_created`, `idx_public_case_log_community_published`. |
| 3 | `2026-04-15-100200_add_jury_system` | Tables `jury_pool`, `jury_assignment`, `jury_vote`. Unique constraints on `(community_id, person_id)`, `(case_id, person_id)`, `(case_id, juror_id)`. |
| 4 | `2026-04-15-100300_add_reputation_and_surety` | Tables `surety`, `endorsement`, `reputation_event`, `reputation_snapshot`. |
| 5 | `2026-04-15-100400_add_actor_pseudonym` | Table `actor_pseudonym` (GDPR pseudonymisation, ADR-015). `person_id` + `pseudonym` both UNIQUE. |
| 6 | `2026-04-15-100500_add_governance_log` | Table `governance_log` (BIGSERIAL hash chain). `CREATE EXTENSION pgcrypto`. Indexes on `created_at`, `entry_kind`. |
| 7 | `2026-04-18-000000_add_governance_config` | Table `governance_config` (append-only typed key-value), view `governance_config_current` (DISTINCT ON latest per `(scope,key)`), `CHECK governance_config_typed`. Also `ALTER reputation_snapshot ADD can_sponsor` + partial unique index. **Seeds 34 instance-scoped config keys.** |
| 8 | `2026-04-18-000100_add_person_membership_state` | Enum `membership_state`. `ALTER person ADD membership_state NOT NULL DEFAULT 'member'`. Deferred-enforcement (OQ-016) — no v0 handler reads it. |
| 9 | `2026-04-19-000000_add_restoration_sanction_variant` | `ALTER TYPE sanction_action ADD VALUE 'Restoration'` (`-- no-transaction`). |
| 10 | `2026-04-20-000000_add_governance_log_notify` | Trigger fn `governance_log_notify()` + trigger (initially `AFTER INSERT`). |
| 11 | `2026-04-20-000100_fix_governance_log_notify_trigger_after_sign` | Replaces fn + trigger: now fires `AFTER UPDATE OF signature` guarded by `OLD.signature IS NULL AND NEW.signature IS NOT NULL` (PR #10 F14 — notify on the sign transition, not on insert). |
| 12 | `2026-04-21-000000_add_federation_attestations` | Tables `federation_attestation`, `remote_sanction_notice` (outbound-federation Phase 6). Indexes on subject/actor/target/source. |

### v1 admin / rule-set / config (v1-AD-*)

| # | Migration dir | What it creates |
|---|---|---|
| 13 | `2026-04-22-000000_add_rule_set_versions` | Table `rule_set_version` (self-referential version chain; `UNIQUE (community_id, version)`). |
| 14 | `2026-04-22-000100_add_sponsor_allowlist` | Table `sponsor_allowlist` (`UNIQUE (community_id, person_id)`). |
| 15 | `2026-04-22-000200_add_case_applied_config_snapshot` | `ALTER moderation_case ADD applied_config_snapshot JSONB`, `ADD rule_set_version_id INTEGER REFERENCES rule_set_version`. Partial index `idx_moderation_case_rule_set_version`. |
| 16 | `2026-04-22-000300_seed_v1_config_keys` | **Seeds 27 instance-scoped config keys** (jury severity thresholds, deadline window, decay half-lives, onboarding, federation toggles, rule-set propagation, dashboard flags). `rule_set.active_version_id` is deliberately NOT seeded (absence = "no active version"). |
| 17 | `2026-04-22-005541_update_modlog_check_constraint` | Stock-Lemmy `modlog` CHECK-constraint rebuild + backfill (per-`kind` `num_nonnulls` discriminator). No governance enum/table content; included for completeness. |

### v1 jury mechanics (v1-JM-*)

| # | Migration dir | What it creates |
|---|---|---|
| 18 | `2026-04-23-000000_add_jury_mechanics_enums` | Enums `severity_tier` (Minor/Moderate/Severe), `case_status_tier` (Founder/Regular/Probation), `jury_assignment_role` (Original/Appeal). |
| 19 | `2026-04-23-000050_add_jury_constraint_relaxation_reason_enum` | Enum `jury_constraint_relaxation_reason` (SmallPool/ClusterPressure/ClusterPressureExhausted/AdminOverride) — bounded vocabulary, **no `Other`** (PR #92 cr-9, ADR-015). |
| 20 | `2026-04-23-000100_add_jury_mechanics_columns` | `moderation_case` +6 cols (`severity_tier`, `status_tier`, `panel_size_snapshot`, `quorum_snapshot`, `threshold_count_snapshot`, `appeal_window_expires_at`); `jury_assignment` +2 cols (`selected_under_constraints`, `role`); table `jury_constraint_violation_log`. Backfill pins v0 cases to 5/3/3 (PRD §8.4). Single-transaction. |
| 21 | `2026-04-23-000200_seed_v1_jm_config_keys` | **Seeds 27 config keys** (panel-size cascade `jury.panel_size.<status>.<severity>`, quorum/threshold fractions per severity, diversity constraints, appeal panel multipliers). |

### v1 appeals (v1-JM-d)

| # | Migration dir | What it creates |
|---|---|---|
| 22 | `2026-04-27-000000_add_appeal_requester_role_enum` | Enum `appeal_requester_role` (Defendant/OriginalReporter). |
| 23 | `2026-04-27-000100_add_appeals_v1_columns` | `appeal` +3 cols (`requester_role`, `panel_size_snapshot`, `threshold_count_snapshot`); `moderation_case ADD winning_decision jury_decision`. No backfill. |

### v1 sponsor liability (v1-SL-*)

| # | Migration dir | What it creates |
|---|---|---|
| 24 | `2026-05-03-000000_add_case_status_sponsor_liability_variants` | `ALTER TYPE case_status ADD VALUE` × 3: `SponsorLiabilityPending`, `SponsorLiabilityFired`, `SponsorLiabilityEscaped` (`-- no-transaction`). |
| 25 | `2026-05-03-000100_add_sponsor_liability_grace_window` | `moderation_case` +2 cols (`grace_expires_at`, `liability_escape_reason JSONB`); partial index `moderation_case_grace_expires_idx WHERE status='SponsorLiabilityPending'`; partial index `surety_sponsored_id_active WHERE revoked_at IS NULL`. **Seeds 13 config keys** (grace windows per severity, escape rules, grace-check job knobs). Bounded 24h backfill for v0 mid-flight cases. |

### v1 reputation tuning (v1-RT-r1)

| # | Migration dir | What it creates |
|---|---|---|
| 26 | `2026-05-10-000000_add_reputation_event_v1_columns` | Enum `reputation_event_source_type` (9 variants). `reputation_event` +2 cols (`dedupe_key`, `source_event_type NOT NULL DEFAULT 'Endorsement'`); partial unique index `reputation_event_dedupe_key_partial_idx WHERE dedupe_key IS NOT NULL`. |
| 27 | `2026-05-10-000100_extend_sponsor_allowlist_for_r1` | `sponsor_allowlist`: `community_id DROP NOT NULL` (NULL = instance-wide); `ADD added_by_admin_id INTEGER NOT NULL REFERENCES person`; `ADD note TEXT`. |
| 28 | `2026-05-10-000200_backfill_reputation_event_source_type` | Data backfill: refines `source_event_type` via reason-ILIKE precedence chain (idempotent, guarded by `WHERE source_event_type='Endorsement'`). |
| 29 | `2026-05-10-000300_seed_v1_rt_config_keys` | **Seeds 26 config keys** (per-dimension decay half-lives + bounds, participation/evidence deltas, rollup job knobs, `feature.reputation_v1_decay_enabled = false`). |

### v1 federation inbound (v1-federation-inbound-a)

| # | Migration dir | What it creates |
|---|---|---|
| 30 | `2026-05-17-000000_add_federation_inbound_v1` | Enums `federation_peer_trust_enum`, `federation_inbox_admin_action_enum` (lowercase values). Tables `federation_peer`, `federation_inbox_dropped_log`, `federation_inbox_nonce` (composite PK), `remote_moderation_label`. ALTERs add admin-review columns to `remote_sanction_notice` + `federation_attestation`. Pre-condition `DO` guard checks Phase-6 prerequisites. **Seeds 11 config keys** (`federation.inbound.*` trust/rate/size/replay knobs). |

> **Cumulative config-seed count invariant:** 34 (v1-AD seed at #7) + 27 (#16) + 27 (#21) + 13 (#25) + 26 (#29) = **127**, plus 11 federation-inbound (#30) = **138** instance-scoped rows seeded across the lifecycle. The reputation-snapshot/recompute job and admin-config handler assert subsets of these counts (`EXPECTED_SEED_COUNT_V1_*` consts in `crates/api/api/src/governance/config.rs`).

### ADR-017 author-as-defendant backfill

| # | Migration dir | What it does |
|---|---|---|
| 31 | `2026-06-01-000000_backfill_author_defendant` | **Data-only backfill** (no DDL). Sets `moderation_case.target_person_id` to the content author's `creator_id` for the historical post/comment-targeted tail (ADR-017), making the author a first-class defendant. **Status-scoped:** excludes `Open`/`ThresholdMet`/`EmergencyRemove` (the three statuses from which a jury can still be sized) so it cannot retroactively resize an in-flight or seated jury (ADR-010). PascalCase enum literals (`'Post'`/`'Comment'` + status tokens) per the verbatim DB convention. Reversible: `down.sql` nulls only rows whose value still equals the content author. |

---

## 2. Enums

All governance enums live in `crates/db_schema_file/src/enums.rs` as Rust `enum`s deriving `DbEnum` (under `feature = "full"`) with `ExistingTypePath = "crate::schema::sql_types::<Name>"`. **Two `DbValueStyle` conventions are in play** — get this right or rows fail to deserialize:

- **`DbValueStyle = "verbatim"` (PascalCase DB tokens)** — the majority. The Postgres enum literal is the PascalCase variant name (e.g. `'SponsorLiabilityPending'`). Serde renders snake_case (e.g. `"sponsor_liability_pending"`) for JSON/wire.
- **`DbValueStyle = "snake_case"` (lowercase DB tokens)** — `MembershipState`, `FederationPeerTrust`, `FederationInboxAdminAction`. The Postgres enum literal is lowercase snake_case (e.g. `'untrusted_receive'`). These were chosen because their values round-trip as plain config-text / wire strings.

### v0 enums (Phase 1)

| Rust enum | PG type | Style | Variants (source order; `#[default]` marked) |
|---|---|---|---|
| `CaseStatus` | `case_status` | verbatim | **Open**, ThresholdMet, JurySelection, InReview, Decided, Appealed, Closed, EmergencyRemove, AdminReview, SponsorLiabilityPending, SponsorLiabilityFired, SponsorLiabilityEscaped *(last 3 added v1-SL-a)* |
| `CaseTargetType` | `case_target_type` | verbatim | **Post**, Comment, Person, Community, RemoteInstance |
| `CaseSeverity` | `case_severity` | verbatim | Low, **Medium**, High, Critical |
| `EvidenceVisibility` | `evidence_visibility` | verbatim | **JuryOnly**, PrivateAdmin, PublicRedacted |
| `JuryAssignmentStatus` | `jury_assignment_status` | verbatim | **Selected**, Accepted, Declined, Conflicted, Submitted, Expired |
| `JuryDecision` | `jury_decision` | verbatim | **NoAction**, AdvisoryLabel, Warning, Cooldown, RemoveContent, SuspendLocalUser, SuspendCommunityMember, RecommendFederationAction |
| `SanctionScope` | `sanction_scope` | verbatim | **Community**, Instance, FederatedRecommendation |
| `SanctionAction` | `sanction_action` | verbatim | **Label**, VisibilityReduction, TemporaryRestriction, ContentRemoval, CommunityExclusion, InstanceSuspension, FederationQuarantineRecommendation, Restoration *(added v0 #9; reserved for v1 restorative actions)* |
| `AppealStatus` | `appeal_status` | verbatim | **Requested**, Accepted, Rejected, Decided |
| `ReputationDimension` | `reputation_dimension` | verbatim | **ReportingAccuracy**, JuryReliability, ParticipationConsistency, EndorsementStrength |
| `AttestationType` | `attestation_type` | verbatim | **TrustedReporter**, JuryEligible, SanctionNotice, QuarantineRecommendation |

### v1 enums

| Rust enum | PG type | Style | Variants | Added by |
|---|---|---|---|---|
| `MembershipState` | `membership_state` | **snake_case** | **Member**, Provisional, Suspended → DB `member`/`provisional`/`suspended` | v0 #8 (deferred-enforcement, OQ-016) |
| `ReputationEventSourceType` | `reputation_event_source_type` | verbatim | **Endorsement**, JuryVote, SponsorLiability, FounderSeed, ParticipationCron, DormancyCron, VoteOutcome, EvidenceQuality, ManualSeed | v1-RT-r1 |
| `SeverityTier` | `severity_tier` | verbatim | **Minor**, Moderate, Severe | v1-JM-a |
| `CaseStatusTier` | `case_status_tier` | verbatim | Founder, **Regular**, Probation | v1-JM-a |
| `JuryAssignmentRole` | `jury_assignment_role` | verbatim | **Original**, Appeal | v1-JM-a |
| `AppealRequesterRole` | `appeal_requester_role` | verbatim | **Defendant**, OriginalReporter | v1-JM-d |
| `JuryConstraintRelaxationReason` | `jury_constraint_relaxation_reason` | verbatim | **SmallPool**, ClusterPressure, ClusterPressureExhausted, AdminOverride | v1-JM-a (PR #92 cr-9) |
| `FederationPeerTrust` | `federation_peer_trust_enum` | **snake_case** | **Unknown**, Allowlisted, UntrustedReceive, Blocklisted → DB `unknown`/`allowlisted`/`untrusted_receive`/`blocklisted` | v1-fed-inbound-a |
| `FederationInboxAdminAction` | `federation_inbox_admin_action_enum` | **snake_case** | **Unreviewed**, CrossLinked, Dismissed → DB `unreviewed`/`cross_linked`/`dismissed` | v1-fed-inbound-a |

> **`SeverityTier` ≠ `CaseSeverity`.** `severity` (`CaseSeverity`: Low/Medium/High/Critical) is the report's classification; `severity_tier` (`SeverityTier`: Minor/Moderate/Severe) is the *procedural* tier that, together with `status_tier` (`CaseStatusTier`: Founder/Regular/Probation), keys the `jury.panel_size.<status>.<severity>` config cascade. Both tiers are frozen at `admin_assign_jury` time (ADR-010 — no retroactive invalidation of in-flight juries).

> **`entry_kind` is NOT an enum.** `governance_log.entry_kind` is plain `TEXT`. The 55 valid kinds are `pub const ENTRY_KIND_*: &str` constants in `crates/db_schema/src/source/governance/governance_log.rs` and the authoritative registry is `.claude/rules/governance-log-entry-kind-registry.md` (§13).

---

## 3. Diesel models

**25 governance model files** under `crates/db_schema/src/source/governance/` (`mod.rs` aggregates them). All read structs derive `Identifiable, Queryable, Selectable` (under `feature = "full"`) + `PartialEq, Eq, Serialize, Deserialize, Debug, Clone`, `#[skip_serializing_none]`, `check_for_backend(Pg)`, and (most) `ts_rs::TS`. Newtype IDs (`ModerationCaseId`, `PersonId`, etc.) wrap the integer PKs.

> **Insert-form vs read-struct asymmetry (load-bearing).** Many models have enum/timestamp columns that are **NON-Option on the read struct but `Option<…>` on the InsertForm** because the DB supplies a DEFAULT. For each such column, the SQL column MUST be `NOT NULL DEFAULT …` or the read-side `Queryable` fails at runtime on any row inserted via the default path. Verified instances: `moderation_case.{severity_tier, status_tier}`, `reputation_event.source_event_type`, `sanction.active`, `remote_sanction_notice.{peer_trust_level_at_receipt, admin_action}`, `remote_moderation_label.{peer_trust_level_at_receipt, admin_action}`, `federation_attestation.admin_action`, `federation_peer.notes`. All confirmed `NOT NULL DEFAULT` in their DDL.

> **Append-only models (deliberately no `AsChangeset`).** `governance_log`, `governance_config`, `rule_set_version`, `jury_constraint_violation_log`, `federation_inbox_dropped_log`, `federation_inbox_nonce`, `federation_peer`, `remote_moderation_label`, `sponsor_allowlist` omit `AsChangeset` on their InsertForm (rows are insert-or-delete, never mutated in place — audit-trail / hash-chain integrity).

### Core case workflow

- **`moderation_case.rs` → `ModerationCase`** (table `moderation_case`). The central artefact. Fields: `id`, `community_id: Option<CommunityId>`, `creator_id: Option<PersonId>`, `target_type: CaseTargetType`, `target_post_id/comment_id/person_id/community_id: Option<…>`, `target_remote_url: Option<String>`, `reason_code: String`, `severity: CaseSeverity`, `status: CaseStatus`, `threshold_score: i64`, `opened_at`, `decided_at: Option<…>`, `closed_at: Option<…>`. **v1 additions:** `applied_config_snapshot: Option<Value>` (v1-AD-a), `rule_set_version_id: Option<RuleSetVersionId>` (v1-AD-a/c), `severity_tier: SeverityTier` + `status_tier: CaseStatusTier` + `panel_size_snapshot/quorum_snapshot/threshold_count_snapshot: Option<i32>` + `appeal_window_expires_at: Option<…>` (v1-JM-a), `winning_decision: Option<JuryDecision>` (v1-JM-d), `grace_expires_at: Option<…>` + `liability_escape_reason: Option<Value>` (v1-SL-a). InsertForm + `AsChangeset` (snapshot/tier writes flow through `update().set()`).
  > **`target_person_id` semantics (ADR-017).** For person-targeted cases this is the reported person. **For `Post`/`Comment`-targeted cases it is now the content author's `creator_id`** — the case is dual-populated (`target_post_id`/`target_comment_id` *and* `target_person_id` both set). This makes the author a first-class defendant for appeal (`request_appeal` resolves the defendant via `target_person_id == caller`), sanction inheritance (`submit_jury_vote` §8.5), reputation ban-math, sponsor-liability, and juror exclusion. Populated at report-creation (`create_report::resolve_target`), emergency-removal (`admin_emergency_remove`), and for history via migration #31. Federation is agnostic — `publish_sanction_notice` resolves the AP URL from `target_post_id`/`target_comment_id`, never `target_person_id`. Community-targeted cases leave it NULL (no single author).
- **`case_evidence.rs` → `CaseEvidence`** — `case_id`, `uploader_id`, `storage_key`, `sha256`, `mime_type`, `visibility: EvidenceVisibility`, `created_at`.
- **`sanction.rs` → `Sanction`** — `case_id`, `scope: SanctionScope`, `action: SanctionAction`, `target_*: Option<…>`, `starts_at`, `ends_at: Option<…>`, `active: bool` (InsertForm `Option<bool>`).
- **`appeal.rs` → `Appeal`** — `case_id`, `requester_id`, `reason`, `status: AppealStatus`, `created_at`, `decided_at: Option<…>`. **v1-JM-d:** `requester_role: AppealRequesterRole`, `panel_size_snapshot: Option<i32>`, `threshold_count_snapshot: Option<i32>` (NULL until `select_appeal_panel` runs).
- **`public_case_log.rs` → `PublicCaseLog`** — `case_id`, `community_id: Option<…>`, `summary`, `rationale_redacted: Option<String>`, `published_at`. `summary` + `rationale_redacted` pass through `redaction::scrub` before persist (ADR-015).

### Jury

- **`jury_pool.rs` → `JuryPool`** — `community_id: Option<…>`, `person_id`, `eligible_from`, `created_at`.
- **`jury_assignment.rs` → `JuryAssignment`** — `case_id`, `person_id`, `status: JuryAssignmentStatus`, `selected_at`, `responded_at/submitted_at: Option<…>`. **v1-JM-a:** `selected_under_constraints: Option<Value>` (constraint *names* only, no PII — PRD Watch 10), `role: JuryAssignmentRole` (`Original` vs `Appeal` distinguishes panels on the same case).
- **`jury_vote.rs` → `JuryVote`** — `case_id`, `juror_id`, `decision: JuryDecision`, `rationale: Option<String>`, `submitted_at`.
- **`jury_constraint_violation_log.rs` → `JuryConstraintViolationLog`** (v1-JM-a) — `case_id`, `constraint_name: String`, `reason_code: JuryConstraintRelaxationReason`, `relaxation_metadata: Option<Value>` (bounded fields ONLY — never free-text), `pool_size_at_relax: i32`, `panel_size_target: i32`, `relaxed_at`. Append-only.

### Reputation, surety, endorsement

- **`reputation_event.rs` → `ReputationEvent`** — `person_id`, `community_id: Option<…>`, `dimension: ReputationDimension`, `delta: i32`, `source_case_id: Option<ModerationCaseId>`, `source_report_id: Option<i32>` (**raw `i32`, no newtype** — intentional, no `report` PK to wrap in v0), `reason: String`, `created_at`, `expires_at: Option<…>`. **v1-RT-r1:** `dedupe_key: Option<String>` (idempotency for cron events), `source_event_type: ReputationEventSourceType`.
- **`reputation_snapshot.rs` → `ReputationSnapshot`** — `person_id`, `community_id: Option<…>`, four `i32` scores (`reporting_accuracy`, `jury_reliability`, `participation_consistency`, `endorsement_strength`), `jury_eligible: bool`, `trusted_reporter: bool`, `calculated_at`, `can_sponsor: bool` (v0 #7 — populated but **not read by any v0 handler**, OQ-014; `lint-no-can-sponsor-read.sh` enforces silence).
- **`surety.rs` → `Surety`** — `sponsor_id`, `sponsored_id`, `community_id: Option<…>`, `created_at`, `revoked_at: Option<…>`.
- **`endorsement.rs` → `Endorsement`** — `from_person_id`, `to_person_id`, `community_id: Option<…>`, `created_at`, `revoked_at: Option<…>`.

### Config, log, pseudonym, rule-sets, allowlist

- **`governance_config.rs` → `GovernanceConfig`** — `scope`, `key`, `value_type`, `value_int: Option<i64>`, `value_float: Option<f64>`, `value_bool: Option<bool>`, `value_text: Option<String>`, `valid_from`, `updated_by: Option<PersonId>`. **Read struct derives `PartialEq` but NOT `Eq`** (the `f64` field). Append-only — edits INSERT a new `(scope, key, valid_from)` row; the `governance_config_current` view surfaces the latest. `CHECK governance_config_typed` enforces exactly-one-of `value_*` matches `value_type`.
- **`governance_log.rs` → `GovernanceLog`** — `id: GovernanceLogId` (BIGSERIAL), `prev_hash: Vec<u8>`, `entry_hash: Vec<u8>` (both trigger-populated bytea), `entry_kind: String`, `payload: Value` (JSONB), `actor_pseudonym: Option<String>`, `created_at`, `signature: Option<Vec<u8>>` (UPDATE-populated). InsertForm exposes ONLY `entry_kind`/`payload`/`actor_pseudonym`. **`append(pool, entry_kind, payload, actor_pseudonym)` is the only sanctioned insert path** — payload scrubbed via `scrub_json` (ADR-015), INSERT + signature UPDATE wrapped in `run_transaction` (ADR-008). `load_signing_key()` reads ed25519 from env `GOVERNANCE_LOG_SIGNING_KEY` (32-byte hex seed); missing/malformed = hard error, no unsigned fallback. No `ts_rs` derive.
- **`actor_pseudonym.rs` → `ActorPseudonym`** — `person_id`, `pseudonym`, `created_at`. The pseudonymous handle used in governance-log entries (ADR-015).
- **`rule_set_version.rs` → `RuleSetVersion`** (v1-AD-c) — `community_id: CommunityId` (NON-Option), `version: i32`, `parent_id: Option<RuleSetVersionId>` (self-ref chain), `text_sha256: Vec<u8>` (canonical 32-byte SHA-256), `rule_text: String`, `created_at`, `created_by: Option<PersonId>`. Append-only (editing past text would break grandfathered juries).
- **`sponsor_allowlist.rs` → `SponsorAllowlist`** — `community_id: Option<CommunityId>` (NULL = instance-wide, relaxed in v1-RT-r1), `person_id`, `created_at`, `added_by_admin_id: PersonId` (v1-RT-r1, NON-Option audit trail), `note: Option<String>` (v1-RT-r1). Add-or-delete, no `AsChangeset`.

### Federation (outbound + inbound)

- **`federation_attestation.rs` → `FederationAttestation`** — `actor_url`, `subject_url`, `attestation_type: AttestationType`, `valid_until: Option<…>`, `created_at`, `signature: String`. **v1-fed-inbound-a admin-review cols:** `source_instance: Option<String>`, `received_at: Option<…>`, `peer_trust_level_at_receipt: Option<FederationPeerTrust>`, `admin_reviewed_at: Option<…>`, `admin_action: FederationInboxAdminAction`, `dismissal_rationale: Option<String>`. Has a separate `…UpdateForm`.
- **`remote_sanction_notice.rs` → `RemoteSanctionNotice`** — `source_instance`, `target_url`, `action: SanctionAction`, `scope: SanctionScope`, `summary`, `published_at`, `signature`, `local_case_id: Option<ModerationCaseId>` (**stays NULL until an admin opens a local case** — ADR-006 advisory-only), `received_at`. **v1-fed-inbound-a:** `peer_trust_level_at_receipt: FederationPeerTrust`, `admin_reviewed_at`, `admin_action`, `dismissal_rationale`. InsertForm + `AsChangeset`; `…UpdateForm` present.
- **`remote_moderation_label.rs` → `RemoteModerationLabel`** (v1-fed-inbound-a) — `source_instance`, `actor_url`, `target_url`, `label`, `summary: Option<String>`, `published_at`, `signature`, `local_case_id: Option<…>`, `received_at`, `peer_trust_level_at_receipt: FederationPeerTrust`, `admin_reviewed_at`, `admin_action`, `dismissal_rationale`. InsertForm is `Insertable`-only (no `AsChangeset`); `…UpdateForm` uses double-`Option` for nullable columns.
- **`federation_peer.rs` → `FederationPeer`** (v1-fed-inbound-a) — `#[diesel(primary_key(instance_id))]`: `instance_id: InstanceId`, `trust_level: FederationPeerTrust`, `added_at`, `added_by_actor: Option<String>`, `notes: Value` (NON-Option), `updated_at`. Free fns: `federation_inbox_check_peer_trust(peer_domain, conn)` (returns `Unknown` when absent), `federation_peer_upsert_trust(...)`.
- **`federation_inbox_dropped_log.rs` → `FederationInboxDroppedLog`** (v1-fed-inbound-a) — `source_instance`, `activity_id: Option<String>`, `drop_reason`, `payload_excerpt: Option<String>`, `dropped_at`. Insert-only append log.
- **`federation_inbox_nonce.rs` → `FederationInboxNonce`** (v1-fed-inbound-a) — `peer_instance`, `activity_id`, `seen_at`. **No `Identifiable` / no `ts_rs`** (composite PK `(peer_instance, activity_id)`). Free fn `delete_older_than(window_days, conn)` (replay cleanup; wired by v1-federation-inbound-b cron).

### Helper module (not a model)

- **`redaction.rs`** — no Diesel struct. `scrub(text) -> String` (strips fediverse mentions, emails, profile URLs → `[redacted]`); `scrub_json(value) -> Value` (recursively scrubs string values, keys preserved). Lives in `lemmy_db_schema` so `governance_log::append` can call it (Phase-6 DQ-6.6 relocation; old path re-exported). GDPR/ADR-015 §4.2 invariant: every string into `public_case_log` / `governance_log.payload` passes through these.

---

## 4. Read models (db_views)

Four governance view crates under `crates/db_views/`. None of the multi-source views derive `Selectable` (they aggregate via tuple-load + map / separate COUNT round-trips).

### `governance_case` — three view types

- **`GovernanceCaseSummaryView`** (list rows; backs the create-report response + `list_cases`): `case_id: i32`, `status`, `severity`, `reason_code`, `opened_at`, `community_id: Option<i32>`, `community_name: Option<String>`, `target_type`, `reporter_count: i64` *(drift stub)*, `jury_needed: i32` *(hardcoded `5` per [05 §3])*, `jury_submitted: i32`.
- **`GovernanceCaseDetailRow`** (`feature="full"`): `case_row: ModerationCase`, `evidence_count: i64`, `appeal_status: Option<AppealStatus>`, `target_creator_id: Option<i32>`.
- **`GovernanceCaseDetailView`** (`feature="full"`): `row: GovernanceCaseDetailRow`, `sanctions: Vec<Sanction>`.

### `governance_modlog` — two types

- **`GovernanceModlogView`** (one row per `public_case_log` entry; backs `list_modlog`, public/no-auth): `case_id`, `community_id`, `community_name`, `decision: Option<JuryDecision>` *(drift — None pre-Phase-4)*, `sanction_action: Option<SanctionAction>` *(drift)*, `summary`, `published_at`, `appealed: bool` (derived via separate appeal IN-list query).
- **`CapabilityChangeLogEntry`** (one `governance_log` row as an observability entry; backs admin tooling / dashboard): `id: i64`, `entry_kind`, `payload: Value`, `actor_pseudonym: Option<String>`, `created_at`, `signature: Option<Vec<u8>>`.

### `jury_queue` — one view

- **`JuryQueueView`** (backs `list_my_jury_queue`; two semantics — assigned vs available — depending on the producing query): `case_id`, `severity`, `reason_code`, `opened_at`, `deadline_at: Option<…>` *(drift — None in Phase 2a)*, `community_id`, `community_name`.

### `reputation` — two views

- **`ReputationSummaryView`** (backs `get_my_reputation`): `person_id: i32`, `community_id: Option<i32>`, four `i32` dimension scores **all `#[serde(skip)]` + `ts(skip)` per ADR-005** (raw scores never leave the server), `jury_eligible: bool`, `trusted_reporter: bool`, `calculated_at`, `active_sanctions: i64` (derived COUNT). `can_sponsor` is intentionally absent (OQ-014, lint-enforced). Has `From<&ReputationSnapshot>`.
- **`EndorsementSummaryView`**: `person_id`, `inbound_endorsements: i64`, `outbound_endorsements: i64`, `active_sureties_inbound: i64`, `active_sureties_outbound: i64`.

---

## 5. API DTOs

Request/response types live in `crates/api/api_common/src/governance.rs`. Grouped by feature area; every field listed.

### Reports + cases

- **`CreateGovernanceReport`** → `community_id: Option<CommunityId>`, `target_type: CaseTargetType`, `target_id: i32` (cast by `target_type`), `reason_code: String`, `description: Option<String>`.
- **`CreateGovernanceReportResponse`** → `case_id: Option<ModerationCaseId>`, `threshold_met: bool`, `case: GovernanceCaseSummaryView`.
- **`GetGovernanceCase`** → `case_id: ModerationCaseId`.
- **`ListGovernanceCases`** → `community_id: Option<…>`, `status: Option<CaseStatus>`, `page: Option<i64>`, `limit: Option<i64>`.
- **`ListGovernanceCasesResponse`** → `cases: Vec<GovernanceCaseSummaryView>`.

### Jury

- **`SubmitJuryVote`** → `case_id`, `decision: JuryDecision`, `rationale: Option<String>`.
- **`SubmitJuryVoteResponse`** → `vote_recorded: bool`, `case_decided: bool`, `decision: Option<JuryDecision>`.
- **`AcceptJuryAssignment`** → `case_id`; **`…Response`** → `case_id`, `accepted: bool`.
- **`DeclineJuryAssignment`** → `case_id`, `reason: Option<String>`; **`…Response`** → `case_id`, `declined: bool`, `replacement_person_id: Option<PersonId>`.

### Appeals

- **`RequestAppeal`** → `case_id`, `reason: String`; **`…Response`** → `appeal_id: AppealId`, `case_id`.

### Public log

- **`ListGovernanceModlog`** (public, no auth) → `community_id: Option<…>`, `page: Option<i64>`, `limit: Option<i64>`.

### Reputation / trust

- **`GetMyReputation`** → `community_id: Option<…>`; **`…Response`** → `view: ReputationSummaryView`.
- **`CreateEndorsement`** → `person_id: PersonId`, `community_id: Option<…>`; **`…Response`** → `endorsement_id: EndorsementId`, `surety_created: bool`.
- **`RevokeEndorsement`** → `endorsement_id: EndorsementId`, `reason: String`; **`…Response`** → `endorsement_id`, `revoked_at: DateTime<Utc>`, `liability_chain_severed_for_cases: Vec<ModerationCaseId>`.

### Admin backstops

- **`AdminAssignJury`** → `case_id`; **`…Response`** → `case_id`, `assigned_person_ids: Vec<PersonId>`.
- **`AdminCloseCase`** → `case_id`, `reason: String`; **`…Response`** → `case_id`, `closed: bool`.
- **`FlagBadFaithEmergencyReport`** → `case_id`; **`…Response`** → `case_id`, `flagged: bool`.
- **`AdminTriggerAppealRejury`** → `case_id`, `step_up_token: Option<String>` (read-but-ignored in v1; reserved for v2 step-up auth); **`…Response`** → `case_id`, `appeal_id: AppealId`, `panel_person_ids: Vec<PersonId>`.

### Admin observability — `admin_reputation_stats`

- **`AdminReputationStats`** → `community_id: Option<…>`.
- **`AdminReputationStatsResponse`** → `buckets: ReputationBuckets`, `thresholds_current: ThresholdsSnapshot`, `capability_counts: CapabilityCounts`, `founder_event_stats: FounderEventStats`, `calculated_at`.
  - `ReputationBuckets` → four `[i64; 5]` histograms (one per dimension).
  - `ThresholdsSnapshot` → `jury_reliability/reporting_accuracy/endorsement_strength: i64`.
  - `CapabilityCounts` → `jury_eligible_count/trusted_reporter_count/can_sponsor_count: i64`.
  - `FounderEventStats` → `active_count/expired_count: i64`.
- **`AdminReputationRollup`** → `person_id: PersonId` (**required** — the per-person scoping id; `GET /governance/admin/reputation/rollup`).
- **`AdminReputationRollupResponse`** → `rollup: Option<ReputationSnapshot>` (instance-wide snapshot, `community_id IS NULL`; `None` until the rollup cron has run or if all the person's communities are banned), `contributing: Vec<ReputationSnapshot>` (the per-community snapshots that fed the rollup; may be empty).

### Admin config (v1-AD-b)

- **`AdminSetConfig`** → `key`, `value_type`, `value: Value`, `scope` (`"instance"`|`"community:<id>"`), `apply_at: Option<String>`, `dry_run: Option<bool>`, `reason: String`.
- **`AdminSetConfigResponse`** → `applied: bool`, `config_id: Option<i64>`, `governance_log_id: Option<i64>`, `preview: ConfigChangePreview`, `applied_at: Option<…>`.
  - `ConfigChangePreview` → `previous/new: ConfigValueWithProvenance`, `downstream_impact: Value`.
  - `ConfigValueWithProvenance` → `value: Value`, `effective_from: String`.
- **`AdminGetConfig`** → `key: Option<String>`, `community_id: Option<…>`; **`…Response`** → `entries: Vec<AdminConfigEntry>`.
  - `AdminConfigEntry` → `key`, `value_type`, `value: Value`, `effective_from`, `scope`, `requires_re_jury: bool`, `requires_step_up: bool`, `apply_at_default`, `description`, `doc_anchor`, `valid_range: Option<(f64,f64)>`, `valid_enum: Option<Vec<String>>`.
- **`AdminGetConfigAudit`** → `key/scope/actor_pseudonym: Option<String>`, `since/until: Option<DateTime<Utc>>`, `page/limit: Option<i64>`.
  - `AdminConfigAuditEntry` → `id: i64`, `entry_kind`, `scope`, `key`, `value_type`, `previous_value: Option<Value>`, `previous_from: Option<String>`, `new_value: Value`, `reason`, `actor_pseudonym: Option<String>`, `created_at`, `signature: Option<Vec<u8>>`, `denial_reason: Option<String>`.

### Rule-set versioning (v1-AD-c)

- **`AdminCreateRuleSet`** → `community_id: CommunityId`, `rule_text: String`, `parent_id: Option<i32>`, `reason: String`.
- **`AdminCreateRuleSetResponse`** → `rule_set_version_id: i32`, `version: i32`, `config_id: Option<i64>`, `governance_log_id: i64`, `created_at`.
- **`AdminListRuleSetsRequest`** → `community_id: CommunityId`; **`…Response`** → `versions: Vec<RuleSetVersionView>`, `active_version_id: Option<i32>`.
  - `RuleSetVersionView` → `id`, `community_id`, `version`, `parent_id: Option<i32>`, `text_sha256_hex: String`, `rule_text`, `created_at`, `created_by_pseudonym: Option<String>`.

### Admin dashboard aggregate (v1-AD-d)

- **`AdminDashboardResponse`** → `active_cases: ActiveCasesSummary`, `jury_queue: JuryQueueSummary`, `recent_config_changes: Vec<AdminConfigAuditEntry>`, `federation: FederationSummary`, `reputation: AdminReputationStatsResponse`, `rule_sets: RuleSetSummary`, `calculated_at`.
  - `ActiveCasesSummary` → `by_status: BTreeMap<String,i64>`, `total_active: i64` (excludes Decided/Closed/EmergencyRemove).
  - `JuryQueueSummary` → `pending_accept/accepted/submitted: i64`.
  - `FederationSummary` → `active/expired/total: i64`.
  - `RuleSetSummary` → `communities_with_rule_sets: i64`, `total_versions: i64`, `per_community: Vec<PerCommunityActiveRuleSet>` (bounded 100).
  - `PerCommunityActiveRuleSet` → `community_id`, `active_version_id: Option<i32>`.

> **HTML/SSE endpoints have no DTO struct.** `admin_dashboard_html`, `admin_audit_html`, and `admin_audit_stream` emit `text/html` / `text/event-stream` directly — no request/response struct in `governance.rs`.

---

## 6. Handlers

Governance handlers live in `crates/api/api/src/governance/` (32 files) and `crates/api/api_crud/src/governance/` (the CRUD-style `create_report`, `create_endorsement`, `revoke_endorsement`, `request_appeal`). Notable internal helpers (not endpoints): `actor_pseudonym_helper` (get-or-create pseudonym), `jury_common` (panel-pick + replacement), `sponsor_liability` / `sponsor_liability_grace` (liability compute + grace-window), `participation_cron` / `appeal_window_expiry` (scheduled jobs), `case_open_snapshot` / `audit_projection` (config-snapshot + audit read).

Admin enforcement is **in the handlers** (capability checks reading `reputation_snapshot` flags + admin role per CLAUDE.md authz model), not a route guard.

---

## 7. Routes

**27 governance routes, all under `/api/v4`** (the live prefix — there is **no `/api/v3`**; older PRD/comment references to v3 are stale). Built inline in `crates/api/routes/src/lib.rs` `config()` via nested actix-web `scope(...)`/`.route(...)` (NOT a macro, NOT a separate `routes/src/governance.rs` — that file does not exist). The whole `/governance` scope is wrapped with `rate_limit.post()`; the outer `/api/v4` scope's `rate_limit.message()` is the ceiling.

### Top-level `/api/v4/governance`

| Method | Path | Handler |
|---|---|---|
| POST | `/governance/report` | `create_report` |
| POST | `/governance/endorsement` | `create_endorsement` |
| POST | `/governance/endorsement/revoke` | `revoke_endorsement` |
| POST | `/governance/appeal` | `request_appeal` |
| GET | `/governance/case` | `get_case` |
| GET | `/governance/cases` | `list_cases` |
| GET | `/governance/modlog` | `list_modlog` *(public)* |
| GET | `/governance/reputation/me` | `get_my_reputation` |

### Jury `/api/v4/governance/jury`

| Method | Path | Handler |
|---|---|---|
| GET | `/governance/jury/me` | `list_my_jury_queue` |
| POST | `/governance/jury/accept` | `accept_jury_assignment` |
| POST | `/governance/jury/decline` | `decline_jury_assignment` |
| POST | `/governance/jury/vote` | `submit_jury_vote` |

### Admin `/api/v4/governance/admin` (admin-only sub-tree)

| Method | Path | Handler |
|---|---|---|
| POST | `/governance/admin/assign-jury` | `admin_assign_jury` |
| POST | `/governance/admin/close-case` | `admin_close_case` |
| POST | `/governance/admin/trigger-appeal-rejury` | `admin_trigger_appeal_rejury` |
| GET | `/governance/admin/reputation-stats` | `admin_reputation_stats` |
| GET | `/governance/admin/reputation/rollup` | `admin_reputation_rollup` |
| GET | `/governance/admin/dashboard` | `admin_dashboard` |
| GET | `/governance/admin/dashboard/view` | `admin_dashboard_html` |
| POST | `/governance/admin/config` | `admin_set_config` |
| GET | `/governance/admin/config` | `admin_get_config` |
| GET | `/governance/admin/config/audit` | `admin_get_config_audit` |
| POST | `/governance/admin/rule-sets` | `admin_create_rule_set` |
| GET | `/governance/admin/rule-sets` | `admin_list_rule_sets` |
| GET | `/governance/admin/audit/stream` | `admin_audit_stream` *(SSE)* |
| GET | `/governance/admin/audit/view` | `admin_audit_html` |
| POST | `/governance/admin/emergency-remove/flag-bad-faith` | `flag_bad_faith_emergency_report` |

> **v0 11-endpoint baseline (per [05 §2]):** the original v0 scope was the 8 top-level + jury endpoints (report, endorsement, appeal, case, cases, modlog, reputation/me, jury/me, jury/accept, jury/decline, jury/vote). The `/admin/*` sub-tree, `endorsement/revoke`, and the config/rule-set/dashboard/audit endpoints are all v1 (v1-AD-*, v1-RT-r1, v1-SL-b) additions.

---

## 8. Aggregation rules (config-driven; NOT "v0 deliberately simple")

Reputation, jury panel-sizing, quorum/threshold, grace windows, and federation-inbound limits are **NOT hardcoded** — they live in `governance_config` (scope `instance` by default; `community:<id>` overrides supported) and are read at runtime via `config::get_int/get_float/get_bool`. The full v0+v1 seed surface is 138 instance-scoped rows (§1 cumulative invariant). Key groups:

- **Jury panel-size cascade** (v1-JM seed): `jury.panel_size.<status_tier>.<severity_tier>` — 9 keys (e.g. `jury.panel_size.regular.minor = 5`, `…founder.severe = 9`, `…probation.minor = 3`). `jury.quorum_fraction.<severity>` + `jury.threshold_fraction.<severity>` set the per-case quorum/threshold (snapshotted to `moderation_case.{quorum_snapshot, threshold_count_snapshot, panel_size_snapshot}` at `admin_assign_jury`).
- **Diversity constraints** (v1-JM seed): `jury.constraints.*` (no-same-sponsor-cluster, geographic-diversity, no-recent-repeat, cooldown days, max-retries-before-relax). Relaxations log `jury_constraint_violation_log` + `ENTRY_KIND_JURY_CONSTRAINT_RELAXED`.
- **Reputation deltas + decay** (v1-AD + v1-RT seed): `deltas.*` (juror aligned/outlier, reporter upheld/dismissed, endorsement, sponsor-liability minor/moderate/severe, participation, evidence). `decay.<dimension>.{positive,negative}_half_life_days`. `bounds.<dimension>.{floor,ceiling}`. `feature.reputation_v1_decay_enabled` gates the v1 decay path (seeded `false`).
- **Sponsor-liability grace** (v1-SL seed): `liability.grace_window_{minor,moderate,severe}_hours` (24/72/168), min/max clamps, `liability.restoration_escapes_liability`, `liability.multi_sponsor_escape_rule`, grace-check job knobs.
- **Federation inbound** (v1-fed-inbound seed): `federation.inbound.{default_trust_for_new_peers, per_peer_rate_per_hour, per_actor_attestation_rate_per_hour, per_peer_storage_cap, max_payload_bytes_*, replay_window_days, …}`.

### `submit_jury_vote` lifecycle (the canonical aggregation flow)

`crates/api/api/src/governance/submit_jury_vote.rs`. The handler opens one `conn.run_transaction` and delegates to `process_vote` (original jury) or `process_appeal_vote` (if `jury_assignment.role == Appeal`). **All writes are in one transaction; any failure rolls back the whole thing.** Original-jury path:

1. **Auth** — `check_local_user_valid`; resolve `juror_id` + pseudonym; begin transaction.
2. **Assignment check** — `SELECT jury_assignment WHERE case_id + person_id + status = Accepted` (double-vote guard; not-found → error).
3. **Lock** — `SELECT … FOR UPDATE` on the `moderation_case` row **before** the vote INSERT (lock-ordering fix, PR #98 cr-2 / DQ #50 — concurrent voters serialise on the row lock instead of deadlocking). Loads `quorum_snapshot`, `panel_size_snapshot`, `threshold_count_snapshot`, targets, `severity`.
4. **Role dispatch** — if `role == Appeal`, hand off to `process_appeal_vote` *before* the idempotency guard (Appealed is terminal for the original path but expected for the appeal path).
5. **Insert vote** — `INSERT jury_vote`.
6. **Flip assignment** — `UPDATE jury_assignment → Submitted, submitted_at = now`.
7. **Log vote** — `governance_log::append "jury_vote_submitted"`.
8. **Idempotency guard** — if case status is terminal (`Decided`/`Closed`/`Appealed`/`EmergencyRemove`/`AdminReview`/the 3 SponsorLiability* states), return early `{vote_recorded: true, case_decided: true, decision: None}` (the vote row persists for audit; no side effects re-fire — handles late votes 4-5 of 5).
9. **Quorum check** — `COUNT(*) jury_vote`; if `< quorum_snapshot`, return `{case_decided: false}`.
10. **Threshold tally** — load all votes, tally by decision; iterate the fixed `ALL_JURY_DECISIONS` order (8 variants); the first decision whose count `>= threshold_count_snapshot` **wins**.
11. **Deadlock branch** — if no winner AND `vote_count == panel_size_snapshot` (all voted) → `UPDATE status = AdminReview`, append `ENTRY_KIND_JURY_DEADLOCK`, return `{case_decided: false}`. (Partial tally — no winner, not all voted — returns the same shape without the status flip.)
12. **Sanction map + insert** — `map_decision_to_sanction(winning)`; `NoAction` → no row; else `INSERT sanction (scope, action, target_*, active=true)`, append `"sanction_created"`.
13. **Sponsor-liability branch** (v1-SL-d) — only if `target_person_id` is Some: `compute_sponsor_liability(...)`; if deltas non-empty, set path = Pending, compute `grace_expires = now + grace_window_for_severity(severity)`.
14. **Case status flip** — Pending path → `status = SponsorLiabilityPending, decided_at = now, winning_decision = Some(…), grace_expires_at = Some(…)`. Decided path → `status = Decided, decided_at = now, winning_decision = Some(…)`.
15. **Post-decision block (Decided path only)** — public log (`scrub`'d summary + rationales → `INSERT public_case_log`, append `"public_log_published"`); juror reputation (aligned/outlier on JuryReliability + participation on aligned jurors); reporter reputation (upheld/dismissed on ReportingAccuracy); evidence-cited bonus (RT-r3, if reporter uploaded evidence + a winning rationale ≥ threshold chars). On the Pending path these steps are deferred — the grace scheduler runs them at fire/escape time.
16. **Appeal-window write (both paths)** — read `appeal.window_days` **(the one deliberate LIVE config read, not a snapshot)**; `UPDATE appeal_window_expires_at = now + days(window_days)`.
17. **Log case decided** — `governance_log::append "case_decided"` (logged **before** any federation broadcast — ADR-008 causality / hash-chain ordering).
18. **Sponsor-liability-pending log (Pending path)** — append `ENTRY_KIND_SPONSOR_LIABILITY_PENDING`.
19. **Federation outbound (Decided path)** — if winning sanction scope == `FederatedRecommendation`, call `federation_outbox::send_local_sanction_notice` (atomic with the tx).
20. **Return** `{vote_recorded: true, case_decided: true, decision: Some(winning)}`.

**Appeal path (`process_appeal_vote`, v1-JM-e):** insert vote / flip assignment / log (same shape) → narrower idempotency guard (only `Closed`/`EmergencyRemove`/`AdminReview` short-circuit) → load Appeal snapshots → tally **Appeal-role votes only** (INNER JOIN `jury_assignment` filter `role = Appeal`) → on verdict: `appeal → Decided`, `moderation_case → Closed` in one step (PRD §6.7), append `ENTRY_KIND_APPEAL_DECIDED`; no new sanction, no federation outbound (those fired at original decision). Appeal deadlock → `AdminReview`.

### Decision → sanction map (exhaustive, ADR-013)

| `JuryDecision` | `(scope, action)` |
|---|---|
| NoAction | *(no sanction row)* |
| AdvisoryLabel | (Community, Label) |
| Warning | (Community, VisibilityReduction) |
| Cooldown | (Community, TemporaryRestriction) |
| RemoveContent | (Community, ContentRemoval) |
| SuspendLocalUser | (Instance, InstanceSuspension) |
| SuspendCommunityMember | (Community, CommunityExclusion) |
| RecommendFederationAction | (FederatedRecommendation, FederationQuarantineRecommendation) |

### Case-status transitions triggered in `submit_jury_vote`

- non-terminal → **Decided** (quorum met + a decision met threshold + not sponsor-liability path)
- non-terminal → **SponsorLiabilityPending** (decision met threshold + active sureties on a Person target; sets `grace_expires_at`)
- non-terminal → **AdminReview** (all jurors voted + no threshold met — deadlock)
- terminal → unchanged (idempotency short-circuit)
- **Appeal path:** Appealed → Closed (appeal verdict met threshold) | Appealed → AdminReview (appeal deadlock)

---

## 9. Federation surface

ActivityPub-based; protocol/object types in `crates/apub/objects/`, activity wrappers + inbound receivers in `crates/apub/activities/`, the public boundary façade in `crates/apub/apub/`.

### AP objects (`crates/apub/objects/src/{protocol/,}governance/`)

| Protocol struct | `type` | Purpose |
|---|---|---|
| `SanctionNoticeProtocol` | `SanctionNotice` | Wire-form sanction notice (`actor`, `target`, `action`, `scope`, `summary` [pre-redacted], `published`). `deny_unknown_fields`. |
| `TrustAttestationProtocol` | `TrustAttestation` | Wire-form trust attestation (`subject`, `attestation_type`, `valid_until`). |
| `ModerationLabelProtocol` | `ModerationLabel` | Wire-form moderation label (`target`, `label`, `summary?`). |

Object newtypes (`Object` trait impls) — `ApubSanctionNotice`, `ApubTrustAttestation`, `ApubModerationLabel` — are inbound-advisory stubs: `read_from_id → Ok(None)`, `into_json/from_json/delete → NotFound`, `verify → verify_domains_match` (ADR-006: not URL-addressable; `local_case_id` always NULL).

### AP activities (`crates/apub/activities/src/{protocol/,}governance/`)

| Activity wrapper | AP kind | Receive → | Outbound builder |
|---|---|---|---|
| `PublishSanctionNotice` | Create | `receive_remote_sanction_notice` | `build_local_sanction_notice_plan` → `SanctionNoticeSendPlan`; `enqueue_sanction_notice_activity` |
| `PublishTrustAttestation` | Create | `receive_remote_trust_attestation` | `build_local_trust_attestation_plan` (plumbed; **no v0 caller**) |
| `PublishLabel` | Create | `receive_remote_moderation_label` | *(none — inbound only)* |

Builders do **DB reads only**; the tx-owning orchestrator (`lemmy_api::governance::federation_outbox`) opens the transaction, enqueues the `sent_activity`, and appends the governance-log entry atomically (builder/orchestrator split, DQ-6.6).

### Inbox / outbox wiring

- **Inbound dispatch registry** — `crates/apub/activities/src/activity_lists.rs`: the `SharedInboxActivities` untagged enum registers the three governance Create wrappers **before** the `RawAnnouncableActivities` catch-all (declaration order is load-bearing for `#[serde(untagged)]` matching). HTTP entry is Lemmy's existing `shared_inbox` (`crates/apub/apub/src/http/mod.rs`), unchanged.
- **Outbound façade** — `crates/apub/apub/src/governance/{mod,outbox,verify}.rs` re-exports the builders so consumers depend only on `lemmy_apub`.
- **Server wiring** — `crates/server/src/governance.rs` registers **no** federation routes; `schedule_governance_jobs` only logs that two scheduled jobs are set up elsewhere (`scheduled_tasks::setup`): reputation snapshot recalc (`run_snapshot_batch`, 15-min; `BREHON_DISABLE_SNAPSHOT_JOB=1`) and appeal-window expiry (`run_appeal_window_expiry_batch`, hourly; `BREHON_DISABLE_APPEAL_WINDOW_JOB=1`).

### Inbound (v1 federation-inbound lane)

All inbound `receive_remote_*` bodies live in `crates/apub/activities/src/governance/inbox.rs` (per DQ-6.6-inbound; `crates/apub/apub/src/governance/mod.rs` only re-exports them — `crates/apub/apub/src/governance/inbox.rs` does **not** exist). **ADR-006 invariant: inbound is advisory-only** — never auto-applied; each successful receive writes exactly two rows (one advisory row with `local_case_id = NULL`, one `governance_log` entry).

The `wrap_governance_inbound` wrapper (v1-federation-inbound-b) applies a six-gate policy before delegating:

| Gate | Check | Drop → status + log kind |
|---|---|---|
| 1 | Peer trust (`federation_inbox_check_peer_trust`) — Blocklisted | 403 · `FEDERATION_INBOUND_BLOCKED` |
| 2 | Per-type payload size cap | 413 · `FEDERATION_INBOUND_DROPPED_OVERSIZE` |
| 3 | Schema strictness (serde `deny_unknown_fields`) | dropped at deserialize |
| 4 | Per-peer hourly rate | 429 · `FEDERATION_INBOUND_DROPPED_RATE_LIMIT_PEER` |
| 5 | Per-actor rate (trust attestations only) | 429 · `FEDERATION_INBOUND_DROPPED_RATE_LIMIT_ACTOR` |
| 6 | Replay nonce (`INSERT federation_inbox_nonce`; unique violation) | 409 · `FEDERATION_INBOUND_DROPPED_REPLAY` |

Supporting infra (all in `inbox.rs`): `log_inbox_drop` (writes `federation_inbox_dropped_log` + governance-log atomically), storage-cap eviction (`pg_advisory_xact_lock` keyed on `peer:table` + `evict_oldest_unreviewed_if_needed_in_tx` → `FEDERATION_INBOUND_DROPPED_STORAGE_CAP_EVICTED`), best-effort `FEDERATION_INBOUND_PERSIST_FAILED` on rollback, in-memory rate-limit state, and a local `get_inbound_config_int` reader (avoids the `lemmy_api → lemmy_apub → lemmy_apub_activities` dep cycle). The admin peer-trust flip handler (`FEDERATION_PEER_TRUST_CHANGED`) is v1-federation-inbound-c (pending).

---

## 10. Governance-log hash chain + signing

`governance_log` is an append-only hash chain (ADR-008). On INSERT, a `BEFORE INSERT` trigger (`governance_log_hash_chain_*`) populates `prev_hash` + `entry_hash` (`sha2`). The single-shot signature UPDATE (Phase 4b) populates `signature` (ed25519, `ed25519-dalek`), gated by a `BEFORE UPDATE` trigger that forbids any other signature transition or column change. The `AFTER UPDATE OF signature` notify trigger fires `pg_notify('governance_events', …)` on the NULL→NOT NULL transition (migration #11). Merkle root (`rs_merkle`) batching is for the audit-stream/checkpoint surface. The signing key lives in env `GOVERNANCE_LOG_SIGNING_KEY` for v0 (per CLAUDE.md hard constraints).

---

## 11. Entry-kind registry

`governance_log.entry_kind` is plain `TEXT`; the **55 valid kinds** are `pub const ENTRY_KIND_*` in `crates/db_schema/src/source/governance/governance_log.rs`. The authoritative human-readable registry (kind → payload schema → which sub-phase emits it) is **`.claude/rules/governance-log-entry-kind-registry.md`** — do not duplicate it here; that file owns the list. (The separate `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` is the **harness-observability** JSONL sidecar — the advisor/retro-bypass trail — NOT a product governance-log surface; don't conflate the two.)

Count by era (for traceability): 19 v0 + 4 Phase-6 federation outbound + 2 v1-AD-a config + 1 v1-AD-c rule-set + 7 v1-JM (incl. `JURY_DEADLOCK`) + 5 v1-SL-a + 7 v1-RT-r1 + 10 v1-federation-inbound (9 a + 1 b) = 55.

---

## 12. Scheduled jobs

| Job | Fn | Cadence | Disable env |
|---|---|---|---|
| Reputation snapshot recalc | `run_snapshot_batch` | 15-min tick (`job.snapshot_interval_seconds = 900`) | `BREHON_DISABLE_SNAPSHOT_JOB=1` |
| Appeal-window expiry | `run_appeal_window_expiry_batch` | hourly | `BREHON_DISABLE_APPEAL_WINDOW_JOB=1` |
| Sponsor-liability grace check | `sponsor_liability_grace` (SL-c) | `job.grace_check_interval_minutes = 5` | — |
| Participation / dormancy cron | `participation_cron` | `job.participation_interval_days = 7` | — |
| Replay-nonce cleanup | `federation_inbox_nonce::delete_older_than` | wired by v1-federation-inbound-b | — |

Registered in `scheduled_tasks::setup`; `crates/server/src/governance.rs::schedule_governance_jobs` is a declarative log-only stub.

---

## 13. Cross-references

- **Concepts + lifecycles:** [02-domain-model.md](02-domain-model.md)
- **Architecture + crate layout:** [03-architecture.md](03-architecture.md)
- **MVP scope + the v0 11-endpoint baseline:** [05-mvp-and-delivery-plan.md](05-mvp-and-delivery-plan.md)
- **ADRs + open questions:** [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md)
- **Security + threat model:** [06-security-and-threat-model.md](06-security-and-threat-model.md)
- **Entry-kind registry (authoritative):** `.claude/rules/governance-log-entry-kind-registry.md`
- **Harness-observability JSONL sidecar (NOT product):** [governance-log-kinds-jsonl.md](governance-log-kinds-jsonl.md)
- **v1 PRDs (design intent — secondary to code):** `.claude/PRPs/prds/v1-*.prd.md`

---

## 14. API behaviour notes (Docker smoke-test reconciliation — CODE WINS)

End-to-end Docker smoke testing surfaced six API behaviours that differ from
a naive reading of the DTOs/routes above. They are **intended** behaviour and
recorded here so callers (and future doc readers) aren't surprised. No code
change accompanies this reconciliation pass.

- **DIFF-1 — governance enums serialize lowercase over JSON.** The Postgres
  enum literal is PascalCase (`'Post'`, `'Warning'`) per the `verbatim` DB
  convention (§2), but the serde/wire representation is snake_case/lowercase
  (`"post"`, `"warning"`). **Both are correct at their own layer:** SQL
  (including migrations) must use the PascalCase DB token; API request/response
  bodies use the lowercase serde token. Conflating them silently matches
  nothing (this is exactly why migration #31's status filter uses `'Open'`,
  not `'open'`).
- **DIFF-2 — `GET /governance/modlog` returns summarised `public_case_log`
  rows, not raw `governance_log`.** The public modlog is the redacted
  case-summary surface (ADR-015 scrubbed). Hash-chain / pseudonym audit of the
  raw append-only log is a DB-level or future admin-only endpoint concern, not
  this route.
- **DIFF-3 — double-endorse returns `not_found`, not a true idempotency
  conflict.** A repeat endorsement within the 48h cooldown
  (`liability.revoke_rate_limit_per_day`) surfaces as `not_found` rather than a
  semantic `conflict`/`already_endorsed`. **Known wart (opaque error).**
  *Future:* return `conflict`/`already_endorsed`. No code change this pass.
- **DIFF-4 — `modlog` returns a bare JSON array on success, a dict on
  rate-limit.** Callers must handle both shapes (array = results; object =
  rate-limit envelope).
- **DIFF-5 — `POST /admin/config` requires a non-empty `reason`; `value` is a
  raw JSON scalar.** Not `value_int`/`value_text` split fields — a single
  `value` carrying the JSON scalar, plus a mandatory `reason` string.
- **DIFF-6 — some admin GETs require their scoping id.** `GET /governance/admin/rule-sets`
  requires `community_id` (`AdminListRuleSetsRequest.community_id`); `GET /governance/admin/reputation/rollup`
  requires `person_id` (`AdminReputationRollup.person_id`). Omitting them is a client error,
  not an unscoped "list all". **Do not confuse `/reputation/rollup` (per-person, `person_id`
  required) with the sibling `GET /governance/admin/reputation-stats`, whose `community_id`
  is *optional*** (absent → instance-wide stats). Both routes exist in code (`routes/src/lib.rs:503-504`);
  the §7 table above now lists both.
