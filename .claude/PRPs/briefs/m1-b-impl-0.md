---
phase: m1-b
role: impl-task
n: 0
authored: 2026-06-04
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m1-b
task_number: 0
---

# [role:impl-task] m1-b Task 0 — Pre-flight harness audit + branch verification

## 1. Role + dispatch line

```
[role:impl-task] m1-b task-0 pre-flight harness audit — see .claude/PRPs/briefs/m1-b-impl-0.md
```

**This is a PROBE-ONLY task. No commits. No code changes. No DQ entries unless a probe fails.**

## 2. Scope

### What to produce

Run probes 0–6 listed in plan §13 Task 0 and emit a structured output report. If all probes pass, exit cleanly with "PROBES: ALL PASS — ready for T1". If any probe reports an unexpected result that blocks T1, emit "PROBE FAIL: <probe number> <reason>", raise a `kind: "blocker"` DQ entry (commit + push on `phase-m1-b`), then stop.

### Explicit boundaries

- **No commit.** This is verification-only; the task produces no git artifacts.
- **No code changes** to `crates/`, `migrations/`, `tests/`, `docs/`.
- **No DQ entries** unless a probe reports results that block T1.
- **DO NOT run cargo / clippy / any `.bat` wrapper.** M1 is pre-Shape-G; per the project HARD RULE, cargo NEVER runs on the EliteDesk daemon — it runs only on the laptop via the validate-pending-laptop handler. The `scripts\brehon\cargo-*.bat` wrappers are Windows-only and will fail on the Linux daemon regardless. **Probe 7 (clippy baseline) is PRE-SATISFIED — see Probe 7 note below; do NOT attempt it.**

### Probes to run (in order) — probes 0–6 only

Per plan §13 Task 0 (R5 — enumerate ALL probes explicitly). These are all read-only git/grep/gh operations that run fine on the daemon worker:

**Probe 0** — branch:
```bash
git branch --show-current
```
EXPECT: `phase-m1-b` (NOT `governance-v0`). If not on `phase-m1-b`, STOP immediately and raise a DQ blocker.

**Probe 1** — config table still ABSENT on base (M1 builds it):
```bash
grep -rl "governance_messaging_config" crates/db_schema*/src/ ; echo "exit: $?"
```
EXPECT: no match (exit 1).

**Probe 2** — `services/bridge` still ABSENT:
```bash
test ! -d services/bridge && echo "ABSENT OK" || echo "EXISTS — STOP"
```
EXPECT: `ABSENT OK`.

**Probe 3** — PM notification hooks intact:
```bash
grep -n "plugin_hook_notification" crates/api/api_utils/src/notify.rs
```
EXPECT: line :305 present (and typically :123, :348 — report whatever is found).

**Probe 4** — `person.rs` `matrix_user_id` intact:
```bash
grep -n "matrix_user_id" crates/apub/objects/src/protocol/person.rs
```
EXPECT: line :49 present.

**Probe 5** — workspace pulls zero matrix deps today (baseline):
```bash
grep -rl "matrix-sdk\|ruma" crates/*/Cargo.toml Cargo.toml ; echo "exit: $?"
```
EXPECT: no match (exit 1).

**Probe 6** — concurrent-PR check (no open PR touches §11 files):
```bash
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("messaging_config|services/bridge|api_utils/src/notify")) | {number,title}'
```
EXPECT: empty (clarify `-048` confirmed none at plan time). Report whatever is found.

**Probe 7** — clippy baseline: **PRE-SATISFIED — DO NOT RUN.**
The advisor already ran `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings` on the laptop at gate-1 plan-approval (result: exit 0, marker `CLIPPY_EXIT_0`, log `.claude/PRPs/debug/m1-gate1-clippy.log`). The advisor verified `git diff origin/governance-v0 origin/phase-m1-b -- crates/ migrations/ tests/` is EMPTY (the phase branch is byte-identical to `governance-v0` in all code paths — only meta-files differ). Therefore the clippy baseline carries over to `phase-m1-b` unchanged. Report `Probe 7: PRE-SATISFIED (laptop gate-1 CLIPPY_EXIT_0; phase branch code-identical to governance-v0)`. Do NOT invoke cargo.

### Output format

Emit a numbered result for each probe:
```
Probe 0: PASS (phase-m1-b)
Probe 1: PASS (no match, exit 1 — config table absent)
Probe 2: PASS (ABSENT OK)
Probe 3: PASS (notify.rs hooks at :305 [+ :123, :348])
Probe 4: PASS (person.rs matrix_user_id at :49)
Probe 5: PASS (no match, exit 1 — zero matrix deps)
Probe 6: PASS (open PRs touching §11 files: [list or none])
Probe 7: PRE-SATISFIED (laptop gate-1 clippy clean; phase branch code-identical)

PROBES: ALL PASS — ready for T1
```

If any probe FAILS, raise a DQ blocker (commit + push on `phase-m1-b`) and stop.

## 3. Required reading

- `.claude/PRPs/plans/m1.plan.md` §13 Task 0 (full probe list), §18 risks R3/R5 (context for the verify items)
- `.claude/rules/decision-queue.md` — DQ schema (only if a probe fails and you must raise a blocker)

## 4. Constraints

- **BRANCH must be `phase-m1-b`** — confirm with `git branch --show-current` as the first action (Probe 0). If not on this branch, STOP immediately and raise a DQ blocker.
- **No commit** — the task is probe-only. Do NOT call `git add` or `git commit` at any point (unless raising a DQ blocker on probe failure).
- **No DQ entries** unless a probe fails. Passing probes generate no DQ.
- **DO NOT run cargo / clippy / `.bat` wrappers** — pre-Shape-G no-cargo-on-daemon HARD RULE; Probe 7 is pre-satisfied (see above).
- **Attribution**: if you DO raise a DQ entry (probe failure), set `from: "impl"`, `kind: "blocker"`, use `bash scripts/brehon/dq-v3-new-entry.sh` for the id. NEVER `answered_by: "advisor"` from this session.
- **DQ mid-task push** (only on probe failure): `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised DQ #<id> — <slug>" && git push origin phase-m1-b`.
