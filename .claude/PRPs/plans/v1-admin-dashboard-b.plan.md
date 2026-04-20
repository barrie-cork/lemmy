# Plan: v1-AD-b — Admin Dashboard config HTTP endpoints (write / read / audit)

## Table of contents

| § | Heading | Line |
|---|---|---|
| 1 | Summary | 30 |
| 2 | Source | 40 |
| 3 | Problem statement | 55 |
| 4 | Solution statement | 75 |
| 5 | Metadata | 110 |
| 6 | Sub-phase position (v1-AD-a → b → c/d) | 128 |
| 7 | Open questions already resolved / reserved | 145 |
| 8 | Flow design | 170 |
| 9 | Mandatory reading | 240 |
| 10 | Patterns to mirror | 280 |
| 11 | Files to change | 560 |
| 12 | NOT building in v1-AD-b | 600 |
| 13 | Step-by-step tasks | 625 |
| 14 | Testing strategy | 1160 |
| 15 | Validation commands (DoD) | 1205 |
| 16 | Acceptance criteria | 1285 |
| 17 | Completion checklist | 1320 |
| 18 | Risks and mitigations | 1345 |
| 19 | Notes | 1395 |

---

## 1. Summary

v1-AD-b is the **write-endpoint sub-phase** of the v1 admin-dashboard keystone. It ships the first HTTP surface that lets admins edit `governance_config` over JSON (replacing `scripts/brehon/admin-config-write.sh`), reads effective config with provenance, and paginates the config-change audit trail.

Three routes (`POST /admin/config`, `GET /admin/config`, `GET /admin/config/audit`), one new accessor family (`get_int_opt` / `get_float_opt` / `get_bool_opt` / `get_text_opt`), one new capability gate reusing `is_admin` + `CommunityModeratorView`, and one read-only pre-tx dry-run query function dispatched by key category. Every successful write emits an `admin_config_changed` entry via `governance_log::append`; every denied write emits `admin_config_change_denied` (both consts shipped in v1-AD-a). Payload byte-identical to the shell script so the NOT5 deprecation gate 3 holds. No schema migrations — v1-AD-a provided the substrate.

---

## 2. Source

- [../prds/v1-admin-dashboard.prd.md](../prds/v1-admin-dashboard.prd.md) §4 (HTTP API surface), §5.2 (default-values table for impact-preview shapes), §7 (security / capability checks), §8.4 (NOT5 shell-script deprecation gate).
- [../plans/completed/v1-admin-dashboard-a.plan.md](completed/v1-admin-dashboard-a.plan.md) §4.1 (load-bearing decisions), §20 v1-AD-b stub — this plan's parent.
- [../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) ADR-008 (signed log), ADR-010 (staged releases, no retroactive invalidation), ADR-013 (EmergencyRemove exhaustive match), ADR-015 (pseudonyms), OQ-018 (admin config write — **this plan implements it**), OQ-V1-AD-03 (resolved 2026-04-20: dry-run pre-tx read-only query, no SAVEPOINT).
- [../../../docs/brehon-law-inspired-network/04-data-model-and-api.md](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) §3 (governance_config + governance_log models), §7 (route table).
- [../../../docs/brehon-law-inspired-network/06-security-and-threat-model.md](../../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) §2.1 (step-up reserved slot — v2, not v1-AD-b).
- v1-AD-a retro: `.claude/PRPs/reports/v1-AD-a-retro.md` (the "test-target compile gate" carry-forward is adopted in §15 Level 4).
- v1-AD-a complete report: `.claude/PRPs/reports/v1-AD-a-complete-report.md` (baseline state).
- Shell script: `scripts/brehon/admin-config-write.sh:133-161` (byte-identity target for NOT5 gate 3).

---

## 3. Problem statement

Four things must exist on the wire before v1-AD-a's substrate becomes operator-usable:

