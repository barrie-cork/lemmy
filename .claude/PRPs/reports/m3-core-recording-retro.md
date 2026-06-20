# Retro — m3-core-recording

**Sub-phase:** m3-core-recording (M3 Phase 5) — optional evidentiary-store recording (LiveKit Egress → MP4 → S3 → `content_sha256` chain entry via `append_room_event` → participant-floor fetch).
**Plan:** `99ab500ef` (8 tasks: T0 preflight + T1–6 impl + T7 retro), complexity 3/10.
**PR:** #205 → MERGED `2a400a6ab` (2026-06-20 08:17 UTC). Phase branch deleted (L16 ✓).
**Mode:** B (mobile remote-control). Driven by `/auto-phase m3-core-recording` (7 resume cycles).
**TL;DR:** Clean ship. The new `rust-s3` dep resolved first-try on Linux; the §2.5 forward-declared gate held for T5/T6 (the 3× fix-impl recurrence front-loaded to T2/T3); CR surfaced 2 real fail-closed/ADR findings, both fixed + validated. One merge-forward runlog conflict (recurrence). No catch-fires, no cycle-count trips, no phantom completions.

## §1 What shipped

- **Binary `RoomEventPayload` recording fields** (`governance_log.rs`): 5 optional `skip_serializing_if=Option::is_none` fields — `media_url`, `content_sha256`, `duration_s`, `speakers: Option<Vec<String>>` (pseudonyms), `attendance_count`. Non-breaking (existing emissions byte-identical). Windows `--workspace --features full` validated (T1).
- **Bridge S3 config + `record_town_halls` flag-gate** (`config.rs` + `bridge_room.rs`): 4 operator-level S3 settings (endpoint from config/env, never hardcoded), 3 `recording_config`-column helpers; the flag rides the EXISTING column (no migration, R8) (T2).
- **Recording primitives + the one new dep** (`recording.rs`): `compute_content_sha256`, `RecordingSink` trait + `LiveSink` real impl (Egress via reqwest+livekit_jwt; S3 PUT via `rust-s3` 0.34) + spy. Cargo.lock same-commit (R14) (T3).
- **Flag-gated emission + clean-posture invariant** (`maybe_record` + `stage.rs::record_uploaded`): zero side-effects when `!enabled`; `room_recording_uploaded` EmitIntent on the existing `drain_emits → post_room_event → append_room_event` seam; delete-the-gate negative-invariant test (T4).
- **Participant-floor fetch** (`is_participant` + `appservice.rs /brehon/recording/{id}`): non-participant → 403, ADR-015 floor cannot be zero (T5).
- **Docker-gated e2e stub** (`tests/recording.rs`): 3 `#[ignore]` compile-gated scenarios, `todo!()` bodies, Phase-6 pilot grade (T6).
- **CR fix-in-pr:** cr-4 fail-closed `LiveSink` (`anyhow::bail!` on scaffold stubs), cr-5 non-empty `actor_pseudonym` guard in `record_uploaded` (`()→Result<()>`, 2 callsites `?`-propagated).

Registry count UNCHANGED at 72 (emit-only — the `room_recording_uploaded` const shipped Phase 2). No migration.

## §2 CR triage summary (PR #205, 5 findings, gate-3 user-accepted)

| id | finding | bucket | outcome |
|---|---|---|---|
| cr-1 | stale Task-6 DQ timestamp | **rebut** | already-resolved at CR snapshot time |
| cr-2 | requester-pseudonym trust/spoofable | **carry-forward** (Phase-6) | scaffold; live session-auth wires it |
| cr-3 | participant-set stubbed / empty-set | **carry-forward** (Phase-6) | scaffold; all callers 403 until wired |
| cr-4 | LiveSink fail-OPEN (success without work) | **fix-in-pr** | FIXED — `anyhow::bail!` fail-closed, validated |
| cr-5 | None `actor_pseudonym` (unauditable chain entry) | **fix-in-pr** | FIXED — non-empty guard before EmitIntent, validated |

CR caught **two genuine pre-pilot bugs** (cr-4 fail-open, cr-5 ADR-016/015 contract gap) — high signal on the bridge code. cr-2/cr-3 are the expected scaffold-grade limitations the plan flagged for Phase-6.

## §3 Per-task metrics (Tier-1 signals per `feedback_retro_task_complexity_score.md`)

`<files>/<commits-incl-fix>/<fix-impl-cycles>`

| Task | files | commits | fix-impl | notes |
|---|---|---|---|---|
| T1 RoomEventPayload (crates, Win) | 1 | 1 | 0 | clean first-try |
| T2 S3 config + flag helpers | 2 | 3 | **2** (fix-impl 1: forward-declared `#[allow(dead_code)]`; 1b: InsertForm 4-field default propagation to test helper) | forward-declared + struct-field class |
| T3 recording primitives + rust-s3 | 1+lock | 2 | **1** (fix-impl 3: forward-declared `compute_content_sha256`/`RecordingSink` `#[allow]`) | same class; rust-s3 clean |
| T4 flag-gated emission + DTO mirror | 3 | 2 | **1** (clippy `too_many_arguments` 8/7 on `maybe_record`) | NEW class (arity, not forward-declared) |
| T5 participant-floor fetch | 2 | 1 | 0 | clean first-try (§2.5 gate held) |
| T6 docker-gated ITC stub | 1 | 1 | 0 | clean first-try (§2.5 gate held) |
| fix-in-pr cr-4+cr-5 | 2 | 1 | 0 | callsite-enumeration (§2.1) held — no `unused_must_use` |

