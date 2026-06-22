# Plan: m3-core-e2e-pilot-e2e-fixes — make the 3 live-gate e2e defects pass

## 1. Summary

The first-ever live in-stack run of the Tuwunel-based Phase-6 acceptance harness
(advisor e2e gate 2026-06-21, phase tip `4df0ed991`) validated the cycle-4 fixes
(Tuwunel swap, MinIO bucket, LiveKit `--keys`) and exposed **three distinct
remaining defects**, one per failing `services/bridge` integration test:

1. `recording_lands_with_hash_on_chain` — the on-chain `room_recording_uploaded`
   POST hits `http://localhost:3000` (Brehon governance) but **nothing listens on
   host :3000** in the e2e stack → connection refused.
2. `rtc_disabled_townhall_clean_posture` (case 77007) — the provisioner seats a
   chair whenever LiveKit **creds are configured**, not on any per-event flag. The
   wire contract has **no `rtc_enabled` field**, so the test cannot express
   "rtc disabled" → R7 negative invariant (`chair_id IS NULL`) fails.
3. `mute_all_drops_all_publishers_cross_instance_under_500ms` — each never-connected
   publisher costs a ~6 s psrpc `unavailable` timeout **inside** the timed T0→T1
   window → `6007ms > 500ms`.

Headline acceptance condition: all three bridge integration tests pass against the
live e2e stack (`recording`, `room_provisioning`, `emergency_mute` targets green),
with the `rtc_enabled` wire-contract field consumed at **both** ends (governance
emitter + bridge gate), and the <500 ms automated perf proof kept meaningful.

This is the deliberate **replanned** path the user authorised after the cycle-3
reactive-auto-fix catch-fire. No reactive compose/code guesses — each option below
was picked on probe evidence captured during planning (see §7 / §19).

## 2. Source

