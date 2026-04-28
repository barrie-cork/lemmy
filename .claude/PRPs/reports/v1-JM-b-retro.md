# v1-JM-b retro — what worked, what didn't, advisor-actionable follow-ups

**Sub-phase**: v1-JM-b (Jury-mechanics handler — diversity-aware panel selection + severity/status cascade + R1/R2/R3 relaxation cascade)
**Branch**: `phase-v1-JM-b`
**Base**: `governance-v0` @ `02189988d` (post-JM-a-merge tip; same base as JM-a)
**Dates**: 2026-04-23 (plan + tasks 0–4) → 2026-04-24 (handover, advisor plan-drift fix) → 2026-04-25 (tasks 5–10, this retro)
**Impl sessions**: three. Session 1 (2026-04-23) wrote plan + tasks 0–3. Session 2 (2026-04-24) executed task 4, wrote handover. Session 3 (2026-04-25) resumed from handover, executed tasks 5–10.
**Commits**: plan + 9 task commits + JM-a-drift chore + governance-v0 trunk-merge + advisor plan-drift fix = 12 commits ahead of `governance-v0`.

---

## TL;DR for the advisor

**Plan-faithful execution with one significant test-fixture surprise (small-pool fallback obscures steady-state constraint record values).** All Tasks 1–9 shipped; all 8 new e2e tests green; cargo check / clippy / full e2e suite all exit 0. The `compute_status_tier` Founder/Probation/Regular cascade lit up correctly under e2e exercise. `severity_tier_frozen` emission landed once per assign-jury, with all the snapshot fields populated. The JSONB `selected_under_constraints` payload shape matches PRD §5.1 verbatim.

Three plan-drift surprises during execution; all three resolved without advisor blocking:

1. **Task 5 — `as_conversions` lint on f64 → i32 ceil narrowing.** Plan §10.4 GOTCHA flagged this exact class but the inline `try_from(... as i64)` shape still tripped the lint. Resolved with a `ceil_count` helper bounded by the PRD §10 panel_size range + `is_finite`/non-negative guards, wrapped in matching `#[expect(clippy::as_conversions, clippy::cast_possible_truncation)]`. Mirrors the existing `geographic_diversity_score` pattern at `admin_assign_jury.rs:708`.
2. **Task 7 — small-pool fallback obscures constraint-record assertion.** Test 4 (selected_under_constraints JSONB shape assertion) initially failed because, with no `reputation_snapshot` rows seeded, the strict eligibility query under-fills, R1 fires, and the constraint record records `relaxed_small_pool` instead of `applied`. Resolved by adding a `seed_jury_eligible_snapshots` fixture helper and seeding before the assign call. Steady-state assertion now meaningful.
3. **Task 7 — JM-a-drift bleed-through into e2e.rs.** Three pre-existing `JuryAssignmentInsertForm` literals in e2e.rs (lines 917, 3170, 3811) were missed by the `cb7b6bdb2 chore(v1-JM-a-drift)` commit and lacked the `selected_under_constraints` field. Without these, the e2e binary doesn't compile and Task 7 validation can't run. Folded mechanically into the Task 7 commit with `selected_under_constraints: None` to match the v0 pre-cascade shape.

