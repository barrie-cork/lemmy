# Brief: v1-quality-r2a fix-impl-3

`[role:impl-task] v1-quality-r2a fix-impl-3 — fail-closed semantics + timestamp accuracy — see .claude/PRPs/briefs/v1-quality-r2a-fix-impl-3.md`

## 1. Role + dispatch

Junior `impl-task` subagent (Sonnet 4.6) executes a policy reversal on `phase-v1-quality-r2`. Addresses 2 new CR findings on the fix-impl-2 tip + 1 Copilot duplicate. CR's re-review on commit `2f106c714` posted these new findings that effectively reverse cr-6's framing.

## 2. Scope

**Three files to edit, three discrete changes:**

- `scripts/brehon/dq-lint-durations.sh` — remove the `try/except ValueError` wrapper from `parse()`; on malformed timestamp, the lint must report the offending entry id AND exit non-zero (fail closed). cr-new-2 (Major, `#3321737411`) + Copilot `#3321383511` (same class).
- `.claude/PRPs/plans/v1-quality-r2a.plan.md` — update risks-table row at line 672 to describe fail-closed behaviour (the cr-new-2 suggestion diff).
- `.claude/PRPs/debug/v1-quality-r2a-fix-impl-1-c3-blocker.json` — replace placeholder timestamp `"2026-05-29T00:00:00Z"` with actual blocker-creation time `"2026-05-29T01:14:34Z"` (the commit timestamp of `8621a84bb` where DQ `745da950edad-001` was first raised). cr-new-1 (Minor, `#3321685668`).

**Zero changes outside the three files above.** No new tests beyond worker self-tests below. No new DQ entries beyond the impl-raised validate-pending-laptop at end. No cargo (zero Rust).

### 2.1 §G4 NON-allowlist

This is CR-triage re-review work, not §G4 workflow-fail. No verbatim §G4 row required. Recipes in §2.2-2.4 are the contract; if any anchor diverges, raise `kind: "blocker"` DQ — do NOT improvise.

### 2.2 Edit A — `scripts/brehon/dq-lint-durations.sh` fail-closed parse()

**Anchor verified by advisor 2026-05-29 against `phase-v1-quality-r2 @ 2f106c714`.** The file is 33 lines after fix-impl-1's Edit A. Lines 14-19 contain the current `parse()` with silent-skip:

```python
def parse(t):
    if not t: return None
    try:
        return datetime.fromisoformat(t.replace('Z', '+00:00'))
    except ValueError:
        return None
```

The change reverses the silent-skip to fail-closed. Two sub-edits in one Edit pass:

**Sub-edit A.1**: replace the parse() body with strict-raise:

`old_string`:

```
def parse(t):
    if not t: return None
    try:
        return datetime.fromisoformat(t.replace('Z', '+00:00'))
    except ValueError:
        return None
```

`new_string`:

```
def parse(t, entry_id, field):
    if not t: return None
    try:
        return datetime.fromisoformat(t.replace('Z', '+00:00'))
    except ValueError:
        print(f'DQ-LINT FAIL: entry "{entry_id}" {field}="{t}" is not a valid ISO-8601 timestamp', file=sys.stderr)
        raise SystemExit(1)
```

**Sub-edit A.2**: update the two parse() call sites to pass `entry_id` and `field`:

`old_string`:

```
        ts = parse(e.get('timestamp'))
        ra = parse(e.get('resolved_at'))
```

`new_string`:

```
        ts = parse(e.get('timestamp'), e.get('id'), 'timestamp')
        ra = parse(e.get('resolved_at'), e.get('id'), 'resolved_at')
```

These two sub-edits constitute Edit A. After both apply, the script is:

- Returns `None` only when the field is missing/empty (legitimate — entry never resolved).
- Raises `SystemExit(1)` with a `DQ-LINT FAIL` line naming the entry id + field + bad value when the timestamp is present but malformed.
- The existing negative-duration loop is unaffected.

**Mid-loop SystemExit is preserved** by `raise SystemExit(1)` propagating up through the for-loop. The script no longer reaches the final `sys.exit(1 if bad else 0)` for malformed-timestamp paths — that's the desired fail-closed behaviour.

### 2.3 Edit B — `.claude/PRPs/plans/v1-quality-r2a.plan.md` line 672

**Anchor verified by advisor 2026-05-29 against `phase-v1-quality-r2 @ 2f106c714`.** Line 672 currently reads (post-fix-impl-2):

`old_string`:

