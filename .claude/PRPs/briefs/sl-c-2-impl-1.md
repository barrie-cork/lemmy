---
role: impl-task
plan_task: 1
phase: v1-SL-c-2
created: 2026-05-08
related_dq: null
---

# Brief — v1-SL-c-2 Task 1 — e2e test #1 (fire path) + open `mod v1_sl_c_fixtures` + helpers

## 1. Role + dispatch line

`[role:impl-task] sl-c-2-impl-1 — see .claude/PRPs/briefs/sl-c-2-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 1
from `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` §13 (lines
1316–1508). Single Edit at end of `crates/server/tests/e2e.rs` to open
the new `mod v1_sl_c_fixtures` block, add the 2 helper fns, and the
first test fn `grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries`.

## 2. Scope

**Produce one commit on your worker branch:**

- Single Edit on `crates/server/tests/e2e.rs` — anchor-insert AFTER the
  closing `}` of `mod v1_sl_b_fixtures` (Probe 11 from Task 0 confirmed
  this anchor at line 10980). Append the full mod block carried verbatim
  from plan §13 Task 1 lines 1346–1465 (use block + 2 helpers + 1 test
  fn). The test fn body is already a stub-with-spec — the plan provides
  setup/drive/assert comments + `Ok(())` placeholder. Task 1 ships ONLY
  the mod shell + helpers + the empty test stub. Tasks 2–5 fill in test
  bodies in subsequent commits.

- After commit + push, write `kind: "validate-pending"` DQ entry per
  Shape G + commit + push immediately (per `.claude/rules/decision-queue.md`
  "Mid-task visibility").

**Do NOT** in this task:

- Touch any file other than `crates/server/tests/e2e.rs`.
- Bulk-edit multiple tests in one Edit. Per `feedback_junior_worker_e2e_edit_hang.md`
  + plan §13 Task 1 GOTCHA: ONE anchor-Edit at file end, inserting the
  mod shell + helpers + Task 1's test stub only.
- Run cargo locally (Shape G — workspace check runs on GH Actions post-push).
- Modify `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.

## 3. Required reading

In order, before any Edit:

1. **Plan §13 Task 1** (lines 1316–1508 of
   `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md`). Canonical
   ACTION + FILES yaml + IMPLEMENT block + MIRROR refs + every GOTCHA.
2. **Plan §10.6, §10.8** — fire-branch governance_log payload shape +
   fixture mod shape. Cited as MIRROR in §13 Task 1.
3. **Lessons:**
   - `feedback_junior_worker_e2e_edit_hang.md` — anchor-Edit discipline;
     never bulk-edit `crates/server/tests/e2e.rs`. Single Edit at file end
     only.
   - `feedback_features_full_p_crate_incompatible.md` — never use
     `-p <crate> --features full` patterns. (Bound only on validate-pending
     DQ commands; Task 1 itself doesn't run cargo.)
   - `feedback_pipes_mask_exit_codes.md` — capture log to file separately
     from exit-code check. (Bound on validate-pending; not needed during
     Task 1's Edit phase.)
4. **Adjacent fixture mods (read structure only):**
   - `crates/server/tests/e2e.rs` `mod v1_sl_b_fixtures` (closes ~10979) —
     immediate sibling above your insert point.
   - `crates/server/tests/e2e.rs` `mod v1_jm_e_fixtures` (~9786) —
     fallback anchor (NOT applicable; Probe 10 confirmed SL-b merged).

## 3a. Handover from prior cohort

(none — first impl task after Task 0 pre-flight. Task 0 verified all 20
probes: PASS. Branch tip on origin: `47af884c8` (= bm-cut brief commit).
Worker branch starts here. c-1 module confirmed present at
`crates/api/api/src/governance/sponsor_liability_grace.rs` with all 4 pub
async fns; Task 1's import block in plan §13 lines 1353–1370 will resolve.)

## 4. Constraints

### Branch + environment

- Junior worktree branched off `phase-v1-SL-c-2` tip `47af884c8`.
- Single commit on worker branch with subject from plan §13 Task 1
  COMMIT MESSAGE line.
- Push the worker branch. Junior daemon's finalize-merge runs after
  workspace-check resolves green.

### Edit discipline (R4 + e2e edit-hang lesson)

- ONE Edit on `crates/server/tests/e2e.rs`. Anchor at the closing `}` of
  `mod v1_sl_b_fixtures` (line 10980 per Task 0 Probe 11; reconfirm at
  task start with `grep -n '^mod v1_' crates/server/tests/e2e.rs | tail -1`).
- New mod opens AFTER that closing `}`.
- Content is the verbatim block from plan §13 Task 1 (lines 1348–1465).
  Test fn body stays as the stub spec from the plan — do NOT fill in
  setup/drive/assert in this task; that's Tasks 2-5's territory or
  follow-up work in this same task only if Task 1's brief explicitly
  asks. **It does not — Task 1 ships the empty stub.**

### Validate-pending DQ (Shape G)

After push, write a `kind: "validate-pending"` DQ entry per
`decision-queue.md` schema:

```json
{
  "id": <max(all ids in pending+resolved)+1>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<ISO 8601 UTC>",
  "question": "Does cargo-validate-workspace pass on sl-c-2 task 1 (fire-path test stub + mod v1_sl_c_fixtures shell)?",
  "options": [
    "(A) pass — advance to Task 2 (escape path test)",
    "(B) fail — advisor §G4 triage"
  ],
  "context": "Pushed test(v1-SL-c-2): e2e test #1 — fire path expired pending case + mod v1_sl_c_fixtures shell (task 1) to <branch>. cargo-validate-workspace.yml run triggered on push.",
  "branch": "<your worker branch>",
  "phase_task": "sl-c-2-task-1",
  "workflow_run_id": <captured via `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --workflow cargo-validate-workspace --json databaseId`>,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null
}
```

Commit + push the DQ write IMMEDIATELY after writing it (mid-task
visibility — advisor's polling loop reads `decision-queue.json`).

### DQ attribution

- `from: "impl"`. Self-resolve only as `"impl-self-resolved"` if needed
  (rare; should not be needed for Task 1).
- NEVER write `answered_by: "advisor"` or `"user"`.
- NEVER write `kind: "clarify"` (advisor-only).

### Hard refusals

- Do NOT modify `.claude/PRPs/plans/**`, `.claude/PRPs/briefs/**`,
  `.github/workflows/**`, `Cargo.toml`, `Cargo.lock`,
  `crates/api/api/src/governance/sponsor_liability_grace.rs` (c-1
  module — read only).
- Do NOT bulk-edit `crates/server/tests/e2e.rs` (per
  `feedback_junior_worker_e2e_edit_hang.md` — file is 11,925 lines;
  bulk-edit hangs the Junior worker).
- Do NOT run cargo locally (Shape G — workflow YAML runs cargo on GH
  hosted runners post-push).
- Do NOT touch the runlog, BM-owned files, or other branches.

## 5. Acceptance

Task 1 passes if:

- Single commit on worker branch with subject
  `test(v1-SL-c-2): e2e test #1 — fire path expired pending case + mod v1_sl_c_fixtures shell (task 1)`.
- Diff: ONLY `crates/server/tests/e2e.rs`, additive only (mod block
  appended at file end after `mod v1_sl_b_fixtures` closes), ~120 lines
  added, 0 removed.
- Worker branch pushed.
- One `kind: "validate-pending"` DQ entry written + committed + pushed,
  citing the workflow_run_id of the workspace-check run triggered by the
  push.
- No additional DQ pending entries from this task (or, if any, they are
  blocker-class with full evidence).
