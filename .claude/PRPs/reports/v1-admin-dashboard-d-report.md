# Implementation Report — v1-AD-d Dashboard aggregate + SSE audit stream

**Plan**: `.claude/PRPs/plans/v1-admin-dashboard-d.plan.md`
**Source**: v1 post-MVP sub-phase; ADR-010 (v1 staged releases). Closes the v1-AD admin-dashboard keystone API surface.
**Branch**: `phase-v1-AD-d`
**Date**: 2026-04-23
**Status**: COMPLETE

---

## Summary

Ship two new read-only governance admin endpoints:

- `GET /api/v4/governance/admin/dashboard` — single-round-trip aggregate across six widgets (active cases, jury queue, recent config changes, federation, reputation, rule-sets) with a top-level `calculated_at` stamp.
- `GET /api/v4/governance/admin/audit/stream` — hand-rolled Server-Sent Events endpoint wrapping Postgres `LISTEN governance_events`, filtered to the two `admin_config_*` entry kinds.

Zero migrations, zero new governance_log entry kinds, zero writes. Pure consumers of the v1-AD-a/b/c substrate.

---

## Assessment vs Reality

| Metric | Predicted | Actual | Reasoning |
|---|---|---|---|
| Complexity | MEDIUM | MEDIUM | SSE + LISTEN was novel but the plan's skeleton closely matched the implementation; the only "surprise" was the LemmyErrorType status-code mapping for the 409 case |
| Confidence | 8.0/10 | 9/10 in hindsight | Everything the plan predicted came true; the one real issue (409 via LemmyErrorType would map to 400) was caught via self-review before task 6 tests, not via test failure |

**Deviations from plan:**

1. **Per-admin cap 409 path**: the plan's skeleton (§10 SSE_HAND_ROLLED_STREAM) used `actix_web::error::ErrorConflict(...)`. I initially used `LemmyErrorType::Unknown(...)` which maps to HTTP 400 — not the 409 the acceptance criterion (§16) requires. Fixed at task 6 by returning `HttpResponse::Conflict().body(...)` directly on the duplicate-connection path, bypassing `LemmyErrorType`. This is pragmatic — adding a dedicated `LemmyErrorType::SseStreamAlreadyOpen` variant would touch the shared error enum across the workspace for one handler.

2. **e2e SSE "emits on admin_config_changed" test** (initially swapped, then reinstated on advisor review): the plan §14 specified this as one of 5 tests, requiring an in-process actix HTTP server + `reqwest::Client::get(...).bytes_stream()`. I initially swapped it for `admin_audit_stream_forbidden_for_non_admin`, reasoning that the NOTIFY substrate was already validated by `governance_events_notify_fires` and the SSE body was pure logic. **Advisor review on 2026-04-23 reversed this**: the substrate test does NOT cover (a) the `kind == ADMIN_CONFIG_CHANGED || CHANGE_DENIED` filter branch at `admin_audit_stream.rs:181-185`, (b) the `entry_id` → `governance_log` row hydration at `:186-200`, or (c) the SSE frame-format assertion `event: X\ndata: Y\n\n` per HTML5 §9.2.4 at `:203-205`. A new test `admin_audit_stream_emits_frame_on_config_change` was added in a follow-up commit that exercises all three end-to-end in ~150 lines using `MessageBody::poll_next` on the `HttpResponse` body — no in-process HTTP server, no new dev-deps, mirrors the `governance_events_notify_fires` bridge pattern. Final test count on phase-v1-AD-d: **6 tests** (3 dashboard + 3 SSE, all passing). Policy captured in DQ #45 resolution.

3. **`once_cell` vs `OnceLock`**: the plan's §10 snippet used `once_cell::sync::Lazy`; task 4 GOTCHA permitted `std::sync::OnceLock<Mutex<...>>` as a no-new-dep alternative. I used `OnceLock` since `once_cell` wasn't already a workspace dep (checked per task 4).

---

## Tasks Completed

