# Retro — v1-federation-inbound-b (HTTP-path enforcement: wrap + per-peer trust + per-actor rate + replay nonce + payload cap + replay-cleanup cron + handler-e2e)

**Phase branch:** `phase-v1-federation-inbound-b` (merged + deleted)
**PR:** #139 → `governance-v0`, merge commit `413ef5899`, merged 2026-05-20T13:45:40Z
**Phase wall-clock:** ~37 hours (cut from `7af873731` on 2026-05-19 ~00:34 UTC; merged 2026-05-20 13:45 UTC)
**Canonical ref (structure mirrored):** `.claude/PRPs/reports/v1-federation-inbound-a-retro.md`

## 0. Outcome

SHIPPED. 10 plan tasks (Task 0 pre-flight + Cohort A 1–3 [P] + serial Cohort B 4–9) + fix-impls 1–8 (5 §15-driven during impl phase + 1 Phase-2-e2e-driven + 1 CR-fix-in-pr-driven). Phase-2 e2e GREEN at round-2 on `4efce35c8` (**102 passed / 0 failed / 5 ignored**, 34m32s — +5 fed-in-b fixtures over fed-in-a's 97/0/5 baseline). All 19 PR-review findings triaged (3 CR fix-in-pr addressed in fix-impl-8 commit `9e78c886a`, 3 Copilot carry-forward to fed-in-c/DoS-hardening, 1 Copilot done-duplicate-of-cr-2, 12 CR + 1 CR-poll-2 wont-fix markdownlint on archived advisor artifacts). PR mergeable:CLEAN, CR statusCheckRollup `SUCCESS` on tip `512e42d1c`, all 9 fed-in-b validate-pending-laptop DQs (#285-#290) result:pass, pending=[]. Trunk L14 runlog COMPLETE entry landed first-try (BM Junior #351 ✓ — contrast with v1-ship-1-r2 Junior #322 L14 self-conflict). L16 branch-deleted clean. ~60 commits.

## 1. The arc (what actually happened)

1. **bm-cut + Task 0 pre-flight + Cohort A [P] 1–3 dispatch** (per `0a-2f-handover` from canonical brehon-fork lane on `governance-v0` per gate-1 — fed-in-b's first phase under the brehon-fork-fed-in-b dedicated lane worktree).
2. **Cohort B serial Tasks 4–9 + 6 fix-impls** during impl phase: each task had its §15 (cargo-check + clippy -D warnings + test --no-run) on the laptop via `validate-pending-laptop` DQ shape per Shape-G-SUSPENDED policy. Tasks 4/5/6 each needed a fix-impl (fix-impl-1 Task 4 §15 compile-failure Option A; fix-impl-2 §15 mechanical dead_code×6 + redundant-closure; fix-impl-3 Finding 6.1 domain hard-error Phase-6 sibling mirror; fix-impl-4 Task 5 §15 HRTB `<'a>` + dead_code; fix-impl-5 Task 6 §15 E0283 publish_trust_attestation drop spurious `.into()`; fix-impl-6 Task 6 §15 R2 cmd-2 clippy hoist CONFIG_KEY). Tasks 7/8/9 §15-GREEN first-try — validates §2.4 worker-side pre-push cargo-check discipline per `feedback_fix_impl_pre_push_cargo_check.md` (the fail-rate inflection from Task 4–6 to Task 7–9 is the proof).
3. **Phase-2 e2e round-1 FAIL** (DQ #290 raised by advisor on tip `95ee8274f`): 100 passed / **2 failed** / 5 ignored / 35m50s. Failures: (a) `sanction_notice_round_trip` @ e2e.rs:5254 (NotFound on wrong-DB lookup — Phase-6 fixture created `instance-a.test` only on url_a; Task 9 §10.6 insert queried url_b); (b) `per_peer_rate_limit_returns_429` @ e2e.rs:15618 (`governance_config` append-history collision — Task 9 §10.7 raw INSERT vs migration's seed row at different `valid_from` → reader's first-row-pick returned cap=100 instead of cap=2).
4. **RCA subagent classified both as TEST-FIXTURE defects** (not production-code bugs). User chose "Spawn RCA subagent → Queue fix-impl-7" option. fix-impl-7 brief authored canonical-sibling-mirror v1-federation-inbound-b-fix-impl-5; Junior #348 dispatched + clean §15 + finalize-push-skip recovered via ssh-push-from-daemon. Phase-2 e2e round-2 on `4efce35c8` PASSED CLEAN 102/0/5 in 34m32s. DQ #290 mutated to pass.
5. **bm-pr Junior #349** opened PR #139 (3min runtime) — runlog COMPLETE entry written by Junior on bm-runlog.md POST-merge per L14 REVISED (the 2026-05-18 fix landing as designed for the FIRST time on this phase). 2 BM process misses (non-blocking): phase-runlog `v1-federation-inbound-b-runlog.md` NOT updated (brief §4 said "BOTH"), findings.yaml shell NOT pre-written (gitignored anyway).
6. **CR + Copilot review on PR #139** (12:00:05 / 11:51:44 UTC): 15 CR + 4 Copilot = 19 findings. Triage 3 fix-in-pr / 3 carry-forward / 1 done-dup / 12 wont-fix. User-gate-3 APPROVED 2026-05-20.
7. **fix-impl-8** addressed cr-1+cr-2+cr-3 in 1 commit / 2 files / 3 hunks / ~9 lines net (clamp `replay_window_days >= 1` + bare-`?` seed loop + `assert_eq!(log_count, 1)`). Hunk-2 deviated from CR's literal `anyhow::anyhow!` recipe to bare-`?` per Case-A LemmyResult discipline (per `feedback_lemmy_error_no_std_error.md`) — the literal CR recipe would not compile. Junior #350 §15-GREEN first-try + finalize-push-skip recovered. CR re-reviewed tip `512e42d1c` at 13:12:54 UTC = SUCCESS / zero new actionable.
8. **bm-merge Junior #351** (1m40s): user-gate-5b APPROVED → `gh pr merge 139 --merge --delete-branch` → 413ef5899 merge SHA → trunk pulled clean → bm-runlog.md `chore(bm): merge PR #139 complete` POST-merge entry → L14 REVISED honored first-try → L16 branch-deleted clean.

## 2. Per-role signals

### Advisor (orchestrator)

- **Strong:** RCA-subagent path for Phase-2 e2e fail correctly classified both failures as TEST-FIXTURE (not production); fix-impl-7 brief landed canonical-sibling-mirror with verbatim §G4 blockquote + 2-hunk cap; round-2 e2e clean first try. CR triage drafted in one pass (no re-classification needed at gate-3). fix-impl-8 Hunk-2 deviation from CR's literal recipe was right (LemmyResult<()> Case-A bare-`?` over `anyhow::anyhow!`; `anyhow` not in dev-deps; would have failed E0277). bm-pr / bm-merge briefs canonical-sibling-mirror correctly (v1-AD-e for bm-pr; v1-ship-1-r2-bm-merge-2 for bm-merge L14 REVISED). post-condition-verify caught finalize-push-skip class 3× this phase (all recovered via ssh-push-from-daemon). DQ-mutate atomic protocol (fetch→read-fresh→mutate→verify→commit→push→re-verify) held all phase.
- **Weak:** **Stale-wakeup polling overhead** ≥4× this session segment (e.g. arriving with prompts referencing `tip 512e42d1c` to-be-validated when work already 2-3 stages past). Wakeup prompts encode decision state at scheduling time; by fire time the state has moved on. Cost: ~30s each verifying-then-deflecting. Per `feedback_thin_wakeup_prompts_verify_live_state.md` already; promote-now signal (≥4 occurrences in 1 session vs prior session's ≥3). **CR poll-2 mis-attribution moment:** the 2nd CR review at 12:48:30 UTC was on `edaff34612` (brief cherry-pick), not the fix-impl-8 code tip — I initially worried CR had skipped re-reviewing `512e42d1c` (the code tip) and dispatched a 30-min wait wakeup; the SUCCESS status check at 13:12:54 cleared it on the next poll. Cost: ~15 min wall-clock extra wait that would have been avoided with a more careful first read of `gh pr view --json statusCheckRollup` (which already had `state: SUCCESS`). Watch-item.
- **bg-notification distrust:** held all phase; verified `E2E_EXIT_0` markers explicitly. Promoted lesson `feedback_background_task_notification_lies` (from fed-in-a) reinforced — this is a permanent operational discipline.

### Planning

- Plan §13 nominal for Tasks 0-9 — only fix-impl-7 surfaced 2 planner-side oversights (§10.6 wording said "instance-a.test by domain (created by Phase 6 fixture)" without naming which DB; §10.7 INSERT pattern didn't flag the append-history collision against the migration's seed row). Both are runtime-only defects (not catchable at §15 cargo gates); both fixed with TEST-FIXTURE edits, NOT production-code edits. Plan §10 is the natural place to add `validate-pending-laptop-e2e` failure-mode hints for future test-fixture-class planning (carry-forward).
- §13 fail-rate inflection (Tasks 4/5/6 each needed fix-impl; Tasks 7/8/9 first-try §15-green) validates the §2.4 worker-side pre-push cargo-check rule's leading-indicator value. The planner-side ✓ here is: §2.4 prose was already in every impl-task brief — no plan revision needed. The workers internalised it across Cohort B.

### Impl (Junior)

- **All 9 §13 tasks + 8 fix-impls** delivered the §2.2 contract byte-identically. fix-impl-7 Hunk-2 disambiguation was solid (worker resolved brief-said-cap-1 vs reality-cap-11 `INSERT INTO governance_config` count from the codebase — 10 other occurrences have explicit `valid_from` for append-history (correct usage); only line 15607 lacked it (the defect). Worker reasoned from the codebase per the brief's hard rule; did NOT raise a blocker DQ. Per `feedback_impl_task_enumerated_transform_all_or_blocker.md` — Junior scrubs obvious subset + reports.
- fix-impl-8 Junior #350 honored the Hunk-2 contract verbatim (bare-`?` not the CR literal `anyhow::anyhow!`) — the brief's §2.2 reasoning was load-bearing; worker followed it correctly. Validates: when a brief's §2.2 contract DEVIATES from a CR-posted committable suggestion, the brief reasoning IS the contract (not the CR recipe).

### BM (Junior)

- bm-pr Junior #349: clean PR open, but missed 2 of 3 deliverables (phase-runlog absent; findings.yaml shell skipped — both gitignored or low-blast-radius, watch-items).
- bm-merge Junior #351: **first-try L14 REVISED success** — wrote `chore(bm): merge PR #139 complete` post-merge on bm-runlog.md without race or self-conflict. Contrast v1-ship-1-r2 Junior #322 (3× retry + protected-trunk `git push -f` + self-conflict + advisor-resolved). The L14 REVISED rule (`feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md`) + the bm-runlog.md `merge=union` `.gitattributes` (commit `fd96360c9`) both load-bearing here.
- **Daemon finalize-push-skip class** recurred 3× this session (Junior #348 fix-impl-7, #349 bm-pr, #350 fix-impl-8 — all recovered via `ssh homeserver 'git push origin <branch>'`). Junior #351 bm-merge pushed cleanly without skip (its push is `gh pr merge`-driven, not daemon-finalize-driven). Promote-now signal — see §3.

## 3. Actions for the next sub-phase

- **(advisor, PROMOTE-NOW)** **Daemon finalize-push-skip lesson**: write `feedback_daemon_finalize_push_skip_load_bearing_recovery.md` capturing the recurrence pattern (3+ occurrences fed-in-b alone; ≥4 across fed-in-a + fed-in-b) + the canonical `ssh homeserver 'cd /srv/brehon-fork && git push origin <branch>'` recovery + a request upstream to `barrie-cork/lemmy` issue for the daemon finalize stage to always push origin even when worker pre-pushed. Already a watch-item per `feedback_junior_finalize_skips_when_worker_pre_pushes.md`; this retro promotes to definite lesson + fix request.
- **(advisor, PROMOTE-NOW)** **Stale-wakeup polling overhead**: `feedback_thin_wakeup_prompts_verify_live_state.md` ≥4× this session segment. Promote from watch to definite with the structural fix: wakeup prompts should be ≤2 lines (wake trigger + resume-stage hint) — decision logic lives in this file + advisor-orchestrator.md, read at fire time. Embedded decision trees in wakeup prompts go stale within minutes. Already drafted; needs commit.
- **(planning carry-forward)** Latent reader-side append-history defect in `get_inbound_config_int` (`crates/apub/activities/src/governance/inbox.rs:421-438`) AND its mirror in `crates/apub/activities/src/governance/publish_trust_attestation.rs:142-152` — both lack `.order_by(governance_config::valid_from.desc())`. Read-side bug class on the entire `governance_config` reader path. **File as fed-in-c or storage-hardening follow-on sub-phase**. Symptom: any test or admin-edit-config flow that adds a 2nd row for an existing `(scope, key)` sees arbitrary value pick. Migration 2026-04-18 explicitly warned "Do NOT use (scope, key) as the conflict target" but readers don't honour the append-history semantic.
- **(planning carry-forward)** Copilot DoS-hardening family (copilot-1/2/3 from PR #139): unbounded in-memory rate-limit maps + raw-string HashMap keys + TOCTOU eviction in `inbox.rs:473/698` + `publish_trust_attestation.rs:165`. Design work for fed-in-c OR dedicated DoS-hardening follow-on. **File as one GH issue at fed-in-b close.**
- **(planning carry-forward)** **Phase-6 convention-divergence class** (standing user directive a, 2026-05-19): fed-in-b Task 4 new code in `inbox.rs` diverges from same-file Phase-6 conventions (helper organization, naming, error-type discipline within the same module). Thorough product-grade interpretation deferred to retro. **Carry-forward: do a Phase-6-convention-conformance audit pass over fed-in-b's `inbox.rs` additions before fed-in-c starts.** This was deferred-NOT-fixed during impl per user directive; the retro records it as actionable now.
- **(advisor / audit-tooling, carry-forward)** **Audit-tooling generalization** (standing user directive b, 2026-05-19): generalize the conformance audit pattern into either (a) a project-scope subagent at `.claude/agents/security-auditor.md` (or `brehon-conformance-auditor.md`) OR (b) a new Brehon audit skill (`/brehon-conformance-audit`). Already in flight per the concurrent advisor session's `chore(advisor): author brehon-conformance-audit planning brief — supersedes 2026-05-20 guidance + integrates Rust best-practices research` (commit `57ce4c322`). Carry-forward: advisor sessions converge on whether the new skill or subagent shape lands first.
- **(BM)** bm-pr brief §4 carries deliverable list ("BOTH phase-runlog AND bm-runlog"); Junior #349 honored bm-runlog but not phase-runlog. Either tighten the brief contract (a §0 grep self-verify confirming phase-runlog write at task close — like the impl-task §0 anchors) OR accept the gitignored-state risk and remove the phase-runlog item from the brief. Watch.
- **(advisor / process, carry-forward)** **CR poll-2 mis-attribution moment**: when CR posts a 2nd review during a cherry-pick window, the review's `commit_id` field shows the cherry-pick tip, NOT the next post-fix-impl tip. The reliable signal is `gh pr view --json statusCheckRollup` (CodeRabbit state SUCCESS / PENDING / FAILURE) — NOT the review's commit_id alone. Add a one-line note to advisor-orchestrator.md §3.1 or `bm-poll-cr.md`.

## 4. Per-task complexity scores

`<files>/<commits>/<runtime-min>/<max-log-silence-min>`

| Task | Score | Notes |
|---|---|---|
| Task 0 pre-flight | 0/0/~5/~1 | harness audit; trivial |
| Cohort A [P] 1–3 | 3/3/~80/~5 | parallel impl-tasks; clean |
| Cohort B 4 (impl + fix-impl-1+2+3) | 6/4/~120/~10 | mid-stream — typestate cascade + dead_code mechanical sweeps; Option A user-authorised |
| Cohort B 5 (impl + fix-impl-4) | 3/2/~75/~8 | wrap_governance_inbound HRTB cascade |
| Cohort B 6 (impl + fix-impl-5+6) | 4/3/~95/~6 | publish_trust_attestation .into() cascade + CONFIG_KEY hoist |
| Cohort B 7 (impl) | 2/1/~45/~3 | publish_label — §15 first-try GREEN |
| Cohort B 8 (impl) | 1/1/~35/~2 | scheduled_tasks replay-cleanup cron — §15 first-try GREEN |
| Cohort B 9 (impl + fix-impl-7) | 2/2/~75/~5 | e2e.rs Phase-6 fixture + handler-e2e (5 tests) — §15 first-try GREEN; Phase-2 e2e fail → fix-impl-7 2-hunk fix |
| Phase-2 e2e (2 rounds) | n/a/n/a/~35avg/~3 | round-1 FAIL 100/2; round-2 PASS 102/0/5 |
| fix-impl-8 (CR fix-in-pr) | 2/1/~12/~2 | 3 hunks (clamp + bare-? + assert_eq); §15-GREEN first-try |
| bm-pr / bm-merge | 0/2/~5 total/~1 | first-try L14 REVISED clean |

Aggregate: ~10h advisor wall-clock for the impl phase (Cohort A → Phase-2 e2e round-2 PASS) + ~2h wall-clock for the CR-fix-in-pr + merge segment. Dominated by Phase-2 e2e (~70min for 2 rounds combined) + Cohort B 4/5/6 fix-impl cascades (~90min).

## 5. Watch-items (promote if recurring)

1. **Daemon finalize-push-skip** (3rd-this-phase, ~4-5th-across-recent-phases): **PROMOTE-NOW** per §3 action.
2. **Stale-wakeup polling overhead** (4th-this-session): **PROMOTE-NOW** per §3 action.
3. **CR poll-2 mis-attribution** (1st observed; cherry-pick triggered transient review state): watch — may need a one-line operating note in `bm-poll-cr.md`.
4. **gov-v0-drift-during-lane-flight** (3rd occurrence pattern this multi-lane session — 2 concurrent advisor sessions on canonical writing briefs/retros while fed-in-b lane in flight): the atomic-DQ-write protocol + lane-isolated DQ JSON + bm-runlog.md `merge=union` all worked. Multi-lane discipline + `feedback_lane_dq_resolution_append_to_trunk.md` holding. → consider promote to "expected state" rather than "watch".
5. **BM Junior process misses (bm-pr Junior #349)**: phase-runlog absent + findings.yaml shell skipped. 2 non-blocking misses; first observed this phase. → §3 advisor-side action on the bm-pr brief contract.
6. **Phase-6 convention-divergence class** (standing user directive a): NOT itself a recurrence pattern — user directive ratified; deferred-NOT-fixed by design. The carry-forward IS the action. → §3.
7. **Reader-side append-history defect class** (1st observed; affects `governance_config` reader path): high-impact carry-forward. → §3.
8. **bm-merge first-try L14 REVISED success**: 1st observed under the REVISED ordering on this lane. The 2026-05-18 lesson held. → keep ordering; carry forward as confirmed-success precedent.
9. **§15 fail-rate inflection (Tasks 4/5/6 → Tasks 7/8/9)**: 1st observed; suggests the §2.4 pre-push cargo-check discipline + the fix-impl recipe-mirror discipline both bedded in over Cohort B. → §2 planning-side note.

## 6. Sign-off

Awaiting user sign-off (USER-GATE-6).

- Phase shipped: ✅ PR #139 merged `413ef5899`, federation-inbound HTTP-path enforcement on governance-v0.
- e2e GREEN on tip: ✅ round-2 102/0/5 on `4efce35c8` (then fix-impl-8 added 3 hunks; the 3 hunks are quality-bar improvements without semantic change to the enforcement path — no re-run of Phase-2 e2e needed per fix-impl-8 brief §5).
- All gates honoured: ✅ gate-1 plan approval (2026-05-19 proceed-as-one); gate-3 CR triage (2026-05-20 APPROVE); gate-5 merge confirm (initial WAIT-for-CR-re-review + final APPROVE on tip 512e42d1c); gate-5b CR-re-review confirm (CR=SUCCESS on tip); gate-6 (this).
- L14 REVISED: ✅ honored first-try by BM Junior #351; runlog COMPLETE entry on trunk in commit `4480a1bdb`.
- L16 branch-delete: ✅ verified empty `ls-remote origin refs/heads/phase-v1-federation-inbound-b`.

## 7. Phase-transition gate compatibility (canonical 3-section view)

- **What shipped:** federation-inbound v1 HTTP-path enforcement — `GovernanceInboundActivity` trait + `wrap_governance_inbound` shared enforcement (peer trust allowlisting + payload size caps + replay detection + rate limiting) + per-actor rate override (publish_trust_attestation) + `receive_remote_moderation_label` advisory handler + replay-cleanup cron tick wiring + 6 `LemmyErrorType` variants for federation errors + `deny_unknown_fields` on 3 protocol DTOs + 5 e2e fixture tests (`v1_federation_inbound_b_fixtures`: allowlisted_happy_path + blocklisted_peer_returns_403 + per_peer_rate_limit_returns_429 + replayed_activity_returns_409 + moderation_label_handler_persists_and_logs) + Phase-6 fixture Allowlist extension in `sanction_notice_round_trip` + best-effort `federation_inbound_persist_failed` log + clamp on `replay_window_days` (admin-misconfig guard from CR fix-in-pr) + governance-log entry-kind constants + governance-log-entry-kind-registry doc update (54→55).
- **Carry-forward (8 items):** see §3 + §5. HIGH: reader-side append-history defect class (`get_inbound_config_int`) + Copilot DoS-hardening family (rate-map LRU + key hash + TOCTOU eviction) + Phase-6 convention-divergence audit (user directive a) + audit-tooling generalization (user directive b). PROMOTE-NOW: daemon finalize-push-skip lesson + stale-wakeup polling overhead lesson. Lower-blast: bm-pr deliverable contract tightening + CR poll-2 mis-attribution operating note.
- **Next sub-phase prerequisite:** none blocking — `governance-v0` is clean, `413ef5899` merge committed, L14 runlog on trunk, L16 phase branch deleted. Next sub-phase planner reads this retro's §3 + §5 + §7 carry-forwards. The concurrent advisor session's `brehon-conformance-audit` planning brief (commit `57ce4c322`) is in-flight independently — that thread + the Phase-6-convention-divergence audit converge once both planning passes complete.
