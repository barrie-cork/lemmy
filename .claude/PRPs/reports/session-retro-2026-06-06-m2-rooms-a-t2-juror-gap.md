# Session retro — 2026-06-06 — m2-rooms-a-t2-juror-gap

**Harness:** claude-code
**Session window:** 2026-06-06 (~resume from `/clear`) → 2026-06-06 close (~90 min wall-clock, mostly read/analysis)
**Branch at start:** `0aafcc00f` (`governance-v0`)
**Branch at end:** `17963af56` (`governance-v0`)
**Files touched:** 3 (decision-queue.json, 1 new lesson, 1 new handover)
**Commits:** 4 (auto: 0, explicit: 4)

## TL;DR

Resumed m2-rooms-a to author the T2 impl-task brief (bridge `room_provisioner.rs`,
C2.1 jury path). The verify checks passed and the brief looked routine — but
reading the MIRROR refs surfaced a real plan gap: plan §13 T2 requires the
bridge to invite "the 5 assigned jurors as `Juror-<suffix>`", yet the
`CaseTransitionEvent` the bridge receives carries no juror identities and the
bridge has no DB access. The headline acceptance criterion is unimplementable
as scoped. Rather than silently expand T2 across the bridge-only/workspace
toolchain boundary the planner drew, I surfaced it to the planner as DQ
`a3d0e9941441-055` (per user direction). The most load-bearing finding —
promoted to a lesson — is that I proposed a fix ("augment the event") *before*
tracing the data-flow precedent that would both confirm the gap and dictate the
fix shape; only a user nudge ("look again") triggered the precedent trace. The
top change: trace the consumer's existing data-flow before proposing how to
supply missing data.

---

## What surprised us

- **A "one-liner" brief-authoring task uncovered an unimplementable plan task.**
  The handover framed T2 as "next concrete action: author the brief." The plan
  scored 8/10 confidence, all 4 clarify-DQs were resolved, and §3.4 DoD smoke +
  §3.5 watchpoint gates had passed at plan-approval. None of that caught that the
  bridge has no way to learn *which* jurors to invite. The gap was invisible
  until someone read `CaseTransitionEvent`'s actual fields against T2 step (d)'s
  requirement. Plan-approval gates validate command-executability and watchpoint
  specificity — they do **not** trace consumer-side data availability.

- **I reached for a solution before grounding it.** My first instinct was
  "augment `CaseTransitionEvent` with `juror_pseudonyms`" — which turned out to
  be *correct*, but I asserted it before checking the M1 `relay.rs` precedent. It
  took the user's "have a look at this again" to make me trace how the bridge
  already learns governance identities (binary pushes `brehon_sender`/
  `brehon_recipient`; bridge never GETs back). That trace confirmed both the gap
  AND that "augment the payload" (not "add a GET-back route") was the
  precedent-consistent shape. The right answer was reachable in one step; I took
  a detour.

- **The lesson file shipped without frontmatter and the PMD sync silently
  skipped it** (`WARN: could not parse frontmatter`, `imported: 0`). The sync
  script expects flat `name`/`description`/`type` keys; I'd internalised the
  MEMORY.md auto-memory format (nested `metadata:` block) which is a *different*
  store. Caught only because I read the sync output instead of trusting the
  exit-0.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | New lesson `feedback_trace_data_flow_precedent_before_proposing_seam.md` (already written this session): when a plan step assumes a consumer has data it doesn't receive, trace how that consumer gets its OTHER data BEFORE proposing a supply mechanism. | Avoids proposing precedent-violating seams; turns guesses into grounded recommendations citing file:line. | minor (lesson authored) | 1× this session + adjacent to `feedback_falsifiable_hypothesis_before_structural_fix` (verify-before-fix family) → meets ≥1-here-+-prior threshold |
| 2 | Add a **consumer-data-availability check** to the clarify gate (`.claude/commands/brehon-clarify.md`) or the planning subagent contract: for any plan task where a cross-process consumer (bridge, external service) must act on entity data, the clarify pass must confirm the data reaches the consumer (in its event payload OR via a named fetch route). | Catches the T2-class gap at plan-authoring time, not brief-authoring time — saves a full planner round-trip per occurrence. | medium (edit clarify command + planning agent §) | 1× this session; watch for 2nd before promoting to a hard gate |
| 3 | When authoring a `feedback_*.md`/`reference_*.md` lesson, use the flat `name`/`description`/`type` frontmatter (NOT the MEMORY.md nested `metadata:` block). Confirm via `head -6` of an existing imported lesson before writing. The `.claude/hooks/lesson-pmd-sync.sh` PostToolUse hook (HTTP topology) is the modern path; `sync-lessons-to-pmd.sh` is the fallback — either way, no-frontmatter = silent skip. | Lesson is queryable in PMD on first write; no skipped-import + amend cycle (cost this session: 1 extra commit + 1 re-sync). | minor | 1× this session; recorded, below promotion threshold |

