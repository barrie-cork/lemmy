---
phase: v1-federation-inbound-a
role: impl-task
task: 7
brief_n: 7
authored: 2026-05-18
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
plan_task: "§13 Task 7 — CREATE federation_inbox_nonce.rs + remote_moderation_label.rs (+ mod.rs append)"
parent_phase_tip: "Task 6 validated tip (Task 7 dispatched only after Task 6 validate-pending-laptop result:pass)"
cohort: "Cohort B (Tasks 6-8) — dispatched SERIAL cap=1 per recovery_plan.step_3 (plan [P] markers advisory-only for rest of phase)"
related_dq: "232 (additive-only shared files), 234 (reuse InstanceId), 230 (helpers in db_schema query module)"
requires: "task 2 (schema.rs table blocks — VALIDATED-PASS) + task 5 (newtypes.rs RemoteModerationLabelId — VALIDATED-PASS)"
---

# [role:impl-task] v1-federation-inbound-a Task 7 — federation_inbox_nonce + remote_moderation_label Diesel models — see .claude/PRPs/briefs/federation-inbound-a-impl-7.md

> **Cohort/serial provenance:** Cohort B runs strictly serial (concurrency_cap=1) per `recovery_plan.step_3` (post 5-way SIGTERM catch-fire; user-directed). The plan §13 `[P]` marker on Task 7 is advisory-only. Task 7 is dispatched ONLY after Task 6 reached `validate-pending-laptop result:pass` AND Task 6's worker branch was finalize-merged into `phase-v1-federation-inbound-a`. Your base therefore already contains Task 6's `federation_peer.rs` + `federation_inbox_dropped_log.rs` + the Task 6 `mod.rs` append — the "mod.rs overlap with Task 6" the plan §13 GOTCHA warns about is MOOT under serial dispatch (Task 6's append is already committed; you append AFTER it, no concurrent-write race).

> **requires: gate:** Task 7 `requires: task 2 + task 5`. Both VALIDATED-PASS and on `phase-v1-federation-inbound-a` (schema.rs table blocks @ DQ #242; newtypes.rs `RemoteModerationLabelId` @ DQ #246). Dependency satisfied.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is a Junior worktree branched off `phase-v1-federation-inbound-a`. `git merge-base --is-ancestor <Task-6-validated-tip> HEAD` should be true (your base contains Task 6). If on `phase-v1-federation-inbound-a` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ.
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — if inside a forbidden window exit non-zero `FORBIDDEN_WINDOW: <window>`, UNLESS the dispatch description carries `(user-authorised forbidden-window override per DQ #<id>)` (then read that resolved `answered_by: "user"` DQ + proceed). Shape G SUSPENDED — cargo on laptop, window-cargo concern reduced; keep the check.
- **Confirm Task 6 landed:** `grep -n "pub mod federation_peer" crates/db_schema/src/source/governance/mod.rs` MUST return a hit (Task 6's append). If absent → STOP, file `kind: "blocker"` DQ (serial ordering breached — Task 7 dispatched before Task 6 finalize-merged).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a Task 7 — CREATE federation_inbox_nonce.rs + remote_moderation_label.rs Diesel models`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-a Task 7 — see .claude/PRPs/briefs/federation-inbound-a-impl-7.md
```

## §2 Scope

**Produce** (one commit):

- `crates/db_schema/src/source/governance/federation_inbox_nonce.rs` — **CREATE** per plan §10.4: insert-only model + `FederationInboxNonceInsertForm`, PLUS a `delete_older_than` helper annotated `#[allow(dead_code)]` with a TODO comment (per plan §13 Task 7 — the GC helper is defined now, wired later; the `#[allow(dead_code)]` + TODO keeps clippy `-D warnings` green).
- `crates/db_schema/src/source/governance/remote_moderation_label.rs` — **CREATE** per plan §10.4: the full-shape model (`Queryable` + `Selectable` + `Identifiable`) + `RemoteModerationLabelInsertForm`. Imports `RemoteModerationLabelId` from `crates/db_schema/src/newtypes.rs` (Task 5, merged). `local_case_id` is SET-NULL-on-delete (ADR-006) — model it as `Option<...>` per the schema.rs block Task 2 landed.
- `crates/db_schema/src/source/governance/mod.rs` — **APPEND** `pub mod federation_inbox_nonce;` / `pub mod remote_moderation_label;` AFTER Task 6's appended entries (additive, end of list; do NOT reorder).

**Do NOT** in this task:

- Touch `federation_peer.rs` / `federation_inbox_dropped_log.rs` (Task 6, done), the Phase-6 model files (Task 8), `e2e.rs` (Task 9), `schema.rs`/`newtypes.rs` (done).
- Reorder/reformat existing `mod.rs` entries (DQ #232).

**Commit message** (exactly): `feat(v1-federation-inbound-a): federation_inbox_nonce + remote_moderation_label Diesel models (task 7)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries** — DQ #232 (BINDING — `mod.rs` append strictly additive), DQ #234 (reuse `InstanceId` where applicable).
2. **Plan §10.4** — the authoritative `federation_inbox_nonce.rs` + `remote_moderation_label.rs` model/InsertForm shapes + the `delete_older_than` `#[allow(dead_code)]` + TODO pattern (copy verbatim).
3. **Plan §13 "Task 7"** — step list + GOTCHAs (mod.rs overlap [moot under serial]; `#[allow(dead_code)]` on the GC helper).
4. **Task 6's `federation_inbox_dropped_log.rs`** (now on your base) — the canonical sibling for an insert-only governance model in THIS phase. Mirror its `#[derive(...)]` stack + newtype-import path verbatim for `federation_inbox_nonce.rs`. Canonical-schema-first.
5. **A full-shape Phase-6 sibling** (e.g. `remote_sanction_notice.rs`) — mirror for `remote_moderation_label.rs` (Queryable+Selectable+Identifiable + InsertForm conventions).
6. **Lessons** (MANDATORY per §2.4 + plan §13 binding):
   - `.claude/lessons/feedback_newtype_locations_lemmy_db_schema_vs_file.md` — newtypes in `crates/db_schema/src/newtypes.rs`. **Why:** `remote_moderation_label.rs` imports `RemoteModerationLabelId` — import from `crate::newtypes` (the path Task 6's sibling models use), NOT `lemmy_db_schema_file`.
   - `.claude/lessons/feedback_clippy_test_style.md` — clippy `-D warnings` is in §5. **Why:** the `delete_older_than` helper is unused at this point; `#[allow(dead_code)]` + a TODO comment is the codebase-sanctioned way to keep `-D warnings` green for a defined-but-not-yet-wired helper (plan §13 Task 7 mandates this exact shape).
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always).

## §3a Handover from prior cohort + prior Cohort-B task

```yaml
prior_cohort_tasks:
  - task: 5
    summary: "newtypes.rs — RemoteModerationLabelId. remote_moderation_label.rs imports it from crate::newtypes."
  - task: 2
    summary: "schema.rs — federation_inbox_nonce + remote_moderation_label table blocks. Your models' #[diesel(table_name=...)] resolve against these."
prior_cohort_B_task:
  - task: 6
    summary: "federation_peer.rs + federation_inbox_dropped_log.rs created; mod.rs appended 2 entries. federation_inbox_dropped_log.rs is your canonical insert-only sibling. You append mod.rs entries AFTER Task 6's."
```

## §4 Constraints (hard rules)

### Branch + commit discipline

- Junior worktree off the Task-6-validated phase tip. Finalize merges back; do not push to `phase-v1-federation-inbound-a` directly. One commit.
- Mid-task DQ visibility: commit + push immediately if you raise a `pending` entry.
- No `answered_by: "advisor"`/`"user"`. Self-resolve only as `"impl-self-resolved"`.
- **DQ JSON encoding:** write `.claude/decision-queue.json` entries with `ensure_ascii=False` (literal UTF-8 `—`/`§`), `indent=1`, LF, trailing newline. Do NOT ascii-escape (`\uXXXX`) — recurring breach #283/#305/#308/#310; match the file's existing canonical UTF-8 encoding on the phase tip.

### Harness-gap note (per DQ #235)

If the sensitive-file gate blocks `.claude/decision-queue.json`: write `TASK7_VALIDATE_PENDING.json` + `TASK7_ESCALATION.md` at worktree root, commit both, push, STOP. Advisor transcribes.

### Model GOTCHAs (plan §10.4/§13 Task 7 — load-bearing)

- **`delete_older_than` dead-code:** define it on `federation_inbox_nonce.rs` exactly as plan §10.4 specifies, annotated `#[allow(dead_code)]` with a `// TODO(v1-federation-inbound-*): wire nonce GC` comment. It is NOT called yet; the annotation is mandatory or clippy `-D warnings` (§5 cmd2) fails.
- **ADR-006 SET-NULL:** `remote_moderation_label.local_case_id` is nullable (`Option<...>`) — match the `schema.rs` block Task 2 landed (`grep -n "remote_moderation_label" crates/db_schema_file/src/schema.rs` to confirm the column nullability before writing the model field).
- **mod.rs append-only (DQ #232):** append your 2 `pub mod` lines at the END, after Task 6's. Serial dispatch means Task 6's append is already committed on your base — no race; just append after it.

### Plan-cited content may have drifted

§10.4 is the contract. If a model field references a `schema.rs` column that differs from what Task 2 landed, `grep -n` the actual `schema.rs` block on your base to confirm before writing. Column-count mismatch → file a `kind: "blocker"` DQ pending.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

After commit + push your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry with `from: "impl"`, `phase_task: 7`, `branch: <your-worktree-branch>`, `commands[]` **verbatim** (per plan §13 Task 7 "Push and exit"):

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task7-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-a-task7-clippy.log 2>&1"
```

No `kind: "validate-pending"`, no `workflow_run_id` (Shape G suspended). Advisor laptop runs both sequentially (cmd1 before cmd2) + mutates. Gated path → `TASK7_VALIDATE_PENDING.json` per §4. You do not run these (`feedback_pipes_mask_exit_codes.md`).

## §6 Expected output (return to advisor)

```
## Task 7 complete — v1-federation-inbound-a federation_inbox_nonce + remote_moderation_label models

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/db_schema/src/source/governance/federation_inbox_nonce.rs (+insert-only model +delete_older_than #[allow(dead_code)])
  - crates/db_schema/src/source/governance/remote_moderation_label.rs (+full model +InsertForm)
  - crates/db_schema/src/source/governance/mod.rs (+2 pub mod lines, appended after Task 6's)
**validate-pending-laptop DQ:** #<id> raised (cargo-check.bat + cargo-clippy.bat -D warnings)
**Next:** advisor laptop runs §15 cargo, mutates DQ #<id>; serial Cohort B advances to Task 8 on pass
```

## §7 Why this brief differs from the plan

It does not — Task 7's scope is exactly plan §10.4 + §13 Task 7. Additions: (a) §0 forbidden-window + override-honour + Task-6-landed pre-check, (b) §4 harness-gap escalation, (c) §4 DQ-JSON-encoding constraint, (d) §5 explicit `validate-pending-laptop` shape, (e) serial-dispatch provenance (mod.rs overlap moot under serial). Model shapes are §10.4 verbatim.
