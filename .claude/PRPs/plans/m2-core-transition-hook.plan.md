# Plan: M2-core — Governance Case Transition Hook (in-binary slice)

## Summary

This plan delivers the **in-binary, fully cargo-workspace-gated slice of M2-core**: a `governance_case_after_transition` fire-and-forget notification fired at every `CaseStatus` transition commit point (mirroring the existing PM-scoped `bridge_notify` style), the 10 zero-migration `ENTRY_KIND_ROOM_*` governance-log consts, and a typed `append_room_event(...)` library wrapper the (future, out-of-process) bridge daemon will call into to write `Room::*` metadata onto the hash chain. It does **NOT** build the `services/bridge/` provisioning daemon, any HTTP callback route, or any Matrix integration — those are the bridge-side plan. The slice is pure in-`crates/` Rust: every task is gated by `cargo check --workspace` / `cargo clippy --workspace -- -D warnings` / `cargo test --test e2e -p lemmy_server`, and the workspace pulls zero Matrix deps (M1 story-6 invariant preserved — `services/bridge` is in `exclude`).

## Source

- PRD: [`.claude/PRPs/prds/m2-governance-triggered-rooms.prd.md`](../prds/m2-governance-triggered-rooms.prd.md) — Implementation Phases table, **Phase 1 (hook) + Phase 2 (entry kinds) + the binary-side of Phase 4 (append callback path)**. Bridge-side Phases 3, 4-daemon, 5 are out of this plan's scope (separate plan).
- Relevant design docs: [99 ADR-004 amendment](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), [99 ADR-008](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), [99 ADR-013](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), [99 ADR-015](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), [99 ADR-016](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), [99 OQ-009 resolution](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)
- Relevant ADRs: **ADR-004** (plane separation — bridge state stays out of the binary), **ADR-008** (append-only signed log — `Room::*` metadata only, never content), **ADR-013** (emergency-remove observer), **ADR-015** (pseudonym + scrub), **ADR-016** (first reference integration — the hook is the B-side seam), **ADR-012** (new hook alongside existing PM hooks, no rename/removal).

## Problem Statement

Every `CaseStatus` transition in the Brehon binary today is a bare `update(moderation_case::table).set(status.eq(...))` with **zero observable side-effect** — confirmed by codebase exploration 2026-06-05 (no `governance_case_after_transition` hook exists; only PM-scoped `plugin_hook_notification` + `bridge_notify` do). M2's governance-triggered rooms need a notification fired at each transition so an out-of-process room-provisioning service can react. This plan adds that notification (and the log primitives the bridge will write back through) **without** altering any transition's decision logic — observer-only — and without adding Matrix deps to the workspace.

## Solution Statement

Three cohesive in-binary additions:

1. **Hook (Phase 1)** — extend `crates/api/api_utils/src/bridge_notify.rs` with a `governance_case_after_transition(context, case, old_status, new_status)` fire-and-forget fn, gated by the existing `messaging_enabled` config read, POSTing a **discriminated-union** payload (`PM event | case-transition event`) to the bridge URL. Fire it at all 11 production transition commit points — **after** the enclosing `run_transaction(...).await?` returns (txn already committed, `conn` free), mirroring `notify.rs`'s `spawn_try_task` → internal-fn → `.ok()` style. The hook fn lives in `api_utils` (both `lemmy_api` and `lemmy_api_crud` depend on it; the cron sites in `crates/routes` do too).

2. **Entry kinds (Phase 2)** — 10 `ENTRY_KIND_ROOM_*` consts in `crates/db_schema/src/source/governance/governance_log.rs` (zero-migration; `entry_kind` is TEXT), the matching alphabetical `pub use` re-export in the `lemmy_api` shim, and a new `## M2 room kinds (10)` section + bumped total in `.claude/rules/governance-log-entry-kind-registry.md`.

3. **Append wrapper (Phase 4-binary-side)** — a typed `append_room_event(pool, kind, RoomEventPayload, actor_pseudonym)` wrapper in `crates/api/api/src/governance/governance_log.rs` that (a) refuses any `kind` not in the Room::* set, (b) serialises a typed `RoomEventPayload`, and (c) calls the existing `governance_log::append(...)` (which scrubs + signs + chains via the Postgres trigger). No HTTP route, no auth surface — the bridge daemon's HTTP ingress is deferred to the bridge-side plan (user decision 2026-06-05).

## Metadata

| Field | Value |
|---|---|
| Type | CROSS_CUTTING (hook wiring + log primitives) |
| Complexity | MEDIUM |
| Crates Affected | `lemmy_api_utils`, `lemmy_api`, `lemmy_api_crud`, `lemmy_api_common`, `lemmy_db_schema`, `lemmy_routes` (cron wiring), `lemmy_server` (e2e tests) |
| v0/M Step | M2-core (post-v1, ADR-016 first reference integration) |
| Dependencies | M1 (shipped: `bridge_notify`, `governance_messaging_config`, `append()`, identity validator, pseudonym helper) |
| Estimated Tasks | 9 |

---

## Flow Design

