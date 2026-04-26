---
name: Multi-write handlers must use run_transaction
description: Any handler with 2+ Diesel writes must wrap them in conn.run_transaction(). Pattern at crates/api/api/src/community/ban.rs:59-64.
type: feedback
originSessionId: e8d9be45-74d4-44e0-aba4-2baa51128e3f
---
When a handler performs multiple DB writes that must succeed or fail atomically, use the Lemmy `run_transaction` pattern:

```rust
use lemmy_diesel_utils::connection::get_conn;
use futures::FutureExt; // for .boxed()

let pool = &mut context.pool();
let conn = &mut get_conn(pool).await?;
conn.run_transaction(|conn| {
    async move { /* all writes here */ }.boxed()
}).await?;
```

**Why:** Advisor correction C1 on Phase 4a plan. submit_jury_vote has 5+ writes (vote, assignment update, sanction, case update, public_case_log, reputation_events, governance_log entries). Without a transaction, a failure mid-flow leaves the database in an inconsistent state (e.g., vote recorded but decision not). Individual `context.pool()` calls per insert do NOT share a transaction.

**How to apply:** When planning or implementing any governance handler that does more than one DB write, specify `run_transaction` in the plan and use the pattern above. Auth checks and read-only validation go BEFORE the transaction.
