crate-by-crate implementation checklist next, with the exact structs, enums, Diesel models, and API endpoints to add first.

I have the workspace shape pinned down. Now I’m turning that into an engineer-facing checklist with the first-pass structs, Diesel tables, and endpoints so it’s concrete enough to start coding.

Absolutely. Here’s a crate-by-crate implementation checklist for a Lemmy fork, using Lemmy’s current workspace shape as the scaffold: crates/db_schema for Diesel-backed schema, many focused crates/db_views/* crates for read models, crates/api/* for request/response types and handlers, crates/apub/* for federation, plus crates/routes and crates/server for assembly.

I’m going to optimize for a first shippable governance layer:

reports become cases
cases can escalate to juries
juries issue decisions
decisions create sanctions + public logs
sponsorship and reputation are added early, but can be soft-gated at first
0. First-pass crate plan

Add these crates first:

crates/db_views/governance_case
crates/db_views/governance_modlog
crates/db_views/reputation
crates/db_views/jury_queue

Then add new modules inside existing crates:

crates/db_schema/src/source/governance_*
crates/api/api_common/src/governance.rs
crates/api/api/src/governance/*
crates/api/api_crud/src/governance/*
crates/api/routes/src/governance.rs
crates/apub/objects/src/governance/*
crates/apub/activities/src/governance/*
crates/apub/apub/src/governance/*

Keep crates/server for wiring only, since Lemmy already uses it as the composition root rather than the business-logic home.

1. migrations/

This comes first.

Create migration 1: add_governance_core

Add tables:

moderation_case
case_evidence
sanction
appeal
public_case_log
Create migration 2: add_jury_system

Add tables:

jury_pool
jury_assignment
jury_vote
Create migration 3: add_reputation_and_surety

Add tables:

surety
endorsement
reputation_event
reputation_snapshot
Create migration 4: add_federation_attestations

Add tables:

federation_attestation
remote_sanction_notice
Checklist
Add up.sql and down.sql for each.
Reuse existing Lemmy integer ID style where possible.
Add foreign keys to person, community, post, comment, and existing moderation targets.
Add indexes early for:
moderation_case(status, created_at)
jury_assignment(person_id, status)
reputation_event(person_id, community_id, created_at)
public_case_log(community_id, published_at)
2. crates/db_schema

Lemmy already centralizes database schema and Diesel integration here.

Files to add

Under crates/db_schema/src/source/:

moderation_case.rs
case_evidence.rs
sanction.rs
appeal.rs
jury_pool.rs
jury_assignment.rs
jury_vote.rs
surety.rs
endorsement.rs
reputation_event.rs
reputation_snapshot.rs
public_case_log.rs
federation_attestation.rs
remote_sanction_notice.rs

Also update:

schema.rs
newtypes.rs if you want typed IDs
lib.rs
any enum exports module
Enums to add first

Use Diesel-backed enums if you want parity with Lemmy’s enum-heavy schema style, since the workspace already depends on diesel-derive-enum.

CaseStatus
pub enum CaseStatus {
    Open,
    ThresholdMet,
    JurySelection,
    InReview,
    Decided,
    Appealed,
    Closed,
}
CaseTargetType
pub enum CaseTargetType {
    Post,
    Comment,
    Person,
    Community,
    RemoteInstance,
}
CaseSeverity
pub enum CaseSeverity {
    Low,
    Medium,
    High,
    Critical,
}
EvidenceVisibility
pub enum EvidenceVisibility {
    JuryOnly,
    PrivateAdmin,
    PublicRedacted,
}
JuryAssignmentStatus
pub enum JuryAssignmentStatus {
    Selected,
    Accepted,
    Declined,
    Conflicted,
    Submitted,
    Expired,
}
JuryDecision
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
SanctionScope
pub enum SanctionScope {
    Community,
    Instance,
    FederatedRecommendation,
}
SanctionAction
pub enum SanctionAction {
    Label,
    VisibilityReduction,
    TemporaryRestriction,
    ContentRemoval,
    CommunityExclusion,
    InstanceSuspension,
    FederationQuarantineRecommendation,
}
AppealStatus
pub enum AppealStatus {
    Requested,
    Accepted,
    Rejected,
    Decided,
}
ReputationDimension
pub enum ReputationDimension {
    ReportingAccuracy,
    JuryReliability,
    ParticipationConsistency,
    EndorsementStrength,
}
AttestationType
pub enum AttestationType {
    TrustedReporter,
    JuryEligible,
    SanctionNotice,
    QuarantineRecommendation,
}
Diesel models to add first
ModerationCase
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
InsertForm
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
CaseEvidence
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
JuryAssignment
pub struct JuryAssignment {
    pub id: i32,
    pub case_id: i32,
    pub person_id: i32,
    pub status: JuryAssignmentStatus,
    pub selected_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub submitted_at: Option<DateTime<Utc>>,
}
JuryVote
pub struct JuryVote {
    pub id: i32,
    pub case_id: i32,
    pub juror_id: i32,
    pub decision: JuryDecision,
    pub rationale: Option<String>,
    pub submitted_at: DateTime<Utc>,
}
Sanction
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
Appeal
pub struct Appeal {
    pub id: i32,
    pub case_id: i32,
    pub requester_id: i32,
    pub reason: String,
    pub status: AppealStatus,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
}
Surety
pub struct Surety {
    pub id: i32,
    pub sponsor_id: i32,
    pub sponsored_id: i32,
    pub community_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}
Endorsement
pub struct Endorsement {
    pub id: i32,
    pub from_person_id: i32,
    pub to_person_id: i32,
    pub community_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}
ReputationEvent
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
ReputationSnapshot
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
PublicCaseLog
pub struct PublicCaseLog {
    pub id: i32,
    pub case_id: i32,
    pub community_id: Option<i32>,
    pub summary: String,
    pub rationale_redacted: Option<String>,
    pub published_at: DateTime<Utc>,
}
3. crates/db_views/governance_case

Lemmy already uses dedicated db_views/* crates for denormalized read models, including modlog and report_combined, so this is the right place for UI-facing governance queries.

Add first view structs
GovernanceCaseSummaryView
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
GovernanceCaseDetailView
pub struct GovernanceCaseDetailView {
    pub case_row: ModerationCase,
    pub sanctions: Vec<Sanction>,
    pub evidence_count: i64,
    pub appeal_status: Option<AppealStatus>,
    pub target_creator_id: Option<i32>,
}
Queries to add first
list_open_cases_for_community
read_case_detail
list_cases_for_person
list_cases_needing_jury_selection
4. crates/db_views/jury_queue
Add first view struct
pub struct JuryQueueView {
    pub case_id: i32,
    pub severity: CaseSeverity,
    pub reason_code: String,
    pub opened_at: DateTime<Utc>,
    pub deadline_at: Option<DateTime<Utc>>,
    pub community_id: Option<i32>,
    pub community_name: Option<String>,
}
Queries
list_jury_assignments_for_person
list_available_jury_cases_for_person
count_unsubmitted_jury_assignments
5. crates/db_views/reputation
Add first view structs
ReputationSummaryView
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
EndorsementSummaryView
pub struct EndorsementSummaryView {
    pub person_id: i32,
    pub inbound_endorsements: i64,
    pub outbound_endorsements: i64,
    pub active_sureties: i64,
}
Queries
read_reputation_summary
list_endorsements_for_person
list_sureties_for_person
6. crates/db_views/governance_modlog
Add first view struct
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
Queries
list_public_case_log
list_public_case_log_for_community
read_public_case_log_entry
7. crates/api/api_common

Lemmy already has api_common in the workspace specifically for shared API types.

File to add
src/governance.rs

Update:

src/lib.rs
Request/response types to add first
Reports and cases
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
Jury
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
Appeals
pub struct RequestAppeal {
    pub case_id: i32,
    pub reason: String,
}
Reputation / trust
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
Public log
pub struct ListGovernanceModlog {
    pub community_id: Option<i32>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}
8. crates/api/api_crud

Use this for straight create/revoke operations.

Files to add
src/governance/create_report.rs
src/governance/create_endorsement.rs
src/governance/revoke_endorsement.rs
src/governance/request_appeal.rs

Update:

src/lib.rs
module exports
Handlers to add first
create_report

Responsibilities:

validate target exists
write report row or directly create case row
compute initial threshold contribution
return case_id if threshold opens a case immediately
create_endorsement

Responsibilities:

validate actor can endorse
enforce limits
insert endorsement
enqueue reputation recalculation
revoke_endorsement

Responsibilities:

authorize revocation
soft-revoke row
enqueue recalculation
request_appeal

Responsibilities:

verify appeal window
insert appeal
move case status to Appealed
9. crates/api/api

Use this for workflow and orchestration.

Files to add
src/governance/get_case.rs
src/governance/list_cases.rs
src/governance/list_modlog.rs
src/governance/list_my_jury_queue.rs
src/governance/accept_jury_assignment.rs
src/governance/decline_jury_assignment.rs
src/governance/submit_jury_vote.rs
src/governance/get_my_reputation.rs
src/governance/admin_assign_jury.rs
src/governance/admin_close_case.rs

Update:

src/lib.rs
Handlers to add first
get_case
permission-aware read
show public or juror-private detail depending on caller
list_cases
filter by community / status / assignee
list_modlog
public, redacted moderation log
list_my_jury_queue
list active jury assignments for caller
accept_jury_assignment
check conflict rules
change status to Accepted
submit_jury_vote
verify assignment is active
write vote
if quorum reached, trigger aggregation
create sanction(s)
create public log entry
emit reputation events
get_my_reputation
return local trust summary
admin_assign_jury
temporary admin-only backstop for MVP
builds jury pool and assignments
admin_close_case
emergency/manual path while jury system matures
10. crates/api/routes

Lemmy already has route crates for API registration.

File to add
src/governance.rs
Route groups to add first
CRUD-style
POST /api/v4/governance/report
POST /api/v4/governance/endorsement
POST /api/v4/governance/endorsement/revoke
POST /api/v4/governance/appeal
Read/workflow
GET /api/v4/governance/case
GET /api/v4/governance/cases
GET /api/v4/governance/modlog
GET /api/v4/governance/reputation/me
GET /api/v4/governance/jury/me
POST /api/v4/governance/jury/accept
POST /api/v4/governance/jury/decline
POST /api/v4/governance/jury/vote
Temporary admin backstops
POST /api/v4/governance/admin/assign-jury
POST /api/v4/governance/admin/close-case
11. crates/apub/objects

Lemmy already splits APub into objects, activities, send, and apub.

Files to add
src/governance/moderation_label.rs
src/governance/trust_attestation.rs
src/governance/sanction_notice.rs
Structs to add first
ModerationLabelObject
pub struct ModerationLabelObject {
    pub id: Url,
    pub actor: Url,
    pub target: Url,
    pub label: String,
    pub summary: Option<String>,
    pub published: DateTime<Utc>,
}
TrustAttestationObject
pub struct TrustAttestationObject {
    pub id: Url,
    pub actor: Url,
    pub subject: Url,
    pub attestation_type: AttestationType,
    pub valid_until: Option<DateTime<Utc>>,
    pub published: DateTime<Utc>,
}
SanctionNoticeObject
pub struct SanctionNoticeObject {
    pub id: Url,
    pub actor: Url,
    pub target: Url,
    pub action: SanctionAction,
    pub scope: SanctionScope,
    pub summary: String,
    pub published: DateTime<Utc>,
}
12. crates/apub/activities
Files to add
src/governance/publish_label.rs
src/governance/publish_attestation.rs
src/governance/publish_sanction_notice.rs
Activities to add first
Create for moderation labels
Create for trust attestations
Create for sanction notices
Undo for revoked attestations or rescinded notices

For MVP, keep federation outbound-only at first unless you want remote notices to trigger local workflows immediately.

13. crates/apub/apub
Files to add
src/governance/inbox.rs
src/governance/outbox.rs
src/governance/verify.rs
Functions to add first
receive_remote_sanction_notice
receive_remote_trust_attestation
send_local_sanction_notice
send_local_trust_attestation

For first release:

accept remote notices
store them
do not auto-apply sanctions from remote nodes
surface them as advisory evidence only

That keeps federation safer and simpler.

14. crates/server

Use this only for module registration and startup wiring, which matches how Lemmy composes the rest of the system.

Files to touch
src/lib.rs or equivalent module root
any startup wiring for route registration
background task startup, if present
Add first
governance route registration
periodic reputation snapshot job
periodic expired sanction cleanup job
periodic jury assignment timeout job
15. First exact MVP API set

If you want the smallest useful vertical slice, implement these 11 endpoints first:

Reporting / cases
POST /api/v4/governance/report
GET /api/v4/governance/case
GET /api/v4/governance/cases
Jury
GET /api/v4/governance/jury/me
POST /api/v4/governance/jury/accept
POST /api/v4/governance/jury/decline
POST /api/v4/governance/jury/vote
Appeals
POST /api/v4/governance/appeal
Public transparency
GET /api/v4/governance/modlog
Trust
GET /api/v4/governance/reputation/me
POST /api/v4/governance/endorsement

That is enough for:

report
escalate
jury
decide
publish log
inspect trust
16. Recommended implementation order
Step 1

Schema + Diesel:

moderation_case
jury_assignment
jury_vote
sanction
public_case_log
Step 2

Views:

GovernanceCaseSummaryView
JuryQueueView
GovernanceModlogView
Step 3

API common DTOs

Step 4

Endpoints:

create report
get case
jury queue
submit vote
modlog
Step 5

Add:

endorsement
reputation_event
reputation_snapshot
Step 6

Federation objects/activities

17. Minimal aggregation rules to ship first

When submit_jury_vote is called, implement this simple logic first:

if submitted votes < quorum, persist and stop
if quorum reached:
tally majority
create one sanction
mark case Decided
insert public_case_log
add reputation_event for jurors and reporters
open appeal window

Start simple:

5 jurors
quorum = 3
majority wins for non-severe sanctions
reserve supermajority for suspension later
18. One practical shortcut

For MVP, don’t model separate report and case tables unless you need full backward compatibility immediately.

A workable shortcut is:

every report creates or appends to a moderation_case
keep a child case_report table only if you need per-reporter history

That reduces complexity a lot.

19. One practical warning

Do not overbuild reputation in v1.

You only need these booleans first:

jury_eligible
trusted_reporter

And these four counters:

reporting_accuracy
jury_reliability
participation_consistency
endorsement_strength

Everything else can be derived later.

20. What to code first on Monday

If I were handing this to engineers, the first coding sprint would be:

migration for moderation_case, jury_assignment, jury_vote, sanction, public_case_log
Diesel models for those tables
db_views/governance_case + db_views/jury_queue
api_common/governance.rs
endpoints:
POST report
GET case
GET jury/me
POST jury/vote
GET modlog

That gives you a true end-to-end slice fast.

I can turn this into a sprint-by-sprint engineering plan next, or write a starter Rust file tree with module stubs.