### Before State

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                              BEFORE STATE                                       ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  CaseStatus transition (e.g. Open → JurySelection in admin_assign_jury)         ║
║    → run_transaction { update(moderation_case).set(status.eq(JurySelection)) }  ║
║    → governance_log::append("jury_assigned", …)   (inside the txn)              ║
║    → Ok(Json(response))                                                         ║
║                                                                                ║
║  DATA_FLOW: status update → log append → response. NOTHING observes the         ║
║             transition out-of-process. No room. No Room::* entries.             ║
║  PAIN_POINT: a 5-juror panel has no deliberation room; ADR-016 seam unused.     ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### After State

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                               AFTER STATE                                       ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  CaseStatus transition                                                          ║
║    → run_transaction { update(...).set(status.eq(JurySelection)); append(…) }   ║
║    → (txn COMMITTED, conn free)                                                 ║
║    → governance_case_after_transition(ctx, &case, old, JurySelection)           ║
║         └─ reads messaging_enabled; false → no-op (clean v0 posture)            ║
║         └─ true → spawn_try_task → POST {type:"case_transition", case_id,        ║
║                   old_status, new_status} to bridge; .ok() swallows errors      ║
║    → Ok(Json(response))   (NEVER blocked by hook; NEVER fails on bridge-down)   ║
║                                                                                ║
║  [bridge-side, OUT OF SCOPE] bridge provisions room, then calls back via         ║
║   append_room_event(…) → governance_log WHERE entry_kind LIKE 'room_%'           ║
║                                                                                ║
║  DATA_FLOW: status update → log append → COMMIT → fire-and-forget signal.        ║
║  VALUE_ADD: ADR-016 B-side seam is live; rooms become provisionable; the         ║
║             append-only log gains its Room::* primitives + a typed safe writer.  ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### Endpoint Changes

**No HTTP endpoint changes in this plan.** The hook is an internal notification fired inside existing handlers; the append wrapper is library code. (The bridge HTTP callback endpoint is explicitly deferred — user decision 2026-06-05.) The only externally-observable change is: when `messaging_enabled=true`, a fire-and-forget POST is emitted to the bridge URL after each transition.

---

## Mandatory Reading (implementation agent MUST read before starting)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `crates/api/api_utils/src/bridge_notify.rs` | 1-49 | The exact fire-and-forget + config-gate pattern to extend. MIRROR. |
| P0 | `crates/api/api_utils/src/notify.rs` | 285-310 | `spawn_try_task` → internal fn → `bridge_notify::notify_if_enabled(...).ok()` wiring. MIRROR for how the hook is invoked off the committed-txn path. |
| P0 | `crates/db_schema/src/source/governance/governance_log.rs` | 112-234, 255-314 | `ENTRY_KIND_*` const block (style + append point) + the full `append()` signature/body the wrapper calls. |
| P0 | `crates/api/api/src/governance/governance_log.rs` | 39-66 | The explicit alphabetical `pub use` shim every new const must join. |
| P0 | `.claude/rules/governance-log-entry-kind-registry.md` | all | Add-a-kind 4-step procedure + the M2 placeholder section + the hardcoded total to bump + the collision check. |
| P0 | `crates/api/api/src/governance/admin_assign_jury.rs` | 95-201 | Transition site 1 + its `run_transaction` boundary (the canonical handler shape). |
| P0 | `crates/api/api/src/governance/submit_jury_vote.rs` | 145-151, 201, 215, 379-382, 501-518, 974-977, 1020-1026 | 5 transition sites across 2 fns + the single `run_transaction` boundary at 149. |
| P0 | `crates/api/api_crud/src/governance/request_appeal.rs` | 66-72, 85-158 | Cross-crate transition site (api_crud) + its txn boundary. |
| P0 | `crates/api/api/src/governance/appeal_window_expiry.rs` | 38-75 | Bare-`.execute` cron site — fire after the loop, accumulate transitions. |
| P0 | `crates/api/api/src/governance/sponsor_liability_grace.rs` | 160-169, 405-516 | Per-case-txn cron sites (escape/fire) — fire after each per-case `run_transaction`. |
| P0 | `crates/api/api_crud/src/governance/revoke_endorsement.rs` | 141-149, 247-312 | **Site 12** (Task-0 find) — escape-on-revoke loop inside `process_revocation`'s `run_transaction`; accumulate escaped cases, fire after commit. |
| P1 | `crates/api/api/src/governance/admin_close_case.rs` | 59-75 | Transition site 7 — `try_from` MOVES `case`; capture `old_status` (Copy) before line 67. GOTCHA. |
| P1 | `crates/db_schema/src/source/governance/redaction.rs` | 101-155 | `scrub_json` — string values redacted; keep room identifiers non-string-shaped. |
| P1 | `crates/api/api/src/governance/actor_pseudonym_helper.rs` | 26-80 | `get_or_create` / `get` — for any pseudonym attribution in `append_room_event`. |
| P1 | `crates/db_schema/src/source/governance/governance_messaging_config.rs` | 75-91 | `read_current(pool, "instance", "messaging_enabled")` — the gate. |
| P1 | `crates/db_views/site/src/api.rs` | 762-770 | `#[serde(tag = "type_", rename_all = "snake_case")]` — the internally-tagged enum pattern for the discriminated-union payload. MIRROR. |
| P1 | `crates/server/tests/e2e/jury_mechanics.rs` | 955-1100 | `submit_jury_vote_severe_panel_meets_threshold` — full CaseStatus-transition test to MIRROR. |
| P1 | `crates/server/tests/e2e/governance.rs` | 3840-3973 | `governance_events_notify_fires` — side-effect-on-write test shape (LISTEN/mock pattern). |
| P1 | `crates/server/tests/e2e.rs` | 2223-2256 | `include!` wiring — new tests go in an `include!`d file, never a bare `tests/*.rs`. |

**External Documentation:** None required. All crate versions and APIs are confirmed from the workspace (`reqwest 0.13.4` + `reqwest-middleware 0.5.2` via `context.client()`, `serde 1.0.228`, `serde_json 1.0.150`). No new crate, no unfamiliar Diesel/AP feature.

---

## Patterns to Mirror

