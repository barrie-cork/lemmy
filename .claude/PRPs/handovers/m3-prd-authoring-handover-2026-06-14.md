# M3 PRD Authoring Handover — 2026-06-14

**Purpose:** Self-contained brief for the next session that will author `m3-town-halls-rtc.prd.md`.
**VERIFIED_AT:** `afb62cbad` (governance-v0 HEAD, 2026-06-14)

---

## State at handoff

- `governance-v0` HEAD: `afb62cbad` — synced to origin, clean, no stash
- DQ: pending=0
- No active Junior tasks
- No active worktrees (canonical only)

---

## What M3 is

M3 = V2c = Cluster C3 in the research doc = **Town halls with mic-passing**.

Scope (from `v2-messaging-rtc.prd.md` + `V2/messaging.md §3.4.3`):
- MatrixRTC stage-mode room provisioned for scheduled governance events
- Chair controls (assigned dual-sourced per OQ-V2-05): initial chair from governance plane, mid-session delegate via Matrix-native transfer
- Raised-hand FIFO queue (Matrix state events)
- Chair mic-passing: 30s activation grace, skip/reorder, chair override
- Q&A sidebar: text chat + chair pin/highlight
- Recording as governance artefact (LiveKit Egress → MP4 → pict-rs or media store → `Room::RecordingUploaded` hash-chain entry)
- Emergency mute: room-global, federation-wide (<500ms), per OQ-V2-06
- Anonymous town halls: pseudonym overlay on LiveKit stream, bridge maps identities at JWT issue time

Key acceptance signals (from research §3.5):
- Stage-mode town hall: chair promotes/demotes 4 users in sequence without intervention
- Recording lands as MP4 with hash-chain entry pointing at it
- Emergency mute drops all publishers (including cross-instance) within 500ms

---

## What M2 shipped (M3 builds on this)

All of M2 is on `governance-v0`:

| Phase | PR | What it delivered |
|---|---|---|
| M2-core-hook | #184 | `governance_case_after_transition` hook at 12 transition sites; 10 `ENTRY_KIND_ROOM_*` consts; `append_room_event` wrapper |
| M2-rooms-a | #191 | Bridge room-provisioning service; `bridge_room` table; OQ-009 graduated reveal; idempotent provisioning |
| M2-late | #196 | B-publish sanction propagation |
| M2-late-2 | #196 | Sanction subscriber + power-levels in Matrix rooms |
| M2-late-b-actor | #197 | Portable actor-ID linkage (ADR-016 C3) |

**What M3 adds on top of M2:**
- LiveKit Server + LiveKit JWT service + Element Call — new sidecar stack (none deployed yet)
- Bridge extension: stage-mode provisioning logic, chair-control state machine, raised-hand queue, recording egress trigger
- New `ENTRY_KIND_ROOM_*` kinds to register/emit: `ROOM_RECORDING_UPLOADED` (const registered in M2-core-hook, NOT yet emitted — M3 emits it), plus any new kinds for chair events (`Room::ChairTransferred`, `Room::ChairOverride`, `Room::MuteAll`)

---

## Key OQ to resolve at PRD-write time

**OQ-V2-04 — Recording storage** (status: `Parked for V2c sub-PRD per research §3.7`)

> "Deliberately deferred; actual recording volumes and operational constraints aren't known until V2c is on the roadmap. Candidates when decided: separate MinIO/S3 store, pict-rs extension, or Matrix media repo with hash-chained URL."

You must pick one of these three candidates (or a fourth if justified) and record the decision in the PRD's Decisions Log and in `99-decisions-and-open-questions.md`. This is the only OQ that genuinely gates M3 PRD authorship.

**OQs already resolved that M3 inherits:**
- OQ-V2-05: Chair role = dual-sourced ✅
- OQ-V2-06: Mute-all = room-global, federation-wide ✅
- OQ-V2-07: RTC cost = instance-local ✅
- OQ-V2-08: Brehon↔Brehon only ✅
- OQ-V2-10: Tuwunel homeserver ✅ (already deployed in pilot)

---

## ADRs that govern M3

Same set as M2, plus:
- **ADR-008**: `Room::RecordingUploaded { media_url, duration_s, speakers: [...], attendance_count }` is the new entry kind to emit. Recording content NOT hashed — only URL + content-hash.
- **ADR-015**: Anonymous town halls (`always_pseudonym`) — bridge maps LiveKit participant identities to pseudonyms at JWT issue time. Real identities never transmitted to LiveKit server.
- **ADR-016**: M3 is the completion of the V2 messaging reference integration. No new B-fetch/B-publish/B-actor scope in M3 — those shipped in M2.
- **ADR-011**: Element Call is AGPL-3.0 (same as us). LiveKit is Apache-2.0. LiveKit JWT service is Apache-2.0. No licence drift.

---

## Crates / services affected by M3

