# RT-r4 planning brief

**Written**: 2026-05-29 by advisor session (lane worktree `C:\Users\barri\Developer\brehon-fork-rt-r4` on `phase-v1-RT-r4`, Mode A) via `/auto-roadmap` plan-gap handler.

**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`).

**Worktree**: Junior cuts `junior/rt-r4-planning-1` from `phase-v1-RT-r4` per concurrency-1 default. The plan file commits and pushes to `phase-v1-RT-r4` at finalize.

**Authority anchor**: `v1-reputation-tuning.prd.md` §11 row 4 (`v1.r4 — sponsor-gate strategies`) + §5.4 (sponsor gate-strategy expansion — read verbatim) + §7 (Cross-Cutting Impact — the `sponsor_allowlist` add/remove `ENTRY_KIND` pair) + §10 (Security — admin-only endpoints, both add+remove emit governance_log) + [99 OQ-020](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (three-strategy expansion, composition strategies rejected). PRD §11 row 4 dependency: **v1.r1 (allowlist table)** — verified shipped in v1-RT-r1 (`sponsor_allowlist` table + struct + two `ENTRY_KIND_*` consts all present on the phase branch — see §0 dependency-reality check below). Dependency satisfied.

**Lane mode**: A (dedicated lane worktree on the laptop). Briefs author directly on the phase branch; validate-pending runs locally (Shape G suspended until 2026-06-01 — `validate-pending-laptop`).

**MiniMax M2.7-vs-Sonnet A/B trial — ARMED for this phase.** Per `.claude/PRPs/briefs/minimax-m27-trial-1.md` §3: when this plan's §13 lands, the advisor designates **5 MIRROR-ref-heavy, single-file, cargo-gated §13 tasks** as trial tasks (sponsor-allowlist add/remove handlers + their `ENTRY_KIND` emits + e2e tests are the natural candidates — they mirror 3 prior shipped RT sub-phases' admin-handler + capability-check + `ENTRY_KIND`-emit patterns). **Planner action**: author §13 tasks that are maximally MIRROR-ref-shaped and single-file where the dependency chain allows (so the trial has clean comparison material). The trial runs ALONGSIDE the real phase on throwaway `ab-test/*` branches; it does NOT gate v1-RT-r4 shipping and is NOT part of the plan. The real plan stays Sonnet-targeted (`target_model` default); the trial re-runs the same tasks under MiniMax separately. Do NOT set `target_model: minimax-m2.7` in this plan.

---

## 0. Dependency-reality check (advisor-verified 2026-05-29, before brief author)

Per `feedback_runbook_audit_drift_post_event_check.md` + `feedback_handover_assumptions_need_empirical_verification.md` — every "r1 shipped X" claim below was grep-verified against the `phase-v1-RT-r4` tree, NOT assumed from the PRD:

| Dependency | State on `phase-v1-RT-r4` | Evidence |
|---|---|---|
| `sponsor_allowlist` TABLE | ✅ EXISTS | `crates/db_schema/src/source/governance/sponsor_allowlist.rs` (struct `SponsorAllowlist` + `SponsorAllowlistInsertForm`, current shape: `id, community_id Option<CommunityId>, person_id, created_at, added_by_admin_id, note Option<String>`); in `crates/db_schema_file/src/schema.rs` |
| `SponsorAllowlistId` newtype | ✅ EXISTS | imported in `sponsor_allowlist.rs:1` from `crate::newtypes` |
| `ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED` const | ✅ EXISTS (pre-landed RT-r1) | `crates/db_schema/src/source/governance/governance_log.rs:217` (`= "sponsor_allowlist_added"`); re-exported in api shim `crates/api/api/src/governance/governance_log.rs:60` |
| `ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED` const | ✅ EXISTS (pre-landed RT-r1) | `db_schema/.../governance_log.rs:218` (`= "sponsor_allowlist_removed"`); re-export shim `:61` |
| Registry rows naming RT-r4 as fire site | ✅ PRESENT | `.claude/rules/governance-log-entry-kind-registry.md:190-191` (both rows `(pending)` → RT-r4 `admin_sponsor_allowlist.rs::add`/`::remove`); pre-landed-const exemption at `:237` explicitly links both to v1-RT-r4 |
| `admin_sponsor_allowlist.rs` handler | ❌ DOES NOT EXIST | this is what RT-r4 BUILDS |
| `SponsorGateStrategy` enum + dispatch | ✅ EXISTS (v0, 3 strategies) | `crates/api/api_crud/src/governance/create_endorsement.rs:84` enum (`Age`, `Open`, `Closed`, `Unknown`); `:161` `match &strategy` dispatch; `:94` `parse()` |

**Load-bearing consequence**: RT-r4 needs **NO new migration** for the allowlist table or the entry-kind consts (both shipped in RT-r1). RT-r4 MAY need allowlist query helpers (`insert`, `delete`, `is_allowlisted`) in the db layer if they don't exist yet — planner verifies (see §2.1 deliverable b). The two `ENTRY_KIND_*` consts are **declared-but-not-yet-emitted**; RT-r4 is their first emission site. This flips them from registry `(pending)` to live — planner's §13 must include the registry-row update (drop `(pending)`, fill the real handler-fn name).

---

## 1. Role + dispatch line

`[role:planning] v1-RT-r4 plan — 3 sponsor-gate strategies + sponsor_allowlist admin add/remove endpoints`

The actual create-task description is a single line under 100 chars per `.claude/rules/advisor-orchestrator.md` Junior task description template:

```
[role:planning] v1-RT-r4 plan — see .claude/PRPs/briefs/rt-r4-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at `.claude/PRPs/plans/v1-RT-r4.plan.md` for sub-phase **v1-RT-r4**. The plan covers PRD §11 row 4 in full: three new sponsor-gate strategies + the `sponsor_allowlist` admin add/remove endpoints + the two `ENTRY_KIND` emissions + e2e coverage.

> **Plan filename note**: use `v1-RT-r4.plan.md` (matches the roadmap entry key + the `/auto-phase` glob `.claude/PRPs/plans/<sub-phase>*.plan.md`). The RT-r1 plan used the long form `v1-reputation-tuning-r1.plan.md`; RT-r2/r3 should be checked for the actual convention — but `/auto-roadmap` and `/auto-phase` both glob `v1-RT-r4*`, so the short form is required for the skill chain to find it. Confirm by `ls .claude/PRPs/plans/v1-RT-r*.plan.md` and mirror whichever the most recent RT plan used; if it used the long descriptive form, ALSO ensure the glob still resolves (`v1-RT-r4` prefix). **Default: short form `v1-RT-r4.plan.md`.**

### 2.1 Concrete deliverables (per PRD §5.4 + §7 + §10 + OQ-020)

The plan's §13 task list MUST cover all of these:

a. **Three new sponsor-gate strategies in `create_endorsement.rs`** (per PRD §5.4). The v0 enum at `crates/api/api_crud/src/governance/create_endorsement.rs:84` (`SponsorGateStrategy` — `Age`, `Open`, `Closed`, `Unknown(String)`) gains three variants, the `parse()` at `:94` learns their strings, the `label()` at `:104` learns their labels, and the `match &strategy` dispatch at `:161` gains three arms:
   - **`'age_or_surety'`** — passes if age gate OR caller has ≥1 active surety as the sponsored party. Per PRD §5.4: `OR EXISTS (surety WHERE sponsored_id = caller AND revoked_at IS NULL)`. Planner verifies the `surety` table + its `sponsored_id`/`revoked_at` columns exist (grep `crates/db_schema/src/source/governance/`); the strategy reads the existing age-gate helper at `create_endorsement.rs:328` (`fn enforce_age_gate` or similar — planner reads the actual fn name) AND a new surety-existence check.
   - **`'reputation'`** — passes if `reputation_snapshot.can_sponsor` is true for the caller (community-scoped if `community_id` provided, else instance-scoped). Per PRD §5.4: `can_sponsor` already populates per Phase 5a task 53, gated against `thresholds.endorsement_strength` (default `25`). RT-r4 flips it from computed-but-unread to strategy-readable. Planner verifies `can_sponsor` is a column on `reputation_snapshot` and finds the read path (likely `load_or_compute_snapshot` in `reputation_snapshot.rs`).
   - **`'allowlist'`** — passes only if caller is in `sponsor_allowlist` (community-scoped row `community_id = Some(c)` OR instance-wide row `community_id IS NULL`). Reads the allowlist via the db helper from deliverable (b).

   **Grandfathering** (PRD §5.4): strategy is read at `create_endorsement` time only; existing endorsements are never retroactively validated. No work needed beyond NOT adding retroactive validation — note this in the plan as a non-goal so impl doesn't invent it.

   **Composition strategies rejected** (OQ-020): single-string strategy only. Do NOT plan `'age_or_surety_or_allowlist'` or any combinator. Hard out-of-scope.

b. **Allowlist db-layer query helpers** (if absent — planner verifies first). The `'allowlist'` strategy (a) AND the admin endpoints (c) both need: insert a row, delete a row, check membership. If these don't exist on the `SponsorAllowlist` impl or a `db_views`/`db_schema` impl module, RT-r4 adds them (mirror an existing `governance` db-helper module's insert/delete/exists shape — e.g. the `reputation_event` or `surety` impls). If they DO exist (RT-r1 may have shipped them with the table), reuse — note which in the plan. Planner's grep decides; this deliverable is conditional.

c. **Admin endpoints: `POST /api/v4/governance/admin/sponsor-allowlist/add` + `.../remove`** (per PRD §5.4 + §10). New handler file `crates/api/api/src/governance/admin_sponsor_allowlist.rs`. **Canonical sibling: `crates/api/api/src/governance/admin_config.rs`** (1372 lines — read its `admin_set_config` at `:383` for the full shape):
   - Signature pattern: `pub async fn admin_sponsor_allowlist_add(data: Json<AdminSponsorAllowlistAdd>, context: Data<...>) -> LemmyResult<Json<AdminSponsorAllowlistAddResponse>>` (mirror `admin_set_config`'s signature + arg order).
   - Capability check: instance-admin only. Mirror `admin_config.rs`'s admin-gate (it reads the caller's admin status; planner reads the exact helper — likely `is_admin` / `LocalUserView` check at the handler top).
   - Pseudonym: `actor_pseudonym_helper::get_or_create(pool, admin_id).await?` (per `admin_config.rs:485`).
   - **Transaction + governance_log discipline** (per `feedback_multi_write_handlers_need_transactions.md`): the add path does 2 writes (insert allowlist row + governance_log entry). `admin_config.rs:497` shows the `run_transaction` pattern, BUT note `admin_config.rs:381-422` documents that `governance_log::append` opens its OWN internal tx — so the planner must decide whether the allowlist insert + log entry share one tx or the log is appended outside (mirror exactly what `admin_config.rs` does: the config write is inside `run_transaction`, the log append for the denial path is OUTSIDE; for the success path read the actual ordering at `:497-558`). Get this right — it's the §3.1.1 conformance-audit axis most likely to drift.
   - `ENTRY_KIND`: add path emits `governance_log::append` with `ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED`, payload per registry `:190`: `{allowlist_id, community_id?, person_pseudonym, added_by_admin_pseudonym, note?, added_at}`. Remove path emits `ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED`, payload per registry `:191`: `{allowlist_id, community_id?, person_pseudonym, removed_by_admin_pseudonym, removed_at}`. **person_pseudonym** = the allowlisted person's pseudonym (NOT raw person_id — ADR-015); the admin's pseudonym is `added_by_admin_pseudonym`/`removed_by_admin_pseudonym`. Both via `actor_pseudonym_helper`.
   - Remove path: delete by `(community_id, person_id)` or by `allowlist_id` — planner picks based on the DTO shape (PRD §5.4 implies person+community keyed; the add returns an id). If remove-by-id, the DTO carries `allowlist_id`; if remove-by-person, it carries `person_id` + `community_id?`. **Default: remove by `person_id` + optional `community_id`** (matches "remove this person from the allowlist" admin intent); the handler resolves the row to get `allowlist_id` for the log payload. Planner confirms against the add-response shape.

d. **DTOs in `crates/api/api_common/src/governance.rs`** (canonical sibling: `AdminSetConfig` at `:444` + `AdminSetConfigResponse` at `:465`). Four new structs:
   - `AdminSponsorAllowlistAdd { person_id: PersonId, community_id: Option<CommunityId>, note: Option<String> }`
   - `AdminSponsorAllowlistAddResponse { allowlist_id: SponsorAllowlistId, ... }` (mirror `AdminSetConfigResponse` fields — likely the created row or a success marker)
   - `AdminSponsorAllowlistRemove { person_id: PersonId, community_id: Option<CommunityId> }` (or `allowlist_id` per (c) decision)
   - `AdminSponsorAllowlistRemoveResponse { ... }`
   Derive macros + `ts-rs` gating: mirror `AdminSetConfig`'s derive stack verbatim (`#[skip_serializing_none]`, `Serialize`/`Deserialize`, the `ts-rs` cfg-gate).

e. **Route registration in `crates/api/routes/src/lib.rs`** (per the admin scope at `:493-507`). Two edits:
   - `use` import block (mirror `:36` `admin_config::{admin_get_config, admin_get_config_audit, admin_set_config}`): add `admin_sponsor_allowlist::{admin_sponsor_allowlist_add, admin_sponsor_allowlist_remove}`.
   - Admin scope service block (mirror the `scope("/config")` nested service at `:503`): add a `scope("/sponsor-allowlist")` with `.route("/add", post().to(admin_sponsor_allowlist_add))` + `.route("/remove", post().to(admin_sponsor_allowlist_remove))`.
   - Module declaration: ensure `admin_sponsor_allowlist` is declared in the governance mod tree (mirror how `admin_config` is declared — likely `crates/api/api/src/governance/mod.rs` `pub mod admin_sponsor_allowlist;`).

f. **Registry-row update** in `.claude/rules/governance-log-entry-kind-registry.md:190-191`: drop the `(pending)` markers; replace `admin_sponsor_allowlist.rs::add (handler name TBD; pending)` with the real handler fn names (`admin_sponsor_allowlist_add` / `admin_sponsor_allowlist_remove`). This is a `.claude/` meta-edit (advisor/planner-owned, not impl `crates/` work) but it's part of the RT-r4 deliverable so plan §13 names it as a task (mechanical — single-file edit).

g. **e2e coverage in `crates/server/tests/e2e.rs`** — at minimum: (i) each of the 3 new strategies fires correctly under matching + non-matching conditions; (ii) grandfathering (pre-switch endorsement stays valid after a strategy change); (iii) add endpoint inserts a row + emits the `ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED` log; (iv) remove endpoint deletes + emits `_REMOVED`; (v) non-admin caller rejected. Mirror the most recent RT/SL/JM fixtures sibling module's error-shape (Case A vs B per `feedback_lemmy_error_no_std_error.md` — planner reads the sibling at its line range and picks; **mandatory** per §3.3 file-class table). e2e.rs is **large** (RT-r1 brief cited 12,819 lines post-SL-c-2; verify current count) — anchor-Edit discipline mandatory; pre-locate verbatim `old_string`/`new_string` anchors per `feedback_fix_impl_pre_locate_e2e_anchors.md`.

### 2.2 Inside-handler step ordering

The add handler ordering (mirror `admin_config.rs` exactly):
1. Resolve + check admin capability (reject non-admin BEFORE any write).
2. Resolve admin pseudonym (`actor_pseudonym_helper::get_or_create`).
3. Resolve the allowlisted person's pseudonym.
4. Insert the `sponsor_allowlist` row + append the governance_log entry — get the tx/log ordering from `admin_config.rs:497-558` (the config write is inside `run_transaction`; `governance_log::append` opens its own internal tx). **Do NOT invent a different ordering** — the conformance audit (§3.1.1) checks this axis.
5. Return the response DTO with the created `allowlist_id`.

The remove handler mirrors steps 1-3, then resolves the target row (to capture `allowlist_id` + `community_id` for the log payload), deletes it, appends the `_REMOVED` log entry, returns.

### 2.3 Scope boundary — what is NOT in RT-r4

Per PRD §11 phase table:

- **RT-r2:** Per-dimension chained-halving decay calculator + `DECAY_KNOB_CHANGED` emit at `admin_config.rs`. RT-r4 does NOT touch `compute_applied_delta` / `recompute_snapshot` / the decay knobs.
- **RT-r3:** Multi-source participation events (crons + vote-outcome + evidence-quality emitters) + `flag-bad-faith` endpoint. RT-r4 does NOT add crons or those emitters.
- **RT-r5:** Instance-wide rollup cron + `GET /admin/reputation/rollup`. RT-r4 does NOT add the rollup.
- **RT-r6:** Carry-forward CodeRabbit fixes (#19-#22, #31). RT-r4 does NOT touch them.

**Hard out-of-scope for RT-r4** (per PRD §5.4 + OQ-020):
- **Composition gate strategies** (`'age_or_surety_or_allowlist'` etc.) — explicitly rejected per OQ-020, exponential test cost.
- **Retroactive endorsement re-validation on strategy switch** — grandfathering is the contract; existing endorsements stay valid.
- **Per-community allowlist weight / tuning** — the allowlist is a binary membership table, not a weighted one.
- **New migration for the allowlist table or entry-kind consts** — both shipped in RT-r1; RT-r4 is handler + strategy + emit only. (Deliverable (b) db-helpers are NOT a migration; they're Rust query fns.)

### 2.4 Plan file deliverable shape

Plan file at `.claude/PRPs/plans/v1-RT-r4.plan.md` follows `.claude/PRPs/templates/plan.template.md` verbatim per `feedback_read_canonical_before_writing_spec.md`. Required sections:

- §1 Goal + non-goals
- §2 / §16a Stories — write checkpoint commands so `/brehon-verify` can iterate (per `feedback_brehon_verify_pre_merge.md`); each story has `expects:` + `checkpoint:`.
- §3 Risks + mitigations.
- §4 Watchpoints — **must cite specific tables + files** per `feedback_advisor_watchpoint_specificity.md`. Name `sponsor_allowlist`, `create_endorsement.rs:161`, `admin_sponsor_allowlist.rs`, the `surety` table, `reputation_snapshot.can_sponsor`. Concept-only watchpoints ("watch for capability-check drift" without naming the helper) FAIL the gate.
- §5 Complexity score — per `feedback_complexity_score_pre_split.md`. RT-r4 is one new handler file (2 endpoints) + an enum extension + db helpers + e2e. Estimate likely 6-8. If `> 8`, the planner files a split-or-proceed DQ. **Dominant factor is likely e2e edits (≥2)** → if so, append `feedback_fix_impl_pre_locate_e2e_anchors.md` to §3 Required reading per advisor-orchestrator.md §2.5.
- §13 Task list — DoD-keyed; each task names IMPLEMENT files + per-task validation gate (`cargo check --workspace --features full`; e2e where touched). **Use `--workspace`, NOT `-p lemmy_server --features full`** (per `feedback_features_full_p_crate_incompatible.md`). Mark `[P]` per `feedback_parallel_cohort_dispatch.md` ONLY where YAML `creates:`/`modifies:` arrays prove non-overlap. Each task carries a `FILES:` YAML block with `creates:`+`modifies:`+`requires:` arrays per `feedback_explicit_file_arrays_on_tasks.md` + `feedback_cohort_validation_dependency_check.md`.
  - **Dependency ordering**: the `'allowlist'` strategy arm (a) `requires:` the db-helper task (b); the admin endpoints (c) `requires:` (b) + the DTOs (d); routes (e) `requires:` (c). The strategy arms (a) for `age_or_surety`/`reputation` are independent of the allowlist chain and MAY be `[P]` with (b). Planner threads `requires:` correctly so the cohort dependency-check (advisor-orchestrator.md §4.1 step 4a) passes.
  - **MiniMax-trial-friendly shaping** (per the A/B trial note in the header): make the add-handler, remove-handler, and their e2e tests as single-file + MIRROR-ref-shaped as the dependency chain allows, so the advisor can designate 5 clean trial tasks post-plan.
- §15 DoD validation commands — per `feedback_pre_phase_dod_smoke_test.md`, advisor runs every command literally before plan approval. Each must be executable on a clean checkout of `phase-v1-RT-r4`. (Note: under Shape-G-suspended, the e2e DoD runs locally via the bat wrapper per `feedback_windows_e2e_requires_bat_wrapper.md` — `--workspace --test e2e --features full`, NEVER bare `cargo test`, NEVER `-p lemmy_server --features full`.)

---

## 3. Required reading (in order, before drafting any plan section)

### 3.1 PRD anchors (load first)

1. `.claude/PRPs/prds/v1-reputation-tuning.prd.md` — full PRD. Most load-bearing for r4: **§5.4** (three new strategies + `sponsor_allowlist` admin endpoints + grandfathering — read verbatim), §7 (the `_SPONSOR_ALLOWLIST_ADDED`/`_REMOVED` `ENTRY_KIND` pair + their payloads), §10 (Security — admin-only, both add+remove emit governance_log), §11 row 4 (phase definition + dependency = v1.r1 allowlist table).

2. `.claude/rules/governance-log-entry-kind-registry.md:190-191` + `:237` (pre-landed-const exemption) — the exact payload shapes the two emits must produce + the registry-row-update deliverable.

### 3.2 ADR + open-question anchors

3. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` **OQ-020** (sponsor gate-strategy expansion — three strategies ship, composition strategies rejected). The OUT-of-scope contract.

4. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` **ADR-015** (pseudonymised actor IDs — both emits log `person_pseudonym` + admin pseudonym, never raw `person_id`).

5. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` **ADR-008** (governance_log append-only — the two `ENTRY_KIND` consts are TEXT-keyed, already shipped in RT-r1; verify zero migration).

6. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` **ADR-005** (multi-dimensional reputation — the `'reputation'` strategy reads `can_sponsor` which is gated against `thresholds.endorsement_strength`; do NOT collapse dimensions).

### 3.3 Mandatory file-class lessons (per advisor-orchestrator.md §2.4 file-class table)

RT-r4 IMPLEMENT file list (per §2.1 deliverables):
- `crates/api/api_crud/src/governance/create_endorsement.rs` (modify — enum + parse + label + 3 match arms)
- `crates/api/api/src/governance/admin_sponsor_allowlist.rs` (CREATE — new handler)
- `crates/api/api/src/governance/mod.rs` (modify — module declaration)
- `crates/api/api_common/src/governance.rs` (modify — 4 new DTOs)
- `crates/api/routes/src/lib.rs` (modify — use-import + route registration)
- (conditional) a db-helper module for allowlist insert/delete/exists
- `.claude/rules/governance-log-entry-kind-registry.md` (modify — drop `(pending)`)
- `crates/server/tests/e2e.rs` (modify — strategy + endpoint tests)

Walk against the §2.4 file-class table → these rows fire (mechanical, no judgment):

| Pattern matched | Mandatory lesson(s) for §3 Required reading |
|---|---|
| Any handler under `crates/api/**/src/**` doing 2+ DB writes (the add/remove handlers: insert/delete row + governance_log entry) | `feedback_multi_write_handlers_need_transactions.md` |
| `crates/server/tests/e2e.rs` (any edit) | `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`. **When an RT/SL/JM fixtures sibling module already exists in the same file, mirror its error-shape case (A or B) verbatim — pick by reading the sibling at its cited line range BEFORE authoring.** |
| `crates/server/tests/e2e.rs` (≥2 edits in this task or cohort — RT-r4 adds multiple test fns) | `feedback_fix_impl_pre_locate_e2e_anchors.md` (pre-locate verbatim anchors — the canonical e2e-edit-hang-prevention lesson) |
| Any `#[cfg(feature = "full")]` gate (the `SponsorAllowlist` struct + new DTOs are `full`-gated) | `feedback_features_full_workspace_only.md`, `feedback_features_full_p_crate_incompatible.md` |

The §2.3 hybrid PMD search still runs after this table check — planner runs `memory_search_hybrid(query: "admin handler governance_log emit capability check", tags: "lesson")` once and cites hits (catches non-mechanical / cross-cutting lessons, e.g. the admin-capability-check pattern or the `run_transaction` + `append`-own-tx interaction).

### 3.4 Plan-template + canonical-sibling reads

7. `.claude/PRPs/templates/plan.template.md` — section structure verbatim.

8. **Canonical sibling plan** — the most recent shipped RT sub-phase plan that added a handler. RT-r3 added `flag-bad-faith` (an admin handler with an `ENTRY_KIND` emit) — its plan is the closest structural sibling for "admin handler + capability check + governance_log emit + e2e". `ls -t .claude/PRPs/plans/v1-RT-r3*.plan.md` and read its §13 + §15 + §16a. If RT-r3's plan is unavailable, fall back to a v1-AD or v1-JM admin-handler plan.

9. **Canonical sibling handler** — `crates/api/api/src/governance/admin_config.rs` (the `admin_set_config` shape at `:383-558`: signature, admin-gate, pseudonym resolve, `run_transaction`, `governance_log::append`). This is the verbatim shape `admin_sponsor_allowlist.rs` mirrors.

10. **Canonical sibling for the gate-strategy extension** — `crates/api/api_crud/src/governance/create_endorsement.rs` (the v0 `SponsorGateStrategy` enum + `parse`/`label`/`match` at `:84-171`, the age-gate helper at `:328`). The three new strategies extend this exact structure.

### 3.5 PMD-promoted patterns (load on demand)

- `pattern_cargo_feature_flag_propagation.md` — `--features full` discipline; `--workspace` over `-p`.
- `pattern_spec_schema_co_commit.md` — DTO + handler + route + e2e for one endpoint land in coherent task grouping.
- `pattern_bm_false_success_advisor_post_condition_catch.md` — relevant downstream (BM verbs), not at plan time, but the planner should make §16a checkpoints concrete enough that `/brehon-verify` catches a phantom handler.
- `pattern_context_is_finite.md` — plan body likely 600-900 lines; cite PRD by anchor, copy only contracts (the payload shapes, the strategy semantics).

---

## 4. Constraints (hard rules — violating any is a process breach)

### 4.1 Plan-content discipline

a. **PRD-aligned narrow scope only.** RT-r4 = 3 strategies + 2 admin endpoints + 2 emits + e2e. Any temptation to bundle r5's rollup or r3's crons "while in the governance dir" is OUT — split rejection per `feedback_complexity_score_pre_split.md` + PRD §11 dependency chain.

b. **No new migration.** The allowlist table + both entry-kind consts shipped in RT-r1 (§0 verified). Plan §13 must NOT include a `migrations/**` task. The only db-layer additions are Rust query helpers (deliverable b), which are NOT migrations. If the planner believes a migration IS needed (e.g. a missing index or constraint on `sponsor_allowlist`), file a `kind: "blocker"` planner DQ with the specific gap — do NOT silently add a migration task.

c. **PRD §5.4 + OQ-020 are the contract.** Three strategies, single-string, composition rejected, grandfathering preserved. Drift (e.g. planning a 4th strategy or a combinator) is a process miss; if the planner finds the PRD ambiguous on a strategy's semantics, file a planner DQ citing the §5.4 line rather than guessing.

d. **Pseudonym discipline (ADR-015).** Both emits log `person_pseudonym` (the allowlisted person) + admin pseudonym, never raw `person_id`. Plan §13 must name `actor_pseudonym_helper::get_or_create` in the handler task and the conformance-audit (§3.1.1) checks this.

e. **Transaction discipline (per `feedback_multi_write_handlers_need_transactions.md`).** Both handlers do 2 writes. Plan §13 must reflect the `admin_config.rs:497-558` ordering verbatim (config write inside `run_transaction`; `governance_log::append` opens its own internal tx). The planner reads the actual ordering and documents it — does NOT invent a SAVEPOINT or a single-tx-wrapping-append shape.

### 4.2 Decision-queue discipline (per `.claude/rules/decision-queue.md`)

a. **Planner writes `kind: "blocker"` from `from: "planner"`** with `answered_by: null` for genuine blockers (e.g. "does the `surety` table have `sponsored_id`+`revoked_at`?" if the grep is ambiguous; "does `can_sponsor` exist as a column?").

b. **Planner may pre-seed `answered_by: "planner"` with citation** for design choices the PRD/§0 already addresses (e.g. remove-by-person-id default — cite this brief §2.1c).

c. **Planner does NOT write `answered_by: "advisor"` or `answered_by: "user"`** (attribution-integrity per decision-queue.md §Attribution integrity).

d. **Mid-task push discipline** — every DQ write commits + pushes immediately on the worker branch (`junior/rt-r4-planning-1`) per decision-queue.md "Mid-task visibility". Use the v3 composite-id helper (`bash scripts/brehon/dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh`) per `feedback_dq_v3_append_via_helper_script.md` — NEVER the abolished `max(all_ids)+1` recipe.

### 4.3 File ownership

Planner writes ONLY:
- `.claude/PRPs/plans/v1-RT-r4.plan.md` (the single deliverable)
- `.claude/decision-queue.json` (DQ entries — pending or planner-self-resolved, via the v3 helper)

Planner does NOT write:
- Any file under `crates/**` (impl-task work — planner authors the plan, impl-task authors the code)
- Any other plan file or brief (one brief = one plan)
- `docs/brehon-law-inspired-network/**` (design-doc territory)
- `.claude/lessons/**` (lessons authored by user / advisor / retro)
- `.claude/rules/governance-log-entry-kind-registry.md` — the registry-row update is a §13 *task* (impl-task or advisor executes it), NOT something the planner edits while authoring the plan.

### 4.4 Attribution integrity reminder

Every commit subject on the planning worker branch matches the planner-attribution pattern: `chore(plan): rt-r4 — <slug>` or `feat(plan): rt-r4 — <slug>`. Subjects matching `^(chore|docs)\((advisor|decision-queue)\)` are advisor-only — planner must not author them. Per decision-queue.md §Detection.

### 4.5 Forbidden execution windows

Planner is dispatched as a Junior task (runs on the daemon — no cargo, just plan authoring). Forbidden windows are non-binding for the planning dispatch itself; they bind only the advisor's local validate-pending-laptop cargo runs later. Per advisor-orchestrator.md §5.1.

---

## 5. Pre-commit dogfood (per advisor-orchestrator.md §3.7)

This brief was walked through against (advisor session, 2026-05-29):
- `.claude/PRPs/prds/v1-reputation-tuning.prd.md` §5.4 (parsed — three strategies + admin endpoints + grandfathering), §7 (parsed — the two `ENTRY_KIND` + payloads), §10 (parsed — admin-only + both emit), §11 row 4 (phase definition + v1.r1 dependency).
- `crates/api/api_crud/src/governance/create_endorsement.rs` (read — `SponsorGateStrategy` enum + parse/label/match extension points at `:84`/`:94`/`:104`/`:161`).
- `crates/api/api/src/governance/admin_config.rs` (read — `admin_set_config` shape at `:383-558` as the canonical handler sibling).
- `crates/api/api_common/src/governance.rs` (read — `AdminSetConfig`/`AdminSetConfigResponse` at `:444`/`:465` as the canonical DTO sibling).
- `crates/api/routes/src/lib.rs` (read — admin scope service block at `:493-507` as the canonical route-registration sibling).
- `crates/db_schema/src/source/governance/sponsor_allowlist.rs` (read — table struct + insert form, confirms RT-r1 shipped it).
- `.claude/rules/governance-log-entry-kind-registry.md:190-191`+`:237` (read — payload shapes + pre-landed-const exemption naming RT-r4).
- `.claude/PRPs/briefs/rt-r1-planning-1.md` (canonical sibling brief — same §0-§6 structure mirrored).
- `.claude/PRPs/briefs/minimax-m27-trial-1.md` §3 (read — trial designation happens post-plan, not at brief author; planner shapes §13 to be MIRROR-ref-friendly).

What worked:
- §0 dependency-reality check resolved the single biggest risk up front: the allowlist table + both consts are ALREADY shipped (RT-r1), so RT-r4 needs no migration. This prevents the planner from re-planning the table or treating r4 as a schema phase.
- Every canonical sibling (handler, DTO, route, enum) was located + line-cited, so the plan is a MIRROR-ref exercise, not a green-field design.
- The `[P]`/`requires:` dependency shape is pre-analysed (strategy arms for age_or_surety/reputation independent; allowlist chain serial) so the cohort dependency-check passes.

What I worked through that the planner must resolve (expected planner DQ or grep-verify):
- **`surety` table shape** — the `'age_or_surety'` strategy needs `surety.sponsored_id` + `surety.revoked_at`. Planner greps `crates/db_schema/src/source/governance/` to confirm before writing the strategy task; files a blocker DQ if absent or differently-named.
- **`can_sponsor` column** — the `'reputation'` strategy needs `reputation_snapshot.can_sponsor` to be a real readable column + a read path. Planner verifies (grep `can_sponsor`); files a blocker DQ if it's computed-only with no read accessor.
- **Allowlist db-helpers** — deliverable (b) is conditional on whether RT-r1 shipped insert/delete/exists fns. Planner greps the `SponsorAllowlist` impl + db_views; if present, reuse + note; if absent, RT-r4 adds them (deliverable b becomes a real task).
- **Remove-by-id vs remove-by-person** — brief defaults to remove-by-person+community; planner confirms against the add-response DTO shape and adjusts the remove DTO accordingly.
- **Plan filename convention** — short `v1-RT-r4.plan.md` (glob-compatible) vs RT-r1's long descriptive form. Planner checks the most recent RT plan + ensures the `v1-RT-r4` glob prefix resolves either way.

---

## 6. Acceptance for this brief

Brief is queueable when:
- DQ pending count = 0 OR all pending entries are non-blocking for RT-r4 planning.
- Forbidden-window check at dispatch time per advisor-orchestrator.md §5.1 (non-binding for planning dispatch; advisor confirms anyway).
- ✅ **Clarify gate** per advisor-orchestrator.md §3.3 — runs AFTER this brief is committed + before the planning Junior is dispatched (`/brehon-clarify .claude/PRPs/briefs/rt-r4-planning-1.md`). Any clarify-DQ must resolve before dispatch.

Brief is committed to `phase-v1-RT-r4` (Mode A — directly on the phase branch the planning Junior forks from) with subject:
```
chore(advisor): auto-roadmap auto-author planning brief for v1-RT-r4
```