**FIRE_AND_FORGET_CONFIG_GATE** (extend this fn's sibling):
```rust
// SOURCE: crates/api/api_utils/src/bridge_notify.rs:13-49
pub async fn notify_if_enabled(
  context: &LemmyContext,
  view: &PrivateMessageView,
) -> LemmyResult<()> {
  let pool = &mut context.pool();
  let enabled = match GovernanceMessagingConfig::read_current(pool, "instance", "messaging_enabled")
    .await?
  {
    Some(row) => row.value_bool.unwrap_or(false),
    None => false,
  };
  if !enabled {
    return Ok(());
  }
  #[derive(serde::Serialize)]
  struct Payload { private_message_id: i32, creator_id: i32, recipient_id: i32 }
  let payload = Payload { /* ... */ };
  if let Err(e) = context.client().post(BRIDGE_NOTIFY_URL).json(&payload).send().await {
    tracing::warn!("bridge notify failed (bridge may be down — non-fatal): {e}");
  }
  Ok(())
}
```

**OFF_TXN_HOOK_INVOCATION** (how to call the hook from a handler):
```rust
// SOURCE: crates/api/api_utils/src/notify.rs:285-306
pub fn notify_private_message(view: &PrivateMessageView, is_create: bool, context: &LemmyContext) {
  let view = view.clone();
  let context = context.clone();
  spawn_try_task(async move { notify_private_message_internal(&view, is_create, &context).await })
}
// inside the internal fn, AFTER the committed write:
//   crate::bridge_notify::notify_if_enabled(context, view).await.ok();
```

**ENTRY_KIND_CONST** (append after line 234, before the `SIGNING_KEY_ENV` at 236):
```rust
// SOURCE: crates/db_schema/src/source/governance/governance_log.rs:220-234
pub const ENTRY_KIND_FEDERATION_PEER_TRUST_CHANGED: &str = "federation_peer_trust_changed";
// New M2 consts follow the same `pub const NAME: &str = "snake_case";` style;
// long names wrap the value to the next line indented 2 spaces (rustfmt).
```

**SHIM_REEXPORT** (insert each new const in alphabetical position):
```rust
// SOURCE: crates/api/api/src/governance/governance_log.rs:39-66
pub use lemmy_db_schema::source::governance::governance_log::{
  ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED, /* … alphabetical … */
  ENTRY_KIND_THRESHOLD_MET, ENTRY_KIND_VOTE_OUTCOME_RECORDED, GovernanceLog,
  GovernanceLogInsertForm, append,
};
```

**APPEND_CALL** (what `append_room_event` calls internally):
```rust
// SOURCE: crates/apub/activities/src/governance/inbox.rs:721-727 (None-pseudonym, runtime kind)
governance_log::append(&mut (&mut *conn).into(), entry_kind, payload, None).await?;
// or from a pool directly (handler-entry style, accept_jury_assignment.rs):
governance_log::append(&mut context.pool(), ENTRY_KIND_X, json!({...}), Some(pseudonym)).await?;
```

**TAGGED_ENUM_PAYLOAD** (the discriminated union):
```rust
// SOURCE: crates/db_views/site/src/api.rs:762-770
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "type_", rename_all = "snake_case")]
pub enum PostOrCommentOrPrivateMessage { Post(Post), Comment(Comment), PrivateMessage(PrivateMessage) }
```

**E2E_TRANSITION_TEST** (mirror for the hook test):
```rust
// SOURCE: crates/server/tests/e2e/jury_mechanics.rs:955-1100
async fn submit_jury_vote_severe_panel_meets_threshold() -> lemmy_utils::error::LemmyResult<()> {
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  // … seed instance/community/users/jurors; admin_assign_jury; accept; vote …
  // … assert moderation_case.status == CaseStatus::Decided + governance_log row …
  Ok(())
}
```

---

## Files to Change

| File | Action | Justification |
|---|---|---|
| `crates/api/api_common/src/governance.rs` | UPDATE | Add `CaseTransitionEvent` + the `BridgeNotifyPayload` tagged-union enum (shared DTO, ts-rs-free). |
| `crates/api/api_utils/src/bridge_notify.rs` | UPDATE | Refactor the inline PM `Payload` into the tagged union; add `governance_case_after_transition(...)` sibling fn. |
| `crates/api/api/src/governance/admin_assign_jury.rs` | UPDATE | Fire hook after `run_transaction` (site 1). |
| `crates/api/api/src/governance/submit_jury_vote.rs` | UPDATE | Fire hook after `run_transaction` for the 5 sites (2,3,4,5,6) — derive new status from outcome. |
| `crates/api/api/src/governance/admin_close_case.rs` | UPDATE | Fire hook (site 7); capture `old_status` before the `try_from` move. |
| `crates/api/api_crud/src/governance/request_appeal.rs` | UPDATE | Fire hook (site 8, api_crud crate). |
| `crates/api/api/src/governance/appeal_window_expiry.rs` | UPDATE | Accumulate transitions in-loop; fire hooks after the loop (cron site 9). |
| `crates/api/api/src/governance/sponsor_liability_grace.rs` | UPDATE | Fire hook after each per-case `run_transaction` (cron sites 10,11). |
| `crates/db_schema/src/source/governance/governance_log.rs` | UPDATE | Add 10 `ENTRY_KIND_ROOM_*` consts. |
| `crates/api/api/src/governance/governance_log.rs` | UPDATE | Add 10 consts to the alphabetical `pub use` shim; add `append_room_event` wrapper. |
| `.claude/rules/governance-log-entry-kind-registry.md` | UPDATE | New `## M2 room kinds (10)` section; bump total 55 → 65 + arithmetic line. |
| `crates/server/tests/e2e/governance.rs` (or new `include!`d file) | UPDATE | Add hook-fires / suppression / append-wrapper tests. |
| `crates/api/api_utils/Cargo.toml` | UPDATE (maybe) | Add `serde_json` to `[dependencies]` if the payload needs `json!`/`Value` (api_utils currently has serde but not serde_json). |

---

## NOT Building (scope limits)

- **The `services/bridge/` room-provisioning daemon** — the entire bridge-side (provisioning logic, `bridge_room` table, OQ-009 graduated reveal, idempotency, Matrix calls). Separate bridge-side plan. (PRD Phase 3 + Phase 4-daemon + Phase 5.)
- **Any HTTP callback route / shared-secret auth** for the bridge to reach `append_room_event` — deferred to the bridge-side plan when its consumer exists (user decision 2026-06-05). This plan ships only the library wrapper.
- **B-publish sanction propagation / B-actor portable IDs** — M2-late, gated on OQ-ADR016-02/-04/-03.
- **`ENTRY_KIND_ROOM_RECORDING_UPLOADED` emission** — the const is registered for completeness but never emitted (M3 / OQ-V2-04).
- **Hashing room content** — only the 10 lifecycle metadata kinds; speech inside rooms is never on the chain (research §3.4.2 C2.7).
- **Extending the identity-policy validator to room scopes** — noted as a follow-up (the M1 validator covers `jury*`/`appeal*` only; room-scope pinning belongs with the bridge-side room-config work that introduces those scopes).
- **Any change to a transition's decision logic** — observer-only; M2 never alters which transition fires.
- **A new `CaseStatus` variant.**

---

## Step-by-Step Tasks

Execute in order. One commit per task. Each task has a MIRROR reference, exact file path(s), and a `cargo`/`diesel` validation command. Per the pre-Shape-G constraint, the impl-task writes a `validate-pending-laptop` DQ entry for the cargo command and stops — the laptop advisor runs cargo (see CLAUDE.md "NO CARGO ON ELITEDESK").

> Skill triggers: `/cargo-validate` for every VALIDATE cargo line (gating signal). `/test-write` for Task 8 (new e2e cases needing fixture scaffolding). `/edit-mechanical` for Task 4 (the entry-kind const + shim propagation is a repeat-pattern edit across two files + a doc).

### Task 0: Pre-flight — re-enumerate transition sites against current HEAD
- **ACTION**: Before any edit, run `rg -n "moderation_case::status\.eq\(CaseStatus::" crates/api/` and `rg -n "\.set\(moderation_case::status" crates/api/` and reconcile the hit list against the 11 sites in this plan's Mandatory Reading. The PRD said "~8"; exploration found **11 production statements** (8 handler + 3 cron). If HEAD has drifted (a site added/removed since 2026-06-05), STOP and raise a `kind: blocker` DQ — do not guess.
- **MIRROR**: the transition-site table below.
- **GOTCHA**: `submit_jury_vote.rs` sites 3 (`SponsorLiabilityPending`) and 4 (`Decided`) are mutually exclusive (`if path_kind == Pending {…} else {…}`) — a single hook fired after the `if/else` covers both; derive new status from `outcome`/`path_kind`, do not double-fire.
- **GOTCHA**: site 12 (`sponsor_liability_grace.rs:297` `fire_or_escape_case`) has **no production callers** (only `_inner` is wired). Do NOT wire it unless you place the hook in a shared helper that both paths call.
- **VALIDATE**: `rg -c "moderation_case::status.eq" crates/api/` returns the expected count; no `cargo`. Commit a one-line `chore` noting the verified site count.

**Transition-site reference (verified 2026-06-05):**

| # | File:line | fn | old → new | Crate | Fire point |
|---|---|---|---|---|---|
| 1 | `admin_assign_jury.rs:198` | process_assignment | Open/ThresholdMet/EmergencyRemove → JurySelection | api | after `run_transaction` returns in handler |
| 2 | `submit_jury_vote.rs:380` | process_vote (deadlock) | Active → AdminReview | api | after txn at :149 |
| 3 | `submit_jury_vote.rs:503` | process_vote (SL pending) | Active → SponsorLiabilityPending | api | after txn at :149 |
| 4 | `submit_jury_vote.rs:513` | process_vote (decided) | Active → Decided | api | after txn at :149 |
| 5 | `submit_jury_vote.rs:975` | process_appeal_vote (deadlock) | Appealed → AdminReview | api | after txn at :149 |
| 6 | `submit_jury_vote.rs:1022` | process_appeal_vote (verdict) | Appealed → Closed | api | after txn at :149 |
| 7 | `admin_close_case.rs:71` | process_close | (any but Closed) → Closed | api | after txn; capture old before move at :67 |
| 8 | `request_appeal.rs:156` | process_appeal | Decided/SponsorLiabilityPending → Appealed | **api_crud** | after txn at :70 |
| 9 | `appeal_window_expiry.rs:58` | run_appeal_window_expiry_batch | Decided → Closed | api (cron) | accumulate in loop; fire after loop |
| 10 | `sponsor_liability_grace.rs:480` | fire_or_escape_case_inner (escape) | SponsorLiabilityPending → SponsorLiabilityEscaped | api (cron) | after each per-case txn |
| 11 | `sponsor_liability_grace.rs:511` | fire_or_escape_case_inner (fire) | SponsorLiabilityPending → SponsorLiabilityFired | api (cron) | after each per-case txn |
| **12** | `revoke_endorsement.rs:308` | process_revocation (escape-on-revoke loop) | SponsorLiabilityPending → SponsorLiabilityEscaped | **api_crud** | after `run_transaction` at :142; loops over pending_cases → accumulate + fire once per escaped case |

> **Task-0 amendment (2026-06-05, DQ `a3d0e9941441-050`):** site 12 (`revoke_endorsement.rs:308`) was MISSED by the original exploration. It is a genuine production transition — synchronous SL-escape when a sponsor revokes during the grace window — inside `process_revocation`'s `run_transaction` (:142), looping over `pending_cases`. **Total production transition statements = 12, not 11.** Wiring (Task 7) must cover it. The latent `fire_or_escape_case.rs:297` (no prod callers) remains unwired.

### Task 1: ADD the shared DTO — `CaseTransitionEvent` + `BridgeNotifyPayload` tagged union in `crates/api/api_common/src/governance.rs`
- **ACTION**: Add a `#[serde(tag = "type_", rename_all = "snake_case")]` enum `BridgeNotifyPayload` with two variants: `PrivateMessage(PrivateMessagePayload)` (the existing PM shape, lifted from the inline `bridge_notify.rs` struct) and `CaseTransition(CaseTransitionEvent)`. `CaseTransitionEvent` carries `case_id: i32`, `old_status: Option<CaseStatus>`, `new_status: CaseStatus`, `community_id: Option<i32>`, `target_type: String` (for the bridge to route room type). Use plain `serde` derives — **omit ts-rs** (bridge consumer is not the TS client).
- **IMPLEMENT**: integer fields only (newtype `.0` unwraps); `CaseStatus` serialises `snake_case` (its existing `#[serde(rename_all = "snake_case")]`). No PII fields — `case_id`/`community_id` are opaque ints; the bridge resolves pseudonyms itself.
- **MIRROR**: `crates/db_views/site/src/api.rs:762-770` (tagged enum); `crates/api/api_common/src/governance.rs` existing DTO style.
- **IMPORTS**: `use lemmy_db_schema_file::enums::CaseStatus;` (confirm the enum is reachable from api_common — if it requires a feature gate, mirror how an existing api_common governance DTO imports a governance enum).
- **GOTCHA**: `CaseStatus` derives `Serialize`/`Deserialize` unconditionally (enums.rs:382) — usable in api_common without the `full` feature. Verify with the check.
- **VALIDATE**: `cargo check -p lemmy_api_common --features full`

### Task 2: REFACTOR `bridge_notify.rs` PM path onto the tagged union
- **ACTION**: Replace the inline `#[derive(serde::Serialize)] struct Payload {…}` in `notify_if_enabled` with constructing `BridgeNotifyPayload::PrivateMessage(...)` from Task 1. Behaviour unchanged — still POSTs to `BRIDGE_NOTIFY_URL`, still `messaging_enabled`-gated, still `.ok()`/`warn!` on error. This proves the union round-trips before adding the new variant's producer.
- **IMPLEMENT**: import the DTO from `lemmy_api_common::governance`; serialise the enum (the `type_` tag lands in the JSON automatically).
- **MIRROR**: `crates/api/api_utils/src/bridge_notify.rs:13-49`.
- **GOTCHA**: api_utils may not have `serde_json` as a direct dep — but `.json(&payload)` on a `Serialize` value needs only `reqwest`'s `json` feature (present). Only add `serde_json` to `api_utils/Cargo.toml` if you actually construct a `Value` (you don't here — you pass the typed enum). Confirm no new dep needed.
- **VALIDATE**: `cargo check -p lemmy_api_utils --features full`

