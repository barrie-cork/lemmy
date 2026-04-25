# JM-c impl handover — Task 3 onward (after Task 2 commit)

**Author:** impl session (Claude Opus 4.7 1M, fork-local)
**Written:** 2026-04-25 mid-session, after Task 2 committed cleanly
**Target:** fresh impl session resuming v1-JM-c at Task 3
**Branch:** `phase-v1-JM-c`
**Worktree:** `C:/Users/barri/Developer/brehon-fork-phase-v1-JM-c`
**HEAD at write time:** `c6c43d819` (Task 2 commit)
**Working tree:** clean (no staged or unstaged changes)

---

## TL;DR

Tasks 1 + 2 of the JM-c plan are committed and validated. Pre-phase
harness audit ran clean (all 4 probes + Docker preflight). Resume at
**Task 3 — Step 5 per-decision threshold + deadlock branch** per
`.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` §13.

One **deviation from plan §13 Task 2** is documented in this brief +
in commit `c6c43d819`'s body: `APPEAL_WINDOW_DAYS` const survives Task
2 because it is still referenced at line ~327 (the v0 `closed_at`
write that Task 5 will remove). Pairing const-removal with its single
call site at Task 5 keeps the change atomic. No advisor input required
— this is a self-resolved interpretation per `decision-queue.md` rules
(no DQ written; not needed).

---

## What just shipped

Two commits, in plan order:

```
c6c43d819 feat(v1-JM-c): replace QUORUM const with snapshot read; defer APPEAL_WINDOW_DAYS to task 5 (task 2)
5e2f58181 feat(v1-JM-c): add ENTRY_KIND_JURY_DEADLOCK const + registry row (task 1)
```

Files changed (cumulative across both commits):
- `crates/db_schema/src/source/governance/governance_log.rs` — added
  `pub const ENTRY_KIND_JURY_DEADLOCK: &str = "jury_deadlock";` (with
  full doc-comment per plan §10.1)
- `crates/api/api/src/governance/governance_log.rs` — added
  `ENTRY_KIND_JURY_DEADLOCK` to the alphabetical re-export list
- `.claude/rules/governance-log-entry-kind-registry.md` — added new
  `## v1-JM-c entry kinds (1, this sub-phase)` section + bumped count
  invariant from 32 → 33
- `crates/api/api/src/governance/submit_jury_vote.rs` — deleted
  `const QUORUM`, replaced lines 201-213 vote-count gate with a
  non-locking single-column SELECT of `case.quorum_snapshot` then
  gate via `if vote_count < i64::from(quorum_snapshot)`. Doc-comment
  on the surviving `APPEAL_WINDOW_DAYS` const explains the deferral.

Validation per task (all logged at `.claude/PRPs/debug/v1-JM-c-task<N>-*.log`):

| Task | cargo check | clippy --no-deps -- -D warnings | test --no-run |
|---|---|---|---|
| 1 | exit 0 | exit 0 | exit 0 |
| 2 | exit 0 | exit 0 | exit 0 |

Registry invariant (after Task 1):
- 33 `ENTRY_KIND_*` consts in `db_schema` definition file
- 33 re-exports in `api` shim
- zero duplicate string literal values

---

## Pre-phase audit (already done — do NOT re-run)

Per `.claude/rules/pre-phase-harness-audit.md`, all 4+1 probes ran
before Task 1 and passed. Audit flag `.claude/audit-phase-v1-JM-c-complete.flag`
exists. Logs at:
- `.claude/audit-cargo-check-p.log` (probe 1, exit 0, only `lemmy_utils` checked)
- `.claude/audit-cargo-check-features.log` (probe 2, exit 0, `--features full` honored)
- `.claude/audit-cargo-test.log` (probe 3, exit 0, `--no-run` honored)
- `.claude/audit-cargo-test-negative.log` (probe 4, exit **101** non-zero, exit code propagated)

Docker daemon was running at audit time. If your fresh session sees
Docker stopped (Docker Desktop has a habit of stopping on sleep/resume
on Windows), restart it before any `cargo test --test e2e` run.

---

## Resume point — Task 3

### Plan reference

Read `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` §13 Task 3
(line ~1188), plus the patterns at §10.3 (per-decision tally) and
§10.4 (deadlock branch + governance_log emission).

### What to implement

In `crates/api/api/src/governance/submit_jury_vote.rs`, replace the
lines 215-231 v0 tally + `pick_majority` invocation with:

1. After the existing FOR UPDATE case load + idempotency guard (lines
   ~240-278 in the original, currently lines 252-290 after Task 2's
   edits — verify line numbers before editing), read
   `threshold_count_snapshot` and `panel_size_snapshot` from
   `case_row` via `ok_or` per plan §10.2 GOTCHA. Use the same
   `LemmyErrorType::Unknown(format!(...))` shape as Task 2 (the plan
   names `NoEnoughJurorsAvailable` but that variant does NOT exist in
   `crates/utils/src/error.rs` — see "Plan vs reality" below).
