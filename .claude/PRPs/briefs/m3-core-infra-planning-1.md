# m3-core-infra — planning brief

`[role:planning]` Author the M3 Phase-1 plan: deployable + fully-optional RTC stack, `rtc_enabled` flag (default false), bridge LiveKit JWT minting (identity→pseudonym at issue), `bridge_room` RTC-state columns.

## 1. Role + dispatch

`[role:planning]` — produce `.claude/PRPs/plans/m3-core-infra.plan.md` from the M3 sub-PRD Phase 1, following `.claude/commands/prp-plan.md` + `.claude/PRPs/templates/plan.template.md`. Plan-shaping only — author NO implementation code, NO `crates/**`, NO `services/bridge/**`. The plan is the deliverable.

## 2. Scope

Plan **all of M3-core-infra Phase 1 in one plan** (user decision 2026-06-18 — the natural deploy-unit, matches PRD Phase 1 boundary). Source: `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` §"Phase 1: M3-core infra" (lines 225–228) + §Technical Approach (lines 174–203).

The plan's §13 tasks cover (expected ~6, planner finalizes count + `[P]` markers + ordering):

1. **`rtc_enabled` Postgres config row + read path.** A NEW row in the existing `governance_messaging_config` typed-KV table (Postgres), default **`false`**, scope `instance`. This is a **config-row seed in a new migration** mirroring the `messaging_enabled` seed pattern — NOT a schema/column change. Pin `valid_from` to a stable literal for idempotent reruns. Add the matching `read_current` consumer if the bridge/binary needs to gate on it.
2. **`bridge_room` RTC-state columns** (SQLite, bridge-local) — `chair_id`, `queue_state`, `recording_config` (all `TEXT`, nullable). **Decision (user 2026-06-18): extend the embedded `CREATE TABLE IF NOT EXISTS` block in `services/bridge/src/bridge_room.rs::open()` + add idempotent `ALTER TABLE bridge_room ADD COLUMN <col> TEXT` guards for already-provisioned DBs.** Do NOT introduce a migration-runner framework — the bridge has zero `.sql` migration files today and uses embedded schema; follow that convention. (This resolves the PRD's "bridge-side migration" wording: the bridge has no migration system; the embedded-schema extension IS the bridge schema change.)
3. **LiveKit JWT mint fn (identity→pseudonym).** A bridge-side fn that mints a LiveKit access token. **Decision (user 2026-06-18): reuse-existing JWT approach** — LiveKit tokens are HS256 JWTs (HMAC-SHA256 over the documented `{iss: api_key, sub, video: {room, roomJoin}, exp, nbf}` claims), mintable with a single small dep (`jsonwebtoken` or `hmac`+`sha2`). Do NOT add the `livekit`/`livekit-api` SDK. **ADR-015 LOAD-BEARING (see §4): the JWT `sub`/`identity` claim MUST be the room pseudonym from the allocator, never `person_id`/username/email.**
4. **Docker sidecars** — LiveKit Server (Apache-2.0), lk-jwt-service (Apache-2.0), Element Call (AGPL-3.0) — added to the **bridge** compose stack (`services/bridge/docker-compose.yml`, after the `tuwunel` stanza), NOT `docker/docker-compose.yml` (which stays Lemmy-only). Gated so a governance-only instance (`rtc_enabled=false`) runs none of them. **MinIO is NOT in this phase** (it's Phase 5, gated by `record_town_halls`).
5. **AGPL-NOTICE rows** — add LiveKit / lk-jwt-service / Element Call rows to `AGPL-NOTICE.md` (three-bullet pattern at `AGPL-NOTICE.md:32–36`). ADR-011 source-disclosure obligation. **Advisor-owned `.claude/**`-adjacent meta-file** — actually `AGPL-NOTICE.md` is repo-root; the planner specifies it as an impl-task file, fine.
6. **`rtc_enabled=false` clean-posture story** — a tested success signal that the governance flow runs unchanged with the RTC stack absent / zero side-effects. Must be in §16a as its own story (NOT an untested assumption). PRD success criterion: `cargo test --test e2e` governance flow passes unchanged with RTC disabled; no LiveKit calls.

**Explicit boundaries — do NOT plan:** stage-mode / chair / FIFO queue / mic-passing (Phase 3), emergency-mute (Phase 4), MinIO / Egress / recording (Phase 5), the 3 chair/mute entry-kind consts (Phase 2 — ALREADY SHIPPED @ `a5fc60a2d`). Phase 1 mints a JWT and stands up the stack; it does NOT run a town hall.

## 3. Required reading

- `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` — §Phase 1 (225–228), §Technical Approach (174–203), §Success Criteria (132–151), ADR table (48–60), §Decisions Log D4/D5/D6 (271–273).
- `.claude/PRPs/handovers/m3-core-infra-bootstrap.md` — §4 watchlist (the six infra-specific items), §7 catch-fire, §"Stop-and-ask tripwires".
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` — canonical schema; `governance_messaging_config` shape + the `read_current` view.
- ADR-015 + ADR-011 + ADR-004 in `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`.
- **Reconnaissance file:line anchors (verified 2026-06-18, branch governance-v0 @ `488e1d075`):**
  - `governance_messaging_config` migration: `migrations/2026-06-03-000000-0000_add_governance_messaging_config/up.sql` — table is Postgres; read via the `read_current(pool, scope, key)` view idiom; `messaging_enabled` seed at line ~55 is the pattern to mirror.
  - `bridge_room` embedded schema: `services/bridge/src/bridge_room.rs:6–14` (CREATE TABLE in `open()`); zero `.sql` migration files in `services/bridge/`.
  - Pseudonym allocator: `crates/api/api/src/governance/actor_pseudonym_helper.rs:41–80` — `pub async fn get_or_create(pool: &mut DbPool<'_>, person_id: PersonId) -> LemmyResult<String>` (returns UUIDv4 pseudonym, idempotent on concurrent calls).
  - `actor_pseudonym` table: `crates/db_schema/src/source/governance/actor_pseudonym.rs:18–23`.
  - Bridge deps (greenfield for LiveKit): `services/bridge/Cargo.toml:10–31` — has `ed25519-dalek`, `hex`, `rusqlite 0.37`; NO jwt/livekit dep yet.
  - Compose: `services/bridge/docker-compose.yml:19–47` (tuwunel stanza); `docker/docker-compose.yml` stays Lemmy-only.
  - AGPL-NOTICE: `AGPL-NOTICE.md:32–36` (existing bridge row = the three-bullet format).
- **Lessons (mandatory):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs on Linux ONLY (`scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`; `ruma-common` E0119 on Windows).
  - `feedback_linux_compile_proof_is_a_gate.md` — bridge changes need the `validate-pending-laptop-linux` gate.
  - `feedback_build_what_tests_exercise.md` — the `rtc_enabled=false` off-path must be a TESTED signal, not an assumption.
  - `feedback_postgres_jsonb_canonicalization.md` — only if the config value is JSONB (it's a bool row — likely N/A, but flag if the plan stores `recording_config` as JSON text).

## 4. Constraints (enforce in the plan)

- **ADR-015 JWT pseudonym pin — make it LOAD-BEARING, not named** (per `.claude/rules/advisor-orchestrator.md` §2.4a + `feedback_cheap_model_arm_drops_adr_constraints.md`):
  1. Name the gate: the JWT mint fn MUST call `actor_pseudonym_helper::get_or_create(pool, person_id)` and set the token `sub`/`identity` to that pseudonym.
  2. Why it can't be deferred: a real `person_id`/username in the LiveKit `sub` reaches the LiveKit server, breaking the `always_pseudonym` anonymity guarantee — the load-bearing ADR-015 seam for the whole M3 cluster.
  3. DoD line: `grep` for the allocator call AND the JWT `sub`-assignment callsite in the mint fn; both must be present in the write path.
- **`rtc_enabled` defaults `false`** (PRD D6 + bootstrap tripwire). A plan shipping `true` as default = stop-and-ask.
- **No new Postgres SCHEMA migration** beyond the config-row seed (bootstrap tripwire: a column/table migration under `crates/db_schema/migrations/**` is a scope violation; a `governance_messaging_config` row-seed migration is the established in-scope pattern). The plan must be explicit that task 1 is a row seed, not a schema change.
- **Bridge compiles on Linux only** — every `services/bridge/**` task's DoD must be a `validate-pending-laptop-linux` DQ (`scripts/brehon/cargo-linux.sh check --workspace --features full` is wrong for the bridge — use `--manifest-path services/bridge/Cargo.toml`). The Postgres/binary tasks (config row, any `crates/**` consumer) validate on Windows-local + workspace.
- **Toolchain-boundary §15 DoD discipline** — this plan spans THREE validation surfaces: (a) Postgres + `crates/**` → Windows-local `cargo check --workspace --features full` + e2e; (b) `services/bridge/**` → `cargo-linux.sh --manifest-path services/bridge/Cargo.toml`; (c) docker sidecars → a deploy/up smoke (NOT a cargo gate — name it honestly, don't pretend `cargo` proves a compose stack). The §15 commands MUST be written in the exact form the advisor will run at gate 1 (per `feedback_plan_dod_dry_run_at_write.md` — m3-core-entry-kinds' plan used `rg` (absent) + line-counting greps that didn't dry-run clean).
- **Type-alias `Data<T>` watch** (per PMD retro id 906, m2-late-1 task-5): governance handlers use `activitypub_federation::config::Data<LemmyContext>`, NOT `actix_web::web::Data<LemmyContext>`. If the bridge JWT path or the `rtc_enabled` read consumer touches a governance handler signature, the plan's watchpoints (§4) must name the correct `Data` type to avoid the latent additive-alias mismatch that surfaced only at the caller.
- **Watchpoint specificity** (§3.5 gate, per `feedback_advisor_watchpoint_specificity.md`): every §4 watchpoint cites a specific table / file / fn — e.g. "watch `bridge_room.rs::open()` ALTER guard runs before any RTC-column read", not "watch for schema drift".
- **§16a stories** must include the `rtc_enabled=false` clean-posture story AND an `rtc_enabled=true` mints-a-valid-pseudonym-JWT story (the two PRD success signals for Phase 1).
- **DQ discipline:** planner writes `from: "planner"`; pre-seeds OQs as `answered_by: "planner"` only; mid-task pushes per `decision-queue.md` "Mid-task visibility". Forward-looking infra OQs (e.g. LiveKit api-key/secret provisioning, where the bridge reads them — `.env`? config row?) → raise as `kind: "blocker"` or pre-seed with a recommendation.

---

**Brief authored on `governance-v0`, committed before `create_task` (planning briefs commit on trunk — `.claude/refs/auto-phase.md` §"Brief location per role").** Lessons fired (§2.4 mechanical injection is for impl/fix-impl briefs; this is a planning brief, so the above are §2.3-search + bootstrap-watchlist driven): bridge-Linux, linux-compile-gate, build-what-tests-exercise, ADR-015-load-bearing, type-alias-Data-watch.
