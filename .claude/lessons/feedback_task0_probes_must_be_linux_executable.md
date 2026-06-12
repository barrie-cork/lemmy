---
name: Task-0 pre-flight probes must be Linux-executable (the daemon worker is Linux) — cargo sanity is laptop-side
description: A plan's §13 Task-0 probes (and the impl-task-0 brief derived from them) run on the EliteDesk Junior worker, which is LINUX (no cmd.exe, no cargo on PATH, memory-constrained). A probe written as Windows `cmd //c "scripts\brehon\cargo-*.bat ..."` (a) cannot execute on the daemon (`cmd: command not found`) → PROBE FAIL → the whole /auto-phase catch-fires on Task 0, AND (b) violates NO-CARGO-ON-ELITEDESK. Worker probes must be Linux-pure read-only ops (git/grep/gh/docker/rg); cargo base-sanity is a laptop-advisor pre-launch check or a validate-pending-laptop DQ. Mirror m1-b-impl-0.md.
type: feedback
---

## TL;DR

The EliteDesk Junior daemon is **Linux** (`uname` → `Linux ... Ubuntu`), has **no
`cmd.exe`** and **no `cargo` on PATH**. A Task-0 pre-flight probe written in the
Windows form —

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > ... 2>&1"
```

— fails on the daemon worker with `cmd: command not found`, the probe's
`[ $EXIT -eq 0 ]` guard fires `PROBE FAIL: 3`, the worker raises a `kind: blocker`
DQ and stops, and **the entire `/auto-phase --unattended` catch-fires on Task 0**
before any real work begins. Even if `cmd.exe`/cargo existed, running cargo on the
daemon violates the project HARD RULE **NO-CARGO-ON-ELITEDESK**
(`project_laptop_canonical_cargo_runner.md`).

**Rule:** Task-0 (and any worker-executed probe) contains ONLY Linux-pure
read-only operations: `git branch`, `git submodule status`, `rg`/`grep` for
base-deliverable intactness, `docker ps`, `gh pr list`. **Cargo base-sanity
(workspace check, bridge `cargo-linux.sh` check, negative-exit-guard) is a
laptop-advisor pre-launch check or a `validate-pending-laptop` DQ — NEVER a worker
probe.** This mirrors the canonical `m1-b-impl-0.md` Task-0 brief:

> "DO NOT run cargo / clippy / any `.bat` wrapper. M1 is pre-Shape-G; per the
> project HARD RULE, cargo NEVER runs on the EliteDesk daemon — it runs only on
> the laptop via the validate-pending-laptop handler. The
> `scripts\brehon\cargo-*.bat` wrappers are Windows-only and will fail on the
> Linux daemon regardless."

## Incident (2026-06-12, m2-late-2)

The m2-late-2 plan §13 Task 0 listed probes 3/4/5 in the Windows
`cmd //c "...cargo-check.bat..."` form. The advisor-authored impl-task-0 brief
faithfully copied them into worker-executed probes. The defect was caught at
launch-readiness verification with a **6-minute margin** before a concurrent
session fired `/auto-phase`: the brief was fixed (`6da6b651f`, probes 3/4/5 →
laptop-side SKIP) and synced to the phase tip (`266a2dc94`) at 12:52; task #661
was created at 12:58 and the worker's run log confirmed it forked from the FIXED
tip and emitted `probe-3/4/5: SKIPPED (cargo is laptop-only; NO-CARGO-ON-ELITEDESK)`
instead of catch-firing. Without the fix, #661 forks the broken brief and dies
at Probe 3.

The defect originates in the **plan** (the planner wrote Windows cargo probes),
inherited by the **brief** (advisor copied them). Fixing only the brief leaves the
plan as a re-infection source for any future re-read.

## How to apply

- **Planner (`/prp-plan`, `.claude/agents/planning.md`):** Task-0 probes MUST be
  Linux-executable read-only ops. Do NOT write `cmd //c "...cargo-*.bat..."` or
  `cd services/bridge && cargo` as a Task-0 (worker) probe. Cargo base-sanity goes
  in a separate "advisor pre-launch checks" note, NOT in the worker probe list. If
  a base compile-sanity is genuinely needed before impl, express it as a
  `validate-pending-laptop` the advisor runs locally — not a worker probe.
- **Advisor (brief authorship):** when deriving the impl-task-0 brief from the
  plan, walk the Task-0 probe list. Any `cmd //c`/`.bat`/`cd services/bridge && cargo`
  worker probe is a defect — rewrite it as a laptop-side SKIP block (mirror
  `m1-b-impl-0.md`) and note in §4 that cargo is laptop-only. The
  `brief-worker-cargo-guard.sh` PreToolUse hook WARNs on this at commit time.
- **Advisor (§3.4 DoD smoke test):** a §15 or Task-0 cargo command written in the
  Windows `cmd //c` form for the *worker* is a DoD-issue — flag for the planner to
  re-point.

## Why this is an enforcement gap, not a decision gap

The NO-CARGO-ON-ELITEDESK rule is already stated in CLAUDE.md HARD RULES,
`.claude/agents/impl-task.md`, the forbidden-window check, and
`project_laptop_canonical_cargo_runner.md`. Nobody disputes it — it gets violated
by accident at authorship time. The durable fix is mechanical at the
authorship/commit boundary (this lesson + the planner watchpoint + the
`brief-worker-cargo-guard.sh` hook), NOT a new statement of the rule (an ADR would
add a 5th statement with zero new guard).

## See also

- `project_laptop_canonical_cargo_runner.md` (PMD) — NO-CARGO-ON-ELITEDESK.
- `.claude/PRPs/briefs/m1-b-impl-0.md` — the canonical correct Task-0 brief.
- `.claude/hooks/brief-worker-cargo-guard.sh` — the commit-time mechanical guard.
- `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo is Linux
  (Docker) on the laptop, never a worker probe.
- `.claude/PRPs/reports/session-retro-2026-06-12-launch-readiness-verify.md` — the
  incident this lesson is harvested from.
