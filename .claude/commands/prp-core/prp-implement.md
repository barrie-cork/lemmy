---
description: Execute a Brehon implementation plan with rigorous cargo-based validation loops
argument-hint: <path/to/plan.md> [--base <branch>]
---

# Implement Plan (Brehon)

**Plan**: $ARGUMENTS

---

## Your Mission

Execute the plan end-to-end with rigorous self-validation. You are autonomous.

**Core Philosophy**: Validation loops catch mistakes early. Run `cargo check -p <crate>` after every file change. Fix issues immediately. The goal is a working Rust implementation that respects all 15 committed ADRs, not just code that exists.

**Golden Rule**: If a validation fails, fix it before moving on. Never accumulate broken state.

**ADR-Hardness**: If anything in the plan contradicts an ADR in [99](C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\99-decisions-and-open-questions.md), **STOP** and surface to the user. Do not silently fix in the code.

---

## Brehon Context (read every invocation)

- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\IMPLEMENTATION-PLAN-v0.md` — cross-cutting requirements, test strategy, risk register
- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\04-data-model-and-api.md` — table/enum/struct/route/handler authority
- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\99-decisions-and-open-questions.md` — hard constraints
- Fork-local `CLAUDE.md` — pinned upstream SHA, branch, command list

---

## Phase 0: DETECT — Project Environment

### 0.1 Confirm This Is a Rust Project

This fork is Lemmy 1.0-beta. The toolchain is cargo. Ignore the generic detection table from upstream PRP — **do not look for package.json, pyproject.toml, or bun.lockb**.

```bash
test -f Cargo.toml && echo "RUST OK"
test -f rust-toolchain.toml && cat rust-toolchain.toml
test -f diesel.toml && echo "DIESEL OK"
```

**Runner**: `cargo` for builds/tests, `diesel` for migrations, `gh` for PR operations.

### 0.2 Detect Base Branch

For branching and syncing within this fork:

1. **Check arguments**: if `$ARGUMENTS` contains `--base <branch>`, extract it
2. **Auto-detect from origin** (default `governance-v0` for v0 work):
   ```bash
   git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@'
   ```
3. **Fallback**:
   ```bash
   git remote show origin 2>/dev/null | grep 'HEAD branch' | awk '{print $NF}'
   ```
4. **Last resort**: `governance-v0` (the Brehon working branch, NOT `main`)

**Store as `{base-branch}`** — use for all diffs, rebases, PR creation.

### 0.3 Identify Validation Commands

The plan's **Validation Commands** section is authoritative. The standard set is:

| Level | Command | Expect |
|---|---|---|
| Static | `cargo check --workspace` | Exit 0 |
| Lint | `cargo clippy --workspace -- -D warnings` | Exit 0, zero warnings |
| Tests | `cargo test --test e2e {pattern}` | All green |
| Build | `cargo build --workspace` | Exit 0 |
| Migration (if schema) | `diesel migration run && diesel migration redo` | Round-trip works |

If the plan specifies different commands, use those.

---

## Phase 1: LOAD — Read the Plan

### 1.1 Load Plan File

```bash
cat $ARGUMENTS
```

### 1.2 Extract Key Sections

- **Summary** — scope
- **Source** — which [IMPLEMENTATION-PLAN-v0.md §3 Phase](C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\IMPLEMENTATION-PLAN-v0.md), which ADRs apply
- **Patterns to Mirror** — Rust snippets to copy
- **Files to Change** — CREATE/UPDATE list with exact paths
- **Step-by-Step Tasks** — implementation order (each one is a commit)
- **Validation Commands** — use these, not defaults
- **Acceptance Criteria** — definition of done
- **Cross-Cutting Touches** — does this phase touch the hash chain, `actor_pseudonym`, redaction, or `EmergencyRemove`?

### 1.3 Validate Plan Exists

If plan not found:
```
Error: Plan not found at $ARGUMENTS

