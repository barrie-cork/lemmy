# Plan: m3-core-e2e-pilot — full town-hall acceptance e2e + the D2 pilot run (M3 Phase 6, FINAL)

## 1. Summary

This is the **integration + pilot** phase that closes M3-core: it brings up the live RTC acceptance harness, turns the `#[ignore]`/`todo!()` integration scaffolds shipped across Phases 1–5 into **live, passing** tests against that harness, wires the two recording carry-forwards (cr-2 live requester-pseudonym, cr-3 real participant-set) plus the recording live trigger (real LiveKit Egress POST + S3 PUT replacing the fail-closed `bail!` stubs), and scopes the **D2 pilot town hall** as an operator-run acceptance runbook. It is **NOT a code-add phase** — no new entry-kind const, no migration, no new Brehon-workspace feature (registry stays frozen at 72). The phase has **two halves the plan keeps distinct**: **Half A** (impl/cohort flow — harness bring-up + acceptance e2e + recording carry-forwards, normal §13 tasks with `validate-pending-laptop-linux` compile gates + `validate-pending-laptop-e2e` live gates, bridge-Linux-only) and **Half B** (the D2 pilot — a human-run operational acceptance checklist the advisor coordinates and records in the retro, with **no cargo DoD**). **Headline acceptance:** the full §Success-Criteria table (PRD lines 134–149) is proven — every criterion row maps to exactly one live test fn (§16a) — with the marquee being the **federation-wide emergency mute <500ms measured at the PUBLISHER CLIENT across two federated bridge instances** (never the server — a server-side measurement is a FALSE GREEN, catch-fire), and the real acceptance signal being a **pilot town hall that runs end-to-end** with the chair passing the mic (optional recording landing with its `content_sha256` chain entry). The load-bearing net-new infra is the e2e harness: an **override compose** (`services/bridge/docker-compose.e2e.yml`, layered on the base, mirroring `docker-compose-fed-enable.yml` per clarify-DQ `a3d0e9941441-074`) adding **MinIO** (absent today) + a **second federated tuwunel+bridge instance with a distinct domain** (BUG-15), proven reachable in an **isolated reach-the-containers smoke BEFORE** any acceptance test depends on it.

## 2. Source

- `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` §"Phase 6: M3-core e2e + pilot" (250–253), the FULL §Success Criteria table (134–149 — Phase 6 proves all of it), §Technical Approach risk rows (196–203, esp. federation-mute row 198 [H3 headline] + RTC-topology row 202 [OQ-V2-10 non-loopback]), §Decisions D2 (269) + D6 (273) @ `2fae33bc1`.
- `.claude/PRPs/briefs/m3-core-e2e-pilot-planning-1.md` (the authorising brief) @ `1a65753bc`.
- `.claude/PRPs/handovers/m3-core-e2e-pilot-bootstrap.md` §1 (two-halves framing), §2 (HARD phase), §3 (lessons), §4 watchlist (6 items), §6 (gate-4 LIVE + new runner kinds), §7 catch-fire, §"Stop-and-ask tripwires", §"Git state at handoff" @ `1a65753bc`.
- `.claude/PRPs/reports/m3-core-recording-retro.md` §6 Carry-forwards (1: cr-2/cr-3 live session-auth; 2: recording live trigger Egress POST + S3 PUT) + the cr-2/cr-3 triage rows (lines 26–27) + §8 (MiniMax-as-reviewer now LIVE) @ `2fae33bc1`. **These two carry-forwards ARE Task 2.**
- `.claude/PRPs/plans/m3-core-recording.plan.md` + `.claude/PRPs/plans/m3-core-emergency-mute.plan.md` + `.claude/PRPs/plans/m3-core-stage-mode.plan.md` — the SIBLING bridge plans (canonical-schema-first gate, `feedback_read_canonical_before_writing_spec.md`): §5 complexity, §7 R-guardrails, §10 patterns, §15 three-surface DoD, §16a story shape (negative-invariant marquee + ADR-015 pin + observable-state assertions). Emergency-mute's `mute_all` test is the immediate ancestor of THIS phase's marquee — its publisher-client measurement framing is mirrored.
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` — canonical schema; `RoomEventPayload` shape + `ROOM_KINDS` + `POST /api/v4/governance/room-event` route. CODE WINS where 04 predates the RTC plane (doc-drift row 203).
- ADRs (`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`): **ADR-015** (pseudonymity — recording + chair entries carry pseudonyms; real identity never reaches LiveKit; identity→pseudonym at JWT-issue time); **ADR-016** (backplane — content never hashed beyond `content_sha256`; `federated: true` metadata); **ADR-013** (EmergencyRemove is DISTINCT from the chair "emergency mute" — do not conflate); **ADR-008** (append-only signed log — `content_sha256` rides `append_room_event`); **ADR-011** (AGPL — MinIO/Element-Call AGPL-3.0, LiveKit/lk-jwt Apache-2.0, licence-clean).
- `reference_pilot_test_accounts.md` + `workflow_state_pilot_internal.md` (PMD System-1) — pilot server `http://100.81.145.58:1236`, juror1-5 + testmod/testuser creds, `test_governance` community; the gov→Matrix path is verified (pilot-internal phases 1+2 DONE 2026-06-13). The D2 runbook (Task 7) coordinates with this track.
- Clarify-DQs (advisor-resolved): `a3d0e9941441-074` (override compose `docker-compose.e2e.yml`, distinct 2nd-instance domain — Task 1) + `a3d0e9941441-075` (D2 runbook supports EITHER scenario, operator picks at run time, DoD = retro-recorded outcome — Task 7).
- Lessons that bind decisions:
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`); never Windows-local (`ruma-common` E0119). EVERY bridge DoD command uses this form (watchpoint #3).
  - `feedback_e2e_nextest_filter_groups.md` — scope e2e runs with `-E` / `e2e_filter`; the town-hall group, not the full suite. LOAD-BEARING (e2e is the phase's centre of gravity); `e2e_filter` goes in the `validate-pending-laptop-e2e` DQ.
  - `feedback_lemmy_federation_domain_collision_one_host.md` (BUG-15) — the 2nd instance (Task 1) gets a DISTINCT hostname/domain, not just a distinct port.
  - `feedback_authz_state_machine_test_asserts_negative.md` — the clean-posture tests (recording-off + `rtc_enabled=false`) are NEGATIVE invariants; mechanical check: the test FAILS if the flag-gate is deleted.
  - `feedback_build_what_tests_exercise.md` + `pattern_test_against_reality_not_syntax.md` — the live tests EXERCISE the real flow (real Egress + MinIO object + the real chain entry; real cross-instance publisher-side mute timing) and assert on OBSERVABLE state, not config/struct shape.
  - `feedback_cheap_model_arm_drops_adr_constraints.md` — the ADR-015 pseudonym pins (cr-2 requester-pseudonym, recording chain-entry identity) are LOAD-BEARING, not named (§2.4a).
  - `feedback_fix_impl_pre_locate_e2e_anchors.md` — for the e2e-edit tasks, the `todo!("implement against live docker-compose stack (Phase-6 pilot grade)")` body is the unique Edit anchor (one per fn); the uniqueness gate (`grep -c` == 1) is run before each edit.
  - `feedback_complexity_score_pre_split.md` — this phase's e2e edits are ≥2 → expect the anchor-pre-locate injection; §5 score is computed below.
  - `feedback_plan_dod_dry_run_at_write.md` + `feedback_validate_pending_laptop_write_then_stop.md` — every §15 command is written in the exact form the advisor runs at gate 1; workers write the `validate-pending-laptop[-linux][-e2e]` DQ and STOP (the laptop is the runner).
  - `feedback_pilot_governance_workflow_seeding_order.md` + `feedback_pilot_resilience_test_blast_radius.md` — bind the D2 runbook (Task 7): which API path fires the bridge hook (`/governance/appeal` auto-seat fires + provisions; `admin_trigger_appeal_rejury` does NOT), and the blast-radius coordination (lane-boundary, monitor-collision, `messaging_enabled=false` is an instance-wide kill-switch).

## 3. Problem statement

Phases 1–5 shipped the entire RTC control plane as **scaffold** on `governance-v0` (verified 2026-06-20 @ `2fae33bc1`): `stage.rs::mute_all` (`:339`, with the cr-11 chair-exclusion fix) + `record_uploaded` (`:373`) + the chair state machine; `mute_handler.rs::compute_mute_all_override` (`:27`) + `mute_all_power_levels` (`:64`); `recording.rs::maybe_record` (`:31`, the flag-gate) + `is_participant` (`:111`) + `LiveSink` (`:54`); `appservice.rs::handle_recording_fetch` (`:238`) + the `/brehon/recording/{id}` route (`:318`); `livekit_jwt::mint_access_token` (`:31`, pseudonym → JWT `sub`); the full deterministic unit-test suite **passes**. But **nothing is integration-proven against a live stack, and three live surfaces are still scaffold-closed**:

- **The acceptance harness is net-new and incomplete.** `services/bridge/docker-compose.yml` ships the single-instance RTC stack (`tuwunel:` `:19`, `livekit:` `:53` profile-gated, `lk-jwt-service:` `:59`, `element-call:` `:68`) but has **NO `minio:` service** (the recording store the `recording_lands_with_hash_on_chain` + `participant_floor_fetch` tests need, D4/D5) and **NO second federated instance** (the marquee `mute_all_drops_all_publishers_cross_instance_under_500ms` is meaningless on one instance). The 5 integration files (`tests/{stage_mode,emergency_mute,recording,room_provisioning}.rs`) are `#[tokio::test] #[ignore]` with `todo!()` bodies — none exercise the live flow. **Task 1** is the load-bearing harness bring-up + reach-smoke, a hard `requires:` of every acceptance test.

- **The recording live trigger is fail-closed.** `LiveSink::trigger_egress` (`recording.rs:76`) and `upload` (`:104`) are `anyhow::bail!("…scaffold-only…")` stubs (recording retro §6.2); `RecordingSink` is a **synchronous** trait but the real Egress POST + S3 PUT are async (`reqwest`, `rust-s3::Bucket::put_object`). **Task 2** turns the trait async and replaces the two `bail!` with the live calls.

- **The recording-fetch participant-floor is scaffold-closed (cr-2/cr-3).** `handle_recording_fetch` reads the requester pseudonym from a placeholder `x-requester-pseudonym` header (`appservice.rs:261`) and stubs the participant set as **empty** (`:269`) → every caller is correctly-by-default 403 (recording retro cr-2/cr-3 carry-forward rows 26–27). **Task 2** wires cr-2 (the requester pseudonym is the BRIDGE_CALLBACK_SECRET-Bearer-authenticated forward of a validated session pseudonym) + cr-3 (the real participant set from `bridge_room`/room membership) so a participant fetch can succeed.

## 4. Solution statement

Two halves, kept distinct (bootstrap §1):

**Half A — impl/cohort flow (Tasks 1–6).**

