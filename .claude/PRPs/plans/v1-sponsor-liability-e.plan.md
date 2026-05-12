# Plan: v1-sponsor-liability-e — lane-wide e2e suite (revocation + window-expiry + backfill)

> **Shape G plan** — SL-e ships under Shape G (Layer G2 push-and-exit).
> §15 references workflow YAMLs by path + expected `conclusion`, not
> inline cargo. Phase 1 (workspace check) runs on
> `cargo-validate-workspace.yml` against each `junior/*` worker branch;
> Phase 2 (e2e on `phase-v1-SL-e`) per advisor's local-vs-dispatch user
> gate (PR #105). See `.claude/PRPs/templates/plan.template.md` §15.6 +
> `.claude/PRPs/plans/v1-validate-agent.plan.md`.

## Table of contents

| § | Heading |
|---|---|
| 1 | Summary |
| 2 | Source |
| 3 | Problem statement |
| 4 | Solution statement |
| 5 | Metadata + complexity score |
| 6 | Relationship to other v1-SL sub-phases |
| 7 | Preflight guardrails inherited from prior phases |
| 8 | Flow design |
| 9 | Mandatory reading |
| 10 | Patterns to mirror |
| 11 | Files to change |
| 12 | NOT building in v1-SL-e |
| 13 | Step-by-step tasks |
| 14 | Testing strategy |
| 15 | Validation commands (DoD — Shape G) |
| 16 | Acceptance criteria |
| 16a | Stories (independently-testable behaviour units) |
| 17 | Completion checklist |
| 18 | Risks and mitigations |
| 19 | Notes |
| 20 | Confidence score |

---

## 1. Summary

v1-SL-e is the **lane-closer** for the v1 sponsor-liability lane. After
SL-a (schema + seeds + backfill UPDATE), SL-b (`revoke_endorsement`
handler), SL-c (`run_grace_check_batch` scheduler), and SL-d
(`submit_jury_vote` mutation + `apply_sponsor_liability` split) shipped
in isolation, **SL-e proves the full producer→consumer path works as
one coherent system** via three lane-wide e2e tests:

1. **Revocation-during-window-escapes** — `submit_jury_vote` (SL-d)
   produces a `SponsorLiabilityPending` case; mid-window
   `revoke_endorsement` (SL-b HTTP) severs the chain; `run_grace_check_batch`
   (SL-c) skips the now-`SponsorLiabilityEscaped` case.
2. **Window-expiry-fires** — `submit_jury_vote` (SL-d) produces a
   `SponsorLiabilityPending` case; `grace_expires_at` is force-rewound
   to the past (deterministic; no real-time wait);
   `run_grace_check_batch` (SL-c) fires the wrapper
   `apply_sponsor_liability` (SL-d split + SL-b/SL-c untouched call
   sites); case transitions to `SponsorLiabilityFired`.
3. **Backfill-of-mid-flight** — a v0-shape Decided case is seeded
   directly (no producer); SL-a's backfill UPDATE is invoked
   programmatically; the case transitions to `SponsorLiabilityPending`
   with `grace_expires_at = decided_at + 24h`;
   `run_grace_check_batch` (SL-c) picks it up and fires.

**SL-e is tests-only — no new modules, no new schema, no new handlers,
no new ENTRY_KIND consts, no new config seeds, no migrations.** Every
production code path it exercises is already on `governance-v0`. SL-e's
deliverable is **observational proof that the lane works end-to-end**.

**Headline acceptance condition.** Stories 1 + 2 + 3 are `[done]` with
their Phase 1 + Phase 2 workflows green: (Story 1) the
revocation-during-window-escapes flow exercises SL-d producer + SL-b
HTTP + SL-c scheduler in a single test and asserts the full state
transition; (Story 2) the window-expiry-fires flow exercises SL-d
producer + SL-c scheduler-driven fire and asserts the full reputation
event + log entry shape; (Story 3) the backfill-of-mid-flight flow
exercises SL-a's backfill UPDATE + SL-c scheduler and asserts the
v0→v1 deploy story.

The restoration-during-window-escapes branch (the fourth scope item in
PRD §15 row 5) is **deferred to restorative-mechanics-v1 PRD** per
SL-c DQ #145 LOCKED. SL-e ships only the three branches whose
producers are on trunk.

---

## 2. Source

- `.claude/PRPs/briefs/sl-e-planning-1.md` @ `governance-v0` — the
  advisor brief (written 2026-05-07 by sibling advisor session). The
  brief is this plan's primary contract.
- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` @ `governance-v0` —
  parent PRD. Specifically:
  - §1, §2 (vision/goals; SL-e ships the lane-wide proof).
  - §3.1 + §3.2 (CaseStatus lifecycle — SL-e tests assert all
    transitions through the 3 v1 variants).
  - §4.1 + §4.3 (severity-proportional grace windows; severity
    snapshot semantics — Test #2 + #3 assert).
  - §5.3 + §5.4 (revoke_endorsement handler effect + idempotency —
    Test #1 exercises through HTTP).
  - §6.2 (`run_grace_check_batch` semantics — Test #2 + #3 drive).
  - §8.4 (migration backfill SQL — Test #3 invokes programmatically).
  - §9.1 + §9.3 (compute/fire split + `submit_jury_vote` mutation —
    Test #1 + #2 exercise the SL-d producer).
  - §10 (defaults matrix — SL-e tests assert defaults the lane reads).
  - §11.2 + §11.4 (backwards compat — Test #3 covers §11.2 mid-flight
    cases; Tests #1 + #2 verify §11.4 documented behavioural change).
  - **§15 row 5 — the canonical SL-e scope row** ("Three+ branches:
    revocation-during-window-escapes; restoration-during-window-escapes;
    window-expiry-fires; backfill-of-mid-flight"). Restoration deferred
    per SL-c DQ #145.
  - §17 (cross-cutting — ADR-013 enum-exhaustiveness; ADR-015
    pseudonymisation; SL-e tests assert in payload shapes).
- `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` @ `governance-v0`
  — most recent shipped sibling under Shape G; canonical mirror for
  §6 relationship table, §13 anchor-Edit pattern, `mod v1_sl_e_fixtures`
  placement convention, §15 Shape G DoD shape, §16a Stories grain.
- `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` @ `governance-v0`
  — second-most-recent shipped sibling (5 e2e tests on
  `run_grace_check_batch`). Canonical mirror for scheduler-driven test
  patterns (`BREHON_DISABLE_GRACE_CHECK_JOB=1` envelope, manual
  invocation of `run_grace_check_batch`, per-case assertion shape).
- `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` @ `governance-v0`
  — canonical Case A error-shape fixture mod (`mod v1_sl_b_fixtures`
  at `crates/server/tests/e2e.rs:10980-11924`). Canonical mirror for
  HTTP-call revocation test pattern (Test #1 uses real
  `revoke_endorsement` handler invocation).
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` @ `governance-v0`
  — schema baseline (3 CaseStatus variants, 2 moderation_case columns,
  13 governance_config seeds, the backfill UPDATE). Test #3 invokes
  the §8.4 UPDATE programmatically.
- `.claude/PRPs/templates/plan.template.md` — canonical 20-section
  schema; §15.6 Shape G DoD; §16a Stories mandatory; §13 FILES YAML
  per task; §5.1 complexity breakdown table.
- `.claude/agents/planning.md` — subagent contract; §13 per-task FILES
  YAML block discipline; §5 complexity score rule;
  canonical-schema-first gate.
- `.claude/rules/decision-queue.md` — DQ schema-v2 attribution
  (planner → `from: "planner"`, never `"advisor"`); Recipe 2
  planner-resolved pre-seed; `kind: "validate-pending"` Shape G
  routing.
- `.claude/rules/advisor-orchestrator.md` — Stage shape under Shape
  G; Phase 2 e2e local-vs-dispatch user gate (PR #105); §G4
  classifier rows 4a/4b/4c (per `feedback_lemmy_error_no_std_error.md`
  amendment 2026-05-09).
- `.claude/rules/branch-manager.md` — file-ownership; BM cuts
  `phase-v1-SL-e` after planner ships.
- `.claude/rules/phase-branch.md` — phase-branch +
  PR-into-`governance-v0` flow.
- `.claude/rules/governance-log-entry-kind-registry.md` — confirms
  all 5 SL-relevant ENTRY_KIND consts are active post-SL-d (the
  `_SPONSOR_LIABILITY_PENDING` row's "(pending)" marker was flipped
  at SL-d retro). SL-e tests assert emission of these consts, adds
  none.
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  — **ADR-010** (won't-disadvantage rule — Test #3 verifies the 24h-
  most-lenient backfill), **ADR-013** (enum-exhaustiveness; SL-e
  test bodies assert via direct equality, never `match _ =>`),
  **ADR-014** (federation deferral; SL-e asserts no outbound fires
  on Pending transition — Test #2 includes this defensive assertion
  shape), **ADR-015** (pseudonymisation — SL-e tests assert
  pseudonym strings in payloads, never raw `*_id`).

### Lessons that bind §13 decisions (SL-e)

- **`feedback_lemmy_error_no_std_error.md`** — load-bearing. SL-e
  authors 3 e2e tests + helpers in a new `mod v1_sl_e_fixtures`
  block in `crates/server/tests/e2e.rs`. The v1-SL-* fixture-mod
  canonical siblings at `crates/server/tests/e2e.rs:10980-11924`
  (`mod v1_sl_b_fixtures`), `crates/server/tests/e2e.rs:11927-12819`
  (`mod v1_sl_c_fixtures`), and `crates/server/tests/e2e.rs:12821-13693`
  (`mod v1_sl_d_fixtures`) use uniform `LemmyResult<T>` throughout
  (Case A per the lesson's amended case enumeration). **§13 Tasks
  1-3 helpers + test fns ALL use `LemmyResult<()>` outer.** No
  `Box<dyn Error>` outer; no `.map_err` bridges. The 2026-05-09
  v1-SL-c-2 3-cycle catch-fire (workflow runs `25582548670` /
  `25595869651` / `25603848858`) proved Case C (mixed shapes) is a
  hard refusal; SL-e follows Case A uniformly. The §13 stubs for
  Tasks 1-3 prescribe `LemmyResult<()>` literally; per-task GOTCHA
  blocks cite the canonical sibling.
- **`feedback_junior_worker_e2e_edit_hang.md`** (referenced via
  `feedback_complexity_score_pre_split.md`) — load-bearing. e2e.rs
  is **13,693 lines** post-SL-d (verified at plan-write time;
  `wc -l crates/server/tests/e2e.rs` = 13693). 3 e2e tests = 3
  separate §13 tasks (Tasks 1, 2, 3), each one anchor-Edit at file
  end. Bundle Edits hang Junior workers.
- **`feedback_complexity_score_pre_split.md`** — SL-e score computed
  mechanically in §5.1 below; tripped threshold (10 > 8); planner
  files DQ #190 self-resolved per Recipe 2 with proceed rationale
  (e2e factor dominates; SL-c-2/SL-b/SL-d/JM-e proceed-as-one
  precedents; SL-e's e2e tests are mechanically anchor-Edit-friendly).
- **`feedback_principles_not_rules.md`** — score is a signal, not a
  hard rule. SL-e's tests are tightly bounded (3 tests, ~one mod,
  no helper crate touched). Further splitting yields no meaningful
  reduction.
- **`feedback_advisor_watchpoint_specificity.md`** — every §4
  watchpoint cites a concrete file / handler / `enums.rs` / config
  key. Binds §4 entries (8 watchpoints from brief seed list).
- **`feedback_explicit_file_arrays_on_tasks.md`** — every §13 task
  carries a FILES YAML block; cohort dispatch reads
  `union(creates, modifies)`.
- **`feedback_parallel_cohort_dispatch.md`** — Tasks 1-3 all
  `modifies: crates/server/tests/e2e.rs`; YAML overlap rule refuses
  cohort dispatch — they ship serially. Task 0 (pre-flight) and
  retro task always non-`[P]`.
- **`feedback_pre_phase_dod_smoke_test.md`** +
  **`feedback_plan_dod_dry_run_at_write.md`** — advisor-side DoD
  smoke-test runs every §15 command literally before plan approval.
  Under Shape G the §15 entries name workflow YAML paths +
  `gh run list` queries — both verifiable mid-plan-approval.
- **`feedback_features_full_p_crate_incompatible.md`** — never
  `-p <crate>` + `--features full`. Workflow YAMLs comply
  (`cargo-validate-workspace.yml:88-95` uses `--workspace --features
  full`); §15.7 advisor-side smoke snippets respect the rule.
- **`feedback_features_full_workspace_only.md`** — `--features full`
  required to activate `DbEnum` + `ts-rs` derives. Encoded in
  workflow YAML.
- **`feedback_multi_write_handlers_need_transactions.md`** — SL-e
  tests do NOT add new transactions; they drive the
  already-transactional handlers `submit_jury_vote`,
  `revoke_endorsement`, `run_grace_check_batch`. Citation-only.
- **`feedback_clippy_test_style.md`** — R1 every `i32 ↔ i64`
  comparison uses `i64::from(...)`. Bound where surfaces in §13
  Tasks 1-3 (test bodies comparing `count` (i64) vs literal Vec
  lengths (usize)).
- **`feedback_brehon_verify_pre_merge.md`** — §16a Stories grain
  enables `/brehon-verify` phantom check before `bm-merge`. SL-e's
  three stories (1, 2, 3) are checkpointed by Phase-1 workspace
  check + Phase-2 e2e on phase-branch tip.
- **`feedback_read_canonical_before_writing_spec.md`** — SL-e cites
  trunk SL-d plan (most-recent shipped sibling) + SL-c-2 plan
  (scheduler-test patterns) + SL-b plan + SL-b fixture mod Case A
  shape + SL-d fixture mod (most-recent fixture mod predecessor);
  canonical-schema gate satisfied.
- **`feedback_plan_stub_uniformity_with_canonical_sibling.md`** —
  binds §13 Tasks 1-3: stubs MUST use `LemmyResult<()>` outer
  uniformly with the canonical sibling at `e2e.rs:10980-13693`
  (`mod v1_sl_b_fixtures` Case A + `mod v1_sl_c_fixtures` + `mod
  v1_sl_d_fixtures`). The 3-cycle SL-c-2 catch-fire (cycles same
  `(E0277, e2e.rs)`) was caused by stub-shape non-uniformity; SL-e
  avoids that family.
- **`feedback_handover_trailer_cohort_propagation.md`** — non-binding
  under serial dispatch (no `[P]` cohorts in §13); per-task
  `HANDOVER:` commit trailer recommended where the next task
  benefits (Tasks 1 → 2 share the post-Pending case-state seed
  helper).
- **`feedback_retro_not_report.md`** +
  `feedback_four_role_retro_signals.md` +
  `feedback_retro_task_complexity_score.md` — retro shape (final
  task; per-task complexity table mandatory).
- **`feedback_schema_changing_spec_retrofit_question.md`** —
  citation-only (SL-e changes no spec/template shape).
- **`feedback_laptop_default_for_validate_pending.md`** — Phase 2
  e2e local-default per advisor-orchestrator user gate (PR #105,
  2026-04-28). Binds SL-e Phase 2 gate.
- **`feedback_windows_e2e_requires_bat_wrapper.md`** — if user
  picks (a) local at user gate 4, the bat wrapper invocation per
  `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e
  --features full > <log> 2>&1"` is mandatory on Windows.
  Citation-only at plan-write time.

### Related prior plans (canonical-shape mirror)

- `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` — most recent
  shipped sibling; canonical for §13 anchor-Edit pattern,
  fixture-mod naming + placement, §15 Shape G shape, §16a Stories
  shape.
- `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` — canonical
  for scheduler-driven test patterns (`BREHON_DISABLE_GRACE_CHECK_JOB=1`
  envelope; manual `run_grace_check_batch` invocation; per-sponsor
  assertion shape).
- `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` — canonical
  for HTTP-call test patterns (real handler invocation via
  `revoke_endorsement(Json(...), context, local_user_view)`); Case
  A error-shape fixture mod shape at e2e.rs:10980-11924.
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — schema +
  backfill UPDATE source (Test #3 mirrors the §8.4 SQL shape).
- `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — Shape G + per-
  test anchor-Edit + §16a Stories pattern.

---

## 3. Problem statement

Post-SL-a-merge + post-SL-b-merge + post-SL-c-merge + post-SL-d-merge
on `governance-v0` (DQ #189 confirms SL-d Phase 2 e2e passed 2026-05-11
on phase-v1-SL-d tip `78771349e`):

- **The SL lane is shipped, but only in isolation.** Each sub-phase's
  e2e suite tests a narrow slice: SL-b tests `revoke_endorsement` via
  direct-seeded `SponsorLiabilityPending` cases; SL-c tests
  `run_grace_check_batch` via direct-seeded Pending cases (no producer
  involvement); SL-d tests `submit_jury_vote` mutation via mocked
  jury panel + asserts deferred writes, but does NOT exercise the
  scheduler in the same test. **No existing test runs the full
  producer→consumer chain in one transaction sequence.**
- **PRD §15 row 5 was reserved for this lane-wide proof.** The lane-
  closer demonstrates: (a) revocation-mid-window severs the chain via
  SL-b's handler; (b) window-expiry fires liability via SL-c's
  scheduler; (c) v0 mid-flight cases are correctly backfilled by
  SL-a's UPDATE and then resolved by SL-c.
- **The restoration-during-window-escapes branch is out of scope** per
  SL-c DQ #145 LOCKED: the restoration producer endpoint
  (`POST /api/v4/governance/restoration/complete`) lives in
  `restorative-mechanics-v1` PRD which has not yet been drafted. SL-c
  shipped the consumer-side escape branch as a stub-only Fire
  fallback; testing the stub would just re-test the window-expiry
  flow already covered by Test #2.
- **The deferred-write semantics from PRD §11.4 are the most subtle
  invariant in the lane.** Between the SL-d Pending transition and
  the eventual fire/escape, `public_case_log` is empty for the case.
  Test #1's revocation branch and Test #2's expiry branch BOTH verify
  the timing: zero `public_case_log` rows mid-window; one after
  fire/escape (or zero after escape per PRD §11.4). Without this
  test, a future regression that emits `public_case_log` at
  vote-tally time would silently break auditor-visibility semantics.
- **The v0→v1 deployment story has no live test.** SL-a Task 1
  shipped the backfill UPDATE; SL-a's own e2e tests asserted the
  UPDATE wrote the expected rows but did NOT drive the post-backfill
  scheduler tick. Test #3 closes this gap.

The substrate is in place; SL-e supplies the integration proof.

---

## 4. Solution statement

Five tasks — Task 0 pre-flight + 3 e2e test tasks + retro.

### 4.1 Architecturally load-bearing decisions

- **Tests-only, zero production code edits.** Plan §11 enumerates one
  file modified: `crates/server/tests/e2e.rs`. No edits to
  `crates/api/**`, `migrations/**`, `crates/db_schema*/`,
  `crates/routes/`, `crates/db_views*/`,
  `.coderabbit.yaml`, `Cargo.toml`, `Cargo.lock`,
  `rust-toolchain.toml`, `.github/workflows/`,
  `docs/brehon-law-inspired-network/`, `.claude/PRPs/prds/`. At
  plan-approval time, `git diff governance-v0..phase-v1-SL-e --stat`
  should show one line: `crates/server/tests/e2e.rs | +<lines>`.
  The plan file itself ships on `governance-v0` before BM cuts the
  phase branch.
- **New fixture mod `mod v1_sl_e_fixtures`** opens AFTER the closing
  `}` of `mod v1_sl_d_fixtures` (currently at e2e.rs:13693 —
  verified). Mod skeleton + shared helpers ship in Task 1 (alongside
  Test #1); subsequent tests in Tasks 2-3 anchor-insert inside the
  same mod.
- **Three independent test branches, one per §13 task.** Per the brief
  scope, the three branches are:
  - **Test #1 (Task 1):** revocation-during-window-escapes — exercises
    SL-d producer → SL-b HTTP revoke → SL-c consumer skip.
  - **Test #2 (Task 2):** window-expiry-fires — exercises SL-d producer
    → SL-c consumer fire (wrapper composition).
  - **Test #3 (Task 3):** backfill-of-mid-flight — exercises SL-a
    migration backfill SQL → SL-c consumer fire-or-escape.
- **Each test is an anchor-Edit at file end** per
  `feedback_junior_worker_e2e_edit_hang.md`. e2e.rs is 13,693 lines
  pre-SL-e; Tasks 1-3 ship cleanly as separate anchor-Edits within
  the same new mod.
- **Deterministic time semantics for window-expiry (Test #2).** Per
  brief §4.1 watchpoint #2: the test drives the SL-d producer through
  the Pending transition, then **force-rewinds `grace_expires_at`** to
  a past time via a direct Diesel UPDATE on the case row. This is
  semantically equivalent to "wall-clock advanced past
  `grace_expires_at`" but completes in milliseconds (no
  `tokio::time::sleep` or real-time delay). The seed-with-past-
  `decided_at` alternative the brief mentions is structurally similar
  but requires bypassing `submit_jury_vote`'s computation of
  `grace_expires_at = now + grace_window_for_severity(severity)`. The
  force-rewind approach exercises the SL-d producer end-to-end (which
  is the whole point of a lane-wide test) AND keeps the scheduler tick
  deterministic — it is the canonically correct shape.
- **`BREHON_DISABLE_GRACE_CHECK_JOB=1` envelope** per brief §4.1
  watchpoint #3. Tests #1, #2, and #3 ALL set the env var at fixture
  bootstrap (mirror `mod v1_sl_c_fixtures` pattern at e2e.rs:12046-12049
  + 12141-12146) to prevent the background scheduler tick from
  racing the manually-invoked `run_grace_check_batch`. The env var
  is set at test start, restored at test end.
- **HTTP-call discipline for Test #1 (revocation).** Test #1 invokes
  `revoke_endorsement` (SL-b's handler) via real handler call (mirror
  `mod v1_sl_b_fixtures` pattern at e2e.rs:11157-11199), NOT direct
  DB UPDATE on the `endorsement.revoked_at` column. This exercises:
  - SL-b's transaction body (load endorsement → revoke → grace-window
    severance evaluation → governance_log emission).
  - SL-b's `liability_chain_severed_for_cases` response field
    population.
  - The escape branch path (case status flip to
    `SponsorLiabilityEscaped`, `liability_escape_reason` JSONB write,
    `sponsor_liability_escaped` log entry emission).
  Without HTTP-call discipline, the test degenerates to "did SL-c's
  scheduler skip an Escaped case" — covered already by SL-c-2's
  Test #2. The lane-wide proof requires SL-b's handler in the loop.
- **Programmatic backfill UPDATE for Test #3.** Test #3 cannot wait
  for v1-deploy migration semantics; instead, it invokes the SL-a
  §8.4 UPDATE programmatically via Diesel:
  ```rust
  diesel::sql_query(
    "UPDATE moderation_case
     SET status = 'SponsorLiabilityPending',
         grace_expires_at = decided_at + INTERVAL '24 hours'
     WHERE status = 'Decided'
       AND decided_at IS NOT NULL
       AND decided_at > now() - INTERVAL '24 hours'
       AND target_person_id IS NOT NULL
       AND id IN (
         SELECT mc.id
         FROM moderation_case mc
         WHERE EXISTS (
           SELECT 1 FROM surety s
           WHERE s.sponsored_id = mc.target_person_id
             AND s.revoked_at IS NULL
         )
         AND EXISTS (
           SELECT 1 FROM sanction sa
           WHERE sa.case_id = mc.id
         )
         AND NOT EXISTS (
           SELECT 1 FROM reputation_event re
           WHERE re.source_case_id = mc.id
             AND re.reason = 'sponsor_liability_applied'
         )
       )"
  ).execute(conn).await?;
  ```
  This mirrors PRD §8.4 verbatim. The test seeds a pre-deploy `Decided`
  case + active sureties + sanction + (no sponsor reputation_event)
  per §8.4's WHERE clause, runs the UPDATE, asserts the transition,
  then drives `run_grace_check_batch` and asserts fire-or-escape.
- **Severity-tier choice across the three tests.** Each test picks
  a different severity tier to spot-check the
  `liability.grace_window_<bucket>_hours` mapping (24h Minor / 72h
  Moderate / 168h Severe per PRD §10):
  - Test #1 (revocation): `CaseSeverity::High` (`severe` bucket → 168h
    grace) — mirrors SL-d Test #1.
  - Test #2 (expiry): `CaseSeverity::Low` (`minor` bucket → 24h grace)
    — shortest default, fastest test mental-model.
  - Test #3 (backfill): the backfill UPDATE hardcodes
    `INTERVAL '24 hours'` (ADR-010 most-lenient default), so the
    seeded case's severity doesn't drive the grace duration. Seed
    `CaseSeverity::Medium` (`moderate` bucket — irrelevant for the
    test's grace assertion but produces realistic-looking row data).
- **Pseudonym-discipline assertions.** Per brief §4.1 watchpoint #5 +
  ADR-015 + Watch 10. Each governance_log payload SL-e tests assert
  on has a defensive `assert!(json["<field>_pseudonym"].is_string())`
  and `assert_ne!(json["<field>_pseudonym"].as_str().unwrap(),
  &format!("{}", <raw_id>.0))`. Specific payloads:
  - `sponsor_liability_pending` (SL-d-emitted): `target_pseudonym` +
    `sponsors_pseudonyms` array.
  - `sponsor_liability_fired` (SL-c-emitted): `target_pseudonym`.
  - `sponsor_liability_applied` (SL-c-emitted, per sponsor):
    `sponsor_pseudonym`.
  - `endorsement_revoked` (SL-b-emitted): `revoker_pseudonym` +
    `target_pseudonym`.
  - `sponsor_liability_escaped` (SL-b-emitted): `actor_pseudonym`.
- **Deferred-write semantics assertions** per PRD §11.4. Each test
  asserts:
  - **Test #1 (revocation):** mid-window — 0 `public_case_log` rows;
    0 sponsor `reputation_event` rows; 0 juror `reputation_event`
    rows; 1 `sponsor_liability_pending` log; 1 `case_decided` log.
    Post-revoke — case status `SponsorLiabilityEscaped`; 1
    `endorsement_revoked` log; 1 `sponsor_liability_escaped` log;
    `liability_escape_reason` JSONB populated; STILL 0
    `public_case_log` rows (per PRD §11.4 — escape branch never
    publishes); STILL 0 sponsor `reputation_event` rows.
  - **Test #2 (expiry-fires):** mid-window (immediately after
    transition) — same as Test #1 mid-window assertions. Post-fire
    — case status `SponsorLiabilityFired`; N (= sponsor count)
    sponsor `reputation_event` rows; N `sponsor_liability_applied`
    logs; 1 `sponsor_liability_fired` summary log; 0
    `sponsor_liability_escaped` logs; 1 `public_case_log` row (SL-c
    emits at fire-time per PRD §11.4); juror + reporter
    `reputation_event` rows fire.
  - **Test #3 (backfill):** pre-backfill — case status `Decided`,
    `grace_expires_at` IS NULL. Post-backfill — case status
    `SponsorLiabilityPending`, `grace_expires_at` = `decided_at +
    24h` exactly. Post-scheduler-tick — case status
    `SponsorLiabilityFired` (the seeded test has no
    revocation-during-window action, so the scheduler fires); N
    sponsor `reputation_event` rows; N `sponsor_liability_applied`
    logs; 1 `sponsor_liability_fired` log.
- **No federation outbound on Pending transition** per ADR-014 + PRD
  §11.5. Tests #1 + #2 include a defensive grep-style assertion: zero
  `federation_outbox` rows referencing the case while it's in
  `SponsorLiabilityPending` state. (After fire on Test #2, the SL-c
  scheduler may emit outbound — but Pending → no outbound is the
  invariant.)
- **Shape G — DoD references workflow YAMLs by path + expected
  `conclusion`, not inline cargo.** Per
  `.claude/PRPs/templates/plan.template.md` §15.6 + DQ #67. Every
  §13 task's DoD references `cargo-validate-workspace.yml` on the
  worker branch + workflow_run_id captured by impl-task subagent
  post-push. Phase 2 e2e fires on `phase-v1-SL-e` per advisor's
  local-vs-dispatch user gate (PR #105).
- **No migration round-trip workflow fires.** SL-e touches no
  `migrations/**` paths; `cargo-validate-migration.yml` path filter
  excludes SL-e commits.

### 4.2 Watchpoints (specific files / handlers / `enums.rs` lines)

Per `feedback_advisor_watchpoint_specificity.md`. The 8 watchpoints
below are the seed list per brief §4.1.

1. **Three independent test branches; one anchor-Edit each.** Per
   `feedback_junior_worker_e2e_edit_hang.md`. e2e.rs is **13,693
   lines** (verified `wc -l` at plan-write time). Tasks 1, 2, 3 each
   anchor-Edit at file end; no bundling. Task 1 opens
   `mod v1_sl_e_fixtures` shell + shared helpers + Test #1; Tasks
   2, 3 anchor-insert subsequent tests INSIDE the same mod, ending
   with Task 3's closing `}` for the mod.
2. **Time-handling discipline for Test #2 (window-expiry).** The test
   uses the **force-rewind `grace_expires_at`** approach:
   - Drive `submit_jury_vote` normally → case transitions to
     `SponsorLiabilityPending` with `grace_expires_at = now +
     Duration::hours(<grace_for_severity>)`.
   - Then `diesel::update(moderation_case::table.filter(...))
     .set(moderation_case::grace_expires_at.eq(Some(Utc::now() -
     Duration::minutes(1)))).execute(conn).await?;` — rewind to past.
   - Then invoke `run_grace_check_batch(&context).await?` — SL-c picks
     up the case (its filter is `grace_expires_at.le(Some(now))` per
     `sponsor_liability_grace.rs:149`).
   No `tokio::time::sleep`, no real-time delay. The producer-vote
   sequence runs in the same test transaction-equivalent flow as a
   real-world deploy. Test #2's IMPLEMENT specifies the exact UPDATE
   statement.
3. **`BREHON_DISABLE_GRACE_CHECK_JOB=1` envelope on all 3 tests.** Per
   brief §4.1 watchpoint #3. The `mod v1_sl_c_fixtures` pattern at
   e2e.rs:12046-12049 + 12141-12146 is the canonical mirror — set at
   test start in an `unsafe { std::env::set_var(...) }` block,
   restored at test end. Without the env var, the background
   scheduler tick (every `job.grace_check_interval_minutes` = 5 min
   default) races the manual `run_grace_check_batch` invocation.
4. **Real HTTP-call invocation for revocation in Test #1.** Per brief
   §4.1 watchpoint #4. Test #1 invokes `revoke_endorsement` via:
   ```rust
   let response = lemmy_api_crud::governance::revoke_endorsement::revoke_endorsement(
     Json(RevokeEndorsement {
       endorsement_id,
       reason: "test_revoke_during_window".to_string(),
     }),
     context.clone(),
     sponsor_view,
   ).await?;
   ```
   (Mirror `mod v1_sl_b_fixtures` pattern at e2e.rs:11157-11199.)
   Direct UPDATE on `endorsement.revoked_at` is rejected — it bypasses
   the SL-b transaction body and SL-b's emission of the
   `endorsement_revoked` + `sponsor_liability_escaped` log entries.
5. **Pseudonym-discipline assertions.** Per ADR-015 + brief §4.1
   watchpoint #5. Per §4.1's enumeration, each payload SL-e tests
   assert on has a defensive `assert!(json["<field>_pseudonym"].is_string())`
   AND `assert_ne!(json["<field>_pseudonym"].as_str().unwrap(),
   &format!("{}", <raw_id>.0))` pair. Per-test:
   - Test #1: `sponsor_liability_pending` (target_pseudonym +
     sponsors_pseudonyms array); `endorsement_revoked`
     (revoker_pseudonym + target_pseudonym);
     `sponsor_liability_escaped` (actor_pseudonym).
   - Test #2: `sponsor_liability_pending`; `sponsor_liability_applied`
     (per-row sponsor_pseudonym); `sponsor_liability_fired`
     (target_pseudonym).
   - Test #3: `sponsor_liability_applied`; `sponsor_liability_fired`.
6. **Backfill UPDATE programmatic invocation in Test #3.** Per brief
   §4.1 watchpoint #6 + PRD §8.4. The exact UPDATE shape is in
   Test #3's IMPLEMENT block — verbatim from PRD §8.4 (NOT
   paraphrased). The test seeds a pre-deploy state matching the WHERE
   clause's positive-match conditions (status=Decided, decided_at
   recent, active surety exists, sanction exists, no
   sponsor-reputation_event), invokes the UPDATE via
   `diesel::sql_query(...)` (raw SQL since the migration shape uses
   `INTERVAL '24 hours'` and CTE-style EXISTS subqueries that don't
   map cleanly to Diesel's typed query builder for an UPDATE), and
   asserts the post-UPDATE case state.
7. **No new ENTRY_KIND consts; no new schema; no new module.** Per
   brief §4.1 watchpoint #7. Plan §13 must produce zero new files
   outside `crates/server/tests/e2e.rs`. At plan-approval time,
   `git diff governance-v0..phase-v1-SL-e --stat` must show ONE
   file modified: `crates/server/tests/e2e.rs`. At task-end of each
   §13 task, mechanical greps assert no edits to
   `crates/db_schema/src/source/governance/governance_log.rs`, no
   edits to `crates/api/api/src/governance/**`, no edits to
   `crates/api/api_crud/src/governance/**`, no edits to
   `migrations/**`, no edits to `.github/workflows/**`.
8. **e2e Edit-per-task discipline.** Per
   `feedback_junior_worker_e2e_edit_hang.md`. e2e.rs is 13,693 lines
   post-SL-d. Each test gets its own task; each anchor-Edit appends
   inside `mod v1_sl_e_fixtures` (which Task 1 opens and Task 3
   closes). Bulk-Editing multiple tests in one task is rejected.

### 4.3 Rejected alternatives

- **Bundle the 3 e2e tests into 1-2 §13 tasks.** Rejected per
  `feedback_junior_worker_e2e_edit_hang.md` — e2e.rs is 13,693
  lines; multi-test bulk Edits hang Junior workers. SL-d shipped 4
  tests as 4 tasks; SL-c-2 shipped 5 tests as 5 tasks.
- **Split the SL-e sub-phase further (e.g. SL-e-1 revocation + SL-e-2
  expiry+backfill).** Per `feedback_complexity_score_pre_split.md`
  §5.2 below: planner files DQ #190 self-resolved with proceed
  rationale (e2e factor dominates +9 of the +10 total; further
  splitting yields no meaningful score reduction; SL-d at 16,
  SL-c-2 at 17, SL-b at 38, JM-e at 15 all proceed-as-one
  precedents). Plus SL-e is a lane-closer — fragmenting the
  behavioural test suite across two sub-phases adds 2x finalize +
  retro overhead for no architectural benefit. Advisor may overturn
  at plan-approval time.
- **Include the optional 4th test (admin-overridden status flip on
  Pending case).** Per brief §2.2 + §4.2 boundary-of-judgment.
  **Rejected** to keep SL-e strictly scoped to PRD §15 row 5's
  three branches (revocation + expiry + backfill). The
  admin-override pathway (`admin_close_case` on `SponsorLiabilityPending`)
  is already covered by:
  - `admin_close_case.rs:65-80` exhaustive match — confirmed at
    plan-write time to include `SponsorLiabilityPending`,
    `SponsorLiabilityFired`, `SponsorLiabilityEscaped` in the
    allowed-set (PRD §3.3).
  - Admin-dashboard-v1 sub-phases (already shipped — `v1-AD-a/b/c`)
    own admin-flow tests.
  Adding a 4th test to SL-e duplicates admin-flow test territory and
  adds another anchor-Edit cost. Noted in §19.5 as a follow-up
  candidate if admin-dashboard-v1 retro flags a coverage gap.
- **Include a restoration-during-window-escapes test.** Rejected per
  SL-c DQ #145 LOCKED — the restoration producer endpoint lives in
  `restorative-mechanics-v1` PRD (not yet drafted). SL-c shipped the
  consumer-side restoration-escape branch as a stub-only Fire
  fallback (per `feedback_build_what_tests_exercise.md`); a test of
  the stub would just re-test the Fire branch (covered by Test #2).
  When `restorative-mechanics-v1` lands, the matching test ships
  alongside its producer endpoint.
- **Use `tokio::time::sleep` for window-expiry in Test #2.** Rejected
  per watchpoint #2. Real-time sleep adds 24h+ of wall-clock per
  test invocation — incompatible with the 60-min ci-watcher cap and
  with the impl-task subagent's tooling envelope. The
  force-rewind-`grace_expires_at`-via-UPDATE approach is
  semantically equivalent and runs in <100ms.
- **Use `Result<(), Box<dyn Error>>` outer in any e2e test fn.**
  Rejected per `feedback_lemmy_error_no_std_error.md` Case A
  canonical sibling at `crates/server/tests/e2e.rs:11001-11924`
  (`mod v1_sl_b_fixtures`) and the subsequent `mod v1_sl_c_fixtures`
  / `mod v1_sl_d_fixtures` continuations. All §13 Tasks 1-3 use
  uniform `LemmyResult<()>` outer per the lesson's Case A.
- **Mark Tasks 1-3 as `[P]`.** Rejected per
  `feedback_parallel_cohort_dispatch.md` — all 3 tasks
  `modifies: crates/server/tests/e2e.rs`; YAML overlap rule refuses
  cohort dispatch. Each task anchor-Edits at the prior task's commit
  tip inside the same mod block; cohort dispatch would race the
  closing `}` of the mod (Task 3 closes the mod that Task 1 opens).
- **Test the SL-d compute/fire wrapper composition (Test #4 from
  SL-d).** Rejected — already covered by SL-d Task 6's
  `apply_sponsor_liability_wrapper_preserves_v0_outputs` test at
  e2e.rs:13528-13693. SL-e's Test #2 exercises the wrapper composition
  end-to-end as a side effect (the producer creates the case; the
  scheduler fires the wrapper). Adding a dedicated SL-e wrapper-
  composition test would duplicate SL-d Task 6.

---

## 5. Metadata

- **Phase:** `v1-SL-e`
- **Branch:** `phase-v1-SL-e` (cut by BM-task before Task 1, AFTER
  this plan merges to `governance-v0` and the user approves the
  plan)
- **Target impl-task model:** `sonnet-4-6` (default, per
  `feedback_brehon_subagent_model_effort_assignments.md` baseline).
- **Estimated tasks:** 5 (Task 0 pre-flight + Tasks 1-3 e2e tests +
  Task 4 retro)
- **Estimated cargo budget:** 0 GB peak local (Shape G — cargo
  runs on GH-hosted runners; Phase 2 e2e on laptop ~26 min
  single-threaded if user picks local at gate 4)
- **Forbidden-window applicability:** non-binding for impl-task
  throughput under Shape G; standard for any local diagnostic cargo
  run (rare). Phase 2 e2e on laptop respects forbidden windows per
  user gate.
- **Complexity score:** **10/10** — see breakdown below.
  Threshold-tripping (>8 for Sonnet target); planner DQ #190 filed
  and self-resolved per Recipe 2 with proceed rationale.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Computed mechanically
from §13 task list:

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **0** | 3 impl tasks (Tasks 1-3; Task 0 + retro excluded). `max(0, 3-5) = 0` |
| Migrations touched | +2 each | **0** | SL-e ships zero migrations |
| Crates touched | +1 each | **1** | `crates/server` only (tests/e2e.rs) |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | **9** | Tasks 1, 2, 3 each modify `crates/server/tests/e2e.rs`. 3 × +3 = 9 |
| New ADR-affecting decisions | +2 each | **0** | All ADR decisions inherited from SL-a/b/c/d PRDs |
| Cargo budget peak above 6 GB | +1 per GB | **0** | Shape G — cargo runs off-box |
| **Total** | — | **10** | Threshold for split-DQ (Sonnet target): `>8`. **Tripped.** |

### 5.2 Split-or-proceed DQ

Per `feedback_complexity_score_pre_split.md` + `planning.md` §5
"Decision-queue — pre-seed forward-looking OQs": planner files
**DQ #190** (`from: "planner"`, `kind: "blocker"`, `answered_by:
"planner"` self-resolved with rationale per Recipe 2) BEFORE
committing the plan. Question: "Complexity score 10 exceeds 8 —
split `v1-sponsor-liability-e` further (e.g. `v1-SL-e-1` revocation
[Task 1] + `v1-SL-e-2` expiry + backfill [Tasks 2-3]), or
proceed?". Options: split / proceed.

**Planner observation (binding lean — proceed):** the dominant
factor is the 3 e2e edits (+9 of the +10 total). Splitting SL-e
further yields:

- **SL-e-1** (Task 1 — revocation): 1 impl task, 0 migrations, 1
  crate, 1 e2e edit. Score: `0 + 0 + 1 + 3 + 0 + 0 = 4`. **Below
  threshold.**
- **SL-e-2** (Tasks 2-3 — expiry + backfill): 2 impl tasks, 0
  migrations, 1 crate, 2 e2e edits. Score: `0 + 0 + 1 + 6 + 0 + 0
  = 7`. **Below threshold.**

Mechanically, splitting works — but the cost-benefit is poor:

- The 3 tests share fixture setup (jury panel seeding, sanction
  insertion, surety+endorsement chain construction). Sharing helpers
  in one fixture mod (`mod v1_sl_e_fixtures`) reduces duplication
  by ~40% vs splitting helpers across two mods. Per
  `feedback_handover_trailer_cohort_propagation.md`, the
  helper-handover discipline mitigates this slightly when split, but
  doesn't fully recover the cohesion.
- SL-e is the **lane-closer** — fragmenting the lane-wide proof
  across two sub-phases adds 2× cohort/finalize/retro overhead
  (additional bm-cut, bm-pr, bm-poll-cr, bm-triage, bm-merge,
  retro). Each sub-phase ships ~3-4 hour orchestration wall-clock
  per `feedback_complexity_score_pre_split.md` calibration evidence.
  Doubling that for a 3-test suite is high-cost-low-benefit.
- The 3 tests are mechanically independent and well-bounded; the
  3-cycle catch-fire risk from `feedback_plan_stub_uniformity_with_canonical_sibling.md`
  is mitigated by canonical-Case-A sibling discipline (Tasks 1-3
  GOTCHAs cite the canonical sibling at e2e.rs:10980-13693
  verbatim). No structural reason to expect 1-cycle-per-test cost
  to balloon.

Per `feedback_principles_not_rules.md`: the score is a signal, not
a hard rule. Precedent for "score above threshold but proceed":

- SL-b: shipped at score 38 with proceed-as-one (DQ #143).
- SL-c-2: shipped at score 17 with proceed-as-one (DQ #151).
- SL-d: shipped at score 16 with proceed-as-one (DQ #186; Phase 2
  e2e passed cleanly per DQ #189).
- JM-e: shipped at score 15 with proceed-as-one.
- SL-a: shipped at score 13 with proceed-as-one.

SL-e at 10 sits at the low end of this envelope (only marginally
over threshold; e2e factor entirely dominates). Plan ships under
the **proceed** assumption. DQ #190 filed with `answered_by:
"planner"` self-resolved per Recipe 2 citing the rationale above;
the advisor may overturn at plan-approval time if user prefers a
further split — but the e2e-factor analysis suggests proceeding-as-
one is the cleaner path.

### 5.3 Per-task complexity (Sonnet ceiling: ≤4 files, ≤2 crates per task)

Walking each §13 task against the Sonnet ceiling (per
`planning.md` §5b — non-binding for Sonnet target but tracked):

| Task | files | crates | within Sonnet ceiling? |
|---|---|---|---|
| Task 0 | 0 | 0 | yes (pre-flight, no edits) |
| Task 1 (e2e #1 — revocation + mod shell + helpers) | 1 (e2e.rs) | 1 (server) | yes |
| Task 2 (e2e #2 — window-expiry-fires) | 1 (e2e.rs) | 1 (server) | yes |
| Task 3 (e2e #3 — backfill + close mod) | 1 (e2e.rs) | 1 (server) | yes |
| Task 4 (retro) | 1 (retro.md) | 0 | yes |

All tasks within ceiling. No per-task split needed.

---

## 6. Relationship to other v1-SL sub-phases

| Sub-phase | Status | What it ships | SL-e dependency |
|---|---|---|---|
| v1-SL-a | MERGED (PR #111, governance-v0 @ `790f6101d`) | Schema + 13 seeded keys (incl. `liability.grace_window_<minor\|moderate\|severe>_hours` defaults 24/72/168) + 5 entry-kind consts + 3 CaseStatus variants + Issue #24 partial index + backfill UPDATE in `migrations/2026-04-22-…_add_sponsor_liability_grace_window/up.sql` | **Test #3 invokes SL-a's §8.4 backfill UPDATE programmatically.** Tests #1 + #2 + #3 all assert on the SL-a-seeded config keys' default values |
| v1-SL-b | MERGED (PR #119) | `revoke_endorsement` handler + DTO + route + 9-10 e2e tests at `mod v1_sl_b_fixtures` (e2e.rs:10980-11924) | **Test #1 invokes SL-b's `revoke_endorsement` handler via real HTTP-call.** Test #1's `mod v1_sl_e_fixtures` helpers mirror `mod v1_sl_b_fixtures` shape (canonical Case A error-shape sibling) |
| v1-SL-c (c-1 + c-2) | MERGED (governance-v0; `adc25da49` cherry-pick) | Scheduler module `sponsor_liability_grace.rs` + clokwerk wiring + 5 `grace_check_*` e2e tests at `mod v1_sl_c_fixtures` (e2e.rs:11927-12819) | **Tests #1 + #2 + #3 all drive `sponsor_liability_grace::run_grace_check_batch(&context).await`.** The `mod v1_sl_c_fixtures` pattern (BREHON_DISABLE_GRACE_CHECK_JOB envelope, fixture seeding) is the canonical mirror for scheduler-driven tests |
| v1-SL-d | MERGED (governance-v0; DQ #189 Phase 2 e2e passed 2026-05-11 on tip `78771349e`) | `apply_sponsor_liability` split (compute + fire + wrapper) + `submit_jury_vote` mutation + `grace_window_for_severity` helper + 4 e2e tests + 1 unit-test task at `mod v1_sl_d_fixtures` (e2e.rs:12821-13693) | **Tests #1 + #2 invoke SL-d's `submit_jury_vote` handler via real handler-call.** `mod v1_sl_d_fixtures` is the immediate-predecessor fixture mod; Task 1 anchors `mod v1_sl_e_fixtures` AFTER its closing `}` |
| **v1-SL-e (THIS PLAN)** | NOT YET CUT | Lane-wide e2e suite (3 tests: revocation-during-window-escapes + window-expiry-fires + backfill-of-mid-flight) at new `mod v1_sl_e_fixtures` | — |
| restorative-mechanics-v1 | PENDING (separate PRD; not yet drafted) | `POST /api/v4/governance/restoration/complete` endpoint + restoration-escape branch (defendant-initiated; admin-attested) | SL-e does NOT cover restoration-during-window-escapes (PRD §15 row 5 4th sub-branch); deferred to this future PRD per SL-c DQ #145 LOCKED |
| v1-SL-lane-meta-retro | PENDING (writes after SL-e merges) | Lane-shipping retro per AD-lane precedent (`.claude/PRPs/reports/v1-AD-lane-meta-retro.md`) | SL-e is the **lane-closer**; SL-lane-meta-retro follows. Not part of SL-e itself |

**Cross-PRD sequencing (per PRD §15 + §17.1):**

- SL-e parallel-safe with rep-tuning-r*, admin-dashboard-v1, JM-* —
  different files + different concerns.
- SL-e unblocks the SL lane-meta-retro.
- SL-e closes out the SL lane (modulo the restoration-escape branch
  deferred to restorative-mechanics-v1).

---

## 7. Preflight guardrails inherited from prior phases

- **DQ #44 — Docker daemon preflight.** Enumerated in §13 Task 0
  (R5 — Probe 0). SL-e's e2e suite uses testcontainers-rs; Docker
  Desktop / dockerd MUST be running before any test invocation
  (Phase 2 e2e on laptop) or local diagnostic cargo. Probe 0
  mandatory.
- **DQ #145 — Restoration-during-window-escapes deferred to
  restorative-mechanics-v1 PRD (LOCKED).** SL-e excludes
  restoration tests entirely. §12 enumerates as out-of-scope.
- **DQ #176 — Sponsor notifications OUT (LOCKED 2026-05-10).** SL-e
  inherits — does NOT test `notify_sponsor_of_pending_liability`
  (does not exist on trunk; restorative-mechanics-v1 or v3 polish).
- **DQ #179 — `sponsor_liability_pending` log entry emit-at-transition
  (LOCKED 2026-05-10).** Bound in §4.2 watchpoint #5 — SL-e tests
  assert the log entry is emitted at the SL-d Pending transition
  (NOT deferred to scheduler).
- **DQ #186 — SL-d proceed-as-one (resolved planner).** SL-d shipped
  cleanly with score 16; precedent for SL-e's proceed-as-one at score
  10.
- **R1 — `i64::from(...)` on `i32 ↔ i64`.** Bound in §13 Tasks 1-3
  GOTCHAs where `governance_log::table.filter(...).count()` returns
  `i64` and is compared against literal counts (no `i64::from`
  needed for literal; needed when comparing against
  `usize::len()` of a Vec).
- **R5 — pre-phase harness audit enumerates ALL probes.** Bound in
  §13 Task 0. Probes 0..N listed (SL-e verifies SL-a + SL-b + SL-c
  + SL-d shipped state on top of standard wrapper probes).
- **R6 — uniform `--no-deps -- -D warnings` clippy.** Encoded in
  `cargo-validate-workspace.yml:92`.
- **R7 — `cargo test --no-run -p lemmy_server --test e2e` on tasks
  touching a struct or re-export.** Encoded in
  `cargo-validate-workspace.yml:95`. Bound in Tasks 1-3 (each adds
  test fns to `crates/server/tests/e2e.rs`).
- **JM-c retro lessons — `feedback_clippy_rerun_after_fix.md`.**
  Bound in §13 Tasks 1-3 GOTCHA — if test edits unmask an
  `unused-imports` lint, the workspace-check workflow's clippy step
  catches.
- **JM-d retro §5 — `feedback_laptop_default_for_validate_pending.md`.**
  Phase 2 e2e local-default per advisor-orchestrator user gate (PR
  #105, 2026-04-28). Binds SL-e Phase 2 gate.
- **SL-a retro lessons — pseudonym discipline (ADR-015) + Watch 10.**
  Bound in §4 watchpoint #5 + §13 Tasks 1-3 (per-test pseudonym
  assertions).
- **SL-c-2 retro lessons (carry-forward) —
  `feedback_lemmy_error_no_std_error.md` Case A canonical sibling +
  `feedback_plan_stub_uniformity_with_canonical_sibling.md`.**
  Bound in §13 Tasks 1-3 stub-shape uniformity (LemmyResult<()>
  uniform throughout the new fixture mod, mirroring
  `mod v1_sl_b_fixtures` / `mod v1_sl_c_fixtures` /
  `mod v1_sl_d_fixtures`).
- **SL-d retro lessons (carry-forward) — deferred-write semantics
  test pattern.** SL-d's Test #1 established the "negative
  assertion at vote-tally time, deferred to scheduler" pattern; SL-e
  Test #1 + #2 reuse and extend (Test #2 adds the positive assertion
  at scheduler-tick time — the deferral round-trip).
- **SL-c-2 retro §3 lesson — DQ commits on phase branch cause PR
  DIRTY conflicts.** Bound in §13 Task 4 + Risks §18 — SL-e's
  phase branch should not accumulate DQ mutation commits;
  ci-watcher mutations should land on `governance-v0` directly.
  If a Phase-1 fail surfaces a DQ on the phase branch, the
  advisor's bm-pr smoke check `mergeStateStatus` flags before CR
  review wastes time.

---

## 8. Flow design

### 8.1 Before state (post-SL-d-merge on `governance-v0`)

The sponsor-liability lifecycle today (post-SL-d):

- `submit_jury_vote::process_vote` step 8.5 calls v1-SL-d
  `compute_sponsor_liability` (pure read); branches on
  `Vec::is_empty()`; on non-empty, sets `path_kind = Pending` and the
  step 8.9 case UPDATE writes
  `case.status = SponsorLiabilityPending` + `grace_expires_at = now +
  grace_window_for_severity(case.severity)`. Per-sponsor
  `reputation_event` rows + juror + reporter reputation events are
  deferred (NOT written at vote-tally time).
- `sponsor_liability_pending` governance_log entry is emitted at the
  Pending transition (per DQ #179 LOCKED).
- `case_decided` governance_log entry is emitted on BOTH paths (per
  PRD §11.4 auditor visibility).
- `revoke_endorsement` (SL-b) handles caller-sponsor revocation; on
  revocation of an endorsement whose sponsee has active
  `SponsorLiabilityPending` cases, the handler severs each case's
  liability chain (transitions to `SponsorLiabilityEscaped`, writes
  `liability_escape_reason` JSONB, emits
  `sponsor_liability_escaped` log).
- `sponsor_liability_grace::run_grace_check_batch` (SL-c) iterates
  `SponsorLiabilityPending` cases past `grace_expires_at`, per-case
  evaluates escape conditions, fires-or-escapes via per-case
  transaction.
- **No existing e2e test exercises the producer → consumer chain in
  one transaction sequence.** SL-b's tests pre-seed Pending cases via
  direct INSERT; SL-c's tests pre-seed Pending cases via direct
  INSERT; SL-d's tests pre-seed `Decided` and stop at the Pending
  transition (no scheduler-tick).

### 8.2 After state (post-SL-e-merge)

```
mod v1_sl_e_fixtures (new — opens after e2e.rs:13693 closing `}`)
├── shared helpers (mirror mod v1_sl_b/c/d shape):
│     governance_fixtures::bootstrap + actix_web FederationConfig setup
│     seed_target_with_sureties + seed_endorsement_active
│     count_log_entries + read_log_payload (mirror SL-c-2 helpers)
│     read_case_status_and_escape (mirror SL-b helper)
│     drive_jury_to_quorum (compose admin_assign_jury + accept × N + submit_jury_vote × M)
│
├── Test #1: revocation_during_window_escapes_full_lane
│     [Task 1]  setup: 1 sponsee + 2 sponsors w/ endorsements + sureties;
│                       jury panel; sanction-bearing JuryDecision pending
│     [Task 1]  drive #1: submit_jury_vote → quorum → SponsorLiabilityPending
│     [Task 1]  assert: status == SponsorLiabilityPending; 1 sponsor_liability_pending
│                       log; 0 public_case_log; 0 sponsor reputation_event rows
│     [Task 1]  drive #2: revoke_endorsement(sponsor1) via HTTP handler
│     [Task 1]  assert: status == SponsorLiabilityEscaped;
│                       liability_chain_severed_for_cases ∋ case_id;
│                       1 endorsement_revoked + 1 sponsor_liability_escaped log;
│                       liability_escape_reason JSONB populated; STILL 0
│                       public_case_log; STILL 0 sponsor reputation_event rows
│     [Task 1]  drive #3: run_grace_check_batch
│     [Task 1]  assert: outcome.cases_processed == 0 (or > 0 with case skipped —
│                       case no longer in Pending status, NOT picked up by filter);
│                       no transition; no new logs
│
├── Test #2: window_expiry_fires_full_lane
│     [Task 2]  setup: 1 sponsee + 2 sponsors w/ active sureties; jury panel;
│                       severity = CaseSeverity::Low (→ Minor bucket → 24h grace)
│     [Task 2]  drive #1: submit_jury_vote → quorum → SponsorLiabilityPending
│     [Task 2]  assert: status == SponsorLiabilityPending; grace_expires_at
│                       ≈ now + 24h ± 5s
│     [Task 2]  drive #2: UPDATE moderation_case SET grace_expires_at = now - 1 min
│     [Task 2]  drive #3: run_grace_check_batch
│     [Task 2]  assert: status == SponsorLiabilityFired; 2 sponsor reputation_event
│                       rows; 2 sponsor_liability_applied + 1 sponsor_liability_fired
│                       logs; 1 public_case_log row (SL-c emits at fire); juror +
│                       reporter reputation_event rows fire (deferred from
│                       vote-tally → now fired by scheduler per PRD §11.4)
│
├── Test #3: backfill_of_mid_flight_v0_to_v1_deploy
│     [Task 3]  setup: pre-deploy state — seed case with status=Decided,
│                       decided_at = now - 12h (within 24h backfill window),
│                       target_person_id with 1 active surety, sanction inserted,
│                       NO sponsor reputation_event rows
│     [Task 3]  drive #1: invoke SL-a backfill UPDATE programmatically via
│                       diesel::sql_query(...) (mirror PRD §8.4 SQL verbatim)
│     [Task 3]  assert: status == SponsorLiabilityPending;
│                       grace_expires_at == decided_at + 24h (exact)
│     [Task 3]  drive #2: run_grace_check_batch (case now expired —
│                       grace_expires_at = (now-12h) + 24h = now+12h is in
│                       future; ACTUALLY: with decided_at = now - 12h, grace_expires_at
│                       = now + 12h is in the FUTURE — scheduler doesn't pick up.
│                       Adjust: set decided_at = now - 25h so grace_expires_at = now - 1h)
│     [Task 3]  drive #2 (revised): run_grace_check_batch with case
│                       grace_expires_at in past — case picked up; fires
│     [Task 3]  assert: status == SponsorLiabilityFired; 1 sponsor reputation_event
│                       row; 1 sponsor_liability_applied + 1 sponsor_liability_fired
│                       log; 1 public_case_log row
│     [Task 3]  closes the mod with `}`
│
└── (mod v1_sl_e_fixtures closes at end of Task 3's anchor-Edit)

[Task 4 — retro]
Author .claude/PRPs/reports/v1-SL-e-retro.md per 4-role retro shape.
Per-task complexity table mandatory.
```

**Test #3 timing note (revised in the diagram above):** in the
backfill UPDATE, `grace_expires_at = decided_at + INTERVAL '24 hours'`.
For the scheduler to pick the case up immediately after backfill,
`grace_expires_at` must be in the past — which means `decided_at`
must be more than 24h in the past. But the backfill WHERE clause
requires `decided_at > now() - INTERVAL '24 hours'`. The two
constraints contradict — the backfill is intended for cases mid-flight
within the last 24h, which gives them a future grace window. **For
Test #3**, the test must EITHER:

- **(a)** seed `decided_at = now - 23h 30 min` (within 24h window),
  run the backfill, assert `grace_expires_at = now + 30 min` (future);
  then `UPDATE moderation_case SET grace_expires_at = now - 1 min`
  to rewind (same technique as Test #2); then drive
  `run_grace_check_batch` and assert fire.
- **(b)** seed `decided_at = now - 25h` (OUTSIDE the 24h window),
  show the backfill WHERE clause skips it, then assert NOT-picked-up
  semantics — but this doesn't exercise the fire path.

**Lean (a)** — exercise the backfill + post-rewind fire — gives full
lane coverage. Test #3's IMPLEMENT specifies (a).

### 8.3 Endpoint changes

NONE. SL-e is tests-only. No new HTTP endpoint, no DTO changes, no
route registration.

---

## 9. Mandatory reading

### 9.1 Brehon design docs (P0)

- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` §1, §2, §3.1, §3.2,
  §4.1, §4.3, §5.3, §5.4, §6.2, §8.4, §9.1, §9.3, §10, §11.2, §11.4,
  §15 row 5, §17.
- `.claude/PRPs/briefs/sl-e-planning-1.md` — the advisor brief.
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  ADR-010, ADR-013, ADR-014, ADR-015.

### 9.2 Codebase reads (P0 — mirror these patterns)

- `crates/server/tests/e2e.rs:10980-11924` — `mod v1_sl_b_fixtures`
  (canonical Case A error-shape sibling; HTTP-call revocation
  pattern; helpers `seed_endorsement_active`, `read_endorsement_revoked_at`,
  `read_surety_revoked_at`, `read_case_status_and_escape`,
  `count_log_entries`, `read_log_payload`, `read_snapshot_calculated_at`).
- `crates/server/tests/e2e.rs:11927-12819` — `mod v1_sl_c_fixtures`
  (scheduler-driven test patterns; `BREHON_DISABLE_GRACE_CHECK_JOB`
  envelope at lines 12046-12049 + 12141-12146; helpers
  `count_log_entries`, `read_log_payload`, `seed_pending_case`,
  `seed_active_surety`).
- `crates/server/tests/e2e.rs:12821-13693` — `mod v1_sl_d_fixtures`
  (most-recent fixture mod; SL-d producer test patterns; helpers
  `seed_target_with_sureties`; full producer + jury-quorum drive
  pattern at lines 12891-13132).
- `crates/server/tests/e2e.rs:13693` — final line of `mod
  v1_sl_d_fixtures` closing `}`. Task 1's anchor-Edit appends a new
  `mod v1_sl_e_fixtures` IMMEDIATELY AFTER this line.
- `crates/server/tests/e2e.rs:907` —
  `governance_log_hash_chain_holds` test (canonical
  `governance_log::table.filter(...)` count assertion pattern).
- `crates/api/api/src/governance/sponsor_liability_grace.rs:80-90`
  (`GraceCheckBatchOutcome` struct; SL-e tests destructure
  `.cases_processed`, `.fired`, `.escaped`, `.skipped`).
- `crates/api/api/src/governance/sponsor_liability_grace.rs:127-189`
  (`run_grace_check_batch` body — confirm callable from tests; SL-c
  tests already drive this entry).
- `crates/api/api_crud/src/governance/revoke_endorsement.rs:54-110`
  (handler entry — confirm callable signature SL-b tests already use:
  `revoke_endorsement(Json(RevokeEndorsement), Data<LemmyContext>,
  LocalUserView) -> LemmyResult<Json<RevokeEndorsementResponse>>`).
- `crates/api/api_crud/src/governance/revoke_endorsement.rs:280-369`
  (escape-evaluation + log emission body — Test #1 asserts on the
  emitted log payloads).
- `crates/api/api/src/governance/submit_jury_vote.rs` (entry — Test
  #1 + #2 drive jury votes via real handler-call; mirror SL-d Test
  #1 setup at e2e.rs:12891-12996).
- `crates/api/api/src/governance/admin_assign_jury.rs` (entry — used
  in SL-d fixtures to seat jurors).
- `crates/api/api/src/governance/accept_jury_assignment.rs` (entry —
  used in SL-d fixtures for jury-accept loop).
- `crates/api/api_common/src/governance.rs:312-329`
  (`RevokeEndorsement` + `RevokeEndorsementResponse` DTOs; Test #1
  asserts on `response.liability_chain_severed_for_cases`).
- `crates/db_schema_file/src/enums.rs:393-428` (12 `CaseStatus`
  variants — SL-e tests assert transitions through `Decided`,
  `SponsorLiabilityPending`, `SponsorLiabilityFired`,
  `SponsorLiabilityEscaped`).
- `crates/db_schema_file/src/enums.rs:461-467` (`CaseSeverity` 4
  variants — Tests #1 + #2 + #3 each pick one for setup).
- `crates/db_schema_file/src/schema.rs` `moderation_case` table —
  confirm columns SL-e tests query (`status`, `grace_expires_at`,
  `liability_escape_reason`, `decided_at`, `severity`,
  `target_person_id`, `community_id`, `winning_decision`,
  `appeal_window_expires_at`).
- `crates/api/api/src/governance/governance_log.rs` (api shim) —
  ENTRY_KIND const re-exports SL-e tests assert against (5 SL
  consts shipped in SL-a + the v0 `case_decided`, `sanction_created`,
  `public_log_published`, `reputation_delta` consts).
- `crates/db_schema/src/source/governance/governance_log.rs:195-220`
  — canonical const declarations.
- `crates/api/api/src/governance/config.rs:925-945` —
  `DEFAULT_LIABILITY_GRACE_WINDOW_*` default consts (Tests #1 + #2
  + #3 read defaults via `governance_config` cascade).

### 9.3 Rules (P0 — auto-loaded)

- `.claude/rules/decision-queue.md` — DQ schema-v2; attribution
  integrity; mid-task push; planner Recipe 2 self-resolution.
- `.claude/rules/branch-manager.md` — file-ownership boundaries;
  BM cuts `phase-v1-SL-e` after planner ships.
- `.claude/rules/phase-branch.md` — phase-branch +
  PR-into-`governance-v0` flow.
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy`
  mandatory.
- `.claude/rules/governance-log-entry-kind-registry.md` — SL-e
  emits zero new consts; uses 5 existing SL-a-shipped consts + v0
  consts via SL-c/SL-d emit sites.
- `.claude/rules/cargo-output-capture.md` +
  `.claude/rules/no-cargo-output-paste.md` — cargo invocation
  discipline.
- `.claude/rules/pre-phase-harness-audit.md` — Task 0 audit shape.
- `.claude/rules/advisor-orchestrator.md` §G4 classifier (4a/4b/4c
  rows) — SL-e's e2e tasks mirror Case A canonical sibling shape;
  Case C (mixed shapes) is a hard refusal.

### 9.4 Lessons (P0 — bound to §13 decisions)

(See §2 "Lessons that bind §13 decisions" — full enumeration. Most
load-bearing: `feedback_lemmy_error_no_std_error.md` Case A,
`feedback_junior_worker_e2e_edit_hang.md` (via
`feedback_complexity_score_pre_split.md`),
`feedback_plan_stub_uniformity_with_canonical_sibling.md`,
`feedback_complexity_score_pre_split.md`.)

### 9.5 External documentation

- chrono `Duration::hours(i64)` API — used in Test #2 + #3 for the
  rewind-`grace_expires_at` UPDATE.
- diesel `RunQueryDsl` + `AsyncPgConnection` — existing usage.
- diesel `sql_query` for the Test #3 raw-SQL backfill UPDATE.
- testcontainers-rs Postgres lifecycle — existing usage across
  e2e.rs.

---

## 10. Patterns to mirror

Per `feedback_advisor_watchpoint_specificity.md` + DQ #147 (paired
canonical mirrors).

### 10.1 Fixture mod shape — Case A canonical sibling

**SOURCE:** `crates/server/tests/e2e.rs:10980-11924` (`mod
v1_sl_b_fixtures`). Post-SL-c-2 the immediate predecessor is
`crates/server/tests/e2e.rs:11927-12819` (`mod v1_sl_c_fixtures`).
Post-SL-d the most-recent fixture mod is
`crates/server/tests/e2e.rs:12821-13693` (`mod v1_sl_d_fixtures`).

```rust
mod v1_sl_e_fixtures {
  use super::*;
  use activitypub_federation::config::FederationConfig;
  use actix_web::web::{Data, Json};
  use chrono::{DateTime, Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl, insert_into, sql_query, update};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    sponsor_liability_grace::run_grace_check_batch,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment,
    AdminAssignJury,
    RevokeEndorsement,
    SubmitJuryVote,
  };
  use lemmy_api_crud::governance::revoke_endorsement::revoke_endorsement;
  use lemmy_api_utils::context::LemmyContext;
  use lemmy_db_schema::{
    newtypes::{EndorsementId, ModerationCaseId},
    source::governance::{
      endorsement::EndorsementInsertForm,
      moderation_case::ModerationCaseInsertForm,
      sanction::SanctionInsertForm,
      surety::SuretyInsertForm,
    },
  };
  use lemmy_db_schema_file::{
    InstanceId,
    PersonId,
    enums::{
      CaseSeverity, CaseStatus, CaseTargetType, JuryDecision,
      SanctionAction, SanctionScope, SeverityTier,
    },
    schema::{
      endorsement, governance_log, moderation_case, public_case_log,
      reputation_event, sanction, surety,
    },
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_utils::error::LemmyResult;
  use serde_json::Value;

  // Helpers — see §10.2 for shapes.

  // Three tests follow — see Task 1, 2, 3 IMPLEMENT blocks.
}
```

**GOTCHA (Case A — `feedback_lemmy_error_no_std_error.md`):** every
helper signature `-> LemmyResult<T>`; every test fn signature `->
LemmyResult<()>`. Bare `?` propagation. NO `Box<dyn Error>` outer;
NO `.map_err` bridges. The 3-cycle SL-c-2 catch-fire was caused by
stub-shape non-uniformity; SL-e avoids by following Case A.

**GOTCHA (e2e fixture-mod placement):** the new mod opens
IMMEDIATELY AFTER the closing `}` of `mod v1_sl_d_fixtures` at
e2e.rs:13693. Task 1's IMPLEMENT runs:

```bash
grep -n '^mod v1_' crates/server/tests/e2e.rs | tail -1
# EXPECT: <line>:mod v1_sl_d_fixtures
```

then anchors the new `mod v1_sl_e_fixtures` AFTER its closing `}`.

### 10.2 Shared helpers — composed from SL-b/c/d fixture mods

**SOURCE:** `crates/server/tests/e2e.rs:11003-11151` (`mod
v1_sl_b_fixtures` helpers — `seed_endorsement_active`,
`read_endorsement_revoked_at`, `read_surety_revoked_at`,
`read_case_status_and_escape`, `count_log_entries`,
`read_log_payload`).

**SOURCE:** `crates/server/tests/e2e.rs:11956-12041` (`mod
v1_sl_c_fixtures` helpers — `count_log_entries`,
`read_log_payload`, `seed_pending_case`, `seed_active_surety`).

**SOURCE:** `crates/server/tests/e2e.rs:12854-12888` (`mod
v1_sl_d_fixtures::seed_target_with_sureties` — composes
`governance_fixtures::seed_user` + `surety` insert + endorsement
insert).

**SOURCE:** `crates/server/tests/e2e.rs:12891-12996` (`mod
v1_sl_d_fixtures` Test #1 — full jury-panel-to-quorum drive
pattern). Tests #1 + #2 in SL-e reuse this drive shape.

Each helper SL-e duplicates (rather than imports across mods, since
each fixture mod is self-contained per the v1-SL-* pattern):

- `seed_target_with_sureties_and_endorsements(context, instance_id,
  conn, sponsor_count, prefix) -> LemmyResult<(PersonId, Vec<(PersonId, EndorsementId)>)>`
  — composes `seed_user` × (1 + sponsor_count); inserts
  endorsement + surety per sponsor; returns the sponsee + array of
  (sponsor_id, endorsement_id) pairs (Test #1 needs the endorsement_id
  to call `revoke_endorsement`).
- `count_log_entries(conn, kind) -> LemmyResult<i64>` (mirror SL-c-2
  helper at e2e.rs:11956-11966).
- `read_log_payload(conn, kind) -> LemmyResult<Option<Value>>`
  (mirror SL-c-2 helper at e2e.rs:11968-11980).
- `drive_jury_to_quorum(context, federation_context, instance_id,
  conn, case_id, decision) -> LemmyResult<()>` — composes
  `admin_assign_jury` + `accept_jury_assignment` × panel_size +
  `submit_jury_vote` × threshold_count; mirrors the inline pattern
  in SL-d Test #1 at e2e.rs:12918-13012.

**GOTCHA (helper duplication is canonical):** the v1-SL-* fixture
mods are self-contained — they duplicate small helpers across mods
rather than factor them out (cross-mod import requires `pub`
visibility and pollutes the module-level public surface of the test
crate). Mirror the duplication pattern; do NOT factor helpers into
a `mod v1_sl_shared` or similar.

### 10.3 Inside-test handler invocations — real-HTTP discipline

**SOURCE:** `crates/server/tests/e2e.rs:11154-11199` (`mod
v1_sl_b_fixtures` Test #1 — `revoke_endorsement` handler call
pattern).

```rust
let response = revoke_endorsement(
  Json(RevokeEndorsement {
    endorsement_id,
    reason: "test_sponsor_revoke_during_window".to_string(),
  }),
  context.clone(),
  sponsor_view,
)
.await?
.into_inner();

assert_eq!(response.endorsement_id, endorsement_id);
assert!(response.liability_chain_severed_for_cases.contains(&case_id));
```

**GOTCHA (sponsor_view setup):** `LocalUserView` for the sponsor is
required by the handler's `local_user_view` parameter; obtain via
`governance_fixtures::seed_user(...).await?` which returns
`(PersonId, LocalUserView)`.

**SOURCE:** `crates/server/tests/e2e.rs:12954-13012` (`mod
v1_sl_d_fixtures` Test #1 — `admin_assign_jury` + jury-accept-loop
+ `submit_jury_vote` pattern).

### 10.4 Scheduler-tick invocation — `run_grace_check_batch`

**SOURCE:** `crates/server/tests/e2e.rs:12046-12148` (`mod
v1_sl_c_fixtures` Test #1).

```rust
let prev_disable = std::env::var_os("BREHON_DISABLE_GRACE_CHECK_JOB");
unsafe {
  std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", "1");
}

// ... test setup + drive ...

let outcome = run_grace_check_batch(&context).await?;
assert_eq!(outcome.cases_processed, 1);
assert_eq!(outcome.fired, 1);

unsafe {
  match prev_disable {
    Some(val) => std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", val),
    None => std::env::remove_var("BREHON_DISABLE_GRACE_CHECK_JOB"),
  }
}
```

**GOTCHA (env var thread-safety):** the `unsafe { std::env::set_var(...) }`
block is required by Rust 1.85+'s tightened env-var safety. Mirror
the SL-c-2 pattern verbatim.

**GOTCHA (env var restoration):** if the test panics between set
and restore, the env var leaks to subsequent tests in the same
process. The `prev_disable` capture + restoration block is the
canonical guard; test bodies should use `?` instead of `unwrap()`
to ensure the restoration block is reachable.

### 10.5 Force-rewind `grace_expires_at` (Test #2 binding)

**SOURCE:** mirrored from SL-c-2 Test #1's seed pattern at
e2e.rs:11982-12024 (where `seed_pending_case` constructs a case
with `grace_expires_at = now + grace_offset`, with `grace_offset`
negative to seed an already-expired case).

For SL-e Test #2, we want to drive the SL-d producer (which sets
`grace_expires_at = now + grace_window_for_severity(severity)`),
then rewind the case's `grace_expires_at` to a past time. The
update statement:

```rust
update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
  .set(moderation_case::grace_expires_at.eq(Some(Utc::now() - Duration::minutes(1))))
  .execute(&mut conn)
  .await?;
```

**GOTCHA (no severity-snapshot regression):** the rewind only
overwrites `grace_expires_at`; `case.severity` remains the original
snapshot value. Per ADR-010 won't-disadvantage rule (PRD §4.3), the
severity used for any subsequent re-computation reads from
`case.severity` — Test #2 doesn't violate this since the rewind
doesn't touch severity.

**GOTCHA (lock-ordering vs scheduler FOR UPDATE):** SL-c's
scheduler reads candidates with `FOR UPDATE` per
`sponsor_liability_grace.rs:147-154`. The test's `update(...)`
runs serially against the scheduler invocation (test thread →
manual `run_grace_check_batch` call); no concurrent FOR UPDATE
contention.

### 10.6 Programmatic backfill UPDATE (Test #3 binding)

**SOURCE:** PRD §8.4 (verbatim SQL); migration
`migrations/2026-04-22-…_add_sponsor_liability_grace_window/up.sql`
(SL-a Task 1 shipped — the production version).

```rust
diesel::sql_query(
  "UPDATE moderation_case
   SET status = 'SponsorLiabilityPending',
       grace_expires_at = decided_at + INTERVAL '24 hours'
   WHERE status = 'Decided'
     AND decided_at IS NOT NULL
     AND decided_at > now() - INTERVAL '24 hours'
     AND target_person_id IS NOT NULL
     AND id IN (
       SELECT mc.id
       FROM moderation_case mc
       WHERE EXISTS (
         SELECT 1 FROM surety s
         WHERE s.sponsored_id = mc.target_person_id
           AND s.revoked_at IS NULL
       )
       AND EXISTS (
         SELECT 1 FROM sanction sa
         WHERE sa.case_id = mc.id
       )
       AND NOT EXISTS (
         SELECT 1 FROM reputation_event re
         WHERE re.source_case_id = mc.id
           AND re.reason = 'sponsor_liability_applied'
       )
     )"
)
.execute(&mut conn)
.await?;
```

**GOTCHA (raw SQL discipline):** `INTERVAL '24 hours'` is Postgres-
specific syntax not cleanly representable in Diesel's typed query
builder for an UPDATE; raw `sql_query` is canonical. Test #3 cites
PRD §8.4 in a comment immediately before the query block.

**GOTCHA (Test #3 setup must satisfy the WHERE clause):** to verify
the backfill picks up the seeded case, the seed must satisfy:
`status = Decided` ✓, `decided_at` non-null + within 24h of now ✓,
`target_person_id` non-null ✓, surety exists active ✓, sanction
exists ✓, no `reputation_event` with `reason = sponsor_liability_applied`
✓ (because no SL-d producer ran). The IMPLEMENT enumerates each
condition.

### 10.7 governance_log row count + payload assertion pattern

**SOURCE:** `crates/server/tests/e2e.rs:907`
(`governance_log_hash_chain_holds`) +
`crates/server/tests/e2e.rs:12107-12121` (SL-c-2 Test #1).

```rust
let count: i64 = governance_log::table
  .filter(governance_log::entry_kind.eq("sponsor_liability_pending"))
  .count()
  .get_result(conn)
  .await?;
assert_eq!(count, 1);

let payload = read_log_payload(conn, "sponsor_liability_pending")
  .await?
  .expect("sponsor_liability_pending payload exists");
assert!(payload["target_pseudonym"].is_string());
assert_eq!(payload["case_id"].as_i64(), Some(i64::from(case_id.0)));
```

**GOTCHA (R1 — i64 count comparison):** `.count()` returns `i64`;
compare against literal (no `i64::from` needed for literal; needed
when comparing against `usize::len()` of a Vec).

**GOTCHA (R1 — i64 case_id):** `case_id.0` is `i32`; to compare
against `payload["case_id"].as_i64()` (returns `Option<i64>`), wrap
with `Some(i64::from(case_id.0))`.

---

## 11. Files to change

### `crates/server` crate

- `crates/server/tests/e2e.rs` (currently 13,693 lines): append new
  `mod v1_sl_e_fixtures` AFTER `mod v1_sl_d_fixtures` (closes near
  line 13,693 — verify at task-start). Within the mod, ship 3
  tests via 3 anchor-Edit tasks (Tasks 1-3). Task 1 ships the mod
  shell + shared helpers + Test #1; Tasks 2-3 anchor-insert
  subsequent tests INSIDE the same mod; Task 3's anchor-Edit ends
  with the closing `}` of the mod. **Tasks 1-3**.

### Meta files (reports)

- `.claude/PRPs/reports/v1-SL-e-retro.md` — CREATE retro per
  `feedback_retro_not_report.md` +
  `feedback_four_role_retro_signals.md` +
  `feedback_retro_task_complexity_score.md`. **Task 4**.

### Files explicitly NOT touched

- **Zero edits to production code.** SL-e is tests-only. Specifically:
  - `crates/api/api/src/governance/**` — no edits.
  - `crates/api/api_crud/src/governance/**` — no edits.
  - `crates/api/api_common/src/governance.rs` — no edits.
  - `crates/api/routes/src/lib.rs` — no edits.
  - `crates/db_schema/src/**` — no edits.
  - `crates/db_schema_file/src/**` — no edits.
  - `crates/db_views*/src/**` — no edits.
  - `crates/routes/src/**` — no edits.
  - `crates/lemmy_server/src/**` — no edits.
- **Zero new migrations.** `migrations/**` — no new directories or
  files.
- **Zero workflow YAML edits.** `.github/workflows/**` — no edits.
- **Zero registry edits.** `.claude/rules/governance-log-entry-kind-registry.md`
  — no edits (all 5 SL ENTRY_KIND_* consts shipped + active per
  SL-a/b/c/d retro markers).
- **Zero plan / PRD / ADR edits.** `.claude/PRPs/{plans,prds}/**`,
  `docs/brehon-law-inspired-network/**` — no edits.
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
  `.coderabbit.yaml` — no edits.

At plan-approval time:
```bash
git diff governance-v0..phase-v1-SL-e --stat
# EXPECT: 2 files changed
#   crates/server/tests/e2e.rs               | +<lines>
#   .claude/PRPs/reports/v1-SL-e-retro.md    | +<lines> (new file)
```

---

## 12. NOT building in v1-SL-e

- **Restoration-during-window-escapes test** — out per SL-c DQ #145
  LOCKED. Restoration producer endpoint lives in
  restorative-mechanics-v1 PRD; that PRD's plan will add the matching
  e2e test alongside its producer. **Do NOT attempt to test the
  SL-c stub restoration-escape branch** — the stub just returns
  Fire, which is covered by Test #2's window-expiry assertions.
- **Optional 4th test — admin-overridden status flip on Pending
  case** — out per §4.3. `admin_close_case` already enumerates the
  Pending/Fired/Escaped variants in its exhaustive match (PRD §3.3
  + verified at `admin_close_case.rs:65-80`); admin-flow tests live
  in admin-dashboard-v1 sub-phases.
- **Cross-instance federation of sponsor_liability_pending events**
  — out per ADR-014 + PRD §11.5 (v2 federation work). Tests #1 + #2
  include defensive "no federation outbound on Pending transition"
  assertion shapes but do NOT exercise federation.
- **Step-up auth on admin revocation** — out per PRD §12.3 (v2
  reservation).
- **Sponsor notification UX** — out per PRD §13 OQ-V1-SL-03 (v3
  polish).
- **Multi-sponsor `all_revocation` / `majority_revocation` rules at
  vote-tally time** — out per PRD §13 OQ-V1-SL-01. SL-b's revocation
  uses `any_revocation` default; SL-e Test #1 exercises the default
  (single-revocation severs immediately).
- **Sponsor-of-sponsor liability chain depth (multi-hop)** — out per
  PRD §13 OQ-V1-SL-02 (v2 candidate).
- **`liability_escape_reason` JSONB schema versioning beyond v=1** —
  out per PRD §13 OQ-V1-SL-05 (v=1 shipped by SL-b at PR #119;
  Test #1 asserts `version: 1` in the JSONB).
- **Negative-band reputation reintegration ceremonies (OQ-021)** —
  out (separate v1+ PRD).
- **`Restoration` variant refinement into `Apology` /
  `ContentCorrection` / `CommunityService`** — out (jury-mechanics-
  v1 PRD's territory, OQ-003 amendment).
- **New ENTRY_KIND_* consts** — all needed shipped in SL-a (5
  consts, all active post-SL-d retro markers).
- **New `governance_config` seeds** — all 13 SL-a-seeded keys are
  read by SL-e tests; none added.
- **New CaseStatus variants** — SL-a shipped 3 (the v1 lifecycle
  triad).
- **New `actor_pseudonym` patterns** — Tests assert existing
  `actor_pseudonym_helper::get_or_create` outputs (called by
  SL-b/SL-c/SL-d emit sites); zero new pseudonym logic.
- **Schema migrations** — none.
- **Backfill of v0 cases (production deploy)** — SL-a Task 1
  shipped; Test #3 invokes the UPDATE programmatically against
  test-seeded data.
- **Concurrent-handler race tests (SL-d producer + SL-b revoke +
  SL-c scheduler colliding on the same case)** — defended by FOR
  UPDATE locks per PR #98 cr-2 + SL-c per-case transactions; no
  e2e test (single-threaded test harness can't exercise reliably).
  If concurrency surfaces as a bug, it's an SL-c or SL-d fix-impl
  (not SL-e re-plan).

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task**. SL-e ships 3
impl tasks + Task 0 pre-flight + Task 4 retro = 5 tasks total. No
`[P]` cohorts (Tasks 1-3 all `modifies: crates/server/tests/e2e.rs`
so YAML overlap rule refuses cohort dispatch — they ship serially.
Task 4 retro depends on Tasks 1-3 commits being on the phase
branch). Task 0 is always non-`[P]`.

> **Cohort dispatch (advisor-side):** No `[P]` cohorts in SL-e.
> All tasks dispatch serially per `advisor-orchestrator.md`.
> Per-task `HANDOVER:` commit trailer recommended where the next
> task benefits (Task 1 → Task 2 share the `seed_target_with_sureties_and_endorsements`
> helper Task 1 introduces).

> **Shape G (Layer G2 push-and-exit):** §13 task bodies do NOT
> inline cargo invocations. Each task ends with a push to the
> worker branch; the impl-task subagent writes a
> `kind: "validate-pending"` DQ entry referencing
> `cargo-validate-workspace.yml` per
> `.claude/rules/decision-queue.md` schema-v2.

### Task 0: Pre-flight harness audit + branch verification + SL-a/SL-b/SL-c/SL-d state confirmation

**Goal:** verify environment + branch (`phase-v1-SL-e`) + SL-a +
SL-b + SL-c + SL-d shipped state all merged on `governance-v0`.

**FILES:**

```yaml
creates: []
modifies: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes):**

```bash
# Probe 0 — Docker daemon (e2e harness uses testcontainers-rs)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING — start Docker Desktop / dockerd before continuing"; exit 1; }

# Probe -1 — submodule init (Linux/Junior worktree quirk)
git submodule status > /tmp/sl-e-task0-submodule.log 2>&1
if grep -q '^-' /tmp/sl-e-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/sl-e-task0-submodule-init.log 2>&1
  echo "submodule init exit: $?"
fi

# Probe 1 — branch verification
git branch --show-current
# EXPECT: phase-v1-SL-e (BM-task cuts before Task 1, AFTER plan ships to governance-v0)

# Probe 2 — SL-a state confirmation: 3 CaseStatus variants present
rg -n 'SponsorLiabilityPending|SponsorLiabilityFired|SponsorLiabilityEscaped' crates/db_schema_file/src/enums.rs | head
# EXPECT: 3+ matches near lines 416/421/427

# Probe 3 — SL-a state confirmation: schema columns present
rg -n 'grace_expires_at -> Nullable<Timestamptz>|liability_escape_reason -> Nullable<Jsonb>' crates/db_schema_file/src/schema.rs | head
# EXPECT: lines around 798-800

# Probe 4 — SL-a state confirmation: ENTRY_KIND consts declared (5 SL consts)
rg -n 'ENTRY_KIND_SPONSOR_LIABILITY_PENDING|ENTRY_KIND_SPONSOR_LIABILITY_FIRED|ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED|ENTRY_KIND_ENDORSEMENT_REVOKED|ENTRY_KIND_RESTORATION_COMPLETED' crates/db_schema/src/source/governance/governance_log.rs | head
# EXPECT: 5 matches near lines 195-220

# Probe 5 — SL-a state confirmation: shim re-exports (5 SL consts)
rg -n 'ENTRY_KIND_SPONSOR_LIABILITY_PENDING|ENTRY_KIND_SPONSOR_LIABILITY_FIRED|ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED|ENTRY_KIND_ENDORSEMENT_REVOKED|ENTRY_KIND_RESTORATION_COMPLETED' crates/api/api/src/governance/governance_log.rs | head
# EXPECT: 5 matches in pub use block

# Probe 6 — SL-a state confirmation: config keys + defaults seeded
rg -n 'DEFAULT_LIABILITY_GRACE_WINDOW_MINOR_HOURS|DEFAULT_LIABILITY_GRACE_WINDOW_MODERATE_HOURS|DEFAULT_LIABILITY_GRACE_WINDOW_SEVERE_HOURS' crates/api/api/src/governance/config.rs | head
# EXPECT: 3+ const declarations near lines 925-945

# Probe 7 — SL-b state confirmation: revoke_endorsement handler shipped
test -f crates/api/api_crud/src/governance/revoke_endorsement.rs && echo "SL-B SHIPPED" || { echo "SL-B NOT YET MERGED — surface to advisor"; exit 1; }
rg -n 'pub async fn revoke_endorsement' crates/api/api_crud/src/governance/revoke_endorsement.rs
# EXPECT: 1 line (handler entry)

# Probe 8 — SL-b state confirmation: route registered
rg -n '"/endorsement/revoke"' crates/api/routes/src/lib.rs | head
# EXPECT: 1 line near 525 (POST route)

# Probe 9 — SL-c state confirmation: scheduler module landed
test -f crates/api/api/src/governance/sponsor_liability_grace.rs && echo "SL-C MODULE PRESENT" || { echo "SL-C MODULE NOT FOUND — SL-e cannot proceed"; exit 1; }
rg -n 'pub async fn run_grace_check_batch' crates/api/api/src/governance/sponsor_liability_grace.rs
# EXPECT: 1 line near 127 (scheduler entry)
rg -n 'pub struct GraceCheckBatchOutcome' crates/api/api/src/governance/sponsor_liability_grace.rs
# EXPECT: 1 line near 82 (outcome struct SL-e tests destructure)

# Probe 10 — SL-d state confirmation: compute/fire/wrapper split landed
rg -n 'pub\(crate\) async fn compute_sponsor_liability|pub\(crate\) async fn fire_sponsor_liability|pub\(crate\) async fn apply_sponsor_liability|pub\(crate\) async fn grace_window_for_severity' crates/api/api/src/governance/sponsor_liability.rs
# EXPECT: 4 matches (compute, fire, wrapper, grace-helper)

# Probe 11 — SL-d state confirmation: submit_jury_vote mutation landed (JM-c TODO discharged)
rg -n 'compute_sponsor_liability\b' crates/api/api/src/governance/submit_jury_vote.rs
# EXPECT: 1+ matches (Task 2 of SL-d replaced v0 call with this)
rg -n 'TODO\(v1-sponsor-liability-d\)' crates/api/api/src/governance/submit_jury_vote.rs
# EXPECT: 0 matches (SL-d Task 2 discharged the marker)

# Probe 12 — last fixture mod identification (Task 1 anchor)
rg -n '^mod v1_' crates/server/tests/e2e.rs | tail -1
# EXPECT: <line>:mod v1_sl_d_fixtures (post-SL-d-merge)

# Probe 13 — e2e.rs line count (sanity check)
wc -l crates/server/tests/e2e.rs
# EXPECT: ~13,693 lines (DQ #189 LOCKED at plan-write time)

# Probe 14 — registry markers active for all 5 SL consts
rg -n '\(pending\)' .claude/rules/governance-log-entry-kind-registry.md | rg -E 'SPONSOR_LIABILITY_(PENDING|FIRED|ESCAPED)|ENDORSEMENT_REVOKED'
# EXPECT: 0 lines for SPONSOR_LIABILITY_PENDING (SL-d retro flipped); 0 for SPONSOR_LIABILITY_FIRED (SL-c flipped); 0 for SPONSOR_LIABILITY_ESCAPED (SL-b/c flipped); 0 for ENDORSEMENT_REVOKED (SL-b flipped)
rg -n '\(pending\)' .claude/rules/governance-log-entry-kind-registry.md | rg RESTORATION_COMPLETED
# EXPECT: 1 line (restorative-mechanics-v1 still pending — never SL-e's concern)

# Probe 15 — Shape G workflow YAMLs accessible
ls .github/workflows/cargo-validate-workspace.yml .github/workflows/cargo-test-e2e.yml
# EXPECT: both files exist

# Probe 16 — concurrent-PR check (no other PR touches e2e.rs SL-e-bound region)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path == "crates/server/tests/e2e.rs") | {number, title, headRefName}'
# EXPECT: empty output (no concurrent PRs touching the SL-e target file)

# Probe 17 — DQ pending status (advisory)
python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', [(e['id'], e.get('kind'), e.get('from')) for e in d.get('pending',[])])"
# EXPECT: pending: [] (no pending DQ blockers at SL-e start)

# Probe 18 — SL-d retro present
test -f .claude/PRPs/reports/v1-SL-d-retro.md && echo "SL-D RETRO PRESENT" || echo "SL-D RETRO MISSING — surface to advisor (cross-PR carry-forward chain broken)"
```

**EXPECT:** Probes 0..18 exit 0 (or, for Probe 0/1/7/9, exit 1 with
explicit STOP). Probe 18 is advisory — if SL-d retro is missing,
Task 0 surfaces to advisor but does NOT block (the cross-PR
carry-forward is desirable but not strictly load-bearing for
SL-e's correctness).

**No commit at Task 0** — verification only.

### Task 1: e2e test #1 — Revocation-during-window-escapes (full lane) + open `mod v1_sl_e_fixtures` shell + shared helpers

**ACTION:** in `crates/server/tests/e2e.rs`, append a NEW
`mod v1_sl_e_fixtures` block AFTER `mod v1_sl_d_fixtures` (closes
near line 13,693). Inside the new mod, add the use block + shared
helpers + the first test fn
`revocation_during_window_escapes_full_lane`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # open mod v1_sl_e_fixtures + helpers + test #1
```

**IMPLEMENT (file 1 of 1):** anchor-Edit at file end. Anchor
identification at task-start:
```bash
grep -n '^mod v1_' crates/server/tests/e2e.rs | tail -1
# EXPECT: <line>:mod v1_sl_d_fixtures
```
The new mod opens AFTER the closing `}` of `mod v1_sl_d_fixtures`
(near line 13,693).

**Content shape (Case A canonical sibling mirror at e2e.rs:10980-11924):**

1. Open `mod v1_sl_e_fixtures { use super::*; ... }` block per §10.1
   `use` block.

2. Add shared helpers per §10.2:
   - `seed_target_with_sureties_and_endorsements(context, instance_id,
     conn, sponsor_count, prefix) -> LemmyResult<(PersonId,
     Vec<(PersonId, EndorsementId)>)>` — composes
     `governance_fixtures::seed_user` × (1 + sponsor_count); per
     sponsor: insert `endorsement` (returning id) + insert `surety`;
     return the sponsee + array of (sponsor_id, endorsement_id)
     pairs.
   - `count_log_entries(conn, kind) -> LemmyResult<i64>` (mirror SL-c
     helper at e2e.rs:11956-11966 verbatim).
   - `read_log_payload(conn, kind) -> LemmyResult<Option<Value>>`
     (mirror SL-c helper at e2e.rs:11968-11980 verbatim).
   - `drive_jury_to_quorum(context, federation_context, instance_id,
     conn, case_id, decision) -> LemmyResult<()>` — composes
     `admin_assign_jury` + `accept_jury_assignment` × panel_size +
     `submit_jury_vote` × threshold_count; mirrors the inline
     pattern in SL-d Test #1 at e2e.rs:12918-13012. Returns when
     `response.case_decided == true`.

3. Add Test #1:

   `#[tokio::test] async fn revocation_during_window_escapes_full_lane() -> LemmyResult<()>`:

   **Setup:**
   - `let prev_disable = std::env::var_os("BREHON_DISABLE_GRACE_CHECK_JOB"); unsafe { std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", "1"); }`
   - Bootstrap `LemmyContext` via `governance_fixtures::bootstrap()`.
   - Build `FederationConfig` per SL-d Test #1 pattern at e2e.rs:12902-12909.
   - Seed `Instance` via `Instance::read_or_create`.
   - Seed sponsee + 2 sponsors with active sureties +
     endorsements via `seed_target_with_sureties_and_endorsements`.
   - Seed 5 jury-eligible persons + 1 admin (mirror SL-d Test #1
     at e2e.rs:12919-12931).
   - Seed jury-eligible reputation snapshots via
     `super::v1_jm_b_fixtures::seed_jury_eligible_snapshots`
     (mirror SL-d Test #1 at e2e.rs:12935).
   - Insert case: `target_type = Person`, `target_person_id = sponsee`,
     `severity = CaseSeverity::High` (severe bucket → 168h grace),
     `severity_tier = Some(SeverityTier::Minor)` (Minor panel = 5
     jurors), `status = CaseStatus::Open`, `threshold_score = 1`,
     `reason_code = "v1_sl_e_test_revocation"`.

   **Drive #1 — jury vote to quorum (SL-d producer):**
   - `drive_jury_to_quorum(...)` with `JuryDecision::SuspendCommunityMember`
     (liability-bearing). Returns when case_decided.

   **Assert mid-window:**
   - `case.status == CaseStatus::SponsorLiabilityPending`.
   - `case.grace_expires_at` is `Some(t)` with `|t - (now +
     Duration::hours(168))| < 5 seconds` (severe grace window).
   - `count_log_entries(conn, "sponsor_liability_pending") == 1`.
   - `count_log_entries(conn, "case_decided") == 1` (PRD §11.4
     auditor visibility).
   - `count_log_entries(conn, "sanction_created") == 1`.
   - `governance_log` rows for `sponsor_liability_applied`,
     `sponsor_liability_clamped`, `sponsor_liability_fired`,
     `sponsor_liability_escaped`, `public_log_published`,
     `endorsement_revoked` — ALL `count == 0` (deferred-write
     semantics; revoke not yet called).
   - `reputation_event` rows for sponsors: `count == 0` (deferred).
   - `reputation_event` rows for jurors: `count == 0` (deferred per
     PRD §11.4).
   - `public_case_log` rows: `count == 0` (deferred).
   - **Pseudonym discipline** on `sponsor_liability_pending` payload:
     - `payload["target_pseudonym"].is_string() == true`.
     - `payload["target_pseudonym"].as_str().unwrap() != format!("{}", sponsee.0)`.
     - `payload["sponsors_pseudonyms"].is_array() == true`.
     - `payload["sponsors_pseudonyms"].as_array().unwrap().len() == 2`.

   **Drive #2 — revoke endorsement via real HTTP-call (SL-b consumer-side severance):**
   - Load `LocalUserView` for `sponsor1` (the first sponsor of the
     pair); pass to handler.
   - `let response = revoke_endorsement(Json(RevokeEndorsement { endorsement_id: sponsor1_endorsement_id, reason: "sl-e test revocation".to_string() }), context.clone(), sponsor1_view).await?.into_inner();`

   **Assert post-revocation:**
   - `response.endorsement_id == sponsor1_endorsement_id`.
   - `response.revoked_at` is recent (`< now + 1s`).
   - `response.liability_chain_severed_for_cases.contains(&case_id) == true`.
   - `case.status == CaseStatus::SponsorLiabilityEscaped` (SL-b's
     escape branch fired; default `any_revocation` rule severs on
     first revocation).
   - `case.liability_escape_reason` is `Some(json)` with
     `json["version"] == 1` and `json["reason"] == "sponsor_revoked"`
     and `json["actor_pseudonym"].is_string()` and
     `json["endorsement_id"] == sponsor1_endorsement_id.0`.
   - `count_log_entries(conn, "endorsement_revoked") == 1`.
   - `count_log_entries(conn, "sponsor_liability_escaped") == 1`.
   - `count_log_entries(conn, "sponsor_liability_pending") == 1`
     (unchanged from mid-window — escape doesn't emit another
     pending log).
   - `count_log_entries(conn, "sponsor_liability_fired") == 0`
     (escape path skips fire).
   - `count_log_entries(conn, "sponsor_liability_applied") == 0`
     (no reputation events fired).
   - `reputation_event` rows for sponsors: STILL `count == 0` (escape
     branch never fires `apply_sponsor_liability`).
   - `public_case_log` rows: STILL `count == 0` (per PRD §11.4
     escape branch never publishes).
   - **Pseudonym discipline** on `endorsement_revoked` payload:
     - `payload["revoker_pseudonym"].is_string() == true`.
     - `payload["revoker_pseudonym"].as_str().unwrap() != format!("{}", sponsor1.0)`.
     - `payload["target_pseudonym"].is_string() == true`.
   - **Pseudonym discipline** on `sponsor_liability_escaped` payload:
     - `payload["actor_pseudonym"].is_string() == true`.
     - `payload["reason"].as_str().unwrap() == "sponsor_revoked"`.

   **Drive #3 — scheduler tick (SL-c consumer skip):**
   - `let outcome = run_grace_check_batch(&context).await?;`

   **Assert scheduler skipped the case:**
   - The case is NO LONGER in `SponsorLiabilityPending` status (it's
     `SponsorLiabilityEscaped`); per SL-c scheduler's filter
     `status.eq(CaseStatus::SponsorLiabilityPending)`, the case is
     NOT picked up.
   - `outcome.cases_processed == 0` (the seed case is not in the
     candidate set; no other Pending cases exist in this test).
   - `outcome.fired == 0`. `outcome.escaped == 0`. `outcome.skipped == 0`.
   - Final state unchanged: `case.status == CaseStatus::SponsorLiabilityEscaped`.
   - `count_log_entries(conn, "sponsor_liability_fired") == 0`
     (scheduler skip — no new fire log).

   **Restore env var:**
   - `unsafe { match prev_disable { Some(val) => std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", val), None => std::env::remove_var("BREHON_DISABLE_GRACE_CHECK_JOB"), } }`

   **Return `Ok(())`.**

**MIRROR:** §10.1 (fixture mod shape, Case A canonical sibling);
§10.2 (shared helpers — SL-b + SL-c helpers); §10.3 (real-HTTP
revocation pattern); §10.4 (scheduler-tick invocation); §10.7
(governance_log row-count + payload assertion pattern).

**GOTCHA (R4):** test fn name lowercase snake_case
(`revocation_during_window_escapes_full_lane`).

**GOTCHA (Case A canonical sibling — `feedback_lemmy_error_no_std_error.md`):**
read `crates/server/tests/e2e.rs:11001-11924` (`mod v1_sl_b_fixtures`)
in full at task-start. Mirror the `LemmyResult<T>` outer pattern
verbatim. Helper signatures: `-> LemmyResult<T>`. Test fn signature:
`-> LemmyResult<()>`. Bare `?` propagation. NO `Box<dyn Error>`
outer. NO `.map_err` bridges.

**GOTCHA (anchor-Edit discipline per `feedback_junior_worker_e2e_edit_hang.md`):**
this Task 1 ships the `mod v1_sl_e_fixtures` shell + shared helpers
+ 1 test in ONE Edit at file end. Tasks 2-3 anchor-insert AFTER
this task's test body, INSIDE the same mod. Do NOT bulk-edit
multiple tests in one Edit. e2e.rs is **13,693 lines** post-SL-d.

**GOTCHA (Watchpoint #5 — pseudonym discipline):** every governance_log
payload Test #1 asserts on includes BOTH a positive
`is_string()` check AND a negative `assert_ne!` against
`format!("{}", raw_id.0)` for ADR-015 defense-in-depth.

**GOTCHA (Watchpoint #2 — grace_expires_at value assertion):** the
expected value depends on `case.severity`. Test #1 seeds
`CaseSeverity::High` → severe bucket → 168h default per PRD §10.
Assertion with 5s tolerance handles testcontainers-rs clock skew.

**GOTCHA (env var thread-safety):** Rust 1.85+ requires `unsafe {
std::env::set_var(...) }` blocks. Mirror SL-c-2 Test #1 pattern at
e2e.rs:12046-12049 + 12141-12146 verbatim. Restore the env var via
the same `prev_disable` pattern.

**GOTCHA (governance_log queries in tests):** use diesel
`governance_log::table.filter(...)` per existing JM-c pattern at
`crates/server/tests/e2e.rs:907`. The `count_log_entries` helper
encapsulates the boilerplate.

**GOTCHA (sponsor_view for revoke handler):** `revoke_endorsement`
requires `LocalUserView` for the caller. Obtain via
`governance_fixtures::seed_user(...).await?` (returns `(PersonId,
LocalUserView)`). The PersonId returned MUST match `sponsor1` (the
endorsement's `from_person_id`) so SL-b's caller-is-sponsor check
at `revoke_endorsement.rs` succeeds. **GOTCHA:** if the sponsor's
`LocalUserView` was discarded during seed, re-load via
`LocalUserView::read_person(&mut context.pool(), sponsor1).await?`
(mirror SL-d Test #1 at e2e.rs:12968).

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-e): e2e test #1 — revocation-during-window-escapes (full lane) + mod v1_sl_e_fixtures shell (task 1)`

### Task 2: e2e test #2 — Window-expiry-fires (full lane)

**ACTION:** anchor-insert
`window_expiry_fires_full_lane` test fn inside
`mod v1_sl_e_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #2 inside mod v1_sl_e_fixtures
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 1's test fn,
INSIDE the same mod block.

`#[tokio::test] async fn window_expiry_fires_full_lane() -> LemmyResult<()>`:

**Setup:**
- `let prev_disable = std::env::var_os("BREHON_DISABLE_GRACE_CHECK_JOB"); unsafe { std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", "1"); }`
- Bootstrap `LemmyContext` + FederationConfig + Instance (mirror
  Task 1).
- Seed sponsee + 2 sponsors with active sureties (no endorsement IDs
  needed for Test #2 — no revocation) via
  `seed_target_with_sureties_and_endorsements`.
- Seed jury panel + reputation snapshots (mirror Task 1).
- Insert case: `severity = CaseSeverity::Low` (Minor bucket → 24h
  default grace), `severity_tier = Some(SeverityTier::Minor)`,
  `status = CaseStatus::Open`, `threshold_score = 1`, `reason_code =
  "v1_sl_e_test_expiry"`.

**Drive #1 — jury vote to quorum (SL-d producer):**
- `drive_jury_to_quorum(..., JuryDecision::ContentRemoval).await?;`

**Assert post-quorum (mid-window):**
- `case.status == CaseStatus::SponsorLiabilityPending`.
- `case.grace_expires_at` is `Some(t)` with `|t - (now + Duration::hours(24))| < 5 seconds`.
- `count_log_entries(conn, "sponsor_liability_pending") == 1`.
- `count_log_entries(conn, "case_decided") == 1`.
- `count_log_entries(conn, "sanction_created") == 1`.
- `reputation_event` rows for sponsors: `count == 0` (deferred).
- `reputation_event` rows for jurors: `count == 0` (deferred per PRD §11.4).
- `public_case_log` rows: `count == 0` (deferred).
- **Pseudonym discipline** on `sponsor_liability_pending` payload
  (mirror Task 1 assertions).

**Drive #2 — force-rewind `grace_expires_at`:**

```rust
update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
  .set(moderation_case::grace_expires_at.eq(Some(Utc::now() - Duration::minutes(1))))
  .execute(&mut conn)
  .await?;
```

(Per §10.5 GOTCHA — only `grace_expires_at` overwritten;
`case.severity` snapshot preserved per ADR-010.)

**Drive #3 — scheduler tick (SL-c consumer fire):**
- `let outcome = run_grace_check_batch(&context).await?;`

**Assert post-fire:**
- `outcome.cases_processed == 1`.
- `outcome.fired == 1`. `outcome.escaped == 0`. `outcome.skipped == 0`.
- `case.status == CaseStatus::SponsorLiabilityFired`.
- `case.liability_escape_reason` IS `None` (fire branch never sets).
- 2 `reputation_event` rows with `source_case_id == case_id` and
  `reason == "sponsor_liability_applied"` (1 per sponsor).
- `count_log_entries(conn, "sponsor_liability_applied") == 2` (per
  sponsor).
- `count_log_entries(conn, "sponsor_liability_clamped") == 0` (no
  clamp engaged in this seed shape).
- `count_log_entries(conn, "sponsor_liability_fired") == 1` (summary).
- `count_log_entries(conn, "sponsor_liability_escaped") == 0`.
- 1 `public_case_log` row (SL-c emits at fire time per PRD §11.4).
- `reputation_event` rows for jurors: `count == panel_size`
  (deferred writes fire at scheduler tick).
- `reputation_event` rows for reporter: `count == 0` if case has no
  creator_id; `count == 1` if case has creator_id (seed shape uses
  creator_id == None — `count == 0`).
- **Pseudonym discipline** on `sponsor_liability_fired` payload:
  - `payload["target_pseudonym"].is_string() == true`.
  - `payload["target_pseudonym"].as_str().unwrap() != format!("{}", sponsee.0)`.
  - `payload["sponsor_count"].as_u64() == Some(2)`.
  - `payload["case_id"].as_i64() == Some(i64::from(case_id.0))`.
- **Pseudonym discipline** on `sponsor_liability_applied` payload
  (first row):
  - `payload["sponsor_pseudonym"].is_string() == true`.
  - `payload["sponsor_pseudonym"].as_str().unwrap()` matches neither
    `format!("{}", sponsor1.0)` nor `format!("{}", sponsor2.0)`.

**Restore env var.**

**Return `Ok(())`.**

**MIRROR:** §10.1 (fixture mod, Case A); §10.3 (jury-vote drive);
§10.4 (scheduler-tick invocation); §10.5 (force-rewind grace_expires_at);
§10.7 (assertion pattern); SL-c-2 Test #1 (fire-branch assertions)
at e2e.rs:12046-12148.

**GOTCHA (Watchpoint #2 — time-handling discipline):** the
rewind-grace-via-UPDATE approach (§10.5) is canonical;
`tokio::time::sleep` is rejected per §4.3.

**GOTCHA (Watchpoint #3 — env var):** `BREHON_DISABLE_GRACE_CHECK_JOB=1`
must be set BEFORE the producer drive; otherwise the background
clokwerk scheduler may pick up the case between the producer's
Pending transition and the test's manual `run_grace_check_batch`
invocation.

**GOTCHA (case.severity preserved after rewind):** the rewind UPDATE
sets ONLY `grace_expires_at`; `case.severity` (Low → minor bucket
→ 24h default) is the snapshot that SL-d wrote at the Pending
transition. SL-c's fire path does NOT re-derive severity from
config — it reads `case.severity` (and `case.severity_tier`) from
the case row. Per ADR-010 won't-disadvantage rule + PRD §4.3.

**GOTCHA (reporter reputation_event):** the case-seed shape uses
`creator_id = None` (no reporter in this test). PRD §11.4 says
reporter reputation_event fires on Decided path; on Pending →
Fired round-trip, the reporter event fires at fire time per SL-c.
If `creator_id == None`, no reporter row writes. Assertion accounts.

**GOTCHA (R1 — case_id i32 vs i64):** `case_id.0` is `i32`;
`payload["case_id"].as_i64()` returns `Option<i64>`. Use
`Some(i64::from(case_id.0))` per R1.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-e): e2e test #2 — window-expiry-fires (full lane) (task 2)`

### Task 3: e2e test #3 — Backfill-of-mid-flight (v0→v1 deploy scenario) + close `mod v1_sl_e_fixtures`

**ACTION:** anchor-insert
`backfill_of_mid_flight_v0_to_v1_deploy` test fn inside
`mod v1_sl_e_fixtures`. This is the LAST test in the mod; the
closing `}` of the mod follows.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #3 inside mod v1_sl_e_fixtures (closes the mod)
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 2's test fn,
INSIDE the same mod block. After the test's `Ok(())` line, the
closing `}` of `mod v1_sl_e_fixtures` follows immediately.

`#[tokio::test] async fn backfill_of_mid_flight_v0_to_v1_deploy() -> LemmyResult<()>`:

**Setup:**
- `let prev_disable = std::env::var_os("BREHON_DISABLE_GRACE_CHECK_JOB"); unsafe { std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", "1"); }`
- Bootstrap `LemmyContext` + Instance (FederationConfig not needed
  — no jury vote in Test #3; no producer drive).
- Seed sponsee + 1 sponsor with active surety + endorsement.
- Insert pre-deploy case (simulating v0 mid-flight at v1 deploy
  time):
  - `target_type = Person`.
  - `target_person_id = Some(sponsee)`.
  - `status = CaseStatus::Decided` (v0 immediate-Decided pattern).
  - `severity = CaseSeverity::Medium` (moderate bucket — irrelevant
    for backfill, which hardcodes 24h).
  - `severity_tier = Some(SeverityTier::Minor)`.
  - `reason_code = "v1_sl_e_test_backfill"`.
  - `decided_at = None` initially — set explicitly after insert
    (since `decided_at` may not be settable via InsertForm — verify
    at task-start; if so, use UPDATE post-insert).
- Update case: `decided_at = Some(now - Duration::hours(23) - Duration::minutes(30))`
  (within 24h backfill window per PRD §8.4; gives `grace_expires_at
  = (now - 23h30m) + 24h = now + 30m`, in the future post-backfill).
- Insert sanction for the case:
  - `case_id = case_id`, `scope = SanctionScope::Community`,
    `action = SanctionAction::ContentRemoval`,
    `target_person_id = Some(sponsee)`, `active = Some(true)`.
- **Pre-backfill assertion:** verify the seed satisfies the §8.4
  WHERE clause:
  - `case.status == CaseStatus::Decided`.
  - `case.decided_at` IS `Some(t)` with `t > now - 24h`.
  - `case.target_person_id` IS `Some(sponsee)`.
  - 1 active surety for sponsee.
  - 1 sanction row for case.
  - 0 `reputation_event` rows with `source_case_id == case_id` AND
    `reason == "sponsor_liability_applied"` (the §8.4 NOT EXISTS
    guard).

**Drive #1 — programmatic backfill UPDATE (SL-a migration backfill SQL):**

```rust
// Per PRD §8.4 — verbatim. Comment cites the source location.
let _affected = diesel::sql_query(
  "UPDATE moderation_case
   SET status = 'SponsorLiabilityPending',
       grace_expires_at = decided_at + INTERVAL '24 hours'
   WHERE status = 'Decided'
     AND decided_at IS NOT NULL
     AND decided_at > now() - INTERVAL '24 hours'
     AND target_person_id IS NOT NULL
     AND id IN (
       SELECT mc.id
       FROM moderation_case mc
       WHERE EXISTS (
         SELECT 1 FROM surety s
         WHERE s.sponsored_id = mc.target_person_id
           AND s.revoked_at IS NULL
       )
       AND EXISTS (
         SELECT 1 FROM sanction sa
         WHERE sa.case_id = mc.id
       )
       AND NOT EXISTS (
         SELECT 1 FROM reputation_event re
         WHERE re.source_case_id = mc.id
           AND re.reason = 'sponsor_liability_applied'
       )
     )"
)
.execute(&mut conn)
.await?;
```

**Assert post-backfill (Phase A — backfill set Pending + future grace):**
- `case.status == CaseStatus::SponsorLiabilityPending`.
- `case.grace_expires_at` IS `Some(t)` with `|t - (decided_at + Duration::hours(24))| < 1 second`
  (exact match within 1s tolerance for clock skew).
- 0 governance_log rows for any SL kind (the backfill UPDATE writes
  no log entries — it's a one-shot migration query, not a runtime
  handler).
- 0 `reputation_event` rows for sponsors (no scheduler fire yet).

**Drive #2 — force-rewind `grace_expires_at` to past (per §8.2 timing note):**

```rust
update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
  .set(moderation_case::grace_expires_at.eq(Some(Utc::now() - Duration::minutes(1))))
  .execute(&mut conn)
  .await?;
```

(The original `grace_expires_at = decided_at + 24h = now + 30m`
is in the future; rewind to past so the scheduler picks up.)

**Drive #3 — scheduler tick (SL-c fires the backfilled case):**
- `let outcome = run_grace_check_batch(&context).await?;`

**Assert post-fire (Phase B — scheduler resolves backfilled case):**
- `outcome.cases_processed == 1`.
- `outcome.fired == 1`. `outcome.escaped == 0`. `outcome.skipped == 0`.
- `case.status == CaseStatus::SponsorLiabilityFired`.
- 1 `reputation_event` row for sponsor (1 sponsor seeded).
- `count_log_entries(conn, "sponsor_liability_applied") == 1`.
- `count_log_entries(conn, "sponsor_liability_fired") == 1`.
- `count_log_entries(conn, "sponsor_liability_escaped") == 0` (no
  revocation; fire path).
- 1 `public_case_log` row (SL-c emits at fire time).
- **Pseudonym discipline** on `sponsor_liability_applied` payload:
  - `payload["sponsor_pseudonym"].is_string() == true`.
  - `payload["sponsor_pseudonym"].as_str().unwrap() != format!("{}", sponsor.0)`.

**Restore env var.**

**Return `Ok(())`.**

**Close `mod v1_sl_e_fixtures`:** the closing `}` follows
immediately after Test #3's body (mirror `mod v1_sl_d_fixtures`
shape — last test ends with `Ok(())` then `}` of the test fn then
`}` of the mod).

**MIRROR:** §10.6 (programmatic backfill UPDATE); PRD §8.4 SQL;
§10.4 (scheduler-tick invocation); §10.5 (force-rewind grace_expires_at).

**GOTCHA (Watchpoint #6 — verbatim backfill SQL):** copy PRD §8.4
SQL verbatim into the `diesel::sql_query(...)` call. Do NOT
paraphrase. Comment immediately before the query block cites PRD
§8.4 explicitly.

**GOTCHA (decided_at must be settable):** check at task-start
whether `ModerationCaseInsertForm` accepts `decided_at`. Per the
SL-c-2 fixture pattern at e2e.rs:11982-12008, `decided_at` is set
via UPDATE post-insert (the InsertForm doesn't include
`decided_at`). Mirror that pattern.

**GOTCHA (backfill timing per §8.2 note):** the §8.4 backfill SQL
gives `grace_expires_at = decided_at + 24h`. With `decided_at` =
`now - 23h30m`, `grace_expires_at` = `now + 30m` — IN THE FUTURE.
The scheduler's filter `grace_expires_at.le(Some(now))` does NOT
pick the case up immediately. Test #3 must rewind
`grace_expires_at` to past BEFORE the scheduler tick (Drive #2)
to exercise the fire path. The pre-rewind state validates the
backfill UPDATE; the post-rewind state validates the scheduler
fire — both are SL-e's targets.

**GOTCHA (rewind-as-test-artifact):** the rewind UPDATE is a
test artifact, not a production behaviour. PRD §8.4's backfill
intent is that mid-flight cases get a 24h grace post-deploy; the
scheduler picks them up 24h later. Test #3's rewind compresses
that 24h into milliseconds for test purposes. The IMPLEMENT comment
explicitly notes the rewind is a test artifact + cites PRD §8.4.

**GOTCHA (single-sponsor vs multi-sponsor):** Test #3 uses 1
sponsor for minimal seed; the lane behavior is identical for N
sponsors but the assertion shape is simpler. Test #1 + #2 cover
the 2-sponsor case.

**GOTCHA (closing the mod):** after Test #3's `Ok(())` and the
test fn's closing `}`, the `mod v1_sl_e_fixtures` block's closing
`}` follows immediately. Future v1-SL-* fixture mods open AFTER
this `}` (mirror existing pattern at SL-d → SL-e transition).

**GOTCHA (Watchpoint #7 — no edits outside e2e.rs):** at task-end:
```bash
git diff governance-v0..HEAD --stat
```
EXPECT: ONE file: `crates/server/tests/e2e.rs | +<lines>`. If
any other file appears (especially under `crates/`, `migrations/`,
`.github/workflows/`, `docs/`), STOP and surface.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-e): e2e test #3 — backfill-of-mid-flight (v0→v1 deploy) + close mod v1_sl_e_fixtures (task 3)`

### Task 4: Retro

**Goal:** author retro per `feedback_retro_not_report.md` +
`feedback_four_role_retro_signals.md` +
`feedback_retro_task_complexity_score.md`. SL-e is the lane-closer
— the retro should explicitly flag the lane-wide carry-forward for
the v1-SL-lane-meta-retro that follows.

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-SL-e-retro.md
modifies: []
```

**IMPLEMENT (file 1 of 1):** in
`.claude/PRPs/reports/v1-SL-e-retro.md`, write the retro per the
canonical 4-role format (Advisor / Planning / Impl / BM):

- §1 Summary: SL-e shipped. Stories 1 + 2 + 3 ✓. **Closes the v1
  sponsor-liability lane** (modulo the restoration-during-window-escapes
  branch deferred to restorative-mechanics-v1 PRD per SL-c DQ
  #145). SL-lane-meta-retro follows.
- §2 Per-role signals (4 H2 sections; "what worked" + "what
  surprised us" + "what should change next"). Specifically:
  - **Advisor:** cross-PR carry-forward signals from SL-d → SL-e
    (did SL-d's retro accurately predict SL-e's needs? did Probe
    9-11 catch any SL-c/SL-d ship gaps? did the canonical sibling
    discipline scale to lane-wide tests?).
  - **Planning:** did the canonical Case A discipline (post-SL-c-2
    amendment) prevent another 3-cycle catch-fire on SL-e Tasks
    1-3? did the per-task anchor-Edit discipline scale on
    e2e.rs's 13,693-line baseline?
  - **Impl:** per-test wall-clock + log-silence metrics. Did
    Tasks 1-3 run cleanly without cycles? (SL-d retro target was
    ~5-8 min per test; SL-e expected similar.)
  - **BM:** any CR findings unique to lane-wide tests (e.g.
    "missing federation-outbox defensive assertion shape" or
    "pseudonym discipline coverage gaps")?
- §3 Carry-forward — list of items the lane-meta-retro should
  surface:
  - Lane-wide pseudonym-discipline coverage matrix.
  - Deferred-write semantics test pattern (negative+positive
    assertion pair) — promote to PMD.
  - `BREHON_DISABLE_GRACE_CHECK_JOB` envelope + force-rewind
    `grace_expires_at` patterns — promote to PMD.
  - Open: restoration-during-window-escapes test plan (lives in
    restorative-mechanics-v1 PRD).
- §4 Per-task complexity-score table (mandatory per
  `feedback_retro_task_complexity_score.md`):
  `| task | files-changed | commits | runtime-min | max-log-silence-min |`
  for each of Tasks 0, 1, 2, 3, 4.
- §5 Lessons promotion — any new `feedback_*.md` candidates.
  Likely:
  - `feedback_deferred_write_semantics_test_pattern.md` (paired
    negative+positive assertion at producer vs consumer).
  - `feedback_force_rewind_grace_expires_at_test_technique.md`
    (canonical alternative to `tokio::time::sleep` for time-
    dependent scheduler tests).
- §6 Acceptance — confirm all checkboxes from §17.

**MIRROR:** `.claude/PRPs/reports/v1-SL-d-retro.md` (sibling, when
authored); `.claude/PRPs/reports/v1-SL-c-2-retro.md`;
`.claude/PRPs/reports/v1-SL-b-retro.md`;
`.claude/PRPs/reports/v1-JM-e-retro.md` for section structure.

**GOTCHA (retro before PR):** retro is written BEFORE `gh pr
create` per `feedback_retro_not_report.md`.

**GOTCHA:** §4 per-task complexity-score table is mandatory.

**GOTCHA (lane-meta-retro distinct from this retro):** SL-e retro
is SL-e-scoped (the lane-closer phase). The v1-SL-lane-meta-retro
(per AD-lane precedent at
`.claude/PRPs/reports/v1-AD-lane-meta-retro.md`) is a separate
artifact written AFTER SL-e merges. §3 carry-forward names items
the lane-meta-retro should surface, but does NOT itself author the
lane-meta retro.

**Push and exit (Shape G — retro is meta-work; no `crates/**`
change → `cargo-validate-workspace` does not trigger).**

**COMMIT MESSAGE:** `docs(v1-SL-e): phase retrospective (task 4)`

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features
  full` via `cargo-validate-workspace.yml:88`.
- **Lint:** `cargo clippy --workspace --features full --no-deps
  -- -D warnings` via `cargo-validate-workspace.yml:92` (R6).
- **Test target compile (R7):** `cargo test --no-run -p
  lemmy_server --test e2e` via `cargo-validate-workspace.yml:95`.
- **Migration round-trip:** N/A — SL-e ships zero migrations.
- **e2e execution (Phase 2):** the 3 new e2e tests run via Phase 2
  e2e user gate: (a) local on laptop or (b) GH dispatch via
  `cargo-test-e2e.yml`. **This is the SL-e acceptance gate.**

### 14.1 Pre-existing tests preserved

All SL-d / SL-c-2 / SL-b / SL-a / JM-e / JM-c / JM-b / JM-a tests
preserved verbatim. The new `mod v1_sl_e_fixtures` block is
appended AFTER `mod v1_sl_d_fixtures` — no edits to existing tests.

### 14.2 Edge cases covered

- **Revocation-during-window-escapes:** SL-d producer transitions
  case to Pending; SL-b HTTP revoke severs the chain; SL-c
  scheduler skips the now-Escaped case (Test #1).
- **Window-expiry-fires:** SL-d producer transitions case to
  Pending; `grace_expires_at` force-rewound to past;
  `run_grace_check_batch` fires; reputation events + logs emitted
  (Test #2).
- **Backfill-of-mid-flight (v0→v1 deploy):** v0-shape case seeded
  directly; SL-a §8.4 backfill UPDATE invoked programmatically;
  case transitions to Pending; rewound; `run_grace_check_batch`
  fires (Test #3).

### 14.3 Edge cases NOT covered (out of v1-SL-e scope)

- **Restoration-during-window-escapes** — out per SL-c DQ #145
  LOCKED. Restorative-mechanics-v1 PRD's deliverable.
- **Admin-overridden status flip (`admin_close_case` on Pending
  case)** — out per §4.3. Admin-dashboard-v1 territory.
- **Concurrent SL-d + SL-b + SL-c race on same case** — defended
  by FOR UPDATE locks; single-threaded test harness can't exercise.
- **Multi-sponsor `all_revocation` / `majority_revocation` rules at
  vote-tally time** — out per PRD §13 OQ-V1-SL-01.
- **Federation outbound on Pending transition** — out per ADR-014 +
  PRD §11.5 (defensive zero-count assertion shapes only).
- **Modlog reader semantics during Pending state** — covered by PRD
  §11.4 documentation; no client-side test.

---

## 15. Validation commands (DoD)

> **Shape G plan — DoD is per-workflow, not inline cargo.** Per
> `.claude/PRPs/templates/plan.template.md` §15.6. SL-e ships zero
> migrations, so `cargo-validate-migration.yml` does not fire.

### 15.1 Per-task workspace check (Shape G)

For every §13 impl task that modifies `crates/**` (Tasks 1, 2, 3):

- **DoD entry:** `cargo-validate-workspace.yml` on
  `junior/<task-slug>` SHA `<sha>` → `conclusion: "success"`
- **Validation command:** `gh run list --repo barrie-cork/lemmy
  --branch <branch> --workflow cargo-validate-workspace --limit 1
  --json conclusion,databaseId --jq '.[0]'`
- **EXPECT:** `{"conclusion": "success", "databaseId": <id>}`

The workflow runs `cargo check --workspace --features full`,
`cargo clippy --workspace --features full --no-deps -- -D
warnings`, and `cargo test --no-run -p lemmy_server --test e2e`
per `.github/workflows/cargo-validate-workspace.yml:88-95`. R6 +
R7 are encoded.

### 15.2 Migration round-trip — N/A

SL-e ships zero migrations; `cargo-validate-migration.yml` path
filter `migrations/**` excludes SL-e commits. No DoD entry.

### 15.3 Phase 2 e2e (post-finalize-merge of Task 3)

After Task 3's worker branch finalize-merges into `phase-v1-SL-e`,
the advisor surfaces the **Phase 2 e2e local-vs-dispatch user
gate** per `advisor-orchestrator.md` (PR #105):

- **(a) local:** `cmd //c "scripts\\brehon\\cargo-test.bat
  --workspace --test e2e --features full > <log> 2>&1 && echo
  E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` on
  laptop in `run_in_background`; ~26 min wall-clock; zero billed.
  Per `feedback_windows_e2e_requires_bat_wrapper.md`.
- **(b) dispatch:** `gh workflow run cargo-test-e2e.yml --repo
  barrie-cork/lemmy --ref phase-v1-SL-e`; ci-watcher polls; ~26
  min billed.

Plan-side DoD: e2e exit code 0 with all 3 SL-e e2e tests passing
+ all pre-existing tests still passing; failure path → §G4
classifier on log slice.

### 15.4 Cross-cutting verification (Task 4 retro time)

- [ ] `rg -c '^pub const ENTRY_KIND_'
  crates/db_schema/src/source/governance/governance_log.rs`
  returns the same total as post-SL-d (unchanged — SL-e adds zero
  consts).
- [ ] `rg '^\s+ENTRY_KIND_'
  crates/api/api/src/governance/governance_log.rs` line count
  unchanged (no new shim re-exports).
- [ ] `git diff governance-v0..phase-v1-SL-e --stat` shows ONE
  source-file change: `crates/server/tests/e2e.rs | +<lines>`,
  plus ONE new report file:
  `.claude/PRPs/reports/v1-SL-e-retro.md`.
- [ ] `rg -n 'mod v1_sl_e_fixtures' crates/server/tests/e2e.rs`
  returns 1 line.
- [ ] `rg -n '^  async fn revocation_during_window_escapes_full_lane|^  async fn window_expiry_fires_full_lane|^  async fn backfill_of_mid_flight_v0_to_v1_deploy' crates/server/tests/e2e.rs`
  returns 3 lines (one per Test #1/#2/#3).
- [ ] R1: every `i32 ↔ i64` comparison in Tasks 1-3 uses
  `i64::from(...)`.
- [ ] R6: clippy invocations in `cargo-validate-workspace.yml`
  use `--no-deps -- -D warnings`.
- [ ] No edits to files outside §11 list.
- [ ] Every SL-e §16a story (1, 2, 3) is `[done]`.
- [ ] No new ENTRY_KIND_*** consts under
  `crates/db_schema/src/source/governance/governance_log.rs`.
- [ ] No new migrations: `git diff
  governance-v0..phase-v1-SL-e -- migrations/` returns empty.
- [ ] No production code modifications: `git diff
  governance-v0..phase-v1-SL-e -- 'crates/api/**'
  'crates/db_schema/**' 'crates/db_schema_file/**'
  'crates/db_views*/' 'crates/routes/**' 'crates/lemmy_server/**'`
  returns empty.
- [ ] No workflow YAML edits: `git diff
  governance-v0..phase-v1-SL-e -- .github/workflows/` returns
  empty.
- [ ] No PRD / ADR / plan edits: `git diff
  governance-v0..phase-v1-SL-e -- '.claude/PRPs/prds/**'
  '.claude/PRPs/plans/**' 'docs/brehon-law-inspired-network/**'`
  returns empty.

### 15.5 ADR / OQ compliance verification

- [ ] **ADR-005** honoured — per-sponsor `reputation_event` rows
  preserved by SL-c fire path (Test #2 asserts row count).
- [ ] **ADR-008** honoured — every governance_log emission uses
  `governance_log::append`; Tests #1 + #2 + #3 assert via row count
  + payload reads (no direct INSERT).
- [ ] **ADR-010** honoured — won't-disadvantage rule preserved.
  Test #2 asserts severity-snapshot (Low → 24h grace, not re-derived
  from config at fire-time). Test #3 asserts 24h-most-lenient
  backfill default.
- [ ] **ADR-013** honoured — Tests #1 + #2 + #3 assert via direct
  status equality (e.g. `case.status == CaseStatus::SponsorLiabilityPending`);
  no `match _ =>` wildcard test logic.
- [ ] **ADR-014** honoured — defensive zero-count assertions on
  federation_outbox rows for Pending-state cases (Tests #1 + #2
  shape; optional if the existing test infrastructure exposes the
  outbox query — if not, omit defensively).
- [ ] **ADR-015** honoured — every governance_log payload Tests
  assert on has pseudonym discipline (positive `is_string()` +
  negative `assert_ne!` against `format!("{}", raw_id.0)`). Per
  test: Test #1 covers `sponsor_liability_pending`,
  `endorsement_revoked`, `sponsor_liability_escaped`; Test #2
  covers `sponsor_liability_pending`, `sponsor_liability_applied`,
  `sponsor_liability_fired`; Test #3 covers
  `sponsor_liability_applied`, `sponsor_liability_fired`.
- [ ] **PRD §3.1 + §3.2** honoured — Tests #1 + #2 + #3 exercise
  the 3 v1 CaseStatus variants (Pending → Escaped on Test #1;
  Pending → Fired on Test #2 + #3).
- [ ] **PRD §4.1** honoured — Test #1 + #2 + #3 each pick a
  different severity tier; the chosen severity drives the
  `grace_window_for_severity` default (Test #1 168h Severe; Test
  #2 24h Minor; Test #3 backfill hardcodes 24h independent of
  severity).
- [ ] **PRD §5.3** honoured — Test #1 invokes `revoke_endorsement`
  via real HTTP-call (not direct DB UPDATE); asserts the
  `liability_chain_severed_for_cases` response field.
- [ ] **PRD §6.2** honoured — Tests #1 + #2 + #3 drive
  `run_grace_check_batch`; assert on
  `GraceCheckBatchOutcome.{cases_processed,fired,escaped,skipped}`.
- [ ] **PRD §8.4** honoured — Test #3 invokes the backfill UPDATE
  verbatim from PRD §8.4 SQL (with comment citing PRD §8.4).
- [ ] **PRD §9.3** honoured — Test #1 + #2 exercise the SL-d
  Pending transition; assert on `case.status`, `grace_expires_at`,
  `sponsor_liability_pending` log emission.
- [ ] **PRD §11.2** honoured — Test #3 exercises the §11.2
  mid-flight backfill semantics.
- [ ] **PRD §11.3** honoured — Tests #1 + #2 invoke
  `submit_jury_vote` via real handler-call; DTO + response shape
  unchanged.
- [ ] **PRD §11.4** honoured — Tests #1 + #2 assert deferred-write
  semantics: 0 `public_case_log` rows mid-window (both); 0 sponsor
  `reputation_event` rows mid-window (both); 0 juror
  `reputation_event` rows mid-window (both); positive count after
  fire (Test #2 only; Test #1 stays at 0 after escape).
- [ ] **PRD §15 row 5** honoured — 3 of the 4 listed scope branches
  shipped (revocation + expiry + backfill). Restoration deferred
  per SL-c DQ #145 LOCKED.
- [ ] **DQ #145** honoured — no restoration test in SL-e (Test
  list = exactly 3, none names restoration).
- [ ] **DQ #186** honoured — SL-d shipped successfully; SL-e
  inherits clean lane state.

### 15.6 DoD per workflow (canonical Shape G shape)

**Phase 1 (workspace check, per task):**

- Workflow: `.github/workflows/cargo-validate-workspace.yml`
- Branch (per task): `junior/<task-slug>`
- Expected `conclusion`: `"success"`

**Phase 1b (migration round-trip):** N/A — no migrations in SL-e.

**Phase 2 (e2e):**

- Workflow: `.github/workflows/cargo-test-e2e.yml` (only on user
  dispatch per PR #105)
- OR local: per advisor-orchestrator §5.2
  validate-pending-laptop-e2e — Windows bat wrapper with the
  `--workspace --test e2e --features full` invocation
- Branch: `phase-v1-SL-e`
- Expected: all tests pass (including the 3 new SL-e tests + all
  pre-existing v1-SL-*, v1-JM-*, v1-AD-* tests)

### 15.7 Manual validation snippets (advisor-side smoke only — non-binding)

> Per `feedback_features_full_p_crate_incompatible.md`: never `-p
> <crate>` + `--features full`; use `--workspace --features full`.

> Per `feedback_e2e_filter_assumes_naming.md`-style discipline:
> SL-e e2e tests all live in `mod v1_sl_e_fixtures` — confirm via
> grep before any filtered run.

> Per `cargo-output-capture.md`: capture cargo output to a file;
> never pipe through tail/head/grep.

```bash
ls .github/workflows/cargo-validate-workspace.yml
ls .github/workflows/cargo-test-e2e.yml

gh run list --repo barrie-cork/lemmy \
  --branch phase-v1-SL-e \
  --workflow cargo-validate-workspace \
  --limit 1 --json conclusion,databaseId

# Optional advisor-side local smoke (advisor-laptop only; non-binding):
cargo check --workspace --features full > /tmp/sl-e-check.log 2>&1
status=$?
tail -20 /tmp/sl-e-check.log
echo "exit: $status"

rg '#\[tokio::test\]\s*async fn .*' crates/server/tests/e2e.rs > /tmp/sl-e-tests.log
wc -l /tmp/sl-e-tests.log
# EXPECT: pre-merge baseline + 3 (SL-e Tests #1-#3)

rg -n '^mod v1_' crates/server/tests/e2e.rs
# EXPECT: 5 lines: mod v1_jm_b_fixtures, mod v1_jm_e_fixtures, mod v1_sl_b_fixtures, mod v1_sl_c_fixtures, mod v1_sl_d_fixtures, mod v1_sl_e_fixtures (6 total post-SL-e)
```

These are advisor-side only; not §16 acceptance criteria.

---

## 16. Acceptance criteria

- [ ] All 5 tasks (Task 0..3 + Task 4 retro) committed.
- [ ] §15.1 (`cargo-validate-workspace.yml`) `conclusion:
  "success"` after every impl task push (Tasks 1-3).
- [ ] §15.2 (migration round-trip) — N/A (zero migrations).
- [ ] §15.3 (Phase 2 e2e — local or dispatch) all tests pass; the
  3 new `v1_sl_e_fixtures::*` tests green; the pre-existing
  v1-SL-* + v1-JM-* + v1-AD-* tests all still green.
- [ ] §15.4 (cross-cutting verification — 14 boxes) all ticked.
- [ ] §15.5 (ADR / OQ compliance — 16 boxes) all ticked.
- [ ] §16a Stories 1 + 2 + 3 — all `[done]`.
- [ ] No edits to files outside §11 list (production code,
  migrations, workflows, PRDs, ADRs, plans all untouched).
- [ ] Retro committed per Task 4.
- [ ] PR opens against `governance-v0` (NOT `main`) with
  `--repo barrie-cork/lemmy`.
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-SL-e-verify.md` shows Stories 1-3 ✓.
- [ ] DQ #190 (split-or-proceed) self-resolved by planner with
  proceed rationale at plan-write time (Recipe 2 self-resolved
  per `decision-queue.md`).

---

## 16a. Stories (independently-testable behaviour units)

Per `.claude/PRPs/templates/plan.template.md` §16a +
`feedback_story_grain_checkpoint.md` +
`feedback_brehon_verify_pre_merge.md`. **SL-e ships THREE stories**,
one per test branch.

### Story 1: Revocation-during-window-escapes — full lane (SL-d producer → SL-b HTTP revoke → SL-c consumer skip)

- **Composing tasks:** Task 1.
- **Checkpoint workflow (Phase 1):**
  `cargo-validate-workspace.yml` on Task 1's worker branch SHA →
  `conclusion: "success"`.
- **Checkpoint workflow (Phase 2):** Phase 2 e2e on phase-branch
  tip; `revocation_during_window_escapes_full_lane` test passes.
- **Expected output (local Phase 2):** `1 passed; 0 failed` for
  this test.
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains `mod v1_sl_e_fixtures`
    block.
  - `crates/server/tests/e2e.rs` contains
    `async fn revocation_during_window_escapes_full_lane(`.
  - Test body asserts `case.status ==
    CaseStatus::SponsorLiabilityEscaped` post-revoke.
  - Test body asserts
    `response.liability_chain_severed_for_cases.contains(&case_id)`.
  - Test body asserts `count(sponsor_liability_pending) == 1` AND
    `count(endorsement_revoked) == 1` AND
    `count(sponsor_liability_escaped) == 1` post-revoke.
  - Test body asserts `count(sponsor_liability_fired) == 0` AND
    `count(public_case_log) == 0` AND sponsor reputation_event
    count `== 0` post-revoke (escape branch never publishes,
    never fires).
  - Test body asserts `outcome.cases_processed == 0` after
    scheduler tick (SL-c skips non-Pending case).
  - Test body asserts ADR-015 pseudonym discipline on
    `sponsor_liability_pending`, `endorsement_revoked`, and
    `sponsor_liability_escaped` payloads (positive `is_string()`
    + negative `assert_ne!` against raw id).

### Story 2: Window-expiry-fires — full lane (SL-d producer → SL-c consumer fire via wrapper)

- **Composing tasks:** Task 2.
- **Checkpoint workflow (Phase 1):**
  `cargo-validate-workspace.yml` on Task 2's worker branch SHA →
  `conclusion: "success"`.
- **Checkpoint workflow (Phase 2):** Phase 2 e2e on phase-branch
  tip; `window_expiry_fires_full_lane` test passes.
- **Expected output (local Phase 2):** `1 passed; 0 failed` for
  this test.
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains
    `async fn window_expiry_fires_full_lane(`.
  - Test body asserts `case.status ==
    CaseStatus::SponsorLiabilityPending` post-producer drive.
  - Test body asserts `case.grace_expires_at ≈ now +
    Duration::hours(24)` ± 5s (Low → Minor → 24h default).
  - Test body force-rewinds `grace_expires_at` via UPDATE.
  - Test body asserts `outcome.fired == 1` after scheduler tick.
  - Test body asserts `case.status ==
    CaseStatus::SponsorLiabilityFired` post-scheduler.
  - Test body asserts 2 sponsor `reputation_event` rows (1 per
    sponsor).
  - Test body asserts `count(sponsor_liability_applied) == 2` AND
    `count(sponsor_liability_fired) == 1` AND
    `count(public_case_log) == 1` post-scheduler (deferred writes
    fire at scheduler tick per PRD §11.4).
  - Test body asserts ADR-015 pseudonym discipline on
    `sponsor_liability_applied` (per row) AND
    `sponsor_liability_fired` payloads.

### Story 3: Backfill-of-mid-flight (v0→v1 deploy) — SL-a backfill UPDATE → SL-c consumer fire

- **Composing tasks:** Task 3.
- **Checkpoint workflow (Phase 1):**
  `cargo-validate-workspace.yml` on Task 3's worker branch SHA →
  `conclusion: "success"`.
- **Checkpoint workflow (Phase 2):** Phase 2 e2e on phase-branch
  tip; `backfill_of_mid_flight_v0_to_v1_deploy` test passes.
- **Expected output (local Phase 2):** `1 passed; 0 failed` for
  this test.
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains
    `async fn backfill_of_mid_flight_v0_to_v1_deploy(`.
  - Test body invokes `diesel::sql_query(...)` with PRD §8.4
    SQL verbatim (no paraphrase).
  - Test body asserts pre-backfill seed satisfies the §8.4 WHERE
    clause (positive enumeration of conditions).
  - Test body asserts `case.status ==
    CaseStatus::SponsorLiabilityPending` AND
    `case.grace_expires_at == decided_at + Duration::hours(24)`
    (within 1s) post-backfill UPDATE.
  - Test body force-rewinds `grace_expires_at` to past.
  - Test body asserts `outcome.fired == 1` after scheduler tick.
  - Test body asserts `case.status ==
    CaseStatus::SponsorLiabilityFired` post-scheduler.
  - Test body asserts 1 sponsor `reputation_event` row AND
    `count(sponsor_liability_applied) == 1` AND
    `count(sponsor_liability_fired) == 1`.
  - Test body asserts ADR-015 pseudonym discipline on
    `sponsor_liability_applied` payload.
  - `mod v1_sl_e_fixtures` closes after Test #3 body (the mod's
    closing `}` follows Test #3's test fn closing `}`).

> **Verification mapping:** `/brehon-verify` iterates this section,
> runs each Story's checkpoint workflow on the worktree branch,
> and confirms each Brief-Scope output exists + matches its
> structural pattern.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0..18 confirmed; Probes 7
  (SL-b shipped), 9 (SL-c module), 10 (SL-d split landed), 11
  (JM-c TODO discharged) are critical).
- [ ] Tasks 1..3 committed.
- [ ] Task 4 retro committed.
- [ ] §15 validation green at every gate (Phase 1 workspace check
  ×3, Phase 2 e2e ×1).
- [ ] §16a Stories 1, 2, 3 all `[done]`.
- [ ] PR opened by BM session against `governance-v0`.
- [ ] CodeRabbit review complete with findings triaged per
  `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-SL-e-verify.md` shows Stories 1-3 ✓.
- [ ] Post-merge phase branch retained for retro reads.
- [ ] DQ #190 (split-or-proceed) self-resolved by planner with
  proceed rationale at plan-write time.
- [ ] v1-SL-lane-meta-retro queued post-SL-e merge (per AD-lane
  precedent — separate task, not part of SL-e itself).

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Complexity score 10 prompts advisor to mandate further split | LOW | LOW | §5.2 — planner self-resolved DQ #190 with proceed rationale; further splitting yields no meaningful score reduction (e2e factor dominates +9 of +10 total). SL-c-2 (17), SL-d (16), JM-e (15), SL-b (38), SL-a (13) precedents proceed-as-one |
| Junior worker-hang on e2e.rs Edits (file is 13,693 lines, growing) | MED | HIGH | Per `feedback_junior_worker_e2e_edit_hang.md`: each test is its own §13 task; per-task anchor-Edit appends inside `mod v1_sl_e_fixtures`; no bulk Edit. SL-d shipped 4 tests as 4 tasks at this file size; SL-e ships 3 |
| 3-cycle catch-fire on stub-shape non-uniformity (per SL-c-2 cycles 1-3) | LOW | HIGH | §13 Tasks 1-3 stubs use `LemmyResult<()>` outer uniformly with v1-SL-b `mod v1_sl_b_fixtures` Case A canonical sibling at e2e.rs:11001-11924. `feedback_lemmy_error_no_std_error.md` Case A + `feedback_plan_stub_uniformity_with_canonical_sibling.md` cited in §9.4 + Tasks 1-3 GOTCHAs. §G4 row 4c (Case C — mixed shapes) is hard refusal; will not auto-fix |
| Backfill UPDATE shape diverges from PRD §8.4 (paraphrase) | LOW | HIGH | Task 3 IMPLEMENT specifies "verbatim from PRD §8.4 SQL" + comment cites PRD §8.4 explicitly. SQL block is large enough that paraphrase risk is real; mitigation is explicit verbatim discipline |
| Test #1 sponsor_view setup fails (caller-is-sponsor check rejects) | LOW | MED | Task 1 GOTCHA documents `LocalUserView::read_person(&mut context.pool(), sponsor1).await?` pattern from SL-d Test #1 at e2e.rs:12968. SL-b's check at `revoke_endorsement.rs` requires `from_person_id == caller_id` (sponsor self-revoke); seed shape uses sponsor1 as the endorsement creator, so caller_id == sponsor1 satisfies |
| Force-rewind `grace_expires_at` races with background scheduler clokwerk tick | LOW | MED | `BREHON_DISABLE_GRACE_CHECK_JOB=1` envelope per Watchpoint #3 prevents background scheduler. Test envelope set BEFORE producer drive; restored AFTER scheduler-tick assert. SL-c-2 precedent at e2e.rs:12046-12146 |
| Backfill rewind in Test #3 lands at the wrong second-boundary (post-backfill grace_expires_at = decided_at + 24h is in future, rewind to past needed) | LOW | LOW | Test #3 IMPLEMENT's Drive #2 explicitly rewinds `grace_expires_at` to `Utc::now() - Duration::minutes(1)` AFTER the backfill UPDATE; the §8.2 timing note documents this explicitly |
| Test #1 + #2 trip the SL-c per-case `FOR UPDATE` race against the test's own subsequent UPDATE | LOW | LOW | Single-threaded test harness; `run_grace_check_batch` completes synchronously before the test reads any state. No real concurrency. SL-c per-case tx releases on completion |
| `sponsor_liability_pending` payload's `sponsors_pseudonyms` field count mismatches (e.g. extra sponsor seeded but only 2 surety rows) | LOW | LOW | Task 1 setup seeds 2 active sureties; SL-d compute filters `revoked_at IS NULL`; the `Vec<SponsorDelta>` has length 2; payload's `sponsors_pseudonyms` array length 2. Assertion explicit |
| `case.severity` set as `CaseSeverity::High` mistakenly mapped to non-severe bucket | LOW | LOW | Task 1 IMPLEMENT names `CaseSeverity::High` explicitly; SL-d's mapping helper maps `Critical|High → severe → 168h`. Test asserts `grace_expires_at ≈ now + Duration::hours(168)` within 5s |
| `mod v1_sl_e_fixtures` closing `}` placement drifts (Task 3 doesn't close cleanly) | LOW | MED | Task 3 GOTCHA: the closing `}` follows immediately after Test #3's `Ok(())`. Mirror `mod v1_sl_d_fixtures` ending at e2e.rs:13693. Phase 1 workspace check catches unmatched `{` via cargo check |
| ts-rs derive regression on response DTO (revoke_endorsement) | LOW | LOW | SL-e does NOT modify any DTO (PRD §11.3); existing ts-rs derives unchanged |
| Cross-PR carry-forward miss between SL-d retro and SL-e planning | LOW | LOW | Task 0 Probe 18 reads SL-d retro at `.claude/PRPs/reports/v1-SL-d-retro.md` (advisory — if missing, surfaces but doesn't block). SL-e Task 4 retro records cross-PR carry-forward findings |
| Restoration-during-window-escapes branch tempted into SL-e scope | LOW | HIGH | §4.3 explicit rejection rationale citing SL-c DQ #145 LOCKED; brief §4.2 boundary-of-judgment cited. Plan §12 enumerates as out-of-scope. The SL-c stub-only branch's behaviour duplicates Test #2 (Fire fallback) — testing it would just re-test the expiry path |
| Federation outbound assertion fails (existing federation infra not test-callable from e2e.rs) | LOW | LOW | §15.5 ADR-014 row marks the federation-outbox assertion as "optional if test infrastructure exposes the outbox query". If not callable, omit defensively + note in §19 |
| DQ commits on phase branch cause PR DIRTY conflict (SL-c-2 retro carry-forward) | LOW | MED | Per SL-c-2 retro §3 lesson 1 + §5 watch-item 2 — advisor checks `mergeStateStatus` after bm-pr; if DIRTY, rebase/cherry-pick before CR review. ci-watcher mutations should write `governance-v0` directly, not the phase branch |
| Phase-2 e2e on laptop fails due to Windows bat wrapper invocation pattern (per RCA 2026-05-09) | LOW | MED | §15.3 cites `feedback_windows_e2e_requires_bat_wrapper.md`; the bat wrapper command is explicit; advisor-orchestrator §5.2 documents the canonical invocation |
| PMD #126 — DQ ID collision (concurrent SL-e + rep-tuning-r* work) | LOW | LOW | next-id calc spans archives + live; advisor monitors at finalize-merge. SL-e plan-write computed next_id = 190 (DQ #189 was the most recent entry on `governance-v0`, the SL-d Phase 2 e2e pass) |

---

## 19. Notes

### 19.1 Planner DQs filed (SL-e)

- **DQ #190** (`from: "planner"`, `kind: "blocker"`, `answered_by:
  "planner"` self-resolved per Recipe 2) — SL-e complexity-score
  split decision per §5.2. Question: "v1-sponsor-liability-e
  complexity score 10 exceeds 8 — split into `v1-SL-e-1`
  (revocation, Task 1) + `v1-SL-e-2` (expiry + backfill, Tasks
  2-3), or proceed?". Options: split / proceed. Planner
  self-resolved with **proceed** rationale: e2e factor dominates
  (+9 of +10 total); SL-e-1 and SL-e-2 would individually be
  below threshold (4 and 7 respectively) but fragmenting the
  lane-closer suite across two sub-phases doubles
  cohort/finalize/retro overhead; SL-b at 38 + SL-c-2 at 17 + SL-d
  at 16 + JM-e at 15 + SL-a at 13 all shipped proceed-as-one
  without operational regret. Plus the 3 tests share `mod
  v1_sl_e_fixtures` helpers (~40% duplication savings vs splitting
  helpers across two mods). Per `feedback_principles_not_rules.md`,
  the e2e suite is anchor-Edit-friendly + the canonical Case A
  sibling discipline (post-SL-c-2 amendment) is well-established.

### 19.2 Self-resolved planner findings (LESSON candidates — SL-e)

- **The deferred-write semantics test pattern (paired
  negative+positive assertion at producer vs consumer) is now
  exercised lane-wide.** SL-d Test #1 introduced the "negative
  assertion at vote-tally time, deferred to scheduler" half; SL-e
  Test #2 completes the pair by asserting the positive count after
  scheduler tick. SL-e Test #1 extends to the escape path (escape
  branch preserves zero positive counts — the deferred writes never
  fire). **Promote to PMD lesson candidate (post-SL-e ship)**:
  "deferred-write semantics testing — pair a negative assertion at
  the producer with a positive assertion at the consumer to validate
  the deferral round-trip; for the escape branch, the negative
  assertion holds even post-resolution".

- **The force-rewind `grace_expires_at` test technique is the
  canonical alternative to `tokio::time::sleep` for time-dependent
  scheduler tests.** SL-c-2 demonstrated the technique via direct-
  seeded Pending cases (`grace_offset: Duration::minutes(-1)`);
  SL-e Test #2 + #3 extend it to producer-driven Pending cases via
  post-producer UPDATE. **Promote to PMD lesson candidate
  (post-SL-e ship)**: "force-rewind grace_expires_at via UPDATE —
  drive the producer normally, then UPDATE the case's
  grace_expires_at to a past time, then invoke the scheduler. Test
  runs in milliseconds; semantically equivalent to wall-clock
  advance. Used in Brehon's v1-SL-c-2 / v1-SL-e tests for the
  sponsor-liability grace window".

- **Lane-wide e2e tests (the SL-e shape) are observationally
  distinct from per-sub-phase tests.** SL-b/c/d's tests each
  exercise one sub-phase's code in isolation (with direct-DB-seed
  upstream). SL-e exercises the chain end-to-end via real handler
  invocations. The pattern generalises: any "lane-closer" sub-phase
  whose deliverable is integration coverage should:
  1. Author a new `mod v1_<lane>_e_fixtures` (or equivalent named
     mod) at file end.
  2. Compose helpers from prior fixture mods (don't refactor — the
     v1-SL-* discipline duplicates).
  3. Drive each lane branch via real HTTP-call handler invocations
     + scheduler invocations + raw-SQL backfill invocations as
     appropriate.
  4. Assert end-to-end state transitions across multiple handlers
     in one test body.
  **Promote to PMD lesson candidate (post-SL-e ship)**: "lane-closer
  e2e suite shape — one fixture mod, one test per scope branch,
  each test drives 2-3 handlers in sequence with assertions
  between each handler".

- **Cross-PR carry-forward via SL-d retro → SL-e Task 0 (Probe 7
  SL-b + Probe 9 SL-c + Probe 10 SL-d + Probe 11 JM-c) is the
  load-bearing hand-off discipline for lane-closer sub-phases.**
  SL-e's Task 0 explicitly verifies SL-b + SL-c + SL-d shipped
  state PLUS the JM-c TODO marker discharge. Without these probes,
  SL-e risks being branched off a stale tip. Per
  `feedback_handover_trailer_cohort_propagation.md` (cross-cohort
  handover), the cross-PR analogue is the multi-probe ship-
  confirmation check. **Promote to PMD lesson candidate**:
  "cross-PR lane-closer hand-off requires N-Probe ship-confirmation
  per upstream sub-phase, including TODO-marker-discharged checks".

### 19.3 Restoration-escape stub-and-future-wire breadcrumb

(Inherited from SL-c plan §19.3 + SL-d plan §19.3 — SL-e
likewise does NOT add restoration tests; restorative-mechanics-v1
PRD will add the producer endpoint + consumer test. SL-e is the
last sub-phase to inherit this breadcrumb before
restorative-mechanics-v1 PRD work begins.)

### 19.4 Pre-existing pending DQ entries

- **None at planning time.** `decision-queue.json` shows 0 pending
  entries on `governance-v0`. Recently resolved: DQ #189 (SL-d
  Phase 2 e2e pass on tip `78771349e`, 2026-05-11). DQ #145 (SL-c
  restoration deferral, 2026-05-07).
- **DQ #190 filed by this plan** (split-or-proceed;
  planner-resolved with proceed rationale).

### 19.5 Out-of-scope follow-ups (Task 4 retro candidates)

- **Restoration-during-window-escapes test** — restorative-
  mechanics-v1 PRD.
- **Admin-overridden status flip on Pending case (`admin_close_case`)**
  — admin-dashboard-v1 sub-phase retro flag if coverage gap
  detected.
- **Multi-sponsor `all_revocation` / `majority_revocation` rules
  at vote-tally time test** — out per PRD §13 OQ-V1-SL-01 (v1+
  separate design decision).
- **Federation outbound assertions on Pending transition** — defer
  to inbound/outbound federation PRD work.
- **Concurrent-handler race tests (SL-d + SL-b + SL-c collision)**
  — single-threaded test harness limitation; promote to integration
  test workshop if needed in v2.
- **Sponsor notification UX test** — PRD §13 OQ-V1-SL-03 (v3
  polish).
- **Step-up auth for admin-driven revocation** — v2.
- **v1-SL-lane-meta-retro** — separate task, AFTER SL-e merges,
  per AD-lane precedent at
  `.claude/PRPs/reports/v1-AD-lane-meta-retro.md`.

### 19.6 Confidence bands (SL-e)

- **High (9/10):** Test #1 shape — exact composition of SL-b
  HTTP-call pattern (e2e.rs:11154-11199) + SL-d producer-drive
  pattern (e2e.rs:12891-13132) + SL-c scheduler invocation
  (e2e.rs:12046-12148). Every helper has a canonical source.
- **High (9/10):** Test #2 shape — combines SL-d producer drive +
  force-rewind grace_expires_at (canonical mirror of SL-c-2 Test #1
  seed pattern) + SL-c scheduler invocation. Three well-understood
  primitives.
- **High (8/10):** Test #3 shape — programmatic backfill UPDATE
  via raw SQL `diesel::sql_query`. PRD §8.4 verbatim shape; only
  novel element is the rewind-as-test-artifact for scheduler tick.
- **High (9/10):** Fixture mod placement + Case A discipline —
  exact mirror of SL-d's `mod v1_sl_d_fixtures` ending at
  e2e.rs:13693. Per-task anchor-Edit pattern well-rehearsed across
  SL-b + SL-c-2 + SL-d + JM-e.
- **High (9/10):** Pseudonym discipline assertions — every payload
  Tests assert on has a defensive positive+negative pair (ADR-015
  defense-in-depth). Mirror pattern from SL-d Test #1 + SL-c-2
  Test #1.
- **Moderate (7/10):** §5 complexity score 10 trips threshold;
  planner self-resolved DQ #190 with proceed rationale citing
  e2e-factor analysis + SL-c-2/SL-b/SL-d/JM-e proceed-as-one
  precedents. SL-e at 10 sits at the low end of the proceed-as-one
  envelope (just over threshold).
- **High (9/10):** stub-shape uniformity
  (`feedback_lemmy_error_no_std_error.md` Case A +
  `feedback_plan_stub_uniformity_with_canonical_sibling.md`)
  encoded in §13 Tasks 1-3 GOTCHA blocks; cite canonical sibling
  line range explicitly. Second plan to ship under the
  post-SL-c-2 amendment (SL-d was the first; SL-d shipped
  cleanly without 3-cycle catch-fire).
- **High (9/10):** lane-wide proof shape — SL-e is the lane-closer;
  every prior sub-phase's tests live in isolation, SL-e is the
  integration test. PRD §15 row 5 was reserved for exactly this
  shape since the PRD was authored. No surprise scope.

### 19.7 Why no clarify DQ at impl time (SL-e)

Brief §4.2 enumerates the boundary-of-judgment cases that should
trigger an impl-side DQ. None apply to this SL-e plan as written:

- DQ #145 (restoration deferred) LOCKED 2026-05-07 — referenced in
  plan §6 + §12.
- Time-handling approach for Test #2 — watchpoint #2 specifies
  force-rewind-via-UPDATE (canonical; no DQ needed).
- Optional 4th test — §4.3 explicitly REJECTED (admin-dashboard
  territory); §12 enumerates as out-of-scope.
- Backfill UPDATE SQL — Task 3 IMPLEMENT specifies verbatim PRD
  §8.4 shape; no SL-a migration shape change since SL-a merged.
- Sponsor-view setup for Test #1 — Task 1 GOTCHA documents the
  `LocalUserView::read_person` pattern; SL-b's canonical
  invocation pattern at e2e.rs:11157-11199 is the mirror.

If any baseline assumption changes between plan-write and impl-time
(specifically: SL-d module signatures drift, SL-b handler signature
drifts, e2e.rs line count shifts dramatically, PRD §8.4 SQL is
revised), the impl-task subagent files a DQ pending entry per
`feedback_principles_not_rules.md` + `decision-queue.md` Recipe 1.

### 19.8 Forward-only retrofit scope

Per `feedback_schema_changing_spec_retrofit_question.md` — SL-e
does NOT change the shape of any existing artifact class (no new
template section, no new schema marker, no new YAML field). All
§13 task contracts are routine test-author contracts. No retrofit
question applies.

### 19.9 Cross-PR carry-forward discipline (SL-d → SL-e)

SL-e's Task 0 Probes 7 + 9 + 10 + 11 are the cross-PR
module-presence + signature-stability + TODO-discharge checks
across the SL-b / SL-c / SL-d / JM-c upstream chain. SL-e's Task 4
retro §2 (per-role signals) records cross-PR carry-forward findings:
did the SL-d retro accurately predict SL-e's needs? did Probe 9
catch any SL-c gaps? did Probe 10 catch any SL-d signature drift?
did the SL-d → SL-e retro chain produce a clean hand-off?

This is the **THIRD** in-fork instance of cross-PR carry-forward
applied to a structurally-split sub-phase chain (after SL-c-1 →
SL-c-2 → SL-d). The SL-e Task 4 retro AND the subsequent
v1-SL-lane-meta-retro should produce a structured lessons-promotion
review of the cross-PR carry-forward discipline overall.

### 19.10 Lane-closure framing

SL-e is the **last** v1-SL sub-phase before the lane meta-retro.
Per the brief's "Lean / advisor-side tip":

- After SL-e merges, the SL lane is fully shipped — schema (SL-a)
  + revocation handler (SL-b) + scheduler (SL-c) + producer
  mutation (SL-d) + lane-wide e2e (SL-e). The lane-shipping retro
  (`.claude/PRPs/reports/v1-SL-lane-meta-retro.md` per AD-lane
  precedent) follows SL-e's merge.
- SL-e has the **lowest complexity of the SL lane** — no new
  modules, no new schema, no new entry kinds. The work is purely
  test authoring against an already-shipped production code base.
  Per-task wall-clock under Sonnet 4.6 should be ~5-8 min per test
  (Edit + cargo check on test target + workspace lint).
- The three test branches are **highly orthogonal**. They share
  fixture setup (jury creation, surety creation, sanction creation)
  but diverge on the action triggered (vote+revoke / vote+expire /
  backfill+expire).
- SL-e's strongest assertion is the **deferred-write semantics
  from PRD §11.4**. Test #1 + #2 verify the timing of
  `public_case_log` emission. Without this assertion, a regression
  in PRD §11.4 (e.g. a mid-flight code change that re-emits modlog
  at vote-tally time) would silently break the auditor-visibility
  contract.
- SL-e is **safely run-after-everything**. Unlike earlier phases
  that ship code that downstream phases consume, SL-e consumes
  everything and produces nothing. If SL-e finds bugs in SL-a/b/c/d's
  behaviour, the fix is a chore-PR on the affected lane (not a
  re-plan of SL-e).

---

## 20. Confidence score

- **Plan correctness:** 9/10 — patterns mirror SL-b HTTP-call
  pattern (Test #1) + SL-d producer-drive pattern (Test #1 + #2) +
  SL-c-2 scheduler-tick pattern (Test #1 + #2 + #3) + PRD §8.4
  backfill SQL (Test #3). §13 task bodies anchor to literal line
  numbers + verbatim brief scope.
- **Cargo budget:** 10/10 — Shape G (cargo runs off-box; budget
  non-binding; forbidden-window non-binding for impl-task).
- **Test coverage:** 9/10 — 3 e2e tests covering the three PRD §15
  row 5 scope branches (modulo restoration deferred per SL-c DQ
  #145). Restoration-escape test ships in restorative-mechanics-v1
  PRD. Federation defensive assertions optional per §15.5.
- **Story-grain decomposition:** 9/10 — SL-e ships 3 stories
  cleanly mapping to the 3 PRD §15 row 5 scope branches
  (revocation + expiry + backfill).
- **Plan-shape conformance:** 9/10 — 20-section schema followed
  literally; §15.6 Shape G shape adopted; §16a mandatory present;
  §13 per-task FILES YAML block on every task; §5.1 complexity
  breakdown table mechanical; canonical-Case-A sibling referenced
  explicitly in §13 Tasks 1-3 GOTCHA blocks.

---

_Plan author: planning subagent (Junior worktree
`/srv/brehon-fork/.junior/worktrees/job-228` on
`junior/role-planning-v1-sponsor-liability-e-plan-...-228`,
2026-05-12 — brief authored 2026-05-07 by sibling advisor session;
no clarify-DQ on the brief — no clarify entries surfaced in
.claude/decision-queue.json). Plan committed on the worktree branch;
Junior daemon's finalize step pushes to `governance-v0`. BM-task
cuts `phase-v1-SL-e` from `governance-v0` after plan ships and user
approves. SL-e score 10 above threshold; planner DQ #190
self-resolved with proceed rationale (e2e factor dominates +9 of
+10 total; further splitting yields no meaningful per-phase
reduction; SL-c-2 / SL-b / SL-d / JM-e / SL-a proceed-as-one
precedents). Confidence 9/10. Plan ships under proceed-within-SL-e
assumption (DQ #190). SL-e is the **lane-closer** for v1-SL; SL-
lane-meta-retro follows SL-e's merge._
