# V2 — Messaging, Group Chat, and Real-Time Calls

**Status:** Research report (V2 scope, explicitly post-v0). Not a design doc yet.
**Date:** 2026-04-17
**Scope boundary:** This document is research only. Nothing here is in v0. v0 remains the 11 governance endpoints per [IMPLEMENTATION-PLAN-v0.md §3](../IMPLEMENTATION-PLAN-v0.md) and [05 §2](../05-mvp-and-delivery-plan.md). V2 hooks in v0 are called out explicitly below so we don't close doors we'll later want open.
**Predecessor:** This file previously contained four prompt lines (`git log`). Those prompts are preserved as the three research questions in §1.

---

## 1. Questions

Originating prompts (preserved verbatim for audit):

1. **Does Lemmy allow WhatsApp-style group messaging?**
2. **Can Matrix be integrated so group messaging works at an event run by a community (broadcast-to-all quickly)?**
3. **Can it also handle WhatsApp-style 1:1 text + audio + video calls?**

Answers are in §3 (requirements), §4 (what exists today), §5 (Matrix integration shape), §6 (RTC stack), §7 (recommendation), §8 (v0 hooks we must preserve).

---

## 2. TL;DR

- **No.** Lemmy has 1:1 private messages only — the AP wire type hard-codes `to: [ObjectId; 1]`. There is no group chat, no room concept, no WebSocket/SSE transport, and no audio/video of any kind. Delivery is poll-based with email fall-back.
- **Yes, via a separate Matrix homeserver bridged to Lemmy communities.** The practical path is an **application-service bridge** (mautrix-style) that maps each Lemmy community to a Matrix room, plus an **instance-actor** on the Matrix side that can participate in Lemmy federation. Group chat and broadcast both live in Matrix, not in Lemmy.
- **Yes, via the MatrixRTC stack (Element Call frontend + LiveKit SFU + LiveKit JWT service).** This gives 1:1 and group A/V over WebRTC with end-to-end encryption, and is the path the Matrix ecosystem itself has standardised on (MSC4143 + MSC4195). LiveKit is Apache-2.0 (compatible with our AGPL-3.0 server); Element Call is AGPL-3.0 (same licence as us).
- **Scope:** V2 is **governance-triggered rooms only** — no ad-hoc chat. Every room has a governance owner (a case, an event, an appeal, an emergency). See §3 for the full requirements, user stories, and V2a/V2b/V2c phasing.
- **Deployment:** V2 messaging is **optional and feature-flagged per instance**. A Brehon instance can run governance-only (v0 scope) without Matrix, LiveKit, or the bridge. Operators opt in.
- **v0 impact:** minimal but non-zero. Four concrete constraints in §8 keep the V2 door open without expanding v0 scope.

---

## 3. Requirements and user stories (V2)

This section captures the V2 scope as discussed 2026-04-17. It is the source of truth for what V2 is meant to deliver; subsequent sections describe *how* it could be built.

### 3.1 Scope and non-goals

**In scope for V2:**

