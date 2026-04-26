# Feature Specification

## 1. Feature map for the V3 system

This document describes the intended feature set of Brehon Consensus as a mature V3 system.

The features are grouped by governance function rather than by engineering module.

## 2. Membership and onboarding

### 2.1 Entry paths

Users may enter the system through either:

- sponsorship-backed entry, or
- time-based provisional entry followed by gradual capability unlocking.

### 2.2 Sponsorship

The normal Brehon-style route is sponsorship.

Key properties:

- existing members vouch for a new member,
- sponsorship capacity is limited,
- sponsors take reputation risk,
- repeated bad sponsorship reduces or removes the sponsor's ability to vouch for others.

### 2.3 Membership states

The mature system distinguishes practical user states such as:

- visitor,
- provisional user,
- member,
- trusted member,
- juror-eligible member.

These are states, not castes. A person may move upward or downward depending on conduct, age in system, and active sanctions.

## 3. Reputation and standing

### 3.1 Multi-dimensional model

The system uses four reputation dimensions:

- reporting accuracy,
- jury reliability,
- participation consistency,
- endorsement strength.

### 3.2 Capabilities rather than public scores

Users are primarily affected through capabilities and trust signals such as:

- can report with weight,
- can sponsor,
- can be selected for jury service,
- trusted reporter,
- trusted juror,
- under sanction.

The system avoids a single visible number because that would encourage status gaming and flatten distinct forms of trust into one metric.

### 3.3 Decay and reintegration

Reputation is not static.

- positive standing decays over time so elites do not become permanent,
- negative standing decays more slowly or requires corrective conduct,
- the system is designed to permit recovery short of the most severe cases.

## 4. Reporting and case formation

### 4.1 Reporting

Members can report content, persons, communities, or certain remote targets.

Reports are not counted equally. Their weight depends on trust-related criteria, especially reporting accuracy and anti-brigading controls.

### 4.2 Case opening

When reports reach the relevant threshold, a governance case is opened.

A case is the core unit of formal governance work. It holds:

- the target,
- the relevant rule or rules,
- the severity,
- evidence,
- jury workflow,
- decision,
- sanction,
- appeal status,
- and public log record.

## 5. Evidence and visibility

Evidence can include:

- screenshots,
- links,
- uploaded files,
- contextual notes,
- remote governance signals.

Evidence is visibility-classified so the system can distinguish:

- jury-only material,
- private administrative material,
- publicly redacted material.

## 6. Jury system

### 6.1 Selection

By V3, the intended jury model includes:

- seven-person juries as the normal target for serious cases,
- weighted random selection from an eligible pool,
- diversity constraints to reduce cluster capture,
- conflict screening,
- limits on simultaneous assignments,
- distinct appeal juries larger than the original where appropriate.

### 6.2 Thresholds

The mature design uses thresholds based on severity, such as:

- simple majority for lower-severity matters,
- 60% for moderate matters,
- 75% for severe or highly consequential matters.

### 6.3 Deliberation and voting

Jurors review evidence, vote, and may provide rationales. The system aggregates the result into a formal decision and records the outcome.

## 7. Sanctions

### 7.1 Graduated sanction ladder

The mature sanction ladder includes:

- no action,
- advisory label,
- visibility reduction,
- temporary restriction,
- content removal,
- community exclusion,
- instance suspension,
- federation-level quarantine recommendation.

### 7.2 Restorative measures

The mature system is also expected to model restorative responses more explicitly, including:

- apology requirements,
- content correction,
- structured restoration steps,
- community-service-like obligations,
- timed re-entry paths.

### 7.3 Scope

Sanctions may be scoped to:

- a community,
- a whole instance,
- or a federated recommendation communicated to peers.

## 8. Appeals

The system guarantees at least one meaningful appeal path.

Key features:

- appeal window after decision,
- new jury rather than simple reconsideration by the original panel,
- potential larger panel,
- consequences for original jurors if the appeal strongly overturns the first decision,
- public indication of appeal status.

## 9. Public transparency

For significant cases, the public-facing record includes:

- the rule cited,
- the decision,
- the sanction,
- a redacted rationale,
- and the appeal status.

This is a central legitimacy feature, not a minor logging convenience.

## 10. Rule sets and constitutions

Each community may maintain:

- explicit rules,
- versioned constitutions or policy texts,
- local procedural settings,
- local sanction interpretations,
- local federation preferences within platform-wide guardrails.

This allows a common platform to host communities with different legal-political cultures while preserving some shared safety invariants.

## 11. Administrative powers

The mature system does not abolish administrative power completely. Instead, it narrows, hardens, and audits it.

Administrative functions include:

- emergency removal of clearly illegal or urgent-harm material,
- trust-state changes toward other instances,
- config and policy administration,
- exceptional backstop actions,
- operational oversight.

But high-risk actions are subject to stronger controls, and many ordinary decisions are intended to flow through the jury process instead.

## 12. Federation features

By V3, the intended federation feature set includes:

- outbound publication of governance signals,
- inbound reception and storage of governance signals,
- trust states per remote instance,
- admin review interfaces for remote signals,
- fuller inbound workflows beyond simple storage,
- moderation labels and trust attestations across instance boundaries,
- runbook and dashboard support for federation operations.

## 13. Public verifiability features

The mature system includes:

- append-only governance log,
- external or segregated signing architecture,
- selective anchoring of important governance record hashes,
- inclusion-proof or verification tooling for outside auditors,
- public-memory support for constitutional and governance legitimacy.

## 14. Expert-review summary

For expert review, the most important substantive questions are:

- whether the procedural design meaningfully reflects Brehon principles without romanticizing them,
- whether the standing/reputation model is normatively sound,
- whether sponsorship and liability are fair and intelligible,
- whether appeals and sanctions are proportionate,
- whether local sovereignty is preserved under federation,
- and whether the public-verifiability model serves legitimacy rather than spectacle.
