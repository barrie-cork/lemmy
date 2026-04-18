# Phase 5c — Task 69a fragment: V2 messaging hooks

**Purpose.** Reserve the v0-side hooks V2 messaging (post-v0) needs, per `docs/brehon-law-inspired-network/V2/messaging.md` §8. This fragment is a drop-in for the Phase 5c plan when it's written — paste it after task 69 as task 69a, or fold it into task 68 (route registration + smoke test) if you prefer to keep the task count at 9.

**Source:** V2/messaging.md §8.1 (PM plugin hooks — covered by `.claude/rules/pm-plugin-hooks-stable.md`, no task needed), §8.2 (underscore-prefix usernames), §8.3 (SSE-over-WebSocket preference), §8.4 (governance state-transition event stream).

**Scope.** One new migration, one new doc, two new e2e tests, one plan-doc annotation. No Rust handler changes. No new endpoint. No new route.

---

## Task 69a — V2 messaging hooks (compound task)

Adds four small things that keep the V2 door open without expanding v0 scope:

### 69a.1 — Postgres LISTEN/NOTIFY trigger on `governance_log`

**File:** new migration `crates/db_schema/migrations/{timestamp}_add_governance_log_notify/up.sql` + `down.sql`.

**Up SQL:**
```sql
CREATE OR REPLACE FUNCTION governance_log_notify() RETURNS TRIGGER AS $$
BEGIN
  PERFORM pg_notify(
    'governance_events',
    json_build_object(
      'entry_id', NEW.id,
      'kind', NEW.entry_kind,
      'published_at', NEW.published_at
    )::text
  );
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER governance_log_notify_trigger
  AFTER INSERT ON governance_log
  FOR EACH ROW
  EXECUTE FUNCTION governance_log_notify();
```

**Down SQL:**
```sql
DROP TRIGGER IF EXISTS governance_log_notify_trigger ON governance_log;
DROP FUNCTION IF EXISTS governance_log_notify();
```

