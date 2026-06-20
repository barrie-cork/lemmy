---
phase: m3-core-e2e-pilot
plan: .claude/PRPs/plans/m3-core-e2e-pilot.plan.md   # (not yet authored)
phase_branch: phase-m3-core-e2e-pilot                 # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork        # canonical until bm-cut; Mode B unless a lane worktree is created
lane_mode: B    # B = mobile remote-control (drive from canonical via Junior dispatch); flip to A if a dedicated brehon-fork-m3-core-e2e-pilot worktree is created at bm-cut
authored: 2026-06-20
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the m3-core-e2e-pilot advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon m3-core-e2e-pilot** (M3 town halls, **Phase 6 of 6** — the final M3-core phase: full town-hall acceptance e2e + a real pilot town-hall run end-to-end). This is a fresh session. The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work (Mode B), or `C:/Users/barri/Developer/brehon-fork-m3-core-e2e-pilot` once `bm-cut` creates the lane worktree if Mode A is chosen. There is no homeserver session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane (expect: `governance-v0`, single canonical worktree).
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `379b5446e` (see §"Git state at handoff"); if drifted, `git -C C:/Users/barri/Developer/brehon-fork log --oneline 379b5446e..governance-v0` and update your mental model before acting.
3. Read `.claude/decision-queue.json` pending entries (compare against §"Decision-queue snapshot" below — empty at handoff). Once a phase branch exists, use `scripts/brehon/resolve-dq-canonical.sh m3-core-e2e-pilot`.
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_m3_core_e2e_pilot.md` is the running-state scratchpad (most-recent "Session handoff block" is authoritative on resume). Read `workflow_state_m3_core_recording.md` (CLOSED) once for carry-forward.

## Next concrete action

Author `.claude/PRPs/briefs/m3-core-e2e-pilot-planning-1.md` (scope per `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` "Phase 6: M3-core e2e + pilot" + the §Success Criteria table, lines 132–149) → `/brehon-clarify` → queue planning Junior. **This phase has TWO halves with different shapes** (see §1): (a) the integration/acceptance e2e half — code, validate-pending gates, normal cohort flow; (b) the **real pilot town-hall run** (D2) — operational, not a Junior task; it's an operator action the advisor coordinates + records. The planning brief must scope both and flag that the pilot half is a human-run acceptance step, not impl.

---

## 1. m3-core-e2e-pilot in one paragraph

**Goal:** full M3 town-hall **acceptance** — turn the `#[ignore]` Docker-gated scaffolds shipped across Phases 1–5 into live integration tests that pass the PRD §Success Criteria, **then** run a concrete pilot town hall end-to-end (D2 — "the real acceptance signal"). This is Phase 6 of 6 in M3-core; Phases 1–5 (infra, entry-kinds, stage-mode, emergency-mute, recording) are all shipped + merged. **What's new vs Phase 5:** this is the *integration + pilot* phase, not a code-add phase — most RTC control logic is already on `governance-v0` as scaffold; Phase 6 wires the live LiveKit/Egress/MinIO sidecars + the cross-instance federation path, makes the e2e tests real, and proves it with a human-run pilot. **DoD:** all §Success Criteria integration tests pass (chair passes mic to 4 users in sequence; emergency mute drops all publishers including cross-instance **<500ms** measured at the publisher client; recording → MP4 → MinIO → `Room::RecordingUploaded` with `content_sha256` on chain; non-participant recording fetch rejected; both clean-posture cases `record_town_halls=false` → zero side-effects) **AND** a real pilot town hall runs end-to-end with the chair passing the mic (optional recording landing with its hash entry).

## 2. Why m3-core-e2e-pilot is easier/harder than m3-core-recording

