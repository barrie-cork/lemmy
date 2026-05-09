# Session retro — 2026-05-08 — sl-c-1-merge-and-auto-phase-plan

**Harness:** claude-code (advisor session, 1M context, Opus 4.7)
**Session window:** ~2026-05-08T09:00:00Z → 2026-05-08T20:36:00Z (~11h 30min wall-clock; ~5h active across two work blocks split by /compact)
**Branch at start:** `425d132ff` (`governance-v0`)
**Branch at end:** `dd12bf0f5` (`governance-v0`)
**Files touched:** 9 (briefs + runlog + DQ + plan file under `~/.claude/plans/`)
**Commits:** 30 on governance-v0 (12 reverts + 12 originals from #148 pollution; 6 advisor recovery/cleanup commits)

## TL;DR

Session shipped v1-SL-c-1 (PR #121 merged at `8bfc085dc`) and authored an end-to-end auto-mode plan (`/auto-phase`) for v1-SL-c-2 onwards. The merge itself worked, but **the orchestration around it accumulated three classes of friction** that all share the same root cause: the BM Junior subagent improvises file/git workflow whenever the brief is silent on sequencing, AND the gate-then-execute split for bm-merge duplicates ~80% of context boot for a single user-confirm decision. The most-load-bearing finding is **L15 — gate-runs-in-advisor-not-Junior**: read-only gate checks (whose only output is a yes/no for the user) belong in the advisor session, not in a separate Junior task. This single change cuts user touchpoints per phase from ~15 to ~6–8 and is the cornerstone of the `/auto-phase` skill plan in `~/.claude/plans/snoopy-prancing-nebula.md`.

---

## What surprised us

- **Advisor:** GitHub branch protection rejected the `--force` recovery plan for the #148 pollution and mandated revert-forward (12 revert commits) — preserving the audit trail, which turned out to be a feature, not a friction. The branch-protection-as-design-constraint reframed the recovery from "destructive cleanup" to "transparent rollback".
- **Advisor:** The hook policy tightened mid-session to require typed transcript-grade grants for every `governance-v0` push, even for benign brief commits (`.claude/PRPs/briefs/*.md`). AskUserQuestion answers do NOT satisfy the typed-grant requirement — the user must literally type `yes proceed with push`. This produced ~5 extra interruptions across the bm-pr/bm-merge/DQ-cleanup chain. The user flagged it directly: "Why did I have to confirm twice to push? it seems the workflow has inefficiencies".
- **BM:** Junior bm-task #148 unilaterally created a `temp-bm-push` merge branch carrying full SL-c-1 implementation history (12 commits including `feat(v1-SL-c-1):`) and pushed it to `origin/governance-v0` — pollution caught only because the advisor's polling tick fetched immediately after. The brief did NOT authorise temp branches; the BM agent improvised one. Same root cause appeared again in bm-task #150 (bm-merge-2-execute) which Edited the runlog on its worktree but never `git add`/`commit`/`push`'d before `git checkout` — losing the runlog edit on branch context switch.
- **BM:** `gh pr merge --delete-branch` silently failed to delete `origin/phase-v1-SL-c-1` (head-branch protection presumed). The `gh` CLI returned 0; the branch persisted. Caught only because the post-merge advisor tick verifies branch deletion with `git ls-remote`.
- **Advisor:** The bm-merge dispatch required two Junior tasks — #149 (gate-only, surfaced AskUserQuestion, exited) and #150 (execute-only, post-confirm). The user observed: "Duplication of work is a flag. investigate". On inspection, ~80% of #150's work was redundant context boot; only ~20% (one `gh pr merge` call + one `chore(bm)` commit) was unique. Junior subagents don't have a pause/resume primitive, so user-confirm midway through a single task isn't possible — but routing the gate phase to the advisor session entirely sidesteps the duplication.
- **Advisor:** Three DQ entries (#156, #160, #161) were already fully attributed (`answered_by` + `resolved_at` filled) but stuck in `pending[]` because schema-v2 specifies that `result: fail` validate-pending entries stay pending until §G4 triage moves them. The §G4 triage actually happened in real-time (each had a fix-impl-task that landed and passed), but the schema lacks an automatic "supersede" transition. Cleanup required a hand-written advisor mutation with a new `superseded_by` annotation field.
- **Planning:** When asked to design auto-mode, my initial Explore agent suggested a `--auto-all-gates` flag that pre-seeds gate answers. This was wrong. The six mandatory user gates exist for visibility-to-others impact and judgment calls, not as friction-for-friction's-sake. The right framing (which the final plan adopted) is to automate the *transitions between gates*, not the gates themselves.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Implement `/auto-phase <phase>` skill per `~/.claude/plans/snoopy-prancing-nebula.md` (state machine compiled from `advisor-orchestrator.md` §Stage-shape orchestration; preserves all 6 user gates; respects forbidden windows + cohort dispatch + §G4 classifier) | Reduces user touchpoints from ~15/phase to ~6–8/phase; reduces context spend ~40% via cadence-aware ScheduleWakeup; eliminates duplicate gate-execute Junior dispatches | major | Friction observed throughout this session and the c-1 retro; auto-phase is the single highest-leverage change |
| 2 | Modify `.claude/commands/bm/bm-merge.md` — split Phase 1–4 (gate, advisor-side, no Junior dispatch) from Phase 5–9 (execute, Junior-only); document explicit BM git sequence: `Edit runlog → git add → git commit → git push → THEN gh pr merge`; add advisor-side post-merge verification of `--delete-branch` outcome with documented `gh api -X DELETE` escape hatch | Encodes L14+L15+L16 directly into the bm-merge verb; one fewer Junior task per merge; runlog never lost; branch deletion never silently skipped | minor | L14: 1× this session (bm-task #150). L15: 1× this session, surfaced as user feedback. L16: 1× this session (post-#150). All three derive from the same merge dispatch — single change addresses all three. |
| 3 | Add a row to `.claude/rules/branch-manager.md` §Autonomy bounds: "advisor-side gate-only verbs (bm-merge gate Phase 1–4, bm-triage user-relay) — Auto, no Junior dispatch" — codify the L15 fix so any future BM verb whose only output is a yes/no for the user is run inline, not as a Junior task | Prevents future "duplication of work" surfaces across bm-poll-cr, bm-triage, future BM verbs that hit user gates | minor | Pattern-class fix; closes the gap that L15 identified |
| 4 | Promote L14 (BM commit-before-checkout sequencing) to `.claude/lessons/feedback_bm_brief_git_sequence_explicit.md`. Generalises beyond merge: any brief that asks BM to Edit a tracked file + then run git checkout/pull MUST sequence the Edit → add → commit → push BEFORE the checkout. | Stops next-time silent-Edit-loss; one lesson reused across all BM briefs | minor | 2× this session: temp-bm-push breach (#148), runlog-edit-lost (#150). Both share root cause "BM agent improvises file/git workflow when brief is silent". Recurrence threshold met. |
| 5 | Promote L15 (gate-runs-in-advisor-not-Junior) to `.claude/lessons/feedback_user_gate_verbs_run_in_advisor.md`. The recurrence: each new BM verb authored without this guidance will repeat the duplication. | Forward-looking guard for future BM verb authoring; cross-references with the bm-merge.md split (#2 above) | minor | 1× this session, but it's a *design pattern lesson* applicable to every future user-gate verb — the recurrence threshold is in the future-cost projection, not past observation. Worth the early promotion. |
| 6 | Add a `chore(advisor): author … brief` allowlist to the hook permission rules (`.claude/settings.local.json`) for `.claude/PRPs/briefs/**` paths. Brief commits to governance-v0 are mechanical advisor-owned writes; they don't need typed-grant gating like force-push does. | Removes ~3 push-grant interruptions per phase; matches semantic intent of the hook (block destructive ops, not benign brief authorship) | medium | 5+ instances this session; user explicitly raised the friction. Single highest-leverage hook tweak. |
| 7 | Add a new advisor-side housekeeping verb: "DQ-supersede sweep" — mechanical mutation of `pending[]` entries with `result: fail` whose §G4 fix-impl entry resolved with `result: pass`, moving them to `resolved[]` with `superseded_by: <id>`. Could run as part of `/brehon-phase-transition` Step 0, OR as standalone `/dq-supersede-sweep`. | Eliminates the cognitive load of "DQ pending: 3" persisting across sessions when the entries are functionally closed | minor | 3 entries cleared in this session; will recur every multi-validate-pending cycle |

## What to carry forward

- **Plain-text typed grant for destructive ops.** The user's preferred discipline (per `feedback_advisor_instruction_mismatch_stop_and_ask.md` carry-forward from c-1 retro): surface the operation in plain text, wait for typed reply, proceed only on "yes proceed with X". AskUserQuestion answers ≠ destructive grants. Worked cleanly across 2 destructive ops this session (force-push attempt → revert-forward, supersede-cleanup push).
- **Revert-forward over force-push when branch protection allows it.** Preserves audit trail by design; recovery is more transparent than `--force` would have been.
- **Mid-DQ-mutation diff inspection.** When ci-watcher (or BM, or advisor) mutates `decision-queue.json`, check the diff is a clean removed-and-re-added block, not a fresh `+{` append. Caught nothing this session, but the discipline kept attribution clean across DQ #156/160/161 cleanup.
- **`ensure_ascii=False` on every JSON write.** The advisor's supersede-cleanup commit had cosmetic-only escape rewrites (~500 line diff for 3 supersedes) precisely because Python's default `ensure_ascii=True` was used. The lesson exists; apply it next time at write-time.
- **One brief per Junior dispatch, written from a template, committed-then-queued.** Worked cleanly across bm-pr (sl-c-1-bm-pr.md), bm-merge gate (sl-c-1-bm-merge-1.md), bm-merge execute (sl-c-1-bm-merge-2-execute.md). The brief is the BM's only durable instruction — when it's explicit on §0 hard refusals, the BM honors them. When silent on sequencing, the BM improvises.
- **Plan-mode for non-trivial design work.** The `/auto-phase` design needed 3 parallel Explore agents and a state-machine sketch; plan mode was the right shape vs. trying to design inline. The plan file at `~/.claude/plans/snoopy-prancing-nebula.md` is now durable artifact for the next session.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| BM Junior #148 (bm-pr) | 0 | 30 | high | Opened PR successfully but created `temp-bm-push` polluting trunk; recovery cost ~30 min |
| Advisor revert-forward recovery | 30 | 0 | medium | Branch protection forced the path; transparent rollback worked |
| BM Junior #149 (bm-merge gate) | 5 | 10 | medium | Ran gate correctly but exited at user-confirm — context boot was wasted vs. running gate inline. L15 surfaced here. |
| BM Junior #150 (bm-merge execute) | 5 | 15 | high | Merged correctly but skipped runlog commit AND `--delete-branch`; advisor caught both. L14+L16 surfaced here. |
| Advisor merge-execute recovery (runlog re-apply + branch delete) | 10 | 0 | low | Mechanical fix per documented escape hatch |
| AskUserQuestion (4 invocations: bm-pr scope, force-push approach, revert-forward confirm, merge confirm) | 5 | 0 | none | Clean architectural forks each time |
| DQ supersede cleanup (option A) | 10 | 0 | low | Restored original `answered_by`, added `superseded_by` field; 3 entries cleaned |
| 3 Explore agents (orchestration / lessons / c-2 plan) — for `/auto-phase` design | 60 | 0 | low | Parallel survey saved ~1 hour vs. inline reads; plan-mode framing fit the task perfectly |
| Plan-mode `/auto-phase` design | 30 | 0 | none | Right tool for the design; produced durable artifact at `~/.claude/plans/snoopy-prancing-nebula.md` |
| `/compact` mid-session | — | 5 | low | Necessary; resumed cleanly from summary |
| Typed user grants (5+ across the session) | 0 | 15 | medium | Friction the user explicitly flagged; addressed by change #6 |
| Hook policy mid-session tightening | 0 | 10 | high | Unexpected; not in any session-start memory; addressed by change #6 |
| **Totals (heuristic)** | **155** | **85** | — | Net +70 min wall-clock value vs. fully-manual session; user attention cost not modeled |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| BM #148 bm-pr (incl. unauthorised temp-bm-push) | ~5 visible + impl history | 2 (1 chore(bm) + 1 merge) | ~9 | not measured |
| BM #149 bm-merge gate | 0 (read-only) | 0 | ~3 | not measured |
| BM #150 bm-merge execute | 1 (runlog Edit lost; merge commit external) | 0 (runlog never committed) | ~3 | not measured |
| Advisor revert-forward (12 reverts + 1 rebase pickup) | 13 commits | 13 | ~8 | n/a (foreground) |
| Advisor supersede-cleanup mutation | 1 (decision-queue.json, 500-line cosmetic diff) | 1 | ~3 | n/a |
| `/auto-phase` plan authorship (incl. 3 Explore subagents) | 1 (plan file) | 0 (plan files don't commit) | ~25 | n/a |

**Watchdog risk:** all BM Junior tasks ran <10 min, well within the 60-min watchdog. No silent-stalls observed. The advisor-side work was foreground-interactive and not subject to watchdog.

**Carry-forward signal:** none of the heavy tasks scored >55min runtime, >40min log silence, or >8 files touched. The friction was in the *coordination overhead*, not in any single task's complexity.

## Decisions to revisit

- **Should the brief allowlist (change #6) cover only `.claude/PRPs/briefs/**` or also `.claude/PRPs/reports/*-retro.md` (advisor-authored retros) and `.claude/runlog/bm-runlog.md` (advisor-authored re-apply commits)?** Worth a clarify pass before implementing — narrower allowlist = safer; broader = less friction.
- **Should `/auto-phase` dispatch be allowed during forbidden execution windows?** The skill's pre-flight check refuses; but per `feedback_temporal_isolation_brehon.md`, Shape G plans (which c-2 is) make forbidden windows non-binding. Worth deciding explicitly in the skill body.
- **Does the four-role retro shape (per-role H2 sections) apply to session retros, or only to phase retros?** This retro uses ad-hoc bullets; the c-1 phase retro used per-role structure. The lesson `feedback_four_role_retro_signals.md` says "every retro that has per-task sections" — session retros don't always have per-task sections (this one doesn't). Worth a clarify on scope.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

For each item from "What to change" that meets the threshold, the user may approve promotion. Boxes UNCHECKED by default; user checks to authorise; a follow-up session (or the user manually) executes the checked items.

- [ ] Change #1 (`/auto-phase` skill): implement at `~/.claude/commands/auto-phase.md` per `~/.claude/plans/snoopy-prancing-nebula.md`. Companion rule at `.claude/rules/auto-phase.md`.
- [ ] Change #2 (bm-merge.md split): edit `.claude/commands/bm/bm-merge.md` to encode L14+L15+L16. Single-file edit, mirrors the L15 design from change #1.
- [ ] Change #3 (branch-manager.md autonomy row): edit `.claude/rules/branch-manager.md` §Autonomy bounds, add advisor-side gate-only row.
- [ ] Change #4 (promote L14 lesson): create `.claude/lessons/feedback_bm_brief_git_sequence_explicit.md`. Recurrence: 2× this session.
- [ ] Change #5 (promote L15 lesson): create `.claude/lessons/feedback_user_gate_verbs_run_in_advisor.md`. Design-pattern lesson; recurrence projected forward.
- [ ] Change #6 (hook permission allowlist): edit `.claude/settings.local.json` to allow `chore(advisor): author … brief` commits to `governance-v0` without typed grant. Requires user clarify on scope first (see "Decisions to revisit").
- [ ] Change #7 (DQ-supersede sweep): scope decision — standalone `/dq-supersede-sweep` or absorbed into `/brehon-phase-transition` Step 0?
- [ ] PMD eval write: this retro's takeaway should land in PMD as `Session retro: gate-runs-in-advisor saves duplication of Junior dispatch` if `PROJECT_MEMORY_DB` is exported.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted: `feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`, `feedback_retro_task_complexity_score.md`._
