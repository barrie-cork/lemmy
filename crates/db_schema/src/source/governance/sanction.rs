use crate::newtypes::{CommentId, CommunityId, ModerationCaseId, PostId, SanctionId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::{
  PersonId,
  enums::{SanctionAction, SanctionScope},
};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::sanction;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = sanction))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A sanction applied to a target as the outcome of a moderation case.
pub struct Sanction {
  pub id: SanctionId,
  pub case_id: ModerationCaseId,
  pub scope: SanctionScope,
  pub action: SanctionAction,
  pub target_person_id: Option<PersonId>,
  pub target_post_id: Option<PostId>,
  pub target_comment_id: Option<CommentId>,
  pub target_community_id: Option<CommunityId>,
  pub starts_at: DateTime<Utc>,
  pub ends_at: Option<DateTime<Utc>>,
  pub active: bool,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = sanction))]
pub struct SanctionInsertForm {
  pub case_id: ModerationCaseId,
  pub scope: SanctionScope,
  pub action: SanctionAction,
  pub target_person_id: Option<PersonId>,
  pub target_post_id: Option<PostId>,
  pub target_comment_id: Option<CommentId>,
  pub target_community_id: Option<CommunityId>,
  pub ends_at: Option<DateTime<Utc>>,
  pub active: Option<bool>,
}
