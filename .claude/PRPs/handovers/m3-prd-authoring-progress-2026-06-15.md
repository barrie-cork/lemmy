# M3 PRD Authoring — Session Progress (PAUSED on recording-purpose decision)

**Author:** advisor session on `governance-v0`
**Date:** 2026-06-15
**Supersedes the RESUME pointer in:** `.claude/PRPs/handovers/m3-prd-authoring-handover-2026-06-14.md`
**Status:** PAUSED — PRD not yet written. Blocked on ONE user decision (recording purpose). All other scope decisions resolved.
**VERIFIED_AT:** `2e239e204` (governance-v0 HEAD, clean, in sync with origin)

---

## Why this file exists

The 2026-06-14 handover directed authoring `m3-town-halls-rtc.prd.md`. This session
ran the pre-flight + read all required files + ran the scope-clarification gate with
the user. The user wants to **think over one remaining decision** before the PRD is
written. This file captures everything decided so far so the next session resumes
with zero re-derivation. The companion decision-support doc is
`.claude/PRPs/reports/m3-recording-storage-considerations-2026-06-15.md`.

---

## State at pause

- `governance-v0` HEAD: `2e239e204` — clean, no stash, single canonical worktree, in sync with origin
- DQ: pending = 0
- No active Junior tasks, no active worktrees beyond canonical
- M3 PRD file does NOT yet exist at `.claude/PRPs/prds/m3-town-halls-rtc.prd.md`
- Nothing committed this session (read-only + clarification only)

---

## Decisions LOCKED this session (do not re-litigate)

| # | Decision | Choice | Status |
|---|---|---|---|
| D1 | M3 core deliverable | **Full town-hall RTC** — scheduled stage mode, chair controls (initial + delegate), raised-hand FIFO queue, mic-passing (30s grace), chair override, Q&A sidebar, emergency mute <500ms federation-wide, anonymous town halls (pseudonym overlay), recording-as-artefact | ✅ user-confirmed |
| D2 | Strategic frame | **Pilot-driven feature for live governance events** — success = a real pilot town hall runs end-to-end. NOT the "completes V2 messaging arc" framing (though that's also true). | ✅ user-confirmed |
| D3 | Chair entry kinds | **Name 3 NEW consts in the PRD**: `ENTRY_KIND_ROOM_CHAIR_TRANSFERRED`, `ENTRY_KIND_ROOM_CHAIR_OVERRIDE`, `ENTRY_KIND_ROOM_MUTE_ALL`. EMIT the already-registered `ENTRY_KIND_ROOM_RECORDING_UPLOADED` (M2 registered it, M3 is first to emit). | ✅ user-confirmed |
| D4 | Recording store backend | **Dedicated S3-compatible object store; MinIO self-host as reference backend** (AGPL-3.0, same licence — no drift). Bridge speaks generic S3 API so an operator can swap in real S3/R2. | ✅ user-confirmed — **BUT see OPEN decision below; this may reopen if recording purpose = convenience replay** |

---

## OPEN decision (the pause point)

**D5 — Recording purpose: evidentiary record vs convenience replay.**

The user's three scope answers contained an internal tension I surfaced rather than papered over:

- D2 framed recording as **"minutes-of-record"** → that is an *evidentiary* governance record.
- D4 (MinIO + access control) rests on the recording being a *governance artefact* (plane separation, presigned access, lifecycle/GDPR).
- But on the recording-purpose question the user picked **"convenience replay only"** (lower access-control bar, tamper-proofing optional).

These don't fully cohere. The user paused to consider. The three coherent landing spots
(detailed in the considerations doc) are:

- **(A) Evidentiary** — minutes-of-record; MinIO stands; content_sha256 on chain; presigned access control; retention + `Room::Tombstoned` + GDPR. Consistent with D1+D2.
- **(B) Convenience replay** — courtesy replay, genuinely lower bar; REOPENS storage (pict-rs becomes defensible again); soften "minutes-of-record" in framing.
- **(C) Hybrid: evidentiary store, replay-grade access** — keep MinIO + content_sha256-on-chain (tamper-evidence is nearly free), but DON'T gold-plate access control in M3. Ship-faster middle ground.

**Key technical fact that constrains the decision:** `content_sha256` lands on the hash
chain **regardless** of framing — it's inherent to the `Room::RecordingUploaded { media_url,
content_hash, ... }` entry shape (research §3.4.3 C3.6). So the recording is tamper-EVIDENCED
either way; the only real variable is **access control strictness** and **storage-backend plane placement**.

---

## First action for the resuming session

1. Pre-flight: `pwd && git branch --show-current && git status --short` — confirm `governance-v0`, clean.
2. Read `.claude/PRPs/reports/m3-recording-storage-considerations-2026-06-15.md` (the decision-support doc).
3. Ask the user to resolve **D5** (A / B / C above). If they pick B, RE-ASK the storage backend (pict-rs vs MinIO) before writing.
4. With D1–D5 all locked, author `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` mirroring the M2 sibling structure (see below).
5. Resolve OQ-V2-04 in the PRD Decisions Log + add a `### OQ-V2-04` resolution block + a changelog entry to `99-decisions-and-open-questions.md` + update the umbrella PRD OQ-V2-04 row (line 56).
6. Commit directly to `governance-v0` (PRD is meta-work, not code — direct per `phase-branch.md`).
7. Update `v1-roadmap.json` `implementation_steering.next_logical_sub_phase` → M3 PRD exists, `/prp-plan` is next.
8. `memory_write_eval` at session end (universal-guards §4; retro-check hook fires on governance-v0).

