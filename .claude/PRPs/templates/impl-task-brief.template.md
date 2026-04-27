---
role: impl-task
plan_task: <N>
phase: <phase-slug>      # e.g. v1-JM-d
created: <YYYY-MM-DD>
related_dq: <id-or-null>
---

# Brief — <phase> Task <N> — <title>

## 0. Pre-flight (subagent runs this before reading anything else)

The `impl-task` agent already runs the forbidden-window check from `.claude/agents/impl-task.md` "Task-0 pre-flight". This brief inherits that — do not duplicate the bash. If the dispatch line includes `forbidden-window-override: DQ #<id>`, the agent skips the check.

Forbidden windows (UTC) — full table at `.claude/rules/advisor-orchestrator.md` "Forbidden execution windows":
- Daily 02:55–04:15 (NAS backup chain + web-archive govie-search)
- Sunday 01:55–02:35 (HSE crawl + weekly review)
- Sunday 03:55–04:30 (restore drill)

Recommended Brehon execution windows:
- Primary: 16:00–02:30 UTC
- Secondary: 04:30–14:59 UTC

If the advisor queued this task during a forbidden window without an override DQ, the orchestrator-rule check was skipped — file a DQ catch-fire entry citing `feedback_advisor_orchestrator_forbidden_window_skipped`.

## 1. Role + dispatch line

`[role:impl-task] <phase> task <N> — see .claude/PRPs/briefs/<phase>-impl-<N>.md`

You are the **impl-task** subagent (Sonnet 4.6 per your frontmatter). Execute plan task <N> from `.claude/PRPs/plans/<plan-file>.plan.md` §<plan-section>.

## 2. Scope

**Produce** (one commit per the plan's "one commit per task" rule):

- `<file>` — <what changed and why, citing plan §X>.
- `<file>` — <what changed and why, citing plan §X>.

**Do NOT** in this task:

- <out-of-scope item> (Task <other-N>).
- <out-of-scope item>.

**Commit message** (exactly): `feat(<scope>): <title> (task <N>)`

## 3. Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries** gating this task — name them by id.
2. **Plan §<task-section>** — the canonical step list.
3. **Plan §<schema-section>** — exact field/enum shape (verbatim doc-comment text).
4. **MIRROR refs** — list each `<file>:<line-range>` the plan cites.
5. **Lessons** (Glob `.claude/lessons/`, Read any with filename keywords matching this task):
   - `feedback_<keyword>.md` — <why relevant>
   - `feedback_pipes_mask_exit_codes.md` (always — capture-then-tail rule)
   - `feedback_clippy_test_style.md` (always for cargo work — workspace denies escape-hatches)
   - Any `feedback_features_full_*` or `feedback_pq_sys_*` lessons relevant to the validation gate.

## 4. Constraints

### Branch + commit discipline

- You start on a Junior worktree off `<phase-branch>`. Finalize merges your worktree branch back; do not push to `<phase-branch>` directly.
- One commit. If clippy/check fails on the first attempt, amend or fixup; do not split the commit.
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch per `.claude/CLAUDE.md` cheatsheet.
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Memory-cap awareness

The daemon runs under `MemoryMax=10G`, `MemoryHigh=8G` (deployed at `homeserver` `20f251b`). If `cargo check --workspace --features full` hits the cap, the cgroup OOM-killer terminates the worker process — Junior reports a non-zero exit; you'll see `Killed` in the log. **Do NOT retry blindly** — file a DQ pending entry with the cargo log tail; advisor will decide whether to bump the cap or break the validation per-crate.

`cargo install <anything>` is an advisor-side responsibility. Do not attempt cargo installs without a DQ-asking entry first.

### Plan-cited line numbers may have drifted

The plan was written before this task. If it cites exact line numbers (e.g. `foo.rs:234`), verify by `grep -n` before editing. If line numbers shifted, follow the grep output, not the plan numbers. If the count of expected sites differs, file a DQ pending entry — the schema may have drifted since plan write and the brief needs adjustment.

### Lesson trailer (encouraged)

If during the task you discover something a future impl-task on a related area would have wanted to know — a non-obvious constraint, a footgun, a pattern that bit you — end the commit-message body with a `LESSON:` line per `feedback_junior_pmd_write_convention.md`. One discrete lesson per `LESSON:` line. Cite specific files/lines.

### <Task-specific overrides>

Document any plan-step substitutions, gotchas, or constraints unique to this task. Cite the DQ id that authorised the override.

## 5. Validation gates (per plan §<plan-section> task <N> VALIDATE block)

Capture each to `.claude/PRPs/debug/<phase>-task<N>-<probe>.log`. All exit-0.

1. `bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/<phase>-task<N>-check.log 2>&1` → exit 0.
2. `bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/<phase>-task<N>-clippy.log 2>&1` → exit 0.
3. `<additional gate per plan>` → exit 0.

Per `feedback_pipes_mask_exit_codes.md`, never pipe cargo through tail/head/grep when you need to know if it succeeded — capture full output, then check exit code, then tail the file separately.

If any gate fails, **STOP and surface to advisor via DQ.** Do not patch around `cargo-check` or `clippy` failures by `#[allow]`-spamming — fix the root cause.

## 6. Expected output (return to advisor)

```
## Task <N> complete — <phase> <title>

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - <file> (+<what>)
  - <file> (modified — <what>)
**Validation:** check / clippy / <other> all exit 0
**Next:** advisor queues task <N+1> (<one-line summary>)
```

Plus any DQ #N references if you raised one mid-task.

## 7. Why this brief differs from the plan (if applicable)

Document any overrides:

1. **Step <X> command replaced** — DQ #<id> showed <reason>; canonical path is <new command>.
2. **<Other override>** — <reason + DQ ref>.

Delete this section if the brief is a clean execution of the plan with no overrides.