- **Easier:** no new schema, no new entry-kind consts to register (the 3 chair/mute `Room::*` kinds + `ENTRY_KIND_ROOM_RECORDING_UPLOADED` all shipped in earlier phases — registry count is 72, do NOT add). No new Rust *features* — the control logic is scaffolded; Phase 6 mostly turns `#[ignore]`/`todo!()` stubs live and wires sidecars. The cr-2/cr-3/recording-trigger carry-forwards (live session-auth, real Egress POST + S3 PUT) are explicitly enumerated in `workflow_state_m3_core_recording.md` (CLOSED) §carry-forward + recording retro §6.
- **Not easier (this is the HARD phase of M3-core):** (a) the **federation-wide emergency-mute <500ms** criterion (PRD line 141/176) is the marquee risk — a distributed-systems problem (Matrix power-level propagation across two federated bridge instances), must be measured at the **publisher client**, not the server. (b) The **real pilot** (D2) is operational and can't be a Junior cargo task — it needs a running LiveKit+MinIO+two-bridge-instance stack and a human to chair. (c) e2e tests here are **testcontainers + Docker + LiveKit/MinIO sidecars** — heavier than the Phase-5 `#[ignore]` compile-gated stubs; the gate-4 (Phase-2 e2e local-vs-dispatch) decision is LIVE this phase (it was N/A in Phases 1–5, which were binary-DTO / unit-level).

## 3. Lessons from m3-core-recording that apply to m3-core-e2e-pilot

Reference by filename; do not duplicate.

