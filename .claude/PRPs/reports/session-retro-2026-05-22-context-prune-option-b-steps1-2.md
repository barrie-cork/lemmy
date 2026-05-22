# Session retro — 2026-05-22 — context-prune Option B Steps 1+2

**Harness:** claude-code
**Session window:** ~2026-05-22T20:30Z → ~2026-05-22T21:30Z (~60 min)
**Branch at start:** `84a2698f3` (`governance-v0`)
**Branch at end:** `e89ac013a` (`governance-v0`)
**Files touched:** 10 (this session's 3 commits + 1 handover write)
**Commits:** 4 (auto: 0, explicit: 4) — `a73aa7ddd`, `f4d584345`, `17a3eb1bb`, `e89ac013a`

## TL;DR

Executed Steps 1+2 of the context-prune Option B handover (sentinel
probe → relocate `auto-phase.md` + `auto-roadmap.md` from `.claude/rules/`
to `.claude/refs/`). The headline finding: the handover's plan
contradicted the rule files' OWN self-declarations that they must be
auto-loaded — surfaced as an AskUserQuestion fork before the move, and
the user picked "harden skill bodies first" to preserve the invariant
in behavior (Phase 0 Step 0 Read) before mutating the file location.
The retro-worthy generalisable lesson: **a handover's plan is a
hypothesis, not a contract — load-bearing self-declarations in target
files must be checked before executing the move.**

---

## What surprised us

- **The rules' own preambles declared "read at session start" as a
  load-bearing invariant** — and the handover's Step 2 (move the
  rules) directly contradicted that self-declaration. Neither
  handover-author session (which shipped Option A) nor the precondition
  check in Step 1's sentinel probe caught this. Caught only because I
  re-read the rule preambles before doing `git mv`. Without that
  check, the move would have shipped with stale self-claims AND
  broken the resume-time guarantee that the rule loads automatically
  for `/auto-phase` invocations triggered from auto-state JSON
  resume paths.
- **The handover named `~/.claude/commands/auto-roadmap.md` as a file
  to edit — but that file doesn't exist.** Only project-scope
  `.claude/commands/auto-roadmap.md` exists. The user-scope and
  project-scope `roadmap-next.md` files are also DIFFERENT files
  (not symlinks; `diff -u` shows materially different content). The
  handover's path list was approximate; the actual live read paths
  required investigation.
- **`Edit` requires `Read` before `Edit` even when the file was just
  created via `git mv`** — git mv preserved content, but the harness
  considers the new path "unread" until I `Read` it explicitly. Cost:
  ~3 wasted tool calls before I noticed the pattern.
- **The sentinel-probe pattern worked exactly as `feedback_context_trim_verify_empirically.md`
  prescribed.** ~5 min round trip (commit → user opens fresh CC →
  reports `/context`) for a binary PASS/FAIL on a load-behavior
  claim that documentation could not certify. The user's `/context`
  output showed `64.9k tokens (32.5%) Memory files` baseline with
  `auto-phase.md` (6k) and `auto-roadmap.md` (6.5k) BOTH listed —
  confirming the predicted savings ceiling, AND confirming the
  sentinel file in `.claude/refs/` was NOT loaded.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When a handover or plan proposes RELOCATING a load-bearing file (rule, lesson, template, skill), **always Read the file's own preamble first** for self-declarations of loading discipline ("read at session start", "always-loaded", "auto-loaded", etc.). If present, the move must either (a) preserve the loading invariant via a hardening step (e.g. lazy-load Read in the consuming skill) before the file mutates, or (b) explicitly update the self-declaration as part of the same commit. Add to `feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md` as a sub-pattern under "preconditions to check." | Prevents stale self-claims + broken resume-time invariants when relocating context-budget items. Single occurrence this session, but the pattern generalises directly to any future `.claude/refs/`-style relocation in remaining Option B Steps 5+6 (advisor-orchestrator §3.7/§3.8/§6 externalizations) and beyond. | minor (one-line addition to existing lesson) | 1× this session + matches the generalisation in `feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md` (2026-05-22 same-day lesson) |
| 2 | When executing a multi-step handover authored by a prior session, **inventory the file list against the actual filesystem before trusting the path list.** Specifically: `Glob` or `ls` every named path before any mutation; surface any path that doesn't exist (was the handover author working from a stale mental model? Is the path a typo? Was the file renamed since?). Adopt as an unconditional first action in any handover-execution flow. | Catches handover-author drift early (e.g. this session: `~/.claude/commands/auto-roadmap.md` doesn't exist, was likely typo'd from project-scope `.claude/commands/auto-roadmap.md`). Saves ~5 min wasted exploration per occurrence. | minor (1-2 tool calls) | 1× this session, would have surfaced ~3 prior incidents in PMD if applied earlier |
| 3 | The Step 2 commit had to update **subtle behavioral text** (user-scope `auto-phase.md` §6 "Mid-phase skill-edit deferred-effect awareness") because moving the rule out of session-start auto-load **changed the meaning** of "rule files are sticky for the session." The retro reveals: **when relocating a load-bearing file, audit any documentation in consuming files that asserts loading semantics**, not just citation paths. Add as a checklist item to the relocation-planning rubric in `feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md`. | Catches stale semantics-claims in skill bodies / rules / lessons that would otherwise documentation-rot silently after the move. | minor | 1× this session, anticipatory generalisation |

## What to carry forward

- **Read the rule's own preamble before moving the rule.** When the
  rule self-declares "loaded at session start," do not relocate
  without (a) hardening the consumer's load path OR (b) updating the
  self-declaration. This session's AskUserQuestion fork
  ("Do Step 2 + harden skill bodies first") was the right call;
  it took 2 extra commits but preserved the invariant.
