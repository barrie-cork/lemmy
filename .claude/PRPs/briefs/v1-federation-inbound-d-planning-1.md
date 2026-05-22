# v1-federation-inbound-d planning brief — DoS-hardening option-b (inbox-size cap + per-actor attestation cap)

**Written**: 2026-05-22 by advisor session (laptop, canonical CWD `C:/Users/barri/Developer/brehon-fork`, branch `governance-v0` @ `6a9f004a9`).
**Subagent target**: `planning` (Opus 4.7 — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/v1-federation-inbound-d-planning-1` from `governance-v0` committed HEAD. Plan file commits + pushes back to `governance-v0` at finalize. The phase branch `phase-v1-federation-inbound-d` is cut at `bm-cut` AFTER plan approval (User Gate 1).
**Authority anchor**: this brief IS the canonical planning input for `v1-federation-inbound-d`. Scope is **option-b** (narrow 2-task: inbox-size cap + per-actor attestation cap, e2e tests for each) per user gate-1 decision 2026-05-22. Option-a (full (b) family including TOCTOU eviction fix at `inbox.rs:698`) is deferred — see §0.2 explicit de-scope.
**Sub-phase target**: `v1-federation-inbound-d` — DoS-hardening of the in-memory rate-limit maps in the federation-inbound governance stack. Two code changes + two e2e tests; ships under `validate-pending-laptop` DoD (Shape G SUSPENDED until 2026-06-01 per DQ #229 / `project_shape_g_suspended_2026_05_16.md`).

---

## 0. Why this sub-phase exists (the design problem — read first)

### 0.1 The two defects in scope (option-b)

fed-in-b (`PR #139`, `413ef5899`) wired the enforcement stack in `wrap_governance_inbound` and introduced two in-memory data structures that have unbounded growth under hostile peer load:

**Defect 1 — `inbox.rs:473` — unbounded per-peer rate-limit map (`rate_map`).**
`wrap_governance_inbound` maintains a `DashMap<String, (u32, Instant)>` keyed on peer domain URL (raw string). Under a DoS scenario with `N` distinct peer domains, the map grows to `N` entries without eviction. At 24 bytes per entry + heap overhead, 100k distinct-peer flood → ~10 MB; 1M peers → ~100 MB. There is no bound check, no LRU eviction, no counter reset path beyond expiry jitter. The fix for option-b is to **bound the map at a configurable `MAX_RATE_MAP_ENTRIES`** (e.g. 10_000), evicting the oldest-inserted entry when the bound is reached. This is a code-only change — no new table, no migration.

**Defect 2 — `publish_trust_attestation.rs:165` — unbounded per-actor attestation-count map.**
`publish_trust_attestation` maintains a `HashMap<String, u32>` (or equivalent) keyed on `subject_url` (raw `String`, not hashed). At sustained hostile load with `M` distinct `subject_url` values, the map grows to `M` entries. The fix for option-b is to **bound the map at a configurable `MAX_ATTESTATION_MAP_ENTRIES`** and to **key on a truncated SHA-256 hash of `subject_url`** (fixed 32-byte key) rather than the raw string (variable heap allocation proportional to URL length). Combined: per-entry memory goes from `|url| + 4` bytes to `32 + 4 = 36` bytes, and growth is bounded. This is a code-only change — no new table, no migration.

**What is deferred (not option-b):**
- **`inbox.rs:698` TOCTOU eviction race** — `evict_oldest_unreviewed_if_needed` can over-evict under multi-receive contention and record `storage_cap_evicted` when DELETE affects 0 rows. Fix requires either single-atomic SQL with `RETURNING` (Diesel `.returning(...)`) or transactional row-lock (`SELECT FOR UPDATE SKIP LOCKED` + DELETE). This is a DB-level concurrency design decision and warrants its own plan §4 watchpoint + separate user-gate-1 review. Deferred to v1-federation-inbound-e (or equivalent).

### 0.2 Why option-b is the right cut

The rate-map bound (Defect 1) and the attestation-map key-hash + bound (Defect 2) are **in-process, code-only** changes. They do not require DB schema changes, migrations, or cross-handler coordination. They can be designed, implemented, and tested independently in 2 tasks. The TOCTOU eviction fix (Defect 3, deferred) requires DB-level concurrency reasoning and introduces a meaningful risk surface that should be plan-gated separately.

The retro-favoured cut is option-b: ship the two in-memory bounds in fed-in-d; defer the DB-level concurrency fix to fed-in-e.

---

## 0.1 Pre-resolved facts (READ BEFORE §1 — BINDING; do NOT re-derive or file blockers for them)

### 0.1.1 — PRECON-1 — Scope: option-b only; TOCTOU deferred

**User decision 2026-05-22:** option-b — inbox-size cap (`inbox.rs:473` rate-map bound + MAX_RATE_MAP_ENTRIES) + per-actor attestation cap (`publish_trust_attestation.rs:165` key-hash + map bound + MAX_ATTESTATION_MAP_ENTRIES). Defers:

- **`inbox.rs:698` TOCTOU eviction fix** → v1-federation-inbound-e (or equivalent); DB-level concurrency design warrants separate plan.
- **`inbox.rs:473` LRU-vs-Postgres-backed-counter choice** → planner decides at plan-author time between option (a) insertion-order evict (simple array-backed evict of oldest) or option (b) LRU evict (ordered by last-access — requires tracking last-access timestamp per entry). The plan §4 watchpoint MUST name the choice explicitly; concept-only watchpoints fail the watchpoint-specificity gate (§3.5 + `feedback_advisor_watchpoint_specificity.md`).

**Binding consequence for the plan:** §13 has exactly 2 code tasks + 2 e2e tasks + Task 0 pre-flight + Task 5 retro. No TOCTOU fix task. No new migration task. If the planner proposes a new migration under `migrations/**`, catch-fire.

### 0.1.2 — PRECON-2 — Mitigation shape for Defect 1 (inbox rate-map)

**The advisor's binding decision:** the planner chooses between two insertion-order eviction approaches for the `rate_map` (`inbox.rs:473`):

- **Option A** — `DashMap` with a companion `VecDeque<String>` (insertion-ordered key queue). On insert: if `map.len() >= MAX_RATE_MAP_ENTRIES`, pop the front of the queue (oldest key) and remove from `DashMap`. Cheap per-insert; O(1) eviction; no external dep.
- **Option B** — replace `DashMap` with an `Arc<Mutex<LinkedHashMap<String, (u32, Instant)>>>` from the `linked-hash-map` crate. LinkedHashMap maintains insertion order; eviction is `pop_front()`. Single data structure; simpler invariant; adds a Cargo dep.

**The plan's §4 watchpoint for Defect 1 MUST name which option the planner chose and why.** Concept-only ("use a bounded data structure") is a watchpoint-specificity failure.

**Constant name:** `MAX_RATE_MAP_ENTRIES` (const, not env-config for v0; the `governance_config` table approach is option-a full-scope work). Value: 10_000. Planner may propose a different value with rationale, but must not leave it implicit.

**Thread-safety note:** the existing `DashMap` is `Arc<DashMap<...>>` (tokio-safe concurrent map). Any replacement must preserve the same thread-safety contract — `Arc<Mutex<...>>` is acceptable for the option-b bound since the map is accessed only on the inbound-receive hot path (not on every DB read).

### 0.1.3 — PRECON-3 — Mitigation shape for Defect 2 (attestation map key-hash)

**The advisor's binding decision:** the fix for `publish_trust_attestation.rs:165` has two parts:

1. **Key-hash**: replace the raw `subject_url: String` map key with `key: [u8; 32]` computed as `sha2::Sha256::digest(subject_url.as_bytes()).into()`. The `sha2` crate is already a workspace dep (used by the governance hash chain per `.claude/rules/advisor-orchestrator.md` — "sha2 hash chain via Postgres triggers"). No new dep needed.
2. **Map bound**: apply the same insertion-order eviction pattern as Defect 1 (same option A or B, consistent with the Defect 1 choice). The constant is `MAX_ATTESTATION_MAP_ENTRIES`, value 10_000.

**Binding consequence:** the plan's §13 Task 2 body MUST cite both the key-hash change AND the bound+eviction change. They ship in the same commit; they are not separable (a key-hash without bound still leaks; a bound without key-hash still allows variable heap per entry).

**`sha2` import check:** before authoring the plan, the planner runs `rg "sha2" Cargo.toml crates/*/Cargo.toml` to confirm the crate is already a dep. If not found, the plan's Task 2 includes a `Cargo.toml` addition (workspace dep declaration). Do NOT assume it exists.

### 0.1.4 — PRECON-4 — E2e coverage: two tests, one per defect

**The advisor's binding decision:** the plan adds TWO e2e tests under `crates/server/tests/e2e.rs` (within the existing `mod v1_federation_inbound_b_fixtures` sibling module — same module fed-in-b established at `PR #139`; fed-in-c's Task 3 extended it). The tests exercise the BOUNDS (map does not grow unboundedly) and the hash-key correctness (same `subject_url` always maps to the same bucket).

