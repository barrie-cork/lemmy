---
name: Phase retro gate enforcement at bm-pr
description: Pre-bm-pr must verify `.claude/PRPs/reports/<phase>-retro.md` exists with mtime > halt-retro mtime. Plan §13 Task 11 + CLAUDE.md user gate 6 are easy to forget; needs a file-existence pre-condition check.
type: feedback
---

# Pre-bm-pr retro gate enforcement

Every phase plan's Task 11 (canonical phase-close retro) and CLAUDE.md
user gate 6 (retro sign-off) require the phase retro to exist at
`.claude/PRPs/reports/<phase>-retro.md` BEFORE `bm-pr` opens the PR.
But this gate is **not enforced** — it's a process discipline that
relies on the advisor remembering to dispatch the Task 11 Junior
before queueing bm-pr.

**Why this lesson exists:** Per `.claude/PRPs/reports/v1-RT-r1-retro.md`
§6 process miss. RT-r1 shipped via PR #126 (merged 2026-05-12) without
the canonical Task 11 phase-close retro. Phase went: tasks 1-10 →
fix-impl-1/2/3 → bm-pr → CR triage → fix-impl-4/5/6 → bm-merge. No
retro authored pre-bm-pr. Halt-retro at `ffa2876e3` existed (mid-phase
pause document) but that's a different artifact. The post-ship retro
at `e37d10d53` is the post-hoc fix — best-effort reconstruction from
commit log + brief trail, but live phase signals were lost (no
real-time per-task wall-clock from each Junior task, no
advisor-during-CR-cycle observations recorded).

**Timing nuance — pre-PR vs post-merge retro:**

CLAUDE.md user gate 6 says "Retro sign-off — author retro... → then
`/brehon-phase-transition`" which sequences retro AFTER bm-merge.
Plan template Task N (retro) doesn't specify timing. Some phase
plans (e.g. RT-r1) name retro as Task 11 with "retrospective before
PR open" (cohort barrier on Tasks 1-10).

**The two framings serve different purposes:**

- **Pre-bm-pr retro** captures live signals from the impl phase
  cleanly (before CR/fix-impl noise) but misses CR/fix-impl signals.
- **Post-bm-merge retro** captures the full cycle including CR and
  fix-impl-N, but live impl signals are days old by then.

Many phases will want both, or one that's deliberately late. The
gate below fires only when the **plan §13 names a retro task as a
pre-bm-pr barrier** (parses the plan body for "before PR" or
"before bm-pr" language adjacent to a retro task). When the plan
defers retro to post-merge, the gate is silent.

**How to enforce:** add a pre-condition check to `bm-pr` (the brief
authoring step in the advisor's stage-shape orchestrator AND the
operational script in `.claude/commands/bm/bm-pr.md` Phase 1).

```bash
# Pre-condition (advisor inline, before authoring bm-pr brief)
PHASE_SLUG="<derive from current branch: phase-v1-XXX → v1-XXX>"
PLAN_FILE=".claude/PRPs/plans/${PHASE_SLUG}.plan.md"
RETRO_FILE=".claude/PRPs/reports/${PHASE_SLUG}-retro.md"
HALT_RETRO_FILE=".claude/PRPs/reports/${PHASE_SLUG}-halt-retro.md"

# Detect whether the plan names retro as a pre-bm-pr barrier
RETRO_PRE_PR=0
if [ -f "$PLAN_FILE" ]; then
  if grep -qiE "retro.*(before|prior to).*(pr|bm-pr)" "$PLAN_FILE"; then
    RETRO_PRE_PR=1
  fi
fi

if [ "$RETRO_PRE_PR" = "1" ]; then
  if [ ! -f "$RETRO_FILE" ]; then
    echo "STOP: plan names retro as pre-bm-pr barrier; retro missing at $RETRO_FILE"
    exit 1
  fi
  if [ -f "$HALT_RETRO_FILE" ]; then
    RETRO_MTIME=$(stat -c %Y "$RETRO_FILE" 2>/dev/null || stat -f %m "$RETRO_FILE")
    HALT_MTIME=$(stat -c %Y "$HALT_RETRO_FILE" 2>/dev/null || stat -f %m "$HALT_RETRO_FILE")
    if [ "$RETRO_MTIME" -le "$HALT_MTIME" ]; then
      echo "STOP: phase retro $RETRO_FILE is older than halt-retro (halt-retro is mid-phase, not phase-close)"
      exit 1
    fi
  fi
else
  # Plan defers retro to post-merge → no pre-bm-pr gate
  # But surface a reminder so the advisor doesn't skip retro entirely (CLAUDE.md user gate 6)
  echo "INFO: plan defers retro to post-merge (no pre-bm-pr gate fires here)."
  echo "Reminder: retro IS still required at user gate 6 post-bm-merge."
fi
```

The check is **plan-aware** so phases that intentionally defer retro
to post-merge are not blocked. The check is **content-light** (just
existence + mtime, not quality), so the user-interactive sign-off at
gate 6 still owns content quality judgment.

The check belongs in **two places** (defence in depth):

1. **Advisor stage-shape orchestrator** — before authoring the bm-pr
   brief, the advisor runs the check inline; if missing, refuse to
   queue bm-pr and surface to user with "Task 11 retro missing —
   dispatch Junior to write it first."
2. **bm-pr operational script** — `.claude/commands/bm/bm-pr.md`
   Phase 1 pre-conditions table gets a new row:
   `| Retro missing at .claude/PRPs/reports/<phase>-retro.md | STOP: "Task 11 retro must land before PR opens." |`

**Why both layers:** the advisor-side check is the design-time gate
(no Junior dispatch happens without it). The operational-script check
is the run-time backstop (if a Junior was dispatched some other way,
the bm-pr Junior itself refuses). The two layers cost ~5 lines of
shell each; preventing a re-occurrence of the RT-r1 post-hoc retro is
worth more than the maintenance cost.

**Edge cases:**

- **No halt-retro exists:** skip the mtime check; existence-only check
  suffices. Most phases don't have a halt-retro.
- **Halt-retro authored same day as phase retro:** mtime comparison
  may return equal. Use `-le` (less-than-or-equal) for safety; if they
  truly were written within 1s of each other, that's still a
  signal that something is off (file the case to retro).
- **Retro file present but empty/stub:** the existence check passes
  but the content is hollow. Out of scope for this gate — content
  quality is judged at user gate 6 (retro sign-off), which is a
  separate user-interactive step.
- **Phase shipped without retro (historical):** RT-r1, possibly
  others. The gate cannot retroactively force a retro to exist on
  shipped phases; this is forward-only-immune per
  `feedback_schema_changing_spec_retrofit_question.md`. Post-hoc
  retros (like `e37d10d53` for RT-r1) are the historical fix.

**Companion lessons:**

- `feedback_retro_not_report.md` — retro is a retrospective, not a
  completion report; different artifact.
- `feedback_four_role_retro_signals.md` — retro structure per role.
- `feedback_retro_task_complexity_score.md` — mandatory §4 per-task
  complexity table.
- `feedback_retro_substantive_artifact_verify.md` — verify retro
  claims at retro-gate (separate user-gate 6 concern).

**Where codified:**

- `.claude/commands/bm/bm-pr.md` Phase 1 pre-conditions table.
- `.claude/rules/advisor-orchestrator.md` §3.1 stage-shape
  orchestration "All §16a stories [done] → bm-pr" branch (add retro
  check ahead of bm-pr dispatch).
- This lesson file.
