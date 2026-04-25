---
relay_id: pr95-cr-3-emergency-remove-cascade
from: advisor
to: impl
worktree: C:/Users/barri/Developer/brehon-fork-phase-v1-JM-b
branch: phase-v1-JM-b
pr: 95
finding: cr-3
finding_severity: major
finding_url: https://github.com/barrie-cork/lemmy/pull/95#discussion_r3141635548
file_target: crates/api/api/src/governance/admin_emergency_remove.rs
plan_ref: .claude/PRPs/plans/v1-jury-mechanics-b.plan.md §10.4 §10.5 §10.6
adr_ref: ADR-013
status: open
posted_at: 2026-04-25T01:30:00Z
---

# CR finding cr-3 — emergency-remove must mirror normal-assignment cascade

## TL;DR

`admin_emergency_remove::process_emergency_remove` reads the bare
`jury.panel_size` config key via `config::get_int(...Scope::Instance,
"jury.panel_size")` and never freezes the snapshot fields on the case row
or emits a `severity_tier_frozen` governance_log entry. JM-b's
`process_assignment` does both. Per ADR-013 ("admin power exists but is
auditable and constrained by post-facto review"), the emergency-remove
case must be auditable to the *same standard* as a normal-jury case —
otherwise `panel_size`, `quorum_fraction`, `threshold_fraction` can drift
between case-open and the post-facto jury review when admins reconfigure
mid-flight, breaking the audit trail.

This relay closes CR finding cr-3. **Fast-merge gate:** PR #95 cannot
merge until this lands. Other 8 findings are carry-forward (cr-1/2/6/7
lints, cr-4/5/8 cluster) or wont-fix (cr-9 PR-body template). Triage at
`.claude/PRPs/reviews/pr-95-findings.yaml` (gitignored).

## What to do — file:line refs

### Step 1 — Read the canonical pattern (don't reinvent)

`crates/api/api/src/governance/admin_assign_jury.rs::process_assignment`
is the canonical cascade-aware shape. Lines 134–230 show the exact
sequence. Key structural elements to mirror:

- **Line 134** — `let severity = case.severity_tier;` (already typed
  `SeverityTier`; emergency-remove case has `severity_tier:
  Some(SeverityTier::Severe)` set at insert time per `admin_emergency_remove.rs:154`)
- **Line 141** — `let severity_str = severity_tier_slug(severity);`
  (call `admin_assign_jury::severity_tier_slug` — already `pub(crate)`
  since JM-b task 5 made it module-visible; if not, expose it)
- **Lines ~155–198** — cascade reads (panel_size + quorum + threshold)
  through `config::get_int_cascade` / `get_float_cascade` with the
  status-tier prefix. Emergency-remove cases have `target_person_id:
  None` (see admin_emergency_remove.rs:149) so the status tier
  resolves to `CaseStatusTier::Regular` per `compute_status_tier` line
  333. **You can hardcode `CaseStatusTier::Regular` for emergency-remove
  rather than calling `compute_status_tier` — saves an async DB call
  and the result is deterministic.**
- **Lines 199–207** — snapshot UPDATE: `panel_size_snapshot`,
  `quorum_fraction_snapshot`, `threshold_fraction_snapshot`,
  `status_tier_snapshot`, `severity_tier_snapshot` written onto the
  case row in one tuple
- **Lines 208–230** — `severity_tier_frozen` governance_log emission
  with admin_pseudonym actor + the same snapshot fields in payload

### Step 2 — Replace the bare config read at admin_emergency_remove.rs:177–186

Current code (`admin_emergency_remove.rs:177–186`):

```rust
let mut cache = ConfigCache::new();
let panel_size = config::get_int(
  &mut cache,
  &mut (&mut *conn).into(),
  Scope::Instance,
  "jury.panel_size",
)
.await?;
let (eligible, _record) =
  admin_assign_jury::select_eligible_jurors(conn, &case_row, panel_size, None, &mut cache).await?;
```

Replace with the cascade-aware shape. Sketch (adapt to actual helper
signatures — verify against admin_assign_jury.rs):

```rust
// Per ADR-013: emergency-remove case must be auditable to the same
// standard as a normal-jury case. Mirror process_assignment §10.4-§10.6:
// cascade-read panel/quorum/threshold, freeze snapshot, emit
// severity_tier_frozen.

let mut cache = ConfigCache::new();
let severity = case_row.severity_tier;        // = Some(SeverityTier::Severe)
let severity_for_cascade = severity.unwrap_or(SeverityTier::Minor); // defensive
let severity_str = admin_assign_jury::severity_tier_slug(severity_for_cascade);
let status_tier = CaseStatusTier::Regular;    // emergency-remove has target_person_id = None
let status_str = admin_assign_jury::status_tier_slug(status_tier);

// Cascade read panel_size: jury.panel_size.<status>.<severity> → … → jury.panel_size
let panel_size_i32 = admin_assign_jury::cascade_panel_size(
  &mut cache, &mut (&mut *conn).into(), status_str, severity_str
).await?;

// (similar cascade reads for quorum_fraction + threshold_fraction —
// see admin_assign_jury.rs lines ~165-198 for the exact signatures
// and snapshot-tuple shape)

// 6'. Snapshot onto case row (mirror admin_assign_jury.rs:199-207)
update(moderation_case::table.find(case_id))
  .set((
    moderation_case::panel_size_snapshot.eq(Some(panel_size_i32)),
    moderation_case::quorum_fraction_snapshot.eq(...),
    moderation_case::threshold_fraction_snapshot.eq(...),
    moderation_case::status_tier_snapshot.eq(Some(status_str)),
    moderation_case::severity_tier_snapshot.eq(severity),
  ))
  .execute(conn).await?;

// 7'. severity_tier_frozen governance_log emission (mirror
// admin_assign_jury.rs:208-230). Admin is the actor.
governance_log::append(
  &mut conn.into(),
  ENTRY_KIND_SEVERITY_TIER_FROZEN,
  json!({
    "case_id": case_id.0,
    "severity_tier": severity_str,
    "status_tier": status_str,
    "panel_size_snapshot": panel_size_i32,
    "quorum_fraction_snapshot": ...,
    "threshold_fraction_snapshot": ...,
  }),
  Some(admin_pseudonym.clone()),  // emergency-remove admin = actor
).await?;

// THEN call select_eligible_jurors with the FROZEN panel_size:
let (eligible, _record) =
  admin_assign_jury::select_eligible_jurors(conn, &case_row, panel_size_i32, None, &mut cache).await?;
```

### Step 3 — DO NOT change cr-4 in this commit

cr-4 (the `selected_under_constraints: None` on the JuryAssignmentInsertForm
at line 194) is bucketed `carry-forward`. Do not touch it now — it bleeds
into the same file but addressing it requires propagating the
`ConstraintRecord` through the `forms.iter().map(...)` collect, which is
mechanical but separate scope. Carry-forward issue will batch cr-4 + cr-8
(decline_jury_assignment.rs:164) + cr-5 (config.rs candidate fallback)
post-merge.

If you find yourself touching the `forms: Vec<JuryAssignmentInsertForm>`
construction at line 188, **stop** — that's cr-4 territory. Only touch
the cascade-snapshot block above line 188.

### Step 4 — Validate

```
/cargo-validate check -p lemmy_api --features full
/cargo-validate clippy --workspace --features full --no-deps -- -D warnings
/cargo-validate test --no-run -p lemmy_server --test e2e
```

The Task 8 e2e assertion `admin_emergency_remove sets severity_tier =
Severe` (added at `fcd2b1890`) should still pass. If it now fails because
of the new severity_tier_frozen emission, that's a fixture-setup issue —
the test predates the cascade requirement; either extend the assertion
to also check the snapshot fields, OR leave the test alone and the new
governance_log entry is asserted by a follow-up test in the carry-forward
batch.

If the e2e suite shows new failures, surface back via DQ — don't push.

### Step 5 — Commit + push

Suggested commit subject (single commit on `phase-v1-JM-b`):

```
fix(v1-JM-b): cr-3 — emergency-remove mirrors normal-assignment cascade (ADR-013 audit trail)
```

Body should cite:
- ADR-013 (audit-trail discipline for emergency-remove)
- The mirrored block at `admin_assign_jury.rs:134-230`
- That cr-4/cr-5/cr-8 are carry-forward (do not bundle)

After commit lands, BM session re-polls CR (`/bm-poll-cr 95`), promotes
cr-3 fix-in-pr → done in the YAML, then opens the merge gate
(`/bm-merge 95` — confirms first).

## What this relay does NOT ask you to do

- Fix cr-4 (drop the `None`) — carry-forward
- Fix cr-5 (config.rs fallback) — carry-forward
- Fix cr-8 (decline_jury_assignment.rs replacement) — carry-forward
- Edit PR #95 body — cr-9 wont-fix
- Touch the existing Task 8 e2e test unless it actually breaks
- Add new tests — defer to carry-forward batch

## ADR / Plan / Memory references

- **ADR-013** — `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md:226-239` ("Emergency-remove is visible in the public case log; admin power... constrained by post-facto review")
- **Plan §10.4** — cascade keys for jury.panel_size + jury.quorum_fraction + jury.threshold_fraction
- **Plan §10.5** — single-UPDATE snapshot pattern
- **Plan §10.6** — severity_tier_frozen governance_log payload shape
- **Memory `feedback_advisor_cr_enum_drift.md`** — relay must cite enum/symbol names verified against current source. Verified above: `SeverityTier`, `CaseStatusTier`, `severity_tier_slug`, `status_tier_slug`, `compute_status_tier`, `ENTRY_KIND_SEVERITY_TIER_FROZEN` — all confirmed at admin_assign_jury.rs lines 60, 134, 141, 328, 380.

## Worktree state at relay write

- `phase-v1-JM-b` HEAD: `20c4411b0` (synced with origin/phase-v1-JM-b)
- Working tree: 3 untracked impl-side handovers/audit-json (all gitignored or supersedable)
- No conflict expected — impl edits a single file in a discrete block

## Closing

If you uncover plan-drift while implementing (e.g., `cascade_panel_size`
helper signature differs from sketched), STOP and surface via the same
relay file (append "## impl response 2026-04-25T<HH:MM>Z" — don't spawn
a new file). Advisor watches `.claude/runlog/advisor-relays/` and the
JM-b worktree DQ for DQ #50+.

Per `feedback_advisor_cr_enum_drift.md`: trust enum/symbol names in this
relay; they were verified against `admin_assign_jury.rs` HEAD at write
time. If you find they've drifted (mid-impl rebase, etc.), that's a
plan-drift signal — surface, don't propagate.
