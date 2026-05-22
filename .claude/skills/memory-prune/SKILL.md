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

## Step 3.5: Rule-side cut gate (only if proposing cuts to `.claude/rules/*.md` or `CLAUDE.md`)

**This step fires only when the prune scope extends beyond MEMORY.md** — e.g. the user asked
"what else could be removed from session-injected memory", `/context` showed rule files dominate,
or the audit identified rule-side externalization candidates. Routine MEMORY.md pruning skips
this step entirely.

The risk class this gate addresses: **proposal-pass analysis under-tests load-bearingness on
rule-side cuts.** A section that looks verbose and externalizable in the proposal pass may turn
out to BE the mechanism the rule encodes (e.g. a verbatim-blockquote example that IS the
anti-paraphrase gate; a procedure block that subagents read at task dispatch). Mirrors
`feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md` — cheaper to dry-run
at proposal time than to catch at execute time.

### Step 3.5.a: Load-bearing dry-run per proposed cut

For each proposed cut targeting a `.claude/rules/*.md` or `CLAUDE.md` section, run **both** checks
before listing the cut as admissible:

1. **Cross-reference grep.** Grep the section's distinctive content (heading or a unique phrase
   from the body) across the four downstream consumer paths:

   ```
   .claude/skills/        — skill bodies that may read the rule at invocation
   .claude/agents/        — subagent contracts that may cite the section
   .claude/commands/      — project-scope slash commands
   ~/.claude/commands/    — user-scope slash commands (auto-phase, roadmap-next, etc.)
   ```

   Any match means a downstream consumer depends on the inline form. Externalizing the section
   forces every consumer to add a `Read .claude/refs/<path>.md` step, OR breaks the dependency
   silently.

2. **Is-it-the-mechanism check.** Ask: does the section's CONTENT enforce the rule, or does it
   merely DESCRIBE the rule? Two diagnostic patterns:
   - **Verbatim example that must be copied** (e.g. a blockquote a brief MUST paste literally) →
     the example IS the mechanism. Externalizing defeats the rule.
   - **Procedure a worker reads at dispatch** (e.g. "How to write a question" sections that
     Junior subagents read every task start) → externalizing pushes cost to every dispatch.
   If either pattern fits, the section is load-bearing inline.

If EITHER check identifies a dependency, **drop the cut from the proposal**. Do NOT surface
it to the user as a viable option — that wastes their decision budget on a cut you would
have rejected on execute-pass anyway.

### Step 3.5.b: Verification tag per surviving cut

Every rule-side cut that passes §3.5.a MUST carry one of three explicit tags in the proposal
output:

| Tag | Meaning | Admissible? |
|---|---|---|
| `verified-pure-prose` | Section is descriptive/historical narrative; no consumer cites it; no mechanism depends on inline form | Yes — propose freely |
| `verified-historical-only` | Section documents past migrations or superseded behavior; current state is captured elsewhere | Yes — propose freely |
| `unverified-load-bearing-risk` | §3.5.a couldn't fully clear the section (grep returned ambiguous matches, or mechanism status is unclear) | **No — inadmissible without sentinel-probe (Step 3.5.c) first** |

The tag goes in the findings table's `Reason` column. Forces the proposal pass to declare its
certainty per cut instead of hiding uncertainty in confident prose.

### Step 3.5.c: `.claude/refs/` sentinel-probe (mandatory before any rule relocation)

This step applies when a proposed cut moves a rule from `.claude/rules/<file>.md` to
`.claude/refs/<file>.md` (or any new location). Per
`feedback_context_trim_verify_empirically.md` (PMD #147): documentation claims about Claude
Code load behavior have been wrong before — `.claude/rules/` was thought not to recurse into
subdirectories, and it does; archive trims under that assumption shipped zero savings until
proven empirically.

**Procedure (one-time per session that proposes a relocation):**

1. Write `.claude/refs/_test_trigger_<UTC-ISO>.md` containing a unique sentinel string
   (e.g. `SENTINEL_REFS_PROBE_<random-hex>`).
2. Commit + push to `governance-v0`.
3. Close current Claude Code session.
4. Open fresh session in the project CWD.
5. Run `/context`. Inspect the "Memory files" list.
6. **PASS:** test trigger file NOT in Memory files. `.claude/refs/` does not auto-load.
   Relocation cuts are admissible. Delete sentinel + commit.
7. **FAIL:** test trigger file IS in Memory files. `.claude/refs/` auto-loads. Relocation
   delivers zero savings. **STOP** — re-scope cuts as in-rule compression rather than
   relocation; surface the result to user.

The sentinel-probe is a hard refusal for relocation cuts. Do NOT propose relocation cuts on
the assumption that `.claude/refs/` is non-loading without running this probe first (or
relying on a prior probe result documented in a session retro within the last 30 days).

### Step 3.5 summary

Three gates, in order: (a) dry-run, (b) tag, (c) probe (only for relocations). A rule-side
cut surfaces to the user in the §4 plan only after all applicable gates pass. The two-minute
gate cost prevents the proposal-pass-vs-execute-pass gap (recurrence ≥ 2 in
`session-retro-2026-05-22-context-prune-option-a.md` — G4 example + DQ how-to-write both
correctly self-rejected on execute-pass; proposal pass should have caught both at audit time).

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
