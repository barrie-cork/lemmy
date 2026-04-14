# 05 — MVP & Delivery Plan

**Audience:** PM + backend lead
**Status:** **Living document** — expected to change as work lands
**Sources:** [chat1.md](chat1.md) §15 MVP API set, §16 implementation order, §17 aggregation rules, §18 shortcut, §19 warning, §20 "what to code first on Monday"

This document is the sprint-level execution plan. It holds the MVP vertical slice, sprint ordering, v0 simplifications, the Monday-morning checklist, and what's explicitly deferred. Exact tables and endpoints → [04-data-model-and-api.md](04-data-model-and-api.md).

---

## 1. MVP goal — the minimum useful vertical slice

> **Report → escalate → jury → decide → publish log → inspect trust**

A user can file a report; enough weighted reports open a case; jurors are assigned; jurors vote; a decision aggregates into a sanction; a public log entry is published; and any member can inspect their own reputation summary. No direct admin action required for the golden path (admin backstops exist but are not the hot path).

This is the smallest slice that demonstrates the core Brehon-inspired mechanic: **procedural moderation without a permanent moderator class**.

## 2. MVP API surface — exactly 11 endpoints

Implement these and nothing else in the first shippable slice.

### Reporting / cases
1. `POST /api/v4/governance/report` — create a report (may open a case)
2. `GET  /api/v4/governance/case` — read one case (permission-aware)
3. `GET  /api/v4/governance/cases` — list cases (filtered)

### Jury
4. `GET  /api/v4/governance/jury/me` — list my active jury assignments
5. `POST /api/v4/governance/jury/accept` — accept an assignment
6. `POST /api/v4/governance/jury/decline` — decline an assignment
7. `POST /api/v4/governance/jury/vote` — submit a vote

### Appeals
8. `POST /api/v4/governance/appeal` — request an appeal on a case

### Public transparency
9. `GET  /api/v4/governance/modlog` — public redacted moderation log

### Trust
10. `GET  /api/v4/governance/reputation/me` — my reputation summary
11. `POST /api/v4/governance/endorsement` — create an endorsement

That's the complete MVP. No endorsement revoke, no admin backstops from the client, no federation outbound — those land in the deferred list.

## 3. v0 simplifications (what we deliberately do *not* build in MVP)

Compare against [01-vision-and-principles.md](01-vision-and-principles.md) §5 (the v1 targets).

| Area | v1 target | v0 MVP |
|------|-----------|--------|
| Jury size | 7 | **5** |
| Quorum | Per-severity | **3 (fixed)** |
| Voting rule | majority / 60% / 75% by severity | **Simple majority for all cases** |
| Supermajority | Required for severe sanctions | **Deferred** |
| Jury diversity constraints | No same-sponsor-cluster majority, geographic diversity | **None — pure random-weighted** |
| Appeals jury | Larger than original | **Same size (5)** |
| Reputation decay | Per-dimension decay rates tuned | **Stub; run every 24h but no tuning** |
| Federation attestations | Full inbound + outbound | **Outbound-only; inbound stored but advisory** |
| Restorative actions | Distinct sanction types | **Modelled as flavoured `Label` / `TemporaryRestriction`** |
| Reputation scope | Per-community + instance roll-up | **Per-community only** |
| Public blockchain anchoring | Scheduled | **Hash chain stored locally; public anchoring deferred** |
| Report / case split | `report` → `case` promotion pipeline | **Every report creates or appends directly to a `moderation_case`** |

These are deliberate. They are documented so the team can tell "we haven't built X yet" apart from "we decided X isn't happening". Tracked as open questions in [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md) where appropriate.

## 4. Implementation order (6 steps)

Do these in order. Each step depends on the previous.

### Step 1 — Schema + Diesel foundation

Tables:
- `moderation_case`
- `jury_assignment`
- `jury_vote`
- `sanction`
- `public_case_log`

Plus Diesel models for each (see [04-data-model-and-api.md](04-data-model-and-api.md) §3).

**Done when:** migrations run clean forward and backward, models compile, integration tests can insert and query a `ModerationCase`.

### Step 2 — Read models

Views:
- `GovernanceCaseSummaryView`
- `JuryQueueView`
- `GovernanceModlogView`

**Done when:** each view has at least one query that returns data against a seeded test DB.

### Step 3 — API common DTOs

All the request/response types from [04-data-model-and-api.md](04-data-model-and-api.md) §5, in `crates/api/api_common/src/governance.rs`.

**Done when:** DTOs compile, are exported, and `ts-rs` generates client types if you're using frontend type export.

### Step 4 — First five endpoints (end-to-end slice)

