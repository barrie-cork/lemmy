use crate::objects::person::ApubPerson;
use activitypub_federation::fetch::object_id::ObjectId;
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::{SanctionAction, SanctionScope};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

/// AP `type` discriminator for [`SanctionNoticeProtocol`].
///
/// Single-variant enum mirroring `activitystreams_kinds::object::NoteType`
/// (see `activitypub_federation::kinds::object::NoteType`); serde renames the
/// variant to the literal string `"SanctionNotice"` so untagged dispatch in
/// `SharedInboxActivities` can distinguish governance Create wrappers from
/// vanilla `Note` Create wrappers.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, Default)]
pub enum SanctionNoticeType {
  #[default]
  SanctionNotice,
}

/// AP wire-form of a governance sanction notice.
///
/// Outbound-only in v0 — Phase 6 task 74 builds this from a winning
/// `Sanction` row + redacted reason; Phase 6 task 75 deserialises it from
/// inbound activities and persists to `remote_sanction_notice` (advisory,
/// `local_case_id` stays NULL per ADR-006).
///
/// `target` is a bare `Url` (not `ObjectId<T>`) because v0 does not
/// dereference the target on receive — the advisory notice records the
/// target ap_id verbatim and admin review handles dispatch. v1 may switch
/// to `ObjectId<TargetEnum>` once the inbound apply path lands.
///
/// `summary` MUST be passed through `redaction::scrub()` before
/// serialisation per [docs/brehon-law-inspired-network/06-security-and-threat-model.md §4.2].
/// Phase 6 task 74 owns the scrub call; this struct is a transport
/// container only.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SanctionNoticeProtocol {
  #[serde(rename = "type")]
  pub(crate) kind: SanctionNoticeType,
  pub id: Url,
  pub actor: ObjectId<ApubPerson>,
  pub target: Url,
  pub action: SanctionAction,
  pub scope: SanctionScope,
  pub summary: String,
  pub published: DateTime<Utc>,
}
