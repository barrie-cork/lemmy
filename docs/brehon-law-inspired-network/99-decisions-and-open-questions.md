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
- **Amended:** 2026-04-17 (after Brehon-law historical-fidelity review)
- **Owner:** TBD (domain)
- **Question:** Should "apology requirement", "content correction", and "community service" be distinct `SanctionAction` variants, or stay as flavoured `Label` / `TemporaryRestriction` with a description field?
- **Current lean (amended 2026-04-17):** v0 reserves a single parent variant `Restoration { description: String }` (added in Phase 5b task 56 alongside the severity-mapping match) to avoid the downstream-migration cost of introducing a new variant later across every exhaustive match site, every governance_log entry, every sanction row, and every federation-outbound publish. v0 treats `Restoration` as minor-severity (same as `Label`) for sponsor-liability purposes. Juror UI does not expose `Restoration` selection in v0 — it is a reserved slot for future code paths (e.g. `admin_restorative_action` endpoint). v1 refines `Restoration` into `Apology | ContentCorrection | CommunityService` variants — a refinement of an existing variant rather than replacement of a `Label`-with-description pattern. Rationale: `folog n-othrusa` (sick maintenance, Higgins 2010 pp.6–7) is a core Brehon mechanic named in [vision §4 principle 5](01-vision-and-principles.md) as non-negotiable; the v0 enum should reserve its slot even if v0 juries don't yet populate it.
- **Blocks:** UX design for sanction picker in juror UI.
- **Target:** Before v1 UX design phase.

### OQ-004 — Maximum concurrent jury assignments per user — ✅ RESOLVED 2026-04-17

