use crate::newtypes::{ModerationCaseId, RemoteModerationLabelId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::{FederationInboxAdminAction, FederationPeerTrust};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::remote_moderation_label;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = remote_moderation_label))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RemoteModerationLabel {
  pub id: RemoteModerationLabelId,
  pub source_instance: String,
  pub actor_url: String,
  pub target_url: String,
  pub label: String,
  pub summary: Option<String>,
  pub published_at: DateTime<Utc>,
  pub signature: String,
  pub local_case_id: Option<ModerationCaseId>,
  pub received_at: DateTime<Utc>,
  pub peer_trust_level_at_receipt: FederationPeerTrust,
  pub admin_reviewed_at: Option<DateTime<Utc>>,
  pub admin_action: FederationInboxAdminAction,
  pub dismissal_rationale: Option<String>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = remote_moderation_label))]
pub struct RemoteModerationLabelInsertForm {
  pub source_instance: String,
  pub actor_url: String,
  pub target_url: String,
  pub label: String,
  pub summary: Option<String>,
  pub published_at: DateTime<Utc>,
  pub signature: String,
  pub local_case_id: Option<ModerationCaseId>,
  pub peer_trust_level_at_receipt: Option<FederationPeerTrust>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = remote_moderation_label))]
pub struct RemoteModerationLabelUpdateForm {
  pub local_case_id: Option<Option<ModerationCaseId>>,
  pub admin_reviewed_at: Option<DateTime<Utc>>,
  pub admin_action: Option<FederationInboxAdminAction>,
  pub dismissal_rationale: Option<Option<String>>,
}
