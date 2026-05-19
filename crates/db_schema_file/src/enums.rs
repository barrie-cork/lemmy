#[cfg(feature = "full")]
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::PostSortTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The post sort types. See here for descriptions: https://join-lemmy.org/docs/en/users/03-votes-and-ranking.html
pub enum PostSortType {
  #[default]
  Active,
  Hot,
  New,
  Old,
  Top,
  MostComments,
  NewComments,
  Controversial,
  Scaled,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CommentSortTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The comment sort types. See here for descriptions: https://join-lemmy.org/docs/en/users/03-votes-and-ranking.html
pub enum CommentSortType {
  #[default]
  Hot,
  Top,
  New,
  Old,
  Controversial,
}

#[derive(Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::ListingTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A listing type for post and comment list fetches.
pub enum ListingType {
  /// Content from your own site, as well as all connected / federated sites.
  All,
  /// Content from your site only.
  #[default]
  Local,
  /// Content only from communities you've subscribed to.
  Subscribed,
  /// Content that you can moderate (because you are a moderator of the community it is posted to)
  ModeratorView,
  /// Communities which are recommended by local instance admins
  Suggested,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::RegistrationModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The registration mode for your site. Determines what happens after a user signs up.
pub enum RegistrationMode {
  /// Closed to public.
  Closed,
  /// Open, but pending approval of a registration application.
  RequireApplication,
  /// Open to all.
  #[default]
  Open,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::PostListingModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A post-view mode that changes how multiple post listings look.
pub enum PostListingMode {
  /// A compact, list-type view.
  #[default]
  List,
  /// A larger card-type view.
  Card,
  /// A smaller card-type view, usually with images as thumbnails
  SmallCard,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CommunityVisibility"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Defines who can browse and interact with content in a community.
pub enum CommunityVisibility {
  /// Public community, any local or federated user can interact.
  #[default]
  Public,
  /// Community is unlisted/hidden and doesn't appear in community list. Posts from the community
  /// are not shown in Local and All feeds, except for subscribed users.
  Unlisted,
  /// Unfederated community, only local users can interact (with or without login).
  LocalOnlyPublic,
  /// Unfederated  community, only logged-in local users can interact.
  LocalOnlyPrivate,
  /// Users need to be approved by mods before they are able to browse or post.
  Private,
}

impl CommunityVisibility {
  pub fn can_federate(&self) -> bool {
    use CommunityVisibility::*;
    self != &LocalOnlyPublic && self != &LocalOnlyPrivate
  }
  pub fn can_view_without_login(&self) -> bool {
    use CommunityVisibility::*;
    self == &Public || self == &LocalOnlyPublic
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::FederationModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The federation mode for an item
pub enum FederationMode {
  #[default]
  /// Allows all
  All,
  /// Allows only local
  Local,
  /// Disables
  Disable,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::ImageModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A mode for setting how pictrs handles images.
pub enum ImageMode {
  /// Leave images unchanged, don't generate any local thumbnails for post urls. Instead the
  /// Opengraph image is directly returned as thumbnail
  None,
  /// Generate thumbnails for external post urls and store them persistently in pict-rs. This
  /// ensures that they can be reliably retrieved and can be resized using pict-rs APIs. However it
  /// also increases storage usage.
  ///
  /// This behaviour matches Lemmy 0.18.
  StoreLinkPreviews,
  /// If enabled, all images from remote domains are rewritten to pass through
  /// `/api/v4/image/proxy`, including embedded images in markdown. Images are stored temporarily in
  /// pict-rs for caching. This improves privacy as users don't expose their IP to untrusted
  /// servers, and decreases load on other servers. However it increases bandwidth use for the local
  /// server.
  ///
  /// Requires pict-rs 0.5
  #[default]
  ProxyAllImages,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::ActorTypeEnum"
)]
pub enum ActorType {
  Site,
  Community,
  Person,
  MultiCommunity,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CommunityFollowerState"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub enum CommunityFollowerState {
  Accepted,
  Pending,
  ApprovalRequired,
  Denied,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::TagColorEnum"
)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Color of community tag.
pub enum TagColor {
  #[default]
  Color01,
  Color02,
  Color03,
  Color04,
  Color05,
  Color06,
  Color07,
  Color08,
  Color09,
  Color10,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::VoteShowEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Lets you show votes for others only, show all votes, or hide all votes.
pub enum VoteShow {
  #[default]
  Show,
  ShowForOthers,
  Hide,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::PostNotificationsModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Available settings for post notifications
pub enum PostNotificationsMode {
  AllComments,
  #[default]
  RepliesAndMentions,
  Mute,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CommunityNotificationsModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Available settings for community notifications
pub enum CommunityNotificationsMode {
  AllPostsAndComments,
  AllPosts,
  #[default]
  RepliesAndMentions,
  Mute,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::NotificationTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Types of notifications which can be received in inbox
pub enum NotificationType {
  // Necessary for enumstring
  #[default]
  Mention,
  Reply,
  Subscribed,
  PrivateMessage,
  ModAction,
}

#[derive(Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::ModlogKind"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A list of possible types for the various modlog actions.
pub enum ModlogKind {
  // Necessary for enumstring
  #[default]
  AdminAdd,
  AdminBan,
  AdminAllowInstance,
  AdminBlockInstance,
  AdminPurgeComment,
  AdminPurgeCommunity,
  AdminPurgePerson,
  AdminPurgePost,
  ModAddToCommunity,
  ModBanFromCommunity,
  AdminFeaturePostSite,
  ModFeaturePostCommunity,
  ModChangeCommunityVisibility,
  ModLockPost,
  ModRemoveComment,
  AdminRemoveCommunity,
  ModRemovePost,
  ModTransferCommunity,
  ModLockComment,
  ModWarnComment,
  ModWarnPost,
}

// ========================================================================
// Governance enums (Phase 1)
// Each mirrors the DbEnum / ExistingTypePath / DbValueStyle="verbatim" pattern
// used by the existing enums above (see e.g. RegistrationMode).
// The Postgres enum types are created in migrations/{ts}_add_governance_enums.
// Variant names are PascalCase to match DbValueStyle="verbatim".
// ========================================================================

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CaseStatus"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Lifecycle of a governance moderation case.
pub enum CaseStatus {
  #[default]
  Open,
  ThresholdMet,
  JurySelection,
  InReview,
  Decided,
  Appealed,
  Closed,
  /// Admin invoked the emergency-remove override. A jury reviews post-facto;
  /// the removal stands regardless of the jury's finding. Per ADR-013.
  EmergencyRemove,
  /// A direct moderator action (Lemmy compat layer) paused the case. Admin must
  /// explicitly resume. Per OQ-008 resolution.
  AdminReview,
  /// v1-SL-a §8.1 + PRD §3.1 (OQ-025 athgabál grace window). Case has
  /// been Decided AND a sanction with sponsor-liability implications
  /// was created; the case is in its grace window.
  /// `moderation_case.grace_expires_at` holds the computed deadline.
  /// Sponsor revocation OR defendant restoration during this window
  /// transitions to `SponsorLiabilityEscaped`. Window expiry triggers
  /// `SponsorLiabilityFired`. Set by SL-d (`submit_jury_vote` rewrite);
  /// pre-v1 backfill in v1-SL-a sets it for v0 mid-flight cases.
  SponsorLiabilityPending,
  /// v1-SL-a §8.1 + PRD §3.1. Terminal — the grace window expired
  /// without escape. The `apply_sponsor_liability` helper (v0 Phase 5b
  /// code, gated post-SL-c) ran and the `reputation_event` rows for
  /// sponsors were written. Set by SL-c scheduler.
  SponsorLiabilityFired,
  /// v1-SL-a §8.1 + PRD §3.1. Terminal — sponsor revoked OR defendant
  /// restored within the grace window. `moderation_case.liability_escape_reason`
  /// records the escape mechanism. No `reputation_event` rows for
  /// sponsors were written. Set by SL-b (revoke_endorsement) or SL-c
  /// (scheduler escape branch).
  SponsorLiabilityEscaped,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CaseTargetType"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The kind of entity a moderation case targets.
pub enum CaseTargetType {
  #[default]
  Post,
  Comment,
  Person,
  Community,
  RemoteInstance,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CaseSeverity"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Severity classification assigned to a moderation case.
pub enum CaseSeverity {
  Low,
  #[default]
  Medium,
  High,
  Critical,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::EvidenceVisibility"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Who can view a piece of evidence attached to a case.
pub enum EvidenceVisibility {
  #[default]
  JuryOnly,
  PrivateAdmin,
  PublicRedacted,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::JuryAssignmentStatus"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Juror's response state for a case assignment.
pub enum JuryAssignmentStatus {
  #[default]
  Selected,
  Accepted,
  Declined,
  Conflicted,
  Submitted,
  Expired,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::JuryDecision"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A juror's vote on the outcome of a case.
pub enum JuryDecision {
  #[default]
  NoAction,
  AdvisoryLabel,
  Warning,
  Cooldown,
  RemoveContent,
  SuspendLocalUser,
  SuspendCommunityMember,
  RecommendFederationAction,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::SanctionScope"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Jurisdictional scope of a sanction.
pub enum SanctionScope {
  #[default]
  Community,
  Instance,
  FederatedRecommendation,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::SanctionAction"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The concrete action applied by a sanction.
pub enum SanctionAction {
  #[default]
  Label,
  VisibilityReduction,
  TemporaryRestriction,
  ContentRemoval,
  CommunityExclusion,
  InstanceSuspension,
  FederationQuarantineRecommendation,
  /// v0 reserved slot for `folog n-othrusa`-style restorative sanctions per
  /// [99 OQ-003 (amended 2026-04-17)]. Not selected by any v0 handler — reserved
  /// for v1 `admin_restorative_action` and for enum-exhaustiveness in downstream
  /// matches. Severity bucket: minor (same as `Label`) per §11.1 GOTCHA-56a.
  /// Unit variant (per GOTCHA-56b fallback) because `SanctionAction: Copy`; the
  /// description column lands in a sibling migration if v1 needs payloaded
  /// restoration.
  Restoration,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::AppealStatus"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Lifecycle of an appeal on a decided case.
pub enum AppealStatus {
  #[default]
  Requested,
  Accepted,
  Rejected,
  Decided,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::ReputationDimension"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Dimensions along which reputation is tracked per-actor.
pub enum ReputationDimension {
  #[default]
  ReportingAccuracy,
  JuryReliability,
  ParticipationConsistency,
  EndorsementStrength,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::ReputationEventSourceType"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Per-event source classification for `reputation_event` rows. Per
/// PRD section 5.3 + 7. v0 rows receive Endorsement via column DEFAULT;
/// 2026-05-10-000200-0000 backfill revises by reason ILIKE per DQ #184.
/// r3 emitters write the matching variant explicitly going forward.
pub enum ReputationEventSourceType {
  #[default]
  Endorsement,
  JuryVote,
  SponsorLiability,
  FounderSeed,
  ParticipationCron,
  DormancyCron,
  VoteOutcome,
  EvidenceQuality,
  ManualSeed,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::AttestationType"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Kinds of trust/governance attestations the local instance signs and
/// federates outbound (Phase 6). The Postgres enum `attestation_type` was
/// added in Phase 1's `add_governance_enums` migration; the matching
/// `sql_types::AttestationType` and this Rust enum land alongside the
/// `federation_attestation` table per Phase 6 task 71.
pub enum AttestationType {
  #[default]
  TrustedReporter,
  JuryEligible,
  SanctionNotice,
  QuarantineRecommendation,
}

// ========================================================================
// Governance enums (Phase 5a)
// ========================================================================

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::MembershipState"
)]
#[cfg_attr(feature = "full", DbValueStyle = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Deferred-enforcement membership-state flag per [99 OQ-016]. Ships in v0
/// so v1 can flip `config.onboarding.enforce_membership_state = true` without
/// a schema migration on `person` (expensive at scale). v0 handlers MUST NOT
/// read this column — `scripts/brehon/lint-no-membership-read.sh` enforces
/// the silence. DB tokens are lowercase (`member`/`provisional`/`suspended`)
/// per `DbValueStyle = "snake_case"`; this deliberately differs from the
/// PascalCase verbatim convention of other governance enums because
/// `onboarding.default_membership_state` is a config-text value that must
/// round-trip as a plain lowercase string (see plan §12.1 seed row).
pub enum MembershipState {
  #[default]
  Member,
  Provisional,
  Suspended,
}

// ========================================================================
// Governance enums (v1-JM-a — jury mechanics sub-phase A)
//
// These three enums frame the procedural state every JM-b/c/d/e read or
// write keys off of. All three use DbValueStyle = "verbatim" mirroring
// CaseStatus / JuryDecision / SanctionAction (the majority pattern); they
// are read/written by governance handler code, not by config-text
// round-trips. PascalCase variants match the PostgreSQL CREATE TYPE
// values in migrations/2026-04-23-000000-0000_add_jury_mechanics_enums.
// ========================================================================

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::SeverityTier"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// v1 jury-mechanics severity tier per PRD §3.1. Maps `SanctionAction` /
/// case context to a procedural threshold tier (Minor/Moderate/Severe).
/// Frozen at admin_assign_jury time per ADR-010 (no retroactive
/// invalidation of in-flight juries) — `moderation_case.severity_tier`
/// is the snapshotted value; mid-flight config changes do not alter it.
pub enum SeverityTier {
  #[default]
  Minor,
  Moderate,
  Severe,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CaseStatusTier"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// v1 jury-mechanics target-status tier per PRD §3.1. Determined from
/// the target's `reputation_event` / `membership_state` at case-open time
/// (Founder seeded > Regular default > Probation triggered by adverse
/// reputation events). Cascade key for `jury.panel_size.<status>.<severity>`.
pub enum CaseStatusTier {
  Founder,
  #[default]
  Regular,
  Probation,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::JuryAssignmentRole"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// v1 jury-mechanics role discriminator per PRD §8.2. Distinguishes
/// original-jury rows from appeal-jury rows on the same case so the
/// appeal-panel-pick query (v1-JM-d) can exclude original jurors via
/// `WHERE role = 'Original'` while the appeal panel writes `Appeal` rows.
pub enum JuryAssignmentRole {
  #[default]
  Original,
  Appeal,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::AppealRequesterRole"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// v1-JM-d §6.4 / §9.3: discriminates the requester of an appeal so
/// the eligibility check (defendant always; reporter only on NoAction
/// / AdvisoryLabel) can be reproduced from a stored row without
/// re-walking case ownership. Backfill DEFAULT 'Defendant' covers
/// pre-v1 rows (pre-JM-d, only the target_person_id could appeal).
pub enum AppealRequesterRole {
  #[default]
  Defendant,
  OriginalReporter,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::JuryConstraintRelaxationReason"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// v1 jury-mechanics constraint-relaxation reason code per PRD §5.3
/// R1/R2/R3 cascade + PRD §8.3's explicit `AdminOverride` extension.
/// Written by v1-JM-b's `select_eligible_jurors` every time a
/// diversity/recency/cluster constraint is relaxed during panel
/// assembly. Bounded vocabulary (no `Other`) per PR #92 cr-9
/// resolution — the four codes cover the full cascade surface
/// documented in PRD §5.3 + the admin-bypass path in §8.3. Adding a
/// new reason requires a new enum variant + Postgres enum migration;
/// that friction is the ADR-015 pseudonymisation safeguard (replaces
/// the originally-proposed free-text `relaxation_reason TEXT` which
/// could have leaked usernames / emails into the governance audit
/// log).
///
/// Serde-renders as snake_case (e.g. `SmallPool` → `"small_pool"`)
/// to match PRD §5.3's narrative vocabulary in `governance_log`
/// payload fields. Postgres enum literal is PascalCase (`'SmallPool'`)
/// matching the `DbValueStyle = "verbatim"` convention shared with
/// `SeverityTier` / `CaseStatusTier` / `JuryAssignmentRole`.
pub enum JuryConstraintRelaxationReason {
  /// PRD §5.3 R1: Phase 1 pool too small post-cooldown — cheapest
  /// relaxation, lifts `no_recent_juror_repeat`.
  #[default]
  SmallPool,
  /// PRD §5.3 R2: Phase 2 sample violates
  /// `no_majority_from_same_sponsor_cluster` after N_RETRIES re-rolls;
  /// drop `geographic_diversity_preferred` as a soft bias.
  ClusterPressure,
  /// PRD §5.3 R3: Phase 2 still violates after R2 — drop
  /// `no_majority_from_same_sponsor_cluster` entirely (warn!-logged).
  ClusterPressureExhausted,
  /// PRD §8.3: admin explicitly bypassed the cascade (e.g. emergency
  /// panel assembly under ADR-013 EmergencyRemove pathway).
  AdminOverride,
}

// v1-federation-inbound-a enums (Task 6 — SQL values are lowercase snake_case)

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::FederationPeerTrustEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Per-peer trust level used by the federation inbox router.
/// SQL enum `federation_peer_trust_enum` (lowercase snake_case values).
pub enum FederationPeerTrust {
  #[default]
  Unknown,
  Allowlisted,
  UntrustedReceive,
  Blocklisted,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::FederationInboxAdminActionEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Admin review outcome for inbound federation objects awaiting review.
/// SQL enum `federation_inbox_admin_action_enum` (lowercase snake_case values).
pub enum FederationInboxAdminAction {
  #[default]
  Unreviewed,
  CrossLinked,
  Dismissed,
}
