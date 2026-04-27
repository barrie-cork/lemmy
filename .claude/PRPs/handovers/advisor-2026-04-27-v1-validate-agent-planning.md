# Advisor handover — 2026-04-27 — v1-validate-agent /prp-core:prp-plan paused mid-research

**Written:** 2026-04-27 by foreground advisor session running `/prp-core:prp-plan` in `C:\Users\barri\Developer\brehon-fork` on `governance-v0`.
**Author:** advisor session
**Branch:** governance-v0 @ `ba949f27f` (`chore(advisor): clarify v1-validate-agent-planning-1 — see DQ #62-#67`)
**Purpose:** Self-contained brief so the next session can resume the plan-write at Phase 5 (Architect) → Phase 6 (Generate plan file) without re-reading the 19 required-reading items in the brief.

## TL;DR

- This session: started `/prp-core:prp-plan` against `.claude/PRPs/briefs/v1-validate-agent-planning-1.md`. Completed Phases 0–3 (DETECT, PARSE, EXPLORE — 3 parallel Explore agents — and external `gh run watch` doc fetch). Did NOT begin authoring `.claude/PRPs/plans/v1-validate-agent.plan.md`.
- User pause-signal: "We will continue this in another session" + "so update relevant files".
- Pending: the plan file itself + any planner DQ pre-seeds. Six §13 tasks pre-shaped (see "Plan shape decided" below) but not committed.
- Blocked: nothing on advisor side. Resume is a continuation of the same `/prp-core:prp-plan` flow — no new gates needed.
- Next session: at minimum, read this file + the brief + `.claude/PRPs/templates/plan.template.md`. Skip re-running Explore agents (results are inlined here). Author the plan body per the §13 cohort layout in "Plan shape decided" below; populate §16a Stories per the 6 listed; run §15 dry-run smoke before commit.
- External gates: none active. No PR. No DQ pending blocks this work.

## Cold-resume sequence

