# Escalation note — v1-rls-r1 planning (job-367)

**Author:** planning Junior task `junior/role-planning-v1-rls-r1-plan-see-claude-prps-briefs-v1-rls-r1-planning-1-md-367`
**Authored:** 2026-05-21
**Brief:** `.claude/PRPs/briefs/v1-rls-r1-planning-1.md` @ phase-v1-rls-r1 HEAD `91d0eda6f`
**Plan deliverable:** `PLAN_DELIVERABLE.md` (this commit, repo root)

## 1. What was attempted

Authored the v1-rls-r1 plan per the brief at
`.claude/PRPs/briefs/v1-rls-r1-planning-1.md`. Followed the
`.claude/PRPs/templates/plan.template.md` 20-section schema literally.
Honoured every PRECON-1..PRECON-10 binding in the brief §0.1 + the
five `kind: "clarify"` DQ resolutions (DQ #296-#302). Used the
`v1-federation-inbound-a.plan.md` §15 shape as the laptop-DoD mirror
(per PRECON-7); used `brehon-conformance-audit.plan.md` as the §13
task-shape mirror.

## 2. What failed (harness-side, NOT planner-side)

Two Junior-worker write blockers — both are Claude Code's built-in
sensitive-file protection blocking `.claude/**` writes regardless of
`settings.json` `permissions.allow`. Same pattern observed in
`v1-federation-inbound-a.plan.md`'s leading delivery note (2026-05-16),
now recurring in v1-rls-r1.

### 2a. Plan canonical path blocked

The canonical target path `.claude/PRPs/plans/v1-rls-r1.plan.md` is
blocked for Junior worker writes (the Write tool prompts the harness).
**Workaround applied:** the plan body was written to
`PLAN_DELIVERABLE.md` at the repo root with a leading **DELIVERY NOTE**
instructing the advisor to:

```bash
git mv PLAN_DELIVERABLE.md .claude/PRPs/plans/v1-rls-r1.plan.md
```

The plan body is unchanged — only the path changes on `mv`.

### 2b. Decision-queue write blocked

`.claude/decision-queue.json` is also blocked for Junior worker writes
despite being whitelisted in `settings.json` `permissions.allow`. The
planner therefore could NOT write the four `kind: "log"` pre-seed
entries directly. **Workaround applied:** the four pre-seeds are
described verbatim in PLAN_DELIVERABLE.md §19.1 (next available
id = **303**; max id across live + archives = 302). The advisor
should transcribe them into `.claude/decision-queue.json` directly
when moving the plan file.

## 3. What is needed (advisor action)

After this Junior task's finalize-merge lands on `phase-v1-rls-r1`:

1. **Move the plan to its canonical path:**
   ```bash
   git mv PLAN_DELIVERABLE.md .claude/PRPs/plans/v1-rls-r1.plan.md
   ```
2. **Remove this escalation note:** `git rm ESCALATION_NOTE.md`
   (delivery + audit happened in this commit; the note is no longer
   needed once the plan is on canonical path — but **first read it**
   for the DQ pre-seed JSON).
3. **Transcribe the four DQ pre-seeds** from
   `PLAN_DELIVERABLE.md` §19.1 (now `.claude/PRPs/plans/v1-rls-r1.plan.md`
   §19.1 after step 1) into `.claude/decision-queue.json`. Use ids
   303-306; set `timestamp` + `resolved_at` to the current UTC time.
4. **Commit** with subject pattern matching the attribution rule:
   `chore(advisor): rename plan to canonical path + transcribe v1-rls-r1 planner DQ pre-seeds #303-#306`.
5. **Run the §3.4 DoD smoke + §3.5 watchpoint-specificity + §3.7
   dogfood + §3.8 schema-retrofit gates** per
   `.claude/rules/advisor-orchestrator.md`.
6. **Surface the plan to the user (User Gate 1 plan approval)** —
   per `.claude/rules/advisor-orchestrator.md` §3.2.

## 4. Suggested v1-rls-r2 candidate (DQ #306 pre-seed reasoning)

Extend the Junior worker permission allowlist to cover
`.claude/PRPs/plans/**` + `.claude/decision-queue.json`, OR add a
harness-side configuration option that lets Junior tasks write to
specifically-named paths in `.claude/**` when explicitly authorised
by the dispatch line. The recurrence of this gap across two planning
tasks (v1-fed-in-a, v1-rls-r1) is itself the evidence.

## 5. Plan summary (advisor read-aid)

- **Phase:** `v1-rls-r1` (first wave of RLS hardening).
- **Tracks:** A (doc + spec, no compile dep) / B (SessionStart hook
  + weekly-review Step 2c + lesson) / C (`retro-check.sh` JSONL
  instrumentation + kind-registry doc + lesson) / D (closeout —
  dogfood + orchestrator wiring + paired lessons + retro).
- **§13 tasks:** Task 0 pre-flight + Tasks 1-12 impl + Task 13 retro
  = 14 tasks.
- **Cohort A:** Tasks 1, 2, 3, 4, 7 `[P]` (all `requires: [0]`).
- **Serial chain:** Task 5 (after 4) → Task 6 (after 3) → Task 8
  (after 7) → Task 9 (after 7+8) → Task 10 dogfood (after 3,4,5,6,7,8,9)
  → Task 11 orchestrator wiring (after 3,7,8,9) → Task 12 paired
  lessons `[P]` internally (after 3,7,10) → Task 13 retro (terminal).
- **§5 complexity score:** **6/10** — below the Sonnet `> 8` split
  threshold; no split-DQ filed.
- **§15 DoD:** `validate-pending-laptop` shape per PRECON-7 (Shape G
  suspended). §15.5 Phase 2 e2e SKIPPED (no Rust changes).
- **§16a stories:** 5 stories, each citing composing tasks + a
  shell checkpoint command + Brief-Scope structural outputs for
  `/brehon-verify`.

## 6. Files committed in this Junior task

- `PLAN_DELIVERABLE.md` — the plan body (move to
  `.claude/PRPs/plans/v1-rls-r1.plan.md`).
- `ESCALATION_NOTE.md` — this file (advisor reads, then `git rm`).

No edits under `crates/`, `migrations/`, `tests/`, `docs/brehon-law-
inspired-network/99-decisions-and-open-questions.md`,
`Cargo.toml`, `Cargo.lock`, or `rust-toolchain.toml`. Per planner
hard refusals + brief §2.2 NOT-building list.
