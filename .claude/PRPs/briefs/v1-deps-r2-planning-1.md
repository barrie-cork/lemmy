# Planning Brief — v1-deps-r2: Security-Alert Sweep (webmention inline + mdurl fork + tar dispute + wasmtime defer)

**Phase:** v1-deps-r2
**Branch:** phase-v1-deps-r2 (cut from governance-v0 — exact SHA at lane-cut time)
**Authored:** 2026-05-25
**Authored by:** advisor (canonical brehon-fork session)
**PRD:** (none — pure security-alert sweep; no PRD needed)
**Plan target:** `.claude/PRPs/plans/v1-deps-r2.plan.md`

**STATUS: DEFERRED UNTIL POST-V1-SHIP** (user decision 2026-05-25). This brief is the durable record of the scoping work done 2026-05-25 (Perplexity research + user decisions on the four upgrade vectors). It is NOT queueable until v1 (all in-flight lanes JM/SR/SL/AD/RT/fed-in/ship/etc + the deferred wasmtime work in v1-deps-r3) ships. Rationale: 18 Dependabot alerts in the security tab are non-blocking for v1 pilot launch; the production-risk High (rustls-webpki via webmention) only fires on outbound webmention sends to malicious targets (low real-world exposure); the 2 wasmtime criticals are aarch64-only and Brehon deploys x86_64. Re-verify all scope assumptions (alert counts, callsite counts, crate versions, lane conflicts) at the time of un-deferral — every "as of 2026-05-25" claim in this brief is a hypothesis when this lane is eventually cut.

