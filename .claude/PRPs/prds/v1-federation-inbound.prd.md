# V1 — Federation Inbound Advisory

**Status:** Sub-PRD (v1 scope, post-v0 / post-Phase 6). Not a v0 deviation.
**Scheduling:** **Hard-gated on Phase 6 PR merge to `governance-v0`**; no implementation before gate satisfied (see §8.0). Plan-level review (this PRD) can proceed in parallel with Phase 6 implementation. Per B5 resolution 2026-04-19.
**Scope boundary:** This PRD adds **inbound HTTP federation processing** for governance signals on top of Phase 6's outbound + direct-call inbound foundation. **Auto-apply remains forbidden per ADR-006** — every inbound signal lands as advisory, surfaced in admin review, cross-link/dismiss only on explicit admin action.
**Date:** 2026-04-19
**Predecessor:** Phase 6 (`brehon-fork-advisor-phase6/.claude/PRPs/plans/phase-6-federation.plan.md`) — outbound publish + AP types + direct-call receive functions for the round-trip test only. v1 inbound takes Phase 6's `receive_remote_*` functions and wires them into the live HTTP inbox dispatcher with peer-trust gating, rate limits, abuse defences, and an admin review surface.
**Naming note:** This is **v1-federation-inbound**, distinct from any "Phase 6.x" or "Phase 7" naming. Per ADR-010, v1 is the "production-grade governance" milestone; full inbound processing is v1 scope per [05 §7.1](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md). v3 unlocks **acting on** inbound signals (per [05 §7.3](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md): "Full federation inbound processing (beyond stored-advisory)") — v1 only **stores and surfaces** them.

---

## 1. Vision and goals

The Brehon principle this PRD encodes: **an external honour judgment is data, not a command.** A peer's sanction notice describes what their jury decided about an actor; it does not bind our jury, our admins, or our local truth. ADR-006 formalises this — "a federated instance getting compromised must not cascade its compromise through our moderation." v1 inbound carries this principle into live HTTP processing: every signed activity that reaches our inbox is verified, validated, persisted, surfaced, and **stops there** until a local admin decides it's worth citing or dismissing.

The user framing for v1 is: *"The admin dashboard will allow community groups to experiment with how they configure a lot of these settings, although defaults might have to be decided on."* That reframes inbound federation policy from a hard-coded global default into a **dashboard-configurable surface**. Which peers are trusted, which signals are surfaced, which ones drop silently — all become knobs the dashboard exposes. v1 ships a sane default policy and the surface to change it.

### 1.1 Maturity ladder

| Stage | What ships | Reach | Decision authority |
|---|---|---|---|
| **v0 (Phase 6 in flight)** | Outbound publish; inbound exists only as a direct test-call into `PublishSanctionNotice::receive` from `tests/e2e.rs` | Local-only; no live HTTP path | Phase 6 plan §11 explicitly defers HTTP routing |
| **v1 (this PRD)** | Inbound HTTP path live; signals stored advisory; surfaced in admin dashboard for review; peer-trust state model; rate limiting; abuse defences | Live federation receive | Local admin (cross-link / dismiss); local jury (cite as evidence) |
| **v2 (security hardening per ADR-010)** | SSRF-isolated federation fetch worker; OPA policy engine wires federation policy decisions; signed log outside runtime | Same shape, harder boundary | Same — local admin / jury |
| **v3 (per [05 §7.3](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md))** | Policy-driven auto-apply (still subject to community-defined acceptance rules; never silent) | Live federation receive + policy-driven action | Policy + admin (still no silent action) |

v1's job is to make inbound federation operationally usable — admins can see what's coming in, can curate which peers we listen to, can cite remote evidence in local cases — without ever giving up local sovereignty. v3's job is to add a policy layer; v1 deliberately does not pre-empt it.

### 1.2 Goals

- **G1.** Live HTTP inbox handlers for the three governance AP types Phase 6 defines (`SanctionNoticeObject`, `TrustAttestationObject`, `ModerationLabelObject`).
- **G2.** Per-peer trust state model (`Unknown`/`Allowlisted`/`Blocklisted`/`Untrusted-Receive`) with admin-dashboard configurability.
- **G3.** Admin review surface — paginated list, filters, cross-link to local case, dismiss-as-reviewed.
- **G4.** Per-peer and per-actor rate limits with dashboard-tunable defaults.
- **G5.** Storage caps + drop-log so a hostile peer cannot fill our DB with advisory rows.
- **G6.** Replay-protection (nonce table + configurable window).
- **G7.** Every inbound — accepted, dropped, blocked — emits a governance_log entry for audit.

### 1.3 Non-goals

- **NG1.** Auto-apply — explicitly v3 (§13 OQ-FED-IN-1).
- **NG2.** Cross-instance reputation portability — v2/v3.
- **NG3.** Cross-instance jury (a juror on instance A serving on a case from instance B) — v3.
- **NG4.** Per-community-per-peer trust (peer X trusted for community A but not B) — v2.
- **NG5.** Federation discovery beyond what Lemmy already provides — v2.
- **NG6.** SSRF-isolated media fetch worker — v2 per ADR-010.

---

## 2. Scope

### 2.1 In scope (v1)

| Area | What v1 ships |
|---|---|
| HTTP inbox routing | Governance type variants registered in Lemmy's existing `SharedInboxActivities` dispatcher (Phase 6 already does this); v1 wires the dispatcher's `receive` calls through trust-check + rate-limit + persist + log |
| Signature verification | Delegated to `activitypub_federation`'s existing HTTP-signature check (Lemmy code, trusted as-is per ADR-012); v1 adds **schema validation + size limits** on the activity payload after signature passes |
| Peer-trust state | New `federation_peer` table; four trust states (`Unknown` / `Allowlisted` / `Blocklisted` / `Untrusted-Receive`); admin-dashboard CRUD via the v1 admin config endpoint surface (OQ-018) |
| Inbound classify-and-route | Each AP type → its `receive_remote_*` handler (Phase 6 ships these as direct functions; v1 plumbs them behind the inbox dispatcher) |
| Storage isolation | All inbound rows carry `local_case_id = NULL` per ADR-006; new columns capture peer trust at receipt, admin review state |
| Admin review surface | 3 new REST endpoints — list, cross-link, dismiss — under `/api/v4/governance/admin/federation/*` |
| Rate limiting | Per-peer (default 100/h) and per-actor-across-peers (default 10/h on attestations); dashboard-tunable |
| Abuse defences | Storage cap with oldest-drop policy; replay nonce table; malformed-payload reject at boundary |
| Audit | New `federation_inbox_dropped_log` table + governance_log entries for accept/drop/block events |

### 2.2 Out of scope (deferred)

| Area | Deferred to | Why |
|---|---|---|
| Auto-apply of received sanctions | v3 | ADR-006; user-confirmed framing 2026-04-19 |
| Reputation portability across instances | v2/v3 | New attestation type + cross-instance pseudonym mapping; out of v1's "store + surface" scope |
| Cross-instance jury participation | v3 | Requires identity bridging beyond `actor_pseudonym` |
| Per-community-per-peer trust | v2 | Schema cost (composite key) + dashboard UX cost; v1 keeps it per-instance |
| OPA-driven federation policy | v2 (per [05 §7.2](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md)) | OPA replaces hardcoded checks instance-wide; federation joins that migration |
| SSRF-isolated fetch worker | v2 (per [06 §2.5](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md)) | Separate process is its own work-stream |
| Federation discovery | v2 | Lemmy's existing instance discovery covers content; governance discovery layered on top |
| Vanilla Lemmy `Announce`-wrapped governance gateway | v2/v3 | Vanilla peers will never send these types per ADR-014; degraded-mode is a separate design |

---

## 3. AP types accepted on the inbox

### 3.1 What Phase 6 ships (verify before v1 implementation)

Per the Phase 6 plan §"Files to Change" and tasks 72/73, Phase 6 ships:

| Type | Phase 6 outbound | Phase 6 receive function | Phase 6 inbox wiring |
|---|---|---|---|
| `SanctionNoticeObject` (wrapped in `PublishSanctionNotice` Create activity) | YES (`send_local_sanction_notice` task 74) | YES (`receive_remote_sanction_notice` task 75) called from `Activity::receive` impl | Registered in `SharedInboxActivities` enum (task 73) — **so HTTP inbox already dispatches to it once Phase 6 lands** |
| `TrustAttestationObject` (wrapped in `PublishTrustAttestation`) | YES (`send_local_trust_attestation` task 74; not wired to any v0 endpoint) | YES (`receive_remote_trust_attestation` task 75) | Registered in `SharedInboxActivities` |
| `ModerationLabelObject` (wrapped in `PublishLabel`) | Stub in v0 (no outbound caller); type defined per task 73 | **Stub `receive` returning `Ok(())`** per Phase 6 task 75 GOTCHA | Registered in `SharedInboxActivities`; receive is a no-op |

