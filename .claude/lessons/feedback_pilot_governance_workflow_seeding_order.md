---
name: pilot-governance-workflow-seeding-order
description: How to seed pilot test data in the right order to exercise specific Brehon governance workflows live (appeals, jury, sanctions) — which API path fires the bridge hook, the reputation_snapshot community_id scope trap, and the "fresh case → drive to state → trigger the hook-firing transition" recipe.
type: feedback
---

# Pilot governance workflow testing — seeding order + which path fires the hook

Hard-won during pilot-internal phase-3 (appeal-room provisioning). When you
test a *specific* governance workflow live against the pilot, the failure mode
is not "the code is broken" — it's "I exercised the wrong path / seeded the
wrong data / wrong scope, so the thing I wanted to observe never fired." These
are the traps, in the order they bite.

## 1. Find which API path actually fires the bridge hook — handlers differ

The Matrix bridge room-event POST fires from `governance_case_after_transition`
(`crates/api/api_utils/src/bridge_notify.rs`), which is called at **specific
callsites on specific transitions** — NOT from every handler that touches a case.

**Worked example (appeals):** to provision an appeal room you must drive the
`Decided → Appealed` transition, and the hook is called from `request_appeal`
(`POST /governance/appeal`) at the END of the handler (after the txn commits).
**`admin_trigger_appeal_rejury` (`POST /admin/trigger-appeal-rejury`) does NOT
fire the hook** — it only seats the appeal panel. Re-triggering rejury on an
already-`Appealed` case will seat jurors but provision NO room. The auto-seat
path (`/governance/appeal` with `appeal.auto_select_on_appeal_acceptance=true`,
the live pilot default) BOTH seats the panel AND fires the hook in one call —
that's the path the room-provisioning code lives on.

**Rule:** before testing any room/bridge-effect workflow, `grep -rn
'governance_case_after_transition' crates/` to find the exact callsites, then
read the handler to confirm WHICH route triggers the transition you need.
Drive that route, not a sibling that merely mutates the same case. The callsites
(as of m2/pilot): `admin_assign_jury` (JurySelection), `submit_jury_vote`
(Decided/SponsorLiabilityPending/AdminReview), `request_appeal` (Appealed),
`admin_close_case` (Closed), `admin_emergency_remove` (EmergencyRemove),
`appeal_window_expiry`, `sponsor_liability_grace`, `revoke_endorsement`.

## 2. Each room-provisioning transition needs ITS OWN round's pseudonyms

The bridge enforces ADR-015 by skipping any room whose `juror_pseudonyms` is
empty. The hook must therefore fetch the pseudonyms for the round it is
provisioning: `JurySelection` → the `Original` panel; `Appealed` → the `Appeal`
panel (filter `jury_assignment::role`). The original and appeal panels coexist
as separate `jury_assignment` rows on the same case, so an unfiltered fetch
seeds the wrong room with the wrong jurors. (This was the phase-3 fix —
`bridge_notify.rs` only fetched on `JurySelection`; appeals sent an empty vec
and the bridge silently skipped.)

**Testing tell:** in the bridge logs, the negative signal
`juror_pseudonyms is empty on <transition> — skipping ... provision (ADR-012)`
means "panel was empty OR the hook didn't fetch this round." Distinguish the two
by checking the `jury_assignment` rows for that `role`. A non-empty panel + the
skip WARN = a hook fetch bug. An empty panel + the skip WARN = a seeding problem
(go to §3).

## 3. Jury-eligible seeding — the community_id scope trap (the big one)

"Jury-eligible" = a `reputation_snapshot` row with `jury_eligible=true` for the
person. But the eligibility query (`admin_assign_jury.rs`, `select_*_panel` →
`run_extended_eligibility_query`) joins
`reputation_snapshot.community_id IS NOT DISTINCT FROM <case.community_id>`.

