# Session retro — 2026-06-17 — m3-prd-resume

**Harness:** claude-code
**Session window:** ~2026-06-17 (single focused thread, ~25 min wall-clock)
**Branch at start:** `67bf10e6b` (`governance-v0`)
**Branch at end:** `300161009` (`governance-v0`)
**Files touched:** 5
**Commits:** 2 (auto: 0, explicit: 2)

## TL;DR

`/prp-prd m3-town-halls-rtc` was invoked as a fresh-PRD command, but it was
actually a **resume**: a prior session (handover 2026-06-15) had run the entire
scope-clarification gate, locked D1–D4, and PAUSED on one open decision (D5,
recording purpose). The session correctly read the handover trail first, detected
the resume, surfaced D5 via `AskUserQuestion` (→ Option C), folded in a D6 the user
added mid-flight ("it should be optional too"), and authored the PRD + resolved
OQ-V2-04 + advanced the roadmap. Clean, no dead ends. The one carry-forward worth
acting on: **`/prp-prd`'s Phase-1 scope-check assumes a blank page and has no step
to detect an in-flight authoring handover** — it worked here only because the
handover was read before the command body was followed.

---

## What surprised us

- **A "generate a PRD" command was really a "resume a paused authoring session."**
  The `/prp-prd` Phase-1 triage (already-in-design-docs? already-a-plan-task?
  v1/v2/v3-deferred? contradicts-an-ADR? genuinely-undocumented?) has no branch for
  "this PRD is mid-authoring and paused on a decision." The handover + decision-support
  doc carried 100% of the state; without reading them first, the session would have
  re-run the whole scope gate the prior session already completed.
- **The user added a new decision (D6, recording optional) mid-flight**, after D5 was
  answered, with a four-word message ("IT should be optional too"). It cohered cleanly
  with the existing clean-posture discipline (`messaging_enabled`/`rtc_enabled` flags),
  so it folded into the PRD without a rewrite — but it's a reminder that "locked
  decisions" from a handover can still grow at write time.
- **`ENTRY_KIND_ROOM_RECORDING_UPLOADED` was already registered by M2 but never emitted.**
  D3 said "name 3 new consts, emit the already-registered recording kind." Verifying
  against the registry (line 243, count 69) confirmed this exactly — a pleasant case
  where the handover's claimed facts survived a file:line check rather than drifting.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a "check for an in-flight authoring handover" step to `/prp-prd` Phase 1 (glob `.claude/PRPs/handovers/*<kebab-name>*progress*.md` and `*<kebab-name>*handover*.md`; if found, route to resume-not-author and read it before the scope gate). | A future `/prp-prd <name>` on a paused PRD jumps straight to the open decision instead of re-running the scope gate the prior session finished. Saves the full Phase-1 re-derivation. | minor (one glob + a conditional in the command spec) | 1× this session + 1× in eval #1014 (same finding, recorded independently) |
| 2 | When a handover hands over a "locked decisions" table + one open decision, the resuming session should treat the locked table as *amendable at write time*, not frozen — surface any new user decision (like D6) into the Decisions Log with its own row + date, not silently fold it. | Decisions Log stays a faithful audit trail; D6 got its own row this session, but the discipline wasn't explicit anywhere. | minor (note in the handover rule or prp-prd) | 1× this session |

## What to carry forward

- **Read the handover trail before the command body when a `*-progress-*.md` or
  `*-handover-*.md` exists for the target.** This session's entire efficiency came from
  reading `m3-prd-authoring-progress-2026-06-15.md` + the decision-support doc first,
  which turned a multi-phase scope-clarification command into one `AskUserQuestion`.
- **A paused-authoring handover with a single OPEN decision + a locked-decisions table
  is the highest-leverage resume artifact in the corpus** — zero re-derivation. The
  prior session's discipline (surface the tension, don't paper over D5 to write faster)
  paid off here: D5 was a clean one-question resolve.
- **Ground PRD claims in file:line reality even when a handover asserts them.** The
  registry count (69→72), the already-registered recording kind (line 243), and the
  bridge-validates-on-Linux fact were all re-verified against the actual files, not
  copied on trust. Cheap, and it's the canonical-schema-first discipline.
- **Tombstone a resolved handover + advance the roadmap pointer as part of the same
  session.** The progress handover got a `✅ TOMBSTONE` banner and the roadmap
  `next_logical_sub_phase` moved off shipped-M2 onto M3 `/prp-plan` — so the next
  session doesn't re-enter the resolved gate or chase a stale pointer.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/prp-prd` (as resume) | 30 | 0 | medium | The command spec is fresh-PRD-shaped; resume worked only because the handover was read first. Surprise = "this isn't a fresh authoring" (see What-to-change #1). |
| Reading handover + decision-support doc first | 30 | 0 | none | Turned the full Phase-1 scope gate into one open decision. The load-bearing move of the session. |
| `AskUserQuestion` (D5, with side-by-side previews) | 5 | 0 | low | Clean one-question resolve; previews (A/B/C storage+access+framing+build matrix) made the cascade legible. |
| Grounding against registry + governance_log.rs | 4 | 0 | low | Confirmed count 69→72, recording kind pre-registered, no OQ-V2-04 anchor in 99. No drift found. |
| OQ-V2-04 resolution + changelog + umbrella row | 6 | 0 | none | Mirrored the OQ-V2-10 resolution-block format; newest-first changelog; three edits landed atomically. |
| `memory_write_eval` (#1014) | — | 0 | none | Post-task eval; already flagged the prp-prd resume-detection gap (recurrence anchor for #1). |

## Complexity scores (heavy tasks only)

No Junior impl-tasks ran this session (PRD/doc meta-work only). The complexity metric
(`<files>/<commits>/<runtime-min>/<max-log-silence-min>`) targets the Sonnet+watchdog
impl envelope and does not apply to advisor-authored spec work. Recorded as N/A by
design, not omission.

## Decisions to revisit

- Should `/prp-plan` (the next step on the M3 PRD) get the same resume-detection
  guard as proposed for `/prp-prd` (#1)? The same paused-mid-authoring failure mode
  exists for plans. Worth a one-line check when #1 is implemented — make it a shared
  helper, not two copies.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] What-to-change #1 (prp-prd resume-detection step): update `.claude/commands/prp-core/prp-prd.md` (or the skill body) Phase 1 — recurrence is 1 this session + 1 in eval #1014 (the same finding), so it meets the "≥1 here + ≥1 prior memory" bar. Concrete: add a handover-glob conditional before the triage test. **This is the highest-leverage item.**
- [ ] What-to-change #2 (locked-decisions-amendable-at-write-time): note in `.claude/rules/handover.md` or prp-prd that a resuming author surfaces new mid-write decisions into the Decisions Log with their own dated row — single occurrence, recorded not auto-promoted.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase section omitted — no
`/auto-phase` invocation and no auto-state mutation this session (Step 0.5
trigger did not fire)._