**v1 alignment note (§15):** Phase 6's plan §11 "NOT Building" item #4 says "Federation rate-limiting per source" is v1 — confirming v1 owns the rate-limit work. Item #6 says "HTTP route for admin review of `remote_sanction_notice`" is v1 — confirming v1 owns the admin endpoints. **Phase 6 already wires HTTP dispatch via `SharedInboxActivities`**; v1 does *not* re-route — v1 wraps the existing `receive` calls in trust-check + rate-limit + log + admin-review surface.

### 3.2 What v1 changes per type

**SanctionNotice (live HTTP path, v1):**
1. Lemmy's `shared_inbox` (`crates/apub/apub/src/http/mod.rs:44-60`) receives the POST.
2. `receive_activity_with_hook::<SharedInboxActivities, …>` runs HTTP-signature verification (Lemmy's existing path, no v1 change).
3. Dummy hook fires (`ReceivedActivity::create` already dedupes by `activity.id()` — Phase 6 task 75 GOTCHA confirmed).
4. Activity is dispatched by serde-untagged match to `PublishSanctionNotice::receive`.
5. **NEW v1 wrapper inside `receive`:**
   - Call `federation_inbox_check_peer_trust(activity.actor.domain(), &mut conn)` → returns trust state (or `Unknown` for first-seen).
   - If `Blocklisted`: write `federation_inbox_dropped_log` row + governance_log `federation_inbound_blocked`; return `LemmyResult::Err` so the inbox returns 403.
   - Call `federation_inbox_check_rate_limit(peer, "sanction_notice", &context)` → 429 on exceeded.
   - Call `federation_inbox_check_replay(activity.id(), &mut conn)` → 409 on duplicate within nonce window.
   - Call Phase 6's `receive_remote_sanction_notice` (unchanged from Phase 6).
   - The handler now writes `peer_trust_level_at_receipt` into the new column on `remote_sanction_notice` (v1 column addition).
   - governance_log: `federation_sanction_received` (Phase 6 already writes this).
6. Return 202 Accepted (advisory store, no immediate action promised).

**TrustAttestation (live HTTP path, v1):** identical wrapping; per-actor rate limit (default 10/h) applies *in addition to* per-peer rate limit because attestation spam from many peers about a single target is the abuse vector ([06 §4.7](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md)).

**ModerationLabel (NEW handler in v1):** Phase 6 ships `PublishLabel::receive` as a stub (`Ok(())`). v1 fills it with a `receive_remote_moderation_label` function mirroring `receive_remote_sanction_notice`'s shape, writing into a v1-new advisory table (see §8). `local_case_id = NULL`. governance_log: `federation_label_received` (new entry kind; const string in `governance_log.rs`).

### 3.3 Per-handler invariants

For all three handlers v1 enforces:

- **No call into local sanction/removal code paths.** ADR-006. Verified by lint pattern in §12 ("no auto-apply guard").
- **`local_case_id = NULL` at insert.** Cross-link is admin-driven (§6).
- **Schema validation rejects unknown enum variants.** `SanctionAction`, `SanctionScope`, `AttestationType` use `diesel-derive-enum`; deserialisation fails closed if a peer sends a variant we don't recognise. Returns 400.
- **Size limit on payload.** Enforce `serde_json::Value` size cap (default 64 KiB for sanction notices, 8 KiB for attestations) before persist. Reject 413.
- **Pseudonym = None on log entry.** Phase 6 task 75 confirms: actor is remote, no local pseudonym exists, so `actor_pseudonym = None` in `governance_log::append`. v1 keeps this.

---

## 4. Peer trust state model

### 4.1 Schema

New table `federation_peer` (migration in §8):

```sql
CREATE TABLE federation_peer (
  instance_id     INT PRIMARY KEY REFERENCES instance(id) ON DELETE CASCADE,
  trust_level     federation_peer_trust_enum NOT NULL DEFAULT 'unknown',
  added_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  added_by_actor  TEXT,                                    -- pseudonym of admin who set the state, NULL for system-set
  notes           JSONB NOT NULL DEFAULT '{}',
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TYPE federation_peer_trust_enum AS ENUM (
  'unknown',
  'allowlisted',
  'untrusted_receive',
  'blocklisted'
);

CREATE INDEX idx_federation_peer_trust ON federation_peer (trust_level);
```

`instance_id` is Lemmy's existing `instance.id` newtype — every federated peer Lemmy has ever heard of already gets a row in `instance` (created via `Instance::read_or_create`). v1's `federation_peer` is a **governance overlay** on that table — one row per peer the governance plane has formed an opinion about. Peers Lemmy knows about but governance hasn't seen yet have no `federation_peer` row; the trust-check helper treats absent rows as `Unknown`.

### 4.2 Trust states

| State | Meaning | Inbound governance signals | Outbound governance signals (Phase 6 already ships outbound; v1 leaves it `to_all_instances`) | Default for new peers |
|---|---|---|---|---|
| `Unknown` | Lemmy has seen this peer (instance row exists), governance hasn't formed an opinion yet | Accepted; `peer_trust_level_at_receipt = unknown` recorded; surfaced in admin review with prominent "first contact" flag | Sent (v1 keeps Phase 6's `to_all_instances`; v2 narrows) | **DEFAULT** unless dashboard sets a different `federation.inbound.default_trust_for_new_peers` |
| `Allowlisted` | Admin has explicitly trusted this peer | Accepted; standard surfacing in admin review | Sent | Set by admin only |
| `Untrusted-Receive` | Admin allows inbound for visibility but flags as low-trust | Accepted but flagged in admin review with `requires_admin_attention = true` | Sent | Optional dashboard default for new peers in tighter postures |
| `Blocklisted` | Admin has decided this peer is hostile | **Rejected — HTTP 403 + log to `federation_inbox_dropped_log`** | Sent (v1; v2 may add per-peer outbound block) | Set by admin only |

**v0/Phase 6 alignment (§15):** Phase 6 §11 NOT-Building item #1 — "v0 treats every federated peer as `Allow`" — translates to `Unknown` in v1's terminology. The trust-check helper returning `Unknown` for absent rows produces v0-equivalent behaviour by default (accept everything), which means **v1 inbound is not breaking anything Phase 6 establishes** even if no admin ever touches the peer table.

### 4.3 Configurability

The dashboard knob `federation.inbound.default_trust_for_new_peers` (`enum: 'unknown' | 'untrusted_receive' | 'blocklisted'`, default `'unknown'`) is read by the trust-check helper when it encounters a peer with no `federation_peer` row. Per the user's framing, community groups can experiment — open communities default `'unknown'` (accept-and-surface), tighter communities default `'untrusted_receive'` or `'blocklisted'`.

`federation_peer` rows are CRUD-ed via the v1 admin config endpoint surface (OQ-018) under a federation-specific sub-path: `POST /api/v4/governance/admin/federation/peers/{instance_id}/trust { trust_level, notes }`. Each change emits a governance_log entry `federation_peer_trust_changed` per [06 §2.4](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) "trust state change" — high-risk action, behind the standard step-up-auth gate v1 introduces alongside OQ-018.

### 4.4 Per-instance only in v1

v1 keeps trust **per-peer-instance**, not per-peer-per-community. Per-community trust is v2 because:

- Schema cost (composite key on `federation_peer`).
- Dashboard UX cost (matrix instead of list).
- v1 needs operator data to know whether per-community is even desired before locking the schema.

If a community wants to opt out of receiving from a peer the instance is `Allowlisting`, they can flag inbound rows in their community's review queue as ignored — surfaced via the `admin_action` enum (§8) — but the peer-level trust remains instance-wide.

---

## 5. HTTP inbox routing

### 5.1 What exists pre-v1

Phase 6 task 73 registers the three governance variants in `SharedInboxActivities`:

```rust
// crates/apub/activities/src/activity_lists.rs (post-Phase 6)
pub enum SharedInboxActivities {
    Follow(Follow),
    // ... existing variants ...
    PublishSanctionNotice(PublishSanctionNotice),     // Phase 6 task 73
    PublishTrustAttestation(PublishTrustAttestation), // Phase 6 task 73
    PublishLabel(PublishLabel),                        // Phase 6 task 73 (stub receive)
    RawAnnouncableActivities(RawAnnouncableActivities),  // catch-all, must remain last
}
```

`shared_inbox` (`crates/apub/apub/src/http/mod.rs:44-60`) calls `receive_activity_with_hook::<SharedInboxActivities, …>` which: (a) runs HTTP-signature verification, (b) parses the body via untagged serde dispatch, (c) calls the matched variant's `Activity::verify` then `Activity::receive`, (d) writes `ReceivedActivity` for dedup. **Lemmy's HTTP plumbing is unchanged in v1.**

### 5.2 What v1 adds

A thin **inbound wrapper** invoked from each governance `Activity::receive` impl, layered as:

