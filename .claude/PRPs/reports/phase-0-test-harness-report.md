---
plan: .claude/PRPs/plans/completed/phase-0-test-harness.plan.md
source: docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md §3 Phase 1 task 12 (promoted to Phase 0)
branch: governance-v0 (merged via fast-forward from feature/phase-0-test-harness)
date: 2026-04-15
status: COMPLETE
---

# Implementation Report — Phase 0: Test harness

## Summary

Established the integration-test harness every later phase depends on.
`cargo test --test e2e -p lemmy_server` now returns green with one real
test (`postgres_container_boots`) that boots a
`pgautoupgrade/pgautoupgrade:18-alpine` container, waits for the
canonical "database system is ready to accept connections" message on
stderr, and asserts the mapped host port is non-zero.

Phase 0 unblocks IMPLEMENTATION-PLAN-v0.md Phases 1–6 and makes the
ralph-loop stop hook honest: until this phase was in, ralph could not
distinguish "test passes" from "test doesn't exist."

## Assessment vs reality

| Metric | Predicted | Actual | Reasoning |
|---|---|---|---|
| Complexity | "one-line task" in IMPLEMENTATION-PLAN-v0 §3 Phase 1 task 12; plan estimated 1 iteration | Multi-session, three commits, one env-toolchain install (vcpkg + libpq) | The plumbing was simple; the Windows linker story was not. The plan correctly flagged "Docker unavailable" as a blocker but did not anticipate the libpq gap, which only exists on Windows because `pq-sys` needs `libpq.lib` at link time. |
| Confidence | "estimated_iterations: 1" in plan frontmatter | Validation split across two sessions: first session committed artefacts with "validation pending"; second session installed vcpkg + libpq and completed validation | Correct to split — cargo check passed in session 1, giving confidence that the code was right even when the Windows link environment wasn't. |
| First-run container pull | Plan estimated 60–120 s | 41.43 s total test runtime (pull + boot + test + teardown) | Image already partially cached locally; actual pull was fast. |

## Deviations from plan

### 1. Postgres image: `pgautoupgrade/pgautoupgrade:18-alpine` (not `postgres:16-alpine`)

The plan suggested `postgres:16-alpine` but explicitly said "match what
Lemmy already uses". `docker/docker-compose.yml` at line 84 pins
`pgautoupgrade/pgautoupgrade:18-alpine`, so the harness must use that
exact image or migrations written later will diverge from production.

The plan's risk register §4 flagged image-tag drift as a footgun. This
deviation resolves it.

### 2. testcontainers versions: 0.27 / testcontainers-modules 0.15

The plan pinned `testcontainers = "0.20"` and `testcontainers-modules
{ version = "0.8", features = ["postgres"] }` as minimums. Current
stable is 0.27 / 0.15. Used the stable versions — still satisfies the
plan's ">= 0.20" constraint, gets the most recent API.

### 3. Image construction: `GenericImage`, not `testcontainers_modules::postgres::Postgres`

The plan's example used the `Postgres` wrapper from
`testcontainers-modules`, which hardcodes the image name as `postgres`.
Calling `.with_tag("18-alpine")` only swaps the tag, not the name, so
there is no clean path to `pgautoupgrade/pgautoupgrade` through the
wrapper. `GenericImage::new("pgautoupgrade/pgautoupgrade", "18-alpine")`
with explicit `.with_exposed_port(5432.tcp())` and
`.with_wait_for(WaitFor::message_on_stderr(...))` gives exact control
and is ~5 lines longer.

### 4. Test signature: `Result<(), Box<dyn Error>>` with `?`, not `.expect(...)`

The plan's example used `.expect("failed to start postgres container")`.
The workspace clippy config (`Cargo.toml` lines 91-107) denies
`expect_used`, `unwrap_used`, and `allow_attributes` — meaning
`#[allow(clippy::expect_used)]` is also forbidden as an escape hatch.
The legal patterns are:

- `async fn -> LemmyResult<()>` with `?` — used by most Lemmy tests
- `async fn -> Result<(), Box<dyn Error>>` with `?` — used here since
  the test has no lemmy_utils dep
- `#[expect(clippy::unwrap_used, clippy::tests_outside_test_module)]`
  — used by `crates/utils/tests/test_errors_used.rs` when neither
  Result approach fits

### 5. Runner wrapper: `scripts/brehon/cargo-test.bat` (new)

