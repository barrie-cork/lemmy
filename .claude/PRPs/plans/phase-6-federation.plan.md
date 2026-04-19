# Plan: Phase 6 — Federation objects & activities, outbound-only

## Summary

Phase 6 closes the v0 MVP by adding governance-specific ActivityPub objects, activities, and outbound publish path for federated sanction notices and trust attestations. When instance A decides a case with `SanctionScope::FederatedRecommendation`, instance B receives a `SanctionNotice` AP activity, verifies its HTTP signature, stores an advisory row in `remote_sanction_notice`, surfaces it to admin review, and **never auto-applies** ([99 ADR-006](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)). The nine tasks split into five **layered execution waves** (see §11) — a departure from Phase 5's single-ralph model — with parallel local agents in their own git worktrees coordinated by the advisor session at merge points.

## Source

- [IMPLEMENTATION-PLAN-v0.md](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) §3 Phase 6 (tasks 70–78)
- [04-data-model-and-api.md](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) §1 Migration 4 (federation tables), §3 (Diesel models), §9 (AP objects), §10 (AP activities), §11 (inbox/outbox)
- [05-mvp-and-delivery-plan.md](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) §4 Step 6 (federation DoD), §3 (outbound-only simplification)
- [03-architecture.md](../../../docs/brehon-law-inspired-network/03-architecture.md) §3.3 (federation layer), §6 (governance log) — "inbound federation governance signals are stored as advisory evidence only in MVP"
- [06-security-and-threat-model.md](../../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) §2.5 (harden federation boundary), §4.7 (fake federated attestations)
- Relevant ADRs: **ADR-006** (advisory-only), **ADR-010** (v0/v1 split, no peer table in v0), **ADR-012** (`ap_id` naming, Extism plugins, Lemmy 1.0-beta), **ADR-014** (federation interop with vanilla Lemmy), **ADR-008** (hash-chain log for every federation write), **ADR-015** (actor pseudonymisation for log entries)
- [SUBSCRIPTIONS.md](../../../docs/brehon-law-inspired-network/SUBSCRIPTIONS.md) — V2 hooks catch-up (entry-kind extension for new federation events)
- Prior-phase integration: `submit_jury_vote` (Phase 4 task 42 + Phase 5b task 56), `governance_log::append` (Phase 1 task 5 + Phase 4b hardening), `actor_pseudonym_helper::get_or_create` (Phase 4)

## Problem Statement

The v0 governance layer (Phases 1–5) is fully local: all reports, jury votes, sanctions, and public log entries never leave the instance. A `RecommendFederationAction` vote outcome creates a local `Sanction` with `scope = FederatedRecommendation`, but no AP activity is published; peers learn nothing. This breaks the v0 DoD ([05 §4 Step 6](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md)): "a sanction on instance A publishes a `SanctionNotice` that instance B receives, verifies, stores in `remote_sanction_notice`, and surfaces in admin review (never auto-applies)."

Phase 6 delivers:

1. **Federation persistence** — two new tables (`federation_attestation`, `remote_sanction_notice`) + their Diesel models
2. **AP type layer** — three governance protocol objects (`ModerationLabelObject`, `TrustAttestationObject`, `SanctionNoticeObject`) + their wrapping `Create` activities
3. **Outbound path** — `send_local_sanction_notice(case_id)` and `send_local_trust_attestation(person_id, attestation_type)` that build the activity, enqueue it via Lemmy's existing `send_lemmy_activity` → `sent_activity` queue, and write a `federation_sanction_sent` governance log entry
4. **Inbound path** — `receive_remote_sanction_notice(activity)` and `receive_remote_trust_attestation(activity)` that verify signatures (delegating to `activitypub_federation` HTTP signature check), validate schema, insert an advisory `remote_sanction_notice` row with `local_case_id = NULL`, and emit a `federation_sanction_received` log entry
5. **Handler wiring** — `submit_jury_vote` calls `send_local_sanction_notice` after reputation deltas but before `case_decided` log entry, only when the winning sanction has `scope = FederatedRecommendation`
6. **Round-trip test** — `tests/e2e.rs::sanction_notice_round_trip` boots two Postgres containers, runs the inbound function directly against instance B's context after publishing on instance A, and asserts the advisory row with byte-accurate field copies

## Solution Statement

Extend the existing `crates/apub/{objects,activities,apub}` crates with new `governance/` subdirectories that mirror Lemmy's established patterns (newtype wrapper + `impl Object/Activity` + protocol struct with `#[skip_serializing_none]` + serde camelCase). The outbound path uses `ActivitySendTargets::to_all_instances()` (treating every federated peer as `Allow` — v0 simplification, no peer table). The inbound path is a direct function call from the round-trip test — HTTP transport (signature verification, JSON parsing, retry/backoff) is Lemmy's existing code, trusted as-is per ADR-012.

The governance log wires federation events as two new entry kinds (`federation_sanction_sent`, `federation_sanction_received`, `federation_attestation_sent`, `federation_attestation_received`) that extend the contract documented in [SUBSCRIPTIONS.md](../../../docs/brehon-law-inspired-network/SUBSCRIPTIONS.md) without breaking existing subscribers.

## Metadata

