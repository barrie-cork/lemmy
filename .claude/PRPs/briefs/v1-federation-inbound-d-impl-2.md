# Brief: v1-federation-inbound-d impl Task 2 — unit test for per-actor bound

## 1. Role + dispatch

`[role:impl-task]` v1-fed-in-d-task2-unit-test — see `.claude/PRPs/briefs/v1-federation-inbound-d-impl-2.md`

## 2. Scope

### 2.1 What to produce

Append a `#[cfg(test)] mod tests_per_actor_bound { ... }` to the bottom of
`crates/apub/activities/src/governance/publish_trust_attestation.rs` (after
the file's final non-test item — the `build_trust_attestation_object_stub`
function). The test exercises the `MAX_PER_ACTOR_RATE_ENTRIES` cap behaviour
against the live `rate_per_actor_counts()` `OnceLock`.

**Exactly one file changed.** Exactly one commit:
`test(fed-in-d): unit test for per-actor rate-map bound (task 2)`.

### 2.2 Test module (verbatim — must match Task 1 shipped shape)

**CRITICAL:** Task 1 shipped a `if ... && let Some(oldest_key)` let-chain
(collapsible_if fix, commit `5d72c123f`). The test body MUST mirror that
exact shape, NOT the nested-if shape in the plan. Read
`publish_trust_attestation.rs:179-187` (the production bound block) BEFORE
writing the test body and mirror it verbatim.

Append after the final `}` of `build_trust_attestation_object_stub`:

```rust
#[cfg(test)]
mod tests_per_actor_bound {
  //! Pure-function tests for the per-actor rate-map insertion-order bound
  //! added per v1-federation-inbound-d plan §3. Exercises
  //! `MAX_PER_ACTOR_RATE_ENTRIES` cap behaviour against the live
  //! `rate_per_actor_counts()` `OnceLock` — clears the global at test start
  //! AND end so test order is not load-bearing across the apub-activities
  //! lib-test binary.
  //!
  //! No unwrap/expect per workspace lints — uses `PoisonError::into_inner`
  //! for Mutex-poison recovery (canonical pattern; see
  //! `publish_trust_attestation.rs:161` + `inbox.rs:547`).
  use super::MAX_PER_ACTOR_RATE_ENTRIES;
  use crate::governance::inbox::{current_hour_bucket, rate_per_actor_counts};
  use std::sync::PoisonError;

  #[test]
  fn per_actor_map_evicts_oldest_when_cap_reached() {
    // Clear stale state from prior tests (the OnceLock is process-global).
    {
      let mut counts = rate_per_actor_counts()
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
      counts.clear();
    }

    let bucket = current_hour_bucket();
    let cap = MAX_PER_ACTOR_RATE_ENTRIES;

    // Insert `cap + 1` distinct keys, applying the same bound logic that
    // ships in `check_per_actor_rate_limit` (Task 1).
    for i in 0..=cap {
      let key = (format!("https://test/{i}"), bucket);
      let mut counts = rate_per_actor_counts()
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
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

    // Cleanup: clear the map so other tests in this binary start fresh.
    let mut counts = rate_per_actor_counts()
      .lock()
      .unwrap_or_else(PoisonError::into_inner);
    counts.clear();
  }
}
```

### 2.3 What NOT to produce (hard scope boundaries)

- Do NOT edit any other file besides `publish_trust_attestation.rs`.
- Do NOT add `serial_test` or any other dev-dependency to `Cargo.toml`.
- Do NOT add `#[allow(...)]` annotations.
- Do NOT add `.unwrap()` or `.expect()` — use `PoisonError::into_inner`.
- Do NOT run `cargo test --test e2e` (only `--no-run` for link check).
- Do NOT create new files; do NOT edit any file outside `publish_trust_attestation.rs`.
- Do NOT add a migration.

### 2.4 Post-push DQ entry (validate-pending-laptop)

After `git push origin <worker-branch>`, write a `kind: "validate-pending-laptop"` entry
to `.claude/decision-queue.json` with:

```json
{
  "kind": "validate-pending-laptop",
  "from": "impl",
  "branch": "<worker-branch>",
  "phase_task": 2,
  "commands": [
    "cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-d-task2-check.log 2>&1\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-d-task2-clippy.log 2>&1\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-test.bat -p lemmy_apub_activities --lib > .claude/PRPs/debug/v1-federation-inbound-d-task2-libtest.log 2>&1\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-federation-inbound-d-task2-e2e-norun.log 2>&1\""
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null
}
```

