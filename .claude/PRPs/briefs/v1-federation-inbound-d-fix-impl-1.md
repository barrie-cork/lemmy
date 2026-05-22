# Brief: v1-federation-inbound-d fix-impl Task 1 — fix overly-specific eviction assertion in per-actor bound test

## 1. Role + dispatch

`[role:impl-task]` v1-fed-in-d-fix-impl-1-test-assertion — see `.claude/PRPs/briefs/v1-federation-inbound-d-fix-impl-1.md`

## 2. Scope

### 2.1 §G4 CANONICAL RECIPE

This fix is **not** an allowlist match from advisor-orchestrator.md §G4 — it is a test contract mismatch caught by the validate-pending-laptop lib-test run. The fix recipe below is hand-authored from the RCA.

**Failure:** `per_actor_map_evicts_oldest_when_cap_reached` panicked at `publish_trust_attestation.rs:465`:
```
oldest inserted key (i=0) must be evicted by the bound
```

**Root cause:** All `cap + 1` keys inserted by the test share the same `bucket` value (`current_hour_bucket()`). The production eviction does `min_by_key(|((_, b), _)| *b)` — since every bucket is identical, `min_by_key` returns an **arbitrary** HashMap entry. The test asserts `i=0` is evicted, but the code only guarantees oldest-bucket eviction; when all entries share the same bucket, the evicted key is unspecified. **The production bound logic is correct.** Only the second assertion is wrong.

### 2.2 What to produce (Option B)

In `crates/apub/activities/src/governance/publish_trust_attestation.rs`, inside `tests_per_actor_bound::per_actor_map_evicts_oldest_when_cap_reached`, **replace** the second `assert!` block (the one that asserts `i=0` is absent) with two weaker but correct assertions:

1. Assert the map is still bounded at `cap` (keep the existing `assert_eq!(counts.len(), cap, ...)` — this one is correct and must stay).
2. Assert that the **trigger key** (`i=cap`, the last key inserted, the one that forced the eviction) IS present in the map — the code always inserts the new key after the eviction, so this is guaranteed.
3. **Remove** the assertion `assert!(!counts.contains_key(&first_key), "oldest inserted key (i=0) must be evicted ...")` entirely — replace it with nothing, or with a comment explaining why FIFO ordering is not guaranteed.

**Exact replacement target** (lines 454–469 of current file):

```rust
    // Assert: map size capped at MAX_PER_ACTOR_RATE_ENTRIES.
    {
      let counts = rate_per_actor_counts()
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
      assert_eq!(
        counts.len(),
        cap,
        "per-actor map must be bounded at MAX_PER_ACTOR_RATE_ENTRIES after cap + 1 inserts",
      );
      let first_key = (String::from("https://test/0"), bucket);
      assert!(
        !counts.contains_key(&first_key),
        "oldest inserted key (i=0) must be evicted by the bound",
      );
    }
```

**Replace with:**

```rust
    // Assert: map size capped at MAX_PER_ACTOR_RATE_ENTRIES AND the trigger key
    // (i=cap, the key that forced the eviction) is present.
    // Note: which specific prior key gets evicted is not guaranteed — the eviction
    // uses min_by_key on the bucket value, and when all keys share the same bucket
    // (as in this test), HashMap iteration order is unspecified.
    {
      let counts = rate_per_actor_counts()
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
      assert_eq!(
        counts.len(),
        cap,
        "per-actor map must be bounded at MAX_PER_ACTOR_RATE_ENTRIES after cap + 1 inserts",
      );
      let trigger_key = (format!("https://test/{cap}"), bucket);
      assert!(
        counts.contains_key(&trigger_key),
        "trigger key (i=cap) must be present after insertion-order-bound eviction",
      );
    }
```

**Exactly one file changed.** Exactly one commit:
`test(fed-in-d): fix overly-specific eviction assertion in per-actor bound test (task 2 fix)`

