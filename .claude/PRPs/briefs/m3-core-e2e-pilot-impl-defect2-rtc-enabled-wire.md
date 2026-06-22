# Brief — m3-core-e2e-pilot impl Task 3 (defect 2): rtc_enabled wire-contract field + per-event gate

## §1 Role + dispatch line

`[role:impl-task] defect2-rtc-wire — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-defect2-rtc-enabled-wire.md`

You are the **impl-task** subagent (Sonnet 4.6). This is the ATOMIC wire-contract task: 4 files that MUST land together (governance struct + emitter + bridge mirror struct + gate + test). After the edits, write TWO DQ entries (compile proof + e2e), commit + push, **STOP**. Do NOT run cargo — the advisor runs the compile + e2e.

## §2 Scope

**Branch:** `phase-m3-core-e2e-pilot` (fork tip `d69d41c1a` or newer).

**4 files, ONE atomic change.** Add a per-event `rtc_enabled: Option<bool>` field to the `CaseTransitionEvent` wire contract, consumed at BOTH ends, so a town-hall event can say "rtc disabled" and the bridge skips the RTC stage seat. Plan: §10.2–§10.5 + Task 3.

### Root cause (verified live by advisor 2026-06-21)

`rtc_disabled_townhall_clean_posture` (case 77007) passes `matrix_room_id.is_some()` (Tuwunel swap worked) but FAILS `chair_id.is_none()` (R7). The bridge gates the chair seat on whether LiveKit **creds are configured** (`room_provisioner.rs:252`) — a STACK-WIDE condition — not on any per-event flag. The wire contract has no `rtc_enabled` field. In the e2e stack creds ARE present (cycle-4 `--keys`) → chair seats → R7 fails. The `rtc_enabled` governance config already exists (m3-core-infra `seed_rtc_enabled_config`); this task makes the bridge HONOUR it per-event.

### IMPLEMENT (4 files — apply each plan section verbatim)

**File 1 of 4 — `crates/api/api_common/src/governance.rs`** (governance struct, `~:889-908`): add the §10.2 field (VERBATIM) as the LAST field of `CaseTransitionEvent`:
```rust
    /// Per-event RTC toggle, mirrored on the wire from the governance `rtc_enabled`
    /// config. `None` (absent) defaults RTC ON for back-compat — events without the
    /// field keep the pre-existing creds-gated behaviour. `Some(false)` disables the
    /// RTC stage seat even when LiveKit creds are configured (criterion 146 / R7).
    #[serde(default)]
    pub rtc_enabled: Option<bool>,
```

**File 2 of 4 — `crates/api/api_utils/src/bridge_notify.rs`** (emitter, the sole constructor at `:136`): read the config (after the `messaging_enabled` gate, before building the payload) + add `rtc_enabled` to the `CaseTransitionEvent { … }` literal, per §10.3:
```rust
  let rtc_enabled = GovernanceMessagingConfig::read_current(pool, "instance", "rtc_enabled")
    .await?
    .and_then(|r| r.value_bool);            // Option<bool>: None when the config row is absent
  let payload = BridgeNotifyPayload::CaseTransition(CaseTransitionEvent {
    /* … existing fields unchanged … */
    chair_pseudonym: None,
    rtc_enabled,                            // ← new (defect 2; populated from config)
  });
```
**Confirm the exact config-read shape** against `crates/api/api/src/governance/bridge_read.rs:44-49` (the `rtc_enabled` read pattern) + `bridge_notify.rs:112-119` (the in-function `messaging_enabled` read using the same `GovernanceMessagingConfig::read_current(pool, …)`). Match the existing struct's field name for the bool (`value_bool` vs another accessor) — read the type before writing.

**File 3 of 4 — `services/bridge/src/room_provisioner.rs`**: (a) add the §10.2 field (BYTE-IDENTICAL to File 1) as the last field of the bridge-local `CaseTransitionEvent` (`:20-37`); (b) insert the §10.4 per-event short-circuit IMMEDIATELY BEFORE the creds gate at `:252`:
```rust
    if event.rtc_enabled == Some(false) {
        tracing::debug!(
            case_id = event.case_id,
            "rtc_enabled=false — skipping stage-mode setup (R7 negative invariant)"
        );
        return;
    }
    // … existing `let (api_key, api_secret) = match (…) { … }` creds gate UNCHANGED …
```

