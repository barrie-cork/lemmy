# v1-redaction-r1 impl-task 2 — bounded `scrub_json` recursion (MAX_RECURSION_DEPTH=64)

## 1. Dispatch

```
[role:impl-task] v1-redaction-r1 task 2 — bounded scrub_json recursion — see .claude/PRPs/briefs/v1-redaction-r1-impl-2.md
```

Branch the worker forks from: `phase-v1-redaction-r1`.

## 2. Scope

Implement Plan §13 Task 2 verbatim: rewrite `scrub_json` in
`crates/db_schema/src/source/governance/redaction.rs` to introduce a
file-private `scrub_json_inner(value, depth)` helper + module-level
`const MAX_RECURSION_DEPTH: usize = 64`. At `depth >= MAX_RECURSION_DEPTH`,
substitute `Value::Null` (over-scrub bias per ADR-015 + GDPR §17). Add 1
paired depth-cap test.

**Public signature of `pub fn scrub_json(value: &Value) -> Value` is byte-
identical to v0** — no caller updates, no re-export changes.

**Single file modified:**
- `crates/db_schema/src/source/governance/redaction.rs` —
  (a) insert module-level `const MAX_RECURSION_DEPTH` + doc, ABOVE the
      `/// Strip identifiers…` doc-comment at current line 61;
  (b) replace lines 88-101 (`scrub_json` body) with the §10.3 verbatim
      shape (1-line delegation `scrub_json(value: &Value) -> Value { scrub_json_inner(value, 0) }`
      + new file-private `scrub_json_inner(value: &Value, depth: usize) -> Value`);
  (c) append `scrub_json_at_depth_cap_returns_null_not_truncated_tree` test inside
      `#[cfg(test)] mod tests`, AFTER Task 1's 7 new tests, BEFORE the closing `}`.

### 2.1 §G4 CANONICAL RECIPE

N/A — this is a planned `impl-task`, not a `fix-impl-task` recovering from a §G4 classifier failure.

### 2.2 Verbatim §10.3 production code (paste over current lines 88-101 + insert above line 61)

**Insert this `const` block above current line 61 (the `/// Strip identifiers from…` doc-comment for `scrub`):**

```rust
/// Maximum recursion depth for `scrub_json`. Defence-in-depth against
/// adversarial / buggy upstream JSON trees. Well below stack-overflow
/// threshold (~10K on x86_64) and generous for legitimate use
/// (governance_log payloads are flat — 5-6 levels worst case).
///
/// Hard cap; over-scrub bias under GDPR §17 + ADR-015 — a truncated
/// tree looking complete to the consumer is worse than visibly-null
/// leaves.
const MAX_RECURSION_DEPTH: usize = 64;

```

**Replace current `scrub_json` body (lines 88-101) with:**

```rust
/// Recursively scrub every string value in a JSON tree.
///
/// Object **keys** are left intact because they are schema labels,
/// not user content. Values at every depth — strings, array elements,
/// object values — are passed through [`scrub`]. Non-string scalars
/// (numbers, booleans, nulls) are passed through unchanged.
///
/// Recursion bounded at [`MAX_RECURSION_DEPTH`] (64); deeper subtrees
/// substituted with [`Value::Null`] (over-scrub bias).
pub fn scrub_json(value: &Value) -> Value {
  scrub_json_inner(value, 0)
}

fn scrub_json_inner(value: &Value, depth: usize) -> Value {
  if depth >= MAX_RECURSION_DEPTH {
    return Value::Null;
  }
  match value {
    Value::String(s) => Value::String(scrub(s)),
    Value::Array(items) => Value::Array(
      items.iter().map(|v| scrub_json_inner(v, depth + 1)).collect(),
    ),
    Value::Object(map) => {
      let scrubbed = map
        .iter()
        .map(|(k, v)| (k.clone(), scrub_json_inner(v, depth + 1)))
        .collect();
      Value::Object(scrubbed)
    }
    other => other.clone(),
  }
}
```

