# Handover — v1-RT-r3 fix-impl-2 recovery

**Author:** advisor (laptop session, `brehon-fork-rt-r3` Mode A lane)
**Date:** 2026-05-26T19:05Z (approx — set when next session resumes)
**Reason:** session ending; Junior #479 failed; next session resumes triage

## RESUME block (read this first if you walked into this cold)

- **Current sub-phase:** v1-RT-r3 (multi-source participation_consistency events + flag-bad-faith admin endpoint)
- **Phase branch:** `phase-v1-RT-r3`
- **Lane mode:** A (dedicated worktree at `C:/Users/barri/Developer/brehon-fork-rt-r3` on `phase-v1-RT-r3`)
- **PR:** [#155](https://github.com/barrie-cork/lemmy/pull/155), state OPEN, mergeStateStatus CLEAN, mergeable MERGEABLE
- **Last commit on phase branch:** `040730def` (chore(advisor): brief v1-RT-r3 fix-impl-2 — cr-1 DQ boundary + cr-3 iso-week guards)
- **Stage in state machine:** CR-triage post-`bm-poll-cr` — first fix-impl dispatch FAILED; need user-gate retry decision
- **Next concrete action:** read this handover, then surface the 3 triage options to user (see §"Next concrete action" below)

## What shipped this session

1. **PR #155 opened** by Junior bm-task #478 — phase-v1-RT-r3 → governance-v0 — `87285bbfe` head, includes Task 4 advisor-authored carve-out (10 e2e tests in v1_rt_r3_fixtures)
2. **CR + Copilot review ingested** 2026-05-26T17:31:51Z — 12 findings (1 outside-diff Critical, 1 Major, 3 Medium, 6 Minor, 1 Nit)
3. **Four-bucket triage drafted** + user-gate 3 approved option (a):
   - `.claude/PRPs/reviews/pr-155-findings.yaml` (schema-v1, 12 findings)
   - `.claude/PRPs/reviews/pr-155-comment.md` (triage table + recommended grouping)
   - **5 fix-in-pr** (cr-1 + cr-3 → fix-impl-2; cp-1/2/3 → fix-impl-3)
   - **5 carry-forward** (cr-2, cr-4, cr-7, cp-4, cp-5 → defer to v1-quality-r2)
   - **2 wont-fix** (cr-5 brief markdownlint, cr-6 runlog MD022 — rationale captured)
4. **fix-impl-2 brief authored + committed**: `.claude/PRPs/briefs/v1-RT-r3-fix-impl-2.md` at `040730def`
5. **Junior #479 dispatched** with `base_branch=phase-v1-RT-r3` at 2026-05-26T18:18:59Z

## What failed: Junior #479 (fix-impl-2)

- **Status:** failed at 2026-05-26T18:56:44Z (~38 min runtime, $8.30, 151 turns)
- **Stop reason:** `error_max_turns` (Reached maximum number of turns (150))
- **Branch on origin:** **ABSENT** — `git ls-remote origin refs/heads/junior/role-impl-task-v1-rt-r3-fix-impl-2-...479` returns nothing. Worker never pushed; recovery via tar (per advisor-orchestrator.md §5.6) NOT required since task is already failed (not cancelled); the daemon's auto-reap may or may not have run — worth a check next session: `ssh homeserver "ls /srv/brehon-fork/.junior/worktrees/job-479 2>/dev/null && echo PRESENT || echo REAPED"`.
- **What the worker did:** spent 151 turns reading helpers (Post::create, Comment::create, admin_assign_jury, AdminAssignJuryResponse, governance_log schema, ReputationEventSourceType enum) — none in fix-impl-2's scope.
- **Crucial:** worker's FINAL Edit tool call (in `permission_denials` array) shows the EXACT correct JSON Edit recipe per brief §2.2 — `"failed_commands": null\n    },\n    {\n      "from": "advisor"`. Worker found the right answer; just ran out of turns mid-Edit application.
- **Permission_denials block contents:**
  - 4× `bash scripts/brehon/cargo-check.sh --workspace --features full` (Junior daemon allowlist blocked the .sh path; brief specified .bat which is Windows-only)
  - 1× the final Edit on `.claude/decision-queue.json` (denied because Junior session hit max_turns before the tool call completed; not a permission policy denial)

## Failure class identification

**Same shape as v1-RT-r3 Task 4 cycle (#474–#477)** that triggered §5.3 cycle-count HARD REFUSAL on 2026-05-26 ~05:00Z. That cycle authorised the advisor-side Task 4 carve-out (DQ -033). Pattern:

- Sonnet impl-task model
- Target file `crates/server/tests/e2e.rs` (17000+ lines)
- Worker reads many adjacent helpers + types, drifts off-brief
- `error_max_turns` before any commit
- Recipe correctness is NOT the issue — context-finding-cost-vs-budget IS

This is **cycle 1 for fix-impl-2** — cycle-count meta-rule does NOT auto-HARD-REFUSE. But the failure class is identical. Retry with the same brief shape is +75% likely to fail the same way (gut-level estimate; no formal model).

## 3 paths surfaced to user (decision pending)

(These are the same 3 paths the user interrupted before answering — re-surface on resume.)

### Path A — Split into 2 narrower briefs + retry

Two new briefs:

- **fix-impl-2a** = ONLY `.claude/decision-queue.json:4091` boundary fix (~50 lines brief, 3 lines edit)
- **fix-impl-2b** = ONLY `crates/server/tests/e2e.rs` iso-week guards (~80 lines brief, 2 Edit calls × ~12 lines each)

Each dispatched separately. Lower per-task scope below the Sonnet-drift threshold. ~10 min advisor time to split the brief.

**Risk:** Path A still risks the same drift on 2b (e2e.rs is still 17000 lines; worker still has to find the right anchor). Mitigation in 2b brief: pre-locate the EXACT 5-line `old_string` for each Edit (use sufficient distinctive context — the seed counts `4` vs `3` in dormancy fixtures should anchor uniquely).

### Path B — Advisor-side carve-out for both fixes (extend DQ -033 scope)

Advisor authors locally in this/next session:
- JSON Edit on `.claude/decision-queue.json` (3 lines)
- 2 Edits on `crates/server/tests/e2e.rs` (hoist iso-week guards)
- Pre-push cargo-check on laptop
- Single commit `fix(governance): repair DQ entry boundary + hoist iso-week guards in v1_rt_r3_fixtures (fix-impl-2, advisor carve-out)`
- Push

Total: ~15 min advisor time, ~0 Junior cost. **Bypasses the failure class entirely.**

**Risk:** extends the DQ -033 carve-out scope (was "Task 4 e2e tests"; this extension is "+CR-triage fix-in-PR fixes when Junior fails"). User must re-authorise. Sets precedent that may erode the four-role discipline if invoked repeatedly. The "good session" framing suggests user is open to a clean ship via carve-out.

### Path C — Advisor JSON only + cr-3 → carry-forward

Advisor authors ONLY the JSON fix (3 lines, ~5 min). Move cr-3 (iso-week guard) from fix-in-pr → carry-forward (sweep into v1-quality-r2 along with cr-2/cr-4/cr-7/cp-4/cp-5).

**Recommended IF time is the constraint and the 🟡 Minor classification holds** — cr-3 is a week-boundary flake; tests pass 51 of 52 weeks. Deferring it is low-risk.

**Risk:** minimal — the 🔴 Critical lands; the 🟡 Minor waits. fix-impl-3 (config clamps) still needs to happen before merge regardless.

## Recommended next-session opening

After reading this handover:

1. **Verify state** (1 min):
   ```bash
   git log -1 --oneline origin/phase-v1-RT-r3  # should be 040730def
   gh pr view 155 --repo barrie-cork/lemmy --json state,mergeStateStatus
   ```
2. **Surface the 3 paths** to user via AskUserQuestion (re-use the wording in this file's §"3 paths surfaced").
3. **Recommendation if user defers:** **Path C** — gets the critical landed fastest, lowest carve-out scope. Then fix-impl-3 for the 3 config clamps (those are still mechanical + likely to need either advisor authorship OR a much narrower per-clamp brief). User-gate 5 (merge confirm) → bm-merge.

## Cross-session dependencies / DQ pending

- **DQ pending at handover:** 0 (verified via cat-file blob).
- **DQ ids consumed this session:** None new (no validate-pending entries written this session; fix-impl-2 worker never reached the DQ-write step).
- **Concurrent sessions:** 1 (the canonical `brehon-fork` checkout was idle when this session checked at 2026-05-26 mid-session; no commits on `governance-v0` since `40e4a4f6` 2 days ago — verify with `git log -1 origin/governance-v0` on resume).

## Files modified this session (committed)

- `.claude/PRPs/reviews/pr-155-findings.yaml` (created, gitignored — runtime artifact only; NOT staged)
- `.claude/PRPs/reviews/pr-155-comment.md` (created, gitignored — runtime artifact only; NOT staged)
- `.claude/PRPs/briefs/v1-RT-r3-fix-impl-2.md` (created, committed at `040730def`, pushed to `phase-v1-RT-r3`)
- This handover file (will be committed next; see below)

## Files modified this session (uncommitted at write time of this handover)

- `.claude/PRPs/handovers/v1-RT-r3-fix-impl-2-recovery-2026-05-26.md` (this file)

The handover MUST be committed + pushed before session-end per `advisor-orchestrator.md` §1 "Pre-compact handover discipline".

## Memory + retro state

- Eval ID 590 (Task 4 carve-out retro) + Eval ID 592 (CR triage + fix-impl-2 dispatch retro) written to PMD.
- Stop hook satisfied; no bypasses tracked.

## See also

- `.claude/PRPs/reports/session-retro-2026-05-09-cycle-3-catchfire-replan.md` — historical precedent for cycle-count HARD REFUSAL after same-class Junior failures.
- `.claude/PRPs/handovers/v1-RT-r3-task4-advisor-authorship-2026-05-26.md` — the DQ -033 carve-out handover this session built on.
- `.claude/decision-queue.json` (resolved entries from this session: NONE — no answered_by:advisor commits this session).
- `.claude/PRPs/briefs/v1-RT-r3-fix-impl-2.md` — the brief that failed; reuse or revise per next session's path choice.
