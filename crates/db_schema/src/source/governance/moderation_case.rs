use crate::newtypes::{CommentId, CommunityId, ModerationCaseId, PostId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseSeverity, CaseStatus, CaseTargetType},
};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::moderation_case;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = moderation_case))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A governance moderation case — the central artefact of the jury workflow.
pub struct ModerationCase {
  pub id: ModerationCaseId,
  pub community_id: Option<CommunityId>,
  pub creator_id: Option<PersonId>,
  pub target_type: CaseTargetType,
  pub target_post_id: Option<PostId>,
  pub target_comment_id: Option<CommentId>,
  pub target_person_id: Option<PersonId>,
  pub target_community_id: Option<CommunityId>,
  pub target_remote_url: Option<String>,
  pub reason_code: String,
  pub severity: CaseSeverity,
  pub status: CaseStatus,
  pub threshold_score: i64,
  pub opened_at: DateTime<Utc>,
  pub decided_at: Option<DateTime<Utc>>,
  pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = moderation_case))]
pub struct ModerationCaseInsertForm {
  pub community_id: Option<CommunityId>,
  pub creator_id: Option<PersonId>,
  pub target_type: CaseTargetType,
  pub target_post_id: Option<PostId>,
  pub target_comment_id: Option<CommentId>,
  pub target_person_id: Option<PersonId>,
  pub target_community_id: Option<CommunityId>,
  pub target_remote_url: Option<String>,
  pub reason_code: String,
  pub severity: CaseSeverity,
  pub status: CaseStatus,
  pub threshold_score: i64,
}
