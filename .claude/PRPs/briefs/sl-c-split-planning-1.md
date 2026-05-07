# SL-c split-planning brief

**Written:** 2026-05-07 by advisor session (laptop, brehon-fork CWD) for local Agent subagent_type=planning dispatch.
**Subagent target:** `planning` (Opus 4.7 max, color purple — see `.claude/agents/planning.md`).
**Worktree:** advisor sibling worktree at `/Users/barrie/Developer/lemmy-advisor-sl-c` on `governance-v0` (no Junior cut; subagent commits + the advisor pushes per `feedback_branch_manager_pm_split.md`).
**Authority anchors:**
- Trunk plan: `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` (3915 lines, committed `7af7dfa93`).
- Override DQ: `#150` (advisor) records user's split-into-c-1-plus-c-2 directive at plan-approval gate 2026-05-07.
- Override origin: trunk plan §5.2 + planner DQ #148 (proceed-as-one lean) overridden by user.

## §1. Role + dispatch line

`[role:planning]` Split the trunk SL-c plan into c-1 (module + scheduler wiring + retro) + c-2 (5 e2e tests + retro), preserving §1-§12 + §17-§20 verbatim across both plans.

## §2. Scope

### §2.1 Deliverables

Produce **two** plan files at:
1. `.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md` — Tasks 0, 1, 2, retro (~7 tasks).
2. `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` — Tasks 0 (re-run pre-flight on c-2 phase branch), 3, 4, 5, 6, 7, retro (~7 tasks).

Both plans inherit the trunk plan's §1 (Goal), §2 (Anti-goals — minus the proceed-as-one rationale), §3 (Problem statement), §4 (Solution statement — including all 14 watchpoints as-is), §6 (Relationship to other v1-SL sub-phases — c-2 adds c-1 as upstream MERGED dep), §7-§12 verbatim, §17-§20 verbatim.

