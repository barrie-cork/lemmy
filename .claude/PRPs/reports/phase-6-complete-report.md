# Phase 6 retrospective

**Branch.** `phase-6` cut from `governance-v0` @ `3bbf419da` (post PR #10/Phase 5c merge, 2026-04-19)
**Scope.** Tasks 70–78 (federation outbound + advisory inbound), 9 layered tasks across 7 agents (A, B, C, D, E, E2, F, G, G3)
**Plan.** `.claude/PRPs/plans/phase-6-federation.plan.md`
**Final HEAD.** `558c69c76` — `chore(tests): ignore third cross-test contamination flake (#45)`
**Merge6 gate.** workspace check + clippy + e2e: all exit 0; **14 passed; 0 failed; 3 ignored**

---

## 1. Tasks shipped

| Task | Layer | Agent | Commit | Summary |
|---|---|---|---|---|
| 70 | 1 | A | `fd502548c` | `add_federation_attestations` migration (federation_attestation, remote_sanction_notice tables) |
| 71 | 1 | A | `79f653f34` | Diesel models for federation tables |
| 72 | 2 | B | `aa159a0f4` | AP object types (SanctionNotice, TrustAttestation, ModerationLabel) |
| 73 | 2 | C | `8b24affb9` | AP activities (PublishSanctionNotice, PublishTrustAttestation, PublishLabel) — discriminator-bearing object stub avoids vanilla-Lemmy `SharedInboxActivities` shadowing |
| 74 | 3 | D | `8057b9f65` | Outbound publisher with builder/orchestrator split (DQ-6.6 self-resolution) |
| 75 | 3 | E2 | `c7f57bf0f` | Inbound receiver + governance_log/redaction relocation per DQ-6.6 (a) move-down |
| 76 | 4 | F | `43c3c4780` | Wire submit_jury_vote to federation_outbox; atomic-tx via shared conn pattern (DQ-6.7) |
| 77 | 5 | G/G3 | `3db4668ac` | sanction_notice_round_trip two-DB e2e + SUBSCRIPTIONS.md entry kinds |
| 78 | 3 | E | `8eb803ce9` | verify.rs signature helper (thin wrapper over activitypub_federation) |

**ADR-006 invariant preserved.** Inbound `receive_remote_sanction_notice` writes exactly two rows: one `remote_sanction_notice` with `local_case_id IS NULL`, and one `governance_log` `federation_sanction_received` entry. No mutations to `moderation_case`, `sanction`, or `person.removed`. Asserted by task 77.

**ADR-014 fork-only AP types preserved.** Discriminator-bearing object stub in PublishSanctionNotice avoids shadowing vanilla Mastodon/Lemmy `Create` activities at the `#[serde(untagged)]` `SharedInboxActivities` boundary.

**Endpoint count unchanged.** Phase 6 added no new HTTP endpoints — federation publish is a side-effect of `submit_jury_vote` when `winning_sanction.scope == FederatedRecommendation`. The 11 MVP endpoints from Phase 5c remain the v0 surface area.

---

## 2. Definition-of-done status

| Check | Result | Log |
|---|---|---|
| `cargo check --workspace --features full` | ✅ exit 0 (1m 59s) | `.claude/build-merge6-check.log` |
| `cargo clippy --workspace --no-deps --features full -- -D warnings` | ✅ exit 0 (3m 05s) | `.claude/build-merge6-clippy.log` |
| `cargo test --test e2e -p lemmy_server` | ✅ exit 0; 14 passed; 0 failed; 3 ignored (5m 22s) | `.claude/test-merge6-e2e-after-ignore45.log` |
| `sanction_notice_round_trip` (new in task 77) | ✅ passed | `.claude/test-final-task77.log` (Agent G3) |

**Three flaky tests `#[ignore]`-gated** (see §"What to carry forward" for v0-polish prioritisation).

---

## What surprised us

**Agent F's signature deviation forced by DQ-6.7.** The handoff brief specified `send_local_sanction_notice(case_id, &context)`, but Agent F discovered that signature would either (a) open a second nested `run_transaction` and break the atomic-tx invariant against the `submit_jury_vote` post-decision tx, or (b) commit the federation publish before `submit_jury_vote`'s outer rollback — violating Watch 5. Agent F self-resolved DQ-6.7 by changing the signature to `(case_id, conn, context)` so the orchestrator participates in the existing tx. This was correct judgment under the attribution-integrity rule (`impl-self-resolved`, not `advisor`), and the integration test at task 77 validated the change end-to-end.

**Three pre-existing test flakes surfaced only at Layer 5 baseline.** When Agent G ran the full e2e suite against clean phase-6 tip *before* applying task 77 wip, two tests failed (`phase1_migrations_round_trip` from task 70's missing revert-list extension, `sponsor_liability_with_founder_multiplier` from a known random-jury fallback bug). When Agent G ran the suite *with* task 77 wip applied, a third test failed (`ineligible_user_cannot_be_picked_for_jury` — exposed by adding a 17th test extending suite duration past the contamination tolerance window). The merge gates between Layers 1–4 only ran `cargo check` + clippy + `cargo test --no-run`; they never executed `cargo test --test e2e`. The flakes have been accumulating since Phase 5b but were invisible until the full-suite run at Layer 5.

**Agent G's judgment degraded at ~250k tokens.** Filed DQ #39 with `impl-self-resolved` framing the failures as "intrinsic random flakes (5/6 probability), commit anyway." The on-disk evidence contradicted this — `ineligible` PASSED in baseline (no task 77) and FAILED with task 77 in tree, same alphabetical order, proving deterministic state contamination, not randomness. The advisor halted Agent G, wrote a 290-line handoff from on-disk test-log evidence (more reliable than letting Agent G self-handoff from degraded context), and respawned. Agent G2 was halted ~1 minute later when the user issued a cleaner directive (`#[ignore]` + ship + file issues), and Agent G3 executed it cleanly in three commits.

**Auto-handoff produced better context than Agent G could have.** Writing the handoff myself from preserved logs (test-baseline, test-task77-full, test-task77-full2, test-ineligible-isolated, test-flake-check) yielded a more accurate root-cause hypothesis than Agent G's degraded analysis would have. Per `feedback_explore_before_planning_on_reviews.md`, advisor-side synthesis from artifacts beats agent-side narrative when agent context is stale.

---

## What to change

**Layer merge gates need full e2e runs, not just compile.** Phase 6's gates 1–4 only ran `cargo check --workspace --features full` + `cargo clippy ...` + `cargo test --test e2e --no-run`. Three pre-existing flakes accumulated invisibly across Phase 5b and surfaced all at once at Layer 5 (Agent G's first full run). For Phase 7 / v0-polish:

- Add `cargo test --test e2e -p lemmy_server` to merge gates 2, 3, 4 (the runtime cost is ~6 minutes per gate; acceptable for sequential merges)
- The cost of catching a contamination class earlier (when only 1 test was contaminated, not 3) is dramatically lower than the cost of triaging 3 simultaneous failures at the final agent
- Update `.claude/rules/pre-phase-harness-audit.md` Probe 6 to include "run the full e2e suite at phase-cut to establish baseline" — that single command would have surfaced #43 and the existing #45-class behavior on day 1

**Attribution rule needs an enforcement check, not just documentation.** The DQ #37 attribution incident on 2026-04-19 (advisor labelling an impl-session DQ as `advisor`) led to commit `ec41597d1` introducing the rule. Agent G respected it correctly (DQ #39 used `impl-self-resolved`). But there is no automated check — a future agent could still write `"answered_by": "advisor"` and only catch it on review. Suggested mitigation:

- Add a pre-commit hook or CI lint that scans `.claude/decision-queue.json` for `"answered_by": "advisor"` writes in the same commit author as the change touching it (advisor commits should be the only ones writing that label)
- Cost: ~30 minutes for the lint; probably one false-positive workflow in v1

**Agent token budget warning.** Agent G hit ~250k tokens with significant tool-call repetition before judgment degraded. The pattern:

- 0–100k: focused, executing brief
- 100–200k: still effective, exploring
- 200k+: degraded — re-runs same checks, mis-classifies evidence, fails to read its own prior outputs

For Phase 7 / v0-polish: add a self-imposed budget check at ~150k tokens — agent should write a brief progress note and explicitly request advisor handoff before judgment degrades. The `feedback_subagent_model_and_effort.md` rule already covers model choice; this would extend it to budget self-awareness.

**Worktree management overhead.** Phase 6 used 8 git worktrees (advisor + 7 agent worktrees A, B, C, D, E2, F, G). Submodule init was needed per worktree (Agent A and others hit `crates/email/translations` empty). Submodule init is documented in `feedback_worktree_submodules_not_auto_init.md`, but it's still a per-worktree manual step. For Phase 7: add an `--init-submodules` shortcut to the agent worktree setup (or extend `task-hopper.sh start` to run `git submodule update --init --recursive` automatically).

---

## What to carry forward

### Three flaky tests (v0-polish, prioritised)

**#45 (HIGHEST PRIORITY) — production-code finding, not test hygiene.** `sponsor_liability_with_founder_multiplier`'s NotFound flake reveals that `admin_assign_jury`'s fallback path (`config.jury.fallback_on_small_pool=true`) produces inconsistent state where `accept_jury_assignment` re-check rejects rows the assigner inserted. This is a real eligibility-contract bug: when the random pool is too small, fallback bypasses ineligibility filtering, but the assignment row's later acceptance hits the second-stage filter and fails. The fix touches `crates/api/api/src/governance/admin_assign_jury.rs` and `accept_jury_assignment.rs`, not the test.

**#42 (medium) — test infrastructure.** `ineligible_user_cannot_be_picked_for_jury` fails when other tests run first (cross-test contamination via testcontainer/pool reuse). Fix path: investigate `governance_fixtures::start_postgres` lifecycle, add explicit container teardown between tests, or reset `LEMMY_DATABASE_URL` defensively.

**#43 (lowest) — one-line trivial.** `phase1_migrations_round_trip` revert-list needs to cover Phase 6's two new tables. Bump `.limit(N)` by 2 or extend the explicit migration-name list.

### DQ #37 governance_log relocation cleanup

`crates/db_schema/src/source/governance/governance_log.rs` was moved down from `lemmy_api` per DQ-6.6 (a) inbound resolution. This works but is asymmetric with the outbound side (which uses Agent D's builder/orchestrator split). Consider in v0-polish: either propagate the move-down to outbound for consistency, or move governance_log back up and use Agent D's pattern for inbound. Tracked in `project_brehon_post_phase6_cleanup.md` item 9.

### Cosmetic debt — Agent C's ObjectStub bridge

Agent D bridges Agent C's discriminator-bearing object stub via a JSON round-trip (`decode_sanction_notice_object` re-inserts the `type` discriminator before deserialising). Per Agent C's TODO comment, the cleaner shape would be the wrapper carrying the typed protocol directly. Cosmetic only — current behavior is correct. Defer to v0-polish if at all.

### Federation entry kinds in SUBSCRIPTIONS.md

Task 77 added four entry kinds (`federation_sanction_sent`, `federation_sanction_received`, `federation_attestation_sent`, `federation_attestation_received`). v1 messaging bridge consumers will need to subscribe to these — already documented per the SUBSCRIPTIONS.md stability contract.

### GitHub issue tracking

| # | Title | Issue URL |
|---|---|---|
| 42 | bug(tests): ineligible cross-test contamination | https://github.com/barrie-cork/lemmy/issues/42 |
| 43 | bug(tests): phase1_migrations revert-list bump | https://github.com/barrie-cork/lemmy/issues/43 |
| 45 | bug(tests): sponsor_liability NotFound flake (production-code) | https://github.com/barrie-cork/lemmy/issues/45 |

---

## Decision queue (Phase 6)

Nine DQ entries resolved during Phase 6:

| ID | From | Answered by | Decision |
|---|---|---|---|
| 31 | planner | advisor | DQ-6.1: add `received_at TIMESTAMPTZ NOT NULL DEFAULT now()` column |
| 32 | planner | advisor | DQ-6.2: signature column carries outer Create activity id (`activity.id`) |
| 33 | planner | advisor | DQ-6.3: rely on existing `ReceivedActivity::create` dedup, no second guard |
| 34 | planner | advisor | DQ-6.4: env-var cleanup left as-is, mirror existing pattern at e2e.rs:2195+ |
| 35 | planner | advisor | DQ-6.5: branch on `SanctionScope::FederatedRecommendation` (not action) |
| 36 | impl | impl-self-resolved | DQ-6.6 outbound: builder/orchestrator split (Agent D shipped builder; Agent F shipped orchestrator) |
| 37 | impl | advisor | DQ-6.6 inbound: option (a) move-down — relocate `governance_log::append` to `lemmy_db_schema` |
| 38 | impl | impl-self-resolved | DQ-6.7: Agent F signature change `(case_id, conn, context)` for atomic-tx |
| 39 | impl | impl-self-resolved | DQ-6.8: classify three test failures (Agent G's framing was wrong; corrected by advisor halt + re-spawn) |

**Attribution-rule incident:** DQ #37's `"answered_by": "advisor"` write by an impl session led to `.claude/rules/decision-queue.md` "Attribution integrity" section (commit `ec41597d1`). The label was corrected post-hoc. Future incidents should be caught by the lint suggestion in §"What to change."

---

## Branch + PR

- **Branch:** `phase-6` @ `558c69c76`
- **Cut from:** `governance-v0` @ `3bbf419da`
- **Commits:** 24 (incl. plan, briefs, all 9 task commits, 7 hopper marks, 4 merge points, 3 ignore-flake commits)
- **PR target:** `barrie-cork/lemmy:governance-v0` (per `gh-pr-fork-target.md`, must use `--repo barrie-cork/lemmy`)
- **Merge mode:** `--merge`, NEVER squash (per Phase 5b orphan-commits lesson; preserves task-per-commit history for retros and CodeRabbit review)

---

## Amendment — PR #46 CodeRabbit review cycle (2026-04-19)

CodeRabbit Pro reviewed PR #46 at `558c69c76` and surfaced 23 findings:
2 Critical, 5 Major, 16 Minor/Trivial. Review response ran across two
parallel impl sessions coordinated via `.claude/PRPs/phase-6-runlog/01-phase-6-progress.md`.

### Fixes landed

| # | Severity | Site | Fix SHA | Owner |
|---|---|---|---|---|
| 15 | 🔴 Critical | `submit_jury_vote::process_vote` — idempotency guard below quorum-tally early-return | `41f1d0379` | Impl1 |
| 19 | 🔴 Critical | `Publish*Notice::verify` — inner `object.actor` unbound from outer `activity.actor` (AP spoof) | `d70610980` | Impl1 |
| 13 | 🟠 Major | `task-hopper.json` — absolute worktree paths (privacy) | `addc0c9ab` | Impl2 |
| 22 | 🟠 Major | `sanction_notice_round_trip` — missing no-auto-apply DB-state asserts on B | `455a7dbe4` | Impl1 |
| 11 | 🟡 Minor | `decision-queue.md` — author-based detection rewritten to subject-pattern match | `728659a24` | Impl2 |
| 10 | 🟡 Minor | This retro — "Eight DQ entries" → "Nine" | `e64254261` | Impl1 |
| 21 | 🟡 Minor | `inbox.rs` + `remote_sanction_notice.rs` — `[99 ADR-006]` bare-citation linkified | `e64254261` | Impl1 |
| 7  | 🟡 Minor | `HANDOFF-PROMPT.md` unlabeled fences (python) | `e64254261` | Impl1 |
| 4  | 🟡 Minor | `agent-c.md` MD040 | `297341c8b` | Impl2 |
| 6  | 🟡 Minor | `agent-g.md` MD040 | `297341c8b` | Impl2 |
| 9  | 🟡 Minor | `plan-phase-6-federation.plan.md` MD040 (text) | `297341c8b` | Impl2 |
| 12 | 🟡 Minor | `task-hopper.md` MD040 | `297341c8b` | Impl2 |

### Findings rebutted / deferred

| # | Severity | Site | Rationale | Tracker |
|---|---|---|---|---|
| 1  | 🟠 Major | `prp-ralph-stop.sh:51` | Script-polish; zero v0 user impact | v0-polish |
| 14 | 🟠 Major | `task-hopper.schema.json:147` — schema lifecycle invariants | Larger schema refactor not appropriate in review patch | [#47](https://github.com/barrie-cork/lemmy/issues/47) |
| 20 | 🟠 Major | `redaction.rs:99` — scrub_json control flow | Non-issue for v0: ADR-015 schema contract is the primary barrier (payloads carry only IDs/URLs/enums); scrub_json is defence-in-depth | v0-polish |
| 17 | 🟡 Minor | `publish_trust_attestation.rs:10` — "unused Object trait import" | False positive: `Object::id()` used at lines 146-147 | — |
| 2/3/5/8/16/18/23 | 🟡 Minor | Various MD-lint + helper-extraction | Runlog-archival content (don't rewrite execution record) or defer-to-refactor-sweep | v0-polish |

### Lessons added to durable rules

- **CLAUDE.md §PR review discipline** (`fa78dd8b5`): CodeRabbit Critical
  findings block merge. Three instances this PR alone (row-scoping,
  control-flow, AP actor-binding) caught classes that local advisor +
  e2e missed. Memory: `feedback_coderabbit_block_merge_critical.md`.
- **`decision-queue.md` attribution detection** (`728659a24`): rule
  now matches commit-subject pattern `^(chore|docs)\((advisor|decision-queue)\)`
  rather than hardcoded author name. Solo-dev single-author repos can't
  discriminate by author identity.
- **Cold-build gate for rapid-commit phases** (new carry-forward): during
  this review cycle, a transient clippy false-red at 16:06Z flagged
  `publish_trust_attestation.rs:146-147` `Object` trait out of scope —
  the file at HEAD already had the import. Cold rebuild at 16:19Z came
  back clean. Hypothesis: warm incremental cache in `target/` got into a
  partial state between test-compile and lib-compile feature-resolution.
  Layered-agents phases with rapid task-per-commit cadence should gate
  commit-greenness on cold workspace-check + test-target compile, not
  warm-cache runs. Memory candidate for v1 work.

### CI gates at final HEAD

Final HEAD (`phase-6 @ <this amendment commit>`) validated by Impl1 at 2026-04-19 16:20-16:25Z:

- `cargo check --workspace --features full`: ✅ exit 0
- `cargo clippy --workspace --no-deps --features full -- -D warnings`: ✅ 0 errors, 0 warnings
- `cargo test --test e2e --no-run -p lemmy_server`: ✅ cold compile clean
- `cargo test --test e2e -p lemmy_server`: ✅ 14 passed / 0 failed / 3 ignored (308s)

### Review-cycle commit count

14 review-response commits added after initial PR push (`558c69c76`):
2 Critical fixes, 3 Major fixes, 3 docs sweeps, 1 CLAUDE.md rule,
1 decision-queue rule, 1 CI size-gate, 2 infra/AI-review, plus
coordination runlog entries (6 commits) that carry no code impact
but document the two-impl hand-off.

Full list (oldest → newest): `8b92d14be`, `729c4b768`, `41f1d0379`,
`d70610980`, `addc0c9ab`, `fa78dd8b5`, `728659a24`, `455a7dbe4`,
`250dd9066`, `e64254261`, `8297c5066`, `a7c0a6053`, `ad2459b65`,
`3a42f43c2`, `297341c8b`, `6c5494382`, plus this amendment commit.


---

## Amendment — Bucket C (2026-04-19, CodeRabbit re-review on `8ee45a3ca`)

CodeRabbit re-reviewed `8ee45a3ca` (the initial amendment) and flagged
**10 inline + 2 outside-diff + 2 duplicate** findings (severity spread:
1 Critical, 4 Major, 3 Minor, 2 Trivial, plus duplicates). Per user
directive for critical-only focus, we landed the Critical + one ADR-adjacent
Major in this PR, deferring the rest to GH issue #54.

### Fixed in PR #46 Bucket C

| # | Severity | Commit | Fix |
|---|---|---|---|
| 1 | 🔴 Critical | `0ff06abc9` | Lock-race compare-and-release in `task-hopper.sh` — only release the lock when this process actually owns it |
| 4 | 🟠 Major | `99dce6be2` | ADR-006 atomicity — wrap `insert_remote_sanction_notice` + `governance_log::append` (and trust-attestation analogue) in one `conn.run_transaction()` so the "exactly two rows per notice" invariant holds under failure |
| 5 | 🟠 Major | `9aa1a778a` | `TH_ISSUE_URL` env export — move var into subprocess prefix so Python actually sees it (Impl2) |

### Deferred to follow-up (GH issue #54)

- **#2** hidden `actor_pseudonym` write in send path (ADR-015 edge — log creation event)
- **#3** hash-chain causality — swap `case_decided` / `federation_sanction_sent` order
- **#6** DQ-6.7 wording stale
- **#7** SQL enum typenames in plan:493
- **#8** task-hopper.md rule contradiction
- outside-diff #1 workflow label
- #9/#10 Trivials (v0-polish)

### Merge6b validation on Bucket C tip `19cb13aea`

- `cargo check --workspace --features full`: ✅ exit 0 (log 6203B, 2m)
- `cargo clippy --workspace --no-deps --features full -- -D warnings`: ✅ 0 errors, 0 warnings (2m 58s)
- `cargo test --test e2e --no-run -p lemmy_server`: ✅ cold compile clean (4m 41s)
- `cargo test --test e2e -p lemmy_server`: ✅ 14 passed / 0 failed / 3 ignored (329s)
  — including `sanction_notice_round_trip` which exercises the #4 atomicity refactor

### Bucket C commit list

Local commits since `8ee45a3ca`:

- `0ff06abc9` fix(hopper): compare-and-release lock token — #1 Critical
- `ce9bdd08c` docs(handover): PR #46 Bucket C merge — session-continuity prompt
- `d08056608` chore(runlog): Impl1 Bucket C intake
- `4c0128c93` chore(runlog): Impl1 file-level claim on inbox.rs
- `9aa1a778a` fix(hopper): export TH_ISSUE_URL into py_edit subprocess — #5 (Impl2)
- `99dce6be2` fix(governance): atomic insert + log append in inbound receivers — #4
- `19cb13aea` chore(runlog): Impl1 status — #4 shipped, merge6b 3/4 green
- `693453fab` chore(runlog): merge6b GREEN — 14/0/3 on phase-6 tip
