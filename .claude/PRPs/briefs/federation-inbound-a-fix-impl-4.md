---
phase: v1-federation-inbound-a
role: impl-task
kind: fix-impl
fix_impl_n: 4
authored: 2026-05-18
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
triggering_dq: 266
triggering_task: 9
cycle: "Task 9 §5.2 cycle 1 (error_class_history was empty; (E0432/E0425/E0433, e2e.rs) recorded as cycle 1). Does NOT trigger the ≥3 cycle-count meta-rule catch-fire."
classification: "§G4 ALLOWLIST — compound mechanical, governed by the canonical-sibling-mirror rule. Task 9 (#321) appended a NEW `mod v1_federation_inbound_a_fixtures` to e2e.rs but did NOT mirror the canonical sibling `mod v1_sl_e_fixtures` (Case A) as the Task-9 brief §2.2 explicitly mandated. It invented a non-existent governance_fixtures API (`start_postgres_with_migrations()`, `async_conn()` — neither exists; real API is `bootstrap()`/`start_postgres()`/`db_url()`), imported `InstanceId` from the wrong path (`lemmy_db_schema::newtypes` instead of `lemmy_db_schema_file`), and used a `.map_err(|e| LemmyErrorType::Unknown(...))` shape without importing `LemmyErrorType` (Case A uses bare `?`, no map_err). 7 compile errors, all mechanical divergences from the canonical sibling with a verbatim copy-target in the same file. §G4 rows: `error[E0432]: unresolved import` (add/fix the use per the canonical sibling) + row 4b (sibling fixtures module exists → mirror canonical Case A verbatim) + feedback_newtype_locations_lemmy_db_schema_vs_file + feedback_lemmy_error_no_std_error Case A. Allowlist (mechanical, single file e2e.rs, fix fully determined by the canonical sibling). Advisor diagnosed against the real governance_fixtures API + the canonical sibling v1_sl_e_fixtures before classifying. Retro carry-forward: Task-9 worker did not comply with brief §2.2's mandate to mirror the canonical sibling — recurrence-watch (same class as fix-impl-1 recipe defect: worker authored against an imagined API instead of the cited canonical instance)."
base: "phase-v1-federation-inbound-a @ b206673e9 (Task 9 impl 6ae091e90 + DQ #266 raise 46ecbd53d + daemon finalize-merge b206673e9; lane==origin==daemon-local synced)"
cap: "1 file edit (crates/server/tests/e2e.rs ONLY) — 3 anchor-Edits CONFINED to the existing `mod v1_federation_inbound_a_fixtures` block (e2e.rs lines 15091-15157, ~67 lines, well-separated at EOF). NEVER a full-file Read/rewrite (feedback_junior_worker_e2e_edit_hang)."
serial: "Cohort B strictly serial cap=1 — this fix-impl makes Task 9's §5.2 cmd 3 (cargo-test --test e2e --no-run) pass. Task 9 §5.2 (DQ #266) is re-run by the advisor on the laptop AFTER this fix lands + finalize-merge; Cohort B barrier reaches 4/4 only on DQ #266 result:pass. Task 9 is the LAST impl task before Task 10 retro."
---

# [role:impl-task] v1-federation-inbound-a fix-impl-4 — Task-9 e2e.rs fixtures module: mirror canonical sibling Case A (fix 7 compile errors) — see .claude/PRPs/briefs/federation-inbound-a-fix-impl-4.md

