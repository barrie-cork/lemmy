# 07 — Operations & Federation

**Audience:** Ops + backend on-call
**Status:** Stable (shape); living (runbooks, cadences)
**Sources:** [chat2.md](chat2.md) §Deployment architecture, §Blockchain discussion, §Federation trust model, §Recovery & reintegration

Deployment topology, blockchain-anchoring operations, backups, federation operator runbooks, recovery flows, and incident response hooks. The architecture and flows themselves live in [03-architecture.md](03-architecture.md); the security controls these operations enforce live in [06-security-and-threat-model.md](06-security-and-threat-model.md).

---

## 1. Deployment topology

### 1.1 Baseline production shape

```
                ┌──────────────────────┐
                │   Load Balancer      │
                │  (rate limit, DDoS)  │
                └─────────┬────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ API Server 1 │  │ API Server 2 │  │ API Server N │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                 │
       └──────────┬──────┴──────────┬──────┘
                  ▼                 ▼
        ┌──────────────┐    ┌──────────────┐
        │ PostgreSQL   │    │ OpenSearch   │
        │ (primary +   │    │ (cluster)    │
        │  replicas)   │    │              │
        └──────┬───────┘    └──────────────┘
               │
               ▼
        ┌──────────────┐
        │ Object Store │
        │ (S3 / MinIO) │
        └──────────────┘

Separate machines / accounts:
  - Keycloak (identity)
  - OPA (policy)
  - Governance log signer (HSM-backed in prod)
  - Federation fetch worker (SSRF isolation)
  - CI/CD with Cosign signing
  - Backup target (offline / immutable)
```

### 1.2 Service inventory

| Service | Purpose | Runs where |
|---------|---------|------------|
| API servers | Core + governance + federation handlers | Container fleet behind LB |
| PostgreSQL | Authoritative data store | Primary + read replicas |
| OpenSearch | Search and derived views | Cluster |
| Object store | Evidence, attachments, audit bundles | S3 / MinIO |
| Keycloak | Identity, MFA, step-up auth | Separate host / managed service |
| OPA | Policy evaluation | Sidecar or separate service |
| Governance log signer | Signs append-only log entries | **Separate host, separate creds, HSM-backed in prod** |
| Federation fetch worker | SSRF-isolated remote content fetching | **No DB access**, network-isolated |
| Cron / background worker | Reputation snapshots, sanction cleanup, jury timeout | Part of API server fleet or separate |
| CI/CD | Build, sign, deploy | GitHub Actions / similar |
| Backup runner | Offline + immutable backups | Dedicated host with one-way credentials |

### 1.3 Environments

- **dev** — single host, docker-compose, no federation peers, stub signer, in-memory OPA
- **staging** — multi-host, one federation peer (another staging instance), real signer with non-production key, real OPA
- **prod** — full topology, HSM-backed signer, real federation peers, offline immutable backups

Staging must peer with at least one other staging instance so federation code paths are exercised before prod.

## 2. Background jobs

Registered in `crates/server/src/governance.rs`. Keep these as discrete, idempotent workers.

| Job | Cadence | Purpose |
|-----|---------|---------|
| Reputation snapshot | Every 15 min or on event insertion | Recompute `reputation_snapshot` from `reputation_event` |
| Reputation decay | Daily | Age-out old positive reputation per decay schedule |
| Expired sanction cleanup | Hourly | Mark `sanction.active = false` where `ends_at < now()` |
| Jury assignment timeout | Every 10 min | Expire `jury_assignment` rows that weren't accepted in time; trigger replacement |
| Governance log integrity check | Daily | Verify hash chain against Postgres state; alert on divergence |
| Backup runner | Daily (full) + hourly (incremental) | Encrypted offline backup push |
| Federation inbox processor | Continuous | Drain inbox queue; signature verify; store or reject |

All jobs must log to the standard observability pipeline; governance-relevant jobs (snapshot, integrity check) also write to the append-only governance log.

