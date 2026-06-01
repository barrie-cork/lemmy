# Matrix Homeserver Selection for Self-Hosted Governance Platform with MatrixRTC

## Executive Summary

**Recommendation: Tuwunel (the official conduwuit successor) as the primary option, with Synapse as the fallback for maximum compatibility.** As of mid-2026, Tuwunel is the only Rust-native homeserver with documented MatrixRTC/MSC4143 support (Element Call discovery via LiveKit), a functional appservice framework, and active enterprise-grade development. Synapse remains the reference implementation with the deepest MatrixRTC integration but carries a Python/PostgreSQL stack you explicitly want to avoid. Dendrite is in maintenance mode and should not be used for new deployments. The original Conduit project is effectively abandoned.

***

## The Landscape in Mid-2026: Key Changes You Need to Know

The homeserver ecosystem has shifted significantly since 2024. The key facts before diving into per-server analysis:

- **conduwuit is archived**: The original conduwuit repository is no longer maintained. It has forked into two successors: **Tuwunel** (the official enterprise-backed successor at [matrix-construct/tuwunel](https://github.com/matrix-construct/tuwunel)) and **Continuwuity** (community-driven, at forgejo.ellis.link).[^1][^2]
- **Dendrite is in maintenance mode**: The element-hq/dendrite GitHub repository was archived on November 25, 2024, with only security fixes being applied. It is listed as "Beta" on matrix.org/ecosystem/servers.[^3][^4][^2]
- **Matrix.org lists Tuwunel as "Stable"**: As of mid-2026, Tuwunel and Synapse (community) are the two production-ready options on the Matrix.org ecosystem page.[^2]
- **MatrixRTC is still marked "experimental" in Synapse config**: MSC4143 requires enabling `msc4143_enabled` in `experimental_features` — the endpoint `/‌_matrix/client/unstable/org.matrix.msc4143/rtc/transports` is available but tracked as unstable spec.[^5][^6]
- **The Ansible Matrix playbook explicitly warns**: MatrixRTC/Element Call "very likely only works with the Synapse homeserver" as of the documented 2025 warning — though this has partially changed with Tuwunel v1.4.6.[^7]

***

## Per-Homeserver Analysis

### Synapse (element-hq/synapse)

**Language/DB:** Python, requires PostgreSQL for production[^8]
**Status:** Stable, reference implementation, v1.150+ as of mid-2026[^6]

#### MatrixRTC Support

Synapse has the most mature MatrixRTC implementation. The `msc4143_enabled` experimental flag activates the LiveKit focus discovery endpoint, and Element's official Matrix Conference 2025 workshop showed deploying a full MatrixRTC stack on Synapse. The full stack requires:[^9]
1. `experimental_features.msc4143_enabled: true` in `homeserver.yaml`
2. A LiveKit Server sidecar
3. An `lk-jwt-service` sidecar for token exchange
4. `.well-known/matrix/client` advertising `org.matrix.msc4143.rtc_foci`[^10][^11]

**Known bugs**: Two open issues as of early-to-mid 2026 show that after enabling `msc4143_enabled`, Synapse v1.149.x–v1.150.x did not correctly advertise the feature in `/_matrix/client/versions` and the `/rtc/transports` endpoint returned 404 in some configurations. These have been tracked but fixes were in-progress. MSC4143 remains in the "unstable features" namespace, meaning the endpoint path will change when the spec stabilises.[^12][^5][^6]

For the town-hall "mic-passing / stage mode" use case, Synapse is the only homeserver that has been publicly demonstrated in production with MatrixRTC stage mode. All Element call-related infrastructure documentation targets Synapse.[^7][^10]

#### Application Service API

Synapse's AS API is battle-tested and is the de facto reference implementation. The mautrix bridge documentation treats Synapse as the primary supported target. A known historical issue (#13142) involved events not being delivered to appservices under certain failure conditions, but AS API reliability in Synapse is generally considered production-quality. Configuration requires a `registration.yaml` file referenced in `homeserver.yaml` under `app_service_config_files`. No restart-free appservice registration exists — a server restart is required to add/remove an AS.[^13][^14][^15][^16]

#### Resource Footprint

For 50–200 concurrent users with active MatrixRTC:
- **Synapse idle**: ~300–600 MB RAM for the Python process; PostgreSQL adds another 200–500 MB baseline[^17]
- **Under load with federation**: 1–3 GB RAM total is typical for a moderately active server[^18][^19]
- **CPU**: Single-worker Synapse is GIL-bound to one core; for 200 concurrent users with RTC, running 2–3 workers (sync, federation, media) is recommended[^20]
- On your 16–32 GB server, RAM is not a bottleneck, but CPU single-threadedness is the key constraint under burst load

#### Failure Modes and Operational Risks

- **State resolution**: Synapse uses "State Resolution v2" (room version 2+), which is the most-tested implementation but is CPU-intensive under large state graphs. The known CVE-2022-39374 involved a federated server tricking Synapse into incorrect state resolution — patched in v1.68+.[^21]
- **Restart recovery**: PostgreSQL ensures ACID-compliant event storage; restart recovery is reliable.
- **Event ordering**: Synapse's ordering is topological (DAG-based) with some known client-visible inconsistencies in edge cases, but this is a Matrix protocol property, not Synapse-specific.[^22][^23]
- **Dropped events to appservices**: Synapse uses a retry queue for AS event delivery; if the AS endpoint is unreachable, Synapse will retry, but very long outages can cause queue overflow.

***

### Tuwunel (matrix-construct/tuwunel)

**Language/DB:** Rust, RocksDB (embedded — no external database required)[^24][^25]
**Status:** Stable (listed on matrix.org), v1.6.x as of mid-2026, enterprise-funded by the Government of Switzerland[^26][^27]

Tuwunel is the **official successor to conduwuit**. Migration from conduwuit is a binary swap with no database changes required. The original conduwuit developer archived that project, and Tuwunel is now maintained by full-time staff with enterprise sponsorship.[^25][^27][^28][^24]

#### MatrixRTC Support

**MSC4143 (Element Call / LiveKit discovery) was implemented in Tuwunel v1.4.6** (released ~November 2025). The implementation exposes:[^27]
- `/_matrix/client/unstable/org.matrix.msc4143/rtc/transports` (authenticated, MSC4143 unstable)
- `/.well-known/matrix/client` fallback (if `[global.well_known].client` is configured)

Configuration uses a TOML section:
```toml
[global.matrix_rtc]
foci = [
  { type = "livekit", livekit_service_url = "https://your-lk-jwt-service.example.com" },
]
```

Continuwuity (the other conduwuit successor) has the same MatrixRTC configuration structure and detailed setup documentation. For Tuwunel, the setup requires the same LiveKit + lk-jwt-service sidecar stack as Synapse.[^29][^27]

**Critical limitation for federation-disabled servers**: When `allow_federation = false`, LiveKit's token exchange fails because lk-jwt-service cannot reach the OpenID endpoint via federation paths. The workaround is to enable federation but deny all remote servers:[^29]
```toml
allow_federation = true
forbidden_remote_server_names = [".*"]
```
This applies to your governance use case if you start with local-only operation.

**Town-hall stage mode assessment**: Tuwunel's MatrixRTC implementation handles LiveKit focus discovery (MSC4143). However, as of mid-2026, no public reports confirm Tuwunel running production-scale MatrixRTC stage mode events. The client-side logic (mic-passing, presenter state) is handled by the custom client and the LiveKit SFU, not the homeserver — the homeserver's role is primarily signalling and membership events. In this respect, Tuwunel should be functionally equivalent to Synapse for the signalling plane.

#### Application Service API

Tuwunel supports the AS API, but with a different registration mechanism than Synapse. Registration is done via an admin room command rather than a config file:[^15][^30]
```
!admin appservices register
<paste registration YAML here>
```

A directory-based approach via `appservice_dir` was added in v1.6.0. Appservice registration does **not** require a server restart in Tuwunel.[^31]

**Known AS API issues (active or recently fixed)**:

1. **Wrong `whoami` response code** (issue #219, reported November 2025): Tuwunel responded to the `/_matrix/client/v3/account/whoami` endpoint with a non-spec response code, causing the mautrix bridge to return `M_NOT_FOUND: User does not exist`. This directly affects any matrix-sdk-appservice bridge doing user identity checks.[^32]

2. **`ip_source` regression breaking local appservices** (issue #465, v1.6.1, May 2026): When `ip_source` is set (e.g., for reverse-proxy deployments), the `SecureClientIp` extractor rejects loopback appservice requests with HTTP 500 before route handlers run, breaking all locally-connected bridges. Regressed in v1.6.1, fix pending in v1.6.2+. This is a critical blocker for any sidecar appservice configuration.[^33]

3. **Appservice bot granted admin privileges** (issue #331, February 2026): When the appservice registers and its bot is the first account created, it is inadvertently granted admin privileges. A workaround exists but requires careful deployment ordering.[^34]

4. **Continuwuity fork**: A related issue (#813) in Continuwuity showed appservice users not being created on registration for some bridge types — verify this is not present in current Tuwunel.[^35]

**Assessment**: The AS API in Tuwunel is functional for popular bridges but has had several active regressions in 2025–2026. For a governance-critical Rust appservice using matrix-sdk-appservice for puppet management and auto-room-creation, these issues require careful version pinning and testing before production deployment.

#### Resource Footprint

Tuwunel/conduwuit lineage servers use RocksDB (embedded) and Rust — no PostgreSQL required. Typical figures from community reports:
- **Idle with small deployment**: 50–200 MB RAM[^36][^17]
- **Under load with 50–200 users**: Estimated 200–500 MB RAM; RocksDB memory maps increase with data volume
- **CPU**: Rust's multi-threaded design means no single-core bottleneck; handles concurrent requests across all cores
- For your 16–32 GB server, Tuwunel leaves far more headroom for LiveKit (which is the RAM-hungry component for RTC) and the Lemmy governance stack

#### Failure Modes and Operational Risks

- **Database lock semantics**: RocksDB uses WAL for crash recovery. Process crashes do not corrupt data, but unsynchronised writes since the last memtable flush can be lost. This is acceptable for Matrix event delivery (at-least-once semantics apply).[^37]
- **Fork fragmentation risk**: Switching between different Conduit-lineage forks corrupts the database. Commit to Tuwunel (or Continuwuity) and do not switch.[^24]
- **Migration regression**: The v1.6.2 `mediaid_user` migration panicked on databases originally created by older versions, requiring immediate downgrade to v1.6.1. Monitor release notes before upgrading in production.[^38]
- **State resolution**: Tuwunel uses Matrix state resolution v2 (room version 11 compliance is a stated goal of the ongoing spec compliance campaign), but has fewer years of adversarial federation testing than Synapse.[^39]
- **Event ordering**: Uses `/sync` order for timeline display, which can differ from topological order in edge cases — acceptable for short-lived governance rooms.[^23]

***

### Dendrite (element-hq/dendrite)

**Language/DB:** Go, PostgreSQL or SQLite
**Status:** Maintenance mode — archived November 2024, security fixes only[^4][^3]

**Do not use for new deployments.** Dendrite was archived by Element as part of a repositioning toward Synapse (community and Pro) and is receiving only security patches. The mautrix bridge documentation explicitly states "Dendrite is not a supported environment, as it often has serious bugs. It is strongly recommended to use Synapse instead." There is no MatrixRTC/MSC4143 implementation in Dendrite.[^15]

***

### Original Conduit (famedly/conduit)

**Status:** Beta/effectively stagnant, v0.10.x[^17]

The original Conduit has not kept pace with the active forks. No MatrixRTC support. Not recommended for any production use case. Tuwunel and Continuwuity have superseded it entirely.

***

### Other Contenders

**Continuwuity** (forgejo.ellis.link/continuwuation/continuwuity): The community-driven conduwuit successor. Shares Tuwunel's database format lineage. Has documented MatrixRTC/LiveKit support with a full setup guide and OIDC next-gen auth planned. Listed as "Stable" on matrix.org. Less enterprise-backed than Tuwunel but more community-driven. Viable alternative if community governance of the homeserver itself is a priority.[^40][^2][^29]

**Construct/Tuwunel**: Note that "Construct" (a C++ homeserver) is listed as "Obsolete" on matrix.org — not relevant.[^2]

**Synapse Pro**: Element's enterprise fork (Rust workers + Python core). Designed for nation-scale deployments. Not the right fit for a solo-operated single server.[^20]

***

## Structured Comparison

| Dimension | Synapse (community) | Tuwunel | Continuwuity | Dendrite |
|---|---|---|---|---|
| **Language / DB** | Python / PostgreSQL | Rust / RocksDB | Rust / RocksDB | Go / PostgreSQL |
| **Status (matrix.org)** | Stable[^2] | Stable[^2] | Stable[^2] | Beta / archived[^3] |
| **MatrixRTC (MSC4143)** | Experimental, production-tested[^5] | Implemented v1.4.6+[^27] | Implemented, documented[^29] | Not implemented |
| **LiveKit focus discovery** | Yes (`msc4143_enabled`)[^11] | Yes (`[global.matrix_rtc]`)[^27] | Yes (`[global.matrix_rtc]`)[^29] | No |
| **AS API reliability** | Production-grade, battle-tested[^15][^16] | Functional, active regressions in 2025–2026[^32][^33] | Functional, known user creation bug[^35] | "Often serious bugs" per mautrix[^15] |
| **AS registration method** | Config file + restart[^16] | Admin room command, no restart[^30] | Admin room command[^15] | Config file |
| **Idle RAM** | ~500 MB–1 GB (+ PostgreSQL)[^17][^18] | ~50–200 MB[^17][^36] | ~50–200 MB | ~200–400 MB (+ PostgreSQL) |
| **RAM under 200-user RTC load** | ~2–4 GB total | ~300–600 MB | ~300–600 MB | N/A |
| **Solo operator overhead** | High (PostgreSQL, workers, YAML config)[^8] | Low (single binary + RocksDB)[^24] | Low (single binary + RocksDB) | Medium |
| **Audit log / history reliability** | Excellent (PostgreSQL ACID)[^8] | Good (RocksDB WAL)[^37] | Good (RocksDB WAL) | N/A |
| **Federation-disabled MatrixRTC** | Works (no federation dependency) | Requires workaround[^29] | Requires workaround[^29] | N/A |
| **Custom CS API client support** | Full spec compliance[^17] | Near-full, spec compliance campaign ongoing[^39] | Near-full | Partial |
| **Active development** | Yes (Element-funded)[^20] | Yes (enterprise-sponsored)[^25] | Yes (community) | No (maintenance only)[^3] |

***

## Analysis for Your Specific Use Cases

### Use Case 1: Appservice-Managed Jury Rooms with Puppet Members

This is the most operationally sensitive requirement. Your Rust bridge using `matrix-sdk-appservice` will auto-create rooms and join pseudonymous puppet users (Juror-<suffix>). The critical path is:

1. **Room creation via AS API** — Both Synapse and Tuwunel support this, but Tuwunel's `ip_source` regression (issue #465, v1.6.1) can silently break all locally-connected appservices. If deploying Tuwunel, verify this is fixed in your target version and do not set `ip_source` in the config for the loopback interface.[^33]

2. **Puppet join/registration** — Synapse's AS API is the spec reference. Tuwunel's `whoami` response code issue (issue #219) could cause `matrix-sdk-appservice` to fail user identity checks. Test this endpoint specifically with your SDK version before committing.[^32]

3. **Audit-friendly history** — For governance rooms, message ordering and irrevocability matter. Synapse + PostgreSQL provides stronger guarantees for long-term event storage. Tuwunel + RocksDB provides WAL-backed durability adequate for short-lived rooms (days to weeks).[^37]

### Use Case 2: Town Hall Rooms with Mic-Passing (MatrixRTC Stage Mode)

This is the most demanding RTC use case. The homeserver's role in MatrixRTC is:
- Serving the LiveKit focus discovery endpoint (`/rtc/transports` or `.well-known`)
- Handling `m.call.member` state events tracking RTC participants
- Token exchange via OpenID (`/openid/request_token`)

The actual media routing is handled by the LiveKit SFU sidecar — the homeserver is not in the media path. Therefore, both Synapse and Tuwunel are architecturally capable of supporting stage mode once MatrixRTC is configured correctly.

**The critical differentiator**: Synapse's implementation has been production-demonstrated; Tuwunel's has not been publicly validated at scale for stage mode events. However, given that the homeserver only handles signalling events and not media, the risk is manageable.

**The important caveat for Synapse**: Two open GitHub issues (as of early 2026) show that Synapse v1.149–v1.150 has bugs where `msc4143_enabled` does not correctly advertise the feature and the endpoint returns 404. These may be fixed in later versions — verify your target Synapse version before deployment.[^5][^6]

### Use Case 3: 1:1 DMs

Both Synapse and Tuwunel handle standard messaging without issue.

### Use Case 4: Admin Config for Identity and Room Lifecycle Policy

- **Synapse**: Has a REST admin API (`/_synapse/admin/`) for room management, user management, and purging rooms — well-documented and mature.[^16]
- **Tuwunel**: Lacks an HTTP admin API; administration is done via admin room commands. A community-built web admin UI ([knadh/tuwunel-admin](https://github.com/knadh/tuwunel-admin)) provides a basic interface over the admin room API. For programmatic lifecycle management of short-lived jury rooms, you will need to either use the admin room commands from a bot, or use the standard CS API room management endpoints (which both homeservers support).[^41]

***

## Recommendation

### Primary Recommendation: Tuwunel

For your specific constraints — solo operator, single server, Rust-native preference, no Python/PostgreSQL — **Tuwunel is the correct choice**, subject to resolving the two critical appservice issues before production deployment.

**What you must verify before committing**:
1. Issue #465 (`ip_source` breaks local appservices) must be confirmed fixed in your target release. Do not use `ip_source` with locally-connected bridges.[^33]
2. Issue #219 (`whoami` wrong response code) must be verified against your `matrix-sdk-appservice` version. Test puppet registration and identity checks end-to-end before launch.[^32]
3. For the federation-disabled launch phase, apply the `allow_federation = true; forbidden_remote_server_names = [".*"]` workaround to enable LiveKit's OpenID token exchange.[^29]
4. Never switch between Conduit-lineage forks (Tuwunel ↔ Continuwuity ↔ Conduit) — database corruption is unrecoverable.[^24]

**Architecture for your governance platform**:

```
┌─────────────────────────────────────────────────────────┐
│  Linux server (16–32 GB RAM)                            │
│                                                         │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │   Tuwunel   │  │  LiveKit SFU │  │  lk-jwt-svc   │ │
│  │  ~200 MB    │  │  ~2–4 GB     │  │  ~50 MB       │ │
│  │  port 8008  │  │  ports 7880  │  │  port 8081    │ │
│  └──────┬──────┘  │  7881/tcp    │  └───────────────┘ │
│         │         │  50100-50200 │                     │
│  ┌──────┴──────┐  │  /udp        │                     │
│  │  Rust AS    │  └──────────────┘                     │
│  │  bridge     │                                       │
│  │  (your      │  ┌──────────────┐                     │
│  │   daemon)   │  │  Lemmy fork  │                     │
│  └─────────────┘  │  (governance │                     │
│                   │   platform)  │                     │
│                   └──────────────┘                     │
└─────────────────────────────────────────────────────────┘
```

LiveKit is the dominant RAM consumer for MatrixRTC — it needs ~1–2 GB under active RTC load for 50–200 users. Tuwunel itself runs comfortably in under 500 MB, leaving substantial headroom.

### Fallback: Synapse

If the Tuwunel appservice regressions cannot be resolved satisfactorily for your Rust bridge, **Synapse is the safe fallback** with the understanding that you accept:
- Python + PostgreSQL operational overhead
- ~2–3 GB RAM for homeserver stack alone
- Synapse's own MSC4143 endpoint bugs (verify fix in your target version before deployment)[^6][^5]

Synapse's AS API is the ground truth for spec compliance and is the platform all major bridges are tested against. For governance-critical appservice operations, Synapse's additional operational overhead is a reasonable trade-off.[^15]

***

## What the Matrix.org Foundation and Element Recommend

As of mid-2026:

- **Matrix.org ecosystem page** lists Tuwunel as "Stable" (enterprise-grade) and Synapse as "Stable". Dendrite is "Beta" (effectively deprecated).[^2]
- **Element's official documentation** for MatrixRTC (ESS/matrix-docker-ansible-deploy) targets Synapse. Self-hosters outside ESS are advised to deploy their own LiveKit + lk-jwt-service stack.[^10][^7]
- **The matrix-docker-ansible-deploy playbook** (the most widely used Ansible-based Matrix deployment tool) carries an explicit warning: MatrixRTC "very likely only works with the Synapse homeserver" — written before Tuwunel v1.4.6 added MSC4143 support. The playbook now also supports Tuwunel deployment.[^42][^7]
- **Element Blog (Jan 2025)** explicitly reserves Synapse Pro (Rust workers) for nation-scale deployments; community Synapse is the recommendation for "small, medium and even large homeservers in the public Matrix network".[^20]

***

## Key Open Questions to Monitor

1. **Tuwunel issue #465 (ip_source regression)** — Track the fix release; critical for your sidecar AS architecture.[^33]
2. **MSC4143 spec stabilisation** — When MSC4143 is finalised, the endpoint path changes from `unstable/org.matrix.msc4143/rtc/transports` to a stable path. All homeservers will need updates.[^5]
3. **Tuwunel spec compliance campaign** — The team is running a multi-release campaign targeting Matrix Spec v1.18/v1.19 compliance. Check compliance status for room version 11 and appservice-related endpoints before production deployment.[^39]
4. **Migration from conduwuit → Tuwunel if needed** — Binary swap, no database migration required.[^28][^24]

---

## References

1. [Continuwuity, the official community driven continuation of ... - Ellis Git](https://forgejo.ellis.link/continuwuation/continuwuity) - We aim to provide a stable, well-maintained alternative for current conduwuit users and welcome newc...

2. [Servers - Matrix.org](https://matrix.org/ecosystem/servers/) - Advanced users may want to run a homeserver by themselves for more independence and sovereignty. Her...

3. [Dendrite is a second-generation Matrix homeserver written in Go!](https://github.com/element-hq/dendrite) - Dendrite is a second-generation Matrix homeserver written in Go. It is currently in maintenance mode...

4. [Is Dendrite being actively developed? · Issue #3413 · matrix ... - GitHub](https://github.com/matrix-org/dendrite/issues/3413) - Development is currently best effort, since Dendrite is in maintenance mode. This may change in the ...

5. [MSC4143 enabled in experimental_features but missing from ...](https://github.com/element-hq/synapse/issues/19580) - This causes Element Call to fail with the error "MISSING_MATRIX_RTC_FOCUS" because it cannot determi...

6. [Synapse 1.150.0 does not expose MatrixRTC transports endpoint ...](https://github.com/element-hq/synapse/issues/19652) - I am trying to enable MatrixRTC (MSC4143) with Element Call + LiveKit on a self-hosted setup, but th...

7. [configuring-playbook-matrix-rtc.md](https://github.com/spantaleev/matrix-docker-ansible-deploy/blob/master/docs/configuring-playbook-matrix-rtc.md) - 🐳 Matrix (An open network for secure, decentralized communication) server setup using Ansible and Do...

8. [Matrix Synapse with PostgreSQL database - FreedomBox Forum](https://discuss.freedombox.org/t/matrix-synapse-with-postgresql-database/1913) - The idea is to use PostgreSQL and to keep the Matrix Synapse database all in RAM in order to improve...

9. [Playlist for "Matrix Conference 2025" - media.ccc.de](https://media.ccc.de/v/matrix-conf-2025-73143-getting-started-with-element-server-suite-community/playlist) - This workshop will deploy all the components of the suite: a Synapse homeserver with Matrix Authenti...

10. [End-to-end encrypted voice and video for self-hosted ...](https://element.io/blog/end-to-end-encrypted-voice-and-video-for-self-hosted-community-users/) - Those self-hosting a homeserver outside of ESS will need to deploy their own MatrixRTC infrastructur...

11. [How to add support for Element Call with Docker Compose & Caddy](https://fariszr.com/matrix-rtc-setup-with-docker-caddy/) - setup Matrix RTC (Element Call) Suppport with docker compose and Caddy reverse proxy. May 27, 2025. ...

12. [This Week in Matrix 2026-01-09](https://matrix.org/blog/2026/01/09/this-week-in-matrix-2026-01-09/) - The implementations of a few MSCs where updated, like adding support for the new GET /_matrix/client...

13. [Application Service API - Matrix Specification](https://spec.matrix.org/v1.14/application-service-api/) - The Matrix client-server API and server-server APIs provide the means to implement a consistent self...

14. [Events no longer sent to application service, and no way to debug](https://github.com/matrix-org/synapse/issues/13142) - I'm trying to write an appservice, and when I first started I was getting transaction PUTs. The requ...

15. [Registering appservices - mautrix-bridges](https://docs.mau.fi/bridges/general/registering-appservices.html) - The bridge must be registered on the homeserver as an appservice. In general, this requires root acc...

16. [Application Services - Synapse - GitHub Pages](https://matrix-org.github.io/synapse/latest/application_services.html) - The registration of new application services depends on the homeserver used. In synapse, you need to...

17. [Conduit vs Synapse: Matrix Homeservers Compared - selfhosting.sh](https://selfhosting.sh/compare/conduit-vs-synapse/) - Conduit vs Synapse compared as self-hosted Matrix homeservers — lightweight Rust server versus the r...

18. [Matrix/Riot storage and performance requirements : r/selfhosted](https://www.reddit.com/r/selfhosted/comments/g9h6au/matrixriot_storage_and_performance_requirements/) - Matrix Synapse system requirements for RAM. Best practices ... It generally uses at most 3gb ram, 30...

19. [How to setup a Matrix homeserver – /techblog](https://www.redpill-linpro.com/techblog/2025/04/08/matrix-basic-en.html) - Synapse's system requirements specify at least 1GB of ram for large rooms like #matrix:matrix.org , ...

20. [Scaling to millions of users requires Synapse Pro - Element](https://element.io/blog/scaling-to-millions-of-users-requires-synapse-pro/) - Synapse Pro workers, however, are built to use shared data caches, minimising RAM footprint and serv...

21. [Denial of service due to incorrect application of event authorization ...](https://github.com/matrix-org/synapse/security/advisories/GHSA-p9qp-c452-f9r7) - The malicious homeserver can trick Synapse into accepting previously rejected events into its view o...

22. [Message order in Matrix: right now, we are deliberately inconsistent](https://artificialworlds.net/blog/2024/12/04/message-order-in-matrix/) - Topological ordering: events in a Matrix room are stored in a mathematical structure known as a dire...

23. [Message order in Matrix: right now, we are deliberately inconsistent](https://news.ycombinator.com/item?id=42324114) - The UX is fairly clear in my mind: 1. All up-to-date clients should be displaying the same message o...

24. [Introduction - Tuwunel One](https://matrix-construct.github.io/tuwunel/) - conduwuit? ✓ Yes. This will be supported at a minimum for one year, but likely indefinitely. Synapse...

25. [matrix-construct/tuwunel: Official successor to conduwuit - GitHub](https://github.com/matrix-construct/tuwunel) - This project is the official successor to conduwuit after it reached stability. Tuwunel is now used ...

26. [This Week in Matrix 2026-02-06](https://matrix.org/blog/2026/02/06/this-week-in-matrix-2026-02-06/) - 🔗Tuwunel (website). Enterprise successor to conduwuit, the high-performance and feature-rich fork of...

27. [This Week in Matrix 2025-11-07](https://matrix.org/blog/2025/11/07/this-week-in-matrix-2025-11-07/) - ✨ New Features For Version 1.4.6. Element Call discovery support (MSC4143) was implemented by tototo...

28. [Introduction - Tuwunel One](https://tuwunel.chat) - Tuwunel, a high performance successor to Conduit and Conduwuit

29. [Calls - Continuwuity](https://continuwuity.org/calls) - Matrix supports two types of calls: Element Call powered by MatrixRTC and LiveKitLegacy calls, somet...

30. [Appservices - Tuwunel One](https://tuwunel.chat/appservices.html) - Tuwunel, a high performance successor to Conduit and Conduwuit

31. [matrix-construct/tuwunel v1.6.0 on GitHub](https://newreleases.io/project/github/matrix-construct/tuwunel/release/v1.6.0) - New release matrix-construct/tuwunel version v1.6.0 Release v1.6.0 on GitHub.

32. [Wrong Response Code for whoami Endpoint · Issue #219 - GitHub](https://github.com/matrix-construct/tuwunel/issues/219) - I have been trying to set up the Mautrix Telegram bridge as an appservice in conjunction with my new...

33. [ip_source breaks locally-connected appservices since v1.6.1 #465](https://github.com/matrix-construct/tuwunel/issues/465) - Trusting the loopback address works for on-host deployments, but in fully containerized environments...

34. [Ignore appservice accounts for grant_admin_to_first_user #331](https://github.com/matrix-construct/tuwunel/issues/331) - I maintain Ansible automation for deploying Tuwunel as well as a (bridge) appservice; the latter is ...

35. [bug: appservice users are not created on registration #813 - Ellis Git](https://forgejo.ellis.link/continuwuation/continuwuity/issues/813) - Continuwuity forgets to create the user with the appservice-specified localpart when it is registere...

36. [Reminder - https://conduit.rs/ Rust implementation of the matrix ...](https://news.ycombinator.com/item?id=38163174) - I am really impressed with how little resources Conduit uses. A fresh install with a few rooms was o...

37. [RocksDB FAQ - GitHub](https://github.com/facebook/rocksdb/wiki/RocksDB-FAQ/66b6693d0da17a71c702c6982e1ad1aadb7fdcd5) - Q: If my machine crashes and rebooted, will RocksDB preserve the data? A: Data is synced when you is...

38. [thread main panic after update to v1.6.2 · Issue #452 - GitHub](https://github.com/matrix-construct/tuwunel/issues/452) - The 1.6.2 mediaid_user migration panics on legacy rows from older databases, which catches anyone wh...

39. [Releases · matrix-construct/tuwunel - GitHub](https://github.com/matrix-construct/tuwunel/releases) - We have started a specification compliance campaign which will continue over the next several releas...

40. [#849 - Comparison with tuwunel - continuwuation ... - Ellis Git](https://forgejo.ellis.link/continuwuation/continuwuity/issues/849) - Thanks for taking care of conduwuit! Would it be possible to give some insights to why there are two...

41. [knadh/tuwunel-admin - GitHub](https://github.com/knadh/tuwunel-admin) - Tuwunel has no HTTP admin API and its administration is done by sending text commands to the server'...

42. [Identity Providers (oauth2 /...](https://github.com/spantaleev/matrix-docker-ansible-deploy/blob/master/docs/configuring-playbook-tuwunel.md) - 🐳 Matrix (An open network for secure, decentralized communication) server setup using Ansible and Do...

