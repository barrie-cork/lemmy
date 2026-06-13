---
phase: pilot-internal (testing track — forward plan)
plan: "(none — ops/testing track, no code commits from the testing session)"
phase_branch: "(none)"
lane_mode: B
worktree: C:/Users/barri/Developer/brehon-fork
authored: 2026-06-13
authored_by: infra/code session (canonical brehon-fork / governance-v0)
purpose: The multi-phase testing roadmap for pilot-internal. Phases 1+2 are DONE; this file outlines phases 3–7 (the remaining governance surface + the actual human-pilot go-live). The testing session owns execution; infra/code owns any wiring gaps each phase surfaces.
---

# Pilot-internal testing plan — forward phases

> **Where we are (2026-06-13):** Phases **1 (DB/API/bridge-receipt)** and **2 (Matrix room provisioning + sanction power-levels)** are COMPLETE and independently confirmed (SHARED-STATE §4 — cases 1–5; `m.room.power_levels` 0→-1 verified in Tuwunel state; hash chain `CHAIN_INTACT` across all cases). What's been exercised is the **happy-path jury flow with one sanction kind** (`HideContent`/mute). The phases below cover the rest of the governance surface and the actual point of the pilot — real humans using it.

## How to use this plan

- The **testing session** runs these; **infra/code** fixes wiring gaps each phase surfaces (expect some — the appeal/emergency/sanction-kind paths have the same "never deployed" exposure the jury path had: 7 latent bridge bugs surfaced in phases 1–2).
- Each phase has an **entry gate**, **steps**, **CRITICAL VERIFY** (the pass/fail signal), and **likely-gap** notes (what may not be wired).
- Coordinate via `pilot-internal-SHARED-STATE.md` — append §4 entries, raise wiring needs in §5.
- Use **fresh cases** (case 6+; cases 1–5 spent). Re-login JWTs as needed (`reference_pilot_test_accounts.md`).
- **Do NOT** restart/recreate the `brehon-bridge` / `brehon-tuwunel` / `docker-lemmy-1` containers — infra's lane.

---

## Phase 3 — Appeal flow + appeal-room provisioning

**Why:** appeals are a distinct lifecycle (`Decided` → `Appealed` → appeal panel → `appeal_decided`) with their OWN room type (`appeal-case-N`). Never exercised live. Tests m2 appeal-room provisioning + the higher appeal quorum tier.

**Entry gate:** phase 2 done (✓). A `Decided` case in its appeal window (case 5 is `Decided` — check `appeal_window_expires_at` not yet passed, or drive a fresh case to `Decided` first).

**Steps:**
1. On a `Decided` case (the target/defendant account), `POST /api/v4/governance/appeal {case_id}` → case → `Appealed`.
2. Confirm the hook fires the `appealed` transition → bridge `room-event` → **appeal room provisioned** (`appeal-case-N`).
3. `POST /governance/admin/trigger-appeal-rejury` (or the appeal-panel seat path) → new appeal panel (original jurors EXCLUDED, higher threshold tier).
4. Appeal panel: accept → vote → appeal quorum → `appeal_decided`.

**CRITICAL VERIFY:**
- `bridge_room` has a NEW row `(case_id, room_type='appeal', ...)` — appeal rooms are separate from jury rooms.
- Appeal panel excludes the original jurors (check `jury_assignment` for the appeal round vs original).
- `governance_log` has `appeal_panel_assembled` + `appeal_decided` entries; hash chain stays intact.
- The bridge's `provision_appeal_room` path ran (bridge logs).

**Likely gaps:** the appeal-room provisioning path (`provision_appeal_room` in `room_provisioner.rs`) has NEVER run live — expect the same class of bug as jury rooms (it shares `create_community_room` + `ensure_puppet`, both now fixed, so it MAY work first try; the appeal-specific alias `appeal-case-N` is covered by the `#.*-case-.*` namespace fix). Watch for appeal-panel-seating quirks (higher threshold, juror exclusion).

---

## Phase 4 — Emergency removal (ADR-013, latency-critical)

**Why:** `admin_emergency_remove` is the ADR-013 mandatory path — admin removes content immediately, jury reviews post-facto. It provisions an **emergency room** and has a **<2s acceptance target** (m2 acceptance signal). Distinct room type + latency requirement.

**Entry gate:** phase 2 done (✓). A fresh post to emergency-remove.

