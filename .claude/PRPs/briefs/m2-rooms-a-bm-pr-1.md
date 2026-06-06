---
phase: m2-rooms-a
role: bm-task
task: bm-pr
brief_n: 1
authored: 2026-06-06
plan: .claude/PRPs/plans/m2-rooms-a.plan.md
---

# [role:bm-task] m2-rooms-a bm-pr — open PR for phase-m2-rooms-a into governance-v0 — see .claude/PRPs/briefs/m2-rooms-a-bm-pr-1.md

## §1 Role + dispatch

`[role:bm-task] m2-rooms-a bm-pr — open PR for phase-m2-rooms-a into governance-v0`

Actual create-task description (single line, <100 chars):

```text
[role:bm-task] m2-rooms-a bm-pr — see .claude/PRPs/briefs/m2-rooms-a-bm-pr-1.md
```

## §2 Scope

Run `bm-pr` for `phase-m2-rooms-a`. m2-rooms-a is **M2 Matrix room provisioning + hash-chain emission** — extends the existing `services/bridge/` crate (workspace-EXCLUDED) and `crates/api/` workspace crates to provision Matrix rooms for governance events (jury selection, emergency removal, full case lifecycle), emit hash-chain entries via `append_room_event`, wire bearer-authenticated service endpoints for bridge↔binary communication, and add a bridge integration test suite.

All 8 plan §13 tasks (T1, T1w, T2, T3, T4a, T4b, T5, T6) delivered + validated. Phase includes both workspace (`crates/`) and bridge (`services/bridge/`) changes; the two toolchains are orthogonal (R8 boundary enforced throughout).

**Phase branch:** `phase-m2-rooms-a`
**Tip:** `76e2f2f2d` (`chore(decision-queue): advisor-laptop resolved d07588989c3c-001 result:pass — T6 test compile clean`)
**Base:** `governance-v0`
**Repo:** `barrie-cork/lemmy`

The PR body assembles from:
- the plan: `.claude/PRPs/plans/m2-rooms-a.plan.md` §13 Tasks 1–6 (T1/T1w grouped, T4a/T4b grouped)
- the commit log: `git log governance-v0..HEAD --oneline` (~60 commits)
- the §13 task table + DQ entries below

**Open the PR only. Do NOT merge** (merge is behind user gate 5).

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` — bm-pr operational script (follow phases 1→7 verbatim)
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/rules/phase-branch.md` — phase-branch + PR flow (base `governance-v0`, never `main`; not draft)
- `.claude/PRPs/plans/m2-rooms-a.plan.md` — plan → title + plan reference + §13 task table content
- `.claude/PRPs/briefs/m1-a-bm-pr-1.md` — **canonical-sibling brief** (per advisor-orchestrator.md §3.6); mirror its §1–§7 shape

## §4 Constraints

