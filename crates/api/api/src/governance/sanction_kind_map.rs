use lemmy_db_schema_file::enums::{SanctionAction, SanctionKind};

/// Canonical v0 mapping from the legal SanctionAction taxonomy to the
/// platform-neutral SanctionKind published over B-publish. Exhaustive match
/// (no `_ =>`) so a future SanctionAction variant forces an explicit decision.
/// `None` ⇒ no local Matrix primitive ⇒ skip delivery (not an error).
pub fn map_sanction_action(action: SanctionAction) -> Option<SanctionKind> {
  match action {
    SanctionAction::Label => Some(SanctionKind::RestrictReach),
    SanctionAction::VisibilityReduction => Some(SanctionKind::RestrictReach),
    SanctionAction::TemporaryRestriction => Some(SanctionKind::PreventPost),
    SanctionAction::ContentRemoval => Some(SanctionKind::HideContent),
    SanctionAction::CommunityExclusion => Some(SanctionKind::PreventPost),
    SanctionAction::InstanceSuspension => Some(SanctionKind::PreventPost),
    SanctionAction::FederationQuarantineRecommendation => None,
    SanctionAction::Restoration => None,
  }
}
