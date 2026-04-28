# Synthetic spec-trial-1 planning brief (V1 additives dogfood gate)

**Written**: 2026-04-27 by advisor session (homeserver CWD) for Junior dispatch on EliteDesk
**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`)
**Worktree**: Junior cuts `junior/synthetic-spec-trial-1-planning-1` from `governance-v0` per its concurrency-1 default; the planning subagent commits the plan file and pushes back to `governance-v0` at finalize.

**Special status — this is a dogfood brief.** It exists solely to exercise the V1 spec additives (per `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md`, `feedback_complexity_score_pre_split.md`, `feedback_handover_trailer_cohort_propagation.md`) end-to-end before any real-phase use. The plan it produces will be **read but not implemented** — no `crates/**` edits, no migrations, no e2e tests. The deliverable is a **synthetic plan file** that deliberately exercises every new additive in §5/§13/§16a, plus a dogfood report at `.claude/PRPs/reports/synthetic-spec-trial-1-dogfood.md` capturing what worked + what didn't. After the gate, the synthetic plan + worktree may be deleted; the report stays.

## 1. Role + dispatch line

`[role:planning] synthetic-spec-trial-1 plan — dogfood V1 additives (FILES YAML, complexity score, HANDOVER trailer)`

The actual `mcp__junior-brehon__create_task` description is one line: `[role:planning] synthetic-spec-trial-1 — see .claude/PRPs/briefs/synthetic-spec-trial-1.md`. Everything else lives in this brief.

## 2. Scope

Produce one plan file at `.claude/PRPs/plans/synthetic-spec-trial-1.plan.md` that **deliberately exercises every V1 additive**. The plan is synthetic — its §13 IMPLEMENT lines reference fictitious file paths under `crates/synthetic_demo/**` (which does not exist and will never exist). No real Brehon code is touched. The plan is the artifact under test, not its implementation.

The synthetic plan must hit the following test-shapes (all four are mandatory — drop any one and the dogfood gate fails):

**Test-shape A — Cohort overlap refusal (Opp 1):**

- Two `[P]`-marked §13 tasks whose **FILES** YAML `creates:` or `modifies:` arrays **share at least one path** (deliberately).
- The planner-side discipline (per `feedback_explicit_file_arrays_on_tasks.md`) says `union(creates, modifies)` across `[P]`-cohort peers must be disjoint. Authoring overlap is a deliberate planner-side miss for the dogfood — **note the violation explicitly in §19 Notes** so the dogfood reader knows it's intentional, not a bug.
- The advisor's cohort-dispatch step 4 (`.claude/rules/advisor-orchestrator.md` "Cohort dispatch sequence") must refuse this cohort and degrade to serial. The dogfood report captures whether the refusal fires correctly.

**Test-shape B — Complexity-score DQ escalation (Opp 2):**

- §5.1 factor breakdown produces a total score `> 8`. Easiest: claim 4 e2e.rs edits in §13 (4 × +3 = +12 from e2e factor alone). Other factors can be zero.
- The planner must file a `pending` DQ entry on this brief with `from: "planner"`, `kind: "blocker"`, question shape per `feedback_complexity_score_pre_split.md` ("complexity N exceeds threshold — split into <slug>-1 + <slug>-2, or proceed?"). Commit + push the DQ entry per the mid-task discipline before pushing the plan.
- The advisor's "Plan §5 complexity-score awareness" sub-section reads the score + filed DQ. For dogfood, the advisor will answer the DQ in `--mode advisor` with `proceed (synthetic — no real cargo work)` so the synthetic plan ships without requiring a split.

**Test-shape C — HANDOVER trailer cohort propagation (Opp 3):**

- §13 has at least one cohort of two `[P]`-marked tasks (these can be a *different* pair from test-shape A — see §13 layout below) whose `creates:` / `modifies:` arrays are genuinely disjoint, and a non-`[P]` barrier task immediately after.
- The plan's §13 task body for the `[P]` pair includes a worked example of the HANDOVER trailer the impl-task subagent would write (per `.claude/agents/impl-task.md` "HANDOVER trailer (cohort-internal-share, opt-in)"). Place under a clearly-marked "Synthetic example HANDOVER trailer (illustrative; impl-task does not run)" sub-block in §19 Notes — NOT inside the §13 task body itself, since plans are read by the impl-task subagent and a synthetic trailer in the wrong place could mislead.
- The impl-task brief template's §3a "Handover from prior cohort" is exercised at the dogfood-report level only — describe what a populated §3a would look like for the barrier task, given the synthetic example trailer. Do not author or commit any impl-task brief; the synthetic plan stops at the planning artifact.

**Test-shape D — `/brehon-verify` two-layer Step 4a (Opp 1, second consumer):**

- §16a Stories block authored normally (per `.claude/PRPs/templates/plan.template.md` §16a). At least one story has Brief-Scope outputs whose paths match `creates:` entries in the composing §13 tasks.
- Deliberately omit one Brief-Scope output bullet from §16a that's named in §13's `creates:` — to test that the verify command's "[malformed] §13 FILES YAML drifts from §16a Brief-Scope outputs" classification fires.
- Note the deliberate drift in §19 Notes for dogfood-reader clarity.

**§13 task layout for synthetic plan (4 tasks total, fits the test-shapes above):**

```
Task 0: Pre-flight harness audit (always non-[P], no cohort, no FILES YAML — Task 0 is verification only per template)
Task 1 [P]: Synthetic creates a + modifies b (creates: [crates/synthetic_demo/src/a.rs], modifies: [crates/synthetic_demo/src/lib.rs])
Task 2 [P]: Synthetic creates b + modifies a (creates: [crates/synthetic_demo/src/b.rs], modifies: [crates/synthetic_demo/src/a.rs])    ← OVERLAP with Task 1's `creates: a.rs` — Test-shape A
Task 3: Synthetic barrier (non-[P], creates: [crates/synthetic_demo/src/c.rs], modifies: [crates/synthetic_demo/src/lib.rs])    ← cross-cohort overlap with Task 1's modifies (intentional, demonstrates non-[P] tasks declare YAML too)
Task 4: Retro task (always non-[P], no FILES YAML)
```

Test-shape C's "genuine cohort" requirement collapses into Task 1 + Task 2 with Task 1's `creates: a.rs` reframed in the dogfood-report-side example. Document this collapse in §19 Notes — "Test-shape C is illustrated via §19 example block; the synthetic plan's actual §13 cohort always trips Test-shape A first." That's correct dogfood scope: the synthetic plan demonstrates the *refusal mechanism* (A); the *successful propagation* (C) is illustrated in the report.

For test-shape B, mark §13 tasks as touching e2e.rs — assign `crates/lemmy_server/tests/e2e/synthetic_demo.rs` to the `creates:` of Task 1 (in addition to `crates/synthetic_demo/src/a.rs`), and three more synthetic e2e files (`tests/e2e/synthetic_demo_2.rs`, `tests/e2e/synthetic_demo_3.rs`, `tests/e2e/synthetic_demo_4.rs`) to Tasks 2, 3, 4 respectively. These count as 4 e2e.rs edits in §5.1 → +12. Total complexity `> 8` is automatic.

**Hard out-of-scope for the synthetic plan:**

- Any real `crates/**`, `migrations/**`, `tests/**` change. The `crates/synthetic_demo/**` and `tests/e2e/synthetic_demo*.rs` paths are deliberately fictitious.
- Any §15 DoD command that names real cargo invocations against the synthetic crate. §15 entries should be no-ops or reference the synthetic shape only — explicitly call out "synthetic plan — DoD commands not executable" in §15 header.
- Any real `bm-cut` / `bm-pr` / `bm-merge` follow-up. The synthetic plan never gets implemented; therefore no PR opens.
- Any file under `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, or any other plan file in `.claude/PRPs/plans/` (write only `synthetic-spec-trial-1.plan.md`).

**Commit only the plan file + the planner-side DQ entry** (planner-DQ for test-shape B is mandatory per the constraints below). Two commits at finalize, in order: `chore(decision-queue): planner raised DQ #<id> — synthetic-spec-trial-1 complexity > 8` then `feat(plan): synthetic-spec-trial-1 dogfood plan`. Junior's finalize step pushes both to the worktree branch; the advisor reads `.claude/decision-queue.json` on next polling tick.

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract; read in particular the new sections "§13 per-task creates/modifies YAML block (load-bearing)" and "§5 complexity score + split threshold (load-bearing)" added in commit `feb5c688c`
2. `.claude/PRPs/templates/plan.template.md` — the canonical 20-section schema with the four spec-kit-derived additives ([P], §16a, §5 score, §13 FILES YAML); structure is load-bearing
3. `.claude/PRPs/templates/impl-task-brief.template.md` — for the §3a "Handover from prior cohort" shape (referenced from the synthetic plan's §19 Notes example)
4. `.claude/agents/impl-task.md` — for the HANDOVER trailer shape under "Per-task commit shape" (referenced from §19 Notes example)
5. `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — the FILES YAML discipline + cohort-overlap mechanics being dogfooded
6. `.claude/lessons/feedback_complexity_score_pre_split.md` — the §5.1 factor weights + threshold > 8 escalation being dogfooded
7. `.claude/lessons/feedback_handover_trailer_cohort_propagation.md` — the HANDOVER trailer shape + §3a injection mechanics being dogfooded
8. `.claude/lessons/feedback_parallel_cohort_dispatch.md` — the underlying [P] marker discipline the FILES YAML reinforces
9. `.claude/lessons/feedback_schema_changing_spec_retrofit_question.md` — confirms why this dogfood ships forward-only (synthetic plan exists to prove the V1 pipeline; no retrofit question applies, since the synthetic plan IS the new shape)
10. `.claude/rules/advisor-orchestrator.md` — read "Cohort dispatch sequence" steps 4 + 9, "Cohort handover aggregation" sub-section, "Plan §5 complexity-score awareness" sub-section. The advisor enforces these on the dogfood plan exactly as on a real plan; the synthetic plan must be syntactically parseable by the advisor's logic.
11. `.claude/commands/brehon-verify.md` — read the new two-layer Step 4a + the V1 schema additive sub-section in `<rationale>`. The synthetic plan is what `/brehon-verify` would parse if it were ever run; the test-shape D drift is what would trip [malformed].
12. `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` — a shipped plan example (pre-V1 — no FILES YAML, no §5 score). Read for §13 and §16a structural reference; understand that pre-V1 plans are explicitly back-compat in the V1 additives.
13. `.claude/PRPs/briefs/jm-d-planning-1.md` — a shipped planning brief example (pre-V1). Mirror its 4-section structure (Role + dispatch / Scope / Required reading / Constraints) at the brief level even though the plan you author is V1+.

## 4. Constraints (hard rules — violating any of these is a process breach)

**Plan-content discipline (V1 additives — the dogfood is testing these):**

- §5 Metadata MUST include `Complexity score: <N>/10` line.
- §5.1 factor breakdown table MUST be present, MUST sum to a score `> 8`, and MUST show the e2e.rs factor row dominating (≥ +12 from 4 e2e edits per the §13 layout above).
- §13 task headers MUST carry `[P]` markers per the layout (Task 1 + Task 2 are `[P]`; Tasks 0 + 3 + 4 are non-`[P]`).
- §13 every task header (except Task 0 and Task 4 retro per template) MUST carry the **FILES** YAML block immediately above its IMPLEMENT prose. `creates:` and `modifies:` arrays MUST match the layout in §2 (deliberate overlap between Task 1 + Task 2; deliberate non-overlap between Task 1 + Task 3 except via `modifies: lib.rs` cross-cohort).
- §13 IMPLEMENT lines must name the same files as the YAML's `union(creates, modifies)` for each task — drift between YAML and IMPLEMENT in a *deliberate* way is **not** authorised here; only the cohort-overlap (across tasks) is the deliberate test. Within a task, YAML and IMPLEMENT match.
- §16a Stories block MUST be present with at least 2 stories. One story's Brief-Scope outputs MUST omit one path that's named in the composing §13 tasks' `creates:` arrays — this is the test-shape D deliberate drift.
- §15 Validation commands header MUST explicitly say "synthetic plan — DoD commands not executable; this plan is the artifact under test, not its implementation." No real cargo invocations. §15 entries can be one-line placeholders or omitted entirely.

**Decision-queue discipline (test-shape B):**

- A `pending` DQ entry MUST be filed by the planning subagent before pushing the plan. Shape:
  - `from: "planner"`
  - `kind: "blocker"`
  - `question: "complexity <N> exceeds threshold — split into synthetic-spec-trial-1-1 + synthetic-spec-trial-1-2, or proceed?"` (substitute the actual computed N)
  - `options: ["split into synthetic-spec-trial-1-1 (Tasks 0-2) + synthetic-spec-trial-1-2 (Tasks 3-4)", "proceed despite score (synthetic plan — no real implementation, dogfood gate)"]`
  - `answered_by: null`, `answer: null`, `resolved_at: null`
  - `id: <next>` — read the current max id from `.claude/decision-queue.json` and increment
  - `timestamp: <ISO 8601 UTC at write time>`
- Commit + push the DQ entry per the mid-task discipline (`.claude/rules/decision-queue.md` §"Mid-task visibility (Junior worktrees)") with subject `chore(decision-queue): planner raised DQ #<id> — synthetic-spec-trial-1 complexity > 8`. Push to the worktree branch; advisor's polling fetches all branches.
- After committing the DQ, proceed to commit the plan file. Do NOT block waiting for the DQ to resolve — for this dogfood, the planner ships and the advisor self-resolves on next poll.

**Notes-section discipline (§19):**

- §19 Notes MUST contain a "Dogfood test-shapes exercised" sub-section enumerating: A (cohort overlap refusal — Tasks 1+2 share `crates/synthetic_demo/src/a.rs`), B (complexity DQ escalation — score > 8 from e2e factor), C (HANDOVER illustration — example trailer shown), D (verify [malformed] from §16a-vs-§13 drift — one Brief-Scope output omitted).
- §19 MUST include a "Synthetic example HANDOVER trailer (illustrative; impl-task does not run)" sub-block showing what Task 1 + Task 2's commit-body trailer would look like under real impl.
- §19 MUST include a "Test-shape D deliberate drift" sub-block naming the §13 `creates:` path and the §16a story it's omitted from, so the dogfood report knows what to look for.

**File ownership:**

- Touch only `.claude/PRPs/plans/synthetic-spec-trial-1.plan.md` (CREATE) and `.claude/decision-queue.json` (APPEND test-shape B entry).
- Do NOT edit anything under `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/PRPs/templates/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, `.claude/lessons/`, or any other plan file in `.claude/PRPs/plans/`.
- Do NOT author the dogfood report at `.claude/PRPs/reports/synthetic-spec-trial-1-dogfood.md` — that's authored by the advisor session after walking through the dogfood pipeline (clarify → planning → cohort attempt → verify). The planning subagent's deliverable stops at the synthetic plan + the test-shape B DQ.

**Boundary-of-judgment:**

- If any V1 additive's exact shape is unclear from the lessons + templates, file a `pending` DQ with `from: "planner"`, `kind: "clarify"`, asking the advisor for the canonical answer. Do not guess — the synthetic plan must be V1-conformant or the dogfood signal is meaningless.
- If the §5.1 factor table cannot be computed without invoking conventions the lessons don't document (e.g. how to count "ADR-affecting decisions" for a synthetic plan that has no real ADR ties), file a clarify DQ rather than guessing the count. For the synthetic, set ADR factor to 0 explicitly (no ADRs are touched) and document that choice in §5.1.

**Attribution integrity reminder:** the only valid `answered_by` labels for a Junior planning subagent are `"planner"` (forward-looking pre-resolved entries you have a recommendation on) or `null` (genuinely needs advisor input). Never `"advisor"`, never `"user"`, never `"impl-self-resolved"`.

---

**Lean / advisor-side tip (not a constraint):** the synthetic plan is a smoke test of the V1 spec pipeline. If you find yourself fighting the templates or contradictions surface between the FILES YAML discipline and the §16a structural-pattern discipline that the test-shapes don't already cover, that's a real signal — file a clarify DQ. The dogfood's value is finding latent ambiguities before any real-phase plan hits them. A synthetic plan that ships frictionlessly is good news; a synthetic plan that surfaces 2-3 clarify DQs is even better news (the dogfood gate did its job). Either outcome lands in the dogfood report at the advisor side.
