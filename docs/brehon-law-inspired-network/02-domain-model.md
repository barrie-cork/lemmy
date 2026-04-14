# 02 — Domain Model

**Audience:** Backend engineers + PM (shared vocabulary)
**Status:** Stable
**Sources:** [chat2.md](chat2.md) §Surety+Reputation specification, §system architecture conceptual; [chat1.md](chat1.md) §2 enums (for terminology alignment)

This document holds **nouns and narratives** — what things are, how they relate, how they move through their lifecycles. Storage, columns, and query shapes live in [04-data-model-and-api.md](04-data-model-and-api.md). Policy *values* (thresholds, decay rates) live in [01-vision-and-principles.md](01-vision-and-principles.md) §5.

---

## 1. Glossary

| Term | Definition |
|------|------------|
| **Case** (`ModerationCase`) | The unit of governance work. Opens when reports cross a threshold, ends when a decision is published and the appeal window closes. |
| **Report** | A user-submitted flag on a post, comment, person, community, or remote URL. Reports contribute weighted "pressure" toward opening a case. |
| **Evidence** (`CaseEvidence`) | Material attached to a case: screenshots, links, context. Has a visibility setting (jury-only, private-admin, public-redacted). |
| **Jury** | A set of jurors assembled for one case. MVP uses 5, target v1 uses 7. Never permanent. |
| **Juror** | A member with an active `JuryAssignment` for a given case. |
| **Jury pool** | The set of users currently eligible for jury selection, per community or instance. |
| **Vote** (`JuryVote`) | One juror's decision on one case, recorded with optional rationale. |
| **Decision** | The aggregated outcome of a jury's votes once quorum is reached. |
| **Sanction** | The concrete action applied after a decision: label, visibility reduction, temporary restriction, content removal, community exclusion, instance suspension, or federation quarantine recommendation. |
| **Appeal** | A second hearing by a new, larger jury. One guaranteed appeal per case. |
| **Public case log** (`PublicCaseLog`) | The redacted, public-facing record of a case: rule cited, decision, rationale, appeal status. |
| **Surety** | A sponsorship relationship: one user vouches for another. Creates liability flow. |
| **Endorsement** | A lighter-weight trust signal between members. Contributes to reputation but does not carry the full sponsor-liability chain. |
| **Reputation event** | An append-only record of something that happened which affects a user's reputation on one dimension. |
| **Reputation snapshot** | A derived view: the current values of all four dimensions for a user at a point in time, plus computed capability flags. |
| **Capability** | A binary ability (`can_jury`, `trusted_reporter`, `can_sponsor`) derived from reputation, not exposed as a numeric score. |
| **Attestation** (`FederationAttestation`) | A signed federation-level claim: "this actor is a trusted reporter on our instance", "this actor is jury-eligible", "this content has been sanctioned here". |
| **Remote sanction notice** | An inbound sanction signal from another instance. Stored as advisory evidence; never auto-applied in MVP. |
| **Community** | A single forum / group. The primary unit of governance. Roughly "the túath". |
| **Instance** | A single deployment of the platform. Hosts one or more communities. |
| **Túath** | Informal synonym for community or instance, kept only in narrative/vision docs. Not a code identifier. |

## 2. Actor types

Not a permanent role system — these are **states a user passes through** based on reputation, age, and sanction status. A user can move forward and backward.

| State | How to enter | What they can do |
|-------|--------------|------------------|
| **Visitor** | Not logged in / no account | Read public content only |
| **Provisional User** | Account created; no sponsors yet OR still in time-based fallback window | Read, low-rate post/comment, cannot report, cannot sponsor |
| **Member** | Either: two sponsors confirmed, OR time-based fallback window elapsed with clean history | Full post/comment/vote, can report (unweighted), cannot sponsor |
| **Trusted Member** | Member + high reporting accuracy + ≥ 30 days active | Can sponsor (subject to endorsement limits), weighted reports |
| **Juror Eligible** | Trusted Member + high jury reliability + ≥ 60 days active + no active sanctions | All of the above + can be drawn for jury duty |

### 2.1 Actor state machine

```
                  ┌──────────┐
                  │ Visitor  │
                  └────┬─────┘
                       │ signup
                       ▼
                 ┌───────────────┐
                 │ Provisional   │
                 │ User          │
                 └───┬───────┬───┘
        2 sponsors   │       │  time fallback + clean history
                     ▼       ▼
                  ┌──────────────┐
                  │    Member    │◄─────┐
                  └───┬──────────┘      │ reintegration
      high reporting │                  │ (sanction lifted)
      accuracy +     │                  │
      30 days active │                  │
                     ▼                  │
              ┌──────────────┐          │
              │   Trusted    │          │
              │   Member     │          │
              └───┬──────────┘          │
                  │                     │
      high jury   │                     │
      reliability │                     │
      + 60 days   │                     │
                  ▼                     │
             ┌──────────────┐           │
             │    Juror     │           │
             │   Eligible   │           │
             └──────────────┘           │
                                        │
              ┌─────────────────────────┴──┐
              │                            │
              │ active sanction drops user │
              │ back to Member or below    │
              │ depending on severity      │
              └────────────────────────────┘
```

