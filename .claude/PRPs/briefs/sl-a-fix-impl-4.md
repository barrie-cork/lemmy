---
role: impl-task
plan_task: cr-fix-4
phase: v1-SL-a
created: 2026-05-04
related_pr: 111
related_cr: [1,2,3,4,5,6,7,8,9,11,12,13]
preallocated_dq: [137]
---

# Brief — v1-SL-a fix-impl-4 — CodeRabbit triage fixes for PR #111

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a fix-impl-4 — see .claude/PRPs/briefs/sl-a-fix-impl-4.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a CodeRabbit fix-in-PR task for PR #111 after BM poll/triage. Address the 12 findings left in `bucket: fix-in-pr` in `.claude/PRPs/reviews/pr-111-findings.yaml` (local BM artifact): `cr-1` through `cr-9`, plus `cr-11`, `cr-12`, `cr-13`. Do **not** address `cr-10`; BM triaged it `wont-fix` as semantic-neutral SQL style churn on a validated one-time migration.

**Shape G:** do NOT run cargo locally on the EliteDesk worker. Make the code/docs edits, push your worker branch, let the workspace workflow run, then write exactly one `kind: "validate-pending"` DQ entry: **DQ #137** for the workspace-check workflow run. Do **not** pre-create an e2e `validate-pending` entry with `workflow_run_id: null`; the advisor raises Phase 2 e2e only after the fix branch is finalized into `phase-v1-SL-a`, per the PR #105 local-vs-dispatch user gate.

## 2. Scope

**Produce** one fix commit plus one DQ commit.

Fix these findings:

1. `cr-1` — `.claude/PRPs/briefs/sl-a-ci-watcher-13.md`: add blank lines around fenced code blocks (MD031), including the blocks around lines 20-30 and 47-50.
2. `cr-2` — `.claude/PRPs/briefs/sl-a-ci-watcher-14.md`: same MD031 fenced-block spacing cleanup.
3. `cr-3` — `.claude/PRPs/briefs/sl-a-fix-impl-2.md`: add language tags and blank lines around fenced code blocks in the cited ranges (around 51-55, 187-195, 231-243). Use `text`, `bash`, `json`, or `python` as appropriate.
4. `cr-4` — `.claude/PRPs/briefs/sl-a-fix-impl-2.md`: remove the stale instruction to pre-allocate DQ #134 as `kind: "validate-pending"` with `workflow_run_id: null`. Replace with the current Shape-G rule: workspace-check gets a real workflow run id; Phase 2 e2e is advisor-raised after phase-tip finalize (local-vs-dispatch user gate), or a placeholder must not use `kind: "validate-pending"`.
5. `cr-5` — `.claude/rules/governance-log-entry-kind-registry.md`: update both confirmed-exemption list occurrences to include the five v1-SL-a pre-landed consts: `ENTRY_KIND_SPONSOR_LIABILITY_PENDING`, `ENTRY_KIND_SPONSOR_LIABILITY_FIRED`, `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED`, `ENTRY_KIND_ENDORSEMENT_REVOKED`, `ENTRY_KIND_RESTORATION_COMPLETED`.
6. `cr-6` — `.claude/runlog/v1-SL-a-runlog.md`: add one blank line after every `##` heading flagged by MD022.
7. `cr-7` — `crates/api/api/src/governance/config.rs`: make `job.grace_check_interval_minutes` metadata internally consistent. Prefer changing the description to match `ApplyAt::Immediate` unless code context clearly shows restart-required semantics.
8. `cr-8` — `crates/server/tests/e2e.rs`: remove hard-coded full `governance_config` totals (`101`, `post_up_config_count - 67`) from `phase1_migrations_round_trip`. Assert the specific v1-SL-a governance_config keys are present after forward apply and absent after revert instead.
9. `cr-9` — `crates/server/tests/e2e.rs`: resolve contradictory post-revert enum expectations. The test currently asserts `pg_type.case_status` is dropped, then asserts labels for that type still exist. Make the enum-label post-revert check conditional on whether `case_status` exists, or remove the contradictory label-exists assertion if the type is absent.
10. `cr-11` — `scripts/brehon/migrate-roundtrip.sh`: quote `"${PG_PORT}:5432"` in the first Docker `-p` binding.
11. `cr-12` — `scripts/brehon/migrate-roundtrip.sh`: quote `"${PG_PORT2}:5432"` in the second Docker `-p` binding and keep related container variable usages safely quoted if the local context shows unquoted expansions.
12. `cr-13` — `.claude/PRPs/briefs/sl-a-ci-watcher-13.md`: rewrite the hard-refusal sentence so `kind: "blocker"` is explicitly allowed only for the orphan fallback case and forbidden otherwise. Remove the current contradiction ("never write a NEW DQ entry" vs "orphan case files a blocker" vs "never write kind: blocker").

**Do NOT**:

- Touch unrelated PR #111 findings not listed above.
- Rework the backfill UPDATE for `cr-10`.
- Add new migrations or change migration ordering.
- Change runtime sponsor-liability semantics beyond the metadata-description consistency fix in `config.rs`.
- Run local cargo on the EliteDesk worker.
- Pre-create any e2e `validate-pending` entry with `workflow_run_id: null`.

**Commit messages**:

- Fix commit: `fix(v1-SL-a): address PR 111 CodeRabbit triage findings (fix-impl-4)`
- DQ commit: `chore(decision-queue): impl raised DQ #137 — sl-a-fix-impl-4 workspace-check validate-pending`

## 3. Required reading

Read in this order before editing:

1. This brief in full.
2. PR #111 CodeRabbit findings context:
   - Local BM artifact if present: `.claude/PRPs/reviews/pr-111-findings.yaml`.
   - If absent in the worker worktree (it is gitignored), use `gh pr view 111 --repo barrie-cork/lemmy --json latestReviews,comments` and `gh api repos/barrie-cork/lemmy/pulls/111/comments` to read the 13 CodeRabbit findings.
3. `.claude/commands/bm/bm-triage.md` — bucket semantics; `cr-10` is intentionally not part of this task.
4. `.claude/rules/advisor-orchestrator.md` — Shape G validation and Phase 2 e2e local-vs-dispatch gate. This task must follow the current rule, not the stale instructions in older SL-a briefs.
5. `.claude/rules/governance-log-entry-kind-registry.md` — read both exemption-list occurrences before editing them.
6. Mirror/context files:
   - `.claude/PRPs/briefs/sl-a-ci-watcher-15.md` for current ci-watcher wording if helpful.
   - `.claude/PRPs/briefs/sl-a-fix-impl-3.md` for the current post-PR#105 local-e2e wording; do not copy its old e2e preallocation pattern.
   - `crates/api/api/src/governance/config.rs` surrounding the `job.grace_check_interval_minutes` metadata entry.
   - `crates/server/tests/e2e.rs` surrounding `phase1_migrations_round_trip` and the constants/helper patterns already in that file.
7. Lessons:
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` — capture full command output if you run non-cargo probes.
   - `.claude/lessons/feedback_clippy_test_style.md` — no clippy escape hatches.
   - `.claude/lessons/feedback_validate_pending_laptop_must_use_wrapper.md` and `.claude/lessons/feedback_laptop_default_for_validate_pending.md` — why Phase 2 e2e is advisor-side/local by default.
   - `.claude/lessons/feedback_lemmy_migration_runner.md` — relevant to preserving the e2e migration-roundtrip test intent while fixing brittle assertions.

## 4. Constraints

### Branch + commit discipline

- Start on a Junior worktree off `phase-v1-SL-a`. Do not push directly to `phase-v1-SL-a`.
- One fix commit for file edits, then one DQ commit for DQ #137. If you discover a required split, file a DQ instead of silently broadening.
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"` if needed.
- When writing `.claude/decision-queue.json`, use `json.dump(..., indent=2, ensure_ascii=True)` to avoid encoding-only churn.

### DQ #137 validation entry

After pushing your worker branch and locating the workspace-check workflow run id, append **exactly one** pending entry:

```json
{
  "id": 137,
  "from": "impl",
  "kind": "validate-pending",
  "question": "Shape G workspace-check validation for sl-a-fix-impl-4 CodeRabbit triage fixes.",
  "branch": "<your-junior-branch>",
  "phase_task": "cr-fix-4",
  "workflow_run_id": <real numeric run id>,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

If DQ #137 is already used, STOP and file a blocker DQ explaining the collision. Do not improvise a different id without advisor visibility.

### Current Shape-G e2e rule (important for cr-4)

Do **not** write a Phase 2 e2e `validate-pending` entry from this task. The advisor will detect the finalized `phase-v1-SL-a` tip and ask the user local vs dispatch before raising the e2e validation entry. This is the current rule after PR #105 and is the corrective pattern for `cr-4`.

### Validation probes before pushing

You may run lightweight non-cargo probes only:

```bash
bash -n scripts/brehon/migrate-roundtrip.sh
rg -n 'workflow_run_id: null|workflow_run_id": null|pre-allocate DQ #134|preallocate DQ #134' .claude/PRPs/briefs/sl-a-fix-impl-2.md .claude/PRPs/briefs/sl-a-ci-watcher-13.md
```

The `rg` probe should return no stale preallocation instruction in the edited passages. It may still find historical DQ JSON references elsewhere; inspect, do not blindly delete history.

Do not run cargo/check/clippy/e2e locally. Shape G handles workspace validation.

### Expected output

Return:

```text
## fix-impl-4 complete — PR #111 CodeRabbit triage fixes

Commit: <sha>
DQ: #137 workspace-check run <run-id>
Files changed:
  - <list>
Validation: bash -n passed; workspace-check validate-pending raised
Next: advisor queues ci-watcher for DQ #137; after pass/finalize, advisor handles Phase 2 e2e user gate.
```
