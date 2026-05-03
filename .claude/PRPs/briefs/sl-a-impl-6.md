---
role: impl-task
plan_task: 6
phase: v1-SL-a
cohort: A
created: 2026-05-03
related_dq: [115]
---

# Brief — v1-SL-a Task 6 [P] — UPDATE `crates/api/api/src/governance/config.rs` — 13 new consts + match arms + SEEDED_KEYS + EXPECTED_SEED_COUNT_V1_SL + ConfigKeyMetadata + parity test

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a task 6 — see .claude/PRPs/briefs/sl-a-impl-6.md`

You are the **impl-task** subagent (Sonnet 4.6) executing plan Task 6
of cohort A on `phase-v1-SL-a`. Cohort A = Tasks 1+2+3+6+7 (5-way
parallel); you own only `crates/api/api/src/governance/config.rs`.

## 2. Scope

**Produce:**

1. ONE commit modifying `crates/api/api/src/governance/config.rs`
   with **6 sub-edits** per plan §10.6 verbatim:
   - 13 new `pub const DEFAULT_*` declarations (after the v1-JM-a
     block at line 902).
   - 13 new match arms across `const_default_int` (10) +
     `const_default_float` (1) + `const_default_bool` (1) +
     `const_default_text` (1).
   - 13 new tuples appended to `SEEDED_KEYS_WITH_CONSTS` (after the
     v1-JM-a block, alphabetised within the new block by key).
   - New `pub const EXPECTED_SEED_COUNT_V1_SL: usize = 13;` after
     `EXPECTED_SEED_COUNT_V1_JM` at line 1322.
   - Extend `seeded_keys_count_matches_const_count` parity test at
     lines 2422–2436 to include `EXPECTED_SEED_COUNT_V1_SL`.
   - 13 new `CONFIG_KEY_METADATA` entries before the closing `];` of
     the array at line 2414.
2. **Pre-commit reconciliation gate** (mandatory per plan Task 6
   "Pre-commit reconciliation gate"): four shell commands MUST all
   return `13` (or empty for the diff). If any fail, choose the
   correct outcome (a/b/c/d) per plan; outcome (d) → file DQ pending
   rather than guess.
3. ONE `kind: "validate-pending"` DQ entry referencing
   `cargo-validate-workspace.yml`.
4. Push your worker branch. Junior daemon finalize-merges into
   `phase-v1-SL-a`.

**Do NOT** in this task:

- Touch any other file (no `migrations/**`, no `enums.rs`, no
  `schema.rs`, no other `crates/**` file, no `tests/**`, no docs).
  `crates/api/api/src/governance/config.rs` is the only editable
  file.
- Bump `EXPECTED_SEED_COUNT` (the v0 invariant) or
  `EXPECTED_SEED_COUNT_V1_AD` / `EXPECTED_SEED_COUNT_V1_JM`. Per
  the parametric pattern (advisor directive 2026-04-19 #4), each
  v1 sub-PRD adds its OWN constant beside the existing ones.
- Modify the existing v0 `liability.*` keys at lines 1090–1092
  (`founder_multiplier`, `regular_multiplier`,
  `sponsor_liability_floor`). Both v0 and v1-SL-a `liability.*`
  keys coexist; SL-a's are namespaced `liability.grace_window_*` /
  `restoration_*` / `multi_sponsor_*` / `revoke_*`.
- Modify the existing v0 `job.*` keys at lines 105–106. SL-a adds
  three new `job.grace_check_*` keys to the same flat namespace.
- Use `f64` for any of the 6 `liability.grace_window_*_hours`
  keys. They are `i64` (raw hours) per DQ #115. Only
  `job.grace_check_staleness_alert_multiplier` is `f64`.
- Use `-p lemmy_api --features full` for any local diagnostic
  cargo invocation (incompatible — see lesson §3 Required reading 5).
  Local diagnostic if any: `cargo check --workspace --features full`.
- Run cargo locally for validation. Shape G — workspace-check fires
  on push.

## 3. Required reading

Read in this order before writing any code:

1. **Plan Task 6 body:** `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md`
   lines 1731–1834 — task header, FILES YAML, IMPLEMENT directives,
   Pre-commit reconciliation gate, GOTCHAs (5 of them).
2. **Plan §10.6 (canonical config.rs skeleton):**
   `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` lines 826–995.
   The verbatim source for ALL 6 sub-edits (DEFAULT_* declarations,
   match arms, SEEDED_KEYS, EXPECTED_SEED_COUNT_V1_SL,
   parity-test extension, CONFIG_KEY_METADATA values).
3. **Read the file you'll edit FIRST:** `crates/api/api/src/governance/config.rs`.
   Confirm:
   - Line 902 (or near) is the end of the v1-JM-a `DEFAULT_*` block.
   - Line 1090–1092 contain the v0 `DEFAULT_LIABILITY_*` consts
     (founder/regular/floor) — these MUST stay untouched.
   - Line 1175–1298 contain the v1-JM-a `SEEDED_KEYS_WITH_CONSTS`
     block.
   - Line 1299 (or near) is the closing `];` of `SEEDED_KEYS_WITH_CONSTS`.
   - Line 1322 has `EXPECTED_SEED_COUNT_V1_JM`.
   - Line 2414 (or near) is the closing `];` of `CONFIG_KEY_METADATA`.
   - Lines 2422–2436 contain `seeded_keys_count_matches_const_count`.
   If any of those line numbers diverge in the live file, **adapt to
   the live file** (no reformatting, surgical insertion at the
   identified positions) and document the diverged line numbers in
   the commit body.
4. **DQ #115 (resolved, advisor 2026-05-03):** confirms the 6
   `liability.grace_window_*_hours` keys are raw integer hours, NOT
   micros-scaled. Read `.claude/decision-queue.json` resolved
   entries for context.
5. **`.claude/lessons/feedback_features_full_p_crate_incompatible.md`**
   — `-p <crate> + --features full` errors. Use `--workspace
   --features full` for any local diagnostic. (Reminder: local
   cargo is non-binding under Shape G.)
6. **`.claude/lessons/feedback_advisor_cr_enum_drift.md`** — for the
   `liability.multi_sponsor_escape_rule` enum (`any_revocation |
   all_revocation | majority_revocation`): cross-check plan §10.6
   AGAINST PRD §10. Plan-named enum strings must appear in PRD; if
   not, file a DQ.
7. **PRD §10 defaults-matrix and §18 B4 key-rename table** as cited
   throughout plan §10.6 — the source of truth for ranges, scopes,
   `apply_at_default`, and namespace shape.

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 0
    commit: 1f6131bf823f006bd2ace06b2931e2c1b56cd8ad
    filesCreated: []
    filesModified:
      - scripts/brehon/migrate-roundtrip.sh
    keyDecisions:
      - replaced stub per DQ #114 (Option A: Task 0 of SL-a)
      - "lemmy_diesel_utils binary surface — no CLI sub-commands; uses
        LEMMY_DATABASE_URL env var only (binary at
        crates/diesel_utils/src/main.rs reads from env, takes no positional
        args). Adapted plan §13 GOTCHA cargo invocations accordingly."
    notes: |
      Probe 2 EXPECTED_SEED_COUNT lines confirmed present in
      crates/api/api/src/governance/config.rs. Probe baseline established.
```

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branch derived from `phase-v1-SL-a`
  (tip `1b47a3ff4`).
- `git branch --show-current` must return your `junior/<task-slug>`
  branch. Confirm `git merge-base --is-ancestor phase-v1-SL-a HEAD`
  exits 0 before any write.
- ONE source-code commit. Daemon finalize absorbs into `phase-v1-SL-a`.

### File discipline

- Files you may MODIFY: `crates/api/api/src/governance/config.rs` ONLY.
- **Files you may NOT touch:** any other file in the repo. If you
  find yourself wanting to touch one, STOP and file a DQ.

### Edit discipline

- **Use Edit (not Write).** config.rs is large (>2400 lines). Six
  surgical edits at the identified positions.
- **Verbatim from plan §10.6** for every sub-edit. Do not paraphrase
  doc-comments, change const names, change types, drop `pub`, or
  alphabetise differently.
- **Type discipline (per DQ #115):** all 6
  `DEFAULT_LIABILITY_GRACE_WINDOW_*_HOURS` are `i64`. The bool key
  (`DEFAULT_LIABILITY_RESTORATION_ESCAPES_LIABILITY`) is `bool`.
  The text key (`DEFAULT_LIABILITY_MULTI_SPONSOR_ESCAPE_RULE`) is
  `&str`. Only the float key
  (`DEFAULT_JOB_GRACE_CHECK_STALENESS_ALERT_MULTIPLIER`) is `f64`.
- **SEEDED_KEYS_WITH_CONSTS alphabetisation:** within the new block
  ONLY. Do NOT re-alphabetise the v0 / AD / JM blocks. Plan §10.6
  shows the new block already alphabetised — copy that order verbatim.
- **CONFIG_KEY_METADATA entries:** mirror v1-JM-a entries at lines
  2200-2412 for shape. Per-entry fields per plan §10.6:
  `apply_at_default`, `scope`, `requires_re_jury: false`,
  `requires_step_up: false`, `valid_range` (per PRD §10),
  `valid_enum` (only on `multi_sponsor_escape_rule`), `description`
  (one-sentence operator-facing), `doc_anchor:
  "v1-sponsor-liability.prd.md§10"`.
- **Parity test extension:** plan §10.6 shows the exact replacement
  for `seeded_keys_count_matches_const_count` (lines 933-953).
  Replace verbatim — including the format-string error message
  formatting.

### Pre-commit reconciliation gate (MANDATORY — do not skip)

After all 6 sub-edits and BEFORE `git commit`, run plan Task 6's
reconciliation gate (lines 1765-1785 of the plan):

```bash
# Count new SEEDED_KEYS_WITH_CONSTS v1-SL-a tuples
awk '/v1-SL-a additions/,/^];$/' crates/api/api/src/governance/config.rs | \
  grep -cE '^\s*\("'
# Expected: 13

# Count new DEFAULT_* declarations in the v1-SL-a additions block
awk '/-- v1-SL-a additions/,/^pub\(crate\) fn const_default_int/' crates/api/api/src/governance/config.rs | \
  grep -cE '^pub const DEFAULT_'
# Expected: 13

# Count INSERT rows in Task 1's up.sql (the seed migration)
grep -cE "^\s*\('instance', 'liability\.|^\s*\('instance', 'job\.grace_check_" \
  migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql
# Expected: 13

# Cross-check: every seeded key in up.sql ↔ down.sql
diff <(grep -oE "'(liability|job)\\.[a-z_.]+'" migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql | sort -u) \
     <(grep -oE "'(liability|job)\\.[a-z_.]+'" migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql | sort -u)
# Expected: empty
```

**Cohort A coordination caveat:** Tasks 1 + 2 + 3 + 6 + 7 are
running in parallel. Task 1's migration directory may not exist on
*your* worker branch yet (your branch is derived from `phase-v1-SL-a`
tip `1b47a3ff4` BEFORE Task 1's finalize-merge). Therefore the third
and fourth reconciliation commands ABOVE may fail with `No such file
or directory` on your branch even when Task 1 succeeds in parallel.
**Treat this as expected:** run the first TWO commands (config.rs-
internal counts) on your branch. Skip the migration cross-checks if
the file doesn't exist; document in commit body. The advisor will
re-run the full gate post-cohort-A finalize before queuing cohort B.

If the FIRST TWO commands disagree (e.g. SEEDED_KEYS=13 but
DEFAULT_*=12), DO NOT commit. Apply outcome (a)/(b) per plan
Task 6. If outcome unclear, file a DQ pending entry.

### Shape G discipline (no local cargo)

- Push your worker branch to `junior/<task-slug>`. The push to
  `crates/**` triggers `cargo-validate-workspace.yml`. Migration
  workflow does NOT trigger.
- Capture the workflow run id:
  ```bash
  gh run list --repo barrie-cork/lemmy --branch <branch> --workflow cargo-validate-workspace --limit 1 --json databaseId
  ```
- Write ONE `kind: "validate-pending"` DQ entry: distinct `id`,
  `from: "impl"`, `kind: "validate-pending"`, `workflow_run_id: <id>`,
  `branch: "<your branch>"`, `phase_task: 6`, `result: null`,
  `log_slice: null`, `failed_jobs: null`, `answer: null`,
  `answered_by: null`, `resolved_at: null`. `context` says
  "workspace-check for v1-SL-a task 6".
- Commit + push the DQ entry to your worker branch immediately per
  "Mid-task visibility". Two commits: (1) `feat(v1-SL-a): config.rs
  ...`, (2) `chore(decision-queue): impl raised DQ #N — sl-a-task-6
  validate-pending`.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- Compute next id: scan **both** live `pending`/`resolved` AND any
  `decision-queue-archive-*.json`. **Cohort A coordination:** Tasks
  1 + 2 + 3 + 6 + 7 are parallel; re-read live `decision-queue.json`
  immediately before computing next id.

### Commit shape

- ONE source-code commit:
  - Subject: `feat(v1-SL-a): config.rs — 13 new liability/job consts + metadata + SEEDED_KEYS + EXPECTED_SEED_COUNT_V1_SL parity (task 6)` (verbatim per plan §13 line 1834).
  - Body: include the `HANDOVER:` YAML trailer:
    ```yaml
    HANDOVER:
    filesCreated: []
    filesModified:
      - crates/api/api/src/governance/config.rs
    keyDecisions:
      - "added 13 DEFAULT_* consts (10 i64 + 1 bool + 1 &str + 1 f64) per plan §10.6"
      - "added 13 SEEDED_KEYS_WITH_CONSTS tuples (alphabetised within v1-SL-a block)"
      - "added EXPECTED_SEED_COUNT_V1_SL = 13 + extended parity test"
      - "added 13 CONFIG_KEY_METADATA entries with PRD-grounded ranges/scopes"
      - <reconciliation-gate result: 13/13/n-a/n-a or full 13/13/13/empty>
      - <line-number divergences from plan §10.6 with reason if any>
    notes: <reconciliation-gate cohort-A caveat: migration cross-checks skipped because Task 1 not yet finalize-merged onto phase-v1-SL-a from this branch's perspective>
    ```
- THEN the DQ-entry commit (separate).
- Files in source-code commit: ONLY config.rs.

## 5. Validation gate

Plan §15 names workspace-check (run in GH Actions on push):

- `cargo check --workspace --features full`.
- `cargo clippy --workspace --features full --no-deps -- -D warnings`.
- `cargo test --no-run -p lemmy_server --test e2e` (test-target
  compile per `feedback_test_target_compile_validation.md`).

**Cohort A barrier semantics:** the workspace-check on YOUR branch
will likely FAIL because:

1. Cohort A Tasks 4 and 5 haven't landed (cohort B). The
   non-exhaustive-match clippy at the 6 sites Task 5 covers, and
   the missing-field errors at Task 4's `ModerationCase` struct,
   will trip workspace-check.
2. Even within cohort A, Task 7's ENTRY_KIND_* names referenced by
   Task 6's parity-test or downstream code don't yet exist on YOUR
   branch (Task 7 is parallel; not yet merged onto your branch).

**This is the planner-intended barrier shape.** Plan §14 Story 1
checkpoint is workspace-check `conclusion: "success"` on **Task 5's
push** (after cohort B), not Task 6's push. Advisor §G4 classifier
recognises non-exhaustive-match / missing-field failures during
cohort A as planner-intentional and HOLDS the cohort barrier without
auto-queuing fix-impl-tasks.

If workspace-check fails on a Task-6-specific error (typo, missing
match arm, count mismatch in `EXPECTED_SEED_COUNT_V1_SL` arithmetic,
unhandled enum value in a non-Task-5 site), advisor catch-fires.

## 6. Expected output (return to advisor)

```
## Task 6 complete — v1-SL-a config.rs extended

**Branch:** junior/role-impl-task-...
**Commits:**
  - <sha-a> feat(v1-SL-a): config.rs ... (task 6)
  - <sha-b> chore(decision-queue): impl raised DQ #<N> — sl-a-task-6 validate-pending
**File:** crates/api/api/src/governance/config.rs
**6 sub-edits:**
  - 13 DEFAULT_* consts (10 i64 + 1 bool + 1 &str + 1 f64) inserted after v1-JM-a block
  - 13 match arms (10 const_default_int + 1 float + 1 bool + 1 text)
  - 13 SEEDED_KEYS_WITH_CONSTS tuples (alphabetised within v1-SL-a block)
  - EXPECTED_SEED_COUNT_V1_SL = 13 inserted after EXPECTED_SEED_COUNT_V1_JM
  - parity-test extended (seeded_keys_count_matches_const_count)
  - 13 CONFIG_KEY_METADATA entries (per-key apply_at, scope, range, doc_anchor)
**Pre-commit reconciliation gate:**
  - SEEDED_KEYS_WITH_CONSTS v1-SL-a tuples: 13 (PASS)
  - DEFAULT_* declarations in v1-SL-a block: 13 (PASS)
  - up.sql INSERT rows: <13 OR n/a — Task 1 not finalize-merged onto this branch>
  - up.sql ↔ down.sql diff: <empty OR n/a — same reason>
**Workflow run captured:**
  - workspace-check workflow_run_id: <int> → DQ #<N>
**Expected workspace-check outcome:** non-exhaustive-match + missing-field failures (Task 4/5 in cohort B; Task 7 parallel) — COHORT BARRIER, not Task-6 defect
**Next:** advisor dispatches 1 ci-watcher; cohort A advances when all 5 task validate-pending entries resolve
```

If any pre-write check failed (file path missing, plan §10.6 line
range diverges past adaptation, reconciliation gate first-two-checks
disagree), replace with a DQ catch-fire entry committed + pushed to
your worker branch immediately.
