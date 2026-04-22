# advisor-only (machine-format)

Private. Impl does not read.
Format matches 01-progress: pipe-delimited, terse.

## Risk register (set 2026-04-21)

r1 t4 NOT5-line-must-be-byte-perfect plan§10.5 CLEARED
r2 t2 DTO-add-previous_from breaks-2-construction-sites grep-before-commit CLEARED
r3 t3 3-write-tx UNIQUE-race+savepoint-rollback mirror=admin_emergency_remove.rs:74-218 CLEARED
r4 t5 scope-match community_id:None→Instance verify-create_report.rs-paths CLEARED
r5 t1 Scope::parse_wire-Result-change grep-workspace-for-stray-callers CLEARED
r6 t3 module-wiring-order admin_rule_sets@t3 case_open_snapshot@t5 CLEARED
r7 CR-2 silent-failure-pattern in-check_is_community_moderator.is_ok collapses-DB-errors-into-denial admin_rule_sets.rs:65-83+258-268 ACTIVE-in-phase-fix
r8 CR-4 Display-for-ScopeParseError::Malformed echoes-untrusted-input config.rs:172-178 ACTIVE-in-phase-fix

## CR predictions

cr1 t4 Critical-likely on payload-shape-evolution → defended by NOT5-line@plan§10.5
cr2 tX Major-likely on --all-targets-narrowing → defended by 2-commit-shape+v1-AD-b-precedent+upstream-CI-uses-all-features
cr3 t1-t8 2-3 mechanical fix-in-phase
cr4 t1-t8 1-2 rebuttal-bucket respond-no-change

## Log

2026-04-21T~ad | a | tX | meta | runlog-seeded
2026-04-21T~ad | a | t0 | probe | requires_re_jury=7 base-sha-ok branch-ok substrate-sound
2026-04-21T~ad | a | t0 | meta | clippy-ratchet-incident plan§15-added-all-targets-vs-v1-AD-b-Level6 self-critique=plan-author-missed-cross-check-with-prior-phase-ratchet
2026-04-21T~ad | a | t0 | decide:q1 | C-hybrid 2-commit-shape narrative@02-task0-narrative.md
2026-04-21T~ad | a | tX | meta | runlog-restructured-machine-format saves~3.8K-tokens-per-iteration narrative-archived@02-task0-narrative.md prior-density~78bytes/line new-density~100bytes/line+far-fewer-lines

## Retro candidates (append throughout phase, surface at phase-close)

