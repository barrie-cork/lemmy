---
name: V2 messaging + RTC research doc
description: Pointer to V2/messaging.md — deep-dive requirements for post-v0 Matrix + LiveKit integration. Not active work; load when V2 is discussed or when the user asks "what were we doing about chat/rooms/calls"
type: reference
originSessionId: 61e8410d-f582-458b-b30c-fde9eec7c2db
---
Location: `docs/brehon-law-inspired-network/V2/messaging.md`

This is a decision-grade requirements document for V2 post-v0 work on group chat, 1:1 DMs, and WhatsApp-style A/V calls. Authored 2026-04-17 via a multi-round discussion.

Key commitments captured there (do NOT re-litigate without reading the doc first):

- **Scope:** governance-triggered rooms only. No ad-hoc chat in V2. Every room has a governance owner (case, event, appeal, emergency).
- **Deployment:** V2 is feature-flagged (`messaging_enabled`). Instances without it run clean v0.
- **Stack:** Matrix homeserver (likely Synapse) + Rust appservice bridge (greenfield) + LiveKit SFU + LiveKit JWT service + Element Call.
- **Phasing:** V2a (chat infra) → V2b (governance triggers) → V2c (town hall / RTC).
- **Identity policy:** two-tier. Jury + Appeals rooms are pinned to `always_pseudonym` per ADR-015 (admin panel cannot override). Other room types default `pseudonym_opt_in`. Resolution documented in §3.8 — changing jury pseudonymity requires a new ADR amending ADR-015, not a panel override.
- **Config scope:** per-community with instance-wide default layer.
- **Cross-instance cases:** Matrix-federated rooms. Each instance keeps its own independent hash chain; no canonical shared ledger.
- **Chair role:** dual-sourced — governance plane assigns initial chair, chair can delegate mid-session via Matrix action mirrored to hash chain.
- **Mute-all:** room-global, crosses federation boundary.
- **Recordings:** storage deferred to V2c sub-PRD (OQ-V2-04 is the only remaining open question).
- **Hash chain:** metadata only. Room lifecycle events hashed; individual messages never hashed.
- **RTC cost model:** instance-local — the instance hosting the LiveKit SFU pays for its bandwidth.

v0 hooks preserved for V2 (§8 of the doc):
1. PM plugin hooks stay stable (ADR-012)
2. Don't block `@_`-prefixed usernames at registration
3. Default future RT transport to SSE over WebSocket
4. Emit governance state transitions as subscribable events (new in Phase 5/6)

Next step when V2 is scheduled: run `/prp-prd "V2 messaging + RTC"` against this file. The sub-PRD will turn §3 into a buildable design and resolve OQ-V2-04.
