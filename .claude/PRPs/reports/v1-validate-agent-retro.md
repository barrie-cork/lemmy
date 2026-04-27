# v1-validate-agent retro — Out-of-Junior cargo validation via GitHub Actions

**Sub-phase:** v1-validate-agent (Shape G infrastructure: workflow YAMLs + ci-watcher subagent + schema-additive `kind` values + push-and-exit impl-task contract)
**Branch:** `phase-v1-validate-agent`
**Base:** `governance-v0` @ pre-cut tip (plan @ HEAD before branch cut)
**Plan:** `.claude/PRPs/plans/v1-validate-agent.plan.md` @ committed HEAD before branch cut (confidence 7/10)
**Dates:** 2026-04-27 (single-day execution; 7 commits including this retro)
**Impl model:** **foreground impl session, NOT four-role Junior orchestration.** This retro reflects that delta — see §2 framing.
**Commits ahead of `governance-v0`:** 7 (Tasks 1–5 + handover + this retro). 6 functional commits + 1 handover doc.

---

## TL;DR for the advisor

**Plan held up well; three planning-side surprises caught and self-resolved as `kind: log` DQ entries; one empirical finding promoted to a load-bearing lesson.** All Tasks 0–5 shipped. Schema additivity discipline (atomic Task 3 commit across `decision-queue.md` + `impl-task.md` + `advisor-orchestrator.md`) held. The bounded retrofit (Task 5: `jm-d-impl-2.md` §5 only) shipped cleanly. The empirical-probe gate on Task 2 caught the gh CLI 2.89.0 `--exit-status` bug that would have produced silent false-greens in ci-watcher classifier — Story 1 is partially exercised pending workflow-run-green confirmation; Stories 4 + 5 are `[deferred-to-retro]` because no real failure-allowlist scenario surfaced during execution.

The session ran as **foreground impl from a single laptop session**, not the four-role Junior orchestration the plan describes. The implementation pieces (ci-watcher.md, push-and-exit impl-task contract, validate-stage in advisor-orchestrator.md) were authored but not exercised end-to-end — v1-JM-e is the first sub-phase that will exercise them as designed.

Three planning-side issues stacked up during execution; all three are advisor-actionable carry-forwards:

1. **DQ #68** — `migrate-roundtrip.sh` was referenced as a pre-existing script but did not exist. Stub shipped that exits non-zero on real migration detection (forcing the next migration-authoring sub-phase to replace it). Plan §13 Task 1 cited "existing script per JM-d §10.6 / scheduled-tasks pattern" — neither source actually contains it. Phantom dependency.

2. **DQ #69** — `cargo check --workspace --features full --no-deps` was prescribed in plan §13 Task 1, but `--no-deps` is a clippy-only flag. First push failed at workflow run `25017407659` with `error: unexpected argument '--no-deps' found`. Fix-up commit `ed049970b` dropped it. The plan's §15 dry-run (yamllint / `gh workflow view --yaml`) cannot catch this class — first-push validation is the real gate. Promoted as `feedback_planner_dod_dry_run_caught_partial.md`.

3. **DQ #70** — `gh run watch --exit-status` returns exit 0 in ALL three observed terminal scenarios on gh CLI 2.89.0, despite documenting non-zero on failure. ci-watcher MUST disambiguate via `gh run view <id> --json conclusion`. Probes captured in `.claude/PRPs/debug/v1-validate-agent-task2-gh-run-watch-probes.log`. Promoted as `feedback_gh_run_watch_exit_status_unreliable.md` — load-bearing for any future workflow-watching agent.

Plus, one workflow-tooling surprise outside the plan: `dtolnay/rust-toolchain@master` silently ignored `components: clippy` because `rust-toolchain.toml` pins the channel. Fix added explicit `rustup component add clippy --toolchain 1.95-x86_64-unknown-linux-gnu` step. Promoted as `feedback_dtolnay_rust_toolchain_components_with_toml.md`.

---

## 1. What worked — keep doing

### 1.1 Plan §10 pattern-snippet discipline

Every §10.* block (§10.4 trigger + concurrency, §10.5 ci-watcher frontmatter, §10.6 brief template, §10.7 push-and-exit body, §10.8 hard-refusals append-line, §10.9 advisor stage-shape insertion, §10.10 plan-template §15.X, §10.11 jm-d-impl-2 retrofit) mapped near-1:1 onto the final commits. The single deviation was §10.7's reference to the `gh run list` retry loop — the implementation kept the wording but reality is that ci-watcher still runs separately, so the impl-task body is the canonical reference and matches.

