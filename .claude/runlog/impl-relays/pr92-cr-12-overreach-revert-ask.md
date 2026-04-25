# PR #92 — cr-12 overreach revert (BM → impl relay)

**Date:** 2026-04-24
**Author:** BM session (governance-v0 worktree)
**Target:** impl session (brehon-fork-phase-v1-JM-a worktree)
**PR:** https://github.com/barrie-cork/lemmy/pull/92
**HEAD:** `2974f1b0a` (after cr-12/13/14/15 batch @ `43046e86d` + cr-16 @ `2974f1b0a`)

## The problem

CI e2e on HEAD `2974f1b0a` **FAILED** — 46 passed / 1 failed / 3 ignored.

**Failing test:** `admin_set_config_community_scope_by_moderator` (`crates/server/tests/e2e.rs:5113`)
**Error:** `LemmyError { message: NotAnAdmin, caller: crates/api/api/src/governance/admin_config.rs:453 }`
**CI run:** https://github.com/barrie-cork/lemmy/actions/runs/24907374882/job/72940077616

### Root cause

cr-12's fix was **over-applied** to 37 jury metadata entries. CR's original finding named **exactly one key**:

> cr-16 original text: `jury.max_concurrent_assignments_per_juror_total` scope is `ConfigScope::Both`; should be `ConfigScope::Instance` — cross-community spread bypasses global cap.

That rationale ("cross-community spread bypasses global cap") applies ONLY to per-juror cap keys, NOT to:
- `jury.quorum` (per-community quorum thresholds are legitimate)
- `jury.panel_size` (per-community panel sizes are legitimate)
- `jury.threshold_fraction.*` (per-community vote thresholds are legitimate)
- `jury.severity_thresholds.*`, `jury.diversity_constraints_enabled`, etc.

The failing test at `e2e.rs:5150` specifically picks `jury.quorum` as the moderator-writable `Both`-scope key. Its docblock at `e2e.rs:5106-5111` even documents why that key was chosen:

> NOTE: v1-AD-b plan §11 line 1101 originally named `liability.regular_multiplier` here, but that key is declared `ConfigScope::Instance` in `config.rs:1141-1152` — not `Both`. Swapped to `jury.quorum` (Int, range 1-21, `ConfigScope::Both`) which actually exercises the moderator-write path.

cr-12 flipped `jury.quorum` (and 35 other legitimate-`Both` keys) to `Instance`, breaking the test.

## What the fix needs to do