**3× fix-impl recurrence (T2 ×2, T3 ×1) all forward-declared / struct-field-propagation class** — the codified `feedback_forward_declared_items_need_allow_until_consumer.md` + template §2.5 gate. The gate, hardened mid-phase (commit `ac5daa1a9`), HELD for T5/T6 (both clean). T4's `too_many_arguments` is a distinct 4th class (arity, not forward-declared).

User touchpoints: **3 gates** (plan-approval, cr-triage, merge-confirm) + phase-transition. e2e gate (4) N/A (binary DTO, no e2e track). 7 resume cycles (wakeup-driven validation polling).

## §4 What worked well

- **`rust-s3` 0.34 resolved clean on Linux first-try** — the headline-risk row (§18 row-1: "new S3 dep fails Linux Cargo.lock resolution") did NOT fire. The Task-3-isolates-the-dep + `validate-pending-laptop-linux`-proves-resolution design paid off; the `aws-sdk-s3` fallback was never needed.
- **§2.5 forward-declared gate held for T5/T6.** After 3 fix-impl cycles on T2/T3, the gate was hardened (`ac5daa1a9`) and the brief §4.4 clippy-arity pre-warning was added; T5 + T6 shipped clean first-try. The recurrence was front-loaded, not spread across the whole phase.
- **cr-5 callsite-enumeration (§2.1 in the fix-in-pr brief) held.** The `()→Result<()>` signature change propagated `?` to exactly 2 callsites (enumerated in the brief); clippy `-D warnings` came back clean — no `unused_must_use` §G4 cycle. The enumerate-all-callsites discipline did its job.
- **`/brehon-verify` structural pass without cargo re-run.** All 5 story checkpoint tests + §15.5 ADR-pin greps were already GREEN across the per-task `-linux` gates + fix-in-pr gate; verify did the cheap structural/phantom check (read-only `git archive` on the daemon) instead of re-running ~30 min of cargo.
- **CR signal quality high on bridge code** — 2/5 findings were real fail-closed/ADR bugs, fixed before merge.

## §5 What went wrong / lessons

### L1 — merge-forward runlog conflict (add/add) — recurrence
PR #205 went `CONFLICTING/DIRTY` at the merge gate. Sole conflict: `.claude/runlog/m3-core-recording-runlog.md` add/add (daemon's bm-poll-cr created a runlog on gov-v0; the phase branch's bm-cut/bm-pr also wrote one). **No code conflict.** Resolved by union. This is the same class as `m3-core-emergency-mute-retro.md` L5 (runlog append-append) — it recurs every phase where bm-poll-cr finalizes a runlog onto trunk while the phase branch has its own. **Carry-forward action:** consider giving the daemon-side bm-poll-cr runlog a distinct filename (e.g. `m3-core-recording-runlog-trunk.md`) OR always author the phase runlog only on the phase branch and let the daemon append, never create. Promote to a feedback lesson if it recurs a 3rd time.

### L2 — DQ-mutation-in-Mode-B when daemon checkout is ON the phase branch
The fix-in-pr DQ resolve had to go onto the phase branch, but the daemon checkout was on `phase-m3-core-recording`, so `git fetch origin X:X` refused ("checked out at") and `git checkout` is forbidden (cross-lane hard refusal). **Resolution:** mutate the DQ in the **detached-HEAD throwaway worktree** at the phase tip + `git push origin HEAD:phase-m3-core-recording` (refspec push from detached HEAD). Clean lane-safe path; daemon checkout never touched. Daemon-local ref left 1-behind (harmless — no Junior dispatch remained). Worth a one-liner in `multi-lane-mechanics.md`: "DQ mutation onto a phase branch the daemon has checked out → detached-HEAD throwaway-worktree + refspec-push, never daemon checkout switch."

### L3 — T4 `clippy::too_many_arguments` (8/7) on `maybe_record` — a 4th fix-impl class
The 3× forward-declared recurrence was codified, but T4 added a NEW class: an 8-arg emit-contract fn tripped clippy's 7-arg ceiling. Fixed with `#[allow(clippy::too_many_arguments)]` (commit `1a99bf6f7`). The T5/T6 briefs got a §4.4 pre-warning and didn't recur. **Retro-watch (carried from the auto-handover):** if `too_many_arguments` recurs on another forward-declared/emit-contract fn, extend §2.5 Trigger B to cover >7-param forward-declared fns + add a §G4 allowlist row for the lint.

