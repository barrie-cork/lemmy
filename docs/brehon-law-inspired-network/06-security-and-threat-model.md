# 06 — Security & Threat Model

**Audience:** Security engineer + backend
**Status:** Stable (principles); living (threat table)
**Sources:** [chat2.md](chat2.md) §Security architecture (10 points), §Anti-abuse mechanisms, §Federation trust model, §Transparency rules

This document holds the security architecture, the abuse taxonomy with mitigations, the federation trust model from a security angle, the transparency ↔ privacy boundary, and a threat-model table. Architecture layers and flows live in [03-architecture.md](03-architecture.md). Exact endpoints and DTOs live in [04-data-model-and-api.md](04-data-model-and-api.md).

---

## 1. Threat framing

A grassroots-organisation platform with governance power is a high-value target. Threats come from multiple directions at once:

- **External attackers** — DDoS, credential stuffing, SSRF through federated content fetch, supply-chain via dependencies
- **Bad-faith users** — sybil farms, brigading, sponsor farming, report abuse, jury capture
- **Compromised admins** — stolen session, coerced action, disgruntled operator
- **Hostile federated peers** — injected attestations, fake sanction notices, malicious remote content
- **Legal/state pressure** — takedown requests, subpoenas, forced-action requests targeting evidence or user identity

Our security model must handle all of these, and must fail **transparently** rather than silently. The append-only governance log (see [03-architecture.md](03-architecture.md) §6) is the backbone of this — if someone compromises the DB, the log should reveal it.

### Guiding principle

> Assume attackers will eventually get in somewhere. Make it hard for them to turn that foothold into **silent governance capture**.

## 2. The 10-point security architecture

Each point maps to one or more MVP deliverables. Don't defer all of these — MFA, the governance log, and backups are non-negotiable for v0.

### 2.1 Phishing-resistant MFA + step-up auth for privileged actions

- All privileged governance actions require phishing-resistant MFA (cryptographic authentication — WebAuthn / FIDO2 / passkeys — *not* TOTP codes typed into a session)
- Short session lifetimes, idle timeouts
- **Step-up reauthentication** required before:
  - Closing a case
  - Changing quorum or voting thresholds
  - Reversing a sanction on appeal
  - Remote-node quarantine
  - Federation trust state change
  - Rule-set (community constitution) edit
- Reference: NIST SP 800-63B phishing-resistance guidance; CISA MFA recommendations

### 2.2 Separate governance plane from content plane

The code path serving posts/comments/votes must **not** directly finalise sanctions, reputation changes, jury assignments, or federation trust updates. Enforced by:

- Separate route tree (`/api/v4/governance/*`)
- Separate handler modules (see [03-architecture.md](03-architecture.md) §4)
- Separate permission boundary — governance handlers do their own authz check against OPA, not just "is the user logged in"
- Narrower network exposure for governance endpoints where feasible (e.g. governance admin endpoints bound to internal network + VPN in production)

Reference: OWASP Authorization Cheat Sheet, Broken Access Control (A01).

#### 2.2.1 Emergency-remove override for illegal content

The jury system cannot be the only removal path for content that must come down immediately regardless of procedural fairness: CSAM, credible threats, doxxing of private individuals, legally-compelled takedowns. Some of these have sub-hour legal response requirements.

**Mechanism** (committed in [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md) ADR-013):

- Instance admins have an `emergency_remove` action that immediately removes the content
- A case is auto-opened in a new `EmergencyRemove` state with the removal as its initial fact
- A jury is assigned **post-facto** to review the admin's call
- The jury **cannot un-remove** the content (removal stands regardless) — they determine whether the override was used legitimately
- Persistent abuse of the override is itself a sanctionable offence (meta-case against the admin)
- Emergency-removes are surfaced in the public case log (the *fact* of removal; the content itself is not republished)
- Every use of `emergency_remove` emits a governance-log entry with extra-visible status and notifies all active admins

**Abuse vector this creates:** an admin could use `emergency_remove` to silence legitimate content by labelling it "illegal". This is why post-facto jury review is mandatory and why meta-cases against admins are part of the design. Covered in §7 threat table.

### 2.3 Append-only signed governance log

Already described in [03-architecture.md](03-architecture.md) §6. Security-relevant contract:

- **Append-only at the storage layer** (enforced via DB grants, not just code)
- **Cryptographically signed** — each entry hashed; hash chain signed by a key held outside the main application runtime (separate service account, separate machine, ideally HSM-backed for production)
- **Tamper-evident** — periodic consistency check compares the signed log against Postgres state and alerts on divergence
- **Publishable** — hash digest can be anchored to a public chain (see [07-operations-and-federation.md](07-operations-and-federation.md) §3)
- **Replayable** — the reputation and case state can be reconstructed from the log

Every write to: `moderation_case` state change, `jury_assignment`, `jury_vote`, `sanction`, `appeal`, `federation_attestation`, `reputation_event`, and any federation trust table must emit a log entry **before** the user response returns.

Reference: OWASP Logging Cheat Sheet, OWASP 2025 A09 (Security Logging & Alerting Failures); Sigstore Rekor as design inspiration.

### 2.4 Multi-party approval + delay for high-risk actions

High-risk governance actions require **quorum + delay** — no single compromised admin account can silently alter governance. Applied to:

- Banning a trusted member outside a jury decision
- Quarantining a remote node
- Changing jury-selection policy
- Modifying community constitutions / rule sets
- Reversing an appeal outcome
- Emergency `admin_close_case` (backstop)

**Mechanism:** action is queued; two independent admin sessions must co-sign; a mandatory delay (e.g. 15 minutes, configurable) passes during which the action can be cancelled by any admin; execution happens after delay + quorum.

Reference: OWASP Broken Access Control, CISA least-privilege guidance.

### 2.5 Harden the federation boundary

Federation inputs are **hostile**. Apply:

- HTTPS everywhere for API traffic
- Signature verification on every federated message (ActivityPub HTTP Signatures)
- Strict validation of remote objects (schema + size limits)
- Rate limiting per source instance on inbox processing
- **Isolate media fetching from the main app network path** — use a separate fetch worker with no access to the app DB (SSRF containment)
- **Remote sanction notices are advisory-only in MVP** — never auto-applied

Reference: OWASP REST Security Cheat Sheet, SSRF guidance.

### 2.6 Edge protection: DDoS and abuse

- Front with a CDN or reverse proxy that absorbs volumetric traffic
- Aggressive rate limiting on:
  - Login
  - Report submission
  - Jury voting
  - Endorsement creation
  - Federation inbox
- Edge-level bot mitigation (captcha or proof-of-work for unauthenticated high-value endpoints)
- Written incident-response playbook for DDoS scenarios (who to call, how to drop to static, how to communicate status)

Reference: CISA DDoS guidance.

### 2.7 Reduce the value of compromised credentials

Even with MFA, assume some accounts will be taken over.

- **Least privilege** — any one account (user or admin) has the narrowest capability set that works
- **Anomaly detection** for:
  - Impossible travel
  - Sudden trust-graph changes (a user suddenly endorsing 100 accounts)
  - Mass reporting
  - Mass endorsement creation
  - Policy edits outside normal patterns
  - Governance log write bursts
- **Capability attenuation on age** — new accounts can't immediately sponsor, report with weight, or be selected for juries (baked into [02-domain-model.md](02-domain-model.md) §2.1 actor states)

Reference: CISA event-logging best practices.

### 2.8 Secrets and signing-key management

- **No hardcoded secrets** in app config, CI, or repo
- Secrets manager (Vault, AWS Secrets Manager, or similar), with audit logging on every read
- Secrets rotation policy and tooling
- **Governance-signing keys held outside the main app runtime** — ideally a separate service account on a separate host, HSM-backed for production
- **Sign build artefacts** — CI produces signed container images; runtime verifies signatures before deploy (Cosign / Sigstore)
- **SBOM** for every release

Reference: OWASP Secrets Management Cheat Sheet, CISA SSDF/supply-chain guidance.

### 2.9 Recoverable compromise

- **Encrypted, offline, immutable backups** of Postgres, object store (evidence), and the governance log
- **Restoration drills** on a regular cadence — a backup you haven't tested restoring is not a backup
- **Key recovery** procedure documented and tested
- **Runbook for restoring from log** — the governance log should be sufficient to rebuild the case/reputation state even if Postgres is a total loss

