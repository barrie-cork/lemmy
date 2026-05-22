# Advisor as orchestrator (persistent session)

The advisor session orchestrates one Brehon sub-phase end-to-end via Junior subagents (`planning`, `impl-task`, `bm-task`, `ci-watcher`). Meta-oversight only — never authors content. See `CLAUDE.md` "Four-role model" and `feedback_brehon_autonomy_goals` (autonomous + reliable + slow-OK + model-efficient). Polling is the trigger surface — no Telegram, n8n, or RemoteTrigger.

## 1. Polling loop

- **Session-start CWD check** (per `.claude/rules/multi-lane-worktree.md` 2026-05-11): run `pwd && git branch --show-current && git worktree list` at session start. If another active worktree exists on a different `phase-v1-*` branch, verify this session's CWD matches the intended lane. Lane-dedicated worktrees live at `C:/Users/barri/Developer/brehon-fork-<lane>`; the canonical `brehon-fork` checkout is reserved for `governance-v0` meta-edits (rules / lessons / templates / briefs on trunk). Lane-dedicated sessions write phase-branch DQ entries; canonical sessions do not.
- **Surface-first ritual (2026-05-22, post-RT-r2 session boundary incident):** when ANY of these conditions hold at session start — (a) the UserPromptSubmit DQ-pending hook reports pending > 0, (b) `git worktree list` shows ≥2 worktrees, (c) the `session-start-multi-lane-check.sh` hook emitted a WARN — the FIRST user-visible response in the session MUST be a one-line lane status: `lanes: <CWD>:<branch> active; other-active: <list-of-other-active-lanes-or-"none">`. The line goes BEFORE any task-execution response, BEFORE any tool call, even when an inherited turn's stdout (e.g. post-`/clear` recommendation) frames the next action. The ritual exists to compete against inherited-context momentum: 2026-05-21 the canonical session pushed `cf93b7ba6` to `governance-v0` while another session was driving `phase-v1-federation-inbound-c` + the schema-v3 migration — the worktree list was visible but never surfaced; the inherited "plan RT-r2 now" anchored the session into action before the lane check. Per `project_concurrent_advisor_sessions_2026_05_21.md` + `feedback_falsifiable_hypothesis_before_structural_fix.md`.
- Cadence: every ~10 min call `mcp__junior-brehon__list_tasks` (status only).
- On status transition: `show_task` for the changed task; `git fetch origin`; read `.claude/decision-queue.json`; triage new pending entries; queue next per stage shape (§3.1).
- Read the plan once after planning ships. Read DQ on every poll only if `git fetch` returns new commits. Memory injection (Glob `.claude/lessons/`, PMD search) at session start, not per poll.
- No state change → output is "no change". Nothing else loaded.
- Single-task focus → prefer `/start-brehon --fast <N>` (5 probes). If DQ pending > 0, escalate to `/check-dq`.
- DQ on a non-trunk branch → use `scripts/brehon/git-show-json.sh <ref> <path>` then `python -c "import io, json; ... io.open(r'<path>', encoding='utf-8') ..."`. Never `git show <branch-with-slashes>:<path>` directly. Per `feedback_windows_bash_python_git_show_tmp_traps.md`.
- Reconciling DQ on an in-flight phase (session start, long-poll resume, retro audit) → use `scripts/brehon/resolve-dq-canonical.sh <phase>`. Unions phase-branch DQ with open worker-branch DQs; dedupes by id (worker-branch wins). // 2026-05-09 c-2: laptop saw pending=0 while live advisor saw pending=2 because DQ #164+#165 lived only on worker-159 tip.
- **SessionStart canonical-PMD guard (post-v1-rls-r1):** the tracked `.claude/hooks/pmd-canonical-guard.sh` runs at every session start in lanes wired per the bootstrap checklist (`feedback_phase_lane_worktree_bootstrap_checklist.md` step 6). Surfaces as a stderr WARN on lane drift, exit 0 always — does NOT block. Per `.claude/rules/pmd-invariants.md` invariant #5.
- **SessionStart multi-lane check (post-RT-r2 boundary incident 2026-05-22):** the tracked `.claude/hooks/session-start-multi-lane-check.sh` runs at every session start in lanes wired per the bootstrap checklist. Reads `git worktree list`; for each non-CWD worktree on a `phase-v1-*` / `phase-v2-*` / `phase-brehon-*` branch, checks whether the branch tip was advanced within the last 30 minutes (heuristic for "another session is actively driving"). On detection: stderr WARN with this lane + the other active lane(s) + their last-commit age. WARN-not-FAIL (exit 0 always); the WARN lands in SessionStart system-reminders so the advisor sees it BEFORE the first tool call. Pairs with the surface-first ritual above — the hook is the mechanical signal, the ritual is the prose response. False-positive class: another lane's worktree exists but the session is idle (the threshold is generous to bias toward over-warning). False-negative class: a session active for >30 min without a commit. Both accepted because WARN-not-FAIL is cheap and the alternative (sub-30-min-window cliff) misses the real defect class (sessions that DID commit recently, like the 2026-05-21 fed-in-c session). Threshold tunable via `SESSION_START_MULTI_LANE_THRESHOLD` env var.
- **Pre-compact handover discipline (post-v1-retro-followups-r1, 2026-05-22):** any advisor session likely to span `/compact`, session-end, or context truncation MUST author a self-contained handover file BEFORE the boundary, at `.claude/PRPs/handovers/<phase>-<scope>-<date>.md`. The file's body MUST be readable by a future session with zero conversation context — it states the current sub-phase, the stage in the state machine, the last commit on the relevant branch, the next concrete action, and any cross-session dependencies (DQ pending entries by id, concurrent session activity). Validated by the v1-dq-schema-r1 Cohort 2 handover (`.claude/PRPs/handovers/v1-dq-schema-r1-cohort-2-handover-2026-05-22.md`) — enabled first-pass resume in 3 tool calls. Commit + push the handover file BEFORE invoking `/compact`; the handover lives on `governance-v0` even if the work is on a phase branch, so the canonical checkout always sees the latest.

## 2. Brief authoring

### 2.1 Junior task description template

Every Junior task is preceded by a brief at `.claude/PRPs/briefs/<phase>-<role>-<n>.md`, committed on the branch the Junior worker will fork from **BEFORE** the task is created. Workers fork from `base_branch`; the brief must be visible in that branch's tree at task-spawn time. Per role:

- **Planning briefs** → committed on `governance-v0` (no phase branch yet).
- **bm-cut briefs** → committed on `governance-v0` (no phase branch yet).
- **Impl-task briefs** → committed on `phase-<X>` (the phase branch the impl worker forks from). Authoring on `governance-v0` requires a subsequent cherry-pick + forward-merge to make the brief visible to the worker; the pattern observed in fed-in-b (commit `0ea7ab4f7`) is "author briefs directly on the phase branch". Per session retro 2026-05-20 §2.8.
- **BM-pr / bm-merge briefs** → committed on `governance-v0` (BM worker reads from trunk).
- **ci-watcher briefs** → committed on `governance-v0` (mutation lives on whatever ref the workflow_run_id's branch was; the brief just names IDs).

The dispatch string is intentionally minimal (under 100 chars, no URLs, no inline code, no secrets):

```
[role:<role>] <slug> — see .claude/PRPs/briefs/<file>.md
```

The slug becomes the Junior worktree branch name. Each task gets its own worktree (`feedback_parallel_agents_one_worktree_per_agent`).

### 2.2 Brief shape

Four sections, in order:

1. **Role + dispatch line** — `[role:planning|impl-task|bm-task|ci-watcher] <one-line summary>`. Must match the format the matching subagent reads.
2. **Scope** — what to produce; explicit boundaries (don't author <thing>; commit only <files>).
3. **Required reading** — paths the subagent must read first (`.claude/lessons/<file>`, `.claude/PRPs/plans/<plan>`, ADR sections).
4. **Constraints** — rules to enforce (DQ mid-task push, MIRROR-ref discipline, attribution integrity, file-ownership).

Briefs are tracked in git — the audit trail of what the advisor asked for.

### 2.3 Pre-queue lesson check

Before each brief, one `memory_search_hybrid` call against the brehon-fork PMD (limit: 5, round-trip <2s). Consult-only — internalise hits; surface explicit citations in §4 only when load-bearing. See `.claude/rules/pmd-search-strategy.md` for FTS5-only mode + keyword-vs-natural-language guidance. Lesson corpus is indexed via `scripts/sync-lessons-to-pmd.sh` (idempotent; re-run after authoring a new lesson).

This subsumes the session-start `.claude/lessons/` glob — that still happens once at session start, but pre-queue search adds the fresh lookup right before the brief is written.

### 3.1.1 Conformance-audit prevention checkpoint

**Skill invocation — conformance-audit (per `.claude/skills/brehon-conformance-audit/`)**:
If the brief targets a file matching `crates/apub/activities/src/governance/**.rs` OR
`crates/api/api/src/governance/**.rs` OR `crates/db_schema/src/source/governance/**.rs`,
invoke the skill with `target_scope = file <brief-named-file>` BEFORE `/brehon-clarify`.
Tier-1 findings fold into brief §3 / §4 before clarify-DQ entries.

### 2.4 Mandatory file-class lesson injection

When authoring an `impl-task` OR `fix-impl-task` brief, walk the file list against the table below and inject every match into §3 Required reading. **No judgment call** — pattern matches → lesson goes in.

- **`impl-task`** file list = §13 task's IMPLEMENT files + helper-creation files.
- **`fix-impl-task`** file list = every file:line citation in the failing log slice (parse `--> path:line` from `gh run view <id> --log-failed`).

| File pattern | Mandatory lesson(s) for §3 Required reading |
|---|---|
| `crates/server/tests/e2e.rs` (any edit, any size) | `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`. **When a v1-SL-* or v1-JM-* fixtures sibling module already exists in the same file, mirror its error-shape case (A or B) verbatim per `feedback_lemmy_error_no_std_error.md` case enumeration. Pick by reading sibling at the cited line range BEFORE authoring the brief — canonical-schema-first gate.** |
| `crates/server/tests/e2e.rs` (≥2 edits in this task or cohort) | + `feedback_junior_worker_e2e_edit_hang.md` |
| `crates/db_schema/migrations/**` (any new migration) | `feedback_lemmy_migration_runner.md`, `feedback_postgres_jsonb_canonicalization.md` (if JSONB) |
| Any new test under `crates/*/tests/**` returning `Result<(), Box<dyn Error>>` | `feedback_lemmy_error_no_std_error.md` |
| Any handler under `crates/api/**/src/**` doing 2+ DB writes | `feedback_multi_write_handlers_need_transactions.md` |
| Any `#[cfg(feature = "full")]` gate | `feedback_features_full_workspace_only.md`, `feedback_features_full_p_crate_incompatible.md` |
| Any `pg_advisory_xact_lock` or void PG function call | `feedback_pg_advisory_xact_lock_void_decode.md` |
| Any newtype under `crates/db_schema/src/newtypes/` | `feedback_newtype_locations_lemmy_db_schema_vs_file.md` |
| Any clippy fix involving `-D warnings` + new code | `feedback_clippy_test_style.md`, `feedback_clippy_rerun_after_fix.md` |
| Any `scripts/brehon/cargo-*.bat\|sh` edit | `feedback_wrapper_script_flag_silence.md`, `feedback_pq_sys_wrapper_env_propagation.md` |
| Any `.gitignore` / `.git/info/exclude` / hook addition | `feedback_settings_local_json_worktree_bootstrap.md` |

Brief commit body lists which mandatory lessons fired and why (one line each). The §2.3 hybrid search still runs after the table check — catches non-mechanical / cross-cutting lessons. // 2026-05-09 c-2 fix-impl-1 lapse: same E0277 LemmyError class as cycle-1; brief omitted lesson. Maintenance: when a new mechanical-fix pattern enters the §G4 allowlist (§5.3) and correlates with a file class, add a row here.

### 2.5 Plan §5 complexity-score awareness

When the next pending §13 task is cargo-class (DoD names `cargo check`, `cargo clippy --workspace`, or `cargo test --workspace`) AND plan §5.1 complexity score `> 8`:

1. Confirm planner's split-or-proceed DQ resolved (look in `.claude/decision-queue.json` resolved entries from `planner` referencing this plan).
2. Dominant factor = e2e edits (≥2) → append `feedback_junior_worker_e2e_edit_hang.md` to §3 Required reading.
3. Dominant factor = migrations (≥2) AND pre-Shape-G → expect cargo+migration peak ~6 GB on the laptop; note in polling output if low battery. Per `feedback_complexity_score_pre_split.md`.

Mechanical: read score, read top factor, add citation. No DQ, no escalation.

## 3. Stages and gates

### 3.1 Stage-shape orchestration

Per the c-inherited-dragon plan's stage map. Advisor knows what to queue next on each completion:

- **Brief authored, no planning task yet** → run `/brehon-clarify <brief-path>` → resolve every clarify-DQ entry → queue planning. Skipping clarify on a planning brief is a process breach.
- **Planning complete** → run §3.4 DoD smoke test → run §3.5 watchpoint specificity → user gate 1 (plan approval) → on approval, queue `bm-cut`.
- **bm-cut complete** → queue impl per §4 cohort dispatch: Task 1 (or first non-pre-flight) `[P]` → compute cohort, queue all simultaneously; otherwise queue alone.
- **impl-task complete (pre-Shape-G, v1-JM-d and earlier)** → if cohort has pending peers wait; else compute next cohort. All tasks done → `chore(lint):` follow-up if needed → `bm-pr`.
- **impl-task complete (Shape G, v1-JM-e onward)** — two-phase validation per option (b) 2026-04-28:
  - **Phase 1 (workspace-check on `junior/*`):** impl-task already wrote `kind: "validate-pending"` post-push (workflow_run_id + branch + phase_task; `result`/`log_slice`/`failed_jobs` null). Queue `[role:ci-watcher]` Junior task with brief from DQ entry fields (template `.claude/PRPs/templates/ci-watcher-brief.template.md`). Originating impl-task gated until ci-watcher resolves. **Serial ci-watcher rule (per `feedback_ci_watcher_serial_per_task_pair.md` 2026-05-11):** when a cohort has N members each with a validate-pending DQ, dispatch **ONE ci-watcher per logical task** (long-polling 1-2 workflow runs in sequence inside that ci-watcher), NOT N parallel ci-watchers. Parallel ci-watchers all fork off the same phase-branch tip; sequential finalize-merges then auto-resolve `.claude/decision-queue.json` conflicts by reverting earlier ci-watchers' mutations back to `pending` state ("resurrection bug"). Serial dispatch keeps each ci-watcher's worker branch in causal-order with the previous mutation. **Atomic raise-before-dispatch rule (per `feedback_dq_raise_before_ci_watcher_queue.md` 2026-05-11):** the `validate-pending` DQ entry MUST be committed and pushed BEFORE the `[role:ci-watcher]` Junior task is created. Junior worker branches fork from the current `phase-<phase>` tip at task-creation time; if the raise hasn't pushed yet, the ci-watcher's worker branch will not see the entry and will file a `kind: "blocker"` contract-violation. The atomic ordering is: (a) `git add .claude/decision-queue.json && git commit && git push origin <phase-branch>`, THEN (b) `mcp__junior-brehon__create_task`. NEVER reverse this order.
  - **Phase 2 (e2e, advisor-driven, off-Actions by default — 2026-04-28 minutes-budget audit):** `cargo-test-e2e.yml` no longer auto-fires on `phase-v1-*` push. After daemon finalize-merges, advisor sees new tip on next `git fetch` and runs e2e locally. See §5.2 validate-pending-laptop handler for the full flow (raise `kind: "validate-pending"` with `local_log_path` + `from: "advisor"`, no ci-watcher dispatch, advisor mutates the entry directly when bg cargo exits). User-gate 4 (Phase 2 e2e — local vs dispatch) selects local vs `gh workflow run cargo-test-e2e.yml`. Cohort advancement waits on BOTH workspace AND e2e mutated to `result: "pass"`.
- **ci-watcher complete** → read mutated entry. `kind` stays `"validate-pending"` regardless of result. `result: "pass"` (in `resolved[]`) → advance pipeline. `result: "fail" | "cancelled" | "timed_out"` (still in `pending[]`) → run §5.3 §G4 classifier.
- **All §16a stories `[done]`** (between last impl complete and bm-merge confirm) → run `/brehon-verify` → phantom → catch-fire; else advance to bm-pr.
- **bm-pr complete** → wait for CodeRabbit (`bm-task` polls) → on CR posted, queue `bm-poll-cr`.
- **bm-poll-cr complete** → queue `bm-triage` (draft auto).
- **Triage drafted** → user gate 3 (CR triage) → on approval, queue `impl-task` for fix-in-PR commits.
- **No critical findings open** → confirm `/brehon-verify` ✓ → user gate 5 (merge confirm) → queue `bm-merge`.
- **bm-merge complete** → author retro → user gate 6 (retro sign-off) → run `/brehon-phase-transition`.

Advisor never auto-merges or auto-resolves ADR-affecting DQ.

### 3.2 Mandatory user gates

Six gates, never skipped (goal #3: slow-OK):

1. **Plan approval** — after planning ships and §3.4 DoD smoke test passes.
2. **Judgment-heavy DQ** — ADR-affecting / scope-changing / visible-to-others impact. Use `answered_by: "user"` after relay.
3. **CR triage approval** — after `bm-poll-cr` + `bm-triage` draft. Surface four-bucket counts.
4. **Phase 2 e2e — local vs dispatch** — never auto-pick after PR #105. Options: (a) local — `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`, ~26 min, zero billed. **Never bare `cargo test` on Windows** (libpq.dll missing — bat wrapper sets vcpkg PATH); **never `-p lemmy_server --features full`** (lemmy_server has no `full` feature; use `--workspace`). See `feedback_windows_e2e_requires_bat_wrapper.md`. (b) dispatch — `gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-<phase>`, ~26 min billed, public log, ci-watcher polls.
5. **Merge confirm** — before `bm-merge`.
6. **Retro sign-off** — author retro per `feedback_retro_not_report`, `feedback_four_role_retro_signals`, `feedback_retro_task_complexity_score` (per-task `<files>/<commits>/<runtime-min>/<max-log-silence-min>`, aggregated in §5).

### 3.3 Clarify gate (pre-planning)

Per `.claude/commands/brehon-clarify.md` + `feedback_clarify_before_plan.md`. Before any **planning** Junior task, run `/brehon-clarify <brief-path>`. Produces `kind: "clarify"`, `from: "advisor"` DQ entries; planning is queueable only when every clarify-DQ on this brief is resolved (advisor self-answer with citation, or user reply relayed).

Applies to **planning briefs only** (impl/bm briefs derive from a clarified plan or are mechanical). Surface to user only if (a) brief required edits, or (b) `--mode user-relay` was needed. Pure advisor-mode pass with citations-only DQ entries reports completion without escalation.

### 3.4 DoD smoke test (pre-plan-approval)

Per `feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md`. When planning ships, run **every** validation command in §15 literally against current HEAD; capture exit codes; surface as part of plan approval. If a command is unexecutable (missing `--features full`, missing `--no-deps`, wrapper-script flag silence, `-p <crate>` + `--features full` per `feedback_features_full_p_crate_incompatible`), file a DQ pending from advisor (`chore(decision-queue): advisor noted DoD issue — <slug>`) and ask the planner to revise.

### 3.5 Watchpoint specificity gate (pre-plan-approval)

Per `feedback_advisor_watchpoint_specificity.md`. Every watchpoint in plan §4 must cite a specific table, file, or `schema.rs` line. Concept-only watchpoints ("watch for trait drift" without naming the trait) → file DQ requesting revision before approval.

### 3.6 Canonical-schema-first gate (pre-spec-authorship)

Per `feedback_read_canonical_before_writing_spec.md`. Before authoring any new `*.md` under `.claude/{rules,commands,lessons,PRPs/templates}`, `Glob` + `Read` 1-2 sibling instances first. Cite the canonical example in the new file body or commit body. A commit that adds such a file without citation is a process miss; retro flags it. `grep '^##' <existing>` is always worth the 2-second read.

### 3.7 Dogfood gate (new slash commands)

Per `feedback_dogfood_slash_command_specs.md`. Every new `.claude/commands/<verb>.md` must include a "Pre-commit dogfood" sub-section under `<rationale>` naming the real existing input the command was walked-through against, what worked, what didn't.

| Command class | Dogfood target |
|---|---|
| Planning-stage (e.g. `/brehon-clarify`) | Most-recent `.claude/PRPs/briefs/<phase>-planning-N.md` |
| Impl-stage | Most-recent `.claude/PRPs/briefs/<phase>-impl-N.md` |
| Verification (e.g. `/brehon-verify`) | Most-recent `.claude/PRPs/plans/<phase>.plan.md` |
| BM verb | Most-recent `.claude/runlog/<phase>.md` |

Cost of pre-commit dogfood ~5 min; cost of post-deploy fix ~10× that.

### 3.8 Schema-changing-spec retrofit gate (plan-mode shape changes)

Per `feedback_schema_changing_spec_retrofit_question.md`. When plan-mode produces a plan that changes the shape of an artifact class (new section in a template, new marker in a section, new field in JSON/YAML schema, new required sub-section in a frontmatter), advisor calls `AskUserQuestion` **once before `ExitPlanMode`**:

- "The new pattern applies forward-only to artifacts authored after this lands. Should I also retrofit the existing artifact(s) [<list>] in a follow-up commit?"
- Options: "Retrofit all" / "Retrofit named subset" / "Forward-only (no retrofit)".

Answer goes into the plan's "Out of scope" or a new "Retrofit scope" section verbatim. "Forward-only" → plan ships with explicit "Pre-existing X are not affected; retrofit deferred indefinitely". "Retrofit" → Phase Z appended at end of implementation phases.

**Skip when:** purely additive functionality (new commands not changing existing shapes), bug fixes (retrofit IS the work), plans explicitly limited to one artifact.

### 3.9 Verify gate (post-impl, pre-merge)

Per `.claude/commands/brehon-verify.md` + `feedback_brehon_verify_pre_merge.md`. Before queueing `bm-merge`, run `/brehon-verify <phase>`. Iterates plan §16a stories, runs each story's checkpoint command against the worktree branch, confirms each Brief-Scope output exists + matches its structural pattern, writes report at `.claude/PRPs/reports/<phase>-verify.md`.

| Outcome | Action |
|---|---|
| All stories ✓ | Report committed; advance to user gate 5 (merge confirm) |
| Phantom (task complete but expected output absent/empty) | Catch-fire — surface phantom names + structural patterns + expected-vs-actual |
| ✗ from checkpoint failure (output exists, command exits non-zero) | Catch-fire — "regression suspected; CR triage missed it"; file DQ pending |

Verify catches **phantom completions** (advisor reconciles brief Scope + plan §13 IMPLEMENT vs worktree branch). CR triage catches **regressions and quality findings**. Both stay; neither replaces the other.

### 3.9.1 Conformance-audit detection checkpoint

**Conformance-audit detection** (per `.claude/skills/brehon-conformance-audit/`):
Before queueing `bm-merge`, after `/brehon-verify` returns ✓, run the skill with
`target_scope = phase-diff <phase-branch>`. Tier-1 findings become §3 actions in the retro.
Update the per-phase metrics file at `.claude/PRPs/audit-metrics/<phase>.json`. Run
`compute-metrics.sh` for per-sub-phase calibration.

## 4. Cohort dispatch

Per `.claude/PRPs/templates/plan.template.md` §13 (`[P]` markers) + `feedback_parallel_cohort_dispatch.md`. When a §13 task carries `[P]` and is the next pending, advisor computes the **cohort** — consecutive `[P]`-marked tasks until a non-`[P]` boundary. Task 0 (pre-flight harness audit) is always non-`[P]`.

### 4.1 Cohort dispatch sequence

1. Read plan §13. Locate next pending task by id (smallest task whose impl commit is not on the phase branch).
2. Non-`[P]` (or Task 0) → queue alone via `mcp__junior-brehon__create_task`; wait for complete/failed.
3. `[P]` → walk §13 forward collecting consecutive `[P]` until non-`[P]` boundary or end-of-list. Collected list = cohort.
4. **YAML overlap check** (per `feedback_explicit_file_arrays_on_tasks.md`): parse FILES YAML (`creates:` + `modifies:` arrays). Pairwise intersect across cohort members. Non-empty intersection → degrade to serial. Surface: `cohort overlap detected: tasks <A>+<B> share <path> — degrading to serial`. No DQ filed; planner's `[P]` was wrong; retro flags it. Missing YAML on any member → back-compat: skip check, trust `[P]`. Trust YAML over `[P]` when they disagree.
4a. **`requires:` dependency check** (new — per `feedback_cohort_validation_dependency_check.md` 2026-05-11). For each cohort member, parse FILES YAML `requires:` array. For each `requires: - task: <N>` entry: verify task `<N>`'s impl commit is already on `phase-<phase>` (`git log phase-<phase> --grep "(task <N>)" --oneline | head -1` returns non-empty). If task `<N>` is NOT yet merged: **(a)** if `<N>` is also in this cohort, refuse the cohort — these tasks need bundling, not parallelism. Surface: `cohort refused: tasks <A>+<B> have circular requires: — planner must bundle or re-order`. **(b)** if `<N>` is in a prior cohort not yet merged, defer the cohort until `<N>` lands. Surface: `cohort deferred: task <M> requires task <N> not yet on phase branch`. Missing `requires:` field on a member = no cross-task dependency claimed; trust the planner's `[P]` marker alone (back-compat: pre-2026-05-11 plans). This step prevents the Cohort A / Cohort B-serial isolation-validation bug class (per v1-RT-r1 halt retro `ffa2876e3`).
5. **Budget check** (pre-Shape-G only; Shape G non-binding, cargo runs off-box): each `cargo check --workspace --features full` ~6 GB peak (EliteDesk cap `MemoryMax=10G`). `cohort_size × per_task_peak > 10 GB` → degrade to serial. Per `feedback_resource_budget_pre_queue.md`. Surface: `cohort degraded to serial: budget exceeded (<size> tasks × <peak> GB > 10 GB)`.
6. **Forbidden-window check** (§5.1): if any cohort task starts in a forbidden window, defer the entire cohort.
7. **Queue every cohort task simultaneously** via parallel `create_task` calls (single message, multiple tool uses). Each task gets its own worktree. Brief paths unique per task.
8. **Wait for all members to reach complete or failed** before next cohort. A failed member blocks advancement.
9. **On cohort completion** (all members complete + validated) → run §4.3 handover aggregation before computing next cohort.

### 4.2 Cohort dispatch refusals

- Never queue a cohort whose tasks have not all been clarified. Re-run `/brehon-clarify` if post-clarify edits introduced overlap.
- Never queue a cohort during a forbidden window, even partially.
- Never re-queue a cohort task that already shows running (Junior task IDs are unique per worktree; duplicate worktree + branch name).
- Never queue a `[P]` task whose IMPLEMENT files overlap a non-`[P]` task still running. `[P]` is a within-cohort disjointness promise, not across-boundary.
- Never queue a cohort with non-empty YAML intersection without first degrading to serial. Mechanical: `intersect(union(creates, modifies)_taskA, union(creates, modifies)_taskB) != ∅` → degrade.

### 4.3 Cohort handover aggregation

Per `feedback_handover_trailer_cohort_propagation.md`. Once all cohort members reach `complete` (and under Shape G, all paired `validate-pending` entries mutated to `result: "pass"` for both Phase 1 and Phase 2), populate the *next* cohort's brief §3a "Handover from prior cohort" before queueing.

1. For each cohort member's commit on `phase-<phase>`, parse `HANDOVER:` YAML trailer (`git log -1 --format=%B <sha>`). Missing trailer = degraded handover (note in polling output; not a catch-fire).
2. Aggregate into one block matching §3a schema in `impl-task-brief.template.md`:

```yaml
prior_cohort_tasks:
  - task: <N>
    commit: <sha>
    filesCreated: [...]
    filesModified: [...]
    keyDecisions: [...]
    notes: <verbatim from trailer>
  - task: <N+1>
    ...
```

3. For each next-cohort brief at `.claude/PRPs/briefs/<phase>-impl-<M>.md`, Edit §3a in place — replace `(none — first cohort)` or `(none — prior task non-[P])` with the aggregated block. Commit subject: `chore(advisor): inject prior-cohort handover for <next-cohort-tasks>`.
4. Push to `governance-v0`.
5. Proceed to next-cohort dispatch (§4.1 step 1).

**Skip if** the next cohort is empty (prior cohort was last before retro) — retro task reads §3a as `(none — last cohort)`. **Single-task cohorts** (one `[P]` followed by non-`[P]`) still aggregate — trailer is the unit of handover; cohort size doesn't change the rule.

### 4.4 Notes

**Plans without `[P]` markers** (legacy or planner judged no parallelism safe) → every task non-`[P]`, dispatched serially. Cohort logic does NOT broaden serial into accidental parallel — `[P]` must be explicit.

**Cohort dispatch under Shape G:** members enter `kind: "validate-pending"` simultaneously after their respective push (one workspace-check workflow run per cohort task on GitHub-hosted runners). Advisor dispatches one ci-watcher per pending entry. Cohort advancement waits for **all** Phase-1 members to reach `result: "pass"`. Multiple simultaneous failures → classify each independently per §5.3 (allowlist match → parallel fix-impl-tasks; non-allowlist → single catch-fire bundle). After all Phase-1 pass and daemon finalize-merges each into the phase branch, advisor raises ONE Phase-2 e2e `validate-pending` for the post-finalize phase-branch tip. Next-cohort advancement waits on Phase 2 e2e `result: "pass"` as well.

## 5. Validation, classification, recovery

### 5.1 Forbidden execution windows

EliteDesk shares cron-driven workloads (NAS backups, web-archive crawls, weekly review) with Brehon Junior tasks. Repeated OOM cascades (2026-04-27) confirm temporal isolation > spatial isolation. Source-of-truth: `homeserver/docs/troubleshooting-laptop-elitedesk.md` "Temporal isolation" section.

| Window (UTC) | Why |
|---|---|
| Daily 02:55–04:15 | NAS backup chain (03:00, 03:15) + web-archive `govie-search` (03:00, 03:30) |
| Sunday 01:55–02:35 | HSE crawl (02:00) + `junior-weekly-review.sh` (02:30) |
| Sunday 03:55–04:30 | `restore-drill.timer` (04:00) |
| Wednesday 03:55–04:15 | `web-archive govie-cdx` (04:00) — subset of daily |

**Recommended Brehon execution windows:** Primary 16:00–02:30 UTC (10.5 h, evening/overnight). Secondary 04:30–14:59 UTC (10.5 h, post-crawl, pre-evening).

**Advisor enforcement** — before queueing any new `impl-task`:

1. Compute next "safe" minute (end of current forbidden window).
2. Note deferral in polling output: `deferring <task-slug> until <HH:MM UTC>`.
3. Re-check on next poll. Queue once window closes. **No DQ for routine deferrals.**

Mechanical, not heuristic — read table + `date -u`.

**Subagent enforcement (defence in depth):** the `impl-task` subagent's task-0 pre-flight check refuses to start in a forbidden window, exits non-zero with `FORBIDDEN_WINDOW: <window>` (per `.claude/agents/impl-task.md`). Catches advisor-mistaken queues (e.g. cron table out of sync, DST edge case).

**Shape G note:** under Shape G (v1-validate-agent onward), cargo runs on GitHub-hosted runners — forbidden windows non-binding for Shape-G impl-task dispatch. Still binding for: (a) ad-hoc local cargo by advisor pre-plan-approval (§3.4 DoD smoke test), (b) pre-Shape-G plan dispatches (v1-JM-d and earlier), (c) any local diagnostic cargo authorised by user during a CR fix-in-PR cycle.

### When to override

Forbidden windows protect from contention, not absolute prohibition. If user authorises a forbidden-window run:

1. File a DQ entry citing the user's override.
2. Queue with brief note: "user-authorised forbidden-window override per DQ #<id>".

Do not silently queue inside a forbidden window without a DQ trail.

#### Cargo never runs on the EliteDesk worker

Per 2026-04-28 task #47 incident: `cargo check --workspace --features full` ran for >1 h with sustained OOM-cascade risk. Both validation modes route cargo OFF the worker:

- **Shape-G plans:** GitHub-hosted runners. impl-task pushes branch, raises `kind: "validate-pending"`; ci-watcher polls workflow, mutates entry.
- **Pre-Shape-G plans:** the **laptop** (advisor's CWD `C:\Users\barri\Developer\brehon-fork`). impl-task pushes branch, raises `kind: "validate-pending-laptop"` naming §15 DoD commands verbatim; advisor reads on next poll, runs each command sequentially, mutates the entry. See §5.2 below.

### 5.2 validate-pending-laptop handler

When a `kind: "validate-pending-laptop"` (or `*-laptop-e2e`) entry appears in `pending[]`, the advisor (laptop session) runs the §15 commands locally. Mutation shape, log-slice rules, kind enum, and §G4 fail handling: see `.claude/rules/decision-queue.md` §"ci-watcher mutation pattern" + §"Two-phase validation under Shape G" (mutation is identical; only the runner identity differs — `answered_by: "advisor-laptop"` instead of `"ci-watcher"`).

**Pre-flight (mandatory, before fetch):**

- Clean working tree (`git -C C:/Users/barri/Developer/brehon-fork status --short` empty); dirty → surface to user, do NOT auto-stash.
- `mkdir -p C:/Users/barri/.claude/logs/` (idempotent).
- Concurrent-cargo serialization: if another `validate-pending-laptop` is in-flight (`[P]` cohort fan-out), process this one behind it in DQ id order — two cargos on the same `target/` = lock + thrash.
- Docker Desktop check (e2e only): if `commands[]` includes `--features full` testcontainers paths, `docker ps` must return 0; not running → surface "start Docker Desktop or pick GH dispatch via Phase 2 e2e user gate". `cargo check`/`clippy`/`test --no-run` skip the check.

**Sequence:**

1. `git fetch origin <entry.branch>` then `git checkout origin/<entry.branch>` (detached-HEAD; no edits, just cargo source).
2. Run each command in `entry.commands[]` sequentially. `Bash` `run_in_background: true` for runs >5 min (`cargo-check.sh` ~8 min cold / 3-5 min warm; e2e ~26 min). Capture to `C:\Users\barri\.claude\logs\validate-laptop-<entry.id>-cmd-<n>.log`. Non-zero exit → stop chain.
3. Mutate the DQ entry in place per the canonical mutation shape (see decision-queue.md ref above). Failure stays in `pending[]` for §G4 triage; pass moves to `resolved[]`.
4. Commit + push to `governance-v0`. Subject: `chore(decision-queue): advisor-laptop mutated DQ #<id> — <pass|fail> validate-pending-laptop`.
5. On fail, run §G4 classifier (§5.3): allowlist → narrow fix-impl-task; non-allowlist → catch-fire.
6. `git checkout governance-v0 && git pull --ff-only origin governance-v0` (skip only if user wants laptop kept on worker branch for hand-debug).

**Phase 2 e2e (advisor-driven, off-Actions default — 2026-04-28 minutes-budget audit):** advisor raises the entry up front (`from: "advisor"`, `workflow_run_id: null`, `local_log_path: ".claude/runlog/e2e-<phase>-<sha>.log"`, `branch: "phase-v1-<phase>"`, `phase_task: <N>`, `result: null`). Subject: `chore(advisor): raise local e2e validate-pending for phase-v1-<phase> tip <sha>`. No ci-watcher dispatch (nothing on GH to poll). On bg cargo exit, advisor mutates directly: `answered_by: "advisor"`, `resolved_at`, `log_slice` from runlog tail (~150 lines failures block). **Escape hatch (explicit user request only):** `gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-<phase>` — entry reverts to pre-2026-04-28 shape (`workflow_run_id: <id>`, `local_log_path: null`); ci-watcher queued as for Phase 1.

**Windows invocation (mandatory — 2026-05-09 RCA):** use `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`. Never bare `cargo test` on Windows — libpq.dll requires the bat wrapper's vcpkg PATH setup; bash PATH export does not propagate to the Windows PE DLL loader. Never `-p lemmy_server --features full` — `lemmy_server` has no `full` feature; use `--workspace`. See `feedback_windows_e2e_requires_bat_wrapper.md` and RCA at `.claude/PRPs/reports/rca-phase2-e2e-invocation-failure-2026-05-09.md`.

### 5.3 §G4 classifier

Per `.claude/PRPs/plans/v1-validate-agent.plan.md` §4 watchpoint #7 + §10.9. When a `validate-pending` entry is mutated to `result: "fail" | "cancelled" | "timed_out"` and remains in `pending[]`, advisor reads `result`, `log_slice`, `failed_jobs`, applies the classifier.

**Cycle-count meta-rule (above the allowlist).** Per `feedback_plan_stub_uniformity_with_canonical_sibling.md`: count of prior fails with same `(error_class, file_basename)` for this cohort member ≥3 → **HARD REFUSAL, catch-fire regardless of allowlist match**. Cycles 1+2 classify normally below. Mechanism (parse log slice, append history entry, count tuples) lives in `~/.claude/commands/auto-phase.md` Phase 2 routing; durable record in `current_cohort.members[].error_class_history[]` per template at `.claude/PRPs/templates/auto-phase-state.template.json`.

// 2026-05-09 c-2 cycle-3 catchfire: 3 cycles same `(E0277, e2e.rs)` cost ~123 min before user-prompted re-plan; root cause was §13 stub shape, not recipe. Per session-retro-2026-05-09-cycle-3-catchfire-replan.md proposal #1.

**Allowlist (auto-queue narrow fix-impl-task, ≤3 file edits):**

| Failure signature | Auto-fix | Source lesson |
|---|---|---|
| `clippy::doc_lazy_continuation` warning | reword + mid-paragraph "and" | `feedback_clippy_doc_lazy_continuation_in_doc_comments.md` |
| `error[E0432]: unresolved import` | add the missing `use` per the suggestion | n/a (mechanical) |
| `warning: use of deprecated <api>` | replace with the suggested replacement | n/a (mechanical) |
| **4a** `error[E0277]: ?` couldn't convert `LemmyError`/`LemmyResult<T>` to `Box<dyn Error>` AND test fn returns `Result<(), Box<dyn Error>>` AND helpers all return `Result<T, Box<dyn Error>>` (Case B per lesson) | wrap each Lemmy-native call with **annotated** closure: `.map_err(\|e\| -> Box<dyn std::error::Error + Send + Sync> { format!("{e}").into() })?`. Bare `.map_err(\|e\| format!("{e}").into())?` will fail E0283 because `_` in `Into<_>` cannot resolve through abstract trait objects. | `feedback_lemmy_error_no_std_error.md` Case B |
| **4b** `error[E0277]: ?` couldn't convert `LemmyError`/`LemmyResult<T>` to `Box<dyn Error>` AND a v1-SL-* / v1-JM-* sibling fixtures module exists in the same file using `LemmyResult<()>` outer | flip the test fn signature to `LemmyResult<()>` AND flip ALL helper signatures in this module's fixtures mod to `LemmyResult<T>`. No `.map_err` bridges. Mirror the canonical sibling shape verbatim (Case A per lesson). | `feedback_lemmy_error_no_std_error.md` Case A |
| **4c** `error[E0277]: ?` propagation Send/Sync/Sized cascade (≥3 sites at once) — Case C symptom: test fn outer differs in Result type from helper outer | **HARD REFUSAL** — do NOT auto-queue a fix-impl task. Surface to user as a re-plan signal: type-shape uniformity is mandatory across a single test module; no mechanical bridge resolves Case C. The recipe family is wrong-shaped. | `feedback_lemmy_error_no_std_error.md` Case C |
| `error[E0277]: trait bound \`<T>: <Trait>\` not satisfied` where the lesson corpus has a citation | apply the recipe per the cited lesson | search `.claude/lessons/` for the failing trait + type before classifying |
| `clippy::map_err_ignore` (E0277-adjacent) | rename `\|_\|` → `\|_e\|` per `feedback_clippy_map_err_ignore_pattern_rename.md` (when authored) | mechanical |
| `error: cannot find macro \`<name>\` in this scope` | add the missing `use` from the macro's home crate | n/a (mechanical) |
| `error[E0599]: no method named \`<name>\`` (when method is on a re-exported trait) | add the missing `use` for the trait | n/a (mechanical, but verify the trait isn't intentionally hidden) |

For an allowlist match: author a narrow fix-impl-task brief at `.claude/PRPs/briefs/<phase>-fix-impl-<n>.md` containing failed-job log slice (≤200 lines), specific file:line cited by the lint, auto-fix recipe from source lesson (or mechanical replacement), hard cap "≤3 file edits". Dispatched as normal `[role:impl-task]` Junior task; resulting commit lands on phase branch and re-triggers the workflow.

**Callsite-enumeration discipline (per `feedback_fix_impl_enumerate_all_callsites.md` 2026-05-11):** when the failure signature is a struct-shape change (E0063 missing-field on `<Type>` initializer; renamed/added field; trait-impl signature change) and the compile error points at K specific call sites, the advisor MUST `rg "<Type>" crates/ tests/` to enumerate the FULL set of N callsites BEFORE authoring the fix-impl brief. The brief lists all N callsites; the file-edit cap is the count of distinct files containing those callsites (NOT the ≤3 default — that cap was designed for clippy/unused-import patterns, not struct-shape changes). If N > 10 callsites or > 5 files, the change is no longer "narrow mechanical" — catch-fire to user with the enumeration list so user can decide whether to extend the brief or split the fix across multiple commits. Without enumeration, fix-impl-1 patches only the compile-error-cited K sites and the workflow fails AGAIN on the next N-K callsites (v1-RT-r1 fix-impl-1 DQ #205 incident: brief covered 2 sites; workspace check FAILED with same E0063 on 8 more callsites, forcing fix-impl-2).

**Pre-push cargo-check discipline (per `feedback_fix_impl_pre_push_cargo_check.md` 2026-05-13):** mechanical fix-impl briefs MUST include a §4 Constraint requiring `bash scripts/brehon/cargo-check.sh --workspace --features full` (or `.bat` on Windows worker) BEFORE the worker pushes the worker branch. Non-zero exit → patch in same commit (if in-scope) OR file `kind: "blocker"` DQ (if out-of-scope). NEVER `#[allow]`-spam to bypass. Without this gate, an adjacent regression class (typically unused-import surfacing after a struct-field pad — e.g. v1-RT-r1 fix-impl-3 incident: `seed_founders/main.rs` import unused in non-test build after fix-impl-2 padded the consumers) costs a full ci-watcher cycle + fix-impl-(N+1) recovery. Local `cargo check` is ~30s warm; one extra ci-watcher cycle is ~5 min. Net positive on every cycle.

**Non-allowlist (catch-fire to user):**

| Trigger | Action |
|---|---|
| Compile errors (any `error[E*]` other than `E0432`) | Catch-fire |
| Test failures (panics, assertion fails, e2e flakes, testcontainers issues) | Catch-fire |
| Timeout / OOM / runner death | Catch-fire |
| Any failure whose log slice doesn't match an allowlist row | Catch-fire |
| Conformance-audit Tier-1 finding on `crates/apub/activities/src/governance/**.rs` OR `crates/api/api/src/governance/**.rs` OR `crates/db_schema/src/source/governance/**.rs` | **HARD REFUSAL — catch-fire to user** with audit report + suggested per-axis fix. NOT auto-fix; human-in-the-loop decides. (Per `feedback_mirror_phase6_convention_in_same_file.md`.) |

Surface as: "validate-failed on `<branch>` (workflow run `<id>`): non-allowlist failure. Failed jobs: `<failed_jobs>`. Log slice attached. Surfaced to user — no auto-fix attempted."

The allowlist is **conservative by design** (per `feedback_principles_not_rules.md`). Grow only on retro evidence — if a CR-triage cycle classifies a non-allowlist as "this could have been auto-fixed", record in retro §5 watch-items and add to next sub-phase's plan if pattern reproduces.

#### Mandatory verbatim §G4 row in fix-impl briefs (anti-paraphrase gate)

When a fix-impl-task brief's triggering DQ matches an allowlist row above, the brief's §2 Scope MUST contain a **verbatim block-quote of the matched row text — both columns (Failure signature + Auto-fix recipe + Source lesson) — copy-pasted as a markdown blockquote (`> ...`) BEFORE any file:line context.** Canonical recipe text is the contract; paraphrase is a process miss even if meaning preserved.

Blockquote shape (matches the table row literally):

```markdown
## 2. Scope

### 2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

> | Failure signature | Auto-fix | Source lesson |
> | <row text 1> | <row text 2> | <row text 3> |
```

After the blockquote, brief MAY add file:line context, line-by-line diff targets, acceptance criteria — but the recipe text is the contract Junior implements against. If the rest of the brief contradicts the blockquote, the blockquote wins (Junior hard refusal: stop, raise `kind: "blocker"` DQ citing this gate).

**Detection:** an advisor commit adding `.claude/PRPs/briefs/*-fix-impl-*.md` matching an allowlist row but lacking the verbatim §2 blockquote is a process miss — retro flags it. Future PostToolUse hook on brief Write/Edit can scan §2 for the literal blockquote (not yet implemented).

**Does NOT apply** to non-allowlist fix-impl briefs (catch-fire with hand-authored recipe). Gate prevents paraphrase drift on mechanical recipes, not user-driven fixes.

// 2026-05-09 c-2 cycle-2 catch-fire: fix-impl-1 brief cited canonical lesson in §3 but paraphrased in §2 (signature flip + forbid `.map_err`, instead of canonical "signature stays Box<dyn Error> + add `.map_err`"). Junior #158 followed brief literally. Cost: ~17 min. Verbatim copy-paste makes "I read the row but prescribed something different" structurally impossible.

### 5.4 DQ triage decision tree

For per-kind routing (the `(blocker, pending)` / `(validate-pending, pending)` / etc matrix), see `.claude/rules/decision-queue.md` §"Polling-loop routing per kind". This file owns only the advisor-side commit-subject pattern + the surfacing rule:

When a new pending entry appears in `decision-queue.json`:

1. Read `question`, `options`, `context`.
   - **Falsifiable-hypothesis gate (structural-fix DQs):** if the entry proposes a structural fix (TypeScript patch, harness change, daemon code-path edit, prompt rewrite) AND names a specific code path (file / function / prompt / script) as the defect site, the DQ's own RCA is a hypothesis, NOT a contract. Run a ≤30-min falsification pass BEFORE picking a routing path in step 2: (a) `grep` the named code path for the suspect operation; (b) read the relevant subagent's task log for the actual tool calls issued; (c) check `who` / `last` / `git reflog` on the named host or worktree. If the named code path does NOT contain the suspect operation, surface the falsification to the user via `AskUserQuestion` before any structural work begins — do NOT advance to step 2's advisor-answer / catch-fire / user-relay branches assuming the DQ's premise. Per `.claude/lessons/feedback_falsifiable_hypothesis_before_structural_fix.md` (DQ #338 incident 2026-05-21: named `daemon's finalize step` as defect; daemon code had zero `reset --hard` calls; real vector was a lane-agent ssh-reset from the laptop; ~3 hours of option-a investigation displaced).
2. Decide: **advisor-answer** (clear evidence + defensible answer; write `answer` and `answered_by: "advisor"`) / **catch-fire** (ADR violation, hard-refusal, process breach; stop loop, surface DQ id + cited rule) / **user-relay** (judgment-heavy: visible-to-others, ADR-affecting, scope change; surface one-screen summary; record user reply with `answered_by: "user"` + verbatim wording in `answer`).
3. Commit + push the answer. Subject MUST match `^(chore|docs)\((advisor|decision-queue)\)` per `.claude/rules/decision-queue.md` Attribution integrity §Detection. Narrow form: `chore(advisor|decision-queue): answer DQ #<id>`.

### 5.5 Retro-bypass observability

Per RLS-PMD review §4.7 + autonomy-readiness criterion 5.2 + `.claude/PRPs/plans/v1-rls-r1.plan.md` Task 7. The Stop hook `.claude/hooks/retro-check.sh` fail-open path (3-attempt cap, load-bearing for true loops) emits a JSONL `retro_bypass` record to `.claude/governance-log/retro-bypass.jsonl` on every fail-open. Fields per the kind registry at `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md`.

**Consumer:** the JSONL trail is consumed by future audit reads (weekly-review Step 2c is the retro-corpus sweep over `.claude/PRPs/reports/*.md`, not the JSONL; a dedicated JSONL-rate audit step would be added in a future sub-phase if a `retro_bypass` rate trend becomes load-bearing). **Autonomy signal:** the rate of `retro_bypass` entries per week should be monotonically decreasing. Rising rate → calibration-honesty regression; surface in the next phase retro.

**Advisor-side action:** none required at session-start (the trail is passive). At retro time, the rate trend is part of the four-role retro signals (Advisor role) per `feedback_four_role_retro_signals.md`. See `feedback_retro_bypass_governance_log.md`.

### 5.6 Catch-fire procedures

Stop the loop and surface to user immediately. Include catch-fire reason + cited rule in the message.

| Trigger | Cited rule | Action |
|---|---|---|
| Junior subagent ignores hard refusals (e.g. impl-task writes `crates/**` from non-authorising brief) | `.claude/agents/<name>.md` | Surface breach + agent path |
| `answered_by: "advisor"` in commit whose subject is NOT `^(chore|docs)\((advisor|decision-queue)\)` | `decision-queue.md` Attribution integrity | Surface SHA + subject |
| Subagent commits to `governance-v0` or `main` directly | `phase-branch.md` | Surface SHA + branch |
| `bm-task` opens PR into `main` instead of `governance-v0` | `phase-branch.md` | Surface PR # |
| Phase branch has uncommitted state when Junior reports complete | Junior finalize discipline | Surface `git status --short` |
| Rust-analyzer-lsp missing on EliteDesk daemon AND task depended on `LSP` | `.claude/agents/<role>.md` | Surface task id |
| Workflow run exceeds 60-min ci-watcher cap → `result: "timed_out"` | `.claude/agents/ci-watcher.md` | Surface run id + elapsed; do NOT auto-rerun |
| ci-watcher's `gh run watch <id> --exit-status` returns exit code not in `.claude/agents/ci-watcher.md` "Empirical exit-code table" | classifier-miss | Surface exit code + run id + `gh run view` snapshot; record new pair, update table at retro |

## See also

- `.claude/agents/<name>.md` — subagent contracts (planning, impl-task, bm-task, ci-watcher).
- `.claude/rules/branch-manager.md` + `.claude/commands/bm/<verb>.md` — BM mechanics, file-ownership, autonomy bounds.
- `.claude/rules/decision-queue.md` — DQ schema, attribution, per-kind routing.
- `.claude/rules/auto-phase.md` — `/auto-phase` state machine that compiles §3.1 + §4.1 + §5.3.
- `.claude/rules/pmd-search-strategy.md` — PMD search modes referenced by §2.3.
- `homeserver/.claude/advisor-context-phase-<N>.md` — phase-specific texture (session-start read).