Use `bash scripts/brehon/dq-v3-new-entry.sh` to generate the composite id. Commit + push the DQ entry immediately after writing it.

## 3. Required reading (READ before any edit)

### 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: 5d72c123f
    filesCreated: []
    filesModified:
      - crates/apub/activities/src/governance/publish_trust_attestation.rs
    keyDecisions:
      - "Const MAX_PER_ACTOR_RATE_ENTRIES = 10_000 inserted after use block (line 30), before #[async_trait] (line 32)"
      - "Eviction block uses let-chain: if counts.len() >= MAX && !contains_key(&key) && let Some(oldest_key) = ... { remove } (clippy::collapsible_if fix applied)"
      - "let key = (subject_url.to_string(), bucket) binding shared between contains_key check and entry() insert"
    notes: "cargo-check PASS (8m 33s). clippy initially FAIL (collapsible_if), fixed in commit 5d72c123f using Rust let-chain syntax, then PASS (3m 05s). Validate-pending-laptop DQ faa4577b3cec-001 result: pass."
```

### 3b. Mandatory reads

1. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:1-40`** — confirm const at line 38, use block, `#[async_trait]` attribute location.
2. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:170-200`** — READ the EXACT eviction block that Task 1 shipped; mirror its let-chain shape verbatim in the test body.
3. **`crates/apub/activities/src/governance/publish_trust_attestation.rs` tail (last 30 lines)** — locate the anchor (`build_trust_attestation_object_stub` final `}`) after which to append the test module.
4. **`crates/apub/activities/src/governance/inbox.rs:464-479`** — `rate_per_actor_counts`, `current_hour_bucket` signatures (imports for test module).
5. **`crates/apub/activities/src/governance/publish_sanction_notice.rs:612-666`** — MIRROR ref: canonical crate-internal `#[cfg(test)] mod tests` sibling; match surface: `//!` docstring, `use super::...`, `#[test] fn ...()`.
6. **`.claude/lessons/feedback_clippy_test_style.md`** — workspace lint discipline; no `.unwrap()`/`.expect()`; `PoisonError::into_inner` pattern.
7. **`.claude/lessons/feedback_features_full_workspace_only.md`** — always `--workspace --features full` for check + clippy; lib-test uses `-p lemmy_apub_activities --lib` (no `--features full` for lib test).
8. **`.claude/lessons/feedback_explicit_file_arrays_on_tasks.md`** — FILES YAML; no `[P]` (serial task).

### 3c. Conformance-audit status

Inherited from Task 1 (same file): **0 Tier-1, 0 Tier-2**. Task 2 appends a `#[cfg(test)]` module only — no new production functions. No re-audit required.

## 4. Constraints

1. **Commit subject exactly:** `test(fed-in-d): unit test for per-actor rate-map bound (task 2)`
2. **One commit, one file changed.** `git diff --stat` MUST show: `1 file changed, ~80 insertions(+), 0 deletions(-)`.
3. **Pre-push `cargo check` gate**: run `bash scripts/brehon/cargo-check.sh --workspace --features full` (Linux) before `git push`. Non-zero exit → fix in same commit (if in-scope) OR file `kind: "blocker"` DQ (if out-of-scope).
4. **Test body MUST mirror Task 1 let-chain shape**: read `publish_trust_attestation.rs:179-187` first; use `if counts.len() >= cap && !counts.contains_key(&key) && let Some(oldest_key) = ...` — NOT the nested-if shape in the plan (Task 1 applied the collapsible_if fix).
5. **DQ mid-task push discipline**: commit + push the `validate-pending-laptop` DQ entry immediately after the worker-branch push.
6. **Shape G SUSPENDED**: write `kind: "validate-pending-laptop"` only (4 commands: check + clippy + lib-test + e2e --no-run).
7. **No `#[allow(...)]` additions** unless already present in the file.
8. **GOTCHA — OnceLock is process-global**: always `clear()` the map at test START and END. Test order must not be load-bearing.
9. **GOTCHA — lib-test command**: `cargo-test.bat -p lemmy_apub_activities --lib` (no `--features full` for lib test; lib test binary doesn't gate on the full feature).
10. **No new file** under `crates/apub/activities/src/governance/`. No migration.

## 5. FILES YAML (machine-parseable)

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/publish_trust_attestation.rs
requires:
  - task: 1
    reason: "Task 2 imports MAX_PER_ACTOR_RATE_ENTRIES from the same file and mirrors the bound logic Task 1 ships. Task 1 must be on the phase branch before Task 2 dispatches."
```
