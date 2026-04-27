# validate-agent planning brief

**Written**: 2026-04-27 by advisor session (laptop) for Junior dispatch on EliteDesk
**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`)
**Worktree**: Junior cuts `junior/validate-agent-planning-1` from `governance-v0` per its concurrency-1 default; the planning subagent commits the plan file and pushes back to `governance-v0` at finalize.

## 1. Role + dispatch line

`[role:planning] v1-validate-agent plan — out-of-Junior cargo validation (Shape C + A escalation)`

This is the four-line Junior task description shape per `.claude/rules/advisor-orchestrator.md` "Junior task description template." The actual `mcp__junior-brehon__create_task` description is a single line: `[role:planning] v1-validate-agent plan — see .claude/PRPs/briefs/validate-agent-planning-1.md`. Everything else lives in this brief.

## 2. Scope

Produce one plan file at `.claude/PRPs/plans/v1-validate-agent.plan.md` for sub-phase **v1-validate-agent**. The plan covers the architectural change described in the originating design brief: a non-Junior cron-driven cargo validation runner (Layer C) that handles the 95% binary-pass-fail case, plus a Sonnet-4.6 `[role:validate]` Junior subagent (Layer A) that escalates only when cargo failure isn't pattern-matched.

**Source-of-truth references for scope:**

- `.claude/PRPs/briefs/validate-agent-design.md` — the originating design brief (load-bearing; the plan translates its §3 Solution shape, §4 Files, §5 DQ schema, §6 known-pattern catalogue, §7 wall-clock budget, §8 failure modes into the canonical 20-section plan).
- `.claude/decision-queue.json` resolved entries **#55–#60** — the clarify-pass output that gates this plan. Cite each by id in §2 Source of the plan and honour the answers literally:
  - **DQ #55** (advisor) — Layer A subagent runs under existing `junior@brehon-fork` cgroup (10G/8G); no new systemd unit for memory isolation.
  - **DQ #56** (advisor) — validate-runner.sh + systemd unit live at homeserver-level (`homeserver/scripts/brehon/`, `homeserver/systemd/`). Directory `homeserver/scripts/brehon/` does not yet exist; the plan must include a "create directory" step.
  - **DQ #57** (user) — validate subagent fixes ≤3 file edits inline; >3 escalates to a new impl-task brief. Encode the threshold in `validate.md` frontmatter or `validate-triage.md` hard-refusals.
  - **DQ #58** (user) — Layer C concurrency = tmp-dir clone. Each validate run does `git clone --shared --branch <commit_sha> /tmp/validate-<sha>/`, sets `CARGO_TARGET_DIR=/tmp/validate-<phase>/target/` (phase-scoped shared cache for warm reuse), runs cargo there, deletes source after (target persists until phase close).
  - **DQ #59** (advisor) — branch model = phase-branch + PR. Cut `phase-v1-validate-agent` off `governance-v0` via `bm-cut`; PR back with CodeRabbit review per `feedback_pr_per_phase.md`. **Override the design brief's §10.1 "direct to governance-v0" wording** — that line was the brief author misapplying `feedback_pr_per_phase` (which gates *individual* meta-commits, not whole sub-phases). The plan's §2 Source must cite DQ #59 explicitly.
  - **DQ #60** (user) — `.claude/agents/impl-task.md` change lands AFTER JM-d ships, BEFORE JM-e starts. The validate-agent PR's `bm-cut` happens only after JM-d's `bm-merge` lands on `governance-v0`. No concurrent phase branches. The plan must order Tasks accordingly: contract changes (impl-task.md, planning.md, plan.template.md) come last, gated on JM-d retro signoff.
- `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` — most recent shipped plan; mirror §1..§20 structure literally. Note: JM-d uses the **legacy §15 prose DoD shape**. The validate-agent plan introduces the new machine-readable §15 shape (per design brief §3.5) — mirror JM-d for §1..§14 + §16..§20, but author §15 as `<command, expected-exit-code>` pairs.
- `.claude/agents/{planning,impl-task,bm-task}.md` — canonical four-role contract; the new `validate.md` mirrors `impl-task.md`'s Sonnet-4.6 frontmatter shape (model, effort, narrow tools list).
- `.claude/rules/decision-queue.md` — current schema for the new `kind: "validate"` extension. The schema additive must update both the rule doc and the `decision-queue.json` consumer logic (validate-runner.sh) in the same commit per `feedback_update_rule_doc_with_schema_additive.md`.
- `.claude/rules/advisor-orchestrator.md` — "Stage-shape orchestration" sub-section gains a "validate stage" between impl-task complete and bm-cut/bm-pr. The plan must specify the exact wording of the new stage-line.
- `homeserver/CLAUDE.md` "Junior" sub-section — the 10G cgroup cap context for Layer A.
- `homeserver/library.yaml` — registration shape. Every new script (`validate-runner.sh`), agent (`validate.md`), command (`validate-triage.md`), lesson (`feedback_validate_subagent_known_patterns.md`) registers in the same commit that ships it per `feedback_library_add_after_shipping.md`.

**Hard out-of-scope for v1-validate-agent:**

- **Retrofitting v1-JM-d or any earlier plan.** Per design brief §3.5 mitigation + user plan-mode answer 2026-04-27 + `feedback_schema_changing_spec_retrofit_question.md`: forward-only. The plan must include an explicit "Retrofit scope: forward-only" section confirming pre-existing plans stay under their original §15 shape. JM-d's task #13 is mid-flight on `phase-v1-JM-d`; do not propose any change that affects it.
- **Cross-provider model trials** (MiniMax, Gemini, etc.) — Brehon stays Anthropic-only per `feedback_brehon_anthropic_only.md`. Validate subagent is Sonnet 4.6 (matches impl-task).
- **BM verb changes** (`bm-cut`/`bm-pr`/`bm-merge`/`bm-poll-cr`/`bm-triage`/`bm-merge` semantics) — validate is a sibling stage in the orchestrator, not a BM verb.
- **Replacing the §15 DoD definition itself** — we change WHERE DoD runs (out-of-band vs in-band) and HOW it's expressed (machine-readable vs prose), not WHAT it tests. Semantic content of DoD stays.
- **Auto-retry of failed validation** — `max_retries: 0` discipline in junior config.yaml stays. validate-failed surfaces; advisor decides; no auto-retry.
- **Investigation of PMD #109 (impl-task on Opus)** — separate diagnostic. Independent from this design but informs urgency.
- **Layer C false-pass diagnostics** — covered by JM-e first-trial retro per design brief §8; out of scope for the implementing plan itself.

**Plan file deliverable:**

- One file: `.claude/PRPs/plans/v1-validate-agent.plan.md`.
- Follow the canonical template at `.claude/commands/prp-core/prp-plan.md` literally — the 20-section structure is load-bearing.
- Plan style targets ~4–6 tasks per design brief §13 estimate, each task one commit, MED risk.
- §15 DoD validation commands MUST be executable as written against current `governance-v0` HEAD — dry-run each one before committing the plan. The new machine-readable shape (per DQ #58 + design brief §3.5) does NOT exempt §15 from the dry-run discipline; it just changes the format.
- §4 watchpoints MUST cite specific files / tables / `schema.rs` lines per `.claude/lessons/feedback_advisor_watchpoint_specificity.md` — never abstract concepts. The validate-runner's lock + flock model + tmp-dir cleanup are likely watchpoint targets.
- §13 must include `[P]` cohort markers per `.claude/PRPs/templates/plan.template.md` §13 + `feedback_parallel_cohort_dispatch.md`. Most validate-agent tasks touch disjoint files (Layer C scripts vs Layer A agent vs rule doc updates) and are likely cohort-compatible — but the planner judges file-set overlap mechanically, not heuristically. If §13 has no parallelisable tasks (every task touches an overlapping cross-cutting file), omit `[P]` markers entirely.
- §16a Stories block must enumerate end-to-end testable behaviours. Likely stories: "Layer C runs cargo against a committed phase-branch SHA and writes pass result to DQ", "Layer C runs cargo against a known-fail commit and writes fail-known result with suggested role", "Layer A subagent reads validate-failed entry and applies a ≤3-file fix", "Layer A escalates a >3-file fail-unknown to a new impl-task brief".

**Commit only the plan file.** Don't author Rust code, don't open PRs, don't touch any file under `crates/`, `migrations/`, or `tests/`. The Junior finalize step pushes the plan-file commit to `governance-v0`.

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract; defines model, tools, "Before you start (always)" sequence, plan-content discipline cross-references.
2. `.claude/commands/prp-core/prp-plan.md` — canonical Brehon plan template (the `/prp-core:prp-plan` command); structure is load-bearing.
3. **`.claude/PRPs/briefs/validate-agent-design.md`** — the originating design brief (load-bearing). Read entire file. The plan you write is the canonical translation of this brief's §3–§8 into a sub-phase plan.
4. `.claude/decision-queue.json` resolved entries **#55–#60** — the clarify-pass results. Each answer either decides a plan-shape question or overrides the design brief.
5. `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` — predecessor plan; mirror §1..§14 + §16..§20 shape exactly. Treat its §15 prose shape as the legacy reference; the new plan introduces machine-readable §15.
6. `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` — second-most-recent plan; mirror its `[P]` cohort marker placement and §16a Stories block (the canonical post-spec-kit pattern).
7. `.claude/agents/impl-task.md` — current contract that the validate-agent's plan §13 will modify (Task TBD per DQ #60: this change is gated on JM-d retro signoff, so it must be the **last** code-touching task in §13).
8. `.claude/agents/bm-task.md` — canonical Sonnet-4.6 + narrow-tools shape that `validate.md` will mirror.
9. `.claude/rules/decision-queue.md` — current `kind: "blocker"|"log"|"clarify"` schema; the plan extends it with `kind: "validate"` per design brief §5.
10. `.claude/rules/advisor-orchestrator.md` — current "Stage-shape orchestration" + "Mandatory user gates" + "Forbidden execution windows" sub-sections. The plan adds a "validate stage" between impl-task complete and bm-cut, plus updates "Forbidden execution windows" if validate-runner needs its own deferral logic (per design brief §7).
11. `.claude/PRPs/templates/plan.template.md` §13 + §15 + §16a — current template shape; the plan proposes the §13 `validate_path: out-of-band|inline` field addition + §15 machine-readable shape per design brief §4.2.
12. **Glob `.claude/lessons/` and Read every file whose name keyword matches `validate`, `dod`, `watchpoint`, `cargo`, `wrapper-script`, `features-full`, `clarify`, `parallel-cohort`, `read-canonical`, `dogfood-slash`, `update-rule-doc`, `schema-changing-spec`, `pr-per-phase`, `library-add`, `commit-aggressively`, `pipes-mask-exit-codes`, `clippy-test-style`.** That's the lessons corpus discipline per planning.md step 3.
13. `homeserver/CLAUDE.md` — the orchestrator/infra repo's contract. Read the "What This Repo Is", "Junior", "Project Memory (Operational)", and "Agentics Library" sub-sections. The cgroup cap context (10G/8G) is in "Junior".
14. `homeserver/library.yaml` — current registration shape. Every new artifact (script, agent, command, lesson) the plan creates must register here in the same commit per `feedback_library_add_after_shipping.md`.
15. `homeserver/systemd/` — Glob the directory, Read `restore-drill.timer`, `restore-drill.service`, `pmd-snapshot.timer`, `pmd-snapshot.service`. The validate-runner systemd unit mirrors this canonical timer-driven shape. **Do not** mirror `junior@.service` (that's a templated systemd unit, different pattern from a singleton timer).
16. `homeserver/systemd/junior@brehon-fork.service.d/*.conf` — the deployed cgroup cap drop-in. Confirms the 10G/8G figures cited in DQ #55.

## 4. Constraints (hard rules — violating any of these is a process breach)

**Plan-content discipline (per `.claude/lessons/`):**

- Every §15 DoD command must be executable against current HEAD. Dry-run each before commit, even in the new machine-readable shape. Per `feedback_pre_phase_dod_smoke_test.md` and `feedback_plan_dod_dry_run_at_write.md`.
- The new machine-readable §15 shape is `<command, expected-exit-code>` pairs (per design brief §3.5). Concretely: a YAML block or fenced code block that the validate-runner can parse mechanically. The exact format is a planner judgment call — surface as a planner-attributed DQ if the choice between YAML / JSON / structured Markdown affects the validate-runner's parsing logic.
- Never combine `-p <crate>` with `--features full` in any DoD command — only `--workspace --features full` works. Per `feedback_features_full_p_crate_incompatible.md`.
- Add `--no-deps` to any `clippy ... -- -D warnings` to avoid inheriting upstream lint debt.
- Use `bash scripts/brehon/cargo-{check,clippy,test}.sh` wrappers for cargo invocations — the .sh wrappers were fixed 2026-04-26 in commit `4571c53` to accept `$@` passthrough. Per `feedback_wrapper_script_flag_silence.md`. The validate-runner.sh likely calls these wrappers (or the equivalents in the tmp-dir clone) rather than raw `cargo`.
- Watchpoints in §4 cite specific files / tables / lines — never abstract concepts. Per `feedback_advisor_watchpoint_specificity.md`. For this plan: name the specific lines in `.claude/agents/impl-task.md` that change (the §15 self-run loop), the specific lines in `.claude/rules/advisor-orchestrator.md` "Stage-shape orchestration" that gain the validate stage, the specific lines in `homeserver/systemd/pmd-snapshot.timer` mirrored by validate-runner.timer.

**Canonical-first discipline (per `feedback_read_canonical_before_writing_spec.md`):**

- Before authoring §13 task entries that prescribe new agent shape (`validate.md`, `validate-triage.md`), Glob + Read the canonical examples (`impl-task.md`, `bm-task.md`, `commands/bm/<verb>.md`) and cite them in §10 Patterns to mirror with file paths + line ranges.
- Before authoring the new systemd unit shape, Read `homeserver/systemd/{restore-drill,pmd-snapshot}.{service,timer}` and cite them in §10 Patterns to mirror.
- Before authoring §15 in the machine-readable shape, walk through how an existing JM-d §15.1/§15.2/§15.3 prose entry would translate. If translation reveals semantic loss, surface as a DQ pending entry — the schema is under-specified.

**Dogfood discipline (per `feedback_dogfood_slash_command_specs.md`):**

- Every new slash command (`validate-triage.md`) must include a "Pre-commit dogfood" sub-section. Mental walk-through against a real existing input — for `validate-triage.md`, that's a hypothetical fail-unknown DQ entry shape from design brief §5.

**Schema discipline (per `feedback_update_rule_doc_with_schema_additive.md`):**

- The `kind: "validate"` schema extension and the `decision-queue.md` rule update happen in the same commit. The plan §13 task that adds `kind: "validate"` to the rule doc is the same task that adds it to validate-runner.sh's parser.

**Library registration discipline (per `feedback_library_add_after_shipping.md`):**

- Every new script (`validate-runner.sh`), agent (`validate.md`), command (`validate-triage.md`), lesson (`feedback_validate_subagent_known_patterns.md`), brief template (`validate-brief.template.md`) must be registered in `homeserver/library.yaml` in the same commit that ships it. The 3rd-recurrence pattern (per `homeserver/docs/memory/PATTERNS.md` "library.yaml registration lag") must not happen on this sub-phase. Dedicated §17 Completion checklist line.

**Commit-discipline (per `feedback_commit_aggressively_in_shared_repos.md`):**

- Each `.claude/` edit commits per-edit, not per-session. The validate-agent sub-phase touches `.claude/` files across multiple repos (brehon-fork: agents/, rules/, commands/, lessons/; homeserver: scripts/, systemd/, library.yaml, lessons/). Commits stay tight per task.

**Forward-only retrofit discipline (per `feedback_schema_changing_spec_retrofit_question.md` + DQ #60):**

- Plan must include an explicit "Retrofit scope" section confirming forward-only. Pre-existing plans (v1-JM-d) stay under prose §15. New §15 shape applies to v1-JM-e and later.
- The plan must NOT propose any task that mutates v1-JM-d's plan, briefs, or in-flight worktrees.

**Branch + commit discipline (per DQ #59):**

- Phase branch: `phase-v1-validate-agent`, cut off `governance-v0` via `bm-cut`. **Do not cut while v1-JM-d's `phase-v1-JM-d` is still active.** Per DQ #60: `bm-cut` for validate-agent runs only after JM-d's `bm-merge` confirms `governance-v0` is updated. The plan §17 Completion checklist must enforce this ordering.
- One commit per task. Final PR back to `governance-v0` with CodeRabbit review per `feedback_pr_per_phase.md`.

**Decision-queue discipline (per `.claude/rules/decision-queue.md`):**

- If you (planning subagent) seed any DQ entry in the plan, label it `answered_by: "planner"` — never `"advisor"`. Per the attribution rule §"Subagents and attribution."
- If you discover a question that needs the persistent advisor session to answer (ADR-affecting, scope-changing, judgment-heavy that wasn't covered by the existing #55–#60 clarify pass): leave the entry as `pending` with `answered_by: null` — do NOT pre-answer with advisor wording.
- Mid-task DQ writes from this Junior worktree must commit + push immediately, not wait for finalize. Per `decision-queue.md` §"Mid-task visibility (Junior worktrees)." Subject pattern: `chore(decision-queue): planner raised DQ #<id> — <slug>`. Push to `junior/validate-agent-planning-1` (this worktree's branch), not to `governance-v0` directly — the advisor's polling loop fetches all branches.

**File ownership:**

- Touch only `.claude/PRPs/plans/v1-validate-agent.plan.md` (CREATE) and (optionally) `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries).
- Do NOT edit anything under `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, or any other plan file in `.claude/PRPs/plans/`.
- One commit at finalize: `feat(plan): v1-validate-agent sub-phase plan` (or `docs(plan):` if your project convention prefers — match the existing JM-d plan's commit subject).

**Boundary-of-judgment (when to STOP and queue rather than guess):**

- If design brief §3 Layer C/A boundaries contradict §6 known-pattern catalogue → DQ pending.
- If the proposed `validate.md` Sonnet-4.6 frontmatter conflicts with `impl-task.md`'s established shape (model, effort, tools list) → DQ pending.
- If the proposed `kind: "validate"` schema clashes with existing `kind: "blocker"|"log"|"clarify"` consumers (any rule doc, any subagent contract, any script that reads decision-queue.json) → DQ pending.
- If the proposed plan-template §15 machine-readable shape can't represent any existing JM-d §15 entry (§15.1 static analysis, §15.2 lint, §15.3 test target compile, §15.4 e2e execution, §15.5 migration round-trip, §15.6 cross-cutting verification, §15.7 manual SQL) without semantic loss → DQ pending. The schema is under-specified.
- If `homeserver/scripts/brehon/` doesn't exist (Phase 1 confirmed: it doesn't), the plan must include a "create directory" step in Task 0 or Task 1.
- If `homeserver/systemd/` patterns mirror differently than expected (e.g. `pmd-snapshot.timer` uses `OnCalendar=` while the design brief implies `OnUnitActiveSec=60`), surface as §4 watchpoint, not blocker.

**Attribution integrity reminder:** the only valid `answered_by` labels for a Junior planning subagent are `"planner"` (forward-looking pre-resolved entries you have a recommendation on) or `null` (genuinely needs advisor input). Never `"advisor"`, never `"user"`, never `"impl-self-resolved"` (that's an impl-task label).

---

**Lean / advisor-side tip (not a constraint):** the validate-agent sub-phase's biggest plan-shape risk is the **temporal coupling** between Layer C's tmp-dir clone (DQ #58), the systemd timer cadence (design brief §7: 60s tick), and the cargo cache state. The first validation per phase is cold — 35 minutes upper bound. If the timer fires before the validate-pending DQ entry has been written (impl-task commits → push → DQ entry append → push, all on a Junior worktree), the runner picks up nothing and idles. Conversely, if multiple validate-pending entries arrive before the timer fires, the flock serialises them — but the queue could pile up. The plan should specify: (1) the validate-runner's ordering rule (FIFO by DQ id? by commit timestamp?), (2) the pile-up handling (drain all pending in one tick? one-per-tick?), (3) the timer's `OnUnitActiveSec=` vs `OnCalendar=` choice. Mirror `homeserver/systemd/pmd-snapshot.timer` if it solves an analogous problem (it doesn't — pmd-snapshot is event-driven, not queue-draining); if MIRROR examination shows pmd-snapshot is the wrong pattern, surface as §4 watchpoint and propose an alternative (e.g. event-driven via inotify on `.claude/decision-queue.json`, or a long-running daemon with internal polling rather than systemd-timer-driven). DQ #58's tmp-dir clone path also adds a cleanup concern: `/tmp/validate-<sha>/` is removed after each run, but `CARGO_TARGET_DIR=/tmp/validate-<phase>/target/` persists until phase close — who deletes it on `bm-merge`? Likely a `bm-merge` post-action (mechanical) — surface as §4 watchpoint.
