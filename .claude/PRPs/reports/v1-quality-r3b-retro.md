# Retro — v1-quality-r3b

**Phase:** v1-quality-r3b — Capture DB URL at LemmyContext::create (Issue #167)  
**PR:** #170 — merged `2026-05-31T14:17:09Z` @ `c672781cd`  
**Duration:** ~1 session (2026-05-31)  
**Roles active:** advisor, impl-task (Junior #543), bm-task (bm-cut, bm-pr #546, bm-merge #547)

---

## What surprised us

**Single-task phase required a merge-forward before bm-merge.** governance-v0 had moved 13 commits ahead of the phase branch during the same session (DQ archiving, rule edits, session-retro work all landed on trunk while the phase was in flight). The merge-forward (`d1fb230f1`) was straightforward — DQ conflict resolved as theirs (archived DQ) + carry `be6ddc108436-001` (the active validate-pending-laptop entry). But it was unexpected for a 1-task phase that landed in a few hours. Root cause: this session was unusually busy on trunk (4 rule-edit commits + 3 retro commits) while quality-r3b was in flight. No process failure; the merge-forward procedure worked cleanly.

**Mode B→A flip mid-phase.** Session started in Mode B (canonical `brehon-fork` checkout, no lane worktree). After the impl finalize-merge, a lane worktree `brehon-fork-quality-r3b` was added to run the e2e locally (validate-pending-laptop). The flip worked but added a bootstrap step (submodules + `.mcp.json` + `.env`). For a single-task fix phase, Mode B throughout (dispatch e2e as a gh workflow run) would have been simpler — but Shape G is suspended, so laptop e2e was mandatory.

---

## What to change

**Brief the mode decision in the bootstrap handover.** The v1-quality-r3b bootstrap handover did not explicitly state Mode A or Mode B. The session had to infer it. For future single-task fix phases, state in the handover: "Mode B — no lane worktree needed; all phase-branch work via Junior dispatch from canonical checkout" (or Mode A if e2e is expected). Saves one decision cycle at session start.

**Retro-bypass.jsonl check at session start.** Not done at this session's start (context was tight from the compaction). Should be a 5-second read per `feedback_outcome_not_cause_check_retro_bypass.md`.

---

## What to carry forward

**EnvVarGuard fixture-lifetime footgun is now fixed at root.** `feedback_envvarguard_fixture_lifetime_footgun.md` documents the pattern; the lesson is already in the corpus. Any future handler that reads env vars should capture them at `LemmyContext::create()` time and expose via accessor — the same pattern established here. Planner briefs for new SSE/streaming handlers should reference this lesson.

**§16a spec drift (advisory).** The plan §16a said "exactly ONE `EnvVarGuard::set("LEMMY_DATABASE_URL"…)`" but the correct post-fix count is 14. DQ `kind: log` `a3d0e9941441-042` was filed. At the next plan authoring pass, the planner should update §16a wording. Low priority — the behavioural intent passed; the count mismatch was in the descriptive text only.

**merge-forward before bm-merge is routine, not exceptional.** governance-v0 moves constantly (rule edits, retros, DQ answers). For phases where the phase branch lives for >1 day, expect to merge-forward at least once before bm-merge. The advisor should check `git log origin/governance-v0 ^phase-v1-<phase>` proactively after `/brehon-verify` rather than waiting for bm-pr to flag CONFLICTING.

---

## Per-role signals (four-role model)

**Advisor:** Handled merge-forward cleanly. One miss: didn't check retro-bypass.jsonl at session start. BM post-condition checks all passed (5-signal table satisfied). Cron job cancelled post-merge.

**Planning (Junior #545 — this phase had a prior planning cycle):** Plan was well-structured: single task, clear scope, verbatim anchor citations for e2e edits, Option A fix shape with no callsite enumeration needed. Complexity score appropriate for a 1-task fix phase.

**Impl (Junior #543):** Delivered cleanly in one pass. All 6 structural output checks in the verify report passed. No DQ blockers raised. The pre-locate anchor lesson (`feedback_fix_impl_pre_locate_e2e_anchors.md`) was in the brief and appears to have been effective — no Edit-hang, no anchor collision.

**BM (bm-cut, #546 bm-pr, #547 bm-merge):** All three tasks succeeded with correct post-conditions. bm-pr opened PR with correct base (`governance-v0`), not draft. bm-merge used `--merge` (no squash). Runlog entry written. One false-start: #546 reported UNSTABLE after merge-forward; BM correctly noted CONFLICTING state in the PR and the advisor handled the merge-forward before re-dispatching.

---

## Task complexity metrics (§6 — four-role retro signals)

| Task | Role | Files | Commits | Runtime (min) | Max-log-silence (min) |
|---|---|---|---|---|---|
| T1 impl | impl-task | 3 | 1 | ~15 | ~5 |
| bm-cut | bm-task | 0 | 0 | ~3 | — |
| bm-pr | bm-task | 0 | 0 | ~3 | — |
| bm-merge | bm-task | 0 | 0 | ~2 | — |
| e2e laptop | advisor | 0 | 1 (DQ) | ~47 | ~47 |

Full phase wall-clock: ~4 hours (dominated by e2e run time + merge-forward + compaction boundary).

---

## Lessons promoted / filed

- DQ `a3d0e9941441-042` (`kind: log`) — §16a spec drift "exactly ONE" → "exactly 14"; harvest at next plan authoring.
- No new `.claude/lessons/` files authored this phase (the fix was clean; the root-cause lesson `feedback_envvarguard_fixture_lifetime_footgun.md` already exists from v1-quality-r2).
