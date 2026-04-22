# v1-AD-c CR round 3 — resume brief (2026-04-21 → 2026-04-22)

**Session closed at**: 2026-04-21T22:26Z
**Next entry point**: validate + stage + commit + push the 3-fix bundle sitting in working tree.

## Where we are

- **Branch**: `phase-v1-AD-c`
- **HEAD**: `6c8fc3211` (CR round-2 parity fix, green in CI)
- **PR**: #81 OPEN, MERGEABLE, CLEAN (merge-gate passed at HEAD)
- **Working tree** — 3 files modified, NOT staged, NOT committed:
  - `crates/api/api/src/governance/admin_rule_sets.rs` (Fix B + Fix D handler refactor)
  - `crates/server/tests/e2e.rs` (Fix D test rewrite)
  - `crates/api/api/src/governance/config.rs` (Fix C was already applied in round 2 — this may be a phantom diff; verify)

Run `git diff --stat` first thing tomorrow to confirm the 3 paths and diff shape.

## What this resume covers

CR **round 3** on PR #81 — advisor triage after CR re-reviewed `6c8fc3211`:
- **Rust-code findings** (5 total): A, B, C, D, E
- **Harness findings** (15): explicitly deferred per user directive "ignor harness CR comments"
- **Verdict**:
  - A (DTO v0 scope) → **rebuttal** — text provided, post to PR thread
  - B (list-endpoint snapshot race) → **fix** applied
  - C (ScopeParseError doc drift) → verified already applied in round 2 — no edit needed
  - D (A3 test recreates handler mapping) → **fix** applied via Option 4 (extract helper)
  - E (D1 pin-coverage assert) → **rebuttal** — text provided, post to PR thread

## Edits applied (three files)

### 1. `crates/api/api/src/governance/admin_rule_sets.rs`

#### Fix B — wrap `admin_list_rule_sets` reads in `run_transaction` (L283-311)

Two reads were hitting the DB outside any transaction — `rule_set_version::table.load(conn)` + `config::get_int_opt(...)` — so a concurrent `admin_create_rule_set` could split them across different snapshots. The returned `active_version_id` could point at a version not in the returned `versions` list.

Fix: wrap both reads in a single `conn.run_transaction(|conn| async move { ... }.scope_boxed())` closure. Inside the closure, `&mut conn.into()` converts `&mut AsyncPgConnection` → `&mut DbPool<'_>` for the `get_int_opt` signature (precedent at `admin_rule_sets.rs:247` in the create path). Uses FQN `lemmy_utils::error::LemmyError` in the `Ok::<_, ...>` turbofish to avoid adding an import.

Compile-verified: `cargo-check.bat -p lemmy_api --features full` green in 1m42s at `.claude/build-fixB-check.log`.

#### Fix D, part 1 — extract `pub fn map_rsv_unique_violation` helper (new, after `process_create_rule_set`)

