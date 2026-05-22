# Session retro — 2026-05-22 — context-prune-option-a

**Harness:** claude-code
**Session window:** 2026-05-22 ~18:30 → ~20:30 BST (~2 h wall-clock, mostly interactive)
**Branch at start:** `3d9402804` (`governance-v0`)
**Branch at end:** `af9b268df` (`governance-v0`)
**Files touched:** 3 (`MEMORY.md`, `.claude/rules/decision-queue.md`, `.claude/PRPs/handovers/context-prune-option-b-2026-05-22.md`)
**Commits:** 2 explicit (`cdff6392e` Schema compression, `af9b268df` handover)

## TL;DR

User invoked `/memory-prune`, then escalated to a broader context-injection audit after `/context` showed 65k tokens in Memory files (32.6% of session context). I produced a three-option pruning plan (A/B/C with risk + estimated savings + verification gates), user picked Option A, we landed ~700 tokens of savings (MEMORY.md duplicates + `decision-queue.md` §Schema v2 compression), and authored a durable handover for Option B's ~13k-token rule-relocation work to execute in next session. The load-bearing finding: **two of my proposed Option A rule cuts (G4 example externalization, DQ "How to write a question" externalization) were correctly self-rejected on second-look during execution** — proposal-pass analysis under-tested the cuts' load-bearingness. Top change proposal: bake a "rule-side cut requires a load-bearing dry-run before proposal" gate into the memory-prune skill, mirroring the `feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md` pattern.

---

## What surprised us

