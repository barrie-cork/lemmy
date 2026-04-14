# Library & Tech Stack for the Brehon-Law Lemmy Fork

## Executive Summary

This document catalogues every library, service, and infrastructure component recommended to support a Lemmy fork implementing the Brehon Law governance model (honour-price reputation, surety sponsorship, jury trials, restorative sanctions, federated trust agreements). Choices are grounded in what Lemmy's own workspace already uses, what the Rust ecosystem offers today (April 2026), and the security/governance requirements described in the two design documents (platform-selection analysis + `chat1.md` implementation checklist). Each recommendation maps to a specific layer: **Core** (Lemmy workspace crates), **Governance** (new crates), **Federation**, **Data**, **Auth/Security**, and **Infrastructure/Ops**.

***

## 1. Lemmy Workspace — Existing Crates to Extend

Lemmy's workspace is a Cargo multi-crate monorepo. The crates listed below are the attachment points for governance code; no new runtime binary is needed beyond the existing `lemmy_server` entry-point.[^1]

| Crate | Role | Governance touch-points |
|---|---|---|
| `lemmy_db_schema` | Diesel models, schema macro, migrations[^2] | Add new tables: `moderation_case`, `jury_assignment`, `reputation_event`, etc. |
| `lemmy_db_views` | Read-optimised query views[^3][^4] | Add `governance_case`, `jury_queue`, `reputation`, `modlog` view crates |
| `lemmy_db_views_actor` | Actor-specific views[^3] | Add juror-eligibility view |
| `lemmy_db_views_moderator` | Moderator-facing views[^3] | Extend for admin governance dashboard |
| `lemmy_api_common` | Shared request/response types, TypeScript bindings[^5] | Add `governance.rs` with `CreateGovernanceReport`, `SubmitJuryVote`, etc. |
| `lemmy_api` | Handler dispatch[^1] | Route governance endpoints |
| `lemmy_api_crud` | CRUD handlers[^1] | Case, appeal, endorsement, reputation handlers |
| `lemmy_routes` | Actix-web route table[^6] | Mount `/api/v4/governance/*` |
| `lemmy_apub` | ActivityPub object/activity types[^7] | Add `governance/` sub-module for federation signals |
| `lemmy_utils` | Shared utilities, error types[^1] | Reputation-decay helpers, jury-selection RNG |

**Recent upstream momentum to track:** Lemmy 1.0 development (mid-2025) added multi-communities, refactored API structs into view crates, introduced an `extism`-based WASM plugin hook for federation activities, and renamed `actor_id` → `ap_id` columns. The fork branch should rebase on the `0.19.x` or `1.0-beta` tag.[^4][^8]

***

## 2. Database Layer

### 2.1 Primary — PostgreSQL + Diesel (async)

Lemmy already uses **Diesel 2.x** with **diesel-async** for non-blocking queries. This is the correct choice for governance tables too:[^9][^1]

- Diesel's compile-time schema verification catches column-type mismatches before runtime.[^10]
- `diesel-async` provides an `AsyncMigrationHarness` using `tokio::task::block_in_place`, suitable for startup migration runs.[^11]
- In ORM mode, async-Diesel benchmarks roughly 18% faster than SQLx in ORM-mode.[^12]
- Existing pool is `bb8` or `deadpool`; the fork can reuse whichever the Lemmy version targets.[^10]

The four new migrations needed (`governance_core`, `jury_system`, `reputation_and_surety`, `federation_attestations`) should each live in `crates/db_schema/migrations/YYYYMMDDHHMMSS_<name>/`.

### 2.2 Append-Only Governance Log — Hash Chaining in PostgreSQL

The `public_case_log` table should be tamper-evident without a separate service. Use a PostgreSQL trigger-based hash chain:[^13][^14]

- Each inserted row computes `SHA-256(prev_hash || payload_json)` and stores it as `chain_hash`.
- A verification view (`tamper_log_verify`) detects any gap.[^13]
- For the additional external proof layer, emit a periodic Merkle root to the chosen "public memory" blockchain (see §8).

The Rust crate `rs_merkle` (v1.5, stable) is the recommended library for building and verifying inclusion proofs off the log. For full Certificate-Transparency-style proofs consider `merkle-log` or `sigstore-merkle`.[^15][^16][^17]

