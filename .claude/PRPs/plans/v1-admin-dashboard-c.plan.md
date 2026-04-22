# Plan: v1-AD-c — Rule-set version CRUD + case-open snapshot wiring + two carry-forward fixes

## Table of contents

| § | Heading | Line |
|---|---|---|
| 1 | Summary | 29 |
| 2 | Source | 41 |
| 3 | Problem statement | 61 |
| 4 | Solution statement | 93 |
| 5 | Metadata | 145 |
| 6 | Sub-phase position (v1-AD-a → b → c) | 163 |
| 7 | Open questions already resolved / reserved | 183 |
| 8 | Flow design | 204 |
| 9 | Mandatory reading | 290 |
| 10 | Patterns to mirror | 335 |
| 11 | Files to change | 560 |
| 12 | NOT building in v1-AD-c | 595 |
| 13 | Step-by-step tasks | 620 |
| 14 | Testing strategy | 940 |
| 15 | Validation commands (DoD) | 980 |
| 16 | Acceptance criteria | 1040 |
| 17 | Completion checklist | 1070 |
| 18 | Risks and mitigations | 1095 |
| 19 | Notes | 1120 |

---

## 1. Summary

v1-AD-c is the **rule-set-CRUD + case-open-snapshot sub-phase** of the v1 admin-dashboard keystone. It ships the two `/admin/rule-sets` routes that create and list append-only community rule-set versions, populates `moderation_case.applied_config_snapshot` + `rule_set_version_id` at case-open (both columns landed in v1-AD-a and have been sitting unwritten), and closes the two carry-forward hygiene issues from PR #76 (#77 provenance double-fetch; #78 negative-community-id parser). Four handler additions, one refactor to `process_set_config` + `project_to_audit_entry`, one new `ENTRY_KIND_RULE_SET_VERSION_CREATED` const, two new DTO families, one insert-form edit in `create_report.rs`, zero new migrations. All work mirrors the v1-AD-b shape (capability gate → pre-tx read → single `run_transaction` closure that does governance-table + governance-log writes together).

Scope is **deliberately bounded**: v1-AD-c writes the `applied_config_snapshot` JSONB at case-open but does NOT wire any READ site to consume it — that stays with jury-mechanics-v1. v1-AD-d (dashboard aggregate + SSE) and v1-AD-e (askama pages) remain descoped.

---

## 2. Source

