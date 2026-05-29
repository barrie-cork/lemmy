---
name: Effective context budget is ~200K, not the 1M ceiling — and rules dominate the auto-load
description: Feedback rule — "% of budget" triggers mean % of the ~200K working window (compact past it); the rules corpus is ~86% of the session-start auto-load, MEMORY.md only ~11%
type: feedback
originSessionId: dc348274-fb71-4895-9bd1-b8ea07d42e7c
---
The model's hard context ceiling is 1M tokens (`opus-4-8[1m]`), but the **effective working budget
is ~200K**. Sessions compact or restart around 200K because reasoning and recall quality degrade
dramatically past it (per user, 2026-05-29). **Every "% of context budget" heuristic in any skill,
rule, or trigger means % of ~200K — never % of 1M.**

**Why:** anchoring a "% of budget" trigger to the 1M ceiling hides real problems. The `memory-prune`
skill's "/context shows memory files > 25% of budget" trigger, read against 1M, means 250K tokens of
memory before it fires — absurd. Read against the real 200K window, it means ~50K — a sane upper
bound. On 2026-05-29 the session-start auto-load was measured at **~59K tokens ≈ 30% of the 200K
window** while being only ~6% of 1M: a genuine budget concern that the 1M framing made look harmless.

**The auto-load breakdown (measured 2026-05-29, recompute as it grows):**

| Source | Tokens | Share of auto-load |
|---|---|---|
| `.claude/rules/*.md` (24 files, ~205 KB) | ~51K | **~86%** |
| MEMORY.md | ~6.3K | ~11% |
| CLAUDE.md | ~1.8K | ~3% |
| **Total session-start auto-load** | **~59K** | **~30% of 200K** |

**How to apply:**
- When evaluating "is memory/context too big", divide by **200000**, not 1000000.
- **MEMORY.md is a small minority (~11%) of the auto-load.** When `/context` shows the Memory-files
  bucket heavy, pruning MEMORY.md (via `memory-prune`) barely moves the needle — the lever is the
  **rules corpus**. Route budget-pressure cases to the `harness-audit` skill (it scores
  rule-compression / externalization candidates) + the `memory-prune` Step 3.5 rule-side cut gate.
- Recompute the split when it matters: `find .claude/rules -name '*.md' -exec cat {} + | wc -c` vs
  MEMORY.md `wc -c`; divide by 4 for tokens; compare to 200000.

Companion: `feedback_context_trim_verify_empirically.md` (measure the limit that actually fires;
MEMORY.md is byte-limited at ~24.4 KB, not line-limited). Pattern: [[pattern_context_is_finite]].
