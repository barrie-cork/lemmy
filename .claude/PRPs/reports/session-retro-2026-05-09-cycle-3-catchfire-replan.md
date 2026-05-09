# Session retro — 2026-05-09 — cycle-3 catchfire + replan

**Harness:** claude-code (advisor session, Opus 4.7 1M)
**Session window:** 2026-05-09 ~14:50 UTC → ~16:55 UTC (~125 min wall-clock incl. compaction)
**Branch at start:** `9ba756950` (`governance-v0`)
**Branch at end:** `f588faac0` (`governance-v0`, 3 commits ahead of origin)
**Files touched:** 6 advisor-side artifacts (lesson amendment, new lesson, RCA, replan brief, plan §19a, interim retro) + auto-state JSON (gitignored) + force-pushed worker branch
**Commits:** 3 (all authored by a concurrent advisor session that won the race; this session authored content but did not commit)

## TL;DR

This session resumed `/auto-phase v1-SL-c-2` post-compaction, polled the cycle-3 ci-watcher
through completion, classified the third E0277/E0283 catch-fire on the same e2e.rs site,
and pivoted from §G4 mechanical-fix mode to plan-mode after the user's "go back and plan
it with the knowledge we now have" course-correction. The plan-mode work surfaced the
**root cause that 3 cycles missed**: the v1-SL-c-2 plan §13 Task 1 stub directly
contradicted v1-SL-c-1's §9.2 hand-off by prescribing `Box<dyn Error>` shape when the
canonical sibling v1-SL-b uses uniform `LemmyResult<T>`. The recipe family was wrong-shaped
from the start; mechanical fixes never had a path to converge. Highest-leverage outcome:
the lesson amendment (case A/B/C enumeration) + §G4 row split (4a/4b/4c) + new
`feedback_plan_stub_uniformity_with_canonical_sibling.md` lesson. Highest-leverage process
finding: **3 cycles is the upper bound — at that point, re-plan, do not keep cycling**.
The §G4 classifier needs an explicit `cycle_count >= 3 → catch-fire even if allowlist`
rule, not just per-cycle classification.

---

## What surprised us

- **The §G4 canonical recipe was structurally insufficient even when copied verbatim.**
  The anti-paraphrase gate (commit 558cef1a5) was added between cycles 1 and 2 specifically
  to prevent brief-authorship inversion of the row text. Cycle 3's brief (`fix-impl-2.md`)
  followed the gate to the letter and copied row 4 verbatim. The recipe still failed because
  the row text itself was insufficient for abstract trait object outers (E0283 type-inference
  defeat). The gate solved one class of failure; it can't solve "the canonical text is
  itself wrong-shaped".

- **A concurrent advisor session was authoring identical artifacts in parallel.** When this
  session was in plan-mode authoring the brief / RCA / lesson amendment / plan §19a, a
  *different* advisor session (likely on the EliteDesk via SSH-shared checkout, or another
  CC instance the user opened) was authoring the same files with the same scope and shape.
  Both sessions converged on identical content. The other session won the commit race
  (commits `e109dd82b`, `bbbb92a43`, `f588faac0` at 16:53-16:54 UTC). My uncommitted
  working-tree edits became no-op duplicates. Surprising: the cross-session correctness was
  high (no real conflict), but the parallelism was unintentional from the user's side and
  could have wedged on a less-aligned task.

- **The MCP `list_tasks` shim returned 97k chars** — well past the per-tool token limit
  for direct ingestion, even though the daemon DB only has ~160 jobs. The shim doesn't
  paginate or filter at the source. Forced a fallback to direct `sqlite3` over SSH.

- **The user's "is mechanical-fix robust or should we plan?" gut-check was right** when
  my recommendation was wrong. I'd surfaced "Switch helpers to LemmyResult (Recommended)"
  as the next mechanical hypothesis. The user pushed back: "is that robust or should we
  plan with the knowledge we now have?" Planning was the correct call — and the research
  that planning surfaced (v1-SL-b uniform shape, c-1 §9.2 hand-off) was load-bearing
  evidence the mechanical-fix path had no way to access. The advisor was momentum-biased
  toward the next mechanical try; the user's pause-and-plan instinct was the unbiased
  correct call.