1. Read `CLAUDE.md` + `.claude/rules/*.md` (auto-loaded in `-p`; manually in interactive: `branch-manager.md`, `decision-queue.md`, `handover.md`, `phase-branch.md`, `advisor-orchestrator.md`, `pre-phase-harness-audit.md`, `cargo-output-capture.md`, `no-cargo-output-paste.md`, `pm-plugin-hooks-stable.md`).
2. Read this file in full.
3. Read `.claude/PRPs/briefs/v1-validate-agent-planning-1.md` (the brief; 152 lines; load-bearing).
4. Read `.claude/PRPs/briefs/ci-validation-design.md` (the originating design brief; ~234 lines; load-bearing — the plan is the canonical translation of its §3–§13).
5. Read `.claude/PRPs/templates/plan.template.md` (305 lines; canonical 20-section schema per DQ #65 — follow literally).
6. Skim `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` §1, §13 Task 0, §15, §16, §17 — for shape mirror only (JM-d predates §16a Stories block; v1-validate-agent must include it per plan.template.md §16a).
7. Verify state:
   ```bash
   git fetch origin
   git rev-parse HEAD              # expect ba949f27f
   git status --short              # expect clean
   git log --oneline -5
   gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,mergeStateStatus
   cat .claude/decision-queue.json | python -c "import sys, json; d=json.load(sys.stdin); print('pending:', len(d.get('pending', [])), 'resolved:', len(d.get('resolved', [])))"
   ```
8. Skip Phases 0–3 of `/prp-core:prp-plan` — synthesised inputs are inlined below in "Phase outputs (synthesised)". Resume directly at **Phase 5 (Architect)** → **Phase 6 (Generate plan file)**.
9. Run §15 dry-run smoke against current HEAD before commit per `feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md`. For workflow-referenced §15 entries, the dry-run shape is `yamllint <yaml-path>` exit 0 OR `gh workflow view --repo barrie-cork/lemmy --ref <phase-branch> <workflow>.yml` returning a parsed view (per DQ #67).
10. Commit subject: `docs(plan): v1-validate-agent plan written` (matches the JM-d/JM-c plan-commit convention). Pre-seeded planner DQ entries (if any surface during plan-write) get a separate `chore(decision-queue): pre-seed <ids> from planner` commit.

## State at handover

### Git
- Branch: `governance-v0`
- HEAD: `ba949f27f`
- Working tree: clean as of last `git status --short`. The handover file at `.claude/PRPs/handovers/advisor-2026-04-27-v1-validate-agent-planning.md` is the only new file (this one — committed alongside the runlog append).
- Unpushed: nothing yet (this handover commit is the next push).

### Worktrees
| Path | Branch | Purpose |
|---|---|---|
| `C:/Users/barri/Developer/brehon-fork` | `governance-v0` | Primary — advisor lane (this brief written here) |
| `C:/Users/barri/Developer/brehon-fork-phase-v1-JM-d` | `phase-v1-JM-d` | JM-d work; Task 2 parked per DQ #61 until v1-validate-agent ships |

### Decision queue
- Pending: 0 (all clarify entries on `v1-validate-agent-planning-1.md` resolved, DQ #62–#67).
- Resolved (validate-agent-relevant): #61 (Shape G chosen, JM-d Task 2 parked), #62–#67 (clarify pass), #55, #57, #59 (Shape-agnostic Shape G applies), #56, #58, #60 (Shape-C-specific, do NOT apply).

### Open PRs
- None advisor-gate. No v1-validate-agent PR yet — branch will be cut by `bm-cut` AFTER plan approval (per brief §120).

## Plan shape decided

**Output path:** `.claude/PRPs/plans/v1-validate-agent.plan.md`
**Template:** `.claude/PRPs/templates/plan.template.md` 20-section schema (per DQ #65)

### Files to change (per brief §47–§57)

| File | Action | Purpose |
|---|---|---|
| `.github/workflows/cargo-validate-workspace.yml` | CREATE | check + clippy + test-no-run on push to phase-v1-* + junior/* |
| `.github/workflows/cargo-validate-migration.yml` | CREATE | migration round-trip on push touching `migrations/**` |
| `.claude/agents/ci-watcher.md` | CREATE | new fifth subagent, Haiku 4.5, low effort, `gh run watch --exit-status` polling |
| `.claude/PRPs/templates/ci-watcher-brief.template.md` | CREATE | minimal 3-field brief: workflow_run_id + branch + phase_task |
| `.claude/agents/impl-task.md` | UPDATE | rewrite "Per-task validation gate" body + rename to "Per-task validation gate (out-of-band on GH Actions)" + append one line to "Hard refusals" (per DQ #64; section names, not §-numbers) |
| `.claude/rules/decision-queue.md` | UPDATE | add `kind: validate-pending`, `validate-result`, `validate-failed`; add `from: ci-watcher` only (per DQ #63 — NEVER rewrite historical entries); update polling-loop routing table; update Subagents-and-attribution enumeration; update Hard-refusals (write-side) |
| `.claude/rules/advisor-orchestrator.md` | UPDATE | insert validate-stage transitions in "Stage-shape orchestration" (between current line 197 and line 198); extend "Forbidden execution windows" with note that cargo no longer runs locally for impl-task; extend "Catch-fire procedures" with 1–2 new triggers (workflow timeout, ci-watcher classifier miss); extend "Cohort dispatch" with parallel-validate-pending clause |
| `.claude/PRPs/templates/plan.template.md` | UPDATE | §15 gains "DoD per workflow" sub-section; §13 task composition guidance per design §G5 |
| `.claude/PRPs/briefs/jm-d-impl-2.md` | UPDATE | §5 retrofit (only) — switch from inline cargo to push-and-exit per DQ #61. Bounded retrofit: jm-d-impl-{1,3,4,5,6,7} stay forward-only-immune. |
| `homeserver/library.yaml` | UPDATE (sibling repo) | register every new file (workflows + ci-watcher.md + brief template + any new lessons). Lives in homeserver, not brehon-fork — flag for sibling-session work but the plan §17 Completion checklist must include the registration line. |
| `homeserver/CLAUDE.md` | UPDATE (sibling repo) | one-line note that ci-watcher is a Junior subagent, not a daemon |

### §13 cohort layout (6 §13 tasks)

```
Task 0  (non-[P], barrier)        — Pre-flight harness audit + branch verification
Task 1  [P]                       — Workflow YAMLs (cargo-validate-workspace.yml + cargo-validate-migration.yml)
Task 2  [P]  (cohort with Task 1) — ci-watcher.md + ci-watcher-brief.template.md
Task 3  (non-[P], barrier)        — Schema additive (atomic): decision-queue.md + impl-task.md + advisor-orchestrator.md commit together (per feedback_update_rule_doc_with_schema_additive.md)
Task 4  [P]                       — plan.template.md §15 + §13 update
Task 5  [P]  (cohort with Task 4) — jm-d-impl-2.md §5 retrofit (single-brief bounded retrofit per DQ #61)
Task 6  (non-[P], retro)          — Retro per feedback_retro_not_report.md + feedback_four_role_retro_signals.md + feedback_retro_task_complexity_score.md
```

**Rationale for Task 3 being a barrier:** the three files MUST commit atomically per `feedback_update_rule_doc_with_schema_additive.md` — schema-additive value (`kind` + `from`) and consumers (impl-task writer, advisor reader) ship together. Splitting risks a window where the rule doc enumerates a kind that no consumer can write/read.

**Rationale for Tasks 1+2 being a cohort:** workflow YAMLs (`.github/workflows/`) and ci-watcher.md (`.claude/agents/`) share zero file paths. Independently testable.

**Rationale for Tasks 4+5 being a cohort (post-Task-3):** plan.template.md (`.claude/PRPs/templates/`) and jm-d-impl-2.md (`.claude/PRPs/briefs/`) share zero paths. Both depend on Task 3 having shipped (the new §15 shape references the new `kind` values; the retrofit references the new validation-gate body in impl-task.md).

### §16a Stories block (6 stories)

1. **Story 1 (Task 1):** "impl-task pushes to a phase-v1-* branch and `cargo-validate-workspace.yml` triggers automatically" — checkpoint via `gh workflow view --repo barrie-cork/lemmy --ref phase-v1-validate-agent cargo-validate-workspace.yml` after first test push.
2. **Story 2 (Tasks 1+2+3):** "ci-watcher reads a `validate-pending` DQ entry and polls a workflow run via `gh run watch` to completion."
3. **Story 3 (Tasks 2+3):** "ci-watcher classifies a `success` workflow as `validate-result: pass` and writes the DQ entry."
4. **Story 4 (Tasks 2+3):** "ci-watcher classifies a `failure` workflow as `validate-failed` with the failing log slice attached."
5. **Story 5 (Task 3):** "advisor §G4 classifier auto-queues a fix-impl-task for a known clippy lint" — gated; impl session may stub if no real failure surfaces during the test push.
6. **Story 6 (Task 5):** "validate-agent's bm-merge unblocks JM-d Task 2 (single-brief retrofit + queue)."

### §4 watchpoints (specific file:line)

1. **`gh run watch --exit-status` exit-code semantics** — no codebase precedent (per DQ #62 + Phase-3 doc fetch). External docs at https://cli.github.com/manual/gh_run_watch confirm `--exit-status` returns non-zero on failure but DO NOT document success exit code or timeout behaviour. Plan §4 must require empirical validation before authoring ci-watcher.md:
   ```bash
   gh run watch <success-id> --exit-status; echo "success: $?"
   gh run watch <failure-id> --exit-status; echo "failure: $?"
   gh run watch <cancelled-id> --exit-status; echo "cancelled: $?"
   gh run watch <queued-id> --exit-status; echo "queued (timeout test): $?"
   ```
2. **Workflow trigger pattern** — `on: { push: { branches: ['phase-v1-*', 'junior/*'] } }` only (DQ #66). NO `pull_request` trigger. Defence-in-depth via branch protection + advisor-orchestrator merge gate.
3. **`actions/cache@v4` cache-key collision avoidance** — new keys `cargo-validate-workspace-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}` and `cargo-validate-migration-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}`. Must NOT collide with existing `cargo-e2e-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}`. Restore-keys: prefix-based (`cargo-validate-workspace-${{ runner.os }}-`).
4. **`impl-task.md` "Per-task validation gate" — section anchor preserved on rename** — current H2 lines 53–60. New title: "Per-task validation gate (out-of-band on GH Actions)". Markdown anchor changes (`per-task-validation-gate-out-of-band-on-gh-actions`); audit any cross-file links that target the old anchor before commit.
5. **`decision-queue.md` Subagents-and-attribution enumeration** — currently lines 462–495, with 4 valid `from` values: `planner`, `impl`, `bm`, `advisor`. Adding `ci-watcher` requires: subagent name + dispatch line `[role:ci-watcher]` + `from: "ci-watcher"` write rule + `kind: "validate-result"|"validate-failed"` write rule + ci-watcher-self-resolved `answered_by` rule. Per DQ #63: do NOT rename impl to impl-task; do NOT rewrite historical entries.
6. **`advisor-orchestrator.md` Stage-shape orchestration insertion point** — between current line 197 ("Each impl-task complete") and line 198 ("All §16a stories `[done]`"). Two new bullets inserted as one block: validate-pending dispatch + validate-result/failed routing.
7. **§G4 classifier scope** — DQ #57 says ≤3 file edits = auto-fix-impl-task; >3 = catch-fire. Lives in `advisor-orchestrator.md`. Plan §13 Task 3 specifies the lint allowlist (clippy::doc_lazy_continuation, missing imports, deprecated APIs — narrow per design §8.7).
8. **`gh` CLI auth on EliteDesk** — design §8.5 says `barrie-cork` already authenticated. ci-watcher.md must include `gh auth status` pre-flight probe (mirror impl-task.md task-0 forbidden-window check shape).
9. **PMD search-coverage gap** — three brief-named lessons (`feedback_library_add_after_shipping`, `feedback_update_rule_doc_with_schema_additive`, `feedback_commit_aggressively_in_shared_repos`) exist as homeserver user-scope `.md` files (visible in `~/.claude/projects/.../memory/MEMORY.md`) but are NOT surfaced by `memory_search_hybrid` even with their keywords in the query. Recovered via direct file Read (in homeserver session). Likely cause: 2026-04-27 PMD backfill didn't ingest these. Plan §19 surfaces this for retro to flag PMD ingest pipeline.

### Phase outputs (synthesised — do NOT re-run Explore agents)

**Phase 0 (DETECT):** input is a free-form description from `/prp-plan` slash-command argument; brief is the source-of-truth at `.claude/PRPs/briefs/v1-validate-agent-planning-1.md`. Output target: `.claude/PRPs/plans/v1-validate-agent.plan.md`.

**Phase 1 (PARSE):** post-v0 infrastructure sub-phase (NOT v0 endpoint scope); CROSS_CUTTING type; MED complexity; 11 affected files (see "Files to change" above); none of the 15 ADRs constrain directly; no blocking OQs (all clarify-DQ on this brief resolved).

**Phase 2 (EXPLORE — 3 parallel agents):**

- **Workflow YAML patterns** — mirror `cargo-test-e2e.yml` cache strategy (lines 58–67), libpq install (lines 53–56), Rust toolchain (lines 47–51), submodule recursive checkout (lines 37–45). Diverge on: trigger (DQ #66 push-only), concurrency key (`github.ref` instead of `pull_request.number`), cache key prefix. Confirmed absence of all 4 deliverables: `cargo-validate-workspace.yml`, `cargo-validate-migration.yml`, `ci-watcher.md`, `ci-watcher-brief.template.md`.
- **impl-task.md H2 sections (named, not numbered per DQ #64):** "Per-task validation gate" at lines 53–60 (REWRITE BODY + rename); "Hard refusals" at lines 130–139 (APPEND ONE LINE).
- **decision-queue.md landmarks:** "kind: blocker vs log vs clarify" lines 147–209 (EXTEND with 3 new kinds); polling-loop routing table lines 192–208 (extend with 3 new (kind, status) routes); Hard refusals (write-side) lines 327–337 (EXTEND); Subagents and attribution lines 462–495 (EXTEND with `ci-watcher`). Current `from` distribution: advisor=28, impl=25, planner=11, bm=0, user=0, ci-watcher=0 (confirmed).
- **advisor-orchestrator.md landmarks:** "Stage-shape orchestration" lines 190–205 (insert validate transitions between line 197 and 198); "Forbidden execution windows" lines 63–102 (EXTEND with cargo-no-longer-local note); "Catch-fire procedures" lines 261–272 (EXTEND with 1–2 new triggers); "Cohort dispatch" lines 217–243 (EXTEND with parallel-validate-pending clause); "Mandatory user gates" lines 114–122 (NO change — validate-stage doesn't add a new user gate).

**Phase 3 (RESEARCH — gh run watch external):**

WebFetch on https://cli.github.com/manual/gh_run_watch returned:
- `--exit-status` flag: "Exit with non-zero status if run fails."
- `--interval` default 3 seconds.
- Polls until completion ("Watch[es] a run until it completes").
- Success exit code + timeout behaviour NOT documented.
- Limitation: no fine-grained PAT auth (no `checks:read` permission available).

→ Plan §4 watchpoint #1 above is the empirical-validation requirement.

## Key citations (for the plan to reference verbatim)

**Brief-cited lessons (recovered via homeserver session, NOT in `.claude/lessons/`):**

1. **`feedback_library_add_after_shipping.md`** — "After commits adding files to scripts/, .claude/{skills,commands,rules}/, run `/library add <name> <source>` before moving on. Catalog drift makes new files invisible to /library list/search/sync."
   → Plan §17 Completion checklist gets a `library.yaml registration` line for every new artefact (workflows + ci-watcher.md + brief template + any lessons).

2. **`feedback_update_rule_doc_with_schema_additive.md`** — "When introducing a new enum value / frontmatter field / schema variant for shared artifacts (decision-queue, brief, retro, agent), edit the canonical rule doc in the same commit. Schema drift between rule doc and consumers = most-common silent-correctness bug."
   → Plan §13 Task 3 enforces this: `decision-queue.md` + `impl-task.md` + `advisor-orchestrator.md` commit atomically.

3. **`feedback_commit_aggressively_in_shared_repos.md`** — "After every Edit on `.claude/` files in shared repos (brehon-fork especially): `git status` → `git add` → `git commit` immediately. Don't batch. Sibling session can silently switch worktree branch and discard uncommitted edits. Disk + git log are ground truth; harness 'modified' reminders are cache-stale."
   → Plan §13 task wording enforces one-commit-per-task. No batching.

**Lessons in `.claude/lessons/` directly read:**

- `feedback_clarify_before_plan.md` — already gated planning; informational for retro.
- `feedback_parallel_cohort_dispatch.md` — `[P]` markers + budget check.
- `feedback_pre_phase_dod_smoke_test.md` — every §15 DoD command must dry-run literally.
- `feedback_pipes_mask_exit_codes.md` — exit-code discipline (applies to `gh run watch --exit-status` validation).
- `feedback_story_grain_checkpoint.md` — §16a Stories block.
- `feedback_pr_per_phase.md` — code via PR; meta-work via direct. Two new workflow files in `.github/workflows/` make this a code-PR (per `phase-branch.md` §"What goes through the PR flow").
- `feedback_dogfood_slash_command_specs.md` — applies to ci-watcher.md (mental-walk one validate-failed shape per brief §109).
- `feedback_schema_changing_spec_retrofit_question.md` — schema-additive `kind` values trigger retrofit question (already resolved by DQ #61: bounded retrofit = jm-d-impl-2.md §5 only).
- `feedback_read_canonical_before_writing_spec.md` — applies to ci-watcher.md authoring (mirror impl-task.md/bm-task.md frontmatter shape).
- `feedback_features_full_p_crate_incompatible.md` — workspace-only DoD (already known).
- `feedback_clippy_test_style.md` — N/A for this sub-phase (no Rust code).
- `feedback_principles_not_rules.md` — applies to plan §13 skill-trigger hints.

**PMD-tangential lessons (loose match):**

- PMD #112 (verify-not-given subagent framing) — applies to ci-watcher.md prompt design.
- PMD #8 (pipes mask exit codes) — applies to `gh run watch --exit-status` exit-code semantics.

## Closing state assertions

When this handover was written:

- HEAD = `ba949f27f` on `governance-v0`.
- Working tree clean before this handover commit.
- 0 advisor-gate PRs.
- 0 advisor relays awaiting impl response.
- 0 DQ pending (all clarify-DQ on validate-agent-planning-1 resolved).
- `.claude/PRPs/plans/v1-validate-agent.plan.md` does NOT exist.
- `.claude/agents/ci-watcher.md` does NOT exist.
- `.github/workflows/cargo-validate-workspace.yml` does NOT exist.
- `.github/workflows/cargo-validate-migration.yml` does NOT exist.
- `phase-v1-validate-agent` branch does NOT exist (cut by bm-cut AFTER plan approval).

## What the next session does NOT need to do

- Re-run any of the three Explore agents — outputs are inlined above under "Phase outputs (synthesised)".
- Re-fetch `gh run watch` external docs — Phase 3 result inlined.
- Re-glob `.claude/lessons/` — keyword-matching set already enumerated.
- Re-read all 19 brief required-reading items — only items 2–5 (template, design briefs, JM-d/JM-c plans) need re-reading at minimum to author plan body.
- Re-compute the §13 cohort layout — decided above.
- Re-decide the §16a Stories block — 6 stories pre-shaped above.

## What the next session DOES need to do

1. Read this handover + the brief + the design brief + plan.template.md (4 files; ~1000 lines total).
2. Author `.claude/PRPs/plans/v1-validate-agent.plan.md` body per the §13 cohort layout + §16a stories above. Follow plan.template.md 20-section schema literally.
3. Optionally pre-seed planner DQ entries for any §G4 classifier scope ambiguity that surfaces during plan-write (use `from: "planner"` + `answered_by: "planner"` per `decision-queue.md` Attribution integrity §2).
4. Run §15 dry-run smoke against current HEAD per `feedback_plan_dod_dry_run_at_write.md`. For workflow-referenced DoD entries, the dry-run shape is `yamllint <yaml-path>` exit 0 OR `gh workflow view --repo barrie-cork/lemmy --ref <phase-branch> <workflow>.yml` per DQ #67.
5. Commit: `docs(plan): v1-validate-agent plan written` (matches JM-d/JM-c convention).
6. Optionally, append a `LESSON:` trailer to the commit body if anything in plan-write surfaces something a future planner would have wanted to know.
7. Surface plan to user for explicit approval (per `.claude/rules/advisor-orchestrator.md` "Mandatory user gates — Plan approval"). Wait. Do NOT queue `bm-cut` until user approval.

## Notes for the resume session

- The progress note at `.claude/PRPs/debug/v1-validate-agent-planning-progress.md` (which was tracked but uncommitted at handover-write time) is now superseded by THIS file. Delete it before commit.
- `.claude/PRPs/debug/` is NOT in `.gitignore` — only `*.log`, `build-*.log`, `audit-*.log` are. Future progress files should land at `.claude/PRPs/handovers/` (tracked per `handover.md` rules) or use one of the gitignored patterns.
- The brief expects a forbidden-window-aware queue dispatch (per `advisor-orchestrator.md` §Forbidden execution windows). For ad-hoc local advisor cargo runs (e.g. dry-running the §15 DoD commands), the existing windows still apply. The shipped Shape G removes cargo from impl-task throughput, NOT from advisor's hands.

---

_Handover author: foreground advisor session (laptop, `C:\Users\barri\Developer\brehon-fork`). Written 2026-04-27 in response to user pause-signal mid-Phase-5. Resume in any session — fresh CC instance, this CC instance post-compaction, or next-day session. The plan-write is independent of any other in-flight work and has no time-pressure beyond "before JM-d Task 2 can be queued"._