- **advisor-side:** `feedback_advisor_watchpoint_specificity.md` (every watchpoint cites a file/test/line — see §4). `feedback_laptop_default_for_validate_pending.md` + `project_laptop_canonical_cargo_runner.md` (NO CARGO ON ELITEDESK — workers write `validate-pending-laptop[-e2e]` and STOP; laptop runs ALL cargo/e2e). `feedback_e2e_nextest_filter_groups.md` (the `-E` filter groups + `e2e_filter` in validate-pending — load-bearing now that e2e is the phase's centre of gravity). `feedback_canonical_stale_after_daemon_finalize.md` (recording L4 — Mode-B: after any daemon finalize-merge to gov-v0, fetch+rebase canonical before the next mid-phase trunk commit). `feedback_finalize_merge_where_to_look_first.md` (daemon-local-first, origin-second).
- **planning-side:** the recording plan's §18 risk-row design (isolate the new dep in one task + prove it via a `-linux` gate) worked — apply the same to the LiveKit/MinIO sidecar wiring (isolate sidecar bring-up; prove the e2e harness can reach the containers before the acceptance tests). `feedback_complexity_score_pre_split.md` — this phase's e2e edits will be ≥2 → expect the `feedback_fix_impl_pre_locate_e2e_anchors.md` injection.
- **impl-side:** `feedback_forward_declared_items_need_allow_until_consumer.md` + template §2.5 (the codified 3× recurrence — but most forward-decls are now consumed, so this should fire less). `feedback_lemmy_error_no_std_error.md` Case A/B (any new e2e test returning `Result<(), Box<dyn Error>>`). `feedback_async_pool_test_pattern.md`. The L2 lane-safe DQ-mutation path (detached-HEAD throwaway worktree + `git push origin HEAD:phase-branch` when the daemon checkout is ON the phase branch).
- **BM-side:** `feedback_bm_merge_unstable_admin_bypass.md`. The L1 runlog merge-forward conflict (add/add) — see §4 watchpoint; resolve by union, and consider distinct daemon/phase runlog filenames (recording retro §6.3 — promote to lesson if it recurs a 3rd time).

## 4. m3-core-e2e-pilot-specific watchlist

1. **`crates/server/tests/e2e.rs` AND `services/bridge/tests/recording.rs` (≥2 e2e edits expected):** the plan §13 e2e tasks MUST pre-locate verbatim Edit anchors per `feedback_fix_impl_pre_locate_e2e_anchors.md`; the uniqueness gate (`grep -c '<anchor>'` == 1) is mandatory before any e2e-edit brief is queued. The recording-phase shipped 3 `#[ignore]` scenarios in `services/bridge/tests/recording.rs` with `todo!()` bodies — those are the live-test targets.
2. **The <500ms emergency-mute test (PRD line 141):** the integration test MUST measure at the **publisher client across two federated bridge instances**, NOT at the server — a server-side measurement is a false green. The plan §16a story for this criterion must assert publisher-side drop timing + the chain entry `{chair_pseudonym, federated: true}`. Stop-and-ask if the plan measures mute latency server-side.
3. **`services/bridge` cargo runs LINUX-ONLY** (`scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`, Docker rust:1.95) per `feedback_bridge_validates_on_linux_not_windows.md` — Windows fails `ruma-common` E0119. Any §15 bridge command written in Windows form is a DoD-issue → file advisor DQ + ask planner to re-point. First Docker bridge run is cold (~10–20 min).
4. **Registry count stays 72** (`crates/db_schema/src/source/governance/governance_log.rs`): this phase emits/exercises existing kinds; it adds NONE. If any plan task proposes a new `ENTRY_KIND_ROOM_*` const, that's a scope violation — the 3 chair/mute kinds shipped in stage-mode/emergency-mute. Stop-and-ask.
5. **gate-4 (Phase-2 e2e local-vs-dispatch) is LIVE this phase** (first M3-core phase where it fires): the e2e suite needs Docker + LiveKit/MinIO testcontainers — heavier than prior phases. Surface the local-vs-dispatch choice ONCE per phase (never auto-pick after PR #105); local = bat-wrapper, ~26 min, zero billed; dispatch = billed + public log. Per `feedback_windows_e2e_requires_bat_wrapper.md`.
6. **The pilot (D2) is operational, not a Junior task:** it needs a running LiveKit+MinIO+two-bridge stack (pilot server `http://100.81.145.58:1236`, test accounts in `reference_pilot_test_accounts.md`). The plan must scope it as a human-run acceptance checklist the advisor coordinates + records in the retro — NOT an impl-task. Stop-and-ask if a plan task tries to "implement the pilot".

## 5. Operational rules

Polling cadence ~10 min (`mcp__junior-brehon__list_tasks`, status only). Brief discipline: briefs in `.claude/PRPs/briefs/m3-core-e2e-pilot-<role>-<n>.md`, committed to `governance-v0` first (Mode B); pre-queue `memory_search_hybrid` + `/precheck` + §2.4 mandatory file-class lesson injection (the e2e.rs / recording.rs rows fire hard this phase). LemmyResult Case A override for any e2e brief (`feedback_lemmy_error_no_std_error.md`). **Shape G RESIDUAL-ONLY** (`project_shape_g_suspended_2026_05_16.md`): workers write `validate-pending-laptop[-e2e][-linux]` DQ and STOP; the LAPTOP runs ALL cargo/e2e (`project_laptop_canonical_cargo_runner.md` — HARD rule). Windows e2e bat-wrapper invocation + explicit-exit-marker reads (`feedback_windows_e2e_requires_bat_wrapper.md`, `cargo-output-capture.md` — trust the spill-log EXIT echo, not the bg-task completion notification). Cross-lane total cap = 2 running Junior tasks. Daemon trunk sync before every dispatch (`git fetch origin <base>:<base>`, never `git checkout` on the shared daemon checkout). Model tiering: Planning→Opus, Impl→Sonnet, BM/ci-watcher→Haiku (`feedback_brehon_subagent_model_effort_assignments.md`). Clarify gate before planning. The 6 user gates (plan-approval / ADR-DQ / CR-triage / **gate-4 e2e local-vs-dispatch NOW LIVE** / merge-confirm / retro-sign-off). DQ attribution `^(chore|docs)\((advisor|decision-queue)\)`. Memory headroom: no bulk e2e.rs reads. `--repo barrie-cork/lemmy` on all gh pr commands; PRs base `governance-v0` never `main`.

## 6. What changed from m3-core-recording's rule set

- **gate-4 (Phase-2 e2e local-vs-dispatch) goes from N/A to LIVE.** Phases 1–5 were binary-DTO / unit-level; Phase 6 is the e2e+pilot phase. The advisor surfaces the local-vs-dispatch choice once.
- **New runner kind in play:** `validate-pending-laptop-e2e` (testcontainers + Docker) becomes the dominant gate kind, not the Phase-5 `validate-pending-laptop-linux` compile proof. The `e2e_filter` field (`feedback_e2e_nextest_filter_groups.md`) is now load-bearing — scope e2e runs to the town-hall group, not the full ~12-min suite, where possible.
- **A non-Junior acceptance step exists:** the D2 pilot run. No prior M3-core phase had an operational human-run acceptance gate. The retro must record the pilot outcome (did a real town hall run? did the chair pass the mic? did recording land?).
- **MiniMax governance-ai-review is now live** (shipped 2026-06-20, this transition's session): `.github/workflows/governance-ai-review.yml` auto-reviews small governance-path PRs into `governance-v0` (ADR-001..015 rubric) + posts a comment. It's the post-Copilot second reviewer alongside CodeRabbit. Its first real exercise (the actual MiniMax API call) happens on the first small (<28KB) governance PR — likely a Phase-6 fix-in-PR. `bm-poll-cr` may need a `source: minimax` bucket if it posts findings (carry-forward from recording retro §8, now partially actioned).

## 7. Catch-fire procedures

Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.6: Junior subagent ignores hard refusals (writes `crates/**` or `services/bridge/**` from non-authorising brief); `answered_by: "advisor"` in a non-`chore|docs(advisor|decision-queue):` commit; subagent commits to `governance-v0`/`main` directly; BM opens PR into `main`; phase branch uncommitted when Junior reports complete; ci-watcher exit-code surprise; cohort uncommitted-code-before-cancel (SSH tar first). **Phase-specific additions:** (a) a §G4 validate-fail with the same `(error_class, file_basename)` ≥3 cycles → HARD REFUSAL catch-fire (cycle-count meta-rule); (b) an e2e test that measures emergency-mute latency server-side instead of at the publisher client → catch-fire (false-green risk, watchpoint #2); (c) a plan task that proposes the pilot as impl, or adds a new entry-kind const → catch-fire (scope, watchpoints #4/#6).

## 8. Archive after m3-core-e2e-pilot

m3-core-e2e-pilot is the LAST M3-core phase (Phase 6 of 6). At ship, run `/brehon-phase-transition m3-core-e2e-pilot <next>`. The `<next>` is NOT another M3-core sub-phase — M3-core is complete after this. The next thing is whatever the roadmap names after M3-core acceptance: either M3 full town-hall RTC follow-ons (anonymous town halls C3.8, Q&A sidebar polish if deferred) per the umbrella PRD, OR the pilot-internal / pilot-external track. **Confirm the next milestone with the user at that transition — do not assume.** This skill will close `workflow_state_m3_core_e2e_pilot.md`, delete the two-ago record (`workflow_state_m3_core_recording.md`), create the next skeleton, write the next bootstrap, update MEMORY.md, commit on governance-v0. This bootstrap file stays in `.claude/PRPs/handovers/` as its own archive (git history).

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `379b5446e` (captured 2026-06-20) — `Merge pull request #206 from barrie-cork/chore/ai-review-merge-base-fix`
- Phase branch HEAD: not yet created (branch `phase-m3-core-e2e-pilot` cut at bm-cut)
- Recent governance-v0 commits (`git -C C:/Users/barri/Developer/brehon-fork log --oneline -5 governance-v0`):

  ```
  379b5446e Merge pull request #206 from barrie-cork/chore/ai-review-merge-base-fix
  1a67d40f0 fix(ai-review): honour fallback BASE with plain diff range (CR finding)
  9f57f6f34 fix(ai-review): robust merge-base resolution for workflow_dispatch mode
  e4ebac81e docs(retro): m3-core-recording — clean ship (PR #205, 2a400a6ab); rust-s3 first-try; 3x fix-impl front-loaded (forward-declared, codified); 2 real CR bugs (cr-4 fail-open, cr-5 ADR-015/016) fixed; confidence 0.92
  2a400a6ab Merge pull request #205 from barrie-cork/phase-m3-core-recording
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff)
```

## Stop-and-ask tripwires

- Stop and ask if: the plan measures emergency-mute latency **server-side** rather than at the **publisher client across two federated bridge instances** — that is a false-green against the PRD line-141 <500ms criterion (watchpoint #2).
- Stop and ask if: any plan task proposes a new `ENTRY_KIND_ROOM_*` const or any migration under `crates/db_schema/migrations/**` — this phase is acceptance + pilot; the 3 chair/mute kinds + recording kind all shipped in Phases 2–5, registry count is frozen at 72 (watchpoint #4).
- Stop and ask if: a plan task tries to "implement the pilot town hall" as a Junior impl-task — the D2 pilot is a human-run operational acceptance step, not impl (watchpoint #6).
- Stop and ask if: a `services/bridge` cargo command in any §15 DoD is written in the Windows `cd services/bridge && cargo` form — bridge compiles Linux-only; it must use `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (watchpoint #3).
- Stop and ask if: the e2e suite cannot reach the LiveKit/MinIO testcontainers (sidecar bring-up fails) — prove the harness can reach the containers in an isolated task BEFORE queueing the acceptance tests, or the acceptance failures will be unattributable.
