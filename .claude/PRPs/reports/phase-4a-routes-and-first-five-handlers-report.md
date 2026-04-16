# Implementation Report — Phase 4a

**Plan**: `.claude/PRPs/plans/completed/phase-4a-routes-and-first-five-handlers.plan.md`
**Source**: [IMPLEMENTATION-PLAN-v0.md](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) §3 Phase 4 (tasks 38–43 only; 44–49 deferred to Phase 4b)
**Branch**: `governance-v0`
**Date**: 2026-04-16
**Status**: COMPLETE

---

## Summary

Phase 4a shipped the HTTP layer for the Brehon governance API. Five
handlers are wired up at `/api/v4/governance/` and three cross-cutting
helpers (`governance_log::append`, `actor_pseudonym_helper::get_or_create`,
`redaction::scrub` + `scrub_json`) are in place so every governance
write path satisfies ADR-013 (EmergencyRemove exhaustive) and ADR-015
(GDPR pseudonymisation).

| Route | Handler | Crate |
|---|---|---|
| `POST /api/v4/governance/report` | `create_report` | `lemmy_api_crud` |
| `GET /api/v4/governance/case` | `get_case` | `lemmy_api` |
| `GET /api/v4/governance/modlog` | `list_modlog` | `lemmy_api` |
| `GET /api/v4/governance/jury/me` | `list_my_jury_queue` | `lemmy_api` |
| `POST /api/v4/governance/jury/vote` | `submit_jury_vote` | `lemmy_api` |

---

## Assessment vs Reality

| Metric | Predicted | Actual | Reasoning |
|---|---|---|---|
| Complexity | HIGH | HIGH | `submit_jury_vote` landed in one commit per plan but required careful attention to the `scope_boxed` transaction shape and the reputation-event fan-out. Everything else was straightforward. |
| Task count | 10 | 11 commits (0 through 10) | One task 10 mini-commit fixed a redaction-regex bug surfaced by the unit tests the same task added. Rolled into Task 10 as intended by the plan's "full workspace validation" step. |
| Time | not estimated | ~2 hours wall-clock | Faster than expected. The dominant cost was two iterations on the redaction regex (mention-vs-email ordering) and one iteration on the `Diesel Nullable<Bool>` boxed-expression return type in `create_report`. |

### Deviations from plan

1. **`api_common` wiring added to both `lemmy_api` and `lemmy_api_crud`.**
   The plan assumed Phase 3 DTOs in `api_common/src/governance.rs` were
   already reachable; they were not. Phase 3 shipped the DTO module
   without wiring the crate. Phase 4a added `lemmy_api_common` to the
   workspace `[dependencies]` table and to both handler crates. This is
   a one-line correction, not a substantive deviation — the plan's Task
   1 did specify "add governance view crate deps," and `api_common` is
   the natural extension.

2. **`create_report` added `diesel` and `lemmy_api` as direct deps of
   `lemmy_api_crud`.** Previously `api_crud` only had `diesel-async`;
   the direct `diesel::` uses for `BoxableExpression` type annotations
   forced the addition. `lemmy_api` was added so `create_report` can
   import `actor_pseudonym_helper` and `governance_log` without
   duplicating those helpers. No cycle — `lemmy_api` only has
   `lemmy_api_crud` under `[dev-dependencies]`, not a build-time dep.

3. **Decision-queue entry #2 was factually wrong** and got corrected in
   Task 0's audit commit (`f91d62609`). The original claimed
   `.boxed()` from `futures::FutureExt`; the actual workspace pattern
   is `.scope_boxed()` from `diesel_async::scoped_futures::ScopedFutureExt`
   (verified at `community/ban.rs:106`).

4. **No planned tests deferred — the plan explicitly deferred tests to
   Phase 4b.** Phase 4a added 5 redaction unit tests opportunistically
   because regex ordering bugs are easy to miss at compile time. All
   pass.

---

## Tasks Completed

| # | Task | File | Status |
|---|---|---|---|
| 0 | Pre-phase audit + baseline | (validation only) | ✅ |
| 1 | Cargo.toml deps for api + api_crud | `crates/api/api/Cargo.toml`, `crates/api/api_crud/Cargo.toml` | ✅ |
| 2 | Cross-cutting helpers (3 modules) | `crates/api/api/src/governance/{governance_log,actor_pseudonym_helper,redaction}.rs` | ✅ |
| 3 | Module wiring (mod.rs + lib.rs) | `crates/api/api/src/{lib,governance/mod}.rs`, `crates/api/api_crud/src/{lib,governance/mod}.rs` | ✅ |
| 4 | `create_report` handler | `crates/api/api_crud/src/governance/create_report.rs` | ✅ |
| 5 | `get_case` handler | `crates/api/api/src/governance/get_case.rs` | ✅ |
| 6 | `list_my_jury_queue` handler | `crates/api/api/src/governance/list_my_jury_queue.rs` | ✅ |
| 7 | `submit_jury_vote` handler | `crates/api/api/src/governance/submit_jury_vote.rs` | ✅ |
| 8 | `list_modlog` handler | `crates/api/api/src/governance/list_modlog.rs` | ✅ |
| 9 | Route registration | `crates/api/routes/src/lib.rs` | ✅ |
| 10 | Workspace validation + test fix | `crates/api/api/src/governance/redaction.rs` | ✅ |