### Task 3: ADD `governance_case_after_transition(...)` to `bridge_notify.rs`
- **ACTION**: Add `pub async fn governance_case_after_transition(context: &LemmyContext, case: &ModerationCase, old_status: Option<CaseStatus>, new_status: CaseStatus) -> LemmyResult<()>` — same config gate + fire-and-forget POST shape as `notify_if_enabled`, building `BridgeNotifyPayload::CaseTransition(CaseTransitionEvent { … })` from `case.id.0`, `old_status`, `new_status`, `case.community_id.map(|c| c.0)`, `case.target_type` rendered to string. Errors swallowed via `tracing::warn!`.
- **IMPLEMENT**: `old_status: Option<CaseStatus>` (None for the emergency-remove creation case if ever wired; all 11 transition sites pass `Some`). Read `messaging_enabled` exactly as the sibling does.
- **MIRROR**: `bridge_notify.rs:13-49` + the Task-1 enum.
- **GOTCHA**: import `ModerationCase` + `CaseStatus` from `lemmy_db_schema` (api_utils already depends on `lemmy_db_schema`). Do NOT take `&mut context.pool()` outside the gate read — clone-free, same as sibling.
- **VALIDATE**: `cargo check -p lemmy_api_utils --features full`

### Task 4: ADD the 10 `ENTRY_KIND_ROOM_*` consts + shim re-export + registry doc
- **ACTION**: (a) In `crates/db_schema/src/source/governance/governance_log.rs`, after line 234, add a `// M2 room kinds (10)` block with 10 `pub const ENTRY_KIND_ROOM_*: &str = "room_*";` consts. (b) In `crates/api/api/src/governance/governance_log.rs`, add all 10 to the alphabetical `pub use` list. (c) In `.claude/rules/governance-log-entry-kind-registry.md`, add `## M2 room kinds (10, this sub-phase)` table after the `v1-federation-inbound-b` section, mark the emitter as `append_room_event (pending bridge-side)` per the pre-landed-const exemption, and bump the Acceptance-invariants total `55` → `65` (+ the arithmetic line).
- **IMPLEMENT**: the 10 kinds (from PRD §Proposed Solution + research §3.4.2 C2.1–C2.7): `ROOM_CREATED`, `ROOM_MEMBER_JOINED`, `ROOM_MEMBER_LEFT`, `ROOM_CLOSED`, `ROOM_ARCHIVED`, `ROOM_DELETED`, `ROOM_IDENTITY_REVEALED`, `ROOM_EMERGENCY_PROVISIONED`, `ROOM_APPEAL_PROVISIONED`, `ROOM_RECORDING_UPLOADED` (registered, never emitted in M2). String values = name minus `ENTRY_KIND_` prefix, lowercased: `"room_created"`, etc. Confirm exact list against PRD before committing; if the PRD's "10 Room::* entries" enumerate differently, the PRD wins — raise a `kind: log` DQ noting the mapping.
- **MIRROR**: `governance_log.rs:220-234` (const style); `governance_log.rs:39-66` (shim); the registry doc's existing `## v1-federation-inbound-a` section.
- **GOTCHA**: shim list is rustfmt-alphabetical — insert each in correct position or `cargo fmt` will reorder and bloat the diff.
- **GOTCHA**: run the registry collision check (Count A `^pub const ENTRY_KIND_` == Count B unique string literals; `sort | uniq -d` empty). New `room_*` values must not collide with any existing 55.
- **VALIDATE**: `cargo check -p lemmy_db_schema --features full && cargo check -p lemmy_api --features full`; then `rg -c "^pub const ENTRY_KIND_" crates/db_schema/src/source/governance/governance_log.rs` == 65.

