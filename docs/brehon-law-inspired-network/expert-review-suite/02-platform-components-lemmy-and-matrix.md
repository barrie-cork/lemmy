# Platform Components: Lemmy And Matrix

## 1. Purpose

This document explains the two software platforms that are intended to be core parts of the mature Brehon Consensus system:

- **Lemmy**
- **Matrix**

They serve different functions and should not be collapsed into one another.

## 2. Why both matter

The target system needs two different kinds of digital space:

- a durable, public, community-governance space,
- and a real-time communications and coordination space.

Lemmy is best suited to the first.
Matrix is best suited to the second.

## 3. Lemmy's role

### 3.1 Lemmy as the public community substrate

Lemmy is the core public platform layer because it already provides:

- communities,
- posts,
- comments,
- voting,
- moderation-adjacent structures,
- ActivityPub federation,
- a strong fit for public deliberative spaces.

For Brehon Consensus, Lemmy is the natural home of:

- public communities,
- content under dispute,
- public-facing case and modlog surfaces,
- governance endpoints,
- federated public governance signals,
- durable community context.

### 3.2 Why Lemmy is a strong fit

Lemmy is well-suited because the governance system needs a visible civic environment, not only a private communications tool.

Lemmy already resembles:

- a forum,
- a public meeting place,
- a federated civic square,
- a durable discussion environment.

That makes it a better constitutional base than a pure chat tool.

### 3.3 What remains authoritative in Lemmy

The following should be treated as authoritative when written into the governance layer on top of Lemmy:

- case creation,
- evidence references,
- jury assignment state,
- jury votes,
- decision outcomes,
- sanction records,
- appeal records,
- public case logs,
- formal rule texts and versions,
- federation trust settings,
- governance log entries.

In short:

Lemmy plus the governance layer is the formal institutional record.

## 4. Matrix's role

### 4.1 Matrix as the real-time coordination substrate

Matrix is a strong fit for the parts of the system that need:

- messaging,
- notifications,
- direct communication,
- group coordination,
- encrypted private channels,
- room-based workflows,
- real-time operational awareness.

For Brehon Consensus, Matrix is a natural home for:

- juror summonses,
- reminders,
- sponsor conversations,
- administrator coordination,
- federation-operator coordination,
- appeal-status notifications,
- urgent governance alerts,
- optional structured discussion rooms around cases where policy permits.

### 4.2 Why Matrix should be core

If the system matures beyond a forum-only workflow, people will need live interaction around governance.

Examples:

- a juror may need a secure notification quickly,
- administrators may need an urgent channel during an incident,
- federation operators may need to coordinate across instances,
- sponsors and new members may need guided real-time conversation,
- communities may want governance announcements delivered instantly.

Matrix supplies that communications layer without forcing the formal governance record to become chat-native.

### 4.3 What Matrix should not become

Matrix should not become the authoritative source for:

- final case state,
- sanction validity,
- appeal validity,
- official rule versions,
- public accountability record,
- long-term governance truth.

Chat can support procedure, but should not replace structured procedure.

## 5. Division of labor between Lemmy and Matrix

The most important architectural rule is a division of labor.

### 5.1 Lemmy/governance layer handles

- durable public discourse,
- public community content,
- case records,
- evidence references,
- sanctions,
- appeals,
- public logs,
- federation objects and policy state,
- formal rule texts,
- auditable governance state.

### 5.2 Matrix handles

- notifications,
- real-time messaging,
- ephemeral coordination,
- private working groups,
- escalation channels,
- operational incident communications,
- optional guided interaction around process steps.

### 5.3 Shared principle

Events may begin in one layer and resolve in the other, but formal outcomes should be written back into the authoritative governance system.

## 6. Example interactions

### 6.1 Juror workflow

- Formal assignment appears in the governance system.
- Matrix sends the summons or reminder.
- The juror reviews the formal case interface.
- The juror votes in the formal governance interface.
- Case status updates may be pushed back through Matrix.

### 6.2 Appeal workflow

- Appeal is filed in the formal system.
- Matrix notifies relevant participants of deadlines or status changes.
- Final appeal outcome is recorded in the formal system.

### 6.3 Federation-operator workflow

- A remote sanction notice arrives through federation.
- It is stored in the formal system.
- Matrix may notify the operator team.
- The operator reviews and acts through the formal governance interface.

### 6.4 Incident workflow

- An emergency governance event occurs.
- Matrix serves as the fast response and coordination layer.
- The formal system records emergency actions, reviews, and outcomes.

## 7. Security implications of the split

This separation is also a security benefit.

### 7.1 Benefits

- chat compromise does not automatically rewrite formal governance state,
- forum compromise does not eliminate the need for messaging-layer controls,
- incident coordination can remain available even when parts of the main app are degraded,
- public legitimacy remains anchored in the formal record, not in screenshots of chat.

### 7.2 Risks to manage

- private discussion drifting into unofficial decision-making,
- sensitive reasoning occurring only in chat and never entering the formal record,
- pressure to treat Matrix messages as if they were valid governance actions,
- blurred lines between notification and adjudication.

For that reason, the system should maintain the rule:

communication may happen in Matrix, but authoritative acts must be executed and recorded in the formal governance system.

## 8. Federation implications

Lemmy and Matrix are federated technologies in different ways.

### 8.1 Lemmy federation

Lemmy handles:

- public-content federation,
- ActivityPub governance signals,
- public inter-instance relationships.

### 8.2 Matrix federation

Matrix handles:

- cross-server messaging and rooms,
- operator coordination across organizational boundaries,
- trusted working-group communication where appropriate.

This means the system can support both:

- public inter-community governance signaling,
- and real-time cross-instance human coordination.

## 9. Expert-review questions

The knowledge base should be able to answer:

- Why is Lemmy the base platform rather than Matrix alone?
- Why is Matrix still treated as core?
- Which system is the formal record?
- Which system handles real-time communications?
- Can Matrix messages create sanctions directly?
- How should juror and operator workflows cross between the two?
- What security and legitimacy risks arise if chat becomes the real decision-maker?

## 10. Bottom line

In the mature design:

- **Lemmy** is the public civic and governance substrate.
- **Matrix** is the real-time communications and coordination substrate.

Both are core parts of the overall system, but they are not interchangeable.
