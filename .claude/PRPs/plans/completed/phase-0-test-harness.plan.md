---
phase: 0
title: Test harness — testcontainers + empty e2e.rs
status: ready
estimated_iterations: 1
target_command: /prp-implement
plan_path: .claude/PRPs/plans/phase-0-test-harness.plan.md
depends_on: []
unblocks: [phase-1-schema, phase-2-read-models, phase-3-api-common, phase-4-endpoints, phase-5-reputation, phase-6-federation]
---

# Phase 0 — Test harness

## Goal

Stand up the integration-test harness that every later phase's
definition-of-done depends on. After this phase, `cargo test --test e2e`
must return green with at least one real test that boots Postgres in a
container, connects to it, and tears it down.

This phase exists because [IMPLEMENTATION-PLAN-v0.md](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)
treats the test harness as a one-line task inside Phase 1 (§3 Phase 1
task 12), but in reality it's a Docker-Postgres integration problem and
the precondition for ralph-loop validation. Building it as Phase 0
unblocks all autonomous-loop phases and keeps the loop's stop hook
honest — until `cargo test --test e2e` returns *something*, ralph cannot
distinguish "test passes" from "test doesn't exist."

**Non-goals.** Do NOT add governance schema. Do NOT create migrations.
Do NOT add `db_schema` governance modules. Do NOT touch any crate
outside the test harness itself. This is plumbing only.

## Hard constraints

- **No new top-level crates** unless a workspace-level test crate is
  the only sensible home. Prefer adding `tests/e2e.rs` to an existing
  crate. Recommend `crates/server/tests/e2e.rs` (the binary crate is
  the natural home for end-to-end tests). Confirm by checking what's
  already in `crates/server/tests/`; if the directory exists, use it.
- **Use `testcontainers-rs`** as the Postgres harness. Version `0.20`
  or later — the API stabilised at 0.20.
- **Use `postgres:16-alpine`** to match the Lemmy production image
  (verify against `docker-compose.yml` or `crates/db_schema/Cargo.toml`
  if Lemmy pins a different version — match what Lemmy already uses).
- **No async runtime drama.** `testcontainers` ships sync and async
  APIs. Pick whichever matches Lemmy's existing test style — most
  Lemmy crates use `tokio`, so use the async API with
  `#[tokio::test]`.
- **No Diesel migrations in this phase.** The test only needs to
  prove the container boots and a `tokio_postgres` (or `sqlx`, or
  whatever lemmy already pulls in) connection succeeds. Schema work
  is Phase 1.
- **Do not break upstream tests.** `cargo test --workspace` must
  still pass after this phase. The new `e2e` test target is additive.
- **CRLF / line endings.** New files must be LF — match
  `.gitattributes` if it exists.

## Tasks

Numbered, one commit each. Each task ends with the exact validation
command that proves it's done.

### 1. Survey existing test infrastructure

Read-only. Before writing anything:

- `ls crates/server/tests/` — does the directory exist? what's in it?
- `grep -r "testcontainers" Cargo.toml crates/*/Cargo.toml` — already
  used anywhere?
- `grep -r "tokio-test" crates/*/Cargo.toml` — what async test runtime
  does Lemmy use?
- `grep -rn "postgres:" docker-compose.yml docker/ scripts/ 2>/dev/null`
  — what Postgres image and version does Lemmy use?
- Read `crates/server/Cargo.toml` to see the existing dev-dependencies
  block.

Output: a one-paragraph summary in the progress log of (a) where
`tests/e2e.rs` should live, (b) which async runtime to use, (c) which
Postgres image tag to pin.

**Validation:** none — survey only. Do not commit.

### 2. Add testcontainers to dev-dependencies

Edit the right `Cargo.toml` (probably `crates/server/Cargo.toml`,
confirmed by task 1):

```toml
[dev-dependencies]
testcontainers = "0.20"
testcontainers-modules = { version = "0.8", features = ["postgres"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Match versions to whatever Lemmy already pins for `tokio` — do not
introduce a second tokio version. If Lemmy uses workspace-level
dependency management (`[workspace.dependencies]` in the root
`Cargo.toml`), add testcontainers there and reference with
`testcontainers.workspace = true` in the per-crate `Cargo.toml`.
Match the existing pattern; don't introduce a new one.

**Validation:** `cargo check -p server --tests` (or whichever crate
you edited) — must compile.

### 3. Create the empty e2e.rs test target

Create `crates/server/tests/e2e.rs` (path confirmed by task 1) with
exactly this content:

```rust
//! End-to-end test harness for the Brehon governance fork.
//!
//! This file is the entry point for `cargo test --test e2e`. Phase 0
//! establishes the harness; later phases add real golden-path tests.

