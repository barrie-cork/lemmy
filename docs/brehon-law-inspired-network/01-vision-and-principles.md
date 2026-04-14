# 01 — Vision & Principles

**Audience:** PM, all team members
**Status:** Stable
**Sources:** [chat2.md](chat2.md) §Brehon-law mapping, §core principle, §final system summary; [chat1.md](chat1.md) §0 crate plan (for scope framing)

---

## 1. Problem statement

Existing online-community platforms concentrate moderation power in a small number of admins/moderators whose decisions are opaque, reversible only at their discretion, and brittle at scale. Grassroots organisations — activist groups, mutual-aid networks, cooperatives, movement platforms — need a space for discussion and coordination that is:

- **Federated** so no single host controls the whole network
- **Procedurally legitimate** so moderation decisions can be trusted by members and outside observers
- **Resistant to capture** by bad-faith minorities (brigading, sybil attacks) and by would-be elites (cliques, admin overreach)
- **Restorative** rather than punitive, so members can rejoin after mistakes

## 2. Target users

- **Grassroots organisations** running their own community instance (the "túath" — one Lemmy instance = one self-governing community cluster)
- **Members** participating in one or more communities and potentially serving on juries
- **Federation operators** running instances that peer with others under shared-but-overrideable norms
- **External observers** (journalists, researchers, members-of-members) who want to audit moderation outcomes

## 3. Core principle

> **Justice is not enforced by a state, but by social trust, reputation, and mutual obligation.**

This is the philosophical anchor drawn from early Irish Brehon law. There are no police and no prisons in the system; enforcement is a combination of reputation loss, graduated exclusion, and restitution. Identity is anchored in relationships (sponsorship), so bad behaviour ripples through the network rather than living in an anonymous account.

Four non-negotiable anchor rules follow from this:

1. **Trust is granted by people, not the system**
2. **Trust always carries risk** (both for the trusted and for the person extending trust)
3. **Influence must be earned slowly**
4. **Bad actions propagate through relationships**

Everything in the domain model, data model, and delivery plan enforces these.

## 4. The nine Brehon principles mapped to platform mechanics

| # | Brehon principle | Platform mechanic | Why it matters |
|---|------------------|-------------------|----------------|
| 1 | **Honor price** — every person had a "status value" they could lose | **Reputation as stake at risk.** Users accumulate social capital and lose weighted portions when they act in bad faith. High-trust users suffer *bigger* consequences when wrong. | Prevents elite abuse — the more influence you earn, the more you have to lose |
| 2 | **Sureties** — guarantors backed individuals' legal standing | **Sponsorship / vouching system.** New users join via sponsorship. If a user is sanctioned, their sponsors take a proportional reputation hit. | Creates accountability chains; defends against fake-account farms; builds organic trust networks |
| 3 | **Kin groups (fine)** — layered loyalties | **Nested communities / trust circles.** Users belong to a local community (primary trust) and optional federated alliances (secondary). Moderation starts local and only escalates outward when needed. | Prevents global mob dynamics; keeps decisions close to context |
| 4 | **Brehons** — experts who interpret law, not rulers who enforce it | **Weighted juries (not elites).** Jury members are randomly selected from an eligible pool, with experienced jurors carrying *slightly* higher weight — never absolute authority. | Expertise without centralisation; no permanent judicial class |
| 5 | **Restitution over punishment** | **Restorative actions** — apology, content correction, temporary visibility limits, community service (e.g. moderation duty) — applied before bans. | Keeps people inside the system rather than ejecting them |
| 6 | **Public, known laws** | **Explicit, versioned rule sets** per community. Every moderation case cites the rule used and the interpretation applied. | Eliminates the "hidden rules" problem |
| 7 | **Collective enforcement** — no central police | **Soft-enforcement ladder:** label → reduced reach → temporary restriction → community exclusion → federation-level isolation. Mirrors "you're no longer trusted to trade with." | Graduated, proportional, reversible |
| 8 | **Inter-tribal law (túatha)** — tribes had own rules but recognised shared norms | **Federated trust agreements** between instances. Shared standards and mutual recognition of sanctions, with per-community override. | Formalises what Mastodon-style federation leaves ad-hoc |
| 9 | **Status loss as deterrent** | **Visible trust signals** instead of hidden scores: "trusted juror", "reliable reporter", "under sanction". | Social consequences, not hidden algorithms |

## 5. Baseline policy parameters

These are the v0 numbers the team should implement first. They are tuneable; they are not guesses. See [02-domain-model.md](02-domain-model.md) for the conceptual model and [05-mvp-and-delivery-plan.md](05-mvp-and-delivery-plan.md) for when each gets wired up.

### 5.1 Onboarding

- New user must satisfy one of:
  - **Option A (primary): Sponsored entry** — 2 sponsors required; each sponsor must be a Member for at least 30 days and not currently under sanction
  - **Option B (fallback): Time-based entry** — limited-privilege account that gains capabilities over time
