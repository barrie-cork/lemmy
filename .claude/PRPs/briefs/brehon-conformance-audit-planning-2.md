# Planning brief — brehon-conformance-audit re-plan (Cohort 2 circular requires fix)

[role:planning] revise plan §13 Cohort 2 to fix circular requires (Task 6 requires Tasks 4+5 in same cohort) — see DQ #297

---

## 1. Role + dispatch line

`[role:planning]` — narrow re-plan of an existing plan (`.claude/PRPs/plans/brehon-conformance-audit.plan.md`). One-line summary: split Cohort 2 into Cohort 2 + Cohort 2.5 to honour Task 6's `requires: [4, 5]` dependency, per `advisor-orchestrator.md` §4.1 step 4a.

**Base branch for fork:** `phase-brehon-conformance-audit` (NOT `governance-v0`). The existing plan lives at `.claude/PRPs/plans/brehon-conformance-audit.plan.md` on phase branch; DQ #297 lives in `.claude/decision-queue.json` on phase branch.

## 2. Scope

**Edit one file only:** `.claude/PRPs/plans/brehon-conformance-audit.plan.md`.

**Specific edits (Fix A — user-approved 2026-05-20):**

1. **§13 cohort plan block (lines ~691-697):** restructure the `> **Cohort plan** ...` blockquote. Replace existing lines with:

   ```
   > **Cohort plan (computed mechanically from `union(creates, modifies)` + `requires:`):**
   > - **Cohort 1 (serial):** Task 1 alone — SKILL.md skeleton must exist before axis sub-files reference its frontmatter.
   > - **Cohort 2 (parallel `[P]`):** Tasks 2, 3, 4, 5, 8, 10 — file-set disjoint; Tasks 2-5 depend on Task 1 (Task 8/10 are `requires: [0]` only, but bundled into Cohort 2 for wall-clock).
   > - **Cohort 2.5 (serial, depends on Cohort 2):** Task 6 — `requires: [4, 5]` (compute-metrics.sh consumes the schema in Task 4 and the formulas in Task 5; Task 6 must run after Tasks 4 + 5 finalize-merge onto phase branch).
   > - **Cohort 3 (serial, depends on Cohort 2.5):** Task 7 (dogfood) — `requires: [1, 2, 3, 4, 5, 6, 8]` (skill files must exist before dogfood runs; clippy.toml must exist so the dogfood includes the Track-B integration check).
   > - **Cohort 4 (serial, depends on Cohort 2):** Task 9 (per-module deny) — `requires: [8]`.
   > - **Cohort 5 (parallel `[P]`):** Task 11, Task 12 — disjoint files; depend on prior cohorts (Task 11 `requires: [1, 8, 9]`; Task 12 `requires: [7]`).
   > - **Cohort 6 (serial):** Task 13 retro — depends on all prior.
   ```

   Key changes vs prior: (a) Task 6 REMOVED from Cohort 2; (b) new Cohort 2.5 added between Cohort 2 + Cohort 3; (c) Cohort 3 dependency updated from "Cohort 2" → "Cohort 2.5"; (d) Cohort 4 dependency stays "Cohort 2" (Task 9 only requires Task 8); (e) preamble adds "+ `requires:`" to clarify cohort computation accounts for both.

2. **§13 Task 6 header (line ~1073):** change `### Task 6 [P]: CREATE ...` to `### Task 6: CREATE ...` (drop the `[P]` marker — Task 6 no longer parallel-eligible with the cohort it serializes after).

3. **§13 Task 6 FILES YAML (around line 1078):** `requires: [4, 5]` STAYS as-is. No change to YAML body — only the cohort placement + `[P]` marker change.

4. **Cohort 2.5 emerging step list (if §13 has a "cohort emerging" or numbered cohort list elsewhere):** add Cohort 2.5 wherever cohorts are enumerated. Grep `Cohort 2\b` and `Cohort 3` for all references.

5. **§5 complexity score table (if it references cohort count):** if §5 has "6 cohorts" or similar, bump to "7 cohorts (was 6 + 0.5)". Do NOT change the complexity score itself (10/10 stays; user gate already approved proceed-as-one).

**Out of scope — do NOT touch:**