1. `POST /api/v4/governance/report` — `create_report` (api_crud)
2. `GET  /api/v4/governance/case` — `get_case` (api)
3. `GET  /api/v4/governance/jury/me` — `list_my_jury_queue` (api)
4. `POST /api/v4/governance/jury/vote` — `submit_jury_vote` (api)
5. `GET  /api/v4/governance/modlog` — `list_modlog` (api)

**Done when:** you can integration-test the full flow: create two test users, user A reports user B's post, an admin uses the admin backstop to assign 5 jurors, jurors vote, a decision is recorded, a public log entry exists, and the modlog endpoint returns it.

**This is the true end-to-end MVP slice.** If this works, the governance layer works. Everything after is polish and breadth.

### Step 5 — Reputation & sponsorship

Add:
- `surety`
- `endorsement`
- `reputation_event`
- `reputation_snapshot`

Plus:
- `POST /api/v4/governance/endorsement`
- `GET  /api/v4/governance/reputation/me`

Wire in:
- Report weight uses `reporting_accuracy`
- Jury eligibility uses `jury_reliability`
- Sponsor liability flow on sanction

**Done when:** a reported-and-sanctioned user's sponsors visibly lose reputation; a user with zero jury reliability cannot be selected for jury duty.

### Step 6 — Federation objects & activities

Add the APub objects, activities, inbox/outbox handlers. Outbound-only to start.

**Done when:** a sanction on instance A publishes a `SanctionNotice` that instance B receives, verifies, stores in `remote_sanction_notice`, and surfaces in admin review (never auto-applies).

## 5. Monday morning checklist — the first coding sprint

Hand this directly to engineers on day one. Ordered.

1. **Migration** for `moderation_case`, `jury_assignment`, `jury_vote`, `sanction`, `public_case_log`
2. **Diesel models** for those five tables
3. **`db_views/governance_case`** and **`db_views/jury_queue`** crates with summary views
4. **`crates/api/api_common/src/governance.rs`** with the MVP DTOs
5. **Endpoints:**
   - `POST /api/v4/governance/report`
   - `GET  /api/v4/governance/case`
   - `GET  /api/v4/governance/jury/me`
   - `POST /api/v4/governance/jury/vote`
   - `GET  /api/v4/governance/modlog`

That gives a true end-to-end slice fast. Everything else in the MVP list (endorsement, reputation/me, appeal, jury accept/decline, list cases) is sprint 2.

## 6. Aggregation rules for `submit_jury_vote` in MVP

Exact logic for v0 — from [04-data-model-and-api.md](04-data-model-and-api.md) §8:

```
if submitted_votes < 3:
    persist vote
    return 200 OK

if submitted_votes >= 3:
    tally decisions
    winning decision = simple majority
    create one Sanction row
    mark case Decided
    insert PublicCaseLog entry
    emit ReputationEvent rows for jurors and reporters
    record case.closed_at deadline (appeal window)
```

- **5 jurors**, **quorum 3**, **majority wins**, **no supermajority**.
- Severe sanctions behind supermajority are **deferred** — a v1 item.

## 7. Post-MVP roadmap — v1 / v2 / v3 (staged releases)

Post-MVP work is split into three staged releases. Each is independently shippable and each has a single clear theme. Committed in [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md) as ADR-010.

### 7.1 v1 — "Is it production-grade governance?"

Finishes the governance mechanic. Brings jury parameters and reputation model up to the full spec, and ships self-host packaging so other communities can stand up an instance without reverse-engineering the source.

#### Governance mechanics

- Jury size increase to 7
- Severity-based voting thresholds (simple majority / 60% / 75%)
- Jury diversity constraints in selection (no same-sponsor-cluster majority; community diversity where possible)
- Appeals with larger, different jury (original jurors excluded)
- Reputation decay tuning per dimension
- Restorative-action sanction variants (distinct enum variants, not flavoured labels)
- Per-community rule-set versioning
- Instance-wide reputation roll-up
- `POST /api/v4/governance/endorsement/revoke`

#### Self-host packaging

- `deploy/` directory at repo root: `docker-compose.yml`, `.env.example`, `Caddyfile`
- Seed migration with default starter rule set and default jury parameters
- `scripts/bootstrap.sh` — migrations, first admin, federation keys
- `scripts/backup.sh` — `pg_dump` + object-store snapshot
- One-page `INSTALL.md`

### 7.2 v2 — "Can it survive hostile attention?"

Security hardening as its own focused milestone. Nothing in v2 adds new governance features — it makes the governance plane defensible against the scenarios in [06-security-and-threat-model.md](06-security-and-threat-model.md).

