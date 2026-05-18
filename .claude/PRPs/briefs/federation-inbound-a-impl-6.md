---
phase: v1-federation-inbound-a
role: impl-task
task: 6
brief_n: 6
authored: 2026-05-18
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
plan_task: "§13 Task 6 — CREATE federation_peer.rs + federation_inbox_dropped_log.rs (+ mod.rs append)"
parent_phase_tip: 7fbf92dbd (phase-v1-federation-inbound-a @ Cohort A barrier 5/5 complete)
cohort: "Cohort B (Tasks 6-8) — dispatched SERIAL cap=1 per recovery_plan.step_3 (the plan's [P] markers are advisory-only for the rest of this phase, post 5-way SIGTERM catch-fire; user-directed strict-serial)"
related_dq: "230 (trust helpers in lemmy_db_schema query module), 232 (additive-only shared files), 234 (reuse InstanceId, no FederationPeerId)"
requires: "task 2 (schema.rs table blocks — VALIDATED-PASS) + task 5 (newtypes.rs FederationInboxDroppedLogId — VALIDATED-PASS)"
---

# [role:impl-task] v1-federation-inbound-a Task 6 — federation_peer + dropped_log Diesel models — see .claude/PRPs/briefs/federation-inbound-a-impl-6.md

> **Cohort/serial provenance:** Cohort A's 5-way [P] dispatch SIGTERM-cascaded into a catch-fire (2026-05-16). User directed strict-serial re-dispatch (concurrency_cap=1). Cohort A completed 5/5 serially. Per `recovery_plan.step_3`, **Cohort B (Tasks 6,7,8) ALSO runs strictly serial** — exactly ONE Junior worker at a time. The plan §13 `[P]` markers on Tasks 6/7/8 are advisory-only for the rest of this phase. This brief is dispatched alone; Task 7 is queued only after Task 6 reaches `validate-pending-laptop result:pass`.

> **requires: gate (per `feedback_cohort_validation_dependency_check.md`):** Task 6 `requires: task 2 + task 5`. BOTH are merged on `phase-v1-federation-inbound-a` and VALIDATED-PASS (schema.rs new table blocks @ DQ #242 pass; newtypes.rs `FederationInboxDroppedLogId` @ DQ #246 pass). Dependency satisfied — Task 6 builds against a phase tip that already has both.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is a Junior worktree branched off `phase-v1-federation-inbound-a`. `git log -1 --format=%H` then `git merge-base --is-ancestor 7fbf92dbd HEAD` should be true (your base contains the Cohort A barrier). If `git branch --show-current` is `phase-v1-federation-inbound-a` itself or anything not a `junior/*` worktree → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — if inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) exit non-zero with `FORBIDDEN_WINDOW: <window>`, UNLESS this task's dispatch description carries a `(user-authorised forbidden-window override per DQ #<id>)` annotation — in that case read that DQ entry, confirm it is resolved with `answered_by: "user"`, and proceed. NOTE: Shape G is SUSPENDED (validate-pending-laptop per DQ #229/#231/#235) — cargo runs on the LAPTOP not this worker, so the forbidden-window cargo concern is reduced; keep the self-check anyway.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a Task 6 — CREATE federation_peer.rs + federation_inbox_dropped_log.rs Diesel models + 2 trust-state helpers`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-a Task 6 — see .claude/PRPs/briefs/federation-inbound-a-impl-6.md
```

## §2 Scope

**Produce** (one commit):

- `crates/db_schema/src/source/governance/federation_peer.rs` — **CREATE** per plan §10.4: the `FederationPeer` Diesel model (`Queryable` + `Selectable` + `Identifiable`), `FederationPeerInsertForm`, `FederationPeerUpdateForm`, and the **2 trust-state helpers** per DQ #230 (`federation_inbox_check_peer_trust` + the `federation_peer` write helper). Keys on `instance_id` (`InstanceId`, NOT a new `FederationPeerId` — DQ #234, `federation_blocklist` precedent). `notes JSONB` column — see §4 JSONB GOTCHA.
- `crates/db_schema/src/source/governance/federation_inbox_dropped_log.rs` — **CREATE** per plan §10.4: insert-only model + `FederationInboxDroppedLogInsertForm`. Imports `FederationInboxDroppedLogId` from `crates/db_schema/src/newtypes.rs` (Task 5, already merged).
- `crates/db_schema/src/source/governance/mod.rs` — **APPEND** the two new `pub mod federation_peer;` / `pub mod federation_inbox_dropped_log;` lines (additive, at the end of the existing governance module list — do NOT reorder existing entries; DQ #232).

**Do NOT** in this task:

- Touch `federation_inbox_nonce.rs` / `remote_moderation_label.rs` (Task 7), the Phase-6 model files (Task 8), `e2e.rs` (Task 9), `schema.rs` (Task 2, done), `newtypes.rs` (Task 5, done).
- Reorder or reformat existing `mod.rs` entries or any sibling-lane block (DQ #232 — strictly additive append).

**Commit message** (exactly): `feat(v1-federation-inbound-a): federation_peer + dropped_log Diesel models + trust-state helpers (task 6)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #230 (BINDING — trust helpers go in the `lemmy_db_schema` query module `crates/db_schema/src/source/governance/federation_peer.rs` per plan §10.5, NOT `crates/apub/apub`), DQ #234 (BINDING — reuse `InstanceId`, NO `FederationPeerId` newtype), DQ #232 (BINDING — `mod.rs` append strictly additive, no reorder).
2. **Plan §10.4** — the authoritative `federation_peer.rs` + `federation_inbox_dropped_log.rs` model/InsertForm/UpdateForm shapes (copy verbatim; this is the contract).
3. **Plan §10.5** — the trust-state helper signatures + bodies (`federation_inbox_check_peer_trust`, the `federation_peer` write helper) — DQ #230 binds these into `federation_peer.rs`.
4. **Plan §13 "Task 6"** — the step list + GOTCHAs (trust helpers location; mod.rs append-only).
5. **A canonical sibling Diesel model under `crates/db_schema/src/source/governance/`** — read one existing Phase-6 model file (e.g. `remote_sanction_notice.rs` or `federation_attestation.rs`) BEFORE writing, to mirror the exact `#[derive(...)]` stack, `#[diesel(table_name = ..., check_for_backend(diesel::pg::Pg))]`, and InsertForm/UpdateForm conventions this codebase uses. Canonical-schema-first.
6. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4 file-class injection + plan §13 lesson-binding):
   - `.claude/lessons/feedback_postgres_jsonb_canonicalization.md` — Postgres `::text` on JSONB adds spaces; `serde_json` compact does not. **Why:** `federation_peer.notes JSONB NOT NULL DEFAULT '{}'::JSONB` — the model's `notes` field + any later code reading it must respect this canonicalization footgun (the column default cast `::JSONB` was set in the Task 1 migration; this lesson governs the Rust side).
   - `.claude/lessons/feedback_newtype_locations_lemmy_db_schema_vs_file.md` — newtypes live in `crates/db_schema/src/newtypes.rs` (NOT `lemmy_db_schema_file`). **Why:** `federation_inbox_dropped_log.rs` imports `FederationInboxDroppedLogId` — import it from `crates::db_schema::newtypes` (or the crate-local `crate::newtypes` path the sibling models use), NOT `lemmy_db_schema_file`.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail rule; you do not run the §5 commands yourself, the laptop advisor does).

