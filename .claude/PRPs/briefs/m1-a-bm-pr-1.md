---
phase: m1-a
role: bm-task
task: bm-pr
brief_n: 1
authored: 2026-06-04
plan: .claude/PRPs/plans/m1.plan.md
---

# [role:bm-task] m1-a bm-pr — open PR for phase-m1-a into governance-v0 — see .claude/PRPs/briefs/m1-a-bm-pr-1.md

## §1 Role + dispatch

`[role:bm-task] m1-a bm-pr — open PR for phase-m1-a into governance-v0`

Actual create-task description (single line, <100 chars):

```text
[role:bm-task] m1-a bm-pr — see .claude/PRPs/briefs/m1-a-bm-pr-1.md
```

## §2 Scope

Run `bm-pr` for `phase-m1-a`. m1-a is **M1 Tree A — Brehon Matrix AS bridge** — a greenfield `services/bridge/` Rust crate (workspace-EXCLUDED) implementing the Matrix Application Service bridge: axum HTTP server, AS transaction receiver, puppet-on-first-contact, 1:1 DM relay (text/image/voice), room provisioning, soft-pause drain, and docker-compose integration test scaffolding. Plus Tree C design docs (chat-plane boundary, threat rows, soft-pause posture, AGPL notice extension).

All 7 plan §13 tasks delivered + validated (Tasks 8–14). **Zero Lemmy workspace changes** — phase ships only `services/bridge/` (excluded from workspace) and `docs/**` + `AGPL-NOTICE.md`.

**Phase branch:** `phase-m1-a`
**Tip:** `94aca699c` (`docs(bridge): Task 14 — chat-plane boundary + threat rows + soft-pause posture + AGPL notice (task 14)`)
**Base:** `governance-v0`
**Repo:** `barrie-cork/lemmy`

The PR body assembles from:
- the plan: `.claude/PRPs/plans/m1.plan.md` §13 Tasks 8–14 (Tree A + Tree C)
- the commit log: `git log governance-v0..HEAD --oneline` (~30 commits)
- the §13 task table + DQ entries below

**Open the PR only. Do NOT merge** (merge is behind user gate 5).

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` — bm-pr operational script (follow phases 1→7 verbatim)
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/rules/phase-branch.md` — phase-branch + PR flow (base `governance-v0`, never `main`; not draft)
- `.claude/PRPs/plans/m1.plan.md` — plan → title + plan reference + §13 task table content (read §13 Tasks 8–14 + Tree C section)
- `.claude/PRPs/briefs/v1-rls-r1-bm-pr-1.md` — **canonical-sibling brief** (per advisor-orchestrator.md §3.6); mirror its §1–§7 shape

## §4 Constraints

