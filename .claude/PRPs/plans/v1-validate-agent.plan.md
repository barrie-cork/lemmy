# Plan: v1-validate-agent — Out-of-Junior cargo validation via GitHub Actions + ci-watcher polling

## Table of contents

1. Summary
2. Source
3. Problem statement
4. Solution statement (architectural decisions + watchpoints)
5. Metadata
6. Relationship to other v1 sub-phases
7. Preflight guardrails inherited from prior phases
8. Flow design
9. Mandatory reading
10. Patterns to mirror
11. Files to change
12. NOT building in v1-validate-agent
13. Step-by-step tasks
14. Testing strategy
15. Validation commands (DoD)
16. Acceptance criteria
16a. Stories (independently-testable behaviour units)
17. Completion checklist
18. Risks and mitigations
19. Notes
20. Sub-phase stub (v1-validate-agent retro follow-ups)

---

## 1. Summary

v1-validate-agent is a post-v0 infrastructure sub-phase (NOT v0 endpoint
scope) that **moves cargo validation off the EliteDesk Junior worker
slot and onto GitHub-hosted Actions runners**. It ships two new workflow
YAMLs (`cargo-validate-workspace.yml` for `check + clippy + test --no-run`,
`cargo-validate-migration.yml` for migration round-trip), a fifth Junior
subagent role `[role:ci-watcher]` (Haiku 4.5, low effort, mechanical
polling) that consumes workflow runs via `gh run watch --exit-status`
and writes results back into the decision-queue, and an additive schema
extension to `decision-queue.md` introducing three new `kind` values
(`validate-pending`, `validate-result`, `validate-failed`) plus one new
`from` value (`ci-watcher`). impl-task switches to a push-and-exit
contract (Layer G2): commit, push, write a `validate-pending` DQ entry,
exit; cargo no longer runs locally on Junior. The advisor's
`stage-shape` map gains a validate stage between impl-commit and the
next cohort dispatch. Plan-template §15 introduces a "DoD per workflow"
shape forward-only; pre-existing JM-d/JM-c/JM-b/JM-a plans stay under
prose §15 (no retrofit). One bounded retrofit ships in this sub-phase:
`.claude/PRPs/briefs/jm-d-impl-2.md` §5 switches from inline cargo to
push-and-exit per DQ #61. Headline acceptance: an impl-task brief on a
`phase-v1-*` branch can commit + push + DQ-write + exit within ~5–15
min, the workflow auto-triggers, ci-watcher polls to completion within
the 60-min cap, and the advisor's polling loop reads the result on the
next tick — with zero local cargo invocations on Junior.

---

## 2. Source

- **Originating design brief**: [.claude/PRPs/briefs/ci-validation-design.md](.claude/PRPs/briefs/ci-validation-design.md) (Shape G — load-bearing source-of-truth) @ `5f46d138b`
- **Sibling parked design**: [.claude/PRPs/briefs/validate-agent-design.md](.claude/PRPs/briefs/validate-agent-design.md) (Shape C — PARKED, NOT KILLED; cite for §G4 escalation contract + §G5 planning-side implications shared with Shape G)
- **Planning brief for this plan**: [.claude/PRPs/briefs/v1-validate-agent-planning-1.md](.claude/PRPs/briefs/v1-validate-agent-planning-1.md) @ `28b2e20bc`
- **Handover from paused plan-write session**: [.claude/PRPs/handovers/advisor-2026-04-27-v1-validate-agent-planning.md](.claude/PRPs/handovers/advisor-2026-04-27-v1-validate-agent-planning.md) @ `92433e9ea`
- **Decision-queue inputs (resolved, governing)**:
  - **DQ #61** (advisor 2026-04-27, plan-mode) — Shape G chosen, JM-d Task 2 parked, single-brief bounded retrofit (`jm-d-impl-2.md` §5). Most recent advisor-attributed scope statement.
  - **DQ #62** (advisor 2026-04-27 clarify) — ci-watcher uses `gh run watch <id> --exit-status` (single long-poll, 1 API call/task), NOT `gh run view` polling. Plan §4 watchpoint #1 requires empirical validation of `gh run watch` exit semantics before authoring ci-watcher.md.
  - **DQ #63** (advisor 2026-04-27 clarify) — schema additive: add `from: "ci-watcher"` only; impl-task continues as `from: "impl"` even when entry carries `kind: "validate-pending"`. NEVER rewrite historical entries.
  - **DQ #64** (advisor 2026-04-27 clarify) — impl-task.md sections are NAMED, not numbered. Rename "Per-task validation gate" → "Per-task validation gate (out-of-band on GH Actions)"; rewrite body; append one line to "Hard refusals". Reference H2 NAMES, never §-numbers.
  - **DQ #65** (advisor 2026-04-27 clarify) — `.claude/PRPs/templates/plan.template.md` is the canonical structural template (post-spec-kit). 20-section schema is load-bearing.
  - **DQ #66** (user 2026-04-27 clarify) — workflow trigger pattern is `on: { push: { branches: ['phase-v1-*', 'junior/*'] } }` only. NO `pull_request` trigger. Defence-in-depth via branch-protection + advisor-orchestrator merge gate.
  - **DQ #67** (user 2026-04-27 clarify) — workflow YAML §15 dry-run discipline is YAML parse only via `yamllint <path>` exit 0 OR `gh workflow view --repo barrie-cork/lemmy --ref <phase-branch> <name>.yml`. End-to-end validation = "first test push" §13 task DoD step. Do NOT prescribe `act` invocations.
- **Decision-queue inputs (Shape-agnostic, resolved, applying under Shape G)**:
  - **DQ #55** (advisor) — Layer A subagent (now `[role:ci-watcher]`) runs under existing `junior@brehon-fork` cgroup (10G/8G); no new systemd unit. Reinterpret: ci-watcher is mechanical Haiku-4.5 polling, no cargo, so the cgroup cap is non-binding for it.
  - **DQ #57** (user) — fix-vs-escalate threshold for the failure classifier: ≤3 file edits = auto-fix-impl-task brief; >3 = surface to user. Under Shape G the classifier is the advisor's own §G4 logic operating on `validate-failed` DQ entries (not a subagent's own decision).
  - **DQ #59** (advisor) — branch model = phase-branch + PR. Cut `phase-v1-validate-agent` off `governance-v0` via `bm-cut`; final PR back with CodeRabbit review per `feedback_pr_per_phase.md`.
