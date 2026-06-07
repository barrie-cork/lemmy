# Brief: m2-late-1 Task 4 — Publisher module (`sanction_publisher.rs`)

## 1. Role + dispatch

`[role:impl-task] m2-late-1-task-4-publisher — see .claude/PRPs/briefs/m2-late-1-impl-4.md`

Pre-Shape-G plan. Worker writes `validate-pending-laptop` DQ and STOP; does NOT run workspace cargo on the daemon.

## 2. Scope

**Produce:**

1. **Create** `crates/api/api/src/governance/sanction_publisher.rs` — `enqueue_sanction_event`, `seed_sanction_subscriber`, `SanctionEventPayload`, `SanctionContext`. Additive — compiles but is not yet called (T5 wires the spawn site).

2. **Modify** `crates/api/api/src/governance/mod.rs` — add `pub mod sanction_publisher;` alphabetically (after `pub mod sanction_kind_map;`, before `pub mod sponsor_liability;`).

Then write the `validate-pending-laptop` DQ entry and STOP. Do NOT run workspace cargo on the daemon.

**Explicit boundaries (do NOT touch):**
- Do NOT edit `crates/api/api/src/governance/submit_jury_vote.rs` — that is T5's file.
- Do NOT edit `crates/server/**` — T6's domain.
- Do NOT edit `services/bridge/**` — T7's domain.
- Do NOT edit `crates/server/tests/e2e.rs` or `crates/server/tests/e2e/**` — isolated to T8.
- Do NOT run `cargo check --workspace` on the daemon (R7).

## 3. Required reading

**Before your first edit, Read these files:**

