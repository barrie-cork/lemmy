# v1-JM-a advisor risk register

**Written**: 2026-04-23 by advisor at JM-a impl kickoff
**Pairs with**: `v1-JM-a-advisor-brief.md`
**Purpose**: Anticipated decision points during JM-a impl, keyed to plan task numbers. Pre-seeded advisor leans for each — NOT authoritative answers (those need evidence at the moment the DQ is raised), but starting positions that let the advisor respond faster.

**How to use**: When impl files a DQ during Task N, scan this register for §Task N entries first. If the lean matches the observed situation, apply it + evidence-check. If the lean disagrees with what impl is seeing, that's a signal to investigate before answering.

**Confidence weighting** applied to each lean:
- **HIGH** — plan §N explicitly prescribes or v1-AD-a byte-for-byte precedent makes the answer unambiguous
- **MED** — pattern-consistent with precedent but needs evidence-check at decision time
- **LOW** — genuine uncertainty; advisor should dig before answering

---

## Task 0 — pre-flight audit

### R0.1 — Wrapper probe 4 (negative exit-code) fails

**Symptom**: `scripts\brehon\cargo-check.bat -p lemmy_db_schema --features nonexistent_xyz` returns exit 0 when it should return non-zero.

**Lean (HIGH)**: This is the goto-eof / errorlevel-clobbering bug class from Issue #8 (`feedback_batch_goto_eof_clobbers_errorlevel`). Fix the wrapper in a pre-Task 1 commit; do NOT proceed to Task 1 until probe 4 returns non-zero. If the wrapper was fixed (`e7cad24fd` was the 2026-04-18 fix; verify `exit /b !errorlevel!` pattern is present in the .bat), the bug is NOT regression — check for a second wrapper script missed in the original fix.

**Precedent**: phase-5c risk-reduction Move 7 (`rca-issue-8-cargo-test-exit-code-masking.md`).

---

### R0.2 — `git merge-base` disagrees with current `governance-v0` HEAD

**Symptom**: Impl runs Task 0, finds `git merge-base phase-v1-JM-a governance-v0` does not equal `git log -1 --format=%H governance-v0`.

**Lean (HIGH)**: This happens if the second FF (post-plan-PR-merge) hasn't been run yet, or if trunk advanced after the FF. Two paths:
- (a) Trunk is ahead → BM session runs second `git merge --ff-only origin/governance-v0` on `phase-v1-JM-a`, then impl retries Task 0
- (b) Phase branch has impl commits ahead of where advisor expects → impl already started Task 1+; this is a **serious drift**. STOP and confirm with user before any rewrite.

**Precedent**: no direct AD equivalent; this is a JM-a-specific risk because cherry-pick (`92705f302`) + later-FF means the phase branch may carry a diff vs trunk that is patch-id-identical to an eventual merge commit.

---

## Task 1 — enum migration

### R1.1 — PascalCase vs snake_case variant confusion

**Symptom**: Impl files a DQ asking whether `severity_tier` enum variants should be `'Minor'/'Moderate'/'Severe'` or `'minor'/'moderate'/'severe'`.

**Lean (HIGH)**: PascalCase per plan §4.1 lock-in decision 2 + plan §10.1. Matches the majority governance-enum pattern (CaseStatus, JuryDecision, SanctionAction, CaseSeverity). `MembershipState` is the ONE exception (snake_case) because it's a deferred-enforcement membership-class flag read by shell-script config writers — different use case. JM's three enums are read/written only by handler code → follow majority.

**Evidence to cite**: `crates/db_schema_file/src/enums.rs` — grep `DbValueStyle = "verbatim"` vs `"snake_case"`; `.claude/rules/governance-log-entry-kind-registry.md` for `MembershipState`'s doc comment.

---

## Task 2 — table-column migration + backfill

### R2.1 — `appeal_window_expires_at` backfill edge cases

**Symptom**: Impl asks about cases where `closed_at IS NULL AND decided_at IS NULL` (pre-Decided cases) vs cases with only one of the two set.

