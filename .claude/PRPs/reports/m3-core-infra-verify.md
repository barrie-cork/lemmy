# /brehon-verify report — m3-core-infra

**Phase branch:** `phase-m3-core-infra` @ `387a4b89c`
**PR:** #201 → `governance-v0`
**Verify run:** 2026-06-18 (advisor inline, post-CR-fix)
**Result:** ✅ ALL STORIES PASS — no phantoms, no regressions. Clear to merge.

## §16a Story verification

| Story | Brief-Scope structural outputs | Checkpoint command | Result |
|---|---|---|---|
| **1** — governance-only instance (`rtc_enabled=false`) runs clean, RTC stack absent | `BridgeStatus.rtc_enabled` (×4), `seed_rtc_enabled_config/up.sql` seeds false (×2), `m3_rtc_disabled_clean_posture_governance_unaffected` test present | scoped e2e `test(m3_rtc_disabled_clean_posture)` | ✅ PASS 1/1 (exit 0, 31.4s) |
| **2** — `rtc_enabled=true` mints pseudonym-JWT for `always_pseudonym` room | `livekit_jwt.rs::mint_access_token` (`sub: identity`, no person_id), `get_bridge_actor_pseudonym` endpoint, `/bridge/actor-pseudonym` route wired | livekit_jwt test + Task-4 endpoint e2e | ✅ `mint_pseudonym_claims` ok (Task 3); `m3_actor_pseudonym_endpoint_idempotent_opaque` + `..._route_authed` PASS 2/2 |
| **3** — `bridge_room` carries RTC state, idempotent on old DBs | `bridge_room.rs::open()` CREATE has chair_id/queue_state/recording_config + PRAGMA-gated ALTER (cr-6 fix) | `cargo-linux.sh test ... bridge_room` | ✅ 2 passed (`fresh_db_has_rtc_columns`, `old_shape_db_gets_rtc_columns_and_second_open_is_idempotent`) |
| **4** — RTC sidecars deploy under a profile; no-profile `up` starts none | `docker-compose.yml` 3× `profiles: ["rtc"]`, `AGPL-NOTICE.md` 3 rows, `docker/docker-compose.yml` untouched | §15.6 deploy-smoke | ✅ all 3 boot (livekit `--dev` Up, lk-jwt :8085 HTTP 200, element-call :8086 HTTP 200); no-profile = tuwunel only; `--profile rtc` lists all 3 |

## Supporting validation (this session)

- **Windows cargo check** `--workspace --features full`: exit 0 (Task 4).
- **Bridge Linux compile** (`cargo-linux.sh check`, Docker rust:1.95): exit 0 @ `387a4b89c`-equivalent (5m17s; 4 benign dead-code warnings on `mint_access_token`/`Claims` — Phase-1 mints the capability, callers land Phase 3+).
- **e2e total count:** 147 (145 pre-phase + m3_actor_pseudonym + m3_rtc_disabled).

## CR finding disposition (PR #201, gate-3 cleared)

8 CodeRabbit findings (6 major, 2 low, 0 critical); Copilot quota-blocked (0).
- **rebut:** cr-3 (ADR-015 scrub claim — false-positive: pseudonym IS the redacted value; sibling `get_bridge_status` doesn't scrub; no scrub() convention).
- **advisor-fixed:** cr-1 (dedup duplicate DQ id `7b96c7097`), cr-2 (AGPL "pinned"→"version-tagged" + element-call v0.6.0 `bfcf8aac9`).
- **fix-in-pr:** cr-6/7/8 bridge (Linux PASS), cr-4 route test + cr-5 SQLFluff (e2e PASS), cr-4b E0308 SessionMiddleware deref follow-up (`387a4b89c`).

## Verdict

All 4 §16a stories ✓ with real checkpoints (no phantom completions). All CR findings dispositioned + validated. Clear to advance to merge confirm (Gate 5).