- **Opened:** 2026-04-14
- **Resolved:** 2026-04-17
- **Owner:** solo dev (backend)
- **Question:** Can a single user be an active juror on more than one case simultaneously? If so, what's the cap? Is it per-community or instance-wide?
- **Resolution:** Cap at 3 concurrent assignments in v0; instance-wide; config-driven via `config.jury.max_concurrent_assignments = 3`. v1 OPA migration path preserved (OPA reads from `governance_config` per ADR-005 footnote). Ratifies Phase 5b task 57 implementation.
- **Blocks:** ~~Jury selection algorithm — eligibility query must check concurrent count.~~ Unblocked.
- **Target:** ~~Before Step 4 (Jury endpoints) of delivery plan.~~ Done.

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
- **Historical fidelity note (2026-04-17):** the multiplicative structure (`base × reputation × recency`) is grounded in Brehon evidence weighting (Higgins 2010 p.4: "the oath of a high ranking person automatically outweighing that of a lower ranking person"). The 0.1 floor matches the graduated-but-never-zero pattern (every rank's oath was heard, just outweighed). The 2.0 ceiling is an MVP choice — the historical model had no formal ceiling, only an effective one via finite status hierarchy. v0's 2.0 errs narrower than a full-hierarchy ceiling would imply, favouring consensus-of-reports over single-reporter weight. Pilot-week retro should revisit the ceiling against first-pilot data; if weighted-single-reports concentrate governance weight too narrowly, lower the ceiling or raise `case_threshold_micros` rather than raising the ceiling.
- **Blocks:** Implementation of `create_report` handler.
- **Target:** Before Step 4 of the delivery plan.

### OQ-007 — How is a community created and governed in a fork that disables moderators? — ✅ RESOLVED 2026-04-16

- **Opened:** 2026-04-14
- **Resolved:** 2026-04-16
- **Owner:** TBD (PM + backend)
- **Question:** Lemmy communities today are created and governed by moderators. If our fork deprecates moderator powers, who creates a community, who defines its initial rule set, who appoints its first jury pool?
- **Resolution:** Instance admins create communities in v0. Lemmy's existing `create_community` API is unchanged in the fork. The Phase 4 e2e test (task 48) seeds communities via admin user. Community governance (founder facilitators, initial rule sets, instance-wide jury pool fallback) is deferred to v1 per ADR-010. No code change needed in v0.
- **Blocks:** ~~Anything that touches community creation flow.~~ Unblocked.
- **Target:** ~~Before Step 4 of the delivery plan.~~ Done.

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

### OQ-013 — Endorsement-creation reputation deltas — ✅ RESOLVED 2026-04-17

- **Opened:** 2026-04-17
- **Resolved:** 2026-04-17
- **Owner:** solo dev (backend)
- **Question:** When one user endorses another (creating a `surety` / `endorsement` pair), what immediate reputation deltas fire, on which dimensions, for which party? The vision doc says "repeated bad sponsorship disables the ability to sponsor further" but says nothing about the creation-time credit.
- **Resolution:** At endorsement creation, emit two `reputation_event` rows: `+5` on `EndorsementStrength` for the sponsor, `+5` on `ParticipationConsistency` for the sponsee. Both values are config-driven (`governance.deltas.endorsement_created_sponsor` and `governance.deltas.endorsement_created_sponsee`) so they can be tuned without code changes once pilot data arrives. Ratifies plan task 55's implementation.
- **Blocks:** ~~`create_endorsement` handler.~~ Unblocked.
- **Target:** ~~Phase 5a.~~ Done.

### OQ-014 — `can_sponsor` threshold value — ✅ RESOLVED 2026-04-17

- **Opened:** 2026-04-17
- **Resolved:** 2026-04-17
- **Owner:** solo dev (backend)
- **Question:** Vision §5 requires sponsors to have "sufficient reputation to sponsor" before they can endorse new users. What reputation number counts as "sufficient"? How does the first cohort of users ever cross it (cold-start bootstrap)?
- **Resolution:** v0 uses an **account-age gate only** — `config.onboarding.sponsor_min_account_age_days = 30`. The `can_sponsor` boolean on `reputation_snapshot` is computed against a reputation threshold of `25` on `endorsement_strength` (for observability + future use) but is **not read by any v0 handler**. v1 flips `config.sponsorship.require_reputation_gate = true` to activate enforcement. This deliberately relaxes vision §5's reputation requirement for v0 to solve the cold-start problem — fresh communities have no one with earned `endorsement_strength` yet. Documented in plan task 55.
- **Blocks:** ~~`create_endorsement` handler gate logic.~~ Unblocked.
- **Target:** ~~Phase 5a.~~ Done.

### OQ-015 — *(reserved; sequence gap intentional until a matching question emerges)*

### OQ-016 — Deferred-enforcement `membership_state` column — ✅ RESOLVED 2026-04-17

- **Opened:** 2026-04-17
- **Resolved:** 2026-04-17
- **Owner:** solo dev (backend)
- **Question:** Should v0 ship the `person.membership_state` column + `MembershipState` enum (`Member | Provisional | Suspended`) even though no v0 handler will read or enforce it? Or defer the schema to v1 alongside the handlers that use it?
- **Resolution:** Ship the column in Phase 5a with `DEFAULT 'member'` so existing users grandfather in. Patch Lemmy's register handler to write the config-driven default (`governance.onboarding.default_membership_state`). v0 handlers do **not** guard on the column; v1 flips config and adds handler-side guards without a schema migration. Rationale: migrations on `person` are expensive at scale — ship once, enforce later. Plan task 51 implements.
- **Blocks:** ~~v1 provisional-membership feature design.~~ Unblocked for v0 work.
- **Target:** ~~Phase 5a.~~ Done.

### OQ-017 — Founder seeding governance

- **Opened:** 2026-04-17
- **Owner:** TBD (solo dev + pilot community facilitators)
- **Question:** Phase 5a task 59 ships a `seed_founders` CLI that injects decayable `reputation_event` rows via admin-level DB credentials. The mechanism is built; the **policy** around it is undecided. Who in the community is allowed to run it? Is seeding a one-time bootstrap action, a periodic re-seeding of new cohorts, or both? How do regular community members discover who's been seeded as a founder and when their seed expires? Should seed expiry be visible in the user's public profile, or only in the modlog?
- **Current lean:** For v0, seeding is a solo-dev action (only the instance operator holds DB credentials). Every seed emits a modlog-surfaced `founder_seeded` entry per plan task 59, so the fact of seeding is always public even if the policy isn't formalised. v1 adds a governance process (perhaps: a meta-case where existing members jury on proposed seeds) plus a profile-level "founder seal" badge that renders for as long as the seed is unexpired. Multi-admin seeding (where independent admins seed different cohorts) is deferred further — by v1 the community will have informal norms that inform the formal process.
- **Blocks:** Nothing in v0. Becomes relevant when a second cohort of founders needs to be seeded after the initial bootstrap expires, typically 3–12 months post-launch.
- **Target:** Before the first post-bootstrap re-seeding action (estimate: 6 months after v0 ships).

### OQ-018 — Admin config write endpoint shape

- **Opened:** 2026-04-17
- **Owner:** TBD (backend)
- **Question:** v0 ships `governance_config` as read-only from the code's perspective — admins edit rows via direct `UPDATE governance_config` over psql. v1 needs an HTTP endpoint so the operator (or eventually a community admin via a web UI) can change config without SSH. What's the shape? Per-key single-value writes (`POST /admin/config { key, value }`)? Batch writes (`POST /admin/config { changes: [...] }`)? Dry-run preview (`POST /admin/config?dry_run=true` returns what would change without writing)? Who can call it — instance admins only, or scoped community admins for `scope = 'community:<id>'` rows? What's the audit trail — one `governance_log` entry per row change, or one per batch?
- **Current lean:** v1 ships `POST /api/v4/governance/admin/config` with a **per-key single-value shape**, instance-admin-only in v1, one `governance_log` entry per change. Batch writes and community-scoped admin writes are v1+ if pilot feedback says they're needed. Dry-run as a `?dry_run=true` query param from day one — the whole point of config is that admins iterate on values, and dry-run lets them see downstream effects (e.g. "how many users would lose jury_eligible if I raise the threshold?") before committing.
- **Blocks:** v1 admin-UX work. Until then, the DB-edit path is operationally sufficient for a pilot community with one operator.
- **Target:** v1 scope.

### OQ-019 — `participation_consistency` event sources

- **Opened:** 2026-04-17
- **Owner:** TBD (domain + backend)
- **Question:** In v0, the only writer to the `participation_consistency` reputation dimension is `create_endorsement` (emits `+5` to the sponsee per OQ-013). That means a user who is never sponsored and never sponsors has this counter stuck at zero forever, regardless of how active they are. The vision doc §5.3 describes this dimension as "regular, constructive participation in community" — which implies it should grow with actual participation. What events fire to move it? Options: (a) periodic activity scan (cron: "+1 per week for users with ≥1 comment in that week"); (b) event-driven from post/comment creation (higher frequency, closer to activity); (c) manual admin-entered attestations for real-world participation (attended a meeting, volunteered at an event); (d) some combination. This dimension is the one most tied to the platform's "real-world signals matter most" principle — whichever mechanism lands must not reduce participation to "number of comments posted."
- **Current lean:** v1 ships **option (a)** — a weekly cron that emits `+1 participation_consistency` per user per community where they had ≥1 comment that week, and `-2` for users dormant >30 days. All values config-driven. Option (c) (admin attestations) is also attractive but needs a new `activity_attestation` table and UX; defer to v2 unless the pilot community explicitly asks for it. Option (b) (event-driven per post/comment) risks incentivising spam posting, so it is **rejected** even as a future option.
- **Blocks:** Nothing in v0 (the dimension exists, the column is populated, the threshold machinery works — it just stays at zero for most users). Becomes a usability issue in v1 when users start asking "how do I improve my participation score?".
- **Target:** v1 scope.

### OQ-022 — Sponsor founder-multiplier timestamp semantics — ✅ RESOLVED 2026-04-17

- **Opened:** 2026-04-17
- **Resolved:** 2026-04-17
- **Owner:** solo dev (backend)
- **Question:** Phase 5b task 56 checks each sponsor's founder status via `EXISTS (reputation_event WHERE person_id = ? AND dimension = 'endorsement_strength' AND expires_at IS NOT NULL AND expires_at > now())`. The `now()` is the case-close time (when `apply_sponsor_liability` fires). What about a sponsor whose founder seed was active when the case opened but expired during jury deliberation? Under `now()` semantics they get `regular_multiplier`. Is that correct (the seed was expired at decision-time) or wrong (the sponsorship that caused the liability happened while they were a founder)?
- **Resolution:** `now()` at case-close is correct. Liability is assessed on present-tense sponsor status. A founder who lets their seed expire mid-case is no longer a founder; the Brehon mechanic's principle that "reputation reflects current standing, not historical" supports this. Federation reproducibility (Phase 6 task 70) is preserved because every node computes against `now()` at the same case-close transaction. Plan task 56 must document this in its GOTCHA so the impl agent doesn't drift to `case.opened_at` semantics.
- **Blocks:** ~~Federation reproducibility (Phase 6 task 70) — every node must agree on the timestamp semantic.~~ Unblocked.
- **Target:** ~~Phase 5b task 56.~~ Done.

### OQ-024 — v0 zero-floor clamp on sponsor-liability deltas (honour-price mapping) — ✅ RESOLVED 2026-04-17

- **Opened:** 2026-04-17
- **Resolved:** 2026-04-17
- **Owner:** solo dev (backend)
- **Question:** Should Phase 5b task 56 (`apply_sponsor_liability`) clamp each per-sponsor delta so the sponsor's post-liability `endorsement_strength` cannot go below zero? The Brehon honour-price rule (Higgins 2010 p.4) capped surety at the sponsor's own status value — a sponsor could not be made liable for more than they had to lose. v0 was about to ship without any cap; a sponsor could accrue `endorsement_strength = −10,000` after enough sanctions, producing the permanent-outcast state that [vision §1 principle 5](01-vision-and-principles.md) explicitly forbids. This is distinct from OQ-021 (negative-reputation floor + reintegration) — OQ-021 asks the broader v1 question about how sponsors recover from below-zero reputation and whether a negative band is valid at all. OQ-024 asks the narrower v0 question: does v0 permit below-zero at all?
- **Resolution:** Clamp at zero in v0. Add `governance.liability.sponsor_liability_floor = 0` as a new config key (type `int`, instance-scoped) seeded by Phase 5a task 50; read it in `apply_sponsor_liability` (Phase 5b task 56) before writing the `reputation_event`; clamp the final delta so `new_endorsement_strength ≥ floor`. ~3 lines in task 56, one seed row in task 50. The disablement mechanic (sponsors with `endorsement_strength < can_sponsor_threshold = 25` lose `can_sponsor`) still fires exactly as currently designed; what the clamp prevents is the permanent-outcast failure mode. v1 (via OQ-021) can add a negative band below zero with explicit reintegration ceremony — v0's clamp becomes the configurable default, not the hard rule. Per OQ-024 the config key takes values `'zero'` (v0 default) | `'unbounded'` (preserves the pre-clamp behaviour, opt-in for operators who want it) | `'honour_price_fraction'` (v1: clamp at a fraction of the sponsor's `endorsement_strength` at endorsement time, the most Brehon-faithful interpretation).
- **Blocks:** ~~Phase 5b task 56 final shape.~~ Unblocked.
- **Target:** ~~Resolve before Phase 5b starts.~~ Done.