- **The sentinel-probe pattern works.** ~5 min round trip via a
  fresh-session `/context` measurement for binary verification of a
  load-behavior claim. Use it for every future `.claude/refs/`-style
  relocation in Steps 5+6 (advisor-orchestrator §3.7/§3.8/§6).
- **Handovers as hypotheses.** This session's handover was good but
  not contract-grade — the path list had at least one bad path and
  the plan didn't account for the rules' self-declarations. The
  resume protocol I authored (`context-prune-option-b-step3-onward-2026-05-22b.md`)
  attempts to do better by recording: lanes status, what changed
  behaviorally, what didn't, predicted `/context` state for verification.
- **AskUserQuestion at structural-decision points works cleanly.**
  Three-option fork with previews ("Skip / Do anyway / Do + harden")
  let the user pick the right path in one turn. No back-and-forth.
  Cost: ~30 seconds; saved: a ~12.5k-token broken-invariant landing.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`. Numbers
are defensible from the transcript.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Reading rule preambles before `git mv` | 30+ | 0 | high | Would have shipped a broken auto-load invariant otherwise; the rules self-declared "read at session start" and the handover plan defeated that |
| AskUserQuestion (3-option fork on Step 2 conflict) | 10 | 0 | none | One clean decision; the user-relay was the right turn vs guessing |
| Sentinel-probe pattern (`feedback_context_trim_verify_empirically`) | 0 | 5 | low | Did exactly what the lesson prescribed; the wasted 5 min is the round-trip cost (commit → fresh session → /context → return), which is irreducible for empirical verification |
| `grep -rn "\.claude/rules/auto-(phase|roadmap)\.md"` cross-ref scan | 5 | 0 | low | One sweep caught all 6 stale citations across `bm-merge.md`, `open-roadmap.md`, `advisor-orchestrator.md`, `CLAUDE.md`, and the user-scope skills |
| `git mv` for rename-tracking | 2 | 0 | none | Git correctly tracked as rename; commit diff stat showed `.claude/{rules => refs}/` |
| Read-before-Edit forced rhythm | 0 | 3 | medium | Surprised that `Edit` blocked on `git mv`-relocated files until `Read` ran; cost ~3 wasted tool calls |
| Pre-existing untracked debug JSONs (`.claude/PRPs/debug/dq-frag-clarify-*.json`) | 0 | 0 | none | Out of scope; not touched, surfaced once for awareness |
| Handover author (this session, for next) | — | — | — | Output: 287-line self-contained brief at `.claude/PRPs/handovers/context-prune-option-b-step3-onward-2026-05-22b.md` |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. This
session had one "heavy" task — Step 2 file shuffling — but it was
not a Junior task and has no log silence (interactive session, ~45
min wall-clock, multi-file Edit chain).

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---:|---:|---:|---:|---:|
| Step 1 sentinel probe (commit + verify + cleanup) | 1 (created+deleted) | 2 | ~10 (incl. fresh-session round trip) | n/a |
| Step 2 relocate + harden + cross-ref update | 10 (4 skill bodies + 2 rule files moved + 4 live cross-refs) | 1 | ~30 | n/a |
| Handover author for Steps 3-6 | 1 created (287 lines) | 1 | ~10 | n/a |

No task hit the >55min runtime / >40min log silence / >8 files
thresholds; one (Step 2) hit 10 files but that count includes the 4
"hardening" Step 0 Reads + 2 file moves + 4 cross-ref updates — all
mechanical, none deep. Not a planning bundling-drift signal; the work
is naturally cross-cutting.

## Decisions to revisit

- **Steps 3-6 verification cadence.** Steps 3+4 are MEMORY.md
  prose-only edits — the handover suggests they may skip the
  fresh-session `/context` verification. Should they? Or should EVERY
  Option B step verify? Cost of always-verify: ~5 min per step ×
  4 steps = ~20 min. Cost of skipping: false-confidence in MEMORY.md
  trim effect.
- **The relocated rules' loading semantics change is a behavior delta
  worth a PMD memory.** When a meta-editor session edits a rule
  file at `.claude/refs/`, the next `/auto-phase` tick now picks
  up the edit (vs the old "wait for session restart" behavior).
  This is FASTER but means mid-flight rule edits can surface
  mid-phase. Worth a one-line `project_` memory entry so future
  sessions know.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

| # | Change | Status | Promote-to | User OK? |
|---|---|---|---|---|
| 1 | "Read the rule's own preamble before relocating" — extend `feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md` with this sub-pattern | candidate | augment existing lesson | [ ] |
| 2 | "Inventory file list against filesystem before trusting a handover's path list" — same lesson | candidate | augment existing lesson | [ ] |

Single-occurrence noise (recorded but NOT proposed):

- `Read-before-Edit forced rhythm` surprise — known harness behavior,
  already documented elsewhere; not a recurrence pattern.
- The user-scope `~/.claude/commands/auto-roadmap.md` non-existence
  is a one-off typo in the handover; not a recurrence pattern.

---

## Cross-references

- `.claude/PRPs/handovers/context-prune-option-b-2026-05-22.md` —
  original handover (Option B Steps 1-6)
- `.claude/PRPs/handovers/context-prune-option-b-step3-onward-2026-05-22b.md` —
  this session's output: resume brief for Steps 3-6
- `.claude/lessons/feedback_context_trim_verify_empirically.md` —
  PMD #147, the empirical-verification pattern Step 1 honored
- `.claude/lessons/feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md` —
  the lesson candidates 1+2 would extend
- `17a3eb1bb` — Step 2 ship commit
- `f4d584345` — Step 1 sentinel-probe PASS verdict commit
