# M3 Town Halls — Key Considerations & Option Trade-offs

**Purpose:** Decision-support for the one open M3 PRD decision (recording purpose →
storage backend → access-control bar) plus the surrounding M3 considerations worth
holding in view. Read this, then resolve D5 in the next session.
**Date:** 2026-06-15
**Companion:** `.claude/PRPs/handovers/m3-prd-authoring-progress-2026-06-15.md` (full session state)

---

## TL;DR

M3 = town halls with mic-passing (the last cluster of the V2 messaging track). Almost all
scope is settled. **One decision is open and it cascades:** is a town-hall recording an
*evidentiary governance record* or a *convenience replay*? That answer determines (a) the
storage backend, (b) how hard we work on access control, and (c) the recording success
criteria. Everything else in M3 is well-defined.

**The fact that collapses most of the debate:** the recording's `content_sha256` lands on the
governance hash chain **no matter what** — it's baked into the `Room::RecordingUploaded` entry
shape. So the recording is *tamper-evidenced by design* even in the "convenience replay"
reading. The only things genuinely up for grabs are **where the bytes live** and **who can
fetch them how easily**.

---

## 1. The core decision (D5): what IS a town-hall recording?

A Brehon town hall is a *governance event* — community deliberation, an appeal hearing with
an audience, an emergency coordination broadcast. The recording is therefore not obviously
"just a video." Three readings, each internally coherent:

### Option A — Evidentiary governance record ("minutes-of-record")

The recording is the official record of what was said and decided at a governance event. If
someone later disputes "the chair said X" or "the panel announced Y," the recording is the
artefact you point to.

**Advantages**
- Consistent with the pilot framing you already chose ("recording as the meeting minutes-of-record").
- Consistent with the MinIO storage decision (plane separation, access control, lifecycle) without rework.
- `content_sha256`-on-chain becomes *load-bearing* (proves the stored MP4 is the one the chain attests), not incidental.
- Honours ADR-015: a jury/appeal town-hall recording exposes pseudonymous participants; access-controlled fetch keeps real identities + the recording itself behind authorization.
- Maps cleanly onto the §3.6 hash-chain × GDPR rule: retention policy, `Room::Tombstoned`, GDPR erasure all have a natural home in an object store with lifecycle rules.

**Disadvantages**
- Highest implementation surface in M3: presigned-URL machinery, access-control checks, retention/lifecycle wiring, GDPR-erasure path. More to build, more to test.
- Raises the bar on the recording success criteria (must test access control + tombstone + erasure, not just "MP4 exists with a hash entry").
- MinIO sidecar is a hard part of the M3-core infra phase (one more service, creds, ops runbook).

### Option B — Convenience replay only

The recording is a courtesy for people who missed the event. Not load-bearing as evidence.
Lower bar on access control and tamper-proofing.

**Advantages**
- Smallest implementation surface — no presigned-URL/access-control gold-plating.
- Storage decision *reopens* in your favour for simplicity: pict-rs (already deployed, `VIDEO_CODEC=vp9` already set) becomes defensible again because plane-separation and access control are no longer load-bearing arguments. One fewer sidecar.
- Fastest path to "a pilot town hall runs end-to-end."

