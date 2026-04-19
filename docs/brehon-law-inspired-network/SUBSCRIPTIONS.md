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

### Id gaps under rollback are benign

`governance_log.id` is monotonic but **not contiguous**. Postgres does not
roll back a sequence advance when its parent transaction aborts, so a
committed row may be preceded by gaps where rolled-back inserts consumed
ids that never reached commit. Subscribers must distinguish three cases:

- **Id gap with no corresponding NOTIFY** — benign. The missing ids
  belong to rolled-back transactions (or sequence-cache loss across
  Postgres restart). Do not retry; do not rescan.
- **NOTIFY received whose `entry_id` is not yet visible in
  `governance_log`** — retry. Trigger-vs-commit timing means the row
  may not be visible to the subscriber's snapshot for a few
  milliseconds. Re-issue the row fetch; do not treat as missing.
- **NOTIFY-less catch-up on subscriber start / reconnect** — required.
  Use the `last_seen_id` cursor query above, not gap detection. NOTIFY
  is at-most-once and dropped on disconnect.

## Entry kinds

The `kind` field takes values enumerated as `pub const` strings in
`crates/api/api/src/governance/governance_log.rs` (and the Phase 5b/5c
governance handler modules that extend the list). As of v0 the full
set is (pseudonymised where noted in the payload JSON):

- `report_created`
- `threshold_met`
- `jury_assigned`
- `panel_assembled`
- `jury_replacement_selected`
- `jury_accepted`
- `jury_declined`
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
- `federation_sanction_sent` — local instance published an outbound `PublishSanctionNotice` activity for a finalised case with `FederatedRecommendation` scope
- `federation_sanction_received` — local instance recorded an inbound `PublishSanctionNotice` from a remote instance into `remote_sanction_notice` (advisory only, not auto-applied per ADR-006)
- `federation_attestation_sent` — local instance published an outbound `PublishTrustAttestation` activity (plumbed in v0; no v0 endpoint emits this yet)
- `federation_attestation_received` — local instance recorded an inbound `PublishTrustAttestation` from a remote instance into `federation_attestation` (no inbound enforcement in v0)

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

This is an at-most-once notification channel, not a transport. Messages
may be dropped on subscriber disconnect or queue overflow. Subscribers
must use the last_seen_id catch-up query to backfill missed events.
Do not use it for message content. Use it only to learn *that* a
governance event happened, then fetch details via a normal query
against `governance_log`.

## References

- `docs/brehon-law-inspired-network/V2/messaging.md` §8.4 — rationale
- `migrations/2026-04-20-000000-0000_add_governance_log_notify/up.sql` — trigger source
- `crates/server/tests/e2e.rs::governance_events_notify_fires` — regression test
