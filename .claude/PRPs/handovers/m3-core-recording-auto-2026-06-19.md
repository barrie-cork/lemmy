---
phase: m3-core-recording
kind: auto-phase-handover-refresh
stage: impl-fix-1-running
refreshed_at: 2026-06-20T00:22:00Z
---

# m3-core-recording — auto-phase handover (auto-refreshed)

**Stage:** `impl-fix-1-running` · **Phase branch tip:** `317e500aa` · **Trunk:** governance-v0 @ `13794fc47`

## Next concrete action (re-verify on resume)

Poll fix-impl-1 **#738** (base phase `317e500aa`). On `done`:
1. Read its new `validate-pending-laptop-linux` DQ.
2. **Run the cargo LOCALLY** in a throwaway worktree on the new phase tip: `scripts/brehon/cargo-linux.sh check + clippy --no-deps -- -D warnings + test record_town_halls_flag_parse --manifest-path services/bridge/Cargo.toml`.
3. On GREEN (clippy now passes — the 3 `#[allow(dead_code)]` cleared the dead-code errors): Task 2's clippy gate is satisfied → **Cohort A complete** → advance to Task 3.
4. Mutate DQ `91c7d92933b0-001` → resolved/pass (Task 1 already validated GREEN); reconcile `4ee45cbb6d67-001` (Task 2's original fail, superseded by fix-impl's DQ).
5. **Task 3** (serial, requires Task 2): rust-s3 dep-add + recording.rs primitives — the HEADLINE `validate-pending-laptop-linux` gate (Linux Cargo.lock resolution risk; fallback aws-sdk-s3, surface DQ don't silently swap).

## State

- **Plan:** `.claude/PRPs/plans/m3-core-recording.plan.md` @ origin gov-v0 (7 tasks, complexity 3/10)
- **Auto-state:** `.claude/auto-state/m3-core-recording.json` (stage impl-fix-1-running, resume_count 4)
- **Validation results:** Task 1 crates check GREEN (22m38s cold); Task 2 bridge check GREEN but clippy `-D warnings` FAIL on 3 dead-code errors (forward-declared helpers). fix-impl-1 resolves.
- **In-flight Junior:** #738 (fix-impl-1, base phase 317e500aa)
- **Lane mode:** B · **Mode:** fully-gated (no --unattended)

## Cohort map (plan §13)

- Cohort A: Task 1 #736 (crates, GREEN) + Task 2 #737 (bridge, clippy-fail→fix-impl-1 #738). On fix green → Cohort A done.
- Task 3: serial, requires task 2 (rust-s3 dep + recording.rs) — HEADLINE -linux gate
- Task 4: serial, requires tasks 1+3 (maybe_record flag-gate + Stage::record_uploaded emit + bridge DTO mirror + clean-posture invariant; CONSUMES record_town_halls_enabled → clears the dead-code #[allow] naturally)
- Task 5: serial, requires task 3 (participant-floor fetch; CONSUMES read_recording_config)
- Task 6: serial-docker, requires tasks 3,4,5 (#[ignore] integration, compile-only)
- Task 7: retro

## Cross-session deps / tripwires

- **Root cause note (retro):** my Task 2 brief mirrored the `read_chair_id` helper pattern but those siblings ARE used by stage-mode; the recording helpers aren't used until Task 4/5 → should have required `#[allow(dead_code)]` in the Task 2 brief. Add to retro §What-to-change.
- **Masked-exit trap observed:** the T2 clippy bg-task notification reported "exit 0" but the log body showed 3 `error:` lines + "could not compile" — trusted the log body per cargo-output-capture rule. Watch for this on every bg cargo run.
- Task 3 = rust-s3 Linux Cargo.lock resolution (headline risk)
- content_sha256 MUST ride append_room_event (R11 catch-fire on bypass)
- record_town_halls=false clean-posture zero-side-effect (delete-the-gate check)
- bm-cut finalize hazard recovered earlier; daemon main checkout currently ON phase-m3-core-recording (clean) — merges done there directly are lane-safe while 0 workers running
- Remaining gates: e2e local-vs-dispatch / CR-triage / merge-confirm / retro-sign-off (gate-1 DONE)
