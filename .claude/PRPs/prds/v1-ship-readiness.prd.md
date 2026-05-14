# Sub-PRD: v1 Ship Readiness — From Substrate-Complete to First External User

**Scope:** v0 ship-gate closure. Three sequenced sub-phases addressing the structural gaps identified in `v0-endpoint-coverage-2026-05-14.md` between current state and "ready for first external user." Does NOT replace any ADR; additive to numbered design docs.
**Created:** 2026-05-14T08:00:00Z
**Status:** DRAFT
**Primary input:** `.claude/PRPs/reports/v0-endpoint-coverage-2026-05-14.md`
**Hard boundary:** the 11 v0 endpoints (per `05-mvp-and-delivery-plan.md` §2) + the 15 ADRs (per `99-decisions-and-open-questions.md`). No scope creep into v1 feature breadth (RT-r2..RT-r6 / JM-f / restorative-mechanics-v1) — those are explicitly out-of-scope here.

---

## §1. Vision and Goals

### 1.1 The ship-gate problem

The fork has shipped 9 sub-phases of substrate work (SL-a/b/c-1/c-2/d/e + JM-d/e + RT-r1). The 2026-05-14 endpoint audit confirms:

- **11/11 v0 endpoints have wired handlers, DTOs, routes, Diesel models, migrations.**
- **Quorum-3 simple-majority aggregation matches `05-mvp-and-delivery-plan.md` §6 exactly.**
- **ADRs 011, 013, 014, 015 + hash chain are present in code.**
- **Federation outbound (`SanctionNotice` publish) + advisory inbound (`remote_sanction_notice`) wired.**

But the fork is **not shippable**. Two structural ship-gates remain:

1. **AGPL §13 source-disclosure not user-visible.** LICENSE + root `AGPL-NOTICE.md` exist, but no HTTP endpoint returns the disclosure to a connecting client. The §13 Remote Network Interaction clause triggers the moment any external user connects across a network. Without user-visible disclosure, the fork ships out of license compliance from the first byte.
2. **Per-endpoint e2e coverage absent on 4 of 11 endpoints.** Substrate is wired; HTTP-shape is unexercised for `POST /appeal`, `GET /modlog`, `GET /reputation/me`, `POST /endorsement`. The `report_to_modlog_golden_path` test (`crates/server/tests/e2e.rs:2078`) covers the cross-endpoint flow but does not assert per-endpoint failure modes. First external user hits one of these untested endpoints; failure surfaces as a runtime trace, not a captured assertion.

Plus a tail of Tier-2 polish: `docker-compose.yml` Postgres unpinned, `POST /report` returns raw row instead of a view, sponsor-liability `2-sponsors-lose-reputation` e2e not confirmed by name.

### 1.2 v1-ship goals

1. **Compliance from day one** — AGPL §13 disclosure machinery wired into a fresh-client first-touch surface (`/api/v4/site`); load-bearing for "shippable."
2. **Per-endpoint HTTP signal** — every one of the 11 endpoints has a named e2e test exercising the wire shape. Failure modes (auth, rate-limit, conflict) asserted at least once.
3. **Deploy reproducibility** — `docker-compose.yml` pinned (no `latest` tags); `POST /report` returns a stable view; the §9 done-definition criterion "2 sponsors lose reputation on sanction" has a named test asserting it.
4. **Bounded scope** — three sub-phases, each independently shippable. Total wall-clock estimate: 2-3 weeks if dedicated; 4-6 weeks if interleaved with parallel feature lanes.

### 1.3 Non-goals

- **WebAuthn / passkey MFA** — optional per CLAUDE.md, v2 per ADR-010. Deferred to v2 security-hardening PRD.
- **Public anchoring of governance log** — v3 per `05-mvp-and-delivery-plan.md` §7.3. Deferred.
- **Operator runbook empirical-test (can a non-author execute it?)** — needs an external user; deferred to post-pilot follow-up.
- **RT-r2..RT-r6, JM-f, restorative-mechanics-v1** — tactical lanes addressing v1 feature breadth (decay tuning, sponsor strategies, instance roll-up). They continue in parallel; ship does not gate on them.
- **CodeRabbit triage on pre-existing CR findings** — handled per-PR via the standard `/bm-poll-cr` → `/bm-triage` loop; not a structural ship-gate.
- **Performance / load testing under realistic traffic** — important but not §9-criteria gating; deferred to pre-pilot.

