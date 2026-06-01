---
name: Postgres TEXT forbids null bytes — pg_advisory_xact_lock composite key construction
description: Feedback rule — Postgres TEXT family forbids embedded null bytes; a format!("{domain}\x00{table}") key bound to pg_advisory_xact_lock fails at runtime with "invalid byte sequence for encoding UTF8: 0x00", invisible to cargo check/clippy/test --no-run; use a printable separator like ':' instead
type: feedback
---
# Postgres TEXT type forbids null bytes — advisory-lock key construction

**Context:** When constructing a key for `pg_advisory_xact_lock`, you might use a composite string like `format!("{domain}\x00{table_name}")` to separate components. Postgres `TEXT` (and all text-family types) forbids embedded null bytes at the wire level.

## The defect class

```rust
// WRONG: \x00 separator is forbidden in Postgres TEXT
diesel::dsl::sql::<diesel::sql_types::BigInt>(
    "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))"
).bind::<diesel::sql_types::Text, _>(
    format!("{peer_domain}\x00{table_name}")  // ← Postgres rejects this at runtime
)
```

**Symptom:** `ERROR: invalid byte sequence for encoding "UTF8": 0x00` at runtime, typically surfacing first in an e2e test that exercises the lock path with a real Postgres connection.

**Why compile-time gates don't catch it:** `cargo check`, `cargo clippy`, and `cargo test --no-run` have no knowledge of Postgres TEXT invariants. The error is wire-level — Postgres rejects the bind when the query executes.

## The fix

Use a printable ASCII separator that cannot appear in domain names or table names:

```rust
// CORRECT: use ':' or '/' or '|' as separator — all safe in Postgres TEXT
format!("{peer_domain}:{table_name}")
```

**Source incident:** v1-federation-inbound-e Task 1 (`acquire_evict_lock` in `inbox.rs`). Original commit used `\x00`; defect was invisible to all compile-time gates and was caught only when Task 2's e2e bound the key to `pg_advisory_xact_lock` via a Postgres TEXT bind.

## Mandatory lesson injection

**When:** any `pg_advisory_xact_lock` call site with a composite string key.  
**Check:** if the key is `format!(...)`, scan for `\x00`, `\n`, `\r` (all forbidden in TEXT).  
**Recipe:** replace with a printable separator (`:`).

## See also

- `feedback_pg_advisory_xact_lock_void_decode.md` — companion lesson for `.execute()` vs `.load()` on void PG functions.