**File 4 of 4 — `services/bridge/tests/room_provisioning.rs`** (§10.5): (a) REMOVE the obsolete `:88-97` `LIVEKIT_API_KEY/SECRET` env-var `assert!` guard (it checks the TEST process env, but the gate reads the BRIDGE process config — wrong process; under the per-event contract the bridge legitimately HAS creds while the EVENT disables RTC) — replace with a one-line comment; (b) ADD `"rtc_enabled": false` to the `:120-138` JSON payload. Leave the `:172-179` R7 assertion UNCHANGED (it stays load-bearing).

## §2.4a ADR-constraint load-bearing clause (mandatory)

This task touches the bridge↔governance **wire contract** (ADR-016's M2 reference integration). The constraint is load-bearing, not decorative:

1. **The gate is `event.rtc_enabled == Some(false)` → `return`** at `room_provisioner.rs` immediately before the creds match. This is the R7 negative invariant (criterion 146): RTC disabled ⇒ NO stage seat, even with creds present.
2. **WHY it can't be deferred / weakened:** the whole point of defect 2 is that a stack-wide creds check cannot express a per-CASE rtc toggle. If the short-circuit is dropped or moved AFTER the creds gate, the chair seats whenever creds exist (the current bug) and R7 fails again. The field MUST be consumed at the gate, not just declared.
3. **DoD line:** `grep -n 'event.rtc_enabled' services/bridge/src/room_provisioner.rs` returns the short-circuit in the write path BEFORE the creds match; `grep 'rtc_enabled' crates/api/api_utils/src/bridge_notify.rs` returns BOTH the config read AND the constructor field.

**ADR-016 is NOT contradicted:** `rtc_enabled` is an ADDITIVE field to an existing reference-integration contract (it holds no app creds, makes Brehon no IdP, changes no plane boundary). Additive ≠ new ADR. If you believe it DOES contradict an ADR, STOP and raise a `kind: blocker` DQ — do not proceed.

## §3 Required reading (phase-branch versions)

- `crates/api/api_common/src/governance.rs:884-908` — `CaseTransitionEvent` (gains the field); mirror the `#[serde(default)] pub chair_pseudonym: Option<String>` field shape.
- `crates/api/api_utils/src/bridge_notify.rs:1-12, 108-145` — imports (`GovernanceMessagingConfig` in scope), the `messaging_enabled` read pattern, the sole `CaseTransitionEvent { … }` constructor at `:136`.
- `crates/api/api/src/governance/bridge_read.rs:44-49` — the verbatim `rtc_enabled` config read pattern to mirror in the emitter (confirm the bool accessor name).
- `services/bridge/src/room_provisioner.rs:20-37` (bridge mirror struct) + `:252-265` (the creds gate to amend).
- `services/bridge/tests/room_provisioning.rs:72-190` — the rtc_disabled test: the `:88-97` env guard to remove, the `:120-138` payload to extend, the `:172-179` R7 assertion (unchanged).
- `.claude/PRPs/plans/m3-core-e2e-pilot-e2e-fixes.plan.md` §10.2–§10.5, §11 (callsite enumeration), §19 (ADR-016 framing).
- `.claude/lessons/feedback_entry_kind_runtime_allowlist_check.md` — a field that compiles but isn't consumed at the far end is a SILENT wire break. Both ends MUST consume `rtc_enabled`.
- `.claude/lessons/feedback_forward_declared_items_need_allow_until_consumer.md` + `.claude/lessons/feedback_planner_enumerate_struct_callsites_for_addfield.md` — struct-field-add discipline; the plan §11 already enumerated the sole production constructor (`bridge_notify.rs:136`).
- `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md`.
- `.claude/decision-queue.json`.

## §4 Procedure

1. Confirm branch = `phase-m3-core-e2e-pilot`.
2. Re-run the callsite enumeration to confirm it still holds: `git grep -n 'CaseTransitionEvent {' -- crates/ services/` → exactly ONE production constructor (`bridge_notify.rs:136`); other hits are docs/plans. If a second production constructor exists, it MUST also gain the field (update + note it).
3. Apply Files 1–4 (verbatim §10.2 on Files 1+3 — byte-identical; §10.3/§10.4/§10.5 per their sections).
4. Verify scope: `git diff --stat` = exactly 4 files (the 2 crates files + the 2 bridge files in §11).
5. Verify the field is byte-identical on both structs: `grep -A1 'pub rtc_enabled' crates/api/api_common/src/governance.rs services/bridge/src/room_provisioner.rs` → same declaration both places.
6. Commit (ONE commit): `feat(bridge): rtc_enabled wire-contract field + per-event RTC gate (defect 2)`. Body: enumerate the 4 edit sites; note default-ON back-compat; `LESSON:` on per-event-vs-stack-wide gate. Co-Authored-By + Claude-Session trailers per repo convention.
7. Push.
8. Write TWO DQ entries (compile proof + e2e) via the helper scripts, commit + push. STOP.

DQ fragment 1 (compile proof — the lemmy workspace half):
```
kind: validate-pending-laptop-linux
from: impl
commands: ["./scripts/brehon/cargo-linux.sh check --workspace --features full"]
branch: phase-m3-core-e2e-pilot
phase_task: 3
result: null, log_slice: null, failed_commands: null
context: proves governance.rs + bridge_notify.rs compile in the lemmy workspace (the bridge e2e run does NOT build the lemmy workspace).
```
DQ fragment 2 (e2e — the bridge half):
```
kind: validate-pending-laptop-e2e
from: impl
commands: ["cargo test --manifest-path services/bridge/Cargo.toml --test room_provisioning -- --ignored"]
branch: phase-m3-core-e2e-pilot
phase_task: 3
result: null, log_slice: null, failed_commands: null
context: advisor REBUILDS the bridge image (docker compose ... up -d --build bridge) then runs against the FULL --profile rtc stack; compiles room_provisioner.rs + the test, proves the rtc_disabled R7 invariant. R-BRIDGEREBUILD — a stale image silently runs the old gate.
```

## §5 Constraints

- **EXACTLY 4 files** (§11): `crates/api/api_common/src/governance.rs`, `crates/api/api_utils/src/bridge_notify.rs`, `services/bridge/src/room_provisioner.rs`, `services/bridge/tests/room_provisioning.rs`. No others.
- **ONE commit** — the 4 edits are atomic (the workspace won't compile / the field is a silent break if split).
- **NO cargo run** — write-then-STOP. The advisor runs §15.1 (compile) + §15.2 (e2e with `--build bridge`).
- **R-WIRECONSISTENCY**: `#[serde(default)] pub rtc_enabled: Option<bool>` BYTE-IDENTICAL on both structs.
- **Default-ON**: `None`/absent ⇒ RTC on (back-compat); only `Some(false)` disables. Do NOT change behaviour for events without the field.
- **Keep the R7 assertion** (`room_provisioning.rs:172-179`) unchanged — it's the test's load-bearing gate.
- **NO provisioning-logic rework** — one field + one short-circuit + test setup. If the change wants to grow beyond 4 files, STOP and DQ.
- DQ mid-task push mandatory.

## §6 DoD

- `git diff --stat HEAD~1` = 4 files (the §11 set).
- `grep -c 'pub rtc_enabled: Option<bool>' crates/api/api_common/src/governance.rs` → 1.
- `grep -c 'pub rtc_enabled: Option<bool>' services/bridge/src/room_provisioner.rs` → 1.
- `grep -c 'event.rtc_enabled == Some(false)' services/bridge/src/room_provisioner.rs` → 1 (the short-circuit, BEFORE the creds match).
- `grep -c 'rtc_enabled' crates/api/api_utils/src/bridge_notify.rs` → ≥2 (config read + constructor field).
- `grep -c '"rtc_enabled": false' services/bridge/tests/room_provisioning.rs` → 1.
- The `:88-97` `LIVEKIT_API_KEY` env `assert!` REMOVED; the R7 assert retained.
- BOTH validate DQ entries committed + pushed.

## §7 HANDOVER

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-impl-defect2-rtc-enabled-wire
  field_governance: <"rtc_enabled added to governance.rs CaseTransitionEvent" | "FAIL: <reason>">
  field_bridge: <"rtc_enabled added to room_provisioner.rs (byte-identical)" | "FAIL: <reason>">
  emitter_populates: <"bridge_notify.rs reads config + sets field at :136" | "FAIL: <reason>">
  gate_short_circuit: <"room_provisioner.rs returns early on Some(false) before creds match" | "FAIL: <reason>">
  test_payload: <"room_provisioning.rs sends rtc_enabled:false; :88-97 env guard removed" | "FAIL: <reason>">
  callsite_enum: <"single production constructor confirmed (bridge_notify.rs:136)" | "<n> constructors — all updated">
  files_changed: <list — MUST be 4>
  dq_linux_id: <id>
  dq_e2e_id: <id>
  notes: "<confirm field byte-identical both structs; confirm one commit; confirm R7 assert retained>"
```
