---
phase: test
role: impl-task
n: 0
authored: 2026-06-11
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-test
task_number: 0
---

# [role:impl-task] test Task 0 — Pre-flight harness audit + branch verification

## 1. Role + dispatch line

```
[role:impl-task] test task-0 pre-flight — see .claude/PRPs/briefs/test-impl-task-0.md
```

**PROBE-ONLY task. No commits. No code changes. No DQ entries unless a probe fails.**

## 2. Scope

Run all probes from plan §13 Task 0 and emit a structured output. If all probes pass, output "PROBES: ALL PASS". If any probe fails, emit "PROBE FAIL: <probe-N> <reason>" and raise a `kind: "blocker"` DQ entry, then stop.

### Explicit boundaries

- **No commit.** Verification only.
- **No code changes** to `crates/`, `migrations/`, `tests/`, `docs/`.
- **No DQ entries** unless a probe reports a blocker.

### Probes to run (in order, per plan §13 Task 0)

**Probe -1** — submodule init (lemmy_email build.rs quirk in worktrees):
```bash
git submodule status > /tmp/test-task0-submodule.log 2>&1
if grep -q '^-' /tmp/test-task0-submodule.log; then
  git submodule update --init --recursive >> /tmp/test-task0-submodule.log 2>&1
  echo "submodule init exit: $?"
fi
echo "SUBMODULE OK"
```
EXPECT: no errors

**Probe 1** — branch verification:
```bash
git branch --show-current
```
EXPECT: `phase-test` (or `junior/<task-slug>` worktree branch forked from it — either is OK)

**Probe 2** — target module does NOT already exist:
```bash
test ! -f crates/utils/src/sandbox.rs && echo "SANDBOX ABSENT OK" || { echo "PROBE FAIL: 2 sandbox.rs already exists"; exit 1; }
```
EXPECT: "SANDBOX ABSENT OK"

**Probe 3** — declaration anchor present:
```bash
rg -n '^pub mod error;' crates/utils/src/lib.rs
```
EXPECT: exactly one match (the anchor for `pub mod sandbox;`)

**Probe 4** — anchor uniqueness (Edit old_string must be unique):
```bash
test "$(rg -c '^pub mod error;' crates/utils/src/lib.rs)" = "1" && echo "ANCHOR UNIQUE OK" || { echo "PROBE FAIL: 4 anchor not unique"; exit 1; }
```
EXPECT: "ANCHOR UNIQUE OK"

**Probe 5** — workspace clippy deny list present:
```bash
rg -n 'tests_outside_test_module|as_conversions|unwrap_used' Cargo.toml | head
```
EXPECT: three deny lines (informational — confirms §7 guardrails apply)

**Probe 6** — Shape G workflow YAML present:
```bash
test -f .github/workflows/cargo-validate-workspace.yml && echo "WORKFLOW PRESENT OK" || { echo "PROBE FAIL: 6 workspace-check workflow missing"; exit 1; }
```
EXPECT: "WORKFLOW PRESENT OK"

**Probe 7** — concurrent-PR check:
```bash
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName \
  --jq '.[] | select(.headRefName | test("test|sandbox")) | {number, title, headRefName}'
```
EXPECT: empty output (no conflicting open PRs)

## 3. Required reading

- `.claude/PRPs/plans/test.plan.md` §13 Task 0 — canonical probe list
- `.claude/rules/decision-queue.md` — if any probe fails and you raise a DQ entry, use `kind: "blocker"`, `from: "impl"`, `answered_by: null`

## 4. Constraints

- `answered_by` on any DQ entry: `"impl-self-resolved"` or `null` — NEVER `"advisor"` / `"user"`.
- Do NOT author impl code. Do NOT edit `crates/**`.
- Output "PROBES: ALL PASS" on success. No commit, no push.
- LESSON: trailer in the task completion output per PMD invariant #4 if any non-obvious finding surfaces.
