---
name: Migration plans must enumerate ALL invariants from the canonical mirror, not just shape
description: When a migration plan references a mirror precedent file, the planner must enumerate every load-bearing invariant from the mirror — line-1 directive position, trailing no-op DDL on comments-only down.sql, Postgres-enum-rule split-by-directory, etc. Citing the mirror as a "shape reference" without enumerating its invariants caused three SL-a Task 1 fix-impls.
type: feedback
---
**Rule:** when a migration plan cites an existing migration directory as a precedent, the planner MUST enumerate every load-bearing invariant from the mirror's full content, not just the shape. A migration directory is not a single shape — it's a small bundle of independent invariants, and missing any one of them produces a downstream fail-impl cycle.

**Why this matters:**
- v1-SL-a Task 1 cited `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/` as the shape reference for "enum-only no-transaction migration." The mirror had THREE invariants in 7 lines:
  1. `-- no-transaction` directive on **line 1** (diesel parser reads first non-blank line only)
  2. Trailing `SELECT 1;` no-op DDL in down.sql (Postgres returns "Received an empty query" on a comments-only revert)
  3. **One enum variant per migration directory** — Postgres rejects "unsafe use of new enum value" within the same migration that adds it
- Plan absorbed only invariant 1 (indirectly via §10.1's combined-migration shape). Each missed invariant cost one fix-impl + ~25 min Junior cycle + workspace-check + e2e re-run. Total: ~3 hours of cycle time on three single-line edits.
- Each fix-impl was non-allowlist by §G4 rule but mechanical via mirror comparison — they would have collapsed to zero if the planner had read the mirror's full content at plan-write time.

**How to apply:**
- **Planning task brief:** when authoring a migration task, read the cited mirror file's `up.sql` AND `down.sql` AND any sibling `migration.toml`/`config.toml`/`README.md` in the directory. Enumerate every line as either (a) shape (boilerplate the planner already covered) or (b) invariant (load-bearing rule the impl must preserve). Plan §10.1 cites every (b)-class invariant explicitly.
- **Impl receiving such a task:** if the brief cites a mirror but enumerates fewer invariants than the mirror file actually contains, flag it as a brief gap before writing code. A 5-second `Read` of the mirror catches the planner-side miss.
- **Advisor §G4 classifier extension:** the three SL-a-derived classes are now allowlist candidates:
  - `migration-no-transaction-line-1` (mirror-precedent line-1 directive invariant)
  - `migration-comments-only-down-sql-empty-query` (mirror-precedent trailing-DDL invariant)
  - `migration-postgres-unsafe-enum-use-split` (Postgres-enum-rule split-by-directory)
- **Plan-author-time gate (forward-going):** when a plan §10 references a mirror precedent file, surface a checklist asking: "Did you enumerate every invariant from the mirror's full content (line-1, trailing DDL, enum-rule, etc), or just the shape?" Run as part of the planning subagent's brief-author dogfood gate.

**Generalises to:** any plan citing a mirror — read the mirror's full structure, not just the shape. The same class of bug exists in `replaceable_schema/`, `crates/db_schema/migrations/replaceable_schema/`, and any future "look at the precedent at X" plan reference.

**Symptom to recognise:** an impl-task lands a migration that compiles + runs forward, but the down.sql revert errors with "Received an empty query" / "no transaction" / "unsafe use of new enum value" / similar Postgres parser error. If you see this, do NOT treat it as a single-bug fix — read the mirror and look for OTHER invariants the plan also missed before authoring fix-impl-1.

**Retire when:** the planning subagent's dogfood gate (proposed §7 follow-up #6) automates the mirror-invariant enumeration, OR the §G4 allowlist extends to cover all three classes (proposed §7 follow-up #1). Source: v1-SL-a retro §3.1 + §5 + §7.
