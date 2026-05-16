# Plan: v1-federation-inbound-a — schema + Diesel models + trust-state foundation

> **DELIVERY NOTE (planner — 2026-05-16):** this file is the plan
> deliverable. The canonical target path
> `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` is blocked for
> Junior worker writes by Claude Code's built-in sensitive-file
> protection (every `.claude/**` path triggers the prompt regardless
> of `settings.json` `permissions.allow`). The advisor laptop session
> (which has full Write authority) MUST move this file to its canonical
> path BEFORE plan approval:
>
> ```
> git mv PLAN_DELIVERABLE.md .claude/PRPs/plans/v1-federation-inbound-a.plan.md
> ```
>
> The plan body is unchanged — only the path changes on `mv`. The
> escalation note at `ESCALATION_NOTE.md` (this commit) documents the
> harness gap and asks the advisor to add `.claude/PRPs/plans/**` to
> the Junior worker permission allowlist before the next planning task.
>
> **Planner DQs (#233 split-or-proceed; #234
> FederationPeerId-newtype-decision) are ALSO blocked** — Junior
> workers cannot write to `.claude/decision-queue.json` either,
> despite the path being explicitly whitelisted in `settings.json`.
> The DQ pending entries are described in §5.2 + §19 of THIS plan; the
> advisor must transcribe them to `.claude/decision-queue.json`
> directly when moving this file.

## 1. Summary

