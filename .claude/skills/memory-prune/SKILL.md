---
name: memory-prune
description: >
  Prune stale, duplicate, and superseded entries from MEMORY.md (user auto-memory index).
  Use when: user says "prune memory", "clean up MEMORY.md", "memory is too big", "trim stale entries",
  or when MEMORY.md is over its byte budget (~24.4 KB — the SessionStart warning fires). Also use
  after a phase ships or closes, and ESPECIALLY at a MILESTONE BOUNDARY (e.g. v1 → M1) — a milestone
  transition retires a whole class of prior-milestone phase-texture at once and is the highest-yield
  prune trigger (see Step 2.5). The user signalling "we're on <new-milestone> now" IS this trigger.
  Surfaces findings first, applies only after user confirmation. NOTE: this skill prunes MEMORY.md
  ONLY. If /context shows the whole Memory-files bucket is heavy (> ~25% of the ~200K effective
  working window, i.e. > ~50K tokens — see "Budget framing" below), MEMORY.md is usually NOT the
  culprit (it is ~10% of the auto-load); the rules corpus is. Route that to `harness-audit`, not here.
---

# Memory Prune

Scan MEMORY.md for stale, duplicate, and superseded entries. Report findings with reasoning, then
apply fixes only after user confirmation.

## Why this skill exists

MEMORY.md is auto-loaded into every Claude Code session. Stale entries displace useful context and
can mislead future sessions with outdated claims. Regular pruning keeps the index under budget and
accurate.

**The binding limit is BYTES, not lines** (corrected 2026-05-29 after empirical observation — see
below). The harness loads MEMORY.md up to a **byte budget of ~24.4 KB (24400 bytes)**; past that it
**silently truncates** and emits a SessionStart warning like `MEMORY.md is 25.2KB (limit: 24.4KB) —
index entries are too long. Only part of it was loaded.` There is *also* a soft ~200-line ceiling
(~35 tokens/line), but at normal entry density (~135 bytes/line) the byte budget binds FIRST — a
file can be truncated at ~180 lines of average-length entries while still being "under" 200 lines.

**Why this correction was made:** on 2026-05-29 the file was 186 lines (under 200) but 25375 bytes —
the harness truncated it and warned, while a lines-only gate would have reported 14 lines of
headroom. A skill that measures only lines is blind to the limit that actually truncates the file.
This is the `feedback_context_trim_verify_empirically.md` / `pattern_test_against_reality_not_syntax`
class: the documented limit (lines) did not match observed load behavior (bytes). **Always measure
both; gate on bytes.**

## Budget framing — "% of budget" means the 200K working window, NOT the 1M ceiling

The model's hard context ceiling is 1M tokens (`opus-4-8[1m]`), but **the effective working budget
is ~200K** — sessions compact or restart around 200K because reasoning/recall quality degrades past
it (per user, 2026-05-29). So any "% of budget" trigger in this skill or its description means **% of
~200K, not % of 1M.** "Memory files > 25% of budget" = **> ~50K tokens** (25% of 200K), NOT 250K.
Anchoring to 1M is the trap: it makes a real problem (memory/rules at 30% of the window you actually
operate in) read as "harmless 6%."

**Two separate budgets, do not conflate:**

1. **MEMORY.md's own byte budget (~24.4 KB hard).** This skill's primary job. Mechanical, fires a
   SessionStart truncation warning. Prune MEMORY.md → fixes it.
2. **The whole session-start auto-load (`/context` "Memory files" bucket).** Measured 2026-05-29 at
   **~59K tokens ≈ 30% of the 200K window**, broken down as: **rules corpus `.claude/rules/*.md` ~51K
   (≈86%)**, MEMORY.md ~6.3K (≈11%), CLAUDE.md ~1.8K (≈3%). **MEMORY.md is a small minority of the
   auto-load.** When `/context` shows the bucket heavy, pruning MEMORY.md barely moves it — the lever
   is the **rules corpus**. Route budget-pressure cases to the `harness-audit` skill (it scores
   rule-compression / externalization candidates) + this skill's Step 3.5 rule-side cut gate. Do NOT
   try to solve a 30%-of-window auto-load problem by trimming MEMORY.md here; you'd remove useful
   index entries and the budget needle wouldn't move.

Recompute the rules-vs-MEMORY split when invoked (the rules corpus grows): `find .claude/rules -name
'*.md' -exec cat {} + | wc -c` vs MEMORY.md `wc -c`, divide by 4 for tokens, compare to 200000.

## Step 1: Read current state

Read the MEMORY.md index file. Measure **both** dimensions and record them:

```bash
MEM=~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/MEMORY.md
wc -c "$MEM"   # BYTES — the binding limit (budget ~24400)
wc -l "$MEM"   # lines  — secondary ceiling (~200)
```

**Urgency gate (gate on bytes first):**

