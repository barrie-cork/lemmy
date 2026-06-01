---
phase: v1-quality-r3
role: impl-task
n: 1
authored: 2026-05-30
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-v1-quality-r3
task_number: 1
---

# [role:impl-task] v1-quality-r3 Task 1 — SAFETY-comment 2 bootstrap footgun sites (option-b)

## 1. Role + dispatch line

```
[role:impl-task] v1-quality-r3 task-1 SAFETY-comment bootstrap footgun sites — see .claude/PRPs/briefs/v1-quality-r3-impl-task-1.md
```

## 2. Scope

### What to produce

**2 Edits** to `crates/server/tests/e2e.rs` — add `// SAFETY:` justification comment blocks immediately above the existing `unsafe {` in two fixture `bootstrap()` functions. The raw `set_var` calls stay verbatim; the comment IS the deliverable.

### Explicit boundaries

- **ONLY edits to `crates/server/tests/e2e.rs`** — exactly 2 Edits, one per site.
- Do **NOT** wrap these 2 sites with `EnvVarGuard` — that's the footgun (guard drops at `bootstrap()` return, unsetting vars before the test body runs). Raw `set_var` + SAFETY comment is the correct option-b treatment.
- Do **NOT** touch any test-body sites (those are T2).
- Do **NOT** touch the `boot_context()` site at ~line 17474/17475 (already wrapped — do NOT re-wrap).
- Do **NOT** touch `admin_config_fixtures` local `const SIGNING_SEED_HEX` — keep it exactly as-is.

### Files

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
requires: []
```

### The 2 edits (verbatim old_string / new_string)

**Edit 1 — `governance_fixtures::bootstrap` (site at ~line 831-835):**

`old_string` (verbatim from phase branch tip — must match exactly):
```
    unsafe {
      std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
      std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    }

    let (container, host_port) = start_postgres().await?;
```

`new_string`:
```
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    // These two env vars are intentionally process-scoped (NOT EnvVarGuard-wrapped):
    // both are constant-valued ("1" / fixed signing seed) and bootstrap() has many
    // callers across this test module — wrapping here would drop the guard at
    // bootstrap() return, unsetting the var before the test body runs (see
    // feedback_envvarguard_fixture_lifetime_footgun.md). LEMMY_DATABASE_URL IS
    // guarded (per-call value) at the _g_db_url binding below.
    unsafe {
      std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
      std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    }

    let (container, host_port) = start_postgres().await?;
```

**Edit 2 — `admin_config_fixtures::bootstrap` (site at ~line 6144-6150):**

`old_string` (verbatim from phase branch tip — must match exactly):
```
    const SIGNING_SEED_HEX: &str =
      "0000000000000000000000000000000000000000000000000000000000000001";
    unsafe {
      std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
      std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    }

    let (container, host_port) = super::governance_fixtures::start_postgres().await?;
```

`new_string`:
```
    const SIGNING_SEED_HEX: &str =
      "0000000000000000000000000000000000000000000000000000000000000001";
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    // These two env vars are intentionally process-scoped (NOT EnvVarGuard-wrapped):
    // both are constant-valued ("1" / fixed signing seed) and bootstrap() has many
    // callers across this test module — wrapping here would drop the guard at
    // bootstrap() return, unsetting the var before the test body runs (see
    // feedback_envvarguard_fixture_lifetime_footgun.md). LEMMY_DATABASE_URL IS
    // guarded (per-call value) at the _g_db_url binding below.
    unsafe {
      std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
      std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    }

    let (container, host_port) = super::governance_fixtures::start_postgres().await?;
```

### Uniqueness anchors (R11 — why these old_strings are unique)

- **Edit 1**: `start_postgres().await?` (no `super::` prefix) is present only in `governance_fixtures::bootstrap`. The `admin_config_fixtures::bootstrap` version uses `super::governance_fixtures::start_postgres()`. This distinguishes the two nearly-identical sites.
- **Edit 2**: `const SIGNING_SEED_HEX` immediately before the `unsafe {` (local const), plus `super::governance_fixtures::start_postgres()` after, makes this unique. The governance_fixtures version has `SIGNING_SEED_HEX` as a module-level `pub const` (outside the fn), not a local `const`.

## 3. Required reading

- `.claude/PRPs/plans/v1-quality-r3.plan.md` §10.3 (option-b SAFETY comment shape + canonical text)
- `.claude/PRPs/plans/v1-quality-r3.plan.md` §8 (RAII lifetime note — why bootstrap sites stay raw)
- `.claude/lessons/feedback_envvarguard_fixture_lifetime_footgun.md` — the guard-drop-at-return footgun; this is why option-b exists
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — verbatim anchor discipline (the verbatim old_strings above were pre-located from the phase branch tip per R11)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — (no new error types in T1, but applicable if you touch any Result path)
- `.claude/lessons/feedback_async_pool_test_pattern.md` — context for the test fixture pattern being modified
- `.claude/lessons/feedback_validate_pending_laptop_must_use_wrapper.md` — VALIDATE block uses `.bat` wrappers (R9)

## 4. Constraints

- **BRANCH must be `phase-v1-quality-r3`** — confirm with `git branch --show-current` as first action.
- **R11**: use the verbatim `old_string` / `new_string` above for each Edit. Pre-located from phase branch tip; do NOT re-derive or paraphrase the anchor text.
- **R9**: all cargo invocations use `scripts\brehon\cargo-check.bat` / `cargo-clippy.bat`. Never bare `cargo` on Windows.
- **R10**: all cargo output redirected to `.claude/PRPs/debug/v1-quality-r3-task1-*.log`.
- **Only 2 Edits** — no more. Do not touch any other lines.
- **Attribution**: set `from: "impl"` on any DQ entry. NEVER `answered_by: "advisor"` from impl session.
- DQ mid-task push: if you raise a DQ, immediately `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised DQ — <slug>" && git push origin phase-v1-quality-r3`.

### VALIDATE (after both edits, before commit)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r3-task1-check.log 2>&1"
echo "check exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r3-task1-check.log
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r3-task1-clippy.log 2>&1"
echo "clippy exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r3-task1-clippy.log
```

Both must exit 0. If not, DO NOT commit — raise a DQ blocker with the log tail.

### Commit (only after VALIDATE passes)

Subject: `refactor(e2e): document 2 bootstrap env-var process-scoping exceptions with SAFETY justification (task 1)`

Commit ONLY `crates/server/tests/e2e.rs`. Nothing else.

Push: `git push origin phase-v1-quality-r3`