**Keep**: §10 cite-and-mirror discipline. Every pattern named the source file + line; reading the source took <30 seconds in each case.

### 1.2 Empirical-probe gate on Task 2 (§4 watchpoint #1)

The plan mandated empirical validation of `gh run watch --exit-status` exit semantics BEFORE ci-watcher.md ships. This caught the gh CLI 2.89.0 false-zero bug (DQ #70). Without the gate, ci-watcher.md would have shipped with a classifier that branches on the watch exit code, producing silent false-greens for every failed workflow.

The probes covered 3 of 4 plan-prescribed scenarios (success / completed-failure / in-progress→failure). Cancelled + queued were deferred because the conclusion-string fallback (`gh run view <id> --json conclusion`) covers all eight enumerable conclusion values, making the deferred probes non-load-bearing for the design.

**Keep**: every plan that introduces a new tool dependency should include an empirical-probe gate in the §4 watchpoints. The cost is low (~30 minutes of probe time); the catch is potentially weeks of false-green debt.

### 1.3 Schema-additive atomic commit discipline (Task 3)

Per `feedback_update_rule_doc_with_schema_additive.md`: `decision-queue.md` (rule doc) + `impl-task.md` (writer of `validate-pending`) + `advisor-orchestrator.md` (reader at stage-shape map) committed atomically as one commit `70b3a4d08`. No window where the rule doc enumerated a kind that no consumer could write/read. The plan's GOTCHA framing made this mechanical: stage all three files, then commit.

**Keep**: schema-additive atomic commits. The temptation to split (especially when one file is much larger than the others) is real; the discipline pays off in zero schema-drift incidents.

### 1.4 Self-resolved `kind: log` DQ entries for planning-side findings

DQ #68, #69, #70 all went directly to `resolved[]` with `answered_by: "impl-self-resolved"` and `kind: "log"`. Each captured a finding the next planning task will want — phantom dependency, plan flag drift, empirical bug — without stopping the impl loop or surfacing to user. The advisor harvests these at retro time (§4 below).

**Keep**: `kind: log` for findings that are durable but non-blocking. Don't put them in `pending` (advisor's polling loop stops unnecessarily); don't put them in commit trailers (less searchable). DQ resolved-array placement is the right axis.

### 1.5 Bounded retrofit discipline (DQ #61 + Task 5)

Task 5 retrofitted ONE brief — `jm-d-impl-2.md` §5 — and stopped there. Did not cascade into other §X edits in the same brief, did not edit any other JM-d brief. The plan's GOTCHA framed the boundary explicitly ("retrofit boundary is §5 only"), and the validate block confirmed no other JM-d brief was modified in the commit (`grep -v 'jm-d-impl-2\.md' | wc -l → 0`).

**Keep**: when a sub-phase introduces a forward-only schema change with one bounded retrofit exception, the plan should name the exception precisely (file + section) and the validate block should mechanically confirm the boundary held. Both worked here.

### 1.6 Lessons promoted in same retro commit

Three new `feedback_*.md` files in `.claude/lessons/` ship in the same commit as this retro per `feedback_one_system_memory_in_repo.md`. v1-JM-e + future Shape G plans pick them up immediately on next session start.

**Keep**: lessons promoted at retro commit time, not deferred. Deferral is how lessons rot.

---

## 2. Per-role signals (foreground caveat)

> **Important framing:** this sub-phase ran as **foreground impl session on the laptop**, not under the four-role Junior orchestrator the plan was authored for. v1-validate-agent is the LAST infrastructure step before v1-JM-e becomes the first fully-four-role Shape G sub-phase. Several of the four-role signals below are therefore "designed but not exercised" rather than "exercised cleanly."

### 2.1 Advisor signals — N/A this session

The persistent advisor session lives in `homeserver/` and did not participate. The user gates (plan approval, CR triage approval, merge confirm) are still real but happened in-conversation rather than via polling-loop surfacing. The validate-stage transitions in advisor-orchestrator.md (§10.9 insertion) and the §G4 classifier are authored but not yet exercised end-to-end against a real cohort dispatch.

