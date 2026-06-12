# Session retro — 2026-06-12 — m2-late-2 AB setup + Task 0 brief

**Harness:** claude-code
**Session window:** 2026-06-12 (resumed from compacted prior session)
**Branch at start:** `9725c6ff5` (`governance-v0`)
**Branch at end:** `9baefa91e` (`governance-v0`)
**Files touched:** 23 (per `git diff --stat 9725c6ff5..HEAD`)
**Commits:** 5 authored this session (cdc97fda5, 0fb09f920, 2645da419, 4a9cadd0c, 9baefa91e) — plus prior-session commits in the same 2026-06-12 window

## TL;DR

This session had two goals: (1) resolve the bridge ruma/time E0119 blocker that prevented m2-late-2 from compiling on Linux, and (2) configure the AB/MiniMax trial so the new `/auto-phase` session could start clean. The bridge was fixed (`time = "=0.3.47"` pin + committed `Cargo.lock`). The AB trial was re-armed after a concurrent session conflict. The most load-bearing finding: a concurrent session suspended the AB trial in a memory file, creating state divergence that required an explicit AskUserQuestion to resolve. The top change proposal: make "concurrent session detected, explicit trial-state decision needed" a surface-first ritual entry rather than requiring the user to notice the conflict themselves.

---

## What surprised us

**Advisor:**
- **Concurrent session AB suspension was invisible at session start.** The compacted prior session had armed the trial; a concurrent session suspended it in `workflow_state_m2_late_2.md` while this session was live. The conflict only surfaced when the state file was read and the SUSPENDED flag was spotted. The surface-first ritual detects multi-lane CWD divergence but doesn't detect same-file semantic contradictions across concurrent sessions. 2× recurrence (same class as the 2026-06-07 `.mcp.json` rewrite incident in `reference_retro_check_marker_dir_machine_shared.md`).
- **`git show origin/phase-m2-late-2:...` fails on Windows** with colon-ambiguity in the path separator. Worked around via SSH, but this has now occurred 2× (first in an earlier session). The Windows `git show ref:path` colon trap is a mechanical class — should be a lesson.
- **Mode-B sync merge conflicted on `.claude/PRPs/handovers/m2-late-2-bootstrap.md`.** The phase-branch version had bootstrap content; the governance-v0 merge had later content. Resolved with `--ours` (phase branch wins for `.claude/` per the rule), but the conflict itself was unexpected at merge time — earlier briefs synced cleanly.
- **The bridge ruma/time failure was NOT Linux-only.** Prior session notes and the lesson `feedback_bridge_validates_on_linux_not_windows.md` had the failure mis-characterized as "fails on Windows, compiles on Linux." The actual root cause was a `ruma-common 0.19.0` / `time ^0.3.47` version resolution conflict that failed on BOTH platforms. The `--ours` lesson was already corrected mid-session (commit `0fb09f920`) — catch: the premise-error in the lesson had propagated to the bootstrap file and needed a second fix pass.

**BM/Planning:** Not invoked this session — pure advisor setup work.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add session-start check: if `workflow_state_*.md` shows a trial-state flag (`SUSPENDED`/`RE-ARMED`) that differs from MEMORY.md index, surface as a one-line WARN before any other action (analogous to the surface-first ritual for multi-lane CWD). | Catch concurrent session contradictions at session start, not on first file read. Prevents silent miss. | minor — add 1 grep to the surface-first ritual in `.claude/rules/advisor-orchestrator.md` §1 | 2× (2026-06-07 `.mcp.json` conflict + this session) |
| 2 | Add lesson `feedback_windows_git_show_colon_path.md`: `git show origin/phase-branch:.claude/path` fails on Windows with colon ambiguity; always route through `ssh homeserver "git show ..."` or `git -C /path show ...`. | Prevent future time-wasting from this trap. | minor — 1 new lesson file + PMD write | 2× (2× distinct sessions) |
| 3 | Amend `feedback_bridge_validates_on_linux_not_windows.md` §"Why"  to include: "Always verify the failure mode is host-specific before authoring the lesson — run `cargo-linux.sh` against the same branch before concluding Linux compiles." The current lesson was authored before the Linux run completed, locking in the wrong premise. | Prevents future lessons being authored from incomplete evidence. | minor — 1-line amendment to existing lesson | 1× here; matches broader `feedback_handover_assumptions_need_empirical_verification.md` pattern |
| 4 | Auto-state JSON: add `ab_trial_state: "armed" | "suspended" | "not-applicable"` field (updated by the advisor session that arms/suspends, not by `/auto-phase` itself). This gives the new session a machine-readable signal rather than requiring it to read `workflow_state_*.md` prose. | Makes AB trial state checkable by the skill without PMD search. | medium — auto-state template + skill Phase 0 | 1× new; worth doing for m2-late-2 pilot before the trial arms |
| 5 | Mode-B sync: add `--no-commit` preview step before the merge so `.claude/` conflict paths are visible before resolving with `--ours`. Currently advisor resolves the conflict correctly but without visual confirmation of what was discarded. | Prevent silent loss of governance-v0 updates to `.claude/` files during Mode-B sync. | minor — procedural note in `.claude/refs/multi-lane-mechanics.md` §Mode-B sync | 1× here |