```text
shared_inbox HTTP entry
    │
    ├─ activitypub_federation: HTTP signature verify (Lemmy code; v1 unchanged)
    ├─ Dummy::hook → ReceivedActivity::create (dedup; v1 unchanged)
    │
    └─ untagged serde dispatch → Activity::receive (governance variant)
            │
            └─ NEW v1: wrap_governance_inbound(activity, context, |a, c| {
                    federation_inbox_check_peer_trust(a, c)?       // 403 on Blocklisted
                    federation_inbox_check_size(a, c)?             // 413 on oversize
                    federation_inbox_check_schema(a, c)?           // 400 on schema fail
                    federation_inbox_check_rate_limit(a, c)?       // 429 on exceeded
                    federation_inbox_check_replay(a, c)?           // 409 on replay
                    Phase6::receive_remote_<type>(a, c).await?     // Phase 6 inserts row + logs
                    Ok(())
                }).await
```

**File location for the wrapper:** `crates/apub/apub/src/governance/inbox.rs` (Phase 6 task 75 already creates this file). v1 adds the wrapper alongside Phase 6's `receive_remote_*` functions in the same file.

**The Activity::receive impls (one per governance variant) call into the wrapper** — they remain in `crates/apub/activities/src/governance/publish_*.rs` per Phase 6's layout but their body becomes a single `wrap_governance_inbound` call.

### 5.3 Failure modes and HTTP responses

| Stage | Failure mode | HTTP response | Audit |
|---|---|---|---|
| Sig verify (Lemmy code) | Bad signature | 401 (Lemmy default) | Lemmy logs; no governance_log entry |
| `wrap_governance_inbound` peer-trust | `Blocklisted` peer | 403 | `federation_inbox_dropped_log` row + governance_log `federation_inbound_blocked` |
| `wrap_governance_inbound` size | Payload > cap | 413 | `federation_inbox_dropped_log` row + governance_log `federation_inbound_dropped_oversize` |
| `wrap_governance_inbound` schema | Unknown enum / missing field | 400 | `federation_inbox_dropped_log` row + governance_log `federation_inbound_dropped_schema` |
| `wrap_governance_inbound` rate-limit | Per-peer or per-actor exceeded | 429 | `federation_inbox_dropped_log` row + governance_log `federation_inbound_dropped_rate_limit` |
| `wrap_governance_inbound` replay | Activity ID already seen in nonce window | 409 | (Lemmy's `ReceivedActivity` dedup catches most replays at HTTP layer; v1's check is the application-level safeguard for cases where activity-id is reused with different content) |
| Phase 6's `receive_remote_*` | DB error during insert | 500 | Standard Lemmy error path; governance_log `federation_inbound_persist_failed` (new entry kind) |
| Success | Row inserted, log written | 202 Accepted | governance_log `federation_*_received` (Phase 6 entry kinds) |

**Why 202, not 200:** Per HTTP semantics, 202 means "accepted for processing, no commitment to action." This communicates the v1 advisory-only contract to peers. ActivityPub spec accepts any 2xx; 202 is informational.

### 5.4 Backward compatibility with Phase 6's direct-call test

