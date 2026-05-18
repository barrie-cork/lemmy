---
phase: v1-federation-inbound-a
role: impl-task
task: 8
brief_n: 8
authored: 2026-05-18
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
plan_task: "§13 Task 8 — EXTEND Phase-6 model files (remote_sanction_notice.rs + federation_attestation.rs + inbox.rs default-spread)"
parent_phase_tip: "Task 7 validated tip (Task 8 dispatched only after Task 7 validate-pending-laptop result:pass)"
cohort: "Cohort B (Tasks 6-8) — dispatched SERIAL cap=1 per recovery_plan.step_3 (plan [P] markers advisory-only for rest of phase)"
related_dq: "232 (additive-only shared files — Phase-6 model + inbox.rs are sibling-lane-shared; strictly additive new fields only)"
requires: "task 2 (schema.rs regenerated Phase-6 blocks — VALIDATED-PASS)"
---

# [role:impl-task] v1-federation-inbound-a Task 8 — extend Phase-6 models (UpdateForm + new columns) — see .claude/PRPs/briefs/federation-inbound-a-impl-8.md

> **Cohort/serial provenance:** Cohort B runs strictly serial (concurrency_cap=1) per `recovery_plan.step_3`. The plan §13 `[P]` marker on Task 8 is advisory-only. Task 8 dispatched ONLY after Task 7 reached `validate-pending-laptop result:pass` AND Task 7's worker branch finalize-merged. Your base contains Tasks 1-7 (all VALIDATED-PASS).

> **requires: gate:** Task 8 `requires: task 2` (schema.rs regenerated Phase-6 table blocks — the new struct fields you add must typecheck against the regenerated `schema.rs`). Task 2 VALIDATED-PASS @ DQ #242, on `phase-v1-federation-inbound-a`. Dependency satisfied.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is a Junior worktree off `phase-v1-federation-inbound-a` containing Tasks 1-7. `grep -n "pub mod remote_moderation_label" crates/db_schema/src/source/governance/mod.rs` MUST hit (Task 7's append). If absent → STOP, file `kind: "blocker"` DQ (serial ordering breached).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>`, UNLESS dispatch description carries `(user-authorised forbidden-window override per DQ #<id>)` (read that resolved `answered_by: "user"` DQ + proceed). Shape G SUSPENDED.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a Task 8 — EXTEND remote_sanction_notice.rs + federation_attestation.rs (+4 fields, +InsertForm fields, +NEW RemoteSanctionNoticeUpdateForm) + inbox.rs default-spread`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-a Task 8 — see .claude/PRPs/briefs/federation-inbound-a-impl-8.md
```

## §2 Scope

**Produce** (one commit) — all edits ADDITIVE (DQ #232; these are sibling-lane-shared Phase-6 files):

- `crates/db_schema/src/source/governance/remote_sanction_notice.rs` — per plan §10.3: add **4 new struct fields** to the `RemoteSanctionNotice` model + **4 new InsertForm fields** to `RemoteSanctionNoticeInsertForm` + a **NEW `RemoteSanctionNoticeUpdateForm`** (AsChangeset). New `Option<_>` fields keep existing `..Default::default()` callers compiling (see §4 R9 + the InsertForm-default lesson).
- `crates/db_schema/src/source/governance/federation_attestation.rs` — per plan §10.3: mirror — **6 new fields each** on the `FederationAttestation` model + `FederationAttestationInsertForm` (plan §10.3 says "6 fields each" for federation_attestation; remote_sanction_notice is "4 new struct fields + 4 new InsertForm fields"). Follow §10.3 exactly for the per-file field counts.
- `crates/apub/activities/src/governance/inbox.rs` — add `..Default::default()` spread to the InsertForm initializer(s) at the R9 callsites **IF MISSING** (only if a callsite constructs the InsertForm without the spread and the new `Option<_>` fields would break it). If the callsite already has `..Default::default()`, no inbox.rs edit is needed.

**Do NOT** in this task:

- Touch the new Cohort-B model files (Tasks 6/7 — done), `schema.rs`/`newtypes.rs` (done), `e2e.rs` (Task 9 — even though it has an R9 callsite; see §4).
- Reorder/reformat existing struct fields or sibling-lane blocks (DQ #232 — append new fields after existing ones, in the position §10.3 specifies).

**Commit message** (exactly): `feat(v1-federation-inbound-a): extend Phase-6 models — UpdateForm + new columns (task 8)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries** — DQ #232 (BINDING — Phase-6 model edits strictly additive; new fields appended, no reorder of existing).
2. **Plan §10.3** — the authoritative new-field lists for `remote_sanction_notice.rs` (4 model + 4 InsertForm + NEW UpdateForm) and `federation_attestation.rs` (6 fields each), the exact field names/types, and the `RemoteSanctionNoticeUpdateForm` AsChangeset shape (copy verbatim; this is the contract).
3. **Plan §13 "Task 8"** — step list + the R9 GOTCHA (RE-VERIFY callers at task start).
4. **The current `remote_sanction_notice.rs` + `federation_attestation.rs`** (on your base) — read both fully BEFORE editing, to see the existing field order, the existing InsertForm `#[derive(Default)]` (or equivalent), and where §10.3 wants the new fields appended. Canonical-schema-first.
5. **The R9 callsites** (see §4 enumeration) — read each callsite BEFORE editing the InsertForms, to confirm whether it already has `..Default::default()`.
6. **Lessons** (MANDATORY per §2.4 + plan §13 binding):
   - `.claude/lessons/feedback_insertform_default_propagation.md` — **LOAD-BEARING for Task 8 + R9.** Phase-6 InsertForms have `#[derive(Default)]`; adding new `Option<_>` fields keeps existing `..Default::default()` callers compiling WITHOUT touching them — *provided the new fields are `Option<_>` and the InsertForm keeps `#[derive(Default)]`*. **Why:** this is exactly the Task 8 mechanism — the §10.3 new InsertForm fields MUST be `Option<_>` (or otherwise Default-able) so the R9 callsites don't all need editing. If §10.3 specifies a non-Option new field, that's a callsite-breaking change → re-verify ALL R9 callsites and edit each (see §4).
   - `.claude/lessons/feedback_postgres_jsonb_canonicalization.md` — if any new field is JSONB, the `::JSONB` default-cast footgun applies (mirror Task 6's handling).
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always).

