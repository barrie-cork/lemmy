---
name: Bundle = one worker branch, not one commit
description: When a brief bundles N plan-§13 tasks into one Junior task, "bundled" means ONE worker branch + ONE workspace-check workflow. Each plan task still gets its own feature commit. Brief instructions saying "one commit" violate the impl-task subagent's hard contract.
type: feedback
---

# "Bundled" means one worker branch, NOT one commit

When the advisor authors a brief that consolidates multiple plan §13
tasks into a single Junior dispatch, "bundled" refers to:

- **ONE worker branch** (the daemon cuts a single `junior/...` branch).
- **ONE workspace-check workflow** (the bundled push triggers
  `cargo-validate-workspace.yml` once for the combined tip).
- **ONE ci-watcher cycle** to mutate the resulting validate-pending
  DQ entry.

It does NOT mean one commit. Per `.claude/agents/impl-task.md` line
231 ("One feature commit per plan task. Subject: `feat(<scope>):
<title> (task <N>)`") AND `.claude/PRPs/templates/plan.template.md`
§13 ("**One commit per task** per `feedback_pr_per_phase.md`"), each
plan task still gets its own `feat(<phase>): <title> (task <N>)`
commit on the shared worker branch.

**Why this lesson exists:** Per `.claude/PRPs/reports/v1-RT-r1-retro.md`
§2 Impl miss + §5 Watch-item 2. v1-RT-r1's Cohort B-final mega-bundle
brief (`.claude/PRPs/briefs/rt-r1-impl-8-9-10-bundle.md`) said
"BUNDLED (Tasks 8 + 9 + 10 in one Junior task, **one commit**)" in
its §2 heading AND §6 specified a single commit message
`feat(v1-RT-r1): cohort B-final bundle — config.rs 26 consts +
governance-log 7 ENTRY_KIND + registry + e2e PHASE_1_MIGRATION_COUNT
(tasks 8+9+10)`. The Junior subagent produced THREE per-task commits
instead:

- `198ab6d06 feat(v1-RT-r1): config.rs — 26 new reputation-tuning consts ... (task 8)`
- `f2f9202b7 feat(v1-RT-r1): add 7 ENTRY_KIND consts ... (task 9)`
- `bc2531346 test(v1-RT-r1): extend phase1_migrations_round_trip ... (task 10)`

This was **not a Junior compliance miss** — Junior followed the
subagent's hard contract (one commit per plan task) AND the plan
template's commit rule, both of which outrank the brief. The
workspace-check workflow validated the combined tip just fine; the
PR description captured each commit; CR review attributed comments
correctly per task. **The brief was the outlier.**

Tasks 6+7 earlier in RT-r1 produced the same shape: brief said
"bundled", Junior produced two per-task commits
(`03f6c670e feat(v1-RT-r1): extend schema.rs ... (task 6)` +
`05cf5ae1d feat(v1-RT-r1): extend Diesel models ... (task 7)`).
That bundle worked too — same logical bundling, same per-task commit
shape, no advisor friction. Two-of-two evidence that
multi-commit-per-bundle is the canonical pattern.

**How to apply:** when authoring a bundle brief, the §2 Scope heading
+ §6 commit-message instruction should reflect the **canonical
pattern**:

```markdown
## 2. Scope — bundled (Tasks N+M+... in one Junior task)

**Why bundled:** ONE worker branch, ONE workspace-check workflow,
ONE ci-watcher cycle. Each task in this bundle still ships as its
own `feat(<phase>): <title> (task <N>)` commit per the impl-task
subagent contract and plan §13 commit rule.

(<task descriptions per-task as usual>)

## 6. COMMIT MESSAGES (one per plan task, in dispatch order)

1. `feat(<phase>): <title for task N> (task N)` — covers <file list
   for task N>.
2. `feat(<phase>): <title for task M> (task M)` — covers <file list
   for task M>.
N. (... one per bundled plan task)

The worker branch tip after all N commits is what the workspace-check
workflow validates.
```

**Brief title shape:** prefer `<phase>-impl-<N>+<M>-bundle.md`
(canonical for cohort siblings consolidated into one dispatch). The
"bundle" in the filename denotes one-dispatch-multiple-tasks; the
per-task commits are visible in the brief body.

**When NOT to bundle:**

- If the plan §13 tasks have a `requires:` cross-dependency (per
  `feedback_cohort_validation_dependency_check.md`), bundling is
  the explicit answer to a planner-side gap and IS appropriate.
- If the §13 tasks are truly independent (disjoint FILES YAML
  arrays, no shared workflow), parallel `[P]` cohort dispatch is
  the right answer per `.claude/rules/advisor-orchestrator.md` §4.
  Bundle reduces dispatch overhead but loses parallelism.
- If the §13 tasks would individually exceed worker memory/time
  budget, bundle compounds the problem. Per
  `feedback_resource_budget_pre_queue.md`.

**Edge cases:**

- **Bundle order matters for compile-success.** Within the worker
  branch, commits land sequentially; an intermediate commit's tip
  must compile if subsequent commits depend on it. Brief should list
  commits in dependency order. (Workspace-check only sees the FINAL
  tip; intermediate-commit broken-state is invisible to CI but is a
  CR-reviewer pain point — they can't `git bisect` cleanly.) Mitigate
  by either: (a) ordering bundled commits so each is compile-clean
  in isolation, OR (b) accepting CR-reviewer note + the workspace-
  check pass as sufficient evidence.
- **Bundle commit-count mismatch with plan §13 task count.** If the
  brief bundles 3 plan tasks but Junior produces 2 commits (e.g.
  because Tasks 8+9 ended up edited atomically), file a DQ pending
  entry — the planner-task-to-commit mapping needs explicit
  recording so retro complexity-score table reflects reality.
- **Single-task "bundle":** if only one §13 task is in the bundle,
  the brief is just a normal `<phase>-impl-<N>.md` — no bundling
  semantics needed. Don't author `*-bundle.md` filenames for
  single-task dispatches.

**Companion lessons:**

- `feedback_pr_per_phase.md` — the "one commit per task" rule the
  impl-task subagent honors.
- `feedback_cohort_validation_dependency_check.md` — L1 from RT-r1
  halt retro; when bundling resolves a cross-task validation
  dependency.
- `feedback_explicit_file_arrays_on_tasks.md` — §13 FILES YAML
  arrays per task, even when bundled.
- `feedback_handover_trailer_cohort_propagation.md` — per-task
  `HANDOVER:` trailers stay per-commit; bundle doesn't aggregate
  them into one trailer.

**Where codified:**

- `.claude/PRPs/templates/impl-task-brief.template.md` §2 Scope
  note + new §Bundling appendix.
- This lesson file.