Phase 6's `tests/e2e.rs::sanction_notice_round_trip` calls `PublishSanctionNotice::receive(activity, &context_b).await` **directly** (not through HTTP). After v1's wrapper lands, this call now goes through the trust/rate/replay checks. v1 must update the test (or provide a `receive_remote_sanction_notice_unchecked` for tests) so the existing round-trip test continues to pass — likely by setting `instance-a.test` to `Allowlisted` in the test fixture before calling receive. v1 chooses the fixture-update path; do not introduce an unchecked variant (it's a footgun).

---

## 6. Admin review surface

Three new REST endpoints under `/api/v4/governance/admin/federation/*`. Authz: instance admin only in v1 (matching OQ-018's v1 scope: "instance-admin-only in v1, ... community-scoped admin writes are v1+").

### 6.1 `GET /api/v4/governance/admin/federation/inbox`

Paginated list of advisory inbound rows across all three types.

Request:
```http
GET /api/v4/governance/admin/federation/inbox?type=sanction_notice&peer=peer.example&trust_level=untrusted_receive&age_days=7&page=1&limit=50
```

Filters (all optional, query params):
- `type`: `sanction_notice` | `trust_attestation` | `moderation_label` | (omit for all)
- `peer`: instance domain or `instance_id`
- `trust_level`: `unknown` | `allowlisted` | `untrusted_receive` | `blocklisted` (filters by `peer_trust_level_at_receipt`)
- `age_days`: `i32` (0 = today only, 7 = last week, etc.)
- `admin_action`: `unreviewed` | `cross_linked` | `dismissed` (default `unreviewed`)
- `page`, `limit` (match Phase 4's `list_modlog` pagination pattern)

Response (DTO `ListFederationInboxResponse` in `api_common`):
```rust
pub struct FederationInboxRow {
    pub kind: FederationInboxKind,                    // sanction | attestation | label
    pub row_id: i32,                                   // FK to remote_sanction_notice / federation_attestation / remote_moderation_label
    pub peer_instance: String,
    pub peer_trust_level_at_receipt: FederationPeerTrust,
    pub current_peer_trust_level: FederationPeerTrust, // may differ if admin re-trusted post-receipt
    pub target_url: String,
    pub action: Option<SanctionAction>,                // Some only for sanction_notice
    pub scope: Option<SanctionScope>,                  // Some only for sanction_notice
    pub attestation_type: Option<AttestationType>,     // Some only for trust_attestation
    pub label: Option<String>,                         // Some only for moderation_label
    pub summary_redacted: String,                      // truncated for list view; full payload via separate endpoint if needed
    pub published_at: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
    pub admin_reviewed_at: Option<DateTime<Utc>>,
    pub admin_action: Option<FederationInboxAdminAction>,  // None | CrossLinked(case_id) | Dismissed
    pub local_case_id: Option<i32>,                    // populated only after cross-link
    pub requires_admin_attention: bool,                // computed: trust_level in (unknown, untrusted_receive) AND admin_reviewed_at IS NULL
}

pub struct ListFederationInboxResponse {
    pub rows: Vec<FederationInboxRow>,
    pub total_unreviewed: i64,
    pub total_pending_attention: i64,
}
```

**View crate:** `crates/db_views/federation_inbox/` (new) — UNION query across the three advisory tables, projecting into the unified `FederationInboxRow` shape. Per Phase 2a's `view-crate-selectable-template.md` precedent: each backing table has its own `Selectable` derive; the view assembles them via 3 separate queries into a `Vec<FederationInboxRow>` (UNION-ALL in SQL is also acceptable but harder to debug — start with three separate queries).

### 6.2 `POST /api/v4/governance/admin/federation/inbox/{kind}/{id}/cross-link`

Admin links an advisory row to a local case.

Request:
```json
{
  "local_case_id": 1234,
  "rationale": "Same target user; their peer's jury reached the same finding."
}
```

Behaviour:
- Validate `local_case_id` exists and admin has access.
- `UPDATE remote_sanction_notice SET local_case_id = $1, admin_reviewed_at = NOW(), admin_action = 'cross_linked' WHERE id = $2` (and analogues for the other two types).
- Insert a `case_evidence` row (per [04 §3](../../docs/brehon-law-inspired-network/04-data-model-and-api.md)) linking the case to the advisory notice with `visibility = JuryOnly` (default; admin can set `PublicRedacted` if appropriate).
- governance_log: `federation_inbound_cross_linked` (new entry kind).
- Step-up auth: NOT required in v1 — cross-linking advisory evidence to a case is lower-risk than trust-state change (the jury still decides). Document in §12 security row.

### 6.3 `POST /api/v4/governance/admin/federation/inbox/{kind}/{id}/dismiss`

Admin marks the advisory row as reviewed-no-action.

Request:
```json
{
  "rationale": "Peer is known low-trust; this notice does not warrant action."
}
```

Behaviour:
- `UPDATE … SET admin_reviewed_at = NOW(), admin_action = 'dismissed', dismissal_rationale = $1`.
- governance_log: `federation_inbound_dismissed` (new entry kind).
- Row is **not deleted** — audit trail preserved. Filtered out of default `unreviewed` list view.

### 6.4 What v1 does NOT ship (UI)

The dashboard UI for the admin review surface is **out of v1 scope** per ADR-010 v3 ("Admin dashboard UX"). v1 ships only the REST endpoints + DTOs. The user's framing — "the admin dashboard will allow community groups to experiment" — is satisfied by **the API existing and being callable**; the dashboard frontend is v3 (or whenever the dashboard track ships, which is parallel to v1/v2/v3 per v0's CLAUDE.md "no UI in v0/v1 backend slice" rule).

---

## 7. Rate limiting and abuse defence

### 7.1 Per-peer rate limit

**Default:** 100 inbound governance activities per hour per peer instance (sliding window).

**Storage:** in-memory rolling counter keyed by `(instance_id, hour_bucket)` with periodic flush to a `federation_inbox_rate_log` derived table for observability. Use the same in-process rate-limit pattern Lemmy already has at `crates/api_utils/src/rate_limit/*` — extend rather than fork.

**Tunable via dashboard:** `federation.inbound.per_peer_rate_per_hour` (`int`, range `[1, 100000]`, default `100`).

**Trigger:** at the 101st inbound from the same peer in a sliding hour, return HTTP 429, write `federation_inbox_dropped_log` row + governance_log `federation_inbound_dropped_rate_limit`.

### 7.2 Per-actor rate limit on attestations

**Why:** [06 §4.7](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) — fake federated attestations are a named threat. The vector isn't one peer flooding (caught by §7.1); it's **many peers each sending one attestation about the same target actor** (e.g. 50 peers say "user X is trusted" in a coordinated push to influence our jury pool).

**Default:** 10 attestations per hour about the same `subject_url`, summed across all peers.

**Storage:** in-memory keyed by `(subject_url_hash, hour_bucket)`. `subject_url_hash` because keying by raw URL leaks pseudonym structure into the rate-limit table (small concern, but cheap to avoid).

**Tunable:** `federation.inbound.per_actor_attestation_rate_per_hour` (`int`, range `[1, 10000]`, default `10`).

**Trigger:** 11th attestation about same subject in window → 429, log `federation_inbound_dropped_actor_rate_limit`.

### 7.3 Storage cap

**Why:** even a peer below the rate limit can fill our DB over months (100/h × 24 × 30 = 72,000 rows/peer/month). Need a backstop.

**Default cap:** 10,000 rows per peer per advisory table (`remote_sanction_notice`, `federation_attestation`, `remote_moderation_label`).

**Mechanism:** at insert time, check `SELECT COUNT(*) FROM remote_sanction_notice WHERE source_instance = $1 AND admin_reviewed_at IS NULL`. If ≥ cap, drop the oldest unreviewed row for that peer with a `federation_inbox_dropped_log` entry + governance_log `federation_inbound_storage_cap_evicted`. Reviewed rows (cross-linked or dismissed) are not counted toward the cap — they're audit-preserved.

**Tunable:** `federation.inbound.per_peer_storage_cap` (`int`, range `[100, 1000000]`, default `10000`).

**Why oldest-drop, not reject:** rejecting at the cap means a hostile peer can prevent us from ever seeing newer notices from a legitimate peer once we're at cap. Oldest-drop with eviction log preserves recency at the cost of historical depth.

### 7.4 Schema-fuzzing defence

- **Payload size cap** (per §3.3): default 64 KiB for sanction notices, 8 KiB for attestations/labels. Configurable via `federation.inbound.max_payload_bytes_<type>`.
- **Strict deserialisation:** per Phase 6 task 75, schema validation happens at the serde layer; unknown enum variants for `SanctionAction`/`SanctionScope`/`AttestationType` cause `serde_json::from_value` to fail. v1 adds `#[serde(deny_unknown_fields)]` on the protocol structs in `crates/apub/objects/src/protocol/governance/*.rs` to reject hostile field injections.
- **String length caps** on `summary` fields: 8000 chars per Phase 6's existing convention; oversize → 413.

### 7.5 Replay protection

**Why:** ActivityPub HTTP signatures don't include a nonce by default. A peer can replay a signed activity verbatim. Lemmy's `ReceivedActivity::create` (called by Phase 6's `Dummy::hook` at `crates/apub/apub/src/http/mod.rs:73-74`) already dedupes by `activity.id()` for the lifetime of the row, but — depending on retention — old IDs could be replayed after eviction.

**v1 mechanism:** dedicated `federation_inbox_nonce` table (migration in §8) with `(peer_instance, activity_id)` UNIQUE constraint and `seen_at` timestamp. Window: configurable, default 7 days. A periodic cron (every hour) deletes rows older than the window. Within window, duplicate insert returns 409 Conflict.

**Tunable:** `federation.inbound.replay_window_days` (`int`, range `[1, 30]`, default `7`).

---

## 8. Dependencies, database & migration changes

### 8.0 Dependencies (hard gate — Phase 6 merge)

**Per B5 resolution 2026-04-19**, v1-federation-inbound implementation is hard-gated on Phase 6 PR merge to `governance-v0`. Plan-level review (this PRD) can proceed in parallel, but no v1 migration, handler, or route change lands until all preconditions below are satisfied. This makes the ordering fail-loud (the §8.2 SQL `DO` preamble `RAISE EXCEPTION`s if the migration runs first).

**Preconditions — all must be true at v1 `/prp-plan` time:**

1. **Phase 6 PR merged to `governance-v0`.** Verify via `git log governance-v0` or by checking that Phase 6's commit SHA is an ancestor of the current `governance-v0` HEAD.
2. **Phase 6 tables exist.** `federation_attestation` and `remote_sanction_notice` must be present with all columns Phase 6 creates (task 70). The §8.2 DO block asserts this.
3. **Phase 6 `received_at` column on `remote_sanction_notice`.** Carry-forward DQ-6.1 must be resolved by landing `received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`. v1 relies on this column for §6.1 admin-review queries' `age_days` filter. If DQ-6.1 landed differently, v1 must re-plan.
4. **Phase 6 enum types registered.** `attestation_type_enum`, `sanction_action_enum`, `sanction_scope_enum` must exist as Postgres types (Phase 6 task 70 creates them). v1 extends `federation_attestation`'s shape by adding columns, but the enum columns Phase 6 defines are assumed present.
5. **Phase 6 `governance_log::append` constants present.** String literals `"federation_sanction_received"` and `"federation_attestation_received"` must exist in `crates/api/api/src/governance/governance_log.rs` (Phase 6 task 75). v1 adds further constants on top of these (see §15.1 deliverable table).
6. **Schema assumption — `remote_sanction_notice.source_instance` is TEXT (peer domain).** Phase 6's design choice is a TEXT column holding the peer's domain; it is NOT an FK to `instance.id`. v1's `federation_peer` overlay table joins to it via `JOIN instance ON instance.domain = remote_sanction_notice.source_instance`. Case-sensitivity of this join must be verified on the first v1 integration test. If Phase 6 ships `source_instance` as an FK instead, v1 §4.1 + §8.2 need re-planning.
7. **Migration timestamp invariant.** The v1 migration (`add_federation_inbound_v1`) must sort **after** Phase 6's `add_federation_attestations` (i.e. a later ISO-timestamp prefix). The specific timestamp slot is assigned at v1 implementation time, not reserved here.

**If ANY precondition is not satisfied at v1 `/prp-plan` time, stop.** Do not run the §8.2 migration against an unmet preconditions state — the DO block preamble will `RAISE EXCEPTION` and the migration will abort cleanly, but re-planning is still required to adjust to whatever Phase 6 actually shipped.

**Partial-collateral:** this preconditions list graduates N5 (the FK-vs-domain-string mismatch first surfaced in the §15.3 alignment notes) from "noted — revisit at v1-impl" to "resolved at v1-impl time via explicit precondition #6." A reviewer who disagrees with the TEXT-not-FK assumption must push back at PR review, not silently at migration-time.

### 8.1 Tables Phase 6 creates (verified before v1)

Per Phase 6 task 70:

```sql
-- Phase 6 ships:
CREATE TABLE federation_attestation (
    id SERIAL PRIMARY KEY,
    actor_url TEXT NOT NULL,
    subject_url TEXT NOT NULL,
    attestation_type attestation_type_enum NOT NULL,
    valid_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    signature TEXT NOT NULL
);

CREATE TABLE remote_sanction_notice (
    id SERIAL PRIMARY KEY,
    source_instance TEXT NOT NULL,
    target_url TEXT NOT NULL,
    action sanction_action_enum NOT NULL,
    scope sanction_scope_enum NOT NULL,
    summary TEXT NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    signature TEXT NOT NULL,
    local_case_id INT REFERENCES moderation_case(id) ON DELETE SET NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

`source_instance` on `remote_sanction_notice` is TEXT (peer domain), not an FK to `instance.id`. **v1 alignment note (§15):** this is a Phase 6 design choice that simplifies the inbound path (no need to resolve domain → instance_id at receive time) but means v1's `federation_peer` join needs `JOIN instance ON instance.domain = remote_sanction_notice.source_instance` — verify on first v1 test that Lemmy's `instance.domain` matches AP-actor host case-sensitivity.

### 8.2 v1 migration: `add_federation_inbound_v1`

**Timestamp:** must sort after Phase 6's `add_federation_attestations`. The specific timestamp slot is assigned at v1 implementation time (do NOT pre-reserve it here — Phase 6's ordering within `governance-v0` is still in flight at PRD time).

**Preamble — fail-loud Phase 6 precondition check (per §8.0 hard gate).** The DO block below `RAISE EXCEPTION`s if Phase 6's tables or the DQ-6.1 `received_at` column are not present. This makes accidental out-of-order execution abort cleanly instead of silently creating columns on tables the handler path will never read.

```sql
-- Preamble: verify Phase 6 preconditions present before ALTER.
-- This DO block enforces preconditions #2, #3, #4, #6 from §8.0.
-- Preconditions #1 (Phase 6 merge) and #7 (migration timestamp ordering) are
-- NOT SQL-enforceable — #1 is a `git log` check run at `/prp-plan` time and #7
-- is verified by the implementer naming the migration directory with a
-- timestamp prefix that sorts after `add_federation_attestations`. Migration
-- ordering inside a single `diesel migration run` is by directory-name sort,
-- so a correct timestamp prefix guarantees Phase 6's tables exist when this
-- DO block runs. Precondition #5 (`governance_log::append` const strings) is
-- Rust-only and is enforced at compile time by `use` statements in the v1
-- handler source — there is nothing to check in SQL.
DO $$
BEGIN
  -- Precondition #2: Phase 6 base tables
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.tables
    WHERE table_name = 'federation_attestation'
  ) THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 federation_attestation table (not present)';
  END IF;
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.tables
    WHERE table_name = 'remote_sanction_notice'
  ) THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 remote_sanction_notice table (not present)';
  END IF;
  -- Precondition #3: DQ-6.1 received_at column on remote_sanction_notice
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name = 'remote_sanction_notice' AND column_name = 'received_at'
  ) THEN
    RAISE EXCEPTION 'federation-inbound-v1 assumes Phase 6 DQ-6.1 resolved with remote_sanction_notice.received_at column (not present)';
  END IF;
  -- Precondition #4: Phase 6 enum types registered
  IF NOT EXISTS (
    SELECT 1 FROM pg_type WHERE typname = 'attestation_type_enum'
  ) THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 attestation_type_enum (not registered)';
  END IF;
  IF NOT EXISTS (
    SELECT 1 FROM pg_type WHERE typname = 'sanction_action_enum'
  ) THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 sanction_action_enum (not registered)';
  END IF;
  IF NOT EXISTS (
    SELECT 1 FROM pg_type WHERE typname = 'sanction_scope_enum'
  ) THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 sanction_scope_enum (not registered)';
  END IF;
  -- Precondition #6: remote_sanction_notice.source_instance is TEXT (not FK).
  -- v1's federation_peer JOIN requires this — if Phase 6 ships it as an INT
  -- FK to instance(id), v1 §4.1 + §8.2 need re-planning.
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name = 'remote_sanction_notice'
      AND column_name = 'source_instance'
      AND data_type = 'text'
  ) THEN
    RAISE EXCEPTION 'federation-inbound-v1 assumes remote_sanction_notice.source_instance is TEXT (peer domain); found different type. See §8.0 precondition #6 and §15.3.';
  END IF;