Forward transitions are gated by capability thresholds (see [01-vision-and-principles.md](01-vision-and-principles.md) §5.4). Backward transitions happen when a user acquires an active sanction at a given severity or when their reputation falls below the threshold they crossed going up.

## 3. Case lifecycle

A case is the central narrative of the governance layer. This is the end-to-end flow:

```
User files Report
    │
    ▼
[Reputation Weight Applied]
    │
    ▼
[Threshold Check]
    │
    ├── Below threshold ─────► Store report; no case opened
    │
    └── Threshold met
          │
          ▼
     Case state: OPEN
          │
          ▼
     [Temporary label applied to target]
          │
          ▼
     Case state: THRESHOLD_MET
          │
          ▼
     [Jury Selection Engine]
          │
          ▼
     Case state: JURY_SELECTION
          │
          ▼
     [Jurors notified → accept / decline]
          │
          ▼
     Case state: IN_REVIEW
          │
          ▼
     [Evidence review UI]
          │
          ▼
     [Jurors vote]
          │
          ▼
     [Decision aggregation once quorum reached]
          │
          ▼
     Case state: DECIDED
          │
          ▼
     [Sanction executor applies decision]
          │
          ▼
     [Public log entry published (redacted)]
          │
          ▼
     [Reputation events emitted for jurors and reporters]
          │
          ▼
     [Appeal window opens]
          │
          ├── Appeal filed ────► Case state: APPEALED ──► new jury ──► DECIDED again ──► CLOSED
          │
          └── Window expires ──► Case state: CLOSED
```

### 3.1 Case states

Mapped to the `CaseStatus` enum in [04-data-model-and-api.md](04-data-model-and-api.md):

| State | Meaning |
|-------|---------|
| `Open` | A case record exists but threshold has not been met; reports still accumulating |
| `ThresholdMet` | Enough weighted reports have landed; jury selection is queued |
| `JurySelection` | Jury pool being sampled; candidates being notified |
| `InReview` | Jurors accepted; evidence being reviewed; votes accepted |
| `Decided` | Quorum reached, decision aggregated, sanctions applied, log published |
| `Appealed` | Appeal filed within the window; a second jury is being assembled |
| `Closed` | Appeal window expired or appeal resolved; case is final |
| `EmergencyRemove` | Content has been removed by an instance admin under the illegal-content override. A jury is assigned post-facto to review whether the override was used legitimately. The jury cannot un-remove; they evaluate the admin's call. See [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md) ADR-013 and [06-security-and-threat-model.md §2.2.1](06-security-and-threat-model.md). |

### 3.2 Case targets

A case can target any of: a post, a comment, a person, a community, or a remote URL (for federated content). See `CaseTargetType` in [04-data-model-and-api.md](04-data-model-and-api.md) §2.

### 3.3 Case severity

Every case carries a `CaseSeverity` (Low / Medium / High / Critical) which determines:

- Jury size (v1 target; MVP uses fixed 5)
- Voting threshold (majority / 60% / 75%)
- Which sanctions are available to the jury
- Whether the sanction is locally-scoped or can escalate to federation

## 4. Reputation model (conceptual)

### 4.1 Four dimensions

Reputation is **not a single score**. It is four independent dimensions, each tracked per-user-per-community (and optionally per-instance-wide):

| Dimension | Grows when… | Shrinks when… |
|-----------|-------------|---------------|
| **Reporting accuracy** | Reports contribute to a case that ends in sanction | Reports are rejected, dismissed, or found bad-faith |
| **Jury reliability** | Juror's vote aligns with final decision and survives appeal | Juror is overturned on appeal, or declines repeatedly, or is outlier in bad faith |
| **Participation consistency** | Regular, constructive participation in community | Long dormancy, only reactive participation |
| **Endorsement strength** | Sponsees become good members; endorsements are reciprocated | Sponsees get sanctioned; endorsements are revoked |

### 4.2 Event-sourced, with decay

- **Every change is an event** (`ReputationEvent`) — who, which dimension, how much, why, when, expires-when
- **Snapshots are derived** (`ReputationSnapshot`) — recomputed from events, cached
- **Positive reputation decays** at roughly −10% per 90 days (preventing permanent elites)
- **Negative reputation decays slower** or requires a corrective action (participation, jury service, time)
- **Recovery path exists for every sanction below the top tier** — reintegration is the default, not the exception

### 4.3 Capabilities, not scores

A user is **never shown a number**. Users see capability flags:

- `jury_eligible: bool`
- `trusted_reporter: bool`
- `can_sponsor: bool`

And visible trust signals:

- "Trusted Juror"
- "Reliable Reporter"
- "Under Sanction"

These are derived from snapshot values crossing thresholds. The reason is social: scores invite gaming and status hierarchies; capabilities and signals describe *what a person can do right now*, and fade if they stop earning them.

### 4.4 Why this separation matters

