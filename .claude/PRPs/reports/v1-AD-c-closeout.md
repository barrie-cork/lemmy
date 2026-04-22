# v1-AD-c closeout — 2026-04-22

**Final status:** MERGED. PR #81 → `governance-v0` at `cf89890f3` (merged 2026-04-22T16:19:24Z).
**Next sub-phase:** v1-AD-d.
**Carry-forward issues filed:** #82–#85 (see memory `project_v1_AD_c_closed.md`).

## CR round 3 summary (this session)

**Commit shipped:** `6baabfd7a` — `fix(v1-AD-c): address 3 CR findings on PR #81 (round 3)`

| CR# | Action | Location | Outcome |
|---|---|---|---|
| A | Rebuttal | `api_common/governance.rs:585` | Posted inline [#r3125313924](https://github.com/barrie-cork/lemmy/pull/81#discussion_r3125313924) — v0-scope guideline stale for v1 branches |
| B | Fix | `admin_rule_sets.rs:283-311` | Wrapped `admin_list_rule_sets` reads in `conn.run_transaction` — closes snapshot race |
| C | Fix (refined) | `config.rs:161-163` | Doc comment names `Malformed(String)` internal-carries / Display-suppressed-per-ADR-015 contract (advisor one-line refinement on top of working-tree diff) |
| D | Fix via helper-extract | `admin_rule_sets.rs` + `e2e.rs:5387-5419` | Extracted `pub fn map_rsv_unique_violation`; A3 test routes through production mapping. No tokio::join! non-determinism, no test-only hooks |
| E | Rebuttal | `e2e.rs:5805` | Posted inline [#r3125314746](https://github.com/barrie-cork/lemmy/pull/81#discussion_r3125314746) — column pin asserted at e2e.rs:5779-5783, not in JSON snapshot |

## Local validation gates (all green)

| Gate | Time | Result |
|---|---|---|
| A3 (`admin_create_rule_set_duplicate_version_rejected`) | 25.7s | 1 passed |
| A4 (`admin_list_rule_sets_returns_versions_with_active_version_id`) | 23.5s | 1 passed |
| D1 (`case_open_pins_applied_config_snapshot_and_rule_set_version_id`) | 26.2s | 1 passed |
| `cargo check --workspace --features full` | 2m 35s | exit 0 |
| `cargo clippy -p lemmy_api --features full --no-deps -- -D warnings` | 4m 35s | exit 0 |

## CI on `6baabfd7a` (merge-gate HEAD)

- Red-flag diff scan: SUCCESS
- governance e2e: SUCCESS (~13 min — longer than typical 3-5 min)
- AI review: SUCCESS
- mergeStateStatus: CLEAN

## Process wins this round

- **Advisor one-line refinement on config.rs** — working tree had a doc-comment rewrite claiming `Malformed` "carries the original wire text" without pointing to Display-suppression. Surfaced the misread-risk before staging; advisor added `"internally (suppressed in Display per ADR-015 — see impl below)"` in one line. Closed a loop that CR-4 had just flagged on the same enum in round 1.
- **Task-notification exit-summaries verified independently** — Monitor loop watched `governance e2e` to COMPLETED|SUCCESS; cross-checked via `gh pr view` rollup. Per `feedback_task_notification_exit_summary_unreliable.md`, never trusted the notification alone.
- **Pre-phase audit skip was correct** — mid-phase resumption (all 8 tasks + round-2 already shipped on `phase-v1-AD-c`), per the rule's "When to skip" section.

## Out-of-scope state NOT touched

Untracked files listed in `git status` at session-start (runlog, harness logs, round-2/round-3 reports) remain untouched. Decision whether to stage pre-v1-AD-d belongs to next session.