retro1 plan§15-DoD-cross-check-vs-prior-phase-ratchets must-be-checklist-item-in-/prp-plan-template
retro2 runlog-density-matters-from-day-1 default-to-machine-format-not-narrative-format
retro3 advisor-commit-shape-instructions-must-test-before-state-on-disk advisor-originally-said-"commit1=narrowing-alone" but-plan-file-didn't-exist-pre-branch so-narrowing-as-diff-had-no-referent impl-staged-original-then-reverted-per-literal-reading corrected-to-commit1=plan-with-narrowing-applied+rationale-in-body commit2=audit-report-only
retro4 plan-drafts-§13-VALIDATE-lines-must-grep-memories-for-known-cargo-invocation-traps-before-commit feedback_features_full_workspace_only exists-2-days-but-planner-used-p-api_common-anyway task2-validate-red-caught-at-stage not-at-compile 3-lines-affected(t2+t5b+t7)+all-swappable-to-p-lemmy_api
retro5 cold-resume-from-runlog-worked impl-session-cleared-then-re-oriented-and-surfaced-the-t2-drift-correctly runlog-machine-format-sufficient-to-hand-off-mid-phase 22%-context→~10%-post-clear
retro6 plan-snippet-imports-drifted-4-paths-in-t3(api_common::context→api_utils::context, db_views::local_user→db_views_local_user, db_views::community_moderator→db_views_community_moderator, db_schema::utils→diesel_utils::connection) impl-corrected-silently-per-4-points-noted-but-planner-should-cargo-check-import-snippets-before-committing-plan memory-candidate-for-future-/prp-plan-checklist
retro7 8-arg-fn-hits-too_many_arguments-at-workspace-7-threshold CreateRuleSetTxArgs-struct-pattern-destructure-is-clean-solution worth-promoting-to-mirror-snippet-for-future-multi-write-txs
retro8 CI-cargo-test-e2e-runs-only-integration-tests-(cargo-test--test-e2e-p-lemmy_server) v1-AD-b-task9-payload_parity-unit-tests-have-never-executed-at-runtime api_crud-reqwest_middleware-compile-break-in-lemmy_api-dev-deps-blocks-any-lib-test-execution compile-time-drift-guard-via-clippy+cargo-check-is-sufficient-80%-value future-chore-issue-to-fix-upstream-api_crud-compile
retro9 windows-/tmp-redirect-silently-drops-output-from-cmd-//c-invocations advisor-hit-this-while-investigating-background-test memorised@feedback_windows_tmp_path_unreliable.md always-redirect-under-.claude/-or-%TEMP%
retro10 plan§1258-audit-trail-test-assumed-no-DB-row-default-but-migrations/2026-04-18-000000-0000_add_governance_config/up.sql:82-seeded-jury.panel_size-at-instance-value=5 impl-caught-via-local-runtime-test-run failing-pretty-publicly future-plan-templates-must-mandate-`rg-"jury\\.|report\\.|rule_set\\."-migrations/*/up.sql`-before-writing-assertion-assumptions-for-audited-config-keys
retro11 plan§1284-1286-three-way-contradictory-DoD-language advisory-(NOT-a-DoD-gate)-vs-block-the-commit-until-fixed-vs-file-tracking-issue-at-phase-close resolved-via-narrative-§171-172-commit-body-as-authoritative plan-templates-should-enforce-single-resolution-DoD-comments no-embedded-disagreeing-instructions
retro12 advisor-missed-out-of-phase-chore-commit-66749bb71-landing-between-t8-commit+pr-polling advisor-only-noticed-when-writing-handover-doc gh-pr-view-headRefOid-probe-should-be-part-of-every-phase-close-stage-to-detect-unexpected-upstream-activity
retro13 advisor-rubber-stamped-CR-round-2-finding-E-diff-without-reading-snapshot-shape applying-CR-diff-verbatim-would-have-made-test-FAIL snap_obj-holds-7-requires_re_jury-keys-not-rule_set_version_id column-level-pin-already-strict-at-e2e.rs:5779-5783 impl-correctly-flagged-advisor-instruction-mismatch pattern-fix=advisor-CR-suggested-diff-acceptances-must-be-marked-code-verified-vs-merits-only-from-comment
retro14 advisor-prescribed-Fix-D-option-D.i-(drive-duplicate-via-real-handler)-without-verifying-AdminCreateRuleSet-DTO-surface DTO-has-4-fields-no-version-field-handler-auto-increments-via-lookup_latest_version+1 client-cannot-force-version-collision-through-public-API only-path-satisfying-CR-5-round-1+round-2=option-4-helper-extract impl-correctly-flagged-advisor-reversed pattern-fix=advisor-test-shape-directives-for-handler-exercising-tests-must-verify-handler-input-DTO-surface-before-prescribing-call-shape

## Phase close — 2026-04-22

2026-04-22T16:19Z | a | phase-close | merge | PR #81 MERGED merge-commit=cf89890f3 target=governance-v0 style=merge-commit-not-squash preserves-task-per-commit-history-per-phase-branch.md
2026-04-22T16:22Z | a | phase-close | issues-filed | 4 chore issues: #82 (clippy debt) #83 (reqwest_middleware) #84 (admin-config-write.sh parity) #85 (harness CR 12-findings)
2026-04-22T16:22Z | a | phase-close | retros | retro13+retro14 appended; both are advisor-instruction-mismatch wins for Impl
2026-04-22T16:22Z | a | phase-close | ci | all 3 checks SUCCESS on final HEAD 6baabfd7a: governance-e2e + red-flag-diff + AI-review; zero escalations

