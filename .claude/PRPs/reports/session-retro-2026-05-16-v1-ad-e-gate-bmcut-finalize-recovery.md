# Session retro — 2026-05-16 — v1-ad-e-gate-bmcut-finalize-recovery

**Harness:** claude-code
**Session window:** 2026-05-16T19:30:00Z → 2026-05-16T21:40:00Z (~130 min)
**Branch at start:** `d466d93d7` (`governance-v0`)
**Branch at end:** `cdaa3b6d7` (`governance-v0`)
**Files touched:** 2 (`.claude/decision-queue.json`, `.claude/runlog/v1-AD-e-runlog.md`) + 1 brief created
**Commits:** 5 explicit (`9d3087c1e`, `7d8f84dfc`, `09e0572cc`, `36418c87d`, `cdaa3b6d7`) — all advisor-side, 0 auto

## TL;DR

Drove the v1-AD-e plan-approval gate (gate-1) end-to-end: filed DQ #237/#238, ran the full §15 DoD smoke (4/4 green incl. a 34.7-min e2e baseline), passed watchpoint-specificity, got user approval, resolved both DQs. The bm-cut Junior task (#282) created `phase-v1-AD-e` correctly but its **finalize agent made a spurious content-empty `--no-ff` merge of the phase branch back into daemon-local `governance-v0`** — a distinct failure mode NOT covered by the existing finalize-skip lesson. Recovered cleanly (origin was never contaminated; `git update-ref` reset the daemon trunk). The single highest-leverage proposal: **author a lesson + strengthen the bm-cut brief §6 so Junior's generic finalize agent never merges a bm-cut branch into trunk** — this has now bitten v1-ship-1 AND v1-AD-e.

---

## What surprised us

