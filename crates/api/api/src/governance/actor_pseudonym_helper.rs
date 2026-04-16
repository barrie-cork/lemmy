//! GDPR pseudonym allocator for governance-log entries.
//!
//! [ADR-015] forbids writing `person_id`, `local_user_id`, or `username`
//! into `governance_log`, `public_case_log`, or any fork-only federation
//! payload. Every governance write that needs to attribute an action to
//! an actor MUST call [`get_or_create`] to materialise a stable, opaque
//! pseudonym and use that instead.

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::source::governance::actor_pseudonym::ActorPseudonymInsertForm;
use lemmy_db_schema_file::{PersonId, schema::actor_pseudonym};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;
use uuid::Uuid;

/// Return the caller's pseudonym, inserting a fresh UUIDv4 row on first
/// use. Idempotent under concurrent insertion: if a racing writer wins
/// the `(person_id)` unique constraint, this function re-reads the
/// winning row instead of surfacing a duplicate-key error.
pub async fn get_or_create(pool: &mut DbPool<'_>, person_id: PersonId) -> LemmyResult<String> {
  let conn = &mut get_conn(pool).await?;

  if let Some(existing) = actor_pseudonym::table
    .filter(actor_pseudonym::person_id.eq(person_id))
    .select(actor_pseudonym::pseudonym)
    .first::<String>(conn)
    .await
    .optional()?
  {
    return Ok(existing);
  }

  let pseudonym = Uuid::new_v4().to_string();
  let form = ActorPseudonymInsertForm {
    person_id,
    pseudonym: pseudonym.clone(),
  };

  match insert_into(actor_pseudonym::table)
    .values(&form)
    .returning(actor_pseudonym::pseudonym)
    .get_result::<String>(conn)
    .await
  {
    Ok(inserted) => Ok(inserted),
    Err(diesel::result::Error::DatabaseError(
      diesel::result::DatabaseErrorKind::UniqueViolation,
      _,
    )) => {
      let existing = actor_pseudonym::table
        .filter(actor_pseudonym::person_id.eq(person_id))
        .select(actor_pseudonym::pseudonym)
        .first::<String>(conn)
        .await?;
      Ok(existing)
    }
    Err(e) => Err(e.into()),
  }
}
