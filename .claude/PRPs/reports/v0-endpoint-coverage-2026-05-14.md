# v0 endpoint coverage audit — 2026-05-14

**Method:** 5 parallel Explore subagents, one per endpoint group, read-only inventory.
**Scope:** The 11 v0 endpoints from `docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md` §2, plus cross-cutting AGPL §13 / federation outbound / reproducible-build / runbook / threat-model.
**Read paths:** `migrations/`, `crates/db_schema/`, `crates/db_views/`, `crates/api/api_common/`, `crates/api/api/`, `crates/api/api_crud/`, `crates/api/routes/`, `crates/server/tests/e2e.rs`, `crates/apub/`, root meta-files.
**Output of this report informs:** the v1-production-readiness PRD (next step).

---

## 1. Matrix — all 11 endpoints

Legend: ✓ = present, ✗ = absent, ~ = present but thin (substrate only, no behaviour).

### Reporting / cases (3 endpoints)

| Endpoint | Migration | Diesel | View | DTO | Handler | Route | e2e |
|---|---|---|---|---|---|---|---|
| POST /governance/report | ✓ `2026-04-15-100100` | ✓ `moderation_case` | ✗ none (write-side) | ✓ `CreateGovernanceReport` + response | ✓ `create_report.rs` (api_crud) | ✓ `/governance/report` | ~ endpoint probe @e2e.rs:3791 |
| GET /governance/case | ✓ same migration | ✓ `moderation_case` | ✓ `GovernanceCaseDetailView` | ✓ `GetGovernanceCase` | ✓ `get_case.rs` (api) | ✓ `/governance/case` | ~ endpoint probe @e2e.rs:3798 |
| GET /governance/cases | ✓ same migration | ✓ `moderation_case` | ✓ `GovernanceCaseSummaryView` | ✓ `ListGovernanceCases` + response | ✓ `list_cases.rs` (api) | ✓ `/governance/cases` | ~ endpoint probe @e2e.rs:3799 |

### Jury (4 endpoints)

| Endpoint | Migration | Diesel | View | DTO | Handler | Route | e2e |
|---|---|---|---|---|---|---|---|
| GET /governance/jury/me | ✓ `2026-04-15` add_jury_system | ✓ `JuryAssignment` | ✓ `JuryQueueView` | ✓ implicit response | ✓ `list_my_jury_queue.rs` | ✓ `/governance/jury/me` | ✓ `jury_queue_view_returns_assignments` |
| POST /governance/jury/accept | ✓ same | ✓ `JuryAssignment` + insert form | (CRUD; no view) | ✓ `AcceptJuryAssignment` + resp | ✓ `accept_jury_assignment.rs` | ✓ `/governance/jury/accept` | ~ implicit in mvp+decline tests |
| POST /governance/jury/decline | ✓ same | ✓ `JuryAssignment` (status flip) | (CRUD; no view) | ✓ `DeclineJuryAssignment` + resp | ✓ `decline_jury_assignment.rs` | ✓ `/governance/jury/decline` | ✓ `declining_juror_not_picked_as_own_replacement` |
| POST /governance/jury/vote | ✓ same | ✓ `JuryVote` + insert form | (tally side-effect only) | ✓ `SubmitJuryVote` + resp | ✓ `submit_jury_vote.rs` (9-step aggregation) | ✓ `/governance/jury/vote` | ~ via `report_to_modlog_golden_path` |

### Appeals + modlog (2 endpoints)

| Endpoint | Migration | Diesel | View | DTO | Handler | Route | e2e |
|---|---|---|---|---|---|---|---|
| POST /governance/appeal | ✓ `2026-04-27-000100` | ✓ `appeal.rs` | (CRUD write) | ✓ `RequestAppeal` + resp | ✓ `request_appeal.rs` (api_crud) | ✓ `/governance/appeal` | ✗ **absent** |
| GET /governance/modlog | ✓ `2026-04-15-100500` | ✓ `public_case_log.rs` | ✓ `GovernanceModlogView` | ✓ `ListGovernanceModlog` | ✓ `list_modlog.rs` (api) | ✓ `/governance/modlog` | ✗ **absent** |

### Trust / reputation (2 endpoints)

| Endpoint | Migration | Diesel | View | DTO | Handler | Route | e2e |
|---|---|---|---|---|---|---|---|
| GET /governance/reputation/me | (substrate from other migrations) | ✓ `ReputationSnapshot`, `ReputationEvent` | ✓ `ReputationSummaryView` | ✓ `GetMyReputation` + resp | ✓ `get_my_reputation.rs` | ✓ `/governance/reputation/me` (routes:530) | ✗ **absent** |
| POST /governance/endorsement | (substrate from other migrations) | ✓ `Endorsement`, `Surety`, `ReputationEvent`, `ReputationSnapshot` | ✓ `EndorsementSummaryView` | ✓ `CreateEndorsement` + resp | ✓ `create_endorsement.rs` (api_crud) | ✓ `/governance/endorsement` (routes:524) | ✗ **absent** |

