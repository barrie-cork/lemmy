---
phase: chore/refactor-valid-from
role: impl-task
task: refactor
brief_n: 1
authored: 2026-05-14
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_finding: 3.D.6 (rank 6)
parent_phase_tip: <set by bm-cut — branch tip is governance-v0 HEAD at bm-cut time>
---

# [role:impl-task] chore/refactor-valid-from — pin valid_from on v0 + v1-AD-a seed migrations — see .claude/PRPs/briefs/refactor-valid-from-impl.md

## §1 Role + dispatch

`[role:impl-task] chore/refactor-valid-from — pin valid_from literal on 2 seed migrations per audit §3.D.6`

## §2 Scope

### §2.1 Driving audit finding

Audit §3.D.6 (rank 6, severity MED, effort S, frequency 2):

> `2026-04-22-000300-0000_seed_v1_config_keys/up.sql:53` AND `2026-04-18-000000-0000_add_governance_config/up.sql:73-113` · Lens 2 · Axis-4 quality-fail · **[MED]** seed migrations use `ON CONFLICT (scope, key, valid_from) DO NOTHING` with `valid_from` defaulting to `now()`; idempotency is only true within the same transaction (re-running the migration hours later inserts new rows with different valid_from) · `2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql:34-62` (Phase v1-JM-a) fixes this by pinning `valid_from = '2026-04-23T00:02:00Z'::timestamptz` — gold standard not retro-applied · pin `valid_from` in both pre-v1-JM-a seeds to literal timestamptz values; mirror in down.sql · **S** · 2 files.

### §2.2 The canonical pattern (from `migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql`)

Header comment:

```sql
-- Idempotency: every row pins `valid_from` to a STABLE LITERAL — the
-- migration-authored timestamptz of the v1-JM-a seed wave. Together with
-- (scope, key, valid_from) and `ON CONFLICT DO NOTHING` is a true no-op.
-- Without the literal, `valid_from` defaults to `now()` and each rerun
```

Column list + values:

```sql
INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text, valid_from) VALUES
    ('instance', 'jury.panel_size.regular.minor', 'int', 5, NULL, NULL, NULL, '2026-04-23T00:02:00Z'::timestamptz),
    ...
ON CONFLICT (scope, key, valid_from) DO NOTHING;
```

### §2.3 File 1 — `migrations/2026-04-18-000000-0000_add_governance_config/up.sql`

**Current shape (pre-audit, lines 73-113 approximately):**

```sql
INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
    ('instance', 'thresholds.jury_reliability',           'int',  50,         NULL,    NULL, NULL),
    ('instance', 'thresholds.reporting_accuracy',         'int',  50,         NULL,    NULL, NULL),
    ('instance', 'thresholds.endorsement_strength',       'int',  25,         NULL,    NULL, NULL),
    -- ... ~27 rows of seed config ...
```

**Fix:**

1. Add `valid_from` to the column list.
2. Append `'2026-04-18T00:00:00Z'::timestamptz` to every row's value list (one literal value reused across all rows — the v0 seed wave timestamp).
3. Add the canonical idempotency header comment block (mirror the v1-JM-a wording, substituting the v0 timestamp).
4. Pre-flight `grep -c "^    ('" migrations/2026-04-18-000000-0000_add_governance_config/up.sql` to count rows before edit; same count after edit; verify equal.

**Down.sql:** check `migrations/2026-04-18-000000-0000_add_governance_config/down.sql` — if it has key-based DELETEs scoped by `valid_from`, they remain correct (the pinned literal matches). If the down doesn't reference `valid_from`, no change needed.

### §2.4 File 2 — `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql`

**Current shape:**

```sql
INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
    ('instance', 'jury.severity_thresholds.minor',                     'text', NULL,  NULL, NULL,  'majority'),
    ('instance', 'jury.severity_thresholds.moderate',                  'text', NULL,  NULL, NULL,  '60%'),
    -- ...
ON CONFLICT (scope, key, valid_from) DO NOTHING;
```

**Fix:** same shape as File 1.

1. Add `valid_from` to column list.
2. Append `'2026-04-22T00:03:00Z'::timestamptz` to every row's value list (the v1-AD-a seed wave timestamp; matches the migration's directory name).
3. Add canonical idempotency header comment block.
4. Row-count check pre + post edit.

