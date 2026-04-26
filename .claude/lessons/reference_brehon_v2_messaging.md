---
name: Brehon V2 messaging research (Matrix + LiveKit + bridge)
description: Research report at brehon-fork/docs/brehon-law-inspired-network/V2/messaging.md covers governance-triggered group chat, 1:1 DMs, and RTC town halls as V2a/V2b/V2c. Post-v0 scope with four v0 hooks to preserve. Read when V2 is scheduled or when a Phase 5/6 change risks closing one of the v0 doors.
type: reference
originSessionId: 50cf5c6f-6b81-45b0-b3b4-45ac46b352d8
---
V2 messaging scope is researched and parked. The report lives at:

`C:\Users\barri\Developer\brehon-fork\docs\brehon-law-inspired-network\V2\messaging.md`

**What it covers (V2, not v0):**

- Group chat and 1:1 DMs via a Rust-authored Matrix appservice bridge to Synapse; Lemmy communities map 1:1 to Matrix rooms; Lemmy PMs mirror via the `federated_private_message_after_receive` plugin hook.
- Real-time A/V via MatrixRTC (LiveKit SFU + LiveKit JWT service + Element Call frontend). Apache-2.0 + AGPL-3.0; no licence conflicts with the fork.
- **Governance-triggered rooms only** — no ad-hoc chat. Every room has a governance owner (case, event, appeal, emergency).
- Three-phase roll-out: V2a = bridge + DMs + admin panel; V2b = auto-provisioned rooms on governance state transitions + hash-chained room lifecycle events; V2c = town halls with mic-passing + emergency mute + recording-as-artefact.
- Per-instance feature flag — a Brehon instance can run governance-only (v0 scope) without Matrix, LiveKit, or the bridge.
- Identity policy two-tier: `jury` and `appeals` pinned to `always_pseudonym` (ADR-015 group-property requirement); other room types admin-configurable (`pseudonym_opt_in` default).
- Config scope: per-community policy with instance-wide default layer.
- Cross-instance cases use Matrix-federated rooms; each instance writes its own hash-chain entries — no canonical chain.

**Four v0 hooks that V2 depends on (report §8):**

1. **§8.1 PM plugin hooks stay stable** — `local_private_message_{before,after}_{create,update}`, `federated_private_message_{before,after}_receive`, `private_message_report_after_create`, `plugin_hook_notification`. Any phase touching `crates/api/api_crud/src/private_message/` or `crates/apub/objects/src/objects/private_message.rs` must keep these call sites.
2. **§8.2 Reserve MXID-looking handles** — don't block `@_`-prefixed usernames at registration; leave room for a `matrix:id` property on actor extensions.
3. **§8.3 Prefer SSE over WebSocket** — if Phase 5 or 6 adds real-time transport for governance notifications, pick SSE.
4. **§8.4 Governance state transitions must be hookable events** — V2b's room-provisioning service needs `CaseStatus::*` transitions, jury membership changes, appeal filed, scheduled event open/close emitted as subscribable events (Extism hook, webhook, or Postgres NOTIFY). Not as implicit side-effects of API handlers. Also reserve a `messaging_*` column prefix in the governance_config table so V2 can extend without a schema war.

**Resolved V2 OQs (all 2026-04-17):** OQ-V2-01 identity two-tier, OQ-V2-02 per-community config, OQ-V2-03 Matrix-federated cross-instance rooms, OQ-V2-05 dual-sourced chair role, OQ-V2-06 federation-wide mute-all, OQ-V2-07 instance-local RTC cost. OQ-V2-04 (recording storage) parked for V2c sub-PRD.

**When to read this memory:**

- User says "messaging", "group chat", "town hall", "Matrix", "LiveKit", "V2", or asks about post-v0 scope
- A Phase 5/6 plan proposes to remove or rename a PM plugin hook
- A Phase 5/6 plan proposes a real-time transport for notifications
- A Phase 5/6 plan implements a `CaseStatus` transition or jury membership change — verify the transition is emitted as a hookable event, not just a side-effect
- Any work touches `crates/api/api_crud/src/private_message/`, `crates/apub/objects/src/objects/private_message.rs`, or actor-extension schema
- V2 is scheduled and needs a sub-PRD — route to `/prp-prd` using this report as predecessor research

**When not to read:** v0 Phase 1–6 implementation work that doesn't touch the four preservation hooks. V2 is explicitly out of scope until v0 ships and v1 is planned.
