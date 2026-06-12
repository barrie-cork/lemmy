Prepare a phase-transition-focused `/compact` for an in-flight Brehon implementation session. Reads live state (branch, plan, DQ, current cohort, ci-watcher runs) and emits a tailored `/compact <focus-text>` line for the user to run, while also loading the focus instructions into context as a fallback.

Use this command BEFORE `/compact` during long advisor or impl sessions when context is filling but the phase isn't done. The default `/compact` summarises uniformly across the whole conversation; this one tells the compactor to preserve the load-bearing context for the remaining tasks and drop the texture that's already shipped (retros, completed CR triage, finalized cohorts).

## When to use

- Active phase branch (`phase-v1-*`) — implementation in flight; `/compact` would otherwise blur the cohort handover and DQ state that the next iterations depend on.
- Active advisor session driving a sub-phase via Junior — long polling loop, lots of `list_tasks` / `show_task` history; need to preserve plan §13 task pointer, DQ pending, cohort barrier state.
- About to `/compact` and the next step is a concrete action (queue next task, raise validate-pending, dispatch ci-watcher) — not session close.

Do NOT use this for:
- Sub-phase close (`/brehon-phase-transition` is the right command).
- Pre-session-end checkpoint (`/advisor-checkpoint` already writes a self-contained handover; `/compact` is unnecessary).
- Non-Brehon sessions — focus prompt cites Brehon-specific artifacts (plan §13, DQ kinds, cohort handover trailer).

## Procedure

Run these reads in a single message (parallel where independent), assemble the focus prompt, print it as a fenced block, then load the same instructions into context as a fallback.

### Step 1 — Read live state

Run in parallel:

```bash
git branch --show-current
git worktree list
git log -1 --format='%H %s' HEAD
```

```bash
# Find active plan for current phase
ls .claude/PRPs/plans/*.plan.md 2>/dev/null
```

```bash
# Latest cohort handover trailer (look back 20 commits for HANDOVER:)
git log -20 --format='%H%n%B%n---COMMIT---' | head -200
```

Then read:
- `.claude/decision-queue.json` — extract `pending[]` entries: id, kind, from, question (one-liner each)
- The plan file matching the current phase (e.g. `phase-v1-fed-in-e` → `.claude/PRPs/plans/v1-fed-in-e.plan.md`) — scan §13 for unfinished task numbers (lines without `[x]` or `(task N) shipped`).
- `.claude/runlog/<phase>-runlog.md` if it exists — last 10 lines for current stage.

If any read fails (no plan, no runlog, not on a phase branch), degrade gracefully: emit what you have + a note that the focus prompt is partial.

### Step 2 — Identify cohort + validation state

From git log:
- Latest commit with `HANDOVER:` trailer → cohort just shipped; next cohort needs its handover.
- Any `kind: "validate-pending"` in DQ → ci-watcher in flight; preserve `workflow_run_id` + `branch`.
- Any `kind: "validate-pending-laptop"` in DQ → advisor-laptop §15 commands pending; preserve the command list.

### Step 3 — Compose the focus prompt

Template:

```
/compact focus on: <phase-id> plan §13 unfinished tasks (<task-numbers>),
current cohort handover (commit <sha-or-"none-yet">),
open DQ pending (<dq-ids-comma-list>),
<active-validation-line-if-any>.
Drop: completed retros, finalized task commits before <last-shipped-task-sha>,
CR triage for merged PRs, lessons already promoted to .claude/lessons/.
```

Fill the `<...>` slots from Step 1/2. If a slot has no value (e.g. no DQ pending), omit that clause rather than emitting "none".

Example for a session in v1-fed-in-e mid-cohort A:

```
/compact focus on: v1-fed-in-e plan §13 unfinished tasks (3, 4, 5, 6, 7),
current cohort handover (commit 897d3f72d),
open DQ pending (#341, #342, #343),
validate-pending workflow_run_id 12345678 on phase-v1-fed-in-e.
Drop: completed retros, finalized task 1+2 commits, CR triage for PR #146,
lessons already promoted to .claude/lessons/.
```

### Step 4 — Emit + load

**Print** the composed prompt inside a fenced ` ```text ` block, prefaced with one line: "Recommended `/compact` invocation — paste this:".

**Then** output the focus instructions as prose in the chat (NOT inside a block), framed as: "If you run bare `/compact` instead, apply this focus: <prose version of the same instructions>". This is the fallback so the loaded context steers the compactor even if the user types `/compact` without args.

## Output shape

After running, your reply to the user should be exactly two sections:

1. **One-line state summary** — `Phase: <phase> | Branch: <branch> | Cohort: <last-handover-sha-or-"pre-first">| DQ pending: <count> | Validation: <pending-kind-or-"none">`
2. **Recommended invocation** — the fenced `/compact …` block from Step 3.
3. **Fallback focus** — the prose paragraph from Step 4.

No other prose. No "let me know if…" trailer. The user will paste the line and run it.

## Failure modes

- **Not on a phase branch** (`governance-v0`, `main`, `chore/*`) → refuse with one line: "Not on a phase-v1-* branch — `/compact-transition` doesn't apply. Use bare `/compact` or `/advisor-checkpoint` instead."
- **No plan file for current phase** → emit the focus prompt with `plan §13` replaced by `recent commits` and cite the last 5 commit subjects.
- **DQ unreadable / malformed** → emit prompt without the DQ clause; note "(DQ read failed)" in the state summary line.
- **All §13 tasks complete** → refuse: "Phase looks done. Use `/brehon-verify` then `/brehon-phase-transition`, not `/compact-transition`."

## Why this command exists

`/compact` accepts free-text steering (per Claude Code's `/compact [instructions]` syntax) but in long Brehon advisor sessions, hand-typing the right focus prompt costs context AND drifts — last time you typed it, you forgot the cohort SHA or the validate-pending run id. This command reads the same artifacts your polling loop reads and assembles the prompt deterministically. The print-and-load shape (per user choice 2026-05-23) gives belt-and-braces: the printed line is the primary path; loaded instructions are a fallback if you type bare `/compact`.

The defect class this prevents: a mid-phase `/compact` summarises the conversation evenly, blurs the cohort handover into "earlier work was done", and the next iteration's plan-§13 pointer reads as ambiguous. The next task gets queued against the wrong cohort or misses a pending DQ. Surface time wasted: 5-15 minutes of `git log` + DQ re-reading to recover state the compactor dropped.

## See also

- `/advisor-checkpoint` — full self-contained handover; use at session boundaries.
- `/brehon-phase-transition` — sub-phase close; not for mid-phase compaction.
- `.claude/rules/advisor-orchestrator.md` §1 "Pre-compact handover discipline" — the broader rule this command supports.
