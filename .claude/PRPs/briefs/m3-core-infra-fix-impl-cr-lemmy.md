# Brief: m3-core-infra fix-impl-cr-lemmy (cr-4, cr-5)

## §1 Role + dispatch line

`[role:impl-task] m3-core-infra-fix-cr-lemmy — see .claude/PRPs/briefs/m3-core-infra-fix-impl-cr-lemmy.md`

## §2 Scope

Fix 2 CodeRabbit findings (Lemmy-side, Windows-validated), approved fix-in-pr at gate-3 (PR #201):

- **cr-4** (`crates/server/tests/e2e/governance.rs`, MAJOR/heavy-lift): the test
  `m3_actor_pseudonym_endpoint_idempotent_opaque` (line ~5399) validates only
  `actor_pseudonym_helper::get_or_create` — it does NOT exercise the actual
  `/bridge/actor-pseudonym` HTTP route, bridge-secret auth, query extraction, or the handler
  contract. Refactor (or ADD a second test) so the route is hit via an actix
  `test::init_service` + `test::call_service` request with the bridge-secret header, asserting on
  the JSON response body (`{"pseudonym": "..."}`) — proving the wired route + auth + extraction,
  not just the helper.
- **cr-5** (`migrations/2026-06-18-000000-0000_seed_rtc_enabled_config/down.sql`, LOW/lint):
  `WHERE` is on the same line as `DELETE FROM` — violates SQLFluff LT14. Put `WHERE` and each
  `AND` predicate on its own line.

**Produces:** edits to exactly these 2 files, one commit. Do NOT touch any other file.

**Do NOT:** edit `services/bridge/`, `AGPL-NOTICE.md`, `.claude/`, `up.sql`, or any other file.
cr-6/cr-7/cr-8 are a separate worker (bridge). cr-1/cr-2 already landed advisor-side. cr-3 is
being rebutted (NOT fixed).

**Branch:** forks from `phase-m3-core-infra` (current tip `bfcf8aac9`).

## §3 Required reading

- `crates/server/tests/e2e/governance.rs:3225-3345` — the **canonical actix HTTP-route test
  harness** (`test::init_service` + `App::new().app_data(...).configure(lemmy_api_routes::config)`
  + `test::call_service` + status/body assertions). MIRROR this pattern for the cr-4 route test —
  do NOT invent a new harness.
- `crates/server/tests/e2e/governance.rs:5398-5427` — the current `m3_actor_pseudonym...` test to
  refactor/extend.
- `crates/api/api/src/governance/bridge_read.rs:67-82` — the `get_bridge_actor_pseudonym` handler:
  `bridge_auth::verify_bridge_secret(&req)?` then `actor_pseudonym_helper::get_or_create` →
  `Json(BridgeActorPseudonym { pseudonym })`. The route is `/bridge/actor-pseudonym?person_id=N`.
- `crates/api/api/src/governance/bridge_auth.rs` — read `verify_bridge_secret` to learn the header
  name + the env/secret it checks (so the test sets the right header to get past auth; if the
  shared secret comes from env/config, set it in the test the way the sibling harness does, OR
  assert the 401-without-secret + a separate path — match what the sibling status tests do).
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — `LemmyResult<()>` / `?` error shape;
  mirror the sibling test's case verbatim.
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `governance_fixtures::bootstrap()` +
  `context.pool()` pattern (the current test already uses it; keep it).
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim `old_string`
  anchors before editing this >5400-line file; confirm uniqueness with `grep -c`.

## §4 Constraints

### cr-4 — exercise the real route

Mirror the `test::init_service` harness at lines 3225-3345. The new/refactored test must:
1. Bootstrap fixtures + create a person (keep the existing `governance_fixtures::bootstrap()` +
   `Person::create` setup — note the `use lemmy_diesel_utils::traits::Crud;` standalone import that
   was the E0432 fix; do NOT regress it).
2. Build the actix app with `lemmy_api_routes::config` (mirror line 3241).
3. Issue `GET /bridge/actor-pseudonym?person_id=<id>` WITH the bridge-secret header
   (per `bridge_auth::verify_bridge_secret`), assert status 200 + parse the JSON body, assert
   `pseudonym` is non-empty and `!= person.name` (the ADR-015 opacity check).
4. Keep the idempotency assertion (two calls → same pseudonym) — now via two HTTP requests OR
   one HTTP + the helper, your choice, but at least ONE assertion must go through the route.
5. Optionally assert 401 when the bridge secret is absent/wrong (proves auth is wired).

You MAY keep the existing helper-only test AND add a new route test, or refactor the existing one
to add route coverage — either satisfies cr-4. Prefer adding a distinctly-named second test
(e.g. `m3_actor_pseudonym_endpoint_route_authed`) to keep the diff additive + the anchor unique.

### cr-5 — multi-line the down.sql

```sql
DELETE FROM governance_messaging_config WHERE scope='instance' AND key='rtc_enabled' AND valid_from='2026-06-18T00:00:00Z'::timestamptz;
```
→
```sql
DELETE FROM governance_messaging_config
WHERE scope = 'instance'
  AND key = 'rtc_enabled'
  AND valid_from = '2026-06-18T00:00:00Z'::timestamptz;
```
(WHERE + each AND predicate on its own line, per SQLFluff LT14.)

### Anchor uniqueness gate (pre-edit, mandatory)

1. `grep -c 'm3_actor_pseudonym_endpoint_idempotent_opaque' crates/server/tests/e2e/governance.rs` → `1`
2. If adding a new test, `grep -c '<new-test-fn-name>' crates/server/tests/e2e/governance.rs` → `0` before edit
3. `grep -c 'DELETE FROM governance_messaging_config WHERE' migrations/2026-06-18-000000-0000_seed_rtc_enabled_config/down.sql` → `1`

### Validate — Windows e2e (delegated to laptop)

Write a `validate-pending-laptop-e2e` DQ entry with:
```json
{
  "kind": "validate-pending-laptop-e2e",
  "commands": ["cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/m3-core-infra-cr4-e2e.log 2>&1\""],
  "branch": "phase-m3-core-infra",
  "phase_task": "cr-4",
  "e2e_filter": "test(m3_actor_pseudonym)"
}
```
(Use `test(m3_actor_pseudonym)` so the filter catches both the old + any new test sharing the
prefix; if you name the new test without that prefix, broaden the filter accordingly.) Commit +
push, then **stop** — the laptop advisor runs the scoped e2e. End the commit body with a `LESSON:`
trailer. Commit subject: `fix(e2e,migration): cr-4 exercise actor-pseudonym route + cr-5 SQLFluff LT14 (fix-in-pr)`.
