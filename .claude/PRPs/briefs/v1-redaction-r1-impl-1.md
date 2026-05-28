# Brief: impl-task 1 — v1-redaction-r1 adversarial test corpus (WP-1, WP-2, WP-5, WP-7, WP-8)

**Role:** `[role:impl-task]`
**Phase:** `v1-redaction-r1`
**Authored:** 2026-05-28
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base branch (Junior forks from):** `phase-v1-redaction-r1`
**Lane mode:** Mode B (mobile remote-control). Junior worker runs on the EliteDesk daemon (Linux).

---

## 1. Role + dispatch

`[role:impl-task] v1-redaction-r1 task 1 — adversarial test corpus — see .claude/PRPs/briefs/v1-redaction-r1-impl-1.md`

---

## 2. Scope

Append **5 active + 2 `#[ignore]` adversarial unit tests** (7 total) to the existing `#[cfg(test)] mod tests` in `crates/db_schema/src/source/governance/redaction.rs`. Production code (lines 1–101) is **unchanged**. Existing `mod tests` already contains 5 tests at lines 109–158; this task appends the new tests BEFORE the closing `}` at line 159, AFTER the existing `scrub_preserves_keys_and_non_string_scalars` test ending at line 158.

> **Note on plan-side header count:** plan §13 Task 1 lead says "6 active + 2 `#[ignore]`" but plan §4.1 Surface 1 enumerates exactly 5 active + 2 `#[ignore]` test names (verified in plan lines 144–158). This brief follows the enumerated list (the canonical source); the §13 lead "6" is a plan-side typo. Total after task: 5 existing + 5 new active + 2 `#[ignore]` = 12 tests; 10 run, 2 ignored.

**Files edited (exactly one):**
- `crates/db_schema/src/source/governance/redaction.rs` — append tests inside `mod tests`. **NO other files.**

**No `LemmyResult<()>` outer; no async; no helpers.** Plain `#[test] fn`, `assert_eq!` (the `pretty_assertions::assert_eq` import at line 106 supplies the macro). The existing `use super::*;` (line 105) and `use serde_json::json;` (line 107) cover all imports for the new tests.

### 2.1 Canonical sibling pattern (MIRROR — read first)

The 5 existing tests at `redaction.rs:109-158` ARE the canonical shape. Read the file at the phase-branch tip BEFORE authoring the new tests:

```bash
git show HEAD:crates/db_schema/src/source/governance/redaction.rs | sed -n '103,159p'
```

Confirm: plain `#[test] fn`; bodies use `assert_eq!`; one test (`scrub_json_walks_nested_structure`) uses `json!(...)` directly without an additional `use` (the `serde_json::json;` import at line 107 covers it). **Mirror this shape verbatim — do not introduce `Result<T, E>` returns, do not wrap in async, do not add `?` propagation.** Per `feedback_lemmy_error_no_std_error.md` Case-A discipline (sibling-uniformity over local cleverness).

### 2.2 Test bodies (verbatim from plan §13 Task 1 + §10.1 + §10.2)

Append these to `mod tests`, in this order, after the existing `scrub_preserves_keys_and_non_string_scalars` test:

**Test 1 — `scrub_order_dependence_mention_with_remote_host_not_eaten_by_email`** (WP-1 / H1; verbatim from plan §10.1):

```rust
#[test]
fn scrub_order_dependence_mention_with_remote_host_not_eaten_by_email() {
  // Locks docstring invariant at redaction.rs:66-73.
  // If someone re-orders the 3 replace_all calls in `scrub`, this fails:
  // mention regex must run before email regex so @bob@remote.example
  // is replaced as a whole unit, not as @[redacted] (orphan @).
  assert_eq!(
    scrub("hi @bob@remote.example see you"),
    "hi [redacted] see you"
  );
}
```

**Test 2 — `scrub_handles_newline_separated_mentions`** (WP-7):

```rust
#[test]
fn scrub_handles_newline_separated_mentions() {
  assert_eq!(
    scrub("@alice\n@bob\n@carol"),
    "[redacted]\n[redacted]\n[redacted]"
  );
}
```

