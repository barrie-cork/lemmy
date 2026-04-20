use crate::newtypes::{CommunityId, RuleSetVersionId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::PersonId;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::rule_set_version;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = rule_set_version))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// An append-only rule-set version per community. `version` is the
/// monotonically increasing revision number; `parent_id` points at the
/// previous revision (null for the initial version). `text_sha256` is the
/// canonical 32-byte SHA-256 over `rule_text`; consumers hex-encode for
/// display. Writes go through v1-AD-c's rule-set create handler; reads are
/// pinned on cases via `moderation_case.rule_set_version_id` at decision
/// time. Never mutate — per ADR-010 append-only invariant.
pub struct RuleSetVersion {
  pub id: RuleSetVersionId,
  pub community_id: CommunityId,
  pub version: i32,
  pub parent_id: Option<RuleSetVersionId>,
  pub text_sha256: Vec<u8>,
  pub rule_text: String,
  pub created_at: DateTime<Utc>,
  pub created_by: Option<PersonId>,
}

/// Insert form for `rule_set_version`. Append-only — deliberately does NOT
/// derive `AsChangeset`. Editing a past rule-set text would break in-flight
/// juries grandfathered against `moderation_case.rule_set_version_id`.
#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = rule_set_version))]
pub struct RuleSetVersionInsertForm {
  pub community_id: CommunityId,
  pub version: i32,
  pub parent_id: Option<RuleSetVersionId>,
  pub text_sha256: Vec<u8>,
  pub rule_text: String,
  pub created_by: Option<PersonId>,
}