### OQ-025 — Explicit post-decision grace window for sponsor liability (athgabál analogue)

- **Opened:** 2026-04-17
- **Owner:** TBD (domain + backend)
- **Question:** Brehon distraint (`athgabál`, Higgins 2010 p.7) required formal notice to the defendant with a grace period "depending on the nature of the wrong committed" before seizure could take place. Phase 5b task 56 fires sponsor-liability in the same transaction as the jury's sanction insert — no notice, no grace window. Should v1 add an explicit `sponsor_liability_pending` case state between `Decided` and the sponsor's reputation-event write, with a severity-proportional grace window (config-driven) during which the sponsor can revoke endorsement to escape liability?
- **Current lean:** v1 scope. v0's implicit window (case deliberation time provides several days during which a sponsor monitoring the modlog can revoke) is acceptable for MVP but diverges from the Brehon notice requirement. v1 adds: new `CaseStatus::SponsorLiabilityPending`, new governance_log entry kind `sponsor_liability_notice_issued`, config keys `sponsor_grace_minor_hours`, `sponsor_grace_moderate_hours`, `sponsor_grace_severe_hours` (likely 24/72/168 default). Sponsor revocation during the window emits a modlog entry and escapes liability (the reputation_event is never written). Failure to revoke → window expires → reputation_event fires at the same `apply_sponsor_liability` logic currently in task 56.
- **Blocks:** v1 scope. Not blocking v0.
- **Target:** v1, alongside OQ-018 (admin config write endpoint) which provides the UI to tune grace windows.

