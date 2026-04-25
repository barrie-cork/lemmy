# JM-c impl handover — prp-* skill triggers folded into worktree

**Written:** 2026-04-25 by advisor session
**Target:** v1-JM-c impl session (currently active in `../brehon-fork-phase-v1-JM-c` with Task 1 WIP, OR a fresh CC instance resuming JM-c impl)
**Branch:** `phase-v1-JM-c`
**Worktree:** `C:/Users/barri/Developer/brehon-fork-phase-v1-JM-c`

---

## TL;DR

Advisor merged the new prp-* skill triggers from `governance-v0` into your worktree via fast-forward. Branch HEAD advanced `9e5dd60a5` → `4347284e0`. **Strict descendant fast-forward** (no merge commit). **Your Task 1 WIP was not disturbed** — the modified files do not overlap with the prp-* command files that were updated.

---

## What changed

| Before (your branch's previous HEAD `9e5dd60a5`) | After (your branch's new HEAD `4347284e0`) |
|---|---|
| Plan commit only | Plan commit + plan PR #97 merge commit + chore commit landing prp-* skill triggers |

The two new commits on your branch (added by the FF merge):

```
4347284e0 chore(prp): principle-style skill triggers for prp-implement + prp-plan
73c208f2b Merge pull request #97 from barrie-cork/phase-v1-JM-c
9e5dd60a5 docs(plan): v1-JM-c — submit_jury_vote 9-step handler implementation plan  ← your previous HEAD
```

**Files modified by `4347284e0`** (advisor-lane only — zero overlap with impl files):
- `.claude/commands/prp-core/prp-implement.md` (+13 lines: §3.2 + §3.3 sub-blocks)
- `.claude/commands/prp-core/prp-plan.md` (+10 lines: Step-by-Step Tasks preamble + Validation Commands preamble)

---

## What you should see in your working tree

Run `git status --short` from the JM-c worktree. Expect exactly 3 modified files (your Task 1 WIP, intact):

```
M .claude/rules/governance-log-entry-kind-registry.md
M crates/api/api/src/governance/governance_log.rs
M crates/db_schema/src/source/governance/governance_log.rs
```

Verify with:

```bash
git diff --stat
# Expected:
#  .claude/rules/governance-log-entry-kind-registry.md     | 17 ++++++++++++++++-
#  crates/api/api/src/governance/governance_log.rs         |  1 +
#  .../db_schema/src/source/governance/governance_log.rs   | 11 +++++++++++
#  3 files changed, 28 insertions(+), 1 deletion(-)
```

Verify the Task 1 work is the JM-c plan's Task 1 (declare `ENTRY_KIND_JURY_DEADLOCK` const + re-export through api shim + registry row):

```bash
grep -n "JURY_DEADLOCK" crates/db_schema/src/source/governance/governance_log.rs
# Expected: line ~180:  pub const ENTRY_KIND_JURY_DEADLOCK: &str = "jury_deadlock";

grep -n "JURY_DEADLOCK" crates/api/api/src/governance/governance_log.rs
# Expected: line ~59 (in the use list of the api shim re-export)

grep -n "v1-JM-c entry kinds" .claude/rules/governance-log-entry-kind-registry.md
# Expected: a section "## v1-JM-c entry kinds (1, this sub-phase)" with the JURY_DEADLOCK row
```

If all three checks pass, your Task 1 WIP is intact and the FF merge was non-disruptive. Proceed to commit Task 1 normally per JM-c plan §13.

---

## Three skills now available in your worktree

The advisor's chore commit `4347284e0` added principle-style triggers for three project skills to `.claude/commands/prp-core/prp-implement.md` (which auto-loads when you invoke `/prp-core:prp-implement`). They are **principle triggers, not mandates** — each skill's SKILL.md owns the skip-conditions; the prp-implement.md trigger blocks just name the trade-off so you can decide. **When in doubt, inline is the legitimate default.**