Sliced sections per plan:
- **§5 Metadata:** rewritten per plan (phase slug, branch name, complexity score, no DQ #148 reference — DQ #150 is the binding precedent).
- **§13 Step-by-step tasks:** sliced at Task-2/3 boundary. c-1 inherits Tasks 0, 1, 2 + a new Task 3 retro. c-2 inherits a new Task 0 (pre-flight on c-2 branch confirming c-1 module shipped) + trunk Tasks 3-7 + a new Task 6 retro.
- **§14 Testing strategy:** c-1 §14 names the workspace-check expectation only (no e2e in c-1 since tests aren't authored yet). c-2 §14 names the 5 e2e tests + the e2e local-vs-dispatch user gate.
- **§15.4 Cross-cutting verification:** c-1 §15.4 omits the `grace_check_` test count assertion. c-2 §15.4 keeps it (expect 5).
- **§16 Acceptance criteria:** rewritten per plan task list.
- **§16a Stories:** c-1 ships Story 1 only (module + scheduler wiring). c-2 ships Stories 2 + 3 (fire branch; escape + isolation + batch + no-op).

### §2.2 Boundaries (do NOT)

- **Do not author Rust code** in either plan. Plans are specs, not implementation.
- **Do not modify the trunk plan** (`v1-sponsor-liability-c.plan.md`) — it stays in place as the historical record. Split plans coexist with trunk.
- **Do not re-litigate watchpoints.** The 14 watchpoints in trunk §4.2 are all carried into both split plans verbatim. If a watchpoint is c-1-only or c-2-only, annotate inline (e.g. "Watchpoint #12: e2e Edit-per-task — applies to c-2 only; c-1 has zero e2e edits").
- **Do not author new ADRs or new DQ entries.** DQ #144-#149 are resolved; DQ #150 records the split decision. No new DQ pre-seeds needed (per §5 of the agent contract — pre-seed only forward-looking OQs not already resolved).
- **Do not re-evaluate compute/fire posture.** Trunk §4.1 binding: SL-c calls unsplit v0 `apply_sponsor_liability`; SL-d does the split. Carry forward verbatim.
- **Do not re-evaluate restoration-stub.** Trunk DQ #145 + §4.2 watchpoint #14 binding: stub-only; never fires. Carry forward verbatim.

## §3. Required reading

Read in order; cite by file:line in the produced plans where each anchor is referenced:

### Trunk plan (the single most important input)

- `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` — read end-to-end. The split is mechanical at the §13 Task-2/3 boundary. Preserve everything else.

### Override + clarify DQ

- `.claude/decision-queue.json` — DQ #144 (two-tier ConfigCache), #145 (restoration-stub-only), #146 (staleness multiplier formula), #147 (paired canonical mirrors), #148 (planner's split-or-proceed lean), #149 (baseline_sponsor_count → option-B forward-looking record), **#150 (split override — binding)**.
- `.claude/runlog/advisor-relays/adhoc-sl-c-baseline-sponsor-count.md` — context for DQ #149's forward-looking nature; user adjudication on the relay scope conflict was "keep planner's scope" (SL-c does NOT evaluate `majority_revocation`).

### Lessons (consult before sketching split shape)

- `.claude/lessons/feedback_complexity_score_pre_split.md` — split-or-proceed mechanics; c-2 still scores ~16 (above 8) per §5.2.
- `.claude/lessons/feedback_principles_not_rules.md` — score is a signal, not a hard rule; user's split preference overrides the lean.
- `.claude/lessons/feedback_advisor_watchpoint_specificity.md` — every watchpoint must cite file:line. Trunk §4.2 already conforms; preserve discipline in split plans.
- `.claude/lessons/feedback_plan_dod_dry_run_at_write.md` — DoD smoke at plan-write time; both split plans must have §15.4 commands executable.
- `.claude/lessons/feedback_brehon_verify_pre_merge.md` — §16a stories must be `[done]`-confirmable per `/brehon-verify`.
- `.claude/lessons/feedback_story_grain_checkpoint.md` — story grain rules. c-1 ships Story 1 only (one independently-testable behaviour unit; verifiable structurally because its e2e tests don't yet exist).

### PRD + ADRs

- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` §6, §9.4, §9.5, §15 — same anchors as trunk planning brief.
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — ADRs cited by trunk §15.5: ADR-005, ADR-008, ADR-010, ADR-013, ADR-014, ADR-015; OQ-V1-SL-05.

### Sibling-phase reference plans (split precedent if any)

- `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — proceed-as-one at score 15.
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — proceed-as-one at score 13.
- `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` — proceed-as-one at score 9 (closest to c-1's projected ~7).

(There is no in-fork precedent for an at-write-time split — c-1/c-2 will be the first. That's not a blocker; it just means the split is a planning act, not a copy-paste from precedent.)

## §4. Constraints

### §4.1 Hard constraints inherited from trunk + advisor-orchestrator.md

- **Watchpoint specificity:** every watchpoint cites a specific table, file, or `schema.rs` line per `feedback_advisor_watchpoint_specificity.md`. Trunk §4.2 conforms; split plans MUST conform.
- **DoD executability:** every §15 command must run cleanly at plan-write time per `feedback_plan_dod_dry_run_at_write.md`. Re-run all §15.4 commands against the worktree at split-write time; document expected pre-state outputs in §15.7 manual snippets.
- **Stories `[done]` discipline:** §16a stories must each name composing tasks + a checkpoint workflow + Brief-Scope structural patterns per `feedback_brehon_verify_pre_merge.md`.
- **No new ENTRY_KIND_*** consts:** trunk watchpoint #10 binding. Split plans MUST NOT introduce.
- **No migrations:** trunk watchpoint #11 binding. `git diff governance-v0..phase-v1-SL-c-N -- migrations/` MUST be empty.
- **R1 i64 discipline:** trunk watchpoint #13 binding.
- **Per-case FOR UPDATE:** trunk watchpoint #1 binding (lives in c-1's module).
- **ADR-015 actor_pseudonym:** trunk watchpoint #4 binding (lives in c-1's module).
- **e2e Edit-per-task:** trunk watchpoint #12 binding (applies to c-2 only — c-1 has zero e2e edits).

### §4.2 Phase-branch + topology constraints

- c-1 phase branch: `phase-v1-SL-c-1` (cut by BM-task from `governance-v0` AFTER SL-b PR #119 merges).
- c-2 phase branch: `phase-v1-SL-c-2` (cut by BM-task from `governance-v0` AFTER c-1 PR merges; c-2 §6 must name c-1 as upstream MERGED dep).
- No cross-branch authoring: c-2 plan does NOT depend on c-1's worktree being live; it depends on c-1 having merged to `governance-v0` so c-2's `cargo check --workspace` sees the c-1 module.
- Both plans target `--repo barrie-cork/lemmy --base governance-v0` per `gh-pr-fork-target.md`.

### §4.3 Complexity score per split plan

Recompute §5.1 per plan using the formula in `feedback_complexity_score_pre_split.md`:

- **c-1:** 2 impl tasks, 0 migrations, 2 crates (`lemmy_api`, `lemmy_routes`), 0 e2e edits, 0 ADR-affecting decisions, 0 cargo budget. Subtotal: `max(0, 2-5) + 0 + 2 + 0 + 0 + 0 = 2`. Adjustment: +1 for restoration-stub discipline (carries from trunk §5.1 +1). **Total: 3.** Below threshold. No DQ #148-class entry.
- **c-2:** 5 impl tasks, 0 migrations, 1 crate (`lemmy_server`), 5 e2e edits (5 × +3 = 15), 0 ADR-affecting decisions, 0 cargo budget. Subtotal: `max(0, 5-5) + 0 + 1 + 15 + 0 + 0 = 16`. Adjustment: +1 for c-1 dependency check at Task 0. **Total: 17.** Above threshold. **The c-2 plan MUST file a planner-self-resolved split-or-proceed DQ** per `feedback_complexity_score_pre_split.md`. Recommended self-resolve: proceed (further splitting fragments the e2e suite — each test is anchor-Edit-friendly; bundle is justified per `feedback_principles_not_rules.md`). Cite DQ #150 as precedent for THIS split level + lessons for not re-splitting.

### §4.4 Other

- **`feedback_dogfood_slash_command_specs.md`** does NOT apply (this brief authors plans, not slash commands).
- **`feedback_clarify_before_plan.md`** SKIPPED with justification: trunk plan was clarified at write time (DQ #144-#147); DQ #149 + #150 are post-write user-adjudicated decisions; no new ambiguity surface to disambiguate. The advisor commits both plans with subject `docs(plan): split SL-c per DQ #150` documenting the justification per `advisor-orchestrator.md` "Clarify gate" sub-section.
- **HANDOVER trailer:** N/A (planning subagent does not write impl-style HANDOVER trailers; that's for impl-task per `feedback_handover_trailer_cohort_propagation.md`).

## §5. Validation at brief-write time

The advisor confirmed at brief-write time:

1. Trunk plan exists at expected path (`.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` @ `7af7dfa93`).
2. DQ #150 in `resolved` array (advisor-self-resolved blocker, override).
3. SL-b PR #119 status: still in flight on `phase-v1-SL-b` per advisor's polling-loop knowledge; both c-1 and c-2 phase branches gated on SL-b merging first.
4. Surety schema columns confirmed at `crates/db_schema_file/src/schema.rs` (created_at + revoked_at present); DQ #149's option-B is feasible.
5. Worktree clean (`git status --short` empty).
6. Branch is `governance-v0` (`git branch --show-current` — confirmed at `9a0909814`).

## §6. Pre-flight before authoring split plans

Subagent first read:

1. The trunk plan end-to-end. Note every section's exact byte boundary so the slice at Task-2/3 is clean.
2. DQ entries #144-#150 in `decision-queue.json`. Confirm #150 is the binding override.
3. The relay file `adhoc-sl-c-baseline-sponsor-count.md`.
4. The lessons in §3 above.

Subagent's output: two plan files committed to `governance-v0` in **one commit** with subject:

```
docs(plan): split v1-SL-c into c-1 + c-2 per DQ #150
```

Body cites DQ #150 + the user's split-into-c-1-plus-c-2 directive at plan-approval gate 2026-05-07.

## §7. Notes for the subagent

- The trunk plan is **comprehensive** (3915 lines). Most of the substance carries forward unchanged. The split is structurally mechanical: slice §13/§14/§15.4/§16/§16a at Task-2/3; rewrite §5 metadata + acceptance criteria per plan; everything else is verbatim or near-verbatim with annotations like "(c-1 only)" / "(c-2 only)" where applicable.
- **Don't shrink the trunk plan's content.** Both split plans should be self-contained — a reader of c-2 should understand the architecture without needing c-1 open. Better to repeat §1-§12 than to under-cite.
- **c-2 §6 (Relationship to other v1-SL sub-phases) MUST add a row for c-1.** Like:
  | Sub-phase | Status | What it ships | c-2 dependency |
  | v1-SL-c-1 | MERGED (PR #<TBD>, governance-v0 @ `<sha-TBD>`) | `sponsor_liability_grace.rs` module + scheduler wiring + atomic guard pair | c-2 reads `pub async fn run_grace_check_batch`, `evaluate_escape_conditions`, `fire_or_escape_case`, `check_grace_staleness`; c-2 tests pre-seed `SponsorLiabilityPending` cases via direct DB-write and invoke `run_grace_check_batch` directly (per trunk plan §13 Tasks 3-7). |
- **c-1 retro task:** trunk Task 8 retro signals carry over but the §5 retro topic shifts ("did the c-1 module + scheduler wiring ship cleanly?"). Note that c-1 doesn't yet have e2e tests, so c-1 retro acknowledges the c-2 dependency for behavioural validation.
- **c-2 Task 0 (pre-flight):** must include a probe verifying c-1 module is on `governance-v0` (e.g. `rg -n 'pub mod sponsor_liability_grace;' crates/api/api/src/governance/mod.rs` returns 1 line; the four function signatures present in the module file). Without this probe, c-2 risks being branched off a stale tip.
- **§15.7 DoD smoke at plan-write time:** re-run §15.4 commands per plan against the worktree (currently `governance-v0` @ `9a0909814`); document expected pre-c-1 / pre-c-2 outputs in each plan's §15.7. The c-2 §15.7 will need to refer to "post-c-1-merge" expected state for module/guard/test counts.

## §8. Output expectations

Two plan files at canonical paths. One commit. Subagent reports:

- Final SHA(s) of the commit(s).
- Per plan: complexity score with breakdown.
- Whether c-2 filed a planner-self-resolved split-or-proceed DQ (expected: yes, score 17 trips threshold).
- Any DQ entries pre-seeded (expected: none new beyond the c-2 split-or-proceed self-resolved per Recipe 2).
- Open questions for advisor at plan-approval gate.

The advisor will run watchpoint specificity + DoD smoke gates on each plan before surfacing to user for the final plan-approval gate.
