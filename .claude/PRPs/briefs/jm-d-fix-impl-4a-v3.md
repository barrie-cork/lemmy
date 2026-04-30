---
role: impl-task
plan_task: 4
phase: v1-JM-d
created: 2026-04-29
status: ready
related_dq: 85
supersedes_premise: jm-d-fix-impl-4a-v2.md (fixed unused-import; clippy then caught a separate clone_on_copy in the same handler — this brief fixes that)
---

# Brief — v1-JM-d Fix 4a-v3 — drop `.clone()` on Copy type `AdminTriggerAppealRejury`

## 1. Role + dispatch line

`[role:impl-task] v1-JM-d fix-4a-v3 — see .claude/PRPs/briefs/jm-d-fix-impl-4a-v3.md`

## 2. Scope

**Context:** Task #48 (fix-4a-v2, `5bf306003` — unused-import deletion) shipped successfully, but workspace clippy on workflow run `25081414634` caught a SECOND clippy error in the same Task 4 feat code:

```
error: using `clone` on type `AdminTriggerAppealRejury` which implements the `Copy` trait
  --> crates/api/api/src/governance/admin_trigger_appeal_rejury.rs:47:21
   |
47 |   let data_for_tx = data.clone();
   |                     ^^^^^^^^^^^^ help: try removing the `clone` call: `data`
   |
   = note: `-D clippy::clone-on-copy` implied by `-D clippy::complexity`
```

**Why this happened (do not repeat):** Task #45's worker mirrored `admin_close_case.rs:41` (`let data_for_tx = data.clone();`) verbatim, but `AdminCloseCase` has a `String` field (`reason`) so does not derive `Copy`, while `AdminTriggerAppealRejury` only has `case_id: ModerationCaseId` (a Copy newtype) and does derive `Copy`. Worker missed the type difference. **Lesson for future tasks:** when MIRRORing a `let X_for_tx = X.clone()` pattern, check whether the source type derives `Copy` — `.clone()` on a Copy type is a `clippy::clone_on_copy` deny.

