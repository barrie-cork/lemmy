# Brief: m3-core-e2e-pilot Task 4 — MARQUEE `emergency_mute.rs` live cross-instance <500ms publisher-client mute (criterion 141)

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-task4-emergency-mute-marquee-pubclient-500ms — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-4.md`

You are the **impl-task** subagent (Sonnet 4.6). This is the **MARQUEE** of the whole M3-core phase: turn `mute_all_drops_all_publishers_cross_instance_under_500ms` from `todo!()` into the live integration test, measured **at the PUBLISHER CLIENT** (never the server). Bridge-src test, **Linux-validated**. **Runs ALONE** — it adds the `livekit-client` dev-dep which regenerates `Cargo.lock` in the SAME commit; no other Junior task may run concurrently (Cargo.lock contention).

## §2 Scope

**Plan task: `.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` §13 Task 4** (read it in full — ACTION/IMPLEMENT/MIRROR/GOTCHA/VALIDATE) + §16a Story 2 (the marquee, criterion 141).

**Measurement contract is LAW (DQ `3004b6625b83-001`, answered_by user 2026-06-20, gate-1 ratify) — option-B:**
> Use the livekit Rust **client** dev-dep for an automated **IN-INSTANCE** publisher-client `<500ms` measurement (the PRD line-198 guarantee, measured at the CLIENT not the server) AND route the TRUE **cross-instance** `<500ms` to the **D2 pilot** (Element Call browser clients), documented via this DQ per the scaffold's pre-specified PRD fallback. Faithful publisher-client signal without over-claiming an automated cross-instance bar Matrix federation latency may not deliver.

So: the automated test proves the in-instance publisher-client `<500ms`; the cross-instance `<500ms` is proven at the D2 pilot (Task 7 runbook) — DOCUMENTED via this DQ, never silently dropped or claimed-as-automated.

**Produces (1 file modified + Cargo.toml/Cargo.lock dev-dep, ONE commit):**

- **`services/bridge/tests/emergency_mute.rs`** — turn `mute_all_drops_all_publishers_cross_instance_under_500ms` (the single `todo!()` in the file) into a LIVE body (keep `#[tokio::test] #[ignore = "requires docker-compose stack"]`):
  - Provision a federated town-hall stage room across the TWO bridge instances (chair + N publishers, ≥1 publisher homed on instance-B so the cross-instance path is exercised).
  - Record a per-publisher publish-active baseline **at the publisher client** (via the `livekit-client` dev-dep) — NOT at the server.
  - Fire mute-all: the cross-instance `mute_all_power_levels` Matrix power-level PUT + the local `Stage::mute_all` LiveKit `RevokePublish` sweep.
  - Assert EVERY non-chair publisher's publish right is revoked **at the publisher client** within 500ms (in-instance = the automated guarantee; cross-instance per the DQ-resolved mechanism + the PRD fallback to D2).
  - Assert the cross-instance zero-holder NEGATIVE invariant: NO listed publisher on EITHER instance retains publish (mirror the `stage.rs:727-779` set-equality zero-holder idiom).
  - Query `governance_log` and assert a `room_mute_all` row with `actor_pseudonym` == the chair PSEUDONYM AND `payload.federated == true`.
- **`services/bridge/Cargo.toml`** + **`services/bridge/Cargo.lock`** — add the `livekit-client` (or equivalent livekit Rust client) **dev-dep** (`[dev-dependencies]`), in the SAME commit. The Cargo.lock regeneration is expected (R14-class).

**Do NOT:**
- Measure mute latency at the SERVER. **This is the single most important constraint** — a server-side measurement is a FALSE GREEN (R-PUBCLIENT — CATCH-FIRE, §4).
- Claim the cross-instance `<500ms` as automatically proven. The automated bar is IN-INSTANCE publisher-client; cross-instance routes to D2 (per the DQ).
- Touch any file other than `emergency_mute.rs` + `Cargo.toml` + `Cargo.lock`.
- Touch the harness composes/smoke (Task 1 — GREEN), any bridge SRC (`stage.rs`/`mute_handler.rs` already landed in emergency-mute phase), any `crates/**`, migration, const, registry, or plan.
- Add identity-shaped data — pseudonyms only (ADR-015).
- Add the dev-dep as a regular `[dependencies]` entry — it is a **dev-dep** (test-only).