---

## Canonical sibling + key file map (already read this session — re-read only if needed)

| File | Why | Key facts captured |
|---|---|---|
| `.claude/PRPs/prds/m2-governance-triggered-rooms.prd.md` | **Canonical sibling to mirror** | Flat structure: Header / Problem / Evidence / ADRs / OQs / Proposed Solution / Key Hypothesis / NOT Building / Success Criteria / Cross-Cutting / Users / Technical Approach / Impl Phases / Blocking Deps / Decisions Log / Research Summary |
| `.claude/PRPs/prds/v2-messaging-rtc.prd.md` (umbrella) | H3 hypothesis source | **H3 at lines 95–99** (reuse wording). OQ-V2-04 row at **line 56** (update at write). Phase 3 detail lines 264–269. |
| `docs/brehon-law-inspired-network/V2/messaging.md` §3.4.3 | C3 user stories | Lines 165–211: C3.1–C3.8 (stage mode, raise hand, mic-pass, override, Q&A, recording §C3.6, emergency mute §C3.7, anonymous §C3.8). §3.5 V2c acceptance (lines 208–210). §3.6 hash-chain×GDPR (lines 212–222). §3.7 OQ-V2-04 parked (line 226+). |
| `crates/db_schema/src/source/governance/governance_log.rs` | entry-kind reality | **69 consts total.** M2 room kinds at lines 239–248: `ROOM_RECORDING_UPLOADED` present (line 243); NO chair/mute kinds. `ROOM_LIFECYCLE_EVENT` (catch-all) at line 248. M3 adds 3 → 72 consts. |
| `.claude/rules/governance-log-entry-kind-registry.md` | registry discipline | M3 must add a new "## M3 ... entry kinds (3, this sub-phase)" section + bump the acceptance-invariant count 69→72. Advisor-owned `.claude/**` meta-work (Junior writes only the 2 `crates/` files). Collision check + shim-parity invariant. |
| `docker/docker-compose-vanilla.yml` | pict-rs reality | pict-rs `asonix/pictrs:0.5.17-pre.9` deployed, `VIDEO_CODEC=vp9` set. Image-optimized; MP4 is off-label. |
| `docs/.../99-decisions-and-open-questions.md` | OQ + changelog format | OQ-V2-04 has NO dedicated `### OQ-V2-04` heading yet (lives only in umbrella PRD table + research §3.7). OQ-V2-10 (lines 640–653) is the format template for a resolution block. Changelog appends at line 713+ (newest-first). |

---

## ADRs governing M3 (same set as M2, plus)