### Task 5: ADD the `append_room_event(...)` typed wrapper in `crates/api/api/src/governance/governance_log.rs`
- **ACTION**: Add `pub async fn append_room_event(pool: &mut DbPool<'_>, kind: &str, payload: RoomEventPayload, actor_pseudonym: Option<String>) -> LemmyResult<GovernanceLog>` that (a) validates `kind` is one of the 10 `ENTRY_KIND_ROOM_*` consts (else `LemmyErrorType::Unknown("not a room entry kind: …")`), (b) serialises `payload` to `serde_json::Value`, (c) calls `append(pool, kind, value, actor_pseudonym)`. Define `RoomEventPayload` (a `Serialize` struct: `case_id: i32`, `matrix_room_id: Option<String>`, `lifecycle_stage: String`, `member_count: Option<i32>`, …) — integer + opaque-token fields, NO raw usernames/emails (scrub would strip them and the bridge owns pseudonyms).
- **IMPLEMENT**: the kind-allowlist as a `const ROOM_KINDS: &[&str] = &[ENTRY_KIND_ROOM_CREATED, …];` + `if !ROOM_KINDS.contains(&kind) { return Err(...) }`. This is the safety gate — the bridge can only write Room::* kinds through this wrapper, never arbitrary entry kinds (ADR-008 integrity).
- **MIRROR**: `governance_log.rs` append call sites (`inbox.rs:721` None-pseudonym form); `RoomEventPayload` follows the `bridge_notify.rs` inline-struct field style.
- **GOTCHA**: `append` lives in `db_schema`; the shim already re-exports it (Task 4 keeps it in the `pub use`). Call it via the shim path so the wrapper and existing handlers use one symbol.
- **GOTCHA**: `RoomEventPayload` string fields pass through `scrub_json` inside `append` — `matrix_room_id` (`!room:server` form) may match the URL/mention regex; verify it survives scrub or store it as an opaque non-`!`-prefixed token. Add a test asserting a representative `matrix_room_id` round-trips unredacted (or document the expected transform).
- **VALIDATE**: `cargo check -p lemmy_api --features full`

