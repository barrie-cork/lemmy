# Brief: v1-quality-r2a fix-impl-1

`[role:impl-task] v1-quality-r2a fix-impl-1 — PR #161 6-finding bundle — see .claude/PRPs/briefs/v1-quality-r2a-fix-impl-1.md`

## 1. Role + dispatch

Junior `impl-task` subagent (Sonnet 4.6) executes a non-allowlist fix-impl bundle on `phase-v1-quality-r2`. Addresses 6 of 8 CR/Copilot findings on PR #161 in a single coherent commit. The other 2 (cr-1 cosmetic + cr-2 parent-plan drift) are `carry-forward` to r2b per advisor triage.

## 2. Scope

**Three files to edit** (all already present on phase-v1-quality-r2 @ 270eb64a5):

- `scripts/brehon/dq-lint-durations.sh` — pass `$DQ_PATH` via `python3 - "$DQ_PATH"` argv (defensive against shell metacharacters; mirrors sibling `dq-v3-append-fragment.sh` pattern). Closes cr-4 + cr-5 (duplicates).
- `scripts/brehon/precheck.sh` — derive `REPO_ROOT` from `SCRIPT_DIR` and pass it as the DQ path argument so the wrapper works from arbitrary CWD. Closes cr-8.
- `.claude/PRPs/plans/v1-quality-r2a.plan.md` — three small fixes: line 671 "4 entries" → "3 entries" (cr-3), line 672 mitigation text matches script (cr-6), line 603 verification checklist wording (cr-7). Single Edit pass.

**Zero changes outside the three files above.** No new tests. No production-code regression. No DQ entry mutation beyond the impl-raised validate-pending-laptop entry at end.

### 2.1 §G4 NON-allowlist (catch-fire avoidance via narrow scope)

