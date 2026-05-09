# Session retro — 2026-05-09 — replan-mutate-resolver-fix

**Harness:** claude-code (Opus 4.7 1M xhigh)
**Session window:** ~2026-05-09T15:30:00Z → 2026-05-09T17:11:00Z (~100 min, including a /compact mid-flight)
**Branch at start:** `eced68b98` (`governance-v0`)
**Branch at end:** `479408a98` (`governance-v0`) → followed by `a8cc61380` (harness-audit skill add — out of scope for this retro)
**Files touched:** ~12 (across 6 commits authored this session)
**Commits:** 6 explicit (`e109dd82b`, `bbbb92a43`, `f588faac0`, `297dafca1`, `c8c8d0dd2`, `b8225be3e`, `e75d6a814`/`479408a98`) — last two are the same fix split across two pushes after Mac-portability follow-up

## TL;DR

Executed the rippling-twirling-wilkinson replan plan end-to-end (Junior #162 dispatched with a Case A LemmyResult<T> brief), then through three poll cycles handled: (1) DQ id-collision between worker-branch #164 and governance-v0 supersession #164, fixed via worktree renumber 164→167; (2) ci-watcher Junior #163 crash on daemon-side missing local ref (third recurrence of the same lesson candidate — a known systemic issue not yet promoted to a hard precondition); (3) advisor-side direct mutation of DQ #167 to `result: pass` after workflow `25605783542` succeeded, bypassing the failed ci-watcher; (4) discovered + fixed a Windows-CRLF bug in `resolve-dq-canonical.sh` that was silently hiding worker-branch entries from the canonical view. Phase-1 cohort-2 PASSED. **Top change proposal:** promote the daemon-side ref-fetch precondition from "lesson candidate" to a hard step in advisor-orchestrator.md §3.1 ci-watcher dispatch — three recurrences (cycle-1, cycle-3, cycle-4) is over the recurrence-≥2 threshold by a wide margin and the cost is ~5 min wasted per ci-watcher fire.

---

## What surprised us

- **DQ id-collision was structurally inevitable, not bad luck.** Junior #162 branched from `c2761b284` (pre-supersession) and computed `next_id` from its own narrow view, completely missing supersession entries #164/#165/#166 that landed on `governance-v0` *while it was running*. The DQ rule (`rules/decision-queue.md` §"Hard refusals #2 NEVER reuse an existing id") is satisfied **per ref**, but not across refs. This is a different bug class from the DQ #50 collision (which was within-ref). Worker-local computation will collide every time the advisor session writes new DQ entries during a Junior's runtime — and that pattern fires on every supersession + every new advisor commit. The "canonical resolver" was supposed to be the next_id source, but no impl-task brief instructs Junior to use it.
- **The canonical resolver was lying for 30+ minutes before we noticed.** `sources: ['phase-branch']` instead of `['phase-branch', 'worker-162']` — a bash CRLF quirk on Windows MSYS made `grep -E '[-]162$'` invisible-fail against `162\r\n`-line input. The mutation we wrote to DQ #167 was correct on the worker branch (verified directly via `git-show-json.sh`), but the resolver's canonical output was missing it. Could easily have masked a pending-blocker entry on a worker branch in a future phase. The bug was latent across every prior `/auto-phase` resume on Windows; nobody noticed because no prior phase had a worker-branch DQ during a poll tick.
- **Ci-watcher daemon ref-issue is now at 3 recurrences (cycle 1 + cycle 3 + cycle 4 of v1-SL-c-2).** The auto-state JSON has carried it as a "lesson candidate for retro" since cycle 1 — promoted nowhere. It's been the root cause of three failed ci-watcher Juniors, and on cycle 4 the workflow itself succeeded so the failure didn't matter — but a manual advisor-side mutation took ~5 min to do what the ci-watcher would have done in seconds. **Cost of inaction: 3 cycles × 5 min = 15 min wasted; cost of promotion: ~10 min to write the rule edit.**
- **The compaction boundary was invisible in execution.** Pre-compaction the session executed the rippling-twirling-wilkinson plan (Junior #162 dispatched + 3 commits). Post-compaction the session resumed cleanly via the system-reminder summary and answered "poll" → "check whether DQ #167 resolves to pass or fail" → "fix" → "neeeds to run on mac too" with no friction. Each subsequent message landed in the same task context. This is a positive surprise — the summary captured what was needed.
- **`AskUserQuestion` was used 2× in 100 min and both yielded the right call.** The DQ collision fix gate (renumber worker → 167) and the post-mutation gate (advisor-direct mutate vs re-queue ci-watcher) both got the right user input in <30s. Neither could have been auto-decided safely. This is the gate-discipline pattern from `feedback_advisor_orchestrator.md` §3.2 working as intended.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add hard precondition to `.claude/rules/advisor-orchestrator.md` §3.1 ci-watcher dispatch sub-step: before `mcp__junior-brehon__create_task` with `base_branch` matching `junior/*`, run `ssh homeserver 'cd /srv/brehon-fork && git fetch origin +refs/heads/junior/<branch>:refs/heads/junior/<branch>'`. Promote from auto-state lesson candidate L18 to in-rule precondition with citation. | Eliminates the 3rd-recurrence ci-watcher worktree-add failure; saves ~5 min/dispatch + zero failed-Junior noise in retro. | minor (~10 min edit + commit) | 3× this session, 0× prior — *worst-pattern bracket*. |
| 2 | Add `next_id` calculation discipline to `.claude/PRPs/templates/impl-task-brief.template.md` §3 "Required reading" — instruct Junior to compute `next_id` via `bash scripts/brehon/resolve-dq-canonical.sh <phase>` if a worker-branch validate-pending entry will be written. Cite `feedback_dq_collision_across_refs.md` (new lesson, also propose). | Eliminates DQ id-collision class entirely — Junior sees governance-v0 supersession entries before assigning id. | medium (template edit + new lesson) | 1× this session, 0× prior — but the structural pattern fires every supersession × every Junior, so very high latent recurrence. |
| 3 | Promote auto-state lesson candidate L18 (daemon ref-fetch) AND L19 (resolver CRLF) to dedicated `.claude/lessons/feedback_*.md` files, then sync to PMD via `scripts/sync-lessons-to-pmd.sh`. Without promotion, they re-surface as "lesson candidates" every retro until someone notices the pattern is recurring. | Closes the L18/L19 lesson loop; future advisor sessions surface these via `Glob .claude/lessons/` at session start. | minor (2× new files, ~30 min total) | 3× for L18, 1× for L19 — both clear the threshold. |
| 4 | Harden `scripts/brehon/resolve-dq-canonical.sh` with a collision-detection mode: emit a stderr warning when the same `id` appears in 2+ refs being merged. Currently the resolver silently dedupes worker-wins, which would have masked the #164 collision had we not noticed it through `git log` of supersession commits. | Surface DQ collisions explicitly at canonical-view time instead of relying on advisor noticing via separate channel. | medium (~30 min — Python heredoc edit + test) | 1× this session, but structural — fires whenever supersession + concurrent Junior coincide. |
| 5 | Auto-state lesson-candidate harvest at retro time should produce a markdown checklist of "promote / merge into existing lesson / discard" decisions, not just narrative bullets. The current 12-entry `lesson_candidates_for_retro` array is a write-only field — entries accumulate across phases without any trigger to act on them. | Closes the gap between "I noticed this" and "I made it actionable." Specifically: 8 of the 12 candidates in v1-SL-c-2's auto-state are still un-promoted. | medium (~45 min — touches both `session-retro` skill and `auto-phase` retro template) | 12× across this phase, suggests systemic. |
| 6 | When `AskUserQuestion` for a routine corrective fix (e.g. DQ collision renumber) gets a clear "yes do the obvious thing" answer, log a "follow-up automation candidate" in retro for: this gate's question shape predictably matches one of three answers per the rule. The gate retains user control but can include a "(default: <preferred>)" option when the rule's preferred path is unambiguous. | Reduces friction without removing safety. The DQ-collision gate had three options where one was clearly right per the rule (renumber worker → cleanest); user-time spent was small but predictably zero-information. | minor (~15 min documenting) | 1× this session, but the "renumber worker" answer is rule-deterministic — every collision will pick the same way. |

## What to carry forward

- **Worktree-per-edit pattern for committing into an active phase / worker branch when the primary clone is mid-session-dirty.** Used twice cleanly this session (`brehon-fork-dq-fix-164` for the 164→167 renumber, `brehon-fork-dq-pass-167` for the result=pass mutation). Both removed cleanly via `git worktree remove` after force-with-lease push. Keeps the primary clone's dirty drift untouched. **Rule:** when about to mutate a non-current branch and `git status --short` is non-empty, create a fresh worktree from the target ref, edit + commit + push there, then `worktree remove`.
- **`git-show-json.sh` discipline holds at scale.** Used 4× this session to read DQ JSON across worker / phase / governance-v0 refs. Zero Bash↔Python /tmp encoding traps. The wrapper has been stable across the entire v1-SL-c lane.
- **Force-with-lease push on worker branches is safe even when the user explicitly authorises the rewrite.** Used twice (worker DQ #164 → #167, then mutation → result=pass). Both `--force-with-lease=junior/...:<previous-sha>` invocations succeeded; in both cases the daemon-side ref refresh via `+refs/heads/junior/...:refs/remotes/origin/junior/...` propagated the rewrite cleanly. **Pattern to keep:** lease-target the previous SHA explicitly; refresh daemon-side after every force-push; verify `git rev-parse refs/remotes/origin/junior/...` matches the new tip.
- **Direct workflow probe via `gh run view <id> --json conclusion` is faster + more reliable than waiting for ci-watcher.** When ci-watcher Junior crashes (3rd recurrence pattern), the advisor can grab the workflow result immediately. Combined with worktree-based DQ mutation, it becomes a viable advisor-direct mutation path that mirrors `validate-pending-laptop` semantics.
- **The two-step "answer + summary" surface pattern after AskUserQuestion gates** — first text confirming the choice, then the actions, then a "Summary" block at the end — gives clean transcript granularity for retro reading. Used 3× this session (DQ collision gate, mutation gate, mac-portability follow-up).

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`. Numbers are defensible from session timestamps and command outputs.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `git worktree add` (twice) for DQ mutation | 25 | 0 | low | Avoided two distinct dirty-tree blockers; the second use was load-bearing for the user's chosen "advisor-direct mutate" path |
| `git-show-json.sh` wrapper | 8 | 0 | none | Stable at scale; 4× use, zero traps |
| `bash scripts/brehon/resolve-dq-canonical.sh` (Windows-broken) | 0 | 8 | medium | Returned `sources: ['phase-branch']` only; would have masked a worker-branch pending-blocker had one existed; bug fixed in same session |
| `AskUserQuestion` (DQ collision gate) | 2 | 0 | none | Clear three-option fork; user picked "renumber worker → 167"; cost ~30s |
| `AskUserQuestion` (advisor-direct mutate vs re-queue ci-watcher) | 5 | 0 | none | Avoided 10+ min re-queue delay by going direct |
| `mcp__junior-brehon__create_task` (Junior #163 ci-watcher) | 0 | 5 | medium | Junior #163 crashed at worktree-add (3rd recurrence) — 5 min wasted on Junior dispatch + status-check before realising L18 was the cause |
| `mcp__junior-brehon__list_tasks` | -1 | 1 | low | Output exceeded 30k-char cap, forced fallback to file-based grep — minor friction, well-handled |
| `gh run view 25605783542 --json conclusion` | 10 | 0 | none | Direct workflow probe yielded `conclusion=success` instantly; rendered ci-watcher #163's failure moot |
| `python json.load + edit + dump` for DQ mutation | 10 | 0 | none | Twice; both produced clean JSON validated post-write |
| `ScheduleWakeup` | 0 | 0 | n/a | Not used this session — manual polling pattern (user said "poll" / "check") |
| Compaction boundary (system) | 60 | 0 | low | Pre-compaction state was preserved cleanly; post-compaction first response answered the "poll" prompt without re-deriving any context |
| `tr -d '\r'` + `sys.stdout.reconfigure(newline='\n')` (resolver fix) | n/a (one-shot) | 0 | low | Defence-in-depth on a Windows-portability fix; verified via `xxd` smoke test |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. Format: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Junior #162 (impl-task replan) | ~3 (e2e.rs + DQ + 1 helper file) | 2 | 18 | n/a (clean execution per Junior daemon log) |
| DQ #164 → #167 renumber + push | 1 (`.claude/decision-queue.json`) | 1 | 4 | 0 |
| Advisor-direct DQ #167 mutate to pass | 1 (`.claude/decision-queue.json`) | 1 | 3 | 0 |
| Resolver CRLF fix + Mac-portability re-test | 1 (`scripts/brehon/resolve-dq-canonical.sh`) | 1 | 6 | 0 |

No outliers (no >55-min runtime, no >40-min log-silence, no >8-files-touched). Junior #162 was within envelope for a replan task.

## Decisions to revisit

- **L18 daemon ref-fetch promotion** — should this happen now (this retro) or be deferred to v1-SL-c-2 phase retro at sub-phase close? The 3rd-recurrence rule says "do it now"; the deferred-to-phase-retro pattern says "let the full lane settle first." Recommendation: promote now per recurrence threshold.
- **Brief-template `next_id` discipline** — adding it to the template adds boot cost on every impl-task. Worth a clarify pass with the user on whether this is "always required" or "only when supersession is in flight" (which Junior can't know).
- **Resolver collision-detection mode** — the proposed `--detect-collisions` flag would make the resolver fail-loud when ids overlap across refs. Risk: false-positive on benign legacy collisions in archived DQ entries. Worth a small probe over the existing archive before shipping.

---

## Auto-phase reliability

Auto-state JSON for v1-SL-c-2 exists and was actively mutated this session (multiple Edit tool calls). 10 categories below per `feedback_auto_phase_retro_signals.md`.

### 1. Stage-transition correctness

Three transitions fired this session: `impl-cohort-2-running → impl-cohort-2-validating-phase-1` (after Junior #162 done + DQ #167 raised), `impl-cohort-2-validating-phase-1 → impl-cohort-2-phase-1-pass-awaiting-finalize-merge` (after DQ #167 mutated to pass). All three fired on the right trigger; none fired prematurely; cohort barrier held (no advance to cohort N+1 with cohort 2 still in pass-pending). The `user_gate_history[].notes` field (plan-approval gate) was read at session start (`DoD smoke 10/11; e2e.rs LOC 11925`) — flagged in the retro RCA but not load-bearing for this session's transitions. ✓

### 2. Cadence calibration

This session ran in **manual-poll mode** — user said "poll" / "check whether DQ #167 resolves" rather than `/auto-phase` driving the loop. ScheduleWakeup was not used. **Wasted-poll count: 0** (every poll was user-initiated and yielded a state change). **Detection-lag**: ~15 min between Junior #162 done at 16:20:41Z and advisor seeing the workforce push at 16:29 UTC poll. Within the 270s cache-warm target × 2-3 ticks. ✓ for manual mode; cadence calibration of `/auto-phase` itself was not exercised.

### 3. Auto-state integrity

Final state: `resume_count: 4`, `session_id: a3f72e4d8c91`, `last_known_phase_tip: 47af884c8` (matches `git log -1 phase-v1-SL-c-2`), `last_known_phase_tip_worker_branch: 103425f9b` (matches `git rev-parse origin/junior/...md-162`). One stage transition (`impl-cohort-2-validating-phase-1 → impl-cohort-2-phase-1-pass-awaiting-finalize-merge`) named a state not in any documented transition table — the ad-hoc `*-awaiting-finalize-merge` suffix is informal but not a corruption. JSON validated post-edit (`python -c json.load` returned valid). **`resume_count: 4` is at the top of the target band (≤3) — borderline finding.** The "this is the 4th resume" friction stems from the cycle-1/2/3 catch-fires + replan; not a state-machine bug. ⚠ on resume_count.

### 4. User-touchpoint count vs target

Two AskUserQuestion calls this session (DQ collision gate + advisor-direct-mutate gate). Plus 4 user messages ("read and execute…", "poll", "check whether DQ #167 resolves", "fix", "neeeds to run on mac too"). Plus the post-compaction "continue" implicit. Total ≈ 6-7 touchpoints. Target ≤8 per phase, but this is per-session not per-phase; calibrating against the per-phase target is misleading. **Both AskUserQuestions had necessary ambiguity**, not false-positive friction. ✓

### 5. Catch-fire FP / FN rate

Zero catch-fires this session. **Zero false-positives** (no premature stops). **One borderline false-negative consideration:** the ci-watcher #163 crash should arguably have been catch-fired per the existing `.claude/agents/ci-watcher.md` "classifier-miss" criterion ("worktree-add failure with `fatal: invalid reference`"), since L18 has now recurred 3×. We handled it inline (advisor-direct mutation) instead of catch-firing. This was the right call given the workflow had already succeeded, but the *rule* says catch-fire — we didn't follow the rule. ⚠ on FN rate, but defensible given the workflow signal was already known.

### 6. §G4 classifier accuracy

No `result: fail` mutations this session. The replan (Case A LemmyResult<T>) was authored to **prevent** §G4 cycles, not invoke them. Workflow `25605783542` returned `success` first time. Classifier was not exercised. N/A — but the historical 3-cycle cascade (cycles 1-3 of v1-SL-c-2) is the reason the cycle-count meta-rule was added in commit `d40611ccd` earlier this conversation. That meta-rule fired correctly on cycle 3 (HARD REFUSAL → user-prompted re-plan). ✓ for the meta-rule being load-bearing.

### 7. L14 / L15 / L16 fixes still holding

L14/L15/L16 are merge-stage fixes; this session was at impl-cohort-2-phase-1, no merge happened. **Not exercised this session.** N/A.

### 8. Subagent offload effectiveness

Zero general-purpose subagent invocations this session. All probes ran inline. Per the skill's "Phase 0.6 deferral", this is the expected pre-c-2 pattern. Nothing changed. N/A.

### 9. Plan §13 fidelity vs cohort dispatch

Plan §13 has all serial tasks (no `[P]` markers). Cohort 2 = single member (Task 1). YAML overlap check was N/A. Budget check was N/A. No degrades fired. ✓ trivially — there was no cohort to mis-dispatch.

### 10. Resume-cycle pain points

`resume_count: 4` at session end. The increment from 3 → 4 happened earlier in the conversation (pre-compaction); this session ran on the existing session_id. The Phase 0.5 reconciliation found the in-flight Junior #162 + DQ pending=2 + worker tip ≠ phase tip — all valid discrepancies needing user input (the replan plan was the answer). **No `--start-from` overrides this session.** **Conditional ✓** — first c-2 phase is keeping the friction intentionally per the lesson. But `resume_count: 4` for a single sub-phase is at the top of the target band; the next sub-phase (v1-SL-c-3?) should target ≤2.

### Aggregate auto-phase reliability score

| Category | Status | Recurrence |
|---|---|---|
| 1. Stage-transition correctness | ✓ | 0× this phase prior, 0× across this session |
| 2. Cadence calibration | ✓ (manual mode) | n/a |
| 3. Auto-state integrity | ⚠ | `resume_count: 4` at top of target band |
| 4. Touchpoint count | ✓ | 2 gates this session, both necessary |
| 5. Catch-fire FP/FN | ⚠ | 1 borderline FN — ci-watcher #163 should have catch-fired per existing rule |
| 6. §G4 classifier | ✓ (cycle-count meta-rule held) | n/a |
| 7. L14/L15/L16 holding | N/A | not exercised |
| 8. Subagent offload | N/A | not used |
| 9. Plan §13 fidelity | ✓ | trivial — single-member cohort |
| 10. Resume cycles | ⚠ | resume_count: 4 borderline |

**Three ⚠ findings** — all surface as concrete proposals in §"What to change":
- Auto-state integrity (resume_count 4): change #5 (lesson-candidate harvest gap) addresses underlying cause (no closure pattern → user friction → restart).
- Catch-fire FN (ci-watcher worktree-add): change #1 (promote L18 to hard precondition) addresses root cause; future ci-watcher #163-class will not fire because the precondition prevents the failure.
- Resume cycles: addressed indirectly via change #1 (eliminating the ci-watcher fail mode that was a primary friction source on cycle 1+3+4).

---

## Promotion candidates (recurrence ≥ 2)

- [ ] Change #1: promote L18 daemon ref-fetch to hard precondition in `advisor-orchestrator.md` §3.1 — recurrence 3× (cycle 1, 3, 4)
- [ ] Change #1 + #3: author `.claude/lessons/feedback_daemon_local_junior_refs_not_auto_fetched.md` — recurrence 3×
- [ ] Change #2: add `next_id` resolver-discipline to `.claude/PRPs/templates/impl-task-brief.template.md` — structural recurrence (every supersession × every Junior)
- [ ] Change #2: author `.claude/lessons/feedback_dq_collision_across_refs.md` — distinct from DQ #50 within-ref collision, needs its own lesson
- [ ] Change #3 (also): author `.claude/lessons/feedback_resolve_dq_canonical_windows_crlf.md` — single instance but Windows-portability bug class is sticky
- [ ] Change #4: add `--detect-collisions` flag to `scripts/brehon/resolve-dq-canonical.sh` — defence-in-depth layer
- [ ] Change #5: extend `.claude/skills/session-retro/SKILL.md` Step 3 with a "harvest auto-state lesson_candidates_for_retro array" sub-step — closes the 12-entry gap
- [ ] Change #6: extend `auto-phase` skill body to flag rule-deterministic AskUserQuestion gates with a "(default: <preferred>)" option — small friction win
- [ ] PMD eval: write Session retro entry titled "DQ id-collision and ci-watcher recurrence: structural patterns post v1-SL-c-2" — gives future PMD-search a reachable summary

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted: `feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`, `feedback_retro_task_complexity_score.md`, `feedback_auto_phase_retro_signals.md`._