END
$$;

-- New trust-state enum
CREATE TYPE federation_peer_trust_enum AS ENUM (
  'unknown',
  'allowlisted',
  'untrusted_receive',
  'blocklisted'
);

-- New admin-action enum for advisory-row review state
CREATE TYPE federation_inbox_admin_action_enum AS ENUM (
  'unreviewed',
  'cross_linked',
  'dismissed'
);

-- 1. Peer-trust overlay table
CREATE TABLE federation_peer (
  instance_id     INT PRIMARY KEY REFERENCES instance(id) ON DELETE CASCADE,
  trust_level     federation_peer_trust_enum NOT NULL DEFAULT 'unknown',
  added_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  added_by_actor  TEXT,
  notes           JSONB NOT NULL DEFAULT '{}'::JSONB,
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_federation_peer_trust ON federation_peer (trust_level);

-- 2. Drop-log for audit
CREATE TABLE federation_inbox_dropped_log (
  id              BIGSERIAL PRIMARY KEY,
  source_instance TEXT NOT NULL,
  activity_id     TEXT,
  drop_reason     TEXT NOT NULL,                  -- 'blocklisted' | 'oversize' | 'schema' | 'rate_limit_peer' | 'rate_limit_actor' | 'replay' | 'storage_cap_evicted'
  payload_excerpt TEXT,                            -- first 256 chars of body for diagnostic; NULL on auto-evict
  dropped_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_fil_drop_source ON federation_inbox_dropped_log (source_instance, dropped_at DESC);
CREATE INDEX idx_fil_drop_reason ON federation_inbox_dropped_log (drop_reason, dropped_at DESC);

-- 3. Replay-protection nonce table
CREATE TABLE federation_inbox_nonce (
  peer_instance   TEXT NOT NULL,
  activity_id     TEXT NOT NULL,
  seen_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (peer_instance, activity_id)
);
CREATE INDEX idx_fin_nonce_seen_at ON federation_inbox_nonce (seen_at);

-- 4. ModerationLabel advisory table (Phase 6 ships type but no table; v1 adds it)
CREATE TABLE remote_moderation_label (
  id              SERIAL PRIMARY KEY,
  source_instance TEXT NOT NULL,
  actor_url       TEXT NOT NULL,
  target_url      TEXT NOT NULL,
  label           TEXT NOT NULL,
  summary         TEXT,
  published_at    TIMESTAMPTZ NOT NULL,
  signature       TEXT NOT NULL,
  local_case_id   INT REFERENCES moderation_case(id) ON DELETE SET NULL,
  received_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  peer_trust_level_at_receipt federation_peer_trust_enum NOT NULL DEFAULT 'unknown',
  admin_reviewed_at TIMESTAMPTZ,
  admin_action    federation_inbox_admin_action_enum NOT NULL DEFAULT 'unreviewed',
  dismissal_rationale TEXT
);
CREATE INDEX idx_rml_target ON remote_moderation_label (target_url);
CREATE INDEX idx_rml_source ON remote_moderation_label (source_instance, received_at DESC);
CREATE INDEX idx_rml_admin_action ON remote_moderation_label (admin_action) WHERE admin_action = 'unreviewed';

-- 5. Add v1 columns to Phase 6's tables
ALTER TABLE remote_sanction_notice
  ADD COLUMN peer_trust_level_at_receipt federation_peer_trust_enum NOT NULL DEFAULT 'unknown',
  ADD COLUMN admin_reviewed_at TIMESTAMPTZ,
  ADD COLUMN admin_action federation_inbox_admin_action_enum NOT NULL DEFAULT 'unreviewed',
  ADD COLUMN dismissal_rationale TEXT;
CREATE INDEX idx_rsn_admin_action ON remote_sanction_notice (admin_action) WHERE admin_action = 'unreviewed';

ALTER TABLE federation_attestation
  ADD COLUMN source_instance TEXT,                 -- nullable: outbound rows have NULL; inbound rows populated
  ADD COLUMN received_at TIMESTAMPTZ,              -- nullable: outbound rows have NULL
  ADD COLUMN peer_trust_level_at_receipt federation_peer_trust_enum,
  ADD COLUMN admin_reviewed_at TIMESTAMPTZ,
  ADD COLUMN admin_action federation_inbox_admin_action_enum NOT NULL DEFAULT 'unreviewed',
  ADD COLUMN dismissal_rationale TEXT;
CREATE INDEX idx_fa_admin_action ON federation_attestation (admin_action) WHERE admin_action = 'unreviewed';
CREATE INDEX idx_fa_source ON federation_attestation (source_instance, received_at DESC) WHERE source_instance IS NOT NULL;
```

**down.sql:** Drop in reverse order. The columns added to `federation_attestation` and `remote_sanction_notice` are reversible because Phase 6's tables are empty pre-v1 (no production data yet), so column-drop is safe.

### 8.3 Backfill

Phase 6's tables are empty pre-v1 (the round-trip test inserts and tears down). No data backfill needed. New columns default to `'unknown'` / `'unreviewed'` so any Phase 6 test rows that survive into v1 land in a consistent state.

### 8.4 Diesel models

Per Phase 6's pattern, mirror the existing `crates/db_schema/src/source/governance/sanction.rs` shape:

| New model file | Purpose |
|---|---|
| `crates/db_schema/src/source/governance/federation_peer.rs` | `FederationPeer`, `FederationPeerInsertForm`, `FederationPeerUpdateForm` |
| `crates/db_schema/src/source/governance/federation_inbox_dropped_log.rs` | Insert-only model |
| `crates/db_schema/src/source/governance/federation_inbox_nonce.rs` | Insert-only + period-cleanup query |
| `crates/db_schema/src/source/governance/remote_moderation_label.rs` | `RemoteModerationLabel`, `RemoteModerationLabelInsertForm`, `RemoteModerationLabelUpdateForm` |

Plus extending Phase 6's models in `remote_sanction_notice.rs` and `federation_attestation.rs` with new columns + an `UpdateForm` (Phase 6 ships insert-only forms).

Newtypes in `crates/db_schema/src/newtypes.rs`: `FederationPeerId(pub i32)` (just `instance_id` reuse acceptable; create newtype only if Diesel benefits), `FederationInboxDroppedLogId(pub i64)`, `RemoteModerationLabelId(pub i32)`.

---

## 9. Handler shape

### 9.1 Module layout (extending Phase 6's `crates/apub/apub/src/governance/`)

```text
crates/apub/apub/src/governance/
├── mod.rs                  (Phase 6; re-export wiring)
├── inbox.rs                (Phase 6: receive_remote_*; v1 ADDS: wrap_governance_inbound, peer-trust + rate-limit + replay helpers)
├── outbox.rs               (Phase 6; v1 unchanged in scope, may consume federation_peer.trust_level for v2-prep)
├── verify.rs               (Phase 6 stub; v1 unchanged)
└── inbox_admin.rs          (NEW v1: handlers for the 3 admin endpoints)
```

The 3 admin endpoints are NOT in `crates/apub/`; they're standard governance handlers in `crates/api/api/src/governance/federation_inbox/`:

```text
crates/api/api/src/governance/federation_inbox/
├── mod.rs
├── list_inbox.rs
├── cross_link.rs
└── dismiss.rs
```

`crates/api/routes/src/governance.rs` adds the 3 routes.

### 9.2 Wrapper signature

```rust
// crates/apub/apub/src/governance/inbox.rs (v1 addition)

pub(crate) async fn wrap_governance_inbound<F, Fut, A>(
    activity: A,
    context: &Data<LemmyContext>,
    inner: F,
) -> LemmyResult<()>
where
    F: FnOnce(A, &Data<LemmyContext>) -> Fut,
    Fut: Future<Output = LemmyResult<()>>,
    A: GovernanceInboundActivity,                  // trait that exposes activity_id, actor_domain, payload_size_bytes
{
    let pool = &mut context.pool();
    let peer_domain = activity.actor_domain()?;
    let activity_id = activity.activity_id()?;

    // 1. Peer trust gate
    let trust = federation_inbox_check_peer_trust(&peer_domain, pool).await?;
    if trust == FederationPeerTrust::Blocklisted {
        log_inbox_drop(&peer_domain, &activity_id, "blocklisted", None, pool).await?;
        return Err(LemmyErrorType::FederationPeerBlocklisted.into());  // → HTTP 403
    }

    // 2. Size cap (per type)
    let size_cap = activity.payload_size_cap_bytes(&context.settings()?);
    if activity.payload_size_bytes()? > size_cap {
        log_inbox_drop(&peer_domain, &activity_id, "oversize", None, pool).await?;
        return Err(LemmyErrorType::FederationPayloadTooLarge.into());  // → HTTP 413
    }

    // 3. Schema validation happens at serde layer; serialise re-check is unnecessary

    // 4. Rate limit per peer
    federation_inbox_check_peer_rate_limit(&peer_domain, context).await?;

    // 5. Per-actor rate limit (only for attestations)
    activity.check_per_actor_rate_limit(context).await?;

    // 6. Replay protection
    federation_inbox_check_replay(&peer_domain, &activity_id, context).await?;

    // 7. Phase 6's domain handler
    inner(activity, context).await?;

    Ok(())
}
```

**Failure modes are exposed via new `LemmyErrorType` variants:**
- `FederationPeerBlocklisted` (403)
- `FederationPayloadTooLarge` (413)
- `FederationSchemaInvalid` (400)
- `FederationPeerRateLimitExceeded` (429)
- `FederationActorRateLimitExceeded` (429)
- `FederationActivityReplayed` (409)

These map to HTTP status codes via the existing Lemmy error → response mapper (extend the mapping if needed).

### 9.3 Per-handler patches

Each Phase 6 `Activity::receive` impl becomes:

```rust
// crates/apub/activities/src/governance/publish_sanction_notice.rs
async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    wrap_governance_inbound(self, context, |activity, ctx| async move {
        crate::governance::inbox::receive_remote_sanction_notice(activity, ctx).await
    }).await
}
```

(Same shape for `PublishTrustAttestation` and `PublishLabel`.)

### 9.4 New handler for `ModerationLabel`

```rust
// crates/apub/apub/src/governance/inbox.rs (v1 addition)
pub async fn receive_remote_moderation_label(
    activity: PublishLabel,
    context: &Data<LemmyContext>,
) -> LemmyResult<()> {
    let form = RemoteModerationLabelInsertForm {
        source_instance: activity.actor.inner().domain().unwrap_or_default().to_string(),
        actor_url: activity.actor.inner().to_string(),
        target_url: activity.object.target.to_string(),
        label: activity.object.label.clone(),
        summary: activity.object.summary.clone(),
        published_at: activity.object.published,
        signature: String::new(),  // DQ-FED-IN-1 carry-forward from Phase 6 task 75
        local_case_id: None,                                  // ADR-006
        peer_trust_level_at_receipt: /* from wrap_governance_inbound context */,
    };
    RemoteModerationLabel::create(&mut context.pool(), &form).await?;
    governance_log::append(
        &mut context.pool(),
        ENTRY_KIND_FEDERATION_LABEL_RECEIVED,
        json!({
            "source_instance": form.source_instance,
            "target_url": form.target_url,
            "label": form.label,
        }),
        None,
    ).await?;
    Ok(())
}
```

---

## 10. Defaults matrix

All knobs read from `governance_config` (Phase 5a's typed-column config table; OQ-018 ships the v1 admin write endpoint). Per the user framing — "defaults might have to be decided on" — the following are v1 starting points; communities can change them via the dashboard once OQ-018 ships, or via direct `UPDATE governance_config` until then.

| Knob | Namespace | Default | Range | Per-instance? | Citation |
|---|---|---|---|---|---|
| `federation.inbound.default_trust_for_new_peers` | `federation.inbound` | `'unknown'` | enum {unknown, untrusted_receive, blocklisted} | yes | §4.3 |
| `federation.inbound.per_peer_rate_per_hour` | `federation.inbound` | `100` | `[1, 100000]` | yes | §7.1 |
| `federation.inbound.per_actor_attestation_rate_per_hour` | `federation.inbound` | `10` | `[1, 10000]` | yes | §7.2 |
| `federation.inbound.per_peer_storage_cap` | `federation.inbound` | `10000` | `[100, 1000000]` | yes | §7.3 |
| `federation.inbound.max_payload_bytes_sanction_notice` | `federation.inbound` | `65536` | `[1024, 1048576]` | yes | §3.3 |
| `federation.inbound.max_payload_bytes_trust_attestation` | `federation.inbound` | `8192` | `[1024, 65536]` | yes | §3.3 |
| `federation.inbound.max_payload_bytes_moderation_label` | `federation.inbound` | `8192` | `[1024, 65536]` | yes | §3.3 |
| `federation.inbound.replay_window_days` | `federation.inbound` | `7` | `[1, 30]` | yes | §7.5 |
| `federation.inbound.replay_cleanup_cron_interval_minutes` | `federation.inbound` | `60` | `[5, 1440]` | yes | §7.5 |
| `federation.inbound.summary_max_chars` | `federation.inbound` | `8000` | `[256, 32000]` | yes | §7.4 |
| `federation.inbound.admin_review_default_filter_days` | `federation.inbound` | `7` | `[0, 365]` | yes | §6.1 |

Seed these rows in the migration alongside the schema changes per Phase 5a task 50's pattern (insert into `governance_config` with `scope = 'instance'`).

---

## 11. Backwards compatibility

### 11.1 Phase 6 outbound

**Unchanged in v1.** Outbound publish (`send_local_sanction_notice`, `send_local_trust_attestation`) continues to use `ActivitySendTargets::to_all_instances()` per Phase 6 task 74. v1 does not narrow outbound by `federation_peer.trust_level` — that's v2 work (it would require a new outbound-side filter pass that's a separate design discussion).

### 11.2 Existing peers

There are no existing federation peers as of pre-v1 — Phase 6 is in flight, no production deployment with governance-federation peers exists yet. If by v1 ship-date a Brehon instance has federated peers, they'll all be `Unknown` until an admin classifies them (see §4.2 default behaviour).

### 11.3 Vanilla Lemmy peers

Per ADR-014, vanilla Lemmy:
- **Sends** content activities only (posts, comments, votes, follows, etc.). They never send `PublishSanctionNotice` / `PublishTrustAttestation` / `PublishLabel` because they don't know about these types.
- **Receives** governance activities from us but ignores them (untagged serde dispatch falls through to `RawAnnouncableActivities` catch-all per Phase 6 task 73 GOTCHA).

**v1 implication:** a vanilla Lemmy peer never appears in `federation_peer` because the only path to creating a row is admin action (no first-contact governance activity from them ever fires). Content federation continues unchanged. The trust-check helper has no opinion about vanilla peers — they simply don't trigger governance-inbound code.

### 11.4 Phase 6's `sanction_notice_round_trip` test

v1's wrapper means the existing direct-call test now goes through trust-check + rate-limit + replay. v1 must update the test fixture to:
1. Insert `federation_peer` row for `instance-a.test` with `trust_level = 'allowlisted'` before calling receive.
2. Confirm test still passes.

This is a test-only change; no production code regression.

---

## 12. Security

### 12.1 Signature verification

Delegated to `activitypub_federation::actix_web::inbox::receive_activity_with_hook` (Lemmy's existing path). v1 does not modify signature verification. Per Phase 6 task 75 GOTCHA, the raw signature header value is not trivially available inside `Activity::receive` — Phase 6 stores an empty string with a `// TODO(v1): plumb actual HTTP signature through activitypub_federation hook`. **v1 carries this TODO forward** as DQ-FED-IN-1; resolving it requires a hook extension in `activitypub_federation` upstream or a fork-local wrapper. v1 ships without resolving (signature was already verified before reaching `receive`; storing the header value is for future re-verify, not v1 enforcement).

### 12.2 Schema validation strictness

Per §7.4: `#[serde(deny_unknown_fields)]` on protocol structs. Reject 400 on unknown variants of `SanctionAction`, `SanctionScope`, `AttestationType`. Per [06 §2.5](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md): "Strict validation of remote objects (schema + size limits)."

### 12.3 Storage isolation

All advisory rows have `local_case_id = NULL` at insert. Cross-link is admin-driven and emits a governance_log entry. No code path inserts an advisory row with a non-NULL `local_case_id` directly. Lint pattern (§12.6) catches drift.

### 12.4 Audit

Every inbound — accepted, dropped, or blocked — produces either a `governance_log` entry or a `federation_inbox_dropped_log` row (or both). Per [06 §2.5](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) "Federation inbound rate limiting" and [06 §7](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) row "Federation inbox" mitigation.

### 12.5 Step-up auth

Per [06 §2.4](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md), federation trust-state changes are high-risk actions requiring step-up auth + quorum + delay. v1 ships:
- **Step-up:** YES on `POST /api/v4/governance/admin/federation/peers/{id}/trust` (if step-up infrastructure ships in v1; otherwise gate this endpoint behind `webauthn-rs` MFA per ADR-010 v0/v1 simplification).
- **Quorum + delay:** Deferred. v1's admin model is single-admin. Quorum + delay is v2 alongside the broader OPA migration.
- **Cross-link / dismiss:** No step-up — admins act on advisory evidence frequently; step-up here would friction the workflow without commensurate security benefit.

### 12.6 Reserved slot for v2

Per [06 §2.5](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md): "Isolate media fetching from the main app network path — use a separate fetch worker with no access to the app DB (SSRF containment)." If a v1 inbound activity references remote media (e.g. evidence URL inside `summary`), v1 does NOT fetch it — the URL is stored as text only. Fetching is v2's SSRF-isolated worker.

### 12.7 New threat-model rows for [06 §7](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md)

v1 should append the following rows when this PRD lands:

| Asset | Threat | Mitigation | Reference |
|---|---|---|---|
| Federation inbox advisory storage | Hostile peer floods to fill DB | Per-peer rate limit (§7.1), per-peer storage cap (§7.3) with oldest-drop, admin can `Blocklist` peer | §7, §4.2 |
| Trust-attestation aggregate signal | Many peers attest the same target to influence local jury | Per-actor rate limit (§7.2), advisory-only (no auto-apply) | §7.2, ADR-006 |
| Federation inbox replay | Peer replays signed activity | Lemmy's `ReceivedActivity` dedup + v1 nonce table with sliding window | §7.5 |
| Advisory→local-case link | Compromised admin cross-links false notice to a real case | Cross-link emits governance_log entry; jury still decides; admin's cross-link history visible in modlog | §6.2 |
| `federation_peer` trust state | Compromised admin silently flips peer to `Allowlisted` | Step-up auth on trust-change endpoint; governance_log entry per change; v2 adds quorum + delay | §4.3, §12.5 |

### 12.8 Lint guards

Recommended fork-local lint patterns (in `.claude/rules/` or similar) to prevent future drift:

```bash
# No-auto-apply guard
rg -A 5 "RemoteSanctionNotice::|RemoteModerationLabel::|FederationAttestation::" crates/api/api/ crates/api/api_crud/
# Look for any insert with local_case_id = Some(_) NOT in the cross_link.rs handler
# Look for any call chain reaching Sanction::create from inbox handler

# No bypass of wrap_governance_inbound
rg "receive_remote_(sanction_notice|trust_attestation|moderation_label)" crates/apub/activities/
# Every match must be inside wrap_governance_inbound or in tests
```

---

## 13. Open questions for v1 design phase

### 13.1 OQ-FED-IN-1 — Cross-instance actor pseudonyms

**Question:** When peer B sends a `SanctionNoticeObject` about user X (identified by `target_url = https://b.example/u/x`), does X have a stable cross-instance pseudonym we can show in our admin review surface?

**Proposed resolution:** Each instance maintains its own `actor_pseudonym` table per ADR-015. Cross-instance attestations include `ap_id` of the attestee. At admin review time, the admin sees the raw `target_url`. If the admin cross-links to a local case where the same person has a local `actor_pseudonym`, the cross-linked view renders as `<local-pseudonym> ≈ <peer-domain>` ("locally known as X, peer claims is Y"). No automatic mapping — admin judgment.

**Status:** Open; resolve before v1 implementation.

### 13.2 OQ-FED-IN-2 — Vanilla Lemmy actor as TrustAttestation target

**Question:** If peer B (a Brehon fork) sends us a `TrustAttestationObject` about user X who lives on `lemmy.world` (vanilla Lemmy, no governance), can we accept it?

**Proposed resolution:** Yes — it's advisory-only. The target's home instance type doesn't matter; we store the attestation, surface it in admin review, and a local jury or admin can cite it if it's relevant to a case targeting the same actor's local activity. Never auto-apply (ADR-006 covers this regardless of target's instance type).

