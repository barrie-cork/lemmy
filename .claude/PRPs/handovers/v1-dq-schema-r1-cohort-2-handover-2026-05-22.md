# v1-dq-schema-r1 — Cohort 2 (Task 3) handover

**Author:** advisor session (laptop, `C:/Users/barri/Developer/brehon-fork`, branch `governance-v0`)
**Written:** 2026-05-22 (pre-compact handover)
**Origin tip:** `891256058` — Cohort 1 fully validated + merged.

## Why this file exists

Pre-`/compact` handover capturing live state of v1-dq-schema-r1 so the next session resumes cleanly without scanning the full conversation. Read this first on resume.

## Sub-phase context

Sub-phase `v1-dq-schema-r1` ships DQ schema-v3 (composite id `<session_id>-<seq>` + `approved_by`/`approved_at` fields) to structurally eliminate id-collision races and close CR-1 audit gap from PR #141. Source: GitHub issue #142.

- **Plan:** `.claude/PRPs/plans/v1-dq-schema-r1.plan.md` (5 tasks; complexity 0/10; direct-commit policy on `governance-v0` — no phase branch, no PR).
- **Brief:** `.claude/PRPs/briefs/v1-dq-schema-r1-planning-1.md` (with §0.1 PRECON-1..5 BINDING).
- **Clarify pass:** DQ #330-#336 resolved at `02b5a99ed`.

## Status as of 2026-05-22 pre-compact

| Task | Track | State | Commit(s) on governance-v0 |
|---|---|---|---|
| Task 0 | pre-flight (inline) | ✓ run by impl-task subagent at start of each task | n/a |
| Task 1 [P] | A — scripts + migration | ✓ done + validated pass | feature `92e0ed425`, DQ raise `2128dddbf`, merge `0cff8150d` |
| Task 2 [P] | B — 8 doc updates | ✓ done + validated pass | feature `1803b8546`, DQ raise `496a018f5`, merge `9f7ba68bb` |
| Task 3 | C — resolve-dq-canonical.sh | **NEXT: brief authoring + dispatch** | (pending) |
| Task 4 | retro | (pending; queue after Task 3) | (pending) |

**Live DQ state (origin governance-v0 @ `891256058`):**
- `schema_version: 3` ✓
- 133 entries migrated with `id_v1` + `approved_by: null` + `approved_at: null`
- First v3 native composite id: `ae5ff4ec3e8c-001` (validated pass, in resolved[])
- Task 2's int DQ #339 (validated pass, in resolved[])
- Pending: **[338]** (daemon-finalize-resets-trunk bug; recommendation appended by user 2026-05-21 in cf93b7ba6; awaiting user-relay decision on option-a structural fix)

## Critical context for Task 3 dispatch

### 1. Daemon reset-bug (DQ #338) is active

The Junior daemon's finalize-merge step has a confirmed bug class documented in `.claude/lessons/feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md` (authored by user in `cf93b7ba6`). Symptoms:
- Daemon merges worker into local `governance-v0`, then resets `governance-v0` to a phase-branch tip (e.g. `origin/phase-v1-federation-inbound-c`).
- Plan/feature commits become unreachable.
- Origin protected ONLY because reset happens before push.

**Cohort 1 mitigation used:** advisor-laptop manual finalize-merge per the recovery recipe (see "Recovery recipe" in the lesson). Workers pushed to origin; laptop pulled both worker branches; merged with `--no-ff`; resolved DQ conflicts semantically; pushed.

**For Task 3:** same risk applies. EliteDesk daemon HEAD as of last check was on `phase-v1-federation-inbound-c` (from fed-in-c parallel-lane finalize `877bd849c`). Two options for Task 3:
- **Option A (recommended):** dispatch Junior Task 3 normally; advisor-laptop manual finalize if daemon HEAD still wedged.
- **Option B:** skip Junior entirely; advisor-laptop authors Task 3's change directly (it's a 2-line sort-key change + ~4-line header comment in `scripts/brehon/resolve-dq-canonical.sh`). Faster and avoids daemon entirely. **NOTE: This violates the "advisor never authors content" rule — needs user approval before taking option B.**

### 2. Cross-session activity

