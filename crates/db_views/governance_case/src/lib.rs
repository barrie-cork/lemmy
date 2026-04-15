//! Read models for the governance case workflow.
//!
//! Phase 2a ships a plain-struct summary view and a plain-struct detail
//! row + wrapper pair per [04 §4.1]. None of the view types derive
//! `Selectable` directly because their field sets mix source-table scalars
//! with aggregate subqueries and drift-stub constants — the canonical
//! `modlog` / `report_combined` template requires every field to be
//! either `#[diesel(embed)]` from a source struct or
//! `#[diesel(select_expression = ...)]`, and Phase 2a's shapes cannot
//! satisfy that in a single derive. The impls build each view by mapping
//! an explicit Diesel tuple select (plan §12 R1 and R2 fallback paths,
//! taken pre-emptively).
//!
//! * [`GovernanceCaseSummaryView`] — list rows for community /
//!   target-person / jury-selection screens. Backs tasks 16, 18, and 19.
//! * [`GovernanceCaseDetailRow`] + [`GovernanceCaseDetailView`] — hydrated
//!   single-case shape backing task 17. Gated behind `feature = "full"`
//!   because they reference `ModerationCase` and `Sanction`, which live
//!   in `lemmy_db_schema::source::governance` — a module that is itself
//!   only compiled under `full`.

use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::{CaseSeverity, CaseStatus, CaseTargetType};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  lemmy_db_schema::source::governance::{moderation_case::ModerationCase, sanction::Sanction},
  lemmy_db_schema_file::enums::AppealStatus,
};

#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
/// Summary row for a governance moderation case. Backs list endpoints.
///
/// Plain Rust struct, NOT a Diesel Queryable. Built by mapping an
/// explicit tuple select in the impls module — see
/// `impls::list_open_cases_for_community` for the canonical shape.
pub struct GovernanceCaseSummaryView {
  pub case_id: i32,
  pub status: CaseStatus,
  pub severity: CaseSeverity,
  pub reason_code: String,
  pub opened_at: DateTime<Utc>,
  pub community_id: Option<i32>,
  pub community_name: Option<String>,
  pub target_type: CaseTargetType,
  /// [04 §4.1] drift stub — no report source table in Phase 1. See plan §2 Drift 1.
  pub reporter_count: i64,
  /// Hardcoded per [05 §3] — jury panel size is a v0 constant of 5.
  pub jury_needed: i32,
  /// Count of `jury_assignment` rows with `status = 'submitted'` for this case.
  pub jury_submitted: i32,
}

#[cfg(feature = "full")]
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
/// First-round-trip shape for the detail view. Plain Rust struct built by
/// mapping an explicit tuple select in `impls::read_case_detail`; does NOT
/// derive `Queryable`/`Selectable` because the `evidence_count`,
/// `appeal_status`, and `target_creator_id` fields need aggregate
/// subquery / left-join / COALESCE expressions that cannot be inlined as
/// `#[diesel(select_expression = ...)]` attributes alongside the
/// `ModerationCase` embed without tripping Selectable's auto-resolution
/// of a default table name from the struct name (plan §12 R2, confirmed
/// at task 15 against the tree — the advisor's fallback-of-the-fallback
/// is taken pre-emptively for the detail row as well).
pub struct GovernanceCaseDetailRow {
  pub case_row: ModerationCase,
  pub evidence_count: i64,
  pub appeal_status: Option<AppealStatus>,
  pub target_creator_id: Option<i32>,
}

#[cfg(feature = "full")]
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
/// Hydrated detail view of a single case. Callers that match on
/// `row.case_row.status` MUST handle `CaseStatus::EmergencyRemove` and
/// `CaseStatus::AdminReview` exhaustively per ADR-013.
pub struct GovernanceCaseDetailView {
  pub row: GovernanceCaseDetailRow,
  pub sanctions: Vec<Sanction>,
}