> **Provenance:** Task 9 (#321) appended a new test module `mod v1_federation_inbound_a_fixtures` at the end of `crates/server/tests/e2e.rs` (impl `6ae091e90`). The Task-9 brief §2.2 **mandated mirroring the canonical sibling `mod v1_sl_b_fixtures` / most-recent `mod v1_sl_e_fixtures` (Case A error shape) verbatim**. The worker did **not** comply: it (a) imported `InstanceId` from `lemmy_db_schema::newtypes` (does not exist there — the canonical sibling uses `lemmy_db_schema_file::InstanceId`), (b) called `governance_fixtures::start_postgres_with_migrations()` and `governance_fixtures::async_conn(&db_url)` — **neither function exists** in the `governance_fixtures` module (the real API is `bootstrap()` / `start_postgres()` / `db_url()`), and (c) used `.map_err(|e| LemmyErrorType::Unknown(format!("{e}")))?` without importing `LemmyErrorType` — whereas Case A (the canonical sibling shape) uses **bare `?`** because `bootstrap()` returns `LemmyResult`. Result: `cargo-test --workspace --features full --test e2e --no-run` (Task 9 §5.2 cmd 3) fails with **7 compile errors** (1×E0432, 4×E0425, 2×E0433) + 1 unused-import warning, all confined to the new module (e2e.rs:15091-15157). Every error is a **direct mechanical divergence from the canonical sibling** `mod v1_sl_e_fixtures` (lines 14073-15090, same file) which demonstrates the correct Case A shape verbatim. Classification: **§G4 allowlist — compound mechanical, canonical-sibling-mirror** (Task 9 §5.2 **cycle 1**; `error_class_history` was empty; does NOT trigger the ≥3 cycle-count catch-fire). The fix is fully determined: rewrite the **3 broken regions of the new module ONLY** to mirror the canonical sibling's import + container-acquisition + error shape.

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD branch is a Junior worktree branched off `phase-v1-federation-inbound-a` (base tip `b206673e9`). `git merge-base --is-ancestor b206673e9 HEAD` MUST be true. If on `phase-v1-federation-inbound-a` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ **into `.claude/decision-queue.json`** (NOT a repo-root file — see §4 Constraint 3).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — if inside a forbidden window exit non-zero `FORBIDDEN_WINDOW: <window>`, UNLESS the dispatch description carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — window-cargo concern reduced; keep the check.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** run `git submodule update --init 2>&1` from the worktree root. The `crates/email/translations` git submodule is NOT auto-initialized in a fresh worktree; without it `cargo-check` fails pre-existing (ENOENT on `translations/backend/`) — the same false-negative class that hit Cohort-A Task-5 / DQ #236 and fix-impl-1 #315. Infra, NOT part of the fix; initialize it so the §4.2 pre-push cargo-check is meaningful.
- Confirm the target module is present and unmodified at the expected anchor: `grep -n "mod v1_federation_inbound_a_fixtures" crates/server/tests/e2e.rs` MUST return line **15091**; `grep -n "mod v1_sl_e_fixtures" crates/server/tests/e2e.rs` MUST return line **14073** (the canonical sibling — the MIRROR ref). `sed -n '15091,15157p' crates/server/tests/e2e.rs` MUST show the broken module exactly as quoted in §2.2. If the line numbers differ or the module body differs from §2.2's "BEFORE" → STOP, file `kind: "blocker"` DQ **into `.claude/decision-queue.json`** (base mismatch — Task 9 not on this worktree's base, or a concurrent edit moved it).
- **Never Read the whole `crates/server/tests/e2e.rs` file** (it is ~15,157 lines; a full-file Read or multi-hunk rewrite hangs the worker — `feedback_junior_worker_e2e_edit_hang`). Use `sed -n '<lo>,<hi>p'` to inspect ONLY the two regions named: the broken module (15091-15157) and the canonical sibling (14073-14130 is enough for the import + acquisition idiom). The 3 Edits below are small, well-separated anchor-Edits inside the EOF module.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a fix-impl-4 — Task-9 e2e.rs fixtures: mirror canonical sibling Case A`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-a fix-impl-4 — see .claude/PRPs/briefs/federation-inbound-a-fix-impl-4.md
```

## §2 Scope

### 2.0 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table — the contract; do NOT paraphrase)

