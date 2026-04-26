---
name: cargo --features full incompatible with -p lemmy_server
description: lemmy_server crate doesn't declare the `full` feature; `-p lemmy_server --features full` errors. Use `--workspace --features full` instead.
type: feedback
originSessionId: bae44467-63e7-4c21-ad41-057cefebb80e
---
`cargo check/test -p lemmy_server --features full` fails with:

```
error: the package 'lemmy_server' does not contain this feature: full
help: packages with the missing feature: lemmy_utils, lemmy_db_schema, ... (37 crates)
```

**Why:** `lemmy_server` crate's `Cargo.toml` does not declare a `full` feature. Most workspace crates do, but `-p <crate> --features <f>` activates the feature ON THAT CRATE; if the named crate doesn't have it, cargo errors out before compiling anything.

**How to apply:**
- Per-crate check on lemmy_server → drop `--features full`: `cargo check -p lemmy_server` (depends on default features activated transitively)
- Workspace-wide check with `full` → use `--workspace --features full`: activates `full` on every crate that declares it, skips crates that don't
- E2E test compile → `cargo test --workspace --features full --no-run` (NOT `-p lemmy_server --features full`)
- Related: `feedback_api_crud_oauth_feature_quirk.md` notes a similar per-crate oddity for `-p lemmy_api_crud`
- The wrapper scripts `scripts/brehon/cargo-*.bat` pass args through — the check lives in your invocation, not the wrapper

Discovered 2026-04-19 during Phase 5c task 68 recovery when advisor noted "`--features full` is incompatible with `-p <crate>`". First probe `cargo-test.bat --test e2e --no-run -p lemmy_server --features full` errored; switching to `--workspace --features full` green.