**Status:** Resolved as proposed; document in PRD body and proceed.

### 13.3 OQ-FED-IN-3 — Replay window default

**Question:** Is 7 days the right default for the nonce window (§7.5)?

**Proposed resolution:** 7 days is conservative (longer than a typical retry storm; short enough to bound the nonce table size: 100/h × 24 × 7 × N peers ≈ 17K rows for 1 peer at peak rate, 170K for 10 peers — well within Postgres comfort zone for an indexed lookup table). Configurable, so an instance with high-volume governance traffic can shorten it.

**Status:** Resolved as proposed; configurable.

### 13.4 OQ-FED-IN-4 — Federation discovery

**Question:** How do new Brehon instances find each other in v1?

**Proposed resolution:** **Defer to v2.** Lemmy's existing instance discovery (admin-curated allowlist, content federation handshakes) covers content-level discovery. Governance discovery as a separate concern (e.g. a `.well-known/brehon-governance` endpoint that announces governance-AP-type support) is v2 alongside the broader federation-policy-engine work.

**Status:** Resolved as deferred to v2.

### 13.5 OQ-FED-IN-5 — Admin notification on first-contact

**Question:** When a peer with `trust_level = unknown` (no `federation_peer` row) sends its first inbound governance activity, should the admin get a notification?

