# Conformance-Audit — `v1-federation-inbound-c` Cohort 1 (prevention checkpoint)

**Date authored:** 2026-05-21
**Skill version:** 1.0.0 (axes 1-6, post-PR #141 merge `7cfc21c23` + L14 belt-and-braces `8b1aa3645`)
**Mode:** `file` × 2 — prevention checkpoint per `advisor-orchestrator.md` §3.1.1
**Inputs:**
- `crates/apub/activities/src/governance/inbox.rs` — Task 1 target (get_inbound_config_int @ line 421)
- `crates/apub/activities/src/governance/publish_trust_attestation.rs` — Task 2 target (actor_cap block @ line 139)
**Lane:** `phase-v1-federation-inbound-c` @ `05975274c` (lane worktree `brehon-fork-fed-in-c`)
**Trunk reference:** `origin/governance-v0` @ `8b1aa3645`

---

## TL;DR

**Cohort 1 edits are SAFE on all six axes for the code they introduce.** The 1-line `.order_by(governance_config::valid_from.desc())` insertions at Task 1 + Task 2 sites do not introduce any axis-1/2/3/4/5/6 divergence. Detection swept every site in both target files; the only axis-4 hits in either file are non-trust-boundary (mutex-poison recovery + local enum synthesis).

**One Tier-2 finding surfaces at the lane level, not at the brief level:** lane base `6dc489c9e` predates PR #141 merge `7cfc21c23`, so the lane's three federation `mod.rs` files lack the `#![deny(clippy::disallowed_methods)]` attribute that ships on `origin/governance-v0`. The lane is **25 commits behind trunk**. This does not weaken the Cohort 1 *edits*, but it does weaken the lane's Track-B Clippy enforcement until a forward-merge of `governance-v0` into `phase-v1-federation-inbound-c` lands.

## Risk-tier summary

| Tier | Count | Description |
|---|---|---|
| 1 (catch-fire) | 0 | None — no enforced contract is weakened by Cohort 1 edits |
| 2 (watch-item) | 1 | Lane base predates Clippy gate; forward-merge gov-v0 → phase branch recommended |
| 3 (stylistic / N-A) | 2 | Non-trust-boundary `.unwrap_or*` hits in Task-2 file (mutex recovery + local enum) |

---

## Per-axis findings

### Axis 1 — Conn-type / tx-boundary

**Sweep result:** `inbox.rs` contains five `.run_transaction(` call sites at lines 199, 307, 621, 688, 793 — none within Tasks 1+2 edit windows. `publish_trust_attestation.rs` contains zero `.run_transaction(` calls.

**Edit-site verdict:**
- Task 1 (inbox.rs:421-438) → `get_inbound_config_int` is a **read-only single-statement helper**, no transaction needed. Signature `pool: &mut DbPool<'_>` is correct for a read-only path (Lemmy convention).
- Task 2 (publish_trust_attestation.rs:139-152) → `actor_cap` block is a **read-only single-statement read**, no transaction needed. Acquires `&mut get_conn(pool)` from `context.pool()`, runs one `.first()` query, no writes.

**Risk tier:** N/A (no new function added; both edits insert one chain link inside an existing read path).

### Axis 2 — Append reborrow shape

**Sweep result:** `inbox.rs` has multiple `governance_log::append(` callers (lines 204, 219, 310, 324, 627, 698, 799, 813), all outside Tasks 1+2 edit windows. `publish_trust_attestation.rs` has a doc-comment mention at line 229 only — no live append callers.

**Edit-site verdict:** Neither task introduces a new `governance_log::append` call. The 1-line `.order_by()` insertion is a Diesel query method, not a log-append call.

**Risk tier:** N/A.

### Axis 3 — Trait-bound completeness

**Sweep result:** `inbox.rs` declares one generic async function: `pub(crate) async fn wrap_governance_inbound<'a, F, Fut, A>` at line 484 (outside Task 1's edit window at line 426-427). Its bounds were already set + verified by Phase-6 fix-impl-1. `publish_trust_attestation.rs` has zero `async fn .*<` matches.

**Edit-site verdict:** Neither task introduces a new generic function. No new bounds added; no bounds removed.

**Risk tier:** N/A.

### Axis 4 — Error idiom at trust boundary

**Sweep result on Task 1 file (`inbox.rs`):**
- Only `.unwrap_or` hit is at line 524: `i64::try_from(activity.payload_size_bytes()?).unwrap_or(i64::MAX)`. This is a numeric overflow-saturation pattern on the result of `i64::try_from(usize)`, NOT a trust-boundary `Option<String>` / `.domain()` / `.actor_id()` field access. Tier-3.

**Sweep result on Task 2 file (`publish_trust_attestation.rs`):**
- Line 160: `.unwrap_or_else(std::sync::PoisonError::into_inner)` — canonical mutex-recovery idiom mirroring `inbox.rs:629` and elsewhere. NOT trust-boundary. Tier-3.
- Line 350: `.unwrap_or("unknown")` on a `serde_json::Value::as_str()` of an **AttestationType enum** (locally generated outbound URL synthesis in `synthesise_object_id`). NOT a remote-actor field access; the enum is internally owned. Tier-3.
- Line 351: `.unwrap_or_else(|_| "unknown".to_string())` — same outbound synthesis path. Tier-3.

**Edit-site verdict:**
- Task 1 edit (line 426-427): no `.unwrap_or*`, `.expect()`, `.unwrap()` introduced.
- Task 2 edit (line 145-146): no `.unwrap_or*`, `.expect()`, `.unwrap()` introduced.

Both insertions are pure Diesel `.order_by(governance_config::valid_from.desc())` chain links between an existing `.select()` and an existing `.first()`. They do not interact with remote-actor data.

**Track-B (Clippy `#![deny(clippy::disallowed_methods)]`) — Tier-2 lane-level finding:**

The shipped Clippy enforcement on `origin/governance-v0` has three federation module roots carrying the deny attribute:

| File | gov-v0 line | gov-v0 sibling | Lane status |
|---|---|---|---|
| `crates/apub/activities/src/governance/mod.rs` | 22 | `#![deny(clippy::disallowed_methods)]` | **MISSING on lane** |
| `crates/api/api/src/governance/mod.rs` | 12 | `#![deny(clippy::disallowed_methods)]` | **MISSING on lane** |
| `crates/db_schema/src/source/governance/mod.rs` | 1 | `#![deny(clippy::disallowed_methods)]` | **MISSING on lane** |

`origin/governance-v0:clippy.toml` declares `disallowed-methods = [...]` (Option::unwrap_or_default + Result::unwrap_or_default) — also missing from lane (no `clippy.toml` at lane root).

`origin/governance-v0:Cargo.toml [workspace.lints.clippy]` sets `disallowed_methods = "allow"` (workspace-default ALLOW; per-module deny re-enables enforcement per rustc lint-precedence rule 4) — lane `Cargo.toml` lacks this stanza.

**Lane is 25 commits behind `origin/governance-v0`** (lane base `6dc489c9e` predates PR #141 merge `7cfc21c23`). The Clippy gate IS shipped on trunk; the lane will inherit it on its next forward-merge of `governance-v0` → `phase-v1-federation-inbound-c`. Until then, the per-module Clippy deny is unenforced on lane builds.

**Impact on Cohort 1:** None at code level — Tasks 1+2 add no `.unwrap_or_default()` so axis-4 Track-B would not fire anyway. Risk is purely lane-staleness: if subsequent fed-in-c tasks (Tasks 3-5) DO touch trust-boundary error idioms, they would compile on the lane without the deny safety net.

**Risk tier:** 2 (watch-item: forward-merge `origin/governance-v0` into `phase-v1-federation-inbound-c` before queueing any task that touches trust-boundary error idioms).

### Axis 5 — Conn acquisition idiom

**Sweep result:**
- `inbox.rs:422` (Task 1 edit site, line above the `.order_by` insertion): `let conn = &mut get_conn(pool).await?;` — canonical form.
- `publish_trust_attestation.rs:141` (Task 2 edit site, inside the `actor_cap` block above the `.order_by` insertion): `let conn = &mut get_conn(pool).await?;` — canonical form.

Both edit sites already use the canonical `let conn = &mut get_conn(pool).await?;` pattern. The 1-line `.order_by()` insertion does NOT change conn acquisition.

**Risk tier:** N/A (existing canonical pattern preserved).

### Axis 6 — ADR-015 pseudonym handling for remote actors

**Sweep result:**
- `inbox.rs`: line 125 is a doc-comment mention only (`/// actor_pseudonym = None because`). Zero call sites.
- `publish_trust_attestation.rs`: zero `actor_pseudonym` matches.

Neither task introduces a new DB-write helper or struct initializer carrying `actor_pseudonym`. Both edits are read-side `governance_config` query chains.

**Risk tier:** N/A.

---

## Canonical sibling — read-side `order_by(valid_from.desc())` enforcement

The MIRROR pattern declared in the briefs (plan §10.1) — `fetch_value_at_scope` in `crates/api/api/src/governance/config.rs:734-769` — verified at lane HEAD `05975274c`:

```rust
// crates/api/api/src/governance/config.rs:734-758 (verbatim, lane HEAD)
async fn fetch_value_at_scope(
  pool: &mut DbPool<'_>,
  scope_str: Cow<'static, str>,
  key: &str,
) -> LemmyResult<Option<CachedValue>> {
  let conn = &mut get_conn(pool).await?;

  // governance_config is append-only with multiple rows per (scope, key)
  // keyed by valid_from. ORDER BY valid_from DESC + LIMIT 1 (.first) is
  // load-bearing — without it Postgres returns arbitrary order and reads
  // can return stale seeded rows instead of admin_set_config writes.
  // Regression history: commit 8e3bba1 dropped the governance_config_current
  // view; this code path needs to do the latest-wins ordering itself.
  let row: Option<ConfigRow> = governance_config::table
    .filter(governance_config::scope.eq(scope_str.into_owned()))
    .filter(governance_config::key.eq(key))
    .select((...))
    .order_by(governance_config::valid_from.desc())
    .first::<ConfigRow>(conn)
    .await
    .optional()?;
```

Tasks 1+2 mirror this `.order_by(valid_from.desc())` placement (between `.select(...)` and `.first(...)`) verbatim. **The Cohort 1 edits ARE the alignment to this canonical sibling** — by definition this is not a divergence; it is conformance.

## Schema sanity

Per Task 1 + Task 2 brief preconditions:

```
$ grep -c governance_config_current crates/db_schema_file/src/schema.rs
0
```

The intentionally-absent `governance_config_current` view is confirmed absent. Both briefs' "no view usage; base-table + `.order_by`" precondition holds.

## Brief amendments — none recommended

Per `advisor-orchestrator.md` §3.1.1: "Tier-1 findings fold into brief §3 / §4 before clarify-DQ entries". This audit produced **zero Tier-1 findings**, so no brief §3 (Required reading) or §4 (Constraints) amendments are needed for Cohort 1.

The Tier-2 lane-staleness finding is **post-cohort-impl scope** (would land before Task 3 dispatches) — recommended action is a forward-merge of `origin/governance-v0` into `phase-v1-federation-inbound-c` after Cohort 1 finalize-merges, before Cohort 2 (Task 3 e2e) dispatches. Surfacing here for advisor awareness; no action required pre-Cohort-1.

## Ground-truth journal (populated post-§15 + post-merge)

`ground_truth_compile_caught[]` and `ground_truth_runtime[]` are empty at audit time — to be populated:

1. **Post-§15** — after advisor-laptop runs DQ #328 + #329 commands (cargo-check + cargo-clippy), record any `error[E*]` or clippy warning that matches an axis prediction.
2. **Post-CR triage on the eventual fed-in-c PR** — record any sibling-conformance finding flagged by CodeRabbit / Claude.
3. **Post-merge to gov-v0 + 30-day window** — record any bug fix commit citing a fed-in-c regression touching these files.

If post-§15 surfaces an `error[E*]` whose axis matches a `predictions[]` entry: true positive (TP). If no predictions matched and a real error fires: false negative (FN, post-§15 ground truth feeds the false-negative side of precision/recall). If a prediction was made but no compiler / CR / runtime evidence fires within 30 days: false positive (FP, move to `false_positives[]`).

## Validation evidence

- Schema parse: PASS (3 predictions, all evidence strings ≤120 chars, required keys present)
- Read-only invariant: PASS (no `cargo` invocation, no `Edit/Write` outside the two declared output paths)
- Axis sub-files: read all 6 before sweep
- Lane HEAD: `05975274c`
- Trunk HEAD: `8b1aa3645`

---

_Audit author: advisor session in canonical `brehon-fork` (governance-v0 meta-edit lane). Lane worktree consulted read-only for the sweep — no lane-side writes per `.claude/rules/multi-lane-worktree.md` hard refusal #2._
