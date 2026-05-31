# Retro harvest — 2026-05-31

**Scope:** 50 session-retros + 17 phase-retros | window last-21d (2026-05-10→2026-05-31)
**Branch / HEAD:** `governance-v0` @ `0f0c9a3fd`
**Delegation:** Explore subagents (Phase 1 ×1, Phase 2 ×4 batches)
**Date oracle:** git-author-date override — all 17 phase-retros verified against `git log -1 --format=%ad`; no clone-artifact mtime clash found (dates span 2026-05-21 to 2026-05-31)
**Extracted:** ~112 open proposals, ~46 already-closed, across 67 retros

---

## TL;DR

7 Tier-1 LIVE items ready to act on now — highest-leverage is the **cross-session `git add -A` defence hook** (2× confirmed incidents, PreToolUse hook explicitly "not yet implemented" in the lesson, costs ~20 lines of bash). 23 items are cleanly STALE/SUPERSEDED — just box-ticking. ~27 Tier-2 singles worth scheduling. The 50 session retros from the last 3 weeks show healthy signal: most "1× watch" proposals stayed single-occurrence (correctly deferred), and the multi-occurrence promotions are concentrated in 4 tight clusters.

---

## Tier 1 — act now (LIVE, recurrence ≥2 or high impact, cost minor/medium)

> **ALL 7 SHIPPED** — commit `675db1963` on `governance-v0`, 2026-05-31. See "Suggested next action" column for what was done.

| # | Proposal | Source retro(s) | Recurrence | Cost | Evidence it's still LIVE | Suggested next action | Done |
|---|---|---|---|---|---|---|---|
| 1 | Cross-session `git add -A` defence: PreToolUse hook on `git commit` compares staged files vs session-scoped intent | session-retro-2026-05-22-v1-dq-schema-r1-cohort-2-ship + session-retro-2026-05-23-rules-trim-and-verb-retire | 3× (2× incidents + explicit "not yet implemented" in lesson) | medium | `feedback_cross_session_commit_attribution_collision.md` §3 mitigation 3: "Not yet implemented" | Write `~20-line` PreToolUse bash hook; wire in `settings.local.json` on canonical checkout | ✅ |
| 2 | Uniqueness-check (`grep -c`) at brief-author time for e2e Edit anchors (verify old_string is unique before queuing) | session-retro-2026-05-30-v1-quality-r3-bootstrap + session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail | 2× | minor | No grep for "grep -c" or "uniqueness-check" in advisor-orchestrator.md §2.4 | Add 1-line pre-flight gate to advisor-orchestrator.md §2.4 mandatory file-class table row for `e2e.rs` | ✅ |
| 3 | Update `feedback_brehon_subagent_model_effort_assignments.md` lesson with current model versions (4.8/4.6/4.5) | session-retro-2026-05-29-quality-r2-resume-opus-bump | 1× but high impact (wrong model versions actively mis-inform briefs) | minor | Lesson file absent from `.claude/lessons/` (MEMORY.md index points to a missing file) | Create/restore `.claude/lessons/feedback_brehon_subagent_model_effort_assignments.md` with Opus 4.8 / Sonnet 4.6 / Haiku 4.5 | ✅ |
| 4 | Telegram completion hook check at session start — advisor should verify hook ID 1 exists and recreate if missing | session-retro-2026-05-30-v1-quality-r3-bootstrap + MEMORY.md daemon hook entry | 2× pain (hook silently absent after daemon restart) | minor | `list_hooks` check absent from advisor-orchestrator.md §1 polling-loop session-start ritual | Add one-liner to §1: "check `list_hooks` — recreate hook ID 1 if missing" | ✅ |
| 5 | Mode B trunk→phase sync standalone lesson (the SSH merge path for brief visibility in mobile-remote mode) | session-retro-2026-05-30-v1-quality-r3-bootstrap + multi-lane-worktree.md §"Brief location" (procedure is there but un-promoted as a lesson) | 2× (RT-r3 + quality-r3) | minor | `Glob .claude/lessons/feedback_mode_b_trunk_phase_sync*.md` → no match | Write `.claude/lessons/feedback_mode_b_trunk_phase_sync.md` pointing at multi-lane-worktree.md procedure | ✅ |
| 6 | Update harness-audit SKILL.md to use `~3.3` chars/token ratio (not `2.4`) for budget estimation | session-retro-2026-05-29-harness-budget-floor-validated + session-retro-2026-05-29-harness-context-budget-trim | 2× | minor | harness-audit SKILL.md Phase 3 references scoring-matrix.md; 3.3 ratio absent | Edit harness-audit SKILL.md Phase 3 budget formula; update `feedback_context_trim_verify_empirically.md` | ✅ |
| 7 | `code-audit` SKILL.md Tier 1: add Rust/clippy row + `const blocks` longest-fn caveat + deferral-note guidance | session-retro-2026-05-30-code-audit-governance-deferred | 1× but certain to recur on every Rust audit; no Rust row in Tier 1 language table | medium | code-audit SKILL.md Tier 1 table has no Rust/clippy row; confirmed by subagent | Edit `.claude/skills/code-audit/SKILL.md` Tier 1 table: add Rust/clippy row + const-block caveat + deferral section | ✅ |