> | Failure signature | Auto-fix | Source lesson |
> | **4b** `error[E0277]: ?` couldn't convert `LemmyError`/`LemmyResult<T>` to `Box<dyn Error>` AND a v1-SL-* / v1-JM-* sibling fixtures module exists in the same file using `LemmyResult<()>` outer | flip the test fn signature to `LemmyResult<()>` AND flip ALL helper signatures in this module's fixtures mod to `LemmyResult<T>`. No `.map_err` bridges. Mirror the canonical sibling shape verbatim (Case A per lesson). | `feedback_lemmy_error_no_std_error.md` Case A |

> | Failure signature | Auto-fix | Source lesson |
> | `error[E0432]: unresolved import` | add the missing `use` per the suggestion | n/a (mechanical) |

**Application note (this fix-impl):** the symptom here is the Case-A-divergence root (the worker used `.map_err(|e| LemmyErrorType::Unknown(...))` against a non-existent helper API instead of bare `?` against the canonical `bootstrap()`), surfacing as E0425 (invented API) + E0433 (un-imported `LemmyErrorType`) + E0432 (wrong newtype path) rather than a bare E0277 — but the §G4 row-4b prescription is identical and exact: **mirror the canonical sibling `mod v1_sl_e_fixtures` shape verbatim (Case A): `LemmyResult` outer + bare `?` + no `.map_err` bridges**, and fix the import per the E0432 row (`lemmy_db_schema_file::InstanceId`, not `lemmy_db_schema::newtypes::InstanceId`). The canonical sibling is in THIS file at lines 14073-15090 — copy its idiom, do not invent.

### 2.1 The failure being fixed (the contract)

After Task 9 (`6ae091e90`) appended `mod v1_federation_inbound_a_fixtures`, `cargo-test --workspace --features full --test e2e --no-run` (Task 9 §5.2 command 3) fails with 7 errors, all in the new module:

```
error[E0432]: unresolved import `lemmy_db_schema::newtypes::InstanceId`
   --> crates\server\tests\e2e.rs:15096:5   — no `InstanceId` in `newtypes`

error[E0425]: cannot find function `start_postgres_with_migrations` in module `governance_fixtures`
   --> crates\server\tests\e2e.rs:15134:56   (and again at 15148:56)

error[E0425]: cannot find function `async_conn` in module `governance_fixtures`
   --> crates\server\tests\e2e.rs:15138:41   (and again at 15152:41)

error[E0433]: cannot find type `LemmyErrorType` in this scope
   --> crates\server\tests\e2e.rs:15136:20   (and again at 15150:20)

warning: unused import: `QueryDsl`
   --> crates\server\tests\e2e.rs:15094:35
```

**Root cause:** the worker authored against an **imagined** `governance_fixtures` API. The **real** API (verified from `mod governance_fixtures` in the same file, lines 118-890) is:

- `pub async fn start_postgres() -> LemmyResult<(ContainerAsync, u16)>` — there is NO `start_postgres_with_migrations`.
- `pub async fn bootstrap() -> LemmyResult<(ContainerAsync, Data<LemmyContext>, String)>` — spins Postgres, applies the full schema (incl. `instance` + `federation_peer` tables), returns `(_container, context, db_url)`. **This is what the canonical sibling uses.**
- `pub fn db_url(host_port: u16) -> String` — exists, but unnecessary if you use `bootstrap()` (it returns `db_url`).
- There is NO `governance_fixtures::async_conn`. An `AsyncPgConnection` is established directly via `AsyncPgConnection::establish(&db_url).await?` (the canonical sibling does exactly this).
- `InstanceId` is re-exported from `lemmy_db_schema_file`, NOT `lemmy_db_schema::newtypes` (canonical sibling line 14102-14104: `use lemmy_db_schema_file::{InstanceId, PersonId, ...}`).
- Case A error shape: helpers/`bootstrap()`/`establish()` all return `LemmyResult` (or are `?`-compatible); the canonical sibling uses **bare `?`** with **no `.map_err`** and does **not** import `LemmyErrorType`.

### 2.2 The exact fix (the contract — implement THIS via 3 anchor-Edits, do NOT paraphrase, do NOT rewrite the whole file)