- **ADR-008** — `Room::RecordingUploaded { media_url, duration_s, speakers, attendance_count }`; content NOT hashed, only URL + content-hash. Chair kinds also go on chain.
- **ADR-015** — anonymous town halls (`always_pseudonym`): bridge maps LiveKit participant identities → pseudonyms at JWT issue time; real identities never reach the LiveKit server. Pseudonym overlay on stream (C3.8). NOTE: this is pseudonymity, not anonymity — LiveKit operators can still correlate via IP (document in admin panel).
- **ADR-016** — M3 adds NO new B-fetch/B-publish/B-actor scope (all shipped in M2). M3 is the completion of the V2 messaging reference integration.
- **ADR-011** — Element Call AGPL-3.0 (same as us); LiveKit Apache-2.0; lk-jwt-service Apache-2.0; MinIO AGPL-3.0 (same as us). No licence drift. Record the MinIO row.
- **ADR-004** — plane separation: recording-store placement is the live tension (governance artefact vs content-plane pict-rs). D4=MinIO honours plane separation; depends on D5.

## OQs M3 inherits (already resolved)

- OQ-V2-05 (chair = dual-sourced) ✅ · OQ-V2-06 (mute-all room-global/federation-wide) ✅ · OQ-V2-07 (RTC cost instance-local) ✅ · OQ-V2-08 (Brehon↔Brehon only) ✅ · OQ-V2-10 (Tuwunel homeserver, deployed) ✅
- **OQ-V2-04 (recording storage)** = the ONLY OQ that gates M3 PRD authorship. Resolution pending D5.

## Services M3 introduces (M3-core infra phase)

- LiveKit Server (Apache-2.0) — SFU
- lk-jwt-service (Apache-2.0, ships in Element Call repo) — JWT minting
- Element Call (AGPL-3.0) — frontend
- **MinIO (AGPL-3.0)** — recording store [conditional on D5 ≠ convenience-replay-with-pict-rs]
- Bridge extension (`services/bridge/`): stage-mode provisioning, chair state machine, raised-hand queue, LiveKit JWT integration, Egress trigger, `bridge_room` table extension (`chair_id`, `queue_state`, `recording_config`)

## Crates touched (Brehon workspace)

- `crates/db_schema/src/source/governance/governance_log.rs` — register 3 new chair consts
- `crates/api/api/src/governance/governance_log.rs` — `pub use` shim re-export (alphabetical)
- `.claude/rules/governance-log-entry-kind-registry.md` — new M3 section + invariant count bump
- NO new migration expected (entry_kind is TEXT; provisioning is bridge-side)

## Explicitly OUT of M3 scope

- Recording **content** hashing (only URL + content-hash on chain)
- Cross-instance recording storage federation (recordings stay instance-local, per §3.6 + OQ-V2-07)
- E2EE of governance chat content (ruled out)
- Ad-hoc user-initiated town halls (governance-triggered only)

---

## Implementation-phase sketch (PRD will refine; for the §Impl-Phases table)

| # | Phase | Description | Depends |
|---|---|---|---|
| 1 | M3-core infra | LiveKit + lk-jwt + Element Call [+ MinIO] deploy; `rtc_enabled` flag; bridge LiveKit JWT integration | M2 |
| 2 | M3-core stage-mode | stage-mode provisioning; chair assignment (dual-source); raised-hand queue; mic-passing; chair override; emergency mute | 1 |
| 3 | M3-core recording | Egress → MP4 → MinIO → `Room::RecordingUploaded` hash-chain entry | 1, 2, OQ-V2-04 (D5) |
| 4 | M3-core e2e | integration tests: 4-user mic-pass, recording + hash entry, emergency-mute <500ms | 1–3 |

---

## Process note for the resuming session

The user explicitly wanted the recording-purpose tension SURFACED, not silently
resolved. Do not paper over D5 to get to writing faster. The PRD's Success Criteria,
the MinIO-vs-pict-rs storage call, and the access-control bar all flow from D5 — writing
the PRD before D5 is locked means a rewrite.
