# Plan: Phase 4a — Route Module + First Five Endpoint Handlers

## Summary

Phase 4a delivers the HTTP layer for the Brehon governance API: a new route module under `/api/v4/governance/` and the first five endpoint handlers that prove the governance mechanic works end-to-end. This phase creates the cross-cutting infrastructure (governance log writer, actor pseudonym helper, redaction service) and wires five handlers that consume the Phase 1 schema, Phase 2 views, and Phase 3 DTOs. The remaining Phase 4 tasks (admin backstops, EmergencyRemove helper, server wiring, golden-path e2e test, threshold placeholder) are deferred to Phase 4b.

## Source

- [IMPLEMENTATION-PLAN-v0.md](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) §3 Phase 4 (tasks 38–43 only)
- Relevant [04](docs/brehon-law-inspired-network/04-data-model-and-api.md) sections: §5 (DTOs), §6 (Handlers), §7 (Routes), §8 (Aggregation)
- Relevant ADRs from [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md): ADR-004 (governance plane separation), ADR-007 (5-juror/quorum-3/simple-majority), ADR-008 (append-only signed log), ADR-013 (EmergencyRemove exhaustive match), ADR-015 (GDPR pseudonyms)

## Problem Statement

Phases 1–3 created the schema, views, and DTOs but no HTTP surface. A client cannot yet call any governance endpoint. Phase 4a bridges this gap by registering `/api/v4/governance/` routes and implementing the five core handlers that form the vertical slice: report, get-case, jury-queue, jury-vote, and modlog. The three cross-cutting helpers (governance log append, actor pseudonym get-or-create, redaction scrub) are also created here because every governance write handler depends on them.

## Solution Statement

Add governance routes as a new `.service(scope("/governance"))` block inside the existing `crates/api/routes/src/lib.rs` `config()` function — the same pattern used for `/community`, `/post`, `/admin`, etc. Handlers live in `crates/api/api/src/governance/` (workflow handlers) and `crates/api/api_crud/src/governance/` (CRUD handlers), matching Lemmy's organisational split. Cross-cutting helpers live alongside the handlers in `crates/api/api/src/governance/`. Cargo.toml files for `lemmy_api` and `lemmy_api_crud` gain dependencies on the governance view crates.

## Metadata

| Field | Value |
|---|---|
| Type | HANDLER |
| Complexity | HIGH |
| Crates Affected | `api/routes`, `api/api`, `api/api_crud`, (Cargo.toml only: `api/api`, `api/api_crud`) |
| v0 Step | Step 4 from [05 §4](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) |
| Dependencies | Phase 1 (schema + models), Phase 2 (view crates + queries), Phase 3 (DTOs) |
| Estimated Tasks | 10 |

---

## Flow Design

### Before State

```
╔══════════════════════════════════════════════════════════════════╗
║  Phases 1-3 complete:                                          ║
║  - Governance tables + Diesel models exist (Phase 1)           ║
║  - View crates with queries exist (Phase 2)                    ║
║  - DTOs in api_common compile (Phase 3)                        ║
║                                                                ║
║  NO HTTP SURFACE:                                              ║
║  - No routes registered under /api/v4/governance/*             ║
║  - No handler functions anywhere                               ║
║  - No governance log writer, pseudonym helper, or redaction    ║
║  - Client cannot call any governance endpoint                  ║
╚══════════════════════════════════════════════════════════════════╝
```

### After State

```
╔══════════════════════════════════════════════════════════════════╗
║  Phase 4a adds the HTTP layer:                                 ║
║                                                                ║
║  POST /api/v4/governance/report  → create_report handler       ║
║  GET  /api/v4/governance/case    → get_case handler            ║
║  GET  /api/v4/governance/jury/me → list_my_jury_queue handler  ║
║  POST /api/v4/governance/jury/vote → submit_jury_vote handler  ║
║  GET  /api/v4/governance/modlog  → list_modlog handler         ║
║                                                                ║
║  Cross-cutting helpers (used by all write handlers):           ║
║  governance_log::append()  → hash-chained, signed log entry    ║
║  actor_pseudonym::get_or_create() → GDPR pseudonym             ║
║  redaction::scrub()        → strip identifiers from strings    ║
║                                                                ║
║  Data flow:                                                    ║
║  HTTP request → handler → view query / Diesel insert →         ║
║  governance_log::append (with pseudonym + scrub) → response    ║
╚══════════════════════════════════════════════════════════════════╝
```

### Endpoint Changes

| Endpoint | Before | After |
|---|---|---|
| `POST /api/v4/governance/report` | didn't exist | Creates/appends to `moderation_case`; emits log entry |
| `GET /api/v4/governance/case` | didn't exist | Permission-aware read of one case |
| `GET /api/v4/governance/jury/me` | didn't exist | Lists caller's jury assignments |
| `POST /api/v4/governance/jury/vote` | didn't exist | Records vote; triggers decision if quorum reached |
| `GET /api/v4/governance/modlog` | didn't exist | Public, redacted moderation log (no auth) |

---