**(1) The e2e harness (Task 1) — net-new infra, the load-bearing risk-isolation task.** A new `services/bridge/docker-compose.e2e.yml` **override** (layered: `docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc up`, mirroring `docker/docker-compose-fed-enable.yml`; clarify-DQ `a3d0e9941441-074`) adds: (a) `minio:` (AGPL-3.0, the generic-S3 recording store — endpoint wired to the bridge's `S3_ENDPOINT` config, NEVER a hardcoded literal in source); (b) a **second federated instance** — `tuwunel-b:` + `bridge-b:` — with a **distinct `MATRIX_SERVER_NAME` domain** (e.g. `matrix-b.localhost` / `matrix-a.localhost`), not just a distinct port (BUG-15: Lemmy/Matrix key by port-stripped domain; same host:port → the cross-instance mute test cannot tell the instances apart), federation enabled between the two homeservers. The **reach-the-containers smoke** (`scripts/brehon/e2e-harness-smoke.sh`, new) brings the stack up and asserts bridge↔livekit↔minio↔second-instance connectivity (HTTP/health probes per container) BEFORE any acceptance test depends on it — the same risk-isolation pattern the recording plan §18 used for the S3 dep. Per bootstrap stop-and-ask: prove the harness reaches the containers in an ISOLATED task or acceptance failures are unattributable.

**(2) Recording carry-forwards + live trigger (Task 2) — bridge-src wiring, the one substantive code change.** `recording.rs`: convert `RecordingSink` to an **async** trait (its `trigger_egress`/`upload` become `async fn`) and make `maybe_record` **generic over `S: RecordingSink`** (drop the `&mut dyn` so native async-fn-in-trait stays object-safe-free — no `async-trait` dep; §19 (2)); replace the two `LiveSink` `bail!` with the live async Egress POST (`reqwest` + the `livekit_jwt`-minted token, to `{livekit_url}/twirp/livekit.proto.Egress/StartRoomCompositeEgress`) and the live S3 PUT (`rust-s3::Bucket::put_object`, endpoint/bucket/creds from `config` — R12); convert the `Recorder` spy + the two deterministic tests (`clean_posture_no_side_effects_when_disabled`, `maybe_record_enabled_records_sink_and_emits_intent`) to `#[tokio::test]` + `.await`. `appservice.rs`: cr-2 — the requester pseudonym is accepted ONLY from the BRIDGE_CALLBACK_SECRET-Bearer-authenticated forward (the Bearer gate at `:250` is the trust boundary; harden: reject empty pseudonym) and cr-3 — resolve the real participant set from `bridge_room` room membership for the recording's room, replacing the empty `Vec::new()` at `:269`. `content_sha256` STILL rides `append_room_event` (R-CHAIN), pseudonyms-only in the chain entry (ADR-015).

**(3) Acceptance e2e go-live (Tasks 3–6) — turn `todo!()` into passing live tests, one §13 task per behaviour cluster.** Each task edits ONE integration test file (the `todo!()` body is the unique anchor): Task 3 `stage_mode.rs` (4-user mic-pass + 30s grace + chair override + chair transfer); Task 4 `emergency_mute.rs` (MARQUEE — publisher-client cross-instance <500ms per DQ `3004b6625b83-001`); Task 5 `recording.rs` (recording-lands + clean-posture-off + participant-floor fetch); Task 6 `room_provisioning.rs` (`rtc_enabled=false` town-hall clean-posture + anonymous-identity-never-reaches-LiveKit, citing `livekit_jwt::mint_pseudonym_claims`). Every §Success-Criteria row maps to exactly one bare-fn test (§16a); an uncovered row is a planning gap → DQ.

**Half B — the D2 pilot (Task 7), operational, NON-impl.** A human-run operational acceptance runbook (operator-run, no cargo DoD): bring up the full RTC stack on the pilot server (`http://100.81.145.58:1236`, accounts in `reference_pilot_test_accounts.md`), open a concrete pilot town hall (operator picks: community-deliberation broadcast OR appeal hearing with audience — clarify-DQ `a3d0e9941441-075`), the chair passes the mic to ≥1 user, optionally enable recording and confirm it lands with its hash entry. The DoD is a **recorded outcome in the retro** (did a real town hall run? did the chair pass the mic? did recording land?), NOT a test pass. The advisor coordinates + records; the plan scopes the checklist + go/no-go. **No `validate-pending-laptop` gate — it is not impl.**

A reader can predict §11 from this: a new override compose + a reach-smoke script (Task 1); `recording.rs` + `appservice.rs` (+ a `bridge_room` participant-set helper if needed) (Task 2); four integration-test files turned live (Tasks 3–6); a pilot runbook doc (Task 7). **No migration, no new const, no registry bump, no new prod dependency, no new in-binary sidecar code.**

## 5. Metadata

- **Phase:** `m3-core-e2e-pilot` (M3 Phase 6 of 6 — FINAL)
- **Branch:** `phase-m3-core-e2e-pilot` (cut by `bm-cut` before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 9 (Task 0 pre-flight + 6 impl + Task 7 pilot-runbook (NON-impl) + Task 8 retro)
- **Estimated cargo budget:** N/A on daemon — **no cargo runs on EliteDesk** (validate-pending-laptop[-linux][-e2e] discipline; the laptop is the runner). Bridge Linux container first-run COLD (~10–20 min). The live `-e2e` run needs the full two-instance + MinIO + LiveKit stack (gate-4, laptop or dispatch — user's choice).
- **Forbidden-window applicability:** binding for the **local** laptop cargo + the live e2e stack bring-up; non-binding for daemon impl-task throughput (no daemon cargo). Shape G RESIDUAL-ONLY.
- **Complexity score:** `7/10` — see breakdown. Under the Sonnet split threshold (8). **Proceed-as-one** (but it is the high end — the e2e-edit count + the net-new harness + the async-trait conversion drive it; see §5.1 + the §18 risk rows + the marquee DQ).

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 1 | 6 impl tasks (1–6); one above the 5 baseline |
| Migrations touched | +2 each | 0 | **No migration** — acceptance + pilot only |
| Crates touched | +1 each | 1 | `brehon-bridge` (workspace-excluded; Task 2 src + Tasks 3–6 tests). No `crates/**` change this phase (cr-2/cr-3 favour bridge-side; §12) |
| `services/bridge/tests/*.rs` (bridge integration) edits | +1 each | 4 | 4 integration files turned live (Tasks 3–6). NOT the 18k-line `crates/server/tests/e2e.rs` — bridge tests are small files (0 town-hall matches in `crates/server/tests/e2e.rs`), so no worker-hang risk; weighted +1 (not +3) accordingly |
| New ADR-affecting decisions | +2 each | 0 | Consumes ADR-004/008/011/013/015/016 + all M3 OQs (resolved); supersedes none |
| Cargo budget peak above 6 GB | +1 per GB | 0 | No daemon cargo (Shape-G residual) |
| Net-new infra harness (override compose + 2nd instance + MinIO + reach-smoke) | +1 | 1 | Task 1 — the load-bearing risk-isolation task (not a standard cargo task) |
| Async-trait conversion of a shipped sync trait | +1 | 1 | Task 2 — `RecordingSink` sync→async ripples to `maybe_record` + Recorder + 2 deterministic tests |
| **Total** | — | **7** | Threshold for split-DQ: `>8` (Sonnet) |

**Split decision:** 7 ≤ 8 — **proceed-as-one**, no split-DQ. It is the high end of the M3-core range (vs recording 3, emergency-mute 2): the e2e-edit count (4 files) + the net-new harness + the async conversion are real. Each individual task still respects the Sonnet ≤4-file/≤2-crate ceiling (§5.2), and the e2e-edit tasks are file-disjoint (`[P]`-eligible). The genuine risk is concentrated in the marquee measurement (DQ `3004b6625b83-001`) + the harness reach (Task 1 reach-smoke) — both isolated, both gated. The score is honest per the rubric; the residual risk is captured as §18 rows, not the mechanical number.

### 5.2 Per-task complexity ceiling (Sonnet target ≤4 files / ≤2 crates)

All tasks satisfy the Sonnet ceiling. **Task 1** creates 2 files (`docker-compose.e2e.yml`, `e2e-harness-smoke.sh`) — no crate. **Task 2** edits ≤3 files (`recording.rs`, `appservice.rs`, optionally `bridge_room.rs` for the participant-set helper) — 1 crate; closest to the ceiling. **Tasks 3–6** each edit exactly 1 integration test file — 1 crate; Task 4 MAY add a `[dev-dependencies]` livekit-client line (DQ-gated) → 2 files (`emergency_mute.rs` + `Cargo.toml`/`Cargo.lock`), still ≤4. **Task 7** writes 1 runbook doc (no crate, no cargo). No `crates/server/tests/e2e.rs` appears in any `modifies:`.

## 6. Relationship to other M3 sub-phases

- **Depends on:** m3-core-infra (Phase 1) — LiveKit JWT mint + identity→pseudonym at issue, `rtc_enabled`, `livekit_*`/`s3_*` config, the `recording_config` column. m3-core-entry-kinds (Phase 2) — the 3 chair/mute consts + `room_recording_uploaded`, `ROOM_KINDS`, registry 72. m3-core-stage-mode (Phase 3) — `Stage` + `GrantSink`/`GrantCmd` + `EmitIntent`/`pending_emits`/`drain_emits`/`post_room_event` + the chair state machine. m3-core-emergency-mute (Phase 4) — `mute_handler` (power-level path) + `Stage::mute_all` + `federated`. m3-core-recording (Phase 5) — `maybe_record`/`LiveSink`/`is_participant`/`record_uploaded` + `handle_recording_fetch` + the 5 recording DTO fields + `rust-s3`.
- **Followed by:** **nothing in M3-core** — this is Phase 6 of 6, the LAST M3-core sub-phase. At ship, `/brehon-phase-transition` to whatever the roadmap names next (confirm with user; bootstrap §8). The D2 pilot coordinates with the pilot-internal track.
- **Reuses (builds NOTHING new in-binary):** the entire shipped control plane. The only net-new surfaces are the e2e harness (compose + smoke), the recording live-trigger bodies (replacing `bail!`), the cr-2/cr-3 fetch wiring, and the live test bodies. No new emitter, no new const, no new migration, no new prod dep.

## 7. Preflight guardrails inherited from prior phases

- **R1 (bridge-Linux):** every `services/bridge` cargo command runs via `scripts/brehon/cargo-linux.sh … --manifest-path services/bridge/Cargo.toml`; the Windows-local `cd services/bridge && cargo` form fails with `ruma-common` E0119 and is reference-only (`feedback_bridge_validates_on_linux_not_windows.md`, watchpoint #3).
- **R2 (Linux-compile gate):** every bridge-src-touching task (2–6) writes a `validate-pending-laptop-linux` DQ for the COMPILE proof; `bm-pr` gates on `result:pass` (`feedback_linux_compile_proof_is_a_gate.md`). Task 4 is the headline `-linux` gate IF the livekit-client dev-dep lands (a new Linux Cargo.lock resolution).
- **R3 (no daemon cargo):** workers write the `validate-pending-laptop[-linux][-e2e]` DQ and **stop**; the laptop advisor runs all cargo/e2e (`feedback_validate_pending_laptop_write_then_stop.md`, `project_laptop_canonical_cargo_runner.md`).
- **R4 (cargo capture):** every cargo invocation captures to a log file and reads `tail -20` + `echo exit: $?`; never pipe cargo through `tail`/`grep` (`cargo-output-capture.md` + `no-cargo-output-paste.md`).
- **R5 (Task 0 enumerates all probes explicitly):** see Task 0.
- **R6 (clippy uniform):** all clippy invocations use `--no-deps -- -D warnings`; bridge clippy runs via `cargo-linux.sh`.
- **R7 (the negative invariant is TESTED, not assumed):** both clean-posture invariants (recording-off line 145 + `rtc_enabled=false` line 146) are §16a stories whose tests FAIL when the flag-gate is deleted (`feedback_authz_state_machine_test_asserts_negative.md`); an always-skip clean-posture test is invalid (§3.5 watchpoint failure).
- **R8 (no schema migration / no new const / no registry bump):** Phase 6 EXERCISES existing kinds (registry frozen at 72). A new `ENTRY_KIND_ROOM_*` const, a `crates/db_schema/migrations/**` migration, or a `services/bridge` embedded-schema column is a scope violation → STOP and raise a `kind: "blocker"` DQ (bootstrap tripwires / watchpoint #4).
- **R9 (ADR-015 load-bearing, §2.4a):** (a) cr-2 — the recording-fetch requester pseudonym is accepted ONLY from the BRIDGE_CALLBACK_SECRET-Bearer-authenticated forward (the trust boundary), never an unauthenticated header; `is_participant` is called BEFORE serving (callsite in the fetch path, not a named constant). (b) every `room_*` chain entry (chair entries, recording `speakers`/actor) carries pseudonyms — never `person_id`/username/MXID. (c) anonymous town hall — the LiveKit server receives only the pseudonym (`livekit_jwt::mint_access_token` assigns the pseudonym verbatim to JWT `sub`, `:46`; the shipped `mint_pseudonym_claims` test `:69` is the unit anchor; the live test asserts the server-received identity is the pseudonym). Grep DoD in §15.5.
- **R10 (ADR-016 metadata-only):** only metadata (`{media_url, content_sha256, duration_s, speakers, attendance_count}` for recording; `{chair_pseudonym, federated}` for mute) reaches `append_room_event`; MP4 bytes go to S3, never the chain. §15.5 asserts the emit payloads carry only metadata.
- **R-CHAIN (`content_sha256` rides `append_room_event`, never a bypass — CATCH-FIRE):** the recording's `content_sha256` flows `recording.rs → record_uploaded EmitIntent → drain_emits → post_room_event → append_room_event` (which runs `scrub_json` + ed25519 hash-chain + signing). The live `recording_lands_with_hash_on_chain` test asserts the hash appears in a `governance_log` chain row with a valid signature/prev-hash link, NOT merely in a MinIO object's metadata. A raw-digest side channel is an ADR-008/016 violation (bootstrap §7 catch-fire). Inherited verbatim from recording R11.
- **R-PUBCLIENT (emergency-mute <500ms is measured at the PUBLISHER CLIENT, never the server — CATCH-FIRE, the marquee):** per Success Criterion 141 + PRD risk row 198 + watchpoint #2 + tripwire #1. A server-side measurement is a FALSE GREEN. The measurement mechanism is DQ `3004b6625b83-001` (planner-recommended option-B; advisor/user ratifies at gate-1/gate-4). The PRD fallback (best-effort cross-instance SLA + in-instance <500ms guarantee) is DOCUMENTED via that DQ before declaring the criterion failed — never silently dropped.
- **R-DOMAIN (two-instance distinct domains — BUG-15):** the 2nd instance (Task 1) has a distinct `MATRIX_SERVER_NAME` domain, not just a distinct port; the override compose names both domains + the federation link (`feedback_lemmy_federation_domain_collision_one_host.md`).
- **R-S3ENDPOINT (generic S3, no hardcoded endpoint):** the S3 endpoint comes from `config.s3_endpoint`/env, NEVER a hardcoded `minio:9000` in bridge source (`rg -n 'minio\.' services/bridge/src/` returns nothing after impl). Inherited from recording R12. The override compose MAY name `minio:9000` (it is config wiring, not source).
- **R-BAREFN (bare fn name for bridge test filters):** every §15/§16a test-filter uses the BARE fn name (e.g. `mute_all_drops_all_publishers_cross_instance_under_500ms`), NOT `module::tests::…` (promoted from emergency-mute L1).
- **R-ANCHOR (pre-locate verbatim e2e Edit anchors):** for Tasks 3–6, the `todo!("implement against live docker-compose stack (Phase-6 pilot grade)")` body is the unique Edit anchor; run `grep -c 'todo!("implement against live docker-compose stack (Phase-6 pilot grade)")' <file>` == 1 before editing (`feedback_fix_impl_pre_locate_e2e_anchors.md`). Within a file with >1 `todo!()` (recording.rs has 3, room_provisioning.rs has 5), anchor on the **enclosing fn signature** line first.

## 8. Flow design

```
BEFORE (Phases 1–5 shipped on governance-v0 @ 2fae33bc1):
  harness  docker-compose.yml : tuwunel + livekit(profile rtc) + lk-jwt + element-call
                                NO minio ; NO second instance
  bridge   recording.rs : RecordingSink (SYNC trait) ; LiveSink::{trigger_egress,upload} = bail!("scaffold-only")
                          maybe_record(&mut dyn RecordingSink) (sync) ; is_participant (live)
  bridge   appservice.rs : handle_recording_fetch — requester from x-requester-pseudonym HEADER (placeholder)
                           participants = Vec::new() (empty → all 403) ; media_url = "{id}.mp4" (placeholder)
  bridge   stage.rs : mute_all (live, cr-11 chair-exclusion) ; record_uploaded (live)
  bridge   mute_handler.rs : compute_mute_all_override ; mute_all_power_levels (live, #[allow(dead_code)])
  bridge   livekit_jwt.rs : mint_access_token(identity=pseudonym → JWT sub) ; mint_pseudonym_claims test
  tests    {stage_mode,emergency_mute,recording,room_provisioning}.rs : #[ignore] todo!() bodies

AFTER (m3-core-e2e-pilot):
  harness  docker-compose.e2e.yml (NEW override, layered on base) :                                    [Task 1]
             + minio: (AGPL-3.0 generic-S3 store ; S3_ENDPOINT wired to bridge config)
             + tuwunel-b: + bridge-b: (2nd instance, DISTINCT MATRIX_SERVER_NAME domain — BUG-15)
             + federation link between the two homeservers
           scripts/brehon/e2e-harness-smoke.sh (NEW) : up → assert bridge↔livekit↔minio↔instance-b reach  [Task 1]
  bridge   recording.rs : RecordingSink (ASYNC trait) ; maybe_record<S: RecordingSink> (async, generic)   [Task 2]
             LiveSink::trigger_egress = live reqwest POST {livekit_url}/twirp/…/StartRoomCompositeEgress
             LiveSink::upload         = live rust-s3 Bucket::put_object (endpoint from config — R-S3ENDPOINT)
             Recorder spy + clean_posture/enabled tests → #[tokio::test] + .await
  bridge   appservice.rs : handle_recording_fetch —                                                       [Task 2]
             cr-2: requester pseudonym from the Bearer-authed forward (reject empty; trust boundary doc)
             cr-3: participants = bridge_room room-membership lookup (real set → participant fetch 200)
  tests    stage_mode.rs : four_mic_pass_then_grace_boundary_emits_chair_entries — LIVE (criteria 136–140) [Task 3]
  tests    emergency_mute.rs : mute_all_…_cross_instance_under_500ms — LIVE, publisher-client (141 MARQUEE) [Task 4]
  tests    recording.rs : recording_lands_with_hash_on_chain (143) + clean_posture_no_recording (145)      [Task 5]
                          + participant_floor_fetch (144) — LIVE
  tests    room_provisioning.rs : rtc_disabled town-hall clean-posture (146) + anonymous identity (142)    [Task 6]
  doc      .claude/PRPs/runbooks/m3-core-e2e-pilot-d2-runbook.md (operator-run, NON-impl, NO cargo DoD)    [Task 7]
```

The acceptance tests RUN against Task 1's live harness (gate-4 `validate-pending-laptop-e2e`, laptop or dispatch — user picks once). The COMPILE proof (`--no-run` / unit) is the `validate-pending-laptop-linux` gate the worker writes. The D2 pilot (Task 7) is the human-run signal, recorded in the retro.

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit:

- **Harness (Task 1)** — `services/bridge/docker-compose.yml` (base: `tuwunel:` `:19`, `livekit:` `:53` profile-gated, `lk-jwt-service:` `:59`, `element-call:` `:68`, `bridge-net:` `:91`); `docker/docker-compose-fed-enable.yml` (the override-layering precedent — re-creates only changed services); `services/bridge/docker-compose.pilot.yml` (the real-Tuwunel + bridge single-instance shape to mirror for `tuwunel-b`/`bridge-b`, incl. `MATRIX_SERVER_NAME`, `extra_hosts`, env wiring); `services/bridge/registration.yaml` + `services/bridge/tuwunel-pilot.toml` (the AS/homeserver config to clone-with-distinct-domain for instance B).
- **Recording live trigger + cr-2/cr-3 (Task 2)** — `services/bridge/src/recording.rs:17-22` (`RecordingSink` trait → async), `:31-49` (`maybe_record` → generic+async), `:54-106` (`LiveSink` + the two `bail!` at `:76`/`:104` + the Phase-6 POST/PUT comments at `:71-75`/`:102`), `:115-263` (the `Recorder` spy + the 2 deterministic tests → `#[tokio::test]`); `services/bridge/src/appservice.rs:238-284` (`handle_recording_fetch` — cr-2 at `:259-265`, cr-3 at `:267-269`, media_url at `:281-282`), `:243-257` (the BRIDGE_CALLBACK_SECRET Bearer gate = the cr-2 trust boundary); `services/bridge/src/bridge_room.rs:47` (`lookup`), `:59` (`lookup_by_case`) — the room-membership source for cr-3; `services/bridge/src/livekit_jwt.rs:31-46` (`mint_access_token` — Egress token mint). [Task 2]
- **Stage seam EXERCISED by the live tests (do NOT rebuild)** — `services/bridge/src/stage.rs:57-68` (`Stage` fields), `:74-104` (`load`), `:105-201` (`promote_next` + mic-pass + 30s grace), `:202-287` (`chair_override`), `:288-338` (`transfer_chair`), `:339-366` (`mute_all`, cr-11 chair-exclusion), `:373-410` (`record_uploaded`). [Tasks 3, 4, 5]
- **Mute power-level path EXERCISED by the marquee** — `services/bridge/src/mute_handler.rs:27-62` (`compute_mute_all_override`), `:64-98` (`mute_all_power_levels`); `services/bridge/src/sanction_handler.rs` (`get_power_levels`/`put_power_levels` `pub(crate)`). [Task 4]
- **Provisioning + drain + chair seat (reference)** — `services/bridge/src/room_provisioner.rs:201` (`provision_townhall_stage_room`), `:267-308` (chair seat + presenter-token mint = the pseudonym source), `:350` (the `drain_emits` callsite). [Tasks 3, 4, 5, 6]
- **Integration-test scaffolds to turn live (MIRROR the assertion style each names)** — `services/bridge/tests/stage_mode.rs:14` (step comments), `services/bridge/tests/emergency_mute.rs:14-48` (the pre-specified publisher-client measurement + PRD fallback), `services/bridge/tests/recording.rs:14,33,47` (3 scenarios), `services/bridge/tests/room_provisioning.rs:57` (`messaging_disabled_prevents_provisioning` — the clean-posture model). [Tasks 3–6]
- **Registry (do NOT re-touch)** — `.claude/rules/governance-log-entry-kind-registry.md` (count 72, frozen this phase). [verification only]
- **Pilot runbook sources (Task 7)** — `reference_pilot_test_accounts.md` (PMD), `workflow_state_pilot_internal.md` (PMD; gov→Matrix verified), `feedback_pilot_governance_workflow_seeding_order.md` (which API path fires the bridge hook), `feedback_pilot_resilience_test_blast_radius.md` (blast-radius coordination). [Task 7]
- **Lessons** — `feedback_bridge_validates_on_linux_not_windows.md`, `feedback_e2e_nextest_filter_groups.md`, `feedback_lemmy_federation_domain_collision_one_host.md`, `feedback_authz_state_machine_test_asserts_negative.md`, `feedback_build_what_tests_exercise.md`, `pattern_test_against_reality_not_syntax.md`, `feedback_cheap_model_arm_drops_adr_constraints.md`, `feedback_fix_impl_pre_locate_e2e_anchors.md`. [Tasks 1–6]

## 10. Patterns to mirror

### 10.1 e2e override compose — MinIO + 2nd federated instance (mirror `docker-compose-fed-enable.yml` layering + `docker-compose.pilot.yml` instance shape)

**Mirror:** `docker/docker-compose-fed-enable.yml` (layering: re-create only changed/added services) + `services/bridge/docker-compose.pilot.yml:12-89` (the tuwunel+bridge instance shape, `MATRIX_SERVER_NAME`, `extra_hosts`, env).

```yaml
# services/bridge/docker-compose.e2e.yml — Phase-6 acceptance harness OVERRIDE.
# Apply:  docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc up -d
# Adds the two things base lacks: MinIO (recording store) + a 2nd federated instance.
services:
  minio:                                  # AGPL-3.0 generic-S3 recording store (D4/D5)
    image: minio/minio:<pin>
    command: ["server", "/data"]
    environment: { MINIO_ROOT_USER: "<from .env>", MINIO_ROOT_PASSWORD: "<from .env>" }
    ports: ["9000:9000"]
    networks: [bridge-net]
  tuwunel-b:                              # 2nd homeserver — DISTINCT domain (BUG-15)
    image: <same tuwunel pin as base>
    environment: { CONDUIT_SERVER_NAME: "matrix-b.localhost", CONDUIT_ALLOW_FEDERATION: "true" }
    networks: [bridge-net]
  bridge-b:                              # 2nd bridge bound to tuwunel-b
    build: { context: ., dockerfile: Dockerfile }
    environment:
      TUWUNEL_URL: "http://tuwunel-b:8008"
      MATRIX_SERVER_NAME: "matrix-b.localhost"   # distinct domain — NOT a port change
      S3_ENDPOINT: "http://minio:9000"           # config wiring (R-S3ENDPOINT: source must not hardcode)
      S3_BUCKET: "recordings" ; S3_ACCESS_KEY: "<env>" ; S3_SECRET_KEY: "<env>"
    networks: [bridge-net]
  # base `tuwunel`/`bridge` represent instance-A (domain matrix-a.localhost via base override of CONDUIT_SERVER_NAME);
  # the federation link is established by enabling federation on BOTH homeservers + cross-signing keys at smoke time.
```

The exact service names, the federation-enable wiring, and the MinIO/tuwunel image pins are finalized by the impl-task against the live image tags (the deploy-smoke confirms tags). **R-DOMAIN:** instance-A = `matrix-a.localhost`, instance-B = `matrix-b.localhost` — distinct domains, the cross-instance mute test routes a publisher onto each.

### 10.2 reach-the-containers smoke (the risk-isolation proof — mirror the recording §18 dep-isolation pattern)

```bash
# scripts/brehon/e2e-harness-smoke.sh — prove the harness REACHES the containers
# BEFORE any acceptance test depends on them (bootstrap stop-and-ask).
set -euo pipefail
docker compose -f services/bridge/docker-compose.yml -f services/bridge/docker-compose.e2e.yml --profile rtc up -d
# 1. MinIO reachable + bucket creatable
curl -fsS http://localhost:9000/minio/health/live           # EXPECT: 200
# 2. LiveKit reachable
curl -fsS http://localhost:7880                              # EXPECT: livekit handshake
# 3. instance-A bridge ↔ instance-A tuwunel
curl -fsS http://localhost:<bridgeA>/healthz || true        # bridge liveness
# 4. instance-B tuwunel reachable + federates with instance-A (distinct domains)
curl -fsS http://localhost:<tuwunelB>/_matrix/federation/v1/version  # EXPECT: 200, server_name matrix-b.localhost
echo "E2E_HARNESS_REACH_OK"
```

DoD: `E2E_HARNESS_REACH_OK` printed + every probe green. A failing reach-smoke STOPS the phase (acceptance failures would be unattributable). This is the laptop-run gate for Task 1.

### 10.3 `RecordingSink` sync→async + live `LiveSink` (replace the two `bail!`)

**Mirror:** `services/bridge/src/recording.rs:17-22` (current sync trait), `:60-106` (`LiveSink` impl with the Phase-6 POST/PUT comments). `livekit_jwt::mint_access_token` for the Egress token; `rust-s3::Bucket::put_object` for the upload.

```rust
// recording.rs — async trait (native async fn in trait, Rust 1.95; no async-trait dep — §19 (2))
pub trait RecordingSink {
    async fn trigger_egress(&mut self, room_id: &str) -> anyhow::Result<()>;
    async fn upload(&mut self, key: &str, bytes: &[u8]) -> anyhow::Result<String>;
}

// maybe_record: generic over S (NOT &mut dyn) so the async trait stays object-safety-free.
pub async fn maybe_record<S: RecordingSink>(
    enabled: bool, sink: &mut S, stage: &mut crate::stage::Stage,
    room_id: &str, mp4_bytes: &[u8], duration_s: i64,
    speakers: Vec<String>, attendance_count: i32,
) -> anyhow::Result<()> {
    if !enabled { return Ok(()); }            // the load-bearing flag-gate (delete → clean-posture test FAILS)
    sink.trigger_egress(room_id).await?;
    let content_sha256 = compute_content_sha256(mp4_bytes);
    let media_url = sink.upload(&format!("{room_id}.mp4"), mp4_bytes).await?;
    stage.record_uploaded(media_url, content_sha256, duration_s, speakers, attendance_count)?;
    Ok(())
}

impl RecordingSink for LiveSink<'_> {
    async fn trigger_egress(&mut self, room_id: &str) -> anyhow::Result<()> {
        // mint token (existing), then LIVE: self.client.post(format!("{livekit_url}/twirp/livekit.proto.Egress/StartRoomCompositeEgress"))
        //   .bearer_auth(&token).json(&egress_request).send().await?.error_for_status()?;  (replaces bail!)
    }
    async fn upload(&mut self, key: &str, bytes: &[u8]) -> anyhow::Result<String> {
        // build bucket (existing creds/region/endpoint from config — R-S3ENDPOINT), then LIVE:
        //   bucket.put_object(format!("/{key}"), bytes).await?;  return the object URL.  (replaces bail!)
    }
}
```

The `Recorder` spy + `clean_posture_no_side_effects_when_disabled` + `maybe_record_enabled_records_sink_and_emits_intent` become `#[tokio::test]` + `.await`. **The clean-posture negative invariant is preserved:** `maybe_record(false, …)` still produces zero `trigger_egress`/`upload`/EmitIntent; the delete-the-gate check still holds (R7).

### 10.4 cr-2 requester-pseudonym + cr-3 participant-set (ADR-015 — LOAD-BEARING, §2.4a)

**Mirror:** `services/bridge/src/appservice.rs:238-284` (current scaffold). cr-2: the requester pseudonym is the BRIDGE_CALLBACK_SECRET-Bearer-authenticated forward — the Bearer gate at `:250` IS the trust boundary (the Brehon binary authenticates the user's session and forwards the resolved pseudonym; the bridge trusts it ONLY because the caller proved the callback secret). Harden: reject an empty pseudonym (`400`/`403`), not just rely on the empty-set floor. cr-3: replace `let participants: Vec<String> = Vec::new();` (`:269`) with a real lookup of the recording's room participant set from `bridge_room` membership.

```rust
// cr-2: requester pseudonym from the authenticated forward (already Bearer-gated at :250).
let requester_pseudonym = headers.get("x-requester-pseudonym")
    .and_then(|v| v.to_str().ok()).unwrap_or("").to_owned();
if requester_pseudonym.is_empty() {            // cr-2 hardening: no anonymous fetch
    return (StatusCode::FORBIDDEN, "missing requester pseudonym").into_response();
}
// cr-3: real participant set for the recording's room (was Vec::new()).
let participants = crate::bridge_room::participants_for_recording(&conn, &recording_id)?; // pseudonyms (ADR-015)
if !crate::recording::is_participant(&requester_pseudonym, &participants) {   // called BEFORE serving
    return (StatusCode::FORBIDDEN, "not a participant").into_response();      // ADR-015 floor
}
```

`participants_for_recording` (a new `bridge_room` helper if no existing membership query fits) returns PSEUDONYMS only. **DoD grep:** `grep is_participant services/bridge/src/appservice.rs` returns the callsite AND it runs BEFORE the 200 path. **Why it can't be deferred:** a pseudonymous town hall's recording served to a non-participant leaks the event (the D5 Option C access bar).

### 10.5 Marquee — publisher-client cross-instance <500ms (the negative invariant, R-PUBCLIENT)

**Mirror:** the emergency-mute plan §10.4 + `services/bridge/tests/emergency_mute.rs:14-48` (the pre-specified publisher-client measurement + PRD fallback) + `stage.rs` mute_all test idiom (`:727-779`, set-equality zero-holder, NOT "≥1 revoke").

Per DQ `3004b6625b83-001` (planner-recommended option-B): the live test (a) records a per-publisher publish-active baseline **at the publisher client** (a LiveKit client connection's own `can_publish` state); (b) fires mute-all (the cross-instance `mute_all_power_levels` Matrix power-level PUT + the local `Stage::mute_all` LiveKit `RevokePublish` sweep); (c) asserts EVERY non-chair publisher's publish right is revoked **at the publisher client** within 500ms (in-instance: the automated guarantee; cross-instance: the D2-pilot signal per the PRD fallback, documented via the DQ); (d) asserts the cross-instance zero-holder negative invariant (no listed publisher on EITHER instance retains publish); (e) asserts the `governance_log` `room_mute_all` row carries `actor_pseudonym` = the chair PSEUDONYM + `payload.federated == true`. **A server-side measurement is a FALSE GREEN — catch-fire.** The publisher-client measurement point (which client API, which timestamp) is finalized at gate-1 ratification of the DQ.

### 10.6 Anonymous-town-hall identity-never-reaches-LiveKit (criterion 142 — cite the JWT-issue-time anchor)

**Mirror:** `services/bridge/src/livekit_jwt.rs:31-46` (`mint_access_token` assigns the pseudonym verbatim to JWT `sub`) + `:69` (`mint_pseudonym_claims` — the shipped unit anchor that asserts the JWT carries the pseudonym, not a real identity).

The live assertion (Task 6) confirms the LiveKit server receives only the pseudonym: provision an `always_pseudonym` town hall, mint a participant token, connect, and assert the LiveKit-server-side participant identity == the pseudonym (never the Lemmy username/`person_id`/MXID). The unit-level guarantee is `mint_pseudonym_claims` (cited, not re-implemented); the live test adds the server-received-identity check.

## 11. Files to change

**`services/bridge` harness (Task 1) — no crate, no cargo:**
- `services/bridge/docker-compose.e2e.yml` — NEW override: `minio:` + `tuwunel-b:`/`bridge-b:` (distinct domain) + federation link (Task 1)
- `scripts/brehon/e2e-harness-smoke.sh` — NEW reach-the-containers smoke (Task 1)
- (if instance-B needs its own AS/HS config) `services/bridge/registration-b.yaml` + a `tuwunel-b` config snippet — NEW, cloned-with-distinct-domain from `registration.yaml`/`tuwunel-pilot.toml` (Task 1; the impl-task confirms whether the base files parameterise the domain or need a B-variant)

**`services/bridge/src` (brehon-bridge, workspace-excluded), Linux-validated (Task 2):**
- `services/bridge/src/recording.rs` — `RecordingSink` sync→async; `maybe_record` async+generic; `LiveSink::{trigger_egress,upload}` live (replace the 2 `bail!`); `Recorder` + 2 deterministic tests → `#[tokio::test]` (Task 2)
- `services/bridge/src/appservice.rs` — cr-2 (reject empty requester pseudonym) + cr-3 (real participant set) in `handle_recording_fetch`; resolve the real `media_url` (Task 2)
- `services/bridge/src/bridge_room.rs` — `participants_for_recording` helper IF no existing membership query fits (Task 2; the impl-task checks `lookup`/`lookup_by_case` first)

**`services/bridge/tests` (brehon-bridge integration), Linux-validated (Tasks 3–6):**
- `services/bridge/tests/stage_mode.rs` — `four_mic_pass_then_grace_boundary_emits_chair_entries` live (Task 3)
- `services/bridge/tests/emergency_mute.rs` — `mute_all_drops_all_publishers_cross_instance_under_500ms` live (Task 4); MAY add `Cargo.toml`/`Cargo.lock` IF the livekit-client dev-dep lands (DQ `3004b6625b83-001` option-A/B)
- `services/bridge/tests/recording.rs` — `recording_lands_with_hash_on_chain` + `clean_posture_no_recording_when_disabled` + `participant_floor_fetch` live (Task 5)
- `services/bridge/tests/room_provisioning.rs` — NEW `rtc_disabled_townhall_clean_posture` + `anonymous_townhall_identity_never_reaches_livekit` fns (Task 6) — added alongside the existing M2 scaffolds (which stay `#[ignore] todo!()` — out of Phase-6 scope)

**Pilot runbook (Task 7) — NON-impl, no cargo:**
- `.claude/PRPs/runbooks/m3-core-e2e-pilot-d2-runbook.md` — NEW operator-run acceptance checklist + go/no-go criteria (Task 7)

### Struct-field add: enumerate all callsites (mandatory)

- **No public struct gains a field this phase.** The `RoomEventPayload` (binary + bridge mirror) already carries all chair/recording/`federated` fields (shipped Phases 4–5). The only signature change is `RecordingSink` (sync→async) + `maybe_record` (`&mut dyn` → generic `S`): callers are `LiveSink` + `Recorder` (impls, same file) + the 2 deterministic tests (same file) + (Phase-6 only, none yet) the live town-hall-start trigger. `rg "maybe_record\(" services/bridge/` returns ONLY the 2 deterministic tests in `recording.rs` (the live trigger is not yet wired). All callsites are in Task 2's single file/commit — the async change is non-breaking outside `recording.rs`.

## 12. NOT building in m3-core-e2e-pilot

- **No new entry-kind const / `ROOM_KINDS` add / registry bump** — frozen at 72 (R8). The 3 chair/mute kinds + `room_recording_uploaded` all shipped Phases 2–5. A new `ENTRY_KIND_ROOM_*` const is a SCOPE ERROR → STOP + `kind: "blocker"` DQ (watchpoint #4).
- **No new migration / no new `bridge_room` column** — acceptance + pilot only (R8). A `crates/db_schema/migrations/**` or embedded-schema column add → STOP + `kind: "blocker"` DQ.
- **No new Brehon-workspace Rust feature** — the control logic is scaffolded; this phase wires + tests it. The ONLY `crates/**` change allowed is a cr-2/cr-3 binary-side touch IF unavoidable (favour bridge-side; §4) — and the plan finds cr-2/cr-3 are bridge-side (the participant set is in `bridge_room`; the requester pseudonym rides the Bearer-authed forward), so **zero `crates/**` change is planned**. A `crates/**` edit beyond a genuinely-required cr-2/cr-3 binary route is a scope error.
- **No measuring emergency-mute latency server-side** — publisher-client only (R-PUBCLIENT, watchpoint #2). Catch-fire.
- **No "implement the pilot" as a cargo impl-task** — D2 is operational (Half B, Task 7, no cargo DoD). A §13 task that tries to implement the pilot is a SCOPE ERROR → catch-fire (watchpoint #6).
- **No strict presigned-URL ACL / retention / tombstone / GDPR-erasure** — DEFERRED (D5 Option C). Participant-floor only.
- **No cross-instance recording-store federation** — recordings stay instance-local (OQ-V2-07). The 2nd instance (Task 1) is for the MUTE marquee, NOT for replicating recordings.
- **No `async-trait` dep** (preferred) — native async-fn-in-trait + generic `maybe_record` (§19 (2)); `async-trait` is the documented fallback IF native surfaces a Send/object-safety issue at validate time (do NOT silently add it — surface a DQ).
- **No turning the M2 `room_provisioning.rs` scaffolds live** (jury/emergency/lifecycle/idempotency `todo!()`s) — those are M2 provisioning, out of Phase-6 scope. Task 6 adds ONLY the two new town-hall criterion fns (142 + 146).

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task.** Task 0 is non-`[P]` (barrier). Task 7 (pilot runbook) is NON-impl (no cargo DoD, no validate-pending gate).

> **Cohort note:** Task 1 (harness, no crate) and Task 2 (bridge src) are sequenced (Task 2's live tests in Tasks 3–6 need Task 1's harness; Task 2 itself has no harness dependency at COMPILE time but Tasks 3–6 do). Tasks 3–6 each edit ONE disjoint integration test file → `[P]`-eligible at the EDIT layer, but the LIVE `-e2e` validation is inherently **laptop-serial** (one stack) and the bridge cold build serializes the `-linux` compile gate; per the cross-lane cap (≤2 running Junior tasks) + the bridge-serial precedent (recording/emergency-mute), the advisor dispatches Tasks 3–6 in ≤2-wide cohorts and the laptop runs the `-e2e` gate serially. Task 4 MAY add a dev-dep (DQ-gated) → if it does, it runs alone (Cargo.lock contention).

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment + branch + base state; confirm the seams Phase 6 EXERCISES are present + the const/migration it must NOT touch are unchanged; confirm MinIO + 2nd instance are genuinely absent (Task 1 is net-new).

**Probes (R5 — enumerate ALL explicitly):**

```bash
# Probe 0 — Docker daemon (cargo-linux.sh + the e2e harness need it)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }
# Probe 1 — Docker in LINUX-container mode (cargo-linux.sh requires it)
docker info --format '{{.OSType}}'                                   # EXPECT: linux
# Probe 2 — on the phase branch
git branch --show-current                                            # EXPECT: phase-m3-core-e2e-pilot
# Probe 3 — registry count UNCHANGED at 72 (Phase 6 adds NO const)
rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs   # EXPECT: 72
# Probe 4 — the 4 chair/mute/recording kinds are SHIPPED (Phase 6 exercises, never re-declares)
rg -c 'ENTRY_KIND_ROOM_(MUTE_ALL|CHAIR_OVERRIDE|CHAIR_TRANSFERRED|RECORDING_UPLOADED)' \
  crates/db_schema/src/source/governance/governance_log.rs           # EXPECT: 4
# Probe 5 — the recording live-trigger stubs to replace are present (bail!)
rg -c 'scaffold-only' services/bridge/src/recording.rs               # EXPECT: 2 (trigger_egress + upload)
# Probe 6 — the cr-2/cr-3 scaffold points are present (header placeholder + empty participant set)
rg -c 'x-requester-pseudonym|let participants: Vec<String> = Vec::new' services/bridge/src/appservice.rs  # EXPECT: 2
# Probe 7 — the seams the live tests EXERCISE are present
rg -c 'pub fn mute_all|pub fn record_uploaded' services/bridge/src/stage.rs                  # EXPECT: 2
rg -c 'pub\(crate\) async fn mute_all_power_levels' services/bridge/src/mute_handler.rs      # EXPECT: 1
rg -c 'pub fn is_participant|fn handle_recording_fetch' services/bridge/src/{recording,appservice}.rs  # EXPECT: >=2
# Probe 8 — MinIO + 2nd instance are GENUINELY ABSENT in the base compose (Task 1 is net-new)
rg -c 'minio:|tuwunel-b:|bridge-b:' services/bridge/docker-compose.yml                       # EXPECT: 0
# Probe 9 — the override-layering precedent + the instance shape to mirror are present
test -f docker/docker-compose-fed-enable.yml && test -f services/bridge/docker-compose.pilot.yml && echo "PRECEDENTS OK"
# Probe 10 — the 4 integration scaffolds are #[ignore] todo!() (the live-test targets)
rg -c 'todo!\("implement against live docker-compose stack' \
  services/bridge/tests/{stage_mode,emergency_mute,recording,room_provisioning}.rs           # EXPECT: 1,1,3,5
# Probe 11 — bridge baseline COMPILES on Linux (cold ~10-20 min; warm after)
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-e2e-task0-bridge-check.log 2>&1
echo "bridge baseline exit: $?"; tail -20 .claude/PRPs/debug/m3-e2e-task0-bridge-check.log  # EXPECT: exit 0
# Probe 12 — the 4 integration scaffolds COMPILE (--no-run) on the base (regression guard)
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --no-run \
  > .claude/PRPs/debug/m3-e2e-task0-bridge-itc.log 2>&1
echo "itc exit: $?"; tail -20 .claude/PRPs/debug/m3-e2e-task0-bridge-itc.log                 # EXPECT: exit 0
# Probe 13 (negative) — wrapper propagates failure
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/m3-e2e-task0-neg.log 2>&1"
echo "negative exit: $?"                                             # EXPECT: NON-ZERO
```

**EXPECT:** Probes 0–12 succeed per their inline expectations (Probe 8 EXPECT 0 = net-new confirmation); Probe 13 exits non-zero. **No commit at Task 0.**

### Task 1: e2e acceptance harness — MinIO + 2nd federated instance override + reach-the-containers smoke

**ACTION:** author `services/bridge/docker-compose.e2e.yml` (override adding MinIO + a second federated tuwunel+bridge instance with a distinct domain) + `scripts/brehon/e2e-harness-smoke.sh` (the reach-the-containers proof); confirm the harness reaches every container BEFORE any acceptance test depends on it.

**FILES:**

```yaml
creates:
  - services/bridge/docker-compose.e2e.yml
  - scripts/brehon/e2e-harness-smoke.sh
modifies: []   # may also CREATE registration-b.yaml + a tuwunel-b config if the base files don't parameterise the domain
```

**IMPLEMENT:** per §10.1 + §10.2 — the override compose adds `minio:` (generic-S3, `S3_ENDPOINT` wired to the bridge, R-S3ENDPOINT keeps the literal OUT of source), `tuwunel-b:` + `bridge-b:` with a **distinct `MATRIX_SERVER_NAME`/`CONDUIT_SERVER_NAME` domain** (`matrix-b.localhost` vs `matrix-a.localhost` — R-DOMAIN/BUG-15), and a federation link between the two homeservers; the reach-smoke brings the stack up and asserts MinIO-health + LiveKit + bridge-A + tuwunel-B federation reachability, printing `E2E_HARNESS_REACH_OK`. Confirm whether `registration.yaml`/`tuwunel-pilot.toml` parameterise the domain or need a B-variant; if a B-variant is needed, clone-with-distinct-domain.

**MIRROR:** §10.1, §10.2; `docker/docker-compose-fed-enable.yml` (layering), `services/bridge/docker-compose.pilot.yml:12-89` (instance shape), `services/bridge/registration.yaml` + `tuwunel-pilot.toml` (AS/HS config).

**GOTCHA:** the 2nd instance needs a DISTINCT DOMAIN, not just a distinct port (BUG-15 — Lemmy/Matrix key by port-stripped domain; same host:port → the cross-instance mute test can't distinguish instances). MinIO image tag is pinned + confirmed at smoke time. Federation between two local homeservers needs both `ALLOW_FEDERATION: true` + reachable `server_name` resolution (the `extra_hosts`/network alias) — verify at smoke. This is infra config + a smoke script; it adds NO Rust, NO dep, NO cargo.

**VALIDATE (laptop — Docker, NOT cargo):** Task 1 produces a `validate-pending-laptop-e2e` DQ whose `commands` is the reach-smoke (`bash scripts/brehon/e2e-harness-smoke.sh`); the laptop runs it. EXPECT: `E2E_HARNESS_REACH_OK` + all probes green. **A failing reach-smoke STOPS the phase** (acceptance failures would be unattributable — bootstrap stop-and-ask). The worker authors the files + writes the DQ + stops; it does NOT bring up the stack (no daemon Docker for this heavy stack).

### Task 2: Recording carry-forwards (cr-2 + cr-3) + live `LiveSink` (async trigger + S3 PUT)

**ACTION:** make `RecordingSink` async + `maybe_record` generic; replace the two `LiveSink` `bail!` with the live Egress POST + S3 PUT; wire cr-2 (requester pseudonym from the Bearer-authed forward, reject empty) + cr-3 (real participant set) in `handle_recording_fetch`.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/src/recording.rs    # RecordingSink async; maybe_record generic+async; LiveSink live; tests -> #[tokio::test]
  - services/bridge/src/appservice.rs   # cr-2 reject-empty + cr-3 real participant set + real media_url in handle_recording_fetch
  - services/bridge/src/bridge_room.rs  # participants_for_recording helper IF no existing membership query fits (check lookup/lookup_by_case first)
requires:
  - task: 1
    reason: the live LiveSink S3 PUT + the participant-fetch are exercised against Task 1's MinIO + harness at the -e2e gate (compile is independent; the live run needs the stack)
```

**IMPLEMENT (file 1 of ≤3):** `recording.rs` per §10.3 — convert `RecordingSink::{trigger_egress,upload}` to `async fn` (native, Rust 1.95; NO `async-trait` dep — §19 (2)); make `maybe_record<S: RecordingSink>(sink: &mut S, …)` async (drop `&mut dyn`); replace the two `LiveSink` `bail!` (`:76`, `:104`) with the live async Egress POST (`reqwest` + `livekit_jwt` token to `{livekit_url}/twirp/livekit.proto.Egress/StartRoomCompositeEgress`, `.error_for_status()?`) + the live S3 PUT (`rust-s3::Bucket::put_object`, endpoint/bucket/creds from `config` — R-S3ENDPOINT); convert the `Recorder` spy methods + `clean_posture_no_side_effects_when_disabled` + `maybe_record_enabled_records_sink_and_emits_intent` to `#[tokio::test]` + `.await`. **The clean-posture negative invariant + the delete-the-gate check are PRESERVED (R7).**
**IMPLEMENT (file 2 of ≤3):** `appservice.rs` per §10.4 — cr-2: reject an empty requester pseudonym (`403`) after the Bearer gate (the trust boundary); cr-3: replace `let participants: Vec<String> = Vec::new();` (`:269`) with the real participant-set lookup; resolve the real `media_url` (from `bridge_room`/the recording record) instead of the `format!("{id}.mp4")` placeholder. `is_participant` stays called BEFORE serving.
**IMPLEMENT (file 3 of ≤3, IF NEEDED):** `bridge_room.rs` — `participants_for_recording(conn, recording_id) -> Result<Vec<String>>` (pseudonyms only) IF no existing membership query fits; otherwise reuse `lookup`/`lookup_by_case`.

**MIRROR:** §10.3, §10.4; `services/bridge/src/recording.rs:17-106`, `:115-263`; `services/bridge/src/appservice.rs:238-284`; `services/bridge/src/bridge_room.rs:47,59`.

**GOTCHA (async ripple):** turning the trait async makes `maybe_record` async → the 2 deterministic tests need `#[tokio::test]` + `.await`; the `Recorder` spy methods become `async fn`. **GOTCHA (no async-trait):** use native async-fn-in-trait + generic `S` (object-safety-free); if native surfaces a Send/`dyn` issue at validate, surface a DQ before adding `async-trait` (do NOT silently add a dep). **GOTCHA (R-S3ENDPOINT):** the S3 endpoint stays config-sourced — `rg -n 'minio\.' services/bridge/src/` returns nothing. **GOTCHA (R9/cr-2):** the requester pseudonym is trusted ONLY because the caller proved BRIDGE_CALLBACK_SECRET; reject empty. **GOTCHA (R-CHAIN):** `content_sha256` still rides `append_room_event` via `record_uploaded` → `drain_emits` — no direct chain write in `recording.rs`. Bridge compiles on **Linux only**.

**VALIDATE (Linux compile gate):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-e2e-task2-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-e2e-task2-bridge-check.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml clean_posture_no_side_effects_when_disabled \
  > .claude/PRPs/debug/m3-e2e-task2-bridge-clean.log 2>&1
echo "clean exit: $?"; tail -20 .claude/PRPs/debug/m3-e2e-task2-bridge-clean.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop-linux` DQ (`commands` = `cargo-linux.sh check` + `clippy --no-deps -- -D warnings` + `test … clean_posture_no_side_effects_when_disabled` + `test … maybe_record_enabled_records_sink_and_emits_intent`), commit + push, then **stop**.

### Task 3 [P]: `stage_mode.rs` live — 4-user mic-pass + 30s grace + chair override + chair transfer (criteria 136–140)

**ACTION:** turn `four_mic_pass_then_grace_boundary_emits_chair_entries` from `todo!()` into a live integration test against Task 1's harness.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/tests/stage_mode.rs   # four_mic_pass_then_grace_boundary_emits_chair_entries — live body
requires:
  - task: 1
    reason: runs against the live LiveKit + tuwunel harness (the -e2e gate)
```

**IMPLEMENT:** per the scaffold step comments (`stage_mode.rs:14-22`) — provision a town-hall stage room (chair + 4 watchers); drive 4 raise-hand + promote/activate passes via the real LiveKit grants; drive a 5th promote with no activation, assert auto-revoke + next-promote at the 30s grace boundary; drive a chair transfer + a chair override; query `governance_log` and assert `room_chair_transferred {from_pseudonym, to_pseudonym, at}` + `room_chair_override {action, target_pseudonym}` rows exist with PSEUDONYM payload fields (ADR-015). Keep `#[tokio::test] #[ignore = "requires docker-compose stack"]` (it needs the live stack). **R-ANCHOR:** `grep -c 'todo!("implement against live docker-compose stack (Phase-6 pilot grade)")' services/bridge/tests/stage_mode.rs` == 1 before editing.

**MIRROR:** the scaffold body + `stage.rs:105-338` (promote_next/grace/chair_override/transfer_chair seams) + `services/bridge/tests/room_provisioning.rs` (the `AsyncPgConnection::establish` + `governance_log` query idiom).

**GOTCHA:** assert on OBSERVABLE state (the `governance_log` rows + the LiveKit grant transitions), not config (`feedback_build_what_tests_exercise.md`). PSEUDONYMS only in the asserted payload fields (ADR-015). Bridge compiles on **Linux only**; the live run is the `-e2e` gate.

**VALIDATE (Linux compile gate + e2e live gate):**

```bash
# compile (worker-writable -linux gate):
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test stage_mode --no-run \
  > .claude/PRPs/debug/m3-e2e-task3-itc.log 2>&1
echo "itc exit: $?"; tail -20 .claude/PRPs/debug/m3-e2e-task3-itc.log   # EXPECT: exit 0
```

Write TWO DQs: (1) a `validate-pending-laptop-linux` (`commands` = `cargo-linux.sh check` + `clippy --no-deps -- -D warnings` + `test --test stage_mode --no-run`); (2) a `validate-pending-laptop-e2e` (`commands` = `cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test stage_mode -- --ignored`, `e2e_filter: null` — bridge test target, run after `e2e-harness-smoke.sh` brings the stack up; **gate-4 local-vs-dispatch is the advisor's user gate**). Commit + push, then **stop**.

### Task 4 [P]: `emergency_mute.rs` live — the MARQUEE cross-instance <500ms publisher-client mute (criterion 141)

**ACTION:** turn `mute_all_drops_all_publishers_cross_instance_under_500ms` from `todo!()` into the live cross-instance integration test, measured at the PUBLISHER CLIENT.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/tests/emergency_mute.rs   # mute_all_drops_all_publishers_cross_instance_under_500ms — live body
  # MAY ALSO modify (DQ 3004b6625b83-001 option-A/B): services/bridge/Cargo.toml + Cargo.lock (livekit-client dev-dep)
requires:
  - task: 1
    reason: needs the 2nd federated instance (cross-instance) + LiveKit from Task 1
```

**IMPLEMENT:** per §10.5 + the scaffold body (`emergency_mute.rs:14-48`) — provision a federated town-hall stage room across the TWO bridge instances (chair + N publishers, some homed on instance-B so the cross-instance path is exercised); record a per-publisher publish-active baseline **at the publisher client** (NOT the server); fire mute-all (the cross-instance `mute_all_power_levels` Matrix power-level PUT + the local `Stage::mute_all` LiveKit `RevokePublish` sweep); assert EVERY non-chair publisher's publish right is revoked **at the publisher client** within 500ms (in-instance = automated guarantee; cross-instance per the DQ-resolved mechanism + the PRD fallback); assert the cross-instance zero-holder negative invariant (no listed publisher on EITHER instance retains publish); query `governance_log` and assert a `room_mute_all` row with `actor_pseudonym` = the chair PSEUDONYM + `payload.federated == true`. **The publisher-client measurement mechanism is DQ `3004b6625b83-001` (gate-1 ratify) — implement per the resolved option.** The PRD fallback is DOCUMENTED via the DQ, never silently dropped. **R-ANCHOR:** the `todo!()` is the unique anchor (one fn in the file).

**MIRROR:** §10.5; `services/bridge/tests/emergency_mute.rs:14-48`; `stage.rs:339-366` (mute_all + cr-11 chair-exclusion) + `:727-779` (the set-equality zero-holder idiom); `mute_handler.rs:64-98` (the power-level PUT).

**GOTCHA (R-PUBCLIENT — CATCH-FIRE):** measure at the publisher client, NEVER the server — a server-side measurement is a FALSE GREEN. **GOTCHA (DQ-gated dep):** do NOT add the livekit-client dev-dep until DQ `3004b6625b83-001` is resolved; if it lands, `Cargo.lock` regenerates in the SAME commit (R14-class) and this task runs ALONE (Cargo.lock contention). **GOTCHA (R-DOMAIN):** the cross-instance path requires the two instances on DISTINCT domains (Task 1). Bridge compiles on **Linux only**.

**VALIDATE (Linux compile + e2e live):** same two-DQ shape as Task 3 (`--test emergency_mute`). The `-e2e` DQ's `commands` = `cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test emergency_mute -- --ignored` (run after the harness is up). Commit + push, then **stop**. **This is the headline `-e2e` gate** — the cross-instance <500ms is proven HERE (automated in-instance) + at the D2 pilot (cross-instance).

### Task 5 [P]: `recording.rs` live — recording-lands + clean-posture-off + participant-floor fetch (criteria 143/145/144)

**ACTION:** turn the 3 `recording.rs` scenarios from `todo!()` into live integration tests against Task 1's MinIO + Task 2's live LiveSink + fetch.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/tests/recording.rs   # recording_lands_with_hash_on_chain + clean_posture_no_recording_when_disabled + participant_floor_fetch — live bodies
requires:
  - task: 1
    reason: needs MinIO + LiveKit from the harness
  - task: 2
    reason: exercises the live LiveSink (Egress POST + S3 PUT) + cr-2/cr-3 participant-floor fetch
```

**IMPLEMENT:** per the scaffold bodies (`recording.rs:14-55`) — (1) `recording_lands_with_hash_on_chain`: provision a `record_town_halls=true` room; trigger Egress (live LiveSink); assert the MP4 object exists in MinIO (S3 object-exists, endpoint from config — R-S3ENDPOINT); assert a `governance_log` `room_recording_uploaded` row carries `{media_url, content_sha256, duration_s, speakers, attendance_count}` where `content_sha256` matches `compute_content_sha256(mp4_bytes)` AND the row carries a valid signature/prev-hash link (R-CHAIN — the hash RODE `append_room_event`, not a bypass); assert `speakers` + actor are PSEUDONYMS (ADR-015). (2) `clean_posture_no_recording_when_disabled`: provision a `record_town_halls=false` room; run the session; assert NO MinIO object, NO `room_recording_uploaded` chain row, NO Egress call (the integration-level clean-posture negative invariant — R7). (3) `participant_floor_fetch`: non-participant fetch → 403, participant fetch → 200 + `media_url` (exercises cr-2/cr-3). **R-ANCHOR:** anchor each edit on the enclosing fn signature first (3 `todo!()`s in the file), then the `todo!()` line.

**MIRROR:** the scaffold bodies; `recording.rs:108-263` (`is_participant` + Recorder); `appservice.rs:238-284` (the live fetch from Task 2); `stage.rs:373-410` (`record_uploaded`).

**GOTCHA (R-CHAIN — CATCH-FIRE):** `recording_lands_with_hash_on_chain` asserts the `content_sha256` is in the `governance_log` chain row with a valid signature, NOT merely in the MinIO object metadata. **GOTCHA (R7):** the clean-posture test FAILS if the `maybe_record` flag-gate is removed; it is a NEGATIVE invariant (no Egress / no S3 / no chain row), not a happy-path skip. Bridge compiles on **Linux only**.

**VALIDATE:** same two-DQ shape (`--test recording`). The `-e2e` DQ runs `cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test recording -- --ignored` after the harness is up. Commit + push, then **stop**.

### Task 6 [P]: `room_provisioning.rs` — `rtc_enabled=false` town-hall clean-posture + anonymous identity (criteria 146/142)

**ACTION:** add two new live test fns covering the `rtc_enabled=false` clean-posture (146) + the anonymous-town-hall identity-never-reaches-LiveKit (142) criteria, alongside the existing M2 scaffolds (which stay out-of-scope).

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/tests/room_provisioning.rs   # ADD rtc_disabled_townhall_clean_posture + anonymous_townhall_identity_never_reaches_livekit
requires:
  - task: 1
    reason: the anonymous-identity live check connects to the LiveKit harness; the clean-posture check needs the stack to assert ZERO RTC provisioning
```

**IMPLEMENT:** add two new `#[tokio::test] #[ignore = "requires docker-compose stack"]` fns. (1) `rtc_disabled_townhall_clean_posture` (criterion 146): with `rtc_enabled=false`, drive a town-hall event; assert ZERO RTC provisioning (no LiveKit room, no MinIO, no chair seat) AND the governance flow is unchanged; **mechanical R7 check:** the test FAILS if the `rtc_enabled` gate is forced on. Cite the `crates/server/tests/e2e.rs` governance suite for the "governance flow passes unchanged with RTC disabled" half (criterion 146 names `cargo test --test e2e`); this bridge test owns the "zero RTC provisioning / no LiveKit/MinIO calls" half. (2) `anonymous_townhall_identity_never_reaches_livekit` (criterion 142): provision an `always_pseudonym` town hall; mint a participant token; connect; assert the LiveKit-server-received identity == the PSEUDONYM (never the Lemmy username/`person_id`/MXID). Cite `livekit_jwt::mint_pseudonym_claims` (`:69`) as the JWT-issue-time unit anchor (§10.6); this test adds the server-received-identity live check. Do NOT touch the existing M2 `todo!()` scaffolds (jury/emergency/lifecycle/idempotency/messaging-disabled — out of Phase-6 scope). **R-ANCHOR:** these are NEW fns appended after the existing ones — no `todo!()` collision; place after `messaging_disabled_prevents_provisioning` (`:64`).

**MIRROR:** `services/bridge/tests/room_provisioning.rs:55-64` (`messaging_disabled_prevents_provisioning` — the clean-posture model) + `livekit_jwt.rs:69` (`mint_pseudonym_claims`).

**GOTCHA (R7):** the `rtc_enabled=false` clean-posture is a NEGATIVE invariant — assert ZERO RTC side-effects + FAIL-if-gate-forced-on. A trivial always-skip is invalid (§3.5). **GOTCHA (ADR-015):** the anonymous-identity test asserts the LiveKit server NEVER receives the real identity. Bridge compiles on **Linux only**.

**VALIDATE:** same two-DQ shape (`--test room_provisioning`, `-- --ignored`). Commit + push, then **stop**.

### Task 7: D2 pilot run — operator-run acceptance runbook (Half B, NON-impl, NO cargo DoD)

**ACTION:** author `.claude/PRPs/runbooks/m3-core-e2e-pilot-d2-runbook.md` — a human-run operational acceptance checklist + go/no-go criteria for the pilot town hall. **This is NOT impl: no cargo, no `validate-pending-laptop` gate; its DoD is a retro-recorded outcome.**

**FILES:**

```yaml
creates:
  - .claude/PRPs/runbooks/m3-core-e2e-pilot-d2-runbook.md
modifies: []
```

**IMPLEMENT:** the runbook scopes (per clarify-DQ `a3d0e9941441-075` — supports EITHER scenario, operator picks at run time): (1) **Pre-flight** — bring up the full RTC stack on the pilot server (`http://100.81.145.58:1236`, accounts in `reference_pilot_test_accounts.md`); confirm the gov→Matrix path (pilot-internal phases 1+2 verified — `workflow_state_pilot_internal.md`); confirm which API path fires the bridge hook (`feedback_pilot_governance_workflow_seeding_order.md`: `/governance/appeal` auto-seat fires + provisions; `admin_trigger_appeal_rejury` does NOT). (2) **Scenario (operator picks)** — community-deliberation broadcast OR appeal hearing with audience; seed the case to the hook-firing transition. (3) **The acceptance script** — open the town hall in stage mode; the chair holds the floor; the chair passes the mic to ≥1 user; OPTIONALLY enable recording and confirm it lands with its `content_sha256` hash entry on the chain. (4) **Go/no-go criteria** — GO: a real town hall ran, the chair passed the mic to ≥1 user (recording optional). NO-GO: the stack didn't come up, the chair couldn't pass the mic, or (if recording enabled) it didn't land. (5) **Blast-radius coordination** (`feedback_pilot_resilience_test_blast_radius.md`) — the bridge/Tuwunel containers are infra-owned; `messaging_enabled=false` is an instance-wide kill-switch; coordinate in the pilot shared-state file. (6) **DoD** — a recorded outcome in the retro (did a real town hall run? did the chair pass the mic? did recording land?), NOT a test pass. **The advisor coordinates + runs the pilot; this task only authors the runbook.**

**MIRROR:** `reference_pilot_test_accounts.md`, `workflow_state_pilot_internal.md`, `feedback_pilot_governance_workflow_seeding_order.md`, `feedback_pilot_resilience_test_blast_radius.md`.

**GOTCHA (watchpoint #6 — CATCH-FIRE):** this task authors a RUNBOOK, it does NOT implement the pilot as cargo work. A task that tries to "implement the pilot" (any cargo DoD, any `validate-pending-laptop` gate) is a SCOPE ERROR. The runbook is `.claude/**` meta-work (no PR-flow code; direct-commit class per `phase-branch.md`).

**VALIDATE:** none (NON-impl, no cargo). The runbook is reviewed at gate-1/the retro; the actual pilot run is the advisor-coordinated Half-B acceptance step recorded in the retro.

### Task 8: Retro

**Goal:** author the retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` — one H2 per role (Advisor / Planning / Impl / BM) with signals + lessons + per-task complexity scores (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`). Promote any new lessons to `.claude/lessons/feedback_*.md` in the retro commit. **Mandatory Half-B record (bootstrap §6):** the pilot outcome — did a real town hall run end-to-end? did the chair pass the mic to ≥1 user? did recording land with its hash entry? (the D2 acceptance signal, NOT a test pass). Phase-specific signals: did the e2e harness (MinIO + 2nd instance + reach-smoke) come up first-try, or surface federation/domain/network issues (BUG-15)?; did the marquee publisher-client <500ms measurement land per the DQ-resolved option, and was the cross-instance bar met or did the PRD fallback fire?; did the `RecordingSink` sync→async conversion (native async-fn-in-trait, no `async-trait`) compile clean, or want `async-trait`?; did cr-2/cr-3 land bridge-side (zero `crates/**` change), or need a binary route?; did MiniMax governance-ai-review fire on a Phase-6 fix-in-PR (recording retro §8)? Record the gate-4 (local-vs-dispatch) decision the user made.

---

## 14. Testing strategy

- **bridge static (Linux):** `cargo-linux.sh check/clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings` (Tasks 2–6).
- **bridge unit (Linux, no docker):** the converted deterministic recording tests (`clean_posture_no_side_effects_when_disabled`, `maybe_record_enabled_records_sink_and_emits_intent`) under `#[tokio::test]` (Task 2) — the always-on negative-invariant DoD that does NOT need the stack.
- **bridge integration compile (Linux, `--no-run`):** Tasks 3–6 each compile their `--ignored` test target — the worker-writable `-linux` gate (no stack needed).
- **bridge integration LIVE (Linux, `-- --ignored`, needs the full harness):** the acceptance run — `e2e-harness-smoke.sh` up, then `cargo-linux.sh test --test <name> -- --ignored` per target (Tasks 3–6). This is the gate-4 `validate-pending-laptop-e2e` run (laptop or dispatch — user picks ONCE; never auto-pick after PR #105). The marquee carries the PRD fallback (in-instance automated / cross-instance pilot).
- **crates governance e2e (criterion 146 half):** the existing `crates/server/tests/e2e.rs` governance suite passes unchanged with RTC disabled (cited by Task 6, not re-implemented).
- **No migration round-trip** (no new migration). **No new const** (registry frozen at 72). **D2 pilot = operator-run, not a cargo gate** (Task 7).

## 15. Validation commands (DoD)

> Every command is written in the exact form the advisor runs at gate 1 (`feedback_plan_dod_dry_run_at_write.md`); all dry-run clean against `phase-m3-core-e2e-pilot` HEAD. Bridge cargo uses `cargo-linux.sh --manifest-path` (NEVER Windows-local — `ruma-common` E0119; watchpoint #3). The plan distinguishes the COMPILE gate (`-linux`) from the LIVE integration gate (`-e2e`, needs the stack).

### 15.1 Harness reach-smoke (Task 1 — laptop Docker, NOT cargo)

```bash
bash scripts/brehon/e2e-harness-smoke.sh > .claude/PRPs/debug/m3-e2e-harness-smoke.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-e2e-harness-smoke.log   # EXPECT: 0 + "E2E_HARNESS_REACH_OK"
```

### 15.2 bridge static + lint (Tasks 2–6 — Linux, Docker rust:1.95)

```bash
scripts/brehon/cargo-linux.sh check  --manifest-path services/bridge/Cargo.toml > .claude/PRPs/debug/m3-e2e-bridge-check.log 2>&1;  echo "exit: $?"  # EXPECT: 0
scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings > .claude/PRPs/debug/m3-e2e-bridge-clippy.log 2>&1; echo "exit: $?"  # EXPECT: 0
```

### 15.3 bridge unit (Task 2 — Linux, no docker; the always-on negative-invariant DoD)

```bash
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml clean_posture_no_side_effects_when_disabled > .claude/PRPs/debug/m3-e2e-clean.log 2>&1; echo "exit: $?"  # EXPECT: 0
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml maybe_record_enabled_records_sink_and_emits_intent > .claude/PRPs/debug/m3-e2e-enabled.log 2>&1; echo "exit: $?"  # EXPECT: 0
```

### 15.4 bridge integration compile (Tasks 3–6 — Linux, `--no-run`; the worker-writable -linux gate)

```bash
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --no-run > .claude/PRPs/debug/m3-e2e-bridge-itc.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-e2e-bridge-itc.log   # EXPECT: 0 (all 4 #[ignore] targets compile)
```

### 15.5 bridge integration LIVE (gate-4 — Linux + full harness; user picks local-vs-dispatch ONCE)

```bash
# Pre: bash scripts/brehon/e2e-harness-smoke.sh  →  E2E_HARNESS_REACH_OK
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test stage_mode      -- --ignored  # criteria 136-140
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test emergency_mute  -- --ignored  # criterion 141 (MARQUEE; PRD fallback documented)
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test recording       -- --ignored  # criteria 143/145/144
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test room_provisioning -- --ignored # criteria 146/142 (the two new fns)
# EXPECT each: the named live test passes (or, for the marquee cross-instance bar, the PRD fallback fires + is DQ-documented)
```

### 15.6 Cross-cutting verification (planner asserts at end-of-phase)

- [ ] R8: NO new migration / no new const / no registry bump — `git diff --stat governance-v0..HEAD -- crates/db_schema/ migrations/` is EMPTY; `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns **72**.
- [ ] Zero `crates/**` change — `git diff --stat governance-v0..HEAD -- crates/` is EMPTY (cr-2/cr-3 landed bridge-side; the only criterion-146 `crates/server/tests/e2e.rs` touch is a CITATION, not an edit).
- [ ] R-CHAIN: `rg -n 'append|governance_log|INSERT' services/bridge/src/recording.rs` shows NO direct chain write; `content_sha256` reaches the chain ONLY via `record_uploaded` → `drain_emits` → `append_room_event`; the live `recording_lands_with_hash_on_chain` asserts the chain row + signature.
- [ ] R-S3ENDPOINT: `rg -n 'minio\.' services/bridge/src/` returns nothing (or comments only); the S3 endpoint reads from `config.s3_endpoint` (the override compose may name `minio:9000` — config wiring, not source).
- [ ] R-PUBCLIENT: the marquee test measures at the publisher client (NOT the server); `rg -n 'server|power_levels' services/bridge/tests/emergency_mute.rs` shows the assertion is on the publisher-client publish state, not a server-side query; the PRD fallback is DQ-documented.
- [ ] R-DOMAIN: `services/bridge/docker-compose.e2e.yml` names two DISTINCT domains for the two instances (BUG-15).
- [ ] R9 (ADR-015): `rg -i 'person_id|username|@.*:' services/bridge/src/recording.rs services/bridge/src/appservice.rs` returns nothing identity-shaped; `handle_recording_fetch` calls `is_participant` BEFORE serving + rejects an empty requester pseudonym (cr-2).
- [ ] R7: both clean-posture tests (recording-off + `rtc_enabled=false`) assert the NEGATIVE invariant + FAIL-if-gate-deleted; neither is a trivial always-skip.
- [ ] No `async-trait` dep added (unless DQ-surfaced): `rg -c 'async-trait' services/bridge/Cargo.toml` returns 0.
- [ ] §16a maps EVERY §Success-Criteria row (134–149) to exactly one live test fn (or a cited existing anchor / the D2 pilot for 149).

## 16. Acceptance criteria

- [ ] Tasks 0–8 completed in dependency order
- [ ] §15.1 (harness reach-smoke) → `E2E_HARNESS_REACH_OK` (Task 1)
- [ ] §15.2 (bridge check + clippy) exit 0 (Tasks 2–6)
- [ ] §15.3 (bridge unit, the negative-invariant DoD) exit 0 (Task 2)
- [ ] §15.4 (4 integration targets compile `--no-run`) exit 0 (Tasks 3–6)
- [ ] §15.5 (LIVE acceptance run, gate-4) — each named live test passes; the marquee cross-instance bar met OR the PRD fallback fired + DQ-documented
- [ ] §15.6 cross-cutting boxes all ticked
- [ ] §16a Stories all `[done]`
- [ ] **D2 pilot ran end-to-end** (Half B) — recorded in the retro (chair passed the mic; recording optional)
- [ ] No edits outside §11; registry frozen at 72; no migration; no new prod dep; zero `crates/**` change
- [ ] Retro committed (Task 8)
- [ ] **Linux-compile gate:** every bridge-touching task's `validate-pending-laptop-linux` DQ at `result:pass` before `bm-pr`
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`

## 16a. Stories

> Every §Success-Criteria row (PRD 134–149) maps to exactly one live test fn (bare name) or a cited anchor / the D2 pilot. The marquee + both clean-postures satisfy the delete-the-gate check; ADR-015 pins are asserted on observable state.

### Story 1: Stage mode opens + 4-user mic-pass + 30s grace + chair override + chair transfer (criteria 136–140)

- **Composing tasks:** Task 1 (harness), Task 3 (`stage_mode.rs`)
- **Checkpoint command (compile):** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test stage_mode --no-run`
- **Checkpoint command (live, gate-4):** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test stage_mode -- --ignored`
- **Expected output:** `four_mic_pass_then_grace_boundary_emits_chair_entries` passes — 4 mic-passes in sequence; the 30s-grace auto-revoke + next-promote; `room_chair_transferred {from_pseudonym, to_pseudonym, at}` + `room_chair_override {action, target_pseudonym}` chain rows with PSEUDONYM fields (ADR-015).
- **Brief-Scope outputs to verify:** `services/bridge/tests/stage_mode.rs` has a live `four_mic_pass…` body (no `todo!()`).

### Story 2: Federation-wide emergency mute <500ms at the publisher client, cross-instance (criterion 141 — MARQUEE)

- **Composing tasks:** Task 1 (2nd instance), Task 4 (`emergency_mute.rs`)
- **Checkpoint command (live):** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test emergency_mute -- --ignored`
- **Expected output:** `mute_all_drops_all_publishers_cross_instance_under_500ms` passes — EVERY non-chair publisher's publish right revoked **at the publisher client** within 500ms (in-instance automated; cross-instance per DQ `3004b6625b83-001` + the PRD fallback, documented not dropped); the cross-instance zero-holder negative invariant; a `room_mute_all` chain row with `actor_pseudonym` = chair pseudonym + `payload.federated == true`. **Mechanical check (must hold):** the measurement is at the publisher client, NOT the server (a server-side measurement is a FALSE GREEN — R-PUBCLIENT catch-fire).
- **Brief-Scope outputs to verify:** `services/bridge/tests/emergency_mute.rs` has a live body asserting publisher-client revoke timing + the chain row.

### Story 3: Recording lands in MinIO with its `content_sha256` chain entry (criterion 143)

- **Composing tasks:** Task 1 (MinIO), Task 2 (live LiveSink), Task 5 (`recording.rs`)
- **Checkpoint command (live):** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test recording -- --ignored recording_lands_with_hash_on_chain`
- **Expected output:** the MP4 object exists in MinIO; a `governance_log` `room_recording_uploaded` row carries the schema with `content_sha256 == compute_content_sha256(mp4_bytes)` AND a valid signature/prev-hash link (R-CHAIN — the hash RODE `append_room_event`); `speakers` + actor are pseudonyms (ADR-015).
- **Brief-Scope outputs to verify:** `services/bridge/tests/recording.rs::recording_lands_with_hash_on_chain` live body; `services/bridge/src/recording.rs` `LiveSink` has live Egress + S3 PUT (no `bail!`).

### Story 4: Participant-floor recording fetch — non-participant 403, participant 200 (criterion 144, cr-2/cr-3)

- **Composing tasks:** Task 2 (cr-2/cr-3), Task 5 (`participant_floor_fetch`)
- **Checkpoint command (live):** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test recording -- --ignored participant_floor_fetch`
- **Expected output:** a non-participant pseudonym → 403; a participant pseudonym → 200 + `media_url`. The handler calls `is_participant` BEFORE serving + rejects an empty requester pseudonym (cr-2); the participant set is the real `bridge_room` membership (cr-3).
- **Brief-Scope outputs to verify:** `services/bridge/src/appservice.rs` `handle_recording_fetch` has the real participant set (no `Vec::new()` stub) + the empty-pseudonym rejection.

### Story 5: `record_town_halls = false` clean-posture — zero recording side-effects (criterion 145, negative invariant)

- **Composing tasks:** Task 2 (the always-on unit), Task 5 (the live integration variant)
- **Checkpoint command (unit, always-on):** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml clean_posture_no_side_effects_when_disabled`
- **Checkpoint command (live):** `… --test recording -- --ignored clean_posture_no_recording_when_disabled`
- **Expected output:** `maybe_record(false, …)` → zero Egress/upload/EmitIntent (unit); the live variant → NO MinIO object, NO `room_recording_uploaded` chain row, NO Egress call. **Mechanical check (must hold):** deleting the `if !enabled { return }` flag-gate makes BOTH FAIL (R7); the positive `enabled=true` companion proves the test asserts the GATE.
- **Brief-Scope outputs to verify:** `services/bridge/src/recording.rs` `maybe_record` retains the `if !enabled { return }` gate; `tests/recording.rs::clean_posture_no_recording_when_disabled` live body.

### Story 6: `rtc_enabled = false` clean-posture + anonymous-town-hall identity (criteria 146 + 142)

- **Composing tasks:** Task 6 (`room_provisioning.rs`), Task 1 (harness for the live checks)
- **Checkpoint command (live):** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test room_provisioning -- --ignored rtc_disabled_townhall_clean_posture anonymous_townhall_identity_never_reaches_livekit`
- **Expected output:** (146) `rtc_enabled=false` → ZERO RTC provisioning (no LiveKit/MinIO calls, no chair seat), governance flow unchanged (the `crates/server/tests/e2e.rs` governance suite half cited); FAILS if the gate is forced on (R7). (142) the LiveKit-server-received identity == the pseudonym, never the real identity (the `mint_pseudonym_claims` JWT-issue-time anchor cited + the server-received check).
- **Brief-Scope outputs to verify:** `services/bridge/tests/room_provisioning.rs` has the two new fns; the M2 scaffolds stay `todo!()` (untouched).

### Story 7: Pilot town hall runs end-to-end (criterion 149 — D2, Half B, operator-run)

- **Composing tasks:** Task 7 (runbook) + the advisor-coordinated pilot run
- **Checkpoint command:** none (NON-impl) — the runbook's go/no-go criteria, executed by the operator
- **Expected output:** a recorded retro outcome — a real town hall ran; the chair passed the mic to ≥1 user; (optional) recording landed with its `content_sha256` chain entry. NOT a test pass.
- **Brief-Scope outputs to verify:** `.claude/PRPs/runbooks/m3-core-e2e-pilot-d2-runbook.md` exists with the scenario-agnostic checklist + go/no-go; the retro records the pilot outcome.

> **Verification mapping:** `/brehon-verify` iterates this section, runs each Story's checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Story 7 (pilot) is verified by the retro record, not a checkpoint command. Phantoms (task complete but output absent/empty) trigger the catch-fire procedure.

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0–13 per expectations)
- [ ] Tasks 1–6 committed (one commit each); Task 7 runbook committed; Task 8 retro committed
- [ ] §15 validation green at every gate (harness reach-smoke / bridge check+clippy / unit negative-invariant / 4 integration compiles / LIVE acceptance run)
- [ ] §16a Stories 1–7 all `[done]` (Story 7 = retro-recorded pilot outcome)
- [ ] **Linux-compile gate:** every bridge-touching task's `validate-pending-laptop-linux` DQ at `result:pass` before `bm-pr`
- [ ] **gate-4 surfaced** to the user (local-vs-dispatch, once) — the LIVE `-e2e` run path chosen
- [ ] DQ `3004b6625b83-001` (marquee publisher-client measurement) resolved at gate-1 before Task 4 dispatch
- [ ] PR opened by BM against `governance-v0` with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit (+ MiniMax governance-ai-review on small PRs) triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report shows all stories ✓
- [ ] D2 pilot outcome recorded in the retro (Half B)
- [ ] Post-merge phase branch retained for retro reads

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| **Marquee cross-instance <500ms infeasible** (Matrix power-level federation latency, H3 headline, PRD row 198) | **HIGH** | HIGH | The publisher-client measurement is isolated to Task 4 + DQ `3004b6625b83-001` (option-B: automated in-instance <500ms + cross-instance at the D2 pilot). The PRD fallback (best-effort cross-instance SLA + in-instance guarantee) is DOCUMENTED via the DQ before declaring the criterion failed — never silently dropped. `Stage::mute_all`'s local LiveKit sweep gives the deterministic in-instance bar |
| **Server-side mute measurement (FALSE GREEN)** | MED | HIGH | R-PUBCLIENT catch-fire + §16a Story 2 mechanical check + §15.6 grep (`emergency_mute.rs` asserts on publisher-client publish state, not a server query); watchpoint #2 |
| **e2e harness doesn't come up / 2nd-instance federation fails** (BUG-15 domain collision; non-loopback bridge↔LiveKit, OQ-V2-10) | MED | HIGH | Task 1 is the ISOLATED risk-isolation task: the reach-smoke (`E2E_HARNESS_REACH_OK`) proves reachability BEFORE any acceptance test; R-DOMAIN gives the 2nd instance a distinct domain; STOP-the-phase if the reach-smoke fails (bootstrap stop-and-ask) |
| **`RecordingSink` sync→async wants `async-trait`** (native async-fn-in-dyn-trait object-safety) | MED | MED | §19 (2): `maybe_record` is generic over `S` (no `&mut dyn`) so native async-fn-in-trait stays object-safety-free; if native surfaces a Send/`dyn` issue at validate, surface a DQ before adding `async-trait` (do NOT silently add a dep); the `-linux` compile gate catches it |
| **Marquee livekit-client dev-dep fails Linux Cargo.lock resolution** (if DQ picks option-A/B) | MED | MED | Task 4 isolates the dep (runs ALONE; Cargo.lock in the same commit) gated by `validate-pending-laptop-linux`; option-C (no client dep, pilot-only measurement) is the documented fallback |
| **cr-2/cr-3 need a binary-side route** (participant set / session-auth not bridge-resolvable) | LOW | MED | The participant set is in `bridge_room` (cr-3 bridge-side); the requester pseudonym rides the BRIDGE_CALLBACK_SECRET-Bearer-authed forward (cr-2 bridge-side); §12 plans ZERO `crates/**` change — if a binary route IS unavoidable, surface a DQ (do NOT silently edit `crates/**` beyond the genuine cr-2/cr-3 need) |
| **Clean-posture test is a trivial always-skip** (not the negative invariant) | LOW | HIGH | R7 + §16a Stories 5/6 delete-the-gate check + the always-on unit (`clean_posture_no_side_effects_when_disabled`) + the positive companion; advisor §3.5 rejects an always-skip |
| **Live `-e2e` tests flake / can't all pass at gate** (heavy two-instance + MinIO + client stack) | MED | MED | The deterministic units (Task 2) are the always-on DoD; the `-linux` compile gate is worker-writable; the LIVE run is gate-4 (user picks local-vs-dispatch); the marquee carries the PRD fallback; the D2 pilot is the real acceptance signal (D2), not green tests alone |
| **Plan adds a const / migration / `crates/**` feature (scope error)** | LOW | HIGH | §12 + Task 0 Probes 3/4/8 + R8; §15.6 asserts `crates/` + `migrations/` diffs EMPTY + count 72; bootstrap tripwires → STOP + `kind: "blocker"` DQ (watchpoint #4) |
| **Pilot scoped as a cargo impl-task (scope error)** | LOW | HIGH | Task 7 is explicitly NON-impl (no cargo DoD, no validate-pending gate); watchpoint #6 catch-fire if a task tries to implement the pilot |

## 19. Notes

**Two clarify-DQs already resolved by the advisor (baked into the plan, not re-litigated):** `a3d0e9941441-074` — Task 1 uses an OVERRIDE compose `services/bridge/docker-compose.e2e.yml` layered on the base (mirroring `docker-compose-fed-enable.yml`), 2nd instance with a DISTINCT domain (BUG-15); `a3d0e9941441-075` — Task 7's runbook supports EITHER scenario (operator picks at run time), DoD = retro-recorded outcome.

**One pending blocker raised for advisor/user (gate-1):**

**(1) The marquee publisher-client measurement mechanism (DQ `3004b6625b83-001`, `kind: blocker`, `from: planner`).** The brief flags this as the most likely DQ. No LiveKit client SDK is a bridge dep today; measuring "<500ms at the publisher client" needs a client connection. Options: (A) add the `livekit` Rust client as a `[dev-dependencies]`, connect N publisher clients (some cross-instance), timestamp each client's `can_publish→false` flip — fullest automated cross-instance measurement, heaviest dep; (B, RECOMMENDED) use the client dev-dep for an automated IN-INSTANCE publisher-client <500ms (the PRD line-198 in-instance guarantee, measured at the client not the server) AND route the TRUE cross-instance <500ms to the D2 pilot (Element Call browser clients) — documented via this DQ per the scaffold's pre-specified PRD fallback; (C) no automated publisher-client test, rely on the shipped deterministic `Stage::mute_all` zero-holder + `mute_all_power_levels` power-level-PUT tests + observe the publisher-client <500ms only at the pilot. **Planner recommends option-B** — it gives a faithful automated publisher-client signal (not server-side) without over-claiming an automated cross-instance bar that Matrix federation latency may not deliver, and matches the scaffold's documented PRD fallback exactly. **Resolve at gate-1 before Task 4 dispatch.**

**(2) `RecordingSink` sync→async WITHOUT `async-trait` (planner recommendation, advisor ratifies at gate-1).** The recording live trigger (Egress POST + S3 PUT) is async; the shipped `RecordingSink` trait is sync. The cleanest minimal conversion: make the trait methods `async fn` (native, Rust 1.95) + make `maybe_record` generic over `S: RecordingSink` (drop the `&mut dyn`) so the async trait stays object-safety-free — **no `async-trait` dependency**. The only `dyn` use today is `maybe_record(&mut dyn RecordingSink)`; switching to a generic bound removes it. The `Recorder` spy + `LiveSink` both impl the async trait; the 2 deterministic tests become `#[tokio::test]`. **Fallback** (documented, DQ-gated): if native async-fn-in-trait surfaces a `Send`/object-safety issue at validate time, add `async-trait` as the one new dep (Cargo.lock in the same commit) — but surface a DQ first, do NOT silently add it.

**(3) Zero `crates/**` change planned.** cr-2 (requester pseudonym) rides the existing BRIDGE_CALLBACK_SECRET-Bearer-authed forward (bridge-side hardening only); cr-3 (participant set) reads `bridge_room` membership (bridge-side). The only criterion-146 `crates/server/tests/e2e.rs` reference is a CITATION (the governance suite passes unchanged with RTC off), not an edit. If a binary-side participant-set route turns out unavoidable, surface a DQ — do NOT silently edit `crates/**` (the bootstrap trivial-DTO carve-out from recording does NOT apply here; this phase adds no DTO field).

**No new const, no migration, no registry bump, no new prod dep, no in-binary sidecar code.** Phase 6 composes the entire shipped Phases 1–5 control plane into a live, integration-proven + pilot-proven whole. The ONLY net-new surfaces are the e2e harness (override compose + reach-smoke), the recording live-trigger bodies (replacing 2 `bail!`), the cr-2/cr-3 fetch wiring, the 4 live test bodies + 2 new criterion fns, and the D2 runbook. Complexity 7/10 (high end of M3-core; proceed-as-one).

## 20. Confidence score

- **Plan correctness:** 8/10 — every Phase-6 surface is reconciled against the live code @ `2fae33bc1` (the `LiveSink` `bail!` stubs at `recording.rs:76/104`; the cr-2/cr-3 scaffold points at `appservice.rs:261/269`; the absent MinIO/2nd-instance in `docker-compose.yml`; the 5 `#[ignore] todo!()` scaffolds; the shipped `mute_all`/`record_uploaded`/`mute_all_power_levels`/`is_participant`/`mint_pseudonym_claims` seams). The headline judgment call is the marquee publisher-client measurement (DQ `3004b6625b83-001` + the PRD fallback + the gate-1 ratify), which is the H3 risk the whole phase front-loads.
- **Cargo budget:** 8/10 — no daemon cargo; the time cost is the bridge cold build + the heavy live `-e2e` stack (gate-4, user picks local-vs-dispatch). The async conversion + the (DQ-gated) dev-dep are the only Cargo.lock-resolution surfaces, both isolated + `-linux`-gated.
- **Test coverage:** 8/10 — every §Success-Criteria row (134–149) maps to exactly one live test fn (§16a) or a cited anchor / the D2 pilot; the two clean-posture negative invariants + the recording-lands chain-hash + the participant-floor + the anonymous-identity are all observable-state assertions; the marquee carries the documented PRD fallback; the real acceptance signal is the D2 pilot (D2), not green tests alone.
