---
name: e2e-nextest-filter-groups
description: Canonical nextest -E filter expressions for each e2e test group, enabling scoped validate-pending-laptop runs instead of always running all 41 tests.
metadata:
  type: feedback
---

# E2E nextest filter groups

Use these expressions as the `e2e_filter` field in `validate-pending-laptop` DQ
entries to scope the e2e run to the tests relevant to the task. The advisor
appends `-E "<value>"` to the nextest command when this field is present and
non-null.

## Syntax reference

- `test(~name)` — substring match on test function name (most useful)
- `test(=name)` — exact match
- `binary(=e2e)` — all tests in the e2e binary (used in .config/nextest.toml for
  concurrency scoping; not needed in `-E` because `--test e2e` already selects
  the binary)
- `A and B`, `A or B`, `not A` — boolean combinators

## Canonical group → filter table

| Group | `-E` value | Tests covered | Approx wall-clock |
|---|---|---|---|
| Infrastructure smoke | `test(~container_boots) or test(~template_dump)` | 2 | ~45s |
| Migration integrity | `test(~migration) or test(~revert) or test(~reapply)` | 3 | ~1 min |
| Core schema / case | `test(~moderation_case) or test(~hash_chain) or test(~list_open_cases)` | 3 | ~1 min |
| Jury queue & views | `test(~jury_queue) or test(~modlog_view)` | 2 | ~45s |
| Jury voting | `test(~submit_jury_vote)` | 7 | ~2.5 min |
| Reputation / endorsement | `test(~revoke_endorsement) or test(~window_expiry) or test(~revocation_during)` | 11 | ~4 min |
| Grace check | `test(~grace_check) or test(~apply_sponsor)` | 6 | ~2 min |
| Snapshot / backfill | `test(~snapshot) or test(~backfill) or test(~seed_migration)` | 4 | ~1.5 min |
| Config / misc governance | `test(~config_parity) or test(~scope_parse) or test(~username)` | 3 | ~1 min |
| **Full suite** | *(omit `-E`)* | 41 | ~12 min |

Wall-clock estimates assume nextest `threads-required=4` on an 8-core machine
(2 containers simultaneously, ~22s per test). Actual time varies ±20% based
on Docker startup jitter.

## How to pick the right filter for a brief

Walk the task's IMPLEMENT files against this table. Match on the governance
area the task touches:

- Task edits `submit_jury_vote` handler → `test(~submit_jury_vote)` (7 tests)
- Task edits endorsement revocation → `test(~revoke_endorsement) or test(~window_expiry) or test(~revocation_during)` (11 tests)
- Task edits grace-check cron → `test(~grace_check) or test(~apply_sponsor)` (6 tests)
- Task edits migration → `test(~migration) or test(~snapshot) or test(~backfill)` (7 tests)
- Task touches multiple areas OR scope is unclear → omit `e2e_filter` (full suite)
- bm-merge gate → always full suite (omit `e2e_filter`)

## Multi-lane safety

Two lanes running scoped e2e simultaneously (e.g. jury 7 tests + reputation 11
tests) peak at ~4 containers live — well within Docker Desktop's named-pipe
concurrency on a 64 GB machine. Two simultaneous full-suite runs (41 + 41 tests,
~8 containers) may contend; serialize those at the bm-merge gate.

## Full test inventory (as of 2026-06-01, 41 tests in e2e.rs)
<!-- verified: grep -c '#\[tokio::test\]' crates/server/tests/e2e.rs, 2026-06-01 — re-verify before citing -->

| Line | Function | Group |
|---|---|---|
| 44 | `postgres_container_boots` | infrastructure |
| 79 | `template_dump_capture` | infrastructure |
| 924 | `can_insert_moderation_case` | core schema/case |
| 968 | `governance_log_hash_chain_holds` | core schema/case |
| 1220 | `phase1_revert_list_matches_disk` | migration |
| 1230 | `test_phase1_migrations_forward` | migration |
| 1546 | `test_phase1_migrations_revert` | migration |
| 1882 | `test_phase1_migrations_reapply` | migration |
| 1955 | `v1_jm_a_backfill_populates_v0_snapshot` | snapshot/backfill |
| 2258 | `list_open_cases_returns_seeded_rows` | core schema/case |
| 2319 | `jury_queue_view_returns_assignments` | jury queue/views |
| 2419 | `modlog_view_returns_published_entries` | jury queue/views |
| 3153 | `config_parity_round_trip` | config/misc |
| 3211 | `v1_jm_a_seed_migration_is_idempotent` | snapshot/backfill |
| 3901 | `capability_change_entries_reachable_via_modlog_crate` | core schema/case |
| 4899 | `underscore_prefix_usernames_still_register` | config/misc |
| 7485 | `scope_parse_wire_rejects_negative_community_id` | config/misc |
| 9492 | `submit_jury_vote_severe_panel_meets_threshold` | jury voting |
| 9648 | `submit_jury_vote_deadlock_flips_to_admin_review` | jury voting |
| 9866 | `submit_jury_vote_writes_appeal_window_default` | jury voting |
| 9984 | `submit_jury_vote_writes_appeal_window_live_config` | jury voting |
| 10307 | `submit_jury_vote_concurrent_votes_decide_exactly_once` | jury voting |
| 11854 | `revoke_endorsement_self_succeeds_updates_surety_and_recomputes_snapshots` | reputation |
| 11991 | `revoke_endorsement_admin_succeeds_under_threshold` | reputation |
| 12068 | `revoke_endorsement_re_revoke_returns_existing_revoked_at_no_log_no_severance` | reputation |
| 12159 | `revoke_endorsement_non_sponsor_non_admin_rejects_with_not_found` | reputation |
| 12230 | `revoke_endorsement_empty_reason_rejects` | reputation |
| 12293 | `revoke_endorsement_rate_limit_enforces_unless_admin_bypasses` | reputation |
| 12426 | `revoke_endorsement_severs_grace_window_single_sponsor_case` | reputation |
| 12532 | `revoke_endorsement_multi_sponsor_any_revocation_severs_chain` | reputation |
| 12621 | `revoke_endorsement_no_pending_case_no_severance_only_revoked_log` | reputation |
| 12820 | `grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries` | grace check |
| 12930 | `grace_check_escapes_case_when_sponsor_revoked_after_decided_at` | grace check |
| 13121 | `grace_check_no_op_when_grace_expires_at_in_future` | grace check |
| 13229 | `grace_check_per_case_isolation_skips_bad_case_processes_good_case` | grace check |
| 13362 | `grace_check_batch_size_config_caps_iteration` | grace check |
| 13654 | `submit_jury_vote_transitions_to_pending_for_liability_bearing_sponsored_case` | jury voting |
| 13896 | `submit_jury_vote_preserves_v0_decided_for_no_sponsor_target` | jury voting |
| 14090 | `submit_jury_vote_no_action_skips_liability_machinery` | jury voting |
| 14285 | `apply_sponsor_liability_wrapper_preserves_v0_outputs` | grace check |
| 14633 | `revocation_during_window_escapes_full_lane` | reputation |
| 14958 | `window_expiry_fires_full_lane` | reputation |
| 15174 | `backfill_of_mid_flight_v0_to_v1_deploy` | snapshot/backfill |

**Maintenance:** when a new test is added to e2e.rs, add a row to this table
and verify it falls into an existing group (or create a new group + filter
expression). The anchor for the uniqueness gate in the advisor-orchestrator
brief-authoring rules applies here too — grep the function name before adding.
Before citing the test count in a brief, run:
`grep -c '#[tokio::test]' crates/server/tests/e2e.rs`
