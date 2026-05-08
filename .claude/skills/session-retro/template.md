# Session retro — <YYYY-MM-DD> — <slug>

**Harness:** <pi | claude-code>
**Session window:** <start-ISO> → <end-ISO> (~<wall-clock-minutes> min)
**Branch at start:** `<sha-short>` (`<branch-name>`)
**Branch at end:** `<sha-short>` (`<branch-name>`)
**Files touched:** <count>
**Commits:** <count> (auto: <n>, explicit: <m>)

## TL;DR

<One paragraph. The session's main thread, the most-load-bearing
finding, and the top change proposal. Reads cleanly out of context
six months from now.>

---

## What surprised us

<Per `.claude/lessons/feedback_retro_not_report.md` canonical-header
requirement. Things that didn't go as expected, in either direction
— surprising friction OR surprising effectiveness. One bullet per
item; specific enough that someone reading this without conversation
context understands what surprised whom.>

- <e.g. "Pi self-authored a memory file hardcoding GITHUB_EVENT_NUMBER
  as canonical — unexpected failure mode where the agent's own
  artifacts re-anchor a wrong premise across iterations.">
- ...

## What to change

<Forward-going changes the next session should adopt. THE HEART of
the retro. Each item: concrete proposal with file path, command, or
skill reference. Cost-tagged. Recurrence-tagged. If this section has
fewer than 2 concrete items, the retro is incomplete — rework
before saving.>

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | <e.g. "add /ci-debug-mode toggle to .pi/extensions/lemmy-hooks.ts"> | <e.g. "suppresses auto-commit during CI iteration → no spam-fired workflow runs"> | minor / medium / major | 2× this session, 0× prior |
| 2 | ... | ... | ... | ... |

## What to carry forward

<Patterns and discipline the next session should explicitly inherit.
What worked well enough to make a routine. One bullet per item.>

- <e.g. "Worktree-per-edit pattern for committing to an active
  branch when the primary clone may have a session in flight —
  avoided two would-be-conflicts this session, used twice cleanly.">
