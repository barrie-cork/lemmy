# 04 — Data Model & API

**Audience:** Backend engineers (daily-driver reference)
**Status:** Stable starting point; exact field types may change during implementation
**Sources:** [chat1.md](chat1.md) §1–§15 in full (verbatim where load-bearing)

This is the implementation reference. It holds migrations, tables, enums, Diesel structs, view structs, request/response DTOs, the REST route table, and handler responsibilities. Concepts and lifecycles live in [02-domain-model.md](02-domain-model.md). Architecture and crate layout live in [03-architecture.md](03-architecture.md).

---

## 1. Migrations (dependency order)

Four migrations, each with up.sql and down.sql. Reuse existing Lemmy integer ID style and add foreign keys to `person`, `community`, `post`, `comment`, and existing moderation targets.

### Migration 1 — `add_governance_core`

Tables:
- `moderation_case`
- `case_evidence`
- `sanction`
- `appeal`
- `public_case_log`

### Migration 2 — `add_jury_system`

Tables:
- `jury_pool`
- `jury_assignment`
- `jury_vote`

### Migration 3 — `add_reputation_and_surety`

Tables:
- `surety`
- `endorsement`
- `reputation_event`
- `reputation_snapshot`

### Migration 4 — `add_federation_attestations`

Tables:
- `federation_attestation`
- `remote_sanction_notice`

### Indexes to create early

- `moderation_case(status, created_at)`
- `jury_assignment(person_id, status)`
- `reputation_event(person_id, community_id, created_at)`
- `public_case_log(community_id, published_at)`

## 2. Enums (Diesel-backed)

Use Diesel-backed enums for parity with Lemmy's enum-heavy schema style (`diesel-derive-enum` is already in the workspace).

### CaseStatus
```rust
pub enum CaseStatus {
    Open,
    ThresholdMet,
    JurySelection,
    InReview,
    Decided,
    Appealed,
    Closed,
    EmergencyRemove,  // Per ADR-013: admin emergency-remove path for illegal content;
                      // jury assigned post-facto to review the override.
}
```

### CaseTargetType
```rust
pub enum CaseTargetType {
    Post,
    Comment,
    Person,
    Community,
    RemoteInstance,
}
```

### CaseSeverity
```rust
pub enum CaseSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

### EvidenceVisibility
```rust
pub enum EvidenceVisibility {
    JuryOnly,
    PrivateAdmin,
    PublicRedacted,
}
```

### JuryAssignmentStatus
```rust
pub enum JuryAssignmentStatus {
    Selected,
    Accepted,
    Declined,
    Conflicted,
    Submitted,
    Expired,
}
```

### JuryDecision
```rust
pub enum JuryDecision {
    NoAction,
    AdvisoryLabel,
    Warning,
    Cooldown,
    RemoveContent,
    SuspendLocalUser,
    SuspendCommunityMember,
    RecommendFederationAction,
}
```

### SanctionScope
```rust
pub enum SanctionScope {
    Community,
    Instance,
    FederatedRecommendation,
}
```

### SanctionAction
```rust
pub enum SanctionAction {
    Label,
    VisibilityReduction,
    TemporaryRestriction,
    ContentRemoval,
    CommunityExclusion,
    InstanceSuspension,
    FederationQuarantineRecommendation,
}
```

### AppealStatus
```rust
pub enum AppealStatus {
    Requested,
    Accepted,
    Rejected,
    Decided,
}
```

### ReputationDimension
```rust
pub enum ReputationDimension {
    ReportingAccuracy,
    JuryReliability,
    ParticipationConsistency,
    EndorsementStrength,
}
```

### AttestationType
```rust
pub enum AttestationType {
    TrustedReporter,
    JuryEligible,
    SanctionNotice,
    QuarantineRecommendation,
}
```

## 3. Diesel models (source tables)

Add under `crates/db_schema/src/source/`. Also update `schema.rs`, `newtypes.rs` (for typed IDs if desired), `lib.rs`, and enum exports.

### ModerationCase
```rust
#[derive(Queryable, Selectable, Identifiable, Associations, Debug, Clone)]
#[diesel(table_name = moderation_case)]
pub struct ModerationCase {
    pub id: i32,
    pub community_id: Option<i32>,
    pub creator_id: Option<i32>,
    pub target_type: CaseTargetType,
    pub target_post_id: Option<i32>,
    pub target_comment_id: Option<i32>,
    pub target_person_id: Option<i32>,
    pub target_community_id: Option<i32>,
    pub target_remote_url: Option<String>,
    pub reason_code: String,
    pub severity: CaseSeverity,
    pub status: CaseStatus,
    pub threshold_score: i64,
    pub opened_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = moderation_case)]