## §3a Handover from prior cohort + prior Cohort-B tasks

```yaml
prior_cohort_tasks:
  - task: 2
    summary: "schema.rs — regenerated Phase-6 blocks (remote_sanction_notice, federation_attestation) WITH the new columns the Task 1 migration added. Your new model/InsertForm fields typecheck against these regenerated blocks."
prior_cohort_B_tasks:
  - task: 6
    summary: "federation_peer.rs + federation_inbox_dropped_log.rs created."
  - task: 7
    summary: "federation_inbox_nonce.rs + remote_moderation_label.rs created. mod.rs has all 4 Cohort-B model entries."
```

## §4 Constraints (hard rules)

### R9 callsite enumeration (per `feedback_fix_impl_enumerate_all_callsites.md` applied to struct-shape change — RE-VERIFY at task start)

Task 0's pre-flight probe (state file) enumerated the R9 callers as of phase start:

- `RemoteSanctionNoticeInsertForm` — **1 caller:** `crates/apub/activities/src/governance/inbox.rs:138`
- `FederationAttestationInsertForm` — **2 callers:** `crates/apub/activities/src/governance/inbox.rs:213` + `crates/server/tests/e2e.rs:7451`

= **3 callsites across 2 files** (`inbox.rs` ×2, `e2e.rs` ×1).

**RE-VERIFY at task start (mandatory — line numbers drift):** run `rg -n "RemoteSanctionNoticeInsertForm|FederationAttestationInsertForm" crates/ tests/` and confirm the full set. If the count is still 3 callsites / 2 files (inbox.rs + e2e.rs), proceed:

- **inbox.rs callsites:** in Task 8's scope. If the new InsertForm fields are all `Option<_>` and the InsertForm keeps `#[derive(Default)]`, the callsite compiles unchanged IF it already uses `..Default::default()`. If a callsite enumerates all fields explicitly (no spread), add the new fields OR add `..Default::default()`. Plan §13 says "add `..Default::default()` spread in inbox.rs if missing".
- **e2e.rs:7451 callsite:** **Task 8 does NOT edit `e2e.rs`** (that's Task 9's file; editing it here risks the `feedback_junior_worker_e2e_edit_hang` hazard + cross-task file collision). If the e2e.rs:7451 callsite would break from a non-Option new field, that is a **plan-shape problem** — file a `kind: "blocker"` DQ pending citing this brief (the §10.3 fields should be Option-able precisely so e2e.rs:7451 stays compiling without a Task-8 e2e.rs edit; Task 9 will add the new-field coverage). Do NOT edit e2e.rs from Task 8.

If RE-VERIFY finds **>3 callsites or >2 files** (drift since Task 0): STOP, file a `kind: "blocker"` DQ pending with the full `rg` enumeration — the change is no longer the narrow shape the plan assumed; advisor decides scope.

### Branch + commit discipline

