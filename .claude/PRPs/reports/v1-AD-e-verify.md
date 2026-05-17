# Verify report — v1-AD-e (post-CR-fix re-run)

**Run at:** 2026-05-17T12:25:00Z
**Phase branch:** `phase-v1-AD-e` @ `9f10b0178`
**Plan:** `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` @ `9f10b0178`
**Outcome summary:** 3 stories: 3✓ 0✗-phantom 0✗-regression 0[malformed]
**Trigger:** re-verify after PR #133 CR fix-in-PR (cr-1/cr-4/cr-5/cr-6
addressed in `5d742a323`, recovered from Junior #297). Supersedes the
pre-fix verify run (same 3✓ outcome; this run confirms the cr-4 handler
reorder did not regress any story).

---

## §13 task-commit reconciliation (pre-flight)

| Task | Commit on `phase-v1-AD-e` | Status |
|---|---|---|
| Task 0 — pre-flight harness audit | (none — read-only by plan §13 design) | ✓ expected |
| Task 1 — maud engine dep | `55c373016` | ✓ |
| Task 2 — gather_dashboard extract + dashboard HTML + route | `ade904851` | ✓ |
| Task 3 — /audit/view route + admin_audit_html import | `5afe5b378` | ✓ |
| Task 4 — audit live-tail `<script>` | (none — PRE-SATISFIED in Task-2 `ade904851` via scope-bleed; verified prior) | ✓ expected |
| Task 5 — e2e test | `460214d5e` | ✓ |
| CR fix-in-PR (cr-1/4/5/6) | `5d742a323` (recovered from Junior #297; validated DQ #245 result=pass) | ✓ |

All §13 task commits present. The CR-fix commit `5d742a323` was
recovered from Junior #297 (worker did the work correctly but exited
without push; advisor recovered via daemon recovery-ref + clean
cherry-pick — zero advisor-file deletions). DQ #245
(validate-pending-laptop) `result: pass`: cargo-check + clippy +
test--no-run + FULL e2e all green.

---

## Story 1 — An instance admin sees the governance dashboard as a web page

- **Composing tasks:** Task 1, Task 2
- **Outputs:**
  - ✓ `crates/api/api/src/governance/admin_dashboard_html.rs` exists, non-empty, `pub async fn admin_dashboard_html`
  - ✓ `crates/api/api/src/governance/admin_dashboard.rs` `pub(crate) async fn gather_dashboard`
  - ✓ `crates/api/api/src/governance/mod.rs` `mod admin_dashboard_html`
  - ✓ `crates/api/routes/src/lib.rs` `.route("/dashboard/view"` (line 546)
- **Checkpoint:** ✓ exit 0 — reused the DQ #245 full e2e run
  (`cargo-test --workspace --test e2e --features full`, advisor-laptop,
  Docker, 1925.48s): `test result: ok. 94 passed; 0 failed; 5 ignored`.
  `admin_dashboard_html_returns_html_for_admin` **ok**,
  `admin_dashboard_html_forbidden_for_non_admin` **ok**.
- **Outcome:** ✓

## Story 2 — An instance admin watches config changes live in a web page

- **Composing tasks:** Task 3, Task 4
- **Outputs:**
  - ✓ `admin_dashboard_html.rs` `pub async fn admin_audit_html`
  - ✓ `admin_dashboard_html.rs` `EventSource(`
  - ✓ `admin_dashboard_html.rs` `addEventListener('admin_config_changed'` (single-quoted in source — quote-agnostic check; §18 SSE-footgun closed: named events, NOT `onmessage`)
  - ✓ `admin_dashboard_html.rs` `addEventListener('admin_config_change_denied'`
  - ✓ `crates/api/routes/src/lib.rs` `/audit/view` route — **present as a nested scope**: `scope("/audit").route("/stream", ...).route("/view", get().to(admin_audit_html))` (lib.rs:559-561), `admin_audit_html` imported at lib.rs:42
