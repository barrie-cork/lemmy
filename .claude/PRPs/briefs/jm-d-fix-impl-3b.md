---
role: impl-task
plan_task: 3
phase: v1-JM-d
created: 2026-04-28
status: ready
related_dq: 80
model_override: opus[1m]
---

# Brief — v1-JM-d Fix 3b — 7 independent e2e failures (Phase 2 run 25056300174)

## 1. Role + dispatch line

`[role:planning] v1-JM-d fix-3b — see .claude/PRPs/briefs/jm-d-fix-impl-3b.md`

> **Note:** The `[role:planning]` prefix is intentional — this brief requests Opus 4.7 (1M)
> for the impl work due to the complexity of the 3-cluster diagnosis below. You are acting
> as **impl-task** (code author), not as a planner. Produce one commit, push your branch,
> raise a `validate-pending` DQ entry. Do not author a plan document.

## 2. Scope

**Context:** fix-3a (`limit(4→6)` + 2×`closed_at→appeal_window_expires_at`) shipped and
passed Phase 1 workspace check (DQ #79). Phase 2 e2e run 25056300174 shows **7 failures,
55 passed** (up from 54 before fix-3a; `v1_jm_a_backfill_populates_v0_snapshot` now passes).
The 7 remaining failures are independent Task 3 code regressions, not cascade. Fix all 7 in
one commit to `crates/server/tests/e2e.rs` only.

**Do NOT edit** any file outside `crates/server/tests/e2e.rs`. The production code is correct;
all 7 failures are test-setup deficiencies that did not anticipate Task 3's new invariants.

---

### Cluster A — `panel_size_snapshot` NULL on test-helper-created Decided cases (3 tests)

**Root cause:** Task 3's `request_appeal` now calls `select_appeal_panel()` (via the
`appeal.auto_select_on_appeal_acceptance = true` default) which reads
`case.panel_size_snapshot` from `moderation_case`. This column is populated only when
`admin_assign_jury` runs (writes `moderation_case.panel_size_snapshot` at
`admin_assign_jury.rs:210`). E2e tests that create a Decided case via a shortcut helper
that sets `status = Decided` directly (bypassing `admin_assign_jury`) leave
`panel_size_snapshot = NULL`, causing the defensive guard at `admin_assign_jury.rs:1097` to
return `LemmyErrorType::Unknown("appeal panel: case.panel_size_snapshot is null on a Decided
case")`.

**Failing tests:**
- `appeal_inside_window_succeeds_expired_rejects` — error: `Unknown: admin_assign_jury.rs:1097`
- `all_mvp_endpoints_return_non_404` — appeal returns 400 (upstream error from same cause)
- `submit_jury_vote_writes_appeal_window_live_config` — 7d gap instead of expected 30d

Wait — `submit_jury_vote_writes_appeal_window_live_config` is in Cluster B below. Only the
first two are Cluster A. Read §Cluster B carefully.

**Fix required:** In each failing test, find the helper call that creates a Decided
moderation case without going through `admin_assign_jury`. Replace it (or augment it) so
`moderation_case.panel_size_snapshot` is populated before `request_appeal` is called.
The simplest fix: after the direct-status-set, run a raw SQL UPDATE:

```sql
UPDATE moderation_case SET panel_size_snapshot = 5 WHERE id = <case_id>
```

Use diesel syntax consistent with the surrounding test code. Value `5` is the seeded default
for `jury.panel_size.regular.minor`. If the test uses a different severity tier, match the
seeded value from migration `2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql`.

**Affected e2e.rs locations:**
- Search for the test function `appeal_inside_window_succeeds_expired_rejects` and find
  where the Decided case is set up.
- Search for `all_mvp_endpoints_return_non_404` and find the `Decided` case setup. This
  test's panic is at `e2e.rs:2933`; read ±50 lines above to find the case setup.

---

### Cluster B — `appeal_window_expires_at` uses seeded default (7d) not LIVE config (3 tests)

**Root cause:** Tests `submit_jury_vote_writes_appeal_window_live_config` and
`v0_case_completes_under_v0_rules_after_v1_config_flip` explicitly bump
`appeal.window_days` via `admin_set_config` and then assert that a *newly decided* case
has `appeal_window_expires_at` reflecting the bumped value (30d and 60d respectively).
The appeal window is written by `submit_jury_vote.rs:607–618` — a LIVE config read at
decision moment. The tests' gap measurement shows `TimeDelta { secs: 604800 }` = 7 days
(the seeded default). This means `submit_jury_vote` is reading the config correctly but
the **test's `admin_set_config` call is not persisting** the bumped value before the
jury vote fires, or the Decided case in the test is pre-existing (created before the
config bump).

**Panic locations:**
- `submit_jury_vote_writes_appeal_window_live_config` → `e2e.rs:8592`
- `v0_case_completes_under_v0_rules_after_v1_config_flip` → `e2e.rs:8763`

