---
phase: m2-late-2
role: bm-task
task: bm-pr
brief_n: 1
authored: 2026-06-12
plan: .claude/PRPs/plans/m2-late.plan.md
---

# [role:bm-task] m2-late-2 bm-pr — open PR for phase-m2-late-2 into governance-v0 — see .claude/PRPs/briefs/m2-late-2-bm-pr-1.md

## §1 Role + dispatch

`[role:bm-task] m2-late-2 bm-pr — open PR for phase-m2-late-2 into governance-v0`

Actual create-task description (single line, <100 chars):

```text
[role:bm-task] m2-late-2 bm-pr — see .claude/PRPs/briefs/m2-late-2-bm-pr-1.md
```

## §2 Scope

Run `bm-pr` for `phase-m2-late-2`. m2-late-2 is **M2 B-publish sanction propagation (live wiring + CR fixes)** — the second sub-phase of m2-late. It wires the sanction event machinery live: T1 adds `case_id` to `SanctionEventPayload` + e2e assertion; T2 (CR-A fix) wraps the `sanction_event` INSERT and `governance_log::append` in `run_transaction` (ADR-008 atomicity); T3 mirrors `case_id` in the bridge `SanctionEventPayload`; T4 rewrites `handle_sanction_event` with power-level enforcement logic; T5 adds dep-free tests + one `#[ignore]` live test to `sanction_handler.rs`.

All 5 plan §13 tasks delivered + validated. Phase spans workspace (`crates/`) and bridge (`services/bridge/`) changes under the R8 boundary.

**Phase branch:** `phase-m2-late-2`
**Tip:** `b595bb4b4` (`chore(decision-queue): advisor-laptop resolved DQ 0365093316ed-001 — T5 bridge tests validate pass`)
**Base:** `governance-v0`
**Repo:** `barrie-cork/lemmy`

**Merge-forward required:** `governance-v0` has 18 commits not in the phase branch. Run `git fetch origin && git merge origin/governance-v0 --no-edit` on the phase branch before opening the PR, then push.

The PR body assembles from:
- the plan: `.claude/PRPs/plans/m2-late.plan.md` §13 Tasks T1–T5
- the commit log: `git log governance-v0..HEAD --oneline` (~32 commits)
- the §13 task table + DQ entries below

**Open the PR only. Do NOT merge** (merge is behind user gate 5).

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` — bm-pr operational script (follow phases 1→7 verbatim)
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/rules/phase-branch.md` — phase-branch + PR flow (base `governance-v0`, never `main`; not draft)
- `.claude/PRPs/plans/m2-late.plan.md` — plan → title + plan reference + §13 task table content
- `.claude/PRPs/briefs/m2-rooms-a-bm-pr-1.md` — **canonical-sibling brief** (per advisor-orchestrator.md §3.6); mirror its §1–§7 shape

## §4 Constraints

- **`--repo barrie-cork/lemmy`** on every `gh` command.
- Base: `governance-v0` (NOT `main`). Head: `phase-m2-late-2`.
- **Not draft** (CodeRabbit skips drafts).
- **PR title:** `Phase m2-late-2 — B-publish sanction propagation: live wiring + CR atomicity fix (workspace + bridge, 5 tasks)`
- **Merge-forward first:** before `gh pr create`, run on the phase branch:
  ```bash
  git fetch origin
  git merge origin/governance-v0 --no-edit
  git push origin phase-m2-late-2
  ```
- **PR body** assembles from (in this order):
  - `## Summary` — one paragraph: m2-late-2 wires the B-publish sanction propagation machinery live. T1 adds `case_id` to `SanctionEventPayload` and the e2e assertion so the event payload is traceable to the originating governance case. T2 (CR-A fix from the m2-late-1 review) wraps the `sanction_event` INSERT and `governance_log::append` call in a single `run_transaction`, satisfying ADR-008 atomicity for dual-write paths. T3 mirrors `case_id` in the bridge `SanctionEventPayload` so the Matrix bridge can look up the associated room. T4 rewrites `handle_sanction_event` with full power-level enforcement logic (ban/mute/prevent_post → power level −1, etc.) replacing the earlier stub. T5 adds a dep-free test suite to `sanction_handler.rs` (auth-fail path, no-rooms early-return, `compute_power_override` unit cases, one `#[ignore]` live integration test). Workspace validated via `cargo-check.sh --workspace --features full`; bridge validated via `cargo test` + `cargo clippy --no-deps` under Linux Docker.
  - `## Plan reference` — `.claude/PRPs/plans/m2-late.plan.md` §13 Tasks T1–T5
  - `## Task table` — Tasks with commits + DQ entries:
    - **Task 1** (`case_id` in payload + e2e): `crates/api/api/src/governance/sanction_publisher.rs` (+`case_id` field), `crates/api/api/src/governance/submit_jury_vote.rs` (populate `case_id`), `crates/server/tests/e2e/m2_late.rs` (assert `case_id` in event payload). Merge `523bd299f`. DQ `7d3a1a7f8581-001` result:pass.
    - **Task 2** (CR-A atomicity fix — `run_transaction`): `crates/api/api/src/governance/sanction_publisher.rs` (wrap dual-write in `conn.run_transaction()`). Merge `dad6cfe82`. DQ `8d79e47cceeb-001` result:pass.
    - **Task 3** (bridge `case_id` mirror): `services/bridge/src/sanction_handler.rs` (+`case_id` field), bridge `SanctionEventPayload` struct updated. Merge `dc2a864e8`. DQ `8f91e3725dc5-001` result:pass.
    - **Task 4** (power-level enforcement): `services/bridge/src/sanction_handler.rs` (rewrite `handle_sanction_event` with `compute_power_override`, power-level PUT Matrix call). Merge `f88d867b1`. DQ `c8e294aa18d8-001` result:pass.
    - **Task 5** (dep-free tests): `services/bridge/src/sanction_handler.rs` (`#[cfg(test)] mod tests` — 3 sync + 1 `#[ignore]` async). Merge `a23923b13`. DQ `0365093316ed-001` result:pass.
  - `## Validation` — Workspace: `./scripts/brehon/cargo-check.sh --workspace --features full` EXIT 0 + e2e 135/135 PASS (Phase 2 e2e-2 retry). Bridge: `cd services/bridge && cargo test` EXIT 0 (3 pass, 1 skipped) + `cargo clippy --no-deps -- -D warnings` 18 pre-existing baseline only. Pre-Shape-G — cargo on laptop, no billed GH Actions.
  - `## Commits` — `git log governance-v0..HEAD --oneline` output (~32 commits), one bullet per line.
  - `## Closes` — `(none — plan-driven sub-phase; no GH issues referenced)`

