---
name: validate-pending-laptop commands must go through scripts\brehon\cargo-*.bat wrappers
description: On Windows, raw cargo invocations in DQ commands[] arrays cause STATUS_DLL_NOT_FOUND on test binaries because libpq.dll isn't on PATH at runtime. Use the brehon wrappers.
type: feedback
---

# Wrapper discipline for `validate-pending-laptop[-e2e]` DQ entries on Windows

When the advisor authors a `kind: "validate-pending-laptop"` or `kind: "validate-pending-laptop-e2e"` DQ entry on the laptop (Windows), every cargo invocation in the `commands` array MUST go through the brehon wrapper scripts:

| Raw command | Wrapper-prefixed |
|---|---|
| `cargo check ...` | `cmd //c "scripts\\brehon\\cargo-check.bat ..."` |
| `cargo clippy ...` | `cmd //c "scripts\\brehon\\cargo-clippy.bat ..."` |
| `cargo test ...` | `cmd //c "scripts\\brehon\\cargo-test.bat ..."` |

**Why:** the wrappers do three things raw cargo cannot do on Windows:

1. Call `vcvars64.bat` so `link.exe` + MSVC runtime libs are on PATH (**link time**).
2. Set `PQ_LIB_DIR` and `PQ_INCLUDE_DIR` to vcpkg's libpq install (**build.rs + link time**).
3. Prepend `vcpkg\installed\x64-windows\bin` to PATH so `libpq.dll` is discoverable at **test runtime** (not just link time).

Raw `cargo test --workspace --features full --test e2e -- --test-threads=1` will COMPILE successfully (link succeeds because env vars persist from prior wrapper invocations or pq-sys cache), but the test binary will exit with `STATUS_DLL_NOT_FOUND (0xc0000135)` because libpq.dll is not on the spawning shell's PATH. cargo's outer exit code masks this — cargo prints `error: test failed` while the wrapper-shell exit code can still come back as 0 in some configurations, deceiving the validate-pending-laptop polling.

## How to apply

- When raising `validate-pending-laptop[-e2e]` on Windows, populate the `commands` array with wrapper-prefixed strings, e.g.:

  ```json
  "commands": [
    "cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --workspace --features full --test e2e -- --test-threads=1\""
  ]
  ```

- The validate-pending-laptop handler in `.claude/rules/advisor-orchestrator.md` runs each command in `commands[]` verbatim — author the wrapper prefix at write-time, not after the failure.

- Same rule applies to ad-hoc local cargo runs the advisor performs outside DQ-driven validation (DoD smoke tests, CR fix-in-PR cycles, diagnostic runs).

## Generalises to

- Any laptop-side cargo invocation by the advisor (interactive or DQ-driven). The wrapper preserves the libpq.dll PATH prepend that raw cargo cannot.
- **Does NOT apply to** EliteDesk Linux Junior workers — apt-installed `libpq-dev` is discovered via `pkg-config`, no wrapper needed there. Junior `[role:impl-task]` briefs targeting Shape G GH Actions also don't need wrappers (CI runs Linux runners). Pre-Shape-G plans (v1-JM-d and earlier) running cargo on the laptop ARE in scope.

## Symptom to recognise

Background cargo task reports `exit code 0` BUT the tee'd log ends with:

```text
error: test failed, to rerun pass `-p lemmy_server --test e2e`

Caused by:
  process didn't exit successfully: `target\debug\deps\e2e-<hash>.exe --test-threads=1` (exit code: 0xc0000135, STATUS_DLL_NOT_FOUND)
```

That is libpq.dll not on PATH at runtime. Tests didn't actually run — this is an environment failure, not a test fail/pass signal. Don't classify as test failure under §G4. Re-run under the wrapper.

## Confirmed cycle

2026-05-01, DQ #105: first attempt used raw cargo `cargo test --workspace --features full --test e2e -- --test-threads=1`, hit STATUS_DLL_NOT_FOUND after ~26 min compile. Re-launched under `scripts\brehon\cargo-test.bat` (cold rebuild because env differs from raw-cargo run). Lesson promoted to repo (this file) + PMD pattern memory #240.

## Related

- `.claude/rules/advisor-orchestrator.md` "validate-pending-laptop handler" sub-section.
- `scripts/brehon/cargo-test.bat`, `cargo-check.bat`, `cargo-clippy.bat` — the wrappers.
- `feedback_pq_sys_stale_cache.md` — adjacent pq-sys env discussion.
- `feedback_pq_sys_wrapper_env_propagation.md` — adjacent wrapper env discussion.
- `feedback_features_full_workspace_only.md` — `--features full` requires `--workspace`.
