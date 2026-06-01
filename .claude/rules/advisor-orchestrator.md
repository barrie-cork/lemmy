# Advisor as orchestrator (persistent session)

The advisor session orchestrates one Brehon sub-phase end-to-end via Junior subagents (`planning`, `impl-task`, `bm-task`, `ci-watcher`). Meta-oversight only — never authors content. See `CLAUDE.md` "Four-role model" and `feedback_brehon_autonomy_goals` (autonomous + reliable + slow-OK + model-efficient). Polling is the trigger surface — no Telegram, n8n, or RemoteTrigger.

## 1. Polling loop

- **Session-start CWD check** (per `.claude/rules/multi-lane-worktree.md` 2026-05-11): run `pwd && git branch --show-current && git worktree list` at session start. If another active worktree exists on a different `phase-v1-*` branch, verify this session's CWD matches the intended lane. Lane-dedicated worktrees live at `C:/Users/barri/Developer/brehon-fork-<lane>`; the canonical `brehon-fork` checkout is reserved for `governance-v0` meta-edits (rules / lessons / templates / briefs on trunk). Lane-dedicated sessions write phase-branch DQ entries; canonical sessions do not.
- **Surface-first ritual (2026-05-22, post-RT-r2 session boundary incident):** when ANY of these hold at session start — (a) the UserPromptSubmit DQ-pending hook reports pending > 0, (b) `git worktree list` shows ≥2 worktrees, (c) the `session-start-multi-lane-check.sh` hook emitted a WARN — the FIRST user-visible response MUST be a one-line lane status: `lanes: <CWD>:<branch> active; other-active: <list-or-"none">`, BEFORE any task-execution response or tool call, even when inherited stdout frames the next action — include the full absolute CWD path, not just the branch name, so the reader can immediately distinguish canonical (`brehon-fork`) from lane (`brehon-fork-<lane>`) without opening a terminal. Rationale + cf93b7ba6 incident: `.claude/refs/advisor-orchestrator-incidents.md` §"Surface-first ritual".
- Cadence: every ~10 min call `mcp__junior-brehon__list_tasks` (status only).
- On status transition: `show_task` for the changed task; `git fetch origin`; read `.claude/decision-queue.json`; triage new pending entries; queue next per stage shape (§3.1).
- Read the plan once after planning ships. Read DQ on every poll only if `git fetch` returns new commits. Memory injection (Glob `.claude/lessons/`, PMD search) at session start, not per poll.
- No state change → output is "no change". Nothing else loaded.
- Single-task focus → prefer `/start-brehon --fast <N>` (5 probes). If DQ pending > 0, escalate to `/check-dq`.
- DQ on a non-trunk branch → use `scripts/brehon/git-show-json.sh <ref> <path>` then `python -c "import io, json; ... io.open(r'<path>', encoding='utf-8') ..."`. Never `git show <branch-with-slashes>:<path>` directly. Per `feedback_windows_bash_python_git_show_tmp_traps.md`.
- Reconciling DQ on an in-flight phase (session start, long-poll resume, retro audit) → use `scripts/brehon/resolve-dq-canonical.sh <phase>`. Unions phase-branch DQ with open worker-branch DQs; dedupes by id (worker-branch wins). // 2026-05-09 c-2: laptop saw pending=0 while live advisor saw pending=2 because DQ #164+#165 lived only on worker-159 tip.
- **SessionStart canonical-PMD guard (post-v1-rls-r1):** the tracked `.claude/hooks/pmd-canonical-guard.sh` runs at every session start in lanes wired per the bootstrap checklist (`feedback_phase_lane_worktree_bootstrap_checklist.md` step 6). Surfaces as a stderr WARN on lane drift, exit 0 always — does NOT block. Per `.claude/rules/pmd-invariants.md` invariant #5.
- **SessionStart multi-lane check (post-RT-r2 boundary incident 2026-05-22):** the tracked `.claude/hooks/session-start-multi-lane-check.sh` runs at every session start in wired lanes. Reads `git worktree list`; for each non-CWD worktree on a `phase-v1-*`/`phase-v2-*`/`phase-brehon-*` branch, WARNs if the tip advanced in the last 30 min (heuristic for "another session is driving"). WARN-not-FAIL (exit 0 always); lands in SessionStart system-reminders before the first tool call. Pairs with the surface-first ritual (hook = mechanical signal, ritual = prose response). FP/FN taxonomy + threshold rationale (`SESSION_START_MULTI_LANE_THRESHOLD`): `.claude/refs/advisor-orchestrator-incidents.md` §"SessionStart multi-lane check".
- **Pre-compact handover discipline (post-v1-retro-followups-r1, 2026-05-22):** any advisor session likely to span `/compact`, session-end, or context truncation MUST author a self-contained handover at `.claude/PRPs/handovers/<phase>-<scope>-<date>.md` BEFORE the boundary — readable with zero conversation context (current sub-phase, state-machine stage, last commit on the relevant branch, next concrete action, cross-session deps: DQ pending ids, concurrent activity). Commit + push the handover (always on `governance-v0`, even when work is on a phase branch) BEFORE invoking `/compact`. Validation example: `.claude/refs/advisor-orchestrator-incidents.md` §"Pre-compact handover discipline".
- **Telegram completion hook check (post-v1-quality-r3b):** at session start, call `mcp__junior-brehon__list_hooks`. If hook ID 1 is absent (daemon restarts wipe hooks), recreate it via `mcp__junior-brehon__create_hook` with the same payload documented in `feedback_daemon_telegram_completion_hook.md` (✅/❌ on done/failed). Per `feedback_daemon_telegram_completion_hook.md`.

