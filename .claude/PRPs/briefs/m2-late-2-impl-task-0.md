---
phase: m2-late-2
role: impl-task
n: 0
authored: 2026-06-12
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-late-2
task_number: 0
---

# [role:impl-task] m2-late-2 Task 0 — Pre-flight harness audit + branch verification

## 1. Role + dispatch line

```
[role:impl-task] m2-late-2 task-0 pre-flight — see .claude/PRPs/briefs/m2-late-2-impl-task-0.md
```

**PROBE-ONLY task. No commits. No code changes. No DQ entries unless a probe fails.**

## 2. Scope

Run all probes from plan §13 Task 0 and emit structured output. If all probes pass, output "PROBES: ALL PASS". If any probe fails, emit "PROBE FAIL: <probe-N> <reason>" and raise a `kind: "blocker"` DQ entry, then stop.

### Explicit boundaries

- **No commit.** Verification only.
- **No code changes** to `crates/`, `migrations/`, `tests/`, `docs/`, `services/bridge/`.
- **No DQ entries** unless a probe reports a blocker.

### Probes to run (in order, per plan §13 Task 0)

**Probe -1** — submodule init (lemmy_email build.rs quirk in worktrees):
```bash
git submodule status > /tmp/m2-late-2-task0-submodule.log 2>&1
if grep -q '^-' /tmp/m2-late-2-task0-submodule.log; then
  git submodule update --init --recursive >> /tmp/m2-late-2-task0-submodule.log 2>&1
  echo "submodule init exit: $?"
fi
echo "SUBMODULE OK"
```
EXPECT: no errors

**Probe 0** — Docker daemon (e2e uses testcontainers Postgres):
```bash
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "PROBE FAIL: 0 Docker not running"; exit 1; }
```
EXPECT: "DOCKER OK"

**Probe 1** — branch:
```bash
git branch --show-current
```
EXPECT: `phase-m2-late-2` (or the Junior worktree branch forked from it)

**Probe 2** — m2-late-1 base deliverables intact:
```bash
rg -n "applied: false" services/bridge/src/sanction_handler.rs && echo "PROBE 2a OK" || { echo "PROBE FAIL: 2a applied:false stub missing"; exit 1; }
rg -n "enqueue_sanction_event" crates/api/api/src/governance/sanction_publisher.rs && echo "PROBE 2b OK" || { echo "PROBE FAIL: 2b enqueue_sanction_event missing"; exit 1; }
```
EXPECT: both match present

**Probes 3, 4, 5 — DO NOT RUN on the worker (cargo is laptop-only).** The EliteDesk
daemon is **Linux** and has **no `cmd.exe` and no cargo on PATH**; the
`scripts\brehon\cargo-*.bat` wrappers are Windows-only and would fail to execute
(`cmd: command not found`), and running cargo on the daemon at all violates the
project HARD RULE **NO-CARGO-ON-ELITEDESK**. The plan §13 Task 0 lists Windows
`cmd //c "...cargo-check.bat..."` probes (3, 4, 5) — those are **laptop-advisor
pre-launch checks**, not worker probes. The advisor runs the workspace +
bridge-Linux + negative-guard cargo sanity **locally on the laptop before
dispatching this cohort** (the base is already known-green: workspace compiled at
the gate-1 DoD smoke test; bridge `Cargo.lock` is committed at `cdc97fda5` with
`time = "=0.3.47"` and verified `BRIDGE_FIX_EXIT_0`). **The worker SKIPS probes 3,
4, 5 entirely and proceeds to Probe 6.** Per the canonical `m1-b-impl-0.md` Task-0
pattern ("DO NOT run cargo / clippy / any `.bat` wrapper ... cargo NEVER runs on
the EliteDesk daemon"). Mirrors `project_laptop_canonical_cargo_runner.md`.

**Probe 6** — no concurrent PR on target files:
```bash
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName \
  --jq '.[] | select(.headRefName | test("phase-m2-late-2|ab-test/m2-late-2")) | {number, title, headRefName}'
```
EXPECT: empty (no active PRs on the phase or AB-test branches yet)

### Output format

After all probes:
```
PROBES: ALL PASS
  probe-(-1): SUBMODULE OK
  probe-0: DOCKER OK
  probe-1: branch=phase-m2-late-2
  probe-2: m2-late-1 base intact
  probe-3/4/5: SKIPPED (cargo is laptop-only; NO-CARGO-ON-ELITEDESK)
  probe-6: no concurrent PRs
```

If any probe fails: raise a `kind: "blocker"` DQ entry (per `decision-queue.md` Recipe 1), then stop.

## 3. Required reading

- `.claude/PRPs/plans/m2-late-2.plan.md` — §13 Task 0 (probe definitions; note probes 3/4/5 are laptop-side, NOT worker probes — see §4)
- `project_laptop_canonical_cargo_runner.md` (PMD) — NO-CARGO-ON-ELITEDESK; the worker runs no cargo
- `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` — bridge is Linux-only (for the advisor's pre-launch bridge sanity, not a worker probe)

## 4. Constraints

- **NO cargo on EliteDesk (NO-CARGO-ON-ELITEDESK)** — the worker runs ONLY the Linux-executable probes (submodule, docker, branch, rg base-intact, gh PR list). Probes 3/4/5 (workspace cargo, bridge cargo, negative-guard) are **laptop-advisor pre-launch checks, NOT worker probes** — the daemon is Linux with no cargo/cmd.exe, and cargo never runs on the daemon regardless. The worker SKIPS them. Per `project_laptop_canonical_cargo_runner.md` + the canonical `m1-b-impl-0.md` pattern.
- **Bridge cargo LINUX-ONLY (advisor-side)** — when the advisor runs the pre-launch bridge sanity locally, it MUST use `scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml` (Docker). Never `cd services/bridge && cargo check` on Windows (ruma-common E0119). Lockfile committed at `cdc97fda5`; the principle stays — bridge validates on the Linux CI mirror.
- **No commit at Task 0** — probes are read-only; a DQ blocker is the only valid output on failure.
- **DQ v3 id** — if raising a blocker DQ: generate id via `bash scripts/brehon/dq-v3-new-entry.sh`, commit + push before stopping.

### Mandatory lesson fires (file-class table check)

| File class | Match? | Lesson injected |
|---|---|---|
| `crates/server/tests/e2e.rs` edits | ❌ no edits | — |
| `scripts/brehon/cargo-*.bat\|sh` (worker-invoked) | ❌ worker runs NO cargo (probes 3/4/5 are laptop-side) | — (NO-CARGO-ON-ELITEDESK) |
| bridge `cargo-linux.sh` (worker-invoked) | ❌ bridge sanity is advisor-side pre-launch | — |
