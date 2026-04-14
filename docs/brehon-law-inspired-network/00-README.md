# Brehon-Law-Inspired Network — Documentation Suite

A decentralised, federated governance platform for grassroots organisations, forked from Lemmy, with a governance layer inspired by early Irish Brehon law: sponsorship-backed identity, reputation-at-risk, jury arbitration, graduated restorative sanctions, and transparent procedure.

This directory is the dev-team-facing documentation suite. Two raw chat transcripts (`chat1.md` and `chat2.md`) are the immutable source material; the numbered docs are the organised, audience-separated distillation used to plan and execute implementation.

---

## Documents

| # | File | Audience | Status |
|---|------|----------|--------|
| 00 | **[README](00-README.md)** | All | This file |
| 01 | **[Vision & Principles](01-vision-and-principles.md)** | PM, all | Stable |
| 02 | **[Domain Model](02-domain-model.md)** | Backend + PM | Stable |
| 03 | **[Architecture](03-architecture.md)** | Backend + ops | Stable |
| 04 | **[Data Model & API](04-data-model-and-api.md)** | Backend (daily driver) | Stable starting point |
| 05 | **[MVP & Delivery Plan](05-mvp-and-delivery-plan.md)** | PM + backend lead | Living |
| 06 | **[Security & Threat Model](06-security-and-threat-model.md)** | Security + backend | Stable principles, living threat table |
| 07 | **[Operations & Federation](07-operations-and-federation.md)** | Ops + on-call | Stable shape, living runbooks |
| 99 | **[Decisions & Open Questions](99-decisions-and-open-questions.md)** | All | Living |

### Raw source (immutable)

- [chat1.md](chat1.md) — engineer-facing Lemmy fork implementation checklist
- [chat2.md](chat2.md) — vision, Brehon-law mapping, architecture, security, blockchain scope, Surety+Reputation spec

Do not edit the raw chats. If a chat claim needs to change, update the relevant numbered doc and note the change in [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md).

---

## Reading paths by role

Pick the path that matches your role. Each path is the minimum needed to be useful on the project; the rest becomes background reading as you go.

### New backend engineer on day one
1. [00 — README](00-README.md) (this file)
2. [01 — Vision & Principles](01-vision-and-principles.md)
3. [02 — Domain Model](02-domain-model.md)
4. [04 — Data Model & API](04-data-model-and-api.md) — your daily-driver reference
5. [05 — MVP & Delivery Plan](05-mvp-and-delivery-plan.md) — specifically §5 "Monday morning checklist"

Then as needed: [03 — Architecture](03-architecture.md) for the big picture, [06 — Security & Threat Model](06-security-and-threat-model.md) for the authz and audit-log model, [99 — Decisions & Open Questions](99-decisions-and-open-questions.md) for the ADRs and open items you'll encounter.

### PM scoping work
1. [00 — README](00-README.md)
2. [01 — Vision & Principles](01-vision-and-principles.md)
3. [02 — Domain Model](02-domain-model.md) (for shared vocabulary)
4. [05 — MVP & Delivery Plan](05-mvp-and-delivery-plan.md)
5. [99 — Decisions & Open Questions](99-decisions-and-open-questions.md)

### Security reviewer
1. [00 — README](00-README.md)
2. [03 — Architecture](03-architecture.md) (especially §4 plane separation and §6 governance log)
3. [06 — Security & Threat Model](06-security-and-threat-model.md)
4. [04 — Data Model & API](04-data-model-and-api.md) (authz surface)

### Ops / on-call
1. [00 — README](00-README.md)
2. [03 — Architecture](03-architecture.md) (especially §10 deployment topology)
3. [07 — Operations & Federation](07-operations-and-federation.md)
4. [06 — Security & Threat Model](06-security-and-threat-model.md) (incident hooks)

---

## What this project is

A **federated platform for grassroots organisations** where moderation is not performed by a permanent moderator class but by procedural juries drawn from members. Users join via sponsorship (two existing members vouch for them and take reputation risk); bad actions ripple through trust relationships; sanctions are graduated and restorative rather than punitive; every decision is logged publicly with cited rules and redacted rationale.

