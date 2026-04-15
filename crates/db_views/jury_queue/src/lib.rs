//! Read models for the jury queue and per-juror assignments.
//!
//! Phase 2a ships one plain-struct view per [04 §4.2]:
//!
//! * [`JuryQueueView`] — one row per (case, juror-assignment-or-available-
//!   case) pair in the queue. Backs tasks 22 and 23. The struct is a
//!   plain Rust type (no `Queryable`/`Selectable` derives); impls build
//!   it by mapping an explicit Diesel tuple select, same pattern as
//!   `lemmy_db_views_governance_case`.

use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::CaseSeverity;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
/// A single row in the jury queue — either (a) a case currently assigned
/// to the viewing juror, or (b) a case they could opt into. Which of the
/// two semantics the row has is determined by which query produced it:
/// `impls::list_jury_assignments_for_person` (assigned) vs
/// `impls::list_available_jury_cases_for_person` (available). Both
/// queries land in task 22/23.
///
/// `deadline_at` is always `None` in Phase 2a. Per plan §2 Drift 3 the
/// underlying schema has no `deadline_at` column and [04 §4.2] is
/// internally inconsistent about whether it should be stored or computed.
/// Phase 4 wiring will resolve this when it adds the jury-timeout job.
pub struct JuryQueueView {
  pub case_id: i32,
  pub severity: CaseSeverity,
  pub reason_code: String,
  pub opened_at: DateTime<Utc>,
  /// Always `None` in Phase 2a — see plan §2 Drift 3.
  pub deadline_at: Option<DateTime<Utc>>,
  pub community_id: Option<i32>,
  pub community_name: Option<String>,
}