- Keycloak + phishing-resistant MFA (WebAuthn / FIDO2 / passkeys)
- Step-up authentication on privileged governance actions
- OPA or OpenFGA for policy-driven authorisation (replaces hardcoded capability checks)
- Append-only log signer running **outside** the app runtime (separate host; HSM-backed in prod)
- Quorum + delay on high-risk admin actions
- Separate SSRF-isolated federation fetch worker
- Vault / secrets manager with rotation
- Offline immutable backups with tested restore drills (regular cadence)
- Full abuse-case red-team pass (sybil, clique, brigading, sponsor farming, fake attestations, admin compromise)
- Signed build artefacts (Cosign / Sigstore) + SBOM per release

### 7.3 v3 — "Is it publicly verifiable and polished?"

External verifiability, full federation, and polished operator/user experience. The last milestone because most items have high ops cost for value gained and should come after the system has earned the scrutiny.

#### Verifiability

- Blockchain anchoring of the append-only governance log — Sigstore Rekor first (free, open), BTC `OP_RETURN` or Ethereum L2 if wanted later
- Per-event anchoring for Critical-severity cases
- Batched Merkle-root anchoring for normal cases (hourly / daily digests)
- External inclusion-proof verification tooling so observers can independently verify decisions

#### Federation

- Full federation inbound processing (beyond stored-advisory)
- Multi-instance federation runbook UI

#### UX

- Admin dashboard UX
- Onboarding + sponsorship UX flows (resolves OQ-005)
- Juror notification UX (in-platform inbox, email, optional push)

### 7.4 Note on "v1 target" references

Older references in [01-vision-and-principles.md §5.6](01-vision-and-principles.md), [02-domain-model.md §3.3](02-domain-model.md), [04-data-model-and-api.md §8](04-data-model-and-api.md), and [06-security-and-threat-model.md §4](06-security-and-threat-model.md) use "v1 target" to mean "the proper governance spec, not MVP". Those references remain correct — the full jury parameters and reputation model land in **v1** specifically. Security and anchoring items that those sections discuss (signed log outside runtime, blockchain) are **v2 and v3** respectively under this staging.

## 8. Warnings — things we will be tempted to do and must not

From [chat1.md](chat1.md) §18–§19 and the team's own experience. Put these in PR review checklists.

### Do not overbuild reputation in v1

Only the four counters and two booleans from [04-data-model-and-api.md](04-data-model-and-api.md) §13. Everything else is derived later. It is tempting to build elaborate multi-factor scoring before you have any real data. Resist this.

### Do not embed governance logic in `crates/server`

`server` is composition root only. Business logic in `api`, `api_crud`, and the governance sub-modules. See [03-architecture.md](03-architecture.md) §11.

### Do not replace all Lemmy moderation pathways in one pass

Keep Lemmy's existing report/modlog/moderator flow as a compatibility layer during the transition. Gate severe actions behind jury decisions progressively. Trying to rip it out early means fighting Lemmy's existing data shape and UI at the same time as building a new governance layer.

### Do not model trust as a single score on `person` or `local_user`

Reputation is events + derived views. A single "trust score" field is load-bearing in ways that become impossible to refactor after the first month.

### Do not auto-apply remote sanctions in MVP

Remote sanction notices are **advisory evidence only**. A federated instance getting compromised must not be able to cascade its compromise through our moderation. See [06-security-and-threat-model.md](06-security-and-threat-model.md) §5.

## 9. Done-definition for v0 ship

The v0 is considered shipped when all of the following are true:

- [ ] All 11 MVP endpoints return 200 on the happy path
- [ ] An integration test exists for the full flow: report → threshold → case → jury assignment (via admin backstop) → vote → decision → sanction → public log → modlog read
- [ ] A second integration test exists for the sponsor-liability flow: user signs up with 2 sponsors, commits a sanctionable offence, both sponsors lose reputation on `endorsement_strength`
- [ ] The 5 success criteria from [01-vision-and-principles.md](01-vision-and-principles.md) §7 are demonstrably met
- [ ] The append-only governance log records every case state change and is verifiable by hash chain (local only; public anchoring deferred)
- [ ] Deployment runbook in [07-operations-and-federation.md](07-operations-and-federation.md) is executable by someone other than the author
- [ ] Threat-model table in [06-security-and-threat-model.md](06-security-and-threat-model.md) has been reviewed and MVP-relevant mitigations are in place

## 10. Explicit v0 non-goals

Not just deferred — actively out of scope. These are decisions, not backlog items.

- No token-based governance, ever
- No on-chain voting, ever
- No single reputation score
- No permanent moderator class (admin backstops are for emergencies, not regular operation)
- No auto-applied federation sanctions

## 11. Cross-references

- What to build → this doc
- How each thing is built → [04-data-model-and-api.md](04-data-model-and-api.md)
- Why we're building it → [01-vision-and-principles.md](01-vision-and-principles.md)
- How it fits together → [03-architecture.md](03-architecture.md)
- Security review for each endpoint → [06-security-and-threat-model.md](06-security-and-threat-model.md)
- Deferred-question register → [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md)
