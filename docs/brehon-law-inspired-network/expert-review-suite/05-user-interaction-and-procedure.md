# User Interaction And Procedure

## 1. Purpose of this document

This document describes how people are intended to experience the Brehon Consensus system in practice.

It focuses on procedure from the user's point of view rather than on backend mechanics.

## 2. Main participant types

The principal participant perspectives are:

- visitor,
- provisional member,
- ordinary member,
- trusted member,
- juror,
- sponsored person,
- sponsor,
- sanctioned person,
- appellant,
- community administrator,
- federation operator,
- public observer.

The main interaction surfaces are also split:

- **Lemmy-facing surfaces** for public communities, posts, case pages, reputation views, and public logs,
- **Matrix-facing surfaces** for real-time notices, private coordination, queue alerts, and optional protected discussion spaces.

## 3. Typical user journeys

### 3.1 A new person joining

The person creates an account and enters as provisional unless a stronger entry path is immediately available.

They may then:

- seek sponsorship,
- wait through a time-based progression window,
- begin participating at a limited level,
- gradually acquire fuller standing.

The interface should make clear:

- what their current state is,
- what they can do now,
- what they cannot yet do,
- and what would move them forward.

### 3.2 A sponsor vouching for someone

A sponsor should see:

- who they are vouching for,
- what the consequences of sponsorship are,
- what liability they may later incur,
- how much sponsorship capacity they currently have,
- whether any cooldown or policy rule prevents the action.

The interaction should feel more like assuming responsibility than casually clicking approval.

### 3.3 A member making a report

The member identifies the target, cites the relevant concern, and submits a report.

The system may later tell them:

- whether the report contributed to a case,
- whether the case was dismissed or upheld,
- whether their reporting standing changed as a result.

The design should discourage weaponized or casual reporting by making process and consequences clear.

### 3.4 A juror handling a case

A selected juror should experience a defined procedure:

1. notification of selection,
2. accept or decline,
3. conflict or ineligibility checks,
4. access to evidence appropriate to the role,
5. clear statement of the rule(s) in issue,
6. decision options,
7. rationale submission where appropriate,
8. indication of whether the case later closed or was appealed.

The juror should not feel like they are improvising moderation from scratch. They should feel they are participating in an actual civic process.

In a Lemmy + Matrix architecture, that typically means:

- the formal case file and voting action live in the governance interface,
- while summonses, reminders, and coordination notices may arrive through Matrix.

### 3.5 A person under sanction

The person should be able to understand:

- what happened,
- what rule was applied,
- what sanction is in force,
- how long it lasts,
- whether they may appeal,
- what restoration or reintegration path exists.

The system should avoid opaque black-box punishment.

### 3.6 A person filing an appeal

The person should be able to:

- see whether the appeal window is open,
- understand what grounds or explanation are expected,
- know whether a new jury will hear the matter,
- track the appeal status.

### 3.7 A public observer reading the case log

A non-party observer should be able to see enough to understand:

- that a case existed,
- what kind of rule was involved,
- what outcome was reached,
- whether the matter was appealed,
- and that the rationale was not fabricated after the fact.

But they should not receive unnecessary private or identifying material.

That public-facing inspection function belongs primarily to the Lemmy-facing governance and modlog surfaces, not to Matrix rooms.

## 4. Procedural design goals

The user experience should communicate the following norms.

### 4.1 Governance is not arbitrary

People should be able to follow the process.

### 4.2 Standing is earned and can change

People should understand that trust is dynamic.

### 4.3 Sponsorship is serious

The system should make social risk visible at the moment of endorsement.

### 4.4 Jury service is civic work

Jurors should feel responsibility, not gamified point-collecting.

### 4.5 Sanctions are bounded and intelligible

The person under sanction should know what is happening and why.

### 4.6 Appeals are real, not decorative

An appeal path should be understandable and meaningful.

## 5. Administrator and operator interactions

### 5.1 Community administrators

Administrators should have interfaces for:

- emergency action,
- case oversight,
- configuration changes,
- federation review,
- audit review,
- rule-set editing,
- exceptional interventions.

But those interfaces should also expose the extra procedural burden attached to those powers.

### 5.2 Federation operators

Federation operators should be able to:

- review inbound signals,
- assess peer trust state,
- inspect signature and source information,
- quarantine or limit peers with due process,
- see why a remote signal did or did not affect local review.

### 5.3 Matrix-supported interaction

Matrix is especially useful for:

- juror summons and reminders,
- sponsor-request conversations,
- administrator escalation channels,
- federation-operator coordination,
- appeal-status notifications,
- time-sensitive governance alerts.

The design principle should be:

- communication may happen in Matrix,
- but authoritative governance state should be written back to the formal system.

### 5.4 Dashboard and review UX

By V3, the mature system is expected to provide stronger UI support for:

- operator dashboards,
- audit visibility,
- federation runbooks,
- notification and queue management,
- onboarding and sponsorship guidance,
- juror workflow,
- public verification tooling.

## 6. Interaction questions the expert may ask

The review database should be able to answer questions such as:

- What does a new user see?
- How does sponsorship work in practice?
- What information is shown to a juror and when?
- How does a sanctioned person learn the grounds of the decision?
- What does the public modlog reveal?
- How are administrators prevented from quietly overriding process?
- How would a community member understand whether the system is fair?

## 7. Practical design interpretation

The intended experience is neither a normal social-media moderation panel nor a courtroom replica.

It is a hybrid civic-governance environment in which:

- ordinary participants can understand the rules,
- serious decisions are procedural,
- administrators retain emergency powers but under constraint,
- and the platform teaches members that trust, standing, and responsibility are social institutions rather than invisible backend scores.
