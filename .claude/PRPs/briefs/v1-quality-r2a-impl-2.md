# v1-quality-r2a impl-task 2 — C3 deferral DQ entry (Issue #158)

## 1. Dispatch

```
[role:impl-task] v1-quality-r2a task 2 — C3 deferral DQ — see .claude/PRPs/briefs/v1-quality-r2a-impl-2.md
```

Branch the worker forks from: `phase-v1-quality-r2`.

## 2. Scope

Implement Plan §13 Task 2 verbatim: append exactly **one** `kind: "log"`,
`from: "planner"`, `answered_by: "planner"` entry to `resolved[]` in
`.claude/decision-queue.json` recording the C3 deferral decision (Issue
#158 — `emit_reputation_event` helper extraction deferred per WP-2
premature-DRY gate). Single file modified.

**No `crates/**` edits. No script edits. No new files (the fragment file is
authored under `.claude/PRPs/debug/` per .gitignore — the fragment is an
ephemeral working file, not a tracked artifact).**

### 2.1 §G4 CANONICAL RECIPE

N/A — this is a planned `impl-task`, not a `fix-impl-task` recovering from a §G4 classifier failure.

### 2.2 Verbatim §10.3 fragment JSON (author at `.claude/PRPs/debug/v1-quality-r2a-c3-defer.json`)

Substitute `<ISO8601 at T2 execution time>` with the actual current UTC ISO 8601
timestamp at the moment of authoring (NOT a hardcoded value). Both `timestamp`
and `resolved_at` must be the same value (kind: "log" goes directly to
`resolved[]` with the writer as both raiser and answerer).

```json
{
  "from": "planner",
  "kind": "log",
  "timestamp": "<ISO8601 at T2 execution time>",
  "question": "C3 (Issue #158) emit_reputation_event helper extraction deferred per premature-DRY gate",
  "answer": "v1-quality-r2 brief WP-2 default: defer C3 unless planner identifies a 3rd consumer in near-term roadmap. As of 2026-05-28, federation_inbound lane status='done' in .claude/PRPs/v1-roadmap.json — no upcoming sub-phase introduces a 3rd reputation-event emitter. Therefore: 2 consumers as of plan author time; revisit when 3rd materialises. Per feedback_principles_not_rules.md.",
  "options": ["defer", "extract"],
  "context": "Two existing emitter sites: crates/api/api/src/governance/admin_emergency_remove.rs:448 (emit_reputation_event_local) and crates/api/api/src/governance/submit_jury_vote.rs:1096 (emit_reputation_event). Bodies are byte-identical. Extraction target: crates/api/api/src/governance/reputation_helpers.rs. Trigger condition for v1-quality-r3 follow-on: any sub-phase introduces a 3rd reputation-event emit path.",
  "answered_by": "planner",
  "resolved_at": "<ISO8601 same as timestamp>",
  "approved_by": null,
  "approved_at": null
}
```

### 2.3 Execution sequence (mechanical)

1. Generate current UTC ISO 8601 timestamp:
   `python3 -c "from datetime import datetime, timezone; print(datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ'))"`
   Capture as `$TS` (or similar shell variable).
2. Author the fragment file at `.claude/PRPs/debug/v1-quality-r2a-c3-defer.json`
   with the JSON above, substituting `$TS` into BOTH `timestamp` AND `resolved_at`
   slots. Use the Write tool (Bash heredoc tends to mangle JSON quoting on
   Windows).
3. Append via helper (NO `--pending` flag — goes directly to `resolved[]`):
   ```bash
   bash scripts/brehon/dq-v3-append-fragment.sh .claude/PRPs/debug/v1-quality-r2a-c3-defer.json
   ```
   The helper generates the composite v3 id (`bash scripts/brehon/dq-v3-new-entry.sh`)
   internally, injects it, appends to `.claude/decision-queue.json` `resolved[]`,
   writes the file.
4. Verify per §4.2 below.

### 2.4 Boundary conditions

- The fragment goes to `resolved[]`, NOT `pending[]`. Do NOT pass `--pending`.
- `id` is auto-injected by the helper — do NOT include `"id":` in the fragment.
- `timestamp` and `resolved_at` MUST be byte-identical strings (run-once
  freshness; `kind: "log"` entries are immediately self-resolved by definition).
- `answered_by: "planner"` (NOT `"impl"` — the planner authored the deferral
  decision; impl is just transcribing it; the historical convention for `kind:
  "log"` planner entries is `answered_by: "planner"`).
