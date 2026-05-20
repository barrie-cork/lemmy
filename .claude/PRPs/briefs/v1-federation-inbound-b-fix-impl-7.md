---
phase: v1-federation-inbound-b
role: impl-task
kind: fix-impl
fix_impl_n: 7
authored: 2026-05-20
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
triggering_dq: 290
triggering_task: 9
classification: "Two TEST-FIXTURE defects in Task-9 e2e.rs work (NOT production-code bugs in inbox.rs / publish_*.rs). Phase-2 e2e on tip 95ee8274f returned 100 passed / 2 failed / 5 ignored / 35m50s; both failures localised to crates/server/tests/e2e.rs. Failure 1: `sanction_notice_round_trip` @ e2e.rs:5254 — `instance::table.filter(domain.eq(\"instance-a.test\")).first::<i32>(...)` on `async_conn_b_fixture` (url_b) returns NotFound because Phase-6 created `instance-a.test` only on `url_a` via `Instance::read_or_create(&mut context_a.pool(), \"instance-a.test\")`. Symmetric pattern is at e2e.rs:5225 (`_instance_b = Instance::read_or_create(&mut context_b.pool(), \"instance-b.test\")`). Failure 2: `v1_federation_inbound_b_fixtures::per_peer_rate_limit_returns_429` @ e2e.rs:15618 — raw INSERT of `('instance', 'federation.inbound.per_peer_rate_per_hour', 'int', 2)` collides on append-history with the migration's pre-seeded `value_int=100` row at `valid_from='2026-05-17T00:00:00Z'`; UNIQUE is on `(scope, key, valid_from)` (NOT `(scope, key)` per migration comment); `get_inbound_config_int` reads first row without `ORDER BY` → arbitrary pick of value=100 → 3 receives < cap=100 → rate gate never fires → `assert!(result.is_err())` panics. Both fixes are TEST-FIXTURE corrections, NOT production-code changes. **§G4-allowlist-equivalent mechanical fixes** (two byte-identical-canonical-sibling mirrors of patterns already in the same file)."
base: "phase-v1-federation-inbound-b @ 95ee8274f (post-Task-9-e2e-merge + advisor's DQ #290 raise commit aa4caaf68). Phase-2 e2e FAILED on this tip with the 2 fixture defects above; fix-impl-7 unblocks DQ #290's pass-mutation."
cap: "EXACTLY 2 hunks (≤30 lines net), 1 file ONLY: crates/server/tests/e2e.rs. Hunk-1 @ ~e2e.rs:5252-5258 (sanction_notice_round_trip — replace 7-line `instance::table.filter(...).first::<i32>(...)` lookup with single-line `Instance::read_or_create(&mut context_b.pool(), \"instance-a.test\").await?.id` AND drop the now-unused `use lemmy_db_schema_file::schema::{...instance...}` AND `use lemmy_db_schema_file::InstanceId` imports from the inner `{ }` block). Hunk-2 @ ~e2e.rs:15606-15611 (per_peer_rate_limit_returns_429 — change raw `INSERT INTO governance_config (...) VALUES ('instance', 'federation.inbound.per_peer_rate_per_hour', 'int', 2)` to `UPDATE governance_config SET value_int = 2 WHERE scope = 'instance' AND key = 'federation.inbound.per_peer_rate_per_hour'`). NEVER touch any other file. NEVER touch production code (crates/apub/**, crates/db_schema/**, crates/api/**, crates/utils/**, crates/routes/**, migrations/**). NEVER touch the 4 other v1_federation_inbound_b_fixtures tests (allowlisted_happy_path, blocklisted_peer_returns_403, replayed_activity_returns_409, moderation_label_handler_persists_and_logs — those PASSED, leave them byte-for-byte). NEVER touch the new module's helpers (bootstrap_with_peer, build_unique_sanction_notice_activity, the 4 build_* helpers) — only the rate-test fn body's INSERT-vs-UPDATE statement. NEVER touch the Phase-6 outer fixture (lines ~4960-5240) outside the §10.6 fixture insert block. A 3rd hunk or any other-file edit → STOP + kind:blocker."
serial: "Single-task barrier fix (Task 9 §15-validation → Phase-2 e2e), strictly serial cap=1 — only in-flight Junior for this lane. Phase-2 e2e (DQ #290) is the gate; advisor re-runs LOCAL Phase-2 e2e AFTER this fix's finalize-merge. fed-in-b IMPL PHASE is gated behind THIS fix-impl + a re-run of Phase-2 e2e that returns E2E_EXIT_0 with `test result: ok. <N> passed; 0 failed`. NO further impl-task is queueable until DQ #290 (or its successor) reaches result:pass."
---

# [role:impl-task] v1-federation-inbound-b fix-impl-7 — e2e.rs two test-fixture defects (sanction_notice_round_trip @ url_b lookup + per_peer_rate_limit append-history INSERT→UPDATE)