Create one first: /prp-plan "Phase N — <name from IMPLEMENTATION-PLAN-v0.md §3>"
```

**PHASE_1_CHECKPOINT:**
- [ ] Plan file loaded
- [ ] Key sections identified
- [ ] Tasks list extracted
- [ ] Cross-cutting touches noted

---

## Phase 2: PREPARE — Git State

### 2.1 Check Current State

```bash
git branch --show-current
git status --porcelain
git worktree list
git remote -v
```

**Expect**: `origin` = `barrie-cork/lemmy`, `upstream` = `LemmyNet/lemmy`. If not, something is wrong — stop and report.

### 2.2 Branch Decision

| Current State | Action |
|---|---|
| In a worktree | Use it. Log "Using worktree at {path}" |
| On `{base-branch}` (governance-v0), clean | Create feature branch: `git checkout -b feature/{plan-slug}` |
| On `{base-branch}`, dirty | STOP: "Stash or commit changes first" |
| On a feature branch already | Use it. Log "Using existing branch {name}" |
| On `main` | STOP: `main` tracks upstream Lemmy. Switch to `governance-v0` or a feature branch |

### 2.3 Sync with Upstream

For v0 work, keep `governance-v0` loosely in sync with `upstream/main`:

```bash
git fetch upstream
git fetch origin
# Rebase the feature branch onto governance-v0 if behind
git pull --rebase origin {base-branch} 2>/dev/null || true
```

**Do NOT rebase onto `upstream/main` automatically** — that's a weekly discipline the user drives, not an implementation-step action.

**PHASE_2_CHECKPOINT:**
- [ ] On correct branch (not `main`, not `governance-v0` with uncommitted work)
- [ ] Working directory ready
- [ ] Origin + upstream remotes verified

---

## Phase 3: EXECUTE — Implement Tasks

For each numbered task in the plan's **Step-by-Step Tasks** section:

### 3.1 Read Context

1. Read the **MIRROR** file reference — understand the Rust pattern
2. Read any **IMPORTS** specified
3. Re-read the relevant [04](C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\04-data-model-and-api.md) section for the authoritative field list
4. If the task touches governance log / pseudonyms / `EmergencyRemove` / redaction, re-read [IMPLEMENTATION-PLAN-v0.md §4](C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\IMPLEMENTATION-PLAN-v0.md)

### 3.2 Implement

1. Make the change exactly as specified in the plan
2. Follow the MIRROR pattern verbatim
3. Respect every GOTCHA in the task
4. **Never inline `Uuid::new_v4()` for pseudonyms** — always call `actor_pseudonym::get_or_create(person_id)`
5. **Never write a raw string to `public_case_log` or `governance_log.payload`** — always go through `redaction::scrub(...)` + `governance_log::append(...)`
6. **Never use `_ =>` on a `match case.status` arm** — `CaseStatus::EmergencyRemove` must be handled explicitly ([ADR-013](C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\99-decisions-and-open-questions.md))

### 3.3 Validate Immediately

After **every file change**, run:

```bash
cargo check -p <affected-crate>
```

**If it fails:**
1. Read the error (Rust errors are usually precise — trust them)
2. Fix the issue
3. Re-run `cargo check`
4. Only proceed when passing

**Common Rust failures and fast fixes:**

| Error | Fix |
|---|---|
| `unresolved import` | Add `pub mod xx;` + `pub use` in `mod.rs` |
| `non-exhaustive patterns: CaseStatus::EmergencyRemove` | Add the arm explicitly — do NOT `_ =>` |
| `trait bound not satisfied` in Diesel | Check `Queryable` vs `Selectable`, ensure `table_name` matches `schema.rs` |
| `cannot find macro table! in scope` | Regen `schema.rs` via `diesel print-schema > crates/db_schema/src/schema.rs` |
| `async fn` signature mismatch | Check the existing handler pattern in `crates/api/api/src/` — param order matters |

### 3.4 Track Progress

Log each task as you complete it:

```
Task 1: CREATE migrations/{ts}_add_governance_core/up.sql   ✅
Task 2: CREATE crates/db_schema/src/source/governance/moderation_case.rs   ✅
Task 3: UPDATE crates/db_schema/src/source/governance/mod.rs   ✅
```

**Deviation Handling:** if you must deviate from the plan, note WHAT changed and WHY. Continue with the deviation documented for the implementation report.

**PHASE_3_CHECKPOINT:**
- [ ] All tasks executed in order
- [ ] Each task passed `cargo check` before the next started
- [ ] Cross-cutting helpers used (hash chain / pseudonym / redaction / EmergencyRemove)
- [ ] Deviations documented

---

## Phase 4: VALIDATE — Full Verification

### 4.1 Static Analysis

```bash
cargo check --workspace
cargo clippy --workspace -- -D warnings
```

**Must pass with zero errors AND zero warnings.**

If clippy complains:
1. Try `cargo clippy --fix --workspace --allow-dirty` for auto-fixable lints
2. Manually fix anything clippy couldn't auto-fix
3. **Never suppress with `#[allow(...)]` unless the clippy lint is genuinely wrong** — explain in a code comment if you do