## §3a Handover from prior cohort

Cohort A (Tasks 1-5) all VALIDATED-PASS on `phase-v1-federation-inbound-a` (barrier 5/5 complete @ `7fbf92dbd`). Relevant prior-cohort outputs you build on:

```yaml
prior_cohort_tasks:
  - task: 2
    commit: "on phase tip (DQ #242 pass)"
    summary: "schema.rs — 4 new table blocks (federation_peer, federation_inbox_dropped_log, federation_inbox_nonce, remote_moderation_label) + extended Phase-6 blocks. Your model's #[diesel(table_name = ...)] resolves against these."
  - task: 5
    commit: "on phase tip (DQ #246 pass)"
    summary: "newtypes.rs — FederationInboxDroppedLogId + RemoteModerationLabelId. federation_inbox_dropped_log.rs imports FederationInboxDroppedLogId from here."
```

Task 6 is the first Cohort B task; no Cohort B handover precedes it.

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-a` (post-Cohort-A-barrier tip `7fbf92dbd` or later). Finalize merges your worktree branch back; do not push to `phase-v1-federation-inbound-a` directly.
- One commit. (Task 6 is single-file-group; the §5 validation is cargo-check + clippy only.)
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.
- **DQ JSON encoding (per recurring breach #283/#305/#308/#310):** when you write the §5 `validate-pending-laptop` entry (or any DQ entry) to `.claude/decision-queue.json`, write it with `ensure_ascii=False` (literal UTF-8: `—`, `§`), `indent=1`, LF newlines, trailing newline. Do NOT write ascii-escaped (`\uXXXX`) — that forces the advisor to whole-file re-encode on mutation. Match the existing file's encoding (it is canonical UTF-8 on the phase tip).

### Harness-gap note (per DQ #235, resolved fix-daemon — interim escalation-and-transcribe)

If the Claude Code sensitive-file gate blocks writing `.claude/decision-queue.json` (same gate that blocked planning Junior #271): (a) write the intended DQ-entry JSON object to a worktree-root file `TASK6_VALIDATE_PENDING.json`, (b) write a short `TASK6_ESCALATION.md` naming the issue, (c) commit both at worktree root + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`. **Happy path (models authored, no DQ-write gate) has the §5 entry written normally.**