### OQ-026 — Status-aware rule extension pattern for governance_config

- **Opened:** 2026-04-17
- **Owner:** TBD (backend)
- **Question:** Phase 5a task 50's typed-column `governance_config` handles the current 32 v0 keys cleanly, but Brehon procedure was heavily status-conditional (e.g. fasting applied only to high-rank defendants, Higgins 2010 p.7). Phase 5 already ships one status dimension (`founder` via `expires_at`), consumed in task 56 for `founder_multiplier`. When v1 needs more status-conditional rules (e.g. `jury.panel_size.founder = 7`, `jury.panel_size.regular = 5`), which extension pattern does `governance_config` use: (a) keyed-by-status rows with dotted namespace + reader-side cascade; (b) per-status multiplier keys composed at read time; (c) add nullable `status_tier TEXT` column (schema migration); (d) migrate to OPA at v2+ for composable rules?
- **Current lean:** v1 uses (a) or (b) depending on rule shape; schema unchanged. v2+ migrates composable rules to OPA. Option (c) only considered if patterns (a) / (b) become unwieldy (>3 status-conditional variants per rule, which v1 is unlikely to hit).
- **Blocks:** v1 status-aware rule design. Not blocking v0.
- **Target:** v1 scope. Document the migration path before pilot retro so status-conditional rule requests from the pilot can be slotted against a known v1 shape.

