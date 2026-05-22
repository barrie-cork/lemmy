# v1-federation-inbound-d planning brief — per-actor rate-map bound (option-b, post-clarify narrowing)

**Written**: 2026-05-22 by advisor session (laptop, canonical CWD `C:/Users/barri/Developer/brehon-fork`, branch `governance-v0`).
**Revised**: 2026-05-22 after `/brehon-clarify` surfaced bootstrap-vs-reality drift. Original brief @ `bcc022310` mis-cited file:line numbers and data-structure shapes from the fed-in-d bootstrap (which itself drifted from HEAD). User answered three clarify questions (B1/B2/B3); resulting scope is ~70% narrower than the original draft. This is the corrected brief authored at HEAD `bcc022310`.
**Subagent target**: `planning` (Opus 4.7 — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/v1-federation-inbound-d-planning-1` from `governance-v0` committed HEAD. Plan file commits + pushes back to `governance-v0` at finalize. The phase branch `phase-v1-federation-inbound-d` is cut at `bm-cut` AFTER plan approval (User Gate 1).
**Authority anchor**: this brief IS the canonical planning input for `v1-federation-inbound-d`. Scope is the **post-clarify narrowing of option-b**: a single fix — insertion-order eviction bound on the **per-actor** rate-limit map (`rate_per_actor_counts`) at the use site in `publish_trust_attestation.rs`. The per-peer map bound (originally Defect 1) is OUT (existing 2-hour `retain()` is acceptable per user B1). SHA-256 key-hash is OUT (per user B3). Cohort `[P]` is OUT — only one task touches `crates/`.
**Sub-phase target**: `v1-federation-inbound-d` — DoS-hardening for the per-actor inbound rate-limit counter. One Rust-side bound check + one e2e test; ships under `validate-pending-laptop` DoD (Shape G SUSPENDED until 2026-06-01 per DQ #229 / `project_shape_g_suspended_2026_05_16.md`).

---

## 0. Why this sub-phase exists (the design problem — read first)

### 0.1 The defect in scope (post-clarify option-b)

fed-in-b (PR #139, `413ef5899`) introduced two in-memory rate-limit maps in `crates/apub/activities/src/governance/inbox.rs`:

```rust
// inbox.rs:464-468  — per-peer
pub(crate) fn rate_per_peer_counts() -> &'static Mutex<HashMap<(String, i64), u32>> {
  static CELL: OnceLock<Mutex<HashMap<(String, i64), u32>>> = OnceLock::new();
  CELL.get_or_init(|| Mutex::new(HashMap::new()))
}

