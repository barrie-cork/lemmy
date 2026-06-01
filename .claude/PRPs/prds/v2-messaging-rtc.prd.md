# Sub-PRD: V2 Messaging + Real-Time Communication

**Scope**: Forward-looking capability track — supplements the numbered design docs. Does NOT replace any ADR. Additive to `docs/brehon-law-inspired-network/V2/messaging.md` (the predecessor research report).
**Created**: 2026-04-19
**Status**: DRAFT
**Scheduled**: **TBD** — no gate on v0 / v1 / v2-security completion per user decision 2026-04-19; trigger is operator-demand signal once v0 ships, contingent on §Blocking-Dependencies.
**Naming note**: "V2" in this document refers to the messaging/RTC capability track (C1/C2/C3). It is **distinct from ADR-010's v2-security milestone**. The two should not be conflated — neither gates the other.

---

## Problem Statement

A Brehon fork in operation will need real-time communication for four concrete governance moments — jury deliberation, appeals, community town halls, and emergency coordination — that the content-plane alone cannot serve. Lemmy has no group chat, no real-time transport, and no audio/video at any layer; the `to: [ObjectId; 1]` fixed-size array in `crates/apub/objects/src/protocol/private_message.rs:19-33` is a hard block. A jury of 5 assigned to a case has no shared room; an emergency_remove admin action has no coordination channel; a scheduled community event has no live broadcast.

V2 adds a **parallel chat/RTC plane** via an existing federated chat protocol (Matrix) bridged into the Brehon fork, with governance-triggered rooms whose metadata-only lifecycle is hashed into the governance log. The chat plane is **optional, feature-flagged per instance**, and governance-triggered — no ad-hoc rooms.

**Primary actor** ([02 §2](../../docs/brehon-law-inspired-network/02-domain-model.md)): **Juror Eligible / Juror**. The ADR-015 pinned-pseudonym jury room is the load-bearing new capability. If the group-property pseudonymity argument from research §3.8 works for jurors, the rest of the V2 architecture (admin-configurable rooms, town halls, emergency coordination) is downstream commodity work.

**Secondary actors**: Instance admin (feature-flag enablement, identity/lifecycle policy configuration), Trusted Member (town-hall participation, 1:1 DM, rich media), Appeals panel member (pseudonymity-pinned per ADR-015).

---

## Evidence

**Predecessor research doc** — `docs/brehon-law-inspired-network/V2/messaging.md` §1–§11 (dated 2026-04-17) documents the originating questions, TL;DR, per-cluster user stories, reconciliation with ADR-015 (§3.8), the hash-chain × GDPR rule (§3.6), the four v0 hooks V2 depends on (§8), and the resolutions of OQ-V2-01…07 reached during the writing session.

**v0 readiness signal** — all seven PM plugin hooks V2 depends on fire at verifiable call sites today (see §Research-Summary). Greenfield confirmed: zero Matrix/LiveKit/WebRTC/WebSocket/SSE/Synapse/mautrix/Conduit dependencies anywhere in the workspace.

**OQ-V2-01…07 all resolved on 2026-04-17** per predecessor research §3.7 — identity-policy two-tier model (OQ-V2-01), per-community config with instance-default (OQ-V2-02), Matrix-federated cross-instance rooms (OQ-V2-03), recording storage deferred (OQ-V2-04), dual-sourced chair role (OQ-V2-05), room-global mute-all (OQ-V2-06), instance-local RTC cost model (OQ-V2-07).

---

## ADRs That Govern This

