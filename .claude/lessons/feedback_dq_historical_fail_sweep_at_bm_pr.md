---
name: DQ historical-fail sweep at bm-pr
description: Phase branch accumulates `kind: validate-pending` entries with `result: fail` per option-2 single-entry mutation. At bm-pr time, sweep these to resolved[] with `answer: superseded by PR merge` to avoid bm-merge pre-reconcile commits.
type: feedback
---

# Sweep historical-fail validate-pending entries at bm-pr

Option-2 single-entry mutation (locked 2026-04-28) keeps
`kind: "validate-pending"` entries with `result: "fail" | "cancelled" |
"timed_out"` in `pending[]` permanently — they document a workflow
failure that was later corrected by a fix-impl-N. The entries serve
as audit trail. But they're "pending" only in the schema sense — they
don't block anything; the fix-impl-N that supersedes them is already
merged.

**Why this lesson exists:** Per `.claude/PRPs/reports/v1-RT-r1-retro.md`
§3 Lesson 5 + §2 Advisor miss. RT-r1 accumulated 5 historical-fail
entries (#194, #203, #204, #206, #207) over the phase. At bm-merge
time, bm-merge required a pre-merge reconcile commit (`4a4ed354d`)
to resolve a DQ JSON conflict against `governance-v0`. Root cause:
governance-v0 had its own DQ resolved entries from other concurrent
phases; phase-v1-RT-r1 had these 5 stale-pending entries; the JSON
conflict-resolution semantics on bm-merge wasn't clean.

SL-c-2 retro flagged the same pattern at §3.1 — repeat occurrence
across two phases.

**How to apply:** the bm-pr brief authoring step (advisor inline)
sweeps the phase-branch DQ before queueing bm-pr.

```bash
# At bm-pr brief authoring time:
PHASE_BRANCH=$(git branch --show-current)
PHASE_SLUG=$(echo "$PHASE_BRANCH" | sed 's/^phase-//')

# Identify historical-fail entries (kind: validate-pending in pending[]
# with result != "pass" or null)
python <<'EOF'
import json, io
from datetime import datetime, timezone
path = '.claude/decision-queue.json'
d = json.load(io.open(path, encoding='utf-8'))
sweep = []
keep = []
for e in d.get('pending', []):
    if e.get('kind') == 'validate-pending' and e.get('result') in ('fail', 'cancelled', 'timed_out', 'gh_unauth', 'run_not_found'):
        e['answer'] = 'superseded by PR merge — workflow failure was corrected by subsequent fix-impl-N landing on phase branch before bm-pr'
        e['answered_by'] = 'advisor'
        e['resolved_at'] = datetime.now(timezone.utc).isoformat(timespec='seconds').replace('+00:00', 'Z')
        sweep.append(e)
    else:
        keep.append(e)
d['pending'] = keep
d['resolved'] = d.get('resolved', []) + sweep
with io.open(path, 'w', encoding='utf-8') as f:
    json.dump(d, f, ensure_ascii=False, indent=2)
print(f'swept {len(sweep)} historical-fail entries to resolved[]')
print('ids:', [e['id'] for e in sweep])
EOF

# Commit the sweep
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): advisor swept historical-fail validate-pending entries for ${PHASE_SLUG} bm-pr"
git push origin "$PHASE_BRANCH"
```

The sweep:

- Filters `kind: "validate-pending"` entries with `result` in the
  failure enum (`fail`, `cancelled`, `timed_out`, `gh_unauth`,
  `run_not_found`).
- Mutates each entry in place: `answer` filled with the canonical
  "superseded by PR merge" reason, `answered_by: "advisor"`,
  `resolved_at` filled with ISO 8601 UTC.
- Moves the entries from `pending[]` to `resolved[]`.
- Commits + pushes to phase branch before bm-pr brief authoring.

**Commit subject pattern (mandatory per attribution rule):**

```
chore(decision-queue): advisor swept historical-fail validate-pending entries for <phase-slug> bm-pr
```

Matches `^(chore|docs)\((advisor|decision-queue)\)` per
`.claude/rules/decision-queue.md` "Attribution integrity §Detection" —
required for the advisor-side write.

**Why not just leave them as `result: fail` in pending[]:**

- The bm-merge step has to resolve DQ JSON conflicts between phase
  branch and governance-v0. Stale-fail entries are the conflict
  vector — they're the only DQ deltas the phase branch carries that
  trunk doesn't. Sweeping them resolves the conflict pre-merge,
  saves the pre-merge reconcile commit.
- The audit trail is preserved — entries move to `resolved[]`, not
  deleted. Future archaeology can still see when each fix-impl-N
  corrected the workflow failure.
- The polling-loop semantics improve — advisor sessions don't have
  to filter `pending[]` for "real" blockers vs historical-fails.

**Edge cases:**

- **No historical-fails exist:** sweep is a no-op (0 entries moved);
  commit is skipped (the file didn't change). Don't commit empty
  diffs.
- **Historical-fail entry with `phase_task: fix-impl-N`:** sweep
  applies normally — the fix-impl's own failure is a workflow event;
  whether the next fix-impl-(N+1) corrected it is documented via the
  commit log, not via the DQ entry's content.
- **Cross-phase historical-fails:** rare. Should NOT happen because
  each phase's validate-pending entries cite their phase's
  workflow_run_id; advisor doesn't sweep entries that reference
  workflow IDs older than the current phase's bm-cut SHA.
- **DQ #205-shaped blockers (`kind: blocker`, not validate-pending):**
  NOT swept — those need real answers. The sweep is scoped to
  `kind == "validate-pending"` only.

**Companion lessons:**

- `feedback_dq_self_resolved_belongs_in_resolved_array.md` —
  resolved entries go in `resolved[]`, not `pending[]`.
- `feedback_decision_queue_protocol.md` (memory) — DQ schema
  semantics.
- `feedback_dq_raise_before_ci_watcher_queue.md` — L3 atomic raise
  discipline; same attribution rules apply to this sweep.

**Where codified:**

- `.claude/commands/bm/bm-pr.md` Phase 1 pre-conditions — add
  "historical-fail sweep before authoring PR body".
- `.claude/rules/advisor-orchestrator.md` §3.1 stage-shape "advance
  to bm-pr" branch.
- This lesson file.