| # | Task | Files | Status |
|---|---|---|---|
| 0 | Pre-phase harness audit + branch check | `.claude/PRPs/reports/v1-AD-d-task0-audit.md` | ✅ (pre-existing) |
| 1 | Extract `project_to_audit_entry` to shared module | `crates/api/api/src/governance/audit_projection.rs` (CREATE), `crates/api/api/src/governance/admin_config.rs` (UPDATE), `crates/api/api/src/governance/mod.rs` (UPDATE) | ✅ (commit `1a190dd71`) |
| 2 | Dashboard DTOs | `crates/api/api_common/src/governance.rs` (UPDATE) | ✅ (commit `a0fd5d911`) |
| 3 | `admin_dashboard` handler | `crates/api/api/src/governance/admin_dashboard.rs` (CREATE), `mod.rs` (UPDATE) | ✅ (commit `be9fae19e`) |
| 4 | async-stream + reqwest stream feature | `Cargo.toml`, `Cargo.lock`, `crates/api/api/Cargo.toml` | ✅ (commit `521703715`) |
| 5 | `admin_audit_stream` SSE + routes | `crates/api/api/src/governance/admin_audit_stream.rs` (CREATE), `mod.rs`, `crates/api/routes/src/lib.rs` | ✅ (commit `f9b6ed8dd`) |
| 6 | 5 e2e tests + 409 fix | `crates/server/tests/e2e.rs` (+302 lines), `admin_audit_stream.rs` (409 fix) | ✅ (commit `674231b4d`) |
| 6b | Add 6th e2e test — live SSE emission on `admin_config_changed` (advisor review follow-up per §Deviation 2) | `crates/server/tests/e2e.rs` (+~150 lines) | ✅ (commit pending; see §Deviation 2) |

---

## Validation Results

