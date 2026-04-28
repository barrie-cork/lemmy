# Brief: issue-96-impl-1b — Test/lint cluster (cr-1, cr-2, cr-6, cr-7, cr-10)

## 1. Role + dispatch

[role:impl-task] issue-96-cluster2 — fix GitHub issue #96 cr-1 (markdown fence), cr-2 (no-op binding), cr-6 (fixture refactor), cr-7 (seed_case severity), cr-10 (assert CaseStatus)

## 2. Scope

Fix five carry-forward findings from PR #95. These are test-only and documentation fixes — no production logic changes.

**cr-1** — `.claude/PRPs/reports/v1-JM-b-retro.md` around line 100
Add a language tag to a fenced code block. Change ` ``` ` to ` ```bash ` on the `git grep` snippet.

**cr-2** — `crates/api/api/src/governance/admin_assign_jury.rs:718`
Delete the no-op binding `let _ = current_geo_enabled;`. The comment above it already documents intent. One-line deletion.

**cr-6** — `crates/server/tests/e2e.rs` around line 6961–7053
Relocate the four helper functions (`bootstrap`, `seed_user`, `seed_community`, `seed_jurors`) and the `SIGNING_SEED_HEX` constant + unsafe env-set logic into the existing `governance_fixtures` module. Update their visibility to `pub`. Update all call sites within `e2e.rs` that reference these helpers.

Before editing: search for all call sites with `grep -n "bootstrap\|seed_user\|seed_community\|seed_jurors\|SIGNING_SEED_HEX" crates/server/tests/e2e.rs` to locate every usage.

**cr-7** — `crates/server/tests/e2e.rs` around line 7060–7089
Inside `seed_case`, derive `severity` from the incoming `severity_tier` argument rather than using `CaseSeverity::default()`. Add a mapping from `SeverityTier → CaseSeverity` and set `form.severity` to the derived value.

IMPORTANT (lesson from PMD #190): this is a fixture-shape fix, not a production read-after-write failure. The test currently passes because callers supply explicit severity_tier values, so the `default()` mismatch does not trigger assertion failures. Verify this is purely a fixture-shape change before editing.

**cr-10** — `crates/server/tests/e2e.rs` around line 7820
In the test `admin_emergency_remove_case_has_severity_tier_severe`, add one assertion:
```rust
assert_eq!(case.status, CaseStatus::EmergencyRemove);
```
Production correctness is already guaranteed by `admin_emergency_remove.rs:155`. This is test-rigor strengthening only.

**Files authorised to edit:**
- `.claude/PRPs/reports/v1-JM-b-retro.md`
- `crates/api/api/src/governance/admin_assign_jury.rs`
- `crates/server/tests/e2e.rs`

Do NOT edit any other file. Do NOT open PRs. Do NOT push to governance-v0. Push your worktree branch only.

**Branch:** `fix/issue-96-cluster2` (Junior creates this automatically in your worktree)

## 3. Required reading

- `.claude/lessons/feedback_check_git_before_junior_queue.md`
- `.claude/lessons/feedback_junior_finalize_skips_when_worker_pre_pushes.md`
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — e2e.rs is 8945+ lines; use targeted Grep + Read with offset/limit, never load the whole file. Use Edit with precise old_string, not Write.
- GitHub issue body: `gh issue view 96 --repo barrie-cork/lemmy` (read for full finding context)

## 4. Constraints

- **Commit shape:** three commits:
  1. `chore(lint): clear PR #95 cosmetic carry-forwards — cr-1 fence tag, cr-2 no-op binding (#96)`
  2. `refactor(test): consolidate e2e fixture helpers into governance_fixtures module (cr-6 of #96)`
  3. `test(governance): strengthen assertions in seed_case + emergency_remove test (cr-7, cr-10 of #96)`
- **Shape G validation:** after committing all fixes and pushing your worktree branch to origin, write a `kind: "validate-pending"` entry to `.claude/decision-queue.json` containing the `workflow_run_id` of the triggered `cargo-validate-workspace.yml` run. Commit and push the DQ entry immediately.
- Do not author plan files, rule files, ADR changes, or lesson files.
- Do not amend commits after pushing.
- cr-6 is a refactor — all call sites within `e2e.rs` must be updated. Grep for every usage before moving the functions.
- Do not run `cargo test --workspace` — Shape G handles validation on GitHub runners.