## What to carry forward

- **AskUserQuestion as the explicit tie-breaker for concurrent-session conflicts.** When two sessions have contradictory state about a reversible decision (AB trial armed vs suspended), AskUserQuestion is the right escalation path — not inferring from commit timestamps. This worked cleanly here.
- **`--ours` merge conflict resolution for `.claude/` files during Mode-B sync.** The rule is clear (phase branch wins for `.claude/` files); the `git checkout --ours` + `git add` + `git commit` sequence resolves it deterministically. Document in brief whenever Mode-B sync is part of the handoff.
- **Bridge Cargo.lock now committed** — this was the missing artifact that caused the unpinned resolver to grab `time 0.3.48+`. Future bridge tasks should check for Cargo.lock existence before assuming pinned deps.
- **Lesson-amendment-at-session-end pattern.** When a session corrects a prior lesson's premise, the amendment should propagate to (a) the lesson file, (b) the bootstrap file if it cited the wrong premise, (c) the MEMORY.md index summary. All three were updated this session — keep the 3-step pattern for future premise corrections.
- **Task 0 brief authored on `governance-v0` and Mode-B synced.** For Mode-B phases, the brief commit + Mode-B sync must complete before the session ends — not deferred to the next session. This was done correctly; carry forward the discipline.

---

## Three-signal scoring

