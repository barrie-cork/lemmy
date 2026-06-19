# /brehon-verify — m3-core-stage-mode (re-run after cr-4 fix)

**Phase branch:** `phase-m3-core-stage-mode` @ `8b8c877a7`
**Plan:** `.claude/PRPs/plans/m3-core-stage-mode.plan.md`
**Verified:** 2026-06-19 (re-run; prior pass @ `9152aa4d6`)
**Outcome:** ✅ ALL STORIES PASS — no phantoms, no checkpoint failures

This is a **re-verify after the cr-4 fix-in-PR commit** (single-presenter revoke-before-grant). The only diff vs the prior all-✓ verify (`9152aa4d6`) is `services/bridge/src/stage.rs`: two `RevokePublish`-before-`GrantPublish` blocks (`promote_next`, `chair_override(ForcePromote)`) + a stronger `fifo_mic_pass_in_sequence` assertion. No other story's outputs changed.

## §16a Story verification

| Story | Checkpoint | Brief-Scope output | Result |
|---|---|---|---|
| 1 — chair drives 4 mic-passes (marquee) | `test stage::fifo_mic_pass_in_sequence` | `stage.rs` `promote_next`/`raise_hand`/`on_activate` + FIFO + **revoke-before-grant** | ✅ test passed (validate-722); now asserts `RevokePublish(W1/W2/W3)` before each next `GrantPublish` |
| 2 — 30s no-activate auto-revoke + next-promote (boundary) | `test stage::grace_no_activate_auto_revokes_and_promotes_next` | `stage.rs` `on_grace_expired` + `GRACE_SECS=30` + `#[tokio::test(start_paused)]` | ✅ test passed (virtual-time); unaffected by cr-4 (grace path already revoked) |
| 3 — chair override emits `room_chair_override` (pseudonym target) | `test room_event_client::override_*` | `room_event_client.rs` `post_room_event`; `stage.rs::chair_override` pushes intent; `RoomEventPayload.action`/`target_pseudonym` | ✅ ForcePromote now also revokes-before-grant; emit-intent push unchanged |
| 4 — chair transfer emits `room_chair_transferred` | `test room_event_client::transfer_*` | `stage.rs::transfer_chair` returns `(from,to)` + pushes intent; `RoomEventPayload.from/to/at` | ✅ passed; unchanged by cr-4 |
| 5 — town-hall opens in stage mode, Q&A live, e2e emits chair entries | `test --test stage_mode --no-run` (compile-gate) | `room_provisioner.rs` stage-mode path seats `chair_id` + Q&A=Matrix timeline; `tests/stage_mode.rs` | ✅ unchanged by cr-4; symbols present |

## Validation summary (bridge Linux, advisor-laptop, DQ 8c61e18ffcea-001 @ `485d0c8d2`)

- `cargo-linux check --manifest-path services/bridge/Cargo.toml` → **rc=0**
- `cargo-linux clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings` → **rc=0** (0 warnings; the cr-4 fix added no dead code)
- `cargo-linux test --manifest-path services/bridge/Cargo.toml stage` → **10 passed, 0 failed** — all 10 stage tests green at the cr-4 tip:
  - `fifo_mic_pass_in_sequence` (marquee, now asserting revoke-before-grant on W1→W2, W2→W3, W3→W4)
  - `grace_no_activate_auto_revokes_and_promotes_next` (Task-4 boundary, virtual-time)
  - `grace_activate_before_expiry_no_revoke` (contrast)
  - `chair_override_produces_emit_intent`, `transfer_chair_produces_emit_intent` (emit-intent)
  - `chair_override_reorder`, `promote_next_on_empty_fifo_errors`, `on_activate_wrong_participant_errors`, `raise_hand_no_dup`, `stage_load_roundtrip` (type-state guards)

The cr-4 fix is strictly additive to the test surface (a stronger assertion on the existing marquee) — no test was removed or weakened, and the empty-FIFO guard still fires `pop_front()?` BEFORE the new revoke (so an empty-FIFO error never revokes the seated speaker).

## cr-4 closure (single-presenter invariant)

The CodeRabbit MAJOR finding (PR #202): `promote_next` + `chair_override(ForcePromote)` issued `GrantPublish` for the new participant without revoking the prior `self.current` holder → two concurrent publishers possible on back-to-back promotes. Fixed: both paths now `self.current.take()` + `GrantCmd::RevokePublish(prev)` before the `GrantPublish`. The grace path (`on_grace_expired`) already revoked correctly (Task 4) — this brings the direct-promote paths in line. Verified by reading `promote_next:107` + `ForcePromote:243` on the phase tip and by the marquee test's new revoke-before-grant assertions.

## Conformance-audit detection (§3.9.1)

NOT triggered — the cr-4 diff is bridge-side (`services/bridge/src/stage.rs`); no `crates/apub/activities/src/governance/**`, `crates/api/api/src/governance/**`, or `crates/db_schema/src/source/governance/**` scope. No Tier-1 audit scope (unchanged from prior verify).

## ADR conformance (§15.5 cross-cutting)

- **ADR-015 (pseudonyms-only):** cr-4 touched only the publish-grant revoke ordering (`RevokePublish(prev)` where `prev` is the seated participant pseudonym already in `self.current`) — no new identity surface, no chain-payload change. ✅
- **ADR-016 (metadata-only):** no change to `post_room_event` or Q&A. ✅
- **No migration, no new dep.** ✅

## Verdict

✅ **Advance to merge-confirm (gate 5).** All 6 impl tasks + the cr-4 fix shipped; all 5 stories re-verified; bridge Linux-compile gate at `result:pass` on the cr-4 tip. No phantom completions, no regressions. All CR findings addressed (cr-4 fixed+tested, cr-1/cr-2 runlog-fixed, cr-3 rebutted).
