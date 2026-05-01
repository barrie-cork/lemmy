---
name: Runbook / audit claims drift from reality — verify each claim against live state before executing
description: Runbooks and migration audits written before an operation commonly mis-state current reality (already-gitignored files, wrong counts, inverted polarity, same-name-different-content files). Treat every runbook claim as a hypothesis to verify, not a fact to execute.
type: feedback
---

When a runbook or pre-migration audit says "file X is not gitignored", "there are N servers", or "file A and file B are duplicates" — verify each claim against the actual live state before acting on it.

**Why:** The advisor-CWD migration (2026-04-30) audited the brehon-fork and homeserver CWDs before migration. The audit got several material details wrong at execution time:

- `.mcp.json` was already in `.gitignore` (line 81 — the audit said it was not)
- `.mcp.json` had 6 configured MCP servers, not 8 (the audit counted stubs and dev entries)
- `memory-injection.md` polarity was inverted — brehon-fork PMD was the richer system, not homeserver (the audit said the opposite)
- The three "overlap files" (rules that appeared in both repos) had diverged content — they were not identical copies. Treating them as "pick one" would have lost changes.
- Commands listed as needing relative-path patches had no relative paths to patch (already used correct paths)

None of these errors blocked the migration outright, but each required a live re-check under smoke pressure, adding friction and risk of error compounding.

**How to apply:**

- For any runbook step that makes a factual claim about current state ("file X exists/is-gitignored/has-N-lines/is-identical-to-Y"), add an explicit verify step before the action step:
  - `.gitignore` membership: `grep -n '<file>' .gitignore` before concluding it is or isn't tracked
  - Count claims: `wc -l <file>` or `ls -1 | wc -l` before asserting a count
  - "Identical file" claims: `diff <a> <b>` before treating as duplicates — same filename ≠ same content after parallel editing
  - Polarity / which-system-is-richer claims: read both sides and compare line counts or section headers before deciding which is canonical
- Write runbooks with a `## Pre-flight verification` block that lists the live checks needed to validate the audit's assumptions, separate from the `## Steps` block that executes.
- If a runbook claim is wrong, update the runbook immediately (same session) so the next reader starts from reality.

**Symptom to recognise:** You start executing a runbook step and the action doesn't make sense (e.g. "add X to .gitignore" but `grep` shows it's already there). That's a runbook-drift event — stop, verify the remaining claims in the same section, then continue.

**Generalises to:** Any pre-written audit, migration checklist, or multi-step setup doc that was authored before execution. The longer the gap between writing and running, the more drift accumulates. Post-event verification is mandatory if the gap was > 1 day.

**Related lessons:**
- `feedback_plan_drift_metadata_cross_check.md` — same class (plan claim vs reality diverges between write and execute time)
- `feedback_test_scripts_against_real_targets.md` — test scripts against real input, not mental models