### L4 — canonical-checkout stale after daemon finalize-merge to gov-v0 (recurrence, from prior session)
After a daemon bm-poll-cr finalize advanced origin gov-v0, the laptop canonical was stale and a brief push was rejected non-fast-forward. Fixed with fetch+rebase. First flagged in eval 1053 #1 last session; recurred conceptually this session as the merge-forward (trunk advanced past the phase base). **Promote to a feedback lesson** (`feedback_canonical_stale_after_daemon_finalize.md`): in Mode-B, after ANY daemon finalize-merge to gov-v0, fetch+rebase the canonical before the next mid-phase trunk commit.

## §6 Carry-forwards

1. **cr-2 + cr-3 → Phase-6** (live session-auth): requester-pseudonym binding + participant-set fetch. The scaffold returns 403 for all fetches until wired; the floor is correct-by-default.
2. **Recording live trigger → Phase-6**: `LiveSink::trigger_egress`/`upload` are now fail-closed `bail!` stubs; Phase-6 implements the real Egress POST + S3 PUT. The `tests/recording.rs` `#[ignore]` scenarios become the live e2e then.
3. **L1 runlog merge-forward conflict** — distinct daemon/phase runlog filenames OR phase-branch-only authorship (promote to lesson if 3rd recurrence).
4. **L3 `too_many_arguments`** — extend §2.5 Trigger B + §G4 allowlist if it recurs.
5. **L4 canonical-stale-after-finalize** — promote to `feedback_canonical_stale_after_daemon_finalize.md`.

## §7 ADR compliance check

- **ADR-015 (pseudonymity):** ✓ `speakers` + `actor_pseudonym` are pseudonyms (no `person_id`/MXID in recording.rs/stage.rs); fetch calls `is_participant` BEFORE serving; cr-5 hardened the non-empty actor guard.
- **ADR-016 (recording-event contract, metadata-only):** ✓ payload = `{media_url, content_sha256, duration_s, speakers, attendance_count}` + `actor_pseudonym`; MP4 bytes → S3, never the chain.
- **ADR-008/016 (`content_sha256` rides the chain):** ✓ R11 — hash flows EmitIntent → drain_emits → post_room_event → append_room_event; no bypass digest write in recording.rs.
- **No new entry-kind / registry count unchanged (72):** ✓ emit-only.
- **No migration / no new column:** ✓ flag rides existing `recording_config` column.

## §8 Open questions / phase-transition notes

- **M3-core status after this ship:** entry-kinds (PR#200), stage-mode (PR#202), emergency-mute (PR#204), **recording (PR#205)** all merged. Check whether M3-core is now complete or has remaining sub-phases before `/brehon-phase-transition`.
- **Copilot review gap (surfaced this session):** Copilot stopped reviewing (no minutes) between PR#202 and #204; CodeRabbit is now the sole automated reviewer. MiniMax is wired for the comparator/cheap-arm, NOT as a PR reviewer — a MiniMax-as-Copilot-replacement second reviewer is NOT built (no workflow, no script, no homeserver service). Open question for the user: build it (GH Action `pull_request` → `minimax-api.sh` semantic-reviewer → PR comment → teach bm-poll-cr a `source: minimax`) or accept CodeRabbit-only.

## §9 Auto-phase reliability (10-category, per `feedback_auto_phase_retro_signals.md`)

1. **Stage-transition correctness:** ✓ all transitions fired correctly; merge-forward conflict handled inline at the merge gate (not a state-machine miss).
2. **Cadence calibration:** ✓ validation polls (270s/120s/540s) tracked the bg cargo durations well; no over-polling.
3. **Auto-state integrity:** ✓ digest ring at 12 with 2 spills to JSONL; tip + DQ-pending fields stayed accurate; gate-5 backfilled into history at retro time (minor — should be written at gate-clear, not retro).
4. **Touchpoint count vs target:** 3 gates + transition (target 6–8 total) — under target (no e2e gate, clean planning).
5. **Catch-fire FP/FN:** 0 catch-fires; none warranted.
6. **§G4 classifier accuracy:** N/A this phase (no validate-fail; the 3 fix-impls were per-task `-linux` fails handled by the per-task gate, not §G4 — all forward-declared allowlist class).
7. **L14/L15/L16 fixes holding:** ✓ L15 (advisor-inline merge-gate checks + `gh pr merge`), L16 (branch deletion verified — `--delete-branch` worked). L14 N/A (advisor-inline merge, no BM runlog brief).
8. **Subagent offload:** not used this phase (resumes were wakeup-driven validation polls, not cold-context reconciliation — the offload is resume-from-compaction specific).
9. **Plan §13 fidelity:** ✓ 6 tasks → 6 impl cohorts (all size-1, no `[P]`); matched plan exactly.
10. **Resume-cycle pain:** 7 resumes, all clean; the EXIT-echo-trust discipline (read CHECK/CLIPPY/TEST*_EXIT from the spill log, not the bg-completion notification) held across every validation tick.

## §10 Confidence score (post-mortem)

**0.92** — clean ship, all ADRs satisfied, 2 real CR bugs caught + fixed, the headline dep-risk didn't fire. Minor friction (runlog merge-forward L1, DQ-mutation-Mode-B L2) was handled lane-safely without incident. The 3× fix-impl recurrence is a known, codified, front-loaded class — not a surprise.