- `approved_by: null` MUST stay null (Hard refusal #8 — advisor-exclusive).

## 3. Required reading (mandatory)

### 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: 18bb926ed
    filesCreated:
      - scripts/brehon/dq-lint-durations.sh
      - scripts/brehon/precheck.sh
    filesModified:
      - .claude/decision-queue.json
    keyDecisions:
      - "Option A floor applied to all 3 entries (ids: 315, 1b8527b076d4-001, 81719cf8ca8d-001)"
      - "Lint composite-id-aware; precheck wires lint as first gate"
    notes: |
      Task 1 swept the 3 back-dated entries and shipped the lint + precheck scripts.
      Task 2 appends one NEW entry to the cleaned-state DQ.
      Re-fetch trunk between T1 commit and T2 read (worker should be on the
      junior/* branch forked from phase-v1-quality-r2 @ f25120c4f or later;
      the cleaned-state DQ is what the helper appends to).
      Post-Task-1 lint baseline: `bash scripts/brehon/dq-lint-durations.sh` exits 0.
      Task 2 MUST re-run this lint after the append to confirm no negative
      duration introduced (timestamp == resolved_at on this kind: "log" entry).
```

### 3b. Required lessons (file-class injection per advisor-orchestrator §2.4)

Mandatory reading; rules apply silently in the implementation:

- `.claude/lessons/feedback_windows_backslash_path_dq_via_write_fragment.md`
  (author the fragment via Write tool with forward-slash literal path; do NOT
  use Bash heredoc with `\b` in scripts/brehon path — backslash mangling).
- `.claude/lessons/feedback_dq_v3_append_via_helper_script.md` (this is the
  canonical recipe: fragment file + helper script; never inline Python with
  manual id calc).
- `.claude/lessons/feedback_principles_not_rules.md` (cited in the `answer`
  field — the "defer unless 3rd consumer materialises" decision is principle-
  based, not a hard rule).
- `.claude/lessons/feedback_lane_dq_resolution_append_to_trunk.md` (lane DQ writes
  must trunk-sync; the daemon finalize-merge handles this automatically — this
  is informational).

### 3c. Plan references

- Plan §13 Task 2 (read via `bash scripts/brehon/git-show-json.sh <phase-tip>
  .claude/PRPs/plans/v1-quality-r2a.plan.md`, lines 483-546) — Goal + FILES +
  ACTION + IMPLEMENT + MIRROR + GOTCHA + VALIDATE + Commit subject.
- Plan §10.3 (same file, lines 266-288) — verbatim fragment JSON shape; this
  brief §2.2 above pastes it directly.
- `.claude/rules/decision-queue.md` — Hard refusals #1, #8, #9 (cited below).

## 4. Constraints

### 4.1 File-ownership boundaries

- **CREATE:** `.claude/PRPs/debug/v1-quality-r2a-c3-defer.json` (gitignored;
  ephemeral fragment input to the helper script).
- **MODIFY:** `.claude/decision-queue.json` (via `dq-v3-append-fragment.sh`).
- **DO NOT modify** any `crates/**` file. Plan §12 says C3 itself is deferred;
  the deferral DQ is the only artifact this task creates.
- **DO NOT modify** any script under `scripts/brehon/` — Task 1 shipped them.
- **DO NOT edit** issue #158 directly — that's a BM-task post-merge step (planned
  as a deferral comment), not impl-task scope.

### 4.2 Worker-side validation gates (mandatory before push)

Run all 3 in sequence; all must exit 0. No cargo gates (this task has zero
crate-side impact):

```bash
# Gate 1: verify the entry was appended correctly.
python3 -c "
import json
d = json.load(open('.claude/decision-queue.json'))
hits = [e for e in d['resolved'] if e.get('answered_by') == 'planner' and 'C3' in e.get('question', '') and '#158' in e.get('question', '')]
assert len(hits) == 1, f'expected 1 C3 deferral entry, found {len(hits)}'
print('C3 deferral DQ entry id:', hits[0]['id'])
assert hits[0]['kind'] == 'log'
assert hits[0]['answered_by'] == 'planner'
assert hits[0].get('approved_by') is None
print('OK')
"
echo "verify exit: $?"
# EXPECT: exit 0; "C3 deferral DQ entry id: <12-hex>-<3-digit>" + "OK"

# Gate 2: re-run lint to confirm no negative duration introduced.
bash scripts/brehon/dq-lint-durations.sh
echo "lint exit: $?"
# EXPECT: exit 0 (silent — no "DQ-LINT FAIL" output)

# Gate 3: precheck wraps the lint; same result.
bash scripts/brehon/precheck.sh
echo "precheck exit: $?"
# EXPECT: exit 0; "[precheck] OK"
```

### 4.3 Mid-task push discipline

After all 3 gates pass, in a single sequence:

1. `git add .claude/decision-queue.json` (NOT the fragment — that's
   gitignored under `.claude/PRPs/debug/`).
2. `git commit -m "chore(decision-queue): planner-defer C3 emit_reputation_event helper (Issue #158, task 2)"` (full body in §5).
3. `git push origin <worker-branch>`.

NO `kind: "validate-pending-laptop"` entry needed (Task 2 has no cargo
side-effect; the §4.2 §15-equivalent gates ARE the validation).

### 4.4 Refusals

- NEVER write `answered_by: "advisor"` (Hard refusal #1 — `decision-queue.md`).
- NEVER write `answered_by: "user"` (only the persistent advisor session can
  relay user replies).
- NEVER write `approved_by: <non-null>` (Hard refusal #8 — advisor-exclusive).
- NEVER hand-compute `next_id` via `max(all_ids)+1` (Hard refusal #9; use the
  helper which calls `dq-v3-new-entry.sh`).
- NEVER manually edit `.claude/decision-queue.json` — always via the helper
  script (eliminates JSON-quoting + id-collision risk).
- NEVER modify any `crates/**` file (Plan §12: C3 deferred; this task records
  the deferral, doesn't enact it).
- NEVER touch issue #158 (BM post-merge step).
- NEVER commit `.claude/PRPs/debug/v1-quality-r2a-c3-defer.json` (gitignored).

## 5. Commit subject + body

**Subject:**

```
chore(decision-queue): planner-defer C3 emit_reputation_event helper (Issue #158, task 2)
```

**Body:**

```
Implements Plan §13 Task 2 (v1-quality-r2a / Issue #158 / C3):
records the planner's WP-2 DEFER decision as an immutable DQ log entry.

Appends one entry to `.claude/decision-queue.json` `resolved[]`:
- from: planner
- kind: log
- answered_by: planner
- approved_by: null (advisor-exclusive per Hard refusal #8)
- timestamp == resolved_at (kind: "log" entries are immediately self-resolved)
- composite v3 id from scripts/brehon/dq-v3-new-entry.sh

Rationale recorded verbatim from Plan §10.3:
- WP-2 default = defer unless 3rd consumer materialises.
- As of 2026-05-28, federation_inbound lane status='done' in v1-roadmap.json
  — no upcoming sub-phase introduces a 3rd reputation-event emitter.
- Two existing emitter sites: admin_emergency_remove.rs:448 (emit_reputation_event_local)
  and submit_jury_vote.rs:1096 (emit_reputation_event); bodies byte-identical.
- Trigger condition for v1-quality-r3 follow-on: any sub-phase introduces a
  3rd reputation-event emit path.

No `crates/**` change. No script change. Issue #158 stays OPEN with a deferral
comment to be filed by the BM session post-merge.

Worker-side gates:
- python3 verify (1 entry found; kind=log; answered_by=planner; approved_by=null) exit 0
- bash scripts/brehon/dq-lint-durations.sh exit 0 (no negative durations)
- bash scripts/brehon/precheck.sh exit 0 ([precheck] OK)

Mandatory lessons applied per §3.b brief:
- feedback_windows_backslash_path_dq_via_write_fragment.md: fragment authored via
  Write tool with forward-slash literal path.
- feedback_dq_v3_append_via_helper_script.md: dq-v3-append-fragment.sh is the
  canonical mechanism; zero inline Python on the live DQ.
- feedback_principles_not_rules.md: cited in the `answer` field of the new entry.

HANDOVER:
filesCreated: []
filesModified: [.claude/decision-queue.json]
keyDecisions: "One kind:log + from:planner + answered_by:planner entry appended to resolved[]; composite v3 id via helper; timestamp == resolved_at; approved_by=null"
notes: "BM post-merge step: file a deferral comment on Issue #158 quoting the new DQ entry id + answer text; issue stays OPEN (not closed) so v1-quality-r3 has a re-entry point if a 3rd emitter materialises."
```

## 6. Acceptance criteria (advisor-side post-checks)

Advisor verifies after worker reports done:

1. Worker pushed exactly 1 commit:
   `chore(decision-queue): planner-defer C3 emit_reputation_event helper (Issue #158, task 2)`.
2. `git diff phase-v1-quality-r2..<worker-branch> -- .claude/decision-queue.json`
   shows exactly ONE new entry appended to `resolved[]` matching the §2.2
   shape with composite v3 id + `kind: "log"` + `answered_by: "planner"` +
   `approved_by: null`.
3. No other files changed.
4. `bash scripts/brehon/dq-lint-durations.sh` on the worker branch exits 0
   (advisor re-runs as smoke).
5. Daemon false-success check: SSH `homeserver "cd /srv/brehon-fork && git log
   <worker-branch> --oneline -3"` after task reports done; if commit absent from
   origin, advisor manually `git push`s.