**Hard precondition (when un-deferred):** v1-deps-r1 must be merged to `governance-v0` and its retro signed off (both confirmed 2026-05-25: PR #153 `ab1e79a70` merged; retro at `.claude/PRPs/reports/session-retro-2026-05-25-v1-deps-r1.md`). Cargo workspace surface is now on `diesel-async 0.9` + `sha2 0.11`; the migration this phase performs (webmention inline + mdurl patch) does NOT touch `.run_transaction` or `Sha256` callsites.

**Hard precondition #2 (when un-deferred):** Re-verify no active lane conflicts with `crates/api/api_utils/src/utils.rs::send_webmention` (lines 977-998) or `webmention` imports (line 62). At 2026-05-25 RT-r3 was in-flight (driven from `brehon-fork-rt-r3` Mode-A worktree) and did not overlap; by the time this lane is un-deferred, all v1 lanes will have shipped, so this check becomes a sanity re-check rather than an active-lane gate.

**Lane mode (when un-deferred):** Default to **Mode B (mobile remote-control)** per `multi-lane-worktree.md` §"Lane modes" (added 2026-05-25). The work is small (≤80 LOC of edits + Dependabot UI clicks + a forked-crate `[patch.crates-io]` line); spinning a dedicated Mode-A lane worktree is overhead for a 1-2 day phase. Brief authored on `governance-v0` in canonical; trunk→phase sync via daemon SSH (per `multi-lane-worktree.md` §"Brief location and trunk→phase sync"). If laptop is at desk and a quick dedicated worktree is preferred, fall back to Mode A — both modes are valid; the brief content is mode-agnostic.

---

## 1. What this phase delivers

A targeted security-alert sweep retiring **6 of 18 open GitHub Dependabot alerts** against `barrie-cork/lemmy` (`Cargo.lock`-side) by addressing each vulnerable transitive crate at its real root cause. The remaining 12 alerts (all on `wasmtime 41.0.4` via `extism 1.21.0`) are deferred to `v1-deps-r3` with documented Dependabot suppressions per user decision 2026-05-25 (see §10).

The phase consolidates three independent change classes:

1. **Inline replacement of `webmention 0.6.0` crate — production code edit.** Removes the legacy `rustls 0.21.12` → `rustls-webpki 0.101.7` chain entirely. The `webmention` crate is unmaintained (latest is 0.6.0 from 2024, no open PRs bumping to `reqwest 0.12+`) and is pulled ONLY by 1 production callsite in `crates/api/api_utils/src/utils.rs::send_webmention` (lines 977-998), plus 1 import + 1 enum match arm. The W3C Webmention spec (RFC 6249-style) is ~80 LOC of `reqwest 0.13` — `GET target` → parse `Link` header for endpoint → POST `application/x-www-form-urlencoded` with `source` + `target`. We already have `reqwest 0.13.3` in the lockfile via `activitypub_federation`. **Closes 3 alerts: 1 High (GHSA-82j2-j2ch-gfr8 rustls-webpki DoS via malformed CRL BIT STRING) + 2 Low (GHSA-xgp8-3hg3-c2mh, GHSA-965h-392x-2mh5 — webpki name constraints).**

2. **Fork `rlidwka/mdurl.rs` + `[patch.crates-io]` — workspace dependency redirect.** The `mdurl 0.3.1` crate (pulled by `markdown-it 0.6.1` for post-body rendering) pins `idna 0.3.0` and is unmaintained. `idna 1.1.0` is ALREADY in our lockfile (via `url` / `reqwest 0.13`), so the duplicate `idna 0.3.0` entry is purely cosmetic — removing it retires `GHSA-h97m-ww89-6jmq` (medium, Punycode-decoding bypass). Fork `barrie-cork/mdurl` from `rlidwka/mdurl.rs` upstream, bump `idna = "0.3"` → `idna = "1.1"` in its `Cargo.toml`, adapt ~5 call-sites (`domain_to_ascii` / `domain_to_unicode` API surface is structurally similar between idna 0.3 and 1.1; the difference is error-type handling), and pin via `[patch.crates-io]` in the workspace root `Cargo.toml`. **Closes 1 alert: GHSA-h97m-ww89-6jmq.**

3. **Dispute `astral-tokio-tar 0.6.0` advisories — GitHub UI action only, no code change.** Per Perplexity research 2026-05-25, `astral-tokio-tar 0.6.0` IS the fix release for both GHSA-fp55-jw48-c537 (medium, PAX header desync — fixed in 0.5.6) and GHSA-xx64-wwv2-hcqq (low, chmod-via-symlink — fixed in 0.6.0 per Fedora 42 update notes). Dependabot has stale advisory data. File "dispute advisory" via GitHub Security tab citing the release notes. **Closes 2 alerts via Dependabot dismissal: GHSA-fp55-jw48-c537 + GHSA-xx64-wwv2-hcqq.**

4. **Defer `wasmtime` upgrade to `v1-deps-r3` — documented in code-free DQ entries.** Per Perplexity research + user decision 2026-05-25: extism 1.21.0 pins `wasmtime 41.0.4` via internal `wiggle` macro-generated WASI shims; a `[patch.crates-io]` override of `wasmtime` will NOT compile without forking extism itself (1-3 days of work, high risk against ADR-012 plugin host). Both critical CVEs (GHSA-xx5w-cvp6-jv83, GHSA-jhxm-h53p-jm7w) are aarch64-only — Brehon's deployment is x86_64. File a `kind: "log"` DQ entry on this phase recording: (a) the deferral rationale, (b) the upstream tracking issue (extism/extism Dependabot PR #847 open since 2026-04-07), (c) the Dependabot dismissal reasons for all 13 wasmtime alerts ("not exploitable on x86_64 deployment, awaiting upstream Extism release"). Update roadmap to add `v1-deps-r3` as unstarted, gated on upstream Extism release. **Closes 0 alerts in code; closes 13 in Dependabot UI via dismissal with documented reasons.**

All four classes ship in one PR. Splitting is wasteful — Task 2 (mdurl fork) and Task 3 (tar dispute) have zero overlap with Task 1 (webmention inline) and zero overlap with each other; bundling amortizes the e2e gate (~26 min on the laptop).

---

## 2. Key file anchors (verify these before authoring the plan)

| File | Anchor | Purpose |
|---|---|---|
| `crates/api/api_utils/src/utils.rs:62` | `use webmention::{Webmention, WebmentionError};` | Task 1: import to remove |
| `crates/api/api_utils/src/utils.rs:977-998` | `pub fn send_webmention(post: Post, community: &Community, context: Data<LemmyContext>)` | Task 1: function body to rewrite inline (~22 lines current, ~70-80 lines after) |
| `crates/api/api_utils/Cargo.toml:73` | `webmention = { version = "0.6.0" }` | Task 1: dep to remove |
| `crates/api/api_utils/Cargo.toml` | `reqwest = { workspace = true }` (line 67) | Task 1: existing dep used by replacement |
| `Cargo.toml` (workspace root) | `reqwest = { version = "0.13.2", default-features = false, features = [ ... ] }` (line 200) | Task 1: confirm reqwest 0.13.x features include what's needed (`rustls-tls`, no `default-tls`) |
| `Cargo.toml` (workspace root) | (new) `[patch.crates-io]` table or addition | Task 2: add `mdurl = { git = "https://github.com/barrie-cork/mdurl", branch = "idna-1x" }` |
| `Cargo.lock` | (auto-updated; expect ~200 line delta) | Task 1 removes rustls 0.21 / rustls-webpki 0.101 / reqwest 0.11 / hyper-rustls 0.24 / tokio-rustls 0.24 chain; Task 2 removes idna 0.3.0 + dependent crates |
| (external) `github.com/rlidwka/mdurl.rs` | Upstream to fork | Task 2: fork to `github.com/barrie-cork/mdurl`, branch `idna-1x` |
| (GitHub UI) `barrie-cork/lemmy` → Security tab → Dependabot alerts | GHSA-fp55-jw48-c537 + GHSA-xx64-wwv2-hcqq | Task 3: "Dismiss alert" → reason "Fix already implemented (astral-tokio-tar 0.6.0)" |
| (GitHub UI) `barrie-cork/lemmy` → Security tab → Dependabot alerts | 13× GHSA-* on `wasmtime 41.0.4` | Task 4: "Dismiss alert" → reason "Not vulnerable on x86_64 deployment (aarch64-only criticals); upstream Extism release pending (extism/extism Dependabot PR #847 open since 2026-04-07)" |
| `.claude/PRPs/v1-roadmap.json` | `lanes.deps.sub_phases` | Post-merge: add `v1-deps-r2` (done) + `v1-deps-r3` (unstarted, gated on upstream Extism release) entries via `/auto-roadmap` or hand-edit |

Production webmention surface is exactly **3 references** in the fork:
```
crates/api/api_utils/src/utils.rs:62          use webmention::{Webmention, WebmentionError};
crates/api/api_utils/src/utils.rs:985         let mut webmention = Webmention::new::<Url>(post.ap_id.clone().into(), url.clone().into())?;
crates/api/api_utils/src/utils.rs:992         Err(WebmentionError::NoEndpointDiscovered(_)) => Ok(()),
```

Plus 3 callsites of `send_webmention()`:
```
crates/api/api_crud/src/post/create.rs:144    send_webmention(inserted_post.clone(), community, context.clone());
crates/api/api_crud/src/post/update.rs:179    send_webmention(updated_post.clone(), &community, context.clone());
crates/routes/src/utils/scheduled_tasks.rs:902 send_webmention(post, &community, context.clone());
```

**Planner MUST re-enumerate at plan-author time** — per `feedback_fix_impl_enumerate_all_callsites.md`. If `rg "webmention::|Webmention::|WebmentionError\|send_webmention" crates/` returns a different count than 3 + 3 = 6 total, file a `kind: "blocker"` DQ before authoring.

---

## 3. Watchpoints for the planner

**WP-1 (Task 1 — webmention inline replacement must preserve semantic behavior).** The current `send_webmention` does these things in order:

1. Returns early if `post.url` is None OR community visibility doesn't allow no-login viewing.
2. Spawns a `spawn_try_task` (fire-and-forget) with the actual send.
3. Validates `url` is not a private IP via `context.is_valid_ip(&url)` — returns Ok(()) if invalid (silent no-op).
4. Constructs a `Webmention::new(source=post.ap_id, target=url)`.
5. Sets `.set_checked(true)` (skips a TLS verification step internally — verify this isn't load-bearing).
6. Calls `.send()` instrumented with a tracing span "Sending webmention".
7. Maps `NoEndpointDiscovered` to Ok (target doesn't accept webmentions — silent no-op) and any other error to `CouldntSendWebmention`.

The inline replacement MUST preserve all 7 behaviors. Per `feedback_multi_write_handlers_need_transactions.md` (transaction-scope analogue): **no silent semantic widening or narrowing**. The Webmention protocol per [W3C Rec](https://www.w3.org/TR/webmention/) requires:

- **Endpoint discovery:** GET (or HEAD) the target URL; check for `Link: <endpoint>; rel="webmention"` header OR `<link rel="webmention" href="...">` in the HTML body. Header takes priority. If neither found, treat as `NoEndpointDiscovered` (silent no-op per current behavior).
- **Send:** POST to the discovered endpoint with `Content-Type: application/x-www-form-urlencoded`, body `source=<urlencoded source>&target=<urlencoded target>`. Any 2xx response = success. Any other response or network error = `CouldntSendWebmention`.

The `.set_checked(true)` skips one of the spec's optional verification steps (the current crate documents this as "skip the source-contains-target backlink check"). For our use case (we're the source, we know the link is there), `checked = true` is correct. The inline replacement does not need to implement this check at all.

**The `is_valid_ip` private-IP guard MUST be preserved verbatim** — this is a security gate (SSRF protection). Do NOT inline the private-IP check; keep the existing `context.is_valid_ip(&url).await.is_err()` branch.

**WP-2 (Task 1 — reqwest 0.13 features in api_utils/Cargo.toml).** Verify `crates/api/api_utils/Cargo.toml` line 67 `reqwest = { workspace = true }` resolves to a workspace pin that includes `rustls-tls` (not `default-tls`) — otherwise the inline replacement could re-introduce a different TLS stack. Pre-flight check at plan-author time:

```bash
grep -A5 "^reqwest = " Cargo.toml | head -10
```

Expected: workspace pin uses `default-features = false` + explicit `rustls-tls` feature. If the workspace pin includes `default-tls` (which would pull native-tls / OpenSSL), the inline replacement MUST add `features = ["rustls-tls"]` to its local `reqwest = { workspace = true, features = [...] }` to avoid pulling a new TLS stack.

**Risk-side outcome:** if reqwest 0.13's TLS stack is mis-configured, the inline replacement could pull a NEWER `rustls-webpki` line (e.g. 0.103.x — already in lockfile) but ALSO somehow re-introduce a separate TLS chain. Verify post-merge by `cargo tree -i rustls-webpki:0.101.7 --workspace` returning empty (the old chain should be GONE entirely).

**WP-3 (Task 1 — Link header parsing).** Multiple Webmention endpoints discoverable per spec; **first match wins**. The header can be multi-valued (`Link: <ep1>; rel="webmention", <ep2>; rel="webmention"`). Use `reqwest::header::HeaderMap::get_all("link")` to iterate. Do NOT use a crate that hands back only the first header — that loses the multi-value case. Acceptable: `linkify` or `linkparser` crate (small, well-maintained) for parsing the `Link` header value, OR hand-roll a ~20-line parser. Planner picks.

**HTML `<link>` fallback** is OPTIONAL per spec; the current `webmention` crate implements it. Planner decides whether to implement HTML fallback in the inline rewrite. Default: **YES, implement it** — without HTML fallback, ~30% of WordPress/Webmention targets fail discovery (they only emit the `<link>` element, not the header). Use a minimal HTML parser (e.g. `scraper 0.20` — already in lockfile if any Lemmy code uses it; verify with `grep scraper Cargo.lock`). If `scraper` is not in lockfile, hand-roll a ~30-line regex-based parser (Webmention `<link>` lives in `<head>`, format `<link rel="webmention" href="..."` — a regex is acceptable for this narrow case).

**WP-4 (Task 2 — mdurl fork must compile in isolation BEFORE patch).** Before adding `[patch.crates-io]` to the workspace `Cargo.toml`, verify the forked `mdurl` crate compiles independently:

```bash
git clone https://github.com/barrie-cork/mdurl /tmp/mdurl-fork
cd /tmp/mdurl-fork
cargo build --release
```

If this fails, the patch will silently break the whole workspace (cargo can't resolve the workspace's `markdown-it` → `mdurl` dependency). The fork's compile is a separate gate from the workspace's compile.

**API adaptation from idna 0.3 → 1.1:**
- `idna 0.3`: `domain_to_ascii(domain: &str) -> Result<String, Errors>` where `Errors` is a struct.
- `idna 1.1`: `domain_to_ascii(domain: &str) -> Result<String, ProcessingError>` where `ProcessingError` is an enum with named variants.
- Adaptation: change `Result<String, idna::Errors>` to `Result<String, idna::ProcessingError>` in the ~5 mdurl call-sites. Error mapping in mdurl is internal — the public mdurl API surface doesn't expose idna error types (verify by grep of upstream mdurl source).

**WP-5 (Task 2 — `[patch.crates-io]` git pin discipline).** Per `feedback_carry_patch_todos.md`: any `[patch.crates-io]` entry that points to a `barrie-cork/*` fork MUST be accompanied by:

1. A `TODO(brehon-fork): upstream this` comment immediately above the patch line in `Cargo.toml`.
2. A `kind: "log"` DQ entry naming: (a) the upstream repo + commit/branch we're forking from, (b) the reason (idna 0.3 → 1.1 bump to retire GHSA-h97m-ww89-6jmq), (c) the upstream-PR URL once filed (file upstream PR to `rlidwka/mdurl.rs` as part of Task 2).
3. An entry in `.claude/lessons/feedback_carry_patch_inventory.md` (if it exists) or a new `feedback_carry_patch_inventory.md` row naming the patch + its retirement condition (when upstream merges, drop the patch).

Pin the fork to a **specific commit SHA**, not a branch tip:
```toml
[patch.crates-io]
mdurl = { git = "https://github.com/barrie-cork/mdurl", rev = "<exact-commit-sha>" }
```

Branch-tip pins are mutable; commit-SHA pins are immutable. Per `feedback_principles_not_rules.md`: reproducibility > convenience.

**WP-6 (Task 3 — Dependabot dismissal reasons must cite verifiable evidence).** Each dismissal reason MUST cite:

- For `astral-tokio-tar` alerts: the Fedora 42 update notes URL + the upstream `astral-sh/tokio-tar` 0.6.0 release notes (or commit fixing the CVE) + the lockfile line showing we're on 0.6.0.
- For `wasmtime` alerts: the extism/extism Dependabot PR #847 URL + the GHSA advisory's "Affected versions" field showing the aarch64-only restriction (verify each of the 13 alerts independently — some may NOT be aarch64-only; those need a different dismissal rationale).

**WP-7 (Task 4 — wasmtime defer DQ entry must enumerate all 13 alerts).** The `kind: "log"` DQ entry for the wasmtime defer MUST list every one of the 13 GHSA IDs with: severity, exploit prerequisite (aarch64? x86_64?), Dependabot dismissal reason, retirement condition (which extism release retires it). Format:

```yaml
{
  "id": "<session-id>-NNN",
  "kind": "log",
  "from": "advisor",
  "answered_by": "advisor",
  "question": "Defer wasmtime 41.0.4 upgrade to v1-deps-r3 — extism 1.21.0 internal-API pin blocks [patch.crates-io]",
  "answer": "Per Perplexity research 2026-05-25 + user decision: 13 wasmtime alerts deferred to v1-deps-r3, gated on upstream Extism release. Critical alerts (GHSA-xx5w-cvp6-jv83, GHSA-jhxm-h53p-jm7w) are aarch64-only; Brehon deployment is x86_64. Tracking: extism/extism Dependabot PR #847 (open since 2026-04-07). Dependabot dismissals filed with citation.",
  "context": "{\"alerts\":[{\"ghsa\":\"GHSA-xx5w-cvp6-jv83\",\"severity\":\"critical\",\"arch\":\"aarch64\",\"dismissal_reason\":\"...\"}, ...×13]}",
  "resolved_at": "<ISO>"
}
```

This entry is the durable record — Dependabot dismissals can be undone by anyone with security-tab access; the DQ entry is the audit trail.

**WP-8 (post-lane drift — RT-r3 may have landed by lane-cut time).** This brief references trunk SHA `e6989d3f8` (2026-05-25). At lane-cut time, trunk may include RT-r3 merge. Planner MUST re-run:

```bash
rg "webmention::|Webmention::|WebmentionError|send_webmention" crates/
```

and confirm the 6-reference count holds against current trunk. If RT-r3 added new callsites of `send_webmention`, those need to be in scope for Task 1's function-signature compatibility check (the new inline `send_webmention` MUST keep the same public signature: `pub fn send_webmention(post: Post, community: &Community, context: Data<LemmyContext>)`).

**WP-9 (post-lane drift — Dependabot may have re-opened PRs).** Between brief-author time and lane-cut time, Dependabot may have:

- Re-opened PR #136 (or a successor) with newer transitive bumps — `cargo update -p webmention` is a no-op (latest is 0.6.0) but Dependabot may have surfaced new `wasmtime` advisories.
- Re-counted the alerts (some may have been dismissed or new ones surfaced).

At lane-cut time, re-run `gh api repos/barrie-cork/lemmy/dependabot/alerts --paginate -q '[.[] | select(.state == "open")]'` and update the brief's "6 alerts retired" count if it has drifted from 6.

---

## 4. DoD gates (cargo + e2e)

Per `feedback_plan_dod_dry_run_at_write.md`. **All DoD commands MUST be wrapper-prefixed (`cmd //c "scripts\brehon\cargo-<verb>.bat ..."`) per the v1-deps-r1 DQ `a3d0e9941441-011` clarify (Windows + libpq.dll + vcvars discipline).** Per-task `commands[]` arrays for `validate-pending-laptop` DQ entries are class-targeted per DQ `a3d0e9941441-016` (see §4a below).

**Task 1 (webmention inline replacement — production code + 3 callers untouched):**
```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"          # must exit 0
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"    # must exit 0
cmd //c "scripts\brehon\cargo-test.bat -p lemmy_api_utils --features full --lib"  # targeted lib-test for send_webmention
```

**Task 1 e2e gate** (validate-pending-laptop-e2e per `feedback_laptop_default_for_validate_pending.md` + `feedback_validate_pending_laptop_must_use_wrapper.md` + `feedback_windows_e2e_requires_bat_wrapper.md`):
```
cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-deps-r2-task1-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-deps-r2-task1-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-deps-r2-task1-e2e.log"
```
(run_in_background: true; ~26 min on the laptop)

**Task 2 (mdurl `[patch.crates-io]` fork redirect — workspace `Cargo.toml` + `Cargo.lock` only):**
```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"
```
No e2e gate for Task 2 in isolation — phase-tip e2e gate covers both Task 1 and Task 2.

**Task 3 (astral-tokio-tar dismissals — GitHub UI only, no code change):**
- DoD: `gh api repos/barrie-cork/lemmy/dependabot/alerts/57 --jq .state` returns `"dismissed"` (the PAX header GHSA)
- DoD: `gh api repos/barrie-cork/lemmy/dependabot/alerts/58 --jq .state` returns `"dismissed"` (the chmod-via-symlink GHSA)
- No cargo gate (no code change).

**Task 4 (wasmtime defer — `kind: "log"` DQ entry + 13× Dependabot dismissals + roadmap update — no code change):**
- DoD: 13× `gh api repos/barrie-cork/lemmy/dependabot/alerts/<N> --jq .state` returns `"dismissed"` for each wasmtime alert (numbers 37-47, 60 per current alert listing — re-enumerate at lane-cut time).
- DoD: `.claude/decision-queue.json` contains the `kind: "log"` deferral entry on `phase-v1-deps-r2`.
- DoD: `.claude/PRPs/v1-roadmap.json` `lanes.deps.sub_phases` contains a new `v1-deps-r3` entry with status `unstarted` and `depends_on: ["upstream: extism wasmtime release"]`.
- No cargo gate (no code change).

**Phase-tip e2e gate (post-Task 4, pre-PR):** full workspace e2e per the validate-pending-laptop pattern. Per `v1-deps-r1` precedent (DQ `a3d0e9941441-015` clarify), Shape G stays SUSPENDED through 2026-06-01 — if the lane runs past June 1, advisor files a `kind: "log"` DQ at the boundary and subsequent impl-tasks raise `kind: "validate-pending"` (Shape G) instead of `validate-pending-laptop`; plan §15 does NOT need re-authoring (polling loop handles both kinds transparently). Result MUST be `pass` for every test (no skips, no flakes) before opening the PR.

**Phase-tip Dependabot count gate (post-Task 4, pre-PR):**
```
gh api repos/barrie-cork/lemmy/dependabot/alerts --paginate -q '[.[] | select(.state == "open")] | length'
```
MUST return `0` (all 18 alerts closed: 6 via code/dispute, 12 via wasmtime dismissals — actually 13 if total wasmtime alerts is 13; verify count). If non-zero, surface to user before opening the PR.

### 4a. Per-task class-targeted `commands[]` (per DQ `a3d0e9941441-016` from v1-deps-r1)

| Task | Class | `commands[]` shape (each wrapper-prefixed per DQ 011) |
|---|---|---|
| T1 | Production code rewrite (1 file, narrow surface) | check + clippy + lib-test for `lemmy_api_utils` + e2e (raise as `validate-pending-laptop-e2e` kind) |
| T2 | Workspace `Cargo.toml` patch entry (no behavioral change) | check + clippy |
| T3 | GitHub UI dismissals only (no code change) | no cargo DoD; `gh api` verification |
| T4 | DQ entry + roadmap edit (no code change in `crates/` or `migrations/`) | no cargo DoD; `gh api` verification + DQ presence check |
| Phase-tip (post-T4, pre-PR) | Full workspace e2e single gate + Dependabot count gate | `cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full"` + `gh api ... --jq length == 0` |

This replaces four per-task e2e runs with one phase-tip e2e gate (saves ~78 min lane-time).

---

## 5. Lesson injections (mandatory, per advisor-orchestrator §2.4)

**Inject for Task 1 (webmention inline replacement touching api_utils/src/utils.rs):**
- `feedback_fix_impl_enumerate_all_callsites.md` — `rg "webmention::|Webmention::|WebmentionError|send_webmention" crates/` before authoring; planner MUST cite the actual count vs the brief's 6-reference count. Drift is the failure mode (per WP-8).
- `feedback_multi_write_handlers_need_transactions.md` — analogue: the inline replacement is a semantic-preserving rewrite. **No silent widening or narrowing of behavior** (per WP-1's 7-behavior enumeration). The `is_valid_ip` SSRF guard MUST be preserved verbatim.
- `feedback_async_pool_test_pattern.md` — if Task 1 adds a focused unit test for endpoint discovery against a testcontainers HTTP mock, use the canonical async pool pattern.
- `feedback_lemmy_error_no_std_error.md` — Case A discipline (LemmyResult<()> outer) for any new test fn added by Task 1.
- `feedback_carry_patch_todos.md` — N/A for Task 1 (no `[patch.crates-io]`); applies to Task 2.

**Inject for Task 2 (mdurl fork + `[patch.crates-io]`):**
- `feedback_carry_patch_todos.md` — `TODO(brehon-fork): upstream this` comment above the patch line; `kind: "log"` DQ entry with upstream-PR URL once filed; entry in `feedback_carry_patch_inventory.md` (or create the inventory lesson if it doesn't exist).
- `feedback_carry_patch_inventory.md` (companion) — new patch entry: `mdurl idna 0.3 → 1.1`, retirement condition `upstream rlidwka/mdurl.rs merges idna 1.x bump`.

**Inject for Task 4 (wasmtime defer DQ):**
- `feedback_dq_v3_append_via_helper_script.md` — use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json>` for the deferral entry; do NOT hand-write JSON inline (per the v1-dq-schema-r1 discipline).
- `feedback_principles_not_rules.md` — the deferral rationale lives in the DQ entry's `answer` text, not in the dismissal-button's free-text field. Dismissal text references the DQ entry ID.

**Inject for all tasks:**
- `feedback_clippy_test_style.md` — `#![deny(unwrap, expect)]` applies; use `?` throughout.
- `feedback_features_full_workspace_only.md` — every cargo gate above uses `--workspace --features full`; never `-p lemmy_server --features full` (the recurring footgun).
- `feedback_features_full_p_crate_incompatible.md` — companion to the workspace-only rule; `-p <crate> --features full` is structurally broken in this workspace except where the crate defines a `full` feature (Task 1's `-p lemmy_api_utils --features full --lib` is valid because `lemmy_api_utils` defines `full`; verify at plan-author time).
- `pattern_cargo_feature_flag_propagation.md` — covers feature unification edge cases.
- `feedback_validate_pending_laptop_must_use_wrapper.md` — Windows wrapper discipline for every cargo invocation in `commands[]` arrays.
- `feedback_targeted_validate_pending_laptop_commands.md` — per-task class-targeted command selection.
- `feedback_windows_e2e_requires_bat_wrapper.md` — e2e invocation on Windows requires `cmd //c "scripts\brehon\cargo-test.bat ..."` for libpq.dll discovery.

**Do NOT inject** `feedback_junior_worker_e2e_edit_hang.md` — Task 1's edits are in `api_utils/src/utils.rs` (a ~1000-line file, well within Edit-safe size); no e2e.rs edits in this phase.

---

## 6. Scope boundaries (stop-and-ask tripwires)

- **Stop if** the `webmention` crate has been bumped past 0.6.0 between brief-author time and lane-cut time AND the new version drops `reqwest 0.11` (`cargo info webmention` at lane-cut time). If so, the inline replacement may be unnecessary — file `kind: "blocker"` DQ asking whether to bump-and-keep vs inline-and-drop.
- **Stop if** the `is_valid_ip` SSRF guard's behavior has changed between brief-author time and lane-cut time (`git log governance-v0 -- crates/api/api_utils/src/utils.rs | grep is_valid_ip`). The inline replacement MUST keep the guard semantically identical.
- **Stop if** RT-r3 (or another in-flight lane) has added a new callsite of `send_webmention` whose call shape differs from the current 3 (file `kind: "blocker"` DQ — the function signature may need extending).
- **Stop if** any of the 13 wasmtime alerts turns out to NOT be aarch64-only (re-verify each GHSA's "Affected configurations" field at lane-cut time). If a wasmtime CVE is x86_64-exploitable, escalate to user — the deferral rationale changes.
- **Stop if** the `mdurl` upstream has released a new version that bumps `idna` (`cargo info mdurl` at lane-cut time). If so, fork is unnecessary — bump the workspace pin directly.
- **Stop if** the workspace `Cargo.toml`'s `reqwest` feature set does NOT include `rustls-tls` (WP-2 check). The inline replacement would re-introduce a different TLS stack — file `kind: "blocker"` DQ asking whether to add `rustls-tls` to the workspace pin or scope-add it to `api_utils`'s local entry.
- **Stop if** the cargo gates run locally on Windows fail for reasons other than the rewrite itself (libpq env, vcvars, etc) — fix the wrapper, not the dependency. Per `pre-phase-harness-audit.md` Probe 4.

---

## 7. Not in scope for v1-deps-r2

- **wasmtime upgrade (deferred to `v1-deps-r3`)** — the brief explicitly defers per user decision 2026-05-25. Task 4 of this phase records the deferral; the upgrade itself is the next phase.
- Major-version bumps that Dependabot has not surfaced yet (e.g. `markdown-it 0.7+` if pre-released).
- Removal of the `markdown-it` crate or replacement with `pulldown-cmark` (separate refactor decision, out of scope).
- Removal of the `Webmention` feature entirely (a product decision; current scope is preserve-feature-replace-implementation).
- Any change to the W3C Webmention protocol handling beyond inline-replacement-with-spec-fidelity.
- Workspace-wide rustc edition migration — separate concern.
- Removal of the `scripts/brehon/cargo-*` wrappers — wrappers stay.

---

## 8. Plan structure guidance

The plan should have 4 §13 tasks. **Task 0 MUST include the four wrapper-probes from `pre-phase-harness-audit.md` §1 AND the clippy baseline capture from §3** (per v1-deps-r1 DQ `a3d0e9941441-014` clarify) — non-zero baseline → planner adds a `chore(lint):` pre-task before Task 1 to clear pre-existing debt:

| Task # | Deliverable | Files | `[P]`? |
|---|---|---|---|
| T0 | Pre-phase harness audit + clippy baseline capture + re-enumerate webmention callsites + re-enumerate Dependabot alerts | `.claude/PRPs/debug/v1-deps-r2-task0-*.log` (diagnostic) | No |
| T1 | webmention inline replacement | `crates/api/api_utils/src/utils.rs` (1 file, ~80 LOC net add), `crates/api/api_utils/Cargo.toml` (remove webmention dep) | No |
| T2 | mdurl fork + `[patch.crates-io]` workspace entry | `Cargo.toml` (add `[patch.crates-io]` table + 1 line), `Cargo.lock` (auto), `.claude/lessons/feedback_carry_patch_inventory.md` (1 row added) | No — depends on T1's `Cargo.lock` settling |
| T3 | astral-tokio-tar 2× Dependabot dismissals | GitHub Security UI (no code) | Yes — `[P]` parallel to T1+T2 (independent surface) |
| T4 | wasmtime defer: 13× Dependabot dismissals + `kind: "log"` DQ + roadmap update | `.claude/decision-queue.json`, `.claude/PRPs/v1-roadmap.json`, GitHub Security UI | Yes — `[P]` parallel to T1+T2+T3 |

**Wait — T3 + T4 `[P]` would normally allow parallel dispatch with T1.** But per `feedback_explicit_file_arrays_on_tasks.md` YAML overlap check: T1 + T2 share `Cargo.lock` (T1's webmention removal modifies it; T2's `[patch.crates-io]` modifies it). T1 + T2 MUST be serial. T3 + T4 share nothing with T1/T2 (T3 = GitHub UI only; T4 = `.claude/` files only). T3 + T4 CAN run `[P]` parallel to each other and to T1 (T1's `Cargo.lock` edit and T4's `decision-queue.json` edit don't intersect).

Recommended cohort plan:
- **Cohort 1:** T0 alone (pre-phase audit).
- **Cohort 2:** T1 + T3 + T4 dispatched in parallel `[P]`. T2 deferred to Cohort 3.
- **Cohort 3:** T2 alone (after T1's `Cargo.lock` settles).
- **Phase-tip gate:** e2e + Dependabot count.

Cohort 2's `[P]` deserves a planner sanity check: are T3 and T4's "no code" status really file-disjoint from T1's edit? T3 is purely GitHub API calls (zero file touches in this lane's worktree). T4 touches `.claude/decision-queue.json` + `.claude/PRPs/v1-roadmap.json` — neither file is touched by T1. Verified disjoint. `[P]` is safe.

Plan complexity: **4/10**. Estimated wall-clock: 1-2 days (T1 is the bulk at ~3-4 hours impl + 26 min e2e; T2 ~1-2 hours; T3 + T4 each ~30 min). Cohort 2 parallelism saves ~1 hour wall-clock.

---

## 9. Pre-queue checklist (advisor to run before dispatching planning Junior)

- [ ] **Hard precondition check:** v1-deps-r1 merged (PR #153 `ab1e79a70` confirmed 2026-05-25); retro signed off (confirmed).
- [ ] **RT-r3 conflict check:** `git diff origin/governance-v0..origin/phase-v1-RT-r3 -- crates/api/api_utils/src/utils.rs` — confirm no overlap with `send_webmention` body or `webmention` imports. If overlap, surface to user.
- [ ] **Alert count re-verification:** `gh api repos/barrie-cork/lemmy/dependabot/alerts --paginate -q '[.[] | select(.state == "open")] | length'` — should return 18 (or note drift in the brief commit body).
- [ ] **Webmention callsite count re-verification:** `rg "webmention::|Webmention::|WebmentionError|send_webmention" crates/ | wc -l` — should return 6.
- [ ] `/brehon-clarify` run on this brief — candidate clarify-DQs: (a) WP-3 HTML `<link>` fallback choice (`scraper` vs hand-roll vs skip), (b) WP-5 `[patch.crates-io]` commit-SHA vs branch-tip discipline, (c) WP-7 DQ entry format for the wasmtime deferral.
- [ ] `git -C C:/Users/barri/Developer/brehon-fork show governance-v0:.claude/PRPs/briefs/v1-deps-r2-planning-1.md` — must succeed (brief committed before dispatch).
- [ ] `memory_search_hybrid("webmention reqwest inline replacement", limit: 5)` + `memory_search_hybrid("[patch.crates-io] fork upstream tracking", limit: 5)` — check for lessons authored since 2026-05-23.

---

## 10. Background: why this is scoped now

GitHub Dependabot reports 18 open alerts (2 critical + 1 high + 10 medium + 5 low) against `Cargo.lock` on `barrie-cork/lemmy:governance-v0` as of 2026-05-25. Investigation (per user prompt 2026-05-25 + Perplexity research 2026-05-25):

- The v1-deps-r1 phase (PR #153, merged same day) addressed the **Dependabot PR #136 bundle** (12 SemVer bumps + 2 breaking changes). It did NOT touch the four vulnerable transitive chains that produce the 18 alerts.
- Every upstream crate of each vulnerable transitive is **already at its latest stable version**: `webmention 0.6.0` (latest), `markdown-it 0.6.1` (latest), `testcontainers 0.27.3` (latest), `extism 1.21.0` (latest). So the alerts are not closable by `cargo update` — they require either upstream coordination (filing PRs / waiting for releases), forking (mdurl), inlining (webmention), or dismissal-with-evidence (astral-tokio-tar 0.6.0 IS already the fix; wasmtime is aarch64-only).
- User decision (2026-05-25, after Perplexity research):
  - **webmention** → inline replacement (Recommended path: ~70 LOC reqwest 0.13 implementation).
  - **mdurl/idna** → fork + `[patch.crates-io]` (Recommended path: 1-2 hours work).
  - **astral-tokio-tar** → dispute as already-fixed (no code change).
  - **wasmtime** → defer to v1-deps-r3 with documented suppressions (2 critical CVEs are aarch64-only; Brehon x86_64 deployment is not exploitable).

This brief is the durable record of those decisions.

Roadmap entry: add to `.claude/PRPs/v1-roadmap.json` `lanes.deps.sub_phases`:
- `v1-deps-r2`: status `unstarted` → flips `in_flight` at `/roadmap-next` cut → `done` post-merge.
- `v1-deps-r3`: status `unstarted`, `depends_on: ["upstream: extism wasmtime release"]`, scope `"wasmtime 41.x → 44.x via extism upgrade or fork+patch"`.