## 2. Brief authoring

### 2.1 Junior task description template

Every Junior task is preceded by a brief at `.claude/PRPs/briefs/<phase>-<role>-<n>.md`, committed on the branch the worker forks from (`base_branch`) **BEFORE** `create_task` — the brief must be reachable at that ref at task-spawn time. **Per-role brief location + the Mode-A/Mode-B impl-task procedures (SSH trunk→phase sync) are in `.claude/refs/auto-phase.md` §"Brief location per role + lane mode (canonical detail)".** Quick rule: planning / bm-* / ci-watcher briefs commit on `governance-v0`; impl-task briefs must be visible on `phase-<X>` (Mode A: author on the phase branch directly; Mode B: author on trunk + SSH-merge into the phase branch).

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
| `crates/server/tests/e2e.rs` (any edit, any size) | `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`. **When a v1-SL-* or v1-JM-* fixtures sibling module already exists in the same file, mirror its error-shape case (A or B) verbatim per `feedback_lemmy_error_no_std_error.md` case enumeration. Pick by reading sibling at the cited line range BEFORE authoring the brief — canonical-schema-first gate.** **Uniqueness gate (pre-dispatch):** for every `old_string` anchor the brief will use as an Edit target, run `grep -c '<anchor>' crates/server/tests/e2e.rs` and confirm the result is `1`. If > 1, revise the anchor to be unique before queuing. Per retro-harvest-2026-05-31 Tier-1 #2 (2× anchor-collision incidents). |
| `crates/server/tests/e2e.rs` (≥2 edits in this task or cohort) | + `feedback_fix_impl_pre_locate_e2e_anchors.md` (pre-locate verbatim anchors; the canonical e2e-edit-hang-prevention lesson — the historically-cited `feedback_junior_worker_e2e_edit_hang.md` slug was never authored; repointed 2026-05-29 per v1-rt-r3-followup retro item #4) |
| `crates/server/tests/e2e.rs` AND brief is a fix-impl (filename matches `*-fix-impl-*.md`) | + `feedback_fix_impl_pre_locate_e2e_anchors.md` (pre-locate verbatim `old_string`/`new_string` anchors; subject to template §2.0 scope gate: ≤150 lines, ≤2 file edits, ≤2 Edits/file). 2× recurrence at v1-RT-r3 Task 4 cycle (DQ `a3d0e9941441-033`) + fix-impl-2 dispatch (Junior #479). |
| `crates/db_schema/migrations/**` (any new migration) | `feedback_lemmy_migration_runner.md`, `feedback_postgres_jsonb_canonicalization.md` (if JSONB) |
| Any new test under `crates/*/tests/**` returning `Result<(), Box<dyn Error>>` | `feedback_lemmy_error_no_std_error.md` |
| Any handler under `crates/api/**/src/**` doing 2+ DB writes | `feedback_multi_write_handlers_need_transactions.md` |
| Any new file under `crates/api/api/src/governance/**` that loads `ModerationCase` from DB and matches on `case.status` before proceeding | `feedback_governance_type_state_handlers.md` |
| Any `#[cfg(feature = "full")]` gate | `feedback_features_full_workspace_only.md`, `feedback_features_full_p_crate_incompatible.md` |
| Any `pg_advisory_xact_lock` or void PG function call | `feedback_pg_advisory_xact_lock_void_decode.md` |
| Any newtype under `crates/db_schema/src/newtypes/` | `feedback_newtype_locations_lemmy_db_schema_vs_file.md` |
| Any clippy fix involving `-D warnings` + new code | `feedback_clippy_test_style.md`, `feedback_clippy_rerun_after_fix.md` |
| Any `scripts/brehon/cargo-*.bat\|sh` edit | `feedback_wrapper_script_flag_silence.md`, `feedback_pq_sys_wrapper_env_propagation.md` |
| Any `.gitignore` / `.git/info/exclude` / hook addition | `feedback_settings_local_json_worktree_bootstrap.md` |

Brief commit body lists which mandatory lessons fired and why (one line each). The §2.3 hybrid search still runs after the table check — catches non-mechanical / cross-cutting lessons. // 2026-05-09 c-2 fix-impl-1 lapse: same E0277 LemmyError class as cycle-1; brief omitted lesson. Maintenance: when a new mechanical-fix pattern enters the §G4 allowlist (§5.3) and correlates with a file class, add a row here.

**Pre-Shape-G validate-pending-laptop constraint (mandatory for all impl-task briefs under pre-Shape-G plans):** Every impl-task brief §4 MUST include: "Write the `validate-pending-laptop` DQ entry with `commands: [\"./scripts/brehon/cargo-check.sh --workspace --features full\"]`, commit + push, then **stop**. Do NOT run `cargo-check.sh` yourself — validation is delegated to the laptop advisor." Workers running cargo on the daemon cause file-lock contention across concurrent cohort members (v1-RT-r5 Cohort A: ~45 min serialized wait). Lesson: `feedback_validate_pending_laptop_write_then_stop.md`.

### 2.5 Plan §5 complexity-score awareness

When the next pending §13 task is cargo-class (DoD names `cargo check`, `cargo clippy --workspace`, or `cargo test --workspace`) AND plan §5.1 complexity score `> 8`:

1. Confirm planner's split-or-proceed DQ resolved (look in `.claude/decision-queue.json` resolved entries from `planner` referencing this plan).
2. Dominant factor = e2e edits (≥2) → append `feedback_fix_impl_pre_locate_e2e_anchors.md` to §3 Required reading (the canonical e2e-edit-hang-prevention lesson; the old `feedback_junior_worker_e2e_edit_hang.md` slug was never authored — repointed 2026-05-29).
3. Dominant factor = migrations (≥2) AND pre-Shape-G → expect cargo+migration peak ~6 GB on the laptop; note in polling output if low battery. Per `feedback_complexity_score_pre_split.md`.

Mechanical: read score, read top factor, add citation. No DQ, no escalation.

## 3. Stages and gates

### 3.1 Stage-shape orchestration

Per the c-inherited-dragon plan's stage map. Advisor knows what to queue next on each completion. Full state-transition contract (with Phase 1/Phase 2 multi-paragraph detail): `.claude/refs/auto-phase.md` §"Stage-shape orchestration (canonical contract)".

- **Brief authored, no planning task yet** → `/brehon-clarify` → resolve clarify-DQ → queue planning.
- **Planning complete** → §3.4 + §3.5 + §3.5a → gate 1 (plan approval) → queue `bm-cut`.
- **bm-cut complete** → §4 cohort dispatch.
- **impl-task complete (pre-Shape-G, ≤v1-JM-d)** → cohort barrier → next cohort or `chore(lint):` → `bm-pr`.
- **impl-task complete (Shape G, ≥v1-JM-e)** → Phase 1 ci-watcher (workspace check) → Phase 2 e2e (advisor-driven). Cohort advancement waits on BOTH `result: "pass"`. Detail in refs.
- **ci-watcher complete** → `pass` advances; `fail/cancelled/timed_out` → §5.3 §G4 classifier.
- **All §16a stories `[done]`** → `/brehon-verify` → phantom = catch-fire; else `bm-pr`.
- **bm-pr complete** → CodeRabbit polls → `bm-poll-cr`.
- **bm-poll-cr complete** → `bm-triage` (draft auto).
- **Triage drafted** → gate 3 (CR triage) → fix-in-PR impl-tasks.
- **bm-pr complete → before gate 5** → merge-forward check (`git log origin/governance-v0 ^phase-v1-<phase>`); non-empty → checkout + merge + push.
- **Junior task on `governance-v0` reports `done`** → the daemon's finalize-merge is **daemon-local-first, origin-push-second**. Look in this order, NOT origin-first: (1) `ssh homeserver "cd /srv/brehon-fork && git log governance-v0 -1 --oneline"` (the merge lands here first), (2) `git ls-remote origin governance-v0` (did the daemon push yet?). Daemon-local ahead of origin = daemon hasn't pushed → `ssh homeserver "cd /srv/brehon-fork && git push origin governance-v0"`, then pull locally. Checking origin first shows a stale pre-merge tip and triggers a multi-probe hunt. Per `feedback_finalize_merge_where_to_look_first.md`.
- **No critical findings open** → `/brehon-verify` ✓ → gate 5 (merge confirm) → `bm-merge`.
- **bm-merge complete** → author retro → gate 6 (retro sign-off) → `/brehon-phase-transition`.

Advisor never auto-merges or auto-resolves ADR-affecting DQ.

### 3.2 Mandatory user gates

Six gates, never skipped (goal #3: slow-OK):

1. **Plan approval** — after planning ships and §3.4 DoD smoke test passes.
2. **Judgment-heavy DQ** — ADR-affecting / scope-changing / visible-to-others impact. Use `answered_by: "user"` after relay.
3. **CR triage approval** — after `bm-poll-cr` + `bm-triage` draft. Surface four-bucket counts. When any fix-impl brief in the triage queue targets a known-fragile pattern (e2e.rs ≥2 edits, multi-helper refactor), the AskUserQuestion MUST include the failure-class signature and observed retry rate so the user authorises with full risk context. 2× threshold met: v1-RT-r3 Task 4 (#474–#477) + fix-impl-2 (#479) both burned >30 min before catch-fire with no upfront risk signal.
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

### 3.5a MiniMax trial designation (pre-plan-approval)

After §3.5 watchpoint gate, before surfacing to user. Walk every `[role:impl-task]` task in the plan's §13 against the five qualifying criteria in `.claude/PRPs/briefs/minimax-m27-trial-1.md` §0.1 (not-e2e, MIRROR-ref-heavy, ≤2 files, cargo-gated DoD). For each task record one row in the §3 designation table in the runbook — `✅` or `❌` with a one-line reason on miss. Count the cumulative total (including prior-phase rows already in the table). Report the count in the plan-approval surface: `MiniMax trial: N/5 qualifying tasks accumulated`.

When cumulative count reaches ≥5: note in the plan-approval surface that the trial fires this phase — dispatch both arms per §2.3 of the runbook alongside the real impl tasks. When count < 5: no action beyond recording the rows.

Mechanical — no DQ, no user gate. The only output is the updated §3 table row(s) and the count line in the plan-approval surface.

### 3.6 Canonical-schema-first gate (pre-spec-authorship)

Per `feedback_read_canonical_before_writing_spec.md`. Before authoring any new `*.md` under `.claude/{rules,commands,lessons,PRPs/templates}`, `Glob` + `Read` 1-2 sibling instances first. Cite the canonical example in the new file body or commit body. A commit that adds such a file without citation is a process miss; retro flags it. `grep '^##' <existing>` is always worth the 2-second read.

**Narrative-to-refs at author time (per `feedback_rule_narrative_to_refs_at_author_time.md`):** when *editing* an always-load `.claude/rules/*.md` file, keep the rule *statement* inline (terse imperative + trigger) but author any incident narrative, FP/FN taxonomy, locked-decision rationale, or worked example (>3 lines) in `.claude/refs/<rule>-incidents.md` (or an existing refs file) with a one-line pointer. Section *headings* stay in the rule file (`.pi/` + skills cite them by name — moving a heading breaks the dual-harness contract). The always-load corpus is ~35% of the 200K budget; growth-discipline at author time is the durable lever (one-time extraction of the already-trimmed corpus yields ~1% — 2026-05-29 harness-audit).

**Source code extension (Tier-2 harvest 2026-05-31):** the same read-sibling-first discipline applies before authoring any new source file under `crates/` — read 1-2 sibling files in the same module directory before writing the new file. This catches crate-local conventions (import style, error propagation, module pub patterns) that aren't captured in lessons.

### 3.7 Dogfood gate + 3.8 Schema-retrofit gate

Both gates fire only when authoring new slash commands or when plan-mode produces a new artifact shape. Procedure: `.claude/refs/advisor-narrow-gates.md`.

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

**Cross-lane total cap (hard, pre-dispatch gate):** Before queuing ANY Junior task, call `list_tasks(status="running")` and count. If count ≥ 2 → defer; do NOT dispatch until a slot frees. This applies even to size-1 cohorts and even when the running tasks are on different lanes/phases — all daemon worktrees share one `.git/index.lock`. Cap is 2 total, not 2 per lane. Per `feedback_cohort_shared_git_index_contention.md` §"Cross-lane total cap". (v1-RT-r5 incident: 3 concurrent workers → ~2h lock contention vs expected ~30 min.)

Per `.claude/PRPs/templates/plan.template.md` §13 (`[P]` markers) + `feedback_parallel_cohort_dispatch.md`. When a §13 task carries `[P]` and is the next pending, advisor computes the **cohort** — consecutive `[P]`-marked tasks until a non-`[P]` boundary. Task 0 is always non-`[P]`. The dispatch is gated by five checks that can degrade a cohort to serial or defer it: YAML file-overlap, `requires:` dependency, pre-Shape-G memory budget, shared-`.git/index.lock` hazard (daemon single-`.git/`, cohort ≥3), and forbidden-window. Cohort members are queued simultaneously, advance only when ALL reach `complete` + validated, then a handover-trailer aggregation seeds the next cohort's brief §3a.

**Full mechanism — the 9-step sequence, the five degrade/refuse checks verbatim, handover aggregation, and the Shape-G cohort flow — is in `.claude/refs/auto-phase.md` §"Cohort dispatch mechanism (canonical detail)".** It fires only inside `/auto-phase` `impl-cohort-N` (which reads it JIT). Read that section before acting on any cohort decision outside `/auto-phase`.

### 4.1 Cohort dispatch sequence

The 9-step sequence (locate next pending → `[P]`-walk → YAML overlap → `requires:` → budget → index.lock hazard → forbidden-window → simultaneous queue → barrier on all-complete) lives verbatim in `.claude/refs/auto-phase.md` §"Cohort dispatch sequence (steps 1–9)". (Heading kept resident — cited by name from `~/.claude/commands/auto-phase.md`.)

## 5. Validation, classification, recovery

### 5.1 Forbidden execution windows

EliteDesk shares cron-driven workloads (NAS backups, web-archive crawls, weekly review) with Brehon Junior tasks. Repeated OOM cascades (2026-04-27) confirm temporal isolation > spatial isolation. Full window table + override + cargo-routing detail: `.claude/refs/advisor-validation.md` §"Forbidden execution windows".

**Shape G note:** under Shape G (v1-validate-agent onward), cargo runs on GitHub-hosted runners — forbidden windows **non-binding** for Shape-G impl-task dispatch. Binding only for: (a) ad-hoc local cargo by advisor pre-plan-approval (§3.4 DoD smoke test), (b) pre-Shape-G plan dispatches (v1-JM-d and earlier), (c) any local diagnostic cargo authorised by user during a CR fix-in-PR cycle.

**Advisor enforcement** (when binding) — before queueing: compute next safe minute, note deferral (`deferring <task-slug> until <HH:MM UTC>`), re-check on next poll. Mechanical, no DQ for routine deferrals. **Subagent defence-in-depth:** `impl-task` task-0 pre-flight refuses with `FORBIDDEN_WINDOW: <window>` if advisor mis-queues.

#### When to override

User may authorise a forbidden-window run. Procedure (file DQ citing the user's override; queue with the `forbidden-window-override: DQ #<id>` dispatch note so the subagent skips its time check): `.claude/refs/advisor-validation.md` §"When to override". (Anchor kept resident — `.pi/skills/impl-task/SKILL.md` cites this heading by name.)

### 5.2 validate-pending-laptop handler

When a `kind: "validate-pending-laptop"` (or `*-laptop-e2e`) entry appears in `pending[]`, the advisor runs the §15 commands locally. Full pre-flight, sequence, Phase-2 e2e advisor-driven flow, escape hatch, Windows invocation: `.claude/refs/advisor-validation.md` §"validate-pending-laptop handler".

Mutation shape, log-slice rules, kind enum, §G4 fail handling: `.claude/rules/decision-queue.md` §"ci-watcher mutation pattern" + §"Two-phase validation under Shape G" (mutation identical; `answered_by: "advisor-laptop"` instead of `"ci-watcher"`).

### 5.3 §G4 classifier

Per `.claude/PRPs/plans/v1-validate-agent.plan.md` §4 watchpoint #7 + §10.9. When a `validate-pending` entry is mutated to `result: "fail" | "cancelled" | "timed_out"` and remains in `pending[]`, advisor reads `result`, `log_slice`, `failed_jobs`, applies the classifier.

**Cycle-count meta-rule (above the allowlist — resident SAFETY policy).** Per `feedback_plan_stub_uniformity_with_canonical_sibling.md`: count of prior fails with same `(error_class, file_basename)` for this cohort member ≥3 → **HARD REFUSAL, catch-fire regardless of allowlist match**. Cycles 1+2 classify normally. Mechanism (parse log slice, append history entry, count tuples) lives in `~/.claude/commands/auto-phase.md` Phase 2 routing; durable record in `current_cohort.members[].error_class_history[]` per `.claude/PRPs/templates/auto-phase-state.template.json`. // 2026-05-09 c-2 cycle-3 catchfire: 3 cycles same `(E0277, e2e.rs)` cost ~123 min before user-prompted re-plan.

**Routing (resident SAFETY policy):** failure log_slice matches an **allowlist** row → auto-queue a narrow fix-impl-task (≤3 file edits, or the callsite-enumeration count for struct-shape changes). Anything else — any `error[E*]` except `E0432`, any test/panic/e2e failure, timeout/OOM/runner-death, a conformance-audit Tier-1 governance finding, or **any log_slice that does not match an allowlist row** → **catch-fire to user, no auto-fix attempted.** The allowlist is conservative by design (`feedback_principles_not_rules.md`); grow only on retro evidence.

**Full mechanism — the allowlist + non-allowlist recipe tables, the callsite-enumeration discipline, the pre-push cargo-check discipline, and the mandatory-verbatim-§G4-row anti-paraphrase gate — is in `.claude/refs/auto-phase.md` §"§G4 classifier — allowlist + recipe tables (canonical detail)".** It fires only on a `validate-pending` failure (mid-orchestration); the Phase-2 routing reads it JIT. Fix-impl briefs copy the matched allowlist row verbatim from that refs table (the anti-paraphrase gate's verbatim-source). Read it before classifying any validate-fail outside `/auto-phase`.

### 5.4 DQ triage decision tree

For per-kind routing (the `(blocker, pending)` / `(validate-pending, pending)` / etc matrix), see `.claude/rules/decision-queue.md` §"Polling-loop routing per kind". This file owns only the advisor-side commit-subject pattern + the surfacing rule:

When a new pending entry appears in `decision-queue.json`:

1. Read `question`, `options`, `context`.
   - **Falsifiable-hypothesis gate (structural-fix DQs):** if the entry proposes a structural fix (TypeScript patch, harness change, daemon code-path edit, prompt rewrite) AND names a specific code path (file / function / prompt / script) as the defect site, the DQ's own RCA is a hypothesis, NOT a contract. Run a ≤30-min falsification pass BEFORE picking a routing path in step 2: (a) `grep` the named code path for the suspect operation; (b) read the relevant subagent's task log for the actual tool calls issued; (c) check `who` / `last` / `git reflog` on the named host or worktree. If the named code path does NOT contain the suspect operation, surface the falsification to the user via `AskUserQuestion` before any structural work begins — do NOT advance to step 2's advisor-answer / catch-fire / user-relay branches assuming the DQ's premise. Per `.claude/lessons/feedback_falsifiable_hypothesis_before_structural_fix.md` (DQ #338 incident 2026-05-21: named `daemon's finalize step` as defect; daemon code had zero `reset --hard` calls; real vector was a lane-agent ssh-reset from the laptop; ~3 hours of option-a investigation displaced).
   **CR-finding variant:** the same falsifiable-hypothesis discipline applies to CodeRabbit and Copilot findings — trait/type/lifetime claims from automated reviewers are hypotheses, not contracts. Compile-check (`cargo check --workspace`) before triaging a CR finding as `bucket: fix-in-pr`. Per `feedback_verify_automated_reviewer_claims_against_compiler.md` (PR#132: `.get(0)`→`.first()` broke Diesel LimitDsl after CR recommendation).
2. Decide: **advisor-answer** (clear evidence + defensible answer; write `answer` and `answered_by: "advisor"`) / **catch-fire** (ADR violation, hard-refusal, process breach; stop loop, surface DQ id + cited rule) / **user-relay** (judgment-heavy: visible-to-others, ADR-affecting, scope change; surface one-screen summary; record user reply with `answered_by: "user"` + verbatim wording in `answer`).
3. Commit + push the answer. Subject MUST match `^(chore|docs)\((advisor|decision-queue)\)` per `.claude/rules/decision-queue.md` Attribution integrity §Detection. Narrow form: `chore(advisor|decision-queue): answer DQ #<id>`.

### 5.5 Retro-bypass observability

Per RLS-PMD review §4.7 + autonomy-readiness criterion 5.2 + `.claude/PRPs/plans/v1-rls-r1.plan.md` Task 7. The Stop hook `.claude/hooks/retro-check.sh` fail-open path (3-attempt cap, load-bearing for true loops) emits a JSONL `retro_bypass` record to `.claude/governance-log/retro-bypass.jsonl` on every fail-open. Fields per the kind registry at `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md`.

**Consumer:** the JSONL trail is consumed by future audit reads (weekly-review Step 2c is the retro-corpus sweep over `.claude/PRPs/reports/*.md`, not the JSONL; a dedicated JSONL-rate audit step would be added in a future sub-phase if a `retro_bypass` rate trend becomes load-bearing). **Autonomy signal:** the rate of `retro_bypass` entries per week should be monotonically decreasing. Rising rate → calibration-honesty regression; surface in the next phase retro.

**Advisor-side action:** none required at session-start (the trail is passive). At retro time, the rate trend is part of the four-role retro signals (Advisor role) per `feedback_four_role_retro_signals.md`. See `feedback_retro_bypass_governance_log.md`.

**Orchestration failure first-check (2026-05-31):** when an orchestration anomaly is observed (task reported done but expected artefact absent, retro not found, wrong branch state), check `.claude/governance-log/retro-bypass.jsonl` BEFORE diagnosing code or DQ causes — a hook fail-open is often the proximate cause, not a process error.

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
| Cancelling a Junior task whose worker log shows uncommitted code (i.e. worker wrote files but did not `git commit`) | `feedback_cohort_shared_git_index_contention.md` | **Before issuing `cancel_task`:** SSH to `/srv/brehon-fork/.junior/worktrees/job-<id>` and `tar czf /tmp/job-<id>-recovery-$(date +%s).tar.gz .` to preserve uncommitted code. Then cancel. The daemon's cancel handler reaps both the worktree FS state and `.git/worktrees/<name>/` admin metadata **immediately and completely** — the cancel is irreversible and forfeits all uncommitted code. The tar step is lossless and takes <30s per worker. SSH template: `ssh homeserver "tar czf /tmp/job-<id>-recovery-\$(date +%s).tar.gz -C /srv/brehon-fork/.junior/worktrees job-<id>" && echo preserved`. If the worktree path is absent (already reaped or task never started), skip silently — no harm. |

## 6. Subagent delegation (advisor-side `Agent` tool dispatch)

Distinct from the four Junior subagents (planning / impl-task / bm-task / ci-watcher) — this section covers the **laptop-side** `Agent` tool the advisor invokes for in-session research, file edits, audits, or any independent deliverable that doesn't need to run on the EliteDesk. The Junior subagents are queued via `mcp__junior-brehon__create_task` and run on the daemon; the `Agent` tool subagents run in the advisor session's harness and return inline.

### 6.1 Parallel dispatch + 6.2 Verify-after-subagent-completes

Both patterns are sub-promotion (single recurrence each at promotion time). Procedure: `.claude/refs/advisor-subagent-dispatch.md`. Read on demand when the corresponding dispatch shape fits the current task. When a 2nd recurrence lands for either, lift back from refs/ into this section.

### 6.3 Bounded sub-agent dispatch and report semantics

Independent of §6.1/§6.2, two invariants for all `Agent` tool dispatch:

- **Brief like a smart colleague who just walked into the room.** Sub-agents see no parent conversation. Include: what you're trying to accomplish, what you've ruled out, the surrounding context that lets the sub-agent make judgment calls. Terse command-style prompts produce shallow generic work.
- **Trust but verify.** Sub-agent reports describe what they *intended* to do, not necessarily what they did. For any edit to a tracked file, verify via §6.2 grep. For any code change, run the relevant validation (`cargo check`, the test, the lint) before assuming the change is sound.

## See also

- `.claude/agents/<name>.md` — Junior subagent contracts (planning, impl-task, bm-task, ci-watcher) — distinct from §6's advisor-side `Agent` tool subagents.
- `.claude/rules/branch-manager.md` + `.claude/commands/bm/<verb>.md` — BM mechanics, file-ownership, autonomy bounds.
- `.claude/rules/decision-queue.md` — DQ schema, attribution, per-kind routing.
- `.claude/refs/auto-phase.md` — `/auto-phase` state machine that compiles §3.1 + §4.1 + §5.3 (lazy-loaded by the skill body's Phase 0 Step 0).
- `.claude/refs/advisor-orchestrator-incidents.md` — §1 polling-loop incident narratives + hook FP/FN taxonomies (extracted 2026-05-29).
- `.claude/rules/pmd-search-strategy.md` — PMD search modes referenced by §2.3.
- `homeserver/.claude/advisor-context-phase-<N>.md` — phase-specific texture (session-start read).
- `.claude/lessons/feedback_cross_session_commit_attribution_collision.md` — Race-A + Race-B mitigations cited from §6.2.
- `.claude/PRPs/reports/session-retro-2026-05-22-parallel-subagent-dispatch.md` — wall-clock evidence + status rationale for §6.1.