## Mandatory Reading (implementation agent MUST read before starting)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `crates/api/routes/src/lib.rs` | 192–485 | Route registration pattern to MIRROR |
| P0 | `crates/api/api/src/reports/comment_report/create.rs` | 31–60 | Handler signature pattern: auth, validation, DB, response |
| P0 | `crates/api/api/src/site/mod_log.rs` | all | GET handler with optional auth + pagination to mirror for `list_modlog` |
| P0 | `crates/api/api_common/src/governance.rs` | all | Phase 3 DTOs — the request/response types handlers consume |
| P0 | `crates/db_views/governance_case/src/impls.rs` | all | View queries the handlers call |
| P0 | `crates/db_views/jury_queue/src/impls.rs` | all | `list_jury_assignments_for_person` for jury/me handler |
| P0 | `crates/db_views/governance_modlog/src/impls.rs` | all | `list_public_case_log` for modlog handler |
| P0 | `crates/db_schema/src/source/governance/governance_log.rs` | all | GovernanceLogInsertForm for log writer |
| P0 | `crates/db_schema/src/source/governance/actor_pseudonym.rs` | all | ActorPseudonymInsertForm for pseudonym helper |
| P0 | `crates/api/api/src/community/ban.rs` | 59-64 | `run_transaction` pattern — MUST use for `submit_jury_vote` |
| P1 | `crates/api/api_utils/src/utils.rs` | 149, 188 | `is_admin()` and `check_local_user_valid()` patterns |
| P1 | `crates/api/api/src/local_user/add_admin.rs` | all | Admin-auth handler pattern |
| P1 | `crates/api/api_crud/src/community/list.rs` | all | GET+pagination handler pattern |
| P1 | `docs/brehon-law-inspired-network/04-data-model-and-api.md` | §6, §7, §8 | Handler responsibilities, route tree, aggregation rules |

---

## Patterns to Mirror

**HANDLER_SIGNATURE (POST, required auth):**
```rust
// SOURCE: crates/api/api/src/reports/comment_report/create.rs:31-35
pub async fn create_comment_report(
  Json(data): Json<CreateCommentReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentReportResponse>> {
```

**HANDLER_SIGNATURE (GET, optional auth):**
```rust
// SOURCE: crates/api/api_crud/src/community/list.rs:9-13
pub async fn list_communities(
  Query(data): Query<ListCommunities>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<PagedResponse<CommunityView>>> {
```

**HANDLER_SIGNATURE (GET, required auth, query params):**
```rust
// SOURCE: crates/api/api_crud/src/post/read.rs:21-25
pub async fn get_post(
  Query(data): Query<GetPost>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPostResponse>> {
```

**ADMIN_AUTH_CHECK:**
```rust
// SOURCE: crates/api/api/src/local_user/add_admin.rs:21
is_admin(&local_user_view)?;
```

**USER_VALIDATION:**
```rust
// SOURCE: crates/api/api/src/reports/comment_report/create.rs:36
check_local_user_valid(&local_user_view)?;
```

**DB_POOL_ACCESS:**
```rust
// SOURCE: crates/api/api/src/reports/comment_report/create.rs:44
let comment_view = CommentView::read(
  &mut context.pool(),
  comment_id,
  ...
).await?;
```

**TRANSACTION_BOUNDARY (multi-step write):**
```rust
// SOURCE: crates/api/api/src/community/ban.rs:59-64
// IMPORT: use lemmy_diesel_utils::connection::get_conn;
let pool = &mut context.pool();
let conn = &mut get_conn(pool).await?;
let tx_data = data.clone();
conn
  .run_transaction(|conn| {
    async move {
      // All DB writes inside this closure
      // Use &mut conn.into() for each Diesel operation
    }
    .boxed()
  })
  .await?;
```

**ROUTE_REGISTRATION (scope-based):**
```rust
// SOURCE: crates/api/routes/src/lib.rs:225-253
.service(
  scope("/community")
    .route("", get().to(get_community))
    .route("", put().to(edit_community))
    .route("/follow", post().to(follow_community))
    .service(
      scope("/pending_follows")
        .route("/list", get().to(get_pending_follows_list))
        .route("/approve", post().to(post_pending_follows_approve)),
    ),
)
```

---

## Files to Change

| File | Action | Justification |
|---|---|---|
| `crates/api/api/Cargo.toml` | UPDATE | Add governance view crate dependencies |
| `crates/api/api_crud/Cargo.toml` | UPDATE | Add governance view crate dependencies |
| `crates/api/api/src/governance/mod.rs` | CREATE | Module root for governance handlers |
| `crates/api/api/src/governance/governance_log.rs` | CREATE | Cross-cutting: `append()` log writer |
| `crates/api/api/src/governance/actor_pseudonym_helper.rs` | CREATE | Cross-cutting: `get_or_create()` pseudonym |
| `crates/api/api/src/governance/redaction.rs` | CREATE | Cross-cutting: `scrub()` identifier stripper |
| `crates/api/api/src/governance/get_case.rs` | CREATE | GET /case handler |
| `crates/api/api/src/governance/list_my_jury_queue.rs` | CREATE | GET /jury/me handler |
| `crates/api/api/src/governance/submit_jury_vote.rs` | CREATE | POST /jury/vote handler |
| `crates/api/api/src/governance/list_modlog.rs` | CREATE | GET /modlog handler |
| `crates/api/api/src/lib.rs` | UPDATE | Add `pub mod governance;` |
| `crates/api/api_crud/src/governance/mod.rs` | CREATE | Module root for governance CRUD |
| `crates/api/api_crud/src/governance/create_report.rs` | CREATE | POST /report handler |
| `crates/api/api_crud/src/lib.rs` | UPDATE | Add `pub mod governance;` |
| `crates/api/routes/src/lib.rs` | UPDATE | Add governance route scope |

---

## NOT Building (v0 scope limits)

- `POST /api/v4/governance/admin/assign-jury` — Phase 4b (task 44)
- `POST /api/v4/governance/admin/close-case` — Phase 4b (task 45)
- `EmergencyRemove` helper function — Phase 4b (task 46)
- Server wiring (`crates/server/src/governance.rs`) — Phase 4b (task 47)
- Golden-path e2e test — Phase 4b (task 48)
- Threshold formula placeholder — Phase 4b (task 49)
- Remaining 6 MVP endpoints (accept/decline jury, appeal, reputation/me, endorsement, list cases) — Phase 5
- Federation outbound — Phase 6
- Frontend UI — not in v0
- Supermajority voting logic — v1 per [99 ADR-007](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)

---

## Step-by-Step Tasks

Execute in order. One commit per task. Each task has a MIRROR reference, exact file paths, and a validation command.

### Task 0: VERIFY branch state + pre-phase audit

