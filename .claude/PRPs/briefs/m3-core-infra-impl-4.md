# Brief: m3-core-infra impl-task 4

## §1 Role + dispatch line

`[role:impl-task] m3-core-infra-task4-actor-pseudonym-endpoint — see .claude/PRPs/briefs/m3-core-infra-impl-4.md`

## §2 Scope

Add the Bearer-secret-authed `GET /api/v4/governance/bridge/actor-pseudonym` endpoint
(the ADR-015 allocator callsite) to `bridge_read.rs`, wire its route, and add a small
additive e2e test proving idempotency and pseudonym opacity.

**Produces:**
- `crates/api/api/src/governance/bridge_read.rs` — add `get_bridge_actor_pseudonym` handler + `BridgeActorPseudonymQuery` + `BridgeActorPseudonym` types; add `BridgeStatus.rtc_enabled` field (this was Task 1's change — verify it's already there after the pre-merge; if missing, add it here)
- `crates/api/routes/src/lib.rs` — add `get_bridge_actor_pseudonym` to import + add route after `/bridge/messaging-status`
- `crates/server/tests/e2e/governance.rs` — append one `#[tokio::test]` at end of file testing the endpoint

**Do NOT touch:** `services/bridge/**`, `migrations/**`, `docs/**`, `.claude/PRPs/plans/**`.