### 4.2 Integration Tests

**You MUST write or update tests for new code.** Per [IMPLEMENTATION-PLAN-v0.md §5](C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\IMPLEMENTATION-PLAN-v0.md): integration-only, all in `tests/e2e.rs`.

Write tests, then run:

```bash
cargo test --test e2e {pattern-from-plan}
```

**Postgres in Docker requirement**: the e2e harness spins up a real Postgres per test run. **Always run the container with `--user $(id -u):$(id -g)`** to prevent root-owned files from blocking git worktree cleanup.

**If tests fail:**
1. Read the failure output (cargo test shows clear panics)
2. Determine: bug in implementation or bug in test?
3. Fix the actual issue (usually implementation)
4. Re-run
5. Repeat until green

### 4.3 Build Check

```bash
cargo build --workspace
```

**Must complete without errors.** Warnings from `cargo build` are uncommon if clippy is clean, but investigate any that appear.

### 4.4 Migration Round-Trip (if schema changed)

```bash
diesel migration run
diesel migration redo
psql -h localhost -U lemmy -d lemmy_test -c '\d {new_table_name}'
```

**Verify the table shape matches [04](C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\04-data-model-and-api.md)** — columns, types, indexes, constraints.

### 4.5 Cross-Cutting Verification (if applicable)

If the phase touched the governance log or pseudonyms:

- [ ] `cargo test --test e2e governance_log_hash_chain_holds` — hash chain still verifies
- [ ] `cargo test --test e2e redaction_strips_identifiers` — redaction still strips
- [ ] Manual grep: `grep -rn 'person_id.*governance_log\|governance_log.*person_id' crates/` — should return zero hits (pseudonym-only)
- [ ] Manual grep: `grep -rn 'CaseStatus::' crates/ | grep -v 'EmergencyRemove'` — spot-check all `CaseStatus` matches cover the variant

**PHASE_4_CHECKPOINT:**
- [ ] `cargo check --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --test e2e` all green
- [ ] `cargo build --workspace` succeeds
- [ ] Migration round-trip works (if schema changed)
- [ ] Cross-cutting verification passes (if log/pseudonyms touched)

---

## Phase 5: REPORT — Create Implementation Report

### 5.1 Create Report Directory

```bash
mkdir -p .claude/PRPs/reports
```

### 5.2 Generate Report

**Path**: `.claude/PRPs/reports/{plan-name}-report.md`

```markdown
# Implementation Report

**Plan**: `$ARGUMENTS`
**Source**: {IMPLEMENTATION-PLAN-v0.md §3 Phase N | other}
**Branch**: `{branch-name}`
**Date**: {YYYY-MM-DD}
**Status**: {COMPLETE | PARTIAL}

---

## Summary

{Brief: what was implemented and which v0 step it advances}

---

## Assessment vs Reality

| Metric | Predicted | Actual | Reasoning |
|---|---|---|---|
| Complexity | {from plan} | {actual} | {why matched or differed} |
| Confidence | {from plan} | {actual} | {evidence from implementation} |

**Deviations from plan:** {list with rationale, or "None"}

---

## Tasks Completed

| # | Task | File | Status |
|---|---|---|---|
| 1 | {description} | `migrations/...` | ✅ |
| 2 | {description} | `crates/db_schema/src/source/governance/...` | ✅ |

---

## Validation Results

| Check | Result | Details |
|---|---|---|
| `cargo check --workspace` | ✅ | 0 errors |
| `cargo clippy --workspace -- -D warnings` | ✅ | 0 warnings |
| `cargo test --test e2e` | ✅ | {N} passed |
| `cargo build --workspace` | ✅ | Compiled |
| Migration round-trip | ✅ / ⏭️ | {result or "N/A"} |
| Cross-cutting verification | ✅ / ⏭️ | {result or "N/A"} |

---

## Files Changed

| File | Action | Lines |
|---|---|---|
| `migrations/...` | CREATE | +{N} |
| `crates/db_schema/src/source/governance/xx.rs` | CREATE | +{N} |

---

## Cross-Cutting Impact

- [ ] Hash chain appends wired up for new writes
- [ ] `actor_pseudonym::get_or_create` used (no direct `person_id`)
- [ ] `redaction::scrub` called on all strings reaching the log
- [ ] `CaseStatus::EmergencyRemove` exhaustively matched
- [ ] AGPL notice unchanged (if release artefacts weren't produced)

---

## Issues Encountered

{List, or "None"}

---

## Tests Written

| Test | Validates |
|---|---|
| `{test_name}` | {what} |

---

## Next Steps

- [ ] Review implementation
- [ ] Create PR: `/prp-pr` (if ready)
- [ ] Mark the relevant phase in [IMPLEMENTATION-PLAN-v0.md](C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\IMPLEMENTATION-PLAN-v0.md) as done (in a separate commit in the homeserver repo — do NOT edit from inside brehon-fork)
```