// inbox.rs:470-474  — per-actor
pub(crate) fn rate_per_actor_counts() -> &'static Mutex<HashMap<(String, i64), u32>> {
  static CELL: OnceLock<Mutex<HashMap<(String, i64), u32>>> = OnceLock::new();
  CELL.get_or_init(|| Mutex::new(HashMap::new()))
}
```

Both maps are keyed on `(identifier, hour_bucket)` where the identifier is `peer_domain` (per-peer) or `subject_url` (per-actor), and `hour_bucket = chrono::Utc::now().timestamp() / 3600` (defined at `inbox.rs:477-479`).

**Both maps already have insertion-time eviction** for entries older than `bucket - 1` (i.e. the prior hour). Confirmed:

- Per-peer use site at `inbox.rs:544-552` runs `counts.retain(|(_, b), _| *b >= bucket - 1);` on every insert.
- Per-actor use site at `crates/apub/activities/src/governance/publish_trust_attestation.rs:158-167` runs the SAME `counts.retain(|(_, b), _| *b >= bucket - 1);` on every insert (line 163).

So the maps are NOT unbounded across time — they self-prune to ≤2 hour-buckets' worth of entries. The DoS surface is therefore the count of **distinct identifiers within the current + prior hour bucket**, not "all-time growth".

**Where the per-peer map's surface IS bounded by current code:**
Per-peer DoS requires N distinct *peer domains*. To inflate `rate_per_peer_counts` by N entries within an hour, an attacker needs N distinct peers federating to us — i.e. N distinct domains. This is naturally rate-limited by inbound federation's allowlist + signature checks (a peer that isn't already trusted/allowed gets dropped at Gate 1, before the rate map is touched).

**Where the per-actor map's surface IS unbounded by current code:**
Per-actor DoS requires N distinct `subject_url` strings. A *single* attacking peer (already past the trust gate) can craft N distinct `subject_url` values per inbound trust-attestation activity and force N distinct entries into `rate_per_actor_counts`. Within a single hour bucket, this maps to ~80 bytes per URL (typical) × N → linear heap growth driven entirely by malicious peer payload variety. The 2-hour `retain()` only ages out the prior hour; within the active hour bucket, there is no cap.

**The fix in scope:** add an insertion-order eviction bound at the per-actor use site (`publish_trust_attestation.rs:158-167`) that caps the per-actor map at `MAX_PER_ACTOR_RATE_ENTRIES` (a Rust `const`). When `counts.len()` reaches the cap, evict the oldest-inserted entry before inserting the new one. The per-peer map is NOT touched (B1).

### 0.2 What this sub-phase does NOT ship (explicit de-scope)

Per user clarify-gate answers 2026-05-22:

- **Per-peer rate-map bound (originally Defect 1)** — not in scope (user B1). The existing 2-hour `retain()` is acceptable given the inbound allowlist + signature checks gate domain-distinct flooding upstream.
- **SHA-256 key-hash for per-actor map key** — not in scope (user B3). The `subject_url: String` key stays readable for debug-log lookup; key-hash deferred indefinitely (lands only if post-pilot metrics show per-entry heap is load-bearing).
- **TOCTOU eviction race on `inbox.rs:654` (`evict_oldest_unreviewed_if_needed`)** — not in scope. DB-level concurrency design (atomic-SQL vs row-lock) warrants its own plan + gate. Defer to v1-federation-inbound-e.
- **`[P]` cohort dispatch** — not applicable. Only ONE task edits `crates/`; cohort logic does not apply.
- **`governance_config`-backed `MAX_PER_ACTOR_RATE_ENTRIES`** — out of scope (option-a). Rust `const` is the v0 approach.
- **Helper extraction across `inbox.rs` + `publish_trust_attestation.rs`** — circular-dep constraint stands (fed-in-c PRECON-3).
- **Any new migration** — code-only fix; if planner proposes one, catch-fire.
- **Shape G workflow change** — SUSPENDED per DQ #229.

---

## 0.1 Pre-resolved facts (READ BEFORE §1 — BINDING)

### 0.1.1 — PRECON-1 — Scope: per-actor rate-map bound only

**User decision 2026-05-22 (post-clarify B1+B2+B3):** v1-federation-inbound-d ships exactly one code change — an insertion-order eviction bound on `rate_per_actor_counts`. The per-peer map is OUT. The key-hash is OUT. The cohort split is OUT. The TOCTOU fix is OUT.

**Binding consequence:** §13 has exactly 4 tasks (Task 0 pre-flight + Task 1 bound + Task 2 e2e + Task 3 retro). If the planner proposes a 5th task — for any reason — catch-fire.

### 0.1.2 — PRECON-2 — Bound location: at the use site, not the definition

**The advisor's binding decision:** the bound check + eviction logic is added at the **use site** (`publish_trust_attestation.rs:158-167`), inside the `let exceeded_actor = { ... }` scope, BEFORE the `let entry = counts.entry(...).or_insert(0);` line. The map definition at `inbox.rs:470-474` stays unchanged (no helper extraction, no per-map config — the bound is local to the per-actor counter only).

**Rationale:** the per-peer map at `inbox.rs:544-552` does NOT get the same bound (per B1), so a shared helper at the map-definition level would imply a cap that applies to both maps. Localising the bound to the per-actor use site preserves the "different maps, different policies" invariant.

**The fix shape:**

```rust
// Before counts.entry(...).or_insert(0):
const MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000;