**Down.sql:** same audit as File 1; update if needed.

### §2.5 Post-edit verification

```bash
# Both files now declare valid_from in the column list
grep -E "INSERT INTO governance_config.*valid_from" migrations/2026-04-18-000000-0000_add_governance_config/up.sql migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql

# Both files use literal timestamptz (no DEFAULT now())
grep -c "::timestamptz" migrations/2026-04-18-000000-0000_add_governance_config/up.sql migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql
# Expected: each file has at least (number_of_rows + 0) hits if every row has its own literal

# Row count preserved per file
# (compare pre + post; use the audit's row count as ground truth)
```

### §2.6 Validation gate (Shape G)

Push the worker branch. Both `cargo-validate-workspace.yml` AND `cargo-validate-migration.yml` will trigger (path filter `migrations/**` matches the latter).

**Migration round-trip is the load-bearing signal** — the `phase1_migrations_round_trip` test in `crates/server/tests/e2e.rs` re-applies + reverts these migrations. If valid_from literals are wrong (e.g. accidentally same literal across both files creates a unique-constraint collision), the round-trip will fail.

Capture both workflow_run_ids. Raise TWO `kind: "validate-pending"` DQ entries (one per workflow) per Recipe 1. Both must mutate to `result: "pass"` before PR opens.

## §3 Required reading

- `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` §3.D.6 — driving finding; §3.D.7 — canonical exemplar reference
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` — PR-4 of 6 context
- `migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql` — canonical pattern to mirror
- `.claude/agents/impl-task.md` — subagent contract
- `.claude/rules/decision-queue.md` Recipe 1 — DQ entry shape
- `.claude/lessons/feedback_postgres_jsonb_canonicalization.md` — adjacent migration-class lesson on text-rendering invariants
- `.claude/lessons/feedback_lemmy_migration_runner.md` — Diesel CLI usage rules

## §4 Constraints

- **Files:** ONLY the 2 specified migrations (up.sql and possibly their down.sql counterparts). NO other migrations. NO Rust code changes.
- **Edits:** column-list expansion + per-row literal append + header comment block. No row count changes. No reorderings.
- **Timestamps must be unique across the 2 files** — `2026-04-18T00:00:00Z` for v0 seed, `2026-04-22T00:03:00Z` for v1-AD-a seed. These match the migration directory names. **Do NOT use the same literal in both files** — would create a uniqueness conflict between the two seed waves.
- **Down.sql idempotency:** if down.sql DELETEs reference `valid_from`, they must use the same literal. If they DELETE by `(scope, key)` only, no change needed.
- **Branch:** `chore/refactor-valid-from`.
- **Pre-push cargo-check (mandatory per `feedback_fix_impl_pre_push_cargo_check.md`):**

  ```bash
  bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/refactor-valid-from-precheck.log 2>&1
  status=$?
  tail -20 .claude/PRPs/debug/refactor-valid-from-precheck.log
  [ $status -eq 0 ] || exit $status
  ```

  Per `.claude/rules/cargo-output-capture.md`. This is migration-only — cargo-check should be unaffected. If it fails, file a DQ blocker.

- **Shape G:** push triggers workspace + migration workflows. Raise 2 validate-pending DQ entries.
- **DQ atomic raise + ascii=False + next_id-spans-archives** per standard discipline.
- **COMMIT MESSAGE:** `chore(refactor): pin valid_from literals on v0 + v1-AD-a seed migrations (audit 3.D.6)`

## §5 Out of scope

- The v1-RT-r1 seed migration `2026-05-10-000300-0000_seed_v1_rt_config_keys/up.sql` — verify it already pins valid_from (it likely does post v1-JM-a convention). If it does NOT pin, file a DQ pending entry noting the third occurrence; do NOT fix in this PR (separate finding outside fix-before-next-phase tier).
- Adding indexes on `governance_config(scope, key, valid_from)` — audit §3.D.13+§3.D.14 are separate findings (rank 15, `accept-and-document` tier).
- Refactoring the down.sql to add SAVEPOINT or transaction discipline.
- Renaming the migration directories.

## §6 HANDOVER trailer

This PR runs parallel to PRs 3, 5, 6 on independent branches; no cohort handover. Trailer optional. If added: cite audit 3.D.6 + 3.D.7 (the canonical exemplar) as the key decision source.
