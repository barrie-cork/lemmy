# Task 0 narrative archive

Read once. Reference from progress log via runlog-Q-ID. Do not re-read on subsequent iterations.

## Q1 — clippy baseline RED with 135 errors

Plan §15 Level 4 ratcheted from `--workspace --features full --no-deps` (v1-AD-b Level 6) to `--workspace --all-targets --features full --no-deps`. The `--all-targets` addition was silent — no RFC, no v1-AD-b precedent.

Upstream Lemmy `.woodpecker.yml` runs `--workspace --tests --all-targets --all-features` and passes. Therefore the 135 errors are 100% governance-fork test-code debt. Lemmy's own integration tests are TypeScript/Jest, not Rust.

Distribution (from `.claude/audit-clippy-baseline.log`):
- 56× tests_outside_test_module (false-positive class for `tests/e2e.rs` — IS the test binary by convention)
- 34× indexing_slicing (`[0]`/`[1]` in test assertions)
- 27× items_after_statements (inline `use` after stmts in test fns)
- 5× expect_used
- 2× unwrap_used
- 11× misc

Files affected:
- `crates/server/tests/e2e.rs` — 126 errors (all)
- `crates/api/api/src/governance/reputation_snapshot.rs` — 5 errors (lines 838-856, all in `#[cfg(test)] mod tests`)
- `crates/api/api/src/governance/admin_config.rs` — 2 errors (1356/1375, parity helper)

## Decision: Option C hybrid

A=narrow-DoD-only, B=fix-135-errors, C=narrow+task8-delta-guardrail.

Picked C. A leaves test-target debt unobserved as new tests land. B is wrong scope (~40 min mechanical + masks real bugs under allow-attrs). C narrows the DoD to v1-AD-b ratchet AND adds advisory delta-vs-135 check at task 8.

Two-commit shape required:
1. `docs(plan): narrow v1-AD-c clippy DoD to match v1-AD-b ratchet (drop --all-targets)` — narrowing alone, audit-visible
2. `docs(plan): v1-AD-c plan + Task 0 pre-phase audit` — bundle

Why two: CodeRabbit needs to see the ratchet narrowing as a discrete commit subject, not buried in `docs(plan): bundle`.

## 5 plan edit sites (verbatim text Impl applies)

All edits in `.claude/PRPs/plans/v1-admin-dashboard-c.plan.md`.

### Site 1 — Line 666 (§13 Task 0 step 5)

KEEP `--all-targets` (purpose: record baseline). Change trailing expectation:

- Before: `→ expect **exit 0** (prior-phase clippy baseline)`
- After: `→ **expect red — capture exit code and error count as baseline N for Task 8 delta check; baseline = 135 at v1-AD-c base per .claude/audit-clippy-baseline.log**`

### Site 2 — Line 1361 (§15 Level 4)

DROP `--all-targets` from the clippy command. Append sentence after the command block:

> Test-target debt is tracked separately per Task 0 audit report; v1-AD-c does not gate on `--all-targets` to avoid blocking on pre-existing inherited debt unrelated to the sub-phase scope. Task 8 captures `--all-targets` delta vs baseline N=135 as advisory.

### Site 3 — Line 1409 (§16 acceptance criteria 3rd checkbox)

DROP `--all-targets`. Add sub-bullet:

> Task 8 captures `--all-targets` delta vs baseline 135; net-new errors block the task-8 commit.

### Site 4 — §13 Task 0 OUTPUT (~line 670)

EXTEND audit report enumeration to include lint distribution table (the 56+34+27+5+2+11=135 breakdown above).

### Site 5 — §13 Task 8 VALIDATE block

