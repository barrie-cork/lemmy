---
harvest_tier: <1|2|3>
batch: <N>           # batch number within the tier, e.g. 1
date: <YYYY-MM-DD>
item_count: <N>
source_retro: <path-to-retro-file-or-"multiple">
head_sha: <git-sha-verified-against>
---

# Retro Harvest <YYYY-MM-DD> — Tier <N> Batch <M> Implementation Plan

> **Canonical sibling:** this template was promoted 2026-05-31 after 2× recurrence
> (Tier-1 batch + Tier-2 batch-1 in the same harvest session). Per
> `feedback_read_canonical_before_writing_spec.md`, read 1-2 sibling harvest plans
> before filling this template — the existing plans at `~/.claude/plans/create-a-plan-to-cheeky-flute.md`
> (Tier-2 batch-1) and the Tier-1 batch (committed at `675db1963`) are the canonical examples.

## Context

<One paragraph: what harvest tier/batch this covers, item count, filter criteria
(e.g. "LIVE + advisor-session-executable + pure doc/rule/skill — no Rust, no migrations,
no PR needed"), and where the items originate (retro file(s), phase).)>

Key constraints for THIS batch:
- Fully LIVE (verified against HEAD `<sha>`)
- <constraint 2, e.g. "Executable in one advisor session without Junior dispatch">
- <constraint 3, e.g. "Pure doc/rule/skill edits — no Rust, no migrations, no PR needed">

All changes land directly on `<branch, e.g. governance-v0>`.

---

## Items (in execution order)

<!-- ITEM FORMAT — repeat this block for each item. Order items so that:
     (a) items editing the same file are grouped together (one file-pass per file),
     (b) lower-risk / smaller items precede larger ones,
     (c) file-creation items (new lessons) follow their related rule-edits.
     The execution order section below references these item numbers. -->

### Item 1 — <Title> (<Source tier/item ref, e.g. Tier 2 #12>)
**<size-tag: trivial|minor|medium|major> | <what it adds: e.g. "1 sentence added to existing bullet">**

<1-2 sentence description of the gap this item closes. What exists, what's missing,
why it matters. Read like a commit body — no padding.>

**Edit:** `<file-path>` — <what section/location is being changed>

<CHOOSE ONE of the three edit formats below and delete the others:>

<!-- FORMAT A — exact old_string/new_string (preferred for all rule/lesson edits) -->
Current text (verbatim):
> <exact quote of old_string, including surrounding context if needed for uniqueness>

Replace with:
> <exact quote of new_string>

<!-- FORMAT B — append at end of section (when there's no clean old_string anchor) -->
Append after `<last-line-of-target-section>`:
```
<verbatim text to append>
```

<!-- FORMAT C — new file creation -->
**Create:** `<file-path>`

```markdown
<full file content, including YAML frontmatter if it's a lesson>
```

---

### Item 2 — <Title> (<Source ref>)
**<size-tag> | <what it adds>**

<description>

**Edit:** `<file-path>` — <location>

<!-- ... -->

---

<!-- Add more Item N blocks as needed. -->

---

## Execution order

<!-- List the actual execution sequence. Group file-passes so a file with multiple
     items is opened once, all edits applied, then closed. -->

1. Items <N> & <M> — both edit `<file>`; do in one pass
2. Item <P> — edit `<file>` (refs file)
3. Items <Q> & <R> — Item Q edits `<file>` §<section> + creates lesson; Item R edits `<file>` §<section2>; group into one file-pass
4. Item <S> — edit `<file>`
5. Item <T> — edit `<file>` (need to read current structure first)
6. Single commit: `<commit subject, e.g. docs(advisor): retro-harvest YYYY-MM-DD Tier-N batch-M — N items>`

## Verification

<!-- One grep/ls per item. These run AFTER all edits and BEFORE the commit.
     Every item must have at least one verifiable signal. -->

- `grep -n "<distinctive-phrase-item-1>" <file-1>` → <expected result, e.g. §1 surface-first bullet>
- `grep -n "<distinctive-phrase-item-2>" <file-2>` → <section name>
- `grep -n "<distinctive-phrase-item-3>" <file-3>` → new subsection
- `grep -n "<distinctive-phrase-item-4>" <file-4>` → §5.5
- `ls <lesson-file-path>` → exists
- `grep -n "<distinctive-phrase-item-N>" <file-N>` → <section>
