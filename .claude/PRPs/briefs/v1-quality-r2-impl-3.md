# Brief: impl-task 3 — v1-quality-r2 C1 fixtures-module process-env safety doc-comment audit (14 modules)

**Role:** `[role:impl-task]`
**Phase:** `v1-quality-r2` (branch `phase-v1-quality-r2`)
**Authored:** 2026-05-29
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base branch (Junior forks from):** `phase-v1-quality-r2`
**Lane mode:** Mode B (mobile remote-control). Junior worker runs on the EliteDesk daemon (Linux).
**Closes:** GitHub issue #156.
**Plan task:** §13 Task 3 (Story 3). Anchors re-derived LIVE from `crates/server/tests/e2e.rs` on `origin/phase-v1-quality-r2` @ `34dd093fc` (file = 18095 lines; plan §10.7 line numbers were captured 2026-05-28 against trunk `1edb8b94c` and are STALE — use the anchors in §2 below, not the plan's line numbers).

---

## 1. Role + dispatch

`[role:impl-task] v1-quality-r2 task 3 — fixtures doc-comment audit — see .claude/PRPs/briefs/v1-quality-r2-impl-3.md`

---

## 2. Scope

**Doc-comment-only edit. No behavioural change, no compile-relevant change.** Add a process-env safety doc-comment header to all 14 `mod *_fixtures` blocks in `crates/server/tests/e2e.rs`, citing the mandatory `--test-threads=1` constraint. Each test in these modules mutates process-wide env vars; the safety of those mutations depends on the single-threaded test runner.

This is a **mechanical sweep of 14 verbatim Edit pairs**, applied in order. Every `old_string`/`new_string` pair is pre-located below (§2.2). Do NOT search for anchors yourself — paste the pairs verbatim.

### 2.1 The constraint-citation block (identical for all 14 modules)

Insert this exact 5-line doc-comment block (note: NO line-number reference — `EnvVarGuard` is hoisted to crate root in a later task, so the block cites the symbol by name, not a line):

```rust
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
```

**Two insertion shapes** (each module's pair below tells you which applies):

- **Shape NO-DOC (modules 1–13):** the module currently has NO `//!` block — `mod X {` is immediately followed by a `use` line. Insert the 5-line block between the `{` and the first `use` line, followed by a blank `//!`-free line is NOT added (the block sits directly above the first `use`).
- **Shape APPEND (module 14, `v1_rt_r3_fixtures`):** the module ALREADY has a `//!` block. APPEND the constraint citation to the existing block (do not replace the existing lines); the new lines go after the existing final `//!` line and before the blank line preceding `use super::*;`.

### 2.2 The 14 verbatim Edit pairs (apply in this order)

> **Anchor discipline (R11):** each `old_string` below is the verbatim current text on the phase tip. The `mod X_fixtures {` line makes each anchor unique in the file. If ANY `old_string` does not match the live file at Edit time (e.g. a concurrent merge shifted text), STOP — do NOT fuzzy-match — and raise a `kind: "blocker"` DQ per §4. Do not proceed past a non-matching anchor.

---

**Edit 1 — `mod governance_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod governance_fixtures {
  use actix_web::web::Data;
```
new_string:
```rust
mod governance_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use actix_web::web::Data;
```

---

**Edit 2 — `mod admin_config_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod admin_config_fixtures {
  use actix_web::web::Data;
```
new_string:
```rust
mod admin_config_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use actix_web::web::Data;
```

---

**Edit 3 — `mod v1_jm_b_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod v1_jm_b_fixtures {
  use chrono::{Duration as ChronoDuration, Utc};
```
new_string:
```rust
mod v1_jm_b_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use chrono::{Duration as ChronoDuration, Utc};
```

---

**Edit 4 — `mod v1_jm_e_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod v1_jm_e_fixtures {
  use actix_web::web::Json;
```
new_string:
```rust
mod v1_jm_e_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use actix_web::web::Json;
```

---

**Edit 5 — `mod v1_sl_b_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod v1_sl_b_fixtures {
  use super::*;
  use actix_web::web::Json;
```
new_string:
```rust
mod v1_sl_b_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::*;
  use actix_web::web::Json;
```

---

**Edit 6 — `mod v1_sl_c_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod v1_sl_c_fixtures {
  use super::*;
  use chrono::{Duration, Utc};
```
new_string:
```rust
mod v1_sl_c_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::*;
  use chrono::{Duration, Utc};
```

---

**Edit 7 — `mod v1_sl_d_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod v1_sl_d_fixtures {
  use super::*;
  use activitypub_federation::config::FederationConfig;
```
new_string:
```rust
mod v1_sl_d_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::*;
  use activitypub_federation::config::FederationConfig;
```

---

**Edit 8 — `mod v1_sl_e_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod v1_sl_e_fixtures {
  use super::*;
  use activitypub_federation::config::FederationConfig;
  use actix_web::web::{Data, Json};
```
new_string:
```rust
mod v1_sl_e_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::*;
  use activitypub_federation::config::FederationConfig;
  use actix_web::web::{Data, Json};
```

> **Edit 8 anchor note:** `mod v1_sl_d_fixtures` (Edit 7) and `mod v1_sl_e_fixtures` (Edit 8) BOTH begin `use super::*;\n  use activitypub_federation::config::FederationConfig;`. The third line disambiguates: `v1_sl_e` has `use actix_web::web::{Data, Json};` as its third line. The `mod <name> {` first line already makes each anchor unique, but the third line is included for safety.

---

**Edit 9 — `mod v1_federation_inbound_a_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod v1_federation_inbound_a_fixtures {
  use super::*;
  use diesel::ExpressionMethods;
```
new_string:
```rust
mod v1_federation_inbound_a_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::*;
  use diesel::ExpressionMethods;
```

---

**Edit 10 — `mod v1_ship_2_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod v1_ship_2_fixtures {
  use super::*;
  use actix_web::{App, test, web::Data};
```
new_string:
```rust
mod v1_ship_2_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::*;
  use actix_web::{App, test, web::Data};
```

---

**Edit 11 — `mod v1_federation_inbound_b_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod v1_federation_inbound_b_fixtures {
  use super::*;
  use activitypub_federation::config::FederationConfig;
  use activitypub_federation::traits::Activity as ActivityTrait;
```
new_string:
```rust
mod v1_federation_inbound_b_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::*;
  use activitypub_federation::config::FederationConfig;
  use activitypub_federation::traits::Activity as ActivityTrait;
```

---

**Edit 12 — `mod v1_federation_inbound_e_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod v1_federation_inbound_e_fixtures {
  use super::*;
  use activitypub_federation::config::FederationConfig;
  use activitypub_federation::traits::Activity as ActivityTrait;
  use diesel::{ExpressionMethods, QueryDsl};
```
new_string:
```rust
mod v1_federation_inbound_e_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::*;
  use activitypub_federation::config::FederationConfig;
  use activitypub_federation::traits::Activity as ActivityTrait;
  use diesel::{ExpressionMethods, QueryDsl};
```

> **Edit 11 vs Edit 12 anchor note:** `v1_federation_inbound_b` and `v1_federation_inbound_e` both begin `use super::*;\n  use activitypub_federation::config::FederationConfig;\n  use activitypub_federation::traits::Activity as ActivityTrait;`. The `mod <name> {` first line disambiguates them; Edit 12 additionally includes the 4th line (`use diesel::{ExpressionMethods, QueryDsl};`) which `inbound_b` does NOT have (inbound_b's 4th line is `use actix_web::error::ResponseError;`).

---

**Edit 13 — `mod v1_ship_3_fixtures`** (Shape NO-DOC)

old_string:
```rust
mod v1_ship_3_fixtures {
  use super::*;
  use actix_web::web::{Data, Json};
  use diesel::{Connection as _, ExpressionMethods, PgConnection, QueryDsl};
```
new_string:
```rust
mod v1_ship_3_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::*;
  use actix_web::web::{Data, Json};
  use diesel::{Connection as _, ExpressionMethods, PgConnection, QueryDsl};
```

---

**Edit 14 — `mod v1_rt_r3_fixtures`** (Shape APPEND — this module ALREADY has a `//!` block; append, do not replace)

old_string:
```rust
  //! Advisor-authored carve-out per cycle-count §5.3 hard-refusal on Junior dispatch
  //! (DQ a3d0e9941441-033). One-time exception to "advisor never authors crates/**".

  use super::*;
```
new_string:
```rust
  //! Advisor-authored carve-out per cycle-count §5.3 hard-refusal on Junior dispatch
  //! (DQ a3d0e9941441-033). One-time exception to "advisor never authors crates/**".
  //!
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.

  use super::*;
```

> **Edit 14 note:** this is the ONLY APPEND-shape edit. The `old_string` anchors on the existing block's final two `//!` lines + the blank line + `use super::*;` — this 4-line span is unique in the file (only `v1_rt_r3_fixtures` carries the "Advisor-authored carve-out" text). The constraint citation is inserted after a separating blank `//!` line, preserving the existing block.

### 2.3 Out of scope (do NOT do)

- Do NOT touch any `use` statements, function bodies, test logic, or `EnvVarGuard` itself.
- Do NOT add a `--test-threads=1` flag to any config file (that is already set; this task only DOCUMENTS the constraint).
- Do NOT edit `mod *_fixtures` blocks that already cite `--test-threads=1` in their header beyond the 14 enumerated (Probe 8 confirmed exactly 14 `*_fixtures` modules — no more, no less).
- Do NOT reorder modules or `use` lines.
- Do NOT edit any file other than `crates/server/tests/e2e.rs`.

---

## 3. Required reading

Read these BEFORE making any edit:

1. **`.claude/PRPs/plans/v1-quality-r2.plan.md`** §10.7 (the 14-module list + suggested citation form) + §13 Task 3 (the task contract, ACTION/IMPLEMENT/VALIDATE/GOTCHA) + §16a Story 3 (the verify checkpoint).
2. **`.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md`** — pre-locate verbatim anchors discipline (the canonical e2e-edit-hang-prevention lesson). **This brief already pre-locates all 14 anchors in §2.2 — apply them verbatim; do NOT re-search.**
3. **`.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`** — why full-file Edits into the ~18K-line e2e.rs hang the worker; each Edit here is a small, anchored, doc-only change (≤10 lines of context). Apply edits one at a time, top to bottom.
4. **`.claude/lessons/feedback_lemmy_error_no_std_error.md`** — error-shape discipline for e2e.rs (mandatory per advisor-orchestrator §2.4 for any e2e.rs edit). **NOTE: this is a doc-only task — no error-handling code changes — but the lesson is injected per the mechanical file-class rule; you will not need its recipe unless a compile surfaces a pre-existing error.**
5. **`.claude/lessons/feedback_async_pool_test_pattern.md`** — async pool/conn test fixture pattern (mandatory per advisor-orchestrator §2.4 for any e2e.rs edit). **NOTE: doc-only task; injected per the mechanical rule; reference only.**
6. **`.claude/rules/decision-queue.md`** §"Mid-task visibility" + Hard refusals — for the mid-task push protocol if you raise a blocker.

> Lessons 4 and 5 are injected per the advisor-orchestrator §2.4 mandatory file-class table (`crates/server/tests/e2e.rs` with ≥2 edits → `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` + `feedback_fix_impl_pre_locate_e2e_anchors.md`). This task is doc-comment-only, so their recipes are reference-only — but the mechanical rule fires on the file pattern regardless of edit shape.

---

## 4. Constraints

1. **Anchor-match discipline (R11):** apply the 14 `old_string`/`new_string` pairs from §2.2 verbatim, in order. If any `old_string` does not match the live file, STOP at that edit — do NOT fuzzy-match or improvise an anchor. Raise a `kind: "blocker"` DQ (`from: "impl"`) naming the module whose anchor failed, commit + push it to the worker branch immediately (per `.claude/rules/decision-queue.md` "Mid-task visibility"), and halt. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id.

2. **One file only:** every edit lands in `crates/server/tests/e2e.rs`. Touching any other file is out of scope — STOP and raise a blocker if you believe another file needs editing.

3. **Edit one module at a time, top to bottom** (Edit 1 → Edit 14). Do not batch all 14 into a single multi-region Edit (that is the e2e.rs edit-hang failure mode per lesson 3).

4. **Validate-pending-laptop DoD (Shape G is SUSPENDED through 2026-06-01).** After all 14 edits, run the VALIDATE block from plan §13 Task 3 on the daemon (Linux wrappers `scripts/brehon/cargo-*.sh`, NOT the `.bat` siblings):
   - `bash scripts/brehon/cargo-check.sh --workspace --features full` → EXPECT exit 0
   - `bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings` → EXPECT exit 0
   - `bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server --features full` → EXPECT exit 0 (R7 cheap insurance; this is a doc change so compile must still pass)
   - The Python audit from §13 Task 3 VALIDATE → EXPECT `OK — all 14 fixtures modules cite --test-threads=1 in doc-comment`
   Capture each to `.claude/PRPs/debug/v1-quality-r2-task3-<gate>.log`. **Do NOT run the full e2e suite** (that is e2e-execution, raised separately at T5 as `validate-pending-laptop-e2e` and run by the advisor on the laptop).

5. **Raise the validate handoff, do not self-validate cargo as final.** After the local check/clippy/test-norun/audit gates pass on the daemon, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`) per `.claude/rules/decision-queue.md`, with `commands` = the four gate commands above (verbatim, with `--features full` + scope flags), `branch` = your worker branch, `phase_task` = 3. Commit + push it. The advisor re-runs the gates on the laptop and mutates the entry. **If any local gate fails on the daemon, do NOT raise validate-pending — instead patch (if the failure is a doc-comment typo in your own edit) or raise a `kind: "blocker"` (if the failure is pre-existing / out of scope).**

6. **Commit discipline.** One commit for all 14 doc edits. Subject (verbatim from plan §13 Task 3): `style(e2e): audit fixtures-module process-env safety doc-comments (closes #156, task 3)`. Push to your worker branch (`junior/...`), NOT to `phase-v1-quality-r2` directly (the daemon finalize-merges).

7. **No `#[allow]`-spam, no `--no-verify`, no force-push.** Per `.claude/rules/no-destructive-defaults.md` + `.claude/rules/circuit-breaker.md`.

8. **LESSON trailer.** If you discover a durable pattern (e.g. an anchor that drifted, a module shape the brief missed), end your commit body with `LESSON: <one-line observation>` per `.claude/rules/pmd-invariants.md` invariant #4. The advisor harvests at retro.

---

## 5. Definition of done

- All 14 `mod *_fixtures` blocks in `crates/server/tests/e2e.rs` contain the `--test-threads=1` constraint citation in their doc-comment header.
- `cargo check --workspace --features full` exits 0 (doc change must not break compile).
- `cargo clippy --workspace --features full --no-deps -- -D warnings` exits 0 (no new `clippy::doc_lazy_continuation` or similar — the block is plain prose, no lists).
- `cargo test --test e2e --no-run -p lemmy_server --features full` exits 0.
- The §13 Task 3 Python audit prints `OK — all 14 fixtures modules cite --test-threads=1 in doc-comment`.
- One commit on the worker branch with the exact subject from §4.6.
- A `kind: "validate-pending-laptop"` DQ entry raised + pushed (or a `kind: "blocker"` if a gate failed out-of-scope).