---

## 2. Aggregation logic verification

The quorum-3 + simple-majority aggregation is **fully implemented** in `crates/api/api/src/governance/submit_jury_vote.rs:299–374`. At `vote_count >= case.quorum_snapshot` (default 3 of 5 per migration line 79), the handler tallies votes per decision, iterates `ALL_JURY_DECISIONS` in stable enum order, and awards the first decision reaching `threshold_count_snapshot` (also 3, lines 343–374). On panel-full deadlock (all 5 voted, no threshold met), status flips to `AdminReview` (lines 384–393). Majority-winning jurors receive `+10 JuryReliability`; misaligned receive `-5` (lines 589–622). Matches `05-mvp-and-delivery-plan.md` §6 exactly.

## 3. Reputation event emit-sites

- After jury vote (verdict reached): `crates/api/api/src/governance/submit_jury_vote.rs:999` — emits `ReputationEventSourceType::JuryVote`.
- After sanction (sponsor liability): `crates/api/api/src/governance/sponsor_liability.rs:429` — invoked from `submit_jury_vote.rs` step 8.5 when sanction inserted.
- Sponsor liability mechanic: `apply_sponsor_liability()` reduces `EndorsementStrength` per severity tier (Minor/Moderate/Severe deltas), with floor clamp (OQ-024). Snapshot recompute deferred to 15-min background job per OQ-024.

## 4. ADR conformance spot-check (cross-cutting)

| ADR | Citation | Status |
|---|---|---|
| ADR-013 EmergencyRemove (illegal content) | `crates/db_schema_file/src/enums.rs:404` (`CaseStatus::EmergencyRemove` variant) | ✓ present |
| ADR-015 GDPR pseudonym table | `crates/db_schema/src/source/governance/actor_pseudonym.rs` | ✓ present |
| Hash chain (v0 verifiability) | `crates/db_schema/src/source/governance/governance_log.rs:282` — SHA-256 digest signed ed25519 | ✓ present |
| ADR-014 outbound-only federation — SanctionNotice publish | `crates/apub/activities/src/governance/publish_sanction_notice.rs:226–334` + `enqueue_sanction_notice_activity:368–397` | ✓ present |
| ADR-014 inbound advisory `remote_sanction_notice` | `crates/db_schema/src/source/governance/remote_sanction_notice.rs:23–34` (local_case_id always NULL) | ✓ present |
| Signing key (ed25519-dalek) | `crates/db_schema/src/source/governance/governance_log.rs:62` + `Cargo.toml:35` + e2e.rs:2090 | ✓ present |
| ADR-011 AGPLv3 license | root `LICENSE` (662 lines, AGPL-3.0) | ✓ present |
| ADR-011 AGPL §13 source-disclosure notice (user-visible) | root `AGPL-NOTICE.md` exists; no `/api/v4/site` or `/about` HTTP handler returning the notice | ✗ **gap** |
| Release artifact source tarball machinery | `.github/workflows/` has `cargo-*.yml` validation only; no release target | ✗ **gap** |

## 5. Reproducible build

- `rust-toolchain.toml`: `1.95` (pinned, stable channel)
- `Cargo.lock`: tracked, 9626 lines
- Wrappers: `scripts/brehon/cargo-{check,clippy,nextest,test}.{sh,bat}` (8 files; mirror cross-platform)
- `.cargo/config.toml`: `rust-lld` linker for Windows MSVC determinism
- `docker/Dockerfile`: present
- `docker-compose.yml`: `lemmy-ui:nightly` (unpinned tag), `pictrs:0.5.17-pre.9`, `nginx:1-alpine`; **PostgreSQL not pinned** (deferred to container auto-upgrade)

## 6. Operator runbook + threat model

- `docs/.../07-operations-and-federation.md`: 332 lines, non-empty; covers deployment topology §1.1–1.3, background jobs §2, blockchain §3, federation ops §4, incident response, recovery flows
- `docs/.../06-security-and-threat-model.md`: 411 lines; STRIDE present (§1, §2, §7); FIDO2/passkey discussed; ADR-013 emergency-remove override detailed
- WebAuthn/passkey wiring: ✗ absent in `crates/` (optional per v0 — v2 per ADR-010)

---

## 7. Where the substrate stands vs §9 done-definition

Cross-walk against `05-mvp-and-delivery-plan.md` §9 "Done-definition for v0 ship":