- **ACTION**: Confirm we're on `governance-v0`, working tree is clean, Phase 3 HEAD is `61877804d`, and `cargo check --workspace` passes
- **VALIDATE**:
  ```bash
  git branch --show-current   # expect: governance-v0
  git status                  # expect: clean
  git log --oneline -1        # expect: 61877804d
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace > .claude/audit-phase4a-baseline.log 2>&1"
  tail -20 .claude/audit-phase4a-baseline.log
  echo "exit: $?"
  ```
- **EXPECT**: exit 0

---

### Task 1: UPDATE Cargo.toml files — add governance view crate dependencies

- **ACTION**: Add governance view crate dependencies to `lemmy_api` and `lemmy_api_crud` so handlers can import view structs and query functions
- **IMPLEMENT**:
  - In `crates/api/api/Cargo.toml` `[dependencies]` section, add:
    ```toml
    lemmy_db_views_governance_case = { workspace = true, features = ["full"] }
    lemmy_db_views_governance_modlog = { workspace = true, features = ["full"] }
    lemmy_db_views_jury_queue = { workspace = true, features = ["full"] }
    ```
  - In `crates/api/api_crud/Cargo.toml` `[dependencies]` section, add:
    ```toml
    lemmy_db_views_governance_case = { workspace = true, features = ["full"] }
    ```
    (api_crud only needs governance_case for `create_report` — the other views are used by api handlers)
- **MIRROR**: Existing view crate deps in `crates/api/api/Cargo.toml:25-50` — follow the `{ workspace = true, features = ["full"] }` pattern
- **GOTCHA**: The workspace `Cargo.toml` already declares `lemmy_db_views_governance_case`, `lemmy_db_views_governance_modlog`, and `lemmy_db_views_jury_queue` as workspace dependencies (confirmed). No root Cargo.toml change needed.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace > .claude/build-task01.log 2>&1"
  status=$?; tail -20 .claude/build-task01.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0, no new errors

---

### Task 2: CREATE cross-cutting helpers — governance_log, actor_pseudonym, redaction

- **ACTION**: Create three helper modules in `crates/api/api/src/governance/` that implement the cross-cutting requirements from [IMPLEMENTATION-PLAN-v0.md §4.1, §4.2](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)
- **IMPLEMENT**:

  #### `crates/api/api/src/governance/governance_log.rs`

  Public function: `pub async fn append(pool: &mut DbPool<'_>, entry_kind: &str, payload: serde_json::Value, actor_pseudonym: Option<String>) -> LemmyResult<GovernanceLog>`

  Steps:
  1. Call `redaction::scrub_json(&payload)` on the payload before insert (ensures no identifiers leak)
  2. Construct `GovernanceLogInsertForm { entry_kind, payload (scrubbed), actor_pseudonym }`
  3. `diesel::insert_into(governance_log::table).values(&form).get_result::<GovernanceLog>(conn).await?`
  4. The Postgres trigger computes `prev_hash` and `entry_hash` on INSERT
  5. **Signing (v0)**: Read back the row's `entry_hash`, sign with `ed25519-dalek` using key from `GOVERNANCE_LOG_SIGNING_KEY` env var, update the `signature` column:
     ```sql
     UPDATE governance_log SET signature = $1 WHERE id = $2 AND signature IS NULL
     ```
  6. Return the completed `GovernanceLog` row

  **IMPORTANT design choice for v0**: The signing step requires `ed25519-dalek` which is not yet in the `lemmy_api` Cargo.toml. Two options:
  - **(a) Add `ed25519-dalek` now** and implement real signing
  - **(b) Stub signing** — set `signature = None`, add `// TODO(brehon-fork): implement ed25519 signing in Phase 4b` comment

  **Recommend (b)** — signing is mentioned in IMPLEMENTATION-PLAN §4.1 Option A but the complexity of key management + the crate dependency is better validated with the full golden-path test in Phase 4b. The hash chain (trigger-side) is the critical integrity property; signing adds non-repudiation which is a v2 concern per [99 ADR-010](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md). The insert + hash chain is the Phase 4a deliverable; signing is Phase 4b.

  #### `crates/api/api/src/governance/actor_pseudonym_helper.rs`

  Public function: `pub async fn get_or_create(pool: &mut DbPool<'_>, person_id: PersonId) -> LemmyResult<String>`

  Steps:
  1. Query: `SELECT pseudonym FROM actor_pseudonym WHERE person_id = $1`
  2. If found: return pseudonym
  3. If not: generate `uuid::Uuid::new_v4().to_string()`, construct `ActorPseudonymInsertForm { person_id, pseudonym }`, insert, return pseudonym
  4. Handle race condition: if insert fails with unique violation (concurrent insert), retry the SELECT

  **Dependency**: `uuid` is already a workspace dependency (`uuid = { version = "1.22.0", features = ["serde"] }` at root `Cargo.toml:197`), but `lemmy_api`'s `Cargo.toml` does not list it. **Add `uuid = { workspace = true }` to `crates/api/api/Cargo.toml` `[dependencies]`.** This is deterministic — do not "check", just add it.

  #### `crates/api/api/src/governance/redaction.rs`

  Public functions:
  - `pub fn scrub(text: &str) -> String` — strips usernames, emails, URLs with username paths
  - `pub fn scrub_json(value: &serde_json::Value) -> serde_json::Value` — recursively scrubs all string values in a JSON tree

  Regex patterns:
  - Usernames: `@[a-zA-Z0-9_-]+`
  - Emails: `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}`
  - URLs with usernames: `https?://[^\s]+/(?:u|user|profile)/[a-zA-Z0-9_-]+`

  **v0 simplification**: No startup-loaded display-name blocklist. That's a v1 performance concern per §4.2. The regex-based scrub is sufficient for v0.

