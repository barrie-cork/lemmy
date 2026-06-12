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

**Probe 3** — workspace cargo-check wrapper (exit-code gated per `feedback_wrapper_script_flag_silence.md`):
```bash
mkdir -p .claude/PRPs/debug
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m2-late-2-task0-ws.log 2>&1"
WS_EXIT=$?
echo "workspace exit: $WS_EXIT"
tail -20 .claude/PRPs/debug/m2-late-2-task0-ws.log
[ $WS_EXIT -eq 0 ] || { echo "PROBE FAIL: 3 workspace cargo-check non-zero"; exit 1; }
```
EXPECT: exit 0

**Probe 4** — bridge cargo sanity via Docker Linux (R9 — LINUX-BRIDGE RIDER; never Windows-local):
```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml > /tmp/m2-late-2-task0-bridge.log 2>&1
BRIDGE_EXIT=$?
echo "bridge exit: $BRIDGE_EXIT"
tail -20 /tmp/m2-late-2-task0-bridge.log
[ $BRIDGE_EXIT -eq 0 ] || { echo "PROBE FAIL: 4 bridge cargo-linux.sh non-zero"; exit 1; }
```
EXPECT: exit 0. NOTE: First Docker pull is cold (~10–20 min). `Cargo.lock` committed at `cdc97fda5` with `time = "=0.3.47"` pin — should compile clean.

**Probe 5** — negative exit-code masking guard:
```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/m2-late-2-task0-neg.log 2>&1"
NEG_EXIT=$?
echo "neg exit: $NEG_EXIT"
[ $NEG_EXIT -ne 0 ] || { echo "PROBE FAIL: 5 wrapper masked non-zero exit"; exit 1; }
```
EXPECT: non-zero (wrapper correctly propagates cargo error)

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
  probe-1: SUBMODULE OK
  probe-0: DOCKER OK
  probe-1: branch=phase-m2-late-2
  probe-2: m2-late-1 base intact
  probe-3: workspace exit=0
  probe-4: bridge exit=0
  probe-5: neg exit=<nonzero>
  probe-6: no concurrent PRs
```

If any probe fails: raise a `kind: "blocker"` DQ entry (per `decision-queue.md` Recipe 1), then stop.

## 3. Required reading

- `.claude/PRPs/plans/m2-late-2.plan.md` — §13 Task 0 (probe definitions, EXPECT values)
- `.claude/lessons/feedback_wrapper_script_flag_silence.md` — cargo wrapper exit-code discipline (Probe 5)
- `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` — R9 Linux-only rule (Probe 4)
- `.claude/lessons/feedback_background_task_notification_lies.md` — if any probe runs in background, the notification exit code is unreliable; read the log file marker

## 4. Constraints

- **NO cargo on EliteDesk (NO-CARGO-ON-ELITEDESK)** — Probe 3 and Probe 5 use the `.bat` wrapper on the Windows worker host. Probe 4 uses `cargo-linux.sh` (Docker). Neither is `cargo` run directly on the daemon; the wrapper scripts isolate execution.
- **Bridge cargo LINUX-ONLY** — Probe 4 MUST use `scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml`. Never `cargo check --manifest-path services/bridge/Cargo.toml` directly (ruma-common E0119 on Windows host pre-lockfile-pin; lockfile is now committed but the principle stays — bridge validates on Linux CI mirror).
- **No commit at Task 0** — probes are read-only; a DQ blocker is the only valid output on failure.
- **DQ v3 id** — if raising a blocker DQ: generate id via `bash scripts/brehon/dq-v3-new-entry.sh`, commit + push before stopping.

### Mandatory lesson fires (file-class table check)

| File class | Match? | Lesson injected |
|---|---|---|
| `crates/server/tests/e2e.rs` edits | ❌ no edits | — |
| `scripts/brehon/cargo-*.bat\|sh` | ✅ Probes 3/4/5 invoke wrappers | `feedback_wrapper_script_flag_silence.md` (§3 ✓) |
| bridge `cargo-linux.sh` | ✅ Probe 4 | `feedback_bridge_validates_on_linux_not_windows.md` (§3 ✓) |