---

## §2. Scope

### IN scope (v1-ship)

- **Phase v1-ship-1 — AGPL §13 surface.** Extend `GetSiteResponse` (`crates/db_views/site/src/api.rs:337`) with a `source_disclosure` field bundling: license SPDX, repo URL, current fork commit (build-time injected), notice text. Wire `get_site` handler (`crates/api/api/src/site/`) to populate it. Add a dedicated `/api/v4/source` route returning the full `AGPL-NOTICE.md` body as JSON `{notice: string, license: "AGPL-3.0"}`. Acceptance: fresh client (no auth) hitting `/api/v4/site` receives the disclosure URL; the URL resolves to the notice. e2e test asserts both.
- **Phase v1-ship-2 — per-endpoint e2e backfill.** Author named e2e tests for the 4 untested endpoints (`POST /appeal`, `GET /modlog`, `GET /reputation/me`, `POST /endorsement`). Each test exercises: (a) happy path (200 + expected body shape), (b) one failure mode (auth-missing 401, conflict 409, or validation 400). Tests follow `report_to_modlog_golden_path` shape (`crates/server/tests/e2e.rs:2078`) and the `fixtures` pattern used in SL-a..SL-e sibling tests. Acceptance: every one of the 11 endpoints has a named test in `e2e.rs`; running the test file passes cleanly under `cargo test --workspace --test e2e --features full`.
- **Phase v1-ship-3 — tactical polish bundle.** Three additive deliverables:
  - `docker-compose.yml` Postgres pin (currently uses unpinned tag; audit found `lemmy-ui:nightly` and `nginx:1-alpine` already drift-prone).
  - `POST /report` returns `CreateGovernanceReportResponse` wrapping a prepared view (parallel to `GovernanceCaseSummaryView` returned by `GET /cases`); align field-stability contract across read+write surface.
  - Named e2e test asserting "user signs up with 2 sponsors, commits a sanctionable offence, both sponsors lose reputation on `endorsement_strength`" per `05-mvp-and-delivery-plan.md` §9 done-criterion. The substrate (`apply_sponsor_liability` at `crates/api/api/src/governance/sponsor_liability.rs:429`) is wired; the named assertion is missing.

### OUT of scope (deferred — named, not detailed)

- **WebAuthn / passkey MFA** — v2 per ADR-010; documented in `06-security-and-threat-model.md`.
- **Public anchoring of governance log** — v3 per `05-mvp-and-delivery-plan.md` §7.3 (Sigstore Rekor / BTC `OP_RETURN`).
- **Empirical runbook test** — needs external user; post-pilot.
- **Performance / load characterization** — needs realistic traffic; pre-pilot follow-up PRD.
- **Release artifact pipeline (signed binaries / SBOM / tarball)** — Tier-1 in the audit but **moved out of v1-ship scope** per §3 decision below. Reasoning: this PRD's IN-scope items unblock "first external user connects compliantly and the API works"; binary distribution and SBOM are pilot-scale concerns, not first-user-scale. Tracked in a follow-up PRD `v2-release-pipeline.prd.md`.
- **Everything in RT-r2..RT-r6 + JM-f + restorative-mechanics-v1** — parallel tracks; do not gate ship.

---

## §3. Why this PRD shape (and not larger)

The v0 endpoint coverage audit (`v0-endpoint-coverage-2026-05-14.md` §8) identified three Tier-1 gaps:

1. AGPL §13 disclosure not user-visible.
2. e2e coverage gaps on 4 endpoints.
3. No release-artifact pipeline.

This PRD scopes (1) + (2) + Tier-2 polish. It does **not** scope (3) the release-artifact pipeline. The decision rationale:

- **First external user** = a person connecting an HTTP client to the running instance. They need: (a) the API to work end-to-end (gap 2), (b) the fork to be in legal compliance the moment they connect (gap 1).
- They do **not** need: a signed binary release, an SBOM, a cargo-binstall-friendly tarball. Those are pilot-scale concerns (multi-instance distribution, supply chain verification) for the §7.2 v2 milestone.
- A separate `v2-release-pipeline.prd.md` will cover (3); it's named here as the natural follow-up.

