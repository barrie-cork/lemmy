---
role: design-brief
target: planning-task (next session)
phase: cross-phase architectural — alternative shape to validate-agent-design.md
created: 2026-04-27
related_pmd: 109, 110, 111
related_briefs: validate-agent-design.md, validate-agent-planning-1.md
status: design-only — no code in this brief
---

# Design brief — Out-of-Junior cargo validation via GitHub Actions + ci-watcher polling agent (Shape G)

## 1. Purpose

Decouple long-running cargo validation work from Junior worker occupancy by running it on **GitHub Actions runners** (clean caches, dedicated CPU+RAM, no contention with the EliteDesk's other workloads), and introduce a `ci-watcher` polling agent that consumes workflow results and feeds them back into the decision-queue.

This is an **alternative shape to `validate-agent-design.md` (Shape C/A)**. Where Shape C runs validation on a local cron-driven runner on the EliteDesk, Shape G outsources it entirely to GitHub. Both shapes share the goal of freeing the impl-task slot at commit time; they differ on where the cargo work executes.

## 2. Why this exists (problem statement)

Same root problem as `validate-agent-design.md` §2 — impl-task wall-clock occupancy on cargo runs the model doesn't need to attend. Shape G adds two specific advantages over Shape C and one new wrinkle:

**Advantages over Shape C:**

1. **Zero EliteDesk resource contention.** Shape C runs cargo locally; even out-of-Junior, it competes with `rust-analyzer` LSP processes, OpenSearch JVM (~2.2 GB), other daemons, and the cgroup memory ceiling. Today's pattern of cgroup at 8.6 GB / 10 GB cap during a single cargo workspace check is not sustainable as more sub-phases queue impl-tasks. Shape G removes the cargo workload from the box entirely.
2. **Ephemeral runners eliminate orphan-process risk.** PMD #110 surfaced that task #13 left an orphan `cargo check` process (PID 210539) that survived the worker SIGTERM and held 5.4 GB of cgroup memory until manual cleanup. GitHub runners are destroyed after each job — orphans are physically impossible.

**Wrinkle Shape G introduces:**

- **Workflow latency floor.** A GitHub-hosted runner cold start + cargo cache restore + cargo check workspace+features full is ~10-25 minutes of wall-clock (faster than EliteDesk on a clean cache, slower than EliteDesk on a hot cache). vs. local cron Shape C which can keep the cargo target cache hot indefinitely. **Net wall-clock per validation is comparable; throughput-of-impl-tasks is much better under Shape G** because the EliteDesk slot frees immediately.

**Brehon goal alignment:**

- Goal #1 (autonomous): Shape G is fully autonomous — `git push` → workflow auto-runs → ci-watcher polls → DQ entry → advisor polls. No user-in-loop steps.
- Goal #2 (reliable): runners are clean each time; no `pq-sys` stale-cache issues, no `target/` corruption, no swap thrash.
- Goal #3 (slow-OK): polling cadence ~5 min is acceptable per the "can be slow" goal.
- Goal #4 (model-efficient): impl-task subagent exits after `git push`, freeing the worker slot for the next task. Roughly the same model-token savings as Shape C.

## 3. Solution shape — "Shape G"

### Layer G1 — GitHub Actions workflows for the §15 DoD gates

Today, `.github/workflows/cargo-test-e2e.yml` covers the e2e test suite only (not the full DoD gates). Add two complementary workflows:

- **`cargo-validate-workspace.yml`** — runs `cargo check --workspace --features full --no-deps`, `cargo clippy --workspace --features full --no-deps -- -D warnings`, `cargo test --no-run -p lemmy_server`. Triggered on any push to a `phase-v1-*` or `junior/*` branch (NOT just PRs to `governance-v0` — we need it on impl-task branches before merge). Concurrency-gated by branch + cancel-in-progress.
- **`cargo-validate-migration.yml`** — runs the migration round-trip check (currently a §15.5 DoD step, only on plan tasks that touch `migrations/`). Trigger: push containing `migrations/**` changes. Same branch-pattern trigger as G1.

Existing `cargo-test-e2e.yml` stays as-is; G1 supplements it.

Both workflows use `actions/cache@v4` for the cargo registry + target dir, keyed by `Cargo.lock` hash. Cold cache run: ~25 min. Hot cache run: ~5-10 min.

### Layer G2 — impl-task brief contract change

The impl-task subagent rule (`.claude/agents/impl-task.md`) updates §5 (Validation) to read:

```
Validation runs out-of-band on GitHub Actions. After committing your work,
push to your worktree branch and exit. Do NOT run cargo locally.
```

§4 (Constraints) gains:

```
- After `git push`, capture the workflow_run id via `gh run list --branch <your-branch> --limit 1 --json databaseId --jq '.[0].databaseId'`.
- Append a `validate-pending` entry to `.claude/decision-queue.json` containing:
    - kind: "validate-pending"
    - from: "impl-task"
    - workflow_run_id: <id>
    - branch: <your-branch>
    - phase_task: <task number>
- Commit + push the DQ update.
- Exit with success.
```

The impl-task slot frees as soon as the push lands. Wall-clock from queue to slot-release: ~5-15 min.

### Layer G3 — ci-watcher subagent

A new fifth subagent role at `.claude/agents/ci-watcher.md`. Frontmatter pins Haiku 4.5 — this is mechanical polling, no judgment. Effort: low.

The advisor queues a `[role:ci-watcher]` task whenever a `validate-pending` DQ entry appears. The ci-watcher's contract:

1. Read the DQ entry's `workflow_run_id` + `branch`.
2. Poll `gh run view <id> --json status,conclusion,jobs` every ~30 seconds (in-task, not orchestrator-side) until `status: "completed"`. Hard cap: 60 minutes; if exceeded, write a `validate-failed` DQ entry with the timeout reason.
3. On completion, classify:
   - `conclusion: "success"` → write `validate-result` DQ entry with `result: "pass"`. Commit + push.
   - `conclusion: "failure"` → fetch the failing job logs via `gh run view <id> --log-failed`, slice last ~200 lines per failed job, write a `validate-failed` DQ entry with `result: "fail"` and the log slice attached. Commit + push.
4. Exit.

The ci-watcher does **not** apply fixes. That's where Shape A escalation comes in.

### Layer G4 — escalation path (mirror of validate-agent-design.md Layer A)

When a `validate-failed` DQ appears, the advisor reads the log slice and decides:

- **Trivial fix matching a known pattern** (clippy lint, missing import, deprecated API): queue a follow-up `[role:impl-task]` with a small fix-only brief; ci-watcher re-runs after the fix push.
- **Real surprise**: surface to user via the standard catch-fire procedure.

This layer is identical between Shape C and Shape G — only the upstream classifier differs (cron-runner pattern-match in Shape C, ci-watcher classification in Shape G).

### Layer G5 — Planning-side implication (mirror of validate-agent-design.md §3.5)

Same as Shape C. Plan §13 task composition shifts from "edit + validate-inline + commit" to "edit + commit; validation runs async". Plan §15 DoD becomes the workflow's job, not the impl-task's.

## 4. Files this design will touch (when implemented)

### 4.1 New artifacts

- `.github/workflows/cargo-validate-workspace.yml` — workspace check + clippy + test-no-run.
- `.github/workflows/cargo-validate-migration.yml` — migration round-trip check, conditional on `migrations/**` changes.
- `.claude/agents/ci-watcher.md` — fifth subagent role, Haiku 4.5, low effort.
- `.claude/PRPs/templates/ci-watcher-brief.template.md` — minimal brief template (workflow_run_id + branch + phase_task only).

### 4.2 Existing artifacts to extend

- `.claude/agents/impl-task.md` — §4 Constraints + §5 Validation per Layer G2 above.
- `.claude/rules/decision-queue.md` — add `kind: "validate-pending"`, `kind: "validate-result"`, `kind: "validate-failed"` to the schema enumeration.
- `.claude/rules/advisor-orchestrator.md` — Stage-shape orchestration map: after impl-task commit, queue ci-watcher; after validate-result pass, advance; after validate-failed, run the §G4 classifier.
- `.claude/PRPs/templates/plan.template.md` — §15 (Validation commands) re-cast as "DoD per workflow" with workflow file references; §13 task composition guidance per §G5.

### 4.3 NOT changing

- `.github/workflows/cargo-test-e2e.yml` — stays as-is (already PR-triggered, mature). G1 supplements it.
- Junior's executor or watchdog code. Shape G is purely a Brehon-side change.
- Any production code in `crates/`. The validation gates' content is unchanged; only their execution location moves.

## 5. Decision-queue schema additions

Three new `kind` values, all defined under `.claude/rules/decision-queue.md`:

- **`validate-pending`** (`from: "impl-task"`): impl-task pushed and is awaiting CI. Fields: `kind`, `from`, `workflow_run_id`, `branch`, `phase_task`. Always pending until ci-watcher resolves it.
- **`validate-result`** (`from: "ci-watcher"`): CI completed cleanly. Fields: `kind`, `from`, `answered_by`, `workflow_run_id`, `result: "pass"`. Resolved on write.
- **`validate-failed`** (`from: "ci-watcher"`): CI failed or timed out. Fields: `kind`, `from`, `workflow_run_id`, `result: "fail" | "timeout"`, `log_slice`, `failed_jobs`. Pending until advisor classifies (advisor-mode trivial-fix, or user-relay surprise).

Schema additions follow `feedback_update_rule_doc_with_schema_additive.md` — must land in the same commit as the consumers (ci-watcher.md + advisor-orchestrator.md update).

## 6. Comparison: Shape G vs Shape C

| Aspect | Shape C (local cron) | Shape G (GitHub Actions) |
|---|---|---|
| Runner location | EliteDesk | GitHub-hosted |
| Resource contention | Yes — competes with daemons, LSP, JVM | No — runner is ephemeral |
| Cache freshness | Hot indefinitely | Cold on first run, hot via actions/cache |
| Wall-clock per validation | ~5-15 min (hot cache) | ~10-25 min (cold) / ~5-10 min (hot) |
| Orphan-process risk | Yes (PMD #110 evidence) | No |
| Budget concerns | Already-paid hardware | GitHub Actions minutes (free tier: 2000/month for private repo) |
| Visibility | Local logs only | GitHub UI + status checks |
| Failure classifier | Cron-side pattern-match | ci-watcher subagent |
| User-facing PR feedback | None directly | Yes — workflow status visible on PRs |
| Implementation surface | new cron unit + DQ kinds + planning rework | new workflow + new subagent + DQ kinds + planning rework |
| Net new infrastructure | Timer + screen-session wrapper + DQ writer | Two workflow YAMLs + ci-watcher subagent |

**Cost note:** GitHub Actions free tier on private repos is 2000 minutes/month. A workspace cargo check run is ~10-25 min. ~80-200 validations/month before hitting the limit. JM-d alone has ~6 impl-tasks. Capacity headroom is comfortable for 4-5 sub-phases per month; tight if we go higher. Mitigation: enable runner caching aggressively, only run G1 on phase branches not personal branches.

**Recommendation in this brief:** prefer Shape G unless GitHub Actions minutes become the binding constraint, in which case fall back to Shape C. Both designs share §G4 (escalation) and §G5 (planning-side implications) — adopting one doesn't preclude the other later.

## 7. Wall-clock budget and forbidden-window interaction

Shape G is **insensitive to the EliteDesk's forbidden-window table** (`advisor-orchestrator.md` §Forbidden execution windows) — GitHub runners don't share resources with the EliteDesk's NAS backup or web-archive crawl jobs. The forbidden-window check stays in place for any *local* cargo work (e.g. ad-hoc validation by the advisor before user-relay) but doesn't gate impl-task queue cadence.

## 8. Failure modes to design against

1. **GitHub Actions outage.** ci-watcher times out (60-min cap), writes `validate-failed: timeout`. Advisor surfaces to user; manual fallback is local validation.
2. **Workflow flakes** (e.g. testcontainer flake on e2e). ci-watcher classifies as `validate-failed: fail`. Advisor reads the log; if the failure looks like a known flake pattern (timeouts, network), the advisor can re-trigger the workflow via `gh run rerun <id>` — adds a "re-run once" recipe to the §G4 classifier.
3. **GitHub rate-limit on `gh api`.** ci-watcher hits 5000 req/hour limit (very unlikely at 30-sec poll cadence: 120 calls/hour per task). Mitigation: use `gh run watch` (single long-poll) instead of repeated `gh run view` — same outcome, one API call per task.
4. **Push-to-trigger lag.** workflow_run can be queued for 1-2 min before starting. ci-watcher accommodates by re-checking the workflow's `status` field, not blocking on `started_at`.
5. **`gh` CLI auth on the daemon.** ci-watcher runs on a Junior worker; the EliteDesk daemon already has `gh` authenticated as `barrie-cork` (per `project_brehon_advisor_takes_over_plan.md` Phase 1). No new auth setup needed.
6. **DQ write race.** Multiple ci-watchers writing to `decision-queue.json` simultaneously. Mitigated by Junior's concurrency=1 — only one ci-watcher runs at a time. If concurrency ever increases, add a `--lock` step.
7. **Workflow misclassification.** ci-watcher might classify a real bug as a "known pattern" trivial fix. Mitigation: keep the §G4 classifier conservative — only auto-queue trivial-fix impl-tasks for a small allowlist (clippy lints with auto-fix, missing imports). Anything else surfaces.

## 9. What the implementing session needs to do

The next planning session should produce a plan covering:

1. **Workflow authorship.** Two new YAMLs at `.github/workflows/cargo-validate-workspace.yml` + `cargo-validate-migration.yml`. Adapt patterns from existing `cargo-test-e2e.yml` (cache strategy, libpq install, Rust toolchain). DoD: each workflow runs green on a sample push.
2. **ci-watcher subagent.** New file `.claude/agents/ci-watcher.md` — frontmatter (Haiku 4.5, low effort), §1 contract, §2 hard refusals (no fixes, no impl edits, no clippy applies), §3 expected output shape.
3. **Brief template.** `.claude/PRPs/templates/ci-watcher-brief.template.md` — three-line minimum: workflow_run_id, branch, phase_task. The ci-watcher's brief is mostly auto-generated by the advisor from the `validate-pending` DQ entry.
4. **impl-task §5 rewrite.** Update `.claude/agents/impl-task.md` per Layer G2 contract change. Cite this brief by filename in the rationale.
5. **decision-queue.md schema additions.** Add the three new `kind` values. Cite ci-watcher as writer; cite advisor + impl-task as readers.
6. **advisor-orchestrator.md stage-shape update.** Add the `validate-pending → ci-watcher → validate-result|validate-failed` transitions to the existing stage-shape map. The cohort-dispatch rule (§Cohort dispatch) stays — multiple impl-tasks can be in `validate-pending` simultaneously since each has its own workflow run.
7. **plan.template.md §15 rewrite.** §15 becomes "DoD per workflow" — references the workflow file rather than enumerating cargo commands. §13 task composition guidance per §G5.

Plan should produce ~5-7 §13 tasks, each small (single-file or two-file edits). Total estimated ship time: ~2-3 sub-phases worth of advisor effort, but recoverable across all subsequent sub-phases.

## 10. References

- `validate-agent-design.md` — sibling design, Shape C (local cron) — share §G4 + §G5 with this brief.
- `validate-agent-planning-1.md` — sibling planning brief for Shape C; the new planning brief should adopt either Shape C's or Shape G's task list, not both.
- PMD #109 — impl-task running Opus despite frontmatter (informs subagent model-pinning discipline for ci-watcher).
- PMD #110 — orphan cargo PID + telemetry blind-spot — direct evidence Shape G eliminates.
- PMD #111 — Server Boss alert tuning — sibling effort (alert noise reduction); both reduce the operational-load pressure on the EliteDesk.
- `.github/workflows/cargo-test-e2e.yml` — pattern source for workflow YAML structure.
- `feedback_brehon_autonomy_goals.md` — goal #4 model-efficient is the primary justification for both Shapes.
- `feedback_brehon_subagent_model_effort_assignments.md` — supports ci-watcher's Haiku 4.5 pin.
- `feedback_resource_budget_pre_queue.md` — Shape G makes this rule mostly obsolete for cargo-heavy validation queues (the EliteDesk no longer carries that load).

## 11. Out of scope (explicit)

- **Generalising beyond Brehon.** Shape G targets the brehon-fork repo only. Other server repos (agent-grey, dog-shelter, food-producer) use Junior with their own local CI patterns; this design does not touch them.
- **Modifying upstream Lemmy CI.** The upstream Woodpecker pipeline at `.woodpecker.yml` stays untouched.
- **Self-hosted GitHub runners.** Tempting to run GH Actions on the EliteDesk via `actions-runner` to bypass minutes-budget — but that re-introduces the resource-contention problem this design solves. Only revisit if private-repo minute limits become binding for >2 consecutive months.
- **Replacing `cargo-test-e2e.yml`.** It works; G1 supplements rather than replaces it. e2e is structurally different (testcontainers per test) and benefits from being its own workflow.
- **Migrating PR-review work.** The existing `governance-ai-review.yml` + `adr-compliance.yml` are unrelated.

## 12. Acceptance criteria

When Shape G is fully implemented:

- [ ] An impl-task brief's §5 Validation reads "push and exit; CI runs"; no cargo commands inline.
- [ ] On `git push` to a `phase-v1-*` or `junior/*` branch, `cargo-validate-workspace.yml` triggers automatically.
- [ ] An impl-task subagent commits + pushes + DQ-writes + exits within ~5-15 minutes; no cargo wait.
- [ ] A `validate-pending` DQ entry appears within seconds of impl-task exit.
- [ ] Advisor's polling loop notices the `validate-pending` and queues a `ci-watcher` task.
- [ ] ci-watcher polls `gh run view` and writes `validate-result: pass` or `validate-failed: <log>` within 60 minutes (typically 10-25).
- [ ] Advisor's polling loop reads the result on next tick; on pass, advances; on fail, classifies + acts.
- [ ] No EliteDesk cgroup memory pressure during impl-task workflows.
- [ ] One full sub-phase (e.g. v1-JM-e) ships end-to-end via Shape G with no orphan cargo processes and no advisor-side cargo invocations.

## 13. Estimated effort (rough)

- Workflow authorship: 2-4 hours (one impl-task per workflow if the planner splits them).
- ci-watcher subagent + brief template: 2-3 hours (one impl-task).
- impl-task.md §5 rewrite + decision-queue.md schema additions + advisor-orchestrator.md stage-shape update: 2-3 hours (one impl-task — coupled changes).
- plan.template.md §15 + §13 update: 1-2 hours (one impl-task).
- Total: ~3-4 impl-tasks worth of work, ~10-15 hours of model + advisor time. Single sub-phase scoped.

## 14. Pre-commit dogfood (per `.claude/lessons/feedback_dogfood_slash_command_specs.md`)

Mentally walked through this brief against:

- **PMD #109 + #110** (real recent incident — task #13 wedge): under Shape G, the wedge would not have happened — the impl-task would have committed at ~15:26 UTC and exited; the cargo work would have run on a runner (~25 min cold cache), and ci-watcher would have polled and written the result by ~15:55 UTC. Total elapsed: 40 min vs today's 130+ min.
- **`validate-agent-design.md` Shape C**: the comparison table in §6 reads cleanly; both shapes are real options, neither dominates absolutely. Plan-mode session should pick one.
- **PMD #111 Server Boss alert tuning**: complementary, not redundant. Shape G removes one source of pressure (cargo); PMD #111 reduces alert noise on the remaining pressure. Both should ship.

What worked: §6 comparison table makes the trade-off explicit; §G4 + §G5 share the same content as Shape C, so adopting Shape G doesn't fork the future planning surface.

What didn't fit cleanly: the workflow-minute budget question (§6 cost note) is the strongest counter-argument; the brief flags it but doesn't resolve it. The planning session should compute "validation runs per sub-phase × wall-clock per run × $0.008/min" and decide whether the cost is acceptable. Initial estimate: 6 impl-tasks × 1.5 validations each (one + one fix-pass) × 20 min × ~$0.008 = ~$1.40/sub-phase. Probably fine even at scale.
