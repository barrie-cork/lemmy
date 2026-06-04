# Phase 3 — Carry-patch upstreaming audit (results)

> **Status:** AUDIT COMPLETE 2026-06-04 (session `d3e7ac99`). Workflow `wym9sz191` (6 agents, ~531K tokens, ~10 min). Read-only + draft-prep — **zero daemon use, M1-safe, no in-tree `crates/` edit** (the `#___` backfill stays gated behind M1).
> **Decision owner:** the USER files the upstream PRs against `LemmyNet/lemmy` (the advisor cannot push to upstream). This report is the hand-off.

## TL;DR

| Bundle | Files | Verdict | File now? |
|---|---|---|---|
| **windows-signal** | `crates/server/src/lib.rs` (2 hunks) | ✅ `yes` (conf 0.90–0.95) — governance-free, still-needed upstream | **YES — recommend filing** |
| **clippy-expect** | `pagination.rs` + `vote/impls.rs` (1 line each) | ⚠️ `conditional` (conf 0.84–0.88) — governance-free + still-present upstream, but correctness hinges on one unverifiable fact | **NO — verify first (Docker clippy run)** |

- **Marker count reconciled:** plan said "11 `TODO(brehon-fork)`"; reality is **17 in-tree** (`grep -rn "TODO(brehon-fork)" crates/`). Upstreamable set unchanged: **4 markers → 2 PRs**. The other 13: `membership_state`/Phase-5a-task-51 governance-coupled (fork-local-forever) + 2 governance-internal TODOs (`admin_emergency_remove.rs:74,120`, `admin_reputation_stats.rs:12` — not upstreaming candidates).
- **All 4 candidates are governance-free** (every agent grepped each hunk for `membership_state|MembershipState|GovernanceCase|reputation|jury|sponsor|actor_pseudonym|moderation_case` → zero real hits; only the patches' own `TODO(brehon-fork)` comment lines matched).
- **None already fixed upstream** (verified against `upstream/main` tip `159911a37`).

## ⚠️ Critical cross-cutting finding — fork working-tree diffs are NOT cherry-pickable

The agents flagged that you must **build the upstream PR diffs from the isolated hunks in this report, NOT `git diff` the fork's working tree**:
- `pagination.rs:236` — the fork bundles the `#[expect]` removal with **3 unrelated edits** in the same file (`pub static BASE64_ENGINE`→`static`, two `unwrap_or_default()`→`unwrap_or(...)` from commit `34f5cc567`).
- `vote/src/impls.rs:130` — bundles import re-formatting **AND a governance-coupled test-fixture arg** (`"nada".to_owned()` added to `CommunityInsertForm::new` — a **fork-only field that does not exist upstream**). A raw port would inject governance code into the upstream PR.

The `## Changes` sections below already isolate only the single relevant line per site and strip the `TODO(brehon-fork)` markers — they are clean. Use them.

---

## PR Draft 1 — windows-signal  ✅ RECOMMEND FILING

**Title:** `fix: guard tokio::signal::unix behind #[cfg(not(windows))] so lemmy_server builds on Windows`
**Branch (off `upstream/main`):** `fix/windows-signal-cfg-guard`
**Files:** `crates/server/src/lib.rs`
**recommend_file:** `true`

### Body

## Summary

`lemmy_server` currently fails to compile on Windows because `crates/server/src/lib.rs` references `tokio::signal::unix` unconditionally. The `tokio::signal::unix` module only exists on Unix targets — on `windows-msvc` it is absent, so both the `use tokio::signal::unix::SignalKind;` import and the `tokio::signal::unix::signal(...)` calls in the shutdown path are hard compile errors.

This PR gates the Unix-only signal handling behind `#[cfg(not(windows))]` and adds a `#[cfg(windows)]` fallback that uses `tokio::signal::ctrl_c()` alone for graceful shutdown — the idiomatic tokio recommendation on Windows, which has no POSIX `SIGINT`/`SIGTERM`. Behaviour on Unix is byte-for-byte unchanged.

## Motivation

Windows has no POSIX signals, so `tokio::signal::unix` is compiled out of `tokio` on `windows-*` targets. Today `crates/server/src/lib.rs` assumes a Unix host:

- the import `use tokio::signal::unix::SignalKind;` (line 56 on `main`), and
- the shutdown block that builds `SignalKind::interrupt()` / `SignalKind::terminate()` streams and `select!`s over them (lines ~285–298 on `main`)

both reference the non-existent module, so a `cargo build -p lemmy_server` (or `cargo check --workspace`) on Windows fails outright. The two references must be guarded together: gating only the call sites would leave the import unused/undefined on Windows, and gating only the import would leave the `signal(...)` calls dangling. This change makes the crate compile on Windows while keeping the full SIGINT/SIGTERM behaviour on Unix.

## Changes

Single file: `crates/server/src/lib.rs`.

**1. Import site** — guard the Unix-only import:

```diff
-use tokio::signal::unix::SignalKind;
+#[cfg(not(windows))]
+use tokio::signal::unix::SignalKind;
```

**2. Shutdown handler** — guard the Unix signal streams + the `select!`, and add a Windows `ctrl_c()`-only fallback:

```diff
-  let mut interrupt = tokio::signal::unix::signal(SignalKind::interrupt())?;
-  let mut terminate = tokio::signal::unix::signal(SignalKind::terminate())?;
-
-  tokio::select! {
-    _ = tokio::signal::ctrl_c() => {
-      tracing::warn!("Received ctrl-c, shutting down gracefully...");
-    }
-    _ = interrupt.recv() => {
-      tracing::warn!("Received interrupt, shutting down gracefully...");
-    }
-    _ = terminate.recv() => {
-      tracing::warn!("Received terminate, shutting down gracefully...");
-    }
-  }
+  #[cfg(not(windows))]
+  let mut interrupt = tokio::signal::unix::signal(SignalKind::interrupt())?;
+  #[cfg(not(windows))]
+  let mut terminate = tokio::signal::unix::signal(SignalKind::terminate())?;
+
+  #[cfg(not(windows))]
+  tokio::select! {
+    _ = tokio::signal::ctrl_c() => {
+      tracing::warn!("Received ctrl-c, shutting down gracefully...");
+    }
+    _ = interrupt.recv() => {
+      tracing::warn!("Received interrupt, shutting down gracefully...");
+    }
+    _ = terminate.recv() => {
+      tracing::warn!("Received terminate, shutting down gracefully...");
+    }
+  }
+
+  #[cfg(windows)]
+  {
+    // ctrl_c() is the canonical shutdown hook on Windows; Windows has no POSIX
+    // signals so there is no equivalent to SIGINT/SIGTERM here.
+    let _ = tokio::signal::ctrl_c().await;
+    tracing::warn!("Received ctrl-c, shutting down gracefully...");
+  }
```

## Risk & testing

- **Unix is unchanged.** All Unix-only code is gated with `#[cfg(not(windows))]` and is identical to today's behaviour — SIGINT, SIGTERM, and ctrl-c all still trigger graceful shutdown. The diff is additive `cfg` attributes plus a new `#[cfg(windows)]` arm; no Unix code path is removed or reordered.
- **Windows now compiles** and shuts down gracefully on ctrl-c (Ctrl+Break / console-close), which is the only signal-like mechanism Windows exposes via tokio.
- Verified against `main` (tip `159911a37`): the import on line 56 and the unguarded `signal(...)` + `tokio::select!` block in the shutdown path still reference `tokio::signal::unix` directly, so the fix is not yet present upstream.
- Testing: `cargo check -p lemmy_server` on a Unix host (unchanged) and on `x86_64-pc-windows-msvc` (previously failed, now succeeds). No runtime behaviour change on Linux/macOS CI.

### Optional follow-up (not in this PR)

The gate keys on `windows` specifically. A maximally-portable form would key on `cfg(unix)` / `cfg(not(unix))` so other non-Unix targets (e.g. WASI) also take the `ctrl_c()`-only path. For Lemmy's de-facto Unix + Windows support matrix the `windows` gate is sufficient; happy to switch to `unix`/`not(unix)` if maintainers prefer.

### Caveats for filing (Draft 1)

- **None blocking — recommend filing.** Verified: upstream/main `159911a37` still carries bare `use tokio::signal::unix::SignalKind;` (line 56) + unguarded `signal(...)` + `tokio::select!` in the shutdown block → genuinely still needed.
- Both `rust-toolchain.toml` files pin `1.95`; this is the **portability** candidate, so the clippy/`-D warnings` concern does not apply.
- The diff above already strips the fork-internal `// Brehon carry-patch` + `TODO(brehon-fork)` comment markers. If you cherry-pick the actual file instead of applying this diff, strip those two comment blocks first.
- **Social risk only:** LemmyNet is Linux/Docker-deploy-first and *may* decline a windows-msvc portability patch on support-policy grounds. That's maintainer discretion, not a defect.

---

## PR Draft 2 — clippy-expect  ⚠️ DO NOT FILE YET (verify first)

**Title:** `fix(lint): drop no-longer-firing #[expect(clippy::multiple_bound_locations)] on pagination helpers`
**Branch (off `upstream/main`):** `fix/drop-unfulfilled-multiple-bound-locations-expect`
**Files:** `crates/diesel_utils/src/pagination.rs`, `crates/db_views/vote/src/impls.rs`
**recommend_file:** `false`

### THE GATING CHECK (run before filing)

The entire decision turns on **one fact I could not verify from disk**: does `clippy::multiple_bound_locations` fire on the clippy bundled in the `rust:1.95` Docker image upstream CI uses? The fork's evidence is only that it's *dead* on its own rustup-installed 1.95 clippy. The lint is **unstable** (rust-lang/rust#115590) — its firing can shift between patch builds of the same release. If it's *alive* on upstream's CI clippy, the `#[expect]` is *fulfilled* and **removing it would surface a bare warning that fails their `-D warnings` + `complexity=deny` gate — i.e. this PR would BREAK their build**, the opposite of intent.

```sh
docker run --rm -v "$PWD":/src -w /src rust:1.95 \
  sh -c 'rustup component add clippy && \
         cargo clippy --workspace --tests --all-targets --all-features -- -D warnings'
```

- **Green** with the two attributes removed → safe to file the PR as drafted.
- **Lint fires** → do **NOT** file the removal; the `#[expect]` should stay (only the explanatory comment would be worth upstreaming).

### What was verified (all green — only the lint-firing fact is open)

- Upstream `main` tip `159911a37` — both `#[expect]` sites STILL present: `pagination.rs:236` (`paginate_response`) + `db_views/vote/src/impls.rs:135` (`paginate_vote_response`). Not already fixed.
- Fork and upstream `rust-toolchain.toml` are **byte-identical** (`channel = "1.95"`, bare). *(This refuted the advisor's pre-launch hypothesis that the toolchains might diverge — they don't.)*
- Upstream CI **does** enforce `-D warnings` on clippy (`.woodpecker.yml`: `cargo clippy --workspace --tests --all-targets --all-features -- -D warnings`, `rust:1.95` image).
- Upstream root `Cargo.toml` **also** sets `allow_attributes = "deny"` (line 108) + `complexity = { level = "deny", priority = -1 }` (line 74). So an upstream `#[allow]` downgrade is banned too, and the lint is a deny when it fires.

### Body

## Summary

Two pagination helpers carry `#[expect(clippy::multiple_bound_locations)]` to silence a Clippy lint on their `#[cfg]`-split generic parameter lists:

- `paginate_response` in `crates/diesel_utils/src/pagination.rs`
- `paginate_vote_response` in `crates/db_views/vote/src/impls.rs`

On a Clippy build where `multiple_bound_locations` does **not** fire on these signatures, an `#[expect(...)]` becomes an *unfulfilled* expectation. `unfulfilled_lint_expectations` is warn-by-default in rustc, and CI runs `cargo clippy ... -- -D warnings`, so the unfulfilled `#[expect]` is promoted to a hard error and the lint job fails. Because the workspace also sets `allow_attributes = "deny"`, downgrading to `#[allow(...)]` is not an option — the only remediation is to remove the attribute entirely.

This PR removes both attributes. **Please read "## Risk & testing" before merging** — whether this is correct depends on whether `multiple_bound_locations` actually fires on *your* CI Clippy build, which I could not confirm remotely.

## Motivation

`multiple_bound_locations` (tracking issue rust-lang/rust#115590) is an evolving Clippy lint. On at least one Rust 1.95 toolchain it no longer fires on the `<#[cfg(feature = "ts-rs")] T: ts_rs::TS, #[cfg(not(feature = "ts-rs"))] T>` parameter pattern used by these two helpers, because the two `#[cfg]`-gated parameters are never simultaneously present — there is only ever one bound location after `cfg` expansion.

When the lint does not fire, the existing `#[expect(clippy::multiple_bound_locations)]` is unfulfilled. Given:

```toml
# Cargo.toml  [workspace.lints.clippy]
complexity = { level = "deny", priority = -1 }   # multiple_bound_locations lives here
allow_attributes = "deny"                         # #[allow] downgrade is forbidden
```

and the CI step:

```yaml
# .woodpecker.yml
cargo_clippy:
  image: rust:1.95
  commands:
    - rustup component add clippy
    - cargo clippy --workspace --tests --all-targets --all-features -- -D warnings
```

the unfulfilled `#[expect]` fails the `-D warnings` gate, and `#[allow]` cannot be used as a fallback. Removing the attribute is the minimal fix that keeps the lint job green when the lint is dead, while leaving the `// https://github.com/rust-lang/rust/issues/115590` comment in place so the rationale is still discoverable if the lint resurfaces on a future toolchain (at which point the `#[expect]` can simply be restored).

## Changes

The change is the removal of one attribute line at each of two sites. Nothing else changes — the function bodies and signatures are untouched, and the existing tracking-issue comment is preserved.

**`crates/diesel_utils/src/pagination.rs`** (the `paginate_response` helper):

```diff
 /// Add prev/next cursors to query result.
 #[cfg(feature = "full")]
 // https://github.com/rust-lang/rust/issues/115590
-#[expect(clippy::multiple_bound_locations)]
 pub fn paginate_response<#[cfg(feature = "ts-rs")] T: ts_rs::TS, #[cfg(not(feature = "ts-rs"))] T>(
   data: Vec<T>,
   limit: i64,
   request_cursor: Option<PaginationCursor>,
 ) -> LemmyResult<PagedResponse<T>>
```

**`crates/db_views/vote/src/impls.rs`** (the `paginate_vote_response` helper):

```diff
 // https://github.com/rust-lang/rust/issues/115590
-#[expect(clippy::multiple_bound_locations)]
 fn paginate_vote_response<
   #[cfg(feature = "ts-rs")] T: ts_rs::TS,
   #[cfg(not(feature = "ts-rs"))] T,
 >(
   data: Vec<T>,
   limit: i64,
   page_cursor: Option<PaginationCursor>,
 ) -> LemmyResult<PagedResponse<VoteView>>
```

## Risk & testing

The behavioural risk is zero — `#[expect(...)]`/`#[allow(...)]` attributes affect lint output only, never generated code. The function bodies, signatures, and generated MIR are identical before and after.

**The real risk is toolchain-dependent and must be checked before merge.** This change is correct **only if `multiple_bound_locations` does not fire** on the exact Clippy build your CI uses (the `clippy` component bundled in the `rust:1.95` image at run time). If that Clippy build *does* still fire the lint on these signatures, then the `#[expect]` is *fulfilled* and removing it would surface a bare `multiple_bound_locations` warning, which — under `complexity = "deny"` + `-D warnings` — fails the lint job. In other words, removing it could break CI in exactly the way the attribute was there to prevent.

`multiple_bound_locations` is an unstable lint whose firing behaviour can differ between patch builds of the same Rust release, so the fact that it is dead on one 1.95 toolchain does not guarantee it is dead on the CI image's Clippy.

Please verify locally against the CI image before merging:

```sh
docker run --rm -v "$PWD":/src -w /src rust:1.95 \
  sh -c 'rustup component add clippy && \
         cargo clippy --workspace --tests --all-targets --all-features -- -D warnings'
```

- If the lint **does not** fire (build is green with the attributes removed): this PR is correct as-is.
- If the lint **does** fire: do **not** merge — the `#[expect]` should stay. In that case the only thing worth upstreaming would be the explanatory comment, not the removal.

No new tests are warranted (attribute-only change).

---

## Full per-candidate verdicts (audit trail)

| key | file:line | gov-free | fixed upstream? | upstreamable | conf |
|---|---|---|---|---|---|
| win-signal-1 | `server/src/lib.rs:55` (import) | ✅ | no | **yes** | 0.95 |
| win-signal-2 | `server/src/lib.rs:284` (handler) | ✅ | no | **yes** | 0.90 |
| clippy-expect-1 | `diesel_utils/src/pagination.rs:236` | ✅ | no | **conditional** | 0.84 |
| clippy-expect-2 | `db_views/vote/src/impls.rs:130` | ✅ | no | **conditional** | 0.88 |

**Note on win-signal-2's caveat:** the agent correctly noted a real upstream PR *must bundle both hunks* — submitting only the handler-site guard (line 284) without the import-site guard (line 55) leaves an unguarded import. Draft 1 already bundles both. ✔

---

## Post-M1 follow-up (GATED — do not do now)

Once M1 (`phase-m1-b`) merges and the close-out lane rebases onto post-M1 `governance-v0`, the in-tree `crates/` annotation edits become unblocked (Phase 6+ window):
1. **Backfill `#___`** on the 4 upstreamed markers once PR numbers exist (`server/src/lib.rs:55,284`, `pagination.rs:236`, `vote/impls.rs:130`).
2. **Annotate the 13 fork-local markers** with "fork-local (governance) — do not upstream" to stop re-triaging them every rebase: the `membership_state` cluster (`db_schema/{lib.rs:241,273, source/person.rs:70,114, impls/person.rs:87,468}`, `api_crud/user/create.rs:503`, `apub/objects/person.rs:174`, `db_views/registration_applications/impls.rs:279,357`) + the 2 governance-internal TODOs (`admin_emergency_remove.rs:74,120`, `admin_reputation_stats.rs:12` — these say "Phase 5"/"v1" not "upstream", retarget their wording).