**Lean (HIGH)**: Leave `appeal_window_expires_at` NULL for rows where both `closed_at` and `decided_at` are NULL (per plan §10.4 GOTCHA + §8.4). v1-JM-c writes this field when a case reaches `Decided`; pre-Decided v0 cases never entered that path. The COALESCE expression `COALESCE(closed_at, decided_at + INTERVAL '7 days')` naturally returns NULL if both are NULL, so the SQL is self-documenting.

**Evidence to cite**: plan §10.4 up.sql backfill block; plan §11 risk #2 row.

---

### R2.2 — ENUM column fast backfill vs UPDATE

**Symptom**: Impl wonders whether to `ALTER TABLE ADD COLUMN severity_tier NOT NULL DEFAULT 'Minor'` (Postgres 11+ fast-metadata) or `ADD COLUMN NULLABLE` → `UPDATE` → `ALTER SET NOT NULL` (old style).

**Lean (HIGH)**: Fast-metadata per plan §4.1 lock-in decision 3. `NOT NULL DEFAULT 'Minor'` is O(1) on Postgres 11+. Avoid the three-step dance. Precedent: `migrations/2026-04-18-000100-0000_add_person_membership_state/up.sql`.

---

### R2.3 — Backfill UPDATE and idempotency

**Symptom**: Impl asks if the backfill UPDATE should `WHERE panel_size_snapshot IS NULL` or unconditional.

**Lean (HIGH)**: `WHERE panel_size_snapshot IS NULL` per plan §10.4 GOTCHA. Makes down/up cycles idempotent. Enum columns don't need the WHERE (the DEFAULT handles them); integer snapshots do.

---

## Task 3 — Rust enums

### R3.1 — Newtype location for `JuryConstraintViolationLogId`

**Symptom**: Impl asks whether `JuryConstraintViolationLogId` goes in `crates/db_schema/src/newtypes.rs` or `crates/db_schema_file/src/newtypes.rs`.

**Lean (HIGH)**: `crates/db_schema/src/newtypes.rs` (the `db_schema` crate, not `db_schema_file`). Memory `feedback_newtype_locations_lemmy_db_schema_vs_file.md` already established this for `ModerationCaseId`; JM follows suit.

### R3.2 — CAUGHT — Task 3 validate `expect 0` is a plan drift vs Phase 1 precedent

**Symptom observed 2026-04-23 during impl**: Task 3 plan §13 says `cargo-check.bat --workspace --features full # expect 0`. Task 3 adds three enums with `ExistingTypePath = "crate::schema::sql_types::SeverityTier"` etc., but Task 4 (not yet run) is what adds `sql_types::SeverityTier` to `schema.rs`. Therefore Task 3 standalone **cannot** green `--features full` compile.

**Precedent**: Phase 1 commit `083a9f3f9 feat(db-schema-file): add governance enums` deliberately committed with a known interim-failure commit message:

> This commit intentionally fails `cargo check -p lemmy_db_schema_file` — the new enums reference `crate::schema::sql_types::CaseStatus` etc. which do not exist until task 10 regenerates schema.rs via `diesel print-schema`. Task 10 greens this.

**Resolution applied (user-delivered to impl 2026-04-23)**: Follow Phase 1 precedent. Task 3 commits with an explicit fail-note; Task 4 greens. Skip Task 3's plan §13 `expect 0` check or document the non-zero exit as expected.

**Retro carry-forward (Task 11 must capture this)**:

- **Observation**: JM-a plan §13 Task 3 drifted from Phase 1 precedent on the validate step. The drift is low-severity (impl caught it, advisor confirmed via Phase 1 inspection in ~2 min), but if left in subsequent JM-b/c/d/e plans that follow the same "enums first, schema.rs next" split, it would regress the well-established interim-failure pattern.
- **Root cause hypothesis**: JM-a plan author treated the two-step split as purely organisational (Task 3 = edit enums.rs, Task 4 = edit schema.rs) without noting that the `ExistingTypePath` annotation creates a hard compile-time coupling. v1-AD-a precedent, which JM-a was pattern-mirroring, did NOT involve new Rust enums (it added `rule_set` table, not enum types), so the enum-specific interim-failure pattern was not carried forward from Phase 1.
- **Plan-amendment recommendation**: before JM-b is planned, update the plan-template guidance (or the JM-a plan file directly, if still useful) so the Task N validate command for "enums.rs-only" commits documents the known interim-failure mode rather than `expect 0`. Alternative: restructure as "Task 3: combined enums + schema.rs extensions" single commit. Phase 1's split (two commits) is the cleaner precedent — keep the split, fix the `expect 0`.
- **Not a DQ**: impl caught this themselves and the user-delivered answer unblocked within minutes. No queue entry needed. Documented here for Task 11 retro only.

