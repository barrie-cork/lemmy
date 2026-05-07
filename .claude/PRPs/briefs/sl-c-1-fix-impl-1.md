---
role: impl-task
plan_task: 1-fix
phase: v1-SL-c-1
created: 2026-05-07
related_dq: 160
---

# Brief — v1-SL-c-1 fix-impl-1 — wrong import paths in sponsor_liability_grace.rs

## 1. Role + dispatch line

`[role:impl-task] sl-c-1-fix-impl-1 — see .claude/PRPs/briefs/sl-c-1-fix-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a **§G4-allowlist
narrow fix-in-pr** task per `.claude/rules/advisor-orchestrator.md` "§G4
classifier" (E0432: unresolved import row). Scope is **strictly** the two
import-path mistakes in `sponsor_liability_grace.rs` that broke
workspace-check on workflow run `25525281390`. No other changes
permitted.

## 2. Scope

**Produce:**

1. **One impl commit** with import-block fixes + one fn signature change
   on `crates/api/api/src/governance/sponsor_liability_grace.rs`. ≤3
   logical edits, all in this single file.
2. **Push the worker branch** to `origin/junior/<task-slug>`.
3. **Write a `kind: "validate-pending"` DQ entry** capturing the new
   `cargo-validate-workspace.yml` workflow run id per Shape G Layer G2.
   This is a SECOND commit on the same worker branch.

**Do NOT** in this task:

- Touch any file other than `sponsor_liability_grace.rs` and
  `.claude/decision-queue.json`.
- Refactor any other code paths or "improve" surrounding logic.
- Touch the doc-comments at the top of the file or any field on the 4
  pub items (`run_grace_check_batch`, `evaluate_escape_conditions`,
  `fire_or_escape_case`, `check_grace_staleness`, plus the `EscapeStatus`
  enum and `GraceCheckBatchOutcome` struct) other than the one signature
  change in `fire_or_escape_case`.
- Add new tests.
- Run cargo locally — Shape G; cargo runs on GH-hosted runners.
- Modify `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.
- Resolve DQ #160 — its mutation already populated `result: fail`; this
  fix-impl raises a NEW `validate-pending` DQ entry referencing a NEW
  workflow run on the new worker branch.

## 3. Required reading

Read in this order before writing the Edits:

1. **DQ #160** in `.claude/decision-queue.json` `pending[]` — read the
   `log_slice` field for the verbatim two compile errors (E0432 +
   E0599) and their file:line references.
2. **`crates/api/api/src/governance/sponsor_liability_grace.rs`** lines
   50-75 (import block) AND lines 279-340 (`fire_or_escape_case`
   signature + body). Don't read the full file body — only those two
   regions are in scope.
3. **`crates/api/api/src/governance/submit_jury_vote.rs`** lines 56-92
   (canonical import pattern in same crate) — `PersonId` is imported
   from `lemmy_db_schema_file::PersonId`, NOT `lemmy_db_schema::newtypes`.
   Sibling files `jury_common.rs:10`, `reputation_snapshot.rs:58`,
   `sponsor_liability.rs:59`, `actor_pseudonym_helper.rs:17`,
   `admin_audit_stream.rs:49` all confirm the same path.
4. **`crates/diesel_utils/src/connection.rs`** lines 50-80 — read the
   `DbConn` enum and the `impl DbConn<'_>::run_transaction` block. The
   `run_transaction` method exists on `DbConn`, NOT on
   `AsyncPgConnection`. The plan §13 Task 1 IMPLEMENT block prescribed
   `&mut AsyncPgConnection` for `fire_or_escape_case`'s `conn` param;
   that is wrong for callers needing to open a tx.
5. **`crates/tools/seed_founders/src/main.rs:40`** — canonical
   `use lemmy_diesel_utils::connection::{DbConn, ...};` import path.
6. **`.claude/lessons/feedback_newtype_locations_lemmy_db_schema_vs_file.md`**
   — the exact trap the IMPLEMENT block fell into. PersonId, LocalUserId,
   LocalSiteId etc. are in `lemmy_db_schema_file::`; only the governance
   newtypes (CommunityId, ModerationCaseId, etc.) are in
   `lemmy_db_schema::newtypes::`.