**Fix required:** Read the test bodies (lines 8550–8600 and 8720–8780 respectively).
Determine the ordering: does the config bump happen BEFORE or AFTER `submit_jury_vote`
is called? If the config is bumped after the case is decided, the test must be restructured
so the bump happens before the final jury vote. If the case was decided by a test helper
shortcut (not `submit_jury_vote`), the test must either:
  (a) write `appeal_window_expires_at` explicitly to the expected value after deciding, or
  (b) drive the decision through `submit_jury_vote` after bumping the config.

Option (a) is simpler if the test only needs to verify the downstream appeal-window check.
Option (b) is more correct if the test is specifically validating that `submit_jury_vote`
reads LIVE config.

The third test in this cluster is `appeal_inside_window_succeeds_expired_rejects` (also
Cluster A). It may fail both because of the `panel_size_snapshot` NULL AND because
`appeal_window_expires_at` is NULL (both missing from a helper-created Decided case).
Fix both deficiencies for that test.

---

### Cluster C — admin config tests read back wrong values (3 tests)

**Root cause:** These 3 tests were likely passing before because the backfill panic caused
them to be skipped or their failures were lumped as cascade. They are pre-existing
test-isolation issues now exposed independently.

**Failing tests and panic locations:**
- `admin_get_config_single_key_with_provenance` → `e2e.rs:5295`: writes `jury.panel_size = 11`, reads back `5`
- `admin_list_rule_sets_returns_versions_with_active_version_id` → `e2e.rs:5911`: expects `active_version_id = Some(ids[2])`, got `Some(1)`
- `admin_set_config_persists_previous_value_and_from` → `e2e.rs:6024`: expects `previous_value = Some(7)`, got `Some(5)`

**Pattern:** Config writes are not being read back correctly. The seeded migration value for
`jury.panel_size.regular.minor` is `5` — this is what's being returned. The test writes to
key `jury.panel_size` (without the tier suffix) but the config system may be reading the
nearest matching seeded row. OR the test's `admin_set_config` call is failing silently and
the read returns the seeded default.

**Fix required:** Read the test bodies (lines 5260–5310, 5870–5920, 5990–6030 respectively).
Determine:
1. Is the write actually using the correct key string? (`jury.panel_size` vs a tier-qualified
   key like `jury.panel_size.regular.minor`?)
2. Is there a missing `await` or error propagation that causes the write to silently fail?
3. Is the `active_version_id` test using the correct ID — does it rely on auto-increment IDs
   that shift because Task 3's migration seeded additional `governance_config` rows (shifting
   the DB sequence)?

If IDs shifted: replace hardcoded ID lookups with queries that fetch by version number or
creation order rather than relying on auto-increment values.

If wrong key string: update the test to use the correct fully-qualified key.

---

## 3. Required reading

- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **CRITICAL**: do NOT use the
  Edit tool on the full 8945-line e2e.rs. Use targeted line-range reads + surgical edits.
- `crates/server/tests/e2e.rs` — read only the specific line ranges called out per cluster:
  - Cluster A: search `appeal_inside_window_succeeds_expired_rejects` + lines ~2880–2940
  - Cluster B: lines 8550–8600 + lines 8720–8780
  - Cluster C: lines 5260–5310 + lines 5870–5920 + lines 5990–6030
- `crates/api/api/src/governance/admin_assign_jury.rs:1083–1102` — `select_appeal_panel`
  signature and `panel_size_snapshot` guard
- `migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql` — seeded config
  values (panel sizes, `appeal.window_days = 7`)
- `crates/api/api/src/governance/submit_jury_vote.rs:607–618` — the LIVE config read for
  `appeal.window_days` that writes `appeal_window_expires_at`
- `crates/api/api_crud/src/governance/request_appeal.rs:115–145` — the
  `appeal_window_expires_at` check + `winning_decision` eligibility check Task 3 added

## 4. Constraints

- **One commit.** Subject: `fix(test): e2e fixture gaps for Task 3 invariants (fix-3b)`
- **Only edit `crates/server/tests/e2e.rs`.** No production code, no migration edits.
- **Do NOT touch `PHASE_1_MIGRATION_COUNT`** at line 346 — `#[ignore]`'d, separate concern.
- Push your branch to origin. The branch name is set by the Junior daemon from your task title.
- **Post-commit:** raise a `kind: "validate-pending"` DQ entry. Use `gh run list` to capture
  the workspace-check workflow_run_id from your push. JSON skeleton:

```json
{
  "id": 81,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<ISO-8601 now>",
  "workflow_run_id": <id from gh run list>,
  "branch": "<your junior/* branch>",
  "phase_task": 3,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

  Append to `pending[]` in `.claude/decision-queue.json` on the `governance-v0` branch.
  Commit: `chore(decision-queue): impl raised DQ #81 — validate-pending fix-3b workspace run <id>`
  Push to `governance-v0`.

- If you cannot determine the root cause for any of the 3 clusters without running cargo
  locally, file a DQ `kind: "blocker"` entry (id: 81) with your analysis (test name,
  suspected root cause, files involved) and exit cleanly. Do not guess.