- **`/auto-phase` Phase 0.7 lazy-load discipline worked as designed.** Resume #5 (after
  the latest compaction) loaded only the state file + skill body; the file-class table
  was NOT preloaded. When the cycle-3 failure required reading `feedback_lemmy_error_no_std_error.md`,
  it was read just-in-time. Total resume token spend was visibly lower than resume #2
  (which had pre-Phase-0.7 anti-pattern preloading), though I don't have exact `/context`
  measurements.

- **Force-push to a worker branch + daemon ref refresh chain has 3 known failure modes**
  this session hit two of them: (1) `git fetch origin +refs/heads/junior/*:refs/heads/junior/*`
  fails with "refusing to fetch into branch ... checked out at" when a daemon worktree has
  the branch active (#161 ci-watcher-3 was running on its branch, blocking refresh of all
  junior branches); (2) targeted single-branch fetch into `refs/remotes/origin/*` + then
  `git reset --hard` on the daemon worktree works when the worktree's branch IS the target,
  but not when it's a sibling branch. Both modes are documented in lesson_candidates_for_retro
  but the chain still tripped me twice this session.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add `cycle_count >= 3 on same compile-error class → catch-fire as catch-fire to user, regardless of allowlist match` to `.claude/rules/advisor-orchestrator.md` §5.3 §G4 classifier preamble. The §G4 row split (4a/4b/4c) is now in place; this rule prevents the classifier from cycling forever even when each individual classification looks defensible. | Forces re-plan signal at the 3-cycle boundary; saves ~30 min per same-class repeat catch-fire. Aligns with what user instinct already demanded this session. | minor (one paragraph + one cyclecount field in auto-state schema) | 1× this session (cycles 1-3 on same site); the prior pattern was always handled manually by user |
| 2 | Promote `feedback_plan_stub_uniformity_with_canonical_sibling.md` (already authored this session) to PMD via lesson re-sync. After commits land on origin, run `bash scripts/sync-lessons-to-pmd.sh` AND `sqlite3 .project-memory/memory.db "DELETE FROM memories WHERE title = 'LemmyError doesn''t implement std::error::Error';"` (force re-import the amended lesson body). | Future PMD search returns case-enumerated body, not pre-amendment recipe. Closes the lesson lifecycle loop (file → PMD → searchable from impl-task subagent). | minor (3 commands) | this session triggered the amendment; PMD desync is the standard hazard for amended lessons |
| 3 | Add a "concurrent advisor session" detection probe to `/auto-phase` Phase 0.5 Step E. Currently the rule says "concurrent advisor session writing same phase" is a refusal but only detects via `.claude/agent-activity.json` which doesn't exist on this checkout. Add a fallback probe: `git log --since='10 min ago' --grep '^chore(advisor)\|^docs(advisor)\|^chore(decision-queue)' --author='solo-dev'` — if this session sees recent advisor commits not authored by it, surface to user before any state-changing action. | Prevents the no-op duplicate-authoring pattern this session hit. Even when content is identical, the parallel work wastes cycles and hides genuine conflicts behind "they both committed". | minor (one Bash probe in Phase 0.5 Step E) | 1× this session; not previously seen but emerging risk as user opens parallel CC instances |
| 4 | Replace the MCP `mcp__junior-brehon__list_tasks` ingestion path in `/start-brehon` and `/auto-phase` Phase 0.5 with direct `sqlite3` over SSH **as default** (already established as the workaround per lesson_candidates_for_retro entry). Move from "fallback when MCP returns >100k chars" to "primary path; MCP only when SSH unreachable". | Saves ~3-5s per probe + avoids context-bloat from oversized shim returns. The MCP shim isn't fixable from the advisor side; the SSH path is reliable. | minor (one Bash command swap) | ≥3× across c-2 sub-phase (every resume cycle) |
| 5 | Document the force-push-to-worker-branch + daemon-refresh chain as a single advisor recipe in `.claude/rules/advisor-orchestrator.md`. The 3 failure modes + 3 workarounds already in `auto-state.lesson_candidates_for_retro[]` should consolidate into one "Worker branch hard-reset on daemon" sub-section that the next sub-phase's first-time-encounter advisor can read once. | Saves ~5-10 min on the next worker-branch-reset (replan) episode. The chain is mechanical but not obvious. | minor (one new sub-section) | 2× this session (force-push + ssh refresh); ≥4× across v1-SL-* phases per the lesson candidates |
| 6 | Add `decline_jury_assignment` symptom to the v1-SL-c-2 retro carry-forward (per `feedback_lemmy_error_no_std_error.md` symptom-recognised section): the v1-JM-e gap was already noted in PMD index, this session noticed the same pattern. Surface as a watch item: when the next jury-mechanics phase opens, check whether `decline_jury_assignment` test fn or helpers were written under Case A or Case B shape. | Prevents next sub-phase repeating the same catch-fire across a different module. The case enumeration in the lesson is the preventative; the watch item makes "did we apply it" visible. | minor (one watch item in carry-forward) | 1× this session (planning side); next jury phase will validate |

