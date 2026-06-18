---
phase: m3-core-stage-mode
plan: .claude/PRPs/plans/m3-core-stage-mode.plan.md   # (not yet authored)
phase_branch: phase-m3-core-stage-mode                 # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork         # canonical until bm-cut; lane worktree optional
authored: 2026-06-18
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the m3-core-stage-mode advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon m3-core-stage-mode (M3 town halls Phase 3, bridge-side).** This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-<lane>` once `bm-cut` creates a lane worktree (per `.claude/rules/multi-lane-worktree.md`). There is no homeserver session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane. Should be single canonical worktree on governance-v0 (m3-core-infra's throwaway worktrees were cleaned at transition).
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — should equal `eb1bcaca0` or later (see §"Git state at handoff"); if drifted, `git log --oneline eb1bcaca0..governance-v0` and update your mental model before acting.
3. Read `.claude/decision-queue.json` for any pending entries since handoff; compare against the §"Decision-queue snapshot" below (empty at handoff).
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_m3_core_stage_mode.md` is the running-state scratchpad. Read `workflow_state_m3_core_infra.md` (CLOSED) ONCE for carry-forward, then don't re-read.
5. **Daemon hooks:** `mcp__junior-brehon__list_hooks` — recreate hook ID 1 (Telegram completion) if absent (daemon restarts wipe hooks). Per `feedback_daemon_telegram_completion_hook.md`.

## Next concrete action

No plan exists yet. Author `.claude/PRPs/briefs/m3-core-stage-mode-planning-1.md` (scope per `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` §"Phase 3: M3-core stage-mode" + the OQ resolutions noted in §1 below) → run `/brehon-clarify .claude/PRPs/briefs/m3-core-stage-mode-planning-1.md` → resolve clarify-DQ → queue planning Junior. Then `/auto-phase m3-core-stage-mode`.

---

## 1. m3-core-stage-mode in one paragraph

**M3 town halls Phase 3, bridge-side.** Goal: chair-controlled stage mode with mic-passing. Scope (PRD §Phase 3): stage-mode room provisioning; dual-sourced chair seat (OQ-V2-05 resolved); FIFO raised-hand queue; mic-passing with **30s grace + auto-revoke + next-promote**; chair override (force-demote/promote); Q&A text sidebar; emission of the `room_chair_transferred` + `room_chair_override` chain entries (the consts m3-core-entry-kinds pre-registered — Phase 3 is their FIRST emitter). **DoD / success signal:** bridge integration test passes **4 mic-passes in sequence** + the **30s no-activate boundary** (auto-revoke fires when a promoted speaker doesn't activate within 30s). This is the first phase to EMIT M3 chair entry kinds — m3-core-infra built the deployable RTC stack, entry-kinds registered the strings, stage-mode wires the chair control loop that produces them.

## 2. Why m3-core-stage-mode is easier/harder than m3-core-infra

- **Easier:** the RTC stack is already deployable + optional (m3-core-infra shipped it); the chair/mute entry-kind consts already exist (`ENTRY_KIND_ROOM_CHAIR_TRANSFERRED`, `_CHAIR_OVERRIDE` — entry-kinds Phase 2); `bridge_room` already carries `chair_id`/`queue_state`/`recording_config` columns (m3-core-infra Task 2). The schema and infra plumbing is done — Phase 3 is logic on top.
- **Harder / NOT easier:** this is **bridge-side application logic**, not Lemmy crates — the work lands in `services/bridge/src/**` (Rust, compiles + tests on **Linux only** via `cargo-linux.sh`, NOT Windows — `ruma-common` E0119 on Windows host). It's stateful (FIFO queue, grace timers, chair-seat transitions) — the type of logic where `feedback_build_what_tests_exercise` + state-machine type-state patterns matter. The 30s grace/auto-revoke is a timing concern; the integration test must exercise the boundary, not just the happy path. First real emission of chain entries from the bridge → the `append()` path must be exercised, not stubbed.

## 3. Lessons from m3-core-infra that apply to m3-core-stage-mode

Reference by filename — never duplicate.

- **Advisor-side:** `feedback_fix_impl_workers_skip_validate_pending_dq.md` — the DOMINANT carry-forward. Fix-impl workers skip the validate-pending DQ-write (2× in m3-core-infra). A structural finalize-gate shipped in `.claude/agents/impl-task.md` (commit `eb1bcaca0`), but it's a contract nudge, not daemon enforcement: **after ANY fix-impl reports done, verify the fix landed via DoD grep on the phase tip + run the validation directly — do NOT block on the validate DQ surfacing.** Also `feedback_verify_automated_reviewer_claims_against_compiler.md` + `feedback_falsifiable_hypothesis_before_structural_fix.md` (m3-core-infra rebutted CR cr-3 ADR-015 false-positive with code evidence — apply to every CR trait/type/ADR claim).
- **Bridge-side (impl):** `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs via `cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker rust:1.95), never Windows-local. `feedback_linux_compile_proof_is_a_gate.md` — any `services/bridge` code change raises `validate-pending-laptop-linux`. `feedback_governance_type_state_handlers.md` — the chair-seat state transitions are a type-state candidate (phantom-typed `Stage<S>`).
- **Process:** `feedback_task_notification_exit_summary_unreliable.md` — daemon `status` field lags actual completion; verify via log tail / phase-tip advance (re-observed in m3-core-infra §3.2). `pattern_test_against_reality_not_syntax.md` — m3-core-infra's deploy-smoke caught 2 config bugs a lint would miss; the stage-mode integration test must actually run the mic-pass sequence, not assert on config.
- **ADR pins:** `room_chair_transferred`/`room_chair_override` chain entries are metadata-only (content inside rooms NEVER hashed — ADR-016 backplane contract, roadmap §M3 scope line). Jury rooms pinned `always_pseudonym` (ADR-015).

