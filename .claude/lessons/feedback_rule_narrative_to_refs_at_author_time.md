---
name: Author rule narratives in refs/ from the start, not inline — growth-discipline beats one-time trims
description: Feedback rule — the durable lever on always-load context budget is preventing rule-file bloat at author time (incident narratives + FP/FN taxonomies go to .claude/refs/ with a one-line rule pointer), because one-time extraction of an already-trimmed dual-harness corpus yields only ~1%
type: feedback
originSessionId: ad2ab1cf-98b9-486f-93f4-f7b97794bd4d
---
The always-load rules corpus is the dominant session-start context cost (~70.8k tokens ≈ 35% of
the ~200K effective working window, measured live 2026-05-29). The instinct when it grows is to
run a one-time extraction pass. The 2026-05-29 harness-audit proved that instinct yields little:
the corpus was already heavily refs-externalized (6–7 `refs/` pointers in the top-2 files) AND
dual-harness shared (`.pi/` cites the top rule files 68× across 18 files, mostly by named section
anchor), so conservative-depth extraction recovered only **~1,800 tokens (~0.9%)**. Most of the
bulk is load-bearing schema / mechanism / Pi-cited content that cannot move.

**Why:** rule files accrete *narrative* over time — each incident adds a post-mortem paragraph
(the cf93b7ba6 lane-collision story), each hook adds a false-positive/false-negative taxonomy,
each locked decision adds a "why we chose option (b)" block. These are valuable but they are the
**why**, not the **what**. Inlined, they 1.6× the always-load cost of every session forever. The
rule statement itself (the **what** the model must obey) is usually 1–2 sentences; the narrative
is 3–10× longer. A corpus that started lean becomes a 20k-token file one incident at a time, and
by the time anyone audits it, the only safe cuts are small because the headings are now load-bearing
(cited by `.pi/`, by skills, by briefs).

**How to apply:**
- **At author time**, when adding to a `.claude/rules/*.md` file: put the *rule statement* inline
  (terse imperative + the trigger/condition). Put the *incident narrative, FP/FN taxonomy, locked-
  decision rationale, or worked example* in `.claude/refs/<rule-name>-incidents.md` (or an existing
  refs file) with a one-line pointer: `Rationale + <incident> detail: .claude/refs/<file>.md §"<anchor>"`.
- **Section headings stay in the rule file** — `.pi/` and skills cite them by name; moving a heading
  breaks the dual-harness contract silently. Only the explanatory prose *inside* a section moves.
- **The canonical-schema-first gate** (advisor-orchestrator.md §3.6) already requires reading 1–2
  sibling rule files before authoring a new `.claude/rules` file — extend that habit to *edits*: if
  the addition is >3 lines of narrative, author it in refs/ from the start.
- **Don't run speculative one-time extraction passes** on an already-trimmed corpus expecting big
  wins. Audit first (`harness-audit`); if the corpus is already refs-externalized, the ROI is ~1%
  and growth-discipline is the better investment.

**Calibration note for harness-audit:** the chars/4 token estimate under-counts markdown by ~1.6×
(live `/context`: advisor-orchestrator 47,413 chars = 19.8k tokens). Use **~2.4× chars→tokens** for
markdown. And the scoring-matrix redundancy term over-counts when fed a shared-keyword grep —
"SessionStart" matched 4 distinct invariants in pmd-invariants/branch-manager/multi-lane/session-
awareness that share a word but not prose; a high redundancy score needs execute-pass prose-diff
confirmation before acting. Companion: [[feedback_context_trim_verify_empirically]],
[[feedback_effective_budget_200k_not_1m]], [[feedback_read_canonical_before_writing_spec]].