- **`--repo barrie-cork/lemmy`** on every `gh` command.
- Base: `governance-v0` (NOT `main`). Head: `phase-m1-a`.
- **Not draft** (CodeRabbit skips drafts).
- **PR title:** `Phase m1-a — M1 Tree A: Matrix AS bridge (services/bridge/) + Tree C docs (7 tasks)`
- **PR body** assembles from (in this order):
  - `## Summary` — one paragraph: m1-a delivers M1 Tree A — a greenfield Matrix Application Service bridge at `services/bridge/` (workspace-excluded from Lemmy's Cargo workspace). The bridge implements an axum HTTP server for AS transaction handling, puppet-on-first-contact using matrix-sdk, 1:1 DM relay supporting text/image/voice events in both directions, admin room provisioning via `POST /admin/provision-room`, soft-pause drain reading `messaging_enabled` from the Brehon governance config API, and a docker-compose integration test scaffold with Tuwunel (federation-disabled). Tree C (Task 14) extends the security design doc (§2.2.2 chat-plane boundary + 4 threat rows), the operations doc (§5.6 soft-pause posture table), and AGPL-NOTICE.md to cover the bridge daemon and Tuwunel. Zero Lemmy workspace changes — `crates/`, `migrations/`, and `tests/` untouched.
  - `## Plan reference` — `.claude/PRPs/plans/m1.plan.md` §13 Tasks 8–14 (Tree A + Tree C)
  - `## Task table` — Tasks 8–14 with commits + DQ entries:
    - Task 8 — `services/bridge/` scaffold: `Cargo.toml` (workspace-excluded), `src/main.rs`, `src/config.rs`, `src/lib.rs` — `830391936` (merge `e32c1c831`) — DQ `9d8da421eca0-001` pass
    - Task 9 — AS transaction server: axum routes, `hs_token` auth middleware, `src/appservice.rs` skeleton — `e720a3a01` (merge `1a244decd`) — DQ `216da0ce90a6-001` + `216da0ce90a6-002` pass
    - Task 10 — Puppet-on-first-contact: matrix-sdk AS client, bridge-local puppet map, `src/puppet.rs` — `700628d19` (merge `cd9ace533`) — DQ `a9a0325ee9f3-001` pass
    - Task 11 — 1:1 DM relay: `handle_inbound` + `send_as_puppet`, AppState HTTP client, `src/relay.rs` — `b86b0e2e0` (merge `9b67b1dd6`) — DQ `9882adecbcbc-001` pass
    - Task 12 — Room provisioning + soft-pause: `src/provision.rs`, `src/soft_pause.rs`, relay_enabled flag — `ed229bbce` (merge `357663320`) — DQ `c5d7b8ae42ab-001` pass
    - Task 13 — Integration test scaffold + docker-compose: `registration.yaml`, `docker-compose.yml`, `tests/dm_round_trip.rs` (5 `#[ignore]` stubs), `POST /admin/provision-room` route — `c529500a6` (merge `a835ba4a9`) — DQ `fedec7727980-001` pass (SQLITE3_LIB_DIR=conda; corrected test command omits `--ignored` flag)
    - Task 14 — Tree C docs: `06-security` §2.2.2 + 4 bridge threat rows; `07-operations` §5.6 soft-pause table; `AGPL-NOTICE.md` bridge + Tuwunel disclosure — `94aca699c` (advisor-direct) — no cargo DQ (doc-only)
  - `## Validation` — Zero Lemmy workspace changes; all `cd services/bridge && cargo check` runs EXIT 0 (Tasks 8–13 each had a `validate-pending-laptop` DQ resolved by advisor-laptop). Docker-compose validate: `docker compose up -d` + `cargo test --test dm_round_trip` (5 tests ignored) + `docker compose down` all EXIT 0. Tree C: `git diff --stat` confirmed only `docs/` + `AGPL-NOTICE.md`. No Phase 2 e2e (zero `crates/server/tests/e2e.rs` edits). Pre-Shape-G — cargo on laptop, no billed GH Actions.
  - `## Commits` — `git log governance-v0..HEAD --oneline` output (~30 commits), one bullet per line.
  - `## Closes` — `(none — plan-driven sub-phase; no GH issues referenced)`

- **Phase 1b (DQ historical-fail sweep):** run it. DQ pending=0 on tip `94aca699c` (all DQ resolved). Expected: no-op. If sweep finds anything → STOP + `kind:"blocker"`.
- **Phase 1c (Phase 2 e2e gate):** NOT applicable — zero `crates/server/tests/e2e.rs` edits (Tree A is workspace-EXCLUDED). Skip phase 1c silently.
- Write `.claude/PRPs/reviews/pr-<N>-findings.yaml` shell per `.claude/PRPs/reviews/SCHEMA.md`.
- Append bm-pr "PR opened" runlog entry to `.claude/runlog/bm-runlog.md` (create if absent).
- Do **NOT** post a Telegram ping (advisor handles outbound).
- Do **NOT** touch `crates/**`, `migrations/**`, `tests/**` (BM file-ownership HARD boundary).
- Do **NOT** mutate any DQ entry beyond Phase 1b sweep.

## §5 Validation gates

- Phase 1 pre-conditions: branch is `phase-m1-a`, pushed, clean tree, ~30 commits ahead of `governance-v0`, no existing PR.
- Phase 1b: no-op sweep (DQ pending=0 expected).
- Phase 1c: skip (no workspace Rust edits).
- Phase 2: PR body assembles cleanly from §4 ingredients.
- Phase 3: `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-m1-a --title "<§4 title>" --body-file <tempfile>` returns PR URL.
- Phase 4: PR body matches assembled content.
- Phase 5: findings.yaml shell written at `.claude/PRPs/reviews/pr-<N>-findings.yaml`.
- Phase 6: runlog appended to `bm-runlog.md`.
- Phase 7: return PR URL + "Next suggested: bm-poll-cr after ~5-15min for CodeRabbit findings".

## §6 Expected output (return to advisor)

```text
## bm-pr complete — PR #<N> opened on phase-m1-a

**PR URL:** https://github.com/barrie-cork/lemmy/pull/<N>
**Title:** Phase m1-a — M1 Tree A: Matrix AS bridge (services/bridge/) + Tree C docs (7 tasks)
**Base:** governance-v0
**Head:** phase-m1-a @ 94aca699c
**Body length:** ~<N> lines (~30 commits in §Commits section)
**findings.yaml:** .claude/PRPs/reviews/pr-<N>-findings.yaml (shell only — counters all 0)
**Runlog entries:** bm-runlog.md trunk per L14 ordering
**Phase 1b sweep:** no-op (DQ pending=0, expected)
**Phase 1c e2e gate:** skipped (no workspace Rust edits)
**Next suggested:** bm-poll-cr after ~5-15min for CodeRabbit findings
```

## §7 Why this brief differs from the canonical sibling

Mirrors `.claude/PRPs/briefs/v1-rls-r1-bm-pr-1.md` schema verbatim. Differences:
- **Scope:** m1-a is a greenfield crate (services/bridge/) + Tree C docs; no Lemmy workspace Rust.
- **Workspace exclusion:** R8 boundary — never `--workspace`, never `--features full` for this crate.
- **Phase-2 e2e:** OMITTED (zero workspace Rust edits; Tree A is excluded from Lemmy workspace).
- **Docker-compose validate:** ADDED in §Validation (Task 13's unique integration-test scaffold validate step).
- **Task 14 advisor-direct:** noted (doc-only task authored by advisor without Junior dispatch).
- **SQLITE3_LIB_DIR note:** documented in Task 13 DQ annotation (environment gap on Windows; requires conda sqlite3.lib).

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; m1a worktree at `brehon-fork-m1a`). Brief committed on `governance-v0`. bm-task worker uses `base_branch=governance-v0`._
