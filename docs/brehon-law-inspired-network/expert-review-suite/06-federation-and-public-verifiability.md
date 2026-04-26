# Federation And Public Verifiability

## 1. Why this document matters

This document explains the two parts of the system most likely to interest a technically curious expert reviewer:

- federation between communities,
- and the limited V3 role of blockchain or public transparency ledgers.

These are related, but they solve different problems.

## 2. Federation in constitutional terms

Federation means that self-governing instances can communicate governance-relevant signals while preserving local jurisdiction.

The design goal is:

- cooperation without subordination,
- signal-sharing without automatic obedience,
- and common protocol without a central sovereign.

## 3. Governance signals exchanged between instances

The mature system is intended to support signals such as:

- moderation labels,
- trust attestations,
- sanction notices,
- quarantine recommendations,
- related governance metadata.

These signals can help one community understand how another has treated a person, piece of content, or governance question.

## 4. Trust states toward peers

Instances may assign a trust posture toward other instances, such as:

- allow,
- limit,
- quarantine,
- block.

Those postures affect how the local system treats inbound content and governance signals.

## 5. The key safeguard: local judgment remains local

The most important federation safeguard is this:

> a remote sanction notice is not automatically binding simply because it was sent.

Remote signals can:

- be verified,
- be stored,
- be reviewed,
- be considered as evidence,
- influence human judgment.

But they do not, by themselves, execute local sanctions automatically.

This preserves local sovereignty and reduces the danger of compromise contagion across the network.

## 6. Inbound federation beyond MVP

By V3, the system is expected to go beyond merely storing inbound notices. It should support richer review workflows for:

- evaluating remote signals,
- linking them to local cases,
- surfacing them in admin or jury review where appropriate,
- tracking how local policy treated those signals,
- and presenting clearer federation-operational context to operators.

Even at this stage, however, the safeguard against automatic blind enforcement remains important.

## 7. Outbound federation

When a community reaches a serious governance decision, it may publish a structured signal to peers.

In the mature design, this is most justified for matters such as:

- sanction notices of wider relevance,
- trust-relevant attestations,
- quarantine recommendations,
- constitutional or rule-change notices where interoperability matters.

## 8. Why blockchain is deferred and limited

The project deliberately rejects the common blockchain instinct to place the whole governance process on-chain.

Reasons include:

- privacy,
- cost,
- latency,
- legal complexity,
- poor UX,
- and the fact that chain systems do not solve legitimacy, judgment, or context.

Therefore blockchain is introduced only later, in V3, and only as a narrow verification layer.

## 9. What is actually anchored in V3

The mature design contemplates anchoring only selected governance records, such as:

- important case decisions,
- constitutional or rule-set changes,
- federation-level sanction notices,
- major governance-policy changes,
- especially critical events requiring stronger public assurance.

Routine user activity, private evidence, identity data, and ordinary participation are not intended to be put on-chain.

## 10. Function of anchoring

Anchoring serves as a public-memory mechanism.

It provides evidence that:

- a record existed,
- the record had a given cryptographic form,
- and the record was not silently rewritten later.

That is all.

It does not:

- decide cases,
- determine identity,
- replace appeals,
- assign trust,
- or force communities to obey an external chain result.

## 11. Relationship between internal log and public chain

The internal governance log is primary for system operation.

The public chain or transparency ledger is secondary.

The relationship is:

1. the system records a governance event internally,
2. the event contributes to an append-only signed log,
3. selected hashes from that log are later anchored publicly,
4. outside observers can verify inclusion or integrity.

The application itself does not need to consult the chain to know how to govern.

## 12. Why this matters to a Brehon-law-informed review

Federation and public anchoring are where the design most visibly departs from any historical analogue. They therefore require especially careful interpretation.

The defensible reading is:

- Brehon law supplies the social and legal logic,
- federation supplies the pluralist network structure,
- public anchoring supplies durable memory and post hoc verification.

So blockchain is not the law. It is evidence about the record of law-like procedure.

## 13. Questions the database should answer

The review database should be able to answer:

- What kinds of federated signals exist?
- Can a remote instance force a sanction on a local one?
- What do allow, limit, quarantine, and block mean?
- What changes between MVP federation and V3 federation?
- What data is intended to be anchored publicly?
- Why is blockchain deferred?
- How does a public observer verify a case record?
- What remains off-chain and why?

## 14. Summary

Federation is about inter-community communication under retained local sovereignty.

Blockchain, in V3, is about selective public verifiability of important governance records.

Neither is intended to replace local judgment, human review, or constitutional process.