**Test 3 — `scrub_does_not_treat_username_as_regex_pattern`** (WP-8):

```rust
#[test]
fn scrub_does_not_treat_username_as_regex_pattern() {
  // Username class is restrictive [A-Za-z0-9_-]+, so adversarial
  // patterns like @.*, @[abc], @(group) don't match — they're
  // literal text, not matched as mentions. Verifies the regex
  // engine isn't tricked into treating username content as a
  // pattern.
  assert_eq!(
    scrub("ok @abc bad @.* worse @[xyz]"),
    "ok [redacted] bad @.* worse @[xyz]"
  );
}
```

**Test 4 — `scrub_json_preserves_integer_id_fields`** (WP-5):

```rust
#[test]
fn scrub_json_preserves_integer_id_fields() {
  // Confirms scrub_json passes integer scalars through unchanged
  // (issue #58 hint: "preserve schema-typed fields, scrub only
  // string identifiers"). Pseudonyms ARE strings but opaque-by-
  // construction; the scrubber regex doesn't match them.
  let input = json!({"community_id": 42, "actor_pseudonym": "abc123def"});
  let result = scrub_json(&input);
  assert_eq!(result["community_id"], json!(42));
  assert_eq!(
    result["actor_pseudonym"],
    json!("abc123def"),
    "pseudonym is opaque-by-construction; scrubber regex doesn't match"
  );
}
```

**Test 5 — `scrub_mention_after_cyrillic_letter_is_scrubbed`** (WP-2 active):

```rust
#[test]
fn scrub_mention_after_cyrillic_letter_is_scrubbed() {
  // The ASCII boundary class [^A-Za-z0-9._%+\-] is permissive for
  // non-ASCII letters (they fall INTO the boundary set because
  // they're not in the excluded ASCII alphanumeric set). So a
  // mention preceded by a Cyrillic 'а' IS scrubbed.
  assert_eq!(
    scrub("hi а@alice"),
    "hi а[redacted]"
  );
}
```

**Test 6 — `scrub_zero_width_joiner_between_at_and_handle`** (`#[ignore]`; WP-2 deferred; verbatim from plan §10.2):

```rust
#[test]
#[ignore = "v1-redaction-r2 will normalise via unicode-normalization crate"]
fn scrub_zero_width_joiner_between_at_and_handle() {
  // TODO(v1-redaction-r2): handle Unicode confusables (ZWJ injection
  // between @ and handle). v0 over-scrub bias is acceptable but ZWJ
  // adversarial vector is not currently caught. Tracked via DQ
  // kind:"log" carry-forward written at Task 1 completion.
  assert_eq!(
    scrub("@\u{200D}alice"),
    "[redacted]"
  );
}
```

**Test 7 — `scrub_handle_with_unicode_confusable_in_username`** (`#[ignore]`; WP-2 deferred):

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
```

**Total new tests: 7.** (The plan §13 Task 1 lead says "6 active + 2 #[ignore]" but the enumerated list has 5 active + 2 #[ignore] = 7. Confirmed against plan §10.1 + §10.2: Test 1 is one of the 5 active. Net: 5 existing + 5 new active + 2 #[ignore]'d = 12 tests; 10 run, 2 ignored.)

### 2.3 Gotchas (verbatim from plan §13 Task 1)

- The existing test module imports `use super::*;` at line 105 and `use serde_json::json;` at line 107. **Test 4 uses `json!(...)` without an additional import.**
- `pretty_assertions::assert_eq` at line 106 supplies the `assert_eq!` macro. **Do NOT re-import `std::assert_eq` or the `assert_eq!` shadowing breaks.**
- `#[ignore = "..."]` attribute reason string is Rust 1.41+; valid on the project's `rust-toolchain.toml` 1.95.
- WP-5 test: the synthetic `community_id: 42` is a leaf integer, not a key the rest of the test suite cares about — keep as-is.
- The `а@alice` test (Cyrillic) uses **U+0430 (NOT U+0061)**. The file is UTF-8 (`redaction.rs` already contains the em-dash `—` at line 4); the Cyrillic char is also UTF-8 safe. **Author the brief and the test body in UTF-8.**

