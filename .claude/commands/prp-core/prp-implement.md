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

**ADR-Hardness**: If anything in the plan contradicts an ADR in [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), **STOP** and surface to the user. Do not silently fix in the code.

---

## Brehon Context (read every invocation)

- `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` — cross-cutting requirements, test strategy, risk register
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` — table/enum/struct/route/handler authority (LIVING; current v0+v1 schema from live code, CODE WINS on discrepancy)
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — hard constraints
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
- **Source** — which [IMPLEMENTATION-PLAN-v0.md §3 Phase](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md), which ADRs apply
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

### 1.4 Detect Already-Completed Tasks (resumed-session safety)

Before starting work, check whether any plan tasks have already been committed on the current branch. This protects against duplicate-task commits when a session is resumed after a context-window reset, a branch is cut part-way through a phase, or an impl session picks up work another session started.

```bash
git log {base-branch}..HEAD --oneline
```

For each commit subject returned, match it against the plan's §13 "COMMIT MESSAGE" lines (one per task). The expected convention per `feedback_commit_hygiene_lockfiles_and_task_labels.md` is `feat(scope): <title> (task N)` or similar — the parenthesised task number is the match key.

Print one line per plan task:

```text
Task 1: ALREADY DONE (commit abc1234)
Task 2: ALREADY DONE (commit def5678)
Task 3: ALREADY DONE (commit 90ab12c)
Task 4: STARTING HERE
Task 5: pending
Task 6: pending
...
```

**STOP conditions** (surface to user before any file edit):

- A plan task has **two** matching commits (duplicate work already landed).
- A commit exists on the branch that does **not** match any plan task (scope drift or unrelated commit).
- Commits exist but none cite a task number verbatim (commit-hygiene failure — cannot safely resume; ask user which task to start at).
- The plan's §13 task list is missing COMMIT MESSAGE lines (plan is malformed for resume).

**Happy paths:**

- Zero commits since `{base-branch}` → start at Task 1.
- N contiguous commits matching Tasks 1..N → start at Task N+1.
- Non-contiguous matches (Task 1, 2, 4 done but not 3) → STOP and surface; ordering matters.

Per DQ #42 (v1-AD-d retro §2.1). Replaces the v1-AD-d-era manual `git log --oneline -8` scan with a structural guardrail. This step is cheap (~1 second) and catches a failure class the Phase 2.2 branch-decision matrix does not cover.

**PHASE_1_CHECKPOINT:**
- [ ] Plan file loaded
- [ ] Key sections identified
- [ ] Tasks list extracted
- [ ] Cross-cutting touches noted
- [ ] Already-completed tasks detected; resume-point printed; STOP conditions checked

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
3. Re-read the relevant [04](docs/brehon-law-inspired-network/04-data-model-and-api.md) section for the authoritative field list
4. If the task touches governance log / pseudonyms / `EmergencyRemove` / redaction, re-read [IMPLEMENTATION-PLAN-v0.md §4](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)

### 3.2 Implement

1. Make the change exactly as specified in the plan
2. Follow the MIRROR pattern verbatim
3. Respect every GOTCHA in the task
4. **Never inline `Uuid::new_v4()` for pseudonyms** — always call `actor_pseudonym::get_or_create(person_id)`
5. **Never write a raw string to `public_case_log` or `governance_log.payload`** — always go through `redaction::scrub(...)` + `governance_log::append(...)`
6. **Never use `_ =>` on a `match case.status` arm** — `CaseStatus::EmergencyRemove` must be handled explicitly ([ADR-013](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md))

#### Implementation skills — when they shorten the work

Two project skills exist to handle implementation patterns that recur often enough across Brehon impl sessions to be worth treating as their own discipline. Both are user-invocable (`/test-write`, `/edit-mechanical`) and authoritative on their own scope — defer to each skill's `## When to invoke` / `## Skip when` blocks for the conditions; the principles below name the trade-off they embody so you can decide whether to delegate.