**Disadvantages**
- Directly contradicts the "minutes-of-record" framing you picked for D2 — you'd need to soften that language in the PRD.
- Weakens the plane-separation argument: a governance recording sitting in Lemmy's content-plane pict-rs blurs the ADR-004 boundary M2 was careful to keep clean.
- pict-rs is image/thumbnail-optimized; large MP4s are an off-label workload (works, but not what it's tuned for; size limits + eviction behaviour are a sharp edge).
- You still get `content_sha256` on the chain (unavoidable), so you have *partial* evidentiary weight with *none* of the access control — arguably the worst-of-both: the chain says "here's the tamper-proof hash of the recording" but anyone with the path can fetch it.
- Access-control regret risk: if the pilot ever runs an appeal hearing or a sensitive deliberation as a town hall, "convenience replay, fetchable by anyone" becomes a problem you have to retrofit.

### Option C — Evidentiary store, replay-grade access (hybrid)

Keep MinIO + `content_sha256`-on-chain (tamper-evidence is nearly free and worth having), but
*don't* gold-plate access control in M3 — recordings are fetchable by event participants
without elaborate presigned-URL machinery. Defer the strict access-control layer to a later
sub-phase if the pilot shows it's needed.

**Advantages**
- Keeps the chain guarantee + plane separation (MinIO) — no architectural regret.
- Ships faster than full Option A by deferring the access-control machinery.
- Leaves a clean upgrade path: tightening access control later is additive (presigned URLs in front of an existing bucket), not a migration.
- Honest about the `content_sha256`-is-free reality — you take the tamper-evidence you're getting anyway and don't pretend it's "just replay."

**Disadvantages**
- "Replay-grade access" needs a crisp definition in the PRD or it becomes a hand-wave (who exactly can fetch? event participants by pseudonym? anyone with the URL? — must be specified).
- For a jury/appeal town hall under `always_pseudonym`, even "participant-fetchable" needs *some* authorization or it leaks the recording — so the access bar can't be literally zero; the PRD must state the floor.
- Slightly more PRD nuance to write (two-tier: integrity now, strict ACL later).

---

## 2. Why this decision cascades (the dependency chain)

```
D5 recording purpose
   │
   ├─► storage backend
   │      A → MinIO (stands)
   │      B → reopen: pict-rs defensible (simpler)
   │      C → MinIO (stands)
   │
   ├─► access-control success criteria
   │      A → presigned + ACL + retention + tombstone + GDPR  (high bar)
   │      B → none / minimal                                   (low bar)
   │      C → integrity now, participant-floor ACL, strict later (medium)
   │
   └─► PRD framing language
          A/C → "minutes-of-record" stands
          B   → soften to "replay"
```

Writing the PRD before D5 means rewriting Success Criteria, the storage section, and the
framing. That's why we paused here rather than pushing through.

---

## 3. Considerations NOT up for debate (locked — for context only)

These are settled; listed so the recording decision is made with the full picture.

- **Scope is full town-hall RTC** (D1): stage mode, chair controls, raised-hand FIFO, mic-passing (30s grace), chair override, Q&A sidebar, emergency mute <500ms federation-wide, anonymous town halls. Recording is one slice of this — even if recording is descoped to "replay," the live-RTC half is the bulk of M3.
- **3 new chair entry kinds** (D3): `room_chair_transferred`, `room_chair_override`, `room_mute_all` go on the hash chain regardless of D5. These are evidentiary by nature (who passed the mic to whom, who force-muted) and nobody questioned that — interesting that the *chair actions* are unambiguously governance records while the *recording* is the one in question.
- **Anonymous town halls** (ADR-015): bridge maps LiveKit identities → pseudonyms at JWT-issue time. This is independent of recording storage, but note: an anonymous town hall's *recording* shows the pseudonym overlay, and if that recording is freely fetchable (Option B), you're publishing a pseudonymous person's voice to anyone with the URL. This is a point in favour of A or C for any room under `always_pseudonym`.
- **Emergency mute <500ms federation-wide** (OQ-V2-06): the single hardest *technical* success criterion in M3 (cross-instance publisher drop). Worth noting the real engineering risk in M3 is here, not in storage — storage is an ops/policy decision, mute-latency is a distributed-systems problem.
- **No new backplane scope** (ADR-016): M3 builds no B-fetch/B-publish/B-actor. All shipped in M2.

---

## 4. Other M3 considerations worth surfacing (independent of D5)

These don't need a decision now but the PRD will touch them — flagging so nothing surprises you later.

1. **Emergency-mute <500ms cross-instance is the marquee risk.** Matrix power-level propagation across federated homeservers is not instant. The PRD should treat this as the primary technical hypothesis (it's H3's headline) and the e2e phase must measure it at the publisher client, not the server.

2. **MinIO is AGPL-3.0 — same as us.** No licence drift (unlike if we'd reached for a proprietary store). Worth a one-line ADR-011 row. Element Call is also AGPL-3.0; LiveKit + lk-jwt-service are Apache-2.0. The whole M3 stack is licence-clean.

3. **`rtc_enabled` / `record_town_halls` flags must preserve clean posture.** Just as `messaging_enabled=false` keeps M1/M2 dormant, `rtc_enabled=false` must keep the LiveKit/MinIO stack entirely optional — a governance-only instance must run with none of it. The PRD's clean-posture success criterion carries forward from M1/M2.

4. **Pilot framing changes what "done" means.** Because you chose pilot-driven framing (D2), the strongest M3 success signal is "a real pilot town hall ran end-to-end," not just green integration tests. The PRD should name a concrete pilot scenario (e.g. a community deliberation broadcast or an appeal hearing with audience) as the acceptance demo. This is a stronger, more honest bar than the umbrella PRD's test-only criteria — and it's a point in favour of the recording being evidentiary (a pilot governance event's record matters).

5. **The chair-vs-recording asymmetry is a useful tell.** You (correctly, unquestioned) treated chair actions as governance records worth their own hash-chain kinds. A recording is the *fuller* record of the same event. If chair-transferred is worth hashing, the recording it contextualizes is plausibly worth treating as evidence too. Not decisive — just a consistency signal pointing toward A/C.

---

## 5. My read (advisory — you decide)

If I had to recommend: **Option C (evidentiary store, replay-grade access)**. Reasoning:
- `content_sha256` is on the chain whether you like it or not, so you're already evidentiary in substance — Option B pays the cost (a tamper-hash on chain) without taking the benefit (a store + access model that matches).
- MinIO keeps the plane boundary clean and is the right home for variable-length recordings (your own observation about long town halls).
- Deferring the *strict* access-control machinery keeps M3 shippable for the pilot while leaving a clean additive upgrade path.
- It's the only option that's fully consistent with all of D1, D2, D3, and the `always_pseudonym` requirement *without* over-building.

Option A is also fully coherent and is the right call if you want the recording to be
unambiguous minutes-of-record from day one and don't mind the extra access-control build.

Option B is the one I'd push back on: it reopens settled decisions and leaves you with a
tamper-hash on the chain but a recording anyone can fetch — which is the awkward middle, not
the simple one it looks like.

**No action needed from this doc — it's for your consideration. Resolve D5 next session and
I'll write the PRD.**