### 2.4 Insertion point (mechanical, no ambiguity)

The existing `mod tests` block ends at line 159 with a single closing `}`. Insert the 7 new tests BEFORE that closing brace, with one blank line between each test and one blank line before the closing `}`. Use the `Edit` tool with `old_string`:

```
  fn scrub_preserves_keys_and_non_string_scalars() {
    let input = json!({
      "@handle_key": 42,
      "flag": true,
      "missing": null,
    });
    assert_eq!(scrub_json(&input), input);
  }
}
```

`new_string`:

```
  fn scrub_preserves_keys_and_non_string_scalars() {
    let input = json!({
      "@handle_key": 42,
      "flag": true,
      "missing": null,
    });
    assert_eq!(scrub_json(&input), input);
  }

  <test 1 verbatim from §2.2>

  <test 2 verbatim from §2.2>

  <test 3 verbatim from §2.2>

  <test 4 verbatim from §2.2>

  <test 5 verbatim from §2.2>

  <test 6 verbatim from §2.2>

  <test 7 verbatim from §2.2>
}
```

Indent each new test body with 2-space indentation matching the existing `mod tests` block. Each `#[test]` line gets 2 spaces; each line inside the fn body gets 4 spaces (matching the existing tests). Verify by reading the file post-edit before staging.

---

## 3. Required reading

- `.claude/PRPs/plans/v1-redaction-r1.plan.md` §10.1 (canonical test shape), §10.2 (`#[ignore]` shape), §13 Task 1 (action + gotchas), §15.1–§15.3 (DoD commands), §16a Story 1 (verify gate)
- `.claude/PRPs/briefs/v1-redaction-r1-impl-0.md` — Task 0 precedent (probe-style verification of harness; this brief's §4 mirrors its DQ + push-discipline structure)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case-A sibling-uniformity (this task: plain `#[test] fn`, NO `LemmyResult` since canonical siblings don't use it)
- `.claude/lessons/feedback_clippy_test_style.md` — workspace clippy denies `unwrap`/`expect`/`allow_attributes`; the new tests use `assert_eq!` only, no `unwrap`
- `.claude/lessons/feedback_async_pool_test_pattern.md` — informational; the canonical siblings are synchronous so this task does NOT need the async-pool pattern (sibling-pattern check)
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` — submodule init pre-empt (mandatory before cargo touches `lemmy_email` indirectly via workspace check)
- `.claude/lessons/feedback_pipes_mask_exit_codes.md` — never pipe wrapper output through `tail`/`head`/`grep` (Lane Q-r2a Task 0 confirmed this trap again)
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md` — laptop runs cargo after worker push; impl-task does NOT run cargo for the final gate

---

## 4. Constraints