## 3. Blockchain as "public memory"

The only place blockchain touches this system. Used **surgically** — not for identity, not for voting, not for reputation.

### 3.1 What gets anchored

Periodic digests of the append-only governance log, plus individual records for:

- Case decisions (hash of the decision + metadata, not the content)
- Rule / constitution changes
- Federation-level sanction recommendations
- Instance-wide policy edits

### 3.2 What does NOT get anchored

- Individual votes (privacy + cost)
- Reputation events (volume + privacy)
- Routine content (posts, comments)
- Identity records
- Evidence files

### 3.3 Anchoring cadence (target — deferred in MVP)

- **Per-event**: decisions on Critical-severity cases anchor immediately
- **Batched**: a Merkle root of all log entries from the past hour is anchored every hour for normal cases
- **Daily digest**: a single root hash summarising a full day is anchored for long-term auditability

For MVP, the governance log runs **locally only** — hash chain is computed and signed but nothing is pushed to a public chain. Public anchoring turns on in v1 after backup and signer operations are proven stable.

### 3.4 Verification

Any external observer can:

1. Fetch the public anchor (chain transaction)
2. Fetch the corresponding log segment (published digest)
3. Recompute the hash
4. Confirm the system has not silently rewritten the record

This is the "truth of record" guarantee. It does **not** guarantee "truth of process" — that's the jury system's job.

### 3.5 Architectural boundary

The chain interaction is **one-way** (out). The main application never reads from the chain to make a decision. If the chain provider disappears, anchoring stops but nothing in the running system breaks.

### 3.6 Explicit non-goals (restated from [01-vision-and-principles.md](01-vision-and-principles.md) §6)

- No on-chain voting
- No governance tokens
- No smart-contract-based rule enforcement (rules live as versioned rows; enforcement is in code + OPA policy)
- No on-chain identity

See [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md) ADR-003.

## 4. Backup strategy

### 4.1 What to back up

- PostgreSQL (all Lemmy tables + all governance tables)
- Object store (evidence files, audit bundles)
- Append-only governance log (authoritative)
- Signing keys and their rotation state
- Keycloak identity store
- OPA policies (these live in version control, but back up the deployed state too)

### 4.2 How

- **Encrypted at rest** with keys held separately from the backup target
- **Offline / immutable** — write-once storage or object-lock enabled; attackers with admin credentials cannot delete backups
- **Tested** — restore drill on a regular cadence (monthly target)
- **Multiple retention tiers** — daily (30d), weekly (90d), monthly (1y)

### 4.3 Restoration drill procedure

1. Provision clean target environment
2. Restore PostgreSQL from most recent backup
3. Restore object store
4. Restore governance log; verify hash chain integrity
5. Replay log against restored PostgreSQL to detect any divergence
6. Spin up API server against restored state
7. Run smoke tests (login, list cases, fetch modlog)
8. Document time-to-recovery and any issues found

### 4.4 Recovery from catastrophic DB loss

If Postgres is a total loss and the governance log survives, the log is authoritative. Case and reputation state can be rebuilt from the log. This is why §2 includes a daily integrity check — if the log and the DB diverge silently for weeks, recovery from the log becomes harder.

## 5. Federation operator runbook

### 5.1 Onboarding a new federation peer

1. **Verify peer identity out-of-band** — confirm the instance operator is who they claim to be, ideally via an existing trust relationship or public key already known to you
2. **Initial trust state: `Limit`** — never start a new peer at `Allow`
3. **Monitor inbound signals for 7 days** — watch for: signature failures, payload anomalies, unexpected object types, rate anomalies
4. **Discuss shared norms** — confirm the peer understands that sanction notices are advisory, not binding
5. **Promote to `Allow` via step-up + quorum + delay** — a trust-state change is a high-risk governance action (see [06-security-and-threat-model.md](06-security-and-threat-model.md) §2.4)
6. **Log the decision** to the append-only governance log with rationale

### 5.2 Receiving a sanction notice from a peer