ADD advisory command:

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --all-targets --features full --no-deps -- -D warnings > .claude/build-task8-clippy-alltargets.log 2>&1"; echo "exit: $?"
# Advisory check, NOT a DoD gate. Acceptance: error count <= 135 (baseline at v1-AD-c base).
# Net-new errors introduced by task 8's 8 new tests block the commit until fixed.
# Net-zero delta is fine. File a tracking issue at phase-close per the runlog decision.
```

## Phase-close followup issue (NOT now)

Title: `chore(lint): clear e2e.rs + governance test-target clippy debt (135 errors at v1-AD-c base)`
Body: link runlog Q1, attach lint distribution.

## Resume sequence after edits

1. Re-run narrowed clippy: `cargo clippy --workspace --features full --no-deps -- -D warnings` → expect 0
2. Stage commit 1 (plan-edits-only)
3. Write `.claude/PRPs/reports/v1-AD-c-task0-audit.md` with lint distribution table
4. Stage commit 2 (plan + audit-report bundle)
5. Append `i | t0 | stage | both-commits-staged commit-msgs-as-above` to progress log
6. Wait for advisor `a | t0 | review-go` before `git commit`

## plan-drift-t2-t5-t7 (2026-04-21)

**Detected.** Plan §13 VALIDATE lines at:
- L757 (Task 2): `cargo-check.bat -p lemmy_api_common --features full`
- L1195 (Task 5b): `cargo-check.bat -p lemmy_api_crud --features full`
- L1239 (Task 7): `cargo-check.bat -p lemmy_api_routes --features full`

all fail with `error[E0433]: cannot find governance in source` because `--features full` activates on the named crate only, not transitively, and `lemmy_db_schema::source::governance` is gated behind `#[cfg(feature = "full")]`. Downstream crates like `lemmy_db_views_reputation` (pulled in by `lemmy_api_common`'s transitive closure) see the module as configured-out.

**Root cause.** Documented in memory `feedback_features_full_workspace_only.md` (2 days old): `cargo check -p <crate> --features full` fails when the crate's transitive deps need `full` propagation. Only `-p lemmy_api --features full` (or `--workspace --features full`) walks the dep tree correctly because `lemmy_api` depends on every governance-touching downstream crate with `full` propagated via its own `Cargo.toml`.

**Fix.** Swap all 3 VALIDATE lines to `-p lemmy_api --features full`:

- L757 (Task 2): swap `-p lemmy_api_common` → `-p lemmy_api`. Log path stays `.claude/build-task2.log`.
- L1195 (Task 5b): swap `-p lemmy_api_crud` → `-p lemmy_api`. Log path stays `.claude/build-task5b.log`.
- L1239 (Task 7): swap `-p lemmy_api_routes` → `-p lemmy_api`. Log path stays `.claude/build-task7.log`.

Why `-p lemmy_api` not `--workspace`: lemmy_api is the broadest crate that compiles governance code cleanly under `--features full`. `--workspace` would re-check all upstream Lemmy crates unnecessarily (~5 min vs ~2 min). Task 1 already uses `-p lemmy_api --features full` and passes green in 1m 27s.

**Commit shape.** Impl must land the plan-drift fix as ITS OWN commit BEFORE staging task 2 code:

```
docs(plan): fix task 2/5b/7 VALIDATE to use -p lemmy_api for --features full propagation

The drafted -p lemmy_api_common / -p lemmy_api_crud / -p lemmy_api_routes
paths activate --features full on the named crate only; lemmy_db_schema::
source::governance (gated behind full) is configured-out for downstream
transitive compiles of lemmy_db_views_reputation. Per memory
feedback_features_full_workspace_only.md: only -p lemmy_api (or
--workspace) propagates full through the dep tree.

Detected at task 2 stage — see .claude/PRPs/v1-AD-c-runlog/
02-task0-narrative.md#plan-drift-t2-t5-t7.
```

Commit subject fits under 70 chars. Body explains the why.

THEN task 2 stages as normal per the pause-per-commit protocol. Task 2 code diff is already confirmed green via downstream `-p lemmy_api --features full` check at `.claude/build-task2-api.log` (1m 59s exit 0). No re-run needed after plan edit — the plan edit is purely documentation; the code validation already passed under the correct shape.

## commit-shape-corrected (2026-04-21)

**Detected.** Impl staged the original plan (with `--all-targets` intact at lines 666/1361/1409) as commit 1 under subject `docs(plan): v1-AD-c rule-set CRUD + snapshot wiring + carry-forward fixes (#77, #78)`. This inverts the intended shape.

**Why Impl's shape fails:**

- CR reads commit 1 as "introduces gate that never greens" (because `--all-targets` baseline = 135 errors); commit 2 as "quietly removes the gate that commit 1 just added". Both are bad readings; neither reflects the actual decision trail.
- If commit 2 is ever dropped (rebase mishap, partial cherry-pick, CI squash), commit 1 leaves the branch with a plan stating an unachievable DoD. Poison for any resume-from-partial-merge scenario.
- The audit-visible advantage of the 2-commit shape is lost — a reviewer grepping `git log --oneline -- v1-admin-dashboard-c.plan.md` sees "add plan" then "narrow DoD", rather than "add narrowed plan" then "add audit report".