### `/cargo-validate` — when the cargo run is the gating signal

Wraps any cargo run via the Brehon Windows wrapper (`scripts\\brehon\\cargo-check.bat` etc.); captures full output to `.claude/PRPs/debug/v1-JM-c-<task>-<verb>.log`; tails 20 lines into chat; returns the exit code as the report's headline.

**Use it for:** every per-task DoD check after each Edit, plan §15 validation gates, the negative-probe check at session start.

**Skip it when:** the cargo run is incidental (one-off scratch invocation), an outer harness has already captured the output, or the cargo run is itself a long-running background job (use the cargo-runner background subagent instead).

The plan §13 task validation blocks already specify the canonical capture-then-tail pattern: `cmd //c "scripts\\brehon\\cargo-<verb>.bat <args> > .claude/PRPs/debug/v1-JM-c-task<N>-<verb>.log 2>&1"; echo "exit: $?"; tail -20 ...`. The `/cargo-validate` skill encodes this exact pattern — invoking `/cargo-validate` is equivalent to running the inline shape, but cleaner.

### `/test-write` — for new e2e cases that need fixture scaffolding

Authors integration tests under `crates/server/tests/e2e.rs` following the canonical `LemmyResult<()>` + pseudonymisation + no-`unwrap`/`expect` discipline. Reuses the `v1_jm_b_fixtures` module (`bootstrap`, `seed_user`, `seed_community`, `seed_jurors`, `seed_case`, `seed_jury_eligible_snapshots`, `seed_founder_event`).

**Use it for:** Plan §13 Task 6 — the 6 new e2e tests (snapshot-aware threshold, deadlock, appeal-window default, appeal-window live config, v0-compat regression, concurrent-votes exactly-once).

**Skip it when:** the test is a `#[cfg(test)] mod tests` unit case that doesn't touch the harness, or you're extending an existing test with one extra assertion.

**Plan §10.7 R2 reminder** — every JM-c test that exercises submit_jury_vote MUST call `v1_jm_b_fixtures::seed_jury_eligible_snapshots(conn, &juror_ids)` BEFORE `admin_assign_jury` to bypass the small-pool fallback path. The skill enforces this discipline; if you go inline, remember to do it manually.

### `/edit-mechanical` — for repeat-pattern edits across N call sites

