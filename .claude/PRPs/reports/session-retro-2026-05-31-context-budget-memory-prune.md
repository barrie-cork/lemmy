# Session retro — 2026-05-31 — context-budget-memory-prune

**Harness:** claude-code
**Session window:** 2026-05-31 (single session, ~30 min)
**Branch at start:** `f40dca07f` (`governance-v0`)
**Branch at end:** `f40dca07f` (`governance-v0`) — no repo commits this session
**Files touched:** 1 (MEMORY.md user-scope only)
**Commits:** 0 repo commits; 1 MEMORY.md edit applied directly

## TL;DR

Session investigated why post-compaction context starts at 58%. Found the framing was correct (compact summary weight, not memory flooding) but the MEMORY.md byte budget was over-limit (25,362 bytes vs 24,400 ceiling). The most load-bearing finding: the initial analysis misclassified `governance-log-entry-kind-registry.md` as always-loaded (it has `paths:` frontmatter — SCOPED). PMD retrieval immediately surfaced the prior incident (PMD id=252) that documented this exact trap. Session closed with MEMORY.md pruned to 22,784 bytes and a clear decision tree for context management.

---

## What surprised us

- **`governance-log-entry-kind-registry.md` misclassified again.** First analysis reported it as an always-load 27 KB problem. PMD search immediately returned the lesson documenting the prior identical misclassification (2026-05-09, task retro id=226, lesson `feedback_subagent_frontmatter_detection.md`). The lesson existed, was in PMD, and was returned correctly — the failure was in the manual analysis step that didn't check frontmatter before claiming the file was always-loaded. This is a 2× recurrence of the same advisor-side miss.

- **True always-load baseline is ~39K tokens (~19% of 200K), down from the 2026-05-29 measurement of ~59K.** The prior trim work reduced the corpus by more than the MEMORY.md monitoring suggested. Fresh measurement (now accurate post-PMD recovery) shows headroom is healthier than expected.

- **Post-compaction 58% is compact-summary weight, not auto-load.** The user's concern was well-founded (58% is high) but the root cause is the compaction summary itself, not memory growth. The `compact-phase.md` instructions are correct; the question is whether the model follows them strictly in a given session.

- **MEMORY.md over-limit was entirely concentrated in 3 verbose Watch-section lines.** The byte overage (962 bytes) was recoverable by shortening 3 lines and moving 4 stale workflow entries to Historical — no structural pruning needed.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a **frontmatter-check step to the advisor's manual harness analysis** pattern: before citing any `.claude/rules/*.md` file as always-loaded, read its first line — if `---`, it's SCOPED | Prevents the governance-log-registry misclassification from firing a third time; same fix as lesson `feedback_subagent_frontmatter_detection.md` but for *advisor-inline* analysis, not just subagents | minor (one-line reminder in the analysis mental model) | 2× (2026-05-09 + this session) |
| 2 | Extend `compact-phase.md` to add an explicit **token-budget ceiling instruction**: "if the resulting summary would exceed ~5,000 tokens, compress further — omit all tool output, all exploration, all git diffs; retain only active thread + task list + user messages" | Post-compaction start context stays reliably low; current instruction says what to omit but doesn't set a hard ceiling | minor (1–2 lines added to compact-phase.md) | 1× this session (observed 58% post-compact) |
| 3 | Add **MEMORY.md byte-budget check to session-start ritual** (or a simple pre-session check): `wc -c ~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/MEMORY.md` — if >24000, run `/memory-prune` before starting any advisory work | Prevents the silent truncation from accumulating; the current warning fires at session start but there's no enforcement to act on it before the session proceeds | minor (habit, not tooling change) | 1× this session |

## What to carry forward

- **PMD hybrid search is the right first move for any harness-analysis question.** This session the PMD immediately surfaced the prior frontmatter-detection incident (id=252) and the prior trim retro (id=226) — both directly corrected the initial analysis. The 30-second PMD search saved ~10 min of re-investigation.
- **`/memory-prune` → `/compact` → `/harness-audit` decision tree is correct and complete.** The ordering matters: pruning first ensures MEMORY.md isn't truncating, compacting clears conversation weight, harness-audit is only needed when the fresh-session baseline is persistently heavy. Don't reach for harness-audit when compact is the right lever.
- **Byte overage in MEMORY.md is almost always verbose Watch-section entries or stale CLOSED workflow lines.** Those are the cheapest targets: shorten inline (Watch entries) or move to Historical (CLOSED workflow state). Never prune promoted patterns or active carry-forward entries.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Manual harness analysis (initial) | — | 5 | high | Misclassified governance-log-registry as always-load; PMD immediately corrected it |
| `memory_search_hybrid` (2× PMD checks) | 10 | 0 | medium | Both searches returned directly relevant prior incidents; semantic search working correctly post-DNS fix |
| `wc -c` byte measurement | 2 | 0 | low | Confirmed MEMORY.md over-limit; gave exact overage figure for prune targeting |
| Frontmatter classification bash loop | 3 | 0 | none | Clean classification of all 24 rules files; confirmed 11 SCOPED vs 13 ALWAYS |
| `/memory-prune` skill | 8 | 0 | low | Correct findings, clean apply; 8 entries actioned without needing a second pass |
| User decision tree explanation | 5 | 0 | none | `/compact` → `/memory-prune` → `/harness-audit` ordering was useful and not previously written down this clearly |

## Complexity scores (heavy tasks only)

No impl-tasks this session. All work was advisory/analysis + one MEMORY.md edit. N/A.

## Decisions to revisit

- `compact-phase.md` token ceiling: worth adding but low urgency — the current instruction works when followed strictly; the ceiling is a guardrail for sessions that generate verbose summaries.
- MiniMax A/B trial (moved to Historical): status unknown at RT-r4 ship. Revisit when authoring the next RT planning brief — check `.claude/PRPs/briefs/minimax-m27-trial-1.md` to see if the trial was ever dispatched.

---

## Promotion candidates (recurrence ≥ 2)

- [x] **Frontmatter-check before always-load claim (2× misclassification, same file):** add one sentence to the advisor's inline analysis discipline — "before citing a `.claude/rules/*.md` file as always-loaded, read line 1; `---` means SCOPED." Candidate for a one-liner addition to `feedback_subagent_frontmatter_detection.md` (broaden scope from subagents to advisor-inline analysis). **User must confirm before edit.**

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