### 2.3 Event Store for Reputation

Reputation is stored as an **event-sourced append-only stream** per user. Each `reputation_event` row carries `(user_id, dimension, delta, reason_code, occurred_at)`. A periodic `reputation_snapshot` job (scheduled Tokio task) materialises the current four-dimensional score for query performance. The CQRS pattern fits naturally: commands (jury votes, sponsor violations) append events; queries read the snapshot.[^18]

### 2.4 Cache Layer — Valkey / Redis-rs + deadpool-redis

Session tokens, rate-limit counters, jury-selection randomness seeds, and read-through reputation snapshots belong in an in-memory store. **Valkey** (Linux Foundation fork of Redis 7.2.4, 2024) is recommended over Redis due to its open licence and improved I/O multithreading. The Rust integration is identical: `redis` crate + `deadpool-redis`. `actix-session` supports `RedisSessionStore` out of the box.[^19][^20][^21][^22][^23]

***

## 3. Web Framework & API

### 3.1 Actix-Web 4 (existing)

The fork inherits Lemmy's **actix-web 4** HTTP stack. Governance routes are added as a scope-mounted `web::scope("/api/v4/governance")` in `lemmy_routes`. All existing middleware (rate limiting, authentication, CORS) applies automatically.[^24][^6][^1]

### 3.2 Authentication — JWT + Keycloak for Privileged Actions

Lemmy currently issues its own JWT tokens. For **phishing-resistant MFA on privileged governance actions** (jury acceptance, admin sanctioning, appeal finalisation), the recommended pattern is:

- **Keycloak** as the OIDC/OAuth2 identity provider — supports WebAuthn/FIDO2 out-of-the-box, hardware token MFA, and step-up authentication flows.[^25]
- Regular API calls use Lemmy's existing JWT middleware.
- Governance write endpoints (jury vote, high-severity sanctions) require a short-lived Keycloak step-up token (AMR claim `mfa`).
- PASETO v4 (`paseto-auth` crate, Ed25519, Rust) is an alternative to JWT for governance tokens if the team prefers avoiding the JWKS complexity; PASETO tokens cannot be replayed to other services because they are audience-locked.[^26]

### 3.3 Authorisation — OPA or OpenFGA

Two viable options for the **governance plane authorisation** (separating content plane from governance plane):

| Engine | Model | Rust integration | Best for this project |
|---|---|---|---|
| **Open Policy Agent (OPA)** | Rego policy language, sidecar or HTTP[^27][^25] | HTTP call from Actix middleware | When governance rules are complex but relatively static (versioned rule sets) |
| **OpenFGA** | Zanzibar ReBAC, CNCF Incubating[^28][^29] | HTTP SDK; also Postgres-native fork[^30] | When trust-circle / kin-group membership queries are the primary access-check pattern |

**Recommendation:** Use **OpenFGA** (or its Postgres-native variant) because the Brehon model's "kin groups → nested trust circles" and "status levels → capability thresholds" map directly to Zanzibar relationship tuples. OPA is better suited when the policy is declarative-rule-heavy rather than graph-relation-heavy. Run OpenFGA as a Docker sidecar; the governance handlers call its check endpoint before writing jury assignments or federation signals.[^31][^32]

The Keycloak + OPA integration (JWT claims feed OPA as PIP) is also documented and viable if the team already runs Keycloak.[^27][^33]

***

## 4. Federation Layer

### 4.1 activitypub-federation-rust

