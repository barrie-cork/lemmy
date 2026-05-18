# v1-ship-1-r2 planning brief (RE-PLAN — rebuild ONLY the e2e harness on `FederationConfig::to_request_data()`)

**Written**: 2026-05-18 by advisor session (lane-dedicated, CWD `C:/Users/barri/Developer/brehon-fork-ship-1`, worktree on `phase-v1-ship-1` @ `fc5359195`).
**Subagent target**: `planning` (Opus 4.7 — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/v1-ship-1-r2-planning-1` from `phase-v1-ship-1` committed HEAD (CAS-synced — advisor syncs the daemon-local `phase-v1-ship-1` ref to origin before dispatch). Plan file commits + pushes back to `phase-v1-ship-1` at finalize (this is a lane-dedicated re-plan of an in-flight phase, NOT a governance-v0 plan — the AGPL §13 code is already merged on the phase branch; only the e2e harness is being re-planned).
**Authority anchor**: `.claude/PRPs/prds/v1-ship-readiness.prd.md` §7.1 (Phase v1-ship-1 — AGPL §13 surface). ADR-011 (AGPLv3 inherited + source-disclosure required) governs. This re-plan does NOT change that surface — it changes ONLY how the single acceptance e2e test constructs its test App.

---

## 0. Why this is a RE-PLAN, and what is in vs out of scope (READ THIS FIRST)

A complete plan exists at **`.claude/PRPs/plans/v1-ship-1.plan.md`** (the `-r1` refresh is `v1-ship-1-r1.plan.md`; both are the audit trail — do NOT overwrite either). **The AGPL §13 surface code (DTO field, `build.rs` commit injection, `GET /api/v4/source` handler+route) is PROVEN CORRECT and ALREADY MERGED on `phase-v1-ship-1`** — Tasks 1-4 shipped, every §5.2 Phase-1 (cargo-check + clippy `-D warnings` + cargo-test `--no-run`) is GREEN on the current tip, and the production code is not in question.

**The ONLY defect is the single acceptance e2e test's App construction.** The test `agpl_source_disclosure_surface_returns_notice` in `crates/server/tests/e2e.rs` HTTP-drives `GET /api/v4/site` via a **hand-assembled `actix_web::test::init_service(App::new()...)`** and returns HTTP 500 `"Requested application data is not configured correctly. View/enable debug logs for more details."`. Five successive impl attempts (the original §13 e2e task → fix-impl-5 → fix-impl-6 → fix-impl-7 → fix-impl-7b → fix-impl-8) failed against this same actix-Data-500. fix-impl-8 mirrored the COMPLETE real-server middleware stack (`FederationMiddleware` + `IdempotencyMiddleware` + `SessionMiddleware`, `FederationConfig` via the canonical in-file builder idiom) — §5.2 Phase-1 PASSED (it type-checks) but §5.2 Phase-2 e2e FAILED with the **byte-identical** actix-Data-500 (DQ #255). §G4 hard-refusal fired (2nd same-tuple `(actix-Data-500, e2e.rs)` post-fix-impl-8; override-scope DQ #247/#249/#252 exhausted). The user directed (DQ #256, `answered_by: user`): advisor investigates, then authors this re-plan brief.

**The advisor's read-only investigation is COMPLETE and the root cause + the known-good pattern are VERIFIED (do NOT re-derive — APPLY).** See §0.1.

### 0.0 Scope fence (HARD)

**LOCKED — carry forward verbatim, do NOT re-plan, do NOT re-touch:**
- Plan §1-§9, §12, §16, §17, §18, §19, §20 design narrative.
- §13 **Tasks 1, 2, 3, 4** (the `GetSiteResponse.source_disclosure` field + its constructor update; the `build.rs` `BREHON_FORK_COMMIT` injection; the `get_source` handler + `/api/v4/source` route). These are MERGED and PROVEN. The re-plan's §13 carries them as **already-shipped context** (one short "carried-forward, merged at <tip>, no re-impl" note per task) — NOT as tasks to re-execute.
- §10.1 (`GetSiteResponse` DTO mirror), §10.2 (`read_site` handler-population mirror), §10.3 (`federated_instances.rs` handler mirror), §10.4 (site scope route mirror) — the production-code mirrors are settled; the re-plan does not re-derive them (cite "settled, merged" and move on).

**IN SCOPE — the entire re-plan effort is exactly this:**
- **§10.5 / §10.6 / §10.7 — replace the e2e-test-harness mirror.** The old plan's e2e test built the App by hand-assembling middleware (the wrong-shaped approach proven to fail 5×). The new plan's e2e test MUST construct its request context via **`activitypub_federation::config::FederationConfig::to_request_data()`** (the `LemmyContext::init_test_context()` idiom — see §0.1), mirroring the REAL server's composition (`lib.rs:247` + `lib.rs:364`), so the `Data<LemmyContext>` the `get_site` handler extracts IS the federation request-data, by construction — not by hand-mirrored `.wrap(FederationMiddleware::...)` order.
- **§13 the single e2e task** — rewrite ONLY the e2e task (the one that adds `agpl_source_disclosure_surface_returns_notice`). New IMPLEMENT body, new MIRROR refs (the `init_test_context` idiom + the real-server `lib.rs:247/364` composition + the verified canonical in-file `FederationConfig::builder()` site), new GOTCHAs, new VALIDATE checkpoint. **Preserve fix-impl-6 Part A (body-on-failure asserts) and Part B (complete `SiteInsertForm`) as REQUIRED elements of the new test** — they are correct and were twice-decisive in making the failure legible; the new test keeps them.
- **§4 watchpoints, §5 complexity, §16a Stories** — re-derive ONLY as they pertain to the e2e task (Stories 1+2 for the merged surface stay as-is conceptually; Story 3 — the e2e — is the one whose checkpoint changes).
- **§14 / §15 DoD** — confirm the §5.2 validate-pending-laptop shape is current (Shape G is SUSPENDED repo-wide per DQ #229 until 2026-06-01 — see §0.2). The e2e DoD command is `cmd //c scripts\brehon\cargo-test.bat --workspace --test e2e --features full` (the proven §5.2 Phase-2 invocation) — re-confirm, do not invent.

**If the planner finds the production code (Tasks 1-4) needs ANY change to make the e2e pass → STOP and file a `kind: "blocker"` DQ.** The investigation (§0.1) refuted that — the 500 is the test harness's federation request-data registration, NOT the site row and NOT the handler. A finding that production code must change is an advisor/user decision, not a planner re-derivation.

### 0.1 Investigation findings — the VERIFIED root cause + the known-good pattern (BINDING — APPLY, do NOT re-derive)

The advisor performed a read-only deep-dive (Explore subagent + `ref-context` + direct `Read` verification of every load-bearing claim). These are **binding inputs to the plan**:

**Finding 1 — there is an existing canonical in-workspace test-context fixture the e2e test is NOT using.** `crates/api/api_utils/src/context.rs:65-100` (verified by direct Read):

```rust
pub async fn init_test_federation_config() -> FederationConfig<LemmyContext> {
    let pool = build_db_pool_for_tests();           // runs migrations
    let client = client_builder(&SETTINGS).build().expect("build client");
    let client = ClientBuilder::new(client).build();
    let secret = Secret { id: 0, jwt_secret: String::new().into() };
    let rate_limit_cell = RateLimit::with_debug_config();
    let context = LemmyContext::create(pool, client.clone(), client, secret, rate_limit_cell.clone());
    FederationConfig::builder()
      .domain(context.settings().hostname.clone())
      .app_data(context)                            // FederationConfig OWNS the context
      .debug(true)
      .http_fetch_limit(0)
      .build().await.expect("build federation config")
}
pub async fn init_test_context() -> Data<LemmyContext> {
    let config = Self::init_test_federation_config().await;
    config.to_request_data()                        // ← returns Data<LemmyContext> == the request-time extension
}
```

`FederationConfig::to_request_data()` returns a `Data<LemmyContext>` that **IS** the request-time extension `FederationMiddleware` would otherwise insert. This is the canonical Lemmy test idiom for an HTTP-route test.

**Finding 2 — the real server's composition (verified `Read` of `crates/server/src/lib.rs:225-386`):**
- `lib.rs:228-241` — builds `federation_config`; its `.app_data(context.clone())` makes `FederationConfig` *own* the context.
- `lib.rs:247` — `let request_data = federation_config.to_request_data();`
- `lib.rs:349-386` `create_http_server(federation_config, ...)` — the App is built INSIDE `HttpServer::new(move || { ... })` where:
  - `lib.rs:364` — **`let context: LemmyContext = federation_config.deref().clone();`** ← the App's context comes *out of* the FederationConfig (NOT an independently-built one).
  - `lib.rs:379-382` — `.app_data(Data::new(context.clone()))` + `.wrap(FederationMiddleware::new(federation_config.clone()))` + `.wrap(IdempotencyMiddleware::new(idempotency_set.clone()))` + `.wrap(SessionMiddleware::new(context.clone()))`.

The load-bearing asymmetry the 5 fix-impls all missed: **the real server's App context and its `FederationConfig` are ONE object** (`context` is `federation_config.deref().clone()`). The failed e2e test obtained `context` from `governance_fixtures::bootstrap()` (a *separate* `Data<LemmyContext>` built from a *different* pool) and then built a *different* `FederationConfig` from `(**context).clone()` — so the `Data<LemmyContext>` the handler extracts and the federation request-data are different objects. Hand-wrapping `FederationMiddleware` does not reconcile that; the canonical `to_request_data()` path does (by construction).

**Finding 3 — `get_site` extractor chain (verified `Read` of `crates/api/api_crud/src/site/read.rs:24-27`):** `pub async fn get_site(local_user_view: Option<LocalUserView>, context: Data<LemmyContext>) -> LemmyResult<Json<GetSiteResponse>>`. It extracts the **actix `web::Data<LemmyContext>`** (the value `to_request_data()` / `FederationMiddleware` registers). It does NOT itself take an `activitypub_federation::config::Data<T>`. So the fix is purely "make the registered `Data<LemmyContext>` be the federation request-data," which `to_request_data()` does directly.

**Finding 4 — secondary hypothesis REFUTED.** The incomplete-`site`-row Diesel-deserialization theory (from the original plan) is NOT the cause: `crates/routes/src/utils/setup_local_site.rs` populates all ActivityPub fields, and the failed test (fix-impl-6 Part B) already replicates that complete scaffold. The 500 is the federation request-data registration, full stop. The new plan must NOT chase the site-row theory.

### 0.1.1 The CENTRAL design decision the planner MUST resolve (this is the re-plan's core work)

`init_test_context()` builds its **own** pool/context via `build_db_pool_for_tests()`. The failed e2e test relied on `governance_fixtures::bootstrap()` for its **testcontainer Postgres + the AGPL-surface seed** (instance / sysacct Person / complete `Site` / `LocalSite` / `LocalSiteRateLimit`). These two context-acquisition paths must be **reconciled** so that the pool the test SEEDS into is the SAME pool the request-time `Data<LemmyContext>` the handler extracts is bound to. The planner MUST design and specify ONE of (decide with quoted source evidence, state the chosen option + rejected option + why in plan §8 + §10.7):

- **Option (a) — seed into the `init_test_context()`-derived context's pool.** Call `LemmyContext::init_test_context()` (or `init_test_federation_config().to_request_data()`); obtain a pool handle from that returned `Data<LemmyContext>` (verify the accessor — likely `.pool()` / `.inner_pool()` on the deref'd context); run the AGPL-surface seed against THAT pool; build the App with that `Data<LemmyContext>` as `.app_data(...)`. Question the planner must resolve from source: does `init_test_context()`'s `build_db_pool_for_tests()` give a usable testcontainer-or-equivalent DB that the seed can write to and the handler can read from in-process? (Read `build_db_pool_for_tests` — likely in `crates/db_schema` or `api_utils` — and confirm what DB it provisions; this determines whether (a) is viable without `governance_fixtures::bootstrap()`'s container.)
- **Option (b) — thread the `bootstrap()` context THROUGH `FederationConfig`.** Keep `governance_fixtures::bootstrap()` for the testcontainer + seed; then build `FederationConfig::builder().domain(...).app_data((**bootstrap_context).clone()).debug(true).http_fetch_limit(0).build().await?` and call `.to_request_data()` on it to obtain the `Data<LemmyContext>` to register as `.app_data(...)` — so the registered context derives from the SAME `bootstrap()` context the seed wrote into, via the canonical idiom (mirrors `lib.rs:241→247→364` exactly: build config from the context, then context-out-of-config). The App still wraps `FederationMiddleware::new(federation_config.clone())` + Idempotency + Session per `lib.rs:379-382` (those are correct; the fix is that the `.app_data` `Data<LemmyContext>` is now `federation_config.to_request_data()`, NOT a hand-cloned `Data::new(context.clone())` that is a different object from the federation config's inner context).

**Advisor's prior (NOT binding — the planner decides with source evidence):** option (b) is the smaller, lower-risk change (it preserves the proven `governance_fixtures::bootstrap()` testcontainer + the fix-impl-6 Part B complete-seed, and changes ONLY how the `Data<LemmyContext>` is derived — from `federation_config.to_request_data()` instead of a hand-cloned `Data::new`), and it mirrors the real-server `lib.rs:241→247→364` composition most literally. Option (a) is cleaner conceptually but hinges on whether `build_db_pool_for_tests()` provisions a seed-able DB without `bootstrap()`'s container — if it does NOT, (a) is not viable and (b) is mandatory. The planner MUST read `build_db_pool_for_tests` + `governance_fixtures::bootstrap()` + the `FederationConfig` `to_request_data` API and choose on evidence. **If neither option cleanly works (e.g. `to_request_data()` consumes/moves the config in a way incompatible with also wrapping `FederationMiddleware`, or the pool handles can't be shared) → `kind: "blocker"` DQ; do not guess.**

### 0.1.2 Clarify-pass resolutions (READ BEFORE §1 — these RESOLVE four residual ambiguities; canonical sibling shape per `v1-ship-1-r1-planning-1.md` §0.1)

The advisor ran `/brehon-clarify` on this brief (2026-05-18, advisor-mode). Four clarify-DQ entries are RESOLVED and are **binding inputs to the plan** — they pre-empt four round-trips and supersede the corresponding "the planner must discover…" language in §0.1.1 / §2.2 item 1 / §3 item 11 / §4.1 watchpoint #2. The Option (a)/(b) **design decision itself remains the planner's evidence-based call** (DQ #257 narrows the inputs but does NOT pre-decide a-vs-b).

- **DQ #257 (RESOLVED, advisor) — `build_db_pool_for_tests()` location + DB-provisioning model.** It is at **`crates/diesel_utils/src/connection.rs:197`** (verified Read), NOT `crates/db_schema`/`api_utils` (the §0.1.1 / §3 item 11 path hint is **corrected** — the planner cites `crates/diesel_utils/src/connection.rs:197` in §10.7). It calls `build_db_pool()` which connects to the Postgres at **`LEMMY_DATABASE_URL`** — it does **NOT** provision its own testcontainer. Binding implication for §0.1.1: Option (a) is viable ONLY if `governance_fixtures::bootstrap()`'s testcontainer is running and `LEMMY_DATABASE_URL` points at it BEFORE `init_test_context()` is called (so seed + request-time context share the same env-resolved pool by construction). The planner MUST read `governance_fixtures::bootstrap()` to confirm whether it sets `LEMMY_DATABASE_URL` as a process env the subsequent `build_db_pool_for_tests()` reads, or returns a pool handle the test threads explicitly — then choose Option (a)/(b) on that evidence per §0.1.1. Discovery is narrowed (correct location + model given); a-vs-b is NOT pre-decided; a `kind: "blocker"` is still correct if the bootstrap()/env ordering is genuinely ambiguous after the source read.
- **DQ #258 (RESOLVED, advisor) — `to_request_data()` borrows, does NOT consume.** Verified by direct Read of `activitypub_federation-0.7.0-beta.11/src/config.rs:143`: `pub fn to_request_data(&self) -> Data<T> { Data { config: self.clone(), … } }` — **borrows `&self`**, clones the config internally; `FederationConfig<T>: Deref<Target=T>` at `config.rs:264-270`. Production proof: `lib.rs:247` calls `.to_request_data()` then `lib.rs:262/380` reuse the same `federation_config` binding (`.wrap(FederationMiddleware::new(federation_config.clone()))`). Binding implication: §4.1 watchpoint #2 **stays** in §4 (planner cites `config.rs:143` as the SoT signature per `feedback_advisor_watchpoint_specificity`), but the **decision-relevant fact is SETTLED** — no consume-vs-borrow obstacle. §10.7's e2e construction mirrors `lib.rs:241→247→364→379-382` directly: derive `Data<LemmyContext>` via `.to_request_data()` for `.app_data(...)` AND `.wrap(FederationMiddleware::new(federation_config.clone()))` + Idempotency + Session from the same still-alive `federation_config`. The §0.1.1 / §4.1 escape-hatch branch "`to_request_data()` consumes self incompatibly with wrapping `FederationMiddleware`" is **empirically refuted** — the planner does NOT file a blocker on borrow-vs-consume grounds (the *pool-sharing* branch remains genuinely open per DQ #257).
- **DQ #259 (RESOLVED, advisor) — §15 Phase-1 cmd3 shape is correct AS WRITTEN.** §0.2's cmd3 `cargo-test.bat --no-run -p lemmy_server --test e2e` (NO `--features full`, `-p` not `--workspace`) is the proven §5.2-laptop trio: DQ #254 (fix-impl-8 Phase-1) + DQ #250 (fix-impl-7b Phase-1) both record it PASSING (cargo-test --no-run 2m01s 0err, e2e binary linked, 0 E0308). `lemmy_server` has NO `full` feature (`-p lemmy_server --features full` would trip `feedback_features_full_p_crate_incompatible`); the e2e target resolves `full`-gated code transitively. Binding implication: the planner transcribes the §0.2 Phase-1 trio + Phase-2 e2e command **VERBATIM** into §15 and does **NOT** normalize cmd3 to `--workspace --features full`; cites §0.2 + DQ #254 + the §0.2 'Shape G suspended per DQ #229 until 2026-06-01' as the §15 authority.
- **DQ #260 (RESOLVED, advisor) — no cross-lane file-ownership conflict on `crates/server/tests/e2e.rs`.** Verified: `git diff --stat governance-v0...origin/phase-v1-federation-inbound-a -- crates/server/tests/e2e.rs` is EMPTY (that phase's tasks 2-3 are `schema.rs` + `config.rs` federation.inbound.* keys, DQ #242/#243/#244). Other worktrees: `tooling-local-validation` (infra), canonical `governance-v0` (meta-edits). Binding implication: the single §13 e2e task has SOLE ownership of the `agpl_source_disclosure_surface_returns_notice` fn region across all active lanes — the plan does **NOT** add a cross-phase file-ownership watchpoint in §4 and the planner does **NOT** file a blocker on cross-lane-contention grounds. (Intra-file Edit-size discipline — §4.1 watchpoint #6 / `feedback_junior_worker_e2e_edit_hang`: one targeted anchored Edit of the single fn, never a full-file Edit — is unchanged and already in the brief.)

These four resolutions are binding; all other §0.1.1 / §2.2 design work (especially the Option (a)/(b) decision) remains the planner's, to be resolved against current-tip source as the brief specifies.

### 0.2 Validation context (binding)

- **Shape G is SUSPENDED repo-wide (DQ #229) until 2026-06-01.** ALL cargo validation runs LOCALLY via §5.2 validate-pending-laptop on the canonical checkout `C:/Users/barri/Developer/brehon-fork`. The plan's §15 DoD MUST use the §5.2 validate-pending-laptop shape (the proven Phase-1 trio + the Phase-2 e2e command), NOT the Shape-G per-workflow shape. Phase-1 trio: `cmd //c scripts\brehon\cargo-check.bat --workspace --features full`; `cmd //c scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings`; `cmd //c scripts\brehon\cargo-test.bat --no-run -p lemmy_server --test e2e`. Phase-2 e2e: `cmd //c scripts\brehon\cargo-test.bat --workspace --test e2e --features full`. (The original plan's §15 is Shape-G-shaped — the re-plan MUST convert §15 to the §5.2-laptop shape; cite this §0.2 as the authority.)
- **Lane discipline:** this re-plan's plan file commits to `phase-v1-ship-1` (the lane), NOT `governance-v0` — the AGPL code is already on the phase branch and the e2e test will be added there by a subsequent impl-task. The planner's worktree is cut from `phase-v1-ship-1`.

---

## 1. Role + dispatch line

`[role:planning] v1-ship-1-r2 re-plan — rebuild e2e harness on FederationConfig::to_request_data`

The actual create-task description (single line, <100 chars, per `.claude/rules/advisor-orchestrator.md` Junior task description template):

```
[role:planning] v1-ship-1-r2 re-plan — see .claude/PRPs/briefs/v1-ship-1-r2-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at **`.claude/PRPs/plans/v1-ship-1-r2.plan.md`** for sub-phase **v1-ship-1** (the `-r2` suffix marks the second re-plan; do NOT overwrite `v1-ship-1.plan.md` or `v1-ship-1-r1.plan.md` — both stay as the audit trail per `.claude/rules/no-destructive-defaults.md`).

### 2.1 Carry forward verbatim (LOCKED — do NOT re-litigate; cite "merged, settled")

Per §0.0 scope fence: §1-§9/§12/§16-§20 design narrative; §13 Tasks 1-4 as already-shipped context (one-line "merged at <tip>, no re-impl" each — NOT re-executed); §10.1-§10.4 production-code mirrors as settled. The AGPL §13 surface (two endpoints, cache-safety, `build.rs` fork-commit, scope boundary, PRD §7.1 acceptance) is UNCHANGED.

### 2.2 What you MUST re-derive (the actual re-plan work — ALL of it is the e2e test)

1. **Resolve §0.1.1's central design decision (Option (a) vs (b)).** Read `build_db_pool_for_tests` (find it — `grep -rn "fn build_db_pool_for_tests" crates/`), `LemmyContext::init_test_context` / `init_test_federation_config` (`context.rs:65-100`, already cited — re-Read to confirm at current tip), `governance_fixtures::bootstrap()` (`grep -n "pub async fn bootstrap" crates/server/tests/e2e.rs` — confirm current line + signature + exactly which `mod` it is in; the r1 brief flagged a dual-`bootstrap()` — re-confirm which one the AGPL test uses and its full return tuple), and the `activitypub_federation` `FederationConfig` / `to_request_data` API (it is `activitypub_federation-0.7.0-beta.10` or `.11` per `Cargo.lock` — confirm the exact version, then read its `src/config.rs` `to_request_data` signature from the cargo registry cache: `~/.cargo/registry/src/*/activitypub_federation-*/src/config.rs` — confirm whether `to_request_data(&self)` borrows or `to_request_data(self)` consumes, since that determines whether the App can ALSO `.wrap(FederationMiddleware::new(federation_config.clone()))` after calling it). State the chosen option in §8 + §10.7 with the rejected option + the source-quoted reason.
2. **Re-derive §10.5 / §10.6 / §10.7 (the e2e-harness mirror).** §10.7 becomes the canonical `init_test_context()` / `to_request_data()` idiom (quote `context.rs:65-100` + the real-server `lib.rs:241/247/364/379-382` composition verbatim as the pattern-to-mirror). §10.5/§10.6 (the e2e outer-shape + in-process-HTTP mirror): re-confirm the CURRENT canonical e2e error-shape from a recent passing sibling test post-LemmyResult-unification (the test fn returns `lemmy_utils::error::LemmyResult<()>`; cite the case A/B/C per `feedback_lemmy_error_no_std_error.md` — the `FederationConfig::builder()...build().await?` uses a bare `?` since the fn is `LemmyResult<()>`; mirror the verified canonical in-file `FederationConfig::builder()` site — `grep -n "FederationConfig::builder" crates/server/tests/e2e.rs`, READ the first hit's full block, that is the byte-for-byte mirror for the builder).
3. **Rewrite the single §13 e2e task.** It REPLACES the old e2e §13 task entirely. Its FILES block: `modifies: [crates/server/tests/e2e.rs]` only; `requires:` Tasks 1+4 (the DTO field + the `/api/v4/source` route must exist — they do, on the phase branch; note "merged"). Its IMPLEMENT body specifies: (i) the chosen context-acquisition path (Option a or b) with the exact construction lines mirroring `context.rs`/`lib.rs`; (ii) **preserve fix-impl-6 Part A** — body-on-failure asserts for BOTH `/api/v4/site` and `/api/v4/source` (`let status = resp.status().as_u16(); let body = test::read_body(resp).await; assert_eq!(status, 200, "... — body: {}", String::from_utf8_lossy(&body));`) — these are REQUIRED, mirror the proven in-file pattern; (iii) **preserve fix-impl-6 Part B** — the complete `SiteInsertForm` (with `ap_id`/`inbox_url`/`public_key`/`private_key`/`last_refreshed_at` + `generate_actor_keypair`) and the full instance/sysacct/LocalSite/LocalSiteRateLimit seed (this seed is correct; only WHERE it is seeded — which pool — changes per the Option a/b decision); (iv) the App builder per the chosen option mirroring `lib.rs:379-382` wrap order; (v) the two `test::TestRequest::get().uri(...)` calls + the disclosure-block assertions per PRD §7.1. The test fn name is **`agpl_source_disclosure_surface_returns_notice`** verbatim (PRD §7.1 + §16a Story 3 + `/brehon-verify` greps this literal — GOTCHA). Single anchored Edit (one append/replace of the existing fn region; e2e.rs is ~14.9k lines — per `feedback_junior_worker_e2e_edit_hang.md` the impl-task makes a targeted anchored Edit into the single fn, NEVER a full-file Edit; cite the anchor).
4. **§15 DoD → §5.2-laptop shape** (per §0.2). Phase-1 trio + Phase-2 e2e command verbatim as in §0.2. Remove the Shape-G per-workflow shape. Add the §0.2 authority citation.
5. **§4 watchpoints** — re-derive the e2e-relevant ones (seed list §4.1 below). The merged-surface watchpoints (Stories 1+2) may be cited "settled — merged"; the e2e watchpoints are the live ones.
6. **§5 complexity** — re-derive. The surface is merged; the residual work is ONE e2e test rebuilt on a known-good idiom. Score should be LOW (~3/10, Sonnet 4.6 target, split-DQ threshold `>8`) — but compute honestly; if the Option-a/b reconciliation proves to need a non-trivial pool-sharing mechanism, say so (still likely a blocker-or-low, not a high score).
7. **§16a Stories** — Story 1 (`source_disclosure` on `GetSiteResponse`) + Story 2 (`/api/v4/source` body) are MERGED — mark their checkpoints "satisfied on phase branch @ <tip>" (the planner cites the current phase tip). Story 3 (the agpl e2e asserting BOTH surfaces) is the ONLY live story; its checkpoint is the §5.2 Phase-2 e2e green + the `/brehon-verify` grep for the literal test name. The plan's §16a must make explicit that ONLY Story 3 has outstanding impl.

### 2.3 Plan file deliverable shape

- **One file:** `.claude/PRPs/plans/v1-ship-1-r2.plan.md`.
- **Follow `.claude/PRPs/templates/plan.template.md` 20-section schema literally.** §16a Stories mandatory.
- **§5.2-laptop DoD shape** (Shape G suspended — §0.2). Inline cargo invocations forbidden EXCEPT the explicit §5.2-laptop `cmd //c scripts\brehon\cargo-*.bat ...` commands (those ARE the §5.2 shape, not raw cargo).
- **§5 complexity score breakdown table** per `feedback_complexity_score_pre_split.md`.
- **§4 watchpoints** every entry cites a SPECIFIC file:line / handler / API at CURRENT tip (per `feedback_advisor_watchpoint_specificity.md`).
- **§6 "Relationship to v1-ship-2 / v1-ship-3"** — UNCHANGED (carry forward; v1-ship-1 ships ONLY the §13 surface).
- **One commit at finalize:** `feat(plan): v1-ship-1-r2 sub-phase plan (e2e harness rebuilt on FederationConfig::to_request_data)`.

**Commit only the plan file** (and any planner DQ entries, pushed immediately per mid-task visibility). Plan-file commit pushes to `phase-v1-ship-1` after the advisor's DoD smoke + watchpoint-specificity gates pass and the user approves (User Gate 1).

---

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract.
2. `.claude/PRPs/templates/plan.template.md` — canonical 20-section plan schema.
3. **`.claude/PRPs/plans/v1-ship-1.plan.md`** AND **`.claude/PRPs/plans/v1-ship-1-r1.plan.md`** — the parked + refreshed plans. Read §1-§9/§12/§16-§20 (carried-forward design — do NOT change) and §10.5/§10.6/§10.7 + the e2e §13 task (the ONLY parts you re-derive). Note where r1 already refreshed citations vs r-original.
4. **`.claude/PRPs/briefs/v1-ship-1-r1-planning-1.md`** — the prior re-plan brief (canonical sibling shape, per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate). Mirror its structural discipline (the "carry design / re-derive evidence" pattern), adapted to "carry surface / rebuild e2e harness".
5. `.claude/PRPs/prds/v1-ship-readiness.prd.md` §1, §2, §4, §7.1 (the acceptance the e2e test asserts), §7.2/§7.3 (what is DEFERRED — keep scope tight).
6. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — **ADR-011** (governs); confirm no ADR contradicts the test approach.
7. **`crates/api/api_utils/src/context.rs:1-101`** — READ IN FULL. `init_test_federation_config` (`:65-96`) + `init_test_context` (`:97-100`) are the canonical idiom to mirror in §10.7. Confirm the exact imports + the `Data<LemmyContext>` type path (`activitypub_federation::config::Data` vs `actix_web::web::Data` — note which `Data` `to_request_data()` returns).
8. **`crates/server/src/lib.rs:225-390`** — READ IN FULL. The real-server composition: `:228-241` (federation_config build), `:247` (`to_request_data`), `:349-386` `create_http_server` (esp. `:364` context-out-of-config + `:379-382` the App `.app_data`+`.wrap` stack). This is the §10.7 real-server mirror — quote it verbatim.
9. **`crates/api/api_crud/src/site/read.rs:1-90`** — `get_site` (`:24-27` — the extractor chain; verified `Data<LemmyContext>` actix) + `read_site` + the `CacheLock`/`LazyLock` (confirm current lines; the cache is build-time-stable for `source_disclosure` — settled, but re-confirm the agpl test's first `/api/v4/site` call isn't defeated by a stale cache from a sibling test under `--test-threads=1`; if the cache could be cross-test-contaminated, that is a §4 watchpoint).
10. **`crates/server/tests/e2e.rs`** — ~14.9k lines. Via `grep -n` then READ the block: every `pub async fn bootstrap()` (confirm which `mod`, current line, full signature + return tuple — the AGPL test uses `governance_fixtures::bootstrap()`; confirm); the FIRST `FederationConfig::builder(` site (the byte-for-byte builder mirror); the current `agpl_source_disclosure_surface_returns_notice` fn IN FULL (the region the new §13 task replaces — note it currently has fix-impl-6 Part A + Part B + fix-impl-7/7b/8's middleware wraps; the new task KEEPS Part A + Part B, REPLACES the App-construction); a recent PASSING sibling test that HTTP-drives a route (if any exists post-investigation — the investigation found NO other test HTTP-drives a `Data<LemmyContext>` route, so the canonical mirror is `init_test_context()` + the real server, NOT a sibling e2e; state this explicitly in §10.7 — "no in-workspace sibling HTTP-route test exists; mirror the production `to_request_data()` composition").
11. `grep -rn "fn build_db_pool_for_tests" crates/` then READ it — determines Option (a) viability (does it provision a seed-able DB without `governance_fixtures::bootstrap()`'s testcontainer?).
12. `Cargo.lock` — confirm the exact `activitypub_federation` version; then `~/.cargo/registry/src/*/activitypub_federation-<ver>/src/config.rs` — READ `to_request_data` (borrows `&self` or consumes `self`?) + the `FederationConfig` `Data<T>` `Deref` target. Also `src/actix_web/middleware.rs` `FederationMiddleware::call` (what extension it inserts) for §10.7 completeness. (Per `feedback_verify_files_with_read.md` — verify the API, do not assume.)
13. `crates/routes/src/utils/setup_local_site.rs` — READ (confirms Finding 4: the complete-`SiteInsertForm` shape fix-impl-6 Part B already mirrors; the new test keeps Part B — do NOT re-chase the site-row theory).
14. **Glob `.claude/lessons/feedback_*.md` and Read** every name-keyword match: `lemmy_error_no_std_error`, `junior_worker_e2e_edit_hang`, `async_pool_test_pattern`, `e2e_filter`, `read_canonical`, `verify_files_with_read`, `build_what_tests_exercise`, `test_target_compile_validation`, `complexity_score`, `pre_phase_dod`, `plan_dod_dry_run`, `advisor_watchpoint_specificity`, `plan_baseline_self_reference`, `principles_not_rules`, `windows_e2e_requires_bat_wrapper`, `laptop_default_for_validate_pending`. (Per `.claude/rules/advisor-orchestrator.md` §2.4 — planning brief, so the impl-file-class table does not auto-fire, but these are the lessons the PLAN bakes into §4/§10/§13.)
15. **Glob `.claude/lessons/reference_*.md` and Read** any match: `phase_branch`, `prp_commands`.
16. `.claude/PRPs/reports/v1-ship-1-fix-impl-8.md`-adjacent context: the fix-impl-8 brief `.claude/PRPs/briefs/v1-ship-1-fix-impl-8.md` (read for awareness — it documents the wrong-shaped hand-assembled approach the re-plan must NOT repeat) + the runlog `.claude/runlog/v1-ship-1-runlog.md` tail (the §G4 hard-refusal FAIL block + the 5-attempt history). Do NOT transcribe; read for "what was tried and why it failed."

---

## 4. Constraints (hard rules — violating any is a process breach)

### 4.1 Plan-content discipline

- **Surface design is LOCKED; ONLY the e2e harness is re-derived.** Do NOT change the AGPL §13 surface, the two-endpoint shape, the cache-safety reasoning, the `build.rs` approach, or the scope boundary. Do NOT re-execute Tasks 1-4 (merged). If you believe the surface design itself (not the e2e harness) must change → STOP, `kind: "blocker"` DQ — advisor/user decision.
- **The e2e harness MUST use `FederationConfig::to_request_data()` (the `init_test_context()` idiom).** The hand-assembled `.wrap(FederationMiddleware::...)` approach is PROVEN-WRONG (5 failed attempts; §0.1). The plan does NOT propose another hand-assembly variant. Per `feedback_read_canonical_before_writing_spec.md` — mirror the EXISTING canonical fixture (`context.rs:65-100`) + the real-server composition (`lib.rs:241/247/364/379-382`), not a new invented shape.
- **Preserve fix-impl-6 Part A + Part B in the new test** (body-on-failure asserts; complete `SiteInsertForm` + keypair + full seed). They are correct and were twice-decisive. The new test KEEPS them; only the App-construction + which-pool-is-seeded changes.
- **Every `file:line` / API in the new plan MUST be verified by an actual Read at current tip.** Per `feedback_verify_files_with_read.md` + `pattern_verify_before_trusting_shell_output` — a `grep -n` hit is a pointer; READ the block. Per `feedback_plan_baseline_self_reference.md` — cite by grep-able symbol + ancestry where a stable reference is needed.
- **§5.2-laptop DoD shape** (Shape G suspended — §0.2). The §15 commands are the §0.2 verbatim list.
- **§16a Stories** — Stories 1+2 "satisfied/merged @ <tip>"; Story 3 (agpl e2e) the only live story.
- **§4 watchpoints cite specific file:line / API at current tip** (per `feedback_advisor_watchpoint_specificity.md`). **Seed list (≥6):**
  1. **Context-acquisition path** — the chosen Option (a)/(b); cite the exact `to_request_data()` call site shape + which pool the seed writes (the wrong reconciliation re-introduces the actix-Data-500).
  2. **`to_request_data()` borrow-vs-consume** — cite the `activitypub_federation` `config.rs` signature; if it consumes `self`, the App cannot also `.wrap(FederationMiddleware::new(federation_config.clone()))` from the same binding — the watchpoint names how the plan sequences the clone (mirror `lib.rs:241→247→clone-into-App`).
  3. **`bootstrap()` selection** — confirm the AGPL test uses `governance_fixtures::bootstrap()` (cite current `mod`+line+return tuple); the impl-task must not pick the sibling `bootstrap()` (the r1 dual-`bootstrap()` watchpoint carries forward).
  4. **fix-impl-6 Part A/B preservation** — the new test MUST retain body-on-failure asserts + the complete `SiteInsertForm`/keypair/seed; cite the current fn line range; the impl-task must not drop them when rewriting the App-construction.
  5. **e2e canonical error-shape** — the fn is `lemmy_utils::error::LemmyResult<()>`; `FederationConfig::builder()...build().await?` uses bare `?` (Case A per `feedback_lemmy_error_no_std_error.md`); cite the verified canonical in-file builder site as the byte-for-byte mirror.
  6. **e2e Edit-size discipline** — one targeted anchored Edit replacing the single `agpl_*` fn's App-construction region (~14.9k-line file); per `feedback_junior_worker_e2e_edit_hang.md` NEVER a full-file Edit; cite the anchor (the fn signature line).
  7. **read_site cache cross-test contamination** — if `CacheLock<GetSiteResponse>` could carry a stale entry from a sibling test under `--test-threads=1`, the agpl test's first `/api/v4/site` call might read a pre-`source_disclosure` cached value; confirm from `read.rs` whether the cache is TTL-0/disabled in debug (the original plan claimed so — re-confirm) and watchpoint it.
- **§5 complexity score breakdown table mandatory.** Re-derive; expected ~3/10 (residual = one e2e test on a known-good idiom).
- **R-rule inheritance** from prior retros applies (R9 struct-field caller-enumeration was for the merged Task 1 — settled; the live R-discipline here is "mirror the canonical fixture, don't invent").

### 4.2 Decision-queue discipline

- **Attribution integrity.** `from: "planner"` or `null`. NEVER `"advisor"`, `"user"`, `"impl-self-resolved"`, `"clarify"` (clarify is advisor-only).
- **Mid-task DQ commits push immediately** to the worker branch (per `.claude/rules/decision-queue.md` "Mid-task visibility").
- **Boundary-of-judgment — STOP and `kind: "blocker"` DQ rather than guess when:**
  - Neither Option (a) nor (b) cleanly works (e.g. `to_request_data()` consumes `self` AND the App must wrap `FederationMiddleware` from a separate binding the plan can't cleanly produce; or `build_db_pool_for_tests()` provides no seed-able DB AND `bootstrap()`'s context can't be threaded through `FederationConfig`) — §0.1.1.
  - The investigation's Finding 1-4 appear contradicted by what you Read at current tip (e.g. `init_test_context()` no longer exists / changed shape; `get_site` extractor chain differs from `read.rs:24-27`).
  - The surface design itself (not the e2e harness) appears to need change — §4.1.
  - Any §3-cited file no longer exists / was renamed.
- **Do NOT file `kind: "clarify"`** — advisor-only. The advisor runs `/brehon-clarify` on THIS brief after it commits; you only write `blocker` or `log`.

### 4.3 File ownership

- **Touch only:**
  - `.claude/PRPs/plans/v1-ship-1-r2.plan.md` (CREATE — do NOT modify `v1-ship-1.plan.md` or `v1-ship-1-r1.plan.md`).
  - `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries; push immediately).
- **Do NOT edit anything under** `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, `.github/workflows/`, the parked plan files, or any other plan file.
- **One commit at finalize:** `feat(plan): v1-ship-1-r2 sub-phase plan (e2e harness rebuilt on FederationConfig::to_request_data)`.

### 4.4 Schema-first discipline

- **Read the relevant fixture/handler/API source at current tip BEFORE designing §10.7/§13.** Confirm at planning time: `init_test_context()`/`init_test_federation_config()` exist with the §0.1-quoted shape; the real-server `lib.rs:241/247/364/379-382` composition; `get_site`'s extractor chain (`read.rs:24-27`); the `to_request_data()` API signature; `governance_fixtures::bootstrap()`'s current line+return tuple. If any baseline assumption fails, `kind: "blocker"` DQ.

### 4.5 Cross-cutting from PMD-promoted patterns

- **`pattern_verify_before_trusting_shell_output`** — every grep hit confirmed by a direct Read.
- **`pattern_test_against_reality_not_syntax`** — the §10.7 verbatim blocks match current-tip source exactly; the plan's central claim (the `to_request_data()` idiom resolves the actix-Data-500) is grounded in the verified real-server composition, not asserted.
- **`pattern_cargo_feature_flag_propagation`** — §15 uses `--workspace --features full` (the §0.2 §5.2-laptop commands).
- **`feedback_read_canonical_before_writing_spec`** — mirror the CURRENT canonical fixture (`context.rs`) + production composition (`lib.rs`), NOT a new hand-assembled App.
- **`feedback_principles_not_rules`** — the Option (a)/(b) decision is the planner's judgment on source evidence; the brief gives the prior + the blocker boundary, not a forced answer.

### 4.6 Attribution integrity reminder

The only valid `answered_by` labels for a Junior planning subagent are `"planner"` or `null`. Never `"advisor"`, `"user"`, `"impl-self-resolved"`.

---

**Lean / advisor-side tip (not a constraint):** the entire value of this re-plan is *replacing one wrong-shaped test harness with the canonical in-workspace idiom that already exists*. The surface is merged and proven; do NOT re-open it. The 5 prior fix attempts all failed because they kept hand-assembling the actix App and hand-wrapping `FederationMiddleware` — none used `FederationConfig::to_request_data()`, which is exactly the call the real server uses (`lib.rs:247`) and exactly what `LemmyContext::init_test_context()` exists to provide. A high-quality plan reads `context.rs:65-100` + `lib.rs:241/247/364/379-382` + the `to_request_data` API, resolves the seed-pool reconciliation (Option a vs b) on source evidence, rewrites ONLY the single e2e task to mirror that composition, keeps fix-impl-6 Part A+B, and changes nothing else.

A second observation: the highest-risk planner error here is proposing a THIRD hand-assembly variant ("add one more middleware / change the wrap order again"). That is the proven-failed class. If the plan's e2e construction is not literally "build the `Data<LemmyContext>` via `FederationConfig::...build().await?.to_request_data()` mirroring `lib.rs`", it is wrong. The canonical idiom is non-negotiable; only the seed-pool reconciliation (a vs b) is a genuine design choice.

A third observation: this is the **hard ship-gate** (the fork is out of AGPL §13 compliance without a passing acceptance test). Correctness over speed. If the Option-a/b reconciliation is genuinely ambiguous after reading the source, a `kind: "blocker"` DQ is the right move — a guessed reconciliation that re-produces the actix-Data-500 costs another full §5.2 Phase-2 e2e cycle (~31 min) to discover.

---

_Brief author: advisor session (lane-dedicated, CWD `C:/Users/barri/Developer/brehon-fork-ship-1`, worktree on `phase-v1-ship-1` @ `fc5359195`, 2026-05-18). Brief committed on `phase-v1-ship-1` before the Junior planning task is queued. Per `.claude/rules/advisor-orchestrator.md` §3.3 clarify gate, the advisor runs `/brehon-clarify .claude/PRPs/briefs/v1-ship-1-r2-planning-1.md` after this brief commits + pushes; the planning task only queues after every clarify-DQ entry on this brief is resolved. Authority for the re-plan: DQ #256 (`answered_by: user` — "advisor investigates first, then re-plan brief"); the investigation findings (§0.1) are advisor-verified by direct Read and are binding inputs._