**Branch:** forks from `phase-m3-core-e2e-pilot` (current tip — confirm at task spawn; ≥ `08d783779`).

## §3 Required reading

- **`.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` §13 Task 4** (~lines 473-495) + §16a Story 2 (the marquee).
- **`services/bridge/tests/emergency_mute.rs`** — the scaffold (`:14-48`); the single `todo!()` is the unique Edit anchor — anchor on the enclosing fn signature line first, then the `todo!()` line (`grep -c 'todo!' services/bridge/tests/emergency_mute.rs` should be 1; confirm before editing).
- **`services/bridge/src/stage.rs:339-366`** (`mute_all` + the cr-11 chair-exclusion) + **`:727-779`** (the set-equality zero-holder idiom to MIRROR for the negative invariant).
- **`services/bridge/src/mute_handler.rs:64-98`** (the cross-instance `mute_all_power_levels` Matrix power-level PUT).
- **`services/bridge/tests/recording.rs`** + **`tests/room_provisioning.rs`** — the `AsyncPgConnection::establish` + `governance_log` query idiom + the federated-room provisioning setup to MIRROR.
- **The Cohort-A sibling briefs** `.claude/PRPs/briefs/m3-core-e2e-pilot-impl-3.md` + `impl-5.md` — same DQ shape + §4 discipline. Mirror the two-DQ block + LESSON-trailer + no-stray-frag rules.
- **DQ `3004b6625b83-001`** (resolved option-B, above) — the measurement contract. Re-read its `answer` field in `.claude/decision-queue.json` before implementing.
- **Lessons (mandatory — §2.4 file-class injection: bridge test target + Cargo.lock dev-dep):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo via cargo-linux.sh; you run NO cargo.
  - `feedback_authz_state_machine_test_asserts_negative.md` — the zero-holder is a NEGATIVE invariant (FAILS if a publisher retains publish).
  - `feedback_build_what_tests_exercise.md` + `pattern_test_against_reality_not_syntax.md` — measure OBSERVABLE publisher-client state (real publish-right revocation timing), not a server-side proxy.
  - `feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate the single `todo!()` anchor on the enclosing fn sig.
  - `feedback_cheap_model_arm_drops_adr_constraints.md` — the ADR-015 chair-pseudonym + `federated:true` assertions are LOAD-BEARING (§2.4a), not decorative; a model under token pressure may drop them — do NOT.
  - `feedback_lemmy_error_no_std_error.md` Case A — mirror the file's existing outer-Result shape; do NOT introduce a divergent error shape.
  - `feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` for the `governance_log` query.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP.

## §4 Constraints

- **R-PUBCLIENT (CATCH-FIRE — the load-bearing constraint of this entire task):** the `<500ms` MUST be measured at the **publisher client** (via the `livekit-client` dev-dep), NEVER at the server. A server-side measurement is a FALSE GREEN against PRD line-141. **Why it can't be deferred or proxied:** the PRD criterion is publisher-perceived mute latency; the server may register a `RevokePublish` instantly while the publisher's client still has live publish rights for hundreds of ms. Measuring server-side proves nothing the criterion asks for. **DoD:** the test imports the livekit client crate and reads the publish-active/revoked state from a CLIENT handle, not from a server/handler return value. If you cannot wire a publisher-client handle, STOP and raise a `kind: "blocker"` DQ — do NOT fall back to a server-side measurement.
- **Measurement-contract fidelity (DQ `3004b6625b83-001`):** automated assertion is IN-INSTANCE publisher-client `<500ms`. The cross-instance `<500ms` is asserted as the zero-holder negative invariant (no publisher on EITHER instance retains publish — correctness) but the `<500ms` *timing* across instances is routed to the D2 pilot, documented via the DQ. Do NOT add an automated cross-instance timing assertion that may flake on Matrix federation latency; do NOT silently drop the cross-instance correctness check.
- **ADR-015 + ADR-016 (pseudonymity + federation metadata — load-bearing):** the `governance_log` `room_mute_all` row MUST carry `actor_pseudonym` == the chair PSEUDONYM (never real identity) AND `payload.federated == true`. **DoD:** `rg -i 'person_id|username|@[a-z].*:' services/bridge/tests/emergency_mute.rs` returns nothing identity-shaped; the test asserts both the pseudonym field AND `federated == true`.
- **R14 (Cargo.lock regenerates in the SAME commit):** adding the dev-dep regenerates `services/bridge/Cargo.lock`. Commit `emergency_mute.rs` + `Cargo.toml` + `Cargo.lock` together in ONE commit. Do NOT split. This is why the task runs SOLO.
- **R-ANCHOR (pre-locate):** the single `todo!()` is the unique anchor — anchor on the enclosing fn signature line first, then the `todo!()` line.
- **Keep `#[ignore]`:** the test stays `#[tokio::test] #[ignore = "requires docker-compose stack"]`.
- **NO daemon cargo.** Write TWO DQs (`bash scripts/brehon/dq-v3-new-entry.sh` for ids, `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending`):
  1. `validate-pending-laptop-linux` (worker COMPILE gate — includes the new dev-dep resolution):
     ```
     kind: "validate-pending-laptop-linux"
     branch: "phase-m3-core-e2e-pilot"
     phase_task: 4
     commands: [
       "scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
       "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings",
       "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test emergency_mute --no-run"
     ]
     result: null
     log_slice: null
     failed_commands: null
     ```
  2. `validate-pending-laptop-e2e` (LIVE run — gate-4=LOCAL; the HEADLINE e2e gate):
     ```
     kind: "validate-pending-laptop-e2e"
     branch: "phase-m3-core-e2e-pilot"
     phase_task: 4
     commands: ["scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test emergency_mute -- --ignored"]
     e2e_filter: null
     result: null
     log_slice: null
     failed_commands: null
     ```
  Commit + push BOTH on the worker branch, then **STOP**.