| Field | Value |
|---|---|
| Type | FEDERATION |
| Complexity | HIGH |
| Crates Affected | `lemmy_db_schema`, `lemmy_db_schema_file`, `lemmy_apub_objects`, `lemmy_apub_activities`, `lemmy_apub`, `lemmy_api`, `lemmy_server` (test) |
| v0 Step | Step 6 of [05 §4](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) |
| Dependencies | Phase 1 (hash-chain log), Phase 4 (`submit_jury_vote`), Phase 5a (reputation snapshot), Phase 5b (sponsor-liability), Phase 5c (task 68 route registration) |
| Estimated Tasks | 9 (tasks 70–78) |
| Layers | 5 execution waves, max 2 concurrent agents |
| Target branch | `phase-6` (cut from `governance-v0` post PR #10 merge) |

---

## Pre-Flight Gate (MUST resolve before any agent spawns)

The invocation assumed the following prerequisites; as of planning time **none are met**. Plan execution cannot start until all three are green.

| Prereq | Required State | Observed State (2026-04-19) | Blocker |
|---|---|---|---|
| 1 | PR #10 merged into `governance-v0` via `--merge` (not squash) | **OPEN** — `gh pr view 10` reports `state: OPEN, mergedAt: null` | Yes — merging PR #10 brings tasks 54–69a into `governance-v0`; without it, Phase 6 commits would conflict with those changes on rebase |
| 2 | `/brehon-phase-transition 5 6` has run (archive memory, fresh advisor context, updated `project_brehon_governance_platform.md`) | Memory state uncertain — `MEMORY.md` still points at Phase 5c complete; no `advisor-context-phase-6.md` observed in tree | Soft — plan can proceed without, but advisor loses continuity context |
| 3 | `phase-6` branch cut from updated `governance-v0` HEAD | `git rev-parse --verify phase-6` → `fatal: Needed a single revision` (branch absent) | Yes — per `.claude/rules/phase-branch.md`, no Phase 6 commits land until this branch exists |

**Action item (advisor, not impl):** before Agent A (Layer 1) is spawned, the user must:

1. Merge PR #10 with `gh pr merge 10 --repo barrie-cork/lemmy --merge` (preserves task-per-commit history; see `.claude/rules/phase-branch.md` "Do not squash the PR at merge").
2. Run `/brehon-phase-transition 5 6` to archive Phase 5 advisor context and produce `.claude/advisor-context-phase-6.md` + updated memory.
3. From the primary worktree (which will have pending planning edits), advance the local `governance-v0` ref via the pattern in [`feedback_preserve_active_worktree_state.md`](../../../../.claude/projects/C--Users-barri-Developer-brehon-fork/memory/feedback_preserve_active_worktree_state.md): `git fetch origin governance-v0 && git update-ref refs/heads/governance-v0 origin/governance-v0` (requires `git merge-base --is-ancestor` check first). Then cut `phase-6` via `git branch phase-6 governance-v0` — **do not checkout-switch** the primary worktree.
4. For the pre-phase harness audit (below, §Pre-Phase Harness Audit), create an auxiliary worktree: `git worktree add ../brehon-fork-phase6 phase-6` and run the audit there.

If any of the three prereqs above are unmet, the plan **flags but does not proceed**. Impl agents spawned against a missing `phase-6` branch will fail fast per `phase-branch.md`.

---

## Flow Design

### Before State (post-Phase 5c)

```text
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              BEFORE                                            ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║  Instance A:                                                                  ║
║    submit_jury_vote(RecommendFederationAction)                                ║
║       └─> Sanction(scope=FederatedRecommendation) inserted                    ║
║       └─> case_decided log entry                                              ║
║       └─> HTTP 200 returned                                                   ║
║                                                                               ║
║  Instance B:                                                                  ║
║    (receives nothing; no federation path exists)                              ║
║                                                                               ║
║  PAIN_POINT: FederatedRecommendation scope has no outbound effect.            ║
║  v0 DoD Step 6 unmet.                                                         ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### After State

```text
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              AFTER                                             ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║  Instance A:                                                                  ║
║    submit_jury_vote(RecommendFederationAction)                                ║
║       ├─> Sanction row inserted                                               ║
║       ├─> sponsor-liability deltas (Phase 5b)                                 ║
║       ├─> case flip → Decided                                                 ║
║       ├─> public_case_log published                                           ║
║       ├─> juror/reporter reputation deltas                                    ║
║       ├─> [NEW] if scope == FederatedRecommendation:                          ║
║       │      send_local_sanction_notice(case_id)                              ║
║       │        ├─> build SanctionNoticeObject (redacted summary)              ║
║       │        ├─> wrap in PublishSanctionNotice Create activity              ║
║       │        ├─> SentActivity::create with to_all_instances()               ║
║       │        └─> governance_log::append("federation_sanction_sent", ...)    ║
║       └─> case_decided log entry                                              ║
║                                                                               ║
║  Instance B (existing HTTP inbox):                                            ║
║    POST /inbox                                                                ║
║       └─> receive_activity_with_hook (Lemmy existing, sig-verified)           ║
║              └─> PublishSanctionNotice::verify (domain match, remote obj)     ║
║              └─> PublishSanctionNotice::receive                               ║
║                    ├─> insert remote_sanction_notice(local_case_id = NULL)    ║
║                    ├─> governance_log::append("federation_sanction_received") ║
║                    └─> **NO auto-apply** (ADR-006)                            ║
║                                                                               ║
║  VALUE_ADD: v0 MVP DoD Step 6 met; closes Phase 6.                            ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### Endpoint Changes

| Endpoint / Entrypoint | Before | After | Impact |
|---|---|---|---|
| `submit_jury_vote` POST handler | No federation side-effect | Calls `send_local_sanction_notice` after reputation deltas, before `case_decided` log | One additional enqueue per federated decision; test at task 77 verifies ordering |
| Lemmy's existing `/inbox` route | Rejects unknown activity types | Accepts `PublishSanctionNotice`, `PublishTrustAttestation`, `PublishLabel` via `SharedInboxActivities` enum | No new route; dispatch happens in `activity_lists.rs` |
| No new REST route | — | — | Federation uses AP inbox, not REST |

---

## Mandatory Reading (every Phase 6 agent MUST read before starting)

### P0 — every agent

| File | Lines | Why |
|---|---|---|
| [IMPLEMENTATION-PLAN-v0.md](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) | §3 Phase 6 (421–447) | Task specs 70–78, DoD, dependency chain |
| [04-data-model-and-api.md](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) | §1 Migration 4 (39–50), §3 FederationAttestation + RemoteSanctionNotice (361–405), §9 (762–799), §10 (801–808), §11 (810–828) | Table shapes, AP object field lists, function names |
| [99-decisions-and-open-questions.md](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | ADR-006 (85–96), ADR-012 (212–224), ADR-014 (241–255), ADR-015 (256–273) | Advisory-only, `ap_id` naming, vanilla-Lemmy interop, pseudonymisation |
| `CLAUDE.md` (fork root) | "Hard constraints" section | The 15 ADRs that cannot be re-litigated |
| [SUBSCRIPTIONS.md](../../../docs/brehon-law-inspired-network/SUBSCRIPTIONS.md) | §Entry kinds (44–72), §Stability contract (73–82) | V2 hook catch-up; new entry kinds must extend (not break) the list |

### P1 — by layer

**Layer 1 (schema + models, Agent A):**
- `migrations/2026-04-15-100100-0000_add_governance_core/up.sql` — naming pattern for `add_governance_*` migrations
- `migrations/2026-04-15-100500-0000_add_governance_log/up.sql` — trigger/index idioms
- `crates/db_schema/src/source/governance/sanction.rs` — Diesel model pattern to mirror
- `crates/db_schema_file/src/enums.rs` — current `SanctionAction`, `SanctionScope`, `AttestationType` variants

**Layer 2 (AP types, Agents B + C):**
- `crates/apub/objects/src/objects/comment.rs:48-242` — canonical `Object` trait impl
- `crates/apub/objects/src/objects/private_message.rs:44-177` — simpler Object example (no community context)
- `crates/apub/objects/src/protocol/note.rs:30-58` — protocol type conventions
- `crates/apub/activities/src/block/block_user.rs:39-206` — canonical `Activity` trait impl with `send` helper
- `crates/apub/activities/src/community/report.rs:51-182` — closest analogy to SanctionNotice (moderation-adjacent, single target)
- `crates/apub/activities/src/protocol/community/report.rs:17-42` — protocol type for moderation activities
- `crates/apub/activities/src/activity_lists.rs:38-98` — `SharedInboxActivities` / `AnnouncableActivities` dispatch enum registration
- `crates/db_schema/src/source/activity.rs:14-54` — `ActivitySendTargets` full API

**Layer 3 (publisher + receiver, Agents D + E):**
- `crates/apub/activities/src/lib.rs:113-143` — `send_lemmy_activity` signature
- `crates/apub/apub/src/http/mod.rs:44-114` — inbox HTTP handler + `receive_activity_with_hook` integration
- `crates/api/api/src/governance/governance_log.rs:72-126` — `append` signature + entry-kind const list
- `crates/api/api/src/governance/actor_pseudonym_helper.rs:21-60` — pseudonym helper
- `crates/api/api/src/governance/redaction.rs` — `scrub_json` called inside `governance_log::append` (verify it handles federation payloads)

**Layer 4 (handler wiring, Agent F):**
- `crates/api/api/src/governance/submit_jury_vote.rs` — full flow: lines 92–119 (signature), 238–241 (sanction insert), 295–308 (public_case_log), 347–361 (juror reputation), 385–395 (reporter reputation), 397–407 (case_decided log). **Task 76 hook goes between line 395 and line 397.**

**Layer 5 (round-trip test, Agent G):**
- `crates/server/tests/e2e.rs:42-113` — `governance_fixtures` module (container boot, schema apply, URL build)
- `crates/server/tests/e2e.rs:743-1300` — `report_to_modlog_golden_path` (closest analogy for multi-step handler + DB assertions)
- `crates/server/tests/e2e.rs:856-872` — `seed_person` helper
- `crates/server/tests/e2e.rs:2195-2444` — `all_mvp_endpoints_return_non_404` (context construction + make_user + mint_jwt helpers)
- `crates/api/api_utils/src/context.rs:24-38` — `LemmyContext::create` signature for second-instance build

### External Documentation

| Source | Version | Section | Why |
|---|---|---|---|
| [activitypub_federation](https://docs.rs/activitypub_federation/0.7.0-beta.10) | 0.7.0-beta.10 (workspace pin) | `traits::Object`, `traits::Activity`, `actix_web::inbox` | Required-method signatures; **no derive macros — impls are hand-written with `#[async_trait]`** (confirmed by comment.rs/block_user.rs patterns) |
| [serde_with `skip_serializing_none`](https://docs.rs/serde_with) | matches workspace | `#[skip_serializing_none]` attribute | Required on every protocol struct (omits null fields to match AP spec) |
| [enum_delegate](https://docs.rs/enum_delegate/0.2.0) | 0.2.0 | `#[enum_delegate::implement(Activity)]` | Used when adding new variants to `SharedInboxActivities` enum |

---

## Patterns to Mirror

### MIGRATION_FILE_NAMING
```text
migrations/2026-MM-DD-HHMMSS-NNNN_add_federation_attestations/up.sql
migrations/2026-MM-DD-HHMMSS-NNNN_add_federation_attestations/down.sql
```
Timestamp must sort **after** `2026-04-20-000000-0000_add_governance_log_notify/` (the last governance migration).

### DIESEL_MODEL_STRUCT
```rust
// SOURCE: crates/db_schema/src/source/governance/sanction.rs
// (Audit confirmed fields; Phase 6 mirrors this exact derive + table_name pattern.)
#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
#[diesel(table_name = sanction)]
pub struct Sanction {
    pub id: SanctionId,
    pub case_id: ModerationCaseId,
    pub scope: SanctionScope,
    pub action: SanctionAction,
    // ... etc
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = sanction)]
pub struct SanctionInsertForm { /* mirror field-for-field minus id + server-assigned fields */ }
```

### AP_OBJECT_TRAIT_IMPL
```rust
// SOURCE: crates/apub/objects/src/objects/comment.rs:65-242
#[async_trait::async_trait]
impl Object for ApubSanctionNotice {
    type DataType = LemmyContext;
    type Kind = SanctionNoticeProtocol;  // from protocol/governance/sanction_notice.rs
    type Error = LemmyError;

    fn id(&self) -> &Url { self.ap_id.inner() }

    async fn read_from_id(
        object_id: Url,
        context: &Data<Self::DataType>,
    ) -> LemmyResult<Option<Self>> {
        // Look up by ap_id; may return None for send-only objects
        Ok(None)  // v0: SanctionNotice is outbound-only; no lookup
    }

    async fn delete(&self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
        Err(LemmyErrorType::NotFound.into())  // not deletable in v0
    }

    fn is_deleted(&self) -> bool { false }

    async fn into_json(self, context: &Data<Self::DataType>) -> LemmyResult<SanctionNoticeProtocol> { /* build protocol struct */ }

    async fn verify(
        object: &SanctionNoticeProtocol,
        expected_domain: &Url,
        context: &Data<Self::DataType>,
    ) -> LemmyResult<()> {
        verify_domains_match(object.id.inner(), expected_domain)?;
        Ok(())
    }

    async fn from_json(object: SanctionNoticeProtocol, context: &Data<Self::DataType>) -> LemmyResult<Self> { /* insert remote_sanction_notice */ }
}
```

### PROTOCOL_STRUCT
```rust
// SOURCE: crates/apub/objects/src/protocol/note.rs:30-58
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SanctionNoticeProtocol {
    #[serde(rename = "type")]
    pub(crate) kind: SanctionNoticeType,  // custom kind const (e.g., "SanctionNotice")
    pub id: Url,
    pub actor: ObjectId<ApubPerson>,
    pub target: Url,                 // target ap_id (user, post, community)
    pub action: SanctionAction,
    pub scope: SanctionScope,
    pub summary: String,             // MUST be scrubbed before serialisation (§4.2)
    pub published: DateTime<Utc>,
}
```

### AP_ACTIVITY_TRAIT_IMPL
```rust
// SOURCE: crates/apub/activities/src/block/block_user.rs:99-206
#[async_trait::async_trait]
impl Activity for PublishSanctionNotice {
    type DataType = LemmyContext;
    type Error = LemmyError;

    fn id(&self) -> &Url { &self.id }
    fn actor(&self) -> &Url { self.actor.inner() }

    async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
        verify_is_public(&self.to, &self.cc)?;
        // Sanction notices are instance-wide broadcasts (not community-scoped)
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
        // 1. Signature already verified by activitypub_federation::actix_web::inbox
        // 2. Schema validate: verify action + scope enum variants match what we support
        // 3. Build RemoteSanctionNoticeInsertForm from self.object
        // 4. Insert row with local_case_id = NULL (no auto-apply per ADR-006)
        // 5. governance_log::append("federation_sanction_received", json!({...}), None)
        Ok(())
    }
}
```

### SEND_HELPER
```rust
// SOURCE: crates/apub/activities/src/block/block_user.rs:65-96
pub async fn send_local_sanction_notice(
    case_id: ModerationCaseId,
    context: &Data<LemmyContext>,
) -> LemmyResult<()> {
    // 1. Load ModerationCase + winning Sanction row
    // 2. Load actor (local admin / system actor) for ap_id + key signing
    // 3. Load target Person/Post/Comment/Community to get target_url
    // 4. Build SanctionNoticeProtocol (redact summary via scrub())
    // 5. Wrap in PublishSanctionNotice Create activity (generate id via generate_activity_id)
    // 6. Choose targets: v0 = ActivitySendTargets::to_all_instances()
    // 7. send_lemmy_activity(context, publish, actor, targets, sensitive=false).await?;
    // 8. governance_log::append("federation_sanction_sent", json!({"case_id": ...}), Some(actor_pseudonym))
    Ok(())
}
```

### ACTIVITY_ENUM_REGISTRATION
```rust
// SOURCE: crates/apub/activities/src/activity_lists.rs:38-98
// ADD NEW VARIANTS TO SharedInboxActivities enum (AnnouncableActivities NOT needed —
// sanction notices are instance-wide, not community-wrapped Announce)
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
#[enum_delegate::implement(Activity)]
pub enum SharedInboxActivities {
    Follow(Follow),
    // ... existing variants ...
    PublishSanctionNotice(PublishSanctionNotice),     // NEW
    PublishTrustAttestation(PublishTrustAttestation), // NEW
    PublishLabel(PublishLabel),                        // NEW (stub for future; may Undo-wrap)
    RawAnnouncableActivities(RawAnnouncableActivities),  // MUST remain last (catch-all)
}
```

### GOVERNANCE_LOG_APPEND
```rust
// SOURCE: crates/api/api/src/governance/governance_log.rs:87-126
pub async fn append(
    pool: &mut DbPool<'_>,
    entry_kind: &str,
    payload: Value,
    actor_pseudonym: Option<String>,
) -> LemmyResult<GovernanceLog>

// Phase 6 adds these const strings:
pub const ENTRY_KIND_FEDERATION_SANCTION_SENT: &str = "federation_sanction_sent";
pub const ENTRY_KIND_FEDERATION_SANCTION_RECEIVED: &str = "federation_sanction_received";
pub const ENTRY_KIND_FEDERATION_ATTESTATION_SENT: &str = "federation_attestation_sent";
pub const ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED: &str = "federation_attestation_received";
```

### TWO_DB_TEST_PATTERN
```rust
// SOURCE: crates/server/tests/e2e.rs:83-107 (start_postgres) + 2195-2444 (context building)
#[tokio::test(flavor = "multi_thread")]
async fn sanction_notice_round_trip() -> Result<(), Box<dyn Error>> {
    // Instance A
    let (_container_a, port_a) = governance_fixtures::start_postgres().await?;
    let url_a = governance_fixtures::db_url(port_a);
    { let mut c = PgConnection::establish(&url_a)?; governance_fixtures::apply_all_schema(&mut c)?; }
    unsafe { std::env::set_var("LEMMY_DATABASE_URL", &url_a); }
    let context_a = build_test_context().await?;  // helper from task-68 inventory
    let instance_a = Instance::read_or_create(&mut context_a.pool(), "instance-a.test").await?;

    // Instance B (identical pattern, different port)
    let (_container_b, port_b) = governance_fixtures::start_postgres().await?;
    let url_b = governance_fixtures::db_url(port_b);
    { let mut c = PgConnection::establish(&url_b)?; governance_fixtures::apply_all_schema(&mut c)?; }
    unsafe { std::env::set_var("LEMMY_DATABASE_URL", &url_b); }
    let context_b = build_test_context().await?;
    let instance_b = Instance::read_or_create(&mut context_b.pool(), "instance-b.test").await?;

    // A: drive jury → decision → send_local_sanction_notice
    // ...
    // Serialize SentActivity row from A into a PublishSanctionNotice struct
    // Call PublishSanctionNotice::receive(context_b) directly (no HTTP)
    // Assert remote_sanction_notice row exists on B with local_case_id IS NULL
    Ok(())
}
```

**Note on env-var swap pattern:** tests 2195+ in `e2e.rs` use `unsafe { std::env::set_var("LEMMY_DATABASE_URL", ...) }` between instances. In a two-DB test **serialised sequentially** this works; with `#[tokio::test(flavor = "multi_thread")]` any interleaving between instances could corrupt pool construction. Construct pool A fully before swapping env, then swap and construct pool B. **Do not share a single tokio runtime across the two pool builds without explicit sequencing.** See §12 R1.

---

## Files to Change

### NEW FILES

| File | Purpose |
|---|---|
| `migrations/{timestamp}_add_federation_attestations/up.sql` | Tables `federation_attestation`, `remote_sanction_notice` + indexes per [04 §1](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) |
| `migrations/{timestamp}_add_federation_attestations/down.sql` | Reverse: DROP TABLE both + any created enum extensions |
| `crates/db_schema/src/source/governance/federation_attestation.rs` | `FederationAttestation` + `FederationAttestationInsertForm` |
| `crates/db_schema/src/source/governance/remote_sanction_notice.rs` | `RemoteSanctionNotice` + `RemoteSanctionNoticeInsertForm` |
| `crates/apub/objects/src/governance/mod.rs` | Exports for new governance object types |
| `crates/apub/objects/src/governance/sanction_notice.rs` | `ApubSanctionNotice` newtype + `impl Object` |
| `crates/apub/objects/src/governance/trust_attestation.rs` | `ApubTrustAttestation` newtype + `impl Object` |
| `crates/apub/objects/src/governance/moderation_label.rs` | `ApubModerationLabel` newtype + `impl Object` (stub — outbound only, no receive logic in v0) |
| `crates/apub/objects/src/protocol/governance/mod.rs` | Exports for new protocol types |
| `crates/apub/objects/src/protocol/governance/sanction_notice.rs` | `SanctionNoticeProtocol` (serde struct) |
| `crates/apub/objects/src/protocol/governance/trust_attestation.rs` | `TrustAttestationProtocol` |
| `crates/apub/objects/src/protocol/governance/moderation_label.rs` | `ModerationLabelProtocol` |
| `crates/apub/activities/src/governance/mod.rs` | Exports for new governance activities |
| `crates/apub/activities/src/governance/publish_sanction_notice.rs` | `PublishSanctionNotice` Create activity + `send_local_sanction_notice` |
| `crates/apub/activities/src/governance/publish_trust_attestation.rs` | `PublishTrustAttestation` + `send_local_trust_attestation` |
| `crates/apub/activities/src/governance/publish_label.rs` | `PublishLabel` (stub) |
| `crates/apub/activities/src/protocol/governance/mod.rs` | Exports for activity protocol types |
| `crates/apub/activities/src/protocol/governance/publish_sanction_notice.rs` | Protocol struct for the activity |
| `crates/apub/activities/src/protocol/governance/publish_trust_attestation.rs` | Protocol struct |
| `crates/apub/activities/src/protocol/governance/publish_label.rs` | Protocol struct |
| `crates/apub/apub/src/governance/mod.rs` | Wires `inbox.rs`, `outbox.rs`, `verify.rs` |
| `crates/apub/apub/src/governance/inbox.rs` | `receive_remote_sanction_notice`, `receive_remote_trust_attestation` free functions (called from within the Activity::receive impls) |
| `crates/apub/apub/src/governance/outbox.rs` | `send_local_sanction_notice`, `send_local_trust_attestation` re-exports (the actual impl lives in `activities/governance/*.rs`; this file is a thin re-export boundary per [04 §11](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md)) |
| `crates/apub/apub/src/governance/verify.rs` | Thin wrapper over `verify_domains_match` / `check_apub_id_valid_with_strictness`; home for future custom federation checks per [04 §11](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) |

### MODIFIED FILES

| File | Action |
|---|---|
| `crates/db_schema/src/schema.rs` | `diesel print-schema` output extended with two new tables |
| `crates/db_schema/src/source/governance/mod.rs` | `pub mod federation_attestation; pub mod remote_sanction_notice; pub use {federation_attestation::*, remote_sanction_notice::*};` |
| `crates/db_schema/src/newtypes.rs` | Add `FederationAttestationId`, `RemoteSanctionNoticeId` newtypes (mirror existing pattern) |
| `crates/apub/objects/src/lib.rs` | `pub mod governance;` + `pub mod protocol { pub mod governance; ... }` |
| `crates/apub/activities/src/lib.rs` | `pub mod governance;` + `pub mod protocol { pub mod governance; ... }` |
| `crates/apub/activities/src/activity_lists.rs` | Add `PublishSanctionNotice`, `PublishTrustAttestation`, `PublishLabel` variants to `SharedInboxActivities` enum — **before** `RawAnnouncableActivities` (catch-all must remain last) |
| `crates/apub/apub/src/lib.rs` | `pub mod governance;` |
| `crates/api/api/src/governance/governance_log.rs` | Add four new entry-kind const strings (see `GOVERNANCE_LOG_APPEND` pattern above) |
| `crates/api/api/src/governance/submit_jury_vote.rs` | Insert `send_local_sanction_notice` call between reputation-delta block (ends line 395) and `case_decided` log entry (line 397) — **only when winning sanction has `scope == SanctionScope::FederatedRecommendation`** |
| `crates/server/tests/e2e.rs` | New `#[tokio::test]` function `sanction_notice_round_trip` per TWO_DB_TEST_PATTERN |
| `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md` | Append four new entry kinds to §Entry kinds (non-breaking extension per §Stability contract) |
| `Cargo.lock` | Regenerated by cargo when any Cargo.toml dependency block changes (no manual edit) |

**No Cargo.toml changes expected** — audit confirmed `activitypub_federation`, `sha2`, `ed25519-dalek`, `serde_json`, `chrono`, `url`, `enum_delegate` are all workspace deps and all apub sub-crates already import what Phase 6 needs.

---

## NOT Building (v0 scope limits)

The following are **explicitly deferred** per ADR-010 and [05 §3](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md):

- **Peer/instance allowlist table** — v0 treats every federated peer as `Allow`. A `federation_peer` table with `state IN ('Allow','Limit','Block')` is v1 work. Task 74's `to_all_instances()` is the v0 proxy.
- **Auto-apply of remote sanctions** — ADR-006 is non-negotiable. Any PR that routes `remote_sanction_notice` writes through local sanction application paths is a bug, not a feature.
- **Undo activities** — `Undo` variants for revoked attestations / rescinded sanctions are stubs in Phase 6 (empty file, type alias). Wire-up is v1 (reaction to `RevokeEndorsement` or admin rescission).
- **Outbound retry UX** — Lemmy's existing `sent_activity` queue retries with backoff. No governance-specific retry logic.
- **Federation rate-limiting per source** — [06 §2.5](../../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) says "apply" but v0 relies on Lemmy's existing inbox rate-limit. Governance-specific per-source rate-limit is v1.
- **SSRF-isolated media fetch worker** — [06 §2.5](../../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) v2.
- **Signed build artefacts / Sigstore** — ADR-010 v2.
- **HTTP route for admin review of `remote_sanction_notice`** — the "admin review" DoD means "the row exists and is queryable via psql / future admin UI". A REST endpoint for surfacing advisory notices to admins is v1 (requires new DTO, new `db_views/remote_sanction` crate, new route).
- **UI surfacing** — v0 is backend only. The public case log does not currently render `remote_sanction_notice` rows.
- **`federation_attestation` write path from local events** — Phase 6 ships the **table** and the AP type + Activity + send function (task 74 has `send_local_trust_attestation`), but **no v0 endpoint emits a trust attestation**. The endorsement handler is not wired to publish; that's v1. The function exists to prove the shape is right and will be wired in v1.

---

## Step-by-Step Tasks

Nine tasks, five execution waves. Each task is one commit. Validation commands listed use the Windows wrappers at `scripts/brehon/cargo-*.bat` (set `PQ_LIB_DIR` for libpq parity — see pre-phase audit).

### Task 70 — Migration `add_federation_attestations` (LAYER 1, Agent A)
- **ACTION**: Create `migrations/{timestamp}_add_federation_attestations/{up,down}.sql`. Timestamp must be strictly greater than `2026-04-20-000000-0000`. Use `diesel migration generate add_federation_attestations` if wrappers support it, otherwise handwrite.
- **IMPLEMENT** (up.sql): two tables per [04 §3](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) lines 361–405:
  ```sql
  CREATE TABLE federation_attestation (
    id            SERIAL PRIMARY KEY,
    actor_url     TEXT NOT NULL,
    subject_url   TEXT NOT NULL,
    attestation_type attestation_type NOT NULL,  -- reuse existing enum from add_governance_enums
    valid_until   TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    signature     TEXT NOT NULL
  );
  CREATE INDEX idx_fed_attestation_subject ON federation_attestation (subject_url);
  CREATE INDEX idx_fed_attestation_actor   ON federation_attestation (actor_url);

  CREATE TABLE remote_sanction_notice (
    id              SERIAL PRIMARY KEY,
    source_instance TEXT NOT NULL,
    target_url      TEXT NOT NULL,
    action          sanction_action NOT NULL,
    scope           sanction_scope  NOT NULL,
    summary         TEXT NOT NULL,
    published_at    TIMESTAMPTZ NOT NULL,
    signature       TEXT NOT NULL,
    local_case_id   INT REFERENCES moderation_case(id) ON DELETE SET NULL,
    received_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX idx_remote_sanction_notice_target ON remote_sanction_notice (target_url);
  CREATE INDEX idx_remote_sanction_notice_source ON remote_sanction_notice (source_instance, received_at);
  ```
  (down.sql): DROP TABLE both tables in reverse order.
- **MIRROR**: `migrations/2026-04-15-100100-0000_add_governance_core/up.sql` for style; reuse existing enums from `2026-04-15-100000-0000_add_governance_enums` (**verify enum names via `grep` first** — the audit flagged that plan doc names may drift from actual DB names).
- **GOTCHA**: `attestation_type` enum: confirm the DB enum name is `attestation_type_enum` vs `attestation_type` vs `attestationtype` via `diesel print-schema`. The audit found drift between plan naming and actual enum names for `CaseStatus` (`JurySelecting` → `JurySelection`); same class of error could hit here. **If the enum doesn't exist yet, add it in this migration** and document in the up.sql comment header.
- **GOTCHA**: `received_at` column is NOT in [04 §3 RemoteSanctionNotice](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) (which lists `published_at` only). Added because the design doc is silent on when the local instance received it — useful for admin-review queries ("show notices received in last 24h"). **Flag as decision-queue entry DQ-6.1** before committing.
- **GOTCHA**: `signature TEXT NOT NULL` — the signature is the HTTP-signature header value of the inbound activity, captured as-is. v0 stores for audit; v1 may re-verify on re-read.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema_file > .claude/build-task70.log 2>&1"
  tail -20 .claude/build-task70.log
  # Plus: inspect down.sql round-trip via task 71 (migration runs as part of DB test there)
  ```
- **COMMIT MESSAGE**:
  ```
  feat(governance): task 70 — add_federation_attestations migration

  Two tables per [04 §1 Migration 4]: federation_attestation holds
  outbound-federation attestations (TrustedReporter, JuryEligible, etc);
  remote_sanction_notice stores advisory inbound notices with
  local_case_id NULL (ADR-006: never auto-applied).
  ```

### Task 71 — Diesel models for federation tables (LAYER 1, Agent A)
- **ACTION**: Create `crates/db_schema/src/source/governance/federation_attestation.rs` and `crates/db_schema/src/source/governance/remote_sanction_notice.rs`. Update `crates/db_schema/src/source/governance/mod.rs` and `crates/db_schema/src/newtypes.rs`.
- **IMPLEMENT**:
  - `FederationAttestation` + `FederationAttestationInsertForm` (Queryable/Selectable/Identifiable + Insertable)
  - `RemoteSanctionNotice` + `RemoteSanctionNoticeInsertForm`
  - Newtype `FederationAttestationId(pub i32)` and `RemoteSanctionNoticeId(pub i32)` in `newtypes.rs`
- **MIRROR**: `crates/db_schema/src/source/governance/sanction.rs` field-for-field — imports, derives, `#[diesel(table_name = ...)]`, `pub struct X { pub id: XId, ... }`.
- **GOTCHA**: `moderation_case.id` is `ModerationCaseId(pub i32)`. The FK column `local_case_id` must be typed as `Option<ModerationCaseId>` (not `Option<i32>`) or Diesel's `RunQueryDsl` loses the newtype. Check the Sanction model for the convention.
- **GOTCHA**: Per `.claude/rules/view-crate-selectable-template.md` — `RemoteSanctionNotice` has no bare-scalar fields so `Selectable` derive is fine. `FederationAttestation` ditto. No tuple-load fallback needed here (unlike Phase 2a views).
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema > .claude/build-task71.log 2>&1"
  tail -20 .claude/build-task71.log
  # exit must be 0
  ```
- **COMMIT MESSAGE**:
  ```
  feat(governance): task 71 — Diesel models for federation tables

  FederationAttestation + RemoteSanctionNotice with their insert forms
  and newtype IDs. Follows the Sanction model pattern. Advisory rows
  keep local_case_id: Option<ModerationCaseId> (NULL for unmatched
  inbound notices per ADR-006).
  ```

### Task 72 — AP object types (LAYER 2, Agent B — parallel with Agent C)
- **ACTION**: Create the governance/ subdirectory under `crates/apub/objects/src/` and under `crates/apub/objects/src/protocol/`. Add all three object types and their protocol structs.
- **IMPLEMENT**:
  - `protocol/governance/sanction_notice.rs` — `SanctionNoticeProtocol` per `PROTOCOL_STRUCT` pattern above
  - `protocol/governance/trust_attestation.rs` — `TrustAttestationProtocol`
  - `protocol/governance/moderation_label.rs` — `ModerationLabelProtocol`
  - `governance/sanction_notice.rs` — `ApubSanctionNotice` newtype + `impl Object`
  - `governance/trust_attestation.rs` — `ApubTrustAttestation` newtype + `impl Object`
  - `governance/moderation_label.rs` — `ApubModerationLabel` newtype + `impl Object` (stubs acceptable — outbound only in v0)
- **MIRROR**: `crates/apub/objects/src/objects/private_message.rs` (simpler Object) for `read_from_id` returning None and `delete` returning NotFound. `crates/apub/objects/src/protocol/note.rs` for protocol struct attributes.
- **GOTCHA**: `SanctionNoticeProtocol.kind` — must be a constant type (like `NoteType::Note` in `note.rs`). The AP spec's `type` field is a discriminator for untagged deserialisation. Define a `SanctionNoticeType` enum with one variant (`SanctionNotice`) and use serde's `rename` to make it serialise/deserialise as the string "SanctionNotice". Mirror the pattern at `activitypub_federation::kinds::object::NoteType`.
- **GOTCHA**: `target` field is `Url` (outbound) but should accept `ObjectId<T>` if it needs to be dereferenced on receive. For v0 outbound-only use Url; do not implement the inbound dereference chain. Document in the struct comment.
- **GOTCHA**: `ap_id` naming (ADR-012) — when the struct wraps a local DB row (e.g. on receive, wrapping `RemoteSanctionNotice`), call the field `ap_id`, not `actor_id`. The audit confirmed codebase is already on `ap_id`; do not introduce `actor_id` anywhere.
- **GOTCHA**: `Object::verify` for SanctionNotice — at minimum, `verify_domains_match(object.id.inner(), expected_domain)?`. The actor's authority to sanction is **not** verified in v0 (v2 item); just verify structural integrity and that the activity comes from the instance it claims to.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub_objects > .claude/build-task72.log 2>&1"
  tail -20 .claude/build-task72.log
  # exit must be 0
  # Do NOT combine -p lemmy_apub_objects with --features full
  # (per feedback_features_full_p_crate_incompatible.md)
  ```
- **COMMIT MESSAGE**:
  ```
  feat(governance): task 72 — AP object types for governance

  Three new AP objects — SanctionNotice, TrustAttestation,
  ModerationLabel — mirroring PrivateMessage for simpler Object impls.
  Protocol structs use #[skip_serializing_none] + camelCase + serde
  rename for the "type" discriminator.
  ```

### Task 73 — AP activity types (LAYER 2, Agent C — parallel with Agent B)
- **ACTION**: Create `crates/apub/activities/src/governance/` + `crates/apub/activities/src/protocol/governance/`. Add three Create wrappers and register them in `activity_lists.rs`.
- **IMPLEMENT**:
  - `protocol/governance/publish_sanction_notice.rs` — `PublishSanctionNoticeProtocol` (the serde struct — wraps the object from task 72)
  - `protocol/governance/publish_trust_attestation.rs` — `PublishTrustAttestationProtocol`
  - `protocol/governance/publish_label.rs` — `PublishLabelProtocol`
  - `governance/publish_sanction_notice.rs` — `PublishSanctionNotice` + `impl Activity` per `AP_ACTIVITY_TRAIT_IMPL`. **The `send` helper is in task 74**, not here — this task just defines the type and trait impl.
  - `governance/publish_trust_attestation.rs` — same pattern
  - `governance/publish_label.rs` — stub (empty `verify`/`receive` acceptable in v0 as it's not wired from any handler)
  - Update `activity_lists.rs` per `ACTIVITY_ENUM_REGISTRATION`
- **MIRROR**: `crates/apub/activities/src/community/report.rs` for a single-target moderation-adjacent Create activity; `crates/apub/activities/src/block/block_user.rs` for the enum registration pattern.
- **GOTCHA**: `#[serde(untagged)]` on `SharedInboxActivities` means serde tries variants in **declaration order**. New variants must be inserted **before** `RawAnnouncableActivities` (catch-all). If the three new variants have overlapping shapes (shouldn't — they each have a distinct `type` field on the wrapped object), ordering matters. Test by deserialising a known-good `PublishSanctionNotice` JSON and asserting it matches the right variant.
- **GOTCHA**: `PublishSanctionNotice::receive` calls into `crates/apub/apub/src/governance/inbox.rs::receive_remote_sanction_notice` — which is Layer 3 code. Agent C **stubs** the receive body with `Ok(())` + a `// TODO(task75): wire receive_remote_sanction_notice` comment. Agent E (Layer 3) fills the body. This is an intentional seam to allow parallel execution.
- **GOTCHA**: `generate_activity_id(kind, context)` — see `crates/apub/activities/src/community/report.rs:60`. Use the same helper; don't roll your own.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub_activities > .claude/build-task73.log 2>&1"
  tail -20 .claude/build-task73.log
  # exit must be 0
  cmd //c "scripts\\brehon\\cargo-clippy.bat -p lemmy_apub_activities --no-deps -- -D warnings > .claude/clippy-task73.log 2>&1"
  tail -20 .claude/clippy-task73.log
  # exit must be 0
  ```
- **COMMIT MESSAGE**:
  ```
  feat(governance): task 73 — AP activities for governance

  Three Create wrappers — PublishSanctionNotice, PublishTrustAttestation,
  PublishLabel — registered in SharedInboxActivities before the
  RawAnnouncableActivities catch-all. verify() delegates to
  verify_domains_match; receive() stubbed pending task 75.
  ```

---

**MERGE POINT 1 (after tasks 72 + 73 complete):** advisor merges both agent worktrees into phase-6, runs:
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/merge1-check.log 2>&1"
tail -30 .claude/merge1-check.log
```
Exit must be 0 before Layer 3 spawns. If Agents B and C took conflicting liberties with shared types (e.g. both renamed `ap_id` to `actor_id`), this is where it surfaces.

---

### Task 74 — Outbound publisher (LAYER 3, Agent D — parallel with Agent E)
- **ACTION**: Implement `send_local_sanction_notice(case_id, context)` and `send_local_trust_attestation(person_id, attestation_type, context)` in `crates/apub/activities/src/governance/publish_sanction_notice.rs` (and the sibling file). Re-export from `crates/apub/apub/src/governance/outbox.rs`.
- **IMPLEMENT**: per `SEND_HELPER` pattern.
  1. Load `ModerationCase` + winning `Sanction` row from DB
  2. Load actor `Person` (local admin who finalised the case — `case.creator_id` is not always the right actor; for v0 use the **system local admin** per `actor_pseudonym_helper` convention, which already single-admin-v0 per ADR-010)
  3. Load target entity (Person/Post/Comment/Community) to get `ap_id`
  4. Call `redaction::scrub(&sanction.reason)` to build the redacted summary
  5. Construct `SanctionNoticeProtocol` struct with `published = Utc::now()`
  6. Wrap in `PublishSanctionNotice { id: generate_activity_id(..., context)?, actor: actor.ap_id.clone().into(), to: [public_url()], cc: [], object: sanction_notice_protocol }`
  7. `let targets = ActivitySendTargets::to_all_instances();`
  8. `send_lemmy_activity(context, publish, &actor, targets, false).await?;`
  9. `governance_log::append(&mut context.pool(), ENTRY_KIND_FEDERATION_SANCTION_SENT, json!({"case_id": case_id.0, "sanction_id": sanction.id.0, "target_url": target_ap_id.to_string()}), Some(actor_pseudonym))?;`
- **MIRROR**: `crates/apub/activities/src/block/block_user.rs:65-96` (`send` function shape) + `crates/apub/activities/src/community/report.rs:62-74` (Report::send).
- **GOTCHA**: `public_url()` — the `to` field for a public AP broadcast is the magic URL `https://www.w3.org/ns/activitystreams#Public`. Check if a constant exists in `activitypub_federation::kinds` (likely `public()` or similar). Do not hardcode the string.
- **GOTCHA**: The `actor` loaded in step 2 must have a valid signing key (checked via `GetActorType`). Local admins always do; other actors may not. Use `SiteActor` or `Person` per Lemmy's existing mod-action outbound path.
- **GOTCHA**: **Transaction boundary** — steps 1–7 build the activity without DB writes; step 8 `send_lemmy_activity` writes to `sent_activity` (one INSERT); step 9 writes to `governance_log` (trigger-hashed INSERT + UPDATE for signature). If step 9 fails, the activity is already enqueued and will be delivered. That's acceptable (the event happened) but the governance log will be missing. **Per [06 §2.3](../../../docs/brehon-law-inspired-network/06-security-and-threat-model.md): "before the user response returns"** — step 9 must succeed. Wrap steps 8+9 in a `conn.run_transaction()` per `feedback_multi_write_handlers_need_transactions.md`.
- **GOTCHA**: `send_local_trust_attestation` — the audit flagged that **no v0 endpoint emits a trust attestation**. This function is plumbed but never called in v0. Ship it anyway (proves the shape), but do not add an endpoint. v1 wires this into endorsement creation.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub_activities > .claude/build-task74-act.log 2>&1"
  tail -20 .claude/build-task74-act.log
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub > .claude/build-task74-apub.log 2>&1"
  tail -20 .claude/build-task74-apub.log
  # both exit 0
  ```
- **COMMIT MESSAGE**:
  ```
  feat(governance): task 74 — outbound publisher

  send_local_sanction_notice and send_local_trust_attestation build
  the AP activity, enqueue via send_lemmy_activity, and emit a
  federation_sanction_sent governance log entry inside a transaction.
  v0 uses ActivitySendTargets::to_all_instances (no peer table).
  ```

### Task 75 — Inbound receiver (LAYER 3, Agent E — parallel with Agent D)
- **ACTION**: Fill the `receive` bodies stubbed by Agent C in task 73. Create free functions in `crates/apub/apub/src/governance/inbox.rs`.
- **IMPLEMENT**:
  - `receive_remote_sanction_notice(activity: PublishSanctionNotice, context: &Data<LemmyContext>) -> LemmyResult<()>`
    1. **Signature already verified** by `activitypub_federation::actix_web::inbox::receive_activity_with_hook` before this function is called. Do not re-verify.
    2. **Schema validate**: `activity.object.action` and `.scope` must deserialise to known `SanctionAction` / `SanctionScope` variants. Diesel's derive-enum rejects unknown, so this is a deserialisation error at the activity-parse step — if we get here, schema is valid.
    3. `let form = RemoteSanctionNoticeInsertForm { source_instance: activity.actor.inner().domain().to_string(), target_url: activity.object.target.to_string(), action: activity.object.action, scope: activity.object.scope, summary: activity.object.summary, published_at: activity.object.published, signature: /* header value — see below */, local_case_id: None };`
    4. `RemoteSanctionNotice::create(&mut context.pool(), &form).await?;`
    5. `governance_log::append(&mut context.pool(), ENTRY_KIND_FEDERATION_SANCTION_RECEIVED, json!({"source_instance": form.source_instance, "target_url": form.target_url, "action": form.action.to_string()}), None)?;` — `actor_pseudonym = None` because the actor is remote (no local pseudonym exists).
    6. **Do not** call into any local sanction / removal code. ADR-006.
  - `receive_remote_trust_attestation(activity: PublishTrustAttestation, context: &Data<LemmyContext>) -> LemmyResult<()>`
    1. Same shape. Write to `FederationAttestation` table (v0: inbound attestations stored in the same table as outbound, keyed by source via `actor_url`).
    2. Governance log: `ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED`.
  - Wire `PublishSanctionNotice::receive` (stub from task 73) to call `receive_remote_sanction_notice(self, context).await`.
  - Wire `PublishTrustAttestation::receive` similarly.
  - `PublishLabel::receive` remains a stub with `Ok(())` in v0.
- **MIRROR**: `crates/apub/activities/src/community/report.rs::receive` for the shape; use `self.actor.dereference(context).await?` if you need the remote actor row.
- **GOTCHA**: **Signature capture** — the HTTP signature header is consumed by `activitypub_federation` before `Activity::receive` is called. The raw header value is not trivially available inside `receive`. For v0, store the signature value from `activity.object.id.to_string()` or an empty string with a `// TODO(v1): plumb actual HTTP signature through activitypub_federation hook`. **Flag as DQ-6.2**.
- **GOTCHA**: **Idempotency** — if the same notice is received twice (retry from sender), inserting the same `RemoteSanctionNoticeInsertForm` creates two rows. Add a partial unique index `(source_instance, target_url, published_at)` in task 70's up.sql? Or handle duplicate inserts via `on_conflict().do_nothing()`? The audit did not find `ReceivedActivity` dedup logic touching governance. **Flag as DQ-6.3** — current Lemmy `Dummy::hook` in `shared_inbox` already deduplicates by `activity.id()` via `ReceivedActivity::create`, which should prevent double-receive at the HTTP layer. Verify before adding a second safeguard.
- **GOTCHA**: **Plugin-hook preservation** (`.claude/rules/pm-plugin-hooks-stable.md`) — this rule applies to private-message hooks, **not** governance. Phase 6 doesn't touch PM code; no hook impact.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub > .claude/build-task75.log 2>&1"
  tail -20 .claude/build-task75.log
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub_activities > .claude/build-task75-act.log 2>&1"
  tail -20 .claude/build-task75-act.log
  # both exit 0
  ```
- **COMMIT MESSAGE**:
  ```
  feat(governance): task 75 — inbound receiver

  receive_remote_sanction_notice writes an advisory row with
  local_case_id NULL and emits federation_sanction_received in the
  governance log. Never applies the sanction locally (ADR-006).
  Signature verification delegates to activitypub_federation's
  inbox HTTP signature check.
  ```

### Task 78 — verify.rs signature verification helper (LAYER 3, Agent E — same worktree as task 75)
- **ACTION**: Create `crates/apub/apub/src/governance/verify.rs`. Thin wrapper over existing Lemmy/`activitypub_federation` helpers — home for future custom verification logic.
- **IMPLEMENT**:
  ```rust
  // crates/apub/apub/src/governance/verify.rs
  use activitypub_federation::{config::Data, protocol::verification::verify_domains_match};
  use lemmy_api_utils::context::LemmyContext;
  use lemmy_utils::error::LemmyResult;
  use url::Url;

  /// Shared verification helpers for governance federation activities.
  /// Thin wrappers today; future custom verification (e.g. jury-panel attestation
  /// strength) will be added here per [04 §11].
  pub fn verify_governance_activity_domain(activity_id: &Url, expected_domain: &Url) -> LemmyResult<()> {
      verify_domains_match(activity_id, expected_domain)
  }
  ```
- **MIRROR**: Any existing `verify.rs` in the apub crates (e.g. `crates/apub/apub/src/fetcher/post_or_comment.rs` has verification idioms).
- **GOTCHA**: Task 78 was bundled with task 75 per the original plan for a reason — Agent E owns both, one commit each, serialised within Agent E's worktree. This is **not parallelisable** with task 75 because both edit `crates/apub/apub/`.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub > .claude/build-task78.log 2>&1"
  tail -20 .claude/build-task78.log
  # exit 0
  ```
- **COMMIT MESSAGE**:
  ```
  feat(governance): task 78 — verify.rs signature helper

  Thin wrapper over verify_domains_match for governance federation
  activities. Home for future custom verification beyond the base
  HTTP-signature check that activitypub_federation handles.
  ```

---

**MERGE POINT 2 (after tasks 74, 75, 78 complete):** advisor merges Agent D and Agent E worktrees. Runs:
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/merge2-check.log 2>&1"
tail -30 .claude/merge2-check.log
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/merge2-clippy.log 2>&1"
tail -30 .claude/merge2-clippy.log
```
Both must exit 0 before Layer 4.

---

### Task 76 — Wire `submit_jury_vote` to publish on FederatedRecommendation (LAYER 4, Agent F)
- **ACTION**: Insert a call to `send_local_sanction_notice(case_id, &context)` in `crates/api/api/src/governance/submit_jury_vote.rs` **between** the reporter reputation update (ending ~line 395) and the `case_decided` log entry (starting ~line 397).
- **IMPLEMENT**:
  ```rust
  // After reporter_reputation block, before case_decided log append:
  if winning_sanction.scope == SanctionScope::FederatedRecommendation {
      crate::governance::federation_outbox::send_local_sanction_notice(case_id, &context).await?;
  }
  ```
  Wrap the entire post-decision block (sanction insert → sponsor-liability → case flip → public_log → reputation deltas → federation publish → case_decided) in a single `conn.run_transaction()` if not already. Audit showed Phase 5b's sanction insert + sponsor-liability are already transactional; verify the scope includes the new federation call.
- **MIRROR**: The existing branching pattern at `submit_jury_vote.rs:457-460` which matches on `JuryDecision::RecommendFederationAction`. The audit confirmed `SanctionScope::FederatedRecommendation` is the exact variant name (not `Federated` or `FederationQuarantineRecommendation` — that's the `SanctionAction`).
- **GOTCHA**: **Ordering** — from the audit, the actual sequence is: sanction insert (line 238) → sponsor-liability (line 259) → case flip (line 273) → public_log (line 295) → juror reputation (line 347) → reporter reputation (line 385) → case_decided (line 397). The new federation publish must go **between line 395 (reporter reputation complete) and line 397 (case_decided log)**. Any other location either publishes before the case is officially decided (wrong) or after (loses the causality chain).
- **GOTCHA**: **`context` is a `Data<LemmyContext>`** — the outbound publisher signature takes `&Data<LemmyContext>`, not `&LemmyContext`. Inside the handler, `context` is already `Data<LemmyContext>` so pass `&context`.
- **GOTCHA**: **Transaction semantics** — if `send_local_sanction_notice` fails (e.g. `sent_activity` insert fails on the DB), the entire `submit_jury_vote` call should fail and roll back the sanction/case decision. This is the **correct** behaviour for v0: we don't want half-decided cases where the local state says "Decided" but no peer was notified. Confirm that `.await?` propagation is inside the transaction boundary.
- **GOTCHA**: **Early-exit path** at `submit_jury_vote.rs:192-197` (pre-quorum return) must **not** hit the federation publish. The `if winning_sanction.scope == ...` check inside the post-decision block guarantees this.
- **GOTCHA**: **Test implication** — existing test `all_mvp_endpoints_return_non_404` (Phase 5c task 68) calls `submit_jury_vote` and may now trip the federation publish if it seeds a `FederatedRecommendation` vote. Audit showed task 68 votes `NoAction` and `RemoveContent`, not `RecommendFederationAction`, so no regression. But verify by re-running task 68's test as part of the DoD.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/build-task76.log 2>&1"
  tail -20 .claude/build-task76.log
  # existing test suite must still pass:
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/build-task76-test.log 2>&1"
  tail -20 .claude/build-task76-test.log
  # both exit 0 (test builds, not runs yet — task 77 adds the new test)
  ```
- **COMMIT MESSAGE**:
  ```
  feat(governance): task 76 — wire submit_jury_vote to federation publish

  When the winning sanction has scope = FederatedRecommendation,
  send_local_sanction_notice fires between reporter reputation
  deltas and the case_decided log entry. Inside the existing
  post-decision transaction so failure rolls back the whole
  decision.
  ```

---

**MERGE POINT 3 (after task 76):** trivial (one agent, one file). Advisor runs the same workspace check + clippy sweep before Layer 5.

---

### Task 77 — Federation integration test (LAYER 5, Agent G)
- **ACTION**: Add `#[tokio::test(flavor = "multi_thread")] async fn sanction_notice_round_trip()` to `crates/server/tests/e2e.rs`. Update `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md` in the same commit with the four new entry kinds.
- **IMPLEMENT**: per `TWO_DB_TEST_PATTERN`.
  1. Boot container A, apply schema, build `context_a`
  2. Boot container B, apply schema, build `context_b`
  3. Seed instance A: create `Instance`, admin `Person` (via `seed_person`), target `Person`, open a `ModerationCase` directly via `ModerationCase::create` (bypass `create_report` for test speed), assign a 5-juror panel via `admin_assign_jury`, accept all 5, submit 3 `RecommendFederationAction` votes to trip quorum
  4. Assert instance A's `sent_activity` table has exactly one row with `activity.type == "PublishSanctionNotice"`
  5. Deserialise the `sent_activity.data` JSONB into a `PublishSanctionNotice` struct
  6. Call `PublishSanctionNotice::receive(activity, &context_b).await?` **directly** (no HTTP, per [IMPLEMENTATION-PLAN-v0.md §3 Phase 6 task 77](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): "Do not require a real HTTP federation transport for this test — call the inbox function directly")
  7. Assert instance B has exactly one `remote_sanction_notice` row with: `local_case_id IS NULL`, `action = FederationQuarantineRecommendation`, `scope = FederatedRecommendation`, `target_url` matches the target from A, `source_instance = "instance-a.test"`, `summary` is non-empty and redacted
  8. Assert instance B's `governance_log` has exactly one `federation_sanction_received` entry for this notice
  9. Cleanup: containers drop automatically at test end (they're owned in locals)
- **MIRROR**: `report_to_modlog_golden_path` (e2e.rs:743-1300) for multi-step handler invocation + DB state assertions; `sponsor_liability_with_founder_multiplier` (1406-1969) for multi-branch flow.
- **GOTCHA**: **`LEMMY_DATABASE_URL` env swap** — the existing tests at e2e.rs:2195+ use `unsafe { std::env::set_var(...) }` to swap. Two-DB tests need to swap between instances. **Build pool A fully before swapping to instance B**. Do NOT interleave:
  ```rust
  // CORRECT
  std::env::set_var("LEMMY_DATABASE_URL", &url_a);
  let pool_a = build_db_pool_for_tests();  // reads env, completes
  let context_a = LemmyContext::create(pool_a, ...);
  // ...
  std::env::set_var("LEMMY_DATABASE_URL", &url_b);
  let pool_b = build_db_pool_for_tests();
  let context_b = LemmyContext::create(pool_b, ...);

  // WRONG (pool_b may see url_a or vice versa)
  let (pool_a, pool_b) = tokio::join!(build_pool_with_env(&url_a), build_pool_with_env(&url_b));
  ```
  Use a single-threaded sequencing even though the test is `flavor = "multi_thread"` (the multi-thread flavor is needed for the pool's internal async work, not for setup parallelism).
- **GOTCHA**: **Rate-limit on multi-endpoint test** — per `feedback_rate_limit_debug_config_post_bucket.md`, `RateLimit::with_debug_config()` has a POST bucket of 6/300s. The test hits `admin_assign_jury` + 5 × `accept_jury_assignment` + 5 × `submit_jury_vote` + `create_report` = 12 POSTs on instance A alone. Either raise the bucket via `set_config` or route half the calls through direct `ModerationCase::create` / `JuryAssignment::update` (bypassing handlers).
- **GOTCHA**: **Error bridging** — per audit, tests return `Result<(), Box<dyn Error>>` with `.map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?`. LemmyError does not implement `std::error::Error`. All handler calls need the explicit `map_err`.
- **GOTCHA**: **Seeding shortcut** — instead of going through `create_report` → threshold → admin_assign_jury → 5×accept → 5×vote (slow), seed the `moderation_case` and `sanction` rows directly via raw SQL or `ModerationCase::create` + `Sanction::create`. Task 76 wiring is tested by the fact that `send_local_sanction_notice` is called from the code path; verifying the whole chain from report upward is Phase 5's golden-path test's job, not this one.
- **GOTCHA**: **Env-var leakage across tests** — `LEMMY_DATABASE_URL` is set globally. If another test in e2e.rs runs after this one, it inherits the wrong URL. Use `std::env::set_var` / `std::env::remove_var` defensively at start and end (or `#[serial]` via `serial_test` crate if available). The existing pattern at 2195+ does not always clean up — this is a latent bug. Document but do not fix in Phase 6; flag as DQ-6.4.
- **GOTCHA**: **Two containers = ~40s startup on cold boot**. `governance_fixtures::start_postgres` waits for Postgres ready signal. Two sequential boots roughly double the time. Acceptable for v0 (one test, pre-merge only).
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server sanction_notice_round_trip > .claude/test-task77.log 2>&1"
  tail -50 .claude/test-task77.log
  # exit 0 AND "test result: ok. 1 passed" in tail
  # Also: run full e2e to confirm no regression
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/test-task77-full.log 2>&1"
  tail -30 .claude/test-task77-full.log
  # exit 0 AND all tests pass
  ```
- **COMMIT MESSAGE**:
  ```
  feat(governance): task 77 — sanction_notice_round_trip e2e

  Two-Postgres test: instance A decides a case with
  FederatedRecommendation scope, PublishSanctionNotice is enqueued,
  directly delivered to instance B's receive function, advisory row
  lands in remote_sanction_notice with local_case_id NULL.
  SUBSCRIPTIONS.md updated with four new federation entry kinds.

  Closes Phase 6 DoD.
  ```

---

## Layer Structure (execution waves)

```text
LAYER 1 (sequential, 1 agent):
  Agent A                    [tasks 70 + 71 — schema + models]
         │
    merge point 1
         │
LAYER 2 (parallel, 2 agents in separate worktrees):
  Agent B  ─┬─                [task 72 — AP objects]
  Agent C  ─┴─                [task 73 — AP activities]
         │
    merge point 1 (workspace check + clippy)
         │
LAYER 3 (parallel, 2 agents in separate worktrees):
  Agent D  ─┬─                [task 74 — outbound publisher]
  Agent E  ─┴─                [tasks 75 + 78 — inbound + verify]
         │
    merge point 2 (workspace check + clippy)
         │
LAYER 4 (sequential, 1 agent):
  Agent F                    [task 76 — wire submit_jury_vote]
         │
    merge point 3 (workspace check + clippy + existing test suite)
         │
LAYER 5 (sequential, 1 agent):
  Agent G                    [task 77 — federation round-trip test]
         │
    merge point 4 (full e2e suite must pass)
         │
    PR open: phase-6 → governance-v0
```

### Why this structure

- **Layer 1 is sequential** because tasks 70 and 71 edit the same files (schema.rs, governance/mod.rs, newtypes.rs). Parallel would race.
- **Layer 2 is parallel** because task 72 lives in `crates/apub/objects/` and task 73 lives in `crates/apub/activities/`. The only shared touch is `activity_lists.rs` (task 73 only). Agent B and Agent C work in separate worktrees to prevent index races per `feedback_parallel_agents_one_worktree_per_agent.md`.
- **Layer 3 is parallel** because task 74 lives in `crates/apub/activities/src/governance/` and task 75 lives in `crates/apub/apub/src/governance/`. They share no files. Agent E bundles task 78 because both edit `crates/apub/apub/src/governance/` (serialised within the agent).
- **Layer 4 is sequential + single-agent** because task 76 edits the hottest file in the project (`submit_jury_vote.rs`) and needs full attention to preserve Phase 5b's transaction semantics.
- **Layer 5 is sequential + single-agent** because task 77 is a single test file edit and needs the full picture of what was built in layers 1–4.

### Agent brief template (see `AGENT_BRIEFS.md` generated alongside this plan)

Every agent receives a brief. Below is the template; the full briefs (one per agent, A through G) live in `.claude/PRPs/plans/phase-6-federation.agents.md` — generated as a sibling to this file before Layer 1 spawns.

```markdown
### Agent X: Layer N — [task list]

**Scope:** [1-2 sentence description of what this agent owns]

**Files to create:**
- path/to/new/file.rs — [purpose]

**Files to modify:**
- path/to/existing/file.rs — [what changes]

**Dependencies:** [which prior-layer commits must exist before this agent runs]

**Validation commands:**
- `cmd //c "scripts\\brehon\\cargo-check.bat -p <crate> > .claude/build-agentX.log 2>&1"`
- `cmd //c "scripts\\brehon\\cargo-clippy.bat -p <crate> --no-deps -- -D warnings > .claude/clippy-agentX.log 2>&1"`
- [any test commands]

**Commit message template:**
```
feat(governance): task NN — [short description]

[<=60 words body explaining the change + why]
```text

**Decision-queue pre-seeds:** [DQ-6.X entries this agent may hit; see §Decision Queue Pre-Seeds]

**Catch-fire triggers (stop and surface):**
- Wrapper script discards `-p <crate>` or loses exit code (pre-phase audit signal)
- Workspace cargo check red on commit — roll back, do not force-push
- `activitypub_federation` API mismatch (likely 0.7.0-beta version drift) — flag DQ before continuing
- Any need to rename an AP type across crates after sibling agent has committed it — stop, merge-point alignment required

**Worktree:**
- Agent X owns `../brehon-fork-agentX-phase6` via `git worktree add ../brehon-fork-agentX-phase6 -b agentX-phase6 phase-6`
- At task end: `git push origin agentX-phase6`; advisor merges into phase-6 at the merge point
- At agent end: `git worktree remove ../brehon-fork-agentX-phase6`
```

---

## Pre-Phase Harness Audit (MANDATORY, Layer 1 step 0)

Per `.claude/rules/pre-phase-harness-audit.md`. **Do not spawn Agent A until these four probes exit 0.** Agent A itself runs the audit as its first action; advisor reviews probe output before Agent A commits task 70.

```bash
# Probe 1 — per-crate check honours -p
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema_file > .claude/audit-check-p.log 2>&1"
tail -20 .claude/audit-check-p.log
# Expected: only lemmy_db_schema_file compiles

# Probe 2 — feature-flag activation (workspace-only per feedback_features_full_p_crate_incompatible.md)
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/audit-features.log 2>&1"
tail -20 .claude/audit-features.log
# Expected: compiles clean

# Probe 3 — cargo-test wrapper honours target selection
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-test.log 2>&1"
tail -20 .claude/audit-test.log
# Expected: only e2e test target compiles

# Probe 4 — wrappers fail loud on cargo errors (exit-code propagation)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/audit-negative.log 2>&1"
echo "exit: $?"
# Expected: NON-ZERO exit. If 0, wrappers mask exit codes (issue #8 regression).

# Probe 5 (Phase 6-specific) — apub crates build per-package
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub_objects > .claude/audit-apub-obj.log 2>&1"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub_activities > .claude/audit-apub-act.log 2>&1"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub > .claude/audit-apub.log 2>&1"
# Expected: all three exit 0 — this phase edits all three crates

# Probe 6 (Phase 6-specific) — PR #10 merged, phase-6 branch exists, we're on it
git branch --show-current
# Expected: "phase-6" (not "phase-5c" — the advisor must complete pre-flight first)

gh pr view 10 --repo barrie-cork/lemmy --json state --jq .state
# Expected: "MERGED"

git log --oneline governance-v0..HEAD | head -5
# Expected: empty (phase-6 cut at governance-v0 HEAD, no commits yet)
```

**Probe 6 is the hard gate.** If it doesn't match expectations, stop and surface to the user.

---

## Testing Strategy

Per [IMPLEMENTATION-PLAN-v0.md §5](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): integration-only for v0; no unit tests until a bug breaks twice.

### Tests to Add

| Test Name | File | What It Validates |
|---|---|---|
| `sanction_notice_round_trip` | `tests/e2e.rs` | Two-instance federation: A decides → B stores advisory → no auto-apply |

### Tests That Must Still Pass (regression set)

All Phase 5 tests:
- `postgres_container_boots`
- `can_insert_moderation_case`
- `governance_log_hash_chain_holds`
- `phase1_migrations_round_trip`
- `list_open_cases_returns_seeded_rows`
- `jury_queue_view_returns_assignments`
- `modlog_view_returns_published_entries`
- `config_parity_round_trip`
- `sponsor_liability_with_founder_multiplier`
- `capability_change_entries_reachable_via_modlog_crate`
- `snapshot_staleness_alert_fires_when_max_calculated_at_is_old`
- `all_mvp_endpoints_return_non_404`
- `ineligible_user_cannot_be_picked_for_jury`
- `governance_events_notify_fires`
- `underscore_prefix_usernames_still_register`

**Particular regression risk:** `all_mvp_endpoints_return_non_404` calls `submit_jury_vote`. Task 76 adds a new code path; the existing test votes `NoAction` / `RemoveContent` (audit-verified) which don't trip `FederatedRecommendation`, so no regression — but the full e2e run at merge point 4 confirms this empirically.

### Edge Cases

- [ ] Hash-chain integrity still holds after `federation_sanction_sent` and `federation_sanction_received` writes (governance_log_hash_chain_holds, unchanged)
- [ ] `SanctionScope::FederatedRecommendation` is the only trigger for `send_local_sanction_notice`; other scopes must not publish (tested by deliberately voting `NoAction` in a branch of the round-trip test or a sibling test — optional)
- [ ] `local_case_id = NULL` strictly on the receiving side (ADR-006 enforcement — asserted by task 77)
- [ ] Redacted summary: the inbound `summary` on B must not contain usernames, emails, or URLs from A's local rationale — asserted by task 77
- [ ] Rollback migration (`diesel migration redo` on tasks 70's migration) preserves `local_case_id` FK behaviour (tested implicitly by `phase1_migrations_round_trip` once the test is extended to cover Phase 6's migration — do in a follow-up commit within task 77)
- [ ] Exhaustive `CaseStatus` match: Phase 6 doesn't introduce a new `CaseStatus` variant, but task 76 adds branching on `SanctionScope`. Compiler enforces exhaustiveness; `EmergencyRemove` not touched.
- [ ] Re-receive same activity (idempotency) — deferred to DQ-6.3

---

## Validation Commands (summary across phase)

### Level 1 — Static Analysis (every task)
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/level1-check.log 2>&1"
tail -20 .claude/level1-check.log
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/level1-clippy.log 2>&1"
tail -30 .claude/level1-clippy.log
# Both exit 0
```

### Level 2 — Integration Tests (tasks 76, 77)
```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/level2-e2e.log 2>&1"
tail -50 .claude/level2-e2e.log
# Exit 0; "test result: ok. N passed" where N includes sanction_notice_round_trip
```

### Level 3 — Full Build
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/level3-build.log 2>&1"
tail -20 .claude/level3-build.log
# Exit 0
```

### Level 4 — Migration Round-Trip (task 70 only)
Not directly scriptable on Windows due to the `forbid_diesel_cli` trigger; covered by task 71's `cargo check -p lemmy_db_schema` which embeds the migration via `embed_migrations!` and applies it during test setup. If deeper round-trip is needed, add a migration-redo test to `phase1_migrations_round_trip` as a sibling commit.

### Level 5 — Cross-Cutting Verification (every task with a handler edit)
- [ ] All federation writes call `governance_log::append(...)` (audit via `rg "federation_sanction_(sent|received)" crates/apub/ crates/api/`)
- [ ] No direct `person_id` or username in federation payloads (redaction applied in `send_local_sanction_notice`; receive path uses empty pseudonym)
- [ ] No new `_ =>` match arms on `CaseStatus` or `SanctionScope`
- [ ] SUBSCRIPTIONS.md updated (task 77 commit)

### Level 6 — Manual Validation (advisor at merge points)
```bash
# Confirm new entry kinds appear only in the intended places
rg "federation_sanction_(sent|received)|federation_attestation_(sent|received)" --type=rust

# Confirm no auto-apply: the inbox must not call sanction creation
rg -A 3 "receive_remote_sanction_notice" crates/
# Look for any call chain that reaches Sanction::create — FAIL if present

# Confirm send path: outbox must enqueue SentActivity
rg -A 5 "send_local_sanction_notice" crates/
# Look for send_lemmy_activity call — PASS
```

---

## Acceptance Criteria

- [ ] All 9 tasks (70–78) committed on `phase-6`, one commit per task
- [ ] Level 1 (workspace check + clippy) exits 0 at every merge point and on final phase-6 HEAD
- [ ] Level 2 (e2e tests) exits 0; `sanction_notice_round_trip` passes on first try (Phase 5 precedent: this target is realistic)
- [ ] Level 3 (full workspace build) exits 0
- [ ] `SUBSCRIPTIONS.md` updated with 4 new entry kinds (stability-contract-compliant append)
- [ ] No new `cargo clippy` warnings introduced beyond workspace deny set
- [ ] No contradictions with the 15 ADRs in [99](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md). Specifically verified:
  - ADR-006: no inbound activity path calls `Sanction::create` or any local sanction application code
  - ADR-012: all new field names use `ap_id`, never `actor_id`
  - ADR-014: vanilla Lemmy instances receiving our activities ignore them (untagged deserialisation falls through to `RawAnnouncableActivities`; no crash)
- [ ] PR from `phase-6` → `governance-v0` opened via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-6` (not squash)
- [ ] CodeRabbit review completed with no unaddressed blocking comments (process same as Phase 5c)

---

## Completion Checklist

- [ ] Layer 1 complete: tasks 70, 71 committed; merge point 1 green
- [ ] Layer 2 complete: tasks 72, 73 committed; merge point 1 green (workspace check)
- [ ] Layer 3 complete: tasks 74, 75, 78 committed; merge point 2 green
- [ ] Layer 4 complete: task 76 committed; merge point 3 green
- [ ] Layer 5 complete: task 77 committed + SUBSCRIPTIONS.md updated; merge point 4 green
- [ ] Phase 6 completion report at `.claude/PRPs/reports/phase-6-complete-report.md` (follow Phase 5c template)
- [ ] PR opened into `governance-v0`
- [ ] `project_brehon_phase_6_complete.md` memory written

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| R1: Env-var swap between two test pools corrupts pool B | LOW | MED | Sequence pool builds strictly; no `tokio::join!` on pool construction; task 77 inline comment documents the pattern |
| R2: `#[serde(untagged)]` variant ordering makes new AP types unreachable | MED | MED | Add new variants **before** `RawAnnouncableActivities`; add a JSON deserialisation assertion in task 77 setup |
| R3: `activitypub_federation` 0.7.0-beta.10 API drifts vs docs.rs snapshot | LOW | HIGH | Probe 5 in pre-phase audit catches version mismatch before Agent B/C spawn; if drift is real, raise a DQ entry before continuing |
| R4: Hash-chain signature UPDATE races with federation enqueue (`sent_activity` insert) | LOW | HIGH | Task 74 wraps enqueue + log in `conn.run_transaction()` per `feedback_multi_write_handlers_need_transactions.md` |
| R5: PR #10 not merged before phase-6 starts | HIGH (observed) | HIGH | Pre-flight gate (§above) hard-requires merge; impl agents refuse to spawn against missing phase-6 branch per `phase-branch.md` |
| R6: Two-container test timing exceeds CI budget | MED | LOW | Test runs <2min locally per audit sizing; acceptable for v0 (one test, one run); if CI times out, gate test behind `cfg(feature = "slow-tests")` |
| R7: Redaction misses an identifier in summary → GDPR leak via federation | LOW | HIGH | `redaction::scrub` already single-path per [06 §6](../../../docs/brehon-law-inspired-network/06-security-and-threat-model.md); task 74 reuses it. Task 77 asserts redacted summary on B. |
| R8: Parallel-agent index races at merge points 1 and 2 | MED | MED | Each agent gets its own worktree per `feedback_parallel_agents_one_worktree_per_agent.md`; advisor reviews diffs serially at merge point |
| R9: Upstream Lemmy 1.0-beta rebase during Phase 6 breaks AP trait signatures | LOW | HIGH | No rebase is scheduled during Phase 6 per CLAUDE.md weekly cadence (next rebase 2026-04-25 earliest); if rebase becomes necessary mid-phase, pause and re-run pre-phase audit |
| R10: Signature capture stub in task 75 stores empty string | LOW (DQ-6.2) | LOW for v0 | Document as DQ entry; v0 stores what it can; v1 plumbs real HTTP signature via activitypub_federation hook extension |

---

## Decision Queue Pre-Seeds

Entries pre-filled into `.claude/decision-queue.json` before Layer 1 spawns, so agents find answers waiting rather than needing to stop the loop.

### DQ-6.1 — Add `received_at` column to `remote_sanction_notice`?
- **from:** plan (Agent A, task 70)
- **question:** [04 §3 RemoteSanctionNotice](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) lists `published_at` but not `received_at`. Should task 70 add a `received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` column?
- **options:** ["add-received-at", "mirror-design-doc-exactly"]
- **context:** Admin-review queries ("show notices received in last 24h") need it; the field is load-bearing for a future admin dashboard. Adding it is a non-breaking extension. Not adding it means a timestamp is lost.
- **advisor-expected-answer:** "add-received-at" — small cost, obvious future need, pre-approved deviation from the design doc.

### DQ-6.2 — Where do we store the HTTP signature value on inbound notices?
- **from:** plan (Agent E, task 75)
- **question:** `activitypub_federation` consumes the HTTP Signature header before `Activity::receive` is called. How does task 75 populate `remote_sanction_notice.signature`?
- **options:** ["empty-string-with-v1-todo", "extract-via-hook", "store-activity-id"]
- **context:** The signature is verified (so we know the row is authentic), but we don't get the raw header. v0 doesn't need to re-verify; we just need something non-null per the NOT NULL constraint. Simplest: store `activity.object.id.to_string()` as a placeholder, keep NOT NULL but document in a comment that v0 uses the activity id as a proxy for signature traceability.
- **advisor-expected-answer:** "store-activity-id" — satisfies the NOT NULL constraint, preserves a unique identifier per notice, defers real signature storage to v1.

### DQ-6.3 — Idempotency of inbound activity receipt
- **from:** plan (Agent E, task 75)
- **question:** If the same `PublishSanctionNotice` activity is received twice (sender retry), should we dedupe on the inbox side or rely on `ReceivedActivity::create` in Lemmy's existing `Dummy::hook`?
- **options:** ["rely-on-received-activity-dedup", "add-unique-index", "do-both"]
- **context:** `Dummy::hook` at `crates/apub/apub/src/http/mod.rs:99-114` writes to `received_activity` table via `ReceivedActivity::create`, which dedupes by `activity.id()`. This should prevent `Activity::receive` from running twice for the same activity at the HTTP layer. A unique index on `remote_sanction_notice` would be belt-and-braces.
- **advisor-expected-answer:** "rely-on-received-activity-dedup" — Lemmy's existing mechanism is the right layer to dedupe at; adding a second safeguard is defensive coding we don't need in v0.

### DQ-6.4 — Env-var cleanup after `sanction_notice_round_trip`
- **from:** plan (Agent G, task 77)
- **question:** The test sets `LEMMY_DATABASE_URL` globally. Should it restore / remove the env var in a teardown block?
- **options:** ["leave-as-is-existing-pattern", "add-teardown", "move-to-once_cell-override"]
- **context:** Existing tests at e2e.rs:2195+ don't clean up, which is a latent bug (next test in the binary inherits the url). In practice tests run serially and each overwrites, so no observed failure. But a future test added before `sanction_notice_round_trip` in declaration order could inherit its url.
- **advisor-expected-answer:** "leave-as-is-existing-pattern" — mirror Phase 5c; document as a known limitation; fix comprehensively in a dedicated test-infra cleanup phase.

### DQ-6.5 — Plan-vs-spec naming drift for `FederationQuarantineRecommendation`
- **from:** plan (Agent A, task 70 + Agent F, task 76)
- **question:** Invocation prompt uses `FederationQuarantineRecommendation` scope; codebase uses `SanctionScope::FederatedRecommendation` + `SanctionAction::FederationQuarantineRecommendation`. Confirm task 76 branches on `scope == SanctionScope::FederatedRecommendation`.
- **options:** ["branch-on-scope-confirmed", "rename-in-plan", "flag-to-user"]
- **context:** Audit confirmed `SanctionScope::FederatedRecommendation` is the exact variant. The action name (`FederationQuarantineRecommendation`) is distinct and pairs with the scope. This is correct, not a drift — the plan docs just express the combined pair loosely.
- **advisor-expected-answer:** "branch-on-scope-confirmed" — plan text in task 76 uses `SanctionScope::FederatedRecommendation` throughout.

---

## Notes

### Sub-phase split — not taken

Q1 asked whether Phase 6 needs a 6a/6b split like Phase 5. Answer: **no**.

Phase 5 split because reputation math had cross-cutting dependencies (config reads, snapshot calculator, sponsor-liability, capability re-evaluation) that required three review cycles. Phase 6 is structurally simpler: schema → types → publisher/receiver → wiring → test. Each layer is independently reviewable. The layer-parallel execution model (max 2 agents concurrent) gives the same review throughput as a 6a/6b split without the overhead of two PRs, two CodeRabbit runs, two phase-transition ceremonies, and two context archives.

If Phase 6 encounters unforeseen scope creep (e.g. a DQ answer forces a new migration mid-phase, or a fork of the plan is required), fall back to a 6a (tasks 70–75 + 78) / 6b (tasks 76–77) split at merge point 2 (the natural boundary between infrastructure and wiring).

### Sub-agent spawn parameters (per `feedback_subagent_model_and_effort.md`)

Every Agent tool call for this phase MUST include:
- `model: "opus"` (forces Opus 4.7, not inherited Sonnet/Haiku)
- Prompt preamble: "Run at maximum effort — deepest reasoning, most thorough exploration. You are continuing a Brehon governance-fork Phase 6 layered-execution plan."

### Worktree isolation (per `feedback_preserve_active_worktree_state.md`)

Advisor creates each agent's worktree via:
```bash
git worktree add ../brehon-fork-agent-<X>-phase6 -b agent-<X>-phase6 phase-6
```
Agent X operates in `../brehon-fork-agent-X-phase6` exclusively. At task completion, agent pushes its branch. At merge point, advisor fast-forwards or merges the agent branch into phase-6 and removes the worktree via `git worktree remove ../brehon-fork-agent-X-phase6`.

**The primary worktree (`C:\Users\barri\Developer\brehon-fork`) stays on `phase-6` for coordination. It is NEVER checkout-switched during Phase 6** — per the memory, `-f` destroys pending work and non-`-f` aborts.

### Output capture (per `cargo-output-capture.md` + `no-cargo-output-paste.md`)

Every wrapper call redirects to `.claude/build-*.log` or `.claude/audit-*.log`. Tails into conversation limited to 20–30 lines. Full logs on disk for diagnosis.

### Phase completion report

After task 77 green: write `.claude/PRPs/reports/phase-6-complete-report.md` following the Phase 5c template. Sections:
1. What shipped (9 tasks)
2. What we learned (layered-execution retrospective: did 5-wave parallel beat single-ralph?)
3. Carry-forwards (any v1 items surfaced: peer allowlist table, trust-attestation wiring, HTTP signature capture)
4. DQ resolutions summary
5. Reviewer notes for PR #<N>

### V0 MVP closes with this phase

Phase 6 is the last v0 phase. After PR merge, the 11 MVP endpoints + the governance log + reputation + sponsor-liability + federation outbound are all in. The next scope is v1 per [IMPLEMENTATION-PLAN-v0.md §7 / 05 §7 / ADR-010](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) — different plan, different cadence, likely pauses for UX decisions (OQ-005).

---

## Confidence Score

**7.5/10** for one-pass implementation success.

**Rationale:**
- **+** Audit confirmed all integration points (submit_jury_vote ordering, governance_log signature, SanctionScope variant name, ap_id naming, AP trait shapes, test fixture reentrancy). Zero mismatches between plan assumptions and reality.
- **+** Parallel-agent layering is a natural fit: the 9 tasks decompose cleanly across 4 file-space-disjoint crate boundaries.
- **+** No new migrations beyond task 70; no upstream Lemmy churn on federation paths (ADR-012 is stable, ap_id already migrated).
- **+** Redaction, hash-chain, and pseudonymisation are existing helpers — no new cross-cutting wiring.
- **−** Two-database test (task 77) is the most complex test in v0; env-var swap pattern has a documented latent bug (DQ-6.4). First-pass risk: ~20% of test flakes on ephemeral-port timing; 5% risk of env-var leakage affecting a sibling test.
- **−** `activitypub_federation` 0.7.0-beta.10 API drift is the tail risk: the crate is pre-1.0, and the Brehon docs snapshot is from Phase 1. Probe 5 in the pre-phase audit catches this but doesn't pre-empt it.
- **−** Pre-flight gate (PR #10 merged + phase-6 branch + phase transition) is **not currently satisfied**. The plan assumes it will be before Layer 1 spawns. If the user skips any of the three prereqs, the plan cannot execute.
- **−** Worktree coordination across 5 agents over 5 merge points is more advisor overhead than Phase 5's single ralph. Higher attention cost per merge point.

Lowering the score below 8 is primarily because of the pre-flight state (PR #10 open, branch absent) — the plan itself is implementation-ready but cannot execute as-written until the three prereqs are met.
