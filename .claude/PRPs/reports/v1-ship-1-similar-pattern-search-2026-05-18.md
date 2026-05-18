# v1-ship-1 — systematic similar-issue-pattern search (2026-05-18)

**Author:** advisor session (lane-dedicated, `phase-v1-ship-1`). Read-only — 4 parallel Explore subagents. **Purpose:** feed Task 5 retro carry-forward + candidate lessons. Triggered by the §G4 hard-refusal on the agpl e2e harness (DQ #255/#256).

**Root-cause class searched for:** a test/spec constructed by "mirroring" a pattern that did NOT exercise the same runtime path → passed static gates (compile/clippy/no-run) → failed only at runtime; AND the test harness hand-assembled instead of using the existing canonical fixture (`FederationConfig::to_request_data()` / `LemmyContext::init_test_context()`).

---

## Finding 1 — Hand-assembled-actix-App defect is ISOLATED (not a cluster) + STRONGLY validates the r2 re-plan

The agpl test (`crates/server/tests/e2e.rs:14918`, `agpl_source_disclosure_surface_returns_notice`) is the **ONLY** instance in the entire workspace that hand-assembles `App::new()...wrap(FederationMiddleware::new(...))` for an HTTP-route test through a `Data<LemmyContext>` extractor WITHOUT calling `.to_request_data()`.

**Decisive corroborating evidence:** every OTHER `FederationConfig::builder()` site in `e2e.rs` — **11 sites** (lines 2361, 3094, 4694, 4731, 9006, 9162, 9377, 9500, 9680, 9825, 10338) — **correctly calls `.to_request_data()` immediately after `.build().await?`.** The other `test::init_service` site (`all_mvp_endpoints_return_non_404`, e2e.rs:3857) does NOT use `FederationMiddleware` at all (no defect).

**Implication for the r2 plan:** the canonical `to_request_data()` idiom is already used 11× IN THE SAME FILE — the planner has abundant in-file byte-for-byte mirrors, not just the production `lib.rs:247` composition. This *materially de-risks* the re-plan and is the strongest possible confirmation the direction is correct: the fix is to make the agpl test do what 11 sibling tests in the same file already do. **Action:** ensure the r2 plan's §10.7 cites a representative one of these 11 `.to_request_data()` sibling sites as the byte-for-byte in-file mirror (not only `lib.rs`).

---

## Finding 2 — Status-assert-discards-body: contained, but ONE live latent instance

fix-impl-6 Part A is confirmed correctly applied to **both** agpl assertions (e2e.rs:14931 `/api/v4/site` + e2e.rs:14960 `/api/v4/source` — both read body before asserting, embed in failure message). 9/10 other status-assert points are CLEAN (read body via `read_body_json`, or SSE-stream/expected-409 cases where body is N/A).

**ONE genuine surviving anti-pattern:** `all_mvp_endpoints_return_non_404` at **e2e.rs:3897** — a 14-governance-endpoint sweep asserting `allowed.contains(&status)` while discarding the body. If any of those 14 endpoints 500s (the exact failure mode that cost this phase 5 attempts), the panic shows only the status code, not the error payload. Moderate-to-high latent risk (fresh-wiring sweep where 5xx = misconfiguration).

**Action (carry-forward, NOT a v1-ship-1 fix — out of scope per plan §"Out of scope"):** retro item — capture the response body before the loop and include it in the failure message at e2e.rs:3897. Candidate for a brief-template §4 default: "any e2e brief adding an HTTP-route status assertion MUST read+include the body on failure."

---

## Finding 3 — Plan mirror-precedent defect: one instance, a TEMPLATE GAP (the systemic root)

v1-ship-1 §10.6 defect CONFIRMED exactly: §10.6 cited `all_mvp_endpoints_return_non_404` (e2e.rs:3700-3880, a governance-route smoke test that never calls `/api/v4/site`); the impl actually mirrored `governance_outbox_emits_remote_sanction_notice_on_local_sanction` (an INTERNAL `SiteView::read_local`-only scaffold, no HTTP). Neither precedent HTTP-drives the target endpoint.

Audited **27 current plans**: no OTHER materially-misleading mirror citation (JM-d/JM-e/SL-e all correctly cite real e2e test names or explicitly-labeled "real-HTTP" patterns). **This is a one-instance manifestation of a template gap, not a recurring plan-authoring anti-pattern.**

**The systemic root:** `.claude/PRPs/templates/plan.template.md` §10 enforces *specificity* (file:line citation, concept-only rejected) but **NOT *path-congruence*** (no requirement that a mirror precedent exercise the same runtime path/endpoint/extractor as the task). The v1-ship-1 §10.6 defect passed advisor-side review *because it was specific and verifiable* — the template had no gate for "does this cited test actually call `/api/v4/site` over HTTP?"

**Action (PRIMARY carry-forward):** amend `plan.template.md` §10 to add: *"For e2e / HTTP-handler tasks: the mirror precedent MUST cite a test that exercises the SAME endpoint or handler-invocation chain at runtime. Internal/unit-only paths or cross-domain smoke tests are NOT valid precedents for an HTTP-route task."* Mechanical, planner-checkable at write-time. (Template edit = governance-v0 meta-work, separate from this lane — note for the retro, apply on trunk.)

---

## Finding 4 — Lessons corpus: 1 NEW, 1 AMEND, 1 REINFORCE + a critical "existed-but-not-applied"

| Sub-lesson | Status | Action |
|---|---|---|
| (i) mirror precedent must exercise the same runtime path | PARTIAL — `feedback_migration_invariants_full_mirror.md` covers "enumerate full invariants" but framed as file-structure, not runtime behaviour | AMEND that lesson: generalize beyond migrations to test harnesses — enumerate the **runtime endpoints/API calls the fixture exercises**, not just type-shape/imports |
| (ii) e2e status-assert discarding body = illegible failure | GAP — 0 corpus matches; `feedback_risk_reduction_8_moves_pattern.md` Move 4 prescribes body assertions but never names the *cost* of omitting them | AUTHOR new: `feedback_e2e_status_assertion_without_body_illegible.md` — thesis: "status-only assert is 80% of the assertion work, 0% of its diagnostic value on failure; always read+assert the body" |
| (iii) canonical-schema-first for a test harness = mirror a WHOLE proven unit, not a hand-reassembled subset | COVERED (strong) — `feedback_plan_stub_uniformity_with_canonical_sibling.md` + `feedback_read_canonical_before_writing_spec.md`, but both scoped to Rust type-signature uniformity, not runtime composition | REINFORCE + extend the plan-approval gate: "does the test-harness stub mirror a COMPLETE functional unit (real server App composition + canonical `init_test_context()`), or a hand-assembled subset? Require explicit canonical-fixture citation." |

**CRITICAL — existed-but-not-applied:** `pattern_test_against_reality_not_syntax.md` (PMD-promoted, MEMORY.md line 19: *"bash -n / lint / prose review insufficient"*) **already captures this exact class** (passed static gates, failed at runtime because it didn't exercise the claimed behaviour). It was NOT invoked as a harness-authoring self-check. **Single highest-value retro action:** make this pattern a *mandatory self-check gate* in any impl-task brief that adds/rebuilds a test harness — "before pushing, confirm the harness exercises the REAL endpoint at runtime; compile + --no-run passing is NOT sufficient." This single gate would have collapsed the 5-attempt chain to 1.

---

## Consolidated Task 5 retro carry-forward (additive to the existing set)

- **(vii)** Hand-assembled-actix-App defect was ISOLATED — 11 sibling `to_request_data()` sites in the same file prove the canonical idiom; the agpl test was the lone deviation. The r2 plan §10.7 should cite an in-file sibling, not only `lib.rs`. (Validates the re-plan direction with maximal confidence.)
- **(viii)** Live latent anti-pattern: `e2e.rs:3897` (`all_mvp_endpoints_return_non_404`) status-only sweep over 14 governance endpoints — body-on-failure fix is a post-ship hygiene item (out of v1-ship-1 scope).
- **(ix) PRIMARY:** `plan.template.md` §10 gap — enforces specificity, NOT runtime-path-congruence of mirror precedents. Amend §10 with the path-congruence requirement (trunk meta-work).
- **(x)** Lessons: AUTHOR `feedback_e2e_status_assertion_without_body_illegible.md`; AMEND `feedback_migration_invariants_full_mirror.md` (generalize to test-harness runtime behaviour); REINFORCE `feedback_plan_stub_uniformity_with_canonical_sibling.md` at the plan-approval gate.
- **(xi) HIGHEST-VALUE:** `pattern_test_against_reality_not_syntax.md` existed and would have prevented the entire 5-attempt chain — make it a mandatory self-check gate in any test-harness impl-task brief ("confirm the harness exercises the REAL endpoint at runtime before push; static gates insufficient"). Candidate: add to `advisor-orchestrator.md` §2.4 mandatory-file-class-lesson injection for `crates/server/tests/e2e.rs` edits.

(These are additive to the prior set: plan-§10.7-mirror-precedent / e2e-status-discards-body / canonical-schema-first-whole-unit / cherry-pick-x-finalize / CAS-sync-daemon-ref / Junior-worktree-no-submodule-init / advisor-unverified-framework-assumption / clippy::unused_async-allowlist / parallel-cohort-DQ-collision / daemon-multi-lane-ref-isolation / post-finalize-DQ-resurrection-WORKED / §5.2-win-bat-chain / e2e-rerun-recompiles-on-workspace-fingerprint / AD-e-cross-lane-TOCTOU+CAS / fix_impl_pre_push_cargo_check-escape-hatch-validated / project-memory-junction-gap-Task#6.)
