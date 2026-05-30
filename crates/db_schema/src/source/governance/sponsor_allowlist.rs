use crate::newtypes::{CommunityId, SponsorAllowlistId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
#[cfg(feature = "full")]
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::sponsor_allowlist;
#[cfg(feature = "full")]
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = sponsor_allowlist))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Per-community sponsor-eligibility allowlist row. NULL community_id
/// means instance-wide.
pub struct SponsorAllowlist {
  pub id: SponsorAllowlistId,
  /// v1-RT-r1: relaxed from NOT NULL to nullable.
  pub community_id: Option<CommunityId>,
  pub person_id: PersonId,
  pub created_at: DateTime<Utc>,
  /// v1-RT-r1: admin who added the row (audit trail).
  pub added_by_admin_id: PersonId,
  /// v1-RT-r1: admin-supplied free-text rationale.
  pub note: Option<String>,
}

/// Insert form for `sponsor_allowlist`. No `AsChangeset` — rows are
/// add-or-delete, never updated in place.
#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = sponsor_allowlist))]
pub struct SponsorAllowlistInsertForm {
  /// v1-RT-r1: relaxed from required to optional.
  pub community_id: Option<CommunityId>,
  pub person_id: PersonId,
  /// v1-RT-r1: required at insert time. r4 endpoints set explicitly.
  pub added_by_admin_id: PersonId,
  /// v1-RT-r1: optional admin note.
  pub note: Option<String>,
}

#[cfg(feature = "full")]
pub async fn sponsor_allowlist_insert(
  form: &SponsorAllowlistInsertForm,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<SponsorAllowlist> {
  let row = diesel::insert_into(sponsor_allowlist::table)
    .values(form)
    .returning(SponsorAllowlist::as_returning())
    .get_result(conn)
    .await?;
  Ok(row)
}

#[cfg(feature = "full")]
pub async fn sponsor_allowlist_delete(
  allowlist_id: SponsorAllowlistId,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<usize> {
  let count = diesel::delete(sponsor_allowlist::table.find(allowlist_id))
    .execute(conn)
    .await?;
  Ok(count)
}

#[cfg(feature = "full")]
pub async fn sponsor_allowlist_exists(
  person_id: PersonId,
  community_id: Option<CommunityId>,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<bool> {
  let found = match community_id {
    Some(cid) => {
      sponsor_allowlist::table
        .filter(sponsor_allowlist::person_id.eq(person_id))
        .filter(sponsor_allowlist::community_id.eq(cid))
        .first::<SponsorAllowlist>(conn)
        .await
        .optional()?
        .is_some()
    }
    None => {
      sponsor_allowlist::table
        .filter(sponsor_allowlist::person_id.eq(person_id))
        .filter(sponsor_allowlist::community_id.is_null())
        .first::<SponsorAllowlist>(conn)
        .await
        .optional()?
        .is_some()
    }
  };
  Ok(found)
}
