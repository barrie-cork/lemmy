# [role:impl-task] governance-v0 bug fix: semantic correctness fixes #188 #189 #190

## 1. Role + dispatch

`[role:impl-task]` Fix three medium-severity governance semantic-correctness bugs (GH #188, #189, #190) on `governance-v0`.

**Base branch:** `governance-v0`  
**Worker branch:** derived from base branch HEAD by the daemon.

**IMPORTANT:** This brief must not be dispatched until the companion `governance-v0-fix-impl-high-1` task (fixes #185–#187) has completed and its worker branch has been merged back to `governance-v0`. Both sets of fixes touch `admin_assign_jury.rs` and `decline_jury_assignment.rs` — concurrent workers on the same files will conflict.

---

## 2. Scope

### What to produce

Three targeted fixes — one per issue — committed as three separate commits on the worker branch. No new features, no refactors beyond the minimal corrections.

### Fix D — Issue #188: reconcile `panel_has_sponsor_majority_cluster` with `shares_active_sponsor` (`jury_common.rs`)

**File:** `crates/api/api/src/governance/jury_common.rs`

**Problem:** The two helpers encode different cluster definitions:
- `shares_active_sponsor` (L32–53): two-member pairwise check — "do A and B share any common upstream sponsor?" (self-join on `surety.sponsor_id`).
- `panel_has_sponsor_majority_cluster` (L67–94): groups by `sponsor_id`, counts `COUNT(DISTINCT sponsored_id)` — counts how many panel members each sponsor *directly* sponsors, then checks if any one sponsor's direct-sponsoree count ≥ majority.

These are not the same relation. A panel member who is themselves a sponsor of others is counted differently; two members who share an upstream sponsor but are not directly sponsored by the *same* sponsor will be caught by `shares_active_sponsor` but not by the majority query.

**Fix:** The PRD §5.1 constraint is "no majority from the same sponsor cluster." The `shares_active_sponsor` pairwise definition (shared upstream sponsor) is the canonical one. Rewrite `panel_has_sponsor_majority_cluster` to be consistent with it:

The majority check should count the largest cluster of panel members who share ANY common sponsor (the same self-join semantics as `shares_active_sponsor` but across all N members of the panel, not just pairs). One correct SQL rewrite:

```sql
SELECT COALESCE(MAX(c), 0) AS max_shared FROM (
  SELECT s1.sponsor_id, COUNT(DISTINCT s1.sponsored_id) AS c
  FROM surety s1
  JOIN surety s2 ON s1.sponsor_id = s2.sponsor_id
  WHERE s1.sponsored_id = ANY($1)
    AND s2.sponsored_id = ANY($1)
    AND s1.revoked_at IS NULL
    AND s2.revoked_at IS NULL
  GROUP BY s1.sponsor_id
) t
```

This counts, for each sponsor, how many panel members share that sponsor as a common upstream — matching the pairwise semantics. Replace the current SQL in `panel_has_sponsor_majority_cluster` with this.

If the Diesel `sql_query` API or the existing `ClusterCountRow` shape requires adjustment, make the minimal change. The `ClusterCountRow` struct and bind types should be compatible since the column alias stays `max_shared` and the bind stays `Array<Integer>`.

Add a doc comment to `panel_has_sponsor_majority_cluster` explaining the fixed semantics and citing `shares_active_sponsor` as the canonical single-pair reference.

### Fix E — Issue #189: `geographic_diversity_score` denominator (`admin_assign_jury.rs`)

**File:** `crates/api/api/src/governance/admin_assign_jury.rs`

**Problem:** `geographic_diversity_score` at L909–927 computes:
```
COUNT(DISTINCT mc.community_id) / sample.len()
```
The numerator uses `INNER JOIN jury_assignment ... moderation_case` — only jurors with prior assignment history contribute to `distinct_count`. But the denominator is the full `sample.len()`. A panel where only 2/5 have history is capped at 0.4 even if those 2 served entirely distinct communities, biasing the tiebreaker toward panels with more experienced jurors rather than more geographically diverse ones.

**Fix:** Change the denominator to the count of jurors that actually have history. Rewrite the SQL to also return the count of jurors with at least one prior assignment:

```sql
SELECT COUNT(DISTINCT mc.community_id) AS distinct_count,
       COUNT(DISTINCT ja.person_id) AS history_count
FROM jury_assignment ja
INNER JOIN moderation_case mc ON mc.id = ja.case_id
WHERE ja.person_id = ANY($1)
  AND mc.community_id IS NOT NULL
```

Update `DistinctCommunityRow` (or add a second field `history_count: i64`) and divide by `history_count` instead of `sample.len()`. Guard against division by zero: if `history_count == 0` (none of the sample has history), return `0.0` rather than panicking.

Update the `#[expect(clippy::as_conversions, clippy::cast_precision_loss, ...)]` attribute reason comment to reflect the new denominator.

### Fix F — Issue #190: suppress duplicate relaxation audit rows on replacement/appeal (`decline_jury_assignment.rs` + `admin_assign_jury.rs`)

**Files:**
- `crates/api/api/src/governance/decline_jury_assignment.rs` (replacement path, L157)
- `crates/api/api/src/governance/admin_assign_jury.rs` (`select_appeal_panel`, L1139)

**Problem:** Both the decline-replacement pick and the appeal-panel assembly call `select_eligible_jurors`, which re-runs the full R1→R3 relaxation cascade AND re-writes the relaxation audit (`write_constraint_relaxation`) on every invocation. On a small instance, every decline emits fresh R1/R2/R3 rows even though the constraint was already relaxed and recorded at original panel assembly — inflating the hash chain with duplicate relaxation events.

**Fix:** Use approach (b) from the issue — record replacement-context relaxations under a distinct reason marker so the audit timeline doesn't read as repeated independent relaxations. The `JuryConstraintRelaxationReason` enum already has `SmallPool`, `ClusterPressure`, `ClusterPressureExhausted`, `AdminOverride` variants. The fix does NOT require a new enum variant; instead, pass context metadata to `write_constraint_relaxation` to distinguish replacement/appeal context.

Concretely: `write_constraint_relaxation` already takes a `metadata: Value` argument. When called from the replacement/appeal code path (via `select_eligible_jurors`), add a `"context": "replacement"` or `"context": "appeal"` key to the metadata JSON so audit readers can distinguish the call site. The audit log row is still written (required for hash-chain integrity), but it is clearly tagged as a replacement/appeal re-relaxation, not a fresh independent relaxation.

Implementation: `select_eligible_jurors` doesn't currently accept a "call-site context" argument. Simplest approach: add an optional `context: Option<&'static str>` parameter to `select_eligible_jurors` (defaulting to `None` for original assembly), and thread it through to `write_constraint_relaxation`'s metadata `json!` objects. Where callers pass `None`, the existing behaviour is unchanged. Where callers pass `Some("replacement")` or `Some("appeal")`, the metadata gains a `"context"` field.

Update call sites:
- `decline_jury_assignment.rs` L157: pass `Some("replacement")`.
- `select_appeal_panel` in `admin_assign_jury.rs` L1139: pass `Some("appeal")`.
- Original assembly in `admin_assign_jury.rs` (the `process_assignment` path, L199): pass `None` (no change to existing behaviour).

If adding a parameter to `select_eligible_jurors` causes compile errors at other call sites, update them all to pass `None`.

### What NOT to produce

- No new migrations.
- No new entry kinds in `governance_log.rs`.
- No new `JuryConstraintRelaxationReason` enum variants.
- No changes to e2e tests (the semantic correctness issues are unit-testable but not in scope here — fix the logic).
- No changes to any file outside the four listed above.

---

## 3. Required reading

- `crates/api/api/src/governance/jury_common.rs` L30–95 — full bodies of `shares_active_sponsor` and `panel_has_sponsor_majority_cluster` before touching.
- `crates/api/api/src/governance/admin_assign_jury.rs` L899–984 — `geographic_diversity_score` + `write_constraint_relaxation` bodies.
- `crates/api/api/src/governance/admin_assign_jury.rs` L467–760 — full `select_eligible_jurors` body (understand all call sites before adding a parameter).
- `crates/api/api/src/governance/admin_assign_jury.rs` L1076–1170 — `select_appeal_panel` body.
- `crates/api/api/src/governance/decline_jury_assignment.rs` L139–196 — replacement-selection block.
- `.claude/lessons/feedback_governance_type_state_handlers.md` — type-state pattern (context for `select_eligible_jurors` call sites).

---

## 4. Constraints

1. **Three separate commits** — one per fix, in order D, E, F. Commit subjects: `fix(governance): reconcile panel_has_sponsor_majority_cluster semantics — closes #188`, `fix(governance): geographic_diversity_score denominator — closes #189`, `fix(governance): tag replacement/appeal relaxation audit context — closes #190`.
2. **DoD per commit:** after each commit run `cargo check --workspace --features full` (via `./scripts/brehon/cargo-check.sh --workspace --features full`). All three must exit 0.
3. **Write a `validate-pending-laptop` DQ entry** after the third commit with `commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`, `branch: <worker branch name>`, `phase_task: "governance-fix-med-1"`. Commit + push the DQ entry, then **stop**. Do NOT run cargo-check yourself after writing the DQ.
4. **No `answered_by: "advisor"` in DQ entries** from this worker session.
5. **Fix D SQL change**: the self-join adds `s2.sponsored_id = ANY($1)` — the bind receives the same array bound twice. Check if Diesel `sql_query` supports `$1` referenced twice in the same query; if not, use a subquery or a CTE to avoid the double-bind constraint.
6. **Fix F `select_eligible_jurors` signature change**: if adding `context: Option<&'static str>` triggers more than 5 call-site updates, stop and raise a `kind: "blocker"` DQ with the count before proceeding.
