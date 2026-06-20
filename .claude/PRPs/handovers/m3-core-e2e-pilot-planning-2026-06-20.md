---
phase: m3-core-e2e-pilot
scope: planning-queued (pre-compact handover)
lane_mode: B
authored: 2026-06-20
authored_by: advisor (canonical brehon-fork / governance-v0 session)
last_handover_path: .claude/PRPs/handovers/m3-core-e2e-pilot-planning-2026-06-20.md
purpose: Resume the m3-core-e2e-pilot advisor session after a /compact boundary. Planning Junior #746 is in flight. Read this block first.
---

# ⏩ RESUME — read first (post-/compact)

**You are the advisor for Brehon m3-core-e2e-pilot** (M3 town halls, **Phase 6 of 6 — FINAL**). CWD `C:/Users/barri/Developer/brehon-fork`, branch `governance-v0` (Mode B — drive from canonical via Junior dispatch). There is no homeserver advisor session.

## State at handoff (literal — do not paraphrase)

- **governance-v0 HEAD: `88950b89e`** (origin == local == daemon-local, all SYNC at handoff).
- **Planning Junior #746 RUNNING** (Opus, base `governance-v0`): `[role:planning] m3-core-e2e-pilot — see .claude/PRPs/briefs/m3-core-e2e-pilot-planning-1.md`. Produces `.claude/PRPs/plans/m3-core-e2e-pilot.plan.md`.
- **DQ pending: 0.** Two clarify entries resolved by advisor: `a3d0e9941441-074` (Task-1 topology → override compose) + `a3d0e9941441-075` (Task-4 D2 pilot → runbook supports either scenario). Planning gate was CLEAR.
- Commits this session: `3fb647a42` (planning brief), `1a65753bc` (clarify DQ), `88950b89e` (DQ archive — 107 legacy int-id entries → `decision-queue-archive-pre-m3-core-legacy-int-ids.json`; live file 733KB→507KB).

## NEXT CONCRETE ACTION

**Poll #746** (`mcp__junior-brehon__list_tasks`, ~10-min cadence). On status transition:

1. **#746 `done`** → `git fetch origin && show_task 746` → read `.claude/PRPs/plans/m3-core-e2e-pilot.plan.md`. Then the plan-approval gate sequence (advisor-orchestrator §3.1 "Planning complete"):
   - **§3.4 DoD smoke test** — run EVERY §15 validation command literally against current HEAD; capture exit codes. **Bridge §15 commands run LINUX-ONLY** via `scripts/brehon/cargo-linux.sh ... --manifest-path services/bridge/Cargo.toml` — a §15 bridge command in the Windows `cd services/bridge && cargo` form is itself a DoD-issue (file advisor DQ + ask planner to re-point). First Docker bridge run is cold (~10–20 min).
   - **§3.5 watchpoint-specificity gate** — every §4 watchpoint cites a file:line/test/fn; concept-only → file DQ requesting revision.
   - **§3.5a MiniMax-trial designation** — walk every `[role:impl-task]` task vs the 5 qualifying criteria; record rows + cumulative count; report `MiniMax trial: N/5` in the approval surface. (Trial RE-ARMED 2026-06-12; fires at cumulative ≥5.)
   - **gate 1 (plan approval)** → surface to USER → WAIT. Then `bm-cut`.
2. **#746 `failed`/`cancelled`** → `show_task 746` for the escalation; check `.claude/governance-log/retro-bypass.jsonl` FIRST (orchestration-failure first-check); re-dispatch or surface per the failure.
3. **DQ pending appears mid-run** → triage per advisor-orchestrator §5.4 (falsifiable-hypothesis gate for structural-fix DQs; advisor-answer / catch-fire / user-relay).

## Phase-specific tripwires (catch-fire — from bootstrap §7 + brief §4)

- **Emergency-mute <500ms measured SERVER-SIDE** instead of at the publisher client across two federated instances → catch-fire (false-green, PRD line 141). The plan's marquee §16a story MUST measure at the publisher client.
- **New `ENTRY_KIND_ROOM_*` const or any `crates/db_schema/migrations/**`** → catch-fire (registry FROZEN at 72; this is acceptance+pilot, adds none).
- **A §13 task that "implements the pilot town hall" as a cargo impl-task** → catch-fire (D2 is Half-B operational, NON-impl; runbook only, DoD = retro-recorded outcome).
- **Bridge §15 command in Windows form** → DoD-issue (Linux-only; `ruma-common` E0119).