// Insertion-order eviction: if the map already at cap and this key
// is not present, evict the oldest entry by bucket-then-arbitrary order.
let key = (subject_url.to_string(), bucket);
if counts.len() >= MAX_PER_ACTOR_RATE_ENTRIES && !counts.contains_key(&key) {
    // Find the lowest bucket value (oldest hour) and remove one entry from it.
    if let Some(oldest_key) = counts
        .iter()
        .min_by_key(|(k, _)| k.1)
        .map(|(k, _)| k.clone())
    {
        counts.remove(&oldest_key);
    }
}
let entry = counts.entry(key).or_insert(0);
```

(Planner refines the exact wording — this is the shape contract; the planner may use `iter()` / `keys().next()` / a `BTreeMap` derivation if cleaner. The contract is: insertion-order or bucket-priority eviction; O(N) eviction acceptable at this cap.)

**Constant name and value:** `MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000`. Planner may propose a different value with rationale in plan §4 watchpoint, but must NOT leave it implicit.

### 0.1.3 — PRECON-3 — Crate location (factual correction from original brief)

**The advisor's binding decision:** `publish_trust_attestation.rs` lives at `crates/apub/activities/src/governance/publish_trust_attestation.rs` (the apub crate). The original draft brief incorrectly placed it under `crates/api/api/...`; that was a bootstrap-drift artifact corrected at clarify-gate time. The §4.7 conformance-audit prevention checkpoint applies to `crates/apub/activities/src/governance/**.rs` regardless — Task 1's target is on the audit list.

### 0.1.4 — PRECON-4 — Migrations: ZERO

The fix is in-memory data-structure logic only. Expected migration count: **zero**. If the generated plan proposes a new migration under `migrations/**`, catch-fire (scope violation).

### 0.1.5 — PRECON-5 — `validate-pending-laptop` DoD shape (Shape G SUSPENDED)

Shape G is SUSPENDED until 2026-06-01 (DQ #229; `project_shape_g_suspended_2026_05_16.md`). v1-federation-inbound-d ships under `validate-pending-laptop` per `.claude/rules/advisor-orchestrator.md` §5.2.

**Binding consequence:** §15 MUST include:

- `bash scripts/brehon/cargo-check.sh --workspace --features full`
- `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings`
- `cargo test --test e2e --no-run -p lemmy_server`
- **Phase 2 e2e** (full run) per user gate 4: default LOCAL via `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-d-<sha>.log 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`.

**Forward reminder:** if v1-federation-inbound-d crosses 2026-06-01 UTC, re-check DQ #229 status at new-day session start.

### 0.1.6 — PRECON-6 — No cohort dispatch

**The advisor's binding decision:** only ONE code task touches `crates/` (Task 1). Task 2 (e2e) is non-`[P]` and `requires: [1]`. There is no `[P]` cohort.

**Binding consequence:** Plan §13 task list does NOT use `[P]` markers. The advisor will not pre-reserve cohort DQ ids (per `feedback_cohort_dq_id_collision.md` Option 3, the mitigation is only needed for `[P]` cohorts).

### 0.1.7 — PRECON-7 — Review point: user gate after clarify, before planning-Junior dispatch

Standard four-role Brehon discipline:

1. Author this brief (revised). ✓
2. Commit + push to `governance-v0`.
3. (Already run /brehon-clarify; user resolved B1/B2/B3 at clarify-gate; this brief reflects those answers.) Run /brehon-clarify a second time against this revised brief for residual coverage gaps.
4. Resolve any new `kind: "clarify"` DQ raised.
5. Surface brief to user for approval (brief-level).
6. On user approval, queue the planning Junior task with `base_branch=governance-v0`.
7. Planning Junior writes the plan; advisor runs §3.4 DoD smoke + §3.5 watchpoint-specificity + §3.7 dogfood gates; surfaces plan for User Gate 1 (plan approval).

---

## 1. Role + dispatch line

`[role:planning] v1-federation-inbound-d plan — per-actor rate-map bound (option-b post-clarify); validate-pending-laptop DoD`

The actual create-task description (single line, <100 chars):

```
[role:planning] v1-federation-inbound-d plan — see .claude/PRPs/briefs/v1-federation-inbound-d-planning-1.md
```

---

## 2. Scope — what to produce

Produce **one plan file** at **`.claude/PRPs/plans/v1-federation-inbound-d.plan.md`** following `.claude/PRPs/templates/plan.template.md`'s 20-section schema literally.

### 2.1 What this sub-phase ships (the deliverable surface)

The plan's §13 task list MUST cover EXACTLY the following four tasks and NOTHING beyond:

**Task 0 — Pre-flight harness audit** (mandatory per `.claude/rules/pre-phase-harness-audit.md`):

- Probes 0-4 (Docker daemon + wrapper script behaviour: per-crate, feature, test target, exit-code propagation).
- DoD smoke: every §15 command run literally against phase-branch HEAD; expected-red set documented for Task 1 not yet shipped.
- Clippy baseline capture against phase-branch HEAD (must be exit 0 before Task 1 dispatch).

**Task 1 — Per-actor rate-map bound at `publish_trust_attestation.rs:158-167`:**

- IMPLEMENT: Add `MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000` const at module scope or function scope (planner picks). Inside the existing `let exceeded_actor = { ... }` block at line 158-167, BEFORE the `let entry = counts.entry(...).or_insert(0);` line, insert the insertion-order eviction guard per PRECON-2's fix shape. Preserve the existing `counts.retain(|(_, b), _| *b >= bucket - 1);` two-hour prune at line 163 (it stays).
- MIRROR ref: the per-peer use site at `crates/apub/activities/src/governance/inbox.rs:544-552` shows the canonical `Mutex<HashMap<(String, i64), u32>>` access pattern (lock + retain + entry-or-insert + saturating_add). The new bound check fits between `retain` and `entry`. Cite this line range in plan §10 MIRROR ref.
- GOTCHA: preserve the `Mutex` guard discipline — do NOT release the guard between `retain()`, the new bound check, and `entry()`; they must all run under the same lock. Planner verifies no `.await` between `counts.lock()` (line 159-161) and the end of the block.
- GOTCHA: the const is Rust-side only; do NOT read from `governance_config` (out of scope per PRECON-1 + PRECON-4).
- GOTCHA: do NOT touch `rate_per_peer_counts` or its use site at `inbox.rs:544-552` (per B1 + PRECON-2).
- VALIDATE: `bash scripts/brehon/cargo-check.sh --workspace --features full` + `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings` both exit 0.
- FILES YAML: `creates: []`, `modifies: [crates/apub/activities/src/governance/publish_trust_attestation.rs]`, `requires: []`.

**Task 2 — E2e for Task 1 (per-actor rate-map bound, `requires: [1]`):**

- IMPLEMENT: Add ONE `#[tokio::test(flavor = "multi_thread")]` inside existing `mod v1_federation_inbound_b_fixtures` in `crates/server/tests/e2e.rs`. Test shape:
  1. Setup: seed config to allow many actor attestations (raise `federation.inbound.per_actor_rate_per_hour` cap to a value > MAX_PER_ACTOR_RATE_ENTRIES so the cap is the bound enforcer, not the per-actor rate).
  2. Act: send `MAX_PER_ACTOR_RATE_ENTRIES + 1` trust-attestation activities, each with a distinct `subject_url`.
  3. Assert: `rate_per_actor_counts().lock().unwrap().len() == MAX_PER_ACTOR_RATE_ENTRIES` (NOT `MAX + 1`).
  4. Assert: the FIRST `subject_url` inserted is no longer present in the map (evicted).
- MIRROR ref: an existing test inside `mod v1_federation_inbound_b_fixtures` that exercises rate-counter state (planner picks the closest sibling at canonical-schema-first read; cite the exact line range in plan §10). The `per_peer_rate_limit_returns_429` test at lines 15600+ is the closest shape (planner verifies line range against current HEAD).
- GOTCHA: `LemmyResult<()>` outer + bare `?` (Case A per `feedback_lemmy_error_no_std_error.md`). Edit budget ≤200 lines per `feedback_junior_worker_e2e_edit_hang.md`.
- GOTCHA: e2e file is 15600+ lines; never full-file Edit. Use Read with offset+limit to find the sibling module, then targeted Edit.
- VALIDATE: `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-d-<sha>.log 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`. Exit marker: `E2E_EXIT_0`.
- FILES YAML: `creates: []`, `modifies: [crates/server/tests/e2e.rs]`, `requires: [1]`.

**Task 3 — Retro** per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`:

- Output: `.claude/PRPs/reports/v1-federation-inbound-d-retro.md` with per-role signals, per-task complexity score, lessons promoted, carry-forward items.
- **Mandatory carry-forward item**: bootstrap-vs-reality drift class. The fed-in-d bootstrap committed at `dc9bf17a2` cited `inbox.rs:473` / `publish_trust_attestation.rs:165` / `inbox.rs:698` — all three line numbers were wrong vs HEAD. Original brief inherited the drift. `/brehon-clarify` caught it because the advisor `grep`ed live code before authoring DQ entries. Retro proposes a tighter discipline (the bootstrap author MUST `rg` live code for every file:line citation before commit) and a lesson candidate: `feedback_bootstrap_file_line_citations_must_grep_head.md` (if user judges promotable).
- **Mandatory carry-forward item**: TOCTOU eviction fix on `inbox.rs:654` (`evict_oldest_unreviewed_if_needed`) — still deferred. Retro proposes the next slice (likely v1-federation-inbound-e) with a §0.1 PRECON naming atomic-SQL-with-RETURNING vs `SELECT FOR UPDATE SKIP LOCKED` choice as the gate-1 question.
- **Optional carry-forward item**: per-peer rate-map bound (was Defect 1, dropped per user B1). Retro evaluates whether post-pilot DoS metrics justify revisiting.

### 2.2 Scope boundary — explicit NOT building list

Per §0.2. The plan's §12 "NOT building" enumerates these (each with one-line rationale and future trigger):

1. **Per-peer rate-map bound (`inbox.rs:544-552`)** — user B1 2026-05-22; existing 2-hour `retain()` + upstream allowlist gate considered adequate. Trigger: post-pilot DoS metrics show per-peer surface needs hardening.
2. **SHA-256 key-hash for per-actor map** — user B3 2026-05-22; per-entry heap savings (~800 KB at 10k entries × 80B URL) not worth losing debuggability. Trigger: post-pilot metrics show per-entry heap is load-bearing.
3. **TOCTOU eviction fix on `inbox.rs:654`** — DB-level concurrency design (atomic-SQL vs row-lock) warrants own plan + gate. Trigger: v1-federation-inbound-e.
4. **`governance_config`-backed `MAX_PER_ACTOR_RATE_ENTRIES`** — option-a scope; Rust `const` is v0. Trigger: post-pilot config-driven tuning.
5. **Postgres-backed rate counters** — option-a scope; adds DB write per inbound. Trigger: post-pilot scale analysis.
6. **`[P]` cohort dispatch** — only one task touches `crates/`; cohort logic does not apply. Trigger: never (architectural).
7. **New migration** — code-only fix; catch-fire if proposed.
8. **Shape G workflow change** — SUSPENDED per DQ #229. Trigger: 2026-06-01 re-enable check.
9. **Helper extraction across `inbox.rs` + `publish_trust_attestation.rs`** — circular-dep constraint (fed-in-c PRECON-3) + B1 prevents shared helper. Trigger: never (architectural).

---

## 3. Required reading (in order, before drafting any plan section)

### 3.1 The defect + fix shape

1. **`crates/apub/activities/src/governance/inbox.rs:464-479`** — the map definitions (`rate_per_peer_counts`, `rate_per_actor_counts`, `current_hour_bucket`). Planner reads to confirm the `Mutex<HashMap<(String, i64), u32>>` shape.
2. **`crates/apub/activities/src/governance/inbox.rs:540-565`** — the per-peer USE site (MIRROR ref for the lock+retain+entry pattern; NOT the fix site).
3. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:130-190`** — the per-actor USE site (the fix site for Task 1). Read to confirm the `let exceeded_actor = { ... }` block layout and the `subject_url` key construction.
4. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:155-167` (drill-down)** — exact lines where Task 1's bound check is inserted (between `counts.retain(...)` and `counts.entry(...)`).

### 3.2 The e2e sibling pattern

5. **`crates/server/tests/e2e.rs` — `mod v1_federation_inbound_b_fixtures`** — read via `rg -n "mod v1_federation_inbound_b_fixtures" crates/server/tests/e2e.rs` to find the module's line range, then `Read` with `offset: <start>, limit: 300`. Identify the test that most closely resembles "trust-attestation flood" (likely `per_peer_rate_limit_returns_429` or a sibling). Cite the exact line range as MIRROR ref for Task 2 in plan §10.

### 3.3 Mandatory file-class lesson injection (§2.4 of advisor-orchestrator.md)

For Task 2 (e2e edit), inject these lessons into §3 Required reading and §4 Constraints:

6. **`.claude/lessons/feedback_lemmy_error_no_std_error.md`** — Case A (`LemmyResult<()>` outer + bare `?`).
7. **`.claude/lessons/feedback_async_pool_test_pattern.md`** — connection acquisition.
8. **`.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`** — Edit budget ≤200 lines on e2e.rs.

### 3.4 The carry-forward context

9. **`.claude/PRPs/reports/v1-federation-inbound-c-retro.md`** — read §Carry-forward for the DoS-hardening family description.
10. **`.claude/PRPs/handovers/v1-federation-inbound-d-bootstrap.md`** — advisor context. **NOTE**: this file's file:line citations (`inbox.rs:473`, `publish_trust_attestation.rs:165`, `inbox.rs:698`) are WRONG vs HEAD; the planner uses this brief's corrected citations (§3.1 above), not the bootstrap's. Cited only for the "(b) Copilot DoS-hardening family" framing.

### 3.5 Operational rules

11. **`.claude/rules/advisor-orchestrator.md`** — §1, §2, §3, §5, §5.2 (validate-pending-laptop). Note: §4 cohort dispatch does NOT apply (no `[P]` tasks).
12. **`.claude/rules/branch-manager.md`** — file ownership.
13. **`.claude/rules/decision-queue.md`** — DQ schema + attribution.
14. **`.claude/rules/multi-lane-worktree.md`** — bm-cut creates `brehon-fork-fed-in-d` per `feedback_phase_lane_worktree_bootstrap_checklist.md`.
15. **`.claude/rules/pmd-invariants.md`** — five PMD invariants.

### 3.6 The §15 DoD shape

16. **`.claude/PRPs/plans/v1-federation-inbound-c.plan.md` §15** — mirror verbatim, adapted for v1-federation-inbound-d's single-file edit set.

---

## 4. Constraints — what the planner MUST enforce

### 4.1 Scope discipline

- §13 has EXACTLY 4 tasks (Task 0 + Tasks 1-3). No fifth task. No `[P]` markers.
- §12 "NOT building" enumerates the 9 items from §2.2.
- Silent migration introduction → catch-fire.
- Any second-defect addition (per-peer bound, key-hash, TOCTOU, helper extraction) → file `kind: "log"`; do NOT in-scope.

### 4.2 Mirror discipline

- §10 MIRROR ref for Task 1 cites `crates/apub/activities/src/governance/inbox.rs:540-565` (the per-peer use site) as the canonical lock+retain+entry pattern. Task 1 introduces the bound check on top of this pattern at the per-actor site.
- §10 MIRROR ref for Task 2 cites the exact line range of a sibling test inside `mod v1_federation_inbound_b_fixtures` (planner picks at canonical-schema-first read).

### 4.3 Watchpoint specificity (§3.5 gate)

Every §4 watchpoint MUST name a specific file, line, struct, or const. Minimum required watchpoints:

- **WP-1**: `publish_trust_attestation.rs:158-167` — Task 1 fix site. Watchpoint names: the eviction strategy (oldest-by-bucket OR `iter().next()` OR `BTreeMap` derivation), the `MAX_PER_ACTOR_RATE_ENTRIES` const value (10_000) and its location (function-local vs module-scope), and the lock-discipline contract (no `.await` between `lock()` and end of block).
- **WP-2**: `inbox.rs:471-474` — the map definition stays unchanged. Watchpoint notes "no edits expected here" (forward catch if the planner is tempted to add the bound at the definition).
- **WP-3**: `inbox.rs:544-552` — the per-peer use site stays unchanged. Watchpoint cites user B1 as the rationale.
- **WP-4**: `inbox.rs:654` — the TOCTOU eviction function. Watchpoint notes deferral to v1-federation-inbound-e with the open choice (atomic-SQL-with-RETURNING vs `SELECT FOR UPDATE SKIP LOCKED`).

### 4.4 DQ pre-reservation: NOT applicable

Only one task touches `crates/`; no `[P]` cohort. No DQ pre-reservation needed (per PRECON-6). Plan does NOT cite `feedback_cohort_dq_id_collision.md` Option 3.

### 4.5 §G4 classifier awareness

Anticipated fail modes for Task 1 cargo runs:

- `clippy::needless_collect` or `clippy::if_same_then_else` — if the eviction code uses an unnecessary intermediate collection. Mechanical fix per `feedback_clippy_test_style.md`.
- `clippy::await_holding_lock` — if any `.await` slips into the `let exceeded_actor = { ... }` block. Mechanical fix: move await out of block.
- Cycle-count meta-rule: ≥3 fails with same `(error_class, file_basename)` → HARD REFUSAL catch-fire.

### 4.6 File-class lesson injection on Task 2 brief (post-plan-approval, advisor-side)

Task 2 impl-task brief MUST inject lessons 6-8 from §3.3 (mechanical per advisor-orchestrator.md §2.4). The plan documents this in §13 Task 2's "Brief constraints" sub-section.

### 4.7 Conformance-audit prevention checkpoint (§3.1.1)

Task 1 targets `crates/apub/activities/src/governance/publish_trust_attestation.rs` — Tier-1 audit scope. The plan §13 Task 1 includes a "Brief constraints" note: advisor MUST run `/brehon-conformance-audit target_scope=file crates/apub/activities/src/governance/publish_trust_attestation.rs` before authoring Task 1's impl-task brief (HARD STOP if omitted).

### 4.8 Brehon governance rules

- No destructive defaults (`.claude/rules/no-destructive-defaults.md`).
- DQ attribution (`.claude/rules/decision-queue.md`) — `validate-pending-laptop` entries: `from: "impl"`, advisor mutates `answered_by: "advisor-laptop"`.
- No cargo output paste (`.claude/rules/no-cargo-output-paste.md`).
- Pre-phase harness audit (`.claude/rules/pre-phase-harness-audit.md`) — Task 0 mandatory.

---

## 5. Dogfood — pre-commit walkthrough

This planning brief was dogfooded by the advisor against codebase state at HEAD `bcc022310` (advisor's brief commit; reads happened immediately after):

1. **Defect verification via live code**:
   - `grep -n "rate_per_actor_counts" crates/apub/activities/src/governance/inbox.rs` → line 471 (definition).
   - `grep -n "rate_per_actor_counts" crates/apub/activities/src/governance/publish_trust_attestation.rs` → line 159 (use site).
   - `Read inbox.rs[460-490]`: confirmed `Mutex<HashMap<(String, i64), u32>>`, NOT `DashMap<String, (u32, Instant)>` as bootstrap claimed.
   - `Read inbox.rs[540-565]`: confirmed per-peer use site has `counts.retain(|(_, b), _| *b >= bucket - 1)` at line 548 — pre-existing 2-hour eviction.
   - `Read publish_trust_attestation.rs[155-170]`: confirmed per-actor use site has the SAME `counts.retain(...)` prune at line 163.
2. **TOCTOU function location**:
   - `grep -n "evict_oldest_unreviewed_if_needed" crates/apub/activities/src/governance/inbox.rs` → line 654 (definition). Bootstrap said `:698`; HEAD shows `:654`. Bootstrap is wrong; this brief uses `:654` and the deferred-to-fed-in-e watchpoint cites it correctly.
3. **Crate location**:
   - `find crates/apub -name "publish_trust_attestation.rs"` → confirmed `crates/apub/activities/src/governance/publish_trust_attestation.rs`. Bootstrap said `crates/api/api/...`; that path doesn't even exist. Brief corrected.
4. **`sha2` dep**: confirmed at workspace `Cargo.toml` (`sha2 = "0.10"`). Now irrelevant since user B3 dropped the key-hash, but pre-verified for the brief's audit trail.
5. **Sibling e2e module existence**: `grep -n "mod v1_federation_inbound_b_fixtures" crates/server/tests/e2e.rs` — confirmed by fed-in-c retro + grep. Planner reads at canonical-schema-first time.
6. **No silent migration**: Task 1 is in-memory data-structure logic only; `creates: []` in FILES YAML. Watchpoint WP-2 + WP-3 + scope §4.1 enforce.
7. **PMD pre-search** (§2.3): `memory_search_hybrid("rate limit map bounded eviction federation inbound")` — relevant hits: `feedback_multi_write_handlers_need_transactions.md` (advisory — not required; no multi-write in Task 1), `feedback_pg_advisory_xact_lock_void_decode.md` (advisory — TOCTOU is deferred). No prior bound-eviction lessons; novel territory.

The dogfood reads were done at HEAD; no remaining bootstrap-vs-reality drift in this brief.

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

- [x] §0 establishes the single in-scope defect with file:line citations grepped from HEAD.
- [x] §0.1 captures 7 PRECONs (scope, bound location, crate location, no migration, DoD shape, no cohort, review point).
- [x] §0.2 explicitly de-scopes per-peer bound + key-hash + TOCTOU + helper extraction + new migration + cohort + Shape G + governance_config-backed const + Postgres-backed counter with per-item rationale citing user clarify answers.
- [x] §1 dispatch line is <100 chars.
- [x] §2.1 enumerates exactly 4 tasks (Task 0 + Tasks 1-3) with IMPLEMENT/MIRROR/GOTCHA/VALIDATE/FILES YAML shape.
- [x] §2.2 mirrors §0.2 in plan-template "NOT building" enumeration shape.
- [x] §3 Required reading is 16 items; bootstrap drift documented at item 10 so planner doesn't re-import the wrong line numbers.
- [x] §4 Constraints cover scope, mirror, 4 watchpoints, §G4 awareness, conformance-audit gate, governance rules.
- [x] §5 dogfood is grounded in 7 live-code reads against HEAD.
- [x] §6 names the post-planner stage shape.
- [x] No emojis, no inline log dumps, no speculation, no unwritten cross-references.