- **File ownership:** edit ONLY `crates/db_schema/src/source/governance/redaction.rs`. No other file edits.
- **Sibling-pattern discipline:** mirror the 5 existing tests at lines 109–158 verbatim (plain `#[test] fn`, `assert_eq!`, no `Result<T, E>` returns, no async). Per `feedback_lemmy_error_no_std_error.md` Case-A. The advisor will catch-fire if the new tests use `LemmyResult<()>` outer because the canonical siblings do not.
- **No `unwrap` / `expect` / `allow_attributes`** anywhere in the new tests. Workspace clippy denies these in test code (`feedback_clippy_test_style.md`). All new tests use `assert_eq!` only.
- **Submodule init pre-empt:** before the first cargo invocation, run `git submodule update --init --recursive` (per `feedback_phase_lane_worktree_bootstrap_checklist.md` + Lane A Task 0 #489 + Lane Q-r2a Task 0 #490 confirmed).
- **Pipe-mask-exit-code discipline:** when capturing exit codes from `bash scripts/brehon/cargo-*.sh`, always redirect to a log file FIRST, then `tail` the log; never pipe through `tail`/`head` directly (the pipe sets `$?` to 0).
- **Cargo invocations on the daemon for worker self-checks** — the plan's §15 commands use Windows `cmd //c "scripts\\brehon\\cargo-*.bat"`. **The Junior worker runs on Linux**, so for worker-side self-checks (compile, clippy) use the Linux equivalents:

  ```bash
  bash scripts/brehon/cargo-check.sh -p lemmy_db_schema --features full > .claude/PRPs/debug/v1-redaction-r1-task1-worker-check.log 2>&1
  echo "check exit: $?"
  tail -40 .claude/PRPs/debug/v1-redaction-r1-task1-worker-check.log

  bash scripts/brehon/cargo-clippy.sh -p lemmy_db_schema --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-redaction-r1-task1-worker-clippy.log 2>&1
  echo "clippy exit: $?"
  tail -60 .claude/PRPs/debug/v1-redaction-r1-task1-worker-clippy.log

  bash scripts/brehon/cargo-test.sh -p lemmy_db_schema --features full redaction::tests > .claude/PRPs/debug/v1-redaction-r1-task1-worker-test.log 2>&1
  echo "test exit: $?"
  tail -40 .claude/PRPs/debug/v1-redaction-r1-task1-worker-test.log
  ```

  **EXPECT (worker-side):** all 3 exit 0; the test log shows `test result: ok. 10 passed; 0 failed; 2 ignored` (5 existing + 5 new active + 2 `#[ignore]` = 12 tests; 10 run, 2 ignored).

  If any worker-side gate fails: do NOT push the test edits. File `kind: "blocker"` DQ on the worker branch citing the failing log slice (≤200 lines) and stop.

- **Push discipline + post-push validate-pending-laptop DQ:** after worker-side cargo gates pass, commit + push the test edits as ONE commit, then write a `kind: "validate-pending-laptop"` DQ entry per `.claude/rules/decision-queue.md` §"validate-pending-laptop handler" + `feedback_laptop_default_for_validate_pending.md`. **The advisor runs the final §15 gates on the laptop** with the plan's Windows `cmd //c "...bat"` commands; the worker does NOT re-run them post-push. The DQ entry's `commands` array MUST list the plan's three canonical `cmd //c "scripts\\brehon\\cargo-{check,clippy,test}.bat ..."` lines from plan §15.1–§15.3 / §13 Task 1 VALIDATE block verbatim — the laptop runs the Windows path.

  Append the DQ entry via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` (the helper is now `100755` on trunk; the chmod fix shipped at trunk `817291c9a`, merged into phase via the Lane Q-r2a chain). Commit + push the DQ entry on the worker branch (per `decision-queue.md` §"Mid-task visibility").

- **kind: "log" DQ at task completion** (per plan §13 Task 1 final block): write a `kind: "log"` DQ entry recording the 2 `#[ignore]`'d Unicode confusable tests as v1-redaction-r2 carry-forward. Schema per plan §13 Task 1 lines 1054–1069 (verbatim shape). `from: "impl"`, `answered_by: "impl-self-resolved"`. Goes DIRECTLY to `resolved[]` (per `decision-queue.md` Recipe 2 — never `pending[]`).

- **Attribution (Hard refusals):** the worker's DQ writes use `from: "impl"`. **NEVER write `answered_by: "advisor"` or `answered_by: "user"` or `approved_by: <non-null>`.** Per `.claude/rules/decision-queue.md` Hard refusals #1, #6, #8, #9.

- **DQ id generation:** use `bash scripts/brehon/dq-v3-new-entry.sh` for new ids (composite `<session>-<seq>`). NEVER use the abolished `next_id = max(all_ids)+1` recipe.

- **Commit subject:** `feat(redaction): adversarial test corpus locks docstring invariants (task 1)`. Include in the commit body the `HANDOVER:` YAML trailer per the impl-task template (filesCreated: `[]`; filesModified: `[crates/db_schema/src/source/governance/redaction.rs]`; keyDecisions: sibling-pattern mirror + 2 `#[ignore]` carry-forward).

- **Shape G SUSPENDED:** per DQ #229 + `project_shape_g_suspended_2026_05_16.md`. This is a pre-Shape-G plan; cargo gates run laptop-side via the validate-pending-laptop pathway. **No `cargo test --workspace --test e2e` from the worker** — the phase-tip e2e gate fires after Task 3 completes, not after Task 1.

---

## 5. Forbidden-window check (advisor pre-queue)

Per `.claude/rules/advisor-orchestrator.md` §5.1: Shape G suspended → forbidden-window check IS binding for worker-side cargo on EliteDesk. Advisor verified at queue time (~21:00 UTC, outside standard forbidden windows). Subagent's task pre-flight refuses with `FORBIDDEN_WINDOW: <window>` if mis-queued.

---

## 6. Context

- **Phase:** v1-redaction-r1 (issue #58 — GDPR identifier-scrubber hardening)
- **Plan:** `.claude/PRPs/plans/v1-redaction-r1.plan.md` on `phase-v1-redaction-r1` at `5e9b1e35b`
- **Phase branch:** `phase-v1-redaction-r1` (cut from `governance-v0`; current tip `9fa20b0bc`)
- **Base branch for this task:** `phase-v1-redaction-r1`
- **Lane mode:** Mode B (no laptop-side phase worktree; daemon-side Junior runs in isolated worktree)
- **Plan-approved at:** 2026-05-28 gate-1 sign-off (DoD smoke pass + watchpoint specificity gate pass; complexity 4/10)
- **Task 0 precedent:** #489 completed 2026-05-28T20:00 with all 13 probes PASS — submodule init verified, all `bash scripts/brehon/cargo-*.sh` wrappers honor `-p`, `--features`, `--no-deps`, `-D warnings`. Pipe-mask-exit-code trap self-corrected mid-task.
- **Concurrent lane activity:** Lane Q-r2a Task 0 #490 completed 2026-05-28T20:27 (resolved with plan amendment at phase tip `3ff47b26e`). Zero file overlap (Lane A = redaction.rs; Lane Q-r2a = scripts + decision-queue.json).
- **After Task 1 worker push + cargo gates pass on laptop:** advisor mutates the validate-pending-laptop entry to `result: "pass"`; advances to Task 2 brief author.
- **If Task 1 worker push triggers a laptop-side compile/clippy/test failure:** advisor reads `log_slice`, classifies per `.claude/rules/advisor-orchestrator.md` §5.3 §G4 classifier; allowlist match → narrow fix-impl task; non-allowlist → catch-fire to user.

---

## 7. Acceptance criteria

- `crates/db_schema/src/source/governance/redaction.rs` contains 7 new test functions appended INSIDE `mod tests`, AFTER the existing `scrub_preserves_keys_and_non_string_scalars` (line 158), BEFORE the closing `}`.
- All 7 use plain `#[test] fn`; 2 carry `#[ignore = "..."]` attribute; bodies use `assert_eq!` only (no `unwrap`/`expect`).
- Worker-side `bash scripts/brehon/cargo-check.sh -p lemmy_db_schema --features full` exits 0.
- Worker-side `bash scripts/brehon/cargo-clippy.sh -p lemmy_db_schema --features full --no-deps -- -D warnings` exits 0.
- Worker-side `bash scripts/brehon/cargo-test.sh -p lemmy_db_schema --features full redaction::tests` exits 0 with `10 passed; 0 failed; 2 ignored`.
- Exactly one commit on the worker branch with subject `feat(redaction): adversarial test corpus locks docstring invariants (task 1)` and a `HANDOVER:` trailer in the body.
- Exactly two DQ writes on the worker branch (pushed before task completion): (a) `kind: "validate-pending-laptop"` in `pending[]` listing the plan's three Windows `cmd //c "...bat"` commands verbatim; (b) `kind: "log"` in `resolved[]` recording the 2 `#[ignore]`'d Unicode confusable carry-forwards with `answered_by: "impl-self-resolved"`.
