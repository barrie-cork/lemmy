---
role: impl-task
plan_task: 3
phase: v1-JM-d
created: 2026-04-28
status: ready
related_dq: 81
supersedes_premise: jm-d-fix-impl-3b.md (premise corrected — production bug, not test fixture)
---

# Brief — v1-JM-d Fix 3c — production fix for governance_config latest-wins read

## 1. Role + dispatch line

`[role:impl-task] v1-JM-d fix-3c — see .claude/PRPs/briefs/jm-d-fix-impl-3c.md`

## 2. Scope

**Context:** fix-3b (Task #41, Opus 4.7, partial commit `cd6d74b` already merged into
`phase-v1-JM-d` at `715008574`) shipped Cluster A test fixture seeding for 2 of 7
failures. Opus's diagnosis revealed that **Clusters B + C (5 of the 7 failures) are
not test issues — they're a production bug** introduced by commit `8e3bba1`
("fix(governance): rename governance_config_current → governance_config (schema
cleanup)").

**The bug:** `crates/api/api/src/governance/config.rs:733` `fetch_value_at_scope` reads
from the raw `governance_config` table — which is **append-only with multiple rows per
(scope, key)** keyed by `valid_from` — without ordering. PostgreSQL returns rows in
arbitrary order, so `.first::<ConfigRow>(conn)` may return any row including the seeded
default rather than the most recent `admin_set_config` write. Pre-`8e3bba1`, code read
from a `governance_config_current` view that did the latest-wins ordering server-side;
the rename dropped the view but the reader code wasn't updated.

**Symptoms this explains (5 of 7 fix-3b failures):**

| Test | Observed | Root cause |
|---|---|---|
| `submit_jury_vote_writes_appeal_window_live_config` | gap = 7d (seeded) not 30d (LIVE) | `appeal.window_days` read returned seeded row, not admin write |
| `v0_case_completes_under_v0_rules_after_v1_config_flip` | gap = 7d not 60d (LIVE) | same |
| `admin_get_config_single_key_with_provenance` | `5 ≠ 11` | `jury.panel_size` read returned seeded row |
| `admin_list_rule_sets_returns_versions_with_active_version_id` | `Some(1) ≠ Some(3)` | active_version_id resolves via config read |
| `admin_set_config_persists_previous_value_and_from` | `Some(5) ≠ Some(7)` | previous_value resolves via the same broken read |

**Produce** (one commit on a worktree branch off `governance-v0`):

### Fix — `crates/api/api/src/governance/config.rs`

Add `ORDER BY valid_from DESC` to the SELECT in `fetch_value_at_scope` so that the
most recent insert wins. The current code starting at line 733:

```rust
async fn fetch_value_at_scope(
  pool: &mut DbPool<'_>,
  scope_str: Cow<'static, str>,
  key: &str,
) -> LemmyResult<Option<CachedValue>> {
  let conn = &mut get_conn(pool).await?;

  let row: Option<ConfigRow> = governance_config::table
    .filter(governance_config::scope.eq(scope_str.into_owned()))
    .filter(governance_config::key.eq(key))
    .select((
      governance_config::value_type,
      governance_config::value_int,
      governance_config::value_float,
      governance_config::value_bool,
      governance_config::value_text,
    ))
    .first::<ConfigRow>(conn)
    .await
    .optional()?;
  ...
```

Add `.order_by(governance_config::valid_from.desc())` after the `.select(...)` and
before `.first::<ConfigRow>(conn)`:

```rust
    .select((
      governance_config::value_type,
      governance_config::value_int,
      governance_config::value_float,
      governance_config::value_bool,
      governance_config::value_text,
    ))
    .order_by(governance_config::valid_from.desc())
    .first::<ConfigRow>(conn)
```

**Sweep for other readers of the same table.** Grep `crates/api/` and
`crates/api_crud/` for other call sites that SELECT FROM `governance_config` without
ORDER BY. If any found, apply the same `.order_by(governance_config::valid_from.desc())`
or refactor to use a shared latest-wins helper. Do NOT touch `admin_set_config` or
INSERT paths — only READ paths are broken.

**Add a regression-prevention comment** above the SELECT:

```rust
// governance_config is append-only with multiple rows per (scope, key)
// keyed by valid_from. ORDER BY valid_from DESC + LIMIT 1 (.first) is
// load-bearing — without it Postgres returns arbitrary order and reads
// can return stale seeded rows instead of admin_set_config writes.
// Regression history: commit 8e3bba1 dropped the governance_config_current
// view; this code path needs to do the latest-wins ordering itself.
```

**Do NOT:**

- Edit any e2e test in `crates/server/tests/e2e.rs` (Opus already did Cluster A; the
  remaining 5 tests will pass once this production fix lands).
- Re-add the `governance_config_current` view (the rename was deliberate per `8e3bba1`).
- Touch migration files.
- Run cargo locally (Shape G).

## 3. Required reading

- `crates/api/api/src/governance/config.rs:720-755` — the broken `fetch_value_at_scope`
- `crates/db_schema_file/src/schema.rs:455-468` — `governance_config` table with `valid_from`
- `migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql` — confirms multi-row
  append-only design with `valid_from` timestamps and
  `ON CONFLICT (scope, key, valid_from) DO NOTHING`
- Commit `8e3bba1` (`fix(governance): rename governance_config_current → governance_config
  (schema cleanup)`) — the regression-introducing commit
- Commit `cd6d74b` (Cluster A fix already shipped) — for context on test fixtures already
  patched

## 4. Constraints

- **One commit.** Subject: `fix(governance): order governance_config reads by valid_from DESC (fix-3c)`
- **Push your branch to origin.** Do NOT push to `phase-v1-JM-d` or `governance-v0`.
- **Post-commit:** raise a `kind: "validate-pending"` DQ entry on the
  `governance-v0` branch. Use `gh run list --repo barrie-cork/lemmy --branch <your-branch>
  --workflow cargo-validate-workspace --limit 1 --json databaseId` to capture the
  workflow_run_id from the workspace-check that fires on your push. JSON skeleton:

```json
{
  "id": 82,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<ISO-8601 now>",
  "workflow_run_id": <id from gh run list>,
  "branch": "<your junior/* branch>",
  "phase_task": 3,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

  Append to `pending[]` in `.claude/decision-queue.json` on `governance-v0`.
  Commit subject: `chore(decision-queue): impl raised DQ #82 — validate-pending fix-3c workspace run <id>`
  Push that DQ-update commit to `governance-v0`.

- If the grep sweep reveals **multiple** call sites that read `governance_config` without
  ORDER BY (more than just `fetch_value_at_scope`), fix all of them in the same commit
  but list each in the commit body. Do NOT split into multiple commits.
- If no other call sites need fixing, just fix `fetch_value_at_scope` and ship.
