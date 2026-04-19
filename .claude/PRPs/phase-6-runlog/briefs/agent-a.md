# Agent A — Phase 6 Layer 1: Schema + Diesel models

**Model:** opus. **Effort:** maximum — deepest reasoning, most thorough exploration.

**Scope:** Tasks 70 + 71 of `.claude/PRPs/plans/phase-6-federation.plan.md`. Schema migration for `federation_attestation` + `remote_sanction_notice` tables, plus their Diesel `Queryable`/`Insertable` structs and newtype IDs.

You are continuing a Brehon governance-fork Phase 6 layered-execution plan. Read `CLAUDE.md`, `.claude/PRPs/plans/phase-6-federation.plan.md` tasks 70–71, and `.claude/decision-queue.json` DQ-6.1 (advisor-answered: add `received_at`) before starting.

## Worktree

- Advisor has created `../brehon-fork-agent-a-phase6` on branch `agent-a-phase6` cut from `phase-6`.
- Operate exclusively in that worktree. Do NOT touch any other worktree.
- **First command:** `cd ../brehon-fork-agent-a-phase6 && git log --oneline -3` — confirm you are on `agent-a-phase6` at the expected HEAD.
- **Ensure submodules are initialised in this worktree** via `git submodule update --init --recursive` — Lemmy's `crates/email/translations` submodule is not automatically checked out by `git worktree add` and `cargo check --workspace --features full` fails without it. Advisor may have done this; verify with `ls crates/email/translations/backend/ | head`.

## Task-hopper envelope

Before each file edit:
```
scripts/brehon/task-hopper.sh start 70 \
  --agent agent-a --kind migration --layer 1 \
  --label "add_federation_attestations migration" \
  --worktree "$(pwd)"
```

After task 70 commit lands (same worktree):
```
scripts/brehon/task-hopper.sh complete 70 --commit-sha "$(git rev-parse --short HEAD)"
```

Then:
```
scripts/brehon/task-hopper.sh start 71 \
  --agent agent-a --kind cargo_check --layer 1 \
  --label "Diesel models for federation tables" \
  --worktree "$(pwd)"
```
→ implement task 71 → complete 71.

Per `.claude/rules/task-hopper.md`. On failure: `retry` (auto-escalates at the kind's cap). On give-up: `escalate`.

## Task 70 — migration

Create `migrations/{timestamp}_add_federation_attestations/{up,down}.sql`. Timestamp MUST sort after `2026-04-20-000000-0000_add_governance_log_notify/` — use something like `2026-04-21-000000-0000` or the current UTC datetime in `diesel migration generate` format.

**up.sql** — two tables, indexes, per plan §Step-by-Step task 70. Key deviations to apply:

- Include `received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` on `remote_sanction_notice` (DQ-6.1 resolved: add).
- Before writing, run `diesel print-schema` or read an existing migration in `migrations/2026-04-*/up.sql` to confirm enum names. The audit flagged potential drift between design-doc and DB names. Expected: `sanction_action_enum`, `sanction_scope_enum`, `attestation_type_enum`. If any enum is missing, add it in this migration with an up.sql comment header explaining.
- `signature TEXT NOT NULL`. For inbound (stored by Agent E), task 75 populates with `activity.id.to_string()` (DQ-6.2 resolved).
- No partial-unique idempotency index on `remote_sanction_notice` (DQ-6.3 resolved — rely on `ReceivedActivity` dedup).

**down.sql** — reverse order `DROP TABLE remote_sanction_notice; DROP TABLE federation_attestation;` + any created enums.

Commit as `feat(governance): task 70 — add_federation_attestations migration` with the body from plan §task 70 commit template.

### Validate

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema_file > .claude/build-task70.log 2>&1"
tail -20 .claude/build-task70.log
echo "exit: $?"
```

Exit must be 0. Tail must show `Finished` + no errors.

## Task 71 — Diesel models

Create `crates/db_schema/src/source/governance/federation_attestation.rs` and `crates/db_schema/src/source/governance/remote_sanction_notice.rs`. Update `crates/db_schema/src/source/governance/mod.rs` and `crates/db_schema/src/newtypes.rs`.

- Mirror `crates/db_schema/src/source/governance/sanction.rs` — same derive stack, `#[diesel(table_name = ...)]` pattern.
- Newtype IDs in `newtypes.rs`: `FederationAttestationId(pub i32)` and `RemoteSanctionNoticeId(pub i32)`.
- FK typing: `local_case_id: Option<ModerationCaseId>` (NOT `Option<i32>`) — mirror existing newtype-typed FK patterns.
- Include `received_at: DateTime<Utc>` on `RemoteSanctionNotice` (per task 70).
- `RemoteSanctionNotice` has no bare-scalar fields — `Selectable` derive is fine. Same for `FederationAttestation`. No tuple-load fallback needed per `.claude/rules/view-crate-selectable-template.md`.

Also run `diesel print-schema` via the schema-setup binary (see `feedback_lemmy_migration_runner.md`) to update `crates/db_schema/src/schema.rs` if needed. Depending on how the workspace is set up, you may need to hand-write the `table!` additions in `schema.rs` for the two new tables.

Commit as `feat(governance): task 71 — Diesel models for federation tables` with plan-template body.

### Validate

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema > .claude/build-task71.log 2>&1"
tail -20 .claude/build-task71.log
echo "exit: $?"
```

Exit must be 0.

## Catch-fire triggers (stop and escalate via decision-queue)

- If enum names differ from plan assumptions (`sanction_action_enum` / `sanction_scope_enum` / `attestation_type_enum`) — write a DQ entry before proceeding.
- If `sanction.rs` model pattern has drifted from what the plan describes — write a DQ entry.
- If `cargo-check.bat` wrapper returns exit 0 on a file-missing error (regression of issue #8) — escalate, do not continue.
- If workspace cargo-check red after your commits — roll back, do not force-push.

## Commit-message discipline

One commit per task. Ending with `Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>`.

## When done

Push: `git push -u origin agent-a-phase6`. Exit with one-line summary citing commit SHAs. Advisor merges into `phase-6` at merge point 1.
