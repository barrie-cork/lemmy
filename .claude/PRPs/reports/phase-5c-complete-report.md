# Phase 5c completion report

**Branch.** `phase-5c` cut from `governance-v0` @ `750e1fc4d` (post upstream-rebase 2026-04-18)
**Scope.** Tasks 61–69 + 69a + task 0 (pre-phase audit) + task 70 (phase-close)
**Plan.** `.claude/PRPs/plans/phase-5c-remaining-endpoints-and-observability.plan.md`

---

## 1. Tasks shipped

| Task | Commit | Summary |
|---|---|---|
| 0 (pre-phase) | `c2b69e19e`, `fcaf36013`, `a2709a99e`, `d83d9e4dd`, `f10bcad3a`, `adca419de`, `8012f67a5`, `90d65ae1f`, `0757a51de` | Risk-reduction Moves 1–8 + decision-queue sweep + cargo-test.bat exit-code propagation fix (Issue #8) + harness-audit rule §1 probe 4 |
| 61 | `3870ffecd` | `get_my_reputation` handler — GET `/api/v4/governance/reputation/me` |
| 62 | `82bb67253` | `admin_reputation_stats` handler — POST `/api/v4/governance/admin/reputation-stats` |
| 63 | `cb15482b7` | Threshold-crossing log wire-up + 63d staleness alert |
| 63c lint | `504b89d9b` | Authorise task 62 + 63d `can_sponsor` observability sites |
| 64 | `c034fd1e4` | `accept_jury_assignment` handler + `admin_assign_jury` flip Accepted → Selected |
| 65 | `48e657778` | `decline_jury_assignment` handler + replacement pick |
| 66 | `0078cda3f` | `request_appeal` handler (api_crud) |
| 67 | `303681e3c` | `list_cases` handler + `CasesFilter` view-fn |
| 68 | `93dcffff8` | Route registration + compound `all_mvp_endpoints_return_non_404` e2e (non-404 sweep + 4 authed happy-paths) |
| 69 | `ce4b8a6ae` | `ineligible_user_cannot_be_picked_for_jury` e2e (3 branches: basic / concurrent-cap / config flip) + Watch 10 PII sweep |
| 69a | `cde8f47c1` | V2 hooks: NOTIFY trigger + SUBSCRIPTIONS.md + 2 e2e tests + plan §6.1 SSE-over-WS annotation |
| 70 | (this commit) | Completion report + admin-config-write.sh + PR |

**Endpoint count:** 11 MVP endpoints live (§6.1 of [05](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) satisfied) plus 3 admin backstops.

---

## 2. Definition-of-done status

| Check | Result | Log |
|---|---|---|
| `cargo check --workspace --features full` | ✅ exit 0 | `.claude/build-task68-check.log` |
| `cargo clippy --workspace --no-deps --features full -- -D warnings` | ✅ exit 0 | `.claude/build-task69a-clippy.log` |
| `cargo test --test e2e --workspace --features full --no-run` | ✅ exit 0 | `.claude/build-task69a-test-compile.log` |
| `report_to_modlog_golden_path` regression (post-rebase) | ✅ passed | `.claude/build-rebase-regression.log` |
| `governance_events_notify_fires` | ✅ passed | `.claude/build-task69a-run-notify2.log` |
| `underscore_prefix_usernames_still_register` | ✅ passed | `.claude/build-task69a-run-username.log` |
| `all_mvp_endpoints_return_non_404` | ⏳ pending CI | `.claude/build-task70-run-68.log` |
| `ineligible_user_cannot_be_picked_for_jury` | ⏳ pending CI | |

The full Level 3 e2e run is deferred to CI per usual cadence — local DoD smoke covers the four NEW tests shipped in 68/69/69a and the golden-path regression. The two ⏳ gates above cover the Phase 5c test additions (tasks 68 + 69) and must turn green in CI before PR #10 merges — neither blocks local DoD, but both are pre-merge required.

---

## 3. Carry-forwards (do NOT address in 5c)

These are noted for Phase 6 / v1 or separate issue tickets:

1. **Original-reporter appeals** — `request_appeal` handler rejects anyone except the sanction target. IMPLEMENTATION-PLAN-v0.md line 381 explicitly defers original-reporter appeal paths to v1. Doc comment at `crates/api/api_crud/src/governance/request_appeal.rs` cites this.
2. **`assignee` filter on `list_cases`** — DTO `ListGovernanceCases` intentionally omits an `assignee` field (v1 scope per plan §11.7 GOTCHA). Added to v1 backlog.
3. **Per-community permission filter on `list_cases`** — v0 exposes every case to any authenticated caller. v1 adds community-membership gating. Tracked in handler doc comment.
4. **Re-jury on `Appealed` → ?** — v0 has no re-jury; the case sits in `Appealed` until admin closes it. v1 adds a re-jury path per IMPLEMENTATION-PLAN-v0.md line 381.
5. **Appeal window formalisation** — v0 defines "appeal window is open" as `case.closed_at IS NULL`. v1 should codify a bounded window (e.g. 7 days post-decision) and surface it in DTOs.
6. **OQ-018 admin HTTP endpoint** — `admin-config-write.sh` ships as sibling operator tooling per DQ #13; the proper HTTP endpoint lands in v1. Both DQ #13 context and OQ-018 tracking remain open.
7. **Tokio-postgres poll-based notification bridge** — if upstream Lemmy ever needs a similar NOTIFY subscriber, extract the `poll_fn + Pin + poll_message + mpsc` bridge helper from `e2e.rs::governance_events_notify_fires` into a shared utility.

### GitHub issue tracking

| # | Title | Issue URL |
|---|---|---|
| 1 | v1: original-reporter appeals on request_appeal | https://github.com/barrie-cork/lemmy/issues/11 |
| 2 | v1: add assignee filter to list_cases DTO | https://github.com/barrie-cork/lemmy/issues/12 |
| 3 | v1: per-community permission filter on list_cases | https://github.com/barrie-cork/lemmy/issues/13 |
| 4 | v1: re-jury path for Appealed cases | https://github.com/barrie-cork/lemmy/issues/14 |
| 5 | v1: formalise appeal window with bounded duration | https://github.com/barrie-cork/lemmy/issues/15 |
| 6 | v1: OQ-018 admin config-write HTTP endpoint | https://github.com/barrie-cork/lemmy/issues/16 |
| 7 | v1: extract tokio-postgres NOTIFY bridge helper | https://github.com/barrie-cork/lemmy/issues/17 |

---

## 4. Deviations from plan

- **Stash recovery instead of ralph loop** — after rebasing phase-5c onto the upstream-rebased `governance-v0` tip, I resumed from a stash rather than re-running the full ralph loop. Tasks 61–67 landed pre-pause on earlier iterations; tasks 68, 69, 69a, 70 landed via the recovery.
- **NOTIFY payload field name** — plan §11.10 and fragment referenced `published_at`; the governance_log column is actually `created_at`. Migration + test + SUBSCRIPTIONS.md all use `created_at`. One fix iteration (the first test run surfaced the trigger error at `governance_log.rs:103` on insert).
- **`--features full` + `-p lemmy_server` incompatibility** — advisor note was correct: `lemmy_server` does NOT declare the `full` feature. All validation commands in this report used `--workspace --features full`, which exercises every crate's `full` gate cleanly.
- **e2e test import rewrites** — stash-shipped test code referenced `lemmy_db_schema_file::LocalUserId`, `::CommunityId`, `::ModerationCaseId` but those newtypes live in `lemmy_db_schema::newtypes`. Fixed inline. `lemmy_db_schema_file` does re-export `PersonId` and `InstanceId`.
- **Missing `use lemmy_diesel_utils::traits::Crud;`** — stash-shipped test blocks used `Person::create` / `Community::create` / `LocalUser::create` without importing the `Crud` trait. Added to all three new test modules (68, 69, 69a.3, 69a.4).

---

## 5. ADR / invariant audit

- **ADR-012 (Extism)** — no change; PM plugin hooks remain stable per `.claude/rules/pm-plugin-hooks-stable.md`.
- **ADR-013 (EmergencyRemove)** — every new handler matching `CaseStatus` handles `EmergencyRemove` exhaustively (no `_ =>` arms). Verified in `request_appeal.rs` + `list_cases` filter.
- **ADR-015 (pseudonymisation)** — every `governance_log::append` call uses `actor_pseudonym`, and task 69's Watch 10 PII sweep asserts no raw person_id / username / email leaks into any payload.
- **Hash-chain integrity** — NOTIFY trigger fires AFTER INSERT, same lifecycle point as the hash-chain trigger. No interaction.

---

## 6. Next up — Phase 6

Phase 5 is complete with this merge. Phase 6 (federation outbound + advisory inbound) is the only remaining phase before v0 ship. See IMPLEMENTATION-PLAN-v0.md §3 Phase 6.
