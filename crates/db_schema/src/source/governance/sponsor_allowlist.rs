use crate::newtypes::{CommunityId, SponsorAllowlistId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::PersonId;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::sponsor_allowlist;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = sponsor_allowlist))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A community-scoped allowlist row for sponsor eligibility. Reserved in
/// v1-AD-a; no read-path callers yet (they land in v1-AD-b alongside the
/// `onboarding.sponsor_allowlist_table_name` resolver). UNIQUE
/// (community_id, person_id) is the conflict target for idempotent
/// inserts.
pub struct SponsorAllowlist {
  pub id: SponsorAllowlistId,
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub created_at: DateTime<Utc>,
}

/// Insert form for `sponsor_allowlist`. No `AsChangeset` — rows are
/// add-or-delete, never updated in place.
#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = sponsor_allowlist))]
pub struct SponsorAllowlistInsertForm {
  pub community_id: CommunityId,
  pub person_id: PersonId,
}