**Carry-forward for v1-JM-e retro:** the first real exercise of the validate-stage will be v1-JM-e Task 1 (or whichever cohort runs first under Shape G). Watch-items: does the advisor's polling loop correctly route `(validate-pending, pending)` → ci-watcher dispatch within one polling tick? Does cohort dispatch wait for ALL members to reach `validate-result: pass` before advancing?

### 2.2 Planning signals — three plan-side surprises, one watchpoint near-miss

The plan's accuracy was high overall but missed three flag/dependency-level details:

- **Phantom dependency (DQ #68):** `migrate-roundtrip.sh` cited as pre-existing was not. Plan-side checking should have run `test -e scripts/brehon/migrate-roundtrip.sh` against current HEAD before commit. Generalises to any plan that cites a sibling-script "per pattern X" — verify the script actually exists.
- **Flag drift (DQ #69):** `cargo check --workspace --features full --no-deps` from clippy DoD inheritance. Plan §15 dry-run (yamllint / gh workflow view) catches YAML parse but not nested-shell semantic correctness. Promoted as `feedback_planner_dod_dry_run_caught_partial.md` — first-push validation is the real gate.
- **dtolnay/rust-toolchain components ignored (no DQ — runtime-discovered):** plan §13 Task 1 IMPLEMENT block did not anticipate the silent-ignore behaviour when `rust-toolchain.toml` is present. Fix-up commit added explicit `rustup component add clippy` step. Promoted as `feedback_dtolnay_rust_toolchain_components_with_toml.md`.

Watchpoint accuracy was good: §4 watchpoint #1 (`gh run watch --exit-status` empirical validation) was load-bearing and gated correctly — DQ #70 surfaced because the gate fired. Watchpoints #2-9 held (no schema breach, no anchor-link rot, no cache-key collision).

**Carry-forward:** when a future plan references a sibling script "per pattern X / per phase Y," the plan author should `test -e <path>` against HEAD before commit and remove the citation if the script is missing. The "per pattern" framing is sometimes a rationalisation for assumed-but-not-verified prior art.

### 2.3 Impl signals — foreground delta, manual mid-task pushes

This session played impl. Notes on the foreground-vs-Junior delta:

- **No `[role:impl-task]` dispatch line.** The session opened on the worktree directly, read the brief equivalent (the plan), and proceeded.
- **No Junior worktree isolation.** All seven commits shipped on the same `phase-v1-validate-agent` branch in the same worktree. No mid-task DQ trapping (the issue `feedback_decision_queue_mid_task_visibility.md` addresses).
- **Manual mid-task push for DQ visibility.** I pushed each commit immediately after committing rather than at finalize-time, which is what Junior subagents would do automatically. No regressions.
- **Cohort dispatch was sequential, not parallel.** The plan's `[P]+[P]` Task 1 + Task 2 cohort and Task 4 + Task 5 cohort ran one-at-a-time because I'm one session; cohort-parallelism is a four-role-only feature. No throughput cost — the tasks were small enough that sequential vs parallel was within minutes either way.

The push-and-exit contract in `impl-task.md` itself is unexercised end-to-end; v1-JM-e will be the first real test. Watch-items: does `gh run list --branch <branch> --limit 1 --json databaseId` correctly capture the workflow_run_id within the 1-2 min push-to-trigger lag? Does the retry/backoff loop work as designed?

### 2.4 BM signals — pending

BM-equivalent work (PR creation + CR triage + merge) is still pending and will happen post-retro. The BM session for v1-validate-agent will exercise:
- `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-validate-agent` per `gh-pr-fork-target.md` + `phase-branch.md`
- CodeRabbit auto-review (`.coderabbit.yaml` triggers on PRs into `governance-v0`)
- Four-bucket CR triage per `feedback_pr_review_triage_pattern.md`
- `/brehon-verify v1-validate-agent` to iterate §16a stories
- `bm-merge` per `feedback_pr_per_phase.md` (NOT --squash; task-per-commit history is load-bearing for retros)

Watch-items at retro time: did CR find anything substantial? Did any finding contradict an ADR or watchpoint?

---

## 3. Per-task complexity score table

Per `feedback_retro_task_complexity_score.md`, the metric format is `<files-touched>/<commits>/<runtime-min>/<max-log-silence-min>`. For foreground impl sessions, `runtime-min` = wall-clock from task-start to commit-push; `max-log-silence-min` is N/A because there's no Junior watchdog (foreground sessions don't have stdout-stall risk).

| Task | Subject | Files | Commits | Runtime (min) | Watchdog risk | Notes |
|---|---|---|---|---|---|---|
| 0 | Pre-flight harness audit | 0 | 0 | ~10 | N/A (verification only) | All 9 probes passed |
| 1 | Workflow YAMLs + migrate-roundtrip.sh stub | 3 | 2 | ~50 | low (foreground) | 2 commits because of `--no-deps` fix-up `ed049970b` |
| 2 | ci-watcher subagent + brief template | 3 | 1 | ~75 | medium (foreground) | Empirical probe time + clippy-toolchain fix bundled in same commit |
| 3 | Schema-additive atomic | 3 | 1 | ~40 | low (foreground) | Atomic commit held; no split |
| 4 | Plan template §15 + §13 | 1 | 1 | ~12 | low (foreground) | Forward-only-immune; no cascade |
| 5 | jm-d-impl-2.md §5 retrofit | 1 | 1 | ~10 | low (foreground) | Bounded; no other JM-d brief touched |
| 6 | Retro + lessons promoted | 4 (this commit) | 1 | ~45 | low | Including 3 new lessons |

**Aggregate:** 15 distinct files, 7 commits, ~242 min total wall-clock. Median task ~30 min. Outlier: Task 2 (~75 min) — empirical probes + clippy-toolchain fix added time. Under Junior four-role with Sonnet impl-task watchdog (60 min wall-clock + 40 min log-silence), Task 2 would be at-risk; the embedded fix-up makes it borderline.

**Carry-forward:** if Task 2 is split into "Task 2a: empirical probes + ci-watcher.md" and "Task 2b: dtolnay/rust-toolchain fix-up" the wall-clock drops below the 60 min envelope. Worth flagging for the next plan that bundles empirical-probe work with file authoring — split aggressively.

---

## 4. Lessons promoted to `.claude/lessons/`

Three new lessons land in the same commit as this retro:

1. **`feedback_gh_run_watch_exit_status_unreliable.md`** — load-bearing. gh CLI 2.89.0 `gh run watch --exit-status` returns exit 0 in all observed terminal scenarios; ci-watcher (and any future workflow-watching agent) must disambiguate via `gh run view <id> --json conclusion`. Source: DQ #70.

2. **`feedback_dtolnay_rust_toolchain_components_with_toml.md`** — high-applicability. `dtolnay/rust-toolchain@master` silently ignores `components:` input when `rust-toolchain.toml` pins a channel. Workaround: explicit `rustup component add` step. Generalises to rustfmt, rust-src, any `rustup` component. Source: workflow run `25017554049` failure.

3. **`feedback_planner_dod_dry_run_caught_partial.md`** — meta-pattern. Plan §15 dry-run (yamllint, gh workflow view) catches YAML parse errors but cannot catch invalid cargo flags inside YAML `run:` bodies. Generalises to Dockerfile RUN, npm scripts, Makefile recipes. First-push validation is the real gate. Source: DQ #69.

All three pass the lesson-template shape (`name:` / `description:` / `type: feedback`) and follow the canonical "Why / How to apply / Generalises to / Symptom to recognise" body shape per `feedback_read_canonical_before_writing_spec.md`.

---

## 5. Watch-items for next sub-phase (v1-JM-e first under Shape G)

### 5.1 First fully-four-role exercise

v1-JM-e is the first sub-phase intended for full four-role Junior orchestration under Shape G. Watch-items:

- Does the advisor's stage-shape `(validate-pending, pending)` → ci-watcher dispatch happen within one polling tick (~10 min)?
- Does ci-watcher's `gh auth status` pre-flight catch the missing-auth case (or is `barrie-cork` auth permanent on the EliteDesk)?
- Does the conclusion-string fallback (per `feedback_gh_run_watch_exit_status_unreliable.md`) classify a real failure correctly into `validate-failed` with a non-empty `log_slice`?
- Does cohort-parallel `validate-pending` work as designed (each cohort member spawns its own workflow run; advisor advances on all-pass)?

### 5.2 §G4 classifier allowlist tuning (deferred Story 5)

Story 5 is `[deferred-to-retro]` because no real allowlist-eligible failure surfaced during execution (the clippy-toolchain-missing failure observed in 25017554049 is a workflow-config issue, not a code-level lint, so even classification would have catch-fired). After 2-3 sub-phases run under Shape G with real `validate-failed` signals, the allowlist's true-positive rate becomes measurable. Recommend v1-JM-e or v1-rep-tuning-r3 retro propose additions/removals.

### 5.3 Branch protection rules (DQ #66 deferred)

The workflows trigger on push (not pull_request); GitHub branch-protection rules need configuring out-of-band to require `cargo-validate-workspace.yml` to pass before merging into `governance-v0`. This is a one-time GitHub-UI action, not a plan deliverable. Recommend doing it during BM session for v1-validate-agent merge (or right after) so v1-JM-e onward inherits the protection.

### 5.4 GitHub Actions minutes-budget signal

Estimate: 80-200 validations/month free-tier; ~$1.40/sub-phase paid. For v1-JM-e onward, watch the monthly minutes consumption — if approaching 2000/month for the private repo, surface as catch-fire. Per plan §18 risk row 2 (LOW likelihood, MED impact).

### 5.5 PMD ingest pipeline gap (handover §"Phase outputs (synthesised)" #9)

Three brief-named lessons (`feedback_library_add_after_shipping`, `feedback_update_rule_doc_with_schema_additive`, `feedback_commit_aggressively_in_shared_repos`) exist as homeserver user-scope `.md` files but were NOT surfaced by `memory_search_hybrid` during plan-write (recovered via direct file read). NOT a brehon-fork plan deliverable; surfaced for homeserver-side diagnostic.

### 5.6 ci-watcher Haiku-4.5 effort calibration

ci-watcher is pinned to Haiku 4.5, low effort, narrow tools. First real run under v1-JM-e will reveal whether the pin is correct: too low (model can't classify) or too high (cost dominates). The model is mechanical — Read DQ entry, run gh commands, write DQ entry — so low-effort should suffice. Confirm at v1-JM-e retro.

### 5.7 PR-side status checks via branch protection (sibling to 5.3)

Once branch protection requires `cargo-validate-workspace.yml` to pass on PRs into `governance-v0`, every phase PR gets a status check. Should improve CR triage (CR has more signal: lint-clean + test-clean + build-clean before review starts). Watch at v1-JM-e PR for any CR-vs-status-check tension.

### 5.8 `migrate-roundtrip.sh` stub replacement

The stub at `scripts/brehon/migrate-roundtrip.sh` exits non-zero on real migration detection vs `governance-v0` — forces the next migration-authoring sub-phase (likely v1-JM-d Task 2 once unblocked, or earlier if a v1.x sub-phase ships migrations) to replace it with a real round-trip implementation. Document as a TODO at `scripts/brehon/migrate-roundtrip.sh:1-3` so the next migration author sees it.

---

## 6. Confidence score

**8/10.**

Higher than plan's predicted 7/10 because the empirical-probe gate worked (caught the gh CLI bug before ci-watcher.md shipped), the schema-additive atomic commit held, and the bounded retrofit was clean. The two-decimal confidence boost: empirical evidence > plan-time estimate.

Discounted from 10/10 because:

- Story 1 (workflow run green) acceptance is conditional pending the in-progress workflow run `25018404875`; if it fails for a fourth reason, the retro overstates "Tasks 0-5 shipped clean."
- Stories 4 + 5 are `[deferred-to-retro]` — not failures, but unexercised. v1-JM-e is the real test.
- Foreground-vs-Junior delta means the four-role pieces are designed but not exercised end-to-end. Some risk that v1-JM-e surfaces orchestration bugs the foreground session masked.
- Three plan-side surprises (DQ #68, #69, #70) on a 7-task plan is ~40% surprise rate. Lower than v1-AD or v1-JM-a/b/c retros measured, but still room for plan-side tightening (the surprises are themselves the planner's feedback signal — they fed three lessons).

The headline acceptance ("an impl-task brief on a `phase-v1-*` branch can commit + push + DQ-write + exit within ~5-15 min, the workflow auto-triggers, ci-watcher polls to completion within the 60-min cap, and the advisor's polling loop reads the result on the next tick — with zero local cargo invocations on Junior") is **designed and shipped, but not exercised end-to-end**. v1-JM-e will be the real test.

---

_Retro author: foreground impl session (laptop, `C:\Users\barri\Developer\brehon-fork`,
2026-04-27). Resumed from `.claude/PRPs/handovers/impl-2026-04-27-v1-validate-agent-tasks-3to5-and-2-shipped.md` @ `7bafb078c`. Three lessons promoted to `.claude/lessons/` in the same commit. PR + CR triage + bm-merge pending user approval per `phase-branch.md`._
