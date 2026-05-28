# Brief: impl-task 1 — v1-quality-r2a DQ negative-duration lint + sweep (closes #157)

**Role:** `[role:impl-task]`
**Phase:** `v1-quality-r2a` (branch stays `phase-v1-quality-r2`; plan filename signals scope)
**Authored:** 2026-05-28
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base branch (Junior forks from):** `phase-v1-quality-r2`
**Lane mode:** Mode B (mobile remote-control). Junior worker runs on the EliteDesk daemon (Linux).

---

## 1. Role + dispatch

`[role:impl-task] v1-quality-r2a task 1 — DQ duration lint + sweep — see .claude/PRPs/briefs/v1-quality-r2a-impl-1.md`

---

## 2. Scope

Ship **2 new shell scripts + 1 atomic DQ sweep**, all on the phase branch's worker worktree, then push. Closes issue #157 (DQ negative-duration data corruption gate).

**Files modified (exactly 3 — all listed in plan §13 Task 1 FILES yaml):**
- `scripts/brehon/dq-lint-durations.sh` — **NEW** (~50 LOC actual; plan §10.1 estimates ~80 LOC including comments)
- `scripts/brehon/precheck.sh` — **NEW** (~10 LOC)
- `.claude/decision-queue.json` — **MODIFIED** (3 entries' `resolved_at` floored to `timestamp`)

**No edits to any other file.** No `crates/**`, no `migrations/**`, no `tests/**`, no `docs/**`.

### 2.1 §G4 CANONICAL RECIPE — not applicable

This task is original feature work, not a §G4-classified fix-impl. No prior workflow_run_id; no allowlist row to copy verbatim. Worker follows plan §13 Task 1 + §10.1 + §10.2 directly.

### 2.2 Sub-task A — `scripts/brehon/dq-lint-durations.sh`

**MIRROR (read these BEFORE authoring the script):**
- `scripts/brehon/resolve-dq-canonical.sh` — DQ-iteration pattern (`pending[]` + `resolved[]` traversal idiom).
- `scripts/brehon/dq-schema-v3-migrate.sh` — JSON mutation idiom (`set -euo pipefail`, here-doc Python invocation, exit-code propagation).

**Script body (verbatim from plan §10.1):**

```bash
#!/usr/bin/env bash
# scripts/brehon/dq-lint-durations.sh — flag DQ entries where resolved_at < timestamp.
# Exits 0 if no negative-duration entries; non-zero with a list otherwise.
# Composite-id-aware (schema-v3): reports by `id` verbatim.
#
set -euo pipefail
DQ_PATH="${1:-.claude/decision-queue.json}"
[ -f "$DQ_PATH" ] || { echo "FATAL: not found: $DQ_PATH" >&2; exit 2; }

python3 -c "
import json, sys
from datetime import datetime
def parse(t):
    if not t: return None
    return datetime.fromisoformat(t.replace('Z', '+00:00'))
with open('$DQ_PATH') as f:
    dq = json.load(f)
bad = []
for arr in ('pending', 'resolved'):
    for e in dq.get(arr, []):
        ts = parse(e.get('timestamp'))
        ra = parse(e.get('resolved_at'))
        if ts and ra and ra < ts:
            delta = ts - ra
            bad.append((str(e.get('id')), e.get('timestamp'), e.get('resolved_at'), str(delta)))
for (eid, ts, ra, d) in bad:
    print(f'DQ-LINT FAIL: entry \"{eid}\" has resolved_at ({ra}) earlier than timestamp ({ts}) by {d}')
sys.exit(1 if bad else 0)
"
```

After writing the file: `chmod +x scripts/brehon/dq-lint-durations.sh`.

### 2.3 Sub-task B — `scripts/brehon/precheck.sh`

**Script body (verbatim from plan §10.2):**

```bash
#!/usr/bin/env bash
# scripts/brehon/precheck.sh — run before queueing Junior real-work tasks.
# Exits 0 if all gates pass; non-zero on failure.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "[precheck] dq-lint-durations.sh ..."
"$SCRIPT_DIR/dq-lint-durations.sh"
echo "[precheck] OK"
```

After writing the file: `chmod +x scripts/brehon/precheck.sh`.

### 2.4 Sub-task C — `.claude/decision-queue.json` floor-sweep

**Three target entries on the worker branch's `.claude/decision-queue.json` (verified back-dated at phase tip `3ff47b26e`, 2026-05-28T20:55Z):**

| id | timestamp | resolved_at (current) |
|---|---|---|
| `315` | `2026-05-21T10:45:00Z` | `2026-05-21T08:53:00Z` |
| `1b8527b076d4-001` | `2026-05-25T19:30:00Z` | `2026-05-25T19:11:20.705989Z` |
| `81719cf8ca8d-001` | `2026-05-26T19:10:00Z` | `2026-05-26T07:08:50.884428Z` |

**Sweep policy (Option A floor):** for each of the 3 ids, if `resolved_at < timestamp`, set `resolved_at = timestamp`. This is the conservative default; Option B (resolve from git history) is allowed PER ENTRY only if `git log --all --format='%H %cI' --grep='<id>' | head -1` unambiguously returns a single commit-date AFTER `timestamp` AND BEFORE the current `resolved_at`. Per `feedback_falsifiable_hypothesis_before_structural_fix.md` — ambiguous git-log output triggers Option A floor for that entry.

**Recommended path:** apply Option A (floor) to all 3 entries. The plan does not require Option B exploration; only the planner offered it as an option. Option A is simpler, deterministic, and aligned with the post-sweep lint passing.

**Atomic protocol (per `multi-lane-worktree.md` Hard refusal #6 + `decision-queue.md` "Mid-task visibility"):**

```bash
# All in one shell sequence — do NOT split across multiple tool calls.
git fetch origin phase-v1-quality-r2
# (worker is on junior/* off phase-v1-quality-r2; ff-only is automatic on first fetch)

python3 -c "
import json
with open('.claude/decision-queue.json') as f:
    dq = json.load(f)
target_ids = {'315', 315, '1b8527b076d4-001', '81719cf8ca8d-001'}
swept = []
for e in dq.get('resolved', []):
    if e.get('id') in target_ids or e.get('id_v1') in target_ids:
        ts = e.get('timestamp')
        ra = e.get('resolved_at')
        if ts and ra and ra < ts:
            e['resolved_at'] = ts
            swept.append(e.get('id'))
print(f'swept {len(swept)} entries: {swept}')
with open('.claude/decision-queue.json', 'w') as f:
    json.dump(dq, f, ensure_ascii=False, indent=2)
    f.write('\n')
"
# EXPECT: swept 3 entries: ['315', '1b8527b076d4-001', '81719cf8ca8d-001']

# Verify the lint passes BEFORE staging
bash scripts/brehon/dq-lint-durations.sh > .claude/PRPs/debug/v1-quality-r2a-task1-postsweep.log 2>&1
echo "lint exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r2a-task1-postsweep.log
# EXPECT: lint exit: 0; no DQ-LINT FAIL lines in the log
```

**SINGLE commit covers all three sub-tasks** (lint script + precheck wrapper + sweep). Plan §13 Task 1 GOTCHA: "The sweep is a SINGLE commit, not 4 commits" — this generalises to the whole task: one commit covers files 1, 2, and 3.

### 2.5 Insertion / authoring notes

- **Use the `Write` tool** to create both shell scripts. After each Write, run `chmod +x <path>`.
- **Use Python inline** for the sweep mutation (the `python3 -c "..."` block above) — do NOT use the `Edit` tool on `.claude/decision-queue.json` directly (the file is JSON and pretty-printed indentation is load-bearing; Python's `json.dump(..., indent=2)` preserves the canonical format).
- **Do NOT pre-compute `resolved_at`** in a way that introduces a NEW back-dated entry — when the lint runs post-sweep, any newly-introduced back-dated entry will trip it and fail the §15.2 gate.

---

## 3. Required reading

- `.claude/PRPs/plans/v1-quality-r2a.plan.md` §10.1 (lint script body), §10.2 (precheck body), §13 Task 1 (action + 3 IMPLEMENT files), §15.2 (post-sweep gate), §16a Story 1 (verify gate)
- `.claude/PRPs/briefs/v1-quality-r2a-impl-0.md` — Task 0 precedent (probe-style env verification; SAME branch / same worker mode)
- `.claude/rules/decision-queue.md` Hard refusals #1, #6, #8, #9 + §"Mid-task visibility" + §"Recipe 2" + §"Recipe 3"
- `.claude/rules/multi-lane-worktree.md` Hard refusal #6 (atomic read-mutate-commit-push protocol)
- `.claude/lessons/feedback_falsifiable_hypothesis_before_structural_fix.md` — Option A floor vs Option B git-log resolution
- `.claude/lessons/feedback_principles_not_rules.md` — referenced from r2a plan §3 (deferral discipline)
- `.claude/lessons/feedback_pipes_mask_exit_codes.md` — redirect-then-tail pattern (Lane A Task 0 #489 + Lane Q-r2a Task 0 #490 both encountered this; the lint's `tail -20 <log>` after `bash <script> > <log>` is the correct shape)
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` — submodule init pre-empt (no cargo in this task, but Junior worktrees still need init for completeness; see §4)
- `scripts/brehon/resolve-dq-canonical.sh` — DQ-iteration MIRROR
- `scripts/brehon/dq-schema-v3-migrate.sh` — JSON-mutation MIRROR

---

## 4. Constraints

- **File ownership:** edit ONLY the 3 files listed in §2 (`scripts/brehon/dq-lint-durations.sh`, `scripts/brehon/precheck.sh`, `.claude/decision-queue.json`). No other file edits.
- **Submodule init pre-empt:** before any cargo invocation (Task 1 has none, but the worker's `git submodule update --init --recursive` is still good hygiene per `feedback_phase_lane_worktree_bootstrap_checklist.md`; run it if the worker plans to also run the lint synthetic-negative check that touches `.claude/PRPs/debug/dq-fixture.json`).
- **Pipe-mask-exit-code discipline:** when capturing the lint's exit code, always redirect to a log file FIRST, then `tail` the log; never pipe the lint through `tail`/`head` directly. Lane A Task 0 (#489) + Lane Q-r2a Task 0 (#490) both encountered the trap.
- **No `cargo` invocations** — Task 1 ships zero Rust changes. The §15.2 self-test is a plain `bash scripts/brehon/dq-lint-durations.sh` invocation; no cargo gate.
- **Atomic read-mutate-commit-push** — the sweep + the script adds + the precheck add must be ONE commit. Per `multi-lane-worktree.md` Hard refusal #6, the sequence is: `git fetch origin phase-v1-quality-r2` → write scripts → chmod +x → run sweep Python block → verify lint passes locally → `git add scripts/brehon/dq-lint-durations.sh scripts/brehon/precheck.sh .claude/decision-queue.json` → `git commit` → `git push origin <worker-branch>`. **No intermediate commits.** No splitting across tool calls between mutate and add.

- **Attribution (Hard refusals):** the worker's commits on `.claude/decision-queue.json` use no `answered_by` field at all for THIS task (the sweep modifies existing `resolved_at` fields, it does NOT answer or resolve new pending entries). `from`, `kind`, `answered_by` are NOT touched on the 3 swept entries — only `resolved_at` changes.

  **NEVER write `answered_by: "advisor"` or `answered_by: "user"` or `approved_by: <non-null>`** per `.claude/rules/decision-queue.md` Hard refusals #1, #6, #8.

- **Commit subject (verbatim from plan §13 Task 1):** `feat(scripts/brehon): add dq-lint-durations.sh + precheck.sh + sweep back-dated DQ entries (closes #157, task 1)`. Include a `HANDOVER:` YAML trailer in the commit body per the impl-task template (filesCreated: `[scripts/brehon/dq-lint-durations.sh, scripts/brehon/precheck.sh]`; filesModified: `[.claude/decision-queue.json]`; keyDecisions: "Option A floor applied to all 3 entries; lint composite-id-aware; precheck wires lint as first gate").

- **Mid-task push discipline:** if the worker hits a blocker BEFORE the sweep is complete (e.g. synthetic-negative check fails the lint script for an unexpected reason; lint script's Python block emits FATAL), commit + push the partial state on the worker branch with a `kind: "blocker"` DQ entry and stop. Per `decision-queue.md` §"Mid-task visibility".

- **Worker-side §15.2 self-test (mandatory before push):**

  ```bash
  bash scripts/brehon/dq-lint-durations.sh > .claude/PRPs/debug/v1-quality-r2a-task1-postsweep.log 2>&1
  echo "lint exit: $?"
  tail -20 .claude/PRPs/debug/v1-quality-r2a-task1-postsweep.log
  # EXPECT: lint exit: 0; no DQ-LINT FAIL lines

  # Synthetic-negative check (lint must FAIL on a back-dated fixture)
  cp .claude/decision-queue.json .claude/PRPs/debug/dq-fixture.json
  python3 -c "
  import json
  with open('.claude/PRPs/debug/dq-fixture.json') as f: dq = json.load(f)
  if dq['resolved']:
      dq['resolved'][0]['resolved_at'] = '2020-01-01T00:00:00+00:00'
  with open('.claude/PRPs/debug/dq-fixture.json', 'w') as f: json.dump(dq, f)
  "
  bash scripts/brehon/dq-lint-durations.sh .claude/PRPs/debug/dq-fixture.json > .claude/PRPs/debug/v1-quality-r2a-task1-synthetic.log 2>&1
  echo "synthetic exit: $?"
  tail -10 .claude/PRPs/debug/v1-quality-r2a-task1-synthetic.log
  # EXPECT: synthetic exit: 1 (non-zero); output contains DQ-LINT FAIL

  bash scripts/brehon/precheck.sh > .claude/PRPs/debug/v1-quality-r2a-task1-precheck.log 2>&1
  echo "precheck exit: $?"
  tail -10 .claude/PRPs/debug/v1-quality-r2a-task1-precheck.log
  # EXPECT: precheck exit: 0; "[precheck] OK"
  ```

  All three gates must pass before push. If the synthetic-negative check passes (exit 0) instead of failing — meaning the lint did NOT catch a known back-dated entry — that's a lint-correctness bug; file `kind: "blocker"` DQ and stop.

- **No `kind: "validate-pending-laptop"` DQ for Task 1.** This task ships zero Rust; the §15.2 lint self-test is the ENTIRE validation gate. The plan does not call for a laptop cargo validation handoff at T1. The lint runs on the daemon and is self-validating.

- **Shape G non-binding** for r2a (no cargo). Forbidden-window check non-binding for the worker-side lint invocation (cargo-only check). Advisor verified at queue time.

- **Concurrent DQ-write hazard awareness:** Lane A Task 1 (#491) is concurrently running on a DIFFERENT phase branch (`phase-v1-redaction-r1`) and a DIFFERENT worker worktree. The two lanes are file-disjoint:
  - Lane A's #491 modifies `crates/db_schema/src/source/governance/redaction.rs` only.
  - Lane Q-r2a's THIS task modifies `scripts/brehon/dq-lint-durations.sh`, `scripts/brehon/precheck.sh`, and `.claude/decision-queue.json` only.
  - The two lanes write `.claude/decision-queue.json` on different phase branches; concurrent writes on the same path are isolated by branch. Per `feedback_cohort_shared_git_index_contention.md`, the cohort-≥3 `.git/index.lock` threshold does NOT trigger for 2 active lanes on the daemon's single `.git/`.

---

## 5. Forbidden-window check (advisor pre-queue)

Per `.claude/rules/advisor-orchestrator.md` §5.1: Shape G suspended; cargo forbidden windows non-binding. **Lint-only task has no cargo runtime → no forbidden-window binding at all.** Advisor verified at queue time (~21:30 UTC, outside any conceivable window). Subagent's pre-flight refuses with `FORBIDDEN_WINDOW: <window>` only if mis-queued during a cargo-binding window; for this task, the check is vacuous.

---

## 6. Context

- **Phase:** v1-quality-r2a (narrowed scope per gate-1 split; closes #157)
- **Plan:** `.claude/PRPs/plans/v1-quality-r2a.plan.md` on `phase-v1-quality-r2` at `3ff47b26e` (post-amendment HEAD)
- **Original full-scope predecessor:** `v1-quality-r2.plan.md` on same branch — historical context for r2b only; NOT read by Junior for r2a Task 1
- **Phase branch:** `phase-v1-quality-r2` (cut from governance-v0; current tip `3ff47b26e`)
- **Base branch for this task:** `phase-v1-quality-r2`
- **Lane mode:** Mode B (no laptop-side phase worktree)
- **Plan-amended at:** 2026-05-28T20:55Z (advisor option-A resolution of DQ #3ef987b66db4-001 — Task 0 worker correctly identified plan-authoring bug; plan §13 Task 1 updated to sweep 3 real ids `315`/`1b8527b076d4-001`/`81719cf8ca8d-001`)
- **Task 0 precedent:** #490 completed 2026-05-28T20:27 with 8 of 9 probes PASS + 1 plan-defect catch (Probe 6); blocker resolved at `3ff47b26e`; submodule init verified, all `bash scripts/brehon/cargo-*.sh` wrappers honor flags, `dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh` both present (and `dq-v3-append-fragment.sh` is now `100755` post-chmod fix at `817291c9a`).
- **Concurrent lane activity:** Lane A Task 1 #491 is running on `phase-v1-redaction-r1` worker worktree. Zero file overlap (Lane A = `crates/db_schema/src/source/governance/redaction.rs`; Lane Q-r2a Task 1 = scripts + decision-queue.json). Per `feedback_cohort_shared_git_index_contention.md` 2 active workers on the daemon is below the cohort-≥3 `.git/index.lock` cascade threshold.
- **After Task 1 push + self-test green:** advisor reads task output, verifies single commit + the post-sweep DQ state via `git show`, queues Task 2 brief author + dispatch.
- **If Task 1 surfaces a blocker DQ:** advisor reads the DQ on next poll, routes per §5.4 DQ triage decision tree.

---

## 7. Acceptance criteria

- `scripts/brehon/dq-lint-durations.sh` exists; has `#!/usr/bin/env bash`, `set -euo pipefail`; is `+x`; iterates `pending[]` + `resolved[]`; emits `DQ-LINT FAIL: entry "<id>" has resolved_at (<rfc3339>) earlier than timestamp (<rfc3339>) by <duration>` on stdout per finding; exits non-zero iff any finding.
- `scripts/brehon/precheck.sh` exists; has `#!/usr/bin/env bash`, `set -euo pipefail`; is `+x`; resolves `SCRIPT_DIR` via `BASH_SOURCE[0]`; calls `dq-lint-durations.sh`; prints `[precheck] OK` on success.
- `.claude/decision-queue.json` entries `id=315`, `id=1b8527b076d4-001`, `id=81719cf8ca8d-001` have `resolved_at == timestamp` (Option A floor applied). No other entry's `resolved_at` modified.
- Worker-side `bash scripts/brehon/dq-lint-durations.sh` exits 0 on the post-sweep DQ.
- Worker-side synthetic-negative check (lint against `.claude/PRPs/debug/dq-fixture.json` with one entry's `resolved_at` set to `2020-01-01T00:00:00+00:00`) exits non-zero; output contains a `DQ-LINT FAIL` line.
- Worker-side `bash scripts/brehon/precheck.sh` exits 0; output contains `[precheck] OK`.
- Exactly ONE commit on the worker branch with subject `feat(scripts/brehon): add dq-lint-durations.sh + precheck.sh + sweep back-dated DQ entries (closes #157, task 1)` and a `HANDOVER:` trailer.
- Worker branch pushed to origin.