---

## Validation Results

| Check | Result | Details |
|---|---|---|
| `cargo check --workspace` | ✅ | Exit 0, no governance-related warnings |
| `cargo clippy --workspace --no-deps` | ✅ | Exit 0. Two pre-existing upstream `unfulfilled_lint_expectations` warnings in `lemmy_diesel_utils` and `lemmy_db_views_vote`, unrelated. |
| Redaction unit tests (5) | ✅ | All pass: `scrub_strips_mentions`, `scrub_strips_email`, `scrub_strips_profile_urls`, `scrub_json_walks_nested_structure`, `scrub_preserves_keys_and_non_string_scalars` |
| Migration round-trip | ⏭️ N/A | Phase 4a changed no migrations. |
| e2e integration test | ⏭️ Phase 4b | The golden-path test is task 48 per the plan's §Testing Strategy. Phase 4a's validation is compile-only. |
| Cross-cutting verification — no person_id in governance_log | ✅ | `grep "person_id.*governance_log"` returns zero hits. All log writes use the `actor_pseudonym` helper. |
| Cross-cutting verification — `CaseStatus::EmergencyRemove` handled | ✅ | `get_case.rs:53` explicitly lists it; all other `CaseStatus::` uses are specific variant values (not matches). |

---

## Files Changed

| File | Action | Lines |
|---|---|---|
| `Cargo.toml` (root) | UPDATE | +1 (workspace dep `lemmy_api_common`) |
| `Cargo.lock` | UPDATE | (auto) |
| `crates/api/api/Cargo.toml` | UPDATE | +5 (3 view crates + uuid + api_common) |
| `crates/api/api_crud/Cargo.toml` | UPDATE | +4 (governance_case + api_common + diesel + lemmy_api) |
| `crates/api/api/src/lib.rs` | UPDATE | +1 (`pub mod governance;`) |
| `crates/api/api_crud/src/lib.rs` | UPDATE | +1 (`pub mod governance;`) |
| `crates/api/api_common/src/governance.rs` | UPDATE | +12 (add `SubmitJuryVoteResponse`) |
| `crates/api/api/src/governance/mod.rs` | CREATE | +13 |
| `crates/api/api/src/governance/actor_pseudonym_helper.rs` | CREATE | +58 |
| `crates/api/api/src/governance/governance_log.rs` | CREATE | +65 |
| `crates/api/api/src/governance/redaction.rs` | CREATE | +138 (incl. 5 tests) |
| `crates/api/api/src/governance/get_case.rs` | CREATE | +57 |
| `crates/api/api/src/governance/list_my_jury_queue.rs` | CREATE | +22 |
| `crates/api/api/src/governance/list_modlog.rs` | CREATE | +49 |
| `crates/api/api/src/governance/submit_jury_vote.rs` | CREATE | +370 |
| `crates/api/api_crud/src/governance/mod.rs` | CREATE | +9 |
| `crates/api/api_crud/src/governance/create_report.rs` | CREATE | +293 |
| `crates/api/routes/src/lib.rs` | UPDATE | +23 (imports + scope block) |

---

## Cross-Cutting Impact

- [x] Hash chain appends wired up for new writes — every governance write path (`create_report`, `submit_jury_vote`) calls `governance_log::append` for each state-changing event, scrubbing the JSON payload through `scrub_json` on the way in. Hash chain populated by the Phase 1 Postgres trigger.
- [x] `actor_pseudonym_helper::get_or_create` used on every log-emitting handler. No `person_id` or `local_user_id` ever reaches `governance_log.actor_pseudonym`.
- [x] `redaction::scrub` called on `public_case_log.summary` (always) and `public_case_log.rationale_redacted` (when rationales are concatenated from majority voters in `submit_jury_vote`).
- [x] `CaseStatus::EmergencyRemove` exhaustively matched in `get_case::is_public_status`. All other `CaseStatus::` call sites use specific variant values, not matches, so no `_ =>` catchall exists anywhere.
- [x] AGPL notice unchanged (no release artefact produced).

### v0 Design Decisions Deferred to Phase 4b

- **ed25519 signing of `governance_log` rows** — the `signature` column
  is left NULL. The hash chain (trigger-side) ships now; signing lands
  in Phase 4b when the golden-path test can exercise write + sign +
  verify end-to-end. Documented inline in `governance_log.rs` and in
  the plan's §Design Decision: Signing Deferred to Phase 4b.

---

## Issues Encountered