- Group chat rooms spawned by governance state transitions (cases, scheduled events, appeals, emergencies).
- 1:1 direct messages between Brehon users, mirrored through the bridge (so a user's full conversational history lives in one place).
- Rich media — voice notes, images, short video — in both group rooms and 1:1 DMs.
- Town-hall-style RTC with a single active presenter, a raised-hand queue, chair-controlled mic-passing, and an optional recording artefact.
- Per-room-type identity policy (pseudonymous vs. real-name), chosen via an admin config panel at instance setup.
- Per-community, per-case-type room-lifecycle policy (archive read-only / hard delete after retention / keep writable), chosen via the same config panel.

**Explicitly out of scope for V2:**

- Ad-hoc user-initiated rooms. "Start a chat with a friend outside any governance context" is not a V2 feature. Users who want that can use regular Matrix outside Brehon.
- Community-moderator-initiated rooms outside the governance flow (e.g. "general chat" rooms that aren't tied to a case or event). Deferred to V3 if demand exists.
- E2EE of governance chat content against the Brehon operator. Matrix E2EE still applies between clients; the bridge can see plaintext (needed for metadata logging and moderation). Operators are already trusted per ADR-015.
- Federating Matrix rooms through ActivityPub (doesn't exist as a spec; see §8.4).
- Voice notes as Lemmy PM content (the Lemmy PM schema stays text-only; rich media lives Matrix-side).

### 3.2 Deployment posture

V2 is a **feature flag** at the instance level, not a mandatory dependency.

- An instance with `messaging_enabled = false` runs the v0 governance endpoints as they stand today. Nothing changes for it. No Matrix, no LiveKit, no bridge process, no extra ports, no extra RAM.
- An instance with `messaging_enabled = true` additionally runs a Matrix homeserver, the Brehon↔Matrix bridge, and (if `rtc_enabled = true`) the LiveKit + JWT service stack. These are separate processes, licenced independently, and can be disabled without a data migration.
- Federation between instances is unaffected by the flag — a messaging-enabled instance can federate governance signals with a messaging-disabled one. The messaging-disabled instance simply won't see the `matrix:id` actor property advertised by remote actors.
- Flipping the flag off is supported; flipping it back on is supported. Chat history for governance rooms that existed during the previous "on" window is preserved on the Matrix side.

### 3.3 Configurability — what admins can tune

Two orthogonal policies are exposed via an admin config panel. Both are chosen at V2 enablement time and stored in a governance_config table (or equivalent; schema TBD in the V2 sub-PRD). Policies are **whitelisted enum values**, not free-form text, so they're auditable and enforceable.

**Room-lifecycle policy** — indexed by `(community_id, room_type)`:

| Enum value | Behaviour |
|---|---|
| `archive_readonly` | On case/event close the room becomes read-only; members retain read access; governance log gets `Room::Archived` entry. |
| `hard_delete_after_days(n)` | Room stays writable until close + N days, then content is deleted. Hash-chain entries referencing the room remain but resolve to `Tombstoned` (see §3.6). |
| `keep_writable` | Room stays open post-close for after-action discussion. Close event is logged; subsequent messages are not mirrored into the governance chain. |

**Identity policy** — indexed by `room_type`. Two tiers: **ADR-015-gated** (admin panel cannot override) and **admin-configurable**.

ADR-015-gated room types — pseudonymity is mandatory because juror identity is a group property (see §3.8):

| Room type | Pinned policy | Rationale |
|---|---|---|
| `jury` | `always_pseudonym` (pinned) | Per ADR-015; any opt-out breaks group pseudonymity for every juror on the panel. |
| `appeals` | `always_pseudonym` (pinned) | Same reasoning — appeals panels are jury-equivalent. |

Admin-configurable room types — the admin panel chooses one of four enum values:

| Enum value | Behaviour |
|---|---|
| `always_pseudonym` | Users appear as a pseudonym rendering (e.g. `Speaker-7B2F`) derived from `actor_pseudonym`. Real display name not exposed. |
| `always_real_name` | Users appear as their Lemmy display name. Pseudonym table is not consulted. Suitable for public town halls. |
| `pseudonym_opt_in` | Default real name; user can self-flag as pseudonymous per room. |
| `pseudonym_opt_out` | Default pseudonym; user can self-flag as real-name per room. |

Admin panel applies to: `town_hall`, `emergency`, `spin_out`, `event`. Attempting to set `jury` or `appeals` via the panel returns a validation error referencing ADR-015.

The combination (`room_type` × identity policy) × (`(community_id, room_type)` × lifecycle policy) is the configuration matrix. Shipped defaults (decided 2026-04-17):

- Jury rooms → `always_pseudonym` (pinned) + `archive_readonly` (community-configurable)
- Appeals → `always_pseudonym` (pinned) + `archive_readonly` (community-configurable)
- Emergency coordination → `pseudonym_opt_in` + `archive_readonly`
- Town halls → `pseudonym_opt_in` + `keep_writable`
- Spin-outs (non-jury/appeals parents) → `pseudonym_opt_in` + inherit parent lifecycle
- Spin-outs (jury/appeals parents) → `always_pseudonym` (pinned by parent room type) + inherit parent lifecycle
- Community events → `pseudonym_opt_in` + `archive_readonly`

Admin config panel can change the non-pinned rows before any room is created. Jury and appeals defaults are hard-coded.

**Configuration scope (OQ-V2-02 decision, 2026-04-17):** policy tables are **per-community** with an instance-wide default layer. Schema sketch: `governance_messaging_config(community_id NULL, room_type, lifecycle_policy, identity_policy)`. A `community_id IS NULL` row is the instance default; a row with `community_id` set overrides for that community. Resolution picks the most-specific row. Cross-instance cases with conflicting per-community policies are handled per §3.6 and OQ-V2-03's Matrix-federated-room resolution.

### 3.4 User stories — by feature cluster

User stories are grouped into three clusters. Each cluster corresponds to one phase of V2.

#### 3.4.1 Cluster C1 — Chat infrastructure (phase V2a)

Delivers: bridge process, community-scoped Matrix rooms, 1:1 DMs, rich media. No governance triggers yet — rooms are provisioned manually during V2a for integration testing, and automatically in V2b.

**C1.1 — Bridge provisions a user's Matrix puppet on first Brehon login.**
When a Brehon user with `messaging_enabled = true` on their instance logs in for the first time post-V2a, the bridge creates a Matrix puppet account and writes the MXID into the user's profile. Subsequent logins do not recreate. If the user's instance disables messaging, the MXID remains dormant (not deleted) pending re-enable.

**C1.2 — 1:1 DM between two Brehon users.**
User A opens a 1:1 DM with user B (same or different instance). Bridge creates a Matrix room between the two puppets. Both users see the room in their Matrix client. Messages flow bidirectionally. The existing Lemmy `POST /private_message` continues to work for text-only users who don't engage with Matrix; for users who do, the Lemmy PM is mirrored into the Matrix room via the existing `federated_private_message_after_receive` plugin hook.

**C1.3 — Rich media in 1:1.**
User posts a voice note (≤ 60s audio), image (≤ 10 MB), or short video (≤ 50 MB, ≤ 60s) in a 1:1 DM. Media is uploaded to the Matrix media repo. Recipient sees inline preview. Sender can delete their own media (right to erasure).

**C1.4 — Community-linked Matrix room (manual in V2a).**
An admin creates a community-wide Matrix room for a named Lemmy community. The bridge registers the alias `#_brehon_<community>_<instance>:matrix.example.com`. Community members can opt in; opt-in populates the Matrix room's member list. This is the integration-test path for V2a; in V2b, governance-triggered community rooms replace manual creation.

**C1.5 — Admin config panel — messaging tab.**
Admin sees a new "Messaging" tab with: enable/disable messaging, enable/disable RTC, per-room-type identity policy (4 enum values), per-community per-room-type lifecycle policy (3 enum values). Changes take effect on next room creation; existing rooms retain their creation-time policy. All policy changes are logged to the modlog.

#### 3.4.2 Cluster C2 — Governance-triggered rooms (phase V2b)

Delivers: room-provisioning service that watches governance state transitions, automatic room creation/membership-mirror/archival, metadata-only hash-chain entries.

**C2.1 — Case discussion room.**
When case #N enters `CaseStatus::JuryDeliberation`, the room-provisioning service:
1. Creates a Matrix room with identity policy **pinned to `always_pseudonym`** per ADR-015 (§3.8). The admin panel cannot override for this room type.
2. Adds the 5 assigned jurors as members (as puppets; rendered as `Juror-<pseudonym-suffix>`, never as Lemmy usernames).
3. Writes a `Room::Created { case_id: N, room_id, room_type: jury, identity_policy: always_pseudonym, lifecycle_policy }` entry to the hash chain.
4. Does NOT add the reporter, the reported party, or any admin.
5. If the case spans multiple instances (jurors on instances A and B), the room is **Matrix-federated** per OQ-V2-03 — each participating instance's homeserver hosts the room via standard Matrix federation. Hash-chain reconciliation follows §3.6.
When the case transitions to `CaseStatus::Closed` (any closure reason), the service applies the configured lifecycle policy: archive, schedule deletion, or leave writable. A `Room::LifecycleApplied` hash-chain entry records the action.

**C2.2 — Community event room.**
When a moderator schedules a governance event (new event type, to be defined in V2b — see §8.5), a room is provisioned 15 minutes before start time. All community members are auto-invited (as defined by community role). Identity policy defaults to `always_real_name` but is governed by the config. On event end, the lifecycle policy applies.

**C2.3 — Spin-out room from a case.**
A juror in an active case room requests a spin-out with a subset of fellow jurors (not the full jury). The provisioning service creates a linked room, records `Room::SpunOut { parent_room_id, selected_members }` in the hash chain, and applies the parent room's lifecycle policy (inherits, not configurable independently).

**C2.4 — Appeal room.**
When an appeal is filed against a closed case, a new room is provisioned containing the appealing party, the original reporter, and the appeals panel. The original jury room is NOT invited (stage separation). Identity policy = appeals' configured policy.

**C2.5 — Emergency coordination room.**
When `emergency_remove` is invoked per ADR-013, an emergency room is provisioned with all instance admins + a configured legal-contact MXID. Identity policy defaults to `always_real_name`. Entry: `Room::EmergencyCreated { trigger_case_id }`.

**C2.6 — Membership mirror.**
When a juror is added, removed, or rotated on a case, the corresponding Matrix room membership is updated by the provisioning service (not by Matrix-side admin action). `Room::MembershipChanged { added: [...], removed: [...] }` is hashed.

**C2.7 — Metadata-only hashing.**
Individual chat messages are NOT written to the governance hash chain. Only room lifecycle events (Created, MembershipChanged, Archived, RecordingUploaded, EmergencyCreated, SpunOut, LifecycleApplied, Tombstoned) are hashed. This bounds log growth and keeps the chain's GDPR story consistent with ADR-015 — metadata about rooms is governance; speech inside rooms is not.

#### 3.4.3 Cluster C3 — Town hall with mic-passing (phase V2c)

Delivers: stage-mode RTC, chair controls, raised-hand queue, recording-as-governance-artefact. Built on MatrixRTC + LiveKit + chair-control UI.

**C3.1 — Scheduled town hall opens in stage mode.**
At event start time, the event's Matrix room enters stage mode. LiveKit room is provisioned via JWT service. One presenter slot is reserved, held by the **initial chair** assigned by the governance plane (event creator, or jury foreperson for jury-adjacent events). The chair role is dual-sourced per OQ-V2-05: governance plane sets the initial holder; the holder can delegate mid-session via a Matrix-native "transfer chair" action that the bridge mirrors into the hash chain as `Room::ChairTransferred { from_pseudonym, to_pseudonym, at }`. All other attendees join as watchers — camera off, mic muted.

**C3.2 — Raise hand.**
A watcher clicks "raise hand." A signalling event is emitted as a Matrix state event in the room. The chair's UI shows an ordered FIFO queue of raised hands with timestamps.

**C3.3 — Chair passes the mic.**
Chair clicks "next." Current presenter's LiveKit publish permissions are revoked; the next queued user's publish permissions are granted for 30s. If they don't activate their camera/mic within 30s, permissions revoke and the next in queue is promoted. Chair can skip or reorder the queue.

**C3.4 — Chair override.**
Chair can force-demote the current presenter (e.g. off-topic, abusive). Chair can force-promote a specific user out of queue order. All overrides are logged as `Room::ChairOverride { action, target_user_id }`.

**C3.5 — Q&A sidebar.**
Text chat in the town hall room is not suspended during RTC. Watchers can type; chair can pin/highlight a question (pinning is a Matrix state event).

**C3.6 — Recording as governance artefact.**
If the community's config sets `record_town_halls = true`, LiveKit Egress records the session to MP4. On event end, the MP4 is uploaded to pict-rs (or a dedicated media store, TBD in V2c sub-PRD). A `Room::RecordingUploaded { media_url, duration_s, speakers: [...], attendance_count }` entry is hashed. The recording itself is NOT hashed (too large); its URL + content hash is.

**C3.7 — Emergency mute.**
Chair clicks "mute all." All non-chair LiveKit publishers are muted atomically, **including participants from federated instances** (OQ-V2-06 decision, 2026-04-17): chair authority is room-global, not instance-scoped, matching Matrix power-level semantics. Logged as `Room::MuteAll { chair_pseudonym, federated: true|false }`. Unmute is by re-granting publish permissions individually (no "unmute all" — deliberate friction).

**C3.8 — Anonymous town halls.**
If the room's identity policy is `always_pseudonym`, the presenter slot shows the pseudonym as the name overlay on the LiveKit stream. The bridge maps LiveKit participant identities to pseudonyms at JWT issue time — real identities are never transmitted to the LiveKit server. (Important: LiveKit operators can still correlate presence via IP; this isn't anonymity, it's pseudonymity. Documented in the admin panel.)

### 3.5 Acceptance criteria — per cluster

These are the deliverables that close each phase.

**V2a (C1) closes when:**
- A Brehon user on instance A can DM a user on instance B via the bridge, text + image + voice note, round-trip in < 3s.
- The admin config panel is live and changes to identity/lifecycle policies persist across restarts.
- `messaging_enabled = false` still runs a clean governance-only instance with no regressions against v0.

**V2b (C2) closes when:**
- A case entering `JuryDeliberation` auto-provisions a jury room within 5s, with correct members and correct identity policy.
- Closing the case applies the configured lifecycle policy (archive / delete-schedule / writable) observably in the Matrix room.
- Six `Room::*` entries land in the governance hash chain with correct schema (one per lifecycle stage).
- An emergency_remove call provisions an emergency room within 2s.

**V2c (C3) closes when:**
- A scheduled town hall opens in stage mode, chair can promote/demote four different users in sequence within a single session without manual intervention, and the recording lands as an MP4 with a hash-chain entry pointing at it.
- Emergency mute drops all publishers within 500ms.

### 3.6 Hash-chain × GDPR reconciliation

The configurable lifecycle creates a tension with the hash chain's append-only nature. This section records the rule.

- The hash chain entries themselves are **never deleted or rewritten**. `Room::Created { room_id: R }` at position K in the chain stays at position K forever.
- Room content (Matrix events, recordings, media) **is subject to the lifecycle policy**. Under `hard_delete_after_days`, content is erased from Matrix at T+N days.
- A "tombstone" entry is added to the chain: `Room::Tombstoned { room_id: R, reason: 'retention_expiry' | 'gdpr_erasure' | 'operator_action' }`. Resolution attempts for R after the tombstone return the tombstone, not 404.
- GDPR right-to-erasure is honoured by (a) redacting the user's messages within Matrix per standard Matrix redaction; (b) removing the user's `actor_pseudonym` mapping per [04 §3 ActorPseudonym](../04-data-model-and-api.md). After the mapping is deleted, lookups of "who was `Juror-7B2F`" return `Erased` — permanently and irreversibly, per ADR-015. The hash chain retains `Room::MembershipChanged { added: [pseudonym-7B2F] }` because the pseudonym alone, without its mapping row, is not PII.
- Matrix-side immutability and the chain work together: the chain records that a message-redaction event was observed, but not the redacted content.

**Cross-instance federated rooms (OQ-V2-03 decision):** when a room is Matrix-federated across two instances A and B, each instance writes its own hash-chain entries for events it observed. `Room::Created` entries on A's chain and B's chain have different positions, different prev-hashes, and different instance signatures. Neither is "canonical" — they are independent observations of the same underlying Matrix state. Verifiers must query the chain of the instance making a claim, not a single "true" chain. This parallels how ActivityPub federation already handles state: each instance has its own log, federation is reconciliation by observation not by shared ledger. Cross-instance lifecycle policy conflicts (e.g. A says `archive_readonly`, B says `hard_delete_after_days(30)`) are resolved **room-locally**: each instance applies its own policy to the content on its own homeserver. A juror on instance A may see the room archived read-only while a juror on instance B sees it deleted. The hash chains record this honestly — `Room::LifecycleApplied` on each instance reflects what *that* instance did.

### 3.7 Open questions — status

Questions deferred during the 2026-04-17 discussion. **Resolved** items are noted with their decision; **Open** items still need answers before V2a starts.

- **OQ-V2-01 — Resolved (P1, 2026-04-17).** Identity-policy defaults. Resolution: two-tier model. Jury and appeals are pinned to `always_pseudonym` (ADR-015 group-property requirement, see §3.8). All other room types default to `pseudonym_opt_in` with admin-configurable override. See §3.3.
- **OQ-V2-02 — Resolved (2026-04-17).** Config scope = **per-community** with an instance-wide default layer. Schema sketch in §3.3. Sub-PRD confirms column shape.
- **OQ-V2-03 — Resolved (2026-04-17).** Cross-instance cases = **Matrix-federated room**. Each instance's homeserver hosts the room; hash chains are independent observations per §3.6.
- **OQ-V2-04 — Parked for V2c sub-PRD (2026-04-17).** Recording storage. Deliberately deferred; actual recording volumes and operational constraints aren't known until V2c is on the roadmap. Candidates when decided: separate MinIO/S3 store, pict-rs extension, or Matrix media repo with hash-chained URL.
- **OQ-V2-05 — Resolved (2026-04-17).** Chair role = **dual-sourced**. Governance plane assigns initial chair at room creation; chair can delegate mid-session via a Matrix-native transfer action that the bridge mirrors into the hash chain as `Room::ChairTransferred`. See §3.4.3 C3.1.
- **OQ-V2-06 — Resolved (2026-04-17).** Mute-all scope = **room-global, federation-wide**. Chair authority follows Matrix power-level semantics; a chair with room-admin power-level can mute any publisher regardless of home instance. Consistent with Matrix UX; simplifies incident response mid-event. See §3.4.3 C3.7.
- **OQ-V2-07 — Resolved (2026-04-17).** RTC cost model = **instance-local**. Each instance hosting a LiveKit SFU pays for its own bandwidth. Matches how Matrix federation costs work today. Delivery-plan row must size the expected cost before V2c rollout, but the responsibility model is settled.

### 3.8 ADR-015 reconciliation (OQ-V2-01 resolution note)

This is an audit trail of a deliberate design decision taken on 2026-04-17 after a conflict surfaced between the "customisable defaults" preference and ADR-015's mandatory pseudonymisation.

**The conflict:** the initial draft of §3.3 proposed `pseudonym_opt_in` as the default for every room type, per user preference for experimentation. ADR-015 ([99-decisions-and-open-questions.md](../99-decisions-and-open-questions.md)) and [04 §3 ActorPseudonym](../04-data-model-and-api.md) mandate that the governance log reference `pseudonym`, never `person_id` or username — and that this rule is a **GDPR compliance requirement**, not a user preference.

**Why jury rooms in particular:** pseudonymity is a *group property*, not an individual one. If Juror-7B2F can opt out and reveal as `alice_from_dublin`, the other four jurors on her panel become correlatable — by timing, attendance, speech patterns, presence at recess, anything else Alice exposes. One juror's opt-out degrades pseudonymity for every juror. ADR-015 works precisely because it's blanket, not opt-in.

**The resolution (P1):** narrow the opt-in to room types where pseudonymity is not a group property.

- **Jury** and **appeals** room types are pinned to `always_pseudonym`. The admin config panel cannot override this; the validator rejects any attempt. A sub-PRD wishing to change this would need a new ADR amending ADR-015.
- **Town hall**, **emergency coordination**, **spin-outs** (for non-jury/appeals parents), and **community events** default to `pseudonym_opt_in` with admin-configurable override. Pseudonymity in these rooms is an individual preference, not a group-protection requirement.
- Spin-outs from a jury or appeals parent room inherit the parent's pinned `always_pseudonym` (group property propagates).

**What this preserves:**

- Juror identity protection per ADR-015.
- GDPR right-to-erasure soundness (jury/appeals rooms never expose real identities, so erasure only needs to sever the `actor_pseudonym` mapping).
- User preference for experimentation on room types where it's safe.

**What this gives up:**

- Jurors cannot voluntarily reveal their identity to their fellow jurors in-room. This is a deliberate non-feature — if a juror wants to be known, they can communicate that out-of-band, but the room itself never renders their real name.
- The admin panel has a pinned row that can confuse first-time operators. The panel surfaces a tooltip referencing ADR-015 to explain.

**Appeals path:** if future operational experience suggests jury-room pseudonymity is too strict (e.g. small communities where everyone already knows everyone), the correct path is a new ADR amending ADR-015, not a quiet override in the admin panel. The admin panel's hard-coded pin enforces this.

---

## 4. What Lemmy has today

All citations are against the fork tree at `C:\Users\barri\Developer\brehon-fork\` (Lemmy 1.0-beta @ `811d0d09c`).

### 4.1 Private messages — strictly 1:1

- **Schema** `crates/db_schema_file/src/schema.rs:1044-1058` — `private_message { id, creator_id, recipient_id, content, deleted, published_at, updated_at, ap_id, local, removed, deleted_by_recipient }`. Single recipient FK. No `thread_id`, `subject`, `parent_id`, `attachment_url`, or `read` column.
- **API** `crates/api/routes/src/lib.rs:334-341` — `POST /private_message`, `PUT /private_message`, `DELETE /private_message`, plus two report endpoints. `MarkPrivateMessageAsRead` and `GetPrivateMessages` have been **removed**; unread state lives in the unified `/notification` endpoints (`lib.rs:378-381`).
- **DTO** `crates/db_views/private_message/src/api.rs:10-39` — `CreatePrivateMessage { content, recipient_id }`. Single recipient.
- **AP wire type** `crates/apub/objects/src/protocol/private_message.rs:19-33`:
  ```rust
  pub struct PrivateMessage {
    pub(crate) kind: PrivateMessageType, // ChatMessage | Note
    pub to: [ObjectId<ApubPerson>; 1],   // <-- fixed-size array, one recipient
    ...
  }
  ```
  `PrivateMessageType::ChatMessage` is emitted only to pre-0.20 peers (`crates/apub/objects/src/objects/private_message.rs:98-106`); the default wire form is an AS2 `Note`. No `cc`, `bto`, or `bcc`.
- **Outbound federation** `crates/apub/activities/src/create_or_update/private_message.rs:19-39` — activity is sent to exactly one remote inbox. There is no path to fan-out.

**Conclusion:** Group PMs are not supported at any layer — schema, API, DTO, or AP wire. Adding them would require a new message table, a new AP type, and an outbound fan-out.

### 4.2 No real-time transport

- **No WebSocket.** Lemmy removed WS in 0.18; there is no `actix-ws` / `tokio-tungstenite` in `Cargo.toml`.
- **No SSE.** No `EventStream` / `text/event-stream` / `sse::Sse` in crates.
- **No web push / UnifiedPush / FCM.** No `web_push`, `vapid`, `fcm` hits.
- **Delivery** is (a) `notification` table row (`schema.rs:815-827`) polled by clients via `GET /notification/list`, plus (b) optional email via `NotificationEmailData::PrivateMessage` (`crates/api/api_utils/src/notify.rs:285-320`).

**Conclusion:** Lemmy has no substrate for real-time bidirectional messaging. Any V2 chat feature must bring its own transport.

### 4.3 Media plane cannot carry calls

- `crates/routes/src/images/upload.rs:184-271` proxies uploads to pict-rs (`POST {pictrs_url}/image`). `allow_video` is a boolean flag gated by `local_site.image_allow_video_uploads` — codec support is whatever pict-rs decides.
- No signed URLs, no presigned URLs, no HLS/DASH, no Range/seek handling in `crates/routes/src/images/download.rs:96-124`.
- No WebRTC anywhere. No TURN. No signalling.

**Conclusion:** pict-rs can (at a stretch) store voice notes as short clips but cannot be stretched into audio/video calls. RTC needs a dedicated stack.

### 4.4 Plugin hooks relevant to messaging

Per ADR-012 the Extism host is wired. Existing PM hook points (`crates/api/api_utils/src/plugins.rs:40-68`, called from `private_message/create.rs:75-80`, `update.rs:57,60`, `objects/private_message.rs:170,173`, `notify.rs:301-305`):

- `local_private_message_before_create` / `_after_create`
- `local_private_message_before_update` / `_after_update`
- `federated_private_message_before_receive` / `_after_receive`
- `private_message_report_after_create`
- `plugin_hook_notification` (PM, mention, post/comment reply notify fan-outs)

`_before_*` hooks can mutate the insert form. **A V2 Matrix bridge could ride this seam** to mirror PMs into a Matrix room instead of expanding the core schema — see §5.

---

## 5. Matrix bridge integration

Matrix is the only mature federated-chat protocol that already solves group rooms, E2EE, presence, typing indicators, read receipts, federated roaming, mobile push, and A/V calls as a coherent stack. The integration shape we want is a **bridge**, not a replacement.

### 5.1 Application-service (AS) bridge — the mautrix pattern

Matrix has a well-trodden bridge ecosystem: mautrix-{whatsapp, discord, slack, signal, telegram, …} plus matrix-appservice-{irc, discord, kakaotalk, …}. All follow the same shape:

- The bridge runs as a **homeserver appservice** — a long-running process registered with Synapse via a YAML registration file. It claims an MXID namespace (e.g. `@_lemmy_*:example.com`) and a room-alias namespace (e.g. `#_lemmy_*:example.com`).
- Each remote user on the third-party side gets a **puppet MXID** the bridge controls. Each room on the third-party side gets a **ghost room** on the Matrix side.
- The bridge transforms traffic both ways: remote-inbound events → Matrix room events; Matrix-outbound events → remote protocol actions. `matrix-docker-ansible-deploy docs/container-images.md:85+` lists ~20 such bridges in common production use.
- **Double puppeting** (`configuring-playbook-bridge-mautrix-bridges.md:173-223`) lets the bridge act as the *real* Matrix user rather than a ghost, via an appservice token granted through `matrix_appservice_double_puppet_enabled`. Without it, messages from Matrix-side users to the remote side show up as bot-authored; with it, they show up as the user themselves.
- **End-to-bridge encryption (E2BE)** is supported (`configuring-playbook-bridge-mautrix-bridges.md:145-160`). Matrix-side encryption is preserved up to the bridge boundary; the bridge then re-encodes for the remote protocol.

**No existing Lemmy↔Matrix bridge ships today** (ref-context found none, and mautrix/matrix-appservice catalogues don't list one). So V2 has two sub-options:

**Option A — Bridge as a separate service (recommended).**
A Rust or Go daemon outside the Brehon server process, implementing the Matrix AS spec, talking to Lemmy over its HTTP API (`/private_message`, `/community`, `/post`, `/comment`) and/or over ActivityPub. This is the same pattern as matrix-appservice-irc and mautrix-discord. Advantages: zero coupling to the v0 server binary, can ship independently, licenced independently, can be replaced or disabled without touching Brehon.

**Option B — In-process bridge via Extism plugin.**
The PM plugin hooks in §4.4 are enough to mirror Lemmy PMs outbound into Matrix (via the Matrix Client-Server API called from inside the plugin). Advantages: no separate daemon. Disadvantages: (a) Matrix AS spec needs inbound HTTP endpoints, which means opening routes inside the Lemmy binary — scope creep, plus a surface Extism wasn't designed for; (b) no clean story for Matrix-side rooms mapping to Lemmy communities; (c) E2BE is hard without direct access to the Matrix room state machine.

**Recommendation: Option A.** Extism plugins are the right seam for *governance* hooks, not for standing up a chat protocol gateway.

### 5.2 Lemmy community → Matrix room mapping

Natural mapping for "event happening, broadcast to the community":

- Each Lemmy community (`Community` actor) maps to a Matrix room the bridge creates, aliased `#_lemmy_<community_name>_<instance>:matrix.example.com`.
- Every community member the bridge has seen becomes either a puppet (if they haven't opted into double puppeting) or their real MXID (if they have).
- Posts/comments on the Lemmy side **are not** mirrored — the bridge is for real-time chat, not content duplication. Otherwise every Lemmy community becomes a mirror of every Mastodon reply and the room is unreadable within a week.
- Instead, the Matrix room is a **first-class secondary channel** for the community: announcements, event coordination, Q&A during a live event. Exactly the WhatsApp-group-at-a-meeting use case in the original prompt.
- A Brehon-specific AP actor type (governance-plane, out of v0 scope) could announce "this community has a Matrix room at `#_lemmy_foo:…`" so remote instances know the bridge exists.

**Cross-instance story:** If another Brehon instance `B` federates with ours `A`, each runs its own Matrix homeserver and its own bridge. Instance-B users join `#_lemmy_foo_A:matrix.A.example` through normal Matrix federation. This is already how Matrix federation works — nothing Brehon-specific to design.

### 5.3 1:1 PMs via the bridge

For WhatsApp-style 1:1 DMs the bridge creates a private Matrix room between the two puppets/users. Inbound Lemmy PMs arrive via the `federated_private_message_after_receive` plugin hook (or via polling the Lemmy API) and are forwarded as Matrix messages. Outbound Matrix messages become Lemmy `CreatePrivateMessage` calls. The single-recipient AP wire constraint (§4.1) is **not a blocker for 1:1** — only for group PMs. Group chat lives in the room mapping (§5.2), not in AP private messages.

---

## 6. Real-time audio/video calls — MatrixRTC stack

The Matrix ecosystem's standardised answer for WhatsApp-style 1:1 and group A/V is **MatrixRTC** per MSC4143, with the LiveKit back-end per MSC4195. This is the path Element (the reference client) uses in production.

### 6.1 Component topology

Per `configuring-playbook-element-call.md:8-60` and `configuring-playbook-matrix-rtc.md`:

- **LiveKit Server** (Apache-2.0, Go, Pion WebRTC) — the SFU. Handles signalling, NAT traversal, RTP routing, adaptive degradation, end-to-end encryption passthrough, simulcast, SVC codecs (VP9/AV1), moderation APIs, webhooks. Single binary, Docker, K8s. Self-host is a first-class path (`docs.livekit.io/intro/about.md`).
- **LiveKit JWT Service** — helper that mints room-scoped JWTs from Matrix appservice tokens, so MatrixRTC users authorise into LiveKit rooms without running their own LiveKit identity system. Small Node service; ships in the Element Call repo.
- **Element Call frontend** (AGPL-3.0) — the UI. In practice Element Web and Element X embed it; a standalone install is "largely unnecessary" per the playbook unless you want guest access.
- **Matrix clients** drive the whole thing via MatrixRTC state events in the room. Signalling is in Matrix; media is in LiveKit.

### 6.2 Licence fit

- LiveKit Server: **Apache-2.0** → compatible with our AGPL-3.0 server (no viral concern either way since it runs as a separate process).
- Element Call: **AGPL-3.0** → same licence as us; if we ship it, we're already AGPL so no licence drift. Source disclosure obligation is identical to what we already carry from Lemmy (see `AGPL-NOTICE.md`).
- LiveKit JWT service: **Apache-2.0**.

No licence blockers. No hidden CLA traps. No dual-licence re-negotiation risk.

### 6.3 Alternative: Jitsi

Jitsi (Apache-2.0, Java SFU `jitsi-videobridge` + Prosody + Jicofo + web UI) is the other mature AGPL-friendly option. It's a proven federation-free videoconf stack, deployable via the same playbook (`container-images.md:155-158`). Weakness for our case: no Matrix-native integration path — Jitsi predates MatrixRTC and doesn't speak MatrixRTC state events. We'd be bolting signalling on top of Matrix ourselves. The MatrixRTC ecosystem standardised on LiveKit in 2024–2025 exactly to avoid this. Recommend Jitsi only if LiveKit is blocked (e.g. ops constraint).

### 6.4 What does NOT work

- **Peer-to-peer WebRTC without an SFU** — fine for 1:1, fails at 5+ participants because upload bandwidth becomes O(n²). Rule it out for group events.
- **MCU (transcoding mixer)** — historically what Jitsi did before Jicofo+JVB decoupled; never what we want for >10 participants.
- **Pure federation of A/V without an SFU per-instance** — not a thing. RTC is not ActivityPub. Each Brehon instance that wants calls runs its own LiveKit.

---

## 7. Recommendation

For the V2 phase (no date yet), ship the messaging/RTC layer as a **separate stack** bridged into the Brehon fork, not as an expansion of the Lemmy-inherited core:

| Layer | Component | Licence | Brehon coupling |
|---|---|---|---|
| Group chat, 1:1 chat | **Matrix** (Synapse or Conduit) | AGPL-3.0 / Apache-2.0 | Deployed alongside Brehon, not in the binary |
| Brehon↔Matrix integration | **New appservice bridge** (Rust, greenfield) | Apache-2.0 or AGPL-3.0 | Talks to Lemmy HTTP API + the governance AP types |
| A/V calls | **LiveKit + LiveKit JWT service + Element Call** | Apache-2.0 / AGPL-3.0 | Deployed alongside Matrix; no direct Brehon coupling |
| Matrix clients | Element Web + Element X | AGPL-3.0 | User-side; Brehon doesn't ship a client in v0/v2 |

Sub-recommendations:

- Write the bridge in Rust. It's a new project, we need long-term maintenance, and the team is already in Rust.
- Do **not** put the bridge inside the Brehon server process. Separate service, separate deploy, separate blast radius.
- Make Matrix-room-per-community an **opt-in** setting per community (governance decision), not automatic — otherwise federation partners will see rooms they didn't ask for.
- Treat 1:1 PM mirroring as **user opt-in** per account. Lemmy's existing PM endpoints keep working untouched for users who don't want Matrix in their flow.

---

## 8. v0 hooks we must preserve

V2 depends on four things already being true in v0/phase-5/phase-6. None of these expand v0 scope — they're consistency constraints on work we're already doing.

### 8.1 Keep the PM plugin hooks stable (ADR-012)

The PM-path Extism hooks listed in §4.4 are the cleanest seam for the future V2 bridge to mirror PMs without touching the schema. v0 and its successor phases must **not** delete these hooks or rename them in a breaking way. If a governance feature adds a new hook in the PM path, land it beside the existing ones, not replacing them.

**How to apply:** any phase plan touching `crates/api/api_crud/src/private_message/` or `crates/apub/objects/src/objects/private_message.rs` must keep the existing `plugin_hook_{before,after}_*` call sites. If a removal is proposed, surface as an open question.

### 8.2 Reserve MXID-looking actor handles

A future Matrix bridge will want to advertise MXIDs of the form `@_lemmy_<user>:<matrix-domain>` on the Lemmy side (as a profile field or actor extension). v0 should **not** enumerate or block `@_`-prefixed usernames at registration, and the governance-plane AP actor extensions we add should leave room for a `matrix:id` property.

**How to apply:** any v0 work on `Person` / actor extensions should not introduce a character-class restriction that rules out `@_`. Document in the actor-extension schema once that lands.

### 8.3 Don't close the door on SSE for governance notifications

Phase 5+ will want fan-out for governance events (jury invitations, case status changes). If we add SSE or WebSocket for that, the same transport can carry V2 chat-adjacent notifications (typing indicators, presence) later. v0 currently has no RT transport and doesn't need one, but if the decision comes up in Phase 5 or 6, pick **SSE over WebSocket** — it's simpler, it composes with HTTP caching, and the Matrix bridge doesn't need Lemmy's RT transport anyway (it polls or uses plugin hooks).

**How to apply:** if a real-time transport question appears in a future plan, add a link back to this §8.3 and default to SSE unless there's a specific reason for WS.

### 8.4 Expose case state transitions as a subscribable event stream

V2b (cluster C2, see §3.4.2) requires a **room-provisioning service** that watches governance state transitions and creates/archives Matrix rooms in response. For that service to exist without modifying the Brehon server binary, v0/phase-5/phase-6 must emit state transitions as events the provisioning service can subscribe to — either via an internal pub-sub channel with an Extism hook, or via an outbound signal (governance AP activity, webhook, or NOTIFY on a Postgres channel).

The specific events V2b needs:

- `CaseStatus::*` transitions (especially `JuryDeliberation`, `Closed`, `EmergencyRemove` per ADR-013)
- Jury membership changes (added, removed, rotated)
- Appeal filed
- Scheduled governance events opening and closing (event type itself is TBD in phase 6)

v0's existing `plugin_hook_notification` in `crates/api/api_utils/src/notify.rs` is one precedent but is PM-scoped. Phase 5+ should add an equivalent hook for governance state transitions (e.g. `governance_case_after_transition`) in the same style.

**How to apply:** when any phase implements a governance state transition, emit a hookable event at the commit point. Do not let state transitions happen only as implicit side-effects of an API handler. If the hook doesn't fit, surface as an open question — retrofitting event emission in V2 will require re-auditing every state-mutating code path, which is expensive and error-prone.

**Corollary:** the governance_config table (or wherever admin-panel policies live) should reserve a `messaging_*` prefix so V2 can add columns without a schema war with other phases.

---

## 9. Out of scope (explicitly)

For honesty with future readers, the following are **not** answered here and should not be assumed:

- **E2EE between Brehon users over Lemmy AP.** Out of scope. If users want E2EE, they use the Matrix side, which has E2EE (Megolm/Olm). Lemmy AP PMs are not end-to-end encrypted and that's not changing in V2.
- **Push notifications to mobile.** Matrix handles this via Sygnal / UnifiedPush / ntfy (see `container-images.md:182-183`). Brehon doesn't need its own push stack — piggybacking on Matrix's is cheaper.
- **Federation of Matrix rooms via ActivityPub.** Doesn't exist as a mature spec (searched FEP registry via ref-context, no results). Matrix federates via Matrix server-server; AP federates via AP server-server; they don't merge. The bridge is the seam.
- **Voice notes in Lemmy PMs.** pict-rs could store short clips but the PM `content` field is a single text column. A "voice note in a PM" would be a Matrix-room event, not a Lemmy PM.
- **Choosing Synapse vs Conduit/Dendrite.** Ops detail, not architectural. Synapse is the default and is what MatrixRTC tested with (`configuring-playbook-element-call.md:36-40` warns MatrixRTC "very likely only works with Synapse").
- **A sub-PRD for V2 messaging.** When V2 is actually on the roadmap, run `/prp-prd` to generate one. This file is predecessor research, not a design doc.

---

## 10. Open questions to escalate (if/when V2 is scheduled)

1. **Bridge authorship — in-house or vendor/fork of an existing mautrix project?** Leaning in-house because no existing Lemmy bridge ships; mautrix-whatsapp would not fit without rewriting the remote-protocol half.
2. **Matrix homeserver choice: Synapse (Python, reference) vs Conduit (Rust, lightweight).** Synapse is the only MatrixRTC-compatible choice today; Conduit/Dendrite catch up at an unknown rate.
3. **Who funds/operates the RTC infrastructure for federation partners?** Each instance runs its own LiveKit; small instances may balk at the ops overhead. Worth a v2-delivery-plan row.
4. **FEP path for governance↔Matrix interop.** We couldn't find a matching FEP via ref-context. Worth tracking FEPs in the messaging space over the next year before locking the bridge design.

---

## 11. Sources

Codebase (fork-local):

- `crates/db_schema_file/src/schema.rs:1044-1058, 1060-1072, 815-827` — PM, PM report, notification tables
- `crates/api/routes/src/lib.rs:334-341, 371-381` — PM + notification routes
- `crates/db_views/private_message/src/api.rs:10-39` — PM DTOs
- `crates/apub/objects/src/protocol/private_message.rs:19-33` — AP wire struct, `to: [ObjectId; 1]`
- `crates/apub/objects/src/objects/private_message.rs:98-177` — legacy ChatMessage compat + plugin hooks
- `crates/apub/activities/src/create_or_update/private_message.rs:19-39` — outbound send
- `crates/api/api_crud/src/private_message/{create,update,delete}.rs` — handlers + plugin hook sites
- `crates/api/api_utils/src/plugins.rs:40-68`, `notify.rs:285-320` — plugin hook surface + notification path
- `crates/routes/src/images/{upload,download}.rs` — pict-rs proxy (no signed URL, no streaming)

External (via `ref-context`, authoritative):

- LiveKit — `docs.livekit.io/intro/about.md` (SFU architecture), `github.com/livekit/livekit/README.md` (features, SDKs, Apache-2.0 licence confirmation)
- MatrixRTC stack — `matrix-docker-ansible-deploy/docs/configuring-playbook-element-call.md`, `configuring-playbook-matrix-rtc.md`, `configuring-playbook-livekit-jwt-service.md` (MSC4143, MSC4195, component topology)
- Mautrix bridge pattern — `configuring-playbook-bridge-mautrix-bridges.md:1-225` (appservice model, double puppeting, E2BE, permissions, relay mode)
- Bridge catalogue — `container-images.md:83-128` (mautrix-whatsapp, -telegram, -signal, matrix-appservice-irc, etc.)
- Fedify AP addressing model — `fedify/docs/manual/inbox.md:111+` (`to`, `cc`, `bto`, `bcc` behaviour, shared vs personal inbox, instance-actor pattern) — informs why Lemmy's 1-element `to` array is a hard block for AP-native group PMs

Design-doc anchors:

- [IMPLEMENTATION-PLAN-v0.md §3](../IMPLEMENTATION-PLAN-v0.md) — v0 scope, V2 deferral
- [05-mvp-and-delivery-plan.md §2, §3](../05-mvp-and-delivery-plan.md) — 11-endpoint v0 and simplifications
- [99-decisions-and-open-questions.md ADR-012](../99-decisions-and-open-questions.md) — Extism plugin host (relevant to §5.1 Option B and §8.1)
- [99-decisions-and-open-questions.md ADR-011](../99-decisions-and-open-questions.md) — AGPL-3.0 (relevant to §6.2)
