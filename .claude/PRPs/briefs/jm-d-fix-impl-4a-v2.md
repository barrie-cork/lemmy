---
role: impl-task
plan_task: 4
phase: v1-JM-d
created: 2026-04-28
status: ready
related_dq: 84
supersedes_premise: jm-d-fix-impl-4a.md (worker ran cargo locally for >1 hour despite brief; agent doc updated 7419ce715 to bind anti-cargo rule, this brief uses validate-pending-laptop flow)
---

# Brief — v1-JM-d Fix 4a-v2 — remove unused `Appeal` import (laptop validation)

## 1. Role + dispatch line

`[role:impl-task] v1-JM-d fix-4a-v2 — see .claude/PRPs/briefs/jm-d-fix-impl-4a-v2.md`

## 2. Scope

**Context — what shipped, what didn't:** Task #45 (`0d2975388`, feat: admin_trigger_appeal_rejury + seat_appeal_panel) shipped the Task 4 work but Phase 1 workspace check (run `25073660917`, ci-watcher Task #46) failed clippy with one allowlist-class error. Task #47 (the first fix-4a attempt) was **cancelled** after the worker ran `cargo check --workspace --features full` on the EliteDesk for 1h 35min, sustained 4.0 GB swap exhaustion (incident: `project_elitedesk_hung_2026_04_27`). Per agent-doc commit `7419ce715` ("docs(rules): cargo+e2e run on laptop, never on EliteDesk worker"), pre-Shape-G plans now delegate cargo to the laptop advisor session via `kind: "validate-pending-laptop"` DQ entries. This brief uses that flow.

**The clippy error to fix:**

```
error: unused import: `Appeal`
   --> crates/api/api/src/governance/admin_assign_jury.rs:62:5
    |
62  |     appeal::Appeal,
    |     ^^^^^^^^^^^^^^
    = note: `#[deny(unused_imports)]` on by default
