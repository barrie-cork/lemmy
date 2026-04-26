---
name: PHASE_1_MIGRATION_COUNT is LIFO-positional, not a semantic set
description: The constant is a revert-limit, not a named-migration list. Any migration added post-trunk silently takes the Nth slot. Plan VALIDATE blocks must mention LIFO semantics for migration-adding tasks.
type: feedback
originSessionId: 211a8c3b-c503-4ae0-afdb-bd47dba4fdc6
---
**Rule:** `PHASE_1_MIGRATION_COUNT` in `crates/server/tests/e2e.rs` drives `lemmy_diesel_utils::schema_setup::run(Options::default().revert().limit(N), ...)`. The runner reverts the **top-N-by-timestamp pending migrations** — it is a LIFO-positional count, not a semantic set. The constant's name suggests "number of migrations in the Phase 1 bootstrap set" — that framing is wrong and has caused mid-phase surprise twice (Phase 5b, v1-JM-a).

**Why this matters:**
- Any migration added to the fork *after* the last count bump silently takes the Nth slot in the LIFO window. The comment block next to the constant naming specific migrations rots — the runner doesn't honour the names.
- v1-AD-a shipped 4 migrations without bumping the count. Those 4 are currently uncounted (below the v1-JM-a count=12 LIFO window) and would be silently swapped into the revert list if the test is un-ignored (GH #43).
- `phase1_migrations_round_trip` is currently `#[ignore]`'d for unrelated reasons; un-ignoring it surfaces the LIFO gap immediately.

**How to apply:**
- **Planning a task that adds migrations:** the task's VALIDATE block must mention:
  - "bumps PHASE_1_MIGRATION_COUNT by N (LIFO-positional; the constant is a revert-limit, not a named set)",
  - the new count value,
  - a note that the comment block next to the constant must be rewritten to reflect new reality, not patched.
- **Planning a task that does NOT add migrations (e.g., handler-only like v1-JM-b):** do NOT touch the count or its comment. Silence is correct.
- **Impl receiving such a task:** do NOT infer a semantic set from the constant's name. If the plan says "extend by N", extend by N mechanically and rewrite (not patch) the comment. If the plan implies a semantic set ("count migrations in set X"), queue a decision-queue entry.
- **Architectural improvement (roadmap, not in-phase):** replace the count with a named-migration list via `schema_setup::run(Options::default().revert_to("<name>"), ...)`. Scoped as a v1.5-candidate GH issue per v1-JM-a retro §3.3. Naming the migration target eliminates the silent-slot-swap class entirely.

**Retire when:** the count is replaced with a named-migration list (v1.5-candidate). Until then, this rule applies to every Brehon plan that adds migrations. Source: v1-JM-a retro §2.3 R10.1 + plan-amendment row 3 (§3.2).