- **Decision-queue inputs (Shape-C-specific, resolved, NOT applying)**: DQ #56, #58, #60. Cited for historical context only; do NOT honour their answers literally under Shape G.
- **Most recent shipped plan (mirror §1..§14, §16..§20 shape)**: [.claude/PRPs/plans/v1-jury-mechanics-d.plan.md](.claude/PRPs/plans/v1-jury-mechanics-d.plan.md) @ MERGED PR #95
- **§16a + `[P]` cohort-marker pattern source**: [.claude/PRPs/plans/v1-jury-mechanics-c.plan.md](.claude/PRPs/plans/v1-jury-mechanics-c.plan.md) @ `4d2b93ed9`
- **Lessons that bind decisions** (each cited in §10 / §13 / §15 / §17 below):
  - `feedback_clarify_before_plan.md` — clarify gate already ran (DQ #62–#67 closed)
  - `feedback_parallel_cohort_dispatch.md` — `[P]` markers; budget check
  - `feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md` — every §15 entry dry-run before commit
  - `feedback_pipes_mask_exit_codes.md` — exit-code discipline (applies to `gh run watch --exit-status` validation)
  - `feedback_story_grain_checkpoint.md` — §16a Stories block
  - `feedback_pr_per_phase.md` — code via PR; meta-work via direct. Two new workflow files in `.github/workflows/` make this a code-PR.
  - `feedback_dogfood_slash_command_specs.md` — applied to ci-watcher.md (mental-walk one validate-failed shape per brief §109; this plan does NOT introduce slash commands so the gate is non-binding for plan body)
  - `feedback_schema_changing_spec_retrofit_question.md` — schema-additive `kind` values triggered the retrofit question; resolved by DQ #61 (bounded retrofit = jm-d-impl-2.md §5 only)
  - `feedback_read_canonical_before_writing_spec.md` — applied to ci-watcher.md authoring (mirror impl-task.md/bm-task.md/planning.md frontmatter shape)
  - `feedback_features_full_p_crate_incompatible.md` — workspace-only DoD (already known)
  - `feedback_advisor_watchpoint_specificity.md` — every §4 watchpoint cites file/line
  - `feedback_principles_not_rules.md` — applies to plan §13 skill-trigger hints
  - **Brief-cited lessons (recovered via homeserver session, not in `.claude/lessons/` at plan-write time)**: `feedback_library_add_after_shipping.md` (drives §17 library.yaml line), `feedback_update_rule_doc_with_schema_additive.md` (drives Task 3 atomicity), `feedback_commit_aggressively_in_shared_repos.md` (drives one-commit-per-task wording)
- **Relevant ADRs**: ADR-014 (federation of governance signals — only constraint here is that none of v1-validate-agent's deliverables touches AP federation; non-binding for this plan). No ADR contradicts this design.

---

## 3. Problem statement

Three concrete failures the v0 → v1 in-band validation model produces,
each tied to a §10 pattern or §13 task:

- **EliteDesk resource contention.** The Junior daemon's `cargo
  check --workspace --features full` competes with `rust-analyzer`
  LSP processes, OpenSearch JVM (~2.2 GB), other server daemons, and
  the cgroup memory ceiling. Today's pattern of cgroup at 8.6 GB / 10
  GB cap during a single workspace check is not sustainable as more
  sub-phases queue impl-tasks. Tied to §13 Task 1 (workflow YAMLs
  shift the workload off-box).

- **Orphan-process risk.** PMD #110 surfaced that task #13 left an
  orphan `cargo check` process (PID 210539) that survived the worker
  SIGTERM and held 5.4 GB of cgroup memory until manual cleanup.
  GitHub runners are destroyed after each job — orphans are
  physically impossible. Tied to §13 Task 1 (ephemeral runners
  eliminate this entirely).

- **impl-task wall-clock occupancy on cargo runs the model doesn't
  attend.** Today an impl-task subagent stays alive for the full
  10–25 min cargo workspace+features-full check, blocking the next
  task on the worker slot. Under Shape G the impl-task exits in 5–15
  min after `git push`, freeing the slot immediately; ci-watcher
  (Haiku 4.5, low effort) holds the polling slot at 1/15th of an
  impl-task's model cost. Tied to §13 Tasks 2 + 3 (ci-watcher +
  impl-task contract change).

A fourth, lower-priority concern — visibility: today's local cargo
validations produce no PR-side status checks. Shape G's workflow runs
appear in the GitHub UI and on PR pages. Non-blocking benefit; not a
plan-driver.

---

## 4. Solution statement

### 4.1 Architecturally load-bearing decisions locked in this sub-phase

1. **Workflow execution location**: GitHub-hosted Actions runners
   (ubuntu-latest), not self-hosted EliteDesk runners. Per
   ci-validation-design.md §11 (self-hosted runners re-introduce the
   resource contention this design solves; out of scope unless private-
   repo minute limits become binding for >2 consecutive months).

2. **Workflow trigger pattern** (DQ #66, load-bearing): `on: { push:
   { branches: ['phase-v1-*', 'junior/*'] } }` only. NO `pull_request`
   trigger. Defence-in-depth via branch-protection rules + advisor-
   orchestrator merge gate (validate-result must exist on the latest
   phase-branch SHA before bm-merge). This keeps GitHub Actions free-
   tier minutes bounded at ~80–200 validations/month (~$1.40/sub-phase
   even paid; comfortable for 4–5 sub-phases/month).

3. **ci-watcher polling mechanism** (DQ #62, load-bearing): `gh run
   watch <id> --exit-status` (single long-poll, 1 API call per task),
   NOT repeated `gh run view <id>` polling. Mitigates the GitHub `gh
   api` rate-limit (5000 req/hour) failure mode in design §8.3. Exit-
   code semantics MUST be empirically validated before authoring
   ci-watcher.md (§4.2 watchpoint #1).

4. **Schema additive — three new `kind` values + one new `from` value**:
   `validate-pending` (`from: "impl"`, written by impl-task post-push),
   `validate-result` (`from: "ci-watcher"`, `result: "pass"`), and
   `validate-failed` (`from: "ci-watcher"`, `result: "fail" | "timeout"`,
   includes a log slice + failed-jobs list). The new `from: "ci-watcher"`
   is the only new `from` (DQ #63). impl-task continues writing as
   `from: "impl"` even when its entry carries `kind: "validate-pending"`.

5. **Schema atomicity** (per `feedback_update_rule_doc_with_schema_additive.md`):
   `decision-queue.md` (the rule doc) + `impl-task.md` (the writer of
   `validate-pending`) + `advisor-orchestrator.md` (the reader at the
   stage-shape map) commit atomically as Task 3. Splitting risks a
   window where the rule doc enumerates a kind that no consumer can
   write/read.

6. **§G4 classifier scope** (DQ #57): the advisor's classifier
   (lives in `advisor-orchestrator.md`'s "Catch-fire procedures" /
   "Stage-shape orchestration") auto-queues a fix-impl-task only for
   trivial fixes ≤3 file edits matching a narrow allowlist (clippy
   lints with auto-fix, missing imports, deprecated APIs). Anything
   else surfaces to user. The classifier is advisor logic, not
   ci-watcher logic — ci-watcher writes the `validate-failed` entry
   and exits; the advisor reads it on the next polling tick.

7. **impl-task contract change (Layer G2)**: rename "Per-task
   validation gate" H2 → "Per-task validation gate (out-of-band on GH
   Actions)" and rewrite the body to: commit, push, capture
   `workflow_run_id` via `gh run list --branch <branch> --limit 1
   --json databaseId --jq '.[0].databaseId'` (with a small retry/backoff
   loop for the 1–2 min push-to-trigger lag per design §8.4), append a
   `validate-pending` DQ entry containing `kind`, `from: "impl"`,
   `workflow_run_id`, `branch`, `phase_task`, commit + push the DQ
   update, exit. Append one line to "Hard refusals": "Never invoke
   cargo for build/lint/test — validation runs out-of-band on GH
   Actions per the validation gate above" (DQ #64).

8. **Forward-only retrofit scope** (DQ #61): single bounded
   retrofit — `jm-d-impl-2.md` §5 switches from inline cargo to push-
   and-exit. All other JM-d briefs (jm-d-impl-1, 3, 4, 5, 6, 7) and
   all earlier-phase briefs/plans (v1-AD-*, v1-JM-a/b/c) stay forward-
   only-immune. Plan-template §15 "DoD per workflow" shape applies to
   v1-JM-e and later; pre-existing plans stay under prose §15.

9. **Branch model**: `phase-v1-validate-agent` cut off `governance-v0`
   via `bm-cut` AFTER plan approval. PR back to `governance-v0` per
   `feedback_pr_per_phase.md` (code-touching: two new YAMLs in
   `.github/workflows/` qualify as code-PR scope). CodeRabbit auto-
   reviews. Final merge unblocks JM-d Task 2 queueing.

### 4.2 Watchpoints (file:line specific — per `feedback_advisor_watchpoint_specificity.md`)

1. **`gh run watch --exit-status` exit-code semantics** — no codebase
   precedent (per DQ #62 + Phase-3 doc fetch in handover §"Phase
   outputs (synthesised)"). External docs at
   https://cli.github.com/manual/gh_run_watch confirm `--exit-status`
   returns non-zero on failure but DO NOT document success exit code,
   timeout behaviour, or behaviour on `cancelled`/`queued`. The
   implementing session for Task 2 MUST run these four probes
   empirically against real workflow runs before finalising
   ci-watcher.md, capturing exit codes:
   ```bash
   gh run watch <success-id> --exit-status; echo "success: $?"
   gh run watch <failure-id> --exit-status; echo "failure: $?"
   gh run watch <cancelled-id> --exit-status; echo "cancelled: $?"
   gh run watch <queued-id> --exit-status; echo "queued (timeout test): $?"
   ```
   ci-watcher.md classifier logic must branch on the empirically-
   observed exit codes, not on assumptions.

2. **Workflow trigger pattern** (DQ #66) — `on: { push: { branches:
   ['phase-v1-*', 'junior/*'] } }` only in BOTH new workflows. NO
   `pull_request` trigger. The first test push (Task 1.5 sub-step in
   §13 — see "first test push" DoD step) confirms the trigger fires
   on a real push to a `phase-v1-*` branch.

3. **`actions/cache@v4` cache-key collision avoidance** — new keys
   `cargo-validate-workspace-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}`
   and `cargo-validate-migration-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}`.
   Must NOT collide with existing `cargo-e2e-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}`
   in `.github/workflows/cargo-test-e2e.yml:65`. Restore-keys: prefix-
   based (`cargo-validate-workspace-${{ runner.os }}-`) so cache thaws
   warm across `Cargo.lock` updates.

4. **`impl-task.md` "Per-task validation gate" — section anchor
   preserved on rename** — current H2 at `.claude/agents/impl-task.md:53`.
   New title: "Per-task validation gate (out-of-band on GH Actions)".
   The Markdown anchor changes (slug becomes
   `per-task-validation-gate-out-of-band-on-gh-actions`); audit any
   cross-file links that target the old anchor before commit. Concrete
   audit command: `rg -n 'per-task-validation-gate' .claude/`. Update
   any hits in the same Task 3 commit.

5. **`decision-queue.md` Subagents-and-attribution enumeration** —
   currently at `.claude/rules/decision-queue.md:462`. Adding
   `ci-watcher` requires (in order, all in Task 3):
   - the subagent name + dispatch line `[role:ci-watcher]`
   - `from: "ci-watcher"` write rule (§G3)
   - `kind: "validate-result" | "validate-failed"` write rule
   - `ci-watcher-self-resolved` `answered_by` rule
   Per DQ #63: do NOT rename `impl` to `impl-task`; do NOT rewrite
   historical entries. Three new `kind` values added to the
   "kind: blocker vs log vs clarify" sub-section at
   `.claude/rules/decision-queue.md:147` — note that the sub-section
   title needs renaming too (e.g. "kind: blocker vs log vs clarify vs
   validate-* (3 variants)"). Polling-loop routing table (currently
   "Polling-loop routing per kind", roughly lines 192–208) extends
   with three new (kind, status) routes:
   - `(validate-pending, pending)` — advisor dispatches a
     `[role:ci-watcher]` task on the next poll. Same task that raised
     it stays gated until ci-watcher resolves.
   - `(validate-result, resolved)` — advisor reads `result: "pass"`
     and advances the §13-task pipeline.
   - `(validate-failed, pending)` — advisor reads, runs §G4
     classifier; either auto-queues a fix-impl-task (≤3 edits,
     allowlist match) or catch-fires to user.
   Hard-refusals (write-side) at roughly lines 327–337 gain one
   bullet: "NEVER write `kind: "validate-result" | "validate-failed"`
   from a non-ci-watcher session." Hard-refusal #6 (kind: "clarify"
   from non-advisor) stays in place as a sibling.

6. **`advisor-orchestrator.md` Stage-shape orchestration insertion
   point** — between current line 197 ("Each impl-task complete")
   and line 198 ("All §16a stories `[done]`"). Two new bullets
   inserted as one block in Task 3:
   - **Each impl-task complete (under Shape G)** → impl-task already
     wrote `validate-pending`; advisor queues `[role:ci-watcher]`
     task with the brief at `.claude/PRPs/templates/ci-watcher-brief.template.md`
     populated from the DQ entry's `workflow_run_id` + `branch` +
     `phase_task`.
   - **ci-watcher complete** → read DQ entry; if `validate-result`
     pass, advance per old §"Each impl-task complete" rule (cohort
     check); if `validate-failed`, run §G4 classifier (allowlist
     match → queue fix-impl-task; else catch-fire to user).
   "Forbidden execution windows" at lines 63–102 gains a one-line
   note that cargo no longer runs locally for impl-task throughput
   under Shape G; ad-hoc local cargo validation by advisor pre-plan-
   approval still respects the windows. "Catch-fire procedures" at
   lines 261–272 gains 1–2 new triggers: workflow timeout (60-min
   ci-watcher cap exceeded), ci-watcher classifier miss (`gh run
   watch --exit-status` returned an exit code not enumerated in the
   classifier table). "Cohort dispatch" at lines 217–243 gains a
   parallel-validate-pending clause: cohort members can each be in
   `validate-pending` simultaneously since each has its own workflow
   run; advisor advances cohort only when ALL members reach
   `validate-result: pass`.

7. **§G4 classifier allowlist** — the narrow allowlist for trivial
   auto-fix-impl-tasks lives in `advisor-orchestrator.md` (Task 3,
   inside the Stage-shape orchestration insertion). Allowlist entries
   for v1-validate-agent ship: `clippy::doc_lazy_continuation` (auto-
   fix per `feedback_clippy_doc_lazy_continuation_in_doc_comments.md`),
   missing imports (`error[E0432]: unresolved import`), deprecated
   APIs (`warning: use of deprecated`). Anything else (compile errors,
   test failures, e2e flakes, panics) → catch-fire to user. The
   allowlist is conservative by design (per ci-validation-design.md
   §8.7); it can grow at retro time after empirical evidence.

8. **`gh` CLI auth on EliteDesk** — design §8.5 says `barrie-cork`
   already authenticated. ci-watcher.md MUST include a `gh auth
   status` pre-flight probe (mirror the impl-task.md task-0 forbidden-
   window check shape) before any `gh run watch` call. If unauth:
   ci-watcher writes a `validate-failed` DQ entry with reason
   `gh_unauth` and exits — the advisor surfaces it as a catch-fire,
   the human re-runs `gh auth login`, then re-queues ci-watcher.

9. **PMD search-coverage gap** (handover-noted) — three brief-named
   lessons (`feedback_library_add_after_shipping`,
   `feedback_update_rule_doc_with_schema_additive`,
   `feedback_commit_aggressively_in_shared_repos`) exist as homeserver
   user-scope `.md` files but were NOT surfaced by `memory_search_hybrid`
   even with their keywords in the query. Likely cause: 2026-04-27 PMD
   backfill didn't ingest these. NOT a plan-blocker; surfaced for
   retro to flag PMD ingest pipeline. §19 carries this forward.

---

## 5. Metadata

- **Phase:** `v1-validate-agent`
- **Branch:** `phase-v1-validate-agent` (cut by `bm-cut` AFTER plan approval)
- **Estimated tasks:** 7 (Task 0 pre-flight + Tasks 1–5 deliverables + Task 6 retro)
- **Estimated cargo budget:** N/A — this sub-phase does NOT run cargo on Junior. Workflow YAMLs run cargo on GitHub runners (~10–25 min cold cache, ~5–10 min hot), but local impl-task throughput is non-cargo. Local advisor-side dry-run of §15 entries (Task 0 audit) uses the existing forbidden-window table.
- **Forbidden-window applicability:** standard for any local cargo work (advisor's pre-plan-approval ad-hoc validation, Task 0 audit if it includes cargo probes); non-binding for impl-task throughput under Shape G.

---

## 6. Relationship to other v1 sub-phases

- **Predecessor (resolved scope-blocker):** v1-JM-d Task 2 is **parked**
  per DQ #61 until validate-agent's `bm-merge` lands. The plan §17
  Completion checklist explicitly carries: "validate-agent bm-merge
  unblocks JM-d Task 2 queueing — flag for advisor."
- **Coexists with:** `phase-v1-JM-d` may still exist with parked Task
  2 work; the branches are file-disjoint (validate-agent touches
  `.github/workflows/`, `.claude/agents/ci-watcher.md`, `.claude/rules/`,
  `.claude/PRPs/briefs/jm-d-impl-2.md` retrofit; JM-d's parked Task 2
  will eventually touch `crates/db_schema/src/source/governance/...`).
  Per `feedback_parallel_agents_one_worktree_per_agent.md`.
- **Predecessor sequencing rule (DQ #60 partially overridden by #61):**
  validate-agent ships → JM-d Task 2 retrofit + queue → JM-d remaining
  tasks → JM-d retro → JM-e. NOT JM-d-then-validate-agent.
- **Post-validate-agent v1 sub-phases:** v1-JM-e, v1-rep-tuning-r3,
  v1-SL-d, etc. all author plans under the new "DoD per workflow" §15
  shape and write impl-task briefs under the push-and-exit Layer G2
  contract.

---

## 7. Preflight guardrails inherited from prior phases

- **G1:** every workflow YAML is registered in `homeserver/library.yaml`
  in the same commit that ships it (per `feedback_library_add_after_shipping.md`).
  Catalog drift makes new files invisible to `/library list/search/sync`.
- **G2:** schema-additive enum-value introductions edit the canonical
  rule doc (`decision-queue.md`) in the SAME commit as the consumers
  (`impl-task.md`, `ci-watcher.md`, `advisor-orchestrator.md`)
  (per `feedback_update_rule_doc_with_schema_additive.md`). Splitting
  risks a window where the rule doc enumerates a kind no consumer can
  write/read.
- **G3:** every §15 DoD command is dry-run literally against current
  HEAD before plan commit (per `feedback_pre_phase_dod_smoke_test.md`
  + `feedback_plan_dod_dry_run_at_write.md`). For workflow-referenced
  §15 entries: dry-run shape is `yamllint <yaml-path>` exit 0 OR
  `gh workflow view --repo barrie-cork/lemmy --ref <phase-branch> <name>.yml`
  per DQ #67.
- **G4:** every §4 watchpoint cites a specific file/line/section
  anchor (per `feedback_advisor_watchpoint_specificity.md`). Abstract
  watchpoints are advisor-side rejection grounds.
- **G5:** `gh pr` invocations always include `--repo barrie-cork/lemmy`
  (per `.claude/rules/gh-pr-fork-target.md`). Default would target
  upstream `LemmyNet/lemmy`.
- **G6:** PR opens against `governance-v0` (NOT `main`). `main` is
  reserved for upstream-rebase work (per `.claude/rules/phase-branch.md`).
- **G7:** every `.claude/` edit commits per-edit, not per-session
  (per `feedback_commit_aggressively_in_shared_repos.md`). Sibling
  sessions can silently switch worktree branches and discard
  uncommitted edits.
- **G8:** ci-watcher subagent frontmatter is authored by mirroring an
  existing canonical agent file (per
  `feedback_read_canonical_before_writing_spec.md`). Glob + Read 1–2
  sibling agents (`bm-task.md` for narrow-tools mechanical pattern,
  `impl-task.md` for the impl-side reference) before writing.

---

## 8. Flow design

**Before (v0/v1 in-band Junior validation):**

```
[Junior worker slot]
   │
   ▼
