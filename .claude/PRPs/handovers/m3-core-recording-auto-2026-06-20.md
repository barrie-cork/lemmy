# m3-core-recording — /auto-phase auto-handover (2026-06-20)

**Self-contained resume artifact. Zero conversation context required.**

## RESUME block

- **Sub-phase:** m3-core-recording (M3 Phase 5) — optional evidentiary-store recording (LiveKit Egress → MP4 → S3 → `content_sha256` via `append_room_event` → participant-floor fetch). Plan `99ab500ef` (8 tasks: T0 preflight + T1–6 impl + T7 retro), complexity 3/10.
- **Mode:** B (mobile remote-control) — daemon does phase-branch commits/DQ; canonical `brehon-fork` on `governance-v0` does meta-edits only.
- **State-machine stage:** `impl-task-6-running` (auto-state `.claude/auto-state/m3-core-recording.json`, resume_count 7).
- **Phase branch tip:** `14f06206f` (`origin/phase-m3-core-recording`) — last commit: `chore(advisor): m3-core-recording impl-6 brief on phase branch (Mode-B single-file pull)`.
- **In-flight Junior:** **#742** `[role:impl-task] m3-core-recording-task6` (base phase-m3-core-recording @ `14f06206f`) — the docker-gated `#[ignore]` recording integration test (compile-only).
- **DQ pending:** 0.
- **Concurrent activity:** none (0 running Junior tasks at dispatch; single advisor session).

## Progress

- **T1–T5 DONE + validated** (all `-linux` gates GREEN; 3× fix-impl recurrence this phase, all forward-declared/struct-field-propagation class — codified as `feedback_forward_declared_items_need_allow_until_consumer.md` + template §2.5 gate). T4 added a 4th distinct clippy class: `too_many_arguments` arity (8/7) on `maybe_record` — retro-watch.
- **T5 (`a933a00a7`):** participant-floor fetch (`is_participant` + `handle_recording_fetch` route, ADR-015). -linux GREEN (check/clippy -D warnings/`is_participant_floor` test all 0). §4.4 clippy-arity pre-warning held — no `too_many_arguments` on `handle_recording_fetch`.
- **T6 (in flight, #742):** `services/bridge/tests/recording.rs` — 3 `#[ignore]` `#[tokio::test]` stubs (compile-only; live stack is Phase-6). Mirrors `stage_mode.rs`/`emergency_mute.rs`.

## NEXT concrete action (re-verify on resume)

Poll #742 → on done, validate its `validate-pending-laptop-linux` DQ locally in a throwaway worktree on the new phase tip: `cargo-linux.sh check` + `clippy --no-deps -- -D warnings` + `test --test recording --no-run` (the `#[ignore]` tests must **compile**, not run). On GREEN → mutate DQ resolved/pass via daemon (`answered_by: advisor-laptop`, commit `chore(decision-queue):`, push) → clean up worktree → **Task 6 is the LAST impl task** → advance to `bm-pr-pending` (author bm-pr brief, queue `[role:bm-task]`). On clippy/test-compile fail → §G4 classify (likely a mirror gotcha: `anyhow` dev-dep or a `[[test]]` Cargo.toml entry).

**Remaining user gates:** e2e local-vs-dispatch (gate 4), CR-triage (gate 3), merge-confirm (gate 5), retro-sign-off (gate 6). Task 7 retro authored at the `retro-author` stage.

## Phase-retro carry-forwards

1. 3× fix-impl recurrence (forward-declared/struct-field-propagation) — already codified; confirm the §2.5 gate held for T4/T5/T6.
2. T4 `clippy::too_many_arguments` arity case — if it recurs, extend §2.5 Trigger B for >7-param forward-declared fns + add a §G4 allowlist row for `too_many_arguments`.
3. Daemon finalize-tail-vs-hang diagnosis ambiguity (#741 last session was a finalize-race, not a hang — `running` DB row + absent worker process is ambiguous; `docker ps`/wait-one-poll resolves either way).
