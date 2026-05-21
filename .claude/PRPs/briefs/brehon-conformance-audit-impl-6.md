# brehon-conformance-audit — impl Task 6 brief (Cohort 2.5)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit Task 6 — compute-metrics.sh — see .claude/PRPs/briefs/brehon-conformance-audit-impl-6.md`

## 2. Scope

Implement plan §13 **Task 6** of `.claude/PRPs/plans/brehon-conformance-audit.plan.md` (lines 1117-1180). ONE file created: `.claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh`. Bash script with a Python3 here-doc wrapper that reads audit-metrics JSON files, validates them against `audit-metrics.schema.json` (Task 4), and computes per-axis precision/recall + lead-time + latent-footgun catch rate. No cargo invocation (per Watchpoint #3).

```yaml
creates:
  - .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh
modifies: []
requires:
  - task: 4
    reason: "compute-metrics.sh consumes the JSON schema defined in audit-metrics.schema.json (Task 4 — already shipped, file present at .claude/skills/brehon-conformance-audit/audit-metrics.schema.json)."
  - task: 5
    reason: "compute-metrics.sh implements the formulas defined in METRICS.md (Task 5 — already shipped, file present)."
```

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at `6c7e80e29` — Phase 1 mechanism complete; clippy.toml + Cargo.toml workspace-allow + 3 federation mod.rs deny attributes all in place).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges.

