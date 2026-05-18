---
name: A lesson documenting a footgun in a repo-owned fixable artifact must ship or track the structural fix, not just describe the risk
description: Authoring a lesson about a footgun in a script/wrapper/config the repo controls is necessary but NOT sufficient — a documented-but-unfixed fixable footgun re-fires on the next run AND gives false confidence the risk is "handled". Pair the lesson with the structural fix in the same/next commit, or record "fix status: SHIPPED <sha> / PENDING" in the lesson body so the gap is visible, not invisible.
type: feedback
---

# A lesson about a *fixable* footgun must be paired with the structural fix

When you author a `.claude/lessons/feedback_*.md` (or any durable
write-up) describing a footgun whose root cause lives in an **artifact
this repo owns and can change** — a script, a wrapper, a tracked
config, a template, a hook — the lesson alone does **not** close the
risk. The lesson is the *knowledge*; the structural fix is what
*removes* the footgun. Shipping the lesson without the fix is strictly
worse than shipping nothing, because:

1. The footgun **re-fires on the very next run** (the mechanism is
   unchanged), and
2. The lesson's existence creates **false confidence** that the risk is
   "handled" — a future session searching PMD finds the lesson, reads
   "known footgun, documented", and moves on without realising the
   mechanism is still live.

This is distinct from a footgun in something we *don't* control
(upstream tool behaviour, a harness gate, a platform limitation) —
there a lesson + a mitigation recipe is genuinely the best available
artifact, because there is no structural fix to make. The discipline
here is **scoped to footguns whose root cause is fixable by us**.

## The rule

When the lesson's root cause is a repo-owned fixable artifact, the
lesson must be accompanied by **one of**:

- **(preferred)** the structural fix, committed in the same commit or
  the immediately-following commit; the lesson body states
  **`fix status: SHIPPED <sha>`**; or
- the structural fix opened as the **explicit next action**, with the
  lesson body stating **`fix status: PENDING — <what + where + why
  deferred>`** so the gap is *visible*, not silently implied.

A lesson on a fixable footgun with neither a shipped fix nor an
explicit `fix status: PENDING` line is the failure this lesson names.
"We wrote it down" is not "we fixed it."

## Why (the incident)

**v1-ship-1-r2, 2026-05-18.** `feedback_pmd_cross_lane_canonical_db.md`
was authored this phase, correctly documenting that
`scripts/sync-lessons-to-pmd.sh` strands lessons in a *lane-local* DB
when run from a lane worktree (the relative `PROJECT_MEMORY_DB`
resolves against the per-worktree `PROJECT_ROOT`). The lesson described
the failure mode precisely. **But the script itself was not fixed.**

The unfixed script then ran — from the lane worktree, during the
session-retro Step 5.5 — and stranded all 5 of that session's lessons
in `brehon-fork-ship-1/.project-memory/memory.db` instead of the
canonical `brehon-fork/.project-memory/memory.db`. Among the 5
stranded: `feedback_pmd_cross_lane_canonical_db.md` itself — the lesson
about lane-local stranding was made invisible to cross-lane search *by
the exact bug it documented*. The knowledge existed; the mechanism was
live; the lesson's own reachability depended on the fix it implied but
did not perform.

The same lane-local-default root manifested **twice this phase**
(recurrence ≥2): (1) stranded `Task retro:` eval rows that needed a
separate reconciliation (debug scaffolding left at
`.claude/PRPs/debug/reconcile-*.json`), and (2) this stranded-lesson
-sync. Both would have been prevented had the *first* lesson about the
lane-local-default been paired with the `sync-lessons-to-pmd.sh`
structural fix instead of describing the risk and leaving the script
unchanged. The fix, when finally made (`53b1a52c1`: `--db` precedence
+ lane-local sanity-check WARN + an arg-parse bug fix), took ~15
minutes — cheaper than the remediation it followed, and far cheaper had
it ridden with the original lesson.

## How to apply

- **At lesson-authoring time**, ask: *is the root cause in something
  this repo owns and can change?* If yes → the lesson is incomplete
  until it carries a shipped fix or an explicit `fix status: PENDING`
  line. If no (upstream / harness / platform) → a mitigation recipe in
  the lesson is the correct and sufficient artifact; this discipline
  does not apply.
- **Prefer same-commit pairing.** A lesson + its structural fix in one
  commit is self-evidently closed. Split only when the fix needs its
  own focused session — and then the `fix status: PENDING` line, with
  the *where* and *why-deferred*, is mandatory so the gap is auditable.
- **Retro / session-close check.** When a session promotes a new lesson
  (session-retro §3, post-task-retro step 4/5), the promotion is not
  "done" until each promoted lesson whose root cause is repo-owned has
  either a shipped fix or a recorded `fix status: PENDING`. Surface
  any unpaired fixable-footgun lesson in the Step 6 user surface as an
  open item, not a closed one.
- **Triage check.** When reading a lesson during triage and it
  describes a fixable footgun with no `fix status:` line, treat the
  mechanism as **still live** — do not assume "documented" means
  "handled". The absence of a fix-status line is itself the signal
  that the structural fix may never have been made.

## Generalises to

Any durable knowledge artifact (lesson, runbook entry, ADR note,
postmortem action item) that documents a defect whose root cause the
team can structurally remove. The failure mode — "we wrote up the
footgun, so we feel covered, but the mechanism is unchanged and
re-fires" — is the classic *action-item-without-an-owner* /
*documentation-as-substitute-for-fix* anti-pattern. The cure is the
same everywhere: a write-up of a *fixable* defect must carry the fix
or an explicit, visible pointer to the pending fix. Same family as
`feedback_pre_phase_dod_smoke_test.md` (act on verified current
reality, not a prior assumption that something is handled) and
`feedback_principles_not_rules.md` (the artifact must change behaviour,
not just record intent).

## Symptom to recognise

A footgun re-fires; investigation finds a lesson that *already
documented exactly this footgun* was on disk (often authored the same
phase, sometimes by the same session); the lesson has no
`fix status:` line and the root-cause artifact (script/config/wrapper)
is unchanged since the lesson was written. The sharpest tell: the
lesson about the footgun was *itself* damaged by the footgun (stranded,
overwritten, skipped) — the documentation and the unmade fix were the
same gap.

## See also

- `feedback_pmd_cross_lane_canonical_db.md` — the lesson that triggered
  this one by being stranded by the bug it documented; now paired with
  its structural fix (`53b1a52c1`).
- `.claude/PRPs/reports/session-retro-2026-05-18-pmd-stranding-remediation.md`
  — the incident retro ("What to change" #1).
- `feedback_principles_not_rules.md` — an artifact must change
  behaviour, not merely record an intention.
- `feedback_pre_phase_dod_smoke_test.md` — verify current reality
  rather than assuming a prior write-up made the risk go away.