| Bytes | Lines | Verdict |
|---|---|---|
| > 24400 | (any) | **OVER — truncating now.** Prune is mandatory; SessionStart already warned. |
| 22000–24400 | (any) | Near budget — prune proactively. |
| < 22000 | > 190 | Line-pressure even though bytes OK (many terse entries) — prune or compress. |
| < 22000 | < 190 | Healthy — prune only stale/duplicate/superseded entries, no urgency. |

A file can be OVER on bytes while UNDER on lines (e.g. 186 lines / 25375 bytes). **Bytes win** —
report the byte overage as the headline, lines as secondary.

The file location is the user-scope auto-memory index. In this project it lives at:
`~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/MEMORY.md`

Also note the "Historical" section at the bottom — entries moved there are recoverable by filename
but no longer auto-loaded. **Moving a long entry to Historical reclaims its full byte cost** from the
auto-loaded budget; shortening an entry in place reclaims only the trimmed bytes. When the file is
OVER on bytes, prefer (a) shortening the longest entries and (b) moving stale long entries to
Historical — both directly attack the byte overage; removing short entries barely moves it.

## Step 2: Classify every entry

Walk each line in MEMORY.md. For each entry, classify into one of:

| Classification | Criteria | Action |
|---|---|---|
| **Stale** | References a phase/PR/feature that shipped or closed > 7 days ago AND the entry's signal is now encoded in an auto-loaded rule file (`.claude/rules/*.md`) | Move to Historical |
| **Duplicate** | Same `.md` file referenced from two different lines, OR two entries covering the same topic where one subsumes the other | Remove the weaker/older entry |
| **Superseded** | A newer entry explicitly replaces this one (e.g. "SUSPENDED" supersedes "SHIPPED") | Remove the older entry |
| **Redundant with rules** | The entry's signal is fully covered by an auto-loaded `.claude/rules/*.md` file | Remove (the rule IS the enforcement; the memory adds nothing) |
| **Verbose** | Entry line exceeds ~200 chars (≈200 bytes) and can be shortened without losing the retrieval hook | Shorten in place (when OVER on bytes, this is a PRIMARY lever, not optional polish — target the longest entries first; detail belongs in the linked `.md` file, the index line is just a retrieval hook) |
| **Keep** | Still load-bearing — references active work, encodes a decision not in any rule, or is a promoted pattern | No action |

To check whether a memory is redundant with rules, grep the rules directory for the key concept.
If the rule covers it comprehensively, the memory is redundant.

To check whether a "CLOSED" workflow state entry is stale: if the phase shipped > 7 days ago AND
no active work references it (no "carry-forward" or "cleanup at next" note), it can move to
Historical.

## Step 2.5: Milestone-transition sweep (highest-yield prune lever)

**Fires when:** the project has crossed a milestone boundary since the last prune — the user says
"we're on <M> now", or git/roadmap shows a new milestone branch, or the prior milestone's entries
dominate MEMORY.md. This is a DIFFERENT axis from Step 2's per-entry staleness: a milestone
boundary retires a whole *class* of entries at once (prior-milestone phase texture), and it is
empirically the biggest single lever — a tiny byte-overage can hide a large milestone-archival
opportunity (2026-06-04 run: 311-byte overage, but the v1→M1 reframe surfaced 5 cuts the
byte-gate alone would never have prioritised).

**The discriminator that matters — transferable pattern vs phase-pinned texture.** This is the
single most error-prone call in a milestone prune. Most entries that *cite* a closed-milestone
phase still encode a pattern that transfers to the new milestone. Cutting them by keyword-match
("it says v1, we're on M1, delete") destroys durable knowledge. Apply this test to EVERY
entry that mentions a prior-milestone phase code:

> **Transfer test:** Strip the phase citation from the entry. Does the remaining statement still
> describe a mechanism, footgun, or pattern that the NEW milestone's work will hit?
> - **YES → KEEP** (the phase code is just the incident-evidence anchor, not the subject).
>   Example: "Cohort `[P]` shared `.git/index.lock` contention (RT-r3 cohort-2 cascade)" — strip
>   "RT-r3 cohort-2"; "cohort `[P]` shares `.git/index.lock`" is true for M1's daemon/cohort work
>   identically. Keep.
> - **NO → retire to Historical** (the entry IS phase texture — a scratchpad, a shipped-fix
>   workflow-state, a closed-phase handoff with no forward mechanism).
>   Example: "Brehon V1 planning advisor notes — V1 scratchpad" — strip the v1; nothing
>   general remains, it's a dated planning notebook. Retire.