**Laptop pre-validation done:** Advisor session ran `bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings` on the laptop checkout of `45b59da2d` (Task #48 worker tip). Result: this is the **only** clippy error in the workspace; no other lints surfaced before lemmy_api errored out. Single-line fix is sufficient to clear Phase 1.

**This is a §G4 allowlist auto-queue case** — single file, single line, clippy auto-fix class.

**Branching strategy:**

This fix builds on top of #48's fix-4a-v2 work (`45b59da2d` worker branch tip). Branch from there:

```bash
git fetch origin junior/role-impl-task-v1-jm-d-fix-4a-v2-see-claude-prps-briefs-jm-d-fix-impl-4a-v2-md-48
git checkout 45b59da2d -b junior/fix-task-4a-v3-clone-on-copy
```

The daemon's default base branch (governance-v0) does NOT have the unused-import fix yet — branching from there would lose that work. You MUST manually checkout `45b59da2d` first.

**Produce** (one fix commit on top of `45b59da2d`):

### Fix — `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs`

Line 47: change `let data_for_tx = data.clone();` to `let data_for_tx = data;`. Drop the `.clone()` call only — the `let` binding name and surrounding code stay.

Before:
```rust
  let data_for_tx = data.clone();
  let pseudonym_for_tx = admin_pseudonym.clone();
```

After:
```rust
  let data_for_tx = data;
  let pseudonym_for_tx = admin_pseudonym.clone();
```

Note: line 48 (`admin_pseudonym.clone()`) STAYS — `admin_pseudonym` is a `String` (not Copy), so `.clone()` is required there. Do NOT touch line 48.

That is the **only** edit. Do not touch any other line, file, or import.

## 3. Required reading

- `.claude/agents/impl-task.md` — full file. Especially "Per-task validation gate" → push-and-exit discipline. **Hard refusal: do NOT run cargo locally on the EliteDesk worker** (incident task #47 cited inline; same applies to this fix).
- `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs:40-55` (after checkout of `45b59da2d`) — context for line 47
- `crates/api/api_common/src/governance.rs` — search for `AdminTriggerAppealRejury` to confirm it derives `Copy`
- DQ #85 in `.claude/decision-queue.json` on worker branch `junior/...md-48` — the failed Phase 1 entry (`result: "fail"`, `log_slice` confirms clippy clone_on_copy)

## 4. Constraints

### What you DO

- **Branch from `45b59da2d`** — `git checkout 45b59da2d -b junior/fix-task-4a-v3-clone-on-copy`. Do not branch from governance-v0 (the unused-import fix is not on trunk yet).
- **One commit.** Subject: `fix(v1-JM-d): drop redundant .clone() on Copy type AdminTriggerAppealRejury (fix-4a-v3)`
- **Push your branch to origin.**
- **Raise a NEW `kind: "validate-pending"` DQ entry** to track the workflow run. Use `gh run list --repo barrie-cork/lemmy --branch <your-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId` to capture the workflow_run_id of the workspace check that fires on your push. JSON skeleton (next id is **88**):

```json
{
  "id": 88,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<ISO 8601 UTC now>",
  "workflow_run_id": <id from gh run list>,
  "branch": "junior/fix-task-4a-v3-clone-on-copy",
  "phase_task": 4,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

  Append to `pending[]` in `.claude/decision-queue.json` on your **worker branch** (same pattern as #48 — DQ raise commits live on the worker branch and get merged to trunk by the daemon on workflow pass). Commit subject: `chore(decision-queue): impl raised DQ #88 — validate-pending fix-4a-v3 workspace run <id>`. Push that DQ-update commit to your worker branch (NOT to governance-v0).

- DQ #84 + #85 (the previous failed entries) stay in `pending[]` on their respective branches. The advisor will close them as `superseded_by: 88` when fix-4a-v3's workflow passes. Do NOT mutate or move #84 or #85.

### What you DO NOT

- **Do NOT run `cargo` (any subcommand) on the EliteDesk worker.** Per agent doc commit `7419ce715` and the task #47 incident. The advisor has already validated locally on the laptop that this fix clears clippy.
- **Do NOT run `bash scripts/brehon/cargo-*.sh`** wrapper scripts on the worker. Same reason.
- **Do NOT push to `phase-v1-JM-d` or `governance-v0` directly.** Only push your worker branch.
- **Do NOT modify the fix-4a-v2 commits** (`5bf306003`, `45b59da2d`) — no `--amend`, no rebase. Add a NEW fix commit on top.
- **Do NOT touch line 48** (`admin_pseudonym.clone()`). String is not Copy; that `.clone()` is required.
- **Do NOT use `kind: "validate-pending-laptop"`** — the Shape-G workflow path was used by #48 and worked (workflow ran, ci-watcher mutated the entry). Stay consistent: `kind: "validate-pending"`.

## 5. Validation gate (GH Actions, not laptop, not worker)

This task's validation runs on **GitHub Actions** (workflow `cargo-validate-workspace.yml`), triggered by your branch push. Your worker branch is on the `junior/*` glob the workflow listens to. After your push, the workflow fires automatically; the workflow_run_id you capture in DQ #88 lets the advisor queue a ci-watcher to mutate the entry. You exit cleanly after pushing branch + DQ-update; do NOT wait for validation locally and do NOT run cargo.

Expected wall-clock to GH Actions completion: 8-15 min cold. Advisor will queue ci-watcher 10 to poll workflow + mutate DQ #88 with the result.

## 6. Expected output

You should produce, on `junior/fix-task-4a-v3-clone-on-copy` (off `45b59da2d`):

```
<sha1>  fix(v1-JM-d): drop redundant .clone() on Copy type AdminTriggerAppealRejury (fix-4a-v3)
        crates/api/api/src/governance/admin_trigger_appeal_rejury.rs | 2 +-

<sha2>  chore(decision-queue): impl raised DQ #88 — validate-pending fix-4a-v3 workspace run <id>
        .claude/decision-queue.json | 15 +++++++++++++++
```

Push both. Then exit. The advisor + daemon take it from there.

## 7. Why this brief differs from jm-d-fix-impl-4a-v2.md

v2 used `kind: "validate-pending-laptop"` per the agent doc's pre-Shape-G flow, but the worker actually wrote `kind: "validate-pending"` (Shape-G workflow flow) — and that worked because workflow YAMLs DO fire on `junior/*` branches under option-b. v3 stays consistent with what worked: Shape-G workflow validation. The new failure is a separate clippy lint (`clone_on_copy`) that the unused-import fix didn't address — clippy's `-D warnings` stops at first error, so the second one only surfaced after the first was cleared. Laptop pre-validation confirms this is the ONLY remaining clippy error.