| §9 criterion | Status |
|---|---|
| All 11 MVP endpoints return 200 on happy path | ~ Substrate present for all 11; **golden-path integration test exists (`report_to_modlog_golden_path`)** confirming flow; per-endpoint e2e coverage uneven (4 endpoints have no e2e; 4 endpoints have implicit-only coverage; 3 endpoints have endpoint probes only) |
| Integration test for full flow: report → threshold → case → jury → vote → decision → sanction → public log → modlog | ✓ `report_to_modlog_golden_path` exists |
| Integration test for sponsor-liability flow | ~ `apply_sponsor_liability` exists + emit-site wired; explicit e2e for "2 sponsors lose reputation on sanction" not confirmed (substrate is there; test name not found in audit) |
| 5 success criteria from §7 visions | (out-of-band; not auditable from grep) |
| Hash chain verifiable | ✓ signing wired (`governance_log.rs:282`); chain verification helper presence not audited this pass |
| Deployment runbook executable by non-author | ~ `07-operations-and-federation.md` exists at 332 lines; "executable by someone other than the author" is empirical |
| Threat-model reviewed + MVP mitigations in place | ~ doc exists at 411 lines; "reviewed" is empirical |

---

## 8. Production-readiness gaps — ranked

### Tier 1 — blocks shipping to first external user

1. **AGPL §13 source-disclosure not user-visible.** LICENSE + root NOTICE exist; no HTTP-endpoint or UI footer returns the notice. The fork goes private→public the moment the first external user connects ("Remote Network Interaction" clause triggers). Without a user-visible disclosure path, you ship out of compliance from day one. Fix: extend `/api/v4/site` response with `source_disclosure_url` field; ship a static `/source` route serving the notice + repo URL. Single-digit hour fix; load-bearing for "shippable".

2. **e2e coverage gaps on 4 of 11 endpoints — appeals, modlog, reputation/me, endorsement.** Substrate is wired; HTTP-shape is not exercised. The CR-block-merge-on-critical rule already caught one similar gap (3× confirmed pattern). Risk: first external user hits one of these untested endpoints; failure surfaces as a runtime trace, not a captured assertion. Fix: 4 e2e test cases following `report_to_modlog_golden_path` shape. Sub-phase-scale (1 sub-phase, 4 tasks).

3. **No release-artifact pipeline.** `.github/workflows/` validates cargo but doesn't produce a tagged build, source tarball, or signed binary. To ship to a first external user, "what artifact do they install?" needs a concrete answer. Fix: cargo-binstall-friendly release workflow; tarball with `cargo vendor`; SHA256SUMS. Sub-phase-scale.

### Tier 2 — should land before pilot

4. **`docker-compose.yml` Postgres unpinned.** Production deploy reproducibility breaks if the user pulls `postgres:latest` six months from now and the DB upgrades unexpectedly. Fix: pin `postgres:16.x` explicitly. Minutes-scale.

5. **`POST /report` returns raw `moderation_case` row instead of a prepared view.** Forces client to marshal field-by-field; breaks the field-stability contract `GET /cases` already established with `GovernanceCaseSummaryView`. Fix: introduce `CreateReportResponse` view; align with other read endpoints. Within a single task.

6. **Sponsor-liability e2e for "2 sponsors lose reputation on sanction" not confirmed.** The substrate is there, the emit-sites are wired, but the specific §9 done-definition criterion requires this test exists by name. Confirm or add. Hours-scale.

### Tier 3 — pilot polish (deferable past v0 ship)

7. **WebAuthn/passkey MFA absent.** Optional per CLAUDE.md; v2 per ADR-010. Defer to v2.
8. **Public anchoring of governance log.** Explicitly v3 per `05-mvp-and-delivery-plan.md` §7.3 "Verifiability". Defer.
9. **Operator runbook empirical-test (can a non-author execute it?).** Out-of-band; needs a friendly external user. Defer.

---

## 9. Judgment — single biggest production-readiness gap

**The fork is substrate-complete for v0 but ship-incomplete in two structural ways:** (a) AGPL §13 user-visible disclosure is not wired to any HTTP endpoint, which is a compliance-not-just-polish concern the moment one external user joins; and (b) 4 of 11 endpoints have no e2e coverage even though all 4 have wired handlers, so the "all 11 return 200 on happy path" §9 criterion is unproven for 36% of the API surface. These two gaps — neither of which is large — are the difference between "everything works on my machine" and "first external user can join without me staying on call."

The remaining tactical sub-phases (RT-r2 through RT-r6, JM-f) build feature breadth (reputation tuning, etc) without addressing the ship-gate. The structural gap is **no PRD connecting current state to ship-criteria.**

---

## 10. Next step

Step (2) — ADR conformance audit — and step (3) — v1-production-readiness PRD authoring — derive from this report.

The PRD should sequence: AGPL §13 disclosure → e2e backfill (4 endpoints) → release pipeline → Postgres pin + sponsor-liability e2e confirmation → ship gate. Estimated total: 2 sub-phases (one for AGPL+e2e backfill, one for release pipeline + final ship-gate audit). Everything past that is post-ship polish (v1 feature breadth, v2 hardening, v3 verifiability) per the existing 3-version roadmap.