- **MIRROR**: Import patterns from `crates/api/api/src/reports/comment_report/create.rs` (for DB access style)
- **GOTCHA**: `GovernanceLogInsertForm` only has three fields (`entry_kind`, `payload`, `actor_pseudonym`) — the trigger handles `prev_hash`, `entry_hash`, and `signature` starts as NULL
- **GOTCHA**: `governance_log::append()` wraps redaction internally — callers never call `scrub()` separately for log payloads. Direct `scrub()` is for `public_case_log` strings.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/build-task02.log 2>&1"
  status=$?; tail -20 .claude/build-task02.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0

---

### Task 3: CREATE governance module structure in api and api_crud

- **ACTION**: Create `mod.rs` files and wire the governance modules into the crate lib.rs files
- **IMPLEMENT**:

  #### `crates/api/api/src/governance/mod.rs`
  ```rust
  pub mod actor_pseudonym_helper;
  pub mod get_case;
  pub mod governance_log;
  pub mod list_modlog;
  pub mod list_my_jury_queue;
  pub mod redaction;
  pub mod submit_jury_vote;
  ```

  #### `crates/api/api/src/lib.rs`
  Add `pub mod governance;` to the module list (after existing modules).

  #### `crates/api/api_crud/src/governance/mod.rs`
  ```rust
  pub mod create_report;
  ```

  #### `crates/api/api_crud/src/lib.rs`
  Add `pub mod governance;` to the module list.

- **MIRROR**: `crates/api/api/src/lib.rs:11-18` module declarations
- **GOTCHA**: The handler files from tasks 4-8 won't exist yet — create stub files with `// TODO: implement in task N` or create this task after 2 but before 4, with the mod.rs referencing files created in tasks 4-8. **Recommendation**: merge this task with task 2 (the helper files) and create empty stub handler files (`pub async fn handler_name() { todo!() }`) that get filled in tasks 4-8, OR create this task last after all handler files exist. **Best approach**: create this alongside the first handler file (task 4) and add each module declaration as each handler file is created. Task 3's validation is just `cargo check -p lemmy_api` — but at this point only the cross-cutting helpers from task 2 and the mod.rs exist.

  **Implementation note**: If task 2's helpers compile and the mod.rs only references them (not yet-unwritten handlers), task 3 can validate independently. Add handler module declarations one-by-one as tasks 4-8 land.

  **Revised approach**: Task 3 creates ONLY:
  - `crates/api/api/src/governance/mod.rs` with `pub mod governance_log; pub mod actor_pseudonym_helper; pub mod redaction;`
  - The `pub mod governance;` line in `crates/api/api/src/lib.rs`
  - `crates/api/api_crud/src/governance/mod.rs` (empty initially)
  - The `pub mod governance;` line in `crates/api/api_crud/src/lib.rs`

  Handler modules are added to mod.rs as each task creates the file.

- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/build-task03.log 2>&1"
  status=$?; tail -20 .claude/build-task03.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0

---

### Task 4: CREATE handler — `POST /api/v4/governance/report` → `create_report`

- **ACTION**: Implement `create_report` handler in `crates/api/api_crud/src/governance/create_report.rs`
- **IMPLEMENT** per [04 §6.1](docs/brehon-law-inspired-network/04-data-model-and-api.md):

  ```rust
  pub async fn create_report(
    Json(data): Json<CreateGovernanceReport>,
    context: Data<LemmyContext>,
    local_user_view: LocalUserView,
  ) -> LemmyResult<Json<CreateGovernanceReportResponse>> {
  ```

  Handler steps:
  1. `check_local_user_valid(&local_user_view)?`
  2. Validate target exists based on `data.target_type`:
     - `Post` → `Post::read(&mut context.pool(), PostId(data.target_id)).await?`
     - `Comment` → `Comment::read(&mut context.pool(), CommentId(data.target_id)).await?`
     - `Person` → `Person::read(&mut context.pool(), PersonId(data.target_id)).await?`
     - `Community` → `Community::read(&mut context.pool(), CommunityId(data.target_id)).await?`
     - `RemoteInstance` → validate URL format (for v0, just check non-empty)
  3. **Threshold contribution (v0 stub)**: `weight = 1.0` (reporter reputation stubbed at 1.0 per [99 OQ-006](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md))
  4. Check for existing open case on same target:
     - Query `moderation_case` WHERE matching target fields AND `status NOT IN (Closed, Decided)`
     - If found: increment `threshold_score` by weight (as i64), update case row
     - If not found: insert new `ModerationCase` via `ModerationCaseInsertForm`
  5. Check if accumulated `threshold_score > 3` (threshold constant per OQ-006 interim):
     - If yes: update `status = ThresholdMet`
     - Set `threshold_met = true` in response
  6. **Emit governance log entry**:
     - Get pseudonym: `actor_pseudonym_helper::get_or_create(&mut context.pool(), local_user_view.person.id).await?`
     - `governance_log::append(&mut context.pool(), "report_created", json!({ "case_id": case_id, "target_type": ..., "threshold_score": ..., "threshold_met": ... }), Some(pseudonym)).await?`
  7. If threshold just met, emit a second log entry: `governance_log::append(... "threshold_met" ...)`
  8. Return `CreateGovernanceReportResponse { case_id: Some(case_id), threshold_met }`

- **MIRROR**: `crates/api/api/src/reports/comment_report/create.rs:31-60` — validation, DB write, response pattern
- **GOTCHA**: No separate `report` table per [04 §13 shortcut](docs/brehon-law-inspired-network/04-data-model-and-api.md) — reports collapse into `moderation_case` directly
- **GOTCHA**: `threshold_score` is `Int8` in the Diesel schema macro (`crates/db_schema_file/src/schema.rs:700`) = `BigInt` = Rust `i64`. Verified: the model at `crates/db_schema/src/source/governance/moderation_case.rs:33` declares `pub threshold_score: i64`. **Use `1_i64` for v0** (reputation-weighted threshold stubbed to 1 per report). The threshold constant `3_i64` is the comparison target. Verify these types match before writing the handler — if the schema ever changes to `Int4`, the constant types must change too.
- **GOTCHA**: GDPR — log entry uses `actor_pseudonym`, never person_id or username
- **GOTCHA**: Update `crates/api/api_crud/src/governance/mod.rs` to add `pub mod create_report;`
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_crud > .claude/build-task04.log 2>&1"
  status=$?; tail -20 .claude/build-task04.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0

