# Verify report — v1-AD-e (post-fix-impl-2 re-run, cr-5 ADR-015 COMPLETE)

**Run at:** 2026-05-17T19:18:00Z
**Phase branch:** `phase-v1-AD-e` @ `477f6e549`
**Plan:** `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` @ `477f6e549`
**Outcome summary:** 3 stories: 3✓ 0✗-phantom 0✗-regression 0[malformed]
**Trigger:** re-verify after fix-impl-2 (`5805ab27f`, recovered from
Junior #299 — 5th #292 stale-base rescue) completed the cr-5 ADR-015
scrub. Supersedes the post-CR-fix run (`7de5bfe45`, same 3✓ outcome).
This run confirms the additional `scrub()`/`scrub_json()` calls in
`audit_entry_row` did not regress any story (render-only change;
validated by DQ #246 full e2e: 94 passed / 0 failed).

---

## §13 task-commit reconciliation (pre-flight)

| Task | Commit on `phase-v1-AD-e` | Status |
|---|---|---|
| Task 0 — pre-flight harness audit | (none — read-only by plan §13 design) | ✓ expected |
| Task 1 — maud engine dep | `bcade7d94` (finalize-merge junior-287) | ✓ |
| Task 2 — gather_dashboard extract + dashboard HTML + route | `a0a19637d` (finalize-merge junior-288) | ✓ |
| Task 3 — /audit/view route + admin_audit_html import | `4003f6ff5` (finalize-merge junior-289) | ✓ |
| Task 4 — audit live-tail `<script>` | (none — PRE-SATISFIED in Task-2 via scope-bleed; verified prior) | ✓ expected |
| Task 5 — e2e test | `cf9a5c6b1` (finalize-merge junior-290) | ✓ |
| CR fix-in-PR cycle 1 (cr-1/4/5-partial/6) | `5d742a323` (recovered from Junior #297) | ✓ |
| CR fix-in-PR cycle 2 (cr-5 ADR-015 completion) | `5805ab27f` (recovered from Junior #299; validated DQ #246 result=pass) | ✓ |

All §13 task commits present. Both CR-fix commits are on the branch.
`5d742a323` (fix-impl-1, recovered from Junior #297) addressed
cr-1/cr-4/cr-6 + cr-5-partial (reason+denial_reason scrub).
`5805ab27f` (fix-impl-2, recovered from Junior #299 — the 5th #292
stale-base-self-merge rescue; clean cherry-pick, zero advisor-file
deletions) completed cr-5's ADR-015 coverage. DQ #246
(validate-pending-laptop) `result: pass`: cargo-check 0 err +
clippy -D warnings 0 warn + test--no-run + FULL e2e 94 passed /
0 failed.

---

## Story 1 — An instance admin sees the governance dashboard as a web page

- **Composing tasks:** Task 1, Task 2
- **Outputs:**
  - ✓ `crates/api/api/src/governance/admin_dashboard_html.rs` exists, non-empty, `pub async fn admin_dashboard_html` (L68)
  - ✓ `crates/api/api/src/governance/admin_dashboard.rs` `pub(crate) async fn gather_dashboard`
  - ✓ `crates/api/api/src/governance/mod.rs` `mod admin_dashboard_html`
  - ✓ `crates/api/routes/src/lib.rs` `.route("/dashboard/view"` (line 546)
- **Checkpoint:** ✓ exit 0 — reused the DQ #246 full e2e run
  (`cargo-test --workspace --test e2e --features full`, advisor-laptop,
  Docker, 2017.49s): `test result: ok. 94 passed; 0 failed; 5 ignored`.
  `admin_dashboard_html_returns_html_for_admin` **ok**,
  `admin_dashboard_html_forbidden_for_non_admin` **ok**.
- **Outcome:** ✓

## Story 2 — An instance admin watches config changes live in a web page

- **Composing tasks:** Task 3, Task 4
- **Outputs:**
  - ✓ `admin_dashboard_html.rs` `pub async fn admin_audit_html` (L242)
  - ✓ `admin_dashboard_html.rs` `EventSource(` (L37)
  - ✓ `admin_dashboard_html.rs` `addEventListener('admin_config_changed'` (L56 — single-quoted in source; quote-agnostic check; §18 SSE-footgun closed: named events, NOT `onmessage`)
  - ✓ `admin_dashboard_html.rs` `addEventListener('admin_config_change_denied'` (L59)
  - ✓ `crates/api/routes/src/lib.rs` `/audit/view` route — **present as a nested scope**: `scope("/audit").route("/stream", ...).route("/view", get().to(admin_audit_html))` (lib.rs:559-561), `admin_audit_html` imported at lib.rs:42
- **Note (descriptor-vs-impl wording, NOT a phantom — unchanged from
  prior runs):** the §16a Brief-Scope descriptor reads
  `contains .route("/audit/view"` (a grep-literal). The actual
  implementation composes the path via
  `scope("/audit").route("/view", ...)` — the idiomatic actix
  nesting, functionally identical and proven by the passing
  `admin_audit_html_*` e2e tests. This is a plan-wording imprecision
  (a grep-literal that doesn't match the idiomatic scope structure),
  the SAME disposition the prior two verify runs accepted. NOT a
  `[malformed]` planner-escalation (output present + behaviour
  proven) and NOT a phantom (route exists + wired + e2e-green).
- **cr-5 ADR-015 scrub-completion verification (this re-run's
  purpose):** fix-impl-2 (`5805ab27f`) extended canonical
  `crate::governance::redaction::{scrub, scrub_json}` to all
  identifier-bearing `audit_entry_row` columns: `scrub(&e.entry_kind)`,
  `scrub(&e.scope)`, `scrub(&e.key)`, `scrub(p)` (actor_pseudonym
  Some-arm), `scrub_json(v).to_string()` (previous_value Some-arm),
  `scrub_json(&e.new_value).to_string()`. `e.reason` + `e.denial_reason`
  remain single-scrub (done by fix-impl-1 `835681ff5`, NOT
  double-scrubbed). id/created_at/signature non-PII left raw by
  design. The `admin_audit_html_returns_html_for_admin` e2e test —
  which renders `audit_entry_row` end-to-end — **passes**, proving
  the `scrub_json().to_string()` shape change does not break the
  audit-page render.
- **Checkpoint:** ✓ exit 0 — same e2e run.
  `admin_audit_html_returns_html_for_admin` **ok**,
  `admin_audit_html_forbidden_for_non_admin` **ok** (the cr-6 test).
- **Outcome:** ✓

## Story 3 — The pages can be disabled instance-wide

- **Composing tasks:** Task 2 + Task 3 (the `html_pages_enabled` 404
  gate in both handlers), Task 5 (the flag-off assertion)
- **Outputs:**
  - ✓ `admin_dashboard_html.rs` reads `governance.dashboard.html_pages_enabled` (`HTML_PAGES_KEY` L28; consumed L74 + L248)
  - ✓ `admin_dashboard_html.rs` has a `NotFound()` (404) path — exactly **2** occurrences (L76 `admin_dashboard_html` + L250 `admin_audit_html`), confirming both pages gate identically (R-html-3: 404 not 403)
- **cr-4 disposition (carried, unchanged from prior run):** the
  fix-impl-1 cr-4 fix moved `is_admin(&local_user_view)?;` to AFTER
  the `if !enabled { return NotFound }` block in BOTH handlers, so a
  non-admin + flag-off path returns **404 (not 403)** — closing the
  R-html-3 leak. fix-impl-2 touched ONLY `audit_entry_row` (the
  render helper), NOT the handler flag/auth ordering — so the cr-4
  fix is untouched and the e2e `admin_html_pages_flag_off_returns_404`
  test **still passes** (94/0/5, 0 failed).
- **Checkpoint:** ✓ exit 0 — same e2e run.
  `admin_html_pages_flag_off_returns_404` **ok**.
- **Outcome:** ✓

---

## Required actions

None — all 3 stories ✓. No phantoms, no regressions, no malformed
§16a entries. fix-impl-2 is a render-only change to `audit_entry_row`
(adds `scrub`/`scrub_json` redaction); it does not move any story's
verifiable output and does not regress the cr-4 handler reorder. The
full e2e (94 passed / 0 failed, all 5 v1-AD-e tests `ok`) is the
proof. **Merge-confirm gate is CLEAR.**

## Notes

- **fix-impl-2 re-verify:** this run exists to confirm the
  post-fix-impl-2 phase tip still satisfies all 3 stories AND that
  the cr-5 ADR-015 scrub-completion is render-correct. It does. The
  critical signal is `admin_audit_html_returns_html_for_admin` — it
  renders `audit_entry_row` with the new `scrub_json().to_string()`
  calls; passing proves the shape is correct (a `.to_string()` shape
  bug on the `serde_json::Value` payload would fail this test at
  render).
- **cr-5 NOW FULLY ADR-015-COMPLETE (resolves the prior gate-5
  caveat):** the prior verify run carried cr-5 as a PARTIAL/gate-5
  decision. The user chose "complete cr-5 first" at gate 5. fix-impl-2
  applied the full ADR-015 field set. The triage YAML
  (`pr-133-findings.yaml` @ `477f6e549`) records cr-5
  `addressed_in=5d742a323` with a `notes:` documenting the
  two-commit complete coverage; recommendation stays `approve`. cr-5
  is no longer a carried caveat — it is a closed finding.
- **Checkpoint reuse:** all 3 stories share the full e2e binary. It
  ran once at DQ #246 (advisor-laptop, Docker, 2017.49s, 94 passed /
  0 failed / 5 ignored, all 5 v1-AD-e tests `ok`) and is reused here
  rather than re-running a ~34-min e2e — the single run covers all
  three stories' sub-assertions, the cr-4 regression guard, and the
  fix-impl-2 scrub-render proof.
- **Task-4 pre-satisfied** disposition (Task-2 scope-bleed) unchanged
  and re-confirmed: Story 2's audit outputs all present + conformant.
- **#292 stale-base rescue (5th occurrence):** fix-impl-2 was
  recovered from Junior #299 via clean cherry-pick of the 1-file fix
  commit onto the live phase tip (zero advisor-file deletions). The
  recurring daemon-local-ref bug is a headline retro item; it did
  NOT affect the correctness of the shipped fix (spot-checked
  byte-level against brief §2.2).
