# Glossary And Query Map

## 1. Purpose

This document is designed for database ingestion and quick retrieval.

It provides:

- a concise glossary of core terms,
- a map from likely expert questions to the best source document in this suite,
- and a small set of canonical answer anchors.

## 2. Core glossary

### Case

The formal unit of governance work. A case begins when sufficient reporting or another trigger creates a justiciable governance matter, and it proceeds through evidence, review, decision, sanction, and possible appeal.

### Sponsorship / Surety

A trust relationship in which an existing member vouches for a newer participant and accepts some reputational exposure if that participant later behaves badly.

### Reputation

A multi-dimensional record of standing built from conduct over time. In this system it is not a single score.

### Reporting accuracy

A dimension of standing related to whether a person's reports tend to correspond to genuine governance concerns rather than false or bad-faith accusations.

### Jury reliability

A dimension of standing related to whether a person's jury participation tends to align with durable, upheld outcomes.

### Participation consistency

A dimension of standing related to sustained, constructive community presence.

### Endorsement strength

A dimension of standing related to the quality and consequences of a person's endorsements or sponsorships.

### Capability

A binary or practical permission derived from standing, such as sponsor-eligibility or jury-eligibility.

### Sanction

A formal governance consequence imposed after a decision, ranging from advisory labels to exclusion or federated recommendation.

### Restorative action

A measure intended to repair, correct, or reintegrate rather than simply punish.

### Appeal

A second review path, normally by a different and stronger panel, through which a prior decision may be reconsidered.

### Public case log

The redacted public-facing record of a governance matter.

### Federation

The structured exchange of content and governance signals between self-governing instances.

### Lemmy

The primary public-community and ActivityPub platform substrate on which the governance layer is built.

### Matrix

The primary real-time messaging and coordination substrate used alongside the main governance platform.

### Trust state

The local posture taken toward a remote instance, such as allow, limit, quarantine, or block.

### Governance log

The append-only tamper-evident record of important governance actions.

### Public anchoring

The V3 process of publishing selected governance-record hashes to a blockchain or transparency ledger so outsiders can verify integrity.

## 3. Query map

### If the question is about first principles

Use:

- [01-system-overview.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/01-system-overview.md)

Example questions:

- What is the system fundamentally trying to do?
- In what sense is it Brehon-inspired?
- What does it reject?

### If the question is about planned features

Use:

- [02-feature-specification.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/02-feature-specification.md)

### If the question is about Lemmy and Matrix roles

Use:

- [02-platform-components-lemmy-and-matrix.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/02-platform-components-lemmy-and-matrix.md)

Example questions:

- Why is Lemmy the base platform?
- What role does Matrix play?
- Which system holds the authoritative governance record?
- What belongs in chat and what belongs in the formal case system?

Example questions:

- What sanctions exist?
- How does the jury system work at maturity?
- What features appear by V3?

### If the question is about customization

Use:

- [03-customization-and-rulemaking.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/03-customization-and-rulemaking.md)

Example questions:

- What can a community change?
- What should remain constitutionally fixed?
- How are rule versions handled?

### If the question is about security or abuse

Use:

- [04-security-and-assurance.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/04-security-and-assurance.md)

Example questions:

- How is administrator abuse constrained?
- How are remote attacks handled?
- What happens after compromise?

### If the question is about user experience and procedure

Use:

- [05-user-interaction-and-procedure.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/05-user-interaction-and-procedure.md)

Example questions:

- What does a juror see?
- How does sponsorship feel in practice?
- What does a sanctioned person learn?

### If the question is about federation or blockchain

Use:

- [06-federation-and-public-verifiability.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/06-federation-and-public-verifiability.md)

Example questions:

- Can one instance force another to sanction someone?
- What is blockchain used for?
- What gets anchored and what stays off-chain?

## 4. Canonical short answers

### Q: Is this a blockchain-governance system?

No. It is a federated governance system with a later, narrow blockchain-based public-verifiability layer for selected governance records.

### Q: Is reputation one score?

No. It is multi-dimensional and primarily affects capabilities and trust signals.

### Q: Are moderators abolished?

No. Ordinary serious governance is intended to move toward jury-based procedure, but administrators retain constrained emergency and operational powers.

### Q: Are remote sanctions automatically binding?

No. Remote signals are not self-executing merely because they arrive from another instance.

### Q: Can communities customize rules?

Yes, substantially, but not without limits. The system is pluralist in policy while retaining certain constitutional safeguards.

### Q: Why use blockchain at all?

To provide public evidence that selected important governance records were not silently altered after the fact.

## 5. Suggested database tags

If you build a queryable knowledge base from this suite, useful tags would include:

- `principles`
- `breehon-law-mapping`
- `membership`
- `sponsorship`
- `reputation`
- `jury`
- `sanctions`
- `appeals`
- `customization`
- `rule-versioning`
- `security`
- `admin-powers`
- `privacy`
- `gdpr`
- `federation`
- `trust-states`
- `blockchain`
- `public-verifiability`
- `user-journeys`
- `glossary`

## 6. Ingestion note

For retrieval quality, the best ingestion order is:

1. `00-README.md`
2. `01-system-overview.md`
3. `02-platform-components-lemmy-and-matrix.md`
4. `02-feature-specification.md`
5. `03-customization-and-rulemaking.md`
6. `04-security-and-assurance.md`
7. `05-user-interaction-and-procedure.md`
8. `06-federation-and-public-verifiability.md`
9. `07-glossary-and-query-map.md`
