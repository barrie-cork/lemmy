# m3-core-recording — /auto-phase auto-handover (2026-06-20)

**Self-contained resume artifact. Zero conversation context required.**

## RESUME block

- **Sub-phase:** m3-core-recording (M3 Phase 5) — optional evidentiary-store recording (LiveKit Egress → MP4 → S3 → `content_sha256` via `append_room_event` → participant-floor fetch). Plan `99ab500ef` (8 tasks: T0 preflight + T1–6 impl + T7 retro), complexity 3/10.
- **Mode:** B (mobile remote-control) — daemon does phase-branch commits/DQ; canonical `brehon-fork` on `governance-v0` does meta-edits only.
- **State-machine stage:** `bm-pr-running` (auto-state `.claude/auto-state/m3-core-recording.json`).
- **Phase branch tip:** `851aca028` (`origin/phase-m3-core-recording`) — all 6 impl tasks done + validated.
- **In-flight Junior:** **#743** `[role:bm-task] m3-core-recording bm-pr` (base governance-v0) — opening the PR into `governance-v0`.
- **DQ pending:** 0 (all 6 validation DQs resolved/pass: Task 1 crates `91c7d92933b0-001`; bridge `-linux` Cohort A/`031b1fbf8eb1`/`96dd2bccbd79`/`cdbea3487eca`/`0aa481cce3a6`).
- **Concurrent activity:** none.

## Progress — ALL IMPL DONE

- **T1 (crates, Windows-validated):** binary `RoomEventPayload` 5 optional recording fields. DQ `91c7d92933b0-001` pass.
- **T2–T6 (bridge, Linux-validated):** config+flag-gate, recording primitives+rust-s3 dep, flag-gated emission+clean-posture, participant-floor fetch, docker-gated `#[ignore]` ITC. All 5 `-linux` gates GREEN (check / clippy -D warnings / unit+compile tests).
- **3× fix-impl recurrence** this phase, all forward-declared/struct-field-propagation class (codified `feedback_forward_declared_items_need_allow_until_consumer.md` + §2.5 gate). T4 added a 4th distinct class: `clippy::too_many_arguments` arity (8/7) on `maybe_record` — retro-watch.

## NEXT concrete action (re-verify on resume)

Poll #743 → on done, verify the PR exists (`gh pr view --repo barrie-cork/lemmy --json number,state,baseRefName,isDraft` — base `governance-v0`, NOT draft) → record PR# in auto-state → advance to `cr-wait` (poll for CodeRabbit comment, cadence 600s). When CR posts → `bm-poll-cr` → `bm-triage` → **gate 3 (CR-triage, AskUserQuestion, four-bucket counts)**. Then `verify-running` (`/brehon-verify` iterates §16a Stories 1–5 + §15.5 cross-cutting R8/R11/R12/R9/R10/R14) → **gate 5 merge-confirm** → `bm-merge` → Task 7 retro → **gate 6 retro-sign-off** → `/brehon-phase-transition`.

**No e2e track in this plan** (binary DTO = non-breaking optional fields, validated by `--workspace --features full` check+clippy; gate 4 e2e local-vs-dispatch does NOT apply).

## Phase-retro carry-forwards (Task 7)

1. 3× fix-impl recurrence (forward-declared/struct-field-propagation) — already codified; confirm §2.5 gate held for T4/T5/T6 (it did — T5/T6 clean first-try).
2. T4 `clippy::too_many_arguments` arity case — if it recurs, extend §2.5 Trigger B for >7-param forward-declared fns + add a §G4 allowlist row for `too_many_arguments`. The §4.4 brief pre-warning on T5/T6 held (no recurrence).
3. Daemon finalize-tail-vs-hang diagnosis ambiguity (#741 was a finalize-race; the look-order rule + `docker ps`/wait-one-poll resolves either way).
4. `rust-s3` 0.34 resolved clean on Linux first-try (no Cargo.lock licence/version conflict — the headline-risk row did not fire).
