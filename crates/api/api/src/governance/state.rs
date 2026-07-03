//! Phantom-typed `GovernanceCase<S>` wrappers centralising per-handler `CaseStatus` guards.
//!
//! Each state marker corresponds to the set of `CaseStatus` variants a handler accepts.
//! `TryFrom<ModerationCase>` for each marker encodes the exhaustive allow-list; handlers
//! call `GovernanceCase::<Marker>::try_from(case)?` immediately after the DB load and use
//! `case.inner` downstream — returning `LemmyErrorType::NotFound` (existing behaviour,
//! behaviour-preserving). Pattern: `.claude/lessons/feedback_governance_type_state_handlers.md`.

#[cfg(feature = "full")]
mod inner {
  use std::marker::PhantomData;

  use lemmy_db_schema::source::governance::moderation_case::ModerationCase;
  use lemmy_db_schema_file::enums::CaseStatus;
  use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};

  // ── State marker zero-sized structs ───────────────────────────────────────

  /// Site 1 (accept_jury_assignment, Original arm): JurySelection | InReview.
  pub struct JurySelection;

  /// Site 1 (accept_jury_assignment, Appeal arm) + Site 4 (admin_trigger_appeal_rejury): Appealed.
  pub struct Appealed;

  /// Site 2 (admin_assign_jury): Open | ThresholdMet | EmergencyRemove.
  pub struct PreJuryAssignable;

  /// Site 3 (admin_close_case): all 11 variants except Closed.
  pub struct NotYetClosed;

  /// Site 5 (sponsor_liability_grace per-case wrap): SponsorLiabilityPending.
  pub struct SponsorLiabilityPending;

  /// Site 6 (submit_jury_vote): Open | ThresholdMet | JurySelection | InReview.
  /// The terminal-state idempotency guard preserves `Ok(case_decided:true)` semantics
  /// — see `GovernanceCase::<Active>::try_active_vote`.
  pub struct Active;

  // ── Core wrapper ──────────────────────────────────────────────────────────

  pub struct GovernanceCase<S> {
    pub inner: ModerationCase,
    _state: PhantomData<S>,
  }

  impl<S> GovernanceCase<S> {
    fn new(inner: ModerationCase) -> Self {
      Self {
        inner,
        _state: PhantomData,
      }
    }
  }

  // ── TryFrom impls — exhaustive, no `_` arm (ADR-013) ─────────────────────

  /// Site 1 Original arm: JurySelection | InReview.
  impl TryFrom<ModerationCase> for GovernanceCase<JurySelection> {
    type Error = LemmyError;
    fn try_from(c: ModerationCase) -> LemmyResult<Self> {
      match c.status {
        CaseStatus::JurySelection | CaseStatus::InReview => Ok(Self::new(c)),
        CaseStatus::Open
        | CaseStatus::ThresholdMet
        | CaseStatus::Decided
        | CaseStatus::Appealed
        | CaseStatus::Closed
        | CaseStatus::EmergencyRemove
        | CaseStatus::AdminReview
        | CaseStatus::SponsorLiabilityPending
        | CaseStatus::SponsorLiabilityFired
        | CaseStatus::SponsorLiabilityEscaped => Err(LemmyErrorType::NotFound.into()),
      }
    }
  }

  /// Site 1 Appeal arm + Site 4: Appealed only.
  impl TryFrom<ModerationCase> for GovernanceCase<Appealed> {
    type Error = LemmyError;
    fn try_from(c: ModerationCase) -> LemmyResult<Self> {
      match c.status {
        CaseStatus::Appealed => Ok(Self::new(c)),
        CaseStatus::Open
        | CaseStatus::ThresholdMet
        | CaseStatus::JurySelection
        | CaseStatus::InReview
        | CaseStatus::Decided
        | CaseStatus::Closed
        | CaseStatus::EmergencyRemove
        | CaseStatus::AdminReview
        | CaseStatus::SponsorLiabilityPending
        | CaseStatus::SponsorLiabilityFired
        | CaseStatus::SponsorLiabilityEscaped => Err(LemmyErrorType::NotFound.into()),
      }
    }
  }

  /// Site 2: Open | ThresholdMet | EmergencyRemove.
  impl TryFrom<ModerationCase> for GovernanceCase<PreJuryAssignable> {
    type Error = LemmyError;
    fn try_from(c: ModerationCase) -> LemmyResult<Self> {
      match c.status {
        CaseStatus::Open | CaseStatus::ThresholdMet | CaseStatus::EmergencyRemove => {
          Ok(Self::new(c))
        }
        CaseStatus::JurySelection
        | CaseStatus::InReview
        | CaseStatus::Decided
        | CaseStatus::Appealed
        | CaseStatus::Closed
        | CaseStatus::AdminReview
        | CaseStatus::SponsorLiabilityPending
        | CaseStatus::SponsorLiabilityFired
        | CaseStatus::SponsorLiabilityEscaped => Err(LemmyErrorType::NotFound.into()),
      }
    }
  }

  /// Site 3: all 11 variants except Closed.
  impl TryFrom<ModerationCase> for GovernanceCase<NotYetClosed> {
    type Error = LemmyError;
    fn try_from(c: ModerationCase) -> LemmyResult<Self> {
      match c.status {
        CaseStatus::Open
        | CaseStatus::ThresholdMet
        | CaseStatus::JurySelection
        | CaseStatus::InReview
        | CaseStatus::Decided
        | CaseStatus::Appealed
        | CaseStatus::EmergencyRemove
        | CaseStatus::AdminReview
        | CaseStatus::SponsorLiabilityPending
        | CaseStatus::SponsorLiabilityFired
        | CaseStatus::SponsorLiabilityEscaped => Ok(Self::new(c)),
        CaseStatus::Closed => Err(LemmyErrorType::NotFound.into()),
      }
    }
  }

  /// Site 5: SponsorLiabilityPending only.
  impl TryFrom<ModerationCase> for GovernanceCase<SponsorLiabilityPending> {
    type Error = LemmyError;
    fn try_from(c: ModerationCase) -> LemmyResult<Self> {
      match c.status {
        CaseStatus::SponsorLiabilityPending => Ok(Self::new(c)),
        CaseStatus::Open
        | CaseStatus::ThresholdMet
        | CaseStatus::JurySelection
        | CaseStatus::InReview
        | CaseStatus::Decided
        | CaseStatus::Appealed
        | CaseStatus::Closed
        | CaseStatus::EmergencyRemove
        | CaseStatus::AdminReview
        | CaseStatus::SponsorLiabilityFired
        | CaseStatus::SponsorLiabilityEscaped => Err(LemmyErrorType::NotFound.into()),
      }
    }
  }

  /// Site 6: Open | ThresholdMet | JurySelection | InReview (non-terminal vote-accepting states).
  ///
  /// This site has **success-not-error** semantics on terminal states:
  /// `submit_jury_vote` returns `Ok(case_decided:true)` for terminal cases, NOT a 404.
  /// Use `GovernanceCase::<Active>::try_active_vote(case)` which returns the sentinel
  /// `ActiveVoteResult::AlreadyDecided` on the terminal branch instead of `Err`.
  impl TryFrom<ModerationCase> for GovernanceCase<Active> {
    type Error = LemmyError;
    fn try_from(c: ModerationCase) -> LemmyResult<Self> {
      match c.status {
        CaseStatus::Open
        | CaseStatus::ThresholdMet
        | CaseStatus::JurySelection
        | CaseStatus::InReview => Ok(Self::new(c)),
        CaseStatus::Decided
        | CaseStatus::Closed
        | CaseStatus::Appealed
        | CaseStatus::EmergencyRemove
        | CaseStatus::AdminReview
        | CaseStatus::SponsorLiabilityPending
        | CaseStatus::SponsorLiabilityFired
        | CaseStatus::SponsorLiabilityEscaped => Err(LemmyErrorType::NotFound.into()),
      }
    }
  }

  /// Result of `GovernanceCase::<Active>::try_active_vote` — preserves the
  /// success-early-return semantics of `submit_jury_vote`'s terminal-state guard.
  // ponytail: allow the size gap, don't Box. This enum is built once per vote and
  // consumed immediately (try_active_vote → match → drop); the 352-byte variant is
  // never stored in a collection. Boxing would add a deref-move footgun at the 2
  // consume sites for zero real-world gain. Upgrade to Box if it ever lands in a Vec.
  #[expect(clippy::large_enum_variant)]
  pub enum ActiveVoteResult {
    /// Case is in a vote-accepting state; proceed with tally logic.
    Active(GovernanceCase<Active>),
    /// Case is in a terminal state — caller should return `Ok(case_decided:true)`.
    AlreadyDecided,
  }

  impl GovernanceCase<Active> {
    /// Success-preserving entry-point for site 6.
    /// Returns `AlreadyDecided` (not `Err`) on terminal states so the caller
    /// can map it to `Ok(SubmitJuryVoteResponse { case_decided: true, .. })`.
    pub fn try_active_vote(c: ModerationCase) -> ActiveVoteResult {
      match c.status {
        CaseStatus::Open
        | CaseStatus::ThresholdMet
        | CaseStatus::JurySelection
        | CaseStatus::InReview => ActiveVoteResult::Active(Self::new(c)),
        CaseStatus::Decided
        | CaseStatus::Closed
        | CaseStatus::Appealed
        | CaseStatus::EmergencyRemove
        | CaseStatus::AdminReview
        | CaseStatus::SponsorLiabilityPending
        | CaseStatus::SponsorLiabilityFired
        | CaseStatus::SponsorLiabilityEscaped => ActiveVoteResult::AlreadyDecided,
      }
    }
  }
}

#[cfg(feature = "full")]
pub use inner::{
  Active, ActiveVoteResult, Appealed, GovernanceCase, JurySelection,
  NotYetClosed, PreJuryAssignable, SponsorLiabilityPending,
};
