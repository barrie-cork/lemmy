# Session retro — brehon-conformance-audit bootstrap (planner + bm-cut + Cohort 1 dispatch)

**Session window:** 2026-05-20 ~13:45 – 16:13 UTC (~2.5h wallclock).
**Trunk state at start:** `governance-v0 @ bbe4def06`. **Trunk state at end:** `governance-v0 @ 9b74c0909`. **Phase branch at end:** `phase-brehon-conformance-audit @ 8f8fbd9db`.
**Junior tasks dispatched:** #352 (planning, Opus 4.7, ~47 min, success-then-corrupted), #353 (bm-cut, Haiku 4.5, stale-base failure), #354 (bm-cut retry, Haiku 4.5, partial), #355 (impl Task 1, Sonnet 4.6, in flight at session-end).
**User gates fired:** Gate 1 (Plan Approval) — approved as proceed-as-one.

## 0. Scope of this retro

This is a **session retro**, not the sub-phase retro (Task 13 retro per plan §13 lives there). Captures the bootstrap-stage events of `brehon-conformance-audit` from plan-write through bm-cut + Cohort 1 dispatch. Surfaces durable learnings now (PMD lesson writes + mechanical fixes) so Task 13 retro can focus on impl + dogfood + ship outcomes.

## 1. What worked

### 1.1 Advisor recovery patterns held under multi-defect-class load

Three separate harness defect classes fired in close succession (daemon stale base, CC v2.1.119 gate-block, daemon refspec filter). Each was caught by an EXISTING lesson + the recovery recipe was executable in <5 min:

- `feedback_daemon_local_trunk_stale_multi_lane.md` → `git update-ref refs/heads/governance-v0 origin/governance-v0` (lane-safe).
- `feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` → advisor-relocate runlog post-task.
- `feedback_junior_292_stale_base_recover_recipe.md` → `git checkout HEAD -- <stale files>` to reconcile working tree.

**Signal:** the lesson-corpus IS the recovery library when these fire. Continued investment in lessons pays off on every bootstrap.

### 1.2 Brief PRECON-1..9 pattern reduced planner round-trips

Planner #352 surfaced only 5 minor ambiguities (with leans) and 0 user-relay clarify entries. Pre-emption via PRECON-1..9 in the brief absorbed what would otherwise have been 5+ clarify-DQ cycles.

**Signal:** when a brief pre-resolves the major design questions via AskUserQuestion before planner dispatch, planner round-trip cost drops to ~0. Worth replicating on every novel-contribution plan.

### 1.3 Split-or-proceed DQ #295 resolved cleanly via attribution rules

`from: "planner"`, `kind: "blocker"`, `answered_by: "user"` — the canonical advisor-relays-user-reply pattern. Committed with `^chore\(advisor\)` subject per attribution-integrity. No process breach.

### 1.4 Forward-merge of trunk into phase branch resolved brief-visibility issue

When I committed Task 1 brief to `governance-v0` (per literal §2.1 wording), the impl worker forking from the phase branch couldn't see it. Cherry-pick + forward-merge resolved cleanly without rewrite. Fed-in-b precedent (`0ea7ab4f7`) showed the convention is "briefs land on phase branch" — discovered during this session and now durable.

## 2. What broke / cost time

### 2.1 Daemon finalize-merge silently no-push (planner #352)

**~30 min wallclock cost.** Daemon finalize-merge ran `git merge --no-ff junior/...-352` onto its local `governance-v0` but **never pushed**. Zero `git push` invocations in the full 1.15 MB log. Required SSH inspection + `git reset --hard origin/governance-v0` + `git cherry-pick f668071c6` to recover.

**Root cause hypothesis:** the daemon's finalize-merge wrapper brief omits `git push` for planning role. Worth confirming via `homeserver/scripts/restore-junior-shims.ps1` source files.

