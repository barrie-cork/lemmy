---
phase: m3-core-recording
kind: auto-phase-handover-refresh
stage: impl-task-4-running
refreshed_at: 2026-06-20T01:48:00Z
---

# m3-core-recording — auto-phase handover (auto-refreshed)

**Stage:** `impl-task-4-running` · **Phase branch tip:** `df5345765` · **Trunk:** governance-v0 @ `9a62a366a` · **Lane mode:** B (mobile remote-control)

## Next concrete action (re-verify on resume)

Poll **Junior #740** (Task 4 impl, base phase `df5345765`). On `done`:
1. `git fetch origin phase-m3-core-recording`; if daemon-local ahead of origin → push daemon phase branch (look-order rule). Read the worker's new `validate-pending-laptop-linux` DQ.
2. Check for a worker blocker DQ first (struct-field propagation miss, etc).
3. **Run the cargo LOCALLY** in a throwaway worktree on the new phase tip (`git worktree add ../brehon-fork-validate-740 origin/phase-m3-core-recording`; bootstrap submodules + `.env`):
   - `cargo-linux.sh check --manifest-path services/bridge/Cargo.toml`
   - `cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings`
   - `cargo-linux.sh test ... record_uploaded_emits_recording_intent`
   - `cargo-linux.sh test ... clean_posture_no_side_effects_when_disabled`
   - `cargo-linux.sh test ... room_recording_uploaded_request_json_shape`
   - Trust the EXIT echoes in the log body, NOT the bg-task notification.
4. On GREEN: mutate the validate DQ → `resolved/pass` (`advisor-laptop`) via the **daemon** (Mode B). Clean up the throwaway worktree.
5. **Advance to Task 5** (serial, requires 3): participant-floor recording-fetch endpoint — `is_participant` (§10.7) in recording.rs + `handle_recording_fetch` route in appservice.rs; CONSUMES `read_recording_config`; ADR-015 participant-floor authz is LOAD-BEARING (a recording served to a non-participant leaks a pseudonymous town hall).

## State

- **Plan:** `.claude/PRPs/plans/m3-core-recording.plan.md` @ `99ab500ef` (7 tasks, complexity 3/10)
- **Auto-state:** `.claude/auto-state/m3-core-recording.json` (stage impl-task-4-running, resume_count 7)
- **Tasks 1–3:** DONE. Task 1 (binary DTO) + Task 2 (config/helpers, 2 fix-impls) + Task 3 (recording.rs + rust-s3, 1 fix-impl). All validate DQ resolved/pass. pending 0.
- **In-flight Junior:** #740 (Task 4 impl, base phase df5345765)
- **Daemon main checkout:** ON `phase-m3-core-recording`. After #740 starts a worker, daemon has 1 worker worktree — do NOT direct-commit until it finishes.

## Task map (plan §13)

- Tasks 1, 2, 3 — DONE (3 fix-impls total: bridge_room.rs dead_code, sanction_handler E0063, recording.rs dead_code).
- **Task 4 (#740, IN FLIGHT):** maybe_record flag-gate (§10.4) + Stage::record_uploaded emit (§10.5) + bridge RoomEventPayload 5-field DTO mirror (§10.6) + clean-posture negative invariant. CONSUMES record_town_halls_enabled + compute_content_sha256 + RecordingSink → clears the Task-2/Task-3 dead-code #[allow]s. HIGHEST-propagation task.
- Task 5: serial, requires 3 (participant-floor fetch; is_participant; CONSUMES read_recording_config; ADR-015)
- Task 6: serial-docker, requires 3,4,5 (#[ignore] integration tests/recording.rs, compile-only)
- Task 7: retro

## Cross-session deps / tripwires

- **R7 (clean-posture):** the negative test MUST fail if the flag-gate is deleted; paired with a positive enabled=true test.
- **R11 (CATCH-FIRE):** content_sha256 rides append_room_event via the EmitIntent; recording.rs/stage.rs do NOT write the chain. DoD: grep returns nothing.
- **Struct-field-add propagation (3× recurrence this phase):** Task 4 adds 5 fields to bridge RoomEventPayload → ALL existing literals across stage.rs + room_event_client.rs need them. Brief requires `rg "RoomEventPayload\s*\{"` enumeration FIRST. A missed literal = E0063. cargo test passing does NOT prove clippy-clean (test code counts a forward-declared item as used).
- **Dead-code #[allow] discipline:** remove only for items Task 4 makes used in the non-test build (compute_content_sha256, RecordingSink); LEAVE on still-unused (LiveSink, record_town_halls_enabled if not read this task). Err toward leaving; clippy gate catches over-removal harmlessly (stale #[allow] on a used item is not an error).
- **Masked-exit trap:** bg cargo notifications report process exit, not cargo result — read the EXIT echoes in the log body.
- **Retro carry-forward (STRONG — 3× this phase):** every fix-impl traced to a forward-declared-until-consumer OR struct-field-propagation brief gap. The impl-task brief template needs a hard rule: when a task creates forward-declared items (fn, trait, AND struct) or adds struct fields, enumerate ALL of them for #[allow(dead_code)] / constructor-propagation, AND note that cargo test passing masks dead-code because #[cfg(test)] code counts as a use. Add to phase retro §What-to-change + propose a lesson (feedback_forward_declared_items_need_allow_until_consumer.md or extend the §2.4 file-class table).
- **Remaining gates:** e2e local-vs-dispatch / CR-triage / merge-confirm / retro-sign-off (gate-1 DONE).
