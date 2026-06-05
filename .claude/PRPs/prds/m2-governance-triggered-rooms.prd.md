# Sub-PRD: M2 — Governance-Triggered Rooms

**Parent**: `.claude/PRPs/prds/v2-messaging-rtc.prd.md` (umbrella) — this sub-PRD is the
detailed spec for the second cluster (M2, formerly V2b/C2).
**Predecessor**: `.claude/PRPs/prds/m1-chat-infrastructure.prd.md` (M1 — chat infrastructure, SHIPPED 2026-06-04, PRs #177 + #179).
**Created**: 2026-06-05
**Status**: DRAFT — ready for `/prp-plan` once §Blocking-Dependencies are satisfied (M2-core has none; M2-late is gated).
**Scope**: M-track milestone (post-v1). Supplements the numbered design docs. Does NOT replace any ADR.
**Naming**: "M2" per ADR-016 (renamed from V2b to end the ADR-010 v2-security naming collision). The umbrella PRD body still uses V2a/V2b/V2c in places; M2 = V2b = governance-triggered rooms.

> **Scope decision (user, 2026-06-05): ROOMS-FIRST.** M2 splits into two phases:
> - **M2-core** — governance-triggered room provisioning + the 10 lifecycle `Room::*`
>   hash-chain entries + the `governance_case_after_transition` hook. This is the
>   first **real** exercise of the ADR-016 backplane seam (the hook is the B-side
>   integration point) and needs NO sanction publish, so it ships WITHOUT resolving
>   OQ-ADR016-02 / OQ-ADR016-04.
> - **M2-late** — B-publish sanction propagation + B-actor portable-ID linkage.
>   GATED on OQ-ADR016-02 (B-publish event schema) + OQ-ADR016-04 (sanction
>   translation semantics). A later `/prp-plan` phase, authored when those OQs
>   resolve.
>
> Room provisioning is governance-metadata emission, not sanction propagation —
> the split is clean. See §Implementation Phases.

---

## Problem Statement

A jury of 5 assigned to a case has no shared deliberation room; an appeal has no panel channel; an `emergency_remove` admin action has no coordination space. M1 shipped the chat plane (bridge + Tuwunel + 1:1 DM + admin config) but deliberately wired NO governance triggers — every `CaseStatus` transition in the Brehon binary today is a bare `update(moderation_case::table)` with zero observable side-effect (confirmed: no `governance_case_after_transition` hook exists; only PM-scoped `plugin_hook_notification` does). M2 makes governance moments *provision rooms automatically*, and records each room's metadata lifecycle on the append-only hash chain — without ever hashing room content.

**Actor(s) affected** ([02 §2](../../docs/brehon-law-inspired-network/02-domain-model.md)): **Juror Eligible / Juror** (primary — the ADR-015 pinned-pseudonym jury room is the load-bearing capability), **Appeals panel member**, **Instance admin** (emergency coordination + lifecycle-policy config), **Trusted Member** (community-event rooms).

---

## Evidence

- **Predecessor research** — `docs/brehon-law-inspired-network/V2/messaging.md` §3.4.2 (Cluster C2) documents the seven room scenarios (C2.1 jury room … C2.7 metadata-only hashing) + the §8.5 corollary that v0/phase-5+ "should add a `governance_case_after_transition` hook in the same style" as `plugin_hook_notification`. **That hook was never added** — confirmed by codebase exploration 2026-06-05 (zero `plugin_hook_*` calls in any `crates/api/api/src/governance/*.rs`). M2 must add it.
- **M1 readiness signal** — the four primitives M2 builds on are all shipped + on `governance-v0`: the hash-chain `append()` writer (`crates/db_schema/src/source/governance/governance_log.rs:239-314`); `bridge_notify::notify_if_enabled` fire-and-forget HTTP contract (`crates/api/api_utils/src/bridge_notify.rs:13-49`); the `governance_messaging_config` KV table (`migrations/2026-06-03-000000-0000_add_governance_messaging_config/`); the ADR-015 identity-policy validator (`crates/api/api/src/governance/messaging_config.rs:69-83`) + pseudonym allocator (`crates/api/api/src/governance/actor_pseudonym_helper.rs:26-80`).
- **OQ leans documented** — OQ-009 (juror anonymity → graduated mutual visibility, resolved 2026-06-01), OQ-V2-08 (Brehon↔Brehon only, resolved 2026-06-03), OQ-V2-10 (Tuwunel, resolved 2026-06-01) are all settled. The two ADR-016 OQs that gate sanction work are deferred to M2-late.

---

## ADRs That Govern This

| ADR | Summary | How it constrains us |
|---|---|---|
| [ADR-004](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (amended 2026-05-23) | Governance plane separated from content plane; extended with federated app planes | M2 adds the **third plane** (chat/RTC). The within-Brehon two-plane invariant is unchanged; the bridge is not in-process and does not share Brehon's permission boundary. Room metadata may cross into `governance_log`; room **content** may not. [06 §2.2](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) wording must be extended at plan time. |
| [ADR-008](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Append-only signed governance log | The 10 M2-core `Room::*` entries go on the chain via the existing `append()` writer (sha2 chain + ed25519 sig, Postgres-trigger-driven). **Content inside rooms is NEVER hashed** (research §3.4.2 C2.7) — only lifecycle metadata. Zero-migration: `entry_kind` is TEXT, so new `ENTRY_KIND_ROOM_*` consts only. |
| [ADR-013](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Illegal-content emergency-remove | M2 provisions an **emergency room within 2s** on `emergency_remove` invocation (C2.5). No change to the `EmergencyRemove` enum or the existing handler — M2 adds a transition-hook consumer only. Emergency room = all instance admins + configured legal-contact MXID; reported party NEVER invited. |
| [ADR-015](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Pseudonymised actor IDs | **Load-bearing.** Jury + appeals rooms pinned `always_pseudonym` (group-property argument: one juror's opt-out degrades the whole panel). The M1 validator already structurally rejects non-pseudonymous `identity_policy` on `jury*`/`appeal*` scopes — M2 inherits it. Jurors rendered `Juror-<suffix>`, never Lemmy usernames. Reveal logic per OQ-009. |
| [ADR-016](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Brehon is a cross-app governance backplane (B-fetch / B-publish / B-actor) | M2 is the **first reference integration**. M2-core exercises the seam via the `governance_case_after_transition` hook → bridge notification (the B-side integration point). M2-late adds B-publish (sanction events) + B-actor (portable-ID linkage). B-fetch (evidence) is not in M2 scope. |
| [ADR-014](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Federation interop with vanilla Lemmy | Per OQ-V2-08 (resolved): messaging plane is **Brehon↔Brehon only**. Cross-instance jury rooms (jurors on instances A+B) use standard Matrix federation, NOT AP; vanilla peers see nothing of the messaging plane. |
| [ADR-010](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Staged releases v0→v1→v2→v3 | M2 messaging is **parallel to** ADR-010's v2-security track, not gated on it (the M1/M2/M3 rename exists to prevent this collision). |
| [ADR-011](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | AGPLv3 inherited | Bridge stays AGPL-3.0 (in-house). Tuwunel (Apache-2.0) compatible. M1 Tree C already added the AGPL-NOTICE bridge+Tuwunel lines. |

**Contradiction check**: **None found.** All sourced ADRs are either directly implemented (008, 013, 015) or remain compatible (004 amendment present, 016 forward-compatible, 014 via OQ-V2-08). Codebase exploration 2026-06-05 confirmed zero contradictions.

---

## Open Questions This Touches

| OQ | Status | Impact on this sub-PRD |
|---|---|---|
| [OQ-ADR016-02](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (B-publish event schema + subscriber contract) | **OPEN — blocks M2-late only** | M2-core (room provisioning) emits NO sanction events → not blocked. M2-late (sanction propagation) is gated: resolve before the M2-late `/prp-plan`. Lean: webhook delivery; universal event schema with `sanction_kind`; at-least-once idempotency. |
| [OQ-ADR016-04](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (sanction translation semantics per app) | **OPEN — blocks M2-late only** | Same split. M2-late needs the minimum primitive set (`prevent_post`, `mute_voice`, `hide_content`, `restrict_reach`) codified so the Matrix subscriber knows how to translate a sanction into room power-level changes. |
| [OQ-ADR016-01](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (B-fetch SPI) | OPEN — does NOT block M2 | M2 builds no evidence-fetch adapter. First-blocks whichever M-phase / app integration introduces B-fetch. |
| [OQ-ADR016-03](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (B-actor link-flow UX) | OPEN — soft gate for M2-late | M2-core uses bridge-local puppet IDs (inherited from M1). M2-late portable-ID linkage needs the link-flow; resolve before M2-late if it includes B-actor. |
| [OQ-009](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (juror anonymity in decision phase) | **RESOLVED 2026-06-01** | Graduated mutual visibility: `Juror-<suffix>` revealed to fellow jurors only when they enter a room where discussion is already in progress (≥1 posted comment, admin-configurable threshold, default 1). `always_pseudonym` pin non-configurable. M2 jury-room membership-rendering logic has a concrete spec. |
| [OQ-V2-08](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (vanilla interop) | **RESOLVED 2026-06-03** | Brehon↔Brehon only. Cross-instance rooms via Matrix federation, not AP. |
| [OQ-V2-10](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (Matrix homeserver) | **RESOLVED 2026-06-01** | Tuwunel (already deployed by M1). M2 inherits the homeserver. |
| [OQ-005](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (juror notification UX) | 99-register; soft gate | M2 introduces a Matrix-invite notification vector. Ideal: OQ-005 resolves before M2 commits to how Matrix invites render. Soft, not hard. |

---

## Proposed Solution

M2-core adds **one governance-binary hook** and **one out-of-binary consumer**:

1. **In the Brehon binary** — a `governance_case_after_transition(case, old_status, new_status)` notification fired at every `CaseStatus` transition commit point (~8 sites: `admin_assign_jury.rs`, `submit_jury_vote.rs` ×3, `admin_emergency_remove.rs`, `request_appeal.rs`, `appeal_window_expiry.rs`), mirroring the existing PM-scoped `plugin_hook_notification` style. The hook extends the M1 `bridge_notify` fire-and-forget HTTP contract with a discriminated-union payload (PM event | case-transition event), gated by the same `messaging_enabled` config. **Brehon never calls Matrix** — it only emits the transition signal.

2. **In `services/bridge/`** (the M1 daemon, extended) — a **room-provisioning service** that receives case-transition notifications, provisions/archives Matrix rooms per the C2.1–C2.6 rules (jury / community-event / spin-out / appeal / emergency / membership-mirror), pins `always_pseudonym` for jury+appeal rooms, renders `Juror-<suffix>` with OQ-009 graduated reveal, and — crucially — **calls back into the Brehon binary's governance-log `append()` path** to write the 10 `Room::*` metadata entries onto the hash chain. Idempotency by `case_id` + last-seen log row id survives bridge restart (no duplicate `Room::Created`).

The 10 M2-core `Room::*` entry kinds are a **zero-migration** addition (`entry_kind` is TEXT). The `governance_messaging_config` KV table needs no schema change — room lifecycle/identity policies are new config *rows* (`(scope='jury_rooms', key='lifecycle_policy', …)`). A new **`bridge_room` mapping table** (case_id ↔ matrix_room_id ↔ lifecycle state) is the one greenfield schema addition, and it lives bridge-side (the bridge owns its own store; the Brehon binary stays free of room state per ADR-004 plane separation).

---

## Key Hypothesis

> We believe **a log-tailing / hook-driven room-provisioning service** will **auto-provision a correctly-scoped, correctly-pseudonymised governance room within 5s of the triggering case transition (within 2s for emergency_remove), and record its lifecycle as `Room::*` hash-chain entries** for **jurors, appeals panels, and instance admins**.
>
> We'll know we're right when: a case entering `JurySelection` auto-provisions a jury room with exactly the 5 assigned jurors as `Juror-<suffix>` puppets (no reporter, no reported party, no admin) within 5s; an `emergency_remove` provisions an emergency room within 2s; closing the case applies the configured lifecycle policy observably in Matrix; the corresponding `Room::*` entries land on the hash chain with correct schema; and a bridge restart mid-case resumes provisioning with **no duplicate `Room::Created`**.

---

## What We're NOT Building

- **B-publish sanction propagation** — deferred to **M2-late** (gated on OQ-ADR016-02 + OQ-ADR016-04). Room provisioning needs no sanction publish.
- **B-actor portable-ID linkage** — M2-core uses M1's bridge-local puppet map. Portable IDs are M2-late (or later), gated on OQ-ADR016-03.
- **B-fetch evidence adapter** — not in the messaging track's M2; first-blocks a future evidence-fetch phase (OQ-ADR016-01).
- **Town halls / MatrixRTC / LiveKit / Element Call / recording-as-artefact** — that's **M3** (`ENTRY_KIND_ROOM_RECORDING_UPLOADED` is registered for completeness but NOT emitted in M2; recording storage is OQ-V2-04, M3-gated).
- **Hashing room content** — only the 10 lifecycle metadata entries are hashed; speech inside rooms is never on the chain (research §3.4.2 C2.7, GDPR/ADR-015 consistency).
- **Vanilla-Lemmy interop for the messaging plane** — Brehon↔Brehon only (OQ-V2-08).
- **A new `CaseStatus` variant or any change to existing governance handlers' decision logic** — M2 only *observes* transitions; it never alters them.

---

## Success Criteria (verifiable)

| Criterion | How Verified |
|---|---|
| `CaseStatus::JurySelection` transition → jury room provisioned in <5s with exactly the 5 assigned jurors, no reporter/reported/admin | Integration test (`services/bridge/tests/`) hooking `admin_assign_jury` via the transition notification; assert room membership + identity_policy=always_pseudonym |
| `emergency_remove` → emergency room in <2s with admins + legal-contact MXID, reported party absent | Integration test on `admin_emergency_remove` transition path |
| 10 `Room::*` entries land on the hash chain with correct schema after one full case lifecycle | Read `governance_log WHERE entry_kind LIKE 'room_%'`; assert one per lifecycle stage + chain integrity (prev_hash links) |
| Bridge restart mid-case resumes provisioning with no duplicate `Room::Created` | Integration test with bridge restart; assert single `room_created` per `case_id` |
| Jury-room `always_pseudonym` cannot be admin-overridden | `cargo test` on the M1 identity-policy validator extended for room scopes; assert reject |
| OQ-009 graduated reveal: juror handles visible to fellow jurors only after ≥1 posted comment | Integration test on room-membership rendering at threshold boundary |
| `messaging_enabled = false` → zero room provisioning, zero `Room::*` entries (clean posture preserved) | `cargo test --test e2e` governance flow passes unchanged with messaging disabled |
| New `ENTRY_KIND_ROOM_*` consts registered + shimmed + in the registry doc | `cargo check --workspace`; registry-doc grep; entry-kind collision check |
| No new `cargo clippy --workspace -- -D warnings` failures | clippy gate |

---

## Cross-Cutting Impact ([IMPLEMENTATION-PLAN-v0.md §4](../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md))

- [x] **Hash-chain governance log touched?** YES — 10 new `Room::*` metadata entry kinds (zero-migration; TEXT `entry_kind`). Content never hashed. Uses the existing `append()` writer unchanged.
- [x] **`actor_pseudonym` table or redaction service touched?** YES (read-only) — M2 calls `actor_pseudonym_helper::get_or_create` for juror rendering; payloads pass through `scrub_json` (existing ADR-015 path). No schema change.
- [x] **`CaseStatus::EmergencyRemove` affected?** YES (observer only) — M2 provisions an emergency room on the transition; NO change to the enum or the `admin_emergency_remove` handler logic.
- [ ] **AGPLv3 notice / source disclosure affected?** No new change — M1 Tree C already added the bridge+Tuwunel AGPL-NOTICE lines.
- [x] **New hook in the Brehon binary** — `governance_case_after_transition` at ~8 transition sites. This is the one invasive in-binary change; every site is a commit-point notification, never a logic change.

---

## Users & Context

**Primary actor**: **Juror** ([02 §2](../../docs/brehon-law-inspired-network/02-domain-model.md)).
- **Current behaviour**: a 5-juror panel assigned to a case has no shared channel; deliberation happens nowhere or out-of-band.
- **Trigger**: case enters `CaseStatus::JurySelection` (jury assigned).
- **Success state**: within 5s, a Matrix room exists with the 5 jurors as `Juror-<suffix>` pseudonymous puppets, where text deliberation can happen, with all lifecycle events (created, members joined, closed/archived) hashed into the governance log — and with fellow-juror handles revealed only once discussion begins (OQ-009).

**Non-actors**: the reported party is **never** invited to the jury room (research §3.4.2 C2.1 step 4). Reporters, admins, and visitors are **never** invited to jury rooms. (Emergency rooms DO include admins; appeal rooms include the appealing party + original reporter + appeals panel, but NOT the original jury — stage separation.)

---

## Technical Approach

**Feasibility**: **HIGH** — ~80% of the infrastructure shipped in M1 and is on `governance-v0`. The hash-chain writer, bridge HTTP contract, config KV table, identity-policy validator, and pseudonym allocator are all production-tested. M2 is primarily **wiring** (the transition hook + ~8 call sites) + **const registration** (10 `ENTRY_KIND_ROOM_*`) + **bridge-side provisioning logic** + **one greenfield bridge table** (`bridge_room`). No core schema migration, no hashing/signature change.

**Crate(s) affected**:
- `crates/api/api_utils/src/` — new `governance_case_after_transition` notification fn (mirror `notify.rs` / `plugins.rs` style); extend `bridge_notify.rs` payload to a discriminated union.
- `crates/api/api/src/governance/` — fire the hook at the ~8 `CaseStatus` transition sites (`admin_assign_jury.rs`, `submit_jury_vote.rs`, `admin_emergency_remove.rs`, `request_appeal.rs`, + the `appeal_window_expiry` scheduled job). Observer-only; no decision-logic change.
- `crates/db_schema/src/source/governance/governance_log.rs` — add 10 `ENTRY_KIND_ROOM_*` consts (canonical location since Phase 6 DQ-6.6).
- `crates/api/api/src/governance/governance_log.rs` — re-export the new consts via the `pub use` shim.
- `.claude/rules/governance-log-entry-kind-registry.md` — new M2 section listing the 10 kinds + collision check.
- **`services/bridge/`** (out-of-workspace, M1 daemon extended) — room-provisioning service, `bridge_room` table + migration (bridge-local store), OQ-009 reveal logic, idempotent provisioning by `case_id`.

**Architecture fit** ([03 §4](../../docs/brehon-law-inspired-network/03-architecture.md)): the chat/RTC plane is the third plane (ADR-004 amendment). The room-provisioning service lives in `services/bridge/` **outside** the Brehon workspace (`cargo build --workspace` still pulls zero Matrix deps — M1's story-6 invariant holds). The Brehon binary emits transition signals and owns the hash-chain `append()`; the bridge owns room state. The bridge writes `Room::*` entries by calling back into the binary's governance-log path (the integration seam), never by writing `governance_log` directly.

**New dependencies**: none in the Brehon workspace (the invariant: zero Matrix deps in `cargo build --workspace`). Bridge-side additions (Matrix room-management APIs) live in `services/bridge/Cargo.toml`, already excluded.

**Technical risks**:

| Risk | Likelihood | Mitigation |
|---|---|---|
| Log-tailing / hook delivery loses ordering on bridge restart → duplicate `Room::Created` | Medium | Bridge tracks last-seen governance_log row id + idempotent room creation keyed by `case_id`. Acceptance criterion tests restart-mid-case. |
| The ~8 transition-hook sites are easy to miss one (retrofitting event emission is error-prone — the research §8.5 warning) | Medium | Enumerate ALL `CaseStatus`-mutating sites first (`rg` for `update(moderation_case`); the plan's task-0 audits every state-mutating path before wiring. Per `feedback_fix_impl_enumerate_all_callsites.md`. |
| `governance_case_after_transition` payload leaks PII into the bridge | Low | Payload carries `case_id` + status + pseudonymous actor refs only; passes through `scrub_json` (existing ADR-015 path). |
| Cross-instance jury room (jurors on A+B) Matrix-federation reconciliation with the hash chain | Medium | Research §3.6 hash-chain × federation rule; cross-instance is a single acceptance test; can be deferred to an M2 sub-phase if it expands scope. |
| Doc 04 drift — design doc claims no `governance_messaging_config` table (it exists, M1-b shipped it) | Low | Doc-drift follow-up: update 04 to reflect the M1 + M2 schema. CODE WINS. |

---

## Implementation Phases (for follow-up `/prp-plan` runs)

<!--
  STATUS: pending | in-progress | complete
  PRP: link to generated plan file once /prp-plan runs on this
-->

| # | Phase | Description | Status | Depends | PRP Plan |
|---|---|---|---|---|---|
| 1 | **M2-core hook** | `governance_case_after_transition` notification + ~8 transition-site wiring + extend `bridge_notify` payload | pending | M1 | - |
| 2 | **M2-core entry kinds** | 10 `ENTRY_KIND_ROOM_*` consts + shim re-export + registry-doc section | pending | - | - |
| 3 | **M2-core provisioning** | `services/bridge/` room-provisioning service (jury/appeal/emergency/event/spin-out) + `bridge_room` table + OQ-009 reveal + idempotency | pending | 1, 2 | - |
| 4 | **M2-core hash-chain emission** | bridge calls back into `append()` to write the 10 `Room::*` entries; restart-idempotency | pending | 2, 3 | - |
| 5 | **M2-core e2e + clean-posture** | integration tests (jury <5s, emergency <2s, 10 entries, restart-no-dup, `messaging_enabled=false` clean) | pending | 1-4 | - |
| 6 | **M2-late B-publish** (GATED) | sanction event publish + Matrix subscriber translation | pending | OQ-ADR016-02, OQ-ADR016-04 | - |
| 7 | **M2-late B-actor** (GATED, optional) | portable-ID linkage replacing bridge-local puppet map | pending | OQ-ADR016-03 | - |

### Phase Details

**Phase 1: M2-core hook**
- **Goal**: a `governance_case_after_transition` notification fires at every `CaseStatus` transition commit point.
- **Scope**: new notification fn in `crates/api/api_utils/` mirroring `plugin_hook_notification`; fire at the ~8 enumerated sites; extend `bridge_notify` to a discriminated-union payload gated by `messaging_enabled`.
- **Success signal**: `cargo check --workspace`; a unit test asserts the hook fires on a synthetic transition; `messaging_enabled=false` suppresses it.

**Phase 2: M2-core entry kinds**
- **Goal**: 10 `Room::*` entry kinds registered.
- **Scope**: consts in `governance_log.rs` + shim re-export + registry-doc M2 section + collision check.
- **Success signal**: `cargo check --workspace`; registry grep; no collision.

**Phase 3: M2-core provisioning** (bridge-side)
- **Goal**: room provisioning for C2.1–C2.6 scenarios.
- **Scope**: `services/bridge/` provisioning service; `bridge_room` table + migration; `always_pseudonym` pin for jury/appeal; OQ-009 graduated reveal; idempotent-by-`case_id`.
- **Success signal**: bridge integration test provisions a jury room with correct membership.

**Phase 4: M2-core hash-chain emission**
- **Goal**: `Room::*` entries land on the chain.
- **Scope**: bridge calls back into the binary's `append()`; restart-idempotency via last-seen row id.
- **Success signal**: `governance_log WHERE entry_kind LIKE 'room_%'` shows correct entries; restart test → no dup.

**Phase 5: M2-core e2e**
- **Goal**: full acceptance.
- **Scope**: the §Success Criteria integration tests, including clean-posture.
- **Success signal**: all §Success Criteria pass.

**Phase 6 (M2-late, GATED): B-publish** — author only after OQ-ADR016-02 + OQ-ADR016-04 resolve.

**Phase 7 (M2-late, GATED, optional): B-actor** — author only if portable IDs are in M2-late scope; gated on OQ-ADR016-03.

---

## Blocking Dependencies

- **M2-core (Phases 1–5)**: NONE beyond M1 (shipped). OQ-009, OQ-V2-08, OQ-V2-10 all resolved. Ready for `/prp-plan` now.
- **M2-late (Phases 6–7)**: **OQ-ADR016-02** (B-publish schema) + **OQ-ADR016-04** (sanction translation) must resolve before the M2-late `/prp-plan`. OQ-ADR016-03 (B-actor link-flow) gates Phase 7 if included.

---

## Decisions Log

| Decision | Choice | Alternatives | Rationale |
|---|---|---|---|
| M2 backplane scope | **Rooms-first; defer B-publish/B-actor to M2-late** | (a) full M2 resolving both OQs now; (b) rooms-only, all backplane to a post-M3 milestone | User 2026-06-05. Room provisioning is governance-metadata emission, needs no sanction publish — clean split. Ships M2-core without resolving OQ-ADR016-02/-04; still honours ADR-016 "first integration" via the transition hook + Room::* metadata seam. |
| Where room state lives | **`bridge_room` table in `services/bridge/`** (bridge-local store) | core governance schema in `crates/db_schema/` | ADR-004 plane separation: the Brehon binary stays free of room state; the bridge owns its own store. Preserves M1's zero-Matrix-deps-in-workspace invariant. |
| How `Room::*` entries reach the chain | **Bridge calls back into the binary's `append()` path** | bridge writes `governance_log` directly | Hash-chain integrity (sha2 + ed25519 + Postgres trigger) must stay in the binary; the bridge is not a governance-plane writer. The callback is the integration seam. |
| `governance_messaging_config` change | **New config rows, no schema change** | add `lifecycle_policy`/`identity_policy` columns | The table is a typed KV store (`(scope, key, value_*)`); room policies are rows, not columns. M1 designed it this way. |
| New entry kinds | **10 zero-migration TEXT consts** | a `room_event` enum / new table | `entry_kind` is TEXT (not a Diesel enum); the established pattern (55 existing kinds) is const-string + shim + registry. |

---

## Research Summary

**Codebase findings** (Explore agents, 2026-06-05):
- `crates/api/api_utils/src/plugins.rs:48-64` — `plugin_hook_notification`, PM-scoped only; the style to mirror.
- **No `governance_case_after_transition` hook exists** — zero `plugin_hook_*` calls in any `crates/api/api/src/governance/*.rs`. ~8 bare `update(moderation_case::table)` transition sites (`admin_assign_jury.rs`; `submit_jury_vote.rs:503,513,1020+`; `admin_emergency_remove.rs`; `appeal_window_expiry.rs`). **This is M2's primary in-binary work.**
- `crates/api/api_utils/src/bridge_notify.rs:13-49` — `notify_if_enabled`, fire-and-forget POST to the bridge, gated by `messaging_enabled`; ad-hoc PM payload → extend to a discriminated union.
- `crates/db_schema/src/source/governance/governance_log.rs:239-314` — `append(pool, entry_kind, payload, actor_pseudonym)`; 55 existing `ENTRY_KIND_*` consts at 112-234; `scrub_json` at 268. Production-ready for new kinds.
- `governance_messaging_config` (migration `2026-06-03-…`; model `…/governance_messaging_config.rs`) — typed KV table; seed rows `messaging_enabled=false`, `identity_policy=pseudonymous`.
- `crates/api/api/src/governance/messaging_config.rs:69-83` — `validate_identity_policy` already rejects non-`pseudonymous` on `jury*`/`appeal*` scopes (ADR-015). `actor_pseudonym_helper.rs:26-80` — `get_or_create` idempotent UUID allocator.

**Design-doc alignment**:
- `99` ADR-008 — "every write to [governance tables] must emit a log entry before the user response returns"; room metadata qualifies; content does not.
- `99` ADR-013 — emergency-remove → post-facto jury; M2 provisions the emergency room.
- `99` ADR-015 + OQ-009 — `always_pseudonym` group-property pin (non-configurable) + graduated mutual visibility reveal spec.
- `99` ADR-016 — B-fetch/B-publish/B-actor full text; M2 = first reference integration; M2-core exercises the seam, M2-late adds B-publish.
- `04 §2` — `governance_log` is a TEXT-`entry_kind` hash chain; `Room::*` is zero-migration. `CaseStatus` variants M2 keys off: `JurySelection`, `Appealed`, `EmergencyRemove`, `Decided`, `Closed`.
- `04` **drift flagged** — doc claims no `governance_messaging_config` table; it exists (M1-b). Follow-up: update 04 + 06 §2.2/§7 (plane boundary + threat rows for bridge/room/B-fetch/B-publish/B-actor) at plan time.

---

*Generated: 2026-06-05*
*Status: DRAFT — M2-core ready for `/prp-plan`; M2-late gated on OQ-ADR016-02 + OQ-ADR016-04.*