## What to carry forward

- **Read the MIRROR refs before writing the brief, not after.** The gap was only
  visible because I read `appservice.rs`, `provision.rs`, `puppet.rs`,
  `relay.rs`, and the `CaseTransitionEvent` definition before drafting. A
  brief authored from the plan text alone would have shipped the gap to an
  impl-task worker.
- **Surface scope-crossing decisions to the planner; don't decide them in a
  brief.** The user's "Surface to planner first" choice was the right call — the
  bridge-only/workspace toolchain split (plan §5.2, R8) is a deliberate boundary,
  and a brief that quietly added two workspace files to a bridge-only task would
  have been a silent scope override. Mandatory-user-gate discipline (judgment-
  heavy DQ) worked.
- **Atomic read-mutate-commit for governance-v0 DQ writes held cleanly.** Fetch →
  read fresh → append via `dq-v3-append-fragment.sh` → verify in pending → stage
  → commit → push → verify in committed HEAD. Per multi-lane Hard refusal #6. No
  race, no clobber, entry survived.
- **Read background-task output instead of trusting the summary.** The PMD-sync
  "exit code 0" notification was true but masked a skipped import; reading the
  actual output (`imported: 0 ... errors: N`) caught it. Per
  `pattern_verify_before_trusting_shell_output`.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Handover-resume (4 ASSUMES/VERIFY checks) | 15 | 0 | none | Clean cold-resume; all 4 checks passed in parallel in ~1 min. The VERIFIED_AT + verify-table discipline paid off. |
| MIRROR-ref read before brief (manual) | 60+ | 0 | high | The single highest-leverage action: caught an unimplementable plan task that 3 plan-approval gates missed. "Saved" = the full impl-task cycle (dispatch + worker + validate + catch-fire) that would have burned on an unbuildable brief. |
| AskUserQuestion (juror-sourcing fork) | 5 | 0 | none | Clean 3-option fork; user chose "surface to planner." No false-positive gate. |
| Initial "augment the event" proposal (pre-grounding) | — | 10 | medium | Proposed before tracing the precedent; user "look again" nudge cost a round-trip. Root cause of the promoted lesson. |
| DQ atomic write (`dq-v3-append-fragment.sh` + protocol) | 8 | 0 | none | Helper + protocol worked; post-push verify confirmed survival. |
| Telegram hook recreate (`remove_hook` + `create_hook`) | 0 | 5 | low | Recreation failed ("Junior MCP not configured"); known daemon dep issue. Time spent confirming it's broken, not fixable from here. |
| Lesson PMD sync (`sync-lessons-to-pmd.sh`) | 0 | 8 | medium | First run silently skipped my lesson (no frontmatter); needed frontmatter fix + re-sync + a direct `memory_search` to confirm. |
| post-task-retro (eval 843) | 5 | 2 | low | `memory_write_eval` first call failed on missing `skill_or_tool` required field; one ToolSearch + retry. |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. No Junior impl-tasks
ran this session (all work was advisor-side, manual). No task exceeded the
>55min-runtime / >40min-silence / >8-files thresholds. The only multi-file
activity was advisor-authored docs (3 files across 4 commits) — well within
envelope; complexity metric (designed for Junior watchdog risk) is N/A for
foreground advisor work.

## Decisions to revisit

- **DQ `a3d0e9941441-055` outcome shapes T2's whole shape.** If the planner picks
  option-a (augment `CaseTransitionEvent`), T2's validation flips to
  `--workspace --features full` and the §2.4 file-class lesson injection must
  re-run against the new workspace files. Verify the *plan* was amended, not just
  the DQ answered.
- **Change #2 (consumer-data-availability clarify check) needs a 2nd occurrence
  before promotion to a hard gate.** Recorded here; if a similar consumer-side
  data gap recurs in m2-late or M3, promote it.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [x] Change #1: `feedback_trace_data_flow_precedent_before_proposing_seam.md` — **already promoted this session** (committed `45506e177`, live in PMD as memory ID 844). Adjacent to the verify-before-fix lesson family.
- [ ] Change #2: add consumer-data-availability check to `.claude/commands/brehon-clarify.md` + planning subagent contract — **hold for 2nd occurrence** before promoting to a hard gate.
- [ ] Change #3: lesson-frontmatter-shape note — below promotion threshold (1×); recorded in this retro's "What to change" #3 only.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
