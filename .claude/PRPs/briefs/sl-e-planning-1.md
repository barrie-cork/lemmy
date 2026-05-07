# SL-e planning brief

**Written**: 2026-05-07 by advisor session (this Mac, brehon-fork CWD `/Users/barrie/Developer/lemmy-advisor-sl-c` — sibling worktree on `governance-v0`).
**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/sl-e-planning-1` from `governance-v0` (or local sibling worktree if Junior daemon unavailable). Plan file commits and pushes back to `governance-v0` at finalize.
**Authority anchor**: `v1-sponsor-liability.prd.md` §15 row 5 (`v1-SL-e — e2e test suite`). Per PRD §15 explicitly **depends on Phases 1-4 (SL-a + SL-b + SL-c + SL-d)** — this is the lane-closer. PRD §15 row description: "Three+ branches: revocation-during-window-escapes; restoration-during-window-escapes; window-expiry-fires; backfill-of-mid-flight". The restoration branch is **out of scope** in v1-SL-e per SL-c DQ #145 (restoration producer doesn't ship until restorative-mechanics-v1 PRD lands). SL-e ships the **other three** branches as full-lane e2e tests.

**Lane-closure scope**: SL-e is **tests-only** — no new modules, no new handlers, no new schema, no new ENTRY_KIND consts, no new config seeds. The full producer→consumer path SL-a..SL-d shipped in isolation gets exercised end-to-end here. SL-e's role is to prove the lane works as a single coherent system.

---

## 1. Role + dispatch line

`[role:planning] v1-sponsor-liability-e plan — lane-wide e2e suite (revocation + window-expiry + backfill)`

The actual create-task description is a single line under 100 chars per `.claude/rules/advisor-orchestrator.md` Junior task description template:

```
[role:planning] v1-sponsor-liability-e plan — see .claude/PRPs/briefs/sl-e-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` for sub-phase **v1-SL-e**. The plan covers PRD §15 row 5 minus the restoration branch (deferred to restorative-mechanics-v1 per SL-c DQ #145).

### 2.1 Three concrete e2e test branches (per PRD §15 row 5 minus restoration)

The plan's §13 task list MUST cover all three. Each test = one §13 task with anchor-pattern Edit at file end (per `feedback_junior_worker_e2e_edit_hang.md` — `e2e.rs` is now ~10,500-10,800+ lines post-SL-d).

a. **Revocation-during-window-escapes (full lane)** — exercises SL-d (producer) → SL-c (consumer-escape branch) → SL-b (revocation-side severance). Test flow:
  1. **Producer side (SL-d)**: trigger a jury vote (via `submit_jury_vote` API call) on a case with target having 1+ active sureties. Verify case transitions to `SponsorLiabilityPending` (NOT `Decided`); `grace_expires_at` set; `sponsor_liability_pending` log emitted; deferred-write set respected (no juror reputation events yet).
  2. **Mid-window** (now < grace_expires_at): caller-sponsor invokes `POST /api/v4/governance/endorsement/revoke` (SL-b's handler) with reason. Verify endorsement.revoked_at set; surety.revoked_at set; `endorsement_revoked` log emitted; case transitions to `SponsorLiabilityEscaped` (severance via SL-b's revocation branch); `liability_escape_reason` JSONB matches `{"version": 1, "reason": "sponsor_revoked", ...}`; `sponsor_liability_escaped` log emitted; response's `liability_chain_severed_for_cases` contains case id.
  3. **Consumer side (SL-c)**: drive a manual `run_grace_check_batch` tick (via `BREHON_DISABLE_GRACE_CHECK_JOB=1` + direct call). Verify the now-`SponsorLiabilityEscaped` case is NOT picked up (status no longer `SponsorLiabilityPending`); zero new transitions; zero new log entries.

  This test crosses three sub-phases' code paths and three log-entry types. It's the highest-leverage test in the lane.

b. **Window-expiry-fires (full lane, no revocation, no restoration)** — exercises SL-d (producer) → SL-c (consumer-fire branch). Test flow:
  1. **Producer side (SL-d)**: trigger jury vote on case with target having active sureties. Verify Decided→Pending transition; `grace_expires_at` set to e.g. now+1minute (use Minor severity for shortest grace).
  2. **No mid-window action** — sponsor doesn't revoke; defendant doesn't restore (restoration not shipped anyway).
  3. **Window expires** — sleep or fast-forward `now()` past `grace_expires_at`. (Pure-time tests in e2e.rs may use `tokio::time::sleep` or seed a case with `decided_at` in the past so `grace_expires_at = decided_at + grace_window` is already past at test-start. Latter is cleaner — no real time passage needed.)
  4. **Consumer side (SL-c)**: drive manual `run_grace_check_batch` tick. Verify case transitions to `SponsorLiabilityFired`; per-sponsor `reputation_event` rows written (count matches active sponsor count); `sponsor_liability_applied` log entries emitted (one per sponsor); `sponsor_liability_fired` summary log emitted; case NO LONGER appears in pending-status batch query.

c. **Backfill-of-mid-flight (v0→v1 deployment scenario)** — exercises SL-a (migration backfill) → SL-c (consumer fire/escape on retroactively-pending cases). Test flow:
  1. **Pre-deploy state simulation**: seed a v0-shape case directly into the DB with `status = 'Decided'`, `decided_at = now() - 12 hours` (within the 24h backfill window per PRD §8.4), `target_person_id` with active sureties, sanction inserted, NO `reputation_event` rows for sponsors yet (simulating v0 mid-flight at v1 deploy).
  2. **Apply backfill UPDATE** (the SL-a migration query at PRD §8.4) — invoke programmatically via Diesel UPDATE matching the migration's WHERE clause. Verify case transitions to `SponsorLiabilityPending` with `grace_expires_at = decided_at + 24h` (most-lenient default per ADR-010 won't-disadvantage rule).
  3. **Consumer side (SL-c)**: drive manual `run_grace_check_batch` tick. Verify the backfilled case is picked up; either fired or escaped per the case's actual sponsor state at backfill-tick time.

  This test proves the v0→v1 deploy story end-to-end.

### 2.2 Optional fourth test — admin-overridden status flip

(Planner judgment.) Per PRD §3.3 SL-d's match-site updates allow admins to force-close terminal liability states for ops purposes. SL-e MAY include a test for `admin_close_case` on a `SponsorLiabilityPending` case (verify admin can transition Pending→Closed bypassing the scheduler). If included, it's a 4th §13 task. Pre-estimate doesn't gate inclusion.

### 2.3 Scope boundary — what is NOT in SL-e

- **Restoration-during-window-escapes** — out of scope per SL-c DQ #145 (restoration producer = restorative-mechanics-v1 PRD, not yet drafted). The matching e2e test lives in that future PRD.
- **No code in `crates/api/api/`, `crates/api/api_crud/`, `crates/api/routes/`, `crates/db_schema*/`, `crates/routes/src/utils/scheduled_tasks.rs`** — SL-e is tests-only. SL-a/b/c/d ship the production code; SL-e exercises it.
- **No new migration.** SL-a shipped the schema; SL-e adds zero migrations.
- **No new ENTRY_KIND_*** — all consts shipped in SL-a; SL-e tests assert their emission.
- **No new CaseStatus variants** — SL-a shipped 3; SL-e tests their lifecycle.
- **No new `governance_config` seed** — all 13 SL-a-seeded keys are read by SL-e tests; none added.
- **No new HTTP endpoints, DTOs, routes** — SL-e tests existing endpoints.
- **No retro task changes** — the standard retro task at the end of §13 is the same as prior phases.

**Hard out-of-scope** (per PRD §2 OUT + §15):

- Cross-instance sponsor-liability federation — v2 per ADR-014.
- Step-up auth — v2 reservation.
- Restoration completion endpoint — restorative-mechanics-v1 PRD.
- Notification UX richness — v3 polish.

### 2.4 Plan file deliverable shape

- **One file:** `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md`.
- **Follow `.claude/PRPs/templates/plan.template.md` 20-section schema literally.** §16a Stories mandatory.
- **Shape G applies — SL-e is the fifth plan under Shape G.** §15 DoD MUST use the per-workflow shape. Inline cargo invocations forbidden in §15.
- **§5 complexity score breakdown table** per `feedback_complexity_score_pre_split.md`. Pre-estimate **3-5** (very low — three to four e2e tests, one file modified, no migrations, no new crates touched). Split-or-proceed gate unlikely to fire.
- **§16a Stories** — every story names composing §13 tasks + Shape-G checkpoint + Brief-Scope outputs. Likely **3 stories, one per test branch** (revocation / window-expiry / backfill).
- **§4 watchpoints** — every entry cites a specific file, table, or `schema.rs` line. The 8-watchpoint seed list below.
- **Explicit scope-boundary section in §6** ("Relationship to other v1-SL sub-phases") confirming SL-e doesn't touch production code.

**Commit only the plan file.** Plan-file commit pushes to `governance-v0` after DoD smoke + watchpoint-specificity gates pass and the user approves.

---

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract.
2. `.claude/PRPs/templates/plan.template.md` — canonical Brehon plan template.
3. `.claude/PRPs/prds/v1-sponsor-liability.prd.md` — read in full. Specifically:
   - §1 + §2 (vision/scope).
   - §3 (CaseStatus extensions — SL-e tests assert all 3 lifecycle states).
   - §4 (grace-window durations — SL-e tests assert severity-tier mapping in test #2).
   - §5 (revoke_endorsement endpoint — SL-e test #1 exercises SL-b's handler).
   - §6 (scheduler — SL-e tests #1, #2, #3 drive SL-c's `run_grace_check_batch`).
   - §8.4 (migration backfill — SL-e test #3 exercises the SL-a backfill UPDATE).
   - §9.1 + §9.3 (compute/fire split + submit_jury_vote mutation — SL-e test #1 + #2 exercise the SL-d producer).
   - §10 (defaults matrix — SL-e tests assert default values for the keys it reads).
   - §11.2 + §11.4 (backwards compat — SL-e test #3 covers §11.2 mid-flight cases; SL-e test #1 + #2 verify §11.4 documented behavioural change).
   - §15 row 5 (the canonical scope row).
4. **The 4 prior SL plans, in order**:
   - `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — schema + seeds + backfill (SL-e test #3 exercises the §8.4 backfill).
   - `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` — revoke_endorsement (SL-e test #1 exercises this handler end-to-end via real HTTP, not direct DB-write).
   - `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` — scheduler (SL-e tests #1, #2, #3 drive `run_grace_check_batch`).
   - `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` — producer mutation (SL-e tests #1 + #2 exercise the Decided→Pending transition).
5. **`crates/server/tests/e2e.rs`** — locate (~10,500-10,800+ lines post-SL-d). Read:
   - First 100 lines (test-setup helpers — `setup_e2e_pool`, fixture builders).
   - Most-recent SL-d tests (Decided→Pending transition test patterns).
   - Most-recent SL-c tests (scheduler-driven test patterns; specifically how `BREHON_DISABLE_GRACE_CHECK_JOB=1` is set in fixtures).
   - Most-recent SL-b tests (revoke_endorsement HTTP-call test patterns).
   - Existing endorsement/surety/sanction test helpers.
   - Existing `submit_jury_vote` test helpers (jury fixture seeding).
6. `crates/api/api/src/governance/sponsor_liability.rs` — confirm post-SL-d split:
   - `compute_sponsor_liability` exists.
   - `fire_sponsor_liability` exists.
   - `apply_sponsor_liability` is the thin wrapper.
   - SL-e tests don't call these directly (they call the producer via `submit_jury_vote` API and the consumer via `run_grace_check_batch`); but understanding the split shape clarifies what SL-e should assert (e.g. test #1 expects `compute_sponsor_liability` to be called on the Pending transition, but `fire_sponsor_liability` to NOT be called).
7. `crates/api/api/src/governance/sponsor_liability_grace.rs` (post-SL-c) — confirm `run_grace_check_batch` exists and is callable from tests.
8. `crates/api/api_crud/src/governance/revoke_endorsement.rs` (post-SL-b) — confirm `revoke_endorsement` handler exists; verify the response shape `RevokeEndorsementResponse` SL-e test #1 asserts on.
9. `crates/api/api/src/governance/submit_jury_vote.rs` (post-SL-d) — confirm the producer mutation lives at the expected line; SL-e test #1 + #2 drive jury votes via the public API.
10. `crates/db_schema_file/src/enums.rs` — confirm `CaseStatus::SponsorLiabilityPending`, `Fired`, `Escaped` all defined; SL-e tests assert lifecycle transitions through these states.
11. `crates/db_schema_file/src/schema.rs` — verify `moderation_case` table columns SL-e queries: `status`, `grace_expires_at`, `liability_escape_reason`, `decided_at`, `severity`, `target_person_id`, `community_id`.
12. `crates/api/api/src/governance/governance_log.rs` — confirm all 5 SL-relevant ENTRY_KIND consts exist (SL-e tests assert their emission via row counts in `governance_log` table).
13. `crates/api/api/src/governance/config.rs` — confirm the 13 SL-a-seeded keys; SL-e tests read defaults via `get_int`/`get_float`/`get_bool` to verify production-default behaviour.
14. **Glob `.claude/lessons/feedback_*.md` and Read every file whose name keyword matches:** `e2e_filter`, `e2e_edit_hang`, `junior_worker_e2e`, `complexity_score`, `pre_queue`, `watchpoint`, `dod`, `validate`, `shape_g`, `validate_pending`, `governance_log`, `entry_kind`, `seed`, `cargo`, `features-full`, `features_full_p_crate`, `wrapper-script`, `clippy`, `test-style`, `pre_phase_dod`, `dry_run`, `wrapper_silence`, `principles_not_rules`, `read_canonical`, `parallel_cohort`, `cohort_yaml`, `four_role`, `retro_not_report`, `pre_phase_harness_audit`, `e2e_test_helpers`, `setup_e2e_pool`, `fixture`, `await_consistency`, `tokio_time`, `disable_job_env_var`. That is the lessons-corpus discipline.
15. **Glob `.claude/lessons/reference_*.md` and Read every file whose name keyword matches:** `governance_log_entry_kind_registry`, `phase_branch`, `branch_manager`, `worktree`.
16. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — read **ADR-010** (won't-disadvantage rule — SL-e test #3 verifies the 24h-most-lenient backfill), **ADR-013** (SL-e tests assert exhaustive enum handling), **ADR-015** (pseudonymisation — SL-e tests assert pseudonyms in log payloads, never raw ids).
17. `.claude/PRPs/briefs/sl-a-planning-1.md`, `sl-b-planning-1.md`, `sl-c-planning-1.md`, `sl-d-planning-1.md` — exemplar planning briefs.
18. `.github/workflows/cargo-validate-workspace.yml` + `cargo-validate-features-full.yml` + `cargo-test-e2e.yml` — Shape-G workflows. SL-e does NOT add or edit workflows.
19. **DQ #67 resolved (per user 2026-04-27)** — workflow YAML DoD dry-run discipline.
20. **DQ #145 resolved (advisor 2026-05-07 via /brehon-clarify on sl-c-planning-1)** — restoration-escape branch is stub-only in v1-SL-c; SL-e excludes restoration tests entirely (deferred to restorative-mechanics-v1 PRD).

---

## 4. Constraints (hard rules — violating any of these is a process breach)

### 4.1 Plan-content discipline

- **Shape G DoD shape (forward-only).** §15 MUST use the per-workflow shape per `.claude/PRPs/templates/plan.template.md` §15.6. Inline cargo invocations forbidden.
- **§16a Stories mandatory.** Likely 3 stories, one per test branch.
- **§4 watchpoints cite specific files / handlers / `enums.rs` lines**, never abstract concepts. **The 8 watchpoints below are the seed list:**
   1. **Three independent test branches.** Each test in §13 is its own §13 task with anchor-Edit at file end. No bundling. Per `feedback_junior_worker_e2e_edit_hang.md`.
   2. **Time-handling discipline.** Tests #2 and #3 require `now() > grace_expires_at` semantics. Plan §13 task that authors test #2 MUST specify which approach: (a) `tokio::time::sleep` (real wall-clock), (b) seed a case with `decided_at` already in the past so `grace_expires_at` is already past at test-start. **(b) is preferred** — no real-time delay, deterministic, fast.
   3. **`BREHON_DISABLE_GRACE_CHECK_JOB=1` env var set in tests #1, #2, #3.** Otherwise the cron tick races the test's manual `run_grace_check_batch` invocation. Plan §13 must verify this is set in `setup_e2e_pool` or per-test fixture.
   4. **Real-HTTP for revocation in test #1.** SL-e exercises SL-b's HTTP handler — not direct DB-write. Plan §13 test #1 task must use the HTTP-call test pattern (verify against existing `revoke_endorsement` test fixtures from SL-b's plan).
   5. **Pseudonym discipline assertions.** SL-e tests assert that every emitted governance_log payload uses `*_pseudonym` strings, never raw `*_id`. Plan §13 must specify the assertion shape (e.g. `assert!(payload.get("target_pseudonym").is_some()); assert!(payload.get("target_person_id").is_none());`).
   6. **Backfill-test programmatic invocation in test #3.** SL-e test #3 cannot wait for v1-deploy migration; instead, it programmatically invokes the SL-a backfill UPDATE via Diesel. Plan §13 must specify the exact UPDATE shape (mirror PRD §8.4 SQL).
   7. **No new ENTRY_KIND consts; no new schema; no new module.** Plan §13 must produce zero new files outside `crates/server/tests/e2e.rs`. `git diff governance-v0..phase-v1-SL-e` at plan-approval time should show only `crates/server/tests/e2e.rs` (and `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md`) modified.
   8. **e2e Edit-per-task discipline** per `feedback_junior_worker_e2e_edit_hang.md`. e2e.rs is now ~10,500-10,800+ lines (post-SL-d).

- **§5 complexity score breakdown table mandatory.** Pre-estimate 3-5.
- **e2e edit discipline** per `feedback_junior_worker_e2e_edit_hang.md` + `feedback_e2e_filter_assumes_naming.md`.
- **Cargo feature/flag propagation** per `pattern_cargo_feature_flag_propagation.md` + `feedback_features_full_p_crate_incompatible.md`.
- **R-rule inheritance from JM-a/b/c/d/e + AD-a + SL-a/b/c/d retros** — every R1-R7 from prior retros applies.

### 4.2 Decision-queue discipline

- **Attribution integrity.** `from: "planner"` or `null`. NEVER `"advisor"`, `"user"`, `"impl-self-resolved"`.
- **Mid-task DQ commits push immediately**, not at finalize.
- **Boundary-of-judgment** — when to STOP and queue rather than guess:
  - **Time-handling approach for test #2** — watchpoint #2. If the planner discovers the existing test patterns use one approach exclusively, no DQ; if they use a mix, planner DQ asking which is preferred.
  - **Optional 4th test (admin-overridden status flip)** — §2.2. Planner judgment whether to include; if included, no DQ; if excluded, briefly note in §19.
  - **Backfill UPDATE in test #3** — if SL-a's migration shape changed between SL-a merge and SL-e planning (unlikely), planner DQ.
  - **Restoration test surfacing temptation** — if the planner is tempted to include a restoration-escape test "just to verify the stub", refuse the temptation (per DQ #145, restoration is stub-only in v1-SL-c — testing the stub returns Fire which is what the window-expiry test already covers). Surface as DQ if uncertain.

### 4.3 File ownership

- **Touch only:**
  - `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` (CREATE).
  - `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries).
- **Do NOT edit anything under** `crates/`, `migrations/`, `tests/` (other than `crates/server/tests/e2e.rs` which is the SL-e impl-task target — but the planner doesn't edit it; the impl-task does), `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, `.github/workflows/`, or any other plan file in `.claude/PRPs/plans/`.
- **One commit at finalize:** `feat(plan): v1-SL-e sub-phase plan`.

### 4.4 Schema-first discipline

- **Read `crates/db_schema_file/src/schema.rs` BEFORE designing §13.** Confirm at planning time:
  - `moderation_case.{grace_expires_at, liability_escape_reason, decided_at, severity, status, target_person_id, community_id}` all present (SL-a baseline).
  - `endorsement.revoked_at`, `surety.revoked_at` (Phase 5a baseline).
  - `case_status` enum 12 variants.
  - `sanction` table baseline.
  - `governance_log` baseline.
- If any baseline assumption fails (extremely unlikely at this point in the lane), file a `kind: "blocker"` DQ.

### 4.5 Cross-cutting from PMD-promoted patterns

- **Pattern: `verify_before_trusting_shell_output`** — when the planner runs git-grep for test patterns, verify via direct file read.
- **Pattern: `cargo_feature_flag_propagation`** — `--workspace --features full` only.
- **Pattern: `read_canonical`** — SL-e tests mirror existing e2e test patterns from SL-a/b/c/d. Plan §10 cites the most-relevant existing test as the canonical mirror per branch (e.g. test #1 mirrors SL-b's grace-window-severance tests; test #2 mirrors SL-c's scheduler-fire test; test #3 mirrors SL-a's backfill-coverage test).

### 4.6 Attribution integrity reminder

The only valid `answered_by` labels for a Junior planning subagent are `"planner"` or `null`. Never `"advisor"`, `"user"`, `"impl-self-resolved"`.

---

**Lean / advisor-side tip (not a constraint):** SL-e is the **lane-closer**. After it merges, the SL lane is fully shipped — schema (SL-a) + revocation handler (SL-b) + scheduler (SL-c) + producer mutation (SL-d) + lane-wide e2e (SL-e). The lane-shipping retro (`reports/v1-SL-lane-meta-retro.md` per the AD-lane precedent) follows SL-e's merge.

A second observation: SL-e has the lowest complexity of the SL lane — no new modules, no new schema, no new entry kinds. The work is purely test authoring against an already-shipped production code base. Pre-estimate 3-5 complexity reflects this. Per-task wall-clock under Sonnet 4.6 should be ~5-8 min per test (Edit + cargo check on test target + workspace lint).

A third observation: the three test branches are **highly orthogonal**. They share fixture setup (jury creation, surety creation, sanction creation) but diverge on the action triggered (vote-only / vote+revoke / vote+expire / backfill+expire). Cohort dispatch via `[P]` is plausible — the planner can mark tests #1, #2, #3 as `[P]` if the FILES YAML overlap check confirms each test is a separate anchor-Edit at distinct anchor patterns within `e2e.rs`. (Per `feedback_parallel_cohort_dispatch.md`, the `[P]` marker requires file-set disjointness within the cohort; if all 3 tests Edit the SAME file `e2e.rs`, they're NOT cohort-compatible — same-file Edits race at finalize-merge.) **Default lean: serial dispatch** — 3-4 tests in sequence is fast enough; cohort-dispatch saves ~5 min wall-clock vs 15-25 min serial, marginal benefit at this complexity.

A fourth observation: SL-e's strongest assertion is the **deferred-write semantics from PRD §11.4**. Test #1 must verify that between the jury vote (Decided→Pending transition) and the eventual escape (Pending→Escaped via SL-b's revocation), the modlog endpoint reports "no entry" for the case. This is the documented behavioural change that v1 introduces. Without this assertion, a regression in PRD §11.4 (e.g. a mid-flight code change that re-emits modlog at vote-tally time) would silently break the auditor-visibility contract.

A fifth observation: SL-e is **safely run-after-everything**. Unlike earlier phases that ship code that downstream phases consume, SL-e consumes everything and produces nothing. If SL-e finds bugs in SL-a/b/c/d's behaviour, the fix is a chore-PR on the affected lane (not a re-plan of SL-e). Plan §18 Risks should note: "SL-e's role is to surface lane-integration bugs; if any test fails, the failure is data for an SL-* fix-impl PR, not a re-plan of SL-e itself."

---

_Brief author: advisor session (this Mac CWD `/Users/barrie/Developer/lemmy-advisor-sl-c`, sibling worktree on `governance-v0`, 2026-05-07). Brief committed on `governance-v0` before Junior planning task is queued. Per the advisor-orchestrator clarify gate, advisor runs `/brehon-clarify .claude/PRPs/briefs/sl-e-planning-1.md` after this brief commits and pushes; planning task only queues after every clarify-DQ entry is resolved. Lane progression confirmed 2026-05-07: SL-e scope is lane-wide e2e suite per PRD §15 row 5, restoration-branch test deferred to restorative-mechanics-v1 PRD per SL-c DQ #145._
