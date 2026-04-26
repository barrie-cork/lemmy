# Brehon Consensus Expert Review Suite

## Purpose

This document set is a review-oriented specification suite for a Brehon law expert.

It is not written as an implementation plan. It is written as a **target-state reference set** describing what the Brehon Consensus system is intended to look like **after V3**.

The suite is designed so it can be:

- read sequentially by a domain expert,
- queried piecemeal as a reference source,
- ingested into a database or retrieval system, and
- used to answer detailed questions about governance, customization, security, federation, and user experience.

## How to use this suite

If the expert wants:

- the overall idea: start with [01-system-overview.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/01-system-overview.md)
- how Lemmy and Matrix fit into the target architecture: read [02-platform-components-lemmy-and-matrix.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/02-platform-components-lemmy-and-matrix.md)
- the full feature set: read [02-feature-specification.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/02-feature-specification.md)
- customization and local legal-political variation: read [03-customization-and-rulemaking.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/03-customization-and-rulemaking.md)
- the security and anti-abuse model: read [04-security-and-assurance.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/04-security-and-assurance.md)
- how people actually experience the system: read [05-user-interaction-and-procedure.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/05-user-interaction-and-procedure.md)
- federation and blockchain specifics: read [06-federation-and-public-verifiability.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/06-federation-and-public-verifiability.md)
- a glossary and answer map: read [07-glossary-and-query-map.md](/abs/path/c:/Users/barri/Developer/brehon-fork/docs/brehon-law-inspired-network/expert-review-suite/07-glossary-and-query-map.md)

## Target-state assumptions

This suite assumes the project has passed through:

- **v0**: core governance mechanic works,
- **v1**: governance becomes production-grade,
- **v2**: security model becomes operationally robust,
- **v3**: public verifiability, polished federation UX, and limited blockchain anchoring are added.

Where the underlying project docs still describe an MVP-only simplification, this suite states the intended **V3 position** and notes the design direction where useful.

## Source basis

This suite is derived from the main project docs in `docs/brehon-law-inspired-network/`, especially:

- `01-vision-and-principles.md`
- `02-domain-model.md`
- `03-architecture.md`
- `04-data-model-and-api.md`
- `05-mvp-and-delivery-plan.md`
- `06-security-and-threat-model.md`
- `07-operations-and-federation.md`
- `99-decisions-and-open-questions.md`

## Interpretation rule

If there is any apparent conflict between an MVP simplification and a later-stage target:

- this suite treats **the V3 target-state** as authoritative for expert review,
- while preserving important safeguards already committed in the base docs,
- especially:
  - no token-based governance,
  - no on-chain voting,
  - no single reputation score,
  - no permanent moderator class as the normal mode of governance,
  - no automatic application of remote sanctions merely because they were received from another instance.

## Platform note

For this review suite, the target architecture assumes:

- **Lemmy** remains the core public-community and ActivityPub foundation,
- **Matrix** is treated as a core complementary communications layer for real-time interaction, notifications, and private coordination,
- and the formal governance record still lives in the main governance system rather than in chat.