**Steps:**
1. Fresh post (testuser) in `test_governance`.
2. Admin `POST` the emergency-remove endpoint (find exact route — `admin_emergency_remove`; check `routes/src/lib.rs` for the path) → case → `EmergencyRemove`.
3. Confirm hook fires `emergency_remove` transition → bridge → **emergency room provisioned** (`emergency-case-N`), with the legal-contact MXID (`@legal:localhost`) invited.

**CRITICAL VERIFY:**
- `bridge_room` row `(case_id, room_type='emergency', ...)`.
- **Latency:** time from emergency-remove API call → `bridge_room` row present. m2 target is <2s. Measure it (bridge log timestamp vs API-call timestamp).
- `governance_log` `emergency_removed` entry; content actually removed (post hidden/removed in Lemmy).
- The legal-contact invite landed (emergency rooms invite `LEGAL_CONTACT_MXID` per the provisioner).

**Likely gaps:** `provision_emergency_room` never run live. The `@legal:localhost` invite uses `invite_to_room` (now fixed). The <2s target is the interesting one — fire-and-forget means the API returns fast, but measure the actual room-provisioning latency from logs.

---

## Phase 5 — Sanction-kind coverage (all 4 power-level translations)

**Why:** only `HideContent` (mute) tested. The bridge's `compute_power_override` maps 4 kinds to distinct power-levels + reason codes. Each needs verifying against real Matrix room state.

**SanctionKind → bridge behavior (from `sanction_handler.rs::compute_power_override`):**
| SanctionKind (wire) | Power level set | reason_code |
|---|---|---|
| `prevent_post` | `events_default - 1` (below post threshold) | `power_level_reduced_below_post_threshold` |
| `mute_voice` | (voice-specific PL reduction) | (mute_voice branch) |
| `hide_content` | `events_default - 1` | `redaction_not_available_in_m2_late_2` ← tested (case 5) |
| `restrict_reach` | `events_default - 1` | `restrict_reach_translated_to_power_level_reduction` |

**Entry gate:** phase 2 done (✓). A provisioned jury room per kind (or reuse one room and send different sanction kinds).

**Steps:** for each of `prevent_post`, `mute_voice`, `restrict_reach` (hide_content done):
1. Drive a case to a state with a provisioned room (or POST a sanction-event directly with that `sanction_kind` against an existing `bridge_room` case, as infra did for cases 3/4 — but prefer a real quorum-driven sanction where the kind is selected by the jury decision).
2. Confirm `applied:true` + the expected reason_code.
3. Read the room's `m.room.power_levels` and confirm the subject's PL matches the table.

**CRITICAL VERIFY:** each kind produces its documented power-level + reason_code in actual Tuwunel room state. Note the M2 limitations baked into the reason codes (`redaction_not_available_in_m2_late_2`, ban==power-reduction-not-membership-ban — these are deliberate m2-late-2 scope limits, NOT bugs; confirm they behave as the reason codes claim).

**Likely gaps:** `mute_voice` may have a distinct (non `-1`) level — read the actual branch. None of these paths beyond `hide_content` have run live.

---

## Phase 6 — Adversarial / negative paths

**Why:** phases 1–5 are happy-path. Real governance must handle the unhappy cases. These exercise the v1-JM (jury mechanics) + v1-SL (sponsor liability) logic that's in the binary but unexercised in the pilot.

**Sub-cases (each a fresh case):**
1. **Jury deadlock** — panel votes but no decision meets quorum → case → `AdminReview`, `jury_deadlock` log entry, NO sanction/room-power-change. (v1-JM-c)
2. **Declined juror + replacement** — a juror `POST /jury/decline` → `jury_declined` + `jury_replacement_selected`, replacement seated, room membership updated. (v1-JM)
3. **Non-quorum / insufficient votes** — fewer than quorum vote within the window → confirm no premature decision.
4. **Bad-faith report flag** — admin flags a report as bad-faith → `evidence_quality_recorded` (-1 reporting_accuracy). (v1-RT)
5. **Sponsor liability** (if endorsements are seeded) — a `Decided` case with an active surety → `SponsorLiabilityPending` → grace window → either `SponsorLiabilityFired` (window expires) or `SponsorLiabilityEscaped` (revoke/restore during window). (v1-SL)