### 5.3 Archive Plan

```bash
mkdir -p .claude/PRPs/plans/completed
mv $ARGUMENTS .claude/PRPs/plans/completed/
```

**PHASE_5_CHECKPOINT:**
- [ ] Report created
- [ ] Plan archived

---

## Phase 6: OUTPUT — Report to User

```markdown
## Implementation Complete

**Plan**: `$ARGUMENTS`
**Branch**: `{branch-name}`
**Status**: ✅ Complete

### Validation

| Check | Result |
|---|---|
| `cargo check --workspace` | ✅ |
| `cargo clippy -- -D warnings` | ✅ |
| `cargo test --test e2e` | ✅ ({N} passed) |
| `cargo build --workspace` | ✅ |

### Files

- {N} files created
- {M} files updated
- {K} integration tests written

### Cross-Cutting

{Which of hash chain / actor_pseudonym / EmergencyRemove / redaction were touched, or "None"}

### Deviations

{Summary, or "Implementation matched the plan."}

### Artifacts

- Report: `.claude/PRPs/reports/{name}-report.md`
- Plan archived to: `.claude/PRPs/plans/completed/`

### Next Steps

1. Review the report (especially deviations)
2. Create PR: `/prp-pr` or `gh pr create --base governance-v0`
3. Update [IMPLEMENTATION-PLAN-v0.md](C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\IMPLEMENTATION-PLAN-v0.md) phase status in the `homeserver` repo (separate commit)
```

---

## Handling Failures

### `cargo check` Fails

1. Read the rustc error (they're precise)
2. Fix the type/import issue
3. Re-run `cargo check`
4. Never proceed until passing

### Clippy Fails

1. Try `cargo clippy --fix --workspace --allow-dirty` for auto-fixable lints
2. Manually fix the rest
3. Do not suppress with `#[allow(...)]` unless justified and commented

### Integration Test Fails

1. Read the panic
2. Determine: implementation bug vs test bug (usually implementation)
3. Fix the root cause
4. Re-run
5. If tests mock the DB, STOP — integration tests MUST hit a real Postgres (user preference — mocked DB tests have masked broken migrations in prior projects)

### Migration Fails

1. `diesel migration revert` to get back to a known-good state
2. Fix the SQL
3. `diesel migration run` again
4. Then `diesel migration redo` to verify round-trip

### Build Fails

1. Usually a type or import issue already caught by `cargo check` — rare to see at `cargo build` stage
2. Check `Cargo.lock` for dependency mismatches after upstream rebases

### Integration with Lemmy Core Breaks

1. Check if `upstream/main` moved (`git fetch upstream && git log upstream/main..HEAD --oneline`)
2. If yes, rebase is needed — stop and hand back to the user (upstream rebases are a manual discipline)

---

## Success Criteria

- **TASKS_COMPLETE**: All plan tasks executed
- **CHECK_PASS**: `cargo check --workspace` exits 0
- **CLIPPY_PASS**: `cargo clippy --workspace -- -D warnings` exits 0
- **TESTS_PASS**: Integration tests all green
- **BUILD_PASS**: `cargo build --workspace` succeeds
- **ADR_INTACT**: Zero contradictions with [99 ADRs](C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\99-decisions-and-open-questions.md)
- **CROSS_CUTTING_INTACT**: Hash chain / pseudonyms / redaction / `EmergencyRemove` respected
- **REPORT_CREATED**: Implementation report exists
- **PLAN_ARCHIVED**: Original plan moved to completed
</content>
</invoke>