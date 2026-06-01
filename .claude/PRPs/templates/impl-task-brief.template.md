---
role: impl-task
plan_task: <N>
phase: <phase-slug>      # e.g. v1-JM-d
created: <YYYY-MM-DD>
related_dq: <id-or-null>
---

# Brief — <phase> Task <N> — <title>

> **Clarify provenance:** this brief's parent **planning** brief was clarified via `/brehon-clarify` before the planning task was queued (per `.claude/rules/advisor-orchestrator.md` "Clarify gate"). Any DQ #<id> with `from: "advisor"` and `kind: "clarify"` referenced in §3 below is a clarification that gated planning — read those entries to understand decisions baked into the plan. If this brief is for an impl-task and no clarify-DQ is cited, that's expected: clarify gates planning briefs only.

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

### 2.0 Scope gate — e2e.rs fix-impl size cap (mandatory)

Per `feedback_fix_impl_pre_locate_e2e_anchors.md` §"How to apply" + the
2× recurrence at v1-RT-r3 Task 4 cycle (Junior #474–#477, DQ
`a3d0e9941441-033`) + fix-impl-2 dispatch (Junior #479, 2026-05-26).

**When this brief is a fix-impl** (filename matches `*-fix-impl-*.md`)
**AND** the `modifies:` array (or §2 "Produce" list) contains
`crates/server/tests/e2e.rs`, the brief MUST satisfy ALL THREE caps:

- **Brief length ≤ 150 lines** (excluding frontmatter, excluding any
  pasted `old_string`/`new_string` blocks — those are bounded by the
  edit-count cap below, not the line cap).
- **File edits ≤ 2** (the `modifies:` array, plus `creates:` if any,
  must total at most 2 distinct files).
- **Edit calls per file ≤ 2** (each file's worker-side Edits must
  number ≤ 2; if the recipe requires 3+ Edits per file, the brief is
  over the cap).

**Briefs exceeding any cap MUST split** into narrower
`<phase>-fix-impl-<N>a.md` + `<phase>-fix-impl-<N>b.md` briefs. Each
split brief independently satisfies all three caps. The advisor
dispatches them as separate Junior tasks (serial or `[P]`-cohort per
the cohort dispatch rule).

**Pre-located verbatim anchors are required** (not just guidance) for
every Edit in a fix-impl brief targeting e2e.rs. The brief MUST paste
the exact `old_string` (5-10 lines of distinctive surrounding context)
and `new_string` (the replacement) into the recipe section. Per
`feedback_fix_impl_pre_locate_e2e_anchors.md` §"How to apply".

**Why the cap exists:** a fix-impl brief targeting e2e.rs that violates
any of the three caps statistically reproduces the v1-RT-r3 Junior
#474–#477 + #479 `error_max_turns` failure class. Sonnet 4.6 impl-task
on a 17,000-line file cannot afford the Read+Grep budget to locate
anchors AND apply 3+ Edits within the 150-turn budget. The cap
mechanically forces the brief author to either (a) shrink the scope
per-dispatch or (b) pre-locate so the worker spends turns on Edits, not
on Reads.

**This gate applies to fix-impl briefs only.** Non-fix-impl briefs
(plan §13 task briefs adding NEW e2e fixtures modules) may legitimately
exceed the cap when authoring net-new modules of 200-500 lines — those
go through the canonical impl-task-brief shape with §2.4 lesson
injection per `feedback_junior_worker_e2e_edit_hang.md` (parent class).

Delete this §2.0 block from a non-fix-impl brief.

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

## 3a. Handover from prior cohort

(Populated by the advisor on cohort-transition. If this task is the first cohort of the phase, or the prior cohort was a single non-`[P]` task whose `HANDOVER:` trailer was a no-op, this section reads `(none — first cohort)` or `(none — prior task non-[P])` and the impl-task subagent skips it.)

Per `feedback_handover_trailer_cohort_propagation.md`. The advisor parses each cohort task's commit body for the `HANDOVER:` YAML trailer (per `.claude/agents/impl-task.md` "Per-task commit shape"), aggregates across the cohort, and prepends the result here. The next cohort's impl-task subagents read this **before** their first edit so consistent decisions made by Cohort N (e.g. "used `parking_lot::RwLock` not `std::sync::RwLock`") propagate to Cohort N+1 without grep-discovery.

Format (filled by advisor):

```yaml
prior_cohort_tasks:
  - task: <N>
    commit: <sha>
    filesCreated: [...]
    filesModified: [...]
    keyDecisions: [...]
    notes: <free-text>
  - task: <N+1>
    commit: <sha>
    filesCreated: [...]
    filesModified: [...]
    keyDecisions: [...]
    notes: <free-text>
```

The impl-task subagent treats `keyDecisions` from the prior cohort as **load-bearing context** — diverging from a prior keyDecision without a stated reason is a planner-side gap (file a DQ pending entry). Diverging *with* a stated reason (e.g. "Cohort N chose A; this task chose B because <plan §10.5 mirror demands B>") is fine and should appear in this task's own `HANDOVER:` trailer.

Single-task non-`[P]` impl-tasks (no cohort siblings, no parallelism) skip the trailer entirely on output (per impl-task.md). They still read this section on input — a non-`[P]` task that follows a `[P]` cohort still benefits from the prior cohort's keyDecisions.

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

**Linux-compile gate (diff-scoped — include ONLY if this task's files match the trigger):** if this task touches `Cargo.toml` / `Cargo.lock` / `migrations/**`, OR adds `cfg(unix)` / `cfg(target_os)` / path-separator-shaped code, ALSO write a `kind: "validate-pending-laptop-linux"` DQ entry with `commands: ["./scripts/brehon/cargo-linux.sh check --workspace --features full"]`, commit + push, then **stop** — the laptop advisor runs the Linux compile and mutates the entry. Do NOT run `cargo-linux.sh` on the daemon (NO CARGO ON ELITEDESK). Pure governance-logic tasks (no dep/migration/cfg change) compile identically on Windows and Linux — **omit this gate** for those; it adds ceremony, not coverage. This gate blocks `bm-pr` Phase-1d. Per `feedback_linux_compile_proof_is_a_gate.md`.

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

**Verification-only tasks (Task 0, audit-probe tasks, anchor-drift sentinels):** for each probe, paste the **actual tool output** (Bash stdout, Grep result, Read excerpt), not a ✓/✗ summary. A checkmark without output is fabrication-equivalent — the advisor cannot distinguish "right by work" from "right by luck". 1× incident: v1-rt-r3-followup Task 0 (#504) asserted Probes 8/9/10 PASS in thinking with zero measuring tool calls; advisor re-ran all three on the lane worktree (happened to be correct). Post-condition re-verify is the only reliable catch.

## 7. Why this brief differs from the plan (if applicable)

Document any overrides:

1. **Step <X> command replaced** — DQ #<id> showed <reason>; canonical path is <new command>.
2. **<Other override>** — <reason + DQ ref>.

Delete this section if the brief is a clean execution of the plan with no overrides.

## 8. Bundling (when consolidating multiple plan §13 tasks into one Junior dispatch)

Per `feedback_bundle_means_one_worker_branch_not_one_commit.md`. When a brief consolidates N plan §13 tasks into one Junior task (typical reason: shared workspace-check workflow, cross-task validation dependency, or post-halt-retro recovery), the **canonical pattern is**:

- **One worker branch** (the daemon cuts a single `junior/...` branch).
- **One workspace-check workflow** validates the combined tip.
- **One ci-watcher cycle** mutates the resulting `validate-pending` DQ entry.
- **N commits**, ONE per plan §13 task, each with the per-task subject pattern `feat(<phase>): <title> (task <N>)`.

A bundle brief MUST NOT instruct Junior to produce "one commit covering all N tasks". Junior's hard contract (`.claude/agents/impl-task.md` line 231: "One feature commit per plan task") and plan §13's commit rule both outrank brief overrides. Briefs asking for a single commit are a brief-authoring miss; Junior correctly defaults to N commits.

**Brief filename convention:** `<phase>-impl-<N>+<M>-bundle.md` (e.g. `rt-r1-impl-8-9-10-bundle.md`). Single-task dispatches use `<phase>-impl-<N>.md` without "bundle" in the name.

**§2 Scope shape for a bundle brief:**

```markdown
## 2. Scope — bundled (Tasks N+M+... in one Junior task)

**Why bundled:** ONE worker branch, ONE workspace-check workflow, ONE ci-watcher cycle. Each task in this bundle ships as its own `feat(<phase>): <title> (task <X>)` commit per the impl-task subagent contract.

### 2.1 Task <N> scope (<file or feature>)
<sub-edits + IMPLEMENT details per plan §X.Y>

### 2.2 Task <M> scope (<file or feature>)
<sub-edits + IMPLEMENT details per plan §X.Z>

(... per bundled task)
```

**§6 commit-message shape for a bundle brief:**

```markdown
## 6. COMMIT MESSAGES (one per plan task, in dependency order)

1. `feat(<phase>): <title for task N> (task N)`
2. `feat(<phase>): <title for task M> (task M)`
N. (... one per bundled plan task)

The worker branch tip after all N commits is what the workspace-check workflow validates.
```

**Compile-success between bundled commits:** order commits so each intermediate tip is compile-clean (helps CR `git bisect` review). If intermediate commits would not compile in isolation (e.g. Task A adds a struct field, Task B pads the callers), document the ordering rationale in §2 and accept the CR reviewer note.

**Out-of-bundle scope:** if a §13 task has a `requires:` dependency on a task NOT in this bundle, file a DQ pending entry — the planner-side cohort design needs revision before bundling.

Delete this section if the brief dispatches a single plan task (no bundling).
