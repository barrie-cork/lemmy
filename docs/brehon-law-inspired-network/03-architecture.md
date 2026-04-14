# 03 — Architecture

**Audience:** Backend engineers + ops
**Status:** Stable
**Sources:** [chat2.md](chat2.md) §Technical architecture diagram, §Component architecture, §Mapping to Lemmy crate structure; [chat1.md](chat1.md) §0 first-pass crate plan

This document describes the **system shape** — what components exist, how they're layered, how the Lemmy fork is organised, and how the major flows run. Exact tables, enums, structs, and endpoints live in [04-data-model-and-api.md](04-data-model-and-api.md). Security controls live in [06-security-and-threat-model.md](06-security-and-threat-model.md).

---

## 1. Platform decision

The platform is a **fork of Lemmy**. Lemmy provides federation (ActivityPub), communities, posts/comments, voting, users, and the Rust workspace structure we'll extend. The governance layer is added as new crates and new modules *alongside* existing Lemmy code, not by rewriting Lemmy's core. Rationale: [99-decisions-and-open-questions.md#adr-001](99-decisions-and-open-questions.md).

## 2. High-level context diagram

```
                        ┌──────────────────────────────┐
                        │        Client Apps           │
                        │  (Web / Mobile / API SDK)    │
                        └─────────────┬────────────────┘
                                      │
                                      ▼
                        ┌──────────────────────────────┐
                        │        API Gateway           │
                        │ (REST / Auth / Rate limit)   │
                        └─────────────┬────────────────┘
                                      │
        ┌─────────────────────────────┼─────────────────────────────┐
        ▼                             ▼                             ▼

┌───────────────────┐     ┌──────────────────────┐     ┌──────────────────────┐
│   Core Platform   │     │   Governance Layer   │     │   Federation Layer   │
│   (Lemmy fork)    │     │  (New, this fork)    │     │    (ActivityPub)     │
└─────────┬─────────┘     └──────────┬───────────┘     └──────────┬───────────┘
          │                          │                            │
          ▼                          ▼                            ▼
┌───────────────────┐     ┌──────────────────────┐     ┌──────────────────────┐
│ Communities       │     │ Reputation Engine    │     │ Remote Instances     │
│ Posts/Comments    │     │ Jury System          │     │                      │
│ Voting            │     │ Case Management      │     │                      │
│ Users             │     │ Surety / Endorsement │     └──────────────────────┘
└─────────┬─────────┘     │ Public case logs     │
          │               └──────────┬───────────┘
          └──────────────┬───────────┘
                         ▼
              ┌──────────────────────┐
              │     Data Layer       │
              └──────────┬───────────┘
                         │
     ┌───────────────────┼────────────────────┐
     ▼                   ▼                    ▼
┌──────────────┐  ┌──────────────┐   ┌──────────────┐
│ PostgreSQL   │  │ OpenSearch   │   │ Object Store │
│ (core data)  │  │ (logs/search)│   │ (evidence)   │
└──────────────┘  └──────────────┘   └──────────────┘
```

## 3. The three planes

The system has three distinct planes that must not be tangled together. This separation is load-bearing for both modifiability and security.

### 3.1 Core Platform (inherited from Lemmy)

Handles:
- Community Service
- Post Service
- Comment Service
- Voting Service
- User Service

Comes effectively for free from Lemmy. We add hooks and, in some cases, gate severe actions behind governance decisions, but we do not rewrite the core.

### 3.2 Governance Layer (our innovation)

```
Governance Layer
├── Reputation Engine
│   ├── Event recorder
│   ├── Snapshot calculator
│   ├── Decay scheduler
│   └── Capability derivation (jury/report/sponsor eligibility)
│
├── Surety System
│   ├── Sponsorship graph
│   └── Endorsement tracking
│
├── Case Management System
│   ├── Report intake
│   ├── Case creation / threshold
│   ├── Evidence manager
│   └── Case lifecycle state machine
│
├── Jury System
│   ├── Jury pool builder
│   ├── Random weighted selector
│   ├── Conflict detection (no same-sponsor-cluster majority)
│   └── Voting aggregator
│
├── Moderation Engine
│   ├── Sanction executor
│   ├── Labeling system
│   └── Appeals handler
│
└── Transparency Layer
    ├── Public log generator
    ├── Redaction service
    └── Audit trail builder (feeds append-only governance log)
```

### 3.3 Federation Layer (extended from Lemmy's ActivityPub)

```
Federation Layer
├── Actor Service (unchanged from Lemmy)
├── Inbox/Outbox Handlers (extended)
├── Remote Object Fetcher
├── Signature Verification
├── Federation Policy Engine
│   ├── Allow / Limit / Quarantine / Block per remote instance
│   └── Trust Attestation Handler
└── Governance Signal Broadcaster
    ├── Publish moderation labels
    ├── Publish trust attestations
    └── Publish sanction notices
```

**Critical rule:** inbound federation governance signals are **stored as advisory evidence only** in MVP. They do not trigger local sanctions automatically. See [06-security-and-threat-model.md](06-security-and-threat-model.md) §5.

## 4. Governance plane vs content plane — a hard boundary

This is the most important architectural rule in this document:

> **The code path that serves posts/comments/votes must not directly finalise sanctions, reputation changes, jury assignments, or federation trust updates.**

Concretely:

- Governance endpoints live under a separate route tree (`/api/v4/governance/*`).
- Governance handlers live in their own modules (`crates/api/api/src/governance/*`, `crates/api/api_crud/src/governance/*`).
- Every governance write flows through the Transparency Layer's audit trail builder, which appends to a tamper-evident governance log (see §6 below).
- High-risk governance actions (closing a case, changing federation trust, editing a policy threshold) are gated behind step-up auth and/or quorum, enforced at handler level.
- OPA (Open Policy Agent) provides the policy-evaluation layer; authorization decisions are policy-driven, not hardcoded in handlers.

This boundary is what makes compromise containable. A stolen admin session on the content plane cannot, on its own, silently rewrite a case decision.

## 5. Data layer

### 5.1 PostgreSQL (authoritative)

Holds all core and governance state:

- Users, communities, posts, comments, votes (inherited from Lemmy)
- `moderation_case`, `case_evidence`, `sanction`, `appeal`, `public_case_log`
- `jury_pool`, `jury_assignment`, `jury_vote`
- `surety`, `endorsement`, `reputation_event`, `reputation_snapshot`
- `federation_attestation`, `remote_sanction_notice`

Full schema → [04-data-model-and-api.md](04-data-model-and-api.md) §2.

### 5.2 OpenSearch (derived / searchable)

- Moderation log search and facets
- Public case log search across communities
- Analytics dashboards (for instance admins)
- Full-text evidence search (restricted to authorised viewers)

OpenSearch is a **derived store**. Postgres is always authoritative; OpenSearch is rebuilt from Postgres on schema migration or corruption.

### 5.3 Object Storage (S3 / MinIO)

- Evidence files (images, documents, link archives)
- Audit bundles (periodic signed exports of the governance log)
- Public attachments

Evidence objects are referenced from `case_evidence` rows by `storage_key` + `sha256`, enabling integrity verification.

## 6. The append-only governance log (interface)

Separate from Lemmy's existing modlog. Every governance event — case open, case state change, jury assignment, jury vote, decision, sanction, appeal, federation trust change — is written to an **append-only, signed event stream** before the user-facing response is returned.

### 6.1 Contract

- **Append-only**: no updates, no deletes at the storage layer
- **Signed**: each entry is hashed and the hash chain is signed by a key held outside the main app runtime
- **Timestamped**: monotonic, instance-local, plus wall-clock
- **Publishable**: entry hashes can be periodically anchored to a public blockchain as "public memory" (see [07-operations-and-federation.md](07-operations-and-federation.md) §3)
- **Replayable**: the full reputation and case state can be reconstructed from the log

### 6.2 Why it's a separate layer

- **Tamper detection** — if Postgres rows disagree with the log, something is wrong
- **Audit independence** — a reader can verify the log without trusting the DB
- **Recovery** — if a DB is corrupted, the log is the rebuild source
- **Transparency** — hashed digests can be published, allowing outside observers to verify that no case was silently altered

This is the single most security-relevant architectural component in the system.

## 7. Lemmy fork topology

Lemmy's workspace already splits code by concern: schema, views, API, APub, routes, server. We extend that split rather than inventing a new structure.

### 7.1 New crates

```
crates/db_views/governance_case        ← case read models
crates/db_views/governance_modlog      ← public moderation log views
crates/db_views/reputation             ← reputation summary views
crates/db_views/jury_queue             ← juror dashboard views
```

### 7.2 New modules inside existing crates

```
crates/db_schema/src/source/governance_*.rs
    moderation_case.rs
    case_evidence.rs
    sanction.rs
    appeal.rs
    public_case_log.rs
    jury_pool.rs
    jury_assignment.rs
    jury_vote.rs
    surety.rs
    endorsement.rs
    reputation_event.rs
    reputation_snapshot.rs
    federation_attestation.rs
    remote_sanction_notice.rs

crates/api/api_common/src/governance.rs
crates/api/api/src/governance/
    get_case.rs
    list_cases.rs
    list_modlog.rs
    list_my_jury_queue.rs
    accept_jury_assignment.rs
    decline_jury_assignment.rs
    submit_jury_vote.rs
    get_my_reputation.rs
    admin_assign_jury.rs
    admin_close_case.rs
crates/api/api_crud/src/governance/
    create_report.rs
    create_endorsement.rs
    revoke_endorsement.rs
    request_appeal.rs
crates/api/routes/src/governance.rs

crates/apub/objects/src/governance/
    moderation_label.rs
    trust_attestation.rs
    sanction_notice.rs
crates/apub/activities/src/governance/
    publish_label.rs
    publish_attestation.rs
    publish_sanction_notice.rs
crates/apub/apub/src/governance/
    inbox.rs
    outbox.rs
    verify.rs

crates/server/src/governance.rs   ← wiring only
```

### 7.3 What belongs where (decision rules)

| Crate | Purpose | Example contents |
|-------|---------|------------------|
| `db_schema` | Diesel table definitions, Rust types, enums | `ModerationCase` struct, `CaseStatus` enum |
| `db_views/*` | Denormalised, permission-aware read models | `GovernanceCaseSummaryView`, `JuryQueueView` |
| `api_common` | Shared request/response DTOs used by both client and server | `CreateGovernanceReport`, `SubmitJuryVote` |
| `api_crud` | Simple create/read/update/delete handlers | `create_report`, `create_endorsement` |
| `api` | Workflow/orchestration handlers, non-CRUD | `submit_jury_vote`, `admin_close_case` |
| `routes` | HTTP route registration only | Route tree for `/api/v4/governance/*` |
| `apub/objects` | Serialisable ActivityPub objects | `ModerationLabelObject` |
| `apub/activities` | ActivityPub action types (`Create`, `Undo`) | `publish_label` |
| `apub/apub` | Inbox/outbox processing, signature verify | `receive_remote_sanction_notice` |
| `server` | Composition root only — wiring, bootstrapping, background jobs | Route registration, cron jobs |

**Rule:** `server` is the composition root, not a home for business logic. Domain logic lives in `api`, `api_crud`, and the governance sub-modules.

## 8. Key flows (end-to-end)

### 8.1 Moderation flow

```
User Report
    │
    ▼
[POST /api/v4/governance/report]
    │
    ▼
[api_crud/governance/create_report]
    │
    ▼
[Reputation weight applied] ◄── reads reputation_snapshot
    │
    ▼
[Threshold check]
    │
    ├── Below ──► report row persisted, 200 OK
    │
    └── Met ───► moderation_case row upserted
                      │
                      ▼
                [Temporary label applied]
                      │
                      ▼
                [Jury selection engine queued]
                      │
                      ▼
                [Jurors notified via inbox]
                      │
                      ▼
                (jurors review via /api/v4/governance/case)
                      │
                      ▼
                [POST /api/v4/governance/jury/vote]
                      │
                      ▼
                [Quorum check]
                      │
                      ├── Not yet ──► persist, 200 OK
                      │
                      └── Reached ──► [Decision aggregation]
                                         │
                                         ▼
                                   [Sanction executor]
                                         │
                                         ▼
                                   [Public log entry]
                                         │
                                         ▼
                                   [Reputation events emitted]
                                         │
                                         ▼
                                   [Appeal window opens]
```

### 8.2 Federation interaction flow

```
Local Node                                Remote Node
   │                                          │
   │  Local case decided                      │
   │                                          │
   │  [Create sanction notice activity]       │
   │                                          │
   ├── Publish via ActivityPub outbox ───────►│
   │                                          │
   │                                          ▼
   │                                 [Federation Policy Engine]
   │                                          │
   │                          ┌───────────────┼───────────────┐
   │                          │               │               │
   │                          ▼               ▼               ▼
   │                       Accept         Store as         Drop
   │                       as advisory    advisory         (remote blocked)
   │                       evidence       evidence,
   │                                      require manual
   │                                      review
```

**MVP constraint:** remote sanction notices are **never auto-applied**. They become advisory evidence in a local case (if one exists) or are stored standalone in `remote_sanction_notice` for operator review.

### 8.3 Trust & reputation flow

```
User action (post, report, jury vote, endorsement)
    │
    ▼
[Action handler]
    │
    ▼
[Reputation event emitted] ──► reputation_event row
    │
    ▼
[Snapshot recalculated asynchronously]
    │
    ▼
[Capability derivation]
    │
    ▼
reputation_snapshot:
    ├── reporting_accuracy
    ├── jury_reliability
    ├── participation_consistency
    ├── endorsement_strength
    ├── jury_eligible
    └── trusted_reporter
```

Snapshot recalculation runs in a background job, triggered by event insertion. See [04-data-model-and-api.md](04-data-model-and-api.md) §5 for exact fields.

## 9. Security architecture (high-level; full detail in 06)

```
           ┌──────────────────────┐
           │   Identity Layer     │
           │   (Keycloak)         │
           │ - MFA (phishing-     │
           │   resistant)         │
           │ - Step-up auth       │
           └──────────┬───────────┘
                      │
                      ▼
           ┌──────────────────────┐
           │ Policy Engine (OPA)  │
           │ - Report permissions │
           │ - Jury eligibility   │
           │ - Sanction rules     │
           │ - Federation policy  │
           └──────────┬───────────┘
                      │
                      ▼
           ┌──────────────────────┐
           │ Authorization Layer  │
           │ (enforcement in      │
           │  handlers)           │
           └──────────┬───────────┘
                      │
                      ▼
           ┌──────────────────────┐
           │   Application Logic  │
           │ (api / api_crud)     │
           └──────────┬───────────┘
                      │
                      ▼
           ┌──────────────────────┐
           │ Governance Audit Log │
           │ (append-only,        │
           │  signed)             │
           └──────────────────────┘
```

Full security model, abuse taxonomy, and threat table → [06-security-and-threat-model.md](06-security-and-threat-model.md).

## 10. Deployment topology

```
                ┌──────────────────────┐
                │   Load Balancer      │
                │  (rate limit, DDoS)  │
                └─────────┬────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ API Server 1 │  │ API Server 2 │  │ API Server N │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                 │
       └──────────┬──────┴──────────┬──────┘
                  ▼                 ▼
        ┌──────────────┐    ┌──────────────┐
        │ PostgreSQL   │    │ OpenSearch   │
        └──────────────┘    └──────────────┘
                  │
                  ▼
        ┌──────────────┐
        │ Object Store │
        └──────────────┘
```

Full deployment, backup, federation peering, and operations detail → [07-operations-and-federation.md](07-operations-and-federation.md).

## 11. What not to build here

Three explicit warnings, carried from the source chats:

1. **Do not put governance logic directly into `crates/server`.** Server is the composition root; it depends on everything else. Business logic there creates a tangled graph.
2. **Do not replace every existing Lemmy moderation action in one pass.** Keep Lemmy's report/modlog/moderator pathways as a compatibility layer during migration; gate severe actions behind governance decisions progressively.
3. **Do not model trust as a single score on `person` or `local_user`.** Reputation is events + derived views. A single field will be load-bearing in ways you won't be able to refactor later.

## 12. One architectural insight worth internalising

> Keep governance as a **separate layer**, not embedded inside Lemmy logic.

Easier to evolve, easier to test, reusable across platforms, and — most importantly — it keeps the governance plane defensible when the content plane gets compromised.