### Task 6: WIRE the hook at the 8 handler sites (Tasks split: handlers here, cron in Task 7)
- **ACTION**: At sites 1–8, after the enclosing `run_transaction(...).await?` returns (txn committed, before `Ok(Json(...))`), call `governance_case_after_transition(&context, &case, Some(old_status), new_status).await.ok();`. Derive `old_status` from the in-memory case row loaded pre-transition (it retains the pre-update value); derive `new_status` from the literal set in the `.set(...)` (or the outcome for the submit_jury_vote if/else). For site 7 (`admin_close_case.rs`), capture `let old_status = case.status;` BEFORE the `try_from` move at :67.
- **IMPLEMENT**: site 8 is in `api_crud` — `lemmy_api_crud` already depends on `lemmy_api_utils`, so import the hook fn there too. For `submit_jury_vote` (sites 2–6), fire once per handler invocation with the actually-reached new status (the `outcome` already encodes which branch fired); do not fire for non-deciding votes.
- **MIRROR**: `notify.rs:285-306` (off-txn `.ok()` invocation).
- **GOTCHA**: NEVER fire inside `run_transaction` (it does HTTP I/O; would hold the conn + risk the txn). The fire point is strictly after `.await?`.
- **GOTCHA**: the hook must be reachable — confirm `governance_case_after_transition` is `pub` and the import path resolves from both `lemmy_api` and `lemmy_api_crud`.
- **VALIDATE**: `cargo check -p lemmy_api --features full && cargo check -p lemmy_api_crud --features full && cargo clippy -p lemmy_api -p lemmy_api_crud --features full --no-deps -- -D warnings`

### Task 7: WIRE the hook at the 3 cron sites + the api_crud loop site (site 12)
- **ACTION**: Site 9 (`appeal_window_expiry.rs`): collect `(case_id, old_status, new_status)` tuples into a `Vec` inside the batch loop; after the loop completes (conn free), iterate and fire `governance_case_after_transition(...).ok()` per tuple. Sites 10/11 (`sponsor_liability_grace.rs::fire_or_escape_case_inner`): fire after each per-case `run_transaction(...).await` resolves, deriving new status from the `PerCaseOutcome` (Fired→`SponsorLiabilityFired`, Escaped→`SponsorLiabilityEscaped`). **Site 12 (`revoke_endorsement.rs`, api_crud)**: `process_revocation` loops over `pending_cases` and may escape several; the escapes happen *inside* the `run_transaction` (:142). Accumulate the escaped `(case_id, SponsorLiabilityPending, SponsorLiabilityEscaped)` tuples (e.g. return them in `outcome`), then fire the hooks **after** the `run_transaction(...).await?` returns at the handler level — never inside the txn.
- **IMPLEMENT**: the cron fns run under the scheduler (`crates/routes/src/utils/scheduled_tasks.rs`); `context` is available. Do NOT fire inside the loop while holding `conn` (sites 9, 12) — accumulate then fire. For sites 10/11, the per-case `case`/`re_loaded` snapshot (status == `SponsorLiabilityPending`) is the old status. For site 12, `lemmy_api_crud` already depends on `lemmy_api_utils` (Task 6 import works there).
- **MIRROR**: site-9/10/11/12 commit-boundary notes in this plan's Mandatory Reading; the accumulate-then-fire shape of site 9.
- **GOTCHA**: site 12's escapes are accumulated inside the txn closure but fired only after commit — thread the escaped-case list out via the closure's return (`outcome`), since `process_revocation` already returns an outcome struct. Do NOT fire from inside `process_revocation`.
- **GOTCHA**: a batch/revocation may transition many cases; firing N fire-and-forget POSTs is fine (each `.ok()`-swallowed). Do not let one bridge-down POST abort the batch — `.ok()` guarantees this.
- **VALIDATE**: `cargo check -p lemmy_api -p lemmy_api_crud --features full && cargo clippy -p lemmy_api -p lemmy_api_crud --features full --no-deps -- -D warnings`

