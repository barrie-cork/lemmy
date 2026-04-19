# Phase 6 advisor runlog — state snapshot

Last updated: 2026-04-19 04:30Z — end of prep session, pre overnight run.

## Run model

Overnight autonomous advisor session executes all 7 Phase 6 agents
sequentially (A → G). User directive: avoid risky parallelisation,
sequential is fine, externalise context, take your time. No
double-agent layers; each agent gets its own worktree and the
advisor merges back into `phase-6` between spawns.

## Current git state

- `governance-v0` @ `3bbf419da` (PR #10 + PR #32 merged)
- `phase-6` @ `15f8cbbd0` (pushed to origin):
  - `3bbf419da` (governance-v0 tip)
  - `506563a92` chore(phase-6-prep): task-hopper infra + phase-6 plan + wrapper fixes
  - `15f8cbbd0` chore(phase-6): decision-queue pre-seeds DQ-6.1..6.5 + phase bump
- Primary worktree: `C:\Users\barri\Developer\brehon-fork` on
  `governance-v0`, clean working tree.
- Advisor worktree: `C:\Users\barri\Developer\brehon-fork-advisor-phase6`
  on `phase-6`, submodules initialised.
- No agent worktrees exist yet — overnight session creates them.

## Pre-flight status

| Check | Status | Evidence |
|---|---|---|
| PR #10 merged | OK | gh pr view 10 -> MERGED at 156db7cc8 |
| PR #32 merged | OK | governance-v0 tip at 3bbf419da |
| phase-6 branch exists | OK | git rev-parse phase-6 -> 15f8cbbd0 |
| phase-6 pushed to origin | OK | git push -u origin phase-6 at 04:00Z |
| Scaffolding on phase-6 | OK | 506563a92 cherry-picked |
| Submodules in advisor wt | OK | crates/email/translations/backend/*.json present |
| DQ pre-seeds on phase-6 | OK | 15f8cbbd0; 5 entries DQ-6.1..6.5 |
| Wrapper exit-code propagation | OK | probe 4 exit 101 on bogus feature |
| Workspace + features full | OK | probe 2 exit 0 in 7m 36s (post submodule init) |
| Advisor briefs authored | OK | 7 files in .claude/PRPs/phase-6-runlog/briefs/ |
| Handoff prompt authored | OK | HANDOFF-PROMPT.md in runlog dir |

## Agent sequence

| # | Agent | Tasks | Worktree | Branch |
|---|---|---|---|---|
| 1 | A | 70, 71 (migration + Diesel models) | `../brehon-fork-agent-a-phase6` | `agent-a-phase6` |
| 2 | B | 72 (AP objects) | `../brehon-fork-agent-b-phase6` | `agent-b-phase6` |
| 3 | C | 73 (AP activities) | `../brehon-fork-agent-c-phase6` | `agent-c-phase6` |
| 4 | D | 74 (outbound publisher) | `../brehon-fork-agent-d-phase6` | `agent-d-phase6` |
| 5 | E | 75, 78 (inbound + verify) | `../brehon-fork-agent-e-phase6` | `agent-e-phase6` |
| 6 | F | 76 (wire submit_jury_vote) | `../brehon-fork-agent-f-phase6` | `agent-f-phase6` |
| 7 | G | 77 (round-trip e2e test + SUBSCRIPTIONS.md) | `../brehon-fork-agent-g-phase6` | `agent-g-phase6` |

## Merge points

| # | After agent | Validation |
|---|---|---|
| 1 | A (tasks 70+71) | workspace check |
| 2 | B (task 72) | workspace check |
| 3 | C (task 73) | workspace check + clippy |
| 4 | D (task 74) | workspace check |
| 5 | E (tasks 75+78) | workspace check + clippy |
| 6 | F (task 76) | workspace check + clippy + e2e-compile |
| 7 | G (task 77) | full e2e suite |

## Decision queue (phase: 6)

All DQ entries for Phase 6 pre-seeded + advisor-answered:

| ID | Question | Answer (advisor) |
|---|---|---|
| 31 (DQ-6.1) | add received_at to remote_sanction_notice? | add-received-at |
| 32 (DQ-6.2) | where store HTTP signature? | store-activity-id |
| 33 (DQ-6.3) | idempotency safeguard? | rely-on-received-activity-dedup |
| 34 (DQ-6.4) | env-var cleanup? | leave-as-is-existing-pattern |
| 35 (DQ-6.5) | branch on scope or action? | branch-on-scope-confirmed |

## Externalised artefacts

- `00-advisor-state.md` — this file, state snapshot
- `01-phase-6-progress.md` — append-only log
- `briefs/agent-<A..G>.md` — per-agent self-contained briefs
- `HANDOFF-PROMPT.md` — fresh-session advisor prompt
- `.claude/task-hopper.json` — per-task execution ledger (starts empty)
- `.claude/decision-queue.json` — Phase 6 DQ answers

## What the overnight session does

Read `HANDOFF-PROMPT.md`. It provides the full playbook. The
summary: read each brief, spawn agent, wait, merge, validate,
push, repeat through G. After G, write completion report, open PR.

## What could go wrong

- Agent drift from brief (most likely source of churn). Mitigation:
  each brief is 300-500 lines with task-hopper envelope, pattern
  citations, gotchas, and catch-fire triggers.
- Plan text drift vs codebase (reported line numbers off by 10-30
  lines). Mitigation: briefs say "semantic anchoring, not
  line-number anchoring".
- activitypub_federation API drift vs docs.rs snapshot. Probe 2 at
  pre-flight compiled cleanly — evidence the crate's current API is
  compatible with Lemmy-beta.10 workspace pins. If an agent hits an
  API surprise, it writes a DQ entry rather than guessing.
- e2e test flake on task 77 (two-container boot timing). Retry cap
  on `cargo_test` kind is 2; hopper auto-escalates.
- Merge conflict on enum or route-list files. Unlikely (sequential
  agents) but possible if an agent rewrote instead of added. Advisor
  stops and resolves at merge point.