- **ONE commit** — `feat(rtc): emergency_mute.rs marquee — in-instance publisher-client <500ms mute-all + cross-instance zero-holder (task 4)`. `LESSON:` trailer if durable.
- **No stray temp artifacts:** delete any DQ fragment file after appending.
- **DQ mid-task discipline:** any blocker (esp. cannot-wire-publisher-client) → `kind: "blocker"` DQ, commit + push, stop. Do NOT guess; do NOT fall back to server-side.
- **Handover:** inline (no handover file).

## §3a Handover from prior tasks

- Task 1 — e2e harness GREEN: two federated bridge instances on DISTINCT domains (`matrix-b.localhost` — BUG-15) + LiveKit + MinIO; reach-smoke OK. Your cross-instance path needs the 2nd instance — it's up.
- Task 2 (`5abf83328`) — live LiveSink (not exercised here).
- Tasks 3/5/6 — landed/landing test files on the phase branch (disjoint; do not touch).
- `src/stage.rs` `mute_all` + cr-11 chair-exclusion + `src/mute_handler.rs` cross-instance power-level PUT — both landed in the emergency-mute phase; you EXERCISE them, do not modify them.

## §5 HANDOVER (worker fills at task end — return inline)

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-task4
  filesModified: [services/bridge/tests/emergency_mute.rs, services/bridge/Cargo.toml, services/bridge/Cargo.lock]
  keyDecisions:
    - "in-instance publisher-client <500ms measured at the CLIENT via livekit-client dev-dep (R-PUBCLIENT, criterion 141, DQ 3004b6625b83-001 option-B)"
    - "cross-instance: zero-holder negative invariant asserted (correctness); <500ms timing routed to D2 pilot per PRD fallback (documented via DQ, not automated)"
    - "governance_log room_mute_all row: actor_pseudonym == chair pseudonym + payload.federated == true (ADR-015/016)"
    - "livekit-client added as dev-dep; Cargo.lock regenerated in same commit (R14); kept #[ignore]"
  validate_dq_linux: <id>
  validate_dq_e2e: <id>
  notes: "<confirm <500ms measured at publisher CLIENT not server (R-PUBCLIENT); confirm dev-dep in [dev-dependencies] not [dependencies]; confirm Cargo.lock committed in same commit; confirm chair-pseudonym + federated:true asserted; confirm zero-holder set-equality mirrors stage.rs:727-779; confirm single todo!() anchor; confirm pseudonym-only; confirm no stray dq-frag committed>"
```