The broken module currently reads (`crates/server/tests/e2e.rs` lines **15091-15157**):

```rust
mod v1_federation_inbound_a_fixtures {
  use super::*;
  use diesel_async::{AsyncPgConnection, RunQueryDsl};
  use diesel::{ExpressionMethods, QueryDsl};
  use lemmy_db_schema::{
    newtypes::InstanceId,
    source::governance::federation_peer::{
      federation_inbox_check_peer_trust,
      FederationPeerInsertForm,
    },
  };
  use lemmy_db_schema_file::enums::FederationPeerTrust;
  use lemmy_db_schema_file::schema::{federation_peer, instance};
  use lemmy_utils::error::LemmyResult;

  async fn seed_federation_peer(
    conn: &mut AsyncPgConnection,
    domain: &str,
    trust: FederationPeerTrust,
  ) -> LemmyResult<InstanceId> {
    let instance_id: InstanceId = diesel::insert_into(instance::table)
      .values((
        instance::domain.eq(domain),
        instance::published_at.eq(diesel::dsl::now),
      ))
      .returning(instance::id)
      .get_result(conn)
      .await?;
    let form = FederationPeerInsertForm {
      instance_id,
      trust_level: Some(trust),
      added_by_actor: None,
      notes: None,
    };
    diesel::insert_into(federation_peer::table)
      .values(&form)
      .execute(conn)
      .await?;
    Ok(instance_id)
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn federation_peer_trust_lookup_returns_seeded_state() -> LemmyResult<()> {
    let (_container, host_port) = governance_fixtures::start_postgres_with_migrations()
      .await
      .map_err(|e| LemmyErrorType::Unknown(format!("{e}")))?;
    let db_url = governance_fixtures::db_url(host_port);
    let mut conn = governance_fixtures::async_conn(&db_url).await?;
    let _instance_id =
      seed_federation_peer(&mut conn, "allowlisted.test", FederationPeerTrust::Allowlisted).await?;
    let trust = federation_inbox_check_peer_trust("allowlisted.test", &mut conn).await?;
    assert_eq!(trust, FederationPeerTrust::Allowlisted);
    Ok(())
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn federation_peer_trust_lookup_returns_unknown_for_first_seen() -> LemmyResult<()> {
    let (_container, host_port) = governance_fixtures::start_postgres_with_migrations()
      .await
      .map_err(|e| LemmyErrorType::Unknown(format!("{e}")))?;
    let db_url = governance_fixtures::db_url(host_port);
    let mut conn = governance_fixtures::async_conn(&db_url).await?;
    let trust = federation_inbox_check_peer_trust("unknown-peer.test", &mut conn).await?;
    assert_eq!(trust, FederationPeerTrust::Unknown);
    Ok(())
  }
}
```

Apply **exactly these 3 anchor-Edits** (each keyed on a unique, well-separated string inside this EOF module — NOT a full-file read/rewrite; `feedback_junior_worker_e2e_edit_hang`):

**EDIT 1 — fix the `use` block (fixes E0432 + the unused-`QueryDsl` warning).**

`old_string` (anchor — the 7-line use cluster):
```rust
  use diesel_async::{AsyncPgConnection, RunQueryDsl};
  use diesel::{ExpressionMethods, QueryDsl};
  use lemmy_db_schema::{
    newtypes::InstanceId,
    source::governance::federation_peer::{
      federation_inbox_check_peer_trust,
      FederationPeerInsertForm,
    },
  };
  use lemmy_db_schema_file::enums::FederationPeerTrust;
  use lemmy_db_schema_file::schema::{federation_peer, instance};
  use lemmy_utils::error::LemmyResult;
```

`new_string`:
```rust
  use diesel::ExpressionMethods;
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_db_schema::source::governance::federation_peer::{
    federation_inbox_check_peer_trust,
    FederationPeerInsertForm,
  };
  use lemmy_db_schema_file::enums::FederationPeerTrust;
  use lemmy_db_schema_file::schema::{federation_peer, instance};
  use lemmy_db_schema_file::InstanceId;
  use lemmy_utils::error::LemmyResult;
```