- ...

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`. Numbers
don't have to be exact; they have to be defensible from the
transcript.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| <e.g. ci-debug subagent (proposed)> | — | — | — | not yet invoked this session — proposal only |
| <e.g. /ci-debug-mode toggle (proposed)> | — | — | — | not yet invoked this session — proposal only |
| <e.g. auto(pi) tool_result handler> | 5 | 25 | high | <e.g. amplified loop on adr-compliance debug — see "What to change" #1> |
| <e.g. AskUserQuestion> | 3 | 0 | none | clean architectural fork on retro location |
| <e.g. pi-best-practices-audit skill> | 10 | 0 | low | confirmed structural shape clean; surfaced PMD-wiring finding |
| ... | | | | |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. Format:
`<files>/<commits>/<runtime-min>/<max-log-silence-min>`. Flag any
task that scored >55min runtime, >40min log silence, or >8 files
touched as a carry-forward signal for next session.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| <e.g. adr-compliance fix + 5 pi-tooling commits> | 6 | 5 | 90 | 8 |
| ... | | | | |

## Decisions to revisit

<Optional. List anything that warrants its own follow-up — a
separate retro session, a clarify pass, an architectural review.
One line per item.>

- <e.g. "agentScope:'both' is invocation-time only; could a
  .pi/settings.json default reduce friction? Worth a clarify.">

---

## Auto-phase reliability (only if `.claude/auto-state/<phase>.json` or `*-archived-*.json` or `*-catchfire-*.md` exists)

<Per `.claude/lessons/feedback_auto_phase_retro_signals.md`. Skip
this whole section if no auto-phase artifacts exist for this session.

If artifacts DO exist, fill the 10 categories below. "Nothing
surprising" is a valid empty answer — the structure makes absences
visible. Vague findings are not valid. Each category that goes wrong
must have a concrete proposed fix.>

### 1. Stage-transition correctness
<Did each `state X → state Y` fire on the right trigger? Premature
fires? Missed fires? Cohort barriers held?>

### 2. Cadence calibration
<Wasted-poll count per stage. Detection-lag on state changes. Any
300s sleeps? Specific pairs of (delay → next state-change-time).>

### 3. Auto-state integrity
<Final `resume_count`. `session_id` rotations. `last_known_phase_tip`
accuracy. Any hand-edits to recover.>

### 4. User-touchpoint count vs target
<Total count. Six mandatory gates all fired? Any false-positive
AskUserQuestions? Push grants counted separately.>

### 5. Catch-fire FP / FN rate
<Each catch-fire dump classified: real-issue or routine-friction.
Any silent advances past should-have-been-catch-fire? **Zero is the
target — even one is retro red-flag.**>

### 6. §G4 classifier accuracy
<For each `result: fail` mutation: was allowlist/non-allowlist
classification correct? False-positive (auto-fix-impl shipped wrong
code)? False-negative (catch-fire on existing allowlist pattern)?>

### 7. L14 / L15 / L16 fixes still holding
<L14: BM commits runlog before merge? L15: gate-side checks ran
inline (no pre-confirm Junior dispatch)? L16: post-merge
`git ls-remote` showed branch deleted, or fallback fired silently?>

### 8. Subagent offload effectiveness (when used)
<For each Agent invocation under /auto-phase: synthesis usable on
first read? Net-positive context conservation? Token cost vs inline?
If never used, note "deferred per Phase 0.6" and skip.>

### 9. Plan §13 fidelity vs cohort dispatch
<For each cohort dispatched: were `[P]` markers actually file-
disjoint? Any YAML-overlap-check degrades-to-serial? Any budget-
check degrades?>

### 10. Resume-cycle pain points
<For each `resume_count` increment: Phase 0.5 found discrepancies
needing user input, or was 'continue' prompt noise? Any
`--start-from` overrides? **First c-2 phase: keep the friction.
c-3+: if zero discrepancies across N phases, propose auto-continue.**>

### Aggregate auto-phase reliability score

| Category | Status | Recurrence |
|---|---|---|
| 1. Stage-transition correctness | ✓ / ⚠ / ✗ | 1× this phase, 0× prior |
| 2. Cadence calibration | ✓ / ⚠ / ✗ | ... |
| 3. Auto-state integrity | ✓ / ⚠ / ✗ | ... |
| 4. Touchpoint count | ✓ / ⚠ / ✗ | ... |
| 5. Catch-fire FP/FN | ✓ / ⚠ / ✗ | ... |
| 6. §G4 classifier | ✓ / ⚠ / ✗ | ... |
| 7. L14/L15/L16 holding | ✓ / ⚠ / ✗ | ... |
| 8. Subagent offload | ✓ / ⚠ / ✗ / N/A | ... |
| 9. Plan §13 fidelity | ✓ / ⚠ / ✗ | ... |
| 10. Resume cycles | ✓ / ⚠ / ✗ | ... |

Any ⚠ or ✗ MUST appear as a concrete proposal in §"What to change"
above. Aggregate trend across phases (track in successive retros):
- All ✓ on N consecutive phases = automation is reliable; consider
  loosening some manual gates (e.g. auto-continue resume).
- ⚠/✗ recurring on the same category across 2 phases = promote to
  a new lesson (`feedback_<specific-topic>.md`); add a stronger
  preventative measure to the skill body.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

For each item from "What to change" that meets the threshold, the
user may approve promotion. Boxes UNCHECKED by default; user
checks to authorise; a follow-up session (or the user manually)
executes the checked items.

- [ ] <change>: promote to `.claude/lessons/feedback_<topic>.md` (cross-harness lesson)
- [ ] <change>: update existing skill `.claude/skills/<name>/SKILL.md`
- [ ] <change>: new project-scope subagent at `.pi/agents/<name>.md` or `.claude/agents/<name>.md`
- [ ] <change>: new pi extension hook (`pi.registerCommand` / event handler in `.pi/extensions/lemmy-hooks.ts`)
- [ ] <change>: new Claude Code hook (`.claude/settings.local.json`)
- [ ] <change>: PMD eval write (requires `PROJECT_MEMORY_DB` exported on pi side; `setup-memory.sh` on Claude side)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