| ADR | Summary | How it constrains us |
|---|---|---|
| [ADR-001](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Fork Lemmy as base platform | V2 is additive to the fork. Bridge runs as separate service, not Lemmy-binary changes. Must not break upstream-rebase discipline. |
| [ADR-004](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Governance plane separated from content plane | V2 adds a **third plane** (chat/RTC). [06 §2.2](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) plane boundary wording must be extended at V2a schedule time. |
| [ADR-006](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Remote sanctions advisory-only in MVP | Parallel principle: no inbound Matrix message auto-applies governance — commits cannot write to `governance_log` except via governance handlers or the bridge-side `Room::*` metadata writer. |
| [ADR-008](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Append-only signed governance log | V2b `Room::Created`, `Room::MembershipChanged`, `Room::Archived`, `Room::Tombstoned`, `Room::ChairTransferred`, `Room::ChairOverride`, `Room::MuteAll`, `Room::RecordingUploaded`, `Room::EmergencyCreated`, `Room::SpunOut`, `Room::LifecycleApplied` entries go on the chain. **Content inside rooms is NEVER hashed** — deliberate scope (research §3.4.2 C2.7). |
| [ADR-010](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | v0 → v1 → v2 → v3 staged releases | V2 messaging is **parallel to** this track, not gated on it. ADR-010 v2 = security hardening; the messaging V2 is unrelated. PRD consistently uses "V2 messaging" or "V2a/V2b/V2c" to avoid collision. |
| [ADR-011](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | AGPLv3 inherited | LiveKit (Apache-2.0): compatible. Element Call (AGPL-3.0): same licence, no drift. Matrix homeserver (Synapse AGPL / Conduit Apache / Dendrite Apache): all acceptable. Bridge written in-house stays AGPL-3.0 per fork norm. |
| [ADR-012](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Base Lemmy 1.0-beta with Extism plugin system | The Extism plugin hooks (research §4.4, §8.1) are V2's primary integration seam. All seven PM hooks verified present (see §Research-Summary). |
| [ADR-013](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Illegal-content emergency-remove | V2b C2.5 provisions an emergency room within 2s on `emergency_remove` invocation. Depends on the hookable state-transition event per §Blocking-Dependencies. |
| [ADR-014](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Federation interop with vanilla Lemmy | Governance signals are fork-only. V2 messaging follows the same principle (see [OQ-V2-08](#open-questions-carried-forward)) but the exact stance is unresolved and left as an open question per user decision 2026-04-19. |
| [ADR-015](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Pseudonymised actor IDs | **Load-bearing.** Research §3.8 is the full reconciliation. Jury and appeals rooms are pinned to `always_pseudonym`; admin config panel cannot override; amendment requires a new ADR superseding ADR-015. This is a **group-property** argument: one juror's opt-out would degrade pseudonymity for the entire panel. |

**Contradiction check**: None found. Research §3.8's reconciliation with ADR-015 is deliberate and documented as an audit trail. No ADR needs superseding for this PRD to execute.

---

## Open Questions Carried Forward

| OQ | Status | Impact on this sub-PRD |
|---|---|---|
| **OQ-V2-04** (recording storage) | Parked for V2c sub-PRD per research §3.7 | PRD defers to a V2c-specific sub-PRD when V2c is scheduled. Candidates when decided: separate MinIO/S3 store, pict-rs extension, or Matrix media repo with hash-chained URL. |
| **OQ-V2-08** (new) — Vanilla-Lemmy interop for V2 messaging | Opened here; unresolved | Two readings: (a) V2 is Brehon↔Brehon only, consistent with ADR-014 governance-signals-fork-only; (b) degraded-mode interop (e.g. `Announce` mirrored post "there was a town hall" visible to vanilla peers). User decision 2026-04-19: leave as OQ, do not commit. Resolution required before V2a-integration-test phase. |
| **OQ-V2-09** (new) — Rollback story for V2 disable-after-enable | Partially resolved; ops mechanics deferred to V2a sub-PRD. **Data contract** (resolved): chat history preserved on Matrix side; governance_log refs preserved. **Ops lean (2026-06-01):** **soft pause** — `messaging_enabled = false` stops the bridge process but leaves the homeserver running; rooms remain accessible and the flag is reversible. Hard disable (homeserver decommission) is a separate, explicit decommission story, not the day-to-day toggle. V2a sub-PRD owns: bridge shutdown signal + in-flight room handling + active-jury-room-on-disable policy + media-store and GDPR artefact cleanup runbook. |
| **OQ-V2-10** (new) — Matrix homeserver choice | ✅ Resolved 2026-06-01 — **Tuwunel** (Rust/RocksDB, MSC4143 v1.4.6+); Synapse named fallback. Both AS API blockers (#465, #219) confirmed fixed. See `99-decisions-and-open-questions.md` OQ-V2-10. V2a sub-PRD ops section must document containerised deployment topology (non-host-network bridge traffic is not loopback — needs explicit handling). | No longer blocks V2a start. |
| [OQ-005](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (juror notification UX) | 99-register; v1 scope | V2 introduces a new notification vector (Matrix). Ideal order: OQ-005 resolves (which notification channels exist) before V2a commits to how Matrix invites render. Soft gate, not hard. |
| [OQ-009](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (juror anonymity in decision phase) | ✅ Resolved 2026-06-01 — graduated mutual visibility: `Juror-<suffix>` handles revealed to fellow jurors only when they enter a room where discussion is already in progress (≥1 posted comment, admin-configurable threshold). `always_pseudonym` pin remains non-configurable. See `99-decisions-and-open-questions.md` OQ-009. | V2b jury room membership rendering logic now has a concrete spec. |
| [OQ-018](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (admin config write endpoint) | 99-register; v1 scope | V2's identity/lifecycle policy tables (`governance_messaging_config`) must land through the same endpoint when OQ-018 ships. PRD requires that OQ-018's endpoint supports the `messaging_*` prefix reserved in research §8.4 corollary. |
| [OQ-019](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (participation_consistency events) | 99-register; v1 scope | Low-intersection. Only relevant if V2c recording-as-governance-artefact produces reputation events. Currently research §3.4.3 C3.6 does not emit reputation side-effects; non-blocking for V2. |

---

## Proposed Solution

Deploy a **Matrix homeserver + Brehon↔Matrix bridge + (optional) LiveKit RTC stack** alongside the Brehon server, with the bridge consuming the governance log via log-tailing to provision rooms on case-state transitions. Bridge is a **separate Rust daemon** (Matrix appservice spec), not an in-process Extism plugin. Crate layout follows [07 §1.2](../../docs/brehon-law-inspired-network/07-operations-and-federation.md) external-services pattern (same posture as the governance log signer), not [03 §7](../../docs/brehon-law-inspired-network/03-architecture.md) in-process crate layout.

V2 ships as three sequential clusters per research §3.4:

- **V2a (C1 Chat infrastructure)** — bridge process, community Matrix rooms (manually provisioned), 1:1 DMs, rich media, admin config panel.
- **V2b (C2 Governance-triggered rooms)** — room-provisioning service that log-tails `governance_log` and creates/archives rooms on case-state transitions. Metadata-only hash-chain entries.
- **V2c (C3 Town hall with mic-passing)** — MatrixRTC stage-mode, chair controls, raised-hand queue, recording-as-governance-artefact.

Each cluster is independently shippable with its own integration-test acceptance signal. Each cluster's sub-PRD (V2a.prd.md, V2b.prd.md, V2c.prd.md) will be generated when that cluster is scheduled; this PRD is the umbrella.

---

## Key Hypotheses (per cluster, per Q3=B decision)

### H1 (C1 / V2a)

> We believe **the mautrix-style application-service bridge pattern can be adapted to the Lemmy 1.0-beta fork using only the existing Extism plugin hooks**, with no modifications to the Lemmy binary's PM path, for text + image + voice note 1:1 messaging.
>
> We'll know we're right when: a Brehon user on instance A DMs a user on instance B via the bridge — text + image + voice note — with round-trip latency **under 3 seconds**, and the admin-panel identity-policy change on a non-pinned room type persists across restart.

### H2 (C2 / V2b)

> We believe **a log-tailing consumer against `governance_log` is a sufficient event source for room lifecycle management**, obviating the need for plugin-hook events on case state transitions, provided `CaseStatus::JurySelection` (not the research-doc-hypothesised `JuryDeliberation`) is adopted as the deliberation-start signal.
>
> We'll know we're right when: a case entering `CaseStatus::JurySelection` auto-provisions a jury room **within 5s** with the 5 assigned jurors as members, `always_pseudonym` identity policy pinned by ADR-015, and six `Room::*` entries on the governance hash chain; the bridge process can survive a restart and resume provisioning without duplicate `Room::Created` entries.

### H3 (C3 / V2c)

> We believe **the MatrixRTC stack (LiveKit + JWT service + Element Call) supports chair-controlled stage-mode with mic-passing and cross-instance emergency-mute** at single-event operator cost commensurate with the Matrix federation model already accepted in OQ-V2-07.
>
> We'll know we're right when: a scheduled town hall opens in stage mode, chair promotes/demotes four different users in sequence within a single session without manual intervention, recording lands as an MP4 with a hash-chain entry pointing at it, and emergency-mute drops all publishers — **including cross-instance participants per OQ-V2-06** — within 500ms.

---

## What We're NOT Building

From the research doc §3.1 (explicitly out of V2 scope; PRD repeats verbatim):

- Ad-hoc user-initiated rooms ("start a chat with a friend outside any governance context" is not a V2 feature).
- Community-moderator-initiated rooms outside the governance flow (deferred to V3 if demand exists).
- E2EE of governance chat content against the Brehon operator (per ADR-015, operators are already trusted).
- Federating Matrix rooms through ActivityPub (doesn't exist as a spec).
- Voice notes as Lemmy PM content (Lemmy PM schema stays text-only; rich media lives Matrix-side).

**PRD-level deferrals (new, per user Q7 decision 2026-04-19):**

- **Data-export from deleted rooms.** A `hard_delete_after_days(n)` room that has been deleted must NOT leave exportable transcripts on the Brehon instance. Export capability lives in a V2 successor sub-PRD if ever.
- **Matrix homeserver reverse proxying via the Brehon server.** The Brehon binary must NOT proxy Matrix traffic; this preserves the "Brehon binary has no Matrix deps" property permanently. Deployment is side-by-side via separate DNS / reverse proxy.
- **Bridge-as-library consumed by the Brehon binary.** Research §5.1 Option B (in-process bridge via Extism) is **explicitly rejected** — V2 commits to the daemon shape (Option A) only.

**And anything deferred to v1/v2/v3 per [ADR-010](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)** that intersects with messaging stays out of this PRD.

---

## Success Criteria (verifiable — per cluster)

| Cluster | Criterion | How Verified |
|---|---|---|
| V2a | 1:1 DM text+image+voice round-trip < 3s | Integration test under `bridge/tests/dm_round_trip.rs` (crate TBD) |
| V2a | Admin panel identity-policy change persists across restart | Integration test; restart fixture via docker-compose in deploy runbook |
| V2a | `messaging_enabled = false` preserves clean v0 governance-only posture | `cargo test --test e2e` governance flow passes unchanged |
| V2b | `CaseStatus::JurySelection` transition → jury room provisioned in <5s | Integration test hooking into `admin_assign_jury` via existing `governance_log` tail |
| V2b | Six `Room::*` entries land on hash chain with correct schema | Integration test reading `governance_log WHERE entry_kind LIKE 'room_%'` after one case lifecycle |
| V2b | Bridge restart resumes provisioning without duplicate `Room::Created` | Integration test with bridge restart mid-case |
| V2b | `emergency_remove` provisions emergency room in <2s | Integration test on `admin_emergency_remove` handler |
| V2c | Stage-mode town hall: chair sequences 4 mic-passes without intervention | LiveKit integration test + Element Call E2E (external test rig) |
| V2c | Recording lands as MP4 with hash-chain entry | Check pict-rs / media-store for artefact; query `governance_log WHERE entry_kind = 'room_recording_uploaded'` |
| V2c | Emergency-mute drops all publishers in <500ms including cross-instance | Multi-instance test rig; timing measured at publisher client |

---

## Cross-Cutting Impact

Adapted from the v0 template to V2-relevant dimensions:

- [x] **Hash-chain governance log touched?** YES — new `ENTRY_KIND_ROOM_*` constants added to `crates\api\api\src\governance\governance_log.rs`. Zero migration per §Research-Summary Q4 (entry_kind is TEXT, not Diesel enum). All room lifecycle metadata hashed; **room content NEVER hashed**.
- [x] **`actor_pseudonym` table or redaction service touched?** YES — jury and appeals rooms use `actor_pseudonym` mapping for room membership rendering (`Juror-<pseudonym-suffix>`). Redaction service scope is unchanged — redaction writes **into** the governance log; V2 does not route chat content into the log, so no new redaction responsibility.
- [x] **`CaseStatus::EmergencyRemove` affected?** YES — V2b C2.5 provisions an emergency room on `emergency_remove` invocation. No change to the enum or existing handler; V2b adds a new log-tailing consumer only.
- [x] **AGPLv3 notice / source disclosure affected?** Minor — the AGPL notice must be extended to mention the bridge and (if shipped) the LiveKit / Element Call / JWT-service AGPL components. No licence change to any existing component.

**V2-specific cross-cutting dimensions (new):**

- [x] **Plane boundary extension** — [06 §2.2](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) plane-separation wording references content plane and governance plane. V2 adds a chat/RTC plane. Wording must be extended at V2a schedule time (not at PRD-write time).
- [x] **Threat-model table extension** — [06 §7](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) has 20 rows; zero cover bridge/SFU/room-artefact threats. V2a sub-PRD adds rows for: (a) bridge process compromise, (b) LiveKit SFU compromise, (c) hostile Matrix homeserver payload, (d) E2EE key mishandling in bridge, (e) recording artefact leak via pict-rs path confusion.
- [x] **Federation trust model extension** — [06 §5 line 328](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) states "no inbound message auto-applies a local sanction in MVP." V2 must add: "no inbound Matrix message writes to `governance_log` except via the bridge's own structural metadata writer, which writes Room::* entry kinds only."
- [x] **Federation-disabled posture extension** — [07 §5](../../docs/brehon-law-inspired-network/07-operations-and-federation.md) has no rows for federation-disabled / interop-degraded modes. V2's `messaging_enabled = false` posture needs a parallel row (see OQ-V2-09).

---

## Users & Context

**Primary actor**: Juror (or Juror Eligible) — see [02 §2](../../docs/brehon-law-inspired-network/02-domain-model.md) for full definition.

- **Current behaviour (today, v0)**: Juror is assigned to a case, sees the case in their assignments list (`GET /api/v4/governance/jury/me`), can read the case record, can submit a vote. Has **no in-platform channel to deliberate with other jurors on the same case** — jurors are assigned but not connected.
- **Trigger**: Case transitions to `CaseStatus::JurySelection` (the actual v0 transition name; research doc used `JuryDeliberation` which doesn't exist in the enum — see §Research-Summary Q3).
- **Success state**: Within 5s of the transition, a Matrix room exists with the 5 jurors as members, rendered as pseudonyms (`Juror-<suffix>`), where text and (V2c) voice deliberation can happen, with all lifecycle events (room created, members joined, chair actions, room closed) hashed into the governance log.

**Non-actors**: The reported party is **never** invited to the jury room (research §3.4.2 C2.1 step 4). Reporters are **never** invited. Admins are **never** invited. Visitors are **never** invited. This is a deliberate stage-separation.

**Secondary actors:**
- **Instance admin** — configures `messaging_enabled`, `rtc_enabled`, identity-policy panel, per-community lifecycle policy. Sees and operates the bridge ops.
- **Trusted Member** — participates in community town halls (V2c), 1:1 DMs (V2a), community-event rooms (V2b).
- **Appeals panel member** — same pseudonymity-pinned treatment as juror per ADR-015.

---

## Technical Approach

**Feasibility**: **HIGH** — three concrete reasons from §Research-Summary:

1. All seven PM plugin hooks already fire at verifiable call sites with zero core Lemmy changes required.
2. `governance_log.entry_kind` is a TEXT column (not a Diesel enum), so `Room::*` entries are a **zero-migration** addition — new `pub const ENTRY_KIND_ROOM_*` strings only.
3. Greenfield confirmed: zero Matrix / LiveKit / WebRTC / WebSocket / SSE / Synapse / mautrix / Conduit dependencies anywhere in the workspace. No transitive deps to dislodge.

**Crate(s) / services affected:**

- **NEW**: `services/bridge/` (greenfield Rust crate OR separate repo — decision deferred to V2a sub-PRD). Lives **outside** the Brehon workspace per research §5.1 Option A. Does NOT become a workspace member. Follows [07 §1.2](../../docs/brehon-law-inspired-network/07-operations-and-federation.md) external-services pattern: separate host, separate credentials, no DB access except read-only log-tail, network-isolated.
- **MODIFIED (additive)**: `crates/api/api/src/governance/governance_log.rs` — add `ENTRY_KIND_ROOM_*` const strings (13 of them per research §3.4 room event set). No signature change, no migration.
- **MODIFIED (additive, possibly)**: `crates/apub/objects/src/protocol/person.rs` and `crates/apub/objects/src/objects/person.rs:114, 175` — extend `Person` AP protocol struct with an optional `brehon_matrix_room_advertisement` field OR reuse the existing `matrix_user_id` field. Decision deferred to V2a sub-PRD. **V2 would be the first Brehon-added actor-extension on the wire** (`membership_state` is stripped per `crates/apub/objects/src/objects/person.rs:177-181`).
- **UNTOUCHED**: `crates/api/api_crud/src/private_message/*.rs` (hooks already fire), `crates/db_schema/*` (governance_log schema supports new variants via TEXT), `crates/api/routes/*` (bridge does not register Brehon routes — it's a separate HTTP service).

**New external services**:
- Matrix homeserver (Synapse / Conduit / Dendrite — OQ-V2-10).
- LiveKit Server (V2c only).
- LiveKit JWT service (V2c only).
- Element Call front-end (V2c only; shipped to users via separate URL).

**Architecture fit** ([03 §4](../../docs/brehon-law-inspired-network/03-architecture.md) plane separation, extended):

```
┌─────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│  Content plane  │     │ Governance plane │     │ Chat/RTC plane   │
│  (Lemmy core)   │     │ (crates/api/     │     │ (bridge + Matrix │
│                 │◄───►│  governance)     │◄────┤  + LiveKit)      │
└─────────────────┘     └──────────────────┘     └──────────────────┘
         ▲                      ▲                          ▲
         │                      │                          │
         └──── Writes to public_case_log, governance_log ───┘
                        (log-tail feeds bridge)
```

Bridge subscribes to governance_log (read-only); content plane unchanged. Bridge never writes back to Lemmy DB — only emits `Room::*` log entries via a narrow governance-log-writer interface.

**New dependencies** (all deferred to V2a sub-PRD for exact version pins):

- **Matrix SDK** for Rust bridge (e.g. `matrix-sdk-appservice` — check licence + maturity at V2a schedule time).
- **Tokio / reqwest** already present.
- **Serde / serde_json** already present.

**Technical risks**:

| Risk | Likelihood | Mitigation |
|---|---|---|
| `CaseStatus::InReview` has no production flip site today — V2 can't use it as deliberation-start signal | Certain (verified) | PRD commits to `CaseStatus::JurySelection` as the signal. Documented in H2 and §Blocking-Dependencies. |
| Log-tailing loses ordering under bridge restart (duplicate `Room::Created`) | Medium | Bridge tracks last-seen log row ID; idempotent room creation by `case_id`. V2b acceptance criterion #3 tests this. |
| Matrix homeserver choice decays between PRD write and V2 schedule (MatrixRTC landscape changes) | Medium | OQ-V2-10 explicitly parks the decision. Re-take at V2a schedule time. |
| ADR-015 group-property pseudonymity pin becomes operationally contested | Low (speculative) | Admin panel validator rejects jury/appeals identity-policy overrides with reference to ADR-015. Amendment requires new ADR superseding ADR-015. |
| Bridge process becomes a new SPOF for governance notifications | Medium | Bridge is OPT-IN (per-instance flag). Brehon instances without messaging run unchanged. Bridge failure does not block governance_log writes or case decisions — only delays room provisioning. |
| LiveKit SFU operational cost at V2c scale surprises the operator | Medium | OQ-V2-07 resolved: instance-local cost model. Document expected cost in V2c delivery plan before rollout. |
| Matrix federation trust model diverges from ActivityPub federation trust model, creating inconsistent "trusted instance" lists | Medium | OQ-V2-08 captures this. V2a/V2b/V2c sub-PRDs each re-evaluate. |
| `matrix_user_id` field already exists at `crates/apub/objects/src/protocol/person.rs:49` — V2 must not collide with the legacy self-declared-handle semantics | Low | V2 either reuses the field (with verified-bridge-routed semantics) or introduces a new field. Decision in V2a sub-PRD. Either way is additive. |

---

## Implementation Phases (for follow-up `/prp-plan` runs)

<!--
  STATUS: pending | in-progress | complete
  PRP: link to generated plan file once /prp-plan runs on this
-->

Per user decision Q10 2026-04-19: **strict sequential — V2a → V2b → V2c.** Each cluster is independently shippable; each has its own integration-test acceptance signal.

| # | Phase | Description | Status | Depends | PRP Plan |
|---|---|---|---|---|---|
| 1 | V2a — Chat infrastructure | Bridge process, community Matrix rooms (manual), 1:1 DMs, rich media, admin config panel | pending | §Blocking-Dependencies | - |
| 2 | V2b — Governance-triggered rooms | Room-provisioning service, log-tailing consumer, automatic case/event/emergency room creation, `Room::*` governance_log entries | pending | V2a complete | - |
| 3 | V2c — Town hall with mic-passing | LiveKit + JWT + Element Call, stage-mode with chair controls, raised-hand queue, recording-as-artefact | pending | V2b complete | - |

### Phase Details

**Phase 1: V2a — Chat infrastructure**

- **Goal**: Prove the bridge architecture works in isolation before wiring governance triggers on top of it.
- **Scope**: Matrix homeserver deployment, bridge daemon (appservice), puppet-on-first-login, 1:1 DM with rich media, manual community Matrix rooms (C1.4 shape), admin config panel with identity/lifecycle policy tables, `messaging_enabled = false` preserves v0-clean posture.
- **Success signal**: H1 integration test passes (cross-instance DM with rich media in <3s).
- **Upstream deps that must be stable at schedule time**: Phase 5c/Phase 6 must not have renamed or removed any of the 7 PM plugin hooks; Extism plugin host integrity preserved.

**Phase 2: V2b — Governance-triggered rooms**

- **Goal**: Prove that a log-tailing consumer can drive room lifecycle for every governance moment (cases, events, emergencies).
- **Scope**: Room-provisioning service, log-tail implementation against `governance_log`, automatic jury-room provisioning on `CaseStatus::JurySelection`, automatic appeal-room provisioning on appeal request, automatic emergency-room provisioning on `admin_emergency_remove`, `Room::*` metadata hash-chain writer, membership mirror on jury rotations.
- **Success signal**: H2 integration test passes (JurySelection → room in 5s, 6 hash-chain entries, bridge-restart idempotence).
- **Upstream deps that must be stable at schedule time**: `governance_log` table shape preserved; `CaseStatus::JurySelection` remains the transition that fires from `admin_assign_jury`; every governance state transition continues to emit a governance_log entry (already true per §Research-Summary Q3).

**Phase 3: V2c — Town hall with mic-passing**

- **Goal**: Prove MatrixRTC stage-mode can deliver town-hall UX at a scale and cost profile a single instance operator can run.
- **Scope**: LiveKit Server + JWT service + Element Call deployment, stage-mode provisioning on scheduled governance events, chair assignment (dual-sourced per OQ-V2-05), raised-hand queue, chair mic-passing with 30s activation grace, chair overrides, Q&A sidebar with pinning, recording-as-governance-artefact (subject to community config), emergency-mute room-global per OQ-V2-06.
- **Success signal**: H3 integration test passes (stage-mode with 4 mic-passes, recording + hash entry, emergency-mute <500ms cross-instance).
- **Upstream deps that must be stable at schedule time**: V2b's room-provisioning service remains the event source; OQ-V2-04 recording storage is resolved before this phase.

---

## Blocking Dependencies

**Hard gates (must be true at V2 schedule time):**

- **PM plugin hooks stable.** All seven hooks (research §4.4 + §8.1) still firing at the call sites listed in §Research-Summary Q1. `.claude/rules/pm-plugin-hooks-stable.md:62, 117` already codifies this rule in the fork — not a new requirement.
- **`governance_log.entry_kind` remains TEXT** (not migrated to a Diesel enum). Phase 5c cosmetic tasks or later refactors must NOT propose tightening this to an enum — the TEXT shape is load-bearing for V2 zero-migration extension.
- **`CaseStatus::JurySelection` still fires from `admin_assign_jury`.** No future phase proposes restructuring the state transition shape such that jury-room-provisioning has no reliable trigger. (`CaseStatus::InReview` being a dead-letter variant is fine; just don't remove `JurySelection` as the actually-firing deliberation-start.)
- **Actor-extension point on `Person` AP protocol struct remains extensible.** `membership_state` stripping precedent at `crates/apub/objects/src/objects/person.rs:177-181` shows the extension mechanism works. No future phase proposes locking the `Person` struct against additive fields.

**Soft gates (advisable, not blocking):**

- **OQ-005 (juror notification UX)** resolved before V2a commits to Matrix-invite rendering.
- **OQ-018 (admin config write endpoint)** shipped so V2's `governance_messaging_config` can land through the same API. Until OQ-018, direct-psql config is acceptable.

**NOT gates:**

- v0 completion.
- v1 completion.
- ADR-010 v2-security-hardening completion.
- Any specific Phase 5 / 6 / 7 Brehon task.

Per user Q2 decision 2026-04-19: V2 messaging is parallel to, not sequential with, any v0/v1/v2-security milestone.

**Triggering condition for scheduling** (from operator-demand reading of user Q2):

- At least one operator running a Brehon instance requests V2 in a way that justifies the ops overhead of standing up a Matrix homeserver. Without that pull signal, V2 stays post-v0 work with no specific date.

---

## Decisions Log

| Decision | Choice | Alternatives | Rationale |
|---|---|---|---|
| Bridge location | Separate Rust daemon outside Brehon binary | (a) In-process Extism plugin; (b) separate Go/Node daemon | Research §5.1 Option A; keeps Brehon binary free of Matrix deps; matches 07 §1.2 external-services pattern. Extism is the right seam for *governance* hooks, not a whole chat-protocol gateway. Rust because the team is in Rust and long-term maintenance lives with us. |
| Event source for V2b | Log-tailing `governance_log` | Plugin hooks on every state transition | Research §Q3 finding: no `plugin_hook_*` fires on case state transitions today; `governance_log::append` IS the emit seam. Log-tailing is zero-change to Brehon code; adding hooks means auditing every state-mutating path. |
| Deliberation-start signal | `CaseStatus::JurySelection` | `CaseStatus::JuryDeliberation` (research-doc nomenclature); `CaseStatus::InReview` | Codebase truth: `JuryDeliberation` doesn't exist in the enum; `InReview` has no production flip site. `JurySelection` fires at `admin_assign_jury.rs:155` — verified. |
| Identity policy for jury rooms | `always_pseudonym` (pinned) | `pseudonym_opt_in` | Research §3.8 group-property argument; ADR-015; one juror's opt-out would degrade pseudonymity for the panel. Amendment requires new ADR. |
| Cross-instance room topology | Matrix-federated (each homeserver hosts the room) | Single canonical instance; third-party relay | OQ-V2-03 resolution 2026-04-17; parallels ActivityPub federation model. |
| Mute-all scope | Room-global, federation-wide | Home-instance only | OQ-V2-06 resolution 2026-04-17; chair authority follows Matrix power-level semantics. |
| RTC cost model | Instance-local | Shared federation pool; relay network | OQ-V2-07 resolution 2026-04-17. |
| Recording storage | Deferred to V2c sub-PRD | Decide now | OQ-V2-04 resolution 2026-04-17 — operational constraints unknown until V2c scheduled. |
| Chair role sourcing | Dual-sourced (governance-plane-initial + Matrix-native-delegate) | Governance-plane only; Matrix-native only | OQ-V2-05 resolution 2026-04-17. |
| V2 scheduling trigger | Operator-demand after v0 (no v1/v2-security gate) | Strict v0→v1 sequential; gate on ADR-010 v2 | User Q2 decision 2026-04-19. |
| Cluster ordering | Strict sequential V2a → V2b → V2c | Interleaved V2a+V2b bundle | User Q10 decision 2026-04-19. Lets V2a smoke-test bridge before layering governance triggers. |
| Vanilla-Lemmy interop | Leave as OQ-V2-08 | Commit to Brehon↔Brehon-only now; commit to degraded-mode now | User Q4 decision 2026-04-19. |
| Matrix homeserver choice | Deferred to OQ-V2-10 | Commit to Synapse now | User Q5 implicit; MatrixRTC support across homeservers evolves, premature commit. |
| Rollback data contract | Chat history preserved, governance_log refs preserved | Full data wipe on disable | User Q5 decision 2026-04-19. Ops mechanics deferred to V2a sub-PRD. |
| Reverse-proxying Matrix via Brehon binary | Explicit NON-goal | Could be ops-convenient | Preserves "Brehon binary has no Matrix deps" property permanently. |
| Bridge-as-library | Rejected explicitly | Could reduce one process | Research §5.1 Option B disadvantages; user Q7 decision 2026-04-19 captures the rejection. |

---

## Research Summary

Condensed from the technical-grounding phase (two Explore agents, 2026-04-19). Not a substitute for reading `docs/brehon-law-inspired-network/V2/messaging.md` §1–§11 — the PRD builds on that research, does not duplicate it.

### Codebase findings (v0 state as of 2026-04-19)

**Q1 — PM plugin hook seams (all 7 verified)**

Dispatcher at `crates\api\api_utils\src\plugins.rs:40-51, 127-149` (generic `plugin_hook_before<T>` / `plugin_hook_after<T>` + specialized `plugin_hook_notification`). All 7 call sites fire:

| Hook name | Call site |
|---|---|
| `local_private_message_before_create` | `crates\api\api_crud\src\private_message\create.rs:75` |
| `local_private_message_after_create` | `crates\api\api_crud\src\private_message\create.rs:77-80` |
| `local_private_message_before_update` | `crates\api\api_crud\src\private_message\update.rs:57` |
| `local_private_message_after_update` | `crates\api\api_crud\src\private_message\update.rs:60` |
| `federated_private_message_before_receive` | `crates\apub\objects\src\objects\private_message.rs:170` |
| `federated_private_message_after_receive` | `crates\apub\objects\src\objects\private_message.rs:173` |
| `private_message_report_after_create` | `crates\api\api\src\reports\private_message_report\create.rs:56-59` |
| `plugin_hook_notification` (PM notify) | `crates\api\api_utils\src\notify.rs:305` |

Stability rule already codified: `.claude/rules/pm-plugin-hooks-stable.md:62, 117`.

**Q2 — Actor-extension surface for matrix identity**

- Person AP struct: `crates\apub\objects\src\protocol\person.rs:27-53` — already has `matrix_user_id: Option<String>` at line 49 (upstream Lemmy legacy field, self-declared handle).
- DB: `crates\db_schema_file\src\schema.rs:893` (`matrix_user_id -> Nullable<Text>`).
- Serialization: `crates\apub\objects\src\objects\person.rs:114, 175`.
- Validation: `crates\utils\src\utils\validation.rs:87-91` (`is_valid_matrix_id`).
- Username regex (`crates\utils\src\utils\validation.rs:39-55`) ALREADY permits `_`-prefixed usernames; V2 `@_matrix_*` reservation is a reserved-list check, not a regex change.
- `membership_state` precedent: `crates\apub\objects\src\objects\person.rs:177-181` — Brehon Phase 5a task 51 strips `membership_state` before AP serialization (TODO(brehon-fork) upstream). **V2 will be the first Brehon-added actor-extension that DOES cross the AP wire.**

**Q3 — Governance state-transition emit seams**

`CaseStatus` enum at `crates\db_schema_file\src\enums.rs:393-408` has 9 variants. Every production flip site already appends to `governance_log`:

| Site | Transition | Log entry kind |
|---|---|---|
| `crates\api\api_crud\src\governance\create_report.rs:154-176` | `Open → ThresholdMet` | `threshold_met` |
| `crates\api\api\src\governance\admin_assign_jury.rs:155` | `* → JurySelection` | `jury_assigned`, `panel_assembled` |
| `crates\api\api\src\governance\accept_jury_assignment.rs:121` | jury `Selected → Accepted` | `jury_accepted` |
| `crates\api\api\src\governance\decline_jury_assignment.rs:108, 152` | jury `* → Declined` + replacement | `jury_declined`, `jury_replacement_selected` |
| `crates\api\api\src\governance\submit_jury_vote.rs:275` | `JurySelection → Decided` | `jury_vote_submitted`, `sanction_created`, `public_log_published` |
| `crates\api\api_crud\src\governance\request_appeal.rs:129` | `Decided → Appealed` | `appeal_requested` |
| `crates\api\api\src\governance\admin_close_case.rs:79` | `* → Closed` | `admin_case_closed` |
| `crates\api\api\src\governance\admin_emergency_remove.rs:154` | `→ EmergencyRemove` (new case) | `jury_assigned`, `emergency_removed` |

**Critical V2 implication**: `CaseStatus::InReview` is a defined variant with NO production flip site (only test code) — research-doc's `JuryDeliberation` terminology does not match the codebase. **V2 commits to `JurySelection` as the deliberation-start trigger** (fires at `admin_assign_jury.rs:155`). `CaseStatus::AdminReview` is similarly dead in v0.

No `plugin_hook_*` fires on state transitions today; log-tailing is the event source.

**Q4 — Hash-chain append surface**

Writer at `crates\api\api\src\governance\governance_log.rs:87-126`. Representative call:

```rust
governance_log::append(
  &mut conn.into(),
  "jury_vote_submitted",
  json!({ "case_id": data.case_id.0, "decision": data.decision, "vote_id": vote_row.id.0 }),
  Some(juror_pseudonym.clone()),
).await?;
```

Schema at `crates\db_schema\src\source\governance\governance_log.rs:23-44` — `entry_kind` is `String` (TEXT column), NOT a Diesel enum. **Adding `Room::*` entry kinds is zero-migration** — new `pub const ENTRY_KIND_ROOM_*` strings in `governance_log.rs` and calls from the bridge's governance-log-writer.

Current const discipline: `crates\api\api\src\governance\governance_log.rs:50-68` has 19 consts; docstring notes Phase 4 call sites still use literal strings (pending Phase 5c cosmetic migration).

`prev_hash` / `entry_hash` / `signature` are trigger-populated per ADR-008.

**Q5 — Greenfield confirmation**

Zero Matrix / LiveKit / WebRTC / WebSocket / SSE / Synapse / mautrix / Conduit dependencies in workspace `Cargo.toml:114-244`. Verified via grep across all Cargo.toml files.

Only surface-level references to Matrix in source: (a) the legacy `matrix_user_id` field plumbing; (b) format validator `is_valid_matrix_id`; (c) localization strings like "Matrix user ID" in `crates/email/translations/frontend/*.json`. **No SDK, no client, no appservice code, no room state machine, no WebRTC transport anywhere.**

### Design-doc findings

**Numbered docs anticipate V2 in exactly one place:**

- `IMPLEMENTATION-PLAN-v0.md:578-580` §6.1 — the one explicit cross-reference: *"If a future phase proposes adding a real-time push channel... default to Server-Sent Events (SSE) over WebSocket... The V2 messaging bridge does not require Brehon's own RT transport — it polls `governance_log` or subscribes via Postgres NOTIFY per SUBSCRIPTIONS.md. See V2/messaging.md §8.3 for the full reasoning."*

`99-decisions-and-open-questions.md:28` references Matrix once as a rejected platform alternative (ADR-001 context). No other hits for Matrix / LiveKit / RTC / realtime in 00..07.

**Schema extensions required:** [04-data-model-and-api.md](../../docs/brehon-law-inspired-network/04-data-model-and-api.md) has no `governance_messaging_config` table, no `Room::*` entry-kind enumeration (entry_kind is not an enum — it's TEXT per Q4), no recording-artefact column. `CaseEvidence` at 04 §3 line 224 is the closest existing shape (storage_key + sha256 + mime_type + visibility) — V2c recording references may parallel it.

**Crate layout room for a bridge:** [03 §7](../../docs/brehon-law-inspired-network/03-architecture.md) does NOT freeze the crate layout. [07 §1.2](../../docs/brehon-law-inspired-network/07-operations-and-federation.md) lines 50-65 explicitly lists external services (Keycloak, OPA, log-signer, federation fetch worker) — a Matrix bridge fits this pattern (separate host, narrow creds, no DB access, network-isolated).

**Security / threat-model baselines V2 must extend:**

- [06 §2.2](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) plane boundary mentions content + governance planes; no chat plane yet.
- [06 §7](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) threat table has 20 rows; **zero** cover bridge/SFU/room-artefact threats. Templates at rows 375-376 (federation inbox, media fetch path) — V2a adds 5 new rows.
- [06 §5 line 328](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md): *"No inbound message auto-applies a local sanction in MVP."* V2 equivalent ("no inbound Matrix message writes to governance_log except via the bridge's structural metadata writer") needs to be added.
- [07 §5](../../docs/brehon-law-inspired-network/07-operations-and-federation.md) federation operator runbook has no "disabled" or "interop-degraded" posture rows. V2 `messaging_enabled = false` posture needs a parallel row — tracked via OQ-V2-09.

**Vision principles bearing on V2** ([01-vision-and-principles.md §4](../../docs/brehon-law-inspired-network/01-vision-and-principles.md) 9 principles):

- Principle 2 (Sureties/sponsor liability): V2 room participation is sponsored-user activity; sponsor liability propagates through misbehaviour in rooms.
- Principle 3 (Kin groups / nested communities): V2 rooms are per-community by construction; this constrains cross-community room visibility.
- Principle 5 (Restitution over punishment): explicitly supports **post-sanction restorative dialogue** in rooms — "keep people inside the system rather than ejecting them." This is a design-positive for keeping V2 rooms accessible to post-sanctioned users (subject to community policy).
- Principle 6 (Public, known laws): V2 room rules must be the community's versioned rule set, not a chat-only rule set.
- Principle 7 (Soft-enforcement ladder): V2 must slot into existing `SanctionAction` variants, NOT introduce chat-specific sanctions.
- Principle 8 (Inter-tribal law / federated trust): V2 must decide whether Matrix-homeserver federation mirrors ActivityPub federation trust, or is a separate trust dimension — contributes to OQ-V2-08.
- Principle 9 (Visible trust signals — "under sanction"): V2 rooms must render the "under sanction" state visibly; presence of sanction propagates into chat.

---

*Generated: 2026-04-19*
*Status: DRAFT — forward-looking. Scheduled: TBD. Review before running `/prp-plan` (at V2 schedule time, not before).*