- **Phase 1b (DQ historical-fail sweep):** run it. DQ pending=0 on tip `b595bb4b4` (all DQ resolved). Expected: no-op. If sweep finds anything → STOP + `kind:"blocker"`.
- **Phase 1c (Phase 2 e2e gate):** applicable — `crates/server/tests/e2e/m2_late.rs` was edited. Confirm Phase 2 e2e result is available: e2e 135/135 PASS recorded at `.claude/auto-state/m2-late-2.json` stage_digest `phase-2-e2e-2-retry`. No re-run needed.
- Write `.claude/PRPs/reviews/pr-<N>-findings.yaml` shell per `.claude/PRPs/reviews/SCHEMA.md`.
- Append bm-pr "PR opened" runlog entry to `.claude/runlog/bm-runlog.md` (create if absent).
- Do **NOT** post a Telegram ping (advisor handles outbound).
- Do **NOT** touch `crates/**`, `migrations/**`, `tests/**` (BM file-ownership HARD boundary).
- Do **NOT** mutate any DQ entry beyond Phase 1b sweep.

## §5 Validation gates

- Phase 1 pre-conditions: branch is `phase-m2-late-2`, pushed, clean tree after merge-forward, ~32 commits ahead of `governance-v0`, no existing PR.
- Phase 1b: no-op sweep (DQ pending=0 expected on `b595bb4b4`).
- Phase 1c: e2e confirmed 135/135 PASS (already recorded — no re-run).
- Phase 2: PR body assembles cleanly from §4 ingredients.
- Phase 3: `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-m2-late-2 --title "<§4 title>" --body-file <tempfile>` returns PR URL.
- Phase 4: PR body matches assembled content.
- Phase 5: findings.yaml shell written at `.claude/PRPs/reviews/pr-<N>-findings.yaml`.
- Phase 6: runlog appended to `bm-runlog.md`.
- Phase 7: return PR URL + "Next suggested: bm-poll-cr after ~5-15min for CodeRabbit findings".

## §6 Expected output (return to advisor)

```text
## bm-pr complete — PR #<N> opened on phase-m2-late-2

**PR URL:** https://github.com/barrie-cork/lemmy/pull/<N>
**Title:** Phase m2-late-2 — B-publish sanction propagation: live wiring + CR atomicity fix (workspace + bridge, 5 tasks)
**Base:** governance-v0
**Head:** phase-m2-late-2 @ b595bb4b4 (post merge-forward)
**Body length:** ~<N> lines (~32 commits in §Commits section)
**findings.yaml:** .claude/PRPs/reviews/pr-<N>-findings.yaml (shell only — counters all 0)
**Runlog entries:** bm-runlog.md appended
**Phase 1b sweep:** no-op (DQ pending=0, expected)
**Phase 1c e2e gate:** confirmed PASS (135/135 from phase-2-e2e-2-retry digest)
**Next suggested:** bm-poll-cr after ~5-15min for CodeRabbit findings
```

## §7 Why this brief differs from the canonical sibling

Mirrors `.claude/PRPs/briefs/m2-rooms-a-bm-pr-1.md` schema verbatim. Differences:
- **Scope:** m2-late-2 spans BOTH workspace (`crates/`) AND bridge (`services/bridge/`) — two toolchains, R8 boundary enforced (same as m2-rooms-a).
- **Task count:** 5 tasks (T1–T5); T2 is a CR-A fix from the m2-late-1 review cycle.
- **Merge-forward:** explicit instruction in §4 (18 trunk commits not in phase branch).
- **Phase 1c:** applicable (m2_late.rs edited) but confirmed via existing Phase 2 e2e-2 retry result rather than a new run.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`). Brief committed on `governance-v0`. bm-task worker uses `base_branch=governance-v0`._
