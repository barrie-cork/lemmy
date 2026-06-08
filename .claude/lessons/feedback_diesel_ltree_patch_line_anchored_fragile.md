---
name: diesel_ltree.patch is a line-anchored fixup — fragile system-of-record for schema.rs regen
description: The committed schema.rs manual fixups (Ltree import + allow_tables additions) live in a line-anchored git diff patch whose hunk #2 (~line 1366) silently rots when a new migration inserts tables and shifts offsets; replace with table-name-keyed idempotent post-processing or build-time drift-detection codegen test
type: feedback
---

# diesel_ltree.patch — line-anchored patch is the fragile link in the schema.rs regen loop

## TL;DR

`crates/db_schema_file/diesel_ltree.patch` is the system-of-record for two **manual fixups** that `diesel print-schema` (the schema.rs regeneration step run by the migration runner) does NOT produce on its own:

- **Hunk #1** (~line 152): rewrites the `comment` table's `Ltree` import from `super::sql_types::Ltree` → `diesel_ltree::sql_types::Ltree`.
- **Hunk #2** (~line 1366): appends `person_actions` + `image_details` to the `allow_tables_to_appear_in_same_query!` macro.

Both hunks are **anchored to line numbers**. When a new migration adds tables, the regenerated `schema.rs` inserts new `diesel::table!` blocks and new entries in the `allow_tables_to_appear_in_same_query!` macro — shifting every downstream line offset. Hunk #2 sits near the END of the file (line ~1366, after `surety`), so it is the most exposed: any table that sorts before the hunk's context shifts its anchor and can make `git apply` fail with "patch does not apply" or, worse, apply against the wrong context.

## Why this is in play right now (m2-late-1 T1, 2026-06-07)

T1 adds `sanction_event` + `sanction_subscriber` tables. `sanction_*` sorts alphabetically before `surety`/`site`, so a `print-schema` regen would insert those table blocks AND their macro entries **above** hunk #2's anchor region — shifting hunk #2's line offset. This is the "hunk #2 staleness" risk.

**Mechanism (verified 2026-06-07 — corrects the earlier "git apply" framing in this lesson):** the patch is applied by **diesel_cli (2.3.7) via the `diffy` 0.4.2 crate**, NOT `git apply`, and ONLY during `diesel print-schema` (it is a `diesel.toml` `patch_file` input; never consumed at build time). `diffy` IS offset-tolerant — it locates a shifted hunk by outward search from the declared line — so a **pure offset shift still applies**. It has **zero fuzz**, though: context lines must match byte-for-byte, so if a new table lands *inside* hunk #2's contiguous context block, the hunk fails. On failure `diffy::apply` → `?`-propagates → diesel exits 1 with NOTHING written; and because the regen scripts use `diesel print-schema > schema.rs`, the `>` redirect has already **truncated schema.rs to empty** — the nonzero exit is the only safe signal. (Evidence: diesel_cli-2.3.7 `src/print_schema.rs:289-308`, `src/main.rs:45-50`; diffy-0.4.2 `src/apply.rs` `find_position`/`match_fragment`.)

**Mitigation when print-schema IS used:** regenerate the patch from the correct post-regen baseline if hunk #2 fails, rather than force-applying. **When hand-editing schema.rs (the brehon-fork convention — prior phases fed-in-a `0c9329da9`, v1-AD-a `dbaf58e4b`, v1-SL-a `c7a977078` all hand-extended the `@generated` file; see `.claude/PRPs/handovers/m2-late-1-t1-validate-resume.md` Task A for the turnkey edit spec), print-schema never runs, so the patch and its staleness never engage** — the patch's effect is already baked into the committed schema.rs.

## The structural weakness

A **line-fragile patch as the system-of-record for a manual fixup** is a weak point. Line numbers are not a stable key for "the Ltree import on the comment table" or "person_actions belongs in the same-query allowlist" — the table NAME is the stable key. The patch encodes a semantic fixup in a positional format, so it rots on every migration that touches table ordering.

## Better engineering (follow-up, not in-phase)

Two durable replacements, either of which eliminates the staleness class:

1. **Build-time drift-detection codegen test.** A test that regenerates schema.rs (`diesel print-schema` against a migrated throwaway DB, read-only — safe per the forbid-trigger, the print-schema path doesn't take `pg_advisory_lock(0)`) + applies the manual fixups, then asserts the result byte-matches the committed `schema.rs`. Drift → loud test failure at the source change, not a latent `git apply` failure two migrations later. (Same class of fix as the e2e.rs corruption that sat latent — see the m2-core-hook retro "trunk compile-only CI gate" item.)

2. **Idempotent post-processing script keyed on table name, not line number.** Replace the patch with a `sed`/AST-rewrite step that: (a) rewrites the `Ltree` import on the `comment` table block by matching the block, not the line; (b) ensures `person_actions` + `image_details` (and any future manual-allowlist additions) are present in `allow_tables_to_appear_in_same_query!` by name, inserting only if absent. Re-applies cleanly regardless of offset; no patch to regenerate per migration.

Option 2 is the lower-effort durable fix (no DB spin-up in the test loop); option 1 is the stronger guarantee (catches ANY divergence, not just these two fixups). They compose — the post-processor produces the canonical file, the drift test asserts the committed copy matches it.

## How to apply (until the structural fix lands)

- **Any migration that adds tables** → after `schema.rs` regen, verify hunk #2 of `diesel_ltree.patch` still applies. If offsets are stale, **regenerate the patch** from the new baseline (`git diff` the regenerated-and-manually-fixed schema.rs against the print-schema output) rather than force-applying.
- **Watchpoint for the planner:** a plan that adds tables AND touches the `allow_tables_to_appear_in_same_query!` allowlist should name the patch-regen step explicitly in the task DoD, not assume the committed patch applies.

## See also

- `feedback_lemmy_migration_runner.md` — the migration runner (`cargo run -p lemmy_diesel_utils --features full`, NO sub-command args — `main.rs` bails on any arg) + `diesel print-schema` is read-only/safe.
- `feedback_phase1_migration_count_lifo.md` — adjacent migration-fragility class (LIFO revert-count rots the same way the patch anchors do).
- m2-core-hook retro "trunk compile-only CI gate" — the latent-drift-caught-late pattern this fix prevents.