Per `feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| AskUserQuestion (AB trial re-arm) | 15 | 3 | low | Clean resolution of concurrent-session conflict; right tool for this class of decision |
| `/update-state` (end of session) | 10 | 2 | none | MEMORY.md and workflow_state file patched correctly; index SUSPENDED line updated |
| `git show origin/... (Windows)` | 0 | 8 | medium | Colon-path trap; 2× recurrence → lesson candidate |
| `ssh homeserver "git show ..."` (workaround) | 0 | 3 | none | Reliable workaround once identified; should be first choice not fallback |
| `cargo-linux.sh` (bridge warmup validation) | ~20 | 12 | high | Initial premise ("Linux-only") wrong; failure reproduced on Linux; corrected mid-session |
| `scripts/check-excluded-lockfiles.sh` (new) | 0 | 0 | none | Authored as lesson action item; not yet invoked in anger |
| Mode-B sync (`git merge origin/governance-v0` on daemon) | 0 | 10 | medium | Unexpected bootstrap conflict; resolved cleanly with `--ours` |
| Task 0 brief authorship (advisor) | 0 | 5 | none | Straightforward brief; 7 probes well-formed; mode-B sync added friction |

## Complexity scores (heavy tasks only)

This session was advisor-only (no impl-task or bm-task subagents). No cargo compilation by workers. Heavy advisor tasks:

| Task | Files | Commits | Runtime (min) | Notes |
|---|---:|---:|---:|---|
| Bridge ruma/time fix | 2 | 1 (`cdc97fda5`) | ~45 (incl. Docker cold pull) | All local; `cargo-linux.sh` cold-pull was ~15 min of that |
| Retro action items execution | 4 | 1 (`2645da419`) | ~20 | Lesson amendments + bootstrap update |
| Task 0 brief + Mode-B sync | 3 | 1 (`9baefa91e`) + daemon merge | ~30 | Mode-B conflict resolution ~10 min |

---

## Decisions to revisit

- **Auto-state JSON `ab_trial_state` field**: is the trial state recoverable from `workflow_state_*.md` prose + `minimax-m27-trial-1.md` via search, or is a first-class JSON field worth the template change? Worth a 5-min clarify before m2-late-2 T3 dispatch.
- **Mode-B vs Mode-A for m2-late-2**: should a lane worktree be cut? The current Mode-B is functional but the merge conflicts suggest Mode-A would have cleaner handoffs. Non-blocking for m2-late-2 at this point.

---

## Auto-phase reliability (`.claude/auto-state/m2-late-2.json` exists — trigger fires)

This session initialized `m2-late-2.json` but did NOT run `/auto-phase` — the file was written as setup for the NEXT session. The 10 categories therefore evaluate the auto-state initialization process, not a running `/auto-phase` cycle.

### 1. Stage-transition correctness

No transitions fired this session — state was initialized at `stage: "impl-cohort-0"` with `--start-from impl-cohort-0`. Gate 1 pre-cleared. The initialization was correct: no prior state existed to race, and the `--start-from` flag documents the explicit override. ✓

### 2. Cadence calibration

No `ScheduleWakeup` calls this session (no running `/auto-phase`). N/A for this session. The next session's cadence will initialize fresh. ✓

### 3. Auto-state integrity

- State file written fresh (no prior file to corrupt)
- `session_id: "300f16d6cd0d"`, `resume_count: 0`
- `last_known_phase_tip: "42431e8f9..."` — ⚠ **stale at write time**: phase tip had advanced to `32cffec41` after Mode-B sync. The field was written before the sync committed. Next session's Phase 0.5 will detect drift and correct it.
- `user_gate_history[0].gate: 1` — pre-cleared correctly

**Finding:** `last_known_phase_tip` was written before Mode-B sync, so it reflects the pre-sync tip `42431e8f9` not the current `32cffec41`. Low severity — Phase 0.5 corrects this — but the initialization sequence should write the tip AFTER all branch mutations complete. ⚠

### 4. User-touchpoint count vs target

1 user interaction (AskUserQuestion for AB trial conflict). Gate 1 was pre-cleared in a prior session; this session had only the re-arm confirmation. Count: 1 (below the 6-gate target, but this was a pure setup session, not a full phase run). No false-positive AskUserQuestions. ✓

### 5. Catch-fire FP/FN rate

No catch-fires this session. The concurrent session AB suspension was surfaced via AskUserQuestion (correct path), not catch-fire. No should-have-been-catch-fire situations observed. ✓

### 6. §G4 classifier accuracy

No `validate-pending` failures this session. N/A. ✓

### 7. L14/L15/L16 fixes still holding

No BM merge, no ci-watcher, no branch delete this session. N/A — will evaluate at m2-late-2 phase close. ✓ (N/A)

### 8. Subagent offload effectiveness

No subagent dispatched under `/auto-phase` this session. The `brehon-state-status` agent was available but not invoked — session started from a compacted prior context with state already synthesized. ✓ (N/A)

### 9. Plan §13 fidelity vs cohort dispatch

No cohorts dispatched. The plan §13 cohort structure (T1+T3 Cohort 1, T2+T4 Cohort 2, T5 serial) was read and verified. No dispatch-time changes needed. ✓

### 10. Resume-cycle pain points

Session initialized fresh (resume_count=0). `--start-from impl-cohort-0` was the explicit override. No Phase 0.5 'continue' friction this session. First c-2 phase; keeping friction for the next session per the lesson's guidance. ✓

### Aggregate auto-phase reliability score

| Category | Status | Recurrence |
|---|---|---|
| 1. Stage-transition correctness | ✓ (N/A — init only) | — |
| 2. Cadence calibration | ✓ (N/A — no running phase) | — |
| 3. Auto-state integrity | ⚠ `last_known_phase_tip` stale at write | 1× new |
| 4. Touchpoint count | ✓ (1 real gate, correct) | — |
| 5. Catch-fire FP/FN | ✓ (no fires) | — |
| 6. §G4 classifier | ✓ (N/A) | — |
| 7. L14/L15/L16 holding | ✓ (N/A) | — |
| 8. Subagent offload | ✓ (N/A) | — |
| 9. Plan §13 fidelity | ✓ (verified, not dispatched) | — |
| 10. Resume cycles | ✓ (init only) | — |

The one ⚠ (category 3) has a concrete fix proposal: write `last_known_phase_tip` AFTER all branch mutations complete (see "What to change" §4 — the `ab_trial_state` field addition is a separate issue; the tip stale write is a sequencing discipline).

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Windows git show colon-path trap → promote to `.claude/lessons/feedback_windows_git_show_colon_path.md`** (2× distinct sessions; blocked reads both times; workaround via SSH is reliable and repeatable)
- [ ] **Surface-first ritual: concurrent-session trial-state contradiction → add 1 grep to `.claude/rules/advisor-orchestrator.md` §1** (2× same class of concurrent-session conflict: `mcp.json` rewrite 2026-06-07 + AB trial suspension this session; both required user escalation to resolve)
- [ ] **Auto-state init: write `last_known_phase_tip` AFTER branch mutations** — update the auto-state initialization sequence comment in the skill body (`~/.claude/commands/auto-phase.md` Phase 0 Step 5) to note: "if Mode-B sync or any phase-branch mutation follows initialization, update `last_known_phase_tip` before writing the file." (1× new; medium priority)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`, `feedback_auto_phase_retro_signals.md`._
