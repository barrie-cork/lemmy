# Impl handover — 2026-05-23 — ship3-tasks-in-flight

**Written:** 2026-05-23T14:30Z (approx)
**Author:** advisor session (C:/Users/barri/Developer/brehon-fork-ship-3)
**Branch:** phase-v1-ship-3 @ d015525bd
**Plan:** .claude/PRPs/plans/v1-ship-3.plan.md
**Purpose:** Self-contained brief for next-session resume after advisor session close.

## TL;DR

- Task 0 (pre-flight) not yet formally run as a Junior task — probes were verified manually by advisor (all passed).
- Task 1 (docker-compose.yml) DONE — commit `2fe9c17c7` on homeserver worker branch `junior/.../438`; needs cherry-pick onto `phase-v1-ship-3`.
- Task 2 (POST /report DTO) IN FLIGHT — Junior task #439 running on homeserver; wrong base branch (RT-r2 ancestry); commit will need cherry-pick when done.
- Task 3 (2-sponsors e2e) NOT STARTED — must queue after Task 2 lands.
- Task 4 (retro) NOT STARTED.
- **Base-branch contamination:** daemon default HEAD is `phase-v1-RT-r2`; `baseBranchOverride` in Junior MCP is NOT honoured (workers branch from daemon default regardless). Both #438 and #439 have RT-r2 history in ancestry. Recovery: cherry-pick actual work commits onto `phase-v1-ship-3` after each worker finishes.

## Cold-resume sequence

1. Read `CLAUDE.md` + `.claude/rules/*.md` (auto-loaded).
2. Read this file in full.
3. Read `.claude/PRPs/plans/v1-ship-3.plan.md` — focus on §13 Task 2 (in-flight) and Task 3 (next).
4. Check Junior task status:
   ```
   mcp__junior-brehon__list_tasks (status: running)
   mcp__junior-brehon__list_tasks (status: done)
   ```
5. Verify git state in ship-3 worktree:
   ```bash
   git -C C:/Users/barri/Developer/brehon-fork-ship-3 branch --show-current   # expect phase-v1-ship-3
   git -C C:/Users/barri/Developer/brehon-fork-ship-3 rev-parse --short HEAD  # expect d015525bd
   git -C C:/Users/barri/Developer/brehon-fork-ship-3 status --short          # expect clean
   git -C C:/Users/barri/Developer/brehon-fork-ship-3 log governance-v0..HEAD --oneline
   # expect: d015525bd docs(plan): v1-ship-3 plan written — Tactical polish bundle
   ```
6. Check homeserver worker branches:
   ```bash
   ssh homeserver "git -C /srv/brehon-fork log --oneline junior/role-impl-task-v1-ship-3-task-1-document-postgres-image-intent-in-docker-compose-yml-438 | head -3"
   ssh homeserver "git -C /srv/brehon-fork log --oneline junior/role-impl-task-v1-ship-3-task-2-post-report-returns-governancecasesummaryview-439 | head -3"
   ```

## State at handover

### Git
- Branch: `phase-v1-ship-3` (tracking `origin/phase-v1-ship-3`)
- HEAD: `d015525bd` (`docs(plan): v1-ship-3 plan written — Tactical polish bundle`)
- Commits ahead of `governance-v0`: 1 (plan commit only)
- Unpushed to `origin/phase-v1-ship-3`: 0 (plan commit was pushed)
- Working tree: clean
- Stash: 2 entries (pre-existing on governance-v0, unrelated to ship-3)

### Plan position
- Active plan: `.claude/PRPs/plans/v1-ship-3.plan.md`
- Tasks completed: none fully on `phase-v1-ship-3` yet
- Task 0: advisor-verified manually (all 12 probes passed — see advisor session notes)
- Task 1: done on worker branch `junior/.../438` (commit `2fe9c17c7`) — NOT yet cherry-picked to phase branch
- Task 2: in-flight on worker branch `junior/.../439` — NOT yet done
- Task 3: not started (serial after Task 2)
- Task 4 (retro): not started

### Junior tasks
- **#438** (Task 1 — docker-compose.yml): status=running (likely done — commit exists). Worker branch: `junior/role-impl-task-v1-ship-3-task-1-document-postgres-image-intent-in-docker-compose-yml-438`. Commit: `2fe9c17c7 feat(docker): pin postgres image intent (task 1)`. **BASE WRONG** — has RT-r2 ancestry. Cherry-pick needed.
- **#439** (Task 2 — POST /report DTO): status=running. Worker branch: `junior/role-impl-task-v1-ship-3-task-2-post-report-returns-governancecasesummaryview-439`. **BASE WRONG** — has RT-r2 ancestry. Cherry-pick needed when done.
- **#435** (RT-r2 Task 2): status=running — separate lane, unrelated to ship-3.