**Do NOT** author:
- Any change to other skill files (`audit-metrics.schema.json`, `METRICS.md`, `SKILL.md`, axis sub-files).
- Any change to other scripts (`find-sibling.sh` already shipped at `6388ad42f`).
- Any cargo invocation in the script (Watchpoint #3 — the script is for metrics analysis, not validation).
- Any new lesson file or rule file.

## 3. Required reading

### 3.0 Plan + sibling files (P0)

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §13 Task 6 (lines 1117-1180) — full task text.
2. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.7 — compute-metrics.sh template (the verbatim source).
3. `.claude/skills/brehon-conformance-audit/audit-metrics.schema.json` — JSON schema to validate against. Read it to understand the required fields: `schema_version`, `scope`, `head_sha`, `run_at`, `skill_version`, `predictions[]`, `ground_truth_compile_caught[]`, `ground_truth_runtime[]`. Each prediction has `{axis, risk_tier, target, sibling, evidence}`. Each ground-truth entry has `{axis, target, source, evidence_commit_sha}`.
4. `.claude/skills/brehon-conformance-audit/METRICS.md` — formula definitions (precision, recall, lead-time, latent-footgun catch rate). Read to understand the formulas the script implements.
5. `.claude/skills/brehon-conformance-audit/scripts/find-sibling.sh` — existing companion script (shipped at `6388ad42f`). Read to understand the bash + (none) pattern used in the skill. compute-metrics.sh uses bash + python3 (different pattern; document in leading comment).

### 3.1 Cross-platform conventions

6. `scripts/brehon/cargo-check.sh` — existing bash script (not python). Read its shebang + set flags + portable command patterns.
7. `scripts/brehon/cargo-check.bat` — Windows companion. Task 6 does NOT need a `.bat` companion because Python3 is cross-platform on the user's setup (laptop has Python 3.14; daemon has python3). The bash script invokes `python3` directly.

### 3.2 Rules (auto-loaded)

8. `.claude/rules/decision-queue.md` — `kind: "validate-pending-laptop"` shape for §5.3 DQ raise.
9. `.claude/rules/phase-branch.md` — worker branch push discipline.

### 3.3 §G4 CANONICAL RECIPE (no allowlist row applies)

This is a structural Task 6 implementation, NOT a fix-impl. No §G4 allowlist row applies. The §10.7 template in the plan is the contract.

## 4. Constraints

1. **Mid-task push discipline** (per `.claude/rules/decision-queue.md` "Mid-task visibility") — any DQ pushed to worker branch immediately after raise.
2. **Attribution integrity** — `from: "impl"`; never write `answered_by: "advisor"`/`"user"`.
3. **Single-file diff** — `git status --short` after the change must show EXACTLY:
   - `?? .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh`
   (then `A` after `git add`)
   Any other file in the diff is out of scope.
4. **Executable bit** — set the file as executable (chmod +x; git tracks the executable bit on commit).
5. **No cargo invocation** — per Watchpoint #3, the script analyses metrics; it never runs cargo. If it needs SHA author-date for lead-time, use `git log -1 --format=%aI <sha>` (no cargo).
6. **No `--no-verify`** — never skip hooks (per `.claude/rules/no-destructive-defaults.md`).
7. **Schema validation graceful fallback** (per §10.7 GOTCHA) — try-import `jsonschema`; if absent, fall back to manual key-presence + axis-enum + evidence-string-length checks. Document fallback path in leading comment.
8. **Pre-push smoke test** — worker runs the fixture-based smoke test from plan §13 Task 6 VALIDATE block (lines 1153-1180) LOCALLY before push. Non-zero exit → file `kind: "blocker"` DQ; do not push.
9. **Commit subject** — `feat(skill): add compute-metrics.sh script for brehon-conformance-audit (task 6)`.
10. **Commit body** — cite plan §10.7 template + §13 Task 6 requirements + Watchpoint #3 (no-cargo).

## 5. Validation gate (DoD per plan §15)

### 5.1 Pre-push smoke test (worker runs locally before push)

```bash
# Per plan §13 Task 6 VALIDATE block — fixture-based smoke test
python3 -c "
import json
fixture = {
  'schema_version': 1,
  'scope': 'phase-diff phase-v1-federation-inbound-b',
  'head_sha': '4a60667c9',
  'run_at': '2026-05-20T12:00:00Z',
  'skill_version': '1.0.0',
  'predictions': [
    { 'axis': '4', 'risk_tier': '1', 'target': 'crates/apub/activities/src/governance/inbox.rs:735',
      'sibling': 'crates/apub/activities/src/governance/inbox.rs:105',
      'evidence': 'axis-4: inbox.rs:735 new=.unwrap_or_default() sibling=inbox.rs:105 .ok_or_else()' }
  ],
  'ground_truth_compile_caught': [],
  'ground_truth_runtime': [
    { 'axis': '4', 'target': 'crates/apub/activities/src/governance/inbox.rs:735',
      'source': 'fix-impl-3', 'evidence_commit_sha': '8b04e69a6' }
  ]
}
json.dump(fixture, open('/tmp/fixture-metrics.json', 'w'), ensure_ascii=False, indent=2)
print('fixture written')
"

bash .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh /tmp/fixture-metrics.json > /tmp/compute-metrics-smoke.log 2>&1
echo "exit: $?"
cat /tmp/compute-metrics-smoke.log
# EXPECT: exit 0; stdout contains "axis-4 precision: 1.000" and "axis-4 recall: 1.000"
```

### 5.2 Structural check

```bash
test -x .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh
echo "executable: $?"
# EXPECT: exit 0

head -2 .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh
# EXPECT: shebang line + "set -euo pipefail"

grep -c "^#!/usr/bin/env bash" .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh
# EXPECT: 1

grep -c "set -euo pipefail" .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh
# EXPECT: 1

grep -c "python3" .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh
# EXPECT: ≥1 (python3 here-doc)

# No cargo invocations
! grep -q "cargo" .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh
echo "no cargo: $?"
# EXPECT: exit 0

git status --short
# EXPECT: exactly 1 line "A .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh"
```

### 5.3 §15 validate-pending-laptop DQ (raise after worker push)

After worker pushes the worker branch, raise a `kind: "validate-pending-laptop"` DQ entry naming the pre-push smoke test from §5.1 verbatim.

```json
{
  "id": <next-id>,
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<UTC ISO 8601>",
  "branch": "<worker-branch-name>",
  "phase_task": 6,
  "commands": [
    "bash .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh /tmp/fixture-metrics.json"
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answered_by": null,
  "resolved_at": null
}
```

Note: the DQ command names just the script invocation; the fixture-writing Python step is implicit (advisor runs the full smoke test sequence with fixture-write + script-invocation).

After raising, commit + push to worker branch:

```
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): impl raised DQ #<id> — task 6 validate-pending-laptop"
git push origin <worker-branch>
```

## 6. Script structure recipe

Per plan §10.7 (read inline); the worker's implementation follows this skeleton:

```bash
#!/usr/bin/env bash
set -euo pipefail
# compute-metrics.sh — reads audit-metrics JSON files, computes per-axis
# precision/recall/lead-time/latent-footgun catch rate.
# Pattern: bash wrapper + python3 here-doc for cross-platform portability.
# Schema validation tries `jsonschema` package; falls back to manual checks.
# NO cargo invocation (Watchpoint #3). Lead-time uses `git log -1 --format=%aI`.

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <audit-metrics.json> [audit-metrics.json ...]" >&2
  exit 2
fi

python3 - "$@" <<'PYEOF'
import json, sys, subprocess
from pathlib import Path

# Load schema
schema_path = Path(__file__).parent.parent / 'audit-metrics.schema.json' if '__file__' in dir() else None
# In a here-doc, __file__ is undefined. Resolve via the script invocation:
script_path = Path(sys.argv[0]) if sys.argv else None

# ... full implementation per §10.7 ...
PYEOF
```

Note: `__file__` in a python3 here-doc is undefined. Use a fixed relative path from the bash script's `$(dirname "$0")` resolved + passed via environment variable, OR hardcode the schema-file relative path.

Recommended pattern:

```bash
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SCHEMA_FILE="$SCRIPT_DIR/../audit-metrics.schema.json"
export SCHEMA_FILE
python3 - "$@" <<'PYEOF'
import os, json, sys, subprocess
schema_path = os.environ['SCHEMA_FILE']
# ... load + validate + compute ...
PYEOF
```

This handles the `__file__`-in-heredoc gotcha cleanly.

## 7. Failure-class mapping

| Symptom | Action |
|---|---|
| Pre-push fixture smoke test exits non-zero | `kind: "blocker"` DQ — implementation bug; do not push. Attach output. |
| `python3 -c 'import jsonschema'` exits non-zero on the daemon | Expected per §10.7 GOTCHA. Script must fall back gracefully — manual key-presence checks. Worker tests this fallback by uninstalling/skipping jsonschema temporarily if available; OR the worker uses `python3 -c 'import jsonschema' || echo skipping` pattern in the smoke test setup. Implementation MUST handle the absence gracefully. |
| Smoke test stdout missing "axis-4 precision: 1.000" or "axis-4 recall: 1.000" | `kind: "blocker"` DQ — implementation bug; output format mismatch. Do not push. |
| Any non-compute-metrics.sh file in `git status --short` | `kind: "blocker"` DQ — accidental contamination. |
| Daemon finalize-merge conflict on `.claude/decision-queue.json` | Standard pattern per `feedback_junior_finalize_merge_race_lossless_reconcile.md`. |

## 8. Out of scope

- Editing `audit-metrics.schema.json` (Task 4 output; already shipped).
- Editing `METRICS.md` (Task 5 output; already shipped; Task 7 backfills the `## Worked example` section).
- Editing `find-sibling.sh` (Task 3 output; already shipped).
- Editing `SKILL.md` (Task 1 output; already shipped).
- Editing any axis sub-file under `.claude/skills/brehon-conformance-audit/axes/`.
- Adding a `compute-metrics.bat` Windows companion — Python3 is cross-platform; bash script works in Git Bash on Windows.
- Adding a new lesson file (defer to Task 12 lesson-promotion phase).
- Cargo invocations (Watchpoint #3 hard refusal).
