# Retro — m3-core-emergency-mute

**Phase:** M3 Phase 4 — federation-wide emergency mute-all
**PR:** #204 @ `34f647ad9` → `governance-v0`
**Date:** 2026-06-19
**Wall-clock:** ~10h (bootstrap `4407d23cb` → merge `34f647ad9`, single day)

---

## §1 What shipped

Federation-wide emergency mute-all for M3 town-hall rooms:

- **`governance_log.rs`** — `RoomEventPayload.federated: Option<bool>` (non-breaking DTO extension; `skip_serializing_if` keeps existing emissions byte-identical) (Task 1)
- **`mute_handler.rs`** — NEW: `compute_mute_all_override` (pure; raises `events["m.call.member"]` + MSC3401 alias above `users_default`) + `mute_all_power_levels` (per-room GET→mutate→PUT, mirrors `sanction_handler`); `sanction_handler.rs` `get_power_levels`/`put_power_levels` widened to `pub(crate)` (Task 2)
- **`stage.rs`** — `pub fn mute_all(publishers, federated, sink)`: RevokePublish sweep over all non-chair publishers + one `room_mute_all` EmitIntent; marquee test `mute_all_revokes_all_publishers` asserts zero-holder set-equality invariant; `room_event_client.rs` `federated` mirror field + `mute_all_request_json_shape` test (Task 3)
- **`emergency_mute.rs`** — NEW: `#[ignore]` compile-gated cross-instance e2e stub (Phase-6 pilot grade; `--no-run` gate passes) (Task 4)
- **Bundled clippy-debt fix** — 7 governance/bridge files, 20 pre-existing `-D warnings` lints; `bridge_auth.rs` `?`-propagation on `BRIDGE_CALLBACK_SECRET` (real security boundary fix)
- **fix-impl-1** (`8bbcb4ec8`) — CR cr-9 max()-guard (never lower pre-existing threshold), cr-10 `events` absent/non-object error propagation, cr-11 chair-exclusion from revoke loop
- **fix-impl-2** (`be251582e`) — CR cr-2/3/7/8 governance hook `.ok()` → `?` / `tracing::warn!` propagation

All 4 §16a stories ✓. No open CR findings at merge.

---

## §2 CR triage summary

| Bucket | Count | Notes |
|---|---|---|
| `done` | 7 | cr-2/3/7/8/9/10/11 — all fixed in fix-impl-1+2 |
| `rebut` | 1 | cr-12 `todo!()` — intentional Phase-6 stub; DoD is compile-only |
| `wont-fix` | 4 | cr-1/4/5/6 — outside-diff noise (meta files, MD lint, DQ audit) |

5 critical + 2 major → 0 open at merge. CR was the primary correctness gate for this phase (no cargo regression in CI; local bridge unit tests + Windows crates check = internal coverage).

---

## §3 Per-task metrics (Tier-1 signals per `feedback_retro_task_complexity_score.md`)

| Task | Files | Commits | Runtime (est.) | Max log silence |
|---|---|---|---|---|
| Task 1 (federated DTO) | 1 | 1 | ~30 min | ~20 min (cargo check) |
| Task 2 (mute_handler) | 3 | 1 | ~60 min | ~45 min (bridge cold build) |
| Task 3 (stage mute_all) | 2 | 1 | ~45 min | ~30 min (bridge test) |
| Task 4 (e2e stub) | 1 | 1 | ~30 min | ~20 min (compile-gate) |
| fix-impl-1 (bridge CR) | 2 | 1 | ~50 min | ~40 min (bridge test) |
| fix-impl-2 (crates CR) | 4 | 1 | ~30 min | ~20 min (clippy) |
| **Total** | **13** | **6** | **~245 min** | — |

---

## §4 What worked well

1. **cr-4 zero-holder negative invariant (Stage 3 marquee test)** — the lesson from m3-core-stage-mode's cr-4 finding was pre-loaded into the brief. The test asserted set-equality of revokes from the start; CR had nothing to add here. Pattern: `feedback_authz_state_machine_test_asserts_negative.md` is load-bearing; cite it in every `mute`/`revoke` brief.

2. **Parallel fix-impl dispatch** — fix-impl-1 (bridge) and fix-impl-2 (crates) ran concurrently. Bridge is Linux-only; crates is Windows. Zero file overlap. Wall-clock for both = max(50, 30) ≈ 50 min vs 80 min serial. Pattern confirmed: file-disjoint fix cohorts parallelise cleanly.

3. **Worker LESSON trailers** — both fix-impl workers shipped useful LESSON trailers:
   - `8bbcb4ec8`: "When a pure fn changes return type from T to Result<T>, the caller in a best-effort loop needs match-continue, not ? to preserve per-room isolation."
   - `be251582e`: "Post-transaction hook calls must never silently discard errors. Default is ? propagation; warn is acceptable only when a sibling in the same file demonstrates the best-effort pattern for the same hook type."

4. **Mode B brief visibility (surgical checkout)** — `git checkout origin/governance-v0 -- <brief-files>` onto the phase branch avoided the `git merge` roadmap conflict that would have occurred with a full merge. Pattern holding since m2-late.

---

## §5 What went wrong / lessons

### L1 — `--bins <module>::<fn>` filter breaks when `mod tests {}` wrapper is present (recurrence-2)

**Incident:** Task 3 validate brief specified `--bins stage::mute_all`. Worker ran 0 tests (the filter substring `stage::mute_all` does not match the full path `stage::tests::mute_all_revokes_all_publishers` when `mod tests {}` is the outer wrapper). Brief had to be corrected to use bare fn name `mute_all_revokes_all_publishers`.