1. Notice hits federation inbox
2. Signature verified (reject on failure)
3. Payload validated against schema
4. Stored in `remote_sanction_notice` with `local_case_id = null`
5. **Not auto-applied** — surfaced in admin review queue
6. If the notice matches a local case or content, it becomes advisory evidence (`CaseEvidence` with `visibility = PrivateAdmin` or similar)
7. A local jury may cite it when deciding a local case about the same actor, but the notice itself does not execute a sanction

### 5.3 Publishing a local sanction notice outbound

1. Local case reaches `Decided` with a sanction of `FederationQuarantineRecommendation` scope
2. Sanction executor builds a `SanctionNoticeObject`
3. Activity is signed and queued in ActivityPub outbox
4. Delivery worker publishes to all federated peers in `Allow` or `Limit` state
5. Emission logged to append-only governance log
6. **Two-admin co-sign + delay required** — federation-scope sanctions are high-risk governance actions

### 5.4 Downgrading trust on a peer

Trigger conditions:
- Repeated signature failures
- Peer publishes sanctions that another community determines were in bad faith
- Peer's admin compromised (reported to us)
- Anomalous burst of inbound signals

Procedure:
1. Evidence gathered in an internal review case
2. Admin proposes trust downgrade (`Allow` → `Limit`, `Limit` → `Quarantine`, etc.)
3. Step-up auth + quorum + delay (see [06-security-and-threat-model.md](06-security-and-threat-model.md) §2.4)
4. Change logged to governance log
5. Affected peer notified (out-of-band) if appropriate

### 5.5 Lifting a quarantine

Must be explicit — quarantines do not expire automatically. Same change-control procedure as §5.4 in reverse.

### 5.6 Bridge chat-plane soft-pause posture

When the governance admin sets `messaging_enabled = false` in `governance_messaging_config` (via the admin panel or API), the bridge relay enters a **soft-pause** state:

| State | Bridge relay behaviour | User-visible effect |
|---|---|---|
| `messaging_enabled = true` (normal) | Accepts and forwards all DM events; puppet users respond | Chat plane fully operational |
| `messaging_enabled = false` (soft-pause) | Drains in-flight events; stops accepting new inbound Matrix events; puppet users go idle | No new DMs delivered; existing sessions preserved |
| Resuming (`true` again) | Relay resumes without process restart; reads config from Brehon API on next poll cycle | Chat plane resumes; no message loss for events sent during drain |

**Operator notes:**
- Soft-pause does NOT disconnect puppet Matrix accounts or invalidate AS registration.
- Events queued at the homeserver during pause are NOT consumed until resume — they accumulate in the Tuwunel queue and are delivered in order on resume.
- Soft-pause is config-driven (no bridge process restart required). Hard-stop requires `docker compose down`.
- The bridge polls `BREHON_READ_URL` for the messaging config; poll interval is set in bridge startup config. A pause takes effect within one poll cycle of the config change.

## 6. User recovery and reintegration

The system is explicitly restorative (see [01-vision-and-principles.md](01-vision-and-principles.md) §4, principle 5). Every sanction below the top tier has a documented path back.

### 6.1 Reintegration mechanisms

- **Participation** — active, constructive participation restores participation_consistency
- **Jury service** — serving on juries and voting in alignment with final decisions restores jury_reliability
- **Time** — positive reputation decays slowly, but so does negative; the user's standing relative to the decay curve improves with time
- **Good reporting** — accurate reports restore reporting_accuracy

### 6.2 What a user sees

A member with an active sanction sees:

- Which sanction is active (label, restriction, exclusion)
- Why (the cited rule, redacted rationale)
- Duration if time-boxed
- What they can do to restore reputation (capability-granting actions)
- Appeal status if applicable

### 6.3 Permanent exclusion

`InstanceSuspension` is reserved for Critical cases with unanimous or supermajority jury decisions. It is not time-boxed. It is not automatic — it is a specific jury decision, subject to appeal and review.