### Known daemon issue
`baseBranchOverride` in `mcp__junior-brehon__create_task` is NOT honoured — daemon always branches from its local default HEAD (currently `phase-v1-RT-r2`). Workaround for future dispatches: update daemon's local HEAD before queuing, OR cherry-pick work commits after completion. The cherry-pick approach is cleaner since it avoids touching the daemon's HEAD while RT-r2 Task 2 (#435) is still running.

### Background jobs
No `.claude/build-*.log` or `.claude/audit-*.log` files present in ship-3 worktree.

### PR / CR
No open PR on `phase-v1-ship-3` yet — BM will open after Tasks 1-3 land.

## What's pending (ordered)

**1. Wait for #439 (Task 2) to complete**
- Context: Task 2 is the heaviest task — 4 files, new `read_summary_for_case` fn, DTO reshape, e2e assertion update.
- Check: `mcp__junior-brehon__list_tasks(status: "done")` — when #439 appears, inspect with `mcp__junior-brehon__show_task 439`.
- Expected outcome: commit on `junior/.../439` branch with `feat(api): POST /report returns governance case summary view (task 2)`.

**2. Cherry-pick Task 1 commit onto `phase-v1-ship-3`**
- Context: commit `2fe9c17c7` is the real Task 1 work; only `docker/docker-compose.yml` was modified.
- Command:
  ```bash
  git -C C:/Users/barri/Developer/brehon-fork-ship-3 cherry-pick 2fe9c17c7
  git -C C:/Users/barri/Developer/brehon-fork-ship-3 push origin phase-v1-ship-3
  ```
- Validate after: `grep -E "image:.*(latest|nightly)" docker/docker-compose.yml` → only `lemmy-ui:nightly`.
- Expected outcome: `phase-v1-ship-3` has 2 commits ahead of `governance-v0`.

**3. Identify Task 2 work commit and cherry-pick onto `phase-v1-ship-3`**
- Context: same base-branch contamination as Task 1 — find the actual `feat(api):` commit on the worker branch.
- Command pattern:
  ```bash
  ssh homeserver "git -C /srv/brehon-fork log --oneline junior/role-impl-task-v1-ship-3-task-2-post-report-returns-governancecasesummaryview-439 | head -5"
  # identify the feat(api): commit SHA
  git -C C:/Users/barri/Developer/brehon-fork-ship-3 cherry-pick <SHA>
  ```
- Validate after: run §15.1-§15.4 DoD gates (cargo check → clippy → --no-run → report_to_modlog_golden_path e2e).

**4. Run Task 2 DoD gates (laptop)**
- Commands from plan §15:
  ```
  cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"
  cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"
  cmd //c "scripts\brehon\cargo-test.bat --workspace --features full --test e2e --no-run"
  cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full report_to_modlog_golden_path"
  ```
- All must exit 0. Run from `C:/Users/barri/Developer/brehon-fork-ship-3`.

**5. Queue Task 3 (2-sponsors e2e)**
- Context: serial after Task 2. Appends `mod v1_ship_3_fixtures` at EOF of `e2e.rs`.
- **Before queueing:** update daemon's local ship-3 ref OR accept cherry-pick workflow again.
  ```bash
  ssh homeserver "git -C /srv/brehon-fork fetch origin phase-v1-ship-3:phase-v1-ship-3 --update-head-ok"
  ```
- Then dispatch with `base_branch: "phase-v1-ship-3"` (note: override may still not be honoured — monitor).
- Task 3 brief is in plan §13 Task 3. Key points:
  - Append `mod v1_ship_3_fixtures` at EOF (after line 16692)
  - Seed 2 sponsors at `endorsement_strength=10`, drive ContentRemoval pipeline
  - Read delta from `governance_config` key `"deltas.sponsor_liability_moderate"` at runtime
  - Assert both sponsors land at `endorsement_strength == 0`
  - Test name must be exactly: `two_sponsors_lose_endorsement_strength_on_sanction`

**6. Run Task 3 DoD gates (laptop)**
- After cherry-pick of Task 3 commit:
  ```
  cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full two_sponsors_lose_endorsement_strength_on_sanction"
  cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full v1_ship_3_fixtures"
  cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full"   # full regression
  ```

**7. BM: open PR + CR triage + merge**
- After Tasks 1-3 green: dispatch bm-pr session against `governance-v0`.
- Then retro (Task 4).

## What NOT to touch

- `.claude/PRPs/plans/v1-ship-3.plan.md` — advisor-owned; do not amend.
- `.claude/decision-queue.json` — advisor-only writes.
- `.claude/runlog/bm-runlog.md` — BM-owned (lives on `governance-v0`).
- Any files outside the §11 list: `docker/docker-compose.yml`, `crates/db_views/governance_case/src/impls.rs`, `crates/api/api_common/src/governance.rs`, `crates/api/api_crud/src/governance/create_report.rs`, `crates/server/tests/e2e.rs`.
- RT-r2 lane files — separate worktree at `C:/Users/barri/Developer/brehon-fork-rt-r2`.

## Bootstrap prompt (paste into next session)

```
I'm resuming impl work on the Brehon governance fork (v1-ship-3 — tactical polish bundle).
Previous session wrote a handover at `.claude/PRPs/handovers/impl-2026-05-23-ship3-tasks-in-flight.md`.

First steps:
1. Read CLAUDE.md + .claude/rules/*.md (auto-loaded in -p mode).
2. Read `.claude/PRPs/handovers/impl-2026-05-23-ship3-tasks-in-flight.md` in full.
3. Execute its "Cold-resume sequence" in order.
4. Report the TL;DR summary plus any state drift you detected back to me.

Do not take any state-changing actions until I confirm.
```

## Closing state assertions (verify on resume)

- `git -C C:/Users/barri/Developer/brehon-fork-ship-3 branch --show-current` → `phase-v1-ship-3`
- `git -C C:/Users/barri/Developer/brehon-fork-ship-3 rev-parse HEAD` → `d015525bd07651dbb7469e2b8c395c3553060ce2`
- `git -C C:/Users/barri/Developer/brehon-fork-ship-3 rev-parse origin/phase-v1-ship-3` → same (plan commit was pushed)
- `git -C C:/Users/barri/Developer/brehon-fork-ship-3 status --short` → (empty — clean)
- `git -C C:/Users/barri/Developer/brehon-fork-ship-3 log governance-v0..HEAD --oneline | wc -l` → `1`
- `test -f C:/Users/barri/Developer/brehon-fork-ship-3/.claude/PRPs/handovers/impl-2026-05-23-ship3-tasks-in-flight.md` → exists
- `test -f C:/Users/barri/Developer/brehon-fork-ship-3/.claude/PRPs/plans/v1-ship-3.plan.md` → exists
