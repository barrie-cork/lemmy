# m3-core-stage-mode — planning brief

`[role:planning]` Author the M3 Phase-3 plan: chair-controlled stage mode with FIFO raised-hand queue, mic-passing (30s grace + auto-revoke + next-promote), chair override, Q&A text sidebar, and the FIRST emission of the `room_chair_transferred` + `room_chair_override` chain entries. Bridge-side only.

## 1. Role + dispatch

`[role:planning]` — produce `.claude/PRPs/plans/m3-core-stage-mode.plan.md` from the M3 sub-PRD Phase 3, following `.claude/commands/prp-plan.md` + `.claude/PRPs/templates/plan.template.md`. Plan-shaping only — author NO implementation code, NO `crates/**`, NO `services/bridge/**`. The plan is the deliverable.

## 2. Scope

Plan **all of M3-core-stage-mode Phase 3 in one plan**. Source: `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` §"Phase 3: M3-core stage-mode" (lines 235–238) + §Technical Approach (174–203) + §Success Criteria rows for stage mode / mic-pass / 30s-grace / chair-override / chair-transfer (lines 136–140).

The plan's §13 tasks cover (planner finalizes count + `[P]` markers + ordering):

1. **Stage-mode room provisioning + dual-sourced chair seat (OQ-V2-05).** Extend the bridge's town-hall room-open path so the event opens in **stage mode**: the governance-assigned initial chair holds the one presenter publish-grant, all others are muted watchers. Chair seat is dual-sourced (OQ-V2-05, RESOLVED 2026-04-17, PRD line 69): the **governance plane sets the initial chair** at event creation (event creator, or jury foreperson for jury-adjacent events) — persisted in `bridge_room.chair_id` (the column m3-core-infra shipped, `bridge_room.rs:13`); the holder **delegates mid-session** via a Matrix-native transfer the bridge mirrors. The plan must name HOW `chair_id` is populated at provision time and the delegate path.
2. **FIFO raised-hand queue.** A FIFO queue of watchers requesting the mic. Decision the planner must make explicit in §4: does the queue live in `bridge_room.queue_state` (the TEXT/JSON column m3-core-infra shipped, `bridge_room.rs:14` — **persisted**, survives bridge restart) or in-memory (lost on restart)? The PRD/research model is a persisted queue; default to `queue_state` unless the planner documents a reason otherwise.
3. **Mic-passing with 30s grace + auto-revoke + next-promote.** Chair (or queue head) promotes a watcher: grant publish for **30s**; if the promoted speaker does NOT activate within 30s, **auto-revoke** the grant AND **promote the next** queued watcher. This is the load-bearing timing logic (watchlist #3). The chair can also force-pass down the FIFO queue.
4. **Chair override (force-demote / force-promote).** The chair can force-demote the current speaker or force-promote an arbitrary watcher out of FIFO order. Emits `room_chair_override` with `{action, target_pseudonym}` (Success Criteria line 139).
5. **Q&A text sidebar.** A live text channel alongside the stage for watcher questions (Matrix-native room/thread). Lowest-risk item; the planner scopes how it attaches to the event room.
6. **Chair-action chain emission (the FIRST emitter — see §4 LOAD-BEARING).** Wire the bridge to emit `room_chair_transferred` (`{from_pseudonym, to_pseudonym, at}`, Success Criteria line 140) on delegate/transfer AND `room_chair_override` (`{action, target_pseudonym}`) on override. **This is the first phase to actually EXERCISE the bridge→binary chain-emission callback** (see the §4 "bridge→binary callback gap" constraint — the callback CLIENT may not exist yet; if absent, building it is a §13 task, not an assumption).

**Explicit boundaries — do NOT plan:** federation-wide emergency-mute / `room_mute_all` (Phase 4 — the const exists at `governance_log.rs:255` but stage-mode does NOT emit it); MinIO / LiveKit Egress / recording / `room_recording_uploaded` (Phase 5); anonymous-town-hall pseudonym-overlay work beyond what the existing m3-core-infra JWT mint already provides (Phase 6 / inherited). The 3 chair/mute entry-kind consts are ALREADY SHIPPED (`governance_log.rs:253-255`, m3-core-entry-kinds @ `a5fc60a2d`) — the planner must NOT re-touch the entry-kind registry or re-declare consts.

**No new migration expected.** m3-core-infra added the `bridge_room` RTC columns (`chair_id`, `queue_state`, `recording_config`); stage-mode USES them. If the plan introduces a new `services/bridge` embedded-schema column or a `crates/db_schema/migrations/**` migration, that is a scope expansion — surface it as a `kind: "blocker"` DQ before authoring §13 around it (bootstrap tripwire).

## 3. Required reading

- `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` — §Phase 3 (235–238), §Technical Approach (174–203), §Success Criteria stage-mode rows (136–140), §Cross-Cutting Impact (155–159), ADR table (48–69, esp. OQ-V2-05 at line 69).
- `.claude/PRPs/handovers/m3-core-stage-mode-bootstrap.md` — §1 (one-paragraph scope + DoD), §4 watchlist (the six stage-mode-specific items), §7 catch-fire, §"Stop-and-ask tripwires".
- `.claude/PRPs/plans/m3-core-infra.plan.md` — the SIBLING plan (canonical-schema-first gate). Mirror its §5 complexity breakdown, §7 preflight guardrails (R1–R6), §10 patterns-to-mirror, §15 three-validation-surface DoD shape, §16a story shape. m3-core-infra is the immediately-preceding bridge plan — match its section conventions.
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` — canonical schema; the `RoomEventPayload` shape + `ROOM_KINDS` array + the `POST /api/v4/governance/room-event` route contract.
- ADR-015 + ADR-016 + ADR-011 in `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` (and OQ-V2-05 resolution).
- **Reconnaissance file:line anchors (verified 2026-06-18, branch `governance-v0` @ `307747138`):**
  - Chair/mute entry-kind consts (SHIPPED, do NOT re-declare): `crates/db_schema/src/source/governance/governance_log.rs:253-255` — `ENTRY_KIND_ROOM_CHAIR_TRANSFERRED` (`"room_chair_transferred"`), `_CHAIR_OVERRIDE` (`"room_chair_override"`), `_MUTE_ALL` (`"room_mute_all"`, Phase-4 only).
  - `bridge_room` RTC columns (SHIPPED, USE them): `services/bridge/src/bridge_room.rs:13-15` — `chair_id TEXT`, `queue_state TEXT`, `recording_config TEXT`; idempotent ALTER guards at `:30-34`.
  - **Bridge→binary chain-emission seam (the GAP — read carefully):**
    - Binary side EXISTS + mounted: `crates/api/api/src/governance/governance_log.rs:125 pub async fn append_room_event(pool, kind, payload, actor_pseudonym)` (gates on `ROOM_KINDS`, runs `scrub_json` + hash-chain + signing); route mounted at `crates/api/routes/src/lib.rs:484` (`POST /room-event` → `handle_room_event`); payload type `RoomEventPayload`.
    - Bridge side does NOT yet POST it: `services/bridge/src/config.rs:35-40` — `brehon_room_event_url` is a **config surface only**; the comment says *"the bridge currently receives room-events rather than POSTing them. Retained as config surface for a **future** bridge→binary room-event callback."* `room_provisioner.rs:391,439,463` explicitly defer chain-emission ("chain-emission deferred to T5"). **There is no bridge code that POSTs to `/governance/room-event` today.**
    - Auth + the reverse direction (binary→bridge, already wired, MIRROR for the auth pattern): `services/bridge/src/appservice.rs:196-253 handle_room_event` (validates `bridge_callback_secret` Bearer); `crates/api/api_utils/src/bridge_notify.rs:64,146` (binary's outbound POST-to-bridge client, the shape to mirror for the bridge's outbound POST-to-binary client).
  - **Mic-grant mechanism = LiveKit publish-grant (clarify DQ `a3d0e9941441-067`, advisor 2026-06-18), NOT Matrix power-levels.** Stage-mode promote/demote grants/revokes `canPublish` by re-minting the LiveKit access token at `services/bridge/src/livekit_jwt.rs:25 pub fn mint_access_token(..., identity: &str, ...)` (`:23-24` doc: *"`identity` is the opaque pseudonym string (ADR-015); the caller must supply a pre-derived pseudonym."*). Matrix power-levels are the **Phase-4 emergency-mute** mechanism only (the cross-instance <500ms problem) — do NOT use them for stage-mode mic-passing.
  - Room provisioner (extend for stage mode): `services/bridge/src/room_provisioner.rs:52 handle_transition`, `:85 provision_jury_room`, the community-event path; town-hall/event-room provisioning is the extension point.
  - Existing bridge integration tests (SIBLINGS — mirror the harness): `services/bridge/tests/dm_round_trip.rs`, `services/bridge/tests/room_provisioning.rs`. The stage-mode integration test joins these.
- **Lessons (mandatory):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs on Linux ONLY (`scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`; `ruma-common` E0119 on Windows).
  - `feedback_linux_compile_proof_is_a_gate.md` — every bridge task needs the `validate-pending-laptop-linux` gate; `bm-pr` is gated on `result:pass`.
  - `feedback_build_what_tests_exercise.md` — the integration test must EXERCISE the real flows: the 4-mic-pass sequence AND the 30s no-activate boundary AND the real `append()`-callback emission (not a stub). The DoD success signals are tests, not assumptions.
  - `feedback_governance_type_state_handlers.md` — the chair-seat / mic-grant state transitions are a type-state candidate (phantom-typed stage state, e.g. `Watcher`/`Promoted`/`Speaking`); flag it as a §10 pattern if the planner sees fit.
  - `pattern_test_against_reality_not_syntax.md` — assert on observable LiveKit/room state + the actual chain entry rows, not on config or struct shape.

## 4. Constraints (enforce in the plan)

- **Bridge→binary callback gap — surface it as the FIRST scope decision (LOAD-BEARING; clarify DQ `a3d0e9941441-066`, advisor-resolved to option (a) "build the client").** The chair-action chain emission (task 6) requires the bridge to POST `RoomEventPayload` to the binary's `/api/v4/governance/room-event` route. **That outbound client does not exist in the bridge today** (`config.rs:35-40` is config-only; `room_provisioner.rs` defers it). The plan §13 MUST either (a) include building the bridge→binary callback client (mirror `bridge_notify.rs:64` shape + `bridge_callback_secret` Bearer auth, POST to `brehon_room_event_url`) as its own task, OR (b) document that an existing client covers it with a file:line cite. The planner must NOT assume the emission "just calls append()" — `append_room_event` lives in the BINARY, reachable from the bridge only over HTTP. Gate: `grep -rn "brehon_room_event_url" services/bridge/src/` must, after impl, return an actual `.post(...)` callsite (not just the config field). This is watchlist #4 made concrete.

- **ADR-015 pseudonym pin — make it LOAD-BEARING, not named** (per `.claude/rules/advisor-orchestrator.md` §2.4a + `feedback_cheap_model_arm_drops_adr_constraints.md`):
  1. Name the gate: every chair-action chain entry (`room_chair_transferred`, `room_chair_override`) carries **pseudonyms** (`from_pseudonym`/`to_pseudonym`/`target_pseudonym`/`chair_pseudonym`), NEVER `person_id`/username/MXID. The pseudonym comes from the same allocator the JWT mint uses (`actor_pseudonym_helper::get_or_create` on the binary side, surfaced to the bridge).
  2. Why it can't be deferred: a real identity in a chain entry is permanent (hash-chained, append-only) — it cannot be scrubbed later, so a leak here is unrecoverable, breaking the `always_pseudonym` guarantee for the whole M3 cluster.
  3. DoD line: the integration test asserting the chain entry MUST assert the payload field is a pseudonym string, never a numeric `person_id` or an `@user:domain` MXID.

- **Content is never hashed (ADR-016 backplane contract).** Only chair-action METADATA (`{action, target_pseudonym, at}`) reaches the chain. Speech/video/Q&A-text bytes are NEVER hashed or sent to `append_room_event`. The plan must be explicit that only metadata is emitted.

- **30s grace boundary is a TESTED signal, not a happy-path assert** (watchlist #3, DoD). The integration test MUST drive a no-activate speaker and assert BOTH the auto-revoke fires at the grace boundary AND the next-promote happens. Cite the test fn name in §16a. A test that only asserts a successful promote does NOT satisfy the DoD — surface as a §3.5 watchpoint failure if the plan's §16a story omits the boundary.

- **4-mic-pass-in-sequence is the marquee DoD** (Success Criteria line 137). §16a must have a story driving the FIFO queue through 4 mic-passes with no manual intervention, asserting the publish-grant transitions.

- **Bridge compiles + tests on Linux only.** Every `services/bridge/**` task's DoD = a `validate-pending-laptop-linux` DQ; the cargo command is `scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml` (and the integration test runs via `cargo-linux.sh test --manifest-path services/bridge/Cargo.toml`). The Windows-local `cd services/bridge && cargo` form is a DoD-issue (`ruma-common` E0119). The §15 commands MUST be written in the EXACT form the advisor runs at gate 1 (per `feedback_plan_dod_dry_run_at_write.md`) — dry-run them mentally before committing the plan.

- **Cargo.lock sync on any dep add** (watchlist #6, m3-core-infra loose end). If a task adds a `services/bridge/Cargo.toml` dependency, the regenerated `Cargo.lock` MUST land in the SAME commit (it's a generated artifact regenerated by `cargo-linux.sh`, committed — mirror m3-core-infra plan §5.2 / §11). Stage-mode likely needs no new dep (reuses `livekit_jwt`, `rusqlite`, `reqwest`); flag if it does.

- **Watchpoint specificity** (§3.5 gate, per `feedback_advisor_watchpoint_specificity.md`): every §4 watchpoint cites a specific table / file:line / fn — e.g. "watch the bridge→binary callback POSTs to `brehon_room_event_url` with `bridge_callback_secret` Bearer (mirror `bridge_notify.rs:64`)", not "watch chain emission".

- **§16a stories** must include, at minimum: (1) the **4-mic-pass-in-sequence** story; (2) the **30s no-activate → auto-revoke + next-promote** boundary story; (3) a **chair-override emits `room_chair_override` with a pseudonym target** story; (4) a **chair-transfer emits `room_chair_transferred`** story. Each cites its integration-test fn name and asserts on observable state + the chain entry row, not config.

- **DQ discipline:** planner writes `from: "planner"`; pre-seeds OQs as `answered_by: "planner"` only; mid-task pushes per `decision-queue.md` "Mid-task visibility". The bridge→binary-callback scope decision (task 6) is the most likely DQ — raise it as `kind: "blocker"` with the (a)/(b) options above, or pre-seed a recommendation with the file:line evidence.

---

**Brief authored on `governance-v0`, committed before `create_task` (planning briefs commit on trunk — `.claude/refs/auto-phase.md` §"Brief location per role").** Lessons fired (§2.3-search + bootstrap-watchlist driven; §2.4 mechanical injection is impl/fix-impl-only): bridge-validates-on-linux, linux-compile-gate, build-what-tests-exercise, governance-type-state-handlers, test-against-reality-not-syntax, ADR-015-load-bearing, plan-dod-dry-run-at-write. Key recon finding surfaced for the planner: the bridge→binary chain-emission callback CLIENT does not exist yet (config-only at `config.rs:35-40`) — stage-mode is the first emitter and may need to build it.