pub struct ModerationCaseInsertForm {
    pub community_id: Option<i32>,
    pub creator_id: Option<i32>,
    pub target_type: CaseTargetType,
    pub target_post_id: Option<i32>,
    pub target_comment_id: Option<i32>,
    pub target_person_id: Option<i32>,
    pub target_community_id: Option<i32>,
    pub target_remote_url: Option<String>,
    pub reason_code: String,
    pub severity: CaseSeverity,
    pub status: CaseStatus,
    pub threshold_score: i64,
}
```

### CaseEvidence
```rust
pub struct CaseEvidence {
    pub id: i32,
    pub case_id: i32,
    pub uploader_id: i32,
    pub storage_key: String,
    pub sha256: String,
    pub mime_type: String,
    pub visibility: EvidenceVisibility,
    pub created_at: DateTime<Utc>,
}
```

### JuryAssignment
```rust
pub struct JuryAssignment {
    pub id: i32,
    pub case_id: i32,
    pub person_id: i32,
    pub status: JuryAssignmentStatus,
    pub selected_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub submitted_at: Option<DateTime<Utc>>,
}
```

### JuryVote
```rust
pub struct JuryVote {
    pub id: i32,
    pub case_id: i32,
    pub juror_id: i32,
    pub decision: JuryDecision,
    pub rationale: Option<String>,
    pub submitted_at: DateTime<Utc>,
}
```

### Sanction
```rust
pub struct Sanction {
    pub id: i32,
    pub case_id: i32,
    pub scope: SanctionScope,
    pub action: SanctionAction,
    pub target_person_id: Option<i32>,
    pub target_post_id: Option<i32>,
    pub target_comment_id: Option<i32>,
    pub target_community_id: Option<i32>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub active: bool,
}
```

### Appeal
```rust
pub struct Appeal {
    pub id: i32,
    pub case_id: i32,
    pub requester_id: i32,
    pub reason: String,
    pub status: AppealStatus,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
}
```

### Surety
```rust
pub struct Surety {
    pub id: i32,
    pub sponsor_id: i32,
    pub sponsored_id: i32,
    pub community_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}
```

### Endorsement
```rust
pub struct Endorsement {
    pub id: i32,
    pub from_person_id: i32,
    pub to_person_id: i32,
    pub community_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}