impl-task subagent (Sonnet 4.6, medium)
   ├─ edit
   ├─ commit
   ├─ cargo check --workspace --features full   (~10–25 min, cgroup 8.6 GB / 10 GB cap)
   ├─ cargo clippy --workspace --features full --no-deps -- -D warnings  (~3 min)
   ├─ cargo test --no-run -p lemmy_server       (~5 min)
   ├─ on green: push, write completion DQ, exit
   └─ on red: write blocker DQ, exit (slot held throughout)
[slot free after 20–35 min]
   │
   ▼
[advisor poll] → next cohort
```

**After (Shape G out-of-band GitHub Actions validation):**

```
[Junior worker slot — impl phase]
   │
   ▼
impl-task subagent (Sonnet 4.6, medium)
   ├─ edit
   ├─ commit
   ├─ git push origin <phase-branch>             (~10s)
   ├─ gh run list --branch ... --limit 1 ...     (with retry/backoff for 1–2 min trigger lag)
   ├─ append validate-pending DQ entry
   ├─ commit + push DQ update                    (~5s)
   └─ exit
[slot free after 5–15 min, ALWAYS]
   │
   │           ┌────────────────────────────────────────┐
   │           │  GitHub Actions (off-box, ephemeral)   │
   │           │  cargo-validate-workspace.yml          │
   │           │    ├─ check + clippy + test --no-run   │
   │           │    └─ ~10–25 min cold / ~5–10 min hot  │
   │           │  cargo-validate-migration.yml          │
   │           │    └─ migration round-trip (when triggered)
   │           └────────────────────────────────────────┘
   ▼
[advisor poll] reads validate-pending → dispatches ci-watcher
   │
   ▼
[Junior worker slot — ci-watcher phase]
   │
   ▼
ci-watcher subagent (Haiku 4.5, low)
   ├─ gh auth status                             (pre-flight probe)
   ├─ gh run watch <id> --exit-status            (single long-poll, 1 API call)
   ├─ on success: write validate-result pass DQ, commit + push, exit
   ├─ on failure: gh run view <id> --log-failed, slice last ~200 lines,
   │              write validate-failed DQ with log slice, commit + push, exit
   └─ on 60-min timeout: write validate-failed timeout DQ, commit + push, exit
[slot free, ~10 sec model time even during 25-min watch (long-poll)]
   │
   ▼
[advisor poll] reads validate-result | validate-failed
   ├─ validate-result pass:    advance cohort (per old "Each impl-task complete" rule)
   └─ validate-failed:         §G4 classifier
       ├─ allowlist match (≤3 edits): queue fix-impl-task with narrow brief
       └─ else:                       catch-fire to user
```

The key invariant: the impl-task slot frees within 5–15 min ALWAYS,
regardless of cargo wall-clock. The ci-watcher slot is held only during
the `gh run watch` long-poll (~10 sec of model effort across 5–25 min
of wall-clock). Net Junior throughput: ~3–4× higher than v0/v1
in-band. Net model-token cost: ~1/15th per validation (Haiku 4.5
low-effort vs Sonnet 4.6 medium-effort × shorter task).

---

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit, grouped
by purpose:

- **Schema/type definitions (extending shape):**
  - `.claude/rules/decision-queue.md:147–209` (kind: blocker vs log vs clarify) — extends with three new kinds
  - `.claude/rules/decision-queue.md:462–495` (Subagents and attribution) — extends with `ci-watcher`
  - `.claude/rules/decision-queue.md:327–337` (Hard refusals — write-side) — extends with one bullet

- **Existing agent shapes (MIRROR refs §10):**
  - `.claude/agents/impl-task.md:53–60` (Per-task validation gate H2) — REWRITE BODY + RENAME
  - `.claude/agents/impl-task.md:130–139` (Hard refusals H2) — APPEND ONE LINE
  - `.claude/agents/bm-task.md` (full file) — narrow-tools mechanical-poller frontmatter pattern
  - `.claude/agents/planning.md` (full file) — frontmatter shape canonical
  - `.claude/agents/impl-task.md` (full file) — frontmatter shape canonical

- **Existing workflow patterns (MIRROR refs §10):**
  - `.github/workflows/cargo-test-e2e.yml` (full file, 75 lines) — cache strategy (lines 58–67), libpq install (53–56), Rust toolchain (47–51), submodule recursive checkout (37–45). DO NOT mirror trigger pattern (it uses pull_request; new YAMLs use push per DQ #66).

- **Stage-shape orchestration:**
  - `.claude/rules/advisor-orchestrator.md:190–205` (Stage-shape orchestration sub-section) — insert validate-stage transitions
  - `.claude/rules/advisor-orchestrator.md:63–102` (Forbidden execution windows) — extend with cargo-no-longer-local note
  - `.claude/rules/advisor-orchestrator.md:261–272` (Catch-fire procedures) — extend with workflow-timeout + classifier-miss triggers
  - `.claude/rules/advisor-orchestrator.md:217–243` (Cohort dispatch) — extend with parallel-validate-pending clause

- **Adjacent retrofit target:**
  - `.claude/PRPs/briefs/jm-d-impl-2.md` §5 only (single-brief bounded retrofit per DQ #61)

- **Plan template extension target:**
  - `.claude/PRPs/templates/plan.template.md:181–217` (§15 Validation commands DoD) — gain "DoD per workflow" sub-section forward-only
  - `.claude/PRPs/templates/plan.template.md:103–161` (§13 task composition guidance) — gain Shape-G push-and-exit composition note

- **Lessons (each gating a §13 task):**
  - `feedback_clarify_before_plan.md` — informational (clarify already ran)
  - `feedback_parallel_cohort_dispatch.md` — Task 1+2, Task 4+5 cohort eligibility
  - `feedback_update_rule_doc_with_schema_additive.md` — Task 3 atomicity
  - `feedback_library_add_after_shipping.md` — §17 + per-task library.yaml registration
  - `feedback_commit_aggressively_in_shared_repos.md` — one-commit-per-task wording
  - `feedback_pr_per_phase.md` — code via PR; this sub-phase qualifies
  - `feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md` — every §15 entry dry-run before commit
  - `feedback_pipes_mask_exit_codes.md` — exit-code discipline (cited in Task 2 §G3 watchpoint)
  - `feedback_read_canonical_before_writing_spec.md` — Task 2 ci-watcher.md authoring
  - `feedback_dogfood_slash_command_specs.md` — informational; this plan introduces no slash commands
  - `feedback_schema_changing_spec_retrofit_question.md` — Retrofit-scope §12 + §17 lines (already resolved by DQ #61)
  - `feedback_advisor_watchpoint_specificity.md` — every §4 watchpoint cites file/line
  - `feedback_brehon_subagent_model_effort_assignments.md` — Task 2 ci-watcher.md frontmatter pin (Haiku 4.5, low effort)
  - `feedback_principles_not_rules.md` — applied to Task 3 §G4 classifier wording (allowlist is principles + skip-conditions, not rigid rules)

---

## 10. Patterns to mirror

### 10.1 Workflow YAML cache strategy (`cargo-validate-workspace.yml`)

**SOURCE:** `.github/workflows/cargo-test-e2e.yml:58–67` (cache block).

**JM-validate-agent adaptation (Task 1; new file
`.github/workflows/cargo-validate-workspace.yml`):**

```yaml
- name: Cache cargo registry + target
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: cargo-validate-workspace-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}
    restore-keys: |
      cargo-validate-workspace-${{ runner.os }}-
```

Cache key namespace `cargo-validate-workspace-` is distinct from
`cargo-e2e-` (line 65 of cargo-test-e2e.yml). Cache thaws warm
across `Cargo.lock` updates via the prefix-only restore-keys per §4
watchpoint #3. `cargo-validate-migration.yml` uses
`cargo-validate-migration-` as its prefix (third namespace).

### 10.2 Workflow YAML libpq install + Rust toolchain

**SOURCE:** `.github/workflows/cargo-test-e2e.yml:47–56`.

**JM-validate-agent adaptation (Task 1; both new YAMLs):**

```yaml
- name: Install Rust toolchain
  uses: dtolnay/rust-toolchain@master
  with:
    # Pinned in rust-toolchain.toml; the action reads that file.
    toolchain: stable

- name: Install libpq
  run: |
    sudo apt-get update
    sudo apt-get install -y libpq-dev pkg-config
```

Identical to cargo-test-e2e.yml. No drift.

### 10.3 Workflow YAML submodule recursive checkout

**SOURCE:** `.github/workflows/cargo-test-e2e.yml:37–45`.

**JM-validate-agent adaptation (Task 1; both new YAMLs):**

```yaml
- name: Checkout
  uses: actions/checkout@v4
  with:
    fetch-depth: 1
    # crates/email/translations/ is a submodule pointing to
    # LemmyNet/lemmy-translations; lemmy_email/build.rs reads it
    # at compile time, so we must initialise it.
    submodules: recursive
```

Trigger pattern divergence: the new YAMLs check out `${{ github.sha
}}` (push event) rather than `${{ github.event.pull_request.head.sha
}}` (pull_request event). Default fetch behaviour suffices.

### 10.4 Workflow YAML trigger + concurrency (DQ #66 — divergent from cargo-test-e2e.yml)

**JM-validate-agent NEW shape (Task 1; both new YAMLs):**

```yaml
on:
  push:
    branches:
      - 'phase-v1-*'
      - 'junior/*'
    paths:
      - 'crates/**'
      - 'migrations/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
      - 'rust-toolchain.toml'
      - '.github/workflows/cargo-validate-workspace.yml'  # or cargo-validate-migration.yml

permissions:
  contents: read

concurrency:
  # Use github.ref so concurrent pushes to the same branch cancel-in-progress;
  # different branches run independently. Differs from cargo-test-e2e.yml
  # which uses pull_request.number.
  group: cargo-validate-workspace-${{ github.ref }}
  cancel-in-progress: true
