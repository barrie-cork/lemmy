# Task hopper

Parallel-agent phases (starting with Phase 6 — federation) need a shared
per-task execution ledger. The hopper is that ledger: one JSON file at
`.claude/task-hopper.json`, one helper at `scripts/brehon/task-hopper.sh`,
four verbs. Every agent working a Phase 6 task brackets its work with
four hopper calls.

The hopper is **orthogonal** to both the decision queue (questions) and
the ralph state file (whole-plan iteration counter). Don't conflate them.

## When to use

Use on every Phase 6 (and later) task executed by a parallel local
agent. The layered-execution model has up to 2 concurrent agents per
layer; without the hopper, failures go unobserved until the next merge
point and retros reconstruct "what happened when" from scrollback.

Skip for:
- Single-ralph phases (the existing ralph state file is sufficient)
- One-off ad-hoc edits outside a phase (no agent, no layer)
- Research / exploration sessions (no commit-per-task shape)

The hopper is cost-free when absent; no other tooling depends on it.

## The four verbs

Every task runs the same four-call envelope (three in the happy path;
four if a retry happens):

### 1. `start` — before any file edit

```bash
scripts/brehon/task-hopper.sh start <task-id> \
  --agent <agent-name> --kind <kind> --layer <n> \
  --label "<human-readable>" --worktree "<path>"
```

Required flags on **first** start of a task id: all of `--agent`,
`--kind`, `--layer`, `--label`, `--worktree`. On subsequent starts
(after a retry), the task entry already exists; flags may be omitted
(they're ignored).

`<kind>` must be one of: `cargo_check`, `cargo_clippy`, `cargo_test`,
`migration`, `agent_brief`, `other`. Retry caps differ per kind — see
§ Retry caps below.

Exit 0 on success. Exit 2 if the task is already `in_progress` (an
attempt is in flight — you probably want `retry` first), `completed`,
or `escalated`.

### 2. `retry` — when an attempt fails and you want another try

```
scripts/brehon/task-hopper.sh retry <task-id> \
  --reason "<one-line summary of why it failed>" \
  [--log <path-to-captured-log>]
```

Finalises the current attempt as `failed`. The task stays `in_progress`;
call `start` again to register attempt N+1.

**Auto-escalation:** if the attempt count has reached the retry cap for
this task's kind, `retry` transitions the task to `escalated`, files a
GitHub issue (unless `TASK_HOPPER_NO_GH=1`), and exits 4. The caller
(your agent loop) should surface this instead of retrying again.

### 3. `complete` — after the task's commit lands

```
scripts/brehon/task-hopper.sh complete <task-id> \
  --commit-sha "$(git rev-parse --short HEAD)" \
  [--log <path>]
```

Exactly once per task. Finalises the current attempt as `completed`,
records the commit SHA, transitions the task to `completed`.

Exit 0 on success. Exit 2 if the task isn't `in_progress`.

### 4. `escalate` — voluntary early give-up

```
scripts/brehon/task-hopper.sh escalate <task-id> \
  --reason "<why the agent is giving up before the retry cap>" \
  [--log <path>]
```

Use only when the agent decides the task cannot be completed by further
retries (e.g. a blocking DQ entry, an upstream API drift, a plan
contradiction). Files a GitHub issue and exits 4.

Agents do NOT call `escalate` when the retry cap is hit — the helper
does that automatically on the capped `retry`.

## Retry caps

Configured in `.claude/task-hopper.json` under `retry_caps`. Defaults:

| Kind          | Cap | Rationale |
|---------------|-----|-----------|
| `cargo_check` | 3 | Most failures are import/path errors; 2–3 attempts fix them |
| `cargo_clippy`| 3 | Same shape as cargo_check |
| `cargo_test`  | 2 | Test failures are signal; more than 2 retries wastes cycles |
| `migration`   | 2 | Low-iteration, high-consequence; escalate fast |
| `agent_brief` | 3 | Catch-all for brief-level work that doesn't fit other kinds |
| `other`       | 3 | Generic escape hatch |
| `default`     | 3 | Applied when kind has no override |

Advisor tunes caps by editing the JSON file; no code change required.

## Interaction with the decision queue

| Concern | Tool |
|---|---|
| "What decision is blocked on advisor input?" | `.claude/decision-queue.json` |
| "What's happening with task N right now?" | `.claude/task-hopper.json` |

Both can be populated simultaneously. If a retry is rooted in a
decision-queue question, cite it in `--reason`:

```
scripts/brehon/task-hopper.sh retry task-75 \
  --reason "blocked on DQ-6.2 — waiting on signature-capture decision"
```

## Interaction with the ralph state file

Ralph tracks **whole-plan iterations** (single-agent single-loop).
Hopper tracks **per-task attempts** (one entry per task, N attempts
per entry). They do not overlap.

Phase 6 does not use ralph at all — it uses parallel local agents.
The ralph state file is absent during Phase 6; the hopper is the only
execution ledger.

## Concurrency discipline

1. **One task per agent at a time.** Agent B never writes agent C's
   entries. Cross-agent coordination happens in the decision queue,
   not the hopper.
2. **mkdir-based exclusive lock** (`.claude/task-hopper.json.lock/`)
   for the duration of each read-modify-write. Contention window is
   milliseconds. Lock timeout defaults to 30s;
   `TASK_HOPPER_LOCK_TIMEOUT` overrides.
3. **Stale-lock detection** — locks older than 60s
   (`TASK_HOPPER_STALE_LOCK_AGE`) are assumed orphaned and removed
   with a warning to stderr.
4. **Append-only `attempts[]`.** Each attempt is a new object
   appended; no existing attempt is mutated. Same append-only
   invariant as the governance hash chain.
5. **Agents never hand-edit the JSON file.** Always through the helper
   — the helper enforces the state machine.

## GitHub issue format (escalation)

Issues are filed via `gh issue create --repo barrie-cork/lemmy` with
title `Phase <N> <task-id> blocked after <attempts> attempts` and
labels `phase-<N>/agent-blocked`, `agent:<agent>`, `kind:<kind>`.

The issue body includes:
- Task id, label, layer, agent, kind, worktree
- All attempts with started/ended timestamps and reasons
- Last attempt's transcript path (links back to the agent's context)
- Plan reference: `.claude/PRPs/plans/phase-<N>-federation.plan.md`
- Hopper snapshot SHA (current git HEAD)