Reference: CISA ransomware and backup guidance.

### 2.10 Abuse-case testing

Run threat modelling and abuse-case review specifically on:

- Brigading a single user with coordinated reports
- Juror capture (same sponsor cluster dominating a jury pool)
- Sponsor farming (creating lots of low-quality sponsees to inflate influence)
- Fake federated attestations (instance B claims user X is jury-eligible on instance A)
- Malicious appeals (using appeal flow to delay sanctions indefinitely)
- **"Admin account compromise during a contentious case"** — the single highest-risk scenario

Each abuse case must have at least one test (integration or red-team) exercised before v0 ship.

Reference: OWASP Threat Modeling Cheat Sheet.

## 3. Identity & authorization layer

### 3.1 Keycloak (identity provider)

- WebAuthn / FIDO2 / passkey support enabled
- Step-up auth flow for privileged actions
- Short session lifetimes; refresh tokens isolated from access tokens
- Separate realm for admin accounts

### 3.2 OPA (policy engine)

OPA owns all authorization decisions. Handlers *query* OPA; they do not hardcode policy. Policies cover:

- Report permissions (can this user report at all; is their report weighted?)
- Jury eligibility (does this user's snapshot satisfy `jury_eligible = true`)
- Sanction application rules (is this severity allowed for this jury's decision?)
- Federation policy (accept / limit / quarantine / block per remote instance)
- Rule-set edit permissions (who can propose, who must co-sign)

Benefit: a policy bug is fixable without a code deploy; policy diffs live in version control separately from app code; security review can audit policy without auditing Rust.

## 4. Abuse taxonomy and mitigations

A structured register of the bad-faith scenarios the Brehon mechanics exist to counter. Keep this table updated as new patterns are observed in production.

### 4.1 Sybil attacks (fake accounts)

**Threat:** attacker creates many accounts to manipulate reports, votes, endorsements, or jury pools.

**Mitigations:**
- Sponsorship requirement for full membership (two sponsors, each ≥ 30 days)
- Sponsor liability on sanction (sponsors lose reputation when sponsees misbehave) — makes sponsor-for-hire expensive
- Time gates on capabilities (no jury eligibility until 60 days active)
- Endorsement rate limits (max 5 active endorsements per sponsor, 48h cooldown)
- Anomaly detection on sudden graph changes
- Multi-dimensional reputation (hard to farm all four dimensions simultaneously)

### 4.2 Clique / cartel capture

**Threat:** a connected group dominates jury pools or report weight in a community.

**Mitigations:**
- Jury diversity constraints (v1 target) — no same-sponsor-cluster majority; geographic/community diversity where possible
- Reputation decay (prevents permanent elites)
- Cross-community exposure for federated cases
- Random weighted selection (not pure top-N)
- Appeals with new, larger jury excluding original jurors

### 4.3 Brigading

**Threat:** coordinated mass-report of a user to trigger case threshold.

**Mitigations:**
- Weight reports by reporter reputation (new accounts count for less)
- Ignore low-trust clusters in threshold calculation
- Time-based smoothing (rapid report spikes get dampened)
- Human review triggered on anomalous report velocity against a single target

### 4.4 Sponsor farming

**Threat:** an account sponsors many new accounts to inflate their `endorsement_strength` or proxy influence.

**Mitigations:**
- Max 5 active endorsements per sponsor
- 48h cooldown between endorsements
- Reputation penalties cascade — bad sponsees penalise sponsor's endorsement_strength
- Repeated bad sponsorship disables the capability entirely
- Anomaly detection on sponsor graph growth

### 4.5 Report abuse (weaponised reporting)

**Threat:** user uses the report system to harass or silence others.

**Mitigations:**
- Reporting accuracy dimension — bad reports reduce future report weight
- Threshold requires accumulated weight, not just count
- Public log of dismissed cases eventually deters habitual bad reporters
- Sanction for proven bad-faith reporting (handled by the same case system — meta-cases against abusive reporters)

### 4.6 Jury capture

**Threat:** attacker controls enough jurors in a single case to force a bad decision.

**Mitigations:**
- Random weighted selection from large eligible pool
- Diversity constraints (v1)
- Appeals overturn + reputation penalty for bad jurors
- Case severity → higher voting thresholds (v1 target)
- Anomaly detection on jury assignment → decision alignment (if every juror from one cluster consistently votes together, flag it)

### 4.7 Fake federated attestations

**Threat:** a malicious instance sends forged trust attestations or sanction notices.

**Mitigations:**
- Signature verification on every inbound message
- Trust-per-instance states (Allow / Limit / Quarantine / Block)
- Inbound sanctions are **advisory-only** — never auto-applied
- Federation inbound rate limiting
- Human admin review required before a remote notice affects local state

### 4.8 Appeal-abuse (infinite-loop)

**Threat:** sanctioned user files appeals repeatedly to delay enforcement.

**Mitigations:**
- **One guaranteed appeal per case** (hard limit)
- Subsequent appeals require admin discretion + quorum
- Appeal deadline is fixed; expired appeals auto-close

### 4.9 Admin session compromise during a contentious case

**Threat:** the highest-risk single scenario. An admin's session is stolen during a politically charged case and used to close the case, reverse a sanction, or edit the rule set.

**Mitigations:**
- Every sensitive action behind step-up auth (§2.1)
- Quorum + delay on high-risk actions (§2.4)
- Governance log is signed outside the app runtime (§2.3)
- Admin accounts use separate Keycloak realm
- Anomaly detection on admin activity bursts
- Incident response playbook rehearsed

### 4.10 Emergency-remove abuse

**Threat:** an admin uses `emergency_remove` (§2.2.1) to silence legitimate content by labelling it "illegal" when it isn't.

**Mitigations:**

- Post-facto jury review is mandatory — the jury determines whether the override was used legitimately
- Persistent misuse triggers a meta-case against the admin (their removal actions become the subject of a jury case)
- Every `emergency_remove` is logged to the append-only governance log with extra-visible status
- All active admins receive a notification whenever the override is used (peer awareness)
- Emergency-remove metrics (usage rate per admin) are on the governance-health dashboard; burst patterns trigger alerts
- Statistics on emergency-remove usage are published in the public case log (aggregate, not per-item)

## 5. Federation trust model (security angle)

Conceptually from [02-domain-model.md](02-domain-model.md) §8. Security invariants:

| Trust state | Inbound content | Inbound signals | Outbound |
|-------------|-----------------|-----------------|----------|
| **Allow** | Accepted, normal | Verified + stored + surfaced in UI | Normal |
| **Limit** | Accepted, downranked | Verified + stored, flagged for review | Normal |
| **Quarantine** | Accepted but hidden | Verified + stored as advisory only | Normal (may reconsider) |
| **Block** | Rejected at inbox | Rejected | No outbound to this peer |

**Invariants enforced regardless of trust state:**
- Signature verification always runs (rejection on failure)
- Inbound rate limits always apply
- SSRF protection on media fetch always applies
- No inbound message auto-applies a local sanction in MVP

## 6. Transparency ↔ privacy boundary

The transparency layer is a security feature: it deters silent corruption. But transparency must not expose private evidence or reporter identity.

### Public (for every case)
- Case ID
- Rule(s) cited
- Decision and sanction
- Rationale, **redacted** by the Redaction Service
- Appeal status
- Juror count (aggregate; identities not public)
- Reporter count (aggregate)

### Private (jury + authorised admins only)
- Full evidence files (unless explicitly flagged `PublicRedacted`)
- Reporter identities
- Juror identities before decision
- Private deliberation and rationale drafts

### Juror-visible (during case)
- Evidence with `JuryOnly` visibility
- Other jurors' identities (after accepting assignment)

Enforcement: `EvidenceVisibility` enum on `case_evidence`; separation between `ModerationCase` (holds private detail) and `PublicCaseLog` (holds publishable summary). The redaction service is a single code path — not distributed across handlers — so redaction logic can be audited.

### 6.1 Pseudonymous actor IDs in the governance log (GDPR)

Committed in [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md) ADR-015. The append-only governance log never stores direct personal identifiers (usernames, emails, display names). It stores an opaque **pseudonymous actor ID** per user. A separate `actor_pseudonym` table in Postgres maps `person_id ↔ pseudonymous_id`.

- On GDPR right-to-delete, the mapping row is deleted
- The hash chain is untouched; it remains verifiable
- The deleted user becomes permanently anonymous in the audit trail
- Case decisions and sanctions remain publicly logged (transparency of process); the actor behind them is unrecoverable

**Redaction service — extra responsibility:** before any rationale text is appended to the governance log or the public case log, the redaction service must scrub direct identifiers (usernames, emails, display names, addresses, phone numbers, external URLs that identify a person). This is a hard requirement — once identifiers are in the log, ADR-015's right-to-delete strategy fails.

## 7. Threat-model table

Keep this table updated. Every significant new feature adds rows. Every incident adds rows.

| Asset | Threat | Mitigation | Reference |
|-------|--------|------------|-----------|
| Session token (privileged) | Stolen / phished | Phishing-resistant MFA + step-up auth | §2.1 |
| Governance log | Tampered | Append-only storage, signed outside app runtime | §2.3 |
| Admin account | Compromised during contentious case | Quorum + delay on high-risk actions, anomaly detection | §2.4, §2.7, §4.9 |
| Federation inbox | Hostile remote payload | Signature verify, schema validate, rate limit, SSRF-isolated fetch | §2.5 |
| Media fetch path | SSRF via federated content | Separate fetch worker, no DB access | §2.5 |
| API endpoints | DDoS | CDN, rate limits, incident playbook | §2.6 |
| DB backups | Encrypted/destroyed by attacker | Offline immutable backups, restoration drills | §2.9 |
| Build artefacts | Supply-chain poisoning | Signed builds, SBOM, provenance verify | §2.8 |
| Jury pool | Captured by sponsor cluster | Diversity constraints (v1), reputation decay, appeals | §4.2, §4.6 |
| Report threshold | Brigading | Report weight by reputation, time smoothing | §4.3 |
| Sponsor graph | Sybil farm | Sponsor limits, liability cascade, graph anomaly detection | §4.1, §4.4 |
| Reporter identity | Leaked via modlog | Redaction service, `EvidenceVisibility` enum | §6 |
| Governance-signing key | Stolen | Held outside app runtime, HSM in production, separate realm | §2.3, §2.8 |
| Reputation snapshots | Mass-rewritten by compromised DB | Governance log is authoritative; divergence alarm | §2.3 |
| Appeals system | Infinite-loop abuse | Single guaranteed appeal; fixed deadline | §4.8 |
| Federation trust state | Silently flipped by attacker | Step-up auth + quorum + delay + log entry | §2.1, §2.4 |
| Illegal content (CSAM etc.) | Slow jury response blocks legal compliance | Admin emergency-remove with post-facto jury review | §2.2.1 |
| Emergency-remove override | Abused by admin to silence legitimate content | Post-facto jury review, meta-cases, peer notification, burst alerts | §4.10 |
| Personal data in governance log | GDPR right-to-delete conflicts with append-only log | Pseudonymised actor IDs + deletable mapping; redaction service scrubs identifiers from rationale text | §6.1 |

## 8. Priority order for implementation

If forced to pick only the top security items for v0:

1. **Phishing-resistant MFA + step-up auth** for privileged governance actions (§2.1)
2. **Strict authz and least privilege** on every governance endpoint and object (§2.2, §2.7)
3. **Append-only, signed governance log** for case decisions, sanctions, appeals, and federation trust changes (§2.3)
4. **Rate limiting and DDoS protection** at edge and on sensitive APIs (§2.6)
5. **Immutable offline backups** with restore drills (§2.9)
6. **Signed builds, dependency visibility, secret management** in CI/CD (§2.8)

Items 4, 5, 6 and 7–9 of the 10-point list are not optional — they are the minimum to avoid silent governance capture.

## 9. Cross-references

- Architecture layers → [03-architecture.md](03-architecture.md)
- Endpoints and DTOs this applies to → [04-data-model-and-api.md](04-data-model-and-api.md)
- What ships in v0 (so we can see what's protected and what isn't yet) → [05-mvp-and-delivery-plan.md](05-mvp-and-delivery-plan.md)
- Backup and key-management operations → [07-operations-and-federation.md](07-operations-and-federation.md)
- Security-relevant open questions → [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md)
