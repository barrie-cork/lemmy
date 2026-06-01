# Brief: v1-redaction-r1 fix-impl-1 — 6 clippy lints in reputation_snapshot.rs

## 1. Role + dispatch line

`[role:impl-task] v1-redaction-r1 fix-impl-1 clippy lints — see .claude/PRPs/briefs/v1-redaction-r1-fix-impl-1.md`

## 2. Scope

Fix exactly 6 clippy lints in one file:

- `crates/api/api/src/governance/reputation_snapshot.rs`

**Do NOT touch any other file.** No e2e.rs edits. No migrations. No new files.
No changes to `redaction.rs`. No changes to any crate other than `lemmy_api`.

After fixing, write a `validate-pending-laptop` DQ entry with
`commands: ["./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings"]`,
commit + push, then **stop**. Do NOT run cargo yourself.

## 3. Required reading

**Mandatory lessons:**

- `.claude/lessons/feedback_clippy_test_style.md` — no `unwrap`/`expect`; use `?` or `.unwrap_or`
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — do not assume one-shot fix; re-run clippy after changes
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write DQ entry, commit, push, STOP

**MIRROR refs (read before editing):**

- `crates/api/api/src/governance/reputation_snapshot.rs:595-641` — the 6 lint sites (read this section first)

**Pre-locate anchors** (run before any Edit — each must return exactly `1`):

```bash
grep -c "map_or(false" crates/api/api/src/governance/reputation_snapshot.rs
# EXPECT: 1

grep -n "denominator as i64" crates/api/api/src/governance/reputation_snapshot.rs
# EXPECT: 1 hit at line ~618

grep -n "denom_i64) as i32" crates/api/api/src/governance/reputation_snapshot.rs
# EXPECT: 4 hits (lines ~626, 631, 636, 641) — each is a unique multi-line expression
```

## 4. Implementation

### Fix 1 — `map_or(false, ...)` → `is_some_and(...)` (line 601-602)

**Clippy error:** `unnecessary_map_or` — `map_or(false, |cid| ...)` is simplified to `is_some_and(|cid| ...)`

**File:** `crates/api/api/src/governance/reputation_snapshot.rs`

```rust
// OLD (lines 601-602):
      s.community_id
        .map_or(false, |cid| !banned_cids.contains(&cid))

// NEW:
      s.community_id
        .is_some_and(|cid| !banned_cids.contains(&cid))
```

### Fix 2 — `denominator as i64` → `i64::try_from` (line 618)

**Clippy error:** `as_conversions` — `usize as i64` is potentially dangerous (silent truncation on 32-bit; usize is platform-width).

**Context:** `denominator` is `contributing.len()` (type `usize`). On 64-bit platforms this fits, but clippy enforces explicit conversion.

```rust
// OLD (line 618):
  let denom_i64 = denominator as i64;

// NEW:
  let denom_i64 = i64::try_from(denominator).unwrap_or(i64::MAX);
```

### Fixes 3-6 — four `(... / denom_i64) as i32` → `i32::try_from` (lines 626, 631, 636, 641)

**Clippy error:** `as_conversions` — `i64 as i32` is potentially dangerous (silent truncation).

**Context:** These are equal-weighted integer means of `i32` values. The mean of `i32` values divided by a `usize` denominator fits in `i32` by construction, but clippy enforces explicit conversion.

Apply the same pattern to all four expressions:

```rust
// OLD pattern (applied to all 4 — reporting_accuracy, jury_reliability, participation_consistency, endorsement_strength):
  let <dim> = (contributing
    .iter()
    .map(|s| i64::from(s.<dim>))
    .sum::<i64>()
    / denom_i64) as i32;

// NEW pattern:
  let <dim> = i32::try_from(
    contributing
      .iter()
      .map(|s| i64::from(s.<dim>))
      .sum::<i64>()
      / denom_i64,
  )
  .unwrap_or(i32::MAX);
```

The four expressions are `reporting_accuracy` (line 622-626), `jury_reliability` (line 627-631), `participation_consistency` (line 632-636), `endorsement_strength` (line 637-641). Apply Fix 3-6 to each.

**GOTCHA:** Each old_string spans multiple lines. Use the full multi-line block as the Edit anchor to ensure uniqueness. For example, for `reporting_accuracy`:

```
old_string: "  let reporting_accuracy = (contributing\n    .iter()\n    .map(|s| i64::from(s.reporting_accuracy))\n    .sum::<i64>()\n    / denom_i64) as i32;"
```

Verify uniqueness with `grep -c "reporting_accuracy = (contributing" crates/api/api/src/governance/reputation_snapshot.rs` before using.

## 5. Validation (DO NOT RUN — write DQ entry and stop)

After making all 6 fixes and confirming they compile mentally:

Write a `validate-pending-laptop` DQ entry:

```json
{
  "kind": "validate-pending-laptop",
  "from": "impl",
  "commands": ["./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings"],
  "branch": "phase-v1-redaction-r1",
  "phase_task": "fix-impl-1"
}
```

Use `bash scripts/brehon/dq-v3-new-entry.sh` to generate the id. Use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` to append.

Commit with: `fix(reputation): clippy lints — map_or→is_some_and + as_conversions→try_from (fix-impl-1)`

Push to `origin/phase-v1-redaction-r1`, then **STOP**. The laptop advisor runs the clippy validation.

## 6. Mandatory lesson injections

- `feedback_clippy_test_style.md` — clippy deny style
- `feedback_clippy_rerun_after_fix.md` — re-run verification
- `feedback_validate_pending_laptop_write_then_stop.md` — stop after DQ write