- `.claude/PRPs/briefs/m3-core-e2e-pilot-planning-e2e-fixes.md` @ `c387d8fde` — this plan's brief.
- `.claude/PRPs/handovers/m3-core-e2e-pilot-e2e-run-2026-06-21c.md` @ `c387d8fde` — the live-gate ground truth (per-defect detail + options analysis).
- Phase branch `phase-m3-core-e2e-pilot` @ `d69d41c1a` — the branch carrying the implemented bridge integration tests + bridge src (impl tasks fork from here, NOT `governance-v0`).
- `.claude/lessons/feedback_entry_kind_runtime_allowlist_check.md` — wire-contract: a field that compiles but isn't consumed at the far end is a silent break (binds defect 2 task structure — both ends consume `rtc_enabled`).
- `.claude/lessons/feedback_planner_enumerate_struct_callsites_for_addfield.md` — enumerate every constructor of a struct gaining a field (binds §11 callsite enumeration for defect 2).
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — workers write the validate DQ, commit, push, STOP; the advisor is the cargo/e2e runner (binds every task DoD).
- `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` — `services/bridge` cargo runs on Linux (the daemon is Linux; the laptop uses `scripts/brehon/cargo-linux.sh`).
- ADR-016 (cross-app governance backplane) @ `c387d8fde`, `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md:273` — M2 governance→bridge is the **first reference integration** of the backplane contract; adding `rtc_enabled` to its wire shape is an **additive field to the reference integration**, not a new/contradicting ADR (see §19).
- PRD intent recorded in DQ `3004b6625b83-001` (option-B, user gate-1 ratify 2026-06-20) — the <500 ms is the IN-INSTANCE AUTOMATED publisher-client proof; cross-instance timing routes to the D2 pilot (binds defect 3's "keep the gate meaningful").

## 3. Problem statement

Three independent failures, each tied to a §13 task:

- **P1 (→ Task 1):** the recording test's R-CHAIN assertion POSTs to a Brehon
  governance endpoint that does not exist in the base e2e stack. Probe (planning,
  on daemon): `curl -m3 http://localhost:3000/api/v4/governance/room-event` →
  connection refused; nothing binds host :3000.
- **P2 (→ Task 3):** `room_provisioner.rs:252` gates the chair-seat block on
  `(livekit_api_key, livekit_api_secret)` both `Some` — a **stack-wide** condition.
  The bridge-local `CaseTransitionEvent` (`room_provisioner.rs:20-37`) and its
  governance-side mirror (`crates/api/api_common/src/governance.rs:889`) carry **no
  `rtc_enabled` field**, so a town-hall event cannot say "rtc disabled". In the
  e2e stack creds ARE present (cycle-4 `--keys`) → chair seats → R7 fails. The
  test's existing env-var guard (`room_provisioning.rs:88-97`) checks the **test
  process** env, but the gate reads the **bridge process** config — wrong process.
- **P3 (→ Task 2):** `emergency_mute.rs:191` calls `update_participant().await` per
  publisher inside the timed window opened at `:188` (`t0`). Two never-connected
  publishers × ~3 s psrpc `unavailable` timeout = ~6 s. The #764 fix made the error
  *classification* correct (timeout ⇒ zero-holder-by-absence) but did nothing about
  the *6 s to reach it*. `:228` elapsed → `:232` `< 500ms` assert fails.

## 4. Solution statement

Three file-disjoint, independently-validatable fixes (one cohort, all `[P]`):

```
Defect 1 (env/infra)   docker-compose.e2e.yml  ──►  add room-event-stub container
                       (caddy respond :3000, 200)     publishes host :3000, 200s any POST
                                                       → recording R-CHAIN status-success met

Defect 2 (wire contract) governance CaseTransitionEvent + emitter   ──►  rtc_enabled: Option<bool>
                         bridge CaseTransitionEvent + :252 gate            (BOTH ends consume it)
                         room_provisioning test sends rtc_enabled:false  → chair NOT seated → R7 ok

Defect 3 (timing)      emergency_mute.rs revoke loop  ──►  wrap each update_participant in
                                                            tokio::time::timeout(200ms);
                                                            Elapsed ⇒ zero-holder-by-absence
                                                          → 2 × 200ms = 400ms < 500ms
```

From §4 alone the reader can predict §11: defect 1 touches one compose file; defect
3 touches one test file; defect 2 touches the governance struct, its sole emitter,
the bridge mirror struct + gate, and the bridge test.

## 5. Metadata

- **Phase:** `m3-core-e2e-pilot` (M3-core Phase 6/6 — FINAL; this is the cycle-5 fix layer)
- **Branch:** `phase-m3-core-e2e-pilot` (ALREADY cut at `d69d41c1a` — no new branch; impl tasks fork from this tip)
- **Target impl-task model:** `sonnet-4-6` (default)
- **Estimated tasks:** 5 (Task 0 pre-flight + 3 impl + retro)
- **Estimated cargo budget:** ~3 GB peak (bridge crate compile; lemmy workspace check for defect 2 runs in the `rust:1.95` Docker mirror) — Gate-4 is LOCAL on the daemon, not Shape G
- **Forbidden-window applicability:** standard, BUT cargo/e2e runs are advisor-driven (workers write-then-STOP), so the forbidden window binds the advisor's e2e RUN, not worker dispatch
- **Complexity score:** `3/10` — see breakdown below

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 3 impl tasks (Tasks 1–3); none beyond the 5th |
| Migrations touched | +2 each | 0 | `seed_rtc_enabled_config` already shipped (m3-core-infra); this plan touches none |
| Crates touched | +1 each | 3 | `lemmy_api_common`, `lemmy_api_utils`, `brehon-bridge` (defect 2); defects 1 & 3 touch a compose file + a bridge test |
| `crates/server/tests/e2e/*.rs` edits | +3 each | 0 | the giant `e2e.rs` is NOT touched; bridge tests are small standalone files |
| New ADR-affecting decisions | +2 each | 0 | `rtc_enabled` is additive within ADR-016's M2 reference integration — supersedes no ADR |
| Cargo budget peak above 6 GB | +1 per GB | 0 | bridge compile ~3 GB; under 6 GB |
| **Total** | — | **3** | Threshold for split-DQ: `>8` (Sonnet). **3 ≤ 8 → no split-DQ.** |

### 5.2 Per-task complexity ceiling (Sonnet target: ≤4 files / ≤2 crates)

- **Task 1** (defect 1): 1 file, 0 crates. ✓
- **Task 2** (defect 3): 1 file (`emergency_mute.rs`), 1 crate (bridge test). ✓
- **Task 3** (defect 2): 4 files across 3 crate-prefixes (`crates/api/api_common`,
  `crates/api/api_utils`, `services/bridge`). This **exceeds the ≤2-crate norm** —
  intentionally, and per the brief's explicit framing ("enumerate every edit site
  for the wire-contract field … keep the change minimal: one field + one gate +
  test setup"). The four edits are an **atomic wire-contract change**: the governance
  struct, its sole constructor (`bridge_notify.rs`), the bridge mirror struct, and
  the test must land together or the workspace fails to compile / the field is a
  silent break (`feedback_entry_kind_runtime_allowlist_check.md`). Splitting would
  create a non-compiling or contract-inconsistent intermediate. This is the
  documented atomic-contract exception, **not** a split-candidate. The single worker
  pins the **verbatim** field declaration from §10.2 on BOTH structs, guaranteeing
  wire consistency (a risk two parallel workers would invite).

## 6. Relationship to other m3-core sub-phases

This is the final cycle-5 fix layer of `m3-core-e2e-pilot` (Phase 6/6 of M3-core).
It depends on the cycle-4 fixes already on tip `4df0ed991` (Tuwunel swap, MinIO
bucket, LiveKit `--keys`) — VALIDATED, do not revisit. After all three tests pass:
Task 7 (D2 pilot runbook, NON-impl, per the handover) → retro → `/brehon-verify
m3-core-e2e-pilot` → bm-pr → CR triage → merge → phase-transition. `rtc_enabled` as
a governance config was introduced by m3-core-infra (`seed_rtc_enabled_config`
migration); this plan makes the bridge **honour** it per-event.

## 7. Preflight guardrails inherited from prior phases

- **R-NOREACTIVE:** no reactive compose/code guesses — each option chosen on probe
  evidence (cycle-3 catch-fire is in effect for the OLD reactive loop). Defect 1's
  :3000-refused, defect 2's creds-gate, defect 3's 6 s-timeout were all confirmed by
  reading the live test source + the gate code on the phase branch (§3 probes).
- **R-WIRECONSISTENCY:** the `rtc_enabled` field MUST be declared **identically** on
  both `CaseTransitionEvent` structs — `#[serde(default)] pub rtc_enabled:
  Option<bool>` — and consumed at both ends (governance emitter populates from
  config; bridge gate reads it). A field added to one end only is a silent contract
  break (`feedback_entry_kind_runtime_allowlist_check.md`).
- **R-WRITE-THEN-STOP:** workers edit + write the validate DQ + commit + push +
  **STOP**. Workers do NOT run cargo/e2e on the daemon for the main crates; the
  advisor is the runner (`feedback_validate_pending_laptop_write_then_stop.md`).
- **R-BRIDGEREBUILD:** defect 2 changes `room_provisioner.rs` (bridge **src**) →
  the advisor MUST rebuild the bridge container image before the e2e run
  (`docker compose … up -d --build bridge`); a stale image silently runs the old gate.
- **R-PORT3000:** host :3000 must be free before stack-up (probe: free at planning).
  The known `bridge-b :8081` caddy conflict is unrelated (D2-deferred, non-blocking).
- **R-TIMEOUT-MATH:** defect 3's per-call timeout `T` must satisfy `N_publishers × T
  < 500ms`. With `N=2`, `T=200ms` → 400 ms worst case. Do not raise `T` without
  re-checking the product.
- **R-BRIDGELINUX:** `services/bridge` and the lemmy workspace compile on Linux; the
  daemon is Linux (native cargo per the handover re-run procedure). The
  laptop-advisor path uses `scripts/brehon/cargo-linux.sh` (Docker `rust:1.95`).

## 8. Flow design

**Defect 1 — recording on-chain POST (before → after):**

```
BEFORE:  recording test ──POST :3000──► (nothing) ──► connection refused ──► test FAIL
         bridge drain  ──POST host.docker.internal:3000──► (nothing)
AFTER:   recording test ──POST :3000──► room-event-stub (caddy respond, 200) ──► is_success() ──► PASS
         bridge drain  ──POST host.docker.internal:3000──► room-event-stub (200)
```

**Defect 2 — rtc_enabled wire contract (before → after):**

```
BEFORE:  governance emitter ─CaseTransitionEvent{…no rtc flag…}─► bridge
         bridge :252  if (creds Some,Some) { seat chair }   ← stack-wide creds only
         test sends {town_hall, chair_pseudonym}             ← cannot say "rtc off"
         → creds present → chair seated → R7 FAIL

AFTER:   governance emitter reads rtc_enabled config ─CaseTransitionEvent{…, rtc_enabled}─► bridge   [Task 3]
         bridge :252  if event.rtc_enabled == Some(false) { return }   ← per-event short-circuit
                      else (creds Some,Some) { seat chair }
         test sends {town_hall, chair_pseudonym, rtc_enabled:false}     ← expresses "rtc off"
         → short-circuit → no chair → chair_id NULL → R7 PASS
```

**Defect 3 — emergency_mute timed loop (before → after):**

```
BEFORE:  t0; for pub in [a,b] { update_participant(pub).await }   ← each ~3s psrpc timeout
         elapsed = t0.elapsed()  ≈ 6007ms ;  assert < 500ms       ← FAIL
AFTER:   t0; for pub in [a,b] {
           match timeout(200ms, update_participant(pub)).await {
             Ok(inner) => …existing Ok/Err arms…
             Err(Elapsed) => revoke_results.push((pub,true))       ← zero-holder by absence
           } }
         elapsed ≈ 400ms ;  assert < 500ms                        ← PASS
```

## 9. Mandatory reading

Each impl-task subagent reads, before its first edit (phase-branch versions):

- **Defect 1 (Task 1):**
  - `services/bridge/docker-compose.e2e.yml` — the `minio` / `minio-init` sidecar
    pattern to mirror; the `bridge` env block (`BREHON_ROOM_EVENT_URL:
    http://host.docker.internal:3000/...`) showing the bridge already targets host :3000.
  - `services/bridge/tests/recording.rs:88-133` — the R-CHAIN assertion is ONLY
    `chain_resp.status().is_success()` (line 118-123); no body read, no persistence
    check. Confirms a 200-only stub is e2e-honest for THIS test (the principle the
    brief named "build what tests exercise" — that lesson slug does not exist in the
    corpus; see §19).
- **Defect 2 (Task 3):**
  - `crates/api/api_common/src/governance.rs:884-908` — `CaseTransitionEvent` struct (gains the field).
  - `crates/api/api_utils/src/bridge_notify.rs:1-12,108-145` — imports (`GovernanceMessagingConfig` already in scope), the `messaging_enabled` read pattern, and the sole `CaseTransitionEvent { … }` constructor at `:136`.
  - `crates/api/api/src/governance/bridge_read.rs:44-49` — the verbatim `rtc_enabled` config read pattern to mirror in the emitter.
  - `services/bridge/src/room_provisioner.rs:20-37` (bridge mirror struct) + `:252-265` (the creds gate to amend).
  - `services/bridge/tests/room_provisioning.rs:72-190` — the rtc_disabled test (the `:88-97` env-var guard to remove + the `:120-138` JSON payload to extend + the `:172-179` R7 assertion).
  - `feedback_entry_kind_runtime_allowlist_check.md`, `feedback_planner_enumerate_struct_callsites_for_addfield.md`.
- **Defect 3 (Task 2):**
  - `services/bridge/tests/emergency_mute.rs:12-20` (criterion 141 doc) + `:185-235` (the timed revoke loop, the existing `unavailable`/`not found` accept arm, the <500 ms assert).

## 10. Patterns to mirror

### 10.1 Compose sidecar (defect 1 stub)

**Mirror:** `services/bridge/docker-compose.e2e.yml:96-123` (`minio` + `minio-init`
service shape — pinned image, `networks: [bridge-net]`, the e2e-overlay comment style).

```yaml
  # ─── Brehon governance :3000 stub for recording on-chain POST (defect 1) ────
  # recording_lands_with_hash_on_chain (and the bridge's drain_emits) POST
  # room_recording_uploaded to BREHON_ROOM_EVENT_URL (host :3000). The base e2e
  # stack runs no Brehon binary; this stub returns HTTP 200 to ANY request so the
  # test's R-CHAIN status-success assertion (recording.rs:118-123) is satisfied.
  # It does NOT persist a governance_log row — chain persistence is the governance
  # suite's job (crates/server/tests/e2e/governance.rs), NOT this bridge integration
  # test (which only asserts the bridge POSTs a well-formed 2xx room-event).
  room-event-stub:
    image: caddy:2.8-alpine            # Apache-2.0; pinned. `caddy respond` = fixed-status responder.
    command: ["caddy", "respond", "--listen", ":3000", "--status", "200", "--body", "room-event-stub-ok"]
    ports:
      - "3000:3000"
    networks:
      - bridge-net
```

**GOTCHA:** no `profiles:` key → the stub is up whenever the bridge is up (the
bridge's `BREHON_ROOM_EVENT_URL` targets :3000 unconditionally). The recording run
uses `--profile rtc` (a superset), so a no-profile service is included. The impl
agent confirms `caddy respond` accepts these flags on the pinned tag; if the pinned
`caddy:2.8-alpine` lacks `respond`, fall back to `traefik/whoami` (returns 200 to
any method) — note the substitution in the commit body (no silent image swap).

### 10.2 `rtc_enabled` wire-contract field (defect 2 — VERBATIM on BOTH structs)

**Mirror (governance):** `crates/api/api_common/src/governance.rs:889-908`
(`CaseTransitionEvent`, the `#[serde(default)] pub chair_pseudonym: Option<String>`
field shape). **Mirror (bridge):** `services/bridge/src/room_provisioner.rs:20-37`
(same struct name, `#[serde(default)]` on optional fields).

Add this field **identically** to both `CaseTransitionEvent` structs (last field):

```rust
    /// Per-event RTC toggle, mirrored on the wire from the governance `rtc_enabled`
    /// config. `None` (absent) defaults RTC ON for back-compat — events without the
    /// field keep the pre-existing creds-gated behaviour. `Some(false)` disables the
    /// RTC stage seat even when LiveKit creds are configured (criterion 146 / R7).
    #[serde(default)]
    pub rtc_enabled: Option<bool>,
```

### 10.3 Emitter populates `rtc_enabled` from config (defect 2 — `bridge_notify.rs`)

**Mirror:** `crates/api/api/src/governance/bridge_read.rs:44-49` (the `rtc_enabled`
config read) + `bridge_notify.rs:112-119` (the in-function `messaging_enabled` read,
same `GovernanceMessagingConfig::read_current(pool, …)` shape).

Read the config (after the `messaging_enabled` gate, before building the payload) and
add `rtc_enabled` to the `CaseTransitionEvent { … }` literal at `:136`:

```rust
  // Per-event RTC toggle: mirror the instance `rtc_enabled` governance config onto the
  // wire so the bridge can honour it (defect-2 contract; consumed at room_provisioner.rs).
  let rtc_enabled = GovernanceMessagingConfig::read_current(pool, "instance", "rtc_enabled")
    .await?
    .and_then(|r| r.value_bool);            // Option<bool>: None when the config row is absent
  let payload = BridgeNotifyPayload::CaseTransition(CaseTransitionEvent {
    case_id: case.id.0,
    old_status,
    new_status,
    community_id: case.community_id.map(|c| c.0),
    target_type: case.target_type,
    juror_pseudonyms,
    chair_pseudonym: None,
    rtc_enabled,                            // ← new (defect 2; populated from config)
  });
```

### 10.4 Per-event gate short-circuit (defect 2 — `room_provisioner.rs:252`)

**Mirror:** the existing creds-gate `match` at `:252-265`. Insert the per-event
short-circuit **immediately before** it (so explicit `Some(false)` wins regardless of
creds):

```rust
    // (e) stage mode — gated on (1) the per-event rtc_enabled flag, then (2) RTC config.
    // rtc_enabled defaults ON when absent (back-compat); explicit Some(false) disables
    // the RTC stage seat even when LiveKit creds are present (criterion 146 / R7).
    if event.rtc_enabled == Some(false) {
        tracing::debug!(
            case_id = event.case_id,
            "rtc_enabled=false — skipping stage-mode setup (R7 negative invariant)"
        );
        return;
    }
    let (api_key, api_secret) = match (
        state.config.livekit_api_key.as_deref(),
        state.config.livekit_api_secret.as_deref(),
    ) { /* … unchanged … */ };
```

### 10.5 Test payload + guard fix (defect 2 — `room_provisioning.rs`)

The `:88-97` env-var guard checks the **test process** env (`LIVEKIT_API_KEY/SECRET`),
but the gate reads the **bridge process** config — wrong process. Under the per-event
contract the bridge legitimately HAS creds (full rtc stack) while the EVENT disables
RTC. **Remove** the `:88-97` `assert!` and replace with a one-line comment; **add**
`"rtc_enabled": false` to the `:120-138` JSON payload. The `:172-179` R7 assertion is
unchanged and stays load-bearing (remove the `room_provisioner.rs:252` short-circuit
and, with creds present, `chair_id` becomes non-NULL → test fails).

```rust
    // (the :88-97 LIVEKIT env-var assert is REMOVED — under the rtc_enabled wire
    // contract the gate is per-event, not per-stack-cred; the bridge may hold creds.)
    …
        .json(&serde_json::json!({
            "type_": "case_transition",
            "case_id": case_id,
            "new_status": "town_hall",
            "chair_pseudonym": chair_pseudonym,
            "rtc_enabled": false                // ← defect 2: express "rtc disabled" per-event
        }))
```

### 10.6 Fast-fail revoke (defect 3 — `emergency_mute.rs`)

**Mirror:** the existing `match result { Ok(info) => …, Err(e) => … }` block and its
`revoke_results.push((publisher, true))` zero-holder-by-absence arm.

```rust
    for &publisher in publishers {
        // Fast-fail: a never-connected publisher has no psrpc handler → ~3s timeout.
        // Cap each call so N×cap stays < 500ms (R-TIMEOUT-MATH: 2 × 200ms = 400ms).
        // Elapsed ⇒ zero-holder-by-absence (same semantics as the `unavailable` arm).
        let result = match tokio::time::timeout(
            Duration::from_millis(200),
            lk_client.update_participant(
                &lk_room_name,
                publisher,
                UpdateParticipantOptions { /* … unchanged … */ },
            ),
        ).await {
            Ok(inner) => inner,                       // inner: Result<ParticipantInfo, _> — existing arms handle it
            Err(_elapsed) => {
                // No response within budget = never-connected = zero-holder satisfied by absence.
                revoke_results.push((publisher, true));
                continue;
            }
        };
        match result { /* … existing Ok / Err arms unchanged … */ }
    }
```

`Duration` is already imported (`emergency_mute.rs:23`). `tokio::time::timeout` needs
no new dependency (`tokio` is already a dev-dependency for `#[tokio::test]`).

## 11. Files to change

**`services/bridge/` (defects 1, 3, and bridge half of 2):**
- `services/bridge/docker-compose.e2e.yml` — add the `room-event-stub` sidecar (Task 1).
- `services/bridge/tests/emergency_mute.rs` — wrap `update_participant` in a 200 ms timeout (Task 2).
- `services/bridge/src/room_provisioner.rs` — add `rtc_enabled` to the bridge-local `CaseTransitionEvent` + the `:252` short-circuit (Task 3).
- `services/bridge/tests/room_provisioning.rs` — send `rtc_enabled:false` + remove the obsolete `:88-97` env guard (Task 3).

**`crates/` (governance half of defect 2):**
- `crates/api/api_common/src/governance.rs` — add `rtc_enabled: Option<bool>` to `CaseTransitionEvent` (Task 3).
- `crates/api/api_utils/src/bridge_notify.rs` — read the `rtc_enabled` config + populate the field in the `:136` constructor (Task 3).

### Struct-field add: enumerate all callsites (mandatory)

Per `feedback_planner_enumerate_struct_callsites_for_addfield.md`. `rg
"CaseTransitionEvent \{" crates/ services/` at planning time (phase branch
`d69d41c1a`) returns exactly **one production constructor** of the **governance-side**
struct:

- `crates/api/api_utils/src/bridge_notify.rs:136` — the sole `CaseTransitionEvent { … }`
  literal (in `governance_case_after_transition`). Updated in Task 3 (§10.3). No
  `Default` impl, so the literal MUST gain the field or the crate fails to compile.

(Other `CaseTransitionEvent {` hits are in `.claude/PRPs/briefs/*` and `*.plan.md` —
documentation, not compiled.) The **bridge-side** `CaseTransitionEvent`
(`room_provisioner.rs:20`) is **deserialize-only** — constructed by serde from JSON,
never by a struct literal — so adding a `#[serde(default)]` field needs no constructor
edit there. The struct is **not** `ts-rs`-exported (no `#[cfg_attr(feature = "ts-rs", …)]`),
so no TS-binding regeneration. **Caller crates (compiles-only-after-Task-3):**
`lemmy_api_utils` (the constructor) — same task, so no cross-task `requires:`.

## 12. NOT building in m3-core-e2e-pilot-e2e-fixes

- **A real Brehon governance binary on :3000 for e2e** — deferred; the recording
  test asserts only "bridge POSTs a well-formed 2xx room-event", not "Lemmy
  persisted the chain row" (recording.rs:118-123). A 200-stub is e2e-honest;
  chain-persistence coverage is the governance suite's job
  (`crates/server/tests/e2e/governance.rs`). Reason: defect-1 Option B (stand up a
  governance-capable Lemmy on :3000) is the largest option and out of proportion to
  what the test checks.
- **Reworking the provisioner's RTC logic** — defect 2 is one field + one gate
  short-circuit + test setup, NOT a stage-mode redesign. Reason: brief Constraint (c).
- **Cross-instance <500 ms automated proof** — stays routed to the D2 pilot (real
  Element Call clients) per DQ `3004b6625b83-001`; the base stack proves in-instance
  timing + cross-instance zero-holder correctness only. Reason: defect-3 Option C cost.
- **Defect-3 connectivity pre-check** (Option B) — rejected; the `tokio::time::timeout`
  wrap (Option A) is fewer lines and keeps the perf gate meaningful (a fast-failing
  call still proves the revocation path is sub-budget).

---

## 13. Step-by-step tasks

> Impl tasks fork from `phase-m3-core-e2e-pilot` (`d69d41c1a`), where the implemented
> bridge tests + src live — NOT `governance-v0`. One commit per task. Tasks 1–3 are a
> single `[P]` cohort (file-disjoint per §11). Gate-4 is LOCAL: each impl task ends by
> writing its validate DQ + commit + push + **STOP** (R-WRITE-THEN-STOP); the advisor
> runs cargo/e2e and mutates the DQ.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** confirm the environment is ready and the phase tip is intact before any edit.

**Probes (R5 — enumerate ALL explicitly):**

```bash
# Probe 0 — on the phase branch tip
git fetch origin phase-m3-core-e2e-pilot && git log -1 --oneline origin/phase-m3-core-e2e-pilot
# EXPECT: d69d41c1a (or a later tip if advanced); STOP if it moved unexpectedly

# Probe 1 — Docker daemon up (needed for the advisor's later e2e run)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 2 — host :3000 free (defect 1 stub will bind it)
(curl -sS -m3 -o /dev/null -w "%{http_code}" http://localhost:3000/ 2>/dev/null && echo " — :3000 OCCUPIED, STOP") || echo ":3000 free"

# Probe 3 — the 3 target test fns exist on the phase branch (anchors for the edits)
git grep -c 'recording_lands_with_hash_on_chain' origin/phase-m3-core-e2e-pilot -- services/bridge/tests/recording.rs
git grep -c 'rtc_disabled_townhall_clean_posture'  origin/phase-m3-core-e2e-pilot -- services/bridge/tests/room_provisioning.rs
git grep -c 'under_500ms'                            origin/phase-m3-core-e2e-pilot -- services/bridge/tests/emergency_mute.rs
# EXPECT: each ≥ 1

# Probe 4 — sole governance-side constructor (defect 2 callsite enumeration holds)
git grep -n 'CaseTransitionEvent {' origin/phase-m3-core-e2e-pilot -- crates/
# EXPECT: exactly one non-doc hit — crates/api/api_utils/src/bridge_notify.rs:136

# Probe 5 (negative) — confirm exit-code propagation
false && echo "should not print" || echo "negative probe ok"
```

**EXPECT block:** Probes 0–4 succeed; Probe 5 prints `negative probe ok`. **No commit at Task 0.**

### Task 1 [P]: Defect 1 — :3000 room-event stub sidecar

**ACTION:** add a `room-event-stub` container (caddy respond, 200) to
`docker-compose.e2e.yml` so the recording on-chain POST gets a 2xx.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/docker-compose.e2e.yml   # add room-event-stub service (host :3000, 200s any POST)
requires: []
```

**IMPLEMENT (file 1 of 1):** in `services/bridge/docker-compose.e2e.yml`, add the
`room-event-stub` service under `services:` using the VERBATIM block from §10.1. No
`profiles:` key. Place it adjacent to the `minio` block for readability.

**MIRROR:** `services/bridge/docker-compose.e2e.yml:96-123` (`minio`/`minio-init` shape).

**GOTCHA:** the bridge already targets host :3000 (`BREHON_ROOM_EVENT_URL:
http://host.docker.internal:3000/...`, line 61) — the stub backs BOTH the host-side
test (`localhost:3000`) and the in-container bridge drain (host-gateway :3000). If
`caddy respond` is unavailable on the pinned tag, fall back to `traefik/whoami` and
note the swap in the commit body.

**VALIDATE (write-then-STOP):** write a `validate-pending-laptop-e2e` DQ entry, then
commit + push + STOP. Do NOT run docker/cargo.

```
kind: validate-pending-laptop-e2e
commands: ["cargo test --manifest-path services/bridge/Cargo.toml --test recording -- --ignored"]
branch: phase-m3-core-e2e-pilot
phase_task: 1
context: advisor brings up the FULL e2e stack first
         (cd services/bridge && docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc up -d)
         which now includes room-event-stub on :3000; export PATH + BRIDGE_DB_PATH per the handover re-run procedure.
```

### Task 2 [P]: Defect 3 — fast-fail emergency-mute revoke loop

**ACTION:** wrap each `update_participant` call in `tokio::time::timeout(200ms)`;
treat `Elapsed` as zero-holder-by-absence so mute-all stays < 500 ms.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/tests/emergency_mute.rs   # wrap update_participant in 200ms timeout; Elapsed ⇒ revoked-by-absence
requires: []
```

**IMPLEMENT (file 1 of 1):** in `services/bridge/tests/emergency_mute.rs`, replace the
`let result = lk_client.update_participant(…).await;` at `~:191` with the
`tokio::time::timeout(Duration::from_millis(200), …)` wrap from §10.6, keeping the
existing `match result { Ok/Err }` arms intact.

**MIRROR:** `emergency_mute.rs:185-235` (the timed loop + the `revoke_results.push((publisher, true))` absence arm).

**GOTCHA:** R-TIMEOUT-MATH — `N_publishers × 200ms < 500ms`; with `N=2` → 400 ms. Do
NOT raise the 200 ms without re-checking the product. `Duration` is already imported
(`:23`); `tokio::time::timeout` adds no dependency. Keep the existing `unavailable`/
`not found` Err arm — a connected publisher that responds fast still flows through it.

**VALIDATE (write-then-STOP):** write a `validate-pending-laptop-e2e` DQ entry, commit
+ push + STOP.

```
kind: validate-pending-laptop-e2e
commands: ["cargo test --manifest-path services/bridge/Cargo.toml --test emergency_mute -- --ignored"]
branch: phase-m3-core-e2e-pilot
phase_task: 2
context: advisor runs against the FULL e2e stack (--profile rtc) per the handover re-run procedure.
```

### Task 3 [P]: Defect 2 — `rtc_enabled` wire-contract field + per-event gate

**ACTION:** add `rtc_enabled: Option<bool>` to BOTH `CaseTransitionEvent` structs,
populate it from the governance config in the emitter, short-circuit the bridge gate
on `Some(false)`, and update the test to send `rtc_enabled:false`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api_common/src/governance.rs        # add rtc_enabled: Option<bool> to CaseTransitionEvent
  - crates/api/api_utils/src/bridge_notify.rs       # read rtc_enabled config + populate the :136 constructor
  - services/bridge/src/room_provisioner.rs         # mirror field + :252 per-event short-circuit
  - services/bridge/tests/room_provisioning.rs      # send rtc_enabled:false; remove obsolete :88-97 env guard
requires: []        # both compile independently; coupled only by the on-the-wire field name (pinned in §10.2)
```

**IMPLEMENT (file 1 of 4):** `crates/api/api_common/src/governance.rs` — add the §10.2 field (verbatim) to `CaseTransitionEvent` (`:889-908`).
**IMPLEMENT (file 2 of 4):** `crates/api/api_utils/src/bridge_notify.rs` — add the §10.3 config read + `rtc_enabled` in the `:136` constructor.
**IMPLEMENT (file 3 of 4):** `services/bridge/src/room_provisioner.rs` — add the §10.2 field (verbatim) to the bridge-local `CaseTransitionEvent` (`:20-37`) + the §10.4 short-circuit at `:252`.
**IMPLEMENT (file 4 of 4):** `services/bridge/tests/room_provisioning.rs` — apply §10.5 (remove `:88-97` env guard; add `"rtc_enabled": false` to the `:120-138` payload).

**MIRROR:** §10.2 (field, both structs) / §10.3 (emitter) / §10.4 (gate) / §10.5 (test).

**GOTCHA:** R-WIRECONSISTENCY — the field declaration MUST be byte-identical on both
structs (`#[serde(default)] pub rtc_enabled: Option<bool>`). Default-ON semantics
(`None`/absent ⇒ RTC on) preserve every existing caller (no other emitter sets the
field); only explicit `Some(false)` disables. The bridge mirror is deserialize-only
(no constructor edit). R-BRIDGEREBUILD: the advisor MUST `up -d --build bridge` before
the e2e run (room_provisioner.rs is bridge **src**, baked into the image).

**VALIDATE (write-then-STOP):** write TWO DQ entries (compile proof for the lemmy
workspace half + e2e for the bridge half), commit + push + STOP.

```
kind: validate-pending-laptop-linux
commands: ["./scripts/brehon/cargo-linux.sh check --workspace --features full"]
branch: phase-m3-core-e2e-pilot
phase_task: 3
context: proves governance.rs + bridge_notify.rs compile in the lemmy workspace
         (the bridge e2e run does NOT build the lemmy workspace).
---
kind: validate-pending-laptop-e2e
commands: ["cargo test --manifest-path services/bridge/Cargo.toml --test room_provisioning -- --ignored"]
branch: phase-m3-core-e2e-pilot
phase_task: 3
context: advisor rebuilds the bridge image (up -d --build bridge) then runs against the FULL --profile rtc stack;
         compiles room_provisioner.rs + the test, and proves the rtc_disabled R7 invariant.
```

### Task 4: Retro

**Goal:** author the retro per `feedback_retro_not_report.md` +
`feedback_four_role_retro_signals.md` — one H2 per role (Advisor / Planning / Impl /
BM) with signals + lessons. Capture: the cycle-3→replanned discipline that worked;
the missing `feedback_build_what_tests_exercise.md` lesson reference in the brief (§19);
per-task complexity (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`). Promote
any new lessons to `.claude/lessons/feedback_*.md` in the same commit.

---

## 14. Testing strategy

- **Compile (defect 2 lemmy half):** `./scripts/brehon/cargo-linux.sh check --workspace --features full` (Docker `rust:1.95`; or native `cargo check --workspace --features full` on the Linux daemon).
- **Compile (bridge):** building any bridge test target compiles `room_provisioner.rs` + the tests; `cargo test --manifest-path services/bridge/Cargo.toml --no-run` covers it.
- **e2e execution (advisor, LOCAL, full stack):** per the handover re-run procedure —
  1. free :8080 (`docker stop web-archive-frontend`), `up -d` the stack (`--profile rtc`, now including `room-event-stub`); for defect 2 add `--build bridge`.
  2. `export PATH=$HOME/.cargo/bin:$PATH && export BRIDGE_DB_PATH=/srv/brehon-fork/services/bridge/.e2e-data/bridge-a/bridge-a.db`
  3. per target: `cargo test --manifest-path services/bridge/Cargo.toml --test <recording|room_provisioning|emergency_mute> -- --ignored > /tmp/m3-e2e-<T>.log 2>&1; echo EXIT=$?` — **read the LOG**, not the exit code (cargo returns 101 for lock-fail too).
  4. teardown + `docker start docker-portainer-1 web-archive-frontend`.

## 15. Validation commands (DoD)

> Gate-4 is LOCAL + write-then-STOP: workers write the DQ entries below; the **advisor**
> executes the commands and mutates the DQ (`answered_by: advisor-laptop`). No worker
> runs cargo/e2e on the daemon for these.

### 15.1 Defect 2 — lemmy workspace compile proof (Task 3)

```bash
./scripts/brehon/cargo-linux.sh check --workspace --features full
# EXPECT: exit 0  (governance.rs + bridge_notify.rs compile)
```

### 15.2 e2e targets (advisor, full stack — Tasks 1, 2, 3)

```bash
# after `up -d` (+ `--build bridge` for defect 2):
cargo test --manifest-path services/bridge/Cargo.toml --test recording        -- --ignored   # Task 1
cargo test --manifest-path services/bridge/Cargo.toml --test emergency_mute    -- --ignored   # Task 2
cargo test --manifest-path services/bridge/Cargo.toml --test room_provisioning -- --ignored   # Task 3
# EXPECT (read the log): the 3 previously-failing fns now pass; previously-passing fns still pass:
#   recording: recording_lands_with_hash_on_chain PASS (+ clean_posture, participant_floor_fetch still pass)
#   room_provisioning: rtc_disabled_townhall_clean_posture PASS (+ anonymous_townhall still passes; 5 todo!() stubs D2-deferred)
#   emergency_mute: mute_all_…_under_500ms PASS (elapsed < 500ms)
```

### 15.3 Cross-cutting verification

- [ ] R-WIRECONSISTENCY: `rtc_enabled: Option<bool>` declared identically on both `CaseTransitionEvent` structs; emitter populates it; bridge gate reads it (`grep rtc_enabled` returns BOTH structs + the emitter + the gate).
- [ ] R-BRIDGEREBUILD: the advisor's defect-2 e2e run used `up -d --build bridge`.
- [ ] R-TIMEOUT-MATH: defect-3 per-call timeout × publisher count < 500 ms.
- [ ] No edits to files outside §11.
- [ ] The `:88-97` env-var guard removed from `room_provisioning.rs`; R7 assertion (`:172-179`) retained.

## 16. Acceptance criteria

- [ ] Task 0 probes confirmed
- [ ] Tasks 1–3 committed (one commit each) on `phase-m3-core-e2e-pilot`
- [ ] §15.1 (lemmy workspace check) exit 0
- [ ] §15.2 — the 3 target tests pass; no previously-passing bridge test regressed
- [ ] §15.3 cross-cutting boxes ticked
- [ ] §16a stories all `[done]`
- [ ] No edits outside §11
- [ ] Retro committed (Task 4)
- [ ] PR (later, BM) opens against `governance-v0` with `--repo barrie-cork/lemmy`

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: Recording on-chain POST lands a 2xx

- **Composing tasks:** Task 1 (`[P]`)
- **Checkpoint command:** `cargo test --manifest-path services/bridge/Cargo.toml --test recording -- --ignored` (advisor, full `--profile rtc` stack incl. `room-event-stub`)
- **Expected output:** `recording_lands_with_hash_on_chain` passes; `clean_posture_no_recording_when_disabled` + `participant_floor_fetch` still pass.
- **Brief-Scope outputs to verify:** `services/bridge/docker-compose.e2e.yml` contains a `room-event-stub` service publishing `3000:3000`.

### Story 2: rtc-disabled town hall seats no chair (R7)

- **Composing tasks:** Task 3 (`[P]`)
- **Checkpoint command:** `cargo test --manifest-path services/bridge/Cargo.toml --test room_provisioning -- --ignored` (advisor; bridge image rebuilt)
- **Expected output:** `rtc_disabled_townhall_clean_posture` passes (`matrix_room_id` set, `chair_id` NULL); `anonymous_townhall_identity_never_reaches_livekit` still passes.
- **Brief-Scope outputs to verify:**
  - `crates/api/api_common/src/governance.rs` `CaseTransitionEvent` contains `rtc_enabled`.
  - `crates/api/api_utils/src/bridge_notify.rs` reads `rtc_enabled` config + sets it in the constructor.
  - `services/bridge/src/room_provisioner.rs` short-circuits on `event.rtc_enabled == Some(false)`.
  - `services/bridge/tests/room_provisioning.rs` payload contains `"rtc_enabled": false`.

### Story 3: Mute-all completes under 500 ms

- **Composing tasks:** Task 2 (`[P]`)
- **Checkpoint command:** `cargo test --manifest-path services/bridge/Cargo.toml --test emergency_mute -- --ignored` (advisor, full stack)
- **Expected output:** `mute_all_drops_all_publishers_cross_instance_under_500ms` passes (elapsed < 500 ms).
- **Brief-Scope outputs to verify:** `services/bridge/tests/emergency_mute.rs` wraps `update_participant` in `tokio::time::timeout`.

> `/brehon-verify` iterates these stories against the worktree branch; a phantom
> (task complete but output absent) triggers catch-fire per advisor-orchestrator.md.

---

## 17. Completion checklist

- [ ] Task 0 audit complete
- [ ] Tasks 1–3 committed + pushed; validate DQ entries written (write-then-STOP)
- [ ] Advisor ran §15 (compile proof + 3 e2e targets); all green
- [ ] §16a stories all `[done]`
- [ ] Retro committed (Task 4)
- [ ] BM opens PR against `governance-v0`
- [ ] CodeRabbit review triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report shows all stories ✓
- [ ] Phase branch retained for retro reads

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `caddy respond` flags differ on the pinned tag → stub fails to start | LOW | MED | §10.1 fallback to `traefik/whoami` (200-to-any); advisor curl-probes `:3000` after `up -d` before running the recording test |
| Stale bridge image runs the OLD gate (defect 2 silently still seats chair) | MED | HIGH | R-BRIDGEREBUILD: `up -d --build bridge` mandatory before the defect-2 e2e run (§15.2 / §15.3 checkbox) |
| `rtc_enabled` declared on only one struct → silent wire break | LOW | HIGH | R-WIRECONSISTENCY: §10.2 verbatim block on BOTH; §15.3 grep check; single worker owns both ends |
| 200 ms timeout too tight for a real in-instance revoke (D2 pilot) | LOW | LOW | base stack has no connected publishers (all timeout by absence); real revoke round-trip is tens of ms ≪ 200 ms; cross-instance timing is D2-manual per DQ `3004b6625b83-001` |
| Host :3000 occupied at run time | LOW | MED | Task 0 Probe 2 + R-PORT3000; advisor frees it before `up -d` |
| 200-stub masks a real chain-persistence regression | LOW | LOW | by design — chain persistence is the governance suite's gate, not this bridge test (§12); documented, not silent |

---

## 19. Notes

- **Dead lesson reference in the brief:** the brief's Required Reading cites
  `.claude/lessons/feedback_build_what_tests_exercise.md`, which **does not exist** in
  the corpus (`ls .claude/lessons/ | grep build_what` → empty). The principle it names
  (validate by observable behaviour — defect-1 Option C rests on what the test
  actually asserts) is sound and was confirmed by **directly reading**
  `recording.rs:118-123` (the R-CHAIN assertion is `status().is_success()` only). The
  retro (Task 4) should either author that lesson or correct the brief template's
  reference. Recorded here rather than as a blocking DQ (the brief says raise
  ambiguity as `kind: log`, do NOT block; the finding is non-blocking and the plan
  proceeds).
- **ADR-016 check (defect 2):** M2 (governance-triggered rooms) is ADR-016's *first
  reference integration* of the cross-app backplane contract; ADR-016 explicitly
  defers wire formats to "the M1 sub-PRD and per-app integration ADRs … commits the
  principles, not the wire formats." Adding `rtc_enabled` to the M2 `CaseTransitionEvent`
  wire shape is an **additive field to an existing reference-integration contract** —
  it neither holds app admin creds, makes Brehon an IdP, nor changes the binary's
  two-plane boundary. It supersedes no ADR; no new ADR required.
- **Why defect 2 is one task, not a parallel split:** the four edits are an atomic
  wire-contract change. Two parallel workers could declare `rtc_enabled` inconsistently
  (`bool` vs `Option<bool>`, differing serde attrs) → exactly the silent-contract-break
  class `feedback_entry_kind_runtime_allowlist_check.md` warns about. One worker pins
  §10.2 verbatim on both ends. The ≤2-crate Sonnet ceiling is intentionally exceeded
  per §5.2 (documented exception).
- **Default-ON semantics rationale:** `None ⇒ RTC on` keeps every existing town-hall
  emit (which sends no `rtc_enabled`) behaving exactly as today (creds-gated). Only an
  explicit `Some(false)` — what the test sends and what the governance config now
  propagates — disables the seat. This is the minimal, back-compat-preserving choice.
- **Cohort:** Tasks 1, 2, 3 are file-disjoint (§11) → one `[P]` cohort, dispatched
  simultaneously. The cross-lane total cap (≤2 running Junior tasks) applies; if it
  binds, dispatch serially. No `requires:` between them.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — all three root causes confirmed against live test source on the phase branch; each fix is the brief's recommended option, validated by code reading (recording assertion, gate code, timed loop).
- **Cargo budget:** 9/10 — bridge compile ~3 GB; lemmy workspace check in Docker; well under any limit.
- **Test coverage:** 8/10 — each defect maps to one e2e target; the 200-stub deliberately scopes recording to "POST is well-formed + 2xx" (chain persistence stays the governance suite's gate, documented in §12).