- **Note (descriptor-vs-impl wording, NOT a phantom):** the §16a
  Brief-Scope descriptor reads `contains .route("/audit/view"` (a
  grep-literal). The actual implementation composes the path via
  `scope("/audit").route("/view", ...)` — the idiomatic actix nesting,
  functionally identical and proven by the passing
  `admin_audit_html_*` e2e tests. This is a plan-wording imprecision
  (a grep-literal that doesn't match the idiomatic scope structure),
  the SAME disposition the pre-fix verify run accepted. NOT a
  `[malformed]` planner-escalation (output present + behaviour proven;
  the descriptor grammar, not the output, is imprecise) and NOT a
  phantom (route exists + wired + e2e-green).
- **Checkpoint:** ✓ exit 0 — same e2e run.
  `admin_audit_html_returns_html_for_admin` **ok**,
  `admin_audit_html_forbidden_for_non_admin` **ok** (the new cr-6
  test).
- **Outcome:** ✓

## Story 3 — The pages can be disabled instance-wide

- **Composing tasks:** Task 2 + Task 3 (the `html_pages_enabled` 404
  gate in both handlers), Task 5 (the flag-off assertion)
- **Outputs:**
  - ✓ `admin_dashboard_html.rs` reads `governance.dashboard.html_pages_enabled` (`HTML_PAGES_KEY`)
  - ✓ `admin_dashboard_html.rs` has a `NotFound()` (404) path — exactly **2** occurrences (one per handler: `admin_dashboard_html` + `admin_audit_html`), confirming both pages gate identically (R-html-3: 404 not 403)
- **cr-4 verification (this re-run's purpose):** the CR fix moved
  `is_admin(&local_user_view)?;` to AFTER the
  `if !enabled { return NotFound }` block in BOTH handlers, so a
  non-admin + flag-off path now returns **404 (not 403)** — closing
  the R-html-3 leak. The e2e `admin_html_pages_flag_off_returns_404`
  test **still passes** (94/0/5, 0 failed) — the reorder did NOT
  regress the existing flag-off-404 behaviour.
- **Checkpoint:** ✓ exit 0 — same e2e run.
  `admin_html_pages_flag_off_returns_404` **ok**.
- **Outcome:** ✓

---

## Required actions

None — all 3 stories ✓. No phantoms, no regressions, no malformed
§16a entries. The cr-4 handler reorder is regression-free (the
flag-off-404 + both non-admin tests still pass; +1 net test from
cr-6's new `admin_audit_html_forbidden_for_non_admin`). Merge-confirm
gate is CLEAR.

## Notes

- **CR-fix re-verify:** this run exists to confirm the post-CR-fix
  phase tip still satisfies all 3 stories. It does. The cr-4
  behaviour change (flag-check-before-auth) is exactly Story 3's
  R-html-3 intent — the e2e proves both the existing flag-off-404
  and the new non-admin paths hold.
- **cr-5 PARTIAL (carried to user gate 5, NOT a verify finding):**
  the CR fix applied canonical `crate::governance::redaction::scrub`
  to `e.reason` + `e.denial_reason` only (admin_dashboard_html.rs:323,
  331). The fix-impl-1 brief §2.2 listed a broader ADR-015 field set
  (scope/key/entry_kind/actor_pseudonym) + `scrub_json` for
  `previous_value`/`new_value`. This is a SCOPE-completeness question
  for the user at merge-confirm (gate 5), not a story phantom — Story
  2's verifiable outputs (audit handler + EventSource + route) are all
  present and the e2e is green. The triage YAML records cr-5
  `addressed_in=5d742a323` with a `notes:` documenting the partial
  scope. **The advisor does not adjudicate cr-5 sufficiency — gate 5
  does.**
- **Checkpoint reuse:** all 3 stories share the full e2e binary. It
  ran once at DQ #245 (advisor-laptop, Docker, 1925.48s, 94 passed /
  0 failed / 5 ignored, all 5 v1-AD-e tests `ok`) and is reused here
  rather than re-running a ~32-min e2e — the single run covers all
  three stories' sub-assertions and the cr-4 regression guard.
- **Task-4 pre-satisfied** disposition (Task-2 scope-bleed) unchanged
  and re-confirmed: Story 2's audit outputs all present + conformant.