| Check | Result | Details |
|---|---|---|
| `cargo check --workspace --features full` | ✅ | exit 0, zero errors |
| `cargo clippy --workspace --features full --no-deps -- -D warnings` | ✅ | exit 0, zero warnings (matches v1-AD-c baseline) |
| `cargo test --test e2e admin_dashboard -p lemmy_server` | ✅ | 3/3 pass in 108.87s (post task-6b re-run) |
| `cargo test --test e2e admin_audit_stream -p lemmy_server` | ✅ | 3/3 pass in 115.80s (post task-6b re-run; includes new emission test) |
| `cargo test --test e2e -p lemmy_server` (full regression, pre task-6b) | ✅ | 42 passed, 0 failed, 3 ignored (all pre-existing known flakes per GH #42/#43/#45) in 955s |
| Migration round-trip | ⏭️ | N/A — zero migrations added |
| Registry invariant (Level 5) | ✅ | ENTRY_KIND_* count = 26 (unchanged); shim re-export parity = 26; zero duplicate literals |
| Cross-cutting verification (Level 6) | ✅ | Zero `match.*CaseStatus` in new files; zero `governance_log::append` in new files |

---

## Files Changed

| File | Action | Lines |
|---|---|---|
| `Cargo.toml` (root) | UPDATE | +2 (async-stream dep, reqwest stream feature) |
| `Cargo.lock` | UPDATE | autogen |
| `crates/api/api/Cargo.toml` | UPDATE | +4 (futures-util, async-stream, tokio, tokio-postgres) |
| `crates/api/api/src/governance/mod.rs` | UPDATE | +1 (admin_audit_stream module) |
| `crates/api/api/src/governance/admin_audit_stream.rs` | CREATE | +213 |
| `crates/api/routes/src/lib.rs` | UPDATE | +4 (imports + 2 route registrations) |
| `crates/server/tests/e2e.rs` | UPDATE | +302 (5 new tests + comment banner) in commit `674231b4d`; +~150 (6th emission test) in pending follow-up commit (task 6b) |

Tasks 1–3 (audit-projection extraction, DTOs, dashboard handler) landed in earlier commits on the same phase branch.

---

## Cross-Cutting Impact

- [x] Hash chain appends wired up for new writes — **N/A** (v1-AD-d writes zero bytes)
- [x] `actor_pseudonym::get_or_create` used (no direct `person_id`) — **N/A** (read-only; pseudonyms come through the shared projection helper)
- [x] `redaction::scrub` called on all strings reaching the log — **N/A** (no log writes)
- [x] `CaseStatus::EmergencyRemove` exhaustively matched — **N/A** (filter uses string comparison on `status::text`, not a Rust `match` on the enum)
- [x] AGPL notice unchanged — yes (no release artefacts produced)

---

## Issues Encountered

1. **`LemmyErrorType::Unknown` maps to HTTP 400, not 409**: caught during self-review before writing tests. Fixed by returning `HttpResponse::Conflict().body(...)` directly on the duplicate-connection path. Captured in the task-6 commit message.

2. **`diesel-async` `.first()` expected `&mut _`, got `DbConn<'_>`**: small compile error during task 5. Fixed by adding `mut` to the `let Ok(conn)` pattern and `&mut conn` to the `.first()` call. Trivial — the pooled connection wrapper needs a mutable reference for the trait call.

3. **clippy `redundant_closure_for_method_calls`**: `.and_then(|v| v.as_i64())` → `.and_then(serde_json::Value::as_i64)`. Fixed in task 5 before commit.

4. **Docker not running during first test attempt**: not a code issue. Tests compile fine without Docker; they just can't execute. Started Docker, all 5 tests pass, plus the full 42-test regression.

---

## Tests Written

| Test | Validates |
|---|---|
| `admin_dashboard_returns_aggregate_for_admin` | Happy path; all 6 widgets populate with defaults on zero-row DB; `calculated_at` within request window |
| `admin_dashboard_forbidden_for_non_admin` | Capability gate; ADR-008 compliance (no `governance_log` entry on rejection) |
| `admin_dashboard_aggregates_populated_data` | Data fidelity with 3 cases across 3 statuses, 1 attestation, 1 rule-set version; `total_active` excludes `Decided`; `per_community.active_version_id = None` without seeded config |
| `admin_audit_stream_forbidden_for_non_admin` | Capability gate on SSE handler |
| `admin_audit_stream_enforces_per_admin_cap` | 1st connection 200 + text/event-stream; 2nd concurrent same-admin connection 409 Conflict; 3rd connection after 1st dropped succeeds (proves `SseGuard::Drop` releases the slot) |
| `admin_audit_stream_emits_frame_on_config_change` (task 6b — advisor-review follow-up) | End-to-end live SSE emission: (1) initial `event: retry\ndata: 10000\n\n` frame, (2) `admin_set_config` INSERT fires `governance_events` NOTIFY, (3) handler's LISTEN→filter→row-hydration→`project_to_audit_entry` produces `event: admin_config_changed\ndata: {json}\n\n` within 10s, (4) JSON payload matches projected `AdminConfigAuditEntry` shape (key, scope, value_type, new_value, entry_kind, id, created_at) |

---

## Next Steps

- [ ] Open PR `phase-v1-AD-d` → `governance-v0` via `/prp-pr` or the bm skill
- [ ] CodeRabbit review (auto-fires on non-draft PRs to `governance-v0`)
- [ ] Triage CR findings per `feedback_pr_review_triage_pattern.md`
- [ ] Mark v1-AD-d as DONE in IMPLEMENTATION-PLAN-v0.md §3 phase status (separate commit in the homeserver repo)
- [ ] Update advisor memory: `project_v1_AD_d_closed.md` with carry-forward issues (if any)
- [ ] v1-AD-e (askama HTML pages) is unblocked — it consumes `AdminDashboardResponse` shipped here and the `admin_audit_stream` endpoint via client-side `EventSource`

---

## v1-AD keystone status after this sub-phase

| Endpoint | Status | Origin |
|---|---|---|
| `POST /admin/assign-jury` | ✅ shipped | v0 Phase 4 |
| `POST /admin/close-case` | ✅ shipped | v0 Phase 4 |
| `GET /admin/reputation-stats` | ✅ shipped | v0 Phase 5a |
| `POST /admin/config` | ✅ shipped | v1-AD-b |
| `GET /admin/config` | ✅ shipped | v1-AD-b |
| `GET /admin/config/audit` | ✅ shipped | v1-AD-b |
| `POST /admin/rule-sets` | ✅ shipped | v1-AD-c |
| `GET /admin/rule-sets` | ✅ shipped | v1-AD-c |
| **`GET /admin/dashboard`** | **✅ shipped** | **v1-AD-d (this)** |
| **`GET /admin/audit/stream`** | **✅ shipped** | **v1-AD-d (this)** |
| Askama HTML pages | ❌ DEFERRED | OQ-V1-AD-01 → v1-AD-e / v1.x |

The v1-AD admin-dashboard keystone **API surface is now complete**. v1-AD-e is pure templating.