2. Per-decision tally per plan §10.3: load `Vec<JuryDecision>` for
   the case, count via `HashMap<JuryDecision, i64>`, iterate the
   8 hardcoded `JuryDecision` variants in stable enum-order, pick the
   first decision meeting `>= i64::from(threshold_count_snapshot)`.
3. Deadlock branch per plan §10.4: if `winning_decision.is_none()` and
   `vote_count == i64::from(panel_size_snapshot)`, UPDATE
   `case.status = AdminReview`, emit `ENTRY_KIND_JURY_DEADLOCK`
   governance_log entry (Task 1 already shipped this const — import
   from `crate::governance::governance_log::ENTRY_KIND_JURY_DEADLOCK`),
   early-return `vote_recorded: true, case_decided: false`.
4. If `winning_decision.is_none()` and `vote_count < panel_size_snapshot`,
   early-return without UPDATE or log emission (partial tally).
5. Delete the `pick_majority` helper at lines 510-518 (now unused).

### Plan vs reality — error variant naming

Plan §10.2 + §13 Task 2/3 both name `LemmyErrorType::NoEnoughJurorsAvailable`.
That variant does **not** exist in `crates/utils/src/error.rs`. Task
2's commit used `LemmyErrorType::Unknown(format!("case {} has NULL
quorum_snapshot; admin_assign_jury did not run", data.case_id.0))`
which mirrors `admin_assign_jury.rs:189` and `:304` ("unexpected
internal-state" convention). Use the same shape for the
`threshold_count_snapshot` and `panel_size_snapshot` reads in Task 3.
Don't try to add a new error variant — it would touch
`crates/utils/src/error.rs` which is out of JM-c file-ownership scope.

### Plan §10.4 GOTCHA — `juror_pseudonym` reuse

The deadlock log entry's `actor_pseudonym` argument is the casting
juror's pseudonym. Reuse the existing `juror_pseudonym` variable
(set at line ~101-102 of the handler). Do NOT call
`actor_pseudonym_helper::get_or_create` twice in the same handler
invocation.

### Plan §10.4 GOTCHA — deadlock UPDATE shape

Write `status = AdminReview` ONLY. No `decided_at`, no `closed_at`,
no `appeal_window_expires_at`. A deadlocked case is not "decided" —
it's "stuck pending admin." Lifecycle terminates here.

### Plan §10.3 GOTCHA — exhaustiveness anchor

The per-decision iteration hardcodes 8 `JuryDecision` variants. The
compile-time exhaustiveness anchor is `map_decision_to_sanction` at
lines ~523-549 (which already requires exhaustive match). Add the
inline comment from §10.3 noting this dependency.

---

## Validation pattern — same as Tasks 1 + 2

After every Edit:

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-c-task3-check.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/v1-JM-c-task3-check.log

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-c-task3-clippy.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/v1-JM-c-task3-clippy.log

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-JM-c-task3-test-no-run.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/v1-JM-c-task3-test-no-run.log
```

All three must exit 0 before the Task 3 commit. Per
`cargo-output-capture.md`: never pipe cargo through tail/head/grep
without capturing first; per `no-cargo-output-paste.md`: never paste
the full log into chat — tail-20 only.

Estimated runtimes (warm cache, this worktree, just observed):
- cargo check workspace --features full: ~2 min
- cargo clippy --workspace --features full --no-deps: ~2-6 min
- cargo test --test e2e --no-run -p lemmy_server: ~2-4 min

---

## Tasks remaining (5 of 8)

| # | Subject | Status |
|---|---|---|
| 3 | Step 5 per-decision threshold + deadlock branch | RESUME HERE |
| 4 | Step 7 sponsor-liability TODO insertion-point comment | pending |
| 5 | Step 9 appeal_window write + closed_at removal (also deletes APPEAL_WINDOW_DAYS const carried over from Task 2) | pending |
| 6 | 6 e2e tests + `lookup_local_user_view` helper | pending |
| 7 | Full-workspace validation pass (no commit) | pending |
| 8 | Retrospective + report | pending |

---

## Coordination state

### Decision queue

DQ #47 (OQ-V1-JM-07) still pending — planner-attributed,
"post-JM-b general case-open severity-tier inference question." Per
plan §19 + §12: **does NOT block JM-c**. JM-c MUST NOT add case-open
severity_tier writers — non-emergency paths inherit JM-a DEFAULT
'Minor'. v1.5 territory. No action required from impl side this
session.

No new DQ entries written by this impl session. Nothing to surface
at session start beyond confirming DQ #47 still pending.

### Runlog

No runlog edits this session. Session worked on `phase-v1-JM-c` only;
the BM-runlog at `.claude/runlog/bm-runlog.md` lives on
`governance-v0` (cross-branch writes break phase discipline per
`branch-manager.md`).

### Skill triggers (informational)

The advisor's chore commit `4347284e0` (already in this branch's
history) added principle-triggers for three skills to
`/prp-core:prp-implement`:

- **`/cargo-validate`** — encodes the capture-then-tail-20 pattern.
  Inline equivalent shown above; either form is fine. Lean inline if
  the validation is one-off.
- **`/test-write`** — relevant for Task 6 (the 6 new e2e tests).
  Skill enforces R2 fixture seeding (`seed_jury_eligible_snapshots`
  before `admin_assign_jury`). Inline is also fine; just remember R2.
- **`/edit-mechanical`** — for repeat-pattern edits across N call
  sites. Tasks 3-5 are mostly judgment-call edits (per-decision
  tally, deadlock branch wording, TODO comment, two-UPDATE shape) —
  inline is the right shape there. Could be useful for Task 5's
  `closed_at` removal if there's any caller-side adjustment needed.

When in doubt, inline is the legitimate default. See
`impl-2026-04-25-jm-c-prp-skill-triggers-fold.md` (in primary
worktree at `C:/Users/barri/Developer/brehon-fork/.claude/PRPs/handovers/`)
for the full advisor brief on the skill triggers.

---

## Closing-state assertions

Verify the following at session-resume to confirm this brief still
matches reality:

```bash
git rev-parse HEAD
# Expect: c6c43d819f1fe8303264f57d9eecf6ce5e25aaca

git status --porcelain
# Expect: empty (clean working tree)

git log --oneline phase-v1-JM-c -2
# Expect:
#   c6c43d819 feat(v1-JM-c): replace QUORUM const with snapshot read; defer APPEAL_WINDOW_DAYS to task 5 (task 2)
#   5e2f58181 feat(v1-JM-c): add ENTRY_KIND_JURY_DEADLOCK const + registry row (task 1)

grep -c "^pub const ENTRY_KIND_" crates/db_schema/src/source/governance/governance_log.rs
# Expect: 33

grep -c "^  ENTRY_KIND_" crates/api/api/src/governance/governance_log.rs
# Expect: 33

grep -n "QUORUM" crates/api/api/src/governance/submit_jury_vote.rs
# Expect: zero matches (const deleted in Task 2)

grep -n "APPEAL_WINDOW_DAYS" crates/api/api/src/governance/submit_jury_vote.rs
# Expect: 2 matches — line ~85 (const declaration with the deferral
# doc-comment) and line ~327 (the v0 closed_at usage that Task 5 removes)

grep -n "quorum_snapshot" crates/api/api/src/governance/submit_jury_vote.rs
# Expect: 2-3 matches — the SELECT, the .first::<Option<i32>>() unwrap,
# the i64::from() comparison
```

If any of those drift from expected, STOP and reconcile before Task 3.

---

## Bootstrap prompt for next session

Open a fresh Claude Code instance in
`C:/Users/barri/Developer/brehon-fork-phase-v1-JM-c`. Paste:

```
Resume v1-JM-c impl per .claude/PRPs/handovers/impl-2026-04-25-jm-c-task-3-onward.md.
Tasks 1 + 2 are committed; Task 3 is the resume point. Read the
handover brief first, run the closing-state assertions to confirm the
state matches, then continue per .claude/PRPs/plans/v1-jury-mechanics-c.plan.md §13 Task 3.
```

After the assertions pass, the next action is the Task 3 implementation
per plan §13 + the patterns at §10.3 + §10.4. Use `/prp-core:prp-implement`
or work the plan inline; either way the validation gates are the same
three cargo invocations shown above.

---

## What this handover does NOT cover

- The plan file itself (`.claude/PRPs/plans/v1-jury-mechanics-c.plan.md`)
  — that's canonical for everything substantive. This brief just covers
  resume-state, the Task 2 deviation, and the error-variant correction.
- BM-side state (PR cycle for JM-c) — JM-c PR not yet open. BM session
  cuts the PR after Task 8 retro commits per `branch-manager.md`.
- Advisor session state — advisor wrote the prp-skill-triggers
  handover (also still relevant if you want skill-discipline context)
  and is otherwise standing by for DQ answers, not actively monitoring
  impl.
- Cargo log bodies — see the `.claude/PRPs/debug/v1-JM-c-task<N>-*.log`
  files on disk; tails are 20 lines per `no-cargo-output-paste.md`.

---

_Brief author: impl session 2026-04-25 mid-session. Branch
`phase-v1-JM-c` HEAD `c6c43d819`. Working tree clean. Resume at Task 3._