Reputation *policy* — the thresholds, decay rates, and capability rules — lives here conceptually and in [01-vision-and-principles.md](01-vision-and-principles.md) §5. Reputation *storage* — the tables, events, snapshots, queries — lives in [04-data-model-and-api.md](04-data-model-and-api.md) §5. The policy should be tuneable without rewriting the storage layer, and the storage should be stable even as policy values evolve.

## 5. Sponsorship & liability flow

```
New user signup
    │
    ▼
[Needs 2 sponsors OR time-fallback]
    │
    ▼
Sponsor A, Sponsor B create Surety records
    │
    ▼
User becomes Member
    │
    ▼
... time passes ...
    │
    ▼
User commits a violation
    │
    ▼
Case opened, jury decides, sanction applied
    │
    ▼
Sanction executor emits reputation events:
    ├── User: large negative on the relevant dimension
    ├── Sponsor A: proportional negative (split)
    └── Sponsor B: proportional negative (split)
    │
    ▼
If sponsor accumulates too many bad-sponsorship events:
    └── Loses `can_sponsor` capability
```

Sponsors may **revoke a Surety early** — this reduces their future exposure but does not erase past events. A revoked Surety also stops contributing to the sponsee's good-standing derivation.

## 6. Sanction ladder

Graduated, reversible, proportional. A jury chooses one sanction from the set available for the case's severity. MVP defaults to the lower tiers; the full ladder is documented here.

| Level | `SanctionAction` | Effect | Who can apply |
|-------|------------------|--------|---------------|
| 0 | — | No action taken | (Case closed with no sanction) |
| 1 | `Label` | Advisory label attached to content or profile | Jury for any severity |
| 2 | `VisibilityReduction` | Reduced reach / downranked in feeds | Jury, Low/Medium severity |
| 3 | `TemporaryRestriction` | Time-boxed limit on posting/commenting/reporting | Jury, Medium/High severity |
| 4 | `ContentRemoval` | Specific post/comment removed from community | Jury, High severity |
| 5 | `CommunityExclusion` | User removed from a specific community | Jury, High severity |
| 6 | `InstanceSuspension` | User suspended instance-wide (local) | Jury, Critical severity |
| 7 | `FederationQuarantineRecommendation` | Signal sent to federated instances suggesting quarantine; remote instances choose to apply, ignore, or themselves quarantine | Jury, Critical severity; instance admin must co-sign |

Each sanction has a `SanctionScope` (`Community`, `Instance`, or `FederatedRecommendation`) and, where applicable, a `starts_at` / `ends_at` window.

## 7. Restorative actions

Alongside the ladder, restorative actions exist as alternatives or adjuncts to punitive sanctions. A jury can impose:

- **Apology requirement** — member must post a visible acknowledgement before the case closes
- **Content correction** — original content is edited, corrected, or annotated rather than removed
- **Community service** — member serves additional jury terms or moderation duty
- **Temporary visibility limits with clear duration and automatic restoration**

These exist to keep members inside the system. They are not stored as separate sanction types in MVP — they are `TemporaryRestriction` or `Label` sanctions with a description. Separate modelling is an [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md) question.

## 8. Federation trust (conceptual)

Each community tracks trust per remote instance:

| Trust state | Meaning |
|-------------|---------|
| **Allow** | Full federation; accept content and signals normally |
| **Limit** | Content accepted but down-ranked; signals flagged for review |
| **Quarantine** | Content accepted but hidden by default; signals stored as advisory-only |
| **Block** | No content, no signals accepted |

Inter-instance signals carried by `FederationAttestation` and `RemoteSanctionNotice`:

- Public moderation labels (advisory)
- Trust attestations ("this actor is jury-eligible here", "this actor is a trusted reporter")
- Sanction notices (informational; **never auto-applied** in MVP)
- Quarantine recommendations

The operational/runbook side of federation lives in [07-operations-and-federation.md](07-operations-and-federation.md).

## 9. Transparency boundary

For every case the following split is enforced:

### Public
- Case ID
- Rule(s) cited
- Outcome (decision + sanction)
- Rationale (redacted to remove private details)
- Appeal status

### Private (jury + authorised admins only)
- Evidence files (unless explicitly published as public-redacted)
- Reporter identities
- Juror identities before decision (revealed to each other after quorum; to public only as aggregate count)
- Private discussion / deliberation

The data model enforces this via `EvidenceVisibility` (`JuryOnly`, `PrivateAdmin`, `PublicRedacted`) and via separation between `ModerationCase` (holds private detail) and `PublicCaseLog` (holds publishable summary).

## 10. Open domain questions

Tracked in full in [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md). Highlights:

- Do restorative actions get their own sanction enum variants or stay as flavoured `Label`/`TemporaryRestriction`?
- Is reputation computed per-community, per-instance, or both? (MVP: per-community with an optional instance-wide roll-up)
- How is a community's "rule set" versioned and how are old cases preserved against the rule-version they were decided under?
- Can a single user be a juror on more than one active case simultaneously, and under what caps?
- UX flows for onboarding, sponsorship request, and juror notification — not in any source chat; needs a design pass.