**Test 1 (Defect 1 — rate-map bound):**
1. Setup: pre-fill the rate-map with `MAX_RATE_MAP_ENTRIES` distinct peer domain entries via helper or direct map access in the test fixture.
2. Assert: after inserting one more entry, `map.len() == MAX_RATE_MAP_ENTRIES` (not `MAX + 1`).
3. Assert: the evicted entry is the first one inserted (insertion-order eviction semantics — verifies FIFO, not arbitrary eviction).

**Test 2 (Defect 2 — attestation-map key-hash + bound):**
1. Setup: call `publish_trust_attestation` twice with the same `subject_url` — different timestamps.
2. Assert: both calls map to the same rate-bucket (same SHA-256 key → same counter incremented, not two separate entries).
3. Assert: the map's length is bounded after `MAX_ATTESTATION_MAP_ENTRIES` distinct `subject_url` inputs.

**Binding consequence:** §13 has Task 3 (e2e for Defect 1 rate-map) and Task 4 (e2e for Defect 2 attestation-map). Both are non-`[P]` (depend on Tasks 1+2 respectively). Task 3 depends only on Task 1; Task 4 depends only on Task 2.

**Apply `feedback_lemmy_error_no_std_error.md` Case A** for both e2e tasks — mirror the `v1_federation_inbound_b_fixtures` sibling module's `LemmyResult<()>` outer + bare `?` shape verbatim, no `Box<dyn Error>` bridges.
**Apply `feedback_async_pool_test_pattern.md`** for connection acquisition.
**Apply `feedback_junior_worker_e2e_edit_hang.md`** — Edit budget ≤200 lines per task; never full-file Edit on `e2e.rs`.

