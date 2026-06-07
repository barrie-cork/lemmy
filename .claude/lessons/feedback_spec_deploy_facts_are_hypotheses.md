---
name: A spec's deploy/runtime section is a hypothesis, not fact — verify against the live system before acting
description: The deploy/runtime/operations section of a spec or patch contract (PIDs, restart commands, process-manager names, row/record counts, port numbers, "the service runs as X") is a snapshot captured when the spec was authored. Infra drifts on its own clock — a process manager gets swapped, a service is restarted under a new PID, a DB grows, a manual start becomes a systemd/NSSM unit. Treat every deploy-section fact as a hypothesis to verify against the live system in the same step you act on it. The 2026-06-07 read-pheromone session hit this twice from one spec — "PID 5900, restart manually" (was actually NSSM `Restart-Service pmd-http-mcp`) and "579 rows" (was 767) — both caught only by a live re-check, not by trusting the spec.
type: feedback
---

## TL;DR

When a spec, patch contract, or implementation plan has a **deploy / runtime /
operations** section — process IDs, restart commands, process-manager names, port
numbers, row/record counts, "the service runs as X", "manual restart via Y" — treat
every one of those facts as a hypothesis captured at spec-authoring time, **not** as
current truth. Verify each against the live system in the same step you act on it.
Infra drifts independently of the spec text, so a spec that's correct about the
*code change* can be confidently wrong about *how to deploy it*.

This is the deploy-section specialization of
`feedback_runbook_audit_drift_post_event_check.md` (runbook/plan/audit claims drift
from reality). It earns its own file because the trigger is sharply recognizable —
you're reading a `## Deploy` / `## Runtime` / `## Operations` / "how to restart"
block — and because deploy facts have a distinct failure signature: they don't fail
the *code*, they fail the *rollout step*, late, after the risky part is already done.

## Why this matters (read-pheromone session, 2026-06-07)

`.claude/PRPs/specs/mcp-pmd-read-pheromone.md` was a finished, well-written patch
contract. Its code-change sections were accurate. Its deploy/runtime facts were stale
in two load-bearing ways, both authored confidently:

1. **"PID 5900; restart the server manually."** The live laptop PMD was actually an
   **NSSM Windows service** (`pmd-http-mcp`, Running/Automatic) — restart is
   `Restart-Service pmd-http-mcp` (which needs elevation), not a manual `kill`/restart
   of a loose PID. Acting on the spec's claim would have meant hunting a PID that no
   longer mapped to the process, or killing the wrong thing. The correction surfaced
   only because the plan's ground-truth pass ran `Get-Service` / `Get-NetTCPConnection`
   against the live box before writing the restart step.

2. **"579 rows" (later "767" in one place).** The live DB was 767 rows / 611 vectors at
   plan time. A row-count assertion in a spec is a timestamp in disguise — the DB grows
   every session. The temp-DB validation suite that asserted "row count still N after
   migration" had to bind N to the *live* count read at run time, not the spec's number,
   or the no-op-migration proof would have falsely failed.

