# federation-inbound-a planning brief

**Written**: 2026-05-16 by advisor session (laptop, brehon-fork CWD `C:/Users/barri/Developer/brehon-fork`, canonical checkout on `governance-v0` @ `821cbe718`).
**Subagent target**: `planning` (Opus 4.7 — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/federation-inbound-a-planning-1` from `governance-v0` committed HEAD `821cbe718`. Plan file commits + pushes back to `governance-v0` at finalize.
**Authority anchor**: `.claude/PRPs/prds/v1-federation-inbound.prd.md` — §8.0 (hard-gate preconditions), §8.2 (the v1 migration SQL), §8.4 (Diesel models), §4 (peer-trust state model), §10 (defaults matrix). This is the **first sub-phase of the federation-inbound v1 track**; ADR-006 (inbound governance signals are advisory-only) + ADR-015 (pseudonymisation) + ADR-014 (vanilla-Lemmy interop) are the load-bearing constraints. [05 §7.1](../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) authorises "Full inbound + outbound" under the v1 production-grade-governance milestone.

---

## 0. Track carving — why this sub-phase exists (read this first)

`v1-federation-inbound.prd.md` is a **single coherent design with no PRD-prescribed sub-phase carving** (unlike `v1-sponsor-liability.prd.md` whose §15 row table pre-carves SL-a..SL-e). The advisor has carved the federation-inbound v1 track along the PRD's natural seams. **This carving is a binding input to the plan — do NOT re-carve it:**

| Sub-phase | Scope (advisor-carved 2026-05-16) | PRD sections |
|---|---|---|
| **`v1-federation-inbound-a`** (THIS sub-phase) | **Schema + Diesel models + trust-state read/update foundation + config seeds + entry-kind consts.** Purely additive — zero HTTP-path change, zero behaviour change. Mirrors the SL-a "schema-first foundation" pattern exactly. | §8.2, §8.4, §4.1, §4.3, §10, the new `governance_log` entry-kind consts |
| `-b` (future, NOT this plan) | Inbound wrapper (`wrap_governance_inbound` §9.2) + per-handler patches (§9.3) + new `receive_remote_moderation_label` handler (§9.4) + rate-limit/replay enforcement (§7) + Phase 6 round-trip-test fixture update (§11.4) + handler-path e2e | §9, §7, §11.4 |
| `-c` (future, NOT this plan) | Admin review surface — 3 REST endpoints (§6 list/cross-link/dismiss) + unified inbox view projection + OQ-FED-IN-1 pseudonym rendering + admin-path e2e | §6, OQ-FED-IN-1, OQ-FED-IN-5 |

**Rationale for this carving (the planner does NOT re-decide this):** `-a` is the lowest-risk first sub-phase — it ships only schema + model + pure-DB-read helpers + seeds, touching **no HTTP path and no `receive_remote_*` function**. This is exactly the SL-a pattern (schema + seeds + backfill foundation shipped in isolation; downstream sub-phases consume it). It unblocks `-b` and `-c` to be planned/dispatched later. The PRD's §9 wrapper + §6 admin surface are deliberately deferred so `-a` stays a clean additive migration+model sub-phase with a migration-round-trip e2e and nothing else.

---

## 0.1 Pre-resolved facts (READ BEFORE §1 — these are BINDING; do NOT re-derive or file blockers for them)

The advisor verified these against `governance-v0` @ `821cbe718` before authoring this brief. They pre-empt round-trips the planner would otherwise have to make:

- **PRECON-1 — Phase 6 hard gate is SATISFIED.** PRD §8.0 makes Phase 6 merge a hard gate. The advisor confirmed on trunk: `federation_attestation` and `remote_sanction_notice` are both in `crates/db_schema_file/src/schema.rs` on `governance-v0`; migration `migrations/2026-04-21-000000-0000_add_federation_attestations/` exists. **The lane is unblocked.** The planner does NOT re-run the `git log` Phase-6 check (it's done); the planner DOES still author the §8.2 SQL `DO $$ … $$` fail-loud preamble verbatim (it is defence-in-depth for out-of-order `diesel migration run`, per PRD §8.2 preamble rationale).
- **PRECON-2 — `source_instance` is TEXT (the FK-vs-TEXT ambiguity is RESOLVED in Phase-6's favour).** PRD §4.1 declares `federation_peer.instance_id INT FK`, but PRD §8.0 precondition #6 + §15.3 flagged a possible mismatch with Phase 6's `remote_sanction_notice.source_instance`. The advisor confirmed Phase 6's migration on trunk ships `source_instance TEXT NOT NULL` and `received_at TIMESTAMPTZ NOT NULL DEFAULT now()`. **Binding consequence:** (a) `federation_peer.instance_id INT PRIMARY KEY REFERENCES instance(id)` is correct as PRD §8.2 specifies (the FK is on `federation_peer`, NOT on the Phase 6 tables). (b) The PRD §8.0 precondition #3 (`received_at` present) and #6 (`source_instance` TEXT) are **already true on trunk** — the planner authors the `DO $$` checks verbatim but knows they will pass; this is NOT a blocker. (c) Any JOIN from `federation_peer` to a Phase 6 advisory row is `JOIN instance ON instance.domain = remote_sanction_notice.source_instance` then `JOIN federation_peer ON federation_peer.instance_id = instance.id` — the TEXT-domain → instance.id → federation_peer.instance_id path. The planner notes this join path in the plan's §4/§10 (it is the canonical trust-lookup shape `-b` will consume) but `-a` ships only the **table + the read/update helpers**, not the wrapper that calls them.
- **PRECON-3 — Shape G is SUSPENDED until 2026-06-01.** Per `governance-v0` commits `086cfa5d4` + `821cbe718` and PMD `project_shape_g_suspended_2026_05_16.md`: GH-Actions cargo validation is suspended (Actions minutes exhausted). **`validate-pending-laptop` mode is active** (DQ #229 tracks re-enable on 2026-06-01). **Binding consequence for the plan's §15:** the plan ships under the **pre-Shape-G `validate-pending-laptop` DoD shape**, NOT the Shape-G per-workflow shape. §15 must name the §15 DoD cargo commands **verbatim with scope flags + `--features full`** (the impl-task raises `kind: "validate-pending-laptop"` post-push; the advisor laptop session runs them). Use `pattern_cargo_feature_flag_propagation` discipline. Do NOT author the Shape-G `.github/workflows/*.yml` per-workflow DoD section — that is dormant until 2026-06-01. (If the planner sees prior plans like `v1-sponsor-liability-e` using the Shape-G shape, that is a pre-suspension artifact — this plan uses the laptop shape per PRECON-3.)
- **PRECON-4 — structural mirror is `v1-sponsor-liability-a.plan.md`.** The federation-inbound-a work (CREATE migration with enums+tables+ALTERs+config-seeds; regenerate `schema.rs`; new Diesel model files + UpdateForms; new config consts; new `governance_log` ENTRY_KIND consts; extend the `phase1_migrations_round_trip` e2e) is structurally near-identical to SL-a. The planner MUST read `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` in full and mirror its §13 task decomposition, its §5 complexity-factor table shape, its §15 per-task DoD shape (adjusted to the `validate-pending-laptop` form per PRECON-3), and its §16a Stories shape. This is the canonical sibling per `.claude/rules/advisor-orchestrator.md` §3.6 (canonical-schema-first gate) and `feedback_read_canonical_before_writing_spec.md`.

---

## 0.2 Clarify-pass resolutions (READ BEFORE §1 — these are BINDING; resolve 3 ambiguities the planner would otherwise round-trip)

The advisor ran `/brehon-clarify` on this brief (2026-05-16, advisor-mode). Three clarify-DQ entries are RESOLVED with evidence and are **binding inputs to the plan**. They supersede the corresponding "the planner discovers / picks" language in §2.1/§2.4 for these three points:

- **DQ #230 (RESOLVED, advisor) — trust-state helper module home is DECIDED.** The §2.1 item-4 trust-state helpers (`federation_inbox_check_peer_trust` + the `federation_peer` write helper) go in a **`lemmy_db_schema` query module** (e.g. colocated in `crates/db_schema/src/source/governance/federation_peer.rs`, OR a sibling query/impls module per the crate's existing query-fn convention). **Rationale:** Phase 6's `crates/apub/activities/src/governance/inbox.rs:1-30` doc-comment + resolved DQ id 37 (DQ-6.6-inbound) prove the PRD §9.1 `crates/apub/apub/src/governance/` location is UNREACHABLE — Phase 6's receivers were forced into `lemmy_apub_activities` ONLY because `Activity::receive` is invoked at that dep-graph layer. `-a`'s helpers have NO such constraint (called by `-b`'s future wrapper, not by `Activity::receive`), so they belong with the data layer. The PRD §9.1 "extending `crates/apub/apub/src/governance/`" language describes **`-b`'s wrapper** (AP-framework-adjacent, inherits the corrected `lemmy_apub_activities` placement). The plan's §10 MUST cite `inbox.rs:1-30` + resolved DQ id 37 as the precedent; §6 MUST note the corrected `-b` wrapper placement so `-b`'s planner inherits it. `-a` ships the helper signatures + the `get_conn(pool)->AsyncPgConnection` access pattern (mirror `inbox.rs:157`); does NOT wire them into any HTTP/receive path. **The planner does NOT re-discover the module home — it is decided; just cite the precedent and place per this resolution.**
- **DQ #231 (RESOLVED, advisor) — §15 DoD shape: TRANSLATE SL-a's Shape-G §15 into `validate-pending-laptop`, do NOT copy it.** PRECON-4 names SL-a as the structural mirror, but SL-a's §15 is itself **Shape-G** (`v1-sponsor-liability-a.plan.md` §15 references `cargo-validate-workspace.yml` + `cargo-validate-migration.yml` + `kind:"validate-pending"`, lines 21/50/235/237/1246-1248). Per PRECON-3 Shape G is SUSPENDED until 2026-06-01. **Binding §15 authoring rule:** (a) name the DoD commands VERBATIM with scope flags — `bash scripts/brehon/cargo-check.sh --workspace --features full`, the migration round-trip (mirror SL-a's `cargo run -p lemmy_diesel_utils --features full -- migration run/revert` + `scripts/brehon/migrate-roundtrip.sh`, SL-a lines 61-63/1164), the e2e test-name filter; (b) §15 states "impl-task raises `kind:"validate-pending-laptop"` post-push naming these verbatim; advisor laptop runs them sequentially per `.claude/rules/advisor-orchestrator.md` §5.2 and mutates the entry"; (c) **NO `.github/workflows/*.yml` §15.6 section** (Shape-G section dormant until 2026-06-01 / DQ #229); (d) add a §15 note recording the suspension window + DQ #229 + "PLAN ships laptop-shape; advisor MAY switch open entries to Shape-G post-2026-06-01 if impl still in flight". **SL-a is the mirror for §13/§5/§16a structure ONLY; its §15 is explicitly NOT mirrored — translated per DQ #231.** The planner reads `.claude/rules/advisor-orchestrator.md` §5.2 + `.claude/rules/decision-queue.md` `kind:"validate-pending-laptop"` for the exact entry shape.
- **DQ #232 (RESOLVED, advisor) — shared-file edits MUST be strictly additive / collision-isolated by construction.** Three CC advisor sessions run concurrently (this `-a` lane, **v1-ship**, **v1-AD-e**) and v1-RT-r1 very recently mutated the exact shared files `-a` will edit (`schema.rs` `03f6c670e`; `config.rs`+`EXPECTED_SEED_COUNT_V1_RT`+`SEEDED_KEYS` `198ab6d06`; `governance_log.rs`+registry `f2f9202b7`; pending RT-r1 fix-impl `03f519f37`). Per `multi-lane-worktree.md` the DQ is per-worktree but shared **code** files are NOT isolated — they collide at phase-branch reconcile + final merge. **Binding §13/§4 design rule:** every `-a` shared-file edit is append-only/additive — (1) `config.rs`: NEW `federation.inbound` const block + NEW `EXPECTED_SEED_COUNT_V1_FED_IN` + 11 keys APPENDED to `SEEDED_KEYS` (never reorder/reformat existing arrays or sibling-lane blocks) + a NEW per-lane parity test; mirror exactly how SL-a (`7cfb18b8f`) + RT-r1 (`198ab6d06`) coexist; (2) `governance_log.rs`: ~9 new `federation_*` consts as a NEW group + appended match arms (db_schema-define + api-shim-re-export split per SL-a `8b11841b9` / RT-r1 `f2f9202b7`); (3) `governance-log-entry-kind-registry.md`: APPEND a NEW `## v1-federation-inbound-a` section at file end (never edit sibling sections) + non-collision parity test vs ALL existing strings; (4) `schema.rs`: ADD-only (4 new table blocks + ALTER-added columns to existing blocks; no reorder/reformat of the diesel-managed block); (5) `e2e.rs`: ONE anchor-Edit at file end. The plan's §4 MUST carry an explicit watchpoint: "Concurrent lanes (v1-ship, v1-AD-e, v1-RT-r1) touch the same shared files; ALL `-a` shared-file edits are append-only/additive — a reformat or array-reorder of an existing sibling-lane block is a process breach that guarantees a reconcile-time merge conflict."

These three resolutions supersede the "planner discovers / picks / mirrors" language for those specific points. All other §2.1/§2.4 latitude (e.g. `FederationPeerId` newtype create-vs-reuse, the precise §13 task split) remains the planner's per the brief.

---

## 1. Role + dispatch line

`[role:planning] v1-federation-inbound-a plan — schema + Diesel models + trust-state foundation`

The actual create-task description (single line, <100 chars, per `.claude/rules/advisor-orchestrator.md` Junior task description template):

```
[role:planning] v1-federation-inbound-a plan — see .claude/PRPs/briefs/federation-inbound-a-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at **`.claude/PRPs/plans/v1-federation-inbound-a.plan.md`** for sub-phase **v1-federation-inbound-a**, following `.claude/PRPs/templates/plan.template.md`'s 20-section schema literally.

### 2.1 What `-a` ships (the additive schema + model + foundation layer)

The plan's §13 task list MUST cover exactly the following, and NOTHING beyond it (cross-check every task against the §0 carving table — anything in `-b`/`-c` scope is OUT):

1. **Migration `add_federation_inbound_v1`** (PRD §8.2, verbatim SQL):
   - The fail-loud `DO $$ … $$` Phase-6 precondition preamble (verbatim from PRD §8.2 — preconditions #2/#3/#4/#6).
   - `CREATE TYPE federation_peer_trust_enum` (4 variants) + `CREATE TYPE federation_inbox_admin_action_enum` (3 variants).
   - `CREATE TABLE federation_peer` (+ `idx_federation_peer_trust`).
   - `CREATE TABLE federation_inbox_dropped_log` (+ 2 indexes).
   - `CREATE TABLE federation_inbox_nonce` (+ `idx_fin_nonce_seen_at`).
   - `CREATE TABLE remote_moderation_label` (+ 3 indexes).
   - `ALTER TABLE remote_sanction_notice ADD COLUMN …` (4 columns + partial index).
   - `ALTER TABLE federation_attestation ADD COLUMN …` (6 columns + 2 indexes).
   - **Seed the 11 `federation.inbound.*` `governance_config` rows** (PRD §10 defaults matrix) in the SAME migration, `scope = 'instance'`, mirroring Phase 5a task 50's seed pattern (the planner reads SL-a's plan Task 1 / Task 6 for the exact `INSERT INTO governance_config` shape and the parity-test discipline).
   - **`down.sql`**: drop in reverse order (PRD §8.2 confirms column-drop is safe — Phase 6 tables are empty pre-v1).
   - **Timestamp slot**: directory-name prefix MUST sort strictly after `2026-04-21-000000-0000` (Phase 6's `add_federation_attestations`). The planner picks a concrete slot in the plan (e.g. `2026-05-NN-000000-0000_add_federation_inbound_v1`) and the plan's §4 watchpoint cites the directory-sort invariant explicitly (per PRD §8.2: `diesel migration run` orders by directory-name sort).
2. **Regenerate `crates/db_schema_file/src/schema.rs`** for the 4 new tables + the 2 ALTER-added column sets. Mirror SL-a plan Task 3's `schema.rs` discipline (the diesel-managed block; `print-schema` / forbid hand-edit divergence per `feedback_lemmy_migration_runner.md`).
3. **New Diesel model files** (PRD §8.4), mirroring `crates/db_schema/src/source/governance/sanction.rs` shape:
   - `crates/db_schema/src/source/governance/federation_peer.rs` — `FederationPeer`, `FederationPeerInsertForm`, `FederationPeerUpdateForm`.
   - `crates/db_schema/src/source/governance/federation_inbox_dropped_log.rs` — insert-only.
   - `crates/db_schema/src/source/governance/federation_inbox_nonce.rs` — insert-only + a period-cleanup query helper (the nonce-window cleanup the §7.5 cron will call later; `-a` ships the query fn, `-b`/cron wires the schedule).
   - `crates/db_schema/src/source/governance/remote_moderation_label.rs` — `RemoteModerationLabel`, `RemoteModerationLabelInsertForm`, `RemoteModerationLabelUpdateForm`.
   - **Extend Phase 6's insert-only models** in `remote_sanction_notice.rs` + `federation_attestation.rs` with the new columns + an `UpdateForm` each (Phase 6 ships insert-only forms; `-a` adds the UpdateForm + reads new columns). Per `feedback_insertform_default_propagation.md`: enumerate every existing `*InsertForm` construction site and confirm new nullable/defaulted columns propagate correctly.
   - **Newtypes** in `crates/db_schema/src/newtypes.rs`: `FederationInboxDroppedLogId(pub i64)`, `RemoteModerationLabelId(pub i32)`. `FederationPeerId` — PRD §8.4 says `instance_id` reuse is acceptable; create the newtype only if Diesel benefits (planner judgment; if uncertain → `kind: "log"` noting the decision, NOT a blocker). Per `feedback_newtype_locations_lemmy_db_schema_vs_file.md`.
4. **Trust-state read/update helpers** (PRD §4 — the pure-DB-read foundation `-b`'s wrapper consumes):
   - `federation_inbox_check_peer_trust(peer_domain, conn) -> FederationPeerTrust` (the TEXT-domain → `instance.id` → `federation_peer.instance_id` JOIN per PRECON-2; returns `Unknown` for first-seen — no `federation_peer` row). Module location: mirror Phase 6's governance module layout (PRD §9.1 says `-b`'s wrapper extends `crates/apub/apub/src/governance/`; `-a`'s pure helper belongs with the db-read layer — the planner reads Phase 6's `receive_remote_sanction_notice` module to pick the canonical home and states it explicitly in §10).
   - Trust-state write helper(s) for the `federation_peer` row (insert/update trust_level). NO HTTP endpoint in `-a` (the `POST …/trust` admin endpoint is `-c`); `-a` ships only the DB-layer fn the endpoint will later call.
   - **These helpers do NOT call any `receive_remote_*` function and are not wired into any HTTP path in `-a`.** That wiring is `-b`. `-a`'s e2e exercises them via direct DB-fixture calls only.
5. **New `governance_log` ENTRY_KIND consts** (PRD §3.2/§3.3/§12.4 reference these; per the v1-planning-queue resolution at line 318 federation-inbound opens 8+ new entry-kind strings): the const strings the inbound path will emit — at minimum `federation_inbound_blocked`, `federation_inbound_dropped_oversize`, `federation_inbound_dropped_schema`, `federation_inbound_dropped_rate_limit_peer`, `federation_inbound_dropped_rate_limit_actor`, `federation_inbound_dropped_replay`, `federation_inbound_dropped_storage_cap_evicted`, `federation_peer_trust_changed`, `federation_label_received`. Define them in `crates/db_schema/src/source/governance/governance_log.rs` + `crates/api/api/src/governance/governance_log.rs` + register them in `.claude/rules/governance-log-entry-kind-registry.md` — mirror SL-a plan Task 7's exact discipline (consts + match arms + registry section + the non-collision parity test against v0 + sibling-PRD consts). **Non-collision is critical** (governance_log dispatches by entry_kind string). `-a` ships the consts; `-b` emits them.
6. **Config consts + parity** (PRD §10): the 11 `federation.inbound.*` keys as consts in `crates/api/api/src/governance/config.rs` + match arms + `SEEDED_KEYS` + an `EXPECTED_SEED_COUNT_V1_FED_IN` + `ConfigKeyMetadata` entries (range validators per the PRD §10 Range column) + the parity test asserting seed-count == const-count. Mirror SL-a plan Task 6 verbatim in discipline.
7. **e2e — extend `phase1_migrations_round_trip`** in `crates/server/tests/e2e.rs` (mirror SL-a plan Task 8): assert the new migration applies+reverts cleanly and the new tables/columns/enums/config-seeds exist post-migrate. PLUS a focused trust-state foundation test: insert a `federation_peer` row, call `federation_inbox_check_peer_trust` for that domain (expect the seeded trust level), call it for an unknown domain (expect `Unknown`), exercise the trust-state write helper. **One anchor-Edit at file end** per `feedback_junior_worker_e2e_edit_hang.md` (e2e.rs is ~14,862 lines at HEAD `821cbe718` post-refactor-tier — the planner re-confirms the exact line count + the canonical e2e error-shape from a recent sibling test per `feedback_lemmy_error_no_std_error.md` case A/B/C, since the PR-1 #132 LemmyResult-unification changed the error-bridge idiom).
8. **Retro task** — the standard final §13 retro task, same as prior phases.

### 2.2 Scope boundary — what is explicitly NOT in `-a`

Per the §0 carving table. The plan's §12 ("NOT building") MUST enumerate these:

- **No `wrap_governance_inbound` wrapper** (PRD §9.2) — that is `-b`.
- **No per-handler patches** to `PublishSanctionNotice::receive` / `PublishTrustAttestation::receive` (PRD §9.3) — `-b`. `-a` does NOT touch `crates/apub/activities/src/governance/`.
- **No `receive_remote_moderation_label` handler** (PRD §9.4) — `-b` fills the Phase 6 `PublishLabel::receive` stub. `-a` ships the `remote_moderation_label` TABLE + model only.
- **No rate-limit / replay enforcement logic** (PRD §7) — `-b`. `-a` ships the `federation_inbox_nonce` table + the cleanup query fn only; the enforcement call sites are `-b`.
- **No admin REST endpoints** (PRD §6 list/cross-link/dismiss) — `-c`. `-a` ships zero new routes, zero new DTOs, zero new handlers under `crates/api/api/` or `crates/api/api_crud/` (the trust-state helpers in §2.1 item 4 are db-layer fns, NOT HTTP handlers).
- **No Phase 6 round-trip-test fixture update** (PRD §11.4 — inserting a `federation_peer` allowlisted row before `receive`) — that belongs in `-b` (it is the wrapper sub-phase that makes the existing direct-call test go through trust-check). `-a` does NOT modify `sanction_notice_round_trip`.
- **No OQ-FED-IN-1 pseudonym-rendering** — `-c` (admin surface).
- **No `webauthn-rs` step-up on trust-change** (PRD §12.5) — `-c` (it gates the `-c` admin endpoint, not the `-a` db helper).
- **No outbound change** (PRD §11.1 — outbound stays `to_all_instances()`).
- **No fork-local lint guards** (PRD §12.8) — those are a `.claude/rules/` meta-edit, advisor-side, not impl scope; out of `-a`.

**Hard out-of-scope (per PRD §2.2):** auto-apply (v3 / ADR-006), reputation portability (v2/v3), cross-instance jury (v3), per-community-per-peer trust (v2), OPA federation policy (v2), SSRF-isolated fetch worker (v2), federation discovery (v2).

### 2.3 Open questions — clarify-gate inputs

The PRD's 5 OQs (`OQ-FED-IN-1..5`) are mostly resolved or out of `-a` scope:

| OQ | PRD status | `-a` relevance |
|---|---|---|
| OQ-FED-IN-1 (cross-instance pseudonyms) | Open; resolve before v1 impl | **Out of `-a`** — pure admin-surface rendering concern; `-c`. `-a`'s schema stores `actor_url`/`target_url` TEXT as PRD §8.2 specifies; no pseudonym mapping in `-a`. The plan notes this deferral in §12. |
| OQ-FED-IN-2 (vanilla-Lemmy attestation target) | Resolved as proposed | No `-a` action — `-a` stores rows regardless of target instance type (advisory-only, ADR-006). |
| OQ-FED-IN-3 (replay window default 7d) | Resolved; configurable | `-a` seeds `federation.inbound.replay_window_days = 7` per PRD §10. No open decision. |
| OQ-FED-IN-4 (federation discovery) | Deferred to v2 | No `-a` action. |
| OQ-FED-IN-5 (first-contact notification) | Resolved (covered by §6.1 filter) | `-c` concern. No `-a` action. |

So `-a` carries **no unresolved PRD OQ into the plan**. The clarify-gate (advisor runs `/brehon-clarify` on THIS brief after it commits) will surface only **scope-boundary / mechanical** ambiguities, e.g.: the canonical module home for the trust-state helpers (item 4); whether the `federation_inbox_nonce` cleanup-query fn ships in `-a` or `-b` (this brief says `-a` ships the fn, `-b` schedules it — confirm); `FederationPeerId` newtype create-or-reuse (item 3). The planner does NOT file `kind: "clarify"` (advisor-only); planner ambiguities are `kind: "blocker"` or `kind: "log"` per §4.2.

### 2.4 Plan file deliverable shape

- **One file:** `.claude/PRPs/plans/v1-federation-inbound-a.plan.md`.
- **Follow `.claude/PRPs/templates/plan.template.md` 20-section schema literally.** §16a Stories mandatory.
- **PRECON-3 DoD shape — `validate-pending-laptop`, NOT Shape-G.** §15 names the cargo DoD commands verbatim with `--workspace --features full` + the migration round-trip command + the e2e test-name filter. Do NOT author the Shape-G `.github/workflows/*.yml` per-workflow §15.6 section (dormant until 2026-06-01 per DQ #229). Mirror a PRE-Shape-G plan's §15 shape (the planner can read an earlier `v1-JM-*` or `v1-SL-a` plan's §15 — but adjust to the `validate-pending-laptop` mode: the impl-task raises `kind: "validate-pending-laptop"` post-push naming these commands; the advisor laptop runs them).
- **§5 complexity-factor breakdown table** per `feedback_complexity_score_pre_split.md`. **Pre-estimate: moderate, ~9-12** (one migration with 2 enums + 4 tables + 2 ALTERs + 11 config seeds; `schema.rs` regen; 4 new model files + 2 UpdateForm extensions; trust-state helpers; ~11 config consts; ~9 entry-kind consts; one e2e extension). This is comparable to SL-a (which scored 13 and tripped the split-DQ). **The planner MUST compute the score honestly via the FILES-YAML-per-task heuristic. If score > 8, the planner files the split-or-proceed DQ** (`kind: "blocker"`, `from: "planner"`) per the template's §5 split threshold — exactly as SL-a did. The advisor expects this DQ may fire; that is normal and is resolved at plan-approval gate, not silently absorbed.
- **§13 per-task `creates:` / `modifies:` FILES YAML block** mandatory (load-bearing for cohort dispatch + the §5 complexity heuristic). `[P]` markers only where FILES-YAML disjointness genuinely holds (the migration task, the schema.rs task, the config task, the entry-kind task may be `[P]` like SL-a's Tasks 1/3/6/7 IF the planner confirms file-set disjointness; the Diesel-model task depends on schema.rs so it is NOT `[P]` with the schema task; the e2e task is last and non-`[P]`). Default lean: mirror SL-a's `[P]` decisions.
- **§16a Stories** — every story names composing §13 tasks + the (laptop-mode) DoD checkpoint + Brief-Scope outputs. Likely **3 stories**: (1) migration applies+reverts + config seeds present; (2) Diesel models + UpdateForms compile and round-trip; (3) trust-state helpers return correct trust for known/unknown peers.
- **§4 watchpoints** — every entry cites a SPECIFIC file:line / table / `schema.rs` line / migration-directory-name (per `feedback_advisor_watchpoint_specificity.md`). The 8-watchpoint seed list is in §4.1 below.
- **§6 "Relationship to other v1-federation-inbound sub-phases"** — restate the §0 carving table; confirm `-a` ships only the additive schema+model+foundation layer; `-b` wrapper + `-c` admin surface are separate later sub-phases.

**Commit only the plan file** (and any planner DQ entries — pushed immediately per `.claude/rules/decision-queue.md` "Mid-task visibility"). Plan-file commit pushes to `governance-v0` after the advisor's DoD smoke + watchpoint-specificity gates pass and the user approves (User Gate 1).

---

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract (model-enforcement, §13 FILES YAML, §5 split threshold, §16a Stories, hard refusals).
2. `.claude/PRPs/templates/plan.template.md` — canonical 20-section plan schema.
3. **`.claude/PRPs/prds/v1-federation-inbound.prd.md`** — read in FULL. Specifically: §1 (vision/maturity ladder), §2 (IN/OUT scope — the v1 boundary), §3.1 (what Phase 6 ships — verify-before-impl), §3.2/§3.3 (per-type changes + per-handler invariants — context for what the `-a` schema must support), §4 (peer-trust state model — the `-a` core), §8.0 (the 7 hard-gate preconditions — PRECON-1/2 already verified, but author the §8.2 DO-block verbatim), §8.2 (the migration SQL — `-a` ships this verbatim), §8.3 (backfill — none needed), §8.4 (Diesel models — the `-a` model layer), §10 (defaults matrix — the 11 config seeds), §11 (backwards compat — note §11.4 is `-b` not `-a`), §12 (security — context; §12.8 lint guards are out-of-`-a`), §13 (the 5 OQs — all resolved/out-of-`-a` per §2.3), §15 (Phase 6 alignment notes — confirms Phase 6 wires HTTP dispatch already; `-a` does not touch dispatch), §16 (resolutions applied 2026-04-19 — context).
4. **`.claude/PRPs/plans/v1-sponsor-liability-a.plan.md`** — read in FULL. This is the **structural mirror** (PRECON-4). Mirror its §13 task decomposition (migration / schema.rs / Diesel model / config consts / entry-kind consts / phase1_migrations_round_trip e2e / retro), its §5 complexity-factor table shape, its §15 per-task DoD shape (adjusted to `validate-pending-laptop` per PRECON-3), its §16a Stories shape, its `[P]` marker decisions. Per `.claude/rules/advisor-orchestrator.md` §3.6 + `feedback_read_canonical_before_writing_spec.md`. Cite it in the plan's §10.
5. **Phase 6 federation code** (verify-before-impl per PRD §3.1; `-a` builds ON these, do NOT duplicate or modify):
   - `crates/db_schema/src/source/governance/sanction.rs` — the canonical Diesel model shape `-a`'s 4 new models mirror (`<Type>`, `<Type>InsertForm`, `<Type>UpdateForm`).
   - `crates/db_schema/src/source/governance/remote_sanction_notice.rs` + `crates/db_schema/src/source/governance/federation_attestation.rs` (Phase 6's insert-only models) — `-a` EXTENDS these with new columns + an UpdateForm; READ the current shape first.
   - `crates/apub/activities/src/governance/inbox.rs` — Phase 6's `receive_remote_sanction_notice` (~line 105) + `receive_remote_trust_attestation` (~line 194). READ for module-layout context (to pick the canonical home for `-a`'s trust-state helpers, §2.1 item 4) — do NOT modify this file in `-a`.
   - `crates/apub/activities/src/activity_lists.rs` (~line 46, `SharedInboxActivities`) — context only; confirms Phase 6 already registers the governance types (so `-a` does not touch dispatch). Do NOT modify.
   - `crates/db_schema_file/src/schema.rs` — locate the CURRENT `remote_sanction_notice` + `federation_attestation` table blocks (so the §13 schema.rs-regen task knows the exact blocks the 2 ALTERs extend). Confirm the diesel-managed-block convention.
   - `crates/api/api/src/governance/config.rs` — the config-const + `SEEDED_KEYS` + `EXPECTED_SEED_COUNT_*` + `ConfigKeyMetadata` + parity-test pattern `-a`'s config task mirrors (also read SL-a plan Task 6 for the exact discipline).
   - `crates/api/api/src/governance/governance_log.rs` + `crates/db_schema/src/source/governance/governance_log.rs` — the ENTRY_KIND const + match-arm + dispatch pattern `-a`'s entry-kind task mirrors.
   - `.claude/rules/governance-log-entry-kind-registry.md` — the registry section `-a` appends to; READ existing entries to confirm non-collision for the ~9 new `federation_*` consts.
6. **Phase 6 migration on trunk** — `migrations/2026-04-21-000000-0000_add_federation_attestations/up.sql` (the planner reads it via `git show governance-v0:migrations/2026-04-21-000000-0000_add_federation_attestations/up.sql` — it confirms `source_instance TEXT NOT NULL` + `received_at TIMESTAMPTZ NOT NULL` per PRECON-2, and is the timestamp-ordering anchor: the `-a` migration directory MUST sort strictly after `2026-04-21-000000-0000`).
7. `crates/server/tests/e2e.rs` — locate (via `grep -n`, then READ the block) the CURRENT `phase1_migrations_round_trip` test + the canonical e2e error-shape from a recent sibling test (post-PR-1-#132 LemmyResult-unification — cite per `feedback_lemmy_error_no_std_error.md` case A/B/C). Confirm current total line count (~14,862 at HEAD `821cbe718`).
8. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — **ADR-006** (advisory-only inbound — `-a`'s schema enforces `local_case_id` NULL-able / SET NULL; the hard rule), **ADR-014** (vanilla-Lemmy interop — context; `-a` unaffected), **ADR-015** (pseudonymisation — `-a`'s schema stores `actor_url`/`target_url` TEXT; no raw-id leakage), **ADR-013** (`CaseStatus::EmergencyRemove` / enum-exhaustiveness — context; `-a` adds enums but they are new federation enums, not `CaseStatus`).
9. **Glob `.claude/lessons/feedback_*.md` and Read every file whose name keyword matches:** `lemmy_migration_runner`, `postgres_jsonb_canonicalization`, `insertform_default_propagation`, `newtype_locations`, `lemmy_error_no_std_error`, `async_pool_test_pattern`, `junior_worker_e2e_edit_hang`, `e2e_filter`, `complexity_score`, `pre_phase_dod`, `plan_dod_dry_run`, `advisor_watchpoint_specificity`, `read_canonical`, `features_full`, `features_full_p_crate`, `clippy_test_style`, `clippy_rerun_after_fix`, `laptop_default_for_validate_pending`, `validate_pending`, `shape_g`, `dogfood`, `principles_not_rules`, `plan_baseline_self_reference`, `build_what_tests_exercise`, `test_target_compile_validation`, `verify_files_with_read`, `cargo_feature_flag_propagation`, `planner_enumerate_struct_callsites`, `multi_write_handlers_need_transactions`. That is the lessons-corpus discipline (per `.claude/rules/advisor-orchestrator.md` §2.4 — planning brief, so the impl-file-class table does not auto-fire here, but these are the lessons the PLAN must bake into §4/§10/§13).
10. **Glob `.claude/lessons/reference_*.md` and Read** any matching: `governance_log_entry_kind_registry`, `phase_branch`, `branch_manager`, `worktree`, `prp_commands`.
11. **Glob `.claude/PRPs/plans/v1-sponsor-liability-{b,c,d,e}.plan.md`** for awareness of how the SL lane carved its later sub-phases (parallel precedent for how `-b`/`-c` will consume `-a`'s foundation — do NOT plan them, just understand the seam so `-a`'s §6 carving narrative is accurate).
12. `.claude/PRPs/briefs/sl-e-planning-1.md` + `.claude/PRPs/briefs/v1-ship-1-r1-planning-1.md` — exemplar planning briefs (canonical brief shape, per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate).
13. **Shape-G suspension context:** `governance-v0` commits `086cfa5d4` + `821cbe718` (the suspension + the DQ #229 re-enable reminder). Confirms PRECON-3. The plan's §15 uses the `validate-pending-laptop` shape.

---

## 4. Constraints (hard rules — violating any is a process breach)

### 4.1 Plan-content discipline

- **The §0 track carving is BINDING.** Do NOT re-carve. Do NOT pull `-b` (wrapper / handler patches / rate-limit enforcement) or `-c` (admin endpoints / pseudonym rendering) work into `-a`. Every §13 task must map to a §2.1 item; cross-check §12 against the §2.2 OUT list. If the planner believes the carve itself is wrong (e.g. the trust-state helpers genuinely cannot ship without the wrapper), STOP and file a `kind: "blocker"` DQ — re-carving is an advisor/user decision, not a planner re-derivation.
- **PRECON-1..4 are BINDING and pre-resolved.** Do NOT file blockers for: Phase 6 merge status (verified), `source_instance` TEXT vs FK (resolved — TEXT), Shape-G vs laptop DoD (laptop, per PRECON-3), the structural mirror (SL-a). The planner APPLIES these (authors the §8.2 DO-block verbatim, uses the TEXT-domain JOIN path in §4/§10, uses the `validate-pending-laptop` §15 shape, mirrors SL-a's §13/§5/§15/§16a) rather than re-deriving them.
- **PRD §8.2 SQL is authored verbatim.** The migration `up.sql` reproduces PRD §8.2's SQL exactly — the `DO $$` preamble, both `CREATE TYPE`s, all 4 `CREATE TABLE`s + their indexes, both `ALTER TABLE`s + their indexes — PLUS the 11 `governance_config` seed inserts per PRD §10 (the planner authors the seed `INSERT`s mirroring SL-a plan Task 1/6's seed shape; PRD §8.2 says "seed these rows in the migration alongside the schema changes per Phase 5a task 50's pattern"). Per `pattern_spec_schema_co_commit` + `feedback_postgres_jsonb_canonicalization.md` (the `notes JSONB DEFAULT '{}'` column — confirm canonical text rendering if any test asserts on it).
- **`schema.rs` is diesel-regenerated, not hand-edited beyond the managed block.** Per `feedback_lemmy_migration_runner.md` (forbid_diesel_cli; the project regenerates via its diesel-utils path — the planner cites the exact command SL-a plan Task 3 uses).
- **PRECON-3 DoD shape (`validate-pending-laptop`, NOT Shape-G).** §15 per the pre-Shape-G laptop shape. Do NOT author `.github/workflows/*.yml` per-workflow §15.6. Per `feedback_laptop_default_for_validate_pending.md` + `project_shape_g_suspended_2026_05_16` (PMD).
- **§16a Stories mandatory.** ~3 stories. Each names composing §13 tasks + the (laptop-mode) DoD checkpoint + Brief-Scope outputs.
- **§4 watchpoints cite specific file:line / table / `schema.rs` line / migration-directory-name at CURRENT HEAD**, never abstract concepts (per `feedback_advisor_watchpoint_specificity.md`). **Seed list (≥8):**
  1. **Migration timestamp ordering** — the `-a` migration directory name MUST sort strictly after `2026-04-21-000000-0000_add_federation_attestations`. Cite the chosen directory name + the `diesel migration run` directory-sort invariant (PRD §8.2). Wrong ordering → the `DO $$` preamble `RAISE EXCEPTION`s (correct fail-loud behaviour, but a planning error).
  2. **`source_instance` TEXT JOIN path** — `federation_peer.instance_id INT FK` joins to Phase 6 advisory rows via `instance.domain = remote_sanction_notice.source_instance` (TEXT) → `instance.id` → `federation_peer.instance_id`. Cite `crates/db_schema_file/src/schema.rs` Phase 6 table blocks + PRECON-2. Wrong JOIN assumption (treating `source_instance` as an INT FK) → `-a`'s trust helper returns wrong data.
  3. **Phase 6 model EXTENSION not REPLACEMENT** — `remote_sanction_notice.rs` + `federation_attestation.rs` get new columns + an UpdateForm ADDED; the existing Phase 6 InsertForm shape is preserved. Cite the current line of each Phase 6 model. Per `feedback_insertform_default_propagation.md` — enumerate every existing InsertForm construction site; confirm new defaulted/nullable columns don't break them.
  4. **ENTRY_KIND non-collision** — the ~9 new `federation_*` consts must not collide with v0 consts or sibling-PRD consts (governance_log dispatches by string). Cite the registry file + the parity-test pattern from SL-a plan Task 7. Per the v1-planning-queue resolution at line 318.
  5. **Config seed-count parity** — `EXPECTED_SEED_COUNT_V1_FED_IN` == 11 == const-count == migration seed-row-count. Cite `config.rs` + SL-a plan Task 6's parity test. A mismatch is a silent config-drift bug.
  6. **e2e canonical error-shape post-LemmyResult-unification** — cite the current sibling test in `e2e.rs` + its case (A/B/C per `feedback_lemmy_error_no_std_error.md`); the new test's outer return + `?`-bridge MUST mirror it verbatim (PR-1 #132 changed this idiom). Cite the anchor line for the file-end Edit.
  7. **e2e Edit-size discipline** — the new test is one anchor-Edit at file end; e2e.rs is ~14,862 lines; per `feedback_junior_worker_e2e_edit_hang.md` keep it one append well under the hang threshold. Cite the anchor pattern.
  8. **`local_case_id` advisory-only invariant (ADR-006)** — `remote_moderation_label.local_case_id INT REFERENCES moderation_case(id) ON DELETE SET NULL`, default NULL; `-a` ships NO code path that inserts a non-NULL `local_case_id` (cross-link is `-c`). Cite the PRD §8.2 column def + ADR-006. This is the load-bearing security invariant.
- **§5 complexity-factor breakdown table mandatory.** Pre-estimate ~9-12. **Compute honestly via the FILES-YAML-per-task heuristic; if > 8, file the split-or-proceed `kind: "blocker"` DQ** (`from: "planner"`) per the template §5 split threshold — exactly as SL-a did at score 13. The advisor expects this DQ; it resolves at the plan-approval gate.
- **R-rule inheritance** from prior retros (R1-R9) applies. R9 (struct-field-add caller enumeration — here, the Phase 6 InsertForm extension in §2.1 item 3 + watchpoint #3) is load-bearing.

### 4.2 Decision-queue discipline

- **Attribution integrity.** `from: "planner"` or `null`. NEVER `"advisor"`, `"user"`, `"impl-self-resolved"`, `"clarify"` (clarify is advisor-only — the advisor runs `/brehon-clarify` on THIS brief after it commits; the planner only ever writes `kind: "blocker"` or `kind: "log"`).
- **Mid-task DQ commits push immediately** to the worker branch, not at finalize (per `.claude/rules/decision-queue.md` "Mid-task visibility").
- **Boundary-of-judgment — STOP and file a `kind: "blocker"` DQ rather than guess when:**
  - The §0 carve appears genuinely wrong (the trust-state helpers cannot ship without the `-b` wrapper, or the schema cannot apply without `-b` code) — §4.1.
  - The §5 complexity score exceeds 8 — file the split-or-proceed DQ (this is EXPECTED, not exceptional).
  - A PRD §8.2 SQL statement fails the trunk `DO $$` precondition assumptions (it should not, per PRECON-1/2 — but if e.g. an enum type name differs from PRD's assumed name, surface it).
  - Any Phase 6 model file named in §3 was renamed/removed (verify-before-impl per PRD §3.1).
  - The canonical module home for the trust-state helpers (§2.1 item 4) is genuinely ambiguous after reading Phase 6's module layout.
- **Use `kind: "log"` (directly to `resolved[]`, `answered_by: "planner-self-resolved"`)** for non-blocking durable findings worth retro-harvest, e.g.: the `FederationPeerId` newtype create-vs-reuse decision (§2.1 item 3); the `federation_inbox_nonce` cleanup-fn `-a`-vs-`-b` boundary (this brief says `-a` ships the fn — record the decision); any PRD §8.2 SQL detail that needed a judgment call to translate into a Diesel migration.
- **Do NOT file `kind: "clarify"`** — advisor-only.

### 4.3 File ownership

- **Touch only:**
  - `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` (CREATE — new file).
  - `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries; push immediately).
- **Do NOT edit anything under** `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, `.github/workflows/`, or any other plan/brief file. (The plan DESCRIBES the migration + model + e2e changes; the impl-task — a later, separate Junior — makes them. The planner authors only the plan file.)
- **One commit at finalize:** `feat(plan): v1-federation-inbound-a sub-phase plan (schema + Diesel models + trust-state foundation)`.

### 4.4 Schema-first discipline

- **Read `crates/db_schema_file/src/schema.rs` (Phase 6 `remote_sanction_notice` + `federation_attestation` blocks) and the Phase 6 migration on trunk BEFORE designing §13.** Confirm at planning time: Phase 6 tables exist with `source_instance TEXT` + `received_at TIMESTAMPTZ` (PRECON-2 — already verified, re-confirm via `git show`); the Phase 6 enum type names (`attestation_type_enum`, `sanction_action_enum`, `sanction_scope_enum`) match what the PRD §8.2 `DO $$` preamble checks; the `governance_config` table shape (Phase 5a) the 11 seeds INSERT into. If any baseline assumption fails, `kind: "blocker"` DQ.

### 4.5 Cross-cutting from PMD-promoted patterns

- **`pattern_verify_before_trusting_shell_output`** — every `grep -n` hit confirmed by a direct Read of the block; every `git show` of a trunk file confirmed by reading the actual SQL.
- **`pattern_test_against_reality_not_syntax`** — the plan's §10 verbatim blocks (the migration SQL, the Diesel model shape) must match PRD §8.2 + the current Phase 6 model source exactly.
- **`pattern_cargo_feature_flag_propagation`** — §15 uses `--workspace --features full` only; never `-p <crate> --features full` (per `feedback_features_full_p_crate_incompatible.md`).
- **`pattern_spec_schema_co_commit`** — migration SQL + schema.rs + Diesel models are co-designed; the plan sequences them so each task's DoD validates the prior (migration applies → schema.rs regenerates → models compile against regenerated schema → e2e round-trips).
- **`feedback_read_canonical_before_writing_spec`** — mirror SL-a's plan structure + Phase 6's canonical Diesel model shape, not invented patterns.

### 4.6 Attribution integrity reminder

The only valid `answered_by` labels for a Junior planning subagent are `"planner"` or `null`. Never `"advisor"`, `"user"`, `"impl-self-resolved"`, `"clarify"`.

---

**Lean / advisor-side tip (not a constraint):** `-a` is the **foundation sub-phase** of the federation-inbound v1 track — the SL-a analogue. Its entire value is a clean, additive, well-tested schema+model+helper layer that `-b` (wrapper) and `-c` (admin surface) consume without surprises. The strongest property to preserve is **purely additive, zero behaviour change**: `-a` touches no HTTP path, no `receive_remote_*` function, no existing test's behaviour. `git diff governance-v0..phase-v1-federation-inbound-a` at plan-approval should show only: one new migration directory, a regenerated `schema.rs` block, 4 new Diesel model files + 2 extended Phase 6 model files, `config.rs` + `governance_log.rs` const additions, the entry-kind registry append, one `e2e.rs` anchor-Edit, the plan file, the retro. Nothing under `crates/apub/activities/`, nothing under `crates/api/api_crud/`, no new routes.

A second observation: the complexity is front-loaded into ONE migration (2 enums + 4 tables + 2 ALTERs + 11 seeds). Per SL-a precedent (scored 13, split-DQ fired, was approved at the gate after the advisor surfaced it), the planner should EXPECT the §5 score to exceed 8 and file the split-or-proceed DQ as a normal step — not agonise over whether to. The advisor resolves it at the plan-approval gate; pre-emptively splitting a coherent schema migration across sub-phases would be the wrong call (it fragments one atomic schema change).

A third observation: the `source_instance` TEXT-vs-FK question consumed real PRD-resolution effort (v1-planning-queue line 257, flagged "should be a first-order implementation warning"). It is now RESOLVED on trunk (TEXT wins). The plan's §4/§10 must encode the correct JOIN path so `-b` inherits it cleanly — getting this wrong in `-a` propagates a wrong trust-lookup into every downstream sub-phase. Watchpoint #2 is the guard.

A fourth observation: `-a` ships the `federation_inbox_nonce` table + a cleanup-query fn but NOT the cron schedule that calls it (the replay-cleanup cron is `-b`/scheduler work per PRD §7.5). This brief deliberately puts the *query fn* in `-a` (it is a pure DB helper, mirrors the model layer) and the *scheduling* in `-b`. If the planner finds this seam awkward (e.g. the fn has no caller in `-a` so clippy flags dead code), the right move is a `kind: "log"` recording the decision + a `#[allow(dead_code)]` with a `// TODO(v1-federation-inbound-b): wired by replay-cleanup cron` note — NOT pulling the cron into `-a`. Surface as `kind: "blocker"` only if clippy `-D warnings` genuinely cannot be satisfied without the caller.

A fifth observation: this is the **first sub-phase of a brand-new track with no prior federation-inbound retro to inherit R-rules from** (unlike SL-e which inherited R1-R7 from SL-a..d). The plan's §7 ("Preflight guardrails inherited from prior phases") should inherit the cross-cutting R-rules from the most-recent shipped lanes (JM, SL, AD, RT retros — R1-R9) since they are codebase-wide, not lane-specific. The federation-inbound lane will accumulate its own R-rules starting from `-a`'s retro.

---

_Brief author: advisor session (laptop CWD `C:/Users/barri/Developer/brehon-fork`, canonical checkout on `governance-v0` @ `821cbe718`, 2026-05-16). Brief committed on `governance-v0` before the Junior planning task is queued. Per `.claude/rules/advisor-orchestrator.md` §3.3 clarify gate, the advisor runs `/brehon-clarify .claude/PRPs/briefs/federation-inbound-a-planning-1.md` after this brief commits + pushes; the planning task only queues after every clarify-DQ entry on this brief is resolved. This is the first sub-phase of the federation-inbound v1 track; the advisor-carved track decomposition (§0) is binding input to the plan and is not re-litigated by the planner._