**Corrected shape:**

1. **Commit 1 (the narrowing, visible alone)** — subject `docs(plan): v1-AD-c plan + narrow clippy DoD to match v1-AD-b ratchet (drop --all-targets)`. Content = the plan WITH the 5 edits applied. Body explains the ratchet narrowing with the v1-AD-b precedent + baseline=135 reference.
2. **Commit 2 (the audit report)** — subject `docs(plan): Task 0 pre-phase audit report`. Content = ONLY the new `.claude/PRPs/reports/v1-AD-c-task0-audit.md` file.

This shape differs from my original instruction — I originally said commit 1 = "narrowing alone, commit 2 = plan + audit". On reflection: the plan didn't exist on disk before this sub-phase, so there's no "before narrowing" state to preserve in git. Adding the plan AND narrowing it in the same commit, with the commit message making the ratchet decision explicit, is the correct compromise. Commit 2 remains audit-report-only.

**Impl actions to correct:**

1. `git reset HEAD .claude/PRPs/plans/v1-admin-dashboard-c.plan.md` — unstage current commit 1 content
2. Apply the 5 edits per sites 1-5 above to the working-tree plan
3. Verify: `grep -n 'all-targets' .claude/PRPs/plans/v1-admin-dashboard-c.plan.md` → expect 1 remaining match (line 666, the baseline-capture command we intentionally keep)
4. Re-run narrowed Level-4 clippy: `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/audit-clippy-baseline-narrowed.log 2>&1"; echo "exit: $?"` → expect 0
5. Stage the narrowed plan as commit 1 with corrected subject + body
6. Write `.claude/PRPs/reports/v1-AD-c-task0-audit.md` (lint distribution table + probe exit codes + requires_re_jury=7 verification)
7. Stage audit report as commit 2
8. Log `i | t0 | stage | c1=docs(plan)-narrow c2=docs(plan)-task0-audit` and wait for `a | t0 | review-go`

**Commit 1 message body template:**

```
Introduces v1-AD-c sub-phase plan (rule-set CRUD + case-open snapshot +
#77/#78 carry-forward fixes) AND narrows the clippy DoD from the drafted
--workspace --all-targets --features full --no-deps to --workspace
--features full --no-deps, matching v1-AD-b's Level 6 ratchet precedent.

Why narrowed: pre-phase audit (.claude/audit-clippy-baseline.log) recorded
135 clippy errors under --all-targets at v1-AD-c base. All 135 are pre-
existing governance-fork test-code debt (tests/e2e.rs + reputation_snapshot
cfg(test) + admin_config parity helper) unrelated to v1-AD-c scope.
Distribution: 56 tests_outside_test_module + 34 indexing_slicing + 27
items_after_statements + 5 expect_used + 2 unwrap_used + 11 misc. v1-AD-b's
Level 6 shipped green without --all-targets; v1-AD-c maintains that ratchet.

Task 8 captures --all-targets delta vs baseline 135 as an advisory check
(non-DoD). Phase-close opens chore(lint) tracking issue per runlog decision.

Plan file pre-narrowed at site 2 (§15 Level 4) and site 3 (§16 acceptance
checkbox 3) to drop --all-targets. Site 1 (§13 Task 0 step 5, line 666)
retains --all-targets because the purpose is baseline-capture. Site 4
(§13 Task 0 OUTPUT) extended to include lint distribution. Site 5 (§13
Task 8 VALIDATE) adds advisory delta-capture command.
```

**Commit 2 message body template:**

```
Task 0 pre-phase audit per .claude/rules/pre-phase-harness-audit.md.

Probes:
- Probe 1 (-p scoping): exit 0 (5m 51s) — wrapper honors -p
- Probe 2 (--features full): exit 0 (29s incremental)
- Probe 3 (e2e test target build): exit 0 (2m 10s)
- Probe 4 (negative feature): exit 101 — wrapper propagates cargo failure
- Level-4 clippy (narrowed): exit 0 — production-code gate green
- --all-targets baseline: exit 101, 135 errors — recorded as baseline N

CONFIG_KEY_METADATA.requires_re_jury = 7 rows; key names byte-identical to
plan §13 task 5 REQUIRES_RE_JURY_KEYS constant. No drift.

v1-AD-b surfaces verified: get_int_opt, admin_set_config, applied_config_
snapshot column, rule_set_version_id column, RuleSetVersion struct,
RuleSetVersionInsertForm struct — all present on f03ed1cba.
```