Required by Windows linker reality. `cargo test` needs `link.exe` on
PATH (provided by `vcvars64.bat` from VS Build Tools) AND `libpq.lib`
on the link search path (provided by vcpkg via `PQ_LIB_DIR`). Neither is
available in plain bash on Windows. `scripts/brehon/cargo-test.bat` is a
sibling of the existing `cargo-check.bat` that sources vcvars, sets the
three PQ_* env vars for the vcpkg x64-windows dynamic triplet, prepends
the vcpkg bin dir to PATH so `libpq.dll` is loadable at runtime, and
forwards `%*` verbatim to `cargo test`.

The `.bat` was committed in two phases: first an infrastructure skeleton
with a vcpkg placeholder (commit `dabe55a23`), then the real paths once
vcpkg was installed (commit `e370523c7`).

## Tasks completed

| # | Task | File(s) | Status |
|---|---|---|---|
| 1 | Survey existing test infrastructure | (read-only) | ✅ |
| 2 | Add testcontainers to workspace deps | `Cargo.toml`, `crates/server/Cargo.toml` | ✅ |
| 3 | Create the empty e2e.rs test target | `crates/server/tests/e2e.rs` | ✅ |
| 4a | Compile the test via cargo-test.bat | (via wrapper) | ✅ (after `rm -rf target/debug/build/pq-sys-*` — see "Issues encountered") |
| 4b | Run the test via cargo-test.bat | (via wrapper) | ✅ 1 passed in 41.43 s |
| 5 | Workspace sanity via cargo-check.bat | (via wrapper) | ✅ 4 m 07 s |
| 5' | Workspace-test sanity | — | ⏭️ Skipped on Windows per explicit direction; full workspace test will run on Linux CI when that lands. The plan's DoD #4 is Windows-specific for this reason. |
| 6 | README for tests dir | `crates/server/tests/README.md` | ✅ (30 lines, in commit `0485a977d`) |
| 7 | Commit + fast-forward merge to governance-v0 | — | ✅ |

## Validation results