A parallel user/advisor session was active 2026-05-21:
- Authored `cf93b7ba6` (DQ #326 → resolved, DQ #338 recommendation appended, lesson file `feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md` created)
- Manually finalized fed-in-c Task 3 via `877bd849c`
- Likely still running; coordinate on Task 3 dispatch.

### 3. Task 3 specs

From plan §13 Task 3 (lines 544-586 of `.claude/PRPs/plans/v1-dq-schema-r1.plan.md`):

**FILES YAML:**
```yaml
creates: []
modifies:
  - scripts/brehon/resolve-dq-canonical.sh
requires:
  - task: 1  (str(e['id']) sort tested against v3-migrated DQ)
  - task: 2  (header cites .claude/rules/decision-queue.md §Schema (v3) — Task 2 introduced)
```

Tasks 1+2 are already merged at `0cff8150d` and `9f7ba68bb` → requires satisfied.

**Two edits in one file:**
1. Insert "Schema-v3 note (post-v1-dq-schema-r1, 2026-05-21)" paragraph at lines 25-35 of `scripts/brehon/resolve-dq-canonical.sh` (between "worker-branch wins on collision (most recent)." and "Usage:").
2. Change lines 179-180 `key=lambda e: e['id']` → `key=lambda e: str(e['id'])` (both `acc['pending']` and `acc['resolved']` sorts).

**DoD per plan §15.2 / §16a Story 3:**
```bash
bash -n scripts/brehon/resolve-dq-canonical.sh    # exit 0
grep -c "key=lambda e: str(e\['id'\])" scripts/brehon/resolve-dq-canonical.sh   # 2
grep -c "key=lambda e: e\['id'\]" scripts/brehon/resolve-dq-canonical.sh         # 0
grep -c "Schema-v3 note (post-v1-dq-schema-r1" scripts/brehon/resolve-dq-canonical.sh  # ≥1
bash scripts/brehon/resolve-dq-canonical.sh v1-dq-schema-r1 >/dev/null 2>&1     # exit 0 smoke
```

## Recommended next actions on resume

1. Read `.claude/PRPs/plans/v1-dq-schema-r1.plan.md` §13 Task 3 + §15.2 + §16a Story 3 (lines 544-836).
2. Read this file (`v1-dq-schema-r1-cohort-2-handover-2026-05-22.md`) fully.
3. Check `mcp__junior-brehon__list_tasks` for any new running tasks (parallel session).
4. Check `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` for any divergence.
5. Surface to user via `AskUserQuestion`: option-A (Junior dispatch with manual-finalize fallback) vs option-B (advisor-direct authorship, requires user approval).
6. On option-A: author `.claude/PRPs/briefs/v1-dq-schema-r1-impl-3.md` (use `v1-dq-schema-r1-impl-1.md` and `-impl-2.md` as templates); commit on `governance-v0`; dispatch Junior `[role:impl-task]` with `base_branch: governance-v0`.
7. On option-B: edit `scripts/brehon/resolve-dq-canonical.sh` directly; run DoD locally; commit on governance-v0; advance to Task 4 (retro).
8. After Task 3 ships + validates pass: queue Task 4 (retro).

## Pinned references

- `.claude/PRPs/plans/v1-dq-schema-r1.plan.md` @ `f746a00b9` (last touched)
- `.claude/PRPs/briefs/v1-dq-schema-r1-planning-1.md` @ `02b5a99ed` (clarified)
- `.claude/PRPs/briefs/v1-dq-schema-r1-impl-1.md` + `-impl-2.md` @ `74dc2d5e7` (templates for impl-3)
- `.claude/lessons/feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md` @ `cf93b7ba6` (recovery recipe)
- `.claude/rules/decision-queue.md` (now has Schema (v3) § appended at line ~28 post-merge)
- `.claude/refs/dq-recipes.md` (Recipe 1+2 swapped to `dq-v3-new-entry.sh`)
- `scripts/brehon/dq-schema-v3-migrate.sh` + `dq-v3-new-entry.sh` (NEW; shipped Task 1)

## Issue #142 RCA summary (for retro)

Two root causes addressed:
1. **Integer-id race** — 4 confirmed collisions in one month (v1-rls-r1 #50, v1-fed-in-a finalize-renumbers, v1-conformance-audit #318, PR #141 #320). Composite id `<session_id>-<seq>` makes collisions arithmetically impossible.
2. **`answered_by` records resolver-not-approver** — CR-1 on PR #141. Added `approved_by` / `approved_at` fields; advisor-exclusive after user-gate relay; Junior subagents hard-refuse.

## Daemon bug (DQ #338) incidents observed during this sub-phase

1. **Task #399 (planning):** daemon reset gov-v0 to `origin/phase-v1-federation-inbound-c`; plan-commit `b1a6553b8` became unreachable; recovered via cherry-pick → `c02dc8617` (daemon) → push to `recovery/v1-dq-schema-r1-plan` → pull-cherry-pick on laptop → `f746a00b9`. Recovery: ~25 min.
2. **Task #401 (Track B impl):** daemon worker finished + pushed; daemon shows stale `running`; finalize-agent never started (daemon HEAD wedged on phase-v1-federation-inbound-c). Recovered via advisor-laptop manual `--no-ff` merge of both worker branches. Recovery: ~15 min (incl. semantic DQ conflict resolution).

Both incidents documented in `feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md`.
