# JM-d planning brief

**Written**: 2026-04-26 by advisor session (laptop) for Junior dispatch on EliteDesk
**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`)
**Worktree**: Junior cuts `junior/jm-d-planning-1` from `governance-v0` per its concurrency-1 default; the planning subagent commits the plan file and pushes back to `governance-v0` at finalize.

## 1. Role + dispatch line

`[role:planning] v1-jury-mechanics-d plan — appeal-window expiry job + reporter-rights + auto-rejury`

This is the four-line Junior task description shape per `.claude/rules/advisor-orchestrator.md` "Junior task description template." The actual `mcp__junior-brehon__create_task` description is a single line: `[role:planning] v1-jury-mechanics-d plan — see .claude/PRPs/briefs/jm-d-planning-1.md`. Everything else lives in this brief.

## 2. Scope

Produce one plan file at `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` for sub-phase **v1-JM-d**. The plan covers the whole §17 row 4 deliverable: the `request_appeal` rewrite (bounded window + reporter-rights), the new `admin_trigger_appeal_rejury` handler, the appeal-window-expiry background job, and the e2e tests for the three new acceptance criteria.

**Source-of-truth references for scope:**

- `.claude/PRPs/prds/v1-jury-mechanics.prd.md` §17 row 4 (the row entry)
- `.claude/PRPs/prds/v1-jury-mechanics.prd.md` §6 (Appeals — first-class) — §6.1 panel sizing, §6.2 original-jurors-excluded hard rule, §6.3 threshold-tier bump, §6.4 reporter-eligibility, §6.5 bounded window, §6.6 auto re-jury, §6.7 status state machine
- `.claude/PRPs/prds/v1-jury-mechanics.prd.md` §9.3 (`request_appeal` rewrite), §9.4 (`admin_trigger_appeal_rejury`), §9.5 (background job)
- `.claude/PRPs/prds/v1-jury-mechanics.prd.md` §10 (default values matrix — appeal.* knobs already seeded by JM-a per §17 row 1)
- `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` §10.8 GOTCHA (concurrency-test pattern lessons that JM-c surfaced) and the §9 step-9 write of `appeal_window_expires_at` that JM-d depends on
- `.claude/decision-queue.json` resolved entry **#50** (deferred concurrency Test 6, FK-SHARE-then-FOR-UPDATE deadlock pattern — JM-d may want to revisit if the appeal-window job acquires FOR UPDATE)
- `.claude/PRPs/prds/v1-jury-mechanics.prd.md` §17.4 (preflight guardrails — DQs #42/43/44/46, all resolved at command-template level; cite for traceability, no inline compensation needed)
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` §17 (table contracts; verify `appeal_window_expires_at` column shape per JM-a migration `8.1`)

**Hard out-of-scope for JM-d:**

- `submit_jury_vote` step 7 sponsor-liability compute branch — owned by SL-d (per §17.1: JM-c stabilises step 7, SL-d grafts compute, JM-d/e parallelise on different files).
- `submit_jury_vote` step 7 vote-outcome / evidence-quality emitters — owned by rep-tuning-r3.
- v1-JM-e capstone integration test (`v0_case_completes_under_v0_rules_after_v1_config_flip` is JM-c's; the chained capstone is JM-e's).
- Cross-instance jury, composable constraints, `no_same_endorsement_chain` — v2-candidate/v1.5-candidate per PRD §2 OUT.

**Plan file deliverable:**