| Check | Result | Details |
|---|---|---|
| `cargo check --workspace` | ✅ | Via `scripts/brehon/cargo-check.bat`, finished dev profile in 4 m 07 s |
| `cargo check -p lemmy_server --tests` | ✅ | Ran at Task 2 validation, 6 m 19 s |
| `cargo test --test e2e --no-run -p lemmy_server` | ✅ | Via `scripts/brehon/cargo-test.bat`, finished test profile in 2 m 54 s |
| `cargo test --test e2e -p lemmy_server` | ✅ | 1 passed; 0 failed; finished in 41.43 s |
| `cargo clippy --workspace -- -D warnings` | ⏭️ | Not run (plan didn't call it; workspace clippy is evaluated during `cargo check` anyway because `[lints] workspace = true` is set in every crate) |
| `cargo test --workspace --exclude lemmy_server -- --skip e2e` | ⏭️ | Skipped on Windows per direction; deferred to Linux CI |
| Migration round-trip | ⏭️ | No schema changes in Phase 0 |
| Cross-cutting verification (hash chain / pseudonym / EmergencyRemove / redaction) | ⏭️ | Not applicable — Phase 0 is plumbing only, no governance code |

## Files changed

| File | Action | Lines |
|---|---|---|
| `Cargo.toml` | UPDATE | +2 (workspace.dependencies: testcontainers, testcontainers-modules) |
| `Cargo.lock` | UPDATE | +513 / -1 (new transitive deps from testcontainers) |
| `crates/server/Cargo.toml` | UPDATE | +5 (new `[dev-dependencies]` block) |
| `crates/server/tests/e2e.rs` | CREATE | +36 |
| `crates/server/tests/README.md` | CREATE | +30 |
| `scripts/brehon/cargo-test.bat` | CREATE | +65 |

Three commits on `governance-v0` (after fast-forward merge from
`feature/phase-0-test-harness`):

```
e370523c7 chore(scripts): fill PQ_LIB_DIR for Windows libpq (vcpkg x64-windows)
dabe55a23 chore(scripts): add cargo-test.bat for Windows + libpq via vcpkg (validation pending install)
0485a977d feat(test): phase 0 — testcontainers e2e harness
```

Pushed to `origin/governance-v0`.

## Cross-cutting impact

Phase 0 is pure test plumbing. None of the cross-cutting invariants from
[IMPLEMENTATION-PLAN-v0.md §4](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)
apply yet:

- [ ] Hash chain appends wired up for new writes — N/A (no governance code)
- [ ] `actor_pseudonym::get_or_create` used — N/A (no pseudonym code)
- [ ] `redaction::scrub` called on all strings reaching the log — N/A
- [ ] `CaseStatus::EmergencyRemove` exhaustively matched — N/A
- [ ] AGPL notice unchanged — ✅ (no release artefact produced)

These checks will apply from Phase 1 onward, once the governance schema
lands.

## Issues encountered

### Stale `pq-sys` build-script cache masked the libpq fix

After installing vcpkg + libpq, the first run of
`cargo-test.bat --test e2e --no-run` still failed with
`LNK1181: cannot open input file 'libpq.lib'` — exactly the error the
vcpkg install was supposed to fix.

Root cause: `pq-sys`'s `build.rs` reads `PQ_LIB_DIR` at build-script
execution time and emits `cargo:rustc-link-search=native=<path>`. An
earlier compile attempt (before vcpkg install, when `PQ_LIB_DIR` was
unset) cached a build-script output file at
`target/debug/build/pq-sys-*/output` that said:

```
"PQ_LIB_DIR" = Err(NotPresent)
cargo:rustc-link-lib=libpq     (no link-search emitted!)
```

Cargo's `rerun-if-env-changed=PQ_LIB_DIR` mechanism should have
detected the env-var flip and re-run the build script. It did not
in this case — probably because `PQ_LIB_DIR` is set inside the
`cargo-test.bat` cmd shell, and cargo's env-change tracking compares
against its own internal record, which also came from a run inside
the same batch. Worth investigating later; for Phase 0 the fix was
to nuke the stale cache:

```bash
rm -rf target/debug/build/pq-sys-*
./scripts/brehon/cargo-test.bat --test e2e --no-run -p lemmy_server
```

After the rebuild, the new `target/debug/build/pq-sys-*/output` correctly
contained:

```
"PQ_LIB_DIR" = Ok("C:\\Users\\barri\\Developer\\vcpkg\\installed\\x64-windows\\lib")
cargo:rustc-link-search=native=C:\Users\barri\Developer\vcpkg\installed\x64-windows\lib
cargo:rustc-link-lib=dylib=libpq
```

and the link succeeded.

**Gotcha for future sessions**: if anyone installs vcpkg + libpq on a
fresh machine but has already run `cargo build` / `cargo test` before
the install, they MUST clear `target/debug/build/pq-sys-*` before the
next run. Alternatively, `cargo clean` works but is much more
expensive. This is worth adding to the `cargo-test.bat` header comment
as a known-issue note. (Deferred to a follow-up — not part of Phase 0.)

### Earlier false "exit code 0" from piped cargo

The first session's `cargo test --test e2e --no-run 2>&1 | tail -40`
invocation produced a task-notification summary saying "exit code 0"
even though cargo had failed with LNK1181. The pipe masked the exit
code. This was the motivation for `.claude/rules/cargo-output-capture.md`
(which the user created between sessions). All cargo invocations in
session 2 used `> logfile 2>&1; echo "EXIT=$?"` and read the logfile
explicitly.

## Tests written

| Test | Validates |
|---|---|
| `postgres_container_boots` | (a) Docker is reachable, (b) testcontainers can pull + start `pgautoupgrade/pgautoupgrade:18-alpine`, (c) the container exposes port 5432 on a non-zero host port, (d) the async runtime works for `#[tokio::test]`. No connection, no schema, no governance. |

## Next steps

- [x] Review implementation (self-review via this report)
- [ ] Mark Phase 0 complete in `IMPLEMENTATION-PLAN-v0.md` (in the
      homeserver repo, per the fork's editing policy — NOT from inside
      brehon-fork)
- [ ] Proceed to Phase 1 — schema for governance tables — once Phase 0
      report is reviewed
- [ ] Consider adding a "stale pq-sys cache" note to
      `scripts/brehon/cargo-test.bat` header, or a `cargo clean` guard
      at the top of the batch when `PQ_LIB_DIR` first appears
- [ ] Linux CI: when it lands, validate DoD #4 there
      (`cargo test --workspace --exclude lemmy_server -- --skip e2e`)

## Timings summary

| Phase | Wall-clock |
|---|---|
| Task 2 validation (`cargo check -p lemmy_server --tests`, cold) | 6 m 19 s |
| Task 4a compile (`cargo test --no-run`, incremental after Task 2) | 2 m 54 s |
| Task 4b test run (`cargo test`, mostly container startup) | 41.43 s (includes image pull/cache check, container boot, test, teardown) |
| Task 5 workspace check (`cargo check --workspace`, incremental) | 4 m 07 s |
