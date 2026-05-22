---
name: memory-prune
description: >
  Prune stale, duplicate, and superseded entries from MEMORY.md (user auto-memory index).
  Use when: user says "prune memory", "clean up MEMORY.md", "memory is too big", "trim stale entries",
  or when /context shows memory files > 25% of context budget. Also use after a phase ships or closes
  (entries referencing the closed phase are pruning candidates). Surfaces findings first, applies only
  after user confirmation.
---

# Memory Prune

Scan MEMORY.md for stale, duplicate, and superseded entries. Report findings with reasoning, then
apply fixes only after user confirmation.

## Why this skill exists

MEMORY.md is auto-loaded into every Claude Code session. It has a 200-line hard limit (lines past 200
are silently truncated). Each line costs ~35 tokens. Stale entries displace useful context and can
mislead future sessions with outdated claims. Regular pruning keeps the index under budget and
accurate.

## Step 1: Read current state

Read the MEMORY.md index file. Note the current line count. If over 190 lines, flag urgency.

The file location is the user-scope auto-memory index. In this project it lives at:
`~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/MEMORY.md`

Also note the "Historical" section at the bottom — entries moved there are recoverable by filename
but no longer auto-loaded.

## Step 2: Classify every entry

Walk each line in MEMORY.md. For each entry, classify into one of:

| Classification | Criteria | Action |
|---|---|---|
| **Stale** | References a phase/PR/feature that shipped or closed > 7 days ago AND the entry's signal is now encoded in an auto-loaded rule file (`.claude/rules/*.md`) | Move to Historical |
| **Duplicate** | Same `.md` file referenced from two different lines, OR two entries covering the same topic where one subsumes the other | Remove the weaker/older entry |
| **Superseded** | A newer entry explicitly replaces this one (e.g. "SUSPENDED" supersedes "SHIPPED") | Remove the older entry |
| **Redundant with rules** | The entry's signal is fully covered by an auto-loaded `.claude/rules/*.md` file | Remove (the rule IS the enforcement; the memory adds nothing) |
| **Verbose** | Entry line exceeds ~200 chars and can be shortened without losing the retrieval hook | Shorten in place |
| **Keep** | Still load-bearing — references active work, encodes a decision not in any rule, or is a promoted pattern | No action |

To check whether a memory is redundant with rules, grep the rules directory for the key concept.
If the rule covers it comprehensively, the memory is redundant.

To check whether a "CLOSED" workflow state entry is stale: if the phase shipped > 7 days ago AND
no active work references it (no "carry-forward" or "cleanup at next" note), it can move to
Historical.

## Step 3: Report findings

Present a table to the user:

```
## Memory Prune Findings

Current: <N> lines (limit: 200)
Post-prune estimate: <M> lines

| # | Line | Entry (short) | Classification | Reason |
|---|------|---------------|----------------|--------|
| 1 | 35   | Shape G adoption | Superseded | SUSPENDED entry on line 36 carries current state |
| 2 | 38   | DQ protocol | Redundant | Fully in auto-loaded decision-queue.md rule |
| ... | | | | |

Entries marked Keep: <count> (not shown)
```

Group by classification. List "Keep" count but don't enumerate them — the user cares about what's
being removed, not what stays.

## Step 4: Confirm and apply

Ask the user: "Apply these changes?" with options:
- "Apply all" — execute all proposed changes
- "Apply selected" — let the user pick which findings to act on
- "Report only" — stop here, user will handle manually

On confirmation, apply the edits:

1. **Remove/shorten** entries from their current sections
2. **Move to Historical** — append removed entries to the "Historical" section at the bottom,
   grouped by date: `- archived <date>: <comma-separated list of filenames with one-word reason>`
3. **Verify** line count is under 200 after all edits
4. **Report** final line count and estimated token savings (~35 tokens per line removed)

## Edge cases

- **Entries with "carry-forward" or "cleanup at next"**: These are NOT stale even if the referenced
  phase is closed. They carry an explicit future obligation. Keep them until that obligation fires.
- **Promoted patterns (3+ occurrences)**: These are the highest-value entries. Never prune unless
  the pattern is now enforced by a structural fix (auto-loaded rule + hook).
- **Watch items**: Keep unless the watch condition has been resolved or the item has been quiet for
  30+ days with no third occurrence.
- **Reference entries**: Low cost (one line each). Only prune if the reference is broken (file
  deleted) or subsumed by a rule.
- **Empty lines / section headers with no entries**: Clean up orphaned headers only if the entire
  section is empty post-prune.

## What NOT to prune

- Active workflow state entries (the ACTIVE line)
- Entries less than 7 days old (too fresh to judge)
- Entries referenced by name in any `.claude/rules/*.md` file
- The "Promoted patterns" section (these are confirmed 3+ occurrence failure modes)
- Entries the user explicitly asked to keep in a prior session
