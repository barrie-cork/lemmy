---
name: Frontmatter detection for auto-load classification (subagent and advisor-inline)
description: Before classifying any .claude/rules/*.md file as always-loaded — whether in a subagent prompt or advisor inline analysis — read line 1. If it starts with ---, the file has paths: frontmatter and is SCOPED, not ALWAYS. Misclassification has recurred twice on the same file (governance-log-entry-kind-registry.md).
type: feedback
---

Before classifying any `.claude/rules/*.md` file as always-loaded, **read its first line**. The frontmatter is the contract — filename, size, and topic are not reliable proxies.

**Detection rule (applies to both subagent prompts and advisor inline analysis):**

1. Read the first 10 lines of the file.
2. If line 1 is `---` AND a `paths:` key appears in the frontmatter block → **SCOPED** (loads only when the session Reads a matching path; never contributes to the session-start baseline).
3. Every other file → **ALWAYS** (auto-loads at session start regardless of which file the session Reads).

**Why:** Recurred twice on `governance-log-entry-kind-registry.md` (27 KB):

- **2026-05-09 (c-2 audit):** subagent inferred load-scope from filename pattern without reading frontmatter → misclassified as ALWAYS → inflated "ALWAYS load" char count → near-false trim recommendation. ~10 min reframe.
- **2026-05-31 (context budget session):** advisor inline analysis claimed the file was an always-load problem without checking frontmatter → same misclassification → PMD retrieval corrected it within minutes, but the initial analysis was wrong.

Both times the lesson was already on disk and in PMD. The failure was skipping the one-line check. The frontmatter-check is mechanical and costs <5 seconds; skipping it cost 5–10 min each time.

**How to apply:**

- **In subagent prompts:** copy the detection logic verbatim — "Read first 10 lines; line 1 is `---` and contains a `paths:` block → SCOPED; else ALWAYS." Do NOT paraphrase or infer from filename.
- **In advisor inline analysis:** before stating "file X is always-loaded and contributes Y KB to baseline," run `head -1 .claude/rules/X.md` or read first 10 lines. One-line check, non-negotiable.

Applies to: harness-audit skill Phase 1, `/memory-prune` Step 3.5 rule-side cuts, any ad-hoc context budget investigation.
