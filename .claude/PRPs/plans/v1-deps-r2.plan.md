# Plan: v1-deps-r2 — inline the webmention sender, retire the legacy rustls-webpki chain

> **Scope was re-verified live on 2026-06-04 at un-deferral (per the brief's own mandate).**
> The brief (`v1-deps-r2-planning-1.md`, frozen 2026-05-25) specced **4 tasks closing 6 of 18
> alerts**. Live Dependabot + lockfile + git verification collapses that to **1 actionable task
> closing 3 of 6 open alerts**. The reconciliation is in §2 + §3 + §12, and is filed as a planner
> `kind: "log"` DQ for advisor harvest. The brief's verbatim licence for this collapse:
> *"STATUS: DEFERRED UNTIL POST-V1-SHIP … Re-verify all scope assumptions (alert counts, callsite
> counts, crate versions, lane conflicts) at the time of un-deferral … every 'as of 2026-05-25'
> claim in this brief is a hypothesis."*

## Table of contents

1. Summary · 2. Source · 3. Problem statement · 4. Solution statement · 5. Metadata · 6. Relationship to other v1 sub-phases · 7. Preflight guardrails · 8. Flow design · 9. Mandatory reading · 10. Patterns to mirror · 11. Files to change · 12. NOT building · 13. Step-by-step tasks · 14. Testing strategy · 15. Validation commands (DoD) · 16. Acceptance criteria · 16a. Stories · 17. Completion checklist · 18. Risks · 19. Notes · 20. Confidence score

## 1. Summary

Replace the third-party `webmention` 0.6.0 crate (which transitively pins the vulnerable
`reqwest 0.11.27 → hyper-rustls 0.24.2 → rustls 0.21.12 → rustls-webpki 0.101.7` chain) with an
~80-LOC inline W3C-Webmention sender built on the workspace's existing `reqwest 0.13.2` outbound
client (`context.client()`). Removing the dep eliminates the legacy chain entirely and retires the
3 open `rustls-webpki` Dependabot alerts (one HIGH). One impl task, one crate, two files.

## 2. Source

- **Brief:** `.claude/PRPs/briefs/v1-deps-r2-planning-1.md` (frozen 2026-05-25; re-verified 2026-06-04).
- **Parent plan:** `.claude/PRPs/plans/v1-closeout.plan.md` §"Phase 4 — Deferred security: deps-r2"
  (lines 271–290). That section explicitly names this brief and notes "deps-r2 has no plan file" —
  this plan fills that gap. Phase 4's T1 == this plan's Task 1.
- **Prior decision record:** `.claude/PRPs/reports/v1-deps-r3-wasmtime-deferral-decision.md`
  (2026-06-04) — records that the brief's Task 4 (wasmtime) is DONE and Task 3 (astral-tokio-tar)
  premise is inverted.

### 2.1 Scope reconciliation vs the frozen brief (live ground-truth, 2026-06-04)