- **Junior's finalize agent merged a bm-cut branch into trunk.** bm-cut #282's worker did its job (branch at `09e0572cc`), but the generic finalize agent's "commit + merge worktree into base branch" logic treated `phase-v1-AD-e` as a feature branch and ran `git merge --no-ff phase-v1-AD-e` into `governance-v0` (commit `5317fa1fd`). bm-cut's entire purpose is to *create divergence from trunk*, never merge back. The existing `feedback_junior_finalize_skips_when_worker_pre_pushes.md` covers the inverse (pre-push → finalize sees nothing); the merge-into-trunk shape is uncovered. Surprise: high.
- **Origin was never contaminated despite the bad merge.** `5317fa1fd` lived only on the EliteDesk daemon-local `governance-v0`; it never pushed (the CC v2.1.119 `.claude/**` gate / no-push-step blocked it). Laptop + origin stayed pristine at `09e0572cc` the whole time. The blast radius was far smaller than the "merge into trunk" headline implied — the diff was content-empty (all commits already on both branches). Surprise: medium (the gate that blocked the runlog also incidentally contained the merge damage).
- **Concurrent shared-`.git/` activity moved HEAD 3+ times mid-session.** `b114937b8` (DQ #229 move — discarded my first uncommitted #237/#238 append, forcing a re-apply), `50ec13495` (lessons-sync), `d30281170` (skills) all landed from other concurrent CC sessions on the canonical `brehon-fork` checkout while I was mid-write. `multi-lane-worktree.md` explicitly says the canonical checkout should NOT have concurrent DQ writers — reality diverged. Surprise: medium (rule exists; not enforced).
- **Windows `git` multi-arg / `<ref>:<path>` mangling fired twice.** `git rev-parse --short A B` (two refs) and `git show origin/governance-v0:.claude/runlog/...` both got mangled by PowerShell/Git-for-Windows into single bad tokens. Known lesson family (`feedback_windows_bash_python_git_show_tmp_traps.md`) but surprised mid-flow because the single-arg forms worked fine. Surprise: low.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Author `.claude/lessons/feedback_junior_finalize_merges_bm_cut_branch.md` — Junior's generic finalize agent wrongly `--no-ff` merges a bm-cut/branch-creation worktree into trunk; symptom = spurious content-empty merge commit on daemon-local trunk; recovery = `git update-ref refs/heads/<trunk> origin/<trunk>` (working-tree-safe) + push the phase branch. Cross-reference `feedback_junior_finalize_skips_when_worker_pre_pushes.md` (sibling shape). | Next bm-cut session recognises the symptom in 1 read instead of re-deriving the 6-probe damage assessment (~15 min this session). | minor | 2× (v1-AD-e #282 + v1-ship-1 bm-cut per brief §6 note) |
| 2 | Add a §2 Phase-3.5 step to the bm-cut brief template: after `git push -u origin phase-<X>`, the worker writes a `FINALIZE: do-not-merge — bm-cut creates divergence, NOT a feature branch` sentinel to the worktree root (or the brief instructs the finalize-skip). Until the daemon finalize agent is bm-cut-aware, every bm-cut needs the advisor to verify daemon-trunk post-task. | Eliminates the spurious-merge recovery cycle (~20 min/occurrence) on every future bm-cut. | medium | 2× as above |
| 3 | Author `.claude/lessons/feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` — the CC v2.1.119 hardcoded `.claude/**` sensitive-file gate blocks Junior BM runlog + DQ writes even under `bypassPermissions`; the §6 advisor-relocate recovery is now the EXPECTED path until the scoped PreToolUse hook ships. Currently this is only documented inline in bm-cut briefs, not the lesson corpus (so PMD search can't surface it). | bm-cut/bm-pr briefs stop re-explaining the gate from scratch; advisor sessions find the recovery via PMD. | minor | ≥2 (this session + v1-ship-1 + codified in every bm brief §6) |
| 4 | Strengthen `multi-lane-worktree.md` enforcement OR add a PreToolUse guard: the canonical `brehon-fork` checkout must not have concurrent DQ writers. Either (a) a session-start advisory that refuses DQ writes on the canonical checkout when `agent-activity.json` shows another write-mode session, or (b) move v1-AD-e gate-1 DQ writes onto a dedicated worktree even pre-bm-cut. | Removes the discarded-DQ-write re-apply cost (~5 min this session) + the HEAD-move-mid-write friction class. | medium | ≥2 (this session 3+ HEAD-moves + `feedback_concurrent_advisor_session_collision.md` exists) |
| 5 | Add a one-line note to `feedback_windows_bash_python_git_show_tmp_traps.md`: `git rev-parse --short <refA> <refB>` (multi-ref) is also mangled on Windows — use separate single-ref invocations. | Stops the multi-arg rev-parse abort mid-chain (~3 min this session). | trivial | 1× this session + existing lesson (augment, not new) |

## What to carry forward

- **Atomic DQ write-then-commit under concurrent sessions.** After the first discarded #237/#238 append, the re-apply did read-fresh → mutate → verify → commit → push in one shell sequence, minimising the race window. It worked (no second loss). Make this the default for any DQ write on the canonical checkout.
- **`git update-ref` over `git reset --hard` when the checkout is on the target branch.** The daemon main checkout was *on* `governance-v0`; `update-ref` moved the pointer without touching the working tree (no risk to the untracked hook file, no checkout switch that could race the concurrent fed-inbound-a tasks). Cleaner than `reset --hard` for daemon-trunk recovery — carry this discipline.
- **Pre-mutation safety snapshot before any daemon destructive op.** Captured `5317fa1fd` → `/tmp/pre-reset-gov-v0-282.txt` + relied on reflog before the ref move. Cheap insurance; made the "is this recoverable?" question trivially answerable. Routine for daemon-side recovery.
- **Surface multi-step git recovery to the user as an AskUserQuestion before acting.** The bm-cut-recovery and the daemon-reset both went through explicit user-choice gates rather than autonomous repair. Correct call for hard-to-reverse daemon/branch ops — keep this reflex.
- **DoD smoke chained serially on shared `target/`.** Ran §15.1→15.2→15.3→15.4 strictly one-at-a-time (each waits for the prior trailer) because they share the laptop `target/`. No thrash, all 4 green. Carry: never parallelise cargo commands on one `target/`.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/start-brehon v1-AD-e` (full) | 15 | 0 | none | Clean state synthesis; correctly surfaced the multi-lane topology + stale next-id (#236 on fed-inbound-a lane, not #230 as gate-note estimated). |
| `/start-brehon --fast 282` (scheduled wakeup) | 3 | 0 | none | Cheap poll; correctly read #282 finalize state. |
| §15 DoD smoke (4 cargo commands, bg) | 20 | 0 | none | All green; the e2e baseline (89 pass) is the regression guard for the Task-2 extraction. Saved vs running blind into impl. |
| AskUserQuestion ×4 (DQ#237/#238, plan-approval, cargo-order, bm-cut-recovery, next-step) | 8 | 0 | none | Every judgment-heavy fork went to the user cleanly. No false-positive gates. |
| bm-cut Junior #282 (`[role:bm-task]`) | 2 | 25 | high | Created the branch (the load-bearing deliverable) but the finalize-merge bug + gate-blocked runlog cost ~25 min of damage-assessment + recovery + advisor-side runlog authoring. Net-negative this run. |
| bm-cut Junior #281 (cancelled) | 0 | 3 | low | Dispatched then immediately cancelled per user "do not dispatch" (message arrived post-create). Clean cancel, no remote artifact. Minor wasted dispatch. |
| Daemon-trunk recovery (`update-ref` + push) | 18 | 0 | none | Salvaged #282's branch without re-running bm-cut. Saved a full re-cut cycle. |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. No impl-tasks ran this session (gate + bm-cut only). bm-cut #282 recorded for the recurrence trail:

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| bm-cut #282 (branch create + gate-blocked runlog + spurious finalize merge) | 1 (branch ref) + 1 attempted runlog | 1 (the spurious `5317fa1fd`, dropped) | 4 (21:22:43→21:26:32) | n/a (Haiku, sub-minute turns) |

No task exceeded any watchdog threshold (>55min / >40min silence / >8 files). The bm-cut task was *fast*; the cost was entirely in the **post-task recovery**, which the complexity metric doesn't capture — a known blind spot worth noting (the metric measures the worker, not the cleanup the worker's bug forces).

## Decisions to revisit

- **Should v1-AD-e get its lane worktree pre-impl, or stay parked?** User chose "stabilise first" (daemon saturated by fed-inbound-a; concurrent git friction). Resume trigger recorded in the runlog. Revisit when fed-inbound-a #278/#280 reach terminal + #276/#277 re-dispatched-or-abandoned.
- **fed-inbound-a Cohort A health is its own lane session's problem** — but Tasks 1+2 failed + 3+5 stuck ~2h is a 5-way-`[P]`-cohort-saturation signal worth a cross-lane note (does the cohort-budget check in `advisor-orchestrator.md` §4.1 step 5 need a daemon-saturation gate, not just a per-task RAM gate?). Flag for the fed-inbound-a lane retro, not this one.
- **The CC v2.1.119 scoped PreToolUse hook is still deferred** ("until the user is at the laptop terminal" per multiple brief §6 notes). Every bm-cut until then pays the runlog-relocate tax. Worth scheduling the hook install as its own small task.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

**User approved all candidates 2026-05-16 ("Go for it"). All executed this session:**

- [x] #1 finalize-merges-bm-cut-branch → `.claude/lessons/feedback_junior_finalize_merges_bm_cut_branch.md` CREATED (sibling to `feedback_junior_finalize_skips_when_worker_pre_pushes.md`, cited per schema-first gate)
- [x] #2 bm-cut finalize hazard → `.claude/commands/bm/bm-cut.md` "Phase 6 — Finalize hazard" section + refusal-case line + See-also links ADDED
- [x] #3 CC v2.1.119 gate → `.claude/lessons/feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` CREATED (now PMD-searchable via `sync-lessons-to-pmd.sh`)
- [x] #4 canonical-checkout concurrent-DQ-writer guard → `.claude/rules/multi-lane-worktree.md` Hard refusal #6 (atomic read-mutate-commit protocol) ADDED
- [x] #5 multi-arg rev-parse Windows note → `feedback_windows_bash_python_git_show_tmp_traps.md` Trap 4 + symptom row ADDED (augment, no new file)
- [x] PMD eval written: ID 349 ("Session retro: Junior finalize agent merges bm-cut branches into trunk (distinct from finalize-skip)")

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase reliability section
omitted — Step 0.5 trigger did NOT fire (no `/auto-phase` invocation; the
two on-disk auto-state JSONs are prior-session artifacts this session only
observed, never advanced)._
