use lemmy_db_schema::newtypes::{CommunityId, EndorsementId, ModerationCaseId};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseStatus, CaseTargetType, JuryDecision},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