**Proposed resolution:** YES, but as a derived signal — the admin review surface already filters `requires_admin_attention = true` (§6.1) which includes `unknown` peers. v1 does not push notifications (no notification infrastructure in v0/v1). v3's notification UX (per ADR-010) picks this up.

**Status:** Resolved; covered by §6.1's filter.

---

## 14. Cross-references

- **Phase 6 plan** — `brehon-fork-advisor-phase6/.claude/PRPs/plans/phase-6-federation.plan.md` (foundation; v1 builds on; do not duplicate AP type definitions or outbound publisher).
- **admin-dashboard-v1 PRD** (sibling) — v1 dashboard wraps the §6 endpoints and the federation_peer CRUD.
- **OQ-018** ([99-decisions-and-open-questions.md](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)) — admin config write endpoint shape; v1 federation policy knobs (§10) ride on this. The `federation.inbound.*` namespace must be supported by OQ-018's per-key write endpoint.
- **ADR-006** — advisory-only federation; v1's hard rule.
- **ADR-014** — vanilla Lemmy interop; v1 unaffected (vanilla peers never send governance types).
- **[05 §7.1](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md)** — v1 scope; "Full inbound + outbound" federation under "v1 production-grade governance" milestone.
- **[05 §7.3](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md)** — v3 scope; "Full federation inbound processing (beyond stored-advisory)" — confirms v1 is store + surface only, v3 is the auto-apply milestone.
- **[06 §2.5](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md)** — federation boundary hardening; v1 implements rate-limiting and schema validation lines.
- **[06 §4.7](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md)** — fake federated attestations; v1 §7.2 is the per-actor rate-limit defence.
- **[07 §5.2](../../docs/brehon-law-inspired-network/07-operations-and-federation.md)** — operator runbook for receiving sanction notices; v1's §6 endpoints are the API behind step 5 of that runbook.
- **v1 carry-forward issues:** none directly federation-tagged but **issue #16 (admin config endpoint, OQ-018)** is the surface federation policy knobs ride on.

