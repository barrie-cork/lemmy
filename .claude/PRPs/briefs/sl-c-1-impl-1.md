---
role: impl-task
plan_task: 1
phase: v1-SL-c-1
created: 2026-05-07
related_dq: null
---

# Brief — v1-SL-c-1 Task 1 — Create `sponsor_liability_grace.rs` module + 4 pub fns + module wiring

## 1. Role + dispatch line

`[role:impl-task] sl-c-1-impl-1 — see .claude/PRPs/briefs/sl-c-1-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 1
from `.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md` §13 — the
core c-1 deliverable. **Two file changes, one commit + one DQ
validate-pending commit.**

## 2. Scope

**Produce:**

1. **One impl commit** creating
   `crates/api/api/src/governance/sponsor_liability_grace.rs` AND
   modifying `crates/api/api/src/governance/mod.rs` (one
   `pub mod sponsor_liability_grace;` line inserted alphabetically
   after the existing `pub mod sponsor_liability;` at line 38).

   The new module exports:
   - `pub async fn run_grace_check_batch(context: &LemmyContext) -> LemmyResult<GraceCheckBatchOutcome>`
   - `pub async fn evaluate_escape_conditions(conn: &mut AsyncPgConnection, case_id: ModerationCaseId, target_person_id: PersonId, community_id: Option<CommunityId>, decided_at: DateTime<Utc>, cache: &mut ConfigCache) -> LemmyResult<EscapeStatus>`
   - `pub async fn fire_or_escape_case(conn: &mut AsyncPgConnection, case_row: ModerationCase, status: EscapeStatus, cache: &mut ConfigCache) -> LemmyResult<()>`
   - `pub async fn check_grace_staleness(conn: &mut AsyncPgConnection, max_grace_hours: i64, multiplier: f64, now: DateTime<Utc>) -> LemmyResult<()>`
   - `pub enum EscapeStatus { Escape { reason: String, actor_pseudonym: String, ref_id: i64 }, Fire }`
   - `pub struct GraceCheckBatchOutcome { cases_processed: usize, fired: usize, escaped: usize, skipped: usize }`
   - Private `enum PerCaseOutcome` + private `async fn fire_or_escape_case_inner` per trunk plan §10.2.

2. **Push the worker branch** to `origin/junior/<task-slug>`.

3. **Write a `kind: "validate-pending"` DQ entry** (second commit on
   the worker branch) capturing the workflow run id of
   `cargo-validate-workspace.yml` per Shape G Layer G2.

**Do NOT** in this task:

- Touch any other file in `crates/**` (the scheduler block in
  `scheduled_tasks.rs` is Task 2; e2e tests live in c-2).
- Run cargo locally — Shape G plan; cargo runs on GH-hosted runners
  after push.
- Modify `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.
- Add new ENTRY_KIND consts, CaseStatus variants, schema columns, or
  config seeds — ALL substrate shipped in SL-a (verified by Task 0
  Probes 2-6).
- Write to `.claude/decision-queue.json` outside the validate-pending
  entry + any blocker entries.

## 3. Required reading

Read in this order before writing the file:

1. **c-1 plan §13 Task 1**
   (`.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md` lines
   1218-1324) — task envelope + GOTCHAs.
2. **Trunk plan §13 Task 1 IMPLEMENT block**
   (`.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` line 2019
   onwards) — the c-1 plan delegates the IMPLEMENT body to the trunk.
   You write the same Rust whether you read c-1 or trunk; the c-1
   §13 Task 1 explicitly says "the trunk is the canonical source ...
   the code emitted is the same as if proceed-as-one had been
   chosen."
3. **Trunk plan §10.1/§10.2/§10.4/§10.6/§10.7** (canonical code
   blocks for outer batch runner, per-case transaction body,
   staleness check, governance_log payload schemas, sanction action
   lookup) at `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md`
   lines 1181-1652.
4. **c-1 plan §4.2** watchpoints (lines 457-547) — 14 entries; #1, 2,
   4, 5, 6, 7, 8, 13, 14 bind Task 1 directly.
5. **Source files to mirror (read each at least once before
   writing):**
   - `crates/api/api/src/governance/reputation_snapshot.rs:361-459`
     — outer batch runner pattern + two-tier ConfigCache (lines
     363/389) + staleness check (lines 423-459).
   - `crates/api/api/src/governance/appeal_window_expiry.rs:1-87`
     — per-row body pattern (closer fit for c-1's per-case shape).
   - `crates/api/api/src/governance/sponsor_liability.rs:142-358`
     — v0 `apply_sponsor_liability` signature + body c-1's fire
     branch invokes.
   - `crates/api/api/src/governance/governance_log.rs` —
     `governance_log::append` signature + the 3 ENTRY_KIND
     re-exports (lines 68/74/75).
   - `crates/api/api/src/governance/actor_pseudonym_helper.rs`
     — `get_or_create` signature for the escape-branch's
     `actor_pseudonym` derivation.
6. **PRD §8.1**
   (`.claude/PRPs/prds/v1-sponsor-liability.prd.md` §8.1) —
   `liability_escape_reason` JSONB schema; `version: 1` mandatory.
7. **`.claude/rules/decision-queue.md`** — schema-v2 for the
   `validate-pending` entry shape; `from: "impl"`,
   `kind: "validate-pending"`, `branch: <worker branch>`,
   `phase_task: "sl-c-1-impl-1"`, `workflow_run_id: <int>`,
   `commands: [...]`.
8. **`.claude/rules/pm-plugin-hooks-stable.md`** — non-binding for
   c-1 (no PM-handler edits) but read so the impl knows the hook
   names are not in scope to modify.
9. **Lessons** (Glob `.claude/lessons/`):
   - `feedback_features_full_workspace_only.md` — workspace-check
     uses `--workspace --features full`; ts-rs derives activate.
   - `feedback_features_full_p_crate_incompatible.md` — never
     `-p <crate> --features full`.
   - `feedback_clippy_test_style.md` — R1 `i64::from(...)` on `i32 ↔
     i64`; bound by c-1 watchpoint #13.
   - `feedback_multi_write_handlers_need_transactions.md` — c-1's
     per-case body uses `conn.run_transaction(|conn| async move {
     ... }.scope_boxed())`; outer batch does NOT.
   - `feedback_pr_per_phase.md` — one impl commit + one DQ-entry
     commit on the worker branch.
   - `feedback_explicit_file_arrays_on_tasks.md` — FILES YAML block
     from c-1 plan §13 Task 1 binds the modify-set; Task 1 changes
     EXACTLY two files (one created, one modified).
   - `feedback_clippy_rerun_after_fix.md` — workflow may surface
     `unused-imports` on intermediate iterations; the workflow
     re-fires on each push.

## 3a. Handover from prior cohort

**From Task 0 (sl-c-1-impl-0, task #141):** all 16 probes PASS.
Task 0 made no commits (verification-only); `phase-v1-SL-c-1` tip
remains at `477f0c55c chore(bm): branch cut — phase-v1-SL-c-1 off
governance-v0`.

SL-a substrate confirmed in place:

- 3 `CaseStatus` variants at `crates/db_schema_file/src/enums.rs`
  lines 416 (`SponsorLiabilityPending`), 421 (`SponsorLiabilityFired`),
  427 (`SponsorLiabilityEscaped`).
- 2 schema columns at `crates/db_schema_file/src/schema.rs` lines
  799 (`grace_expires_at -> Nullable<Timestamptz>`), 800
  (`liability_escape_reason -> Nullable<Jsonb>`).
- 3 ENTRY_KIND consts at
  `crates/db_schema/src/source/governance/governance_log.rs` lines
  196 (`_RESTORATION_COMPLETED`), 197 (`_SPONSOR_LIABILITY_ESCAPED`),
  198 (`_SPONSOR_LIABILITY_FIRED`).
- 3 shim re-exports at
  `crates/api/api/src/governance/governance_log.rs` lines 68, 74,
  75.
- 4 SL-a config defaults at
  `crates/api/api/src/governance/config.rs` lines 929
  (`DEFAULT_LIABILITY_GRACE_WINDOW_MAXIMUM_HOURS = 720`), 943
  (`DEFAULT_JOB_GRACE_CHECK_INTERVAL_MINUTES = 5`), 944
  (`DEFAULT_JOB_GRACE_CHECK_BATCH_SIZE = 100`), 945
  (`DEFAULT_JOB_GRACE_CHECK_STALENESS_ALERT_MULTIPLIER = 2.0`); all
  also wired into `governance_config_resolve_*` match arms (lines
  ~1023, 1031, 1032, 1055) and the SEEDED_KEYS_WITH_CONSTS table
  (lines ~1373/1374).

SL-b shipped (PR #119 merged 2026-05-07): `revoke_endorsement.rs`
present at `crates/api/api_crud/src/governance/`. v0 helper signature
confirmed at `sponsor_liability.rs:142`:
`pub(crate) async fn apply_sponsor_liability(conn: &mut
AsyncPgConnection, target_person_id: PersonId, case_id:
ModerationCaseId, community_id: Option<CommunityId>, action:
SanctionAction, cache: &mut ConfigCache) -> LemmyResult<usize>`.

Sanction multiplicity 1:1 confirmed: `submit_jury_vote.rs:435` is
the single `insert_into(sanction::table)` site in
`crates/api/`.

REPUTATION/APPEAL guard mirrors confirmed at
`scheduled_tasks.rs:71-79` + `:83-91` (2 existing pairs). c-1
Task 2 will add the third.

No concurrent PRs touch c-1's target files
(`sponsor_liability_grace.rs`, `governance/mod.rs`,
`scheduled_tasks.rs`).

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branched off `phase-v1-SL-c-1`
  (tip `477f0c55c`).
- `git branch --show-current` should return a `junior/role-impl-task-...-<task-id>`
  branch — adapt to your actual worktree branch name.
- **Two commits at task end:**
  1. **Impl commit**, subject:
     `feat(v1-SL-c-1): create sponsor_liability_grace module + 4 pub fns + module wiring (task 1)`
  2. **DQ-entry commit**, subject (matches schema-v2):
     `chore(decision-queue): impl raised DQ #<next-id> — sl-c-1-impl-1 validate-pending`
- After both commits land, push the worker branch to origin AS A
  SINGLE PUSH. Junior's daemon finalize-merges into
  `phase-v1-SL-c-1` on completion.

### Implementation discipline

- **File 1: `crates/api/api/src/governance/sponsor_liability_grace.rs`
  — NEW.** Write the full module body per trunk plan §10.1
  (`run_grace_check_batch`) + §10.2 (`fire_or_escape_case` +
  `fire_or_escape_case_inner`) + §10.4 (`check_grace_staleness`)
  + the `evaluate_escape_conditions` body. The c-1 plan §13 Task 1
  explicitly says "the trunk is the canonical source ... the
  plan-implement subagent reads the trunk's task 1 body and writes
  identical Rust."

- **File 2: `crates/api/api/src/governance/mod.rs` — MODIFY.**
  After line 38 (`pub mod sponsor_liability;`), insert exactly one
  line:
  ```rust
  pub mod sponsor_liability_grace;
  ```
  Insertion lands between `pub mod sponsor_liability;` and
  `pub mod submit_jury_vote;` (alphabetical). Verified position
  pre-Task-1 by Task 0.

- **R1 (i64 typing — c-1 watchpoint #13 + plan §13 Task 1
  GOTCHA):**
  - `batch_size: i64` (config `get_int` returns i64); diesel
    `limit(...)` accepts i64.
  - `chrono::Duration::hours(threshold_hours)` requires i64.
  - Staleness arithmetic: `(max_grace_hours as f64 *
    multiplier).round() as i64` — no i32 intermediate.
  - `usize::try_from(batch_size)` for the for-loop counter (mirror
    `reputation_snapshot.rs:367-371`).

- **Two-tier ConfigCache (DQ #144 in c-1 plan §4.1 + GOTCHA):**
  - Outer cache: `let mut cache = ConfigCache::new();` at the top
    of `run_grace_check_batch`, used for batch-level reads
    (`job.grace_check_batch_size`,
    `liability.grace_window_maximum_hours`,
    `job.grace_check_staleness_alert_multiplier`).
  - Per-case cache: `let mut per_case_cache = ConfigCache::new();`
    at the top of `fire_or_escape_case_inner` (inside the per-case
    scope), threaded into `apply_sponsor_liability(... &mut
    per_case_cache)`.

- **Per-case `run_transaction` (per c-1 plan §4.1 step ordering +
  watchpoint #1 FOR UPDATE):** the outer `run_grace_check_batch`
  does NOT open a transaction. Each iteration of the for-loop
  opens its own `conn.run_transaction(|conn| async move {
  ... }.scope_boxed())`. Inside-tx step order:
  1. Re-load `moderation_case` row with `for_update()`.
  2. Re-check status; if not `SponsorLiabilityPending`, return
     `Ok(())` from the per-case body.
  3. Lookup sanction action via
     `sanction::table.filter(case_id.eq(case_id)).order_by(id.asc()).limit(1).select((action, scope)).first(conn).await.optional()`.
     If `None`, `tracing::error!` + return `Ok(())`.
  4. Branch on `EscapeStatus`:
     - `Escape{reason, actor_pseudonym, ref_id}`: UPDATE case
       to `SponsorLiabilityEscaped`, set `liability_escape_reason`
       JSONB per PRD §8.1 schema (`{"version": 1, "reason": ...,
       "actor_pseudonym": ..., "endorsement_id": ...}`); emit
       `governance_log::append(... ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED, ...)`.
     - `Fire`: invoke
       `apply_sponsor_liability(conn, target_person_id, case_id,
       community_id, action, &mut per_case_cache).await?`; UPDATE
       case to `SponsorLiabilityFired`; emit
       `governance_log::append(... ENTRY_KIND_SPONSOR_LIABILITY_FIRED, ...)`
       — single summary entry on TOP of v0's per-sponsor entries.

- **Outer per-case error swallow (c-1 plan §4.1 + watchpoint #8):**
  the for-loop body wraps the `fire_or_escape_case` call;
  `.inspect_err(|e| warn!("grace_check case_id={}: {e}", case.id)).ok()`
  on `Err`. Outer function returns `Ok(...)` regardless.

- **Restoration-escape stub (c-1 watchpoint #14 + DQ #145):**
  `evaluate_escape_conditions` defines the
  `EscapeStatus::Escape{reason: "restoration_completed", ...}`
  enum value as a documented future-wire branch but the function
  body NEVER constructs it. The restoration-completed read returns
  `Fire` unconditionally. Use `let _ = case_id;` (or similar)
  inside the future-wire branch to suppress unused-var lints.
  Document the stub in a doc-comment block.

- **ADR-013 exhaustive match (c-1 watchpoint #14):** every `match`
  on `EscapeStatus` MUST enumerate `Escape{...}` AND `Fire`
  arms. NO `_ =>` arm.

- **ADR-015 pseudonym discipline (c-1 watchpoint #4):**
  `liability_escape_reason` JSONB carries `actor_pseudonym`
  (string from `actor_pseudonym_helper::get_or_create`), NOT raw
  `caller_id` / raw `from_person_id`. Any raw id in payload is a
  GDPR-013 violation — STOP and file DQ blocker.

- **`Selectable` derive on ModerationCase:** struct already derives
  `Identifiable, Queryable, Selectable` per
  `crates/db_schema/src/source/governance/moderation_case.rs:13-94`
  (verified Task 0). Use `ModerationCase::as_select()` for the
  batch query.

- **No `#[cfg(feature = ...)]` gate on the new module.** Plain
  `pub mod` line; the module body is not feature-gated.

### Validate-pending DQ entry shape (Shape G Layer G2)

After the impl commit lands and you push to origin, capture the
workflow_run_id:

```bash
gh run list --repo barrie-cork/lemmy --branch <your-worker-branch> \
  --workflow cargo-validate-workspace --limit 1 \
  --json databaseId --jq '.[0].databaseId'
```

If `gh run list` returns empty (workflow hasn't started yet), wait
up to 60 seconds and retry — GH Actions ingestion lag.

DQ entry to write (commit subject:
`chore(decision-queue): impl raised DQ #<next-id> — sl-c-1-impl-1 validate-pending`):

```json
{
  "id": <next-int>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<NOW_ISO>",
  "subject": "sl-c-1-impl-1 workspace-check validate-pending",
  "branch": "junior/<task-slug>",
  "phase_task": "sl-c-1-impl-1",
  "workflow_run_id": <int from gh run list>,
  "commands": [
    "cargo check --workspace --features full",
    "cargo clippy --workspace --features full --no-deps -- -D warnings",
    "cargo test --no-run -p lemmy_server --test e2e"
  ],
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

Compute `next-int` by reading `.claude/decision-queue.json` and
returning `max(all ids in pending + resolved) + 1`. The current max
is `159`; next id `160` unless other concurrent writes raced you.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"` (rare for impl-task;
  only for trivial questions answerable with file evidence).
- Any pending DQ entry written from this task uses `from: "impl"`.

### Hard refusals

- Do NOT modify any file outside `crates/api/api/src/governance/sponsor_liability_grace.rs`
  (NEW), `crates/api/api/src/governance/mod.rs` (MODIFY), and
  `.claude/decision-queue.json` (the validate-pending entry).
- Do NOT touch `scheduled_tasks.rs` — that's Task 2.
- Do NOT touch any `crates/server/tests/e2e.rs` content — c-2's
  scope.
- Do NOT touch any `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.
- Do NOT add new `ENTRY_KIND_*` consts, `CaseStatus` variants,
  schema columns, or `governance_config` seeds.
- Do NOT introduce a `_ =>` catch-all arm in the `EscapeStatus`
  match.
- Do NOT add a `#[cfg(feature = ...)]` gate on the new module or
  any of its public symbols.
- Do NOT push the worker branch BEFORE both commits (impl + DQ
  entry) land.
- Do NOT run cargo locally — Shape G Layer G2; cargo runs on
  GitHub-hosted runners after push.

### Sentinel sequence (mid-task visibility)

Per `.claude/rules/decision-queue.md` "Mid-task visibility": if you
discover an unexpected blocker mid-task (e.g. a struct shape Task 0
didn't anticipate, a missing import, a downstream callee signature
mismatch), write a DQ blocker entry immediately, commit, and push
the worker branch IMMEDIATELY (not at task end) so the advisor can
see it on next polling tick. The push is the ONE non-finalize push
allowed mid-task.

## 5. Acceptance

Task 1 passes if:

- `crates/api/api/src/governance/sponsor_liability_grace.rs` is
  created with all required public symbols
  (`run_grace_check_batch`, `evaluate_escape_conditions`,
  `fire_or_escape_case`, `check_grace_staleness`, `EscapeStatus`,
  `GraceCheckBatchOutcome`).
- `crates/api/api/src/governance/mod.rs` line 38 area shows
  `pub mod sponsor_liability;` then on the next line
  `pub mod sponsor_liability_grace;` then `pub mod submit_jury_vote;`
  (alphabetical).
- One impl commit + one DQ-entry commit on the worker branch.
- Worker branch pushed to origin.
- DQ entry written with `kind: "validate-pending"`, `from: "impl"`,
  populated `workflow_run_id`.
- ci-watcher's later poll of `cargo-validate-workspace.yml` returns
  `conclusion: "success"`.

(The last bullet is verified by ci-watcher — not by this Task 1
worker. Task 1 worker is done after writing the DQ entry +
pushing.)
