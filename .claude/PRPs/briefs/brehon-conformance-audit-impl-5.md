# brehon-conformance-audit — impl Task 5 brief (Cohort 2)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit Task 5 — METRICS.md — see .claude/PRPs/briefs/brehon-conformance-audit-impl-5.md`

## 2. Scope

Implement plan §13 **Task 5** (lines 1032-1072) of `.claude/PRPs/plans/brehon-conformance-audit.plan.md` (revised at `7bb6c51bb`; phase tip `e3b62793f`). One commit, one file created:

```yaml
creates:
  - .claude/skills/brehon-conformance-audit/METRICS.md
modifies: []
requires:
  - task: 1
    reason: "METRICS.md is REFERENCED by SKILL.md Outputs section + per-axis sub-files (Task 2)."
```

Markdown doc per §10.6 verbatim: ground-truth attribution rules + metrics formulas + calibration cadences. Section structure: §1 title, §2 per-run journal shape (points at audit-metrics.schema.json Task 4), §3 four ground-truth attribution rules, §4 four metrics formulas, §5 three calibration cadences, §6 worked example (PLACEHOLDER — Task 7 dogfood backfills numbers), §7 adding a new metric.

**Do NOT** author:
- SKILL.md skeleton (shipped Task 1).
- Axis sub-files (Task 2 peer).
- find-sibling.sh (Task 3 peer).
- audit-metrics.schema.json (Task 4 peer).
- compute-metrics.sh (Task 6, Cohort 2.5).
- Any other plan §13 file.

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at `e3b62793f`).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges into phase branch.

## 3. Required reading

### 3.0 Plan + Task 1 spine (P0)

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.6 (METRICS.md — verbatim), §13 Task 5 (lines 1032-1072).
2. `.claude/skills/brehon-conformance-audit/SKILL.md` (Task 1 output) — Outputs section forward-references METRICS.md.

### 3.1 MIRROR

3. `.claude/skills/post-task-retro/SKILL.md` — "Score calibration" section for the explainer tone (plan §13 Task 5 MIRROR).

### 3.2 PMD source

4. PMD `project_phase6_convention_divergence_class.md` — origin of the six-axis schema; the calibration cadence rationale draws from here.
5. `feedback_clippy_test_style.md` + `feedback_lemmy_error_no_std_error.md` — Track A vs Track B distinction; METRICS.md must distinguish the two enforcement tracks.

### 3.3 Rules (auto-loaded)

6. `.claude/rules/decision-queue.md` — mid-task visibility.
7. `.claude/rules/phase-branch.md` — worker branch push.

## 4. Constraints

1. **Mid-task push discipline** — any DQ pushed immediately to worker branch.
2. **Attribution integrity** — `from: "impl"`; never `answered_by: "advisor"`/`"user"`.
3. **Seven sections, in order** — §1..§7 per plan §13 Task 5 IMPLEMENT verbatim.
4. **§6 Worked example is a PLACEHOLDER** — Task 7 (dogfood) backfills real numbers via Edit. The Task 5 author writes the structure + a placeholder note ("populated by Task 7 dogfood backfill"). Per plan §13 Task 5 GOTCHA explicit.
5. **§3 ground-truth attribution rules verbatim from §10.6** — four rules; do NOT abridge.
6. **§4 metrics formulas verbatim from §10.6** — four formulas (precision@axis, recall@axis, lead-time, latent-footgun catch rate).
7. **§5 calibration cadences verbatim from §10.6** — three cadences (post-task, per-phase retro, every-major-version).
8. **NO cargo invocation in METRICS.md body** (Watchpoint #3).
9. **Reference audit-metrics.schema.json by path** — `.claude/skills/brehon-conformance-audit/audit-metrics.schema.json`. Validates in §5.1.
10. **Commit subject template** — `feat(skill): add METRICS.md for brehon-conformance-audit (task 5)`.
11. **Single commit** per plan §13 norm.
12. **No `--no-verify`** — never skip hooks.

## 5. Validation gate (DoD per plan §15)

### 5.1 Pre-push structural check (worker runs locally before push)

```bash
# Section count
grep -c "^## " .claude/skills/brehon-conformance-audit/METRICS.md
# EXPECT: >= 6 (excludes §1 H1)

# Mandatory references
grep -l "audit-metrics.schema.json" .claude/skills/brehon-conformance-audit/METRICS.md
grep -l "precision per axis\|recall per axis\|lead time\|latent-footgun" .claude/skills/brehon-conformance-audit/METRICS.md
# EXPECT: both grep -l return the METRICS.md path
```

### 5.2 Cargo-check (workspace clean — sanity)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task5-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/brehon-conformance-audit-task5-check.log
# EXPECT: exit 0 (no Rust changes)
```

### 5.3 §15 validate-pending-laptop DQ (raise after worker push)

```json
{
  "id": "<next>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO>",
  "question": "validate phase-brehon-conformance-audit task 5 worker branch",
  "branch": "<worker-branch>",
  "phase_task": 5,
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task5-check.log 2>&1; echo exit: $?"
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

DQ entry committed + pushed BEFORE worker exits.

## 6. KNOWN harness limitations

1. **CC v2.1.119 sensitive-file gate** — `.claude/skills/**` allowed; if Write blocked, `/tmp + mv` per plan §19.5 LESSON.
2. **Daemon finalize-merge no-push** — advisor handles SSH-push.

## 7. Next steps after this task

- Daemon finalize-merge + advisor polling + validate-pending-laptop mutation.
- Task 6 (Cohort 2.5) consumes METRICS.md formulas after Tasks 4 + 5 finalize-merge.
- Task 7 (Cohort 3 dogfood) backfills §6 Worked example numbers via Edit.

## 8. Commit subject template (verbatim)

```
feat(skill): add METRICS.md for brehon-conformance-audit (task 5)
```

Body cites plan path + SHA + seven sections per §10.6.

## 9. DoD for this brief (advisor-side gate — completed at brief commit)

- [x] Brief committed on `phase-brehon-conformance-audit` BEFORE Junior task dispatch.
- [x] Cites plan §13 Task 5 line range (1032-1072).
- [x] §4 enumerates 12 constraints.
- [x] §5 names validate-pending-laptop DQ shape verbatim.