1. **`api_common` not wired to any consumer crate pre-Phase 4a.** Spent
   ~5 min tracing DTO paths. Fix: add `lemmy_api_common` workspace dep
   + `api` + `api_crud` dep edges. Memoed as part of Task 4's commit
   message.

2. **Advisor decision-queue entry #2 cited the wrong transaction helper.**
   Caught during Task 0 audit by grepping for `run_transaction` use
   sites. Corrected the queue entry and annotated with the verification
   source (`community/ban.rs:106`).

3. **`cargo check -p lemmy_api_crud` false-reds on unrelated OAuth
   code** in `user/create.rs` when run without `--features full` or
   `--workspace`. Pre-existing upstream feature-resolver quirk with
   reqwest's `multipart` activation. Documented in memory file
   `feedback_api_crud_oauth_feature_quirk.md` for future phases —
   always prefer `cargo check --workspace` for `api_crud` validation.

4. **Redaction regex ordering bug** — `foo.bar@example.com` originally
   had `@example` eaten as a fediverse mention. Fixed by anchoring the
   mention regex to a non-identifier boundary with a capture group,
   then running mentions before emails in the pipeline. 5 unit tests
   protect this going forward.

5. **`Diesel Nullable<Bool>` vs `Bool` on the `match_target_filter`
   boxed expression** — the target id columns on `moderation_case` are
   all `Nullable<Int4>`, so `.eq()` returns `Nullable<Bool>`. Fixed by
   changing the return SqlType to `Nullable<Bool>`; the `WHERE` clause
   accepts nullable booleans (NULL rows evaluate to UNKNOWN, which is
   falsy — correct semantics for this query).

---

## Tests Written

| Test | Validates |
|---|---|
| `redaction::scrub_strips_mentions` | `@alice` and `@bob@remote.example` both become `[redacted]` in a mixed string |
| `redaction::scrub_strips_email` | `foo.bar@example.com` is redacted without eating surrounding non-identifier chars |
| `redaction::scrub_strips_profile_urls` | `https://example/u/alice` and `https://example/user/bob` both redacted |
| `redaction::scrub_json_walks_nested_structure` | Recursive scrub works on arrays, objects, and nested objects. Object keys preserved. |
| `redaction::scrub_preserves_keys_and_non_string_scalars` | Numbers, bools, nulls unchanged; object keys with `@` preserved |

Integration tests (golden-path e2e) are Phase 4b per the plan.

---

## Commits (chronological)

| SHA | Task | Message |
|---|---|---|
| `f91d62609` | 0 | `chore: task 0 — Phase 4a baseline + plan landed` |
| `c62f2dc4c` | 1 | `feat(api): task 1 — add governance view crate deps to api + api_crud` |
| `204fa37cb` | 2+3 | `feat(api): tasks 2+3 — cross-cutting governance helpers` |
| `6a574a06e` | 4 | `feat(api_crud): task 4 — create_report handler (POST /governance/report)` |
| `329f66985` | 5 | `feat(api): task 5 — get_case handler (GET /governance/case)` |
| `716fbe507` | 6 | `feat(api): task 6 — list_my_jury_queue handler (GET /governance/jury/me)` |
| `a066b1a6b` | 8 | `feat(api): task 8 — list_modlog handler (GET /governance/modlog)` |
| `9267a6dee` | 7 | `feat(api): task 7 — submit_jury_vote handler (POST /governance/jury/vote)` |
| `5d08ad715` | 9 | `feat(api_routes): task 9 — register /governance route scope` |
| `ac0f7d2ee` | 10 | `fix(api): task 10 — redaction ordering + mention boundary` |

Ordering note: tasks 7 and 8 were landed in the reverse of their task
numbers (8 first, then 7) because `list_modlog` was significantly
simpler and landing it first let me confirm the module-registration
pattern was solid before the 370-line `submit_jury_vote`. Both tasks'
validation ran against the same workspace check; no observable
regression.

---

## Next Steps

- [ ] Review the Phase 4a diff. All 10 commits are linear on
  `governance-v0` from `61877804d` to `ac0f7d2ee`.
- [ ] Create a PR against `governance-v0`: run `/prp-pr` or
  `gh pr create --repo barrie-cork/lemmy --base governance-v0`. The
  commit stack is narrow enough to squash-merge if the reviewer
  prefers.
- [ ] **Phase 4b** (tasks 44–49) is the next implementation phase:
  - Task 44 — `POST /governance/admin/assign-jury`
  - Task 45 — `POST /governance/admin/close-case`
  - Task 46 — `EmergencyRemove` helper function
  - Task 47 — Server wiring (`crates/server/src/governance.rs`)
  - Task 48 — Golden-path e2e integration test (the validation that
    proves these five handlers actually work against a real Postgres)
  - Task 49 — Threshold formula placeholder
- [ ] Update the homeserver-side `IMPLEMENTATION-PLAN-v0.md` to mark
  Phase 4 tasks 38–43 as done.
