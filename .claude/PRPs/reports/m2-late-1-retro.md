# m2-late-1 Retro — B-publish Sanction Propagation

**Phase:** m2-late-1  
**PR:** #192 → `governance-v0`  
**Merge commit:** `be8134d0b` @ 2026-06-08T07:21:56Z  
**Phase branch tip (pre-merge):** `991e35de5`  
**Wall-clock:** ~2 sessions (2026-06-07 start → 2026-06-08 merge)

---

## What shipped

- `crates/db_schema/src/source/governance/sanction_event.rs` — new `SanctionEvent` model + `SanctionEventInsertForm`
- `crates/db_schema/src/source/governance/sanction_subscriber.rs` — new `SanctionSubscriber` model + `SanctionSubscriberInsertForm`
- `crates/db_schema_file/src/schema.rs` — hand-extended with `sanction_event` + `sanction_subscriber` tables and `SanctionKind` enum (Task 1)
- `crates/api/api/src/governance/sanction_kind_map.rs` — `SanctionAction` → `Option<SanctionKind>` mapping (Task 3)
- `crates/api/api/src/governance/sanction_publisher.rs` — `enqueue_sanction_event` fire-and-forget publisher; `seed_sanction_subscriber` idempotent seed (Task 4)
- `crates/api/api/src/governance/submit_jury_vote.rs` — `tokio::spawn` of `enqueue_sanction_event` gated on `outcome.case_decided` (Task 5)
- `crates/server/src/lib.rs` — `BRIDGE_SANCTION_CALLBACK_URL` startup seed (Task 6)
- `services/bridge/src/sanction_handler.rs` — POST `/brehon/sanction-event` handler with Bearer auth + `sanction_kind_to_power_level` stub (Task 7)
- `services/bridge/src/appservice.rs` + `services/bridge/src/main.rs` — route registration outside `hs_token_auth` layer (Task 7)
- `crates/server/tests/e2e/m2_late.rs` — `sanction_event_delivered_to_subscriber` e2e test with mock HTTP subscriber (Task 8)
- `migrations/2026-06-07-000000-0000_add_sanction_event/` — `up.sql` + `down.sql` with `sanction_event` + `sanction_subscriber` tables and CHECK constraint (Task 1 + CR-8 fix)
- `.claude/rules/governance-log-entry-kind-registry.md` — m2-late-1 section (2 consts: `sanction_published` + `sanction_event_delivery_failed`)

**CR fix-in-pr (10 findings addressed):**
- CR-2: `seed_sanction_subscriber` `do_nothing()` → `do_update().set(active.eq(true))`
- CR-3: `unwrap_or_default()` → explicit `Err` on missing governance_log row
- CR-4: over-voting guard — `enqueue_sanction_event` spawn gated on `outcome.case_decided`
- CR-5: `BRIDGE_SANCTION_CALLBACK_URL` trim + empty check before seeding
- CR-7: `multi_thread` → `current_thread` tokio test flavor (UB with `set_var`)
- CR-8: CHECK constraint `effective_until IS NULL OR effective_until > effective_from`
- CR-9(a): `applied: true` → `applied: false` (stub correctly signals not-yet-applied)
- CR-9(b): extend `sanction_kind_to_power_level` with `prevent_post`/`mute_voice` → 0, `hide_content`/`restrict_reach` → 25
- CR-B: reqwest `Client::new()` → builder with `connect_timeout(10s)` + `timeout(30s)`
- CR-C: doc comment updated to reflect 3-tier power-level mapping

**Carry-forward (1):**
- CR-A: atomicity gap between `sanction_event` INSERT and `governance_log::append` — accepted for m2-late-1 (R8: fire-and-forget outside vote transaction; deferred to m2-late-2)

---

## What surprised us

### Advisor role
- The `outcome.case_decided` over-voting guard (CR-4) was a genuine correctness bug, not a nitpick. Votes 4 and 5 each re-queried and found the active sanction, spawning `enqueue_sanction_event` twice. CR caught it; the fix was load-bearing.
- CodeRabbit's round-2 "outside diff" comments (CR-A, CR-B, CR-C) arrived before fix-impl-1 had landed on the PR — CR was reviewing the merge-forward commit's diff, not the fix-impl-1 diff. This caused a confusing timeline where CR-B appeared "new" but was actually a re-finding on code that was already being fixed in the inflight Junior task.
- CR did not post a round-3 review after fix-impl-2 was pushed (~17 min wait, no review). This is a known CodeRabbit behaviour on small incremental pushes. Proceeding to gate 5 without round-3 was correct given all material findings were addressed.

