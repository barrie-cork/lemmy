---
role: impl-task
plan_task: 8
phase: v1-SL-a
cohort: B-tail
created: 2026-05-03
related_dq: []
preallocated_dq: [131]
---

# Brief — v1-SL-a Task 8 — UPDATE `crates/server/tests/e2e.rs` — extend `phase1_migrations_round_trip`

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a task 8 — see .claude/PRPs/briefs/sl-a-impl-8.md`

You are the **impl-task** subagent (Sonnet 4.6) executing plan Task 8
(cohort B tail, NOT `[P]`) on `phase-v1-SL-a`. This is the e2e
migration round-trip extension — bumps `PHASE_1_MIGRATION_COUNT` and
extends post-condition probe lists in `phase1_migrations_round_trip`
to assert SL-a's effects per plan §10.8.

## 2. Scope

**Produce:**

1. ONE source-code commit modifying ONE file:
   - `crates/server/tests/e2e.rs` — bump `PHASE_1_MIGRATION_COUNT` +2
     (NOT +1 as plan §13 line 1923 says; see §7 Override) + extend
     probe lists in `phase1_migrations_round_trip` per plan §10.8.
2. ONE `kind: "validate-pending"` DQ entry referencing
   `cargo-validate-workspace.yml` (the workspace-check workflow runs
   `cargo test --no-run -p lemmy_server --test e2e` step which
   compiles this test). **Pre-allocated DQ id: #131.**
3. Push your worker branch. Junior daemon finalize-merges (or advisor
   manually finalizes per the known finalize-merge bug).

**Do NOT** in this task:

- Add any NEW test fns. Per plan §13 GOTCHA: "single Edit-with-anchor
  block, no new test fns". `phase1_migrations_round_trip` is the only
  function touched.
- Touch any other file. ONLY `crates/server/tests/e2e.rs`.
- Run `cargo test` locally. Shape G — workspace-check fires on push.
  The workspace-check runs `cargo test --no-run` (compile only). Full
  e2e execution is the Phase 2 user-gate after finalize-merge,
  advisor-driven.
- Touch `crates/server/tests/e2e.rs` outside the
  `phase1_migrations_round_trip` function body and its
  `PHASE_1_MIGRATION_COUNT` const declaration.

**Commit message** (verbatim per plan §13 line 1973): `test(v1-SL-a): extend phase1_migrations_round_trip — bump count + new schema effects (task 8)`

## 3. Required reading

Read in this order before writing any code:

1. **Plan Task 8 body:** `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md`
   lines 1914–1976 — task header, FILES YAML (1 file), 2 IMPLEMENT
   sub-edits (anchor 1 + anchor 2), 3 GOTCHAs.
2. **Plan §10.8** (the canonical spec for what to assert): same plan
   file lines 1067–1100. The 4 post-up.sql probes + 4 post-down.sql
   probes are the exact list to add.
3. **MIRROR — v1-JM-a Task 10 precedent:** read your phase-tip
   `crates/server/tests/e2e.rs` lines **463–601** (the existing
   `phase1_migrations_round_trip` function). Specifically:
   - Line 463: the `PHASE_1_MIGRATION_COUNT = 12` declaration with
     its 25-line doc-comment that lists per-sub-phase contributions.
   - Lines 484–509: post-forward probe loop (table existence).
   - Lines 522–548: post-revert probe loop (tables gone).
   - Lines 553–580: pg_type drop probe (enum types removed after
     revert).
   - You will EXTEND those probe loops (or add adjacent probe blocks
     for SL-a's specific column/index/enum-value/config-row checks
     per §10.8). Do NOT replace the JM-a probes.
4. **`.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`** —
   `crates/server/tests/e2e.rs` is currently 10,230 lines on phase
   tip; full-file Edits hang the worker. **Use Edit-with-anchor**:
   each Edit's `old_string` must be a unique 5-30 line slice anchored
   on a stable landmark (the function header, the `const` declaration,
   a `for table in [` line, etc). Do NOT pass the entire file as
   context.
5. **`.claude/lessons/feedback_clippy_test_style.md`** — workspace
   clippy denies `unwrap()`, `expect()`, `_ => arms` on enum matches,
   `#[allow]` escape-hatches. Use `?` propagation with
   `Result<_, Box<dyn Error>>` (the function already returns this).
6. **`.claude/lessons/feedback_pipes_mask_exit_codes.md`** — never
   pipe cargo through tail/head/grep when checking exit. Capture
   full output, then check `$?`, then tail.

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1-fix
    commit: a7f824047b21af41d349061d19c54dd785725b41
    filesCreated:
      - migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/up.sql
      - migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/down.sql
      - migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/up.sql
      - migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/down.sql
    keyDecisions:
      - "split combined SL-a migration into enum-add + non-enum directories per Phase 5b Restoration precedent"
      - "migration count delta: SL-a contributes +2 migrations (was +1); Task 8 must reflect bump of +2"
    notes: "PRD §8.5 + plan §10.1 superseded; verified pass via DQ #126; planner-miss for SL-a retro"
  - task: 2
    commit: bfc8eaa416498d6efd45e64d35e3b80a5466a39e
    filesModified:
      - crates/db_schema_file/src/enums.rs
    keyDecisions:
      - "added 3 CaseStatus variants (SponsorLiabilityPending/Fired/Escaped) per plan §10.2 verbatim"
      - "post-edit count = 12 variants (was 9); #[default] Open unchanged"
    notes: "your pg_enum probe should assert these 3 NEW values appear after up.sql AND remain after down.sql (Postgres limitation)"
  - task: 3
    commit: c7a977078e1a2b84ec2a9c91ba4f86c8ac4a7b1c
    filesModified:
      - crates/db_schema_file/src/schema.rs
    keyDecisions:
      - "added 2 columns (grace_expires_at, liability_escape_reason) to moderation_case"
    notes: "your column-name probe asserts these 2 columns exist after up.sql, absent after down.sql"
  - task: 4
    commit: d9f7d1ac6
    filesModified:
      - crates/db_schema/src/source/governance/moderation_case.rs
    keyDecisions:
      - "added 2 fields (grace_expires_at, liability_escape_reason) to BOTH ModerationCase + ModerationCaseInsertForm per plan §10.4"
    notes: "Rust struct-side; not relevant to Task 8 (test queries via raw sql_query, not typed Diesel)"
  - task: 5
    commit: b2e31c97e
    filesModified:
      - crates/api/api_crud/src/governance/request_appeal.rs
      - crates/api/api/src/governance/admin_close_case.rs
      - crates/api/api/src/governance/admin_trigger_appeal_rejury.rs
      - crates/api/api/src/governance/accept_jury_assignment.rs
      - crates/api/api/src/governance/admin_assign_jury.rs
      - crates/api/api/src/governance/submit_jury_vote.rs
    keyDecisions:
      - "extended 6 match sites + 1 matches!() guard per plan §10.5"
      - "closed cohort A barrier; workspace-check passed GREEN on retry workflow 25281617538 at 14:48:13Z (DQ #130)"
    notes: "Task 5 + advisor §G4 manual fix at eff00b19d (clippy::map-err-ignore |_| → |_e|) closed the unifying green-gate; Task 8 builds atop both"
  - task: 5-r3-addendum
    commit: e5872d65e
    filesModified:
      - crates/api/api/src/governance/admin_dashboard.rs
      - crates/api/api/src/governance/get_case.rs
    keyDecisions:
      - "extended 2 helper match fns (admin_dashboard::is_active_status, admin_dashboard::status_key, get_case::is_public_status) — planner-miss carry-forward"
      - "is_active_status: SponsorLiabilityPending=true (grace window active); Fired/Escaped=false (terminal)"
      - "is_public_status: all 3 → true (post-Decided, public-visible same as Decided/Closed)"
    notes: "planner missed these in §10.5; surface in SL-a retro as feedback_advisor_watchpoint_specificity instance"
  - task: 6
    commit: <on-phase-tip-already>
    filesModified:
      - crates/api/api/src/governance/config.rs
    keyDecisions:
      - "added 13 new SL-a config keys (liability.grace_window_*_hours triplet + 10 others) + EXPECTED_SEED_COUNT_V1_SL constant + parity test extension"
    notes: "your governance_config row-count delta probe asserts +13 rows after up.sql vs pre-up.sql baseline"
  - task: 7
    commit: <on-phase-tip-already>
    filesModified:
      - crates/db_schema/src/source/governance/governance_log.rs
      - crates/api/api/src/governance/governance_log.rs
      - .claude/rules/governance-log-entry-kind-registry.md
    keyDecisions:
      - "5 new ENTRY_KIND consts (sponsor.liability.*); registry section populated; Acceptance count 33→38"
    notes: "Task 7 is governance_log entry kinds, not schema migrations — does NOT affect PHASE_1_MIGRATION_COUNT or your probe lists"
```

**Load-bearing context for Task 8:**

- All 7 prior cohort A+B tasks have landed on `phase-v1-SL-a` and the
  green-gate has closed (workspace-check `success` on workflow run
  25281617538 at 2026-05-03 14:48:13Z, DQ #130 mutated to `pass` on
  origin via commit `f45b0207b`).
- Phase tip is now `09aaa3ebf` (cascade-resolution of cohort-A/B
  fail entries; see commit body for context).
- The 2 SL-a migrations are at:
  `migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/`
  + `migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/`
- **Migration count delta is +2 (NOT +1 as plan §13 line 1923 says).**
  The plan was authored before Task 1 was split via fix-impl-1; the
  task body's "Bump by +1" wording is a planner-side staleness. The
  Task 5 brief's HANDOVER trailer surfaces this; this brief restates
  it in §7 Override.

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branch derived from `phase-v1-SL-a`
  (current tip `09aaa3ebf`). Use `git fetch origin && git log
  phase-v1-SL-a` to confirm — if the tip has moved further, adapt
  to the new tip; the Task 8 anchors do not depend on later tasks.
- `git branch --show-current` must return your `junior/<task-slug>`
  branch. Confirm `git merge-base --is-ancestor phase-v1-SL-a HEAD`
  exits 0 before any write.
- ONE source-code commit. Daemon finalize-merges into `phase-v1-SL-a`.

### File discipline

- File you may MODIFY: **only** `crates/server/tests/e2e.rs`.
- Within that file: **only** the `PHASE_1_MIGRATION_COUNT` const
  declaration (its value + doc-comment) and the
  `phase1_migrations_round_trip` function body. Do NOT touch any
  other test, helper, or module-level item.
- **Files you may NOT touch:** any `migrations/**`, any `crates/`
  file other than `crates/server/tests/e2e.rs`, any other test file,
  `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, docs, scripts.
- The plan's Task 5 R3 sweep pattern (`match \w+\.status`) does NOT
  apply to Task 8 — this is a test extension, not an enum-arm sweep.

### Edit-with-anchor discipline (CRITICAL — e2e.rs is 10,230 lines)

Per `feedback_junior_worker_e2e_edit_hang.md`: full-file Edits hang
the worker. Use 2 anchored Edits, each with a unique 5-30 line
`old_string` slice:

#### Anchor 1 — `PHASE_1_MIGRATION_COUNT` declaration

Anchor on the const declaration + its 1-line preceding comment block.
The exact slice on phase tip (`09aaa3ebf`) at line 463:

```
  const PHASE_1_MIGRATION_COUNT: u64 = 12;
```

Plus the doc-comment block above it (lines 427–462) listing per-sub-
phase contributions. **Add an SL-a section** at the appropriate
chronological position in the breakdown list (after the v1-JM-a
section), and bump the value to **14** (12 + 2 SL-a migrations).

The doc-comment additions should look like (verify the exact
surrounding text via `Read` first; do not paste blindly):

```
  /// Bumped to 14 in v1-SL-a (adds 2: add_case_status_sponsor_liability_variants
  /// @ 2026-05-03-000000, add_sponsor_liability_grace_window @
  /// 2026-05-03-000100). Phase-by-phase breakdown (bookkeeping, not enforced):
  ///   - 6 Phase 1 migrations (...)
  ///   - 2 Phase 5a migrations (...)
  ///   - 1 Phase 5b Slice A migration (...)
  ///   - 3 v1-JM-a migrations (...)
  ///   - 2 v1-SL-a migrations (this bump)
```

(Replace the existing "Bumped to 12 in v1-JM-a (adds 3..." line with
the new "Bumped to 14 in v1-SL-a (adds 2..." line; preserve the
JM-a bookkeeping entry in the breakdown list. The "Uncounted drift"
paragraph about v1-AD-a stays unchanged. Do NOT remove or rewrite
any existing prose other than the lead-line bump.)

#### Anchor 2 — post-condition probe extensions

Plan §10.8 prescribes 4 post-up.sql + 4 post-down.sql probes.
Implement these by **extending existing probe blocks** in
`phase1_migrations_round_trip` (do NOT add a new function).

The function body (lines 472–600) has 5 logical blocks:
- Step 1 (line 479): forward apply.
- Post-forward probes (lines 484–509): table existence loop.
- Step 2 (lines 515–518): revert.
- Post-revert probes (lines 522–548): table-gone loop + pg_type drop
  probe (lines 553–580).
- Step 3 (line 585): re-apply.
- Final probe (lines 590–598): governance_log queryable.

**Where to add SL-a probes:**

Add 2 new probe scopes (each in `{ ... }` braces matching the JM-a
style, with its own `let mut conn = PgConnection::establish(...)`):

**(A) Post-forward (after line 509, before Step 2 revert at line 515).**
4 probes per §10.8:

1. **`moderation_case` columns:** assert `grace_expires_at` and
   `liability_escape_reason` columns exist via
   `information_schema.columns` query (`SELECT count(*) AS n FROM
   information_schema.columns WHERE table_name = 'moderation_case' AND
   column_name = $1`). Each should return n=1.
2. **Index existence:** assert `moderation_case_grace_expires_idx` and
   `surety_sponsored_id_active` exist via `pg_indexes` query (`SELECT
   count(*) AS n FROM pg_indexes WHERE indexname = $1`). Each n=1.
3. **`pg_enum` values:** assert `SponsorLiabilityPending`,
   `SponsorLiabilityFired`, `SponsorLiabilityEscaped` appear for the
   `case_status` enum via `pg_enum` JOIN `pg_type` query. Each n=1.
4. **`governance_config` row count delta:** capture the row count
   BEFORE up.sql (you'll need to re-architect Step 1 minimally to
   measure pre-state) OR use the post-forward count and compare
   against a known baseline. **The plan says "+13 after up.sql,
   restored after down.sql"** — implement as: probe row count after
   up.sql, store as a `let post_up_count: i64`, then re-probe after
   down.sql and assert exact restoration.

   **Minimum-touch interpretation:** add a `governance_config` row-
   count read into a local variable in scope (A) (post-up.sql), and
   compare to the post-down.sql count in scope (B) — assert equality.
   The plan's "+13" assertion is the SL-a delta; assert it by reading
   the count after up.sql and asserting `>= 13` (lower-bound is safer
   than exact since other phases also seed config rows; or, if the
   trunk baseline is known to be 27 from v1-AD-a per JM-a precedent,
   assert `== 27 + 13 == 40`). **If the exact baseline is unclear,
   file a DQ pending entry with the actual observed count and let
   advisor decide between "exact == 40" or ">= 13 delta".**

**(B) Post-down.sql (after line 580, before Step 3 re-apply at line
585).** 4 probes per §10.8:

1. **Columns absent:** assert `grace_expires_at` and
   `liability_escape_reason` no longer exist (n=0).
2. **Indexes absent:** assert `moderation_case_grace_expires_idx` and
   `surety_sponsored_id_active` no longer exist (n=0).
3. **pg_enum residuals:** assert `SponsorLiabilityPending`,
   `SponsorLiabilityFired`, `SponsorLiabilityEscaped` **STILL EXIST**
   (n=1 each). Add a doc-comment explaining: "Postgres ALTER TYPE
   DROP VALUE is unsupported; values added in up.sql persist after
   down.sql per PRD §3.4 down.sql doc-comment + Phase 5b Restoration
   variant precedent. Asserting the residual EXISTS makes the
   limitation explicit."
4. **`governance_config` row count restored:** assert the count
   matches the pre-up.sql baseline (or whatever scope (A) captured).

### Memory-cap awareness

The daemon runs under `MemoryMax=10G`, `MemoryHigh=8G`. If
`cargo test --no-run` (the workspace-check step) hits the cap, the
cgroup OOM-killer terminates the worker — Junior reports non-zero
exit. **Do NOT retry blindly** — file a DQ pending entry with the
log tail; advisor will decide.

### Plan-cited line numbers may have drifted

The plan body cites lines 463 (PHASE_1_MIGRATION_COUNT) + ~423–601
(function body). Phase tip 09aaa3ebf preserves these locations
(verified by advisor before brief-write). If you observe drift,
follow the grep, not the plan numbers, and document in commit body.

### Lesson trailer (encouraged)

If during the task you discover something a future impl-task on the
e2e harness would have wanted to know — non-obvious test structure,
a probe pattern that worked, a Postgres footgun — end the commit-
message body with a `LESSON:` line per
`feedback_junior_pmd_write_convention.md`. One discrete lesson per
line. Cite specific files/lines.

### Shape G discipline (no local cargo)

- Push your worker branch. The push triggers
  `cargo-validate-workspace.yml`. **Migration workflow does NOT
  trigger** (no `migrations/**` change in this task — Task 8 is
  Rust-only).
- Capture the workspace-check workflow run id:
  ```bash
  gh run list --repo barrie-cork/lemmy --branch <branch> --workflow cargo-validate-workspace --limit 1 --json databaseId
  ```
- Write ONE `kind: "validate-pending"` DQ entry: **id = 131**
  (pre-allocated by advisor; do NOT compute, do NOT use any other
  id), `from: "impl"`, `kind: "validate-pending"`,
  `workflow_run_id: <captured id>`, `branch: "<your branch>"`,
  `phase_task: 8`, `result: null`, `log_slice: null`,
  `failed_jobs: null`, `answer: null`, `answered_by: null`,
  `resolved_at: null`. `context` says "workspace-check for v1-SL-a
  task 8 (e2e PHASE_1_MIGRATION_COUNT bump + post-condition probe
  extensions per plan §10.8)".
- Commit + push the DQ entry to your worker branch immediately per
  "Mid-task visibility". Two commits acceptable.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- Pre-allocated id #131. No read-modify-write race.

### Commit shape

- ONE source-code commit:
  - Subject: `test(v1-SL-a): extend phase1_migrations_round_trip — bump count + new schema effects (task 8)` (verbatim per plan §13 line 1973).
  - Body: include the `HANDOVER:` YAML trailer:
    ```yaml
    HANDOVER:
    filesCreated: []
    filesModified:
      - crates/server/tests/e2e.rs
    keyDecisions:
      - "PHASE_1_MIGRATION_COUNT bumped 12 → 14 (+2 SL-a migrations per fix-impl-1 split decision; plan §13 line 1923 said +1, which is stale)"
      - "added post-up.sql probe block (4 SL-a-specific assertions: 2 columns, 2 indexes, 3 enum values, governance_config delta) per plan §10.8"
      - "added post-down.sql probe block (4 assertions including the deliberate pg_enum residual assertion per Postgres limitation)"
      - "<governance_config baseline decision: '== 40 (27 trunk + 13 SL-a)' OR '>= 13 delta' per advisor DQ resolution if filed>"
    notes: "Task 8 closes the cohort B tail; only Task 9 retro remains. e2e harness now asserts SL-a's full schema surface end-to-end."
    ```
- THEN the DQ-entry commit (separate).
- Files in source-code commit: ONLY `crates/server/tests/e2e.rs`.
  Files in DQ commit: ONLY `.claude/decision-queue.json`.

## 5. Validation gate

Plan §15 + Task 8 §15 names this (run in GH Actions on push):

- `cargo test --no-run -p lemmy_server --test e2e --features full`
  (workspace-check workflow's e2e compile step)

The workspace-check workflow also runs `cargo check --workspace
--features full` and `cargo clippy --workspace --features full
--no-deps -- -D warnings` — Task 8 should NOT introduce check or
clippy regressions (it's a test extension only, no API surface
changes), but the workflow gates all 3.

**Expected outcome:** workspace-check `success`. Task 8 is a pure
test extension; no production-code reach. The post-condition probes
won't actually execute (only `--no-run` compile in workspace-check)
— full e2e execution happens after finalize-merge via the Phase 2
e2e user-gate (advisor surfaces local-vs-dispatch; user-default per
`feedback_default_local_testing` is local).

If workspace-check fails on:
- A typo in your probe (column name, index name, enum value name) →
  re-edit and amend.
- A Diesel `QueryableByName` / `sql_type` mismatch → most likely a
  type-binding issue with the new pg_indexes / information_schema
  probe; align the `Count` shape (already declared in scope at line
  466–470) or define a new local `#[derive(QueryableByName)]` shape.
- A `Box<dyn Error>` propagation issue with `?` on the new probes →
  the function already returns this; no special handling needed.
- A clippy lint on the doc-comment additions → reword.

## 6. Expected output (return to advisor)

```
## Task 8 complete — v1-SL-a e2e migration round-trip extension

**Branch:** junior/role-impl-task-...
**Commits:**
  - <sha-a> test(v1-SL-a): extend phase1_migrations_round_trip — bump count + new schema effects (task 8)
  - <sha-b> chore(decision-queue): impl raised DQ #131 — sl-a-task-8 validate-pending
**Files modified:** crates/server/tests/e2e.rs (PHASE_1_MIGRATION_COUNT 12→14 + 8 new probes in 2 scopes within phase1_migrations_round_trip)
**Workflow run captured:**
  - workspace-check workflow_run_id: <int> → DQ #131
**Expected workspace-check outcome:** GREEN (test extension; no production-code regression risk)
**Next:** advisor dispatches 1 ci-watcher; on pass, advisor surfaces Phase 2 e2e user-gate (local recommended per feedback_default_local_testing); only Task 9 (retro) remains
```

If any pre-write check failed (file path missing, plan §10.8 spec
diverges past adaptation, MIRROR file lines unreadable), replace with
a DQ catch-fire entry committed + pushed to your worker branch
immediately.

## 7. Why this brief differs from the plan

1. **Migration count bump value: +2, not +1.** Plan §13 Task 8
   "ACTION" subsection (line 1923) says "Bump `PHASE_1_MIGRATION_COUNT`
   constant by +1". This was authored before Task 1 was split via
   fix-impl-1 (commit `a7f824047`) into 2 migrations:
   - `2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/`
   - `2026-05-03-000100-0000_add_sponsor_liability_grace_window/`

   Task 5's HANDOVER trailer at commit `b2e31c97e` already surfaced
   this: "migration count delta: SL-a contributes +2 migrations (was
   +1); Task 8 will see this and reflect". The bump is therefore
   **12 → 14**, not 12 → 13.

   This deviation is a planner-staleness issue, not a design change.
   Surface in the SL-a retro as a planner-miss carry-forward.

2. **Doc-comment update format:** the plan's IMPLEMENT block (line
   1925–1929) says "updates the doc-comment to add the SL-a entry to
   the per-sub-phase breakdown". This brief specifies the exact
   format (replace lead-line "Bumped to 12 in v1-JM-a..." with
   "Bumped to 14 in v1-SL-a..."; add new bookkeeping bullet for
   "2 v1-SL-a migrations") to match the existing JM-a precedent at
   lines 439–451. This is execution detail, not deviation.

3. **`governance_config` baseline decision deferred:** the plan's
   §10.8 says "+13 after up.sql, restored after down.sql" but does
   NOT specify whether the assertion is delta-based (`>= 13`) or
   absolute (`== 40` if the trunk baseline is known). This brief
   defers the decision to the impl-task subagent: pick the safer
   delta-based assertion and document in commit body, OR file a DQ
   pending entry with the observed phase-tip baseline and let advisor
   decide. Either path is acceptable; document in `keyDecisions`.
