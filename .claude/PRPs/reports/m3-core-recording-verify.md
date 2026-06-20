# /brehon-verify — m3-core-recording

**Run:** 2026-06-20 (advisor inline, /auto-phase verify-running stage)
**Phase branch tip:** `39f028279` (source-equivalent to `84c449704`; the `39f028279` delta is the DQ-resolve commit only — no source change)
**PR:** #205 (`phase-m3-core-recording → governance-v0`, barrie-cork/lemmy)
**Verdict:** ✅ ALL 5 STORIES PASS — no phantoms, no regressions. Advance to gate 5 (merge-confirm).

## §16a Stories (5/5 ✓)

| Story | Checkpoint | Brief-Scope outputs | Verdict |
|---|---|---|---|
| 1 — content_sha256 chain entry (marquee, hash rides `append_room_event`) | `record_uploaded_emits_recording_intent` (GREEN in Task 4 -linux gate + fix-in-pr #745 gate) | `stage.rs::record_uploaded` pushes `room_recording_uploaded` EmitIntent; `governance_log.rs` + `room_event_client.rs` RoomEventPayload carry the 5 fields | ✅ |
| 2 — `record_town_halls` from existing column (no migration) | `record_town_halls_flag_parse` (GREEN in Task 2 -linux gate) | `bridge_room.rs` 3 helpers; `config.rs` 4 S3 settings; NO new column/migration | ✅ |
| 3 — clean-posture negative invariant (marquee) | `clean_posture_no_side_effects_when_disabled` (GREEN in Task 4 -linux gate + fix-in-pr #745 gate, 1 passed) | `recording.rs::maybe_record` with `if !enabled { return Ok(()) }` gate at :41 + delete-the-gate test | ✅ |
| 4 — participant-floor fetch (ADR-015) | `is_participant_floor` (GREEN in Task 5 -linux gate) | `recording.rs::is_participant`; `appservice.rs` `/brehon/recording/{id}` + `handle_recording_fetch` calls `is_participant` BEFORE serving | ✅ |
| 5 — full flow compiles (Phase-6-pilot-grade docker test) | `--test recording --no-run` (GREEN in Task 6 -linux gate) | `tests/recording.rs` 4 `#[ignore]` tests, 3 scenarios (`recording_lands_with_hash_on_chain`/`clean_posture_no_recording_when_disabled`/`participant_floor_fetch`), `anyhow::Result<()>` | ✅ |

## §15.5 cross-cutting (R8/R9/R10/R11/R12/R14 — all ✓)

- **R8 (no migration / no new column):** migrations diff empty; `bridge_room.rs` phase diff = 66 insertions, 0 `ALTER TABLE`/`ADD COLUMN` (the 4 `ALTER` hits are pre-existing idempotent helper code, not phase-added); writes use existing `recording_config` column via `INSERT ... ON CONFLICT DO UPDATE`. ✅
- **R9 (ADR-015 pseudonyms):** no `person_id`/`username`/MXID-shaped strings in `recording.rs`/`stage.rs`; fetch calls `is_participant` before serving. ✅
- **R10 (ADR-016 metadata-only):** payload = `{media_url, content_sha256, duration_s, speakers, attendance_count}` + top-level `actor_pseudonym`; MP4 bytes → S3, never chain. ✅
- **R11 (hash rides `append_room_event`):** `compute_content_sha256` present in `recording.rs`; NO direct `governance_log`/`append`/`INSERT` write (the 2 matches are doc-comment refs explaining the constraint); hash flows EmitIntent → drain_emits → post_room_event → append_room_event. ✅
- **R12 (generic S3, no hardcoded endpoint):** no `minio.` literal anywhere in `services/bridge/src/`; `s3_endpoint` read from config in `recording.rs`. ✅
- **R14 (deps confined to Task 3, lock synced):** `services/bridge/Cargo.toml` diff = `rust-s3 = "0.34"` + `sha2 = "0.10"` (sha2 promoted transitive→direct); `Cargo.lock` changed in same diff (425/+267); no other task touched deps. ✅

## CR fix-in-pr coverage (PR #205)

- **cr-4 (LiveSink fail-open):** FIXED — `trigger_egress`/`upload` scaffold stubs now `anyhow::bail!` (fail-closed; refuse success until implemented). Validated.
- **cr-5 (None actor_pseudonym):** FIXED — `record_uploaded` non-empty chair-pseudonym guard before EmitIntent push; `()→anyhow::Result<()>`; 2 callsites `?`-propagated; clippy `-D warnings` clean (no `unused_must_use`). Validated (DQ `7019c9444712-001` resolved/pass).
- **cr-1 (stale Task 6 DQ):** rebutted (already-resolved at CR snapshot time).
- **cr-2 + cr-3 (scaffold requester-pseudonym + participant-set):** carry-forward to Phase-6 (live session-auth wiring) per gate-3.

## Validation provenance

No cargo re-run needed at verify time: all 5 story checkpoint tests + the §15.5 ADR-pin greps were proven GREEN on Linux (Docker rust:1.95) across the 5 per-task `-linux` gates + the fix-in-pr #745 gate (check + clippy `-D warnings` + the 2 unit tests, all EXIT 0). Verify is the structural/phantom-completion check: every Brief-Scope output exists + matches its pattern on the phase tree. Confirmed via read-only `git archive` extract on the daemon.