- `crates/api/api/src/governance/bridge_auth.rs:1-18` — Bearer header verification pattern; the outbound POST uses `Authorization: Bearer <BRIDGE_CALLBACK_SECRET>` (same secret, outbound direction).
- `crates/api/api/src/governance/submit_jury_vote.rs:163-178` — existing `governance_case_after_transition` re-query pattern (pool connection re-acquire after tx).
- `crates/api/api/src/governance/submit_jury_vote.rs:452-480` — existing sanction emission inside the tx (provides the `Sanction` struct fields you'll need: `id`, `sanction_action`, `governance_log_entry_hash`, etc.).
- `crates/db_schema/src/source/governance/sanction_event.rs` — `SanctionEvent` + `SanctionEventInsertForm` (T2 output; your INSERT target).
- `crates/db_schema/src/source/governance/sanction_subscriber.rs` — `SanctionSubscriber` (T2 output; your SELECT source).
- `crates/api/api/src/governance/sanction_kind_map.rs` — `map_sanction_action` (T3 output; your mapping call).
- `crates/api/api/src/governance/governance_log.rs:39-70` — the pub use block (find `ENTRY_KIND_SANCTION_PUBLISHED` + `ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED` from T3).
- `.claude/PRPs/plans/m2-late.plan.md §10.3` — full `enqueue_sanction_event` body spec.
- `.claude/PRPs/plans/m2-late.plan.md §10.8` — `seed_sanction_subscriber` idempotent INSERT spec.
- `feedback_validate_pending_laptop_write_then_stop.md` — STOP after DQ entry.
- `feedback_multi_write_handlers_need_transactions.md` — NOTE: `enqueue_sanction_event` does NOT wrap in a transaction because it runs OUTSIDE the vote tx (R8/WP-2); each DB write uses a fresh pool connection per R8.

## 3a. Handover from prior cohort

T2 completed at `bf90316f0` (Diesel models + newtype). T3 completed at `6c3b212d6` (consts + shim + sanction_kind_map). Both DQ entries resolved `result: pass` at `531c51e48`. Phase tip is currently `531c51e48`.

```yaml
prior_cohort_tasks:
  - task: 2
    commit: bf90316f0
    filesCreated:
      - crates/db_schema/src/source/governance/sanction_event.rs
      - crates/db_schema/src/source/governance/sanction_subscriber.rs
    filesModified:
      - crates/db_schema/src/source/governance/mod.rs
      - crates/db_schema/src/newtypes.rs
    keyDecisions:
      - SanctionEventId newtype in newtypes.rs after SanctionId
      - SanctionSubscriber uses i32 id directly (no SanctionSubscriberId newtype)
      - governance_log_entry_hash: String in both SanctionEvent model and InsertForm
  - task: 3
    commit: 6c3b212d6
    filesCreated:
      - crates/api/api/src/governance/sanction_kind_map.rs
    filesModified:
      - crates/db_schema/src/source/governance/governance_log.rs
      - crates/api/api/src/governance/governance_log.rs
      - crates/api/api/src/governance/mod.rs
    keyDecisions:
      - Exhaustive map map_sanction_action: 8 variants, no wildcard, MuteVoice absent (plan §19)
      - pub mod sanction_kind_map already added to api governance/mod.rs
      - ENTRY_KIND_SANCTION_PUBLISHED + ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED now in both files
```

## 4. Constraints

- **R7:** Do NOT run `./scripts/brehon/cargo-check.sh` or any `cargo` command on the daemon. After committing, write `validate-pending-laptop` DQ entry then STOP immediately.
- **R8 (CRITICAL):** `enqueue_sanction_event` runs OUTSIDE any transaction. Use a FRESH pool connection for each DB operation (the vote tx has already committed when this function runs). Each DB call: `context.pool().get().await?`. Do NOT share a connection across the SELECT subscribers + INSERT sanction_event + INSERT governance_log calls.
- **Additive — not wired:** T4 creates the publisher module but nothing calls `enqueue_sanction_event` yet. T5 wires the spawn site. Do NOT add any spawn or call in T4.
- **SanctionContext:** a thin wrapper around `LemmyContext` (or the pool + env values needed). Keep it minimal — just what `enqueue_sanction_event` needs to compile. Pattern: `pub struct SanctionContext { pub pool: DbPool, pub callback_secret: String, pub callback_url: String }` OR simply `pub type SanctionContext = LemmyContext` — choose whichever compiles cleanly with the existing context plumbing in `submit_jury_vote.rs`.
- **entry_hash field:** `governance_log.entry_hash` is `Vec<u8>` on the Diesel model. Convert to hex string via `hex::encode(entry_hash)` for `SanctionEventPayload.governance_log_entry_hash`.
- **DQ commit subject:** `chore(decision-queue): impl raised validate-pending-laptop for m2-late-1 task-4`.
- **Impl commit subject:** `feat(api): add sanction_publisher module — enqueue_sanction_event + seed_sanction_subscriber (task 4)`.
- One impl commit + DQ commit then STOP. Push both commits to `origin phase-m2-late-1` immediately after each (note: `git push` via HTTPS may hang silently on the daemon due to libcurl-gnutls — the finalize-merge step will push; do NOT retry push more than once).

### Key functions to implement (verbatim from plan §10.3 + §10.8):

**`SanctionEventPayload`:**
```rust
pub struct SanctionEventPayload {
  pub sanction_kind: SanctionKind,
  pub subject_actor_pseudonym: String,
  pub effective_from: DateTime<Utc>,
  pub effective_until: Option<DateTime<Utc>>,
  pub governance_log_entry_hash: String,
}
```

**`enqueue_sanction_event(sanction, ctx)` body (plan §10.3):**
1. Call `map_sanction_action(sanction.sanction_action)` — if `None`, `tracing::debug!` + return Ok.
2. Resolve `actor_pseudonym.pseudonym` for `sanction.target_person_id` (if None, return Ok — non-person target guard).
3. Read active subscribers from `sanction_subscriber` table (`active = true`).
4. Build `SanctionEventPayload` from the sanction row.
5. For each subscriber, POST JSON payload to `subscriber.callback_url` with `Authorization: Bearer <BRIDGE_CALLBACK_SECRET>` header (from `std::env::var("BRIDGE_CALLBACK_SECRET")`). Use `reqwest::Client`.
6. Count successes. If ≥1 success: append `ENTRY_KIND_SANCTION_PUBLISHED` governance log via a NEW pool connection (R8).
7. If 0 successes (all failed): append `ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED` governance log.
8. Insert into `sanction_event` table via `SanctionEventInsertForm`.

**`seed_sanction_subscriber(url, pool)` (plan §10.8):**
```rust
// INSERT INTO sanction_subscriber (callback_url, active) VALUES ($1, true)
// ON CONFLICT (callback_url) DO NOTHING
```
Use `diesel::insert_into(sanction_subscriber::table).values(...).on_conflict(sanction_subscriber::callback_url).do_nothing().execute(conn).await?`.

**Mandatory lessons fired (file-class table match):**
- `feedback_validate_pending_laptop_write_then_stop.md` — any new file under `crates/api/api/src/governance/**`.
- `feedback_multi_write_handlers_need_transactions.md` — 2+ DB writes in a handler (NOTE: this is intentionally NOT wrapped in a transaction — R8/WP-2).
