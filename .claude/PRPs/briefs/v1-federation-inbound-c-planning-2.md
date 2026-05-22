---
phase: v1-federation-inbound-c
role: planning
task: 4
brief_n: planning-2
authored: 2026-05-21
plan: .claude/PRPs/plans/v1-federation-inbound-c.plan.md
plan_task: "§13 Task 4 — Retro authorship"
parent_phase_tip: a195421d8 (phase-v1-federation-inbound-c @ DQ #340 mutated to pass — Phase 2 e2e clean 103/0/5)
cohort: "Cohort 3 (Task 4 alone — retro; requires Task 3 + Phase 2 e2e + any CR-fix-in-PR cycles done per plan §13 requires)"
related_dq: "324 (advisor INSERT-vs-UPDATE clarify, resolved); 325 (workaround-comment plan, resolved); 326 (Junior #393 finalize deletion log + later renumbered conformance-audit leftover); 328 (Task 1 validate pass); 329 (Task 2 validate pass); 337 (renumbered runlog-deletion log); 338 (gov-v0 daemon wrong-ref reset bug — retro analysis target); 339 (Task 3 validate pass); 340 (Phase 2 e2e pass)"
---

# [role:planning] v1-federation-inbound-c Task 4 — retro authorship — see .claude/PRPs/briefs/v1-federation-inbound-c-planning-2.md

> **Role note:** this is a `[role:planning]` Junior task even though it's plan §13 Task 4. Reason: it authors a `.md` report file (`.claude/PRPs/reports/v1-federation-inbound-c-retro.md`) and synthesizes signals across the whole sub-phase — that's planning-role content per CLAUDE.md four-role model. The advisor cannot author content. Pinned to Opus.
>
> **Cohort context:** Task 4 is the SOLE member of Cohort 3 — it has `requires: - task: 3` per plan §13 lines 621-624. Task 3 finalize-merged (commit `877bd849c`) + §5 validated (DQ #339 `result: pass`) + Phase 2 e2e clean (DQ #340 `result: pass`, 103/0/5).

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch matches `junior/<task-slug>-<task-id>` AND it was forked from `phase-v1-federation-inbound-c` at tip `a195421d8` (or any descendant — advisor may have authored further runlog commits between worker-spawn and your read). If `git branch --show-current` shows anything else, or if `git merge-base HEAD phase-v1-federation-inbound-c` is empty → STOP, file `kind: "blocker"` DQ (`from: "planner"`).
- Forbidden-window self-check (per `.claude/agents/planning.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — if inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) exit non-zero with `FORBIDDEN_WINDOW: <window>`. NOTE: this is a markdown-only authorship task (no cargo), so forbidden-window concern is minimal; keep self-check anyway.
- **DAEMON-BUG awareness (DQ #338, gov-v0):** the Junior daemon recently issued a wrong-ref reset on `governance-v0` after a planning task's finalize-merge (Task #399's planning commit). This is documented + investigated in another advisor session. **Mitigation in §4:** worker pre-pushes its commit to the worker branch BEFORE finalize runs; advisor manually finalize-merges from `origin/junior/<branch>` (the cohort-1 + Task 3 pattern). Do NOT rely on daemon finalize-merge.

## §1 Role + dispatch

`[role:planning] v1-federation-inbound-c Task 4 — retro authorship`

The actual create-task description (single line, <100 chars):

```
[role:planning] v1-fed-in-c task 4 retro — see .claude/PRPs/briefs/v1-federation-inbound-c-planning-2.md
```

## §2 Scope

**Produce** (one commit):

- `.claude/PRPs/reports/v1-federation-inbound-c-retro.md` — full four-role retrospective per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`. **NEW (not a report — a retro):** completion reports list what happened; retros surface what should change. Structure: four H2 sections (one per role) + an Aggregate section + a Carry-forward section + a Lessons-promoted section.

**Required structure (literal):**

```markdown
# v1-federation-inbound-c retro

> Phase tip at retro authorship: <SHA from git log -1 --oneline phase-v1-federation-inbound-c>
> Plan: `.claude/PRPs/plans/v1-federation-inbound-c.plan.md`
> Authored: <YYYY-MM-DD by Junior planner #N>
> Phase wall-clock: <bm-cut timestamp> → <last commit timestamp> = <N> hours
> §16a stories: 3/3 done (Stories 1+2+3)
> Phase 2 e2e: 103 passed / 0 failed / 5 ignored (v0-polish TODOs)

## Advisor signals

(... per feedback_four_role_retro_signals.md Advisor section — what advisor did well + what advisor missed + recurrence-class observations ...)

### Recurrence-class observations (load-bearing — promote to lessons if 3+ occurrences)

- **/tmp Bash↔Python path mismatch recurrence (2x this phase, 2x prior).** Despite 12-day-old canonical lesson `feedback_windows_bash_python_git_show_tmp_traps.md`, the trap fired twice during this phase's DQ reconcile + post-merge inspection (user explicitly flagged "Note this for retro: /tmp path mismatch again" 2026-05-21). Hypotheses captured in runlog entry "/tmp Bash↔Python path mismatch retro-candidate (user-flagged 2026-05-21)" lines 63-106. Two structural-fix options: (a) `/check-tmp-paths` lint or Bash-tool pre-execution hook flagging `/tmp/...` paths; (b) user-scope CLAUDE.md instruction "on Windows, never use /tmp for cross-process file handoff; default to $LOCALAPPDATA/Temp" so the muscle-memory default shifts at brief-time, not error-time. **PROMOTION RECOMMENDATION:** add to PMD as `feedback_tmp_path_mismatch_promote_to_session_start_default.md` (NEW lesson) — fires across phases, not phase-specific.

- **Lane bootstrap submodule init miss (2x: this lane + at least one prior — verify by reading prior bootstrap retros).** Lane worktree creation via `git worktree add` does NOT auto-init submodules; the `crates/email/translations` submodule was uninitialized in `brehon-fork-fed-in-c`, causing Probe 3 first-run failure with `lemmy_email build.rs Os code 3 NotFound`. Recovery: `git submodule update --init --recursive crates/email/translations`. Canonical lesson `feedback_phase_lane_worktree_bootstrap_checklist.md` exists; was not applied at lane-creation time. **PROMOTION RECOMMENDATION:** the lesson exists; the gap is checklist-execution discipline. File a `kind: "log"` DQ flagging that lane-bootstrap steps need a one-command idempotent runner script (e.g. `scripts/brehon/lane-bootstrap.sh <phase-name>` that does worktree-add + submodule-init + .mcp.json copy + settings.local.json copy + PMD-canonical-path verification).

- **DQ id-collision recurrence (3rd documented occurrence).** Conformance-audit lane renumbered gov-v0 #308 to #326; this lane then independently filed a #326 (runlog-deletion RCA) and forward-merge had to renumber phase's #326 to #337. Per session-start cross-lane next_id walk would catch this; the walk is currently advisor-side manual. **PROMOTION RECOMMENDATION:** the lesson `pattern_dq_log_shape_as_blocker.md` is sibling but not the same root. File NEW lesson `feedback_multi_lane_dq_id_collision_walk.md` (or check if `feedback_cohort_dq_id_collision.md` already covers — extend if not). Structural fix: `scripts/brehon/resolve-dq-canonical.sh` already supports cross-archive; extend to cross-lane via `git worktree list` parse + `git show <other-lane>:.claude/decision-queue.json` next_id walk. **Future scope tracker** (don't ship in this retro).

- **DQ #338 daemon wrong-ref reset bug AVOIDED via pre-push mandate.** Cohort 1 (Tasks 1+2) and Task 3 all used the pre-push pattern (worker commits + pushes; advisor manually finalize-merges from `origin/junior/<branch>`). Daemon-finalize wrong-ref reset never had opportunity to fire on `phase-v1-federation-inbound-c`. Per cohort 1 + Task 3 daemon-side phase reflog: only the 3 lossless reset-to-origin steps (BOTH-RAN reconcile post-merge). **PROMOTION RECOMMENDATION:** the lesson `feedback_junior_finalize_skips_when_worker_pre_pushes.md` exists and was load-bearing. Confirm that lesson + brief §4 PRE-PUSH MANDATE successfully migrated into the planning workflow. Note in carry-forward: until DQ #338 structural fix lands, every impl-task brief MUST carry the pre-push mandate.

## Planning signals

(... per feedback_four_role_retro_signals.md Planning section — dogfood evidence held; PRECON enumeration honored; §10.1 mirror correctly cited by both impl-task briefs; §13 task spec precision (cite line numbers + anchor-by-text rules); etc. ...)

### Specific dogfood checks (verify at retro time, not at plan-write time)

- §15.1 cargo-check: passed at Tasks 1, 2, 3 + post-merge — confirmed via DQ #328, #329, #339 + post-merge cargo-check log.
- §15.2 clippy: passed uniform at Tasks 1, 2, 3 + post-merge — confirmed via same DQ + post-merge clippy log. NO disallowed_methods firings from new federation-mod-roots deny attribute (conformance-audit merge).
- §15.3 cargo test --no-run: passed at Task 3 — confirmed via DQ #339.
- §15.4 Phase 2 e2e: 103/0/5 — confirmed via DQ #340 + log.
- §15.6 cross-cutting verification: walk all 21 boxes against phase tip. (Junior planner runs `grep`/`rg` per box at authorship time and reports the count.)
- §15.7 ADR/OQ compliance: walk all 5 boxes. (Mechanical — no row-shape changes; reader-side only.)

## Impl signals

### Per-task complexity scores (per feedback_retro_task_complexity_score.md)

```
Task 1: <files-touched>/<commits>/<wall-clock-minutes>/<max-log-silence-minutes>
Task 2: <files-touched>/<commits>/<wall-clock-minutes>/<max-log-silence-minutes>
Task 3: <files-touched>/<commits>/<wall-clock-minutes>/<max-log-silence-minutes>
Task 4 (this retro): <files-touched>/<commits>/<wall-clock-minutes>/<max-log-silence-minutes>
```

Source: read `git log --format="%H %ai %s" phase-v1-federation-inbound-c` and `mcp__junior-brehon__show_task` for #397, #398, #402; Task 4's score = your own runtime.

### Aggregate signal section

```
Total wall-clock: <hours>
Total advisor user-gates: 4 (gate 1 plan approval; gate 4 e2e local-vs-dispatch chosen LOCAL; the DQ #338 pre-push gate; the Task 4 retro gate)
Total catch-fires: 0
Total breaches: 0 (no advisor authoring content; no advisor-attribution in non-advisor commits; no DQ raised without atomic push)
```

### Impl-specific observations

- **Cohort 1 perfect cohort dispatch:** Tasks 1+2 ran in parallel; zero file overlap; YAML overlap check passed at advisor-side dispatch; both completed in ~3 min each (1-line edits); workers pre-pushed; cohort barrier on both validate-pending-laptop result:pass cleared cleanly.
- **Task 3 textbook execution:** worker chose `appended_config_override_takes_effect_returns_429` from the 3 candidate names; mirrored sibling Case A error-shape verbatim; added missing `governance_config` import correctly identified pre-brief; HANDOVER trailer well-formed; 47-line edit (one import + 33-line test fn + 6-line comment swap), no full-file Edit.

## BM signals

### Pre-bm-pr (signal still pending — bm-pr happens AFTER this retro per stage-shape orchestration)

- **`bm-cut` (#393) finalize deleted runlog as index-only file** — captured in DQ #326 (kind:log) + first runlog entry "advisor: re-apply runlog (belt-and-braces — Junior #393 finalize deleted index-only file)". Root cause was Junior's plumbing-path (`git hash-object` + `update-index`) workaround for PostToolUse hook blocking Write on `.claude/runlog/**`; finalize then committed the deletion. Already documented; surface here in case bm-cut behaviour needs revision before next sub-phase.
- **No bm-pr / bm-merge / CR triage signals YET** — those happen post-retro, post-/brehon-verify, pre-merge. If this retro is being read AFTER bm-merge, ignore this caveat and fill in the BM section with bm-pr workflow signals, CR finding counts (4-bucket), and any merge-time issues.

## Lessons promoted this phase

(... 0+ entries; each = full path under `.claude/lessons/feedback_*.md` with the promotion-from-PMD provenance line. If zero entries, write "(none promoted this phase — observations stay in retro recurrence-class section above; promote at v1-federation-inbound-d if patterns persist)" ...)

## Carry-forward to next sub-phase

### v1-federation-inbound-d decision-point (b family in/out)

The (b) Copilot DoS-hardening family was deferred per advisor 2026-05-21 (brief §0.1.1 + §0.2 item #1). Now that (a) reader-side append-history is shipped + e2e-validated, evaluate:

- Is (b) still the next-best-value follow-up? (Weigh against other fed-in-* gaps.)
- If yes, what scope sub-set of (b) is most load-bearing for an MVP-pilot perspective? (Original (b) family enumerated 4+ items; the in-scope cut may be just 1-2.)
- Is (b) decision-blocked on anything else (e.g. pending ADR ratification, pending PRD scope refresh)?

Surface a recommendation (option-a "ship (b) full next sub-phase" / option-b "ship narrow (b) subset" / option-c "defer (b) for a different fed-in-* gap" / option-d "defer (b) until pilot threshold") for advisor + user decision at v1-fed-in-c phase-transition gate.

### Residual conformance-audit gap evaluation (brief §0.2 item #4)

Conformance-audit ship (PR #141 merged into governance-v0 + forward-merged into this phase) installed `#![deny(clippy::disallowed_methods)]` on 3 federation mod roots + `clippy.toml` workspace-allow. Audit found Tier-1 violations only on `lemmy_diesel_utils` (DQ #326 leftover, blocked on DQ #307 fix-impl-2). Evaluate at this retro:

- Did this phase's edits (`order_by` additions + new e2e test) introduce ANY conformance-audit-relevant violations? Walk new code against the audit's axis #1-4 checklists (use the brehon-conformance-audit skill in phase-diff mode).
- Are there residual gaps in coverage (e.g. governance modules not yet under deny-scope)? If yes, list as next-sub-phase candidates.

### DQ #338 daemon-bug structural fix (cross-session)

Track that DQ #338 is in another advisor session's investigation queue. Until it resolves (option-a structural fix to daemon's finalize code path), every impl-task brief that involves finalize-merge MUST carry the pre-push mandate. Document this as a process invariant in the next plan's PRECON enumeration.

### DQ stale leftover (#326 conformance-audit task 8)

DQ #326 (conformance-audit task 8 fail blocked on DQ #307) is stale-pending and was renumbered from #308 during the gov-v0 merge into conformance-audit (not this lane). It re-appeared on our lane after the forward-merge (as the gov-v0 entry, still pending). This is housekeeping — not a fed-in-c regression. Either: (a) the conformance-audit lane's next sub-phase resolves DQ #307 and #326 closes; or (b) we explicitly defer in retro. Recommend (a) — out of scope here.

---
```

The above structure is a SKELETON. The retro planner fills in ALL `(...)` blocks with the actual signals — read every artifact in §3 Required reading + run every `grep`/`rg` named in §3 to populate. Don't paraphrase; cite SHAs / DQ ids / file paths / line numbers.

**Commit message** (exactly): `docs(advisor): v1-federation-inbound-c retro authored`

**HANDOVER trailer in commit body** (per `feedback_handover_trailer_cohort_propagation.md`):

```
HANDOVER:
  filesCreated: [.claude/PRPs/reports/v1-federation-inbound-c-retro.md]
  filesModified: []
  keyDecisions:
    - "promotion recommendations: /tmp path mismatch lesson; lane bootstrap runner script; cross-lane DQ id walk extension"
    - "carry-forward (b) family in/out decision-point surfaced to advisor"
    - "DQ #338 daemon-bug pre-push mandate documented as process invariant until structural fix"
  notes: "Final commit of v1-federation-inbound-c. /brehon-verify runs next (advisor inline); then bm-pr; then CR triage; then bm-merge."
```

**Do NOT** in this task:

- Author content for any `crates/`, `migrations/`, `tests/`, or `docs/brehon-law-inspired-network/` file (advisor never authors content rule + planning role is markdown-only for reports).
- Author content for `.claude/lessons/` files directly. Lessons-promoted ENTRIES go inside the retro body's "Lessons promoted this phase" section as full paths + provenance — but the actual lesson `.md` files (if any new lesson is promoted at retro time per `feedback_one_system_memory_in_repo.md`) lands in the SAME retro commit. Author the new lesson file at the path you cite in the retro body. If no new lesson is promoted, write "(none promoted this phase)".
- Edit `.claude/PRPs/plans/v1-federation-inbound-c.plan.md` (plan is post-write read-only per advisor-orchestrator.md §3.1.1).
- Edit `.claude/decision-queue.json` (you DO NOT need to mutate any DQ entry for Task 4; this retro is a pure markdown authorship task; advisor will write the runlog entry + transition DQ).
- Push to `phase-v1-federation-inbound-c` directly — see §4 PRE-PUSH MANDATE.

## §3 Required reading

In this order:

1. **The plan** — `.claude/PRPs/plans/v1-federation-inbound-c.plan.md` end-to-end. §15.6 + §15.7 + §16a + §17 list the boxes you walk to populate the retro signals.
2. **All briefs authored this phase:**
   - `.claude/PRPs/briefs/v1-federation-inbound-c-bm-cut-1.md` (bm-cut)
   - `.claude/PRPs/briefs/v1-federation-inbound-c-planning-1.md` (the planning brief that produced this plan)
   - `.claude/PRPs/briefs/v1-federation-inbound-c-impl-1.md` (Task 1 brief; canonical impl-1 shape mirrored by 2 and 3)
   - `.claude/PRPs/briefs/v1-federation-inbound-c-impl-2.md` (Task 2 brief)
   - `.claude/PRPs/briefs/v1-federation-inbound-c-impl-3.md` (Task 3 brief — this brief's sibling; carries the DQ #338 PRE-PUSH MANDATE)
3. **Full runlog** — `.claude/runlog/v1-federation-inbound-c-runlog.md`. Read entry-by-entry; cite advisor entries that surface load-bearing observations (e.g. the `/tmp` retro-candidate entry at lines 63-106; the audit pass + submodule init recovery at lines 30-61).
4. **All `feat(fed-in-c):` commits + advisor `chore(advisor):` commits + `chore(decision-queue):` commits + the merge commits** — `git log --oneline phase-v1-federation-inbound-c` from `6dc489c9e` (bm-cut) to HEAD. Note commit subjects + timestamps for the per-task complexity score block.
5. **All resolved + pending DQ entries from this phase** — `python -c "import io,json; d=json.load(io.open('.claude/decision-queue.json',encoding='utf-8')); [print(e['id'], e.get('kind'), e.get('from'), e.get('question','')[:80]) for e in d['pending']+d['resolved'] if 324 <= e['id'] <= 340]"`. Cross-reference with the entry IDs cited in this brief's `related_dq` frontmatter.
6. **All cargo logs from this phase** — `.claude/PRPs/debug/v1-federation-inbound-c-*.log` (per-task check + clippy + test-norun) and `.claude/runlog/e2e-v1-federation-inbound-c-e06e918e2.log` (Phase 2 e2e). Cite test counts + runtime + the 5 ignored TODOs from e2e log.
7. **Junior task records** — `mcp__junior-brehon__show_task` for #393 (bm-cut), `<planning-task-id>` (planning), #397 (Task 1), #398 (Task 2), #402 (Task 3). Cite duration (succeeded-at minus created-at) for the per-task complexity score block.
8. **Daemon reflog for daemon-bug-avoidance evidence** — `ssh homeserver "cd /srv/brehon-fork && git reflog phase-v1-federation-inbound-c -10"`. Cite the 3 reset-to-origin lossless syncs; assert daemon's wrong-ref reset never fired on phase branch.
9. **Lessons (per `.claude/rules/advisor-orchestrator.md` §2.4 file-class table — retro is `.claude/PRPs/reports/*.md`, mandatory injection: none in the table; consult the cross-cutting lessons):**
   - `.claude/lessons/feedback_retro_not_report.md` — **Why:** completion reports list what happened; retros surface what should change. The four H2 sections + carry-forward + lessons-promoted structure is the contract.
   - `.claude/lessons/feedback_four_role_retro_signals.md` — **Why:** each H2 (Advisor / Planning / Impl / BM) must surface signals specific to that role — "did the role honour its file-ownership / autonomy bounds / process invariants?".
   - `.claude/lessons/feedback_retro_task_complexity_score.md` — **Why:** per-task `<files>/<commits>/<runtime-min>/<max-log-silence-min>` is the mechanical signal that feeds resource-budget-pre-queue (lesson) for next sub-phase plan-write.
   - `.claude/lessons/feedback_one_system_memory_in_repo.md` — **Why:** if any NEW lesson is promoted at retro time, it lands in the SAME retro commit (file under `.claude/lessons/feedback_*.md` + index in retro body's "Lessons promoted this phase" section).
   - `.claude/lessons/feedback_dogfood_slash_command_specs.md` — **Why:** plan-write-time dogfood is a planner-role obligation; verify at retro that the dogfood evidence held (e.g. §15 commands all passed against final HEAD).

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-c` (tip `a195421d8` or descendant). Forked at task-spawn time.
- **One commit.** Single retro file + (optionally) a single new lesson file if you promote one inline per `feedback_one_system_memory_in_repo.md`. Do NOT split into multiple commits.
- **CRITICAL — DAEMON-BUG PRE-PUSH MANDATE (per DQ #338 on gov-v0, 2026-05-21T20:16Z):** the Junior daemon's finalize-merge code path is known-buggy as of this task. **To avoid the bug:** after your commit, run `git push origin HEAD:$(git branch --show-current)` to push your worker branch to `origin/junior/...` BEFORE the daemon's finalize step runs. The advisor laptop session will manually finalize-merge from `origin/junior/<branch>`. Do NOT push to `phase-v1-federation-inbound-c` directly.
- Mid-task DQ visibility: if you raise a NEW `pending` entry (e.g. a blocker), commit + push immediately to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility". For this Task 4, you do NOT raise or mutate any DQ entry under normal conditions.
- No `answered_by: "advisor"` or `"user"` from this subagent. If you self-resolve a `kind: "log"` DQ as part of recurrence-class observation harvest, use `answered_by: "planner-self-resolved"`.

### Mutation discipline (no DQ mutations expected for Task 4)

Task 4 is a pure markdown authorship task. The only valid DQ writes from this subagent are: (a) `kind: "log"` entries you choose to file as planner observations (e.g. if you spot a recurrence class that warrants explicit DQ tracking beyond the retro body itself), or (b) `kind: "blocker"` entries if you cannot complete authorship (file ownership block, missing required reading, ambiguity in the plan's §13 Task 4 spec). NO `validate-pending-laptop` writes (this task has no cargo DoD).

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If you need to write the retro file and the Claude Code sensitive-file gate blocks it: (a) write the retro content to `TASK4_RETRO.md` at worktree root, (b) write a short `TASK4_ESCALATION.md` naming the issue, (c) commit both + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`.

### Task-4 GOTCHAs (from plan §13 Task 4 + retro-authoring lessons)

- **Retro, not report.** Completion reports list what happened (already in runlog); retros surface what should change. Each section's job is to identify recurrence classes + promotion candidates + carry-forward items — not to recapitulate the runlog.
- **Per-role signals — not generic process narration.** The four H2 sections each ask a role-specific question (per `feedback_four_role_retro_signals.md`). Advisor: did pre-reservation + canonical-schema-first + dogfood gates hold? Planning: did the plan's DoD smoke + watchpoint specificity + cohort YAML guide impl correctly? Impl: per-task complexity + cohort dispatch evidence + mirror-ref discipline. BM: bm-cut sequencing + bm-pr/bm-merge if already run.
- **Per-task complexity score is MECHANICAL — not subjective.** Format `<files>/<commits>/<runtime-min>/<max-log-silence-min>`. Source from `git log` (files + commits) + `mcp__junior-brehon__show_task` (runtime) + worker log mtime gaps (silence). Do NOT estimate; cite the numbers.
- **Lessons-promoted section can be empty.** If no new lesson is promoted, write "(none promoted this phase — observations stay in retro recurrence-class section above; promote at v1-federation-inbound-d if patterns persist)". An empty retro on lessons is honest; a falsely-pre-promoted lesson is process noise.
- **Carry-forward must be ACTIONABLE.** Each carry-forward item names a follow-up sub-phase candidate + a decision-point the advisor will surface to user at phase-transition gate. Vague "follow up on X" is process noise; "v1-fed-in-d decision-point: ship (b) family full / narrow / defer (a/b/c options)" is actionable.

## §5 Validation gates

This is a markdown authorship task — no cargo DoD. Validation is structural:

```bash
# Structural sanity post-write:
test -f .claude/PRPs/reports/v1-federation-inbound-c-retro.md
grep -c '^## ' .claude/PRPs/reports/v1-federation-inbound-c-retro.md  # EXPECT >= 4 (Advisor / Planning / Impl / BM minimum)
grep -c '^### Recurrence-class observations' .claude/PRPs/reports/v1-federation-inbound-c-retro.md  # EXPECT 1
grep -c 'PROMOTION RECOMMENDATION' .claude/PRPs/reports/v1-federation-inbound-c-retro.md  # EXPECT >= 0 (0 if no promotions surfaced; 1+ if recurrence-class observations name candidates)
grep -c '^## Carry-forward' .claude/PRPs/reports/v1-federation-inbound-c-retro.md  # EXPECT 1
```

NO `validate-pending-laptop` DQ — pure markdown authorship has no cargo gate. The /brehon-verify Story 3 checkpoint command (per plan §16a Story 3) will run AFTER your worker commit + pre-push + advisor finalize-merge:

```bash
test -f .claude/PRPs/reports/v1-federation-inbound-c-retro.md && grep -c '^## ' .claude/PRPs/reports/v1-federation-inbound-c-retro.md
# EXPECT exit 0 + count >= 4
```

## §6 Expected output (return to advisor)

```
## Task 4 complete — v1-federation-inbound-c retro authored

**Commit:** <sha> on <worker-branch>
**Files created:** .claude/PRPs/reports/v1-federation-inbound-c-retro.md
**Files modified:** (none — pure authorship)
**Retro structure verified:**
  - H2 sections: 4 (Advisor / Planning / Impl / BM) + Aggregate + Lessons promoted + Carry-forward
  - Recurrence-class observations: <N> items in Advisor H2 (with PROMOTION RECOMMENDATION lines)
  - Per-task complexity scores: 4 (Tasks 1, 2, 3, 4-this-retro)
  - Carry-forward: (b) family decision-point + residual conformance-audit gap eval + DQ #338 invariant + DQ #326 stale housekeeping
**Lessons promoted this phase:** <count + paths, or "(none promoted this phase)">
**Worker branch pushed:** YES (DQ #338 daemon-bug mitigation per §4)
**Next:** advisor manually finalize-merges from origin/<worker-branch>; /brehon-verify Story 3 checkpoint; user gate 6 (retro sign-off); bm-pr; CR triage; bm-merge; /brehon-phase-transition.
```

Plus any `kind: "log"` DQ #N references if you self-resolved planner observations.

## §7 Why this brief differs from the plan

It does not — Task 4's scope is exactly plan §13 Task 4 (lines 611-636). This brief adds only:

(a) §0 forbidden-window self-check wording + DAEMON-BUG awareness (per DQ #338 on gov-v0).
(b) §2 explicit SKELETON for the retro structure including pre-filled recurrence-class observations the advisor has been tracking through this phase (`/tmp` recurrence, lane bootstrap submodule miss, DQ id-collision, DQ #338 avoidance evidence). These are NOT pre-empting the planner's role — they're observations the advisor already documented in runlog + DQ; the planner reads + verifies + writes them up. The planner MUST validate every cite before propagating (e.g. read the runlog entry at lines 63-106 verbatim; confirm the recurrence count; identify any prior occurrences from PMD search).
(c) §3 explicit required reading list (8 items + lessons).
(d) §4 PRE-PUSH MANDATE (per DQ #338 daemon wrong-ref reset bug — explicit `git push origin HEAD:$(git branch --show-current)` after commit, advisor manually finalize-merges) + harness-gap interim escalation path.
(e) §5 STRUCTURAL validation gates (no cargo — this is markdown authorship).
(f) Brief carries DAEMON-BUG awareness explicitly because the brief itself is the LAST major Junior task before bm-pr / bm-merge; if the daemon refires the wrong-ref bug here, recovery is more complex than a simple cherry-pick.

The retro CONTENT (the actual observations + signals + scores) is the planner's authorship — this brief provides skeleton + structural guarantees, not pre-filled content. The advisor cannot author retro content per CLAUDE.md four-role model.

---