### Task 8: ADD e2e tests (hook fires / suppression / append-wrapper / no-content-hash)
- **ACTION**: In `crates/server/tests/e2e/governance.rs` (an existing `include!`d file), add: (a) `m2_transition_hook_fires_when_messaging_enabled` — seed `messaging_enabled=true`, stand up a mock HTTP listener on the bridge port (or assert via a test seam), drive `admin_assign_jury`, assert a `case_transition` payload was POSTed with correct `old_status`/`new_status`; (b) `m2_transition_hook_suppressed_when_disabled` — default `messaging_enabled=false`, drive the same transition, assert NO POST; (c) `m2_append_room_event_writes_chain_entry` — call `append_room_event(pool, ENTRY_KIND_ROOM_CREATED, …)`, assert one `governance_log WHERE entry_kind = 'room_created'` row with intact `prev_hash`/`signature`; (d) `m2_append_room_event_rejects_non_room_kind` — assert `append_room_event(pool, "report_created", …)` errors.
- **IMPLEMENT**: return `lemmy_utils::error::LemmyResult<()>`; use `governance_fixtures::bootstrap()`; mirror the transition-drive in `submit_jury_vote_severe_panel_meets_threshold`. For the POST-assertion, mirror `governance_events_notify_fires` (governance.rs:3840) — since `bridge_notify` POSTs to `localhost:9009` rather than emitting PG NOTIFY, stand up a one-shot mock server (or override `BRIDGE_NOTIFY_URL` via a test seam if one is added). If no clean seam exists, prefer asserting the suppression path (b) + the wrapper paths (c,d) which are deterministic, and raise a `kind: log` DQ proposing a test-seam for the POST-fires assertion.
- **MIRROR**: `jury_mechanics.rs:955-1100`, `governance.rs:3840-3973`, `e2e.rs:2223-2256` (`include!` wiring — do NOT create a bare `tests/*.rs`).
- **GOTCHA**: e2e requires Docker (testcontainers `pgautoupgrade:18-alpine`); tests run `--test-threads=1` (env-mutation safety). `GOVERNANCE_LOG_SIGNING_KEY` is set by `bootstrap()`.
- **GOTCHA**: per `feedback_lemmy_error_no_std_error.md` — bridge non-`LemmyError` errors use `.map_err(|e| anyhow::anyhow!("{e}"))?`. Pre-locate verbatim `old_string` anchors before editing `governance.rs` (it's large; per `feedback_fix_impl_pre_locate_e2e_anchors.md`).
- **VALIDATE**: `cargo test --test e2e -p lemmy_server m2_` (the 4 new tests pass)

### Task 9: Clippy + workspace gate + doc-drift follow-up DQ
- **ACTION**: Run the full workspace gate. Raise a `kind: log` DQ noting the two PRD-flagged doc-drift follow-ups for a later docs pass: (1) `04-data-model-and-api.md` should reflect the `governance_messaging_config` table + the new Room::* entry kinds; (2) `06-security-and-threat-model.md` §2.2/§7 should extend the plane-boundary wording + add threat rows for the bridge/room seam (ADR-004 amendment + ADR-016). These are docs in the sibling design-doc repo — out of this code plan's scope, flagged for the user.
- **VALIDATE**: `cargo check --workspace --features full && cargo clippy --workspace --features full -- -D warnings && cargo test --test e2e -p lemmy_server`

---

## Testing Strategy

Per [IMPLEMENTATION-PLAN-v0.md §5](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): integration-first. New tests live in an `include!`d e2e file.

### Tests to Add

| Test Name | What It Validates |
|---|---|
| `m2_transition_hook_fires_when_messaging_enabled` | hook POSTs `case_transition` payload with correct old/new status (or suppression-only if no POST seam) |
| `m2_transition_hook_suppressed_when_disabled` | `messaging_enabled=false` → zero POST, clean v0 posture (Success Criterion: governance flow unchanged) |
| `m2_append_room_event_writes_chain_entry` | `append_room_event` writes a `room_*` entry with intact chain + signature |
| `m2_append_room_event_rejects_non_room_kind` | wrapper refuses any non-Room::* kind (ADR-008 integrity gate) |

### Edge Cases

- [ ] `messaging_enabled=false` → no hook side-effect; existing governance e2e tests pass unchanged.
- [ ] Hash chain still holds after `append_room_event` (`governance_log_hash_chain_holds` still green).
- [ ] `append_room_event` rejects non-room kinds.
- [ ] `submit_jury_vote` fires the hook exactly once with the reached new status (not on non-deciding votes; not twice for the SL-pending/decided branches).
- [ ] `admin_close_case` captures `old_status` despite the `try_from` move.
- [ ] cron batch (site 9) fires per transitioned case, never inside the loop's held conn.
- [ ] `matrix_room_id` in `RoomEventPayload` survives `scrub_json` (or transform is documented + tested).

---

## Validation Commands

> Wrapping cargo: impl-task writes a `validate-pending-laptop` DQ with these commands verbatim (incl. `--features full`) and STOPS; the laptop advisor runs them. Per CLAUDE.md "NO CARGO ON ELITEDESK".

### Level 1: STATIC_ANALYSIS
```bash
cargo check --workspace --features full
cargo clippy --workspace --features full -- -D warnings
```
**EXPECT**: Exit 0, zero errors, zero warnings.

### Level 2: INTEGRATION_TESTS
```bash
cargo test --test e2e -p lemmy_server m2_
```
**EXPECT**: the 4 new tests pass (Docker required; first run pulls `pgautoupgrade:18-alpine`).

### Level 3: FULL_E2E_REGRESSION
```bash
cargo test --test e2e -p lemmy_server
```
**EXPECT**: full governance suite green — proves observer-only (no transition behaviour changed) + clean-posture (messaging disabled by default).

### Level 4: REGISTRY_CONSISTENCY (no schema change — entry_kind is TEXT)
```bash
rg -c "^pub const ENTRY_KIND_" crates/db_schema/src/source/governance/governance_log.rs   # == 65
# collision check per .claude/rules/governance-log-entry-kind-registry.md:
#   Count A (^pub const ENTRY_KIND_) == Count B (unique snake_case literals); `sort|uniq -d` empty
```
**EXPECT**: 65 consts; shim count matches; no duplicate string literals.

### Level 5: CROSS_CUTTING_VERIFICATION
- [ ] `append_room_event` is the only new path to the log and rejects non-Room::* kinds.
- [ ] No raw `person_id`/username/email written by the hook payload or `RoomEventPayload` (ints + opaque tokens only; `scrub_json` runs inside `append`).
- [ ] `governance_log_hash_chain_holds` still passes.
- [ ] The hook fires strictly AFTER `run_transaction` commit at every site (grep each site).

### Level 6: MANUAL_VALIDATION
- `rg "governance_case_after_transition" crates/` — confirm 11 production fire sites + 1 definition.
- `rg "ENTRY_KIND_ROOM_" crates/db_schema crates/api/api` — confirm 10 defs + 10 shim re-exports.

---

## Acceptance Criteria

- [ ] `governance_case_after_transition` fires at all 11 verified transition sites, after txn commit, gated by `messaging_enabled`.
- [ ] 10 `ENTRY_KIND_ROOM_*` consts defined, shimmed (alphabetical), and in the registry doc; total 65; collision check clean.
- [ ] `append_room_event` wrapper validates Room::* kinds and writes via `append` (scrub + chain + sign intact).
- [ ] Level 1–3 pass exit 0.
- [ ] `messaging_enabled=false` → zero room provisioning side-effect (clean posture preserved).
- [ ] No new clippy warnings.
- [ ] No contradiction with ADR-004/008/013/015/016; no `CaseStatus` variant added; no transition logic changed.
- [ ] Workspace pulls zero Matrix deps (`services/bridge` stays in `exclude`).

---

## Completion Checklist

- [ ] Tasks completed in dependency order (0 → 9).
- [ ] Each task validated immediately (Level 1 after each).
- [ ] Level 1: `cargo check --workspace --features full` + clippy pass.
- [ ] Level 2/3: e2e tests pass.
- [ ] Level 4: registry consistency (65; no collision).
- [ ] Level 5: cross-cutting verification passes.
- [ ] Doc-drift follow-up DQ raised (04 + 06 sibling-repo docs).
- [ ] All acceptance criteria met.

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| A transition site is missed (retrofitting emission is error-prone — research §8.5 warning) | MED | MED | Task 0 re-enumerates against HEAD via `rg`; the 11-site table is verified 2026-06-05; cron sites called out explicitly. |
| Hook fired inside `run_transaction` holds conn / risks txn (it does HTTP I/O) | MED | HIGH | Fire strictly after `.await?`; Level 5 greps each site; `.ok()` swallows bridge-down. |
| `submit_jury_vote` double-fires (SL-pending vs decided branches) | MED | LOW | Fire once per handler from `outcome`; edge-case test asserts single fire. |
| `admin_close_case` loses `old_status` to the `try_from` move | LOW | LOW | Capture `let old = case.status;` (Copy) before line 67; called out in Task 6. |
| `matrix_room_id` redacted by `scrub_json` | MED | MED | Test round-trip; store as opaque non-URL token if it matches the regex; documented in Task 5. |
| Cross-crate hook reachability (`api_crud` site 8, cron in `routes`) | LOW | MED | Hook fn in `api_utils` (all three crates depend on it); confirmed via Cargo.toml grep. |
| No clean test seam to assert the POST *fires* (bridge URL hardcoded) | MED | LOW | Suppression + wrapper tests are deterministic; raise `kind: log` DQ proposing a `BRIDGE_NOTIFY_URL` test seam rather than blocking. |
| Upstream rebase moves transition line numbers | LOW | LOW | Task 0 reconciles by `rg` pattern, not line number; plan cites fn names + patterns. |

---

## Notes

- **Scope decisions (user, 2026-06-05):** (1) in-binary slice only — Phases 1+2+4-binary-side, no `services/bridge/` daemon, no Matrix; (2) `append_room_event` is a reusable library wrapper with NO HTTP route — the bridge's HTTP ingress + shared-secret auth is deferred to the bridge-side plan when its consumer exists (avoids designing a log-write auth surface before the caller).
- **The "~8 sites" in the PRD is an undercount** — exploration found 11, and **Task 0 (run 2026-06-05) found a 12th: `revoke_endorsement.rs:308`** (SL-escape-on-revoke, api_crud), MISSED by exploration. Total production transition statements = **12** (8 handler + 3 cron + 1 api_crud-loop; 13 with the latent `fire_or_escape_case`). `submit_jury_vote.rs` alone holds 5. DQ `a3d0e9941441-050`. This validated the plan's own Task-0 gate — the single highest-risk correctness item.
- **Identity-policy validator extension is deferred** — the M1 validator covers `jury*`/`appeal*` scopes only and does NOT auto-extend to room scopes. Room-scope pinning belongs with the bridge-side room-config work that introduces those scopes; noted in NOT Building.
- **OQ-009 graduated reveal** has a concrete spec (handles revealed to fellow jurors only at ≥1 posted comment, default-1 admin-configurable threshold) — but it's **bridge-side membership-rendering logic**, not in this in-binary slice. The `ENTRY_KIND_ROOM_IDENTITY_REVEALED` const is registered here for when the bridge emits it.
- **Doc drift:** `04` claims no `governance_messaging_config` table (it exists, M1-b) and lacks the Room::* kinds; `06 §2.2/§7` needs the ADR-004-amendment plane wording + ADR-016 threat rows. These are sibling-repo design docs — flagged as a follow-up DQ (Task 9), not edited from this fork (CODE WINS policy).