## What to carry forward

- **Plan-mode pivot at cycle 3 boundary.** The user's "is mechanical-fix robust or should
  we plan?" decision was the inflection point. Carry forward: at any 3-cycle boundary on
  the same compile-error class, **default to plan-mode** rather than continuing §G4
  cycling. This pairs with change #1 above.

- **Canonical-schema-first as a load-bearing gate, not advisory.** The plan-mode research
  this session demonstrated that reading the canonical sibling (v1-SL-b at e2e.rs:11001-11924)
  before authoring any new fixtures module is the difference between "1 cycle of correct
  work" and "3 cycles of failed mechanical fixes". The canonical-schema-first rule (already
  in `advisor-orchestrator.md` §3.6) is the right rule; carry forward by treating it as
  hard refusal at brief-authoring time, not "advisory before".

- **Lesson amendment in place + new sibling lesson + rule split as one coherent commit
  set.** The other-session commits `e109dd82b` + `bbbb92a43` + `f588faac0` form a clean
  retro-time intervention pattern: lesson body amended (forward-only), new lesson authored
  (sibling discipline), rule split for the classifier. The pattern is repeatable for any
  future "the recipe family was wrong-shaped" scenario. Carry forward as a recipe in the
  next phase retro template.

- **Direct `sqlite3` over SSH as the primary daemon-DB read path.** Already established
  this session; carry forward by codifying in the rules per change #4.