```

### ReputationEvent
```rust
pub struct ReputationEvent {
    pub id: i32,
    pub person_id: i32,
    pub community_id: Option<i32>,
    pub dimension: ReputationDimension,
    pub delta: i32,
    pub source_case_id: Option<i32>,
    pub source_report_id: Option<i32>,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
```

### ReputationSnapshot
```rust
pub struct ReputationSnapshot {
    pub id: i32,
    pub person_id: i32,
    pub community_id: Option<i32>,
    pub reporting_accuracy: i32,
    pub jury_reliability: i32,
    pub participation_consistency: i32,
    pub endorsement_strength: i32,
    pub jury_eligible: bool,
    pub trusted_reporter: bool,
    pub calculated_at: DateTime<Utc>,
}
```

### PublicCaseLog
```rust
pub struct PublicCaseLog {
    pub id: i32,
    pub case_id: i32,
    pub community_id: Option<i32>,
    pub summary: String,
    pub rationale_redacted: Option<String>,
    pub published_at: DateTime<Utc>,
}
```

### FederationAttestation

Signed federation-level claim about an actor or content. Minimum fields:

- `id: i32`
- `actor_url: String`
- `subject_url: String`
- `attestation_type: AttestationType`
- `valid_until: Option<DateTime<Utc>>`
- `created_at: DateTime<Utc>`
- `signature: String`

### RemoteSanctionNotice

Inbound sanction signal from a federated instance. Stored as advisory evidence only (MVP never auto-applies). Minimum fields:

- `id: i32`
- `source_instance: String`
- `target_url: String`
- `action: SanctionAction`
- `scope: SanctionScope`
- `summary: String`
- `published_at: DateTime<Utc>`
- `signature: String`
- `local_case_id: Option<i32>` — linked if a local case exists or is created

### ActorPseudonym

Per [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md) ADR-015 and [06-security-and-threat-model.md §6.1](06-security-and-threat-model.md). Maps a real `person_id` to an opaque pseudonymous actor ID that the governance log uses in place of any direct personal identifier. On GDPR right-to-delete, the mapping row for a user is deleted; the governance log itself remains hash-chain-verifiable, but the actor behind those log entries becomes permanently unrecoverable.

Minimum fields:

- `id: i32`
- `person_id: i32` — FK to `person`
- `pseudonym: String` — opaque, high-entropy (UUID or random 128-bit), unique, never reused
- `created_at: DateTime<Utc>`

Rules:

- The governance log **must** reference `pseudonym`, never `person_id` or username
- Deletion of a row in this table is a GDPR response action — log it to a separate admin audit trail (not the governance log, since the point is to anonymise in the governance log)
- Pseudonym generation must be cryptographically random; derivation from `person_id` defeats the property
- A deleted pseudonym is **not replaced** — subsequent actions by the same user generate a new pseudonym so the new activity isn't linked to the old

**Redaction service contract** (enforced at code level): before any string is appended to `public_case_log.summary`, `public_case_log.rationale_redacted`, or any governance log entry, the redaction service scrubs direct identifiers (usernames, emails, display names, addresses, phone numbers, external URLs that identify a person). This is a hard prerequisite for ADR-015 — once identifiers leak into the log, the right-to-delete strategy fails.

## 4. Read models (db_views crates)

### 4.1 `crates/db_views/governance_case`

```rust
pub struct GovernanceCaseSummaryView {
    pub case_id: i32,
    pub status: CaseStatus,
    pub severity: CaseSeverity,
    pub reason_code: String,
    pub opened_at: DateTime<Utc>,
    pub community_id: Option<i32>,
    pub community_name: Option<String>,
    pub target_type: CaseTargetType,
    pub reporter_count: i64,
    pub jury_needed: i32,
    pub jury_submitted: i32,
}

pub struct GovernanceCaseDetailView {
    pub case_row: ModerationCase,
    pub sanctions: Vec<Sanction>,
    pub evidence_count: i64,
    pub appeal_status: Option<AppealStatus>,
    pub target_creator_id: Option<i32>,
}
```

Queries:
- `list_open_cases_for_community`
- `read_case_detail`
- `list_cases_for_person`
- `list_cases_needing_jury_selection`

### 4.2 `crates/db_views/jury_queue`

```rust
pub struct JuryQueueView {
    pub case_id: i32,
    pub severity: CaseSeverity,
    pub reason_code: String,
    pub opened_at: DateTime<Utc>,
    pub deadline_at: Option<DateTime<Utc>>,
    pub community_id: Option<i32>,
    pub community_name: Option<String>,
}
```

Queries:
- `list_jury_assignments_for_person`
- `list_available_jury_cases_for_person`
- `count_unsubmitted_jury_assignments`

### 4.3 `crates/db_views/reputation`

```rust
pub struct ReputationSummaryView {
    pub person_id: i32,
    pub community_id: Option<i32>,
    pub reporting_accuracy: i32,
    pub jury_reliability: i32,
    pub participation_consistency: i32,
    pub endorsement_strength: i32,
    pub jury_eligible: bool,
    pub trusted_reporter: bool,
    pub active_sanctions: i64,
}

pub struct EndorsementSummaryView {
    pub person_id: i32,
    pub inbound_endorsements: i64,
    pub outbound_endorsements: i64,
    pub active_sureties: i64,
}
```

Queries:
- `read_reputation_summary`
- `list_endorsements_for_person`
- `list_sureties_for_person`

### 4.4 `crates/db_views/governance_modlog`

```rust
pub struct GovernanceModlogView {
    pub case_id: i32,
    pub community_id: Option<i32>,
    pub community_name: Option<String>,
    pub decision: Option<JuryDecision>,
    pub sanction_action: Option<SanctionAction>,
    pub summary: String,
    pub published_at: DateTime<Utc>,
    pub appealed: bool,
}
```

Queries:
- `list_public_case_log`
- `list_public_case_log_for_community`
- `read_public_case_log_entry`

## 5. API request/response types (`crates/api/api_common/src/governance.rs`)

Update `src/lib.rs` to export the new module.

### Reports and cases
```rust
pub struct CreateGovernanceReport {
    pub community_id: Option<i32>,
    pub target_type: CaseTargetType,
    pub target_id: i32,
    pub reason_code: String,
    pub description: Option<String>,
}

pub struct CreateGovernanceReportResponse {
    pub case_id: Option<i32>,
    pub threshold_met: bool,
}

pub struct GetGovernanceCase {
    pub case_id: i32,
}

pub struct ListGovernanceCases {
    pub community_id: Option<i32>,
    pub status: Option<CaseStatus>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}
```

### Jury
```rust
pub struct SubmitJuryVote {
    pub case_id: i32,
    pub decision: JuryDecision,
    pub rationale: Option<String>,
}

pub struct AcceptJuryAssignment {
    pub case_id: i32,
}

pub struct DeclineJuryAssignment {
    pub case_id: i32,
    pub reason: Option<String>,
}
```

### Appeals
```rust
pub struct RequestAppeal {
    pub case_id: i32,
    pub reason: String,
}
```

### Reputation / trust
```rust
pub struct GetMyReputation {
    pub community_id: Option<i32>,
}

pub struct CreateEndorsement {
    pub person_id: i32,
    pub community_id: Option<i32>,
}

pub struct RevokeEndorsement {
    pub endorsement_id: i32,
}
```

### Public log
```rust
pub struct ListGovernanceModlog {
    pub community_id: Option<i32>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}
```

## 6. Handlers

### 6.1 `crates/api/api_crud` — simple CRUD

Files:
- `src/governance/create_report.rs`
- `src/governance/create_endorsement.rs`
- `src/governance/revoke_endorsement.rs`
- `src/governance/request_appeal.rs`

#### `create_report`
- Validate target exists
- Write report row or directly create/upsert case row
- Compute initial threshold contribution (report weight × reporter reputation × recency factor)
- Return `case_id` if threshold opens a case immediately

#### `create_endorsement`
- Validate actor can endorse (capability: `can_sponsor`)
- Enforce endorsement limits (max 5 active, 48h cooldown)
- Insert endorsement row
- Enqueue reputation recalculation for both parties

#### `revoke_endorsement`
- Authorise revocation (must be the endorser)
- Soft-revoke: set `revoked_at`
- Enqueue reputation recalculation

#### `request_appeal`
- Verify appeal window is still open for the case
- Verify requester is entitled to appeal (target of sanction, or original reporter in limited cases)
- Insert appeal row with status `Requested`
- Move case status to `Appealed`

### 6.2 `crates/api/api` — workflow/orchestration

Files:
- `src/governance/get_case.rs`
- `src/governance/list_cases.rs`
- `src/governance/list_modlog.rs`
- `src/governance/list_my_jury_queue.rs`
- `src/governance/accept_jury_assignment.rs`
- `src/governance/decline_jury_assignment.rs`
- `src/governance/submit_jury_vote.rs`
- `src/governance/get_my_reputation.rs`
- `src/governance/admin_assign_jury.rs`
- `src/governance/admin_close_case.rs`

#### `get_case`
- Permission-aware read
- Show public view, juror-private view, or admin view depending on caller role

#### `list_cases`
- Filter by community / status / assignee
- Pagination

#### `list_modlog`
- Public, redacted moderation log
- Pagination, per-community filter

#### `list_my_jury_queue`
- List active jury assignments for caller
- Sort by deadline

#### `accept_jury_assignment`
- Check conflict rules (not same sponsor cluster, not connected to target)
- Change `jury_assignment.status` to `Accepted`

#### `decline_jury_assignment`
- Change status to `Declined`
- Log reason if provided
- Trigger replacement juror selection

#### `submit_jury_vote`
- Verify assignment is active (`Accepted`)
- Write `jury_vote`
- If quorum reached:
  1. Aggregate decision (see §8 below)
  2. Create `sanction` row(s)
  3. Mark case `Decided`
  4. Insert `public_case_log` entry
  5. Emit reputation events for jurors and reporters
  6. Open appeal window

#### `get_my_reputation`
- Return `ReputationSummaryView` for the caller, scoped to community if provided

#### `admin_assign_jury`
- **Temporary admin-only backstop for MVP**
- Builds jury pool and assignments manually for testing

#### `admin_close_case`
- **Emergency/manual path while jury system matures**
- Requires admin + audit log entry

## 7. Routes (`crates/api/routes/src/governance.rs`)

All routes live under `/api/v4/governance/`.

### CRUD-style
```
POST /api/v4/governance/report
POST /api/v4/governance/endorsement
POST /api/v4/governance/endorsement/revoke
POST /api/v4/governance/appeal
```

### Read / workflow
```
GET  /api/v4/governance/case
GET  /api/v4/governance/cases
GET  /api/v4/governance/modlog
GET  /api/v4/governance/reputation/me
GET  /api/v4/governance/jury/me
POST /api/v4/governance/jury/accept
POST /api/v4/governance/jury/decline
POST /api/v4/governance/jury/vote
```

### Temporary admin backstops
```
POST /api/v4/governance/admin/assign-jury
POST /api/v4/governance/admin/close-case
```

## 8. Aggregation rules (v0, deliberately simple)

When `submit_jury_vote` is called, apply this logic:

```
if submitted_votes < quorum:
    persist vote
    return 200 OK, no further action

if quorum reached:
    tally decisions
    majority decision wins (for non-severe sanctions)
    supermajority required for suspension (reserved for v1)
    create one Sanction row
    mark case Decided
    insert PublicCaseLog row
    emit reputation events for:
        - jurors (aligned with final decision → +rep; outlier → −rep)
        - reporters (contributed to a sanctioned case → +rep; dismissed case → −rep)
    open appeal window (record case.closed_at deadline)
```

**MVP parameters (exactly these values for v0):**
- Jury size: **5**
- Quorum: **3**
- Winning rule: **simple majority** for non-severe sanctions
- Supermajority: reserved for v1

**v1 parameters** (target, documented in [01-vision-and-principles.md](01-vision-and-principles.md) §5.6):
- Jury size: 7
- Voting thresholds by severity: majority / 60% / 75%

## 9. Federation objects (`crates/apub/objects/src/governance/`)

### ModerationLabelObject
```rust
pub struct ModerationLabelObject {
    pub id: Url,
    pub actor: Url,
    pub target: Url,
    pub label: String,
    pub summary: Option<String>,
    pub published: DateTime<Utc>,
}
```

### TrustAttestationObject
```rust
pub struct TrustAttestationObject {
    pub id: Url,
    pub actor: Url,
    pub subject: Url,
    pub attestation_type: AttestationType,
    pub valid_until: Option<DateTime<Utc>>,
    pub published: DateTime<Utc>,
}
```

### SanctionNoticeObject
```rust
pub struct SanctionNoticeObject {
    pub id: Url,
    pub actor: Url,
    pub target: Url,
    pub action: SanctionAction,
    pub scope: SanctionScope,
    pub summary: String,
    pub published: DateTime<Utc>,
}
```

## 10. Federation activities (`crates/apub/activities/src/governance/`)

- `publish_label.rs` — `Create` activity for moderation labels
- `publish_attestation.rs` — `Create` activity for trust attestations
- `publish_sanction_notice.rs` — `Create` activity for sanction notices
- `Undo` variants for revoked attestations or rescinded notices

For MVP, federation is **outbound-only** unless we want remote notices to trigger local workflows immediately (we do not).

## 11. Federation inbox/outbox (`crates/apub/apub/src/governance/`)

- `inbox.rs`
- `outbox.rs`
- `verify.rs`

Functions to add first:
- `receive_remote_sanction_notice` — accept, verify signature, store in `remote_sanction_notice`, **do not auto-apply**
- `receive_remote_trust_attestation` — accept, verify, store
- `send_local_sanction_notice` — outbound publish
- `send_local_trust_attestation` — outbound publish

For first release:
- Accept remote notices
- Store them
- **Never auto-apply sanctions from remote nodes**
- Surface them as advisory evidence only

This keeps federation safer and simpler. See [06-security-and-threat-model.md](06-security-and-threat-model.md) §5.

## 12. Server wiring (`crates/server/src/governance.rs`)

`crates/server` is **module registration and startup wiring only**. It does not host business logic.

Add:
- Governance route registration
- Periodic reputation snapshot job (background)
- Periodic expired sanction cleanup job (background)
- Periodic jury assignment timeout job (background)

## 13. Practical shortcuts and warnings

### Shortcut: collapse reports into cases directly
For MVP, do **not** model separate `report` and `case` tables unless you need full backward compatibility with Lemmy's reports. Every report creates or appends to a `moderation_case`. Add a child `case_report` table only if you need per-reporter history.

### Warning: do not overbuild reputation in v1
Only implement these **two booleans** and **four counters** in v0:

Booleans:
- `jury_eligible`
- `trusted_reporter`

Counters:
- `reporting_accuracy`
- `jury_reliability`
- `participation_consistency`
- `endorsement_strength`

Everything else is derived later. Decay, capability thresholds, and per-community roll-ups can be tuned as data arrives.

### Warning: do not embed governance in `crates/server`
`server` is composition root only. See [03-architecture.md](03-architecture.md) §11.

## 14. Cross-references

- Concepts and lifecycles → [02-domain-model.md](02-domain-model.md)
- Architecture, crate layout, governance log contract → [03-architecture.md](03-architecture.md)
- What to ship first → [05-mvp-and-delivery-plan.md](05-mvp-and-delivery-plan.md)
- Security controls on these endpoints → [06-security-and-threat-model.md](06-security-and-threat-model.md)