> **Tier 2 batch-1 (7 items) — PLANNED, not yet executed.** Plan at `C:\Users\barri\.claude\plans\create-a-plan-to-cheeky-flute.md`. Execute in next session. Items: §1 surface-first CWD, §3.6 source-code extension, advisor-validation falsification subsection, outcome≠cause lesson + §5.5 bullet, CR falsifiable-hypothesis cross-ref, BM bot-author race note, weekly-review disk check.

---

## Tier 2 — schedule (LIVE, single-occurrence or high-cost)

| # | Proposal | Source retro | Cost | Evidence it's still LIVE | Note |
|---|---|---|---|---|---|
| 1 | cleanup-junior-logs.sh helper + weekly cron on homeserver | session-retro-2026-05-25-homeserver-disk-cleanup | medium | `cleanup-junior-logs.sh` absent from homeserver/scripts/ | Disk pressure is continuous; recurrence guaranteed |
| 2 | Disk-headroom check in weekly-review (75% threshold) | session-retro-2026-05-25-homeserver-disk-cleanup | minor | weekly-review SKILL.md has no disk/headroom check | Pairs with #1 above |
| 3 | systemd journal size cap (`SystemMaxUse=500M`) | session-retro-2026-05-25-homeserver-disk-cleanup | minor | No `SystemMaxUse` in homeserver systemd configs | One-line in journald.conf |
| 4 | Outcome ≠ cause verification step — check `retro-bypass.jsonl` FIRST before diagnosing orchestration failures (2× threshold) | session-retro-2026-05-24-task4-followups-ship3-deps-r1 | minor | Lesson not in `.claude/lessons/`; marked "owner: next session" in retro | Write lesson + add to advisor-orchestrator.md §5 |
| 5 | Slash-command scope-vs-locality hard-refuse hook (2× threshold met) | session-retro-2026-05-24-task4-followups-ship3-deps-r1 | medium | No hook or rule artifact found in `.claude/hooks/` or `.claude/rules/` | 4× underlying incidents per retro |
| 6 | Falsification recipe in validate-pending-laptop handler | session-retro-2026-05-25-t1-validate-gate-cleared | minor | advisor-orchestrator.md §5.2 has no falsification step | High-leverage even at 1× given DQ #338 precedent |
| 7 | Brief overspec prevents reconnaissance-hang on >5k-line files | session-retro-2026-05-26-rt-r3-task4-max-turns-audit-harvest | minor | No "overspec" or "5k-line" guidance in advisor-orchestrator.md §2 | Watch for 2nd recurrence; document scoping heuristic |
| 8 | `cargo-test.bat` exit-code propagation audit — confirm `cmd //c` layer propagates correctly (2× threshold) | session-retro-2026-05-25-t1-validate-gate-cleared | minor | Batch agent said STALE (exit /b !errorlevel! found) but noted inconsistent layers — manual verify recommended | Low-confidence STALE from batch 2; treat as LIVE pending explicit test |
| 9 | Session-start inherited-context-anchors lesson (2× across sessions) | session-retro-2026-05-22-dq338-rca-revision-option-b-ship | minor | Batch 4 called STALE citing a lesson file; lesson filename not confirmed in glob — listed here as **unverified STALE** (see §Unverified below) | Spot-check `.claude/lessons/feedback_session_start_inherited_context_anchors_action.md` exists |
| 10 | Handover-assumption verification gate in MEMORY.md is a lesson, but wasn't explicitly confirmed with a Glob check | session-retro-2026-05-24-t4a-check-role-health | minor | MEMORY.md references `feedback_handover_assumptions_need_empirical_verification.md` | Verify file exists; check if content covers "scope vs locality" per the retro's proposal |
| 11 | Read source code before speccing (fold into existing `feedback_read_canonical_before_writing_spec.md`) | session-retro-2026-05-31-pmd-validation-pheromone-spec | minor | No dedicated lesson; advisor-orchestrator.md §3.6 is close but scoped to `.claude/` files only | Extend §3.6 to cover code source files |
| 12 | Canonical-checkout multi-lane surface in status line (surface CWD in advisor session opening) | session-retro-2026-05-23-adr-016-cross-app-backplane (2× threshold) | minor | advisor-orchestrator.md §1 has SessionStart WARN; no status-line one-liner in "Surface-first ritual" | Add one-liner to §1 surface-first ritual prose |
| 13 | Workspace lints check before installing third-party skill (prevent lint regressions) | session-retro-2026-05-31-type-state-lesson-apollo-assessment | minor | No rule or skill-install checklist has this gate | Useful given brehon uses `-D warnings` |
| 14 | Post-finalize check: advisor §3.1 step to verify daemon-local and origin refs match after finalize-merge | session-retro-2026-05-31-pmd-validation-pheromone-spec | minor | Batch 1 called STALE but verifying daemon-local-first is in §3.1 (line 111); not a full "check after" step | Spot-check — likely STALE |
| 15 | `Step 5b` MCP-disconnected fallback in `post-task-retro` SKILL.md | session-retro-2026-05-24-role-customization-t1a | minor | post-task-retro SKILL.md steps verified; no Step 5b | Low-priority; relevant when MCP is flaky |
| 16 | Extend `validate-pending-laptop` DQ schema with `env_setup`/`harness_required` fields | session-retro-2026-05-25-t1-validate-gate-cleared | medium | decision-queue.md schema has no these fields | Requires DQ schema update + migration script |
| 17 | Stop hook early-exit refactor for non-`-p` mode | session-retro-2026-05-24-t4a-check-role-health | medium | No refactored hook found | Reduces noise in interactive sessions |
| 18 | Filter background-proc kills by known PID not pattern (prevents accidental kills) | session-retro-2026-05-24-role-customization-t1a | minor | No PID-filtering lesson; 1× + 1× prior | Write `.claude/lessons/` entry |
| 19 | Heading-anchor extraction pattern lesson (applied 6× in one session) | session-retro-2026-05-23-rules-trim-and-verb-retire | minor | No heading-anchor lesson in `.claude/lessons/` | 1-session × 6 applications crosses informal threshold |
| 20 | Pre-delete citation grep in `command-retro` SKILL.md before removing a verb | session-retro-2026-05-23-rules-trim-and-verb-retire | minor | command-retro SKILL.md Step 0a exists for task retros; verb-removal case not covered | Add to SKILL.md §verb-retirement |
| 21 | Lib-test env recipe lesson (LEMMY_DATABASE_URL + LEMMY_CONFIG_LOCATION setup) | session-retro-2026-05-25-t1-validate-gate-cleared | minor | No single "lib-test env recipe" lesson | Useful for any test that bypasses testcontainers |
| 22 | Model-value drift check in restore script: PATCH-MARKER.md auto-update | session-retro-2026-05-29-quality-r2-resume-opus-bump | minor | Script checks drift but does NOT auto-update PATCH-MARKER.md | Narrows the manual step after each model upgrade |
| 23 | Scoring-matrix redundancy term calibration for `harness-audit` | session-retro-2026-05-29-harness-context-budget-trim | minor | No artifact found; 1× occurrence | Hold until 2nd audit hits this |
| 24 | Empirical token measurement guidance: prefer harness-audit over manual char-counting | session-retro-2026-05-23-rules-trim-and-verb-retire | minor | No dedicated rule; 1× occurrence | Short rule addition to harness-audit SKILL.md |
| 25 | `CR-finding falsifiable-hypothesis` variant (apply the DQ #338 pattern to CR findings too) | session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail | minor | Only general falsifiable-hypothesis gate in §5.4; no CR-specific variant | Borderline 2× (DQ #338 + CR #479 cycle) |
| 26 | Duplicate-key JSON validation recipe | session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail | minor | No lesson/rule found | Low-cost python one-liner; document |
| 27 | Fetch-before-push on trunk: extend to bot-author races (not just human-session races) | session-retro-2026-05-23-adr-016-cross-app-backplane | minor | No fetch-before-push bot-author extension in branch-manager.md | 1-line addition |

---

## Tier 3 — closeable (STALE / SUPERSEDED — just check the box)

| Proposal | Source retro | Verdict | Evidence | Action |
|---|---|---|---|---|
| Falsifiable-hypothesis gate to advisor-orchestrator §5.4 | session-retro-2026-05-22-dq338-rca-revision-option-b-ship | STALE | advisor-orchestrator.md:219 full gate present, cross-refs lesson | Tick `[x]` |
| Pre-locate verbatim anchors in e2e.rs fix-impl briefs | session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail | STALE | `feedback_fix_impl_pre_locate_e2e_anchors.md` cited at advisor-orchestrator.md:70 | Tick `[x]` |
| Cap fix-impl brief length + edit count on e2e.rs | session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail | STALE | advisor-orchestrator.md:70 scope gate: ≤150 lines, ≤2 edits/file | Tick `[x]` |
| Surface failure-class risk at user-gate 3 | session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail | STALE | advisor-orchestrator.md:123 — "failure-class signature and observed retry rate" present | Tick `[x]` |
| Cohort-size cap of 2 on shared-.git daemon | session-retro-2026-05-25-rt-r3-cohort-2-git-index-cascade | STALE | `.claude/refs/auto-phase.md`:529 — cohort ≥3 → degrade to serial | Tick `[x]` |
| Pre-cancel SSH inventory step to catch-fire procedures | session-retro-2026-05-25-rt-r3-cohort-2-git-index-cascade | STALE | advisor-orchestrator.md §5.6 catch-fire table: SSH tar command present | Tick `[x]` |
| Junior cancel-handler preserve worktrees | session-retro-2026-05-25-rt-r3-cohort-2-git-index-cascade | STALE | advisor-orchestrator.md §5.6: tar preservation before cancel documented | Tick `[x]` |
| Shared-.git/index.lock lesson promotion | session-retro-2026-05-25-rt-r3-cohort-2-git-index-cascade | STALE | `feedback_cohort_shared_git_index_contention.md` exists in lessons/ | Tick `[x]` |
| cargo-test.bat exit-code propagation (2× threshold) | session-retro-2026-05-25-t1-validate-gate-cleared | STALE | cargo-test.bat uses `exit /b !errorlevel!` at lines 110/115/120 | Tick `[x]` |
| Falsification recipe in validate-pending-laptop handler | session-retro-2026-05-25-t1-validate-gate-cleared | STALE | advisor-orchestrator.md §5.4 + §5.2 combined cover this | Tick `[x]` |
| DQ -027 empirical verification pre-Task-4-requeue | session-retro-2026-05-26-rt-r3-task4-max-turns-audit-harvest | STALE | DQ #027 resolved 2026-05-26 in decision-queue.json | Tick `[x]` |
| Handover-assumption verification gate (2× threshold) | session-retro-2026-05-24-t4a-check-role-health | STALE | MEMORY.md: `feedback_handover_assumptions_need_empirical_verification.md` listed | Tick `[x]` (after spot-checking file exists) |
| Sibling-pattern gitignore audit (2× threshold) | session-retro-2026-05-24-t4a-check-role-health | STALE | MEMORY.md line 5: "promoted 2026-05-25 → `feedback_sibling_gitignore_at_feature_author.md`" | Tick `[x]` |
| PowerShell Remove-Item -Force VS Code watcher fallback | session-retro-2026-05-24-task4-followups-ship3-deps-r1 | STALE | Pattern tracked in project memory (17 PMD memory matches) | Tick `[x]` |
| Hooks directory audit via harness-audit | session-retro-2026-05-22-dq338-rca-revision-option-b-ship | STALE | harness-audit SKILL.md Phase 1 explicitly inventories `.claude/hooks/` | Tick `[x]` |
| Audit cdff6392e cross-session attribution | session-retro-2026-05-22-carry-forward-shipped + context-prune-option-a | STALE | `git show cdff6392e` — `docs(rules)` commit 2026-05-22; attribution present | Tick `[x]` in both source retros |
| Extend cross-session collision lesson to working-tree-mutation | session-retro-2026-05-22-parallel-subagent-dispatch | SUPERSEDED | `feedback_cross_session_commit_attribution_collision.md` covers Race-A + Race-B generically | Collapse into Race-B note; tick `[x]` |
| Finalize-merge lookup-order lesson | session-retro-2026-05-31-pmd-validation-pheromone-spec | STALE | `feedback_finalize_merge_where_to_look_first.md` exists; cited in advisor-orchestrator.md:111 | Tick `[x]` |
| Post-finalize check to advisor-orchestrator §3.1 | session-retro-2026-05-31-pmd-validation-pheromone-spec | STALE | advisor-orchestrator.md §3.1 line 112 documents daemon-local-first lookup | Tick `[x]` |
| Sync `feedback_governance_type_state_handlers.md` to PMD | session-retro-2026-05-31-type-state-lesson-apollo-assessment | STALE | Lesson exists at `.claude/lessons/feedback_governance_type_state_handlers.md` (2026-05-31) | Tick `[x]` |
| Add `feedback_governance_type_state_handlers.md` to MEMORY.md | session-retro-2026-05-31-type-state-lesson-apollo-assessment | STALE | MEMORY.md line 97: entry present | Tick `[x]` |
| Model-value drift check in restore script | session-retro-2026-05-29-quality-r2-resume-opus-bump | STALE | restore-junior-server-patches.sh has `check_marker()` drift-detection (lines 55-73) | Tick `[x]` |
| `feedback_context_trim_verify_empirically.md` chars/token correction | session-retro-2026-05-29-harness-budget-floor-validated | STALE | Lesson exists with chars/ratio documentation | Tick `[x]` |

---

## Unverified STALE claims (downgraded to LIVE — needs a human look)

| Proposal | Source retro | Claimed STALE by | Why unverified | Recommended check |
|---|---|---|---|---|
| Session-start inherited-context-anchors lesson (2× threshold) | session-retro-2026-05-22-dq338-rca-revision-option-b-ship | Batch 4 subagent | Cited `feedback_session_start_inherited_context_anchors_action.md` but filename is non-standard; no glob confirmation | `ls .claude/lessons/feedback_session_start*` |
| Default CR ingest to `gh api` (vs `gh pr view`) | session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail | Batch 2 subagent called it LIVE with caveat | bm-poll-cr.md uses `gh api` already but no "default" statement | Read bm-poll-cr.md top of file for explicit default |
| cargo-test.bat exit-code propagation (2× threshold) | session-retro-2026-05-25-t1-validate-gate-cleared | Batch 2 subagent | Found `exit /b !errorlevel!` but noted possible inconsistency across layers | Run: `grep -n "exit\|errorlevel\|cmd" scripts/brehon/cargo-test.bat` |

---

## Suggested action sequence

The skill does NOT execute these — user decides and acts.

**Immediate (≤30 min total, Tier 1 items):**

1. **Tier 1 #1** — Write PreToolUse bash hook for `git commit` that checks `git diff --cached --name-only` against a session-intent file. Wire in `.claude/settings.local.json` on canonical checkout. Source: `feedback_cross_session_commit_attribution_collision.md` §3 mitigation 3 for the exact spec.
2. **Tier 1 #3** — Restore `.claude/lessons/feedback_brehon_subagent_model_effort_assignments.md` with correct model IDs: BM=Haiku 4.5, Impl=Sonnet 4.6, Planning=Opus 4.8. One file, ~15 lines.
3. **Tier 1 #4** — Add one line to advisor-orchestrator.md §1 polling-loop ritual: "check `list_hooks`; if hook ID 1 absent, recreate via `mcp__junior-brehon__create_hook`."
4. **Tier 1 #6** — Edit harness-audit SKILL.md Phase 3: replace `2.4` with `3.3` in the chars/token budget formula. Then update `feedback_context_trim_verify_empirically.md` accordingly.
5. **Tier 1 #2** — Add one line to advisor-orchestrator.md §2.4 `crates/server/tests/e2e.rs` mandatory-lesson row: "before queuing, verify `old_string` uniqueness with `grep -c '<anchor>' crates/server/tests/e2e.rs` = 1."
6. **Tier 1 #5** — Write `.claude/lessons/feedback_mode_b_trunk_phase_sync.md` (3-5 sentences pointing at multi-lane-worktree.md §"Mode B" procedure). Then add to MEMORY.md index.
7. **Tier 1 #7** — Edit `.claude/skills/code-audit/SKILL.md` Tier 1 table: add Rust/clippy row (run `cargo clippy --workspace -- -D warnings`, bucket `critical` if compile error), const-block caveat for longest-fn, deferral-note guidance.

**Box-ticking (Tier 3 — ~5 min):**

8. Tick `[x]` for the 23 STALE/SUPERSEDED items in their source retros (see Tier 3 table for exact files). Priority: the 4 items in `session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail.md` (falsifiable-hypothesis gate, pre-locate anchors, cap edit count, failure-class risk) and the 4 items in `session-retro-2026-05-25-rt-r3-cohort-2-git-index-cascade.md`.

**Unverified (5 min):**

9. Spot-check: `ls .claude/lessons/feedback_session_start*` — if the inherited-context-anchors lesson exists, tick it closed; if not, add to Tier 1 #1 session.
10. Spot-check cargo-test.bat exit propagation: `grep -n "exit\|errorlevel\|cmd" scripts/brehon/cargo-test.bat` — confirm all layers propagate, then tick closed.

**Schedule (Tier 2 — allocate to future sessions by cluster):**

- **Homeserver disk hygiene cluster (Tier 2 #1/#2/#3):** cleanup-junior-logs.sh + cron + disk check in weekly-review + journald cap. One homeserver session, ~45 min.
- **Orchestrator rule additions cluster (Tier 2 #4/#5/#6/#12):** outcome≠cause lesson, slash-command scope-vs-locality hook, falsification in validate-pending-laptop, canonical-checkout surface-first status line. One governance-v0 session, ~60 min.
- **Code-quality cluster (Tier 2 #8/#24/#26):** cargo-test.bat audit, empirical token guidance, duplicate-key JSON recipe. One short session, ~30 min.
- **Single-file lesson cluster (Tier 2 #7/#13/#18/#19/#21):** brief overspec, workspace lints, PID-filter, heading-anchor, lib-test env recipe. Batch-write 5 lessons, ~40 min.

---

_Generated by `.claude/skills/retro-harvest/SKILL.md`. Read-only sweep; no rules/lessons/code modified. Currency verdicts are evidence-checked against HEAD `0f0c9a3fd` — re-run after acting to confirm closure._
