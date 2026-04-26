---
name: pq-sys wrapper-env poisoning (cargo-check.bat then cargo-test.bat)
description: cargo-check.bat does not set PQ_LIB_DIR; pq-sys build.rs caches Err(NotPresent) and subsequent cargo-test.bat runs inherit the stale cache, causing LINK1181 libpq.lib missing.
type: feedback
originSessionId: 50ffed48-0dd6-4010-8ffb-d915ca13314c
---
On Windows, running `scripts\brehon\cargo-check.bat` **before** `scripts\brehon\cargo-test.bat` in the same target directory poisons `pq-sys`'s build-script cache:

1. `cargo-check.bat` does not export `PQ_LIB_DIR`.
2. Cargo runs `pq-sys`'s build.rs with `PQ_LIB_DIR=Err(NotPresent)` and writes that result (plus an empty `cargo:rustc-link-search`) to `target/debug/build/pq-sys-<hash>/output`.
3. Subsequent `cargo-test.bat` invocations DO set `PQ_LIB_DIR` but cargo's fingerprint says the build script doesn't need to re-run; the stale cached output is reused.
4. Linker fails with `LNK1181: cannot open input file 'libpq.lib'` because no link-search path was emitted for vcpkg's libpq.

**Why:** discovered 2026-04-18 during the first weekly upstream rebase (`d1975776a` into `governance-v0`). `cargo check --workspace --features full` exited 0 but the follow-up `cargo test --test e2e --no-run -p lemmy_server` failed at link despite the wrapper setting `PQ_LIB_DIR`. Inspecting `target/debug/build/pq-sys-*/output` showed `"PQ_LIB_DIR" = Err(NotPresent)` and no `rustc-link-search` directive.

**How to apply:**
- If you plan to run both check AND test against the same target dir (fresh worktree or post-clean), either (a) set `PQ_LIB_DIR` in `cargo-check.bat` too, or (b) after check but before the first test run, do `rm -rf target/debug/build/pq-sys-* target/debug/deps/pq_sys-*` to force a rebuild.
- When debugging `LNK1181: libpq.lib` errors, cat `target/debug/build/pq-sys-*/output` FIRST. If it shows `PQ_LIB_DIR = Err(NotPresent)`, the cache is poisoned — clean pq-sys and re-run.
- Long-term fix is a one-line edit to `cargo-check.bat` copying the `set VCPKG_ROOT=... / set PQ_LIB_DIR=... / set PQ_INCLUDE_DIR=... / set PATH=...` block from `cargo-test.bat`. That parity eliminates the cache-poisoning failure mode entirely.

Do NOT confuse this with the `feedback_cargo_invocations.md` topic (which covers vcvars + vcpkg install). The install is correct; the wrapper parity is what fails.