**Same-codebase caveat (prevents over-cutting):** a milestone transition does NOT make
domain/tooling lessons stale if the codebase is unchanged. v1 and M1 are the same Lemmy-fork Rust
on the same Windows laptop + same EliteDesk daemon — so all Cargo/Rust, Windows-wrapper,
git-worktree, and daemon/cohort lessons transfer **unchanged**. Only retire entries whose *subject*
(not just citation) is the prior milestone: closed workflow-states, planning scratchpads, phase
handoffs, and milestone-specific feature notes. When unsure, the entry is one line — KEEP and flag
in the report rather than risk losing a durable pattern.

**What this sweep typically surfaces (in yield order):**
1. Closed prior-milestone **workflow-state** entries (COMPLETE/SHIPPED + >7 days) → Historical.
   These are usually the longest lines, so they pay the most bytes.
2. Prior-milestone **planning scratchpads / handoffs** (`project_<old>_planning.md`, dated session
   notes) → Historical. Verify they don't seed the new milestone (`grep` the linked file for the
   new milestone's name; 2026-06-04: confirmed `project_brehon_v1_planning.md` had zero M1 refs).
3. Entries the milestone transition made **redundant with a rule** (e.g. a workaround now codified)
   → remove per Step 2.

Entries that pass the transfer test go untouched regardless of how old the phase citation is.
Record the sweep's verdict per milestone-citing entry in the Step 3 report (`milestone-retire` vs
`transfers-keep`) so the user sees the discrimination, not just the cuts.

## Step 3: Report findings

Present a table to the user:

```
## Memory Prune Findings

Current: <BYTES> bytes (budget: 24400 — the binding limit) / <N> lines (ceiling: 200)
Post-prune estimate: <BYTES'> bytes / <M> lines
Headline: <"OVER budget by X bytes — truncating" | "under budget, X bytes headroom">

| # | Line | Entry (short) | Classification | Reason |
|---|------|---------------|----------------|--------|
| 1 | 35   | Shape G adoption | Superseded | SUSPENDED entry on line 36 carries current state |
| 2 | 38   | DQ protocol | Redundant | Fully in auto-loaded decision-queue.md rule |
| ... | | | | |

Entries marked Keep: <count> (not shown)
```

Lead with the byte figure (it's what truncates the file); show lines second. If OVER on bytes,
state the overage and that prune is mandatory.

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

1. **Use the Edit/Write tool — never shell heredocs / `sed` / `echo >>` — and grep-verify EACH edit
   landed before trusting it.** MEMORY.md is the user auto-memory index; mangling it with a shell
   redirect (CRLF, encoding, partial write) corrupts every future session's context. Apply each
   removal / shorten / Historical-append as an `Edit` (or `Write` for a full rewrite). After each
   call, the tool returns success or an error — do NOT narrate an edit as "done" until you have seen
   the success result, AND for any non-trivial edit confirm with `grep -c "<a distinctive phrase
   from the new text>" "$MEM"` returning ≥1. The byte/line re-measure in step 4 catches a size
   regression but NOT a no-op edit that changed nothing — a malformed or silently-failed tool call
   leaves the file unchanged AND under budget, reading as success. Confirm CONTENT, not just size.
   (2026-06-04 skill-improvement run: three successive tool calls were malformed, silently no-op'd,
   and were narrated as applied; only `grep -c` of the expected marker per edit exposed it. Grep
   each edit, every time.)
2. **Remove/shorten** entries from their current sections.
3. **Move to Historical** — append removed entries to the "Historical" section at the bottom,
   grouped by date: `- archived <date>: <comma-separated list of filenames with one-word reason>`.

   **⚠ The Historical/archive line you add IS itself a byte cost — budget for it.** A descriptive
   archive line can eat most of the savings you just freed; a verbose one can leave the file STILL
   over budget after a prune that looked sufficient on paper (2026-06-04: a multi-clause archive
   line landed the file 44 bytes OVER, forcing a second trim pass). Rule of thumb: the archive line
   should be SHORTER than the shortest entry you removed. Keep it to filenames + a one-word reason
   each; detail is recoverable from the linked `.md` file and from `git log`, so it does not belong
   in the index. If a removed entry carried a still-live hook (an expiry date, a deferred-bug
   pointer), preserve ONLY that hook in the archive line, not the entry's full prose.
4. **Verify** the file is under BOTH limits AFTER writing the archive line — `wc -c "$MEM"` < 24400
   (hard) AND `wc -l "$MEM"` < 200 (secondary). Re-measure after the archive append, not before (the
   archive line is part of the file's final size). If still over 24400 bytes, the prune is
   incomplete: do another pass on the longest remaining entries (shorten in place, move to
   Historical, OR trim the archive line you just added) until under budget. A pass that lands under
   200 lines but over 24400 bytes has NOT solved the truncation.
5. **Report** final bytes + lines (e.g. `23980 bytes / 184 lines — under budget by 420 bytes`) and
   estimated savings (bytes reclaimed; ~35 tokens per line removed as a secondary figure).

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