1. **A type-safe single-key write endpoint.** Admins currently edit `governance_config` via a bash wrapper that shells out to `psql`. The wrapper works, but: (a) it lives outside the Rust safety net, (b) it cannot validate against `CONFIG_KEY_METADATA.valid_range` / `valid_enum`, and (c) it cannot enforce `ConfigScope::Instance`-vs-`Community`-vs-`Both` policy. A Rust handler backed by `CONFIG_KEY_METADATA` can.
2. **A read endpoint that shows provenance.** `GET /admin/config` must return, for every key, the effective value AND whether it came from a `community:<id>` row, an `instance` row, or the Rust const default. v0 offers no provenance — a human reading the DB has no way to distinguish "instance row was set" from "no row, falling back to const".
3. **A paginated audit trail.** Every `admin_config_changed` and `admin_config_change_denied` entry in `governance_log` must be filterable by `key`, `scope`, `actor_pseudonym`, and date range. No such view exists today — operators needing audit run ad-hoc SQL.
4. **An opt-aware accessor family.** `rule_set.active_version_id` has no seed row AND no Rust const default on purpose (v1-AD-a advisor edit #2). The existing `get_int` errors when both are absent. v1-AD-b needs `get_int_opt` (+ float/bool/text siblings) that returns `Ok(None)` for legitimately-absent keys so v1-AD-c's rule-set handler can call it cleanly.

Without #1 and #4, v1-AD-c (rule-set routes) is blocked. Without #2, the operator UX for any v1 sub-phase regresses relative to `psql`. Without #3, NOT5 gate 3 (byte-identical governance_log payload from shell vs. HTTP paths) cannot be proved.

---

## 4. Solution statement

Build four handlers + one accessor family on top of v1-AD-a's substrate:

- **`POST /api/v4/governance/admin/config`** (`admin_set_config`) — reads key metadata from `CONFIG_KEY_METADATA`, validates scope + type + range + enum, computes dry-run impact **before** opening a transaction, then conditionally inserts a `governance_config` row + `admin_config_changed` log entry inside `run_transaction`. Capability check runs first; denial emits `admin_config_change_denied` without a config write.
- **`GET /api/v4/governance/admin/config`** (`admin_get_config`) — no query params: returns full effective config across all keys in `CONFIG_KEY_METADATA`, each with `effective_from: "community:<id>" | "instance" | "default"`. With `?key=<dotted>&community_id=<id>`: single-key read using the same cascade.
- **`GET /api/v4/governance/admin/config/audit`** (`admin_get_config_audit`) — paginated list of `governance_log` rows filtered to `entry_kind IN ('admin_config_changed', 'admin_config_change_denied')`, with filter params `key`, `scope`, `actor_pseudonym`, `since`, `until`, `page`, `limit`. Payload destructured into typed response fields.
- **`get_int_opt` / `get_float_opt` / `get_bool_opt` / `get_text_opt`** accessors in `crates/api/api/src/governance/config.rs` — thin wrappers around `fetch_value` that return `Ok(None)` when no DB row matches, skipping the `const_default_*` fallback. Caching strategy: add a `CachedValue::Absent` variant so "checked, no value" is distinguishable from "not yet queried".

Payload for `admin_config_changed` is constructed byte-identically to the shell script (`scripts/brehon/admin-config-write.sh:146-157`): a JSON object with field order `scope`, `key`, `value_type`, `value`, `reason`. The `downstream_impact` block from PRD §4.2 is returned in the HTTP response but NOT included in the log payload — keeping payload shape shell-identical for NOT5 gate 3 is more important than adding impact metadata to the log. An optional addendum can ship in v1-AD-b.1 if operators ask.

Routes register in `crates/api/routes/src/lib.rs:531-536`'s existing `/governance/admin` scope, inheriting `rate_limit.post()`. Sub-phase target branch is `phase-v1-AD-b` branched from `governance-v0` (which should have v1-AD-a merged at `e61f78edf`, PR #72); PR target is `governance-v0` per `.claude/rules/phase-branch.md`.

### 4.1 Architecturally load-bearing decisions locked in this sub-phase

- **Dry-run impact is computed PRE-transaction.** Per OQ-V1-AD-03 resolution (2026-04-20 `3f0dd6572`): `diesel_utils::run_transaction` exposes no SAVEPOINT primitive. Adding one for this one use-case was rejected. Instead, the handler runs a read-only Diesel query against the current persisted state + the proposed value before calling `run_transaction`. When `dry_run = true` the handler returns early with the impact payload and no write. When `dry_run = false` the already-computed impact is carried into the 200 response. Post-commit drift (another write between impact-query and tx) is not the handler's responsibility to describe — the pre-tx snapshot is the contract.
- **`get_int_opt` family returns `Ok(None)`, not `Ok(Some(fallback))`.** Per v1-AD-a advisor edit #2 and §20 stub GOTCHA: the no-row-no-const path must return `Ok(None)`. Writing `get_int` with a `-1` sentinel or `Some(0)` fallback re-introduces the footgun advisor edit #2 removed. A new compile-time test `rule_set_active_version_absent_returns_none` asserts this.
- **`admin_config_changed` payload is byte-identical to the shell script.** Field order `{scope, key, value_type, value, reason}` — not alphabetical, not optimized. `serde_json::json!` preserves macro-literal key order; the resulting `Value` serializes in declaration order. A new `governance_log_payload_shell_parity` integration test writes both paths and diffs the stored `payload` column byte-level (signatures differ but the `payload` JSONB must match).
- **Capability-check-denied WRITES to governance_log.** The request path is: `is_admin` check → if fail, look up admin pseudonym, call `governance_log::append` with `ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED`, return 403. This is a new pattern — v0 capability-check failures are silent. Security implication: a user whose admin privileges were revoked mid-session still has a pseudonym row (actor_pseudonym is append-only), so `get_or_create` returns the existing row without requiring admin status. A non-admin caller who has *never* had a pseudonym will trigger `get_or_create` to insert one — acceptable because the `actor_pseudonym` table is the GDPR pseudonymisation layer for ANY person writing to the governance log, including denied actors.
- **`POST /admin/config` edits to community-scoped keys use `CommunityModeratorView::check_is_community_moderator`.** Instance-scope keys use `is_admin`. Keys with `ConfigScope::Both` dispatch on request `scope` field. Keys with `ConfigScope::Instance` reject a community-scoped request with 403 + denial log. Keys with `ConfigScope::Community` reject an instance-scoped request the same way.
- **`apply_at` on the request is honored for `requires_re_jury` keys only.** Other keys ignore `apply_at`. For `NextJuryCycle`, the implementation is "insert row with `valid_from = now() + 0s`" in v1-AD-b — delayed activation is a v1-AD-b.1 refinement. The request-level field parses but v1-AD-b does not yet schedule future `valid_from` values. Documented; no drift risk because the `governance_config_current` view already returns `valid_from <= now()` rows.
- **No new migrations.** v1-AD-a migration `2026-04-22-000300-0000_seed_v1_config_keys` plus the `add_rule_set_versions` / `add_sponsor_allowlist` / `add_case_applied_config_snapshot` migrations already ship the v1 schema. v1-AD-b is pure Rust + route wiring + tests.
- **No SSE, no askama, no askama template rendering.** SSE is v1-AD-d (OQ-V1-AD-02 resolved to hand-rolled async-stream, but landing in v1-AD-d). Askama is v1-AD-e (OQ-V1-AD-01 deferred to v1.x). v1-AD-b ships API-only.

---

## 5. Metadata

| Field | Value |
|---|---|
| Type | `HANDLER + DTO + CROSS_CUTTING` |
| Complexity | MEDIUM-HIGH |
| Crates affected | `lemmy_api_common` (DTOs), `lemmy_api` (3 new handlers + `get_*_opt` accessor family + impact-query module), `lemmy_api_routes` (route registrations), `lemmy_db_views_governance_modlog` (or new view crate if needed for audit list), `lemmy_server` (tests/e2e.rs) |
| v0 step | v1 post-MVP per [ADR-010] — keystone sub-phase continues |
| Dependencies | v1-AD-a merged at `governance-v0` (tip `e61f78edf` post-PR #72); OQ-V1-AD-03 resolved on trunk at `3f0dd6572`. **Pre-flight check: `governance-v0` HEAD must contain `ENTRY_KIND_ADMIN_CONFIG_CHANGED` at `crates/db_schema/src/source/governance/governance_log.rs` — task 0 verifies.** |
| Estimated tasks | **8 implementation tasks + pre-flight Task 0 + 1 integration-test task = 10 total** (Tasks 0–9 enumerated in §13) |
| Sub-phase target branch | `phase-v1-AD-b` branched from `governance-v0` |
| PR target | `governance-v0` (per `.claude/rules/phase-branch.md`) |
| Blocks | v1-AD-c (rule-set routes read `rule_set.active_version_id` via `get_int_opt`), v1-AD-d (dashboard aggregate reads config) |
| Unblocks | none — v1-AD-b is itself unblocked |

---

## 6. Sub-phase position (v1-AD-a → b → c/d)

| Sub-phase | Status | What's landed / pending |
|---|---|---|
| **v1-AD-a** | ✅ merged (PR #72, commit `e61f78edf`) | 4 migrations, `ConfigKeyMetadata` registry (61 entries), 2 new `ENTRY_KIND_ADMIN_CONFIG_*` consts, entry-kind registry rule, 3 OQs opened |
| **v1-AD-b** (this plan) | pending | 3 HTTP handlers, `get_*_opt` accessor family, dry-run impact queries, capability gates, audit list handler |
| v1-AD-c | blocked on v1-AD-b | `rule_set_version` CRUD routes + `submit_jury_vote` wire-up (`applied_config_snapshot` + `rule_set_version_id`) |
| v1-AD-d | blocked on v1-AD-b + OQ-V1-AD-02 (resolved) | `/admin/dashboard` aggregate + SSE `/admin/audit/stream` (hand-rolled async-stream) |
| v1-AD-e | DEFERRED — OQ-V1-AD-01 resolved "ship API-only" | Askama pages — descoped from v1-AD wave per 3f0dd6572 resolution |

v1-AD-c and v1-AD-d will plan on top of merged v1-AD-b; neither is in scope here.

---

## 7. Open questions already resolved / reserved

| OQ | Resolution | Impact on v1-AD-b |
|---|---|---|
| OQ-V1-AD-01 | Defer HTML pages to v1.x (resolved 2026-04-20) | v1-AD-b ships no askama; no `lemmy_api` template crate dep added |
| OQ-V1-AD-02 | Hand-roll SSE via async-stream (resolved 2026-04-20) | v1-AD-b ships no SSE — SSE is v1-AD-d |
| **OQ-V1-AD-03** | **Dry-run impact pre-tx read-only query; no SAVEPOINT** (resolved 2026-04-20) | **Binding — §10.4 and task 3 implement this directly.** |
| OQ-018 | Admin HTTP config-write endpoint (primary) | **v1-AD-b is the implementation.** NOT5 gate 1 (OQ-018 endpoint ships) is satisfied by this plan's task 3 + task 4. |

**No new OQs open in v1-AD-b.** If any blocking ambiguity emerges mid-implementation, it goes to `.claude/decision-queue.json` per `.claude/rules/decision-queue.md`.

---

## 8. Flow design

### Before state (governance-v0 HEAD `e61f78edf`, v1-AD-a merged via PR #72)

```text
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              BEFORE STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  HTTP API surface:                                                            ║
║    /admin/assign-jury        (POST) — v0                                      ║
║    /admin/close-case         (POST) — v0                                      ║
║    /admin/reputation-stats   (GET)  — v0                                      ║
║    /admin/config             DOES NOT EXIST                                   ║
║    /admin/config/audit       DOES NOT EXIST                                   ║
║                                                                               ║
║  Substrate (from v1-AD-a):                                                    ║
║    governance_config         61 seeded rows, parametric parity test           ║
║    CONFIG_KEY_METADATA       61 compile-time entries                          ║
║    ENTRY_KIND_ADMIN_CONFIG_CHANGED         = "admin_config_changed"           ║
║    ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED   = "admin_config_change_denied"     ║
║    moderation_case.applied_config_snapshot — column present, never written    ║
║    rule_set_version — table present, never written                            ║
║                                                                               ║
║  Shell-only path:                                                             ║
║    scripts/brehon/admin-config-write.sh emits admin_config_changed            ║
║    directly via psql INSERT INTO governance_log.                              ║
║                                                                               ║
║  Accessor family:                                                             ║
║    get_int / get_float / get_bool / get_text — error when no DB row           ║
║                                                      AND no DEFAULT_* const   ║
║                                                                               ║
║  PAIN: admins edit JSON-typed config via bash wrappers that can't reject      ║
║        out-of-range or out-of-enum values, can't validate scope policy,       ║
║        and can't provide dry-run previews.                                    ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### After state (end of v1-AD-b)

```text
╔═══════════════════════════════════════════════════════════════════════════════╗
║                               AFTER STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  HTTP API surface:                                                            ║
║    /admin/assign-jury        (POST) — v0 (unchanged)                          ║
║    /admin/close-case         (POST) — v0 (unchanged)                          ║
║    /admin/reputation-stats   (GET)  — v0 (unchanged)                          ║
║    /admin/config             (POST) — v1-AD-b admin_set_config        NEW     ║
║    /admin/config             (GET)  — v1-AD-b admin_get_config        NEW     ║
║    /admin/config/audit       (GET)  — v1-AD-b admin_get_config_audit  NEW     ║
║                                                                               ║
║  Accessor family extended:                                                    ║
║    get_int_opt / get_float_opt / get_bool_opt / get_text_opt — return         ║
║      Ok(None) when no DB row AND no DEFAULT_* const (for rule_set.*           ║
║      active_version_id and any future legitimately-absent keys)               ║
║    CachedValue::Absent variant added for "checked, no value" memoisation      ║
║                                                                               ║
║  governance_log writes:                                                       ║
║    Every successful POST /admin/config → admin_config_changed entry           ║
║    Every denied  POST /admin/config  → admin_config_change_denied entry       ║
║    Both byte-identical-payload-shape to the shell script (NOT5 gate 3 proved) ║
║                                                                               ║
║  Dry-run impact queries:                                                      ║
║    7 key categories (thresholds, jury.panel_size, jury.max_concurrent,        ║
║    liability.sponsor_liability_floor, report.case_threshold_micros,           ║
║    decay.*, other) — each dispatches to a read-only Diesel function           ║
║    executed BEFORE run_transaction opens                                      ║
║                                                                               ║
║  Capability matrix:                                                           ║
║    Instance-scope key write → is_admin                                        ║
║    Community-scope key write → CommunityModeratorView::check_is_community_mod ║
║    requires_step_up = true + governance.dashboard.step_up_enforced = true     ║
║                             → 403 + denial log (v2 reserved behaviour)        ║
║    requires_step_up = true + governance.dashboard.step_up_enforced = false    ║
║                             → allow + denial-log still emitted (v1 advisory)  ║
║                                                                               ║
║  VALUE: operators can edit config via HTTP with type+scope+range safety.      ║
║  VALUE: audit log readable over JSON; no more ad-hoc SQL.                     ║
║  VALUE: v1-AD-c can call config::get_int_opt without crashing on rule_set     ║
║         active_version_id absence.                                            ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### Data flow — POST /admin/config happy path

```
1. HTTP request → extract LocalUserView (Lemmy auth middleware)
2. Look up admin_pseudonym via actor_pseudonym::get_or_create
3. Capability check:
   a. CONFIG_KEY_METADATA lookup by `data.key` → err 400 if not found
   b. Policy check: key.scope vs. request.scope dispatch
      - ConfigScope::Instance + request.scope = Instance → is_admin
      - ConfigScope::Community + request.scope = Community(id) → check_is_community_moderator
      - ConfigScope::Both  + request.scope = Instance → is_admin
      - ConfigScope::Both  + request.scope = Community(id) → check_is_community_moderator
      - Mismatch → governance_log::append(ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED) + 403
4. Type validation: data.value_type matches metadata.value_type
5. Range validation (Int/Float): min <= value <= max (from metadata.valid_range)
6. Enum validation: value ∈ metadata.valid_enum (for Enum keys)
7. requires_step_up check: if metadata.requires_step_up
                          AND config::get_bool(Scope::Instance, "governance.dashboard.step_up_enforced")
                          → 403 + denial log (v2 step-up not implemented yet)
                          → else: advisory log (denied=false) + continue
8. Dry-run impact: dispatch to impact_query_for_key(key, current_value, proposed_value)
                 → returns DownstreamImpact struct
9. Branch on data.dry_run:
   a. dry_run = true  → Return 200 with impact, no write
   b. dry_run = false → Proceed to step 10
10. Open run_transaction:
    a. INSERT INTO governance_config (...) — new append-only row
    b. governance_log::append(ENTRY_KIND_ADMIN_CONFIG_CHANGED, payload, Some(admin_pseudonym))
       where payload = json!({"scope": …, "key": …, "value_type": …, "value": …, "reason": …})
       (field order matches shell script; NOT5 gate 3)
11. Return 200 with {applied: true, config_id, governance_log_id, preview, applied_at}
```

---

## 9. Mandatory reading (implementation agent MUST read before starting)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `crates/api/api/src/governance/config.rs` | 1-154 (types + Scope), 156-292 (get_int/float/bool/text — **copy the template for the `_opt` family**) | The accessor pattern you're extending |
| P0 | `crates/api/api/src/governance/config.rs` | 315-373 (fetch_value + fetch_value_at_scope) | The no-row case returns `Ok(None)` here — the opt family just skips the const-default step |
| P0 | `crates/api/api/src/governance/config.rs` | 453-574 (const_default_* lookups) | Confirm the absence of `rule_set.active_version_id` — task 5's parity test asserts |
| P0 | `crates/api/api/src/governance/config.rs` | 576-715 (SEEDED_KEYS_WITH_CONSTS + EXPECTED_SEED_COUNT_V1_AD) | No changes to this list in v1-AD-b; read so you know it's untouched |
| P0 | `crates/api/api/src/governance/config.rs` | 717-1454 (CONFIG_KEY_METADATA array) | Read `ConfigScope::Instance` / `Community` / `Both` distribution — tasks 3+6 dispatch on it |
| P0 | `crates/api/api/src/governance/admin_assign_jury.rs` | 63-187 (handler + process_assignment tx body) | The HTTP-handler-with-run_transaction pattern |
| P0 | `crates/api/api/src/governance/admin_close_case.rs` | 23-100 (simpler example) | Simpler mirror if the longer handler is too noisy |
| P0 | `crates/db_schema/src/source/governance/governance_log.rs` | 68-103 (model + InsertForm), 145-155 (ENTRY_KIND_* consts incl. 2 from v1-AD-a), 175-236 (append function) | The append signature + the two new consts you call |
| P0 | `crates/db_schema/src/source/governance/governance_config.rs` | 21-53 (model + InsertForm) | The Diesel types you insert into |
| P0 | `scripts/brehon/admin-config-write.sh` | 105-161 (INSERT block) | Byte-identity target for `admin_config_changed` payload (NOT5 gate 3) |
| P0 | `crates/api/api_common/src/governance.rs` | 58-73 (list DTO), 143-187 (admin DTO shape) | DTO conventions (`#[skip_serializing_none]` + `ts_rs`) |
| P0 | `crates/api/routes/src/lib.rs` | 514-537 (governance scope) | Route registration slot (exactly lines 531-536) |
| P0 | `../prds/v1-admin-dashboard.prd.md` | §4 (HTTP API), §7 (security) | Endpoint shapes + capability rules |
| P0 | `.claude/PRPs/plans/completed/v1-admin-dashboard-a.plan.md` | §20 v1-AD-b stub | Your predecessor-written scope |
| P1 | `crates/api/api/src/governance/admin_reputation_stats.rs` | 76-218 (GET pattern + COUNT queries) | Mirror for `admin_get_config` GET pattern + reference for `capability_query` which task 3 mirrors for threshold impact |
| P1 | `crates/api/api/src/governance/list_cases.rs` | 25-43 (paginated GET) | Mirror for `admin_get_config_audit` pagination |
| P1 | `crates/db_views/governance_modlog/src/impls.rs` | 196-234 (`list_capability_changed_entries_since`) | Mirror for governance_log filtering query + pagination |
| P1 | `crates/server/tests/e2e.rs` | 1360-1418 (`config_parity_round_trip`) | Test harness pattern for config tests |
| P1 | `crates/server/tests/e2e.rs` | 979-1025 (admin-endpoint test pattern) | Test harness pattern for direct-handler invocation |
| P1 | `.claude/rules/governance-log-entry-kind-registry.md` | full | Registry MUST remain in sync; v1-AD-b adds no new kinds but the existing 2 are wired through here |
| P1 | `.claude/rules/pm-plugin-hooks-stable.md` | full | PM path is untouched; the 6-hook presence loop runs in §15 Level 6 |
| P1 | `.claude/rules/cargo-output-capture.md` + `no-cargo-output-paste.md` | full | Enforced for every DoD gate |
| P1 | `.claude/rules/pre-phase-harness-audit.md` | full | Task 0 runs the four wrapper probes |
| P2 | `crates/api/api/src/governance/federation_outbox.rs` | 166-201 (denial-shape precedent: `get` strict pseudonym lookup + nested append) | Precedent for policy-check-failed write to governance_log |
| P2 | `crates/api/api/src/governance/sponsor_liability.rs` | 322-354 (append with clamped_from branch) | Precedent for payload with multiple optional fields |

**External Documentation:** none. All dependencies (`serde_json`, `diesel-async`, `activitypub_federation`, `tokio`) are already in the workspace. No Crate.toml edits.

---

## 10. Patterns to mirror

### 10.1 TYPED_ACCESSOR_OPT (new `get_int_opt` — mirror `get_int` with no-const-default fallback)

```rust
// SOURCE: crates/api/api/src/governance/config.rs:162-193 (get_int)
// TARGET: same file, append below get_text (line 292), before line 313 "Private helpers"
// COPY AND ADAPT THIS PATTERN:

pub async fn get_int(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<i64> {
  let scope_repr = scope.as_str();
  if let Some(CachedValue::Int(v)) = cache
    .entries
    .get(&(scope_repr.as_ref().to_string(), key.to_string()))
  {
    return Ok(*v);
  }
  let v = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Int(v)) => v,
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as int but stored as {other:?}"
      ))
      .into());
    }
    None => const_default_int(key).ok_or_else(|| {                         // <-- REMOVE const default fallback
      LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` missing from DB and has no Rust const default"
      ))
    })?,                                                                   // <-- in _opt version; return Ok(None) instead
  };
  cache
    .entries
    .insert((scope_repr.into_owned(), key.to_string()), CachedValue::Int(v));
  Ok(v)
}
```

### 10.2 HTTP_HANDLER_WITH_RUN_TRANSACTION (mirror for `admin_set_config`)

```rust
// SOURCE: crates/api/api/src/governance/admin_assign_jury.rs:63-87
// COPY THIS PATTERN:

pub async fn admin_assign_jury(
  Json(data): Json<AdminAssignJury>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminAssignJuryResponse>> {
  is_admin(&local_user_view)?;

  let admin_id = local_user_view.person.id;
  let admin_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let data_for_tx = data;
  let pseudonym_for_tx = admin_pseudonym.clone();

  let outcome = conn
    .run_transaction(|conn| {
      async move { process_assignment(conn, pseudonym_for_tx, data_for_tx).await }.scope_boxed()
    })
    .await?;

  Ok(Json(outcome))
}
```

### 10.3 POLICY_CHECK_FAILED_LOG_WRITE (precedent for denial emission)

```rust
// SOURCE: crates/api/api/src/governance/admin_assign_jury.rs:165-187 (standard append call site pattern)
// SOURCE: crates/api/api/src/governance/federation_outbox.rs:166 (uses `get` not `get_or_create` when admin pseudonym is expected to exist)
// ADAPT FOR DENIAL PATH in admin_set_config:

// Denial path is OUTSIDE run_transaction (capability check happens before tx)
// but STILL writes to governance_log — wrap just the append in its own tx:

let pool = &mut context.pool();

// Deliberately use get_or_create, not get. A non-admin caller may never have
// had a pseudonym; creating one on first denial is correct GDPR behaviour —
// actor_pseudonym is the redaction layer for every person who interacts
// with governance state, including denied actors.
let actor_pseudonym_str =
  actor_pseudonym_helper::get_or_create(pool, local_user_view.person.id).await?;

let denial_payload = json!({
  "scope": scope_repr,
  "key": data.key,
  "value_type": data.value_type,
  "value": data.value,
  "reason": data.reason,
  "denial_reason": "instance_admin_required" | "community_moderator_required" | "scope_mismatch",
});

governance_log::append(
  pool,
  ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
  denial_payload,
  Some(actor_pseudonym_str),
)
.await?;

return Err(LemmyErrorType::NotAnAdmin.into());  // or appropriate 403 variant
```

### 10.4 DRY_RUN_IMPACT_QUERY (read-only, pre-tx — OQ-V1-AD-03)

```rust
// SOURCE: crates/api/api/src/governance/admin_reputation_stats.rs:195-218
// MIRROR for dry-run threshold-category impact:

// For `thresholds.jury_reliability` / `thresholds.reporting_accuracy` / etc.:
async fn impact_for_threshold_key(
  conn: &mut AsyncPgConnection,
  key: &str,          // "thresholds.jury_reliability" etc.
  community_bind: Option<i32>,
  current_threshold: i64,
  proposed_threshold: i64,
) -> LemmyResult<ThresholdImpact> {
  // Query reputation_snapshot, comparing the raw jury_reliability / reporting_accuracy /
  // endorsement_strength column against BOTH thresholds.
  // Return delta of jury_eligible users (and similar for other flags).

  let column = match key {
    "thresholds.jury_reliability" => "jury_reliability",
    "thresholds.reporting_accuracy" => "reporting_accuracy",
    "thresholds.endorsement_strength" => "endorsement_strength",
    _ => return Err(LemmyErrorType::Unknown(format!("not a threshold key: {key}")).into()),
  };

  // sql_query pattern from admin_reputation_stats.rs:195 (same shape, different columns)
  // COUNT(*) FILTER (WHERE column >= $current) vs. COUNT(*) FILTER (WHERE column >= $proposed)
  // returns (users_losing, users_gaining) computed as diff
  //
  // Hard GOTCHA: reputation_snapshot.jury_eligible is a DERIVED boolean (computed at
  // snapshot time against the current threshold). To preview impact against a PROPOSED
  // threshold, query against the RAW jury_reliability / reporting_accuracy /
  // endorsement_strength columns, NOT against the derived jury_eligible boolean.
  ...
}
```

### 10.5 GOVERNANCE_LOG_APPEND (inside tx, carry `config_id` from the INSERT)

```rust
// SOURCE: crates/api/api/src/governance/admin_assign_jury.rs:165-187
// ADAPT for admin_set_config's write path (inside run_transaction):

// 1. INSERT new governance_config row (append-only append-history shape)
let new_row: GovernanceConfig = diesel::insert_into(governance_config::table)
  .values(&GovernanceConfigInsertForm {
    scope: scope_repr.clone(),
    key: data.key.clone(),
    value_type: data.value_type.as_str().to_string(),
    value_int, value_float, value_bool, value_text,   // one populated based on value_type
    updated_by: Some(admin_id),
  })
  .returning(GovernanceConfig::as_returning())
  .get_result::<GovernanceConfig>(conn)
  .await?;

// 2. Emit admin_config_changed log (byte-identical to shell script)
// Field order: scope, key, value_type, value, reason — serde_json::json! preserves this
let payload = json!({
  "scope":      scope_repr,
  "key":        data.key,
  "value_type": data.value_type.as_str(),
  "value":      data.value,    // rehydrate the typed value back into a JSON Value
  "reason":     data.reason,
});

let log_row = governance_log::append(
  &mut conn.into(),
  ENTRY_KIND_ADMIN_CONFIG_CHANGED,
  payload,
  Some(admin_pseudonym.clone()),
)
.await?;

// log_row.id is the governance_log_id returned in the response
```

### 10.6 PAGINATED_LIST (mirror for `admin_get_config_audit`)

```rust
// SOURCE: crates/api/api/src/governance/list_modlog.rs:30-49
// SOURCE: crates/db_views/governance_modlog/src/impls.rs:196-234
// ADAPT for governance_log audit filtering:

const DEFAULT_PAGE: i64 = 1;
const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 100;

pub async fn admin_get_config_audit(
  Query(data): Query<AdminGetConfigAudit>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<Vec<AdminConfigAuditEntry>>> {
  // Capability: instance_admin sees all; community_admin(id) sees scope="community:<id>"
  // For v1-AD-b: require is_admin (instance view). Community-scoped filter via the `scope` query param.
  is_admin(&local_user_view)?;

  let page = data.page.unwrap_or(DEFAULT_PAGE).max(1);
  let limit = data.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
  let offset = page.saturating_sub(1).saturating_mul(limit);

  // Push filtering into Diesel — DO NOT skip/take Rust-side when filtering
  // is on an indexed column (entry_kind is indexed per Phase 4 migration).
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let mut query = governance_log::table
    .filter(governance_log::entry_kind.eq_any(vec![
      ENTRY_KIND_ADMIN_CONFIG_CHANGED,
      ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
    ]))
    .order_by((governance_log::created_at.desc(), governance_log::id.desc()))
    .limit(limit)
    .offset(offset)
    .into_boxed();

  if let Some(k) = &data.key { /* payload-side JSONB filter — see GOTCHA below */ }
  if let Some(s) = &data.scope { /* payload-side JSONB filter */ }
  if let Some(p) = &data.actor_pseudonym {
    query = query.filter(governance_log::actor_pseudonym.eq(p));
  }
  if let Some(since) = data.since { query = query.filter(governance_log::created_at.ge(since)); }
  if let Some(until) = data.until { query = query.filter(governance_log::created_at.lt(until)); }

  let rows = query.load::<GovernanceLog>(conn).await?;
  let entries = rows.into_iter().map(project_to_audit_entry).collect();
  Ok(Json(entries))
}
```

**GOTCHA for JSONB payload filter:** `governance_log.payload` is `jsonb`. Diesel-side filtering on JSONB uses raw SQL fragments (`sql_query`) or the `diesel-full-text-search` crate — neither is worth adding for v1-AD-b. For the `key` and `scope` filters, load the pre-filtered rows (by `entry_kind` + `actor_pseudonym` + date range) into Rust and post-filter by `payload["key"]` / `payload["scope"]`. Acceptable for v1 since the audit list is expected to be low-volume (<1000 rows/day); a later version can push the JSONB filter into SQL via `@>` operator if profiling shows a need.

### 10.7 DTO_SHAPE (mirror existing admin DTOs)

```rust
// SOURCE: crates/api/api_common/src/governance.rs:143-187
// ADD to the same file:

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminSetConfig {
  pub key: String,
  pub value_type: String,           // "int" | "float" | "bool" | "text"
  pub value: serde_json::Value,     // typed by value_type on server
  pub scope: String,                // "instance" | "community:<id>"
  pub apply_at: Option<String>,     // "immediate" (default) | "next_jury_cycle" | "next_snapshot_job"
  pub dry_run: Option<bool>,        // default false
  pub reason: String,               // required; non-empty trim
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminSetConfigResponse {
  pub applied: bool,                           // false iff dry_run
  pub config_id: Option<i64>,                  // None iff dry_run
  pub governance_log_id: Option<i64>,          // None iff dry_run
  pub preview: ConfigChangePreview,            // always populated
  pub applied_at: Option<chrono::DateTime<chrono::Utc>>,  // None iff dry_run
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ConfigChangePreview {
  pub previous: ConfigValueWithProvenance,
  pub new: ConfigValueWithProvenance,
  pub downstream_impact: serde_json::Value,   // shape varies by key category
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ConfigValueWithProvenance {
  pub value: serde_json::Value,
  pub effective_from: String,   // "community:<id>" | "instance" | "default"
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminGetConfig {
  pub key: Option<String>,           // if Some, return single-key; else full
  pub community_id: Option<CommunityId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminGetConfigResponse {
  pub entries: Vec<AdminConfigEntry>,   // one per key in CONFIG_KEY_METADATA (full) or single
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminConfigEntry {
  pub key: String,
  pub value_type: String,
  pub value: serde_json::Value,
  pub effective_from: String,                 // "community:<id>" | "instance" | "default"
  pub scope: String,                          // metadata.scope: Instance / Community / Both
  pub requires_re_jury: bool,
  pub requires_step_up: bool,
  pub apply_at_default: String,
  pub description: String,
  pub doc_anchor: String,
  pub valid_range: Option<(f64, f64)>,
  pub valid_enum: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminGetConfigAudit {
  pub key: Option<String>,
  pub scope: Option<String>,
  pub actor_pseudonym: Option<String>,
  pub since: Option<chrono::DateTime<chrono::Utc>>,
  pub until: Option<chrono::DateTime<chrono::Utc>>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminConfigAuditEntry {
  pub id: i64,                                     // governance_log.id
  pub entry_kind: String,                          // "admin_config_changed" | "admin_config_change_denied"
  pub scope: String,                               // projected from payload
  pub key: String,                                 // projected from payload
  pub value_type: String,                          // projected from payload
  pub previous_value: Option<serde_json::Value>,   // None on first change / denials
  pub new_value: serde_json::Value,                // projected from payload
  pub reason: String,                              // projected from payload
  pub actor_pseudonym: Option<String>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub signature: Option<Vec<u8>>,
  pub denial_reason: Option<String>,               // populated only on admin_config_change_denied
}
```

### 10.8 ROUTE_REGISTRATION (exact slot in `crates/api/routes/src/lib.rs:531-536`)

```rust
// SOURCE: crates/api/routes/src/lib.rs:531-536
// MODIFY the /admin scope:

.service(
  scope("/admin")
    .route("/assign-jury", post().to(admin_assign_jury))
    .route("/close-case", post().to(admin_close_case))
    .route("/reputation-stats", get().to(admin_reputation_stats))
    .service(
      scope("/config")
        .route("", post().to(admin_set_config))
        .route("", get().to(admin_get_config))
        .route("/audit", get().to(admin_get_config_audit)),
    ),
),
```

### 10.9 TEST_PATTERN (`crates/server/tests/e2e.rs`)

```rust
// SOURCE: crates/server/tests/e2e.rs:979-1025 (admin_assign_jury test pattern)
// MIRROR for admin_set_config happy-path:

#[tokio::test]
async fn admin_set_config_happy_path() -> Result<(), Box<dyn Error>> {
  // 1. Spin up testcontainer
  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);

  // 2. Apply schema
  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;
  }

  // 3. Build async pool + LemmyContext
  let pool: ActualDbPool = build_db_pool_for_tests();
  let rate_limit = RateLimit::with_debug_config();
  rate_limit.set_config(enum_map! { /* bump all buckets to 10k/60s */ });
  let context = LemmyContext::create(pool, ..., secret, rate_limit.clone());

  // 4. Seed admin user + read LocalUserView
  let admin_id = seed_person(&context, instance_id, "admin", true).await?;
  let admin_view = LocalUserView::read_person(&mut context.pool(), admin_id).await?;

  // 5. Issue POST /admin/config to bump jury.panel_size from 5 to 7
  let resp = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".into(),
      value_type: "int".into(),
      value: json!(7),
      scope: "instance".into(),
      apply_at: Some("next_jury_cycle".into()),
      dry_run: Some(false),
      reason: "v1-AD-b happy-path test".into(),
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?
  .into_inner();

  // 6. Assert response
  assert!(resp.applied);
  assert!(resp.config_id.is_some());
  assert!(resp.governance_log_id.is_some());
  assert_eq!(resp.preview.previous.value, json!(5));
  assert_eq!(resp.preview.new.value, json!(7));

  // 7. DB checks
  let conn = &mut get_conn(&mut (&mut /* pool ref */ ).into()).await?;

  // 7a. New governance_config row
  let cfg: GovernanceConfig = governance_config::table
    .filter(governance_config::id.eq(resp.config_id.unwrap() as i32))
    .select(GovernanceConfig::as_select())
    .first(conn)
    .await?;
  assert_eq!(cfg.key, "jury.panel_size");
  assert_eq!(cfg.value_int, Some(7));

  // 7b. Governance_log entry with byte-identical shell-shape payload
  let log_entry: GovernanceLog = governance_log::table
    .filter(governance_log::id.eq(resp.governance_log_id.unwrap()))
    .select(GovernanceLog::as_select())
    .first(conn)
    .await?;
  assert_eq!(log_entry.entry_kind, "admin_config_changed");
  let payload = log_entry.payload.as_object().unwrap();
  assert_eq!(payload["scope"].as_str(), Some("instance"));
  assert_eq!(payload["key"].as_str(), Some("jury.panel_size"));
  assert_eq!(payload["value_type"].as_str(), Some("int"));
  assert_eq!(payload["value"].as_i64(), Some(7));
  assert_eq!(payload["reason"].as_str(), Some("v1-AD-b happy-path test"));

  // 7c. Signature populated
  assert!(log_entry.signature.is_some());

  Ok(())
}
```

---

## 11. Files to change

| File | Action | Justification |
|---|---|---|
| `crates/api/api/src/governance/config.rs` | UPDATE | Add `CachedValue::Absent`, `get_int_opt/get_float_opt/get_bool_opt/get_text_opt` accessors; new parity test `rule_set_active_version_absent_returns_none` (task 2) |
| `crates/api/api/src/governance/admin_config.rs` | CREATE | `admin_set_config` + `admin_get_config` + `admin_get_config_audit` handlers (tasks 3 + 4 + 5); dry-run `impact_query_for_key` dispatcher (task 3) |
| `crates/api/api/src/governance/mod.rs` | UPDATE | `pub mod admin_config;` + `pub use admin_config::…` |
| `crates/api/api_common/src/governance.rs` | UPDATE | New DTOs: `AdminSetConfig`, `AdminSetConfigResponse`, `ConfigChangePreview`, `ConfigValueWithProvenance`, `AdminGetConfig`, `AdminGetConfigResponse`, `AdminConfigEntry`, `AdminGetConfigAudit`, `AdminConfigAuditEntry` (task 6) |
| `crates/api/routes/src/lib.rs` | UPDATE | 3 route registrations in `/governance/admin` scope; 3 new imports for the handlers (task 7) |
| `crates/server/tests/e2e.rs` | UPDATE | New integration tests: `admin_set_config_happy_path`, `admin_set_config_dry_run`, `admin_set_config_type_mismatch_rejected`, `admin_set_config_range_rejected`, `admin_set_config_enum_rejected`, `admin_set_config_non_admin_rejected_with_denial_log`, `admin_set_config_scope_mismatch_rejected`, `admin_set_config_community_scope_by_moderator`, `admin_get_config_full`, `admin_get_config_single_key_with_provenance`, `admin_get_config_audit_paginated`, `governance_log_payload_shell_parity` (task 8) |

Counts: **1 new Rust file, 4 modified Rust files, 1 updated test file. 0 new migrations, 0 new dependencies, 0 schema changes.**

---

## 12. NOT building in v1-AD-b

Explicitly out of scope (enforced by §16 acceptance criteria):

- **No new migrations.** The 4 from v1-AD-a cover the substrate. Any perceived need for a migration → DQ entry.
- **No `/admin/rule-sets` routes.** v1-AD-c.
- **No `/admin/dashboard` aggregate.** v1-AD-d.
- **No SSE `/admin/audit/stream`.** v1-AD-d (OQ-V1-AD-02 resolved to hand-rolled async-stream, but lands in v1-AD-d).
- **No askama HTML pages.** OQ-V1-AD-01 deferred to v1.x.
- **No step-up auth implementation.** v2; v1-AD-b only reads `requires_step_up` metadata + `governance.dashboard.step_up_enforced` flag and emits a denial log entry when blocked.
- **No new `ENTRY_KIND_*` consts.** The 2 from v1-AD-a are sufficient.
- **No `actix-web-lab`, `askama`, `maud`, or any new `Cargo.toml` deps.** Per v1-AD-a retro + OQ resolutions.
- **No `rule_set.active_version_id` seed.** v1-AD-c activates via first rule-set creation.
- **No payload-side JSONB filter pushdown.** v1-AD-b loads pre-filtered rows (by indexed `entry_kind` + `actor_pseudonym` + date) and post-filters in Rust. Profiling-driven optimization is future work.
- **No PM plugin-hook additions.** PM path untouched; §15 Level 6 verifies.
- **No apub changes.** Federation of config changes is v2+ scope per ADR-014.
- **No `dry_run_preview_log` table.** Per PRD §4.6 — dry-runs are not logged.
- **No `downstream_impact` in the governance_log payload.** Carried in HTTP response only; payload stays shell-identical for NOT5 gate 3.
- **No schedule/delayed `valid_from` writes.** `apply_at = "next_jury_cycle"` parses but inserts with `valid_from = now()` in v1-AD-b; delayed activation is v1-AD-b.1.

---

## 13. Step-by-step tasks

Execute in order. One commit per task. Branch: `phase-v1-AD-b` from `governance-v0`. Each task has a MIRROR reference, exact file path(s), and a validation command.

### Task 0: PRE-FLIGHT — branch + wrapper probes + baseline audit

- **Activity**: No commit; gate for tasks 1-8.
- **Actions**:
  1. Verify on `phase-v1-AD-b` (create from `governance-v0` if needed: `git switch -c phase-v1-AD-b governance-v0`).
  2. Verify `governance-v0` merged commit `e61f78edf` (PR #72) is in the ancestry: `git merge-base --is-ancestor e61f78edf HEAD`.
  3. Verify the OQ-V1-AD-03 resolution commit `3f0dd6572` is in the ancestry: same technique.
  4. Run all four `.claude/rules/pre-phase-harness-audit.md` probes per §1:
     ```bash
     cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/audit-cargo-check-p.log 2>&1"
     tail -20 .claude/audit-cargo-check-p.log
     cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/audit-cargo-check-features.log 2>&1"
     tail -20 .claude/audit-cargo-check-features.log
     cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1"
     tail -20 .claude/audit-cargo-test.log
     cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-test-negative.log 2>&1"
     echo "negative exit: $?"
     ```
  5. Verify `rg '^pub const ENTRY_KIND_ADMIN_CONFIG' crates/db_schema/src/source/governance/governance_log.rs` returns 2 entries.
  6. Verify `rg -n 'EXPECTED_SEED_COUNT_V1_AD' crates/api/api/src/governance/config.rs` returns the const at its definition + usage sites.
  7. Confirm `.claude/settings.local.json` is present (per `feedback_settings_local_json_worktree_bootstrap.md`). Copy from primary worktree if this is a fresh worktree.
- **DoD**: All four probes pass (including negative); ancestry confirmed; ENTRY_KIND consts present.
- **GOTCHA**: If baseline Level 5 clippy fails with pre-existing upstream debt unrelated to v1-AD-b, record it as drift-0 in the state file and proceed — per v1-AD-a retro §3 carry-forward #3.

### Task 1: ADD `CachedValue::Absent` variant in `config.rs`

- **ACTION**: Extend the `CachedValue` enum in `crates/api/api/src/governance/config.rs:137-142` with a new `Absent` variant:
  ```rust
  #[derive(Debug, Clone)]
  enum CachedValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Text(String),
    Absent,   // NEW: represents "fetched, no row, no const" — distinct from "not yet queried"
  }
  ```
- **MIRROR**: same file, the existing enum at lines 137-142.
- **GOTCHA**: The variant is `Absent` (unit variant), not `Optional(None)`. Keeps the enum shape flat + matches existing `Int(…) / Float(…) / …` rhyme.
- **GOTCHA**: Existing typed accessors (`get_int`, `get_float`, `get_bool`, `get_text`) MUST NOT read `CachedValue::Absent` without guarding — if they see `Absent` in the cache for a key they're fetching, they should still fall through to the const-default step (absent means "checked, no row"; the existing accessor then returns the const). Add a defensive match arm in each existing accessor to forward through to the const path when `CachedValue::Absent` is cached.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/phase-v1-AD-b-task1-check.log 2>&1"
  echo "exit: $?"
  ```
  **EXPECT**: 0.

### Task 2: ADD `get_int_opt` / `get_float_opt` / `get_bool_opt` / `get_text_opt` + parity test

- **ACTION**: In `crates/api/api/src/governance/config.rs` append below the existing `get_text` (line 292) and before the `// -- Membership state parser` heading. Four new public async functions.
- **IMPLEMENT**:
  ```rust
  pub async fn get_int_opt(
    cache: &mut ConfigCache,
    pool: &mut DbPool<'_>,
    scope: Scope,
    key: &str,
  ) -> LemmyResult<Option<i64>> {
    let scope_repr = scope.as_str();
    let cache_key = (scope_repr.as_ref().to_string(), key.to_string());

    if let Some(entry) = cache.entries.get(&cache_key) {
      return Ok(match entry {
        CachedValue::Int(v) => Some(*v),
        CachedValue::Absent => None,
        other => {
          return Err(LemmyErrorType::Unknown(format!(
            "governance_config key `{key}` requested as int_opt but stored as {other:?}"
          ))
          .into());
        }
      });
    }

    let result = match fetch_value(pool, scope, key).await? {
      Some(CachedValue::Int(v)) => Some(v),
      Some(other) => {
        return Err(LemmyErrorType::Unknown(format!(
          "governance_config key `{key}` requested as int_opt but stored as {other:?}"
        ))
        .into());
      }
      None => None,
    };

    let cached = match result {
      Some(v) => CachedValue::Int(v),
      None => CachedValue::Absent,
    };
    cache.entries.insert(cache_key, cached);
    Ok(result)
  }

  // Similar: get_float_opt, get_bool_opt, get_text_opt — same shape, different variants.
  ```
- **MIRROR**: `crates/api/api/src/governance/config.rs:162-193` (get_int).
- **GOTCHA**: Do NOT call `const_default_int(key)` in the opt variants. The whole point is to return `Ok(None)` instead of falling through. If the caller wants a default, they should call the non-opt `get_int` — which already does that.
- **GOTCHA**: Cache both the `Some(_)` and `None` outcomes. Caching only `Some(_)` means every repeat fetch of an absent key hits the DB — which defeats ConfigCache.
- **GOTCHA**: The test for `rule_set.active_version_id` absent → `Ok(None)` goes at the BOTTOM of `config.rs` in the existing `mod parity`:
  ```rust
  #[cfg(test)]
  mod parity {
    use super::*;
    // ... existing tests ...

    #[test]
    fn rule_set_active_version_not_in_seeded_keys() {
      // This key has NO seed row, NO const default — absence-of-row is the signal.
      assert!(
        !SEEDED_KEYS_WITH_CONSTS.iter().any(|(k, _, _)| *k == "rule_set.active_version_id"),
        "rule_set.active_version_id must NOT be seeded (v1-AD-a advisor edit #2)"
      );
      assert!(
        !CONFIG_KEY_METADATA.iter().any(|m| m.key == "rule_set.active_version_id"),
        "rule_set.active_version_id must NOT be in CONFIG_KEY_METADATA (v1-AD-a advisor edit #2)"
      );
      assert!(
        const_default_int("rule_set.active_version_id").is_none(),
        "rule_set.active_version_id must have NO Rust const default"
      );
    }
  }
  ```
  The e2e-level `rule_set_active_version_absent_returns_none` test goes in `crates/server/tests/e2e.rs` (task 8 — needs a live DB).
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/phase-v1-AD-b-task2-check.log 2>&1"
  echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat --workspace --lib parity > .claude/PRPs/debug/phase-v1-AD-b-task2-parity.log 2>&1"
  echo "parity exit: $?"
  tail -40 .claude/PRPs/debug/phase-v1-AD-b-task2-parity.log
  ```
  **EXPECT**: both 0; parity test `rule_set_active_version_not_in_seeded_keys` passes.

### Task 3: CREATE `crates/api/api/src/governance/admin_config.rs` with dry-run impact dispatcher

- **ACTION**: Create the new module `crates/api/api/src/governance/admin_config.rs` containing, at minimum in this task: the dry-run impact dispatcher and its read-only per-category functions. Handlers land in tasks 4 and 5; this task is scaffold + impact logic only, so that tasks 4-5 can import without editing.
- **IMPLEMENT**:
  - `pub struct DownstreamImpact` returning shape per PRD §4.3 table — heterogeneous by key category; a `serde_json::Value` return is fine
  - `pub async fn compute_downstream_impact(pool: &mut DbPool<'_>, key: &str, scope: Scope, current_value: &serde_json::Value, proposed_value: &serde_json::Value) -> LemmyResult<serde_json::Value>`
  - 7 per-category impl functions (`impact_for_threshold_key`, `impact_for_jury_panel_size`, `impact_for_jury_max_concurrent`, `impact_for_liability_sponsor_floor`, `impact_for_report_case_threshold_micros`, `impact_for_decay_key`, `impact_for_other`) following the PRD §4.3 table; each is READ-ONLY (no INSERT, no UPDATE, no tx).
  - Dispatch logic: match `key` prefix or exact match into a category:
    - `"thresholds.*"` → `impact_for_threshold_key`
    - `"jury.panel_size"` → `impact_for_jury_panel_size`
    - `"jury.max_concurrent_assignments"` → `impact_for_jury_max_concurrent`
    - `"liability.sponsor_liability_floor"` → `impact_for_liability_sponsor_floor`
    - `"report.case_threshold_micros"` → `impact_for_report_case_threshold_micros`
    - `"decay.*"` → `impact_for_decay_key` (returns the "no impact preview — decay is gradual" marker)
    - else → `impact_for_other` (returns `{estimated_first_effect_at: "next handler invocation"}`)
- **MIRROR**:
  - `crates/api/api/src/governance/admin_reputation_stats.rs:195-218` (`capability_query` uses `sql_query` + `get_result` for COUNT queries — shape v1-AD-b's impact queries after this)
  - `crates/db_views/governance_modlog/src/impls.rs:196-234` (governance_log filtered SELECT pattern)
  - For `jury.panel_size`: COUNT `moderation_case` rows WHERE `status IN ('JurySelection', 'InReview')` — see `crates/db_views/governance_case/src/impls.rs:55-70` for the `open_statuses` slice
  - For `jury.max_concurrent_assignments`: see `crates/api/api/src/governance/admin_assign_jury.rs:310-338` for the raw SQL query that excludes over-cap jurors; reuse the `jury_assignment.status IN ('Selected', 'Accepted')` predicate
- **GOTCHA**: Do NOT open `run_transaction` inside the impact functions. `fn(conn: &mut AsyncPgConnection)` or `fn(pool: &mut DbPool<'_>)` only — read-only, no BEGIN/COMMIT.
- **GOTCHA**: For `thresholds.*`: reputation_snapshot.jury_eligible is a DERIVED boolean (computed at snapshot time against the CURRENT threshold). To preview impact of a PROPOSED threshold, query the RAW `jury_reliability` / `reporting_accuracy` / `endorsement_strength` columns (i64 values), not the derived `jury_eligible` bool. The query shape is `COUNT(*) FILTER (WHERE raw_col >= $current) AS c_current, COUNT(*) FILTER (WHERE raw_col >= $proposed) AS c_proposed` — both in one sql_query call.
- **GOTCHA**: `impact_for_decay_key` is a no-op — just returns a `{"message": "Decay changes take effect at next reputation snapshot; no immediate count diff available."}` JSON object per PRD §4.3 row "decay.*".
- **GOTCHA**: Task 3 does NOT yet call this dispatcher anywhere — the call site is task 4 (`admin_set_config` handler). This task just lands the module + the pure-function impact logic so task 4 can import cleanly.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/phase-v1-AD-b-task3-check.log 2>&1"
  echo "exit: $?"
  ```
  **EXPECT**: 0. If `admin_config` module isn't yet imported from `mod.rs`, check will fail at the unused-module warning — add `pub mod admin_config;` to `crates/api/api/src/governance/mod.rs` in this task.

### Task 4: IMPLEMENT `admin_set_config` handler in `admin_config.rs`

- **ACTION**: In `crates/api/api/src/governance/admin_config.rs`, add the `admin_set_config` handler function + the `process_set_config` tx body.
- **IMPLEMENT** the full flow from §8 "Data flow — POST /admin/config happy path":
  1. Handler signature mirror of `admin_assign_jury`: `async fn admin_set_config(Json(data): Json<AdminSetConfig>, context: Data<LemmyContext>, local_user_view: LocalUserView) -> LemmyResult<Json<AdminSetConfigResponse>>`
  2. Capability check — LOOK UP the CONFIG_KEY_METADATA entry first; err 400 if key unknown
  3. Parse `data.scope`: `"instance"` or `r"community:(\d+)"` → `Scope::Instance` / `Scope::Community(CommunityId(n))`
  4. Policy dispatch:
     - `(Instance, Instance)` → `is_admin(&local_user_view)?`
     - `(Both, Instance)` → `is_admin(&local_user_view)?`
     - `(Both, Community(id))` → `CommunityModeratorView::check_is_community_moderator(pool, id, local_user_view.person.id).await?`
     - `(Community, Community(id))` → same community-moderator check
     - `(Instance, Community(id))` → emit `admin_config_change_denied` with `denial_reason: "scope_mismatch_instance_key"` + return `LemmyErrorType::NotAnAdmin` (HTTP 400 or 403, mirroring Lemmy shape)
     - `(Community, Instance)` → emit denial + return error
  5. Validate `data.value_type` matches metadata.value_type
  6. Validate value against metadata.valid_range (for Int/Float) or valid_enum (for Enum)
  7. Reject empty `data.reason`
  8. Step-up reserved slot: if `metadata.requires_step_up`:
     - Read `config::get_bool(&mut cache, pool, Scope::Instance, "governance.dashboard.step_up_enforced")` with default `false`.
     - If `true`: emit denial log with `denial_reason: "step_up_required"`, return `LemmyErrorType::NotAnAdmin` (403) with message `"v2: step-up auth not yet implemented; attempt logged"`.
     - If `false`: continue (advisory mode — attempt logged in the success path as usual; no separate log).
  9. Compute `compute_downstream_impact(pool, &data.key, scope, &current_value, &proposed_value)` — PRE-tx, pure read-only.
  10. Branch on `data.dry_run`:
      - `Some(true)` → return `AdminSetConfigResponse { applied: false, config_id: None, governance_log_id: None, preview: …, applied_at: None }`
      - `Some(false)` or `None` → proceed to tx
  11. Open `run_transaction`:
      - INSERT `GovernanceConfigInsertForm` with `updated_by: Some(admin_id)` + exactly one of `value_int / value_float / value_bool / value_text` populated
      - `governance_log::append(&mut conn.into(), ENTRY_KIND_ADMIN_CONFIG_CHANGED, payload, Some(admin_pseudonym))` — payload shape byte-identical to shell
      - Carry `config.id` and `log_entry.id` out of the closure
  12. Return `AdminSetConfigResponse { applied: true, config_id: Some(cfg.id.0), governance_log_id: Some(log.id), preview: …, applied_at: Some(now) }`
- **MIRROR**:
  - `crates/api/api/src/governance/admin_assign_jury.rs:63-87` (handler entry)
  - `crates/api/api/src/governance/admin_assign_jury.rs:91-122` (tx closure body + ConfigCache usage)
  - `crates/api/api/src/governance/admin_assign_jury.rs:165-187` (governance_log::append inside tx)
  - For the strict-vs-create actor_pseudonym lookup on denial path: `crates/api/api/src/governance/federation_outbox.rs:166-168` (uses `get` not `get_or_create`). For v1-AD-b denial path, use `get_or_create` since denied user may never have had one — see §4.1 load-bearing decision.
- **GOTCHA**: The payload for `admin_config_changed` MUST use `serde_json::json!` with the EXACT field order `{scope, key, value_type, value, reason}` — matches shell. A `ts_rs`-derived struct serialized via `serde_json::to_value` does NOT preserve field order reliably across serde versions; use `json!` inline.
- **GOTCHA**: The payload `"value"` field is the REHYDRATED typed value (i.e., if `value_type = "int"`, the payload should contain a JSON number `7`, not the string `"7"`). The shell script passes the raw value via psql var; v1-AD-b must ensure the JSON carries a typed number/bool/string to match.
- **GOTCHA**: Capability-check-denied MUST NOT open a `run_transaction` and MUST NOT insert into `governance_config`. It ONLY writes the denial entry to `governance_log`. Wrap just the `append` call in its own `run_transaction`-free invocation (append does its own transaction internally — see `governance_log.rs:193-230`).
- **GOTCHA**: For `ConfigScope::Both` community-scoped writes: after the capability check passes, verify the key accepts community scope in the policy dispatch. A key with `scope = ConfigScope::Instance` and a request with `scope: "community:42"` is always a denial, regardless of admin status.
- **GOTCHA**: `apply_at` field — v1-AD-b does NOT implement delayed `valid_from`. Always insert with `valid_from = now()`. Parse the field, store it in the log payload for future-audit, but don't act on it. Note in the log payload's shell-identity requirement: the shell script has NO `apply_at` field, so v1-AD-b's payload MUST NOT include `apply_at` either. Keep it in the HTTP response's `preview.downstream_impact.estimated_first_effect_at` but out of the governance_log payload.
- **GOTCHA (HARD)**: `ModerationCase` has 18 fields as of v1-AD-a task 5 fix. This plan's task 4 doesn't insert `ModerationCase` rows but if the impact queries or the integration test in task 8 construct a `ModerationCase` in-test, use `..Default::default()` — mirror of v1-AD-a fix commit `319b861fb`.
- **GOTCHA**: EmergencyRemove match — if any branch dispatches on `CaseStatus` (e.g., in `impact_for_jury_panel_size`), the match must be exhaustive: `CaseStatus::EmergencyRemove` branch must be handled. Per ADR-013, no `_ =>` fallthrough.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/phase-v1-AD-b-task4-check.log 2>&1"
  echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/phase-v1-AD-b-task4-testcompile.log 2>&1"
  echo "test-compile exit: $?"
  ```
  **EXPECT**: both 0. Per v1-AD-a retro #1 (test-target compile gate), the `--no-run` test-target compile is mandatory per-task from this plan onwards — catches struct-vs-schema drift that `cargo check --workspace` misses.

### Task 5: IMPLEMENT `admin_get_config` and `admin_get_config_audit` handlers

- **ACTION**: In `crates/api/api/src/governance/admin_config.rs`, add two more handler functions.
- **IMPLEMENT `admin_get_config`**:
  1. Signature: `async fn admin_get_config(Query(data): Query<AdminGetConfig>, context, local_user_view) -> LemmyResult<Json<AdminGetConfigResponse>>`
  2. Capability: `is_admin(&local_user_view)?` for now. (Community-admin-partial-view can ship in v1-AD-b.1 if requested.)
  3. If `data.key.is_some()`: single-key read. Use the `CONFIG_KEY_METADATA` entry + `fetch_value_at_scope` chain to return a single `AdminConfigEntry` with provenance.
  4. If `data.key.is_none()`: iterate `CONFIG_KEY_METADATA.iter()` and for each key call the cascade, building `effective_from: "community:<id>" | "instance" | "default"` based on which level returned the value.
  5. Return `AdminGetConfigResponse { entries: vec![...] }`.
- **IMPLEMENT `admin_get_config_audit`**:
  1. Signature: `async fn admin_get_config_audit(Query(data): Query<AdminGetConfigAudit>, context, local_user_view) -> LemmyResult<Json<Vec<AdminConfigAuditEntry>>>`
  2. Capability: `is_admin(&local_user_view)?`
  3. Clamp `page` ≥ 1, `limit` ∈ `[1, 100]`; compute `offset = (page - 1) * limit`.
  4. Build the Diesel query per §10.6. Filter `entry_kind IN ('admin_config_changed', 'admin_config_change_denied')`; optionally `actor_pseudonym.eq(...)`, `created_at.ge(since)`, `created_at.lt(until)`; ORDER BY `created_at DESC, id DESC` LIMIT+OFFSET.
  5. Load rows into `Vec<GovernanceLog>`, then for each, project the `payload` JSONB into `AdminConfigAuditEntry` fields (`scope`, `key`, `value_type`, `new_value`, `reason`, `denial_reason`). `previous_value` is always `None` for v1-AD-b (future: derive from the prior-row in the same scope+key by timestamp).
  6. Post-filter in Rust by `data.key.as_deref()` and `data.scope.as_deref()` by checking `payload["key"] == Some(k)` and `payload["scope"] == Some(s)`. Per §10.6 GOTCHA — JSONB pushdown deferred.
  7. Return `Json(entries)`.
- **MIRROR**:
  - `crates/api/api/src/governance/admin_reputation_stats.rs:76-218` (GET pattern with ConfigCache)
  - `crates/api/api/src/governance/list_cases.rs:25-43` (paginated GET)
  - `crates/api/api/src/governance/list_modlog.rs:30-49` (Rust-side page/limit clamp)
- **GOTCHA**: The full-read iterates the COMPILE-TIME `CONFIG_KEY_METADATA` array — 61+ reads per request. Use `ConfigCache` to memoize. Without the cache, the 61-read case is 61 round-trips × N handlers/sec — operator-UI latency issue. With the cache: first read per `(scope, key)` hits DB, rest are in-memory.
- **GOTCHA**: For the full-read `effective_from`, the logic is: first try `community:<id>` (if `community_id` provided), then `instance`, then `default`. The `fetch_value` function already does the fallback — but it doesn't report WHICH level succeeded. For v1-AD-b, run `fetch_value_at_scope` three times explicitly (community/instance/const) and report the provenance of the first success. Do NOT call `get_int` in a loop — that clobbers the provenance signal.
- **GOTCHA**: `AdminGetConfigAudit.since` and `.until` deserialize from ISO-8601 strings via serde+chrono. Accept `"2026-04-19T00:00:00Z"` and similar. Ensure the response's `created_at` serializes matching format for round-trip.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/phase-v1-AD-b-task5-check.log 2>&1"
  echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/phase-v1-AD-b-task5-testcompile.log 2>&1"
  echo "test-compile exit: $?"
  ```
  **EXPECT**: both 0.

### Task 6: ADD DTOs in `crates/api/api_common/src/governance.rs`

- **ACTION**: Append the 9 new DTOs listed in §10.7 to `crates/api/api_common/src/governance.rs`. Place alphabetically within the admin DTO block starting at line 143 (after `AdminCloseCaseResponse`).
- **IMPLEMENT**: Exact struct shapes per §10.7 patterns. Use `#[skip_serializing_none]`, `#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]`, `#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]`.
- **MIRROR**: `crates/api/api_common/src/governance.rs:143-187` (existing admin DTOs). Import newtypes: `use lemmy_db_schema::newtypes::CommunityId;`.
- **GOTCHA**: Do NOT add `derive(Copy)` to any of these — `Vec<…>`, `String`, `serde_json::Value` are not `Copy`. Use `Clone, Default, PartialEq` (drop `Eq, Hash` because `f64` tuples and `serde_json::Value` don't implement them).
- **GOTCHA**: `ConfigValueWithProvenance.value: serde_json::Value` — `Value` doesn't derive `Default`, but `ConfigValueWithProvenance` does via `#[derive(Default)]`. Replace `Default` with a manual `impl Default for ConfigValueWithProvenance` if compile fails — `Default` for `Value` returns `Value::Null`, which is acceptable.
- **GOTCHA**: `AdminSetConfig.apply_at: Option<String>` — string-typed with expected values `"immediate" | "next_jury_cycle" | "next_snapshot_job"`. The `ApplyAt` enum in `config.rs:90-95` could be serde-derived, but for API simplicity keep it as `Option<String>` and parse in the handler.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_common --features full > .claude/PRPs/debug/phase-v1-AD-b-task6-check.log 2>&1"
  echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-clippy.bat -p lemmy_api_common --features full --no-deps -- -D warnings > .claude/PRPs/debug/phase-v1-AD-b-task6-clippy.log 2>&1"
  echo "clippy exit: $?"
  ```
  **EXPECT**: both 0.

### Task 7: WIRE routes in `crates/api/routes/src/lib.rs`

- **ACTION**: Modify `/governance/admin` scope per §10.8.
- **IMPLEMENT**:
  1. Add three imports at the existing `governance::` block (around line 37): `admin_config::{admin_set_config, admin_get_config, admin_get_config_audit}`.
  2. Modify the scope block from line 531-536 to add the three new routes (per §10.8).
- **MIRROR**: `crates/api/routes/src/lib.rs:531-536` (existing scope). The new scope `config` nests: one `POST ""`, one `GET ""`, one `GET "/audit"`.
- **GOTCHA**: `scope("/config").route("", get().to(...))` correctly maps `GET /admin/config` (no trailing `/`). The empty path `""` is actix's idiom for "match the scope root".
- **GOTCHA**: Do NOT break the existing `/admin/assign-jury`, `/admin/close-case`, `/admin/reputation-stats` routes. Tests in §15 Level 7 (route sweep at e2e.rs:2348-2418) verify the existing paths still return non-404.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/phase-v1-AD-b-task7-check.log 2>&1"
  echo "exit: $?"
  ```
  **EXPECT**: 0.

### Task 8: ADD integration tests in `crates/server/tests/e2e.rs`

- **ACTION**: Append to `crates/server/tests/e2e.rs` a new `#[tokio::test]` block for each of the 12 tests enumerated in §11. Mirror the existing admin-endpoint test shape (lines 979-1025).
- **IMPLEMENT** at minimum these 12 tests — each a stand-alone `#[tokio::test]` with full setup (`start_postgres`, `apply_all_schema`, seed users, call handler, assert DB state):

  | Test name | Covers |
  |---|---|
  | `admin_set_config_happy_path` | Instance-scope int write (bump `jury.panel_size` to 7) → 200 + config_id + log_id + preview; DB row appended; governance_log byte-identical to shell |
  | `admin_set_config_dry_run` | Same write with `dry_run: true` → 200 with `applied: false`, `config_id: None`; NO new config row; NO new log entry |
  | `admin_set_config_type_mismatch_rejected` | Request `value_type: "int"` but metadata says float → 400 + denial log |
  | `admin_set_config_range_rejected` | Request `jury.panel_size = 1000` (out of range 3-21) → 400 + denial log |
  | `admin_set_config_enum_rejected` | Request `jury.severity_thresholds.minor = "invalid"` (not in valid_enum) → 400 + denial log |
  | `admin_set_config_non_admin_rejected_with_denial_log` | Non-admin caller → 403 + denial log with `denial_reason: "instance_admin_required"` |
  | `admin_set_config_scope_mismatch_rejected` | Instance-scope key (e.g., `federation.inbound_advisory_only`) with `scope: "community:42"` → 400 + denial log with `denial_reason: "scope_mismatch_instance_key"` |
  | `admin_set_config_community_scope_by_moderator` | Community-scoped key (`liability.regular_multiplier`, scope `Both`) with `scope: "community:42"` + moderator caller → 200 + row appended |
  | `admin_get_config_full` | GET without filters → 61+ entries in response; each with correct `effective_from` |
  | `admin_get_config_single_key_with_provenance` | GET with `?key=jury.panel_size` after `admin_set_config` write → entry shows `effective_from: "instance"` |
  | `admin_get_config_audit_paginated` | After 3 successful writes + 2 denied writes, GET audit with `limit=3` → 3 entries in desc created_at order |
  | `governance_log_payload_shell_parity` | Write via HTTP AND write via raw SQL (mimicking the shell script). Diff the two `payload` JSONB columns — must be byte-identical field order + values. (NOT5 gate 3 proof.) |
  | `rule_set_active_version_absent_returns_none` | After full migrations + no seed row + no code default, `config::get_int_opt(Scope::Instance, "rule_set.active_version_id")` returns `Ok(None)`. (Advisor edit #2 / v1-AD-a retro.) |

- **MIRROR**:
  - `crates/server/tests/e2e.rs:979-1025` (admin_assign_jury test shape — direct handler invocation, DB assertions)
  - `crates/server/tests/e2e.rs:1360-1418` (config_parity_round_trip — full setup template)
  - `crates/server/tests/e2e.rs:2348-2418` (route sweep — for the community-moderator test seed a community + register moderator via `CommunityModerator::create`)
- **GOTCHA**: Rate-limit bucket — per memory `feedback_rate_limit_debug_config_post_bucket.md`. Multi-endpoint tests (especially `admin_get_config_audit_paginated` which writes 5 times and reads) WILL trip the 6/300s Post bucket. Mirror the `set_config` pattern at e2e.rs:2348-2364 to bump buckets to 10_000/60s.
- **GOTCHA**: `ModerationCase` is at 18 cols (since v1-AD-a fix). Tests that seed cases must use `ModerationCaseInsertForm { ..Default::default(), ...explicit fields }`.
- **GOTCHA**: Docker run — use the `start_postgres()` helper (via testcontainers). Don't run docker directly; the testcontainers integration handles cleanup including `--user $(id -u):$(id -g)` equivalent on Linux. On Windows (this is a Windows-primary repo) cleanup is done via testcontainers regardless.
- **GOTCHA**: Tests in the `#[tokio::test]` form + the lint `tests_outside_test_module` is DENIED at workspace level. Per v1-AD-a retro: 100 pre-existing violations live in `tests/e2e.rs` already, so the lint is a baseline not gated by this fork's GH Actions — new tests SHOULD follow the pattern that doesn't trigger the lint (tests inside a `mod governance_admin_config_tests { ... }` block or similar; check the existing convention). If the existing e2e.rs pattern is `#[tokio::test]` at file scope, follow it (carry-forward #3 from v1-AD-a retro notes this is a pre-existing baseline issue).
- **GOTCHA**: `governance_log_payload_shell_parity` test is the proof of NOT5 gate 3. Write via HTTP handler, then write via raw SQL matching the shell script's exact INSERT at `admin-config-write.sh:146-157`. Load both rows. Compare `payload` JSONB object keys IN ORDER (using `payload.as_object().unwrap().iter().collect::<Vec<_>>()` and checking the key sequence). The `actor_pseudonym`, `signature`, `entry_hash`, `prev_hash` WILL differ between rows — only `entry_kind` + `payload` must match.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/phase-v1-AD-b-task8-testcompile.log 2>&1"
  echo "test-compile exit: $?"
  # With a live Docker Postgres, run the new tests (NB: requires docker running):
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_set_config_happy_path > .claude/PRPs/debug/phase-v1-AD-b-task8-happy.log 2>&1"
  echo "happy exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_ > .claude/PRPs/debug/phase-v1-AD-b-task8-all.log 2>&1"
  echo "all exit: $?"
  ```
  **EXPECT**: test-compile 0. If Docker available: all 12 tests pass. If Docker unavailable in session, document which tests were deferred in the completion report.

### Task 9: ADD parity test `admin_config_payload_no_apply_at` + extension test comment

- **ACTION**: In `crates/api/api/src/governance/admin_config.rs` at the bottom, add a unit test module:
- **IMPLEMENT**:
  ```rust
  #[cfg(test)]
  mod payload_parity {
    use super::*;
    use serde_json::json;

    /// admin_config_changed payload must NOT include apply_at (byte-identity
    /// contract with scripts/brehon/admin-config-write.sh:146-157 for NOT5
    /// gate 3).
    #[test]
    fn payload_has_no_apply_at_field() {
      let payload = json!({
        "scope": "instance",
        "key": "jury.panel_size",
        "value_type": "int",
        "value": 7,
        "reason": "test",
      });
      assert!(!payload.as_object().unwrap().contains_key("apply_at"));
    }

    /// Field order matters — serde_json::json! preserves macro-literal order;
    /// the resulting Value serializes keys in declaration order. This test
    /// captures the expected order.
    #[test]
    fn payload_field_order_matches_shell() {
      let payload = json!({
        "scope": "instance",
        "key": "jury.panel_size",
        "value_type": "int",
        "value": 7,
        "reason": "test",
      });
      let keys: Vec<&String> = payload.as_object().unwrap().keys().collect();
      assert_eq!(
        keys.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        vec!["scope", "key", "value_type", "value", "reason"],
      );
    }
  }
  ```
- **MIRROR**: `crates/api/api/src/governance/config.rs:1455-1527` (parity tests).
- **GOTCHA**: `serde_json::Value` objects preserve insertion order when using the `preserve_order` feature. Check `Cargo.toml` for `serde_json = { version = "…", features = ["preserve_order"] }`. If absent, the `payload_field_order_matches_shell` test may be flaky under `serde_json`'s default BTreeMap ordering. Add `features = ["preserve_order"]` to the workspace `serde_json` dep if not present — but this is a Cargo.toml edit that v1-AD-b §12 says NOT to do. Check first: if `preserve_order` is already enabled in the workspace, the test is safe. Otherwise, either drop the ordering test or surface this as a DQ.
- **GOTCHA**: The `payload_has_no_apply_at_field` test is more important than the ordering test — it locks the shell-identity contract regardless of ordering flake. The ordering test is a belt-and-braces bonus; if `preserve_order` isn't on, skip the ordering test with a `#[ignore]` attribute and leave a TODO.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_api --lib payload_parity > .claude/PRPs/debug/phase-v1-AD-b-task9-parity.log 2>&1"
  echo "exit: $?"
  ```
  **EXPECT**: 0.

---

## 14. Testing strategy

Per [IMPLEMENTATION-PLAN-v0.md §5](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): **integration-only** for Rust handler behaviour, plus compile-time unit parity tests for shape invariants. All tests live in either `crates/server/tests/e2e.rs` (integration) or `crates/api/api/src/governance/**.rs` `#[cfg(test)]` modules (parity).

### Integration tests to add (task 8)

| Test Name | What It Validates | Type |
|---|---|---|
| `admin_set_config_happy_path` | Instance scope write appends row + log | e2e |
| `admin_set_config_dry_run` | `dry_run=true` suppresses write | e2e |
| `admin_set_config_type_mismatch_rejected` | `value_type` mismatch → 400 + denial log | e2e |
| `admin_set_config_range_rejected` | Out-of-range value → 400 + denial log | e2e |
| `admin_set_config_enum_rejected` | Invalid enum value → 400 + denial log | e2e |
| `admin_set_config_non_admin_rejected_with_denial_log` | Non-admin → 403 + denial log | e2e |
| `admin_set_config_scope_mismatch_rejected` | Instance key with community scope → 400 + denial log | e2e |
| `admin_set_config_community_scope_by_moderator` | `Both`-scope key via community moderator → 200 + row | e2e |
| `admin_get_config_full` | GET all → 61+ entries with provenance | e2e |
| `admin_get_config_single_key_with_provenance` | GET one → provenance correct after write | e2e |
| `admin_get_config_audit_paginated` | Audit filter + pagination works | e2e |
| `governance_log_payload_shell_parity` | HTTP + SQL writes produce byte-identical payload | e2e (NOT5 gate 3) |
| `rule_set_active_version_absent_returns_none` | `get_int_opt` returns Ok(None) for absent+no-const key | e2e (advisor edit #2) |

### Parity tests (tasks 2 + 9)

| Test Name | File | What It Validates |
|---|---|---|
| `rule_set_active_version_not_in_seeded_keys` | `crates/api/api/src/governance/config.rs` | `rule_set.active_version_id` absent from all three compile-time tables |
| `payload_has_no_apply_at_field` | `crates/api/api/src/governance/admin_config.rs` | `admin_config_changed` payload excludes apply_at |
| `payload_field_order_matches_shell` | `crates/api/api/src/governance/admin_config.rs` | Field order `{scope, key, value_type, value, reason}` |

### Edge cases covered

- [x] Type mismatch (value_type ≠ metadata.value_type) — task 8 `admin_set_config_type_mismatch_rejected`
- [x] Range violation — task 8 `admin_set_config_range_rejected`
- [x] Enum violation — task 8 `admin_set_config_enum_rejected`
- [x] Scope mismatch (Instance key + Community request) — task 8 `admin_set_config_scope_mismatch_rejected`
- [x] Non-admin caller — task 8 `admin_set_config_non_admin_rejected_with_denial_log`
- [x] Community moderator on `Both`-scope key — task 8 `admin_set_config_community_scope_by_moderator`
- [x] `requires_step_up = true` + `step_up_enforced = false` (advisory) — see §13 task 4 GOTCHA
- [x] `requires_step_up = true` + `step_up_enforced = true` (blocking) — NOT a test in this phase; v2 reserved. Document as a carry-forward test for v2.
- [x] Hash-chain integrity still holds — implicitly tested by `happy_path` + `dry_run` tests that query `governance_log.signature IS NOT NULL`
- [x] `actor_pseudonym` generated for non-admin denial path — tested in `non_admin_rejected_with_denial_log`
- [x] `EmergencyRemove` match exhaustiveness — verified at compile time if any code path matches `CaseStatus`; test not needed (ADR-013)
- [x] Redaction: no `person_id`, username, or email in payload — verified by `governance_log_payload_shell_parity` (shell script never includes those either)

### Test coverage acceptance gate (§16 #N)

- All 13 e2e tests pass under `cargo-test.bat --test e2e`
- All 3 parity tests pass under `cargo-test.bat --workspace --lib`
- `cargo-test.bat --test e2e --no-run -p lemmy_server` exits 0 (compile gate, NEW per v1-AD-a retro)

---

## 15. Validation commands (DoD)

Adopt the v1-AD-a Level 1–6 scheme PLUS Level 7 (route sweep) PLUS the test-target compile gate per v1-AD-a retro carry-forward #1.

### Level 1: Per-task `cargo check --workspace` (foundation)

Run after EVERY task:
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace > .claude/PRPs/debug/phase-v1-AD-b-task${N}-L1.log 2>&1"
STATUS=$?
tail -20 .claude/PRPs/debug/phase-v1-AD-b-task${N}-L1.log
echo "exit: $STATUS"
```
**EXPECT**: 0. Foundation — catches lifetime, trait-bound, async-closure, and moved-value errors.

### Level 2: `cargo check --workspace --features full`

Run after EVERY task (mandatory — `full` gates `ts_rs` and other derives):
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/phase-v1-AD-b-task${N}-L2.log 2>&1"
STATUS=$?
tail -30 .claude/PRPs/debug/phase-v1-AD-b-task${N}-L2.log
echo "exit: $STATUS"
```
**EXPECT**: 0. **Never use `-p lemmy_server --features full`** — lemmy_server doesn't declare the feature per `feedback_features_full_workspace_only.md`.

### Level 3: Parity unit tests

After task 2, 9, and as a per-task sanity check thereafter:
```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --lib parity > .claude/PRPs/debug/phase-v1-AD-b-task${N}-L3.log 2>&1"
STATUS=$?
tail -40 .claude/PRPs/debug/phase-v1-AD-b-task${N}-L3.log
echo "exit: $STATUS"
```
**EXPECT**: 0. Existing parity tests (`seeded_keys_count_matches_const_count`, `every_seeded_key_has_metadata`, `every_seeded_key_has_const_fallback`, NEW `rule_set_active_version_not_in_seeded_keys`, NEW `payload_has_no_apply_at_field`, NEW `payload_field_order_matches_shell`) all pass.

### Level 4: Test-target compile (per v1-AD-a retro carry-forward #1 — NEW PER-TASK GATE)

Run after EVERY task from 4 onwards. This is the fix for the v1-AD-a task 5 gap where `cargo check --workspace` passed but `cargo test --test e2e --no-run` would have caught the Queryable/Insertable struct mismatch:
```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/phase-v1-AD-b-task${N}-L4.log 2>&1"
STATUS=$?
tail -30 .claude/PRPs/debug/phase-v1-AD-b-task${N}-L4.log
echo "exit: $STATUS"
```
**EXPECT**: 0. Cold: ~13 minutes on Windows per v1-AD-a report. Warm: seconds.

### Level 5: e2e integration tests (requires Docker)

After task 8, with Docker Desktop running:
```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server config_parity_round_trip > .claude/PRPs/debug/phase-v1-AD-b-L5-round-trip.log 2>&1"
echo "round-trip exit: $?"
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_ > .claude/PRPs/debug/phase-v1-AD-b-L5-admin.log 2>&1"
echo "admin exit: $?"
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server governance_log_payload_shell_parity > .claude/PRPs/debug/phase-v1-AD-b-L5-shell-parity.log 2>&1"
echo "shell-parity exit: $?"
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server rule_set_active_version_absent_returns_none > .claude/PRPs/debug/phase-v1-AD-b-L5-opt.log 2>&1"
echo "opt exit: $?"
```
**EXPECT**: all 0. If Docker unavailable in session, defer to next session and document which tests were run when.

### Level 6: Clippy with deny-warnings (per plan-drift CI)

After all tasks, mandatory before PR:
```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/phase-v1-AD-b-L6-clippy.log 2>&1"
STATUS=$?
tail -40 .claude/PRPs/debug/phase-v1-AD-b-L6-clippy.log
echo "exit: $STATUS"
```
**EXPECT**: 0. Use `cargo-clippy.bat` (NOT `cargo-check.bat` — per memory `feedback_clippy_vs_check_wrapper.md`). `--no-deps` avoids inherited upstream lint debt per `.claude/rules/pre-phase-harness-audit.md` DoD-footgun #3.

### Level 7: PM-hook integrity (belt-and-braces)

Mandatory per-phase:
```bash
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  rg -q "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done
echo "all PM hooks present"
```
**EXPECT**: prints "all PM hooks present". v1-AD-b doesn't touch PM code but the per-phase guard is mandatory per `.claude/rules/pm-plugin-hooks-stable.md`.

### Level 8: Route sweep (non-404 assertion)

After task 7, per v1-AD-a retro carry-forward via existing e2e.rs:2348-2418 sweep. If the existing test hits all three new routes (or is extended to), it verifies the route registration didn't break any existing path.

### Level 9: Governance-log entry-kind registry + pre-phase audit byte-check

```bash
# Registry unchanged — v1-AD-b adds no new kinds
grep -c '^\s*ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs
# Expect: >= 25 (the shim re-exports; the canonical 25 constants at v1-AD-a tip)

# Registry file total-kinds invariant:
rg '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs | wc -l
# Expect: 25 (unchanged from v1-AD-a)
```
**EXPECT**: all counts unchanged; registry file `.claude/rules/governance-log-entry-kind-registry.md` total-kind count still 25.

---

## 16. Acceptance criteria

- [ ] All 9 tasks (Tasks 0-9, with Task 0 non-commit) complete and committed on `phase-v1-AD-b`
- [ ] Branch is 9 commits ahead of `governance-v0` (Task 0 no commit; corrections land as additional commits per v1-AD-a retro wording change)
- [ ] Levels 1/2/3 exit 0 after every task
- [ ] Level 4 (test-target compile) exit 0 after every task from 4 onwards — NEW per v1-AD-a retro carry-forward #1
- [ ] Level 5 (integration tests) exit 0 for all 13 new tests OR a deferral note in the completion report with a re-run commitment before PR merge
- [ ] Level 6 clippy `--features full --no-deps -- -D warnings` exit 0
- [ ] Level 7 PM-hooks all present
- [ ] Level 9 governance-log entry-kind registry count unchanged at 25
- [ ] `CachedValue::Absent` variant added to `config.rs`
- [ ] 4 new accessors (`get_int_opt`, `get_float_opt`, `get_bool_opt`, `get_text_opt`) added to `config.rs`
- [ ] Parity test `rule_set_active_version_not_in_seeded_keys` passes (asserts key absent from SEEDED_KEYS_WITH_CONSTS, CONFIG_KEY_METADATA, and const_default_int)
- [ ] 9 new DTOs added to `crates/api/api_common/src/governance.rs`
- [ ] 3 new handlers (`admin_set_config`, `admin_get_config`, `admin_get_config_audit`) in `crates/api/api/src/governance/admin_config.rs`
- [ ] Dry-run impact dispatcher `compute_downstream_impact` in same file with 7 per-category functions, none opening transactions
- [ ] 3 new routes registered in `crates/api/routes/src/lib.rs:531-536` `/governance/admin` scope
- [ ] `admin_config_changed` payload byte-identical field order `{scope, key, value_type, value, reason}` to shell script — proved by `governance_log_payload_shell_parity` test
- [ ] `admin_config_change_denied` entry emitted on every capability/type/range/enum/scope violation
- [ ] `actor_pseudonym` populated on every denial entry (via `get_or_create` — acceptable even for non-admin first-interaction)
- [ ] No new migrations
- [ ] No new `Cargo.toml` dependencies
- [ ] No askama, maud, actix-web-lab, async-stream additions
- [ ] No new `ENTRY_KIND_*` consts (the 2 from v1-AD-a are sufficient)
- [ ] No ActivityPub (`crates/apub/`) changes
- [ ] No PM hook additions
- [ ] No contradictions with ADRs 1-15, especially ADR-010 (staged releases — no retroactive invalidation, respected via `apply_at` parse-only), ADR-013 (EmergencyRemove exhaustive match — any `CaseStatus` match in impact queries is exhaustive), ADR-015 (pseudonym write-path — denial path uses get_or_create correctly)
- [ ] Retro written at `.claude/PRPs/reports/phase-v1-AD-b-retro.md` BEFORE PR merge (per `feedback_retro_not_report.md`)
- [ ] Complete-report at `.claude/PRPs/reports/phase-v1-AD-b-complete-report.md` summarising commits + validation + drifts
- [ ] PR opened via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-AD-b` per `.claude/rules/gh-pr-fork-target.md`
- [ ] PR merged with `--merge` (not `--squash`) — preserves task-per-commit history for CodeRabbit review
- [ ] If CodeRabbit flags any Critical (🔴) findings, fix IN-PR per `feedback_coderabbit_block_merge_critical.md`

---

## 17. Completion checklist

- [ ] Task 0 PRE-FLIGHT — branch verified, wrapper probes pass, ancestry confirms v1-AD-a + OQ-V1-AD-03
- [ ] Task 1 — `CachedValue::Absent` variant
- [ ] Task 2 — `get_*_opt` accessor family + `rule_set_active_version_not_in_seeded_keys` parity
- [ ] Task 3 — `admin_config.rs` module scaffold + dry-run impact dispatcher + 7 per-category queries
- [ ] Task 4 — `admin_set_config` handler with capability/validation/dry-run/tx
- [ ] Task 5 — `admin_get_config` + `admin_get_config_audit` handlers
- [ ] Task 6 — 9 new DTOs in api_common
- [ ] Task 7 — routes wired in `crates/api/routes/src/lib.rs`
- [ ] Task 8 — 13 integration tests in `tests/e2e.rs`
- [ ] Task 9 — 2 payload-parity unit tests + shell-byte-identity assertion
- [ ] Acceptance criteria (§16) pass
- [ ] PR #TBD open against `governance-v0` with CodeRabbit auto-review triggered
- [ ] Retro at `.claude/PRPs/reports/phase-v1-AD-b-retro.md`

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| serde_json `preserve_order` feature not enabled → payload field-order test flakes | MED | LOW | Task 9 GOTCHA — check workspace Cargo.toml. If not enabled, mark `payload_field_order_matches_shell` as `#[ignore]` + TODO; `payload_has_no_apply_at_field` is the load-bearing test. The shell-parity e2e test is the ultimate proof — it diffs actual DB rows, not Rust structs. |
| Dry-run impact queries drift between pre-tx snapshot and post-commit state (race) | MED | LOW | Per OQ-V1-AD-03 resolution: pre-tx snapshot IS the contract. Another concurrent write between impact-query and tx-open is acceptable behaviour. Test the intent (dry-run correctness at snapshot time), not the impossible (always-fresh post-tx count). |
| Capability-check-denied `get_or_create` creating pseudonyms for non-admin users leaks intent | LOW | MED | `actor_pseudonym` is the GDPR layer; it already holds every person who has touched governance state. Creating a row on first denial is correct GDPR behaviour (you now HAVE a governance-relevant interaction to log). Documented in §4.1. |
| Payload-shell-parity test breaks if shell script changes | LOW | MED | The shell script is explicitly scheduled for deprecation per NOT5 §8.4. v1-AD-b is the replacement. If the shell changes AFTER v1-AD-b ships, the parity test will fail, signalling "shell and HTTP diverged" — desirable signal. |
| `admin_get_config_audit` post-filtering JSONB payload-side in Rust is slow for high-volume logs | LOW | LOW | Per PRD scope, audit volume is low (<1000/day). Profile later; push down to SQL `@>` if needed. Not a v1 blocker. |
| `admin_get_config` full-read makes 61+ DB calls without ConfigCache | MED | MED | Task 5 GOTCHA mandates ConfigCache use. Per-request instantiation is the caching layer. Without cache: measurable; with: 1 DB round-trip per (scope, key) first-hit, 0 round-trips thereafter. |
| `rule_set.active_version_id` accessor mis-use — implementer writes `get_int` (erroring) instead of `get_int_opt` | MED | HIGH | §4.1 load-bearing + task 2 GOTCHA + `rule_set_active_version_absent_returns_none` e2e test assert `Ok(None)`. If someone writes `get_int`, the e2e test fails loudly at task 8. |
| `ModerationCase` struct grows to 19+ columns between v1-AD-a and v1-AD-b-impl-window | LOW | MED | Per v1-AD-a retro learning: `..Default::default()` is the pattern. Grep-check before task 8 tests that `ModerationCaseInsertForm` struct shape is still 18 cols (post-v1-AD-a fix commit). |
| CodeRabbit flags the payload shell-identity as "fragile coupling" | MED | LOW | Acknowledged coupling — NOT5 §8.4 explicitly requires byte-identity for gate 3 (audit-trail continuity across shell→HTTP deprecation). Reply to CodeRabbit with the NOT5 link. |
| Cold test compile (13+ min) eats into per-iteration budget | HIGH | LOW | Warm subsequent runs (seconds). Task 0 triggers the cold compile (positive probes). Subsequent tasks run in the warm cache window. Schedule task 8 (test additions) for an iteration with enough budget for cold+re-compile. |
| Upstream rebase lands between v1-AD-a and v1-AD-b start → governance_config_current view shape changes | LOW | HIGH | `.claude/rules/pre-phase-harness-audit.md` task 0 validates; if drift detected, treat as a /prp-debug trigger per CLAUDE.md. |
| Wrapper script exit-code masking resurfaces | LOW | HIGH | Task 0's negative probe (`--features nonexistent_xyz`) catches the bug class per `.claude/rules/pre-phase-harness-audit.md` probe 4. |
| 13 e2e tests compile but fail to run due to Docker issues in dev session | MED | MED | §15 Level 5 acknowledges and allows deferral + re-run-before-merge. Not a plan blocker; is a PR-merge blocker. |
| Step-up reserved slot tests create noise | LOW | LOW | v1-AD-b DOES NOT implement step-up blocking — only the advisory-mode log + reserved 403 body. Carry-forward-test list for v2 documents what needs testing later. |

---

## 19. Notes

- v1-AD-b is the **heaviest-by-LOC-but-not-by-complexity** sub-phase. It lands 3 handlers + 4 accessors + dry-run dispatcher + 9 DTOs + 13 tests but no schema or cross-cutting invariant changes. Each task is self-contained; dependencies flow 1→2→3→(4,5 parallel)→6→7→8→9. v1-AD-a retro #5 (task 5 Rust-side gap) drives the Level 4 per-task test-target compile gate.
- The shell-script deprecation gate (NOT5) has three conditions. This plan satisfies condition 1 (endpoint ships) via task 4 + task 7. Condition 3 (byte-identical log payload) via task 8 `governance_log_payload_shell_parity` test + task 9 parity unit tests. Condition 2 (pilot coverage of all key classes) is pilot-operational, not plan-reachable — scheduling is §8.4 of the PRD.
- `rule_set.active_version_id` absence-of-row pattern is architecturally load-bearing for v1-AD-c (which reads this key). If the `get_int_opt` test drifts, v1-AD-c handlers could crash on absent-key reads. Task 2's parity test + task 8's e2e test together lock the contract.
- No new OQs, no new ADRs, no new ENTRY_KIND consts. Scope is deliberately narrow — complexity concentrated in dispatching capability checks + per-key-category impact queries, both pattern-rich mirrors of existing code.
- v1-AD-b is the first sub-phase to adopt the test-target compile gate per-task (v1-AD-a retro carry-forward #1). If this pattern holds, it should land as an amendment to `.claude/rules/pre-phase-harness-audit.md` §4 "Test-target compile gate" for all future phases.
- If CodeRabbit surfaces Critical findings during PR review, fix-in-PR per `feedback_coderabbit_block_merge_critical.md`. Pattern established 3× now (PR #4, PR #46 #15, PR #46 #19). Local advisor + e2e have demonstrated they cannot catch the class of bug CodeRabbit catches (row-scoping, dup-federation, actor-binding) — so expect 1-2 Critical findings on a 9-commit PR and budget time for them.

---

**END OF v1-AD-b PLAN**
