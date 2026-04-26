---
name: Async pool pattern for e2e tests
description: How to bridge sync seed data and async view-crate queries in crates/server/tests/e2e.rs — use AsyncPgConnection::establish + DbPool::Conn, not deadpool/bb8
type: feedback
originSessionId: 11591b2e-3ef6-46b3-82b9-0456890eb1aa
---
When writing integration tests in `crates/server/tests/e2e.rs` that call governance view-crate impls (which take `&mut DbPool<'_>`), use this pattern:

1. Seed data via sync `PgConnection::establish(&db_url)` + `diesel::insert_into(...).execute(&mut sync_conn)` (existing Phase 1 pattern)
2. Build async connection: `let mut async_conn = AsyncPgConnection::establish(&db_url).await?;`
3. Convert to pool: `let mut pool: DbPool<'_> = (&mut async_conn).into();`
4. Call view crate: `let rows = some_view_fn(&mut pool).await.map_err(|e| format!("{e}"))?;`

**Why:** `build_db_pool()` reads `SETTINGS.get_database_url_with_options()` and runs migrations — it doesn't point at testcontainer URLs. `AsyncPgConnection::establish` works directly with plain `postgres://` URLs (no TLS setup needed). The `From<&mut AsyncPgConnection> for DbPool<'_>` impl at `connection.rs:104` wraps it as `DbPool::Conn`.

**How to apply:** Use this pattern in any future Phase 3/4 integration tests. No deadpool/bb8 needed. The `.map_err(|e| format!("{e}").into())?` bridge handles `LemmyError` → `Box<dyn Error>` because `LemmyError` doesn't impl `std::error::Error`.