- **Compact resume report (≤15 lines, ≤500 tokens) is working.** Phase 0.5 Step E's
  compact-report shape was visibly more efficient than the prior verbose form. Resume #5
  did not bloat parent context with raw probe outputs. Carry forward; this is the right
  shape.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/auto-phase v1-SL-c-2 resume` (resume #4 + #5) | 5 | 0 | none | Cadence calibration was right (270s cache-warm); detection of cycle-3 failure happened on first post-completion poll. |
| Plan-mode entry (`/remote-control Fix` → ExitPlanMode) | 60 | 0 | medium | Surfaced the v1-SL-b §9.2 hand-off as the load-bearing root cause that 3 mechanical cycles missed. The user's pivot decision was the trigger; the skill (plan-mode) executed cleanly. |
| 3 parallel Explore subagents (Phase 1) | 25 | 0 | low | Returned exactly the citations needed: v1-SL-b canonical shape, current cycle-3 e2e.rs state, pre-amendment lesson body. ~50-70k subagent tokens for ~1KB synthesis each was net-positive on cold parent context. |
| `mcp__junior-brehon__list_tasks` | 0 | 5 | high | Returned 97k chars, exceeded direct ingestion limit, forced fallback to SSH. Surprise = high because the same call worked on prior sessions (caps appear context-dependent). |
| `gh run view 25603848858 --log-failed` | 3 | 0 | none | Captured E0283 type-annotation hint cleanly; sufficient for §G4 classification without full log read. |
| AskUserQuestion (cycle-3 catch-fire 4 options) | 0 | 0 | medium | User chose "Revert all 3 cycles + re-plan" — the option I had ranked third. The 4-option fan-out was right; the user picked the slow-but-correct path. Surprise = medium because my recommendation ("Switch helpers to LemmyResult") was option 2; user's choice was option 4. |
| AskUserQuestion (revert mechanics — interrupted) | 0 | 0 | n/a | User interrupted with "is that robust or plan?" before I dispatched. Net-zero impact on outcome; carries forward as evidence the user's gut-check on momentum was right. |
| Force-push impl-1 worker branch | 5 | 5 | low | Push succeeded on first attempt; daemon refresh chain hit "refusing to fetch into checked-out branch" but recovered with targeted single-branch refresh + reset --hard. Net-zero against the 3-cycle context. |
| `bash scripts/brehon/git-show-json.sh` (DQ on worker branch) | 2 | 0 | none | Standard tool, worked as designed. |
| ScheduleWakeup (270s cache-warm) | 3 | 0 | none | Right delay for the in-progress workflow. |
| Plan file write (`rippling-twirling-wilkinson.md`) | 8 | 0 | low | Captured the entire research → recommendation → mechanics → verification chain in one file; ExitPlanMode handed off cleanly. |

**Aggregate:** ~111 min "saved" (largely on the plan-mode pivot + parallel research) vs ~10 min "wasted" (MCP shim shape + force-push chain friction). Net win, but the win was unlocked by the user's correction, not by skill defaults.

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. Format: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Plan-mode authoring (brief + RCA + lesson amendment + new lesson + plan §19a + retro draft) | 6 | 0* | ~50 | n/a (foreground session) |
| Force-push impl-1 worker branch + daemon refresh | 1 ref | 0 | ~5 | n/a |
| ci-watcher-3 polling cycle (#161, workflow 25603848858) | 0 (read-only) | 0 | ~22 (workflow 21m + advisor poll latency) | ~22 (workflow opaque until completion) |

*Commits = 0 because the concurrent advisor session won the race; this session authored content but did not commit. The other-session commits totalled 3 (`e109dd82b`, `bbbb92a43`, `f588faac0`).

**Outliers:** none cross the 55min/40min/8files thresholds individually. The aggregate session (~125 min) was driven by the plan-mode pivot, not by any single task's bloat. **The 3-cycle aggregate task** (Task 1 across cycles 1-3 + replan) totals ~123 min wall-clock for ~6 lines of code change, per the interim cycles-retro §4 score (9/10 complexity).

## Decisions to revisit

- **Concurrent advisor session detection probe** (change #3) — needs a clarify pass: is the
  user opening parallel CC sessions deliberately (pi-style multi-agent), or is this an
  EliteDesk-side SSH session running the same `/auto-phase`? The detection probe is right
  in either case, but the user's intent shapes whether we should refuse or merely surface.

- **PMD lesson re-sync timing.** Currently manual: after a lesson body amendment, advisor
  must remember to run `sync-lessons-to-pmd.sh` + `DELETE WHERE title=`. This session's
  amendment will desync until someone runs it. Worth a `PostToolUse` hook on
  `.claude/lessons/feedback_*.md` Write/Edit? The skill body §"Pre-queue lesson check"
  notes the manual cadence is "matched to authorship cadence ~1-2 per phase" but missed
  amendments are silent until the next phase's PMD search returns stale content.

- **§G4 row split (4a/4b/4c) effectiveness validation.** Plan-mode landed the row split;
  the next E0277 LemmyError catch-fire is the validation. If the next sub-phase dispatches
  a fix-impl that picks the right row first-try, the case enumeration was sufficient. If
  not, the row text itself is still wrong-shaped and needs another iteration.

---

## Auto-phase reliability

Per `feedback_auto_phase_retro_signals.md`. v1-SL-c-2 is the first sub-phase under
`/auto-phase`; this is the first such retro for c-2.

### 1. Stage-transition correctness

**⚠ — One transition was structurally correct but operationally wrong.** The skill correctly
detected workflow 25603848858 completed=failure, correctly read the mutated DQ #166 with
`result: fail`, correctly classified as cycle-3 of the same E0277/E0283 class, correctly
surfaced via AskUserQuestion. All transitions fired on the right trigger. BUT: the §G4
classifier had no rule to escalate cycle-N >= 3 to catch-fire automatically. The transition
"validate-pending fail → AskUserQuestion 4-option fan-out" was correct skill-body behaviour
but the right transition for cycle 3 should have been "auto-catch-fire with cycle-count
context" without requiring the AskUserQuestion at all. **Fix:** change #1 above.

`user_gate_history` notes were read at retro time — the plan-approval gate's note ("DoD
smoke PASS 10/11; watchpoint specificity gate PASS 14/14; e2e.rs LOC 11925 (higher than
planner's ~10500-10600 expectation)") was a durable finding that matched the cycle-3 root
cause: planner's expectation of e2e.rs LOC was off, suggesting the canonical sibling read
was incomplete. Surfaces as evidence for change #6.

### 2. Cadence calibration

**✓** — 270s cache-warm cycle was right for ci-watcher Phase 1 workspace polling. Detection
lag from workflow completion (15:06 UTC) to advisor classification (15:08 UTC, next tick) was
~2 min — well under the 5-min ceiling. No 300s sleeps. Wasted-poll count: 1 (the 14:57 tick
saw workflow still in_progress with no state change). Acceptable.

### 3. Auto-state integrity

**⚠** — `resume_count: 4` after cycle-3 catch-fire pivot, then incremented to 5 in the
last_action narrative (the JSON's `resume_count` field was not actually incremented in this
session's edits; only the narrative captures the increment). State file accuracy:
`last_known_phase_tip: 47af884c8` is correct (phase tip never advanced because all 3
cycles failed at workspace-check, no daemon finalize-merge happened). `last_known_phase_tip_worker_branch:
3fdb666a4` is now stale (worker branch was hard-reset to `c2761b284` this session) — the
field needs an update if it's load-bearing for next-tick logic. **Fix:** add `resume_count`
auto-increment to Phase 0.5 Step C action; reconcile `last_known_phase_tip_worker_branch`
post-revert.

### 4. User-touchpoint count vs target

**⚠** — Touchpoints this session: 3 AskUserQuestion (cycle-3 4-option, revert mechanics
[interrupted], plan approval ExitPlanMode). Plus 1 `/remote-control Fix` invocation. Total
4 in ~125 min — well within the ≤8 target on a per-session basis. BUT: the cycle-3
4-option AskUserQuestion was a borderline false-positive friction — if change #1 had been
in place, it should have been an auto-catch-fire surface, not a 4-option fan-out. The
4-option shape *was* useful here because it surfaced the user's "go back and plan" course
correction; under change #1 the same surface would happen but with the cycle-count framed
explicitly.

The six mandatory gates: 1 fired this session (plan-approval, before c-2 began on
2026-05-08). Gates 2-6 are all post-impl, deferred. None skipped, none missed. ✓ on the
6-gate count specifically.

### 5. Catch-fire FP / FN rate

**⚠ — One ambiguity-class catch-fire that resolved correctly but should have surfaced
sooner.** The cycle-3 surface to user was correct (per Phase 5 "do NOT auto-retry" rule);
the §G4 classifier never advanced past a fail without surfacing. So FP rate = 0 (no
spurious catch-fires), FN rate = 0 (no silent advances past real issues). But the
*timing* of catch-fire was sub-optimal: cycle 3 should have catch-fired on cycle-count
threshold, not on each-cycle classification. Currently the skill catch-fires per the
classifier's per-cycle decision; the user had to make the cycle-count call. Borderline —
classifier was correct on a per-cycle basis (cycle 3 legitimately matched a row-shape
question worth asking), but the meta-pattern (3 cycles same class) was the real signal.

### 6. §G4 classifier accuracy

**⚠ — All 3 classifications were defensible per the row text in place at each cycle's
classification time, but the row text itself was insufficient.** Cycle 1: classifier
didn't fire because impl-task brief lacked the file-class injection (the rule was added
mid-phase). Cycle 2: classifier correctly identified E0277, surfaced fix-impl-1 brief
that inverted the canonical recipe (brief-authorship issue, not classifier issue).
Cycle 3: classifier correctly identified E0277, brief copied row 4 verbatim, recipe
itself was wrong-shaped. **Fix:** the row split (4a/4b/4c) landed this session via the
other-session commits + this session's plan-mode authoring of the same content. Validation
deferred to next E0277 phase.

False-positive rate: 0 (no auto-fix-impl was queued for a real bug — every fix-impl was
queued for a legitimate compile error). False-negative rate: 0 (no catch-fire missed a
real issue). Classifier accuracy was correct in the binary sense; the row-shape
correctness is the real lesson.

### 7. L14 / L15 / L16 fixes still holding

**N/A** — c-2 has not reached merge stage yet. L14/L15/L16 cover bm-merge phase mechanics;
this session was mid-impl-cohort, no bm-pr or bm-merge activity. Defer to full c-2 retro.

### 8. Subagent offload effectiveness (when used)

**✓** — Three Explore subagents in plan-mode Phase 1 returned tightly-scoped syntheses
(<400 words each per the prompt cap). Net context: subagents burned ~150-210k tokens
combined; parent received ~3 KB of synthesis (~1 KB each). On a session that was
post-compaction-cold, this was the right delegation pattern. First-read usability:
high — each synthesis answered the prompt directly with file:line citations; no
follow-up reads needed.

### 9. Plan §13 fidelity vs cohort dispatch

**⚠ — Plan §13 Task 1 stub was wrong-shaped from the start.** Cohort 1 dispatched
Task 0 (pre-flight harness audit) as serial (no `[P]` markers); that worked correctly.
Cohort 2 dispatched Task 1 (the catch-fire); the dispatch itself was correct but the
*content* of the §13 stub it dispatched was wrong (Box<dyn Error> outer when canonical
sibling uses LemmyResult<()>). The skill's cohort dispatch logic worked; the plan it
dispatched from was the bug. **Fix:** change #6 above (canonical-sibling watch item) +
the new `feedback_plan_stub_uniformity_with_canonical_sibling.md` lesson to prevent
recurrence at next-plan time.

YAML overlap check + budget check: both fired correctly (no degrade-to-serial events
this session because the plan has no `[P]` markers).

### 10. Resume-cycle pain points

**⚠ — `resume_count: 4` (or 5 per narrative) is at the upper edge of the ≤3 target.**
Each resume corresponded to a real session boundary (compaction events at end of c-1
retro, end of c-2 plan-approval, mid-cycle 2, post-cycle-3 catch-fire). Phase 0.5
reconciliation found discrepancies needing user input each time — none were noisy.
Zero `--start-from` overrides. Resume friction was load-bearing on every resume
(the user's "continue" prompt was informative, not noise). **No change needed for c-2;**
revisit at c-3 boundary if the count keeps climbing.

### Aggregate auto-phase reliability score

| Category | Status | Recurrence |
|---|---|---|
| 1. Stage-transition correctness | ⚠ | 1× this phase (cycle-count threshold missing) |
| 2. Cadence calibration | ✓ | 0× concerns |
| 3. Auto-state integrity | ⚠ | 1× this phase (resume_count not auto-incremented; phase_tip_worker stale) |
| 4. Touchpoint count | ⚠ | 1× borderline (cycle-3 4-option could be auto-catch-fire) |
| 5. Catch-fire FP/FN | ⚠ | 1× this phase (cycle-3 timing sub-optimal) |
| 6. §G4 classifier | ⚠ | 1× this phase (row text wrong-shaped; now split) |
| 7. L14/L15/L16 holding | N/A | (c-2 not yet at merge stage) |
| 8. Subagent offload | ✓ | 0× concerns |
| 9. Plan §13 fidelity | ⚠ | 1× this phase (stub vs canonical sibling mismatch) |
| 10. Resume cycles | ⚠ | resume_count 4-5, upper edge of target |

**Trend:** 6 ⚠ on first c-2 phase. All trace back to **the cycle-count meta-rule**
(changes #1) and **the lesson body case-enumeration miss** (already fixed via the
other-session lesson amendment commit). With those two changes, 4-5 of the ⚠s would
flip to ✓ on next phase. The remaining ⚠s (auto-state integrity, resume-cycle count)
are minor-cost minor-effect.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Change #1** (cycle_count >= 3 → catch-fire rule): promote to `.claude/rules/advisor-orchestrator.md` §5.3 §G4 classifier preamble. ≥1 here + ≥1 in prior memory (every prior catch-fire surface was user-initiated, not skill-initiated).
- [ ] **Change #2** (PMD lesson re-sync after amendment): if recurrence increases, build a `PostToolUse` hook on `.claude/lessons/feedback_*.md` Write/Edit. Currently 1× this session.
- [ ] **Change #3** (concurrent advisor session detection probe): promote to `/auto-phase` Phase 0.5 Step E. 1× this session; emerging risk pattern.
- [ ] **Change #4** (sqlite3-over-SSH as primary daemon-DB read path): promote to `/start-brehon` and `/auto-phase` Phase 0.5 rules. ≥3× this session, ≥1× prior (per lesson_candidates_for_retro entry).
- [ ] **Change #5** (force-push + daemon-refresh recipe): promote to `.claude/rules/advisor-orchestrator.md` "Worker branch hard-reset on daemon" sub-section. 2× this session; ≥4× across v1-SL-* per lesson candidates.
- [ ] **Change #6** (canonical-sibling watch item): merge into `feedback_plan_stub_uniformity_with_canonical_sibling.md` symptom-recognised section. 1× this session; high preventative value for next phase.
- [ ] PMD eval write: title `Session retro: 3-cycle catch-fire on same compile-error class triggers re-plan, not classifier-row iteration`; tags `retro,brehon,auto-phase,g4-classifier,catch-fire,replan`; source_ref `governance-v0`. PMD is wired (`.project-memory/memory.db` exists) — eligible for write.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`, `feedback_auto_phase_retro_signals.md`._