use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};

#[tokio::test]
async fn postgres_container_boots() {
    let container = Postgres::default()
        .start()
        .await
        .expect("failed to start postgres container");

    let host_port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("failed to get mapped postgres port");

    assert!(host_port > 0, "postgres mapped port should be non-zero");
}
```

This is deliberately the smallest test that actually exercises the
harness. It proves: (a) Docker is reachable, (b) testcontainers can
pull and start the image, (c) the container exposes a port, (d) the
async runtime works. No connection, no schema, no governance — those
all come later.

**Validation:** `cargo test --test e2e --no-run` — must compile.

### 4. Run the test

```bash
cargo test --test e2e -- --nocapture
```

Must pass. If it fails, debug:
- `docker info` — is Docker reachable?
- Does the user have permission to run containers?
- Did `testcontainers` pull the image? (first run is slow.)
- Is the postgres image tag correct?

Do not paper over failures. If Docker isn't available in the loop
environment, **stop** and surface that as a blocker — Phase 0 cannot
complete without a working Docker socket, and no later phase will
either.

**Validation:** `cargo test --test e2e` exits 0.

### 5. Verify the workspace still builds and tests still pass

```bash
cargo check --workspace
cargo test --workspace --exclude server -- --skip e2e
```

The exclusion is to keep iteration time sane — we already ran the e2e
test in task 4. The point of this task is to confirm that adding
testcontainers as a dev-dependency didn't break any other crate's
build or test compile.

**Validation:** both commands exit 0.

### 6. Document the harness

Add a short `crates/server/tests/README.md` (max 30 lines):

- What `cargo test --test e2e` does
- Docker prerequisite
- How to run a single test (`cargo test --test e2e <test_name>`)
- A note that future phases will add real tests here

Do not write more than 30 lines. This is a pointer file, not
documentation.

**Validation:** file exists, file is under 30 lines.

### 7. Commit

```bash
git add crates/server/tests/e2e.rs \
        crates/server/tests/README.md \
        crates/server/Cargo.toml \
        Cargo.toml \
        Cargo.lock
git commit -m "feat(test): phase 0 — testcontainers e2e harness

Establishes cargo test --test e2e as the integration-test entry point
for the governance fork. Boots a Postgres 16 container and asserts the
mapped port is reachable. No schema, no governance code — pure plumbing
to unblock Phase 1+.

Prerequisite for IMPLEMENTATION-PLAN-v0.md Phases 1–6."
```

**Validation:** `git status` clean, `git log -1 --stat` shows the
expected files.

## Phase definition-of-done

All of the following must be true:

1. `crates/server/tests/e2e.rs` exists and is non-empty
2. `cargo test --test e2e` exits 0
3. `cargo check --workspace` exits 0
4. `cargo test --workspace --exclude server -- --skip e2e` exits 0
5. `git status` is clean
6. The commit from task 7 is on `HEAD` of `governance-v0`

If any of those are false, the phase is not done. Do not emit
`<promise>COMPLETE</promise>`.

## Cross-cutting checks

None for Phase 0. The hash-chain, `actor_pseudonym`, and
`EmergencyRemove` cross-cutters from the implementation plan §4 do
not apply until Phase 1 lands schema.

## Risks

- **Docker unavailable in the run environment.** Hard blocker. If
  `docker info` fails, stop and surface — this phase cannot complete
  and no later phase will either.
- **Tokio version conflict.** If Lemmy pins an old tokio (1.0.x)
  testcontainers 0.20 may demand newer. Resolve by matching whatever
  Lemmy uses; do not pin two tokio versions.
- **First-run image pull slow.** The first `cargo test --test e2e`
  may take 60–120s while testcontainers pulls `postgres:16-alpine`.
  Subsequent runs are fast. Don't time-out the loop on iteration 1.
- **Image tag drift.** If Lemmy pins a different Postgres image
  (e.g. `postgres:16` without `-alpine`, or a custom image), match
  that — Phase 1 migrations will be tested against the same image,
  and version skew between harness and prod is a footgun.

## Out of scope (deferred)

- Migration runner inside the harness — Phase 1 task
- Seeded fixtures — Phase 2 task
- Golden-path test — Phase 4 task
- CI integration (GitHub Actions workflow that runs `cargo test
  --test e2e`) — Phase 1 §2.4 task 9, deferred so this phase can
  ship without touching CI config
- Parallel test isolation (each test gets its own container) —
  add when a Phase 5+ test demands it

## Definition of "do not start"

Do not start this plan if:

- Docker is not running on the host
- The current branch is not `governance-v0`
- `cargo check --workspace` is currently red (fix the baseline first)
- There are uncommitted changes in `crates/server/` (commit or stash)
