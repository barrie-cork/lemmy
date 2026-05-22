---
role: impl-task
phase: v1-federation-inbound-d
brief_n: fix-in-pr-1
kind: fix-in-pr
---

# [role:impl-task] v1-federation-inbound-d fix-in-pr-1 — hold single mutex guard through test loop

## 1. Role + dispatch

`[role:impl-task]` Fix CodeRabbit finding cr-1 on PR #146: test `per_actor_map_evicts_oldest_when_cap_reached` re-acquires the mutex on every loop iteration, allowing test interleave. Acquire one guard before the loop and hold it through the loop body, assertions, and `clear()`.

## 2. Scope

### 2.1 CR finding (verbatim)

From CR run 69d3d75c, `publish_trust_attestation.rs` around line 435-479:

> The test repeatedly locks and unlocks the global map via
> `rate_per_actor_counts().lock().unwrap_or_else(PoisonError::into_inner)`, allowing
> other tests to interleave; instead acquire a single mutable guard (e.g.,
> `let mut counts = rate_per_actor_counts().lock().unwrap_or_else(PoisonError::into_inner)`)
> before the insertion loop and hold it through the for loop, the assertions and
> the final `counts.clear()` so the map is not unlocked/re-locked mid-test; update
> references that currently re-lock to use that single counts guard and remove the
> extra lock calls.

### 2.2 What to change

File: `crates/apub/activities/src/governance/publish_trust_attestation.rs`

Function: `tests_per_actor_bound::per_actor_map_evicts_oldest_when_cap_reached` (approx. lines 421-481)

**Current structure (4 separate lock acquisitions):**
1. `{let mut counts = ... .lock()...; counts.clear(); }` — pre-test clear, drops guard
2. `for i in 0..=cap { let mut counts = ... .lock()...; ... }` — re-acquires each iteration
3. `{ let counts = ... .lock()...; assert_eq!(...); assert!(...); }` — re-acquires for assertions
4. `let mut counts = ... .lock()...; counts.clear();` — re-acquires for cleanup

**Target structure (single guard from step 2 onward):**

```rust
#[test]
fn per_actor_map_evicts_oldest_when_cap_reached() {
  // Acquire a single guard and hold it through the entire test body to prevent
  // interleaving from concurrent tests in the same binary.
  let mut counts = rate_per_actor_counts()
    .lock()
    .unwrap_or_else(PoisonError::into_inner);

  // Clear stale state from prior tests (the OnceLock is process-global).
  counts.clear();

  let bucket = current_hour_bucket();
  let cap = MAX_PER_ACTOR_RATE_ENTRIES;

  // Insert `cap + 1` distinct keys, applying the same bound logic that
  // ships in `check_per_actor_rate_limit` (Task 1).
  for i in 0..=cap {
    let key = (format!("https://test/{i}"), bucket);
    counts.retain(|(_, b), _| *b >= bucket - 1);
    if counts.len() >= cap
      && !counts.contains_key(&key)
      && let Some(oldest_key) = counts
        .iter()
        .min_by_key(|((_, b), _)| *b)
        .map(|(k, _)| k.clone())
    {
      counts.remove(&oldest_key);
    }
    let entry = counts.entry(key).or_insert(0);
    *entry = entry.saturating_add(1);
  }

  // Assert: map size capped at MAX_PER_ACTOR_RATE_ENTRIES AND the trigger key
  // (i=cap, the key that forced the eviction) is present.
  // Note: which specific prior key gets evicted is not guaranteed — the eviction
  // uses min_by_key on the bucket value, and when all keys share the same bucket
  // (as in this test), HashMap iteration order is unspecified.
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

  // Cleanup: clear the map so other tests in this binary start fresh.
  counts.clear();
}
```

**Edit cap: exactly 1 file, ~20 lines changed (restructure existing lines).**

Do NOT change:
- The production code logic (`check_per_actor_rate_limit`, `MAX_PER_ACTOR_RATE_ENTRIES`, etc.)
- Any imports or `use` statements
- The test module name `tests_per_actor_bound`
- The test function name `per_actor_map_evicts_oldest_when_cap_reached`
- Any comments on the non-test code

## 3. Required reading

- Plan: `.claude/PRPs/plans/v1-federation-inbound-d.plan.md` (§3 for test conventions)
- `.claude/lessons/feedback_clippy_test_style.md` — workspace lints forbid `unwrap`/`expect`; use `PoisonError::into_inner` for mutex poison recovery
- Mirror refs: the existing test at lines 404-481 in `publish_trust_attestation.rs`

## 4. Constraints

1. **Pre-push gate (mandatory):** run `bash scripts/brehon/cargo-check.sh --workspace --features full` before pushing. Non-zero exit → fix in same commit. Never `#[allow]`-spam.
2. **No DQ entries needed** for this task — the fix is mechanical and the scope is exact.
3. **Commit subject format:** `fix(fed-in-d): hold single mutex guard in per-actor-bound test (fix-in-pr cr-1)`
4. **One commit.** Do not split.
5. Push to `phase-v1-federation-inbound-d` (same branch as PR #146).
6. Do NOT open a new PR — this commit lands on the existing open PR #146.
7. Raise a `validate-pending-laptop` DQ entry after pushing per the plan §15 DoD. Commands: `["bash scripts/brehon/cargo-check.sh --workspace --features full"]` only — no e2e needed for a test restructure.

## 3a. Handover from prior cohort

(none — fix-in-pr task, no prior [P] cohort)