### 2.3 What NOT to produce

- Do NOT edit any other file.
- Do NOT add `#[allow(...)]` annotations.
- Do NOT add `.unwrap()` or `.expect()`.
- Do NOT change the first `assert_eq!(counts.len(), cap, ...)` assertion — it is correct.
- Do NOT change the production eviction logic in `check_per_actor_rate_limit`.
- Do NOT add a migration.
- Do NOT run `cargo test --test e2e` (only `--no-run` for link check).

### 2.4 Post-push DQ entry (validate-pending-laptop)

After `git push origin <worker-branch>`, write a `kind: "validate-pending-laptop"` entry
to `.claude/decision-queue.json` with:

```json
{
  "id": "817043cf3f31-001",
  "kind": "validate-pending-laptop",
  "from": "impl",
  "branch": "<worker-branch>",
  "phase_task": "2-fix",
  "commands": [
    "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-d-task2-fix-clippy.log 2>&1\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-test.bat -p lemmy_apub_activities --lib > .claude/PRPs/debug/v1-federation-inbound-d-task2-fix-libtest.log 2>&1\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-federation-inbound-d-task2-fix-e2e-norun.log 2>&1\""
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null
}
```

Use `bash scripts/brehon/dq-v3-new-entry.sh` to confirm the id matches `817043cf3f31-001`. Commit + push the DQ entry immediately after writing it.

## 3. Required reading (READ before any edit)

### 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 2
    commit: "(see git log for Task 2 commit on worker branch)"
    filesCreated: []
    filesModified:
      - crates/apub/activities/src/governance/publish_trust_attestation.rs
    keyDecisions:
      - "Test module appended at bottom of file (after build_trust_attestation_object_stub)"
      - "Test uses let-chain shape matching Task 1 production code (collapsible_if fix)"
      - "cargo-check PASS, cargo-clippy PASS, lib-test FAIL at assertion on i=0 eviction"
    notes: "cargo-check PASS (1m 31s), clippy PASS (2m 54s), lib-test FAIL: per_actor_map_evicts_oldest_when_cap_reached panicked at rs:465 — all keys share same bucket so min_by_key evicts arbitrary entry, not i=0. Production code is correct; test assertion is overly-specific. Fix: replace i=0-absent assertion with i=cap-present assertion."
```

### 3b. Mandatory reads

1. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:420-477`** — READ the full test module as it currently exists; understand the failing assertion at line 465 and why it is wrong.
2. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:179-187`** — READ the production eviction block; confirm `min_by_key` uses bucket as the sort key.
3. **`.claude/lessons/feedback_clippy_test_style.md`** — workspace lint discipline; no `.unwrap()`/`.expect()`; `PoisonError::into_inner` pattern.
4. **`.claude/lessons/feedback_features_full_workspace_only.md`** — always `--workspace --features full` for check + clippy; lib-test uses `-p lemmy_apub_activities --lib` (no `--features full`).

### 3c. Conformance-audit status

No new production functions — test-module edit only. No re-audit required.

## 4. Constraints

1. **Commit subject exactly:** `test(fed-in-d): fix overly-specific eviction assertion in per-actor bound test (task 2 fix)`
2. **One commit, one file changed.**
3. **Pre-push `cargo clippy` gate**: run `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"` before `git push`. Non-zero exit → fix in same commit.
4. **Pre-push lib-test gate**: run `cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_apub_activities --lib"` before `git push`. The new test MUST pass.
5. **DQ mid-task push discipline**: commit + push the `validate-pending-laptop` DQ entry immediately after the worker-branch push.
6. **No `#[allow(...)]` additions**.
7. **No production code changes** — only the test assertion block changes.

## 5. FILES YAML (machine-parseable)

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/publish_trust_attestation.rs
requires:
  - task: 2
    reason: "Fix amends the Task 2 test. Task 2 commit must be on the phase branch (or worker branch) before this fix dispatches."
```
