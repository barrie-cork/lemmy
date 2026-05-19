# Session retro — 2026-05-18 — fed-in-a Cohort A serial recovery + Item 3 provenance reject

**Harness:** claude-code
**Session window:** ~2026-05-16T23:00Z → 2026-05-18T11:37Z (~spanned, with /clear + multiple ScheduleWakeup ticks; ~10h active advisor wall-clock across the recovery)
**Branch at start:** `fdd3a5a78` (`phase-v1-federation-inbound-a`, lane worktree) / catch-fire stage
**Branch at end:** `5e02c622f` (`phase-v1-federation-inbound-a`); daemon-local `82ac0b885` ahead (6th finalize-merge race, handed to resume)
**Files touched:** ~6 distinct (migrate-roundtrip.sh + new .bat, 3 Cohort B briefs, decision-queue.json ×many, auto-state JSON ×many) — all advisor-class (no crates/ — advisor never authors content)
**Commits:** 21 on phase branch (`961101fc7`..`5e02c622f`) — all advisor/impl/decision-queue scoped; 0 auto-commits (advisor session)

## TL;DR

A `/auto-phase` recovery session that drove `v1-federation-inbound-a` out of a catch-fire (5-way Cohort A SIGTERM cascade) through a clean strict-serial re-run of all 5 Cohort A tasks (5/5 VALIDATED-PASS, zero auto-retries, every git reconcile lossless), authored + dispatched Cohort B Task 6, and resolved the long-standing Item 3 by **rejecting an unattributed security-gate-bypass hook after provenance verification failed**. The single most load-bearing finding: the **Junior finalize-merge race recurred 6 times** with an identical proven lossless-reconcile recipe (verify tree-hash equiv → confirm origin-is-ancestor → push daemon's merge commit FF-FORWARD) — this is now a definite lesson-promotion candidate, and the recipe should be codified so future advisors don't re-derive it every cohort task. Second: the Item 3 handling validated a discipline worth promoting — *absence of provenance is itself a reject signal for a security-boundary artifact*, and the prior session's classifier-block of that exact trunk commit was correct.

---

## What surprised us

- **The catch-fire dump's world-state was already stale on resume.** The dump (written before `/clear`) prescribed a destructive `git restore --staged --worktree` to discard a DQ #232 breach on the daemon. By the time the next session reconciled, the daemon had moved to the `phase-v1-ship-1` lane and the breach was *already cleared* — executing the dump's prescribed destructive op would have been both unnecessary AND wrong. Reconciliation-before-acting-on-dump-prescriptions saved a wrong destructive action.
- **The Junior finalize-merge race is not an anomaly — it's deterministic recurring behaviour.** It fired on Tasks 2, 3, 4, 5, AND Cohort-B-Task-6 (5 times this session; #283 was the manual baseline). Every single time: worker pre-pushes its branch → advisor manual-FF AND daemon-finalize both merge the same worker → tree-identical merge commits → lossless reconcile via FF-forward. The *consistency* was the surprise; this should never have been treated as a per-incident reconciliation.
- **The impl-task ascii-escape DQ-write breach is intermittent, not universal.** Workers #283/#305/#308/#310 wrote `.claude/decision-queue.json` ascii-escaped (`ensure_ascii=True`); worker #307 wrote it canonical. Same subagent, same brief class, different encoding — suggesting the breach depends on *which* DQ-write code path the impl-task takes, not a blanket helper bug.
- **`git diff --stat` line-count actively misleads on JSON re-serialization.** A DQ mutation showed `3995/3995` (whole-file churn, looked like a DQ #232 reformat breach) while the *content* diff (deep JSON equality) showed only the one mutated entry changed. The git line-count is whitespace-sensitive on `\uXXXX`-vs-literal-UTF8 re-encoding; it is NOT a valid DQ #232 reformat check — deep-equality on entry content is.
- **Machine clocks (laptop AND EliteDesk) were ~59 min behind user wall-clock**, and they *agreed with each other*. When the user said "it's 4.15" the machines said ~03:16. Not a single-machine NTP glitch — a consistent ~59min skew across both, recurring (the state file had flagged ~57min earlier in the phase).
- **The impl-task task-0 forbidden-window guard HONOURS a dispatch-string override annotation.** I'd recorded a pessimistic caveat that the guard reads the EliteDesk clock and would `FORBIDDEN_WINDOW`-exit regardless of a DQ. Verified by reading #310's task-0 log: the subagent explicitly read the `(user-authorised forbidden-window override per DQ #245)` annotation in the dispatch description and proceeded. The override mechanism is the *annotation in the create_task string*, not the DQ alone — and it works.
- **The Item 3 hook's provenance did not survive verification.** It is untracked with zero git history anywhere, sha256-stable but re-touched 2026-05-17 19:37 (≈23h *after* the catch-fire dump's claimed creation, outside any traceable session), never registered in any settings.json (inert the whole time), and — decisively — the only cited provenance trail (the Junior #270 escalation report) proposes a *different* fix (daemon-spawn-args or settings.json-allowlist), never a PreToolUse hook. The escalation also proves the settings.json-allowlist approach is ineffective against the hardcoded gate, so the real fix is a homeserver daemon patch, not a brehon-fork commit.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Promote a lesson `feedback_junior_finalize_merge_race_lossless_reconcile.md`** + extend `feedback_junior_finalize_skips_when_worker_pre_pushes.md` (which covers only the SKIP case, not the BOTH-RAN case). Codify the proven recipe: worker pre-push → advisor-FF + daemon-finalize both merge same worker → verify `<daemon-ref>^{tree} == <my-FF>^{tree}` → confirm `origin` is ancestor of daemon-ref → push daemon's merge commit **FF-FORWARD** to origin (never force/rewind) → sync lane. | Advisor stops re-deriving the reconcile every cohort task; ~3-5 min/task saved across every future multi-cohort phase. The recipe is proven 6×. | minor (one lesson file + one extension) | **6× this session**, recurs every Shape-G-suspended cohort task |
| 2 | **Document the override-via-dispatch-annotation mechanism in `advisor-orchestrator.md` §5.1 "When to override".** Add: "the override is effected by the `(user-authorised forbidden-window override per DQ #<id>)` annotation IN the `create_task` description string — the subagent's task-0 guard reads the description and honours it; the DQ is the audit trail, not the mechanism the subagent reads." | Removes advisor uncertainty (this session carried a wrong pessimistic caveat); makes the override path reliable + documented. | minor (rule edit) | 1× this session + 0 prior, but it's a correctness gap in a documented procedure |
| 3 | **Promote a lesson on unattributed-security-artifact rejection** (`feedback_unattributed_security_artifact_reject.md`): a security-boundary artifact on shared infra (hook, settings change) with no git history / no escalation-report tie / re-touched outside any traceable session MUST NOT be adopted on a multiple-choice or terse authorization. **Absence of provenance is itself a reject signal.** Cite: the prior session's classifier-block of this exact trunk commit was CORRECT. | Codifies the discipline that prevented a security-gate-bypass commit of unknown origin; reusable for any future "adopt this infra artifact?" decision. | minor (one lesson file) | 1× here + 1× prior (the prior session's classifier-block of the same artifact) → meets ≥2 threshold |
| 4 | **Refine the serial-cap check to query `branch LIKE '%<phase>%'` at the sqlite level** (in the auto-phase tick + advisor-orchestrator §4.1), instead of `COUNT(*)` all-non-terminal-then-identify. | Removes the recurring cross-lane false-alarm (3× this session: #306, #309, plus — every n_active>0 was a different-lane task) + saves one SSH round-trip per tick. | minor (query change in skill body) | **3× this session**, 0× prior |
| 5 | **Add to `.claude/lessons/feedback_json_dump_ensure_ascii_false.md` (or DQ-recipes ref):** the real DQ #232 reformat check is deep-JSON-equality on entry content (`content-changed entry ids`), NOT `git diff --stat` line-count — the latter misleads on `ensure_ascii` re-serialization even when content is byte-equivalent. | Prevents future advisors from either (a) panicking on a benign 3995/3995 churn or (b) missing a real reformat hidden by whitespace-insensitive diff. | minor (lesson augmentation) | 2× this session (Tasks 2+4 canonical-re-encode), 0× prior |
| 6 | **Pin `ensure_ascii=False` in the impl-task DQ-write recipe** (`.claude/refs/dq-recipes.md` + `.claude/agents/impl-task.md` + the impl-task subagent's mid-task DQ-raise helper path). Audit *both* DQ-write paths (Recipe 1 vs ad-hoc) since the breach is intermittent (#307 canonical, #283/305/308/310 escaped). | Eliminates the recurring ascii-escape breach at source → no more advisor canonical-re-encode-on-mutate (which forces a 500+-line diff + disclosure every time). | medium (touches impl-task contract + helper; intermittent so needs both-path audit) | **4× this session** (#283/305/308/310), recurs ~every other impl-task |
| 7 | **Scope the real DQ #235 fix as a dedicated homeserver task** (the §5(a) daemon-spawn-args patch: `--dangerously-skip-permissions` at `/opt/junior-src/src/daemon/executor.ts` + `/src/core/claude.ts`). NOT a brehon-fork commit; NOT mid-phase. | Closes DQ #235 properly (currently every planning Junior escalates; the rejected hook was an unattributed band-aid). | medium (homeserver daemon patch + smoke test) | DQ #235 unresolved since 2026-05-16; recorded DQ #247 |
| 8 | **NTP investigation on laptop + EliteDesk** (a homeserver infra item). Until fixed: forbidden-window math uses machine `date -u` (authoritative for the cron contention the window guards); user wall-clock disagreements go via the DQ + dispatch-annotation override mechanism. | Removes the recurring clock-skew confusion (surfaced as a user-facing decision twice). | medium (infra; homeserver-side) | ~2× flagged this phase (~57min then ~59min, consistent) |

## What to carry forward

- **Verify-then-act on catch-fire dump prescriptions.** A catch-fire dump is a *hypothesis about world-state at dump-time*, not a runbook. Reconcile live world-state before executing any dump-prescribed destructive op. This session that discipline prevented a wrong `git restore --staged --worktree` on shared daemon state. (Directly applies the state-file's own `_process_lapse_2026_05_16` verify-then-reset lesson.)
- **"I authorise" / terse-authorization → AskUserQuestion to confirm scope before acting** when multiple distinct items are outstanding, *especially* if any is a security-boundary. Did this twice cleanly this session ("I authorise" → which item? → verify-first); never inferred scope. The cost of one extra question is far below the cost of an unwanted security-gate commit.
- **Verified-not-trusted, applied relentlessly.** Every bg-task notification checked against the explicit exit-marker (`feedback_background_task_notification_lies`); every daemon-ref divergence tree-hash-verified before reconcile; every serial-cap n_active>0 identified by branch field (never assumed a violation); brief-scope checked by file-level `git diff --name-only`, not the commit subject. This caught the wrong-destructive-op, the masked-DLL-failures, and 3 false serial-cap alarms.
- **Strict-serial recovery discipline under Shape-G-suspended.** The 5-way [P] Cohort A SIGTERM-cascaded; the user-directed cap=1 re-run completed all 5 + Cohort B Task 6 with zero auto-retries and zero contention-kills. On this EliteDesk hardware, under validate-pending-laptop, cohort width must be 1 — the [P] markers are not safe here. Carry this into any future Shape-G-suspended phase.
- **Safe-stop discipline.** When the user says "stop when safe", choose the boundary *after* a clean recorded decision, do NOT rush an in-flight reconcile (the 6th finalize-merge race) at a session edge, and write a precise actionable `_RESUME_HANDOFF` to auto-state. Correctly identified the canonical-checkout detached-HEAD as orthogonal-and-not-to-touch-at-the-boundary.
- **Brief authoring at cohort-transition follows the full discipline even mid-phase:** §2.3 PMD pre-search (consult-only), §2.4 mandatory file-class lesson injection, §3.6 canonical-schema-first (mirrored the Cohort A brief verbatim for structure), and embedding the R9 callsite enumeration into brief 8 per `feedback_fix_impl_enumerate_all_callsites` applied to a struct-shape change.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/auto-phase` state machine (whole session) | ~120 | ~25 | medium | Drove ~21 transitions across catch-fire→recovery→Cohort-A-5/5→Cohort-B-T6 with ~6 user touchpoints; the ~25 wasted is the re-derived finalize-merge reconcile ×6 (→ change #1) + 3 cross-lane serial-cap false-alarms (→ change #4) |
| validate-pending-laptop §5.2 handler (×6: DQ #241,242,243,244,246 + Cohort-B-T6 pending) | ~40 | ~5 | low | Each cargo-validation verified via explicit exit-marker not bg-notification; clean. ~5 wasted on the migrate-roundtrip harness-gap diagnosis (cycle 1, before the fix) |
| Item 3 provenance-verify (escalation report read + daemon git history + sha256/mtime/registration probes) | ~60 | 0 | high | Prevented an unattributed security-gate-bypass trunk commit; concluded reject on a non-obvious finding (hook ≠ what escalation proposed). High-value, zero waste |
| AskUserQuestion (×~6: gov-mirror, DQ#232 reencode, clock-override, "I authorise"-scope, Item-3-after-provenance, Cohort-B-mode) | ~30 | 0 | none | Every one was a genuine judgment-heavy / visible-to-others fork correctly surfaced rather than auto-decided. The "actuarially in serial" correction caught a sub-optimal prior answer cleanly |
| Cohort B brief authoring (3 briefs from plan §13 + MIRROR + §2.4 lessons) | ~25 | 0 | low | Mirrored Cohort A structure (canonical-schema-first); R9 enumeration embedded in brief 8 pre-empts a likely Task-8 fix-impl cycle |
| Item 1 migrate-roundtrip.sh OS-conditional fix (961101fc7) | ~30 | ~10 | medium | Fixed the Windows libpq harness gap; ~10 wasted on the initial STATUS_DLL_NOT_FOUND diagnosis (but it was a documented pattern — cross-referenced `feedback_windows_e2e_requires_bat_wrapper` quickly) |
| Lossless finalize-merge-race reconcile (×6) | ~15 | ~20 | medium | NET-NEGATIVE on wasted because re-derived each time — this is precisely why change #1 (codify the recipe) is the top proposal |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. Format `<files>/<commits>/<runtime-min>/<max-log-silence-min>`. (These are *advisor-orchestration* tasks, not Junior impl — runtime is wall-clock advisor-attended; "log silence" ≈ longest gap between advisor state-changing actions during that sub-thread.)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Catch-fire recovery (Items 1+2+3 triage + Item 1 fix + DQ #241 re-validate) | 4 | 3 | ~75 | ~25 (bg migrate-roundtrip) |
| Cohort A serial T2→T5 (4 tasks × {finalize-merge reconcile + §5.2 + DQ mutate}) | ~3 | 12 | ~210 | ~40 (longest = Task 5 dispatch deferred past forbidden window, then ~2h ScheduleWakeup) |
| Item 3 provenance-verify + reject + DQ #247 | 1 | 1 | ~30 | 0 |
| Cohort B brief authoring + T6 dispatch | 3 | 2 | ~35 | 0 |

No single advisor sub-thread exceeded the >55min-runtime flag in a way that signals planning-bundling drift (the long ones are gated waits — forbidden window, bg cargo — not work-density). The Cohort A serial block's ~210min total is the cost of the user-directed cap=1 discipline (correct trade vs the 5-way SIGTERM); not a bundling problem.

## Decisions to revisit

- **Item 1 gov-v0 mirror is still deferred** (the migrate-roundtrip.sh fix `961101fc7` lives only on the lane branch). It folds into the eventual merge-to-trunk, but if another lane needs migrate-roundtrip on Windows before this phase merges, the deferral becomes a gap. Revisit at phase-merge or if a sibling lane hits the same harness gap.
- **Canonical `brehon-fork` checkout is detached-HEAD + behind `origin/governance-v0`** (multi-lane churn). Orthogonal to the recovery, deliberately untouched at the session boundary. A future canonical-checkout meta-edit will need to reconcile it first — worth a clean-up pass when no lane is mid-flight.
- **Cohort B serial cap=1 vs the plan's 3-way [P]:** the user re-confirmed serial ("actuarially in serial"). If a future phase wants Cohort-B-style parallelism, the recovery_plan.step_3 override needs an explicit lift decision — the [P] markers are advisory-only for the rest of *this* phase but the next phase's planner may legitimately want parallel. Worth a clarify at the next phase's plan-approval.

---

## Auto-phase reliability

Per `feedback_auto_phase_retro_signals.md`. Artifacts: `v1-federation-inbound-a.json` + 2 catch-fire dumps (`*-2201Z.md`, `*-221221Z.md`). Trigger fired (direct `/auto-phase` invocations + auto-state mutations throughout).

### 1. Stage-transition correctness
✓ — Every `impl-cohort-A-recovery-serial` → per-task → validate → next-task transition fired on the right trigger. Cohort A barrier held (5/5 required before Cohort B; verified). The catch-fire→recovery transition required the explicit `--start-from impl-cohort-1` (correct: catch-fire is terminal, manual re-entry only). No premature/missed fires. Cohort A→B barrier correctly gated brief-authoring (Cohort B briefs authored only after 5/5).

### 2. Cadence calibration
⚠ — No 300s sleeps (good; used 270s/600s/1500s/3600s per the schedule). One sub-optimal: a ScheduleWakeup landed at 05:08 when ~04:16 was intended (scheduler resolved "now" later than estimated, ~52min over). Benign under slow-OK but worth noting the estimate-vs-actual gap. Cross-lane false-alarm SSH probes (3×) added wasted polls (→ change #4).

### 3. Auto-state integrity
✓ — `last_known_phase_tip` tracked accurately through ~21 commits + 6 finalize-merge races. No hand-edits needed to *recover* state (all auto-state writes were forward progress). The `_RESUME_HANDOFF` block was written precisely at the safe-stop. One observation: the auto-state JSON grew large (many `_*_note` keys accreted across ticks) — not an integrity problem but a future-tidiness signal.

### 4. User-touchpoint count vs target
✓ — ~6-8 touchpoints across the whole recovery (catch-fire recovery direction was pre-given; this session's gates: gov-mirror, DQ#232-reencode, clock-override, "I authorise"-scope, Item-3-post-provenance, Cohort-B-mode + the "actuarially in serial" correction). All judgment-heavy / visible-to-others. Zero false-positive AskUserQuestions — each was a genuine fork. Within the 6-8 target.

### 5. Catch-fire FP / FN rate
✓ — Zero new catch-fires this session (the 2 dumps were from the *prior* session's 5-way SIGTERM; this session resolved them). Zero silent advances past a should-have-been-catch-fire. The recovery correctly treated the inherited catch-fire as terminal-requiring-explicit-re-entry.

### 6. §G4 classifier accuracy
✓ — One §G4-relevant event: DQ #241 cmd2 (migrate-roundtrip) fail. Correctly classified **non-allowlist** (harness/environment gap, NOT a code-error allowlist pattern) → catch-fire to user rather than auto-fix-impl. This was the right call (the fix was an OS-conditional wrapper change, not a code recipe). Zero false-positive (no wrong auto-fix shipped), zero false-negative.

### 7. L14 / L15 / L16 fixes still holding
N/A this session — no `bm-merge` reached (phase still in impl cohorts). L14/L15/L16 are merge-stage; will be exercised at the eventual bm-pr→merge. Carry forward: verify them when the phase reaches merge.

### 8. Subagent offload effectiveness
N/A — no `general-purpose` reconciliation subagent dispatched this session (the resume was via `--start-from` from catch-fire, not a Phase-0.5 cold-resume; ticks were warm-context state-machine ticks). Deferred per Phase 0.6; correct not to force it.

### 9. Plan §13 fidelity vs cohort dispatch
✓ (with override) — Cohort A's [P] markers were *overridden* to serial cap=1 per recovery_plan.step_3 (post-SIGTERM, user-directed) — this is a deliberate documented override, not a fidelity failure. Cohort B's `requires:` dependency check ran correctly (Tasks 6/7/8 require task 2+5, both VALIDATED-PASS before dispatch; no circular requires). The mod.rs YAML-overlap (Task 6+7) was correctly noted as moot under serial dispatch. Task 8's R9 enumeration embedded per the callsite-enumeration discipline.

### 10. Resume-cycle pain points
⚠ — Multiple resume cycles (post-`/clear` + ~6 ScheduleWakeup ticks). The post-`/clear` resume correctly read the catch-fire dump + reconciled (and caught the stale-dump-prescription). The repeated ticks were mostly clean state-machine advances, BUT: the cross-lane serial-cap false-alarm recurred 3× (→ change #4) and the finalize-merge-race re-derivation recurred 6× (→ change #1) — these are the resume-cycle friction points. Per the lesson's c-2 framing: first phase under this recovery-serial shape, keep the friction; the two recurring patterns are now promotion candidates.

### Aggregate auto-phase reliability score

| Category | Status | Recurrence |
|---|---|---|
| 1. Stage-transition correctness | ✓ | clean this phase |
| 2. Cadence calibration | ⚠ | 1× over-estimate + 3× false-alarm polls this phase |
| 3. Auto-state integrity | ✓ | clean (tidiness note only) |
| 4. Touchpoint count | ✓ | within 6-8 target |
| 5. Catch-fire FP/FN | ✓ | zero this session |
| 6. §G4 classifier | ✓ | 1× correct non-allowlist call |
| 7. L14/L15/L16 holding | N/A | merge stage not reached |
| 8. Subagent offload | N/A | deferred per Phase 0.6 (correct) |
| 9. Plan §13 fidelity | ✓ | override is deliberate+documented |
| 10. Resume cycles | ⚠ | 6× finalize-merge re-derive + 3× serial-cap false-alarm |

The two ⚠ (categories 2, 10) both reduce to the same two root patterns → changes #1 (codify finalize-merge reconcile) and #4 (branch-filtered serial-cap query). Both meet the ≥2-recurrence promotion threshold. No ✗.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Change #1** (finalize-merge race lossless-reconcile recipe, 6×): promote to `.claude/lessons/feedback_junior_finalize_merge_race_lossless_reconcile.md` + extend `feedback_junior_finalize_skips_when_worker_pre_pushes.md`
- [ ] **Change #2** (override-via-dispatch-annotation mechanism): update `.claude/rules/advisor-orchestrator.md` §5.1 "When to override"
- [ ] **Change #3** (unattributed-security-artifact reject, 1× here + 1× prior classifier-block): promote to `.claude/lessons/feedback_unattributed_security_artifact_reject.md`
- [ ] **Change #4** (branch-filtered serial-cap query, 3×): update `~/.claude/commands/auto-phase.md` tick + `.claude/rules/advisor-orchestrator.md` §4.1
- [ ] **Change #5** (deep-equality not git-line-count for DQ #232 check, 2×): augment `.claude/lessons/feedback_json_dump_ensure_ascii_false.md`
- [ ] **Change #6** (pin ensure_ascii=False in impl-task DQ-write recipe, 4×): update `.claude/refs/dq-recipes.md` + `.claude/agents/impl-task.md`
- [ ] **Change #7** (DQ #235 real fix as homeserver daemon-args task): scope a dedicated homeserver task (per DQ #247 + escalation §5a)
- [ ] **Change #8** (NTP investigation laptop+EliteDesk): homeserver infra item

Single-instance noted-not-promoted: the ScheduleWakeup estimate-vs-actual gap (1×, benign under slow-OK); the auto-state JSON tidiness accretion (1×, not an integrity issue); the canonical-checkout detached-HEAD (1×, orthogonal infra state).

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted: `feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`, `feedback_retro_task_complexity_score.md`, `feedback_auto_phase_retro_signals.md`. PMD eval skipped — `PROJECT_MEMORY_DB` unset this session._