---

### Task 5: CREATE handler — `GET /api/v4/governance/case` → `get_case`

- **ACTION**: Implement `get_case` handler in `crates/api/api/src/governance/get_case.rs`
- **IMPLEMENT** per [04 §6.2](docs/brehon-law-inspired-network/04-data-model-and-api.md):

  ```rust
  pub async fn get_case(
    Query(data): Query<GetGovernanceCase>,
    context: Data<LemmyContext>,
    local_user_view: Option<LocalUserView>,
  ) -> LemmyResult<Json<GovernanceCaseDetailView>> {
  ```

  Handler steps:
  1. Call `read_case_detail(&mut context.pool(), data.case_id).await?`
  2. **Permission-aware response** (v0 simplified):
     - If caller is `None` (unauthenticated): return public slice — strip `evidence_count`, `target_creator_id`, redact sanctions list to just action types
     - If caller is admin (`local_user_view.local_user.admin == true`): return full detail
     - If caller is the target person: return full detail minus evidence
     - If caller is a juror on this case: return full detail
     - Otherwise: return public slice
  3. For the juror check: query `jury_assignment WHERE case_id = data.case_id AND person_id = caller_person_id AND status IN (Accepted, Submitted)`
  4. **Exhaustive match on `case_row.status`** — if `EmergencyRemove`, the response must include the fact of removal but NOT the content. If `AdminReview`, note the case is paused.

