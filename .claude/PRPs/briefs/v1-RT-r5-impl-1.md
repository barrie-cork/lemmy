# Brief: v1-RT-r5 impl-task 1 — rollup compute + batch in `reputation_snapshot.rs`

## 1. Role + dispatch line

`[role:impl-task]` v1-RT-r5 Task 1 — rollup compute + batch in reputation_snapshot.rs

## 2. Scope

Add `compute_rollup_snapshot`, `run_rollup_batch`, `load_rollup_candidates`, and
`RollupBatchOutcome` to `crates/api/api/src/governance/reputation_snapshot.rs`.
Compute equal-weighted per-dimension integer mean of non-banned per-community snapshots,
write via shared `upsert_snapshot` with `community_id=None`, emit `ROLLUP_RECOMPUTED`
(None/system pseudonym, ADR-015) and `CAPABILITY_CHANGED`(null community_id).

**Produce:**
- `RollupBatchOutcome` struct (mirror `SnapshotBatchOutcome` :127)
- `load_rollup_candidates(pool) -> LemmyResult<Vec<PersonId>>` — distinct `person_id`
  WHERE `community_id IS NOT NULL` (persons with ≥1 per-community snapshot)
- `compute_rollup_snapshot(conn, person_id, cache) -> LemmyResult<Option<ReputationSnapshot>>`
  per §10.1 (load per-community, exclude banned via sanction filter, integer mean,
  denominator-0 → return `Ok(None)`, capability bits, `upsert_snapshot` with
  `community_id=None`, emit §10.4 ROLLUP_RECOMPUTED + flip §10.2 CAPABILITY_CHANGED)
- `pub async fn run_rollup_batch(context: &LemmyContext) -> LemmyResult<RollupBatchOutcome>`
  per §10.1 control flow (mirror `run_snapshot_batch :481-530`)
- One commit on the task worktree branch
- `validate-pending-laptop` DQ entry with
  `commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`

**Do NOT:**
- Touch any file outside `crates/api/api/src/governance/reputation_snapshot.rs`
- Call `recompute_snapshot` (:222) for rollup — that is event-sum, categorically wrong
  (clarify DQ `81ae24440317-001`); use the new dedicated `compute_rollup_snapshot`
- Add new `ENTRY_KIND_*` consts (count stays 55; `ROLLUP_RECOMPUTED` already exists at :215)
- Use `as` casts for `i32 ↔ i64`; use `i64::from(...)` (R1)
- Commit to `governance-v0` or any branch other than the task worktree branch
- Write `approved_by: "advisor"` in any DQ entry

## 3. Required reading (in order)

Read all of these before the first Edit:

### Schema / type definitions
- `crates/db_schema/src/source/governance/reputation_snapshot.rs:1-47` — `ReputationSnapshot`
  (NOT `Copy`/`Default`/`Hash`; HAS `PartialEq`/`Eq`) + `ReputationSnapshotInsertForm`
  (`Clone, Default` + Insertable/AsChangeset)
- `crates/db_schema/src/source/governance/governance_log.rs:121,215,256` —
  `ENTRY_KIND_CAPABILITY_CHANGED` (:121), `ENTRY_KIND_ROLLUP_RECOMPUTED` (:215),
  `append(pool, entry_kind, payload, actor_pseudonym)` signature (:256)

### MIRROR refs (read and mirror their shape exactly)
- `crates/api/api/src/governance/reputation_snapshot.rs:481-530` — `run_snapshot_batch`
  (control-flow mirror for `run_rollup_batch`: chunk_size config, candidates, batch loop)
- `crates/api/api/src/governance/reputation_snapshot.rs:350-399` — capability bits
  + flip emit (mirror for `compute_rollup_snapshot` capability section; note `i64::from`)
- `crates/api/api/src/governance/reputation_snapshot.rs:736-768` — `load_person_context`
  (canonical banned-detection sanction filter — pass `community_id=None` for instance-wide
  `active_sanctions`)
- `crates/api/api/src/governance/reputation_snapshot.rs:770` — `upsert_snapshot`
  (SHARED write path; call with `community_id=None`)
- `crates/api/api/src/governance/reputation_snapshot.rs:125-141` — `SnapshotBatchOutcome`
  (mirror struct shape for `RollupBatchOutcome`)

### ANTI-MIRROR (do NOT copy)
- `crates/api/api/src/governance/reputation_snapshot.rs:222` — `recompute_snapshot`:
  this is **event-sourced instance recompute** (event-sum via `load_live_events :682`),
  NOT a weighted average of per-community snapshot values. Must not be reused.
- `crates/api/api/src/governance/reputation_snapshot.rs:236,682` — `load_live_events`:
  filters `community_id IS NULL` and sums `reputation_event` rows. Wrong computation.

### Lessons (mandatory)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — functions return
  `LemmyResult<...>`; never `Box<dyn Error>`; use `?` + annotated closures
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish`
  + `DbPool::Conn` for any test helper (not needed for impl itself but load for context)
- `.claude/lessons/feedback_clippy_test_style.md` — R1: `i64::from(dim) >= threshold`,
  never `dim as i64`; clippy denies `unwrap`/`expect`
- `.claude/lessons/feedback_features_full_workspace_only.md` — `--workspace --features full`
  only; never `-p <crate> --features full`
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — detail on why

## 4. Constraints

- **ROLLUP_RECOMPUTED payload shape (registry :188):**
  ```json
  {
    "person_id": person_id.0,
    "contributing_community_count": non_banned_count,
    "rollup_dimensions": { "reporting_accuracy": ra, "jury_reliability": jr,
                           "participation_consistency": pc, "endorsement_strength": es },
    "recomputed_at": now.to_rfc3339()
  }
  ```
  Actor pseudonym arg: `None` (system, ADR-015 / PRD §10).

- **CAPABILITY_CHANGED for rollup:** `community_id=None` → `snapshot_community_id: null`
  in payload (mirror :391 `community_id.map(|c| c.0)` with `community_id = None`).

- **Instance-wide `active_sanctions`:** call
  `load_person_context(conn, person_id, None).await?.1` (the `None` community-filter arm).

- **Denominator-0 → skip:** if all contributing communities are banned (denominator == 0),
  return `Ok(None)` — no `upsert_snapshot`, no `governance_log::append`, no capability flip.

- **Integer division:** per dimension: `sum(dim) / denominator` as `i32`. Truncates toward
  zero. This MUST match what Task 5's e2e computes for mean assertions.

- **DQ mid-task push:** after writing the `validate-pending-laptop` entry, immediately:
  `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised validate-pending-laptop DQ — v1-RT-r5 task 1" && git push origin <branch>`.

- **No `answered_by: "advisor"` from this session.** Use `answered_by: "impl-self-resolved"`
  for self-resolutions.

- **commit subject pattern:** `feat(reputation): add rollup compute + batch (task 1)`

### Mandatory lessons fired for this brief
- `reputation_snapshot.rs` (MIRROR ref at :481-530): `feedback_lemmy_error_no_std_error.md`
  (any `LemmyResult` return) ✓
- `reputation_snapshot.rs` (capability bits at :350-399): `feedback_clippy_test_style.md`
  (R1 `i64::from`) ✓
- `#[cfg(feature = "full")]` context: `feedback_features_full_workspace_only.md` ✓
