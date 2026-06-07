# S6 — diesel_ltree.patch line-anchored staleness (schema-tooling follow-up)

**Status:** OPEN — proposed scoped impl-task, not yet planned.
**Origin:** carved out of the 2026-06-07 m2-late-1 T1 validate-pending RCA (S6 in the
four-solution analysis). Orthogonal to the branch-correctness fix shipped in `24d457f61`
— this bug bites even on a perfectly correct branch.
**Severity:** medium. Latent; fires only when a migration adds/removes tables.
**Will fire next at:** the m2-late `sanction_event` / `sanction_subscriber` migration's
schema.rs regen (those tables sort alphabetically before `surety`/`site`, shifting hunk #2).
**Related PMD:** memory **#881** ("diesel_ltree.patch is a line-anchored fixup — fragile
system-of-record for schema.rs regen"). NOTE: #881's `file_path` claims
`.claude/lessons/feedback_diesel_ltree_patch_line_anchored_fragile.md`, but that file was
**never committed** — the PMD row exists, the lesson file does not. Either commit the
lesson file or correct the PMD row's `file_path` when this is actioned.

## The defect

`crates/db_schema_file/diesel_ltree.patch` is the committed system-of-record for two
**manual fixups** that `diesel print-schema` does NOT emit on its own:

- **Hunk #1** (~`crates/db_schema_file/src/schema.rs:152`): rewrites the `comment`
  table's `Ltree` import from `super::sql_types::Ltree` → `diesel_ltree::sql_types::Ltree`.
- **Hunk #2** (near EOF, ~line 1366 pre-m2-late): appends `person_actions` + `image_details`
  to the `allow_tables_to_appear_in_same_query!` macro.

Both hunks are **anchored to line numbers / surrounding context lines**. When a new
migration adds tables, regenerated `schema.rs` inserts new `diesel::table!` blocks and new
macro entries — shifting downstream line offsets. Hunk #2 sits near the END of the file, so
it is the most exposed: any new table that sorts before its anchor shifts the offset and can
make `git apply` fail with "error applying hunk #2" (the exact symptom seen in the T1 run,
trace step 7) or, worse, apply against the wrong context.

The structural weakness: **a line-fragile patch encoding a SEMANTIC fixup**. The stable key
for "the Ltree import on the comment table" or "person_actions belongs in the same-query
allowlist" is the **table name**, not a line number. The patch encodes a semantic fixup in a
positional format, so it rots on every migration that touches table ordering.

## Why it's not in the T1 fix

The T1 failure was a wrong-branch error (8 steps against `governance-v0`). Once that's fixed
(worktree-not-checkout, `24d457f61`), schema regen runs from the correct branch — but the
patch-staleness remains: it is a property of the patch format vs. table count, independent of
which branch you regen from. Folding it into the handler change would muddy both. Hence this
separate artifact.

## Proposed fixes (either eliminates the staleness class)

**Option A — build-time drift-detection codegen test (stronger guarantee).**
A test that: (1) regenerates `schema.rs` via `diesel print-schema` against a migrated
throwaway DB (read-only — safe under the forbid-trigger, print-schema doesn't take
`pg_advisory_lock(0)`, per `feedback_lemmy_migration_runner.md`); (2) applies the manual
fixups; (3) asserts the result byte-matches the committed `schema.rs`. Drift → loud test
failure at the source change, not a latent `git apply` failure two migrations later. Same
class of guard as the m2-core-hook retro's "trunk compile-only CI gate" (catch-latent-drift-
early). Cost: spins a DB in the test loop.

**Option B — idempotent post-processing keyed on table name (lower effort).**
Replace the patch with a `sed`/AST-rewrite step that: (a) rewrites the `Ltree` import on the
`comment` table block by matching the BLOCK, not the line; (b) ensures `person_actions` +
`image_details` (and any future manual-allowlist additions) are present in
`allow_tables_to_appear_in_same_query!` by name, inserting only if absent. Re-applies cleanly
regardless of offset; no patch to regenerate per migration. Cost: no DB spin-up.

**They compose:** B produces the canonical file; A asserts the committed copy matches it.
Recommend B first (unblocks m2-late), A as a follow-up hardening.

## Interim mitigation (until the structural fix lands)

- **Any migration that adds tables** → after `schema.rs` regen, verify hunk #2 of
  `diesel_ltree.patch` still applies. If offsets are stale, **regenerate the patch** from the
  new baseline (`git diff` the regenerated-and-manually-fixed schema.rs against the
  print-schema output) rather than force-applying.
- **Watchpoint for the planner:** a plan that adds tables AND touches the
  `allow_tables_to_appear_in_same_query!` allowlist should name the patch-regen step
  explicitly in the task DoD, not assume the committed patch applies.

## Scope for the impl-task (when planned)

- IMPLEMENT: replace `crates/db_schema_file/diesel_ltree.patch` consumption with Option B
  (table-name-keyed post-processor); optionally add Option A drift test.
- Files: `crates/db_schema_file/` (schema regen tooling), the post-processor script location
  (likely `scripts/brehon/`), the manual-fixup invocation site.
- DoD: regen `schema.rs` with a freshly-added throwaway table → fixups apply by name,
  idempotent on re-run, no line-offset dependence; (Option A) drift test fails loudly on a
  hand-edited schema.rs.
- Out of scope: the migration runner itself (`lemmy_diesel_utils`), the forbid-trigger.

## Cross-references

- PMD #881 — the lesson (file uncommitted; see NOTE above).
- `.claude/lessons/feedback_lemmy_migration_runner.md` — migration runner + print-schema
  read-only safety.
- `.claude/PRPs/debug/m2-late-1-t1-validate-pending-advisor-session-trace.md` §"diesel_ltree.patch
  hunk #2 staleness" — the T1 occurrence.
- `crates/db_schema_file/diesel_ltree.patch` — the artifact to replace.
- `crates/db_schema_file/src/schema.rs` — the regen target (hunk #1 ~:152, hunk #2 near EOF).
- m2-core-hook retro "trunk compile-only CI gate" — the latent-drift-caught-late pattern
  Option A prevents.