- Max active endorsements per sponsor: **5**
- Cooldown between endorsements: **48 hours**
- Sponsor must have clean recent history and sufficient reputation to sponsor

### 5.2 Sponsor liability (when sponsee misbehaves)

| Sponsee violation | Sponsor reputation impact |
|-------------------|---------------------------|
| Minor | −1% |
| Moderate | −5% |
| Severe (ban-level) | −10% to −20% |

Impact is **split across sponsors**. Sponsors may revoke endorsement early to reduce their exposure. Repeated bad sponsorship disables the ability to sponsor further.

### 5.3 Reputation dimensions (four; no single score)

| Dimension | Purpose |
|-----------|---------|
| Reporting accuracy | Trust in flagging |
| Jury reliability | Trust in decisions |
| Participation consistency | Engagement quality |
| Endorsement strength (sponsorship quality) | Trust in vouching |

Reputation is **event-sourced**: every action writes an event, snapshots are recomputed. Positive reputation decays slowly (≈ −10% per 90 days). Negative reputation decays slower or requires corrective action. No permanent elites; no permanent outcasts.

### 5.4 Capability thresholds (abilities, not scores)

| Capability | Requirement |
|------------|-------------|
| Report | Basic membership |
| Weighted report (higher flagging weight) | High reporting accuracy |
| Jury eligibility | High jury reliability + member for ≥ 60 days |
| Sponsor others | High endorsement strength |

### 5.5 Moderation escalation ladder (7 levels)

| Level | Action |
|-------|--------|
| 0 | No action |
| 1 | Label ("under review") |
| 2 | Limited visibility |
| 3 | Temporary restriction |
| 4 | Jury case |
| 5 | Community exclusion |
| 6 | Federation signal |

### 5.6 Jury parameters (v1 target — MVP uses simplified set; see [05-mvp-and-delivery-plan.md](05-mvp-and-delivery-plan.md))

- Jury size: **7** (MVP: 5, quorum 3)
- Eligibility: member ≥ 60 days, no active sanctions, sufficient jury reliability + participation
- Selection: random from eligible pool, weighted by reputation, with diversity constraints (no same-sponsor-cluster majority; community diversity where possible)
- Voting thresholds by case severity:

| Case type | Rule |
|-----------|------|
| Minor | Simple majority |
| Moderate | 60% |
| Severe | 75% supermajority |

### 5.7 Appeals

- One guaranteed appeal per case
- Appeal heard by a **new, larger jury**
- Previous jurors are excluded from the appeal panel
- If the appeal overturns the original decision, the original jurors lose reputation on the Jury Reliability dimension

## 6. Non-goals

What this platform explicitly does **not** try to be. These are committed decisions — see [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md) for ADRs.

- **Not a token-based governance system.** No on-chain voting, no "more tokens = more power", no governance tokens. Token-weighted governance is incompatible with the Brehon principle that influence is earned slowly through relationships, not bought.
- **Not a blockchain-of-everything.** Blockchain is used only as "public memory" — append-only hashes of case decisions, rule changes, and federation sanctions for auditability. Identity, reputation, voting, and core logic all stay off-chain. See [07-operations-and-federation.md](07-operations-and-federation.md).
- **Not a replacement for traditional moderator tooling in one pass.** Existing Lemmy moderation pathways remain available as a compatibility layer during transition; jury-based decisions progressively gate severe actions.
- **Not permanent bans by default.** Reintegration paths exist for every sanction below the highest tier. Exclusion is a tool, not a reflex.
- **Not a single reputation score.** Reputation is multi-dimensional and exposed as capabilities, not as a leaderboard number.

## 7. Success criteria

A v1 of this platform is considered successful when:

1. A user can be reported, a case opened, a jury assembled, a decision reached, and a public log published — **end-to-end without direct admin action**.
2. The public moderation log is readable by any member and shows, for every case: rule cited, decision, rationale (redacted), and appeal status.
3. A new account cannot post, report, or sponsor until it has either two sponsors or has aged through the time-based fallback.
4. A sponsor whose sponsee is sanctioned visibly loses reputation.
5. At least one remote instance can federate moderation signals (sanction notice) and the receiving instance can apply, ignore, or quarantine them per local policy.
6. An overturned appeal demonstrably reduces the original jurors' Jury Reliability score.

These criteria are what [05-mvp-and-delivery-plan.md](05-mvp-and-delivery-plan.md) exists to deliver.

## 8. Key tension to watch

> Brehon law worked because **people were embedded in relationships**. Online systems fail because **identity is cheap and relationships are shallow**.

The system must therefore:

- Make identity **costly to abandon** (sponsorship chains, reputation accumulation, visible trust signals)
- Make trust **slow to build** (time gates, dimensioned reputation, decay)
- Make reputation **worth protecting** (capabilities unlock with it, and sponsors share the risk)

Every design decision downstream — data model, jury algorithm, sanction ladder — is a knob on one of these three dials.

## 9. One-line takeaway

> **Trust is not given by the system — it is extended by other people, and risk is shared.**
