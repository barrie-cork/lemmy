# Retro — v1-federation-inbound-a (federation-inbound foundation: peer trust + inbox nonce + remote moderation labels + Phase-6 model ext + e2e probes)

**Phase branch:** `phase-v1-federation-inbound-a` (merged + deleted)
**PR:** #138 → `governance-v0`, merge commit `7af873731`, merged 2026-05-19T00:33:36Z
**Canonical ref (structure mirrored):** `.claude/PRPs/reports/v1-ship-1-r2-retro.md`

## 0. Outcome

SHIPPED. 9 plan tasks (Cohort A 1–5 + serial Cohort B 6–9) + fix-impl 1–5 + fix-cr-1. Phase-2 e2e GREEN at three checkpoints: #267 @122187ebd (91/0/5, pre-fix-cr-1), #271 @f432faa3f (91/0/5, post-fix-cr-1), #272 @cec5a6556 (**97/0/5, post-gov-v0-merge** — +6 gov-v0 top-level tests). All 29 CR/Copilot findings triaged (7 fix-in-pr, 1 carry-forward, 21 wont-fix). `/brehon-verify` 3/3 §16a stories ✓. PR mergeable:CLEAN, all DQ resolved (pending=[]). 72 commits.

## 1. The arc (what actually happened)

1. Cohort A (Tasks 1–5) + serial Cohort B (6–9) shipped + §5.2 workspace-validated (pre-compaction segment; DQ #242/#244/#246/#248/#249/#251/#252/#264/#266).
2. Phase-2 e2e regression on `v1_jm_a_backfill_populates_v0_snapshot` → user-authorised Option A "fix forward" → fix-impl-5 (down.sql exact-inverse of up.sql) → e2e #267 GREEN.
3. bm-pr #326 → PR #138 opened. bm-poll-cr #327 ingested 29 findings. **bm-triage #328 FALSE-SUCCESS**: reported `done` but wrote triaged YAML+comment to its per-job worktree which the daemon CLEANED post-task (gitignored runtime artifacts → lost). Caught via post-condition verify; full triage recovered from #328 task log + reconstructed durably on lane worktree.
4. USER-GATE-3: "fix all 7 in-PR". fix-cr-1 #329 (cr-21 critical enum + cr-22/23 + copilot-3/4/5/6). Advisor pre-verified cr-21 against migration/enums.rs/config.rs (genuine defect). #329 finalize-merge race reconciled lane-safe (daemon-local `ea930b25e` clean descendant → FF). DQ #270 §5.2 validate-pending-laptop GREEN.
5. USER-GATE-5 BLOCKED: PR #138 CONFLICTING — gov-v0 advanced +23 commits (v1-ship-1-r2 PR#137 merge + retro lesson thread) while fed-in-a in flight. Exactly 2 conflicts, both additive (e2e.rs EOF append-zone + decision-queue.json cross-lane). User chose Option A (advisor resolves inline; precedent ship-1-r2 33edf7848).
6. e2e.rs resolution took **4 iterations** — phase branch's `-a` probe insertions shifted line numbers, marker-mangled boundaries misleading; v4 used index-stage blobs (:1/:2/:3) + per-branch grep for exact ranges. decision-queue.json: canonical union+dedupe (254 resolved, #229 resolved-on-gov-v0). Merge `cec5a6556`. Post-merge cargo-check GREEN (resolution proven sound) → e2e #4 GREEN 97/0/5.
7. USER-GATE-5 confirmed → advisor `gh pr merge #138 --merge --delete-branch` inline (L15). MERGED `7af873731`, L16 branch-deleted. L14 runlog SKIPPED (user-authorised) — blocked by canonical-checkout detached-HEAD + gov-v0-bound-to-ship-1-worktree topology.

## 2. Per-role signals

### Advisor (orchestrator)
- **Strong:** post-condition-verify discipline caught TWO false-successes (#328 bm-triage, #329 phase-tip-unchanged) — `feedback_bm_false_success_advisor_post_condition_catch` held both times. Pre-merge mergeability check caught the cross-lane drift BEFORE surfacing USER-GATE-5 with a doomed PR. cargo-check-before-e2e gate correctly ordered (8min early-failure detector vs 30min). Every conflict-resolution script assert-before-write → 4 wrong-boundary attempts, ZERO file corruption / ZERO bad commit.
- **Weak:** e2e.rs splice took 4 iterations because I trusted conflict-marker line arithmetic before deriving boundaries from index-stage blobs. The line-shift (phase `-a` probes) made marker math unreliable from the start.
- **bg-notification distrust:** held all phase — verified explicit `E2E_EXIT_0`/`CHECK_EXIT_0`/`CHAIN_DONE` markers every time, never trusted the bg-completion notification (lied 3× this phase: b1ezbrx1d, bq688ivry-area, baeezhm40-area).

### Planning
- §13 revert-list-extension gap surfaced as the fix-impl-5 root cause (down.sql not exact-inverse of up.sql). Plan §16a Story 1 Brief-Scope said "schema.rs" but Lemmy-1.0 actual path is the separate crate `crates/db_schema_file/src/schema.rs` — descriptor under-specified the crate path (non-blocking [descriptor-note] in verify report; planner should crate-qualify schema paths).

### Impl (Junior)
- fix-cr-1 #329 clean: all 7 findings in exactly the brief's enumerated file set (6 files), pre-push cargo-check ran, validate-pending-laptop DQ raised correctly. Worker flagged a "pre-existing lemmy_email build-script failure" that did NOT recur on the laptop — empirical verification (not the worker's word) was the right call.
- bm-triage #328 false-success: structural — wrote gitignored deliverables to a per-job worktree the daemon cleaned. Not a Junior-judgment failure; a runtime-artifact-lifecycle gap.

### BM (Junior)
- bm-pr #326 skipped writing `pr-138-findings.yaml` (low severity; bm-poll-cr creates it).
- bm-poll-cr #327 + bm-triage #328 deliverables are gitignored → die with cleaned per-job worktrees (HIGH structural class).

## 3. Actions for the next sub-phase

- **(L14 topology)** Before next phase: reattach canonical `brehon-fork` to `governance-v0` (resolve detached-HEAD `154cd608c` + the gov-v0-bound-to-`brehon-fork-ship-1`-worktree binding). The L14 post-merge runlog write needs canonical = gov-v0.
- **(planning)** Plans touching Lemmy-1.0 schema MUST crate-qualify paths (`crates/db_schema_file/src/schema.rs`, not bare "schema.rs"). Add to plan-authoring checklist.
- **(advisor)** New lesson candidate `feedback_3way_eof_splice_use_index_stages_not_markers`: when a 3-way conflict involves a file BOTH sides append to AND one side mid-edited (line-shift), derive block boundaries from `git show :2:`/`:3:` blobs + per-branch grep FIRST; do not trust conflict-marker line math.
- **(structural, HIGH)** gitignored bm-verb deliverables (`pr-N-findings.yaml`, `pr-N-comment.md`) die with cleaned per-job worktrees. Options: (a) bm verbs commit to a non-gitignored path on the phase branch, OR (b) bm verbs echo the full artifact into task output for advisor reconstruction (current workaround), OR (c) daemon retains per-job worktrees for bm-task verbs. Decide before next CR cycle.
- **(advisor)** `feedback_background_task_notification_lies` — promote to definite lesson (3× this phase; the marker-verification discipline is mandatory, not optional).

## 4. Per-task complexity scores

`<files>/<commits>/<runtime-min>/<max-log-silence-min>`

| Task | Score | Notes |
|---|---|---|
| Cohort A 1–5 | 5/5/~60/~3 | serial §5.2, clean |
| Cohort B 6–9 | 4/4/~50/~4 | serial; T9 e2e edit |
| fix-impl-5 | 1/1/~25/~2 | down.sql revert-symmetry |
| fix-cr-1 #329 | 6/1/~28/~5 | 7 findings, 1 commit, pre-push cargo-check |
| gov-v0 merge resolve | 2/3/~50/~1 | **4 e2e.rs splice iterations** (the cost driver); cargo-check 6m27s + e2e #4 33m |
| Phase-2 e2e (×4 runs) | n/a/n/a/~31avg/~4 | #267/#271/#272 all GREEN; #4 = 97/0/5 |

Aggregate: ~4h advisor wall-clock for the post-compaction tail (fix-cr-1 dispatch → merge), dominated by 3× full e2e runs (~93 min total) + the 4-iteration splice (~30 min).

## 5. Watch-items (promote if recurring)

1. **gov-v0-drift-during-lane-flight** (2nd occurrence pattern w/ multi-lane): a long-running lane WILL see trunk advance; mergeability-check-at-gate-5 is the catch. multi-lane-worktree.md class. → promote if 3rd.
2. **e2e.rs 3-way-EOF-splice 4-iteration** (1st): index-stage-not-marker lesson. → promote if recurs on next cross-lane merge.
3. **gitignored-bm-verb-artifact-loss** (HIGH, recurs every bm-poll-cr/bm-triage on cleaned per-job worktree): structural — needs the §3 fix, not just a watch.
4. **bm-false-success post-condition-verify** (held 2× this phase): discipline working; keep.
5. **bg-notification-lie** (3× this phase): → §3 promote-to-definite-lesson.
6. **canonical-checkout topology drift** (detached-HEAD + gov-v0-bound-elsewhere): 1st observed; → watch, may need a session-start canonical-checkout health assertion.

## 6. Sign-off

Awaiting user sign-off (USER-GATE-6).

- Phase shipped: ✅ PR #138 merged `7af873731`, federation foundation on governance-v0.
- e2e GREEN post-merge: ✅ 97/0/5.
- All gates honoured: ✅ USER-GATE-3 (triage), -5 (merge confirm), Option-A (fix-fwd + conflict-resolve), -6 (this).
- L14 runlog: ⚠ SKIPPED (user-authorised; topology-blocked; carried as §3 action).

## 7. Phase-transition gate compatibility (canonical 3-section view)

- **What shipped:** federation-inbound v1 foundation — `federation_peer` (trust state + 2 helpers), `federation_inbox_nonce` + `federation_inbox_dropped_log`, `remote_moderation_label`, Phase-6 model extensions (UpdateForm + columns), 11 governance_config keys, 9 ENTRY_KIND_FEDERATION consts, e2e phase1 round-trip `-a` probes + `v1_federation_inbound_a_fixtures` trust-state module. Migration `2026-05-17-000000-0000_add_federation_inbound_v1` (up/down exact-inverse).
- **Carry-forward (10 items):** see §3 + §5. HIGH: gitignored-bm-verb-artifact-loss; canonical-checkout-topology (blocks next L14). Lessons to author: `feedback_3way_eof_splice_use_index_stages_not_markers`, promote `feedback_background_task_notification_lies` to definite.
- **Next sub-phase prerequisite:** reattach canonical `brehon-fork` to `governance-v0` before any phase needing the L14 post-merge runlog path.
