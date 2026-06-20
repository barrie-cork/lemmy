# Brief: m3-core-e2e-pilot Task 6 — `room_provisioning.rs` ADD rtc-disabled clean-posture + anonymous-identity (criteria 146/142)

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-task6-room-provisioning-rtc-disabled-anon-identity — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-6.md`

You are the **impl-task** subagent (Sonnet 4.6). ADD two NEW live integration test fns to `services/bridge/tests/room_provisioning.rs` — these are NOT `todo!()` scaffold conversions, they are net-new fns appended after the existing ones. Bridge-src tests, **Linux-validated**. Runs ALONE (cross-lane cap = 2; the marquee Task 4 runs solo before/after you due to Cargo.lock contention — you do not overlap it).

## §2 Scope

**Plan task: `.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` §13 Task 6** (read it — full ACTION/IMPLEMENT/MIRROR/GOTCHA/VALIDATE) + §16a Story 6 (criteria 146/142).

**Produces (1 file modified, ONE commit):**

- **`services/bridge/tests/room_provisioning.rs`** — ADD two NEW `#[tokio::test] #[ignore = "requires docker-compose stack"]` fns, appended AFTER `messaging_disabled_prevents_provisioning` (`:64`):
  1. **`rtc_disabled_townhall_clean_posture`** (criterion 146, NEGATIVE invariant): with `rtc_enabled=false`, drive a town-hall event; assert ZERO RTC provisioning (NO LiveKit room created, NO MinIO, NO chair seat) AND the governance flow is unchanged. **Mechanical R7 check:** the test FAILS if the `rtc_enabled` gate is forced on. This bridge test owns the "zero RTC provisioning / no LiveKit/MinIO calls" half; the `crates/server/tests/e2e.rs` governance suite owns the "governance flow passes unchanged with RTC disabled" half (criterion 146 names `cargo test --test e2e` for that half — do NOT duplicate it here).
  2. **`anonymous_townhall_identity_never_reaches_livekit`** (criterion 142): provision an `always_pseudonym` town hall; mint a participant token; connect; assert the LiveKit-SERVER-received identity == the PSEUDONYM (never the Lemmy username / `person_id` / MXID). Cite `livekit_jwt::mint_pseudonym_claims` (`:69`) as the JWT-issue-time unit anchor; this test adds the server-received-identity LIVE check.

**Do NOT:**
- Touch any file other than `services/bridge/tests/room_provisioning.rs`.
- Touch or modify the EXISTING M2 `todo!()` scaffolds in this file (jury / emergency / lifecycle / idempotency / `messaging_disabled_prevents_provisioning`) — they are OUT of Phase-6 scope. ADD-only after `:64`.
- Touch the harness composes/smoke (Task 1 — GREEN), any bridge SRC, any `crates/**`, migration, const, registry, or plan.
- Add a dependency (you are test-only; NO Cargo.toml/Cargo.lock edit — that's Task 4's solo concern).
- Add identity-shaped data — pseudonyms only (ADR-015).

**Branch:** forks from `phase-m3-core-e2e-pilot` (current tip — confirm at task spawn; ≥ `08d783779`).

## §3 Required reading