**In `services/bridge/`** (the M2 daemon, extended):
- Stage-mode provisioning logic for scheduled governance events
- Chair-control state machine (initial assignment, mid-session delegate, override)
- Raised-hand queue handler (Matrix state events)
- LiveKit JWT minting integration
- Recording egress trigger (LiveKit Egress API call on event end)
- `bridge_room` table extension (stage-mode fields: `chair_id`, `queue_state`, `recording_config`)

**New external services (not yet deployed):**
- LiveKit Server (Apache-2.0)
- LiveKit JWT service (Apache-2.0, ships in Element Call repo)
- Element Call frontend (AGPL-3.0)

**In `crates/`** (Brehon workspace):
- `governance_log.rs`: register/emit `Room::ChairTransferred`, `Room::ChairOverride`, `Room::MuteAll` consts (some may already be registered in M2 — verify with grep)
- `governance_log.rs`: `ENTRY_KIND_ROOM_RECORDING_UPLOADED` const is ALREADY registered (M2-core-hook Task 4). M3 is the first phase to EMIT it.
- No new migration expected (entry_kind is TEXT, governance event provisioning is bridge-side)

**NOT in M3 scope (explicitly out):**
- Recording content hashing (only URL + content-hash lands on chain, per research §3.4.3 C3.6)
- Cross-instance recording storage federation
- E2EE of governance chat content (already ruled out)
- Ad-hoc user-initiated town halls

---

## Sibling PRD to mirror

Read the M2 PRD as the canonical sibling before writing:
`.claude/PRPs/prds/m2-governance-triggered-rooms.prd.md`

It has exactly the right structure. Mirror:
- Header (Parent, Predecessor, Created, Status, Scope, Naming)
- Problem Statement
- Evidence (predecessor research citations — use `V2/messaging.md §3.4.3`)
- ADRs That Govern This
- Open Questions (OQ-V2-04 is the only OPEN one; others are RESOLVED)
- Proposed Solution
- Key Hypothesis (mirror H3 from umbrella PRD)
- What We're NOT Building
- Success Criteria (verifiable)
- Cross-Cutting Impact
- Users & Context
- Technical Approach (feasibility, crates affected, new dependencies, risks)
- Implementation Phases (for follow-up `/prp-plan` runs)
- Blocking Dependencies
- Decisions Log (including OQ-V2-04 resolution)
- Research Summary

---

## Implementation Phases (sketch — PRD will refine)

| # | Phase | Description | Status | Depends |
|---|---|---|---|---|
| 1 | **M3-core infra** | LiveKit Server + JWT service + Element Call deployment; `rtc_enabled` config flag; bridge LiveKit JWT integration | pending | M2 |
| 2 | **M3-core stage-mode** | Stage-mode provisioning for scheduled governance events; chair assignment; raised-hand queue; mic-passing; chair override; emergency mute | pending | 1 |
| 3 | **M3-core recording** | Recording-as-governance-artefact: LiveKit Egress → MP4 → media store → `Room::RecordingUploaded` hash-chain entry | pending | 1, 2, OQ-V2-04 |
| 4 | **M3-core e2e** | Integration tests: 4-user mic-pass, recording + hash entry, emergency-mute <500ms | pending | 1-3 |

---

## Files the session needs to read (in order)

1. This handover (you're reading it)
2. `.claude/PRPs/prds/m2-governance-triggered-rooms.prd.md` — canonical sibling shape
3. `docs/brehon-law-inspired-network/V2/messaging.md §3.4.3` (lines 165–211) — C3 user stories
4. `docs/brehon-law-inspired-network/v2-messaging-rtc.prd.md` §"Key Hypotheses" H3 + §"Implementation Phases" Phase 3 — H3 wording to reuse
5. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — OQ-V2-04 entry (currently parked) + OQ-V2-05/06/07 resolved entries
6. `crates/db_schema/src/source/governance/governance_log.rs` lines 235–260 — verify which `ENTRY_KIND_ROOM_*` consts are already registered vs which M3 needs to add

---

## First action for the new session

1. **Pre-flight**: `pwd && git branch --show-current && git status --short` — confirm on `governance-v0`, clean
2. **Resolve OQ-V2-04**: decide recording storage option (pict-rs extension is the lean given pict-rs is already deployed in the pilot stack). Write the decision in the PRD Decisions Log + `99-decisions-and-open-questions.md`.
3. **Author `m3-town-halls-rtc.prd.md`** at `.claude/PRPs/prds/m3-town-halls-rtc.prd.md`
4. **Commit directly to `governance-v0`** (PRD is meta-work, not code — direct per `phase-branch.md`)
5. Update `v1-roadmap.json` `implementation_steering.next_logical_sub_phase` to reflect M3 PRD exists + `/prp-plan` is next

---

## DQ session ID for this session

Next DQ entry (if any clarify entries needed): `a3d0e9941441-063` (pre-generated; use `bash scripts/brehon/dq-v3-new-entry.sh` to get a fresh one in the new session).

---

*Authored: 2026-06-14 by advisor session on governance-v0 @ afb62cbad*