---

## Task 5 — Diesel models

### R5.1 — CAUGHT — Plan §10.7 GOTCHA wording incomplete re: InsertForm call-site impact

**Symptom observed 2026-04-23 during impl**: Plan §10.7 GOTCHA claims "Option<_> → v0/earlier-v1 callers continue to compile without setting them explicitly". Impl hit compile error on `create_report.rs:204` ("missing fields ... and 3 other fields in initializer") — Rust struct literals require every field to be named or `..rest` fallback; `Option<_>` typing doesn't magic away the naming requirement.

**Resolution (user-delivered to impl 2026-04-23)**: Go with (a) — `..Default::default()` at call sites. Key facts verified:
- `ModerationCaseInsertForm` ALREADY derives `Default` (line 77-78 of `moderation_case.rs`) — impl doesn't need to add the derive
- `JuryAssignmentInsertForm` hasn't gained new fields (per plan line 917, role isn't added to InsertForm) — zero call-site changes needed for jury_assignment writers
- Scope reduction: impl's initial list of 5 affected files narrows to 2 production + e2e.rs (admin_assign_jury.rs and decline_jury_assignment.rs build `JuryAssignmentInsertForm`, not `ModerationCaseInsertForm` — verified via grep)
- The one OUT-list file that remains (`admin_emergency_remove.rs`) gets a single `..Default::default()` line — syntax-only, no handler-logic edit, §12 OUT intent preserved

**Retro carry-forward (Task 11 must capture this)**:

- **Observation**: JM-a plan §10.7 GOTCHA wording overloaded "Option<_> → no call-site change" onto a mechanism that actually requires `..Default::default()`. The plan's *intent* (struct derives Default, calls use struct-update syntax) is correct; the *language* implied something Rust doesn't do.
- **Root cause hypothesis**: Plan author likely wrote the GOTCHA mid-way through drafting §10.7 and didn't compile-verify. This is a second Task-N plan drift caught by impl (first was R3.2 Task 3 `expect 0`); both are "plan-author model vs Rust compiler" errors.
- **Plan-amendment recommendation**: before JM-b is planned, update the plan-template guidance so InsertForm-extension GOTCHAs use language like "struct already derives Default; callers use `..Default::default()`" rather than "Option<_> → no call-site change". Pattern-repetition hazard: every future sub-phase that extends an InsertForm will hit this same drift if left as-is.
- **Scope deviation logged**: impl's commit message for Task 5 includes a paragraph naming `admin_emergency_remove.rs`'s single-line struct-update edit as syntax-only, §12 OUT-intent preserved. This is the audit trail.

---

## Task 6 — config.rs extension

### R6.1 — Plan count (27) vs actual count after reconciliation

**Symptom**: Task 8's reconciliation gate counts >27 or <27 rows in `SEEDED_KEYS_WITH_CONSTS` v1-JM-a extension.

**Lean (MED — needs evidence)**: The plan's count of 27 is an estimate (plan §4.1 lock-in 4 + §13 Task 8 reconciliation gate). If actual differs:
- Actual >27 = impl either added legitimate new keys (confirm against PRD §10 defaults matrix — it authoritatively enumerates 27) OR duplicated a key (fix). PRD is authoritative; if impl's count exceeds PRD, impl drifted scope. Answer: trim to 27.
- Actual <27 = impl missed a key from PRD §10. Answer: check PRD §10 row-by-row, identify which row is missing, impl adds it.
- Actual equals 27 but counter disagrees = awk/grep drift. Fix the counting method; the keyset is canonical.

**Escalation rule**: If impl has already committed Task 6 and the count is wrong, amendment requires a **new commit** (not `--amend` on committed code). Say "commit the fix, keep Task 6's original commit intact — this is a Task 6.5 correction, not a Task 6 rewrite."