```
| `dq-lint-durations.sh` regex misses an edge-case timestamp format | LOW | MED | Python `fromisoformat` handles all valid ISO-8601; malformed timestamps raise `ValueError` which the `parse()` wrapper catches and returns `None`, so the entry is silently skipped rather than crashing the lint |
```

`new_string` (per cr-new-2's exact suggestion diff):

```
| `dq-lint-durations.sh` misses or receives malformed timestamp formats | LOW | MED | Parse timestamps with strict error handling; if any `timestamp`/`resolved_at` is malformed, emit `DQ-LINT FAIL` with entry id and exit non-zero (fail closed), rather than skipping |
```

Two changes: (1) row label edited; (2) mitigation text describes fail-closed.

### 2.4 Edit C — `.claude/PRPs/debug/v1-quality-r2a-fix-impl-1-c3-blocker.json` line 4

**Anchor verified by advisor 2026-05-29 against `phase-v1-quality-r2 @ 2f106c714`.** Line 4 currently reads `"timestamp": "2026-05-29T00:00:00Z",`.

`old_string`:

```
  "timestamp": "2026-05-29T00:00:00Z",
```

`new_string`:

```
  "timestamp": "2026-05-29T01:14:34Z",
```

The new timestamp is the commit time of `8621a84bb` (`feat(quality): fix-impl-1 — PR #161 6-finding bundle...`) — the commit on which Junior #498 raised the blocker. Source: `git log -1 --format=%cI 8621a84bb` returns `2026-05-29T01:14:34+00:00`.

### 2.5 Worker self-tests (run AFTER all three Edits, BEFORE git add)

In order:

1. **Anchor verification for Edit A.1 success**: `rg "raise SystemExit\(1\)" scripts/brehon/dq-lint-durations.sh` → exactly **1** match.
2. **Anchor verification for Edit A.2 success**: `rg "parse\(e\.get\('timestamp'\), e\.get\('id'\), 'timestamp'\)" scripts/brehon/dq-lint-durations.sh` → exactly **1** match.
3. **Old silent-skip absent**: `rg "return None$" scripts/brehon/dq-lint-durations.sh` → exactly **1** match (the missing-value `if not t: return None` line; NOT the old except ValueError → return None).
4. **Edit B success**: `rg "Parse timestamps with strict error handling" .claude/PRPs/plans/v1-quality-r2a.plan.md` → exactly **1** match on line 672.
5. **Edit B old text absent**: `rg "malformed timestamps raise .ValueError. which the .parse" .claude/PRPs/plans/v1-quality-r2a.plan.md` → exactly **0** matches.
6. **Edit C success**: `rg "2026-05-29T01:14:34Z" .claude/PRPs/debug/v1-quality-r2a-fix-impl-1-c3-blocker.json` → exactly **1** match.
7. **Edit C old text absent**: `rg "2026-05-29T00:00:00Z" .claude/PRPs/debug/v1-quality-r2a-fix-impl-1-c3-blocker.json` → exactly **0** matches.
8. **POSITIVE gate — clean DQ exit 0**: `bash scripts/brehon/dq-lint-durations.sh .claude/decision-queue.json` → exit 0. (Confirms no regression on legit data; current phase-branch DQ has 0 malformed timestamps.)
9. **NEGATIVE gate — fail-closed verification**: create a test DQ at `/tmp/test-malformed-dq.json` with a single resolved entry whose `timestamp` is a malformed string (e.g. `"not-an-iso-date"`); run `bash scripts/brehon/dq-lint-durations.sh /tmp/test-malformed-dq.json`; verify exit **1** AND verify stderr contains the literal string `DQ-LINT FAIL: entry "<the-test-id>"` with the bad value. Remove the test DQ after. **Recipe**:
   ```bash
   cat > /tmp/test-malformed-dq.json <<'EOF'
   {"schema_version": 3, "pending": [], "resolved": [{"id": "test-malformed-001", "from": "test", "kind": "log", "timestamp": "not-an-iso-date", "resolved_at": "2026-05-29T00:00:00Z", "answer": "test", "answered_by": "test", "approved_by": null, "approved_at": null}]}
   EOF
   bash scripts/brehon/dq-lint-durations.sh /tmp/test-malformed-dq.json
   rc=$?
   rm /tmp/test-malformed-dq.json
   if [ "$rc" -ne 1 ]; then
     echo "FAIL: expected exit 1, got $rc" >&2
     exit 1
   fi
   ```
10. **No scope creep**: `git diff phase-v1-quality-r2 -- crates/ migrations/ tests/ docs/ Cargo.toml Cargo.lock` returns empty.
11. **Plan + script + debug-json diff**: `git diff --stat phase-v1-quality-r2` shows exactly **3** files changed (`scripts/brehon/dq-lint-durations.sh`, `.claude/PRPs/plans/v1-quality-r2a.plan.md`, `.claude/PRPs/debug/v1-quality-r2a-fix-impl-1-c3-blocker.json`) plus the DQ change for the validate-pending-laptop entry.

All eleven must exit cleanly. Any failure → raise `kind: "blocker"` DQ, do NOT push.

## 3. Required reading

Per §2.3 of `.claude/rules/advisor-orchestrator.md`, mandatory file-class lesson injection:

- `.claude/lessons/feedback_dq_v3_append_via_helper_script.md` — for the validate-pending entry.
- `.claude/lessons/feedback_falsifiable_hypothesis_before_structural_fix.md` — anchor-verify ALL three Edits via rg before writing.

CR finding context:
- cr-new-2 (`#3321737411`, Major): the fail-closed framing.
- cr-new-1 (`#3321685668`, Minor): the timestamp placeholder.
- Copilot `#3321383511`: duplicate of cr-new-2 framing (same shell-injection-adjacent concern).

## 4. Constraints

### 4.1 File-ownership

- ALLOWED: `scripts/brehon/dq-lint-durations.sh`, `.claude/PRPs/plans/v1-quality-r2a.plan.md`, `.claude/PRPs/debug/v1-quality-r2a-fix-impl-1-c3-blocker.json`, `.claude/decision-queue.json`.
- FORBIDDEN: anything else. Specifically no edits to `scripts/brehon/precheck.sh`, no edits to `crates/`, no new tests under `crates/server/tests/`. Zero scope creep.

### 4.2 Worker self-tests in §2.5

Run all 11 BEFORE `git add`. The NEGATIVE gate (step 9) is the policy-correctness signal — without it, this fix may regress to silent-skip if future maintenance touches `parse()`. Any failure → raise `kind: "blocker"` DQ; do NOT push.

### 4.3 validate-pending-laptop DQ entry at end

After committing the Edits, raise a `kind: "validate-pending-laptop"` DQ entry:

- `kind`: `"validate-pending-laptop"`
- `from`: `"impl"`
- `branch`: `"phase-v1-quality-r2"`
- `phase_task`: `"v1-quality-r2a-fix-impl-3"`
- `commands`: `["bash scripts/brehon/dq-lint-durations.sh", "bash scripts/brehon/precheck.sh"]` (NO cargo — zero Rust)
- `result`: `null`
- `log_slice`: `null`
- `failed_commands`: `null`

Generate composite v3 id via `bash scripts/brehon/dq-v3-new-entry.sh`. Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment> --pending`.

### 4.4 Commit subjects

TWO commits, in order:

1. `feat(quality): fix-impl-3 — fail-closed timestamp parsing + plan + debug-json — closes cr-new-1,cr-new-2`
2. `chore(decision-queue): impl raised validate-pending-laptop for v1-quality-r2a fix-impl-3`

Per `.claude/rules/decision-queue.md` Attribution integrity. Body for commit 1 ends with HANDOVER trailer:

```yaml
HANDOVER:
  task: v1-quality-r2a-fix-impl-3
  filesCreated: []
  filesModified:
    - scripts/brehon/dq-lint-durations.sh
    - .claude/PRPs/plans/v1-quality-r2a.plan.md
    - .claude/PRPs/debug/v1-quality-r2a-fix-impl-1-c3-blocker.json
    - .claude/decision-queue.json
  keyDecisions:
    - "User option-1 2026-05-29: fail-closed semantics per cr-new-2; reverses cr-6's silent-skip framing because the lint is a data-quality gate, not informational"
    - "Negative-test gate (§2.5 step 9) added to lock fail-closed behaviour against future regression"
    - "Timestamp on blocker JSON fragment updated to 8621a84bb commit time per cr-new-1"
  notes: "All three Edits are anchor-verified at brief-author time. After laptop validate-pending mutates to pass, PR #161 is ready for bm-poll-cr re-poll → addressed_in updates for cr-new-1, cr-new-2 → bm-merge."
```

### 4.5 No cargo

Zero Rust. No `cargo check`, no `cargo clippy`, no `cargo test`. validate-pending in §4.3 lists ONLY bash gates.

### 4.6 Push discipline

After both commits, `git push origin <worker-branch>`. Standard impl-task push pattern.