- **MIRROR**: `crates/api/api_crud/src/post/read.rs:21-112` — optional auth, permission-aware response
- **GOTCHA**: The `GovernanceCaseDetailView` from Phase 2 contains `row: GovernanceCaseDetailRow` and `sanctions: Vec<Sanction>`. For unauthenticated callers, strip or redact fields. For v0 simplicity, return the full struct for authenticated users and a reduced version for anonymous. Consider returning the full struct always (it's already scrubbed of personal data via Phase 2 views) and adding permission-based filtering as a v1 enhancement. **Decision**: return full `GovernanceCaseDetailView` for any authenticated user, and for unauthenticated callers, only `Decided` and `Closed` cases. **For non-public cases from unauthenticated callers, return `LemmyErrorType::NotFound`** (not silent filtering) so the caller knows the resource is not accessible rather than nonexistent.
- **GOTCHA**: Update `crates/api/api/src/governance/mod.rs` to add `pub mod get_case;`
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/build-task05.log 2>&1"
  status=$?; tail -20 .claude/build-task05.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0

---

### Task 6: CREATE handler — `GET /api/v4/governance/jury/me` → `list_my_jury_queue`

- **ACTION**: Implement `list_my_jury_queue` handler in `crates/api/api/src/governance/list_my_jury_queue.rs`
- **IMPLEMENT** per [04 §6.2](docs/brehon-law-inspired-network/04-data-model-and-api.md):

  ```rust
  pub async fn list_my_jury_queue(
    context: Data<LemmyContext>,
    local_user_view: LocalUserView,
  ) -> LemmyResult<Json<Vec<JuryQueueView>>> {
  ```

  Handler steps:
  1. `check_local_user_valid(&local_user_view)?`
  2. Call `list_jury_assignments_for_person(&mut context.pool(), local_user_view.person.id).await?`
  3. Return the `Vec<JuryQueueView>` directly — already sorted by `selected_at DESC` per Phase 2 implementation

- **MIRROR**: This is the simplest handler — analogous to `list_person_saved` pattern (auth required, direct view call, return list)
- **GOTCHA**: No request body or query params needed — the person_id comes from the JWT. If we want optional filtering (e.g., by status), add it as query params later. For v0, return all assignments.
- **GOTCHA**: `JuryQueueView` uses raw `i32` for `case_id` and `community_id` (not newtypes) — that's the Phase 2 design choice for the view layer.
- **GOTCHA**: Update `crates/api/api/src/governance/mod.rs` to add `pub mod list_my_jury_queue;`
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/build-task06.log 2>&1"
  status=$?; tail -20 .claude/build-task06.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0

---

### Task 7: CREATE handler — `POST /api/v4/governance/jury/vote` → `submit_jury_vote`

- **ACTION**: Implement `submit_jury_vote` handler in `crates/api/api/src/governance/submit_jury_vote.rs`
- **IMPLEMENT** per [04 §6.2](docs/brehon-law-inspired-network/04-data-model-and-api.md), [04 §8](docs/brehon-law-inspired-network/04-data-model-and-api.md), and [05 §6](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md):

  This is the most complex handler in v0.

  ```rust
  pub async fn submit_jury_vote(
    Json(data): Json<SubmitJuryVote>,
    context: Data<LemmyContext>,
    local_user_view: LocalUserView,
  ) -> LemmyResult<Json<SubmitJuryVoteResponse>> {
  ```

  **Note**: `SubmitJuryVoteResponse` is NOT yet defined in Phase 3 DTOs. **Add it** to `crates/api/api_common/src/governance.rs` with the full Phase 3 derive stack:
  ```rust
  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  /// Response from submitting a jury vote.
  pub struct SubmitJuryVoteResponse {
    pub vote_recorded: bool,
    pub case_decided: bool,
    pub decision: Option<JuryDecision>,
  }
  ```

  **CRITICAL — TRANSACTION BOUNDARY**: All writes in this handler MUST execute inside a single DB transaction. Use `run_transaction` per the Lemmy upstream pattern:

  ```rust
  // MIRROR: crates/api/api/src/community/ban.rs:60-64
  use lemmy_diesel_utils::connection::get_conn;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let tx_data = data.clone();
  conn
    .run_transaction(|conn| {
      async move {
        // ALL steps 2-8h inside this closure
        // Use &mut conn.into() for each Diesel operation
      }
      .boxed()
    })
    .await?;
  ```

  If any step fails (vote insert, sanction insert, case update, log append), the entire transaction rolls back. This prevents partial writes where a vote is recorded but the decision is not, or a sanction is created but the public log entry is not.

  Handler steps:
  1. `check_local_user_valid(&local_user_view)?` (BEFORE the transaction — this is a read-only auth check)
  2. **Inside `run_transaction`**: **Verify assignment**: query `jury_assignment WHERE case_id = data.case_id AND person_id = local_user_view.person.id AND status = Accepted`. If not found, return `LemmyErrorType::NotFound` or a governance-specific error.
  3. **Insert vote**: construct `JuryVoteInsertForm { case_id, juror_id: person_id, decision: data.decision, rationale: data.rationale.clone() }`, insert into `jury_vote`.
  4. **Update assignment status**: set `jury_assignment.status = Submitted`, `submitted_at = now()`
  5. **Emit governance log**: `governance_log::append(... "jury_vote_submitted" ...)` with pseudonym
  6. **Count submitted votes**: `SELECT COUNT(*) FROM jury_vote WHERE case_id = data.case_id`
  7. **If < 3 (quorum not reached)**: return `SubmitJuryVoteResponse { vote_recorded: true, case_decided: false, decision: None }`
  8. **If >= 3 (quorum reached)**:
     a. **Tally decisions**: group votes by `decision`, pick simple majority winner
     b. **Create sanction row**: map `JuryDecision` → `(SanctionScope, SanctionAction)`:
        - `NoAction` → no sanction row
        - `AdvisoryLabel` → `(Community, Label)`
        - `Warning` → `(Community, VisibilityReduction)`
        - `Cooldown` → `(Community, TemporaryRestriction)`
        - `RemoveContent` → `(Community, ContentRemoval)`
        - `SuspendLocalUser` → `(Instance, InstanceSuspension)`
        - `SuspendCommunityMember` → `(Community, CommunityExclusion)`
        - `RecommendFederationAction` → `(FederatedRecommendation, FederationQuarantineRecommendation)`
     c. If not `NoAction`: insert `SanctionInsertForm` with targets copied from the case
     d. **Update case**: `status = Decided`, `decided_at = now()`, `closed_at = now() + 7 days` (appeal window)
     e. **Insert PublicCaseLog row**: summary = `redaction::scrub(...)` applied to a generated summary string; `rationale_redacted = redaction::scrub(winning_rationale)` where winning_rationale is concatenated from majority voters
     f. **Emit reputation events** for each juror:
        - Aligned with majority → `ReputationEvent { dimension: JuryReliability, delta: +10 }`
        - Outlier → `ReputationEvent { dimension: JuryReliability, delta: -5 }`
     g. **Emit reputation events** for reporters on this case:
        - If sanctioned (not NoAction) → `ReputationEvent { dimension: ReportingAccuracy, delta: +10 }`
        - If dismissed (NoAction) → `ReputationEvent { dimension: ReportingAccuracy, delta: -5 }`
     h. **Emit governance log entries** for: decision, sanction creation (if any), public log publication, each reputation event
     i. Return `SubmitJuryVoteResponse { vote_recorded: true, case_decided: true, decision: Some(winning_decision) }`

- **MIRROR**: No direct Lemmy analogue for the complexity — closest is `crates/api/api/src/reports/comment_report/create.rs` for the multi-step write pattern
- **GOTCHA**: The "reporter" for reputation events — the field is `moderation_case.creator_id: Option<PersonId>` (verified at `crates/db_schema/src/source/governance/moderation_case.rs:23`). This is the first person to create the case. The impl agent **MUST verify this field name against the actual Diesel model** before writing the handler — do not assume a `reported_by_person_id` or similar name exists. If `creator_id` is `None` (e.g., system-generated case), skip reporter reputation events. Multiple reporters per case is a v1 concern.
- **GOTCHA**: Reputation event delta values (±10, ±5) are v0 placeholders. Document with `// TODO(brehon-fork): tune reputation deltas per OQ-006 before v1`.
- **GOTCHA**: The `ReputationEvent` insert form needs `source_case_id`. Set `source_report_id = None` (no separate report table in v0).
- **GOTCHA**: The appeal window `closed_at = now() + 7 days` is a hardcoded v0 constant. Document it.
- **GOTCHA**: Every `match` on `CaseStatus` or `JuryDecision` must be exhaustive — no `_ =>` catchall per ADR-013 principle.
- **GOTCHA**: `SubmitJuryVoteResponse` DTO must be added to `crates/api/api_common/src/governance.rs` as part of this task (Phase 3 didn't include it). Also add it to `mod.rs` exports.
- **GOTCHA**: Update `crates/api/api/src/governance/mod.rs` to add `pub mod submit_jury_vote;`
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/build-task07.log 2>&1"
  status=$?; tail -20 .claude/build-task07.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0

---

### Task 8: CREATE handler — `GET /api/v4/governance/modlog` → `list_modlog`

- **ACTION**: Implement `list_modlog` handler in `crates/api/api/src/governance/list_modlog.rs`
- **IMPLEMENT** per [04 §6.2](docs/brehon-law-inspired-network/04-data-model-and-api.md):

  ```rust
  pub async fn list_modlog(
    Query(data): Query<ListGovernanceModlog>,
    context: Data<LemmyContext>,
    _local_user_view: Option<LocalUserView>,
  ) -> LemmyResult<Json<Vec<GovernanceModlogView>>> {
  ```

  Handler steps:
  1. **No auth required** — this is a public transparency endpoint per [04 §7](docs/brehon-law-inspired-network/04-data-model-and-api.md). Accept `Option<LocalUserView>` but don't gate on it.
  2. If `data.community_id` is `Some`: call `list_public_case_log_for_community(&mut context.pool(), community_id).await?`
  3. If `data.community_id` is `None`: call `list_public_case_log(&mut context.pool()).await?`
  4. **Pagination**: the Phase 2 queries don't yet support pagination. For v0, apply `page` and `limit` as Rust-side `.skip()` and `.take()` on the result Vec. Default: `page = 1`, `limit = 20`, max limit = 50. This is acceptable for v0 — the governance modlog will have few entries. Phase 5 or v1 can push pagination into the Diesel query.
  5. Return the (possibly truncated) `Vec<GovernanceModlogView>`

- **MIRROR**: `crates/api/api/src/site/mod_log.rs` — public GET endpoint with optional auth and pagination
- **GOTCHA**: The Phase 2 `list_public_case_log()` function signature takes only `pool`. The `list_public_case_log_for_community()` may not exist yet (Phase 2 spec mentions it but check). If missing, call the full list and filter by community_id in Rust.
- **GOTCHA**: The modlog is the public transparency surface — ensure response is clean for unauthenticated callers. `GovernanceModlogView` already has `decision: Option<JuryDecision>` and `sanction_action: Option<SanctionAction>` as stubs (None) — that's fine; they'll be populated once `submit_jury_vote` creates the `PublicCaseLog` entries.
- **GOTCHA**: Update `crates/api/api/src/governance/mod.rs` to add `pub mod list_modlog;`
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/build-task08.log 2>&1"
  status=$?; tail -20 .claude/build-task08.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0

---

### Task 9: CREATE route module — register governance endpoints

- **ACTION**: Add governance route registration to `crates/api/routes/src/lib.rs`
- **IMPLEMENT**:

  In `crates/api/routes/src/lib.rs`:

  1. Add imports at the top of the file (in the `lemmy_api` and `lemmy_api_crud` import blocks):
     ```rust
     // In the lemmy_api_crud use block:
     use lemmy_api_crud::governance::create_report::create_report;

     // In the lemmy_api use block:
     use lemmy_api::governance::{
       get_case::get_case,
       list_modlog::list_modlog,
       list_my_jury_queue::list_my_jury_queue,
       submit_jury_vote::submit_jury_vote,
     };
     ```

  2. Add a new `.service(scope("/governance")...)` block inside the `scope("/api/v4")` chain, after the existing feature scopes (e.g., after the `/image` scope, before the closing `);`):

     ```rust
     // Brehon governance (fork-only) — see docs/brehon-law-inspired-network/04-data-model-and-api.md §7
     .service(
       scope("/governance")
         .route("/report", post().to(create_report))
         .route("/case", get().to(get_case))
         .route("/modlog", get().to(list_modlog))
         .service(
           scope("/jury")
             .route("/me", get().to(list_my_jury_queue))
             .route("/vote", post().to(submit_jury_vote)),
         ),
     )
     ```

  This registers exactly the five Phase 4a endpoints under `/api/v4/governance/`. Phase 4b will add `/admin/assign-jury` and `/admin/close-case`; Phase 5 will add the remaining endpoints.

- **MIRROR**: `crates/api/routes/src/lib.rs:225-253` — community scope pattern with nested sub-scopes
- **GOTCHA**: Route registration order within the `scope("/api/v4")` block doesn't matter for actix-web — but place the governance scope near the end (before the closing `)`) to keep the diff minimal and reviewable.
- **GOTCHA**: No rate-limit wrapper specific to governance in v0. Use the parent scope's `rate_limit.message()` which wraps the entire `/api/v4` scope. Custom governance rate limits are a v1 concern.
- **GOTCHA**: The `POST /governance/report` endpoint uses the `post()` extractor, same as other creation endpoints. The `GET /governance/case` uses `get()` with `Query` params. This matches the pattern for `/post`, `/community`, etc.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_routes > .claude/build-task09.log 2>&1"
  status=$?; tail -20 .claude/build-task09.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0

---

### Task 10: WORKSPACE validation — full check

- **ACTION**: Run full workspace cargo check + clippy to confirm everything integrates
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace > .claude/build-task10-check.log 2>&1"
  status=$?; tail -20 .claude/build-task10-check.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0, no governance-related warnings

  Then run clippy using the **clippy wrapper** (NOT cargo-check.bat — that runs `cargo check`, not `cargo clippy`). Use `--no-deps` to avoid upstream lint debt:
  ```bash
  cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --no-deps > .claude/build-task10-clippy.log 2>&1"
  status=$?; tail -20 .claude/build-task10-clippy.log; echo "exit: $status"
  ```
  Fix any clippy warnings on governance code before committing.

---

## Testing Strategy

Per [IMPLEMENTATION-PLAN-v0.md §5](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): **integration-only** for v0. Phase 4a does NOT include the golden-path e2e test (that's task 48 in Phase 4b). Phase 4a's validation is `cargo check --workspace` — it proves the handlers compile, the types align, and the route registration wires up. The e2e test in Phase 4b will prove the endpoints actually work end-to-end.

### Tests Deferred to Phase 4b

| Test Name | What It Validates | Phase 4b Task |
|---|---|---|
| `report_to_modlog_golden_path` | Full flow: report → jury → decision → modlog | 48 |
| `redaction_strips_identifiers` | Scrub removes usernames/emails from public log | 48 (part of golden path) |

### Edge Cases (verified by Phase 4b tests)

- [ ] Hash-chain integrity holds after governance log writes
- [ ] `actor_pseudonym` is generated (not person_id) in log entries
- [ ] `EmergencyRemove` branch handled in `get_case` response
- [ ] Redaction strips identifiers from public_case_log strings
- [ ] Duplicate vote by same juror is rejected
- [ ] Vote on non-existent case returns error
- [ ] Vote without accepted assignment returns error

---

## Validation Commands

Use these exact commands — do NOT substitute npm/pnpm/etc. This is a Rust project.

### Level 1: STATIC_ANALYSIS (every task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace > /tmp/check.log 2>&1"
status=$?; tail -20 /tmp/check.log; echo "exit: $status"
```

**EXPECT**: Exit 0, zero errors

### Level 2: PER-CRATE CHECK (after each handler)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > /tmp/check-api.log 2>&1"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_crud > /tmp/check-crud.log 2>&1"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_routes > /tmp/check-routes.log 2>&1"
```

### Level 3: FULL_BUILD (task 10)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace > /tmp/build.log 2>&1"
```

**EXPECT**: Exit 0, no errors

---

## Acceptance Criteria

- [ ] All five handlers compile (`cargo check -p lemmy_api`, `-p lemmy_api_crud`)
- [ ] Route module compiles (`cargo check -p lemmy_api_routes`)
- [ ] Full workspace compiles (`cargo check --workspace`)
- [ ] Cross-cutting helpers exist: `governance_log::append()`, `actor_pseudonym_helper::get_or_create()`, `redaction::scrub()`
- [ ] Routes registered at: `POST /governance/report`, `GET /governance/case`, `GET /governance/jury/me`, `POST /governance/jury/vote`, `GET /governance/modlog`
- [ ] Handler signatures match Lemmy patterns (actix-web extractors, `LemmyResult`, `Data<LemmyContext>`)
- [ ] `submit_jury_vote` implements the full quorum/decision/sanction/reputation/log flow per [04 §8](docs/brehon-law-inspired-network/04-data-model-and-api.md)
- [ ] `SubmitJuryVoteResponse` DTO added to api_common governance module
- [ ] No new `cargo clippy` warnings introduced
- [ ] No contradictions with the 15 ADRs
- [ ] Every governance log write calls `governance_log::append()` (not raw Diesel insert)
- [ ] Every log write uses `actor_pseudonym`, never person_id
- [ ] `CaseStatus` matches in `get_case` handle `EmergencyRemove` and `AdminReview` exhaustively

---

## Completion Checklist

- [ ] Task 0: Branch verified, baseline passes
- [ ] Task 1: Cargo.toml dependencies added
- [ ] Task 2: Cross-cutting helpers compile
- [ ] Task 3: Module structure wired
- [ ] Task 4: `create_report` handler compiles
- [ ] Task 5: `get_case` handler compiles
- [ ] Task 6: `list_my_jury_queue` handler compiles
- [ ] Task 7: `submit_jury_vote` handler compiles (most complex)
- [ ] Task 8: `list_modlog` handler compiles
- [ ] Task 9: Route module compiles and registers 5 endpoints
- [ ] Task 10: Full workspace validation passes
- [ ] All acceptance criteria met

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `uuid` crate not in workspace — adding it may conflict | LOW | LOW | Check root Cargo.toml first; `uuid` is common and likely already a transitive dep. If not, add as workspace dependency. |
| Governance log `INSERT` + trigger interaction fails at runtime | MED | MED | Phase 4a validates at compile time only. Phase 4b's golden-path test catches runtime trigger issues. Fallback: side-table for signatures per §7.1 of IMPL-PLAN. |
| `submit_jury_vote` handler is too complex for one task | MED | LOW | The handler compiles in one task but is tested in Phase 4b. If compilation issues arise, split into: vote-insert subtask + quorum-decision subtask. |
| Phase 2 view queries don't accept pagination params | HIGH | LOW | Apply Rust-side `.skip()/.take()` pagination in the handler (v0 acceptable). Push to Diesel queries in v1. |
| Missing response DTO for `submit_jury_vote` | CERTAIN | LOW | Task 7 adds `SubmitJuryVoteResponse` to api_common. This is a minor Phase 3 gap — document in the commit message. |
| Cross-cutting helpers need `serde_json` for log payloads | LOW | LOW | `serde_json` is already a workspace dependency of `lemmy_api`. No new dep needed. |
| Redaction regex may be too aggressive (strips legitimate @ mentions) | LOW | LOW | v0 acceptable — false positives in redaction are safer than false negatives. Tune regex in v1. |

---

## Design Decision: Signing Deferred to Phase 4b

The governance log `append()` helper in task 2 does NOT implement ed25519 signing. The hash chain (Postgres trigger) is the critical integrity property. Signing adds non-repudiation, which per [99 ADR-010](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) is a v2 concern (external signer). For v0, the signing key is in `.env` and the sign-then-update step can be added in Phase 4b when the golden-path test exercises the full write path. This avoids adding `ed25519-dalek` as a dependency before the full integration is testable.

## Design Decision: Pagination in Modlog Handler

The Phase 2 `list_public_case_log()` query returns all rows (no pagination params). The `list_modlog` handler applies Rust-side pagination via `.skip()` and `.take()`. This is acceptable for v0 (the governance modlog will have few entries). Phase 5 or v1 should push pagination into the Diesel query for efficiency.

## Design Decision: Response DTO Gap

`SubmitJuryVoteResponse` was not in the Phase 3 DTOs. Task 7 adds it to `crates/api/api_common/src/governance.rs`. This is a minor Phase 3 gap — the DTO wasn't specified in [04 §5](docs/brehon-law-inspired-network/04-data-model-and-api.md) because it's an operational response (not a domain object). The commit message should note this.

---

## Notes

- Phase 4a is the "compile" half of Phase 4. Phase 4b (tasks 44-49) adds the admin backstops, server wiring, and the golden-path e2e test that actually runs these handlers.
- The `submit_jury_vote` handler (task 7) is the single most complex function in v0. It implements the full aggregation rules from [04 §8](docs/brehon-law-inspired-network/04-data-model-and-api.md). If it compiles, the mechanical structure is sound; Phase 4b proves it works at runtime.
- Cross-cutting helpers are created in task 2, before any handler, because every write handler depends on them. This ensures the dependency order is correct.
- Sponsor-liability reputation events in `submit_jury_vote` are deferred to Phase 5 (task 55). Phase 4a only emits juror and reporter reputation events.