### 0.1.5 — PRECON-5 — Migrations: ZERO

**The advisor's binding decision:** v1-federation-inbound-d is code-only. Both fixes are in-memory data structure changes. Expected migration count: **zero**.

**Binding consequence:** if the generated plan proposes a new migration under `migrations/**`, catch-fire (scope violation). The `MAX_RATE_MAP_ENTRIES` and `MAX_ATTESTATION_MAP_ENTRIES` constants are Rust `const` definitions, not config-table rows. If the planner proposes storing these in `governance_config`, file `kind: "log"` noting it as a future option-a follow-up; do NOT in-scope it.

### 0.1.6 — PRECON-6 — `validate-pending-laptop` DoD shape (Shape G SUSPENDED)

Shape G is SUSPENDED until 2026-06-01 (per PMD `project_shape_g_suspended_2026_05_16.md`; DQ #229 pending re-enable). v1-federation-inbound-d ships under the `validate-pending-laptop` DoD pattern (per `.claude/rules/advisor-orchestrator.md` §5.2). The §15 cargo commands mirror `.claude/PRPs/plans/v1-federation-inbound-c.plan.md` §15 (the most-recent same-lane exemplar).

**Binding consequence:** §15 MUST include:
- `bash scripts/brehon/cargo-check.sh --workspace --features full`
- `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings`
- `cargo test --test e2e --no-run -p lemmy_server` — sanity: e2e harness still links post-edit.
- **Phase 2 e2e** (full run) per user gate 4: default-recommended LOCAL via `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-d-<sha>.log 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true` (per `feedback_windows_e2e_requires_bat_wrapper.md` + `feedback_default_local_testing.md`).

**Forward reminder:** if v1-federation-inbound-d crosses 2026-06-01 UTC, re-check DQ #229 status at new-day session start. Shape G may be re-enabled.

### 0.1.7 — PRECON-7 — Cohort dispatch: Tasks 1+2 are `[P]` (disjoint files)

**The advisor's binding decision:** Tasks 1 and 2 touch disjoint files (`inbox.rs` vs `publish_trust_attestation.rs`) and can run in parallel. Tasks 3 and 4 (e2e) are non-`[P]`; Task 3 requires Task 1; Task 4 requires Task 2. Task 0 (pre-flight) is always non-`[P]`.

**Binding consequence:** §13 task order:
1. Task 0 — pre-flight harness audit (non-`[P]`).
2. Task 1 `[P]` — inbox rate-map bound (`inbox.rs:473`).
3. Task 2 `[P]` — attestation map key-hash + bound (`publish_trust_attestation.rs:165`).
4. Task 3 — e2e for Task 1 (rate-map bound, non-`[P]`; `requires: [1]`).
5. Task 4 — e2e for Task 2 (attestation-map, non-`[P]`; `requires: [2]`).
6. Task 5 — retro.

**DQ id pre-reservation:** advisor pre-authors 2 `validate-pending-laptop` DQ stubs in `pending[]` BEFORE dispatching the Tasks 1+2 cohort. Each impl brief names its assigned DQ id in §4 Constraints. Plan §13 Tasks 1+2 use placeholder syntax `<id-A>` and `<id-B>` (advisor fills at dispatch time).

### 0.1.8 — PRECON-8 — Review point: user gate after clarify, before planning-Junior dispatch

Standard four-role Brehon discipline (matches fed-in-c precedent):

1. Author this brief. ✓
2. Commit + push to `governance-v0`.
3. Run `/brehon-clarify .claude/PRPs/briefs/v1-federation-inbound-d-planning-1.md`.
4. Resolve every `kind: "clarify"` DQ raised (advisor-self-answer with citations OR user-relay).
5. Surface clarified brief to user for approval (brief-level).
6. On user approval, queue the planning Junior task.
7. Planning Junior writes the plan; advisor runs §3.4 DoD smoke + §3.5 watchpoint-specificity + §3.7 dogfood gates; surfaces plan for User Gate 1 (plan approval).

---

## 0.2 What this brief is NOT (explicit de-scope)

Per user decision 2026-05-22 (option-b):

- **NOT building: `inbox.rs:698` TOCTOU eviction fix** — concurrency design (atomic-SQL vs transactional row-lock) warrants separate plan + user gate. Defer to v1-federation-inbound-e.
- **NOT building: option-a full (b) family** — LRU with access-timestamp tracking, Postgres-backed rate counters, `governance_config`-driven `MAX_*` constants. Defer to option-a scope if needed post-retro.
- **NOT building: any new migration** — PRECON-5.
- **NOT building: Shape G workflow change** — Shape G SUSPENDED per DQ #229.
- **NOT building: `governance_config`-table-backed MAX_* constants** — v0 scope uses Rust `const`; DB-config approach is option-a scope.
- **NOT building: Postgres-backed rate counters** — in-process map with eviction is the option-b approach; Postgres-backed counter adds a DB write per inbound and is option-a scope.
- **NOT building: helper extraction or shared module** between `inbox.rs` and `publish_trust_attestation.rs` — circular-dep constraint from fed-in-c PRECON-3 still applies.

---

## 1. Role + dispatch line

`[role:planning] v1-federation-inbound-d plan — DoS-hardening option-b (inbox rate-map bound + attestation-map key-hash + bound); validate-pending-laptop DoD`

The actual create-task description (single line, <100 chars):

```
[role:planning] v1-federation-inbound-d plan — see .claude/PRPs/briefs/v1-federation-inbound-d-planning-1.md
```

---

## 2. Scope — what to produce

Produce **one plan file** at **`.claude/PRPs/plans/v1-federation-inbound-d.plan.md`** following `.claude/PRPs/templates/plan.template.md`'s 20-section schema literally.

### 2.1 What this sub-phase ships (the deliverable surface)

The plan's §13 task list MUST cover EXACTLY the following six tasks and NOTHING beyond:

**Task 0 — Pre-flight harness audit** (non-`[P]`, mandatory):
- Probes 0-4 (Docker daemon + wrapper script behaviour: per-crate, feature, test target, exit-code propagation).
- DoD smoke: every §15 command run literally against phase-branch HEAD; expected-red set documented for tasks not yet shipped.
- Clippy baseline capture against phase-branch HEAD (must be exit 0 before Task 1 dispatch).

**Task 1 `[P]` — Bound the rate-map at `inbox.rs:473`:**
- IMPLEMENT: Add `MAX_RATE_MAP_ENTRIES: usize = 10_000` const. Add insertion-order eviction (PRECON-2 choice — planner picks option A or B, documents in §4 watchpoint). On every insert to `rate_map`: if `map.len() >= MAX_RATE_MAP_ENTRIES`, evict the oldest-keyed entry before inserting the new one.
- MIRROR ref: the existing `rate_map` insert site at `inbox.rs:473` (planner reads the exact line range at canonical-schema-first time) + eviction pattern from PRECON-2 (planner cites the chosen option's data structure in §10 MIRROR ref).
- GOTCHA: preserve the existing `Arc<DashMap<...>>` thread-safety contract. If switching to `Arc<Mutex<LinkedHashMap<...>>>` (option B), ensure the `Mutex` guard is not held across `.await` points.
- GOTCHA: the `MAX_RATE_MAP_ENTRIES` const is Rust-side only; do NOT read from `governance_config` (out of scope per PRECON-1 + PRECON-5).
- VALIDATE: `bash scripts/brehon/cargo-check.sh --workspace --features full` + `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings` both exit 0.
- FILES YAML: `creates: []`, `modifies: [crates/apub/activities/src/governance/inbox.rs]`, `requires: []`.
- Pre-reserved DQ id: `<id-A>`.

**Task 2 `[P]` — Key-hash + bound the attestation map at `publish_trust_attestation.rs:165`:**
- IMPLEMENT: (a) Replace raw `subject_url: String` key with `[u8; 32]` (SHA-256 of `subject_url.as_bytes()` via `sha2::Sha256::digest(...).into()`). (b) Apply the same insertion-order eviction pattern chosen in Task 1 with `MAX_ATTESTATION_MAP_ENTRIES: usize = 10_000`.
- MIRROR ref: the `sha2` usage in the governance hash chain (planner locates existing `sha2::Sha256` callsite in `crates/` at canonical-schema-first time and cites in §10). Also: the Task 1 `inbox.rs` eviction pattern (cross-link post-Task-1 fix shape).
- GOTCHA: `sha2` must already be a workspace dep — planner confirms via `rg "sha2" Cargo.toml crates/*/Cargo.toml` in §3 Required reading. If not found, brief adds a `Cargo.toml` workspace dep line (one-line addition; no version churn).
- GOTCHA: the map key type changes from `String` to `[u8; 32]` — the planner enumerates ALL callsites that construct or look up the map key via `rg "subject_url" crates/api/api/src/governance/publish_trust_attestation.rs` before writing the task body. If >5 callsites, catch-fire to advisor.
- VALIDATE: same as Task 1.
- FILES YAML: `creates: []`, `modifies: [crates/api/api/src/governance/publish_trust_attestation.rs]`, `requires: []`.
- Pre-reserved DQ id: `<id-B>`.

**Task 3 — E2e for Task 1 (rate-map bound, non-`[P]`; `requires: [1]`):**
- IMPLEMENT: Add ONE `#[tokio::test(flavor = "multi_thread")]` inside existing `mod v1_federation_inbound_b_fixtures` in `crates/server/tests/e2e.rs`. Test shape per PRECON-4 Test 1.
- MIRROR ref: an existing sibling test inside `mod v1_federation_inbound_b_fixtures` (planner picks at canonical-schema-first read; cite exact line range in §10).
- GOTCHA: `LemmyResult<()>` outer + bare `?` (Case A per `feedback_lemmy_error_no_std_error.md`). Edit budget ≤200 lines.
- FILES YAML: `creates: []`, `modifies: [crates/server/tests/e2e.rs]`, `requires: [1]`.

**Task 4 — E2e for Task 2 (attestation-map, non-`[P]`; `requires: [2]`):**
- IMPLEMENT: Add ONE `#[tokio::test(flavor = "multi_thread")]` inside existing `mod v1_federation_inbound_b_fixtures`. Test shape per PRECON-4 Test 2.
- MIRROR ref: same sibling module; planner cites the Task 3 new test as an additional within-cohort sibling (after Task 3 lands, its shape is the nearest canonical).
- GOTCHA: same as Task 3.
- FILES YAML: `creates: []`, `modifies: [crates/server/tests/e2e.rs]`, `requires: [2]`.

**Task 5 — Retro** per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`:
- Output: `.claude/PRPs/reports/v1-federation-inbound-d-retro.md` with per-role signals + per-task complexity score + lessons promoted this phase + carry-forward items (primary candidate: TOCTOU eviction fix → v1-federation-inbound-e).

### 2.2 Scope boundary — explicit NOT building list

Per §0.2. The plan's §12 "NOT building" enumerates each item with a one-line rationale and future trigger:

1. **`inbox.rs:698` TOCTOU eviction fix** — DB-level concurrency design (atomic-SQL vs row-lock); warrants own plan + gate. Trigger: v1-federation-inbound-d retro.
2. **`governance_config`-backed MAX_* constants** — option-a scope; Rust `const` is the v0 approach. Trigger: post-pilot config-driven tuning.
3. **Postgres-backed rate counters** — option-a scope; adds DB write per inbound. Trigger: post-pilot scale analysis.
4. **LRU with access-timestamp tracking** — option-a scope; insertion-order eviction is option-b. Trigger: if eviction metrics show access-recency matters.
5. **New migration** — code-only fix; scope violation if proposed. Catch-fire.
6. **Shape G workflow change** — SUSPENDED per DQ #229. Trigger: 2026-06-01 re-enable check.
7. **Helper extraction across `inbox.rs` + `publish_trust_attestation.rs`** — circular-dep constraint stands (fed-in-c PRECON-3).

---

## 3. Required reading (in order, before drafting any plan section)

### 3.1 The defect + fix shape

1. **`crates/apub/activities/src/governance/inbox.rs:465-490`** — the rate-map insert site and surrounding context. Planner reads to confirm the exact data structure (`DashMap` vs other), line of insert, and any existing eviction logic.
2. **`crates/api/api/src/governance/publish_trust_attestation.rs:155-180`** — the attestation-map insert site. Planner reads to confirm the data structure and the current `subject_url` key construction.
3. **`rg "sha2" Cargo.toml crates/*/Cargo.toml` (shell command)** — confirm `sha2` is a workspace dep before committing to the key-hash approach in Task 2.
4. **`rg "sha2::Sha256" crates/ --include="*.rs" -l` (shell command)** — locate an existing `Sha256::digest` callsite in the codebase to use as MIRROR ref for Task 2.
5. **`rg "subject_url" crates/api/api/src/governance/publish_trust_attestation.rs` (shell command)** — enumerate ALL callsites of the map key construction before writing Task 2 body (PRECON-3 callsite-enumeration discipline).

### 3.2 The e2e sibling pattern

6. **`crates/server/tests/e2e.rs` — `mod v1_federation_inbound_b_fixtures`** — read via `rg -n "mod v1_federation_inbound_b_fixtures" crates/server/tests/e2e.rs` to find the module's line range, then `Read` with `offset: <start>, limit: 300`. Cite a specific sibling test as MIRROR ref for Tasks 3+4.

### 3.3 Mandatory file-class lesson injection (§2.4 of advisor-orchestrator.md)

For Tasks 3+4 (e2e edits), inject these lessons into §3 Required reading and §4 Constraints:

7. **`.claude/lessons/feedback_lemmy_error_no_std_error.md`** — Case A (`LemmyResult<()>` outer + bare `?`).
8. **`.claude/lessons/feedback_async_pool_test_pattern.md`** — connection acquisition.
9. **`.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`** — Edit budget ≤200 lines on e2e.rs.

For Task 1 (multi-write potential if Mutex-guarded update is multi-step):

10. **`.claude/lessons/feedback_multi_write_handlers_need_transactions.md`** — if the eviction + insert involves 2+ writes under a shared lock, confirm atomicity contract. (Advisory — planner judges applicability.)

### 3.4 The carry-forward context

11. **`.claude/PRPs/reports/v1-federation-inbound-c-retro.md`** — read §Carry-forward for the (b) DoS-hardening family description and per-role signals.
12. **`.claude/PRPs/handovers/v1-federation-inbound-d-bootstrap.md`** — advisor context; cite in plan §0 PRECONs as authority anchor.

### 3.5 Operational rules

13. **`.claude/rules/advisor-orchestrator.md`** — §1, §2, §3, §4, §5, §5.2 (validate-pending-laptop).
14. **`.claude/rules/branch-manager.md`** — file ownership.
15. **`.claude/rules/decision-queue.md`** — DQ schema + attribution.
16. **`.claude/rules/multi-lane-worktree.md`** — bm-cut creates `brehon-fork-fed-in-d` per `feedback_phase_lane_worktree_bootstrap_checklist.md`.
17. **`.claude/rules/pmd-invariants.md`** — five PMD invariants.

### 3.6 The §15 DoD shape

18. **`.claude/PRPs/plans/v1-federation-inbound-c.plan.md` §15** — mirror verbatim, adapted for v1-federation-inbound-d's edit set.

---

## 4. Constraints — what the planner MUST enforce

### 4.1 Scope discipline

- §13 has EXACTLY 6 tasks (Task 0 + 5 work tasks). No seventh task. No subdivision. If a task body exceeds ~200 lines of plan text, file `kind: "log"` proposing clarify-DQ rather than expanding scope.
- §12 "NOT building" enumerates the 7 items from §2.2.
- Silent migration introduction → catch-fire (scope violation).
- Any `governance_config`-backed MAX_* → file `kind: "log"` as future follow-up; do NOT in-scope.

### 4.2 Mirror discipline

- §10 MIRROR ref for Task 1 cites the exact line range of the `rate_map` insert site in `inbox.rs` (read at canonical-schema-first time).
- §10 MIRROR ref for Task 2 cites the existing `sha2::Sha256::digest` callsite found via `rg` in §3.1 step 4, AND the Task 1 eviction pattern shape.
- §10 MIRROR ref for Tasks 3+4 cites the exact line range of a sibling test inside `mod v1_federation_inbound_b_fixtures`.

### 4.3 Watchpoint specificity (§3.5 gate)

Every §4 watchpoint MUST name a specific file, line, struct, or const — no concept-only watchpoints. Minimum required watchpoints:

- **WP-1**: `inbox.rs:473` — rate-map eviction approach (option A `VecDeque` companion OR option B `LinkedHashMap` — name chosen struct + crate + thread-safety contract).
- **WP-2**: `publish_trust_attestation.rs:165` — SHA-256 key type change (`[u8; 32]`) + `MAX_ATTESTATION_MAP_ENTRIES` const value + eviction pattern (must be consistent with WP-1 choice).
- **WP-3**: `inbox.rs:698` TOCTOU — NOT in scope; watchpoint notes deferral to fed-in-e with the explicit choice (atomic-SQL vs row-lock) still open.
- **WP-4**: `sha2` dep — whether already present (confirmed by `rg` in §3.1 step 3) or requires addition; names the specific `Cargo.toml` location if an addition is needed.

### 4.4 DQ pre-reservation (per PRECON-7)

- Plan §13 Task 1 + Task 2 use placeholder ids `<id-A>` and `<id-B>` for the pre-reserved `validate-pending-laptop` stubs. Advisor fills at dispatch time; planner does NOT pick ids.
- Plan §0 PRECONs cites the `feedback_cohort_dq_id_collision.md` Option 3 mitigation.

### 4.5 §G4 classifier awareness

Anticipated fail modes for Tasks 1+2 cargo runs:

- `clippy::await_holding_lock` — if `Arc<Mutex<...>>` guard is held across `.await` (option B thread-safety gotcha). Plan §15 notes this as the anticipated allowlist class for Task 1/2.
- `clippy::doc_lazy_continuation` — if docstring added to the `MAX_*` const. Allowlist row exists.
- Cycle-count meta-rule: ≥3 fails with same `(error_class, file_basename)` → HARD REFUSAL catch-fire.

### 4.6 File-class lesson injection on e2e task briefs (post-plan-approval, advisor-side)

Tasks 3+4 impl-task briefs MUST inject lessons 7-9 from §3.3 (mechanical per advisor-orchestrator.md §2.4). The plan documents this in §13 Tasks 3+4 "Brief constraints" sub-section so the advisor cannot accidentally skip it.

### 4.7 Conformance-audit prevention checkpoint (§3.1.1)

Tasks 1+2 target `crates/apub/activities/src/governance/inbox.rs` and `crates/api/api/src/governance/publish_trust_attestation.rs` respectively — both match the Tier-1 audit scope. The plan §13 Tasks 1+2 include a "Brief constraints" note: advisor MUST run `/brehon-conformance-audit target_scope=file <path>` before authoring each impl-task brief (HARD STOP if omitted).

### 4.8 Brehon governance rules

- No destructive defaults (`.claude/rules/no-destructive-defaults.md`).
- DQ attribution (`.claude/rules/decision-queue.md`) — `validate-pending-laptop` entries: `from: "impl"`, advisor mutates `answered_by: "advisor-laptop"`.
- No cargo output paste (`.claude/rules/no-cargo-output-paste.md`).
- Pre-phase harness audit (`.claude/rules/pre-phase-harness-audit.md`) — Task 0 mandatory.

---

## 5. Dogfood — pre-commit walkthrough

This planning brief was dogfooded by the advisor against codebase state at HEAD `6a9f004a9`:

1. **Defect 1 verification**: bootstrap §1 cites `inbox.rs:473` as the rate-map insert site. Confirmed the line range is plausible based on fed-in-c retro's carry-forward description (exact line confirmed by planner at canonical-schema-first read time — advisor has not read the file here to avoid burning context on an 8000+ line file; planner reads it in §3 Required reading step 1).
2. **Defect 2 verification**: bootstrap §1 cites `publish_trust_attestation.rs:165` as the raw-string map key site. Same: confirmed plausible; planner reads at §3.1 step 2.
3. **`sha2` dep check**: `rg "sha2" Cargo.toml` — needs planner confirmation at §3.1 step 3. Brief notes catch if not found.
4. **`mod v1_federation_inbound_b_fixtures` existence**: `rg -n "mod v1_federation_inbound_b_fixtures" crates/server/tests/e2e.rs` — fed-in-b established this module (PR #139); fed-in-c Task 3 extended it. Existence confirmed by fed-in-c retro.
5. **TOCTOU deferred**: §0.2 and §2.2 both enumerate the deferral with rationale. Stop-and-ask tripwire in bootstrap §"Stop-and-ask tripwires" covers the silent-migration class.
6. **Watchpoint specificity**: §4.3 requires 4 named watchpoints with file:line or concrete struct. Concept-only watchpoints are explicitly blocked.
7. **PMD pre-search** (§2.3): searched for `rate limit LRU bounded`, `TOCTOU eviction Diesel RETURNING`, `attestation map key hash`. Two load-bearing hits: `feedback_multi_write_handlers_need_transactions.md` (run_transaction) + `feedback_pg_advisory_xact_lock_void_decode.md` (advisory lock mechanics). No prior inbox.rs DoS-hardening lessons — novel territory. Both hits injected where applicable (§3.3 note 10 for multi-write; §4.3 WP-3 notes advisory lock as one future TOCTOU option).

No brief drift from current HEAD state for the elements the advisor can verify without reading 8000+ line files.

---

## 6. After the planner ships

1. **Advisor runs §3.4 DoD smoke test**: every §15 command literally against current HEAD.
2. **Advisor runs §3.5 watchpoint specificity gate**: every §4 watchpoint cites file:line + named struct/const.
3. **Advisor checks migration catch-fire**: confirm §13 has zero migration tasks.
4. **User Gate 1 (plan approval)** via `AskUserQuestion`.
5. **On approval**: queue `[role:bm-task] bm-cut — phase-v1-federation-inbound-d`.
6. **Post-bm-cut**: lane worktree `brehon-fork-fed-in-d` cut off `phase-v1-federation-inbound-d`. Subsequent advisor session is lane-dedicated.

---

## 7. Definition-of-done for THIS planning brief

- [x] §0 establishes both defects with file:line citations + the option-b rationale.
- [x] §0.1 captures 8 PRECONs (scope, mitigation shapes, key-hash, e2e, no migration, DoD shape, cohort, review point).
- [x] §0.2 explicitly de-scopes TOCTOU + option-a items with per-item rationale.
- [x] §1 dispatch line is <100 chars.
- [x] §2.1 enumerates exactly 6 tasks (Task 0 + Tasks 1-5) with IMPLEMENT/MIRROR/GOTCHA/VALIDATE/FILES YAML shape.
- [x] §2.2 mirrors §0.2 in the plan-template "NOT building" enumeration shape.
- [x] §3 Required reading is 18 items, ordered by purpose.
- [x] §4 Constraints cover scope, mirror, watchpoint specificity, DQ pre-reservation, §G4 awareness, conformance-audit gate, governance rules.
- [x] §5 dogfood covers 7 verification steps grounded in current state.
- [x] §6 names the post-planner stage shape.
- [x] No emojis, no inline log dumps, no speculation.
