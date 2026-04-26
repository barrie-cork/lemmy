---
name: pg_advisory_xact_lock returns void — never .load(), use .execute()
description: Postgres advisory lock functions return void; Diesel's .load::<BigInt>() fails with "Received less than 8 bytes while decoding an i64"
type: feedback
originSessionId: bae44467-63e7-4c21-ad41-057cefebb80e
---
`pg_advisory_xact_lock(key)` returns `void` in Postgres. Writing:

```rust
sql_query("SELECT pg_advisory_xact_lock($1)")
    .bind::<BigInt, _>(key)
    .load::<IgnoredRow>(conn).await?;  // <- WRONG
```

...fails at runtime with: `Received less than 8 bytes while decoding an i64. Was an Integer expression accidentally marked as BigInt?`

The fix is `.execute(conn)` — statement, no result-set decode:

```rust
sql_query("SELECT pg_advisory_xact_lock($1)")
    .bind::<BigInt, _>(key)
    .execute(conn).await?;  // CORRECT
```

**Why:** Diesel's `.load::<T>()` tries to decode every returned column into the target type. For `void`, Postgres returns a zero-byte column; Diesel's `BigInt`/`Integer` decoders see <8 bytes and panic. `.execute()` ignores any result set and only tracks rows-affected, which is what pg_advisory_xact_lock actually wants.

**How to apply:**
- Any Postgres function returning `void` (pg_advisory_lock, pg_advisory_xact_lock, pg_notify, pg_advisory_unlock, etc.) → `.execute()` not `.load()`
- Never "alias as _lock_key" to paper over it — that's the exact pattern that breaks
- Same rule applies to `PERFORM` calls wrapped in `SELECT ... FROM pg_...`
- Latent bug discovered 2026-04-19: Phase 5a's `reputation_snapshot::acquire_advisory_xact_lock` had this bug from task 54; no 5a/5b e2e drove it end-to-end, Phase 5c task 69's `run_snapshot_batch` was the first to surface it

Related: `feedback_multi_write_handlers_need_transactions.md` — advisory-lock-per-tx pattern is the standard here.