**Evidence to cite**: PRD §10 defaults matrix; plan §13 Task 8 reconciliation-gate bash block; memory `feedback_plan_drift_metadata_cross_check.md`.

---

### R6.2 — Config key naming: `jury.threshold_fraction.*` vs AD-a's `jury.severity_thresholds.*`

**Symptom**: Impl or CodeRabbit flags `jury.threshold_fraction.minor` (JM-a) as duplicating `jury.severity_thresholds.minor` (AD-a, if it exists).

**Lean (MED — needs evidence)**: Plan §18 risk row 4 anticipates this. Check `SEEDED_KEYS_WITH_CONSTS` v1-AD-a block for `jury.severity_thresholds.*` — if that set exists and is text display strings (not floats), the two namespaces are distinct by intent:
- `jury.severity_thresholds.*` = text display strings (e.g., `"simple majority"`, `"60%"`, `"3/4 supermajority"`) rendered in admin dashboard
- `jury.threshold_fraction.*` = float cascade values (e.g., `0.5001`, `0.6`, `0.75`) read by `submit_jury_vote` at decision time

Answer: both are legitimate, add a comment in `SEEDED_KEYS_WITH_CONSTS` block explaining the distinction. Expected CR response: rebuttal.

---

## Task 7 — seed migration

### R7.1 — Timestamp for seed migration vs enum migration

**Symptom**: Impl asks what timestamp to use for the seed migration.

**Lean (HIGH)**: `2026-04-23-000200-0000_seed_v1_jm_config_keys/` per plan §10.1 header + Task 7 (after `000000` enums and `000100` columns). Diesel's lexicographic ordering is load-bearing.

---

## Task 8 — reconciliation gate

### R8.1 — Gate fails, impl wants to skip

**Symptom**: Impl has committed Task 7 (seed migration) and finds Task 8's awk+grep count doesn't match 27 → impl proposes silently fixing the migration count + continuing.

**Lean (HIGH)**: **HARD STOP**. The reconciliation gate is the load-bearing invariant (plan §4.1 lock-in 4 + §18 risk row 1). Impl must file a DQ BEFORE committing any fix. Advisor decides whether the fix is (a) adjust the count (plan amendment), or (b) find the missing/extra row. Either path requires a fresh commit that fixes the discrepancy with a message naming the root cause.

**Evidence to cite**: plan §13 Task 7/8 block; memory `feedback_advisor_instruction_mismatch_stop_and_ask.md`.

---

## Task 9 — dual-file ENTRY_KIND edit

### R9.1 — Asymmetric commit (schema const without shim re-export)

**Symptom**: Impl commits Task 9 but only edits `crates/db_schema/src/source/governance/governance_log.rs`, missing the `crates/api/api/src/governance/governance_log.rs` re-export. CR or local lint flags `rg -c pub const ENTRY_KIND` mismatch.

**Lean (HIGH)**: Impl re-does Task 9 as a **single new commit** adding the missing file's re-export. Do NOT `git commit --amend` — per CLAUDE.md safety protocol, prefer new commits over amends. Commit message: `fix(v1-JM-a): task 9 follow-up — add missing ENTRY_KIND re-exports in api shim`.

**Evidence to cite**: `.claude/rules/governance-log-entry-kind-registry.md` top-of-file note; plan §18 risk row 3.

---

## Task 10 — e2e migration round-trip

### R10.1 — `PHASE_1_MIGRATION_COUNT` still at 9 (v0-only) vs 13 (post-v1-AD-a)

**Symptom**: Impl reads e2e.rs and finds `PHASE_1_MIGRATION_COUNT = 9` or similar v0-era baseline, not accounting for v1-AD-a's 4 migrations.

**Lean (MED — needs evidence)**: Per plan §18 risk row 7, AD-a may have failed to extend the round-trip count when it shipped its 4 migrations. Two paths:
- (a) **Extend by 7 (3 JM-a + 4 AD-a)**: fix AD-a's drift in the same commit. Only valid if there is clear evidence AD-a was supposed to extend and didn't. Escalates scope beyond JM-a.
- (b) **Extend by 3 (JM-a only)**: accepts AD-a's drift as someone else's problem. Keeps JM-a scope tight.
- (c) **File DQ at Task 10 start**: correct path per plan §18. Advisor answers after checking git log for AD-a era e2e changes.