7. **`.claude/rules/decision-queue.md`** — schema-v2 for the new
   `validate-pending` entry.

## 3a. Handover from prior cohort

**From Task 1 (sl-c-1-impl-1, Junior task #142):** SHIPPED but failed
workspace-check.
- `crates/api/api/src/governance/sponsor_liability_grace.rs` exists at
  ~395 lines; module wired through `mod.rs:39`.
- Two compile errors per DQ #160 mutation by ci-watcher #143:
  - **E0432** at `sponsor_liability_grace.rs:56:45`: `no PersonId in
    newtypes` — `lemmy_db_schema::newtypes::{CommunityId,
    ModerationCaseId, PersonId}` is wrong; PersonId is in
    `lemmy_db_schema_file`.
  - **E0599** at `sponsor_liability_grace.rs:287:6`: `no method named
    run_transaction found for &mut AsyncPgConnection` — the inner
    method exists on `DbConn`, not the bare diesel-async type. Compiler
    suggests `transaction` (the upstream Diesel API name); we want the
    fork's `run_transaction` extension on `DbConn`. Fix is a type
    change on `fire_or_escape_case`'s `conn` param, NOT a method rename.
- Phase tip after ci-watcher mutation: `a53fba8ee` on
  `phase-v1-SL-c-1`.

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branched off `phase-v1-SL-c-1` (tip
  `a53fba8ee` per ci-watcher #143's daemon-finalize push).
- `git branch --show-current` should return a `junior/role-impl-task-...`
  branch (adapt to your actual worktree branch name).
- Two commits at task end:
  1. Impl: `fix(v1-SL-c-1): correct PersonId import path + DbConn type on fire_or_escape_case`
  2. DQ: `chore(decision-queue): impl raised DQ #<next-id> — sl-c-1-fix-impl-1 validate-pending`

### Implementation discipline — the EXACT edits

**The new module's current import block (verify by reading the worktree
copy of `crates/api/api/src/governance/sponsor_liability_grace.rs` lines
50-75):**

```rust
use diesel::{
  ExpressionMethods,
  QueryDsl,
  SelectableHelper,
  dsl::min,
};
use diesel_async::{
  AsyncPgConnection,
  RunQueryDsl,
  scoped_futures::ScopedFutureExt,
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::{CommunityId, ModerationCaseId, PersonId},
  source::governance::moderation_case::ModerationCase,
};
use lemmy_db_schema_file::{
  enums::{CaseStatus, SanctionAction, SanctionScope},
  schema::{endorsement, moderation_case, sanction, surety},
};
use lemmy_diesel_utils::connection::get_conn;
```

**Replace with (only the lines that change — adjust ordering to match
existing alphabetical-within-block convention; per
`crates/api/api_crud/src/governance/admin_close_case.rs:17` the
canonical pattern is `lemmy_db_schema_file::{enums::..., schema::...,
PersonId}` or breaking PersonId onto its own use, both acceptable):**

```rust
use lemmy_db_schema::{
  newtypes::{CommunityId, ModerationCaseId},                 // ← remove PersonId
  source::governance::moderation_case::ModerationCase,
};
use lemmy_db_schema_file::{
  PersonId,                                                  // ← add PersonId
  enums::{CaseStatus, SanctionAction, SanctionScope},
  schema::{endorsement, moderation_case, sanction, surety},
};
use lemmy_diesel_utils::connection::{DbConn, get_conn};       // ← add DbConn
```

The order inside the `lemmy_db_schema_file::{...}` block must be
alphabetical at the top level (`PersonId`, then `enums`, then `schema`)
to match the convention used in `submit_jury_vote.rs:67-88`.

**Then change the `fire_or_escape_case` public entry signature** at
~line 279-284:

```rust
pub async fn fire_or_escape_case(
  conn: &mut AsyncPgConnection,    // ← change
  case: ModerationCase,
  status: EscapeStatus,
  _cache: &mut ConfigCache,
) -> LemmyResult<()> {
```

**Replace with:**

```rust
pub async fn fire_or_escape_case(
  conn: &mut DbConn<'_>,           // ← DbConn for run_transaction
  case: ModerationCase,
  status: EscapeStatus,
  _cache: &mut ConfigCache,
) -> LemmyResult<()> {
```

The body of `fire_or_escape_case` is **unchanged**: the `.run_transaction(|conn| { ... })` call at line 287 will now resolve correctly because `DbConn::run_transaction` is the canonical Brehon-fork API and is the same shape used at line 172 inside `run_grace_check_batch`.

**Do NOT touch** `evaluate_escape_conditions` (line 205), `check_grace_staleness` (line 347), or `fire_or_escape_case_inner` (line 391). Those are correct — they take `&mut AsyncPgConnection` because they are called INSIDE a `run_transaction` closure (where `conn` is rebound to the bare `AsyncPgConnection` per `connection.rs:70`). Only the public top-level `fire_or_escape_case` opens its own tx and therefore needs `DbConn`.

**Do NOT** rename `.run_transaction` to `.transaction`. The compiler suggested it because the upstream Diesel API uses `transaction`; the Brehon fork has a `DbConn::run_transaction` extension that wraps that with `LemmyError` mapping. Renaming would break the fork's error mapping.

### Validate-pending DQ entry shape (Shape G Layer G2)

```json
{
  "id": <next-int>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<NOW_ISO>",
  "subject": "sl-c-1-fix-impl-1 workspace-check validate-pending",
  "branch": "junior/<task-slug>",
  "phase_task": "sl-c-1-fix-impl-1",
  "workflow_run_id": <int from gh run list>,
  "commands": ["cargo check --workspace --features full", "cargo clippy --workspace --features full --no-deps -- -D warnings", "cargo test --no-run -p lemmy_server --test e2e"],
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

**Compute next-id correctly per `.claude/rules/decision-queue.md` Hard
refusal #2:** `max(all_ids, default=0) + 1` across BOTH `pending[]` and
`resolved[]` AND any archive files. The advisor confirmed at 2026-05-07
that no archive files exist at this time, so `max(pending + resolved) +
1` suffices on this run.

**Use `ensure_ascii=False`** when re-serialising the JSON file per
`feedback_json_dump_ensure_ascii_false` — preserves UTF-8 (em-dashes,
section markers) cleanly. Do NOT escape `—` / `§`.

After pushing the impl commit, capture the `workflow_run_id`:

```bash
gh run list --repo barrie-cork/lemmy --branch <your-worker-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId --jq '.[0].databaseId'
```

If `gh run list` returns empty, wait up to 60 seconds and retry.

### Hard refusals

- Do NOT modify any file other than the two named (`sponsor_liability_grace.rs` + `.claude/decision-queue.json`).
- Do NOT touch anything in `sponsor_liability_grace.rs` other than the
  imports (lines ~50-75) and `fire_or_escape_case`'s signature line
  (the `conn: &mut AsyncPgConnection` line). Function bodies, doc
  comments, struct/enum definitions, etc. are out of scope.
- Do NOT add `unwrap()` or `expect()` anywhere.
- Do NOT change the method name from `run_transaction` to `transaction`.
- Do NOT add `#[allow(...)]` to suppress the errors.
- Do NOT mutate or resolve DQ #160. Raise a new entry.
- Do NOT push the worker branch BEFORE both commits land.
- Do NOT escape non-ASCII characters when writing decision-queue.json.

## 5. Acceptance

Fix-impl-1 passes if:

- `git diff HEAD~1 -- crates/api/api/src/governance/sponsor_liability_grace.rs`
  shows ONLY (a) PersonId moved between two `use` blocks, (b) `DbConn`
  added to the `lemmy_diesel_utils::connection::{...}` block, and (c)
  the `fire_or_escape_case` signature line changed from
  `&mut AsyncPgConnection` to `&mut DbConn<'_>`.
- No other diff lines anywhere in `.rs` files.
- One impl commit + one DQ-entry commit on the worker branch.
- DQ entry written with `kind: "validate-pending"`, `from: "impl"`,
  populated `workflow_run_id`.
- ci-watcher's later poll on the new workflow run returns
  `conclusion: "success"`.
