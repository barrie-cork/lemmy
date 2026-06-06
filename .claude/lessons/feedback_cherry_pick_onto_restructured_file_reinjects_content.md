# Cherry-picking a fix onto a since-restructured file silently re-injects stale content

When a commit authored against a **monolithic** file is cherry-picked (or
3-way-merged) **after** that file has been **decomposed/extracted**, the
merge can silently paste pre-decomposition content back into the host
file — content that now lives in the extracted children. The result
compiles-fail (duplicate definitions / orphaned delimiters) or, worse,
duplicates state that drifts from the canonical copy.

**Sibling lesson:** `feedback_merge_forward_e2e_conflict_default_to_governance.md`
(merge-forward *conflict* resolution on `e2e.rs`). Same recurring victim
file; different mechanism (that one = conflict side-choice, this one =
silent re-injection with no visible conflict markers left behind).

## The incident (2026-06-05, m2-core-hook)

`crates/server/tests/e2e.rs` was decomposed early June: ~18,900 lines
→ a **156-line host** that pulls fixtures + tests from
`tests/e2e/{common/mod.rs,governance.rs,admin_config.rs,jury_mechanics.rs,sponsor_liability.rs}`
via `include!`. The BUG-1 fix (`91abf7258`, authored against the
*monolithic* file) was later cherry-picked as `6f4947b48` **after** the
decomposition. The 3-way merge re-injected ~2,100 lines of
pre-decomposition content into `e2e.rs`, producing **three** breakages:

1. A duplicate `governance_fixtures` module pasted **without** its
   `mod governance_fixtures {` opener → dangling `}` →
   `error: unexpected closing delimiter`.
2. Duplicate **test fns + helpers + a const** that `include!` re-expands
   at crate scope → `E0428: defined multiple times` (×9).
3. The fix's own **migration revert-list entry** landed in the dead
   duplicate, never reaching the canonical copy → a parity `#[test]`
   (`phase1_revert_list_matches_disk`) drifts and fails.

`e2e` was uncompilable on `governance-v0` for ~13 hours before the next
e2e run surfaced it. The closeout e2e that "passed" ran *before* the
cherry-pick, so its green was stale evidence.

## Detection (fast)

- **Duplicate-definition scan across the `include!` scope.** For any
  host file using `include!`, scan the host + all `include!`d children
  together for fn/const/mod names defined in 2+ files. Regex:
  `^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)`; flag names with
  count ≥ 2 across the file set. (A name in two `include!`-joined files
  IS an E0428 — `include!` is textual expansion at one scope.)
- **Structural delimiter scan** (braces+parens+brackets, strings/comments
  stripped) catches the orphaned-`}` class but NOT the balanced-duplicate
  class — run both.
- **The cheapest gate of all:** `cargo test --no-run --workspace --test e2e`
  on trunk in CI would have caught all three immediately.

## Fix pattern

- If `git log <last-good>..HEAD -- <file>` shows the corrupting
  cherry-pick is the **only** post-restructure change and its diff is
  **purely additive duplicates**, restore the host file surgically:
  `git checkout <last-good-SHA> -- <host-file>` (confirm `<last-good>`
  is an ancestor of HEAD first).
- Restore ONLY the corrupted host file. Do **not** restore the extracted
  children to the old SHA — they have legitimate later evolution.
- Any state the cherry-pick was *supposed* to update in the canonical
  copy (here: the migration revert-list entry) must be re-applied by
  hand to the canonical copy — it was lost in the dead duplicate.

## Prevention

- **Re-author, don't cherry-pick, across a restructure boundary.** When
  a fix targets a file that has since been split, port the change to the
  new structure by hand instead of cherry-picking the old-shape commit.
- **Post-cherry-pick duplicate scan** on any cherry-pick/merge that
  touches a decomposed `include!` host.
- **Trust the in-log marker, not the task notification.** The background
  `cargo … && echo EXIT_0 || echo EXIT_NONZERO` reported "exit 0" (the
  `echo` succeeded) while cargo exited 101. Per
  `feedback_pipes_mask_exit_codes.md` + `.claude/rules/cargo-output-capture.md`.

**Full incident report:** `.claude/PRPs/debug/e2e-rs-botched-cherry-pick-corruption.md`.