> **Provenance:** Phase-2 e2e local run on tip `95ee8274f` (DQ #290 validate-pending-laptop-e2e, raised by advisor 02:00 UTC 2026-05-20, completed 35m50s). Result: `test result: FAILED. 100 passed; 2 failed; 5 ignored; 0 measured; 0 filtered out`. Both failures localised to `crates/server/tests/e2e.rs`. RCA (`general-purpose` subagent, ~3min, 170k tokens) identified both as TEST-FIXTURE defects in Task 9's authoring, **NOT production-code bugs**. The wrap_governance_inbound rate-gate logic in `crates/apub/activities/src/governance/inbox.rs:542-563` is correct as written; the test's INSERT collides with the migration's seed-row on append-history semantics, so `get_inbound_config_int` reads value=100 instead of value=2. The Phase-6 fixture insert at e2e.rs:5252-5258 queries the wrong database (url_b) for a row the Phase-6 fixture created only on url_a. Both are mechanical canonical-sibling mirrors of patterns already in the same file. **§G4-allowlist-equivalent** (two byte-identical mirrors, zero design ambiguity). DQ #290 stays pending until the next Phase-2 e2e on the post-fix-impl-7 tip returns E2E_EXIT_0.

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD is a Junior worktree branched off `phase-v1-federation-inbound-b` (base tip `95ee8274f` or newer — accept any tip on `phase-v1-federation-inbound-b` that contains commit `d2e002cb0` which added the Task-9 e2e.rs module). `git merge-base --is-ancestor d2e002cb0 HEAD` MUST be true. If on `phase-v1-federation-inbound-b` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ into `.claude/decision-queue.json` (NOT a repo-root file unless the §4 harness-gap fires).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>` unless dispatch carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — keep the check.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** `git submodule update --init 2>&1` from worktree root. `crates/email/translations` is NOT auto-initialized in a fresh worktree; without it cargo fails pre-existing. Infra, not the fix.
- Confirm Hunk-1 anchor present at the expected shape (grep the text; line numbers approximate — base tip `95ee8274f`):
  ```
  grep -n "Look up instance-a.test by domain" crates/server/tests/e2e.rs
  grep -n "first::<i32>" crates/server/tests/e2e.rs
  ```
  Expected: ~1 line around 5252-5253 with `// Look up instance-a.test by domain (created by Phase 6 fixture).`, and ~1 line with `.first::<i32>(&mut async_conn_b_fixture)`. If 0 → STOP + `kind:"blocker"` (already fixed, or upstream rebase changed the shape). If multiple → STOP + `kind:"blocker"` (cap-2-hunk assumption invalid).
- Confirm Hunk-2 anchor present at the expected shape (line numbers approximate — base tip `95ee8274f`):
  ```
  grep -n "INSERT INTO governance_config" crates/server/tests/e2e.rs
  grep -n "federation.inbound.per_peer_rate_per_hour" crates/server/tests/e2e.rs
  ```
  Expected: `INSERT INTO governance_config` should appear **exactly once** in e2e.rs (this fix removes it). `federation.inbound.per_peer_rate_per_hour` should appear in the same hunk-2 area. If `INSERT INTO governance_config` count ≠ 1 → STOP + `kind:"blocker"` (cap-1-occurrence assumption invalid; multiple test fixtures may need the same UPDATE pattern — surface to advisor).
- Confirm the canonical-sibling patterns are present at the expected shapes (the reference patterns — DO NOT EDIT THEM):
  ```
  # Hunk-1 reference: line ~5225 _instance_b creation pattern
  grep -n "Instance::read_or_create.*instance-b.test" crates/server/tests/e2e.rs
  # Expected: 1 line with `let _instance_b = Instance::read_or_create(&mut context_b.pool(), "instance-b.test").await?;`
  # Hunk-1 reference: line ~4972 instance_a creation pattern
  grep -n "Instance::read_or_create.*instance-a.test.*context_a" crates/server/tests/e2e.rs
  # Expected: 1 line with `let instance_a = Instance::read_or_create(&mut context_a.pool(), "instance-a.test").await?;`
  ```
  If either grep returns 0 lines → STOP + `kind:"blocker"` (the canonical reference shape changed).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b fix-impl-7 — e2e.rs two test-fixture defects (Phase-2 e2e DQ #290 fail)`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b fix-impl-7 — see .claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-7.md
```

## §2 Scope

### 2.1 The defects being fixed (the contract — Phase-2 e2e DQ #290 fail, verified by advisor 2026-05-20)

**Defect 1 (Hunk-1):** In `crates/server/tests/e2e.rs`, around line 5245-5270 (inside the `sanction_notice_round_trip` test's Phase-6 fixture block — the Allowlist-insert block added by Task 9 §10.6):

```rust
{
  use lemmy_db_schema::source::governance::federation_peer::FederationPeerInsertForm;
  use lemmy_db_schema_file::enums::FederationPeerTrust;
  use lemmy_db_schema_file::schema::{federation_peer, instance};
  use lemmy_db_schema_file::InstanceId;
  let mut async_conn_b_fixture = AsyncPgConnection::establish(&url_b).await?;
  // Look up instance-a.test by domain (created by Phase 6 fixture).
  let peer_instance_id: i32 = instance::table
    .filter(instance::domain.eq("instance-a.test"))
    .select(instance::id)
    .first::<i32>(&mut async_conn_b_fixture)
    .await?;
  let form = FederationPeerInsertForm {
    instance_id: InstanceId(peer_instance_id),
    trust_level: Some(FederationPeerTrust::Allowlisted),
    added_by_actor: None,
    notes: None,
  };
  diesel::insert_into(federation_peer::table)
    .values(&form)
    .execute(&mut async_conn_b_fixture)
    .await?;
}
```

The Phase-6 outer fixture creates `instance-a.test` ONLY on `url_a` (line ~4972, `let instance_a = Instance::read_or_create(&mut context_a.pool(), "instance-a.test").await?;`). The `_instance_b` is created on `url_b` for `"instance-b.test"` at line ~5225 (`let _instance_b = Instance::read_or_create(&mut context_b.pool(), "instance-b.test").await?;`). Task 9's fixture insert opens `async_conn_b_fixture` against `url_b` and looks up `"instance-a.test"` on that connection — `url_b` has no such row → `.first::<i32>(...).await?` returns `LemmyError { message: NotFound, ..., caller: crates\server\tests\e2e.rs:5254:33, inner: Record not found }`.

The Phase-6 round-trip needs `federation_peer.instance_id` to reference a real `instance` row on `url_b` (because `wrap_governance_inbound`'s peer-trust gate does an inner-join on `instance::table` + `federation_peer::table` on the receive-side DB, which is `url_b` via `federation_context_b`). The lookup pattern is correct in INTENT; the row simply doesn't exist yet on `url_b` because Phase-6's fixture didn't seed `instance-a.test` there.

**Defect 2 (Hunk-2):** In `crates/server/tests/e2e.rs`, around line 15606-15611 (inside the `per_peer_rate_limit_returns_429` test body):

```rust
diesel::sql_query(
  "INSERT INTO governance_config (scope, key, value_type, value_int) \
   VALUES ('instance', 'federation.inbound.per_peer_rate_per_hour', 'int', 2)",
)
.execute(&mut conn)
.await?;
```

The `governance_config` table is **append-history with UNIQUE on `(scope, key, valid_from)`** — NOT on `(scope, key)` (per migration `migrations/2026-04-18-000000-0000_add_governance_config/up.sql:41-42` and its explicit comment "Do NOT use `(scope, key)` as the conflict target"). The migration `migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql:127` already seeds `('instance', 'federation.inbound.per_peer_rate_per_hour', 'int', 100, ..., valid_from='2026-05-17T00:00:00Z')`. The test's raw INSERT succeeds (different `valid_from = now()`) but now TWO rows exist for the same `(scope, key)`.

`get_inbound_config_int` in `crates/apub/activities/src/governance/inbox.rs:421-438` reads via `.first::<Option<i64>>(conn)` WITHOUT `.order_by(governance_config::valid_from.desc())`. Postgres returns rows in implementation-defined order — empirically the physically-first-inserted row (the migration's `value_int = 100`). Result: `peer_cap = 100`, 3 receives ≪ 100, rate gate never fires, `result` is `Ok`, `assert!(result.is_err())` panics at e2e.rs:15618.

**Why test-side fix, not reader-side fix:** `get_inbound_config_int`'s append-history-unaware read is a latent defect on the production read path that should be addressed in a future sub-phase or retro carry-forward (RCA flagged this; see §7). For THIS fix-impl, we change the TEST FIXTURE to mutate the existing row instead of appending a colliding history entry — minimal blast radius, no production-code touch, no risk of cascading regressions in other config-int consumers (e.g. `publish_trust_attestation.rs:142-152` mirror). The fix-impl-7 scope is e2e.rs only.

Test-result verbatim:

```
test result: FAILED. 100 passed; 2 failed; 5 ignored; 0 measured; 0 filtered out; finished in 2150.41s

failures:
    sanction_notice_round_trip
    v1_federation_inbound_b_fixtures::per_peer_rate_limit_returns_429
```

### 2.2 The fix (the contract — TWO verbatim canonical-sibling mirrors; copy EXACTLY, do not paraphrase)

**§G4 CANONICAL RECIPE (verbatim mirrors — there is no single allowlist row for "e2e test-fixture defect", so the §G4-allowlist-equivalent here is the canonical-sibling-mirror rule in `.claude/rules/advisor-orchestrator.md` §3.6 + §G4 classifier "Canonical-sibling-mirror discipline"):**

> | Failure signature | Auto-fix | Source lesson |
> | `LemmyError { NotFound, ..., Record not found }` at `instance::table.filter(domain.eq(X)).first::<i32>(conn_to_DB_Y)` when row X exists only in DB Z (Z ≠ Y) | **Use `Instance::read_or_create(&mut <correct_context>.pool(), X).await?.id`** to (a) idempotently ensure the row exists on the target DB and (b) return its id directly. Mirrors the existing pattern in the SAME file (e.g. e2e.rs:5225 for `_instance_b` on `url_b`, e2e.rs:4972 for `instance_a` on `url_a`). | canonical-sibling discipline + RCA-2026-05-20 |
> | `assert!(result.is_err())` panics on a config-overrride INSERT into `governance_config` (append-history with UNIQUE on `(scope, key, valid_from)`) | **Change raw INSERT to UPDATE** of the existing row: `UPDATE governance_config SET value_<type> = <new_value> WHERE scope = '<scope>' AND key = '<key>'`. The migration's seed row is mutated in place; no append-history collision; reader's first-row-pick correctly resolves to the test-desired value. | canonical-sibling discipline + RCA-2026-05-20 |

**Hunk-1 (replace lines ~5247-5258 — the `instance::table.filter(domain.eq("instance-a.test")).first::<i32>(...)` lookup with `Instance::read_or_create` on context_b):**

CURRENT (verbatim from base tip 95ee8274f, lines ~5247-5269):

```rust
  {
    use lemmy_db_schema::source::governance::federation_peer::FederationPeerInsertForm;
    use lemmy_db_schema_file::enums::FederationPeerTrust;
    use lemmy_db_schema_file::schema::{federation_peer, instance};
    use lemmy_db_schema_file::InstanceId;
    let mut async_conn_b_fixture = AsyncPgConnection::establish(&url_b).await?;
    // Look up instance-a.test by domain (created by Phase 6 fixture).
    let peer_instance_id: i32 = instance::table
      .filter(instance::domain.eq("instance-a.test"))
      .select(instance::id)
      .first::<i32>(&mut async_conn_b_fixture)
      .await?;
    let form = FederationPeerInsertForm {
      instance_id: InstanceId(peer_instance_id),
      trust_level: Some(FederationPeerTrust::Allowlisted),
      added_by_actor: None,
      notes: None,
    };
    diesel::insert_into(federation_peer::table)
      .values(&form)
      .execute(&mut async_conn_b_fixture)
      .await?;
  }
```

REPLACEMENT (the §2.2 contract — apply EXACTLY this; mirrors line 5225 `_instance_b` pattern):

```rust
  {
    use lemmy_db_schema::source::governance::federation_peer::FederationPeerInsertForm;
    use lemmy_db_schema_file::enums::FederationPeerTrust;
    use lemmy_db_schema_file::schema::federation_peer;
    // Phase-6 fixture creates instance-a.test only on url_a's context_a.pool().
    // For the federation_peer.instance_id FK on url_b, we need an instance-a.test
    // row on url_b too — Instance::read_or_create is idempotent (returns existing
    // row if present, else inserts and returns it). Mirrors the _instance_b
    // pattern above at line ~5225.
    let instance_a_on_b = Instance::read_or_create(&mut context_b.pool(), "instance-a.test").await?;
    let mut async_conn_b_fixture = AsyncPgConnection::establish(&url_b).await?;
    let form = FederationPeerInsertForm {
      instance_id: instance_a_on_b.id,
      trust_level: Some(FederationPeerTrust::Allowlisted),
      added_by_actor: None,
      notes: None,
    };
    diesel::insert_into(federation_peer::table)
      .values(&form)
      .execute(&mut async_conn_b_fixture)
      .await?;
  }
```

Net diff: -2 `use` statements (`schema::{... instance ...}` → `schema::federation_peer` only; `InstanceId` removed), +1 `let instance_a_on_b = Instance::read_or_create(...)`, -5 lines of the `instance::table.filter(...).first::<i32>(...)` lookup, +3 lines of comment, +1 line of helper. Net hunk: ~12 lines changed. The `FederationPeerInsertForm`'s `instance_id` field becomes `instance_a_on_b.id` directly (an `InstanceId`, no wrapper needed — `Instance.id` IS the `InstanceId`).

**Hunk-2 (replace lines ~15606-15611 — the raw `INSERT INTO governance_config` with `UPDATE`):**

CURRENT (verbatim from base tip 95ee8274f, lines ~15606-15611):

```rust
    diesel::sql_query(
      "INSERT INTO governance_config (scope, key, value_type, value_int) \
       VALUES ('instance', 'federation.inbound.per_peer_rate_per_hour', 'int', 2)",
    )
    .execute(&mut conn)
    .await?;
```

REPLACEMENT (the §2.2 contract — apply EXACTLY this):

```rust
    // governance_config is append-history with UNIQUE on (scope, key, valid_from)
    // — NOT on (scope, key). The migration 2026-05-17 already seeded this key
    // with value_int=100; raw INSERT would create a second row and the reader
    // (get_inbound_config_int) returns an arbitrary one. UPDATE mutates the
    // existing seed row in place. See migration 2026-04-18 comment "Do NOT use
    // (scope, key) as the conflict target" for the schema invariant.
    diesel::sql_query(
      "UPDATE governance_config SET value_int = 2 \
       WHERE scope = 'instance' AND key = 'federation.inbound.per_peer_rate_per_hour'",
    )
    .execute(&mut conn)
    .await?;
```

Net hunk: ~6 lines changed (3 SQL string lines replaced, 5 comment lines added). Total fix-impl-7 diff: ~18 lines net across 2 hunks in 1 file.

### 2.3 What is NOT in scope (the fence)

- **NEVER touch any file other than `crates/server/tests/e2e.rs`.** No production code in `crates/apub/**`, `crates/db_schema/**`, `crates/api/**`, `crates/utils/**`, `crates/routes/**`. No migrations. No Cargo.*, no schema.rs, no plan, no lesson, no template.
- **NEVER touch the 4 OTHER v1_federation_inbound_b_fixtures tests** (allowlisted_happy_path_persists_advisory_row, blocklisted_peer_returns_403, replayed_activity_returns_409, moderation_label_handler_persists_and_logs). Those all PASSED on tip 95ee8274f; leave byte-for-byte.
- **NEVER touch the new module's helpers** (bootstrap_with_peer, build_unique_sanction_notice_activity, build_sanction_notice_with_id, build_minimal_publish_label_activity, the 4 other build_* helpers). The rate-test test calls these and they work correctly.
- **NEVER touch the Phase-6 outer fixture body** outside the §10.6 Allowlist-insert block (lines ~5247-5269). The rest of `sanction_notice_round_trip` (Phase 1-9 and the post-receive assertions in steps 10+) is correct and the v1-federation-inbound-b changes did not touch it.
- **NEVER touch the production rate-gate code path** in `crates/apub/activities/src/governance/inbox.rs` (`check_per_actor_rate_limit`, `rate_per_actor_counts`, `current_hour_bucket`, `get_inbound_config_int`). The production code is correct as written for the v0-defined config semantics; the latent append-history-unaware read is a retro carry-forward, NOT a fix-impl-7 task.
- **NEVER touch `crates/apub/activities/src/governance/publish_trust_attestation.rs`'s analogous `get_inbound_config_int` call site** (lines ~142-152). Same reasoning — retro carry-forward.
- **NEVER add `#[allow(...)]` or `#[expect(...)]`** anywhere.
- **NEVER attempt to "improve" the canonical sibling references** (lines 4972, 5225). Leave them byte-for-byte.

### 2.4 Verification (run BEFORE committing — pre-push cargo-check + cargo-test --no-run discipline per `feedback_fix_impl_pre_push_cargo_check.md` 2026-05-13)

In the worker's worktree, after applying the §2.2 edits, run a LOCAL pre-push validation BEFORE pushing. **Three cmds in sequence; bat wrapper required on Windows per `feedback_windows_e2e_requires_bat_wrapper`; SEPARATE `cmd //c` invocations per `feedback_batch_goto_eof_clobbers_errorlevel` (NEVER bat-&&-bat chain):**

```
# 1. cargo check
bash scripts/brehon/cargo-check.sh --workspace --features full 2>&1 | tail -30

# 2. cargo clippy -D warnings
bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings 2>&1 | tail -30

# 3. cargo test --test e2e --no-run (verify e2e binary LINKS after edit)
bash scripts/brehon/cargo-test.sh --workspace --features full --test e2e --no-run 2>&1 | tail -30
```

Expected: all three exit 0 with `Finished` line. If any cmd fails:
- **Compile error STILL at the SAME e2e.rs:line cited above** (NotFound, or the INSERT/UPDATE doesn't compile) → STOP + `kind:"blocker"` with the new error text (the §2.2 fix shape is wrong; advisor will re-author).
- **NEW compile error somewhere else** (unused-import on `lemmy_db_schema_file::InstanceId` because Hunk-1 removes its use; unused-import on `lemmy_db_schema_file::schema::instance` because Hunk-1 removes the `instance::table` usage) → patch in-commit (remove the unused imports as part of Hunk-1 per the §2.2 replacement; the spec already specifies removing them). If the patch produces a NEW unused-import elsewhere, STOP + `kind:"blocker"`.
- **Clippy warning treated as error** (e.g. `clippy::needless_borrow` on the new `Instance::read_or_create(&mut context_b.pool(), ...)` — note: `context_b.pool()` returns `DbPool::Conn`-equivalent; `&mut` is correct because `Instance::read_or_create` takes `&mut DbPool`. Mirror line 5225 `_instance_b` shape verbatim; if clippy still complains, surface the exact warning) → STOP + `kind:"blocker"` with the offending clippy text.
- **`cargo test --no-run` fails to LINK e2e binary** (unrelated regression: missing trait, missing import) → STOP + `kind:"blocker"` with the link-error text.

Then grep-verify the §2.2 edits landed cleanly (Note: `grep -c` exits 1 if count is 0, which kills `&&` chains — wrap each with `|| echo "0"` if chaining):

```
# Expected: exactly 0 instances of the OLD instance::table.filter pattern
grep -c "instance::table" crates/server/tests/e2e.rs || echo "0"
# (expected: 0 — we removed the only filter-based lookup; the schema::instance import is also removed)

# Expected: exactly 0 instances of the OLD .first::<i32>(&mut async_conn_b_fixture) pattern  
grep -c "first::<i32>(&mut async_conn_b_fixture)" crates/server/tests/e2e.rs || echo "0"
# (expected: 0 — we replaced this with Instance::read_or_create on context_b)

# Expected: exactly 1 instance of the NEW Instance::read_or_create for instance-a.test on context_b
grep -c "Instance::read_or_create.*&mut context_b.pool().*instance-a.test" crates/server/tests/e2e.rs || echo "0"
# (expected: 1 — the new Hunk-1 line)

# Expected: exactly 0 instances of the OLD INSERT INTO governance_config in test bodies
grep -c "INSERT INTO governance_config" crates/server/tests/e2e.rs || echo "0"
# (expected: 0 — Hunk-2 changed the only occurrence to UPDATE)

# Expected: exactly 1 instance of the NEW UPDATE governance_config
grep -c "UPDATE governance_config" crates/server/tests/e2e.rs || echo "0"
# (expected: 1 — the new Hunk-2 line)

# Expected: canonical sibling references UNCHANGED
grep -c "Instance::read_or_create.*context_b.pool.*instance-b.test" crates/server/tests/e2e.rs || echo "0"
# (expected: 1 — _instance_b creation, unchanged)
grep -c "Instance::read_or_create.*context_a.pool.*instance-a.test" crates/server/tests/e2e.rs || echo "0"
# (expected: 1 — instance_a creation, unchanged)
```

If ANY grep returns the wrong count → patch + re-grep before committing.

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` pending entries** — DQ #290 (validate-pending-laptop-e2e, from:"advisor", branch=phase-v1-federation-inbound-b, phase_task=9) is the gating entry; this fix-impl unblocks it. Read its `commands[]` — the advisor re-runs that exact cmd locally after this fix's finalize-merge to get a clean Phase-2 e2e re-validation.

2. **`crates/server/tests/e2e.rs` Hunk-1 area (~lines 4960-5275)** — read the entire `sanction_notice_round_trip` fn including:
   - Line ~4972: `let instance_a = Instance::read_or_create(&mut context_a.pool(), "instance-a.test").await?;` (Phase-6 creates instance-a.test on url_a)
   - Line ~5225: `let _instance_b = Instance::read_or_create(&mut context_b.pool(), "instance-b.test").await?;` (Phase-6 creates instance-b.test on url_b — the CANONICAL SIBLING that Hunk-1 mirrors)
   - Lines ~5247-5269: the Task 9 §10.6 fixture-insert block (the defect — to be replaced by Hunk-1)
   - Lines ~5270+: `ActivityTrait::verify(&activity, &federation_context_b).await...` (the rest is unchanged; verify reads from url_b after the fixture insert)

3. **`crates/server/tests/e2e.rs` Hunk-2 area (~lines 15580-15625)** — read the entire `per_peer_rate_limit_returns_429` fn body. Note especially:
   - The migration-seeded value: `migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql:127` (`per_peer_rate_per_hour = 100`)
   - The test's intent: cap=2, send 3 activities, expect the 3rd to fail with `FederationPeerRateLimitExceeded`
   - The assertion: `assert!(result.is_err());` at ~e2e.rs:15618 — the panic site

4. **`crates/apub/activities/src/governance/inbox.rs:421-438`** (the read-side defect to NOT fix here — `get_inbound_config_int`'s lack of `.order_by(valid_from.desc())`). The RCA flagged this as a retro carry-forward; fix-impl-7 does NOT touch it.

5. **`migrations/2026-04-18-000000-0000_add_governance_config/up.sql:38-50`** — the `governance_config` schema definition. Read the UNIQUE constraint and the comment "Do NOT use (scope, key) as the conflict target". This is the load-bearing schema invariant for Hunk-2.

6. **`migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql`** — grep for `federation.inbound.per_peer_rate_per_hour` to find line ~127 with the migration's seed row (`value_int = 100`). This is what the test was unknowingly racing against.

7. **`.claude/PRPs/plans/v1-federation-inbound-b.plan.md`** — §10.6 (Phase-6 fixture insert intent — the plan body said "instance-a.test by domain (created by Phase 6 fixture)" without naming which DB; this conflation is the planner-side root cause that produced Hunk-1's defect) + §10.7 (new module's per_peer_rate_limit_returns_429 test intent).

8. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4):
   - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Why:** Case A discipline holds on the new tests; the §2.2 hunks preserve `LemmyResult<()>` outer + bare `?` (no `.map_err` bridges). Mirror existing helpers.
   - `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — **Why:** mechanical fix-impl briefs MUST include the §2.4 pre-push triple-cargo-check (check + clippy -D warnings + test --no-run); local cargo check + clippy + link ~3min vs a full Phase-2 e2e re-run cycle on regression.
   - `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** clippy denies `#[allow]`; the §2.2 fix must NOT use a lint-suppression workaround.
   - `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **Why:** TWO Edits ONLY on the 15k-line e2e.rs (Hunk-1 in-place + Hunk-2 in-place — NEVER read the entire file; locate each hunk via grep, edit, save, move on).
   - `.claude/lessons/feedback_batch_goto_eof_clobbers_errorlevel.md` — **Why:** §2.4 cargo cmds are 3 SEPARATE `cmd //c` invocations (NEVER bat-&&-bat chain).
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always) — capture-then-tail for §2.4 pre-push validation.

## §3a Handover from prior cohort

> **Handover state: PHASE-2-E2E-FAIL-DRIVEN** (per `.claude/rules/advisor-orchestrator.md` §4.3 — Task-9 finalize-merge commit carries no `HANDOVER:` YAML trailer; synthesised by advisor from RCA + tip-95ee8274f). This is NOT a catch-fire; the §0 grep self-verifies the actual symbols on the phase tip as the authoritative existence check.

```yaml
prior_tasks:
  - task: 9
    commit: d2e002cb0 (impl — e2e.rs Phase-6 fixture Allowlist + handler-e2e module, 286-line diff) + finalize-merge 1153d4b62 + DQ #289 mutate 95ee8274f
    filesModified: [crates/server/tests/e2e.rs]
    keyDecisions:
      - "Task 9 inserted §10.6 fixture (federation_peer Allowlisted) BEFORE ActivityTrait::verify in Phase-6 sanction_notice_round_trip (~e2e.rs:5247-5269). Also appended mod v1_federation_inbound_b_fixtures (5 tests + bootstrap_with_peer helper + 4 build_*_activity helpers) per §10.7."
      - "All §15 (check + clippy + test --no-run) PASSED first try on this tip (DQ #289 mutated pass). Phase-2 e2e is the NEXT gate (DQ #290) — and DQ #290 FAILED with 2 fixture defects this fix-impl-7 addresses."
      - "DEFECT-1: §10.6 fixture insert queries instance-a.test on url_b (async_conn_b_fixture), but Phase-6 only created that row on url_a. Phase-6's _instance_b at line ~5225 only seeded instance-b.test on url_b. Fix: Instance::read_or_create(&mut context_b.pool(), 'instance-a.test') — idempotent, returns existing if present, else inserts."
      - "DEFECT-2: §10.7 per_peer_rate_limit_returns_429 raw INSERT into governance_config collides on append-history with the migration's seed row. Fix: change INSERT to UPDATE on the existing (scope, key) pair. Migration explicitly warned 'Do NOT use (scope, key) as the conflict target' — readers' first-row-pick semantic is a latent defect; the test-side fix avoids touching production code."
      - "100 of 102 e2e tests PASSED on tip 95ee8274f; the 4 other v1_federation_inbound_b_fixtures (allowlisted_happy_path / blocklisted_peer_returns_403 / replayed_activity_returns_409 / moderation_label_handler_persists_and_logs) ALL PASSED — confirming wrap_governance_inbound's Allowlist / Blocklist / Replay / Cap / Handler paths work correctly. Only the rate-gate path needed the config-override-via-INSERT to work, and that's where the schema-mismatch hit."
      - "Both fixes are TEST-FIXTURE-only edits, NOT production-code edits. Zero ADR exposure. Zero risk to other tests. The reader-side defect (get_inbound_config_int + publish_trust_attestation.rs:142-152 mirror) is a retro carry-forward — fix-impl-7 explicitly does NOT bundle this."
    notes: "fix-impl-7 CONSUMES Task-9's e2e.rs unchanged except for the 2 hunks at ~lines 5247-5269 (Hunk-1) and ~lines 15606-15611 (Hunk-2). The §0 grep self-verifies the defect symbol presence + canonical-sibling presence as the authoritative existence check. The §2.4 §15-precondition verification (check + clippy + test --no-run) is the worker's contract: §2.4 pre-push MUST pass before exit. Advisor re-runs Phase-2 e2e LOCAL after finalize-merge; DQ #290 stays pending until that re-run returns E2E_EXIT_0 with `test result: ok`."
```

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-b` (tip `95ee8274f` or newer — accept any tip post-`d2e002cb0`). Finalize merges your worktree branch back; do NOT push to `phase-v1-federation-inbound-b` directly.
- ONE commit (single file `crates/server/tests/e2e.rs`, 2 hunks, ~18 lines net). Commit subject:

  ```
  fix(v1-federation-inbound-b): e2e.rs test-fixture corrections — Instance::read_or_create on context_b + UPDATE governance_config (fix-impl-7 → DQ #290 Phase-2 e2e)
  ```

- Mid-task DQ visibility: if you raise a `pending` entry mid-task, **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235)

If you need to write a DQ entry and the Claude Code sensitive-file gate blocks it: (a) write the intended DQ-entry JSON to `FIXIMPL7_BLOCKER_DQ.json` at worktree root, (b) write `FIXIMPL7_ESCALATION.md` naming the issue, (c) commit both + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`. fix-impl-7 has NO `.claude/` deliverable on the happy path — its only deliverable is the 2-hunk e2e.rs edit; there is NO `validate-pending` DQ write from this fix (advisor mutates the EXISTING DQ #290 after re-running Phase-2 e2e on the post-fix-impl-7 tip).

### fix-impl-7 GOTCHAs (load-bearing)

- **TWO Edits ONLY on the 15k-line e2e.rs** per `feedback_junior_worker_e2e_edit_hang.md`. Hunk-1 in-place at ~lines 5247-5269; Hunk-2 in-place at ~lines 15606-15611. NEVER read the entire file (15k lines blows context budget); locate each hunk via grep (the §0 anchors give you exact line numbers), Edit, save, move on. Do NOT re-read e2e.rs after editing.
- **The canonical siblings are in the SAME file.** Hunk-1's reference is line 5225 (`_instance_b = Instance::read_or_create(&mut context_b.pool(), "instance-b.test")`); read it before applying §2.2 Hunk-1. Hunk-2 has no in-file canonical-sibling for UPDATE-vs-INSERT (the migration's seed-row pattern is the implicit canonical) — the §2.2 contract IS the recipe.
- **Hunk-1 removes 2 use statements.** `use lemmy_db_schema_file::schema::{... instance ...};` becomes `use lemmy_db_schema_file::schema::federation_peer;` (drop the `, instance` portion). `use lemmy_db_schema_file::InstanceId;` is fully removed (no longer needed; `Instance.id` is already `InstanceId`-typed). Pre-existing imports near the top of `sanction_notice_round_trip` (e.g. `Instance`) are already in scope from the outer fn — DO NOT add a duplicate `use` statement.
- **Hunk-1 must compile: `Instance::read_or_create` returns `LemmyResult<Instance>`** (per `crates/db_schema/src/source/instance.rs`). The `.id` access on the resulting `Instance` value gives an `InstanceId`. The `FederationPeerInsertForm.instance_id` field expects `InstanceId`. Match types directly: `let instance_a_on_b = Instance::read_or_create(...).await?;` then `instance_id: instance_a_on_b.id,`.
- **Hunk-2's UPDATE returns `usize` (rows affected), NOT a row.** `.execute(&mut conn).await?` works correctly. Do NOT change to `.get_result` or `.load`.
- **DO NOT add `.first` ordering to `get_inbound_config_int`** (the production read site in inbox.rs). That's a retro carry-forward, NOT a fix-impl-7 task.
- **One commit, one file.** If §2.4 pre-push validation surfaces a regression in a different file (unexpected), STOP + `kind:"blocker"` (do NOT add a third hunk or touch another file).

### Plan-cited line numbers may have drifted

§2.2 cites lines ~5247-5269 and ~15606-15611. `grep -n "Look up instance-a.test by domain" crates/server/tests/e2e.rs` and `grep -n "INSERT INTO governance_config" crates/server/tests/e2e.rs` for the real positions on the actual base tip. The edits are identified by the STRING patterns (`Look up instance-a.test by domain`, `instance::table`, `INSERT INTO governance_config`, `federation.inbound.per_peer_rate_per_hour`), not by line numbers.

## §5 Validation gates (advisor re-runs Phase-2 e2e — worker runs §2.4 pre-push validation)

**Worker-side (this fix-impl):** §2.4 pre-push triple-validation (cargo-check + cargo-clippy -D warnings + cargo-test --test e2e --no-run) is the only worker-side validation. Worker does NOT raise a NEW `validate-pending-laptop-e2e` DQ — the existing DQ #290 (raised by advisor 2026-05-20, still pending) is the gate the advisor mutates after re-running Phase-2 e2e on the post-finalize-merge tip.

**Advisor-side (after this fix's finalize-merge):** advisor re-runs DQ #290's `commands[]` verbatim on the laptop lane:

```
cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-fed-in-b-<new-tip-sha>.log 2>&1"
```

On `test result: ok. <N> passed; 0 failed`: advisor mutates DQ #290 result:"pass" + fed-in-b IMPL PHASE COMPLETE. On any failure: surface as CATCH-FIRE per §5.3 (non-allowlist e2e-class fail).

## §6 Expected output (return to advisor)

```
## fix-impl-7 complete — e2e.rs test-fixture corrections (Hunk-1 + Hunk-2)

**Commit:** <sha> on <worktree-branch>
**File changed:** crates/server/tests/e2e.rs (2 hunks, ~18 lines net)
**Hunk-1:** lines ~5247-5269 — replaced instance::table.filter+first lookup with Instance::read_or_create on context_b for "instance-a.test"; dropped unused use statements
**Hunk-2:** lines ~15606-15611 — changed raw INSERT INTO governance_config to UPDATE WHERE scope/key
**PRECON self-check:** §10.6 fixture anchor present at ~e2e.rs:5247; per_peer_rate INSERT anchor present at ~e2e.rs:15606; both canonical siblings (line 5225 _instance_b + line 4972 instance_a) UNCHANGED. Base tip ancestor of d2e002cb0 confirmed.
**No other-file edit:** confirmed (only crates/server/tests/e2e.rs touched).
**§2.4 pre-push validation:** ALL 3 GREEN — cargo-check exit 0, cargo-clippy -D warnings exit 0, cargo-test --test e2e --no-run exit 0 (e2e binary links cleanly).
**Grep-verify:** the 6 grep counts at expected values (instance::table=0, first::<i32>(&mut async_conn_b_fixture)=0, NEW Instance::read_or_create context_b instance-a.test=1, INSERT INTO governance_config=0, NEW UPDATE governance_config=1, canonical siblings unchanged: _instance_b=1, instance_a=1).
**No new DQ raised** (the existing DQ #290 from advisor stays pending; advisor mutates after re-running Phase-2 e2e on the post-finalize-merge tip).
**Next:** advisor laptop re-runs Phase-2 e2e LOCAL on the post-finalize-merge tip; on E2E_EXIT_0 + `test result: ok` → mutate DQ #290 result:pass → fed-in-b IMPL PHASE COMPLETE → user-gate-5 (merge confirm) → bm-pr → CR triage → bm-merge → retro.
```

Plus any DQ #N references if you raised a blocker mid-task (NOT for §2.4 pre-push failure — that you patch in-commit per the §2.4 routing).

## §7 Why this brief differs from the plan

Plan §10.6 (Phase-6 fixture Allowlist insert) and §10.7 (per_peer_rate_limit_returns_429 test body) both shipped as written in Task 9. The §10.6 wording said "instance-a.test by domain (created by Phase 6 fixture)" without naming the target DB; the §10.7 INSERT pattern did not flag the append-history collision against the migration's seed row. Both are planner-side oversights surfaced ONLY by Phase-2 e2e execution — neither was catchable at §15 (compile + clippy + link). This fix-impl is a 2-hunk byte-identical-sibling mirror per §G4-allowlist-equivalent canonical recipe; plan §10.6/§10.7 retrofit is NOT needed (the new patterns are documented inline in fix-impl-7's commit + brief; future test authors should grep these patterns as canonical siblings).

This brief mirrors the canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-5.md` schema per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate (same §0-§7 structure; same single-file CAP discipline; same triggering-DQ-mutated-by-advisor-post-validation pattern). Includes the VERBATIM §G4 blockquote (extended for the two failure signatures) in §2.2 per `.claude/rules/advisor-orchestrator.md` §G4 mandatory-verbatim-recipe rule.

**Retro carry-forward (NOT bundled here — file as `kind:"log"` DQ at fix-impl-7 close):** `get_inbound_config_int` in `crates/apub/activities/src/governance/inbox.rs:421-438` AND its mirror in `crates/apub/activities/src/governance/publish_trust_attestation.rs:142-152` BOTH lack `.order_by(governance_config::valid_from.desc())` — they return the first physically-inserted row, which is wrong when `governance_config` accumulates an admin-edit history. The migration explicitly warned 'Do NOT use (scope, key) as the conflict target' but the readers don't honour the append-history semantic. Should use `governance_config_current` view OR add `.order_by(valid_from.desc()).limit(1)`. Defer to fed-in-b retro / a follow-on sub-phase. Symptom: any test or admin-edit-config flow that adds a 2nd row for an existing (scope, key) will see arbitrary value pick.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b impl under serial-phase policy). Brief committed on `governance-v0` first; then cherry-picked / re-committed onto `phase-v1-federation-inbound-b` so the worker — which branches from the PHASE branch tip — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC to the post-cherry-pick tip before queue. §G4-allowlist-equivalent mechanical fix (canonical-sibling mirror; zero design ambiguity); does NOT require AskUserQuestion — auto-queued per §G4 allowlist routing + user pre-approval 2026-05-20. fix-impl-7 is the FIRST fix-impl for Task 9; numbering: 1-3 = Task 4 chain, 4 = Task 5 wrap-sig HRTB, 5-6 = Task 6 .into() + clippy chain, 7 = Task 9 e2e.rs test-fixture corrections._
