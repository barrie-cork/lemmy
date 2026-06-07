# Brief: m2-late-1 Task 3 — governance-log consts + api shim + exhaustive `SanctionKind` map

## 1. Role + dispatch

`[role:impl-task] m2-late-1-task-3-consts-map — see .claude/PRPs/briefs/m2-late-1-impl-3.md`

Pre-Shape-G plan. Workers write `validate-pending-laptop` DQ and STOP; do NOT run workspace cargo on the daemon.

## 2. Scope

**Produce:**

1. **Modify** `crates/db_schema/src/source/governance/governance_log.rs` — append 2 new `ENTRY_KIND_*` consts after the existing m2-core-hook room kinds block (≈ after line 144 where the m2-room consts end and before the `append` function):
   - `pub const ENTRY_KIND_SANCTION_PUBLISHED: &str = "sanction_published";`
   - `pub const ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED: &str = "sanction_event_delivery_failed";`

2. **Modify** `crates/api/api/src/governance/governance_log.rs` — add both consts to the `pub use` block in alphabetical order:
   - `ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED` goes between `ENTRY_KIND_SANCTION_CREATED` and `ENTRY_KIND_SANCTION_PUBLISHED` — wait: alphabetically: `..._SANCTION_CREATED` < `..._SANCTION_EVENT_DELIVERY_FAILED` < `..._SANCTION_PUBLISHED`. Insert both in that position relative to `..._SANCTION_CREATED`.

3. **Create** `crates/api/api/src/governance/sanction_kind_map.rs` — exhaustive `SanctionAction → Option<SanctionKind>` map with NO wildcard.

4. **Modify** `crates/api/api/src/governance/mod.rs` — add `pub mod sanction_kind_map;` (alphabetical; between `pub mod sanction_liability_grace;` ... actually between `pub mod sponsor_liability_grace;` and `pub mod submit_jury_vote;`).

Then write the `validate-pending-laptop` DQ entry and STOP.

**Explicit boundaries (do NOT touch):**
- Do NOT edit `crates/db_schema/src/source/governance/mod.rs`, `sanction_event.rs`, `sanction_subscriber.rs`, or `newtypes.rs` — those are T2's files.
- Do NOT edit `crates/db_schema_file/**` — T1 completed.
- Do NOT edit `crates/server/**` or `services/bridge/**`.
- Do NOT edit `crates/server/tests/e2e.rs` — isolated to T8.
- Do NOT run `cargo check --workspace` on the daemon (R7).

## 3. Required reading

**Before your first edit, Read these files:**

- `crates/db_schema/src/source/governance/governance_log.rs:112-144` — existing `ENTRY_KIND_*` const block; note the m2-core-hook room kinds at the end; append the 2 new m2-late consts after them (before the `append` fn or at the end of the const block).
- `crates/api/api/src/governance/governance_log.rs:39-70` — the `pub use` block; must add both new consts in alphabetical position within the existing list.
- `crates/api/api/src/governance/mod.rs` — confirm current module list; `pub mod sanction_kind_map;` goes alphabetically.
- `crates/db_schema_file/src/enums.rs:531-577` — `SanctionScope` (derive mirror) + `SanctionAction` 8 variants (the exhaustive map input).
- `.claude/PRPs/plans/m2-late.plan.md` §10.2 — canonical exhaustive map specification.
- `.claude/PRPs/plans/m2-late.plan.md` §10.6 — consts + shim pattern.
- `.claude/rules/governance-log-entry-kind-registry.md` — READ ONLY; the registry section for m2-late (65→67) is advisor meta-work. Junior adds ONLY the 2 `crates/` `governance_log.rs` files (const + shim). Leave a NOTE at the bottom of your impl commit body: `NOTE: advisor must update .claude/rules/governance-log-entry-kind-registry.md (65→67) as WP-7 meta-work.`
- `feedback_validate_pending_laptop_write_then_stop.md` — STOP after DQ entry.

## 3a. Handover from prior cohort

T1 completed on `phase-m2-late-1` at SHA `298f7ed77`. DQ `001f1c47c5dc-001` resolved `result: pass`. (Same as T2 brief §3a — T2 and T3 run in parallel from the same phase tip.)

```yaml
prior_cohort_tasks:
  - task: 1
    commit: 298f7ed77
    filesModified:
      - crates/db_schema_file/src/enums.rs   # SanctionKind enum added
      - crates/db_schema_file/src/schema.rs  # hand-extended: sql_types::SanctionKind + 2 tables
    keyDecisions:
      - SanctionKind has 4 variants: PreventPost, MuteVoice, HideContent, RestrictReach
      - SanctionAction has 8 variants in enums.rs:560-577 (the map source)
    notes: T3's exhaustive map uses SanctionKind from enums.rs (T1 output). Read enums.rs:531-577 to confirm both enum definitions before authoring sanction_kind_map.rs.
```

## 4. Constraints

- **Exhaustive map — NO wildcard:** `match action { ... }` over `SanctionAction` MUST enumerate all 8 variants explicitly. No `_ =>` arm. This is the primary correctness invariant: a future 9th `SanctionAction` variant must fail to compile until mapped.
- **`MuteVoice` is intentionally NOT produced** by any v0 `SanctionAction` mapping. It is a valid `SanctionKind` variant for bridge translation but no v0 input action maps to it. This is NOT a bug — see plan §19. The exhaustive map covers all 8 `SanctionAction` inputs; `MuteVoice` is a valid `SanctionKind` output not selected in v0.
- **R7:** Do NOT run `./scripts/brehon/cargo-check.sh` or any `cargo` command on the daemon. Write `validate-pending-laptop` DQ then STOP.
- **`[P]` with T2:** T3 and T2 are parallel. T3 owns governance_log.rs consts + api shim + sanction_kind_map.rs + api mod.rs. T2 owns db_schema models + db_schema mod.rs + newtypes.rs. File-disjoint — do NOT touch T2's files.
- **Registry is advisor meta-work (WP-7):** Do NOT edit `.claude/rules/governance-log-entry-kind-registry.md`. Leave the NOTE in your commit body (see §3 above).
- **`pub use` alphabetical ordering:** The two new consts (`ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED`, `ENTRY_KIND_SANCTION_PUBLISHED`) insert alphabetically AFTER `ENTRY_KIND_SANCTION_CREATED` and BEFORE `ENTRY_KIND_SANCTION_PUBLISHED` > `..._SEVERITY_TIER_FROZEN`. Verify position by reading the existing list.
- **DQ commit subject:** `chore(decision-queue): impl raised validate-pending-laptop for m2-late-1 task-3`.
- **Impl commit subject:** `feat(db_schema,api): add sanction_published/delivery_failed consts + shim + exhaustive SanctionKind map (task 3)`.
- One impl commit + DQ commit then STOP. Push both commits to `origin phase-m2-late-1` immediately after each.

### Exact map (verbatim from plan §10.2):

```rust
// crates/api/api/src/governance/sanction_kind_map.rs
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
```

**Mandatory lesson fired (file-class table match):**
- `feedback_validate_pending_laptop_write_then_stop.md` — pre-Shape-G plan, any `crates/api/**` edit.