`process_create_rule_set` had an inline `match` on `UniqueViolation` mapping it to `LemmyErrorType::Unknown("rule_set_version already exists...")`. The A3 test was recreating that mapping inline in the test body (CR#D: "if the handler's mapping drifts, this test doesn't fail").

Fix: extract the `match` into `pub fn map_rsv_unique_violation(err: diesel::result::Error) -> lemmy_utils::error::LemmyError` at module scope. `process_create_rule_set` now does `.map_err(map_rsv_unique_violation)?`. Helper has a doc comment citing CR PR #81 round 2 finding D.

**Why `pub` not `pub(crate)`**: the integration test in `crates/server/tests/e2e.rs` (separate crate) imports it. Consistent with `governance_log::append`, `redaction::scrub`, etc.

Compile-verified: `cargo-check.bat -p lemmy_api --features full` green in 6.92s at `.claude/build-fixD-handler.log`.

### 2. `crates/server/tests/e2e.rs`

#### Fix D, part 2 — A3 test routes DieselError through real helper (L5387-5419)

Replaced the inline `LemmyErrorType::Unknown(...).into()` re-creation with a call to `lemmy_api::governance::admin_rule_sets::map_rsv_unique_violation(err)`. Test still drives a DB-layer UniqueViolation via direct `diesel::insert_into` (preserves CR round-1's non-tokio::join! determinism), but the **mapping** is now from production.

Step 3 + Step 4 doc comments updated to reference the helper instead of line numbers of the old inline `match`.

Background compile was in progress at session close — see "Outstanding work" below.

### 3. `crates/api/api/src/governance/config.rs` — possibly no-op

Fix C was the `ScopeParseError` doc comment rewrite. Advisor's verbatim diff target matched what was already in the file (lines 161-163 already read "Keeps a narrow shape so tests can assert on equality; malformed input currently carries the original wire text. See task 1 GOTCHA in the v1-AD-c plan.").

`git status` showed `config.rs` as modified — verify tomorrow whether there's actually a diff. If `git diff config.rs` is empty, this file's M-flag is stale and can be ignored. If there's a diff, it was applied previously in round 2 (commit `5c04e6ec2`) and the working copy matches — no action needed.

## Outstanding work (tomorrow)

### Step 1: verify working tree

```bash
git status --short
git diff --stat
git diff crates/api/api/src/governance/config.rs  # should be empty or near-empty
git diff crates/api/api/src/governance/admin_rule_sets.rs  # Fix B (lines ~283-311) + Fix D handler (lines ~197-264)
git diff crates/server/tests/e2e.rs  # Fix D test (lines ~5387-5419)
```

Expected diff shapes:
- `admin_rule_sets.rs`: ~30 lines net addition (helper extraction ~20 lines, B tx wrap +6 net)
- `e2e.rs`: ~5 lines net deletion (inline match collapses to single `match duplicate_insert` call)
- `config.rs`: empty or stale

### Step 2: finish the background test compile

A `cargo-test.bat --test e2e --no-run -p lemmy_server` was running in background at session close (task ID `bvpvx4gqo`, log at `.claude/build-fixD-test-compile.log`). Check its exit code:

```bash
tail -15 .claude/build-fixD-test-compile.log
# Expected final line: "Finished `test` profile [unoptimized] target(s) in Nm Ns"
# If RED: most likely missed-import on map_rsv_unique_violation pathway
```

If the compile errored:
- Common miss: `use lemmy_api::governance::admin_rule_sets::map_rsv_unique_violation;` never added to the test's `use` block (I kept it as a fully-qualified path in the call site, so this shouldn't bite — but verify).
- Another candidate: `DieselError` / `DatabaseErrorKind` imports inside the test's `use` block are now dead (the `match &duplicate_insert { Err(DieselError::...) => ...}` at Step 3 still uses them, so they should stay).

### Step 3: run validation gates (4 test runs + check + clippy)

```bash
# 1. A3 test (the one Fix D rewrites)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_create_rule_set_duplicate_version_rejected > .claude/build-validate-A3.log 2>&1"; echo "exit: $?"
# Expected: 1 passed in ~25s. If RED, the helper call signature or doc-comment cite is wrong.

# 2. A4 test (exercises Fix B's admin_list_rule_sets path)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_list_rule_sets_returns_versions_with_active_version_id > .claude/build-validate-A4.log 2>&1"; echo "exit: $?"
# Expected: 1 passed. Confirms the run_transaction wrap didn't break the single-snapshot read semantics.

# 3. D1 test (sanity — verifies the pin coverage we're rebutting CR#E on still passes)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server case_open_pins_applied_config_snapshot_and_rule_set_version_id > .claude/build-validate-D1.log 2>&1"; echo "exit: $?"
# Expected: 1 passed. The column pin assert at e2e.rs:5779-5783 is the coverage CR#E claims is missing.

# 4. Workspace check
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/build-validate-workspace.log 2>&1"; echo "exit: $?"

# 5. Clippy on the touched crate
cmd //c "scripts\\brehon\\cargo-clippy.bat -p lemmy_api --features full --no-deps -- -D warnings > .claude/build-validate-clippy.log 2>&1"; echo "exit: $?"
```

All five must be exit 0.

### Step 4: stage for advisor review-go

Advisor's pause-per-commit protocol — stage the 3-file diff (or 2-file, if `config.rs` is stale) and **do not commit yet**. Advisor will diff independently before review-go.

```bash
git add crates/api/api/src/governance/admin_rule_sets.rs crates/server/tests/e2e.rs
# Add config.rs only if `git diff` shows it has a real diff
git diff --cached --stat
```

Commit message template (from advisor, pending D-section tweak that was already provided in advisor's revised ruling):

```
fix(v1-AD-c): address 3 CR findings on PR #81 (round 3)

- B (admin_rule_sets.rs:283-311): wrap versions + active_version_id
  reads in conn.run_transaction so the list response reflects a single
  snapshot. Prevents a concurrent create from splitting the two reads
  across different DB states — the returned active_version_id now
  consistently references a version in the returned `versions` list.
- C (config.rs:161-163): align ScopeParseError doc comment with the
  enum shape; Malformed(String) does carry a payload, which the prior
  "no reason field" claim contradicted.
- D (admin_rule_sets.rs extract map_rsv_unique_violation + e2e.rs:5411-5419
  call it): the duplicate-version test now routes the DB-layer UniqueViolation
  through production's mapping helper instead of recreating the mapping
  inline. Handler's auto-increment version means a client-driven collision
  isn't reachable through the public API; the helper-extraction pattern gives
  regression safety without reintroducing tokio::join! non-determinism or
  adding test-only production hooks.

Rebuttals posted on PR thread:
- A (DTOs expand v0 surface): v1-AD-c extends the governance API per
  the v1-AD-c sub-PRD; the v0 11-endpoint guideline is scoped to
  governance-v0 baseline, not v1 branches.
- E (snapshot pin coverage): the pin is on moderation_case.rule_set_version_id
  column, not in applied_config_snapshot JSON. Column-level pin is
  strictly asserted at e2e.rs:5779-5783; adding it to the JSON would
  break the REQUIRES_RE_JURY_KEYS metadata-parity invariant.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

**If `config.rs` has no real diff**: drop the C bullet and change the subject to "address 2 CR findings on PR #81 (round 3)".

### Step 5: post rebuttals (A + E)

After commit + push, post 2 inline comments on the PR thread.

**A rebuttal — on `crates/api/api_common/src/governance.rs:585`**:

```
The v0 11-endpoint scope limit (05-mvp-and-delivery-plan.md §2) governs governance-v0.
This PR lands on phase-v1-AD-c targeting governance-v0 as the base, but explicitly
extends the governance API surface per the v1-AD-c sub-PRD (admin config + rule-set
versioning). AdminCreateRuleSet*, AdminListRuleSets*, and RuleSetVersionView are the
wire contracts for POST /admin/rule-sets and GET /admin/rule-sets — v1 endpoints
authorised by the sub-PRD, not v0 additions. The coding-guideline sentinel for
v0-scope is stale for v1 work on this branch.
```

**E rebuttal — on `crates/server/tests/e2e.rs:5805`**:

```
applied_config_snapshot is the 7-key JSON payload defined by CONFIG_KEY_METADATA
entries where requires_re_jury == true (see
crates/api/api/src/governance/case_open_snapshot.rs:50-58 and the metadata-parity test
in that file's cfg(test) module). rule_set_version_id is a sibling column on
moderation_case (pinned at crates/api/api_crud/src/governance/create_report.rs:218),
not a requires_re_jury config key. The existing test asserts the column pin strictly
at e2e.rs:5779-5783:

assert_eq!(
  case.rule_set_version_id.map(|r| r.0),
  Some(v1.rule_set_version_id),
  "rule_set_version_id pinned to the active community version",
);

If the handler stopped writing the pin to the column, case.rule_set_version_id would
be None and this assert fails. The coverage concern is addressed in its real location.
Adding rule_set_version_id to the JSON snapshot would break the REQUIRES_RE_JURY_KEYS
metadata-parity invariant — not an acceptable fix path.
```

Use `gh api repos/barrie-cork/lemmy/pulls/81/comments` to post inline (path + line + body). Or post as top-level PR comments if inline threading is painful. Advisor said "can be drafted any time but don't need advisor review-go".

### Step 6: CI watch + merge

Push, then monitor `cargo-test-e2e` run on the new HEAD (~3 min typical). Merge-gate = that one workflow. When green, PR is ready for advisor merge decision.

## Key decisions + advisor rulings captured

- **CR round 3 triage** (2026-04-21T22:00Z): ship B + C + D + rebut A + E. Single commit. Pause-per-commit. Harness CR findings deferred per user.
- **Fix E rebuttal** (2026-04-21T22:10Z): retro14 — advisor rubber-stamped CR#E "accept as-is" without verifying snapshot shape. I stopped per `feedback_advisor_instruction_mismatch_stop_and_ask.md`. Advisor reversed to rebuttal.
- **Fix D helper extraction** (2026-04-21T22:20Z): retro14 expanded — advisor's "drive through real handler" directive presumed an `AdminCreateRuleSet { version: 1, ... }` field that doesn't exist. Auto-increment handler means no client-observable collision path. Advisor reversed to Option 4 (pub fn helper extraction).

Both reversals were caught by the stop-and-ask protocol. Memory entry `feedback_advisor_instruction_mismatch_stop_and_ask.md` is load-bearing — two applications in one round.

## Files to look at first tomorrow

1. `.claude/build-fixD-test-compile.log` — did the background test compile finish? Tail it.
2. `git status` / `git diff --stat` — confirm 3 files modified, correct shape.
3. This brief + `.claude/PRPs/v1-AD-c-runlog/01-v1-AD-c-progress.md` tail (entries after 2026-04-21T22:00Z).

## What's NOT in this brief

- Telegram integration — deferred to v1-AD-c→d transition per memory entry.
- Harness CR findings (15) — out of phase per user directive.
- Post-merge chores (#5 admin-config-write.sh Option A, etc.) — carry-forward, no action.

## Advisor context (pointers)

- Advisor resume brief: `.claude/PRPs/reports/v1-AD-c-advisor-resume-state.md`
- Advisor notes + retros: `.claude/PRPs/v1-AD-c-runlog/00-advisor-notes.md`
- CR review raw: `.claude/cr-inline-rust-6c8fc321.txt` (Rust-only filter) + `.claude/cr-review-pr81-harness.txt` (harness — deferred)
