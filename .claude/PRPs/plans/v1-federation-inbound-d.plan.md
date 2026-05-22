# Plan: v1-federation-inbound-d — per-actor rate-map insertion-order bound at `publish_trust_attestation.rs`

## 1. Summary

This sub-phase ships a single in-memory DoS-hardening fix on the per-actor federation-inbound rate-limit counter: an insertion-order eviction bound on `rate_per_actor_counts` enforced at the use site in `crates/apub/activities/src/governance/publish_trust_attestation.rs::check_per_actor_rate_limit`. Acceptance: after Tasks 1+2 land, a single authenticated peer cannot inflate the in-memory `HashMap<(String, i64), u32>` beyond `MAX_PER_ACTOR_RATE_ENTRIES = 10_000` distinct keys within a single hour bucket by crafting distinct `subject_url` values; a crate-internal unit test asserts the bound holds at `len() == MAX_PER_ACTOR_RATE_ENTRIES` after `MAX + 1` distinct inserts and confirms the oldest entry is the one evicted. Zero migrations, zero new file, zero ADR change, zero e2e edit; one Rust `const` + one branch inside the existing `let exceeded_actor = { ... }` block + one `#[cfg(test)] mod` inside the same file.

## 2. Source

- `.claude/PRPs/briefs/v1-federation-inbound-d-planning-1.md` (this plan's authority anchor; revised post-clarify) @ `7e6c4202f`
- `.claude/PRPs/handovers/v1-federation-inbound-d-bootstrap.md` (initial framing — but file:line citations therein are STALE; this plan uses brief §3.1 corrected citations) @ `6a9f004a9`
- `.claude/PRPs/plans/v1-federation-inbound-c.plan.md` §15 (validate-pending-laptop DoD shape, same-lane exemplar) @ `governance-v0`
- `.claude/PRPs/reports/v1-federation-inbound-c-retro.md` §"Decision-point: v1-federation-inbound-d — (b) Copilot DoS-hardening family in/out?" (carry-forward) @ `governance-v0`
- `crates/apub/activities/src/governance/inbox.rs:464-479` (map definitions + `current_hour_bucket` — read at brief author time)
- `crates/apub/activities/src/governance/inbox.rs:540-565` (per-peer use site — MIRROR ref for the lock+retain+entry shape)
- `crates/apub/activities/src/governance/publish_trust_attestation.rs:118-187` (per-actor use site — Task 1 fix site)
- `crates/apub/activities/src/governance/publish_sanction_notice.rs:612-720` (crate-internal `#[cfg(test)] mod tests` sibling — MIRROR ref for Task 2)
- DQ `a3d0e9941441-005` (advisor user-relayed B1 2026-05-22): Task 2 is a crate-internal unit test, NOT an e2e test (visibility + feasibility)
- DQ `a3d0e9941441-006` (advisor self-resolved 2026-05-22): residual second-pass clarify confirmations

Lessons that bind decisions (cited where they fire):

- `.claude/lessons/feedback_clippy_test_style.md` — denies `unwrap`/`expect` in test bodies; test module follows workspace lint discipline (Task 2)
- `.claude/lessons/feedback_features_full_workspace_only.md` + `feedback_features_full_p_crate_incompatible.md` — `--workspace --features full` for check / clippy; `-p <crate>` never with `--features full` (Tasks 1+2)
- `.claude/lessons/feedback_complexity_score_pre_split.md` — §5 score computation
- `.claude/lessons/feedback_advisor_watchpoint_specificity.md` — §4 watchpoint citations
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` — post-bm-cut worktree hygiene (`brehon-fork-fed-in-d`)
- `.claude/lessons/feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md` — retro shape (Task 3)
- `.claude/lessons/feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md` — §15 dry-run discipline at plan approval
- `.claude/lessons/feedback_wrapper_script_flag_silence.md` — Task 0 Probes 1-4 rationale
- `.claude/lessons/feedback_brehon_verify_pre_merge.md` — §16a stories drive `/brehon-verify`
- `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md` — Phase-2 e2e invocation discipline (used in §15.4 as regression gate, not as the new test's runner)
- `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — canonical-schema-first for §10.2 (sibling `mod tests` read before mirroring)
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — FILES YAML per task
- `.claude/lessons/feedback_principles_not_rules.md` — §G4 allowlist is conservative-by-design (§4.5)

ADRs: none modified. The fix is a pure in-memory data-structure bound; no schema change, no AP-protocol change, no ADR-006 / ADR-013 / ADR-014 / ADR-015 surface touched.

## 3. Problem statement

`rate_per_actor_counts` (defined `crates/apub/activities/src/governance/inbox.rs:470-474`) is an in-memory `Mutex<HashMap<(String, i64), u32>>` keyed on `(subject_url, hour_bucket)` where `subject_url` is an attacker-controllable AP URL string extracted from each inbound `Create(TrustAttestation)`'s `object.rest["subject"]` (per `publish_trust_attestation.rs:126-133`).

The use site at `publish_trust_attestation.rs:158-167` already runs `counts.retain(|(_, b), _| *b >= bucket - 1)` on every insert (line 163), so the map self-prunes to ≤2 hour-buckets' worth of entries — it is NOT unbounded across time.

**However:** within a single hour bucket, a single allowlisted peer that has passed Gate 1 (trust check) can submit N distinct `Create(TrustAttestation)` activities each carrying a unique `object.rest["subject"]` URL and force N distinct `(subject_url, bucket)` keys into the map in O(N) traffic. At ~80 bytes per URL string, N = 1M distinct keys ≈ ~80 MB of heap per attacker-peer per hour bucket; N = 10M ≈ ~800 MB. The 2-hour `retain()` only ages out the prior hour; within the active hour, there is no cap on `subject_url` cardinality.

Contrast with the per-peer map at `inbox.rs:544-552`: per-peer DoS requires N distinct peer domains, which is naturally rate-limited by the inbound allowlist + signature checks (Gate 1). The per-actor map's surface is the inverse — a single peer can craft arbitrarily many subject URLs.

The fix is a single-site insertion-order eviction bound at the per-actor use site: when `counts.len() >= MAX_PER_ACTOR_RATE_ENTRIES` and the new key is not present, evict the entry whose `bucket` value is smallest (i.e. the oldest hour bucket; ties broken arbitrarily by iteration order) before inserting the new entry. The bound is `MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000` — large enough that a benign peer running a high-rate attestation workload does not hit the cap, small enough that the worst-case heap footprint of the per-actor map stays bounded at ~800 KB (10_000 × ~80 bytes/URL) per process.

The per-peer map is NOT touched (user clarify B1 2026-05-22). The per-actor map key stays as the raw `String` URL (NOT hashed; user clarify B3 2026-05-22). The TOCTOU eviction race on `inbox.rs:654` (`evict_oldest_unreviewed_if_needed`) is OUT of scope (deferred to v1-federation-inbound-e).

## 4. Solution statement

Add a single Rust `const MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000;` and a single bound-check branch at one site:

```rust
// publish_trust_attestation.rs:158-167 — current shape, post-fix-target
let exceeded_actor = {
  let mut counts = crate::governance::inbox::rate_per_actor_counts()
    .lock()
    .unwrap_or_else(std::sync::PoisonError::into_inner);
  // Opportunistic prune: drop buckets older than the previous hour.
  counts.retain(|(_, b), _| *b >= bucket - 1);

  // NEW — insertion-order bound (Task 1).
  let key = (subject_url.to_string(), bucket);
  if counts.len() >= MAX_PER_ACTOR_RATE_ENTRIES && !counts.contains_key(&key) {
    if let Some(oldest_key) = counts
      .iter()
      .min_by_key(|((_, b), _)| *b)
      .map(|(k, _)| k.clone())
    {
      counts.remove(&oldest_key);
    }
  }

  let entry = counts.entry(key).or_insert(0);
  *entry = entry.saturating_add(1);
  i64::from(*entry) > actor_cap
};
```

Properties of this shape (consumed by the §4 watchpoints + Task 1 brief):

1. **Same-lock scope:** the bound check + eviction + entry-or-insert all run under the same `MutexGuard`. No `.await` between `counts.lock()` (line 159-161) and the closing brace of the `let exceeded_actor = { ... }` block. Diesel reads happen outside the lock (line 139-153) before the lock is acquired.
2. **Eviction strategy:** O(N) `iter().min_by_key(|((_, b), _)| *b)` selects the entry whose `bucket` value is smallest (oldest hour). Within a tie, `min_by_key` returns the first iteration order — `HashMap` order is unspecified but stable enough for "evict ONE entry" semantics. Acceptable because (a) at the cap, all surviving entries are from the current or previous hour bucket (`retain` already pruned older ones), and (b) the policy is "best-effort eviction to keep the map bounded", not "evict the strictly-LRU entry".
3. **Re-insertion safety:** `if counts.len() >= MAX && !counts.contains_key(&key)` — only evict when the new key is new. If the attacker repeatedly hits the SAME `(subject_url, bucket)` key, no eviction fires and the rate limit (Gate 5) still catches it via the `saturating_add(1)` → `> actor_cap` check. The bound protects key-cardinality, not bucket-count growth.
4. **Constant location:** `const MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000;` lands at module scope at the top of `publish_trust_attestation.rs` (after the existing `use` block; before the `impl Activity for PublishTrustAttestation` block) — visible to both the `check_per_actor_rate_limit` body and the `#[cfg(test)] mod tests_per_actor_bound { use super::*; }` test module that lands in the same file.

Task 2 lands a `#[cfg(test)] mod tests_per_actor_bound { ... }` at the bottom of the same file (after the existing module body, after the file's final `}` closing the impl/fn list) exercising the bound directly. The test acquires the `rate_per_actor_counts()` lock, clears stale state, inserts `MAX_PER_ACTOR_RATE_ENTRIES + 1` distinct keys, asserts `len() == MAX_PER_ACTOR_RATE_ENTRIES`, asserts the first-inserted key is no longer present (evicted), and clears the map at test end.

The per-peer map at `inbox.rs:544-552` is NOT touched. The `subject_url: String` map key stays plaintext (no hashing). The `inbox.rs:654` TOCTOU function stays unchanged. No `governance_config` knob is added (the const is Rust-only). No new file. No migration.

## 5. Metadata

- **Phase:** `v1-federation-inbound-d`
- **Branch:** `phase-v1-federation-inbound-d` (cut by `bm-task` after User Gate 1)
- **Target impl-task model:** `sonnet-4-6` (Junior daemon default; per CLAUDE.md "Four-role model")
- **Estimated tasks:** 4 (Task 0 pre-flight + Task 1 bound + Task 2 unit test + Task 3 retro)
- **Estimated cargo budget:** N/A (validate-pending-laptop DoD — cargo runs on laptop, NOT EliteDesk; Shape G SUSPENDED per DQ #229 / `project_shape_g_suspended_2026_05_16` until 2026-06-01)
- **Forbidden-window applicability:** standard (per advisor-orchestrator.md table; non-binding for impl-task throughput since cargo runs on laptop, but binding for advisor-side §15 dry-run runs and the Phase-2 e2e regression check)
- **Complexity score:** **1/10** — see breakdown below

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Target = `sonnet-4-6` → split-DQ threshold is `> 8`.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 2 impl tasks (Task 1 bound + Task 2 unit test); well below threshold |
| Migrations touched | +2 each | 0 | Zero migrations (PRECON-4) |
| Crates touched | +1 each | 1 | Only `crates/apub/activities/` (Tasks 1+2 both edit the same file) |
| `crates/server/tests/e2e.rs` edits | +3 each | 0 | Brief PRECON post-clarify B1: Task 2 is a CRATE-INTERNAL unit test inside `publish_trust_attestation.rs`; no e2e edit |
| New ADR-affecting decisions | +2 each | 0 | None |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Validate-pending-laptop runs on laptop (Shape G suspended); budget non-binding |
| **Total** | — | **1/10** | Threshold `> 8` not crossed → no split-DQ |

### 5.2 Per-task complexity ceiling (non-Sonnet target only)

Not applicable — target is `sonnet-4-6`. Sonnet ceilings (`≤ 4` files / `≤ 2` crates) are well within each task's footprint:

- Task 1: `modifies: [publish_trust_attestation.rs]` — 1 file / 1 crate.
- Task 2: `modifies: [publish_trust_attestation.rs]` — 1 file / 1 crate.

Tasks 1+2 both modify the same file (sequential commits, not cohort-parallel). No `[P]` cohort; see §13 PRECON-6.

## 6. Relationship to other v1-federation-inbound-* sub-phases

- **Depends on:** `v1-federation-inbound-b` (merged at PR #139, governance-v0 commit `16ede83b6`) — defined the per-actor counter; this plan adds the bound on top.
- **Depends on:** `v1-federation-inbound-c` (merged into governance-v0 most recently) — the reader-side `.order_by(valid_from.desc())` fix has shipped; the `actor_cap` read at `publish_trust_attestation.rs:142-152` is now correct. The plan does not re-touch the reader chain.
- **Carries forward to:** `v1-federation-inbound-e` (likely next) — TOCTOU eviction fix on `inbox.rs:654`'s `evict_oldest_unreviewed_if_needed`. v1-fed-in-d retro proposes the gate-1 question (atomic-SQL-with-RETURNING vs `SELECT FOR UPDATE SKIP LOCKED`).
- **Out-of-scope siblings (mentioned, not in-scope):** per-peer rate-map bound, SHA-256 key-hash, `governance_config`-backed `MAX_PER_ACTOR_RATE_ENTRIES`, Postgres-backed rate counters — see §12 enumeration.

## 7. Preflight guardrails inherited from prior phases

- **R1:** every `i32 ↔ i64` comparison uses `i64::from(...)`, never `as` cast (per `feedback_clippy_test_style.md`). The new code uses `i64::from(*entry) > actor_cap` (pre-existing line 166; unchanged) and `usize` comparisons for the bound (`counts.len() >= MAX_PER_ACTOR_RATE_ENTRIES`) — no new cast surface.
- **R5:** Task 0 enumerates ALL probes explicitly (Probes 0-8 per `.claude/rules/pre-phase-harness-audit.md`); do NOT inherit implicitly.
- **R6:** all clippy invocations use `--workspace --features full --no-deps -- -D warnings` uniformly (per `feedback_clippy_test_style.md`).
- **R7:** Task 2 (`#[cfg(test)] mod tests_per_actor_bound`) introduces a new test target inside `lemmy_apub_activities`'s lib — `cargo test -p lemmy_apub_activities --lib` is the test-target compile + run gate; the workspace `cargo check` covers compile, and the workspace `cargo test --test e2e --no-run` (R7) verifies e2e harness still links after Task 2.
- **R-laptop-cargo:** all cargo invocations run on the laptop (`validate-pending-laptop` DoD per `.claude/rules/advisor-orchestrator.md` §5.2); never on the EliteDesk daemon (Shape G suspended per PMD `project_shape_g_suspended_2026_05_16`, DQ #229 pending re-enable 2026-06-01).
- **R-windows-e2e:** Phase-2 e2e regression check MUST use the bat wrapper invocation per `feedback_windows_e2e_requires_bat_wrapper.md`. Never bare `cargo test` (libpq.dll path); never `-p lemmy_server --features full` (no `full` feature on `lemmy_server`).
- **R-no-cohort:** Tasks 1+2 both modify `publish_trust_attestation.rs` — NO `[P]` markers; serial dispatch only (per PRECON-6 + `feedback_explicit_file_arrays_on_tasks.md` YAML overlap check).
- **R-no-DQ-pre-reserve:** PRECON-6 means no `[P]` cohort; advisor does NOT pre-reserve DQ ids per `feedback_cohort_dq_id_collision.md` Option 3 (that mitigation only applies to `[P]` cohorts).

## 8. Flow design

**Before Task 1 (the defect):** an attacker peer P (already on the allowlist) sends N distinct `Create(TrustAttestation)` activities to our inbox, each carrying a unique `object.rest["subject"]` URL `S_i`. For each:

```
wrap_governance_inbound
  → Gate 1 (peer trust)        — PASS (P is allowlisted)
  → Gate 2 (size cap)          — PASS (small payload)
  → Gate 3 (deny_unknown)      — PASS (well-formed)
  → Gate 4 (per-peer rate)     — PASS while P stays under cap=100/hour (attacker paces)
  → inner.receive
    → PublishTrustAttestation::receive
      → check_per_actor_rate_limit
        → counts.lock()
        → counts.retain(...)              — drops prior-hour entries only
        → counts.entry((S_i, bucket))     — INSERTS new key, no bound check
        → *entry = saturating_add(1)      — 1 (new key); never trips Gate 5
        → drop lock
      → Ok(())
    → receive_remote_trust_attestation    — persists row
```

At N = 10^6 the map carries 10^6 `(String, i64)` keys; at ~80 bytes per URL string the heap footprint is ~80 MB of attacker-controlled data, scaling linearly with attacker traffic up to the per-peer rate cap.

**After Task 1 (the fix):** same flow, with one new branch inside the lock:

```
        → counts.lock()
        → counts.retain(...)              — drops prior-hour entries only
        → key = (S_i.to_string(), bucket)
        → IF counts.len() >= 10_000 && !counts.contains_key(&key):
        →   find oldest_key by min bucket
        →   counts.remove(&oldest_key)
        → counts.entry(key).or_insert(0)
        → *entry = saturating_add(1)
        → drop lock
```

The map's worst-case `len()` is now `MAX_PER_ACTOR_RATE_ENTRIES = 10_000`. Heap footprint stays bounded at ~800 KB (10_000 × ~80 bytes/URL) per process regardless of attacker key cardinality.

Task 2's unit test exercises Steps "len() >= MAX && !contains_key" + "remove(oldest_key)" + "entry(key).or_insert(0)" directly against `rate_per_actor_counts()` in the apub-activities crate's test binary — no Postgres, no testcontainers, no inbox plumbing. Task 2 does NOT exercise the bound through the live inbox path; that broader regression check is the Phase-2 full-e2e run (§15.4) confirming Task 1 doesn't break the existing per-actor rate path (`per_actor_rate_limit_returns_429`-style siblings inside `mod v1_federation_inbound_b_fixtures`).

## 9. Mandatory reading

The impl-task subagent MUST `Read` each of these before its first edit. Cite each in plan §10.

### Schema / type definitions

- **`crates/apub/activities/src/governance/inbox.rs:464-479`** — map definitions (`rate_per_peer_counts`, `rate_per_actor_counts`, `current_hour_bucket`). Confirms shape `Mutex<HashMap<(String, i64), u32>>` and key-tuple `(identifier_string, hour_bucket_i64)`. **READ-ONLY — no edit.**

### Existing patterns (the MIRROR refs §13 tasks point at)

- **`crates/apub/activities/src/governance/inbox.rs:540-565`** — the per-peer use site. **MIRROR ref for Task 1** — shows the canonical lock+retain+entry+saturating_add pattern. The new bound check sits between `retain()` (line 548) and `entry()` (line 549); preserve same-lock scope.
- **`crates/apub/activities/src/governance/publish_trust_attestation.rs:118-187`** — the per-actor use site (the Task 1 fix site). Read fully so the impl agent has the full `check_per_actor_rate_limit` body in context; the bound branch lands inside the existing `let exceeded_actor = { ... }` block at lines 158-167.
- **`crates/apub/activities/src/governance/publish_sanction_notice.rs:612-720`** — the crate-internal `#[cfg(test)] mod tests` sibling. **MIRROR ref for Task 2** — shows the test-module shape inside the apub-activities crate: `use super::...;`, fixture helpers, `#[test] fn ... -> LemmyResult<()>` signature with bare `?` propagation, no `.unwrap()` / `.expect()` in test bodies (workspace lint compliance per `feedback_clippy_test_style.md`).

### Adjacent test fixtures (so impl doesn't re-invent helpers)

- **`crates/apub/activities/src/governance/publish_sanction_notice.rs:613-635`** — module-level docstring + `use super::...` block + the `fixture_person` helper. Task 2's `mod tests_per_actor_bound` mirrors the surface: `//!` doctring stating "Pure-function tests for the per-actor rate-map insertion-order bound added per v1-federation-inbound-d", `use super::*` (or specific imports for `MAX_PER_ACTOR_RATE_ENTRIES` + `crate::governance::inbox::rate_per_actor_counts`), no `.unwrap()` outside structured test assertions.

### Lessons (each gates a §13 task; cite in §3 of each impl-task brief at advisor-side dispatch)

- `.claude/lessons/feedback_clippy_test_style.md` — workspace lint discipline (`-D warnings`); test bodies use `?` propagation + `LemmyResult<()>` return shape; no bare `.unwrap()` / `.expect()` shotguns.
- `.claude/lessons/feedback_features_full_workspace_only.md` + `feedback_features_full_p_crate_incompatible.md` — clippy must use `--workspace --features full --no-deps`; never `-p <crate>` + `--features full`.
- `.claude/lessons/feedback_advisor_watchpoint_specificity.md` — §4 watchpoints cite file:line.
- `.claude/lessons/feedback_complexity_score_pre_split.md` — §5 score computation.
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` — post-bm-cut lane worktree `brehon-fork-fed-in-d`.
- `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md` — Phase-2 e2e invocation discipline (regression-gate only; not Task 2's runner).
- `.claude/lessons/feedback_brehon_verify_pre_merge.md` — §16a stories.
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — FILES YAML disjointness check + no `[P]` here.
- `.claude/lessons/feedback_principles_not_rules.md` — §G4 allowlist is conservative-by-design (§4.5 acknowledges the anticipated fail modes are NOT auto-fixable).
- `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — read sibling `mod tests` before authoring Task 2.

## 10. Patterns to mirror

### 10.1 Canonical lock+retain+bound+entry shape — `publish_trust_attestation.rs::check_per_actor_rate_limit`

**Mirror:** `crates/apub/activities/src/governance/inbox.rs:540-565` (the per-peer use site — same `Mutex<HashMap<(String, i64), u32>>` shape, same `retain()` two-hour prune, same `entry().or_insert(0)` access pattern). Task 1 introduces the bound check ON TOP of this pattern at the per-actor site; the per-peer site stays unchanged (per user clarify B1).

Verbatim shape (with comment showing how the bound branch lands inside the same-lock block — the Task 1 impl-task brief at advisor-side dispatch quotes this block):

```rust
// crates/apub/activities/src/governance/publish_trust_attestation.rs:158-167 — post-Task-1
let bucket = crate::governance::inbox::current_hour_bucket();
let exceeded_actor = {
  let mut counts = crate::governance::inbox::rate_per_actor_counts()
    .lock()
    .unwrap_or_else(std::sync::PoisonError::into_inner);
  // Opportunistic prune: drop buckets older than the previous hour.
  counts.retain(|(_, b), _| *b >= bucket - 1);

  // Bound the per-actor map: a single allowlisted peer can craft arbitrarily
  // many subject URLs within a single hour bucket. retain() above only
  // drops prior-hour entries; within the active bucket, distinct subject_url
  // values would grow the map unboundedly. Insertion-order eviction (by
  // smallest bucket; ties broken by HashMap iteration order) keeps the map
  // at len() <= MAX_PER_ACTOR_RATE_ENTRIES regardless of attacker key
  // cardinality. See v1-federation-inbound-d plan §3/§4.
  let key = (subject_url.to_string(), bucket);
  if counts.len() >= MAX_PER_ACTOR_RATE_ENTRIES && !counts.contains_key(&key) {
    if let Some(oldest_key) = counts
      .iter()
      .min_by_key(|((_, b), _)| *b)
      .map(|(k, _)| k.clone())
    {
      counts.remove(&oldest_key);
    }
  }

  let entry = counts.entry(key).or_insert(0);
  *entry = entry.saturating_add(1);
  i64::from(*entry) > actor_cap
};
```

**Apply (Task 1):**

1. Add a module-scope `const MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000;` near the top of `publish_trust_attestation.rs` (after the `use` block at line 30, before the `impl Activity for PublishTrustAttestation` at line 33). Doc-comment one-liner: `/// Maximum number of distinct (subject_url, hour_bucket) keys held in the per-actor rate-limit map at any moment; see v1-federation-inbound-d plan §3.`
2. Inside the `let exceeded_actor = { ... }` block at lines 158-167, between `counts.retain(...)` (line 163) and `let entry = counts.entry(...).or_insert(0);` (line 164), insert the four-line bound-check + eviction branch verbatim per the shape above. Preserve 6-space indentation (the block is nested inside `check_per_actor_rate_limit`'s body). Move the `let entry = ...` line to use the local `let key = ...` binding so the contains-check and the entry-or-insert share the same key.
3. Do NOT release the lock between `retain()`, the bound check, and `entry()`. Do NOT `.await` anywhere between `counts.lock()` (line 159-161) and the end of the block. Diesel reads happen earlier at lines 139-153, outside the lock.

**GOTCHAs (apply to Task 1):**

- Do **NOT** add the bound at the map DEFINITION site (`inbox.rs:471-474`). The per-peer map at the adjacent definition (`inbox.rs:466-468`) does NOT get the same bound per user clarify B1; localising the bound at the use site preserves the "different maps, different policies" invariant.
- Do **NOT** read `MAX_PER_ACTOR_RATE_ENTRIES` from `governance_config`. The const is Rust-only (out of scope per brief PRECON-1 + PRECON-4; option-a deferred).
- Do **NOT** hash the `subject_url` String to a fixed-size key (out of scope per user clarify B3; `subject_url: String` stays plaintext for debug-log lookup).
- Do **NOT** edit the `rate_per_peer_counts` definition or use site at `inbox.rs:466-468` + `inbox.rs:540-565` (per user clarify B1).
- Do **NOT** edit the TOCTOU eviction function at `inbox.rs:654` (`evict_oldest_unreviewed_if_needed`) — deferred to v1-federation-inbound-e.
- The eviction strategy uses `iter().min_by_key(|((_, b), _)| *b)` — O(N) at the cap. At N = 10_000 this is a 10k-key linear scan once per insert that hits the cap; benign workload (well under the cap) never enters this branch. Acceptable per `feedback_principles_not_rules.md`: simplest correct shape first; optimise only with retro evidence.
- The eviction is **best-effort** when ties exist (multiple keys share the same `bucket`). `HashMap` iteration order is unspecified, so `min_by_key` returns "any one of the smallest-bucket keys" on ties — acceptable because the policy is "keep map bounded", not "evict the strictly oldest entry".
- The `key.clone()` after `min_by_key().map(...)` is intentional — `counts.remove(&oldest_key)` needs `&(String, i64)` borrowed from a value, not a borrow into the map (which would conflict with the `&mut counts` later). Cloning a `(String, i64)` at the cap (~80 bytes) is cheap.

### 10.2 Crate-internal `#[cfg(test)] mod tests` — sibling at `publish_sanction_notice.rs:612-720`

**Mirror:** `crates/apub/activities/src/governance/publish_sanction_notice.rs:612-666` — the existing `#[cfg(test)] mod tests` block in the apub-activities crate. **READ this range verbatim before authoring Task 2's test module.**

Shape (skeleton — the impl-task brief at advisor-side dispatch quotes the sibling docstring + `use super::...` + fixture helper structure):

```rust
// crates/apub/activities/src/governance/publish_sanction_notice.rs:612-666
#[cfg(test)]
mod tests {
  //! Pure-function tests for the two builder-time invariant guards added
  //! per CodeRabbit PR #46 #2p-2 (reject remote actor) and #2p-3 (enforce
  //! federated scope). The full `build_local_sanction_notice_plan` needs
  //! a live AsyncPgConnection ...
  //!
  //! No unwrap/expect per workspace lints — tests return LemmyResult.
  use super::{ApubPerson, SanctionScope, assert_actor_is_local, assert_scope_is_federated};
  // ...
  use lemmy_utils::error::LemmyResult;
  // ...
  #[test]
  fn assert_actor_is_local_accepts_local() -> LemmyResult<()> {
    let actor = fixture_person(true)?;
    assert_actor_is_local(&actor)?;
    Ok(())
  }
  // ...
}
```

**Apply (Task 2 — new `#[cfg(test)] mod tests_per_actor_bound` at the bottom of `publish_trust_attestation.rs`):**

- Module placement: at the bottom of `crates/apub/activities/src/governance/publish_trust_attestation.rs`, after the file's final non-test item (the last function or `impl` block). The pre-fix file is ~510 lines; the new mod appends ~80-100 lines.
- Module name: `tests_per_actor_bound` (descriptive; distinguishes from a future generic `mod tests`).
- Module docstring (`//!`): one paragraph stating "Pure-function tests for the per-actor rate-map insertion-order bound added per v1-federation-inbound-d plan §3. Exercises `MAX_PER_ACTOR_RATE_ENTRIES` cap behaviour against the live `rate_per_actor_counts()` `OnceLock` — clears the global at test start AND end so test order is not load-bearing across the apub-activities lib-test binary."
- Imports: `use super::MAX_PER_ACTOR_RATE_ENTRIES; use crate::governance::inbox::{rate_per_actor_counts, current_hour_bucket}; use std::sync::PoisonError;`
- Test fn signature: `#[test] fn per_actor_map_evicts_oldest_when_cap_reached() -> Result<(), Box<dyn std::error::Error>>` OR `#[test] fn per_actor_map_evicts_oldest_when_cap_reached()`.
  - **Decision (planner-recommended):** the test does NOT propagate errors via `?` — every step is either a direct call against `HashMap` / `Mutex` / `usize` operations or a `PoisonError::into_inner` recovery. Use the simpler `#[test] fn ... ()` signature with no return type. Document the choice in the impl brief; if the impl agent encounters an `?` need at sibling-read time, switch to `LemmyResult<()>` mirroring the canonical sibling at `publish_sanction_notice.rs:613-666`.
- Test body skeleton (the impl agent picks the exact identifier `i` and adjusts whitespace to match local conventions):

```rust
#[test]
fn per_actor_map_evicts_oldest_when_cap_reached() {
  // Clear stale state from prior tests (the OnceLock is process-global).
  {
    let mut counts = rate_per_actor_counts()
      .lock()
      .unwrap_or_else(PoisonError::into_inner);
    counts.clear();
  }

  let bucket = current_hour_bucket();
  let cap = MAX_PER_ACTOR_RATE_ENTRIES;

  // Insert `cap + 1` distinct keys, applying the same bound logic that
  // ships in publish_trust_attestation.rs:158-167 (Task 1).
  for i in 0..=cap {
    let key = (format!("https://test/{i}"), bucket);
    let mut counts = rate_per_actor_counts()
      .lock()
      .unwrap_or_else(PoisonError::into_inner);
    counts.retain(|(_, b), _| *b >= bucket - 1);
    if counts.len() >= cap && !counts.contains_key(&key) {
      if let Some(oldest_key) = counts
        .iter()
        .min_by_key(|((_, b), _)| *b)
        .map(|(k, _)| k.clone())
      {
        counts.remove(&oldest_key);
      }
    }
    let entry = counts.entry(key).or_insert(0);
    *entry = entry.saturating_add(1);
  }

  // Assert: map size capped at MAX_PER_ACTOR_RATE_ENTRIES.
  let counts = rate_per_actor_counts()
    .lock()
    .unwrap_or_else(PoisonError::into_inner);
  assert_eq!(
    counts.len(),
    cap,
    "per-actor map must be bounded at MAX_PER_ACTOR_RATE_ENTRIES after cap + 1 inserts",
  );

  // Assert: the FIRST inserted key (i=0) has been evicted.
  let first_key = (String::from("https://test/0"), bucket);
  assert!(
    !counts.contains_key(&first_key),
    "oldest inserted key (i=0) must be evicted by the bound",
  );

  // Cleanup: clear the map so other tests in this binary start fresh.
  drop(counts);
  let mut counts = rate_per_actor_counts()
    .lock()
    .unwrap_or_else(PoisonError::into_inner);
  counts.clear();
}
```

**GOTCHAs (apply to Task 2):**

- The `OnceLock<Mutex<HashMap<...>>>` at `inbox.rs:471-474` is **process-global** within the apub-activities lib-test binary. Multiple `#[test]` fns in the same crate share it. Always `clear()` at test start AND end so test order is not load-bearing. Cargo test default is parallel; `Mutex` serializes access but does NOT clear residual state between tests.
- **Do NOT use `#[serial]` / `serial_test` crate dependencies** — adding a test-only dep widens scope beyond the brief. The `clear()` + `Mutex` discipline above is sufficient if every per-actor-bound test clears at both ends.
- **Bound recipe MUST match Task 1 verbatim.** If Task 1's impl agent picks a different eviction strategy (e.g. `BTreeMap` derivation, or `iter().next()` for arbitrary-order rather than oldest-bucket selection), Task 2's body MUST follow the same shape so the test exercises what shipped. The impl brief for Task 2 (authored post-Task-1 finalize-merge) reads `publish_trust_attestation.rs` at Task-1 HEAD and mirrors the exact branch verbatim before writing the test.
- **Test fn name discipline:** the planner does NOT lock the exact identifier. Recommended candidates: `per_actor_map_evicts_oldest_when_cap_reached`, `bound_keeps_per_actor_map_at_max_entries`, `per_actor_rate_map_insertion_order_eviction`. The impl agent at canonical-schema-first read time picks one matching the sibling convention (`<scenario>_<assertion>` snake_case) and updates the assertion message strings accordingly.
- **No `.unwrap()` / `.expect()` shotguns** per `feedback_clippy_test_style.md`. `PoisonError::into_inner` recovery is the canonical Mutex-poison fallback already used at `publish_trust_attestation.rs:161` + `inbox.rs:547`. `assert_eq!` / `assert!` are fine (workspace lints permit them in `#[cfg(test)]` blocks).
- **No `--features full` interaction:** `lemmy_apub_activities` has `[features] full = []` (empty — Cargo.toml line 19) and its source has zero `#[cfg(feature = "full")]` gates. `cargo test -p lemmy_apub_activities --lib` compiles the crate without `--features full`. Wrapper invocation: `bash scripts/brehon/cargo-test.sh -p lemmy_apub_activities --lib` (laptop alternative: `cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_apub_activities --lib"`).

## 11. Files to change

Grouped by crate. Each entry: path + one-line purpose + which §13 task(s) write it. The planner verifies each path exists in the workspace at `governance-v0` HEAD `7e6c4202f`.

**`crates/apub/activities/`** (federation activity handlers — governance subdir):

- `crates/apub/activities/src/governance/publish_trust_attestation.rs` — (Task 1) add `const MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000;` at module scope; add the four-line bound-check + eviction branch inside the existing `let exceeded_actor = { ... }` block at lines 158-167 between `counts.retain(...)` and `let entry = counts.entry(...).or_insert(0);`. (Task 2) append a new `#[cfg(test)] mod tests_per_actor_bound { ... }` at the bottom of the file (~80 lines of test code).

**No other crate edits.** No new file under `crates/apub/activities/src/governance/`. No edit to `inbox.rs`, no edit to `publish_sanction_notice.rs`, no edit to any `crates/server/tests/` file, no edit to `crates/api/api/src/governance/`, no edit to `crates/db_schema/src/source/governance/`. The unit test lives crate-internal (per user clarify B1, the e2e route is infeasible because `rate_per_actor_counts` is `pub(crate)` to `lemmy_apub_activities` and cannot be called from `crates/server/tests/e2e.rs`, AND sending 10_001 inbound activities through the live inbox path exceeds per-test budget).

**No struct-field add:** none of Tasks 1-2 add or modify a public struct. The "Struct-field add: enumerate all callsites" discipline (per `feedback_planner_enumerate_struct_callsites_for_addfield.md`) does not fire for this plan; no `rg "<Type>" crates/ tests/` enumeration required.

**No migrations** (PRECON-4). The fix is pure in-memory data-structure logic.

## 12. NOT building in v1-federation-inbound-d

Per brief §0.2 + user clarify-gate answers 2026-05-22. Each entry pairs a "tempting addition" with a deferral pointer; the planner refuses to in-scope any of these mid-impl.

1. **Per-peer rate-map bound (`inbox.rs:466-468` definition + `inbox.rs:544-552` use site)** — user B1 2026-05-22; existing 2-hour `retain()` + upstream allowlist + signature checks gate domain-distinct flooding upstream. Trigger: post-pilot DoS metrics show per-peer surface needs hardening.
2. **SHA-256 key-hash for per-actor map (`subject_url` → 32-byte hash)** — user B3 2026-05-22; per-entry heap savings (~800 KB at 10k entries × ~80B URL) not worth losing debuggability (`subject_url` plaintext in logs). Trigger: post-pilot metrics show per-entry heap is load-bearing.
3. **TOCTOU eviction fix on `inbox.rs:654` (`evict_oldest_unreviewed_if_needed`)** — DB-level concurrency design (atomic-SQL-with-RETURNING vs `SELECT FOR UPDATE SKIP LOCKED`) warrants its own plan + gate-1 question. Trigger: v1-federation-inbound-e (retro proposes the gate-1 question).
4. **`governance_config`-backed `MAX_PER_ACTOR_RATE_ENTRIES`** — option-a scope (config-driven cap); Rust `const` is the v0 approach. Trigger: post-pilot config-driven tuning.
5. **Postgres-backed rate counters (replace in-memory `OnceLock<Mutex<HashMap>>`)** — option-a scope; adds DB write per inbound activity, complicates failover semantics. Trigger: post-pilot scale analysis.
6. **`[P]` cohort dispatch** — Tasks 1+2 both modify `publish_trust_attestation.rs`; cohort logic does not apply. Trigger: never (architectural — single-file fix).
7. **Helper extraction across `inbox.rs` + `publish_trust_attestation.rs`** — fed-in-c PRECON-3 stands (circular-dep avoidance `lemmy_api → lemmy_apub → lemmy_apub_activities`). Trigger: never (architectural — duplication-on-purpose).
8. **New migration under `migrations/**` or `crates/db_schema/migrations/**`** — scope violation per PRECON-4. Catch-fire if proposed.
9. **Shape G workflow change** — SUSPENDED per DQ #229 + PMD `project_shape_g_suspended_2026_05_16`. Trigger: 2026-06-01 re-enable check.
10. **`[serial_test]` crate dependency for Task 2** — adding a test-only dep widens scope beyond the brief; the `clear()` + `Mutex`-poison-recovery discipline in §10.2 is sufficient. Trigger: if process-global state interference makes the lib-test binary flaky in a future sub-phase, evaluate adding `serial_test` or a `RAII` test-isolation guard.

---

## 13. Step-by-step tasks

Execute in dependency order. One commit per task (per `feedback_pr_per_phase.md` + `.claude/rules/branch-manager.md` "Junior finalize-merge: one commit per task"). No `[P]` markers — Tasks 1+2 both modify `publish_trust_attestation.rs` (overlapping `modifies:` arrays per `feedback_explicit_file_arrays_on_tasks.md` cohort check; serial dispatch only).

> **No cohort dispatch:** per brief PRECON-6 + the YAML overlap rule in `.claude/rules/advisor-orchestrator.md` §4.1 step 4. Advisor dispatches Task 1, waits for finalize-merge into `phase-v1-federation-inbound-d`, then dispatches Task 2, then Task 3 (retro). No `[P]` cohort DQ pre-reservation needed.
>
> **Validate-pending-laptop DoD** (per brief PRECON-5 + Shape-G suspended per DQ #229): each impl-task pushes its worker branch and writes `kind: "validate-pending-laptop"` to `.claude/decision-queue.json` naming §15 commands verbatim. Advisor reads on next poll, runs commands locally on the canonical laptop checkout, mutates the entry (`answered_by: "advisor-laptop"`). Cargo never runs on the EliteDesk Junior daemon (per `.claude/rules/advisor-orchestrator.md` §5.2 "Cargo never runs on the EliteDesk worker").

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for `v1-federation-inbound-d`; confirm branch is `phase-v1-federation-inbound-d`; confirm fed-in-c deliverables are intact on the phase-branch base; confirm pre-existing clippy baseline is clean against the §15 invocations.

**FILES:**

```yaml
creates: []
modifies: []
requires: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes explicitly).** Run on the **canonical laptop checkout** (`C:/Users/barri/Developer/brehon-fork-fed-in-d` worktree, post-bm-cut) with the Windows wrapper invocations:

```bash
# Probe 0 - Docker daemon (required for Phase-2 e2e regression check)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 - per-crate check honors -p
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/PRPs/debug/v1-federation-inbound-d-task0-probe1.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-d-task0-probe1.log

# Probe 2 - feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/PRPs/debug/v1-federation-inbound-d-task0-probe2.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-d-task0-probe2.log

# Probe 3 - cargo-test wrapper honors target selection
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-federation-inbound-d-task0-probe3.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-d-task0-probe3.log

# Probe 4 - exit-code propagation on bogus feature (negative probe)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-federation-inbound-d-task0-probe4a.log 2>&1"
echo "cargo-test.bat exit on bogus feature: $?"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-federation-inbound-d-task0-probe4b.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"

# Probe 5 - branch verification
git branch --show-current
# EXPECT: phase-v1-federation-inbound-d
git log governance-v0..HEAD --oneline | wc -l
# EXPECT: 0 (phase branch just cut; no commits ahead of trunk yet)

# Probe 6 - fed-in-c deliverables intact on the base
grep -n "order_by(governance_config::valid_from.desc())" crates/apub/activities/src/governance/inbox.rs
# EXPECT: line ~427 (single match inside get_inbound_config_int)
grep -n "order_by(governance_config::valid_from.desc())" crates/apub/activities/src/governance/publish_trust_attestation.rs
# EXPECT: line ~146 (single match inside actor_cap block)
grep -n "pub(crate) fn rate_per_actor_counts" crates/apub/activities/src/governance/inbox.rs
# EXPECT: line ~471 (per-actor map definition)
grep -n "fn check_per_actor_rate_limit" crates/apub/activities/src/governance/publish_trust_attestation.rs
# EXPECT: line ~118 (per-actor rate-limit use site)
grep -n "counts.retain(|(_, b), _| \*b >= bucket - 1);" crates/apub/activities/src/governance/publish_trust_attestation.rs
# EXPECT: line ~163 (the prune line; bound branch lands AFTER this)

# Probe 7 - sibling test module location for Task 2 MIRROR ref
grep -n "^#\[cfg(test)\]" crates/apub/activities/src/governance/publish_sanction_notice.rs
# EXPECT: line 612 (the crate-internal test sibling Task 2 mirrors)
grep -n "mod tests" crates/apub/activities/src/governance/publish_sanction_notice.rs
# EXPECT: line 613 (mod tests opening)

# Probe 8 - workspace clippy baseline against the §15.2 invocation
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-d-task0-clippy-baseline.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-federation-inbound-d-task0-clippy-baseline.log
```

**EXPECT block (overall):**

- Probes 0..3, 5..8 exit 0.
- Probe 4 (both lines) exits NON-ZERO (negative test confirms exit-code propagation per `feedback_wrapper_script_flag_silence.md`).
- Probe 6 outputs all five grep matches; line numbers may drift +/-2 lines from the stated targets but the matched lines must be present.
- Probe 7 outputs two lines confirming the sibling `#[cfg(test)] mod tests` block at `publish_sanction_notice.rs:612-613`.

**No commit at Task 0** — this is verification only. If any probe fails, file a `kind: "blocker"` DQ (`from: "advisor"`, since Task 0 is run advisor-side as part of plan approval per §15.5 below + `feedback_pre_phase_dod_smoke_test.md`) and STOP. Do NOT dispatch Task 1 with a red probe.

### Task 1: Per-actor rate-map bound — add `MAX_PER_ACTOR_RATE_ENTRIES` const + insertion-order eviction branch

**ACTION:** add a `const MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000;` at module scope of `crates/apub/activities/src/governance/publish_trust_attestation.rs`, and insert a four-line bound-check + eviction branch inside the existing `let exceeded_actor = { ... }` block at lines 158-167 between `counts.retain(...)` and `counts.entry(...).or_insert(0)`. No other line changes.

**FILES (machine-parseable, used by `/brehon-verify`):**

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/publish_trust_attestation.rs
requires: []
```

**IMPLEMENT (file 1 of 1):** in `crates/apub/activities/src/governance/publish_trust_attestation.rs`:

1. **Const declaration.** After the `use` block (currently ends at line 30) and before the `#[async_trait::async_trait]` attribute on `impl Activity for PublishTrustAttestation` (currently at line 32), insert a new module-scope const:

   ```rust
   /// Maximum number of distinct (subject_url, hour_bucket) keys held in the
   /// per-actor rate-limit map at any moment. A single allowlisted peer can
   /// craft arbitrarily many subject URLs within one hour bucket; the
   /// insertion-order eviction at `check_per_actor_rate_limit` keeps the map
   /// bounded regardless of attacker key cardinality. See v1-federation-inbound-d
   /// plan §3.
   const MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000;
   ```

2. **Bound branch inside the same-lock block.** Locate the existing `let exceeded_actor = { ... }` block (currently lines 158-167). Inside the block, between line 163 (`counts.retain(|(_, b), _| *b >= bucket - 1);`) and line 164 (`let entry = counts.entry((subject_url.to_string(), bucket)).or_insert(0);`), insert the bound-check + eviction branch. Refactor the `let entry = ...` line to use a local `let key = (subject_url.to_string(), bucket);` binding so the `contains_key(&key)` check and the `entry(key)` insert share the same value. Post-edit, the block reads:

   ```rust
   let exceeded_actor = {
     let mut counts = crate::governance::inbox::rate_per_actor_counts()
       .lock()
       .unwrap_or_else(std::sync::PoisonError::into_inner);
     // Opportunistic prune: drop buckets older than the previous hour.
     counts.retain(|(_, b), _| *b >= bucket - 1);

     // Insertion-order bound: cap distinct (subject_url, bucket) keys at
     // MAX_PER_ACTOR_RATE_ENTRIES. A single allowlisted peer can craft
     // arbitrarily many subject_url values within one hour bucket; retain()
     // above only drops prior-hour entries. Without this bound the map grows
     // O(attacker key cardinality). See v1-federation-inbound-d plan §3.
     let key = (subject_url.to_string(), bucket);
     if counts.len() >= MAX_PER_ACTOR_RATE_ENTRIES && !counts.contains_key(&key) {
       if let Some(oldest_key) = counts
         .iter()
         .min_by_key(|((_, b), _)| *b)
         .map(|(k, _)| k.clone())
       {
         counts.remove(&oldest_key);
       }
     }

     let entry = counts.entry(key).or_insert(0);
     *entry = entry.saturating_add(1);
     i64::from(*entry) > actor_cap
   };
   ```

   Do **NOT** edit `counts.retain(...)` (preserve the existing 2-hour prune).
   Do **NOT** add an `.await` anywhere inside the block.
   Do **NOT** edit the Diesel reader at lines 139-153 (the `actor_cap` read; fed-in-c already shipped the `.order_by(valid_from.desc())` fix at line 146).
   Do **NOT** edit the rate-counter Gate-5 check at lines 169-184 (the `if exceeded_actor` branch).

**MIRROR:** `crates/apub/activities/src/governance/inbox.rs:540-565` (the per-peer use site) — canonical lock+retain+entry pattern; Task 1 introduces the bound check ON TOP without disturbing the pattern.

**GOTCHA (apply to Task 1):**

- The bound lands at the USE site, NOT at the map DEFINITION (`inbox.rs:471-474`). The per-peer map at the adjacent definition (`inbox.rs:466-468`) does NOT get the same bound per user clarify B1; localising the bound at the per-actor use site preserves the "different maps, different policies" invariant.
- The const is Rust-only — do NOT read from `governance_config` (out of scope per brief PRECON-1 + PRECON-4).
- Do NOT hash the `subject_url` String (out of scope per user clarify B3).
- Do NOT touch `rate_per_peer_counts` or its use site at `inbox.rs:544-552` (per user clarify B1).
- The `key.clone()` after `min_by_key().map(...)` is intentional — `counts.remove(&oldest_key)` needs an owned `(String, i64)` to avoid a borrow conflict with the surrounding `&mut counts`.
- The eviction strategy is O(N) at the cap. At N = 10_000 this is a 10k-entry linear scan once per insert that hits the cap. Acceptable per `feedback_principles_not_rules.md` (simplest correct shape first).
- **Conformance-audit:** Task 1's target file is `crates/apub/activities/src/governance/publish_trust_attestation.rs` — Tier-1 audit scope per `.claude/skills/brehon-conformance-audit/`. Before authoring the Task-1 impl-task brief, the advisor MUST invoke the skill with `target_scope = file crates/apub/activities/src/governance/publish_trust_attestation.rs` (per `.claude/rules/advisor-orchestrator.md` §3.1.1). Tier-1 findings fold into the brief §3/§4 before clarify-DQ.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

Worker pre-push (Junior `impl-task` worktree):

```bash
git diff --stat
# EXPECT: 1 file changed, ~25 insertions(+), 1 deletion(-) (the `let entry = ...` line is refactored)
git add crates/apub/activities/src/governance/publish_trust_attestation.rs
git commit -m "feat(fed-in-d): bound per-actor rate-map at MAX_PER_ACTOR_RATE_ENTRIES (task 1)"
git push origin <worker-branch>
# Worker then writes kind: "validate-pending-laptop" DQ entry with §15 commands verbatim.
```

Post-push (advisor-laptop, on the canonical laptop checkout, per `.claude/rules/advisor-orchestrator.md` §5.2):

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-d-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-d-task1-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-d-task1-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-d-task1-clippy.log
# EXPECT: exit 0
```

Advisor mutates the `validate-pending-laptop` DQ entry per `.claude/rules/advisor-orchestrator.md` §5.2 (`answered_by: "advisor-laptop"`, `result: "pass"`, move to `resolved[]`) on success.

### Task 2: Crate-internal unit test for the per-actor bound

**ACTION:** append a `#[cfg(test)] mod tests_per_actor_bound { ... }` to the bottom of `crates/apub/activities/src/governance/publish_trust_attestation.rs` exercising the Task-1 bound. The test inserts `MAX_PER_ACTOR_RATE_ENTRIES + 1` distinct `(subject_url, bucket)` keys against the live `rate_per_actor_counts()` `OnceLock`, asserts `len() == MAX_PER_ACTOR_RATE_ENTRIES` post-loop, asserts the first-inserted key is no longer present (evicted), and clears the map at test start AND end so test order is not load-bearing.

**FILES:**

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/publish_trust_attestation.rs
requires:
  - task: 1
    reason: "Task 2 imports MAX_PER_ACTOR_RATE_ENTRIES from the same file and mirrors the bound logic Task 1 ships at lines 158-167. The test exercises what shipped; Task 1 must be merged into phase-v1-federation-inbound-d before Task 2 dispatches."
```

**IMPLEMENT (file 1 of 1):**

At the bottom of `crates/apub/activities/src/governance/publish_trust_attestation.rs` (after the file's final non-test item), append a new test module. Recommended anchor: append after the last function or `impl` block; do NOT insert into an existing block.

```rust
#[cfg(test)]
mod tests_per_actor_bound {
  //! Pure-function tests for the per-actor rate-map insertion-order bound
  //! added per v1-federation-inbound-d plan §3. Exercises
  //! `MAX_PER_ACTOR_RATE_ENTRIES` cap behaviour against the live
  //! `rate_per_actor_counts()` `OnceLock` — clears the global at test start
  //! AND end so test order is not load-bearing across the apub-activities
  //! lib-test binary.
  //!
  //! No unwrap/expect per workspace lints — uses `PoisonError::into_inner`
  //! for Mutex-poison recovery (canonical pattern; see
  //! `publish_trust_attestation.rs:161` + `inbox.rs:547`).
  use super::MAX_PER_ACTOR_RATE_ENTRIES;
  use crate::governance::inbox::{current_hour_bucket, rate_per_actor_counts};
  use std::sync::PoisonError;

  #[test]
  fn per_actor_map_evicts_oldest_when_cap_reached() {
    // Clear stale state from prior tests (the OnceLock is process-global).
    {
      let mut counts = rate_per_actor_counts()
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
      counts.clear();
    }

    let bucket = current_hour_bucket();
    let cap = MAX_PER_ACTOR_RATE_ENTRIES;

    // Insert `cap + 1` distinct keys, applying the same bound logic that
    // ships in `check_per_actor_rate_limit` (Task 1).
    for i in 0..=cap {
      let key = (format!("https://test/{i}"), bucket);
      let mut counts = rate_per_actor_counts()
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
      counts.retain(|(_, b), _| *b >= bucket - 1);
      if counts.len() >= cap && !counts.contains_key(&key) {
        if let Some(oldest_key) = counts
          .iter()
          .min_by_key(|((_, b), _)| *b)
          .map(|(k, _)| k.clone())
        {
          counts.remove(&oldest_key);
        }
      }
      let entry = counts.entry(key).or_insert(0);
      *entry = entry.saturating_add(1);
    }

    // Assert: map size capped at MAX_PER_ACTOR_RATE_ENTRIES.
    {
      let counts = rate_per_actor_counts()
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
      assert_eq!(
        counts.len(),
        cap,
        "per-actor map must be bounded at MAX_PER_ACTOR_RATE_ENTRIES after cap + 1 inserts",
      );
      let first_key = (String::from("https://test/0"), bucket);
      assert!(
        !counts.contains_key(&first_key),
        "oldest inserted key (i=0) must be evicted by the bound",
      );
    }

    // Cleanup: clear the map so other tests in this binary start fresh.
    let mut counts = rate_per_actor_counts()
      .lock()
      .unwrap_or_else(PoisonError::into_inner);
    counts.clear();
  }
}
```

The impl agent at canonical-schema-first read time MAY rename the test fn to a sibling-convention-matching alternative (`bound_keeps_per_actor_map_at_max_entries`, `per_actor_rate_map_insertion_order_eviction`, etc.) and update the assertion message strings accordingly.

**MIRROR:** `crates/apub/activities/src/governance/publish_sanction_notice.rs:612-666` — the canonical crate-internal `#[cfg(test)] mod tests` sibling inside the apub-activities crate. Match the surface verbatim: `//!` module docstring, `use super::...` import block, `#[test] fn ...()` (Task 2 uses `()` return rather than `LemmyResult<()>` because no `?`-propagation is needed; the sibling at `publish_sanction_notice.rs` uses `LemmyResult<()>` because its tests need `?` for `fixture_person`). The impl agent picks the simpler signature unless `?` becomes needed at canonical-schema-first read time.

**GOTCHA (apply to Task 2):**

- The `OnceLock<Mutex<HashMap<...>>>` is **process-global** within the apub-activities lib-test binary. Always `clear()` at test start AND end so test order is not load-bearing across cargo's default parallel test execution.
- Do NOT add a `[dev-dependencies] serial_test = "..."` to `crates/apub/activities/Cargo.toml` — the `clear()` + `Mutex` discipline is sufficient (and adding a dev-dep is out of scope per §12 item #10).
- The test body MUST mirror Task 1's bound logic verbatim. If Task 1's impl agent picks a different eviction strategy (e.g. `BTreeMap` derivation, or `keys().next()` for arbitrary-order rather than `iter().min_by_key()` for oldest-bucket selection), Task 2's body MUST follow the same shape so the test exercises what shipped. The Task-2 impl-task brief (authored post-Task-1 finalize-merge) reads `publish_trust_attestation.rs` at Task-1 HEAD and mirrors the exact branch verbatim before writing the test.
- No `.unwrap()` / `.expect()` shotguns per `feedback_clippy_test_style.md`. `PoisonError::into_inner` is the canonical Mutex-poison recovery; `assert_eq!` / `assert!` are workspace-lint permitted in `#[cfg(test)]` blocks.
- Edit budget: ~80 lines added (the new mod) — well under the e2e-edit-hang threshold (`feedback_junior_worker_e2e_edit_hang.md`'s ≤200-line discipline). `publish_trust_attestation.rs` is ~510 lines pre-fix; the append lands at the bottom; impl agent uses `Read` then `Edit` with the file's tail anchor (the last `}` closing the file's outermost item).
- **Conformance-audit:** Task 2's target file is the same Tier-1 file as Task 1. The Task-2 impl-task brief inherits the §3.1.1 audit run from Task 1; advisor SHOULD re-run the audit at Task-2 brief time only if Task 1 introduced new helpers/functions (most likely it did not — bound branch is inline).

**VALIDATE (story-checkpoint feeds §16a Story 1):**

Worker pre-push:

```bash
git diff --stat
# EXPECT: 1 file changed, ~80 insertions(+), 0 deletions(-)
git add crates/apub/activities/src/governance/publish_trust_attestation.rs
git commit -m "test(fed-in-d): unit test for per-actor rate-map bound (task 2)"
git push origin <worker-branch>
# Worker then writes kind: "validate-pending-laptop" DQ entry naming the §15 + lib-test commands.
```

Post-push (advisor-laptop):

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-d-task2-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-d-task2-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-d-task2-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-d-task2-clippy.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_apub_activities --lib > .claude/PRPs/debug/v1-federation-inbound-d-task2-libtest.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-federation-inbound-d-task2-libtest.log
# EXPECT: exit 0; tail shows the new test in "running N tests" + "test tests_per_actor_bound::per_actor_map_evicts_oldest_when_cap_reached ... ok"

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-federation-inbound-d-task2-e2e-norun.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-d-task2-e2e-norun.log
# EXPECT: exit 0 (e2e harness still re-links — R7 check; Task 2 added a new test target inside lemmy_apub_activities but lemmy_server's e2e binary is unaffected)
```

Phase-2 e2e regression check (user gate 4 — local vs dispatch per `feedback_windows_e2e_requires_bat_wrapper.md`). Default-recommended LOCAL after Task 2 finalize-merged onto phase branch:

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-d-<sha>.log 2>&1 && echo E2E_EXIT_0 >> .claude/runlog/e2e-v1-federation-inbound-d-<sha>.log || echo E2E_EXIT_NONZERO >> .claude/runlog/e2e-v1-federation-inbound-d-<sha>.log"
# run_in_background: true - ~26 min
# EXPECT (tail of log): E2E_EXIT_0; all v1_federation_inbound_b_fixtures tests pass (no regression on per-actor rate path); pre-existing fed-in-a/b/c tests still pass. This is a regression gate ONLY — Task 2's unit test is NOT exercised by the e2e binary; the unit test lives in the apub-activities lib-test binary.
```

### Task 3: Retro

**ACTION:** author `.claude/PRPs/reports/v1-federation-inbound-d-retro.md` per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`.

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-federation-inbound-d-retro.md
modifies: []
requires:
  - task: 2
    reason: "Retro signals require Task 2's Phase-2 e2e completion + any CR-fix-in-PR cycles having landed before authorship."
```

**IMPLEMENT:** four H2 sections (Advisor / Planning / Impl / BM) with signals + per-task complexity scores (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`) + lessons promoted this phase + carry-forward items. Mandatory sub-sections per brief §2.1 Task 3:

- **Advisor signal:** did the conformance-audit prevention checkpoint (§3.1.1) at Task 1 brief authoring catch any Tier-1 findings? Did the cycle-count meta-rule fire (≥3 fails on same `(error_class, file_basename)`)? Did the retro-bypass JSONL `.claude/governance-log/retro-bypass.jsonl` show monotonic decrease?
- **Planning signal:** did the §0 / §5 dogfood checks hold against final HEAD? Was Pattern §10.1 (lock+retain+bound+entry) cited correctly by Task 1's impl-task brief? Was Pattern §10.2 (`#[cfg(test)] mod tests` sibling) cited correctly by Task 2's impl-task brief?
- **Impl signal:** per-task complexity expected `1/1/<short>/<short>` for Task 1 (one file, one commit, ~5-10 min runtime, ~1 min max log silence) and `1/1/<short>/<short>` for Task 2 (one file, one commit, ~10-15 min runtime including lib-test run + Phase-2 e2e regression gate, ~1-2 min max log silence). Surface any divergence (e.g. unexpected fix-impl cycles, clippy auto-fix that hit §G4 allowlist).
- **BM signal:** did `bm-cut` + `bm-pr` + `bm-merge` flow cleanly? Any CR triage cycles? Did `gh pr merge` exit clean? Was the phase branch deleted from origin per `bm-merge.md` L16 post-condition?
- **Mandatory carry-forward (per brief §2.1 Task 3):** bootstrap-vs-reality drift class. The fed-in-d bootstrap (committed at `6a9f004a9` as part of `chore(brehon): close v1-federation-inbound-c, bootstrap v1-federation-inbound-d`) cited stale line numbers vs HEAD. The original brief at `bcc022310` inherited the drift. `/brehon-clarify` caught it because the advisor `grep`ed live code before authoring DQ entries. Retro proposes a tighter discipline (the bootstrap author MUST `rg` live code for every file:line citation before commit) and a candidate lesson `feedback_bootstrap_file_line_citations_must_grep_head.md` (if user judges promotable at retro sign-off).
- **Mandatory carry-forward:** TOCTOU eviction fix on `inbox.rs:654` (`evict_oldest_unreviewed_if_needed`) — still deferred. Retro proposes the next slice (likely v1-federation-inbound-e) with a §0.1 PRECON naming atomic-SQL-with-RETURNING vs `SELECT FOR UPDATE SKIP LOCKED` choice as the gate-1 question for the planner.
- **Optional carry-forward:** per-peer rate-map bound (was Defect 1, dropped per user clarify B1). Retro evaluates whether post-pilot DoS metrics or any in-pilot observation justify revisiting.
- **Optional carry-forward:** SHA-256 key-hash for the per-actor map (was dropped per user clarify B3). Retro evaluates whether per-entry heap (~800 KB at cap) is load-bearing per post-pilot observation; if so, the v1-federation-inbound-f planning brief should pre-PRECON the choice (16-byte SHA-256 truncation vs blake3 vs xxhash).

**VALIDATE:** retro committed; no failing checkpoints; the lessons-promoted block (if any) lands in the same retro commit (per `feedback_one_system_memory_in_repo.md` — promote inline at retro time). Subject: `docs(retro): v1-federation-inbound-d — per-actor rate-map bound shipped`.

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features full` after Tasks 1+2 (§15.1).
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings` after Tasks 1+2 (§15.2).
- **Crate-internal lib test:** `cargo test -p lemmy_apub_activities --lib` after Task 2 (§15.3). Exercises the new `tests_per_actor_bound::per_actor_map_evicts_oldest_when_cap_reached`.
- **Test target compile:** `cargo test --workspace --features full --test e2e --no-run` after Task 2 (§15.4) — R7 check: confirms e2e harness still re-links across the workspace after Task 2's new test module. The e2e binary is unaffected by Task 2 (the new test is in `lemmy_apub_activities`'s lib, not `lemmy_server`'s e2e binary), but the R7 check is uniform across tasks-touching-tests.
- **Phase-2 e2e regression gate:** `cargo test --workspace --test e2e --features full` (full run, all tests) after Task 2 finalize-merge — confirms Task 1's bound branch does NOT break the existing per-actor rate path or any other fed-in-* test (§15.5).
- **Migration round-trip:** N/A (zero migrations per PRECON-4).

**Behavioural assertion (post-fix):** Task 2's unit test, when run against an artificial pre-fix codebase (no bound branch in `check_per_actor_rate_limit`), would fail: `counts.len()` after `cap + 1` distinct inserts would equal `cap + 1` (not `cap`), so the first `assert_eq!` would fire. Post-fix, the result is deterministic — `counts.len() == cap` and the first-inserted key is evicted.

## 15. Validation commands (DoD)

> **Planner-side discipline (per `feedback_plan_dod_dry_run_at_write.md` + `feedback_pre_phase_dod_smoke_test.md`):** every command below MUST be dry-run by the advisor against current HEAD before plan approval (User Gate 1). Unexecutable commands are advisor-side rejection grounds.
>
> **DoD shape:** `validate-pending-laptop` (per brief PRECON-5 — Shape G SUSPENDED until 2026-06-01, DQ #229 pending re-enable). Each impl-task raises `kind: "validate-pending-laptop"` post-push naming these §15 commands verbatim with `--workspace --features full`. Advisor mutates with `answered_by: "advisor-laptop"`. Cargo runs on the laptop, never on the EliteDesk daemon.

### 15.1 Per-task workspace check (Tasks 1, 2)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-d-task<N>-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-d-task<N>-check.log
```

**EXPECT:** exit 0.

### 15.2 Per-task clippy (Tasks 1, 2 — uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-d-task<N>-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-d-task<N>-clippy.log
```

**EXPECT:** exit 0.

### 15.3 Crate-internal lib-test execution (Task 2 only)

Task 2 adds `#[cfg(test)] mod tests_per_actor_bound`. The lib-test binary for `lemmy_apub_activities` runs the new test (and any pre-existing crate-internal tests).

```bash
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_apub_activities --lib > .claude/PRPs/debug/v1-federation-inbound-d-task2-libtest.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-federation-inbound-d-task2-libtest.log
```

**EXPECT:** exit 0; tail shows `test tests_per_actor_bound::per_actor_map_evicts_oldest_when_cap_reached ... ok`. (Note: no `--features full` per `feedback_features_full_p_crate_incompatible.md`. The apub-activities crate's `full = []` feature is empty; nothing inside the crate gates on it.)

### 15.4 Test target compile (R7 — Task 2 only)

Task 2 adds a new test module inside `lemmy_apub_activities`; confirm e2e harness still re-links across the workspace:

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-federation-inbound-d-task2-e2e-norun.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-d-task2-e2e-norun.log
```

**EXPECT:** exit 0.

### 15.5 Phase-2 e2e regression gate (post-finalize-merge — user-gate-4)

**(a) Local laptop bg** (default-recommended; ~26 min, zero billed; per `feedback_windows_e2e_requires_bat_wrapper.md`):

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-d-<sha>.log 2>&1 && echo E2E_EXIT_0 >> .claude/runlog/e2e-v1-federation-inbound-d-<sha>.log || echo E2E_EXIT_NONZERO >> .claude/runlog/e2e-v1-federation-inbound-d-<sha>.log"
# run_in_background: true
```

**(b) GH dispatch** (escape hatch only; ~26 min billed; per User Gate 4 option (b)):

```bash
gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-federation-inbound-d
```

**EXPECT:** all tests pass — the pre-existing `v1_federation_inbound_b_fixtures` suite (including any per-actor-rate-related sibling) + pre-existing fed-in-a/c tests still pass; no regression introduced by Task 1's bound branch. Tail of the log: `E2E_EXIT_0`. (This is a regression gate ONLY — Task 2's unit test is exercised by §15.3, NOT by the e2e binary.)

### 15.6 Shape G section — DORMANT until 2026-06-01

**NOT applicable.** Per brief PRECON-5 + DQ #229. If Shape G re-enables before this sub-phase ships, the planner re-files a `chore(decision-queue)` advisor entry switching the §15 shape to Shape G workflow references; no plan re-author needed (forward-only retrofit per `feedback_schema_changing_spec_retrofit_question.md`).

### 15.7 Cross-cutting verification

The planner asserts each box holds at end-of-phase:

- [ ] `grep -n "const MAX_PER_ACTOR_RATE_ENTRIES" crates/apub/activities/src/governance/publish_trust_attestation.rs` returns exactly 1 match (the new module-scope const).
- [ ] `grep -n "MAX_PER_ACTOR_RATE_ENTRIES" crates/apub/activities/src/governance/publish_trust_attestation.rs` returns ≥ 3 matches (const declaration + Task-1 use in `check_per_actor_rate_limit` + Task-2 import in `mod tests_per_actor_bound`).
- [ ] `grep -n "mod tests_per_actor_bound" crates/apub/activities/src/governance/publish_trust_attestation.rs` returns exactly 1 match.
- [ ] `grep -n "fn check_per_actor_rate_limit" crates/apub/activities/src/governance/publish_trust_attestation.rs` returns the function at its existing line (~118; signature unchanged; no symbol drift).
- [ ] `grep -n "pub(crate) fn rate_per_actor_counts" crates/apub/activities/src/governance/inbox.rs` returns the map definition unchanged at ~471 (no edit to `inbox.rs`).
- [ ] `grep -n "pub(crate) fn rate_per_peer_counts" crates/apub/activities/src/governance/inbox.rs` returns the per-peer map definition unchanged at ~465 (per user clarify B1).
- [ ] `grep -c "fn evict_oldest_unreviewed_if_needed" crates/apub/activities/src/governance/inbox.rs` returns 1 (function still present, unchanged; deferred per §12 item #3).
- [ ] `grep -n "counts.retain(|(_, b), _| \*b >= bucket - 1);" crates/apub/activities/src/governance/publish_trust_attestation.rs` returns exactly 1 match at the per-actor use site (pre-existing 2-hour prune preserved).
- [ ] `grep -n "counts.retain(|(_, b), _| \*b >= bucket - 1);" crates/apub/activities/src/governance/inbox.rs` returns exactly 1 match at the per-peer use site (~line 548; unchanged per user clarify B1).
- [ ] `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns **55** (no kind additions/deletions; this sub-phase does not touch the registry).
- [ ] `rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l` returns **55** (shim parity unchanged).
- [ ] No edit to `crates/db_schema/src/source/governance/governance_config.rs`, `crates/apub/activities/src/governance/inbox.rs`, `crates/server/tests/e2e.rs`, `crates/api/api/src/governance/**`, or any file under `migrations/**` (PRECON-4 + brief §2.1 scope wall).
- [ ] No new file under `crates/apub/activities/src/governance/` (PRECON: helper not extracted).
- [ ] R1: no `i32 as i64` casts in new code (the bound uses `usize` comparisons; the pre-existing `i64::from(*entry) > actor_cap` is unchanged).
- [ ] R6: every §15.2 invocation uses `--no-deps -- -D warnings`.
- [ ] R7: Task 2 ran `cargo test --no-run` (§15.4).
- [ ] PRECON-1 (scope: per-actor rate-map bound only) honoured: no per-peer edits, no key-hash edits, no TOCTOU edits, no helper extraction.
- [ ] PRECON-2 (bound at use site) honoured: const declared at module scope; bound branch lands inside `let exceeded_actor = { ... }` at lines 158-167; no edit to `inbox.rs:471-474` map definition.
- [ ] PRECON-3 (crate location) honoured: edits land in `crates/apub/activities/`, NOT `crates/api/api/`.
- [ ] PRECON-4 (zero migrations) honoured.
- [ ] PRECON-5 (validate-pending-laptop DoD) honoured.
- [ ] PRECON-6 (no cohort dispatch) honoured: serial Task-1 → Task-2 → Task-3 dispatch; no `[P]` markers.
- [ ] PRECON-7 (review-point sequencing) honoured: brief approved → planning dispatched → plan approved → bm-cut → Task 1 → Task 2 → Task 3 → bm-pr → bm-merge.

### 15.8 ADR / OQ compliance

- [ ] **ADR-006** (advisory-only inbound persistence): unchanged. The fix is in-memory data-structure logic; no row-shape change; `local_case_id` semantics untouched.
- [ ] **ADR-013** (illegal content / `CaseStatus::EmergencyRemove`): not in code path; unchanged.
- [ ] **ADR-014** (governance signals are fork-only AP types): unchanged. Inbox wrapper still fires only on governance AP types; vanilla Lemmy unaffected.
- [ ] **ADR-015** (pseudonymisation): unchanged. No new TEXT column; no raw-id surface; `subject_url` plaintext as before (per user clarify B3).
- [ ] **Append-only contract** (`governance_config`): preserved. Reader-side `.order_by(valid_from.desc())` from fed-in-c is unchanged.

---

## 16. Acceptance criteria

Roll-up of §15 + §16a story checkpoints. The planner asserts each box is ticked at end-of-phase. The advisor's `/brehon-verify` cross-checks each box against the worktree branch before queueing `bm-merge`.

- [ ] All 4 tasks (Task 0 + 3 work tasks) completed in dependency order.
- [ ] §15.1 (cargo check) exit 0 after Tasks 1, 2.
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after Tasks 1, 2.
- [ ] §15.3 (cargo test -p lemmy_apub_activities --lib) exit 0 after Task 2; tail shows the new test `tests_per_actor_bound::per_actor_map_evicts_oldest_when_cap_reached ... ok`.
- [ ] §15.4 (cargo test --no-run --test e2e) exit 0 after Task 2 (R7).
- [ ] §15.5 (Phase-2 e2e regression gate) — all pre-existing tests still pass; no regression; tail of log shows `E2E_EXIT_0`.
- [ ] §15.6 N/A (Shape G dormant).
- [ ] §15.7 cross-cutting verification — all boxes ticked.
- [ ] §15.8 ADR/OQ compliance — all boxes ticked.
- [ ] §16a stories — all stories `[done]`.
- [ ] No edits to files outside §11 list.
- [ ] Retro committed per §13 Task 3.
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`.

---

## 16a. Stories (independently-testable behaviour units)

> Per `feedback_brehon_verify_pre_merge.md`. Each story names its composing §13 tasks + a checkpoint command and Brief-Scope outputs `/brehon-verify` confirms.

### Story 1: Per-actor rate-map stays bounded under attacker key-cardinality

- **User-facing behaviour:** a single allowlisted peer cannot grow the in-memory per-actor rate-limit map beyond `MAX_PER_ACTOR_RATE_ENTRIES = 10_000` distinct keys by crafting many `subject_url` values inside one hour bucket. The bound is enforced at the use site in `check_per_actor_rate_limit` and exercised by a crate-internal unit test.
- **Composing tasks:** Task 1 (bound branch) + Task 2 (unit test). Sequential (Task 2 `requires: [1]`).
- **Checkpoint command (laptop):** `cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_apub_activities --lib"` against Task 2's phase-branch tip (post-finalize-merge). EXPECT exit 0; tail shows `test tests_per_actor_bound::per_actor_map_evicts_oldest_when_cap_reached ... ok`.
- **Brief-Scope outputs to verify:**
  - `crates/apub/activities/src/governance/publish_trust_attestation.rs` contains a module-scope `const MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000;` (single match in the file).
  - `publish_trust_attestation.rs` contains an `if counts.len() >= MAX_PER_ACTOR_RATE_ENTRIES && !counts.contains_key` branch inside the `let exceeded_actor = { ... }` block.
  - `publish_trust_attestation.rs` contains a `#[cfg(test)] mod tests_per_actor_bound { ... }` at the bottom of the file.
  - The pre-existing `counts.retain(|(_, b), _| *b >= bucket - 1)` line inside `check_per_actor_rate_limit` is present and unchanged.
  - `crates/apub/activities/src/governance/inbox.rs` map definitions at `rate_per_peer_counts` (~465) + `rate_per_actor_counts` (~471) are unchanged.
  - The per-peer use site at `inbox.rs:540-565` is unchanged (per user clarify B1).

### Story 2: Phase-2 e2e regression gate is green

- **User-facing behaviour:** the bound branch added in Task 1 does NOT alter observable per-actor rate-limit semantics for benign traffic (the bound only fires at `len() >= 10_000` which no e2e test approaches). All pre-existing `v1_federation_inbound_b_fixtures` tests + fed-in-a/c tests still pass.
- **Composing tasks:** Task 1 + Task 2 (regression gate runs after both land on phase branch).
- **Checkpoint command (laptop, Phase-2 e2e):** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1"` with `run_in_background: true`. EXPECT `E2E_EXIT_0` in the log tail.
- **Brief-Scope outputs to verify:**
  - `.claude/runlog/e2e-v1-federation-inbound-d-<sha>.log` exists and tail contains `E2E_EXIT_0`.
  - No new test failure introduced (count of "FAILED" in log = 0).
  - Pre-existing `mod v1_federation_inbound_b_fixtures` test count matches the pre-fix baseline (5 tests at fed-in-c retro time; verify no test removed).

### Story 3: Retro captures four-role signals + complexity scores + carry-forward

- **User-facing behaviour:** `bm-merge` runs only after retro authorship; user gate 6 (retro sign-off) gates `/brehon-phase-transition`.
- **Composing tasks:** Task 3 (non-`[P]`; `requires: [2]`).
- **Checkpoint command:** `test -f .claude/PRPs/reports/v1-federation-inbound-d-retro.md && grep -c '^## ' .claude/PRPs/reports/v1-federation-inbound-d-retro.md` returns ≥ 4 (one H2 per role).
- **Brief-Scope outputs to verify:**
  - The retro file exists at the canonical path.
  - Four H2 sections present: Advisor / Planning / Impl / BM.
  - Per-task complexity score block present (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`) per `feedback_retro_task_complexity_score.md`.
  - Carry-forward block names: (a) the bootstrap-drift discipline proposal + candidate lesson `feedback_bootstrap_file_line_citations_must_grep_head.md`; (b) the TOCTOU eviction fix deferral with gate-1 question for v1-federation-inbound-e; (c) optional revisit-triggers for the per-peer bound + SHA-256 key-hash deferrals.
  - Lessons promoted this phase (if any) land in the same retro commit body, paths under `.claude/lessons/feedback_*.md`.

> **Verification mapping:** `/brehon-verify` iterates this section per `.claude/commands/brehon-verify.md`, runs each Story's Checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantoms (task complete but output absent or empty) trigger the catch-fire procedure in advisor-orchestrator.md.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all 9 probe families confirmed: 0..8 inclusive).
- [ ] Tasks 1, 2 committed in dependency order (serial; no cohort).
- [ ] Task 3 retro committed.
- [ ] §15 validation green at every gate.
- [ ] §16a stories all `[done]`.
- [ ] PR opened by BM session against `governance-v0` with `--repo barrie-cork/lemmy`.
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-federation-inbound-d-verify.md` shows all stories ✓.
- [ ] PRECON-1 / PRECON-2 / PRECON-3 / PRECON-4 / PRECON-5 / PRECON-6 / PRECON-7 honoured.
- [ ] Post-merge phase branch retained for retro reads (per `.claude/commands/bm/bm-merge.md` post-condition).

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Impl agent adds the bound at the map DEFINITION (`inbox.rs:471-474`) instead of the use site | LOW | MEDIUM | PRECON-2 + §4.1 + §10.1 GOTCHA forbid; the per-peer map at the adjacent definition is not bounded (per user B1), so a definition-site bound would over-apply. Brief §4.1 enumerates the wall. |
| Impl agent extracts a shared helper across `inbox.rs` + `publish_trust_attestation.rs` | LOW | HIGH | PRECON-3 in brief §0/§12 item #7 + plan §10.1 GOTCHA; fed-in-c PRECON-3 (circular-dep) still binds. Brief §4 Constraints forbids extraction. |
| Impl agent introduces `.await` inside the same-lock block, triggering `clippy::await_holding_lock` | LOW | HIGH | §4 watchpoint WP-1 names the lock-discipline contract; §10.1 GOTCHA forbids `.await` between `counts.lock()` and end-of-block. Diesel reads happen earlier (lines 139-153) outside the lock. The §G4 allowlist does NOT cover `clippy::await_holding_lock`; catch-fire if it fires (re-plan, NOT auto-fix). |
| Impl agent uses `iter().next()` or random-order eviction instead of `min_by_key(bucket)` | LOW | LOW | §10.1 verbatim shape names `iter().min_by_key(|((_, b), _)| *b).map(|(k, _)| k.clone())`; Task 2's test mirrors this shape and would fail if a different strategy ships (the first-inserted key would not be the one evicted under arbitrary-order). |
| Task 2's unit test is flaky due to OnceLock state interference from concurrent tests in the lib-test binary | LOW | MEDIUM | The `clear()` discipline at test start AND end + `Mutex` serialization makes the test safe under parallel execution. If flake emerges, retro flags as carry-forward (consider `serial_test` dep or RAII-based test isolation in a follow-up). |
| MAX_PER_ACTOR_RATE_ENTRIES value (10_000) is wrong (too tight, breaking benign workload OR too loose, leaving DoS surface large) | LOW | MEDIUM | §4 watchpoint WP-1 calls out the value as planner-chosen with rationale; retro evaluates post-pilot; option-a (governance_config-backed) deferred to a follow-up sub-phase if config-driven tuning becomes load-bearing. |
| Cycle-count meta-rule (≥3 fails on same `(error_class, file_basename)`) fires during Task 1 dispatch | LOW | HIGH | Brief §4.5 sets honest expectation: anticipated fail modes (e.g. `clippy::await_holding_lock`, style-clippy on the `min_by_key` chain) are NOT auto-fixable via §G4 allowlist; planner expects catch-fire on red, not retry-with-bridge. The meta-rule is a hard refusal regardless of allowlist match. |
| Advisor-side §3.4 DoD smoke fails at plan-approval time (e.g. Probe 8 baseline clippy regresses against current HEAD) | LOW | HIGH | Task 0 probes capture the baseline; if Probe 8 fails before Task 1 dispatch, advisor files DQ pending and requests planner narrowing OR a pre-phase `chore(lint)` commit per `feedback_pre_phase_dod_smoke_test.md`. |
| Shape G re-enables mid-sub-phase (2026-06-01 boundary crossed) | LOW | LOW | PRECON-5 carries a forward reminder. If crossed, advisor switches the in-flight `kind` from `validate-pending-laptop` to `validate-pending` at next impl-task dispatch; mechanical, no plan re-author. |
| Junior worker forks from stale base (pre-bm-cut tip) and misses fed-in-c's `.order_by(...)` reader fix | LOW | HIGH | bm-cut creates `phase-v1-federation-inbound-d` off `governance-v0` post-fed-in-c-merge; Probe 6 at Task 0 verifies both `.order_by(governance_config::valid_from.desc())` lines exist on the cut branch; per `feedback_junior_292_stale_base_recover_recipe.md` for recovery if drift detected post-dispatch. |
| Task 2 brief is authored before Task 1 finalize-merges, missing Task 1's actual chosen eviction strategy | MEDIUM | MEDIUM | Task 2's `requires: [1]` gates dispatch; advisor authors Task 2's impl-task brief AFTER Task 1 finalize-merge so the canonical-schema-first read of `publish_trust_attestation.rs` reflects Task 1's actual chosen shape (not the plan-time recommended shape). The plan §10.2 skeleton is the recommended-mirror; the brief §3 Required-reading pulls Task 1 HEAD verbatim. |

---

## 19. Notes

- **Scope discipline** — this plan deliberately excludes the per-peer bound, key-hash, TOCTOU fix, governance_config knob, Postgres backing, helper extraction, cohort dispatch, and Shape G switch per brief §0.2. The user reviewed and confirmed scope at clarify-gate 2026-05-22 (DQ `a3d0e9941441-005` user-relayed B1; DQ `a3d0e9941441-006` advisor-resolved). §12 mirrors brief §0.2 with deferral pointers.

- **No `[P]` cohort, no DQ pre-reservation** — Tasks 1+2 both modify `publish_trust_attestation.rs`. The YAML overlap check in `.claude/rules/advisor-orchestrator.md` §4.1 step 4 refuses any `[P]` cohort for overlapping `modifies:`. Serial dispatch only. `feedback_cohort_dq_id_collision.md` Option 3 (pre-reservation) is NOT applicable.

- **PRECON enumeration (7 items)** — recorded in brief §0.1 + plan §12 + plan §15.7 cross-cutting verification. The brief authoritatively states them; the plan reproduces them in actionable form (per-PRECON checkbox under §15.7).

- **Open questions filed as DQ pre-seeds** — none. The brief's §0.1 closes the 7 PRECONs in advance; the two clarify-DQ resolutions (`a3d0e9941441-005` + `a3d0e9941441-006`) cover the residual coverage gaps. The planning subagent does NOT raise additional clarifies (per `.claude/rules/advisor-orchestrator.md` §3.3 — clarify is advisor-only at pre-planning).

- **Lessons promoted this phase (anticipated)** — at retro time the candidate `feedback_bootstrap_file_line_citations_must_grep_head.md` may promote (if user judges retro evidence sufficient). The drift incident (fed-in-d bootstrap citing `inbox.rs:473` / `publish_trust_attestation.rs:165` / `inbox.rs:698` when HEAD has `inbox.rs:471` / `publish_trust_attestation.rs:159` / `inbox.rs:654`) provides the evidence base.

- **Alternative approaches considered (rejected at brief-author time, recorded for retro context)** —
  1. **Per-peer + per-actor bound bundled** (rejected per user clarify B1; per-peer has natural upstream rate-limit via Gate-1 allowlist + signature checks).
  2. **SHA-256 key-hash (16-byte truncation)** (rejected per user clarify B3; per-entry heap savings not worth losing debuggability).
  3. **`governance_config`-backed cap** (rejected per brief PRECON-1 + option-a deferral; Rust `const` is v0 — config-driven tuning waits for post-pilot evidence).
  4. **Postgres-backed rate counters** (rejected per option-a deferral; in-memory bound is the simplest correct shape; DB-backed counters need migration + concurrency design).
  5. **Helper extraction** (rejected per fed-in-c PRECON-3 + circular-dep avoidance + B1 (different policies per map)).
  6. **Bound at the map DEFINITION** (rejected per PRECON-2; per-peer map at the adjacent definition does NOT get the same bound — definition-site bound would over-apply).
  7. **e2e test for Task 2** (rejected per user clarify B1: `rate_per_actor_counts` is `pub(crate)` to `lemmy_apub_activities`; not callable from `crates/server/tests/e2e.rs`; sending 10_001 inbound activities exceeds per-test budget; unit test is the canonical approach).

---

## 20. Confidence score

- **Plan correctness:** 9/10 — defect verified at brief author time (§5 dogfood + planning-time `grep` against HEAD `7e6c4202f`), MIRROR refs verified (`publish_sanction_notice.rs:612` confirmed at planning time as a crate-internal `#[cfg(test)] mod tests` sibling; `inbox.rs:540-565` confirmed as the per-peer use site whose lock+retain+entry pattern Task 1 mirrors). The fix is small (one const + ~4 new lines in the same-lock block) and the unit test is bounded by `cap = 10_000` (~10_001 inserts; in-process). Minor uncertainty in the eviction-strategy choice (planner picked `iter().min_by_key(bucket)` for clarity; the impl agent may justify a different shape at canonical-schema-first read time — flagged as WP-1 watchpoint).
- **Cargo budget:** 10/10 — N/A (validate-pending-laptop runs on laptop; budget non-binding).
- **Test coverage:** 7/10 — Task 2's unit test directly exercises the bound + eviction at the canonical cap value, asserting both `len() == MAX` and "first-inserted key evicted". No e2e coverage of the bound through the live inbox path (deliberately deferred per user clarify B1; pushing 10_001 inbound activities through the trust-attestation receive path is over per-test budget). Regression coverage via §15.5 Phase-2 e2e confirms Task 1 does not break the pre-existing per-actor rate path. If a regression class emerges post-merge that touches the bound's interaction with the rate-limit Gate 5, retro evaluates adding a focused e2e in v1-federation-inbound-e/f.