## Key recon facts the plan should reflect (verified 2026-06-20 @ `2fae33bc1`)

- **Acceptance harness is NET-NEW infra:** `services/bridge/docker-compose.yml` has the single-instance RTC stack (tuwunel + livekit + lk-jwt + element-call) but **NO MinIO + NO 2nd federated instance**. Task 1 builds them via an OVERRIDE compose (clarify `-074`) and proves reach-the-containers BEFORE acceptance tests (hard `requires:` of every acceptance test). 2nd instance needs a DISTINCT domain (BUG-15, `feedback_lemmy_federation_domain_collision_one_host.md`).
- **Town-hall e2e is BRIDGE-side `anyhow::Result`** in `services/bridge/tests/{stage_mode,emergency_mute,recording,room_provisioning}.rs` — NOT `crates/server/tests/e2e.rs` (0 town-hall matches there; corrects bootstrap §4 #1). Scaffolds are `#[tokio::test] #[ignore = "requires docker-compose stack"]` with step-by-step comment bodies + a single `todo!("...Phase-6 pilot grade")` per fn (the natural unique Edit anchor). Live-test target fns: `four_mic_pass_then_grace_boundary_emits_chair_entries`, `mute_all_drops_all_publishers_cross_instance_under_500ms` (MARQUEE), `recording_lands_with_hash_on_chain`/`clean_posture_no_recording_when_disabled`/`participant_floor_fetch`, + the 4 `room_provisioning.rs` scenarios.
- **cr-2/cr-3 recording carry-forwards = Task 2** (live session-auth requester-pseudonym + real participant-set so the floor ADMITS; recording floor returns 403 for ALL callers today). Recording live trigger = real Egress POST + S3 PUT (the `LiveSink` `bail!` stubs go live). `content_sha256` rides `append_room_event`, NEVER a bypass digest.
- **gate-4 (Phase-2 e2e local-vs-dispatch) is LIVE this phase** — surface the local-vs-dispatch choice ONCE per phase to the user (never auto-pick after PR #105). Local = bat-wrapper ~26 min zero-billed; dispatch = billed + public log. The acceptance `--ignored` tests need the full RTC stack UP (Task 1), so they are `validate-pending-laptop-e2e`, not pure `-linux` compile-proofs.
- **MiniMax governance-ai-review is LIVE** (shipped 2026-06-20) — post-Copilot 2nd reviewer on small (<28KB) governance PRs into `governance-v0`. First real API exercise likely a Phase-6 fix-in-PR; `bm-poll-cr` may need a `source: minimax` bucket.

## Operational reminders (Mode B)

- Cross-lane cap = 2 running Junior tasks; daemon trunk sync (`git fetch origin gov:gov`, never `git checkout` on the shared daemon checkout) before every dispatch.
- NO CARGO ON ELITEDESK (`project_laptop_canonical_cargo_runner.md`) — workers write `validate-pending-laptop[-e2e][-linux]` DQ and STOP; the LAPTOP runs ALL cargo/e2e.
- DQ attribution: advisor commits `^(chore|docs)\((advisor|decision-queue)\)`. `--repo barrie-cork/lemmy` on all `gh pr`; PRs base `governance-v0` never `main`.
- Model tiering: Planning→Opus/xhigh, Impl→Sonnet/medium, BM/ci-watcher→Haiku/low.
- The 6 user gates: plan-approval / ADR-DQ / CR-triage / **gate-4 e2e local-vs-dispatch (LIVE)** / merge-confirm / retro-sign-off.

## Source-of-truth files (read on resume if needed)

- `.claude/PRPs/briefs/m3-core-e2e-pilot-planning-1.md` — the full planning brief (two-halves scope, boundaries, constraints, recon anchors).
- `.claude/PRPs/handovers/m3-core-e2e-pilot-bootstrap.md` — the stable phase advice (the original handoff letter).
- `workflow_state_m3_core_e2e_pilot.md` (PMD System-1) — running scratchpad (plan-shaping decisions + clarify resolutions recorded).
- `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` — Phase 6 (250–253) + §Success Criteria (134–149).