| Brief task | Brief premise (2026-05-25) | Live state (2026-06-04, verified) | Disposition |
|---|---|---|---|
| **T1 webmention inline** | drop rustls-webpki chain → close 3 alerts (1 HIGH) | `cargo tree -i rustls-webpki@0.101.7`: the chain reaches the workspace **solely** via `webmention 0.6.0`. Alerts #55 (HIGH), #50 (low), #49 (low) **open**. | **KEPT — this plan's Task 1.** |
| **T2 mdurl fork + idna bump** | fork `rlidwka/mdurl`, bump idna 0.3→1.1, close 1 medium | **No idna Dependabot alert exists in any state.** `gh api …/dependabot/alerts` shows zero idna entries. idna 0.3.0 still transitively present but unflagged. | **DROPPED** — driving alert is gone (§12). |
| **T3 astral-tokio-tar dispute** | dismiss/bump to close alerts | Premise inverted: 0.6.0 is the *vulnerable* pin (0.6.1/0.6.2 are the fixes); dev-only via `testcontainers`. **User chose skip** (post-M1 bump). Alerts #57/#58/#65 stay open deliberately. | **NOT IN PLAN** (per close-out + user decision). |
| **T4 wasmtime defer** | document-and-defer | **DONE** — commit `e719081b2`; 12 wasmtime alerts (#37–#47, #60) dismissed `tolerable_risk`. | **NOT IN PLAN** (completed). |

**Live alert totals:** 18 (brief) → **6 open** (verified). After Task 1: **6 → 3** (astral-tokio-tar
×3 remain, by user decision). The brief's "close 6" target is superseded by "close 3" — the other 3
are deliberate post-M1 work, not this plan's scope.

## 3. Problem statement

`crates/api/api_utils/Cargo.toml:73` depends on `webmention = "0.6.0"`. That crate is built on
`reqwest 0.11.27`, which is the *only* thing in the workspace still pulling `rustls 0.21.12` and
`rustls-webpki 0.101.7` (the main workspace is on `reqwest 0.13.2` + `rustls-no-provider`). Three
Dependabot alerts (RUSTSEC against rustls-webpki 0.101.7, one HIGH = #55) are unfixable while the
dep stays, because no newer `webmention` release moves off reqwest 0.11. The crate is used at a
single function (`send_webmention` in `utils.rs:977`) which performs exactly one outbound
operation: W3C Webmention endpoint discovery + a form POST — re-implementable inline against the
modern client without behaviour change.

## 4. Solution statement

Inline the W3C Webmention protocol (GET target → discover `rel="webmention"` endpoint from the
`Link` header, HTML `<link>`/`<a>` fallback → resolve endpoint relative to target → POST
`source`+`target` as `application/x-www-form-urlencoded`) inside `send_webmention`, using
`context.client()` (the existing `reqwest 0.13.2` `ClientWithMiddleware`). Drop the
`webmention = "0.6.0"` line and the `use webmention::{…}` import. Preserve the existing SSRF guard
(`context.is_valid_ip`), the `spawn_try_task` fire-and-forget shape, the
`NoEndpointDiscovered → Ok(())` swallow, and the `CouldntSendWebmention` error mapping. No public
signature changes → the 3 callsites compile unchanged.

## 5. Metadata

- **Phase:** `v1-deps-r2`
- **Branch:** `phase-v1-deps-r2` (cut by BM-task before Task 1)
- **Target impl-task model:** `sonnet-4-6` (MIRROR-ref-heavy single-file rewrite; MiniMax-trial
  eligibility is the advisor's §3.5a call, not decided here)
- **Estimated tasks:** 3 (Task 0 pre-flight + Task 1 impl + Task 2 retro)
- **Estimated cargo budget:** ~5–6 GB peak (single-crate dep change; one `--workspace` check).
  Runs laptop-local (see §7) — not a daemon dispatch.
- **Forbidden-window applicability:** standard (laptop-local cargo per advisor table).
- **Complexity score:** `1/10` — see breakdown below.

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 1 impl task (Task 1); far under 5 |
| Migrations touched | +2 each | 0 | none |
| Crates touched | +1 each | 1 | `lemmy_api_utils` only (§11) |
| `crates/server/tests/e2e.rs` edits | +3 each | 0 | none (existing e2e coverage unchanged) |
| New ADR-affecting decisions | +2 each | 0 | webmention is content-federation, not governance; no ADR touched |
| Cargo budget peak above 6 GB | +1 per GB | 0 | ~5–6 GB, not above 6 |
| **Total** | — | **1** | Threshold for split-DQ: `>8` (Sonnet). **No split.** |

Score 1 ≪ threshold 8. No split-DQ filed.

### 5.2 Per-task complexity ceiling (non-Sonnet target only)

Target is Sonnet → the Sonnet ceiling (≤4 files / ≤2 crates per task) applies. Task 1 modifies
**2 files in 1 crate** — within ceiling. (If the advisor designates Task 1 for the MiniMax trial
per §3.5a, the non-Sonnet ceiling of ≤3 files / ≤1 crate is also satisfied.)

## 6. Relationship to other v1 sub-phases

- **Parent:** `v1-closeout` Phase 4 — this plan IS the detailed task plan for that phase's T1.
  T2/T3/T4 of close-out Phase 4 are dispositioned in §2.1 (dropped / skipped / done).
- **Sibling (not this plan):** `v1-deps-r3` (wasmtime) — DONE.
- **Downstream:** none. Closing the rustls-webpki alerts is terminal; the residual
  astral-tokio-tar ×3 are a separate, deliberately-deferred post-M1 bump.

## 7. Preflight guardrails inherited from prior phases

- **R1:** clippy/check invocations use `--features full` + `--no-deps` uniformly (per
  `feedback_clippy_test_style.md`, `feedback_pre_phase_dod_smoke_test.md`). `lemmy_api_utils` has a
  `full` feature (verified) so `--workspace --features full` is valid.
- **R5:** Task 0 enumerates ALL probes explicitly (per pre-phase-harness-audit.md).
- **M1-isolation invariant (close-out plan line 431):** Task 1 edits
  `crates/api/api_utils/src/{utils.rs}` + `crates/api/api_utils/Cargo.toml`. M1 (Tree B) edits
  *different* files in the same crate (`{notify,plugins,bridge_notify,lib}.rs`) **and Cargo.toml**.
  The `Cargo.toml` line-removal is the only real overlap surface. Per the live close-out snapshot,
  `phase-m1-b` is **merged to `governance-v0`** (`d6d027794`) → the literal M1 gate is satisfied.
  **Task 0 re-confirms** `git log origin/phase-m1-b ^origin/governance-v0` is empty before any edit.
- **NO-ELITEDESK directive (user, 2026-06-04 — `project_closeout_no_elitedesk_m1_active.md`):**
  while close-out runs, the EliteDesk daemon is owned by M1 → **zero close-out Junior dispatch.**
  This plan's impl + cargo therefore run **laptop-local** (advisor/subagent writes the Rust; laptop
  cargo validates per `project_laptop_canonical_cargo_runner.md`; CR review on the PR). **The
  advisor MUST surface the dispatch-mode choice to the user at the plan-approval / dispatch gate
  (defer-until-m1-a vs laptop-local-now) — do NOT auto-pick** (close-out plan §273–277).

## 8. Flow design

```
BEFORE (webmention 0.6.0):
  send_webmention(post, community, ctx)
    └─ spawn_try_task:
         is_valid_ip(url)?  ── err → Ok(())
         Webmention::new(ap_id, url) ── webmention crate (reqwest 0.11 / rustls 0.21)
           .set_checked(true).send()
             ├─ NoEndpointDiscovered → Ok(())
             ├─ Ok → Ok(())
             └─ Err(e) → Err(CouldntSendWebmention)

AFTER (inline, reqwest 0.13.2 via context.client()):
  send_webmention(post, community, ctx)
    └─ spawn_try_task:
         is_valid_ip(url)?  ── err → Ok(())
         discover_endpoint(ctx.client(), &url):           [new helper, same file]
           GET url → Link header rel="webmention"
                   → else HTML <link>/<a rel="webmention">  (webpage::HTML — already a dep)
                   → None → Ok(())                          [== NoEndpointDiscovered swallow]
         endpoint = base(url).join(found)?
         ctx.client().post(endpoint)
            .form(&[("source", ap_id), ("target", url)]).send()
              ├─ 2xx → Ok(())
              └─ err → Err(CouldntSendWebmention)
```

All boxes live in `crates/api/api_utils/src/utils.rs` (Task 1). No callgraph fan-out — the 3
callers (`create.rs:144`, `update.rs:179`, `scheduled_tasks.rs:1025`) call the unchanged
`send_webmention` signature.

## 9. Mandatory reading

- **MIRROR (outbound client + TLS provider + header parsing):**
  `crates/api/api_utils/src/request.rs:1–60` — imports, `client_builder` (ring crypto provider
  install at line 39), `reqwest::header` usage, `webpage::HTML` for HTML parsing, `urlencoding::encode`.
- **MIRROR (client accessor):** `crates/api/api_utils/src/context.rs:52` —
  `pub fn client(&self) -> &ClientWithMiddleware`.
- **Edit target (verbatim):** `crates/api/api_utils/src/utils.rs:62` (import) and `:977–999`
  (the `send_webmention` fn).
- **Dep line:** `crates/api/api_utils/Cargo.toml:73` (`webmention = { version = "0.6.0" }`).
- **Error variant:** `crates/utils/src/error.rs:163` (`CouldntSendWebmention`) — keep as-is.
- **W3C spec:** Webmention (https://www.w3.org/TR/webmention/) §3 (Sender discovery) — Link header
  first, HTML `<link rel="webmention">`/`<a rel="webmention">` fallback, POST x-www-form-urlencoded.
- **Lessons:** `feedback_carry_patch_todos.md` (n/a here — no patch added), `feedback_clippy_test_style.md`,
  `feedback_pre_phase_dod_smoke_test.md`.

## 10. Patterns to mirror

### 10.1 Outbound fetch via the configured client (not a new `Client::builder`)

**Mirror:** `crates/api/api_utils/src/request.rs:54–60` (`fetch_link_metadata`) + `context.rs:52`.

```rust
// Reuse the workspace client — it already carries the reqwest-middleware stack,
// user-agent, timeouts, and (via client_builder, request.rs:39) the installed
// rustls ring crypto provider. Do NOT construct a new reqwest::Client.
let res = context.client().get(url.as_str()).send().await?;
```

### 10.2 Header + HTML endpoint discovery

**Mirror:** `request.rs:23–34` (the `reqwest::header` + `webpage::HTML` import set).

```rust
// 1. Link header: reqwest::header::LINK, parse `<url>; rel="webmention"`.
//    (add LINK to the existing `header::{CONTENT_TYPE, LOCATION, RANGE}` import set)
// 2. Fallback: parse body with webpage::HTML and scan for
//    <link rel="webmention" href> / <a rel="webmention" href>.
// 3. Resolve the discovered href against the *final* target Url via Url::join.
// 4. None discovered → return Ok(()) (preserves the crate's NoEndpointDiscovered swallow).
```

### 10.3 Fire-and-forget + SSRF guard (preserve verbatim)

**Mirror:** `crates/api/api_utils/src/utils.rs:977–999` (current body).

```rust
// KEEP unchanged: the spawn_try_task wrapper, the `community.visibility
// .can_view_without_login()` guard, and the SSRF check:
if context.is_valid_ip(&url).await.is_err() { return Ok(()); }
// KEEP the error mapping shape on the POST failure:
//   .with_lemmy_type(UntranslatedError::CouldntSendWebmention.into())
```

## 11. Files to change

**Crate `lemmy_api_utils` (`crates/api/api_utils/`):**

- `src/utils.rs` — rewrite `send_webmention` body inline (§10.1–10.3); replace the
  `use webmention::{Webmention, WebmentionError};` import (line 62) with the small import delta
  (`reqwest::header::LINK`; `webpage::HTML` and `urlencoding::encode` if not already in scope in
  this file). (Task 1)
- `Cargo.toml` — remove `webmention = { version = "0.6.0" }` (line 73). (Task 1)

No `creates:`. No struct-field adds → the §11 callsite-enumeration sub-rule does not apply
(`send_webmention`'s signature is unchanged; the 3 callers are read-only confirmations, not edits).

**Callers (compile-only confirmation, NOT edited):** `crates/api/api_crud/src/post/create.rs:144`,
`crates/api/api_crud/src/post/update.rs:179`, `crates/routes/src/utils/scheduled_tasks.rs:1025`.

## 12. NOT building in v1-deps-r2

- **mdurl fork + idna bump (brief T2)** — DROPPED. No idna Dependabot alert exists in any state
  (verified 2026-06-04). Forking a third-party crate + adding the first `[patch.crates-io]` is not
  justified for an unflagged transitive dep. Revisit only if Dependabot re-flags idna or a fresh
  RUSTSEC re-applies. (`idna 0.3.0` remains transitively present but un-alerted.)
- **astral-tokio-tar dismiss/bump (brief T3)** — deferred to a post-M1 bump per the user's
  recorded decision; it is dev-only (`testcontainers`). Alerts #57/#58/#65 stay open deliberately.
- **wasmtime/extism (brief T4 / deps-r3)** — DONE (`e719081b2`); not re-done here.
- **`webpage` removal** — `webpage` stays (used by `request.rs` and reused here for HTML fallback).
- **Changing the 3 caller signatures or the `CouldntSendWebmention` error variant** — out of scope;
  behaviour preserved.

---

## 13. Step-by-step tasks

> **Dispatch mode:** laptop-local (NO-ELITEDESK, §7). No Junior `[P]` cohort — single impl task.
> All tasks are non-`[P]` barriers.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** confirm branch is `phase-v1-deps-r2`; confirm the M1 gate is satisfied; confirm the
wrappers run cargo correctly; capture the rustls-webpki baseline.

**Probes (per `pre-phase-harness-audit.md` — R5, Windows laptop form):**

```bash
# Probe A — branch
git branch --show-current   # EXPECT: phase-v1-deps-r2

# Probe B — M1 gate satisfied (M1-isolation invariant, §7)
git fetch origin --prune
git log origin/phase-m1-b ^origin/governance-v0 --oneline | wc -l   # EXPECT: 0

# Probe C — no other open PR touches the §11 files
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("api_utils/src/utils.rs|api_utils/Cargo.toml")) | {number,title,headRefName}'
# EXPECT: empty

# Probe D — wrapper sanity (honors -p)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_utils --features full > .claude/PRPs/debug/v1-deps-r2-task0-check.log 2>&1"
echo "exit: $?"   # EXPECT: 0; log shows lemmy_api_utils compiling

# Probe E — negative (exit-code propagation)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_utils --features nonexistent_xyz > .claude/PRPs/debug/v1-deps-r2-task0-neg.log 2>&1"
echo "exit: $?"   # EXPECT: non-zero

# Probe F — rustls-webpki baseline (the alert under test)
cargo tree -i rustls-webpki@0.101.7 -e no-dev > .claude/PRPs/debug/v1-deps-r2-task0-tree.log 2>&1
echo "exit: $?"   # EXPECT: 0; root chain = webmention 0.6.0 (this is the chain Task 1 removes)
```

**EXPECT block:** A→`phase-v1-deps-r2`; B→`0`; C→empty; D→exit 0; E→non-zero; F→exit 0 showing the
webmention root. **No commit at Task 0.**

### Task 1: Inline webmention sender; drop the webmention 0.6.0 dependency

**ACTION:** rewrite `send_webmention` to perform W3C Webmention discovery + POST inline via
`context.client()`, and remove the `webmention` crate dependency.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api_utils/src/utils.rs        # inline send_webmention body + import delta (drop `use webmention::…`)
  - crates/api/api_utils/Cargo.toml          # remove `webmention = { version = "0.6.0" }`
requires: []
```

**IMPLEMENT (file 1 of 2):** in `crates/api/api_utils/src/utils.rs`:
- Replace line 62 `use webmention::{Webmention, WebmentionError};` with the needed delta:
  `reqwest::header::LINK` (extend existing reqwest header imports), `webpage::HTML`,
  `urlencoding::encode` (only those not already imported in this file).
- Rewrite the `send_webmention` body (lines 977–999) per §10:
  1. keep the `community.visibility.can_view_without_login()` + `spawn_try_task` shell verbatim;
  2. keep the SSRF guard `if context.is_valid_ip(&url).await.is_err() { return Ok(()); }`;
  3. add a private `async fn` (same file) that does discovery: GET `url` via `context.client()`,
     read `Link` header for `rel="webmention"`; if absent, parse body with `webpage::HTML` for
     `<link rel="webmention">`/`<a rel="webmention">`; resolve via `Url::join`; `Ok(None)` if none;
  4. on `None` → `Ok(())` (mirrors `NoEndpointDiscovered`);
  5. on `Some(endpoint)` → `context.client().post(endpoint).form(&[("source", &source),
     ("target", &target)]).send().await` → map `2xx → Ok(())`, transport/non-2xx →
     `Err(e).with_lemmy_type(UntranslatedError::CouldntSendWebmention.into())`.

**IMPLEMENT (file 2 of 2):** in `crates/api/api_utils/Cargo.toml`, delete line 73
(`webmention = { version = "0.6.0" }`).

**MIRROR:** `request.rs:1–60` (client + headers + HTML) ; `context.rs:52` (`client()`) ;
`utils.rs:977–999` (preserve shell).

**GOTCHA:**
- Do NOT build a new `reqwest::Client` — reuse `context.client()` so the ring crypto provider
  (installed in `client_builder`, `request.rs:39`) and the middleware stack are reused. A fresh
  `reqwest 0.13.2` client without `install_default()` panics on first TLS handshake.
- `redirect::Policy::none()` is set on the workspace client — discovery must resolve the endpoint
  against the *response's* final URL only if you follow redirects manually; for parity with the old
  crate, resolving against the original `target` is acceptable (the old crate did not chase
  redirects for discovery base). Keep it simple: `Url::join` on `target`.
- Preserve the silent-swallow contract: missing endpoint and "can't view without login" both
  return `Ok(())` — webmention failure must never fail post create/update.

**VALIDATE:**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-deps-r2-task1-check.log 2>&1"
echo "exit: $?"   # EXPECT: 0
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-deps-r2-task1-clippy.log 2>&1"
echo "exit: $?"   # EXPECT: 0
cargo tree -i rustls-webpki@0.101.7 -e no-dev > .claude/PRPs/debug/v1-deps-r2-task1-tree.log 2>&1
echo "tree exit: $?"   # EXPECT: non-zero / "package ID specification … did not match" (chain gone)
```

**Commit:** `fix(deps): inline webmention sender, drop webmention 0.6.0 (rustls-webpki) (task 1)`.

### Task 2: Retro

**Goal:** author retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`
(one H2 per role). Record the scope-collapse (4→1 task) as the headline Planning signal, and
whether the live re-verify mandate caught it. Promote any new lesson in the same commit.

---

## 14. Testing strategy

- **Unit (compile-time):** `cargo check --workspace --features full`.
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings`.
- **e2e:** no new e2e test. The existing suite must stay green (webmention is fire-and-forget; no
  e2e currently asserts an outbound webmention). Run the full e2e once at merge-gate to confirm no
  regression from the dep removal (the dep change can shift the lockfile for the test build).
- **Dependency proof:** `cargo tree -i rustls-webpki@0.101.7` must report the package gone.
- **Alert proof (post-merge, manual):** the 3 rustls-webpki alerts (#49/#50/#55) auto-close once
  the lockfile no longer contains rustls-webpki 0.101.7 on `governance-v0`.

## 15. Validation commands (DoD)

> Advisor dry-runs each command against current HEAD at the plan-approval gate
> (`feedback_plan_dod_dry_run_at_write.md`). Windows laptop form (NO-ELITEDESK → laptop-local).

### 15.1 Static analysis (Task 1)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-deps-r2-task1-check.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.2 Lint (Task 1 — uniform R1)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-deps-r2-task1-clippy.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.3 e2e (merge-gate, full suite — regression check)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-deps-r2-e2e.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.4 Dependency removal proof

```bash
cargo tree -i rustls-webpki@0.101.7 -e no-dev > .claude/PRPs/debug/v1-deps-r2-tree-after.log 2>&1
echo "exit: $?"   # EXPECT: non-zero (package no longer in graph)
grep -c "webmention" crates/api/api_utils/Cargo.toml   # EXPECT: 0
rg "webmention::" crates/   # EXPECT: 0 external-crate references
```

## 16. Acceptance criteria

1. `cargo check --workspace --features full` exits 0.
2. `cargo clippy --workspace --features full --no-deps -- -D warnings` exits 0.
3. Full e2e suite green (no regression from the dep removal).
4. `cargo tree -i rustls-webpki@0.101.7` reports the package absent.
5. `rg "webmention::" crates/` returns 0 (external crate fully inlined); the 3 `send_webmention`
   call sites (`create.rs:144`, `update.rs:179`, `scheduled_tasks.rs:1025`) compile unchanged.
6. Post-merge: Dependabot alerts #49, #50, #55 (rustls-webpki) auto-close. Open count 6 → 3.

## 16a. Stories (independently-testable behaviour units)

### Story 1: A federated post still triggers a webmention to a linked external URL

**Checkpoint:** `cargo check --workspace --features full` exits 0 AND `send_webmention`'s 3 callers
compile unchanged. Behaviour: a public-community post whose `url` resolves and passes the SSRF guard
results in a discovery GET + (if an endpoint is found) a form POST — same observable outbound shape
as the old crate. (No e2e asserts the network call; compile + unchanged-signature is the proxy.)

### Story 2: The legacy rustls-webpki chain is gone from the build graph

**Checkpoint:** `cargo tree -i rustls-webpki@0.101.7` exits non-zero; `grep -c webmention
crates/api/api_utils/Cargo.toml` == 0.

### Story 3: Webmention failures never break post creation

**Checkpoint:** code review confirms every error path in `send_webmention` returns `Ok(())` except
the explicit `CouldntSendWebmention` mapping inside the spawned task (which is logged, not
propagated to the caller) — the `spawn_try_task` shell is preserved.

## 17. Completion checklist

- [ ] Task 0 probes pass (branch, M1 gate empty, no PR overlap, wrappers sane, baseline captured).
- [ ] Task 1: `send_webmention` inlined; `webmention` dep + import removed; check + clippy green.
- [ ] `cargo tree` proves rustls-webpki 0.101.7 gone.
- [ ] Full e2e green at merge-gate.
- [ ] PR opened into `governance-v0` (`--repo barrie-cork/lemmy`), CR review clean.
- [ ] Post-merge: alerts #49/#50/#55 auto-closed; open count 6 → 3.
- [ ] Task 2 retro authored; scope-collapse lesson promoted.

## 18. Risks and mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| Inline discovery diverges from the old crate's behaviour (e.g. missing the HTML fallback) | Med | §10.2 mandates both Link-header AND HTML `<link>`/`<a>` fallback; mirror `webpage::HTML` use from request.rs. |
| New `reqwest 0.13.2` client panics on TLS (no crypto provider) | Low | Reuse `context.client()` — provider already installed (`request.rs:39`); §13 GOTCHA forbids a fresh client. |
| `Cargo.toml` edit races an M1 Cargo.toml edit | Low | Task 0 Probe B confirms M1 merged; Probe C confirms no open PR touches the file; NO-ELITEDESK keeps this laptop-local. |
| Removing the dep shifts the lockfile and breaks an unrelated build target | Low | §15.3 full e2e at merge-gate; `--workspace` check covers all crates. |
| Webmention failure starts breaking post create/update | Low | Story 3 + §13 GOTCHA: preserve the `Ok(())` swallow + `spawn_try_task`. |

## 19. Notes

- This plan deliberately ships **one** task. The brief's other three were dispositioned by live
  re-verification (§2.1) — the re-verify mandate is the brief's own instruction, so the collapse is
  conformant, not scope-cutting. A planner `kind: "log"` DQ records the reconciliation for advisor
  harvest.
- `webpage`, `urlencoding`, `reqwest 0.13.2` are all already workspace deps — no new dependency is
  added. This plan only *removes* one (`webmention`).
- No `[patch.crates-io]` is introduced (the brief's T2 was the only source of one, now dropped).

## 20. Confidence score

**8.5/10.** High: the dependency provenance is proven by `cargo tree` (rustls-webpki reaches the
workspace solely via webmention 0.6.0), the edit is a single-file behaviour-preserving rewrite with
an exact in-repo MIRROR (`request.rs`), and the scope is verified against live Dependabot. The 1.5
deduction: the inline discovery is hand-rolled protocol code (the W3C edge cases — relative endpoint
resolution, multiple `rel` values in one Link header) are the one place behaviour could subtly drift
from the crate, and there is no e2e asserting the outbound call, so review rigor on Task 1 carries
the correctness weight.