**Branch:** `phase-m3-core-infra` (fork from daemon-local tip).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-infra.plan.md` §10.4 (handler pattern with `get_or_create` callsite — copy verbatim)
- `crates/api/api/src/governance/bridge_read.rs:1-46` — **MIRROR for handler structure, `Data<LemmyContext>` idiom, `bridge_auth::verify_bridge_secret` pattern**
- `crates/api/api/src/governance/actor_pseudonym_helper.rs:41-80` — `get_or_create(pool, PersonId(person_id))` signature
- `crates/api/routes/src/lib.rs:44-50` (bridge_read import line 46) + `lib.rs:480-490` (route insertion point after line 485 `"/bridge/messaging-status"`)
- `crates/server/tests/e2e/governance.rs:5354-5390` — last test in file (`m2_hook_suppressed_when_messaging_disabled`; append after this)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — LemmyResult test error shape
- `.claude/lessons/feedback_async_pool_test_pattern.md` — async pool test pattern
- `.claude/lessons/feedback_cheap_model_arm_drops_adr_constraints.md` — ADR-015 load-bearing; explains why pseudonym is NOT deferrable

## §4 Constraints

### ADR-015 load-bearing (§2.4a — mandatory, not deferrable)

**Gate: this endpoint returns ONLY the opaque pseudonym — never `person_id`, username, or email.**

- **Specific gate:** `get_bridge_actor_pseudonym` calls `actor_pseudonym_helper::get_or_create(pool, PersonId(params.person_id))` → returns `Json(BridgeActorPseudonym { pseudonym })`. The `BridgeActorPseudonym` response struct has EXACTLY ONE field: `pseudonym: String`. No `person_id`, no name, no email in the response.
- **Why it cannot be deferred:** the endpoint is the seam where the bridge learns which opaque UUID to put in the LiveKit `sub`. Any real identifier leaking here propagates to the LiveKit server log and breaks the `always_pseudonym` guarantee. Once a real identity leaks into a server log, it cannot be retracted.
- **DoD line (required):** `rg 'get_or_create' crates/api/api/src/governance/bridge_read.rs` returns the callsite AND `grep -i 'person_id\|username\|email' crates/api/api/src/governance/bridge_read.rs` does NOT match any response/struct field (only the Query param `person_id: i32` is acceptable — that's the input, not the output).

### Implementation constraints

1. **Handler signature (copy from §10.4 verbatim):**
   ```rust
   pub async fn get_bridge_actor_pseudonym(
     req: HttpRequest,
     Query(params): Query<BridgeActorPseudonymQuery>,
     context: Data<LemmyContext>,
   ) -> LemmyResult<Json<BridgeActorPseudonym>> {
     bridge_auth::verify_bridge_secret(&req)?;
     let pool = &mut context.pool();
     let pseudonym =
       actor_pseudonym_helper::get_or_create(pool, PersonId(params.person_id)).await?;
     Ok(Json(BridgeActorPseudonym { pseudonym }))
   }
   ```
   `Data` is `actix_web::web::Data<LemmyContext>` — NOT `activitypub_federation::config::Data`.

2. **Types to add in `bridge_read.rs`:**
   ```rust
   #[derive(Debug, serde::Deserialize)]
   pub struct BridgeActorPseudonymQuery {
     pub person_id: i32,
   }

   #[derive(Debug, serde::Serialize)]
   pub struct BridgeActorPseudonym {
     pub pseudonym: String,
   }
   ```
   Add these after the existing `BridgeStatus` struct.

3. **Imports:** add to the `use {…}` block at the top of `bridge_read.rs`:
   - `actix_web::web::Query`
   - `lemmy_db_schema::newtypes::PersonId`
   - `super::actor_pseudonym_helper`

4. **Route in `lib.rs`:**
   - Import line 46 → add `get_bridge_actor_pseudonym` to the same `bridge_read::` import.
   - Route: `.route("/bridge/actor-pseudonym", get().to(get_bridge_actor_pseudonym))` inserted immediately after the `.route("/bridge/messaging-status", …)` line.

5. **E2e test — append after end of file (`governance.rs` line 5390):**
   ```rust
   #[tokio::test]
   async fn m3_actor_pseudonym_endpoint_idempotent_opaque() -> LemmyResult<()> {
     let context = TestContext::build(build_db_pool_for_tests()).await;
     let inserted_instance = insert_instance_test_data(context.pool()).await?;
     let person = insert_person_test_data(context.pool(), &inserted_instance).await?;

     // First call allocates a pseudonym
     let p1 = actor_pseudonym_helper::get_or_create(context.pool(), person.id).await?;
     assert!(!p1.is_empty(), "pseudonym must be non-empty");

     // Second call returns same pseudonym (idempotent)
     let p2 = actor_pseudonym_helper::get_or_create(context.pool(), person.id).await?;
     assert_eq!(p1, p2, "pseudonym must be stable across calls");

     // Pseudonym does not equal person name or local_user email
     assert_ne!(p1, person.name, "pseudonym must not equal person name");

     Ok(())
   }
   ```
   Check the sibling tests in governance.rs for the exact import names (`insert_person_test_data`, `insert_instance_test_data`, `TestContext`, `build_db_pool_for_tests`) — mirror the pattern of the nearest sibling test that seeds a person. Do NOT invent function names; read lines 5254-5390 (last two tests) for the exact idiom used in this file.

6. **Anchor uniqueness gate (pre-dispatch):** before queuing, confirm:
   - `grep -c 'get_bridge_actor_pseudonym' crates/api/api/src/governance/bridge_read.rs` → `0` (function doesn't exist yet)
   - `grep -c '"/bridge/messaging-status"' crates/api/routes/src/lib.rs` → `1` (the insertion anchor is unique)
   - `grep -c 'm3_actor_pseudonym_endpoint_idempotent_opaque' crates/server/tests/e2e/governance.rs` → `0` (test doesn't exist yet)

7. **Mid-task DQ push:** after writing the `validate-pending-laptop` DQ entry, commit + push, then **stop**. Do NOT run `cargo-check.bat` yourself — laptop advisor runs it.

### File ownership

This task touches only `crates/api/api/src/governance/bridge_read.rs`, `crates/api/routes/src/lib.rs`, and `crates/server/tests/e2e/governance.rs`. Do NOT edit `services/bridge/**`, `migrations/**`, `docs/**`.

## §5 VALIDATE (Windows)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-core-infra-task4-check.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-core-infra-task4-check.log
# EXPECT: exit 0
```

Write a `validate-pending-laptop` DQ with:
```json
{
  "kind": "validate-pending-laptop",
  "commands": [
    "cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-core-infra-task4-check.log 2>&1\""
  ],
  "branch": "phase-m3-core-infra",
  "phase_task": 4,
  "e2e_filter": "test(m3_actor_pseudonym)"
}
```

Commit + push the DQ entry, then **stop**.