No sanction is automatically permanent. Any sanction can, in principle, be revisited by a new case citing changed circumstances.

## 7. Incident response hooks

The governance log is the primary tamper-detection mechanism. Hook it into the observability pipeline:

### 7.1 Alerting

- Governance log hash chain diverges from Postgres state → critical alert, pager
- Governance log signer unavailable → critical alert, stop governance writes
- Federation inbound signature failure rate exceeds threshold → warning
- Admin action burst (multiple high-risk actions in short window from one account) → warning, possible auto-freeze
- Backup job failure → critical alert
- Backup restoration drill failure → critical alert

### 7.2 Incident-response playbook (contents)

A prepared playbook (not in this doc; lives in a separate ops runbook) must cover:

- Pager rota
- Step-up auth bypass procedure for emergencies (must itself require multi-admin sign-off)
- Freeze procedure (stop all governance writes; contentious cases pause)
- Communication templates (in-platform, out-of-platform)
- Evidence-preservation procedure
- Post-incident review template

### 7.3 "Admin session compromised during a contentious case" — the priority drill

This is the highest-risk scenario from [06-security-and-threat-model.md](06-security-and-threat-model.md) §4.9. The response must be rehearsed. The drill should exercise:

1. Detection: anomaly alert fires
2. Freeze: all governance writes stop
3. Verification: governance log integrity check; identify any writes from the compromised session
4. Rollback: use the log to identify what needs reversing; apply reversal via emergency quorum
5. Recovery: sessions revoked, credentials rotated, keys rotated if necessary
6. Communication: in-platform notice explaining the freeze and the result

## 8. Observability baseline

- Structured logs per request (correlated across services)
- Metrics: per-endpoint latency, error rate, report volume, case-open rate, jury assignment accept rate, federation inbound/outbound volume
- Traces for cross-service flows (especially governance plane → federation plane → log signer)
- Dashboards: governance health (cases opened/closed/appealed), federation health (per-peer signal rates), security (auth failures, step-up frequency, admin action rate)
- Separate dashboard: **governance log integrity** — divergence count, signer availability, anchoring status (when enabled)

## 9. What's deferred — mapped to v1 / v2 / v3

Post-MVP work is staged across three releases per [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md) ADR-010 and [05-mvp-and-delivery-plan.md §7](05-mvp-and-delivery-plan.md). Operations items land roughly as follows:

### v1 — production-grade governance + self-host packaging

- `deploy/` directory (Docker Compose, `.env.example`, Caddyfile)
- `scripts/bootstrap.sh` and `scripts/backup.sh`
- `INSTALL.md` for self-host ease
- Seed migration with default rule set

### v2 — security hardening

- Keycloak deployment (identity provider)
- OPA or OpenFGA sidecar (policy engine)
- Append-only log signer running on a separate host (HSM-backed in prod)
- Vault / secrets manager with rotation
- Offline immutable backups with tested restore drills on a regular cadence
- Separate SSRF-isolated federation fetch worker
- Cosign / Sigstore signed-build pipeline in CI
- Quorum + delay enforcement on high-risk admin actions

### v3 — verifiability and polish

- Public blockchain anchoring (Sigstore Rekor first; BTC `OP_RETURN` or Ethereum L2 optional)
- Per-event anchoring for Critical cases; batched root anchoring for normal cases
- Multi-instance backup replication
- Automated federation-peer onboarding workflows
- Admin dashboard UX for the runbooks above
- Cross-region deployment
- Automated incident detection beyond threshold alerting

## 10. Cross-references

- System architecture → [03-architecture.md](03-architecture.md)
- Security controls these operations enforce → [06-security-and-threat-model.md](06-security-and-threat-model.md)
- What ships in v0 (operations scope follows feature scope) → [05-mvp-and-delivery-plan.md](05-mvp-and-delivery-plan.md)
- Open operational questions → [99-decisions-and-open-questions.md](99-decisions-and-open-questions.md)