The base platform is a fork of [Lemmy](https://github.com/LemmyNet/lemmy) (Rust, ActivityPub, Reddit-style communities). The governance layer is added as new Rust crates alongside the existing workspace.

One-line framing:

> **Trust is not given by the system — it is extended by other people, and risk is shared.**

## What this project is not

- **Not a token-based governance system.** No on-chain voting, no governance tokens, no "more tokens = more power". See [99 — ADR-002](99-decisions-and-open-questions.md).
- **Not a blockchain-of-everything.** Blockchain is used only as "public memory" — append-only hashes of case decisions for audit. Identity, reputation, voting, and application logic stay off-chain. See [99 — ADR-003](99-decisions-and-open-questions.md).
- **Not a replacement for Lemmy moderation in one pass.** Compatibility with Lemmy's existing moderator pathways is maintained during transition. See [99 — ADR-009](99-decisions-and-open-questions.md).
- **Not a single reputation score.** Reputation is four dimensions, event-sourced, exposed as capabilities. See [99 — ADR-005](99-decisions-and-open-questions.md).
- **Not permanent bans by default.** Reintegration paths exist for every sanction below the highest tier.

## Traceability chain

Every implementation detail traces back to a principle:

```
01 Vision principle
        │
        ▼
02 Domain concept (glossary, lifecycle, state machine)
        │
        ▼
04 Table / enum / struct / endpoint (implementation reference)
        │
        ▼
05 Sprint item (what ships in v0)
        │
        ▼
06/07 Security and operational controls
```

Decisions made along the way live in [99](99-decisions-and-open-questions.md) as ADRs.

## Document authoring rules

- **`chat1.md` and `chat2.md` are immutable.** Don't edit them. If a claim needs to change, update the numbered docs and log it in 99.
- **`02` holds nouns and narratives.** What a Case is, how a case flows, what a juror does.
- **`04` holds columns and signatures.** The exact `moderation_case` table, the exact `SubmitJuryVote` DTO.
- **Policy values** (thresholds, decay rates, jury sizes) live in `01` and `05`; they should not be hardcoded in `04` except as Rust constants that mirror the documented values.
- **`99` is append-only for ADRs.** New decisions → new ADR. Superseded decisions → `Supersedes: ADR-XXX` in the new entry; do not delete the old one.
- **Every doc cross-links the others.** If a reader lands cold in the middle, there's a navigable path out.

## Quick facts

- **Base platform:** Fork of Lemmy (Rust workspace, ActivityPub)
- **Primary datastore:** PostgreSQL; OpenSearch for search; S3/MinIO for evidence
- **Identity:** Keycloak with phishing-resistant MFA (WebAuthn / FIDO2 / passkeys)
- **Authorization:** OPA (Open Policy Agent) — policy-driven, not hardcoded
- **MVP jury parameters:** 5 jurors, quorum 3, simple majority
- **v1 jury parameters:** 7 jurors, severity-based thresholds (majority / 60% / 75%), diversity constraints
- **Reputation model:** 4 dimensions (reporting accuracy, jury reliability, participation consistency, endorsement strength), event-sourced, decaying, exposed as capabilities
- **Onboarding:** 2 sponsors (primary) OR time-based fallback
- **MVP endpoints:** 11 endpoints under `/api/v4/governance/*` — list in [05](05-mvp-and-delivery-plan.md) §2
- **Non-negotiable security controls:** phishing-resistant MFA on privileged actions, append-only signed governance log, immutable offline backups

## Where to start changing things

- **Editing a principle or the 9 Brehon mappings** → `01`
- **Adding a new domain concept or actor state** → `02`
- **Changing architecture or the Lemmy fork topology** → `03`
- **Adding a table, enum, or endpoint** → `04`
- **Adjusting what ships in v0** → `05`
- **New threat, new mitigation, new abuse pattern** → `06`
- **New deployment detail, new federation runbook, new background job** → `07`
- **New decision or new open question** → `99`

When in doubt, add to `99` and then migrate the content into its proper numbered doc once resolved.
