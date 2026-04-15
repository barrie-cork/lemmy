use crate::newtypes::{JuryVoteId, ModerationCaseId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::{PersonId, enums::JuryDecision};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::jury_vote;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = jury_vote))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A juror's vote on a moderation case outcome.
pub struct JuryVote {
  pub id: JuryVoteId,
  pub case_id: ModerationCaseId,
  pub juror_id: PersonId,
  pub decision: JuryDecision,
  pub rationale: Option<String>,
  pub submitted_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = jury_vote))]
pub struct JuryVoteInsertForm {
  pub case_id: ModerationCaseId,
  pub juror_id: PersonId,
  pub decision: JuryDecision,
  pub rationale: Option<String>,
}