- [../prds/v1-admin-dashboard.prd.md](../prds/v1-admin-dashboard.prd.md) §3.4 (`requires_re_jury` grandfathering + snapshot semantics), §3.6 (rule-set versioning OQ-002 resolution), §4.1 (endpoint table rows for `/admin/rule-sets` GET/POST), §4.7 (routes wiring), §5.2 (`rule_set.*` defaults), §7 (capability checks, §7.5 rule-set version-change delay), §8.2 (migrations — `add_rule_set_versions` already shipped in v1-AD-a).
- [../plans/completed/v1-admin-dashboard-a.plan.md](completed/v1-admin-dashboard-a.plan.md) — substrate (tables + Diesel models + pin columns).
- [./v1-admin-dashboard-b.plan.md](v1-admin-dashboard-b.plan.md) §4.1 (load-bearing decisions carried over), §6 (AD-c scope inventory), §13 task 5 (shape mirror for `admin_get_config_audit`), §18 (risks carry-over).
- [../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) — ADR-008 (signed log: every governance write logs), ADR-010 (staged releases, no retroactive invalidation → snapshot pin), ADR-013 (EmergencyRemove exhaustive match), ADR-015 (pseudonyms), OQ-002 (rule-set versioning — **this plan ships it**), OQ-V1-AD-03 (pre-tx read-only query; snapshot follows same pattern).
- [../../../docs/brehon-law-inspired-network/04-data-model-and-api.md](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) §3 (`moderation_case` columns including `applied_config_snapshot`, `rule_set_version_id`), §7 (route table).
- v1-AD-b retro / drift notes: `.claude/PRPs/reports/v1-AD-b-plan-drift-notes.md` (`ConfigScope::Instance` vs `::Community` vs `::Both` gotcha — cross-check `CONFIG_KEY_METADATA` before writing tests).
- GitHub Issue [#77](https://github.com/barrie-cork/lemmy/issues/77) — provenance double-fetch in `admin_set_config` audit path (carry-forward from PR #76).
- GitHub Issue [#78](https://github.com/barrie-cork/lemmy/issues/78) — reject negative `community_id` in `Scope::parse_wire` (carry-forward from PR #76).
- `.claude/rules/governance-log-entry-kind-registry.md` — **MANDATORY UPDATE**: add `ENTRY_KIND_RULE_SET_VERSION_CREATED` row to the `v1-AD-c` section (this plan adds the section).
- `.claude/rules/phase-branch.md` — working branch is already `phase-v1-AD-c` per task 0.
- `.claude/rules/pre-phase-harness-audit.md` — task 0 runs this before any code change.

---

## 3. Problem statement

Four things must ship before `v1-AD-d` (dashboard aggregate) or jury-mechanics-v1 can build on the v1 substrate:

1. **`/admin/rule-sets` CRUD.** v1-AD-a laid down the `rule_set_version` table, the Diesel model (`RuleSetVersion` + `RuleSetVersionInsertForm`), and the `moderation_case.rule_set_version_id` pin column. None of it is reachable from the wire today — no HTTP handler, no route registration, no audit entry for rule-set creation. PRD §3.6 and §4.1 define the behaviour: `POST /admin/rule-sets` creates an immutable rule-set row AND flips the `rule_set.active_version_id` config pointer atomically; `GET /admin/rule-sets?community_id=<id>` returns all versions for a community. Without these routes, the only way to populate `rule_set_version` rows is raw SQL, and `rule_set.active_version_id` remains `None` forever because v1-AD-a deliberately left it un-seeded.

2. **Case-open snapshot pinning.** `moderation_case.applied_config_snapshot` (JSONB) and `moderation_case.rule_set_version_id` (nullable FK) were added in v1-AD-a with explicit doc-comments naming v1-AD-c as the phase that writes them. Today `create_report.rs:178-192` constructs `ModerationCaseInsertForm` with `..Default::default()` — both columns stay `None` on every case. This breaks ADR-010's "no retroactive invalidation of in-flight juries" invariant the moment the first `requires_re_jury` key (e.g. `jury.panel_size`) is edited via `admin_set_config`: in-flight cases open before the edit would have nothing to grandfather against.

3. **Issue #77 — provenance double-fetch in the audit path.** PR #76 CodeRabbit finding #4. Today's write path reads `(current_value, current_from)` pre-tx at `admin_config.rs:477`, threads the tuple into the HTTP `preview` response (line 484-494), but the success log payload at line 585-591 omits `previous_value` entirely. The audit-entry projection at line 1284-1330 leaves `previous_value: None` with a comment saying "a future revision may join to the prior row in the same `(scope, key)` bucket to hydrate it" (line 1282-1283). That future revision is v1-AD-c. Adding `previous_value` + `previous_from` to the log payload AND threading the pre-tx tuple through (rather than re-fetching inside the tx closure or inside the audit projection) is the fix Issue #77 specifies. The refactor extracts a `build_admin_config_changed_payload(...)` helper that takes the pre-tx tuple as input; the audit projection stops pretending `previous_value` is hydration-time work and instead reads it directly from the payload.

4. **Issue #78 — negative `community_id` in `Scope::parse_wire`.** PR #76 CodeRabbit finding #5. `config.rs:143-150` currently accepts any `i32`, so `"community:-1"` parses to `Scope::Community(CommunityId(-1))`. The downstream SELECT fails with `CommunityNotFound` — defensible at runtime, but the wire-parser is the correct layer for shape validation and the error message is misleading. Fix: explicit positive-ID check in `parse_wire`, typed rejection distinct from malformed-format rejection. Since v1-AD-c's two new handlers also parse `Scope` from the wire (via `community_id` query param on GET, and an implicit `Scope::Community(community_id)` construction on POST), consolidating the check once lives better here than in each handler.

Without #1 + #2, jury-mechanics-v1 cannot read the snapshot to grandfather juries, and rule-set auditing has no data. Without #3, operators cannot see "what changed from what" in config audit and every audit GET does N+1 queries to re-hydrate previous values one at a time. Without #4, v1-AD-c's rule-set GET endpoint will accept `?community_id=-1` as a valid scope query and 500 on the downstream DB lookup.

---

## 4. Solution statement

Ship **two new handlers**, **one refactor of the v1-AD-b write path** (Issue #77), **one typed enum in `Scope::parse_wire`** (Issue #78), and **one insert-form edit** (case-open snapshot pin):

- **`POST /api/v4/governance/admin/rule-sets`** (`admin_create_rule_set`) — reads `AdminCreateRuleSet { community_id, rule_text, parent_id: Option<i32> }` on the wire, checks `CommunityModeratorView` OR instance-admin capability, computes `version = prev.version + 1` (or 1 if no parent), computes `text_sha256` via the same `sha2::Sha256` instance used by `governance_log::append`, then opens a single `run_transaction` closure doing **three writes atomically**: (a) INSERT `rule_set_version` row, (b) INSERT `governance_config` row for `rule_set.active_version_id` at `Scope::Community(community_id)` with value = new row id, (c) `governance_log::append` emitting `ENTRY_KIND_RULE_SET_VERSION_CREATED` with payload `{community_id, version, parent_id, text_sha256 (hex), rule_set_version_id, config_id, activated_at}`. Returns `AdminCreateRuleSetResponse { rule_set_version_id, version, config_id, governance_log_id }`.

- **`GET /api/v4/governance/admin/rule-sets?community_id=<id>`** (`admin_list_rule_sets`) — reads the query param (validated through the fixed `Scope::parse_wire` or a new `CommunityId::try_from_positive` helper), checks `CommunityModeratorView` OR instance-admin capability, returns `AdminListRuleSetsResponse { versions: Vec<RuleSetVersionView>, active_version_id: Option<i32> }`. The `active_version_id` is read via the existing `config::get_int_opt(cache, pool, Scope::Community(community_id), "rule_set.active_version_id")` — one of the first production call sites for the v1-AD-b absent-aware accessor.

- **`create_report.rs` edit** — before constructing `ModerationCaseInsertForm`, read `rule_set.active_version_id` for the case's community (via `config::get_int_opt`), snapshot the **7 `requires_re_jury` keys** into a JSONB (`build_applied_config_snapshot` helper lives in a new `case_open_snapshot.rs` module in `crates/api/api/src/governance/`), and populate both new fields on the insert form. For the snapshot, use the case's scope — if `community_id.is_some()`, read at `Scope::Community(id)`; else `Scope::Instance`. **No read-site updates** — no handler today reads either column, so v1-AD-c is write-only for the pin.

- **Refactor `admin_config.rs` write path (Issue #77).** Introduce a private helper `build_admin_config_changed_payload(&row, &data, previous: &ConfigValueWithProvenance) -> serde_json::Value` that takes the pre-tx value/provenance tuple already computed at line 477 and builds the full payload `{scope, key, value_type, value, previous_value, previous_from, reason}`. Thread `previous` through `process_set_config` so the tx closure doesn't re-fetch. Update `project_to_audit_entry` to read `previous_value` and `previous_from` directly from the stored payload instead of leaving them `None` with a TODO comment. Byte-identity to the shell script is preserved for `admin_config_changed` entries written by the shell (the shell doesn't emit `previous_value`; the handler now does) — this is a deliberate **payload-shape evolution, not a regression**: NOT5 gate 3 requires *audit-trail continuity* (both writers use the same `entry_kind` and `append` helper), not field-for-field parity when the HTTP path has strictly more info. See §10.5 for the NOT5 rationale line the CR bot will inspect.

- **Fix `Scope::parse_wire` (Issue #78).** Change the signature from `pub fn parse_wire(s: &str) -> Option<Self>` to `pub fn parse_wire(s: &str) -> Result<Self, ScopeParseError>`. New enum `ScopeParseError { Malformed(String), NonPositiveCommunityId(i32) }` in `config.rs`. Add `impl From<ScopeParseError> for LemmyError` (or map at the handler-level `ok_or_else`). Update the single existing call site in `admin_config.rs:419` and the new v1-AD-c call sites (rule-set GET + POST community-id validation). Add an e2e test `scope_parse_wire_rejects_negative_community_id` asserting the error variant (not the message). Eight existing v1-AD-b tests that call `Scope::parse_wire` implicitly through the handler's `data.scope` string do not need to change (happy paths still work; denial paths still test the same behaviour).

Capability checks follow the v1-AD-b pattern: `check_policy(metadata, scope, local_user_view, pool)` returns `Ok(Ok(()))` on permit, `Ok(Err(DenialReason))` on denial — rule-set operations synthesize a virtual metadata shape for the dispatch (`ConfigScope::Community` analogue) since the write is community-scoped. Denials emit `admin_config_change_denied` entries for the rule-set-create path too, mirroring the v1-AD-b denial-log pattern exactly (the denial_reason text differs but the emission site is identical).

Routes register alongside the existing `/admin/config` scope at `crates/api/routes/src/lib.rs:539-541`. Sub-phase branch is `phase-v1-AD-c` (already cut from `governance-v0` at `f03ed1cba`, post-PR #76 merge). PR target is `governance-v0` per `.claude/rules/phase-branch.md`.

### 4.1 Architecturally load-bearing decisions locked in this sub-phase

- **`rule_set_version_created` is a NEW entry_kind.** Const lives in `crates/db_schema/src/source/governance/governance_log.rs` (per the canonical-definition location established at Phase 6 DQ-6.6-inbound), re-exported through the shim at `crates/api/api/src/governance/governance_log.rs`. The `.claude/rules/governance-log-entry-kind-registry.md` registry rule MANDATES matching row in the registry. v1-AD-c's task 6 updates the registry file alongside the const.
- **Rule-set creation writes THREE rows atomically, not two.** (a) `rule_set_version` insert, (b) `governance_config` insert for `rule_set.active_version_id`, (c) `governance_log` append. All three inside one `run_transaction` closure. `governance_log::append`'s internal SAVEPOINT promotion (governance_log.rs:193-204) means a signing failure rolls back (a) and (b) together with the log.
- **Snapshot population is WRITE-ONLY in v1-AD-c.** We populate `moderation_case.applied_config_snapshot` at case-open; **no code reads it yet**. `admin_assign_jury.rs:120` + `:222` still read from live config. jury-mechanics-v1 owns the read-site switch. This is a deliberate scoping choice — adding read-site logic now would either break pre-v1 cases (where the column is `None`) without a fallback, OR require every read site to implement the fallback branch twice (once for pre-v1, once for post-v1). Simpler: v1-AD-c pins the data, jury-mechanics-v1 consumes it with the pre-v1 fallback in the same PR.
- **Rule-set pin is AT CASE-OPEN, not at decision time.** The moderation_case doc-comment (`moderation_case.rs:42-43`) says "at case-open time". The v1-AD-a plan's §4.1 stub line referencing "decision time" is **drift** from the schema doc and is overridden by this plan. Rationale: immutable case = fixed rule context. Decision-time mutation would create surprises (cases transitioning from `InReview` → `Decided` would see the pin appear).
- **Payload shape evolution for `admin_config_changed`.** v1-AD-c extends the payload from `{scope, key, value_type, value, reason}` to `{scope, key, value_type, value, previous_value, previous_from, reason}`. NOT5 gate 3 is re-interpreted: continuity = same `entry_kind` + same `append` helper, NOT field-for-field shell parity. The shell script (`scripts/brehon/admin-config-write.sh:148`) is still eligible for deprecation whenever OQ-018 retirement fires; until then, shell rows just lack the new fields (their JSONB payload parses fine; `project_to_audit_entry` returns `None` for both previous_* fields on shell-written rows).
- **Scope::parse_wire return type changes from Option to Result.** Breaking change inside the api crate; zero external callers exist (only call site is `admin_config.rs:419`). The change is justified by Issue #78 and is audit-visible.
- **No new migrations.** `add_rule_set_versions` + `add_case_applied_config_snapshot` + `seed_v1_config_keys` all shipped in v1-AD-a. v1-AD-c is pure Rust + route wiring + tests.
- **No SSE, no askama, no dashboard aggregate.** Those are v1-AD-d (descoped-for-now per v1-AD-b §6).
- **No step-up auth for rule-set POST.** `rule_set.active_version_id` is listed in `requires_step_up` metadata (PRD §7.2), but v1-AD-b's `governance.dashboard.step_up_enforced = false` default routes rule-set POST through the advisory-only log path per the existing v1-AD-b code at `admin_config.rs:460-474`. v1-AD-c's rule-set handler triggers the same advisory branch for its embedded config flip — no new code needed.

---

## 5. Metadata

| Field | Value |
|---|---|
| Type | `HANDLER + DTO + CROSS_CUTTING + BUG_FIX` |
| Complexity | MEDIUM |
| Crates affected | `lemmy_db_schema` (ENTRY_KIND const), `lemmy_api_common` (DTOs), `lemmy_api` (2 new handlers + snapshot helper module + admin_config refactor + Scope enum + governance_log shim re-export), `lemmy_api_crud` (create_report.rs insert-form edit), `lemmy_api_routes` (route registrations), `lemmy_server` (tests/e2e.rs) |
| v0 step | v1 post-MVP per [ADR-010] — keystone sub-phase closes |
| Dependencies | v1-AD-b merged at `governance-v0` (tip `f03ed1cba`, PR #76). **Pre-flight check: `governance-v0` HEAD must contain `get_int_opt` at `crates/api/api/src/governance/config.rs`, `RuleSetVersion` at `crates/db_schema/src/source/governance/rule_set_version.rs`, and the `applied_config_snapshot` + `rule_set_version_id` columns on `moderation_case` — task 0 verifies.** |
| Estimated tasks | **9 tasks** (Tasks 0–8 enumerated in §13), including 2 bug fixes (#77, #78), 2 handlers, 1 pin-wire-up, 1 registry update, 1 tests-and-e2e batch, plus task 0 pre-flight and task 8 final polish |
| Sub-phase target branch | `phase-v1-AD-c` (already branched from `governance-v0` at `f03ed1cba` per task 0) |
| PR target | `governance-v0` (per `.claude/rules/phase-branch.md`) |
| Blocks | v1-AD-d (dashboard aggregate needs rule-set list surface), jury-mechanics-v1 (reads the snapshot this plan pins) |
| Unblocks | itself — v1-AD-b is merged |

---

## 6. Sub-phase position (v1-AD-a → b → c)

| Sub-phase | Status | What's landed / pending |
|---|---|---|
| v1-AD-a | ✅ merged (PR #72, commit `e61f78edf`) | 4 migrations (incl. `add_rule_set_versions` + `add_case_applied_config_snapshot`), `ConfigKeyMetadata` registry (61 entries), 2 new `ENTRY_KIND_ADMIN_CONFIG_*` consts, Diesel models for `RuleSetVersion`, entry-kind registry rule |
| v1-AD-b | ✅ merged (PR #76, commit `f03ed1cba`) | 3 admin-config handlers, `get_*_opt` accessor family, dry-run impact queries, capability gates, audit list handler, 13 e2e tests |
| **v1-AD-c** (this plan) | pending | 2 rule-set handlers, case-open snapshot pin, #77 provenance-threading refactor, #78 typed-scope-parse, 1 new ENTRY_KIND const |
| v1-AD-d | descoped-for-now (v1-AD-b §6) | `/admin/dashboard` aggregate + SSE `/admin/audit/stream` — picked up as separate plan post-v1-AD-c if pilot demands |
| v1-AD-e | DEFERRED | Askama pages — descoped from v1-AD wave per OQ-V1-AD-01 resolution |

v1-AD-c **closes the admin-dashboard keystone minimum-v1**. After merge, the four consumer sub-PRDs (jury-mechanics, reputation-tuning, sponsor-liability, federation-inbound) can build on a complete config + rule-set + audit substrate.

---

## 7. Open questions already resolved / reserved

| OQ | Resolution | Impact on v1-AD-c |
|---|---|---|
| OQ-002 | Rule-set versioning (append-only, per-community) — resolved via PRD §3.6 | **This plan ships the CRUD surface.** `rule_set_version` + `moderation_case.rule_set_version_id` pin + `rule_set.active_version_id` config-pointer flip — all three wired here. |
| OQ-V1-AD-01 | Defer HTML pages to v1.x | v1-AD-c ships no askama; no template crate dep added |
| OQ-V1-AD-02 | Hand-roll SSE via async-stream | v1-AD-c ships no SSE |
| OQ-V1-AD-03 | Dry-run impact pre-tx read-only query; no SAVEPOINT | Binding on §10.3 refactor: the pre-tx `(current_value, current_from)` read IS the impact-query source, and the refactor merely threads it through rather than re-fetching |
| OQ-018 | Admin HTTP config-write endpoint | Shipped in v1-AD-b; this plan extends the payload shape (see §4.1 and §10.5) |

**No new OQs opened by v1-AD-c.** If blocking ambiguity emerges mid-implementation, it goes to `.claude/decision-queue.json` per `.claude/rules/decision-queue.md`.

### 7.1 Potential decision-queue triggers (pre-identified)

- **DQ trigger A**: if `CONFIG_KEY_METADATA` contains fewer than 7 `requires_re_jury: true` rows at implementation time (drift from §5.2 of the PRD), flag this as DQ-V1-AD-C-0X and pause snapshot wiring until advisor confirms which keys are in-scope for the snapshot.
- **DQ trigger B**: if `governance_log::append` signature drifts (Phase 6 or polish commits may have touched it), surface the diff before task 3 and either adapt or escalate.
- **DQ trigger C**: if pre-phase harness audit probe 4 (negative exit-code propagation, `cargo-check.bat --features nonexistent_xyz`) returns exit 0, stop and fix the wrapper before task 1 per `.claude/rules/pre-phase-harness-audit.md`.

---

## 8. Flow design

### Before state (governance-v0 HEAD `f03ed1cba`, v1-AD-b merged via PR #76)

```text
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              BEFORE STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  HTTP API surface:                                                            ║
║    /admin/config              (POST)  — v1-AD-b admin_set_config              ║
║    /admin/config              (GET)   — v1-AD-b admin_get_config              ║
║    /admin/config/audit        (GET)   — v1-AD-b admin_get_config_audit        ║
║    /admin/rule-sets           DOES NOT EXIST                                  ║
║                                                                               ║
║  Substrate shipped but unused:                                                ║
║    rule_set_version           table present, 0 rows                           ║
║    moderation_case.applied_config_snapshot   column present, 100% NULL        ║
║    moderation_case.rule_set_version_id       column present, 100% NULL        ║
║    rule_set.active_version_id config key     un-seeded (by design)            ║
║                                                                               ║
║  Known bugs from PR #76 CodeRabbit triage:                                    ║
║    #77  admin_set_config log payload missing previous_value + previous_from   ║
║         → project_to_audit_entry returns None for both with a TODO comment    ║
║    #78  Scope::parse_wire accepts community:-1 → downstream 500 at DB lookup  ║
║                                                                               ║
║  PAIN:                                                                        ║
║    - operators cannot create/list rule-set versions except via raw SQL       ║
║    - in-flight cases have nothing to grandfather against when an admin        ║
║      edits a requires_re_jury key — ADR-010 invariant at risk                 ║
║    - audit GET returns N+1 self-joins to hydrate previous_value OR            ║
║      returns None forever; neither is correct                                 ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### After state (end of v1-AD-c)

```text
╔═══════════════════════════════════════════════════════════════════════════════╗
║                               AFTER STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  HTTP API surface:                                                            ║
║    /admin/config              (POST)  — unchanged                             ║
║    /admin/config              (GET)   — unchanged                             ║
║    /admin/config/audit        (GET)   — previous_value + previous_from        ║
║                                          now populated for post-v1-AD-c rows  ║
║    /admin/rule-sets           (POST)  — v1-AD-c admin_create_rule_set   NEW   ║
║    /admin/rule-sets           (GET)   — v1-AD-c admin_list_rule_sets    NEW   ║
║                                                                               ║
║  Substrate fully wired:                                                       ║
║    rule_set_version           writes flow through admin_create_rule_set       ║
║    moderation_case.applied_config_snapshot populated at every case open       ║
║    moderation_case.rule_set_version_id populated when community has active    ║
║                                        rule-set version at case-open time     ║
║    rule_set.active_version_id config key flipped atomically with each         ║
║                               admin_create_rule_set call                      ║
║                                                                               ║
║  governance_log entry_kinds:                                                  ║
║    rule_set_version_created   NEW — emitted by admin_create_rule_set          ║
║    admin_config_changed       extended payload: + previous_value, + prev_from ║
║    admin_config_change_denied unchanged                                       ║
║                                                                               ║
║  Scope validation:                                                            ║
║    Scope::parse_wire  → Result<Scope, ScopeParseError>                        ║
║    ScopeParseError::Malformed / NonPositiveCommunityId — distinct variants    ║
║    handler maps errors → 400 with typed text (not generic Unknown wrapper)    ║
║                                                                               ║
║  VALUE: jury-mechanics-v1 can read the snapshot for grandfathering; rule-set  ║
║         CRUD is end-to-end operator-usable; audit trail self-contained.       ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### Flow — admin_create_rule_set

```text
  POST /admin/rule-sets { community_id: 42, rule_text: "…", parent_id: Some(7) }
              │
              ▼
  parse wire + validate shape (rule_text non-empty, len <= rule_set.text_max_bytes)
              │
              ▼
  check_policy(ConfigScope-like::Community, Scope::Community(42), user, pool)
              │            │
              │            └─── denial → emit admin_config_change_denied + 403
              │                        (scope:"community:42", key:"rule_set.active_version_id",
              │                         denial_reason: "community_moderator_required")
              ▼
  compute version (parent_id.version + 1 OR 1) + text_sha256
              │
              ▼
  actor_pseudonym_helper::get_or_create(pool, admin.person_id)
              │
              ▼
  conn.run_transaction(|conn| async move {
       INSERT rule_set_version  → row.id
       INSERT governance_config (scope="community:42",
                                  key="rule_set.active_version_id",
                                  value_int=row.id.0)
                                → config_id
       governance_log::append(
         ENTRY_KIND_RULE_SET_VERSION_CREATED,
         payload = { community_id, version, parent_id,
                     text_sha256: hex, rule_set_version_id, config_id,
                     activated_at },
         actor_pseudonym,
       ) → log_id
  })
              │
              ▼
  200 { rule_set_version_id, version, config_id, governance_log_id }
```

### Flow — case-open snapshot pin (inside create_report)

```text
  POST /governance/report { target, reason, community_id: Some(42) }
              │
              ▼
  existing validation + dedupe (unchanged)
              │
              ▼
  build_applied_config_snapshot(pool, community_id = Some(42)):
       read 7 requires_re_jury keys via config::get_int / get_bool / get_text
       at Scope::Community(42) (cascading to instance then const)
                                         → serde_json::Value
              │
              ▼
  config::get_int_opt(pool, Scope::Community(42),
                       "rule_set.active_version_id")      → Option<i64>
              │
              ▼
  ModerationCaseInsertForm {
       ... existing fields ...
       applied_config_snapshot: Some(snapshot_json),
       rule_set_version_id: active_version_id
         .and_then(|i| i32::try_from(i).ok())
         .map(RuleSetVersionId),
  }
              │
              ▼
  existing INSERT path (unchanged except the two new fields now populated)
```

---

## 9. Mandatory reading

**Before writing any code in a given task, the implementer reads these file:line ranges.** This section is the fast-path index for the agent's first iteration of each task.

### 9.1 Source files (patterns to mirror)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `crates/api/api/src/governance/admin_config.rs` | 408–544 | `admin_set_config` — overall shape tasks 2 + 3 mirror: capability gate, pre-tx read, single `run_transaction` closure |
| P0 | `crates/api/api/src/governance/admin_config.rs` | 550–602 | `process_set_config` — the tx closure refactor target for Issue #77 |
| P0 | `crates/api/api/src/governance/admin_config.rs` | 763–801 | `check_policy` — the capability dispatcher; task 2 synthesises a virtual metadata row to reuse this pattern, task 3 adds a `ScopeParseError` → `LemmyError` handler branch |
| P0 | `crates/api/api/src/governance/admin_config.rs` | 805–835 | `emit_denial_log` — pattern for denial entries; task 2 reuses verbatim |
| P0 | `crates/api/api/src/governance/admin_config.rs` | 1155–1218 | `admin_get_config_audit` — the audit pagination pattern task 2's `admin_list_rule_sets` half-mirrors (with simpler filters) |
| P0 | `crates/api/api/src/governance/admin_config.rs` | 1284–1330 | `project_to_audit_entry` — Issue #77 read-side fix target (`previous_value: None` → `previous_value: payload.get("previous_value")…`) |
| P0 | `crates/api/api/src/governance/config.rs` | 58–151 | `Scope` + `parse_wire` — Issue #78 edit target |
| P0 | `crates/api/api/src/governance/config.rs` | 333–498 | `get_*_opt` accessor family — used by tasks 1 + 5 |
| P0 | `crates/api/api/src/governance/config.rs` | 923–1420 | `CONFIG_KEY_METADATA` — cross-check `requires_re_jury: true` rows for task 5 snapshot keyset; verify `ConfigScope::{Instance,Community,Both}` before tests (v1-AD-b drift-notes lesson) |
| P0 | `crates/api/api/src/governance/admin_emergency_remove.rs` | 74–218 | Multi-write tx pattern (post.removed + moderation_case + governance_log in one closure) — closest analogue to task 2's three-row rule-set-create tx |
| P0 | `crates/api/api_crud/src/governance/create_report.rs` | 120–240 | Case-open handler — task 5 adds snapshot + rule-set-pin **here** (before the existing `ModerationCaseInsertForm` construction at ~178–192) |
| P0 | `crates/db_schema/src/source/governance/rule_set_version.rs` | all (48 lines) | Diesel model + InsertForm — used verbatim by task 2 |
| P0 | `crates/db_schema/src/source/governance/moderation_case.rs` | 38–68 | InsertForm columns — task 5 populates `applied_config_snapshot` + `rule_set_version_id` |
| P0 | `crates/db_schema/src/source/governance/governance_log.rs` | 110–160 | `ENTRY_KIND_*` const location (canonical); task 6 adds `ENTRY_KIND_RULE_SET_VERSION_CREATED` here |
| P1 | `crates/api/api_common/src/governance.rs` | 364–509 | DTO style conventions for tasks 2's request/response structs |
| P1 | `crates/api/routes/src/lib.rs` | 531–546 | `/admin` scope block — task 7 adds rule-sets subscope |
| P1 | `crates/api/api/src/governance/actor_pseudonym_helper.rs` | all | `get_or_create(pool, person_id)` — called pre-tx by task 2 |
| P1 | `crates/db_views/community_moderator/src/impls.rs` | `check_is_community_moderator` impl | capability gate for community-scoped rule-set writes |
| P1 | `crates/server/tests/e2e.rs` | 4330–4380, 4579–4629, 4635–4710 | v1-AD-b test style for task 8; scope-rejection + moderator-capability patterns |
| P1 | `.claude/rules/governance-log-entry-kind-registry.md` | v1-AD-a section | Registry format — task 6 adds a `v1-AD-c entry kinds (1, this sub-phase)` section |

### 9.2 External documentation

| Source | Version | Section | Why |
|---|---|---|---|
| [`sha2` on docs.rs](https://docs.rs/sha2/) | workspace-pinned (check `Cargo.toml`) | `Sha256::new() / update / finalize` | Task 2 computes `text_sha256` — reuse the same construction used by `governance_log::append` (the hash-chain signer) for consistency |
| [`serde_json::Value` docs](https://docs.rs/serde_json/) | workspace-pinned | `Value::get`, `as_i64`, `as_str` | Task 3 reads `previous_value` back in `project_to_audit_entry` — mirror the `.get(...).and_then(|v| v.as_str())` pattern already at line 1286–1300 |
| [`diesel_async::AsyncConnection::transaction`](https://docs.rs/diesel-async/) | workspace-pinned | `AsyncConnection::transaction` / `scope_boxed` closure shape | Task 2 runs three writes in one closure — mirror `admin_config.rs:520-535` verbatim |
| [PostgreSQL `UNIQUE` constraint semantics](https://www.postgresql.org/docs/current/ddl-constraints.html#DDL-CONSTRAINTS-UNIQUE-CONSTRAINTS) | PG 15+ | UNIQUE on (community_id, version) | Task 2 relies on the `UNIQUE (community_id, version)` constraint in `rule_set_version` — if two admins race on `version = prev.version + 1`, one gets a unique-violation error. The handler surfaces this as a typed 409 conflict (task 2 GOTCHA). |

### 9.3 Forbidden reading

- `chat1.md`, `chat2.md`, `.docx` files in `docs/brehon-law-inspired-network/` — archival only per CLAUDE.md.
- Any `old-prp-commands/` file — superseded by current `.claude/commands/`.
- `docs/research/*` under brehon-law-inspired-network — that path is the OLD canonical before the `docs/brehon-law-inspired-network/` move; use the new path.

---

## 10. Patterns to mirror

### 10.1 Capability gate + denial-log emission (from `admin_config.rs:445–474` + `:805–835`)

```rust
// SOURCE: crates/api/api/src/governance/admin_config.rs:445-474
// COPY THIS PATTERN for admin_create_rule_set's capability branch.

let policy = check_policy(&metadata, scope, &local_user_view, &mut context.pool()).await?;
if let Err(reason) = policy {
  emit_denial_log(
    &mut context.pool(),
    local_user_view.person.id,
    &data_for_denial_payload,  // adapter — see §10.2
    &scope,
    reason,
  )
  .await?;
  return Err(LemmyErrorType::NotAnAdmin.into());  // 403
}
```

For `admin_create_rule_set`, the "metadata" analogue is synthesised inline. The `ConfigKeyMetadata` struct has **10 fields** (no `default` field — defaults live as separate `DEFAULT_*` consts and the parity map `SEEDED_KEYS_WITH_CONSTS`; verify against `config.rs:113-124`):

```rust
// Per §4.1 — rule-set writes are ConfigScope::Community-like. Reuse check_policy
// by synthesising a virtual metadata struct that declares Community scope.
let virtual_metadata = ConfigKeyMetadata {
  key: "rule_set.active_version_id",
  value_type: ValueType::Int,
  valid_range: None,
  valid_enum: None,
  scope: ConfigScope::Community,
  requires_re_jury: false,
  requires_step_up: true,   // per PRD §7.2 — advisory in v1
  apply_at_default: ApplyAt::NextJuryCycle,
  description: "",
  doc_anchor: "",
};
```

**NOTE**: task 3's ACTUAL implementation does NOT synthesise a virtual metadata struct and does NOT reuse `check_policy` — that function's match arms are tightly coupled to the `admin_set_config` data shape (it wants `ConfigKeyMetadata::scope` AND `Scope` as separate args). For rule-set create, the inline if-chain in the §13 task-3 handler code is simpler and correct:

```rust
let is_mod = CommunityModeratorView::check_is_community_moderator(...).await.is_ok();
let is_admin_ok = lemmy_api_utils::utils::is_admin(&local_user_view).is_ok();
if !(is_mod || is_admin_ok) { /* emit denial log + return 403 */ }
```

The virtual-metadata snippet above is retained for **reference only** — if a future refactor wants to unify rule-set-create and admin_set_config under one capability-check call, the metadata shape is what it would pass in.

### 10.2 Scope parsing adapter (Issue #78 resolution)

```rust
// SOURCE: crates/api/api/src/governance/config.rs:143-150 (BEFORE)
pub fn parse_wire(s: &str) -> Option<Self> {
  if s == "instance" {
    return Some(Scope::Instance);
  }
  let rest = s.strip_prefix("community:")?;
  let id = rest.parse::<i32>().ok()?;
  Some(Scope::Community(CommunityId(id)))
}

// AFTER — task 3:
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeParseError {
  Malformed(String),
  NonPositiveCommunityId(i32),
}

impl Scope {
  pub fn parse_wire(s: &str) -> Result<Self, ScopeParseError> {
    if s == "instance" {
      return Ok(Scope::Instance);
    }
    let Some(rest) = s.strip_prefix("community:") else {
      return Err(ScopeParseError::Malformed(s.to_owned()));
    };
    let id: i32 = rest.parse().map_err(|_| ScopeParseError::Malformed(s.to_owned()))?;
    if id < 1 {
      return Err(ScopeParseError::NonPositiveCommunityId(id));
    }
    Ok(Scope::Community(CommunityId(id)))
  }
}

impl std::fmt::Display for ScopeParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ScopeParseError::Malformed(s) => write!(
        f, "scope `{s}` is not recognised — expected `instance` or `community:<positive int>`"
      ),
      ScopeParseError::NonPositiveCommunityId(n) => write!(
        f, "community_id must be >= 1; got {n}"
      ),
    }
  }
}
```

Call-site update at `admin_config.rs:419`:

```rust
// BEFORE
let scope = Scope::parse_wire(&data.scope).ok_or_else(|| {
  LemmyErrorType::Unknown(format!(
    "scope `{}` is not recognised — expected `instance` or `community:<id>`",
    data.scope
  ))
})?;

// AFTER — task 3 edit
let scope = Scope::parse_wire(&data.scope).map_err(|e| LemmyErrorType::Unknown(e.to_string()))?;
```

**Why keep the handler wrap `LemmyErrorType::Unknown`**: changing the HTTP error type on the happy-path handler body would break v1-AD-b's existing scope-mismatch e2e tests that assert on `result.is_err()`. The *new* test (`scope_parse_wire_rejects_negative_community_id`) asserts on the **ScopeParseError variant** pre-wrap, not on the wrapper string. The wrap preserves wire behaviour (400 Bad Request with readable text) while the enum gives us a test-assertable discrimination surface.

### 10.3 Issue #77 refactor — pre-tx tuple threaded into process_set_config

```rust
// SOURCE: crates/api/api/src/governance/admin_config.rs:477 (pre-tx read)
let (current_value, current_from) = read_effective(pool, &metadata, scope).await?;

// AFTER — task 4: thread into process_set_config signature and payload builder
async fn process_set_config(
  conn: &mut diesel_async::AsyncPgConnection,
  admin_id: lemmy_db_schema_file::PersonId,
  admin_pseudonym: String,
  scope: Scope,
  metadata: ConfigKeyMetadata,
  data: AdminSetConfig,
  previous: ConfigValueWithProvenance,  // NEW — was ignored before
) -> LemmyResult<(i32, i64, chrono::DateTime<chrono::Utc>)> {
  // ... unchanged INSERT ...

  // NEW — extract into helper
  let payload = build_admin_config_changed_payload(&row, &data, &previous);

  let log_row = governance_log::append(...).await?;
  Ok((row.id.0, log_row.id.0, row.valid_from))
}

// Imports at top of admin_config.rs (both structs live in api_common):
//   use lemmy_api_common::governance::{ConfigValueWithProvenance, ConfigChangePreview};
fn build_admin_config_changed_payload(
  row: &GovernanceConfig,
  data: &AdminSetConfig,
  previous: &ConfigValueWithProvenance,
) -> serde_json::Value {
  // Field order: shell-wrapper's 5 fields first, then the two new fields last
  // (so shell-written audit rows still parse cleanly; they just lack the tail
  // fields, which project_to_audit_entry degrades to None gracefully).
  json!({
    "scope":          row.scope,
    "key":            row.key,
    "value_type":     row.value_type,
    "value":          data.value,
    "reason":         data.reason,
    "previous_value": previous.value,
    "previous_from":  previous.effective_from,
  })
}
```

Read-side fix at `project_to_audit_entry`:

```rust
// SOURCE: crates/api/api/src/governance/admin_config.rs:1284-1330 (BEFORE)
// AFTER — task 4:
let previous_value = payload.get("previous_value").cloned();  // None for pre-v1-AD-c rows
let previous_from = payload
  .get("previous_from")
  .and_then(|v| v.as_str())
  .map(str::to_owned);

AdminConfigAuditEntry {
  // ... unchanged fields ...
  previous_value,
  previous_from,   // NEW DTO field — task 4 also extends AdminConfigAuditEntry
  // ...
}
```

### 10.4 Multi-write transaction (from `admin_emergency_remove.rs:74-218`)

```rust
// SOURCE: crates/api/api/src/governance/admin_emergency_remove.rs:74-218
// COPY THIS PATTERN for admin_create_rule_set's three-write tx closure.

// Pre-tx: get_or_create pseudonym (actor_pseudonym_helper is the GDPR layer)
let admin_pseudonym = actor_pseudonym_helper::get_or_create(
  &mut context.pool(),
  local_user_view.person.id,
).await?;

let conn = &mut get_conn(&mut context.pool()).await?;

let (rsv_row, config_id, log_id) = conn
  .run_transaction(|conn| {
    async move {
      // Write 1 — rule_set_version
      let rsv_form = RuleSetVersionInsertForm { /* ... */ };
      let rsv: RuleSetVersion = insert_into(rule_set_version::table)
        .values(&rsv_form)
        .returning(RuleSetVersion::as_returning())
        .get_result(conn)
        .await?;

      // Write 2 — governance_config row flipping active_version_id
      let cfg_form = GovernanceConfigInsertForm {
        scope: Scope::Community(community_id).as_str().into_owned(),
        key: "rule_set.active_version_id".to_string(),
        value_type: "int".to_string(),
        value_int: Some(rsv.id.0.into()),
        value_float: None,
        value_bool: None,
        value_text: None,
        updated_by: Some(admin_id),
      };
      let cfg: GovernanceConfig = insert_into(governance_config::table)
        .values(&cfg_form)
        .returning(GovernanceConfig::as_returning())
        .get_result(conn)
        .await?;

      // Write 3 — governance_log
      let payload = json!({
        "community_id":         community_id.0,
        "version":              rsv.version,
        "parent_id":            rsv.parent_id.map(|p| p.0),
        "text_sha256":          hex::encode(&rsv.text_sha256),
        "rule_set_version_id": rsv.id.0,
        "config_id":            cfg.id.0,
        "activated_at":         rsv.created_at,
      });
      let log_row = governance_log::append(
        &mut conn.into(),
        ENTRY_KIND_RULE_SET_VERSION_CREATED,
        payload,
        Some(admin_pseudonym.clone()),
      ).await?;

      Ok((rsv, cfg.id.0, log_row.id.0))
    }
    .scope_boxed()
  })
  .await?;
```

### 10.5 NOT5 reply line (for CR bot)

Paste the following into the task-4 commit body AND the final PR description so CodeRabbit does not flag the payload evolution as a regression:

> NOT5 gate 3 per PRD §8.4 requires *audit-trail continuity* across the shell→HTTP deprecation window, not field-for-field payload parity. Continuity here means: both paths emit `entry_kind = "admin_config_changed"` via the same `governance_log::append` helper, and `project_to_audit_entry` treats missing fields (as in shell-written rows) as `None` rather than erroring. v1-AD-c extends the HTTP path's payload with `previous_value` + `previous_from` per Issue #77; shell-written rows pre- and post-deprecation remain parseable. The shell script is still eligible for retirement whenever the three deprecation conditions hold.

---

## 11. Files to change

| File | Action | Justification |
|---|---|---|
| `crates/db_schema/src/source/governance/governance_log.rs` | UPDATE | Add `pub const ENTRY_KIND_RULE_SET_VERSION_CREATED: &str = "rule_set_version_created";` at the canonical-definition location (alphabetical) |
| `crates/api/api/src/governance/governance_log.rs` | UPDATE | Add `pub use ... ENTRY_KIND_RULE_SET_VERSION_CREATED;` in the shim re-export block per `.claude/rules/governance-log-entry-kind-registry.md` acceptance #3 |
| `.claude/rules/governance-log-entry-kind-registry.md` | UPDATE | Add `v1-AD-c entry kinds (1)` section with `ENTRY_KIND_RULE_SET_VERSION_CREATED` row; update the `Acceptance invariants` count from 27 → 28 at sub-phase end |
| `crates/api/api/src/governance/config.rs` | UPDATE | `parse_wire` signature change + `ScopeParseError` enum + `Display` impl |
| `crates/api/api_common/src/governance.rs` | UPDATE | Add DTOs: `AdminCreateRuleSet`, `AdminCreateRuleSetResponse`, `AdminListRuleSetsRequest`, `AdminListRuleSetsResponse`, `RuleSetVersionView`; add `previous_from: Option<String>` to `AdminConfigAuditEntry` |
| `crates/api/api/src/governance/admin_rule_sets.rs` | CREATE | `admin_create_rule_set` + `admin_list_rule_sets` handlers + private helpers (virtual-metadata synthesiser, text-sha256 hasher) |
| `crates/api/api/src/governance/admin_config.rs` | UPDATE | Issue #77 refactor: extract `build_admin_config_changed_payload`; thread `previous: ConfigValueWithProvenance` into `process_set_config`; update `project_to_audit_entry` to hydrate `previous_value` + `previous_from` from payload; update `parse_wire` call at line 419 |
| `crates/api/api/src/governance/case_open_snapshot.rs` | CREATE | `build_applied_config_snapshot(pool, scope)` — reads the 7 `requires_re_jury` keys at the given scope, returns `serde_json::Value` |
| `crates/api/api/src/governance/mod.rs` | UPDATE | `pub mod admin_rule_sets;` + `pub mod case_open_snapshot;` declarations |
| `crates/api/api_crud/src/governance/create_report.rs` | UPDATE | Read `rule_set.active_version_id` via `get_int_opt` + call `build_applied_config_snapshot`; populate both new fields on `ModerationCaseInsertForm` |
| `crates/api/routes/src/lib.rs` | UPDATE | Add `/admin/rule-sets` GET + POST routes alongside the existing `/admin/config` block |
| `crates/server/tests/e2e.rs` | UPDATE | Add 8 e2e tests: 4 for rule-sets (happy-path, duplicate-version, non-moderator, non-positive-community-id), 2 for Issue #77 (previous_value+from populated, audit GET returns them), 1 for Issue #78 (`scope_parse_wire_rejects_negative_community_id`), 1 for case-open snapshot (`case_open_pins_applied_config_snapshot_and_rule_set_version_id`) |

Total: **12 file edits** (8 UPDATE, 2 CREATE, 2 rules/docs).

---

## 12. NOT building in v1-AD-c

- **No `/admin/dashboard` aggregate endpoint.** v1-AD-d scope per v1-AD-b §6.
- **No SSE `/admin/audit/stream`.** v1-AD-d scope; OQ-V1-AD-02 resolved but implementation lives there.
- **No askama HTML pages.** v1-AD-e DEFERRED per OQ-V1-AD-01.
- **No `rule_set.text_max_bytes` enforcement beyond the wire validation.** Task 2 checks `rule_text.len() <= config::get_int_opt(..., Scope::Instance, "rule_set.text_max_bytes").unwrap_or(65536)` — no DB-side trigger. A trigger would require a new migration, out of v1-AD-c scope per §11.
- **No read-site switch for `applied_config_snapshot`.** `admin_assign_jury.rs:120` + `:222` still read from live config. jury-mechanics-v1 flips them to read from the snapshot with pre-v1 fallback. Per §4.1 load-bearing decision.
- **No step-up auth enforcement.** Rule-set POST is listed in `requires_step_up` keys per PRD §7.2, but v1-AD-b's `governance.dashboard.step_up_enforced = false` default keeps it advisory. v2 milestone.
- **No `admin_update_rule_set` or `admin_delete_rule_set`.** Append-only per ADR-010 + PRD §3.6.
- **No `GET /admin/rule-sets/<id>`** (single-version detail). The list endpoint returns full `RuleSetVersionView` objects including `rule_text` — no detail endpoint needed.
- **No rule-text diff endpoint.** Deferred to v1.x if pilot asks; clients can diff client-side.
- **No federation of rule-set versions.** Rule-sets stay instance-local per ADR-014 (governance signals are fork-only AP types; inbound governance is v1-federation-inbound and is advisory-only).
- **No backfill of `applied_config_snapshot` for pre-v1-AD-c cases.** Pre-existing cases retain `None`. Any consumer that cares (none today) must fallback to live config.
- **No Rust const default for `rule_set.active_version_id`.** v1-AD-a deliberately left the key un-seeded with no `DEFAULT_*` const; v1-AD-b's `get_int_opt` returns `Ok(None)` for legitimately-absent keys. v1-AD-c continues this pattern.
- **No `EXPECTED_SEED_COUNT_V1_AD_C` parity counter.** v1-AD-c ships zero new seed rows; the existing `EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD_A = 61` parity invariant is unchanged.
- **No `community_id: None` (instance-scoped) rule-sets.** Rule-sets are per-community by design (PRD §3.6). The DB column is `NOT NULL`. Task 2 enforces the handler-level check.

---

## 13. Step-by-step tasks

Execute in order. One commit per task. Each task has a MIRROR reference, an exact file path, and a validation command. Task 0 is **mandatory** per `.claude/rules/pre-phase-harness-audit.md` and runs BEFORE any implementation.

### Task 0 — Pre-phase harness audit + branch check (MANDATORY, pre-code)

- **ACTION**:
  1. Confirm branch: `git branch --show-current` → must print `phase-v1-AD-c`. If not, STOP and file DQ per `.claude/rules/phase-branch.md`.
  2. Confirm base commit: `git log -1 --format=%H` → must match `f03ed1cba` (post-PR-#76 merge). If different, advisor has already updated the branch; note the new base in the task 0 report.
  3. Confirm v1-AD-b surfaces exist:
     - `rg '^pub async fn get_int_opt' crates/api/api/src/governance/config.rs` → 1 match
     - `rg '^pub async fn admin_set_config' crates/api/api/src/governance/admin_config.rs` → 1 match
     - `rg 'applied_config_snapshot' crates/db_schema/src/source/governance/moderation_case.rs` → ≥1 match on InsertForm
     - `rg 'rule_set_version_id' crates/db_schema/src/source/governance/moderation_case.rs` → ≥1 match on InsertForm
     - `rg '^pub struct RuleSetVersion' crates/db_schema/src/source/governance/rule_set_version.rs` → 1 match
     - `rg '^pub struct RuleSetVersionInsertForm' crates/db_schema/src/source/governance/rule_set_version.rs` → 1 match
  4. Run all four probes from `.claude/rules/pre-phase-harness-audit.md`:
     - Probe 1 (`-p` crate scoping): `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/audit-cargo-check-p.log 2>&1"` → expect only `lemmy_api` compiles in tail
     - Probe 2 (features activation): `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/audit-cargo-check-features.log 2>&1"`
     - Probe 3 (test target scoping): `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1"`
     - Probe 4 (negative exit-code propagation): `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1"; echo "exit: $?"` → expect **non-zero exit**
  5. DoD baseline capture — run the `--all-targets` superset to record baseline N for Task 8's delta check (NOT the production DoD gate — that is §15 Level 4 without `--all-targets`):
     - `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --all-targets --features full --no-deps -- -D warnings > .claude/audit-clippy-baseline.log 2>&1"; echo "exit: $?"` → **expect red — capture exit code and error count as baseline N for Task 8 delta check; baseline = 135 at v1-AD-c base per .claude/audit-clippy-baseline.log**
     - Also run the narrowed command §15 Level 4 uses: `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/audit-clippy-narrowed.log 2>&1"; echo "exit: $?"` → expect **exit 0** (production-code lint baseline; matches v1-AD-b ratchet from commit 3ca80e736).
  6. Review `CONFIG_KEY_METADATA` for the 7 `requires_re_jury: true` keys. If count != 7, open DQ-V1-AD-C-01 per §7.1 trigger A and pause.
- **VALIDATE**: all six points pass; if any probe or the narrowed clippy baseline fails, STOP, file DQ, do not start task 1. The `--all-targets` superset capture is expected to be red; record the count as baseline N, do not block on it.
- **COMMIT**: none. Task 0 is read-only probe discipline.
- **OUTPUT**: append `.claude/PRPs/reports/v1-AD-c-task0-audit.md` with probe results + exit codes + metadata-count confirmation + lint distribution table (rows: lint name, count, file — e.g. 56× `tests_outside_test_module`, 34× `indexing_slicing`, 27× `items_after_statements`, 5× `expect_used`, 2× `unwrap_used`, 11× misc = 135 total). The distribution table is the stable referent for the phase-close `chore(lint)` follow-up tracking issue.

### Task 1 — Fix `Scope::parse_wire` (Issue #78)

- **ACTION**: Edit `crates/api/api/src/governance/config.rs`:
  - Define `pub enum ScopeParseError { Malformed(String), NonPositiveCommunityId(i32) }` with `Debug, Clone, PartialEq, Eq, Display` (manual `Display` per §10.2 snippet)
  - Change `Scope::parse_wire` return to `Result<Self, ScopeParseError>` per §10.2
  - Update the only existing call site at `admin_config.rs:419` to wrap with `.map_err(|e| LemmyErrorType::Unknown(e.to_string()))?`
- **MIRROR**: §10.2 snippet
- **IMPLEMENT**: exactly the code in §10.2. Do NOT add `From<ScopeParseError> for LemmyError` — the wrap at the call site is sufficient and keeps LemmyErrorType surface clean.
- **GOTCHA**: the error variant is the load-bearing output. The new e2e test (task 8) calls `Scope::parse_wire` directly and asserts `result == Err(ScopeParseError::NonPositiveCommunityId(-1))` — not on wrapped-error message text. If the enum drifts (e.g. adds a `reason` field), the test's `==` comparison breaks. Keep the enum minimal.
- **GOTCHA**: the existing v1-AD-b tests `admin_set_config_scope_mismatch_rejected` (e2e.rs:4635) and others don't call `parse_wire` directly — they post invalid scope strings and check `result.is_err()`. Those keep passing because the wrap preserves wire behaviour (400 Bad Request with a readable string). Verify with `cargo test --test e2e admin_set_config_scope_` after the edit.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task1.log 2>&1"; echo "exit: $?"
  tail -20 .claude/build-task1.log
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/test-task1.log 2>&1"; echo "exit: $?"
  ```
- **COMMIT MESSAGE**: `fix(config): reject non-positive community_id in Scope::parse_wire (task 1, closes #78)`

### Task 2 — Add rule-set DTOs to `api_common`

- **ACTION**: Edit `crates/api/api_common/src/governance.rs`:
  - Add `AdminCreateRuleSet`, `AdminCreateRuleSetResponse`, `AdminListRuleSetsRequest`, `AdminListRuleSetsResponse`, `RuleSetVersionView` structs mirroring v1-AD-b DTO shapes (skip_serializing_none + ts_rs derive block)
  - Add `previous_from: Option<String>` field to existing `AdminConfigAuditEntry` (needed by task 4)
- **MIRROR**: `crates/api/api_common/src/governance.rs:364-509` (v1-AD-b DTOs)
- **IMPLEMENT (shape)**:
  ```rust
  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct AdminCreateRuleSet {
    pub community_id: CommunityId,
    pub rule_text: String,
    pub parent_id: Option<i32>,
    pub reason: String,
  }

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct AdminCreateRuleSetResponse {
    pub rule_set_version_id: i32,
    pub version: i32,
    pub config_id: Option<i64>,
    pub governance_log_id: i64,
    pub created_at: DateTime<Utc>,
  }

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct AdminListRuleSetsRequest {
    pub community_id: CommunityId,
  }

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct AdminListRuleSetsResponse {
    pub versions: Vec<RuleSetVersionView>,
    pub active_version_id: Option<i32>,
  }

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct RuleSetVersionView {
    pub id: i32,
    pub community_id: CommunityId,
    pub version: i32,
    pub parent_id: Option<i32>,
    pub text_sha256_hex: String,
    pub rule_text: String,
    pub created_at: DateTime<Utc>,
    pub created_by_pseudonym: Option<String>,
  }
  ```
- **GOTCHA**: `CommunityId` is imported from `lemmy_db_schema::newtypes`. Match the import style at line ~20 of governance.rs.
- **GOTCHA**: `text_sha256_hex` — a hex string, not `Vec<u8>`. The DB column is `BYTEA` but the wire representation is hex for readability. The view mapper in task 3 does `hex::encode`.
- **GOTCHA**: `created_by_pseudonym` — translated from `rule_set_version.created_by: Option<PersonId>` via a `LEFT JOIN actor_pseudonym` or a secondary lookup. Keep the view lookup simple (secondary query) — task 3's `admin_list_rule_sets` handler resolves this.
- **VALIDATE**: `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task2.log 2>&1"; echo "exit: $?"`
- **COMMIT MESSAGE**: `feat(api-common): rule-set DTOs + AdminConfigAuditEntry.previous_from (task 2)`

### Task 3 — `admin_create_rule_set` + `admin_list_rule_sets` handlers + `ENTRY_KIND_RULE_SET_VERSION_CREATED` const

- **ACTION**:
  1. Add `pub const ENTRY_KIND_RULE_SET_VERSION_CREATED: &str = "rule_set_version_created";` to `crates/db_schema/src/source/governance/governance_log.rs` (alphabetical between `_REPUTATION_DELTA` and `_SANCTION_CREATED`, or at end of v1-AD-b block).
  2. Add matching `pub use ... ENTRY_KIND_RULE_SET_VERSION_CREATED;` to `crates/api/api/src/governance/governance_log.rs` shim (alphabetical within the `pub use` block).
  3. Update `.claude/rules/governance-log-entry-kind-registry.md` — add a new `## v1-AD-c entry kinds (1, this sub-phase)` section with the single row, update the Acceptance invariants count line from 25/27 → 26/28 (verify current count by running the `rg -c` command in the rule file).
  4. Create `crates/api/api/src/governance/admin_rule_sets.rs` with both handlers + private helpers (virtual-metadata synthesiser, text-sha256 hasher, version-computer, view-mapper).
  5. Register `pub mod admin_rule_sets;` in `crates/api/api/src/governance/mod.rs`.
- **MIRROR**: §10.4 for the three-write tx, §10.1 for the capability gate, `admin_config.rs:1155-1218` for the list pagination shape (though rule-set list is simpler — no pagination v1, just "all versions for this community ORDER BY version DESC"), §10.2 for scope parsing where needed.
- **IMPLEMENT**:
  ```rust
  // crates/api/api/src/governance/admin_rule_sets.rs skeleton
  use actix_web::web::Json;
  use chrono::{DateTime, Utc};
  use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
  use diesel_async::{AsyncPgConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
  use lemmy_api_common::{
    context::LemmyContext,
    governance::{
      AdminCreateRuleSet, AdminCreateRuleSetResponse,
      AdminListRuleSetsRequest, AdminListRuleSetsResponse, RuleSetVersionView,
    },
  };
  use lemmy_db_schema::{
    newtypes::RuleSetVersionId,
    source::governance::{
      governance_config::{GovernanceConfig, GovernanceConfigInsertForm},
      governance_log::ENTRY_KIND_RULE_SET_VERSION_CREATED,
      rule_set_version::{RuleSetVersion, RuleSetVersionInsertForm},
    },
    utils::{DbPool, get_conn},
  };
  use lemmy_db_schema_file::schema::{governance_config, rule_set_version};
  use lemmy_db_views::{local_user::LocalUserView, community_moderator::CommunityModeratorView};
  use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult};
  use serde_json::json;
  use sha2::{Digest, Sha256};

  use crate::governance::{
    actor_pseudonym_helper,
    admin_config::{DenialReason, emit_denial_log_adapted},  // may need small adapter
    config::{self, Scope},
    governance_log,
  };

  pub async fn admin_create_rule_set(
    data: Json<AdminCreateRuleSet>,
    context: actix_web::web::Data<LemmyContext>,
    local_user_view: LocalUserView,
  ) -> LemmyResult<Json<AdminCreateRuleSetResponse>> {
    let data = data.into_inner();

    // 1. Capability gate — community_moderator(community_id) OR is_admin
    let is_mod = CommunityModeratorView::check_is_community_moderator(
      &mut context.pool(),
      data.community_id,
      local_user_view.person.id,
    ).await.is_ok();
    let is_admin_ok = lemmy_api_utils::utils::is_admin(&local_user_view).is_ok();
    if !(is_mod || is_admin_ok) {
      emit_rule_set_denial_log(
        &mut context.pool(),
        local_user_view.person.id,
        data.community_id,
        &data.reason,
        "community_moderator_required",
      ).await?;
      return Err(LemmyErrorType::NotAnAdmin.into());
    }

    // 2. Validate rule_text length against instance config
    let mut cache = config::ConfigCache::new();
    let max_bytes = config::get_int_opt(
      &mut cache,
      &mut context.pool(),
      Scope::Instance,
      "rule_set.text_max_bytes",
    ).await?.unwrap_or(65_536);
    if i64::try_from(data.rule_text.len()).unwrap_or(i64::MAX) > max_bytes {
      return Err(LemmyErrorType::Unknown(format!(
        "rule_text exceeds rule_set.text_max_bytes = {max_bytes} bytes"
      )).into());
    }
    if data.rule_text.is_empty() {
      return Err(LemmyErrorType::Unknown("rule_text cannot be empty".to_string()).into());
    }

    // 3. Compute version + text_sha256 — pre-tx (no DB state mutation yet)
    let previous_version = lookup_latest_version(&mut context.pool(), data.community_id).await?;
    let new_version = previous_version.map_or(1, |v| v + 1);
    let text_sha256 = Sha256::digest(data.rule_text.as_bytes()).to_vec();

    // 4. Pseudonym
    let admin_pseudonym = actor_pseudonym_helper::get_or_create(
      &mut context.pool(),
      local_user_view.person.id,
    ).await?;

    // 5. Three-row atomic write
    let conn = &mut get_conn(&mut context.pool()).await?;
    let admin_id = local_user_view.person.id;

    let (rsv_row, cfg_id, log_id, created_at) = conn.run_transaction(|conn| {
      let pseudonym = admin_pseudonym.clone();
      let rule_text = data.rule_text.clone();
      let parent_id = data.parent_id.map(RuleSetVersionId);
      let community_id = data.community_id;
      let text_sha256 = text_sha256.clone();
      async move {
        process_create_rule_set(
          conn, admin_id, pseudonym, community_id,
          new_version, parent_id, text_sha256, rule_text,
        ).await
      }.scope_boxed()
    }).await?;

    Ok(Json(AdminCreateRuleSetResponse {
      rule_set_version_id: rsv_row.id.0,
      version: rsv_row.version,
      config_id: Some(cfg_id.into()),
      governance_log_id: log_id,
      created_at,
    }))
  }

  async fn process_create_rule_set(
    conn: &mut AsyncPgConnection,
    admin_id: lemmy_db_schema_file::PersonId,
    admin_pseudonym: String,
    community_id: lemmy_db_schema::newtypes::CommunityId,
    new_version: i32,
    parent_id: Option<RuleSetVersionId>,
    text_sha256: Vec<u8>,
    rule_text: String,
  ) -> LemmyResult<(RuleSetVersion, i32, i64, DateTime<Utc>)> {
    // Write 1 — rule_set_version
    let rsv_form = RuleSetVersionInsertForm {
      community_id,
      version: new_version,
      parent_id,
      text_sha256: text_sha256.clone(),
      rule_text,
      created_by: Some(admin_id),
    };
    let rsv: RuleSetVersion = diesel::insert_into(rule_set_version::table)
      .values(&rsv_form)
      .returning(RuleSetVersion::as_returning())
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Unknown("rule_set_version insert failed".into()))?;

    // Write 2 — governance_config row flipping active_version_id
    let cfg_form = GovernanceConfigInsertForm {
      scope: Scope::Community(community_id).as_str().into_owned(),
      key: "rule_set.active_version_id".to_string(),
      value_type: "int".to_string(),
      value_int: Some(i64::from(rsv.id.0)),
      value_float: None,
      value_bool: None,
      value_text: None,
      updated_by: Some(admin_id),
    };
    let cfg: GovernanceConfig = diesel::insert_into(governance_config::table)
      .values(&cfg_form)
      .returning(GovernanceConfig::as_returning())
      .get_result(conn)
      .await?;

    // Write 3 — governance_log
    let payload = json!({
      "community_id":          community_id.0,
      "version":               rsv.version,
      "parent_id":             rsv.parent_id.map(|p| p.0),
      "text_sha256":           hex::encode(&text_sha256),
      "rule_set_version_id":   rsv.id.0,
      "config_id":             cfg.id.0,
      "activated_at":          rsv.created_at,
    });
    let log_row = governance_log::append(
      &mut (&mut *conn).into(),
      ENTRY_KIND_RULE_SET_VERSION_CREATED,
      payload,
      Some(admin_pseudonym),
    ).await?;

    let created_at = rsv.created_at;
    Ok((rsv, cfg.id.0, log_row.id.0, created_at))
  }

  pub async fn admin_list_rule_sets(
    data: actix_web::web::Query<AdminListRuleSetsRequest>,
    context: actix_web::web::Data<LemmyContext>,
    local_user_view: LocalUserView,
  ) -> LemmyResult<Json<AdminListRuleSetsResponse>> {
    let data = data.into_inner();

    // Capability — moderator OR admin
    let is_mod = CommunityModeratorView::check_is_community_moderator(
      &mut context.pool(),
      data.community_id,
      local_user_view.person.id,
    ).await.is_ok();
    let is_admin_ok = lemmy_api_utils::utils::is_admin(&local_user_view).is_ok();
    if !(is_mod || is_admin_ok) {
      return Err(LemmyErrorType::NotAnAdmin.into());
    }

    let conn = &mut get_conn(&mut context.pool()).await?;

    let rows: Vec<RuleSetVersion> = rule_set_version::table
      .filter(rule_set_version::community_id.eq(data.community_id))
      .order(rule_set_version::version.desc())
      .select(RuleSetVersion::as_select())
      .load(conn).await?;

    let mut cache = config::ConfigCache::new();
    let active_version_id = config::get_int_opt(
      &mut cache,
      &mut context.pool(),
      Scope::Community(data.community_id),
      "rule_set.active_version_id",
    ).await?.and_then(|i| i32::try_from(i).ok());

    let versions = rows.into_iter().map(|rsv| RuleSetVersionView {
      id: rsv.id.0,
      community_id: rsv.community_id,
      version: rsv.version,
      parent_id: rsv.parent_id.map(|p| p.0),
      text_sha256_hex: hex::encode(&rsv.text_sha256),
      rule_text: rsv.rule_text,
      created_at: rsv.created_at,
      created_by_pseudonym: None,  // v1-AD-c: leave None; pseudonym resolution is v1-AD-d
    }).collect();

    Ok(Json(AdminListRuleSetsResponse { versions, active_version_id }))
  }

  async fn lookup_latest_version(
    pool: &mut DbPool<'_>,
    community_id: lemmy_db_schema::newtypes::CommunityId,
  ) -> LemmyResult<Option<i32>> {
    let conn = &mut get_conn(pool).await?;
    let v: Option<i32> = rule_set_version::table
      .filter(rule_set_version::community_id.eq(community_id))
      .order(rule_set_version::version.desc())
      .select(rule_set_version::version)
      .first(conn).await.optional()?;
    Ok(v)
  }

  async fn emit_rule_set_denial_log(
    pool: &mut DbPool<'_>,
    admin_id: lemmy_db_schema_file::PersonId,
    community_id: lemmy_db_schema::newtypes::CommunityId,
    reason: &str,
    denial_reason: &str,
  ) -> LemmyResult<()> {
    let pseudonym = actor_pseudonym_helper::get_or_create(pool, admin_id).await?;
    let payload = json!({
      "scope":         Scope::Community(community_id).as_str(),
      "key":           "rule_set.active_version_id",
      "value_type":    "int",
      "value":         null,
      "reason":        reason,
      "denial_reason": denial_reason,
    });
    governance_log::append(
      pool,
      governance_log::ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
      payload,
      Some(pseudonym),
    ).await?;
    Ok(())
  }
  ```
- **GOTCHA (race on UNIQUE constraint)**: two admins racing on `version = prev + 1` both see the same `prev` pre-tx. Second tx's INSERT fails with a Postgres unique-violation on `(community_id, version)`. Map this to `LemmyErrorType::Unknown("rule-set version already exists — retry")` OR catch `DatabaseError(UniqueViolation, _)` explicitly. Add an e2e test `admin_create_rule_set_duplicate_version_rejected` (task 8) that triggers the race by inserting the same version twice.
- **GOTCHA (rule_text encoding)**: `Sha256::digest(data.rule_text.as_bytes())` hashes UTF-8 bytes. The DB column is `BYTEA`. Wire representation is `hex::encode(...)`. Never base64 — inconsistent with Phase 6 federation hex convention.
- **GOTCHA (EmergencyRemove exhaustive match)**: neither handler matches on `CaseStatus`, but verify no newly-added match on `CaseStatus` drops `EmergencyRemove` per ADR-013. `rg 'CaseStatus::' crates/api/api/src/governance/admin_rule_sets.rs` must return zero matches.
- **GOTCHA (parent_id validation)**: if `parent_id` is provided, validate it exists AND belongs to the same community. Add a pre-tx check:
  ```rust
  if let Some(pid) = data.parent_id {
    let parent_community: Option<i32> = rule_set_version::table
      .filter(rule_set_version::id.eq(RuleSetVersionId(pid)))
      .select(rule_set_version::community_id.assume_not_null())
      .first::<i32>(conn).await.optional()?;
    match parent_community {
      Some(cid) if cid == data.community_id.0 => (),
      Some(_) => return Err(LemmyErrorType::Unknown("parent_id belongs to a different community".into()).into()),
      None => return Err(LemmyErrorType::Unknown(format!("parent_id {pid} not found")).into()),
    }
  }
  ```
- **GOTCHA (`check_is_community_moderator` signature)**: confirm the exact signature at `crates/db_views/community_moderator/src/impls.rs` before writing the call — v1-AD-b's drift-notes flag drift potential. If the signature differs from the snippet above, adapt.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/build-task3a.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task3b.log 2>&1"; echo "exit: $?"
  # governance-log-entry-kind-registry invariant:
  rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
  # Expected: 26 (was 25 pre-task-3)
  rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d
  # Expected: empty (no duplicate literals)
  rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l
  # Expected: equal to 26 (shim re-export parity)
  ```
- **COMMIT MESSAGE**: `feat(admin-rule-sets): admin_create_rule_set + admin_list_rule_sets handlers + ENTRY_KIND_RULE_SET_VERSION_CREATED (task 3)`

### Task 4 — Issue #77 refactor: thread pre-tx provenance through admin_config write + audit projection

- **ACTION**: Edit `crates/api/api/src/governance/admin_config.rs`:
  1. Extract `fn build_admin_config_changed_payload(row: &GovernanceConfig, data: &AdminSetConfig, previous: &ConfigValueWithProvenance) -> serde_json::Value` per §10.3
  2. Modify `process_set_config` signature to add `previous: ConfigValueWithProvenance` param; replace the inline `json!` at lines 585-591 with `build_admin_config_changed_payload(&row, &data, &previous)`
  3. Update the call site in `admin_set_config` at line 520-535: pass `preview.previous.clone()` (the pre-tx tuple already computed at line 477 and packed at line 484-494) into `process_set_config`
  4. Modify `project_to_audit_entry` at lines 1284-1330: hydrate `previous_value` from `payload.get("previous_value")` and `previous_from` from `payload.get("previous_from")`
- **MIRROR**: §10.3 snippet
- **GOTCHA**: the HTTP response `preview.previous` is the SAME data the payload's `previous_value` + `previous_from` fields now hold. Double-source-of-truth risk: if someone later changes the preview shape, the payload may drift. Add a unit test `build_admin_config_changed_payload_matches_preview_previous` that constructs a `ConfigChangePreview` and asserts the payload's two fields equal the preview's previous fields.
- **GOTCHA (test-impact of the DTO change)**: adding `previous_from: Option<String>` to `AdminConfigAuditEntry` in task 2 means every construction site of that struct in v1-AD-b code needs a new field. Grep for `AdminConfigAuditEntry {` before committing; expected hit count is ~2 (the `project_to_audit_entry` call at line 1316 and any test helpers). Update each.
- **GOTCHA (backward-compat)**: shell-written rows or any pre-task-4 HTTP-written rows will have NO `previous_value`/`previous_from` payload keys. `project_to_audit_entry` must degrade gracefully — the snippet at §10.3 does exactly this via `.get(...).cloned()` and `.and_then(|v| v.as_str()).map(str::to_owned)`.
- **GOTCHA (previous_from as String)**: the existing `ConfigValueWithProvenance.effective_from` is `String` (e.g. `"instance"`, `"community:42"`, `"default"`). Store as-is in the payload — do NOT re-parse through `Scope`. The audit-entry `previous_from` field is `Option<String>` to preserve this raw shape (including `"default"` which has no `Scope` counterpart).
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task4.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/test-task4.log 2>&1"; echo "exit: $?"
  # Re-run v1-AD-b's payload-parity unit tests to confirm no regression:
  cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_api --features full payload_parity > .claude/parity-task4.log 2>&1"; echo "exit: $?"
  ```
- **COMMIT MESSAGE**: `refactor(admin-config): thread pre-tx provenance into audit payload (task 4, closes #77)`

### Task 5 — Case-open snapshot pin in `create_report.rs`

- **ACTION**:
  1. Create `crates/api/api/src/governance/case_open_snapshot.rs` with `build_applied_config_snapshot(pool, scope) -> LemmyResult<serde_json::Value>` that reads the 7 `requires_re_jury` keys at the provided scope and returns a JSON object with one field per key.
  2. Register `pub mod case_open_snapshot;` in `crates/api/api/src/governance/mod.rs`.
  3. Edit `crates/api/api_crud/src/governance/create_report.rs`:
     - Before the existing `ModerationCaseInsertForm { … }` construction at ~178–192, read `rule_set.active_version_id` via `config::get_int_opt` at the appropriate scope (Community if `data.community_id.is_some()`, else Instance)
     - Call `build_applied_config_snapshot(pool, scope)` to get the JSONB snapshot
     - Populate both `applied_config_snapshot: Some(snapshot_json)` and `rule_set_version_id: active_version_id_opt.and_then(|i| i32::try_from(i).ok()).map(RuleSetVersionId)` on the insert form
- **MIRROR**: `admin_config.rs` `get_*_opt` call pattern; InsertForm shape at `moderation_case.rs:48-68`
- **IMPLEMENT (snapshot helper)**:
  ```rust
  // crates/api/api/src/governance/case_open_snapshot.rs
  use lemmy_db_schema::utils::DbPool;
  use lemmy_utils::error::LemmyResult;
  use serde_json::{Value, json};

  use crate::governance::config::{self, ConfigCache, Scope};

  /// The 7 keys with `requires_re_jury: true`. Snapshotted at case-open to
  /// grandfather in-flight juries against later config edits (ADR-010 invariant).
  /// Kept as a module-level constant so the parity test can verify the count.
  pub(crate) const REQUIRES_RE_JURY_KEYS: &[&str] = &[
    "jury.panel_size",
    "jury.quorum",
    "jury.severity_thresholds.minor",
    "jury.severity_thresholds.moderate",
    "jury.severity_thresholds.severe",
    "jury.diversity_constraints_enabled",
    "jury.appeal_panel_size_increase",
  ];

  pub async fn build_applied_config_snapshot(
    pool: &mut DbPool<'_>,
    scope: Scope,
  ) -> LemmyResult<Value> {
    let mut cache = ConfigCache::new();
    let panel_size = config::get_int(&mut cache, pool, scope, "jury.panel_size").await?;
    let quorum = config::get_int(&mut cache, pool, scope, "jury.quorum").await?;
    let sev_minor = config::get_text(&mut cache, pool, scope, "jury.severity_thresholds.minor").await?;
    let sev_moderate = config::get_text(&mut cache, pool, scope, "jury.severity_thresholds.moderate").await?;
    let sev_severe = config::get_text(&mut cache, pool, scope, "jury.severity_thresholds.severe").await?;
    let diversity_enabled = config::get_bool(&mut cache, pool, scope, "jury.diversity_constraints_enabled").await?;
    let appeal_increase = config::get_int(&mut cache, pool, scope, "jury.appeal_panel_size_increase").await?;

    Ok(json!({
      "jury.panel_size":                    panel_size,
      "jury.quorum":                        quorum,
      "jury.severity_thresholds.minor":     sev_minor,
      "jury.severity_thresholds.moderate":  sev_moderate,
      "jury.severity_thresholds.severe":    sev_severe,
      "jury.diversity_constraints_enabled": diversity_enabled,
      "jury.appeal_panel_size_increase":    appeal_increase,
    }))
  }

  #[cfg(test)]
  mod parity {
    use super::REQUIRES_RE_JURY_KEYS;
    use crate::governance::config::CONFIG_KEY_METADATA;

    /// The snapshot keys must equal the set of keys in CONFIG_KEY_METADATA
    /// where requires_re_jury == true. If the metadata adds a new requires_re_jury
    /// key without the snapshot helper being updated, this test fails and
    /// prevents the pin from drifting.
    #[test]
    fn snapshot_keyset_matches_requires_re_jury_metadata() {
      let metadata_keys: Vec<&str> = CONFIG_KEY_METADATA
        .iter()
        .filter(|m| m.requires_re_jury)
        .map(|m| m.key)
        .collect();
      let mut a: Vec<&str> = REQUIRES_RE_JURY_KEYS.to_vec();
      let mut b = metadata_keys;
      a.sort();
      b.sort();
      assert_eq!(a, b, "requires_re_jury metadata keys drifted from REQUIRES_RE_JURY_KEYS");
    }
  }
  ```
- **GOTCHA**: `ApplyAt::NextJuryCycle` is the metadata default for all 7 keys. The snapshot is the READ-SIDE half of the NextJuryCycle semantics — a jury-seating site reading from the snapshot for in-flight cases, and from live config for cases-with-NULL-snapshot (pre-v1-AD-c). Per §4.1, v1-AD-c is write-only for the pin; jury-mechanics-v1 owns the read-side switch.
- **GOTCHA**: `get_int` returns `i64`, but the cache key is string-typed. The snapshot JSON stores as `i64` — consumers reading it (jury-mechanics-v1) will need `.as_i64()`.
- **GOTCHA**: two of the 7 keys are `ValueType::Enum` (stored as TEXT). `config::get_text` handles the read — do NOT try to deserialise via `serde_json`. The snapshot stores the raw string (e.g. `"majority"`, `"60%"`, `"75%"`).
- **GOTCHA**: scope selection for `rule_set.active_version_id` mirrors the case's community:
  ```rust
  let case_scope = match data.community_id {
    Some(cid) => Scope::Community(cid),
    None => Scope::Instance,
  };
  let snapshot = case_open_snapshot::build_applied_config_snapshot(&mut context.pool(), case_scope).await?;
  let active_version_i64 = config::get_int_opt(
    &mut ConfigCache::new(),
    &mut context.pool(),
    case_scope,
    "rule_set.active_version_id",
  ).await?;
  let rule_set_version_id = active_version_i64
    .and_then(|i| i32::try_from(i).ok())
    .map(RuleSetVersionId);
  ```
- **GOTCHA**: verify no OTHER `ModerationCaseInsertForm` construction site drops the pin columns unintentionally. Grep:
  - `rg 'ModerationCaseInsertForm {' crates/` → expect ~2 hits: `create_report.rs` (this task's edit site) and `admin_emergency_remove.rs` (emergency-remove path).
  - The emergency-remove path per §4.1 is **NOT** updated in v1-AD-c — emergency removes are admin-override cases that pin the active config at removal time, not at case-open. Leaving both as `None` via `..Default::default()` is correct. Document this decision in the task-5 commit body.
- **GOTCHA**: EmergencyRemove exhaustive match — `create_report.rs` already handles CaseStatus variants; the new code paths don't touch CaseStatus. Verify with `rg 'CaseStatus::' crates/api/api_crud/src/governance/create_report.rs`.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task5a.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task5b.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_api --features full case_open_snapshot > .claude/test-task5.log 2>&1"; echo "exit: $?"
  ```
- **COMMIT MESSAGE**: `feat(case-open): pin applied_config_snapshot + rule_set_version_id (task 5)`

### Task 6 — Registry file update (governance-log-entry-kind-registry)

- **ACTION**: Edit `.claude/rules/governance-log-entry-kind-registry.md`:
  - Add `## v1-AD-c entry kinds (1, this sub-phase)` section between the v1-AD-a section and `## v1 PRD reservation sections` block
  - Add a row with `ENTRY_KIND_RULE_SET_VERSION_CREATED`, `rule_set_version_created`, v1-AD-c, emitting handler path, semantic description
  - Update the `Acceptance invariants` first bullet to reflect 26 total (19 v0 + 4 Phase 6 + 2 v1-AD-a + 1 v1-AD-c). Old text said 25 at v1-AD-a end; new text says 26 at v1-AD-c end.
- **MIRROR**: the existing v1-AD-a subsection format — table + emitting-handler column + semantic column
- **GOTCHA**: this file is **mandatory reading in `-p` mode** per the rule's own preamble. If the acceptance count is wrong after task 6, every future `/prp-plan` run during v1-AD-c's ralph loop will read stale guidance. Double-check the invariant commands pass with the new count BEFORE committing.
- **VALIDATE**:
  ```bash
  # Follow the exact invariants commands in the rule file:
  rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
  # Expected: 26
  rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort -u | wc -l
  # Expected: 26 (unique literals)
  rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d
  # Expected: empty (no duplicates)
  rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l
  # Expected: 26 (shim re-export parity)
  ```
- **COMMIT MESSAGE**: `docs(rules): register ENTRY_KIND_RULE_SET_VERSION_CREATED in entry-kind registry (task 6)`

### Task 7 — Wire routes in `crates/api/routes/src/lib.rs`

- **ACTION**: Add `/admin/rule-sets` scope to the existing `/admin` scope block at lines 531-546. Add POST + GET routes alongside the existing `admin_set_config`, `admin_get_config`, `admin_get_config_audit` registrations.
- **MIRROR**: `crates/api/routes/src/lib.rs:539-541` (v1-AD-b `/admin/config` block)
- **IMPLEMENT**:
  ```rust
  // In the existing /admin scope block:
  .service(
    scope("/rule-sets")
      .route("", post().to(admin_create_rule_set))
      .route("", get().to(admin_list_rule_sets)),
  )
  ```
- **GOTCHA**: `admin_list_rule_sets` takes `actix_web::web::Query` not `Json` — Actix handles the binding difference automatically but imports at the top of the file may need `use actix_web::web::Query` added (check v1-AD-b admin_get_config — it also uses Query, so the import is likely already present).
- **GOTCHA**: `rate_limit.post()` middleware: the outer `/governance/admin` scope already applies the middleware. The inner `/rule-sets` scope inherits it — no explicit middleware annotation needed.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task7.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/build-task7-workspace.log 2>&1"; echo "exit: $?"
  ```
- **COMMIT MESSAGE**: `feat(routes): wire /admin/rule-sets POST + GET (task 7)`

### Task 8 — e2e tests (8 new tests)

- **ACTION**: Add 8 tests to `crates/server/tests/e2e.rs`, grouped by concern:

**Group A — Rule-set CRUD (4 tests)**:
1. `admin_create_rule_set_happy_path` — moderator creates v1, then v2 with parent_id = v1; assert rule_set_version count, active_version_id config flip, governance_log entry count == 2 (one per creation).
2. `admin_create_rule_set_non_moderator_rejected` — non-moderator attempts; assert 403 + `admin_config_change_denied` entry with `denial_reason = "community_moderator_required"`.
3. `admin_create_rule_set_duplicate_version_rejected` — race two creates against same parent_id; one succeeds, one fails with unique-violation mapped to 400/500 (document which). Count rows to prove atomicity.
4. `admin_list_rule_sets_returns_versions_with_active_version_id` — after 3 creates, GET returns 3 versions ordered by version DESC, active_version_id = 3.

**Group B — Issue #78 Scope parser (1 test)**:
5. `scope_parse_wire_rejects_negative_community_id` — direct call `Scope::parse_wire("community:-1")`; assert `Err(ScopeParseError::NonPositiveCommunityId(-1))`. Second assertion: `Scope::parse_wire("community:0")` → same error variant with `0`.

**Group C — Issue #77 audit payload (2 tests)**:
6. `admin_set_config_persists_previous_value_and_from` — POST /admin/config to change `jury.panel_size` from default 5 to 7; query governance_log row; assert payload has `previous_value: 5, previous_from: "default"`. Second write 7 → 9; assert `previous_value: 7, previous_from: "instance"`.
7. `admin_get_config_audit_hydrates_previous_value` — same setup as #6; hit GET /admin/config/audit; assert response entry has `previous_value: Some(Value::Int(5)), previous_from: Some("default")`.

**Group D — Case-open snapshot (1 test)**:
8. `case_open_pins_applied_config_snapshot_and_rule_set_version_id` — set up community, create rule-set v1 (admin path), POST /governance/report against the community, query moderation_case row; assert `applied_config_snapshot` has all 7 `requires_re_jury` keys with matching values, `rule_set_version_id == Some(v1.id)`.

- **MIRROR**: `e2e.rs:4330-4380` (happy-path), `e2e.rs:4579-4629` (denial-path), `e2e.rs:4635-4710` (scope-mismatch)
- **GOTCHA (fixtures)**: v1-AD-b's `admin_config_fixtures::seed_user` creates a LocalUser + Person but NOT a CommunityModerator. Group A tests need either:
  - a new `admin_config_fixtures::seed_community_moderator(context, instance_id, community_id, name) -> LocalUserView` helper, OR
  - the test seeds `CommunityModerator` rows inline via Diesel
  
  Prefer adding the helper to the existing fixtures module (v1-AD-d will need it too). Document in task-8 commit.
- **GOTCHA (Docker flake)**: e2e tests spin up a real Postgres container per test. 8 new tests + 13 v1-AD-b existing = 21 containers in sequence. Run time: ~5-8 min locally, ~10-15 min in CI. If CI times out, batch-run with `--test-threads=4` or mark the three heaviest as `#[ignore]` and gate in the PR body.
- **GOTCHA (Race test #3)**: true race via tokio::join! requires two connections to the pool simultaneously. Simpler approach: run sequentially — create v1 twice with the same `parent_id = None` forcing `version = 1` on both; the second must fail. This is NOT a race but it exercises the UNIQUE constraint which is what task 3's GOTCHA requires.
- **GOTCHA (previous_from string values)**: `"default"` is the label for "no DB row, const fallback". `"instance"` and `"community:<id>"` are the wire-identifiers for DB-row scopes. Test #6 asserts on these exact strings. If the `read_effective` implementation changes the label format, test #6 breaks — that's desirable signal.
- **GOTCHA (snapshot keyset changes)**: if `CONFIG_KEY_METADATA` adds a new `requires_re_jury: true` key, the unit test `snapshot_keyset_matches_requires_re_jury_metadata` in task 5 fails BEFORE the e2e test runs, making it clear the snapshot helper needs updating. If somehow a key is added without updating the snapshot AND the unit test is bypassed, test #8 fails because the asserted keyset (the 7 keys named in the test body) won't match the actual snapshot keyset — also a desirable red.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/build-task8.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_create_rule_set > .claude/test-task8a.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server scope_parse_wire_rejects_negative_community_id > .claude/test-task8b.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_set_config_persists_previous > .claude/test-task8c.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_get_config_audit_hydrates > .claude/test-task8d.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server case_open_pins > .claude/test-task8e.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_list_rule_sets > .claude/test-task8f.log 2>&1"; echo "exit: $?"

  # Advisory (NOT a DoD gate): --all-targets clippy delta vs Task 0 baseline (135 errors).
  # Acceptance: error count <= 135. Net-new errors introduced by task 8's 8 new tests block the commit until fixed.
  # Net-zero delta is fine. File a tracking issue at phase-close per the runlog decision.
  cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --all-targets --features full --no-deps -- -D warnings > .claude/build-task8-clippy-alltargets.log 2>&1"; echo "exit: $?"
  grep -cE "^error: " .claude/build-task8-clippy-alltargets.log
  ```
- **COMMIT MESSAGE**: `test(v1-AD-c): 8 e2e tests for rule-sets + #77 + #78 + case-open pin (task 8)`

---

## 14. Testing strategy

Per [IMPLEMENTATION-PLAN-v0.md §5](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): **integration-only**. All new tests in `crates/server/tests/e2e.rs`. One exception: the task-5 `case_open_snapshot::parity::snapshot_keyset_matches_requires_re_jury_metadata` unit test lives in the module under `#[cfg(test)]` because it's a compile-time-driven parity check (no DB needed); this follows the v1-AD-b pattern (`admin_config::payload_parity`).

### 14.1 Tests to add

| Test Name | What It Validates | Group |
|---|---|---|
| `admin_create_rule_set_happy_path` | Two sequential creates with parent chain; rule_set_version count, active_version_id flip, 2 governance_log entries | A |
| `admin_create_rule_set_non_moderator_rejected` | 403 + `admin_config_change_denied` with `community_moderator_required` | A |
| `admin_create_rule_set_duplicate_version_rejected` | UNIQUE(community_id, version) constraint enforced at handler layer | A |
| `admin_list_rule_sets_returns_versions_with_active_version_id` | GET returns all versions ordered DESC; `active_version_id` correct | A |
| `scope_parse_wire_rejects_negative_community_id` | Typed error variant on `community:-1` and `community:0` | B |
| `admin_set_config_persists_previous_value_and_from` | Log payload has `previous_value` + `previous_from` (Issue #77 write side) | C |
| `admin_get_config_audit_hydrates_previous_value` | Audit GET response has `previous_value` + `previous_from` (Issue #77 read side) | C |
| `case_open_pins_applied_config_snapshot_and_rule_set_version_id` | Case opens with populated snapshot + rule-set FK | D |

### 14.2 Edge cases covered

- [x] Hash-chain integrity still holds after new `rule_set_version_created` entries (task 8 test A1 implicitly tests via governance_log row count)
- [x] `actor_pseudonym::get_or_create` runs for non-moderator denied users (task 8 test A2 asserts pseudonym on denial row — same pattern as v1-AD-b `admin_set_config_non_admin_rejected_with_denial_log`)
- [x] `EmergencyRemove` branch exhaustively matched — no new `CaseStatus` match in v1-AD-c code (task 3 + task 5 grep checks)
- [x] Redaction — not applicable; no free-text fields hit the public log; rule_text is stored but governance_log payload only carries `text_sha256` hex + scalar metadata
- [x] UNIQUE constraint on `(community_id, version)` — task 8 test A3
- [x] Absent-key `rule_set.active_version_id` returns `Ok(None)` — task 8 test A4 (community with no rule-sets yet) asserts `active_version_id == None`
- [x] Snapshot keyset matches `requires_re_jury` metadata — task 5 unit test + task 8 test D1 consistency
- [x] Shell-written audit rows lack `previous_value`/`previous_from` and are handled as `None` — task 4 GOTCHA + task 8 test C2 variant covers (by hitting audit with mixed post-task-4 + pre-task-4 log rows)

### 14.3 Not tested (out of v1-AD-c scope)

- Read-side of `applied_config_snapshot` — jury-mechanics-v1
- Read-side of `rule_set_version_id` at case-decision display — public-case-log extensions, separate PRP
- Step-up auth enforcement on rule-set POST — v2
- Rule-set federation — v2/v3 (fork-only AP type)
- Rule-set text diff — v1.x if pilot asks

---

## 15. Validation commands (DoD)

Use these EXACT commands. Do NOT substitute `npm`/`pnpm`/etc. This is a Rust project. Per `.claude/rules/cargo-output-capture.md`, always capture to file first; never pipe through `tail`/`head`/`grep` and trust the exit code.

### Level 1: STATIC ANALYSIS (after every task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p <changed-crate> --features full > .claude/build-taskNN.log 2>&1"; echo "exit: $?"
tail -20 .claude/build-taskNN.log
```

**EXPECT**: exit 0, zero errors.

### Level 2: WORKSPACE CHECK (after tasks 3, 4, 5, 7 — handler/refactor additions)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/workspace-taskNN.log 2>&1"; echo "exit: $?"
tail -20 .claude/workspace-taskNN.log
```

**EXPECT**: exit 0. The `--workspace --features full` combo is the only reliable way to flip `full` per `feedback_features_full_workspace_only.md`.

### Level 3: TEST-TARGET COMPILE GATE (after every task that touches `lemmy_api`, `lemmy_api_common`, or `lemmy_server`)

Per `feedback_test_target_compile_validation.md`:

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/test-compile-taskNN.log 2>&1"; echo "exit: $?"
tail -20 .claude/test-compile-taskNN.log
```

**EXPECT**: exit 0. This catches test-surface breakage early even if the source code compiles.

### Level 4: CLIPPY (after tasks 1, 3, 4, 5)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/clippy-taskNN.log 2>&1"; echo "exit: $?"
tail -40 .claude/clippy-taskNN.log
```

**EXPECT**: exit 0, zero warnings. `--no-deps` prevents upstream lint debt from leaking in.

Test-target debt is tracked separately per Task 0 audit report; v1-AD-c does not gate on `--all-targets` to avoid blocking on pre-existing inherited debt unrelated to the sub-phase scope. Task 8 captures `--all-targets` delta vs baseline N=135 as advisory.

### Level 5: INTEGRATION TESTS (task 8)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/e2e-task8.log 2>&1"; echo "exit: $?"
tail -60 .claude/e2e-task8.log
```

**EXPECT**: all 8 new tests pass. If Docker flakes, retry once. If the retry also fails, surface the specific failure + log path in the task-8 commit body and mark the failing test `#[ignore]` with a TODO pointing at the GH issue the retry will become.

### Level 6: COLD-REBUILD GATE (pre-PR close only)

Per `feedback_cold_build_gate_layered_agents.md`:

```bash
cargo clean
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/cold-rebuild.log 2>&1"; echo "exit: $?"
```

**EXPECT**: exit 0 on first try. If this is red but warm builds are green, a target-cache artifact is hiding a real compile break.

### Level 7: GOVERNANCE-LOG REGISTRY INVARIANT (after task 3)

```bash
rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
# Expected: 26 (was 25 pre-v1-AD-c)

rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort -u | wc -l
# Expected: 26

rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d
# Expected: empty

rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l
# Expected: 26 (shim re-export parity)
```

---

## 16. Acceptance criteria

- [ ] All 9 tasks (0–8) completed in dependency order
- [ ] `cargo check --workspace --features full` exits 0
- [ ] `cargo clippy --workspace --features full --no-deps -- -D warnings` exits 0
  - Task 8 captures `--all-targets` delta vs baseline 135; net-new errors block the task-8 commit.
- [ ] `cargo test --test e2e -p lemmy_server` passes — all 21 tests (13 v1-AD-b + 8 v1-AD-c) green
- [ ] Cold rebuild (`cargo clean` + workspace check) exits 0 on first try
- [ ] Registry invariant command block returns expected counts (26 entry kinds; no duplicate literals; shim parity)
- [ ] Issue #77 closed: audit entries written after task-4 have non-null `previous_value` + `previous_from`; audit GET hydrates both
- [ ] Issue #78 closed: `Scope::parse_wire("community:-1")` returns `Err(ScopeParseError::NonPositiveCommunityId(-1))`; e2e test enforces
- [ ] `moderation_case.applied_config_snapshot` populated on every new case (null for pre-existing rows; this is expected)
- [ ] `moderation_case.rule_set_version_id` populated when community has an active rule-set at case-open time
- [ ] `rule_set_version` gains at least one row during the e2e run (task 8 test A1)
- [ ] No new `cargo clippy` warnings introduced
- [ ] No contradictions with the 15 ADRs (ADR-008 log every write; ADR-010 append-only; ADR-013 exhaustive match; ADR-015 pseudonyms)
- [ ] PRD §3.6 OQ-002 resolution is now testable end-to-end

---

## 17. Completion checklist

- [ ] Task 0: pre-phase harness audit passed; audit-cargo-check-p/-features/-test/-negative all show expected exit codes; clippy baseline exit 0
- [ ] Task 1: `Scope::parse_wire` returns `Result<Scope, ScopeParseError>`; handler call site adapts; v1-AD-b existing scope-mismatch tests still pass
- [ ] Task 2: 5 rule-set DTOs + `previous_from` field on `AdminConfigAuditEntry`; `lemmy_api_common` compiles with `--features full`
- [ ] Task 3: `admin_create_rule_set` + `admin_list_rule_sets` + `ENTRY_KIND_RULE_SET_VERSION_CREATED` const + shim re-export; `lemmy_api` compiles
- [ ] Task 4: `build_admin_config_changed_payload` helper extracted; `process_set_config` threads `previous`; `project_to_audit_entry` hydrates from payload; v1-AD-b `payload_parity` unit tests still pass
- [ ] Task 5: `case_open_snapshot` module + snapshot-keyset parity unit test; `create_report.rs` populates both pin columns; `admin_emergency_remove.rs` left untouched (documented)
- [ ] Task 6: registry rule file updated with v1-AD-c section + invariant count corrected from 25 → 26
- [ ] Task 7: routes wired; `lemmy_api_routes` compiles; `lemmy_server` compiles
- [ ] Task 8: 8 e2e tests pass against real Postgres; previously failing paths stay green
- [ ] Cold-rebuild gate passes
- [ ] PR opened against `governance-v0` with body citing §10.5 NOT5 rationale line for the payload-shape evolution

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Race on `(community_id, version)` UNIQUE — two moderators create simultaneously | LOW | MED | Task 3 maps unique-violation to typed 400; task 8 test A3 asserts. Real-world concurrency is very low (admin action, ~minutes between creates). |
| `CommunityModeratorView::check_is_community_moderator` signature drift from v1-AD-b | MED | MED | Task 0 pre-flight probe 1 confirms; task 3 uses the symbol directly (compile-time proof). v1-AD-b drift-notes flag this class of drift. |
| Snapshot adds N+1 config reads at case-open (7 keys × 1 DB round-trip each unless cached) | MED | LOW | `ConfigCache` already memoises within a request. For the 7-key read at open, first hit is 7 queries; subsequent reads in the same case-open request (rule-set lookup) are 0. Measured impact: <50 ms for a cold request. |
| `applied_config_snapshot` JSONB grows as v1 adds more `requires_re_jury` keys | LOW | LOW | JSONB storage is column-per-row; adds ~200 bytes for 7 keys. Adding 20 more keys = ~800 bytes/row. Negligible at v1 scale. |
| Issue #77 refactor breaks a v1-AD-b test through field-order or shape drift | LOW | HIGH | `payload_parity` unit tests (admin_config.rs:1333+) assert exact field presence, not order; task 4 GOTCHA requires re-running them. If a test fails, the refactor went wider than intended. |
| Issue #78 typed-enum change breaks downstream call sites | LOW | LOW | Only 1 call site today (`admin_config.rs:419`); task 1 updates it in the same commit. |
| `moderation_case.rule_set_version_id` set to stale ID if `rule_set.active_version_id` was set then the rule-set row deleted | VERY LOW | MED | Rule-sets are append-only (ADR-010 invariant); deletion is not a code path. FK is `REFERENCES rule_set_version(id)` without `ON DELETE SET NULL` — if someone manually drops a row, the FK blocks case-open. Defensive and correct. |
| Shell wrapper + HTTP path now write differently-shaped payloads | MED | LOW | §4.1 + §10.5 NOT5 line — evolution, not regression. `project_to_audit_entry` degrades missing fields to `None`. Both paths still emit `entry_kind = "admin_config_changed"` via the same `append` helper. |
| Pre-phase harness audit probe 4 (exit-code propagation) reveals wrapper regression | LOW | HIGH | Task 0 catches this before task 1 runs. If probe 4 exits 0 (wrong), file DQ-V1-AD-C-XX and fix wrapper before proceeding. |
| CI timeout on 21 e2e tests | MED | LOW | Task 8 GOTCHA — if needed, `#[ignore]` the three heaviest with TODO. Warm local run is the ultimate truth. |
| CodeRabbit flags `community_moderator_required` denial_reason string as new vocabulary without registry | LOW | LOW | Reply — denial_reason text is internal to the payload, not part of the `entry_kind` registry. v1-AD-b precedent: `instance_admin_required`, `community_moderator_required`, `scope_mismatch_instance_key` are all already in use and unregistered. |
| Rust 1.94 / 1.95 MSRV drift between v1-AD-b and v1-AD-c start | VERY LOW | HIGH | `.claude/rules/pre-phase-harness-audit.md` task 0 compiles with the pinned toolchain; if MSRV drifted, task 0 fails and the advisor is notified. |
| Upstream Lemmy 1.0-beta rebase changes `get_conn` / `DbPool` surface mid-sub-phase | LOW | HIGH | Weekly-rebase discipline per CLAUDE.md. Task 0 confirms base SHA `f03ed1cba`; any drift is surfaced. |

---

## 19. Notes

- v1-AD-c is the **narrowest** of the three AD sub-phases — 9 tasks vs v1-AD-b's 10 and v1-AD-a's 12 — because the substrate is fully in place and the work is mostly composition. Complexity concentrates in task 3 (three-write tx) and task 4 (refactor ripple through the v1-AD-b write path + audit projection).
- **Issue #77's fix is strictly an extension, not a revert.** The v1-AD-b commit message for `process_set_config` explicitly says "apply_at is deliberately absent here so NOT5 gate 3 holds; the response carries apply_at via preview when task 5 extends the preview shape". v1-AD-c's task 4 analogously says: "`previous_value` + `previous_from` deliberately added here now that task 5 of v1-AD-b shipped the `preview.previous` field; threading through is the natural evolution". The CR bot may flag the payload evolution; §10.5 is the pre-emptive rationale line.
- **`REQUIRES_RE_JURY_KEYS` is load-bearing** for jury-mechanics-v1's snapshot-read path. If a future PRD adds a new `requires_re_jury` key without updating `REQUIRES_RE_JURY_KEYS`, the parity unit test fails at `cargo test` time — before any e2e run, before any PR merge. This is deliberate. Do NOT silence the test via `#[allow(dead_code)]` or `#[ignore]`; treat failures as genuine new-key-not-added-to-snapshot signal.
- v1-AD-b retro carry-forward: the **test-target compile gate per-task** pattern (v1-AD-a retro CF #1) is adopted here as Level 3 of §15. If a task says "VALIDATE: cargo check" and skips `cargo test --no-run`, a test-surface break doesn't show until task 8.
- The **emergency-remove path is deliberately untouched** in v1-AD-c. `admin_emergency_remove.rs:143-157` still builds `ModerationCaseInsertForm` with `..Default::default()`. Rationale: emergency-removes are admin-override cases that skip the jury flow entirely; grandfathering them against config edits is conceptually different. If a future PRD needs emergency-removes to pin a snapshot, the edit is localised to that file and follows the exact pattern v1-AD-c establishes in `create_report.rs`.
- **DTO surfaces ship ts-rs-exported**. `AdminCreateRuleSet`, `AdminListRuleSetsRequest`, and their responses appear in the generated TS bindings at PR-merge time; the frontend consumer (v2+) will see them. Name them carefully — `rule_set_version_id` (snake_case on the wire) vs `ruleSetVersionId` (if serde rename) — v1-AD-b uses snake_case consistently; keep it.
- **`created_by_pseudonym` is `None` in v1-AD-c**. Adding it now would require a secondary LEFT JOIN or per-row `actor_pseudonym::get_by_person_id` lookup; noise for the minimal list endpoint. v1-AD-d's dashboard aggregate will need it anyway — build there.
- **If a `/prp-review` or CodeRabbit comment raises "why isn't `created_by_pseudonym` populated on list?"**, reply with this plan §19 bullet + the task-2 DTO comment. Not a defect; deliberate v1-AD-c scope boundary.
- **No upstream-rebase risk to v1-AD-c specifically**. The touched files are governance code (`crates/api/api/src/governance/*`, `crates/api/api_crud/src/governance/*`, `crates/db_schema/src/source/governance/*`) which upstream Lemmy does not have. The only upstream-adjacent file is `crates/api/routes/src/lib.rs` — task 7's diff is additive and won't conflict unless upstream shuffles the admin block (which they have no reason to).
- At v1-AD-c close: v1-AD wave wraps on schedule. v1-AD-d (dashboard + SSE) is the next natural PRP candidate; it's not blocked by anything external but can be deferred indefinitely per pilot demand signal. jury-mechanics-v1 now has the read-side substrate to start.

---

**END OF v1-AD-c PLAN**