- **`--repo barrie-cork/lemmy`** on every `gh` command.
- Base: `governance-v0` (NOT `main`). Head: `phase-m2-rooms-a`.
- **Not draft** (CodeRabbit skips drafts).
- **PR title:** `Phase m2-rooms-a — M2 Matrix room provisioning + hash-chain emission (bridge + workspace, 8 tasks)`
- **PR body** assembles from (in this order):
  - `## Summary` — one paragraph: m2-rooms-a extends the Brehon Matrix AS bridge to provision governance rooms: jury selection rooms (always-pseudonymised Juror-<suffix> members, 5-panel), emergency removal rooms (reported party absent, <2s ADR-013), and full case lifecycle rooms (10 scenario transitions). Room events emit hash-chain entries via `append_room_event` into `governance_log`. Two bearer-authenticated binary endpoints added to the workspace: `POST /governance/room-event` (bridge callback; T4a) and `GET /governance/bridge/messaging-status` (soft-pause poller; T4b). Bridge internal wiring: `soft_pause.rs` bearer header + `Arc<AtomicI64>` for `oq009_reveal_threshold`, idempotency watermark via `bridge_room` SQLite store, `run_poller` signature updated, room_provisioner reads threshold from live AppState. Bridge test suite: 5 `#[ignore]` docker-compose integration tests (`room_provisioning.rs`) + `soft_pause_enable_disable_cycle` un-stubbed in `dm_round_trip.rs`.
  - `## Plan reference` — `.claude/PRPs/plans/m2-rooms-a.plan.md` §13 Tasks T1–T6
  - `## Task table` — Tasks with commits + DQ entries:
    - **Task 1 + T1w** (bridge scaffold + juror-pseudonyms producer): `services/bridge/src/bridge_room.rs` (rusqlite store), `services/bridge/src/config.rs` (BRIDGE_CALLBACK_SECRET, brehon_room_event_url, legal_contact_mxid), `crates/api/api_utils/src/bridge_notify.rs` (juror_pseudonyms field in CaseTransitionEvent), `crates/api/api_utils/Cargo.toml` (diesel + diesel_async deps). Phase: `86573fa1f` (T0+T1 merge), `b7cd5c86a` (T1w merge), `ad537051b` (T1w-fix-2 PASS). DQ: `7d593d3bc09f-001` (rusqlite conflict → pass), `6ad0b18d9dac-001` (T1w diesel missing → fail), `f99a71bc9ed4-001` (JoinOnDsl missing → fail), `ee6325ec1c71-001` (T1w-fix-2 → PASS resolved).
    - **Task 2** (room_provisioner.rs core jury path): `services/bridge/src/room_provisioner.rs` C2.1 — idempotency check, always_pseudonym, Juror-<suffix> construction, 5-panel. Merge `eccae5eaf`. DQ `f0d76f99a08f-001` PASS.
    - **Task 3** (scenarios C2.2–C2.6 + OQ-009 + append_room_event): `services/bridge/src/room_provisioner.rs` extended for all room scenarios; hash-chain emission via `append_room_event`. Merge `a612ccc5f`. DQ `797f00a77338-001` PASS.
    - **Task 4a** (workspace route + bridge_auth): `crates/api/api/src/governance/bridge_auth.rs` (verify_bridge_secret), `crates/api/api/src/governance/room_event_handler.rs` (handle_room_event), `crates/api/api/src/governance/governance_log.rs` (+Deserialize on RoomEventPayload via T4a-fix), route registered in `crates/api/routes/src/lib.rs`. Merges `1ca0cd627` (T4a), `b6e7be76f` (T4a-fix). DQ `0566bc2d61f0-001` (E0277 fail), `025c517a5e70-001` (T4a-fix PASS resolved).
    - **Task 4b** (bridge-read messaging-status): `crates/api/api/src/governance/bridge_read.rs` (GET /governance/bridge/messaging-status, BridgeStatus{messaging_enabled, oq009_reveal_threshold}), route registered. Merge `a5758d883`. DQ `9d80b96935d9-001` PASS.
    - **Task 5** (bearer auth wiring + AtomicI64): `services/bridge/src/soft_pause.rs` (Bearer header, parse oq009_reveal_threshold), `services/bridge/src/appservice.rs` (oq009_reveal_threshold field), `services/bridge/src/main.rs` (Arc<AtomicI64> init + wiring), `services/bridge/src/room_provisioner.rs` (read from state). Commit `19201e4b2`. DQ `ecc4ade814f2-001` PASS.
    - **Task 6** (integration test suite): `services/bridge/tests/room_provisioning.rs` (5 #[ignore] tests), `services/bridge/tests/dm_round_trip.rs` (soft_pause_enable_disable_cycle un-stubbed). Merge `e2ae5716f`. DQ `d07588989c3c-001` PASS.
  - `## Validation` — Workspace: `./scripts/brehon/cargo-check.sh --workspace --features full` EXIT 0 (T4a-fix, T4b validated). Bridge: `cd services/bridge && cargo check` EXIT 0 (T2, T3, T5 validated); `cargo test --no-run` EXIT 0 (T6 validated). Linux: `validate-pending-laptop-linux` DQ `f06ed49af84a-001` result:pass. Pre-Shape-G — cargo on laptop, no billed GH Actions.
  - `## Commits` — `git log governance-v0..HEAD --oneline` output (~60 commits), one bullet per line.
  - `## Closes` — `(none — plan-driven sub-phase; no GH issues referenced)`

- **Phase 1b (DQ historical-fail sweep):** run it. DQ pending=0 on tip `76e2f2f2d` (all DQ resolved). Expected: no-op. If sweep finds anything → STOP + `kind:"blocker"`.
- **Phase 1c (Phase 2 e2e gate):** NOT applicable — zero `crates/server/tests/e2e.rs` edits. Skip phase 1c silently.
- Write `.claude/PRPs/reviews/pr-<N>-findings.yaml` shell per `.claude/PRPs/reviews/SCHEMA.md`.
- Append bm-pr "PR opened" runlog entry to `.claude/runlog/bm-runlog.md` (create if absent).
- Do **NOT** post a Telegram ping (advisor handles outbound).
- Do **NOT** touch `crates/**`, `migrations/**`, `tests/**` (BM file-ownership HARD boundary).
- Do **NOT** mutate any DQ entry beyond Phase 1b sweep.

## §5 Validation gates

- Phase 1 pre-conditions: branch is `phase-m2-rooms-a`, pushed, clean tree, ~60 commits ahead of `governance-v0`, no existing PR.
- Phase 1b: no-op sweep (DQ pending=0 expected on `76e2f2f2d`).
- Phase 1c: skip (no e2e edits).
- Phase 2: PR body assembles cleanly from §4 ingredients.
- Phase 3: `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-m2-rooms-a --title "<§4 title>" --body-file <tempfile>` returns PR URL.
- Phase 4: PR body matches assembled content.
- Phase 5: findings.yaml shell written at `.claude/PRPs/reviews/pr-<N>-findings.yaml`.
- Phase 6: runlog appended to `bm-runlog.md`.
- Phase 7: return PR URL + "Next suggested: bm-poll-cr after ~5-15min for CodeRabbit findings".

## §6 Expected output (return to advisor)

```text
## bm-pr complete — PR #<N> opened on phase-m2-rooms-a

**PR URL:** https://github.com/barrie-cork/lemmy/pull/<N>
**Title:** Phase m2-rooms-a — M2 Matrix room provisioning + hash-chain emission (bridge + workspace, 8 tasks)
**Base:** governance-v0
**Head:** phase-m2-rooms-a @ 76e2f2f2d
**Body length:** ~<N> lines (~60 commits in §Commits section)
**findings.yaml:** .claude/PRPs/reviews/pr-<N>-findings.yaml (shell only — counters all 0)
**Runlog entries:** bm-runlog.md appended
**Phase 1b sweep:** no-op (DQ pending=0, expected)
**Phase 1c e2e gate:** skipped (no workspace e2e edits)
**Next suggested:** bm-poll-cr after ~5-15min for CodeRabbit findings
```

## §7 Why this brief differs from the canonical sibling

Mirrors `.claude/PRPs/briefs/m1-a-bm-pr-1.md` schema verbatim. Differences:
- **Scope:** m2-rooms-a spans BOTH workspace (`crates/`) AND bridge (`services/bridge/`) — two toolchains, R8 boundary enforced.
- **Phase-2 e2e:** OMITTED (zero `crates/server/tests/e2e.rs` edits — all bridge changes + new governance API handlers with no e2e test coverage in this phase).
- **Task count:** 8 tasks (T1, T1w, T2, T3, T4a, T4b, T5, T6) — T1/T1w grouped in task table, T4a/T4b grouped.
- **§G4 fix cycles:** T1w had 3 fix cycles (diesel dep, diesel_async dep, JoinOnDsl); T4a had 1 fix cycle (Deserialize on RoomEventPayload) — both documented in task table DQ entries.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; m2rooms-a worktree at `brehon-fork-m2rooms-a`). Brief committed on `governance-v0`. bm-task worker uses `base_branch=governance-v0`._