- One file: `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md`
- Follow the canonical template literally — it is at `.claude/commands/prp-core/prp-plan.md` (this is the file the `/prp-core:prp-plan` slash command reads; the top-level `.claude/commands/prp-plan.md` is a duplicate)
- Plan style targets ~6-7 tasks per PRD §17 row 4 estimate, each task one commit, MED risk
- §15 DoD validation commands MUST be executable as written against current `governance-v0` HEAD — dry-run each one before committing the plan
- §4 watchpoints MUST cite specific files / tables / `schema.rs` lines per `.claude/lessons/feedback_advisor_watchpoint_specificity.md` — never abstract concepts
- §17.4-cited DQ ids only (don't re-codify the fix; the command template already carries them)

**Commit only the plan file.** Don't author Rust code, don't open PRs, don't touch any file under `crates/`, `migrations/`, or `tests/`. The Junior finalize step pushes the plan-file commit to `governance-v0`.

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract; defines model, tools, "Before you start (always)" sequence, plan-content discipline cross-references
2. `.claude/commands/prp-core/prp-plan.md` — canonical Brehon plan template (the `/prp-core:prp-plan` command); structure is load-bearing
3. `.claude/PRPs/prds/v1-jury-mechanics.prd.md` — the parent PRD; read §6 + §9.3-9.5 + §10 (knobs row 25-30) + §17 row 4 + §17.1 (cross-PRD sequencing) + §17.3 (worktree naming) + §17.4 (preflight)
4. `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` — the predecessor plan; read §10.8 GOTCHA (concurrency lessons), §9 (the step-1-through-9 handler shape JM-d builds on)
5. `.claude/PRPs/reports/phase-v1-JM-a-retro.md` — JM-a retro; pattern lessons that propagated into JM-b/c
6. `.claude/PRPs/reports/v1-JM-b-retro-events.md` — JM-b retro events; the 7 amendments JM-c applied (R1-R7)
7. `.claude/decision-queue.json` resolved entries — #45 (test-substitution policy), #48 (InsertForm drift pattern), #50 (FK-SHARE-then-FOR-UPDATE deadlock; JM-d's background job and `request_appeal` SELECT ... FOR UPDATE may hit similar territory)
8. **Glob `.claude/lessons/` and Read every file whose name keyword matches "appeal", "background-job", "scheduled", "ConstraintRecord", "InsertForm", "concurrency", "lock", "deadlock", "DoD", "watchpoint", "features-full", "wrapper-script", "test-style", "clippy", "migration".** That's the lessons corpus discipline per planning.md step 3.
9. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — ADR-010 (no retroactive invalidation; relevant for §6.5 window column semantics) and any OQ-V1-JM-* with `JM-d` in the resolution
10. `crates/api/api_crud/src/governance/request_appeal.rs` — the file you're rewriting; current shape per §9.3 line refs (line 105 closed_at check, line 110 caller eligibility)
11. `crates/server/src/governance.rs` (or sibling) — the existing `expired sanction cleanup` pattern §9.5 names as the mirror for the new appeal-window-expiry job
12. `crates/db_schema/src/source/governance/moderation_case.rs` — verify `appeal_window_expires_at TIMESTAMPTZ` column is in place (JM-a §8.1) and the InsertForm + read-model carry it (per JM-b DQ #48 InsertForm drift lesson)

## 4. Constraints (hard rules — violating any of these is a process breach)

**Plan-content discipline (per `.claude/lessons/`):**

- Every §15 DoD validation command must be executable against current HEAD. Dry-run each before commit. Per `feedback_pre_phase_dod_smoke_test.md` and `feedback_plan_dod_dry_run_at_write.md`.
- Never combine `-p <crate>` with `--features full` in any DoD command — only `--workspace --features full` works. Per `feedback_features_full_p_crate_incompatible.md`.
- Add `--no-deps` to any `clippy ... -- -D warnings` to avoid inheriting upstream lint debt. Per JM-a/JM-b retro R6.
- Add `--features full` to clippy on governance crates so the gated code compiles. Per `pre-phase-harness-audit.md` §2.
- Use `-p lemmy_server` on `cargo test --test e2e ...`. Per JM-b retro R7.
- Cast `i32 → i64` via `i64::from(...)`, never `as` cast. Per JM-b retro R1.
- Mandatory `seed_jury_eligible_snapshots` before every `admin_assign_jury` in tests. Per JM-b retro R2.
- Watchpoints in §4 cite specific files / tables / `schema.rs` lines — never abstract concepts. Per `feedback_advisor_watchpoint_specificity.md`.
- Wrapper scripts (`scripts/brehon/cargo-*.bat` on Windows, `cargo-*.sh` on Linux) must accept the flags you depend on. Verify before naming any wrapper. Per `feedback_wrapper_script_flag_silence.md`. The Linux wrappers were fixed 2026-04-26 in commit `4571c53` to accept `$@` passthrough; the .bat siblings were already correct.

**Decision-queue discipline (per `.claude/rules/decision-queue.md`):**

- If you (planning subagent) seed any DQ entry in the plan, label it `answered_by: "planner"` — never `"advisor"`. Per the attribution rule §"Subagents and attribution."
- If you discover a question that needs the persistent advisor session to answer (ADR-affecting, scope-changing, judgment-heavy): leave the entry as `pending` with `answered_by: null` — do NOT pre-answer with advisor wording.
- Mid-task DQ writes from this Junior worktree must commit + push immediately, not wait for finalize. Per `decision-queue.md` §"Mid-task visibility (Junior worktrees)." Subject pattern: `chore(decision-queue): planner raised DQ #<id> — <slug>`. Push to `junior/jm-d-planning-1` (this worktree's branch), not to `governance-v0` directly — the advisor's polling loop fetches all branches.

**File ownership:**

- Touch only `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` (CREATE) and (optionally) `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries).
- Do NOT edit anything under `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, or any other plan file in `.claude/PRPs/plans/`.
- One commit at finalize: `feat(plan): v1-JM-d sub-phase plan` (or `docs(plan):` if your project convention prefers — match the existing JM-c plan's commit subject).

**Boundary-of-judgment (when to STOP and queue rather than guess):**

- If PRD §6 / §9 / §10 contradicts itself or contradicts JM-c's §9 step-9 implementation → queue DQ pending, advisor decides which to honour.
- If the existing `expired sanction cleanup` pattern §9.5 names doesn't actually exist in the codebase → queue DQ pending with the location you searched.
- If the appeal panel sizing math (§6.1: 1.5× original rounded up, min original+2) produces ambiguous values for any panel size in §10 (5, 7, 9, 11) → queue DQ for the rounding rule.
- If `select_appeal_panel` (§9.3) is supposed to be a new helper but JM-b/JM-c already shipped a similar helper under a different name → queue DQ for the naming.

**Attribution integrity reminder:** the only valid `answered_by` labels for a Junior planning subagent are `"planner"` (forward-looking pre-resolved entries you have a recommendation on) or `null` (genuinely needs advisor input). Never `"advisor"`, never `"user"`, never `"impl-self-resolved"` (that's an impl-task label).

---

**Lean / advisor-side tip (not a constraint):** JM-d's biggest plan-shape risk is the background job in `crates/server/src/governance.rs` — it's a new pattern relative to JM-a/b/c (all of which were handler-rewrites, not periodic-job authoring). JM-c's §10.8 deadlock GOTCHA (DQ #50) is one signal that lock semantics under concurrency are tricky in this handler family; the appeal-window-expiry job's `UPDATE moderation_case SET status = Closed WHERE appeal_window_expires_at < now()` will need careful attention to whether it uses FOR UPDATE, NOWAIT, or a SKIP LOCKED — the existing sanction-cleanup job is the right MIRROR ref for the pattern. If MIRROR examination shows the existing job uses raw SQL or an unusual locking shape, surface that in §4 watchpoints rather than copying blindly.
