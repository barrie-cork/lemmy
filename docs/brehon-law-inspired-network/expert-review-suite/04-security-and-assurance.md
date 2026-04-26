# Security And Assurance

## 1. Security purpose

The security design of Brehon Consensus is not merely about preventing intrusion. It is about preventing **silent governance capture**.

That means the system is built on the assumption that attackers may eventually compromise:

- a user account,
- an admin account,
- a federated peer,
- a service dependency,
- or even parts of the application environment.

The design goal is to make it difficult for any of those footholds to become an invisible rewrite of governance outcomes.

## 2. Main security principles

### 2.1 Governance must be separately protected

The governance plane is distinct from the ordinary content plane. Governance actions are not treated as just another form submission.

### 2.2 High-risk actions must be hardened

Sensitive actions require stronger controls than normal participation.

Examples:

- step-up authentication,
- phishing-resistant MFA,
- quorum or co-sign requirements,
- execution delays,
- immutable logging,
- anomaly detection.

### 2.3 Logging is a security control

The governance log is a core assurance mechanism. It is not optional observability.

### 2.4 Federation input is hostile by default

Remote instances are treated as potentially wrong, compromised, or malicious. Verification and bounded trust are central.

### 2.5 Privacy must survive transparency

Public legitimacy requires visible process, but the system must not expose private evidence, reporter identity, or direct identifiers without justification.

## 3. Core security architecture

The mature security design includes the following major controls.

### 3.1 Strong identity and privileged-action authentication

- phishing-resistant MFA for privileged actors,
- step-up authentication for especially sensitive actions,
- short session policies,
- stronger identity segregation for admin accounts.

### 3.2 Policy-driven authorization

Authorization is intended to be policy-based rather than improvised in handler code. This allows:

- cleaner review,
- explicit permission models,
- easier legal-policy adjustment,
- stronger auditability of governance powers.

### 3.3 Append-only governance log

Every important governance action is recorded in a tamper-evident way.

This includes:

- case state changes,
- jury assignments,
- jury votes,
- sanctions,
- appeals,
- federation trust events,
- major configuration changes.

### 3.4 Segregated signing and assurance services

The mature design places key assurance functions outside the main app runtime where practical, especially for signing and high-trust functions.

### 3.5 Backup and recoverability

The system assumes that compromise recovery matters as much as compromise prevention. Therefore:

- backups must exist,
- backups must be immutable or difficult to destroy,
- restore drills must be performed,
- the governance record should support reconstruction if the primary database is lost.

## 4. Abuse and capture model

The security model explicitly addresses:

- sybil attacks,
- brigading,
- clique capture,
- sponsor farming,
- jury capture,
- abusive reporting,
- malicious appeals,
- compromised federated peers,
- compromised admins,
- emergency-removal abuse.

This is important for expert review because the system is built around social attack surfaces as much as technical ones.

## 5. Federation-specific security

Federation security includes:

- signature verification,
- schema validation,
- per-peer trust state,
- rate limiting,
- SSRF-resistant media fetching architecture,
- review queues for remote governance signals,
- refusal to auto-apply remote sanctions merely because they were received.

This last rule is especially important. It preserves local legal judgment and limits cross-instance contagion of compromise.

## 6. Emergency powers and legal urgency

The system recognizes that some material may need urgent removal regardless of the normal jury process.

Examples:

- CSAM,
- credible threats,
- urgent legal takedowns,
- severe doxxing.

For those cases, the design includes an emergency removal path, but it is constrained by:

- mandatory post-facto review,
- visible logging,
- meta-governance review if abused,
- heightened visibility to other admins.

This is a deliberate compromise between legal reality and anti-capture governance.

## 7. Privacy and GDPR-compatible transparency

The system attempts to reconcile:

- an append-only governance record,
- public case transparency,
- and deletion/anonymization duties.

The key approach is:

- public and governance records avoid direct identifiers where possible,
- actor identities are pseudonymized in the governance log,
- mapping records can be removed later,
- public rationales are redacted before publication.

This does not erase public accountability for the process, but it aims to reduce unnecessary retention of directly identifying material.

## 8. Public assurance in V3

By V3, assurance extends beyond internal logging.

Selected governance records can be anchored externally so outsiders can verify:

- that a record existed,
- that it had a given form,
- and that it was not silently rewritten later.

This adds public verifiability without moving the core governance process onto chain.

## 9. Security questions the expert may want answered

The database built from this suite should be able to answer:

- How is administrator abuse constrained?
- How are remote-instance attacks contained?
- Can a compromised instance force sanctions onto another?
- What happens if the main database is corrupted?
- How are emergency removals reviewed?
- How are reporter identities protected?
- What logs are public and what remains private?
- How are key governance changes authenticated and approved?
- How does the system balance transparency against legal privacy duties?

## 10. Bottom-line evaluation frame

The security architecture should be reviewed not only as a technical design, but as a constitutional design.

Its main question is:

> can power still be audited, challenged, and reconstructed after compromise?

If the answer is yes, then the platform is behaving consistently with its deeper goals.