---

## 15. Phase 6 alignment notes

### 15.1 What Phase 6 ships that v1 builds on

| Phase 6 deliverable | v1 dependency |
|---|---|
| `SharedInboxActivities` enum with three governance variants (task 73) | v1 wraps the `Activity::receive` impls; does not re-route |
| `receive_remote_sanction_notice`, `receive_remote_trust_attestation` (task 75) | v1 calls these from the wrapper unchanged |
| `governance_log::append` const strings `federation_sanction_received`, `federation_attestation_received` (task 75) | v1 keeps these; adds `federation_label_received`, `federation_inbound_blocked`, `federation_inbound_dropped_*`, `federation_inbound_cross_linked`, `federation_inbound_dismissed`, `federation_peer_trust_changed`, `federation_inbound_storage_cap_evicted`. **Canonical prefix for all new governance-log `entry_kind` strings is `federation_inbound_*`** — matches §6.1 (cross-link) and §6.2 (dismiss). The `federation_inbox_*` prefix in this PRD is reserved for code identifiers (functions `federation_inbox_check_*`, tables `federation_inbox_dropped_log` / `federation_inbox_nonce`, enum `federation_inbox_admin_action_enum`) and must NOT be used as an `entry_kind` string literal. |
| `RemoteSanctionNotice`, `FederationAttestation` Diesel models with insert forms (task 71) | v1 extends with new columns + UpdateForm |
| `ApubModerationLabel` AP object + protocol struct (task 72) | v1 implements the receive function (Phase 6 left it stubbed) |
| HTTP signature verification via `activitypub_federation` (Lemmy code, trusted as-is per ADR-012) | v1 unchanged |

### 15.2 Divergence between Phase 6 plan and v1 assumptions

**Canonical preconditions list:** see §8.0. Any divergence the reviewer disagrees with must be pushed back before v1 migration runs. The DQ-6.1 and DQ-6.2 items below are now prescriptive — v1 assumes these landings; §8.0 lists them as preconditions #3 and (implicitly) as part of #2's "with all columns Phase 6 creates."

| Item | Phase 6 design | v1 design | Why |
|---|---|---|---|
| `local_case_id` column | `INT REFERENCES moderation_case(id) ON DELETE SET NULL` (Phase 6 task 70) | Same; v1 does NOT change | Compatible |
| `received_at` column on `remote_sanction_notice` | **Per §8.0 precondition #3**: Phase 6 ships `received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` resolving DQ-6.1. v1 assumes this landed. | v1 relies on it for admin review queries (§6.1 `age_days` filter) | Prescriptive — if DQ-6.1 landed differently, the §8.2 DO block aborts the migration and v1 re-plans. |
| `signature` column | TEXT NOT NULL; Phase 6 stores empty string per task 75 GOTCHA (DQ-6.2) | v1 accepts the empty-string-for-now state; does not require resolving DQ-6.2 at v1 schedule time | DQ-6.2 carries forward — v1 stores signatures but does not depend on them being non-empty until v2 |
| Idempotency / dedup | Phase 6 relies on Lemmy's `ReceivedActivity::create` dedup (DQ-6.3) | v1 adds a dedicated `federation_inbox_nonce` table for governance-specific replay window control | v1's nonce window is configurable; Lemmy's `ReceivedActivity` retention is separately controlled. Both fire — defence in depth |
| Outbound peer targeting | `to_all_instances()` (Phase 6 task 74) | v1 leaves unchanged; `Blocklisted` peers still receive outbound (asymmetric) | v1 design choice: blocklisting an inbound peer doesn't necessarily mean we want to stop telling them about our decisions. v2 may add per-peer outbound block as a separate knob |
| Test pattern | Direct-call into `Activity::receive` (Phase 6 task 77) | v1's wrapper changes the direct-call's behaviour — test must seed `federation_peer` with `Allowlisted` before calling | Test fixture update only |
| HTTP route registration | None — Phase 6 says "No new REST route" | v1 adds 3 new REST routes for admin review | v1 explicitly adds the admin surface that Phase 6 §11 NOT-Building item #6 defers |

### 15.3 Phase 6 design choices that limit v1 inbound design space

1. **`source_instance` is TEXT, not FK to `instance.id`** on `remote_sanction_notice`. v1 must JOIN by domain string. Workable but case-sensitivity must be tested.
2. **Outbound uses `to_all_instances()`.** v1 does not narrow this — v2 work. So a `Blocklisted` peer still receives our outbound activities; if this is a concern, the `Blocklisted` admin should know that asymmetry exists.
3. **`PublishLabel::receive` is stubbed.** v1 fills it with the new handler; this is additive, not breaking.
4. **No HTTP route layer changes in Phase 6.** v1 adds the admin endpoints but does NOT touch the inbound HTTP route — Phase 6's `SharedInboxActivities` registration already provides HTTP entry; v1 just wraps the receive impls.
5. **`activitypub_federation` v0.7.0-beta.10** is the workspace pin per Phase 6. Replay-protection nonce table and HTTP-signature plumbing (DQ-FED-IN-1) work within this version's hooks. Upgrade is out of v1 scope.

### 15.4 What Phase 6 explicitly defers to v1 (and v1 honours)

From Phase 6 plan §"NOT Building":
- ✅ Peer/instance allowlist table → v1 ships `federation_peer`
- ✅ Federation rate-limiting per source → v1 ships §7
- ✅ HTTP route for admin review → v1 ships §6
- ✅ Undo activities → **NOT in v1 either** — deferred further (v1 ships only the v1 inbound surface; Undo wire-up is its own work, picked up in v1 follow-up or v2)
- ✅ Outbound retry UX → unchanged (Lemmy's `sent_activity` queue retries)
- ✅ SSRF-isolated media fetch worker → v2 (per [06 §2.5](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) and ADR-010)
- ✅ Signed build artefacts → v2 (ADR-010)
- ✅ Auto-apply of remote sanctions → v3 (ADR-006 + [05 §7.3](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md))

---

## 16. Resolutions applied (2026-04-19)

Cross-PRD coherence-audit edits applied during v1-PRD edit pass (see `.claude/PRPs/v1-planning-queue.json`):

| ID | Severity | Scope | Status |
|---|---|---|---|
| **B5** | blocking | Phase 6 dependency formalised as **hard gate** + verify-before-migrate preamble SQL `DO` block. Migration timestamp invariant: "must sort after Phase 6's `add_federation_attestations`" — no specific date reserved. Prescriptive DQ-6.1 / DQ-6.2 references in §15.2. §1 frontmatter + §8 heading + §8.0 new subsection capture the gate. | done |
| **N5** (collateral of B5) | non-blocking | §4.1 FK-shape vs §8.1 TEXT domain-string mismatch graduated from "noted, revisit at v1-impl" to "resolved at v1-impl time via explicit §8.0 precondition #6." Reviewer who disagrees with the TEXT-not-FK assumption must push back at PR review. | done |

### Edits applied

- **§1 frontmatter** — new `Scheduling:` line declaring the hard gate explicitly.
- **§8 heading** renamed from `Database & migration changes` → `Dependencies, database & migration changes`.
- **§8.0 Dependencies** — new subsection listing 7 preconditions (Phase 6 PR merged, 2 tables present, `received_at` column, enum types, entry_kind consts, TEXT-not-FK `source_instance`, timestamp invariant). Explicit STOP directive on unmet preconditions.
- **§8.2 SQL preamble** — `DO $$ … $$;` block checks `federation_attestation` table presence, `remote_sanction_notice` table presence, `remote_sanction_notice.received_at` column presence; `RAISE EXCEPTION` on any missing element.
- **§15.2 divergence table** — DQ-6.1 row rewritten to prescriptive tone ("per §8.0 precondition #3"); DQ-6.2 row tightened (v1 stores signatures but does not gate on non-empty until v2). Preamble note added pointing reviewers at §8.0 as canonical preconditions.
- **Migration timestamp** — invariant ("must sort after Phase 6's `add_federation_attestations`") stated in §8.2; no specific 2026-04-22-style slot reserved, per user direction 2026-04-19.
- **admin-dashboard-v1 §10.1** — cross-reference line added under the federation-inbound consumer row citing this hard gate (see admin-dashboard-v1 §11 Resolutions).

---

*Generated: 2026-04-19. Status: DRAFT — review before running `/prp-plan` against this PRD.*