Rationale: `InstanceId` moves to `lemmy_db_schema_file::InstanceId` (matches canonical sibling line 14102-14104; fixes E0432); `QueryDsl` is dropped (unused — fixes the warning; `-D warnings` would fail clippy CMD2 otherwise); `AsyncConnection` is added to the `diesel_async` import because `AsyncPgConnection::establish` requires the `AsyncConnection` trait in scope (canonical sibling line 14079 imports it).

**EDIT 2 — fix test fn 1's container acquisition (fixes E0425 ×2 + E0433 ×1 at 15134/15138/15136).**

`old_string` (anchor — the 4-line broken acquisition + seed in test 1):
```rust
    let (_container, host_port) = governance_fixtures::start_postgres_with_migrations()
      .await
      .map_err(|e| LemmyErrorType::Unknown(format!("{e}")))?;
    let db_url = governance_fixtures::db_url(host_port);
    let mut conn = governance_fixtures::async_conn(&db_url).await?;
    let _instance_id =
      seed_federation_peer(&mut conn, "allowlisted.test", FederationPeerTrust::Allowlisted).await?;
```

`new_string`:
```rust
    let (_container, _context, db_url) = governance_fixtures::bootstrap().await?;
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let _instance_id =
      seed_federation_peer(&mut conn, "allowlisted.test", FederationPeerTrust::Allowlisted).await?;
```

**EDIT 3 — fix test fn 2's container acquisition (fixes E0425 ×2 + E0433 ×1 at 15148/15152/15150).**