This bundle does NOT match any §G4 allowlist row (it's a multi-file shell-pattern fix + plan typo cluster). The advisor reviewed and authored this brief explicitly per gate-3 user-relay approval. Treat the recipes in §2.2-2.4 as the contract; if the worker's understanding of any Edit anchor diverges from §2.2-2.4, raise `kind: "blocker"` DQ — do NOT improvise.

### 2.2 Edit A — `scripts/brehon/dq-lint-durations.sh` argv pattern (closes cr-4 + cr-5)

**File reachable on phase-v1-quality-r2 @ 270eb64a5** (verified at brief-author time). The full file is 29 lines today; replace lines 10-28 (the python3 heredoc block) with the argv-passing shape per the sibling `dq-v3-append-fragment.sh` (lines 117-160) discipline.

Use `old_string` / `new_string`:

`old_string`:
```
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

`new_string`:
```
python3 - "$DQ_PATH" <<'PY'
import json, sys
from datetime import datetime
dq_path = sys.argv[1]
def parse(t):
    if not t: return None
    try:
        return datetime.fromisoformat(t.replace('Z', '+00:00'))
    except ValueError:
        return None
with open(dq_path) as f:
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
    print(f'DQ-LINT FAIL: entry "{eid}" has resolved_at ({ra}) earlier than timestamp ({ts}) by {d}')
sys.exit(1 if bad else 0)
PY
```

**Key changes:**
1. `python3 -c "..."` → `python3 - "$DQ_PATH" <<'PY' ... PY` — DQ_PATH passed as argv, not interpolated into source. `<<'PY'` (single-quoted delimiter) means no shell expansion inside the heredoc.
2. Inside Python: `dq_path = sys.argv[1]` reads the path.
3. `with open('$DQ_PATH') as f` → `with open(dq_path) as f`.
4. The `print(f'... entry \"{eid}\" ...')` → `print(f'... entry "{eid}" ...')` — heredoc no longer needs double-quote escaping since outer shell quotes are gone.
5. Added `try: ... except ValueError: return None` in `parse()` — addresses cr-6 (Copilot's mitigation-mismatch finding) without changing visible behaviour for valid timestamps. Malformed-ISO timestamps now skip silently (consistent with the existing `if not t: return None` semantics for missing values).

### 2.3 Edit B — `scripts/brehon/precheck.sh` REPO_ROOT derivation (closes cr-8)

**File reachable on phase-v1-quality-r2 @ 270eb64a5** (verified at brief-author time). The full file is 8 lines today.

`old_string`:
```
#!/usr/bin/env bash
# scripts/brehon/precheck.sh — run before queueing Junior real-work tasks.
# Exits 0 if all gates pass; non-zero on failure.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "[precheck] dq-lint-durations.sh ..."
"$SCRIPT_DIR/dq-lint-durations.sh"
echo "[precheck] OK"
```

`new_string`:
```
#!/usr/bin/env bash
# scripts/brehon/precheck.sh — run before queueing Junior real-work tasks.
# Exits 0 if all gates pass; non-zero on failure.
#
# Derives REPO_ROOT from SCRIPT_DIR so the lint operates on the canonical
# .claude/decision-queue.json regardless of caller CWD. Mirrors the sibling
# pattern in scripts/brehon/dq-v3-append-fragment.sh.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
echo "[precheck] dq-lint-durations.sh ..."
"$SCRIPT_DIR/dq-lint-durations.sh" "${REPO_ROOT}/.claude/decision-queue.json"
echo "[precheck] OK"
```

**Key changes:**
1. New `REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"` line — mirrors `dq-v3-append-fragment.sh:54`.
2. The lint invocation now passes the explicit canonical DQ path as argv: `"$SCRIPT_DIR/dq-lint-durations.sh" "${REPO_ROOT}/.claude/decision-queue.json"`. The lint's argv default (`$1` → `.claude/decision-queue.json`) still works for direct invocation from repo root or with explicit arg.
3. Added 3 comment lines above `set -euo pipefail` documenting the new behaviour.

### 2.4 Edit C — Plan file three fixes (closes cr-3 + cr-6 + cr-7)

**File reachable on phase-v1-quality-r2 @ 270eb64a5** at `.claude/PRPs/plans/v1-quality-r2a.plan.md`.

This is THREE small edits on the same file. The worker should make them as three separate `Edit` tool calls (each with unique `old_string`), NOT a single multi-line replace. Each anchor cited below should appear exactly once in the file.

**Edit C.1 — line 603 (cr-7): verification checklist wording**

`old_string`:
```
- [ ] Zero edits to files outside §11 list (no `crates/**`, no `migrations/**`, no `tests/**`).
```

`new_string`:
```
- [ ] No edits to `crates/**`, `migrations/**`, `tests/**` (r2a is zero-Rust by design; PR also includes the plan files and `.claude/PRPs/debug/v1-quality-r2a-c3-defer.json` per CR cr-7).
```

**Edit C.2 — line 671 (cr-3): "4 entries" → "3 entries" typo**

`old_string`:
```
| Git log for one of the 4 entries returns multiple resolving-commit candidates | MED | LOW | Per-entry:
```

`new_string`:
```
| Git log for one of the 3 entries returns multiple resolving-commit candidates | MED | LOW | Per-entry:
```

**Edit C.3 — line 672 (cr-6): mitigation text matches script**

The mitigation text needs to match the new behaviour from Edit A (ValueError handled silently). Look at line ~672 — find the line that describes how malformed timestamps are handled, and update it to say malformed ISO timestamps return None and are skipped silently (same as missing values).

`old_string`:
```
| Negative-duration lint fires on a legitimately-resolved entry (false positive) | LOW | LOW | Lint only flags `resolved_at < timestamp`; legitimate entries have `resolved_at >= timestamp` by construction. False positive impossible. |
```

(Worker: if this exact line is not at 672, search the file with `rg "Negative-duration lint"` and verify the surrounding row block. The Edit anchor must be unique — if the text doesn't match, raise `kind: "blocker"` DQ.)

`new_string`:
```
| Negative-duration lint fires on a legitimately-resolved entry (false positive) | LOW | LOW | Lint only flags `resolved_at < timestamp`; legitimate entries have `resolved_at >= timestamp` by construction. False positive impossible. Malformed-ISO timestamps return `None` from `parse()` and are silently skipped, consistent with the missing-value branch. |
```

### 2.5 Post-edit verification (run BEFORE git push)

```bash
# Verify the dq-lint-durations.sh changes
rg -n "python3 - " scripts/brehon/dq-lint-durations.sh  # EXPECT 1 hit (line 10)
rg -n "with open\(dq_path\)" scripts/brehon/dq-lint-durations.sh  # EXPECT 1 hit
rg -n "except ValueError" scripts/brehon/dq-lint-durations.sh  # EXPECT 1 hit (new line)
rg -n "\\\$DQ_PATH" scripts/brehon/dq-lint-durations.sh  # EXPECT 1 hit (the heredoc argv only)

# Verify the precheck.sh changes
rg -n "REPO_ROOT" scripts/brehon/precheck.sh  # EXPECT 2 hits (assignment + use)
rg -n "decision-queue.json" scripts/brehon/precheck.sh  # EXPECT 1 hit (the new explicit arg)

# Verify the plan edits
rg -n "3 entries returns multiple" .claude/PRPs/plans/v1-quality-r2a.plan.md  # EXPECT 1 hit
rg -n "4 entries returns multiple" .claude/PRPs/plans/v1-quality-r2a.plan.md  # EXPECT 0 hits (must be gone)
rg -n "Malformed-ISO timestamps return" .claude/PRPs/plans/v1-quality-r2a.plan.md  # EXPECT 1 hit (new mitigation text)
```

## 3. Required reading (read BEFORE making any edit)

### 3.a Handover from prior cohort (PR #161 ship state)

```yaml
prior_cohort_tasks:
  - task: 0
    commit: 85665f446
    notes: "T0 pre-flight + DQ #3ef987b66db4-001 plan-id mismatch raised + resolved option-A floor."
  - task: 1
    commit: 18bb926ed
    filesCreated:
      - scripts/brehon/dq-lint-durations.sh
      - scripts/brehon/precheck.sh
    notes: "T1 shipped both scripts (29 LOC lint + 8 LOC wrapper) + swept 3 back-dated DQ entries. Closes #157 at merge."
  - task: 2
    commit: 0bbc3502d
    notes: "T2 C3 deferral DQ entry dd6012873857-001 in resolved[]; kind:log + from:planner. Issue #158 stays OPEN for v1-quality-r3 re-entry."
  - PR-open: 47dcc756c
    notes: "PR #161 opened by Junior bm-task #496 at 2026-05-29T00:23:45Z. Base governance-v0, head phase-v1-quality-r2."
  - CR-review: 2026-05-29T00:28:10Z
    notes: "CR posted 4 actionable + Copilot reviewed with 4 findings; 1 duplicate. Advisor reconciled to 8 unique findings, 6 bucketed fix-in-pr, 2 carry-forward."
```

### 3.b Mandatory lessons

1. `.claude/lessons/feedback_windows_backslash_path_dq_via_write_fragment.md` — when authoring the DQ pending entry at §4.3, use forward-slash literal paths and the Write tool for fragments.
2. `.claude/lessons/feedback_dq_v3_append_via_helper_script.md` — append the validate-pending DQ via `bash scripts/brehon/dq-v3-append-fragment.sh` with the `--pending` flag.
3. `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate all 5 Edit anchors (3 plan + 1 dq-lint heredoc + 1 precheck.sh) and run the rg verification BEFORE committing. The brief scope is small (3 files, 5 Edits) — within the lesson's safe band.
4. `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — no cargo gate fires on bash-only changes, BUT worker MUST run the local `bash scripts/brehon/precheck.sh` after edits to confirm it still exits 0.

### 3.c CR/Copilot finding reconciliation (full picture)

From `.claude/PRPs/reviews/pr-161-findings.yaml` (gitignored runtime artifact reconstructed by advisor at brief-author time after Junior #497 bm-poll-cr lost both deliverables to .gitignore + permission-denial):

| Finding | Source | Severity | File | Bucket |
|---|---|---|---|---|
| cr-1 | CR | nit | `.claude/PRPs/debug/v1-quality-r2a-c3-defer.json:11` | carry-forward (cosmetic) |
| cr-2 | CR | medium | `.claude/PRPs/plans/v1-quality-r2.plan.md:27` | carry-forward (parent-plan drift; r2b's job) |
| cr-3 | CR | medium | `v1-quality-r2a.plan.md:671` | **fix-in-pr** (typo) — Edit C.2 |
| cr-4 | CR | major | `dq-lint-durations.sh:29` | **fix-in-pr** (shell injection class) — Edit A |
| cr-5 | Copilot | major | `dq-lint-durations.sh:16` | **fix-in-pr** (DUP of cr-4) — Edit A |
| cr-6 | Copilot | medium | `v1-quality-r2a.plan.md:672` | **fix-in-pr** (mitigation matches script) — Edit C.3 + Edit A safety net |
| cr-7 | Copilot | medium | `v1-quality-r2a.plan.md:603` | **fix-in-pr** (checklist wording) — Edit C.1 |
| cr-8 | Copilot | medium | `precheck.sh:7` | **fix-in-pr** (CWD-relative DQ path) — Edit B |

## 4. Constraints

### 4.1 Hard refusals

1. **NEVER edit any file outside the three listed in §2.** No `crates/**`, no `migrations/**`, no `tests/**`, no `Cargo.toml`, no other plan files.
2. **NEVER add or modify a Rust test.** This is a bash + plan-prose fix bundle.
3. **NEVER `#[allow]`-spam or shellcheck-disable directives.** The argv pattern is the correct structural fix; sidestepping it via lint disables is wrong-shaped.
4. **NEVER write `"answered_by": "advisor"` or `"approved_by": <value>` in any DQ entry.** Advisor-exclusive per DQ Hard refusals #1 + #8.
5. **NEVER use the abolished `next_id = max(all_ids)+1` recipe.** Use `bash scripts/brehon/dq-v3-new-entry.sh` per DQ Hard refusal #9.

### 4.2 Worker self-test (run BEFORE `git push`)

```bash
# 1. Verify the new dq-lint-durations.sh runs against the live DQ
bash scripts/brehon/dq-lint-durations.sh
echo "exit: $?"
# EXPECT exit 0 (live DQ post-r2a sweep has no negative durations)

# 2. Verify precheck.sh still exits 0 with the new REPO_ROOT derivation
bash scripts/brehon/precheck.sh
echo "exit: $?"
# EXPECT exit 0; output includes "[precheck] OK"

# 3. Verify precheck.sh works from a non-root CWD (cr-8 regression check)
cd /tmp && bash "${REPO_ROOT_ABS}/scripts/brehon/precheck.sh"
echo "exit: $?"
# EXPECT exit 0 (now works from any CWD because REPO_ROOT is derived)

# 4. Negative test for argv injection — feed a path with single quote
echo "{}" > "/tmp/test'inject.json"
bash scripts/brehon/dq-lint-durations.sh "/tmp/test'inject.json"
echo "exit: $?"
# EXPECT: cleanly reports zero entries (exit 0) OR a Python error about empty
# JSON shape — NEITHER a shell parse error NOR arbitrary code execution.
# The pre-fix code WOULD have errored at the python source parse stage on the
# single quote. Verifies cr-4/cr-5 are actually fixed.
rm "/tmp/test'inject.json"

# 5. All rg verifications from §2.5 pass
```

If ANY of the worker self-tests fail, raise `kind: "blocker"` DQ with the failing command + log slice. Do NOT push and do NOT raise a validate-pending entry.

### 4.3 validate-pending-laptop DQ entry (post-push)

After ALL §4.2 worker self-tests exit 0 AND `git push origin <worker-branch>` succeeds:

```bash
NEW_ID=$(bash scripts/brehon/dq-v3-new-entry.sh)
echo "$NEW_ID"
```

Then write fragment to `.claude/PRPs/debug/v1-quality-r2a-fix-impl-1-vp.json` (forward-slash literal path; use Write tool):

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 at fragment-write time>",
  "question": "Run dq-lint-durations.sh + precheck.sh self-tests on canonical brehon-fork lane for v1-quality-r2a fix-impl-1 (PR #161 6-finding bundle). No cargo gate (zero Rust change).",
  "options": ["all gates pass", "any gate fails"],
  "context": "fix-impl-1 addresses 6 of 8 PR #161 findings: cr-3+cr-6+cr-7 plan edits + cr-4+cr-5 dq-lint argv pattern + cr-8 precheck REPO_ROOT. Worker self-tests passed; laptop-side re-runs the same gates against canonical brehon-fork (which sees phase-v1-quality-r2 after the next bm-merge-forward).",
  "answer": null,
  "answered_by": null,
  "approved_by": null,
  "resolved_at": null,
  "workflow_run_id": null,
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "branch": "<worker-branch-name>",
  "phase_task": "v1-quality-r2a-fix-impl-1",
  "commands": [
    "bash scripts/brehon/dq-lint-durations.sh > .claude/PRPs/debug/v1-quality-r2a-fix-impl-1-lint.log 2>&1",
    "bash scripts/brehon/precheck.sh > .claude/PRPs/debug/v1-quality-r2a-fix-impl-1-precheck.log 2>&1"
  ]
}
```

Then append via helper:

```bash
bash scripts/brehon/dq-v3-append-fragment.sh .claude/PRPs/debug/v1-quality-r2a-fix-impl-1-vp.json --pending
```

### 4.4 Commit subjects

ONE feat commit + ONE chore(decision-queue) commit on the worker branch:

```
feat(quality): fix-impl-1 — PR #161 6-finding bundle (cr-3..cr-8) for v1-quality-r2a

chore(decision-queue): impl raised validate-pending-laptop for v1-quality-r2a fix-impl-1
```

Each commit body MUST end with a `HANDOVER:` YAML trailer per `feedback_handover_trailer_cohort_propagation.md`.

The feat commit body should include the per-finding fix mapping (cr-3 → Edit C.2; cr-4 + cr-5 → Edit A; cr-6 → Edit C.3 + Edit A; cr-7 → Edit C.1; cr-8 → Edit B) so the PR's findings YAML can `addressed_in: <sha>` each.

## 5. Definition of done

- [ ] All 5 Edit anchors applied per §2.2-2.4; all 8 `rg -n` checks in §2.5 pass.
- [ ] All 4 worker self-tests in §4.2 exit 0 (plus the rg checks at step 5).
- [ ] Two commits on worker branch with subjects matching §4.4.
- [ ] Worker branch pushed to `origin/junior/...`.
- [ ] New `kind: validate-pending-laptop` entry in `.claude/decision-queue.json` `pending[]` with composite v3 id, `from: impl`, `answered_by: null`, `approved_by: null`.
- [ ] `HANDOVER:` trailer on both commits.
- [ ] feat commit body lists per-finding fix mapping.

## 6. Out of scope (do NOT do)

- Any edit to `crates/`, `Cargo.toml`, `Cargo.lock`, `migrations/`, `tests/`, `docs/`.
- Any edit to `v1-quality-r2.plan.md` (parent plan, carry-forward to r2b).
- Any edit to `v1-quality-r2a-c3-defer.json` (cr-1 nit — carry-forward).
- Posting any comment on PR #161 (BM autonomy bound).
- Submitting any PR review (BM autonomy bound).
- Closing or commenting on Issues #157 / #158 (BM does that at merge time).
- Authoring the Lane Q-r2a retro update (advisor-driven).