- **`/test-write`** exists to enforce the e2e harness's `LemmyResult<()>` + pseudonymisation + no-`unwrap`/`expect` discipline that's easy to drift from when writing tests inline. Prefer it when adding new e2e cases under `crates/server/tests/e2e.rs` that need fixture scaffolding (golden path, error case, idempotency, constraint cascade). Inline is the right shape when the test is a `#[cfg(test)] mod tests` unit case that doesn't touch the harness, when extending an existing test with one extra assertion, or when the assertion patterns of the surrounding tests are already idiomatic and you'd be introducing skill ceremony without value.
- **`/edit-mechanical`** exists to make the rg-enumerate-first step a precondition for repeat-pattern edits — the R5.1 class of bug (PR #92 cr-12, JM-b Event 2) is what happens when a propagation skips enumeration. Prefer it when the change is a single repeated pattern across multiple call sites: adding a field to a struct with `derive(Default)`, renaming an enum variant, applying `#[expect(lint, reason="...")]` to N sites with the same shape, replacing a deprecated API call across known sites. Inline is the right shape when the edit needs type/borrow reasoning beyond the rename surface, when the change is one-of-a-kind, or when the surrounding context makes a single targeted Edit faster than the skill's enumerate→classify→edit→verify ceremony.

### 3.3 Validate Immediately

After **every file change**, run:

```bash
cargo check -p <affected-crate>
```

#### `/cargo-validate` — when the cargo run is the gating signal

The `/cargo-validate` skill exists to keep cargo's exit code intact (per `.claude/rules/cargo-output-capture.md`) and the conversation context lean (per `.claude/rules/no-cargo-output-paste.md`). Both concerns compound across a long implementation session: a piped `cargo ... 2>&1 | tail -40` masks the upstream exit code (cargo can fail and the surrounding tooling reports success), and 4 KB+ of cargo output pasted into the conversation per task drains the reasoning budget by mid-phase. The skill captures full output to a log under `.claude/build-*.log` (or `.claude/PRPs/debug/v1-<phase>-*.log` from a phase-branch worktree), tails the last 20 lines, returns the exit code as the report's headline.

Prefer it whenever the cargo run *is* the gating signal for a decision: per-task DoD checks after each Edit, plan §15 validation gates, the negative-probe check for wrapper sanity at session start (see [pre-phase-harness-audit.md](.claude/rules/pre-phase-harness-audit.md) §1 probe 4). Inline is the right shape when the cargo run is incidental (one-off scratch invocation you won't reference again), when an outer harness has already captured the output, or when the cargo run is itself a long-running background job — in that case use the cargo-runner background subagent instead. See `/cargo-validate` `## Skip when` for the canonical exclusion list.

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

### 4.1.1 HTTP Status Code Audit (if plan names specific HTTP statuses)

`LemmyError::status_code()` at `crates/utils/src/error.rs:223-230` only special-cases `IncorrectLogin → 401` and `NotFound → 404`. Every other `LemmyErrorType` variant — including `LemmyErrorType::Unknown` which is what `actix_web::error::ErrorConflict(...)` and friends flatten into — maps to **HTTP 400**. So a plan skeleton that uses `ErrorConflict`, `ErrorForbidden`, `ErrorGone`, etc. and expects a 4xx other than 400/401/404 will silently produce 400 at runtime.

**Audit the plan before writing tests** that assert on specific status codes:

```bash
# Find status-code references in the plan's §16 Acceptance Criteria and §9 DoD:
grep -E '\b(4[0-9][0-9]|5[0-9][0-9])\b' {plan-file}
```

For each status code named (e.g. `409 Conflict`, `403 Forbidden`, `410 Gone`):

1. Find the handler that should return it.
2. Check: does the handler return `HttpResponse::<Variant>()` directly (e.g. `HttpResponse::Conflict().body(...)`)?
   - **YES** → status flows through correctly. OK.
   - **NO** → handler routes via `LemmyError` / `actix_web::error::Error<Variant>`. That will map to **400**, not the plan's expected code. STOP: patch the handler to return the direct `HttpResponse` variant, or change the plan's expected status to 400.
3. Codes that route through `LemmyError` correctly by special-case: **401** (`IncorrectLogin`), **404** (`NotFound`). All others require direct `HttpResponse::<Variant>()`.

**Skip this step** if the plan's §16 only asserts `200 OK` / generic 4xx-or-5xx. Only run it when a specific non-{200,401,404} code is named.

Per DQ #43 (v1-AD-d retro §2.2). The bug shipped once in v1-AD-d where the plan skeleton used `ErrorConflict(...)` and the 409-asserting test would have failed with "expected 409 got 400" had the implementer trusted the skeleton.

### 4.2 Integration Tests

**You MUST write or update tests for new code.** Per [IMPLEMENTATION-PLAN-v0.md §5](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): integration-only, all in `tests/e2e.rs`.

#### 4.2.0 Docker daemon preflight (MANDATORY before every `cargo test --test e2e`)

The e2e harness spins up a real Postgres via testcontainers-rs. If the Docker daemon isn't running, the test failure is reported as `start_postgres: failed to create a container: Error in the hyper legacy client: client error (Connect)` — which reads like "Postgres container crashed" when the actual problem is "Docker Desktop is stopped." Silent-daemon-down has cost ~15 min of RCA diagnosis per occurrence; the probe below costs ~50ms.

Run this probe immediately before every e2e invocation:

```bash
docker ps > /dev/null 2>&1 || {
  echo "DOCKER NOT RUNNING — start Docker Desktop / dockerd before continuing"
  exit 1
}
```

If the probe fails, STOP and surface to the user — Docker Desktop on Windows has a habit of stopping on sleep/resume and this probe catches it at the right boundary. **Do not** attempt `cargo test --test e2e` before the probe passes.

Write tests, then run:

```bash
cargo test --test e2e {pattern-from-plan}
```

**Postgres in Docker requirement**: the e2e harness spins up a real Postgres per test run. **Always run the container with `--user $(id -u):$(id -g)`** to prevent root-owned files from blocking git worktree cleanup.

Per DQ #44 (v1-AD-d retro §2.3). See also `pre-phase-harness-audit.md` Probe 0 which runs the same check at phase start.

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

**Verify the table shape matches [04](docs/brehon-law-inspired-network/04-data-model-and-api.md)** — columns, types, indexes, constraints.

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
- [ ] Create PR: `/bm-pr` (Brehon four-role flow) or `gh pr create --base governance-v0`
- [ ] Mark the relevant phase in [IMPLEMENTATION-PLAN-v0.md](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) as done (in a separate commit in the homeserver repo — do NOT edit from inside brehon-fork)

---

## Follow-up GH issues (v1 / v2 limitations)

Scan the plan's **§3 (Out of scope)** and **§4.1 (Accepted limitations)** sections. For each bullet that is an intentional v1 scope decision but represents a real limitation a future reader might want to understand or fix, propose a `gh issue create` sketch. The implementer who just wrote the code has the exact tradeoff context in their head; filing the issue now beats advisor-inferred ad-hoc triage weeks later.

| Limitation | Issue title | Label | One-line body |
|---|---|---|---|
| {bullet from plan §3 or §4.1} | {suggested title ≤70 chars} | `v2-candidate` / `v1.5-candidate` | {why it matters + pointer to the code location} |

**If there are no §3/§4.1 limitations worth tracking**, write `(none — implementation covers the full plan scope)`.

Don't auto-create the issues — propose the sketches and let the user / advisor decide which to file. Many §3 bullets are permanent scope decisions that will never be "fixed" (e.g. "instance-wide juries are v2 territory per ADR-014"); those don't need an issue. Only file for bullets where a future implementer would benefit from a first-class TODO.

Per DQ #46 (v1-AD-d retro §3.2). Keeps the advisor's backlog fresh with implementer-written tracking issues where the code author's context is still warm.
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
2. Create PR: `/bm-pr` (Brehon four-role flow) or `gh pr create --base governance-v0`
3. Update [IMPLEMENTATION-PLAN-v0.md](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) phase status in the `homeserver` repo (separate commit)
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
- **ADR_INTACT**: Zero contradictions with [99 ADRs](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)
- **CROSS_CUTTING_INTACT**: Hash chain / pseudonyms / redaction / `EmergencyRemove` respected
- **REPORT_CREATED**: Implementation report exists
- **PLAN_ARCHIVED**: Original plan moved to completed
</content>
</invoke>