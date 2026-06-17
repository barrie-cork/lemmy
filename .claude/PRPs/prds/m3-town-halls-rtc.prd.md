# Sub-PRD: M3 — Town Halls + Real-Time Communication

**Parent**: `.claude/PRPs/prds/v2-messaging-rtc.prd.md` (umbrella) — this sub-PRD is the
detailed spec for the third and final cluster (M3, formerly V2c/C3).
**Predecessors**: `.claude/PRPs/prds/m1-chat-infrastructure.prd.md` (M1 — chat infra, SHIPPED 2026-06-04, PRs #177 + #179); `.claude/PRPs/prds/m2-governance-triggered-rooms.prd.md` (M2-core SHIPPED PR #191 2026-06-06; M2-late B-publish + B-actor SHIPPED PRs #196 + #197 2026-06-12/14).
**Created**: 2026-06-17
**Status**: DRAFT — ready for `/prp-plan` once §Blocking-Dependencies are satisfied (none gate M3-core; OQ-V2-04 resolved in this PRD).
**Scope**: M-track milestone (post-v1, completes the V2 messaging arc). Supplements the numbered design docs. Does NOT replace any ADR.
**Naming**: "M3" per ADR-016 (renamed from V2c to end the ADR-010 v2-security naming collision). The umbrella PRD body still uses V2a/V2b/V2c in places; M3 = V2c = town halls + mic-passing RTC.

> **Scope decision (user, 2026-06-15/17): FULL TOWN-HALL RTC, pilot-driven.** M3 = scheduled
> stage-mode town halls with chair controls, FIFO raised-hand queue, mic-passing, chair override,
> Q&A sidebar, federation-wide emergency mute, anonymous town halls, and recording-as-artefact.
> Success is a **real pilot town hall running end-to-end**, not just green integration tests.
> Recording is **evidentiary-store / replay-grade-access** (D5 = Option C) AND **fully optional**
> per the `record_town_halls` flag (user 2026-06-17) — a town hall can run with recording off,
> and a governance-only instance runs with the whole RTC stack off (`rtc_enabled = false`).
> See §Decisions Log D1–D5 + §Implementation Phases.

---

## Problem Statement

A Brehon community has no way to hold a live governance event. A scheduled deliberation, an appeal
hearing with an audience, an emergency coordination broadcast — none have a real-time floor. M1
shipped the chat plane (bridge + Tuwunel + DMs + admin config); M2 shipped governance-triggered
**text** rooms (jury / appeal / emergency) with metadata-only hash-chain lifecycle. M3 adds the
**real-time voice/video layer on top of those rooms**: stage mode where one speaker holds the floor,
a chair who passes the mic down a FIFO queue, an emergency mute that drops every publisher
federation-wide, and — optionally — a recording of the event hashed onto the chain as its
minutes-of-record. M3 never hashes room *content*; only the recording's URL + content hash and the
chair-action lifecycle metadata go on the chain.

**Actor(s) affected** ([02 §2](../../docs/brehon-law-inspired-network/02-domain-model.md)): **Trusted Member / Member** (primary — town-hall attendees + raised-hand speakers), **Juror / Appeals panel member** (anonymous deliberation broadcasts under `always_pseudonym`), **Instance admin** (event scheduling, chair assignment, `rtc_enabled` / `record_town_halls` config, emergency mute), **Chair** (a dual-sourced role — governance-assigned initial holder, delegable mid-session).

---

## Evidence

- **Predecessor research** — `docs/brehon-law-inspired-network/V2/messaging.md` §3.4.3 (Cluster C3) documents the eight town-hall scenarios: C3.1 stage mode + dual-sourced chair, C3.2 raise hand (FIFO), C3.3 mic-passing (30s grace), C3.4 chair override, C3.5 Q&A sidebar, C3.6 recording-as-artefact, C3.7 federation-wide emergency mute, C3.8 anonymous town halls (pseudonym overlay). §3.5 lists the V2c acceptance criteria (mic-pass 4 users in sequence; recording → MP4 + hash entry; emergency mute <500ms). §3.6 is the hash-chain × GDPR rule the recording lifecycle inherits.
- **M2 readiness signal** — the four primitives M3 builds on are all shipped + on `governance-v0`: the governance-triggered room machinery (`services/bridge/` provisioning + `bridge_room` table); the hash-chain `append()` writer with `ENTRY_KIND_ROOM_RECORDING_UPLOADED` **already registered** (`crates/db_schema/src/source/governance/governance_log.rs:243`, registered by M2, **emitted first by M3**); the `always_pseudonym` identity-policy pin + pseudonym allocator (used for anonymous town halls); the `governance_messaging_config` typed-KV table (the home for `rtc_enabled` + `record_town_halls` rows).
- **Greenfield confirmed** — zero LiveKit / Element Call / MatrixRTC / Egress / MinIO / S3-client dependencies anywhere in the workspace or in `services/bridge/` today. M3 introduces the entire RTC stack as new optional services.
- **OQ leans documented** — OQ-V2-05 (chair dual-sourced), OQ-V2-06 (mute-all room-global/federation-wide), OQ-V2-07 (RTC cost instance-local) all resolved 2026-04-17; OQ-V2-08 (Brehon↔Brehon only) resolved 2026-06-03; OQ-V2-10 (Tuwunel, deployed) resolved 2026-06-01. **OQ-V2-04 (recording storage) is the only OQ that gated M3 PRD authorship — resolved in this PRD (see §Decisions Log D5).**

---

## ADRs That Govern This

| ADR | Summary | How it constrains us |
|---|---|---|
| [ADR-004](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (amended 2026-05-23) | Governance plane separated from content plane; extended with federated app planes | M3 extends the **third plane** (chat/RTC) M2 established. The RTC SFU (LiveKit) and recording store (MinIO) are **out-of-binary** services — `cargo build --workspace` still pulls zero RTC deps. The recording **store placement honours plane separation**: a dedicated object store, NOT the content-plane pict-rs (D5 = Option C). Room/recording **metadata** may cross into `governance_log`; room/recording **content** may not. |
| [ADR-008](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Append-only signed governance log | M3 adds **3 new** chair/mute `Room::*` entry kinds + **emits** the M2-registered `ENTRY_KIND_ROOM_RECORDING_UPLOADED`. Each goes on the chain via the existing `append()` writer (sha2 chain + ed25519 sig). **Content inside rooms is NEVER hashed** (research §3.4.2 C2.7 + §3.4.3 C3.6) — only the recording's `media_url` + `content_sha256` and chair-action metadata. Zero-migration: `entry_kind` is TEXT, so new `ENTRY_KIND_ROOM_*` consts only. |
| [ADR-011](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | AGPLv3 inherited | The whole M3 stack is licence-clean: **Element Call (AGPL-3.0)** = same licence as us, no drift; **LiveKit Server (Apache-2.0)** + **lk-jwt-service (Apache-2.0)** compatible; **MinIO (AGPL-3.0)** = same licence as us. Add the MinIO + LiveKit + Element Call + lk-jwt rows to the AGPL-NOTICE at plan time. |
| [ADR-013](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Illegal-content emergency-remove | M3 changes nothing about `EmergencyRemove`. The emergency *room* (M2) is unchanged; M3's "emergency mute" is a chair RTC control inside a live town hall, distinct from the ADR-013 admin removal. |
| [ADR-015](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Pseudonymised actor IDs | **Load-bearing for anonymous town halls (C3.8).** Under `always_pseudonym`, the bridge maps LiveKit participant identities → pseudonyms **at JWT-issue time**; real identities never reach the LiveKit server, and the presenter slot shows the pseudonym overlay on the stream. Chair actions on the chain carry pseudonyms (`from_pseudonym` / `to_pseudonym`), never Lemmy usernames. **NOTE — pseudonymity, not anonymity:** LiveKit operators can still correlate presence via IP; the admin panel must document this. A recording of a pseudonymous town hall shows the pseudonym overlay and is access-controlled (D5 = C floor) so it isn't published to anyone-with-the-URL. |
| [ADR-016](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Brehon is a cross-app governance backplane (B-fetch / B-publish / B-actor) | M3 adds **NO new backplane scope** — B-fetch / B-publish / B-actor all shipped in M2. M3 is the **completion of the V2 messaging reference integration**, built entirely on the M2 room seam. |
| [ADR-014](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Federation interop with vanilla Lemmy | Per OQ-V2-08 (resolved): the messaging/RTC plane is **Brehon↔Brehon only**. Cross-instance town halls (attendees on instances A+B) use standard Matrix federation; vanilla peers see nothing of the RTC plane. Federation-wide emergency mute (C3.7) crosses Brehon instances only. |
| [ADR-010](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Staged releases v0→v1→v2→v3 | M3 messaging is **parallel to** ADR-010's v2-security track, not gated on it (the M1/M2/M3 rename exists to prevent this collision). |

**Contradiction check**: **None found.** No ADR needs superseding. The recording-store-placement tension that paused PRD authorship is resolved by D5 = Option C (evidentiary store, replay-grade access) which honours ADR-004 plane separation (MinIO, not content-plane pict-rs) and ADR-015 (access-controlled recordings of pseudonymous events). The `content_sha256`-on-chain that makes recordings tamper-evidenced is inherent to the M2-registered `Room::RecordingUploaded` shape — no new ADR commitment.

---

## Open Questions This Touches

| OQ | Status | Impact on this sub-PRD |
|---|---|---|
| **OQ-V2-04** (recording storage) | ✅ **RESOLVED in this PRD (2026-06-17)** — dedicated S3-compatible object store; **MinIO** self-host reference backend (AGPL-3.0). Bridge speaks generic S3 API so an operator can swap real S3/R2. Recording is an **evidentiary store with replay-grade access** (D5 = Option C). | Was the ONLY OQ gating M3 PRD authorship. The recording's `content_sha256` lands on the chain regardless; access control floor is "event participants by authorization" (can't be zero under `always_pseudonym`); strict presigned-URL ACL deferred as an additive upgrade. Add a `### OQ-V2-04` resolution block + changelog entry to `99-...md` + update the umbrella PRD OQ-V2-04 row at plan time. |
| [OQ-V2-05](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (chair role source) | **RESOLVED 2026-04-17** | Dual-sourced: governance plane sets the initial chair at event creation (event creator, or jury foreperson for jury-adjacent events); the holder delegates mid-session via a Matrix-native transfer the bridge mirrors as `Room::ChairTransferred`. M3 has a concrete chair-assignment spec. |
| [OQ-V2-06](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (mute-all scope) | **RESOLVED 2026-04-17** | Room-global, **federation-wide**. Chair authority follows Matrix power-level semantics; a chair drops every publisher regardless of home instance. The single hardest *technical* M3 criterion (cross-instance publisher drop <500ms). |
| [OQ-V2-07](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (RTC cost model) | **RESOLVED 2026-04-17** | Instance-local. Each instance hosting a LiveKit SFU pays its own bandwidth. Recordings stay instance-local (no cross-instance recording-store federation). |
| [OQ-V2-08](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (vanilla interop) | **RESOLVED 2026-06-03** | Brehon↔Brehon only. Cross-instance town halls via Matrix federation, not AP. |
| [OQ-V2-10](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (Matrix homeserver) | **RESOLVED 2026-06-01** | Tuwunel (deployed by M1). M3 ops section must document the **containerised RTC deployment topology** — LiveKit/lk-jwt/Element Call/MinIO sidecars; non-host-network bridge↔LiveKit traffic is not loopback and needs explicit handling. |
| [OQ-009](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (juror anonymity in decision phase) | **RESOLVED 2026-06-01** | Graduated mutual visibility (`Juror-<suffix>` revealed to fellow jurors after ≥1 posted comment). Applies to M3 anonymous deliberation town halls the same way it applies to M2 text jury rooms; `always_pseudonym` pin non-configurable. |
| [OQ-005](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (juror/participant notification UX) | 99-register; soft gate | M3 adds a town-hall-invite notification vector (event scheduled → Matrix invite + RTC join link). Ideal: OQ-005 resolves before M3 commits to how invites render. Soft, not hard. |

---

## Proposed Solution

M3 extends the M2 `services/bridge/` daemon with an **RTC control layer** and stands up the
LiveKit RTC stack alongside the existing Tuwunel homeserver — both **fully optional** behind config
flags. No new in-binary governance hook is needed: M3 builds on the M2 transition-hook + room seam.

1. **RTC stack (new optional sidecars)** — **LiveKit Server** (SFU, Apache-2.0), **lk-jwt-service** (JWT minting, Apache-2.0, ships in the Element Call repo), **Element Call** (frontend, AGPL-3.0), and — only when recording is enabled — **MinIO** (S3-compatible recording store, AGPL-3.0). A governance-only instance runs with **none** of these (`rtc_enabled = false`); a town-hall instance that doesn't want recordings runs without MinIO (`record_town_halls = false`).

2. **In `services/bridge/`** (the M2 daemon, extended) — a **stage-mode RTC controller** that, on a scheduled town-hall event: provisions the LiveKit room, mints participant JWTs (mapping identities → pseudonyms at issue time for `always_pseudonym` rooms), seats the **dual-sourced chair** (governance-assigned initial holder), runs the **FIFO raised-hand queue**, performs **mic-passing** (grant publish for 30s, auto-revoke on no-activate, promote next), handles **chair override** (force-demote / force-promote), keeps the **Q&A text sidebar** live, executes **federation-wide emergency mute** (drop all non-chair publishers via Matrix power-levels across instances), and — when `record_town_halls = true` — triggers **LiveKit Egress** → MP4 → MinIO → computes `content_sha256` → calls back into the binary's `append()` to write `Room::RecordingUploaded { media_url, content_sha256, duration_s, speakers, attendance_count }`.

3. **3 new chair/mute hash-chain entry kinds** (zero-migration TEXT consts) — `ENTRY_KIND_ROOM_CHAIR_TRANSFERRED` (`room_chair_transferred`), `ENTRY_KIND_ROOM_CHAIR_OVERRIDE` (`room_chair_override`), `ENTRY_KIND_ROOM_MUTE_ALL` (`room_mute_all`). The recording entry kind (`ENTRY_KIND_ROOM_RECORDING_UPLOADED`) is **already registered** (M2, line 243) — M3 is its first emitter. Count 69 → **72**.

4. **Config rows, no schema change** — `rtc_enabled` (default `false`) and `record_town_halls` (default `false`) are new rows in the existing `governance_messaging_config` typed-KV table, gated independently. The `bridge_room` table (M2, bridge-local) gains RTC state (`chair_id`, `queue_state`, `recording_config`) — a bridge-side migration, no core-schema change.

The Brehon binary's only M3 change is the **3 const declarations + shim re-export** (advisor authors the registry section). All RTC logic lives bridge-side, preserving M1's zero-RTC-deps-in-workspace invariant.

---

## Key Hypothesis

> We believe **the MatrixRTC stack (LiveKit + lk-jwt-service + Element Call) supports chair-controlled
> stage-mode with mic-passing and cross-instance emergency-mute**, driven entirely from the M2 bridge
> daemon using only the existing room seam, **at single-event operator cost commensurate with the
> instance-local RTC model (OQ-V2-07)** — and that a town-hall recording can be stored as an
> evidentiary-but-replay-grade artefact in a plane-separated object store with its content hash on the
> governance chain.
>
> We'll know we're right when: a scheduled town hall opens in stage mode; the chair promotes/demotes
> **four** different users in sequence within a single session without manual intervention; mic-passing
> auto-revokes after the 30s grace on no-activate and promotes the next queued user; recording
> (when `record_town_halls = true`) lands as an MP4 in MinIO with a `Room::RecordingUploaded` entry
> carrying its `content_sha256` on the chain; **emergency mute drops all publishers — including
> cross-instance participants per OQ-V2-06 — within 500ms** measured at the publisher client; the
> three chair/mute entry kinds land with correct schema; and `rtc_enabled = false` runs a clean
> governance-only instance with the entire RTC stack absent. **The real acceptance signal is a pilot
> town hall running end-to-end** (D2), not just green tests.

---

## What We're NOT Building

- **Ad-hoc user-initiated town halls** — governance/event-triggered only (research §3.1). "Start a video call with a friend" is not an M3 feature.
- **Hashing room/recording content** — only the recording's `media_url` + `content_sha256` and the chair-action lifecycle metadata are hashed; speech, video, and the MP4 bytes are never on the chain (research §3.4.3 C3.6).
- **Cross-instance recording-store federation** — recordings stay **instance-local** (§3.6 + OQ-V2-07). No replicating MinIO objects across instances.
- **Strict presigned-URL access-control machinery** — DEFERRED (D5 = Option C). M3 ships `content_sha256`-on-chain integrity + a **participant-floor authorization** on recording fetch (can't be zero under `always_pseudonym`); the strict presigned-URL ACL is an **additive upgrade** in a later sub-phase if the pilot shows it's needed.
- **Recording as a mandatory feature** — recording is **fully optional** (`record_town_halls = false` by default, independent of `rtc_enabled`). A town hall can run with no recording; the recording phase is independently flag-gated.
- **E2EE of governance RTC content against the operator** — per ADR-015 the operator is trusted (research §3.1).
- **Data-export from deleted rooms / recordings** — a tombstoned/erased recording leaves no exportable transcript on the Brehon instance (umbrella PRD deferral).
- **Vanilla-Lemmy interop for the RTC plane** — Brehon↔Brehon only (OQ-V2-08).
- **A new `CaseStatus` variant or any change to existing governance handlers** — M3 only *consumes* the M2 room seam; it never alters governance decision logic.

---

## Success Criteria (verifiable)

| Criterion | How Verified |
|---|---|
| Scheduled town hall opens in stage mode with the governance-assigned initial chair holding the one presenter slot, all others as muted watchers | Integration test (`services/bridge/tests/`) on the event-start path; assert LiveKit room state + chair publish-grant + watcher mute |
| Chair promotes/demotes **4** different users in sequence in one session, no manual intervention | Integration test driving the FIFO queue through 4 mic-passes; assert publish-grant transitions |
| Mic-passing 30s grace: a promoted user who doesn't activate within 30s is auto-revoked and the next queued user is promoted | Integration test with a no-activate user; assert revoke + next-promote at the grace boundary |
| Chair override force-demote + force-promote logged as `room_chair_override` | Integration test; assert `governance_log WHERE entry_kind = 'room_chair_override'` carries `{action, target_pseudonym}` |
| Chair transfer (delegate) logged as `room_chair_transferred` | Integration test on the Matrix-native transfer path; assert chain entry `{from_pseudonym, to_pseudonym, at}` |
| **Emergency mute drops all publishers — including cross-instance — within 500ms, logged as `room_mute_all`** | Integration test measuring at the **publisher client** (not server) across two federated bridge instances; assert all publishers dropped <500ms + chain entry `{chair_pseudonym, federated: true}` |
| Anonymous town hall (`always_pseudonym`): presenter overlay shows pseudonym; real identity never reaches LiveKit | Integration test asserting JWT-issue-time identity→pseudonym mapping; assert LiveKit server never receives the Lemmy username |
| Recording (when `record_town_halls = true`): Egress → MP4 → MinIO → `Room::RecordingUploaded` with `content_sha256` on chain | Integration test on the recording path; assert MP4 in MinIO + chain entry schema (`media_url, content_sha256, duration_s, speakers, attendance_count`) |
| Recording fetch enforces the participant-floor authorization (not anyone-with-URL) | Integration test asserting a non-participant fetch is rejected; a participant fetch succeeds |
| **`record_town_halls = false` → no Egress, no MinIO write, no `Room::RecordingUploaded`; town hall otherwise runs** | Integration test with recording disabled; assert the RTC session completes with zero recording side-effects |
| **`rtc_enabled = false` → zero RTC provisioning, entire stack absent; governance flow unchanged** | `cargo test --test e2e` governance flow passes unchanged with RTC disabled; assert no LiveKit/MinIO calls |
| 3 new `ENTRY_KIND_ROOM_*` consts registered + shimmed + in the registry doc; count 69→72 | `cargo check --workspace`; registry-doc grep; entry-kind collision check (`A == B`, no dup literals) |
| No new `cargo clippy --workspace -- -D warnings` failures | clippy gate |
| **Pilot town hall runs end-to-end** (the real acceptance signal, D2) | Operator runs a concrete pilot scenario (community deliberation broadcast or appeal hearing with audience); chair passes the mic; optional recording lands with its hash entry |

---

## Cross-Cutting Impact ([IMPLEMENTATION-PLAN-v0.md §4](../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md))

- [x] **Hash-chain governance log touched?** YES — **3 new** chair/mute `Room::*` entry kinds (zero-migration; TEXT `entry_kind`) + **first emission** of the M2-registered `ENTRY_KIND_ROOM_RECORDING_UPLOADED`. Content (speech/video/MP4 bytes) never hashed — only metadata + `content_sha256`. Uses the existing `append()` writer unchanged.
- [x] **`actor_pseudonym` table or redaction service touched?** YES (read-only) — anonymous town halls + chair-action entries call the pseudonym allocator for participant rendering; the identity→pseudonym mapping happens at LiveKit JWT-issue time. No schema change.
- [ ] **`CaseStatus::EmergencyRemove` affected?** No — M3's "emergency mute" is a chair RTC control, distinct from the ADR-013 admin removal. No enum or handler change.
- [x] **AGPLv3 notice / source disclosure affected?** YES — add MinIO + LiveKit Server + lk-jwt-service + Element Call rows to the AGPL-NOTICE (all AGPL-3.0 or Apache-2.0, licence-clean). Done at plan time.
- [ ] **New hook in the Brehon binary?** No — M3 adds no in-binary governance hook; it consumes the M2 transition-hook + room seam. The only binary change is the 3 const declarations + shim re-export.

---

## Users & Context

**Primary actor**: **Town-hall participant** (Trusted Member / Member; or pseudonymous Juror for deliberation broadcasts) ([02 §2](../../docs/brehon-law-inspired-network/02-domain-model.md)).
- **Current behaviour**: a community has text rooms (M2) but no live floor — a deliberation or appeal hearing with an audience happens out-of-band (third-party video calls) with no governance record.
- **Trigger**: a town-hall event reaches its scheduled start time, OR a governance moment (e.g. an appeal hearing) is configured to open as a live event.
- **Success state**: the event opens in stage mode; the chair holds the floor and passes the mic down a FIFO queue; watchers raise hands and speak in turn; the Q&A sidebar stays live; under `always_pseudonym` participants appear as pseudonyms; emergency mute is available to the chair federation-wide; and — if recording is enabled — the event's minutes-of-record lands in the recording store with its content hash on the governance chain.

**Non-actors**: a governance-only instance (`rtc_enabled = false`) has no RTC actors at all. Vanilla-Lemmy peers never participate in the RTC plane (OQ-V2-08). The reported party in an appeal-hearing town hall follows the same M2 membership rules (not invited to deliberation; may be invited to an appeal hearing per stage separation — M3 inherits M2's membership policy, doesn't redefine it).

---

## Technical Approach

**Feasibility**: **MEDIUM** — the bridge daemon, room seam, hash-chain writer, pseudonym pin, and config KV table all shipped in M1/M2. M3 is **new optional infrastructure** (LiveKit/lk-jwt/Element Call/MinIO sidecars) + **bridge-side RTC control logic** (stage mode, chair state machine, FIFO queue, mic-passing, Egress trigger) + **3 const declarations**. The marquee risk is **not** storage — it's the **federation-wide emergency-mute <500ms** criterion (a distributed-systems problem: Matrix power-level propagation across federated homeservers is not instant; must be measured at the publisher client). Storage is an ops/policy decision settled by D5.

**Crate(s) affected** (Brehon workspace — minimal):
- `crates/db_schema/src/source/governance/governance_log.rs` — add **3** `ENTRY_KIND_ROOM_*` chair/mute consts (canonical location since Phase 6 DQ-6.6).
- `crates/api/api/src/governance/governance_log.rs` — re-export the 3 new consts via the `pub use` shim (alphabetical).
- `.claude/rules/governance-log-entry-kind-registry.md` — new "## M3 town-hall entry kinds (3, this sub-phase)" section + bump the acceptance-invariant count **69 → 72**. Advisor-owned `.claude/**` meta-work; the Junior writes only the 2 `crates/` files.

**Services M3 introduces** (out-of-workspace; `cargo build --workspace` pulls zero RTC deps):
- **LiveKit Server** (Apache-2.0) — SFU.
- **lk-jwt-service** (Apache-2.0) — JWT minting, ships in the Element Call repo.
- **Element Call** (AGPL-3.0) — frontend.
- **MinIO** (AGPL-3.0) — recording object store [only when `record_town_halls = true`].
- **`services/bridge/` extension** — stage-mode controller, chair state machine, FIFO raised-hand queue, LiveKit JWT integration, Egress trigger, federation-wide mute, `bridge_room` RTC-state columns (`chair_id`, `queue_state`, `recording_config`) + bridge-side migration. **NOTE — `services/bridge` cargo runs on Linux, not Windows** (Docker `rust:1.95` via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`); `ruma-common` E0119 vs `time` on the Windows host. Per `feedback_bridge_validates_on_linux_not_windows.md`.

**Architecture fit** ([03 §4](../../docs/brehon-law-inspired-network/03-architecture.md)): the RTC plane is the third plane (ADR-004 amendment, M2). The RTC controller lives **outside** the Brehon workspace. The Brehon binary owns the hash-chain `append()`; the bridge writes `Room::*` entries by calling back into the binary's governance-log path (the M2 integration seam), never by writing `governance_log` directly. The recording store (MinIO) is plane-separated from the content-plane pict-rs (D5 = Option C).

**New dependencies**: **none** in the Brehon workspace (the zero-RTC-deps invariant holds). Bridge-side additions (LiveKit Rust SDK / S3 client / Egress trigger) live in `services/bridge/Cargo.toml`, already excluded from `--workspace`.

**Technical risks**:

| Risk | Likelihood | Mitigation |
|---|---|---|
| **Federation-wide emergency mute >500ms** — Matrix power-level propagation across federated homeservers isn't instant | **High** | Treat as the primary technical hypothesis (H3 headline). The e2e phase measures at the **publisher client**, not the server. If <500ms cross-instance proves infeasible, fall back to a documented best-effort SLA + in-instance <500ms guarantee, and surface as a DQ before declaring the criterion failed. |
| Mic-passing 30s-grace state machine races (promote/revoke/next under concurrent raises) | Medium | FIFO queue is single-writer (the chair's bridge controller); grace timer keyed by participant; integration test covers the no-activate boundary explicitly. |
| Recording store ops surface (MinIO sidecar, creds, lifecycle) under-scoped | Medium | D5 = Option C defers strict ACL; M3 ships the participant-floor + `content_sha256` only. MinIO is one sidecar gated by `record_town_halls`; ops runbook is part of the M3-core infra phase. |
| `always_pseudonym` recording leaks identity if Egress captures the pre-overlay stream | Medium | Identity→pseudonym mapping is at JWT-issue time; the overlay is on the LiveKit-rendered stream, so Egress captures the pseudonym overlay. Integration test asserts the LiveKit server never receives the real identity. |
| RTC-stack ops topology (non-host-network bridge↔LiveKit) misconfigured | Medium | OQ-V2-10 already flagged containerised non-loopback bridge traffic; M3 ops section documents the topology (LiveKit/lk-jwt/Element Call/MinIO sidecars + network). |
| Doc 04/06 drift — design docs predate the RTC plane | Low | Doc-drift follow-up at plan time: 04 (recording entry schema), 06 §2.2/§7 (RTC plane boundary + threat rows). CODE WINS. |

---

## Implementation Phases (for follow-up `/prp-plan` runs)

<!--
  STATUS: pending | in-progress | complete
  PRP: link to generated plan file once /prp-plan runs on this
-->

| # | Phase | Description | Status | Depends | PRP Plan |
|---|---|---|---|---|---|
| 1 | **M3-core infra** | LiveKit + lk-jwt + Element Call deploy; `rtc_enabled` flag (default false); bridge LiveKit JWT integration (identity→pseudonym at issue); RTC-state columns on `bridge_room` | pending | M2 | - |
| 2 | **M3-core entry kinds** | 3 chair/mute `ENTRY_KIND_ROOM_*` consts + shim re-export + registry-doc M3 section + count bump 69→72 | pending | - | - |
| 3 | **M3-core stage-mode** | stage-mode provisioning; dual-sourced chair seat; FIFO raised-hand queue; mic-passing (30s grace); chair override; Q&A sidebar; chair transfer/override chain entries | pending | 1, 2 | - |
| 4 | **M3-core emergency-mute** | federation-wide mute-all (Matrix power-levels across instances); `room_mute_all` chain entry; <500ms publisher-client measurement | pending | 1, 3 | - |
| 5 | **M3-core recording** (flag-gated) | MinIO deploy [conditional `record_town_halls`]; LiveKit Egress → MP4 → MinIO → `content_sha256` → `Room::RecordingUploaded`; participant-floor fetch authorization | pending | 1, 3, OQ-V2-04 (✅) | - |
| 6 | **M3-core e2e + clean-posture + pilot** | integration tests (4-user mic-pass, mute <500ms, recording + hash, `record_town_halls=false` clean, `rtc_enabled=false` clean) + the pilot town-hall end-to-end demo | pending | 1–5 | - |

### Phase Details

**Phase 1: M3-core infra**
- **Goal**: the RTC stack is deployable and fully optional; the bridge can mint LiveKit JWTs.
- **Scope**: LiveKit Server + lk-jwt-service + Element Call sidecars; `rtc_enabled` config row (default false); bridge LiveKit JWT integration with identity→pseudonym mapping at issue time; `bridge_room` RTC-state columns + bridge-side migration; AGPL-NOTICE rows.
- **Success signal**: `rtc_enabled=false` runs a clean governance-only instance (stack absent); `rtc_enabled=true` mints a valid LiveKit JWT with a pseudonym identity for an `always_pseudonym` room.

**Phase 2: M3-core entry kinds**
- **Goal**: 3 chair/mute entry kinds registered.
- **Scope**: consts in `governance_log.rs` + shim re-export + registry-doc M3 section + collision check + count bump 69→72.
- **Success signal**: `cargo check --workspace`; registry grep; no collision; invariant `A == B`.

**Phase 3: M3-core stage-mode** (bridge-side)
- **Goal**: chair-controlled stage mode with mic-passing.
- **Scope**: stage-mode provisioning; dual-sourced chair seat (OQ-V2-05); FIFO raised-hand queue; mic-passing with 30s grace + auto-revoke + next-promote; chair override (force-demote/promote); Q&A text sidebar; `room_chair_transferred` + `room_chair_override` chain entries.
- **Success signal**: bridge integration test passes 4 mic-passes in sequence + the 30s no-activate boundary.

**Phase 4: M3-core emergency-mute** (bridge-side)
- **Goal**: federation-wide emergency mute <500ms.
- **Scope**: mute-all via Matrix power-levels across federated instances (OQ-V2-06); `room_mute_all` chain entry; publisher-client latency measurement.
- **Success signal**: cross-instance integration test drops all publishers <500ms at the publisher client.

**Phase 5: M3-core recording** (flag-gated; bridge-side)
- **Goal**: optional evidentiary-store recording with chain-hashed content hash.
- **Scope**: MinIO sidecar [only when `record_town_halls=true`]; LiveKit Egress → MP4 → MinIO → `content_sha256` → callback into binary `append()` for `Room::RecordingUploaded`; participant-floor fetch authorization (D5 Option C floor).
- **Success signal**: recording lands with its hash entry; `record_town_halls=false` produces zero recording side-effects; non-participant fetch rejected.

**Phase 6: M3-core e2e + pilot**
- **Goal**: full acceptance + the real pilot signal.
- **Scope**: the §Success Criteria integration tests including both clean-posture cases; then a concrete pilot town hall run end-to-end (D2).
- **Success signal**: all §Success Criteria pass AND a pilot town hall ran with the chair passing the mic (and optional recording landing with its hash entry).

---

## Blocking Dependencies

- **M3-core (Phases 1–6)**: NONE beyond M2 (shipped). OQ-V2-04 resolved in this PRD; OQ-V2-05/06/07/08/10 + OQ-009 all resolved. Ready for `/prp-plan` now.
- **Soft gate**: OQ-005 (notification UX) ideally resolves before Phase 1 commits to how town-hall invites render — soft, not hard.

---

## Decisions Log

| Decision | Choice | Alternatives | Rationale |
|---|---|---|---|
| **D1** M3 core deliverable | **Full town-hall RTC** — stage mode, chair controls (initial + delegate), FIFO raised-hand queue, mic-passing (30s grace), chair override, Q&A sidebar, federation-wide emergency mute <500ms, anonymous town halls, recording-as-artefact | (a) text-only event rooms; (b) RTC without chair controls | User 2026-06-15. Completes the V2 messaging arc; the full chair-controlled stage is the load-bearing new capability (research §3.4.3). |
| **D2** Strategic frame | **Pilot-driven feature for live governance events** — success = a real pilot town hall runs end-to-end | "completes V2 messaging arc" framing (also true, but weaker acceptance bar) | User 2026-06-15. A pilot governance event running end-to-end is a stronger, more honest bar than test-only criteria. Points toward recording being evidentiary (a pilot event's record matters). |
| **D3** Chair entry kinds | **Name 3 NEW consts**: `ENTRY_KIND_ROOM_CHAIR_TRANSFERRED`, `ENTRY_KIND_ROOM_CHAIR_OVERRIDE`, `ENTRY_KIND_ROOM_MUTE_ALL`. **EMIT** the already-registered `ENTRY_KIND_ROOM_RECORDING_UPLOADED` (M2 registered it; M3 is first to emit) | a generic `room_lifecycle_event` catch-all for all chair actions | User 2026-06-15. Chair actions are unambiguously governance records (who passed the mic, who force-muted) and deserve typed kinds — symmetric with M2's room kinds. Recording kind already exists, so M3 only emits. Count 69→72. |
| **D4** Recording store backend | **Dedicated S3-compatible object store; MinIO self-host as reference backend** (AGPL-3.0, same licence). Bridge speaks generic S3 API so an operator can swap real S3/R2 | content-plane pict-rs (M2 considered, reopened only if D5 = convenience-replay) | User 2026-06-15, confirmed by D5 = C. Plane separation (ADR-004) + right home for variable-length recordings. pict-rs is image-optimized; MP4 is off-label. |
| **D5** Recording purpose | **Option C — evidentiary store, replay-grade access.** MinIO + `content_sha256`-on-chain (tamper-evidence kept), participant-floor authorization on fetch (can't be zero under `always_pseudonym`); strict presigned-URL ACL **deferred** as an additive upgrade | (A) full evidentiary minutes-of-record with presigned ACL + retention + tombstone + GDPR in M3; (B) convenience-replay with pict-rs, no ACL | User 2026-06-17. `content_sha256` lands on the chain regardless, so the recording is evidentiary in substance either way — Option B would pay that cost without the matching store/access model. C keeps the plane boundary clean + ships for the pilot + leaves a clean additive ACL upgrade. Coherent with D1/D2/D3 + `always_pseudonym` without over-building. |
| **D6** Recording is optional | **`record_town_halls` flag (default false), independent of `rtc_enabled`** — a town hall runs with recording off; MinIO deploys only when recording is enabled | recording always-on whenever RTC is on | User 2026-06-17 ("it should be optional too"). Clean-posture discipline carried from M1/M2: `messaging_enabled` / `rtc_enabled` / `record_town_halls` are independently gated. A governance-only instance runs none of the stack; a town-hall instance can run without a recording store. |
| **D7** No new backplane scope | **M3 adds NO B-fetch/B-publish/B-actor** — all shipped in M2 | re-open backplane work in M3 | M3 is the completion of the V2 messaging reference integration, built on the M2 room seam (ADR-016). |

---

## Research Summary

**Codebase findings** (2026-06-15/17, grounded in handover + registry reads):
- `crates/db_schema/src/source/governance/governance_log.rs:236-248` — M2 registered **10** `ENTRY_KIND_ROOM_*` consts (zero-migration, TEXT). `ENTRY_KIND_ROOM_RECORDING_UPLOADED` present at **line 243** (registered, never emitted) — M3 is its first emitter. **NO** chair/mute kinds yet; M3 adds 3.
- `.claude/rules/governance-log-entry-kind-registry.md` — "M2 room kinds (10, m2-core-hook)" section + "Acceptance invariants" count **69** at m2-late-b-actor. M3 adds a new section + bumps to **72**. Advisor-owned `.claude/**`; the Junior writes only the 2 `crates/` `governance_log.rs` files (consts + shim re-export).
- `services/bridge/` — M2 daemon with `bridge_room` table + room provisioning. M3 extends it with the RTC controller; bridge cargo validates on **Linux only** (`cargo-linux.sh --manifest-path services/bridge/Cargo.toml`; `ruma-common` E0119 on Windows). Per `feedback_bridge_validates_on_linux_not_windows.md`.
- `governance_messaging_config` (typed KV table, M1) — `rtc_enabled` + `record_town_halls` are new **rows**, no schema change (the `messaging_enabled`/`identity_policy` pattern).
- `docker/docker-compose-vanilla.yml` — pict-rs `asonix/pictrs:0.5.17-pre.9` deployed (`VIDEO_CODEC=vp9`); image-optimized, MP4 off-label — a point against pict-rs for recordings (D4/D5).

**Design-doc alignment**:
- `V2/messaging.md` §3.4.3 C3.1–C3.8 — the eight town-hall scenarios M3 implements (stage mode, raise hand, mic-pass, override, Q&A, recording §C3.6, emergency mute §C3.7, anonymous §C3.8).
- `V2/messaging.md` §3.5 — V2c acceptance criteria (4-user mic-pass in sequence; recording → MP4 + hash entry; emergency mute <500ms) — M3 §Success Criteria operationalize these.
- `V2/messaging.md` §3.6 — hash-chain × GDPR rule; the recording lifecycle inherits tombstone/erasure semantics (deferred-ACL note in D5).
- `99` ADR-008 (recording URL + content-hash hashed, content not), ADR-011 (Element Call/MinIO AGPL-3.0; LiveKit/lk-jwt Apache-2.0 — licence-clean), ADR-015 (anonymous town-hall pseudonym pin + JWT-issue-time mapping), ADR-016 (no new backplane scope), ADR-004 (recording-store plane separation).
- `99` **OQ-V2-04** — has NO dedicated `### OQ-V2-04` heading yet (lives only in umbrella PRD table + research §3.7). Resolution block + changelog entry to be added at plan time, modeled on the OQ-V2-10 resolution-block format.
- `04`/`06` **drift flagged** — design docs predate the RTC plane. Follow-up at plan time: 04 (recording entry schema), 06 §2.2/§7 (RTC plane boundary + threat rows). CODE WINS.

---

*Generated: 2026-06-17*
*Status: DRAFT — ready for `/prp-plan` (M3-core has no blocking dependencies; OQ-V2-04 resolved 2026-06-17, D5 = Option C). Next: resolve OQ-V2-04 in `99-...md` + update the umbrella PRD OQ-V2-04 row at plan time, then `/prp-plan .claude/PRPs/prds/m3-town-halls-rtc.prd.md`.*