### Model GOTCHAs (from plan §10.4/§10.5/§13 Task 6 — load-bearing)

- **DQ #230 (trust helpers location):** `federation_inbox_check_peer_trust` + the `federation_peer` write helper go in `federation_peer.rs` (the `lemmy_db_schema` query module), NOT `crates/apub/apub`. Plan §10.5 binds this.
- **DQ #234 (no FederationPeerId):** `FederationPeer` keys on `instance_id: InstanceId`. There is NO `FederationPeerId` newtype — do not invent one; `federation_blocklist` is the precedent (also keys on `instance_id`).
- **JSONB (`feedback_postgres_jsonb_canonicalization.md`):** the `notes` column is `JSONB NOT NULL DEFAULT '{}'::JSONB`. In the Diesel model, `notes` is `serde_json::Value` (or the codebase's JSONB Rust type — mirror the sibling Phase-6 model that has a JSONB column). The `::JSONB` cast on the default was set in the Task 1 migration; do NOT re-declare it Rust-side. Any helper that compares/hashes `notes` must read PG's rendering, not re-serialize (per the lesson).
- **mod.rs append-only (DQ #232):** add `pub mod federation_peer;` and `pub mod federation_inbox_dropped_log;` at the END of the existing `pub mod` list in `crates/db_schema/src/source/governance/mod.rs`. Do NOT reorder or reformat the existing entries — Task 7 will append its own two modules after yours; finalize-merge resolves the append trivially.

### Plan-cited content may have drifted

§10.4/§10.5 are the contract. If §10.4's model shape references a `schema.rs` table column that differs from what Task 2 actually landed, `grep -n` the actual `crates/db_schema_file/src/schema.rs` `federation_peer` / `federation_inbox_dropped_log` blocks (on your phase-tip base) to confirm column names + types before writing the model. If a column count differs from §10.4, file a `kind: "blocker"` DQ pending — the plan may have drifted from what Task 2 shipped.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

**Shape-G suspended until 2026-06-01** (DQ #229/#231/#235). After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry (per `.claude/agents/impl-task.md` "Pre-Shape-G plans" + `.claude/rules/decision-queue.md` `kind: "validate-pending-laptop"`) with `from: "impl"`, `phase_task: 6`, `branch: <your-worktree-branch>`, and `commands[]` set **verbatim** to (per plan §13 Task 6 "Push and exit"):

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task6-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-a-task6-clippy.log 2>&1"
```

Do NOT write `kind: "validate-pending"` and do NOT capture a `workflow_run_id` — there is no GitHub Actions run (workflows disabled until 2026-06-01). The advisor laptop session reads this entry, runs both commands locally (sequentially — cmd1 must pass before cmd2), and mutates the entry (`result: pass|fail`).

If the `.claude/decision-queue.json` write is gated, use the §4 harness-gap escalation path with `TASK6_VALIDATE_PENDING.json` at worktree root.

Per `feedback_pipes_mask_exit_codes.md`: you do not run these commands yourself — your job is only to author the 3 files + raise the entry with the commands verbatim.

## §6 Expected output (return to advisor)

```
## Task 6 complete — v1-federation-inbound-a federation_peer + dropped_log models

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/db_schema/src/source/governance/federation_peer.rs (+model +InsertForm +UpdateForm +2 trust helpers per DQ #230)
  - crates/db_schema/src/source/governance/federation_inbox_dropped_log.rs (+insert-only model)
  - crates/db_schema/src/source/governance/mod.rs (+2 pub mod lines, appended)
**validate-pending-laptop DQ:** #<id> raised (commands: cargo-check.bat + cargo-clippy.bat -D warnings)
**Next:** advisor laptop runs §15 cargo, mutates DQ #<id>; serial Cohort B advances to Task 7 on pass
```

Plus any DQ #N references if you raised a blocker mid-task.

## §7 Why this brief differs from the plan

It does not — Task 6's scope is exactly plan §10.4 + §10.5 + §13 Task 6. This brief adds only: (a) §0 forbidden-window self-check + override-annotation honour wording, (b) §4 harness-gap interim escalation path (applies only if a DQ write is gated), (c) §4 DQ-JSON-encoding constraint (per the recurring ascii-escape breach #283/#305/#308/#310), (d) §5 explicit `validate-pending-laptop` shape per DQ #231 (Shape G suspended), (e) the serial-dispatch provenance note (recovery_plan.step_3 overrides the `[P]` marker). The model DDL/helper shapes are §10.4/§10.5 verbatim — do not deviate.
