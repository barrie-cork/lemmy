# Plan: v1-quality-r2 — PR #155 carry-forward bundle (DQ duration lint + fixtures doc audit + EnvVarGuard retrofit; C3 helper extraction deferred)

## 1. Summary

Retire **4 of 5** PR #155 carry-forward issues filed during the v1-RT-r3 BM triage (2026-05-28): #157 (DQ negative-duration lint + bulk sweep + precheck wiring), #156 (fixtures-module process-env safety doc audit), #159 (boot_context env mutations → EnvVarGuard), #160 (LEMMY_DATABASE_URL setter sites → EnvVarGuard, 14 sites across 13 fixtures modules). Issue **#158** (emit_reputation_event shared helper extraction) is **DEFERRED** per the premature-DRY gate (WP-2): no near-term 3rd consumer materialises in the roadmap (federation_inbound lane is `done`); a `kind: "log"` DQ on this phase records the deferral and #158 stays open with a "blocked-on-3rd-consumer" comment. Headline acceptance: workspace cargo gates exit 0, full e2e suite passes locally, `dq-lint-durations.sh` exits 0 on trunk + non-zero on a synthetic back-dated entry, every `*_fixtures` module's doc-comment cites the `--test-threads=1` constraint, and every `boot_context` env mutation + every `LEMMY_DATABASE_URL` setter site in `e2e.rs` is wrapped by the `EnvVarGuard` RAII pattern shipped in v1-RT-r3.

## 2. Source

- Brief: `.claude/PRPs/briefs/v1-quality-r2-planning-1.md` (committed at `27c235a20`).
- PR #155 findings YAML: `.claude/PRPs/reviews/pr-155-findings.yaml` (rows `cr-2`, `cr-4`, `cr-7`, `cp-4`, `cp-5`).
- Canonical sibling plans (per `feedback_read_canonical_before_writing_spec.md`): `.claude/PRPs/plans/v1-quality-r1.plan.md` (prior quality bundle on this lane) + `.claude/PRPs/plans/v1-RT-r3.plan.md` (introduced `EnvVarGuard`, `v1_rt_r3_fixtures`, `emit_reputation_event_local`).
- Roadmap: `.claude/PRPs/v1-roadmap.json` `lanes.quality.sub_phases.v1-quality-r2` (status was `unstarted` at brief-author time; this plan flips it `in_flight`).
- Lessons that materially shaped the plan:
  - `feedback_fix_impl_enumerate_all_callsites.md` — drove the planner-side re-enumeration of `boot_context` + `env::set_var` + `mod *_fixtures` counts (recorded in §3).
  - `feedback_fix_impl_pre_locate_e2e_anchors.md` — every e2e.rs-targeting impl-task brief MUST paste verbatim `old_string`/`new_string` anchors.
  - `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` — bound for any new test fn added under this plan.
  - `feedback_principles_not_rules.md` — drove the WP-2 DEFER decision (premature DRY footgun).
  - `feedback_complexity_score_pre_split.md` — drove the §5.1 split-DQ.
  - `feedback_validate_pending_laptop_must_use_wrapper.md` + `feedback_windows_e2e_requires_bat_wrapper.md` + `feedback_laptop_default_for_validate_pending.md` — Windows wrapper + laptop-default DoD discipline.
  - `feedback_clippy_test_style.md` — `?` + `LemmyResult<()>` style for any new test fn.
