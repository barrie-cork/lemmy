# 99 — Decisions & Open Questions

**Audience:** All (living document)
**Status:** **Living** — open on day one, accretes throughout the project
**Sources:** distilled from [chat1.md](chat1.md) and [chat2.md](chat2.md), plus team conversations

This document is a lightweight ADR log, an open-questions register, and a spec changelog. Three sections: **Decisions**, **Open Questions**, **Changelog**.

Every entry is dated. ADRs are append-only — if we change our mind, we add a new ADR with `Supersedes: ADR-XXX`, we do not edit the original.

---

## Decisions (ADRs)

Lightweight Architecture Decision Records. Each has: ID, title, date, status, context, decision, consequences, alternatives considered (briefly).

### ADR-001 — Fork Lemmy as the base platform

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** We need federation, communities, posts/comments, voting, and a user system. Building from scratch is measured in years. Several mature platforms solve 60–80% of the problem.
- **Decision:** Fork [Lemmy](https://github.com/LemmyNet/lemmy). Add the governance layer as new crates alongside existing workspace members, not as a rewrite.
- **Consequences:**
  - ActivityPub federation comes for free
  - Rust workspace gives us the crate structure we want
  - Governance layer is a clean addition, not a surgery
  - We inherit Lemmy's upgrade path and can stay in sync with upstream where practical
- **Alternatives considered:** Mastodon (Ruby monolith, harder to reshape; not structured for governance), Matrix/Element (infrastructure rather than product, too much to build on top), Discourse (not federated by default, centralised architecture). Rationale detail remains in [chat2.md](chat2.md) §1.
- **Enacted in:** [03-architecture.md](03-architecture.md) §1, §7

### ADR-002 — No token-based governance

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** Token-weighted governance ("more tokens = more power") is incompatible with the Brehon principle that influence is earned slowly through relationships, not bought. Research on DAO governance shows small groups consistently dominate voting power.
- **Decision:** The platform will never use governance tokens, on-chain voting, or token-weighted decision mechanisms. Influence derives from reputation, which derives from behaviour in relationships.
- **Consequences:**
  - No DAO-style on-chain voting
  - No token issuance
  - Governance weight comes from `reputation_snapshot` dimensions
  - Simpler user experience: users vote in juries, they don't manage token balances
- **Alternatives considered:** Quadratic voting, conviction voting, futarchy — all rejected because all require stake/token mechanics that re-introduce the exact elite-capture problems we're designing against.
- **Enacted in:** [01-vision-and-principles.md](01-vision-and-principles.md) §6, [02-domain-model.md](02-domain-model.md) §4

### ADR-003 — Blockchain is used only as "public memory"

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** Blockchain solves *integrity of record* (tamper-evident storage) but does not solve identity, sybil resistance, social manipulation, or governance fairness. It also adds cost, complexity, and UX friction.
- **Decision:** Blockchain is used only to anchor hashes of the append-only governance log, case decisions, rule changes, and federation sanctions. It is never used for identity, reputation, voting, or application logic. Chain interaction is one-way (outbound); the system never reads from the chain to make decisions.
- **Consequences:**
  - Case decisions are verifiable by external observers
  - If the chain provider disappears, anchoring stops but the system keeps running
  - Costs and latency are bounded (batched anchoring)
  - No gas fees in the user path
- **Alternatives considered:** Full on-chain voting (slow, expensive, poor UX, privacy-hostile), smart-contract-based rules (brittle, opaque, harder to evolve than DB-backed rules + OPA). Both rejected.
- **Enacted in:** [07-operations-and-federation.md](07-operations-and-federation.md) §3, [01-vision-and-principles.md](01-vision-and-principles.md) §6

### ADR-004 — Governance plane separated from content plane

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** A compromised admin on the content plane must not be able to silently finalise governance changes. Security reviewers (OWASP, CISA) consistently recommend separating privileged and general-purpose code paths.
- **Decision:** Governance code lives in its own route tree (`/api/v4/governance/*`), its own handler modules (`crates/api/api/src/governance/*`), and its own permission boundary. Every governance write emits an append-only signed log entry before responding. High-risk governance actions require step-up auth, and some require quorum + delay.
- **Consequences:**
  - Cleaner testability and auditability
  - Compromise-containment is realistic
  - Admin action on the governance plane is visible, verifiable, and revertible
- **Alternatives considered:** Single unified API with per-endpoint authz (rejected — hard to audit, easy to regress into).
- **Enacted in:** [03-architecture.md](03-architecture.md) §4, [06-security-and-threat-model.md](06-security-and-threat-model.md) §2.2

### ADR-005 — Reputation is multi-dimensional and event-sourced, never a single score

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** Single-score reputation invites gaming, leaderboards, and permanent elite/outcast classes. It's also almost impossible to refactor after the first month of production data.
- **Decision:** Reputation is four dimensions (reporting accuracy, jury reliability, participation consistency, endorsement strength), stored as append-only events, with snapshots derived by a background job. Users see capabilities (`jury_eligible`, `trusted_reporter`) and visible labels, not numbers.
- **Consequences:**
  - Decay, tuning, and policy changes happen at the snapshot level without touching the event log
  - Audit trail of every reputation change
  - No leaderboard
- **Alternatives considered:** Single trust score on `person` (rejected — load-bearing field nightmare), per-community karma (rejected — too coarse).
- **Enacted in:** [02-domain-model.md](02-domain-model.md) §4, [04-data-model-and-api.md](04-data-model-and-api.md) §3 (ReputationEvent, ReputationSnapshot)

### ADR-006 — Remote sanction notices are advisory only in MVP

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** A federated instance getting compromised must not cascade its compromise through our moderation. Auto-applying remote sanctions would make us vulnerable to upstream compromise and to bad-faith peers.
- **Decision:** Inbound federation governance signals (sanction notices, trust attestations) are stored as advisory evidence. They are never auto-applied. A local jury may cite them; an admin may review them; neither is automatic.
- **Consequences:**
  - Slower federation response to cross-instance bad actors
  - Human-in-the-loop for every cross-instance sanction
  - Contained blast radius of peer compromise
- **Alternatives considered:** Auto-apply from trusted peers (rejected — "trusted" is exactly what changes during compromise), delayed auto-apply with manual override (rejected — creates exactly the silent governance capture path we're protecting against).
- **Enacted in:** [03-architecture.md](03-architecture.md) §8.2, [04-data-model-and-api.md](04-data-model-and-api.md) §11, [06-security-and-threat-model.md](06-security-and-threat-model.md) §5

### ADR-007 — MVP uses simplified jury parameters (5 jurors, quorum 3, simple majority)

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** The v1 target is 7 jurors with severity-based voting thresholds and diversity constraints. That's too much to implement correctly in the first sprint, and some of the parameters (diversity constraints in particular) need real data to tune.
- **Decision:** MVP implements 5-juror panels, fixed quorum of 3, simple majority wins, no diversity constraints, no severity-based thresholds. Document every simplification explicitly so "not yet" and "not happening" are distinguishable.
- **Consequences:**
  - Faster MVP
  - Jury parameters become a v1 tuning exercise rather than a design exercise
  - Risk: v0 is less resistant to jury capture than v1 will be
- **Alternatives considered:** Full v1 from day one (rejected — delays meaningful shipping), smaller MVP (e.g. 3 jurors — rejected as too small to demonstrate the mechanic).
- **Enacted in:** [04-data-model-and-api.md](04-data-model-and-api.md) §8, [05-mvp-and-delivery-plan.md](05-mvp-and-delivery-plan.md) §3

### ADR-008 — Append-only signed governance log is architecturally mandatory

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** The single most important security property of this platform is tamper-evidence: a compromise of the main DB should not be able to silently rewrite case decisions. The governance log is the mechanism.
- **Decision:** Every write to case state, jury state, sanctions, appeals, and federation trust must emit an append-only, signed log entry before the user response returns. The signing key is held outside the main app runtime. The log is the authoritative recovery source if Postgres is lost.
- **Consequences:**
  - Extra write-path latency (small)
  - Operational dependency on the signer service
  - Tamper detection is possible
  - Recovery from catastrophic DB loss is possible
- **Alternatives considered:** Trust Postgres + regular backups (rejected — backups lag compromise; no tamper detection), write-behind logging (rejected — race window allows silent writes to slip through).
- **Enacted in:** [03-architecture.md](03-architecture.md) §6, [06-security-and-threat-model.md](06-security-and-threat-model.md) §2.3, [07-operations-and-federation.md](07-operations-and-federation.md) §4

### ADR-010 — Staged releases: v0 → v1 → v2 → v3

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** The original design in [chat1.md](chat1.md) / [chat2.md](chat2.md) describes a target system with full Brehon jury parameters, a 10-point security architecture (Keycloak/MFA, OPA/OpenFGA, governance-log signer outside app runtime, Vault, offline immutable backups, signed builds, red-team pass), blockchain anchoring, and polished UX. That is 6–12 months of work beyond MVP for a solo developer, and most of the security work is its own full project. Shipping "v1 = everything from the chats" in one release is not realistic and delays every intermediate value delivery.
- **Decision:** Split post-MVP scope into three staged releases, each with a clear theme. Each release must be independently shippable and useful.

  **v0 (MVP)** — *"Does the mechanic work?"*
  - 11-endpoint governance slice
  - 5-juror panels, quorum 3, simple majority
  - Local hash chain (no external anchoring)
  - JWT + `webauthn-rs` for optional passkey MFA
  - Docker Compose on one host
  - Env-var secrets
  - Solo-dev substitutions from the tech-stack assessment

  **v1** — *"Is it production-grade governance?"*
  - Full jury parameters: 7 jurors; severity-based thresholds (majority / 60% / 75%); diversity constraints
  - Appeals with larger, different jury
  - Reputation decay tuning per dimension
  - Restorative-action sanction variants
  - Rule-set versioning per community
  - Instance-wide reputation roll-up
  - `POST /endorsement/revoke`
  - `deploy/` directory, bootstrap script, `INSTALL.md` for self-host ease

  **v2** — *"Can it survive hostile attention?"*
  - Keycloak + phishing-resistant MFA (WebAuthn)
  - Step-up auth for privileged actions
  - OPA or OpenFGA for policy-driven authz
  - Append-only log signer running **outside** the app runtime (separate host, HSM in prod)
  - Quorum + delay for high-risk admin actions
  - Separate SSRF-isolated federation fetch worker
  - Vault / secrets manager with rotation
  - Offline immutable backups with tested restore drills
  - Full abuse-case red-team pass
  - Signed build artefacts (Cosign / Sigstore) + SBOM

  **v3** — *"Is it publicly verifiable and polished?"*
  - Blockchain anchoring (Sigstore Rekor first, then BTC `OP_RETURN` or Ethereum L2 if wanted)
  - Per-event anchoring for Critical cases
  - External inclusion-proof verification tooling
  - Full federation inbound processing (beyond stored-advisory)
  - Admin dashboard UX
  - Onboarding + sponsorship UX flows
  - Juror notification UX
  - Multi-instance federation runbook UI

- **Consequences:**
  - Every release is shippable and delivers visible value
  - Security hardening becomes its own focused milestone (v2) rather than competing with feature work
  - Blockchain anchoring is explicitly the last thing added — it's the smallest-value, highest-ops-cost item and goes after the system has earned the scrutiny
  - Self-host packaging lands in v1 (not MVP), keeping v0 focused on proving the governance mechanic works
  - "v1 target" references throughout the docs remain correct — they still mean "not MVP" — but the broader v1/v2/v3 split is authoritative
- **Alternatives considered:**
  - "v1 = everything from chats" (rejected — unrealistic for solo dev, delays all intermediate value)
  - "Keep v1 loose, decide what's in at time of cutting" (rejected — makes scope conversations circular and encourages scope creep)
  - More than 3 post-MVP releases (rejected — more milestones = more overhead; three themes cover the major concerns cleanly)
- **Enacted in:** [05-mvp-and-delivery-plan.md](05-mvp-and-delivery-plan.md) §7, [07-operations-and-federation.md](07-operations-and-federation.md) §9

### ADR-009 — Compatibility layer with existing Lemmy moderation

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** Ripping out Lemmy's report / modlog / moderator pathways in one pass would mean fighting the existing data shape and UI while also building a new governance layer. Very high risk.
- **Decision:** Keep Lemmy's existing moderation pathways as a compatibility layer during v0 and early v1. Gate severe actions behind jury decisions progressively. Eventually deprecate direct moderator action for the actions governance covers.
- **Consequences:**
  - Two moderation paths coexist during transition
  - Risk of inconsistency if a moderator acts directly on something a jury is deciding — needs a rule: direct moderator action puts a case in an admin-review state
  - Smoother migration for any community that adopts this fork mid-life
- **Alternatives considered:** Hard cutover (rejected as stated above).
- **Enacted in:** [05-mvp-and-delivery-plan.md](05-mvp-and-delivery-plan.md) §8, [03-architecture.md](03-architecture.md) §11

### ADR-011 — Licence: AGPLv3 inherited from Lemmy

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** Lemmy is licensed under AGPLv3. Any fork inherits this licence. The AGPL's network clause means that if the platform is run as a network service (which a federated instance is by definition), operators must make source code available to users of the service, including modifications.
- **Decision:** The fork stays under AGPLv3. Do not attempt to relicence. All governance-layer code contributed to the fork is AGPLv3. Instance operators running this fork are on notice that they must make source available to their users on request.
- **Consequences:**
  - Cannot close-source the project or any community's deployment
  - Must publish modifications (this aligns with the platform's transparency principle)
  - Compatible with a Sigstore / transparency-log tooling ecosystem (mostly Apache-2.0) — we can depend on them but they cannot depend on us
  - Aligns ideologically with the grassroots-organisation target audience
- **Alternatives considered:** Relicencing is not actually available (Lemmy's copyright is held by many contributors and cannot be unilaterally relicenced). Writing from scratch to avoid AGPL was rejected long ago in ADR-001.
- **Enacted in:** [00-README.md](00-README.md) (licence footer to add), repo `LICENSE` file to include AGPLv3 text

### ADR-012 — Base Lemmy version: track 1.0-beta

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** Lemmy 0.19.x is the stable, battle-tested line with many production instances. Lemmy 1.0-beta introduces multi-communities, a refactored API split into view crates, `actor_id` → `ap_id` column rename, and — critically — an Extism-based WebAssembly plugin system with `before_*` and `after_*` hooks for activities. The plugin system is directly useful for governance integration points (`before_governance_report`, `after_jury_verdict`).
- **Decision:** Fork from Lemmy 1.0-beta. Accept the rebase burden in exchange for the plugin system and the refactored API shape (which makes adding governance view crates cleaner).
- **Consequences:**
  - Rebase burden tracking upstream while 1.0 stabilises
  - Governance plugin hooks available from day one — cleaner integration than patching core code
  - `ap_id` naming aligns new governance AP objects with upstream conventions
  - Risk: 1.0 API may still change; plan for small breakages during v0
- **Alternatives considered:** 0.19.x stable (rejected — we'd miss the plugin system and end up patching it in ourselves, defeating the point), tracking main directly (rejected — too much churn).
- **Enacted in:** [03-architecture.md](03-architecture.md) §7 (crate topology assumes 1.0 layout), Cargo.toml once forking begins

### ADR-013 — Illegal-content emergency-remove path

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** The jury system cannot be the only removal path for content that must come down immediately regardless of procedural fairness: CSAM, credible threats, doxxing of private individuals, legally-compelled takedowns. Some of these have sub-hour legal response requirements. A pure-jury model is legally and ethically unworkable for these cases.
- **Decision:** Instance admins have an emergency-remove action that immediately removes the content and creates a case in a new `EmergencyRemove` state. A jury is assigned post-facto to review the admin's call. The jury cannot un-remove the content (the removal stands regardless), but the jury's review determines whether the admin's use of the override was legitimate. Persistent abuse of the emergency-remove override is itself a sanctionable action by meta-case.
- **Consequences:**
  - Legal compliance is possible
  - Admin power exists but is auditable and constrained by post-facto review
  - Emergency-remove is visible in the public case log (the fact of removal, not necessarily the content)
  - A new enum variant or case status is needed — needs a row in [04-data-model-and-api.md](04-data-model-and-api.md) §2 and an entry in [02-domain-model.md](02-domain-model.md) §3.1
  - Needs a threat-table row in [06-security-and-threat-model.md](06-security-and-threat-model.md) §7 for "admin abuses emergency-remove"
- **Alternatives considered:** Admin co-sign quorum + delay (rejected — sub-hour CSAM response impossible at 3am with only one admin awake), mandatory fast-track jury (rejected — can't guarantee jurors available within legal response window), no override at all (rejected — legally and ethically unworkable).
- **Enacted in:** [02-domain-model.md](02-domain-model.md) §3.1, [04-data-model-and-api.md](04-data-model-and-api.md) §2 (new `CaseStatus` variant), [06-security-and-threat-model.md](06-security-and-threat-model.md) §2.2 and §7

### ADR-014 — Federation interop with vanilla Lemmy

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** Vanilla Lemmy instances (lemmy.world, etc.) don't know about our governance AP object types. They will ignore `ModerationLabelObject`, `TrustAttestationObject`, `SanctionNoticeObject`. The question is whether we permit content-level federation (posts, comments, votes) with vanilla instances while keeping governance signals fork-only.
- **Decision:** Yes, content-level federation with vanilla Lemmy works normally. Governance activities are new AP types that vanilla will silently discard; they only propagate between Brehon-forks. A user on lemmy.world can read and post to a community on our fork; our governance system applies to their activity *on our instance*; the vanilla peer doesn't know about the jury verdict but any content action (removal, restriction) is still reflected in the federated content stream via existing Lemmy mechanisms.
- **Consequences:**
  - Immediate access to the existing Fediverse audience
  - Lower adoption friction — communities can migrate gradually
  - Governance model is not diluted (it applies per-instance)
  - Edge case: a vanilla user sanctioned on our instance is unknown to their home instance; no cross-instance reputation sharing in that direction
  - Clear documentation needed so users on vanilla instances understand they're subject to our governance when interacting with our communities
- **Alternatives considered:** Closed federation of Brehon-forks only (rejected — cuts off the existing audience, makes adoption much harder), limited inbound-only (rejected — asymmetric and confusing).
- **Enacted in:** [02-domain-model.md](02-domain-model.md) §8, [03-architecture.md](03-architecture.md) §3.3, [07-operations-and-federation.md](07-operations-and-federation.md) §5

### ADR-015 — GDPR strategy: pseudonymised actor IDs in the governance log

- **Date:** 2026-04-14
- **Status:** Accepted
- **Context:** GDPR's right-to-delete requires the platform to delete a user's personal data on request. The append-only, hash-chain-signed governance log (ADR-008) cannot be rewritten without breaking its integrity property. These two requirements appear in tension.
- **Decision:** The governance log never stores usernames, emails, or any direct personal identifier. It stores a **pseudonymous actor ID** — an opaque, per-instance identifier for each user. A separate `actor_pseudonym` table in Postgres maps `person_id ↔ pseudonymous_id`. On right-to-delete, the mapping row for that user is deleted. The log itself is untouched; the hash chain remains verifiable; the user becomes permanently anonymous in the audit trail. Case decisions, sanctions, and reputation events continue to be verifiable by hash; the identity behind them is unrecoverable.
- **Consequences:**
  - GDPR-compatible right-to-delete without tampering the log
  - Audit integrity preserved (the log still verifies)
  - A deleted user's prior cases remain publicly logged (as they should — transparency of process) but the actor becomes pseudonymous
  - Must ensure NO direct identifiers ever leak into the log (addresses, emails, display names in rationale text) — the redaction service must scrub these
  - New `actor_pseudonym` table needed; add to [04-data-model-and-api.md](04-data-model-and-api.md) §3
  - New responsibility on Redaction Service: scrub direct identifiers from rationale before log append — add to [06-security-and-threat-model.md](06-security-and-threat-model.md) §6
- **Alternatives considered:** Log scrub on request (rejected — breaks the hash chain and defeats ADR-008), legitimate-interest defence (rejected — legally shaky in EU, high regulatory risk), store identifiers then argue about it later (rejected — path-dependent; once identifiers are in the log you can't extract them cleanly).
- **Enacted in:** [04-data-model-and-api.md](04-data-model-and-api.md) §3 (new `actor_pseudonym` table), [06-security-and-threat-model.md](06-security-and-threat-model.md) §6 (redaction service responsibility)

---

## Open Questions

Numbered, dated, with owner + target resolution date. Resolve or escalate — open questions that sit past their target date should be promoted to a team conversation.

### OQ-001 — Per-community vs instance-wide reputation

- **Opened:** 2026-04-14
- **Owner:** TBD (backend lead)
- **Question:** Is reputation computed per-community only, per-instance only, or both? What happens when the same user has divergent reputation in two communities on the same instance?
- **Current lean:** Per-community in MVP. Instance-wide roll-up as a v1 derivation. The snapshot table already supports both via `community_id: Option<i32>`.
- **Blocks:** Finalising snapshot recalculation in [04-data-model-and-api.md](04-data-model-and-api.md).
- **Target:** Resolve before Step 5 of the delivery plan.

### OQ-002 — Rule-set versioning per community

- **Opened:** 2026-04-14
- **Owner:** TBD (PM)
- **Question:** How are community rule sets versioned? Does a case decided under rule-set v3 survive rule-set v4? How are old rationales pinned to their rule version?
- **Current lean:** Version table, case rows reference a `rule_version_id`, public log stores the rule text at decision time. Not in MVP.
- **Blocks:** v1 scope.
- **Target:** Before v1 design phase.

### OQ-003 — Restorative actions as distinct sanction variants?

- **Opened:** 2026-04-14
- **Owner:** TBD (domain)
- **Question:** Should "apology requirement", "content correction", and "community service" be distinct `SanctionAction` variants, or stay as flavoured `Label` / `TemporaryRestriction` with a description field?
- **Current lean:** Flavoured in MVP (keeps enum small). Distinct variants in v1 if usage is high enough to warrant dashboard filters.
- **Blocks:** UX design for sanction picker in juror UI.
- **Target:** Before v1 UX design phase.

### OQ-004 — Maximum concurrent jury assignments per user

- **Opened:** 2026-04-14
- **Owner:** TBD (backend)
- **Question:** Can a single user be an active juror on more than one case simultaneously? If so, what's the cap? Is it per-community or instance-wide?
- **Current lean:** Cap at 3 concurrent assignments in MVP; instance-wide. Tunable via OPA policy.
- **Blocks:** Jury selection algorithm — eligibility query must check concurrent count.
- **Target:** Before Step 4 (Jury endpoints) of delivery plan.

### OQ-005 — UX flows: onboarding, sponsorship, juror notification

- **Opened:** 2026-04-14
- **Owner:** TBD (design — no designer assigned yet)
- **Question:** The source chats contain no UX flows. We need: onboarding for sponsored and time-based entry; sponsor-request flow (who asks whom, how is it confirmed); juror notification (in-platform? email? push?); case review interface; appeal filing interface.
- **Current lean:** Defer to a design pass after MVP data model is stable. Juror notification can start as in-platform inbox items only.
- **Blocks:** Any user testing of the full flow; some of Step 4 and Step 5 of the delivery plan.
- **Target:** Before Step 5 of the delivery plan.

### OQ-006 — Case threshold calculation — exact formula

- **Opened:** 2026-04-14
- **Owner:** TBD (backend)
- **Question:** What exactly is `report_weight = base_weight × reporter_reputation × recency_factor`? What are the units? What is `base_weight`? How does `recency_factor` decay?
- **Current lean:** `base_weight = 1`, `reporter_reputation = clamp(reporting_accuracy / 100, 0.1, 2.0)`, `recency_factor = exp(-hours_old / 168)`. Threshold to open a case: weighted sum > 3.0. All tunable via OPA policy or config table.
- **Blocks:** Implementation of `create_report` handler.
- **Target:** Before Step 4 of the delivery plan.

### OQ-007 — How is a community created and governed in a fork that disables moderators?

- **Opened:** 2026-04-14
- **Owner:** TBD (PM + backend)
- **Question:** Lemmy communities today are created and governed by moderators. If our fork deprecates moderator powers, who creates a community, who defines its initial rule set, who appoints its first jury pool?
- **Current lean:** Instance admins can create communities in MVP. Community founders define an initial rule set and serve as facilitators (not rulers). A community's first cases are decided by an instance-wide jury pool until the community has enough members to form its own.
- **Blocks:** Anything that touches community creation flow.
- **Target:** Before Step 4 of the delivery plan.

### OQ-008 — Handling a direct moderator action on a case under jury review

- **Opened:** 2026-04-14
- **Owner:** TBD (backend)
- **Question:** During the compatibility-layer period (ADR-009), if a moderator acts directly on content that is also the target of an active case, what happens?
- **Current lean:** The direct action puts the case into an `admin-review` state and logs both actions to the governance log. The jury does not continue voting until the conflict is resolved by an admin + delay.
- **Blocks:** Migration plan.
- **Target:** During Step 1 of the delivery plan.

### OQ-009 — Juror anonymity in the decision phase

- **Opened:** 2026-04-14
- **Owner:** TBD (security + domain)
- **Question:** Are jurors anonymous to each other during deliberation? To the public? To the case target? What about after the decision?
- **Current lean:** Anonymous to the case target and the public always. Revealed to each other after accepting assignment. Aggregate count is public.
- **Blocks:** Notification and case-review UI.
- **Target:** Before OQ-005 resolves.

### OQ-010 — What keys sign the governance log in production?

- **Opened:** 2026-04-14
- **Owner:** TBD (security + ops)
- **Question:** HSM-backed? Cloud KMS? Multi-party (threshold) signing? Rotation cadence? Recovery procedure if the signer is lost?
- **Current lean:** Cloud KMS for staging, HSM for production. Rotation yearly. Multi-party signing is desirable but deferred beyond v1.
- **Blocks:** Production deployment.
- **Target:** Before staging launch. **Note:** signer-outside-runtime is a **v2** milestone per ADR-010; for v0/v1 the key lives in `.env`.

### OQ-011 — First target community / primary persona

- **Opened:** 2026-04-14
- **Owner:** TBD (solo dev)
- **Question:** Who is the specific first community that will pilot this platform? "Grassroots organisations" is a category, not a user. A concrete pilot community changes: the default rule set, which v0 features matter most, the MVP demo narrative, localisation needs.
- **Current lean:** Defer. Ship v0 with a generic default rule set (harassment, spam, off-topic, illegal content, rule-breaking, impersonation). Pick a concrete pilot community during v1 so self-host packaging can be shaped around real needs.
- **Blocks:** Nothing in v0. Becomes blocking when the `deploy/` directory and default seed migration ship in v1.
- **Target:** Before v1 self-host packaging work starts.

### OQ-012 — Project name / branding

- **Opened:** 2026-04-14
- **Owner:** TBD (solo dev)
- **Question:** The fork needs an actual name before the first public push: repo, domain, deploy hostname, federation handshake, community identity. "Brehon-law-inspired network" is a working title.
- **Current lean:** Defer until closer to first public push. Working title stays in docs; final name goes into `00-README.md` and repo setup once chosen. Candidate naming vectors: early-Irish-law terms (Túath, Fíanna, Brehon, Cáin), English words that evoke the model (Tribute, Compact, Witness, Accord), or neutral project names that don't lean on historical branding.
- **Blocks:** Nothing technical. Blocks the first public push / domain registration.
- **Target:** Before v0 ships publicly.

---

## Changelog

Append-only record of spec changes. When any of the numbered docs gets a non-trivial revision, add an entry here.

**2026-04-14** — *All docs*
Initial creation from chat1.md + chat2.md. Captured brainstorming into dev-team-facing docs.

**2026-04-14** — *99, 05, 07*
ADR-010 added; v1 split into v1/v2/v3 milestones. Solo-dev realistic staging of post-MVP work.

**2026-04-14** — *99, 06, 02, 04*
ADRs 011–015 added; OQ-011, OQ-012 opened. Handoff-readiness pass: AGPLv3 licence, Lemmy 1.0-beta base version, illegal-content emergency-remove path, federation interop with vanilla Lemmy, GDPR pseudonymisation strategy.
