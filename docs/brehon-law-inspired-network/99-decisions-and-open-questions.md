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
- **Status:** Accepted — **amended by [ADR-016](#adr-016--brehon-is-a-cross-app-governance-backplane-federated-app-planes) on 2026-05-23** to add federated app planes outside the Brehon binary.
- **Context:** A compromised admin on the content plane must not be able to silently finalise governance changes. Security reviewers (OWASP, CISA) consistently recommend separating privileged and general-purpose code paths.
- **Decision:** Governance code lives in its own route tree (`/api/v4/governance/*`), its own handler modules (`crates/api/api/src/governance/*`), and its own permission boundary. Every governance write emits an append-only signed log entry before responding. High-risk governance actions require step-up auth, and some require quorum + delay.
- **Consequences:**
  - Cleaner testability and auditability
  - Compromise-containment is realistic
  - Admin action on the governance plane is visible, verifiable, and revertible
- **Alternatives considered:** Single unified API with per-endpoint authz (rejected — hard to audit, easy to regress into).
- **Enacted in:** [03-architecture.md](03-architecture.md) §4, [06-security-and-threat-model.md](06-security-and-threat-model.md) §2.2
- **Amendment (2026-05-23, via ADR-016):** The two-plane model (governance + content) describes the Brehon binary's internal architecture. ADR-016 extends the model with **federated app planes** — independently operated external apps (Matrix, PeerTube, etc.) that participate in Brehon governance over a defined federation contract. The within-Brehon two-plane invariant is unchanged; app planes are not in-process and do not share Brehon's permission boundary. See ADR-016 for the cross-app contract.

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

### ADR-016 — Brehon is a cross-app governance backplane; federated app planes

- **Date:** 2026-05-23
- **Status:** Accepted
- **Amends:** [ADR-004](#adr-004--governance-plane-separated-from-content-plane) (extends two-plane model with federated app planes)
- **Context:** Brehon's purpose is to provide a substrate for community self-governance, not to be tied to a single content app. Communities standing up alternatives to WhatsApp, YouTube, Reddit, etc. (typically Matrix, PeerTube, Lemmy-fork, and successors) need *one* governance system — one jury pool, one reputation graph, one rule set — applied to misbehaviour wherever it happens. The existing two-plane model (ADR-004) describes Brehon's internal architecture but is silent on apps that live outside the Brehon binary. The V2 messaging PRD (see `.claude/PRPs/prds/v2-messaging-rtc.prd.md`) is the first cross-app integration and exposes the architectural question: how do external, independently-operated apps participate in Brehon governance without becoming part of the Brehon binary?
- **Decision:** Brehon is a **governance backplane**. Apps stay independent: each app is operated by whoever runs it, with its own admin, its own users, its own data. Apps connect to Brehon over a **federation-shaped contract** with three load-bearing components:
  1. **Evidence fetch (B-fetch):** Brehon fetches evidence from the app's existing public API at report-submission time and archives a hashed snapshot. Reporter-side hash at observation catches admin tampering. Per-app adapters live in Brehon's federation fetch worker ([07 §1.2](07-operations-and-federation.md)). Apps do not implement Brehon-specific signing.
  2. **Sanction publish/subscribe (B-publish):** Brehon publishes sanction events on a federation channel. Apps that opt into Brehon governance run a small subscriber that receives events and translates them into local app primitives (Matrix mutes, PeerTube demonetisations, Lemmy-fork removals). Brehon never holds admin credentials on connected apps; the app retains sovereignty.
  3. **Portable actor ID (B-actor):** Brehon issues a portable actor ID per Brehon-participating user. Each app maintains a local mapping from its native user identifier to the Brehon actor ID, established at user opt-in via a Brehon-link flow. Reputation, sanctions, and sponsorship key off the portable actor ID. Apps remain their own identity authority for login; Brehon is *not* an identity provider (no OIDC IdP role).

  The V2 messaging PRD is the **first reference integration** of this contract. Messaging phases are renamed from V2a/V2b/V2c to **M1 (chat infrastructure) / M2 (governance-triggered rooms) / M3 (town halls with mic-passing)** to end the naming collision with [ADR-010](#adr-010--staged-releases-v0--v1--v2--v3)'s v2-security-hardening release. Future apps (PeerTube, others) get their own ADRs codifying their adapter + their participation, citing this ADR-016 as the parent contract.

- **Consequences:**
  - Brehon binary stays free of app-specific code beyond per-app adapters in the federation fetch worker. The Lemmy-fork integration is internal (Brehon and Lemmy-fork share a process today) but is the *exception* to be unified with the cross-app contract over time, not the rule that other apps must follow.
  - Cross-app reputation, sanctions, and trust labels become coherent: one actor, one reputation graph, one set of capabilities visible across every integrated app.
  - Adoption barrier per app is low: communities standing up Matrix/PeerTube/etc. can use the standard upstream release; they do not need Brehon-patched forks. They run a small subscriber alongside the app and register with their Brehon node.
  - Brehon is *not* a single point of failure for app operation: if Brehon is down, apps continue working; only governance events queue. If Brehon is compromised, apps' admin authority is unaffected (no held credentials), and sanctions can be locally suspended until Brehon recovers.
  - E2EE and private-content scenarios are partially supported: B-fetch requires content to be fetchable by Brehon. E2EE messages, private rooms, and gated content require either reporter-supplied decrypted snapshots (loses tamper-evidence — jury weighs accordingly) or app-side cooperation to expose the content. The contract does not promise integrity for content the app cannot serve.
  - Cross-pod / cross-instance federation ([ADR-006](#adr-006--remote-sanction-notices-are-advisory-only-in-mvp) shape) layers cleanly on top: Pod A's verdict is advisory to Pod B; both pods independently publish to their connected apps.
  - The messaging V2 PRD's rename to M1/M2/M3 must be propagated when the PRD next gets edited. The cross-app contract details (protocol shapes, adapter spec, subscriber spec, link-flow UX) are deferred to the M1 sub-PRD and per-app integration ADRs — this ADR commits the principles, not the wire formats.
  - New open questions opened (see §Open Questions below): OQ-ADR016-01 (B-fetch protocol shape and per-app adapter SPI), OQ-ADR016-02 (B-publish event schema and subscriber contract), OQ-ADR016-03 (B-actor link-flow UX, mapping table location, sign-claim model to prevent re-pointing), OQ-ADR016-04 (sanction translation semantics — what "muted globally" means in each app's primitives). Note: the `ADR016-` prefix avoids collision with the pre-existing OQ-016 (deferred-enforcement `membership_state` column, resolved 2026-04-17).

- **Alternatives considered:**
  - **App signs evidence with its instance key (rejected):** Strongest cryptographic chain of custody but requires every app to implement Brehon-specific signing endpoints, forcing forks of Matrix/PeerTube/etc. Breaks "apps stay independent" and the signature only proves the homeserver attests to the evidence — not that the event happened — so the integrity gain over B-fetch + reporter hash is marginal.
  - **Brehon writes to apps via admin API (rejected):** Strongest enforcement but Brehon holds admin credentials on every connected app, becoming a super-admin across communities. Massive blast radius if Brehon is compromised; tight coupling; many apps and admins will refuse the trust requirement.
  - **Sanctions advisory only, local mods enforce (rejected):** Respects app autonomy maximally but undermines cross-app governance — a jury says "muted for 7d" and individual app mods may ignore it, so reputation rolls up incoherently. The cross-app value proposition collapses.
  - **Single sign-on through Brehon (OIDC IdP) (rejected):** Tightest identity binding but makes Brehon a SPOF for app login, locks app choice to OIDC-capable apps, expands Brehon's scope to identity-provider (with all its threat model — account recovery, MFA, breach response). Identity sovereignty is a value the federation-shaped model preserves; SSO sacrifices it.
  - **Apps own identity, Brehon stores per-app IDs separately (rejected):** Lowest integration burden but reputation never rolls up across apps without per-user manual linking; bad actors evade sanctions by switching apps. Functionally turns Brehon into N parallel governance silos.
  - **Defer the cross-app architecture until forced by a second app (rejected):** Leaves the V2 messaging PRD as a one-off integration. Future apps would have to retrofit. Better to commit the contract once and let M1 prove it.

- **Enacted in:**
  - `.claude/PRPs/prds/v2-messaging-rtc.prd.md` (rename V2a/V2b/V2c → M1/M2/M3 + first reference integration of B-fetch / B-publish / B-actor)
  - [03-architecture.md](03-architecture.md) §4 (extend plane model with "federated app planes" subsection)
  - [06-security-and-threat-model.md](06-security-and-threat-model.md) §2.2 (extend plane boundary wording), §7 (new threat-table rows for B-fetch adapter, B-publish subscriber, B-actor mapping compromise)
  - [07-operations-and-federation.md](07-operations-and-federation.md) §1.2 (federation fetch worker carries per-app B-fetch adapters), §5 (operator runbook for connected-app onboarding)
  - Future companion doc `08-cross-app-governance.md` (deferred — written when M1 sub-PRD is scheduled and the protocol-level detail is concrete)
  - Future ADR-017 onward (each new app integration cites this contract)

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
- **v1 tuning path (per OQ-028):** which sub-variants jurors actually pick during dogfood usage shapes the v1 refinement; bundle the selected variant set into the first named profile rather than committing to all three sub-variants up-front.
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
- **v1 tuning path (per OQ-028):** `base_weight`, `clamp_min`, `clamp_max`, `recency_half_life_hours`, `case_threshold_micros` are all config-driven from Phase 5a task 50; dogfood within the dev team between v0 ship and pilot launch produces evidence for the first named profile rather than overwriting hardcoded defaults.
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
- **v1 tuning path (per OQ-028):** the +5/+5 magnitudes are placeholder defaults; dev-team dogfood between v0 ship and pilot launch is the right evidence channel for the first named profile, kept distinct from pilot-1 evidence rather than collapsing both into a canonical overwrite.
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
- **v1 tuning path (per OQ-028):** the +1/-2 magnitudes and the 30-day dormancy threshold are pure feel-it-out numbers; dev-team dogfood produces the first named profile, with pilot-1 forking on its own retro evidence rather than overwriting.
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
- **v1 tuning path (per OQ-028):** which of the three modes (`zero` / `unbounded` / `honour_price_fraction`) the dev team converges on under dogfood usage feeds the first named profile; the choice is not a-priori obvious and benefits from running each mode for a stretch of internal use before the pilot retro.
- **Blocks:** ~~Phase 5b task 56 final shape.~~ Unblocked.
- **Target:** ~~Resolve before Phase 5b starts.~~ Done.

### OQ-025 — Explicit post-decision grace window for sponsor liability (athgabál analogue)

- **Opened:** 2026-04-17
- **Owner:** TBD (domain + backend)
- **Question:** Brehon distraint (`athgabál`, Higgins 2010 p.7) required formal notice to the defendant with a grace period "depending on the nature of the wrong committed" before seizure could take place. Phase 5b task 56 fires sponsor-liability in the same transaction as the jury's sanction insert — no notice, no grace window. Should v1 add an explicit `sponsor_liability_pending` case state between `Decided` and the sponsor's reputation-event write, with a severity-proportional grace window (config-driven) during which the sponsor can revoke endorsement to escape liability?
- **Current lean:** v1 scope. v0's implicit window (case deliberation time provides several days during which a sponsor monitoring the modlog can revoke) is acceptable for MVP but diverges from the Brehon notice requirement. v1 adds: new `CaseStatus::SponsorLiabilityPending`, new governance_log entry kind `sponsor_liability_notice_issued`, config keys `sponsor_grace_minor_hours`, `sponsor_grace_moderate_hours`, `sponsor_grace_severe_hours` (likely 24/72/168 default). Sponsor revocation during the window emits a modlog entry and escapes liability (the reputation_event is never written). Failure to revoke → window expires → reputation_event fires at the same `apply_sponsor_liability` logic currently in task 56.
- **v1 tuning path (per OQ-028):** the 24/72/168h defaults are placeholder magnitudes; how often dev-team-internal sponsors actually revoke during dogfooding (and at what severity) is the right evidence for the first named profile, with pilot-1 forking on its own retro evidence.
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
- **v1 tuning path (per OQ-028):** which of the three new strategies the dev team actually reaches for during dogfood usage shapes the v1 ship list and the first named profile's default; ship in priority order rather than building all three speculatively.
- **Blocks:** Nothing in v0. `'open'` covers the event-driven recruitment use case until real pilot data motivates something more structured.
- **Target:** v1, likely driven by the first community-event-after-launch that needs non-default sponsor gating.

### OQ-027 — Autonomi as governance-log anchor and evidence-storage backend (v2-research)

- **Opened:** 2026-04-25
- **Owner:** TBD (backend + infrastructure)
- **Question:** v0 anchors the governance log via a local sha2 hash chain in Postgres (ADR-003). v2 security hardening needs an external tamper-evidence layer. Should Autonomi's content-addressed immutable storage replace or complement blockchain anchoring for (a) governance-log hash batches, (b) case-evidence archival, and (c) reputation-snapshot checkpoints? Separately: does Autonomi's x0x real-time gossip layer (CRDT pub/sub) offer a viable alternative to the Matrix+LiveKit bridge planned in V2/messaging.md?
- **Options:**
  - **(a) Autonomi for anchoring only** — batch governance-log hashes to Autonomi chunks (pay-once, permanent, content-addressed). Simpler than blockchain; no gas fees; post-quantum transport. Communication stays Matrix.
  - **(b) Autonomi for anchoring + evidence storage** — (a) plus case evidence (screenshots, content snapshots) stored as private DataMaps with juror-scoped key delegation. Replaces an S3/object-store dependency.
  - **(c) Autonomi for anchoring + evidence + comms (x0x)** — (b) plus evaluate x0x gossip layer as a V2 messaging transport instead of Matrix. Highest reward but highest dependency risk — x0x is pre-production as of 2026-04.
  - **(d) No Autonomi** — proceed with blockchain anchoring per ADR-003's original intent; Matrix+LiveKit for V2 comms.
- **Current lean:** (b). The storage-layer fit is strong and aligns with ADR-003's "public memory" posture without blockchain complexity. x0x maturity is the gating question for (c) — evaluate when v1 federation (Phase 6, outbound-only AP) is shipping; by then x0x will have either matured or stalled, and the answer writes itself.
- **Key references:** ADR-003 (blockchain as public memory), ADR-012 (Extism plugin host), V2/messaging.md §4.4/§8.1, autonomi.com, github.com/maidsafe/autonomi.
- **Technical notes:** Autonomi is a Rust monorepo (94% Rust, libp2p/QUIC/Kademlia DHT). Post-quantum crypto (ML-KEM-768, ML-DSA-65). Self-encryption via ChaCha20-Poly1305 + BLAKE3. No ActivityPub — zero overlap with our federation layer. ANT token is storage-payment only (no governance-token conflict with ADR-002).
- **Blocks:** Nothing in v0 or v1. This is a v2-research item.
- **Target:** v2 security-hardening research spike. Revisit when v1 Phase 6 (federation) is shipping.

### OQ-028 — Named governance-profile bundles for v1 tuning rollout

- **Opened:** 2026-04-30
- **Owner:** TBD (solo dev)
- **Question:** Several v1-tunable values (jury thresholds, reputation deltas, decay rates, threshold formula constants, sponsor-gate strategy, jury parameters) will accrue evidence from internal dev-team usage between v0 ship and the OQ-011 pilot launch. The dev team is a small, homogeneous sample — values that "feel right" internally may be wrong for a grassroots-organisation pilot. Do we ship a single set of v1 defaults derived from dev-team dogfooding (which collapses dev-team and pilot evidence at first overwrite), or a **set of named profiles** (e.g. `dev-team-2026Q3`, `pilot-1`, `canonical-v1`) so the pilot retro can fork its own profile rather than overwrite a single canonical default?
- **Current lean:** Named profiles. v0's existing `governance_config` table (Phase 5a task 50) is keyed by `(scope, key)`; v1 introduces a parallel `governance_config_profile` row keyed by profile name with a top-level `config.profile.active` pointer. Profile rows shadow individual `governance_config` keys at read time. v0 ships one implicit profile (`v0-defaults`); v1 promotes accumulated dev-team evidence into a named `dev-team-2026Q3` profile; the OQ-011 pilot launch creates `pilot-1` by copy-on-write rather than overwriting `dev-team-2026Q3`. Treat dev-team-tuned defaults as a **starting point**, not the canonical, so the comparison between dev-team and pilot evidence is preserved rather than collapsed at first overwrite. Profile switching is an instance-admin action via the OQ-018 admin config write endpoint when that lands.
- **Candidate dogfood-tunable OQs that feed v1 profile rollout:** OQ-006 (case threshold formula constants — `clamp_max=2.0` ceiling explicitly flagged for pilot-week retro), OQ-013 resolved deltas (`+5` endorsement_strength / `+5` participation_consistency at endorsement creation), OQ-019 (`participation_consistency` event sources — weekly cadence, `+1` per active week / `-2` per dormant 30 days), OQ-020 (sponsor gate-strategy expansion — which v1 strategies the pilot actually needs), OQ-024 resolved liability floor (`zero` v0 default vs `unbounded` vs `honour_price_fraction`), OQ-025 (sponsor grace-window magnitudes — 24/72/168h placeholders), OQ-003 amended (Restoration sub-variants — `Apology | ContentCorrection | CommunityService` only meaningful once jurors actually want to pick them), ADR-007 jury parameters (5→7 jurors, severity thresholds 60%/75%, diversity constraints — the diversity constraints in particular were noted in ADR-007 as needing real data to tune).
- **Out of scope of dogfood resolution:** OQ-002 (rule-set versioning — schema), OQ-010 (signing keys / HSM — security/ops), OQ-018 (admin config write endpoint shape — API design), OQ-026 (status-aware rule extension — schema architecture). These are architectural choices, not tuning-by-experience choices, and the profile concept does not apply to them.
- **Blocks:** Nothing in v0. Becomes relevant once v1 §7.1 governance-mechanics work begins.
- **Target:** Before v1 design phase begins. Earlier resolution is better — every internal usage between now and the pilot is evidence that should be captured against a known profile name rather than thrown away.

### OQ-V1-AD-01 — Server-rendered HTML page framework for admin dashboard

- **Opened:** 2026-04-19
- **Resolved:** 2026-04-20 — **Defer to v1.x.** v1-AD-d ships API-only (curl+jq is operationally sufficient for pilot). v1-AD-e is descoped from the current AD implementation wave; askama vs maud vs native-React decision revisits after pilot operator feedback. PRD §6 stays as reference for future planning but is not a v1-AD sub-phase gate.
- **Owner:** TBD (backend)
- **Question:** v1 admin-dashboard PRD §6 assumes askama (or maud) for server-rendered admin HTML pages. Neither crate is in the workspace `Cargo.lock` today — adoption would add a templating dep + compile-time step to `lemmy_api`. Alternative: ship v1-AD-d API-only and defer HTML pages to v1.x or v2 (to land alongside the React frontend pass).
- **Blocks:** ~~v1-AD-e (askama HTML pages sub-phase)~~ — v1-AD-e descoped per resolution.
- **Target:** ~~Before v1-AD-e plan writes~~ — resolved.

### OQ-V1-AD-02 — SSE implementation: actix-web-lab vs hand-rolled

- **Opened:** 2026-04-19
- **Resolved:** 2026-04-20 — **Hand-roll (option b).** Build `GET /admin/audit/stream` as a chunked-response handler using `async-stream` (already a transitive dep via tokio-stream — no new supply-chain surface). Handler subscribes to Postgres `LISTEN governance_events` and translates each NOTIFY payload into an `event: <entry_kind>\ndata: <json>\n\n` frame. No `actix-web-lab` dep added. v1-AD-d plan must verify `async-stream` transitive availability at task 0 pre-flight.
- **Owner:** TBD (backend)
- **Question:** PRD §4 enumerates `GET /api/v4/governance/admin/audit/stream` as an SSE endpoint. Upstream Lemmy does not use `actix-web-lab` (grep of `Cargo.lock` 2026-04-19 returns no matches). Two paths: (a) add `actix-web-lab` dep for its `Sse` helper; (b) hand-roll ~80 LOC of chunked-response plumbing using `async-stream` (already a transitive dep). The Postgres `LISTEN governance_events` channel already exists (migration `2026-04-20-000000-0000`) — the SSE endpoint's only job is bridging Postgres NOTIFY payloads onto HTTP.
- **Blocks:** v1-AD-d (dashboard + SSE sub-phase).
- **Target:** ~~Before v1-AD-d plan writes~~ — resolved.

### OQ-V1-AD-03 — Dry-run impact computation inside vs outside run_transaction

- **Opened:** 2026-04-19
- **Resolved:** 2026-04-20 — **Option (b).** Compute impact as a read-only Diesel query BEFORE the `run_transaction` opens, using the current persisted config + the proposed value. No SAVEPOINT primitive needed; no `run_transaction` extension. When `dry_run = true` the write phase is skipped entirely (early return with the impact payload). When `dry_run = false` the already-computed impact is included in the 200 response alongside the post-write `config_id`/`governance_log_id`. PRD §4.3 reshaped in the same commit as this resolution.
  - **TOCTOU mitigation (per CR #75):** between the pre-tx impact query and the `run_transaction` commit, another admin could insert a competing `governance_config` row, rendering the impact payload stale. We accept this as **documented residual risk**, not silent deferral. Rationale: (1) admin-config writes are low-concurrency — only actors with the `admin_config_write` capability reach this path, and simultaneous admin writes against the same key are a governance problem, not a technical race; (2) the drift window is the time between the pre-tx read and the tx commit (milliseconds in the common case); (3) the impact payload is advisory — it does not gate the write, and the post-commit `governance_log` row records the actual `previous_value`/`new_value` pair authoritatively; (4) adding optimistic locking (a `config_version` column) or an advisory "another admin just changed this" warning adds UX/schema complexity for a race whose real-world blast radius is "the impact number the admin saw was slightly off." v1-AD-b handler docstring MUST cite this resolution and name the residual risk; v1-AD-d dashboard (when it ships) MAY surface a "config changed since you viewed this" banner, but that is a UX polish, not a correctness fix. Optimistic locking and advisory-warning options remain open for v2 if pilot operators report real drift; until then, the accepted-risk framing is the contract.
- **Owner:** TBD (backend)
- **Question:** PRD §4.3 proposes running dry-run impact queries inside the same `run_transaction` as the config write with a `SAVEPOINT` rollback when `dry_run = true`. Inspection of `crates/diesel_utils/src/connection.rs:68-79` shows `run_transaction` wraps `diesel-async`'s `.transaction()` — no SAVEPOINT primitive is exposed. Either (a) extend `run_transaction` with a `run_savepoint` helper, or (b) compute impact as a read-only query BEFORE the tx opens against the proposed value and the current config snapshot, with no rollback needed.
- **Blocks:** v1-AD-b (POST /admin/config sub-phase).
- **Target:** ~~Before v1-AD-b plan writes~~ — resolved.

### OQ-V1-JM-07 — Post-JM-b general case-open severity-tier inference

- **Opened:** 2026-04-24 (v1-JM-b Task 1, per plan §7.1)
- **Owner:** TBD (backend + domain)
- **Question:** v1-JM-b makes `admin_assign_jury` severity-tier-aware at pick time, but does NOT populate `moderation_case.severity_tier` at case-open time for non-emergency paths. Every non-emergency-remove case is still created with the JM-a-backfilled `severity_tier = 'Minor'` default. What inference path becomes the general case-open writer? Three shapes: (a) hardcoded `reason_code → severity_tier` table in a new `severity_inference.rs` module, called from `create_report` at case-open; (b) per-community policy stored in a new `report.reason_code_severity_map` text/JSON key under `governance_config`, read via the v1-JM-b cascade at case-open; (c) a new reporter-facing DTO field `severity_tier_suggested` on `create_report` with an admin-override at case-open.
- **Current lean:** (a) — hardcoded table in `severity_inference.rs`. Simplest to ship post-JM-b; keeps the v1 PRs small; avoids a new seeded config key and avoids expanding the reporter DTO surface. Per-community overrides via (b) are v1.5 scope once pilot data indicates the hardcoded table is wrong for specific communities. (c) is v2+ (UX surface change, reporter-facing).
- **Blocks:** v1.5 general-severity-inference sub-phase only. Does NOT block v1-JM-c (vote tally), v1-JM-d (appeals), or v1-JM-e (capstone) because those paths only READ `moderation_case.severity_tier` — and the NOT NULL DEFAULT 'Minor' at the column level means every case pre-inference-ship remains Minor, which the JM-b cascade handles correctly.
- **Target:** v1.5 sub-phase. Resolve before any implementer writes the first call-site in `create_report`.

### OQ-ADR016-01 — B-fetch protocol shape and per-app adapter SPI

- **Opened:** 2026-05-23 (ADR-016)
- **Owner:** TBD (backend + federation)
- **Question:** What is the wire shape Brehon's federation fetch worker uses to retrieve evidence from external apps? Sub-questions: (a) is the per-app adapter SPI a trait inside the fetch worker (Rust trait `EvidenceAdapter` with `fetch(uri: &str) -> Result<SnapshotBlob>`), a subprocess plugin (each adapter is its own binary), or a sidecar service (adapters speak HTTP to the fetch worker)? (b) what is the canonical reference URI shape — `matrix://homeserver/!room/$event`, an `actor://app/identifier` pattern, or app-specific URIs the adapter parses? (c) what metadata must every adapter return alongside the content (timestamp, actor-in-app, app-instance-pubkey-fingerprint, snapshot-hash)? (d) how does the reporter-side hash get to Brehon — embedded in the report DTO, or a separate signed claim?
- **Current lean:** (a) Rust trait inside the fetch worker, one impl per supported app. Matches existing federation-fetch-worker pattern in [07 §1.2](07-operations-and-federation.md) and avoids the operational complexity of subprocesses or sidecars. (b) URI shape per-app; adapter parses its own — no need for a universal scheme until a third app proves the need. (c) Minimum metadata: ISO-8601 timestamp, app-actor identifier, app-instance fingerprint (for later peer-list verification), content-hash; everything else is app-specific. (d) Reporter-side hash in the report DTO as an optional `observed_content_hash` field; Brehon flags divergence to the jury rather than rejecting outright.
- **Blocks:** M1 sub-PRD (chat infrastructure) — adapter SPI must be settled before the M1 reference integration writes the first MatrixEvidenceAdapter. Also blocks any threat-model row for B-fetch adapter compromise (06 §7 extension per ADR-016 Enacted-in).
- **Target:** Resolve at M1 schedule time. Lean is the v0 default; revisit if a second adapter (likely PeerTube) exposes a constraint the lean doesn't handle.

### OQ-ADR016-02 — B-publish event schema and subscriber contract

- **Opened:** 2026-05-23 (ADR-016)
- **Owner:** TBD (backend + federation)
- **Question:** What is the schema Brehon publishes sanction events in, and what guarantees does it offer subscribers? Sub-questions: (a) transport — webhook delivery (HTTP POST per subscriber URL), ActivityPub-style outbox (apps pull from Brehon's outbox), or pub/sub broker (Brehon writes to NATS/Redis/etc., apps consume)? (b) event schema — does Brehon emit one universal event with a `sanction_kind` field, or app-specific events the subscriber filters? (c) delivery guarantees — at-least-once (subscribers idempotent), at-most-once (events lost on subscriber outage), or exactly-once with retry queue? (d) how does a subscriber prove to Brehon it's authorised to receive events about a particular user (the user's Brehon-link-flow established consent — what token / signed claim flows where)?
- **Current lean:** (a) Webhook delivery per subscriber URL — lowest operational complexity, matches HTTP-everywhere posture; pub/sub broker deferred until subscriber count justifies it. (b) One universal event schema with `sanction_kind`, `subject_brehon_actor_id`, `effective_from`, `effective_until`, `governance_log_entry_hash`; the subscriber decides how to translate `sanction_kind` into local primitives (OQ-ADR016-04). (c) At-least-once with subscriber-side idempotency on `governance_log_entry_hash`. (d) At link-flow time (OQ-ADR016-03), the app receives a signed claim binding `app_local_id` ↔ `brehon_actor_id`; the app presents this claim when subscribing to events about that user.
- **Blocks:** M1 sub-PRD if M1 includes any sanction propagation (likely deferred to M2 — governance-triggered rooms). Blocks first non-Matrix app integration ADR.
- **Target:** Resolve at M2 schedule time.

### OQ-ADR016-03 — B-actor link-flow UX and mapping integrity

- **Opened:** 2026-05-23 (ADR-016)
- **Owner:** TBD (backend + UX)
- **Question:** How does a user link their app account to a Brehon actor ID, and how is the mapping kept tamper-resistant? Sub-questions: (a) UX flow — OAuth-style "log in to Brehon" redirect from the app, or app-side "paste your Brehon actor ID + sign this challenge" form, or QR-code pairing? (b) mapping storage — does Brehon hold the canonical mapping table (per-Brehon-node `actor_app_link` table), or does each app hold its own slice (Matrix homeserver stores `@alice ↔ brehon-actor-42` privately, Brehon stores the inverse)? (c) signing — is the mapping a single-signed claim (Brehon signs at link), dual-signed (Brehon + app both sign), or unsigned with both sides agreeing to trust the link-flow transport? (d) revocation — can a user unlink unilaterally, does it require app+Brehon consent, and what happens to in-flight cases/sanctions naming the old link?
- **Current lean:** (a) OAuth-style redirect from app to Brehon; user authenticates against Brehon; Brehon redirects back with a one-time signed link-claim the app stores. (b) Both sides store their half of the mapping (Brehon stores `brehon_actor_id ↔ app_local_id`; app stores the inverse). (c) Dual-signed claim — both Brehon and the app countersign, preventing either side from re-pointing unilaterally. (d) User can request unlink; takes effect prospectively (future sanctions go nowhere); historical case attributions retain the linked actor ID for the audit trail per [ADR-008](#adr-008--append-only-signed-governance-log-is-architecturally-mandatory).
- **Blocks:** M1 sub-PRD (user-facing flow design); first non-Matrix app integration ADR.
- **Target:** Resolve before M1 sub-PRD writes the user-facing link flow. UX research may be needed; soft-target v1.5 / v2 timeframe.

### OQ-ADR016-04 — Sanction translation semantics per app

- **Opened:** 2026-05-23 (ADR-016)
- **Owner:** TBD (backend + domain)
- **Question:** When Brehon publishes a sanction of `sanction_kind = "mute_global_7d"` (or similar), how does each app translate that into its local moderation primitives? Sub-questions: (a) is there a universal vocabulary of sanction kinds Brehon emits, and apps document which ones they implement, or does Brehon emit app-agnostic intent ("reduce reach", "prevent posting", "remove voice") that each subscriber maps? (b) what's the minimum primitive set every connected app must implement to be a "Brehon-governed app" (e.g. block-post, mute-voice, hide-content), and what's optional? (c) how are partial-applicability sanctions handled — e.g. "demonetise" makes sense in PeerTube but not in Matrix; does Matrix's subscriber ignore it, log it, or fail-loud? (d) how does Brehon's audit trail reflect what each app actually did with a sanction (the subscriber acknowledges, Brehon logs per-app application status)?
- **Current lean:** (a) Universal vocabulary — fixed enum of sanction kinds in Brehon, with semver-style expansion. (b) Minimum: `prevent_post`, `mute_voice`, `hide_content`, `restrict_reach`. Beyond that, app-specific extensions. (c) Subscribers receive every event and ignore (with logged "not applicable") for sanction kinds outside their primitives — Brehon's audit reflects the partial-applicability. (d) Subscribers POST acknowledgement back to Brehon (`{ event_hash, applied: true|false, reason, applied_at }`); Brehon stores per-event per-app application status as governance_log entries.
- **Blocks:** M2 sub-PRD (governance-triggered rooms emitting sanctions); per-app integration ADR for any second app.
- **Target:** Resolve at M2 schedule time.

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

**2026-04-19** — *99*
OQ-V1-AD-01, OQ-V1-AD-02, OQ-V1-AD-03 opened (under v1-AD-a task 9, commit `a27b86d63`). All three block v1-AD sub-phases (e/d/b respectively). Leans documented; resolution required before dependent plans write.

**2026-04-20** — *99, v1-admin-dashboard PRD*
OQ-V1-AD-01/02/03 resolved per their documented leans. OQ-V1-AD-01: defer HTML pages to v1.x, v1-AD-e descoped from current wave. OQ-V1-AD-02: hand-roll SSE via `async-stream` (no new dep). OQ-V1-AD-03: compute dry-run impact BEFORE `run_transaction` as a read-only query. PRD §4.3 reshaped to reflect OQ-03 resolution — removes the SAVEPOINT sentence, adds the before-tx read-only contract and the dry-run early-return. Unblocks v1-AD-b/c/d planning.

**2026-04-24** — *99*
OQ-V1-JM-07 opened under v1-JM-b Task 1 per plan §7.1. Post-JM-b general case-open severity-tier inference; lean (a) hardcoded `reason_code → severity_tier` table in a new `severity_inference.rs` module. Blocks v1.5 general-severity-inference sub-phase only; does NOT block v1-JM-c/d/e. JM-b ships without resolution because the NOT NULL DEFAULT 'Minor' column value plus the cascade in `admin_assign_jury` is the complete pick-time contract.

**2026-04-25** — *99*
OQ-027 opened (v2-research): Autonomi as governance-log anchor and evidence-storage backend. Four options (anchoring-only / +evidence / +x0x comms / no Autonomi). Lean (b) — storage-layer fit is strong; comms layer (x0x) deferred pending maturity. References ADR-002 (no conflict), ADR-003 (public memory alignment). Blocks nothing in v0/v1; target is v2 security-hardening research spike.

**2026-04-30** — *99*
OQ-028 (named governance-profile bundles for v1 tuning rollout) opened. Captures the dogfood-vs-pilot evidence-preservation problem: dev-team usage between v0 ship and the OQ-011 pilot launch produces tuning evidence for OQ-006 / OQ-013 / OQ-019 / OQ-020 / OQ-024 / OQ-025 / OQ-003 / ADR-007, but the dev team is a homogeneous sample. Named profiles preserve the dev-team-vs-pilot comparison rather than collapsing it at first overwrite. Architectural OQs (OQ-002 / OQ-010 / OQ-018 / OQ-026) explicitly excluded — they are not tuning-by-experience choices.

**2026-05-23** — *99*
ADR-016 added: Brehon is a cross-app governance backplane; federated app planes. Codifies the three-component federation contract (B-fetch evidence retrieval; B-publish sanction events; B-actor portable IDs) and reframes the V2 messaging PRD as the first reference integration (M1/M2/M3 — renamed from V2a/V2b/V2c to end the ADR-010 v2 naming collision). ADR-004 amended (header) to note the extension. Four new OQs opened — OQ-ADR016-01 (B-fetch SPI), OQ-ADR016-02 (B-publish schema), OQ-ADR016-03 (B-actor link UX), OQ-ADR016-04 (sanction translation per app). PRD edit and design-doc updates (03 §4, 06 §2.2/§7, 07 §1.2/§5) deferred to M1 schedule time. Rejected alternatives: per-app signed-evidence (forks every app), Brehon-as-super-admin via app admin APIs (blast radius + trust gate), advisory-only sanctions (undermines cross-app value), OIDC IdP (Brehon as identity SPOF), per-app-identity-no-roll-up (parallel silos).
