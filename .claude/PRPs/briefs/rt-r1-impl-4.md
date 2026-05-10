---
phase: v1-RT-r1
role: impl-task
task: 4
brief_n: 4
authored: 2026-05-10
parallel_cohort: A
---

# [role:impl-task] v1-RT-r1 task 4 seed 26 net-new RT config keys — see .claude/PRPs/briefs/rt-r1-impl-4.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r1 task 4 seed 26 governance_config rows + feature flag`

## §2 Scope

CREATE migration directory `migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/` with `up.sql` + `down.sql` per plan §10.4 skeleton **verbatim**.

Up.sql contains a single `INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES (...)` statement with **26 net-new rows** + `ON CONFLICT (scope, key, valid_from) DO NOTHING` per plan §10.4.

**Net-new count is 26, NOT 29** (per planner DQ #187). PRD §8 lists 29 conceptual rows but 3 are already shipped under v1-AD-a's seed migration `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql:37-39`:
- `deltas.participation_weekly_active` (default 1)
- `participation.dormancy_window_days` (default 30)
- `deltas.participation_dormant` (default -2)

**Junior MUST NOT include these 3 keys in the RT-r1 seed.** Re-seeding via INSERT ON CONFLICT DO NOTHING would silently swallow them at runtime, but the count drift would break the parity test (`every_seeded_key_has_metadata` enforces 1-to-1 length match between `SEEDED_KEYS_WITH_CONSTS` and `CONFIG_KEY_METADATA` — duplicate keys in the v1-RT-r1 block fail).

**26 rows breakdown** (per plan §10.4 verbatim):
- 8 `decay.<dimension>.<direction>_half_life_days` (int)
- 8 `bounds.<dimension>.<floor|ceiling>` (int)
- 1 `deltas.participation_juror_aligned` (int) — NOT participation_weekly_active (v1-AD-a-owned)
- 2 `participation.activity_threshold_comments` + `participation.lookback_days` (int) — NOT dormancy_window_days (v1-AD-a-owned)
- 2 `deltas.evidence_cited` + `deltas.evidence_bad_faith` (int)
- 1 `participation.evidence_cited_rationale_threshold_chars` (int)
- 2 `job.participation_interval_days` + `job.rollup_interval_days` (int)
- 1 `job.rollup_equal_weights` (bool)
- 1 `feature.reputation_v1_decay_enabled` (bool, default `false`)

**Total: 8 + 8 + 1 + 2 + 2 + 1 + 2 + 1 + 1 = 26.**

Down.sql performs LIFO `DELETE FROM governance_config WHERE scope='instance' AND key IN (...26 keys...) AND value_text IS NULL` (or matched by key alone — planner-judgment per mirror).

### §2.1 Pre-commit reconciliation gate (mandatory per plan §13 Task 4)

Before commit, Junior runs:
```bash
# Count INSERT rows in Task 4 up.sql
grep -cE "^\s*\('instance'," migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/up.sql
# EXPECT: 26
```
If count ≠ 26, fix up.sql before committing. The reconciliation gate prevents drift across SEEDED_KEYS_WITH_CONSTS (Task 8) / DEFAULT_* / match arms / CONFIG_KEY_METADATA / migration up.sql / migration down.sql — all at exactly 26.

**Authority trail comments at top of up.sql:** PRD §8 (29 rows) MINUS 3 v1-AD-a-shipped per planner DQ #187; plan §10.4 Task 4; DQ #185 (advisor logical count 29) + DQ #187 (planner net-new 26).

## §3 Required reading

- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §10.4 (skeleton verbatim — 26-row INSERT block + per-key value_int defaults) + §13 Task 4 (FILES YAML, GOTCHAs, pre-commit reconciliation gate)
- `.claude/PRPs/prds/v1-reputation-tuning.prd.md` §8 Defaults Matrix (29 conceptual rows; per-row value comes from "Default" column)
- `.claude/decision-queue.json` resolved entries **DQ #185** + **DQ #187** — count revision context
- `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql` (v1-AD-a — mirror primary; lines 37-39 are the 3 RT-r1 must-skip duplicates)
- `migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql` (v1-JM-a — mirror secondary)
- `.claude/lessons/feedback_lemmy_migration_runner.md`
- `.claude/lessons/feedback_postgres_jsonb_canonicalization.md` — cite-only
- `.claude/rules/decision-queue.md` — Recipe 1

## §3a Handover from prior cohort

Task 0 pre-flight passed (#195). Probe 6 confirmed the 3 v1-AD-a duplicate keys present at `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql:37-39` (count = 3 per planner DQ #187). Phase branch tip = `80e2e5d0c`.

## §4 Constraints

- **Per DQ #187: 26 rows, NOT 29.** Junior MUST exclude `deltas.participation_weekly_active`, `participation.dormancy_window_days`, `deltas.participation_dormant` from the INSERT VALUES list. Re-seeding them would inflate the parity-test invariant.
- **Pre-commit reconciliation gate is MANDATORY.** Run the grep -c gate before commit; fix drift before commit. Per plan §13 Task 4.
- **Cumulative invariant (Task 8 will assert this):** `34 + 27 + 27 + 13 + 26 = 127`. RT-r1 ships 26 net-new keys; cumulative `SEEDED_KEYS_WITH_CONSTS.len() == 127` post-merge.
- **Per-key value defaults from PRD §8 Default column:** copy verbatim. NOT from PRD §5 prose — the `Default` column is the single source.
- **Migration timestamp ordering:** `2026-05-10-000300-0000` sorts after Task 1, Task 2, Task 3. Re-verify at task start.
- **No Rust edits.** Task 8 owns `EXPECTED_SEED_COUNT_V1_RT_NETNEW` const + parity-test extension. Task 4 is SQL-only.
- **DQ mid-task push rule:** Shape G — 2 `kind: "validate-pending"` DQ entries on push.
- **Attribution:** `from: "impl"`.

## §4.1 CANONICAL CASE OVERRIDE

Not applicable — SQL migration only.

## §5 Concurrency note

Cohort A 4-way parallel. Task 4 has the highest sub-task count (26 INSERT rows + reconciliation gate) but is mechanical pattern-matching from §10.4 skeleton. Independent of Tasks 1, 2, 3 — each in its own worktree.

## §6 COMMIT MESSAGE

`feat(v1-RT-r1): seed 26 net-new reputation-tuning governance_config rows per DQ #187 (task 4)`

Add HANDOVER YAML trailer.
