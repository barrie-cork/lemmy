---
role: impl-task
plan_task: 1
phase: v1-JM-d
created: 2026-04-27
---

# Brief — v1-JM-d Task 1 — schema migrations

## 1. Role + dispatch line

`[role:impl-task] v1-JM-d task 1 — see .claude/PRPs/briefs/jm-d-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6 per your frontmatter). Execute plan task 1 from `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` §13 — schema migrations only. Do task 0 (pre-flight harness audit) first; **STOP and surface to advisor via DQ if any probe fails** — do not attempt task 1 on a broken environment.

## 2. Scope

**Produce** (one commit per the plan's "one commit per task" rule):
- Two new migration directories under `migrations/`, both timestamp-prefixed > `2026-04-23-000200-0000`:
  - `<ts>-000000-0000_add_appeal_requester_role_enum/{up,down}.sql`
  - `<ts>-000100-0000_add_appeals_v1_columns/{up,down}.sql`
- Migration content per plan §10.1 (verbatim SQL — do not improvise the column defaults, the ADR-exception comment block, or the `appeal_requester_role` enum values).
- A round-trip verification: apply → `revert --limit=2` → re-apply, all via `cargo run -p lemmy_diesel_utils --features full`. Capture each run's output to `.claude/PRPs/debug/v1-JM-d-task1-<probe>.log` per `.claude/rules/cargo-output-capture.md`.

**Do NOT** in this task:
- Touch any Rust file under `crates/**` (that's task 2).
- Touch `request_appeal.rs` (that's task 3).
- Run raw `diesel migration run` — forbid-triggered on this workspace per `feedback_lemmy_migration_runner.md`. Use `cargo run -p lemmy_diesel_utils --features full` only.
- Backfill any data. Plan §10.1 explicitly states no backfill; pre-JM-d Decided cases get `winning_decision = NULL` and pre-JM-d Appeal rows get `requester_role = 'Defendant'` via the metadata-only attmissingval path.

**Commit message** (exactly): `feat(v1-JM-d): add appeals v1 schema columns + AppealRequesterRole enum (task 1)`

## 3. Required reading

Read in this order before writing any SQL:

1. **Plan §13 task 0** — full pre-flight probe list. Run all 9 probes. STOP if any fail.
2. **Plan §13 task 1** — exact ACTION steps + GOTCHAs (timestamp ordering, `IF EXISTS` on down.sql, no back-dating).
3. **Plan §10.1** — the verbatim SQL. Both up.sql files (enum + ALTER) and the down.sql reversal shape.
4. **Plan §4 (Solution statement) + §11 (Files to change)** — for the architectural watchpoints (ADR-008, ADR-010, ADR-015 implications of the `winning_decision` column write path).
5. **MIRROR refs** (open and skim):
   - `migrations/2026-04-23-000050-0000_add_jury_constraint_relaxation_reason_enum/up.sql`
   - `migrations/2026-04-23-000050-0000_add_jury_constraint_relaxation_reason_enum/down.sql`
   - `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql`
   - `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/down.sql`
6. **Lessons** (Glob `.claude/lessons/`, Read any with filename keywords matching `migration` / `lemmy` / `cargo-output-capture` / `pipes-mask-exit-codes` / `pq-sys` / `pre-phase-dod`):
   - `feedback_lemmy_migration_runner.md` (forbid `diesel migration run`)
   - `feedback_pipes_mask_exit_codes.md` (capture-then-tail, never pipe cargo)
   - `feedback_pq_sys_stale_cache.md` (if migration build fails post-libpq install)
   - `feedback_pre_phase_dod_smoke_test.md` (informs the task-0 probe shape)
   - Any `feedback_features_full_*` lessons

## 4. Constraints

- **Branch:** you start on a Junior worktree branched off `phase-v1-JM-d` (the EliteDesk's main checkout was switched to `phase-v1-JM-d` at advisor commit `2c561beaa`; new tasks inherit `baseBranch: phase-v1-JM-d` per `cli/task.ts:40` `getCurrentBranch()` semantics — see DQ #53 addendum). Do NOT push directly to `phase-v1-JM-d`; finalize will merge your worktree branch back.
- **Timestamp ordering** (GOTCHA from §10.1): generate the prefix at impl time via `date -u +'%Y-%m-%d-%H%M%S-0000'` and verify uniqueness against `ls migrations/`. The prefix MUST sort AFTER `2026-04-23-000200-0000`. Do NOT hard-code 2026-04-26 or 2026-04-27 — let `date` resolve it.
- **No raw `diesel migration run`** — `cargo run -p lemmy_diesel_utils --features full` is the ONLY supported runner per `feedback_lemmy_migration_runner.md`. Both apply and revert.
- **Cargo output capture:** every cargo invocation goes to `.claude/PRPs/debug/v1-JM-d-task1-<probe>.log` with `> file 2>&1`. Never pipe through tail/head/grep — `feedback_pipes_mask_exit_codes.md` is load-bearing here. Read the log AFTER the command; check `$?` separately.
- **`--features full` cannot combine with `-p <crate>`** per `feedback_features_full_p_crate_incompatible.md`. Migration runner uses `-p lemmy_diesel_utils --features full` which is the documented exception (the `lemmy_diesel_utils` crate's feature set permits `full`); do not generalise this to other -p invocations.
- **down.sql idempotency:** use `DROP COLUMN IF EXISTS` and `DROP TYPE IF EXISTS appeal_requester_role` (reverse order: columns first, then type). Round-trip the migrations to verify down.sql works.
- **No backfill SQL** in either up.sql. The plan explicitly states pre-JM-d rows are correct under the new defaults via the metadata-only attmissingval path.
- **Mid-task DQ visibility** (per `.claude/CLAUDE.md` cheatsheet): if you hit a question that needs advisor input mid-task, write a `pending` entry with `from: "impl"` and **commit + push immediately** to your worktree branch — finalize won't push for hours. Slug example: `chore(decision-queue): impl raised DQ #<id> — task1 schema gotcha`.
- **No `answered_by: "advisor"` or `"user"`** from this subagent. Self-resolve only as `"impl-self-resolved"` per `.claude/rules/decision-queue.md` Attribution integrity.
- **Lesson trailer** (optional, retroable per `feedback_junior_pmd_write_convention.md`): if you discover something a future migration task would have wanted to know — e.g. an unstated `--features` quirk on the migration runner, a Postgres version gotcha for the attmissingval path, a `cargo run -p lemmy_diesel_utils` exit-code surprise — end the commit body with one `LESSON:` line. Skip for routine progress.

## 5. Validation gates (per plan §13 task 1 VALIDATE block)

Run all four. Capture each to `.claude/PRPs/debug/v1-JM-d-task1-<probe>.log`. Each must exit 0 except where noted.

1. `cargo run -p lemmy_diesel_utils --features full > .claude/PRPs/debug/v1-JM-d-task1-migrate.log 2>&1` → exit 0; tail shows both new migration dirs applied.
2. `cargo run -p lemmy_diesel_utils --features full -- revert --limit=2 > .claude/PRPs/debug/v1-JM-d-task1-revert.log 2>&1` → exit 0.
3. `cargo run -p lemmy_diesel_utils --features full > .claude/PRPs/debug/v1-JM-d-task1-reapply.log 2>&1` → exit 0.
4. `PGPASSWORD=password psql -h localhost -U lemmy -d lemmy -c '\d appeal' > .claude/PRPs/debug/v1-JM-d-task1-psql-appeal.log 2>&1` → tail shows `requester_role`, `panel_size_snapshot`, `threshold_count_snapshot`.
5. `PGPASSWORD=password psql -h localhost -U lemmy -d lemmy -c '\d moderation_case' > .claude/PRPs/debug/v1-JM-d-task1-psql-moderation_case.log 2>&1` → tail shows `winning_decision`.

If any validation fails, **STOP** and surface to advisor via DQ — do not patch around it. Schema bugs caught at task 1 are cheap; deferred to task 2+ they cascade.

## 6. Expected output (return to advisor)

A 4-section summary:

```
## Task 1 complete — JM-d schema migrations

**Commit:** <sha> on <worktree-branch>
**Migrations applied:**
  - migrations/<ts>-000000-0000_add_appeal_requester_role_enum/
  - migrations/<ts>-000100-0000_add_appeals_v1_columns/
**Validation:** all 5 probes exit 0; psql confirms columns present
**Next:** advisor queues task 2 (Diesel models + R3 sweep)
```

Plus any DQ-#N references if you raised a pending entry mid-task. The advisor's polling loop will read this on completion-transition and queue task 2.
