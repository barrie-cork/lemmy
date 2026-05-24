# Brief: fix-impl-1 — v1-RT-r2 CR findings

## 1. Role + dispatch

`[role:impl-task] v1-RT-r2 fix-impl-1 — CR fix-in-pr findings — see .claude/PRPs/briefs/v1-RT-r2-fix-impl-1.md`

## 2. Scope

Fix 4 CR findings from PR #150 (v1-RT-r2). All are mechanical — no design decisions required.

### 2.1 Fix 1 — `clamp_dimension_i32` panic on inverted bounds (CR-1)

**File:** `crates/api/api/src/governance/reputation_snapshot.rs`

**Location:** `fn clamp_dimension_i32` (~line 969)

**Current:**
```rust
fn clamp_dimension_i32(value: i32, floor: i64, ceiling: i64) -> i32 {
  let widened = i64::from(value);
  let clamped = widened.clamp(floor, ceiling);
  i32::try_from(clamped).unwrap_or({
    if clamped > 0 { i32::MAX } else { i32::MIN }
  })
}
```

**Fix:** Add bounds normalization before clamp call; also correct the doc comment (it claims i64 prevents overflow-before-clamp, but sums accumulate in i32 so this is inaccurate — narrow the claim).

**Required result:**
```rust
/// Clamp a per-dimension i32 sum to the (floor, ceiling) bounds read as
/// i64 from governance_config. Normalises inverted bounds (floor > ceiling)
/// by swapping, so a misconfigured admin setting cannot panic at runtime.
fn clamp_dimension_i32(value: i32, floor: i64, ceiling: i64) -> i32 {
  let (floor, ceiling) = (floor.min(ceiling), floor.max(ceiling));
  let widened = i64::from(value);
  let clamped = widened.clamp(floor, ceiling);
  i32::try_from(clamped).unwrap_or({
    if clamped > 0 { i32::MAX } else { i32::MIN }
  })
}
```

**Unit test:** The existing `clamp_dimension_i32_within_bounds`, `clamp_dimension_i32_above_ceiling`, `clamp_dimension_i32_below_floor`, `clamp_dimension_i32_negative_below_zero_floor` tests must still pass. Add one new test:

```rust
#[test]
fn clamp_dimension_i32_inverted_bounds_no_panic() {
  // floor > ceiling — should not panic; behaves as if bounds were swapped
  let result = clamp_dimension_i32(50, 100, 0); // floor=100, ceiling=0 → normalised to [0,100]
  assert_eq!(result, 50); // 50 is within [0, 100]
}
```

### 2.2 Fix 2 — Duplicate DQ id `8aca794fb044-001` (CR-2)

**File:** `.claude/decision-queue.json`

**Problem:** Two entries in `resolved[]` share `id: "8aca794fb044-001"`. The second (duplicate) entry has `log_slice: null` and `resolved_at: "2026-05-23T14:00:00Z"`. The first entry has the richer `log_slice` and `resolved_at: "2026-05-23T13:04:47Z"`.

**Fix:** Remove the second duplicate entry entirely. The first entry (with `log_slice` populated and `resolved_at: "2026-05-23T13:04:47Z"`) is the canonical one — keep it unchanged.

After removal, verify with:
```bash
python3 -c "import json; dq=json.load(open('.claude/decision-queue.json',encoding='utf-8')); ids=[e['id'] for e in dq['resolved']]; assert len(ids)==len(set(ids)), 'duplicate ids'; print('OK, no duplicate ids')"
```

### 2.3 Fix 3 — Inverted chronology on `b246616aaf8f-001` (CR-3)

**File:** `.claude/decision-queue.json`

**Problem:** Entry `b246616aaf8f-001` has `timestamp: "2026-05-23T19:30:00Z"` but `resolved_at: "2026-05-23T18:31:54Z"` — resolved_at is earlier than timestamp, which is chronologically impossible.

**Fix:** Set `resolved_at` to `"2026-05-23T19:45:00Z"` (after the timestamp, consistent with the verify-report timestamp 2026-05-23T19:45:00Z which is when this DQ was mutated).

### 2.4 Fix 4 — Markdown lint nits (CR-4, CR-5)

**File:** `.claude/runlog/bm-runlog.md`

Add blank lines before and after the two new headings (lines ~1988 and ~1994). The headings must be surrounded by blank lines to satisfy MD022.

**File:** `.claude/PRPs/briefs/v1-RT-r2-bm-pr-1.md`

Change the opening fence of the PR body code block from ` ``` ` to ` ```md ` to satisfy MD040.

## 3. Required reading

- `.claude/lessons/feedback_clippy_test_style.md` — clippy test conventions
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — re-run clippy after any change

## 4. Constraints

- Touch ONLY the 4 files listed: `reputation_snapshot.rs`, `decision-queue.json`, `bm-runlog.md`, `bm-pr-1.md`
- Do NOT touch `crates/**` except `reputation_snapshot.rs`
- Do NOT edit any other test — only add `clamp_dimension_i32_inverted_bounds_no_panic`
- After all edits, run:
  1. `cargo check --workspace --features full`
  2. `cargo clippy --workspace --features full --no-deps -- -D warnings`
  3. `cargo test --workspace --features full --lib -- reputation_snapshot::tests`
  All three must exit 0 before committing.
- e2e runs on **laptop only** — after cargo-check/clippy/unit-tests pass, write `kind: "validate-pending-laptop"` DQ entry (commands array = the 3 validate commands above, branch = phase-v1-RT-r2, phase_task = "fix-impl-1") and **stop**. Do NOT run e2e.
- Commit subject: `fix(rep-tuning): CR fix-in-pr — clamp bounds guard + DQ dedup + md lint (v1-RT-r2)`
- Commit ALL 4 files in a single commit
- After committing, write the validate-pending DQ entry, then push both the commit and the DQ push to `phase-v1-RT-r2`