**Default lean**: (c) — file the DQ. This is exactly the situation plan §18 risk 7 anticipates. If AD-a really did drift, the fix is a **separate `chore(v1-AD-a-drift)` commit**, not a JM-a Task 10 side-effect.

**Evidence to cite**: `git log -- crates/server/tests/e2e.rs` filtered to v1-AD-a commit range; plan §18 risk row 7.

---

### R10.2 — Backfill smoke test assertion count

**Symptom**: Impl asks how many assertions the backfill smoke test should make.

**Lean (HIGH)**: 6 assertions (one per backfilled column) per plan §10.6 + §18 risk row 2 "6 assertions" explicit count:
1. `severity_tier = 'Minor'`
2. `status_tier = 'Regular'`
3. `panel_size_snapshot = 5`
4. `quorum_snapshot = 3`
5. `threshold_count_snapshot = 3`
6. `appeal_window_expires_at = closed_at` (or `decided_at + 7d` for non-closed cases)

---

## Task 11 — retro + follow-up GH issue sketches

### R11.1 — Which v1.5 / v2 candidates to file

**Symptom**: Impl writes the retro with 3 candidate issue sketches per plan §19 Notes line 1540–1545.

**Lean (HIGH)**: All 3 should be **drafted in the retro** (not silently dropped), per DQ #46 resolution. User + advisor decide which to actually file post-merge. Default expectation:
- `v1.5-candidate: composable diversity constraints` — FILE (small scope, clear v1.5 fit)
- `v1.5-candidate: no_same_endorsement_chain activation` — FILE (tight coupling to this PRD; good breadcrumb)
- `v2-candidate: cross-instance jury eligibility` — FILE (referenced in PRD §2 OUT; worth tracking)

Impl writes all 3 sketches; advisor defers the "file vs park" call to user post-merge.

---

## Cross-cutting: scope-creep detection

### RC.1 — Impl edits a handler file in `crates/api/api/src/governance/`

**Symptom**: Impl touches `submit_jury_vote.rs`, `admin_assign_jury.rs`, `request_appeal.rs`, `accept_jury_assignment.rs`, `decline_jury_assignment.rs`, or `admin_emergency_remove.rs`.

**Lean (HIGH)**: **HARD STOP**. Per plan §4.1 lock-in decision 7 + §12 NOT building list + §19 Notes para 3: "A v1-JM-a session that finds itself editing any of these files has drifted scope". Impl must revert those changes (or stash) and confirm with advisor what motivated the edit. In >90% of cases, the answer is "that change belongs in JM-b/c/d — file a DQ + continue without it".

**Evidence to cite**: plan §§4.1/12/19; PRD §17 row 1 scope.

---

### RC.2 — Impl proposes a config-key rename or removal

**Symptom**: Impl wants to rename a `jury.*` or `appeal.*` key, or drop a key from the 27-count set.

**Lean (HIGH)**: **STOP and amend PRD first**, not the plan. The 27 keys are enumerated in PRD §10 defaults matrix — that's the authoritative list. Plan merely mirrors it. Any rename or drop contradicts the PRD and is an ADR-adjacent change (governance keys are part of the user-visible config surface). Escalate to user before proceeding.

---

## Out-of-scope escalation table (when to go past the advisor)

| Symptom | Escalate to | Why |
|---|---|---|
| ADR contradiction (new ADR needed) | User | Per CLAUDE.md: "Contradicting them requires a new ADR, not a quiet edit" |
| Scope beyond v1-JM-a (touches b/c/d/e) | User | Per PRD §17 phasing |
| PRD §10 matrix needs change | User | PRD is upstream of plan |
| Plan §11 file list needs addition | Advisor (that's this file) | Advisor amends plan |
| CI infrastructure issue | User | `feedback_ci_silent_failure_pattern` territory |
| Wrapper script bug | Advisor (plan-amend pre-Task 1 commit) | Per R0.1 precedent |

---

## Revision log

- **2026-04-23 (advisor initial draft)**: Seeded with 14 risk entries keyed to plan tasks 0–11, 2 cross-cutting scope-creep risks, 1 escalation table. Will update with `R<task>.<N>` entries as impl files DQs during JM-a.
