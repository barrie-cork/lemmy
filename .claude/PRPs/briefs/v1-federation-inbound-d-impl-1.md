# Brief: v1-federation-inbound-d impl Task 1 — per-actor rate-map bound

## 1. Role + dispatch

`[role:impl-task]` v1-fed-in-d-task1-bound — see `.claude/PRPs/briefs/v1-federation-inbound-d-impl-1.md`

## 2. Scope

### 2.1 What to produce

Add a `const MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000;` at module scope in
`crates/apub/activities/src/governance/publish_trust_attestation.rs` and
insert a four-line bound-check + insertion-order eviction branch inside the
existing `let exceeded_actor = { ... }` block at lines 158-167, between
`counts.retain(...)` and `counts.entry(...).or_insert(0)`. Refactor the
`let entry = ...` line to use a local `let key = (subject_url.to_string(),
bucket);` binding so the `contains_key` check and the `entry(key)` insert
share the same value.

**Exactly one file changed.** Exactly one commit: `feat(fed-in-d): bound per-actor rate-map at MAX_PER_ACTOR_RATE_ENTRIES (task 1)`.

### 2.2 Const declaration (verbatim)

After the `use` block (currently ends at line 30) and before the
`#[async_trait::async_trait]` attribute (currently at line 32), insert:

```rust
/// Maximum number of distinct (subject_url, hour_bucket) keys held in the
/// per-actor rate-limit map at any moment. A single allowlisted peer can
/// craft arbitrarily many subject URLs within one hour bucket; the
/// insertion-order eviction at `check_per_actor_rate_limit` keeps the map
/// bounded regardless of attacker key cardinality. See v1-federation-inbound-d
/// plan §3.
const MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000;
```

### 2.3 Post-edit block shape inside `let exceeded_actor = { ... }` (verbatim)

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

### 2.4 What NOT to produce (hard scope boundaries)

- Do NOT add the bound at the map DEFINITION site (`inbox.rs:471-474`).
- Do NOT edit `rate_per_peer_counts` or its use site at `inbox.rs:544-552`.
- Do NOT edit `inbox.rs:654` (`evict_oldest_unreviewed_if_needed`) — deferred.
- Do NOT read `MAX_PER_ACTOR_RATE_ENTRIES` from `governance_config`.
- Do NOT hash the `subject_url` String.
- Do NOT add an `.await` anywhere inside the `let exceeded_actor` block.
- Do NOT edit the `counts.retain(...)` line (preserve the existing 2-hour prune).
- Do NOT edit the Diesel reader at lines 139-153 (the `actor_cap` read).
- Do NOT edit the rate-counter Gate-5 check at lines 169-184.
- Do NOT create new files; do NOT edit any file outside `publish_trust_attestation.rs`.
- Do NOT add a migration.

### 2.5 Post-push DQ entry (validate-pending-laptop)

After `git push origin <worker-branch>`, write a `kind: "validate-pending-laptop"` entry
to `.claude/decision-queue.json` with:

```json
{
  "kind": "validate-pending-laptop",
  "from": "impl",
  "branch": "<worker-branch>",
  "phase_task": 1,
  "commands": [
    "cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-d-task1-check.log 2>&1\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-d-task1-clippy.log 2>&1\""
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null
}
```

Use `bash scripts/brehon/dq-v3-new-entry.sh` to generate the composite id. Commit + push the DQ entry immediately after writing it (per `.claude/rules/decision-queue.md` "Mid-task visibility").

## 3. Required reading (READ before any edit)

### 3a. Handover from prior cohort

(none — first cohort)

### 3b. Mandatory reads

1. **`.claude/PRPs/plans/v1-federation-inbound-d.plan.md` §4 + §10.1** — authoritative post-edit block shape and GOTCHAs.
2. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:1-35`** — confirm use block end line and `#[async_trait::async_trait]` line before inserting the const.
3. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:118-187`** — the full `check_per_actor_rate_limit` body; locate `counts.retain(...)` and `let entry = counts.entry(...)` lines precisely.
4. **`crates/apub/activities/src/governance/inbox.rs:540-565`** — MIRROR ref (per-peer use site): canonical lock+retain+entry pattern; Task 1 introduces the bound ON TOP of this pattern without disturbing it.
5. **`crates/apub/activities/src/governance/inbox.rs:464-479`** — map definitions + `current_hour_bucket`. READ-ONLY — no edit.
6. **`.claude/lessons/feedback_clippy_test_style.md`** — workspace lint discipline; `LemmyResult<()>` shape; no bare `.unwrap()`/`.expect()`.
7. **`.claude/lessons/feedback_features_full_workspace_only.md`** — always `--workspace --features full` for check + clippy; never `-p <crate>` + `--features full`.
8. **`.claude/lessons/feedback_explicit_file_arrays_on_tasks.md`** — FILES YAML; no `[P]` here (Tasks 1+2 overlap on same file).

### 3c. Conformance-audit status

Advisor ran the six-axis conformance audit on the target file before authoring this brief.
**Result: 0 Tier-1, 0 Tier-2.** No brief amendments required.
Report: `.claude/PRPs/reports/conformance-audit-publish-trust-attestation-2026-05-22.md`

## 4. Constraints

1. **Commit subject exactly:** `feat(fed-in-d): bound per-actor rate-map at MAX_PER_ACTOR_RATE_ENTRIES (task 1)`
2. **One commit, one file changed.** `git diff --stat` MUST show exactly: `1 file changed, ~25 insertions(+), 1 deletion(-)` (the `let entry = ...` refactor removes the old line; the new block adds ~25 lines).
3. **Pre-push `cargo check` gate** (per `feedback_fix_impl_pre_push_cargo_check.md`): run `bash scripts/brehon/cargo-check.sh --workspace --features full` (Linux) before `git push`. Non-zero exit → fix in same commit (if the patch is in-scope) OR file `kind: "blocker"` DQ (if out-of-scope). NEVER `#[allow]`-spam to bypass.
4. **DQ mid-task push discipline**: commit + push the `validate-pending-laptop` DQ entry immediately after the worker-branch push. No deferred push.
5. **Shape G SUSPENDED**: cargo validation runs on the laptop (validate-pending-laptop DoD); no `kind: "validate-pending"` (GH Actions shape). Write `kind: "validate-pending-laptop"` only.
6. **No `#[allow(...)]` additions** unless already present in the file and unchanged by this task.
7. **GOTCHA — same-lock scope**: the bound check + eviction + entry-or-insert all run under the same `MutexGuard`. No `.await` between `counts.lock()` and the end of the block.
8. **GOTCHA — `key.clone()`**: intentional — `counts.remove(&oldest_key)` needs an owned `(String, i64)`; cloning after `min_by_key().map(...)` avoids a borrow conflict with the surrounding `&mut counts`.
9. **GOTCHA — O(N) eviction is acceptable**: at N=10_000 the `iter().min_by_key()` scan fires only when the cap is hit; benign workloads never enter the branch. Per `feedback_principles_not_rules.md` (simplest correct shape first).
10. **No new file** under `crates/apub/activities/src/governance/`. No migration under `crates/db_schema/migrations/`.

## 5. FILES YAML (machine-parseable)

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/publish_trust_attestation.rs
requires: []
```