- If the case is **community-scoped** (e.g. pilot cases are scoped to
  community 2), an **instance-scoped (`community_id=NULL`) snapshot does NOT
  match** — `NULL IS NOT DISTINCT FROM 2` is false. The juror will be invisible
  to the selector and the panel seats short/empty even though `jury_eligible=t`.
- **Seed the snapshot with `community_id = <the case's community_id>`**, not
  NULL. Check the target case first: `SELECT id, community_id FROM
  moderation_case WHERE id=<N>;`.

Reproducible INSERT (pilot, community 2):
```sql
INSERT INTO reputation_snapshot
  (person_id, community_id, reporting_accuracy, jury_reliability,
   participation_consistency, endorsement_strength, jury_eligible,
   trusted_reporter, can_sponsor)
VALUES (<pid>, 2, 100, 100, 100, 100, true, false, false);
-- id (sequence) + calculated_at (default now()) are left to defaults.
```
Verify by running the actual eligibility query (not just `SELECT … WHERE
jury_eligible=true`) — it's the only check that proves the join matches.

**Pilot gotcha:** the original juror1–5 accounts are NOT strictly eligible —
they seat via the small-pool *relaxed fallback* (`jury_constraint_relaxed`
logged). So "5 jurors served" does not mean "5 eligible jurors exist." Strict
eligibility requires the snapshot rows above.

## 4. Panels consume their pool — seed enough for the EXCLUSION

The appeal selector **excludes the original panel** and uses a **higher
threshold tier** (`appeal.threshold_tier_bump`, e.g. quorum 3 → 5). So to seat a
full appeal panel you need `appeal_threshold` eligible jurors who did NOT serve
on the original panel. If the original panel happened to grab some of your
freshly-seeded eligibles, the appeal pool shrinks. **Seed generously** — for an
appeal-threshold-5 test, ≥5 fresh eligibles beyond whatever the original panel
might consume (the relaxed fallback will fill any remainder, but a fully-strict
appeal panel needs the headroom).

**Empty-panel footgun:** both `request_appeal` and `admin_trigger_appeal_rejury`
return HTTP 200 while seating an EMPTY panel (no "insufficient jurors" error).
An empty `appeal_panel_assembled` log entry + a `panel_person_ids:[]` response is
the signal you under-seeded. (Whether empty-panel-as-silent-no-op is correct is a
design question, flagged for impl — not a test bug.)

## 5. The general recipe — "fresh case → drive to state → fire the hook"

1. **Use a fresh case.** Spent cases are in a terminal/post-transition state;
   you cannot re-fire a transition that already happened. (Case 5 was already
   `Appealed` — re-triggering rejury would not re-fire the `Appealed` hook.)
2. **Drive it to the entry state** for your workflow via the proven happy path
   (post → report → assign-jury → accept → vote → quorum → `Decided`).
3. **Trigger the specific transition that fires the hook** (§1) — and ensure the
   round's panel is seeded + eligible (§3, §4) so the hook carries a non-empty
   list (§2).
4. **Verify three independent surfaces**, not one: the DB (`moderation_case`
   status, `jury_assignment` rows, `governance_log` chain), the bridge
   (`bridge_room` row + bridge logs for the provision/invite lines, NOT the
   empty-skip WARN), and Matrix itself (Tuwunel room-alias resolve / room-state
   read). A single surface can lie; the cross-check is the proof.

## Cross-refs

- `[[pilot-governance-workflow-seeding-order]]` is the seeding/order companion to
  the per-account creds in `reference_pilot_test_accounts.md` and the rate-limit
  buckets in `reference_lemmy_rate_limit_buckets_pilot.md`.
- e2e fixture canonical shape: `crates/server/tests/e2e/jury_mechanics.rs`
  `seed_jury_eligible_snapshots_scoped` (the `community_id` scope note there is
  the same trap, in test form).
- Run live governance test phases via a subagent — the SSH/curl/SQL/bridge-log
  output is large and belongs out of the main session context (advisor §6.3).