- **`.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` §13 Task 6** (~lines 522-543) + §16a Story 6 (criteria 146/142).
- **`services/bridge/tests/room_provisioning.rs`** — the WHOLE file, esp. `messaging_disabled_prevents_provisioning` (`:55-64`) — the clean-posture model to MIRROR — and the `AsyncPgConnection::establish` + `governance_log` query idiom. Your two new fns go AFTER `:64`.
- **`services/bridge/src/livekit_jwt.rs:69`** (`mint_pseudonym_claims`) — the JWT-issue-time pseudonym seam your anonymous-identity test exercises (the LiveKit server receives the claim minted here).
- **The Cohort-A sibling briefs** `.claude/PRPs/briefs/m3-core-e2e-pilot-impl-3.md` + `impl-5.md` — same DQ shape + same §4 constraint discipline (canonical-schema-first; mirror the two-DQ block + the LESSON-trailer + no-stray-frag rules verbatim).
- **Lessons (mandatory — §2.4 file-class injection: bridge test target):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo via cargo-linux.sh; you run NO cargo.
  - `feedback_authz_state_machine_test_asserts_negative.md` — `rtc_disabled` clean-posture is a NEGATIVE invariant (FAILS if the gate is forced on; a trivial always-skip is invalid per §3.5).
  - `feedback_build_what_tests_exercise.md` + `pattern_test_against_reality_not_syntax.md` — assert OBSERVABLE state (the LiveKit server's received identity; ZERO RTC side-effects), not config/struct shape.
  - `feedback_lemmy_error_no_std_error.md` Case A — if a new test fn returns `Result<(), Box<dyn Error>>`, mirror the file's existing outer-Result shape (Case A: flip to the sibling's `anyhow::Result<()>` / `LemmyResult<()>` shape — read the existing fns' signature FIRST, mirror verbatim; do NOT introduce a divergent error shape).
  - `feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` + `DbPool::Conn` for the `governance_log` query.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP; the laptop runs all cargo.

## §4 Constraints

- **R7 (clean-posture NEGATIVE invariant — load-bearing, CATCH-FIRE on trivial-skip):** `rtc_disabled_townhall_clean_posture` MUST assert NO LiveKit room + NO MinIO + NO chair seat, and MUST FAIL if the `rtc_enabled` flag-gate is forced on. NOT a trivial always-skip. **Why:** `rtc_enabled=false` is the clean default posture; RTC provisioning firing under the false flag is silent resource/identity exposure — the gate must be proven load-bearing by a test that breaks when the gate is deleted.
- **ADR-015 (pseudonymity — load-bearing, CATCH-FIRE on bypass):** `anonymous_townhall_identity_never_reaches_livekit` MUST assert the LiveKit server NEVER receives the real identity — the server-received identity == the pseudonym minted by `mint_pseudonym_claims`. The real Lemmy username / `person_id` / MXID never crosses to LiveKit. **Why:** ADR-015 mandates pseudonymisation at JWT-issue time; an anonymous town hall that leaks the real identity to the media server is an ADR-015 violation. **DoD:** `grep mint_pseudonym_claims services/bridge/tests/room_provisioning.rs` returns a callsite in the new fn; `rg -i 'person_id|username|@[a-z].*:' services/bridge/tests/room_provisioning.rs` returns nothing identity-shaped in your two new fns.
- **ADD-only anchor discipline:** place the two new fns AFTER `messaging_disabled_prevents_provisioning` (`:64`). No `todo!()` collision (these are new fns, not scaffold conversions). Confirm with `grep -n 'fn messaging_disabled_prevents_provisioning' services/bridge/tests/room_provisioning.rs` to locate the insertion point before editing.
- **Keep `#[ignore]`:** both new fns are `#[tokio::test] #[ignore = "requires docker-compose stack"]`.
- **NO daemon cargo.** Write TWO DQs (`bash scripts/brehon/dq-v3-new-entry.sh` for ids, `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending`):
  1. `validate-pending-laptop-linux` (worker COMPILE gate):
     ```
     kind: "validate-pending-laptop-linux"
     branch: "phase-m3-core-e2e-pilot"
     phase_task: 6
     commands: [
       "scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
       "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings",
       "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test room_provisioning --no-run"
     ]
     result: null
     log_slice: null
     failed_commands: null
     ```
  2. `validate-pending-laptop-e2e` (LIVE run — gate-4=LOCAL):
     ```
     kind: "validate-pending-laptop-e2e"
     branch: "phase-m3-core-e2e-pilot"
     phase_task: 6
     commands: ["scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test room_provisioning -- --ignored"]
     e2e_filter: null
     result: null
     log_slice: null
     failed_commands: null
     ```
  Commit + push BOTH on the worker branch, then **STOP**.
- **ONE commit** — `feat(rtc): room_provisioning.rs — rtc-disabled clean-posture + anonymous-identity live e2e (task 6)`. `LESSON:` trailer if durable.
- **No stray temp artifacts:** delete any DQ fragment file after appending (do NOT commit it).
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop. Do NOT guess.
- **Handover:** inline (no handover file).

## §3a Handover from prior tasks

- Task 1 — e2e harness GREEN (MinIO + LiveKit + 2nd federated instance; reach-smoke OK). Your `-e2e` live run executes against this; the anonymous-identity test connects to this LiveKit.
- Task 2 (`5abf83328`) — live LiveSink. Not directly exercised by Task 6 (you assert ZERO provisioning / identity-at-server).
- Tasks 3 (`stage_mode.rs`) + 5 (`recording.rs`) landed on the phase branch — disjoint files; do not touch.
- Task 4 (marquee `emergency_mute.rs`) runs SOLO before/after you (Cargo.lock dev-dep contention) — you never overlap it.

## §5 HANDOVER (worker fills at task end — return inline)

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-task6
  filesModified: [services/bridge/tests/room_provisioning.rs]
  keyDecisions:
    - "rtc_disabled_townhall_clean_posture: ZERO RTC provisioning (no LiveKit room/MinIO/chair seat) + FAIL-if-gate-forced-on (R7 negative invariant, criterion 146)"
    - "anonymous_townhall_identity_never_reaches_livekit: LiveKit-server-received identity == pseudonym from mint_pseudonym_claims, never real identity (ADR-015, criterion 142)"
    - "both NEW fns appended after messaging_disabled_prevents_provisioning (:64); existing M2 scaffolds untouched"
    - "both kept #[ignore] (need live stack)"
  validate_dq_linux: <id>
  validate_dq_e2e: <id>
  notes: "<confirm insertion point after :64; confirm existing scaffolds untouched; confirm R7 fails-if-gate-forced-on; confirm anonymous test asserts SERVER-received identity is pseudonym; confirm pseudonym-only; confirm only tests/room_provisioning.rs edited; confirm no Cargo.toml/lock touched; confirm no stray dq-frag committed>"
```
