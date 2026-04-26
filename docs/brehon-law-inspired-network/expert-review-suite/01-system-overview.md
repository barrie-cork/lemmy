# System Overview

## 1. What Brehon Consensus is

Brehon Consensus is a federated online-governance system for communities that want moderation and dispute resolution to operate through visible procedure rather than through the unreviewable discretion of a permanent moderator class.

It is inspired by several structural features associated with Brehon law:

- trust as a social relation rather than a purely administrative grant,
- surety and sponsorship,
- honour-price or status at risk,
- layered belonging and local jurisdiction,
- public procedure,
- restitution before exclusion where possible.

It is not an attempt to reproduce historical Brehon law literally. It is a modern constitutional and procedural system that borrows its deepest social logic.

## 2. Core claim

The system rests on one central proposition:

> online communities become more legitimate when trust is relational, influence is earned, sanctions follow procedure, and important decisions are publicly inspectable.

## 3. What problem it solves

Most online platforms suffer from some combination of:

- opaque moderation,
- arbitrary enforcement,
- weak appeals,
- capture by cliques,
- easy identity abandonment,
- and poor public accountability.

Brehon Consensus is designed to answer those weaknesses with:

- sponsorship-backed entry,
- event-based reputation,
- juries rather than standing rulers for serious cases,
- graduated sanctions,
- explicit local rule sets,
- an append-only governance record,
- and federation between self-governing communities.

## 4. Basic constitutional structure

The system has three major layers.

### 4.1 Governance logic

This is the social and procedural core:

- reporting,
- case formation,
- jury selection,
- voting,
- sanctions,
- appeals,
- sponsorship,
- reputation,
- public logging.

### 4.2 Federation logic

Each instance is a self-governing community or cluster of communities. Instances can exchange governance signals with other instances, but one instance does not rule another.

Federation allows:

- moderation labels,
- trust attestations,
- sanction notices,
- quarantine recommendations,
- selective cooperation between instances.

### 4.3 Public verifiability logic

Important governance records are logged in a tamper-evident way. In V3, selected record hashes are anchored to a public blockchain or transparency ledger so outside observers can verify that records were not silently rewritten.

## 4A. Core platform components

The target-state architecture assumes two core software substrates working together.

### 4A.1 Lemmy

Lemmy is the primary public platform layer.

It provides:

- communities,
- posts and comments,
- voting,
- public moderation surfaces,
- ActivityPub federation,
- the main public-facing community experience.

In constitutional terms, Lemmy is where the public civic life of the community is most visible.

### 4A.2 Matrix

Matrix is the primary real-time communications layer.

It is suited to:

- direct and group messaging,
- juror notifications,
- administrator coordination,
- appeal and case-status alerts,
- private workflow support,
- optional bridge patterns to community communication spaces.

In constitutional terms, Matrix is not the source of formal governance truth. It is a communications substrate surrounding the governance process.

### 4A.3 Rule of separation

The system should treat the two layers differently:

- **Lemmy/governance system**: authoritative home of formal case state, sanctions, rule versions, and public logs.
- **Matrix**: coordination and messaging layer that helps people interact with the process in real time.

This separation matters because chat is useful, but formal legitimacy depends on durable records, structured workflows, and auditable decisions.

## 5. Governing principles

The following principles should be treated as stable review anchors.

### 5.1 Trust is granted by people

The system does not assume that a fresh account deserves the same standing as a long-trusted participant. Standing is gained through social backing, time, and conduct.

### 5.2 Trust carries risk

Sponsors and endorsers do not merely praise others. They expose their own standing when they vouch.

### 5.3 Influence must be earned slowly

Capabilities arise from conduct over time. They are not purchased and they do not become permanent property.

### 5.4 Procedure matters

The legitimacy of an outcome depends not only on whether it seems correct, but on whether it was reached through a fair and inspectable process.

### 5.5 Sanction should be proportional and, where possible, restorative

The system prefers correction, repair, and timed restriction before permanent exclusion.

### 5.6 Local communities remain sovereign

Federation allows communication and coordination, not a supranational sovereign. Remote signals can inform local judgment, but do not automatically displace it.

## 6. The V3 target-state in one paragraph

At V3, Brehon Consensus is intended to be a production-grade federated governance platform in which users join through sponsorship or time-based progression, build multi-dimensional reputation, participate in explicit rule-bound communities, resolve serious disputes through selected juries, publish redacted public case logs, exchange advisory governance signals across instances, operate under a hardened security architecture, and optionally anchor selected governance records to a public blockchain for external verification.

## 7. What the system is not

The following boundaries are deliberate.

- It is not a token-governance system.
- It does not use on-chain voting.
- It does not treat blockchain as the main application substrate.
- It does not rely on a single trust score.
- It does not assume remote instances should be obeyed automatically.
- It does not aim to eliminate all administrator power; instead it constrains, audits, and reviews that power.