Neither error blocked the *implementation* — but each was a landmine in the *deploy*
leg, exactly where a wrong move (restart the wrong PID, assert a stale count, expect a
process manager that isn't there) is hardest to undo and most likely to be done under
"just ship it" pressure at the end of the task.

A sibling drift in the same session, same class: the EliteDesk PMD's deploy reality
("it's a recent copy of the laptop lineage") was a hypothesis too — diffing its baseline
revealed it was a *much older* lineage with no hybrid search, turning a "copy the file"
deploy into a "version uplift" deploy. That one is captured in the cross-host-copy
lesson (`feedback_diff_baseline_before_cross_host_copy.md`); it's the same root —
**a deploy fact stated in/around a spec is a hypothesis** — applied to a second host.

## When to apply

The gate fires when you're about to **act on a deploy/runtime fact lifted from a spec,
patch contract, plan, or handover** — i.e. the fact tells you *how to deploy or operate*
the thing, not *what the code does*.

Triggering signatures (in a spec's deploy/runtime/operations section, or a plan's
"restart"/"rollout" step):

- A **process identity** claim: a PID, "runs as a loose `node` process", "started
  manually from a terminal", "the daemon at /opt/...".
- A **process-manager** claim: "restart manually", "kill and rerun", "systemd unit X",
  "NSSM service Y", "pm2", "Task Scheduler job Z" — *which* manager, and *how* to bounce
  it.
- A **count / size** claim used as a precondition or assertion: "N rows", "M vectors",
  "the DB is ~K MB", "P open connections".
- A **port / address / path** claim: "listens on 11435", "DB at /srv/.../x.db",
  "Tailscale IP 100.x".
- A **topology** claim: "this host is a recent copy of that one", "both run the same
  version", "the service is Automatic/Running".

Does NOT fire when:

- The spec section is about the **code change itself** (schema, functions, math,
  invariants) — that's reviewed against the source, a different discipline.
- The deploy fact was **captured this session** by a live check you ran (no drift
  window).
- You're reading the deploy section purely for *context* and won't *act* on any specific
  process/port/count value.

## How to apply

In the same step you'd act on the deploy fact, run the cheap live check first. Each is
seconds:

| Spec claim | Live verify before acting |
|---|---|
| "PID N" / "runs as loose process" | `Get-Process` / `Get-CimInstance Win32_Service` / `ps aux \| grep` — find the *actual* process + how it's supervised. |
| "restart manually" / "systemd unit X" / "NSSM Y" | `Get-Service <name>` (Windows) / `systemctl status <unit>` (Linux) — confirm the manager + name BEFORE choosing the restart command. Manual-vs-managed inverts the whole restart step (and may need elevation). |
| "N rows" / "M vectors" (as assertion or precondition) | Read the live count NOW; bind your assertion to that number, never to the spec's literal. |
| "listens on PORT" / "DB at PATH" | `Get-NetTCPConnection -LocalPort PORT` / `Test-Path PATH` / remote `ls` — confirm the endpoint exists where the spec says. |
| "host A is a copy of host B / same version" | Diff the baselines (`feedback_diff_baseline_before_cross_host_copy.md`) — never assume two hosts are in sync from a spec sentence. |

If the live check contradicts the spec: **correct the deploy step in place** (and, if the
spec is durable, fix the spec's deploy section in the same session so the next reader
starts from reality — same discipline as `feedback_runbook_audit_drift_post_event_check.md`
"update the runbook immediately").

## Hard refusals

- **NEVER run a restart/kill/rollout command built from a spec's process-identity or
  process-manager claim without a live `Get-Service`/`systemctl`/`ps` check first.** A
  stale PID or wrong manager name turns "restart the service" into "kill an unrelated
  process" or "command not found at the worst moment".

- **NEVER assert or precondition on a count/size literal copied from a spec.** Counts are
  timestamps. Read the live value and bind to it; a "row count unchanged after migration"
  proof that hardcodes the spec's number will false-pass or false-fail.

- **NEVER assume two hosts named in a spec are in sync.** "Host A is a copy of host B" is
  the highest-stakes deploy hypothesis — a partial file-copy onto a drifted host regresses
  it. Diff baselines first (cross-ref below).

## Cross-references

- `feedback_runbook_audit_drift_post_event_check.md` — the parent discipline (runbook /
  plan / audit claims drift from reality; LOCAL and EXTERNAL state axes). This lesson is
  the deploy/runtime-section specialization with its own recognizable trigger.
- `feedback_diff_baseline_before_cross_host_copy.md` — the sibling caught in the SAME
  session: before deploying to a non-git-checkout host, diff its baseline against the
  source's pre-change baseline. The "host A copies host B" deploy hypothesis.
- `feedback_handover_assumptions_need_empirical_verification.md` — handover-named system
  properties (env vars, CLI tools, remote dir shape, config locality) are hypotheses;
  verify each ≤5 min before patching. Same root applied to handovers rather than specs.
- `feedback_verify_spec_asserted_paths_exist_before_brief.md` — the path-existence sibling:
  a spec/PRD/bootstrap's asserted *file paths* ("X already created this file") are
  hypotheses too; existence-check before locking into a brief. This lesson is the
  runtime/deploy-fact sibling of that path-fact discipline.
- `feedback_falsifiable_hypothesis_before_structural_fix.md` — the broader family: a
  named-defect/named-mechanism claim (DQ, handover, spec) is a hypothesis to falsify
  before acting.
- `.claude/PRPs/reports/session-retro-2026-06-07-pheromone-pmd-impl.md` — the session that
  produced both incidents (the PID/restart drift and the count drift), and the cross-host
  uplift surprise.
