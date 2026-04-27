# v1-validate-agent planning brief (Shape G — GitHub Actions)

**Written**: 2026-04-27 by advisor session (laptop) for Junior dispatch on EliteDesk
**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`)
**Worktree**: Junior cuts `junior/v1-validate-agent-planning-1` from `governance-v0` per its concurrency-1 default; the planning subagent commits the plan file and pushes back to `governance-v0` at finalize.

## 1. Role + dispatch line

`[role:planning] v1-validate-agent plan — out-of-Junior cargo validation via GitHub Actions + ci-watcher (Shape G)`

This is the four-line Junior task description shape per `.claude/rules/advisor-orchestrator.md` "Junior task description template." The actual `mcp__junior-brehon__create_task` description is a single line: `[role:planning] v1-validate-agent plan — see .claude/PRPs/briefs/v1-validate-agent-planning-1.md`. Everything else lives in this brief.

## 2. Scope

Produce one plan file at `.claude/PRPs/plans/v1-validate-agent.plan.md` for sub-phase **v1-validate-agent**. The plan covers the architectural change described in the originating design brief (`ci-validation-design.md`, Shape G): add two GitHub Actions workflows for cargo validation, introduce a Haiku-4.5 `[role:ci-watcher]` Junior subagent that polls workflow runs, change `impl-task.md` to push-and-exit (out-of-band validation), and extend the decision-queue schema with three new validate kinds.

**Source-of-truth references for scope:**

- **`.claude/PRPs/briefs/ci-validation-design.md`** — the originating design brief (load-bearing). Shape G. The plan is the canonical translation of its §3 Solution shape, §4 Files, §5 DQ schema, §6 comparison, §8 failure modes into the canonical 20-section plan.
- `.claude/PRPs/briefs/validate-agent-design.md` — the sibling Shape C design (PARKED, NOT KILLED). Cite for §G4 escalation contract + §G5/§3.5 planning-side implications, which are shared between Shape C and Shape G. The plan implements Shape G's path, not Shape C's.
- `.claude/decision-queue.json` resolved entry **#61** (advisor 2026-04-27) — the plan-mode session decision: Shape G chosen, JM-d Task 2 parked, single-brief retrofit of `jm-d-impl-2.md` §5 once Shape G ships. Cite by id in §2 Source.
- `.claude/decision-queue.json` resolved entries **#62–#67** (advisor + user 2026-04-27 clarify pass on this brief) — the gating clarify-DQs. The plan must honour each:
  - **DQ #62** (advisor) — ci-watcher uses `gh run watch <id> --exit-status` (single long-poll, 1 API call per task), NOT `gh run view` polling. Plan §13 ci-watcher-authoring task wires `gh run watch --exit-status`. Plan §4 includes a watchpoint requiring empirical validation of `gh run watch` exit semantics before authoring ci-watcher.md (no codebase precedent).
  - **DQ #63** (advisor) — schema additive: add `from: "ci-watcher"` only. impl-task continues writing as `from: "impl"` even when the entry carries `kind: "validate-pending"`. Plan §13 schema-additive task adds `ci-watcher` to decision-queue.md's "Subagents and attribution" enumeration; explicitly documents impl-task's continued `from: "impl"` use. NEVER rewrite historical entries.
  - **DQ #64** (advisor) — impl-task.md sections are NAMED, not numbered. Plan §13 task that modifies impl-task.md (a) renames "Per-task validation gate" → "Per-task validation gate (out-of-band on GH Actions)" and rewrites its body; (b) adds one line to "Hard refusals": "Never invoke cargo for build/lint/test — validation runs out-of-band on GH Actions per the validation gate above". Reference impl-task.md sections by H2 NAME, never by §-number — there is no §-number in that file.
  - **DQ #65** (advisor) — `.claude/PRPs/templates/plan.template.md` is the canonical structural template (post-spec-kit). `.claude/commands/prp-core/prp-plan.md` is reference-only for the slash command. Edit Required-reading order: plan.template.md as canonical; prp-plan.md as command-implementation reference.
  - **DQ #66** (user) — workflow trigger pattern is `on: { push: { branches: ['phase-v1-*', 'junior/*'] } }`. NO `pull_request` trigger. Defence-in-depth via branch-protection rules + advisor-orchestrator merge gate (validate-result must exist on the latest phase-branch SHA before bm-merge).
  - **DQ #67** (user) — workflow YAML §15 dry-run discipline is YAML parse only via `yamllint .github/workflows/cargo-validate-workspace.yml` exit 0 OR `gh workflow view --repo barrie-cork/lemmy --ref <phase-branch> cargo-validate-workspace.yml`. The plan must include a "first test push" §13 task DoD step that validates trigger fires + runs green ON the phase branch (separate from the workflow-authoring task). Do NOT prescribe `act` invocations.
- `.claude/decision-queue.json` resolved entries **#55, #57, #59** — Shape-agnostic clarify answers that still apply under Shape G:
  - **DQ #55** (advisor) — Layer A subagent (now `[role:ci-watcher]`) runs under existing `junior@brehon-fork` cgroup (10G/8G); no new systemd unit. **Reinterpret**: ci-watcher is mechanical Haiku-4.5 polling, no cargo, so the cgroup cap is non-binding for it.
  - **DQ #57** (user) — fix-vs-escalate threshold for the failure classifier: ≤3 file edits = auto-fix-impl-task brief; >3 = surface to user. **Reinterpret**: under Shape G the classifier is the advisor's own §G4 logic operating on `validate-failed` DQ entries (not a subagent's own decision). The threshold still applies to which clippy lints / missing imports qualify for an auto-queued fix-impl-task.
  - **DQ #59** (advisor 2026-04-27 plan-mode) — branch model = phase-branch + PR. Cut `phase-v1-validate-agent` off `governance-v0` via `bm-cut`; final PR back with CodeRabbit review per `feedback_pr_per_phase.md`.
- `.claude/decision-queue.json` resolved entries **#56, #58, #60** — Shape-C-specific clarify answers that **DO NOT** apply under Shape G:
  - **DQ #56** (validate-runner.sh + systemd timer at homeserver-level) — Shape G has no validate-runner.sh; cargo runs on GH Actions. Cite as historical context only; do NOT include any homeserver/scripts/brehon/ or homeserver/systemd/ artefact in the plan.
  - **DQ #58** (tmp-dir clone for cargo isolation) — Shape G runs cargo on ephemeral GH runners; no isolation needed. Cite as historical context only.
  - **DQ #60** (impl-task.md change after JM-d ships, before JM-e starts) — **PARTIALLY OVERRIDDEN by DQ #61**. JM-d is no longer ship-then-merge before validate-agent — instead, JM-d Task 2 is **parked** until validate-agent ships. Other JM-d tasks already shipped. The new ordering: validate-agent ships → JM-d Task 2 retrofit + queue → JM-d remaining tasks → JM-d retro → JM-e. The plan §17 Completion checklist must enforce: validate-agent's `bm-merge` happens BEFORE JM-d Task 2 is queued.
- `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` — most recent shipped plan; mirror §1..§14 + §16..§20 shape exactly. JM-d uses the **legacy §15 prose DoD shape**. The validate-agent plan introduces the new "DoD per workflow" §15 shape (per Shape G design §G5 + ci-validation-design.md §4.2 plan.template.md change).
- `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` — second-most-recent plan; mirror its `[P]` cohort marker placement and §16a Stories block (the canonical post-spec-kit pattern).
- `.claude/agents/{planning,impl-task,bm-task}.md` — canonical four-role contract; the new `ci-watcher.md` mirrors a mechanical-poller shape (Haiku-4.5, narrow tool list: Bash + Read + Edit + Write only — no Agent, no LSP, no Glob/Grep needed for polling).
- `.claude/rules/decision-queue.md` — current `kind: "blocker"|"log"|"clarify"` schema; the plan extends it with three new kinds: `"validate-pending"`, `"validate-result"`, `"validate-failed"` per Shape G §5. The schema additive must update the rule doc and the consumers (impl-task.md writer, ci-watcher.md writer, advisor-orchestrator.md reader) in the same commit per `feedback_update_rule_doc_with_schema_additive.md`.
- `.claude/rules/advisor-orchestrator.md` — current "Stage-shape orchestration" + "Mandatory user gates" + "Forbidden execution windows" sub-sections. The plan adds a "validate stage" between impl-task complete and bm-cut/bm-pr (transitions: impl-commit → ci-watcher dispatched → validate-result|validate-failed → §G4 classifier → advance). The "Forbidden execution windows" section gains a note that cargo no longer runs locally for impl-task throughput; ad-hoc local validation by advisor pre-plan-approval still respects the windows.
- `.github/workflows/cargo-test-e2e.yml` — pattern source for new workflow YAMLs. Mirror its cache strategy (`actions/cache@v4`, paths `~/.cargo/registry ~/.cargo/git target/`, key on `Cargo.lock` hash).
- `homeserver/CLAUDE.md` "Junior" sub-section — add a note that ci-watcher is a Junior subagent (not a daemon) and that workflow runs are GH-Actions-side (not a local validate-runner).
- `homeserver/library.yaml` — registration shape. Every new workflow YAML, agent (`ci-watcher.md`), brief template (`ci-watcher-brief.template.md`) registers in the same commit that ships it per `feedback_library_add_after_shipping.md`.
- `.claude/PRPs/templates/plan.template.md` §13 + §15 + §16a — current template shape; the plan proposes the §13 `validate_path: out-of-band|inline` field addition + §15 "DoD per workflow" shape per design brief §4.2.

**Hard out-of-scope for v1-validate-agent:**

- **Retrofitting JM-d briefs OTHER THAN jm-d-impl-2.md.** Per DQ #61: the bounded retrofit is one brief — `jm-d-impl-2.md` §5 — to switch from inline cargo to "push and exit; ci-watcher polls" (Layer G2). All other JM-d briefs (jm-d-impl-1, 3, 4, 5, 6, 7) stay forward-only-immune. The plan must include this single-brief retrofit as one §13 task, AND must NOT propose any retrofit of earlier JM-d briefs or of plans for v1-AD-*, v1-JM-a/b/c.
- **JM-d Task 2 itself.** Task 2 is parked per DQ #61. The validate-agent plan does NOT queue Task 2; it only sets up the conditions (Shape G shipped) under which Task 2 can be queued by the advisor's polling loop AFTER validate-agent's `bm-merge`. The plan §17 Completion checklist must contain a line: "validate-agent bm-merge unblocks JM-d Task 2 queueing — flag for advisor."
- **Cross-provider model trials** (MiniMax, Gemini, etc.) — Brehon stays Anthropic-only per `feedback_brehon_anthropic_only.md`. ci-watcher is Haiku 4.5.
- **BM verb changes** (`bm-cut`/`bm-pr`/`bm-merge`/`bm-poll-cr`/`bm-triage`/`bm-merge` semantics) — validate is a sibling stage in the orchestrator, not a BM verb.
- **Replacing the §15 DoD definition itself** — we change WHERE DoD runs (out-of-band on GH Actions vs in-band on Junior worker) and HOW it's expressed (workflow-referenced vs inline cargo commands), not WHAT it tests. Semantic content of DoD stays.
- **Self-hosted GitHub runners on the EliteDesk.** Tempting for minutes-budget mitigation, but re-introduces resource contention this design solves. Out of scope unless private-repo minute limits become binding for >2 consecutive months. Per ci-validation-design.md §11.
- **Replacing `cargo-test-e2e.yml`.** It works; the new workflows supplement, don't replace. Per Shape G design §3 + §4.3.
- **Auto-retry of failed validation** — `max_retries: 0` discipline in junior config.yaml stays. validate-failed surfaces; advisor decides; no auto-retry. Workflow re-run via `gh run rerun <id>` is an advisor-side recipe in the §G4 classifier, not an auto-action.
- **Investigation of PMD #109 (impl-task on Opus despite Sonnet pin), PMD #110 (telemetry blind-spot), PMD #111 (Server Boss alert AND-gate)** — separate diagnostics. Independent from this design. The Shape G impl-task contract changes (push-and-exit) don't depend on resolving these.

**Plan file deliverable:**

- One file: `.claude/PRPs/plans/v1-validate-agent.plan.md`.
- Follow `.claude/PRPs/templates/plan.template.md` literally — the 20-section structure is load-bearing (per DQ #65).
- Plan style targets ~5–7 tasks per design brief §13 estimate, each task one commit, MED risk.
- §15 DoD validation commands MUST be executable as written against current `governance-v0` HEAD — dry-run each one before committing the plan. The new "DoD per workflow" shape (per design brief §4.2) does NOT exempt §15 from dry-run discipline; it just changes the format. For workflow-referenced DoD entries, dry-run is **YAML parse only via `yamllint .github/workflows/cargo-validate-workspace.yml` exit 0** OR `gh workflow view --repo barrie-cork/lemmy --ref <phase-branch> cargo-validate-workspace.yml` returning a parsed view (per DQ #67). End-to-end validation happens at first push to phase branch — the plan must include a "first test push" §13 task DoD step that confirms trigger fires + workflow runs green ON the phase branch (separate from the workflow-authoring task). Do NOT prescribe `act` invocations.
- §4 watchpoints MUST cite specific files / tables / `schema.rs` lines per `.claude/lessons/feedback_advisor_watchpoint_specificity.md` — never abstract concepts. The ci-watcher's `gh run watch` exit-code semantics, the workflow trigger pattern (`phase-v1-*` and `junior/*` only), and the `actions/cache@v4` key construction are likely watchpoint targets.
- §13 must include `[P]` cohort markers per `.claude/PRPs/templates/plan.template.md` §13 + `feedback_parallel_cohort_dispatch.md`. Most validate-agent tasks touch disjoint files (workflow YAMLs vs ci-watcher.md vs rule doc updates vs jm-d-impl-2.md retrofit) and are likely cohort-compatible — but the planner judges file-set overlap mechanically, not heuristically.
- §16a Stories block must enumerate end-to-end testable behaviours. Likely stories: "impl-task pushes to a phase-v1-* branch and `cargo-validate-workspace.yml` triggers automatically", "ci-watcher reads a `validate-pending` DQ entry and polls a workflow run via `gh run watch` to completion", "ci-watcher classifies a `success` workflow as `validate-result: pass` and writes the DQ entry", "ci-watcher classifies a `failure` workflow as `validate-failed` with the failing log slice attached", "advisor §G4 classifier auto-queues a fix-impl-task for a known clippy lint", "validate-agent's bm-merge unblocks JM-d Task 2 (single-brief retrofit + queue)".

**Commit only the plan file.** Don't author Rust code, don't open PRs, don't touch any file under `crates/`, `migrations/`, `tests/`, or `.github/workflows/` (that's the implementing session's job, not the planner's). The Junior finalize step pushes the plan-file commit to `governance-v0`.

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract; defines model, tools, "Before you start (always)" sequence, plan-content discipline cross-references.
2. **`.claude/PRPs/templates/plan.template.md`** — **canonical** structural template (post-spec-kit). 20-section schema is load-bearing. §13 (with `[P]` markers) + §15 + §16a Stories block are load-bearing additions. Per DQ #65.
3. `.claude/commands/prp-core/prp-plan.md` — reference-only for the `/prp-core:prp-plan` slash-command implementation. NOT canonical for plan structure (DQ #65 supersedes).
4. **`.claude/PRPs/briefs/ci-validation-design.md`** — the originating design brief (load-bearing). Read entire file. The plan is the canonical translation of its §3–§13 into a sub-phase plan. NOTE: §G3 vs §8.3 contradiction on polling mechanism is resolved by DQ #62 (use `gh run watch --exit-status`).
5. `.claude/PRPs/briefs/validate-agent-design.md` — sibling parked Shape C design. Read §3.5, §4.2, §4.3 specifically (planning-side implications shared with Shape G). The rest is Shape-C-specific and informs comparison only.
6. `.claude/decision-queue.json` resolved entry **#61** — the plan-mode decision (Shape G chosen, JM-d Task 2 parked, single-brief bounded retrofit). Read in full; this entry's `answer` field is the most recent advisor-attributed scope statement for the sub-phase.
7. `.claude/decision-queue.json` resolved entries **#62–#67** — clarify pass on this brief (advisor + user 2026-04-27). Each gates a planning decision; honour all six per §2 source-of-truth notes above.
8. `.claude/decision-queue.json` resolved entries **#55, #57, #59** — Shape-agnostic clarify results that apply under Shape G (read §2's "Reinterpret" notes).
9. `.claude/decision-queue.json` resolved entries **#56, #58, #60** — Shape-C-specific clarify results that DO NOT apply (read §2's notes; do not honour their answers literally).
10. `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` — predecessor plan; mirror §1..§14 + §16..§20 shape exactly. Treat its §15 prose shape as the legacy reference; the new plan introduces "DoD per workflow" §15.
11. `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` — second-most-recent plan; mirror its `[P]` cohort marker placement and §16a Stories block.
12. `.claude/agents/impl-task.md` — current contract that the validate-agent plan §13 will modify. **Sections are NAMED, not numbered** (per DQ #64): the change rewrites the H2 "Per-task validation gate" body and renames it to "Per-task validation gate (out-of-band on GH Actions)"; adds one line to "Hard refusals". Reference impl-task.md sections by H2 NAME, never by §-number.
13. `.claude/agents/bm-task.md` — canonical Sonnet-4.6 + narrow-tools shape. `ci-watcher.md` mirrors its frontmatter STRUCTURE (model + effort + tools list shape) but pins to Haiku 4.5 instead of Sonnet 4.6, with `effort: low` (per ci-validation-design.md §G3).
14. `.claude/rules/decision-queue.md` — current `kind: "blocker"|"log"|"clarify"` schema; the plan extends it with three new kinds (`validate-pending`, `validate-result`, `validate-failed`) per Shape G §5. Schema-additive `from` value: add `from: "ci-watcher"` only; impl-task continues as `from: "impl"` (per DQ #63).
15. `.claude/rules/advisor-orchestrator.md` — current "Stage-shape orchestration" sub-section. The plan adds a "validate stage" between impl-task commit and next-cohort. Cite specific line numbers in §4 watchpoints.
16. **`.github/workflows/cargo-test-e2e.yml`** — the canonical existing workflow. Read in full. **Trigger pattern is `pull_request:` to `governance-v0` — but Shape G's new workflows use `push:` to `phase-v1-*` and `junior/*` (per DQ #66). Mirror `actions/cache@v4` strategy (lines ~58-67), `libpq` install (lines ~53-56), and Rust toolchain setup (lines ~47-51); do NOT mirror the trigger pattern.**
17. **Glob `.claude/lessons/` and Read every file whose name keyword matches `validate`, `dod`, `watchpoint`, `cargo`, `wrapper-script`, `features-full`, `clarify`, `parallel-cohort`, `read-canonical`, `dogfood-slash`, `update-rule-doc`, `schema-changing-spec`, `pr-per-phase`, `library-add`, `commit-aggressively`, `pipes-mask-exit-codes`, `clippy-test-style`.** Lessons corpus discipline per planning.md step 3.
18. `homeserver/CLAUDE.md` — orchestrator/infra repo contract. Read "What This Repo Is", "Junior", "Project Memory (Operational)", "Agentics Library" sub-sections.
19. `homeserver/library.yaml` — current registration shape.

## 4. Constraints (hard rules — violating any of these is a process breach)

**Plan-content discipline (per `.claude/lessons/`):**

- Every §15 DoD command must be executable against current HEAD. Dry-run each before commit. For workflow-referenced DoD entries, "executable" means the workflow YAML parses cleanly and triggers correctly on a sample push to a `phase-v1-*` branch. Per `feedback_pre_phase_dod_smoke_test.md` and `feedback_plan_dod_dry_run_at_write.md`.
- The new "DoD per workflow" §15 shape references workflow files by path (`.github/workflows/cargo-validate-workspace.yml`) plus the expected `conclusion` field (`success`). The exact format is a planner judgment call — surface as a planner-attributed DQ if the choice between "workflow path + conclusion" and "structured YAML block in §15" affects ci-watcher's parsing logic.
- Watchpoints in §4 cite specific files / tables / lines — never abstract concepts. Per `feedback_advisor_watchpoint_specificity.md`. For this plan: name the specific lines in `.claude/agents/impl-task.md` that change (the §15 self-run loop), the specific lines in `.claude/rules/advisor-orchestrator.md` "Stage-shape orchestration" that gain the validate stage, the specific lines in `.github/workflows/cargo-test-e2e.yml` that the new workflows mirror.

**Canonical-first discipline (per `feedback_read_canonical_before_writing_spec.md`):**

- Before authoring §13 task entries that prescribe new agent shape (`ci-watcher.md`), Glob + Read the canonical examples (`impl-task.md`, `bm-task.md`, `planning.md`) and cite them in §10 Patterns to mirror with file paths + line ranges.
- Before authoring §13 task entries that prescribe new workflow YAMLs, Read `.github/workflows/cargo-test-e2e.yml` in full and cite its cache strategy + libpq install + Rust toolchain steps in §10 Patterns to mirror.
- Before authoring §15 in the workflow-referenced shape, walk through how an existing JM-d §15.1/§15.2/§15.3 prose entry would translate. If translation reveals semantic loss, surface as a DQ pending entry — the schema is under-specified.

**Dogfood discipline (per `feedback_dogfood_slash_command_specs.md`):**

- The plan does NOT introduce new slash commands. ci-watcher does not have a slash command (it's an agent role, dispatched by `[role:ci-watcher]` in a Junior task description). The dogfood discipline therefore does not gate new commands here, but the planner should still mentally walk the §G4 classifier through one real `validate-failed` shape (workflow run with a known clippy lint, workflow run with an unknown error, workflow run that timed out) before committing the plan. Surface as §4 watchpoints.

**Schema discipline (per `feedback_update_rule_doc_with_schema_additive.md`):**

- The three new `kind` values (`"validate-pending"`, `"validate-result"`, `"validate-failed"`) and the `decision-queue.md` rule update happen in the same commit. The plan §13 task that adds these kinds to the rule doc is the same task that adds them to the impl-task.md writer + ci-watcher.md writer + advisor-orchestrator.md reader.

**Library registration discipline (per `feedback_library_add_after_shipping.md`):**

- Every new workflow YAML (`cargo-validate-workspace.yml`, `cargo-validate-migration.yml`), agent (`ci-watcher.md`), brief template (`ci-watcher-brief.template.md`), lesson (any new `feedback_*.md` the implementing session adds) must be registered in `homeserver/library.yaml` in the same commit that ships it. The 3rd-recurrence pattern (per `homeserver/docs/memory/PATTERNS.md` "library.yaml registration lag") must not happen on this sub-phase. Dedicated §17 Completion checklist line.

**Commit-discipline (per `feedback_commit_aggressively_in_shared_repos.md`):**

- Each `.claude/` edit commits per-edit, not per-session. The validate-agent sub-phase touches `.claude/` files in brehon-fork (agents/, rules/, PRPs/templates/, PRPs/briefs/jm-d-impl-2.md retrofit) plus homeserver (CLAUDE.md, library.yaml, .claude/rules/advisor-orchestrator.md). Commits stay tight per task.

**Forward-only retrofit discipline (per `feedback_schema_changing_spec_retrofit_question.md` + DQ #61):**

- Plan must include an explicit "Retrofit scope" section. The single bounded retrofit is `jm-d-impl-2.md` §5 (per DQ #61). All other JM-d briefs (`jm-d-impl-1`, `jm-d-impl-3`, ..., `jm-d-impl-7`) and all earlier-phase briefs/plans (v1-AD-*, v1-JM-a/b/c) stay forward-only-immune.
- The new §15 "DoD per workflow" shape applies to v1-JM-e and later. Pre-existing JM-d, JM-c, JM-b, JM-a plans stay under prose §15.

**Branch + commit discipline (per DQ #59 + #61):**

- Phase branch: `phase-v1-validate-agent`, cut off `governance-v0` via `bm-cut`. **Cut as soon as governance-v0 has integrated DQ #61's commit (already pushed: `419bbd545`).** No coupling to JM-d phase branch — JM-d's `phase-v1-JM-d` may still exist with parked Task 2 work; the branches are file-disjoint (validate-agent touches workflows + ci-watcher.md + impl-task.md + advisor-orchestrator.md; JM-d's parked Task 2 will eventually touch crates/db_schema models). Per `feedback_parallel_agents_one_worktree_per_agent.md`.
- One commit per task. Final PR back to `governance-v0` with CodeRabbit review per `feedback_pr_per_phase.md`.

**Decision-queue discipline (per `.claude/rules/decision-queue.md`):**

- If you (planning subagent) seed any DQ entry in the plan, label it `answered_by: "planner"` — never `"advisor"`. Per the attribution rule §"Subagents and attribution."
- If you discover a question that needs the persistent advisor session to answer (ADR-affecting, scope-changing, judgment-heavy that wasn't covered by the existing #55–#61 clarify pass): leave the entry as `pending` with `answered_by: null` — do NOT pre-answer with advisor wording.
- Mid-task DQ writes from this Junior worktree must commit + push immediately, not wait for finalize. Per `decision-queue.md` §"Mid-task visibility (Junior worktrees)." Subject pattern: `chore(decision-queue): planner raised DQ #<id> — <slug>`. Push to `junior/v1-validate-agent-planning-1` (this worktree's branch), not to `governance-v0` directly — the advisor's polling loop fetches all branches.

**File ownership:**

- Touch only `.claude/PRPs/plans/v1-validate-agent.plan.md` (CREATE) and (optionally) `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries).
- Do NOT edit anything under `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, `.github/workflows/`, or any other plan file in `.claude/PRPs/plans/`.
- One commit at finalize: `feat(plan): v1-validate-agent sub-phase plan` (or `docs(plan):` if your project convention prefers — match the existing JM-d plan's commit subject).

**Boundary-of-judgment (when to STOP and queue rather than guess):**

- If design brief §3 Layer G1/G2/G3/G4 boundaries contradict §6 comparison table → DQ pending.
- If the proposed `ci-watcher.md` Haiku-4.5 frontmatter conflicts with `bm-task.md` or `impl-task.md`'s established shape (model, effort, tools list) in a way the design brief §G3 doesn't resolve → DQ pending.
- If the proposed `kind: "validate-pending"|"validate-result"|"validate-failed"` schema clashes with existing `kind: "blocker"|"log"|"clarify"` consumers (any rule doc, any subagent contract, any script that reads decision-queue.json) → DQ pending.
- If the proposed plan-template §15 workflow-referenced shape can't represent any existing JM-d §15 entry (§15.1 static analysis, §15.2 lint, §15.3 test target compile, §15.4 e2e execution, §15.5 migration round-trip, §15.6 cross-cutting verification, §15.7 manual SQL) → DQ pending. The schema is under-specified.
- If `.github/workflows/cargo-test-e2e.yml`'s cache strategy reveals a quirk that doesn't translate to workspace-level cargo check (e.g. e2e uses testcontainer-based workflow that requires Docker, workspace check doesn't) → DQ pending.
- If `gh run watch` exit-code semantics on success vs failure aren't documented (they aren't — there's no prior precedent in this codebase) → surface as §4 watchpoint, propose the implementing session validates `gh run watch` behaviour empirically before authoring ci-watcher.md.

**Attribution integrity reminder:** the only valid `answered_by` labels for a Junior planning subagent are `"planner"` (forward-looking pre-resolved entries you have a recommendation on) or `null` (genuinely needs advisor input). Never `"advisor"`, never `"user"`, never `"impl-self-resolved"` (that's an impl-task label).

---

**Lean / advisor-side tip (not a constraint):** the validate-agent sub-phase's biggest plan-shape risk is the **temporal coupling between three async events**: (1) impl-task pushes to a `phase-v1-*` branch, (2) GitHub Actions queues + starts the workflow run, (3) ci-watcher polls until completion via `gh run watch --exit-status` (per DQ #62). Failure modes: (a) impl-task pushes but writes the `validate-pending` DQ entry BEFORE the workflow has been queued (workflow_run_id not yet retrievable via `gh run list`); (b) ci-watcher dispatched before the push has propagated to GitHub; (c) network flake during `gh run watch` mid-poll. Mitigations: (a) impl-task captures workflow_run_id via `gh run list --branch <my-branch> --limit 1 --json databaseId --jq '.[0].databaseId'` AFTER the push completes (with a small retry loop with exponential backoff if the run hasn't appeared yet — 1-2 min push-to-trigger lag is normal per Shape G §8.4); (b) advisor's polling loop only dispatches ci-watcher when a `validate-pending` DQ entry appears, which requires impl-task to have completed step (a); (c) `gh run watch` long-polls + auto-resumes on transient errors. The implementing session must validate `gh run watch --exit-status` exit-code semantics empirically before authoring ci-watcher.md (no codebase precedent — surface as plan §4 watchpoint per DQ #62). A secondary risk: GitHub Actions free-tier minutes (2000/mo private repo). Trigger pattern is `on: { push: { branches: ['phase-v1-*', 'junior/*'] } }` only (per DQ #66) — NO `pull_request` trigger — to keep consumption bounded. Per Shape G §6 cost note: ~$1.40/sub-phase even paid; 80–200 validations before free-tier limit — comfortable for ~5 sub-phases/month.
