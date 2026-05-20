# brehon-conformance-audit — impl Task 4 brief (Cohort 2)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit Task 4 — audit-metrics.schema.json — see .claude/PRPs/briefs/brehon-conformance-audit-impl-4.md`

## 2. Scope

Implement plan §13 **Task 4** (lines 993-1030) of `.claude/PRPs/plans/brehon-conformance-audit.plan.md` (revised at `7bb6c51bb`; phase tip `e3b62793f`). One commit, one file created:

```yaml
creates:
  - .claude/skills/brehon-conformance-audit/audit-metrics.schema.json
modifies: []
requires:
  - task: 1
    reason: "Schema file is REFERENCED by SKILL.md Outputs section."
```

JSON Schema 2020-12 draft per §10.5 verbatim. Fields: `schema_version` (const 1), `scope`, `head_sha` (7-40 hex), `run_at` (date-time), `skill_version` (semver), `predictions[]` (each: `axis` enum 1-6, `risk_tier` enum 1-3, `target` "file:line", `sibling`, `evidence` maxLength 120), `ground_truth_compile_caught[]`, `ground_truth_runtime[]`.

**Do NOT** author:
- SKILL.md skeleton (shipped Task 1).
- Axis sub-files (Task 2 cohort 2 peer).
- find-sibling.sh (Task 3 cohort 2 peer).
- METRICS.md (Task 5 cohort 2 peer).
- `compute-metrics.sh` (Task 6, Cohort 2.5).
- Any other plan §13 file.

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at `e3b62793f`).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges into phase branch.

## 3. Required reading

### 3.0 Plan + Task 1 spine (P0)

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.5 (audit-metrics.schema.json — verbatim), §13 Task 4 (lines 993-1030).
2. `.claude/skills/brehon-conformance-audit/SKILL.md` (Task 1 output) — confirm Outputs section forward-references this file.

### 3.1 JSON Schema 2020-12 reference

3. https://json-schema.org/draft/2020-12/schema — top-level `$schema` value verbatim.

### 3.2 PRECON-6 (full metrics scheme)

4. `.claude/PRPs/briefs/brehon-conformance-audit-planning-1.md` §0.1 PRECON-6 — `evidence` maxLength 120; the v1 cap. The fallback 200-char cap is a v2 concern; do NOT include it.

### 3.3 Rules (auto-loaded)

5. `.claude/rules/decision-queue.md` — mid-task visibility; attribution.
6. `.claude/rules/phase-branch.md` — worker branch push.

## 4. Constraints

1. **Mid-task push discipline** — any DQ pushed immediately to worker branch.
2. **Attribution integrity** — `from: "impl"`; never `answered_by: "advisor"`/`"user"`.
3. **`$schema` value EXACTLY** `https://json-schema.org/draft/2020-12/schema` — `python3 -c "import json; ..."` validates this in §5.1.
4. **`schema_version` field is a `const: 1`** in v1 — NOT a free integer. Per §10.5 verbatim.
5. **`axis` field values are STRINGS** `"1".."6"` per plan §10.5 enum (validates as set membership in §5.1). Worker MUST NOT change to integers.
6. **`evidence` maxLength 120** — per PRECON-6 + plan §13 Task 4 GOTCHA.
7. **`head_sha` pattern 7-40 hex chars** — per plan §13 Task 4 GOTCHA.
8. **`risk_tier` enum 1-3** — Tier 1/2/3 per plan §10.5.
9. **NO cargo invocation** in this task (Watchpoint #3).
10. **Commit subject template** — `feat(skill): add audit-metrics.schema.json for brehon-conformance-audit (task 4)`.
11. **Single commit** per plan §13 norm.
12. **No `--no-verify`** — never skip hooks.

## 5. Validation gate (DoD per plan §15)

### 5.1 Pre-push schema validation (worker runs locally before push)

```bash
python3 -c "
import json
schema = json.load(open('.claude/skills/brehon-conformance-audit/audit-metrics.schema.json'))
assert schema.get('\$schema') == 'https://json-schema.org/draft/2020-12/schema'
assert schema['properties']['schema_version']['const'] == 1
assert set(schema['properties']['predictions']['items']['properties']['axis']['enum']) == set(['1','2','3','4','5','6'])
assert schema['properties']['predictions']['items']['properties']['evidence']['maxLength'] == 120
print('OK')
"
# EXPECT: OK
```

### 5.2 Cargo-check (workspace clean — sanity)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task4-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/brehon-conformance-audit-task4-check.log
# EXPECT: exit 0 (no Rust changes; sanity)
```

### 5.3 §15 validate-pending-laptop DQ (raise after worker push)

```json
{
  "id": "<next>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO>",
  "question": "validate phase-brehon-conformance-audit task 4 worker branch",
  "branch": "<worker-branch>",
  "phase_task": 4,
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task4-check.log 2>&1; echo exit: $?"
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

1. **CC v2.1.119 sensitive-file gate** — `.claude/skills/**` allowed; if Write blocked, fall back to `/tmp + mv` per plan §19.5 LESSON.
2. **Daemon finalize-merge no-push** — advisor handles SSH-push.

## 7. Next steps after this task

- Daemon finalize-merges + advisor polling + validate-pending-laptop mutation as for Task 2.
- Task 6 (Cohort 2.5) consumes this schema after Tasks 4 + 5 finalize-merge.

## 8. Commit subject template (verbatim)

```
feat(skill): add audit-metrics.schema.json for brehon-conformance-audit (task 4)
```

Body cites plan path + SHA + JSON Schema 2020-12 draft.

## 9. DoD for this brief (advisor-side gate — completed at brief commit)

- [x] Brief committed on `phase-brehon-conformance-audit` BEFORE Junior task dispatch.
- [x] Cites plan §13 Task 4 line range (993-1030).
- [x] §4 enumerates 12 constraints.
- [x] §5 names validate-pending-laptop DQ shape verbatim.