### Planning role
- The plan correctly scoped T1 as a hand-edit task (schema.rs is not diesel-generated here) and T3–T8 as additive impl tasks. No plan deviations needed.

### Impl role
- Junior task #648 (fix-impl-1) ran for 47 min and `updatedAt` did not advance during that time — appeared hung but was actively streaming thinking tokens (confirmed via SSH log tail). The `updatedAt` field in the daemon DB is not a reliable activity indicator.
- The `current_thread` tokio fix (CR-7) was subtle: `std::env::set_var` is UB in multi-threaded context; the test mock server uses `set_var` for the `BRIDGE_CALLBACK_SECRET`. This is a real correctness fix, not a style nit.
- Migration ENUM values needed `PascalCase` (`Ban`, `Mute`, etc.) to match `DbValueStyle::Verbatim` in `db_schema_file`. Caught during T8 e2e fix iterations.
- The e2e test required stopping voting at quorum to avoid multiple `sanction_event` rows (a natural consequence of the over-voting bug that CR-4 fixed in prod code; the test needed its own guard).

### BM role
- No BM anomalies. bm-cut, bm-pr, and merge all executed cleanly.

---

## What to change

1. **Brief the `updatedAt` unreliability earlier.** When polling a Junior task that appears hung, SSH log tail should be the first diagnostic (not a last resort). The lesson `feedback_junior_task_updated_at_unreliable.md` should be added and injected for long-running impl-task briefs.

2. **CR "outside diff" timing confusion.** When a merge-forward commit lands on a PR, CR re-reviews the new diff which may not yet include an in-flight fix-impl. The triage should note whether each CR finding was posted against pre- or post-fix code. The `pr-findings.yaml` `addressed_in` field handles this for the YAML but the triage surface to the user should make it explicit.

3. **Reqwest timeout as a default pattern.** Any Brehon code that creates a `reqwest::Client` for outbound webhook/callback delivery should default to `Client::builder().connect_timeout(10s).timeout(30s)`. Add a lesson so future impl-task briefs inject it for HTTP client creation in governance handlers.

4. **`validate-pending-laptop` scope for bridge crates.** Fix-impl-1 touched both `crates/` and `services/bridge/` but the DQ entry only listed `cargo check --workspace --features full`. Bridge is workspace-excluded (R9); a separate `cargo check` scoped to `services/bridge` would have been cleaner validation. The laptop handler should check for bridge-touching diffs and add a bridge-scoped command.

---

## What to carry forward

1. **CR-A atomicity gap** (sanction_event INSERT + governance_log::append on separate pool connections) — deferred to m2-late-2. The R8 constraint prevents wrapping in the vote transaction; m2-late-2 should consider a helper that atomically inserts both rows in a fresh transaction within `enqueue_sanction_event` itself.

2. **Power-level enforcement stub** — `sanction_kind_to_power_level` returns correct values but `handle_sanction_event` returns `applied: false`. Full Matrix power-level enforcement via `send_state_event` (looking up jury rooms by pseudonym, not case_id) is deferred to m2-late-2 B-actor phase where the puppet-map linkage is established.

3. **`seed_sanction_subscriber` call site** — wired in `crates/server/src/lib.rs` at startup. m2-late-2 should verify the `BRIDGE_SANCTION_CALLBACK_URL` env var is set in the pilot deployment and the subscriber row is seeded correctly before testing B-actor flows.

---

## Per-role task metrics

| Task | Files | Commits | Runtime (approx) | Max log silence |
|---|---|---|---|---|
| T1 schema hand-edit | 3 | 1 | 15 min | n/a (advisor inline) |
| T3 sanction_kind_map | 2 | 1 | 20 min | n/a (Junior #646 scope) |
| T4 sanction_publisher | 1 | 1 | 25 min | n/a |
| T5 submit_jury_vote spawn | 1 | 1 | 20 min | n/a |
| T6 server startup seed | 1 | 1 | 15 min | n/a |
| T7 bridge handler | 3 | 1 | 20 min | n/a |
| T8 e2e test | 1 | 4 (fix iterations) | 45 min | n/a |
| fix-impl-1 (CR round-1) | 5 | 1 | **47 min** | 25 min (log monitoring) |
| fix-impl-2 (CR round-2) | 2 | 1 | 10 min (advisor inline) | n/a |

**Complexity score note:** The migration + e2e combination pushed this above a typical single-session phase. The 4 e2e fix iterations (migration ENUM casing, revert window, clock skew, quorum stop) were the dominant cost.
