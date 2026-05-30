---
phase: v1-quality-r3
role: impl-task
n: 0
authored: 2026-05-30
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-v1-quality-r3
task_number: 0
---

# [role:impl-task] v1-quality-r3 Task 0 — Pre-flight harness audit + branch verification + re-enumeration

## 1. Role + dispatch line

```
[role:impl-task] v1-quality-r3 task-0 pre-flight harness audit — see .claude/PRPs/briefs/v1-quality-r3-impl-task-0.md
```

**This is a PROBE-ONLY task. No commits. No code changes. No DQ entries unless a probe fails.**

## 2. Scope

### What to produce

Run all probes listed in plan §13 Task 0 and emit a structured output report. If all probes pass, exit cleanly with "PROBES: ALL PASS". If any probe fails, emit "PROBE FAIL: <probe number> <reason>" and raise a `kind: "blocker"` DQ entry, then stop.

### Explicit boundaries

- **No commit.** This is verification-only; the task produces no git artifacts.
- **No code changes** to `crates/`, `migrations/`, `tests/`, `docs/`.
- **No DQ entries** unless a probe reports unexpected results that block T1.
- Do NOT re-hoist `EnvVarGuard` — it is already at test-crate root (Probe 5 confirms).
- Do NOT wrap any `set_var` calls — T1/T2 are the wrapping tasks.

### Probes to run (in order)

Per plan §13 Task 0 — R5: enumerate ALL probes explicitly:

**Probe 0** — Docker daemon:
```bash
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }
```
EXPECT: "DOCKER OK"

**Probe 1** — cargo-check wrapper honours `-p`:
```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/PRPs/debug/v1-quality-r3-audit-check-p.log 2>&1"
echo "Probe 1 exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r3-audit-check-p.log
```
EXPECT: exit 0

**Probe 2** — feature flag activation:
```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/PRPs/debug/v1-quality-r3-audit-check-features.log 2>&1"
echo "Probe 2 exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r3-audit-check-features.log
```
EXPECT: exit 0

**Probe 3** — cargo-test wrapper honours target selection:
```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features full > .claude/PRPs/debug/v1-quality-r3-audit-test.log 2>&1"
echo "Probe 3 exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r3-audit-test.log
```
EXPECT: exit 0

**Probe 4** — exit-code propagation (negative test):
```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-quality-r3-audit-negative.log 2>&1"
echo "Probe 4 exit: $?"
```
EXPECT: exit NON-ZERO (wrapper correctly propagates cargo error exit)

**Probe 5** — EnvVarGuard struct present at test-crate root:
```bash
grep -n "^struct EnvVarGuard {" crates/server/tests/e2e.rs
grep -n "^mod governance_fixtures {" crates/server/tests/e2e.rs
```
EXPECT: `EnvVarGuard` struct found at approximately line 111; `governance_fixtures` mod found at approximately line 143 (struct PRECEDES the first mod)

**Probe 6** — re-enumerate setter counts (must match plan §3):
```bash
echo "INIT raw set_var sites:"
grep -c 'std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS"' crates/server/tests/e2e.rs
echo "GOV via set_var (any form):"
python3 -c "import re; print(len(re.findall(r'std::env::set_var\(\s*\"GOVERNANCE_LOG_SIGNING_KEY\"', open('crates/server/tests/e2e.rs').read())))"
echo "Already-wrapped EnvVarGuard INIT sites:"
grep -c 'EnvVarGuard::set.*LEMMY_INITIALIZE' crates/server/tests/e2e.rs
```
EXPECT: INIT raw = 13 (12 raw set_var + boot_context EnvVarGuard::set counted together by grep; actual raw set_var count should be 12), GOV = 11. Report both. If counts differ significantly from plan §3 (12 INIT raw + 11 GOV raw), STOP and raise DQ blocker.

Note: `grep -c` on INIT will also match the `EnvVarGuard::set("LEMMY_INITIALIZE...")` line at boot_context. The plan expects **12 raw** `std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS"` lines (sites: 833, 2568, 3347, 4114, 4459, 4788, 4923, 5047, 5675, 5876, 6146, 16823) + **1** `EnvVarGuard::set(...)` already wrapped. So grep -c total of ~13 is expected. Confirm the raw count is 12 by also counting just `std::env::set_var("LEMMY_INITIALIZE`.

**Probe N** — no concurrent PR touching e2e.rs:
```bash
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName \
  --jq '.[] | {number, title, headRefName}'
```
EXPECT: no open PRs on `phase-v1-*` branches other than this one. Report whatever is found.

**Probe 8** — clippy baseline on the full workspace:
```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r3-audit-clippy-baseline.log 2>&1"
echo "Probe 8 clippy baseline exit: $?"
tail -30 .claude/PRPs/debug/v1-quality-r3-audit-clippy-baseline.log
```
EXPECT: exit 0 (clean clippy baseline before any T1/T2 edits)

### Output format

Emit a numbered result for each probe:
```
Probe 0: PASS (DOCKER OK)
Probe 1: PASS (exit 0)
Probe 2: PASS (exit 0)
Probe 3: PASS (exit 0)
Probe 4: PASS (exit non-zero = 101)
Probe 5: PASS (EnvVarGuard at line XXX, governance_fixtures mod at line YYY)
Probe 6: PASS (12 INIT raw + 1 EnvVarGuard::set; 11 GOV raw)
Probe N: PASS (open PRs: [list or none])
Probe 8: PASS (exit 0)

PROBES: ALL PASS — ready for T1
```

If any probe FAILS, append to DQ and stop.

## 3. Required reading

- `.claude/PRPs/plans/v1-quality-r3.plan.md` §7 (guardrails R5-R10), §13 Task 0 (full probe list)
- `.claude/lessons/feedback_validate_pending_laptop_must_use_wrapper.md` — wrapper discipline (applies to Probes 1-4, 8)
- `.claude/lessons/feedback_envvarguard_fixture_lifetime_footgun.md` — understand T1/T2 split (context for Probe 6 expected counts)
- `.claude/rules/pre-phase-harness-audit.md` — if it exists; if not, follow plan §13 Task 0 probes verbatim

## 4. Constraints

- **BRANCH must be `phase-v1-quality-r3`** — confirm with `git branch --show-current` as first action. If not on this branch, STOP immediately and raise DQ blocker.
- **No commit** — the task is probe-only. Do NOT call `git add` or `git commit` at any point.
- **No DQ entries** unless a probe fails. Passing probes generate no DQ.
- **Use `.bat` wrappers** for all cargo invocations (R9). Never bare `cargo check/clippy/test` on Windows.
- **R10**: all cargo output redirected to `.claude/PRPs/debug/v1-quality-r3-audit-*.log` files.
- **Attribution**: if you DO raise a DQ entry (probe failure), set `from: "impl"`, `kind: "blocker"`. NEVER `answered_by: "advisor"` from this session.
