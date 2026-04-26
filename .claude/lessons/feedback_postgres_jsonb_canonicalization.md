---
name: Postgres JSONB text rendering vs serde_json compact output
description: Postgres `::text` on jsonb writes `{"n": 1}` with a space after the colon; serde_json compact writes `{"n":1}` without. When hashing or comparing byte sequences, use PG's own rendering via raw SQL rather than round-tripping through serde_json
type: feedback
originSessionId: 98b0fb05-df8a-4d08-a152-27b3f01323e4
---
Postgres's `jsonb::text` cast produces `{"n": 1}` — space after the colon, space after commas inside objects. `serde_json::to_string(&value)` (compact mode) produces `{"n":1}` — no spaces. The byte-level difference is exactly one byte per object key-value pair and absolutely breaks any sha256 / hmac / equality check that assumes the two renderings match.

**Why**: Phase 1 task 14 (`governance_log_hash_chain_holds`) first attempt recomputed `sha256(prev || kind || payload || ts)` in Rust by reading back the governance_log row via the Diesel model (giving a `serde_json::Value`) and re-serialising it through `serde_json::to_string`. The hash mismatched on row 1 because the Postgres-side trigger had hashed the `{"n": 1}` rendering and Rust was hashing `{"n":1}`. Caught by running the test, narrowed down by querying `SELECT payload::text` against the live container and eyeballing the byte sequence.

**How to apply**:
- When recomputing a hash/digest/signature in Rust to verify something Postgres produced, read the exact Postgres-side bytes via raw `sql_query`, NOT via the Diesel model + re-serialisation. See `phase1_migrations_round_trip` and `governance_log_hash_chain_holds` in `crates/server/tests/e2e.rs` for the pattern: a `#[derive(QueryableByName)]` struct with `#[diesel(sql_type = Text)]` fields for `payload::text` and `to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"')`.
- Phase 4's `governance_log::append` helper will need to canonicalise payloads at insert time so the trigger's byte sequence is deterministic regardless of input format. This is also where to solve the "what if the caller passes nested JSON with unstable key order" problem — the answer is to canonicalise with a sorted BTreeMap-backed JSON serialiser before calling into diesel. Until that helper exists, tests should always read back the PG rendering directly.
- Timestamp formatting has the same class-of-bug: Postgres `to_char(... 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"')` and Rust `chrono::DateTime::<Utc>::to_rfc3339_opts(SecondsFormat::Micros, true)` *usually* produce the same string, but prefer reading the PG rendering back via raw SQL rather than re-formatting in Rust — the tolerance for mismatch is zero in hash-chain contexts.
- This is called out in Phase 1 plan §4 "Open Questions" as OQ-4 (JSONB canonicalisation). The decision was "defer to Phase 4's append helper" — this memory is the interim mitigation.
