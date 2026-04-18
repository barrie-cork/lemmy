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
| `created_at` | ISO-8601 timestamp | `governance_log.created_at` |

Why so small: Postgres NOTIFY caps payloads at ~8KB. For the full row,
subscribers issue `SELECT * FROM governance_log WHERE id = $entry_id`.

## Catch-up on subscriber start

Notifications are ephemeral. A subscriber missing a notification (e.g.
during startup, restart, or network blip) must catch up via:

```sql
SELECT id, entry_kind, created_at, actor_pseudonym, payload
FROM governance_log
WHERE id > $last_seen_id
ORDER BY id ASC;
```

Subscribers are expected to persist `last_seen_id` across restarts
(e.g. in their own local state). `governance_log.id` is monotonic.

## Entry kinds

The `kind` field takes values enumerated as `pub const` strings in
`crates/api/api/src/governance/governance_log.rs` (and the Phase 5b/5c
governance handler modules that extend the list). As of v0 the full
set is (pseudonymised where noted in the payload JSON):

- `report_created`
- `threshold_met`
- `jury_assigned`
- `jury_replacement_selected`
- `jury_assignment_accepted`
- `jury_assignment_declined`
- `jury_voted`
- `case_decided`
- `appeal_requested`
- `sanction_created`
- `public_log_published`
- `reputation_delta`
- `capability_changed`
- `sponsor_liability_applied`
- `sponsor_liability_clamped`
- `founder_seeded`
- `endorsement_created`
- `emergency_removed`

New kinds will be added in future phases; subscribers should treat
unknown kinds as forward-compatible (ignore, don't crash).

## Stability contract

The three items below form the public contract for subscribers. A change
to any of them is a breaking change and must be coordinated with V2
messaging (and any external observer that has LISTENed to the channel):

1. Channel name: `governance_events` (lowercase).
2. Payload shape: `{entry_id, kind, created_at}`.
3. Entry-kind strings: existing values are stable; new values may be added.

## Not a transport

This is an at-least-once notification channel, not a transport. Do
not use it for message content. Use it only to learn *that* a
governance event happened, then fetch details via a normal query
against `governance_log`.

## References

- `docs/brehon-law-inspired-network/V2/messaging.md` §8.4 — rationale
- `migrations/2026-04-20-000000-0000_add_governance_log_notify/up.sql` — trigger source
- `crates/server/tests/e2e.rs::governance_events_notify_fires` — regression test