`old_string` (anchor — the 4-line broken acquisition in test 2; note the trailing line differs from EDIT 2's so the anchors are unique):
```rust
    let (_container, host_port) = governance_fixtures::start_postgres_with_migrations()
      .await
      .map_err(|e| LemmyErrorType::Unknown(format!("{e}")))?;
    let db_url = governance_fixtures::db_url(host_port);
    let mut conn = governance_fixtures::async_conn(&db_url).await?;
    let trust = federation_inbox_check_peer_trust("unknown-peer.test", &mut conn).await?;
```

`new_string`:
```rust
    let (_container, _context, db_url) = governance_fixtures::bootstrap().await?;
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let trust = federation_inbox_check_peer_trust("unknown-peer.test", &mut conn).await?;
```

This mirrors the canonical sibling `mod v1_sl_e_fixtures` verbatim: `let (_container, _context, db_url) = governance_fixtures::bootstrap().await?;` then `let mut conn = AsyncPgConnection::establish(&db_url).await?;` — bare `?`, no `.map_err`, no `LemmyErrorType` (Case A). `bootstrap()` applies the full schema so the `instance` + `federation_peer` tables exist for `seed_federation_peer`.

**Boundaries:**
- Edit **ONLY** `crates/server/tests/e2e.rs`. Cap: **1 file**, exactly **3 anchor-Edits**, all inside the `mod v1_federation_inbound_a_fixtures` block (lines 15091-15157). `git diff` must show changes confined to that block — net change is the 3 regions above, nothing else.
- Do **NOT** change `seed_federation_peer`'s body, the `FederationPeerInsertForm { ... }` literal, the two `#[tokio::test]` attributes, the `assert_eq!` lines, the `Ok(())` lines, or the module's closing `}`. Only the `use` block (EDIT 1) and the 2 acquisition stanzas (EDIT 2, EDIT 3) change.
- Do **NOT** touch any other test, any other module, any other `use`, or any line outside 15091-15157. Do **NOT** Read or rewrite the whole file. Do **NOT** touch `crates/db_schema/**`, `crates/apub/**`, `inbox.rs`, `schema.rs`, `Cargo.toml`, any migration, any `.claude/**` file (except a forced `kind: "blocker"` DQ per §4, written **into `.claude/decision-queue.json`**).
- Do **NOT** add any `#[allow]`, `#[expect]`, or any attribute. The fix is exactly the 3 anchor-Edits above.

## §3 Required reading

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **the load-bearing lesson (Case A).** The canonical sibling `mod v1_sl_e_fixtures` (this file, 14073-15090) uses `LemmyResult` outer + bare `?` + no `.map_err` bridges + no `LemmyErrorType` import. The broken module used a Case-B-ish `.map_err(|e| LemmyErrorType::Unknown(...))` against a non-existent API. The fix is to mirror the canonical sibling's Case A shape verbatim (§G4 row 4b). **Read the canonical sibling lines 14073-14130 in the file directly (`sed -n '14073,14130p'`) — it is the MIRROR ref; copy its import + `bootstrap()` + `AsyncPgConnection::establish` idiom, do not invent.**
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **why this is 3 anchor-Edits, not a full-file rewrite.** `crates/server/tests/e2e.rs` is ~15,157 lines; a worker that Reads/rewrites the whole file (or does a sweeping multi-hunk Edit) hangs (SIGTERM). The 3 Edits here are **small, well-separated, unique-anchored** insertions inside one EOF module — use `Edit` with the exact `old_string`/`new_string` from §2.2. Inspect only via `sed -n '15091,15157p'` (broken module) + `sed -n '14073,14130p'` (canonical sibling). Do NOT Read the whole file. *(If this lesson file is absent from `.claude/lessons/` — it has historically been a lesson-mirror gap — the binding constraint is reproduced verbatim here and in plan §10.8: anchor-Edit only, never full-file Read/rewrite for e2e.rs.)*
- `.claude/lessons/feedback_newtype_locations_lemmy_db_schema_vs_file.md` — the exact `InstanceId` newtype-location class: it lives in `lemmy_db_schema_file`, not `lemmy_db_schema::newtypes`. EDIT 1 fixes this.
- `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — pre-push local cargo-check discipline (see §4 Constraint 2). This discipline caught fix-impl-1's defective recipe; honour it again.
- `.claude/lessons/feedback_worktree_submodules_not_auto_init.md` — why §0 mandates `git submodule update --init` before in-worker cargo-check (CMD1 false-negative class).
- The triggering entry: `.claude/decision-queue.json` DQ #266 (Task 9 validate-pending-laptop, `from: "impl"`, the one you will NOT mutate — the advisor mutates it after re-validation on the fixed tip).
- The plan: `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` §10.8 — the concrete Task-9 fixtures-module spec the worker should have followed (it names the canonical sibling shape). Read §10.8 to confirm the intended module shape matches the canonical-sibling mirror this fix applies.

## §4 Constraints

1. **One commit.** Subject: `fix(v1-federation-inbound-a): e2e.rs v1_federation_inbound_a_fixtures — mirror canonical sibling Case A (bootstrap + AsyncPgConnection::establish; lemmy_db_schema_file::InstanceId) (fix-impl 4)`. Commit body cites: triggering DQ #266 (Task 9 validate-pending-laptop), the 7 compile errors (1×E0432 + 4×E0425 + 2×E0433 + 1 unused-import warning), that Task 9's worker did not comply with brief §2.2's mandate to mirror the canonical sibling (authored against a non-existent `governance_fixtures` API), and that the canonical MIRROR ref is `mod v1_sl_e_fixtures` in the same file (14073-15090).
2. **Pre-push cargo-check discipline** (per `feedback_fix_impl_pre_push_cargo_check.md`): AFTER `git submodule update --init` (§0) and BEFORE pushing the worker branch, run `cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"` locally in-worker. Exit 0 expected. Non-zero → STOP, do NOT push, file `kind: "blocker"` DQ **into `.claude/decision-queue.json`** citing the cargo-check failure verbatim (full last ~80 lines). Do NOT `#[allow]`/`#[expect]`-spam to make it pass. (clippy + `--test e2e --no-run` are re-run by the advisor on the laptop post-finalize-merge per Shape-G-suspended §5.2 — you only run cargo-check pre-push. Note: `cargo-check` does NOT compile the e2e test binary, so it will NOT exercise the e2e.rs module change; it confirms the workspace still builds. The advisor's laptop §5.2 cmd 3 `cargo-test --test e2e --no-run` is the real proof — but pre-push cargo-check still catches any accidental non-test-target breakage.)
3. **DQ raise — write into the canonical `.claude/decision-queue.json`, NOT a repo-root file.** Per `.claude/refs/dq-recipes.md` Recipe + `.claude/rules/decision-queue.md` "Mid-task visibility": this fix-impl does NOT need a new validate-pending DQ (Task 9's validate is already DQ #266, which the advisor re-runs on the fixed tip). You only file a DQ if §0 or §4.2 forces a `kind: "blocker"` — and that blocker MUST be a properly-formed entry **appended to `.claude/decision-queue.json`'s `pending[]`** (computed `id = max(all ids across pending+resolved+archives)+1`; the advisor verified the cross-lane max is **266** so the next id is **267** — but **recompute, do not hardcode**; **pin `ensure_ascii=False`** — the impl-task DQ-write ascii-escape breach recurred 5x this phase, write canonical UTF-8 not `\uXXXX`-escaped), committed + pushed on the worker branch. Do **NOT** mutate or touch DQ #266 (the advisor owns its resolution). Do **NOT** write a repo-root `.md`/`.json` debris file (Task 8's `TASK8_ESCALATION.md` root-file miss must not recur).
4. **MIRROR-ref discipline:** the fix form is dictated by the **canonical sibling `mod v1_sl_e_fixtures`** (this file, 14073-15090) + §G4 row 4b + `feedback_lemmy_error_no_std_error.md` Case A. Copy its import shape (`lemmy_db_schema_file::InstanceId`, `diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl}`), its container-acquisition idiom (`governance_fixtures::bootstrap().await?` then `AsyncPgConnection::establish(&db_url).await?`), and its bare-`?` error shape. Do NOT invent helper names, do NOT use `.map_err`, do NOT import `LemmyErrorType`, do NOT change anything outside the 3 anchor-Edits.
5. **Attribution:** if a blocker DQ is forced, `from: "impl"`, `answered_by: null`. NEVER write `answered_by: "advisor"` / `"user"` / `kind: "clarify"` (per `.claude/rules/decision-queue.md` hard refusals).
6. **Serial discipline:** this fix-impl is the in-flight Task 9 §5.2 e2e-compile recovery under Cohort B serial cap=1. Task 9 is the LAST impl task. Do not dispatch or reference any other task. One worker, one fix, terminal.

## §5 Acceptance

- `crates/server/tests/e2e.rs` `mod v1_federation_inbound_a_fixtures` (was lines 15091-15157): the `use` block matches EDIT 1's `new_string`; test fn 1's acquisition matches EDIT 2's `new_string`; test fn 2's acquisition matches EDIT 3's `new_string`. `seed_federation_peer`'s body, both `#[tokio::test]` attributes, the `assert_eq!`/`Ok(())` lines, and the module's closing `}` are byte-identical.
- `git diff --stat` = 1 file (`crates/server/tests/e2e.rs`); `git diff` changes confined entirely to the `mod v1_federation_inbound_a_fixtures` block; no other module/test/import/line touched.
- No `#[allow]`/`#[expect]`/attribute added. No invented helper names. No `.map_err`. No `LemmyErrorType` import.
- `git submodule update --init` ran in §0 (CMD1 cargo-check meaningful).
- Local `cargo-check.bat --workspace --features full` exit 0 (run pre-push per §4.2, after submodule init).
- No new DQ entry needed in the happy path (Task 9 validate is DQ #266, advisor-owned). If §0/§4.2 forced a `kind: "blocker"`, it is a properly-formed entry in `.claude/decision-queue.json` `pending[]` (`ensure_ascii=False` canonical, recomputed id) — **NOT a repo-root file**.
- DQ #266 NOT touched. No repo-root `.md`/`.json` debris files created.