Plus, the handover from Task 4 → Task 5 was clean: session 3 cold-resume sequence detected one expected drift (advisor's `ab1868137` plan-DoD-normalization commit between handover write and resume), which was the exact note the handover flagged at line 93. Zero re-do risk; the resume brief did its job.

---

## 1. What worked — keep doing

### 1.1 Handover brief at task-4 → task-5 boundary

Session 2 closed at the Task 4 commit `44cd12c21` and wrote a 144-line handover at `.claude/PRPs/handovers/impl-2026-04-25-task5-boundary.md`. Session 3 cold-resumed in <1 minute: read CLAUDE.md + `.claude/rules/*.md`, read the handover, executed the cold-resume sequence's git/DQ/state checks, and reported the TL;DR to the user. The handover's "Closing state assertions" section was load-bearing — it caught the one expected drift (advisor's plan-DoD-normalization commit at `ab1868137`) in one line of git output.

**Keep**: handover briefs at natural task boundaries (multi-task gaps, context-window resets). The structure from `.claude/rules/handover.md` plus the existing exemplar at `.claude/PRPs/reports/v1-AD-c-advisor-resume-state.md` mapped directly onto JM-b's needs. Time spent writing the handover (~15 min) saved ~30 min of cold-resume confusion on the receiving side.

### 1.2 Plan §10 pattern-snippet discipline

Same outcome as JM-a §1.1 — every §10 block had a usable Rust snippet that mapped near-1:1 onto the final code. The §10.3 `compute_status_tier` block named `sponsor_liability.rs:217-228` as the MIRROR; that mirror was real, the pattern was correct, and the only adjustment at write-time was using the file-wide `moderation_case::col.eq(...)` idiom instead of the plan's `mc::col.eq(...)` alias (a stylistic choice, not a correctness issue).

The §10.5 plan example folded the snapshot writes + status flip into one UPDATE ("one round trip instead of two"). That worked verbatim — single Diesel `update().set(tuple).execute()` call. The §10.6 `severity_tier_frozen` block named `Some(admin_pseudonym.clone())` as the actor (with explicit GOTCHA "NOT None"), which matched the implementation exactly.

**Keep**: §10 file:line MIRROR refs. Plans without them cost 10+ minutes of pattern-hunting per task.

### 1.3 Cold-resume task-detection

`/prp-core:prp-implement` Phase 1.4 task-detection (per DQ #42 v1-AD-d retro §2.1) ran at session 3 start. It correctly identified Tasks 1–4 as ALREADY DONE on the branch and Task 5 as STARTING HERE. Zero re-do risk. The `git log governance-v0..HEAD --oneline` scan against the plan's §13 COMMIT MESSAGE lines has paid for itself on this phase: it disambiguates "8 commits ahead, of which 4 are tasks 1–4 verbatim, 1 is JM-a-drift, 1 is trunk-merge, 1 is plan-cherry-pick, 1 is advisor's DoD-normalization" without me having to reason about which is which.

**Keep**: the task-detection guardrail. It composes well with the handover protocol — handover names the next task, task-detection verifies the prior tasks landed correctly.

### 1.4 Workspace clippy `--no-deps` after every task

The advisor's `ab1868137` plan-DoD-normalization commit codified `--no-deps` as the canonical clippy invocation across all per-task validation blocks. This eliminated a class of false-red where clippy on upstream Lemmy crates would report pre-existing lint debt we couldn't fix. Session 3 ran this exact clippy command after every task and got clean exits at each gate.

**Keep**: `--workspace --features full --no-deps` as the canonical clippy DoD shape. Document it in the plan template's §15 (already done implicitly via this phase's plan).

### 1.5 Direct handler invocation in e2e tests

Task 7's tests call `admin_assign_jury(Json(AdminAssignJury { case_id }), context, admin_view).await?` directly rather than building an actix `App` test client. This matches DQ #9's precedent (`feedback_test_impact_verification_before_patching` adjacent — direct invocation is the deterministic shape). Tests stay short (50–100 lines each) and assertions hit DB state directly via `AsyncPgConnection` queries.

**Keep**: direct handler invocation pattern. The actix `App` test-client adds 30+ lines per test and obscures DB state assertions.

---

## 2. What surprised — advisor-actionable for future JM plans

### 2.1 Task 5 — `as_conversions` lint on f64 → i32 ceil narrowing

**Observed (Task 5)**: Plan §10.4 GOTCHA explicitly flagged the i32 narrowing risk but used the inline pattern `(panel_size as f64 * fraction).ceil() as i32`. That pattern trips clippy's `as_conversions` lint under the workspace `-D warnings` gate. The natural `try_from` workaround `i32::try_from(quorum_f64.ceil() as i64)` STILL trips `as_conversions` on the inner `as i64`.

**Resolution (in-channel)**: extracted a `ceil_count` helper that uses an explicit `#[expect(clippy::as_conversions, clippy::cast_possible_truncation)]` attribute citing the PRD §10 bound (`panel_size ∈ [3, 11]`, fractions ∈ `[0, 1]`, so the result is always in `[0, 11]` and fits i32 trivially). Includes `is_finite()` + non-negative guards as defence against malformed config. Mirrors the existing `geographic_diversity_score` pattern at `admin_assign_jury.rs:708-712` (which has the same lint situation with `as f64`).

**Root cause**: plan §10.4 GOTCHA wording presented `as i32` as the intended idiom but the workspace lint config rejects bare `as` casts. The plan author wrote the snippet against the (correct) PRD math but didn't run it through clippy.

**Fix — plan amendment for JM-c/d/e and future cascade-resolution tasks**: when a plan §10 snippet needs an `as` cast, the snippet should EITHER use a guarded helper from the existing codebase (mirror `geographic_diversity_score`) OR be wrapped in an explicit `#[expect(clippy::as_conversions, ...)]` block in the snippet itself. The "expand from f64 → i32 via try_from" suggestion in the GOTCHA is misleading — `try_from` doesn't take f64 in stable Rust, and the implicit `as i64` trips the lint.

### 2.2 Task 7 — small-pool fallback obscures constraint-record steady state

**Observed (Task 7 test 4)**: Test 4 asserts every `jury_assignment.selected_under_constraints` row matches the steady-state shape `{cluster: applied, geographic: applied_soft, cooldown: applied, endorsement_chain: disabled}`. Initial run failed: `cooldown` was `relaxed_small_pool` instead of `applied`.

**Root cause**: when no `reputation_snapshot` rows exist, the strict eligibility query under-fills (returns 0 rows). R1 fires, drops cooldown, re-runs strict — still 0 rows. `fallback_on_small_pool=true` is the JM-a default seed, so legacy fallback fires and seats the panel from `accepted_application=true` persons. The constraint record reflects the cascade that actually fired (R1 relax + legacy_fallback), not the steady state.

The other Task 7 tests (1, 2, 3, 5) pass without snapshots because they only check panel size + snapshot fields + governance_log existence, all of which fire downstream of the legacy-fallback path. Test 4 is the first one to inspect the constraint-record contents directly.

**Resolution (in-channel)**: added `seed_jury_eligible_snapshots` fixture helper (inserts `reputation_snapshot { jury_eligible: true, .. }` for every juror), called it before `admin_assign_jury` in test 4. With snapshots seeded, strict path succeeds, no relaxation fires, constraint record matches steady state.

**Fix — plan amendment for JM-c/d/e and future tests asserting constraint-record contents**: §14 task wording for any test that asserts on constraint-record steady-state values should explicitly say "requires `reputation_snapshot` seeding to bypass the small-pool fallback path." Without that hint, the test author may write the steady-state assertion against a code path that fires R1 + legacy_fallback. The fixture pattern (helper + call before handler invocation) is now in `v1_jm_b_fixtures::seed_jury_eligible_snapshots` and can be reused.

### 2.3 Task 7 — JM-a-drift bleed-through into e2e.rs test fixtures

**Observed (Task 7 compile)**: After adding the new tests, `cargo test --no-run -p lemmy_server --test e2e` failed with three `E0063: missing field selected_under_constraints in initializer of JuryAssignmentInsertForm` errors at lines 917, 3170, 3811 of `crates/server/tests/e2e.rs`.

**Root cause**: the `cb7b6bdb2 chore(v1-JM-a-drift)` commit added `selected_under_constraints: Option<Value>` to the `JuryAssignmentInsertForm` struct. That commit's body says "JM-a Task 5 (commit fdbe7f25c) shipped the read-side field on JuryAssignment and the Nullable<Jsonb> column on schema.rs but missed the InsertForm." The chore added the field to the struct but didn't sweep test fixtures — three pre-existing JuryAssignmentInsertForm literals in e2e.rs still lack the field. This makes `cargo check --workspace --features full` pass (the struct is internally consistent) but `cargo test --no-run` fail (because the test fixtures don't initialize the field).

The latent failure was invisible until Task 7 first ran the e2e binary build.

**Resolution (in-channel, folded into Task 7 commit)**: added `selected_under_constraints: None` to all three literals. Single mechanical change per site; commit body documents the JM-a-drift bleed-through.

**Fix — plan amendment for JM-c/d/e and future struct-extension chores**: when a `chore(<phase>-drift)` commit extends a struct, the chore's commit message should call out which crates the struct is used in (here, `lemmy_db_schema` defines + `lemmy_server` test fixtures consume) and the chore's commit should sweep ALL consumers, not just the canonical writer. The fix is adding a `git grep` step to the chore-commit checklist:

```bash
git grep -l '<StructName> {' -- ':!target' ':!.git'
```

If a chore extends a struct, every file in that grep output must be updated in the same commit. (For `JuryAssignmentInsertForm`, `git grep` would have surfaced e2e.rs and the chore would have caught the latent error before merge.)

### 2.4 Test name "R1" lowercased to "r1" for non_snake_case lint

**Observed (Task 8)**: Plan §14 Task 8 names the test `admin_assign_jury_small_pool_triggers_R1_relaxation` (verbatim). Rustc warns `function should have a snake case name` and under `-D warnings` (clippy invocation) this would escalate to error.

**Resolution**: lowercased to `admin_assign_jury_small_pool_triggers_r1_relaxation`. Docstring + log assertions retain the "R1" naming verbatim per plan.

**Fix — plan amendment for future test-naming**: plan task wording should pre-lowercase any acronym-style identifiers in test names. Low priority (one-line fix; obvious in-channel resolution) but worth noting if the plan template ever auto-generates test names from PRD references.

### 2.5 Plan §14 Task 5 commit-message vs handover-message divergence

**Observed (minor, not a blocker)**: The handover line 50 says the Task 5 commit subject should be `feat(v1-JM-b): compute_status_tier + process_assignment severity/status cascade + severity_tier_frozen emission (task 5)`, but the plan §14 Task 5 COMMIT MESSAGE says `feat(v1-JM-b): severity/status-aware process_assignment + snapshot writes + severity_tier_frozen emission (task 5)`. I followed the plan §14 line because the plan is authoritative.

**Resolution**: no action — plan is authoritative. Handover line 50 is a paraphrase; future handovers should quote-paste plan §14 COMMIT MESSAGE verbatim to avoid the appearance of divergence.

---

## 3. What to carry forward

### 3.1 Follow-up GH issue sketches (v1.5 / v2 candidates per DQ #46)

Per plan §19 + DQ #46:

| # | Title | Label | Body (draft) |
|---|---|---|---|
| 1 | Wire `no_same_endorsement_chain` constraint into select_eligible_jurors | `v1.5-candidate` | Carry-forward from v1-JM-a §3.1 row 2. JM-b's `ConstraintRecord::no_same_endorsement_chain` is hardcoded to `"disabled"` — the field is shipped but the constraint is not consulted in the cascade. PRD §5.1 lists it as a soft constraint. v1.5 wiring requires walking the endorsement graph (depth ≤3) at panel-selection time. Code: `admin_assign_jury.rs::select_eligible_jurors`. |
| 2 | Composable diversity constraint priority via `jury.constraint_priority_list` | `v1.5-candidate` | Carry-forward from v1-JM-a §3.1 row 1. JM-b cascades R1→R2→R3 with hardcoded priority. v1.5 makes the priority configurable per community via a new `jury.constraint_priority_list TEXT[]` config key, so communities with stronger sockpuppet risk can drop cluster-diversity last instead of third. Code: `admin_assign_jury.rs::select_eligible_jurors`. |
| 3 | Replace small-pool fallback with explicit operator notification | `v1.5-candidate` | The current `jury.fallback_on_small_pool=true` default silently falls back to the Phase 4 unfiltered shape when the strict pool under-fills. This is correct v0 behaviour but obscures genuine reputation-pool problems on bootstrapping instances. v1.5 should replace the silent fallback with (a) a `governance_log.entry_kind = 'jury_pool_under_filled'` warn entry, (b) a per-community alert for the operator, (c) a `jury.fallback_on_small_pool` change to `notify` instead of `true` as the bootstrapping-friendly default. Code: `admin_assign_jury.rs::select_eligible_jurors` legacy-fallback branch. |
| 4 | Resolve OQ-V1-JM-07 — general case-open severity_tier writer | `v1.5-candidate` | Carry-forward from DQ #47 (planner-pending). v1-JM-b ships the pick-time severity cascade but does NOT add a case-open severity_tier writer for non-emergency paths (`create_report`, `threshold_met`, etc.). Those paths inherit the JM-a DEFAULT 'Minor'. Three options leaned in DQ #47: (a) hardcoded reason_code → severity_tier table, (b) per-community policy under governance_config, (c) reporter-facing DTO field with admin-override. Awaiting advisor decision; v1.5 implements the chosen option. |

Do NOT auto-file. User / advisor decides post-merge which to actually open.

### 3.2 Plan amendments for JM-c/d/e

| # | Amendment | Source retro item | Priority |
|---|---|---|---|
| 1 | Plan §10 snippets that use `as` casts on numeric narrowing should EITHER mirror an existing guarded helper OR include an explicit `#[expect(clippy::as_conversions, ...)]` in the snippet body. The "use try_from" generic suggestion is insufficient because `try_from` doesn't accept f64 directly. | §2.1 Task 5 | High (every cascade-using sub-phase will hit this) |
| 2 | Plan §14 task wording for tests that assert on constraint-record steady-state values should explicitly require `reputation_snapshot` seeding to bypass the small-pool fallback. | §2.2 Task 7 | High (every JM-c/d/e e2e test inspecting constraints will need this) |
| 3 | Chore commits that extend a struct should include a `git grep` sweep step in the commit checklist. Sweep all consumers, including test fixtures. | §2.3 Task 7 | Medium (relevant to any cross-crate struct extension; JM-c/d/e likely add fields to `JuryVoteInsertForm` and `AppealRequestInsertForm`) |
| 4 | Plan §14 task wording should pre-lowercase acronym identifiers in test names (e.g. `r1` not `R1`). | §2.4 Task 8 | Low (one-line fix when noticed) |
| 5 | Handovers should quote-paste plan §14 COMMIT MESSAGE verbatim, not paraphrase. | §2.5 | Low (cosmetic) |

### 3.3 Handoff notes for JM-c (`submit_jury_vote.rs`)

- **Cascade helpers are live**: `config::get_int_cascade` and `config::get_float_cascade` shipped in JM-b Task 2 + are exercised by JM-b Task 5 + Task 7 test 6 (cascade walk). JM-c can read them directly for `jury.threshold_fraction.<severity>` cascade.
- **Snapshot fields are populated**: every case post-JM-b has `panel_size_snapshot`, `quorum_snapshot`, `threshold_count_snapshot` populated at jury-assemble time. JM-c's quorum-aware threshold check should read `case.quorum_snapshot` directly, NOT re-cascade. The snapshot is frozen per ADR-010 — mid-flight config changes should not affect in-flight cases.
- **Severity_tier writes**: JM-b only writes severity_tier from `admin_emergency_remove` (Severe). All other case-open paths inherit JM-a DEFAULT 'Minor'. JM-c must NOT extend severity_tier writes — that's OQ-V1-JM-07 / v1.5 territory.
- **ConstraintRecord shape**: JM-c does not consume `selected_under_constraints` (it's a write-once-on-assign field). JM-c's `submit_jury_vote` reads from `jury_assignment.status = Accepted` and writes `jury_vote` rows; the constraint-record JSONB stays untouched.
- **ENTRY_KIND consts to use**: existing v0 kinds — `case_decided`, `sanction_created`, `public_log_published`, `jury_voted`. JM-c does NOT introduce new ENTRY_KIND consts.
- **No handler files were edited in JM-b outside `admin_assign_jury.rs` and `admin_emergency_remove.rs`.** `submit_jury_vote.rs`, `request_appeal.rs`, `accept_jury_assignment.rs` are clean for JM-c/d.

### 3.4 Outstanding OQs

- **DQ #47 (planner-pending)** — OQ-V1-JM-07 (general case-open severity_tier writer). Does NOT block JM-c/d/e. Awaiting advisor decision; v1.5 candidate.
- All other OQs from JM-a remain in their post-JM-a state (no JM-b activity on them).

---

### 3.5 Task 9 full-workspace validation pass

Plan §14 Task 9 spec'd a `chore(v1-JM-b): full-workspace validation pass` commit, but the plan note also says the commit "amends only the validation log files in `.claude/PRPs/debug/`; no production source changes." Those log files are `.gitignore`d (`.claude/.gitignore:2: *.log`), so there is nothing to commit at this gate. The validation logs exist locally and the audit trail is captured here. Skipping the empty marker commit; rolling Task 9's gate output into Task 10's retro commit.

Validation results:

| Gate | Command | Result |
|---|---|---|
| Static | `cargo-check.bat --workspace --features full` | exit 0, 1m 15s |
| Lint | `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings` | exit 0, 1m 49s |
| Tests | `cargo-test.bat --test e2e -p lemmy_server` | **55 passed, 0 failed, 3 ignored** in 22m 06s |

Local logs:
- `.claude/PRPs/debug/v1-JM-b-task9-check.log`
- `.claude/PRPs/debug/v1-JM-b-task9-clippy.log`
- `.claude/PRPs/debug/v1-JM-b-task9-e2e.log`

The 3 ignored tests are pre-existing GH #43 carriers (`#[ignore]` on the phase1_migrations_round_trip tests; v1-JM-a's R10.1 retro item §2.3 for context). Not JM-b's concern.

The 8 new JM-b tests (6 from Task 7 + 2 from Task 8) all green inside the 55-pass result. Test names confirmed in the e2e log:

- `admin_assign_jury_severity_tier_regular_minor_panel_5_jurors`
- `admin_assign_jury_severity_tier_regular_severe_panel_7_jurors`
- `admin_assign_jury_severity_tier_founder_severe_panel_9_jurors`
- `admin_assign_jury_writes_selected_under_constraints_jsonb`
- `admin_assign_jury_emits_severity_tier_frozen_governance_log`
- `config_get_int_cascade_resolves_founder_severe_to_bare_then_const`
- `admin_assign_jury_small_pool_triggers_r1_relaxation`
- `admin_emergency_remove_case_has_severity_tier_severe`

---

## 4. What did NOT need fixing (worth preserving)

- **`cargo-output-capture.md` + `no-cargo-output-paste.md`**: zero exit-code masking incidents. Every long cargo run went through `> .claude/PRPs/debug/v1-JM-b-task*.log 2>&1; echo "exit: $?"; tail -N`.
- **Phase-branch discipline**: zero direct commits to `governance-v0` from JM-b sessions. All 12 commits land on `phase-v1-JM-b`. BM session opens the PR.
- **`docker ps` preflight**: ran at session 3 start and before every `cargo test --test e2e` invocation per DQ #44. Zero Docker surprises (daemon was up across all three sessions).
- **DQ attribution discipline**: zero `answered_by: "advisor"` writes from impl. DQ #49 self-resolved at Task 4 with `answered_by: "impl-self-resolved"` per `.claude/rules/decision-queue.md:77-98`.
- **Plan §18 risks table**: the §10.4 GOTCHA risk (i32 narrowing lint) materialised exactly as the row predicted (rated Medium); the resolution path was the alternative the GOTCHA suggested.

---

## 5. Quantified outcomes vs confidence score

Plan's §20 predicted **8/10** confidence for one-pass implementation success. Actual: **8.5/10 in hindsight**. The three plan-drift items (§2.1 / §2.2 / §2.3) each cost ~10 minutes of in-channel resolution. No test re-roll for non-mechanical reasons; no compile failure that wasn't immediately fixed. Session 3's Task 5 → Task 9 took ~3.5 hours wall-clock including all validation runs, which matches the plan's prediction.

If §3.2 amendments 1 + 2 + 3 land in the JM-c/d/e plans, those sub-phases should hit **9+/10** confidence. The §10 snippet discipline + §14 task wording + chore-commit grep-sweep are all proven now.

---

## 6. Tool-use self-assessment

### 6.1 Tools used heavily this phase (session 3)

- `Read`: ~30 reads. Plan file (§§10.3–10.6, 10.11–10.13, Task 5–10 blocks), handover, `admin_assign_jury.rs` (full file at session start, partial edits later), `admin_emergency_remove.rs`, `e2e.rs` (existing fixtures + admin_config_fixtures precedent), JM-a retro for mirror.
- `Edit`: ~25 edits. Mostly small targeted edits in `admin_assign_jury.rs` (process_assignment rewrite + helpers), `admin_emergency_remove.rs` (1-line + 1-import), `e2e.rs` (test additions + JM-a-drift fixes).
- `Bash`: ~50 calls. git status/diff/log/commit, cargo wrappers (cargo-check.bat, cargo-clippy.bat, cargo-test.bat with various filters), `docker ps` preflight, log-file inspection.
- `Grep`: ~15 calls. Cross-checking enum variants, finding `JuryAssignmentInsertForm` literals (caught the JM-a-drift bleed-through), locating `ENTRY_KIND_*` re-exports in the api shim, finding test-fixture precedents.
- `TaskCreate` / `TaskUpdate`: ~10 calls. Tracked Task 5–10 progress.

Approximate ratio: Read:Edit ≈ 1.2:1. Grep:Read ≈ 1:2. Slightly more writing than JM-a (which had 2:1 read:edit) — reflects the heavier handler-edit weight of JM-b vs JM-a's schema-only weight.

### 6.2 Tools NOT used that would have helped

- **Agent (subagent_type=Explore)**: zero usage in session 3, same as JM-a. Three moments where Explore would have been cheaper:
  - **Pre-Task-7 fixture archaeology** — I did sequential Reads of the golden-path test (line 1059), the `seed_case` precedent (line 3110), and `admin_config_fixtures` (line 4595) to extract patterns. A single Explore could have surveyed all three in parallel and produced a unified pattern summary in one turn.
  - **JM-a-drift detection** — the three `JuryAssignmentInsertForm` literals at lines 917/3170/3811 surfaced as compile errors at Task 7 validation time. An Explore at Task 5 ("verify every JuryAssignmentInsertForm callsite has the new field") would have caught them earlier and let me fold the fix into Task 5 instead of Task 7.
  - **Plan §10 cross-reference** — Tasks 5 spans §§10.3, 10.4, 10.5, 10.6, 10.11, 10.12. A single Explore could have read all six sections + extracted the imports + extracted the GOTCHA list. I read them sequentially.
- Same self-assessment as JM-a: under-using Agent is the biggest tool-use gap. Cost-of-Explore is trivial vs context-burn-of-sequential-reads.

### 6.3 Rule-violation near-misses

- `cargo-output-capture.md`: zero exit-code-masking incidents. All cargo runs redirected to `.claude/PRPs/debug/v1-JM-b-task*.log` with explicit `echo "exit: $?"` capture.
- `no-cargo-output-paste.md`: cargo log tails stayed under 30 lines per inspection. The two cases where I read further (compile-error logs at Task 7 and Task 8) used `head -90` / `head -40` against the file, not raw `cat` of the full log.
- `decision-queue.md §attribution-integrity`: zero DQ writes this session. DQ #49 (resolved at Task 4 in session 2) was correctly attributed `impl-self-resolved`. DQ #47 stays pending (planner-attributed; non-blocking).
- `pm-plugin-hooks-stable.md`: no PM-adjacent code touched. Verified at session start.
- `phase-branch.md`: zero direct commits to `governance-v0`. All 4 new commits (tasks 5, 6, 7, 8) on `phase-v1-JM-b`. BM handles PR.
- `pre-phase-harness-audit.md`: skipped at session 3 because the audit was already run at session 1's Task 0 and re-run is only required at branch-cut, not at mid-phase resume.

### 6.4 Context-management signals

- Session 3 token high-water mark: ~180k at Task 9 wait. Stayed in the safe zone (<200k effective-reasoning threshold per `feedback_context_trim_verify_empirically.md`).
- Re-reads: plan §14 re-read per task (expected — 4 tasks × ~30 lines = load-bearing). Handover read once. PRD not re-read (the §10 snippets in the plan inlined the PRD references I needed).
- Cargo output budget: all long cargo output stayed in `.claude/PRPs/debug/*.log`. Conversation cargo output: ~12 `tail -N` reads averaging ~15 lines each = ~180 lines total in conversation. Slightly over JM-a's 120-line baseline due to the multi-task validation loop, but well under the 200-line caution threshold.

### 6.5 Lessons for JM-c and the plan template

1. **Plan §14 task block should have an "Explore prompt" sub-bullet** for any task that says "verify X across multiple files" — primes the session to reach for parallel Explore instead of sequential Read+Grep.
2. **Compile-time vs build-time vs test-time validation** — `cargo check --workspace --features full` does NOT compile test binaries. `cargo test --no-run -p lemmy_server --test e2e` does. The latter must be in the Task DoD whenever the task touches a struct that test fixtures consume. Plan §14 already specifies it for test tasks; should be added to handler tasks that extend structs (cf. §2.3 Task 7 JM-a-drift).
3. **Fixture helper extraction** — the v1_jm_b_fixtures module mirrors `admin_config_fixtures` (line 4595). It's now at file scope alongside `governance_fixtures`. Future JM-c/d/e tests can reuse `v1_jm_b_fixtures::seed_jury_eligible_snapshots` and `seed_founder_event` directly. No need to re-derive.

---

## 7. CR finding quality (deferred to PR open)

CR has not run on this branch yet — branch is unpushed. CR ingestion + four-bucket triage will happen post-`/bm-pr` per `feedback_pr_review_triage_pattern.md`. This section will be amended after the first CR pass.

---

## 8. Suggested action items for the advisor

In priority order:

| # | Action | Effort | Value |
|---|---|---|---|
| 1 | Apply §3.2 amendment 1 to JM-c/d/e plans: §10 snippets with `as` casts must mirror an existing guarded helper or include explicit `#[expect]`. | 2-line plan-template edit | High (every cascade-using sub-phase will hit this) |
| 2 | Apply §3.2 amendment 2: §14 test wording for constraint-record assertions must require reputation_snapshot seeding. | 1-line plan-template edit | High (every JM-c/d/e e2e test inspecting constraints) |
| 3 | Resolve DQ #47 (OQ-V1-JM-07): pick lean (a)/(b)/(c) for general case-open severity_tier writer. | Decision + plan-stub for v1.5 | Medium (no JM-b/c/d/e blocker; v1.5 candidate) |
| 4 | Apply §3.2 amendment 3: chore-commits extending structs must grep-sweep all consumers including test fixtures. | Process note in `feedback_commit_hygiene_lockfiles_and_task_labels.md` | Medium (catches a class of bug like the JM-a-drift bleed-through) |
| 5 | File §3.1 items 1, 2, 3 as v1.5 candidates — or decide none are worth filing yet. | 5-15 min decision + GH filings | Low-Medium (preserves implementer-context but all three are deferrable) |

Items 1, 2, 4 are copy-paste plan/process edits. Item 3 is a planner decision. Item 5 is a GH filing decision.

---

## 9. For future v1-JM wave sub-phases (v1-JM-c, v1-JM-d, v1-JM-e)

Carry-forward specifics:

- **JM-c (handler work — `submit_jury_vote.rs` quorum-snapshot-aware threshold + deadlock-to-AdminReview + sponsor-liability branch + `appeal_window_expires_at` write)**: snapshot fields are populated by JM-b. Read `case.quorum_snapshot` and `case.threshold_count_snapshot` directly — no re-cascade per ADR-010. First new `appeal_window_expires_at` write at JM-c.
- **JM-d (appeal handler work — `request_appeal.rs`, `admin_trigger_appeal_rejury.rs`, background job)**: `selected_under_constraints` JSONB on JM-b's writes is a v0-spec read for JM-d's appeal-panel-assembled audit. JM-d's panel-re-selection writes its OWN `selected_under_constraints` row per the same shape JM-b shipped. First call sites for `appeal_panel_assembled`, `appeal_decided`, `appeal_rejected`, `appeal_window_expired`.
- **JM-e (capstone test — `v0_case_completes_under_v0_rules_after_v1_config_flip`)**: JM-b's `panel_size_snapshot` / `quorum_snapshot` / `threshold_count_snapshot` write semantics are the load-bearing assertion JM-e builds on. The JM-b test 1/2/3 panel-size matrix is the regression baseline.

No inter-dependency gotchas beyond these; JM-c/d/e each land on their own phase branch with `governance-v0` as base.

---

_Retro author: impl session 3 (2026-04-25). Sessions 1 (2026-04-23) and 2 (2026-04-24) executed plan + Tasks 1–4 + handover. Session 3 executed Tasks 5–10 + this retro. Available for advisor follow-up on §2.1 / §2.2 / §2.3 amendments._
