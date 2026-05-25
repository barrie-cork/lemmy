# Session retro — 2026-05-25 — rt-r3-cohort-2-git-index-cascade

**Harness:** claude-code
**Session window:** 2026-05-25T19:54Z → 2026-05-25T22:30Z (~155 min wall-clock; ~30 min active advisor work, rest was polling)
**Branch at start:** `f42a343f6` (`governance-v0`, Mode B canonical)
**Branch at end:** `f42a343f6` (`governance-v0`, Mode B canonical — no advisor commits this session)
**Files touched:** 0 (advisor-side; no commits)
**Commits:** 0

## TL;DR

Cohort-2 of v1-RT-r3 (Tasks 1/2/3 dispatched as Juniors #467/#468/#469 [P] parallel) **cascaded into a `.git/index.lock` contention deadlock** under shared-`.git` parallel writes on the EliteDesk daemon. All three workers wrote code but none could commit — #467 hit API retry loop after lock contention, #468 hit DNS-resolution timeout (779s) on the push retry, #469 ended with six git processes in **D state (uninterruptible disk I/O)** competing with a parallel rustc compile. User chose option (b) cancel-and-recover; cancel reaped the worktrees instantly (both filesystem and `.git/worktrees/` admin metadata), forfeiting all three workers' uncommitted code. Re-dispatched Task 1 only as #470 (serial). The dominant finding: **cohort `[P]` markers assume disjoint *target files*, but the parallel workers also share `.git/index.lock` — a hidden non-disjoint resource the YAML overlap check cannot detect.** Headline change proposal: degrade `[P]` cohorts of size ≥3 to serial under Mode B until daemon supports per-worker `.git` clones, OR change cancel-handler to preserve worktrees (recovery option).

---

## What surprised us

- **Three parallel impl-tasks sharing one `.git/` deadlocked on `index.lock`.** The cohort `[P]` markers in plan §13 + the §4.1 YAML overlap check (intersect `creates:` + `modifies:` arrays) confirmed all three tasks edited disjoint Rust files — and they did. What the overlap check did NOT catch was the **shared `.git/index.lock`** that every `git add` / `git commit` must acquire. With three Sonnet workers running rustc + git simultaneously on a single daemon-side `.git/`, the lock contention cascaded into D-state cluster (uninterruptible I/O wait). The failure was structural: every `[P]` cohort of size ≥2 has this hazard, not just disjoint-file overlaps.

- **Cancel reaped worktrees instantly with zero artifact preservation.** When the user chose option (b) "cancel and recover code manually," I assumed the daemon's cancel handler would leave the worktree directories in `/srv/brehon-fork/.junior/worktrees/job-46*` for SSH inspection. Reality: `ls /srv/brehon-fork/.junior/worktrees/` returned empty within seconds, and `/srv/brehon-fork/.git/worktrees/` (admin metadata) was *also gone* (`No such file or directory`). The three workers' uncommitted code — confirmed by the log inspection to exist on the worktree HEADs — was forfeit. No "soft cancel" preserves state.

- **Task 0 baseline (single dispatch) was clean; Tasks 1/2/3 parallel was a cascade.** The same daemon, same network, same OS — only the parallelism changed. Task 0 (Junior #466) ran without lock contention and finalized cleanly. The defect class is not "Junior worker reliability" — it is "Junior worker × parallel `.git` contention." Mode B is not the cause either (canonical session never wrote DQ from the laptop; all writes were on the daemon).

- **Subagent log triage worked first-try with zero re-derivation.** Three Explore subagents (one per task log file, each spilling out 60-125 KB of JSONL) returned actionable summaries with verbatim last-20-lines + root-cause categorisation in parallel. The synthesis for #468 spotted "DNS-resolve timeout 779s" + "git commit output empty (likely retro-check hook blocking)"; #469 spotted "D-state cluster with rustc competing"; #467 spotted "API retry loop attempt 8/10 after Python lock-removal trick." Three subagents, three correct diagnoses, no follow-up reads required. Saved easily 20+ min of inline log scrolling.

- **The DQ #338 / falsifiable-hypothesis lesson nearly bit again.** When the user said "workers never merge," my first instinct was to suspect daemon finalize-merge code or a worktree-bootstrap defect. The lesson `feedback_falsifiable_hypothesis_before_structural_fix.md` correctly anchored me to pull the actual logs *first* and read what the workers were actually doing. The real cause was visible in the tail logs in ~3 minutes once I stopped triaging at the daemon level and looked at worker-internal state.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Add a `[P]` cohort-size cap of 2 to `advisor-orchestrator.md` §4.1**, OR add a "shared-`.git/index.lock` hazard" rule that auto-degrades any `[P]` cohort of size ≥3 to serial on the daemon. Codify in `.claude/rules/advisor-orchestrator.md` §4.1 step 5 (the budget check), adding a new rule: "On shared-`.git` daemons, cohort size > 2 auto-degrades to serial regardless of file-disjointness — the `.git/index.lock` is not in any task's FILES YAML and the overlap check cannot detect it." | Eliminates this entire failure class. Cost: ~50% wall-clock loss on `[P]` cohorts of 3+ tasks but only when running on a single-`.git` daemon. Under Mode A (lane worktrees) this rule may not apply. | medium (rule edit + retro evidence) | 1× this session, 0× prior — but failure mode is so structural and the cost so high (3 workers' work forfeited) that recurrence threshold met by severity, not count |
| 2 | **Change Junior daemon cancel-handler to preserve worktrees** (or add a `cancel --preserve-worktree` flag). Track as upstream feature request in `homeserver/scripts/restore-junior-server-patches.sh` or as a new issue in the Junior MCP repo. The retro carry-forward: never cancel mid-work-with-uncommitted-code without first SSH-copying the workspace state out. | Recovery option for next time. Cost: nontrivial daemon-side change; meanwhile the user-side discipline (SSH-copy-first) is free. | major (upstream daemon work) | 1× this session, 0× prior |
| 3 | **Add a pre-cancel SSH inventory step to advisor catch-fire procedures.** Update `.claude/rules/advisor-orchestrator.md` §5.6 "Catch-fire procedures" with a new row: "Before cancelling a Junior task whose worker logs show uncommitted code, SSH to `/srv/brehon-fork/.junior/worktrees/job-<id>` and `tar c .` the worktree contents to a recovery file *first*. The daemon's cancel handler reaps both worktree FS state and `.git/worktrees/<name>/` admin metadata immediately." | Prevents code-forfeiture on the next contention cascade. Cost: 1-2 min per cancelled task; lossless. | minor (rule edit) | 1× this session, 0× prior — but cheap to add and structural |
| 4 | **Promote the shared-`.git/index.lock` finding as a lesson at `.claude/lessons/feedback_cohort_shared_git_index_contention.md`.** Cite this retro + the option-a/b cohort dispatch rule + the daemon worktree model. Index in MEMORY.md under "Junior daemon + workers". | Cross-harness lesson; next session sees it via memory-injection. Loadable by both pi and Claude Code sessions. | minor (one lesson file) | 1× this session — but ≥ severity threshold |

## What to carry forward

- **Subagent log triage for any task whose `task_logs` output exceeds the inline-read cap.** Three parallel Explore subagents on the three log files, each given a structured prompt ("(1) last action (2) stuck/working/finished (3) errors") and asked for verbatim last-20-lines, returned three correct root-cause categorisations in parallel with zero follow-up. This is now the canonical pattern for triaging multi-task contention.
- **Falsifiable-hypothesis discipline saved time again.** When the user surfaces "workers never merge," resist the urge to suspect upstream code paths (daemon, finalize-merge) and pull the worker-internal logs first. The actual defect is almost always visible in the worker's own tail log.
- **Mode B dispatch model held cleanly even under cascade.** The cascade was inside the daemon; the canonical brehon-fork session on `governance-v0` was uninvolved and lost no state. Mode B's separation between canonical-trunk session and daemon-resident phase work is structurally sound.
- **Re-dispatch a forfeited task to its original brief without re-authoring.** Briefs are durable on the phase branch (`737e4dbca` / `fc204297` / `7fd097fb`); the task IDs change but the brief paths do not. Just re-queue with the same `description` string.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| 3× parallel Explore subagent log triage (#467/#468/#469) | 25 | 0 | medium | Three subagents in parallel returned correct root-cause categorisations + verbatim quotes. Diagnosed three different failure modes simultaneously. Canonical pattern for log triage. |
| `mcp__junior-brehon__cancel_task` × 3 | 2 | 60 | high | Cancelling reaped all three worktrees + admin metadata instantly. The 60 min "wasted" reflects the forfeited code; the 2 min "saved" reflects stopping the cascade. Net catastrophic for this session; mitigation in §3 #2/#3. |
| `mcp__junior-brehon__create_task` (cohort-2 parallel) | — | 150+ | high | The three Juniors ran ~3+ hours, produced no committed code, no recoverable artifact. 100% wall-clock loss on cohort-2. |
| `mcp__junior-brehon__create_task` (re-dispatch #470 serial) | — | — | none | Queued at retro-end; not yet executed. |
| `mcp__junior-brehon__task_logs` × 3 | 0 | 0 | low | Returned correctly but each spilled context (60-125 KB JSONL); routed to subagents. |
| `mcp__junior-brehon__daemon_status` + show_task × 3 | 5 | 0 | none | Confirmed daemon alive + 3 active jobs. Routine. |
| `ScheduleWakeup`-style polling cadence (manual via user "poll" prompts) | — | 15 | low | ~5 polls × ~3 min each on tasks that were stuck and not going to transition. Could have been replaced with one log-tail pull at ~30 min mark. |
| Falsifiable-hypothesis lesson (consulted, not invoked) | 10 | 0 | low | Anchored "workers never merge" to "pull actual logs" instead of suspecting daemon code. |
| `memory_write_eval` × 2 (Stop-hook compliance) | 0 | 3 | low | Required for hook compliance during polling session. The 60-min retro-window forced a second write. |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. **All three cohort-2 tasks are forfeited / no commits; complexity is N/A in the canonical sense, but the failure-mode complexity is worth recording.**

| Task | Files | Commits | Runtime (min) | Max log silence (min) | Notes |
|---|---:|---:|---:|---:|---|
| #467 (Task 1 — participation cron) | 3 written, 0 committed | 0 | ~140 (cancelled) | likely >30 during API retry loop | Stuck at git index.lock + API retry attempt 8/10 |
| #468 (Task 2 — vote-outcome + evidence emit) | 1 written + DQ fragment, 0 committed | 0 | ~140 (cancelled) | likely >40 during DNS timeout | Stuck at DNS timeout 779s + retro-check hook blocking commit |
| #469 (Task 3 — flag-bad-faith endpoint) | 3 written, 0 committed | 0 | ~140 (cancelled) | likely >30 during D-state cluster | Stuck at filesystem I/O D-state + parallel rustc compete |

**Flagged carry-forward signals for next plan:** all three would have crossed the >55min / >40min log-silence thresholds even without the lock cascade; the cascade only ensured they crossed them all simultaneously. Watch for shared-resource contention in any cohort of ≥3.

## Decisions to revisit

- **Should `[P]` markers on plan §13 explicitly enumerate shared resources beyond files?** E.g. `requires_exclusive: [".git/index"]`. Worth a clarify pass when authoring the next multi-task plan.
- **Should `feedback_cohort_validation_dependency_check.md` (the `requires:` field check) extend to "ambient shared resources"?** Currently only checks task-to-task `requires:` arrays. The `.git/index.lock` is ambient (every task uses it implicitly).
- **Is there a way to make the daemon's worktree-cancel preserve uncommitted state by default?** Worth a daemon-feature-request issue. Even a simple "`tar c worktree/ > /tmp/job-<id>-recovery-<ts>.tar`" on cancel would have saved this session's three workers' work.

---

## Auto-phase reliability

### 1. Stage-transition correctness

State at session start: `impl-cohort-2-running` (per auto-state JSON). Cohort-2 was queued by prior session at 2026-05-25T19:10:00Z. **Cohort barrier held correctly** — the advisor never advanced to bm-pr or next cohort while #467/468/469 were running. After cancel, the cohort is in a half-state (all three members `result: null`, but no longer running); the auto-state JSON has not been updated to reflect cancel + re-dispatch #470. **Carry-forward: update auto-state JSON to reflect the cancel + re-queue, OR archive and re-init the state for cohort-2.**

### 2. Cadence calibration

User-driven "poll" prompts at irregular intervals. No `ScheduleWakeup` used in this session. The 5+ polls on tasks that were stuck-not-progressing were over-eager — could have been one tail-log pull at ~30 min instead. **Symptom of failure: user "poll" cadence not matched to actual state-change cadence. Five polls produced zero transitions; the sixth pull (logs) found the cascade.**

### 3. Auto-state integrity

`resume_count: 1` (acceptable, within ≤3 target). `last_known_phase_tip: 42dd2e073` — accurate (`git log -1 origin/phase-v1-RT-r3` confirms). `session_id: 5ca8b99d78e5` (current); previous: `a6982e7a875b`. **One drift: cohort-2's `members[].junior_id` still names #467/468/469 but those are now cancelled. The JSON is out-of-sync with reality after the cancel. ⚠**

### 4. User-touchpoint count vs target

This session: 1 mandatory gate decision (option a/b on the cascade — counted as a judgment-heavy DQ-equivalent, not a planned gate). User answered "b" (cancel) then "a" (serial re-dispatch). Plus 5 user "poll" prompts (not gate-class, but interrupting). **Touchpoint count for this session alone: ~7, all reactive to the cascade. The phase as a whole is still well under 8.**

### 5. Catch-fire FP / FN rate

**One catch-fire fired correctly** — the cohort-2 cascade was classified as non-allowlist (parallel `.git` contention is not in the §G4 classifier table) and surfaced to user with the four-bucket option. Classification was correct: this is not a clippy / E0432 / E0277 pattern; it's a structural cohort-dispatch defect. **FP rate: 0. FN rate: 0.** The catch-fire itself was the correct action. ✓

### 6. §G4 classifier accuracy

No `validate-pending` entries fired this session (cohort-2 never reached push, let alone workspace-check). N/A.

### 7. L14 / L15 / L16 fixes still holding

N/A this session — no bm-merge, no PR work, no advisor-side gate-only checks (gate decision was inline AskUserQuestion-equivalent through chat). ✓ (by absence — no regression vector tested).

### 8. Subagent offload effectiveness

**Excellent — first-read usable, net-positive context conservation.** Three parallel Explore subagents on the three log files returned actionable categorisations with verbatim quotes. Estimated saved: 20-25 min inline log-scrolling time. Estimated parent-context saved: 60-125 KB × 3 = ~280 KB of JSONL kept out of main context. Token spend: not measured but each subagent's prompt was <500 tokens, response ~3000 tokens. Net positive by an order of magnitude. ✓

### 9. Plan §13 fidelity vs cohort dispatch

**The defect is exactly here.** Plan §13's `[P]` marker for Tasks 1/2/3 + the §4.1 YAML overlap check (intersect `creates:` + `modifies:` arrays) PASSED — all three tasks edit genuinely disjoint files. The cohort dispatched in parallel per the rule. **But the rule does not check ambient shared resources like `.git/index.lock`**, and the daemon's single-`.git` worktree topology made every parallel `git add` / `git commit` a contention point. **⚠ — defect in rule, not planner. Promote per §3 #1.**

### 10. Resume-cycle pain points

`resume_count: 1` at session start (incremented on prior session boundary). This session did not increment resume_count. The auto-state JSON's `last_action` ("queued cohort-2 (Juniors #467, #468, #469) — Tasks 1+2+3 [P] parallel ...") was sufficient to reconstruct state on resume. ✓

### Aggregate auto-phase reliability score

| Category | Status | Recurrence |
|---|---|---|
| 1. Stage-transition correctness | ⚠ | auto-state JSON not updated post-cancel — 1× this phase, 0× prior |
| 2. Cadence calibration | ⚠ | 5 over-eager "poll" prompts on stuck tasks — 1× this session |
| 3. Auto-state integrity | ⚠ | JSON drift post-cancel — 1× this session |
| 4. Touchpoint count | ✓ | reactive only, total within target |
| 5. Catch-fire FP/FN | ✓ | one correct catch-fire fired |
| 6. §G4 classifier | N/A | no validate-pending this session |
| 7. L14/L15/L16 holding | ✓ | by absence (no regression vector tested) |
| 8. Subagent offload | ✓ | three parallel Explore subagents, first-read usable |
| 9. Plan §13 fidelity | ⚠ | rule defect — shared-`.git/index.lock` not checked; promote per §3 #1 |
| 10. Resume cycles | ✓ | resume_count 1, within target |

Trend across phases: First multi-cohort parallel `[P]` cascade observed. Cohorts in prior sub-phases (v1-RT-r2, v1-deps-r1) were single-task or smaller-`[P]` and did not trigger the lock contention. Promote per §3 #1 + #4.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Add `[P]` cohort-size cap of 2 to advisor-orchestrator.md §4.1** (per §"What to change" #1): update `.claude/rules/advisor-orchestrator.md` §4.1 step 5 budget check with explicit shared-`.git/index.lock` hazard rule
- [ ] **New cross-harness lesson**: promote to `.claude/lessons/feedback_cohort_shared_git_index_contention.md` (per §"What to change" #4); index in MEMORY.md under "Junior daemon + workers"
- [ ] **Add pre-cancel SSH inventory** to advisor-orchestrator.md §5.6 "Catch-fire procedures" (per §"What to change" #3)
- [ ] **Daemon feature request**: `cancel --preserve-worktree` flag or default-preserve on cancel (per §"What to change" #2) — track upstream
- [ ] **PMD eval write** to capture the cohort-cascade pattern + 3-parallel-subagent-triage success — already authorised by `PROJECT_MEMORY_DB` exported

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`, `feedback_auto_phase_retro_signals.md`._
