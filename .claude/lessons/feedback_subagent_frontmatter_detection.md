---
name: Subagent frontmatter detection for auto-load classification
description: When a subagent audits rule files for ALWAYS vs SCOPED classification, the probe must read the first 10 lines to detect paths: frontmatter — not just check file existence or line count. Miss costs ~10 min of mid-execution reframe.
type: feedback
---

When delegating a harness inventory to an Explore or general-purpose subagent, the subagent prompt MUST explicitly instruct the agent to:

1. **Read the first 10 lines of each file** before classifying it.
2. Treat a file as **SCOPED** (not ALWAYS) only when line 1 is `---` AND a `paths:` key appears within the frontmatter block.
3. Treat every other file as **ALWAYS** (auto-loads at session start regardless of which file the session Reads).

**Why:** The 2026-05-09 c-2 audit miss — `governance-log-entry-kind-registry.md` was misclassified as ALWAYS because the subagent inferred load-scope from filename pattern rather than reading the frontmatter. The file has `paths:` frontmatter and is SCOPED. The misclassification inflated the "ALWAYS load" char count and almost produced a false recommendation to trim a file that doesn't auto-load in meta-work sessions. ~10 min of mid-execution reframe to correct after the subagent returned its table.

**How to apply:** In any subagent prompt that classifies `.claude/rules/*.md` by load scope, copy the detection logic verbatim — "Read first 10 lines; line 1 is `---` and contains a `paths:` block → SCOPED; else ALWAYS." Do NOT paraphrase or infer from filename. The frontmatter is the contract.

This applies across all harness-auditing skills, advisor clarify runs, and any one-off inventory task that needs to separate scoped from always-load rules.
