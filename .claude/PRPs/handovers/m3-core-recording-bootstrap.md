---
phase: m3-core-recording
plan: .claude/PRPs/plans/m3-core-recording.plan.md   # not yet authored
phase_branch: phase-m3-core-recording                 # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork        # canonical until bm-cut; Mode B unless a lane worktree is created
lane_mode: B    # B = mobile remote-control (drive from canonical via Junior dispatch); flip to A if a dedicated brehon-fork-m3-core-recording worktree is created at bm-cut
authored: 2026-06-19
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the m3-core-recording advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon m3-core-recording** (M3 town halls, Phase 5, bridge-side). This is a fresh session. The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work (Mode B), or `C:/Users/barri/Developer/brehon-fork-m3-core-recording` once `bm-cut` creates the lane worktree if Mode A is chosen.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane (expect: `governance-v0`, single canonical worktree).
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `3b0f6f1c1` (see §"Git state at handoff"); if drifted, `git -C C:/Users/barri/Developer/brehon-fork log --oneline 3b0f6f1c1..governance-v0` and update your mental model before acting.
3. Read `.claude/decision-queue.json` pending entries (compare against §"Decision-queue snapshot" below — empty at handoff).
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_m3_core_recording.md` is the running-state scratchpad. Read `workflow_state_m3_core_emergency_mute.md` (CLOSED) ONCE for carry-forward.
5. `mcp__junior-brehon__list_hooks` — verify Telegram completion hook (ID 1) is present; recreate if absent after daemon restart (per `feedback_daemon_telegram_completion_hook.md`).

## Next concrete action

Author `.claude/PRPs/briefs/m3-core-recording-planning-1.md` (scope per `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` §"Phase 5: M3-core recording") → `/brehon-clarify` → queue planning Junior → `/auto-phase m3-core-recording`.

---

## 1. m3-core-recording in one paragraph

Phase 5 of M3 delivers **optional evidentiary-store recording** for town-hall rooms, gated on `record_town_halls=true` (default false). Scope: LiveKit Egress → MP4 → MinIO sidecar → `content_sha256` → callback into bridge binary `append()` for `Room::RecordingUploaded` chain entry. Participant-floor authorization on fetch (D5 Option C floor — can't be zero under `always_pseudonym`; strict presigned-URL ACL deferred as additive upgrade). A clean-posture town hall with `record_town_halls=false` must produce zero recording side-effects. DoD: recording lands with its chain hash entry; `record_town_halls=false` produces no artefact; non-participant fetch rejected. Bridge-side (`services/bridge/**` + `services/bridge/Cargo.toml` pulling LiveKit Egress + MinIO S3 deps), Linux-compile only.

**OQ-V2-04 resolution block** must be added to `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` at plan time (and the umbrella PRD OQ-V2-04 row updated) — it has no `### OQ-V2-04` heading yet. Model on OQ-V2-10 resolution-block format.

## 2. Why m3-core-recording is easier/harder than m3-core-emergency-mute

- **Easier:** the chain-emission seam is fully established (5 entry kinds already emitting); `append()` is proven. `record_town_halls` follows the existing `rtc_enabled` / `messaging_enabled` flag-gate pattern. No cross-instance complexity — recordings stay instance-local (OQ-V2-07 resolved: per-instance cost model).
- **Not easier / harder:** **new external services** — LiveKit Egress + MinIO S3 are net-new to the bridge's dependency surface. `Cargo.toml` changes → `validate-pending-laptop-linux` gate is MANDATORY (DQ kind `validate-pending-laptop-linux`; triggers diff-check on Cargo.toml). MinIO containerisation adds a new sidecar to the deployment topology. The `content_sha256` chain-hash must be tamper-evident — it needs the `ed25519-dalek` signing path already in the governance log, not a new ad-hoc hash. Authorization on fetch is a new governance-plane surface (participant-floor check). **Licensing:** both LiveKit Egress (Apache-2.0) and MinIO (AGPL-3.0) are licence-clean for embedding (per ADR-011 notes) — but verify at `Cargo.lock` resolution time.

## 3. Lessons from m3-core-emergency-mute that apply to m3-core-recording

**Advisor-side:**
- `feedback_advisor_watchpoint_specificity.md` — every watchpoint in plan §4 MUST cite a specific file/table/line. No concept-only watchpoints.
- `feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md` — run every §15 DoD command literally after planning ships; bridge DoD must use `cargo-linux.sh`, never Windows-local.
- `feedback_bridge_validates_on_linux_not_windows.md` — bridge compiles Linux-only; `ruma-common` E0119 on Windows. All bridge cargo via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`.
- `feedback_finalize_merge_where_to_look_first.md` — after BM tasks, check daemon-local gov-v0 tip first, then origin; push daemon ref before pulling locally.

**Impl-side:**
- **`--bins` bare fn name** (L1, recurrence-2 — PROMOTE THIS PHASE): for bridge test filters, use bare fn name (e.g. `recording_lands_with_hash`), NOT `module::tests::recording_lands_with_hash`. Add to every bridge impl-task brief §4 Constraints.
- `feedback_lemmy_error_no_std_error.md` — Case A/B for any test returning `Result<(), Box<dyn Error>>`.
- `feedback_multi_write_handlers_need_transactions.md` — if any handler does 2+ DB writes (recording callback → append chain entry), use `conn.run_transaction()`.
- `feedback_governance_type_state_handlers.md` — handler for `RecordingUploaded` should guard on case status before appending (type-state pattern).
- `feedback_linux_compile_proof_is_a_gate.md` — `validate-pending-laptop-linux` DQ MANDATORY when Cargo.toml/Cargo.lock changes (this phase definitely will).

**When bundling clippy-debt fix into a feature PR (L4):** budget one fix-impl cohort (~80 min) for CR to surface adjacent outside-diff debt in the same files. This phase will touch `sanction_handler.rs` area again — CR will likely find more `.ok()` patterns. Pre-empt: fix known ones before the PR.

**BM-side:**
- Mode B brief visibility: `git checkout origin/governance-v0 -- <brief-files>` onto phase branch. Never full `git merge` (roadmap conflict).
- Runlog append-append conflict on merge-forward is routine — resolve with keep-both-sides regex.

## 4. m3-core-recording-specific watchlist

1. **`Cargo.toml` / `Cargo.lock` changes → Linux-compile gate is mandatory.** Trigger: `diff origin/governance-v0 phase-m3-core-recording -- services/bridge/Cargo.toml services/bridge/Cargo.lock` returns non-empty. Gate: `validate-pending-laptop-linux` DQ entry per task that changes deps; `scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml`. Do NOT skip even if Windows check passes.

2. **`content_sha256` must use the governance log's existing `ed25519-dalek` signing path, not a bare SHA256.** Check `services/bridge/src/governance_log.rs` (or equivalent) for the `append()` fn signature and confirm the recording callback feeds through it — not a raw `sha2::Sha256::digest` write that bypasses the chain. Structural pattern to assert in §16a: `append(RecordingUploaded { content_sha256: ... })` appears in the callback path.

3. **`record_town_halls=false` clean-posture test is mandatory.** A town hall running without the recording flag must produce zero artefacts in MinIO and zero `RecordingUploaded` chain entries. DoD story must include a negative-invariant test (cf. `mute_all_revokes_all_publishers` zero-holder precedent from Phase 4).

4. **Participant-floor authorization on fetch.** The fetch endpoint must reject non-participant requests. The `actor_pseudonym` table (ADR-015) is the authorization source — check that the fetch handler calls `validate_identity_policy` or equivalent before serving the presigned URL. Brief §4 must make this ADR-015-pinned gate load-bearing (per `advisor-orchestrator.md` §2.4a).

5. **MinIO sidecar deployment topology.** `services/bridge/Cargo.toml` pulling the `aws-sdk-s3` (or equivalent S3 client) crate is a new dep surface. Confirm the bridge speaks generic S3 API (not MinIO-specific) so an operator can swap for real S3/R2 (per D5 design decision). Watchpoint: `grep -r "minio\." services/bridge/src/` — any hardcoded MinIO hostname is a scope violation.

6. **OQ-V2-04 resolution block.** Before plan authorship, confirm `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` gets a `### OQ-V2-04` heading added in this phase's commit. The PRD §"Phase 5" scope note flags this as pending at plan time. Stop and ask if the planner omits it.

## 5. Operational rules

- **Polling cadence:** ~10 min `mcp__junior-brehon__list_tasks`; on transition `show_task + git fetch + read DQ` → triage → queue next.
- **Brief discipline:** briefs at `.claude/PRPs/briefs/m3-core-recording-<role>-<n>.md`, committed to governance-v0 first; pre-queue `memory_search_hybrid` + `/precheck` (Check 3b: daemon-local trunk sync); §2.4 mandatory file-class lesson injection fires on: any `Cargo.toml` edit → `feedback_linux_compile_proof_is_a_gate.md`; any bridge test → `feedback_lemmy_error_no_std_error.md` + bare-fn-name constraint; any 2+ DB write handler → `feedback_multi_write_handlers_need_transactions.md`.
- **Shape G RESIDUAL-ONLY:** `cargo-validate-workspace.yml` disabled at GH API; impl-task workers write `validate-pending-laptop[-linux]` DQ and STOP (per `feedback_validate_pending_laptop_write_then_stop.md`). Advisor runs cargo locally. Bridge tasks use `validate-pending-laptop-linux` (Docker `rust:1.95`); crates tasks use `validate-pending-laptop` (Windows).
- **Windows e2e:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full ..."` — never bare `cargo test`; never `-p lemmy_server --features full` (per `feedback_windows_e2e_requires_bat_wrapper.md`).
- **`--bins` bare fn name (ENFORCE THIS PHASE):** for all bridge test filters, use bare fn name only. Add explicitly to every bridge impl-task brief §4 Constraints.
- **Cross-lane cap:** max 2 concurrent Junior tasks (shared `.git/index.lock`).
- **Model tiering:** Planning → Opus/xhigh; Impl → Sonnet/medium; BM/ci-watcher → Haiku/low.
- **DQ attribution:** `chore|docs(advisor|decision-queue):` commit subject for all DQ writes.
- **6 mandatory user gates:** plan-approval / ADR-DQ / CR-triage / e2e local-vs-dispatch / merge-confirm / retro-sign-off. Never skipped.
- **ADR-015 load-bearing:** fetch handler MUST call `validate_identity_policy` (or the participant-floor equivalent); brief §4 must make this explicit with DoD grep (per `advisor-orchestrator.md` §2.4a).

## 6. What changed from m3-core-emergency-mute's rule set

1. **`--bins` bare fn name is now an explicit brief-template requirement** (was a lesson; promoted to constraint after recurrence-2 in emergency-mute).
2. **`validate-pending-laptop-linux` is ALWAYS required** when `Cargo.toml`/`Cargo.lock` change — this phase will add S3 client dep (was optional in prior phases that didn't touch bridge deps directly).
3. **Participant-floor authorization gate (ADR-015 pin)** is new for this phase — the fetch endpoint is the first governance-log-adjacent surface that needs it in the bridge.
4. **OQ-V2-04 resolution block must land in this phase's commit** — not carried forward again.

## 7. Catch-fire procedures

Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.5:
- Junior subagent ignores hard refusals / commits to governance-v0 directly
- Attribution breach (`answered_by: "advisor"` in non-`chore(advisor)` commit)
- Phase branch has uncommitted state when Junior reports complete
- ci-watcher timeout >60 min
- §G4: 3+ cycles same `(error_class, file_basename)` → HARD REFUSAL regardless of allowlist

Phase-specific additions:
- **`content_sha256` bypasses `append()` chain path** — if impl-task writes a raw `sha2::Sha256::digest` call that doesn't flow through the governance log append path, catch-fire: this is an ADR-violation-class correctness issue.
- **`record_town_halls=false` produces artefacts** — if the clean-posture test is absent or passing trivially (always-skip), catch-fire: the flag-gate is load-bearing for non-recording operator deployments.
- **MinIO hardcoded hostname in bridge source** — catch-fire: operator swap-ability is a design decision (D5 Option C); hardcoding breaks it.

## 8. Archive after m3-core-recording

Run `/brehon-phase-transition m3-core-recording m3-core-e2e` (Phase 6: full acceptance + pilot). This skill will: close `workflow_state_m3_core_recording.md`, delete `workflow_state_m3_core_emergency_mute.md` (the two-ago), create the Phase 6 skeleton, write the next bootstrap, update brehon-fork MEMORY.md, commit + push to governance-v0. This file stays in `.claude/PRPs/handovers/` as its own archive.

---

## Git state at handoff (captured 2026-06-19)

- governance-v0 HEAD: `3b0f6f1c1` (captured 2026-06-19) — `docs(retro): m3-core-emergency-mute — mute-all shipped; L1 --bins bare-fn recurrence-2; L4 outside-diff CR clippy debt; L5 runlog append-append resolved`
- Phase branch HEAD: `not yet created (branch phase-m3-core-recording cut at bm-cut)`
- Recent governance-v0 commits:

  ```
  3b0f6f1c1 docs(retro): m3-core-emergency-mute — mute-all shipped; L1 --bins bare-fn recurrence-2; L4 outside-diff CR clippy debt; L5 runlog append-append resolved
  34f647ad9 Merge pull request #204 from barrie-cork/phase-m3-core-emergency-mute
  f3b6b24cb chore(merge): merge-forward governance-v0 into phase-m3-core-emergency-mute (pre-merge gate)
  20b5fa705 docs(advisor): brehon-verify m3-core-emergency-mute — all 4 stories pass; findings YAML counters updated (7 done, 1 rebut, 4 wont-fix)
  77538bb77 chore(decision-queue): advisor-laptop resolved validate-pending-laptop fef20f6c5226-001 (fix-impl-2 crates hook-errors green)
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff — all m3-core-emergency-mute DQ entries resolved)
```

## Stop-and-ask tripwires

- Stop and ask if: the plan introduces a `content_sha256` implementation that writes a raw digest without flowing through the governance log `append()` path — this breaks tamper-evidence and violates ADR-016.
- Stop and ask if: the plan has `record_town_halls=false` producing any MinIO writes or `RecordingUploaded` chain entries — the clean-posture requirement is a hard DoD invariant.
- Stop and ask if: the planner omits the OQ-V2-04 resolution block from the `99-decisions-and-open-questions.md` commit — it has been deferred twice and must land this phase.
- Stop and ask if: any bridge source file hardcodes a MinIO hostname (e.g. `minio:9000`) — operator swap-ability is a D5 decision; hardcoding breaks it.
- Stop and ask if: the fetch endpoint for recordings does not call a participant-floor authorization gate traceable to ADR-015 — this is a mandatory governance guard, not an optional nicety.