### 2.3 Verbatim depth-cap test (append inside `#[cfg(test)] mod tests`, AFTER Task 1's `scrub_handle_with_unicode_confusable_in_username` `#[ignore]` test, BEFORE the closing `}`)

```rust
  #[test]
  fn scrub_json_at_depth_cap_returns_null_not_truncated_tree() {
    // Locks the WP-3 invariant: at recursion depth >= MAX_RECURSION_DEPTH,
    // the substitution is Value::Null (NOT the un-scrubbed leaf, NOT a
    // truncated subtree). Over-scrub bias per ADR-015.
    let mut tree = Value::String("user @alice email foo@example.com".into());
    for _ in 0..70 {
      tree = Value::Array(vec![tree]);
    }
    let scrubbed = scrub_json(&tree);

    // Walk down 64 levels of the scrubbed tree. At each step, expect an
    // Array of length 1; at level 64 the inner value must be Value::Null
    // (NOT the original string, NOT a partial-scrub representation).
    let mut cursor = &scrubbed;
    for level in 0..64 {
      match cursor {
        Value::Array(items) => {
          assert_eq!(items.len(), 1, "level {level} should be Array(1)");
          cursor = &items[0];
        }
        other => panic!("level {level} expected Array, got {other:?}"),
      }
    }
    // At level 64 (the depth-cap boundary), the inner value is Null.
    assert_eq!(
      cursor,
      &Value::Null,
      "at depth 64, the substitution must be Value::Null (over-scrub bias)"
    );
  }
```

### 2.4 Mechanical Edit anchors (locate before editing)

**Anchor A — const insertion point** (between `profile_url_regex` fn close `}` and `///` doc-comment for `scrub`):

```rust
//   ─── existing profile_url_regex fn closes here ───
}

/// Strip identifiers from a human-readable string.
```

Edit to:

```rust
//   ─── existing profile_url_regex fn closes here ───
}

/// Maximum recursion depth for `scrub_json`. Defence-in-depth against
/// adversarial / buggy upstream JSON trees. Well below stack-overflow
/// threshold (~10K on x86_64) and generous for legitimate use
/// (governance_log payloads are flat — 5-6 levels worst case).
///
/// Hard cap; over-scrub bias under GDPR §17 + ADR-015 — a truncated
/// tree looking complete to the consumer is worse than visibly-null
/// leaves.
const MAX_RECURSION_DEPTH: usize = 64;

/// Strip identifiers from a human-readable string.
```

**Anchor B — `scrub_json` body replacement** (matches verbatim current lines 82-101):

`old_string` (current file content):

```rust
/// Recursively scrub every string value in a JSON tree.
///
/// Object **keys** are left intact because they are schema labels, not
/// user content. Values at every depth — strings, array elements, object
/// values — are passed through [`scrub`]. Non-string scalars (numbers,
/// booleans, nulls) are passed through unchanged.
pub fn scrub_json(value: &Value) -> Value {
  match value {
    Value::String(s) => Value::String(scrub(s)),
    Value::Array(items) => Value::Array(items.iter().map(scrub_json).collect()),
    Value::Object(map) => {
      let scrubbed = map
        .iter()
        .map(|(k, v)| (k.clone(), scrub_json(v)))
        .collect();
      Value::Object(scrubbed)
    }
    other => other.clone(),
  }
}
```

`new_string`:

```rust
/// Recursively scrub every string value in a JSON tree.
///
/// Object **keys** are left intact because they are schema labels,
/// not user content. Values at every depth — strings, array elements,
/// object values — are passed through [`scrub`]. Non-string scalars
/// (numbers, booleans, nulls) are passed through unchanged.
///
/// Recursion bounded at [`MAX_RECURSION_DEPTH`] (64); deeper subtrees
/// substituted with [`Value::Null`] (over-scrub bias).
pub fn scrub_json(value: &Value) -> Value {
  scrub_json_inner(value, 0)
}

fn scrub_json_inner(value: &Value, depth: usize) -> Value {
  if depth >= MAX_RECURSION_DEPTH {
    return Value::Null;
  }
  match value {
    Value::String(s) => Value::String(scrub(s)),
    Value::Array(items) => Value::Array(
      items.iter().map(|v| scrub_json_inner(v, depth + 1)).collect(),
    ),
    Value::Object(map) => {
      let scrubbed = map
        .iter()
        .map(|(k, v)| (k.clone(), scrub_json_inner(v, depth + 1)))
        .collect();
      Value::Object(scrubbed)
    }
    other => other.clone(),
  }
}
```

**Anchor C — depth-cap test insertion point** (just BEFORE closing `}` of `mod tests`, after task 1's `scrub_handle_with_unicode_confusable_in_username` `#[ignore]` test):

`old_string`:

```rust
  #[test]
  #[ignore = "v1-redaction-r2 will normalise via unicode-normalization crate"]
  fn scrub_handle_with_unicode_confusable_in_username() {
    // TODO(v1-redaction-r2): Cyrillic 'а' inside the username slot
    // — current ASCII-only username pattern [A-Za-z0-9_\-]+ doesn't
    // match, so the mention is NOT scrubbed. Adversarial vector.
    assert_eq!(
      scrub("@аlice"),  // 'а' is Cyrillic U+0430
      "[redacted]"
    );
  }
}
```

`new_string`:

```rust
  #[test]
  #[ignore = "v1-redaction-r2 will normalise via unicode-normalization crate"]
  fn scrub_handle_with_unicode_confusable_in_username() {
    // TODO(v1-redaction-r2): Cyrillic 'а' inside the username slot
    // — current ASCII-only username pattern [A-Za-z0-9_\-]+ doesn't
    // match, so the mention is NOT scrubbed. Adversarial vector.
    assert_eq!(
      scrub("@аlice"),  // 'а' is Cyrillic U+0430
      "[redacted]"
    );
  }

  #[test]
  fn scrub_json_at_depth_cap_returns_null_not_truncated_tree() {
    // Locks the WP-3 invariant: at recursion depth >= MAX_RECURSION_DEPTH,
    // the substitution is Value::Null (NOT the un-scrubbed leaf, NOT a
    // truncated subtree). Over-scrub bias per ADR-015.
    let mut tree = Value::String("user @alice email foo@example.com".into());
    for _ in 0..70 {
      tree = Value::Array(vec![tree]);
    }
    let scrubbed = scrub_json(&tree);

    // Walk down 64 levels of the scrubbed tree. At each step, expect an
    // Array of length 1; at level 64 the inner value must be Value::Null
    // (NOT the original string, NOT a partial-scrub representation).
    let mut cursor = &scrubbed;
    for level in 0..64 {
      match cursor {
        Value::Array(items) => {
          assert_eq!(items.len(), 1, "level {level} should be Array(1)");
          cursor = &items[0];
        }
        other => panic!("level {level} expected Array, got {other:?}"),
      }
    }
    // At level 64 (the depth-cap boundary), the inner value is Null.
    assert_eq!(
      cursor,
      &Value::Null,
      "at depth 64, the substitution must be Value::Null (over-scrub bias)"
    );
  }
}
```

### 2.5 Public-API non-regression invariants (verify post-edit)

- `rg "fn scrub_json" crates/db_schema/src/source/governance/redaction.rs` returns **exactly 1 hit** (`pub fn scrub_json(...)` — `scrub_json_inner` does NOT have a `pub` prefix; matches `fn scrub_json_inner` separately).
- `rg "scrub_json_inner" crates/` returns **exactly 3 hits**: 1 definition + 2 recursive calls inside its own body (the `.map(|v| scrub_json_inner(v, depth + 1))` lines for Array + Object).
- `rg "MAX_RECURSION_DEPTH" crates/` returns **exactly 2 hits**: 1 `const` definition + 1 `if depth >= MAX_RECURSION_DEPTH` check.
- `rg "pub use crate::source::governance::redaction" crates/api/api/src/governance/redaction.rs` still hits — `scrub_json` re-export shim unchanged.

## 3. Required reading (mandatory)

### 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: e42cb4d70
    filesCreated: []
    filesModified:
      - crates/db_schema/src/source/governance/redaction.rs
    keyDecisions:
      - sibling-pattern mirror (plain #[test] fn, assert_eq!, no LemmyResult, no async)
      - 2 #[ignore] tests for Unicode confusables carry-forward to v1-redaction-r2
      - WP-2 active test uses Cyrillic U+0430 (not ASCII a) per brief §2.3 gotchas
    notes: |
      Task 1 added 7 tests (5 active + 2 #[ignore]) at the end of #[cfg(test)] mod tests.
      The closing `}` of mod tests is now at a NEW line number (post-Task-1 line numbering
      shifts). Anchor C's old_string above matches the final `}` literally — relies on the
      unique Cyrillic U+0430 test as the immediate predecessor.
      ALL 10 tests passed at advisor-laptop validate-pending-laptop @ 6cee24307 (DQ
      642f320508ca-002 → resolved 46083402a).
```

### 3b. Required lessons (file-class injection per advisor-orchestrator §2.4)

Mandatory reading; rules apply silently in the implementation:

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` (Case-A
  sibling-uniformity discipline: depth-cap test is plain `#[test]` fn returning
  `()`, mirrors the 11 existing tests in the same `mod tests`; no `LemmyResult`,
  no async, no `?`).
- `.claude/lessons/feedback_clippy_test_style.md` (no unwrap/expect; `assert_eq!`
  only; `pretty_assertions::assert_eq` re-export is already in scope via `use
  super::*` + `use pretty_assertions::assert_eq;` at lines 105-106).
- `.claude/lessons/feedback_clippy_doc_lazy_continuation_in_doc_comments.md`
  (the `MAX_RECURSION_DEPTH` doc-comment 7 lines uses blank `///` between
  paragraphs and starts continuation lines with NON-"and" words — see verbatim
  text §2.2; copy literally).
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` (not a fix-impl,
  but the spirit applies: `old_string` anchors in §2.4 MUST match byte-for-byte;
  pre-locate via Read tool BEFORE first Edit call).

### 3c. Plan references

- Plan §13 Task 2 (`.claude/PRPs/plans/v1-redaction-r1.plan.md` lines 1071-1158)
  — ACTION + FILES + IMPLEMENT (a)(b) + MIRROR + GOTCHA + VALIDATE.
- Plan §10.3 (same file, lines 532-624) — verbatim production code + verbatim
  test body. The brief §2.2/§2.3 above pastes from §10.3 directly.
- Plan §3.6 — `MAX_RECURSION_DEPTH = 64` rationale (well below stack-overflow,
  generous for legitimate use).
- ADR-015 (`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`)
  — pseudonym discipline + over-scrub bias rationale.

## 4. Constraints

### 4.1 File-ownership boundaries

- **EDIT ONLY:** `crates/db_schema/src/source/governance/redaction.rs`
  (single file, three Edit anchors in §2.4).
- **DO NOT edit** `crates/api/api/src/governance/redaction.rs` (re-export shim —
  unchanged because `pub fn scrub_json` signature stays byte-identical).
- **DO NOT edit** any other `.rs` file — there are zero call-site changes.

### 4.2 Worker-side validation gates (mandatory before push)

Run all 3 in sequence; all must exit 0. Use Linux-side cargo wrappers:

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full
echo "check exit: $?"
# EXPECT: exit 0

bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings
echo "clippy exit: $?"
# EXPECT: exit 0

bash scripts/brehon/cargo-test.sh -p lemmy_db_schema --features full redaction::tests
echo "test exit: $?"
# EXPECT: exit 0; "test result: ok. 11 passed; 0 failed; 2 ignored"
# (Task 1's 5 active + Task 2's 1 new active + 5 existing = 11 active; +2 ignored)
```

**Plan §13's VALIDATE block (line 1152) says "12 passed". The plan's count is
based on a pre-amendment §13 Task 1 description ("6 active + 2 #[ignore]")
which was a typo for the enumerated 5 active. Task 1 shipped 5 active + 2
ignored (verified at 6cee24307, validate-pending-laptop PASS at 46083402a). The
correct post-Task-2 count is `5 + 1 + 5 = 11 passed; 2 ignored`. Brief
overrides plan §13 line 1152.**

### 4.3 Mid-task push discipline

After cargo gates pass, in a single sequence (per
`feedback_phase_lane_worktree_bootstrap_checklist.md` + the cohort-shared
`.git/index.lock` discipline):

1. `git add crates/db_schema/src/source/governance/redaction.rs`
2. `git commit -m "feat(redaction): bounded scrub_json recursion + depth-cap test (task 2)"` (full body in §5)
3. `git push origin <worker-branch>`

Then write the kind: "validate-pending-laptop" DQ entry per
`feedback_laptop_default_for_validate_pending.md`. Use the dq-v3 helper:

```bash
# Generate composite id + author fragment, then append via helper.
bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending
```

Fragment JSON for this validate-pending-laptop entry:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 at push time>",
  "question": "Run cargo check + clippy + test (redaction::tests) on worker branch for v1-redaction-r1 Task 2 bounded scrub_json recursion",
  "options": ["pass", "fail"],
  "context": "One commit on worker branch (sha = <commit SHA>). Shape-G suspended per project_shape_g_suspended_2026_05_16.md; laptop runs final gates.",
  "commands": [
    "cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-redaction-r1-task2-check.log 2>&1\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-redaction-r1-task2-clippy.log 2>&1\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-test.bat -p lemmy_db_schema --features full redaction::tests > .claude/PRPs/debug/v1-redaction-r1-task2-test.log 2>&1\""
  ],
  "branch": "<worker-branch>",
  "phase_task": "v1-redaction-r1-task-2",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "approved_by": null,
  "approved_at": null
}
```

Then commit the DQ write:

```bash
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): impl raised validate-pending-laptop for v1-redaction-r1 task 2"
git push origin <worker-branch>
```

### 4.4 Refusals

- NEVER write `answered_by: "advisor"` (Hard refusal #1 — `decision-queue.md`).
- NEVER write `approved_by: <non-null>` (Hard refusal #8 — advisor-exclusive).
- NEVER use `next_id = max(all_ids)+1` (Hard refusal #9 — use the helper).
- NEVER edit `crates/api/api/src/governance/redaction.rs` (re-export shim;
  signature unchanged).
- NEVER widen `scrub_json_inner` visibility (it's file-private; the public
  surface is `scrub_json` alone).
- NEVER make `MAX_RECURSION_DEPTH` `pub` — file-private const.
- NEVER skip cargo gates before push.

## 5. Commit subject + body

**Subject:**

```
feat(redaction): bounded scrub_json recursion + depth-cap test (task 2)
```

**Body:**

```
Implements Plan §13 Task 2 (WP-3 / Issue #58 docstring invariant #4):
introduces a bounded-recursion helper `scrub_json_inner(value, depth)`
behind the unchanged `pub fn scrub_json(value: &Value) -> Value` and adds
1 paired depth-cap test.

Production changes in crates/db_schema/src/source/governance/redaction.rs:
- New module-level `const MAX_RECURSION_DEPTH: usize = 64` (inserted above
  the `scrub` fn doc-comment, with 7-line policy doc explaining hard-cap
  + over-scrub bias under GDPR §17 + ADR-015).
- `pub fn scrub_json` reduced to a 1-line delegate `scrub_json_inner(value, 0)`.
- New file-private `fn scrub_json_inner(value: &Value, depth: usize) -> Value`
  with depth check at top (substitutes `Value::Null` when `depth >=
  MAX_RECURSION_DEPTH`) and recursive `depth + 1` calls in Array + Object arms.
- `scrub_json` doc-comment expanded with 2-line "Recursion bounded at
  MAX_RECURSION_DEPTH (64); deeper subtrees substituted with Value::Null
  (over-scrub bias)" addendum.

Test added (in `#[cfg(test)] mod tests`, after task 1's
`scrub_handle_with_unicode_confusable_in_username` `#[ignore]` test):
- scrub_json_at_depth_cap_returns_null_not_truncated_tree — builds a
  70-deep `Value::Array` tree, walks 64 levels asserting `Array(1)` at
  each step, then asserts the leaf at level 64 is `Value::Null`.

Public API non-regression:
- `pub fn scrub_json(value: &Value) -> Value` signature byte-identical to v0.
- `crates/api/api/src/governance/redaction.rs` re-export shim unchanged.
- `rg "fn scrub_json" crates/db_schema/.../redaction.rs` = 1 hit (no `pub` widening).
- `rg "scrub_json_inner" crates/` = 3 hits (1 def + 2 recursive call sites).

Worker-side gates: cargo-check.sh --workspace --features full (exit 0),
cargo-clippy.sh --workspace --features full --no-deps -- -D warnings (exit 0),
cargo-test.sh -p lemmy_db_schema --features full redaction::tests (exit 0;
11 passed; 0 failed; 2 ignored).

Mandatory lessons injected per §2.3 brief:
- feedback_lemmy_error_no_std_error.md Case-A: plain #[test] fn, assert_eq!,
  no LemmyResult — mirrors canonical sibling shape verbatim.
- feedback_clippy_test_style.md: no unwrap/expect; assert_eq! only.
- feedback_clippy_doc_lazy_continuation_in_doc_comments.md: blank `///`
  between paragraphs in the new `const` doc-comment.

HANDOVER:
  filesCreated: []
  filesModified:
    - crates/db_schema/src/source/governance/redaction.rs
  keyDecisions:
    - public scrub_json signature byte-identical (1-line delegate)
    - scrub_json_inner file-private (no pub keyword); MAX_RECURSION_DEPTH module-private const
    - depth-cap substitution = Value::Null (over-scrub bias per ADR-015 + GDPR §17)
    - test asserts Array(1) at each level 0-63 then Null at level 64
  notes: |
    Brief §4.2 corrects plan §13 line 1152 test count: 11 active not 12.
    Task 1 shipped 5 active + 2 ignored (not 6 active per plan typo).
```

## 6. Acceptance criteria (advisor-side post-checks)

Advisor verifies after worker reports done:

1. Worker pushed exactly 2 commits: `feat(redaction): bounded scrub_json
   recursion + depth-cap test (task 2)` AND `chore(decision-queue): impl raised
   validate-pending-laptop for v1-redaction-r1 task 2`.
2. `git diff phase-v1-redaction-r1..<worker-branch> -- crates/db_schema/src/source/governance/redaction.rs`
   shows exactly: 7-line const doc + `const MAX_RECURSION_DEPTH: usize = 64;`,
   2-line `scrub_json` doc addendum, body replaced with 1-line delegate, new
   `scrub_json_inner` fn with depth check + recursive `depth + 1` calls,
   1 new test at end of `mod tests`.
3. No other files changed.
4. DQ entry exists with `kind: "validate-pending-laptop"`, `from: "impl"`,
   `commands` array of 3 Windows `cmd //c` lines, composite id from helper.
5. Advisor runs the 3 §4.2 cargo gates on the lane worktree
   `brehon-fork-redaction-r1`; expects all-pass.
6. Daemon false-success check: SSH `homeserver "cd /srv/brehon-fork && git log
   <worker-branch> --oneline -5"` after task reports done; if commits absent from
   origin, advisor manually `git push`s.