- ADRs: ADR-013 (illegal-content `CaseStatus::EmergencyRemove` — relevant only as boundary of #158's deferral rationale; this plan does NOT modify ADR-013 surface).

## 3. Problem statement

PR #155 (v1-RT-r3) shipped 4 net-new defect classes the BM triage filed as carry-forwards rather than fix-in-PR; each is in a different defect class and the bundle deliberately consolidates them to amortise one CR cycle + one local e2e gate (~26 min). The four addressed in this phase:

1. **DQ negative-duration data corruption risk (#157).** Four DQ entries (3999, 4007, 4016, 4035 per the issue body) carry `resolved_at < timestamp`, producing negative durations in any time-series consumer. No script gates this on write; the issue can recur on every `dq-v3-append-fragment.sh` invocation that lets the caller pre-compute `resolved_at` before `timestamp`. Tied to **Task 1**.
2. **Fixtures-module process-env safety doc-comments (#156).** The `v1_rt_r3_fixtures` module documents `// SAFETY: tests run with --test-threads=1; no concurrent env mutation.` on every `env::set_var` call, but most older fixtures modules (13 of 14 sibling `mod *_fixtures` blocks) omit the `--test-threads=1` constraint citation entirely. A contributor reading those older modules cannot tell that the safety claim depends on a Cargo-runner flag; the convention is enforced by prose, not the compiler. Tied to **Task 3**.
3. **`boot_context` env-leak via early-`?` (#159).** `v1_rt_r3_fixtures::boot_context()` (e2e.rs:17414) mutates three env vars (`LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS`, `GOVERNANCE_LOG_SIGNING_KEY`, `LEMMY_DATABASE_URL` at lines 17424/17425/17431) using raw `std::env::set_var` instead of the `EnvVarGuard` pattern that the same module ships at line 17179. If `start_postgres()` / `apply_all_schema()` / `FederationConfig::builder()` returns an error before `boot_context()` returns, the env vars are not restored — the next test in the same process sees the leaked state. Tied to **Task 4**.
4. **Legacy `LEMMY_DATABASE_URL` setter env-leak (#160).** 14 callsites across 13 fixtures modules set `LEMMY_DATABASE_URL` via raw `std::env::set_var` without `EnvVarGuard` (lines 806, 2544, 3324, 4096, 4443, 4774, 4907, 5044, 5093, 5671, 5874, 6137, 16752, 17431). Same defect class as #159, broader surface. Companion to #159 — shipping only #159 is "half a fix". Tied to **Task 5**.

The fifth issue (#158, `emit_reputation_event` shared helper) is a code-duplication finding the issue body itself flags as "wait until a third consumer materialises before introducing the abstraction (premature DRY is its own footgun)" — Task 2 files the deferral DQ, NOT an extraction.

## 4. Solution statement

Five impl tasks plus Task 0 (harness audit) and a retro task. Tasks 1–2 are pure metadata work (scripts/JSON/DQ), Tasks 3–5 are e2e.rs-confined edits in three disjoint shape classes (doc-comment headers, single-module body refactor, scattered single-line wraps). All five impl tasks ship in one PR against `governance-v0` (Option A per WP-Cluster — callsite enumeration at threshold, not exceeding).

Task shape map:

| Task | Cluster | Surface | Edit shape | Risk |
|---|---|---|---|---|
| T0 | — | (harness audit only) | n/a | — |
| T1 | C2 | `scripts/brehon/dq-lint-durations.sh` (new), `scripts/brehon/precheck.sh` (new), `.claude/decision-queue.json` (bulk sweep edit) | Author shell scripts + run sweep | low |
| T2 | C3 | `.claude/decision-queue.json` (append deferral entry via `dq-v3-append-fragment.sh`) | Single DQ append + close issue comment | low |
| T3 | C1 | `crates/server/tests/e2e.rs` (14 module-header doc-comments) | 14 mechanical doc-comment header rewrites; all in `//! ` lines at the top of each `mod *_fixtures` block | low (doc-only, no compile-relevant change) |
| T4 | C4-A | `crates/server/tests/e2e.rs` (v1_rt_r3_fixtures module body) | Hoist `EnvVarGuard` to test-crate top-level (pub-visible to all sibling fixtures); refactor `boot_context()` return type to thread guards; update 10 same-module callsites | medium (signature change in one well-bounded module; pre-located anchors mandatory) |
| T5 | C4-B | `crates/server/tests/e2e.rs` (LEMMY_DATABASE_URL setter sites in 13 older fixtures modules) | 13 scattered `let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);` wraps replacing `unsafe { std::env::set_var("LEMMY_DATABASE_URL", &db_url); }` blocks (site 14 covered by T4) | medium (13 anchors across 13 modules; pre-located anchors mandatory) |
| T6 | — | `.claude/PRPs/reports/v1-quality-r2-retro.md` (new) | Retro authorship per `feedback_retro_not_report.md` | low |

The `EnvVarGuard` hoist (T4) is the load-bearing structural change: the existing `struct EnvVarGuard` at e2e.rs:17179 lives inside the `v1_rt_r3_fixtures` module and is not `pub`, so T5 cannot reach it without either duplicating the struct in each sibling fixtures module (~13 copies) or hoisting it to the test-crate root where every `mod *_fixtures` block can `use super::EnvVarGuard;`. The hoist is mechanical (move the struct + 2 impl blocks ~30 lines up to the file root) and lets T5 be a uniform mechanical sweep.

The §13 task order serialises every e2e.rs-touching task because the cohort YAML overlap check rejects `[P]` across same-file tasks (per `.claude/rules/advisor-orchestrator.md` §4.2). T1 + T2 BOTH modify `.claude/decision-queue.json` (T1 bulk sweep + T2 append) → serial. No `[P]` markers in this plan.

## 5. Metadata

- **Phase:** `v1-quality-r2`
- **Branch:** `phase-v1-quality-r2` (cut from `governance-v0` at `1edb8b94c` per BM-cut log at `d5f0114eb`)
- **Target impl-task model:** `sonnet-4-6` (default)
- **Estimated tasks:** 7 (Task 0 pre-flight + 5 impl tasks + 1 retro)
- **Estimated cargo budget:** N/A (validate-pending-laptop; cargo runs on laptop ~6 GB peak per `cargo test --workspace --features full`; Shape G suspended through 2026-06-01 per `project_shape_g_suspended_2026_05_16.md`).
- **Forbidden-window applicability:** non-binding for impl-task dispatch under validate-pending-laptop (cargo runs on laptop, not EliteDesk).
- **Complexity score:** **10/10** (over Sonnet threshold; split-DQ filed — see §5.1; planner recommends `proceed` with v1-RT-r3 precedent).

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md` + `plan.template.md` §5.1. Counts are mechanical from the §13 task YAML:

| Factor | Weight | This plan | Source |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **0** | 5 impl tasks (T1, T2, T3, T4, T5); count excludes Task 0 (pre-flight) and the retro task; 5 − 5 = 0. |
| Migrations touched | +2 each | **0** | This plan touches zero `migrations/**` files. |
| Crates touched | +1 each | **+1** | `crates/server/tests/e2e.rs` (1 crate: `lemmy_server`). `scripts/brehon/*` and `.claude/decision-queue.json` are not Cargo crates. |
| `crates/server/tests/e2e.rs` edits | +3 each | **+9** | Tasks 3, 4, 5 each modify `crates/server/tests/e2e.rs`. 3 × +3 = +9. |
| New ADR-affecting decisions | +2 each | **0** | This plan does not supersede any entry in `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`. |
| Cargo budget peak above 6 GB | +1 per GB | **0** | Validate-pending-laptop, ~6 GB peak; +0 above the 6 GB floor. |
| **Total** | — | **10** | Threshold for split-DQ (Sonnet target): `> 8` — **FIRES**. |

**Score 10 > 8 (Sonnet threshold). Split-DQ to be pre-seeded as a `pending` planner entry** (per `.claude/agents/planning.md` "§5 complexity score + split threshold"). Question: "Complexity score 10 exceeds Sonnet threshold 8 (target model: sonnet-4-6) — split `v1-quality-r2` into `v1-quality-r2a` (C2 tooling + C3 deferral) + `v1-quality-r2b` (C1 doc audit + C4 EnvVarGuard retrofit), or proceed with prior-Sonnet-phase precedent (v1-RT-r3 shipped 4 impl tasks with heavy e2e.rs edits successfully under Sonnet 4.6)?"

Planner recommendation in the DQ `context`: **proceed**. Reasoning: (a) the 3 e2e.rs tasks decompose into clearly distinct edit shapes (doc-only header sweep, single-module body refactor with pre-located anchors, scattered single-line wraps with pre-located anchors), each well within Sonnet's e2e-edit envelope per v1-RT-r3 precedent; (b) splitting would double the CR cycle and e2e gate cost (~52 min vs ~26 min) for no Junior-worker benefit (each task is independently dispatched); (c) the phase branch `phase-v1-quality-r2` was already cut at `d5f0114eb` — splitting requires re-cutting two new branches and adds bm-cut + handover overhead.

### 5.2 Per-task complexity ceiling

Target model is Sonnet, so the Sonnet ceiling applies (per template §5.2 / planning.md §5b):

- `count(union(creates, modifies)) ≤ 4` files per task
- `count(distinct crates/<X>/ prefixes in union(creates, modifies)) ≤ 2` crates per task
- `crates/server/tests/e2e.rs` bundling allowed (Sonnet only)

Verified at §13 task YAML walk-time: every task satisfies. T1 = 3 files (`scripts/brehon/dq-lint-durations.sh`, `scripts/brehon/precheck.sh`, `.claude/decision-queue.json`), 0 crates. T2 = 1 file, 0 crates. T3 = 1 file (e2e.rs), 1 crate (lemmy_server). T4 = 1 file (e2e.rs), 1 crate. T5 = 1 file (e2e.rs), 1 crate. T6 = 1 file (retro report), 0 crates.

## 6. Relationship to other v1-quality-* sub-phases

- **Predecessor (merged):** `v1-quality-r1` (PR #145 `eec20a102`, merged 2026-05-22). No carry-forward overlap with this phase.
- **Triggering phase (merged):** `v1-RT-r3` (PR #155 `5ebd8ae23`, merged 2026-05-28). The five issues this phase addresses were filed during PR #155's BM triage as carry-forwards.
- **Successor (likely):** `v1-quality-r3` — possible follow-on for: (a) `emit_reputation_event` helper extraction if a 3rd consumer materialises; (b) C4 follow-on if `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` + `GOVERNANCE_LOG_SIGNING_KEY` env mutations across the same 13 fixtures modules turn out to be similarly leak-prone (28 additional sites; see §19).
- **Concurrent lane:** `v1-redaction-r1` (in-flight 2026-05-28). Zero file overlap with this lane. Two concurrent lanes on the daemon `.git/` → below the cohort-≥3 `.git/index.lock` threshold per `feedback_cohort_shared_git_index_contention.md`.

## 7. Preflight guardrails inherited from prior phases

- **R1** (per `feedback_clippy_test_style.md`): every i32 ↔ i64 comparison uses `i64::from(...)`, never `as` cast. No new arithmetic in this plan; applies only if any new test fn appears.
- **R5** (per JM-b retro): Task 0 enumerates ALL probes explicitly (Docker preflight + 4 wrapper probes per `.claude/rules/pre-phase-harness-audit.md` + clippy baseline capture + brief re-enumeration counts).
- **R6** (per JM-b retro): all `cargo clippy` invocations use `--no-deps --features full -- -D warnings`. Verified in §15.
- **R7** (per JM-b retro): per task that touches a struct or signature, `cargo test --no-run -p lemmy_server --test e2e --features full` runs as a per-task gate. T4's `boot_context()` return-type change qualifies → R7 applies to T4.
- **R8** (per `feedback_features_full_p_crate_incompatible.md`): never combine `-p <crate>` with `--features full` except where the crate defines a `full` feature.
- **R9** (per `feedback_validate_pending_laptop_must_use_wrapper.md` + `feedback_windows_e2e_requires_bat_wrapper.md`): every cargo gate invokes the `scripts/brehon/cargo-*.bat` (Windows) or `.sh` (Linux) wrapper.
- **R10** (per `cargo-output-capture.md` + `no-cargo-output-paste.md`): all cargo invocations redirect to `.claude/PRPs/debug/<phase>-<task>-<verb>.log` with `> log 2>&1`; exit-code preservation via `$?`; never piped through `tail`/`head`/`grep`.
- **R11** (per `feedback_fix_impl_pre_locate_e2e_anchors.md`): every impl-task brief whose `modifies:` array includes `crates/server/tests/e2e.rs` MUST paste verbatim `old_string`/`new_string` anchors for every Edit the worker will perform. T3/T4/T5 briefs honour this.

## 8. Flow design

```
Task 0 (harness audit)
  │
  ▼
Task 1 (C2: dq-lint-durations.sh + precheck.sh + bulk sweep)
  │  modifies .claude/decision-queue.json, scripts/brehon/*
  ▼
Task 2 (C3: deferral DQ for #158)
  │  modifies .claude/decision-queue.json (append-only)
  ▼
Task 3 (C1: 14 fixtures-module doc-comment header rewrites)
  │  modifies crates/server/tests/e2e.rs (doc-only)
  ▼
Task 4 (C4-A: EnvVarGuard hoist + boot_context refactor + 10 callsite updates in v1_rt_r3_fixtures)
  │  modifies crates/server/tests/e2e.rs (structural)
  ▼
Task 5 (C4-B: 13 LEMMY_DATABASE_URL setter sites wrapped with EnvVarGuard across 13 fixtures modules)
  │  modifies crates/server/tests/e2e.rs (mechanical, scattered)
  ▼
Task 6 (Retro)
  │  creates .claude/PRPs/reports/v1-quality-r2-retro.md
  ▼
bm-pr / bm-poll-cr / bm-triage / bm-merge (BM session)
```

Logical data-flow note for T4/T5: the `EnvVarGuard` RAII pattern stores `(key, prev_value)` and restores on `Drop`. Every `let _g_db_url = EnvVarGuard::set(...);` binding MUST live in the same scope as the operation that depends on the env var; an early `?` from a fallible call between `set` and `drop` triggers `Drop::drop` and restores the prior value before the error propagates. T4's `boot_context()` MUST return the guards to the test fn (otherwise they drop at function exit and the env vars revert before the test actually uses them).

## 9. Mandatory reading

For the impl-task subagent before its first Edit:

**Schema/type definitions:**
- `crates/server/tests/e2e.rs:17174-17205` — `EnvVarGuard` struct + impl + Drop. Canonical reference for the RAII pattern T4/T5 replicate.
- `crates/server/tests/e2e.rs:17414-17461` — `boot_context()` definition. T4 refactor target.
- `crates/server/tests/e2e.rs:17576, 17631, 17671, 17715` — existing `EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1")` usages.

**Existing patterns:**
- `crates/server/tests/e2e.rs:115-200` — `mod governance_fixtures` opening; T3 audit baseline.
- `crates/server/tests/e2e.rs:17101-17173` — `mod v1_rt_r3_fixtures` opening (the only module today shipping a `--test-threads=1` SAFETY comment shape).

**Lessons (gate impl-task brief authoring):**
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — T3/T4/T5 briefs paste verbatim anchors.
- `feedback_fix_impl_enumerate_all_callsites.md` — planner already enumerated (14 modules, 11 boot_context refs, 14 LEMMY_DATABASE_URL setter sites).
- `feedback_lemmy_error_no_std_error.md` Case A — outer `LemmyResult<()>` for any new test (none planned; refactor-only).
- `feedback_async_pool_test_pattern.md` — applies to any new test fn.
- `feedback_validate_pending_laptop_must_use_wrapper.md` — wrapper discipline.
- `feedback_windows_e2e_requires_bat_wrapper.md` — `cmd //c "scripts\\brehon\\cargo-test.bat ..."` for libpq discovery.
- `feedback_principles_not_rules.md` — premature-DRY footgun (WP-2 justification).
- `.claude/rules/decision-queue.md` Hard refusals #1, #5, #8, #9 — bind every DQ write under this plan.

## 10. Patterns to mirror

### 10.1 EnvVarGuard RAII (canonical: e2e.rs:17174-17205)

**Mirror:** `crates/server/tests/e2e.rs:17174-17205`

```rust
struct EnvVarGuard {
  key: &'static str,
  prev: Option<String>,
}

impl EnvVarGuard {
  fn set(key: &'static str, value: &str) -> Self {
    let prev = std::env::var(key).ok();
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    unsafe {
      std::env::set_var(key, value);
    }
    Self { key, prev }
  }
}

impl Drop for EnvVarGuard {
  fn drop(&mut self) {
    // SAFETY: same justification — single-threaded test runner.
    unsafe {
      match &self.prev {
        Some(prev) => std::env::set_var(self.key, prev),
        None => std::env::remove_var(self.key),
      }
    }
  }
}
```

T4 hoists this verbatim from inside `mod v1_rt_r3_fixtures` to the test-crate root. Hoisted to the parent scope, each sibling module gets it via `use super::EnvVarGuard;`. `pub` keyword optional but harmless.

### 10.2 EnvVarGuard usage at callsite (canonical: e2e.rs:17576)

**Mirror:** `crates/server/tests/e2e.rs:17576`

```rust
let _guard = EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1");
```

The `_guard` binding (NOT `_`) is load-bearing: `let _ = EnvVarGuard::set(...)` drops the guard immediately. Always bind with a named `_guard` (or `_<descriptor>`).

T5 replaces every:
```rust
// SAFETY: tests run with --test-threads=1.
unsafe {
  std::env::set_var("LEMMY_DATABASE_URL", &db_url);
}
```
with:
```rust
let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
```

The SAFETY comment is REMOVED at the callsite; the justification now lives at e2e.rs:17187.

### 10.3 boot_context return-type extension (T4 target)

**Mirror:** `crates/server/tests/e2e.rs:17414-17461` (current) → extended return type after T4.

Post-T4 (target):
```rust
async fn boot_context() -> LemmyResult<(
  testcontainers::ContainerAsync<testcontainers::GenericImage>,
  Data<LemmyContext>,
  activitypub_federation::config::FederationConfig<LemmyContext>,
  String,
  Vec<EnvVarGuard>,  // NEW: outlives test-fn scope so guards drop on test exit, not boot_context return
)> {
  let mut guards: Vec<EnvVarGuard> = Vec::with_capacity(3);
  guards.push(EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1"));
  guards.push(EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX));
  let (container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  guards.push(EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url));
  // ... rest unchanged ...
  Ok((container, context, federation_config, db_url, guards))
}
```

All 10 callsites (e2e.rs:17577, 17632, 17672, 17716, 17757, 17821, 17865, 17891, 17924, 17982) change from `let (_container, context, _federation_context, db_url) = boot_context().await?;` to `let (_container, context, _federation_context, db_url, _env_guards) = boot_context().await?;`. T4 brief carries every verbatim `old_string`/`new_string` pair.

### 10.4 dq-lint-durations.sh shape (T1 target)

**Mirror:** structurally similar to `scripts/brehon/dq-schema-v3-migrate.sh` and `scripts/brehon/resolve-dq-canonical.sh`.

Skeleton (~80 LOC):

```bash
#!/usr/bin/env bash
# scripts/brehon/dq-lint-durations.sh — flag DQ entries where resolved_at < timestamp.
# Exits 0 if no negative-duration entries; non-zero with a list otherwise.
# Composite-id-aware (schema-v3): reports by `id` verbatim.
#
set -euo pipefail
DQ_PATH="${1:-.claude/decision-queue.json}"
[ -f "$DQ_PATH" ] || { echo "FATAL: not found: $DQ_PATH" >&2; exit 2; }

python3 -c "
import json, sys
from datetime import datetime
def parse(t):
    if not t: return None
    return datetime.fromisoformat(t.replace('Z', '+00:00'))
with open('$DQ_PATH') as f:
    dq = json.load(f)
bad = []
for arr in ('pending', 'resolved'):
    for e in dq.get(arr, []):
        ts = parse(e.get('timestamp'))
        ra = parse(e.get('resolved_at'))
        if ts and ra and ra < ts:
            delta = ts - ra
            bad.append((str(e.get('id')), e.get('timestamp'), e.get('resolved_at'), str(delta)))
for (eid, ts, ra, d) in bad:
    print(f'DQ-LINT FAIL: entry \"{eid}\" has resolved_at ({ra}) earlier than timestamp ({ts}) by {d}')
sys.exit(1 if bad else 0)
"
```

### 10.5 precheck.sh wiring (T1 target — new file)

**Mirror:** no canonical sibling on this lane.

T1 creates `scripts/brehon/precheck.sh` as a new repo-level gate. Shape:

```bash
#!/usr/bin/env bash
# scripts/brehon/precheck.sh — run before queueing Junior real-work tasks.
# Exits 0 if all gates pass; non-zero on failure.
#
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "[precheck] dq-lint-durations.sh ..."
"$SCRIPT_DIR/dq-lint-durations.sh"
echo "[precheck] OK"
```

### 10.6 deferral DQ shape (T2 target — `kind: "log"` entry)

**Mirror:** the v1-RT-r1 deferral DQ entries — `from: "planner"`, `kind: "log"`, `answered_by: "planner"`.

T2 dispatches `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json>` with a fragment of shape:

```json
{
  "from": "planner",
  "kind": "log",
  "timestamp": "<ISO8601 at T2 execution time>",
  "question": "C3 (Issue #158) emit_reputation_event helper extraction deferred per premature-DRY gate",
  "answer": "v1-quality-r2 brief WP-2 default: defer C3 unless planner identifies a 3rd consumer in near-term roadmap. As of 2026-05-28, federation_inbound lane status='done' in .claude/PRPs/v1-roadmap.json — no upcoming sub-phase introduces a 3rd reputation-event emitter. Therefore: 2 consumers as of plan author time; revisit when 3rd materialises. Per feedback_principles_not_rules.md.",
  "options": ["defer", "extract"],
  "context": "Two existing emitter sites: crates/api/api/src/governance/admin_emergency_remove.rs:448 (emit_reputation_event_local) and crates/api/api/src/governance/submit_jury_vote.rs:1096 (emit_reputation_event). Bodies are byte-identical. Extraction target: crates/api/api/src/governance/reputation_helpers.rs. Trigger condition for v1-quality-r3 follow-on: any sub-phase introduces a 3rd reputation-event emit path.",
  "answered_by": "planner",
  "resolved_at": "<same ISO as timestamp>"
}
```

Goes directly to `resolved[]` (kind: log self-resolves per `decision-queue.md` Recipe 2).

T2's brief also includes the GitHub issue comment text (NOT closing the issue; only commenting):

> v1-quality-r2 deferred C3 (this issue) per premature-DRY gate. 2 consumers as of 2026-05-28. DQ log entry: `<id>`. Revisit when a 3rd reputation-event emitter materialises. See `.claude/PRPs/plans/v1-quality-r2.plan.md` §10.6 for rationale.

(The GH issue comment is filed by the BM-session post-merge per §17 Completion checklist, not by T2's worker.)

### 10.7 Fixtures-module doc-comment header (T3 target)

The 14 modules to audit:
1. `mod governance_fixtures` @ e2e.rs:115
2. `mod admin_config_fixtures` @ e2e.rs:6100
3. `mod v1_jm_b_fixtures` @ e2e.rs:8535
4. `mod v1_jm_e_fixtures` @ e2e.rs:10454
5. `mod v1_sl_b_fixtures` @ e2e.rs:11658
6. `mod v1_sl_c_fixtures` @ e2e.rs:12689
7. `mod v1_sl_d_fixtures` @ e2e.rs:13567
8. `mod v1_sl_e_fixtures` @ e2e.rs:14426
9. `mod v1_federation_inbound_a_fixtures` @ e2e.rs:15393
10. `mod v1_ship_2_fixtures` @ e2e.rs:15456
11. `mod v1_federation_inbound_b_fixtures` @ e2e.rs:16186
12. `mod v1_federation_inbound_e_fixtures` @ e2e.rs:16518
13. `mod v1_ship_3_fixtures` @ e2e.rs:16699
14. `mod v1_rt_r3_fixtures` @ e2e.rs:17101 (already SAFETY-cited on `EnvVarGuard::set`; T3 brings header parity)

Suggested constraint-citation form (T3 brief decides exact text):

```rust
  //!
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`). Safety of those
  //! mutations is contingent on `--test-threads=1` (Cargo runner flag set
  //! in `crates/server/.cargo/config.toml` or invoked explicitly). Running
  //! these tests with concurrent threads will produce undefined behavior
  //! and is forbidden — see `EnvVarGuard::set` SAFETY block at e2e.rs:17187.
```

T3 brief pre-locates each module's current `mod <name>_fixtures { ` + existing `//! ` lines as `old_string`; the `new_string` adds the constraint-citation block. 14 anchors, mechanical sweep.

## 11. Files to change

Grouped by lane responsibility:

**Scripts (T1):**
- `scripts/brehon/dq-lint-durations.sh` — NEW shell script enforcing `resolved_at >= timestamp` invariant. ~80 LOC (T1).
- `scripts/brehon/precheck.sh` — NEW shell script wrapping pre-Junior-dispatch gates. ~25 LOC (T1).

**Decision queue (T1 + T2):**
- `.claude/decision-queue.json` — bulk sweep of 4 currently-back-dated entries (3999, 4007, 4016, 4035 per #157) — Option A floor unless git history unambiguously reveals the resolution moment (T1). Append C3 deferral entry via `dq-v3-append-fragment.sh` (T2).

**Tests (T3, T4, T5):**
- `crates/server/tests/e2e.rs` — 14 fixtures-module doc-comment header rewrites (T3) + `EnvVarGuard` hoist + `boot_context()` refactor + 10 callsite updates within `v1_rt_r3_fixtures` (T4) + 13 `LEMMY_DATABASE_URL` setter sites wrapped across 13 older fixtures modules (T5; site 14 handled by T4). Three distinct, non-overlapping edit regions; brief-time pre-located anchors for every Edit. NO new tests authored; all-refactor.

**Reports (T6):**
- `.claude/PRPs/reports/v1-quality-r2-retro.md` — NEW retro file per §13 Task 6.

**Crate manifests:** none modified. No new dependencies.

**Migrations:** none.

**Public API surface:** no public API changes (helper extraction is the only candidate and is DEFERRED per WP-2).

### Struct-field add: enumerate all callsites (mandatory)

Per `feedback_planner_enumerate_struct_callsites_for_addfield.md`: T4 changes the **return type** of `boot_context()` (struct-field analogue — tuple shape extended). Planner enumerated all 10 callsites at brief-author time:

```
crates/server/tests/e2e.rs:17577
crates/server/tests/e2e.rs:17632
crates/server/tests/e2e.rs:17672
crates/server/tests/e2e.rs:17716
crates/server/tests/e2e.rs:17757
crates/server/tests/e2e.rs:17821
crates/server/tests/e2e.rs:17865
crates/server/tests/e2e.rs:17891
crates/server/tests/e2e.rs:17924
crates/server/tests/e2e.rs:17982
```

All 10 are in the **same** module (`v1_rt_r3_fixtures`) and the **same** file (`e2e.rs`). T4's FILES YAML `modifies: [crates/server/tests/e2e.rs]` covers all callers.

## 12. NOT building in v1-quality-r2

- **C3 helper extraction (Issue #158).** Per WP-2 + the issue body's "wait for 3rd consumer" guidance. T2 files a `kind: "log"` deferral DQ on this phase; issue #158 stays OPEN with a deferral comment.
- **C1 attribute macro (WP-1 option b).** Planner default: doc-only sweep. Macro introduction requires `syn`/`quote`/`proc-macro2` deps; scope creep for no demonstrated recurrence risk.
- **C2 bulk sweep option (b) — re-author back-dated timestamps from git history.** Planner default: Option A floor. Option (b) only for entries where git history unambiguously identifies the resolving commit; T1 brief picks per-entry and surfaces ambiguous cases as a planner DQ at impl-time.
- **C4 follow-on env-var coverage (`LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` + `GOVERNANCE_LOG_SIGNING_KEY` retrofit).** ~28 additional `env::set_var` sites in `e2e.rs` set these two env vars without `EnvVarGuard`. Same defect class as #159+#160; deferred to **v1-quality-r3** to keep this phase under the e2e-edit complexity envelope. Tracked in §19; planner-DQ pre-seeded.
- **BREHON_DISABLE_*_JOB env mutation sites (e.g. lines 10845, 11026, 11074, 11224, 12799, 12897, 12930, 13088, 13118, 13196, 13214, 13329, 13373, 13559, 14578, 14588).** Already implement manual save-restore patterns. NOT strict env-leaks; deferred to v1-quality-r3.
- **Redaction-related fixtures or tests** — Lane A's territory (`phase-v1-redaction-r1`).
- **Touching `crates/api/api/src/governance/redaction.rs`** — Lane A's territory.
- **The wasmtime / extism dependency-tree CVE work** — deferred to `v1-deps-r3` per `v1-deps-r2-planning-1.md`.
- **The `webmention` crate inline replacement** — deferred to `v1-deps-r2`.

---

## 13. Step-by-step tasks

> Cohort dispatch: no `[P]` markers. T1 + T2 both mutate `.claude/decision-queue.json` → serial. T3/T4/T5 all modify `crates/server/tests/e2e.rs` → serial. Single-task cohorts throughout.
>
> Shape G is **SUSPENDED** through 2026-06-01 per `project_shape_g_suspended_2026_05_16.md`. Every cargo gate in this plan runs via **validate-pending-laptop** (`kind: "validate-pending-laptop"` for workspace check + clippy; `kind: "validate-pending-laptop-e2e"` for the e2e gate). impl-task subagents raise the laptop-DQ; the advisor laptop session executes the §15 commands.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment ready; confirm branch `phase-v1-quality-r2`; confirm RT-r3 + Quality-r1 deliverables intact on base; confirm clippy baseline clean.

**FILES:**

```yaml
creates: []
modifies: []
requires: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5):**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch confirmation
test "$(git branch --show-current)" = "phase-v1-quality-r2" || { echo "WRONG BRANCH"; exit 1; }

# Probe 2 — RT-r3 deliverables on base (EnvVarGuard struct present at e2e.rs:17179)
grep -n "^  struct EnvVarGuard {" crates/server/tests/e2e.rs > /dev/null || { echo "EnvVarGuard missing"; exit 1; }

# Probe 3 — wrapper sanity (cargo-check honors -p)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/PRPs/debug/v1-quality-r2-task0-check-p.log 2>&1"
echo "check-p exit: $?"

# Probe 4 — wrapper feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/PRPs/debug/v1-quality-r2-task0-check-features.log 2>&1"
echo "check-features exit: $?"

# Probe 5 — wrapper -p test target selection
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-quality-r2-task0-test-norun.log 2>&1"
echo "test-norun exit: $?"

# Probe 6 — negative test (exit-code propagation)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-quality-r2-task0-negative.log 2>&1"
echo "negative-feature exit: $?"
# EXPECT: non-zero (typically 101)

# Probe 7 — clippy baseline on trunk-ahead
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r2-task0-clippy.log 2>&1"
echo "clippy exit: $?"
# EXPECT: exit 0

# Probe 8 — re-enumerate brief §2 anchors against current trunk
echo "boot_context refs:"; rg -c "boot_context" crates/server/tests/e2e.rs
echo "LEMMY_DATABASE_URL setters:"; rg -c 'env::set_var\("LEMMY_DATABASE_URL"' crates/server/tests/e2e.rs
echo "*_fixtures modules:"; rg -c "^mod \w+_fixtures \{" crates/server/tests/e2e.rs
# EXPECT (planner's brief-author counts):
#   boot_context refs: 11   (1 declaration + 10 callsites)
#   LEMMY_DATABASE_URL setters: 14
#   *_fixtures modules: 14
```

**EXPECT block:**
- Probes 0..5 + 7 exit 0
- Probe 6 exits NON-ZERO
- Probe 8 counts match planner baseline (11/14/14)

**No commit at Task 0** — verification only.

---

### Task 1: C2 — DQ negative-duration lint script + precheck wiring + bulk sweep of back-dated entries

**Goal:** add a non-zero-exit lint that catches `resolved_at < timestamp` DQ entries; wire it as the first gate in a new `scripts/brehon/precheck.sh`; one-shot floor-sweep the 4 currently back-dated entries.

**FILES:**

```yaml
creates:
  - scripts/brehon/dq-lint-durations.sh
  - scripts/brehon/precheck.sh
modifies:
  - .claude/decision-queue.json
requires: []
```

**ACTION:** ship the lint script per §10.4 + the precheck wrapper per §10.5 + execute a one-shot floor sweep on `.claude/decision-queue.json` entries `3999`, `4007`, `4016`, `4035`. Verify post-sweep that the lint exits 0 on the live file; verify the lint exits non-zero on a synthetic test fixture with a back-dated `resolved_at`.

**IMPLEMENT (file 1 of 3):** in `scripts/brehon/dq-lint-durations.sh`, write the script per §10.4. Honour: `#!/usr/bin/env bash`, `set -euo pipefail`, accept positional `${1:-.claude/decision-queue.json}`, iterate `pending[]` + `resolved[]`, compare `resolved_at` vs `timestamp`, emit `DQ-LINT FAIL: entry "<id>" has resolved_at (<rfc3339>) earlier than timestamp (<rfc3339>) by <duration>` on stdout, exit non-zero on any finding, support both pre-v3 integer ids AND v3 composite `<session>-<seq>` ids verbatim.

**IMPLEMENT (file 2 of 3):** in `scripts/brehon/precheck.sh`, write the wrapper per §10.5. The script calls `dq-lint-durations.sh` (paths via `SCRIPT_DIR`) and surfaces non-zero exits.

**IMPLEMENT (file 3 of 3):** in `.claude/decision-queue.json`, edit 4 named entries' `resolved_at`:
- For entries 3999, 4007, 4016, 4035: if `resolved_at < timestamp`, set `resolved_at = timestamp` (Option A floor).
- If git log reveals a single unambiguous resolving commit whose `commit-date` is earlier than current `resolved_at` AND later than `timestamp`, prefer Option B: `resolved_at = <commit-date>`. Surface ambiguous cases as a `kind: "blocker"` DQ before applying.
- Use `python3 -c "import json; ..."` or `jq` for the mutation; commit + push the mutation atomically with the script additions.

**MIRROR:** `scripts/brehon/resolve-dq-canonical.sh` for the DQ-iteration pattern; `scripts/brehon/dq-schema-v3-migrate.sh` for the JSON mutation idiom.

**GOTCHA:** `.claude/decision-queue.json` is concurrently writeable in this lane worktree. Use the atomic read-mutate-commit-push protocol per `.claude/rules/multi-lane-worktree.md` Hard refusal #6. The sweep is a SINGLE commit, not 4 commits.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

```bash
bash scripts/brehon/dq-lint-durations.sh > .claude/PRPs/debug/v1-quality-r2-task1-postsweep.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r2-task1-postsweep.log
# EXPECT: exit 0; no DQ-LINT FAIL lines

# Synthetic-negative check
cp .claude/decision-queue.json .claude/PRPs/debug/dq-fixture.json
python3 -c "
import json
with open('.claude/PRPs/debug/dq-fixture.json') as f: dq = json.load(f)
if dq['resolved']:
    dq['resolved'][0]['resolved_at'] = '2020-01-01T00:00:00+00:00'
with open('.claude/PRPs/debug/dq-fixture.json', 'w') as f: json.dump(dq, f)
"
bash scripts/brehon/dq-lint-durations.sh .claude/PRPs/debug/dq-fixture.json > .claude/PRPs/debug/v1-quality-r2-task1-synthetic.log 2>&1
echo "exit: $?"
# EXPECT: exit non-zero; output contains DQ-LINT FAIL

bash scripts/brehon/precheck.sh > .claude/PRPs/debug/v1-quality-r2-task1-precheck.log 2>&1
echo "exit: $?"
# EXPECT: exit 0; "[precheck] OK"
```

**Commit subject:** `feat(scripts/brehon): add dq-lint-durations.sh + precheck.sh + sweep back-dated DQ entries (closes #157, task 1)`.

---

### Task 2: C3 — file `kind: "log"` deferral DQ for emit_reputation_event helper extraction (#158)

**Goal:** record the planner's WP-2 DEFER decision as an immutable DQ log entry; do NOT modify any `crates/**` files; issue #158 stays open with a deferral comment filed by the BM session post-merge.

**FILES:**

```yaml
creates: []
modifies:
  - .claude/decision-queue.json
requires:
  - task: 1
    reason: "Task 1 sweeps existing back-dated entries first; Task 2 appends a new entry to the cleaned-state DQ."
```

**ACTION:** generate a composite v3 DQ id, author the deferral fragment per §10.6, append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json>`. Single commit.

**IMPLEMENT (file 1 of 1):** in `.claude/decision-queue.json`, append exactly one entry to `resolved[]`:
1. Write fragment file at `.claude/PRPs/debug/v1-quality-r2-c3-defer.json` with the JSON shown in §10.6.
2. Run `bash scripts/brehon/dq-v3-append-fragment.sh .claude/PRPs/debug/v1-quality-r2-c3-defer.json` (NO `--pending` flag).
3. The helper computes id via `bash scripts/brehon/dq-v3-new-entry.sh`, injects it, appends to `resolved[]`, writes the file.
4. Verify the entry exists.

**MIRROR:** any `kind: "log"`, `from: "planner"`, `answered_by: "planner"` historical entry.

**GOTCHA:** Hard refusals #1, #8, #9 (`decision-queue.md`): never write `answered_by: "advisor"` or `approved_by: <anything>` from a Junior subagent; never use `max(all_ids)+1`; always go through `dq-v3-append-fragment.sh`.

**VALIDATE (story-checkpoint feeds §16a Story 2):**

```bash
python3 -c "
import json
d = json.load(open('.claude/decision-queue.json'))
hits = [e for e in d['resolved'] if e.get('answered_by') == 'planner' and 'C3' in e.get('question', '')]
assert len(hits) == 1, f'expected 1 C3 deferral entry, found {len(hits)}'
print('C3 deferral DQ entry id:', hits[0]['id'])
assert hits[0]['kind'] == 'log'
assert hits[0]['answered_by'] == 'planner'
assert hits[0].get('approved_by') is None
print('OK')
" > .claude/PRPs/debug/v1-quality-r2-task2-verify.log 2>&1
echo "exit: $?"

bash scripts/brehon/dq-lint-durations.sh > .claude/PRPs/debug/v1-quality-r2-task2-lint.log 2>&1
echo "exit: $?"
```

**Commit subject:** `chore(decision-queue): planner-defer C3 emit_reputation_event helper (Issue #158, task 2)`.

---

### Task 3: C1 — fixtures-module process-env safety doc-comment audit (14 modules)

**Goal:** every `mod *_fixtures` block in `crates/server/tests/e2e.rs` carries a doc-comment that cites the `--test-threads=1` constraint. Closes #156.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
requires: []
```

**ACTION:** 14 Edits in `crates/server/tests/e2e.rs`, each replacing a `mod *_fixtures {` block's existing opening `//! ` doc-comment lines with a new variant that adds the constraint-citation block from §10.7. Doc-only; no compile-relevant change.

**IMPLEMENT (file 1 of 1):** for each of the 14 modules enumerated in §10.7, paste verbatim `old_string` (current `//! ` lines, ~3-6 lines per module) and `new_string` (same content + constraint-citation block appended). T3 brief carries every pre-located anchor pair; 14 mechanical Edits in order.

**MIRROR:** `mod v1_rt_r3_fixtures` (e2e.rs:17101-17118) for the target shape — its `EnvVarGuard::set` SAFETY block at e2e.rs:17187 is the citation T3 propagates.

**GOTCHA:** older modules may lack any existing `//! ` block (e.g. `mod governance_fixtures { ` followed by a `use` line). T3 brief MUST handle the no-existing-doc case by anchoring on `{ ` + first `use` line and inserting the new `//! ` block between them. Pre-locate per-module.

**VALIDATE (story-checkpoint feeds §16a Story 3):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r2-task3-check.log 2>&1"
echo "check exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r2-task3-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features full > .claude/PRPs/debug/v1-quality-r2-task3-test-norun.log 2>&1"
echo "test-norun exit: $?"

# Brief-Scope output check — every fixtures module now contains "--test-threads=1"
python3 -c "
import re
src = open('crates/server/tests/e2e.rs').read()
mods = re.finditer(r'^mod (\w+_fixtures) \{', src, re.MULTILINE)
fails = []
for m in mods:
    name = m.group(1)
    region = src[m.end():m.end() + 3000]
    if '--test-threads=1' not in region:
        fails.append(name)
if fails:
    print('MISSING constraint citation in:', fails)
    raise SystemExit(1)
print('OK — all 14 fixtures modules cite --test-threads=1 in doc-comment')
" > .claude/PRPs/debug/v1-quality-r2-task3-audit.log 2>&1
echo "audit exit: $?"
```

**Commit subject:** `style(e2e): audit fixtures-module process-env safety doc-comments (closes #156, task 3)`.

---

### Task 4: C4-A — hoist EnvVarGuard + refactor boot_context to thread guards (v1_rt_r3_fixtures)

**Goal:** the `struct EnvVarGuard` is hoisted out of `mod v1_rt_r3_fixtures` to the test-crate root so sibling fixtures modules (T5) can `use super::EnvVarGuard;`. `boot_context()` is refactored to return a `Vec<EnvVarGuard>` as the fifth tuple element; all 3 internal `env::set_var` calls become `EnvVarGuard::set`; all 10 same-module callsites update the tuple destructure. Closes #159.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
requires:
  - task: 3
    reason: "T3 doc audit on the same file MUST land before T4's structural edit so the doc-comment lines T3 inserted are not invalidated by T4's anchor mismatches."
```

**ACTION:** 4 distinct edit clusters in `crates/server/tests/e2e.rs`:

1. **Hoist:** move `struct EnvVarGuard`, `impl EnvVarGuard`, `impl Drop for EnvVarGuard` (~32 lines at e2e.rs:17174-17205) from inside `mod v1_rt_r3_fixtures` to the test-crate root, BEFORE the first `mod *_fixtures` block (above e2e.rs:115 — between any existing top-level `use` block and the first `mod governance_fixtures {`).

2. **boot_context refactor:** change return type to add `Vec<EnvVarGuard>` as the fifth tuple element; rewrite 3 internal `unsafe { std::env::set_var(...) }` blocks (e2e.rs:17422-17426 + e2e.rs:17429-17432) as `guards.push(EnvVarGuard::set(...))` per §10.3.

3. **Callsite updates (10 sites):** each `let (_container, context, _federation_context, db_url) = boot_context().await?;` (or `federation_config` variant) becomes `let (_container, context, _federation_context, db_url, _env_guards) = boot_context().await?;`. Pre-located anchors per §11 enumeration: e2e.rs:17577, 17632, 17672, 17716, 17757, 17821, 17865, 17891, 17924, 17982.

4. **Module-internal `use super::EnvVarGuard;` cleanup:** after hoist, `mod v1_rt_r3_fixtures` no longer defines `EnvVarGuard`; insert `use super::EnvVarGuard;` near the top of the module's `use` block so the existing `EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1")` usages at e2e.rs:17576, 17631, 17671, 17715 still compile.

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`. T4 brief carries every verbatim anchor pair across the 4 edit clusters. Total Edit count for T4: ~16 (1 hoist-delete + 1 hoist-insert + 1 module use-line add + 3 boot_context internal mutations + 1 boot_context return-type signature + 10 callsites).

**MIRROR:** §10.3 (boot_context return-type extension); §10.1 (EnvVarGuard struct verbatim).

**GOTCHA:** `Vec<EnvVarGuard>` must outlive the test fn scope; that's why guards return to the caller. The pattern is **return-by-value into the caller's scope** — the binding `_env_guards` at the callsite is the load-bearing lifetime extension.

**GOTCHA #2:** `_env_guards` (with leading underscore) silences the unused-binding warning but does NOT cause early drop. Use `_env_guards`, not `_`.

**GOTCHA #3:** signature change → R7 applies. After T4's commit, run `cargo test --no-run -p lemmy_server --test e2e --features full` (per §15.3) before pushing.

**VALIDATE (story-checkpoint feeds §16a Story 4):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r2-task4-check.log 2>&1"
echo "check exit: $?"

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r2-task4-clippy.log 2>&1"
echo "clippy exit: $?"

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features full > .claude/PRPs/debug/v1-quality-r2-task4-test-norun.log 2>&1"
echo "test-norun exit: $?"

# Brief-Scope output check
python3 -c "
import re
src = open('crates/server/tests/e2e.rs').read()
struct_pos = src.find('struct EnvVarGuard {')
first_mod = re.search(r'^mod \w+_fixtures \{', src, re.MULTILINE).start()
assert struct_pos < first_mod, 'EnvVarGuard struct not hoisted'
rt_r3_start = src.find('mod v1_rt_r3_fixtures {')
rt_r3_region = src[rt_r3_start:rt_r3_start + 100000]
assert 'use super::EnvVarGuard' in rt_r3_region
bc = re.search(r'async fn boot_context\(\)[^{]+\{', rt_r3_region, re.DOTALL)
assert bc is not None and 'EnvVarGuard' in bc.group(0)
print('OK')
" > .claude/PRPs/debug/v1-quality-r2-task4-audit.log 2>&1
echo "audit exit: $?"
```

**Commit subject:** `refactor(e2e): hoist EnvVarGuard + thread guards through boot_context (closes #159, task 4)`.

---

### Task 5: C4-B — wrap 13 LEMMY_DATABASE_URL setter sites with EnvVarGuard (13 fixtures modules)

**Goal:** every `std::env::set_var("LEMMY_DATABASE_URL", ...)` call in `e2e.rs` outside `boot_context()` is wrapped with `EnvVarGuard::set("LEMMY_DATABASE_URL", ...)` and bound to a `_g_db_url` local. Closes #160.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
requires:
  - task: 4
    reason: "T4 hoists EnvVarGuard to test-crate root; T5 requires EnvVarGuard to be visible to all sibling fixtures modules. Without T4's hoist, T5 cannot reach EnvVarGuard except by duplication."
```

**ACTION:** 13 Edits in `crates/server/tests/e2e.rs`, each replacing a `unsafe { std::env::set_var("LEMMY_DATABASE_URL", &<var>); }` block (plus its `// SAFETY:` doc lines) with a single `let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &<var>);` line. For each affected fixtures module, ALSO add `use super::EnvVarGuard;` if not already present from `use super::*;`.

**IMPLEMENT (file 1 of 1):** T5 brief pre-locates all 13 verbatim anchors. Sites (planner-enumerated; site 14 at line 17431 is inside `boot_context()` and handled by T4, NOT T5):

| # | Line | Fixtures-module region | Variable |
|---|---|---|---|
| 1 | 806 | governance_fixtures | `db_url` |
| 2 | 2544 | (test scope) | `db_url` |
| 3 | 3324 | admin_config_fixtures region | `db_url` |
| 4 | 4096 | (test scope) | `db_url` |
| 5 | 4443 | (test scope) | `db_url` |
| 6 | 4774 | (test scope) | `db_url` |
| 7 | 4907 | (test scope) | `db_url` |
| 8 | 5044 | (test scope, two-DB) | `url_a` |
| 9 | 5093 | (test scope, two-DB) | `url_b` |
| 10 | 5671 | (test scope) | `db_url` |
| 11 | 5874 | (test scope) | `db_url` |
| 12 | 6137 | admin_config_fixtures region | `db_url` |
| 13 | 16752 | v1_ship_3_fixtures | `db_url` |

(Site 14 at e2e.rs:17431 is inside `boot_context()` and handled by T4. T5 brief MUST NOT touch line 17431.)

For each of the 13 in-scope sites, brief carries verbatim `old_string` (~3-5 surrounding lines for uniqueness) replaced by a single `let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &<var>);` line per §10.2. Per-module `use super::EnvVarGuard;` insert is conditional (skip if `use super::*;` already present).

**MIRROR:** §10.2 (EnvVarGuard usage shape); T4's commit on phase tip for hoisted `EnvVarGuard` location.

**GOTCHA:** the SAFETY comment is REMOVED. The justification now lives at e2e.rs:17187. Drop the comment when the `unsafe` block goes.

**GOTCHA #2:** if a fixtures module already has `use super::*;` AND T4 hoisted `EnvVarGuard` to test-crate root, no per-module `use super::EnvVarGuard;` is needed. Conditional per module.

**VALIDATE (story-checkpoint feeds §16a Story 5):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r2-task5-check.log 2>&1"
echo "check exit: $?"

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r2-task5-clippy.log 2>&1"
echo "clippy exit: $?"

# Brief-Scope output check — zero unguarded LEMMY_DATABASE_URL setter sites remain (outside EnvVarGuard internals)
python3 -c "
import re
src = open('crates/server/tests/e2e.rs').read()
unguarded = []
for m in re.finditer(r'env::set_var\(\s*\"LEMMY_DATABASE_URL\"', src):
    pos = m.start()
    line_num = src[:pos].count('\n') + 1
    line_start = src.rfind('\n', 0, pos) + 1
    line_end = src.find('\n', pos)
    line = src[line_start:line_end]
    context_above = src[max(0, pos - 500):pos]
    if 'impl EnvVarGuard' in context_above[-500:] or 'impl Drop for EnvVarGuard' in context_above[-500:]:
        continue
    unguarded.append((line_num, line.strip()))
if unguarded:
    print('UNGUARDED LEMMY_DATABASE_URL setter sites remain:', unguarded)
    raise SystemExit(1)
total = len([m for m in re.finditer(r'\"LEMMY_DATABASE_URL\"', src)])
print(f'OK — zero unguarded LEMMY_DATABASE_URL setter sites (validated {total} total mentions)')
" > .claude/PRPs/debug/v1-quality-r2-task5-audit.log 2>&1
echo "audit exit: $?"
```

**Commit subject:** `refactor(e2e): wrap LEMMY_DATABASE_URL setters with EnvVarGuard across 13 fixtures modules (closes #160, task 5)`.

---

### Task 6: Retro

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-quality-r2-retro.md
modifies: []
requires:
  - task: 5
    reason: "Retro authored after all impl tasks land; signals are collected from the live phase branch + PR feedback."
```

**Goal:** author retro per `feedback_retro_not_report.md` and `feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with signals + lessons. Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit.

**ACTION:** write `.claude/PRPs/reports/v1-quality-r2-retro.md` with the canonical four-role structure. Sections:

- Header: phase, PR #, merge SHA, dates
- §1 What surprised us (per-role)
- §2 What to change (per-role; high-confidence proposals only)
- §3 What to carry forward (per-role)
- §4 Per-task complexity score (`<files>/<commits>/<runtime-min>/<max-log-silence-min>` per task) per `feedback_retro_task_complexity_score.md`
- §5 Lessons promoted this phase
- §6 Phase-level four-role retro signals table

**MIRROR:** `.claude/PRPs/reports/v1-RT-r3-retro.md` (committed at `1edb8b94c`).

**GOTCHA:** the retro is NOT a status report — focus on signals worth carrying forward.

**VALIDATE:**

```bash
test -f .claude/PRPs/reports/v1-quality-r2-retro.md && wc -l .claude/PRPs/reports/v1-quality-r2-retro.md
grep -cE "^## (Advisor|Planning|Impl|BM)" .claude/PRPs/reports/v1-quality-r2-retro.md
# EXPECT: file exists, ≥ 50 lines; 4 role headers (relaxed if H3 nesting used)
```

**Commit subject:** `docs(retro): v1-quality-r2 phase retro - PR #<N> merged <sha>`.

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full"` — after T3, T4, T5.
- **Lint:** `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"` — after T3, T4, T5.
- **Test target compile (R7):** `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features full"` — after T4 (signature change).
- **e2e execution (phase-tip gate):** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` — ONCE post-T5. ~26 min on laptop. Raised as `kind: "validate-pending-laptop-e2e"`.
- **DQ lint:** `bash scripts/brehon/dq-lint-durations.sh` — after T1, T2.
- **Migration round-trip:** N/A.

## 15. Validation commands (DoD)

> Planner-side discipline (per `feedback_plan_dod_dry_run_at_write.md`): every command dry-runned by the advisor against `phase-v1-quality-r2` tip before plan approval. Wrappers on Linux at `scripts/brehon/cargo-*.sh`; on the laptop the `.bat` siblings are canonical.

### 15.1 Static analysis (per task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r2-<task>-check.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

Per task in `{task3, task4, task5}`.

### 15.2 Lint (per task touching Rust)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r2-<task>-clippy.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

Per task in `{task3, task4, task5}`.

### 15.3 Test target compile (R7)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features full > .claude/PRPs/debug/v1-quality-r2-<task>-test-norun.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

Per task in `{task4}`. T3/T5 run R7 as cheap insurance too.

### 15.4 e2e execution (phase-tip)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-quality-r2-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-quality-r2-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-quality-r2-e2e.log"
```

Raised as `kind: "validate-pending-laptop-e2e"` from T5's worker; advisor runs on laptop. Required pass before PR open.

### 15.5 Cross-cutting verification

- [ ] R1: no new `i32 ↔ i64` comparisons.
- [ ] R5: Task 0 enumerates all probes (Probes 0–8).
- [ ] R6: all clippy invocations use `--no-deps`.
- [ ] R7: cargo test --no-run runs after T4.
- [ ] R8: never `-p <crate>` with `--features full`.
- [ ] R9: every cargo gate invokes the wrapper.
- [ ] R10: every cargo invocation redirects to a file.
- [ ] R11: every e2e.rs Edit in T3/T4/T5 has pre-located verbatim anchors.
- [ ] `dq-lint-durations.sh` exits 0 on `.claude/decision-queue.json` post-T1+T2.
- [ ] No new `unsafe { std::env::set_var(...) }` blocks outside `EnvVarGuard::set`/`EnvVarGuard::drop` in `e2e.rs`.
- [ ] No edits to files outside §11 list.
- [ ] Issues #156/#157/#159/#160 have closing PR references at merge time; #158 has a deferral comment + remains open.

### 15.6 DoD per workflow (Shape G)

NOT APPLICABLE. Shape G SUSPENDED through 2026-06-01. All cargo gates run via validate-pending-laptop on the advisor session.

---

## 16. Acceptance criteria

- [ ] All 5 impl tasks (T1, T2, T3, T4, T5) completed in dependency order
- [ ] §15.1 exit 0 after T3, T4, T5
- [ ] §15.2 exit 0 after T3, T4, T5
- [ ] §15.3 exit 0 after T4
- [ ] §15.4 exit 0 phase-tip — pre-RT-r3 pass count (103+) or higher, 0 fail
- [ ] §15.5 cross-cutting verification — all checkboxes ticked
- [ ] §16a stories — all 5 stories `[done]`
- [ ] No edits to files outside §11 list
- [ ] Retro committed per §13 Task 6
- [ ] PR opens against `governance-v0` with `--repo barrie-cork/lemmy`
- [ ] Issues #156, #157, #159, #160 closed with merge-commit + PR references
- [ ] Issue #158 carries a deferral comment cross-referencing the C3 deferral DQ id; remains OPEN

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: DQ negative-duration entries are blocked at gate time

- **Composing tasks:** Task 1
- **Checkpoint command (positive):** `bash scripts/brehon/dq-lint-durations.sh; echo "exit: $?"`
- **Expected output:** `exit: 0`.
- **Checkpoint command (negative):** synthetic-fixture invocation per Task 1 §VALIDATE block.
- **Expected output:** non-zero exit, `DQ-LINT FAIL: entry "<id>" has resolved_at ... earlier than timestamp ...`.
- **Brief-Scope outputs to verify:**
  - `scripts/brehon/dq-lint-durations.sh` exists, has `#!/usr/bin/env bash`, `set -euo pipefail`.
  - `scripts/brehon/precheck.sh` exists, sources the lint.
  - `.claude/decision-queue.json` entries 3999, 4007, 4016, 4035 have `resolved_at >= timestamp`.

### Story 2: C3 helper extraction is deferred with audit trail

- **Composing tasks:** Task 2
- **Checkpoint command:** Task 2 §VALIDATE Python block.
- **Expected output:** `OK`.
- **Brief-Scope outputs to verify:**
  - `.claude/decision-queue.json` `resolved[]` contains exactly one entry whose `question` contains `"C3 (Issue #158)"`, `kind == "log"`, `answered_by == "planner"`, `approved_by is None`.
  - The entry's `answer` cites the 2-consumer enumeration and names `feedback_principles_not_rules.md`.

### Story 3: Every fixtures module's doc-comment cites --test-threads=1

- **Composing tasks:** Task 3
- **Checkpoint command:** Task 3 §VALIDATE Python audit.
- **Expected output:** `OK — all 14 fixtures modules cite --test-threads=1 in doc-comment`.
- **Brief-Scope outputs to verify:**
  - 14 `mod *_fixtures` blocks in `e2e.rs` each contain `--test-threads=1` in the doc-comment within ~30 lines of the module-opening `{`.

### Story 4: boot_context threads EnvVarGuards to test fns (Issue #159 fix verified)

- **Composing tasks:** Task 4
- **Checkpoint command:** Task 4 §VALIDATE Python audit + e2e exit 0 from §15.4.
- **Expected output:** `OK` + `E2E_EXIT_0`.
- **Brief-Scope outputs to verify:**
  - `struct EnvVarGuard { ... }` exists BEFORE the first `mod *_fixtures` block.
  - `mod v1_rt_r3_fixtures` contains `use super::EnvVarGuard;`.
  - `async fn boot_context()` return type is a 5-tuple including `Vec<EnvVarGuard>` (or `(EnvVarGuard, EnvVarGuard, EnvVarGuard)` tuple variant).
  - All 10 boot_context callsites destructure 5 elements.

### Story 5: Every LEMMY_DATABASE_URL setter is EnvVarGuard-wrapped (Issue #160 fix verified)

- **Composing tasks:** Task 5
- **Checkpoint command:** Task 5 §VALIDATE Python audit + e2e exit 0 from §15.4.
- **Expected output:** `OK — zero unguarded LEMMY_DATABASE_URL setter sites` + `E2E_EXIT_0`.
- **Brief-Scope outputs to verify:**
  - Zero `unsafe { std::env::set_var("LEMMY_DATABASE_URL", ...) }` blocks outside `EnvVarGuard::set`/`EnvVarGuard::drop` themselves.
  - 13 sites converted; site 14 covered by T4.
  - Every modified fixtures module either has `use super::EnvVarGuard;` or `use super::*;`.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0–8 confirmed; counts match planner baseline 11/14/14)
- [ ] Task 1..5 committed in dependency order on `phase-v1-quality-r2`
- [ ] §15 validation green at every gate
- [ ] §16a stories all `[done]`
- [ ] Retro committed (Task 6)
- [ ] PR opened by BM session against `governance-v0` with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-quality-r2-verify.md` shows all 5 stories ✓
- [ ] Issues #156/#157/#159/#160 closed with merge-commit references by BM
- [ ] Issue #158 carries the deferral comment with C3 DQ id; remains OPEN
- [ ] Roadmap updated: `lanes.quality.sub_phases.v1-quality-r2` flipped `done`
- [ ] Post-merge phase branch retained for retro reads

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| T4 boot_context return-type change breaks callsite tuple destructure the planner missed | MED | HIGH (compile-fail blocks T5) | Pre-located anchors for all 10 callsites in T4 brief; R7 cargo-test --no-run after T4; the audit Python block verifies the 5-tuple shape. |
| T5 site count drift between brief author time and dispatch (Lane A adds a new LEMMY_DATABASE_URL site) | LOW | MED | T5 brief carries 13 verbatim anchors; Task 0 Probe 8 re-enumeration catches new sites. Mitigation: between T2 and T3, advisor re-runs Probe 8 enumeration on live phase tip + governance-v0 tip; non-zero delta surfaces as a `kind: "blocker"` planner-DQ. |
| Hoist of EnvVarGuard breaks an existing import or use scope | LOW | MED | EnvVarGuard hoist is purely upward (test-crate root). Sibling modules currently can't see it; after hoist they can via `use super::EnvVarGuard;`. No callsite outside v1_rt_r3_fixtures references EnvVarGuard today. |
| §5 split-DQ rejected by advisor (advisor decides split, not proceed) | LOW | LOW | Plan re-authors into `v1-quality-r2a` + `v1-quality-r2b`; wall-clock cost ~2-3 hours + 2x bm-cut. Acceptable. |
| §15.4 e2e fails on pre-existing flake | MED | LOW | Per `feedback_phase_2_e2e_gate_enforcement.md`: re-run once; if still failing, surface to user as a `kind: "blocker"` DQ before opening the PR. |
| C2 bulk sweep loses data by floor-collapsing a legitimate later resolved_at | LOW | LOW | The 4 named entries are confirmed back-dated per #157; the Option A floor only EQUALS timestamp (not erases). Option B is the escape hatch when the resolving commit is identifiable. |
| Lane A merges to `governance-v0` between bm-cut and T3 dispatch adding new `mod *_fixtures` blocks | LOW | LOW | T3 mechanical audit catches missing citation. If Lane A merges to `governance-v0`, advisor MUST sync `governance-v0` into `phase-v1-quality-r2` (per brief WP-6 hard rule). |
| §3.9.1 conformance-audit flags a Tier-1 finding | LOW | HIGH (catch-fire) | C3 is DEFERRED — no new file in `crates/api/api/src/governance/**.rs`. Audit at `/brehon-verify` before bm-merge; surface unexpected findings to user. |

---

## 19. Notes

**Planner pre-seeded DQ entries** (per `.claude/agents/planning.md`). The planning Junior writes these AFTER the plan file commit lands on the worker branch:

1. **Split-DQ for §5 complexity score 10 > 8.** Filed as `pending`, `from: "planner"`, `kind: "blocker"`, `answered_by: null`. Context cites v1-RT-r3 precedent for the "proceed" recommendation. Advisor decides at user-gate-1.

2. **C4 follow-on enumeration** (per §12 NOT-building list). Filed as `pending`, `from: "planner"`, `kind: "log"`, `answered_by: "planner"`. Names the ~28 additional `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` + `GOVERNANCE_LOG_SIGNING_KEY` setter sites for a potential `v1-quality-r3`. Defers the decision to the next-phase advisor.

3. **(Conditional) C2 Option B per-entry sweep ambiguity** — IF, during T1, any of the 4 named entries cannot be unambiguously resolved by Option A floor or Option B re-author, T1's worker raises this as `pending`, `from: "impl"`, `kind: "blocker"`. Not pre-seeded; raised on demand.

**Pre-flight precondition for advisor at user-gate-1:**
- Confirm Lane A (`phase-v1-redaction-r1`) has not merged in a way that conflicts with §11.
- Confirm trunk has not advanced past `1edb8b94c` to a SHA that introduces new `mod *_fixtures` blocks (would push T3 count from 14 to 15+).
- Re-run Probe 8 enumeration against current `phase-v1-quality-r2` tip.

**Choice of Option A WP-Cluster despite §5 score**: the e2e edit factor (+9) is the dominant contributor; splitting into 2a+2b doesn't reduce per-task e2e count; it only doubles CR/e2e gate cost. Worker-hang risk is mitigated by pre-located anchors. Hence "proceed" recommendation is defensible.

**Why no `[P]` markers in §13:** every adjacent task pair shares at least one file in YAML (`.claude/decision-queue.json` for T1+T2; `crates/server/tests/e2e.rs` for T3+T4+T5). Serial fallback is correct.

**Lesson promotion candidates** (for retro §5):
- `feedback_envvarguard_hoist_for_fixtures_module_sharing.md` — when an RAII guard struct is defined inside one fixtures module but needed by sibling modules, hoist to test-crate root with `use super::` per module. Single recurrence so far. Promote at retro if it surfaces a 2nd time.
- `feedback_plan_complexity_e2e_edit_factor_overweights_doc_only_tasks.md` — the +3 e2e factor doesn't distinguish doc-only edits from structural edits. May warrant a sub-weight. Defer to retro signal.

---

## 20. Confidence score

- **Plan correctness:** 8/10 — the §5 split-DQ is the load-bearing uncertainty; advisor decision drives the next path. All other watchpoints have planner-default decisions with cited rationale.
- **Cargo budget:** 9/10 — validate-pending-laptop, ~6 GB peak, no off-box concerns.
- **Test coverage:** 8/10 — no new tests added; coverage by-construction (refactor-only). E2E phase-tip gate confirms no behavior regression.
- **Anchor pre-location completeness:** 7/10 — planner enumerated counts and line numbers in §11; impl-task briefs must paste verbatim `old_string`/`new_string` chunks (planner provides enumeration; brief author derives literal anchor bytes from current phase tip). Advisor's bm-cut → brief author pass closes this gap before T3/T4/T5 dispatch.