## 4. m3-core-stage-mode-specific watchlist

Each cites a specific file/symbol + a forward gate.

1. **Chair-seat dual-source (OQ-V2-05).** The plan §13 must name HOW the chair seat is dual-sourced (governance-assigned vs self-claimed) — cite the exact field on `bridge_room` (`chair_id` exists; how is it populated?). Gate: plan must reference `services/bridge/src/bridge_room.rs` chair_id column + the assignment path.
2. **FIFO raised-hand queue persistence.** `bridge_room.queue_state` column exists (m3-core-infra). The plan must say whether the FIFO queue lives in that column (persisted) or in-memory (lost on bridge restart). Gate: plan §4 must cite `queue_state` and state the persistence model.
3. **30s grace auto-revoke timing.** The DoD success signal is the "30s no-activate boundary." The integration test MUST assert the auto-revoke fires AND the next-promote happens. Gate: the first test must assert on the timer boundary, not just a successful promote — cite the test fn name in plan §16a.
4. **Chain-entry emission is the FIRST emitter.** `room_chair_transferred`/`room_chair_override` consts exist but had NO emitters before this phase (entry-kinds was pre-landing). The plan must wire the bridge `append()` call for both. Gate: `grep ENTRY_KIND_ROOM_CHAIR_TRANSFERRED services/bridge/` must return BOTH a definition reference AND an emit callsite after impl (mirror the m3-core-infra ADR-load-bearing-clause discipline).
5. **Bridge Linux-compile gate.** Every `services/bridge/src/**` change touches the Linux-only crate. Plan §15 bridge cargo commands MUST use `cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Windows form is a DoD-issue). Gate: §3.4 DoD smoke runs bridge commands via Docker.
6. **Cargo.lock sync with direct-dep additions.** m3-core-infra §3.4 loose end: a `Cargo.toml` direct-dep add shipped without its lockfile entry. If stage-mode adds a bridge dep, the lockfile sync must land in the SAME commit. Gate: watch any `services/bridge/Cargo.toml` change for a paired `Cargo.lock` diff.

## 5. Operational rules

Polling cadence ~10 min (`mcp__junior-brehon__list_tasks`, status only). Briefs at `.claude/PRPs/briefs/m3-core-stage-mode-<role>-<n>.md`, committed to governance-v0 first (Mode-B sync into phase branch for impl briefs). Pre-queue: `memory_search_hybrid` + `/precheck` + §2.4 mandatory file-class lesson injection (for `services/bridge/**` → inject the bridge-Linux + linux-compile-gate lessons). **NO-CARGO-ON-ELITEDESK** (workers write `validate-pending-laptop[-linux/-e2e]` DQ + STOP; laptop runs all cargo). **Bridge cargo = `cargo-linux.sh` (Docker rust:1.95), Linux-only.** Shape G RESIDUAL-ONLY (public green-check only; `validate-pending-laptop-linux` via Docker is the real internal coverage — `project_shape_g_suspended_2026_05_16.md`). Model tiering: Planning→Opus/xhigh, Impl→Sonnet/medium, BM/ci-watcher→Haiku/low. Clarify gate before planning. The 6 user gates non-skippable; advisor never auto-merges or auto-resolves ADR-affecting DQ. DQ attribution `chore|docs(advisor|decision-queue):`. Memory headroom: no bulk `e2e.rs` reads. Merge with merge-commit, NOT squash (task-per-commit history load-bearing for retros). Run via `/auto-phase m3-core-stage-mode`.

## 6. What changed from m3-core-infra's rule set

- **NEW (this phase, shipped at m3-core-infra retro):** `.claude/agents/impl-task.md` now has a **finalize-gate** making the validate-DQ write a blocking pre-exit check (self-check `git log -3 --stat | grep -q decision-queue.json` + "push DQ before retro" ordering). Applies to fix-impl explicitly. BUT it's a contract nudge — the advisor verify-and-run-directly mitigation stays load-bearing.
- **Domain shift:** m3-core-infra was Lemmy-crates + bridge mixed; m3-core-stage-mode is **bridge-side only** (`services/bridge/src/**`). The Linux-compile gate fires on EVERY task (not diff-scoped — all bridge code is Linux-only). The e2e-on-Lemmy-crates rules (`feedback_lemmy_error_no_std_error`, e2e.rs anchors) mostly don't apply; bridge integration tests are the validation surface.
- **No new migration expected** (m3-core-infra added the `bridge_room` RTC columns; stage-mode uses them). If the plan introduces a new `services/bridge` migration, that's a scope expansion — surface it.

## 7. Catch-fire procedures

Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.6 (subagent hard-refusal, attribution breach, non-allowlist §G4 fail, ci-watcher exit-code surprise, daemon down, commit-to-trunk). Phase-specific additions:
- **Daemon finalize-merge bypassing gates 3/5** — recurrence-3 happened at m3-entry-kinds (PR #200 auto-merged). After bm-pr, the advisor MUST verify PR is OPEN (not MERGED) AND origin trunk does NOT contain the phase tip BEFORE advancing to cr-wait (per `feedback_junior_finalize_merges_bm_cut_branch.md` preventative #2). m3-core-infra honoured gates correctly — keep the OPEN-PR check.
- **Bridge code committed but Linux-compile not proven** — any `services/bridge/**` change without a `validate-pending-laptop-linux` at result:pass before bm-pr → STOP (`bm-pr` Phase-1d gates on it).
- **Chair entry-kind emitted but `append()` stubbed** — if the impl wires the const but not a real chain emission, `/brehon-verify` phantom-catches it; the integration test must exercise the real append path.

## 8. Archive after m3-core-stage-mode

The standard close: run `/brehon-phase-transition m3-core-stage-mode m3-core-emergency-mute` (Phase 4 next). The skill will: close `workflow_state_m3_core_stage_mode.md`, delete the two-ago record (`workflow_state_m3_core_infra.md`), create the next skeleton, write the next bootstrap, update brehon-fork MEMORY.md, commit on governance-v0. This bootstrap stays in `.claude/PRPs/handovers/` as its own archive — git history is the archive; no move.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `eb1bcaca0` (captured 2026-06-18) — `feat(impl-task): finalize-gate the validate-DQ write (fix-impl skip fix)` (the transition commit adds the DQ-resolve + this bootstrap on top; next session should see HEAD = transition commit or later)
- Phase branch HEAD: not yet created (branch `phase-m3-core-stage-mode` cut at bm-cut)
- Recent governance-v0 commits (`git log --oneline -5 governance-v0`):

  ```
  eb1bcaca0 feat(impl-task): finalize-gate the validate-DQ write (fix-impl skip fix)
  5df2b23d9 docs(retro): m3-core-infra — RTC stack deployable + optional; fix-impl validate-DQ skip lesson
  ba4805a3b chore(bridge): sync Cargo.lock for ed25519-dalek/hex direct deps
  1fb97d30c docs(retro): m3-entry-kinds plan — mark promotion candidates #1+#2 shipped
  c3a13570b Merge pull request #201 from barrie-cork/phase-m3-core-infra
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff)
```
The orphan cr-4b `validate-pending-laptop-e2e` (`4f481a50a459-001`) was resolved at transition — e2e passed, validated directly by advisor-laptop, PR #201 merged. No carry-forward DQ.

## Stop-and-ask tripwires

- Stop and ask if: the plan introduces a new migration under `services/bridge` or `crates/db_schema/migrations/**` — m3-core-infra already added the `bridge_room` RTC columns; a new migration is a scope expansion for a logic-only phase.
- Stop and ask if: the plan proposes Lemmy-crate (`crates/**`) changes beyond a trivial DTO — Phase 3 is bridge-side; substantial Lemmy work means the scope drifted.
- Stop and ask if: the integration-test success signal does NOT assert on the 30s grace/auto-revoke boundary — the DoD requires the timing boundary, not just a successful mic-pass.
- Stop and ask if: a `services/bridge/**` change is queued for bm-pr without a `validate-pending-laptop-linux` at result:pass — the Linux-compile gate is mandatory for all bridge code.
- Stop and ask if: any new chair-control logic emits a chain entry via a stub/no-op rather than the real binary `append()` — the chain emission is load-bearing (ADR-016), not a placeholder.