All ActivityPub serialisation, HTTP-signature verification, inbox routing, and activity send queues are handled by **`activitypub_federation`** (LemmyNet's own crate). This is already a dependency of `lemmy_apub`. The governance fork adds a `lemmy_apub/objects/governance/` and `lemmy_apub/activities/governance/` subtree for:[^34][^35][^36]

- `FederationAttestation` — new AP object type carrying a signed hash of a remote sanction or trust-level change.
- `GovernanceSanction` — AP activity sent to remote instances when a federation-level sanction is imposed.

**Security note:** The crate had a SSRF bypass via `0.0.0.0` in versions < 0.7.0-beta.9 (CVE-2025-25194 follow-up). Pin to ≥ 0.7.0-beta.9 and add `is_unspecified()` to your own fork's `v4_is_invalid()` if backporting.[^37]

### 4.2 Federation Trust — Inter-Instance Protocol

Federation trust tiers (allow / limit / quarantine / block) are managed through `federation_attestation` table rows. When a remote node's trust tier changes, the governance plane emits a signed AP `GovernanceSanction` activity to affected instances. The receiving instance's inbox handler updates its local `remote_sanction_notice` table. This mirrors how Lemmy already federates report resolution but carries governance semantics.[^8]

***

## 5. Security Architecture Libraries

| Concern | Library / Tool | Notes |
|---|---|---|
| Phishing-resistant MFA | Keycloak (WebAuthn/FIDO2)[^25] | Step-up flow on governance writes |
| Token issuance | `jsonwebtoken` or `paseto` crate[^26] | Ed25519 signing, short TTL governance tokens |
| Secrets management | HashiCorp Vault / AWS Secrets Manager + `vaultrs` crate | Rotate DB credentials, AP signing keys |
| Append-only audit log | `pg_tamperlog` extension + hash-chain trigger[^13][^14] | Tamper-evident `public_case_log` |
| Merkle proofs (external) | `rs_merkle` v1.5[^16] or `merkle-log`[^17] | Periodic root emission to blockchain "public memory" |
| TLS everywhere | `rustls` + `actix-web` TLS feature | No OpenSSL dependency |
| DDoS / edge hardening | Cloudflare or Bunny CDN in front of federation endpoints | Rate-limit AP inboxes separately from API |
| SSRF protection | `activitypub_federation` ≥ 0.7.0-beta.9[^37] | Must check `is_unspecified()` |
| Policy enforcement | OpenFGA or OPA[^28][^29] | Governance plane separated from content plane |
| Quorum signing | `ed25519-dalek` + manual threshold signature | High-risk sanctions require N-of-M admin keys |

***

## 6. Plugin System (Governance Hooks)

Lemmy 1.0 introduces an **Extism + WebAssembly** plugin system. This is directly useful for the governance fork:[^38][^39][^40]

- `before_governance_report` hook — allows instance-specific content classifiers to pre-screen reports before they enter the jury queue.
- `after_jury_verdict` hook — triggers notification bots, external logging, or custom escalation workflows.
- `federation_sanction_before` hook — allows local policy to review incoming remote sanction notices before applying them.

Plugins compile to WASM and can be written in any language (Rust, Go, JS, Python). The governance-specific hooks should be added to the same `PluginHookType` enum as the existing `local_post_before_create` hook.[^39][^41][^38]

***

## 7. Observability Stack

| Component | Tool | Integration |
|---|---|---|
| Metrics | `prometheus` crate → Prometheus server → Grafana[^42][^43] | Expose `/metrics` endpoint; track jury queue depth, reputation-event throughput, federation-signal latency |
| Traces | `tracing` + `tracing-actix-web` + `opentelemetry-otlp`[^1] | Already in Lemmy; add spans for jury selection and reputation-decay scheduled tasks |
| Logs | `tracing-subscriber` (JSON format) → Loki or CloudWatch | Governance case lifecycle events at `INFO`; evidence access at `WARN` |
| Alerting | Grafana Alerting | Jury queue > 48 h stale, federation quarantine count spike, hash-chain break detected |

***

## 8. Blockchain "Public Memory" Layer

Per the governance design's constraint: **blockchain is used only as an immutable external witness for case-decision hashes, rule-version hashes, and federation-sanction hashes — not for identity, reputation scores, or voting**.[^44][^45]

### Recommended approach — certificate-transparency-style witness log

1. The PostgreSQL `public_case_log` hash-chain produces a Merkle root every N rows (e.g., every 100 decisions).
2. The `rs_merkle` crate builds the tree; the root is signed by the governance node's Ed25519 key.[^16]
3. The signed root is submitted as a simple `OP_RETURN` transaction to **Bitcoin** or an **Ethereum L2** (e.g., Arbitrum) — both offer cheap `calldata` anchoring and have long-term credibility.
4. Anyone can verify any decision is in the log by fetching the root from the chain and verifying an inclusion proof via `rs_merkle`.

This avoids token-based governance entirely, imposes no on-chain computation, and maintains the Brehon principle of "public known laws → explicit versioned rule sets".[^44]

**Alternative:** Use the **Sigstore** transparency log infrastructure (`sigstore-merkle` crate) for a zero-cost, open, append-only witness without on-chain fees. This is more practical for a community-run instance.[^15]

***

## 9. Media & Object Storage

Evidence attachments (screenshots, files) for `case_evidence` rows should be stored outside PostgreSQL:

| Option | Use case | Trade-offs |
|---|---|---|
| **MinIO** (self-hosted)[^46] | Full control, on-prem, S3-compatible | Operational overhead; good for sovereign-data instances |
| **Cloudflare R2**[^46][^47] | Low egress cost, global CDN, managed | Cloudflare dependency; R2 had a 67-min outage in March 2025[^48] |
| **AWS S3** | Maximum ecosystem compatibility | Egress costs at scale |

Integrate with the existing `pict-rs` proxy already in Lemmy for image thumbnailing. Evidence files should be encrypted at rest with a key managed by Vault (separate from content keys).[^1]

***

## 10. Deployment & Infrastructure

### Container / Orchestration

- **Docker Compose** for development (PostgreSQL, Valkey, OpenFGA, Keycloak, the Lemmy fork binary, pict-rs).
- **Kubernetes** (via Helm) for production. The Lemmy community already provides a Helm chart; add sidecar deployments for OpenFGA and OPA.
- Use `mimalloc` as the memory allocator (already adopted by Lemmy upstream) for lower fragmentation under sustained governance workloads.[^8]

### CI/CD

- `cargo check --workspace` + `cargo clippy` + `cargo test --workspace` in CI.
- `diesel migration run` in the deployment pipeline with a pre-flight diff check (Lemmy upstream tooling).[^8]
- Run `cargo-semver-checks` for any governance crates exposed as public libraries.[^49]

### Database

| Concern | Solution |
|---|---|
| Migrations | `diesel_migrations` embedded in binary; `diesel migration run` on startup[^11] |
| Immutable backups | Daily `pg_dump` to S3/R2, retained offline ≥ 90 days; hash the dump file and store hash in append-only log |
| Connection pooling | `bb8` or `deadpool-postgres` (match upstream Lemmy choice) |
| Timescale (optional) | `TimescaleDB` extension for reputation-event time-series if query performance degrades at scale |

***

## 11. Full Dependency Reference (Cargo additions)

These are **net-new** additions beyond Lemmy's existing `Cargo.toml`; existing upstream dependencies (actix-web, diesel, serde, tokio, uuid, chrono, tracing, activitypub_federation, etc.) are not repeated.

```toml
# Governance audit log / Merkle proofs
rs_merkle = "1.5"
merkle-log = "0.1"           # Certificate-transparency-style log (optional)

# Reputation event sourcing
# No new crate needed — standard Diesel + custom StoredEvent<ReputationEvent> structs

# PASETO tokens for governance step-up (alternative to JWT)
pasetors = "0.8"             # Pure-Rust PASETO v4 local/public

# Ed25519 signing (quorum keys, AP governance activities)
ed25519-dalek = { version = "2", features = ["rand_core"] }

# OpenFGA client (governance plane authz)
openfga-rs = "*"             # community SDK; or use reqwest to call OpenFGA HTTP API

# Cache (Valkey-compatible)
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }
deadpool-redis = "0.16"

# Session store
actix-session = { version = "0.9", features = ["redis-session"] }

# Metrics
prometheus = { version = "0.14", features = ["process"] }
actix-web-prom = "0.8"      # Actix middleware for Prometheus

# Object storage (evidence files)
aws-sdk-s3 = "1"             # Works with MinIO, R2, AWS S3

# WASM plugin runtime (Extism — already in Lemmy 1.0)
extism = "1"

# Hash chain tamper detection helper
sha2 = "0.10"
hex = "0.4"
```

***

## 12. Architecture Decision Summary

| Decision | Choice | Rationale |
|---|---|---|
| Base platform | Lemmy fork (Rust, ActivityPub)[^50][^7] | Existing community, federation, voting infrastructure |
| ORM | Diesel + diesel-async[^1][^9] | Type-safe, compile-time verified; already in use |
| Cache | Valkey + deadpool-redis[^22][^23] | Open-source, Redis-compatible, improved threading |
| Auth (standard) | Lemmy JWT (existing) | Continuity |
| Auth (governance step-up) | Keycloak OIDC + WebAuthn[^25] | Phishing-resistant MFA for high-risk actions |
| Authorisation | OpenFGA (Zanzibar ReBAC)[^28][^29] | Trust-circle / reputation-tier queries are graph-native |
| Audit log | PostgreSQL hash-chain trigger[^13][^14] | No external service; append-only and tamper-evident |
| External witness | Sigstore transparency log or BTC OP_RETURN[^15][^16] | Blockchain as "public memory" only; no token governance |
| Plugin hooks | Extism WASM[^38][^39] | Multi-language governance bots; already in Lemmy 1.0 |
| Metrics | Prometheus + Grafana[^42][^43] | Standard Rust observability; existing Lemmy tooling |
| Object storage | MinIO (self-hosted) or R2 (managed)[^46] | S3-compatible; evidence file encryption separate from content |
| Deployment | Docker Compose (dev) → Kubernetes (prod) | Aligns with existing Lemmy infrastructure guidance |

***

## 13. What to Avoid

- **SurrealDB** as the primary store — durability concerns at benchmark time raised by the community; Lemmy's existing Diesel investment makes migration costly.[^51]
- **Token-based on-chain governance** — the Brehon design explicitly prohibits this; blockchain is witness-only.[^45][^44]
- **Redis (proprietary)** — Valkey or Dragonfly are preferred post-licence change.[^22][^23]
- **OPA alone for trust-circle checks** — Rego is a poor fit for graph traversal; OpenFGA handles relationship tuples more efficiently.[^32]
- **Splitting into a separate microservice too early** — the governance logic belongs in the Lemmy workspace as additional crates; a separate binary is only warranted once jury-queue or reputation-event throughput saturates the main server's resources.

---

## References

1. [Crate lemmy_server](https://docs.rs/lemmy_server/latest/lemmy_server/) - API documentation for the Rust `lemmy_server` crate.

2. [lemmy/crates/db_schema/src/schema.rs at main · LemmyNet/lemmy](https://github.com/LemmyNet/lemmy/blob/main/crates/db_schema/src/schema.rs) - 🐀 A link aggregator and forum for the fediverse. Contribute to LemmyNet/lemmy development by creatin...

3. [Crate lemmy_api_common](https://docs.rs/lemmy_api_common/0.18.2/lemmy_api_common/) - API documentation for the Rust `lemmy_api_common` crate.

4. [Lemmy Development Update 2025-02-07 - Lemmy.World](https://lemmy.world/post/25262717) - Here is our regular update that explains what we have been working on for the past two weeks. This s...

5. [lemmy_api_common — Rust auth library // Lib.rs](https://lib.rs/crates/lemmy_api_common) - A link aggregator for the fediverse

6. [lemmy_routes 0.18.2](https://docs.rs/crate/lemmy_routes/latest)

7. [Lemmy Documentation](https://bbs.institute/docs/en/federation/overview.html)

8. [Lemmy Development Update June 2025 - Lemmy.World](https://lemmy.world/post/32313451) - This was a busy month, with ~80 pull requests merged [https://github.com/LemmyNet/lemmy/pulse/monthl...

9. [Diesel-async — db interface for Rust // Lib.rs](https://lib.rs/crates/diesel-async) - Diesel-async provides an async implementation of diesels connection implementation and any method th...

10. [Choosing a Rust Database Crate in 2023: Diesel, SQLx, or Tokio ...](https://rust-trends.com/posts/database-crates-diesel-sqlx-tokio-postgress/) - Explore the pros and cons of Diesel, SQLx, and Tokio-Postgres in Rust development. This comprehensiv...

11. [AsyncMigrationHarness in diesel_async - Rust - Docs.rs](https://docs.rs/diesel-async/latest/diesel_async/struct.AsyncMigrationHarness.html) - A diesel-migration `MigrationHarness` to run migrations via an `AsyncConnection`

12. [Diesel vs Sqlx , my benchmark - Rust Users Forum](https://users.rust-lang.org/t/diesel-vs-sqlx-my-benchmark/111982) - Hello, For high-performance project requirements, I decided to do my own performance test between Sq...

13. [dmtkfs/pg-tamper-log: A tamper-evident audit log table for ...](https://github.com/dmtkfs/pg-tamper-log) - Tamper-evident, append-only audit logging for PostgreSQL. pg_tamperlog is a PostgreSQL extension des...

14. [Tamper-evident audit trails in PostgreSQL with hash chaining](https://appmaster.io/blog/tamper-evident-audit-trails-postgresql) - Learn tamper-evident audit trails in PostgreSQL using append-only tables and hash chaining so edits ...

15. [sigstore-merkle - Lib.rs](https://lib.rs/crates/sigstore-merkle) - RFC 6962 Merkle tree verification for Sigstore

16. [rs_merkle](https://lib.rs/crates/rs_merkle) - The most advanced Merkle Tree library for Rust. Supports creating and verifying proofs, multi-proofs...

17. [merkle_log - Rust](https://docs.rs/merkle-log/latest/merkle_log/) - An implementation of the “Merkle Tree-Structured Log” defined in the blog post Transparent Logs for ...

18. [How to Build Event-Sourced Apps with CQRS in Rust - OneUptime](https://oneuptime.com/blog/post/2026-01-25-event-sourcing-cqrs-rust/view) - A practical guide to implementing event sourcing and CQRS patterns in Rust, with working code exampl...

19. [How to Use Redis with Actix-Web in Rust](https://oneuptime.com/blog/post/2026-03-31-redis-actix-web-rust/view) - Learn how to integrate Redis with Actix-Web in Rust for async caching, session storage, and pub/sub ...

20. [Redis session store with Rust | Docs](https://redis.io/docs/latest/develop/use-cases/session-store/rust/) - Overview. Session storage is a common Redis use case for web applications. Instead of keeping sessio...

21. [RedisSessionStore in actix_session::storage - Rust](https://docs.rs/actix-session/latest/actix_session/storage/struct.RedisSessionStore.html) - Use Redis as session storage backend.

22. [5 Awesome Redis Alternatives you need to know in 2025 - Dev.to](https://dev.to/code42cate/5-awesome-redis-alternatives-you-need-to-know-in-2025-2k0m) - Welcome to the world of key-value databases, where speed is king and drama is... well, unexpectedly....

23. [Redis vs Valkey vs Dragonfly 2026: Full Comparison](https://devtoolswatch.com/en/redis-vs-valkey-vs-dragonfly-2026) - Redis vs Valkey vs Dragonfly in 2026: benchmarks, architecture, licensing, features. A data-driven g...

24. [Rust - JWT Authentication with Actix Web 2026](https://codevoweb.com/rust-jwt-authentication-with-actix-web/) - In this article, we will delve into the implementation of JWT authentication in Rust, covering all c...

25. [Integrating Keycloak with Open Policy Agent for Agile ...](https://hoop.dev/blog/integrating-keycloak-with-open-policy-agent-for-agile-authorization) - The login screen loads. The user enters their credentials. Behind the scenes, a silent pact between ...

26. [alptekinbodur/paseto-auth: Getting started with REST API ...](https://github.com/alptekinbodur/paseto-auth) - A lightweight, secure, and reusable Rust library for creating and verifying PASETO v4.public tokens ...

27. [Keycloak and OPA integration](https://forum.keycloak.org/t/keycloak-and-opa-integration/44) - I am interested in using Keycloak in a cloud native solution. In particular to create users for my a...

28. [OpenFGA](https://github.com/openfga) - OpenFGA is a flexible Authorization system inspired by Google's Zanzibar, designed for reliability a...

29. [OpenFGA: Fine-Grained Authorization](https://openfga.dev) - OpenFGA is an open-source authorization solution that allows developers to build granular access con...

30. [I built an OpenFGA compatible authorization system that ...](https://www.reddit.com/r/golang/comments/1qepq3o/i_built_an_openfga_compatible_authorization/) - The core idea is the same as pgFGA: keep authorization inside Postgres instead of running a separate...

31. [OpenFGA Tutorial: Fine-Grained Authorization with Zanzibar](https://github.com/isurucuma/fga-tutorial) - OpenFGA is an open-source authorization engine that implements Google's Zanzibar model for fine-grai...

32. [Migrating Legacy Access Control to OpenFGA - jguer.space](https://jguer.space/blog/migrating-legacy-access-control-to-openfga) - A practical guide to migrating legacy access control systems to OpenFGA, covering key strategies, ch...

33. [Integrating Keycloak and Open Policy Agent (OPA) with Confluent](https://teamraft.com/resources/insights/integrating-keycloak-and-opa-with-confluent/) - In this article, we will go over how to utilize Keycloak for OAuth2 authentication and Open Policy A...

34. [LemmyNet/activitypub-federation-rust - GitHub](https://github.com/LemmyNet/activitypub-federation-rust) - A high-level framework for ActivityPub federation in Rust. The goal is to encapsulate all basic func...

35. [Presenting Activitypub-Rust crate - Reddit](https://www.reddit.com/r/rust/comments/vnc7f1/presenting_activitypubrust_crate/) - It provides a client to server API for creating, updating and deleting content, as well as a federat...

36. [activitypub_federation - Rust](https://docs.rs/activitypub_federation/) - A high-level framework for ActivityPub federation in Rust. The goal is to encapsulate all basic func...

37. [Activitypub-Federation has SSRF via 0.0.0.0 bypass in ...](https://github.com/advisories/GHSA-q537-8fr5-cw35) - An unauthenticated attacker controlling a remote domain can point it to 0.0.0.0, bypass the SSRF pro...

38. [rfcs/0008-plugins.md at main · LemmyNet/rfcs](https://github.com/LemmyNet/rfcs/blob/main/0008-plugins.md) - Requests for comment for changes to Lemmy. Contribute to LemmyNet/rfcs development by creating an ac...

39. [08-plugins.html](https://join-lemmy.org/docs/contributors/08-plugins.html)

40. [Proof of concept for Plugin system (fixes #3562) by Nutomic · Pull Request #4695 · LemmyNet/lemmy](https://github.com/LemmyNet/lemmy/pull/4695) - This PR adds a basic plugin hook using Extism (webassembly), including a simple example plugin writt...

41. [Extism: a WASM-Powered Plugin System | Chris Griffing](https://www.youtube.com/watch?v=Kpc5rbF_9ug) - How awesome would it be if anyone could write plugins in any languages that interop with your applic...

42. [Monitoring Rust web application with Prometheus and Grafana](https://dev.to/rkudryashov/monitoring-rust-web-application-with-prometheus-and-grafana-4i9f) - Overview In this article, I’ll show you how to set up monitoring of a Rust web...

43. [How to Add Custom Metrics to Rust Applications with Prometheus](https://oneuptime.com/blog/post/2026-01-07-rust-prometheus-custom-metrics/view) - Learn how to add custom metrics to Rust applications using the prometheus crate. This guide covers c...

44. [Accountability protocols? On-chain dynamics in blockchain ...](https://policyreview.info/articles/analysis/chain-dynamics-blockchain-governance) - This paper focuses on the dynamics of accountability in blockchain governance. Drawing on a case stu...

45. [Blockchain and digital governance: Decentralization of ...](https://onlinelibrary.wiley.com/doi/10.1111/ropr.12585) - This paper identifies, debates, and draws on what benefits and risks public policy can yield from th...

46. [MinIO vs Cloudflare R2: Best S3-Compatible Storage](https://startupik.com/minio-vs-cloudflare-r2-best-s3-compatible-storage/) - This article provides a startup-focused, neutral comparison of MinIO and Cloudflare R2: architecture...

47. [Cloudflare R2 - Egress-Free Object Storage](https://www.cloudflare.com/product/r2) - Store application data without egregious egress fees. Download your data from R2 without worrying th...

48. [Cloudflare incident on March 21, 2025](https://blog.cloudflare.com/cloudflare-incident-march-21-2025/) - Multiple Cloudflare services, including R2 object storage, experienced an elevated rate of errors fo...

49. [Project goals update — July 2025 - Rust Blog](https://blog.rust-lang.org/2025/08/05/july-project-goals-update/) - The Rust Project is currently working towards a slate of 40 project goals, with 3 of them designated...

50. [lemmy_server — Rust utility // Lib.rs](https://lib.rs/crates/lemmy_server) - A link aggregator for the fediverse

51. [SurrealDB is sacrificing data durability to make benchmarks look better](https://www.reddit.com/r/rust/comments/1my7xen/surrealdb_is_sacrificing_data_durability_to_make/) - If you are a SurrealDB user running any SurrealDB instance backed by the RocksDB or SurrealKV storag...

