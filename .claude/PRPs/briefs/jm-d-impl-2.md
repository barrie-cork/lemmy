---
role: impl-task
plan_task: 2
phase: v1-JM-d
created: 2026-04-27
related_dq: 55
---

# Brief — v1-JM-d Task 2 — Diesel models + InsertForm R3 sweep

## 1. Role + dispatch line

`[role:impl-task] v1-JM-d task 2 — see .claude/PRPs/briefs/jm-d-impl-2.md`

You are the **impl-task** subagent (Sonnet 4.6 per your frontmatter). Execute plan task 2 from `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` §13. **READ DQ #55 FIRST** — it overrides plan §13 task 2 step 2's command.

## 2. Scope

**Produce** (one commit per the plan's "one commit per task" rule):
- `crates/db_schema_file/src/enums.rs` — add `AppealRequesterRole` enum after line 722 per plan §10.3.
- `crates/db_schema_file/src/schema.rs` — regenerated with the four new column type mappings (see Step 2 override below).
- `crates/db_schema/src/source/governance/moderation_case.rs` — add `winning_decision: Option<JuryDecision>` to both `ModerationCase` (read model) and `ModerationCaseInsertForm` per §10.2.
- `crates/db_schema/src/source/governance/appeal.rs` — add three fields to `Appeal` + `AppealInsertForm` per §10.3 (`requester_role`, `panel_size_snapshot`, `threshold_count_snapshot`).
- `crates/db_schema/src/source/governance/jury_assignment.rs` — add `role: Option<JuryAssignmentRole>` to `JuryAssignmentInsertForm` per §10.5.
- R3 sweep: enumerate every `JuryAssignmentInsertForm {` literal in `crates/` and append `..Default::default()` to each. Plan expects 6 sites — verify by enumeration. **Verify line numbers via grep before editing — plan-cited line numbers may have drifted since plan write.**
- R3 sweep for `ModerationCaseInsertForm` and `AppealInsertForm`: enumerate via grep, add `..Default::default()` to sites that lack it (skip `request_appeal.rs:122-128` per plan §13 step 7 — Task 3 rewrites it).

**Do NOT** in this task:
- Touch `request_appeal.rs` (Task 3).
- Touch `submit_jury_vote.rs` step-8 winning_decision write (Task 3 / Task 6).
- Touch the appeal-window-expiry batch job (Tasks 5/6).
- Touch migrations (Task 1, already shipped at `08f2dad`).
- Author any new tests (Task 8 territory).

**Commit message** (exactly): `feat(v1-JM-d): extend Diesel models for appeals v1 + R3 sweep on InsertForm callers (task 2)`

## 3. Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entry #55** — load-bearing for Step 2. Substitution mechanism for the unexecutable plan command.
2. **Plan §13 Task 2** — the canonical step list. Execute steps 1, 3–7 verbatim; **override step 2** per DQ #55 + below.
3. **Plan §10.2** — exact `ModerationCase` + `ModerationCaseInsertForm` field shape (verbatim doc-comment text).
4. **Plan §10.3** — exact `AppealRequesterRole` enum derives + `Appeal` + `AppealInsertForm` field shape (verbatim doc-comment text). Mirror the `JuryAssignmentRole` enum at `crates/db_schema_file/src/enums.rs:704-722` for derive-attribute order.
5. **Plan §10.5** — `JuryAssignmentInsertForm` `role` field shape + R3 sweep target list. Mirror the JM-a drift-fix doc-comment block at `crates/db_schema/src/source/governance/jury_assignment.rs:48-55`.
6. **`scripts/update_schema_file.sh`** — the canonical schema-regen script. Read it before regenerating.
7. **Lessons** (Glob `.claude/lessons/`, Read any with filename keywords matching `insertform_default` / `lemmy_migration_runner` / `pipes_mask_exit_codes` / `pq-sys` / `mechanical-edit` / `clippy_test_style`):
   - `feedback_insertform_default_propagation.md` (option (b) — `..Default::default()`)
   - `feedback_lemmy_migration_runner.md` (DQ #55 context — CLI binary rejects args)
   - `feedback_pipes_mask_exit_codes.md` (capture-then-tail rule)
   - `feedback_clippy_test_style.md` (workspace clippy denies; tests must use `?`)
   - Any `feedback_pq_sys_*` or `feedback_features_full_*` lessons

## 4. Constraints

### Step 2 override (load-bearing — read DQ #55)

**The plan command is unexecutable as written:**
```
cargo run -p lemmy_diesel_utils --features full -- print-schema > schema.rs.new   # FAILS — binary rejects args
```

**Use this instead** (per `scripts/update_schema_file.sh:11`):

Two stages, in order:

```bash
# Stage A — apply the migrations from Task 1 (no args; the binary takes none)
cargo run --package lemmy_diesel_utils --features full > .claude/PRPs/debug/v1-JM-d-task2-migrate.log 2>&1
echo "exit: $?"; tail -5 .claude/PRPs/debug/v1-JM-d-task2-migrate.log
# Expected: exit 0; idempotent — Task 1's migrations already applied via DQ #55 manual psql, but this is a no-op safety re-apply

# Stage B — regenerate schema.rs via the diesel CLI (NOT lemmy_diesel_utils)
# diesel CLI is pre-installed on the EliteDesk daemon at /home/barrie/.cargo/bin/diesel
# (diesel 2.3.8, postgres backend). Verify with `which diesel`. If it's missing,
# DO NOT attempt to install — file a DQ pending entry; the install belongs to advisor preflight.
diesel print-schema > crates/db_schema_file/src/schema.rs.new 2> .claude/PRPs/debug/v1-JM-d-task2-printschema.log
echo "exit: $?"
mv crates/db_schema_file/src/schema.rs.new crates/db_schema_file/src/schema.rs
cargo +nightly fmt --package lemmy_db_schema_file > .claude/PRPs/debug/v1-JM-d-task2-fmt.log 2>&1
```

Verify the regen produced the four expected mappings:
```
grep -E 'requester_role|panel_size_snapshot|threshold_count_snapshot|winning_decision' \
  crates/db_schema_file/src/schema.rs > .claude/PRPs/debug/v1-JM-d-task2-grep-schema.log
```
Expected lines:
- `appeal::requester_role -> AppealRequesterRole`
- `appeal::panel_size_snapshot -> Nullable<Int4>`
- `appeal::threshold_count_snapshot -> Nullable<Int4>`
- `moderation_case::winning_decision -> Nullable<JuryDecision>`

### Plan-cited line numbers may have drifted

Plan §13 step 6 lists six call sites with exact line numbers (e.g. `admin_assign_jury.rs:234`). **Verify by `grep -n 'JuryAssignmentInsertForm \{'` before editing.** If line numbers shifted, follow the grep output, not the plan numbers. If the count is not 6, file a DQ pending entry — the schema may have drifted since plan write and the brief needs adjustment.

### R3 sweep discipline (from `feedback_insertform_default_propagation.md` option b)

For each `JuryAssignmentInsertForm { ... }` literal:
- If the literal already ends with `..Default::default()` — leave alone.
- Otherwise — append `..Default::default(),` before the closing `}`.
- Don't add `role: None,` — let `Default` fill it.

Same for `ModerationCaseInsertForm` and `AppealInsertForm` enumerations.

**Skip `request_appeal.rs:122-128` AppealInsertForm site** — Task 3 rewrites it with explicit `requester_role: Some(...)`.

### Memory-cap awareness (new since first attempt)

The daemon now runs under a cgroup memory cap: `MemoryMax=10G`, `MemoryHigh=8G` (deployed at `homeserver` repo `20f251b` after task #10's first attempt OOM-cascaded the EliteDesk on 2026-04-27). This cap propagates to your worker subtree.

- If `cargo check --workspace --features full` hits the cap, the cgroup OOM-killer terminates the worker process. Junior reports a non-zero exit; you'll see `Killed` in the log. **Do NOT retry blindly** — file a DQ pending entry with the cargo log tail and the `dmesg | grep oom` output if accessible. Advisor will decide whether to bump the cap or break the validation into per-crate invocations.
- The cap is high enough that workspace-wide checks *should* succeed on a clean cargo cache. If the cache is dirty/cold, the first build may peak higher than usual.
- `cargo install <anything>` is now an advisor-side responsibility — diesel_cli is pre-installed (verified at 11:06 UTC). Do not attempt other cargo installs without DQ-asking first.

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-JM-d` (currently at `08f2dad`). Finalize merges your worktree branch back; do not push to `phase-v1-JM-d` directly.
- One commit. The R3 sweep and the schema/InsertForm extensions go in the same commit. If clippy/check fails on the first attempt, amend or fixup; do not split the commit.
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch per `.claude/CLAUDE.md` cheatsheet. The advisor cannot read worktree-local state otherwise.
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Lesson trailer (encouraged)

Task 1 produced a high-value LESSON trailer about `lemmy_diesel_utils` CLI args (DQ #55). Task 2 is a candidate for similar — particularly if the schema-regen path or `diesel` CLI install surfaces anything plan-relevant. Per `feedback_junior_pmd_write_convention.md`, end the commit body with a single `LESSON:` line for any discrete future-relevant finding.

## 5. Validation gates (per plan §13 task 2 VALIDATE block)

Capture each to `.claude/PRPs/debug/v1-JM-d-task2-<probe>.log`. All exit-0.

1. `bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-JM-d-task2-check.log 2>&1` → exit 0.
2. `bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-d-task2-clippy.log 2>&1` → exit 0.
3. `bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-JM-d-task2-test-no-run.log 2>&1` → exit 0 (compile-only; tests don't run yet — Task 8 territory).

If any fails, **STOP and surface to advisor via DQ.** Do not patch around `cargo-check` or `clippy` failures by `#[allow]`-spamming — fix the root cause.

**GOTCHA from plan §13:** if `cargo check -p lemmy_db_schema --features full` fails after the schema regen with a `check_for_backend(diesel::pg::Pg)` error, the regen mismatched the `Queryable` derive's expected types. Re-run the regen and confirm the `winning_decision -> Nullable<JuryDecision>` mapping is exact.

**GOTCHA from `feedback_clippy_test_style.md`:** the workspace denies `expect_used`, `unwrap_used`, `allow_attributes` *including in tests*. Tests must use `?`. Don't add escape-hatches.

## 6. Expected output (return to advisor)

```
## Task 2 complete — JM-d Diesel models + R3 sweep

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/db_schema_file/src/enums.rs (+AppealRequesterRole)
  - crates/db_schema_file/src/schema.rs (regen — 4 new column mappings)
  - crates/db_schema/src/source/governance/{moderation_case,appeal,jury_assignment}.rs
  - <N> R3-swept call sites (expected 6+, verify by grep)
**Validation:** check / clippy / test-no-run all exit 0
**Next:** advisor queues task 3 (request_appeal.rs rewrite + reporter-rights branch + winning_decision write in submit_jury_vote)
```

Plus any DQ #N references if you raised one mid-task.

## 7. Why this brief differs from the plan

Two overrides documented above:
1. **Step 2 command replaced** — DQ #55 (resolved 2026-04-27) showed `lemmy_diesel_utils` binary rejects args; canonical regen path is `diesel print-schema` per `scripts/update_schema_file.sh:11`.
2. **Line-number verification required** — plan §13 step 6 cites exact line numbers; grep-first to handle any drift since plan write.

The retro brief (`.claude/PRPs/briefs/jm-d-retro.md`) captures this as an architectural lesson: plan-DoD smoke tests should run task-N VALIDATE blocks, not just §15.
