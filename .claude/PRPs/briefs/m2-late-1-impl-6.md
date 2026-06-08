# Brief: m2-late-1 Task 6 — Startup subscriber seed (`crates/server/src/lib.rs`)

## 1. Role + dispatch

`[role:impl-task] m2-late-1-task-6-startup-seed — see .claude/PRPs/briefs/m2-late-1-impl-6.md`

Pre-Shape-G plan. Worker writes `validate-pending-laptop` DQ and STOP; does NOT run workspace cargo on the daemon.

## 2. Scope

**Produce:**

1. **Modify** `crates/server/src/lib.rs` — in the async startup region of `start_lemmy_server`, after `setup_local_site` and the `context` is created, add:
   ```rust
   if let Ok(url) = std::env::var("BRIDGE_SANCTION_CALLBACK_URL") {
     lemmy_api::governance::sanction_publisher::seed_sanction_subscriber(
       &url,
       &mut (&pool).into(),
     )
     .await?;
   }
   ```
   Absent env var ⇒ no-op (clean unconfigured posture).

2. Write the `validate-pending-laptop` DQ entry and STOP. Do NOT run workspace cargo on the daemon.

**Exact insertion point:** after line 247 (`let _scheduled_tasks = tokio::task::spawn(scheduled_tasks::setup(request_data.clone()));`) and before line 250 (`let server = if !args.disable_http_server {`). Insert at the end of the async startup initialization block, before the http server starts accepting connections.

**Explicit boundaries (do NOT touch):**
- Do NOT edit `crates/api/**` — T5's domain.
- Do NOT edit `services/bridge/**` — T7's domain.
- Do NOT edit `crates/server/tests/e2e.rs` or `crates/server/tests/e2e/**` — isolated to T8.
- Do NOT run `cargo check --workspace` on the daemon.

## 3. Required reading

**Before your first edit, Read these files:**

- `crates/server/src/lib.rs:141-260` — `start_lemmy_server` async function; identify the exact insertion point (after scheduled_tasks spawn at ~line 247, before `create_http_server` at ~line 250).
- `crates/server/src/lib.rs:1-58` — existing imports; verify no `lemmy_api` import already present.
- `crates/api/api/src/governance/sanction_publisher.rs:200-220` — `seed_sanction_subscriber` signature: `pub async fn seed_sanction_subscriber(url: &str, pool: &mut DbPool<'_>) -> LemmyResult<()>`.
- `.claude/PRPs/plans/m2-late.plan.md §10.8` — verbatim seed invocation spec.
- `feedback_validate_pending_laptop_write_then_stop.md` — STOP after DQ entry.

## 3a. Handover from prior cohort

T5 completed advisor-side at laptop worktree `b25a0aaa2`. `submit_jury_vote.rs` spawn site is live. DQ `c3deadfbe610-003` resolved pass. Worker #647 was cancelled (stuck in reasoning loop).

## 4. Implementation spec (verbatim from plan §10.8)

Insert in `start_lemmy_server`, AFTER the scheduled tasks spawn and BEFORE `create_http_server`:

```rust
// Brehon B-publish: seed the sanction subscriber from env at startup.
// Absent env var ⇒ no-op; idempotent ON CONFLICT DO NOTHING (T4).
if let Ok(url) = std::env::var("BRIDGE_SANCTION_CALLBACK_URL") {
  lemmy_api::governance::sanction_publisher::seed_sanction_subscriber(
    &url,
    &mut (&pool).into(),
  )
  .await?;
}
```

**No new `use` import required** — call via full path `lemmy_api::governance::sanction_publisher::seed_sanction_subscriber`. The `pool` variable is already in scope (defined at line 181: `let pool = build_db_pool()?`). The `&mut (&pool).into()` pattern converts `ActualDbPool` → `DbPool<'_>` matching the function signature.

## 5. Constraints

- **R7:** Do NOT run `./scripts/brehon/cargo-check.sh` or any `cargo` command on the daemon. After committing, write `validate-pending-laptop` DQ entry then STOP immediately.
- **DQ commit subject:** `chore(decision-queue): impl raised validate-pending-laptop for m2-late-1 task-6`.
- **Impl commit subject:** `feat(server): seed sanction subscriber from BRIDGE_SANCTION_CALLBACK_URL at startup (task 6)`.
- One impl commit + DQ commit then STOP. Push both commits to `origin phase-m2-late-1` immediately after each. Note: `git push` via HTTPS may hang silently on the daemon — do NOT retry push more than once.

**Mandatory lessons fired:**
- `feedback_validate_pending_laptop_write_then_stop.md` — new file under `crates/server/**`.