### OQ-020 — Sponsor gate-strategy expansion

- **Opened:** 2026-04-17
- **Owner:** TBD (backend + domain)
- **Question:** v0 ships three sponsor-gate strategies in `config.onboarding.sponsor_gate_strategy`: `'age'` (default — account age ≥ N days), `'open'` (bypass all checks, recruitment-drive mode), `'closed'` (reject all endorsement attempts, emergency lockdown). These cover the pilot scenarios identified 2026-04-17 but not more sophisticated shapes. Which additional strategies does v1 need, and what's the evaluation order when multiple factors apply?
- **Current lean:** v1 adds three more strategies. (1) `'age_or_surety'`: age gate OR caller has ≥1 active surety — lets newly-vouched members sponsor before hitting the age threshold. (2) `'reputation'`: reads `reputation_snapshot.can_sponsor` (already populated in v0 per the Task 50 column addition; v1 flips config to activate). (3) `'allowlist'`: explicit `person_id` list stored in a new `sponsor_allowlist` table — for high-trust cohorts admins want to bless individually. Each strategy needs its own integration-test matrix. The strategy is a single string, not a composition — "age_or_surety_or_allowlist" is rejected because every new combination is exponential test cost; composition lives in v2 if needed.
- **Blocks:** Nothing in v0. `'open'` covers the event-driven recruitment use case until real pilot data motivates something more structured.
- **Target:** v1, likely driven by the first community-event-after-launch that needs non-default sponsor gating.

### OQ-V1-AD-01 — Server-rendered HTML page framework for admin dashboard

- **Opened:** 2026-04-21
- **Owner:** TBD (backend)
- **Question:** v1 admin-dashboard PRD §6 assumes askama (or maud) for server-rendered admin HTML pages. Neither crate is in the workspace `Cargo.lock` today — adoption would add a templating dep + compile-time step to `lemmy_api`. Alternative: ship v1-AD-d API-only and defer HTML pages to v1.x or v2 (to land alongside the React frontend pass).
- **Current lean:** Defer to v1.x. v1-AD-d ships API-only; askama decision revisits after pilot operator feedback on whether curl+jq-only workflow is operationally sufficient.
- **Blocks:** v1-AD-e (askama HTML pages sub-phase).
- **Target:** Before v1-AD-e plan writes.

### OQ-V1-AD-02 — SSE implementation: actix-web-lab vs hand-rolled

- **Opened:** 2026-04-21
- **Owner:** TBD (backend)
- **Question:** PRD §4 enumerates `GET /api/v4/governance/admin/audit/stream` as an SSE endpoint. Upstream Lemmy does not use `actix-web-lab` (grep of `Cargo.lock` 2026-04-19 returns no matches). Two paths: (a) add `actix-web-lab` dep for its `Sse` helper; (b) hand-roll ~80 LOC of chunked-response plumbing using `async-stream` (already a transitive dep). The Postgres `LISTEN governance_events` channel already exists (migration `2026-04-20-000000-0000`) — the SSE endpoint's only job is bridging Postgres NOTIFY payloads onto HTTP.
- **Current lean:** Hand-roll (option b). Adds no new supply-chain surface and keeps Lemmy's actix-web dep shape intact.
- **Blocks:** v1-AD-d (dashboard + SSE sub-phase).
- **Target:** Before v1-AD-d plan writes.