```

`cargo-validate-migration.yml` uses
`group: cargo-validate-migration-${{ github.ref }}` and the same
trigger shape with `paths` narrowed to `migrations/**` only.

### 10.5 ci-watcher subagent frontmatter (mirror bm-task.md narrow-tools shape)

**SOURCE:** `.claude/agents/bm-task.md` frontmatter (mechanical
narrow-tools subagent, Haiku 4.5).

**JM-validate-agent NEW file (Task 2; `.claude/agents/ci-watcher.md`)
frontmatter:**

```markdown
---
description: |
  Polls a single GitHub Actions workflow run via `gh run watch
  --exit-status` and writes a `validate-result` or `validate-failed`
  DQ entry. Use when a Junior task description starts with
  `[role:ci-watcher]`. Reads the named ci-watcher brief, runs the
  poll loop, and exits. Pinned to Haiku 4.5 — mechanical polling, no
  judgment, no fixes. Never invokes cargo, never edits crates/, never
  applies clippy auto-fixes (those are advisor §G4 classifier work).
model: haiku
effort: low
tools:
  - Read
  - Edit
  - Write
  - Bash
---
```

The `tools` list is the narrowest viable: Read (DQ + brief), Edit/
Write (DQ updates), Bash (`gh auth status`, `gh run watch`,
`gh run view --log-failed`). NO Agent (no sub-dispatch), NO LSP (no
code reasoning), NO Glob/Grep (no file search needed for polling).

### 10.6 ci-watcher brief template (minimal three-field)

**SOURCE:** `.claude/PRPs/templates/plan.template.md` (existing
templates) for shape; ci-validation-design.md §9.3 for content.

**JM-validate-agent NEW file (Task 2;
`.claude/PRPs/templates/ci-watcher-brief.template.md`):**

```markdown
# ci-watcher brief — workflow run <run-id>

**Workflow run id:** <id>
**Branch:** <branch>
**Phase task:** <task-number>

## Action

Poll `gh run watch <id> --exit-status`. On exit:

- Exit code 0 (success): write `validate-result` DQ entry with
  `result: "pass"`, `from: "ci-watcher"`, `answered_by: "ci-watcher-self-resolved"`,
  commit + push, exit 0.

- Exit code non-zero (failure): run `gh run view <id> --log-failed`,
  slice last ~200 lines per failed job, write `validate-failed` DQ
  entry with `result: "fail"`, `log_slice`, `failed_jobs`, commit +
  push, exit 0.

- 60-minute timeout: write `validate-failed` DQ entry with
  `result: "timeout"`, commit + push, exit 0.

Pre-flight: `gh auth status`. If unauthorised, write `validate-failed`
DQ entry with `reason: "gh_unauth"` and exit.
```

The advisor auto-fills `<id>`, `<branch>`, `<task-number>` from the
`validate-pending` DQ entry it dispatches against. The brief
template body is the minimum-viable content; the full ci-watcher
contract lives in `.claude/agents/ci-watcher.md`.

### 10.7 impl-task push-and-exit body (rewrite of "Per-task validation gate" H2)

**SOURCE:** `.claude/agents/impl-task.md:53–60` (current H2 body).

**JM-validate-agent rewrite (Task 3; in same commit as schema-additive
DQ rule + advisor-orchestrator update):**

New title: **"Per-task validation gate (out-of-band on GH Actions)"**

New body:

```markdown
Validation runs out-of-band on GitHub Actions. After committing your
work, push to your worktree branch and exit. Do NOT run cargo locally.

After `git push`:

1. Capture the workflow_run id:
   ```bash
   gh run list --branch <your-branch> --limit 1 \
     --json databaseId --jq '.[0].databaseId'
   ```
   Retry with exponential backoff up to ~2 min if the run hasn't
   appeared yet (push-to-trigger lag is normal).

2. Append a `validate-pending` entry to `.claude/decision-queue.json`:
   ```json
   {
     "id": <next>,
     "from": "impl",
     "kind": "validate-pending",
     "timestamp": "<ISO 8601 UTC>",
     "workflow_run_id": <id>,
     "branch": "<your-branch>",
     "phase_task": <task-number>,
     "answer": null,
     "answered_by": null,
     "resolved_at": null
   }
   ```

3. Commit + push the DQ update.

4. Exit with success.

The impl-task slot frees as soon as the push lands. ci-watcher polls
the workflow asynchronously and writes the result back into the DQ.
The advisor reads `validate-result` (pass) or `validate-failed`
(fail/timeout) on its next polling tick.
```

### 10.8 Hard refusals append-line for impl-task.md

**SOURCE:** `.claude/agents/impl-task.md:130–139` (Hard refusals H2).

**JM-validate-agent append-line (Task 3, in same commit as 10.7):**

```markdown
- Never invoke cargo for build/lint/test — validation runs out-of-band
  on GH Actions per the validation gate above.
```

### 10.9 advisor-orchestrator.md Stage-shape insertion (the validate stage)

**SOURCE:** `.claude/rules/advisor-orchestrator.md:190–205`
(Stage-shape orchestration sub-section).

**JM-validate-agent insertion (Task 3; between current line 197 and
198 — new bullets replace nothing, just insert):**

```markdown
- **Each impl-task complete (under Shape G)** → impl-task wrote
  `validate-pending` post-push; queue `[role:ci-watcher]` task with
  brief filled from the DQ entry's `workflow_run_id` + `branch` +
  `phase_task`. ci-watcher runs ~10 sec model-time during the long-
  poll; the cargo work itself is GitHub-runner-side.

- **ci-watcher complete** → read DQ entry; if `validate-result` pass,
  advance per old "Each impl-task complete" rule (cohort check); if
  `validate-failed`, run §G4 classifier:
    - allowlist match (≤3 file edits + clippy auto-fix or missing
      import or deprecated API) → queue narrow fix-impl-task brief.
    - else → catch-fire to user with the log slice + failed jobs.
```

### 10.10 Plan-template §15 "DoD per workflow" forward-only shape

**SOURCE:** `.claude/PRPs/templates/plan.template.md:181–217` (current
prose §15 DoD shape).

**JM-validate-agent §15 extension (Task 4; appends sub-section
"15.X DoD per workflow (Shape G plans)"):**

```markdown
### 15.X DoD per workflow (Shape G plans — v1-JM-e onward)

For plans authored under Shape G push-and-exit Layer G2: §15 DoD
entries reference workflow files by path + expected `conclusion`
field rather than enumerating cargo commands. The advisor's
`/brehon-verify` cross-checks the latest workflow run on the phase
branch SHA against the listed conclusion before queueing `bm-merge`.

Shape:

- **DoD entry**: `<workflow-name>.yml` on `<phase-branch>` SHA `<sha>` → `conclusion: "success"`
- **Validation command**: `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json conclusion,databaseId --jq '.[0]'`
- **EXPECT**: `{"conclusion": "success", "databaseId": <id>}`

Pre-existing JM-d/JM-c/JM-b/JM-a plans stay under prose §15
(forward-only per `feedback_schema_changing_spec_retrofit_question.md`).
```

### 10.11 jm-d-impl-2.md §5 retrofit (single-brief bounded scope)

**SOURCE:** `.claude/PRPs/briefs/jm-d-impl-2.md` §5 (current inline
cargo body).

**JM-validate-agent retrofit (Task 5; commit-message subject
`docs(briefs): jm-d-impl-2 §5 retrofit — push-and-exit per Shape G`):**

Replace §5 body with the §10.7 push-and-exit pattern from this plan.
Keep the §5 heading "Validation"; only the body changes. No other
JM-d brief edited in this commit.

---

## 11. Files to change

Grouped by directory:

### .github/workflows/ (new)

- `.github/workflows/cargo-validate-workspace.yml` — workspace check + clippy + test --no-run on push to phase-v1-* and junior/* (Task 1)
- `.github/workflows/cargo-validate-migration.yml` — migration round-trip on push touching `migrations/**` (Task 1)

### .claude/agents/ (new + extended)

- `.claude/agents/ci-watcher.md` — fifth subagent role, Haiku 4.5, low effort, narrow tools (Task 2)
- `.claude/agents/impl-task.md` — rename + rewrite "Per-task validation gate" H2; append one line to "Hard refusals" (Task 3)

### .claude/PRPs/templates/ (new + extended)

- `.claude/PRPs/templates/ci-watcher-brief.template.md` — minimal three-field brief template (Task 2)
- `.claude/PRPs/templates/plan.template.md` — append "15.X DoD per workflow (Shape G plans — v1-JM-e onward)" sub-section + §13 task-composition guidance note (Task 4)

### .claude/rules/ (extended)

- `.claude/rules/decision-queue.md` — three new `kind` values, one new `from: "ci-watcher"`, polling-loop routing table extension, Hard refusals (write-side) extension, Subagents and attribution enumeration extension (Task 3 — atomic with impl-task.md + advisor-orchestrator.md per `feedback_update_rule_doc_with_schema_additive.md`)
- `.claude/rules/advisor-orchestrator.md` — Stage-shape orchestration insertion (validate stage), Forbidden-windows note (cargo no longer local for impl-task), Catch-fire procedures extension (workflow timeout + classifier miss), Cohort dispatch parallel-validate-pending clause (Task 3)

### .claude/PRPs/briefs/ (single bounded retrofit)

- `.claude/PRPs/briefs/jm-d-impl-2.md` — §5 only switches from inline cargo to push-and-exit per DQ #61 (Task 5). All other JM-d briefs untouched.

### Sibling repo (homeserver) — out-of-band but flagged in §17

- `homeserver/library.yaml` — register every new file (workflows + ci-watcher.md + ci-watcher-brief.template.md). Lives in homeserver, not brehon-fork. Per `feedback_library_add_after_shipping.md`. The plan §17 Completion checklist carries the line; the actual edit happens in the homeserver repo, not as part of the v1-validate-agent PR.
- `homeserver/CLAUDE.md` — one-line note that ci-watcher is a Junior subagent (not a daemon) and that workflow runs are GH-Actions-side. Same out-of-band.

---

## 12. NOT building in v1-validate-agent

- **Retrofitting JM-d briefs OTHER THAN jm-d-impl-2.md** — deferred indefinitely (forward-only-immune). Per DQ #61 + `feedback_schema_changing_spec_retrofit_question.md`. Pre-existing `jm-d-impl-1.md` (only other JM-d brief that exists at plan-write time) stays untouched.
- **JM-d Task 2 itself** — Task 2 is parked per DQ #61. The validate-agent plan does NOT queue Task 2; it only sets up the conditions (Shape G shipped) under which Task 2 can be queued by the advisor's polling loop AFTER validate-agent's `bm-merge`. §17 Completion checklist carries the flag.
- **Cross-provider model trials** (MiniMax, Gemini, etc.) — Brehon stays Anthropic-only. ci-watcher is Haiku 4.5.
- **BM verb changes** (`bm-cut`/`bm-pr`/`bm-merge`/`bm-poll-cr`/`bm-triage` semantics) — validate is a sibling stage in the orchestrator, not a BM verb.
- **Replacing the §15 DoD definition itself** — we change WHERE DoD runs (out-of-band on GH Actions vs in-band on Junior worker) and HOW it's expressed (workflow-referenced vs inline cargo commands), not WHAT it tests. Semantic content of DoD stays.
- **Self-hosted GitHub runners on the EliteDesk** — tempting for minutes-budget mitigation, but re-introduces resource contention this design solves. Out of scope unless private-repo minute limits become binding for >2 consecutive months. Per ci-validation-design.md §11.
- **Replacing `cargo-test-e2e.yml`** — it works; the new workflows supplement, don't replace. Per Shape G design §3 + §4.3.
- **Auto-retry of failed validation** — `max_retries: 0` discipline in junior config.yaml stays. validate-failed surfaces; advisor decides; no auto-retry. Workflow re-run via `gh run rerun <id>` is an advisor-side recipe in the §G4 classifier, not an auto-action.
- **PR-side `pull_request:` trigger** — DQ #66: push-only triggers. PR status checks come via branch protection rules, configured out-of-band in GitHub settings (deferred to retro for confirmation).
- **PMD ingest pipeline fix** — three brief-named lessons not surfaced by `memory_search_hybrid` (handover §"Phase outputs (synthesised)" #9). Surfaced for retro to flag, NOT a v1-validate-agent deliverable.
- **Investigation of PMD #109/#110/#111** — separate diagnostics (orphan cargo telemetry, Server Boss alert AND-gate). Independent from this design. The Shape G impl-task contract changes (push-and-exit) don't depend on resolving these.
- **`act` (local GitHub Actions runner)** — DQ #67 forbids prescribing `act` invocations. End-to-end validation is the "first test push" §13 sub-step.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** (per
`feedback_pr_per_phase.md` + `feedback_commit_aggressively_in_shared_repos.md`).
Each task header carries a `[P]` marker iff its IMPLEMENT files share
no path with any other `[P]`-marked task in the same cohort. Task 0 is
**always** non-`[P]` (barrier for verification). Task 6 (retro) is also
non-`[P]` (always last).

> **Cohort dispatch (advisor-side):** Task 1 + Task 2 form a cohort
> ([P]+[P], no shared paths — workflows in `.github/workflows/` vs
> agent in `.claude/agents/`). Task 3 is a barrier (atomic schema-
> additive across decision-queue.md + impl-task.md + advisor-orchestrator.md
> per `feedback_update_rule_doc_with_schema_additive.md`). Task 4 +
> Task 5 form a cohort ([P]+[P], no shared paths — template in
> `.claude/PRPs/templates/` vs brief in `.claude/PRPs/briefs/`).
> Tasks 4+5 BLOCKED-BY Task 3 (the new template §15 references the
> new `kind` values; the brief retrofit references the new validation-
> gate body in impl-task.md).

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for `v1-validate-agent`; confirm
branch is `phase-v1-validate-agent` (cut by `bm-cut` AFTER plan
approval); confirm prior phase's deliverables are intact on the base;
confirm `gh` CLI auth on the EliteDesk.

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate
ALL probes explicitly):**

```bash
# Probe 0 — branch verification
git branch --show-current
# EXPECT: phase-v1-validate-agent

# Probe 1 — gh CLI authentication
gh auth status
# EXPECT: "Logged in to github.com as barrie-cork"

# Probe 2 — gh repo access
gh repo view barrie-cork/lemmy --json name --jq '.name'
# EXPECT: "lemmy"

# Probe 3 — gh workflow view (primary YAML-parse for §15 DoD per DQ #67 OR clause)
gh workflow view --help 2>&1 | grep -- '--yaml'
# EXPECT: line containing "--yaml" (the View flag)
# Note: yamllint is the secondary alternative per DQ #67; if available,
# `yamllint --version` exit 0 also satisfies §15.1. yamllint is NOT
# required on the EliteDesk Junior worker; gh workflow view is the
# primary path.

# Probe 4 — gh workflow view command available (for §15 DoD)
gh workflow --help | head -1
# EXPECT: "View the summary of a workflow"

# Probe 5 — gh run watch command available + flag-honour negative test
gh run watch --help 2>&1 | grep -- '--exit-status'
# EXPECT: line containing "--exit-status"

# Probe 6 — concurrent-PR check (no other PR touches §11 files)
gh pr list --repo barrie-cork/lemmy --state open \
  --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | startswith(".github/workflows/cargo-validate-")) | {number, title, headRefName}'
# EXPECT: empty output

# Probe 7 — DQ pull (any pending blocker?)
python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', len(d.get('pending', [])))"
# EXPECT: pending: 0 (or only entries from this v1-validate-agent worktree)

# Probe 8 — handover assertion (key files don't exist yet)
test ! -e .github/workflows/cargo-validate-workspace.yml && echo "OK: workspace.yml absent"
test ! -e .github/workflows/cargo-validate-migration.yml && echo "OK: migration.yml absent"
test ! -e .claude/agents/ci-watcher.md && echo "OK: ci-watcher.md absent"
test ! -e .claude/PRPs/templates/ci-watcher-brief.template.md && echo "OK: ci-watcher-brief.template.md absent"
# EXPECT: 4 lines, each starting with "OK:"
```

**EXPECT block:**
- Probes 0–8 exit 0 (or print expected output)
- No probe surfaces a blocking DQ entry

**No commit at Task 0** — verification only.

---

### Task 1 [P]: Author cargo-validate-workspace.yml + cargo-validate-migration.yml

**ACTION:** Create two new workflow YAMLs that run cargo validation
on push to phase-v1-* and junior/* branches.

**IMPLEMENT (file 1 of 2):** in `.github/workflows/cargo-validate-workspace.yml`,
author a workflow with the §10.4 trigger + concurrency, §10.3 checkout
shape, §10.2 toolchain + libpq, §10.1 cache strategy, and three job
steps:

```yaml
- name: cargo check workspace + features full
  run: cargo check --workspace --features full --no-deps

- name: cargo clippy workspace + features full
  run: cargo clippy --workspace --features full --no-deps -- -D warnings

- name: cargo test compile (no-run)
  run: cargo test --no-run -p lemmy_server --test e2e
```

`runs-on: ubuntu-latest`. `timeout-minutes: 45`. Job name: `validate-workspace`.

**IMPLEMENT (file 2 of 2):** in `.github/workflows/cargo-validate-migration.yml`,
author a sibling workflow with `paths` narrowed to `migrations/**`
only (in the trigger), §10.1 cache strategy with key prefix
`cargo-validate-migration-`, §10.2 toolchain + libpq + Postgres
client install, §10.3 checkout, and one job step:

```yaml
- name: Migration round-trip
  run: bash scripts/brehon/migrate-roundtrip.sh
```

The exact migration-id to round-trip is determined at job time by
inspecting the diff vs `governance-v0`; the `migrate-roundtrip.sh`
script handles this (existing script per JM-d §10.6 / scheduled-tasks
pattern). `runs-on: ubuntu-latest`. `timeout-minutes: 30`.

**MIRROR:** `.github/workflows/cargo-test-e2e.yml:37–67` for cache +
toolchain + libpq + checkout shape; §10.1–§10.4 above for adaptations.

**GOTCHA:** the trigger pattern is `push:` with `branches: ['phase-v1-*',
'junior/*']` — DO NOT mirror `cargo-test-e2e.yml`'s `pull_request:`
trigger (DQ #66). Concurrency key uses `github.ref` (push event) not
`github.event.pull_request.number`. The cache-key prefix
`cargo-validate-workspace-` and `cargo-validate-migration-` MUST be
distinct from `cargo-e2e-` (cache namespace collision per §4
watchpoint #3).

**VALIDATE (story-checkpoint feeds §16a):**

```bash
# Primary YAML-parse via gh (per DQ #67 OR clause). Note: --yaml is REQUIRED
# when --ref is supplied to gh workflow view.
gh workflow view --repo barrie-cork/lemmy --ref phase-v1-validate-agent \
  --yaml cargo-validate-workspace.yml > /dev/null
echo "gh workflow view workspace exit: $?"
gh workflow view --repo barrie-cork/lemmy --ref phase-v1-validate-agent \
  --yaml cargo-validate-migration.yml > /dev/null
echo "gh workflow view migration exit: $?"
# EXPECT: both exit 0 (workflow YAMLs parse on the GitHub side after push)

# Secondary fallback if yamllint is installed locally:
# yamllint .github/workflows/cargo-validate-workspace.yml
# yamllint .github/workflows/cargo-validate-migration.yml
# EXPECT: both exit 0

# After commit + push to phase-v1-validate-agent:
sleep 90  # push-to-trigger lag
gh run list --repo barrie-cork/lemmy --branch phase-v1-validate-agent \
  --workflow cargo-validate-workspace.yml --limit 1 \
  --json status,conclusion,databaseId
# EXPECT: status field present (queued|in_progress|completed)
```

**Commit subject:** `feat(ci): cargo-validate workflows for phase-v1-* + junior/* push triggers`

**Library.yaml registration**: in same commit, append both YAMLs to
`homeserver/library.yaml` (sibling-repo edit; the impl session
performs this in the homeserver checkout, not on the brehon-fork
worktree — flagged in §17).

---

### Task 2 [P] (cohort with Task 1): Author ci-watcher.md + ci-watcher-brief.template.md

**ACTION:** Create the fifth Junior subagent role and its brief
template. Pin Haiku 4.5, low effort, narrow tools (Read, Edit, Write,
Bash). Empirically validate `gh run watch --exit-status` exit codes
on real workflow runs before finalising the body.

**IMPLEMENT (file 1 of 2):** in `.claude/agents/ci-watcher.md`,
author the subagent file. Frontmatter per §10.5. Body sections (in
order):

1. **Role + dispatch line** — single line: `[role:ci-watcher] poll workflow run <id> on <branch> for phase task <N>`
2. **Before you start (always)** — list 3 mirror sequence: read the
   referenced ci-watcher brief; verify `gh auth status`; confirm the
   `workflow_run_id` exists via a single `gh run view <id> --json status`
   call before starting `gh run watch`.
3. **Action sequence** — five steps:
   - a. `gh auth status` pre-flight; on fail, write `validate-failed
     reason: gh_unauth` DQ entry, commit + push, exit 0.
   - b. `gh run watch <id> --exit-status` (single long-poll). Capture
     exit code into `$status`.
   - c. Branch on `$status` per the empirically-observed table (see
     "Empirical exit-code table" below):
     - `0` → success path: write `validate-result` DQ entry with
       `result: "pass"`. Commit + push. Exit 0.
     - `non-zero (failure)` → failure path: `gh run view <id>
       --log-failed > /tmp/failed.log`; slice last ~200 lines per
       failed job using `gh run view <id> --json jobs --jq
       '.jobs[] | select(.conclusion == "failure") | .name'`; write
       `validate-failed` DQ entry with `result: "fail"`,
       `log_slice` (the sliced text), `failed_jobs` (array). Commit
       + push. Exit 0.
     - `60-min wall-clock exceeded` → write `validate-failed` DQ
       entry with `result: "timeout"`. Commit + push. Exit 0. (Note:
       `gh run watch` itself may not enforce 60-min cap; ci-watcher
       wraps it in a `timeout 3600 gh run watch ...` shell-side cap.)
4. **Hard refusals** — list:
   - Never invoke cargo (build, lint, test, anything).
   - Never edit code in `crates/`, `migrations/`, `tests/`, `docs/`,
     `.github/workflows/`, or any plan/PRD file.
   - Never apply clippy auto-fixes — those are advisor §G4 classifier
     work, queued as a fix-impl-task.
   - Never write `kind: "validate-pending"` (that's impl-task's role)
     or `kind: "blocker" | "log" | "clarify"` (those are other
     subagents' roles).
   - Never use `gh run rerun <id>` or any GitHub-side mutation —
     ci-watcher is read-only on workflow state.
   - Never write `answered_by: "advisor" | "user"` — only
     `"ci-watcher-self-resolved"` for self-resolution per
     `decision-queue.md` Attribution integrity.
5. **Empirical exit-code table** — placeholder block to be filled by
   the impl session AFTER §4 watchpoint #1's empirical probes run.
   Until those probes run, the subagent file MUST NOT ship; the impl
   session captures probe outputs in
   `.claude/PRPs/debug/v1-validate-agent-task2-gh-run-watch-probes.log`
   and pastes the resulting (exit_code → conclusion → action) table
   into the subagent body before commit.
6. **Expected output shape** — one paragraph: success exits 0 with
   one new resolved DQ entry of kind `validate-result`; failure
   exits 0 with one new pending DQ entry of kind `validate-failed`.
   Junior worktree finalize pushes the commit.

**IMPLEMENT (file 2 of 2):** in
`.claude/PRPs/templates/ci-watcher-brief.template.md`, author per
§10.6.

**MIRROR:** `.claude/agents/bm-task.md` for narrow-tools mechanical
shape; `.claude/agents/impl-task.md` and `.claude/agents/planning.md`
for frontmatter conventions.

**GOTCHA:** the empirical probes (§4 watchpoint #1) are gating —
ci-watcher.md MUST NOT ship without the exit-code table filled in.
If the impl session can't get a real failed run to probe (no failure
mode reproducible at plan-write time), fall back to `gh run view <id>
--json conclusion --jq '.conclusion'` post-watch to get the
authoritative answer; the exit-code table can list "if exit code 1,
re-confirm via gh run view to disambiguate" as a defensive fallback.

**VALIDATE (story-checkpoint feeds §16a):**

```bash
# Frontmatter shape sanity
head -20 .claude/agents/ci-watcher.md | grep -E '^(model|effort):'
# EXPECT: "model: haiku" + "effort: low"

# Empirical probes ran (gating)
test -e .claude/PRPs/debug/v1-validate-agent-task2-gh-run-watch-probes.log && echo "OK: probes log present"
# EXPECT: "OK: probes log present"

# Exit-code table is filled (no placeholder text)
grep -c 'TBD\|<fill at impl time>' .claude/agents/ci-watcher.md
# EXPECT: 0
```

**Commit subject:** `feat(agents): ci-watcher subagent + brief template (Haiku 4.5, low)`

**Library.yaml registration**: in same commit, append both files to
`homeserver/library.yaml`.

---

### Task 3 (non-[P], barrier): Schema additive — atomic update of decision-queue.md + impl-task.md + advisor-orchestrator.md

**ACTION:** Add three new `kind` values + one new `from` value to
`decision-queue.md`; update `impl-task.md` with the push-and-exit
contract (rename + body rewrite + Hard-refusals append-line); insert
the validate stage into `advisor-orchestrator.md`. ALL THREE FILES
COMMIT TOGETHER per `feedback_update_rule_doc_with_schema_additive.md`
— splitting risks a window where the rule doc enumerates a kind no
consumer can write/read.

**IMPLEMENT (file 1 of 3):** in `.claude/rules/decision-queue.md`:

- At line 147 ("kind: blocker vs log vs clarify"): rename sub-section
  to "kind: blocker vs log vs clarify vs validate-pending vs validate-result vs validate-failed".
- After the existing `clarify` description (~line 184): insert three
  new bullets enumerating the three new kinds, their `from` value,
  their fields (`workflow_run_id`, `branch`, `phase_task` for pending;
  `result: "pass"` for result; `result: "fail" | "timeout"`,
  `log_slice`, `failed_jobs` for failed), and their pending-vs-resolved
  semantics.
- In the polling-loop routing per kind sub-section (~lines 192–208):
  add three new (kind, status) routes per §4 watchpoint #5.
- In Hard refusals (write-side) sub-section (~lines 327–337): append
  one bullet: "NEVER write `kind: \"validate-result\" | \"validate-failed\"` from a non-ci-watcher session."
- In Subagents and attribution sub-section (line 462): add
  `ci-watcher` as the fifth subagent. Document its `from: "ci-watcher"`
  rule, its `kind: "validate-result" | "validate-failed"` write rule,
  its `answered_by: "ci-watcher-self-resolved"` self-resolve rule.
  Document that impl-task continues writing as `from: "impl"` even
  when carrying `kind: "validate-pending"` (DQ #63).
- Per DQ #63: do NOT rename `impl` to `impl-task`; do NOT rewrite
  historical entries.

**IMPLEMENT (file 2 of 3):** in `.claude/agents/impl-task.md`:

- At line 53 ("Per-task validation gate"): rename H2 to "Per-task
  validation gate (out-of-band on GH Actions)". Replace body with
  §10.7.
- At line 130 ("Hard refusals"): append §10.8 line.
- Audit any cross-file links to the old anchor:
  ```bash
  rg -n 'per-task-validation-gate' .claude/
  ```
  Update any hits to the new anchor in this commit.

**IMPLEMENT (file 3 of 3):** in `.claude/rules/advisor-orchestrator.md`:

- Between lines 197 and 198 (Stage-shape orchestration): insert §10.9
  (the validate-stage transitions block).
- In Forbidden execution windows sub-section (lines 63–102): add a
  one-line note: "Under Shape G (v1-validate-agent onward), cargo no
  longer runs locally for impl-task throughput; ad-hoc local cargo
  validation by the advisor pre-plan-approval still respects these
  windows."
- In Catch-fire procedures sub-section (lines 261–272): append two
  triggers:
  - "Workflow run exceeds 60-min ci-watcher cap → ci-watcher writes
    `validate-failed: timeout`; surface to user."
  - "ci-watcher's `gh run watch --exit-status` returns an exit code
    not enumerated in the empirical exit-code table → surface to
    user as classifier-miss."
- In Cohort dispatch sub-section (lines 217–243): append a clause:
  "Under Shape G, cohort members can each be in `validate-pending`
  simultaneously (each has its own workflow run). Advance cohort only
  when ALL members reach `validate-result: pass`. A single
  `validate-failed` in the cohort blocks advancement and triggers §G4
  classifier per the validate-stage Stage-shape rule above."
- New sub-section "§G4 classifier" (insert after Catch-fire
  procedures): allowlist for trivial auto-fix-impl-tasks per §4
  watchpoint #7. Initial allowlist: `clippy::doc_lazy_continuation`,
  missing imports (`error[E0432]`), deprecated APIs (`warning: use of
  deprecated`). ≤3 file edits cap. Anything else surfaces.

**MIRROR:** §10.7, §10.8, §10.9 above.

**GOTCHA:** The three files must commit ATOMICALLY. If you split,
the in-between commit has a rule doc enumerating a kind that no
consumer can write/read — schema drift. Run all three Edits, then
`git add .claude/rules/decision-queue.md .claude/agents/impl-task.md .claude/rules/advisor-orchestrator.md`,
then commit. If a `git status` reveals any untracked file from
`feedback_commit_aggressively_in_shared_repos.md` discipline, address
it in a separate prior commit.

**VALIDATE (story-checkpoint feeds §16a):**

```bash
# All three files modified in one commit
git log -1 --name-only --pretty="" \
  | grep -E '^\.claude/(rules/(decision-queue|advisor-orchestrator)\.md|agents/impl-task\.md)$' | wc -l
# EXPECT: 3

# decision-queue.md enumerates the three new kinds
grep -c -E '"validate-(pending|result|failed)"' .claude/rules/decision-queue.md
# EXPECT: ≥6 (each kind cited at least twice — definition + routing)

# impl-task.md H2 renamed
grep -c '^## Per-task validation gate (out-of-band on GH Actions)$' .claude/agents/impl-task.md
# EXPECT: 1

# advisor-orchestrator.md gained validate-stage transitions
grep -c -E 'validate-pending|ci-watcher complete' .claude/rules/advisor-orchestrator.md
# EXPECT: ≥2

# No stale anchor links to old impl-task.md section
rg -c '#per-task-validation-gate"\|#per-task-validation-gate\)' .claude/ \
  | awk -F: '{sum+=$2} END {print sum}'
# EXPECT: 0 (old anchor links updated)
```

**Commit subject:** `feat(rules,agents): schema-additive validate-* kinds + impl-task push-and-exit + advisor validate-stage`

**Library.yaml registration**: not needed for this task (existing
files extended, not new).

---

### Task 4 [P] (post-Task-3): Plan-template §15 + §13 update

**ACTION:** Append the "DoD per workflow" §15 sub-section and the
§13 task-composition guidance note to plan.template.md. Forward-only;
pre-existing plans stay under prose §15.

**IMPLEMENT:** in `.claude/PRPs/templates/plan.template.md`:

- Append §10.10 (the new "15.X DoD per workflow (Shape G plans —
  v1-JM-e onward)" sub-section) after the existing §15.5 (Cross-cutting
  verification). Number the new sub-section §15.X (the next available
  integer; current shipped JM-d uses §15.1–§15.7, so the new
  template-level addition lands at the end of §15).
- In §13 (Step-by-step tasks intro paragraph, lines 103–107): append
  one note after the cohort-dispatch paragraph: "Plans authored under
  Shape G (Layer G2 push-and-exit, v1-JM-e onward) compose tasks as
  edit + commit + push (validation runs async on GH Actions); cargo
  invocations belong to the workflow YAML, not to the §13 task body.
  Pre-existing plans (v1-JM-d and earlier) keep their inline cargo
  task body — forward-only-immune per
  `feedback_schema_changing_spec_retrofit_question.md`."

**MIRROR:** §10.10 above.

**GOTCHA:** the new §15.X sub-section MUST NOT renumber existing
§15.1–§15.5; appending only. Pre-existing plan files reference
§15.1, §15.2, etc. by number — renumbering breaks them.

**VALIDATE (story-checkpoint feeds §16a):**

```bash
# New sub-section appended
grep -c '15\.X DoD per workflow' .claude/PRPs/templates/plan.template.md
# EXPECT: 1

# §13 intro gained the Shape G note
grep -c 'Shape G (Layer G2 push-and-exit' .claude/PRPs/templates/plan.template.md
# EXPECT: 1

# Existing §15.1–§15.5 unchanged
grep -E '^### 15\.[1-5] ' .claude/PRPs/templates/plan.template.md | wc -l
# EXPECT: 5
```

**Commit subject:** `docs(template): plan.template §15 DoD per workflow + §13 Shape G composition note`

**Library.yaml registration**: not needed (existing file extended).

---

### Task 5 [P] (post-Task-3, cohort with Task 4): jm-d-impl-2.md §5 retrofit

**ACTION:** Single-brief bounded retrofit per DQ #61. Replace
`jm-d-impl-2.md` §5 "Validation" body with the §10.7 push-and-exit
pattern. No other JM-d brief edited.

**IMPLEMENT:** in `.claude/PRPs/briefs/jm-d-impl-2.md`:

- Locate §5 (whatever heading the file uses — e.g. "## 5.
  Constraints" or similar; check the file at impl time). Look for
  the "Validation" sub-section under §5.
- Replace the body of the Validation sub-section with §10.7 (the
  push-and-exit pattern).
- Add a one-line note: "Retrofitted from inline cargo to push-and-
  exit per DQ #61 + v1-validate-agent.plan.md Task 5
  (`feat(plan): v1-validate-agent`)."
- Do NOT edit any other section of `jm-d-impl-2.md`.
- Do NOT edit `jm-d-impl-1.md` or any other JM-d brief.

**MIRROR:** §10.7.

**GOTCHA:** `jm-d-impl-2.md` corresponds to JM-d Task 2, which is
PARKED per DQ #61. The retrofit makes the brief Shape-G-compliant
ahead of when it'll be queued; the parked status doesn't change. The
brief's other sections (§1 Role + dispatch, §2 Scope, §3 Required
reading, §4 Constraints) reference cargo invocations or pre-Shape-G
patterns — DO NOT cascade edits into those sections in this commit;
the retrofit boundary is §5 only (per DQ #61's "single-brief bounded
retrofit (jm-d-impl-2.md §5 only)" wording).

**VALIDATE (story-checkpoint feeds §16a):**

```bash
# §5 body changed
grep -c 'push-and-exit\|out-of-band on GH Actions' .claude/PRPs/briefs/jm-d-impl-2.md
# EXPECT: ≥1

# Other JM-d briefs untouched in this commit
git log -1 --name-only --pretty="" \
  | grep -E '^\.claude/PRPs/briefs/jm-d-impl-' | grep -v 'jm-d-impl-2\.md' | wc -l
# EXPECT: 0
```

**Commit subject:** `docs(briefs): jm-d-impl-2 §5 retrofit — push-and-exit per Shape G`

**Library.yaml registration**: not needed (existing file extended).

---

### Task 6 (non-[P], retro): Author retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`

**Goal:** author retro at `.claude/PRPs/reports/v1-validate-agent-retro.md`
with one H2 per role (Advisor / Planning / Impl / BM) plus signals,
lessons, and per-task complexity scores. Promote any new lessons to
`.claude/lessons/feedback_*.md` in the same retro commit (per
`feedback_one_system_memory_in_repo.md`).

**IMPLEMENT:** at `.claude/PRPs/reports/v1-validate-agent-retro.md`:

1. **§1 Summary** — 1–2 paragraphs: what shipped, what didn't, headline
   wins/misses.
2. **§2 Per-role signals** — one H2 each:
   - Advisor: stage-shape transitions worked? validate-stage caught
     the right things? §G4 classifier allowlist correct?
   - Planning: this plan's accuracy on §13 task-count + cohort layout?
     Watchpoint accuracy?
   - Impl: any unexpected cargo run in spite of Hard refusals append?
     Any DQ kind drift?
   - BM: bm-cut → bm-pr → CodeRabbit → bm-merge cleanliness?
3. **§3 Per-task complexity score** — table per
   `feedback_retro_task_complexity_score.md`: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`
   for each Task 1..6. Aggregate: total files, total commits, sum
   wall-clock, max silence.
4. **§4 Lessons promoted to `.claude/lessons/`** — list any new
   `feedback_*.md` files added in this commit (e.g. lessons learned
   from the empirical `gh run watch` probes if surprising; lessons
   on §G4 classifier tuning if the allowlist needed adjustment).
5. **§5 Watch-items for next sub-phase (v1-JM-e first under Shape G)**
   — explicit watch-list: PMD ingest pipeline (handover §"Phase
   outputs (synthesised)" #9 — three brief-named lessons not
   surfaced); ci-watcher Haiku-4.5 effective at low-effort; any
   minutes-budget signals against the 80–200/month estimate; PR-side
   branch protection rules confirmed (DQ #66 deferred this).
6. **§6 Confidence score** — N/10 per `v1-jury-mechanics-d.plan.md`
   §20 precedent (optional but recommended for v1 sub-phases).

**VALIDATE:** retro file exists; per-role H2 count = 4; per-task
complexity score table has 6 rows.

**Commit subject:** `docs(retro): v1-validate-agent retro + lessons promoted`

---

## 14. Testing strategy

This sub-phase ships infrastructure (workflow YAMLs + subagent +
schema-additive + retrofit), not Rust code. Testing strategy is
correspondingly different from JM-d:

- **YAML structural validation:** `yamllint` exit 0 on each new
  workflow file. Per §15.1.
- **Workflow trigger smoke (end-to-end):** push to `phase-v1-validate-agent`
  triggers `cargo-validate-workspace.yml` automatically. Per §15.2.
- **Workflow run green:** the first push's run reaches
  `conclusion: "success"`. Per §15.3.
- **ci-watcher contract validation:** the `gh run watch --exit-status`
  empirical probes (§4 watchpoint #1) capture authoritative exit
  codes; ci-watcher.md classifier table reflects them. Per §15.4.
- **Schema-additive consistency:** `decision-queue.md` enumerates the
  three new kinds; `impl-task.md` writes `validate-pending`;
  `ci-watcher.md` writes `validate-result` / `validate-failed`;
  `advisor-orchestrator.md` reads all three at the stage-shape map.
  Per §15.5 cross-cutting verification.
- **Retrofit boundary:** `jm-d-impl-2.md` §5 body changed; no other
  brief touched. Per Task 5 VALIDATE block.
- **Story-grain checkpoints:** §16a Stories block names six end-to-end
  testable behaviours; advisor `/brehon-verify` validates each before
  bm-merge.

NOT in this sub-phase: cargo unit tests (no Rust code authored), e2e
test execution (covered by `cargo-test-e2e.yml` post-merge as part of
v1-JM-e and onward), migration round-trip (no migrations authored —
the new `cargo-validate-migration.yml` workflow runs `migrate-roundtrip.sh`
when triggered, but this sub-phase doesn't trigger it because no
migrations are authored).

---

## 15. Validation commands (DoD)

> **Planner-side discipline:** every command in this section MUST be
> dry-run by the advisor against current HEAD before plan approval.
> For workflow-referenced DoD entries, dry-run is `yamllint <yaml-path>`
> exit 0 OR `gh workflow view --repo barrie-cork/lemmy --ref <phase-branch> <name>.yml`
> per DQ #67 (the workflows don't exist at plan-write time, so
> yamllint runs against not-yet-committed paths during dry-run; the
> command itself ("yamllint --version" + "test -e <yaml-path>")
> establishes the tool is available and the path is one we'll create).

### 15.1 Static analysis (per workflow YAML)

```bash
# Primary path per DQ #67 OR clause: gh workflow view --yaml.
# (--yaml is REQUIRED when --ref is supplied.)
gh workflow view --repo barrie-cork/lemmy --ref phase-v1-validate-agent \
  --yaml cargo-validate-workspace.yml > /dev/null
echo "gh workflow view workspace exit: $?"
gh workflow view --repo barrie-cork/lemmy --ref phase-v1-validate-agent \
  --yaml cargo-validate-migration.yml > /dev/null
echo "gh workflow view migration exit: $?"
# EXPECT: both exit 0

# Secondary fallback (if yamllint is installed locally — not required):
# yamllint .github/workflows/cargo-validate-workspace.yml
# yamllint .github/workflows/cargo-validate-migration.yml
# EXPECT: both exit 0
```

Pre-commit dry-run (planning phase): `gh workflow view --help | grep -- '--yaml'`
confirms the flag exists; the workflow paths don't exist yet on the
phase branch (only after Task 1 commit + push). yamllint is NOT
installed on the laptop; gh CLI is. The EliteDesk Junior worker has
gh authenticated as `barrie-cork` (per ci-validation-design.md §8.5),
so this command works in both contexts.

### 15.2 Workflow trigger smoke (first push)

```bash
# After Task 1's commit + push lands on phase-v1-validate-agent:
sleep 90  # push-to-trigger lag (1–2 min normal)
gh run list --repo barrie-cork/lemmy --branch phase-v1-validate-agent \
  --workflow cargo-validate-workspace.yml --limit 1 \
  --json status,conclusion,databaseId
# EXPECT: status field = "queued" | "in_progress" | "completed"

# Confirm trigger pattern is push (not pull_request):
gh run list --repo barrie-cork/lemmy --branch phase-v1-validate-agent \
  --workflow cargo-validate-workspace.yml --limit 1 \
  --json event --jq '.[0].event'
# EXPECT: "push"
```

### 15.3 Workflow run green (cargo validation passes on phase branch)

```bash
# After workflow completes:
gh run list --repo barrie-cork/lemmy --branch phase-v1-validate-agent \
  --workflow cargo-validate-workspace.yml --limit 1 \
  --json conclusion --jq '.[0].conclusion'
# EXPECT: "success"

gh run list --repo barrie-cork/lemmy --branch phase-v1-validate-agent \
  --workflow cargo-validate-migration.yml --limit 1 \
  --json conclusion --jq '.[0].conclusion'
# EXPECT: "success" OR null (workflow not triggered if no migrations/** changed in this push)
```

### 15.4 ci-watcher empirical exit-code probes (Task 2 gating)

```bash
# Captured in .claude/PRPs/debug/v1-validate-agent-task2-gh-run-watch-probes.log
# during Task 2; verified at commit time:
test -e .claude/PRPs/debug/v1-validate-agent-task2-gh-run-watch-probes.log && echo OK
# EXPECT: OK

# Verify the exit-code table in ci-watcher.md is filled:
grep -c 'TBD\|<fill at impl time>' .claude/agents/ci-watcher.md
# EXPECT: 0
```

### 15.5 Schema-additive consistency (Task 3 gating)

```bash
# decision-queue.md enumerates three new kinds:
grep -c -E '"validate-(pending|result|failed)"' .claude/rules/decision-queue.md
# EXPECT: ≥6

# impl-task.md renamed H2:
grep -c '^## Per-task validation gate (out-of-band on GH Actions)$' .claude/agents/impl-task.md
# EXPECT: 1

# impl-task.md Hard-refusals append:
grep -c 'Never invoke cargo for build/lint/test' .claude/agents/impl-task.md
# EXPECT: 1

# advisor-orchestrator.md validate-stage:
grep -c -E '(validate-pending|ci-watcher complete|§G4 classifier)' .claude/rules/advisor-orchestrator.md
# EXPECT: ≥3
```

### 15.6 Cross-cutting verification (Task 6 — final)

- [ ] `homeserver/library.yaml` registers all four new artefacts (cargo-validate-workspace.yml, cargo-validate-migration.yml, ci-watcher.md, ci-watcher-brief.template.md)
- [ ] `homeserver/CLAUDE.md` has the one-line ci-watcher-is-subagent note
- [ ] No edits to files outside §11 list (verify via `git diff --name-only governance-v0..HEAD`)
- [ ] No new ENTRY_KIND consts added (this is an infra phase, not a Rust phase) — `git diff governance-v0..HEAD -- crates/` returns empty
- [ ] `decision-queue.md` schema-additive change: zero historical entries rewritten — `git log -p governance-v0..HEAD -- .claude/decision-queue.json` shows only NEW entries appended (or empty if none added during this sub-phase)
- [ ] `gh run watch --exit-status` empirical probes captured + ci-watcher.md exit-code table filled
- [ ] `jm-d-impl-2.md` §5 retrofit committed; no other JM-d brief modified
- [ ] R5: Task 0 enumerated all 9 probes (0–8)

### 15.7 Manual validation (advisor-side, optional sanity)

After Task 1 + Task 2 + Task 3 commits land on the phase branch:

```bash
# Simulate an impl-task push-and-exit:
echo "test commit" > /tmp/dummy.txt
# (don't actually commit — just verify the push-list-trigger sequence works)

# Verify ci-watcher.md frontmatter:
head -10 .claude/agents/ci-watcher.md
# EXPECT: model: haiku, effort: low

# Confirm advisor-orchestrator validate-stage is reachable:
sed -n '/^## Stage-shape orchestration/,/^## /p' .claude/rules/advisor-orchestrator.md | grep -c 'ci-watcher complete'
# EXPECT: 1
```

---

## 16. Acceptance criteria

- [ ] All 7 tasks (Task 0 audit + Tasks 1–5 deliverables + Task 6 retro) completed in dependency order
- [ ] §15.1 (yamllint) exit 0 on both new workflow YAMLs
- [ ] §15.2 (workflow trigger smoke) — both new workflows fire on push to `phase-v1-validate-agent`; trigger event is `push` not `pull_request`
- [ ] §15.3 (workflow run green) — `cargo-validate-workspace.yml` completes with `conclusion: "success"` on `phase-v1-validate-agent`; `cargo-validate-migration.yml` either succeeds or is not triggered (no migrations in this sub-phase)
- [ ] §15.4 (ci-watcher empirical exit codes) — probes log captured; ci-watcher.md exit-code table filled (no TBDs)
- [ ] §15.5 (schema-additive consistency) — three new `kind` values enumerated in `decision-queue.md`; `impl-task.md` H2 renamed + Hard-refusals append; `advisor-orchestrator.md` validate-stage transitions inserted
- [ ] §15.6 (cross-cutting verification) — all 8 boxes ticked
- [ ] §16a stories — all 6 stories `[done]`
- [ ] No edits to files outside §11 list
- [ ] No edits to `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/plans/` (other than this plan), or `.claude/PRPs/prds/`
- [ ] Retro committed per §13 Task 6
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] DQ #61 unblocking confirmed: validate-agent's bm-merge clears JM-d Task 2 queueing path

---

## 16a. Stories (independently-testable behaviour units)

> Story 5 is gated — the impl session may stub it (story marked `[deferred-to-retro]`) if no real failure surfaces during the test push. Story 6 ships only after `bm-merge`.

### Story 1: impl-task pushes to a phase-v1-* branch and `cargo-validate-workspace.yml` triggers automatically

- **Composing tasks:** Task 1 (workflows authored)
- **Checkpoint command:**
  ```bash
  sleep 90
  gh run list --repo barrie-cork/lemmy --branch phase-v1-validate-agent \
    --workflow cargo-validate-workspace.yml --limit 1 --json event,status \
    --jq '.[0]'
  ```
- **Expected output:** `{"event": "push", "status": "queued" | "in_progress" | "completed"}`
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `.github/workflows/cargo-validate-workspace.yml` exists and is non-empty
  - `.github/workflows/cargo-validate-migration.yml` exists and is non-empty
  - First post-Task-1 push to `phase-v1-validate-agent` is a real GitHub Actions run

### Story 2: ci-watcher reads a `validate-pending` DQ entry and polls a workflow run via `gh run watch` to completion

- **Composing tasks:** Tasks 1 + 2 + 3 (workflows + ci-watcher + schema-additive)
- **Checkpoint command:** Manual relay-test — write a fake `validate-pending` DQ entry pointing at the Task 1 workflow run; dispatch ci-watcher; observe DQ updates.
  ```bash
  # Pseudo-test: ci-watcher subagent runs against a real workflow run
  # The advisor relays the test outcome in the §16a verify report.
  ```
- **Expected output:** ci-watcher exits 0; one new resolved DQ entry of kind `validate-result` appears in `.claude/decision-queue.json`.
- **Brief-Scope outputs to verify:**
  - `.claude/agents/ci-watcher.md` exists with the §10.5 frontmatter
  - `.claude/PRPs/templates/ci-watcher-brief.template.md` exists
  - `.claude/rules/decision-queue.md` enumerates `validate-pending`, `validate-result`, `validate-failed`

### Story 3: ci-watcher classifies a `success` workflow as `validate-result: pass` and writes the DQ entry

- **Composing tasks:** Tasks 2 + 3 (ci-watcher + schema-additive)
- **Checkpoint command:** Inspect a real validate-result entry written by ci-watcher.
  ```bash
  python3 -c "
  import json
  d = json.load(open('.claude/decision-queue.json'))
  results = [e for e in d.get('resolved', []) if e.get('kind') == 'validate-result']
  print(f'validate-result count: {len(results)}')
  if results:
      print(f'last entry: {results[-1]}')
  "
  ```
- **Expected output:** at least one `validate-result` entry with `result: "pass"` and `from: "ci-watcher"`.
- **Brief-Scope outputs to verify:**
  - `.claude/agents/ci-watcher.md` exit-code table reflects the empirical probes (no TBD placeholders)
  - The `validate-result` entry's `answered_by` is `"ci-watcher-self-resolved"` per `decision-queue.md` Attribution integrity

### Story 4: ci-watcher classifies a `failure` workflow as `validate-failed` with the failing log slice attached

- **Composing tasks:** Tasks 2 + 3 (ci-watcher + schema-additive)
- **Checkpoint command:** Force a failing workflow run (intentional clippy lint added then reverted), observe ci-watcher's log slice + failed_jobs fields.
  ```bash
  python3 -c "
  import json
  d = json.load(open('.claude/decision-queue.json'))
  fails = [e for e in d.get('pending', []) if e.get('kind') == 'validate-failed']
  print(f'validate-failed count: {len(fails)}')
  if fails:
      e = fails[-1]
      print(f'has log_slice: {bool(e.get(\"log_slice\"))}')
      print(f'failed_jobs: {e.get(\"failed_jobs\")}')
  "
  ```
- **Expected output:** at least one `validate-failed` entry with non-empty `log_slice` and a `failed_jobs` array.
- **Brief-Scope outputs to verify:**
  - The `validate-failed` entry's structure matches `decision-queue.md` schema enumeration
  - The log slice is ≤200 lines per failed job (per ci-validation-design.md §G3)

### Story 5: advisor §G4 classifier auto-queues a fix-impl-task for a known clippy lint

- **Composing tasks:** Task 3 (§G4 classifier insertion in advisor-orchestrator.md)
- **Checkpoint command:** Manual relay-test — surface a `validate-failed` DQ entry that matches the allowlist (e.g. `clippy::doc_lazy_continuation`) and confirm the advisor queues a fix-impl-task.
  ```bash
  # The advisor's polling loop output relays the classification + dispatch
  # decision in the §16a verify report.
  ```
- **Expected output:** advisor's polling loop dispatches a fix-impl-task with a narrow brief targeting ≤3 file edits.
- **Brief-Scope outputs to verify:**
  - `.claude/rules/advisor-orchestrator.md` enumerates the allowlist (`clippy::doc_lazy_continuation`, missing imports, deprecated APIs) explicitly
  - The fix-impl-task brief at `.claude/PRPs/briefs/v1-validate-agent-fix-impl-<n>.md` is well-formed if the test fires
- **Status if not exercised:** `[deferred-to-retro]` — Story 5 is gated; if no real failure surfaces during the test push, the impl session marks the story `[deferred-to-retro]` in `.claude/PRPs/reports/v1-validate-agent-verify.md` and the retro section §5 carries a watch-item to validate at v1-JM-e.

### Story 6: validate-agent's bm-merge unblocks JM-d Task 2 (single-brief retrofit + queue)

- **Composing tasks:** Task 5 (jm-d-impl-2.md §5 retrofit) + post-merge state
- **Checkpoint command:** After bm-merge, advisor confirms JM-d Task 2 is queueable.
  ```bash
  # Pseudo-test:
  grep -c 'push-and-exit\|out-of-band on GH Actions' .claude/PRPs/briefs/jm-d-impl-2.md
  # EXPECT: ≥1
  python3 -c "
  import json
  d = json.load(open('.claude/decision-queue.json'))
  d61 = next((e for e in d.get('resolved', []) if e.get('id') == 61), None)
  print(f'DQ #61 resolved: {d61 is not None}')
  print(f'DQ #61 answer relevant: {\"validate-agent\" in d61.get(\"answer\", \"\") if d61 else False}')
  "
  ```
- **Expected output:** `jm-d-impl-2.md` §5 contains the push-and-exit body; DQ #61 is resolved and refers to validate-agent.
- **Brief-Scope outputs to verify:**
  - `.claude/PRPs/briefs/jm-d-impl-2.md` §5 body changed
  - No other `jm-d-impl-*.md` brief modified
  - PR for `phase-v1-validate-agent` merged to `governance-v0`

> **Verification mapping:** the advisor's `/brehon-verify` step iterates this section, runs each Story's Checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantoms (task complete but output absent or empty) trigger the catch-fire procedure in advisor-orchestrator.md.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (probes 0–8 confirmed)
- [ ] Task 1 (workflow YAMLs) committed; first push to `phase-v1-validate-agent` triggered both workflows; `cargo-validate-workspace.yml` ran green
- [ ] Task 2 (ci-watcher.md + brief template) committed; empirical `gh run watch --exit-status` probes captured; exit-code table filled (no TBDs)
- [ ] Task 3 (schema-additive: decision-queue.md + impl-task.md + advisor-orchestrator.md) committed atomically
- [ ] Task 4 (plan.template.md §15 + §13) committed
- [ ] Task 5 (jm-d-impl-2.md §5 retrofit) committed; no other JM-d brief modified
- [ ] Task 6 (retro + lessons) committed
- [ ] PR opened by BM session against `governance-v0` with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-validate-agent-verify.md` shows all 6 stories ✓ (Story 5 may be `[deferred-to-retro]`)
- [ ] **`homeserver/library.yaml` registers all four new artefacts** (cargo-validate-workspace.yml, cargo-validate-migration.yml, ci-watcher.md, ci-watcher-brief.template.md) — sibling-repo edit, performed in homeserver checkout per `feedback_library_add_after_shipping.md`
- [ ] **`homeserver/CLAUDE.md`** has the one-line ci-watcher-is-Junior-subagent note — sibling-repo edit
- [ ] **validate-agent bm-merge unblocks JM-d Task 2 queueing — flag for advisor.** Per DQ #61, the advisor's polling loop may queue JM-d Task 2 after this PR merges; validate-agent does NOT queue it itself.
- [ ] Post-merge phase branch retained for retro reads (per JM-b/c/d precedent)

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `gh run watch --exit-status` exit-code semantics surprise (e.g. exits 0 even on failure) | LOW | HIGH | §4 watchpoint #1 mandates empirical probes BEFORE ci-watcher.md ships. Defensive fallback in §10.6: post-watch `gh run view <id> --json conclusion` to disambiguate. Task 2 gating: probes log + non-TBD exit-code table required at commit time. |
| GitHub Actions free-tier minute exhaustion on a busy month | LOW | MED | Trigger pattern is push-only on `phase-v1-*` + `junior/*` (DQ #66); no `pull_request` trigger. Estimated 80–200 validations/month free; ~$1.40/sub-phase paid. Retro §5 watch-item if approaching limit. |
| Workflow run misses on first push to `phase-v1-validate-agent` (trigger pattern bug) | LOW | MED | §15.2 confirms workflow fires on first push; §16a Story 1 is the end-to-end checkpoint. If trigger doesn't fire, Task 1 commit is amended (or a follow-up `chore(ci): fix trigger` commit lands before Task 2). |
| Cache-key collision between new workflows and cargo-test-e2e.yml | LOW | LOW | §4 watchpoint #3 enumerates the new prefixes (`cargo-validate-workspace-`, `cargo-validate-migration-`); §10.1 specifies they're distinct from `cargo-e2e-`. Cache namespace verified at workflow author time. |
| Schema-additive Task 3 splits into multiple commits, creating a window where rule doc drifts from consumers | LOW | HIGH | Task 3 GOTCHA explicitly forbids splitting; commit atomicity is the architectural invariant per `feedback_update_rule_doc_with_schema_additive.md`. `git log -1 --name-only --pretty=""` post-commit confirms all 3 files in one commit. |
| impl-task.md anchor link rot (rename of "Per-task validation gate" → "Per-task validation gate (out-of-band on GH Actions)" breaks cross-file links) | LOW | LOW | §4 watchpoint #4 + Task 3 audit step (`rg -n 'per-task-validation-gate' .claude/`); update any hits in same commit. |
| ci-watcher misclassifies a `validate-failed` workflow as `validate-result: pass` (false-green) | LOW | HIGH | Frontmatter pin to Haiku 4.5 with low effort + narrow tools (no Agent, no LSP) keeps the classifier mechanical. Empirical exit-code table per §4 watchpoint #1 + §10.5 — no heuristic guessing. ci-watcher's tooling is `gh run watch --exit-status` (semantic exit code), not log-grep. |
| `gh` CLI auth on EliteDesk daemon expires mid-session | LOW | MED | ci-watcher pre-flight `gh auth status` probe (§10.5 + §4 watchpoint #8) catches this before the first watch call. ci-watcher writes `validate-failed reason: gh_unauth` and exits 0; advisor surfaces as catch-fire; user re-runs `gh auth login`. |
| jm-d-impl-2.md §5 retrofit cascades into other §X edits accidentally | LOW | LOW | Task 5 GOTCHA explicit: §5 only. `git diff` on the commit shows only §5 body changed. |
| Retro task scope creeps (e.g. tries to author a v1-JM-e plan) | LOW | LOW | Task 6 ACTION specifies retro scope: `.claude/PRPs/reports/v1-validate-agent-retro.md` + lessons promoted. Plan stub for v1-JM-e is §20 (this plan), not a Task 6 deliverable. |
| Mid-phase context-window exhaustion (post-Phase-1 ~360k token zone) | LOW | LOW | This sub-phase is 7 tasks, 2 cohorts of [P]+[P], each cohort ≤2 tasks. Token cost per task is low (no cargo logs to inline; per `.claude/rules/no-cargo-output-paste.md`). |
| `homeserver/library.yaml` registration drift (new files invisible to `/library list/search/sync`) | MED | LOW | §17 Completion checklist requires all four artefacts registered. Each Task 1 + Task 2 commit includes a sibling-repo `library.yaml` edit (per per-task wording in §13). The sibling edit is out-of-band but the §17 checklist enforces it. |

---

## 19. Notes

- **PMD search-coverage gap (handover §"Phase outputs (synthesised)" #9).** Three brief-named lessons (`feedback_library_add_after_shipping`, `feedback_update_rule_doc_with_schema_additive`, `feedback_commit_aggressively_in_shared_repos`) exist as homeserver user-scope `.md` files but were NOT surfaced by `memory_search_hybrid` during plan-write. Recovered via direct file Read in homeserver session. Likely cause: 2026-04-27 PMD backfill didn't ingest these. **NOT a v1-validate-agent deliverable**; surfaced for retro to flag the PMD ingest pipeline. §12 carries this forward; §5 retro watch-item.

- **Empirical probe gating.** §4 watchpoint #1 + §10.5 + §13 Task 2 all reinforce the same rule: ci-watcher.md MUST NOT ship until `gh run watch --exit-status` empirical exit codes are captured against real success/failure/cancelled/queued runs. The plan does NOT pre-fill an exit-code table — that's the implementing session's responsibility. Defensive fallback: post-watch `gh run view <id> --json conclusion` for disambiguation.

- **Single-brief bounded retrofit (DQ #61).** Task 5 retrofits ONE brief — `jm-d-impl-2.md` §5 — to switch from inline cargo to push-and-exit. ALL OTHER JM-d briefs and ALL earlier-phase briefs/plans (v1-AD-*, v1-JM-a/b/c) stay forward-only-immune. Plan-template §15 "DoD per workflow" applies to v1-JM-e and later; pre-existing plans stay under prose §15. Per `feedback_schema_changing_spec_retrofit_question.md`.

- **Cohort dispatch.** Tasks 1+2 form a `[P]+[P]` cohort (file-disjoint: `.github/workflows/` vs `.claude/agents/`). Tasks 4+5 form a `[P]+[P]` cohort post-Task-3 (file-disjoint: `.claude/PRPs/templates/` vs `.claude/PRPs/briefs/`). Task 3 is a barrier (atomic schema-additive). Task 0 (audit) and Task 6 (retro) are non-`[P]` by template convention.

- **Resource budget.** This sub-phase does NOT run cargo on Junior. Budget concern is GitHub Actions free-tier minutes (2000/mo for private repo). Estimated cost: ~$1.40/sub-phase paid; ~80–200 validations/month free-tier. Comfortable for 4–5 sub-phases/month; tight if higher. Per ci-validation-design.md §6 cost note.

- **Forbidden-window applicability.** Standard for any local cargo work (advisor-side dry-run of §15 entries during Task 0 audit). Non-binding for impl-task throughput under Shape G — cargo is GitHub-runner-side. Per `advisor-orchestrator.md` Forbidden execution windows + Task 3 insertion.

- **Confidence score: 7/10.** Discount for: (1) empirical-probe dependency in Task 2 — `gh run watch --exit-status` exit semantics are unconfirmed at plan-write time, and the impl session's first action there is exploratory; (2) §G4 classifier allowlist is conservative-by-design but may need tuning at retro (Story 5 may not exercise on first push); (3) the schema-additive atomicity discipline (Task 3) is well-documented in lessons but new for this sub-phase. The §16a Stories block + per-task validation gates + R5 probe enumeration all proven.

- **Pre-existing PMD entries this plan leans on (without explicit citation in §2):**
  - PMD #109 (impl-task on Opus despite Sonnet pin) — informs ci-watcher Haiku-4.5 frontmatter discipline.
  - PMD #110 (telemetry blind-spot, orphan cargo PID) — direct evidence Shape G eliminates.
  - PMD #111 (Server Boss alert AND-gate) — sibling effort; complementary, not redundant.

- **Why this plan does NOT pre-seed planner DQ entries.** During plan-write, no question surfaced that needed advisor input beyond what DQ #62–#67 already resolved. The empirical-probe dependency in §4 watchpoint #1 is a Task 2 gating concern, not a plan-blocker. If the impl session encounters surprise during empirical probes (e.g. exit codes differ across `gh` CLI versions on EliteDesk vs GitHub runners), file as `kind: "blocker"` from `from: "impl"` per `decision-queue.md` §"Recipes."

- **Why this plan does NOT include LESSON: trailers.** Per `feedback_principles_not_rules.md`: keep DQ tight, prefer LESSON: trailers for purely informational findings. Nothing surprising surfaced during plan-write — every constraint, watchpoint, and pattern derives from existing lessons or the originating design brief. If empirical probes surface a footgun ("ci-watcher's exit-code interpretation depends on WSL-vs-Linux gh CLI") the impl session adds a `kind: "log"` DQ entry per Recipe 2 in `decision-queue.md`.

---

## 20. Sub-phase stub (v1-validate-agent retro follow-ups)

Per JM-b/c/d precedent: each sub-phase's plan declares the next sub-phase's
seed work. v1-validate-agent's retro establishes the watch-list for the
first Shape-G-ready sub-phase (v1-JM-e or v1-rep-tuning-r3, whichever
queues first):

- **v1-JM-e** writes its plan under the new "DoD per workflow" §15
  shape from Task 4. impl-task briefs are authored fresh under Layer
  G2 push-and-exit (no retrofit needed for greenfield briefs).
  Cohort dispatch fully respects Shape G: cohort members each enter
  `validate-pending` simultaneously; advisor advances on all-pass.
- **JM-d Task 2** is the first concrete validate-agent consumer post-
  merge. The advisor's polling loop reads `jm-d-impl-2.md` §5 (now
  push-and-exit), queues the impl-task per the parked-status flag in
  DQ #61's `answer`, observes the workflow run + ci-watcher cycle
  end-to-end. If the cycle works as designed for Task 2, validate-
  agent's design assumptions are confirmed for v1 throughput.
- **Branch-protection rules confirmed (DQ #66 deferred work).** The
  workflows trigger on push (not pull_request); GitHub branch-protection
  rules need configuring out-of-band to require `cargo-validate-workspace.yml`
  to pass before merging into `governance-v0`. Validate-agent's retro
  surfaces this as a watch-item; the configuration itself is a one-
  time GitHub-UI action, not a plan deliverable.
- **PMD ingest pipeline fix.** Three brief-named lessons not surfaced
  by `memory_search_hybrid`. v1-validate-agent retro flags the
  ingest-pipeline issue; the fix itself is a homeserver-side
  diagnostic, not a brehon-fork plan.
- **§G4 classifier allowlist tuning.** After 2–3 sub-phases run under
  Shape G with real validate-failed signals, the allowlist's true-
  positive rate becomes measurable. v1-JM-e or v1-rep-tuning-r3 retro
  proposes additions/removals.

---

_Plan author: foreground advisor session (laptop, `C:\Users\barri\Developer\brehon-fork`,
2026-04-27). Resumed from `.claude/PRPs/handovers/advisor-2026-04-27-v1-validate-agent-planning.md`
@ `92433e9ea`. Plan committed on `governance-v0` per Junior-finalise-from-foreground convention;
the BM session cuts `phase-v1-validate-agent` AFTER user plan approval. Confidence 7/10. Zero
planner DQs raised — clarify pass at DQ #62–#67 closed every coverage question; empirical-probe
gating (§4 watchpoint #1) is Task 2 work, not a plan-blocker._