**Revert 36 of the 37 cr-12 scope flips** — restore `ConfigScope::Instance → ConfigScope::Both` on every cr-12 flip **EXCEPT** `jury.max_concurrent_assignments_per_juror_total`. That one legitimately stays `Instance` (CR's real rationale applies).

### Exception to keep as Instance (1)

| Line (HEAD `2974f1b0a`) | Key |
|---|---|
| 2193 (scope line ~2197) | `jury.max_concurrent_assignments_per_juror_total` — CR's original rationale actually applies here |

### Lines to revert Instance → Both (36)

These are the cr-12-flipped `scope:` lines whose keys should return to `Both`. Line numbers are from HEAD `2974f1b0a`:

| Line | Key |
|---|---|
| 1237 | `jury.panel_size` |
| 1249 | `jury.quorum` ← **the one the failing test uses** |
| 1489 | `report.case_threshold_micros` (verify — cr-12 included it; impl may judge this one) |
| 1610 | `jury.severity_thresholds.minor` |
| 1622 | `jury.severity_thresholds.moderate` |
| 1634 | `jury.severity_thresholds.severe` |
| 1646 | `jury.diversity_constraints_enabled` |
| 1670 | `jury.appeal_panel_size_increase` |
| 1874 | `rule_set.auto_carry_in_flight_cases` (verify — cr-12 included it; impl may judge) |
| 1898 | `rule_set.version_propagation_delay_hours` (verify — as above) |
| 1938 | `jury.panel_size.regular.minor` |
| 1950 | `jury.panel_size.regular.moderate` |
| 1962 | `jury.panel_size.regular.severe` |
| 1974 | `jury.panel_size.founder.minor` |
| 1986 | `jury.panel_size.founder.moderate` |
| 1998 | `jury.panel_size.founder.severe` |
| 2010 | `jury.panel_size.probation.minor` |
| 2022 | `jury.panel_size.probation.moderate` |
| 2034 | `jury.panel_size.probation.severe` |
| 2047 | `jury.quorum_fraction.minor` |
| 2059 | `jury.quorum_fraction.moderate` |
| 2071 | `jury.quorum_fraction.severe` |
| 2084 | `jury.threshold_fraction.minor` |
| 2096 | `jury.threshold_fraction.moderate` |
| 2108 | `jury.threshold_fraction.severe` |
| 2121 | `jury.constraints.no_majority_from_same_sponsor_cluster` |
| 2133 | `jury.constraints.geographic_diversity_preferred` |
| 2145 | `jury.constraints.no_recent_juror_repeat` |
| 2157 | `jury.constraints.juror_cooldown_days` |
| 2169 | `jury.constraints.no_shared_endorsement_chain` |
| 2182 | (verify — cr-12 flipped; impl judge) |
| 2208 | `appeal.panel_size_multiplier` (verify — if pre-cr-12 was `Both`, revert; if always `Instance`, leave) |
| 2220 | `appeal.panel_size_floor_increment` (verify — as above) |
| 2232 | `appeal.threshold_tier_bump` (verify — as above) |
| 2244 | `appeal.window_days` (verify — as above) |
| 2256 | `appeal.auto_select_on_appeal_acceptance` (verify — as above) |

**Authoritative line list:** run `git show 43046e86d -- crates/api/api/src/governance/config.rs` and match every `- scope: ConfigScope::Both` / `+ scope: ConfigScope::Instance` hunk. cr-12 claimed "37 jury/* metadata entries" — the one to keep as `Instance` is the one at line 2193 (`jury.max_concurrent_assignments_per_juror_total`); revert all others.

### Restore the `Both` variant

cr-12 added `#[expect(dead_code)]` to the `Both` variant in the `ConfigScope` enum. With `Both` used again on 36 keys, that attribute must be removed. `Community` variant stays `#[expect(dead_code)]` — it's still unused.

In `crates/api/api/src/governance/config.rs` around line 79-86:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigScope {
  Instance,
  #[expect(dead_code, reason = "forward-compat for v1-AD community-scoped config overrides")]
  Community,
  Both,  // ← remove the #[expect(dead_code)] attribute above this line
}
```

Expected post-revert shape:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigScope {
  Instance,
  #[expect(dead_code, reason = "forward-compat for v1-AD community-scoped config overrides")]
  Community,
  Both,
}
```

## Second-order judgment for impl

cr-2 (from poll #4 notes) flagged a separate metadata-description inconsistency at ~line 1270 on `jury.max_concurrent_assignments` (the older v0 key — NOT the same as the v1 `max_concurrent_assignments_per_juror_total`). That key was **not** flipped by cr-12; it was already `Both`. If impl chooses to ALSO flip `jury.max_concurrent_assignments` (line 1271, scope ~1275) to `Instance` on the same "per-juror cap" rationale, that's a consistency win — but risks breaking another test or contradicting a PRD decision.

**Minimal-risk default: leave `jury.max_concurrent_assignments` as `Both`.** Let CR flag it separately if it's a concern. This revert's scope is "undo cr-12 overreach," not "extend cr-12's logic."

## How to verify

Before pushing:

1. **Compile check:**
   ```
   cmd //c scripts\brehon\cargo-check.bat --workspace --features full > .claude/revert-check.log 2>&1
   echo "exit: $?"
   ```
2. **Test compile check** (ensure e2e target still builds):
   ```
   cmd //c scripts\brehon\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/revert-test-compile.log 2>&1
   echo "exit: $?"
   ```
3. **Run just the previously-failing test locally** (Docker must be up per pre-phase-harness-audit.md Probe 0):
   ```
   cmd //c scripts\brehon\cargo-test.bat --test e2e -p lemmy_server admin_set_config_community_scope_by_moderator > .claude/revert-failing-test.log 2>&1
   echo "exit: $?"
   tail -30 .claude/revert-failing-test.log
   ```
   Expected: 1 passed. If still failing, root cause is NOT cr-12 overreach — report back.

## Suggested commit shape

**Subject:** `fix(v1-JM-a): cr-12 revert overreach — keep only max_concurrent_assignments_per_juror_total as Instance`

**Body:**

```
cr-12 was over-applied to 37 jury/* metadata entries in 43046e86d.
CR's original finding named exactly one key —
jury.max_concurrent_assignments_per_juror_total — and the
"cross-community spread bypasses global cap" rationale applies only
to per-juror caps, not to quorum/panel_size/threshold_fraction/etc.

This revert restores ConfigScope::Both on the 36 keys that legitimately
support per-community overrides (notably jury.quorum, the key that
admin_set_config_community_scope_by_moderator in e2e.rs:5150 relies on).
It leaves jury.max_concurrent_assignments_per_juror_total as
ConfigScope::Instance — that one stays as cr-12 applied it.

#[expect(dead_code)] removed from ConfigScope::Both (variant used again
on 36 keys); retained on ConfigScope::Community (still unused).

CI e2e on HEAD 2974f1b0a caught the regression:
https://github.com/barrie-cork/lemmy/actions/runs/24907374882/job/72940077616
```

## BM pipeline after impl pushes

1. `/bm-poll-cr 92` — expect CR to either confirm addressed (ideal) or post a follow-up (minor-severity clarification on scope-semantic decisions if CR wants to re-check the line list)
2. `/bm-triage 92` — promote or handle
3. **CI e2e must pass** — I won't proceed to merge until `governance e2e` returns `SUCCESS`
4. `/bm-merge 92`