v1-federation-inbound-a ships the schema-only foundation of the federation-
inbound v1 track: one combined Postgres migration `add_federation_inbound_v1`
(2 new enums + 4 new tables + 2 `ALTER TABLE`s on Phase 6's tables + 11
`governance_config` seed rows + a fail-loud Phase-6-precondition `DO $$` SQL
preamble); `schema.rs` regeneration for the 4 new tables + 2 ALTER-added
column sets; 4 new Diesel model files (`federation_peer`,
`federation_inbox_dropped_log`, `federation_inbox_nonce`,
`remote_moderation_label`) including the `federation_inbox_check_peer_trust`
+ `federation_peer_upsert_trust` helpers (colocated in `federation_peer.rs`
per DQ #230); UpdateForm + new-column extensions to Phase 6's two
insert-only models (`remote_sanction_notice`, `federation_attestation`);
2 new newtypes (`FederationInboxDroppedLogId(pub i64)`,
`RemoteModerationLabelId(pub i32)`); 9 new `ENTRY_KIND_FEDERATION_*` const
declarations (declared-only — emitting handlers land in `-b`/`-c` per the
registry-rule pre-landed-const exemption); 11 new
`DEFAULT_FEDERATION_INBOUND_*` config consts + match arms +
`SEEDED_KEYS_WITH_CONSTS` tuples + `EXPECTED_SEED_COUNT_V1_FED_IN: usize
= 11` + `CONFIG_KEY_METADATA` entries + parity-test extension; and a
focused e2e extension to the 3 Phase-1 migration round-trip tests plus a
small trust-state foundation test exercising the
`federation_inbox_check_peer_trust` helper end-to-end via direct DB fixture.
No handler ships in `-a`. **Headline acceptance:** `git diff
governance-v0..phase-v1-federation-inbound-a` at plan-approval shows ONLY
the migration directory, the regenerated `schema.rs` blocks, 4 new Diesel
model files + 2 extended Phase-6 model files, `config.rs` +
`governance_log.rs` const additions, the entry-kind registry append, ONE
e2e.rs anchor-Edit, the plan file, the retro — and nothing under
`crates/apub/activities/`, `crates/api/api/src/governance/` admin
endpoints, or `crates/routes/`. Parity tests stay green; registry's
`rg '^pub const ENTRY_KIND_'` invariant goes **45 → 54**; the
`EXPECTED_SEED_COUNT_*` parametric sum goes **127 → 138**
(`EXPECTED_SEED_COUNT_V1_FED_IN = 11`).

## 2. Source

- `.claude/PRPs/briefs/federation-inbound-a-planning-1.md` @ `8c1d57983`
  — the advisor brief (post-`/brehon-clarify`; DQ #230 + #231 + #232
  resolved 2026-05-16, binding).
- `.claude/PRPs/prds/v1-federation-inbound.prd.md` @ `governance-v0` —
  parent PRD; §1, §2, §3.1–§3.3, §4, §6, §7, §8.0–§8.4, §10, §11.3,
  §11.4 (= `-b`), §12, §13, §15, §16.
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` @ `governance-v0`
  — **structural mirror** per brief PRECON-4 + DQ #231. SL-a's
  §13/§5/§16a/§4 shapes are mirrored; §15 is **NOT** (Shape-G; `-a`
  is laptop-shape per PRECON-3).
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  — ADR-006 (advisory-only inbound; load-bearing), ADR-014, ADR-015
  (pseudonymisation), ADR-013 (`-a` adds federation enums, not
  `CaseStatus`; v0 sweep N/A).
- `.claude/rules/governance-log-entry-kind-registry.md` —
  pre-landed-const exemption + count-invariant (45 → 54 post-`-a`) +
  reserved section `-a` populates.
- `.claude/rules/decision-queue.md` — schema-v2 attribution + Recipe 1
  / 2 + `kind: "validate-pending-laptop"` routing.
- `.claude/rules/advisor-orchestrator.md` — Stage-shape + §5.2
  `validate-pending-laptop` handler + Cohort dispatch + §G4
  classifier + Phase-2 e2e user gate.
- `.claude/PRPs/templates/plan.template.md` — 20-section schema.

### Lessons that bind §13 decisions

- `feedback_lemmy_migration_runner.md` — `cargo run -p
  lemmy_diesel_utils --features full -- migration run/revert`; never
  raw `diesel migration run`.
- `feedback_postgres_jsonb_canonicalization.md` — `notes JSONB DEFAULT
  '{}'::JSONB` cast.
- `feedback_insertform_default_propagation.md` — Phase 6's InsertForms
  have `#[derive(Default)]`; new `Option<_>` fields keep
  `..Default::default()` callers compiling. Binds Task 8 + R9.
- `feedback_newtype_locations_lemmy_db_schema_vs_file.md` — newtypes
  in `crates/db_schema/src/newtypes.rs`. Binds Task 5.
- `feedback_lemmy_error_no_std_error.md` — canonical sibling
  `v1_sl_b_fixtures` at `crates/server/tests/e2e.rs:11131-11924`
  (Case A). Task 9 mirrors verbatim.
- `feedback_complexity_score_pre_split.md` — `-a` scores **13**; DQ
  #233 filed.
- `feedback_explicit_file_arrays_on_tasks.md` — every §13 task
  carries FILES YAML.
- `feedback_parallel_cohort_dispatch.md` — Cohort A (5-way [P]) +
  Cohort B (3-way [P]).
- `feedback_cohort_validation_dependency_check.md` — Tasks 6/7/8
  declare `requires: task 2`.
- `feedback_pre_phase_dod_smoke_test.md` +
  `feedback_plan_dod_dry_run_at_write.md` — advisor smoke-test runs
  every §15 command literally.
- `feedback_features_full_p_crate_incompatible.md` — only `--workspace
  --features full`.
- `feedback_features_full_workspace_only.md` — `--features full`
  activates DbEnum + ts-rs derives.
- `feedback_wrapper_script_flag_silence.md` — all via
  `scripts/brehon/cargo-{check,clippy,test}.{bat,sh}`.
- `feedback_windows_e2e_requires_bat_wrapper.md` — `.bat` wrapper on
  Windows.
- `feedback_clippy_test_style.md` — R1 `i64::from(...)`.
- `feedback_advisor_watchpoint_specificity.md` — watchpoints cite
  concrete file:line.
- `feedback_read_canonical_before_writing_spec.md` — §10 cites Phase-6
  canonical shapes + SL-a precedent.
- `feedback_principles_not_rules.md` — score > 8 is a signal; lean
  proceed-as-one with precedent.
- `feedback_junior_worker_e2e_edit_hang.md` — Task 9 uses single
  anchor-Edits.
- `feedback_async_pool_test_pattern.md` — trust helpers take `&mut
  AsyncPgConnection`.

### Related prior plans (canonical-shape mirror)

- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — **primary
  structural mirror** (PRECON-4).
- `.claude/PRPs/plans/v1-rep-tuning-r1.plan.md` (governance-v0 SHA
  `821cbe718`) — most-recent shared-file sibling; DQ #232 append-only
  precedent.
- Phase 6 plan + retros — context-only.

## 3. Problem statement

Phase 6 (merged at PR #46) ships outbound federation publish + the
direct-call inbound `receive_remote_sanction_notice` /
`receive_remote_trust_attestation` functions for the round-trip e2e
test. The Phase 6 inbox path is wired into HTTP, but there is NO
peer-trust gate, NO rate limit, NO replay protection, NO storage cap,
NO drop-log, NO `ModerationLabel` advisory handler, and NO admin
review surface. Per PRD §3.1 + §15.4, every inbound governance
activity from any peer lands in `remote_sanction_notice` /
`federation_attestation` with no gating beyond HTTP signature
verification.

PRD §1.2 + §4 + §6 + §7 specify 5 v1 corrections — peer-trust state
overlay, schema-bound rate limiting, replay-protection nonce table,
oldest-drop storage cap, and an admin review REST surface. The
federation-inbound v1 track is carved (per advisor 2026-05-16) into
3 sub-phases:

- **`-a` (THIS PLAN)** — additive schema + model + trust-state
  read/update foundation + config seeds + entry-kind consts. Zero
  HTTP-path change.
- `-b` (future) — `wrap_governance_inbound` + per-handler patches +
  `receive_remote_moderation_label` + rate-limit / replay enforcement
  + Phase-6 round-trip-test fixture update.
- `-c` (future) — 3 admin REST endpoints + unified inbox view +
  OQ-FED-IN-1 pseudonym rendering + admin-path e2e.

Without `-a`: 4 new tables don't exist → `-b`'s wrapper has nothing
to JOIN/write to; 2 new enum types don't exist → ALTER columns can't
land; 11 config keys aren't seeded; 9 ENTRY_KIND consts don't exist;
trust-state helpers don't exist. `-a` closes all gaps in a single
additive schema-foundation sub-phase. SL-a precedent (score 13, shipped
proceed-as-one with no operational regret) proves the pattern.

## 4. Solution statement

**One Postgres migration** at
`migrations/2026-05-17-000000-0000_add_federation_inbound_v1/` ships
PRD §8.2's SQL verbatim: fail-loud `DO $$` preamble + 2 new enum types
+ 4 new tables with indexes + 2 `ALTER TABLE` blocks adding 4/6 columns
respectively + 11 `INSERT INTO governance_config … ON CONFLICT DO
NOTHING` rows. `down.sql` reverses LIFO. Directory sorts strictly after
Phase 6's `2026-04-21-000000-0000_add_federation_attestations` and
after the newest existing migration on trunk
(`2026-05-10-000300-0000_seed_v1_rt_config_keys`).

**Rust side mirrors:** `schema.rs` regenerates with 4 new `table!`
blocks + 4 added column lines on `remote_sanction_notice` + 6 added
column lines on `federation_attestation`. Four new Diesel model files
under `crates/db_schema/src/source/governance/`. Phase 6's two
insert-only model files extended with new fields + a new `*UpdateForm`
derived `AsChangeset`. Two new newtypes in
`crates/db_schema/src/newtypes.rs`. **`FederationPeerId` is NOT
created** — `federation_peer.instance_id` reuses Lemmy's existing
`InstanceId` newtype directly (mirrors `federation_blocklist`'s
pattern). Planner-self-resolved `kind: "log"` DQ #234.

**Config + entry-kind decoration:** 11 new `DEFAULT_FEDERATION_INBOUND_*`
consts + 11 match arms + 11 `SEEDED_KEYS_WITH_CONSTS` tuples + new
`EXPECTED_SEED_COUNT_V1_FED_IN: usize = 11;` + 11 new
`CONFIG_KEY_METADATA` entries + parity-test sum extension. 9 new
`ENTRY_KIND_FEDERATION_*` consts (dual-file: declared in `db_schema`,
re-exported in `api` shim) + populated registry section with 9
`(pending)` rows naming downstream `-b`/`-c` emitting handlers.
Registry count: **45 → 54**.

**Trust-state helpers (per DQ #230 BINDING):**
`federation_inbox_check_peer_trust(peer_domain, conn) -> FederationPeerTrust`
+ `federation_peer_upsert_trust(...)` colocate in
`crates/db_schema/src/source/governance/federation_peer.rs`. Pure
DB-layer code; called by `-b`'s future wrapper. TEXT-domain JOIN path
per PRECON-2: `instance.domain = peer_domain` → `instance.id` →
`federation_peer.instance_id`. Returns `Unknown` for first-seen peers.

**e2e migration round-trip extension (Task 9):** the existing 3-test
split at `crates/server/tests/e2e.rs:1199-1737` gets new post-condition
probes for `-a` effects (4 table-existence + 2 pg_type + new-column +
new-index + 2 config-key sentinels). Forward asserts presence; revert
asserts absence (the 2 wholly-new pg_type entries assert absence
because `down.sql` `DROP TYPE`s them entirely).
`MIGRATIONS_TO_REVERT_PHASE_1` slice prepended with the new directory
name.

**Trust-state foundation test (Task 9 sub-edit):** NEW
`mod v1_federation_inbound_a_fixtures` block appended at e2e.rs end
(single anchor-Edit per `feedback_junior_worker_e2e_edit_hang.md`)
mirrors `mod v1_sl_b_fixtures` (Case A — uniform `LemmyResult<()>`
outer). Two test functions exercising the trust helper via direct DB
fixture.

**DoD per PRECON-3 (`validate-pending-laptop`, NOT Shape G):** Shape G
suspended until 2026-06-01 per DQ #229. Each impl-task raises
`kind: "validate-pending-laptop"` post-push naming §15 DoD commands
verbatim with `--workspace --features full`; advisor laptop runs
sequentially via `.bat` wrapper. Phase 2 e2e selects local-vs-dispatch
at user-gate-4. NO `.github/workflows/*.yml` §15.6 section.

## 5. Metadata

- **Phase:** `v1-federation-inbound-a`
- **Branch:** `phase-v1-federation-inbound-a` (cut by BM-task before Task 1)
- **Target impl-task model:** `sonnet-4-6` (default per planning.md §5).
  Split-DQ threshold is `> 8`.
- **Estimated tasks:** 11 (Task 0 + Tasks 1-9 + Task 10 retro)
- **Estimated cargo budget:** 0 GB peak laptop-concurrent. Per
  `advisor-orchestrator.md` §5.2: laptop handler serially processes
  `validate-pending-laptop` entries. Peak ~6 GB at threshold.
- **Forbidden-window applicability:** non-binding for EliteDesk
  worker daemon (cargo runs on laptop).
- **Complexity score:** **13/10** — threshold-tripping; planner DQ
  #233 filed.

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **4** | 9 impl tasks (Tasks 1-9; Task 0 + retro excluded). `max(0, 9-5) = 4` |
| Migrations touched | +2 each | **2** | One logical migration (up + down pair) |
| Crates touched | +1 each | **4** | `lemmy_db_schema_file`, `lemmy_db_schema`, `lemmy_api`, `lemmy_server` |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | **3** | Task 9 modifies e2e.rs |
| New ADR-affecting decisions | +2 each | **0** | PRD §16 settled all decisions |
| Cargo budget peak above 6 GB | +1 per GB | **0** | Cargo runs on laptop serially; peak ~6 GB at threshold |
| **Total** | — | **13** | Threshold (Sonnet): `> 8`. **Tripped.** |

### 5.2 Split-or-proceed DQ

**DQ #233** (`from: "planner"`, `kind: "blocker"`, `answered_by: null`,
`question: "Complexity score 13 exceeds Sonnet threshold 8 — split
v1-federation-inbound-a into v1-federation-inbound-a-1 (Tasks 1-5) +
v1-federation-inbound-a-2 (Tasks 6-9) + retro, or proceed as one
plan?"`, `options: ["split", "proceed"]`, `context: "Mechanical schema-
foundation work mirroring SL-a (which shipped at score 13 proceed-as-
one). Brief §0 names -a as one atomic deliverable. Split races worker
branches on shared files (DQ #232 collision class). Dominant factors:
9 impl tasks (+4), 4 crates (+4), e2e (+3), migration (+2)."`).

**Planner lean: proceed.** SL-a precedent at 13; brief §0 atomic
carving; split races shared files. Plan ships under proceed-as-one
assumption pending DQ #233 resolution.

---

## 6. Relationship to other v1-federation-inbound sub-phases

Per brief §0 carving (advisor-decided 2026-05-16, BINDING):

| Sub-phase | Status | What it ships | `-a` relationship |
|---|---|---|---|
| **v1-federation-inbound-a (THIS)** | NOT YET CUT | Schema + 2 enums + 4 tables + 2 ALTERs + 11 seeds + 9 ENTRY_KIND consts + 2 newtypes + 4 new Diesel models + 2 extended Phase-6 models + trust helpers + e2e round-trip extension + trust foundation test | — |
| v1-federation-inbound-b | pending (after `-a`) | `wrap_governance_inbound` + per-handler patches + `receive_remote_moderation_label` + rate-limit/replay enforcement + Phase 6 round-trip fixture update + handler e2e | consumes `-a`'s helpers + tables + columns + 9 ENTRY_KIND consts + 11 config keys |
| v1-federation-inbound-c | pending (after `-b`) | 3 admin REST endpoints + view crate + admin handlers + OQ-FED-IN-1 pseudonyms + step-up + admin e2e | reads columns `-a` adds; emits 2 entry-kind consts NOT shipped by `-a` |

**Cross-PRD sequencing:** admin-dashboard-v1 PRD (config-write
surface, non-blocking); Phase 6 (merged at PR #46, strict ancestor);
v1-RT-r1 (merged 2026-05-13, most-recent shared-file sibling — `-a`'s
edits APPEND-ONLY per DQ #232).

## 7. Preflight guardrails

- **R1** — `i64::from(...)` for i32↔i64. Bound in Tasks 6 + 8 + 9.
- **R5** — Task 0 enumerates all probes explicitly (Probes 0..10).
- **R6** — uniform `--no-deps -- -D warnings`. §15.2.
- **R7** — `cargo test --no-run --workspace --features full --test e2e`
  on Tasks 6-9. §15.3.
- **R8 (SL-a retro)** — parametric `EXPECTED_SEED_COUNT_V1_*`.
  `EXPECTED_SEED_COUNT_V1_FED_IN = 11` added.
- **R9 (v1-RT-r1 retro)** — InsertForm caller enumeration before
  struct extension. Binds Task 8.
- **DQ #229** (advisor 2026-05-16, ADVISORY-LOG) — Shape G re-enable
  2026-06-01.
- **DQ #230** (advisor 2026-05-16, BINDING) — trust helpers in
  `federation_peer.rs`.
- **DQ #231** (advisor 2026-05-16, BINDING) — §15 laptop-shape, NOT
  Shape G.
- **DQ #232** (advisor 2026-05-16, BINDING) — shared-file edits
  APPEND-ONLY.

## 8. Flow design

### 8.1 Before state (governance-v0 @ `8c1d57983`)

`schema.rs`: Phase-6 `federation_attestation` (7 cols; line 429-437) +
`remote_sanction_notice` (10 cols including `received_at`; line
1185-1196). No `-a` tables/enums.

`crates/db_schema/src/source/governance/`: 2 Phase-6 insert-only model
files. No `-a` files.

`newtypes.rs`: Phase 6 typed-IDs at lines 309/315. No new typed-IDs.

`config.rs`: 127 `SEEDED_KEYS_WITH_CONSTS` entries. No
`federation.inbound.*` keys.

`governance_log.rs`: 45 `ENTRY_KIND_*` consts. Shim re-exports all 45.

Registry: `### federation-inbound-v1 (reserved …)` stub. Count: 45.

`e2e.rs`: 14,862 lines. 3-test Phase-1 round-trip at lines
1199/1399/1620. `MIGRATIONS_TO_REVERT_PHASE_1` (18 entries).

### 8.2 After state (post-`-a` phase-branch tip)

`schema.rs`: 4 new `table!` blocks + extended Phase-6 blocks. 2 new
pg_type entries.

`crates/db_schema/src/source/governance/`: 4 new model files. 2
extended.

`newtypes.rs`: 2 new typed-IDs.

`config.rs`: 138 `SEEDED_KEYS_WITH_CONSTS` entries (= 127 + 11);
`EXPECTED_SEED_COUNT_V1_FED_IN = 11`; 11 new DEFAULTs + match arms +
metadata; parity sum extended.

`governance_log.rs`: 54 `ENTRY_KIND_*` consts. Shim re-exports 54.

Registry: populated 9-row section. Count: 54.

`e2e.rs`: +1 entry in `MIGRATIONS_TO_REVERT_PHASE_1`; 3 new probe
blocks; NEW `mod v1_federation_inbound_a_fixtures` at file end.

### 8.3 Endpoint changes

**None in `-a`.** Phase 6's inbound HTTP path continues unchanged. New
`peer_trust_level_at_receipt` columns get default `'unknown'`.

---

## 9. Mandatory reading

### 9.1 Brehon design docs (P0)

- `.claude/PRPs/prds/v1-federation-inbound.prd.md` — full read.
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  — ADR-006, ADR-014, ADR-015.
- `.claude/rules/governance-log-entry-kind-registry.md`.

### 9.2 Codebase reads (P0)

- `crates/db_schema_file/src/schema.rs:429-437` — Phase 6
  `federation_attestation`. Task 2 EXTENDS.
- `crates/db_schema_file/src/schema.rs:498-510` — `instance` table
  (referenced by `federation_peer` FK). NO edit.
- `crates/db_schema_file/src/schema.rs:776-799` — `moderation_case`
  (referenced by `remote_moderation_label.local_case_id` FK). NO edit.
- `crates/db_schema_file/src/schema.rs:1185-1196` — Phase 6
  `remote_sanction_notice`. Task 2 EXTENDS.
- `crates/db_schema/src/source/governance/sanction.rs:1-48` — canonical
  full-shape Diesel model.
- `crates/db_schema/src/source/governance/remote_sanction_notice.rs:1-48`
  + `federation_attestation.rs:1-40` — Phase 6 insert-only models.
  Task 8 EXTENDS.
- `crates/db_schema/src/newtypes.rs:295-330` — Phase 6 federation
  typed-IDs block + later RT-r1 additions. Task 5 APPENDS new block.
- `crates/apub/activities/src/governance/inbox.rs:1-30 + :105-178 +
  :194-260` — Phase 6 `receive_remote_*` + DQ-6.6-inbound doc-comment.
  READ ONLY for context.
- `crates/apub/activities/src/governance/inbox.rs:155-175` —
  AsyncPgConnection access pattern.
- `crates/api/api/src/governance/config.rs:115-126` —
  `ConfigKeyMetadata` shape.
- `crates/api/api/src/governance/config.rs:774-913` — DEFAULT_*.
- `crates/api/api/src/governance/config.rs:1450-1520` —
  `SEEDED_KEYS_WITH_CONSTS`.
- `crates/api/api/src/governance/config.rs:1525-1576` —
  `EXPECTED_SEED_COUNT_*` parametric block.
- `crates/api/api/src/governance/config.rs:3160-3245` — parity module.
- `crates/api/api/src/governance/config.rs:1599-1601` —
  `ENUM_MULTI_SPONSOR_ESCAPE_RULE`.
- `crates/db_schema/src/source/governance/governance_log.rs:200-218` —
  v1-RT-r1 additions block. Task 4 INSERTS new block.
- `crates/api/api/src/governance/governance_log.rs:39-88` — shim
  `pub use` block.
- `migrations/2026-04-21-000000-0000_add_federation_attestations/up.sql`
  — Phase 6 baseline.
- `migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/up.sql`
  — SL-a precedent.
- `migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/up.sql` —
  v1-RT-r1 idempotent ON CONFLICT precedent.
- `crates/server/tests/e2e.rs:1069-1117` —
  `MIGRATIONS_TO_REVERT_PHASE_1`.
- `crates/server/tests/e2e.rs:1190-1192` —
  `phase1_revert_list_matches_disk`.
- `crates/server/tests/e2e.rs:1259-1316 + :1318-1391` — v1-SL-a +
  v1-RT-r1 forward-state probe blocks.
- `crates/server/tests/e2e.rs:1495-1542 + :1544-1612` — revert-state.
- `crates/server/tests/e2e.rs:1649-1737` — reapply body.
- `crates/server/tests/e2e.rs:11131-11924` — `mod v1_sl_b_fixtures`
  (Case A canonical).
- `crates/server/tests/e2e.rs:7385-7460` —
  `admin_dashboard_aggregates_populated_data` (R9 caller ref).

### 9.3 Rules (P0 — auto-loaded)

- `.claude/rules/governance-log-entry-kind-registry.md`,
  `decision-queue.md`, `advisor-orchestrator.md`, `branch-manager.md`,
  `phase-branch.md`, `cargo-output-capture.md`,
  `no-cargo-output-paste.md`, `pre-phase-harness-audit.md`,
  `multi-lane-worktree.md`.

### 9.4 External documentation

- diesel + diesel-async derive shapes.
- chrono `DateTime<Utc>` + `Duration::days(i64)`.
- serde_json `Value` for `notes JSONB`.

---

## 10. Patterns to mirror

### 10.1 Combined SQL migration

**Mirror primary:** PRD §8.2 verbatim. **Mirror secondary:** SL-a
migration up.sql; v1-RT-r1 seed migration ON CONFLICT pattern.

**up.sql skeleton (Task 1 IMPLEMENT — impl-task authors verbatim PRD
§8.2 SQL):**

```sql
-- v1-federation-inbound-a task 1: combined schema + enum + tables + ALTERs + seed migration.
-- ============================================================
-- ADR exception trail (federation governance tables)
-- ============================================================
-- ADDITIVE only: CREATE TYPE, CREATE TABLE, CREATE INDEX, ALTER TABLE
-- ADD COLUMN, INSERT … ON CONFLICT DO NOTHING.
-- NO DROP, NO ALTER on existing columns, NO UPDATE on Phase-6 rows
-- (PRD §8.3: Phase 6 tables empty pre-v1).
--
-- Controlling ADRs: ADR-006 (advisory-only — local_case_id SET-NULL);
-- ADR-015 (pseudonymisation — actor_url/target_url/subject_url TEXT).
--
-- Authority trail:
--   - PRD: .claude/PRPs/prds/v1-federation-inbound.prd.md §8.2
--   - Plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md §10.1, Task 1
--   - DQ #230 (advisor 2026-05-16): trust-state helpers in db_schema.
--   - DQ #232 (advisor 2026-05-16): shared-file edits append-only.
-- ============================================================

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'federation_attestation') THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 federation_attestation table (not present)';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'remote_sanction_notice') THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 remote_sanction_notice table (not present)';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'remote_sanction_notice' AND column_name = 'received_at') THEN
    RAISE EXCEPTION 'federation-inbound-v1 assumes Phase 6 DQ-6.1 resolved with remote_sanction_notice.received_at column (not present)';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'attestation_type_enum') THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 attestation_type_enum (not registered)';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'sanction_action_enum') THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 sanction_action_enum (not registered)';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'sanction_scope_enum') THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 sanction_scope_enum (not registered)';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'remote_sanction_notice' AND column_name = 'source_instance' AND data_type = 'text') THEN
    RAISE EXCEPTION 'federation-inbound-v1 assumes remote_sanction_notice.source_instance is TEXT (peer domain).';
  END IF;
END
$$;

CREATE TYPE federation_peer_trust_enum AS ENUM ('unknown', 'allowlisted', 'untrusted_receive', 'blocklisted');
CREATE TYPE federation_inbox_admin_action_enum AS ENUM ('unreviewed', 'cross_linked', 'dismissed');

CREATE TABLE federation_peer (
  instance_id     INT PRIMARY KEY REFERENCES instance(id) ON DELETE CASCADE,
  trust_level     federation_peer_trust_enum NOT NULL DEFAULT 'unknown',
  added_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  added_by_actor  TEXT,
  notes           JSONB NOT NULL DEFAULT '{}'::JSONB,
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_federation_peer_trust ON federation_peer (trust_level);

CREATE TABLE federation_inbox_dropped_log (
  id              BIGSERIAL PRIMARY KEY,
  source_instance TEXT NOT NULL,
  activity_id     TEXT,
  drop_reason     TEXT NOT NULL,
  payload_excerpt TEXT,
  dropped_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_fil_drop_source ON federation_inbox_dropped_log (source_instance, dropped_at DESC);
CREATE INDEX idx_fil_drop_reason ON federation_inbox_dropped_log (drop_reason, dropped_at DESC);

CREATE TABLE federation_inbox_nonce (
  peer_instance   TEXT NOT NULL,
  activity_id     TEXT NOT NULL,
  seen_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (peer_instance, activity_id)
);
CREATE INDEX idx_fin_nonce_seen_at ON federation_inbox_nonce (seen_at);

CREATE TABLE remote_moderation_label (
  id              SERIAL PRIMARY KEY,
  source_instance TEXT NOT NULL,
  actor_url       TEXT NOT NULL,
  target_url      TEXT NOT NULL,
  label           TEXT NOT NULL,
  summary         TEXT,
  published_at    TIMESTAMPTZ NOT NULL,
  signature       TEXT NOT NULL,
  local_case_id   INT REFERENCES moderation_case(id) ON DELETE SET NULL,
  received_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  peer_trust_level_at_receipt federation_peer_trust_enum NOT NULL DEFAULT 'unknown',
  admin_reviewed_at TIMESTAMPTZ,
  admin_action    federation_inbox_admin_action_enum NOT NULL DEFAULT 'unreviewed',
  dismissal_rationale TEXT
);
CREATE INDEX idx_rml_target ON remote_moderation_label (target_url);
CREATE INDEX idx_rml_source ON remote_moderation_label (source_instance, received_at DESC);
CREATE INDEX idx_rml_admin_action ON remote_moderation_label (admin_action) WHERE admin_action = 'unreviewed';

ALTER TABLE remote_sanction_notice
  ADD COLUMN peer_trust_level_at_receipt federation_peer_trust_enum NOT NULL DEFAULT 'unknown',
  ADD COLUMN admin_reviewed_at TIMESTAMPTZ,
  ADD COLUMN admin_action federation_inbox_admin_action_enum NOT NULL DEFAULT 'unreviewed',
  ADD COLUMN dismissal_rationale TEXT;
CREATE INDEX idx_rsn_admin_action ON remote_sanction_notice (admin_action) WHERE admin_action = 'unreviewed';

ALTER TABLE federation_attestation
  ADD COLUMN source_instance TEXT,
  ADD COLUMN received_at TIMESTAMPTZ,
  ADD COLUMN peer_trust_level_at_receipt federation_peer_trust_enum,
  ADD COLUMN admin_reviewed_at TIMESTAMPTZ,
  ADD COLUMN admin_action federation_inbox_admin_action_enum NOT NULL DEFAULT 'unreviewed',
  ADD COLUMN dismissal_rationale TEXT;
CREATE INDEX idx_fa_admin_action ON federation_attestation (admin_action) WHERE admin_action = 'unreviewed';
CREATE INDEX idx_fa_source ON federation_attestation (source_instance, received_at DESC) WHERE source_instance IS NOT NULL;

INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
  ('instance', 'federation.inbound.default_trust_for_new_peers',       'text',  NULL,   NULL,  NULL,  'unknown'),
  ('instance', 'federation.inbound.per_peer_rate_per_hour',             'int',   100,    NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.per_actor_attestation_rate_per_hour','int',   10,     NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.per_peer_storage_cap',               'int',   10000,  NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.max_payload_bytes_sanction_notice',  'int',   65536,  NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.max_payload_bytes_trust_attestation','int',   8192,   NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.max_payload_bytes_moderation_label', 'int',   8192,   NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.replay_window_days',                 'int',   7,      NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.replay_cleanup_cron_interval_minutes','int',  60,     NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.summary_max_chars',                  'int',   8000,   NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.admin_review_default_filter_days',   'int',   7,      NULL,  NULL,  NULL)
ON CONFLICT (scope, key, valid_from) DO NOTHING;
```

**down.sql skeleton (LIFO):**

```sql
DELETE FROM governance_config WHERE scope = 'instance' AND key IN (
  'federation.inbound.default_trust_for_new_peers',
  'federation.inbound.per_peer_rate_per_hour',
  'federation.inbound.per_actor_attestation_rate_per_hour',
  'federation.inbound.per_peer_storage_cap',
  'federation.inbound.max_payload_bytes_sanction_notice',
  'federation.inbound.max_payload_bytes_trust_attestation',
  'federation.inbound.max_payload_bytes_moderation_label',
  'federation.inbound.replay_window_days',
  'federation.inbound.replay_cleanup_cron_interval_minutes',
  'federation.inbound.summary_max_chars',
  'federation.inbound.admin_review_default_filter_days'
);

DROP INDEX IF EXISTS idx_fa_source;
DROP INDEX IF EXISTS idx_fa_admin_action;
ALTER TABLE federation_attestation
  DROP COLUMN IF EXISTS dismissal_rationale,
  DROP COLUMN IF EXISTS admin_action,
  DROP COLUMN IF EXISTS admin_reviewed_at,
  DROP COLUMN IF EXISTS peer_trust_level_at_receipt,
  DROP COLUMN IF EXISTS received_at,
  DROP COLUMN IF EXISTS source_instance;

DROP INDEX IF EXISTS idx_rsn_admin_action;
ALTER TABLE remote_sanction_notice
  DROP COLUMN IF EXISTS dismissal_rationale,
  DROP COLUMN IF EXISTS admin_action,
  DROP COLUMN IF EXISTS admin_reviewed_at,
  DROP COLUMN IF EXISTS peer_trust_level_at_receipt;

DROP TABLE IF EXISTS remote_moderation_label;
DROP TABLE IF EXISTS federation_inbox_nonce;
DROP TABLE IF EXISTS federation_inbox_dropped_log;
DROP TABLE IF EXISTS federation_peer;

DROP TYPE IF EXISTS federation_inbox_admin_action_enum;
DROP TYPE IF EXISTS federation_peer_trust_enum;
```

### 10.2 schema.rs additions (Task 2)

**Mirror primary:** existing `federation_attestation` block at lines
425-438 + `remote_sanction_notice` block at lines 1182-1197.

4 new `table!` blocks alphabetical near `federation_*` / `remote_*`
siblings. Example shape:

```rust
diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::FederationPeerTrustEnum;

    federation_peer (instance_id) {
        instance_id -> Int4,
        trust_level -> FederationPeerTrustEnum,
        added_at -> Timestamptz,
        added_by_actor -> Nullable<Text>,
        notes -> Jsonb,
        updated_at -> Timestamptz,
    }
}
```

2 extended Phase-6 blocks with new column lines. `joinable!` macros:
add `diesel::joinable!(federation_peer -> instance (instance_id));`
+ `diesel::joinable!(remote_moderation_label -> moderation_case
(local_case_id));`. Append 4 new tables to
`allow_tables_to_appear_in_same_query!`. Add
`// v1-federation-inbound-a additions:` inline markers.

### 10.3 Phase 6 model extensions (Task 8)

**Mirror:** v1-SL-a + v1-AD-a additions at
`moderation_case.rs:38-83`.

`remote_sanction_notice.rs` extension — 4 new fields on struct (after
`received_at` at line 33) + 4 new `Option<_>` fields on InsertForm
(after `local_case_id` at line 47) + NEW `RemoteSanctionNoticeUpdateForm`:

```rust
// In `pub struct RemoteSanctionNotice`, after received_at:
pub received_at: DateTime<Utc>,
/// v1-federation-inbound-a: peer trust at receipt time.
pub peer_trust_level_at_receipt: FederationPeerTrust,
pub admin_reviewed_at: Option<DateTime<Utc>>,
pub admin_action: FederationInboxAdminAction,
pub dismissal_rationale: Option<String>,
```

```rust
// In InsertForm, after local_case_id (all Option<_>):
pub local_case_id: Option<ModerationCaseId>,
pub peer_trust_level_at_receipt: Option<FederationPeerTrust>,
pub admin_reviewed_at: Option<DateTime<Utc>>,
pub admin_action: Option<FederationInboxAdminAction>,
pub dismissal_rationale: Option<String>,
```

```rust
#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = remote_sanction_notice))]
pub struct RemoteSanctionNoticeUpdateForm {
  pub peer_trust_level_at_receipt: Option<FederationPeerTrust>,
  pub admin_reviewed_at: Option<DateTime<Utc>>,
  pub admin_action: Option<FederationInboxAdminAction>,
  pub dismissal_rationale: Option<String>,
  pub local_case_id: Option<Option<ModerationCaseId>>,
}
```

`federation_attestation.rs` extension — mirror: 6 new struct fields +
6 new InsertForm fields + NEW `FederationAttestationUpdateForm`.

**R9 callers (enumerated at plan-time):**

- `crates/apub/activities/src/governance/inbox.rs:138` (Phase 6
  `receive_remote_sanction_notice`) — Task 8 adds `..Default::default()`.
- `crates/apub/activities/src/governance/inbox.rs:213` (Phase 6
  `receive_remote_trust_attestation`) — same.
- `crates/server/tests/e2e.rs:7450` (admin_dashboard test) — verify
  at Task 8 task start.

### 10.4 New Diesel model files (Tasks 6 + 7)

**Mirror:** `sanction.rs` (full shape).

**`federation_peer.rs` (Task 6 — full shape + trust helpers per DQ #230):**

```rust
use crate::newtypes::InstanceId;
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::FederationPeerTrust;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::{federation_peer, instance};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
#[cfg(feature = "full")]
use diesel_async::{AsyncPgConnection, RunQueryDsl};
#[cfg(feature = "full")]
use lemmy_utils::error::LemmyResult;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_peer))]
#[cfg_attr(feature = "full", diesel(primary_key(instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Per-peer trust-state overlay on Lemmy's `instance` table.
pub struct FederationPeer {
  pub instance_id: InstanceId,
  pub trust_level: FederationPeerTrust,
  pub added_at: DateTime<Utc>,
  pub added_by_actor: Option<String>,
  pub notes: Value,
  pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_peer))]
pub struct FederationPeerInsertForm {
  pub instance_id: InstanceId,
  pub trust_level: Option<FederationPeerTrust>,
  pub added_by_actor: Option<String>,
  pub notes: Option<Value>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = federation_peer))]
pub struct FederationPeerUpdateForm {
  pub trust_level: Option<FederationPeerTrust>,
  pub added_by_actor: Option<Option<String>>,
  pub notes: Option<Value>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(feature = "full")]
pub async fn federation_inbox_check_peer_trust(
  peer_domain: &str,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<FederationPeerTrust> {
  let result: Option<FederationPeerTrust> = federation_peer::table
    .inner_join(instance::table.on(federation_peer::instance_id.eq(instance::id)))
    .filter(instance::domain.eq(peer_domain))
    .select(federation_peer::trust_level)
    .first(conn)
    .await
    .optional()?;
  Ok(result.unwrap_or(FederationPeerTrust::Unknown))
}

#[cfg(feature = "full")]
pub async fn federation_peer_upsert_trust(
  instance_id: InstanceId,
  trust_level: FederationPeerTrust,
  added_by_actor: Option<&str>,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<FederationPeer> {
  let form = FederationPeerInsertForm {
    instance_id,
    trust_level: Some(trust_level),
    added_by_actor: added_by_actor.map(String::from),
    notes: None,
  };
  let row = diesel::insert_into(federation_peer::table)
    .values(&form)
    .on_conflict(federation_peer::instance_id)
    .do_update()
    .set((
      federation_peer::trust_level.eq(trust_level),
      federation_peer::updated_at.eq(diesel::dsl::now),
    ))
    .returning(FederationPeer::as_returning())
    .get_result(conn)
    .await?;
  Ok(row)
}
```

**`federation_inbox_dropped_log.rs` (Task 6 — insert-only):**

```rust
use crate::newtypes::FederationInboxDroppedLogId;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::federation_inbox_dropped_log;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_inbox_dropped_log))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct FederationInboxDroppedLog {
  pub id: FederationInboxDroppedLogId,
  pub source_instance: String,
  pub activity_id: Option<String>,
  pub drop_reason: String,
  pub payload_excerpt: Option<String>,
  pub dropped_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_inbox_dropped_log))]
pub struct FederationInboxDroppedLogInsertForm {
  pub source_instance: String,
  pub activity_id: Option<String>,
  pub drop_reason: String,
  pub payload_excerpt: Option<String>,
}
```

**`federation_inbox_nonce.rs` (Task 7 — insert-only + cleanup fn):**

```rust
use chrono::{DateTime, Duration, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::federation_inbox_nonce;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use diesel::{ExpressionMethods, QueryDsl};
#[cfg(feature = "full")]
use diesel_async::{AsyncPgConnection, RunQueryDsl};
#[cfg(feature = "full")]
use lemmy_utils::error::LemmyResult;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_inbox_nonce))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct FederationInboxNonce {
  pub peer_instance: String,
  pub activity_id: String,
  pub seen_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_inbox_nonce))]
pub struct FederationInboxNonceInsertForm {
  pub peer_instance: String,
  pub activity_id: String,
}

#[cfg(feature = "full")]
#[allow(dead_code)]  // TODO(v1-federation-inbound-b): wired by replay-cleanup cron
pub async fn delete_older_than(
  window_days: i64,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<usize> {
  let cutoff = Utc::now() - Duration::days(window_days);
  let count = diesel::delete(
    federation_inbox_nonce::table.filter(federation_inbox_nonce::seen_at.lt(cutoff)),
  )
  .execute(conn)
  .await?;
  Ok(count)
}
```

**`remote_moderation_label.rs` (Task 7 — full shape):**

```rust
use crate::newtypes::{ModerationCaseId, RemoteModerationLabelId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::{FederationInboxAdminAction, FederationPeerTrust};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::remote_moderation_label;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = remote_moderation_label))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RemoteModerationLabel {
  pub id: RemoteModerationLabelId,
  pub source_instance: String,
  pub actor_url: String,
  pub target_url: String,
  pub label: String,
  pub summary: Option<String>,
  pub published_at: DateTime<Utc>,
  pub signature: String,
  pub local_case_id: Option<ModerationCaseId>,
  pub received_at: DateTime<Utc>,
  pub peer_trust_level_at_receipt: FederationPeerTrust,
  pub admin_reviewed_at: Option<DateTime<Utc>>,
  pub admin_action: FederationInboxAdminAction,
  pub dismissal_rationale: Option<String>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = remote_moderation_label))]
pub struct RemoteModerationLabelInsertForm {
  pub source_instance: String,
  pub actor_url: String,
  pub target_url: String,
  pub label: String,
  pub summary: Option<String>,
  pub published_at: DateTime<Utc>,
  pub signature: String,
  pub local_case_id: Option<ModerationCaseId>,
  pub peer_trust_level_at_receipt: Option<FederationPeerTrust>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = remote_moderation_label))]
pub struct RemoteModerationLabelUpdateForm {
  pub local_case_id: Option<Option<ModerationCaseId>>,
  pub admin_reviewed_at: Option<DateTime<Utc>>,
  pub admin_action: Option<FederationInboxAdminAction>,
  pub dismissal_rationale: Option<String>,
}
```

### 10.5 Trust-state helper module location (DQ #230 — BINDING)

Helpers colocate inside `federation_peer.rs` per DQ #230. Rationale:
Phase 6's `inbox.rs:1-30` doc-comment + DQ-6.6-inbound resolved id 37
documents that PRD §9.1's `crates/apub/apub/src/governance/` location
is unreachable for the AP-framework-invoked code path. `-a`'s helpers
are PURE DB-layer code (called by `-b`'s future wrapper which lives at
the `lemmy_apub_activities` layer); they belong at the data layer.
Access pattern at `-b`'s call site:

```rust
let pool = &mut context.pool();
let conn = &mut get_conn(pool).await?;
let trust = federation_inbox_check_peer_trust(peer_domain, conn).await?;
```

### 10.6 config.rs 11-key extension (Task 3)

**Mirror:** SL-a §10.6 + v1-RT-r1 at config.rs:1498-1519.

**11 new DEFAULT_***:

```rust
pub const DEFAULT_FEDERATION_INBOUND_DEFAULT_TRUST_FOR_NEW_PEERS: &str = "unknown";
pub const DEFAULT_FEDERATION_INBOUND_PER_PEER_RATE_PER_HOUR: i64 = 100;
pub const DEFAULT_FEDERATION_INBOUND_PER_ACTOR_ATTESTATION_RATE_PER_HOUR: i64 = 10;
pub const DEFAULT_FEDERATION_INBOUND_PER_PEER_STORAGE_CAP: i64 = 10_000;
pub const DEFAULT_FEDERATION_INBOUND_MAX_PAYLOAD_BYTES_SANCTION_NOTICE: i64 = 65_536;
pub const DEFAULT_FEDERATION_INBOUND_MAX_PAYLOAD_BYTES_TRUST_ATTESTATION: i64 = 8_192;
pub const DEFAULT_FEDERATION_INBOUND_MAX_PAYLOAD_BYTES_MODERATION_LABEL: i64 = 8_192;
pub const DEFAULT_FEDERATION_INBOUND_REPLAY_WINDOW_DAYS: i64 = 7;
pub const DEFAULT_FEDERATION_INBOUND_REPLAY_CLEANUP_CRON_INTERVAL_MINUTES: i64 = 60;
pub const DEFAULT_FEDERATION_INBOUND_SUMMARY_MAX_CHARS: i64 = 8_000;
pub const DEFAULT_FEDERATION_INBOUND_ADMIN_REVIEW_DEFAULT_FILTER_DAYS: i64 = 7;
```

11 match arms (10 int + 1 text). 11 SEEDED_KEYS tuples alphabetised
within new block. `EXPECTED_SEED_COUNT_V1_FED_IN: usize = 11` after
`EXPECTED_SEED_COUNT_V1_RT` at line 1576. `ENUM_FEDERATION_PEER_TRUST:
&[&str] = &["unknown", "untrusted_receive", "blocklisted"]` after
`ENUM_MULTI_SPONSOR_ESCAPE_RULE` at line 1601. Parity-test sum
extension + error-message extension.

11 CONFIG_KEY_METADATA per-key:

| Key | value_type | valid_range | valid_enum | scope | apply_at |
|---|---|---|---|---|---|
| `federation.inbound.default_trust_for_new_peers` | Text | None | Some(ENUM_FEDERATION_PEER_TRUST) | Instance | Immediate |
| `federation.inbound.per_peer_rate_per_hour` | Int | 1.0..=100000.0 | None | Instance | Immediate |
| `federation.inbound.per_actor_attestation_rate_per_hour` | Int | 1.0..=10000.0 | None | Instance | Immediate |
| `federation.inbound.per_peer_storage_cap` | Int | 100.0..=1000000.0 | None | Instance | Immediate |
| `federation.inbound.max_payload_bytes_sanction_notice` | Int | 1024.0..=1048576.0 | None | Instance | Immediate |
| `federation.inbound.max_payload_bytes_trust_attestation` | Int | 1024.0..=65536.0 | None | Instance | Immediate |
| `federation.inbound.max_payload_bytes_moderation_label` | Int | 1024.0..=65536.0 | None | Instance | Immediate |
| `federation.inbound.replay_window_days` | Int | 1.0..=30.0 | None | Instance | Immediate |
| `federation.inbound.replay_cleanup_cron_interval_minutes` | Int | 5.0..=1440.0 | None | Instance | Immediate (clokwerk pins at startup) |
| `federation.inbound.summary_max_chars` | Int | 256.0..=32000.0 | None | Instance | Immediate |
| `federation.inbound.admin_review_default_filter_days` | Int | 0.0..=365.0 | None | Instance | Immediate |

All `requires_re_jury: false`, `requires_step_up: false`,
`doc_anchor: "v1-federation-inbound.prd.md§10"`.

### 10.7 ENTRY_KIND dual-file edit + registry (Task 4)

9 consts in `crates/db_schema/src/source/governance/governance_log.rs`
(new block after line 218, alphabetical within block):

```rust
pub const ENTRY_KIND_FEDERATION_INBOUND_BLOCKED: &str = "federation_inbound_blocked";
pub const ENTRY_KIND_FEDERATION_INBOUND_DROPPED_OVERSIZE: &str = "federation_inbound_dropped_oversize";
pub const ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_ACTOR: &str = "federation_inbound_dropped_rate_limit_actor";
pub const ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_PEER: &str = "federation_inbound_dropped_rate_limit_peer";
pub const ENTRY_KIND_FEDERATION_INBOUND_DROPPED_REPLAY: &str = "federation_inbound_dropped_replay";
pub const ENTRY_KIND_FEDERATION_INBOUND_DROPPED_SCHEMA: &str = "federation_inbound_dropped_schema";
pub const ENTRY_KIND_FEDERATION_INBOUND_DROPPED_STORAGE_CAP_EVICTED: &str = "federation_inbound_dropped_storage_cap_evicted";
pub const ENTRY_KIND_FEDERATION_LABEL_RECEIVED: &str = "federation_label_received";
pub const ENTRY_KIND_FEDERATION_PEER_TRUST_CHANGED: &str = "federation_peer_trust_changed";
```

9 alphabetical `pub use` re-exports in
`crates/api/api/src/governance/governance_log.rs` lines 39-88.

Registry section (replace `### federation-inbound-v1 (reserved …)`
stub):

```markdown
## v1-federation-inbound-a entry kinds (9, this sub-phase)

Landed alongside task 4's dual-file edit. v1-federation-inbound-a
writes the const declarations only; emitting call sites land in
v1-federation-inbound-b's wrapper + handler patches per the registry
rule's pre-landed-const exemption.

| Rust const | `&str` value | Source | Emitting handler | Semantic |
|---|---|---|---|---|
| `ENTRY_KIND_FEDERATION_INBOUND_BLOCKED` | `federation_inbound_blocked` | -a const; -b call site | -b `inbox.rs::wrap_governance_inbound` peer-trust Blocklisted branch (pending) | Inbound from Blocklisted peer rejected; HTTP 403. Per PRD §3.2 + §5.3. |
| `ENTRY_KIND_FEDERATION_INBOUND_DROPPED_OVERSIZE` | `federation_inbound_dropped_oversize` | -a const; -b call site | -b `wrap_governance_inbound` size-check (pending) | Payload exceeded cap; HTTP 413. Per PRD §3.3 + §7.4. |
| `ENTRY_KIND_FEDERATION_INBOUND_DROPPED_SCHEMA` | `federation_inbound_dropped_schema` | -a const; -b call site | -b `wrap_governance_inbound` schema-check (pending) | Strict-deserialisation rejected; HTTP 400. Per PRD §3.3 + §7.4. |
| `ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_PEER` | `federation_inbound_dropped_rate_limit_peer` | -a const; -b call site | -b `wrap_governance_inbound` per-peer-rate (pending) | Per-peer rate exceeded; HTTP 429. Per PRD §7.1. |
| `ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_ACTOR` | `federation_inbound_dropped_rate_limit_actor` | -a const; -b call site | -b `wrap_governance_inbound` per-actor-rate (pending) | Per-actor rate exceeded; HTTP 429. Per PRD §7.2. |
| `ENTRY_KIND_FEDERATION_INBOUND_DROPPED_REPLAY` | `federation_inbound_dropped_replay` | -a const; -b call site | -b `wrap_governance_inbound` replay-check (pending) | Activity ID seen within window; HTTP 409. Per PRD §7.5. |
| `ENTRY_KIND_FEDERATION_INBOUND_DROPPED_STORAGE_CAP_EVICTED` | `federation_inbound_dropped_storage_cap_evicted` | -a const; -b call site | -b `wrap_governance_inbound` storage-cap (pending) | Storage cap reached; oldest row evicted. Per PRD §7.3. |
| `ENTRY_KIND_FEDERATION_PEER_TRUST_CHANGED` | `federation_peer_trust_changed` | -a const; -c call site | -c admin POST .../peers/{instance_id}/trust handler (pending — owned by -c) | Admin flipped a peer's trust state. Per PRD §4.3 + §12.5. |
| `ENTRY_KIND_FEDERATION_LABEL_RECEIVED` | `federation_label_received` | -a const; -b call site | -b `inbox.rs::receive_remote_moderation_label` (pending — fills Phase 6 stub) | Inbound moderation label persisted. Per PRD §3.2 + §9.4. |

**Deferred (NOT shipped in `-a`):**

- `federation_inbound_persist_failed` (PRD §5.3 row 7) — `-b`.
- `federation_inbound_cross_linked` (PRD §6.2) — `-c`.
- `federation_inbound_dismissed` (PRD §6.3) — `-c`.
```

Acceptance invariants count: update `45` → `54`.

### 10.8 e2e migration round-trip extension (Task 9)

**Mirror:** SL-a §10.8 + v1-RT-r1 precedent at lines
1318-1391 / 1544-1612.

1. `MIGRATIONS_TO_REVERT_PHASE_1` (line 1091-1117): prepend
   `"2026-05-17-000000-0000_add_federation_inbound_v1",` + group-
   comment `// v1-federation-inbound-a (1 migration, bump 18 → 19)`.

2. `test_phase1_migrations_forward` extension (before `Ok(())` at line
   1393): append new block with 4 table-existence + 2 pg_type +
   4+6 column + 10 index + 2 config-key probes.

3. `test_phase1_migrations_revert` extension (before `Ok(())` at line
   1614): append mirror asserting ABSENCE.

4. `test_phase1_migrations_reapply` extension (before `Ok(())` at line
   1736): small re-run block.

5. At END of file, append NEW module `mod v1_federation_inbound_a_fixtures`:

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

**Case A** per `feedback_lemmy_error_no_std_error.md` — uniform
`LemmyResult<()>` outer; every helper `LemmyResult<T>`; every `?`
bare; one `map_err` on testcontainer call (anyhow bridge per v1-SL-b
precedent).

---

## 11. Files to change

### `lemmy_db_schema_file` crate

- `crates/db_schema_file/src/schema.rs` — 4 new `table!` blocks + 2
  extended blocks + 2 `joinable!` + 4 entries in
  `allow_tables_to_appear_in_same_query!`. **Task 2**.

### `lemmy_db_schema` crate

- `crates/db_schema/src/source/governance/federation_peer.rs` —
  CREATE. **Task 6**.
- `crates/db_schema/src/source/governance/federation_inbox_dropped_log.rs`
  — CREATE. **Task 6**.
- `crates/db_schema/src/source/governance/federation_inbox_nonce.rs`
  — CREATE. **Task 7**.
- `crates/db_schema/src/source/governance/remote_moderation_label.rs`
  — CREATE. **Task 7**.
- `crates/db_schema/src/source/governance/remote_sanction_notice.rs`
  — EXTEND. **Task 8**.
- `crates/db_schema/src/source/governance/federation_attestation.rs`
  — EXTEND. **Task 8**.
- `crates/db_schema/src/source/governance/mod.rs` — 4 new `pub mod`
  lines. **Tasks 6 + 7**.
- `crates/db_schema/src/source/governance/governance_log.rs` — 9 new
  const declarations. **Task 4**.
- `crates/db_schema/src/newtypes.rs` — 2 new typed-IDs. **Task 5**.

### `lemmy_api` crate

- `crates/api/api/src/governance/governance_log.rs` — 9 new `pub use`
  re-exports. **Task 4**.
- `crates/api/api/src/governance/config.rs` — 11 new DEFAULT_* + 11
  match arms + 11 SEEDED_KEYS + `EXPECTED_SEED_COUNT_V1_FED_IN` +
  `ENUM_FEDERATION_PEER_TRUST` + 11 CONFIG_KEY_METADATA + parity sum.
  **Task 3**.

### `lemmy_server` crate

- `crates/server/tests/e2e.rs` — `MIGRATIONS_TO_REVERT_PHASE_1` +1; 3
  new probe blocks; NEW test module appended at file end. **Task 9**.

### Migration files

- `migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql` +
  `down.sql` — CREATE. **Task 1**.

### Meta files

- `.claude/rules/governance-log-entry-kind-registry.md` — REPLACE
  reserved stub + count update. **Task 4**.
- `.claude/PRPs/reports/v1-federation-inbound-a-retro.md` — CREATE.
  **Task 10**.

### R9 InsertForm callsites

- `crates/apub/activities/src/governance/inbox.rs:138` + `:213` —
  add `..Default::default()` spread (if missing). **Task 8**.
- `crates/server/tests/e2e.rs:7450` — verify spread (already touched
  by Task 9 if needed). **Task 8 or Task 9**.

### Files explicitly NOT touched

- `crates/apub/activities/src/governance/inbox.rs` body (except
  spread).
- `crates/apub/activities/src/governance/publish_*.rs` — `-b`.
- `crates/apub/apub/src/governance/` — `-b`.
- `crates/api/api/src/governance/federation_inbox/` — `-c`.
- `crates/api/routes/src/lib.rs` — no new routes.
- `crates/db_views/federation_inbox/` — `-c`.
- `crates/utils/src/error.rs` — no new variants.
- `Cargo.toml` / `Cargo.lock` / `rust-toolchain.toml` / `.coderabbit.yaml`.
- `.github/workflows/**` (PRECON-3; Shape G dormant).
- `migrations/**` other than the new `-a` directory.

---

## 12. NOT building in v1-federation-inbound-a

Per brief §0 + §2.2:

- **`wrap_governance_inbound`** (PRD §9.2) — `-b`.
- **Per-handler patches** (PRD §9.3) — `-b`.
- **`receive_remote_moderation_label` HTTP body** (PRD §9.4) — `-b`.
- **Rate-limit / replay enforcement** (PRD §7) — `-b`.
- **Admin REST endpoints** (PRD §6) — `-c`.
- **Phase 6 round-trip fixture update** (PRD §11.4) — `-b`.
- **OQ-FED-IN-1 pseudonym rendering** — `-c`.
- **`webauthn-rs` step-up** (PRD §12.5) — `-c`.
- **Outbound change** (PRD §11.1) — unchanged.
- **Fork-local lint guards** (PRD §12.8) — meta; out of `-a`.
- **3 deferred entry-kind consts** — `_persist_failed` (`-b`),
  `_cross_linked` + `_dismissed` (`-c`).
- **`FederationPeerId` newtype** — reuses `InstanceId`; DQ #234.
- **Behavioural e2e tests** — `-b`/`-c`.
- **Hard out-of-scope** (PRD §2.2): auto-apply (v3), reputation
  portability (v2/v3), cross-instance jury (v3), per-community-per-peer
  trust (v2), OPA (v2), SSRF-isolated fetch worker (v2), federation
  discovery (v2).

---

## 13. Step-by-step tasks

> **Cohort dispatch:** Cohort A = Tasks 1+2+3+4+5 (5-way [P]); Cohort
> B = Tasks 6+7+8 (3-way [P]; all `requires: task 2`). Tasks 0, 9, 10
> barriers.
>
> **PRECON-3 laptop-shape (NOT Shape G):** each impl-task raises
> `kind: "validate-pending-laptop"` post-push naming §15 DoD commands
> verbatim.

### Task 0: Pre-flight harness audit + branch verification

**FILES:**

```yaml
creates: []
modifies: []
```

**Probes (per `pre-phase-harness-audit.md` — R5):**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe -1 — submodule init
git submodule status > /tmp/fed-in-a-task0-submodule.log 2>&1
if grep -q '^-' /tmp/fed-in-a-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/fed-in-a-task0-submodule-init.log 2>&1
fi

# Probe 1 — branch
git branch --show-current
# EXPECT: phase-v1-federation-inbound-a

# Probe 2 — baseline counts
echo "ENTRY_KIND_ count (expect 45):"
grep -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
echo "EXPECTED_SEED_COUNT totals (expect 127):"
grep -nE '^pub const EXPECTED_SEED_COUNT' crates/api/api/src/governance/config.rs

# Probe 3 — schema clean of -a tables
grep -E 'federation_peer \(|federation_inbox_dropped_log \(|federation_inbox_nonce \(|remote_moderation_label \(' crates/db_schema_file/src/schema.rs && {
  echo "ERROR: -a tables already in schema.rs — contamination"; exit 1;
} || echo "schema.rs clean"

# Probe 4 — R9 InsertForm caller enumeration
grep -rnE 'RemoteSanctionNoticeInsertForm\s*\{' crates/ tests/ 2>/dev/null > /tmp/fed-in-a-task0-rsn.log
cat /tmp/fed-in-a-task0-rsn.log
grep -rnE 'FederationAttestationInsertForm\s*\{' crates/ tests/ 2>/dev/null > /tmp/fed-in-a-task0-fa.log
cat /tmp/fed-in-a-task0-fa.log

# Probe 5 — trunk clean of -a enums
grep -rE 'federation_peer_trust_enum|federation_inbox_admin_action_enum' migrations/ 2>/dev/null && {
  echo "ERROR: -a enum already in migration"; exit 1;
} || echo "trunk clean of -a enums"

# Probe 6 — DQ pending
python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', [(e['id'], e.get('kind')) for e in d.get('pending',[])])"

# Probe 7 — PM-plugin-hooks-stable
for h in local_private_message_before_create local_private_message_after_create \
  local_private_message_before_update local_private_message_after_update \
  federated_private_message_before_receive federated_private_message_after_receive; do
  grep -rqE "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done

# Probe 8 — concurrent-PR check
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("crates/db_schema_file/src/schema\\.rs|crates/db_schema/src/source/governance/governance_log\\.rs|crates/api/api/src/governance/config\\.rs|crates/api/api/src/governance/governance_log\\.rs|crates/db_schema/src/newtypes\\.rs|crates/db_schema/src/source/governance/remote_sanction_notice\\.rs|crates/db_schema/src/source/governance/federation_attestation\\.rs|crates/server/tests/e2e\\.rs|migrations/2026-05-17|\\.claude/rules/governance-log-entry-kind-registry\\.md")) | {number, title, headRefName}'

# Probe 9 — migration timestamp slot
ls migrations/ | sort | tail -3
# EXPECT: 2026-05-10 newest. If 2026-05-17 already exists, pick 2026-05-18.

# Probe 10 — wrapper availability
ls scripts/brehon/cargo-check.sh scripts/brehon/cargo-check.bat 2>&1 | head -3
```

**EXPECT:** Probes 0-5, 7-10 exit 0; Probe 6 informational. On
contamination, file `kind: "blocker"` DQ.

**No commit.**

### Task 1 [P]: CREATE migration `2026-05-17-000000-0000_add_federation_inbound_v1/{up,down}.sql`

**FILES:**

```yaml
creates:
  - migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql
  - migrations/2026-05-17-000000-0000_add_federation_inbound_v1/down.sql
modifies: []
```

**IMPLEMENT:** §10.1 verbatim (up.sql + down.sql skeletons).

**MIRROR:** §10.1; PRD §8.2 verbatim; SL-a + v1-RT-r1 precedents.

**GOTCHA:** directory MUST sort after
`2026-05-10-000300-0000_seed_v1_rt_config_keys`. Re-verify; pick
`2026-05-18` if collision.

**GOTCHA:** NO `-- no-transaction` directive (no `ALTER TYPE ADD VALUE`).

**GOTCHA (ADR-006):** `remote_moderation_label.local_case_id` SET-NULL.
ZERO non-NULL inserts in `-a`.

**GOTCHA (ADR-015):** `actor_url`/`target_url` as TEXT.

**GOTCHA (JSONB):** `federation_peer.notes JSONB NOT NULL DEFAULT
'{}'::JSONB` — `::JSONB` cast mandatory.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task1-check.log 2>&1"'
  - 'bash scripts/brehon/migrate-roundtrip.sh > .claude/PRPs/debug/fed-in-a-task1-migrate.log 2>&1'
```

**COMMIT:** `feat(v1-federation-inbound-a): combined migration — 2 enums + 4 tables + ALTER Phase-6 tables + 11 config seeds (task 1)`

### Task 2 [P]: UPDATE `crates/db_schema_file/src/schema.rs`

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema_file/src/schema.rs
```

**IMPLEMENT:** per §10.2 — 4 new `table!` blocks + extend 2 Phase-6
blocks + 2 `joinable!` + 4 entries in
`allow_tables_to_appear_in_same_query!`. Add
`// v1-federation-inbound-a additions:` inline markers.

**MIRROR:** §10.2; Phase 6 blocks.

**GOTCHA (DQ #232):** APPEND-ONLY.

**GOTCHA (no `FederationPeerId`):** use `InstanceId`.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task2-check.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-a): schema.rs — 4 new tables + extend Phase-6 blocks (task 2)`

### Task 3 [P]: UPDATE `crates/api/api/src/governance/config.rs`

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/config.rs
```

**IMPLEMENT:** per §10.6 — 11 DEFAULT_* + 11 match arms + 11
SEEDED_KEYS + `EXPECTED_SEED_COUNT_V1_FED_IN` +
`ENUM_FEDERATION_PEER_TRUST` + 11 CONFIG_KEY_METADATA + parity sum
extension.

**MIRROR:** §10.6; v1-RT-r1 SEEDED_KEYS at lines 1498-1519.

**GOTCHA (DQ #232):** APPEND-ONLY.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task3-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-a-task3-clippy.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-a): config.rs — 11 federation.inbound.* keys + EXPECTED_SEED_COUNT_V1_FED_IN + metadata + parity (task 3)`

### Task 4 [P]: UPDATE `governance_log.rs` × 2 + entry-kind registry

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema/src/source/governance/governance_log.rs
  - crates/api/api/src/governance/governance_log.rs
  - .claude/rules/governance-log-entry-kind-registry.md
```

**IMPLEMENT:** per §10.7 — 9 new consts (db_schema) + 9 alphabetical
`pub use` (api shim) + populate registry stub + count update 45 → 54.

**MIRROR:** §10.7.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task4-check.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-a): governance_log + entry-kind registry — 9 new federation_inbound_* consts (task 4)`

### Task 5 [P]: UPDATE `crates/db_schema/src/newtypes.rs`

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema/src/newtypes.rs
```

**IMPLEMENT:** append AFTER v1-RT-r1 block:

```rust
// ========================================================================
// Federation inbound governance typed IDs (v1-federation-inbound-a)
// ========================================================================

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct FederationInboxDroppedLogId(pub i64);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RemoteModerationLabelId(pub i32);
```

**MIRROR:** Phase 6 federation typed-IDs lines 295-330.

**GOTCHA (BIGSERIAL → i64):** `federation_inbox_dropped_log.id` is
BIGSERIAL. `remote_moderation_label.id` is SERIAL = i32.
`federation_peer` keys on `instance_id` (existing `InstanceId`);
`federation_inbox_nonce` composite PK (no id column).

**GOTCHA (no `FederationPeerId` per DQ #234).**

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task5-check.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-a): newtypes.rs — FederationInboxDroppedLogId + RemoteModerationLabelId (task 5)`

### Task 6: CREATE federation_peer.rs + federation_inbox_dropped_log.rs

**Cohort B.**

**FILES:**

```yaml
creates:
  - crates/db_schema/src/source/governance/federation_peer.rs
  - crates/db_schema/src/source/governance/federation_inbox_dropped_log.rs
modifies:
  - crates/db_schema/src/source/governance/mod.rs
requires:
  - task: 2
    reason: "table_name + check_for_backend(Pg) typecheck requires new table blocks in schema.rs"
  - task: 5
    reason: "federation_inbox_dropped_log.rs imports FederationInboxDroppedLogId from newtypes.rs"
```

**IMPLEMENT:** §10.4 — `federation_peer.rs` (model + InsertForm +
UpdateForm + 2 trust helpers per DQ #230); `federation_inbox_dropped_log.rs`
(insert-only); mod.rs append.

**MIRROR:** §10.4 + §10.5.

**GOTCHA (DQ #230):** trust helpers in `federation_peer.rs`.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task6-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-a-task6-clippy.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-a): federation_peer + dropped_log Diesel models + trust-state helpers (task 6)`

### Task 7 [P]: CREATE federation_inbox_nonce.rs + remote_moderation_label.rs

**Cohort B.**

**FILES:**

```yaml
creates:
  - crates/db_schema/src/source/governance/federation_inbox_nonce.rs
  - crates/db_schema/src/source/governance/remote_moderation_label.rs
modifies:
  - crates/db_schema/src/source/governance/mod.rs
requires:
  - task: 2
    reason: "table_name typecheck"
  - task: 5
    reason: "remote_moderation_label.rs imports RemoteModerationLabelId from newtypes.rs"
```

**IMPLEMENT:** §10.4 — `federation_inbox_nonce.rs` (insert-only +
`delete_older_than` with `#[allow(dead_code)]` + TODO comment);
`remote_moderation_label.rs` (full shape); mod.rs append.

**MIRROR:** §10.4.

**GOTCHA (mod.rs overlap with Task 6):** append-only at disjoint
positions; finalize-merge resolves trivially. Degrade to serial if
cohort-dispatch refuses overlap.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task7-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-a-task7-clippy.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-a): federation_inbox_nonce + remote_moderation_label Diesel models (task 7)`

### Task 8 [P]: EXTEND Phase 6 model files

**Cohort B.**

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema/src/source/governance/remote_sanction_notice.rs
  - crates/db_schema/src/source/governance/federation_attestation.rs
  - crates/apub/activities/src/governance/inbox.rs   # `..Default::default()` spread (if missing)
requires:
  - task: 2
    reason: "new struct fields typecheck against regenerated schema.rs"
```

**IMPLEMENT:** §10.3 — 4 new struct fields + 4 new InsertForm fields +
NEW `RemoteSanctionNoticeUpdateForm`; mirror for
`federation_attestation.rs` (6 fields each); RE-VERIFY R9 callers at
task start; add `..Default::default()` spread in inbox.rs if missing.

**MIRROR:** §10.3.

**GOTCHA (R9):** RE-VERIFY callers at task start.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task8-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-a-task8-clippy.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/fed-in-a-task8-test-norun.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-a): extend Phase-6 models — UpdateForm + new columns (task 8)`

### Task 9: EXTEND `crates/server/tests/e2e.rs`

**Barrier.**

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
requires:
  - task: 1
    reason: "migration directory must exist on phase-branch tip"
  - task: 2
    reason: "schema.rs must have new table blocks for foundation test imports"
  - task: 5
    reason: "newtypes must exist for model imports"
  - task: 6
    reason: "federation_peer.rs (with federation_inbox_check_peer_trust) must exist"
```

**IMPLEMENT:** §10.8 — 1 slice prepend + 3 probe-block extensions +
NEW `mod v1_federation_inbound_a_fixtures` at file end (Case A).

**MIRROR:** §10.8; v1-SL-a + v1-RT-r1 probe blocks; v1-SL-b fixtures.

**GOTCHA (`feedback_junior_worker_e2e_edit_hang.md`):** 4 anchored
Edits; well under hang threshold.

**GOTCHA (Case A):** outer `LemmyResult<()>`; every helper
`LemmyResult<T>`; every `?` bare; one `map_err` on testcontainer.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task9-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-a-task9-clippy.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/fed-in-a-task9-test-norun.log 2>&1"'
```

(Phase 2 e2e execution: user-gate-4 post-finalize-merge.)

**COMMIT:** `feat(v1-federation-inbound-a): e2e.rs — phase1 round-trip probes + trust-state foundation test module (task 9)`

### Task 10: Retro

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-federation-inbound-a-retro.md
modifies: []
```

Per-task complexity recorded per `feedback_retro_task_complexity_score.md`.

**COMMIT:** `docs(retro): v1-federation-inbound-a session retro`

---

## 14. Testing strategy

- **Unit (compile-time):** `cargo check --workspace --features full`.
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings`.
- **Test target compile (R7):** `cargo test --no-run --workspace --features full --test e2e` on Tasks 6-9.
- **e2e execution (Phase 2):** extended 3-test round-trip +
  `v1_federation_inbound_a_fixtures` 2 fns. user-gate-4.
- **Migration round-trip:** `bash scripts/brehon/migrate-roundtrip.sh` on Task 1.
- **Parity tests:** `seeded_keys_count_matches_const_count` +
  `every_seeded_key_has_metadata` + `every_seeded_key_has_const_fallback`.

---

## 15. Validation commands (DoD)

> **PRECON-3 laptop-shape (NOT Shape G).** Per DQ #229. Each impl-task
> raises `kind: "validate-pending-laptop"` post-push naming §15 DoD
> commands verbatim with `--workspace --features full`.

### 15.1 Per-task workspace check (Tasks 1-9)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task<N>-check.log 2>&1"
```

**EXPECT:** exit 0.

### 15.2 Per-task clippy (Tasks 2-9 — R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-a-task<N>-clippy.log 2>&1"
```

**EXPECT:** exit 0.

### 15.3 Test target compile (R7 — Tasks 6-9)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/fed-in-a-task<N>-test-norun.log 2>&1"
```

**EXPECT:** exit 0.

### 15.4 Migration round-trip (Task 1)

```bash
bash scripts/brehon/migrate-roundtrip.sh > .claude/PRPs/debug/fed-in-a-task1-migrate.log 2>&1
```

**EXPECT:** exit 0.

### 15.5 Phase 2 e2e (post-finalize-merge — user-gate-4)

**(a) Local laptop bg** (~26 min, zero billed):

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-a-<sha>.log 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"
```

**(b) GH dispatch** (~26 min billed):

```bash
gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-federation-inbound-a
```

**EXPECT:** exit 0.

### 15.6 (Shape G section — DORMANT until 2026-06-01)

**NOT applicable.** Per PRECON-3 + DQ #229.

### 15.7 Cross-cutting verification

- [ ] `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns **54**.
- [ ] `rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l` returns **54**.
- [ ] `rg -n '\(pending\)' .claude/rules/governance-log-entry-kind-registry.md` shows 9 `-a` rows.
- [ ] `grep -cE "'federation\\.inbound\\." migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql` returns 11.
- [ ] `crates/db_schema_file/src/schema.rs` has 4 new `table!` blocks.
- [ ] `crates/db_schema/src/source/governance/` has 4 new files.
- [ ] `federation_peer.rs` contains 2 trust helpers.
- [ ] `config.rs` has `EXPECTED_SEED_COUNT_V1_FED_IN: usize = 11;`.
- [ ] R1: no `i32 as i64` casts.
- [ ] R6: every clippy uses `--no-deps -- -D warnings`.
- [ ] R9: Task 8 callsite enumeration completed.
- [ ] DQ #232 honoured: all shared-file edits APPEND-ONLY.
- [ ] All `-a` stories `[done]`.

### 15.8 ADR / OQ compliance

- [ ] **ADR-006** honoured.
- [ ] **ADR-014** honoured.
- [ ] **ADR-015** honoured.
- [ ] **PRD §2 OUT** honoured.
- [ ] **PRD §8.0 preconditions** verified via DO preamble.

---

## 16. Acceptance criteria

- [ ] All 11 tasks committed.
- [ ] §15.1-§15.4 exit 0 after each task.
- [ ] §15.5 Phase 2 e2e all tests pass.
- [ ] §15.7 + §15.8 all boxes ticked.
- [ ] §16a stories all `[done]`.
- [ ] No edits outside §11 list.
- [ ] Retro committed.
- [ ] PR opens against `governance-v0` with `--repo barrie-cork/lemmy`.
- [ ] `/brehon-verify` report shows all stories ✓.

---

## 16a. Stories

### Story 1: Schema migration round-trips cleanly + new tables/enums/columns/indexes/seeds exist

- **Composing tasks:** Tasks 1-5 (Cohort A).
- **Checkpoint (laptop):** `cargo-check.bat --workspace --features full`
  on Task 5's worker branch → exit 0.
- **Checkpoint (migration — Task 1):** `migrate-roundtrip.sh` exit 0.
- **Checkpoint (Phase 2):** extended `test_phase1_migrations_forward`
  + `_revert` + `_reapply` pass.
- **Expected output (Phase 2):** `1 passed; 0 failed` per round-trip
  test.
- **Brief-Scope outputs to verify:**
  - up.sql contains `CREATE TYPE federation_peer_trust_enum`,
    `CREATE TABLE federation_peer`, `CREATE TABLE federation_inbox_dropped_log`,
    `CREATE TABLE federation_inbox_nonce`,
    `CREATE TABLE remote_moderation_label`, both `ALTER TABLE`s,
    `INSERT INTO governance_config` (× 11 rows).
  - down.sql contains `DELETE FROM governance_config`,
    `DROP TABLE` (× 4), `DROP TYPE` (× 2).
  - schema.rs has 4 new `table!` blocks.
  - Extended Phase-6 blocks contain new column lines.
  - newtypes.rs contains 2 new typed-IDs.

### Story 2: 11 config keys + 9 ENTRY_KIND consts land with parity + registry invariants

- **Composing tasks:** Tasks 3, 4.
- **Checkpoint (laptop):** `cargo-test.bat --workspace --features full
  --test e2e --no-run` on Task 4's worker branch → exit 0.
- **Checkpoint (Phase 2):** parity tests pass (138 == 138).
- **Brief-Scope outputs:**
  - config.rs contains `EXPECTED_SEED_COUNT_V1_FED_IN: usize = 11;`.
  - Parity test sum contains `+ EXPECTED_SEED_COUNT_V1_FED_IN`.
  - 11 new tuples in `// v1-federation-inbound-a additions` block.
  - 11 new `DEFAULT_FEDERATION_INBOUND_*` declarations.
  - `ENUM_FEDERATION_PEER_TRUST` declared.
  - governance_log.rs has 9 new `pub const ENTRY_KIND_FEDERATION_*`.
  - Shim re-exports 9 new consts alphabetically.
  - Registry has populated `## v1-federation-inbound-a entry kinds
    (9, this sub-phase)` section + 9 `(pending)` markers.
  - Acceptance invariants count = 54.

### Story 3: Trust-state foundation helper compiles + behaves correctly under DB fixture

- **Composing tasks:** Tasks 6, 7, 8, 9.
- **Checkpoint (laptop):** `cargo-clippy.bat --workspace --features
  full --no-deps -- -D warnings` on Task 8's worker branch → exit 0.
- **Checkpoint (Phase 2):** test filter
  `v1_federation_inbound_a_fixtures` → 2 passes; extended round-trip
  passes new `-a` probes.
- **Expected output (Phase 2):** `2 passed; 0 failed` for fixtures
  module.
- **Brief-Scope outputs:**
  - `federation_peer.rs` exists with full shape + 2 trust helpers.
  - 4 new model files exist.
  - `federation_inbox_nonce.rs` `delete_older_than` has `#[allow(dead_code)]`
    + TODO.
  - `remote_sanction_notice.rs` + `federation_attestation.rs` have NEW
    UpdateForm structs + new fields.
  - `mod.rs` has 4 new `pub mod` lines.
  - `MIGRATIONS_TO_REVERT_PHASE_1` has new entry at top.
  - e2e.rs has `mod v1_federation_inbound_a_fixtures` at file end.
  - Phase-1 round-trip tests have new `-a` probe blocks.

---

## 17. Completion checklist

- [ ] Task 0 audit complete.
- [ ] Tasks 1-9 committed.
- [ ] Task 10 retro committed.
- [ ] §15 validation green.
- [ ] §16a stories all `[done]`.
- [ ] PR opened by BM against `governance-v0`.
- [ ] CodeRabbit triaged.
- [ ] `/brehon-verify` report shows all stories ✓.
- [ ] Post-merge phase branch retained.
- [ ] DQ #233 resolved.
- [ ] DQ #234 recorded in resolved[].
- [ ] DQ #230 honoured.
- [ ] DQ #231 honoured.
- [ ] DQ #232 honoured.

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Migration timestamp collides | LOW | LOW | Task 0 Probe 9 + Task 1 GOTCHA — pick `2026-05-18` if needed. |
| DO preamble fires RAISE EXCEPTION | VERY LOW | HIGH | PRECON-1/2 verified; Probes 2 + 5 re-verify. |
| Cohort A serialization wall-clock | MED | LOW | Expected per advisor-orchestrator.md §5.2. |
| Cohort B mod.rs overlap | LOW | LOW | Append-only; finalize-merge resolves. Degrade to serial if needed. |
| Task 8 R9 misses callsite | LOW | MED | Task 0 Probe 4 + Task 8 RE-VERIFIES. |
| Trust helper crate-graph conflict | VERY LOW | LOW | Phase 6 imports already from `lemmy_db_schema::source::governance`. |
| `delete_older_than` clippy `-D warnings` | LOW | LOW | `#[allow(dead_code)]` canonical. |
| pg_type orphans after down.sql | VERY LOW | LOW | Wholly-new types; `DROP TYPE` cleanly removes. |
| Column-default propagation | LOW | LOW | NOT NULL DEFAULTs work via Postgres. |
| Complexity 13 rejected (split-mandated) | HIGH | LOW | DQ #233; SL-a precedent at 13 shipped proceed. |
| `EXPECTED_SEED_COUNT_V1_FED_IN` drift | LOW | MED | Task 3 pre-commit grep reconciliation. |
| Registry count drift | LOW | LOW | APPEND-ONLY; mechanical sum at PR review. |
| Phase 2 e2e laptop saturation | LOW | LOW | user-gate-4 dispatch option. |
| Junior pre-pushes break finalize-merge | LOW | LOW | Post-Shape-G correct per `feedback_junior_finalize_skips_when_worker_pre_pushes.md`. |
| Cross-lane DQ id collision | LOW | LOW | Max id = 232; planner uses 233 + 234. |
| Helpers pull crates/apub/ into db_schema dep graph | VERY LOW | HIGH | Helpers take `&mut AsyncPgConnection` only. |

---

## 19. Notes

### 19.1 Planner DQs filed

- **DQ #233** (`from: "planner"`, `kind: "blocker"`, `answered_by: null`)
  — split-or-proceed per §5.2. Question: "Complexity score 13 exceeds
  Sonnet threshold 8 — split v1-federation-inbound-a into
  v1-federation-inbound-a-1 (Tasks 1-5) + v1-federation-inbound-a-2
  (Tasks 6-9) + retro, or proceed as one plan?" Options: split /
  proceed. Lean: proceed (SL-a precedent at 13; brief §0 atomic; split
  races shared files).

### 19.2 Self-resolved planner findings

- **DQ #234** (`from: "planner"`, `kind: "log"`, `answered_by:
  "planner-self-resolved"`) — `FederationPeerId` newtype decision.
  Resolved: reuse `InstanceId`. Rationale: `federation_peer.instance_id
  Int4 PRIMARY KEY REFERENCES instance(id)`; `federation_blocklist`
  uses the same shape with `InstanceId`. PRD §8.4 says "newtype only
  if Diesel benefits".
- **`federation_inbox_nonce::delete_older_than` boundary**: `-a`
  ships the fn with `#[allow(dead_code)]` + TODO; `-b` wires cron.

### 19.3 Pre-existing pending DQ entries

- **DQ #229** (advisor 2026-05-16, ADVISORY-LOG) — Shape G re-enable
  2026-06-01.

Advisor's clarify-pass produced DQ #230/#231/#232 (all resolved before
plan-write).

### 19.4 Out-of-scope follow-ups

- `wrap_governance_inbound` — `-b`.
- `receive_remote_moderation_label` body — `-b`.
- Rate-limit + replay enforcement — `-b`.
- Admin REST endpoints — `-c`.
- OQ-FED-IN-1 pseudonym rendering — `-c`.
- `webauthn-rs` step-up — `-c`.
- Outbound per-peer blocklist — v2.
- SSRF-isolated fetch worker — v2.
- Federation discovery — v2.
- Per-community-per-peer trust — v2.

### 19.5 Confidence bands

- **High (9/10):** schema migration — verbatim PRD §8.2 mirror.
- **High (9/10):** Diesel model shapes — direct `sanction.rs` mirror.
- **High (8/10):** trust-state helper placement — DQ #230 binding.
- **High (8/10):** entry-kind registry + 9-const dual-file edit.
- **High (8/10):** parametric `EXPECTED_SEED_COUNT_V1_FED_IN`.
- **Moderate (7/10):** Phase-6 model extension R9 — depends on caller
  shapes; Task 8 RE-VERIFIES.
- **High (9/10):** §5 complexity 13 trips DQ #233 mechanically.

### 19.6 Why no clarify DQ at impl time

Brief §4.2 boundary-of-judgment cases — none apply:

- Migration timestamp `2026-05-17` does not collide at plan-write.
- `federation_peer.notes JSONB` has no `version: N` requirement.
- `EXPECTED_SEED_COUNT_V1_FED_IN = 11` matches PRD §10 exactly.
- DQ #230/#231/#232 already settled module location + DoD shape +
  shared-file discipline.
- Registry stub is empty `(reserved)` block at plan-write.

If baseline changes between plan-write and impl-time, impl-task files
`kind: "blocker"` DQ.

---

## 20. Confidence score

- **Plan correctness:** 8/10 — SL-a + RT-r1 patterns mirrored; PRD
  §8.2 SQL verbatim; R9 caller-enumeration highest-uncertainty.
- **Cargo budget:** 9/10 — laptop-shape; peak at threshold.
- **Test coverage:** 7/10 — round-trip extension + trust foundation
  test (2 fns); wrapper semantics deferred to `-b`/`-c`.
- **Story-grain decomposition:** 9/10 — each task maps to one story;
  concrete checkpoints + grep-verifiable outputs.
- **Plan-shape conformance:** 9/10 — 20-section schema; §15
  laptop-shape; §16a present; per-task FILES YAML with `requires:`;
  mechanical §5.1.

---

_Plan author: planning subagent (Junior `role-planning-v1-federation-inbound-a-plan-…-271`,
2026-05-16). Two planner DQs raised at commit time: DQ #233
(split-or-proceed) + DQ #234 (`FederationPeerId` decision). Confidence
8/10. Plan ships under proceed-as-one assumption pending DQ #233._

LESSON: federation-inbound `-a` foundation sub-phases that mirror the
SL-a structural shape consistently land ~9 impl tasks once the FILES
YAML `requires:` discipline is honoured for Cohort B members depending
on Cohort A's `schema.rs` regeneration. Future planning briefs for
`-b` should pre-estimate 8-12 impl tasks; for `-c`, 7-10 tasks.
Pre-landed-const-exemption pattern robust per v1-SL-a + v1-RT-r1.

LESSON (HARNESS GAP — planner-self-resolved 2026-05-16): the Junior
worker daemon's Claude Code permission layer blocks all writes to
`.claude/**` (including paths explicitly whitelisted in
`settings.json` `permissions.allow` like `.claude/PRPs/reviews/**`
and `.claude/decision-queue.json`). This means the planning subagent
cannot write its plan file to the canonical
`.claude/PRPs/plans/<phase>.plan.md` path; nor can it write planner-
raised DQ entries to `.claude/decision-queue.json`. The plan and DQ
content lands at `PLAN_DELIVERABLE.md` + `ESCALATION_NOTE.md` at the
worktree root; the advisor laptop must transcribe to canonical paths
before queueing impl. Harness fix: investigate whether
`--dangerously-skip-permissions` is wired through the daemon
executor; if so, audit the "sensitive file" path-list to add a
worktree-bound exception for `.claude/PRPs/plans/**` and
`.claude/decision-queue.json` per the existing `settings.json`
allowlist.