**CRITICAL VERIFY:** each path writes the correct `governance_log` entry_kind, transitions to the correct `CaseStatus`, and does NOT provision a room / apply power-levels where it shouldn't (deadlock + non-quorum must NOT sanction). Hash chain stays intact through every negative path.

**Likely gaps:** these are DB/API-layer mostly (no new bridge paths except declined-juror room membership). The sponsor-liability grace-window timing may need a config knob or a manual clock advance to test the fired/escaped branches.

---

## Phase 7 — Restart idempotency + resilience

**Why:** m2 acceptance signal: "Bridge restart resumes provisioning without duplicate Room::Created (idempotent by case_id)." The `set_watermark` / `last_seen_governance_log_row_id` machinery exists but is `#[allow(dead_code)]` (not yet wired — see SHARED-STATE). This phase tests what idempotency DOES hold today + documents the gap.

**Steps (coordinate the bridge restart with infra — it's their container):**
1. With a provisioned room for case N, ask infra to restart the bridge (`docker compose -f docker-compose.pilot.yml restart bridge`).
2. Re-fire a room-event for case N (or drive another transition).
3. **CRITICAL VERIFY:** does `bridge_room` get a DUPLICATE row for case N, or is it idempotent? (`bridge_room::lookup` before `create_community_room` should make it idempotent by `(case_id, room_type)` — confirm.)
4. Test the soft-pause path: set `messaging_enabled=false` (admin API) → confirm new transitions DON'T provision (relay paused) → set back to `true` → confirm resumes.
5. Bridge-down resilience: with the bridge stopped, drive a case transition → confirm Lemmy's governance flow does NOT break (the hook is fire-and-forget, swallows transport errors) → the case still decides + sanctions in the DB, just no Matrix effect.

**CRITICAL VERIFY:** governance never blocks on the bridge; idempotency holds by `(case_id, room_type)`; soft-pause gates provisioning cleanly.

**Likely gaps:** the `set_watermark` log-tail-resume machinery is NOT wired (dead_code) — so "resume provisioning after downtime for transitions that happened WHILE down" is NOT implemented. Confirm + document: a transition that fires while the bridge is down is LOST (no replay), because the bridge is push-driven (hook POST) not pull-driven (log tail). This is a known m2 gap, not a pilot bug.

---

## Phase 8 — Human pilot go-live (the actual point)

**Why:** phases 1–7 are scripted API testing. "Internal user testing" means real people using the **UI**, not curl. This is the gate to actually inviting household testers.

**Pre-go-live checklist:**
- [ ] Home-network reachability confirmed: every tester's device can reach `http://192.168.1.157:1236` (LAN) or `http://100.81.145.58:1236` (Tailscale, if enrolled). **User confirms** — only they know the device set.
- [ ] Rate limits sane for real humans (the live raises are testing-generous; decide pilot-realistic values via `PUT /api/v4/site`).
- [ ] A short tester guide: how to register (require_application mode — admin approves), post, report, and what governance looks like from a user's seat.
- [ ] Admin (you) ready to approve registration applications + assign juries as cases come in.
- [ ] Decide: do testers see the Matrix side at all (Element client pointed at Tuwunel), or is Matrix bridge-internal for now? (Tuwunel is localhost-only — testers can't reach it without a client + network path. For initial UI testing, Matrix stays an internal verification surface.)

**This phase is a user decision, not a scripted test.** The infra + governance layers are proven; phase 8 is "turn it on for humans and watch."

---

## Cross-cutting: what each phase should record

For every phase, append a SHARED-STATE §4 entry with: case IDs used, the `governance_log` entry_kinds produced, `bridge_room` rows added, any `m.room.power_levels` reads, hash-chain check result, and any wiring gap found (→ §5 ask to infra). Keep the spent-case ledger current so the next phase uses fresh IDs.

## Known standing gaps (carry into a bridge-hardening sub-phase, not pilot blockers)

1. `dm_round_trip` integration test still `#[ignore]`'d — root cause of the 7 runtime bugs. Un-ignore it for real regression coverage.
2. `#[allow(dead_code)]` scaffolding (`relay.rs` DM-relay, `set_watermark`, `oq009_threshold`) — wire or delete.
3. Bridge crate not in any CI/clippy gate — debt re-accumulates silently (it was 18 errors before the 2026-06-13 cleanup).
4. Bridge is push-only (no log-tail resume) — transitions while the bridge is down are not replayed (phase 7 documents this).