**Fix-impl-1 brief already applied the lesson** (used bare fn name). But the Task 3 brief authored earlier used the module-qualified form. This is a 2nd recurrence (Task 2 in m3-core-stage-mode was the first).

**Candidate lesson:** Add to bridge impl-task brief template §4 Constraints: "For `--bins` test filters, use bare fn name (no `module::` prefix). The `mod tests {}` wrapper breaks substring matching. `--bins mute_all_revokes_all_publishers` not `--bins stage::tests::mute_all_revokes_all_publishers`."

### L2 — Finalize-hazard (daemon merges BM worker branch into local gov-v0) — recurrence 6+

BM tasks (#730 bm-pr, #731 bm-poll-cr) both triggered daemon finalize-merge into daemon-local gov-v0. Both were benign (runlog-only diffs) but required: detect on poll → push daemon gov-v0 to origin → pull locally.

Pattern stable. Mitigation remains: post-BM-task poll always checks daemon gov-v0 tip. The hazard is architectural (daemon's generic finalize always merges to the local ref). No new lesson; existing `feedback_finalize_merge_where_to_look_first.md` covers it.

### L3 — Windows git refspec-fetch failure when daemon is checked out on gov-v0

`git fetch origin governance-v0:governance-v0` fails with "rejected (would clobber existing ref)" when the daemon's working tree IS checked out on `governance-v0`. Fix: `git fetch origin governance-v0 && git merge --ff-only FETCH_HEAD`. Confirmed pattern — already captured in lesson; noted here for recurrence tracking.

### L4 — CR detected 5 outside-diff critical findings (clippy-debt context exposure)

The bundled 7-file clippy-debt fix brought CR's attention to pre-existing `.ok()` / `unwrap_or_default()` patterns in governance handlers that were already on `governance-v0` (not introduced this phase). CR correctly flagged them. Triage decision: fix-in-PR (2 fix-impl tasks). This added ~80 min wall-clock but shipped real correctness improvements (`bridge_auth.rs` pattern now fully consistent across `sanction_publisher.rs`, `actor_app_link.rs`, `submit_jury_vote.rs`, `revoke_endorsement.rs`).

**Lesson candidate:** When bundling a clippy-debt fix into a feature PR, expect CR to surface adjacent debt in the same files. Budget one fix-impl cohort (~80 min) for outside-diff findings. This is net-positive (the fixes are real).

### L5 — merge-forward conflict on runlog (append-append)

Phase branch and gov-v0 both appended to `m3-core-emergency-mute-runlog.md` independently (phase branch: bm-cut; gov-v0: bm-pr + bm-poll-cr). The merge-forward before `gh pr merge` produced an add-add conflict. Resolved mechanically with regex (keep both sides in chronological order).

Pattern: runlog conflicts on merge-forward are routine (same file, different BM tasks append at different times). Consider a `chore(bm): runlog conflict resolved` helper script at retro time.

---

## §6 Carry-forwards

1. **Bridge brief template §4 — bare fn name for `--bins`** (L1, recurrence-2 → promote): add constraint to `.claude/PRPs/templates/` bridge impl-task brief section. Next phase that touches `services/bridge/src/` picks this up.

2. **`feedback_daemon_long_name_refspec_finalize.md`** — lesson from m3-core-stage-mode (4th recurrence of long branch name truncation on daemon finalize) was cited in this phase's retro as still-active. Verify it remains in the brief lesson injection table for the next bridge phase.

---

## §7 ADR compliance check

| ADR | Requirement | Status |
|---|---|---|
| ADR-015 | `room_mute_all` chain entry carries `actor_pseudonym` (chair), no MXID/person_id | ✅ `actor_pseudonym = self.chair` (a pseudonym by Stage contract); JSON-shape test asserts |
| ADR-016 | Metadata only — `{ federated, actor_pseudonym }`, no content hashed | ✅ `federated: Option<bool>` is the only new payload field; no content in the chain entry |
| ADR-014 | RTC federation is Brehon↔Brehon only (Matrix power-levels for cross-instance) | ✅ power-level PUT uses existing `sanction_handler` Matrix HTTP machinery |
| ADR-013 | EmergencyRemove path untouched | ✅ (no modification) |
| ADR-011 | AGPLv3 — no new component | ✅ (no new dependency or sidecar) |

---

## §8 Open questions / phase-transition notes

- `emergency_mute.rs` `todo!()` stub deferred to Phase-6 pilot. The `#[ignore]` gate compiles; the live cross-instance <500ms measurement requires a two-instance docker-compose stack (Phase-6 scope).
- `room_mute_all` is now emittable. Phase 5 (recording/MinIO) emits `room_recording_uploaded`. Phase 6 wires the live HTTP endpoints (`transfer_chair`, `chair_override`, `mute_all` triggers).
- clippy baseline debt in remaining governance handlers: `issue_note_governance_clippy_baseline_debt.md` — 19 lints still open in a `chore/governance-clippy-debt` PR candidate (4× `unwrap_or_default` may be semantic). Not blocking Phase 5.

---

## §9 Confidence score (post-mortem)

- **Correctness:** 9/10 — marquee unit test (zero-holder), JSON-shape test, power-level unit test, compile-gate all green; CR critical/major findings all fixed pre-merge.
- **Process:** 8/10 — L1 `--bins` recurrence and L5 runlog conflict are minor friction; the fix-impl parallel pattern worked cleanly.
- **ADR compliance:** 10/10 — all five load-bearing ADRs verified at merge.