Disciplined Rust edits with **rg-enumerate-first** ordering. Designed to prevent the **R5.1 class of bug** (PR #92 cr-12, JM-b Event 2): "I added a field with `Default::default()` propagation; tests broke because three callers used explicit struct literals."

**Use it for:** repeat-pattern edits like:
- The Task 1 const + re-export propagation (the work you already did manually — for future Task 1-shaped tasks, this skill is the right shape)
- Any future task that adds a struct field with `derive(Default)` (R3 GOTCHA in plan §13 Tasks 1, 2, 4)
- Replacing `as` casts with `i64::from(...)` across multiple sites if you find a missed spot during validation

**Skip it when:** the edit needs type/borrow reasoning beyond the rename surface (most JM-c handler-logic edits — e.g., the per-decision threshold tally in Task 3 needs reasoning about the hardcoded enum-variant iteration vs `strum` dependency, which is judgment-call territory; do that inline), the change is one-of-a-kind (most Task 3, Task 4, Task 5 work), or the edit is in `migrations/**` (out of scope; not relevant for JM-c which has no migrations).

**Note for Task 1:** since you've already completed the const + re-export edit inline, this is informational — no need to re-do via the skill. For Task 2 onward, evaluate per task.

---

## Commit your Task 1 WIP normally — no special handling

Your next action is the Task 1 commit per JM-c plan §13. Use the commit subject the plan specifies verbatim:

```
feat(v1-JM-c): add ENTRY_KIND_JURY_DEADLOCK const + registry row (task 1)
```

Validation per plan §13 Task 1 (you can use `/cargo-validate` or invoke the wrapper inline):

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-c-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task1-check.log
# Expect: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-c-task1-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task1-clippy.log
# Expect: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-JM-c-task1-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task1-test-no-run.log
# Expect: exit 0 (test target compiles cleanly with the new const + re-export)
```

After validation passes, commit:

```bash
git add crates/db_schema/src/source/governance/governance_log.rs \
        crates/api/api/src/governance/governance_log.rs \
        .claude/rules/governance-log-entry-kind-registry.md
git commit -m "$(cat <<'EOF'
feat(v1-JM-c): add ENTRY_KIND_JURY_DEADLOCK const + registry row (task 1)

Declare the canonical entry-kind const for the JM-c deadlock path; re-export through
the api shim; document the call-site contract in the governance-log registry. The const
is consumed by submit_jury_vote.rs::process_vote at the deadlock branch (added in
task 3) when all jurors have voted but no JuryDecision met threshold_count_snapshot.

Per .claude/rules/governance-log-entry-kind-registry.md, every new entry kind needs
both a canonical const + a registry row + a documented call site. The call site lands
in task 3.

Refs: PRD §9.1 step 5 (deadlock semantics); JM-b retro §3.3 (no other ENTRY_KIND
consts to add in JM-c beyond JURY_DEADLOCK).
EOF
)"
```

---

## After Task 1 commits, proceed to Task 2 per the plan

Plan §13 Task 2 — replace `QUORUM` + `APPEAL_WINDOW_DAYS` consts with snapshot reads. Per plan §10.2 GOTCHA (R1 — `i64::from(i32)` not `as` cast). Per plan §10.2 GOTCHA (load case row earlier? — plan resolution: keep FOR UPDATE at current position, add a separate non-locking single-column read for the gate; see plan §13 Task 2 REVISED IMPLEMENT block).

This task is on the boundary of `/edit-mechanical` applicability — it deletes 2 consts and adds a snapshot read at one site. Could go either way; lean inline since the snapshot read needs `ok_or` error-typing reasoning that's not pure mechanical. If you find more `QUORUM`/`APPEAL_WINDOW_DAYS` references elsewhere in the workspace (the plan says no, but verify), then `/edit-mechanical` becomes the right shape.

---

## What changed for the impl session structurally

| Concern | Before | After |
|---|---|---|
| Plan reference | `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` | unchanged |
| `/prp-core:prp-implement` command body | did not mention the three skills | now mentions `/cargo-validate`, `/test-write`, `/edit-mechanical` as principle triggers (advisory, not mandates) |
| Branch HEAD | `9e5dd60a5` | `4347284e0` (FF; no rewrites) |
| Working tree WIP | Task 1 (3 files modified) | unchanged — same 3 files modified, intact |
| Plan task wording | unchanged | unchanged (the new prp-plan triggers will affect *future plan-author runs*, not this already-emitted plan) |

---

## DQ #47 reminder

Still pending — planner-attributed (OQ-V1-JM-07: post-JM-b general case-open severity-tier inference). v1.5 territory; **does NOT block JM-c**. Per JM-b retro §3.3: JM-c MUST NOT add case-open severity_tier writers. Plan §12 + §19 make this explicit.

---

## What this handover does NOT cover

- The on-going JM-c impl is mid-Task-1 — full impl status, the BM session's role for JM-c PR cycle (post-impl), and the JM-c retro author's notes for §6.1 tool-use self-assessment are out of scope here. Refer to JM-c plan §13 (task spec) and the BM runlog at `.claude/runlog/bm-runlog.md` (post-PR-#97 entries) for those.
- The advisor session that wrote this brief is standing by for DQ answers; not actively monitoring impl progress.

---

_Handover author: advisor session 2026-04-25 14:00Z (post-FF-merge of `governance-v0` into `phase-v1-JM-c`). Brief deliberately short — the JM-c plan is canonical for everything else._
