# v1-federation-inbound-c planning brief — reader-side append-history fix (scope (a) only)

**Written**: 2026-05-21 by advisor session (laptop, canonical CWD `C:/Users/barri/Developer/brehon-fork`, branch `governance-v0` @ `253720491`).
**Subagent target**: `planning` (Opus 4.7 — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/v1-federation-inbound-c-planning-1` from `governance-v0` committed HEAD `253720491`. Plan file commits + pushes back to `governance-v0` at finalize. The phase branch `phase-v1-federation-inbound-c` is cut at `bm-cut` AFTER plan approval (User Gate 1).
**Authority anchor**: this brief IS the canonical planning input for `v1-federation-inbound-c`. It is **narrower** than the `.claude/PRPs/handovers/v1-federation-inbound-c-bootstrap.md` (2026-05-20) scope by user direction 2026-05-21 (Q: "scope (a) only"): only the reader-side append-history fix on `get_inbound_config_int` + its mirror in `publish_trust_attestation.rs`. Defers (b) Copilot DoS-hardening family and (c) Phase-6 convention-divergence audit to later sub-phases — see §0.2 explicit de-scope.
**Sub-phase target**: `v1-federation-inbound-c` — reader-side correctness fix on the federation-inbound `governance_config` read path. Two file edits + e2e regression coverage; ships under `validate-pending-laptop` DoD (Shape G SUSPENDED until 2026-06-01 per DQ #229 / `project_shape_g_suspended_2026_05_16`).

---

## 0. Why this sub-phase exists (the design problem — read first)

The `governance_config` table is **append-only by design**: admin edits INSERT a new `(scope, key, valid_from)` row rather than UPDATE the existing row, so the full audit trail is preserved (per `crates/db_schema/src/source/governance/governance_config.rs:34-40` doc-comment + migration `migrations/2026-04-18-000000-0000_add_governance_config/up.sql:1-50`). The migration also defines a `governance_config_current` SQL view that does `DISTINCT-ON (scope, key) ... ORDER BY valid_from DESC` to surface the latest row per key.

**The defect**: two reader call-sites query the **base table** without `.order_by(governance_config::valid_from.desc())` before `.first()`, so Postgres returns rows in arbitrary order. Once `governance_config` accumulates admin-edit history for the same `(scope, key)`, these readers may return a STALE seeded row instead of the most-recent admin write — a load-bearing correctness defect because the values being read are size caps + rate caps that gate inbound federation enforcement.

Concrete evidence the defect is real and the fix is mechanical:

- **`crates/api/api/src/governance/config.rs:740-769` is the canonical reader pattern.** It reads the same `governance_config` base table, but does so via `.order_by(governance_config::valid_from.desc()).first()`. The inline comment at line 741-746 cites the regression history verbatim: *"governance_config is append-only with multiple rows per (scope, key) keyed by valid_from. ORDER BY valid_from DESC + LIMIT 1 (.first) is load-bearing — without it Postgres returns arbitrary order and reads can return stale seeded rows instead of admin_set_config writes. Regression history: commit 8e3bba1 dropped the governance_config_current view; this code path needs to do the latest-wins ordering itself."* The fix in v1-federation-inbound-c is mechanically the same shape applied to two missing call-sites.
- **The two missing call-sites**:
  1. **`crates/apub/activities/src/governance/inbox.rs:421-438`** — `get_inbound_config_int` (private helper used by `wrap_governance_inbound` for size/rate-cap reads). Reads `.first::<Option<i64>>(conn)` without `.order_by(valid_from.desc())`.
  2. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:142-152`** — inline `let actor_cap: i64 = { ... }` block (mirror of the helper above — the comment at line 135-138 says verbatim: *"Read the per-actor rate cap. Mirrors the `get_inbound_config_int` helper in inbox.rs (private there; duplicated here to avoid requiring a pub(crate) expansion of inbox.rs internals — the same circular-dep constraint that caused inbox.rs to define the helper locally)"*). Same defect, same shape.
- **Both sites are read by the wrap_governance_inbound enforcement stack** (fed-in-b: trust → size → schema → rate → replay → handler) — `inbox.rs:421-438` reads `payload_size_cap_<kind>` and `rate_limit_*_per_peer` keys; `publish_trust_attestation.rs:142-152` reads `rate_limit_trust_attestation_per_actor`. A stale read here means the enforcement gate fires (or doesn't) on a value the admin already overrode.

**This defect class is not new code; it is a reader-side correctness bug carried over from fed-in-b**. The fix is mechanical: add `.order_by(governance_config::valid_from.desc())` before each `.first(...)` chain at both call sites. No schema change. No new migration. No handler-contract change. Two file edits + one e2e regression test that asserts an admin override is read by the inbox wrapper.

---

## 0.1 Pre-resolved facts (READ BEFORE §1 — these are BINDING; do NOT re-derive or file blockers for them)

The advisor resolved these in pre-handover exchanges + post-merge handover authoring on 2026-05-21. They pre-empt round-trips the planner would otherwise have to make.

### 0.1.1 — PRECON-1 — Scope: (a) only; (b) + (c) deferred

**User decision 2026-05-21:** scope (a) only — the reader-side append-history fix on `get_inbound_config_int` (inbox.rs:421-438) + its mirror in publish_trust_attestation.rs:142-152. Defers:

- **(b) Copilot DoS-hardening family** (rate-map LRU + key hash + TOCTOU eviction on `inbox.rs:473` + `:698` + `publish_trust_attestation.rs:165`) → re-evaluate at fed-in-c retro time; likely lands as v1-federation-inbound-d.
- **(c) Phase-6 convention-divergence audit over fed-in-b's `inbox.rs` additions** → SUBSUMED by the in-flight `brehon-conformance-audit` lane (planning brief committed at `57ce4c322` on `governance-v0`; phase tip `425ab13c8`; Phase 1 mechanism implementation DONE per MEMORY.md "Active workflow state"). The conformance-audit skill + Clippy gates address the same defect class structurally; no need to in-line the audit into fed-in-c.

**Binding consequence for the plan:** §13 has ONE narrow track — two file edits + one e2e test addition + a brief §15 DoD validation set. The plan must NOT add tasks for DoS hardening or convention audits. If the planner is tempted to broaden scope ("while I'm in inbox.rs let me also fix..."), STOP and file `kind: "log"` recording the temptation; do not in-scope it. Scope creep is the failure mode this PRECON closes.

### 0.1.2 — PRECON-2 — Canonical reader pattern: MIRROR `config.rs:740-769`

**The advisor's binding decision:** the planner mirrors the canonical reader pattern at `crates/api/api/src/governance/config.rs:740-769` (verified at HEAD `253720491`) — the line range that already does this correctly for the same table. Specifically:

- Both fix sites add a `.order_by(governance_config::valid_from.desc())` chain **before** the existing `.first::<Option<i64>>(conn)` call.
- Both fix sites preserve the existing `.filter(scope.eq("instance"))` + `.filter(key.eq(...))` + `.select(value_int)` chain unchanged.
- Both fix sites preserve the existing error-mapping `.map_err(|_e| LemmyErrorType::Unknown(...))?` shape unchanged.
- Both fix sites preserve the existing `val.ok_or_else(...)?` null-handling shape unchanged.
- Neither fix site introduces use of the `governance_config_current` SQL view. The view is not registered as a Diesel table in `crates/db_schema_file/src/schema.rs` (verified by `rg "governance_config_current" crates/db_schema_file/src/schema.rs` → zero matches), so Diesel-typed reads must go to the base table + `.order_by`. The view IS used elsewhere via raw SQL (`crates/api/api/src/governance/admin_config.rs:1003-1073`); that is out-of-scope for v1-federation-inbound-c.

**Binding consequence:** the plan's §10 MIRROR ref column for each fix task MUST cite `crates/api/api/src/governance/config.rs:740-769` literally. The §13 task body MUST quote the relevant chain verbatim ("Add `.order_by(governance_config::valid_from.desc())` between `.select(...)` and `.first::<Option<i64>>(conn)`"). If the planner proposes a different fix shape (e.g. introduce a helper, use the SQL view via raw SQL), STOP and file `kind: "log"` proposing it as a follow-up; do not in-scope it in v1-federation-inbound-c.

### 0.1.3 — PRECON-3 — Helper extraction: NOT in scope

**The advisor's binding decision:** the planner does NOT extract a shared helper across `inbox.rs` and `publish_trust_attestation.rs`. The existing duplication has a doc-comment justification at `publish_trust_attestation.rs:135-138` (circular-dep avoidance: `lemmy_api → lemmy_apub → lemmy_apub_activities`). Extracting the helper would require either (a) `pub(crate)` expansion of `inbox.rs` internals (changes the module boundary, broad impact), or (b) a new shared module under `crates/apub/activities/src/governance/` (new file, new public surface). Both options grow scope beyond "reader-side append-history fix" and re-open the circular-dep question.

**Binding consequence:** the plan ships TWO file edits with the same chain inserted at each site. The duplication remains — the comment at `publish_trust_attestation.rs:135-138` already documents why. If the planner proposes helper-extraction, STOP and file `kind: "log"` proposing it as a follow-up; do not in-scope it.

### 0.1.4 — PRECON-4 — E2e regression coverage: ONE test, append-history-aware

**The advisor's binding decision:** the planner adds ONE new e2e test under `crates/server/tests/e2e.rs` (within the existing `mod v1_federation_inbound_b_fixtures` module — same sibling module fed-in-b established at PR #139). The test exercises the fix in the simplest reproducible shape:

1. **Setup**: insert a baseline `governance_config` row for one of the size-cap or rate-cap keys read by `get_inbound_config_int` (e.g. `payload_size_cap_sanction_notice` — pick whichever key has the simplest existing fixture).
2. **Act 1**: send a federation-inbound activity that triggers `wrap_governance_inbound` reading that key — confirm enforcement uses the baseline value.
3. **Override**: INSERT a SECOND `governance_config` row with the same `(scope, key)` and a more-recent `valid_from`, with a tightened cap value.
4. **Act 2**: send another federation-inbound activity — confirm enforcement uses the OVERRIDE value (not the stale baseline).
5. **Assert**: `governance_log` entries reflect the override-effective behaviour (e.g. a payload that fit under baseline but exceeds override is REJECTED on Act 2 with `federation_inbound_dropped_oversize`).

**Binding consequence:** the §13 task list includes ONE e2e fixture task. **Apply `feedback_lemmy_error_no_std_error.md` Case A** per §2.4 mandatory file-class lesson injection (the sibling module `v1_federation_inbound_b_fixtures` uses `LemmyResult<()>` — mirror that shape verbatim, no `Box<dyn Error>` bridges, no `.map_err` annotation closures). **Apply `feedback_async_pool_test_pattern.md`** for the connection acquisition pattern (`AsyncPgConnection::establish + DbPool::Conn`). Per `feedback_junior_worker_e2e_edit_hang.md`: the e2e file is ~15600+ lines post-fed-in-b; the impl-task brief's Edit MUST target a small line range (≤200 lines) within the existing sibling module — never a full-file Edit. The plan's §13 e2e task includes the exact insertion point (anchor line in the sibling module + ≤200-line insertion budget).

### 0.1.5 — PRECON-5 — Migrations: ZERO

**The advisor's binding decision:** v1-federation-inbound-c is reader-side code-only. Expected migration count: **zero**.

**Binding consequence:** if the generated plan proposes a new migration under `migrations/**` or `crates/db_schema/migrations/**`, catch-fire (scope violation). The schema substrate is fed-in-a's; fed-in-b added no schema; v1-federation-inbound-c does not add schema. The e2e fixture's seed-row INSERT (PRECON-4 step 1 + step 3) is TEST-FIXTURE only — not a production migration — and lands inside the `mod v1_federation_inbound_b_fixtures` test body (as the existing module already does for its own scenarios).

### 0.1.6 — PRECON-6 — `validate-pending-laptop` DoD shape (Shape G SUSPENDED)

Shape G is SUSPENDED until 2026-06-01 (per PMD `project_shape_g_suspended_2026_05_16`; DQ #229 pending re-enable). v1-federation-inbound-c ships under the `validate-pending-laptop` DoD pattern (per `.claude/rules/advisor-orchestrator.md` §5.2). The §15 cargo commands mirror `.claude/PRPs/plans/v1-federation-inbound-b.plan.md` §15 (the most-recent same-lane exemplar).

**Binding consequence:** the §15 set MUST include:

- `bash scripts/brehon/cargo-check.sh --workspace --features full` — workspace compile check.
- `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings` — clippy with `--no-deps` to avoid upstream lint debt + `--features full` to see the governance code under cfg-gate (per `feedback_features_full_workspace_only.md` + `feedback_clippy_test_style.md`).
- `cargo test --test e2e --no-run -p lemmy_server` — sanity: e2e harness still links post-edit.
- **Phase 2 e2e** (full run) per user gate 4 (`feedback_e2e_local_or_dispatch_user_choice.md` — never auto-pick): default-recommended LOCAL via `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-c-<sha>.log 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true` (per `feedback_windows_e2e_requires_bat_wrapper.md` + `feedback_default_local_testing.md`).

**Forward reminder:** if v1-federation-inbound-c crosses 2026-06-01 UTC (likely — storage-hardening sub-phases historically span days), re-check DQ #229 status FIRST THING in the new-day session. Shape G may be re-enabled, and the `kind: "validate-pending"` shape changes (workflow_run_id field reactivates).

### 0.1.7 — PRECON-7 — Cohort dispatch: `[P]` on the two reader-fix tasks (different files)

**The advisor's binding decision:** the two reader-fix tasks (one per file) are `[P]`-able because they touch disjoint files (`inbox.rs` vs `publish_trust_attestation.rs`). The e2e fixture task is non-`[P]` against the reader fixes because it depends on both fixes being merged (its assertion exercises both code paths).

**Binding consequence:** the plan's §13 task list orders as:

1. Task 0 — pre-flight harness audit (non-`[P]`, mandatory per `.claude/rules/pre-phase-harness-audit.md`).
2. Task 1 `[P]` — fix `inbox.rs:421-438` (add `.order_by(valid_from.desc())`).
3. Task 2 `[P]` — fix `publish_trust_attestation.rs:142-152` (add same).
4. Task 3 — e2e regression test in `mod v1_federation_inbound_b_fixtures` (requires Tasks 1+2 merged).
5. Task 4 — retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`.

**Cohort dispatch interim mitigation per `feedback_cohort_dq_id_collision.md` Option 3:** for the `[P]` cohort Tasks 1+2, advisor PRE-AUTHORS 2 reserved `validate-pending-laptop` DQ stubs in `pending[]` BEFORE dispatching the cohort, and each impl brief names its assigned DQ id explicitly in §4 Constraints. This prevents the 5-way collision observed in v1-rls-r1 Cohort A. The plan's §0 PRECONs cite this mitigation explicitly.

### 0.1.8 — PRECON-8 — Review point: user reviews after clarify-gate, before planning-Junior dispatch

**The advisor's binding decision** (matches v1-rls-r1 + fed-in-b precedent): standard four-role Brehon discipline. Stage-shape for this brief:

1. Author this brief (this file). ✓
2. Commit + push to `governance-v0`.
3. Run `/brehon-clarify .claude/PRPs/briefs/v1-federation-inbound-c-planning-1.md`.
4. Resolve every `kind: "clarify"` DQ raised (advisor-self-answer with citations OR user-relay per `feedback_clarify_before_plan.md`).
5. Surface the clarified brief to the user for approval (User Gate 1-pre — brief-level approval).
6. On user approval, queue the planning Junior task with the matching dispatch line in §1, `base_branch=governance-v0`.
7. Planning Junior writes the plan; advisor runs §3.4 DoD smoke + §3.5 watchpoint-specificity + §3.7 dogfood gates; surfaces plan to user for User Gate 1 (plan approval).

---

## 0.2 What this brief is NOT (explicit de-scope)

Per user direction 2026-05-21, the following are NOT in v1-federation-inbound-c scope. They are documented in the bootstrap file (`.claude/PRPs/handovers/v1-federation-inbound-c-bootstrap.md` §"Scoping signals") and deferred to later sub-phases:

- **NOT building: (b) Copilot DoS-hardening family** — `inbox.rs:473` (rate-limit map LRU) + `publish_trust_attestation.rs:165` (subject_url HashMap key hashing) + `inbox.rs:698` (TOCTOU eviction). Defer to v1-federation-inbound-d (or later); the Copilot finding bodies are well-described but the right answer is judgment-heavy (in-process LRU vs Postgres-backed counter vs hash-keyed map) and warrants its own sub-phase + plan-approval gate.
- **NOT building: (c) Phase-6 convention-divergence audit** of fed-in-b's `inbox.rs` additions. SUBSUMED by the in-flight `brehon-conformance-audit` lane — the SKILL + Clippy gates being built there address the same defect class structurally (per `brehon-conformance-audit-planning-1.md` §0 + §0.1.1). Re-evaluate at fed-in-c retro time whether any audit gaps remain after conformance-audit ships.
- **NOT building: helper extraction** across `inbox.rs` + `publish_trust_attestation.rs` (per PRECON-3) — circular-dep constraint stands.
- **NOT building: new SQL view usage** in the Diesel-typed reader path — `governance_config_current` view is not registered in `schema.rs`; using it requires raw SQL (out of scope for the targeted fix shape per PRECON-2).
- **NOT building: any new migration** — see PRECON-5.
- **NOT building: any Shape G workflow change** — Shape G suspended; the `validate-pending-laptop` DoD applies (per PRECON-6).

---

## 1. Role + dispatch line

`[role:planning] v1-federation-inbound-c plan — reader-side append-history fix on get_inbound_config_int + mirror in publish_trust_attestation.rs; scope (a) only per advisor handover 2026-05-21; validate-pending-laptop DoD`

The actual create-task description (single line, <100 chars, per `.claude/rules/advisor-orchestrator.md` §2.1):

```
[role:planning] v1-federation-inbound-c plan — see .claude/PRPs/briefs/v1-federation-inbound-c-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at **`.claude/PRPs/plans/v1-federation-inbound-c.plan.md`** following `.claude/PRPs/templates/plan.template.md`'s 20-section schema literally.

### 2.1 What this sub-phase ships (the deliverable surface)

The plan's §13 task list MUST cover EXACTLY the following five tasks and NOTHING beyond it (cross-check every task against §0.2 "NOT building" — anything not listed below is OUT):

**Task 0 — Pre-flight harness audit** (non-`[P]`, mandatory per `.claude/rules/pre-phase-harness-audit.md`):
- Probes 0-4 (Docker daemon + wrapper script behavior vs intent: per-crate, feature, test target, exit-code propagation).
- DoD smoke test: every §15 command run literally against the phase-branch HEAD; expected-red set documented for tasks not yet shipped.
- Clippy baseline capture against phase-branch HEAD (must be exit 0 before Task 1 dispatch).

**Task 1 `[P]` — Fix `crates/apub/activities/src/governance/inbox.rs:421-438`:**
- IMPLEMENT: Add `.order_by(governance_config::valid_from.desc())` chain in `get_inbound_config_int` between `.select(governance_config::value_int)` (line 426) and `.first::<Option<i64>>(conn)` (line 427).
- MIRROR ref: `crates/api/api/src/governance/config.rs:740-769` (the canonical reader pattern — verbatim citation).
- GOTCHA: do NOT introduce `governance_config_current` view usage (Diesel-typed read; view not in schema.rs).
- VALIDATE: `bash scripts/brehon/cargo-check.sh --workspace --features full` + `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings` both exit 0.
- FILES YAML: `creates: []`, `modifies: [crates/apub/activities/src/governance/inbox.rs]`, `requires: []`.
- Pre-reserved DQ id: `<id-A>` (advisor reserves a `validate-pending-laptop` stub in `pending[]` BEFORE Task 1 dispatch per PRECON-7).

**Task 2 `[P]` — Fix `crates/apub/activities/src/governance/publish_trust_attestation.rs:142-152`:**
- IMPLEMENT: Add `.order_by(governance_config::valid_from.desc())` chain in the inline `let actor_cap: i64 = { ... }` block between `.select(governance_config::value_int)` (line 145) and `.first::<Option<i64>>(conn)` (line 146).
- MIRROR ref: `crates/api/api/src/governance/config.rs:740-769` (same canonical pattern as Task 1) + cross-link `inbox.rs:421-438` post-Task-1 fix.
- GOTCHA: Preserve the existing helper-duplication comment at line 135-138 (justifies the duplication; do not edit). Do NOT extract a shared helper (PRECON-3).
- VALIDATE: `bash scripts/brehon/cargo-check.sh --workspace --features full` + `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings` both exit 0.
- FILES YAML: `creates: []`, `modifies: [crates/apub/activities/src/governance/publish_trust_attestation.rs]`, `requires: []`.
- Pre-reserved DQ id: `<id-B>` (advisor reserves a `validate-pending-laptop` stub in `pending[]` BEFORE Task 2 dispatch per PRECON-7).

**Task 3 — E2e regression test in `mod v1_federation_inbound_b_fixtures`** (non-`[P]`; requires Tasks 1+2):
- IMPLEMENT: Add ONE new `#[tokio::test]` (or whatever the sibling module's convention uses — read the existing module first per canonical-schema-first gate) inside the existing `mod v1_federation_inbound_b_fixtures` in `crates/server/tests/e2e.rs`. Test exercises the override-after-baseline pattern per PRECON-4 steps 1-5.
- MIRROR ref: an existing test inside `mod v1_federation_inbound_b_fixtures` whose shape matches the size-cap or rate-cap enforcement assertion (planner picks; canonical-schema-first gate requires reading the module first to identify the sibling pattern).
- GOTCHA: `LemmyResult<()>` outer + `?` propagation per `feedback_lemmy_error_no_std_error.md` Case A (mirror the sibling). NO `Box<dyn Error>` bridges, NO `.map_err` annotation closures. Use `AsyncPgConnection::establish + DbPool::Conn` per `feedback_async_pool_test_pattern.md`. Multi-write fixture (insert baseline + override) wrapped in `conn.run_transaction(...)` per `feedback_multi_write_handlers_need_transactions.md` if both inserts must commit atomically (planner picks; the assertion is on enforcement behaviour, not on transaction semantics — so non-transactional sequential inserts are also acceptable).
- GOTCHA: Edit budget — ≤200 lines insertion within the sibling module per `feedback_junior_worker_e2e_edit_hang.md`. Never full-file Edit on `e2e.rs` (~15600+ lines).
- VALIDATE: `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-c-<sha>.log 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true` — exit marker `E2E_EXIT_0`.
- FILES YAML: `creates: []`, `modifies: [crates/server/tests/e2e.rs]`, `requires: [1, 2]`.
- Pre-reserved DQ id: not pre-reserved (non-`[P]`; dispatched alone after the cohort cohort).

**Task 4 — Retro** per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`:
- Output: `.claude/PRPs/reports/v1-federation-inbound-c-retro.md` with per-role signals (Advisor / Planning / Impl / BM) + per-task complexity score (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`) + lessons promoted this phase + carry-forward items for the next sub-phase (likely v1-federation-inbound-d if (b) is scoped; or a new lane if storage-hardening is shelved).

### 2.2 Scope boundary — what is explicitly NOT in this sub-phase

Per the 2026-05-21 user decision §0.1.1 + the §0.2 de-scope list above. The plan's §12 "NOT building" MUST enumerate these (each with one-line "why excluded" rationale and the future trigger):

1. **DoS-hardening on `inbox.rs:473` rate-limit map (LRU bounded)** — defer to v1-federation-inbound-d (or later); judgment-heavy mitigation choice warrants its own plan + user gate. Trigger: post-v1-federation-inbound-c retro evaluates whether to in-scope.
2. **DoS-hardening on `publish_trust_attestation.rs:165` raw-string HashMap key (hash mitigation)** — same as #1.
3. **TOCTOU eviction fix on `inbox.rs:698`** — same as #1; concurrency design (single-atomic-SQL vs transactional row-lock) warrants its own plan §4 watchpoint + user-gate-1 review.
4. **Phase-6 convention-divergence audit of fed-in-b inbox.rs additions** — SUBSUMED by `brehon-conformance-audit` lane (planning brief at `57ce4c322` on `governance-v0`). Trigger: v1-federation-inbound-c retro reviews any residual audit gaps after conformance-audit ships.
5. **Helper extraction across `inbox.rs` + `publish_trust_attestation.rs`** — circular-dep constraint stands (per PRECON-3). Trigger: separate refactor sub-phase IF the duplication grows beyond two sites.
6. **Diesel-table registration of `governance_config_current` view in `schema.rs`** — out of scope; Diesel-typed reads use base table + `.order_by`. Trigger: separate schema-cleanup sub-phase IF more than two readers need the view-shaped query.
7. **New migration under `migrations/**` or `crates/db_schema/migrations/**`** — scope violation per PRECON-5. Catch-fire if proposed.
8. **Shape G workflow change** — Shape G SUSPENDED per DQ #229. Trigger: 2026-06-01 re-enable check; if Shape G is back, future sub-phases shift back to `kind: "validate-pending"` (workflow_run_id).

---

## 3. Required reading (in order, before drafting any plan section)

The planning Junior MUST read these files before writing any plan section. Cite each in the plan's §10 Required reading list.

### 3.1 The defect + the fix shape

1. **`crates/apub/activities/src/governance/inbox.rs:421-438`** — the first fix site (`get_inbound_config_int`).
2. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:130-170`** — the second fix site (read line 135-138 doc-comment justifying duplication; line 142-152 is the defect).
3. **`crates/api/api/src/governance/config.rs:730-770`** — the canonical reader pattern (MIRROR ref). Read the inline comment at line 741-746 verbatim — it cites the regression history that gave rise to the discipline.
4. **`crates/db_schema/src/source/governance/governance_config.rs:1-54`** — the model file (read line 16-20 + line 34-40 doc-comments confirming append-only design + `governance_config_current` view).
5. **`migrations/2026-04-18-000000-0000_add_governance_config/up.sql:1-50`** — the migration creating the table + view (read line 48 confirming `CREATE VIEW governance_config_current AS ... DISTINCT-ON (scope, key) ORDER BY valid_from DESC`).

### 3.2 The e2e sibling pattern

6. **`crates/server/tests/e2e.rs`** — find and read the existing `mod v1_federation_inbound_b_fixtures` module. Read ONLY that module (do NOT full-file Read — the file is ~15600+ lines). Use `Grep` for `mod v1_federation_inbound_b_fixtures` to locate the line range, then `Read` with `offset` + `limit`. Cite the exact line range + a representative existing test inside the module as the MIRROR ref for the new e2e test (per canonical-schema-first gate §3.6).

### 3.3 Mandatory file-class lesson injection (§2.4 of advisor-orchestrator.md)

For Task 3 (e2e edit), inject these lessons into §3 Required reading and §4 Constraints:

7. **`.claude/lessons/feedback_lemmy_error_no_std_error.md`** — Case A discipline (`LemmyResult<()>` outer + bare `?`). The fed-in-b sibling module uses Case A; mirror verbatim.
8. **`.claude/lessons/feedback_async_pool_test_pattern.md`** — connection acquisition (`AsyncPgConnection::establish + DbPool::Conn`).
9. **`.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`** — Edit budget ≤200 lines on e2e.rs; never full-file Edit.

Conditional (planner's judgment, IF the e2e test's setup requires atomic baseline+override INSERT):

10. **`.claude/lessons/feedback_multi_write_handlers_need_transactions.md`** — `conn.run_transaction(...)` pattern.

### 3.4 The carry-forward context

11. **`.claude/PRPs/reports/v1-federation-inbound-b-retro.md`** — read once for carry-forward (per-role signals + scope candidates §3 + lessons promoted).
12. **`.claude/PRPs/handovers/v1-federation-inbound-c-bootstrap.md`** + **`.claude/PRPs/handovers/advisor-v1-federation-inbound-c-2026-05-21-scope-a-only.md`** — the 2026-05-20 base context + 2026-05-21 delta. Both are advisor-side context for the planner; cite as authority anchors in plan §0 PRECONs.

### 3.5 Operational rules (verify currency at plan time)

13. **`.claude/rules/advisor-orchestrator.md`** — §1 (polling), §2 (briefs + file-class injection), §3 (stage shape), §4 (cohort dispatch + `requires:` check), §5 (validation + §G4 classifier), §5.2 (validate-pending-laptop handler — load-bearing per PRECON-6).
14. **`.claude/rules/branch-manager.md`** — file ownership (`crates/**` is impl-owned; advisor never authors).
15. **`.claude/rules/decision-queue.md`** — DQ schema + attribution (`from: "impl"` for `validate-pending-laptop`; advisor mutates with `answered_by: "advisor-laptop"`).
16. **`.claude/rules/multi-lane-worktree.md`** — bm-cut creates `brehon-fork-fed-in-c` worktree off `phase-v1-federation-inbound-c` per `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` (PMD canonical path, submodule init, .mcp.json/.env/settings.local.json copies).
17. **`.claude/rules/pmd-invariants.md`** — five non-negotiable PMD invariants (Track A lesson from v1-rls-r1, just shipped).
18. **`.claude/rules/governance-log-entry-kind-registry.md`** — out of scope for v1-federation-inbound-c (no new `ENTRY_KIND_*` consts; fix is reader-side correctness, not new emission paths). The planner confirms this in §10.

### 3.6 The §15 DoD shape (mirror the most-recent same-lane plan)

19. **`.claude/PRPs/plans/v1-federation-inbound-b.plan.md` §15** — the canonical §15 shape for a `validate-pending-laptop`-DoD sub-phase in the federation-inbound lane. Mirror verbatim, adapted for v1-federation-inbound-c's edit set.

---

## 4. Constraints — what the planner MUST enforce

### 4.1 Scope discipline

- The plan §13 task list has EXACTLY 5 tasks (Task 0 + 4 work tasks). No sixth task. No subdivision of Task 1/2 (each is one file edit). No subdivision of Task 3 (one e2e test). If a §13 task body grows past ~200 lines of plan text, the planner files `kind: "log"` proposing a clarify-DQ rather than expanding scope.
- The plan §12 "NOT building" list enumerates the 8 items from §2.2 with one-line rationales each.
- If the planner discovers a third reader site for `governance_config` with the same defect during §3 Required reading (`rg "governance_config::value_(int|float|bool|text)" crates/`) AND it sits OUTSIDE the federation-inbound paths (`crates/apub/activities/src/governance/{inbox,publish_*}.rs`), file `kind: "log"` recording the third site as a follow-up — do NOT in-scope it. Federation-inbound is the v1-federation-inbound-c lane; reader fixes in other lanes belong to those lanes.

### 4.2 Mirror discipline

- §10 MIRROR ref for Tasks 1+2 cites `crates/api/api/src/governance/config.rs:740-769` literally (line range, not just file). The §13 task body quotes the relevant chain shape verbatim.
- §10 MIRROR ref for Task 3 cites the exact line range of an existing sibling test inside `mod v1_federation_inbound_b_fixtures` (canonical-schema-first gate §3.6). If no clear sibling exists, file `kind: "clarify"` BEFORE drafting Task 3.

### 4.3 DQ pre-reservation (per PRECON-7 / `feedback_cohort_dq_id_collision.md` Option 3)

- The plan §13 Task 1 + Task 2 entries name their assigned pre-reserved `validate-pending-laptop` DQ ids in §4 Constraints (placeholder syntax `<id-A>` and `<id-B>` until advisor reserves them at dispatch time — the planner does NOT pick the ids; advisor reserves them in the dispatch sequence after plan approval).
- The plan §0 PRECONs section explicitly cites `feedback_cohort_dq_id_collision.md` Option 3 as the cohort-collision mitigation; the impl-task briefs (authored post-plan-approval) carry forward the assigned ids.

### 4.4 §G4 classifier awareness

- If a Task 1 or Task 2 `validate-pending-laptop` fails on `cargo check` or `cargo clippy`, the planner anticipates the failure class:
  - **Most likely fail mode**: clippy `let_underscore_must_use` or `clippy::needless_borrows_for_generic_args` cascading from the new `.order_by` chain. Allowlist §G4 row exists for both (mechanical rename / mechanical fix recipe). The plan §15 row notes this is the anticipated allowlist class.
  - **Cycle-count meta-rule**: if Task 1 or Task 2 hits the SAME `(error_class, file_basename)` 3× in succession, HARD REFUSAL catch-fire regardless of allowlist (per `feedback_plan_stub_uniformity_with_canonical_sibling.md`). The plan §4 watchpoint #N notes this rule.

### 4.5 File-class lesson injection on Task 3 brief (post-plan-approval, advisor-side)

- The impl-task brief for Task 3 (authored post-plan-approval, NOT in this planning brief) MUST inject lessons 7-10 from §3.3 above per `.claude/rules/advisor-orchestrator.md` §2.4 (mechanical, no judgment call). The plan documents this constraint in §13 Task 3's "Brief constraints" sub-section so the advisor cannot accidentally skip it.

### 4.6 Worktree + PMD discipline

- The plan §0 PRECONs section cites: "v1-federation-inbound-c work happens on a lane-dedicated worktree `brehon-fork-fed-in-c` cut at `bm-cut`. Bootstrap checklist per `feedback_phase_lane_worktree_bootstrap_checklist.md` (canonical PMD path, submodule init, .mcp.json copy, .env copy, settings.local.json copy with SessionStart `pmd-canonical-guard.sh` entry per `.claude/rules/pmd-invariants.md` invariant #5)."
- The plan §0 PRECONs section cites: "PMD writes route to canonical `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` per `.claude/rules/pmd-invariants.md` invariant #1; SessionStart guard `pmd-canonical-guard.sh` (shipped v1-rls-r1 Task 3) catches lane drift."

### 4.7 Brehon governance rules (load-bearing across all subagent invocations)

- **No destructive defaults** (`.claude/rules/no-destructive-defaults.md`) — no `--force`, no `--no-verify`, no `git reset --hard`. Worktree teardown at phase ship uses `git worktree remove --force` ONLY when submodule-pinned (per `feedback_worktree_remove_force_for_submodules.md`).
- **DQ attribution** (`.claude/rules/decision-queue.md`) — `validate-pending-laptop` entries written by impl-task with `from: "impl"`; advisor mutates with `answered_by: "advisor-laptop"`. Commit subjects MUST match `^(chore|docs)\((advisor|decision-queue|bm)\)` for advisor/BM-side DQ writes.
- **Cargo output capture** (`.claude/rules/cargo-output-capture.md`) — never pipe cargo through `tail`/`head`/`grep`; redirect to `.claude/runlog/<phase>-<cmd>-<sha>.log` and exit-code-check separately.
- **No cargo output paste** (`.claude/rules/no-cargo-output-paste.md`) — read `tail -20` of captured logs; never paste full log into plan body or commit messages.
- **Pre-phase harness audit** (`.claude/rules/pre-phase-harness-audit.md`) — Task 0 mandatory (Probes 0-4 per the rule).

---

## 5. Dogfood — pre-commit walkthrough (per `feedback_dogfood_slash_command_specs.md`)

This planning brief was dogfooded by the advisor against the LIVE codebase state at HEAD `253720491`:

1. **Defect verification**: read `crates/apub/activities/src/governance/inbox.rs:421-438` directly — confirmed `.first::<Option<i64>>(conn)` chain has NO `.order_by(...)` between `.select(...)` and `.first(...)`. Confirmed `crates/apub/activities/src/governance/publish_trust_attestation.rs:142-152` has the same defect shape with the duplication-justifying comment at line 135-138.
2. **Canonical pattern verification**: read `crates/api/api/src/governance/config.rs:740-769` directly — confirmed the `.order_by(governance_config::valid_from.desc())` chain exists at line 757, between `.select(...)` (lines 750-756) and `.first::<ConfigRow>(conn)` (line 758). The inline comment at lines 741-746 cites the regression history with commit `8e3bba1` reference.
3. **Helper existence check**: `rg "governance_config_current|latest_value_for|valid_from\.desc" crates/db_schema/src/ crates/apub/activities/src/governance/` returned hits in the `governance_config.rs` model doc-comments only (lines 17, 36) — confirmed NO "latest row per key" helper module exists outside the model file; the fix-shape decision is "use `.order_by` chain in place" rather than "use a helper".
4. **View-vs-table check**: `rg "governance_config_current" crates/` returned 2 hits in `admin_config.rs:1003-1073` (raw SQL probe queries), 1 hit in `admin_rule_sets.rs:16` (doc-comment), 2 hits in `governance_config.rs:17,36` (doc-comments), 2 hits in `migrations/2026-04-18-000000-0000_add_governance_config/up.sql:9,47` (migration creating the view), 1 hit in `e2e.rs:3659` (test-fixture comment). Confirmed the view is NOT registered as a Diesel table (verified by `rg "governance_config_current" crates/db_schema_file/src/schema.rs` → zero matches), so Diesel-typed reads via the view are NOT viable in v1-federation-inbound-c (PRECON-2 confirmed).
5. **Sibling module verification**: `rg -n "mod v1_federation_inbound_b_fixtures" crates/server/tests/e2e.rs` → confirmed the module exists (from fed-in-b PR #139). The planning Junior will read it to identify the e2e test sibling pattern at canonical-schema-first time.
6. **PMD pre-search** (per §2.3 of advisor-orchestrator.md): `memory_search_hybrid("governance_config valid_from latest reader append history")` — no missing-context lessons; the canonical lessons fired by §2.4 file-class injection cover the e2e edit class.

The dogfood confirms the brief is grounded in current HEAD state, not stale 2026-05-20 line numbers. No drift.

---

## 6. After the planner ships

1. **Advisor runs §3.4 DoD smoke test**: every §15 command literally against current HEAD; surface to user as part of plan approval.
2. **Advisor runs §3.5 watchpoint specificity gate**: every watchpoint in plan §4 must cite a specific file:line.
3. **Advisor runs §3.7 dogfood gate**: confirm Task 3's e2e test brief cites the exact sibling module line range in the canonical-schema-first sub-section.
4. **User Gate 1 (plan approval)** via `AskUserQuestion` — surface DoD smoke results + watchpoint-specificity findings + dogfood report.
5. **On user approval**: queue `[role:bm-task] bm-cut — phase-v1-federation-inbound-c — see .claude/PRPs/briefs/v1-federation-inbound-c-bm-cut-1.md`. The bm-cut brief authors itself post-plan-approval (advisor authors brief on `governance-v0`, commits, queues Junior).
6. **Post-bm-cut**: lane worktree `brehon-fork-fed-in-c` cut off `phase-v1-federation-inbound-c` (per `feedback_phase_lane_worktree_bootstrap_checklist.md`). Subsequent advisor session is lane-dedicated; canonical session pauses fed-in-c work.

---

## 7. Definition-of-done for THIS planning brief

This planning brief is DONE when:

- [x] §0 establishes the design problem with file:line citations.
- [x] §0.1 captures all 8 PRECONs (scope, canonical pattern, no helper, e2e shape, no migration, DoD shape, cohort dispatch, review point).
- [x] §0.2 explicitly de-scopes (b) + (c) + helper extraction + new SQL view + new migration + Shape G change with per-item rationale.
- [x] §1 dispatch line is <100 chars and matches the planning subagent contract.
- [x] §2.1 enumerates exactly 5 tasks (Task 0 + Tasks 1-4) with IMPLEMENT/MIRROR/GOTCHA/VALIDATE/FILES YAML shape.
- [x] §2.2 mirrors §0.2 in the plan-template "NOT building" enumeration shape.
- [x] §3 Required reading is 19 items, ordered by purpose (defect → fix shape → e2e sibling → file-class lessons → carry-forward → rules → §15 shape).
- [x] §4 Constraints cover scope, mirror, DQ pre-reservation, §G4 awareness, file-class injection forward-application, worktree/PMD, governance rules.
- [x] §5 dogfood is grounded in live HEAD state (6 verification steps).
- [x] §6 names the post-planner stage shape with user gate.
- [x] No emojis, no inline log dumps, no speculation, no unwritten cross-references.
- [x] Length ~430 lines (within the 150-500 target for a narrow-scope planning brief).

---

## 8. Commit + push

```bash
cd C:/Users/barri/Developer/brehon-fork
git add .claude/PRPs/briefs/v1-federation-inbound-c-planning-1.md
git commit -m "$(cat <<'EOF'
chore(advisor): author v1-federation-inbound-c planning brief (scope (a) only)

Per advisor handover 2026-05-21 + user direction "scope (a) only".

Scope: reader-side append-history fix on get_inbound_config_int
(inbox.rs:421-438) + mirror in publish_trust_attestation.rs:142-152.
Mechanical: add .order_by(governance_config::valid_from.desc()) chain
mirroring canonical reader at config.rs:740-769 (whose inline comment
at 741-746 cites the regression history).

5 §13 tasks: Task 0 pre-flight, Tasks 1+2 [P] reader fixes (disjoint
files), Task 3 non-[P] e2e regression in mod v1_federation_inbound_b_fixtures,
Task 4 retro.

Explicit de-scope: (b) Copilot DoS-hardening + (c) Phase-6 convention
audit (subsumed by brehon-conformance-audit lane). PRECONs 1-8 cover
scope, mirror, no-helper, e2e shape, no-migration, validate-pending-laptop
DoD, cohort DQ pre-reservation, review point.

Next: /brehon-clarify .claude/PRPs/briefs/v1-federation-inbound-c-planning-1.md
EOF
)"
git push origin governance-v0
```

Then run `/brehon-clarify .claude/PRPs/briefs/v1-federation-inbound-c-planning-1.md`.