**Final CR ledger for PR #81 (two rounds):**
- Round 1: 4 Major findings, 0 Critical → all shipped in `5c04e6ec2` (`fix(v1-AD-c): address 4 CR Major findings on PR #81`)
- Round 2: 5 Major findings, 0 Critical → 3 fixes (B/C/D) in `6baabfd7a`, 2 rebuttals (A/E) posted inline with evidence links
  - A: DTOs expand v0 surface → rebuttal (v1-AD-c is v1, not v0 baseline)
  - B: admin_list_rule_sets versions+active_version_id snapshot race → `conn.run_transaction` wrap
  - C: ScopeParseError doc drift → enum shape aligned + Display-suppressed-per-ADR-015 cross-ref
  - D: test recreates handler mapping → `pub fn map_rsv_unique_violation` helper extract
  - E: snapshot pin assert missing key → rebuttal (pin on column at e2e.rs:5779-5783, not JSON)

**Pattern wins carried into v1-AD-d+:**
- Advisor-instruction-mismatch catches still happening: 2 this phase (retro13+retro14). Rule working correctly — Impl pauses and surfaces on contradictions, advisor reverses cleanly.
- Four-bucket CR triage pattern held across both rounds: zero silent patches, zero silent ignores, zero carry-forward smuggled into PR scope.
- Pause-per-commit protocol caught the config.rs doc-comment misread risk in round-3 pre-stage review (advisor called refinement before stage instead of after).

**Memory candidates generated this phase:**
- None net-new — all patterns already in memory (`feedback_advisor_instruction_mismatch_stop_and_ask`, `feedback_coderabbit_triage_four_buckets_confirmed`, `feedback_severity_labels_dont_imply_semantic`, `feedback_branch_manager_pm_split`). Phase validated them in use.
- Update `project_v1_AD_c_mid_phase_state.md` → mark CLOSED or delete at next cleanup pass.

## Cold-resume handoff (2026-04-21)

Advisor session closing at task 3 reviewed, task 4 in-progress (code in working tree, not staged).

**Next advisor session resume checklist:**

1. Read CLAUDE.md + .claude/rules/*.md (auto)
2. Read this file (00-advisor-notes.md) — risk register + CR predictions + retros
3. Read 01-v1-AD-c-progress.md — format spec at top + last ~20 events for current state
4. Read 02-task0-narrative.md ONLY if Q1 (clippy DoD) or plan-drift questions resurface
5. Confirm git state: `git log --oneline -8` should show 6 commits on phase-v1-AD-c ahead of origin; working tree has `M crates/api/api/src/governance/admin_config.rs` (task 4 edits)

**Pending advisor review when Impl stages t4:**
- r1 NOT5 reply line byte-perfect in commit body (plan §10.5)
- payload_parity test compile-time only per retro8 — don't require runtime execution
- `build_admin_config_changed_payload` extraction signature matches plan §10.3
- `project_to_audit_entry` hydration from payload (not None) matches plan §10.3

**CR prediction cr1 remains active** — Critical-likely on t4 payload-shape evolution. Defence: NOT5 reply line in commit body + PR body.

**Tasks 5-8 remaining after t4:**
- t5 (pause) — case-open snapshot pin in create_report.rs; r4 active (community_id:None→Instance match)
- t6 (auto) — governance-log registry mechanical (already landed via t3, may be no-op at t6)
- t7 (auto) — routes wiring 2 lines
- t8 (pause) — 8 e2e tests; heaviest task; includes alltargets clippy delta capture

**Phase-close followup issues (file after PR merges, NOT during):**
- `chore(lint): clear e2e.rs + governance test-target clippy debt (135 errors at v1-AD-c base)` — referent @audit-clippy-baseline.log
- `chore: fix lemmy_api_crud reqwest_middleware transitive compile break in OAuth path` — referent @build-task4-tests.log (prevents lib unit test runtime execution)

**Branch-manager + PM split is working cleanly.** Machine-format runlog + cold-resume verified (Impl cleared once, re-oriented, surfaced Task 2 plan drift correctly). Pause-per-commit protocol adding ~2-3 min/task of advisor-wait, catching ~1 defect per 2 tasks that would otherwise surface at CR. Net positive.