```

The `appeal::Appeal` struct (line 62) was added to the import block but never referenced in the file. Verified: zero `Appeal::` (struct method) usages in `admin_assign_jury.rs`. The `schema::appeal` table import (around line 80) is **separate** and IS used (lines 1225-1228 in `seat_appeal_panel`); leave it untouched.

**This is a §G4 allowlist auto-queue case** — single file, single line, clippy auto-fix class. No design decisions, no scope change.

**Branching strategy (read carefully):**

The Task 4 feat commit `0d2975388` is NOT on `governance-v0` trunk (last successful merge into trunk was `cb4d0576d`, Task 3 stabilization). Trunk is at `1f6921cd1` (post your agent-doc updates). To re-validate the feat + fix together, this fix-impl-task must branch off the feat commit, NOT off governance-v0:

```bash
git fetch origin junior/role-impl-task-v1-jm-d-task-4-see-claude-prps-briefs-jm-d-impl-4-md-45
git checkout 0d2975388 -b junior/fix-task-4a-v2-unused-import
```

The daemon's default base branch (governance-v0) is wrong for this fix — you MUST manually checkout `0d2975388` first.

**Produce** (one fix commit on top of `0d2975388`):

### Fix — `crates/api/api/src/governance/admin_assign_jury.rs`

Delete line 62 (`    appeal::Appeal,`) only. The surrounding `use lemmy_db_schema::{ ... source::governance::{ ... } }` block must remain — leave the other 3 entries (`jury_assignment::JuryAssignmentInsertForm`, `jury_constraint_violation_log::JuryConstraintViolationLogInsertForm`, `moderation_case::ModerationCase`) intact.

After the fix, the block should read:

```rust
use lemmy_db_schema::{
  newtypes::{AppealId, ModerationCaseId},
  source::governance::{
    jury_assignment::JuryAssignmentInsertForm,
    jury_constraint_violation_log::JuryConstraintViolationLogInsertForm,
    moderation_case::ModerationCase,
  },
};
```

That is the **only** edit. Do not touch any other line, file, or import. Do not remove the `schema::appeal` table import — it is used.

## 3. Required reading

- `.claude/agents/impl-task.md` — full file. Pay special attention to:
  - "Per-task validation gate (out-of-band)" → "Pre-Shape-G plans" sub-section (this is the flow you use)
  - Hard refusal: "Do NOT run cargo locally on the EliteDesk worker under any circumstances" — incident task #47 cited inline
- `crates/api/api/src/governance/admin_assign_jury.rs:55-90` (after checkout of `0d2975388`) — the import block as it currently exists post-feat
- DQ #84 in `.claude/decision-queue.json` (governance-v0) — the failed validation context (still pending; will be superseded once fix-4a-v2 passes)

## 4. Constraints

### What you DO

- **Branch from `0d2975388`** — `git checkout 0d2975388 -b junior/fix-task-4a-v2-unused-import`. Do not branch from governance-v0.
- **One commit.** Subject: `fix(v1-JM-d): remove unused appeal::Appeal import (fix-4a-v2)`
- **Push your branch to origin.**
- **Raise a `kind: "validate-pending-laptop"` DQ entry on `governance-v0`.** The advisor (laptop session) will read this entry on its next polling tick, run the §15 DoD commands locally on the laptop's `brehon-fork` checkout, mutate the entry with `result`, and on `pass` the daemon's standard finalize-merge fires. JSON skeleton (next id is **85**):

```json
{
  "id": 85,
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO 8601 UTC now>",
  "branch": "junior/fix-task-4a-v2-unused-import",
  "phase_task": 4,
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full > /tmp/jm-d-fix-4a-v2-check.log 2>&1",
    "bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > /tmp/jm-d-fix-4a-v2-clippy.log 2>&1",
    "bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server > /tmp/jm-d-fix-4a-v2-test-no-run.log 2>&1"
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

These commands are quoted **verbatim** from plan §15 Task 4 VALIDATE block. Append this entry to `pending[]` in `.claude/decision-queue.json` on `governance-v0`. Commit subject: `chore(decision-queue): impl raised DQ #85 — validate-pending-laptop fix-4a-v2`. Push that DQ-update commit to `governance-v0`.

- DQ #84 (the failed Phase-1 entry) stays in `pending[]` for now. The advisor will close it as `superseded_by: 85` when fix-4a-v2's laptop run passes. Do NOT mutate or move DQ #84.

### What you DO NOT

- **Do NOT run `cargo` (any subcommand: check, clippy, test, build, doc, fmt) on the EliteDesk worker.** The agent doc and §G4 of this brief make this a hard refusal. If your task-0 logic suggests running cargo, treat it as a brief violation — exit non-zero with a one-line `cargo-on-elitedesk-blocked` note and DO NOT proceed.
- **Do NOT run e2e tests** (`cargo test --test e2e`, `cargo test --features full`) on the worker. Same reason.
- **Do NOT run `bash scripts/brehon/cargo-*.sh`** wrapper scripts on the worker. The wrapper invokes cargo; same prohibition.
- **Do NOT push to `phase-v1-JM-d` or `governance-v0` directly.** Only push your worker branch + the DQ-update to governance-v0 (the DQ commit is the only governance-v0 push allowed).
- **Do NOT modify the feat commit `0d2975388`** (no `--amend`, no rebase) — add a NEW fix commit on top.
- **Do NOT use `kind: "validate-pending"`** (that's the Shape-G workflow path, not the laptop path). The brief above mandates `validate-pending-laptop`.

## 5. Validation gate (laptop, NOT worker)

This task's validation runs on the **laptop advisor session** (`C:\Users\barri\Developer\brehon-fork`), not on the EliteDesk worker. The §15 DoD commands listed in your `validate-pending-laptop` `commands[]` array will be run by the advisor sequentially after the entry is committed. You exit cleanly after pushing your branch + the DQ-update; do NOT wait for validation locally.

Expected wall-clock to laptop-side validation completion: 3-12 min (cargo-check.sh ~5-8 min cold, ~2-3 min warm; clippy adds ~2-4 min; test --no-run adds ~1-2 min). The advisor will mutate DQ #85 with the result and proceed per the §G4 classifier on fail or finalize-merge on pass.

## 6. Expected output (commits + push)

You should produce, on `junior/fix-task-4a-v2-unused-import` (off `0d2975388`):

```
<sha1>  fix(v1-JM-d): remove unused appeal::Appeal import (fix-4a-v2)
        crates/api/api/src/governance/admin_assign_jury.rs | 1 -
```

And on `governance-v0` (the DQ raise):

```
<sha2>  chore(decision-queue): impl raised DQ #85 — validate-pending-laptop fix-4a-v2
        .claude/decision-queue.json | 17 +++++++++++++++++
```

Push both. Then exit. The advisor + daemon take it from there.

## 7. Why this brief differs from jm-d-fix-impl-4a.md (v1)

Worker in v1 task #47 ran `cargo check --workspace --features full` for 1h 35min on the EliteDesk despite brief constraints saying "Do NOT run cargo locally". Root cause: brief-vs-agent-rule conflict resolved in favor of the older agent rule (which had inline cargo invocation expected). User updated agent doc at `7419ce715` to bind the anti-cargo rule into the agent contract itself + introduce `validate-pending-laptop` flow. This brief (v2) uses the new flow + cites the binding agent rule directly so the worker reads consistent guidance from brief and agent doc both.
