# Verify report — v1-ship-1-r2

**Run at:** 2026-05-18T14:01:44Z
**Phase branch:** `phase-v1-ship-1` @ `3cd53b5da67cfe51dc069eebf99fa61e77ae97f0`
**Plan:** `.claude/PRPs/plans/v1-ship-1-r2.plan.md` @ `5c02567d0c95e5e5d2ec52a854aa55a851dd5310`
**Outcome summary:** 3 stories: 3✓ 0✗-phantom 0✗-regression 0[malformed]

> Note: r2 is a re-plan that rebuilds ONLY the failed e2e test. Stories 1+2
> (Tasks 1-3) are SATISFIED/merged carry-forward — their checkpoints are N/A
> (Phase-1 §5.2 PASSING on phase tip per DQ #254 evidence). Story 3 (Task 6)
> is the only LIVE story; its checkpoint is the Phase-2 e2e run.

---

## Story 1 — First-touch handshake exposes source-disclosure pointer

- **Composing tasks:** Task 1 (DTOs — merged), Task 2 (field + handler populate + build.rs — merged)
- **Outputs:** N/A — SATISFIED on phase-v1-ship-1; field type-correct + populated (DQ #254 Phase-1 §5.2 PASS)
- **Checkpoint:** N/A (merged carry-forward; Phase-1 §5.2 PASSING on phase tip)
- **Outcome:** ✓ (merged, carry-forward)

## Story 2 — Disclosure URL resolves to the AGPL notice body

- **Composing tasks:** Task 3 (handler + route + mod wiring — merged)
- **Outputs:** N/A — SATISFIED on phase-v1-ship-1
- **Checkpoint:** N/A (merged carry-forward; Phase-1 §5.2 PASSING on phase tip)
- **Outcome:** ✓ (merged, carry-forward)

## Story 3 — Named e2e test covers both surfaces with Case A discipline AND the verified-correct App-construction

- **Composing tasks:** Task 6 (the e2e App-construction rebuild — commit `38af8b866`)
- **Outputs (12 Brief-Scope structural patterns, all checked against `origin/phase-v1-ship-1:crates/server/tests/e2e.rs`):**
  - ✓ P1 signature preserved (`async fn agpl_source_disclosure_surface_returns_notice() -> lemmy_utils::error::LemmyResult<()>`)
  - ✓ P2 `let federation_config = FederationConfig::builder()` (§10.5 mirror)
  - ✓ P3 `let inner_context: LemmyContext = federation_config.deref().clone();` (§10.6 lib.rs:364 byte-mirror — the un-double-wrap)
  - ✓ P4 `.app_data(Data::new(inner_context.clone()))` (§10.7 lib.rs:379 mirror — **THE FIX**)
  - ✓ P5 `.wrap(FederationMiddleware::new(federation_config.clone()))` (lib.rs:380 mirror)
  - ✓ P6 `.wrap(IdempotencyMiddleware::new(idempotency_set.clone()))` (lib.rs:381 mirror)
  - ✓ P7 `.wrap(SessionMiddleware::new(inner_context.clone()))` (lib.rs:382 mirror)
  - ✓ P8 BOTH `test::TestRequest::get().uri("/api/v4/site")` AND `…uri("/api/v4/source")` asserted
  - ✓ P9 `governance_fixtures::bootstrap()` present (DQ #226 chosen)
  - ✓ P10 `admin_config_fixtures::bootstrap` ABSENT (DQ #226 rejected sibling — correctly absent)
  - ✓ P11 no `.to_request_data()` as `.app_data(...)` source (DQ #261 type-mismatch — correctly absent)
  - ✓ P12 Case A discipline (no `Box<dyn Error>`, no `.map_err(|e| anyhow::anyhow!(…))?` bridge)
- **Checkpoint:** ✓ exit 0 — Phase-2 full e2e suite (`.claude/runlog/e2e-v1-ship-1-r2-6a2d50803.log`, bg `bv6gh9s2f`, E2E_EXIT_0): `test agpl_source_disclosure_surface_returns_notice ... ok`; `test result: ok. 90 passed; 0 failed; 5 ignored; finished in 1939.35s`. The `--exact` single-test checkpoint is subsumed by the full-suite superset run (DQ #263 PASS).
- **Outcome:** ✓ (all 12 structural patterns matched + checkpoint exit 0; NOT phantom — App-construction is the new verified-correct shape, not the old failed double-wrap)

---

## Required actions

None — all stories ✓. Advance to merge-confirm user gate.

The R11/DQ #261 type-precision fix resolves the 5-cycle HTTP 500
(`Data<Data<LemmyContext>>` double-wrap → `actix-Data-500` / "Requested
application data is not configured"). The acceptance test
`agpl_source_disclosure_surface_returns_notice` now returns HTTP 200
with the AGPL §13 source-disclosure notice on both `/api/v4/site` and
`/api/v4/source`.