- **Auto-mode classifier conservatively denied a legitimate prose compression** as "destructive overwrite" on `.claude/rules/decision-queue.md` §Schema v2. The classifier read `Edit`'s `old_string` → `new_string` swap (which only touches the matched range) as a wider delete than its intent allowed. User explicit approval cleared it. Surfaces a broader point: the classifier's tooling-semantic model and the user's edit-intent model diverge on long `old_string` patterns — long matched text reads as "deleting much" even when semantic content is preserved.
- **Two proposed cuts didn't survive execution inspection.** `advisor-orchestrator.md` §G4 example block: on closer read, the verbatim-blockquote example IS the anti-paraphrase gate — externalizing it defeats the gate's purpose. `decision-queue.md` §"How to write a question": Junior workers (impl/bm) read it at task dispatch, so externalizing pushes cost to every Junior boot. Both were proposal-pass misjudgments. Net: actual Option A savings ~700 tokens vs projected 3.5–4k.
- **Schema v2 compression landed at ~300 tokens, not ~700.** The diff was -17/+12 lines and most of the "removed" content was prose-tightening (compressing two paragraphs of v2 history into one paragraph of current-state schema). Token-equivalent prose at smaller line-count.
- **A side-channel commit attributed to "Claude Opus 4.6" landed during the session.** `cdff6392e` (the Schema v2 compression I edited) shows up in `git log` attributed to a parallel-session Opus author identity, not my Sonnet author trailer. Either a Stop hook auto-committed under a different model identity, OR a parallel session is operating in this checkout. Already covered as a class by `feedback_cross_session_commit_attribution_collision.md` (2026-05-22); this is occurrence #2 in 24 hours — worth a watch.
- **/context revealed the dominant cost is rule files, not MEMORY.md.** `advisor-orchestrator.md` alone is 17k tokens — 2.3× the size of MEMORY.md. The user's intuition ("prune memory") naturally points at the most-visible file (MEMORY.md / "auto-memory"); the actual leverage lives in the rules directory. Option A surfaced this gap.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Add "load-bearing dry-run" step to `.claude/skills/memory-prune/SKILL.md` before proposing any rule-side cut.** For each proposed externalization of a rule section: (a) grep for the section's content across `.claude/skills/`, `.claude/agents/`, `.claude/commands/`, `~/.claude/commands/` to confirm no subagent / skill body / hook depends on the inline form; (b) check whether the section IS the mechanism (e.g. anti-paraphrase gate must remain inline). If either check fails, drop the cut from the proposal. Mirrors `feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md` (30-sec check vs. lost proposal time + reduced trust in skill outputs). | Prevents the proposal-pass-vs-execute-pass gap that cost ~8 min this session; future memory-prune proposals carry higher first-pass accuracy. | minor (~10 line skill edit) | 2× this session (G4 example + DQ how-to-write), 0× prior in this skill |
| 2 | **When proposing rule-side cuts in a multi-option plan, mark each cut with a "verified-load-bearing" tag explicitly.** In the three-option plan output, every rule cut MUST carry one of: `verified-pure-prose` / `verified-historical-only` / `unverified-load-bearing-risk`. Cuts in the third tag bucket are inadmissible without the §1 dry-run above. Forces the proposal pass to declare its certainty per cut. | Surfaces uncertainty to user at decision time instead of hiding it. Aligns with `feedback_principles_not_rules.md` — single-instance proposals tagged honestly, not promoted. | minor (skill template edit) | 1× this session + paired with §1 above |
| 3 | **Add a `.claude/refs/` sentinel-probe runbook to the memory-prune skill** referencing `feedback_context_trim_verify_empirically.md` (PMD #147). The handover authored this session already documents the procedure; lift it into the skill body so any future context-trim work picks it up by default. Empirically-verify-before-design is the explicit lesson; baking it into the skill removes the need to re-derive. | Future context-trim sessions skip directly to verification before designing scope. ~5 min saved per session. | minor (~15 line skill addition) | 1× this session + 1× prior (Phase A archive-trim incident, per PMD #147) — meets ≥1+1 threshold |
| 4 | **Mark `cdff6392e` for cross-session attribution audit in next session.** Confirm whether it was a Stop-hook auto-commit (acceptable) or a parallel session running in the canonical checkout (the failure-mode class `feedback_cross_session_commit_attribution_collision.md` documents). If parallel session: surface to user; if Stop hook: document the hook's commit-author identity in the hook source so it stops looking like a phantom advisor. | Clarifies whether the cross-session race documented 2026-05-22 has re-fired (occurrence #2) or whether a legitimate auto-commit pathway exists that the lesson didn't account for. | minor (audit only) | 1× this session + 1× prior 2026-05-22 |

## What to carry forward

- **Three-option plan structure (A/B/C with risk + savings + verification gates) for ad-hoc audit work.** Used this session for context-injection prune; user picked Option A directly without back-and-forth. The structure makes risk-tolerance the user's lever explicitly, separates "what gets done" from "how aggressive." Re-use shape for: future MEMORY.md prune sessions, runbook audits, skill consolidations, rule-corpus restructures.
- **Mandatory handover file at session-end for plans that span session boundaries.** The Option B handover (`.claude/PRPs/handovers/context-prune-option-b-2026-05-22.md`, 258 lines) is self-contained per `advisor-orchestrator.md` §1 — next session reads it cold without conversation context. Committed + pushed to `governance-v0`. This is the canonical pattern for context-pressure work; do it every time.
- **PMD search before proposing context-trim cuts.** `memory_search_hybrid` returned PMD #147 immediately when queried for "redundant with rules advisor session context bloat" — that's the single most load-bearing constraint on this whole class of work (empirically verify before trimming). The 8 min spent on PMD search saved ~30 min of misjudged trim work.
- **User-explicit-approval pathway for auto-mode classifier denials.** When the classifier denied the Schema v2 compression, I correctly stopped, reported the reduced savings, and waited. User explicit "approve the Schema v2 compression" cleared the denial cleanly. Mechanism works; carry forward.
- **`feedback_context_trim_verify_empirically.md` (PMD #147) sentinel-probe pattern** as the canonical mechanism for any context-trim work that depends on auto-load behavior. Codified in the Option B handover Step 1 as a hard refusal if not run first.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/memory-prune` (initial pass) | 5 | 0 | none | First-pass found 6 clear candidates (3 stale closed states, 3 rule-redundant); all approved by user |
| `memory_search_hybrid` for PMD strategies | 8 | 2 | medium | Returned PMD #147 + retro #451; 2 min on tangential results. Net huge positive — PMD #147 is the load-bearing constraint for the whole session |
| `/context` measurement | 3 | 0 | high | Revealed advisor-orchestrator.md (17k) dwarfs MEMORY.md (7.5k); reframed the prune work entirely |
| Three-option plan authoring | 12 | 5 | high | Plan landed cleanly with user; 5 min wasted on the rule-side proposal-pass cuts that didn't survive execution |
| Option A rule-edit attempts (G4 + DQ how-to-write) | 0 | 8 | medium | Both correctly self-rejected during edit. The wasted time IS the lesson — proposal-pass under-tested load-bearingness |
| Auto-mode classifier denial event | 0 | 3 | medium | Classifier was conservative-correct under its surface signal. Denial was legitimate; my proposal language ("Edit deletes large authoritative portions") triggered the guard appropriately |
| Schema v2 compression (post-approval) | 5 | 0 | low | Landed cleanly. ~300 tokens saved, lower than my ~700 estimate but real |
| Handover file authoring | 15 | 0 | none | Self-contained, 6-step plan, hard refusals captured, expected savings tabulated. Future session executes without me |

**Net session arithmetic:** ~48 min saved against equivalent-from-scratch / ~18 min wasted on proposal-pass misjudgments and one classifier denial cycle. Net positive ~30 min, plus durable Option B handover that captures ~14.8k tokens of next-session savings work.

## Complexity scores (heavy tasks only)

None of this session's work was complexity-class (no multi-file impl, no Junior dispatch, no multi-hour wall-clock task). The heaviest single task was the handover file authoring: `1/1/12/0` (1 file, 1 commit, ~12 min runtime, 0 log-silence — interactive throughout). All others were sub-5-min interactive edits.

## Decisions to revisit

- **Should the memory-prune skill's load-bearing dry-run step be backed by a script?** §1 in "What to change" proposes a manual grep procedure. A `.claude/skills/memory-prune/scripts/check-load-bearing.sh` that takes a rule file + section header and reports cross-references would be a natural evolution. Defer to first re-use — if §1 fires again next session and the manual procedure friction warrants it, build the script.
- **Auto-mode classifier divergence on Edit semantics.** The denial pattern this session (long `old_string` matched text reading as "deleting much") may repeat on other legitimate Edit calls. Worth a watch: if it fires ≥2 more times across sessions, surface a calibration request to the classifier-tuning team / consider splitting long replacements into multiple smaller Edits.
- **The `cdff6392e` cross-session commit attribution.** See §4 in "What to change" — needs audit next session to determine whether it's a Stop-hook auto-commit (benign) or a parallel-session race (occurrence #2 of the documented failure mode).

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [x] Change #1 (load-bearing dry-run step for memory-prune skill): **SHIPPED `563725ab6`** as Step 3.5.a in `.claude/skills/memory-prune/SKILL.md`. Recurrence: 2× this session (G4 + DQ how-to-write), 0× prior in this skill but conceptual sibling exists in `feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md`.
- [x] Change #2 (verification tag per surviving cut): **SHIPPED `563725ab6`** as Step 3.5.b. Three tags: `verified-pure-prose` / `verified-historical-only` / `unverified-load-bearing-risk` — the last is inadmissible without §3.5.c.
- [x] Change #3 (`.claude/refs/` sentinel-probe in memory-prune skill): **SHIPPED `563725ab6`** as Step 3.5.c. Mandatory before any rule relocation cut per `feedback_context_trim_verify_empirically.md` (PMD #147).
- [x] PMD eval written: **memory #487**, embedding backfilled (canonical PMD, 0 missing).
- [ ] Change #4 (audit `cdff6392e` cross-session attribution): one-off audit task for next session, not a lesson promotion. Track only.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Step 0.5 auto-phase trigger:
**did not fire** (no `/auto-phase` invocation, no auto-state JSON mutation
this session). Step 5 (PMD eval): pending user authorisation per
promotion-candidate checkbox above._
