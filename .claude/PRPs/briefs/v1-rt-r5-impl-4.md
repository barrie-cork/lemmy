# Brief: v1-RT-r5 Task 4 — admin rollup endpoint (handler + mod + route)

## 1. Role + dispatch line

`[role:impl-task]` v1-rt-r5-task-4-admin-reputation-rollup-endpoint — see `.claude/PRPs/briefs/v1-rt-r5-impl-4.md`

## 2. Scope

**Produce:** create the `admin_reputation_rollup` handler and wire it into the module tree and route table.

**Exact deliverables (3 files):**

**File 1 (CREATE):** `crates/api/api/src/governance/admin_reputation_rollup.rs`
- Async handler function `pub async fn admin_reputation_rollup(...)` mirroring `admin_reputation_stats.rs:71-82`
- Query-parameter GET: `Query<AdminReputationRollup>` (not JSON body)
- `is_admin(&local_user_view)?` as FIRST line of handler body
- Load `rollup` row: `reputation_snapshot WHERE person_id=$1 AND community_id IS NULL` → `Option<ReputationSnapshot>` via `.first().await.optional()?` or similar
- Load `contributing` rows: `reputation_snapshot WHERE person_id=$1 AND community_id IS NOT NULL` → `Vec<ReputationSnapshot>` via `.load(conn).await?`
- Return `Ok(Json(AdminReputationRollupResponse { rollup, contributing }))`

**File 2 (MODIFY):** `crates/api/api/src/governance/mod.rs`
- Add `pub mod admin_reputation_rollup;` in alphabetical order (BEFORE `pub mod admin_reputation_stats;` at :23)

**File 3 (MODIFY):** `crates/api/routes/src/lib.rs`
- Add `admin_reputation_rollup::admin_reputation_rollup,` import near :40 (alphabetical in governance imports)
- Add `.route("/reputation/rollup", get().to(admin_reputation_rollup))` in the admin scope near :502

**One commit covering all 3 files.**
**One `validate-pending-laptop` DQ entry then STOP — do NOT run cargo yourself.**

**Do NOT touch:** `crates/routes/**`, `crates/db_schema/**`, `crates/api/api_common/**`, `crates/server/tests/e2e.rs`, any migration, any plan or PRD file.

## 3. Required reading

- `.claude/PRPs/plans/v1-RT-r5.plan.md` §10.5 (admin handler head with full code snippet)
- `.claude/PRPs/plans/v1-RT-r5.plan.md` §13 Task 4 (ACTION, FILES, IMPLEMENT, MIRROR, GOTCHA, VALIDATE)
- `.claude/lessons/feedback_features_full_workspace_only.md`
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md`
- `.claude/lessons/feedback_clippy_test_style.md`

**Mandatory MIRROR reads (read BEFORE writing):**
- `crates/api/api/src/governance/admin_reputation_stats.rs:24-25,71-82` — imports + handler head (exact pattern to copy: `use crate::governance::config`, `use actix_web::web`, `is_admin` first line, `Query<...>`, pool + conn setup)
- `crates/api/api/src/governance/mod.rs:22-25` — existing module declarations (insertion point)
- `crates/api/routes/src/lib.rs:38-44` — existing governance imports (insertion point near :40)
- `crates/api/routes/src/lib.rs:498-508` — existing admin route registrations (insertion point near :502)
- `crates/api/api_common/src/governance.rs` (last ~30 lines) — confirm `AdminReputationRollup` + `AdminReputationRollupResponse` are already present (Task 2 added them); use their names verbatim

## 4. Constraints

- **validate-pending-laptop DQ discipline:** after committing all 3 files in one commit, write a `kind: "validate-pending-laptop"` DQ entry with `commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`; commit + push the DQ change; then **STOP**. Do NOT run cargo-check yourself.
- **Route path GOTCHA:** path is `/reputation/rollup` (slash-separated, matching PRD §5.5), NOT `/reputation-rollup` (the stats sibling uses hyphens — this divergence is intentional per plan §13 Task 4). File a `kind: "log"` DQ noting the convention divergence.
- **WRONG crate GOTCHA:** `crates/api/routes/src/lib.rs` is `lemmy_api_routes` — this is DIFFERENT from `crates/routes/` (`lemmy_routes`). Edit `crates/api/routes/src/lib.rs` NOT `crates/routes/src/lib.rs`.
- **Alphabetical mod.rs insertion:** `pub mod admin_reputation_rollup;` must come BEFORE `pub mod admin_reputation_stats;` — it is alphabetically earlier.
- **Single commit, all 3 files.** Commit message: `feat(reputation): Task 4 — admin_reputation_rollup handler + route (v1-RT-r5)`.
- **DQ attribution:** `from: "impl"`, `answered_by: null`. Never write `answered_by: "advisor"`.
- **Mid-task push:** push the DQ commit immediately after writing it.
