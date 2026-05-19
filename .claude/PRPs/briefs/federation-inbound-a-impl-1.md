---
phase: v1-federation-inbound-a
role: impl-task
task: 1
brief_n: 1
authored: 2026-05-16
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
plan_task: "§13 Task 1 — CREATE combined migration (2 enums + 4 tables + ALTER Phase-6 + 11 config seeds)"
parent_phase_tip: e67e794cf (phase-v1-federation-inbound-a @ registry pre-write)
cohort: "Cohort A (Tasks 1-5, 5-way [P]) — dispatched in parallel"
related_dq: "232 (additive-only shared files), 234 (no FederationPeerId)"
---

# [role:impl-task] v1-federation-inbound-a Task 1 — combined migration — see .claude/PRPs/briefs/federation-inbound-a-impl-1.md

> **Clarify provenance:** this brief's parent **planning** brief was clarified via `/brehon-clarify` before the planning task was queued (DQ #230/#231/#232 resolved advisor-mode). DQ #231 (BINDING) = §15 is `validate-pending-laptop` shape (Shape G SUSPENDED until 2026-06-01). DQ #232 (BINDING) = ALL shared-file edits strictly additive. This is an impl-task brief; no clarify-DQ gates it directly — clarify gates planning briefs only.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-a` (you are on the Junior worktree branched off it). If `git branch --show-current` is anything else → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — if inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) exit non-zero with `FORBIDDEN_WINDOW: <window>`. NOTE: Shape G is SUSPENDED (validate-pending-laptop mode per DQ #229/#231/#235) — cargo runs on the LAPTOP not this worker, so the forbidden-window cargo concern is reduced; keep the self-check anyway.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a Task 1 — combined migration: 2 enums + 4 tables + ALTER Phase-6 + 11 config seeds`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-a Task 1 — see .claude/PRPs/briefs/federation-inbound-a-impl-1.md
```

## §2 Scope

**Produce** (one commit):

- `migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql` — **CREATE** per plan §10.1 verbatim: 2 enums (`federation_peer_trust_enum`, `federation_inbox_admin_action_enum`) + 4 tables (`federation_peer`, `federation_inbox_dropped_log`, `federation_inbox_nonce`, `remote_moderation_label`) + ALTER the 2 Phase-6 tables (`remote_sanction_notice`, `federation_attestation` — add the new columns per §10.1) + 11 `INSERT INTO local_site_rate_limit`/config seeds per §10.1.
- `migrations/2026-05-17-000000-0000_add_federation_inbound_v1/down.sql` — **CREATE** the exact inverse per plan §10.1 (DROP tables, DROP enums, revert ALTERs, DELETE config seeds) in reverse dependency order.

**Do NOT** in this task:

- Touch `schema.rs` (Task 2), `config.rs` Rust (Task 3), `governance_log.rs` (Task 4), `newtypes.rs` (Task 5), any Diesel model file (Cohort B Tasks 6-8), or `e2e.rs` (Task 9).
- Add `-- no-transaction` directive — there is **NO** `ALTER TYPE ADD VALUE` in this migration (both enums are freshly `CREATE TYPE`d), so the migration runs in a transaction. (Per plan §13 Task 1 GOTCHA.)

**Commit message** (exactly): `feat(v1-federation-inbound-a): combined migration — 2 enums + 4 tables + ALTER Phase-6 tables + 11 config seeds (task 1)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #232 (BINDING — migration is a NEW directory, inherently collision-isolated; no shared-file concern for Task 1 since it only CREATEs new migration files), DQ #234 (resolved — reuse `InstanceId`, no `FederationPeerId`; relevant because `federation_peer` keys on `instance_id`).
2. **Plan §10.1** — the authoritative `up.sql` + `down.sql` skeletons (copy verbatim; this is the contract).
3. **Plan §13 "Task 1"** — the step list + all GOTCHAs (migration sort order, no-transaction absence, ADR-006 SET-NULL, ADR-015 TEXT urls, JSONB `::JSONB` cast).
4. **PRD §8.2** — referenced verbatim by §10.1 for the table DDL shape.
5. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4 file-class injection — `crates/db_schema/migrations/**` new migration + JSONB present):
   - `.claude/lessons/feedback_lemmy_migration_runner.md` — Lemmy forbids `diesel_cli`; round-trip is `cargo run -p lemmy_diesel_utils --features full`. **Why:** §5 validation runs the migrate-roundtrip wrapper which uses this runner; the migration must be runner-compatible (no `diesel`-CLI-only syntax).
   - `.claude/lessons/feedback_postgres_jsonb_canonicalization.md` — Postgres `::text` on JSONB adds spaces; `serde_json` compact does not. **Why:** `federation_peer.notes JSONB NOT NULL DEFAULT '{}'::JSONB` — the `::JSONB` cast is mandatory (plan §13 Task 1 JSONB GOTCHA); this lesson explains the canonicalization footgun for any later code reading that column.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail rule for the §5 validation commands).

## §3a Handover from prior cohort