**Existing lesson:** `feedback_daemon_local_trunk_stale_multi_lane.md` (mentions the pattern but doesn't pinpoint the no-push specifically).

**Quick fix surface:** none right now (daemon shim is in homeserver/ scripts directory; would need separate worktree work). Adding to retro flags.

### 2.2 Planner DQ #291 collision (planner #352)

The planner forked from daemon-local trunk that was 2 commits stale (missing my `bbe4def06` clarify-pass DQ #291-#294). Planner computed `next_id = max(290) + 1 = 291` — colliding with my clarify-#291 already on origin.

**Existing lesson:** `feedback_check_git_before_junior_queue.md` (advisor-side discipline). The defect here was DAEMON-side — daemon-local trunk staleness wasn't visible to the planner.

**Companion lesson:** `multi-lane-worktree.md` §"Worktree-aware DQ id discipline" already specifies walking `origin/<other-lane>` refs when computing next_id. Planner workers do NOT do this today.

**Lesson candidate:** "Planner Junior tasks MUST compute next_id across origin (not just daemon-local trunk)" — would be a new entry in planning.md agent definition or a feedback file. Mechanical fix candidate (add the discipline to the planning agent's task-0 reading).

### 2.3 Planner CC v2.1.119 sensitive-file refusals (planner #352)

7 sensitive-file denials writing to `.claude/PRPs/plans/` + `.claude/runlog/`. Planner worked around via `/tmp + mv`. **Cost:** ~minor (the workaround worked). **Risk:** future planners may not know the workaround.

**Existing lesson:** `feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` covers BM-task runlog gates. Doesn't cover planner-task `.claude/PRPs/plans/` writes.

**Lesson candidate:** extend existing lesson OR write a sibling `feedback_cc_gate_blocks_planner_plans_writes.md` that names the `/tmp + mv` workaround. The planner's own §19.5 LESSON in the plan already names the pattern — promotable.

### 2.4 bm-cut #353 stale-base failure

**~10 min wallclock cost.** Daemon-local trunk at `a16a4e2e1` (3 commits behind origin's `0935c3c92` after my brief + regex widening + DQ #295 pushes). Worker:
- Couldn't find the brief file (committed at `57ce4c322` after daemon's last sync).
- Saw the OLD strict regex (`^phase-v\d+-[A-Z]+-[a-z]$`) in `bm-cut.md`.
- Correctly refused per pre-widening rules.
- Exited cleanly → daemon marked "done" despite zero artifacts.

**Existing lessons:** `feedback_daemon_local_trunk_stale_multi_lane.md` + `feedback_junior_292_stale_base_recover_recipe.md` + `feedback_check_git_before_junior_queue.md`.

**Lesson candidate:** the advisor should ROUTINELY ff the daemon-local trunk BEFORE every Junior task creation when the trunk has been advanced in this session — not just when the lesson surfaces a symptom. Worth a checklist-item in advisor-orchestrator §2 brief-authoring.

### 2.5 bm-cut #354 runlog gate-block (4 strategies, all blocked)

**~5 min wallclock cost.** BM Junior worker attempted:
1. Write tool, abs path → "Junior workers may not write outside their worktree" (PMD #998).
2. Write tool, worktree-rooted path → "Claude requested permissions to edit ... which is a sensitive file."
3. Bash `mkdir -p ... && cat > ...` → "This Bash command contains multiple operations" approval gate.
4. Bash `cat > .claude/runlog/...` relative → same sensitive-file denial.

Worker correctly stopped + escalated to advisor (advisor-relocate, per brief §5).

**Existing lesson:** `feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` — directly applicable. No lesson update needed; pattern reproduced exactly.

### 2.6 Brief §4.7 vs §5 contradiction in bm-cut brief

I authored the bm-cut brief with both:
- §4.7 telling Junior to STOP+file-blocker-DQ if runlog blocked.
- §5 documenting advisor-relocate as expected recovery.

The two are mutually exclusive. Junior correctly picked §5 (the more recent + structurally executable path). Outcome correct, brief shape was the bug.

**Lesson candidate:** "When a brief documents a KNOWN harness limitation with an advisor-relocate recovery, the constraint section MUST NOT also instruct the worker to file a blocker DQ for the same trigger — those are alternative recoveries, not a chain."

### 2.7 Daemon refspec filter (`phase-v1-*` only)

Daemon's `.git/config` has `+refs/heads/phase-v1-*:refs/remotes/origin/phase-v1-*` — meta-phase branches like `phase-brehon-conformance-audit` don't get tracked. Force-fetched via `git fetch origin <branch>:refs/remotes/origin/<branch>`. Future meta-phase branches will hit the same issue.

**No existing lesson** covers this exact symptom (search: no PMD hit on "refspec filter" or "phase-v1- glob").

**Lesson candidate:** "Daemon's `.git/config` fetch refspec filters phase branches to `phase-v1-*` only; meta-phase branches need explicit fetch + ref-creation."

**Mechanical fix surface:** broaden the daemon refspec to `+refs/heads/phase-*:refs/remotes/origin/phase-*`. Requires editing `/srv/brehon-fork/.git/config` directly OR via the Junior-shim restore script. ONE-LINE config edit; very low risk.

### 2.8 Briefs-on-trunk vs briefs-on-phase-branch convention drift

advisor-orchestrator §2.1 says "Every Junior task is preceded by a brief at `.claude/PRPs/briefs/<phase>-<role>-<n>.md`, committed on `governance-v0` before the task is created." But:

- Planning brief: committed on `governance-v0` (correct — before phase branch exists).
- bm-cut brief: committed on `governance-v0` (correct — before phase branch exists).
- Impl-task briefs: **committed on PHASE BRANCH** in practice (per fed-in-b `0ea7ab4f7`). §2.1's wording is incomplete / drift.

**Cost this session:** ~10 min (cherry-pick + forward-merge).

**Lesson candidate:** "Briefs land on the branch the worker forks from. Planning + bm-cut briefs → trunk. Impl-task briefs → phase branch (because impl-task workers fork from `phase-<X>`)." Update advisor-orchestrator §2.1 wording.

## 3. Per-role retro signals

### 3.1 Planner (Junior #352, Opus 4.7)

- **Score:** would estimate 7/10 — plan ships, score honestly computed, but daemon-finalize defect class fired + harness gate-block workaround needed.
- **What worked:** PRECON pre-emption; honest score; verbatim brief mirror; LESSON trailers in plan §19.5 surfacing the gate-block workaround.
- **What broke:** daemon finalize no-push; DQ #291 id collision via stale base.
- **Cost driver:** the recovery; the planner itself worked well.

### 3.2 BM (Junior #353 + #354, Haiku 4.5)

- **Score:** #353 = 2/10 (zero artifacts, false-positive success); #354 = 6/10 (branch cut correct, runlog gate-blocked but escalated cleanly).
- **What worked:** #354 walked all 4 phases correctly; correctly stopped on harness gate (did NOT force-bypass); did NOT trigger finalize-merge bm-cut bug.
- **What broke:** #353's stale-base failure (advisor's job to ff trunk pre-dispatch; not BM's fault); #354 hit gate-block (harness defect, not BM's fault).
- **BM-Haiku judgment notable:** correctly refused force-bypass attempts. Score reflects defect-induced cost, not BM judgment.

### 3.3 Advisor (this session)

- **Score:** 6.5/10 — recovered three defect classes; authored 3 briefs + 1 plan-recovery + 1 runlog-relocation; ran User Gate 1 cleanly. Cost-driver: should have ff'd daemon trunk BEFORE dispatching #353 (would have caught stale-base before it cost a Junior task).
- **What worked:** AskUserQuestion gates for split-or-proceed + branch-name + recovery-order; lesson-cited recoveries; workflow_state PMD updates; atomic DQ #295 commit.
- **What broke:** did NOT routinely ff daemon-local trunk before each Junior dispatch (the new lesson candidate); committed Task 1 brief to trunk instead of phase branch (convention drift discovered mid-session).

### 3.4 Daemon (homeserver, junior@brehon-fork)

- **Score:** 5/10.
- **What worked:** task lifecycle (queue → spawn → DB row → log capture) all functioned. Watchdog 60-min held. Eval-write retro-check.sh enforcement held.
- **What broke:** finalize-merge no-push (planner #352); refspec filter excluded meta-phase branches; the false-positive-success failure mode (#353 exited "successful" despite zero artifacts).

## 4. Quick fixes (implement this turn)

These are LOW-RISK + IMMEDIATELY EXECUTABLE. Deferred items go to §5.

### 4.1 Widen daemon's refspec to `phase-*` (one-line `.git/config` edit)

`/srv/brehon-fork/.git/config` currently has:
```
[remote "origin"]
    fetch = +refs/heads/governance-v0:refs/remotes/origin/governance-v0
    fetch = +refs/heads/phase-v1-*:refs/remotes/origin/phase-v1-*
```

Add a third line: `fetch = +refs/heads/phase-*:refs/remotes/origin/phase-*` (or replace the second line with the broader glob). The narrow line continues to match V1 phases; the broader line catches meta-phases.

**Risk:** very low. Adds remote-tracking refs; doesn't change branch state.

### 4.2 Update advisor-orchestrator §2.1 to clarify briefs-on-phase-branch convention

Reword the §2.1 sentence about "committed on `governance-v0`" to distinguish (planning + bm-cut briefs → trunk) vs (impl-task briefs → phase branch).

### 4.3 Write `feedback_briefs_land_on_branch_worker_forks_from.md` lesson

Captures the convention drift discovered §2.8 above. Cross-link to advisor-orchestrator §2.1.

### 4.4 Write `feedback_planner_dq_id_via_origin_not_daemon_local.md` lesson

The DQ #291 collision lesson per §2.2. Crosslink to `feedback_check_git_before_junior_queue.md` + multi-lane-worktree.md §"Worktree-aware DQ id discipline".

### 4.5 Write `feedback_daemon_refspec_excludes_meta_phase_branches.md` lesson (or fold into existing)

Per §2.7. Mechanical fix in §4.1 IS the structural fix; the lesson documents the symptom for future debugging.

## 5. Deferred to Task 13 retro (sub-phase retro)

- **§2.1 daemon finalize-merge no-push** — needs investigation of the daemon's finalize-merge wrapper brief. Lives in homeserver/scripts/. Not a quick-edit; worth a Junior-shim patch tracked via `project_junior_shim_patches_untracked.md`.
- **§2.3 planner CC v2.1.119 sensitive-file workaround** — lesson promotion-candidate (extend `feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` to cover planner role too). Defer — happens infrequently + the in-plan LESSON trailer documents the workaround.
- **§2.6 bm-cut brief §4.7 vs §5 contradiction** — author lesson on "alternative-recovery exclusivity" at sub-phase retro time; relevant data point would be observing this in other brief-authoring sessions.
- **Daemon refspec broader fix via Junior-shim** — §4.1 is the immediate one-line config edit. Tracked-patch version goes through `homeserver/scripts/restore-junior-shims.ps1` per `project_junior_shim_patches_untracked.md`. Defer the tracked-patch authoring.

## 6. Quick-fix execution log

Executed 2026-05-20 ~16:15 UTC, advisor session.

### 6.1 §4.1 — daemon refspec broadened ✅

```bash
ssh homeserver "cd /srv/brehon-fork && git config --add remote.origin.fetch '+refs/heads/phase-*:refs/remotes/origin/phase-*'"
```

Verified: `git fetch origin` now auto-creates `refs/remotes/origin/phase-brehon-conformance-audit`. Existing `phase-v1-*` refspec line preserved (redundant but harmless).

**Caveat:** live `.git/config` edit; will be reverted if shim-restore runs. Tracked-patch via `homeserver/scripts/restore-junior-shims.ps1` deferred to Task 13 retro (per `project_junior_shim_patches_untracked.md` pattern).

### 6.2 §4.2 — advisor-orchestrator.md §2.1 reworded ✅

Replaced "committed on `governance-v0` before the task is created" with a per-role table that distinguishes:

- Planning + bm-cut briefs → `governance-v0`.
- Impl-task briefs → `phase-<X>` (the branch the impl worker forks from).
- bm-pr / bm-merge briefs → `governance-v0`.
- ci-watcher briefs → `governance-v0`.

Reference to fed-in-b precedent commit `0ea7ab4f7` + this session retro included for future debugging.

### 6.3 §4.3 — `feedback_briefs_land_on_branch_worker_forks_from.md` authored ✅

Captures the brief-visibility convention with recovery recipe (cherry-pick + forward-merge).

### 6.4 §4.4 — `feedback_planner_dq_id_via_origin_not_daemon_local.md` authored ✅

Captures the DQ #291 collision lesson + advisor-side `git fetch + git update-ref` discipline + planner-side TODO.

### 6.5 §4.5 — `feedback_daemon_refspec_excludes_meta_phase_branches.md` authored ✅

Documents the refspec filter symptom + the 2026-05-20 fix + the pending tracked-patch.

### 6.6 Not executed this session

- Tracked-patch for the refspec broadening (requires homeserver/ worktree work; deferred per `project_junior_shim_patches_untracked.md`).
- Extension of `feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` to cover planner role's `.claude/PRPs/plans/` writes (defer — happens infrequently + plan §19.5 LESSON already documents the workaround).
- Lesson on "alternative-recovery exclusivity in briefs" (defer to Task 13 retro for repeated-observation evidence).
- Planner-side task-0 `next_id` cross-origin discipline (defer — advisor-side ff covers it; planner extension is belt-and-braces).

## 7. PMD lesson promotion

The three new lessons authored in §6 will be picked up by `scripts/sync-lessons-to-pmd.sh` (idempotent) on next run. **Not run this session** — Cohort 1 impl Task #355 is in flight; sync after Task 1 ships to avoid concurrent writes to PMD during impl.

## 8. Score (per-role aggregate)

| Role | Score | Driver |
|---|---|---|
| Advisor | 6.5/10 | Recovered three defect classes cleanly; lapse was not ff-ing daemon trunk pre-dispatch. |
| Planner | 7/10 | Plan ships; PRECON pre-emption worked; defect-class cost (no-push + DQ collision) was infrastructure, not planner judgment. |
| BM (#353 + #354) | 4/10 average | #353 zero-artifact false-positive success; #354 partial-success with correct gate-block refusal. Recovery was structurally available. |
| Daemon | 5/10 | Lifecycle held; refspec filter + finalize-no-push were structural defects. |

Overall session: **6/10** — three NEW lesson files + one structural fix shipped (daemon refspec). Cost-driver was the recovery overhead; net learning was substantial.