This scoping discipline matches `feedback_principles_not_rules.md` (scope what unblocks the next concrete step; defer what unblocks two steps later).

---

## §4. ADRs That Govern This

| ADR | Summary | How it constrains this PRD |
|---|---|---|
| [ADR-010](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | v0 / v1 / v2 / v3 staging | Confirms WebAuthn (v2) + public anchoring (v3) are out-of-scope here |
| [ADR-011](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | AGPL-3.0 inherited + source-disclosure required | **Drives Phase v1-ship-1.** Every release must honour §13 user-visible disclosure |
| [ADR-013](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Illegal content `EmergencyRemove` mandatory | Confirmed present at `crates/db_schema_file/src/enums.rs:404`; not affected by this PRD |
| [ADR-014](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Outbound-only federation in v0 | Confirmed wired (`crates/apub/activities/src/governance/publish_sanction_notice.rs:226`); not affected by this PRD |
| [ADR-015](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | GDPR `actor_pseudonym` table mandatory | Confirmed present at `crates/db_schema/src/source/governance/actor_pseudonym.rs`; e2e tests authored under Phase v1-ship-2 MUST use pseudonyms in any user-visible response assertion |

**Contradiction check:** None. The PRD is additive — extends a DTO, adds a route, authors tests, pins a Docker tag. No ADR is touched in spirit or letter.

---

## §5. Open Questions This Touches

| OQ | Status | Impact on this PRD |
|---|---|---|
| OQ-022 (source-disclosure surface shape) | Not registered as a numbered OQ; new design question | Resolved inline: `/api/v4/site` field + `/api/v4/source` endpoint (§7.1) |

No blocking OQs from `99-decisions-and-open-questions.md`. The disclosure-surface shape (whether to extend `GetSiteResponse` vs add a dedicated `/source` endpoint vs both) is a **design question this PRD resolves**, not an ADR-deferred OQ.

---

## §6. Hard constraints (from `CLAUDE.md` — verbatim boundary)

These bound every phase below. Repeated here so the PRD is self-contained for `/prp-plan`:

- **Lemmy 1.0-beta fork**; Extism plugin host for governance hooks (ADR-012)
- **AGPLv3** inherited (ADR-011) — every release honours the source-disclosure notice
- **v0 scope = exactly the 11 endpoints** in `05 §2`; nothing else
- **v0 simplifications**: 5-juror panels, quorum 3, simple majority, outbound-only federation, local hash chain, reputation-decay stub
- **Solo-dev stack:** NO Keycloak, NO OpenFGA, NO Vault, NO Kubernetes, NO external log signer, NO blockchain anchoring
- **Auth:** Lemmy's existing JWT; optional passkey MFA via `webauthn-rs` (out of scope here)
- **Authz:** hardcoded capability checks in Rust reading `reputation_snapshot` flags
- **Governance log:** `sha2` hash chain via Postgres triggers; `rs_merkle` + `ed25519-dalek` for signing; key in `.env` for v0
- **GDPR from day 1:** pseudonymised `actor_pseudonym` table mandatory (ADR-015)
- **Illegal content from day 1:** `CaseStatus::EmergencyRemove` mandatory (ADR-013)
- **Federation:** content-level with vanilla Lemmy works; governance signals are fork-only AP types (ADR-014)

If any phase plan contradicts these, the planner MUST STOP and surface to the user.

---

## §7. Phase Details

### 7.1 Phase v1-ship-1 — AGPL §13 surface

**Goal:** First external user receives source-disclosure as part of the standard handshake. No client-side opt-in; no auth required.

**Scope:**

1. Extend `GetSiteResponse` (`crates/db_views/site/src/api.rs:337`) with a `source_disclosure: SourceDisclosure` field. `SourceDisclosure` is a new struct in the same file:

   ```rust
   #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
   #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
   #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
   pub struct SourceDisclosure {
     pub license: String,         // "AGPL-3.0"
     pub repo_url: String,        // "https://github.com/barrie-cork/lemmy"
     pub fork_commit: String,     // build-time injected; HEAD SHA
     pub disclosure_url: String,  // "/api/v4/source" — relative path the client follows
   }
   ```

2. Wire `get_site` handler to populate `source_disclosure`. Repo URL + license are constants; `fork_commit` is read from a `BREHON_FORK_COMMIT` env var injected at build time (`build.rs` if needed). Falls back to `"unknown"` if env var is absent — non-blocking.

3. Add new route `GET /api/v4/source` to `crates/api/routes/src/lib.rs`. Handler at `crates/api/api/src/site/source.rs` returns:

   ```rust
   pub struct GetSource {}
   pub struct GetSourceResponse {
     pub notice: String,    // contents of AGPL-NOTICE.md, read at startup or from include_str!
     pub license: String,   // "AGPL-3.0"
   }
   ```

   Implementation reads `AGPL-NOTICE.md` via `include_str!("../../../../../AGPL-NOTICE.md")` at compile time (path relative to crate root). No auth check; public endpoint. No rate-limit (read-only; small payload).

4. e2e test: `agpl_source_disclosure_surface_returns_notice` at `crates/server/tests/e2e.rs`. Asserts:
   - `GET /api/v4/site` returns 200 with `source_disclosure.license == "AGPL-3.0"` and `source_disclosure.disclosure_url == "/api/v4/source"`.
   - `GET /api/v4/source` returns 200 with `notice.len() > 0` and `notice` contains the literal string `"GNU Affero General Public License"`.

**Ship-criteria (beyond cargo green):** a fresh `curl http://localhost:8536/api/v4/site` against a running instance returns `source_disclosure` block; following the `disclosure_url` to `/api/v4/source` returns the notice text. Verifiable by an external observer with no project context.

**Complexity score (per planner template §5):** **3/10.**
- 1 DTO struct addition (no migration).
- 1 handler addition.
- 1 route registration.
- 1 `include_str!` macro for the static notice.
- 1 e2e test.
- 0 type-shape changes that affect existing call sites.
- 0 cross-crate coordination.

**Estimated wall-clock:** 1-2 days. One §13-task plan; cohort marker `[P]` not applicable (single deliverable).

**Risks:**

| Risk | Likelihood | Mitigation |
|---|---|---|
| Build-time `BREHON_FORK_COMMIT` env var missing in some build path (Dockerfile, CI, local) | Medium | Default to `"unknown"` in `build.rs`; warn-log if absent; non-blocking |
| `ts-rs` codegen drift from new field on `GetSiteResponse` | Low | Field is optional in the TS export per `optional_fields` derive |
| `AGPL-NOTICE.md` path drift (file moves) | Low | `include_str!` is compile-time; file move breaks the build loudly |
| Lemmy upstream rebases change `GetSiteResponse` shape | Medium | Field addition is additive; rebase conflicts resolve by re-adding the field |

### 7.2 Phase v1-ship-2 — per-endpoint e2e backfill

**Goal:** Every one of the 11 v0 endpoints has a named e2e test asserting wire-shape + one failure mode. The audit's "4 endpoints untested" gap closes.

**Scope:** Four new e2e tests in `crates/server/tests/e2e.rs`, each following the `report_to_modlog_golden_path` pattern (test fn returning `LemmyResult<()>`, setup via fixtures, assertion via DTO match):

1. **`appeal_endpoint_request_and_validate`** — calls `POST /governance/appeal` on a `Decided` case; asserts 200 + `AppealId` returned; second test arm asserts 409 when appealing an `EmergencyRemove` case (terminal state per ADR-013).

2. **`modlog_endpoint_returns_published_cases`** — seeds a `Decided` case with a `PublicCaseLog` entry; calls `GET /governance/modlog`; asserts entry appears with pseudonymised actor (ADR-015 invariant: no raw `person_id`). Second arm: unauthenticated call succeeds (modlog is public).

3. **`reputation_me_endpoint_returns_summary`** — seeds a user with reputation events; calls `GET /governance/reputation/me`; asserts summary shape matches `GetMyReputationResponse` DTO. Second arm: 401 when unauthenticated (endpoint requires `local_user`).

4. **`endorsement_endpoint_create_and_idempotency`** — calls `POST /governance/endorsement` between two users; asserts 200 + `EndorsementId`. Second arm: re-call within the 48h cooldown window asserts 429 / cooldown error.

Each test:
- Lives in a sibling `mod` to the existing fixtures (per `feedback_lemmy_error_no_std_error.md` Case A discipline: outer `LemmyResult<()>`; helpers match).
- Uses `?` for error propagation with `.map_err()` only where Lemmy-native errors cross into `LemmyResult<()>`.
- Does NOT modify the existing `report_to_modlog_golden_path` — that test stays as the cross-endpoint flow assertion.

**Ship-criteria (beyond cargo green):** `cargo test --workspace --test e2e --features full -- --nocapture` shows 4 new test names in the output; total e2e count for governance endpoints reaches **11 named tests** (one per endpoint, plus the existing golden-path) for the 11 endpoints. Audit re-run (or grep `fn .*_endpoint_` in `e2e.rs`) confirms each endpoint has a named match.

**Complexity score:** **6/10.**
- 4 new e2e tests in the workspace's most fixture-heavy file (`e2e.rs` at 8000+ lines).
- Pattern is well-established (sibling tests exist for `report_to_modlog_golden_path`); but Case A vs Case B error-shape discipline (per `feedback_lemmy_error_no_std_error.md`) requires care.
- Sponsor/case fixtures must be reused (not re-built) — copying the existing fixture pattern verbatim, not authoring new ones.
- Risk of triggering `feedback_junior_worker_e2e_edit_hang.md` if any single Edit touches >300 lines (e2e.rs file size sensitive).

**Estimated wall-clock:** 5-8 days. One sub-phase plan; 4-6 §13 tasks (one per endpoint + one fixture-prep task + one cleanup task). Tasks ARE candidates for `[P]` cohort dispatch if each endpoint's tests are file-disjoint within `e2e.rs` (they are not — all 4 share `e2e.rs`) so serial dispatch with `feedback_junior_worker_e2e_edit_hang.md` discipline.

**Risks:**

| Risk | Likelihood | Mitigation |
|---|---|---|
| e2e.rs edits hang on Junior workers (per lesson `feedback_junior_worker_e2e_edit_hang.md`) | High | Each task edits ≤2 sites in e2e.rs; single-edit-per-commit if needed; if collapse-Junior eventually lands (parked per 2026-05-13), risk dissolves |
| Reputation/endorsement fixtures don't expose what the test needs | Medium | Phase first task is a fixture-audit; if gap, file DQ and pivot brief |
| `report_to_modlog_golden_path` cross-talk (new tests share DB / containers with golden) | Low | testcontainers spawns fresh PG per test; verify with `cargo test --workspace --test e2e --features full --test-threads 1` if flake |
| Test failure on appeal endpoint reveals an actual handler bug (not just missing test) | Medium | Acceptable — that's the test working; PRD §11 says found bugs route via DQ to a follow-up fix-impl task, not in-line |
| Pseudonym assertion fails because handler returns raw `person_id` | Medium | If found: ADR-015 conformance bug; raise blocking DQ; fix is within scope (not a follow-up) since ADR-015 is a hard constraint |

### 7.3 Phase v1-ship-3 — tactical polish bundle

**Goal:** Resolve the audit's Tier-2 items in one bundled sub-phase. These are independent deliverables but co-locate to amortize the bm-cut/bm-pr/bm-merge overhead.

**Scope:**

1. **`docker-compose.yml` Postgres pin.** Current `docker-compose.yml` does not pin Postgres version; the audit flagged `lemmy-ui:nightly` (acceptable for dev), `pictrs:0.5.17-pre.9` (pinned), `nginx:1-alpine` (semi-pinned), Postgres unpinned. Pin to `postgres:16.4` (matches Lemmy 1.0-beta upstream's tested target; verify in `lemmy/docker/docker-compose.yml` or `Cargo.toml` schema_dump-targeted version). One-line YAML edit + commit; no test impact unless integration tests hit Postgres-version-specific behavior (audit them via grep for `pg_version` references).

2. **`POST /report` returns prepared view.** Currently `create_report` returns the raw `moderation_case` row (per audit). Refactor to wrap in `CreateGovernanceReportResponse { case: GovernanceCaseSummaryView }` — parallel to `GET /cases` which returns `Vec<GovernanceCaseSummaryView>`. Aligns the field-stability contract: clients that read `GET /cases` and write `POST /report` see the same shape. Migration story: rename the response type; the DTO field reshape is a v0-internal breaking change since v0 hasn't shipped externally — acceptable. Update existing e2e (`report_to_modlog_golden_path`) to assert against the view shape.

3. **Sponsor-liability `2-sponsors-lose-reputation` e2e by name.** Per `05-mvp-and-delivery-plan.md` §9: "user signs up with 2 sponsors, commits a sanctionable offence, both sponsors lose reputation on `endorsement_strength`." Audit found the substrate wired (`apply_sponsor_liability` at `sponsor_liability.rs:429`) but no named e2e test asserting the 2-sponsor outcome. Author `two_sponsors_lose_endorsement_strength_on_sanction` at `e2e.rs`:
   - Fixture: 1 sponsee + 2 sponsors with `endorsement_strength: 10` initial.
   - Action: sponsee receives a sanction via the full report → jury → vote flow.
   - Assertion: both sponsors' `endorsement_strength` decremented by the per-severity-tier delta.

**Ship-criteria (beyond cargo green):**
- `docker-compose.yml` greps clean for any unpinned image tag (mechanical: `grep -E 'image:.*:(latest|nightly)' docker-compose.yml` returns only the lemmy-ui line, which is intentional for dev).
- `POST /report` returns `CreateGovernanceReportResponse` per audit recommendation; sibling read endpoints unaffected.
- The §9 done-criterion "2 sponsors lose reputation" maps to a named e2e test.

**Complexity score:** **5/10.**
- Three independent deliverables; co-located only for ship-overhead amortization.
- Item 1 is a 1-line YAML edit; trivial.
- Item 2 reshapes a DTO and one e2e test; medium.
- Item 3 is a new e2e test following the same Case A/B discipline as Phase v1-ship-2; medium.

**Estimated wall-clock:** 3-5 days. One sub-phase plan; 3 §13 tasks (one per deliverable). All three are file-disjoint → cohort `[P]` marker is genuinely applicable.

**Risks:**

| Risk | Likelihood | Mitigation |
|---|---|---|
| Postgres pin to `16.x` breaks against installed `postgres:latest` in dev | Low | Lemmy 1.0-beta upstream documents target; CI uses pinned image |
| `POST /report` DTO reshape breaks any in-flight external integrations | Negligible | v0 hasn't shipped externally; no external integrations exist yet |
| `2-sponsors` e2e reveals sponsor-liability bug | Medium | Acceptable — surfaces a real ADR-conformance issue; raise blocking DQ |
| Per-severity-tier delta values not stable enough for assertion | Low | Read defaults from `governance_config` table per `sponsor_liability.rs` — values are deterministic for the test's severity input |

---

## §8. Cross-Cutting Impact

| Surface | Touched? | Detail |
|---|---|---|
| Hash-chain governance log | **No** | Phases 1+3 add a DTO field and a YAML pin; Phase 2 adds tests. No `governance_log` entries authored. |
| `actor_pseudonym` table | **Indirectly (read)** | Phase 2 e2e tests assert pseudonymized response bodies (ADR-015 invariant). If any test reveals raw `person_id` exposure, that's a conformance bug; fix in-scope. |
| `CaseStatus::EmergencyRemove` | **Read-only assertion** | Phase 2 appeal test asserts 409 on `EmergencyRemove` cases (terminal-state invariant per ADR-013). |
| AGPLv3 notice / source disclosure | **YES — primary deliverable** | Phase 1 wires the user-visible disclosure surface. |
| ADR-013 enum-exhaustiveness invariant | **Read-only** | No new `CaseStatus` variants in scope. |
| ADR-010 won't-disadvantage rule | **N/A** | No data migration; no retroactive change to existing cases. |
| ADR-015 pseudonymisation | **Asserted, not modified** | Phase 2 tests assert correctness; helper unchanged. |
| ADR-014 federation outbound | **N/A** | No federation surface touched. |

---

## §9. Backwards Compatibility

- **`GetSiteResponse`** gains a new field. `ts-rs` codegen with `optional_fields` makes the field non-breaking for TS clients. Rust clients matching exhaustively on the struct would fail-loud; existing internal callers don't match exhaustively.
- **`POST /report` DTO reshape (Phase 3 item 2)** is a v0-internal breaking change. The fork has not shipped externally; no external clients depend on the response shape. The audit explicitly identified the raw-row return as a "biggest gap" worth fixing before first external user — exactly the change window where this is cheap.
- **`docker-compose.yml` Postgres pin** is forward-only. Existing local Postgres data volumes survive the pin (same major version family); cross-major-version pins require a backup-restore step but `16.x` should match current deployments.

---

## §10. Implementation Phases (for follow-up `/prp-plan` runs)

| # | Phase | Description | Status | Depends | Complexity | Est. wall-clock |
|---|---|---|---|---|---|---|
| 1 | v1-ship-1 | AGPL §13 surface — extend `GetSiteResponse`, add `/api/v4/source`, e2e test | pending | None | 3/10 | 1-2 days |
| 2 | v1-ship-2 | Per-endpoint e2e backfill — 4 new tests for `POST /appeal`, `GET /modlog`, `GET /reputation/me`, `POST /endorsement` | pending | None (parallel with Phase 1) | 6/10 | 5-8 days |
| 3 | v1-ship-3 | Tactical polish — Postgres pin, `POST /report` view reshape, `2-sponsors` named e2e | pending | Phase 2 (shares `e2e.rs` discipline) | 5/10 | 3-5 days |

**Phase 1 and Phase 2 are independent** — Phase 1 touches `site.rs` + new route file + new e2e; Phase 2 touches existing `e2e.rs` only. They can run in parallel sub-phase lanes. Phase 3 depends on Phase 2 for the e2e discipline rhythm but does not strictly require Phase 2 to land first.

**Total estimated wall-clock:** ~2-3 weeks if dedicated single-lane; ~4-6 weeks if interleaved with parallel RT-r2..RT-r6 / JM-f tracks.

---

## §11. Decisions Log

| Decision | Choice | Alternatives | Rationale |
|---|---|---|---|
| AGPL disclosure surface shape | Both `/api/v4/site` field + dedicated `/api/v4/source` endpoint | (a) site field only (b) dedicated endpoint only | First-touch handshake clients hit `/api/v4/site`; specialized AGPL-audit tools may hit `/source` directly. Both surface the same data; the field acts as a pointer, the endpoint serves the body. Minimal cost; maximum discoverability. |
| Bundle disclosure with Lemmy `GetSiteResponse` vs separate | Bundle | Separate | Lemmy clients already deserialize `GetSiteResponse` on connection; adding a field surfaces the disclosure without any client opt-in. Separate would require client cooperation. |
| Release-artifact pipeline scope | OUT of v1-ship; into `v2-release-pipeline.prd.md` | Bundle with v1-ship | First external user does not need a signed binary; pilot scale does. Defer to match the use-case. |
| e2e test pattern for new endpoints | Mirror `report_to_modlog_golden_path` | Greenfield pattern; or move to `nextest` | The mirror keeps fixtures consistent; pattern is proven across SL-a..SL-e. Greenfield costs context; nextest is a separate tooling decision. |
| `POST /report` view reshape | Inside v1-ship (Phase 3) | Defer to a v1.x patch | First external user observes the inconsistency immediately on first `POST` followed by first `GET`; v0-internal break is cheap NOW, expensive later. |
| `[P]` cohort marker for Phase 3 tasks | Yes — three deliverables are file-disjoint | Serial | The audit confirms `docker-compose.yml`, `crates/api/api_crud/src/governance/create_report.rs`, and `crates/server/tests/e2e.rs` are disjoint; cohort `[P]` is honest. |
| Cohort marker for Phase 2 tasks | No — all four share `e2e.rs` | Yes | `feedback_parallel_cohort_dispatch.md` requires file-disjoint claims; tests sharing `e2e.rs` are not file-disjoint. Serial dispatch with `feedback_junior_worker_e2e_edit_hang.md` discipline. |
| Postgres pin version | `postgres:16.4` | `15.x`; `17.x` | Match Lemmy 1.0-beta upstream; 16 is current-stable; 17 is too new for upstream testing matrix. |

---

## §12. Research Summary

**Codebase findings (from audit + targeted reads):**
- `crates/db_views/site/src/api.rs:337` — `GetSiteResponse` struct (Phase 1 extension point).
- `crates/api/api/src/site/` — handler directory; will house new `source.rs` (Phase 1).
- `crates/api/routes/src/lib.rs` — route registration table (Phase 1 + new route).
- `crates/server/tests/e2e.rs:2078` — `report_to_modlog_golden_path` test signature (template for Phase 2 + Phase 3 item 3).
- `crates/api/api/src/governance/sponsor_liability.rs:429` — `apply_sponsor_liability` emit-site (Phase 3 item 3 assertion target).
- `AGPL-NOTICE.md` (repo root) — Phase 1 `include_str!` target.

**Design-doc alignment:**
- `05-mvp-and-delivery-plan.md §9` — done-definition checkboxes match this PRD's ship-criteria 1:1.
- `99-decisions-and-open-questions.md` — ADR-011 (AGPL) is the load-bearing constraint; no superseding-ADR path needed.
- `06-security-and-threat-model.md` — confirms WebAuthn / passkey is v2 scope; out-of-PRD as designed.

**Out-of-scope tracking (named, not detailed):**
- WebAuthn / passkey MFA — `v2-security-hardening.prd.md` (does not yet exist; named for traceability).
- Public anchoring — `v3-verifiability.prd.md` (does not yet exist).
- Release pipeline — `v2-release-pipeline.prd.md` (does not yet exist; named in §3).
- Performance / load — `pre-pilot-performance.prd.md` (does not yet exist).

---

## §13. Acceptance for v1-ship (composite)

The v1-ship goal is met when all three of the following hold:

1. **Phase v1-ship-1 shipped:** a fresh client receives source-disclosure on `/api/v4/site`; the URL resolves to the notice text. e2e test asserts both.
2. **Phase v1-ship-2 shipped:** all 11 v0 endpoints have a named e2e test exercising HTTP shape + at least one failure mode. Audit re-run confirms 11/11.
3. **Phase v1-ship-3 shipped:** `docker-compose.yml` has no unpinned image tags (except the deliberate `lemmy-ui:nightly` dev choice); `POST /report` returns a view; the §9 done-criterion "2 sponsors lose reputation" maps to a named e2e.

Once all three land, the §9 done-definition for v0 ship is satisfied **modulo** the empirical runbook test (which needs an external user — out of scope here). The fork is ready to onboard the first external user without:
- AGPL §13 breach
- Silent HTTP-shape failures on 4 of 11 endpoints
- Tier-2 polish items biting the first user

---

## §14. Cross-References

- **Primary input:** `.claude/PRPs/reports/v0-endpoint-coverage-2026-05-14.md` (§7 ranked gap analysis, §9 single biggest gap).
- **Sibling PRDs (out of v1-ship scope but tracked):**
  - `v1-sponsor-liability.prd.md` — v1 feature breadth; ships in parallel.
  - `v1-jury-mechanics.prd.md` — v1 feature breadth; ships in parallel.
  - `v1-reputation-tuning.prd.md` — v1 feature breadth; ships in parallel.
  - `v1-admin-dashboard.prd.md` — v1 feature breadth; ships in parallel.
  - `v1-federation-inbound.prd.md` — v1 feature breadth; ships in parallel.
- **Follow-up PRDs (named, do not yet exist):**
  - `v2-release-pipeline.prd.md` — signed binaries, SBOM, tarball machinery (Tier-1 audit item moved out per §3).
  - `v2-security-hardening.prd.md` — WebAuthn, step-up auth, Keycloak (ADR-010 v2).
  - `v3-verifiability.prd.md` — public anchoring (ADR-010 v3).
  - `pre-pilot-performance.prd.md` — load characterization.
- **Lesson citations:**
  - `feedback_lemmy_error_no_std_error.md` — Case A/B/C discipline for Phase 2 + Phase 3 item 3 e2e tests.
  - `feedback_junior_worker_e2e_edit_hang.md` — Phase 2 serial-dispatch discipline.
  - `feedback_parallel_cohort_dispatch.md` — Phase 3 `[P]` cohort honesty rule.
  - `feedback_principles_not_rules.md` — §3 scoping discipline.

---

*Generated: 2026-05-14T08:00:00Z*
*Status: DRAFT — review before running `/prp-plan` on any of the three phases. Phase 1 and Phase 2 may run in parallel; Phase 3 sequences after Phase 2 for e2e discipline rhythm.*