(none — Cohort A is the first impl cohort; Task 0 was a pure read-only probe with no commit and no `HANDOVER:` trailer.)

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-a`. Finalize merges your worktree branch back; do not push to `phase-v1-federation-inbound-a` directly.
- One commit. (Task 1 is single-file-group; no clippy/check split concern — only `cargo-check` + migrate-roundtrip in §5.)
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235, resolved fix-daemon — interim escalation-and-transcribe)

If you need to write a DQ entry to `.claude/decision-queue.json` and the Claude Code sensitive-file gate blocks it (same gate that blocked planning Junior #271): (a) write the intended DQ-entry JSON object to a worktree-root file `TASK1_BLOCKER_DQ.json`, (b) write a short `TASK1_ESCALATION.md` naming the issue, (c) commit both at worktree root + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`. **This applies ONLY if you must raise a DQ — the happy path (migration authored, validation passes) has zero `.claude/` writes** because the §5 `validate-pending-laptop` entry is written by the impl-task per its subagent contract; if THAT write is gated, use the same escalation path with `TASK1_VALIDATE_PENDING.json`.

### Migration GOTCHAs (from plan §13 Task 1 — load-bearing)

- **Sort order:** the directory `2026-05-17-000000-0000_add_federation_inbound_v1` MUST sort strictly after `2026-05-10-000300-0000_seed_v1_rt_config_keys`. Task 0 Probe 9 confirmed the `2026-05-17` slot is FREE. Re-verify with `ls migrations/ | sort | tail -3` at task start; if `2026-05-17-...` now collides (another lane landed it), pick `2026-05-18-000000-0000_add_federation_inbound_v1` for BOTH the directory name and any in-file references, and note the substitution in §7 of your task summary.
- **No `-- no-transaction`:** both enums are `CREATE TYPE` (not `ALTER TYPE ADD VALUE`), so the whole migration is transactional. Do NOT add the directive.
- **ADR-006:** `remote_moderation_label.local_case_id` is SET-NULL on delete. ZERO non-NULL inserts of it in `-a` (no seed rows reference a local case).
- **ADR-015:** `actor_url` / `target_url` columns are `TEXT` (pseudonymisation-compatible — never store a resolvable PII handle as a typed FK).
- **JSONB:** `federation_peer.notes JSONB NOT NULL DEFAULT '{}'::JSONB` — the `::JSONB` cast on the default is mandatory; `'{}'` alone is `text` and Postgres will reject the column default type mismatch.

### Plan-cited content may have drifted

§10.1 is the contract. If §10.1's DDL references a Phase-6 column that has since been renamed on the phase branch, `grep -n` the actual `migrations/` Phase-6 migration to confirm the column name before writing the ALTER. If a column count differs from §10.1, file a DQ pending entry — the plan may have drifted.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

**Shape-G suspended until 2026-06-01** (DQ #229/#231/#235). After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry (per `.claude/agents/impl-task.md` "Pre-Shape-G plans" + `.claude/rules/decision-queue.md` `kind: "validate-pending-laptop"`) with `from: "impl"`, `phase_task: 1`, `branch: <your-worktree-branch>`, and `commands[]` set **verbatim** to:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task1-check.log 2>&1"
bash scripts/brehon/migrate-roundtrip.sh > .claude/PRPs/debug/fed-in-a-task1-migrate.log 2>&1
```

Do NOT write `kind: "validate-pending"` and do NOT capture a `workflow_run_id` — there is no GitHub Actions run (workflows disabled until 2026-06-01). The advisor laptop session reads this entry, runs both commands locally, and mutates the entry (`result: pass|fail`).

If the `.claude/decision-queue.json` write is gated by the sensitive-file gate, use the §4 harness-gap escalation path with `TASK1_VALIDATE_PENDING.json` at worktree root.

Per `feedback_pipes_mask_exit_codes.md`: you do not run these commands yourself (the laptop advisor does) — your job is only to author the migration + raise the entry with the commands verbatim.

## §6 Expected output (return to advisor)

```
## Task 1 complete — v1-federation-inbound-a combined migration

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql (+CREATE 2 enums + 4 tables + ALTER 2 Phase-6 + 11 config seeds)
  - migrations/2026-05-17-000000-0000_add_federation_inbound_v1/down.sql (+inverse)
**Migration slot:** 2026-05-17 (or 2026-05-18 if collision — note which)
**validate-pending-laptop DQ:** #<id> raised (commands: cargo-check.bat + migrate-roundtrip.sh)
**Next:** advisor laptop runs §15 cargo, mutates DQ #<id>; Cohort A barrier waits on all 5 tasks
```

Plus any DQ #N references if you raised a blocker mid-task.

## §7 Why this brief differs from the plan

It does not — Task 1's scope is exactly plan §10.1 + §13 Task 1. This brief adds only: (a) §0 forbidden-window self-check wording, (b) §4 harness-gap interim escalation path (applies only if a DQ write is gated), (c) §5 explicit `validate-pending-laptop` shape per DQ #231 (Shape G suspended). The migration DDL itself is §10.1 verbatim — do not deviate.
