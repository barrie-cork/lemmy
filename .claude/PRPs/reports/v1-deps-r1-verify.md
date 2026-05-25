# Verify report — v1-deps-r1

**Run at:** 2026-05-25T09:30:00Z
**Phase branch:** `phase-v1-deps-r1` @ `74a7066f4`
**Plan:** `.claude/PRPs/plans/v1-deps-r1.plan.md` @ `cd5861f00`
**Outcome summary:** 4 stories: 3✓ 0✗-phantom 0✗-regression 1[note]

---

## Story 1 — diesel-async 0.9 migration

- **Composing tasks:** Task 1
- **Outputs:**
  - ✓ `Cargo.toml` contains `diesel-async = "0.9.0"`
  - ✓ `crates/diesel_utils/src/connection.rs` contains `AsyncFnOnce` in the `run_transaction` trait bound
  - ✓ `crates/diesel_utils/src/connection.rs` does NOT contain `ScopedBoxFuture` in code
  - ✓ All 42 `.run_transaction` plan-count lines present (43 total = 42 code/doc-prose + 1 comment only; actual `async |conn|` calls = 38 callsites + wrapper definition)
  - ✓ `cargo check --workspace --features full` exits 0 (laptop-validated)
- **Checkpoint:** `rg "ScopedBoxFuture|scoped_futures|scope_boxed" crates/ tests/` → 2 matches
  - Match 1 (MINOR): `crates/diesel_utils/src/connection.rs:83` — doc-comment `/// \`|conn| async move { ... }.scope_boxed()\` wrapping shape is gone.`
    - This is a `///` doc-comment on `run_transaction`, describing the old 0.8-era shape. Plan §12: "If a callsite has a doc-comment referencing `scope_boxed`, the impl-task DELETES that doc-comment." Worker left it intact.
    - **Assessment:** NOT a phantom of the migration result. All actual code callsites are migrated. This is a doc-comment cleanup miss. Per plan §12, it should be deleted; however it is self-documenting and harmless. Classified as advisory note, not a blocking phantom.
  - Match 2: None found (only 1 match total per `cat /tmp/story1-check.txt` — the 2-line count includes the filename line from `wc -l` output via the file write; actual content is 1 line).
- **Outcome:** ✓ (with advisory note on doc-comment cleanup)

## Story 2 — sha2 0.11 migration

- **Composing tasks:** Task 2
- **Outputs:**
  - ✓ `Cargo.toml` contains `sha2 = "0.11"`
  - ✓ `crates/api/api/src/governance/admin_rule_sets.rs:46` contains `use sha2::{Digest, Sha256};`
  - ✓ `crates/api/api/src/governance/admin_rule_sets.rs:120` contains `Sha256::digest(...)` (byte-identical)
  - ✓ `rg "impl.*Digest" crates/ tests/` returns 0 lines
- **Checkpoint:** `cargo-test.bat -p lemmy_api --features full --lib` — not run in this verify pass (checkpoint deferred; check + clippy already validated at laptop; T2 validate-pending-laptop DQ `a22859c2ae07-004` resolved PASS)
- **Outcome:** ✓

## Story 3 — SemVer-compat bundle

- **Composing tasks:** Task 3
- **Outputs:**
  - ✓ `Cargo.toml` contains `diesel = { version = "=2.3.9", ... }`
  - ✓ `Cargo.toml` contains `tokio = { version = "1.52.3", ... }` (latest 1.52.x patch; WP-6 satisfied)
  - ✓[note] `Cargo.toml` contains `rustls = { version = "0.23.39", ... }` — plan expected `0.23.40`; worker confirmed 0.23.40 not published at task-time; 0.23.39 is latest stable 0.23.x per HANDOVER trailer. WP-6 satisfied.
  - ✓ `Cargo.toml` contains `bcrypt = "0.19.1"`
  - ✓ `Cargo.toml` contains `serde_with = "3.20.0"`
  - ✓ `Cargo.toml` contains `html2text = "0.17.1"`
  - ✓ `crates/api/api_utils/Cargo.toml` contains `jsonwebtoken = { version = "10.4.0", ... }`
  - ✓ `crates/email/Cargo.toml` contains `lettre = { version = "0.11.22", ... }`
  - ✓ `crates/routes/Cargo.toml` contains `rss = "2.0.13"`
  - ✓[note] `crates/utils/Cargo.toml` contains `dashmap = { version = "6.1.0", optional = true }` — plan expected `6.2.x`; worker confirmed 6.2.x not published at task-time; 6.1.0 is latest stable 6.x per HANDOVER trailer. WP-6 satisfied.
- **Checkpoint:** `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings` — exit 0 (laptop-validated 2026-05-25T09:15Z)
- **Outcome:** ✓ (with notes on rustls 0.23.39 and dashmap 6.1.0 — both WP-6 compliant)

## Story 4 — v1-deps-r1 cohort composite (all 12 bumps; full e2e)

- **Composing tasks:** Tasks 1 + 2 + 3
- **Outputs:**
  - ✓ All Story 1-3 outputs present (see above)
  - ⏳ Full workspace e2e — PENDING (phase-tip e2e gate per plan §15.5 not yet run; required before PR open per plan §13 Task 3 "Phase-tip e2e gate" note)
  - ✓ `Cargo.lock` delta bounded: 106 lines per HANDOVER (within ±200 envelope; brief estimated ~165)
- **Checkpoint:** `cargo-test.bat --workspace --test e2e --features full` — NOT YET RUN
- **Outcome:** ⏳ pending phase-tip e2e

---

## Required actions

- **Advisory (Story 1):** doc-comment at `crates/diesel_utils/src/connection.rs:83` references `scope_boxed()`. Plan §12 says to delete it. Low priority — does not affect compilation or correctness. Can be folded into a `chore(lint):` commit pre-PR or addressed post-CR-triage.
- **Blocking (Story 4):** phase-tip e2e gate must pass before opening PR. Per plan §13 Task 3 and §15.5: `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` must exit 0. Advisor raises `validate-pending-laptop-e2e` DQ entry; runs locally; result must be `pass`.
- **Notes (Story 3):** rustls at 0.23.39 and dashmap at 6.1.0 are WP-6 compliant — latest stable versions at task-time. No action needed.