### OQ-V1-AD-03 — Dry-run impact computation inside vs outside run_transaction

- **Opened:** 2026-04-21
- **Owner:** TBD (backend)
- **Question:** PRD §4.3 proposes running dry-run impact queries inside the same `run_transaction` as the config write with a `SAVEPOINT` rollback when `dry_run = true`. Inspection of `crates/diesel_utils/src/connection.rs:68-79` shows `run_transaction` wraps `diesel-async`'s `.transaction()` — no SAVEPOINT primitive is exposed. Either (a) extend `run_transaction` with a `run_savepoint` helper, or (b) compute impact as a read-only query BEFORE the tx opens against the proposed value and the current config snapshot, with no rollback needed.
- **Current lean:** Option (b). Architecturally cleaner; dry-run is read-only; no tx-state visibility required. Reshape PRD §4.3 to match.
- **Blocks:** v1-AD-b (POST /admin/config sub-phase).
- **Target:** Before v1-AD-b plan writes.

---

## Changelog

Append-only record of spec changes. When any of the numbered docs gets a non-trivial revision, add an entry here.

**2026-04-14** — *All docs*
Initial creation from chat1.md + chat2.md. Captured brainstorming into dev-team-facing docs.

**2026-04-14** — *99, 05, 07*
ADR-010 added; v1 split into v1/v2/v3 milestones. Solo-dev realistic staging of post-MVP work.

**2026-04-14** — *99, 06, 02, 04*
ADRs 011–015 added; OQ-011, OQ-012 opened. Handoff-readiness pass: AGPLv3 licence, Lemmy 1.0-beta base version, illegal-content emergency-remove path, federation interop with vanilla Lemmy, GDPR pseudonymisation strategy.

**2026-04-17** — *99*
OQ-013, OQ-014, OQ-016 added and resolved (OQ-015 reserved as a sequence placeholder). Resolutions ratify the 2026-04-17 Phase 5 reframe: endorsement-creation reputation deltas, age-only `can_sponsor` gate with computed-but-unread boolean, and deferred-enforcement `membership_state` column.

**2026-04-17** — *99*
OQ-017 (founder seeding governance), OQ-018 (admin config write endpoint shape), OQ-019 (`participation_consistency` event sources), OQ-020 (sponsor gate-strategy expansion) opened. All four are v1 scope; v0 ships mechanisms, v1 formalises policy. Closes the cross-reference gap where `IMPLEMENTATION-PLAN-v0.md §3 Phase 5` pointed at these OQ numbers as if they existed.

**2026-04-17** — *99*
OQ-004 resolved (cap=3 instance-wide, config-driven via `config.jury.max_concurrent_assignments`). OQ-022 (sponsor founder-multiplier timestamp = `now()` at case-close) opened and resolved same day. Both ratify Phase 5 design-soundness review findings on jury concurrency cap and federation reproducibility for sponsor-liability math.

**2026-04-17** — *99, 01, IMPLEMENTATION-PLAN-v0*
OQ-024 (v0 zero-floor clamp on sponsor-liability deltas, honour-price mapping) opened and resolved same day. OQ-025 (post-decision grace window, athgabál analogue) and OQ-026 (status-aware config extension pattern) opened, both deferred to v1. OQ-003 (restorative actions) amended: v0 reserves a single `Restoration { description: String }` variant in Phase 5b task 56 to avoid v1 data-migration tax across exhaustive matches and federation-outbound publishes; v1 refines into `Apology | ContentCorrection | CommunityService`. OQ-006 (case threshold formula) amended with a historical-fidelity note: the multiplicative structure is grounded in Brehon evidence weighting (Higgins 2010 p.4); the 0.1 floor matches the graduated-but-never-zero pattern; the 2.0 ceiling errs narrow vs the historical full-hierarchy ceiling and should be revisited at pilot retro. All five OQ changes ratify findings of the Brehon-law historical-fidelity review (PHASE-5-HISTORICAL-FIDELITY.md, uncommitted).

**2026-04-21** — *99*
OQ-V1-AD-01, OQ-V1-AD-02, OQ-V1-AD-03 opened. All three block v1-AD sub-phases (e/d/b respectively). Leans documented; resolution required before dependent plans write.
