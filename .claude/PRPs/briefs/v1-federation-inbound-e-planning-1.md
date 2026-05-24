# [role:planning] v1-federation-inbound-e — per-peer actor-map bound + SHA-256 nonce

## 1. Role + dispatch

`[role:planning] v1-federation-inbound-e — see .claude/PRPs/briefs/v1-federation-inbound-e-planning-1.md`

## 2. Scope

Produce `.claude/PRPs/plans/v1-federation-inbound-e.plan.md`.

**Deliverables (one impl task, one worker branch):**

1. **Per-peer actor-map entry bound** — prevent one hostile peer from evicting legitimate
   actors from other peers by capping how many distinct keys from a single `peer_domain`
   can exist in the actor rate map at any moment.
2. **SHA-256 subject_url nonce** — replace the raw `subject_url: String` key in
   `rate_per_actor_counts()` with a fixed-width `[u8; 32]` SHA-256 digest.

Both changes land in one commit on one worker branch. No new migrations. No new
governance_config keys. No new HTTP endpoints.

**Do NOT author:**
- New governance_log entry kinds
- Any change to `crates/db_schema/`, `migrations/`, or `crates/server/tests/e2e.rs`
- Any change to `rate_per_peer_counts()` (separate per-peer hourly rate map)

## 3. Required reading

### 3a. Handover
None — first task of this phase. Read bootstrap:
`.claude/PRPs/handovers/v1-federation-inbound-e-bootstrap.md` §1 + §2 + §3.
The TOCTOU framing in the bootstrap is corrected by the advisor design decision in §4
of THIS brief — read §4 before §2 of the bootstrap.

### 3b. MIRROR refs (read in full before authoring §13)

- `crates/apub/activities/src/governance/inbox.rs` lines 479–495 — the two
  `OnceLock<Mutex<HashMap>>` statics and their key types. Current actor key:
  `(String, i64)` → target: `(String, [u8; 32], i64)` (domain + hash + bucket).
- `crates/apub/activities/src/governance/publish_trust_attestation.rs` lines 135–220 —
  the `check_per_actor_rate_limit` impl (config read, eviction, increment). Only callsite
  of `rate_per_actor_counts()` outside inbox.rs.
- `crates/apub/activities/src/governance/inbox.rs` lines 555–580 — per-peer Gate 4 rate
  check; mirror this synchronous-guard pattern for the per-domain count logic.
- `crates/apub/activities/Cargo.toml` — `sha2` is NOT yet a direct dep; workspace root
  `Cargo.toml` declares `sha2 = "0.10"`. Adding requires one line in activities
  `[dependencies]`: `sha2 = { workspace = true }`.
- `crates/apub/activities/src/governance/inbox.rs` lines 93–97 — existing `use` block;
  SHA-256 import (`use sha2::{Sha256, Digest};`) goes here.

### 3c. Lessons (mandatory)

- `.claude/lessons/feedback_features_full_workspace_only.md` — `sha2` has no feature gate;
  do not wrap the import in `#[cfg(feature = "full")]`.
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — run clippy after key type change;
  `[u8; 32]` implements `Hash + Eq` natively, no derive needed.
- `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — pre-push cargo-check
  constraint must appear in §4 of every impl-task brief this phase generates.

## 4. Design decision (ADVISOR — authoritative, do not re-litigate)

### Corrected TOCTOU analysis

The existing `check_per_actor_rate_limit` holds the `Mutex` guard for the entire
check-modify-evict block with **no `.await` inside** — there is no real race condition.
The two actual gaps are:

**Gap 1 — Cross-peer eviction unfairness.** At `MAX_PER_ACTOR_RATE_ENTRIES` (10,000) the
current eviction picks the globally-oldest entry. A hostile peer sending N distinct
`subject_url` values fills the map and forces eviction of legitimate entries from other
peers. Fix: evict the oldest entry belonging to the **domain with the highest entry count**.

**Gap 2 — O(URL-length) key inflation.** `subject_url` is attacker-controlled. Fix: hash
it to a fixed `[u8; 32]`.

### Chosen structure

Keep `OnceLock<Mutex<...>>`. Do NOT switch to DashMap (batch eviction becomes non-atomic
under per-key locking) or Postgres (serializes on DB pool; PRD specifies in-memory).

Replace `rate_per_actor_counts()` with `rate_per_actor_state()` returning a new struct:

```rust
struct ActorRateState {
    /// (peer_domain, sha256(subject_url), hour_bucket) → count
    counts: HashMap<(String, [u8; 32], i64), u32>,
    /// peer_domain → distinct-key count currently in `counts`
    per_domain: HashMap<String, usize>,
}
```

**Eviction (replaces the current `contains_key` / `iter().min_by_key` / `remove` block):**

1. After `retain()` prune, rebuild `per_domain` from `counts` (O(n), n ≤ 10,000, fully sync).
2. Check per-domain sub-cap: if `per_domain[new_key.peer_domain] >= MAX_PER_PEER_ACTOR_ENTRIES`
   and the new key is not already present → return `FederationActorRateLimitExceeded`
   immediately (one peer is saturating its slot).
3. If global `counts.len() >= MAX_PER_ACTOR_RATE_ENTRIES` and new key absent → find the
   domain with `argmax(per_domain)`, remove its `min_by_key(bucket)` entry.

New constant alongside `MAX_PER_ACTOR_RATE_ENTRIES` in `inbox.rs`:
```rust
const MAX_PER_PEER_ACTOR_ENTRIES: usize = 500;
```

**SHA-256 helper** (private fn in `publish_trust_attestation.rs`):
```rust
fn sha256_url(s: &str) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    Sha256::digest(s.as_bytes()).into()
}
```

## 5. Files changed

| File | Change |
|---|---|
| `crates/apub/activities/src/governance/inbox.rs` | Add `ActorRateState` struct; replace `rate_per_actor_counts()` with `rate_per_actor_state()`; add `MAX_PER_PEER_ACTOR_ENTRIES`; add `sha2` import |
| `crates/apub/activities/src/governance/publish_trust_attestation.rs` | Update `check_per_actor_rate_limit`: use `rate_per_actor_state()`, 3-tuple key, new eviction; add `sha256_url` helper |
| `crates/apub/activities/Cargo.toml` | Add `sha2 = { workspace = true }` |

## 6. Stop conditions (raise `kind: "blocker"` DQ)

- `rg "rate_per_actor_counts" crates/` returns more than 2 hits (one in inbox.rs, one in
  publish_trust_attestation.rs) — additional callsites change refactor scope.
- `sha2` is already a direct dep of `crates/apub/activities/Cargo.toml`.
- Any migration would be required (this task must produce zero migrations).

## 7. Constraints

- `[role:planning]` only — produce a plan file; do NOT write Rust.
- Plan §4 watchpoints MUST cite specific function names and line numbers.
- Plan §13 IMPLEMENT: exactly 3 files listed in §5 above.
- Plan §15 DoD: `cargo check --workspace --features full` + `cargo clippy --workspace
  --features full --no-deps -- -D warnings` + 22 lib tests still passing
  (`cargo test -p lemmy_apub_activities --lib`). No e2e required for this task.
- If planner adds an e2e story in a later sub-task, the impl-task brief MUST include
  the laptop-only e2e guard: worker stops after unit tests, writes
  `kind: "validate-pending-laptop-e2e"` DQ entry (commands array, branch, phase_task),
  and stops. Laptop advisor session runs e2e and mutates the DQ entry.
- Pre-push cargo-check constraint in every impl-task brief §4 (per
  `feedback_fix_impl_pre_push_cargo_check.md`).