- Junior worktree off the Task-7-validated phase tip. Finalize merges back; do not push to `phase-v1-federation-inbound-a` directly. One commit.
- Mid-task DQ visibility: commit + push immediately if you raise a `pending` entry.
- No `answered_by: "advisor"`/`"user"`. Self-resolve only `"impl-self-resolved"`.
- **DQ JSON encoding:** `.claude/decision-queue.json` writes use `ensure_ascii=False` (literal UTF-8 `—`/`§`), `indent=1`, LF, trailing newline. NOT ascii-escaped (recurring breach #283/#305/#308/#310); match the file's canonical encoding on the phase tip.

### Harness-gap note (per DQ #235)

Sensitive-file gate on `.claude/decision-queue.json` → write `TASK8_VALIDATE_PENDING.json` + `TASK8_ESCALATION.md` at worktree root, commit, push, STOP. Advisor transcribes.

### Model GOTCHAs (plan §10.3/§13 Task 8 — load-bearing)

- **Additive only (DQ #232):** new struct fields appended in the position §10.3 specifies; the `RemoteSanctionNoticeUpdateForm` is a NEW struct (additive). Do NOT reorder/retype existing fields — `remote_sanction_notice.rs`/`federation_attestation.rs` are sibling-lane-shared.
- **Option-ability + `#[derive(Default)]` (R9, the load-bearing constraint):** the new InsertForm fields MUST be Default-able (`Option<_>` per §10.3) and the InsertForm MUST retain `#[derive(Default)]`, so existing `..Default::default()` callers compile unchanged. This is the entire reason R9 is "1+2 callers" not "edit-everywhere". Confirm against §10.3's field types.
- **`RemoteSanctionNoticeUpdateForm`:** NEW `#[derive(AsChangeset)]` struct per §10.3 — mirror an existing UpdateForm in the codebase (e.g. a Phase-6 `*UpdateForm` if one exists, else the AsChangeset convention from a sibling `db_schema` model) for the derive stack + `#[diesel(table_name=...)]`.

### Plan-cited content may have drifted

§10.3 is the contract. If a new field references a `schema.rs` Phase-6 column that differs from what Task 2 regenerated, `grep -n` the actual `remote_sanction_notice`/`federation_attestation` blocks in `crates/db_schema_file/src/schema.rs` on your base before writing. Field-count mismatch vs §10.3 → file a `kind: "blocker"` DQ pending (plan may have drifted from Task 2's regenerated schema).

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

After commit + push your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry with `from: "impl"`, `phase_task: 8`, `branch: <your-worktree-branch>`, `commands[]` **verbatim** (per plan §13 Task 8 "Push and exit" — note Task 8 has THREE commands incl e2e --no-run):

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task8-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-a-task8-clippy.log 2>&1"
cmd //c "scripts\brehon\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/fed-in-a-task8-test-norun.log 2>&1"
```

No `kind: "validate-pending"`, no `workflow_run_id` (Shape G suspended). Advisor laptop runs all 3 sequentially (each must pass before the next) + mutates. The `--test e2e --no-run` is a COMPILE check of the e2e harness against the extended Phase-6 models (it does NOT run e2e — Phase 2 e2e execution is Task 9 + user-gate-4). Gated path → `TASK8_VALIDATE_PENDING.json` per §4. You do not run these (`feedback_pipes_mask_exit_codes.md`).

## §6 Expected output (return to advisor)

```
## Task 8 complete — v1-federation-inbound-a extend Phase-6 models

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/db_schema/src/source/governance/remote_sanction_notice.rs (+4 model fields +4 InsertForm fields +NEW RemoteSanctionNoticeUpdateForm)
  - crates/db_schema/src/source/governance/federation_attestation.rs (+6 fields model +6 InsertForm)
  - crates/apub/activities/src/governance/inbox.rs (+..Default::default() spread at R9 callsites, IF it was missing — else "no edit needed")
**R9 re-verify:** <N> callsites / <M> files (expected 3/2: inbox.rs:138, inbox.rs:213, e2e.rs:7451 — e2e.rs NOT edited by Task 8)
**validate-pending-laptop DQ:** #<id> raised (cargo-check.bat + cargo-clippy.bat -D warnings + cargo-test.bat --test e2e --no-run)
**Next:** advisor laptop runs §15 cargo (3 cmds), mutates DQ #<id>; serial Cohort B barrier complete on pass → Task 9 (e2e.rs)
```

Plus any DQ #N references if you raised a blocker (esp. the R9-drift or e2e.rs:7451-would-break blockers).

## §7 Why this brief differs from the plan

It does not — Task 8's scope is exactly plan §10.3 + §13 Task 8. Additions: (a) §0 forbidden-window + override-honour + Task-7-landed pre-check, (b) §4 R9 callsite enumeration (from Task 0 probe) + RE-VERIFY discipline + the explicit "Task 8 does NOT edit e2e.rs" boundary (e2e.rs:7451 is Task 9's; a non-Option field that breaks it is a plan-shape blocker, not a Task-8 e2e edit), (c) §4 harness-gap escalation, (d) §4 DQ-JSON-encoding constraint, (e) §5 explicit 3-command `validate-pending-laptop` shape, (f) serial-dispatch provenance. The field lists/UpdateForm shape are §10.3 verbatim.