After `gh issue create` succeeds, the URL is written back to the
task's `issue_url` field so the phase completion report can list all
escalations with links.

## Failure modes

| Failure | Behaviour |
|---|---|
| Helper can't acquire lock within timeout | Exits 3; agent must investigate (probably stale lock with writes in-flight) |
| Agent crashes mid-attempt, re-invokes `start` | Helper refuses; agent must manually `escalate` with `--reason "crashed mid-attempt"`. (Hand-editing the JSON to clear the in-progress entry is reserved for the advisor — see "Hopper state file corrupted" below.) |
| `gh issue create` fails (API outage, rate limit) | Helper sets `status=escalated`, leaves `issue_url=null`, logs the `gh` error. Advisor reruns `gh issue create` manually and patches `issue_url` |
| Two agents race on the same task id | Helper enforces single-writer via mkdir-lock; second agent blocks until first releases. If both attempt `complete` / `retry`, the second sees an already-finalised attempt and exits 2 |
| Python interpreter missing | Helper exits 1 immediately with a clear message. Install Python 3.8+ |
| Hopper state file corrupted | Helper exits 5. **Agents: stop and hand off to the advisor — do not attempt repair.** The helper itself has exited, so escalation must go via runlog or out-of-band. **Advisor-only repair**: restore the file from git history, or hand-edit the JSON. |

## Retro integration

After a phase closes, the completion report uses `jq` (or `python`) to
extract the retry history:

```bash
python -c "
import json
d = json.load(open('.claude/task-hopper.json', encoding='utf-8'))
for t in d['tasks']:
    if len(t['attempts']) > 1 or t['status'] == 'escalated':
        print(f\"{t['id']}: {t['status']}, {len(t['attempts'])} attempts\")
        for a in t['attempts']:
            print(f\"  n={a['n']} result={a['result']} reason={a['reason']}\")
"
```

This is the retro's "which tasks were hard and why" section.

## See also

- `scripts/brehon/task-hopper.sh` — the helper
- `.claude/task-hopper.json` — state file
- `.claude/task-hopper.schema.json` — JSON-Schema-lite
- `.claude/rules/decision-queue.md` — companion protocol for questions
- `.claude/rules/phase-branch.md` — phase-branch discipline (hopper
  operates within an already-cut phase branch)
- `.claude/PRPs/plans/phase-6-federation.plan.md` — Phase 6 plan;
  agent briefs reference the hopper

This rule auto-loads in `-p` mode alongside the other rules. Agents
inherit the protocol without per-brief repetition.