- Task 1 (already shipped at `1e3948e43`; SKILL.md is live).
- Any §10 template content.
- §15 DoD commands.
- §16a stories (cohort restructure doesn't change story shape).
- §18 risks / §19 notes.
- Any other §13 task bodies (Tasks 2, 3, 4, 5, 7, 8, 9, 10, 11, 12, 13 are unchanged in content; only Task 6's header + cohort-list mention change).
- ANY file outside `.claude/PRPs/plans/brehon-conformance-audit.plan.md`.

**Commit shape:** one commit. Subject: `chore(plan): revise brehon-conformance-audit Cohort 2 — split Task 6 into Cohort 2.5 (fix circular requires per DQ #297)`. Body cites DQ #297 + the §4.1 step 4a invariant + `feedback_cohort_validation_dependency_check.md`.

## 3. Required reading (planner MUST Read these first)

1. **`.claude/PRPs/plans/brehon-conformance-audit.plan.md`** — the plan being revised. Pay attention to:
   - §13 cohort plan blockquote (lines ~691-697) — the line being edited.
   - Task 6 header + FILES YAML (lines ~1073-1090) — to confirm `requires: [4, 5]` is intact.
   - §10.7 compute-metrics.sh template — to confirm Task 6 truly depends on Task 4 (schema) + Task 5 (formulas), justifying the split.
   - §16a stories (if §16a maps stories to cohorts, ensure the story for Cohort 2 stays valid OR add a Cohort 2.5 story).

2. **`.claude/decision-queue.json`** — read DQ #297 (`pending[]`) for the full advisor catch-fire context.

3. **`.claude/rules/advisor-orchestrator.md`** §4.1 step 4a (lines containing "circular requires") — the rule that triggered the refusal.

4. **`.claude/lessons/feedback_cohort_validation_dependency_check.md`** — the lesson + retro precedent (v1-RT-r1 commit `ffa2876e3`). The "Edge cases" section explicitly enumerates the structural fixes available.

5. **`.claude/rules/advisor-orchestrator.md`** §2 "Brief authoring" — to understand cohort-membership semantics + `[P]` invariants.

## 4. Constraints

- **NEVER** edit any file other than `.claude/PRPs/plans/brehon-conformance-audit.plan.md` (Task 1 is already shipped; this is a §13-only restructure).
- **NEVER** change the complexity score in §5 (user already gated; locked at 10 proceed-as-one).
- **NEVER** drop or move any task other than Task 6's cohort placement.
- **NEVER** rewrite Task 6's content body — only its header line drops the `[P]` marker; FILES YAML stays.
- **MUST** preserve all existing `Cohort 1`, `Cohort 3`, `Cohort 4`, `Cohort 5`, `Cohort 6` semantics — the only structural change is adding `Cohort 2.5` between Cohort 2 and Cohort 3.
- **MUST** update Cohort 3's dependency text from "Cohort 2" → "Cohort 2.5" (since Task 7 dogfood now depends on Task 6 having shipped, which is in Cohort 2.5).
- **MUST** preserve mid-task DQ push discipline: if the planner writes a `kind: "log"` DQ entry recording the cohort-restructure insight, that entry goes to `resolved[]` with `answered_by: "planner-self-resolved"`, NOT to `pending[]`.
- **MUST** include `LESSON:` trailer in the commit body referring back to `feedback_cohort_validation_dependency_check.md` so retro can harvest the trace.

## 5. Validation

After Edit, before commit, the planner MUST:

1. **Render check:** `Read` lines 685-720 of the edited plan; confirm the Cohort 2.5 blockquote line is present + Cohort 3 dependency text updated.
2. **Task 6 header check:** `grep -n "^### Task 6" .claude/PRPs/plans/brehon-conformance-audit.plan.md` → exactly ONE line, NOT containing `[P]`.
3. **YAML preservation check:** `grep -A 12 "^### Task 6" .claude/PRPs/plans/brehon-conformance-audit.plan.md | grep -A 4 "requires:" | head -5` → must show `task: 4` AND `task: 5` (preserved verbatim from before).
4. **Cohort 2 listing check:** `grep -E "Cohort 2 .parallel|Cohort 2 \(parallel" .claude/PRPs/plans/brehon-conformance-audit.plan.md` → the line must NOT contain `6` in the task list (must be `Tasks 2, 3, 4, 5, 8, 10`).
5. **No-other-file check:** `git diff --name-only` MUST return EXACTLY `.claude/PRPs/plans/brehon-conformance-audit.plan.md` and nothing else.

If any check fails, fix and re-run; do NOT commit a partial fix. If a check is impossible to satisfy (e.g. §16a structure prevents clean addition of Cohort 2.5 story), STOP and file a `kind: "blocker"` DQ to advisor with the structural obstacle. Do not improvise.

## 6. Done criteria

- Single commit on planner's worker branch.
- Subject matches the pattern in §2 above.
- Body cites DQ #297 + §4.1 step 4a + `feedback_cohort_validation_dependency_check.md`.
- The 5 validation checks in §5 all pass.
- Worker pushes the branch + daemon finalize-merge brings the commit onto `phase-brehon-conformance-audit`.
- The post-commit DQ entry MUST mutate DQ #297 in place (move from `pending[]` to `resolved[]` with `answered_by: "planner"`, `answer: "Cohort 2 split into Cohort 2 + Cohort 2.5 per plan commit <sha>. Task 6 dropped [P] marker. See plan §13 lines 691-697 + Task 6 header"`). The planner is allowed to mutate the existing entry — this is the canonical resolution path for advisor-raised blockers that the planner answers.

## 7. Anti-patterns to refuse

- Re-running `/brehon-clarify` (this brief IS the narrow mechanical fix; no clarify needed).
- Authoring NEW lessons or rules (this is a §13 plan edit only).
- Re-numbering existing tasks (Task 6 stays Task 6; do not renumber to "Task 5.5").
- Touching `.claude/skills/brehon-conformance-audit/SKILL.md` (already shipped; out of scope).
- Touching the §5 complexity-score table (user already approved proceed-as-one at score 10).
- Adding new cohorts beyond Cohort 2.5 (the fix is narrow; do not introduce a Cohort 7 etc).