**Rationale.** V2 messaging (§8.4 of V2/messaging.md) requires governance state transitions to be observable by an out-of-process bridge without polling. Postgres `LISTEN/NOTIFY` gives exactly that: the V2 bridge connects to the same database, issues `LISTEN governance_events`, and receives one notification per inserted log row. The hash-chain trigger already on `governance_log` is unaffected — NOTIFY runs after the hash-chain trigger (both fire at row-insert time, but NOTIFY doesn't read `prev_hash`).

**GOTCHA.** The payload is capped at 8KB by Postgres; we deliberately keep it to three fields (entry_id, kind, published_at). Subscribers that need the full row issue a follow-up `SELECT ... WHERE id = $1`. Do NOT add the payload JSONB — it can exceed 8KB for large sanction or report entries.

**GOTCHA.** NOTIFY is ephemeral. A subscriber that isn't LISTENing at the moment of insert misses the notification. The V2 bridge must implement catch-up on startup via `SELECT id, entry_kind, published_at FROM governance_log WHERE id > $last_seen_id ORDER BY id ASC`. Document this in SUBSCRIPTIONS.md below.

**GOTCHA.** Channel names are case-folded by Postgres unless double-quoted. Use `governance_events` (all lowercase) everywhere so LISTENers on both sides match. Document the channel name as a stability contract in SUBSCRIPTIONS.md.

### 69a.2 — SUBSCRIPTIONS.md documentation

**File:** new `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md`.

**Content skeleton:**

```markdown
# Governance event subscriptions

Brehon emits a Postgres `NOTIFY` on the channel `governance_events`
for every row inserted into `governance_log`. Subscribers that want
real-time awareness of governance activity connect to the database
and issue `LISTEN governance_events`.

This is an out-of-process contract — the V2 messaging bridge (when
built), external observability tools, and any admin dashboard reading
governance activity all depend on it. Breaking the channel name or
payload shape is a breaking change for downstream consumers.

## Channel

- **Name:** `governance_events` (lowercase, no quoting).
- **Payload:** JSON object with three fields.

## Payload shape

| Field | Type | Description |
|---|---|---|
| `entry_id` | i64 | `governance_log.id` — the row that triggered the notify |
| `kind` | string | `governance_log.entry_kind` — one of the constants in `crates/api/api/src/governance/governance_log.rs` |
| `published_at` | ISO-8601 timestamp | `governance_log.published_at` |

Why so small: Postgres NOTIFY caps payloads at ~8KB. For the full row,
subscribers issue `SELECT * FROM governance_log WHERE id = $entry_id`.

## Catch-up on subscriber start

Notifications are ephemeral. A subscriber missing a notification (e.g.
during startup, restart, or network blip) must catch up via:

```sql
SELECT id, entry_kind, published_at, actor_pseudonym, payload
FROM governance_log
WHERE id > $last_seen_id
ORDER BY id ASC;
```

Subscribers are expected to persist `last_seen_id` across restarts
(e.g. in their own local state). `governance_log.id` is monotonic.

## Entry kinds

The `kind` field takes values enumerated in
`crates/api/api/src/governance/governance_log.rs` as `pub const`s.
As of v0 the full list is (pseudonymised where noted in the payload
JSON):

- `report_created`
- `threshold_met`
- `jury_assigned`
- `jury_voted`
- `sanction_created`
- `public_log_published`
- `reputation_delta`
- `capability_changed`
- `case_decided`
- `sponsor_liability_applied`
- `sponsor_liability_clamped`
- `founder_seeded`
- `endorsement_created`

New kinds are added in future phases; subscribers should treat
unknown kinds as forward-compatible (ignore, don't crash).

## Not a transport

This is an at-least-once notification channel, not a transport. Do
not use it for message content. Use it only to learn *that* a
governance event happened, then fetch details via a normal query.
```

**Rationale.** SUBSCRIPTIONS.md is the stable contract between Brehon and any future observer. Channel rename = breaking change = major version bump. Payload shape change = same.

### 69a.3 — e2e test: `governance_events_notify_fires`

**File:** add to `tests/e2e.rs`.

```rust
#[tokio::test]
async fn governance_events_notify_fires() -> Result<(), Box<dyn std::error::Error>> {
    // Spin up a Postgres connection dedicated to LISTENing (must be
    // separate from the pool — LISTEN is session-scoped).
    let listener_conn = /* direct tokio-postgres Client, not deadpool */;
    listener_conn.batch_execute("LISTEN governance_events").await?;

    // In another connection, insert a governance_log row via a
    // normal handler path — e.g. call create_report with minimal
    // fixture data so `report_created` lands.
    let (context, _test_server) = boot_test_server().await?;
    let reporter = create_test_user(&context, "reporter_alice").await?;
    let target_post = create_test_post(&context, ...).await?;
    call_create_report(&context, &reporter, target_post, "spam").await?;

    // Assert a notification arrives within 1 second.
    let notification = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        listener_conn.notifications().next(),
    ).await??;

    let payload: serde_json::Value = serde_json::from_str(notification.payload())?;
    assert_eq!(payload["kind"], "report_created");
    assert!(payload["entry_id"].is_i64());
    assert!(payload["published_at"].is_string());

    Ok(())
}
```

**GOTCHA.** The exact `tokio-postgres` LISTEN API varies — Diesel's connection wrapper doesn't expose NOTIFY; the test needs a raw `tokio_postgres::Client` or the analogous `sqlx::postgres::PgListener`. Either is fine — pick what's already in the dev-dependencies. If neither exists, `sqlx` is cheaper to add as a dev-dep than a runtime dep of the server.

**GOTCHA.** The listener connection must LISTEN **before** the insert happens, or the test races. The sequence in the test is: LISTEN → insert → await notification (with timeout).

**GOTCHA.** Use the same test pattern as Phase 4/5's e2e tests — `Result<(), Box<dyn Error>>`, `.map_err` for LemmyError bridging, `--user $(id -u):$(id -g)` on Docker (per memory feedback and `advisor-context-phase-5.md`). No `.unwrap()`.

### 69a.4 — e2e test: `underscore_prefix_usernames_still_register`

**File:** add to `tests/e2e.rs`.

```rust
#[tokio::test]
async fn underscore_prefix_usernames_still_register() -> Result<(), Box<dyn std::error::Error>> {
    // Guards V2 §8.2: @_-prefixed usernames must remain registerable
    // because the V2 Matrix bridge will advertise MXIDs like
    // @_lemmy_alice:matrix.example.com on the Lemmy side.
    let (context, _server) = boot_test_server().await?;

    let username = "_lemmy_test_user";
    let person = create_test_user(&context, username).await?;
    assert_eq!(person.name, username);
    Ok(())
}
```

**Rationale.** Cheap regression test. If an upstream Lemmy rebase introduces a username validator that rejects leading underscores, this test fails loudly and we know before V2 planning starts.

**GOTCHA.** Lemmy's current username validator lives in `crates/utils/src/utils/validation.rs` (as of 1.0-beta). If the test fails after a rebase, that's likely where the rule was introduced. Patch the validator, don't rename the test.

### 69a.5 — Plan-doc annotation: SSE over WebSocket preference

**File:** edit `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`.

Add a single paragraph to the "What NOT to build in v0" section (or wherever §6 / post-v0 discussion of real-time transport would land):

> **Real-time transport (if proposed post-v0):** if a future phase proposes adding a real-time push channel for governance notifications (jury invitations, case status changes, etc.), default to Server-Sent Events (SSE) over WebSocket. SSE composes with HTTP caching, has simpler backpressure semantics, and works through the same middleware stack as the existing API. The V2 messaging bridge does not require Brehon's own RT transport — it polls `governance_log` or subscribes via Postgres NOTIFY per `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md`. See V2/messaging.md §8.3 for the full reasoning.

**Rationale.** Documentation only. Reserves the architectural choice without committing to a phase.

---

## Task 69a definition of done

- Migration `add_governance_log_notify` applies cleanly and inserts a trigger that fires on every `governance_log` insert. Reversible via `down.sql`.
- `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md` exists and documents channel name, payload shape, catch-up procedure, and the full list of entry kinds (v0 era).
- `tests/e2e.rs::governance_events_notify_fires` passes — creating a report triggers a `governance_events` notification carrying the `report_created` kind within 1 second.
- `tests/e2e.rs::underscore_prefix_usernames_still_register` passes.
- `IMPLEMENTATION-PLAN-v0.md` has an SSE-over-WebSocket paragraph in the right section.
- `cargo check --features full --workspace --no-deps` clean.
- `cargo clippy --features full --workspace --no-deps -- -D warnings` clean.
- No regression in Phase 4 `report_to_modlog_golden_path`, Phase 5b `sponsor_liability_with_founder_multiplier` (both branches plus `honour_price_floor_clamp`), or Phase 5a job-tick assertions.
- All existing e2e tests still pass.

---

## Out of scope for task 69a

- **Subscriber implementation.** v0 does not ship a LISTENer. That's V2 work.
- **Payload schema versioning.** The three fields (entry_id, kind, published_at) are v1; if we ever need to evolve the shape, that's a new major version of SUBSCRIPTIONS.md and a new channel name (e.g. `governance_events_v2`).
- **Batching / debouncing.** NOTIFY fires once per insert. If a future high-volume scenario needs batching, that's a separate infrastructure decision — not this task.
- **Extism hook equivalent.** V2/messaging.md §8.4 mentions either a Postgres NOTIFY or an Extism hook as alternatives. This task implements the NOTIFY option. The Extism option is NOT needed if NOTIFY works; adding both would be redundant.

---

## Why this fragment, not a full task integration

The Phase 5c plan doesn't exist in `.claude/PRPs/plans/` yet. This fragment is structured so that when `/prp-plan "Phase 5c"` runs, the author pastes this in as task 69a with minimal edits. Alternatively, the NOTIFY trigger + SUBSCRIPTIONS.md could fold into task 68 (route registration + smoke test) and the two e2e tests could fold into task 69 (capability-gating e2e) — both are legitimate organisational choices.

## Commit hygiene

If landing as its own commit, use:

```
feat(governance): task 69a — Postgres NOTIFY on governance_log + V2 hooks

Reserves V2 messaging (post-v0) hooks per docs/brehon-law-inspired-network/V2/messaging.md §8:
- governance_log NOTIFY trigger (§8.4)
- SUBSCRIPTIONS.md channel contract
- e2e: governance_events_notify_fires
- e2e: underscore_prefix_usernames_still_register (§8.2)
- IMPLEMENTATION-PLAN-v0.md SSE-over-WS preference (§8.3)

No Rust handler changes. No new endpoint. Reversible migration.
```

## References

- `docs/brehon-law-inspired-network/V2/messaging.md` §8 — the four V2 hooks
- `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §3 Phase 5c — task numbering context
- `.claude/rules/pm-plugin-hooks-stable.md` — the §8.1 rule (already landed)
- `crates/api/api/src/governance/governance_log.rs` — entry-kind constants
- `crates/db_schema/migrations/` — migration directory layout
- Phase 5a migration `add_governance_config` — reference for migration commit style
