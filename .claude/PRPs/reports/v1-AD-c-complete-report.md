# v1-AD-c completion report

**Phase:** v1 admin dashboard sub-phase C — rule-set CRUD + case-open snapshot + #77/#78 carry-forward
**Branch:** `phase-v1-AD-c`
**Merged:** 2026-04-22T16:19Z into `governance-v0` as merge-commit `cf89890f3`
**PR:** [#81](https://github.com/barrie-cork/lemmy/pull/81) (15 commits, ~4200 additions / ~900 deletions across 3 CR rounds)

## What shipped

Eight feature tasks + three CR-fix rounds.

### Feature tasks (final stack)

| # | Commit | Summary | Closes |
|---|---|---|---|
| t0 | `774374145` + `6a3707228` | Plan file + narrowed clippy DoD + pre-phase audit report | — |
| t1 | `c8c29c973` | `Scope::parse_wire` rejects non-positive community_id | #78 |
| t2 | `746ebc194` | 5 rule-set DTOs + `AdminConfigAuditEntry.previous_from` field | — |
| t3 | `4706715d6` | `admin_create_rule_set` + `admin_list_rule_sets` handlers + `ENTRY_KIND_RULE_SET_VERSION_CREATED` | — |
| t4 | `d623bcff5` | Pre-tx provenance threaded into `admin_config_changed` audit payload | #77 |
| t5 | `a15bf3d04` | Case-open pins `applied_config_snapshot` + `rule_set_version_id` | — |
| t6 | (no-op) | Governance-log registry already updated in t3 | — |
| t7 | `f7c748c16` | `POST /admin/rule-sets` + `GET /admin/rule-sets` route wiring | — |
| t8 | `3b692c1b6` | 8 e2e tests covering rule-set CRUD + #77/#78 + case-open pin | — |

### CR-fix rounds (post-PR-open)

| Round | Commit | Shipped |
|---|---|---|
| 1 | `5c04e6ec2` | 4 Major findings (0 Critical) addressed inline |
| 2 | `6c8fc3211` | Shell-parity test + PRD §8.4 aligned with t4 payload extension |
| 3 | `6baabfd7a` | 3 Major fixes (B snapshot race / C doc / D helper-extract) + 2 rebuttals posted inline |

Plus `66749bb71` — out-of-phase harness commit (3 lifecycle hooks + Telegram/webhook channels + upstream-rebase routine) that landed during PR polling. Noted in retro12; harness-side CR findings deferred to chore issue #85.

## Governance-log registry update

Added 1 new `ENTRY_KIND_*` const:

- `ENTRY_KIND_RULE_SET_VERSION_CREATED` (`rule_set_version_created`) — emitted by `admin_rule_sets.rs::admin_create_rule_set`. Payload: `{community_id, version, parent_id, text_sha256, rule_set_version_id, config_id, activated_at}`.

Total v0+Phase 6+v1-AD-a+v1-AD-c entry_kinds: **26** (19 v0 + 4 Phase 6 + 2 v1-AD-a + 1 v1-AD-c).

Registry file `.claude/rules/governance-log-entry-kind-registry.md` §"v1-AD-c entry kinds" section added with canonical const + call site + payload semantics.

## CR ledger

Two review rounds, zero Critical findings across both.

### Round 1 — 4 Major, all shipped in `5c04e6ec2`

Findings integrated in-phase per `feedback_coderabbit_block_merge_critical.md` protocol (even though Critical rule didn't trigger, local precedent now favours in-phase Major fixes when the PR is still open).

### Round 2 — 5 Major, 3 fixes + 2 rebuttals

Four-bucket triage per `feedback_pr_review_triage_pattern.md`:

| # | Bucket | Disposition |
|---|---|---|
| A — DTOs expand v0 surface | **Rebuttal** | v1-AD-c is v1 scope, not v0 baseline (11-endpoint cap is for governance-v0). Posted inline with sub-PRD citation. |
| B — admin_list_rule_sets snapshot race | **Fix (C)** | Wrapped `versions` + `active_version_id` reads in `conn.run_transaction`. Shipped in `6baabfd7a`. |
| C — ScopeParseError doc drift | **Fix (A)** | Initial alignment in `5c04e6ec2`; refined in `6baabfd7a` to cross-reference Display-suppressed-per-ADR-015. |
| D — test recreates handler mapping | **Fix (C)** | Extracted `pub fn map_rsv_unique_violation` helper, called from both production and test. Shipped in `6baabfd7a`. |
| E — snapshot pin assert missing key | **Rebuttal** | Pin is on `moderation_case.rule_set_version_id` column (strictly asserted at `e2e.rs:5779-5783`), not in `applied_config_snapshot` JSON — adding it to JSON would break `REQUIRES_RE_JURY_KEYS` metadata-parity invariant. Posted inline with test line and metadata cross-ref. |

**Both rebuttals posted as inline file-line comments on the PR thread** (A at `api_common/governance.rs:585`, E at `e2e.rs:5805`). Evidence links included per `feedback_coderabbit_triage_four_buckets_confirmed.md`.

## Carry-forward chore issues (filed post-merge)

All four pre-authorised during phase-close advisor review, filed after merge:

| # | Title | Notes |
|---|---|---|
| [#82](https://github.com/barrie-cork/lemmy/issues/82) | Clear e2e.rs + governance test-target clippy debt (135 errors at v1-AD-c base) | DoD narrowed to drop `--all-targets` per `774374145`; this issue clears debt for future restore. |
| [#83](https://github.com/barrie-cork/lemmy/issues/83) | Fix lemmy_api_crud reqwest_middleware transitive compile break | Upstream-originated, routed around during v1-AD-c via `-p lemmy_api`. |
| [#84](https://github.com/barrie-cork/lemmy/issues/84) | Update admin-config-write.sh to emit previous_value + previous_from | Option A per PRD §8.4 amendment in `5c04e6ec2`. Shell wrapper ↔ handler payload byte-parity. |
| [#85](https://github.com/barrie-cork/lemmy/issues/85) | Address 12 CR findings on .claude/hooks/ + channels/ + routines/ | Harness findings from out-of-phase commit `66749bb71`. |

Issue originally on list ("replace tokio::join! race test with deterministic helper") was **resolved within the phase** via Fix D helper-extract in round 2 — dropped from carry-forward.

## Risk register close-out

All risks from the advisor notes (`00-advisor-notes.md`) cleared:

- r1 (NOT5 byte-perfect in t4 commit + PR body) — cleared at t4 commit
- r2 (DTO add breaks 2 construction sites) — cleared via grep-before-commit
- r3 (3-write tx UNIQUE race + savepoint rollback) — cleared via `admin_emergency_remove.rs` mirror pattern
- r4 (case-open Instance match) — cleared at t5
- r5-r6 — cleared in earlier rounds
- r7 (silent-failure in moderator check / CR-2) — fixed round 1
- r8 (untrusted-input echo in Display / CR-4) — fixed round 1 + refined round 3
- r9 (admin_list_rule_sets snapshot race / CR round 2 B) — fixed round 3
- r10 (test/handler mapping duplication / CR round 2 D) — fixed round 3 via helper extract

## Retrospectives (14 total — 12 from phase body + 2 at phase close)

Full text at `.claude/PRPs/v1-AD-c-runlog/00-advisor-notes.md` under "Retro candidates".

**New patterns worth memory-level promotion:**
- retro13 + retro14: two advisor-instruction-mismatch catches by Impl, both reversed cleanly. The pattern is already in memory (`feedback_advisor_instruction_mismatch_stop_and_ask.md`) — phase validated it twice more.
- retro8: CI runs integration tests only (no library unit tests) — already memorised as `feedback_ci_runs_integration_tests_only.md`.
- retro12: gh pr view headRefOid probe should be part of every phase-close stage to detect out-of-phase activity — candidate for new process rule. Relates to `feedback_parallel_agent_diff_collision_detection.md` but different context (single-session + background PR polling vs parallel agents).

## CI state at merge

| Check | Conclusion |
|---|---|
| Red-flag diff scan | ✅ SUCCESS |
| governance e2e | ✅ SUCCESS |
| AI review | ✅ SUCCESS |

Zero skipped checks. AI review was **not** skipped despite prior-phase 413 oversize pattern (`feedback_ai_review_413_oversize.md`) — this PR's diff was small enough to pass the 28KB cap after the first few rounds of incremental commits.

## Protocol observations

- **Branch-manager + PM split** (per `feedback_branch_manager_pm_split.md`) held cleanly. Advisor never committed on the phase branch during active impl; Impl never wrote to `.claude/decision-queue.json` or PR descriptions. File-ownership by convention worked.
- **Pause-per-commit** added ~2-3 min/task of advisor wait time, caught ~1 defect per 2 tasks pre-CR (round-3 config.rs doc-comment misread risk caught pre-stage; t5 community_id:None→Instance drift caught pre-commit).
- **Machine-format runlog** saved ~3.8K tokens/iteration vs the earlier narrative format. Cold-resume from runlog worked (Impl session cleared mid-phase, re-oriented from runlog alone, surfaced t2 plan drift correctly).
- **Four-bucket CR triage** held across both rounds. No findings escalated, no rebuttals re-flagged by CR on re-pass.

## Next sub-phase

v1-AD-d starts from `governance-v0` at HEAD `cf89890f3`. Advisor to prep plan-phase brief.

**Handover artefacts:**
- Plan file: `.claude/PRPs/plans/v1-admin-dashboard-c.plan.md` (merged with PR)
- Runlog: `.claude/PRPs/v1-AD-c-runlog/` (advisor notes + progress log + task0 narrative)
- CR raw captures: `.claude/cr-review-pr81*.{txt,json}` + `.claude/cr-inline-pr81*.{txt,json}`
- Resume briefs: `.claude/PRPs/reports/v1-AD-c-*-resume-state.md` (two of them — advisor cold-resume + round-3 resume)
